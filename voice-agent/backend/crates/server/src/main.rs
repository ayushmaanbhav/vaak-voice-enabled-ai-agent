//! Voice Agent Server Entry Point

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

use voice_agent_config::{load_settings, DomainConfigManager, MasterDomainConfig, Settings};
use voice_agent_server::{create_router, init_metrics, session::ScyllaSessionStore, AppState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // P0 FIX: Load configuration from files and environment
    // Priority: env vars > config/{env}.yaml > config/default.yaml > defaults
    let env = std::env::var("VOICE_AGENT_ENV").ok();
    let config = match load_settings(env.as_deref()) {
        Ok(settings) => {
            // Tracing not yet initialized, use eprintln for early logging
            eprintln!(
                "Loaded configuration from files (env: {})",
                env.as_deref().unwrap_or("default")
            );
            settings
        }
        Err(e) => {
            eprintln!(
                "Warning: Failed to load config: {}. Using defaults.",
                e
            );
            Settings::default()
        }
    };

    // P5 FIX: Initialize tracing with optional OpenTelemetry
    init_tracing(&config);

    tracing::info!("Starting Voice Agent Server v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!(
        environment = ?config.environment,
        config_path = env.as_deref().unwrap_or("default"),
        "Configuration loaded"
    );

    // P4 FIX: Load domain configuration (legacy)
    let domain_config = load_domain_config(&config.domain_config_path);
    tracing::info!("Loaded domain configuration");

    // P5 FIX: Load hierarchical domain configuration (new)
    let master_domain_config = load_master_domain_config("config");
    tracing::info!(
        domain = %master_domain_config.domain_id,
        "Loaded hierarchical domain configuration"
    );

    // P0 FIX: Initialize Prometheus metrics
    let _metrics_handle = init_metrics();
    tracing::info!("Initialized Prometheus metrics at /metrics");

    // P0 FIX: Optionally initialize ScyllaDB persistence
    let mut state = if config.persistence.enabled {
        tracing::info!("Initializing ScyllaDB persistence layer...");
        match init_persistence(&config).await {
            Ok(persistence) => {
                tracing::info!(
                    hosts = ?config.persistence.scylla_hosts,
                    keyspace = %config.persistence.keyspace,
                    "ScyllaDB persistence initialized"
                );
                let scylla_store = ScyllaSessionStore::new(persistence.sessions);
                // P2 FIX: Wire audit logging for RBI compliance
                let audit_log: Arc<dyn voice_agent_persistence::AuditLog> =
                    Arc::new(persistence.audit);
                // P1-4 FIX: Wire SMS and GoldPrice services into tools
                let sms_service: Arc<dyn voice_agent_persistence::SmsService> =
                    Arc::new(persistence.sms);
                let gold_price_service: Arc<dyn voice_agent_persistence::GoldPriceService> =
                    Arc::new(persistence.gold_price);
                tracing::info!("SMS and GoldPrice services wired into tools");
                AppState::with_full_persistence(
                    config.clone(),
                    Arc::new(scylla_store),
                    domain_config,
                    sms_service,
                    gold_price_service,
                )
                .with_audit_logger(audit_log)
            },
            Err(e) => {
                tracing::error!(
                    "Failed to initialize ScyllaDB: {}. Falling back to in-memory.",
                    e
                );
                AppState::with_domain_config(config.clone(), domain_config)
            },
        }
    } else {
        tracing::info!("Persistence disabled, using in-memory session store");
        AppState::with_domain_config(config.clone(), domain_config)
    };

    // P0 FIX: Optionally initialize VectorStore for RAG
    if config.rag.enabled {
        tracing::info!("Initializing VectorStore for RAG...");
        match init_vector_store(&config).await {
            Ok(vs) => {
                tracing::info!(
                    endpoint = %config.rag.qdrant_endpoint,
                    collection = %config.rag.qdrant_collection,
                    "VectorStore initialized for RAG"
                );
                state = state.with_vector_store(Arc::new(vs));
            },
            Err(e) => {
                tracing::warn!(
                    "Failed to initialize VectorStore: {}. RAG will be disabled.",
                    e
                );
            },
        }
    }

    tracing::info!(
        distributed = state.is_distributed_sessions(),
        rag_enabled = state.vector_store.is_some(),
        "Initialized application state"
    );

    // P2 FIX: Attempt to recover sessions from previous run
    if state.is_distributed_sessions() {
        match state.recover_sessions().await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!(recovered = count, "Session recovery complete");
                }
            },
            Err(e) => {
                tracing::warn!(error = %e, "Session recovery failed (non-fatal)");
            },
        }
    }

    // Create router
    let app = create_router(state);

    // Bind address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Listening on {}", addr);

    // Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // P1 FIX: Graceful shutdown on SIGTERM/SIGINT
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}

/// Wait for shutdown signal (Ctrl+C or SIGTERM)
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, initiating graceful shutdown...");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM, initiating graceful shutdown...");
        }
    }
}

/// Initialize tracing (with optional OpenTelemetry when feature enabled)
#[cfg(feature = "telemetry")]
fn init_tracing(config: &Settings) {
    use opentelemetry_otlp::WithExportConfig;

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let level = &config.observability.log_level;
        format!("voice_agent={},tower_http=debug", level).into()
    });

    let subscriber = tracing_subscriber::registry().with(env_filter);
    let fmt_layer = if config.observability.log_json {
        tracing_subscriber::fmt::layer().json().boxed()
    } else {
        tracing_subscriber::fmt::layer().boxed()
    };

    if let Some(otlp_endpoint) = &config.observability.otlp_endpoint {
        if config.observability.tracing_enabled {
            match opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(
                    opentelemetry_otlp::new_exporter()
                        .tonic()
                        .with_endpoint(otlp_endpoint),
                )
                .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(
                    opentelemetry_sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new("service.name", "voice-agent"),
                        opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                    ]),
                ))
                .install_batch(opentelemetry_sdk::runtime::Tokio)
            {
                Ok(tracer) => {
                    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
                    subscriber.with(fmt_layer).with(otel_layer).init();
                    tracing::info!(endpoint = %otlp_endpoint, "OpenTelemetry tracing enabled");
                    return;
                },
                Err(e) => eprintln!("Failed to initialize OpenTelemetry: {}. Falling back.", e),
            }
        }
    }
    subscriber.with(fmt_layer).init();
}

/// Initialize tracing (console only - telemetry feature disabled)
#[cfg(not(feature = "telemetry"))]
fn init_tracing(config: &Settings) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let level = &config.observability.log_level;
        format!("voice_agent={},tower_http=debug", level).into()
    });

    let subscriber = tracing_subscriber::registry().with(env_filter);
    let fmt_layer = if config.observability.log_json {
        tracing_subscriber::fmt::layer().json().boxed()
    } else {
        tracing_subscriber::fmt::layer().boxed()
    };
    subscriber.with(fmt_layer).init();
}

/// P0 FIX: Initialize ScyllaDB persistence layer
async fn init_persistence(
    config: &Settings,
) -> Result<voice_agent_persistence::PersistenceLayer, voice_agent_persistence::PersistenceError> {
    let scylla_config = voice_agent_persistence::ScyllaConfig {
        hosts: config.persistence.scylla_hosts.clone(),
        keyspace: config.persistence.keyspace.clone(),
        replication_factor: config.persistence.replication_factor,
    };
    voice_agent_persistence::init(scylla_config).await
}

/// P0 FIX: Initialize VectorStore for RAG retrieval
async fn init_vector_store(
    config: &Settings,
) -> Result<voice_agent_rag::VectorStore, voice_agent_rag::RagError> {
    let vs_config = voice_agent_rag::VectorStoreConfig {
        endpoint: config.rag.qdrant_endpoint.clone(),
        collection: config.rag.qdrant_collection.clone(),
        vector_dim: config.rag.vector_dim,
        distance: voice_agent_rag::VectorDistance::Cosine,
        api_key: config.rag.qdrant_api_key.clone(),
    };
    let store = voice_agent_rag::VectorStore::new(vs_config).await?;
    store.ensure_collection().await?;
    Ok(store)
}

/// P4 FIX: Load domain configuration from file
///
/// Attempts to load from the specified path. Falls back to defaults if file not found.
fn load_domain_config(path: &str) -> DomainConfigManager {
    let path = Path::new(path);

    if path.exists() {
        match DomainConfigManager::from_file(path) {
            Ok(manager) => {
                tracing::info!("Domain config loaded from: {}", path.display());

                // Validate the loaded config
                let config = manager.get();
                if let Err(errors) = config.validate() {
                    tracing::warn!("Domain config validation warnings: {:?}", errors);
                }

                manager
            },
            Err(e) => {
                tracing::warn!(
                    "Failed to load domain config from {}: {}. Using defaults.",
                    path.display(),
                    e
                );
                DomainConfigManager::new()
            },
        }
    } else {
        tracing::info!(
            "Domain config not found at {}. Using defaults.",
            path.display()
        );
        DomainConfigManager::new()
    }
}

/// P5 FIX: Load hierarchical domain configuration from YAML files
///
/// Loads the new MasterDomainConfig from config/domains/{domain_id}/ directory.
/// This provides the hierarchical config structure for domain abstraction.
fn load_master_domain_config(config_dir: &str) -> Arc<MasterDomainConfig> {
    let domain_id = std::env::var("DOMAIN_ID").unwrap_or_else(|_| "gold_loan".to_string());
    let config_path = Path::new(config_dir);

    match MasterDomainConfig::load(&domain_id, config_path) {
        Ok(config) => {
            tracing::info!(
                domain_id = %config.domain_id,
                display_name = %config.display_name,
                slots_count = config.slots.slots.len(),
                stages_count = config.stages.stages.len(),
                "Loaded hierarchical domain configuration"
            );
            Arc::new(config)
        }
        Err(e) => {
            tracing::warn!(
                domain_id = %domain_id,
                error = %e,
                "Failed to load hierarchical domain config. Using defaults."
            );
            Arc::new(MasterDomainConfig::default())
        }
    }
}
