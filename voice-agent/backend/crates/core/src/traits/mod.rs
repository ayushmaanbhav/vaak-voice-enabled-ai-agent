//! Core traits for the voice agent system
//!
//! All major components implement these traits to enable:
//! - Pluggable backends (swap implementations without code changes)
//! - Testing with mocks
//! - Runtime switching based on configuration
//!
//! # Trait Hierarchy
//!
//! ```text
//! Speech Processing:
//!   - SpeechToText: Audio → Text transcription
//!   - TextToSpeech: Text → Audio synthesis
//!
//! Language Models:
//!   - LanguageModel: Text generation and tool calling
//!
//! Tools:
//!   - Tool: MCP-compatible tool interface
//!
//! Retrieval:
//!   - Retriever: Document retrieval for RAG
//!
//! Text Processing:
//!   - GrammarCorrector: Fix grammar while preserving domain terms
//!   - Translator: Translate between Indian languages
//!   - PIIRedactor: Detect and redact PII
//!   - ComplianceChecker: Check for regulatory compliance
//!
//! Conversation:
//!   - ConversationFSM: Finite state machine for conversation flow
//!
//! Pipeline:
//!   - FrameProcessor: Process frames in the pipeline
//!
//! Domain Abstractions (P13 FIX: Domain-agnostic traits):
//!   - DomainCalculator: Business calculations (EMI, value, rates)
//!   - SlotSchema: Dynamic slot definitions and extraction
//!   - ConversationGoalSchema: Dynamic goal definitions
//!   - SegmentDetector: Customer segmentation
//!   - ObjectionHandler: Objection detection and response
//!   - LeadScoringStrategy: Lead qualification and scoring
//!   - CompetitorAnalyzer: Competitor comparison
//!
//! P20 FIX: Config-driven abstractions (replaces hardcoded enums):
//!   - FeatureProvider: Config-driven feature definitions (replaces Feature enum)
//!   - ObjectionProvider: Config-driven objection handling (replaces Objection enum)
//!   - ToolArgumentProvider: Config-driven tool defaults and mappings
//!   - LeadClassifier: Config-driven MQL/SQL classification
//! ```

mod fsm;
mod llm;
mod pipeline;
mod retriever;
mod speech;
mod text_processing;
mod tool;
mod tool_factory;

// P13 FIX: Domain-agnostic trait modules
mod calculator;
mod competitors;
mod goals;
mod objections;
mod scoring;
mod segments;
mod slots;

// P20 FIX: Config-driven trait modules (replaces hardcoded enums)
mod feature_provider;
mod lead_classifier;
mod objection_provider;
mod tool_arguments;

// P23 FIX: Generic signal system for domain-agnostic lead scoring
mod signals;

// P23 FIX: Config-driven entity types (replaces CompetitorType, CustomerSegment enums)
mod entity_types;

// P24 FIX: Config-driven persona provider (replaces hardcoded Persona::for_segment, Tone methods)
mod persona_provider;

pub use speech::{SpeechToText, TextToSpeech};
// P1 FIX: Export VoiceActivityDetector trait and types
pub use llm::LanguageModel;
pub use pipeline::{ControlFrame, Frame, FrameProcessor, MetricsEvent, ProcessorContext};
pub use retriever::{
    ConversationContext, ConversationTurn, Document, FilterOp, MetadataFilter, RetrieveOptions,
    Retriever,
};
pub use speech::{AudioProcessor, VADConfig, VADEvent, VADState, VoiceActivityDetector};
pub use text_processing::{
    ComplianceChecker, GrammarCorrector, PIIRedactor, TextProcessor, TextProcessorResult,
    Translator,
};
// P0 FIX: Export ConversationFSM trait and types
pub use fsm::{
    ConversationEvent, ConversationFSM, ConversationOutcome, FSMAction, FSMCheckpoint, FSMError,
    FSMMetrics, ObjectionType, TransitionRecord,
};
// P3 FIX: Export Tool trait and types
pub use tool::{
    validate_property, ContentBlock, ErrorCode, InputSchema, PropertySchema, Tool, ToolError,
    ToolInput, ToolOutput, ToolSchema,
};
// P13 FIX: Export ToolFactory trait for domain-agnostic tool creation
pub use tool_factory::{ToolFactory, ToolFactoryError, ToolFactoryRegistry, ToolMetadata};

// P13 FIX: Export domain-agnostic traits and types
pub use calculator::{
    CalculatorError, ConfigDrivenCalculator, DomainCalculator, QualityFactor, RateTier,
    SavingsResult,
};
pub use competitors::{
    ComparisonPoint, CompetitorAnalyzer, CompetitorInfo, ConfigCompetitorAnalyzer, SavingsAnalysis,
};
pub use goals::{
    ConfigGoalDefinition, ConfigGoalSchema, ConversationGoalSchema, GoalCompletionStatus,
    GoalDefinition, NextAction,
};
pub use objections::{
    AcreResponse, ConfigObjectionDefinition, ConfigObjectionHandler, ObjectionDefinition,
    ObjectionHandler, ObjectionMatch,
};
pub use scoring::{
    ConfigLeadScoring, DynamicScoreBreakdown, EscalationTrigger, LeadClassification,
    LeadScoringStrategy, LeadSignals, QualificationLevel, ScoreBreakdown, ScoringConfig,
    SimpleLeadSignals,
};
pub use segments::{
    ConfigSegmentDefinition, ConfigSegmentDetector, FeatureEmphasis, SegmentDefinition,
    SegmentDetector, SegmentMatch, ValueProposition,
};
pub use slots::{
    ConfigSlotDefinition, ConfigSlotSchema, EnumValue, ExtractedSlot, SlotDefinition, SlotSchema,
    SlotType, SlotValidationError, UnitConversion,
};

// P20 FIX: Export config-driven traits and types
pub use feature_provider::{
    feature_ids, substitute_variables, ConfigFeatureDefinition, ConfigFeatureProvider,
    FeatureDefinition, FeatureDisplay, FeatureId, FeaturePriority, FeatureProvider,
    SegmentFeatureOverride, SegmentValueProposition, VariableMap,
};
pub use lead_classifier::{
    ClassificationRule, ConfigLeadClassifier, EscalationTriggerConfig, EscalationTriggerResult,
    LeadClass, LeadClassifier, LeadSignalsTrait, QualificationThreshold, SimpleLeadSignalsImpl,
};
// Note: QualificationLevel already exported from scoring module
pub use objection_provider::{
    objection_ids, AcreResponseParts, ConfigAcreResponse, ConfigObjectionDef,
    ConfigObjectionProvider, DetectionPattern, ObjectionDefinitionTrait, ObjectionDetection,
    ObjectionId, ObjectionProvider, ObjectionResponse, PatternBoost,
};
pub use tool_arguments::{
    ArgumentValidationError, ConfigToolArgumentProvider, IntentToolMapping, ToolArgumentProvider,
    ToolDefaults,
};

// P23 FIX: Export signal system for domain-agnostic lead scoring
pub use signals::{SignalDefinition, SignalProvider, SignalStore, SignalType, SignalValue};

// P23 FIX: Export entity type system for config-driven type definitions
pub use entity_types::{
    EntityTypeCategory, EntityTypeDefinition, EntityTypeProvider, EntityTypeStore,
};

// P24 FIX: Export persona provider for config-driven persona management
pub use persona_provider::{
    AdaptationRule as PersonaAdaptationRule, ConfigPersonaProvider, PersonaConfig, PersonaProvider,
    SegmentId, ToneConfig,
};
