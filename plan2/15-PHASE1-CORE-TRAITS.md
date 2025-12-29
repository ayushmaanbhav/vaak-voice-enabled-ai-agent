# Phase 1: Core Traits & Types Implementation

> **Priority:** P0 (Blocking)
> **Duration:** 2 weeks
> **Dependencies:** None
> **Blocks:** All other phases

---

## Overview

This phase creates the foundational trait system that all other components depend on. The architecture defines 9 core traits that must be implemented in `crates/core/src/traits/`.

---

## 1. File Structure

### Current Structure
```
crates/core/src/
├── audio.rs
├── conversation.rs
├── customer.rs
├── error.rs
├── lib.rs
└── transcript.rs
```

### Target Structure
```
crates/core/src/
├── audio.rs              # UPDATE: Add missing fields
├── conversation.rs       # UPDATE: Add ConversationState variants
├── customer.rs
├── error.rs
├── lib.rs                # UPDATE: Add trait exports
├── transcript.rs         # UPDATE: Add TranscriptFrame alias
├── traits/               # NEW
│   ├── mod.rs
│   ├── speech.rs         # SpeechToText, TextToSpeech
│   ├── llm.rs            # LanguageModel
│   ├── retriever.rs      # Retriever
│   ├── text_processing.rs
│   └── pipeline.rs       # FrameProcessor
├── language.rs           # NEW: Language enum (22 variants)
├── voice_config.rs       # NEW: VoiceConfig, VoiceInfo
├── pii.rs                # NEW: PII types
├── compliance.rs         # NEW: Compliance types
├── domain_context.rs     # NEW: DomainContext
└── llm_types.rs          # NEW: GenerateRequest, etc.
```

---

## 2. Traits Implementation

### 2.1 traits/mod.rs

```rust
//! Core traits for the voice agent system
//!
//! All major components implement these traits to enable:
//! - Pluggable backends
//! - Testing with mocks
//! - Runtime switching

mod speech;
mod llm;
mod retriever;
mod text_processing;
mod pipeline;

pub use speech::{SpeechToText, TextToSpeech};
pub use llm::LanguageModel;
pub use retriever::Retriever;
pub use text_processing::{GrammarCorrector, Translator, PIIRedactor, ComplianceChecker};
pub use pipeline::FrameProcessor;
```

### 2.2 traits/speech.rs

```rust
//! Speech processing traits

use async_trait::async_trait;
use futures::Stream;
use crate::{AudioFrame, TranscriptFrame, Language, VoiceConfig, VoiceInfo, Result};

/// Speech-to-Text interface
///
/// Implementations:
/// - `IndicConformerStt` - AI4Bharat's multilingual STT
/// - `WhisperStt` - OpenAI Whisper (fallback)
#[async_trait]
pub trait SpeechToText: Send + Sync + 'static {
    /// Transcribe a single audio frame
    async fn transcribe(&self, audio: &AudioFrame) -> Result<TranscriptFrame>;

    /// Stream transcription as audio arrives
    ///
    /// Returns partial transcripts followed by final transcript
    fn transcribe_stream<'a>(
        &'a self,
        audio_stream: impl Stream<Item = AudioFrame> + Send + 'a,
    ) -> Box<dyn Stream<Item = Result<TranscriptFrame>> + Send + Unpin + 'a>;

    /// Get supported languages
    fn supported_languages(&self) -> &[Language];

    /// Get model name for logging
    fn model_name(&self) -> &str;
}

/// Text-to-Speech interface
///
/// Implementations:
/// - `IndicF5Tts` - AI4Bharat's multilingual TTS
/// - `PiperTts` - Fast fallback TTS
#[async_trait]
pub trait TextToSpeech: Send + Sync + 'static {
    /// Synthesize text to audio
    async fn synthesize(&self, text: &str, config: &VoiceConfig) -> Result<AudioFrame>;

    /// Stream synthesis sentence-by-sentence
    ///
    /// Enables low-latency response by starting audio before text is complete
    fn synthesize_stream<'a>(
        &'a self,
        text_stream: impl Stream<Item = String> + Send + 'a,
        config: &VoiceConfig,
    ) -> Box<dyn Stream<Item = Result<AudioFrame>> + Send + Unpin + 'a>;

    /// Get available voices
    fn available_voices(&self) -> &[VoiceInfo];

    /// Get model name for logging
    fn model_name(&self) -> &str;
}
```

### 2.3 traits/llm.rs

```rust
//! Language Model traits

use async_trait::async_trait;
use futures::Stream;
use crate::{Result, GenerateRequest, GenerateResponse, StreamChunk, ToolDefinition};

/// Language Model interface
///
/// Implementations:
/// - `OllamaBackend` - Local Ollama inference
/// - `ClaudeBackend` - Anthropic Claude API (future)
/// - `OpenAIBackend` - OpenAI API (future)
#[async_trait]
pub trait LanguageModel: Send + Sync + 'static {
    /// Generate completion
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse>;

    /// Stream tokens as generated
    fn generate_stream<'a>(
        &'a self,
        request: GenerateRequest,
    ) -> Box<dyn Stream<Item = Result<StreamChunk>> + Send + Unpin + 'a>;

    /// Generate with tool/function calling
    async fn generate_with_tools(
        &self,
        request: GenerateRequest,
        tools: &[ToolDefinition],
    ) -> Result<GenerateResponse>;

    /// Check if model is available
    async fn is_available(&self) -> bool;

    /// Get model name for logging
    fn model_name(&self) -> &str;
}
```

### 2.4 traits/retriever.rs

```rust
//! Retrieval traits

use async_trait::async_trait;
use crate::{Result, Document, RetrieveOptions, ConversationContext};

/// Retriever interface for RAG
///
/// Implementations:
/// - `HybridRetriever` - Dense + Sparse + Reranking
/// - `AgenticRetriever` - Multi-step with query rewriting
#[async_trait]
pub trait Retriever: Send + Sync + 'static {
    /// Retrieve relevant documents
    async fn retrieve(
        &self,
        query: &str,
        options: &RetrieveOptions,
    ) -> Result<Vec<Document>>;

    /// Agentic multi-step retrieval
    ///
    /// Iteratively refines query until sufficient documents found
    async fn retrieve_agentic(
        &self,
        query: &str,
        context: &ConversationContext,
        max_iterations: usize,
    ) -> Result<Vec<Document>>;

    /// Prefetch documents based on partial transcript
    ///
    /// Called on VAD speech detection to reduce latency
    fn prefetch(&self, partial_transcript: &str);

    /// Get retriever name for logging
    fn name(&self) -> &str;
}
```

### 2.5 traits/text_processing.rs

```rust
//! Text processing traits

use async_trait::async_trait;
use futures::Stream;
use crate::{
    Result, Language, DomainContext,
    PIIEntity, RedactionStrategy,
    ComplianceResult,
};

/// Grammar correction interface
///
/// Implementations:
/// - `LLMGrammarCorrector` - Uses LLM for domain-aware correction
/// - `NoopCorrector` - Pass-through (disabled)
#[async_trait]
pub trait GrammarCorrector: Send + Sync + 'static {
    /// Correct grammar with domain context
    async fn correct(&self, text: &str, context: &DomainContext) -> Result<String>;

    /// Stream corrections sentence-by-sentence
    fn correct_stream<'a>(
        &'a self,
        text_stream: impl Stream<Item = String> + Send + 'a,
        context: &DomainContext,
    ) -> Box<dyn Stream<Item = Result<String>> + Send + Unpin + 'a>;

    /// Check if corrector is enabled
    fn is_enabled(&self) -> bool;
}

/// Translation interface
///
/// Implementations:
/// - `IndicTranslator` - IndicTrans2 via ONNX
/// - `GrpcTranslator` - Python sidecar fallback
#[async_trait]
pub trait Translator: Send + Sync + 'static {
    /// Translate text between languages
    async fn translate(
        &self,
        text: &str,
        from: Language,
        to: Language,
    ) -> Result<String>;

    /// Detect language of text
    async fn detect_language(&self, text: &str) -> Result<Language>;

    /// Stream translation sentence-by-sentence
    fn translate_stream<'a>(
        &'a self,
        text_stream: impl Stream<Item = String> + Send + 'a,
        from: Language,
        to: Language,
    ) -> Box<dyn Stream<Item = Result<String>> + Send + Unpin + 'a>;

    /// Check if language pair is supported
    fn supports_pair(&self, from: Language, to: Language) -> bool;

    /// Get translator name for logging
    fn name(&self) -> &str;
}

/// PII detection and redaction interface
///
/// Implementations:
/// - `HybridPIIDetector` - Regex + NER
/// - `RegexPIIDetector` - Regex only (faster)
#[async_trait]
pub trait PIIRedactor: Send + Sync + 'static {
    /// Detect PII entities in text
    async fn detect(&self, text: &str) -> Result<Vec<PIIEntity>>;

    /// Redact PII from text
    async fn redact(
        &self,
        text: &str,
        strategy: &RedactionStrategy,
    ) -> Result<String>;

    /// Get supported PII types
    fn supported_types(&self) -> &[PIIType];
}

/// Compliance checking interface
///
/// Implementations:
/// - `RuleBasedComplianceChecker` - Config-driven rules
/// - `LLMComplianceChecker` - LLM-based checking
#[async_trait]
pub trait ComplianceChecker: Send + Sync + 'static {
    /// Check text for compliance violations
    async fn check(&self, text: &str) -> Result<ComplianceResult>;

    /// Make text compliant by fixing violations
    async fn make_compliant(&self, text: &str) -> Result<String>;

    /// Get compliance rules version
    fn rules_version(&self) -> &str;
}

// Re-export PIIType for trait signature
pub use crate::pii::PIIType;
```

### 2.6 traits/pipeline.rs

```rust
//! Pipeline processing traits

use async_trait::async_trait;
use crate::{Result, Frame, ProcessorContext};

/// Frame processor for pipeline stages
///
/// Each processor receives frames, processes them, and emits output frames.
/// Processors run in separate tokio tasks, connected by channels.
#[async_trait]
pub trait FrameProcessor: Send + Sync + 'static {
    /// Process a frame and emit zero or more output frames
    async fn process(
        &self,
        frame: Frame,
        context: &mut ProcessorContext,
    ) -> Result<Vec<Frame>>;

    /// Get processor name for tracing
    fn name(&self) -> &'static str;

    /// Get processor description
    fn description(&self) -> &str {
        ""
    }
}
```

---

## 3. Supporting Types

### 3.1 language.rs

```rust
//! Language definitions for 22 Indian languages

use serde::{Deserialize, Serialize};

/// Supported languages (22 scheduled Indian languages + English)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    English,
    Hindi,
    Tamil,
    Telugu,
    Kannada,
    Malayalam,
    Bengali,
    Marathi,
    Gujarati,
    Punjabi,
    Odia,
    Assamese,
    Urdu,
    Kashmiri,
    Sindhi,
    Konkani,
    Dogri,
    Bodo,
    Maithili,
    Santali,
    Nepali,
    Manipuri,
    Sanskrit,
}

impl Language {
    /// Get ISO 639-1 code
    pub fn code(&self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Hindi => "hi",
            Self::Tamil => "ta",
            Self::Telugu => "te",
            Self::Kannada => "kn",
            Self::Malayalam => "ml",
            Self::Bengali => "bn",
            Self::Marathi => "mr",
            Self::Gujarati => "gu",
            Self::Punjabi => "pa",
            Self::Odia => "or",
            Self::Assamese => "as",
            Self::Urdu => "ur",
            Self::Kashmiri => "ks",
            Self::Sindhi => "sd",
            Self::Konkani => "kok",
            Self::Dogri => "doi",
            Self::Bodo => "brx",
            Self::Maithili => "mai",
            Self::Santali => "sat",
            Self::Nepali => "ne",
            Self::Manipuri => "mni",
            Self::Sanskrit => "sa",
        }
    }

    /// Get script used by this language
    pub fn script(&self) -> Script {
        match self {
            Self::Hindi | Self::Marathi | Self::Sanskrit | Self::Konkani
            | Self::Dogri | Self::Bodo | Self::Maithili | Self::Nepali => Script::Devanagari,
            Self::Tamil => Script::Tamil,
            Self::Telugu => Script::Telugu,
            Self::Kannada => Script::Kannada,
            Self::Malayalam => Script::Malayalam,
            Self::Bengali | Self::Assamese => Script::Bengali,
            Self::Gujarati => Script::Gujarati,
            Self::Punjabi => Script::Gurmukhi,
            Self::Odia => Script::Odia,
            Self::Urdu | Self::Kashmiri | Self::Sindhi => Script::Arabic,
            Self::Santali => Script::OlChiki,
            Self::Manipuri => Script::MeeteiMayek,
            Self::English => Script::Latin,
        }
    }

    /// Parse from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Some(Self::English),
            "hi" | "hindi" => Some(Self::Hindi),
            "ta" | "tamil" => Some(Self::Tamil),
            "te" | "telugu" => Some(Self::Telugu),
            "kn" | "kannada" => Some(Self::Kannada),
            "ml" | "malayalam" => Some(Self::Malayalam),
            "bn" | "bengali" => Some(Self::Bengali),
            "mr" | "marathi" => Some(Self::Marathi),
            "gu" | "gujarati" => Some(Self::Gujarati),
            "pa" | "punjabi" => Some(Self::Punjabi),
            "or" | "odia" => Some(Self::Odia),
            "as" | "assamese" => Some(Self::Assamese),
            "ur" | "urdu" => Some(Self::Urdu),
            _ => None,
        }
    }
}

/// Script systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Script {
    Latin,
    Devanagari,
    Bengali,
    Tamil,
    Telugu,
    Kannada,
    Malayalam,
    Gujarati,
    Gurmukhi,
    Odia,
    Arabic,
    OlChiki,
    MeeteiMayek,
}
```

### 3.2 voice_config.rs

```rust
//! Voice configuration types

use serde::{Deserialize, Serialize};
use crate::Language;

/// Voice configuration for TTS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    /// Target language
    pub language: Language,
    /// Voice identifier
    pub voice_id: String,
    /// Speech speed (0.5 - 2.0, default 1.0)
    pub speed: f32,
    /// Voice pitch adjustment (-1.0 to 1.0, default 0.0)
    pub pitch: f32,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            language: Language::Hindi,
            voice_id: "default".to_string(),
            speed: 1.0,
            pitch: 0.0,
        }
    }
}

/// Voice information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInfo {
    /// Voice identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Supported language
    pub language: Language,
    /// Gender (optional)
    pub gender: Option<VoiceGender>,
    /// Sample audio URL (optional)
    pub sample_url: Option<String>,
}

/// Voice gender
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VoiceGender {
    Male,
    Female,
    Neutral,
}
```

### 3.3 pii.rs

```rust
//! PII detection types

use serde::{Deserialize, Serialize};

/// PII types specific to India
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PIIType {
    // Standard
    PersonName,
    PhoneNumber,
    Email,
    Address,

    // India-specific
    Aadhaar,
    PAN,
    VoterId,
    DrivingLicense,
    Passport,
    BankAccount,
    IFSC,

    // Financial
    LoanAmount,
    InterestRate,
    CompetitorName,
}

/// Detected PII entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PIIEntity {
    /// Type of PII
    pub pii_type: PIIType,
    /// The actual text
    pub text: String,
    /// Start position in original text
    pub start: usize,
    /// End position in original text
    pub end: usize,
    /// Detection confidence (0.0 - 1.0)
    pub confidence: f32,
}

/// Redaction strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RedactionStrategy {
    /// Replace with [REDACTED]
    Mask,
    /// Replace with type: [PHONE]
    TypeMask,
    /// Replace with asterisks: 98****1234
    PartialMask { visible_chars: usize },
    /// Remove entirely
    Remove,
    /// Replace with fake data
    Synthesize,
}

impl Default for RedactionStrategy {
    fn default() -> Self {
        Self::PartialMask { visible_chars: 4 }
    }
}
```

### 3.4 compliance.rs

```rust
//! Compliance checking types

use serde::{Deserialize, Serialize};

/// Compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    /// Whether text is compliant
    pub is_compliant: bool,
    /// List of violations found
    pub violations: Vec<ComplianceViolation>,
    /// Required additions (disclaimers, etc.)
    pub required_additions: Vec<String>,
    /// Suggested rewrites
    pub suggested_rewrites: Vec<SuggestedRewrite>,
}

/// Compliance violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    /// Rule identifier
    pub rule_id: String,
    /// Human-readable description
    pub description: String,
    /// Severity level
    pub severity: Severity,
    /// Position in text (start, end)
    pub text_span: (usize, usize),
}

/// Severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Warning - can proceed
    Warning,
    /// Error - should fix
    Error,
    /// Critical - must not proceed
    Critical,
}

/// Suggested rewrite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedRewrite {
    /// Original text
    pub original: String,
    /// Suggested replacement
    pub replacement: String,
    /// Reason for change
    pub reason: String,
}
```

### 3.5 domain_context.rs

```rust
//! Domain context for text processing

use serde::{Deserialize, Serialize};

/// Domain-specific context for grammar correction and processing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DomainContext {
    /// Domain-specific vocabulary
    pub vocabulary: Vec<String>,
    /// Common phrases in this domain
    pub phrases: Vec<String>,
    /// Entity types to preserve (names, numbers, etc.)
    pub preserve_entities: Vec<String>,
    /// Domain name (e.g., "gold_loan")
    pub domain: String,
}

impl DomainContext {
    /// Create context for gold loan domain
    pub fn gold_loan() -> Self {
        Self {
            domain: "gold_loan".to_string(),
            vocabulary: vec![
                "gold loan".to_string(),
                "Kotak".to_string(),
                "Muthoot".to_string(),
                "Manappuram".to_string(),
                "IIFL".to_string(),
                "LTV".to_string(),
                "per gram".to_string(),
                "interest rate".to_string(),
                "processing fee".to_string(),
                "balance transfer".to_string(),
                "top-up".to_string(),
                "foreclosure".to_string(),
            ],
            phrases: vec![
                "Kotak Bank se baat kar rahe hain".to_string(),
                "gold loan balance transfer".to_string(),
                "kam interest rate".to_string(),
            ],
            preserve_entities: vec![
                "PersonName".to_string(),
                "PhoneNumber".to_string(),
                "LoanAmount".to_string(),
            ],
        }
    }
}
```

### 3.6 llm_types.rs

```rust
//! LLM request/response types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LLM generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    /// Messages for chat completion
    pub messages: Vec<Message>,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Temperature (0.0 - 2.0)
    pub temperature: Option<f32>,
    /// Top-p sampling
    pub top_p: Option<f32>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Enable streaming
    pub stream: bool,
    /// Model override (optional)
    pub model: Option<String>,
}

impl Default for GenerateRequest {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            max_tokens: None,
            temperature: Some(0.7),
            top_p: None,
            stop: None,
            stream: false,
            model: None,
        }
    }
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// Message role
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// LLM generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    /// Generated text
    pub text: String,
    /// Finish reason
    pub finish_reason: FinishReason,
    /// Token usage
    pub usage: Option<TokenUsage>,
    /// Tool calls (if any)
    pub tool_calls: Vec<ToolCall>,
}

/// Finish reason
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Error,
}

/// Token usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Stream chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// Text delta
    pub delta: String,
    /// Whether this is the final chunk
    pub is_final: bool,
    /// Finish reason (only on final chunk)
    pub finish_reason: Option<FinishReason>,
}

/// Tool definition for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// JSON schema for parameters
    pub parameters: serde_json::Value,
}

/// Tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Call ID
    pub id: String,
    /// Tool name
    pub name: String,
    /// Arguments (JSON)
    pub arguments: HashMap<String, serde_json::Value>,
}
```

---

## 4. Update lib.rs

```rust
//! Core traits and types for the voice agent
//!
//! This crate provides foundational types used across all other crates.

pub mod audio;
pub mod error;
pub mod transcript;
pub mod conversation;
pub mod customer;
pub mod traits;           // NEW
pub mod language;         // NEW
pub mod voice_config;     // NEW
pub mod pii;              // NEW
pub mod compliance;       // NEW
pub mod domain_context;   // NEW
pub mod llm_types;        // NEW

// Re-exports
pub use audio::{AudioFrame, AudioEncoding, Channels, SampleRate};
pub use error::{Error, Result};
pub use transcript::{TranscriptResult, TranscriptFrame, WordTimestamp};
pub use conversation::{Turn, TurnRole, ConversationStage, ConversationState};
pub use customer::{CustomerProfile, CustomerSegment};

// New exports
pub use language::{Language, Script};
pub use voice_config::{VoiceConfig, VoiceInfo, VoiceGender};
pub use pii::{PIIType, PIIEntity, RedactionStrategy};
pub use compliance::{ComplianceResult, ComplianceViolation, Severity, SuggestedRewrite};
pub use domain_context::DomainContext;
pub use llm_types::{
    GenerateRequest, GenerateResponse, Message, Role,
    StreamChunk, FinishReason, TokenUsage,
    ToolDefinition, ToolCall,
};

// Trait exports
pub use traits::{
    SpeechToText, TextToSpeech,
    LanguageModel,
    Retriever,
    GrammarCorrector, Translator, PIIRedactor, ComplianceChecker,
    FrameProcessor,
};
```

---

## 5. Checklist

### 5.1 Traits Module
- [ ] Create `crates/core/src/traits/mod.rs`
- [ ] Create `crates/core/src/traits/speech.rs`
- [ ] Create `crates/core/src/traits/llm.rs`
- [ ] Create `crates/core/src/traits/retriever.rs`
- [ ] Create `crates/core/src/traits/text_processing.rs`
- [ ] Create `crates/core/src/traits/pipeline.rs`

### 5.2 Supporting Types
- [ ] Create `crates/core/src/language.rs`
- [ ] Create `crates/core/src/voice_config.rs`
- [ ] Create `crates/core/src/pii.rs`
- [ ] Create `crates/core/src/compliance.rs`
- [ ] Create `crates/core/src/domain_context.rs`
- [ ] Create `crates/core/src/llm_types.rs`

### 5.3 Updates
- [ ] Update `crates/core/src/lib.rs` with new exports
- [ ] Add `TranscriptFrame` type alias in transcript.rs
- [ ] Add missing ConversationState variants
- [ ] Update Cargo.toml with new dependencies (async-trait, futures)

### 5.4 Backend Updates
- [ ] Update `SttBackend` to implement `SpeechToText`
- [ ] Update `TtsBackend` to implement `TextToSpeech`
- [ ] Update `LlmBackend` to implement `LanguageModel`
- [ ] Update `HybridRetriever` to implement `Retriever`

### 5.5 Tests
- [ ] Add unit tests for Language enum
- [ ] Add unit tests for type conversions
- [ ] Add mock implementations for traits

---

## 6. Dependencies to Add

```toml
# crates/core/Cargo.toml
[dependencies]
async-trait = "0.1"
futures = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

*This phase provides the foundation for all other phases. No other phase can begin until this is complete.*
