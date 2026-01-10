//! ScyllaDB persistence layer for voice-agent-rust
//!
//! Provides persistent storage for:
//! - Sessions (replaces Redis stub)
//! - SMS messages (simulated, persisted for audit)
//! - Gold prices (simulated with realistic fluctuation)
//! - Appointments
//! - Audit logging (P0 FIX: RBI compliance)

pub mod appointments;
pub mod audit;
pub mod client;
pub mod error;
pub mod gold_price;
pub mod schema;
pub mod sessions;
pub mod sms;

pub use appointments::{Appointment, AppointmentStatus, AppointmentStore, ScyllaAppointmentStore};
pub use audit::{
    Actor, AuditEntry, AuditEventType, AuditLog, AuditLogger, AuditOutcome, AuditQuery,
    ScyllaAuditLog,
};
pub use client::{ScyllaClient, ScyllaConfig};
pub use error::PersistenceError;
// Asset price types (domain-agnostic)
pub use gold_price::{AssetPrice, AssetPriceService, SimulatedAssetPriceService, TierDefinition};
pub use sessions::{ScyllaSessionStore, SessionData, SessionStore};
pub use sms::{SimulatedSmsService, SmsMessage, SmsService, SmsStatus, SmsType};

/// Initialize the persistence layer with ScyllaDB and domain-specific tiers
///
/// # Arguments
/// * `config` - ScyllaDB configuration
/// * `base_price` - Base asset price per unit
/// * `tiers` - Tier definitions from domain config (e.g., from ToolsDomainView::quality_tiers_full())
pub async fn init(
    config: ScyllaConfig,
    base_price: f64,
    tiers: Vec<TierDefinition>,
) -> Result<PersistenceLayer, PersistenceError> {
    let client = ScyllaClient::connect(config).await?;
    client.ensure_schema().await?;

    Ok(PersistenceLayer {
        sessions: ScyllaSessionStore::new(client.clone()),
        sms: SimulatedSmsService::new(client.clone()),
        asset_price: SimulatedAssetPriceService::new(client.clone(), base_price, tiers),
        appointments: ScyllaAppointmentStore::new(client.clone()),
        audit: ScyllaAuditLog::new(client),
    })
}


/// Combined persistence layer with all services
pub struct PersistenceLayer {
    pub sessions: ScyllaSessionStore,
    pub sms: SimulatedSmsService,
    /// Asset price service with config-driven tier support
    pub asset_price: SimulatedAssetPriceService,
    pub appointments: ScyllaAppointmentStore,
    /// Audit logging for compliance
    pub audit: ScyllaAuditLog,
}

