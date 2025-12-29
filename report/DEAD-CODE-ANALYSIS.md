# Dead Code and Missing Connections Analysis

> Comprehensive inventory of dead code, stubbed implementations, and disconnected components

---

## Dead Code Inventory

### 1. RAG Crate - Reranker Dead Code

| Item | File | Lines | Status | Reason |
|------|------|-------|--------|--------|
| `should_exit()` | `reranker.rs` | 578-635 | DEAD | ONNX can't provide per-layer outputs |
| `LayerOutput` struct | `reranker.rs` | 128-137 | DEAD | Only used by dead `should_exit()` |
| `cosine_similarity()` | `reranker.rs` | 649-664 | DEAD | Marked `#[allow(dead_code)]` |
| `reranker_config` field | `retriever.rs` | 100-101 | DEAD | Marked "may be used for lazy init" |

**Evidence:**
```rust
// reranker.rs:578
#[allow(dead_code)]
fn should_exit(&self, scores: &[f32], confidence_threshold: f32) -> bool {
    // This function exists but is never called
    // ONNX Runtime executes full graph - no pause capability
}
```

---

### 2. RAG Crate - Query Expansion Dead Flow

| Item | File | Lines | Status | Reason |
|------|------|-------|--------|--------|
| Query expansion in direct search | `retriever.rs` | - | NOT WIRED | Agent calls HybridRetriever directly |

**Flow Analysis:**
```
CONFIGURED PATH (working but unused):
  QueryExpander → EnhancedRetriever adapter → adapter.rs:119-124

ACTUAL PATH (used by agent):
  Agent.process() → HybridRetriever.search() → NO EXPANSION

Location: agent.rs:899
  retriever.search(user_input, vector_store, None).await
  // Query expansion is BYPASSED
```

---

### 3. LLM Crate - Tool Calling Dead Code

| Item | File | Lines | Status | Reason |
|------|------|-------|--------|--------|
| `generate_with_tools()` implementation | `adapter.rs` | 138-146 | STUBBED | Tools parameter ignored |
| `ToolDefinition` struct | `prompt.rs` | 46-53 | CONFLICTING | Different from core type |
| `ParsedToolCall` struct | `prompt.rs` | 107-138 | UNUSED | Never parsed from LLM output |

**Evidence:**
```rust
// adapter.rs:138-146
async fn generate_with_tools(
    &self,
    request: GenerateRequest,
    _tools: &[ToolDefinition],  // Prefixed with _ = unused!
) -> Result<GenerateResponse> {
    // Current backends don't support tool calling, fall back to regular generate
    self.generate(request).await  // Tools completely ignored
}
```

---

### 4. LLM Crate - Adapter Not Used

| Item | File | Lines | Status | Reason |
|------|------|-------|--------|--------|
| `LanguageModelAdapter` | `adapter.rs` | 34-164 | IMPLEMENTED BUT UNUSED | Agent uses LlmBackend directly |

**Evidence:**
```rust
// agent.rs:9 - Direct backend use
use voice_agent_llm::{PromptBuilder, Message, Role, OllamaBackend, LlmBackend, LlmConfig};

// agent.rs:139 - Stores backend, not adapter
llm: Option<Arc<dyn LlmBackend>>,

// Should be:
// llm: Option<Arc<dyn LanguageModel>>,  // Using core trait
```

---

### 5. Agent Crate - Personalization Dead Flow

| Item | File | Lines | Status | Reason |
|------|------|-------|--------|--------|
| Signal detection | `agent.rs` | 615-626 | LOGGED ONLY | Signals not used in behavior |
| `handle_objection()` | `personalization/engine.rs` | - | NOT CALLED | Persuasion engine used instead |
| Auto segment detection | - | - | NOT IMPLEMENTED | Must be manually set |

**Evidence:**
```rust
// agent.rs:620-624 - Signals detected but logged only
let _signals = engine.process_input(&mut ctx, user_input);
let recent = ctx.recent_signals(1);
if !recent.is_empty() {
    tracing::debug!(signals = ?recent, "Detected personalization signals");
    // Signals logged but not used to modify behavior
}
```

---

### 6. Transport Crate - Full Implementation Unused

| Item | File | Lines | Status | Reason |
|------|------|-------|--------|--------|
| `WebRtcTransport` | `webrtc.rs` | - | IMPLEMENTED | Server doesn't use it for audio |
| `WebRtcAudioSink` | `webrtc.rs` | - | IMPLEMENTED | Not connected to pipeline |
| `WebRtcAudioSource` | `webrtc.rs` | - | IMPLEMENTED | Not connected to pipeline |
| `OpusEncoder` | `codec.rs` | - | IMPLEMENTED | Server does raw PCM |
| `OpusDecoder` | `codec.rs` | - | IMPLEMENTED | Server does raw PCM |
| `Transport` trait | `traits.rs` | - | IMPLEMENTED | Server has own WebSocket |

**Evidence:**
```rust
// transport/src/websocket.rs - Returns None
fn audio_sink(&self) -> Option<Box<dyn AudioSink>> {
    None // Implemented in server crate
}
```

---

### 7. Tools Crate - Integration Stubs

| Item | File | Lines | Status | Reason |
|------|------|-------|--------|--------|
| `StubCrmIntegration` | `integrations.rs` | - | DEFAULT USED | No real CRM |
| `StubCalendarIntegration` | `integrations.rs` | - | DEFAULT USED | No real calendar |
| GoldPriceService | `gold_loan.rs` | - | OPTIONAL | Hardcoded fallback used |
| SmsService | `gold_loan.rs` | - | OPTIONAL | Simulated sends |

**Evidence:**
```rust
// registry.rs:203-206 - Default registry uses stubs
registry.register(crate::gold_loan::LeadCaptureTool::new());  // No CRM passed
registry.register(crate::gold_loan::AppointmentSchedulerTool::new());  // No Calendar passed
```

---

### 8. Tools Crate - Config Not Wired

| Item | File | Lines | Status | Reason |
|------|------|-------|--------|--------|
| Domain config usage | `gold_loan.rs` | 180-182 | NOT WIRED | Tools use hardcoded defaults |

**Evidence:**
```rust
// tools/gold_loan.rs:180-182 - Hardcoded config
pub fn new() -> Self {
    Self {
        config: GoldLoanConfig::default(),  // NOT from DomainConfigManager
    }
}

// Should be:
pub fn with_config(config: GoldLoanConfig) -> Self { ... }
```

---

### 9. Core Crate - Missing Trait

| Item | File | Lines | Status | Reason |
|------|------|-------|--------|--------|
| `ConversationFSM` trait | - | - | NOT DEFINED | Only enum exists, no trait |

**What Exists:**
```rust
// conversation.rs - Only enum
pub enum ConversationStage {
    Greeting, Discovery, Qualification,
    Presentation, ObjectionHandling, Closing, Farewell,
}
// Missing: ConversationFSM trait with state(), transition(), checkpoint(), etc.
```

---

### 10. Text Processing - gRPC Stub

| Item | File | Lines | Status | Reason |
|------|------|-------|--------|--------|
| gRPC translator | `translation/grpc.rs` | 93-304 | STUB | Returns input unchanged |

**Evidence:**
```rust
// grpc.rs:135-142
async fn translate(&self, text: &str, from: Language, to: Language) -> Result<String> {
    // TODO: Implement actual HTTP client
    // Placeholder implementation
    Ok(text.to_string())  // Returns unchanged!
}
```

---

## Disconnected Components Summary

### Fully Disconnected

| Component | Location | Reason |
|-----------|----------|--------|
| Transport crate | `crates/transport/` | Server implements own WebSocket |
| WebRTC audio | `server/webrtc.rs` | Signaling only, no audio routing |
| LanguageModelAdapter | `llm/adapter.rs` | Agent uses LlmBackend directly |
| ConversationFSM trait | Not defined | Only enum exists |

### Partially Disconnected

| Component | What Works | What's Broken |
|-----------|------------|---------------|
| Query Expansion | EnhancedRetriever adapter | HybridRetriever direct calls |
| Personalization | Signal detection | Signal → behavior mapping |
| Tool Calling | Intent-based | LLM tool_use parsing |
| Domain Config | Loading, validation | Injection into tools |

### Integration Priority

```
HIGH PRIORITY (breaks production):
1. ConversationFSM trait - blocks proper state machine
2. ToolDefinition type mismatch - blocks tool calling
3. Tools config injection - tools ignore config changes

MEDIUM PRIORITY (reduces functionality):
4. Query expansion wiring - lower Hindi recall
5. Personalization signals - no behavioral adaptation
6. LanguageModelAdapter usage - type inconsistency

LOW PRIORITY (optimization):
7. WebRTC audio routing - WebSocket works
8. Transport crate integration - manual WebSocket works
9. Dead reranker code - doesn't affect function
```

---

## Removal Candidates

Safe to remove without affecting functionality:

```rust
// 1. reranker.rs - Dead early-exit code
#[allow(dead_code)]
fn should_exit(...) { ... }
struct LayerOutput { ... }
fn cosine_similarity(...) { ... }

// 2. prompt.rs - Duplicate ToolDefinition (use core type)
pub struct ToolDefinition { ... }  // Remove, import from core

// 3. sparse_search.rs:61 - Unknown dead code
#[allow(dead_code)]
// Whatever is on line 61
```

---

## Code Smell Indicators

| Pattern | Occurrences | Files |
|---------|-------------|-------|
| `#[allow(dead_code)]` | 5+ | reranker.rs, sparse_search.rs |
| `_` prefixed params | 3+ | adapter.rs, gold_loan.rs |
| `// TODO` comments | 10+ | grpc.rs, integrations.rs |
| `unimplemented!()` | 2 | transport/ |
| `.default()` in tools | 8 | gold_loan.rs |
| `None // Implemented in...` | 2 | transport/websocket.rs |
