//! Configuration management for the voice agent
//!
//! Supports loading configuration from:
//! - YAML/TOML files
//! - Environment variables (VOICE_AGENT_ prefix)
//! - Runtime overrides

pub mod settings;
pub mod pipeline;
pub mod agent;
pub mod gold_loan;

pub use settings::{Settings, ServerConfig, RateLimitConfig, load_settings};
pub use pipeline::PipelineConfig;
pub use agent::{AgentConfig, PersonaConfig};  // P0 FIX: Export PersonaConfig for consolidation
pub use gold_loan::{GoldLoanConfig, PurityFactors, CompetitorRates};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid value for {field}: {message}")]
    InvalidValue { field: String, message: String },

    #[error("Environment error: {0}")]
    Environment(String),
}

impl From<config::ConfigError> for ConfigError {
    fn from(err: config::ConfigError) -> Self {
        ConfigError::ParseError(err.to_string())
    }
}
