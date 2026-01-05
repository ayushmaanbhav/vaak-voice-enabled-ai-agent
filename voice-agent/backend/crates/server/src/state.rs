//! Application State
//!
//! Shared state across all handlers.

use parking_lot::RwLock;
use std::sync::Arc;

use voice_agent_config::{load_settings, DomainConfigManager, Settings};
use voice_agent_rag::VectorStore;
use voice_agent_tools::ToolRegistry;
// P2 FIX: Text processing pipeline for grammar, PII, compliance
use voice_agent_text_processing::{TextProcessingConfig, TextProcessingPipeline, TextSimplifier};
// Deterministic phonetic error correction
use voice_agent_text_processing::grammar::PhoneticCorrector;
// Translation
use voice_agent_text_processing::translation::{TranslationConfig, create_translator};
use voice_agent_core::Translator;
// P2 FIX: Audit logging for RBI compliance
use voice_agent_persistence::{AuditLog, AuditLogger};

use crate::session::{InMemorySessionStore, SessionManager, SessionStore};

/// Application state
#[derive(Clone)]
pub struct AppState {
    /// P1 FIX: Configuration wrapped in RwLock for hot-reload support
    pub config: Arc<RwLock<Settings>>,
    /// P4 FIX: Domain configuration manager for gold loan specific config
    pub domain_config: Arc<DomainConfigManager>,
    /// Session manager
    pub sessions: Arc<SessionManager>,
    /// Tool registry
    pub tools: Arc<ToolRegistry>,
    /// P2-3 FIX: Session store for persistence (ScyllaDB or in-memory)
    pub session_store: Arc<dyn SessionStore>,
    /// P0 FIX: Vector store for RAG retrieval (optional - initialized if Qdrant is available)
    pub vector_store: Option<Arc<VectorStore>>,
    /// P2 FIX: Text processing pipeline for grammar, PII, compliance
    pub text_processing: Arc<TextProcessingPipeline>,
    /// P2 FIX: Text simplifier for TTS output (numbers, abbreviations)
    pub text_simplifier: Arc<TextSimplifier>,
    /// Deterministic phonetic corrector for ASR errors
    pub phonetic_corrector: Arc<PhoneticCorrector>,
    /// Translator for language conversion
    pub translator: Arc<dyn Translator>,
    /// P2 FIX: Audit logger for RBI compliance (wrapped in Arc for Clone)
    pub audit_logger: Option<Arc<AuditLogger>>,
    /// Environment name for config reload
    env: Option<String>,
}

impl AppState {
    /// Create default text processing components, phonetic corrector, and translator
    fn create_text_processing() -> (Arc<TextProcessingPipeline>, Arc<TextSimplifier>, Arc<PhoneticCorrector>, Arc<dyn Translator>) {
        let text_config = TextProcessingConfig::default();
        let text_processing = Arc::new(TextProcessingPipeline::new(text_config, None));
        let text_simplifier = Arc::new(TextSimplifier::default_config());
        // Deterministic phonetic corrector for gold loan domain
        let phonetic_corrector = Arc::new(PhoneticCorrector::gold_loan());
        tracing::info!("Initialized deterministic phonetic corrector for gold_loan domain");
        // Translator for language conversion
        let translator = create_translator(&TranslationConfig::default());
        (text_processing, text_simplifier, phonetic_corrector, translator)
    }

    /// Create new application state with in-memory session store
    pub fn new(config: Settings) -> Self {
        let (text_processing, text_simplifier, phonetic_corrector, translator) = Self::create_text_processing();
        Self {
            config: Arc::new(RwLock::new(config)),
            domain_config: Arc::new(DomainConfigManager::new()),
            sessions: Arc::new(SessionManager::new(100)),
            tools: Arc::new(voice_agent_tools::registry::create_default_registry()),
            session_store: Arc::new(InMemorySessionStore::new()),
            vector_store: None,
            text_processing,
            text_simplifier,
            phonetic_corrector,
            translator,
            audit_logger: None,
            env: None,
        }
    }

    /// P4 FIX: Create new application state with domain config
    pub fn with_domain_config(config: Settings, domain_config: DomainConfigManager) -> Self {
        let (text_processing, text_simplifier, phonetic_corrector, translator) = Self::create_text_processing();
        Self {
            config: Arc::new(RwLock::new(config)),
            domain_config: Arc::new(domain_config),
            sessions: Arc::new(SessionManager::new(100)),
            tools: Arc::new(voice_agent_tools::registry::create_default_registry()),
            session_store: Arc::new(InMemorySessionStore::new()),
            vector_store: None,
            text_processing,
            text_simplifier,
            phonetic_corrector,
            translator,
            audit_logger: None,
            env: None,
        }
    }

    /// Create new application state with environment name for reload support
    pub fn with_env(config: Settings, env: Option<String>) -> Self {
        let (text_processing, text_simplifier, phonetic_corrector, translator) = Self::create_text_processing();
        Self {
            config: Arc::new(RwLock::new(config)),
            domain_config: Arc::new(DomainConfigManager::new()),
            sessions: Arc::new(SessionManager::new(100)),
            tools: Arc::new(voice_agent_tools::registry::create_default_registry()),
            session_store: Arc::new(InMemorySessionStore::new()),
            vector_store: None,
            text_processing,
            text_simplifier,
            phonetic_corrector,
            translator,
            audit_logger: None,
            env,
        }
    }

    /// P2-3 FIX: Create application state with custom session store (e.g., ScyllaDB)
    pub fn with_session_store(config: Settings, store: Arc<dyn SessionStore>) -> Self {
        let (text_processing, text_simplifier, phonetic_corrector, translator) = Self::create_text_processing();
        Self {
            config: Arc::new(RwLock::new(config)),
            domain_config: Arc::new(DomainConfigManager::new()),
            sessions: Arc::new(SessionManager::new(100)),
            tools: Arc::new(voice_agent_tools::registry::create_default_registry()),
            session_store: store,
            vector_store: None,
            text_processing,
            text_simplifier,
            phonetic_corrector,
            translator,
            audit_logger: None,
            env: None,
        }
    }

    /// P0 FIX: Create application state with custom session store AND domain config
    pub fn with_session_store_and_domain(
        config: Settings,
        store: Arc<dyn SessionStore>,
        domain_config: DomainConfigManager,
    ) -> Self {
        let (text_processing, text_simplifier, phonetic_corrector, translator) = Self::create_text_processing();
        Self {
            config: Arc::new(RwLock::new(config)),
            domain_config: Arc::new(domain_config),
            sessions: Arc::new(SessionManager::new(100)),
            tools: Arc::new(voice_agent_tools::registry::create_default_registry()),
            session_store: store,
            vector_store: None,
            text_processing,
            text_simplifier,
            phonetic_corrector,
            translator,
            audit_logger: None,
            env: None,
        }
    }

    /// P1-4 FIX: Create application state with full persistence layer (SMS, GoldPrice wired)
    ///
    /// This method wires the SMS and GoldPrice services from the persistence layer
    /// into the tool registry, enabling proper persistence of SMS messages and
    /// gold price queries to ScyllaDB.
    ///
    /// P2-1 FIX: Also wires domain config (GoldLoanConfig) to tools for business logic.
    pub fn with_full_persistence(
        config: Settings,
        store: Arc<dyn SessionStore>,
        domain_config: DomainConfigManager,
        sms_service: Arc<dyn voice_agent_persistence::SmsService>,
        gold_price_service: Arc<dyn voice_agent_persistence::GoldPriceService>,
    ) -> Self {
        let (text_processing, text_simplifier, phonetic_corrector, translator) = Self::create_text_processing();

        // P2-1 FIX: Extract gold loan config for tools (rates, LTV, etc.)
        let gold_loan_config = domain_config.gold_loan();

        // P1-4 FIX: Create tool registry with persistence services wired
        // P2-1 FIX: Also wire domain config for business logic
        let integration_config = voice_agent_tools::FullIntegrationConfig::default()
            .with_sms_service(sms_service)
            .with_gold_price_service(gold_price_service)
            .with_gold_loan_config(gold_loan_config);
        let tools = voice_agent_tools::create_registry_with_persistence(integration_config);

        Self {
            config: Arc::new(RwLock::new(config)),
            domain_config: Arc::new(domain_config),
            sessions: Arc::new(SessionManager::new(100)),
            tools: Arc::new(tools),
            session_store: store,
            vector_store: None,
            text_processing,
            text_simplifier,
            phonetic_corrector,
            translator,
            audit_logger: None,
            env: None,
        }
    }

    /// P0 FIX: Set vector store for RAG retrieval
    pub fn with_vector_store(mut self, vector_store: Arc<VectorStore>) -> Self {
        self.vector_store = Some(vector_store);
        self
    }

    /// P2 FIX: Set audit logger for RBI compliance logging
    pub fn with_audit_logger(mut self, audit_log: Arc<dyn AuditLog>) -> Self {
        self.audit_logger = Some(Arc::new(AuditLogger::new(audit_log)));
        self
    }

    /// P2 FIX: Log an audit event for RBI compliance
    ///
    /// Returns Ok(()) if logger is not configured (noop).
    pub async fn log_conversation_start(
        &self,
        session_id: &str,
        language: &str,
    ) -> Result<(), crate::ServerError> {
        if let Some(ref logger) = self.audit_logger {
            logger
                .log_conversation_start(session_id, language)
                .await
                .map_err(|e| crate::ServerError::Persistence(e.to_string()))?;
        }
        Ok(())
    }

    /// P2 FIX: Log conversation end
    pub async fn log_conversation_end(
        &self,
        session_id: &str,
        reason: &str,
        duration_secs: u64,
    ) -> Result<(), crate::ServerError> {
        if let Some(ref logger) = self.audit_logger {
            logger
                .log_conversation_end(session_id, reason, duration_secs)
                .await
                .map_err(|e| crate::ServerError::Persistence(e.to_string()))?;
        }
        Ok(())
    }

    /// P1 FIX: Reload configuration from files
    ///
    /// Reloads config from disk and updates the shared state.
    /// Returns the new config on success.
    pub fn reload_config(&self) -> Result<(), String> {
        let new_config = load_settings(self.env.as_deref())
            .map_err(|e| format!("Failed to reload config: {}", e))?;

        // Update the config
        let mut config = self.config.write();
        *config = new_config;

        tracing::info!("Configuration reloaded successfully");
        Ok(())
    }

    /// P4 FIX: Reload domain configuration
    pub fn reload_domain_config(&self) -> Result<(), String> {
        self.domain_config
            .reload()
            .map_err(|e| format!("Failed to reload domain config: {}", e))?;

        tracing::info!("Domain configuration reloaded successfully");
        Ok(())
    }

    /// Get a read guard to the current configuration
    pub fn get_config(&self) -> parking_lot::RwLockReadGuard<'_, Settings> {
        self.config.read()
    }

    /// P4 FIX: Get domain configuration manager
    pub fn get_domain_config(&self) -> &DomainConfigManager {
        &self.domain_config
    }

    /// P2-3 FIX: Persist session metadata to the configured store
    ///
    /// Call this after creating a session or when session state changes
    /// that should be persisted (e.g., stage transitions, turn completion).
    pub async fn persist_session(
        &self,
        session: &crate::session::Session,
    ) -> Result<(), crate::ServerError> {
        self.session_store.store_metadata(session).await
    }

    /// P2-3 FIX: Check if session persistence is distributed (ScyllaDB/Redis)
    pub fn is_distributed_sessions(&self) -> bool {
        self.session_store.is_distributed()
    }

    /// P2 FIX: Recover active sessions on server restart
    ///
    /// Loads session metadata from persistent storage and logs recoverable sessions.
    /// Note: Full agent state recovery requires conversation history serialization
    /// which is not implemented. This method provides visibility into sessions
    /// that were active before restart.
    ///
    /// Returns the count of sessions found (not fully restored).
    pub async fn recover_sessions(&self) -> Result<usize, crate::ServerError> {
        if !self.is_distributed_sessions() {
            tracing::debug!("Session recovery skipped: not using distributed session store");
            return Ok(0);
        }

        match self.session_store.list_active_sessions(100).await {
            Ok(sessions) => {
                let now = chrono::Utc::now();
                let active_sessions: Vec<_> = sessions
                    .into_iter()
                    .filter(|s| s.expires_at > now)
                    .collect();

                if active_sessions.is_empty() {
                    tracing::info!("No active sessions to recover");
                } else {
                    tracing::info!(
                        count = active_sessions.len(),
                        "Found recoverable sessions from previous run"
                    );

                    // Log details of each recoverable session
                    for session in &active_sessions {
                        tracing::info!(
                            session_id = %session.session_id,
                            stage = %session.conversation_stage,
                            turn_count = session.turn_count,
                            language = %session.language,
                            age_minutes = (now - session.created_at).num_minutes(),
                            "Recoverable session found"
                        );
                    }
                }

                Ok(active_sessions.len())
            },
            Err(e) => {
                tracing::warn!(error = %e, "Failed to query active sessions for recovery");
                Err(crate::ServerError::Persistence(e.to_string()))
            },
        }
    }
}
