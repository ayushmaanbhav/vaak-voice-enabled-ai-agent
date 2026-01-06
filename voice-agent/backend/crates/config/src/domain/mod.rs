//! Hierarchical Domain Configuration
//!
//! Provides a layered configuration system:
//! 1. Base config (config/base/defaults.yaml)
//! 2. Domain config (config/domains/{domain}/domain.yaml)
//! 3. Runtime overrides (per-session)
//!
//! Each crate accesses config through a specific "view" that translates
//! raw config into crate-specific terminology.

mod master;
mod objections;
mod prompts;
mod scoring;
mod slots;
mod stages;
mod tools;
mod views;

pub use master::MasterDomainConfig;
pub use objections::{
    ObjectionDefinition, ObjectionResponse, ObjectionsConfig, ObjectionsConfigError,
};
pub use prompts::{PromptsConfig, PromptsConfigError};
pub use scoring::{
    CategoryWeights, ConversionMultipliers, EscalationConfig, QualificationThresholds,
    ScoringConfig, ScoringConfigError, TrustScores,
};
pub use slots::{
    EnumValue, GoalDefinition, SlotDefinition, SlotType, SlotsConfig, SlotsConfigError,
};
pub use stages::{
    StageDefinition, StageRequirements, StagesConfig, StagesConfigError, TransitionTrigger,
};
pub use tools::{ToolParameter, ToolSchema, ToolsConfig, ToolsConfigError};
pub use views::{AgentDomainView, CompetitorInfo, LlmDomainView, ToolsDomainView};

// Re-export legacy DomainConfig for backward compatibility
pub use crate::domain_config::{
    domain_config, init_domain_config, DomainConfig, DomainConfigManager,
};
