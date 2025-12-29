# Implementation Roadmap: Architecture Alignment

> **Goal:** Implement ARCHITECTURE_v2.md as documented
> **Approach:** Complete implementation in 6 phases
> **Estimated Duration:** 8-12 weeks

---

## Overview

This roadmap details the implementation work required to align the voice-agent-rust codebase with ARCHITECTURE_v2.md. Based on the gap analysis in `13-ARCHITECTURE-GAP-ANALYSIS.md`, we have identified **6 major implementation phases**.

### Phase Summary

| Phase | Component | Effort | Priority | Dependencies |
|-------|-----------|--------|----------|--------------|
| **1** | Core Traits & Types | 2 weeks | P0 | None |
| **2** | Text Processing Pipeline | 3 weeks | P0 | Phase 1 |
| **3** | Pipeline Architecture | 2 weeks | P1 | Phase 1 |
| **4** | RAG Enhancement | 1 week | P1 | Phase 1 |
| **5** | Personalization Engine | 1 week | P2 | Phase 1 |
| **6** | Domain Configuration | 2 weeks | P2 | Phase 2, 5 |

---

## Phase 1: Core Traits & Types (P0)

### 1.1 Create Traits Module

**Location:** `crates/core/src/traits/`

**Files to Create:**
```
crates/core/src/traits/
├── mod.rs              # Re-exports
├── speech.rs           # SpeechToText, TextToSpeech
├── llm.rs              # LanguageModel
├── retriever.rs        # Retriever
├── text_processing.rs  # GrammarCorrector, Translator, PIIRedactor, ComplianceChecker
└── pipeline.rs         # FrameProcessor
```

**Traits to Implement:**

#### 1.1.1 SpeechToText Trait
```rust
// crates/core/src/traits/speech.rs
#[async_trait]
pub trait SpeechToText: Send + Sync + 'static {
    async fn transcribe(&self, audio: &AudioFrame) -> Result<TranscriptFrame>;
    fn transcribe_stream(
        &self,
        audio_stream: impl Stream<Item = AudioFrame> + Send,
    ) -> impl Stream<Item = Result<TranscriptFrame>> + Send;
    fn supported_languages(&self) -> &[Language];
}
```

#### 1.1.2 TextToSpeech Trait
```rust
#[async_trait]
pub trait TextToSpeech: Send + Sync + 'static {
    async fn synthesize(&self, text: &str, config: &VoiceConfig) -> Result<AudioFrame>;
    fn synthesize_stream(
        &self,
        text_stream: impl Stream<Item = String> + Send,
        config: &VoiceConfig,
    ) -> impl Stream<Item = Result<AudioFrame>> + Send;
    fn available_voices(&self) -> &[VoiceInfo];
}
```

#### 1.1.3 LanguageModel Trait
```rust
#[async_trait]
pub trait LanguageModel: Send + Sync + 'static {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse>;
    fn generate_stream(
        &self,
        request: GenerateRequest,
    ) -> impl Stream<Item = Result<StreamChunk>> + Send;
    async fn generate_with_tools(
        &self,
        request: GenerateRequest,
        tools: &[ToolDefinition],
    ) -> Result<GenerateResponse>;
}
```

#### 1.1.4 Retriever Trait
```rust
#[async_trait]
pub trait Retriever: Send + Sync + 'static {
    async fn retrieve(&self, query: &str, options: &RetrieveOptions) -> Result<Vec<Document>>;
    async fn retrieve_agentic(
        &self,
        query: &str,
        context: &ConversationContext,
        max_iterations: usize,
    ) -> Result<Vec<Document>>;
}
```

#### 1.1.5 GrammarCorrector Trait
```rust
#[async_trait]
pub trait GrammarCorrector: Send + Sync + 'static {
    async fn correct(&self, text: &str, context: &DomainContext) -> Result<String>;
    fn correct_stream(
        &self,
        text_stream: impl Stream<Item = String> + Send,
        context: &DomainContext,
    ) -> impl Stream<Item = Result<String>> + Send;
}
```

#### 1.1.6 Translator Trait
```rust
#[async_trait]
pub trait Translator: Send + Sync + 'static {
    async fn translate(&self, text: &str, from: Language, to: Language) -> Result<String>;
    async fn detect_language(&self, text: &str) -> Result<Language>;
    fn translate_stream(
        &self,
        text_stream: impl Stream<Item = String> + Send,
        from: Language,
        to: Language,
    ) -> impl Stream<Item = Result<String>> + Send;
}
```

#### 1.1.7 PIIRedactor Trait
```rust
#[async_trait]
pub trait PIIRedactor: Send + Sync + 'static {
    async fn detect(&self, text: &str) -> Result<Vec<PIIEntity>>;
    async fn redact(&self, text: &str, strategy: &RedactionStrategy) -> Result<String>;
}
```

#### 1.1.8 ComplianceChecker Trait
```rust
#[async_trait]
pub trait ComplianceChecker: Send + Sync + 'static {
    async fn check(&self, text: &str) -> Result<ComplianceResult>;
    async fn make_compliant(&self, text: &str) -> Result<String>;
}
```

#### 1.1.9 FrameProcessor Trait
```rust
#[async_trait]
pub trait FrameProcessor: Send + Sync + 'static {
    async fn process(
        &self,
        frame: Frame,
        context: &mut ProcessorContext,
    ) -> Result<Vec<Frame>>;
    fn name(&self) -> &'static str;
}
```

### 1.2 Update Core Types

**Files to Modify:**
- `crates/core/src/audio.rs` - Add documented fields if missing
- `crates/core/src/transcript.rs` - Rename/alias TranscriptResult to TranscriptFrame
- `crates/core/src/conversation.rs` - Add missing ConversationState variants

**New Files:**
```
crates/core/src/
├── language.rs         # Language enum (22 languages)
├── voice_config.rs     # VoiceConfig, VoiceInfo
├── pii.rs              # PIIEntity, PIIType, RedactionStrategy
├── compliance.rs       # ComplianceResult, ComplianceViolation
├── domain_context.rs   # DomainContext
└── llm_types.rs        # GenerateRequest, GenerateResponse, StreamChunk
```

### 1.3 Checklist

- [ ] Create `crates/core/src/traits/mod.rs`
- [ ] Implement SpeechToText trait
- [ ] Implement TextToSpeech trait
- [ ] Implement LanguageModel trait
- [ ] Implement Retriever trait
- [ ] Implement GrammarCorrector trait
- [ ] Implement Translator trait
- [ ] Implement PIIRedactor trait
- [ ] Implement ComplianceChecker trait
- [ ] Implement FrameProcessor trait
- [ ] Create Language enum with 22 variants
- [ ] Create VoiceConfig struct
- [ ] Create PIIEntity, PIIType enums
- [ ] Create ComplianceResult types
- [ ] Create DomainContext struct
- [ ] Create LLM request/response types
- [ ] Add trait re-exports to core lib.rs
- [ ] Update existing backends to implement new traits

---

## Phase 2: Text Processing Pipeline (P0)

### 2.1 Create text_processing Crate

**Location:** `crates/text_processing/`

**Structure:**
```
crates/text_processing/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── grammar/
    │   ├── mod.rs
    │   └── llm_corrector.rs      # LLMGrammarCorrector
    ├── translation/
    │   ├── mod.rs
    │   ├── indictrans.rs         # IndicTranslator (ONNX)
    │   └── grpc.rs               # GrpcTranslator (fallback)
    ├── pii/
    │   ├── mod.rs
    │   ├── detector.rs           # HybridPIIDetector
    │   └── patterns.rs           # Regex patterns for India
    ├── compliance/
    │   ├── mod.rs
    │   └── checker.rs            # RuleBasedComplianceChecker
    └── simplify/
        ├── mod.rs
        └── tts_prep.rs           # Text simplification for TTS
```

### 2.2 Grammar Correction

**Implementation:** `LLMGrammarCorrector`

```rust
// crates/text_processing/src/grammar/llm_corrector.rs
pub struct LLMGrammarCorrector {
    llm: Arc<dyn LanguageModel>,
    domain_context: DomainContext,
}

impl LLMGrammarCorrector {
    pub fn new(llm: Arc<dyn LanguageModel>, domain: &str) -> Self;
    fn build_prompt(&self, text: &str) -> String;
}

impl GrammarCorrector for LLMGrammarCorrector {
    async fn correct(&self, text: &str, context: &DomainContext) -> Result<String>;
    fn correct_stream(...) -> impl Stream<...>;
}
```

### 2.3 Translation

**Implementation:** `IndicTranslator` with ONNX + gRPC fallback

```rust
// crates/text_processing/src/translation/indictrans.rs
pub struct IndicTranslator {
    onnx_session: Option<ort::Session>,
    grpc_client: Option<IndicTransGrpcClient>,
    tokenizer: IndicTransTokenizer,
}

impl IndicTranslator {
    pub async fn new(config: &TranslationConfig) -> Result<Self>;
    async fn load_onnx(path: &Path) -> Result<ort::Session>;
}

impl Translator for IndicTranslator {
    async fn translate(&self, text: &str, from: Language, to: Language) -> Result<String>;
    async fn detect_language(&self, text: &str) -> Result<Language>;
    fn translate_stream(...) -> impl Stream<...>;
}
```

### 2.4 PII Detection

**Implementation:** `HybridPIIDetector` with regex + NER

```rust
// crates/text_processing/src/pii/detector.rs
pub struct HybridPIIDetector {
    ner_model: Option<Arc<NERModel>>,
    regex_patterns: HashMap<PIIType, Regex>,
}

// Patterns for India-specific PII:
// - Aadhaar: \d{4}\s?\d{4}\s?\d{4}
// - PAN: [A-Z]{5}[0-9]{4}[A-Z]
// - Phone: (\+91)?[6-9]\d{9}
// - IFSC: [A-Z]{4}0[A-Z0-9]{6}

impl PIIRedactor for HybridPIIDetector {
    async fn detect(&self, text: &str) -> Result<Vec<PIIEntity>>;
    async fn redact(&self, text: &str, strategy: &RedactionStrategy) -> Result<String>;
}
```

### 2.5 Compliance Checking

**Implementation:** `RuleBasedComplianceChecker`

```rust
// crates/text_processing/src/compliance/checker.rs
pub struct RuleBasedComplianceChecker {
    rules: ComplianceRules,
    forbidden_patterns: Vec<Regex>,
}

impl RuleBasedComplianceChecker {
    pub fn from_config(path: &Path) -> Result<Self>;
}

impl ComplianceChecker for RuleBasedComplianceChecker {
    async fn check(&self, text: &str) -> Result<ComplianceResult>;
    async fn make_compliant(&self, text: &str) -> Result<String>;
}
```

### 2.6 Checklist

- [ ] Create `crates/text_processing/Cargo.toml`
- [ ] Create module structure
- [ ] Implement LLMGrammarCorrector
- [ ] Implement IndicTranslator (ONNX)
- [ ] Implement GrpcTranslator (fallback)
- [ ] Implement HybridPIIDetector
- [ ] Add India-specific PII regex patterns
- [ ] Implement RuleBasedComplianceChecker
- [ ] Create compliance.toml config format
- [ ] Implement TTS text simplification
- [ ] Add unit tests for each component
- [ ] Add integration tests
- [ ] Add to workspace Cargo.toml

---

## Phase 3: Pipeline Architecture (P1)

### 3.1 Create Frame-Based Pipeline

**Update:** `crates/pipeline/src/`

**New Files:**
```
crates/pipeline/src/
├── frame.rs            # Frame enum (all variants)
├── processor.rs        # FrameProcessor implementations
├── orchestrator_v2.rs  # New frame-based orchestrator
├── streaming.rs        # SentenceDetector, SentenceAccumulator
└── interrupt.rs        # InterruptHandler with modes
```

### 3.2 Frame Enum

```rust
// crates/pipeline/src/frame.rs
#[derive(Debug, Clone)]
pub enum Frame {
    // Audio frames
    AudioInput(AudioFrame),
    AudioOutput(AudioFrame),

    // Speech frames
    TranscriptPartial(TranscriptFrame),
    TranscriptFinal(TranscriptFrame),

    // Text processing frames
    GrammarCorrected(String),
    Translated(String, Language, Language),
    ComplianceChecked(String, ComplianceResult),
    PIIRedacted(String),

    // LLM frames
    LLMChunk(String),
    LLMComplete(String),
    ToolCall(ToolCall),
    ToolResult(ToolResult),

    // Control frames
    UserSpeaking,
    UserSilence(Duration),
    BargeIn,
    EndOfTurn,

    // System frames
    StateChange(ConversationState),
    Error(PipelineError),
    Metrics(MetricsEvent),
}
```

### 3.3 Processor Chain

```rust
// crates/pipeline/src/orchestrator_v2.rs
pub struct Pipeline {
    processors: Vec<Arc<dyn FrameProcessor>>,
    input_tx: mpsc::Sender<Frame>,
    output_rx: mpsc::Receiver<Frame>,
}

impl Pipeline {
    pub fn builder() -> PipelineBuilder;
    pub async fn run(&mut self) -> Result<()>;
}

pub struct PipelineBuilder {
    processors: Vec<Arc<dyn FrameProcessor>>,
}

impl PipelineBuilder {
    pub fn add_processor(self, p: impl FrameProcessor) -> Self;
    pub fn build(self) -> Pipeline;
}
```

### 3.4 Sentence Streaming

```rust
// crates/pipeline/src/streaming.rs
pub struct SentenceDetector {
    terminators: HashSet<char>,  // ., !, ?, ।, ॥
}

pub struct SentenceAccumulator {
    buffer: String,
    detector: SentenceDetector,
}

impl SentenceAccumulator {
    pub fn add(&mut self, chunk: &str) -> Vec<String>;
    pub fn flush(&mut self) -> Option<String>;
}

pub struct LLMToTTSStreamer {
    accumulator: SentenceAccumulator,
    tts: Arc<dyn TextToSpeech>,
    voice_config: VoiceConfig,
}

impl FrameProcessor for LLMToTTSStreamer { ... }
```

### 3.5 Interrupt Handling

```rust
// crates/pipeline/src/interrupt.rs
#[derive(Debug, Clone, Copy)]
pub enum InterruptMode {
    SentenceBoundary,
    Immediate,
    WordBoundary,
}

pub struct InterruptConfig {
    pub mode: InterruptMode,
    pub vad_sensitivity: f32,
    pub min_speech_duration_ms: u64,
    pub silence_timeout_ms: u64,
}

pub struct InterruptHandler {
    config: InterruptConfig,
    state: InterruptState,
    vad: Arc<dyn VadEngine>,
}

enum InterruptState {
    Idle,
    AgentSpeaking { start_time: Instant, current_sentence: String },
    UserInterrupting { speech_start: Instant, accumulated_duration: Duration },
}
```

### 3.6 Checklist

- [ ] Create Frame enum with all variants
- [ ] Implement ProcessorContext
- [ ] Create Pipeline orchestrator with channels
- [ ] Implement SentenceDetector (multilingual)
- [ ] Implement SentenceAccumulator
- [ ] Implement LLMToTTSStreamer
- [ ] Implement InterruptHandler with modes
- [ ] Create standard processors (VAD, STT, TTS wrappers)
- [ ] Add PipelineBuilder
- [ ] Migrate existing VoicePipeline to new architecture
- [ ] Add integration tests

---

## Phase 4: RAG Enhancement (P1)

### 4.1 RAG Timing Strategies

**Location:** `crates/rag/src/timing.rs`

```rust
// crates/rag/src/timing.rs
#[derive(Debug, Clone, Copy, Deserialize)]
pub enum RAGTimingMode {
    Sequential,
    PrefetchAsync,
    ParallelInject,
}

pub struct SequentialRAG { ... }
pub struct PrefetchRAG { ... }
pub struct ParallelInjectRAG { ... }

pub fn create_rag_strategy(
    mode: RAGTimingMode,
    retriever: Arc<dyn Retriever>,
    llm: Option<Arc<dyn LanguageModel>>,
) -> Arc<dyn RAGStrategy>;
```

### 4.2 Stage-Aware Context Sizing

```rust
// crates/rag/src/context.rs
#[derive(Debug, Clone)]
pub struct ContextBudget {
    pub max_tokens: usize,
    pub doc_limit: usize,
    pub history_turns: usize,
}

pub fn get_context_budget(state: &ConversationState) -> ContextBudget {
    match state {
        ConversationState::Greeting => ContextBudget { max_tokens: 200, doc_limit: 1, history_turns: 0 },
        ConversationState::Discovery => ContextBudget { max_tokens: 800, doc_limit: 3, history_turns: 2 },
        ConversationState::Pitch => ContextBudget { max_tokens: 2000, doc_limit: 5, history_turns: 4 },
        ConversationState::ObjectionHandling { .. } => ContextBudget { max_tokens: 1500, doc_limit: 4, history_turns: 3 },
        ConversationState::Comparison => ContextBudget { max_tokens: 1800, doc_limit: 5, history_turns: 2 },
        ConversationState::Closing => ContextBudget { max_tokens: 500, doc_limit: 2, history_turns: 5 },
        _ => ContextBudget::default(),
    }
}
```

### 4.3 VAD → Prefetch Integration

```rust
// Update crates/agent/src/voice_session.rs
// On VAD speech detection, trigger RAG prefetch

impl VoiceSession {
    async fn on_partial_transcript(&mut self, transcript: &str) {
        if self.config.rag_prefetch_enabled {
            self.rag_strategy.start_prefetch(transcript);
        }
    }
}
```

### 4.4 Checklist

- [ ] Create RAGTimingMode enum
- [ ] Implement SequentialRAG
- [ ] Implement PrefetchRAG with JoinHandle management
- [ ] Implement ParallelInjectRAG
- [ ] Create ContextBudget struct
- [ ] Implement get_context_budget() function
- [ ] Wire VAD events to prefetch trigger
- [ ] Add sufficiency check with LLM option
- [ ] Update HybridRetriever to use Retriever trait
- [ ] Add configuration for timing strategies
- [ ] Add integration tests

---

## Phase 5: Personalization Engine (P2)

### 5.1 Create personalization Crate

**Location:** `crates/personalization/`

**Structure:**
```
crates/personalization/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── segments.rs         # Segment detection logic
    ├── strategy.rs         # Persuasion strategies per segment
    ├── disclosure.rs       # AI identity disclosure timing
    └── guardrails.rs       # Psychology guardrails
```

### 5.2 Segment Detection

```rust
// crates/personalization/src/segments.rs
pub struct SegmentDetector {
    classifiers: HashMap<CustomerSegment, SegmentClassifier>,
}

impl SegmentDetector {
    pub fn detect(&self, profile: &CustomerProfile, context: &ConversationContext) -> CustomerSegment;
    pub fn confidence(&self, segment: CustomerSegment) -> f32;
}
```

### 5.3 Persuasion Strategy

```rust
// crates/personalization/src/strategy.rs
pub struct PersuasionStrategy {
    segment: CustomerSegment,
    key_messages: Vec<String>,
    warmth: f32,
    formality: f32,
    urgency: f32,
}

impl PersuasionStrategy {
    pub fn for_segment(segment: CustomerSegment) -> Self;
    pub fn apply_to_prompt(&self, prompt: &mut PromptBuilder);
    pub fn adjust_response(&self, response: &str) -> String;
}
```

### 5.4 AI Disclosure

```rust
// crates/personalization/src/disclosure.rs
pub enum DisclosureTiming {
    Immediate,
    AfterGreeting,
    WhenAsked,
    NaturalMention,
}

pub struct DisclosureHandler {
    timing: DisclosureTiming,
    disclosed: bool,
    disclosure_text: String,
}

impl DisclosureHandler {
    pub fn should_disclose(&self, turn: usize, context: &ConversationContext) -> bool;
    pub fn get_disclosure(&self) -> &str;
    pub fn weave_into_response(&self, response: &str) -> String;
}
```

### 5.5 Checklist

- [ ] Create `crates/personalization/Cargo.toml`
- [ ] Implement SegmentDetector
- [ ] Implement PersuasionStrategy per segment
- [ ] Wire key_messages() into prompt building
- [ ] Wire suggested_warmth() into PersonaConfig
- [ ] Implement DisclosureHandler
- [ ] Add psychology guardrails
- [ ] Integrate with agent conversation flow
- [ ] Add configuration for strategies
- [ ] Add unit tests

---

## Phase 6: Domain Configuration (P2)

### 6.1 Create domains/ Directory Structure

**Location:** `voice-agent-rust/domains/`

**Structure:**
```
domains/
└── gold_loan/
    ├── knowledge/
    │   ├── products.yaml
    │   ├── competitors.yaml
    │   ├── objections.yaml
    │   └── faq.yaml
    ├── prompts/
    │   ├── system.tera
    │   ├── greeting.tera
    │   ├── pitch.tera
    │   └── objection_handlers.tera
    ├── segments.toml
    ├── tools.toml
    ├── compliance.toml
    └── experiments.toml
```

### 6.2 Domain Loader

```rust
// crates/config/src/domain.rs
pub struct DomainLoader {
    base_path: PathBuf,
}

impl DomainLoader {
    pub fn new(base_path: &Path) -> Self;
    pub fn load_domain(&self, name: &str) -> Result<Domain>;
}

pub struct Domain {
    pub name: String,
    pub knowledge: KnowledgeBase,
    pub prompts: PromptTemplates,
    pub segments: SegmentConfig,
    pub tools: ToolConfig,
    pub compliance: ComplianceRules,
    pub experiments: ExperimentConfig,
}
```

### 6.3 Tera Templates

```rust
// crates/llm/src/prompt_templates.rs
use tera::Tera;

pub struct PromptTemplates {
    tera: Tera,
}

impl PromptTemplates {
    pub fn from_directory(path: &Path) -> Result<Self>;
    pub fn render(&self, name: &str, context: &tera::Context) -> Result<String>;
}
```

### 6.4 Experiment Framework

```rust
// crates/experiments/src/lib.rs (NEW CRATE)
pub struct Experiment {
    pub id: String,
    pub name: String,
    pub variants: Vec<Variant>,
    pub allocation: AllocationStrategy,
}

pub struct ExperimentRunner {
    experiments: HashMap<String, Experiment>,
}

impl ExperimentRunner {
    pub fn assign_variant(&self, experiment_id: &str, session_id: &str) -> Option<&Variant>;
    pub fn track_conversion(&self, experiment_id: &str, session_id: &str);
}
```

### 6.5 Checklist

- [ ] Create domains/gold_loan/ directory structure
- [ ] Create YAML knowledge base files
- [ ] Create Tera prompt templates
- [ ] Create segments.toml
- [ ] Create tools.toml
- [ ] Create compliance.toml
- [ ] Create experiments.toml
- [ ] Implement DomainLoader
- [ ] Implement PromptTemplates with Tera
- [ ] Create experiments crate
- [ ] Implement ExperimentRunner
- [ ] Migrate hardcoded values to config files
- [ ] Add hot-reload capability for configs
- [ ] Add validation for config files
- [ ] Add integration tests

---

## Implementation Order

### Week 1-2: Phase 1 (Core Traits)
- Create traits module
- Define all 9 traits
- Create supporting types
- Update lib.rs exports

### Week 3-5: Phase 2 (Text Processing)
- Create text_processing crate
- Implement grammar correction
- Implement translation (ONNX + gRPC)
- Implement PII detection
- Implement compliance checking

### Week 6-7: Phase 3 (Pipeline)
- Create Frame enum
- Implement processor chain
- Implement sentence streaming
- Implement interrupt handling
- Migrate existing code

### Week 8: Phase 4 (RAG)
- Add timing strategies
- Implement stage-aware context
- Wire VAD → prefetch

### Week 9: Phase 5 (Personalization)
- Create personalization crate
- Implement segment strategies
- Implement AI disclosure

### Week 10-12: Phase 6 (Domain Config)
- Create domain structure
- Implement loaders
- Migrate hardcoded values
- Add experiment framework

---

## Success Criteria

### Per Phase

1. **Phase 1:** All traits compile, existing backends implement them
2. **Phase 2:** Text processing pipeline functional with tests
3. **Phase 3:** Frame-based pipeline processes audio end-to-end
4. **Phase 4:** RAG timing strategies configurable, prefetch working
5. **Phase 5:** Personalization affects agent responses
6. **Phase 6:** Domain switch works without code changes

### Overall

- [ ] All documented traits implemented
- [ ] All documented crates created
- [ ] ARCHITECTURE_v2.md and code match
- [ ] 70%+ test coverage on new code
- [ ] No hardcoded domain logic in Rust
- [ ] Fallback patterns working for STT/TTS
- [ ] Configuration-driven behavior

---

*This roadmap implements ARCHITECTURE_v2.md as documented. See individual phase documents for detailed implementation specifications.*
