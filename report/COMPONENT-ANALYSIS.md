# Component-by-Component Analysis

> Detailed analysis of each crate against architecture documentation

---

## 1. Core Crate (`crates/core`)

### Status: 90% Complete

### Traits Defined vs Expected

| Trait | Expected (docs/interfaces/core-traits.md) | Implemented | Location |
|-------|-------------------------------------------|-------------|----------|
| SpeechToText | YES | YES | `traits/speech.rs:25-61` |
| TextToSpeech | YES | YES | `traits/speech.rs:76-117` |
| VoiceActivityDetector | YES | YES | `traits/speech.rs:262-324` |
| GrammarCorrector | YES | YES | `traits/text_processing.rs:26-54` |
| Translator | YES | YES | `traits/text_processing.rs:75-130` |
| PIIRedactor | YES | YES | `traits/text_processing.rs:148-180` |
| ComplianceChecker | YES | YES | `traits/text_processing.rs:199-226` |
| LanguageModel | YES | YES | `traits/llm.rs:24-85` |
| Retriever | YES | YES | `traits/retriever.rs:23-72` |
| Tool | YES | **MISPLACED** | `tools/src/mcp.rs:315-372` |
| ConversationFSM | YES | **MISSING** | Not found |

### Type Completeness

| Type | Status | Details |
|------|--------|---------|
| Language enum | COMPLETE | 23 languages (22 Indian + English) |
| Script enum | COMPLETE | 13 scripts including Devanagari, Tamil, etc. |
| AudioFrame | COMPLETE | With Rubato resampling (P5 FIX) |
| TranscriptFrame | COMPLETE | Aliased to TranscriptResult |
| VoiceConfig | COMPLETE | Full builder pattern |
| ConversationStage | PARTIAL | 7 states enum, **NO FSM TRAIT** |
| PIIType | COMPLETE | 18 PII types with severity |

### Critical Gap: ConversationFSM

**Required Implementation:**
```rust
// Expected in core/src/traits/fsm.rs
#[async_trait]
pub trait ConversationFSM: Send + Sync + 'static {
    fn state(&self) -> &ConversationState;
    async fn transition(&mut self, event: ConversationEvent) -> Result<Vec<Action>, FSMError>;
    fn can_transition(&self, event: &ConversationEvent) -> bool;
    fn valid_transitions(&self) -> Vec<ConversationEvent>;
    fn checkpoint(&mut self);
    fn restore(&mut self, checkpoint_index: usize) -> Result<(), FSMError>;
    fn context(&self) -> &ConversationContext;
    fn update_context(&mut self, update: ContextUpdate);
}
```

---

## 2. Pipeline Crate (`crates/pipeline`)

### Status: 90% Complete - BETTER THAN DOCUMENTED

### Architecture Verification

| Feature | Documented | Actual | Evidence |
|---------|------------|--------|----------|
| Frame-based | "Monolithic HIGH GAP" | **Frame-based** | `core/traits/pipeline.rs:14-83` |
| Channel communication | Expected | **Implemented** | `pipeline/src/processors/chain.rs:94-201` |
| FrameProcessor trait | Expected | **Implemented** | `core/traits/pipeline.rs:232-271` |
| Sentence streaming | "Detector only" | **Full streaming** | `pipeline/src/processors/sentence_detector.rs` |

### Frame Types Implemented

```rust
// core/traits/pipeline.rs:14-83
pub enum Frame {
    AudioInput(AudioFrame),
    LLMChunk { text: String, is_final: bool },
    Sentence { text: String, language: Language, index: usize },
    AudioOutput(AudioFrame),
    BargeIn { audio_position_ms: u64, transcript: Option<String> },
    // ... 13 frame types total
}
```

### Processors Implemented

| Processor | File | Status |
|-----------|------|--------|
| SentenceDetector | `processors/sentence_detector.rs` | Working |
| TtsProcessor | `processors/tts_processor.rs` | Working |
| InterruptHandler | `processors/interrupt_handler.rs` | Working |
| ProcessorChain | `processors/chain.rs` | Working |

### Indic Sentence Terminators

```rust
// core/src/language.rs
pub fn sentence_terminators(&self) -> &'static [char] {
    match self.script() {
        Script::Devanagari => &['.', '?', '!', '।', '॥'],  // Hindi danda
        Script::Bengali => &['.', '?', '!', '।'],
        Script::Tamil => &['.', '?', '!', '।'],
        // ... all 13 scripts covered
    }
}
```

### Interrupt Modes

```rust
pub enum InterruptMode {
    Immediate,          // Stop immediately on barge-in
    SentenceBoundary,   // Finish current sentence
    WordBoundary,       // Finish current word
    Disabled,           // Ignore barge-in
}
```

---

## 3. LLM Crate (`crates/llm`)

### Status: 75% Complete - CRITICAL TYPE ISSUES

### Architecture Pattern

Uses **Adapter Pattern** but agent doesn't use the adapter:
- `LlmBackend` trait: Backend-specific (Ollama, OpenAI)
- `LanguageModelAdapter`: Adapts to `core::LanguageModel`
- **Problem**: Agent uses `LlmBackend` directly, bypassing adapter

### Type Mismatches (CRITICAL)

| Type | Core Definition | LLM Crate Definition | Conflict |
|------|-----------------|----------------------|----------|
| `ToolDefinition` | `parameters: serde_json::Value` | `parameters: Vec<ToolParameter>` | **INCOMPATIBLE** |
| `Message.name` | `Option<String>` | **MISSING** | Field lost |
| `Message.tool_call_id` | `Option<String>` | **MISSING** | Tool tracking lost |
| `Role::Tool` | Variant exists | Maps to `Role::User` | **Semantic mismatch** |

### Tool Calling Status

```rust
// adapter.rs:138-146 - STUBBED
async fn generate_with_tools(
    &self,
    request: GenerateRequest,
    _tools: &[ToolDefinition],  // IGNORED!
) -> Result<GenerateResponse> {
    // Tools parameter completely ignored
    self.generate(request).await
}
```

### What's Working

- Ollama backend with KV cache: YES
- OpenAI/Azure backend: YES
- Streaming generation: YES
- Speculative execution (4 modes): YES
- Token counting (multilingual): YES

---

## 4. RAG Crate (`crates/rag`)

### Status: 85% Complete

### Retriever Implementation

| Feature | Status | Evidence |
|---------|--------|----------|
| HybridRetriever (Qdrant + BM25) | WORKING | `retriever.rs:1-516` |
| AgenticRetriever (multi-step) | WORKING | `agentic.rs:1-246` |
| RRF Fusion | WORKING | `retriever.rs:257-304` |
| Cross-encoder reranking | WORKING | `reranker.rs:360-478` |

### Query Expansion Status

```
┌────────────────────────────────────────────────────────┐
│                 QUERY EXPANSION WIRING                  │
├────────────────────────────────────────────────────────┤
│                                                        │
│  QueryExpander (query_expansion.rs)                    │
│       │                                                │
│       ├── EnhancedRetriever adapter ──── WIRED ✓      │
│       │       └── adapter.rs:119-124                   │
│       │                                                │
│       └── HybridRetriever.search() ──── NOT WIRED ✗   │
│               └── Called directly by agent             │
│                                                        │
│  Agent uses HybridRetriever directly (agent.rs:899)   │
│  Query expansion is BYPASSED                           │
│                                                        │
└────────────────────────────────────────────────────────┘
```

### Early-Exit Reranking - DEAD CODE

```rust
// reranker.rs:141-200 - Documentation comment
/// # P0 FIX: Early-Exit Limitation with ONNX Runtime
/// **IMPORTANT**: While this struct is named "EarlyExitReranker",
/// layer-by-layer early exit is **NOT currently functional**.
///
/// Why: Standard ONNX models don't expose per-layer outputs

// Line 578: Dead function
#[allow(dead_code)]
fn should_exit(&self, ...) -> bool { ... }
```

**What's Actually Used**: 2-stage cascaded reranking (not layer-by-layer)

### Context Sizing by Stage

```rust
// context.rs:134-192
Greeting:          200 RAG tokens
Discovery:         800 RAG tokens
Qualification:   1,000 RAG tokens
Presentation:    2,000 RAG tokens (MAXIMUM)
ObjectionHandling: 1,500 RAG tokens
Closing:           500 RAG tokens
Farewell:          100 RAG tokens
```

---

## 5. Text Processing Crate (`crates/text_processing`)

### Status: 90% Complete - TRANSLATION WORKS

### Translation Implementation

| Provider | Status | Evidence |
|----------|--------|----------|
| Candle IndicTrans2 | **WORKING** | `translation/candle_indictrans2.rs:1-1141` |
| ONNX IndicTrans2 | WORKING (feature-gated) | `translation/indictrans2.rs:1-488` |
| gRPC Fallback | STUB | Returns unchanged (TODO comment) |
| NoopTranslator | WORKING | Pass-through |

**Key Finding**: Translation is **NOT 100% stubbed** as documented. Candle implementation is functional.

### Component Status

| Component | Implementation | Streaming |
|-----------|---------------|-----------|
| Translation | Candle + ONNX + gRPC stub | YES |
| Grammar Correction | LLM-based | YES |
| PII Redaction | Regex (11 types) + NER | YES |
| Compliance | Rule-based, TOML config | NO |
| TTS Simplifier | Numbers + abbreviations | N/A |

### Indian PII Patterns

```rust
// patterns.rs - All working with confidence scores
Aadhaar:        \b[2-9]\d{3}\s?\d{4}\s?\d{4}\b       (0.95)
PAN:            \b[A-Z]{3}[ABCFGHLJPT][A-Z][0-9]{4}[A-Z]\b  (0.98)
Phone:          (?:\+91[\-\s]?)?(?:0)?[6-9]\d{9}\b   (0.90)
IFSC:           \b[A-Z]{4}0[A-Z0-9]{6}\b             (0.98)
```

---

## 6. Agent Crate (`crates/agent`)

### Status: 80% Complete

### FSM Implementation

**ConversationStage enum**: 7 states implemented
- Greeting, Discovery, Qualification, Presentation
- ObjectionHandling, Closing, Farewell

**Stage transitions**: Working via `StageManager`
**Checkpointing**: History tracked in `stage_history: Vec<StageTransition>`

### Tool Integration

| Tool | Intent Mapping | Status |
|------|----------------|--------|
| check_eligibility | eligibility_check | WORKING |
| calculate_savings | switch_lender | WORKING |
| find_branches | schedule_visit | WORKING |
| capture_lead | capture_lead, interested | WORKING |
| schedule_appointment | book_appointment | WORKING |
| get_gold_price | - | WORKING |
| escalate_to_human | - | WORKING |
| send_sms | - | WORKING |

**Tool Trigger**: Intent-based only (not LLM tool_use)

### Personalization Status

```
┌────────────────────────────────────────────────────────┐
│              PERSONALIZATION WIRING                     │
├────────────────────────────────────────────────────────┤
│                                                        │
│  PersonalizationEngine ──── CREATED ✓                  │
│       │                                                │
│       ├── process_input() ─── CALLED (line 620)        │
│       │       └── Signals detected                     │
│       │                                                │
│       ├── generate_instructions() ── CALLED (line 862)│
│       │       └── Added to prompt IF signals exist     │
│       │                                                │
│       └── CustomerSegment ──── NOT AUTO-DETECTED ✗    │
│               └── Must be manually set                 │
│                                                        │
│  Result: Signals detected but rarely affect behavior   │
│                                                        │
└────────────────────────────────────────────────────────┘
```

### Translator Integration (Translate-Think-Translate)

**WORKING**: Full implementation at `agent.rs:573-671`
1. Detect language from config
2. Translate input → English
3. LLM processes in English
4. Translate output → user language
5. Fallback if translation fails

---

## 7. Tools Crate (`crates/tools`)

### Status: 70% Complete - CONFIG NOT WIRED

### Critical Issue: Hardcoded Config

```rust
// gold_loan.rs:180-182
pub fn new() -> Self {
    Self {
        config: GoldLoanConfig::default(),  // HARDCODED!
    }
}
```

**Impact**: Domain config loaded but tools ignore it

### MCP Protocol Status

| Feature | Status |
|---------|--------|
| Tool trait | IMPLEMENTED |
| JSON Schema validation | IMPLEMENTED |
| Error codes | IMPLEMENTED |
| Input validation | IMPLEMENTED |
| MCP request/response envelope | NOT IMPLEMENTED |
| Resource management | NOT IMPLEMENTED |

---

## 8. Server Crate (`crates/server`)

### Status: 85% Complete

### Working Endpoints

| Method | Endpoint | Status |
|--------|----------|--------|
| POST | /api/sessions | WORKING |
| GET | /ws/:session_id | WORKING (audio) |
| POST | /api/webrtc/:id/offer | Signaling only |
| GET | /health | WORKING |
| GET | /metrics | WORKING |

### Audio Flow

```
WebSocket → PCM decode → AudioFrame → VoicePipeline
    → VAD → STT → Transcript → Agent → Response
```

### WebRTC Gap

```
WebRTC Signaling ──── WORKING
WebRTC Audio ──────── NOT CONNECTED to pipeline
Transport crate ───── NOT USED by server
```

---

## 9. Transport Crate (`crates/transport`)

### Status: 40% Integrated

**Fully Implemented**:
- WebRTC peer connection
- Opus codec (encode/decode)
- Resampling (rubato)
- Transport traits

**Not Integrated**:
- Server uses own WebSocket implementation
- WebRTC audio not routed to pipeline
- Transport abstraction not used

---

## 10. Config Crate (`crates/config`)

### Status: 95% Complete

**All Working**:
- Domain config loading (YAML/JSON)
- Hot-reload endpoint
- Validation (comprehensive)
- DomainConfigManager singleton

**All Structures**:
- GoldLoanConfig (tiered rates, LTV, fees)
- ProductConfig (4 variants)
- BranchConfig (1600 branches)
- CompetitorConfig (6 competitors)
- PromptTemplates (stage-specific)
