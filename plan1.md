# Voice Sales Agent Architecture Redesign Plan

## Executive Summary

Redesign the voice agent backend from scratch with focus on:
- **Domain-agnostic design** (pluggable for any product/vertical)
- **Event-driven, async-first architecture**
- **Low latency** (sub-800ms target)
- **Agentic RAG workflow engine**
- **Smart context management and personalization**

---

## Analysis of Previous Implementation

### What We Have (Surviving Code)
| Directory | Status | Quality |
|-----------|--------|---------|
| `config/` | Complete | Good settings/features.yaml pattern |
| `plugins/` | Complete | Good STT/TTS/LLM/Translation plugins |
| `rag/` | Complete | Solid hybrid retriever with enhanced features |
| `personalization/` | Complete | Good customer profile model |
| `orchestration/` | Partial | Missing tools.executor dependency |
| `tests/` | Complete | Test stubs available |

### What Was Lost
- `main.py` - FastAPI entry point
- `core/` - Interfaces, registry, pipeline
- `conversation/` - State machine, prompts
- `tools/` - Tool executor, calculators
- `data/knowledge/` - YAML knowledge base

### Previous Implementation Strengths
1. **Plugin Registry Pattern** - Easy provider swapping
2. **Hybrid RAG** - Semantic + BM25 with reranking
3. **Stage-aware retrieval** - Context sizing by conversation stage
4. **Multi-language** - 7+ Indian languages with IndicConformer/IndicF5
5. **Customer segmentation** - P1-P4 segments with tailored messaging

### Previous Implementation Weaknesses
1. **Domain-coupled** - Hard-coded gold loan logic throughout
2. **Synchronous processing** - No async pipeline parallelization
3. **No event-driven architecture** - Sequential processing
4. **Tight prompt coupling** - Prompts embedded in orchestrator
5. **Basic state machine** - Not a proper FSM
6. **Limited tool execution** - No parallel tool calls
7. **No semantic caching** - Repeated LLM calls for similar queries
8. **Missing interrupt handling** - No mid-utterance detection

---

## Research Findings Summary

### Latency Best Practices (Sources: [1][2][3])
- **Target**: Sub-800ms end-to-end latency
- **WebRTC** over telephony (saves ~300ms)
- **4-bit quantization** achieves 40% latency reduction
- **Semantic caching** for repeated/similar queries
- **Persistent gRPC/WebSocket connections**
- **Parallel processing** of pipeline stages

### Framework Recommendations (Sources: [4][5][6])
- **LangGraph** for production graph-based agent flows (recommended)
- **DSPy** for optimized prompt pipelines
- **Pipecat** for voice-specific pipeline architecture

### Rust vs Python Decision (Sources: [7][8])
| Aspect | Python | Rust |
|--------|--------|------|
| Ecosystem | 80% of AI agents | Emerging (Kowalski, llm-chain) |
| Latency | GIL limits parallelism | True parallelism, no GIL |
| Development | Faster iteration | Slower, but safer |
| LLM libraries | Mature | Experimental |

### Detailed Rust Framework Analysis

| Framework | Maturity | Voice Support | Agent Capabilities | Production Ready |
|-----------|----------|---------------|-------------------|------------------|
| **ADK-Rust** | New (2025) | Yes (Realtime API) | Full agent support | Early |
| **Rig** | Growing | No (LLM focus) | Agentic pipelines | Yes (used in prod) |
| **Kowalski** | v0.5 | No | Multi-agent orchestration | Early |
| **llm-chain** | Stable | No | Prompt chaining, RAG | Moderate |
| **Candle** | Stable | No | ML inference only | Yes |

**Key Insights:**
- **ADK-Rust** is the only Rust framework with built-in voice agent support (realtime audio streaming)
- **Rig** has the best production track record and growing community
- The Rust ecosystem is ~2 years behind Python for agentic AI
- Most Rust frameworks focus on LLM/agent logic, not full voice pipelines

**Recommendation Options:**

1. **Pure Rust (ADK-Rust based)**: Bleeding edge, full control, best latency potential. Risk: immature ecosystem, limited Indian language support (IndicConformer/IndicF5 are Python)

2. **Hybrid (Rust core + Python plugins)**: Rust for orchestration/pipeline, Python for STT/TTS/RAG. Best of both worlds but complex FFI.

3. **Python with Rust hot paths**: Python for rapid development, Rust for latency-critical components (audio processing, vector search). Pragmatic approach.

### Event-Driven Architecture (Sources: [9][10])
- **Event-driven FSM** for conversation flow (idle → listening → generating → emitting)
- **Priority scheduling** for interrupts
- **Async queues** between pipeline stages
- **Parallel agent execution** for tool calls

### Sales Conversation Psychology (Sources: [11][12])
- **Persona-aware persuasion** adapts messaging to customer profile
- **Context-aware strategy** selection (credibility appeal, etc.)
- **Prosody alignment** increases perceived personalization
- **Empathic responses** increase trust and task delegation

---

## Proposed Architecture: Pure Rust with ADK-Rust + sherpa-rs

### Technology Stack Decision

Based on user preference and research, we will build a **Pure Rust** architecture using:

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Agent Framework** | [ADK-Rust](https://github.com/zavora-ai/adk-rust) | LLM orchestration, tools, graph workflows |
| **STT/TTS Engine** | [sherpa-rs](https://github.com/thewh1teagle/sherpa-rs) | Rust bindings to sherpa-onnx |
| **Indian Languages** | ONNX-exported IndicConformer/IndicF5 | Via NeMo export → sherpa-onnx |
| **Vector Store** | [qdrant](https://github.com/qdrant/qdrant) | Rust-native vector search |
| **Async Runtime** | Tokio | High-performance async I/O |
| **API Framework** | Axum | Fast, ergonomic web framework |
| **Config** | TOML/YAML + serde | Type-safe configuration |

### Design Principles

1. **Trait-Driven Design**
   - All components implement Rust traits (interfaces)
   - Domain logic injected via configuration, not code

2. **Async-First, Event-Driven**
   - Pipeline stages communicate via async channels (tokio::mpsc)
   - Parallel processing via tokio::spawn

3. **Zero-Cost Abstractions**
   - Pluggable components without runtime overhead
   - Compile-time guarantees for correctness

4. **Domain-Agnostic Core**
   - No product-specific code in core crates
   - Domain knowledge via RAG + prompt templates + tool configs

5. **Conversation as State Machine**
   - Each call is a state machine with checkpointing
   - Events trigger transitions, supports interrupts

---

## High-Level Crate Structure

```
voice-agent/
├── Cargo.toml               # Workspace definition
├── Cargo.lock
│
├── crates/
│   ├── core/                # Core traits and types
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── traits/      # All component traits
│   │   │   │   ├── mod.rs
│   │   │   │   ├── stt.rs         # SpeechToText trait
│   │   │   │   ├── tts.rs         # TextToSpeech trait
│   │   │   │   ├── llm.rs         # LanguageModel trait
│   │   │   │   ├── retriever.rs   # Retriever trait
│   │   │   │   ├── tool.rs        # Tool trait
│   │   │   │   └── persona.rs     # Persona trait
│   │   │   ├── types/       # Core types
│   │   │   │   ├── mod.rs
│   │   │   │   ├── audio.rs       # AudioFrame, AudioConfig
│   │   │   │   ├── message.rs     # Message, Turn, History
│   │   │   │   ├── context.rs     # ConversationContext
│   │   │   │   └── events.rs      # Event types for pipeline
│   │   │   └── error.rs     # Error types
│   │   └── Cargo.toml
│   │
│   ├── pipeline/            # Event-driven audio pipeline
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── processor.rs      # FrameProcessor trait
│   │   │   ├── pipeline.rs       # Pipeline orchestrator
│   │   │   ├── vad.rs            # Voice Activity Detection
│   │   │   ├── turn.rs           # Turn detection
│   │   │   └── interrupt.rs      # Barge-in handling
│   │   └── Cargo.toml
│   │
│   ├── speech/              # STT/TTS implementations
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── stt/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── sherpa.rs     # sherpa-rs STT
│   │   │   │   └── whisper.rs    # Whisper ONNX
│   │   │   ├── tts/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── sherpa.rs     # sherpa-rs TTS
│   │   │   │   └── piper.rs      # Piper ONNX
│   │   │   └── models/
│   │   │       ├── mod.rs
│   │   │       └── indic.rs      # IndicConformer/IndicF5 loaders
│   │   └── Cargo.toml
│   │
│   ├── agent/               # ADK-Rust based agent system
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── sales_agent.rs    # Main sales agent
│   │   │   ├── workflows/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── conversation.rs  # Conversation graph
│   │   │   │   ├── objection.rs     # Objection handling flow
│   │   │   │   └── closing.rs       # Closing flow
│   │   │   ├── tools/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── calculator.rs    # Savings calculator
│   │   │   │   ├── eligibility.rs   # Eligibility checker
│   │   │   │   └── appointment.rs   # Appointment booking
│   │   │   └── prompts/
│   │   │       ├── mod.rs
│   │   │       └── builder.rs       # Dynamic prompt builder
│   │   └── Cargo.toml
│   │
│   ├── rag/                 # Agentic RAG system
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── retriever.rs      # Hybrid retriever
│   │   │   ├── embeddings.rs     # Embedding generation
│   │   │   ├── reranker.rs       # Cross-encoder reranking
│   │   │   ├── agentic.rs        # Multi-step agentic RAG
│   │   │   └── store/
│   │   │       ├── mod.rs
│   │   │       ├── qdrant.rs     # Qdrant vector store
│   │   │       └── bm25.rs       # BM25 sparse search
│   │   └── Cargo.toml
│   │
│   ├── personalization/     # Customer personalization
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── profile.rs        # Customer profile
│   │   │   ├── segment.rs        # Segment classifier
│   │   │   ├── strategy.rs       # Persuasion strategy
│   │   │   └── context.rs        # Context management
│   │   └── Cargo.toml
│   │
│   ├── llm/                 # LLM provider integrations
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── providers/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── ollama.rs     # Local Ollama
│   │   │   │   ├── anthropic.rs  # Claude API
│   │   │   │   ├── openai.rs     # OpenAI API
│   │   │   │   └── gemini.rs     # Google Gemini
│   │   │   ├── router.rs         # Model routing logic
│   │   │   └── cache.rs          # Semantic caching
│   │   └── Cargo.toml
│   │
│   ├── config/              # Configuration management
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── settings.rs       # Global settings
│   │   │   ├── features.rs       # Feature flags
│   │   │   └── domain.rs         # Domain config loader
│   │   └── Cargo.toml
│   │
│   └── server/              # API server
│       ├── src/
│       │   ├── main.rs
│       │   ├── routes/
│       │   │   ├── mod.rs
│       │   │   ├── websocket.rs  # WebSocket handlers
│       │   │   ├── rest.rs       # REST endpoints
│       │   │   └── health.rs     # Health checks
│       │   └── middleware/
│       │       ├── mod.rs
│       │       ├── auth.rs       # Authentication
│       │       └── metrics.rs    # Observability
│       └── Cargo.toml
│
├── domains/                 # Domain-specific configs (NOT code)
│   └── gold_loan/
│       ├── knowledge/       # YAML knowledge base
│       │   ├── products.yaml
│       │   ├── competitors.yaml
│       │   ├── faqs.yaml
│       │   └── objections.yaml
│       ├── prompts/         # Domain prompts (Tera templates)
│       │   ├── system.tera
│       │   ├── segments/
│       │   │   ├── high_value.tera
│       │   │   ├── trust_seeker.tera
│       │   │   └── ...
│       │   └── objections/
│       │       ├── safety.tera
│       │       ├── rate.tera
│       │       └── ...
│       ├── tools.toml       # Domain tools config
│       ├── segments.toml    # Customer segments config
│       └── experiments.toml # A/B test configs
│
├── models/                  # ONNX models (git-lfs or download)
│   ├── stt/
│   │   └── indicconformer/  # Exported from NeMo
│   ├── tts/
│   │   └── indicf5/         # Exported to ONNX
│   └── embeddings/
│       └── multilingual/    # E5 or similar
│
└── tests/                   # Integration tests
    ├── e2e/
    └── benchmarks/
```

---

## Key Architectural Decisions

### 1. Event-Driven Pipeline (Rust)

```rust
// Events/Frames flow through tokio channels
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum Frame {
    Audio(AudioFrame),
    Transcript(TranscriptFrame),
    LLMResponse(LLMResponseFrame),
    Speech(SpeechFrame),
    Control(ControlFrame),
}

#[derive(Debug, Clone)]
pub struct AudioFrame {
    pub data: Vec<i16>,
    pub sample_rate: u32,
    pub channels: u8,
}

#[derive(Debug, Clone)]
pub struct TranscriptFrame {
    pub text: String,
    pub language: String,
    pub is_final: bool,
    pub confidence: f32,
}

// Processors are async trait implementations
#[async_trait]
pub trait FrameProcessor: Send + Sync {
    async fn process(&self, frame: Frame, tx: mpsc::Sender<Frame>) -> Result<()>;
}

// Pipeline connects processors via channels
pub struct Pipeline {
    processors: Vec<Box<dyn FrameProcessor>>,
    channels: Vec<(mpsc::Sender<Frame>, mpsc::Receiver<Frame>)>,
}

impl Pipeline {
    pub async fn run(&mut self) -> Result<()> {
        // Spawn each processor as a tokio task
        for (processor, (tx, mut rx)) in self.processors.iter().zip(&mut self.channels) {
            let tx = tx.clone();
            tokio::spawn(async move {
                while let Some(frame) = rx.recv().await {
                    processor.process(frame, tx.clone()).await?;
                }
                Ok::<_, Error>(())
            });
        }
        Ok(())
    }
}
```

### 2. Conversation State Machine (Rust enum-based FSM)

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ConversationState {
    Idle,
    Greeting,
    Listening,
    Thinking,
    Responding,
    Interrupted,
    // End states
    Converted { appointment_time: Option<String> },
    FollowUp { reason: String },
    Declined { reason: String },
    Escalated { to: String },
}

#[derive(Debug, Clone)]
pub enum ConversationEvent {
    CallStarted { customer_id: String },
    UserSpeaking,
    UserSilence { duration_ms: u64 },
    TranscriptReady { text: String },
    ResponseGenerated { text: String },
    BargeIn,
    ToolResult { tool: String, result: Value },
    UserIntent { intent: Intent },
    Timeout { stage: String },
}

pub struct ConversationFSM {
    state: ConversationState,
    context: ConversationContext,
    checkpoints: Vec<(ConversationState, ConversationContext)>,
}

impl ConversationFSM {
    pub fn transition(&mut self, event: ConversationEvent) -> Result<Vec<Action>> {
        let (new_state, actions) = match (&self.state, event) {
            (ConversationState::Idle, ConversationEvent::CallStarted { customer_id }) => {
                (ConversationState::Greeting, vec![Action::LoadCustomerProfile(customer_id)])
            }
            (ConversationState::Responding, ConversationEvent::BargeIn) => {
                (ConversationState::Interrupted, vec![Action::StopTTS, Action::StartListening])
            }
            // ... more transitions
            _ => (self.state.clone(), vec![]),
        };

        self.checkpoint();
        self.state = new_state;
        Ok(actions)
    }

    fn checkpoint(&mut self) {
        self.checkpoints.push((self.state.clone(), self.context.clone()));
    }
}
```

### 3. Agentic RAG Workflow (Rust with ADK-Rust patterns)

```rust
use adk_agent::Agent;
use adk_tool::Tool;

pub struct AgenticRAG {
    retriever: Box<dyn Retriever>,
    reranker: Box<dyn Reranker>,
    llm: Box<dyn LanguageModel>,
}

impl AgenticRAG {
    pub async fn retrieve(
        &self,
        query: &str,
        context: &ConversationContext,
        max_iterations: usize,
    ) -> Result<Vec<Document>> {
        let mut current_query = query.to_string();
        let mut all_docs = Vec::new();

        for iteration in 0..max_iterations {
            // Step 1: Classify intent (fast model)
            let intent = self.llm.classify_intent(&current_query).await?;

            // Step 2: Retrieve docs based on intent
            let docs = self.retriever
                .retrieve(&current_query, intent.doc_types())
                .await?;

            all_docs.extend(docs.clone());

            // Step 3: Check sufficiency
            let is_sufficient = self.llm
                .check_sufficiency(&current_query, &docs)
                .await?;

            if is_sufficient {
                break;
            }

            // Step 4: Rewrite query if more needed
            current_query = self.llm
                .rewrite_query(&current_query, &docs)
                .await?;
        }

        // Step 5: Rerank all collected docs
        let ranked = self.reranker.rerank(query, all_docs).await?;

        Ok(ranked)
    }
}
```

### 4. Domain Configuration (TOML-based, type-safe)

```toml
# domains/gold_loan/segments.toml
[segments.high_value]
display_name = "High-Value MSME"
priority = 1

[segments.high_value.indicators]
keywords = ["lakh", "crore", "business", "msme", "vyapar"]
loan_range = [500_000, 25_00_000]
age_range = [35, 55]

[segments.high_value.messaging]
focus = ["savings", "business_growth", "relationship_banking"]
tone = "professional"
pace = "measured"

[segments.high_value.persuasion]
primary = "credibility_appeal"
secondary = ["savings_calculation", "competitor_comparison"]
avoid = ["urgency_tactics"]

[segments.high_value.prompts]
system = "segments/high_value.tera"
objection_handlers = { safety = "objections/safety_high_value.tera" }
```

```rust
// Type-safe config loading with serde
#[derive(Debug, Deserialize)]
pub struct SegmentConfig {
    pub display_name: String,
    pub priority: u8,
    pub indicators: SegmentIndicators,
    pub messaging: MessagingConfig,
    pub persuasion: PersuasionConfig,
    pub prompts: PromptPaths,
}

#[derive(Debug, Deserialize)]
pub struct SegmentIndicators {
    pub keywords: Vec<String>,
    pub loan_range: (u64, u64),
    pub age_range: Option<(u8, u8)>,
}
```

### 5. Parallel Tool Execution (Tokio-based)

```rust
use tokio::task::JoinSet;

pub struct ToolExecutor {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolExecutor {
    pub async fn execute_parallel(
        &self,
        tool_calls: Vec<ToolCall>,
    ) -> Vec<ToolResult> {
        let mut join_set = JoinSet::new();

        for call in tool_calls {
            if let Some(tool) = self.tools.get(&call.name) {
                let tool = tool.clone();
                let params = call.parameters.clone();

                join_set.spawn(async move {
                    let result = tool.execute(params).await;
                    ToolResult {
                        tool_name: call.name,
                        success: result.is_ok(),
                        output: result.unwrap_or_else(|e| e.to_string()),
                    }
                });
            }
        }

        let mut results = Vec::new();
        while let Some(res) = join_set.join_next().await {
            if let Ok(result) = res {
                results.push(result);
            }
        }

        results
    }
}
```

### 6. Indian Language Support via ONNX

```rust
// Strategy for IndicConformer/IndicF5 in Rust
use sherpa_rs::{OnlineRecognizer, OfflineTts};

pub struct IndicSpeechProvider {
    stt: OnlineRecognizer,  // Streaming ASR
    tts: OfflineTts,        // TTS synthesis
}

impl IndicSpeechProvider {
    pub fn new(config: &SpeechConfig) -> Result<Self> {
        // Load ONNX-exported IndicConformer model
        let stt = OnlineRecognizer::from_transducer(
            &config.stt_model_path,
            &config.stt_tokens_path,
            config.sample_rate,
        )?;

        // Load ONNX-exported IndicF5/Piper model
        let tts = OfflineTts::from_vits(
            &config.tts_model_path,
            &config.tts_lexicon_path,
        )?;

        Ok(Self { stt, tts })
    }
}

// ONNX Export Pipeline (Python script to run once):
// 1. Load NeMo IndicConformer model
// 2. model.export('indicconformer.onnx', cache_support=True)
// 3. Convert to sherpa-onnx compatible format
// 4. Load in Rust via sherpa-rs
```

---

## Decisions Made

| Question | Decision |
|----------|----------|
| **Language** | Pure Rust with ADK-Rust |
| **Indian Language Support** | ONNX export from NeMo, load via sherpa-rs |
| **Domain Scope** | Industry-agnostic (config-driven) |
| **Deployment** | Provider-agnostic with fallback support |
| **Priority** | Production-ready, quality + latency focused |
| **Approach** | Design docs first, implement component-by-component |

---

## Implementation Phases

### Phase 0: ONNX Model Export (Pre-requisite)
**Goal:** Export Indian language models to ONNX format for Rust consumption

**Tasks:**
1. Export IndicConformer to ONNX via NeMo
2. Export IndicF5/alternative TTS to ONNX
3. Validate models work with sherpa-onnx Python bindings
4. Test sherpa-rs loading of exported models

**Output:** Working ONNX models in `models/` directory

---

### Phase 1: Core Crate - Traits & Types
**Goal:** Define all interfaces and core types

**Files:**
- `crates/core/src/traits/*.rs` - All component traits
- `crates/core/src/types/*.rs` - Core data types
- `crates/core/src/error.rs` - Error handling

**Key Traits:**
```rust
pub trait SpeechToText: Send + Sync {
    async fn transcribe(&self, audio: AudioFrame) -> Result<TranscriptFrame>;
    fn supported_languages(&self) -> Vec<Language>;
}

pub trait TextToSpeech: Send + Sync {
    async fn synthesize(&self, text: &str, voice: &VoiceConfig) -> Result<AudioFrame>;
    fn available_voices(&self) -> Vec<VoiceInfo>;
}

pub trait LanguageModel: Send + Sync {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse>;
    async fn stream(&self, request: GenerateRequest) -> Result<impl Stream<Item = StreamChunk>>;
}

pub trait Retriever: Send + Sync {
    async fn retrieve(&self, query: &str, options: RetrieveOptions) -> Result<Vec<Document>>;
}

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;
    async fn execute(&self, params: Value) -> Result<Value>;
}
```

---

### Phase 2: Config Crate - Domain Configuration
**Goal:** Type-safe configuration loading

**Files:**
- `crates/config/src/settings.rs` - Global settings
- `crates/config/src/domain.rs` - Domain config loader
- `domains/gold_loan/*.toml` - Example domain configs

**Deliverables:**
- TOML schema for segments, prompts, tools
- Hot-reloadable config
- Validation on load

---

### Phase 3: Speech Crate - STT/TTS
**Goal:** sherpa-rs based speech processing

**Files:**
- `crates/speech/src/stt/sherpa.rs`
- `crates/speech/src/tts/sherpa.rs`
- `crates/speech/src/models/indic.rs`

**Dependencies:**
- sherpa-rs (ONNX inference)
- voice-stream (VAD)

---

### Phase 4: Pipeline Crate - Event Processing
**Goal:** Async event-driven audio pipeline

**Files:**
- `crates/pipeline/src/pipeline.rs` - Pipeline orchestrator
- `crates/pipeline/src/processor.rs` - Processor trait
- `crates/pipeline/src/vad.rs` - Voice activity detection
- `crates/pipeline/src/turn.rs` - Turn detection

**Architecture:**
```
Audio → VAD → STT → Agent → TTS → Audio
         ↑                    ↓
         └── Interrupt ←──────┘
```

---

### Phase 5: LLM Crate - Provider Integration
**Goal:** Multi-provider LLM support with routing

**Files:**
- `crates/llm/src/providers/ollama.rs`
- `crates/llm/src/providers/anthropic.rs`
- `crates/llm/src/providers/openai.rs`
- `crates/llm/src/router.rs` - Model routing
- `crates/llm/src/cache.rs` - Semantic caching

**Features:**
- Streaming responses
- Function calling
- Automatic fallback
- Request caching

---

### Phase 6: RAG Crate - Knowledge Retrieval
**Goal:** Agentic hybrid retrieval

**Files:**
- `crates/rag/src/retriever.rs` - Hybrid retriever
- `crates/rag/src/store/qdrant.rs` - Vector store
- `crates/rag/src/store/bm25.rs` - Sparse search
- `crates/rag/src/agentic.rs` - Multi-step RAG

**Features:**
- Query rewriting
- Multi-step retrieval
- Cross-encoder reranking
- Stage-aware context sizing

---

### Phase 7: Agent Crate - Conversation Logic
**Goal:** ADK-Rust based sales agent

**Files:**
- `crates/agent/src/sales_agent.rs`
- `crates/agent/src/workflows/*.rs`
- `crates/agent/src/tools/*.rs`
- `crates/agent/src/prompts/builder.rs`

**Workflows:**
- Greeting → Qualification → Pitch → Objection → Closing
- Interrupt handling
- Graceful end states

---

### Phase 8: Personalization Crate
**Goal:** Customer profiling and strategy

**Files:**
- `crates/personalization/src/profile.rs`
- `crates/personalization/src/segment.rs`
- `crates/personalization/src/strategy.rs`

**Features:**
- Segment classification
- Persuasion strategy selection
- Context-aware personalization
- Smart disclosure (don't reveal known info directly)

---

### Phase 9: Server Crate - API Layer
**Goal:** Axum-based WebSocket server

**Files:**
- `crates/server/src/main.rs`
- `crates/server/src/routes/websocket.rs`
- `crates/server/src/routes/rest.rs`

**Endpoints:**
- `ws://.../conversation/{id}` - Voice conversation
- `GET /api/customers` - Customer profiles
- `GET /api/health` - Health check
- `GET /api/metrics` - Prometheus metrics

---

### Phase 10: Integration & Testing
**Goal:** End-to-end validation

**Tasks:**
- Unit tests for each crate
- Integration tests for pipeline
- Latency benchmarks
- Load testing

---

## Documentation Deliverables

Before implementation, we will create detailed docs for:

1. **Component Interface Specification** (`docs/interfaces.md`)
   - All trait definitions with examples
   - Error handling patterns
   - Async patterns

2. **Domain Configuration Schema** (`docs/domain-schema.md`)
   - TOML schema for all configs
   - Validation rules
   - Examples for different industries

3. **Pipeline Flow Diagrams** (`docs/pipeline.md`)
   - Event flow diagrams
   - State machine diagrams
   - Interrupt handling

4. **Prompt Engineering Guide** (`docs/prompts.md`)
   - Tera template patterns
   - Segment-specific messaging
   - Objection handling strategies

5. **RAG Strategy Document** (`docs/rag.md`)
   - Retrieval strategies
   - Query rewriting patterns
   - Context sizing rules

6. **Latency Optimization Guide** (`docs/latency.md`)
   - Target latencies per stage
   - Optimization techniques
   - Profiling methodology

---

## Risk Assessment & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| **ONNX export fails for IndicConformer** | High | Medium | Fallback to Python microservice via gRPC |
| **sherpa-rs bindings incomplete** | Medium | Low | Contribute fixes upstream or use FFI directly |
| **ADK-Rust immature for production** | Medium | Medium | Fork and maintain; use proven patterns from Python |
| **Latency targets not met** | High | Low | Profile early, optimize hot paths, consider hybrid |
| **Rust learning curve slows team** | Medium | High | Pair programming, incremental adoption, good docs |

---

## Success Criteria

1. **Latency**: End-to-end < 800ms (STT: 200ms, LLM: 400ms, TTS: 200ms)
2. **Quality**: Conversation success rate comparable to Python baseline
3. **Extensibility**: New domain deployable via config only (no code changes)
4. **Reliability**: Zero panics in production, graceful error handling
5. **Observability**: Full tracing, metrics, and logging

---

## Summary

This plan proposes a **Pure Rust architecture** for a production-grade, industry-agnostic voice sales agent. Key innovations:

1. **ADK-Rust + sherpa-rs** for native performance without Python GIL
2. **ONNX-exported Indian language models** for multilingual support
3. **Event-driven pipeline** with Tokio for true parallelism
4. **Config-driven domains** for easy vertical expansion
5. **Agentic RAG** with multi-step retrieval
6. **Type-safe everything** with Rust's compile-time guarantees

The implementation will proceed **documentation-first**, with detailed specs for each component before coding begins.

---

## Sources

### Voice Agent Architecture
1. [Cresta - Engineering for Real-Time Voice Agent Latency](https://cresta.com/blog/engineering-for-real-time-voice-agent-latency)
2. [Nikhil R - How to Optimise Latency for Voice Agents](https://rnikhil.com/2025/05/18/how-to-reduce-latency-voice-agents)
3. [Vonage - Reducing RAG Pipeline Latency](https://developer.vonage.com/en/blog/reducing-rag-pipeline-latency-for-real-time-voice-conversations)
4. [Deepgram - 4 Key Considerations for Voice AI Agents](https://deepgram.com/learn/considerations-for-building-ai-agents)
5. [AWS - Building Voice Agents with Pipecat](https://aws.amazon.com/blogs/machine-learning/building-intelligent-ai-voice-agents-with-pipecat-and-amazon-bedrock-part-1/)

### AI Agent Frameworks
6. [LangWatch - Best AI Agent Frameworks 2025](https://langwatch.ai/blog/best-ai-agent-frameworks-in-2025-comparing-langgraph-dspy-crewai-agno-and-more)
7. [GitHub - Pipecat Framework](https://github.com/pipecat-ai/pipecat)
8. [Weaviate - What is Agentic RAG](https://weaviate.io/blog/what-is-agentic-rag)

### Rust for AI
9. [Red Hat - Why Agentic AI Developers Moving to Rust](https://developers.redhat.com/articles/2025/09/15/why-some-agentic-ai-developers-are-moving-code-python-rust)
10. [Vision on Edge - Rise of Rust in Agentic AI](https://visiononedge.com/rise-of-rust-in-agentic-ai-systems/)
11. [GitHub - ADK-Rust](https://github.com/zavora-ai/adk-rust)
12. [Rig.rs - Rust LLM Framework](https://rig.rs/)
13. [DEV - Kowalski Rust-native Agentic AI](https://dev.to/yarenty/kowalski-the-rust-native-agentic-ai-framework-53k4)

### Speech Processing
14. [GitHub - sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx)
15. [GitHub - sherpa-rs (Rust bindings)](https://github.com/thewh1teagle/sherpa-rs)
16. [NVIDIA NeMo - Exporting Models](https://docs.nvidia.com/nemo-framework/user-guide/24.09/nemotoolkit/core/export.html)
17. [GitHub - AI4Bharat IndicConformerASR](https://github.com/AI4Bharat/IndicConformerASR)

### Event-Driven Architecture
18. [Gcore - Event-Driven AI Architectures](https://gcore.com/blog/event-driven-ai-architectures)
19. [arXiv - Asynchronous Tool Usage for Real-Time Agents](https://arxiv.org/html/2410.21620v1)
20. [GitHub - LlamaIndex Workflows](https://github.com/run-llama/workflows-py)

### Sales Conversation Psychology
21. [ScienceDirect - Persona Aware Persuasive Dialogue](https://www.sciencedirect.com/science/article/abs/pii/S0957417421016067)
22. [Taylor & Francis - Voice Agent Persuasiveness](https://www.tandfonline.com/doi/full/10.1080/0144929X.2024.2420871)
23. [PMC - Reinforcing Personalized Persuasion in Virtual Sales](https://pmc.ncbi.nlm.nih.gov/articles/PMC9815581/)
