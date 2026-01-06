//! Hierarchical Domain Configuration
//!
//! Provides a layered configuration system:
//! 1. Base config (config/base/defaults.yaml)
//! 2. Domain config (config/domains/{domain}/domain.yaml)
//! 3. Runtime overrides (per-session)
//!
//! Each crate accesses config through a specific "view" that translates
//! raw config into crate-specific terminology.

mod branches;
mod competitors;
mod master;
mod objections;
mod prompts;
mod scoring;
mod segments;
mod slots;
mod sms_templates;
mod stages;
mod tools;
mod views;

pub use branches::{BranchDefaults, BranchEntry, BranchesConfig, BranchesConfigError, DoorstepServiceConfig};
pub use competitors::{
    ComparisonPoint, CompetitorDefaults, CompetitorEntry, CompetitorsConfig,
    CompetitorsConfigError, RateRange,
};
pub use master::MasterDomainConfig;
pub use objections::{
    ObjectionDefinition, ObjectionResponse, ObjectionsConfig, ObjectionsConfigError,
};
pub use prompts::{PromptsConfig, PromptsConfigError};
pub use scoring::{
    CategoryWeights, ConversionMultipliers, EscalationConfig, QualificationThresholds,
    ScoringConfig, ScoringConfigError, TrustScores,
};
pub use segments::{
    NumericThreshold, SegmentDefinition, SegmentDetection, SegmentsConfig, SegmentsConfigError,
};
pub use slots::{
    EnumValue, GoalDefinition, SlotDefinition, SlotType, SlotsConfig, SlotsConfigError,
};
pub use sms_templates::{SmsCategories, SmsConfig, SmsTemplatesConfig, SmsTemplatesConfigError};
pub use stages::{
    StageDefinition, StageRequirements, StagesConfig, StagesConfigError, TransitionTrigger,
};
pub use tools::{ToolParameter, ToolSchema, ToolsConfig, ToolsConfigError};
pub use views::{AgentDomainView, CompetitorInfo, LlmDomainView, MonthlySavings, ToolsDomainView};

// Re-export legacy DomainConfig for backward compatibility
pub use crate::domain_config::{
    domain_config, init_domain_config, DomainConfig, DomainConfigManager,
};
