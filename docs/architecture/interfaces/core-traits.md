# Core Traits Specification

> Comprehensive interface definitions for all voice agent components
>
> **Design Principle:** Every component is a trait. Every trait is async. Every trait supports streaming.

---

## Table of Contents

1. [Design Philosophy](#design-philosophy)
2. [Speech Interfaces](#speech-interfaces)
3. [Text Processing Interfaces](#text-processing-interfaces)
4. [Intelligence Interfaces](#intelligence-interfaces)
5. [Conversation Interfaces](#conversation-interfaces)
6. [Memory Interfaces](#memory-interfaces)
7. [Observability Interfaces](#observability-interfaces)
8. [Error Handling](#error-handling)

---

## Design Philosophy

### Why Traits?

Rust traits provide:

1. **Compile-time polymorphism** - Zero runtime overhead for abstraction
2. **Static dispatch by default** - Inlined and optimized
3. **Dynamic dispatch when needed** - `dyn Trait` for runtime flexibility
4. **Clear contracts** - Explicit interface definitions
5. **Testability** - Easy mocking with trait objects

### Design Rules

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         TRAIT DESIGN RULES                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  1. ALL traits are async (#[async_trait])                               │
│     └── Voice agents are I/O bound, async everywhere                    │
│                                                                         │
│  2. ALL traits require Send + Sync + 'static                            │
│     └── Enables safe sharing across Tokio tasks                         │
│                                                                         │
│  3. EVERY operation has a streaming variant                             │
│     └── fn foo(&self, ...) → Result<T>                                  │
│     └── fn foo_stream(&self, ...) → impl Stream<Item = Result<T>>      │
│                                                                         │
│  4. Configuration via associated types or parameters, not constructors  │
│     └── Enables runtime configuration changes                           │
│                                                                         │
│  5. Errors are typed, not stringly-typed                                │
│     └── Custom error enums, not String or anyhow                        │
│                                                                         │
│  6. Traits should be object-safe when possible                          │
│     └── Enables Box<dyn Trait> for runtime polymorphism                 │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Trait Hierarchy

```
                        ┌─────────────────┐
                        │   Component     │
                        │    (marker)     │
                        └────────┬────────┘
                                 │
          ┌──────────────────────┼──────────────────────┐
          │                      │                      │
    ┌─────┴─────┐          ┌─────┴─────┐          ┌─────┴─────┐
    │  Speech   │          │   Text    │          │  Agent    │
    │ Component │          │ Component │          │ Component │
    └─────┬─────┘          └─────┬─────┘          └─────┬─────┘
          │                      │                      │
    ┌─────┴─────┐          ┌─────┴─────┐          ┌─────┴─────┐
    │    STT    │          │  Grammar  │          │    LLM    │
    │    TTS    │          │ Translate │          │    RAG    │
    │    VAD    │          │    PII    │          │   Tool    │
    └───────────┘          │ Compliant │          │   FSM     │
                           └───────────┘          └───────────┘
```

---

## Speech Interfaces

### SpeechToText

```rust
use async_trait::async_trait;
use futures::Stream;

/// Speech-to-Text transcription interface
///
/// # Implementations
/// - `SherpaStt` - sherpa-rs with ONNX models
/// - `WhisperStt` - Whisper via sherpa-onnx
/// - `GrpcStt` - External service via gRPC
///
/// # Example
/// ```rust
/// let stt: Box<dyn SpeechToText> = Box::new(SherpaStt::new(config)?);
/// let transcript = stt.transcribe(&audio_frame).await?;
/// println!("User said: {}", transcript.text);
/// ```
#[async_trait]
pub trait SpeechToText: Send + Sync + 'static {
    /// Transcribe a single audio frame
    ///
    /// # Arguments
    /// * `audio` - Audio frame to transcribe
    ///
    /// # Returns
    /// Transcript with confidence and timing information
    async fn transcribe(&self, audio: &AudioFrame) -> Result<TranscriptFrame, SpeechError>;

    /// Stream transcription as audio arrives
    ///
    /// Emits partial transcripts as they're available, followed by
    /// a final transcript when the utterance is complete.
    ///
    /// # Arguments
    /// * `audio_stream` - Stream of audio frames
    ///
    /// # Returns
    /// Stream of transcript frames (partial and final)
    fn transcribe_stream(
        &self,
        audio_stream: impl Stream<Item = AudioFrame> + Send + 'static,
    ) -> Box<dyn Stream<Item = Result<TranscriptFrame, SpeechError>> + Send + Unpin>;

    /// Get languages supported by this STT provider
    fn supported_languages(&self) -> &[Language];

    /// Check if a specific language is supported
    fn supports_language(&self, language: Language) -> bool {
        self.supported_languages().contains(&language)
    }

    /// Get model information
    fn model_info(&self) -> ModelInfo;
}

/// Model information for observability
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub version: String,
    pub size_mb: u64,
    pub quantization: Option<String>,
}
```

### TextToSpeech

```rust
/// Text-to-Speech synthesis interface
///
/// # Implementations
/// - `SherpaTts` - sherpa-rs with ONNX models (IndicF5, Piper)
/// - `GrpcTts` - External service via gRPC
///
/// # Streaming Design
/// The `synthesize_stream` method is crucial for low latency.
/// It accepts a stream of sentences and synthesizes each independently,
/// allowing audio playback to begin before the full response is generated.
///
/// # Example
/// ```rust
/// let tts: Box<dyn TextToSpeech> = Box::new(SherpaTts::new(config)?);
///
/// // Stream synthesis - low latency
/// let sentences = stream::iter(vec!["Hello sir.", "How can I help?"]);
/// let audio_stream = tts.synthesize_stream(sentences, &voice_config);
///
/// pin_mut!(audio_stream);
/// while let Some(audio) = audio_stream.next().await {
///     player.play(audio?).await?;
/// }
/// ```
#[async_trait]
pub trait TextToSpeech: Send + Sync + 'static {
    /// Synthesize text to audio
    ///
    /// # Arguments
    /// * `text` - Text to synthesize
    /// * `config` - Voice configuration (language, speed, etc.)
    ///
    /// # Returns
    /// Audio frame containing synthesized speech
    async fn synthesize(
        &self,
        text: &str,
        config: &VoiceConfig,
    ) -> Result<AudioFrame, SpeechError>;

    /// Stream synthesis sentence-by-sentence
    ///
    /// This is the primary method for production use. Each sentence
    /// is synthesized independently and emitted as soon as ready.
    ///
    /// # Arguments
    /// * `text_stream` - Stream of sentences to synthesize
    /// * `config` - Voice configuration
    ///
    /// # Returns
    /// Stream of audio frames, one per sentence
    fn synthesize_stream(
        &self,
        text_stream: impl Stream<Item = String> + Send + 'static,
        config: &VoiceConfig,
    ) -> Box<dyn Stream<Item = Result<AudioFrame, SpeechError>> + Send + Unpin>;

    /// Get available voices for a language
    fn available_voices(&self, language: Language) -> Vec<VoiceInfo>;

    /// Get all available voices
    fn all_voices(&self) -> &[VoiceInfo];

    /// Estimate synthesis duration (for scheduling)
    fn estimate_duration(&self, text: &str, config: &VoiceConfig) -> Duration;
}

/// Voice configuration
#[derive(Debug, Clone)]
pub struct VoiceConfig {
    /// Target language
    pub language: Language,
    /// Voice identifier
    pub voice_id: String,
    /// Speaking rate (0.5 = half speed, 2.0 = double speed)
    pub speed: f32,
    /// Pitch adjustment (-1.0 to 1.0)
    pub pitch: f32,
    /// Volume (0.0 to 1.0)
    pub volume: f32,
    /// Audio format
    pub format: AudioFormat,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            language: Language::Hindi,
            voice_id: "default".to_string(),
            speed: 1.0,
            pitch: 0.0,
            volume: 1.0,
            format: AudioFormat::default(),
        }
    }
}

/// Voice information
#[derive(Debug, Clone)]
pub struct VoiceInfo {
    pub id: String,
    pub name: String,
    pub language: Language,
    pub gender: Gender,
    pub style: VoiceStyle,
    pub sample_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceStyle {
    Conversational,
    Professional,
    Friendly,
    Calm,
}
```

### VoiceActivityDetector

```rust
/// Voice Activity Detection interface
///
/// Used for:
/// - Turn detection (when user finishes speaking)
/// - Barge-in detection (when user interrupts agent)
/// - Filtering silence/noise
///
/// # Implementations
/// - `SileroVad` - silero-vad via ONNX
/// - `WebRtcVad` - WebRTC VAD
#[async_trait]
pub trait VoiceActivityDetector: Send + Sync + 'static {
    /// Detect voice activity in audio frame
    ///
    /// # Arguments
    /// * `audio` - Audio frame to analyze
    /// * `sensitivity` - Detection sensitivity (0.0 - 1.0)
    ///
    /// # Returns
    /// True if voice activity detected
    async fn detect(&self, audio: &AudioFrame, sensitivity: f32) -> bool;

    /// Get speech probability (more detailed than boolean)
    async fn speech_probability(&self, audio: &AudioFrame) -> f32;

    /// Process stream and emit VAD events
    fn process_stream(
        &self,
        audio_stream: impl Stream<Item = AudioFrame> + Send + 'static,
        config: &VADConfig,
    ) -> Box<dyn Stream<Item = VADEvent> + Send + Unpin>;
}

/// VAD configuration
#[derive(Debug, Clone)]
pub struct VADConfig {
    /// Minimum speech duration to trigger (ms)
    pub min_speech_duration_ms: u64,
    /// Silence duration to end turn (ms)
    pub silence_timeout_ms: u64,
    /// Detection sensitivity (0.0 - 1.0)
    pub sensitivity: f32,
    /// Frame size (ms)
    pub frame_size_ms: u32,
}

/// VAD events
#[derive(Debug, Clone)]
pub enum VADEvent {
    /// Speech started
    SpeechStart { timestamp_ms: u64 },
    /// Speech continues
    SpeechContinue { timestamp_ms: u64, duration_ms: u64 },
    /// Speech ended (silence detected)
    SpeechEnd { timestamp_ms: u64, duration_ms: u64 },
    /// Background noise detected
    Noise { level_db: f32 },
}
```

---

## Text Processing Interfaces

### GrammarCorrector

```rust
/// Grammar correction interface
///
/// Corrects STT transcription errors using domain knowledge.
///
/// # Why This Matters
/// Indian language STT is error-prone. Domain-aware correction
/// significantly improves downstream LLM understanding.
///
/// # Implementations
/// - `LLMGrammarCorrector` - Uses small LLM for correction
/// - `NlpruleCorrector` - Rule-based (English/German only)
/// - `PassthroughCorrector` - No correction (for testing)
///
/// # Example
/// ```rust
/// let corrector: Box<dyn GrammarCorrector> = Box::new(
///     LLMGrammarCorrector::new(llm, domain_context)?
/// );
///
/// // STT output: "Kotak se gold Ion lena hai" (misheard "loan")
/// // Corrected:  "Kotak se gold loan lena hai"
/// let corrected = corrector.correct(stt_output, &context).await?;
/// ```
#[async_trait]
pub trait GrammarCorrector: Send + Sync + 'static {
    /// Correct grammar in text
    ///
    /// # Arguments
    /// * `text` - Text to correct
    /// * `context` - Domain context with vocabulary/phrases
    ///
    /// # Returns
    /// Corrected text (unchanged if no corrections needed)
    async fn correct(
        &self,
        text: &str,
        context: &DomainContext,
    ) -> Result<String, TextProcessingError>;

    /// Stream corrections sentence-by-sentence
    fn correct_stream(
        &self,
        text_stream: impl Stream<Item = String> + Send + 'static,
        context: &DomainContext,
    ) -> Box<dyn Stream<Item = Result<String, TextProcessingError>> + Send + Unpin>;

    /// Check if correction is available for language
    fn supports_language(&self, language: Language) -> bool;
}

/// Domain context for grammar correction
#[derive(Debug, Clone)]
pub struct DomainContext {
    /// Domain-specific vocabulary to preserve/correct
    pub vocabulary: Vec<String>,
    /// Common phrases in this domain
    pub phrases: Vec<String>,
    /// Entity types to preserve unchanged (names, numbers)
    pub preserve_entities: Vec<EntityType>,
    /// Language of the domain
    pub language: Language,
}

impl DomainContext {
    /// Create from TOML config file
    pub fn from_config(path: &Path) -> Result<Self, ConfigError> {
        // Load from domains/{domain}/grammar_context.toml
        todo!()
    }
}
```

### Translator

```rust
/// Language translation interface
///
/// # Design Decision: Translate-Think-Translate
/// LLMs reason better in English. We translate:
/// 1. Input: Indian language → English (for LLM)
/// 2. Output: English → Indian language (for user)
///
/// # Implementations
/// - `IndicTransOnnx` - IndicTrans2 via ONNX (preferred)
/// - `IndicTransGrpc` - IndicTrans2 via Python sidecar (fallback)
/// - `PassthroughTranslator` - No translation (for English)
///
/// # Latency Consideration
/// Translation adds ~50-100ms per sentence. Use streaming
/// to overlap with other processing.
#[async_trait]
pub trait Translator: Send + Sync + 'static {
    /// Translate text between languages
    ///
    /// # Arguments
    /// * `text` - Text to translate
    /// * `from` - Source language
    /// * `to` - Target language
    ///
    /// # Returns
    /// Translated text
    async fn translate(
        &self,
        text: &str,
        from: Language,
        to: Language,
    ) -> Result<String, TextProcessingError>;

    /// Detect language of text
    async fn detect_language(&self, text: &str) -> Result<Language, TextProcessingError>;

    /// Stream translation sentence-by-sentence
    fn translate_stream(
        &self,
        text_stream: impl Stream<Item = String> + Send + 'static,
        from: Language,
        to: Language,
    ) -> Box<dyn Stream<Item = Result<String, TextProcessingError>> + Send + Unpin>;

    /// Get supported language pairs
    fn supported_pairs(&self) -> &[(Language, Language)];

    /// Check if a specific pair is supported
    fn supports_pair(&self, from: Language, to: Language) -> bool {
        self.supported_pairs().contains(&(from, to))
    }
}
```

### PIIRedactor

```rust
/// PII (Personally Identifiable Information) detection and redaction
///
/// # India-Specific PII Types
/// - Aadhaar (12-digit ID)
/// - PAN (tax ID: ABCDE1234F)
/// - Phone numbers (+91...)
/// - Bank account numbers
/// - IFSC codes
///
/// # Why Redact?
/// 1. Compliance - Banking regulations require PII protection
/// 2. Logging - Redact before logging conversations
/// 3. Privacy - Customer trust
///
/// # Implementations
/// - `HybridPIIRedactor` - rust-bert NER + regex patterns
/// - `RegexPIIRedactor` - Regex-only (faster, less accurate)
/// - `LLMPIIRedactor` - LLM-based (most accurate, slowest)
#[async_trait]
pub trait PIIRedactor: Send + Sync + 'static {
    /// Detect PII entities in text
    ///
    /// # Arguments
    /// * `text` - Text to scan
    ///
    /// # Returns
    /// List of detected PII entities with positions and types
    async fn detect(&self, text: &str) -> Result<Vec<PIIEntity>, TextProcessingError>;

    /// Redact PII from text
    ///
    /// # Arguments
    /// * `text` - Text to redact
    /// * `strategy` - How to redact (mask, remove, etc.)
    ///
    /// # Returns
    /// Text with PII redacted according to strategy
    async fn redact(
        &self,
        text: &str,
        strategy: &RedactionStrategy,
    ) -> Result<String, TextProcessingError>;

    /// Detect and redact in one pass (more efficient)
    async fn detect_and_redact(
        &self,
        text: &str,
        strategy: &RedactionStrategy,
    ) -> Result<RedactionResult, TextProcessingError> {
        let entities = self.detect(text).await?;
        let redacted = self.redact(text, strategy).await?;
        Ok(RedactionResult { entities, redacted_text: redacted })
    }
}

/// Detected PII entity
#[derive(Debug, Clone)]
pub struct PIIEntity {
    /// Type of PII
    pub pii_type: PIIType,
    /// Original text
    pub text: String,
    /// Start position in original text
    pub start: usize,
    /// End position in original text
    pub end: usize,
    /// Detection confidence (0.0 - 1.0)
    pub confidence: f32,
}

/// Types of PII (India-specific)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PIIType {
    // Personal
    PersonName,
    DateOfBirth,
    Gender,

    // Contact
    PhoneNumber,
    Email,
    Address,

    // Government IDs (India)
    Aadhaar,
    PAN,
    VoterId,
    DrivingLicense,
    Passport,

    // Financial
    BankAccount,
    IFSC,
    CreditCard,
    DebitCard,
    UPI,

    // Loan-specific
    LoanAccountNumber,
    CompetitorLoanDetails,
}

/// Redaction strategy
#[derive(Debug, Clone)]
pub enum RedactionStrategy {
    /// Replace with [REDACTED]
    Mask,
    /// Replace with type: [AADHAAR]
    TypeMask,
    /// Partial mask: 1234****5678
    PartialMask { visible_chars: usize },
    /// Remove entirely
    Remove,
    /// Replace with synthetic data
    Synthesize,
    /// Custom replacement per type
    Custom(HashMap<PIIType, String>),
}

/// Result of redaction
#[derive(Debug, Clone)]
pub struct RedactionResult {
    pub entities: Vec<PIIEntity>,
    pub redacted_text: String,
}
```

### ComplianceChecker

```rust
/// Regulatory compliance checking interface
///
/// # Why Compliance Matters
/// Banking conversations must adhere to:
/// - RBI regulations
/// - SEBI guidelines (if investment-related)
/// - Fair lending practices
/// - Advertising standards
///
/// # What We Check
/// - Forbidden claims ("guaranteed approval")
/// - Missing disclaimers
/// - Rate/fee accuracy
/// - Competitor disparagement
///
/// # Implementations
/// - `RuleBasedChecker` - Regex + rule engine
/// - `LLMChecker` - LLM-based evaluation
/// - `HybridChecker` - Rules + LLM
#[async_trait]
pub trait ComplianceChecker: Send + Sync + 'static {
    /// Check text for compliance violations
    ///
    /// # Arguments
    /// * `text` - Text to check
    ///
    /// # Returns
    /// Compliance result with violations and suggestions
    async fn check(&self, text: &str) -> Result<ComplianceResult, TextProcessingError>;

    /// Modify text to be compliant
    ///
    /// Attempts to fix violations while preserving intent.
    ///
    /// # Arguments
    /// * `text` - Text to fix
    ///
    /// # Returns
    /// Compliant version of text
    async fn make_compliant(&self, text: &str) -> Result<String, TextProcessingError>;

    /// Check and fix in one pass
    async fn check_and_fix(
        &self,
        text: &str,
    ) -> Result<(ComplianceResult, String), TextProcessingError> {
        let result = self.check(text).await?;
        let fixed = if result.is_compliant {
            text.to_string()
        } else {
            self.make_compliant(text).await?
        };
        Ok((result, fixed))
    }

    /// Get compliance rules (for debugging/auditing)
    fn rules(&self) -> &ComplianceRules;
}

/// Compliance check result
#[derive(Debug, Clone)]
pub struct ComplianceResult {
    /// Overall compliance status
    pub is_compliant: bool,
    /// List of violations
    pub violations: Vec<ComplianceViolation>,
    /// Required additions (disclaimers, etc.)
    pub required_additions: Vec<RequiredAddition>,
    /// Suggested rewrites
    pub suggestions: Vec<SuggestedRewrite>,
}

/// A compliance violation
#[derive(Debug, Clone)]
pub struct ComplianceViolation {
    /// Rule that was violated
    pub rule_id: String,
    /// Human-readable description
    pub description: String,
    /// Severity level
    pub severity: Severity,
    /// Position in text
    pub text_span: Option<(usize, usize)>,
    /// Offending text
    pub offending_text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational - can proceed
    Info,
    /// Warning - should fix but can proceed
    Warning,
    /// Error - should not proceed
    Error,
    /// Critical - must not proceed under any circumstances
    Critical,
}

/// Required addition (e.g., disclaimer)
#[derive(Debug, Clone)]
pub struct RequiredAddition {
    pub text: String,
    pub position: AdditionPosition,
    pub reason: String,
}

#[derive(Debug, Clone, Copy)]
pub enum AdditionPosition {
    Beginning,
    End,
    AfterClaim, // After the triggering claim
}
```

---

## Intelligence Interfaces

### LanguageModel

```rust
/// Large Language Model interface
///
/// # Implementations
/// - `KalosmLLM` - Local inference via Kalosm
/// - `OllamaLLM` - Local via Ollama API
/// - `ClaudeLLM` - Anthropic Claude API
/// - `OpenAILLM` - OpenAI API
///
/// # Streaming Requirement
/// For voice agents, streaming is mandatory. The `generate_stream`
/// method should be the primary method used in production.
#[async_trait]
pub trait LanguageModel: Send + Sync + 'static {
    /// Generate completion (non-streaming)
    ///
    /// Use for short, time-insensitive generations.
    async fn generate(
        &self,
        request: GenerateRequest,
    ) -> Result<GenerateResponse, LLMError>;

    /// Generate with streaming
    ///
    /// Primary method for voice agents. Emits tokens as generated.
    fn generate_stream(
        &self,
        request: GenerateRequest,
    ) -> Box<dyn Stream<Item = Result<StreamChunk, LLMError>> + Send + Unpin>;

    /// Generate with tool use
    ///
    /// The LLM may choose to call tools instead of/in addition to
    /// generating text.
    async fn generate_with_tools(
        &self,
        request: GenerateRequest,
        tools: &[ToolDefinition],
    ) -> Result<GenerateResponse, LLMError>;

    /// Count tokens in text (for context management)
    fn count_tokens(&self, text: &str) -> usize;

    /// Get model's context window size
    fn context_window(&self) -> usize;

    /// Get model information
    fn model_info(&self) -> &LLMModelInfo;
}

/// Generation request
#[derive(Debug, Clone)]
pub struct GenerateRequest {
    /// System prompt
    pub system: Option<String>,
    /// User/assistant message history
    pub messages: Vec<Message>,
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Temperature (0.0 = deterministic, 1.0 = creative)
    pub temperature: f32,
    /// Top-p sampling
    pub top_p: f32,
    /// Stop sequences
    pub stop: Vec<String>,
    /// Whether to stream
    pub stream: bool,
}

/// Chat message
#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// Generation response
#[derive(Debug, Clone)]
pub struct GenerateResponse {
    /// Generated text
    pub text: String,
    /// Tool calls (if any)
    pub tool_calls: Vec<ToolCall>,
    /// Usage statistics
    pub usage: Usage,
    /// Stop reason
    pub stop_reason: StopReason,
}

/// Stream chunk
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// Token(s) in this chunk
    pub text: String,
    /// Is this the final chunk?
    pub is_final: bool,
    /// Tool call (if this chunk contains one)
    pub tool_call: Option<ToolCall>,
}

/// Token usage
#[derive(Debug, Clone, Default)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum StopReason {
    EndOfText,
    MaxTokens,
    StopSequence,
    ToolUse,
}
```

### Retriever

```rust
/// Document retrieval interface for RAG
///
/// # Hybrid Retrieval
/// Best practice is combining semantic (vector) and lexical (BM25) search.
///
/// # Implementations
/// - `HybridRetriever` - Qdrant vectors + tantivy BM25
/// - `SemanticRetriever` - Vector-only (Qdrant)
/// - `KeywordRetriever` - BM25-only (tantivy)
/// - `AgenticRetriever` - Multi-step with query rewriting
#[async_trait]
pub trait Retriever: Send + Sync + 'static {
    /// Retrieve documents matching query
    ///
    /// # Arguments
    /// * `query` - Search query
    /// * `options` - Retrieval options (filters, limits, etc.)
    ///
    /// # Returns
    /// List of relevant documents with scores
    async fn retrieve(
        &self,
        query: &str,
        options: &RetrieveOptions,
    ) -> Result<Vec<Document>, RAGError>;

    /// Agentic retrieval with multi-step refinement
    ///
    /// Iteratively:
    /// 1. Retrieve documents
    /// 2. Check if sufficient to answer query
    /// 3. If not, rewrite query and repeat
    ///
    /// # Arguments
    /// * `query` - Original query
    /// * `context` - Conversation context
    /// * `max_iterations` - Maximum refinement iterations
    async fn retrieve_agentic(
        &self,
        query: &str,
        context: &ConversationContext,
        max_iterations: usize,
    ) -> Result<Vec<Document>, RAGError>;

    /// Add documents to the index
    async fn index(&self, documents: &[Document]) -> Result<(), RAGError>;

    /// Delete documents from the index
    async fn delete(&self, ids: &[String]) -> Result<(), RAGError>;
}

/// Retrieval options
#[derive(Debug, Clone, Default)]
pub struct RetrieveOptions {
    /// Maximum documents to return
    pub limit: usize,
    /// Minimum relevance score (0.0 - 1.0)
    pub min_score: f32,
    /// Filter by document type
    pub doc_types: Option<Vec<DocumentType>>,
    /// Filter by metadata
    pub metadata_filter: Option<MetadataFilter>,
    /// Whether to use hybrid retrieval
    pub hybrid: bool,
    /// Semantic vs BM25 weight (0.0 = all BM25, 1.0 = all semantic)
    pub semantic_weight: f32,
}

/// Retrieved document
#[derive(Debug, Clone)]
pub struct Document {
    /// Unique identifier
    pub id: String,
    /// Document content
    pub content: String,
    /// Document type
    pub doc_type: DocumentType,
    /// Metadata
    pub metadata: HashMap<String, Value>,
    /// Relevance score
    pub score: f32,
}

/// Document types for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentType {
    Product,
    FAQ,
    Competitor,
    Regulation,
    Objection,
    Script,
    Policy,
}
```

### Tool

```rust
/// Function tool interface for agent actions
///
/// # Examples of Tools
/// - `SavingsCalculator` - Calculate savings vs competitor
/// - `EligibilityChecker` - Check loan eligibility
/// - `AppointmentBooker` - Book branch visit
/// - `RateQuoter` - Get current rates
///
/// # Design Pattern
/// Tools should be stateless. Any state should be passed in parameters.
#[async_trait]
pub trait Tool: Send + Sync + 'static {
    /// Tool name (used by LLM to invoke)
    fn name(&self) -> &str;

    /// Human-readable description (included in LLM prompt)
    fn description(&self) -> &str;

    /// JSON Schema for parameters
    fn parameters_schema(&self) -> Value;

    /// Execute the tool
    ///
    /// # Arguments
    /// * `params` - Tool parameters as JSON
    ///
    /// # Returns
    /// Tool result as JSON
    async fn execute(&self, params: Value) -> Result<Value, ToolError>;

    /// Validate parameters before execution
    fn validate(&self, params: &Value) -> Result<(), ToolError> {
        // Default implementation uses JSON schema validation
        // Implementations can override for custom validation
        Ok(())
    }
}

/// Tool definition for LLM
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value, // JSON Schema
}

impl<T: Tool> From<&T> for ToolDefinition {
    fn from(tool: &T) -> Self {
        Self {
            name: tool.name().to_string(),
            description: tool.description().to_string(),
            parameters: tool.parameters_schema(),
        }
    }
}

/// Tool call from LLM
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

/// Tool execution result
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub success: bool,
    pub output: Value,
    pub error: Option<String>,
}
```

---

## Conversation Interfaces

### ConversationFSM

```rust
/// Finite State Machine for conversation flow
///
/// # Design Philosophy
/// Conversations have structure. A state machine ensures:
/// - Predictable flow
/// - Clear state transitions
/// - Checkpointing for recovery
/// - Metrics per state
///
/// # States
/// - `Idle` → `Greeting` → `Discovery` → `NeedsAnalysis`
/// - `NeedsAnalysis` → `Pitch` → `Closing`
/// - Any state → `ObjectionHandling` → (previous state)
/// - Any state → End states (`Converted`, `FollowUp`, `Declined`, `Escalated`)
#[async_trait]
pub trait ConversationFSM: Send + Sync + 'static {
    /// Get current state
    fn state(&self) -> &ConversationState;

    /// Process event and transition state
    ///
    /// # Arguments
    /// * `event` - Event that occurred
    ///
    /// # Returns
    /// Actions to take based on transition
    async fn transition(
        &mut self,
        event: ConversationEvent,
    ) -> Result<Vec<Action>, FSMError>;

    /// Check if transition is valid
    fn can_transition(&self, event: &ConversationEvent) -> bool;

    /// Get valid transitions from current state
    fn valid_transitions(&self) -> Vec<ConversationEvent>;

    /// Checkpoint current state (for recovery)
    fn checkpoint(&mut self);

    /// Restore from checkpoint
    fn restore(&mut self, checkpoint_index: usize) -> Result<(), FSMError>;

    /// Get conversation context
    fn context(&self) -> &ConversationContext;

    /// Update conversation context
    fn update_context(&mut self, update: ContextUpdate);
}

/// Conversation states
#[derive(Debug, Clone, PartialEq)]
pub enum ConversationState {
    /// Waiting for call
    Idle,
    /// Initial greeting
    Greeting,
    /// Understanding customer needs
    Discovery,
    /// Analyzing requirements
    NeedsAnalysis,
    /// Presenting offer
    Pitch,
    /// Comparing with competitors
    Comparison,
    /// Handling objections
    ObjectionHandling { objection_type: ObjectionType },
    /// Closing the deal
    Closing,

    // End states
    /// Successfully converted
    Converted {
        appointment: Option<String>,
        loan_amount: Option<u64>,
    },
    /// Customer needs follow-up
    FollowUp {
        reason: String,
        scheduled: Option<String>,
    },
    /// Customer declined
    Declined { reason: String },
    /// Escalated to human
    Escalated { to: String, reason: String },
}

/// Events that trigger state transitions
#[derive(Debug, Clone)]
pub enum ConversationEvent {
    // Lifecycle
    CallStarted { customer_id: Option<String> },
    CallEnded,

    // Speech
    UserSpeaking,
    UserSilence { duration: Duration },
    TranscriptReady { text: String, is_final: bool },

    // Agent
    ResponseGenerated { text: String },
    ResponseDelivered,

    // User actions
    UserIntent { intent: Intent },
    UserAgreement,
    UserRefusal { reason: Option<String> },
    UserQuestion { topic: String },
    UserObjection { objection_type: ObjectionType },

    // Interrupts
    BargeIn,
    Timeout { stage: String },

    // Tools
    ToolCallRequested { tool: String },
    ToolResultReady { tool: String, result: Value },

    // Errors
    Error { error: String },
}

/// User intent classification
#[derive(Debug, Clone)]
pub enum Intent {
    // Positive
    Interest,
    ReadyToProceed,
    WantsDetails,

    // Neutral
    Question,
    Clarification,
    Comparison,

    // Negative
    NotInterested,
    Objection,
    Busy,
    WrongPerson,

    // Other
    Greeting,
    Farewell,
    Unknown,
}

/// Objection types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectionType {
    Rate,           // Interest rate too high
    Trust,          // Don't trust banks/new lender
    Timing,         // Not the right time
    Competition,    // Prefer current lender
    Process,        // Process seems complicated
    Safety,         // Concerned about gold safety
    Other(String),
}

/// Actions to take after state transition
#[derive(Debug, Clone)]
pub enum Action {
    // Speech
    StartListening,
    StopListening,
    StartSpeaking { text: String },
    StopSpeaking,

    // Context
    LoadCustomerProfile { customer_id: String },
    UpdateContext { key: String, value: Value },

    // Tools
    ExecuteTool { name: String, params: Value },

    // State
    Checkpoint,
    EndConversation { outcome: ConversationOutcome },
    Escalate { to: String, reason: String },

    // Metrics
    RecordMetric { name: String, value: f64 },
}

/// Final conversation outcome
#[derive(Debug, Clone)]
pub enum ConversationOutcome {
    Converted,
    FollowUp,
    Declined,
    Escalated,
    Error,
}
```

---

## Memory Interfaces

### ConversationMemory

```rust
/// Conversation memory interface
///
/// # Memory Levels
/// 1. **Immediate** - Current turn context (in ConversationContext)
/// 2. **Session** - Full conversation history (stored during call)
/// 3. **Cross-session** - Persisted summaries (across calls)
///
/// # Design Decision
/// Default to stateless (no cross-session memory) for simplicity.
/// Memory can be enabled via configuration.
#[async_trait]
pub trait ConversationMemory: Send + Sync + 'static {
    /// Store conversation summary after call ends
    async fn store(
        &self,
        customer_id: &str,
        summary: &ConversationSummary,
    ) -> Result<(), MemoryError>;

    /// Retrieve previous conversation summaries
    async fn retrieve(
        &self,
        customer_id: &str,
        limit: usize,
    ) -> Result<Vec<ConversationSummary>, MemoryError>;

    /// Check if customer has prior interactions
    async fn has_history(&self, customer_id: &str) -> Result<bool, MemoryError>;

    /// Clear memory for customer (GDPR compliance)
    async fn clear(&self, customer_id: &str) -> Result<(), MemoryError>;
}

/// Conversation summary for cross-session memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSummary {
    /// Conversation ID
    pub conversation_id: String,
    /// Customer ID
    pub customer_id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Duration in seconds
    pub duration_secs: u64,
    /// Final outcome
    pub outcome: ConversationOutcome,
    /// Key topics discussed
    pub topics: Vec<String>,
    /// Customer preferences learned
    pub preferences: HashMap<String, Value>,
    /// Objections raised
    pub objections: Vec<ObjectionType>,
    /// Follow-up actions
    pub follow_ups: Vec<String>,
    /// Compressed transcript summary
    pub summary_text: String,
}

/// Stateless memory implementation (no-op)
pub struct StatelessMemory;

#[async_trait]
impl ConversationMemory for StatelessMemory {
    async fn store(&self, _: &str, _: &ConversationSummary) -> Result<(), MemoryError> {
        Ok(()) // No-op
    }

    async fn retrieve(&self, _: &str, _: usize) -> Result<Vec<ConversationSummary>, MemoryError> {
        Ok(vec![]) // Always empty
    }

    async fn has_history(&self, _: &str) -> Result<bool, MemoryError> {
        Ok(false) // Always no history
    }

    async fn clear(&self, _: &str) -> Result<(), MemoryError> {
        Ok(()) // No-op
    }
}

/// Session summary memory (Redis/SQLite backed)
pub struct SessionSummaryMemory {
    store: Box<dyn KeyValueStore>,
}

// Implementation would use the KV store to persist summaries
```

---

## Observability Interfaces

### MetricsCollector

```rust
/// Metrics collection interface
///
/// # Key Metrics
/// - Latency (per pipeline stage)
/// - Error rates
/// - Conversation funnel
/// - Customer sentiment
#[async_trait]
pub trait MetricsCollector: Send + Sync + 'static {
    /// Record a timing metric
    fn record_timing(&self, name: &str, duration: Duration, tags: &[(&str, &str)]);

    /// Record a counter
    fn increment(&self, name: &str, value: u64, tags: &[(&str, &str)]);

    /// Record a gauge
    fn gauge(&self, name: &str, value: f64, tags: &[(&str, &str)]);

    /// Record histogram value
    fn histogram(&self, name: &str, value: f64, tags: &[(&str, &str)]);

    /// Start a timer (returns handle that records on drop)
    fn start_timer(&self, name: &str, tags: &[(&str, &str)]) -> Box<dyn TimerHandle>;

    /// Record conversation funnel event
    fn record_funnel_event(&self, event: FunnelEvent);

    /// Record sentiment observation
    fn record_sentiment(&self, sentiment: SentimentObservation);
}

/// Funnel events for conversion tracking
#[derive(Debug, Clone)]
pub enum FunnelEvent {
    CallStarted { conversation_id: String, segment: CustomerSegment },
    GreetingDelivered { conversation_id: String },
    CustomerEngaged { conversation_id: String },
    NeedsIdentified { conversation_id: String },
    PitchDelivered { conversation_id: String },
    ObjectionRaised { conversation_id: String, objection: ObjectionType },
    ObjectionResolved { conversation_id: String },
    ClosingAttempted { conversation_id: String },
    Converted { conversation_id: String },
    FollowUpScheduled { conversation_id: String },
    Declined { conversation_id: String, reason: String },
}

/// Sentiment observation
#[derive(Debug, Clone)]
pub struct SentimentObservation {
    pub conversation_id: String,
    pub timestamp: Instant,
    pub sentiment_score: f32,  // -1.0 (negative) to 1.0 (positive)
    pub confidence: f32,
    pub trigger_text: Option<String>,
}

/// Timer handle trait
pub trait TimerHandle: Send {
    /// Stop and record the timer
    fn stop(self: Box<Self>);
}
```

---

## Error Handling

### Error Types

```rust
/// Speech-related errors
#[derive(Debug, thiserror::Error)]
pub enum SpeechError {
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),

    #[error("Language not supported: {0:?}")]
    LanguageNotSupported(Language),

    #[error("Audio format error: {0}")]
    AudioFormatError(String),

    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),

    #[error("Synthesis failed: {0}")]
    SynthesisFailed(String),

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Text processing errors
#[derive(Debug, thiserror::Error)]
pub enum TextProcessingError {
    #[error("Grammar correction failed: {0}")]
    GrammarError(String),

    #[error("Translation failed: {0}")]
    TranslationError(String),

    #[error("PII detection failed: {0}")]
    PIIError(String),

    #[error("Compliance check failed: {0}")]
    ComplianceError(String),

    #[error("Language pair not supported: {0:?} -> {1:?}")]
    UnsupportedLanguagePair(Language, Language),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// LLM errors
#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("Model not available: {0}")]
    ModelNotAvailable(String),

    #[error("Context too long: {0} tokens (max: {1})")]
    ContextTooLong(usize, usize),

    #[error("Rate limited: retry after {0:?}")]
    RateLimited(Duration),

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("Tool execution failed: {0}")]
    ToolError(String),

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("API error: {0}")]
    ApiError(String),
}

/// RAG errors
#[derive(Debug, thiserror::Error)]
pub enum RAGError {
    #[error("Index not ready")]
    IndexNotReady,

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("No documents found")]
    NoDocuments,

    #[error("Embedding failed: {0}")]
    EmbeddingFailed(String),

    #[error("Reranking failed: {0}")]
    RerankingFailed(String),
}

/// FSM errors
#[derive(Debug, thiserror::Error)]
pub enum FSMError {
    #[error("Invalid transition from {0:?} with event {1:?}")]
    InvalidTransition(ConversationState, ConversationEvent),

    #[error("No checkpoint at index {0}")]
    NoCheckpoint(usize),

    #[error("State machine corrupted")]
    Corrupted,
}

/// Tool errors
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Timeout after {0:?}")]
    Timeout(Duration),
}

/// Memory errors
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Customer not found: {0}")]
    CustomerNotFound(String),
}
```

---

## Usage Examples

### Complete Pipeline Example

```rust
use voice_agent::{
    core::traits::*,
    speech::{SherpaStt, SherpaTts},
    text_processing::{LLMGrammarCorrector, IndicTranslator, HybridPIIRedactor},
    llm::OllamaLLM,
    rag::HybridRetriever,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize components
    let stt: Arc<dyn SpeechToText> = Arc::new(SherpaStt::new(&stt_config)?);
    let tts: Arc<dyn TextToSpeech> = Arc::new(SherpaTts::new(&tts_config)?);
    let llm: Arc<dyn LanguageModel> = Arc::new(OllamaLLM::new(&llm_config)?);
    let grammar: Arc<dyn GrammarCorrector> = Arc::new(LLMGrammarCorrector::new(llm.clone(), domain_context)?);
    let translator: Arc<dyn Translator> = Arc::new(IndicTranslator::new(&trans_config).await?);
    let pii: Arc<dyn PIIRedactor> = Arc::new(HybridPIIRedactor::new()?);
    let retriever: Arc<dyn Retriever> = Arc::new(HybridRetriever::new(&rag_config).await?);

    // Process a conversation turn
    async fn process_turn(
        audio: AudioFrame,
        context: &mut ConversationContext,
        components: &Components,
    ) -> Result<AudioFrame, Error> {
        // 1. Speech to text
        let transcript = components.stt.transcribe(&audio).await?;

        // 2. Grammar correction
        let corrected = components.grammar
            .correct(&transcript.text, &components.domain_context)
            .await?;

        // 3. Translate to English (if needed)
        let english = if transcript.language != Language::English {
            components.translator
                .translate(&corrected, transcript.language, Language::English)
                .await?
        } else {
            corrected
        };

        // 4. Retrieve context
        let docs = components.retriever
            .retrieve(&english, &RetrieveOptions::default())
            .await?;

        // 5. Generate response
        let response = components.llm
            .generate(GenerateRequest {
                system: Some(build_system_prompt(context, &docs)),
                messages: context.history.clone(),
                max_tokens: 150,
                temperature: 0.3,
                ..Default::default()
            })
            .await?;

        // 6. Translate back to user's language
        let translated = if transcript.language != Language::English {
            components.translator
                .translate(&response.text, Language::English, transcript.language)
                .await?
        } else {
            response.text
        };

        // 7. PII redaction (for logging)
        let redacted = components.pii
            .redact(&translated, &RedactionStrategy::PartialMask { visible_chars: 4 })
            .await?;
        tracing::info!(response = %redacted, "Agent response");

        // 8. Synthesize speech
        let audio = components.tts
            .synthesize(&translated, &context.voice_config)
            .await?;

        Ok(audio)
    }

    Ok(())
}
```

---

## Next Steps

See related documentation:
- [Pipeline Design](../pipeline/audio-pipeline.md)
- [Text Processing](../pipeline/text-processing-pipeline.md)
- [RAG Strategy](../rag/agentic-rag-strategy.md)
- [Rust Ecosystem](../rust-ecosystem.md)
