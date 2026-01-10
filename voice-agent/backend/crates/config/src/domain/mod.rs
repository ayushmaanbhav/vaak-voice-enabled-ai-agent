//! Hierarchical Domain Configuration
//!
//! Provides a layered configuration system:
//! 1. Base config (config/base/defaults.yaml)
//! 2. Domain config (config/domains/{domain}/domain.yaml)
//! 3. Runtime overrides (per-session)
//!
//! Each crate accesses config through a specific "view" that translates
//! raw config into crate-specific terminology.

mod adaptation;
mod branches;
mod bridge;
mod compliance;
mod competitors;
mod documents;
mod entities;
mod extraction_patterns;
mod features;
mod goals;
mod intents;
mod master;
mod objections;
mod personas;
mod prompts;
mod scoring;
mod segments;
mod signals;
mod slots;
mod sms_templates;
mod stages;
mod tool_responses;
mod tools;
mod validator;
mod views;
mod vocabulary;

pub use adaptation::{
    AdaptationConfig, AdaptationConfigError, SegmentAdaptation, SpecialProgram,
};
pub use branches::{BranchDefaults, BranchEntry, BranchesConfig, BranchesConfigError, DoorstepServiceConfig};
pub use compliance::{
    AutoCorrections, ClaimRule, CompetitorRules as ComplianceCompetitorRules, ComplianceConfig,
    ComplianceConfigError, LanguageRules, RateRules, RegulatoryInfo, RequiredDisclosure,
    SeverityLevels,
};
pub use documents::{
    CustomerTypeEntry, DocumentEntry, DocumentsConfig, DocumentsConfigError, DocumentToolConfig,
    ImportantNotes, ServiceTypeEntry,
};
pub use extraction_patterns::{
    AssetQualityConfig, AssetQualityTier, CityEntry, CompiledCityPattern, CompiledPurposePattern,
    CompiledQualityTier, ExtractionPatternsConfig, ExtractionPatternsError, LocationsConfig,
    PurposeCategory, PurposesConfig, UnitConversionsConfig, ValidationConfig,
};
pub use competitors::{
    ComparisonPoint, CompetitorDefaults, CompetitorEntry, CompetitorsConfig,
    CompetitorsConfigError, RateRange,
};
pub use features::{FeatureDefinition, FeatureId, FeaturesConfig};
pub use goals::{
    ActionContext, ActionTemplate, ActionTemplatesConfig, GoalEntry, GoalsConfig, GoalsConfigError,
};
pub use entities::{
    CompetitorTypeDefaults, CompetitorTypeDefinition, EntitiesConfig, EntitiesConfigError,
    EntityCategory, EntityTypeDefinition,
};
pub use intents::{IntentDefinition, IntentsConfig, IntentsConfigError};
pub use master::{
    BrandConfig, ContextualRule, CurrencyConfig, DisplayUnit, DisplayUnitsConfig, DomainBoostConfig,
    DomainBoostTermEntry, DomainKeywordsConfig, EntityPatternConfig, IntentKeywordConfig,
    MasterDomainConfig, MemoryCompressorConfig, PhoneticCorrectionsConfig,
    PhoneticCorrectorParams, QueryExpansionConfig, QueryExpansionSettings,
    SlotDisplayConfig, VocabularyConfig,
};
pub use objections::{
    ObjectionDefinition, ObjectionResponse, ObjectionsConfig, ObjectionsConfigError,
};
pub use personas::{
    AdaptationRule, ComplexityConfig, EmotionAcknowledgmentConfig, HinglishConfig,
    NameUsageConfig, PersonasConfig, PersonasConfigError, RangeGuideline,
    ResponseLengthGuidelines, ThresholdConfig, ToneConfig, UrgencyConfig,
};
pub use prompts::{PromptsConfig, PromptsConfigError};
pub use scoring::{
    CategoryWeights, ConversionMultipliers, EscalationConfig, QualificationThresholds,
    ScoringConfig, ScoringConfigError, TrustScores,
};
pub use signals::{
    EscalationTriggerDef, ScoringThreshold, SignalCategory, SignalDefinition as SignalDefConfig,
    SignalsConfig, SignalsConfigError,
};
pub use segments::{
    NumericThreshold, SegmentDefinition, SegmentDetection, SegmentId, SegmentPersonaConfig,
    SegmentsConfig, SegmentsConfigError,
};
pub use slots::{
    EnumParsingConfig, EnumValue, GoalDefinition, NumericPatternRule, SlotDefinition, SlotType,
    SlotsConfig, SlotsConfigError,
};
pub use sms_templates::{SmsCategories, SmsConfig, SmsTemplatesConfig, SmsTemplatesConfigError};
pub use stages::{
    StageDefinition, StageRequirements, StagesConfig, StagesConfigError, TransitionTrigger,
};
pub use tool_responses::{ToolResponsesConfig, ToolResponsesConfigError, ToolTemplates, TemplateVariant};
pub use tools::{IntentToolMapping, IntentToolMappingsConfig, ToolDefinition, ToolParameter, ToolSchema, ToolSchemaMetadata, ToolsConfig, ToolsConfigError};
pub use views::{AgentDomainView, CompetitorInfo, LlmDomainView, MonthlySavings, ToolsDomainView};
pub use vocabulary::{DomainTerm, FullVocabularyConfig, FullVocabularyConfigError};

// P13 FIX: Domain bridge for trait implementations
pub use bridge::DomainBridge;

// P5.2 FIX: Config validator for startup validation
pub use validator::{
    ConfigValidator, ValidationCategory, ValidationError, ValidationResult, ValidationSeverity,
};

// P13 FIX: DomainConfig and DomainConfigManager removed - use MasterDomainConfig + views
