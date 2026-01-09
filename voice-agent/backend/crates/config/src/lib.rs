//! Configuration management for the voice agent
//!
//! Supports loading configuration from:
//! - YAML/TOML files
//! - Environment variables (VOICE_AGENT_ prefix)
//! - Runtime overrides
//!
//! # Domain Configuration
//!
//! All domain-specific configuration now lives in config/domains/{domain}/:
//! - domain.yaml - Core rates, competitors, constants
//! - branches.yaml - Branch locations
//! - competitors.yaml - Competitor details
//! - objections.yaml - Objection handling
//! - prompts.yaml - Prompt templates
//! - etc.
//!
//! Access via MasterDomainConfig and crate-specific views:
//! - AgentDomainView for agent crate
//! - LlmDomainView for llm crate
//! - ToolsDomainView for tools crate

pub mod agent;
// P1 FIX: Centralized constants module
pub mod constants;
// P13 FIX: All domain config now in domain/ submodule (YAML-driven)
pub mod domain;
pub mod pipeline;
pub mod settings;

pub use agent::{AgentConfig, MemoryConfig, PersonaConfig};
pub use pipeline::PipelineConfig;
pub use settings::{
    load_settings, AuthConfig, PersistenceConfig, RagConfig, RateLimitConfig, RuntimeEnvironment,
    ServerConfig, Settings, TurnServerConfig,
};

// P13 FIX: Domain configuration via MasterDomainConfig + views
pub use domain::{
    MasterDomainConfig,
    // Sub-config types
    BranchDefaults, BranchEntry, BranchesConfig,
    ComparisonPoint, CompetitorDefaults, CompetitorEntry,
    CompetitorsConfig, NumericThreshold, ObjectionDefinition, ObjectionResponse, ObjectionsConfig,
    PromptsConfig, QualificationThresholds, ScoringConfig, SegmentDefinition, SegmentDetection,
    SegmentsConfig, SlotDefinition, SlotsConfig, SmsTemplatesConfig, StageDefinition, StagesConfig,
    ToolParameter, ToolSchema, ToolsConfig,
    // Goals and action templates (domain-agnostic action instructions)
    ActionContext, ActionTemplate, ActionTemplatesConfig, GoalEntry, GoalsConfig,
    // View types
    AgentDomainView, CompetitorInfo, LlmDomainView, MonthlySavings, ToolsDomainView,
};

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
