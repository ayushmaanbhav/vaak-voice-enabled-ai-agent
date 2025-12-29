# Implementation Fix Plan

> Prioritized fixes for voice agent codebase
> Organized by severity and implementation order

---

## Priority Definitions

| Priority | Definition | Timeline |
|----------|------------|----------|
| **P0** | Blocks production deployment | Immediate |
| **P1** | Required for full feature set | 1-2 weeks |
| **P2** | Production hardening | 2-4 weeks |
| **P3** | Nice to have | Backlog |

---

## P0 - Critical Fixes (Production Blockers)

### P0-1: Define ConversationFSM Trait in Core Crate

**Issue:** ConversationFSM trait is documented but not implemented. Only enum exists.

**Location:** `crates/core/src/traits/`

**Implementation:**

```rust
// NEW FILE: crates/core/src/traits/fsm.rs

use async_trait::async_trait;
use crate::conversation::{ConversationStage, ConversationContext};
use std::time::Duration;

/// Conversation events that trigger state transitions
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
    UserObjection { objection_type: ObjectionType },

    // Interrupts
    BargeIn,
    Timeout { stage: String },

    // Tools
    ToolCallRequested { tool: String },
    ToolResultReady { tool: String, result: serde_json::Value },

    // Errors
    Error { error: String },
}

/// Actions to take after state transition
#[derive(Debug, Clone)]
pub enum Action {
    StartListening,
    StopListening,
    StartSpeaking { text: String },
    StopSpeaking,
    LoadCustomerProfile { customer_id: String },
    UpdateContext { key: String, value: serde_json::Value },
    ExecuteTool { name: String, params: serde_json::Value },
    Checkpoint,
    EndConversation { outcome: ConversationOutcome },
    Escalate { to: String, reason: String },
    RecordMetric { name: String, value: f64 },
}

/// FSM Errors
#[derive(Debug, thiserror::Error)]
pub enum FSMError {
    #[error("Invalid transition from {0:?} with event {1:?}")]
    InvalidTransition(ConversationStage, String),

    #[error("No checkpoint at index {0}")]
    NoCheckpoint(usize),

    #[error("State machine corrupted")]
    Corrupted,
}

/// Finite State Machine for conversation flow
#[async_trait]
pub trait ConversationFSM: Send + Sync + 'static {
    /// Get current state
    fn state(&self) -> &ConversationStage;

    /// Process event and transition state
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
    fn update_context(&mut self, key: &str, value: serde_json::Value);
}
```

**Files to Update:**
1. Create `crates/core/src/traits/fsm.rs`
2. Add to `crates/core/src/traits/mod.rs`: `pub mod fsm;`
3. Add to `crates/core/src/lib.rs` exports

**Estimate:** 4-6 hours

---

### P0-2: Fix ToolDefinition Type Mismatch

**Issue:** Core and LLM crates define different `ToolDefinition` types with incompatible parameter schemas.

**Locations:**
- `crates/core/src/llm_types.rs:269-276`
- `crates/llm/src/prompt.rs:46-53`

**Fix:**

```rust
// REMOVE from crates/llm/src/prompt.rs:46-53
// pub struct ToolDefinition { ... }

// REPLACE WITH:
pub use voice_agent_core::llm_types::ToolDefinition;

// UPDATE any code that creates ToolDefinition to use JSON schema:
let tool_def = ToolDefinition {
    name: "check_eligibility".to_string(),
    description: "Check loan eligibility".to_string(),
    parameters: serde_json::json!({
        "type": "object",
        "properties": {
            "gold_weight_grams": {"type": "number"},
            "gold_purity": {"type": "string", "enum": ["24K", "22K", "18K", "14K"]}
        },
        "required": ["gold_weight_grams"]
    }),
};
```

**Files to Update:**
1. `crates/llm/src/prompt.rs` - Remove duplicate, import from core
2. `crates/llm/src/lib.rs` - Re-export from core
3. Any files that use `llm::ToolDefinition`

**Estimate:** 2-3 hours

---

### P0-3: Implement LLM Tool Calling

**Issue:** `generate_with_tools()` accepts tools parameter but ignores it completely.

**Location:** `crates/llm/src/adapter.rs:138-146`

**Fix:**

```rust
// crates/llm/src/adapter.rs

async fn generate_with_tools(
    &self,
    request: GenerateRequest,
    tools: &[ToolDefinition],
) -> Result<GenerateResponse> {
    // 1. Build system prompt with tool definitions
    let mut system = request.system.clone().unwrap_or_default();
    if !tools.is_empty() {
        system.push_str("\n\n## Available Tools\n");
        for tool in tools {
            system.push_str(&format!(
                "### {}\n{}\nParameters: {}\n\n",
                tool.name, tool.description,
                serde_json::to_string_pretty(&tool.parameters)?
            ));
        }
        system.push_str("\nTo use a tool, respond with:\n[TOOL_CALL: {\"name\": \"tool_name\", \"arguments\": {...}}]\n");
    }

    let modified_request = GenerateRequest {
        system: Some(system),
        ..request
    };

    // 2. Generate response
    let response = self.generate(modified_request).await?;

    // 3. Parse tool calls from response
    let tool_calls = self.parse_tool_calls(&response.text);

    Ok(GenerateResponse {
        tool_calls,
        ..response
    })
}

fn parse_tool_calls(&self, text: &str) -> Vec<ToolCall> {
    let re = regex::Regex::new(r"\[TOOL_CALL:\s*(\{[^}]+\})\]").unwrap();
    re.captures_iter(text)
        .filter_map(|cap| {
            serde_json::from_str::<ToolCallJson>(&cap[1]).ok()
        })
        .map(|tc| ToolCall {
            id: uuid::Uuid::new_v4().to_string(),
            name: tc.name,
            arguments: tc.arguments,
        })
        .collect()
}
```

**Estimate:** 4-6 hours

---

### P0-4: Wire Domain Config to Tools

**Issue:** Tools use `GoldLoanConfig::default()` instead of loaded `DomainConfigManager`.

**Location:** `crates/tools/src/gold_loan.rs`

**Fix:**

```rust
// 1. Update tool constructors to accept config
impl EligibilityCheckTool {
    pub fn new() -> Self {
        Self { config: GoldLoanConfig::default() }
    }

    // NEW: Add config injection
    pub fn with_config(config: GoldLoanConfig) -> Self {
        Self { config }
    }
}

// 2. Update registry factory
pub fn create_registry_with_config(domain_config: &DomainConfigManager) -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    let gold_config = domain_config.config().gold_loan.clone();

    registry.register(EligibilityCheckTool::with_config(gold_config.clone()));
    registry.register(SavingsCalculatorTool::with_config(gold_config.clone()));
    // ... other tools

    registry
}

// 3. Update server state.rs
impl AppState {
    pub fn new(config: Settings, domain_config: Arc<DomainConfigManager>) -> Self {
        let tools = create_registry_with_config(&domain_config);
        // ...
    }
}
```

**Files to Update:**
1. `crates/tools/src/gold_loan.rs` - Add `with_config()` to all tools
2. `crates/tools/src/registry.rs` - Add `create_registry_with_config()`
3. `crates/server/src/state.rs` - Use new factory
4. `crates/server/src/main.rs` - Pass domain config to state

**Estimate:** 3-4 hours

---

## P1 - Required for Full Feature Set

### P1-1: Wire Query Expansion to HybridRetriever

**Issue:** Query expansion implemented but bypassed when agent calls HybridRetriever directly.

**Location:** `crates/rag/src/retriever.rs`

**Fix:**

```rust
// crates/rag/src/retriever.rs

impl HybridRetriever {
    pub async fn search(
        &self,
        query: &str,
        vector_store: &VectorStore,
        options: Option<&RetrieveOptions>,
    ) -> Result<Vec<Document>> {
        // NEW: Apply query expansion before search
        let expanded = if self.config.enable_query_expansion {
            self.expander.expand(query, Language::Hindi).await
        } else {
            QueryExpansion {
                original: query.to_string(),
                expanded_terms: vec![],
                ..Default::default()
            }
        };

        // Use expanded query for search
        let search_query = expanded.to_search_string();

        // Rest of existing search logic...
    }
}
```

**Estimate:** 2-3 hours

---

### P1-2: Use LanguageModelAdapter in Agent

**Issue:** Agent uses `LlmBackend` directly, bypassing the adapter that implements `core::LanguageModel`.

**Location:** `crates/agent/src/agent.rs`

**Fix:**

```rust
// crates/agent/src/agent.rs

// BEFORE:
use voice_agent_llm::{LlmBackend, OllamaBackend};
llm: Option<Arc<dyn LlmBackend>>,

// AFTER:
use voice_agent_core::traits::LanguageModel;
use voice_agent_llm::{LanguageModelAdapter, OllamaBackend};
llm: Option<Arc<dyn LanguageModel>>,

// In builder:
let backend = OllamaBackend::new(config)?;
let llm: Arc<dyn LanguageModel> = Arc::new(LanguageModelAdapter::new(backend));
```

**Estimate:** 2-3 hours

---

### P1-3: Wire Personalization Signals to Behavior

**Issue:** Signals are detected and logged but don't affect agent behavior.

**Location:** `crates/agent/src/agent.rs:615-626`

**Fix:**

```rust
// In generate_response() method:

// 1. Detect signals
let signals = self.personalization_engine.process_input(&mut ctx, user_input);

// 2. Generate personalization instructions
let instructions = self.personalization_engine.generate_instructions(&ctx);

// 3. Adjust response generation parameters based on signals
let mut generation_params = self.default_params.clone();
for signal in &signals {
    match signal {
        Signal::Urgency => {
            generation_params.temperature = 0.3; // More focused
            generation_params.max_tokens = 100; // Shorter responses
        }
        Signal::Confusion => {
            // Add clarification prompt
            instructions.push("Explain more simply and offer examples.");
        }
        Signal::PriceObjection => {
            // Trigger savings calculator
            self.trigger_tool("calculate_savings", &ctx).await?;
        }
        // ... handle other signals
    }
}

// 4. Add instructions to prompt
if !instructions.is_empty() {
    prompt_builder.add_personalization_guidance(&instructions);
}
```

**Estimate:** 4-6 hours

---

### P1-4: Connect WebRTC Audio to Pipeline

**Issue:** WebRTC signaling works but audio is not routed to voice pipeline.

**Location:** `crates/server/src/webrtc.rs`

**Fix:**

```rust
// crates/server/src/webrtc.rs

pub async fn handle_offer(
    session_id: &str,
    offer: &str,
    state: &AppState,
) -> Result<String> {
    let session = state.sessions.get(session_id)?;

    // 1. Create WebRTC transport
    let mut transport = WebRtcTransport::new(webrtc_config).await?;

    // 2. Set up audio callbacks to pipeline
    let (audio_tx, audio_rx) = mpsc::channel(100);
    transport.set_audio_callback(move |audio_data| {
        let _ = audio_tx.try_send(audio_data);
    });

    // 3. Start audio processing task
    let pipeline = session.pipeline.clone();
    tokio::spawn(async move {
        while let Some(audio) = audio_rx.recv().await {
            let frame = AudioFrame::from_opus(&audio)?;
            pipeline.process_audio(frame).await?;
        }
    });

    // 4. Connect pipeline output to transport
    let transport_clone = transport.clone();
    pipeline.on_audio_output(move |frame| {
        let opus = frame.to_opus()?;
        transport_clone.send_audio(&opus)?;
    });

    // 5. Process offer
    let answer = transport.accept(offer).await?;
    session.set_webrtc_transport(transport);

    Ok(answer)
}
```

**Estimate:** 6-8 hours

---

## P2 - Production Hardening

### P2-1: Remove Dead Code

**Files to Clean:**

```rust
// 1. crates/rag/src/reranker.rs
// REMOVE lines 578-664:
// - should_exit()
// - LayerOutput struct
// - cosine_similarity()

// 2. crates/llm/src/prompt.rs
// REMOVE lines 46-53:
// - Duplicate ToolDefinition struct

// 3. crates/transport/src/websocket.rs
// Keep but mark as deprecated or remove if never used
```

**Estimate:** 1-2 hours

---

### P2-2: Add Missing Message Fields in LLM Adapter

**Location:** `crates/llm/src/adapter.rs:56-68`

**Fix:**

```rust
fn convert_messages(request: &GenerateRequest) -> Vec<crate::prompt::Message> {
    request.messages.iter().map(|m| {
        crate::prompt::Message {
            role: match m.role {
                voice_agent_core::llm_types::Role::System => crate::prompt::Role::System,
                voice_agent_core::llm_types::Role::User => crate::prompt::Role::User,
                voice_agent_core::llm_types::Role::Assistant => crate::prompt::Role::Assistant,
                voice_agent_core::llm_types::Role::Tool => crate::prompt::Role::Tool, // FIX: Don't map to User
            },
            content: m.content.clone(),
            name: m.name.clone(),           // FIX: Preserve name
            tool_call_id: m.tool_call_id.clone(), // FIX: Preserve tool_call_id
        }
    }).collect()
}
```

**Estimate:** 1-2 hours

---

### P2-3: Implement Session Persistence

**Location:** `crates/persistence/`

**Options:**
1. Redis (already stubbed) - Implement actual Redis client
2. ScyllaDB (partially implemented) - Complete implementation
3. SQLite - Simple option for single-node deployment

**Estimate:** 8-12 hours (depending on chosen option)

---

## P3 - Nice to Have

### P3-1: Move Tool Trait to Core Crate

**Current:** `crates/tools/src/mcp.rs:315-372`
**Target:** `crates/core/src/traits/tool.rs`

### P3-2: Auto-Detect Customer Segment

Implement automatic segment detection from conversation facts:
- Loan amount mentioned → P1 High Value
- Urgency signals → P2 Urgent
- Price sensitivity → P3 Price Sensitive

### P3-3: Full MCP Protocol Compliance

Add MCP request/response envelopes, resource management, progress reporting.

---

## Implementation Order

```
Week 1:
├── P0-1: ConversationFSM trait (4-6h)
├── P0-2: Fix ToolDefinition mismatch (2-3h)
├── P0-3: Implement LLM tool calling (4-6h)
└── P0-4: Wire domain config to tools (3-4h)

Week 2:
├── P1-1: Wire query expansion (2-3h)
├── P1-2: Use LanguageModelAdapter (2-3h)
├── P1-3: Wire personalization signals (4-6h)
└── P2-1: Remove dead code (1-2h)

Week 3-4:
├── P1-4: Connect WebRTC audio (6-8h)
├── P2-2: Fix message fields (1-2h)
└── P2-3: Session persistence (8-12h)
```

---

## Testing Checklist

After fixes, verify:

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes
- [ ] Tool calling works end-to-end
- [ ] Domain config changes affect tool calculations
- [ ] Query expansion improves Hindi recall
- [ ] Personalization signals modify responses
- [ ] WebRTC audio flows to pipeline
- [ ] Session persistence survives restart
