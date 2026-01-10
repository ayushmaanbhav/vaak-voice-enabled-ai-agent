//! Core traits and types for the voice agent
//!
//! This crate provides foundational types used across all other crates:
//! - Core traits for pluggable backends (STT, TTS, LLM, etc.)
//! - Audio frame types and processing
//! - Language definitions (22 Indian languages)
//! - Text processing types (PII, compliance)
//! - Error types
//! - Conversation types

// Existing modules
pub mod audio;
pub mod conversation;
pub mod customer;
pub mod error;
pub mod transcript;

// New modules (Phase 1)
pub mod compliance;
pub mod domain;
pub mod domain_context;
pub mod language;
pub mod llm_types;
pub mod pii;
pub mod traits;
pub mod voice_config;

// Phase 5: Personalization
pub mod personalization;

// Financial calculations (single source of truth for EMI, etc.)
pub mod financial;

// Re-exports from existing modules
pub use audio::{AudioEncoding, AudioFrame, Channels, SampleRate};
pub use conversation::{ConversationStage, Turn, TurnRole};
pub use customer::{
    CompanyRelationship, CustomerProfile, CustomerSegment, SegmentDetector,
    SegmentId as CustomerSegmentId,  // Re-export for clarity
};
pub use error::{Error, Result};
pub use transcript::{TranscriptResult, WordTimestamp};

// Re-exports from new modules
pub use compliance::{
    AdditionPosition, AdditionType, ComplianceResult, ComplianceViolation, RequiredAddition,
    Severity, SuggestedRewrite, ViolationCategory,
};
pub use domain_context::{Abbreviation, DomainContext};
pub use language::{Language, Script};
pub use llm_types::{
    FinishReason, GenerateRequest, GenerateResponse, Message, Role, StreamChunk, TokenUsage,
    ToolCall, ToolDefinition,
};
pub use pii::{DetectionMethod, PIIEntity, PIISeverity, PIIType, RedactionStrategy};
pub use voice_config::{VoiceConfig, VoiceGender, VoiceInfo};

// Trait re-exports
pub use traits::{
    AudioProcessor,
    ComplianceChecker,
    ControlFrame,
    ConversationContext,
    ConversationEvent,
    // P0 FIX: ConversationFSM trait and types
    ConversationFSM,
    ConversationOutcome,
    ConversationTurn,
    Document,
    FSMAction,
    FSMCheckpoint,
    FSMError,
    FSMMetrics,
    FilterOp,
    Frame,
    // Pipeline
    FrameProcessor,
    // Text processing
    GrammarCorrector,
    // LLM
    LanguageModel,
    MetadataFilter,
    MetricsEvent,
    ObjectionType,
    PIIRedactor,
    ProcessorContext,
    RetrieveOptions,
    // Retrieval
    Retriever,
    // Speech
    SpeechToText,
    // P0 FIX: TextProcessor trait for pipeline integration
    TextProcessor,
    TextProcessorResult,
    TextToSpeech,
    // P13 FIX: ToolFactory trait for domain-agnostic tool creation
    ToolFactory,
    ToolFactoryError,
    ToolFactoryRegistry,
    ToolMetadata,
    TransitionRecord,
    Translator,
    VADConfig,
    VADEvent,
    VADState,
    // P1 FIX: VoiceActivityDetector trait
    VoiceActivityDetector,
    // P24 FIX: Config-driven persona provider
    ConfigPersonaProvider,
    PersonaAdaptationRule,
    PersonaConfig,
    PersonaProvider,
    ToneConfig,
};

// Personalization re-exports
pub use personalization::{
    BehaviorSignal,
    // Adaptation types (config-driven)
    feature_ids, objection_ids, parse_segment_id, segment_to_id,
    FeatureId as PersonalizationFeatureId, ObjectionId as PersonalizationObjectionId,
    ObjectionResponse,
    // Persona
    LanguageComplexity,
    Persona,
    PersonaTemplates,
    PersonalizationContext,
    // Engine
    PersonalizationEngine,
    ResponseUrgency,
    // Adaptation config types
    ObjectionResponseConfig,
    SegmentAdapter,
    SegmentAdapterConfig,
    SignalDetection,
    // Signals
    SignalDetector,
    SignalDetectorConfig,
    Tone,
    TrendAnalysis,
};

// Domain type aliases re-exports
pub use domain::{
    CustomerSignals, FeatureId, GoalDefinition, ObjectionId, Pattern, PatternType, ResponseTemplate,
    ScoringThresholds, ScoringWeights, SegmentId, SlotDefinition, SlotId, SlotType, SlotValidation,
    StageId, ToolId,
};
