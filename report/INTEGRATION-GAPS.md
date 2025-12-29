# Integration Gaps - Missing Connections

> Detailed mapping of what should be connected but isn't

---

## Visual System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           VOICE AGENT SYSTEM ARCHITECTURE                        │
│                           (✓ = Connected, ✗ = Disconnected)                      │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐   │
│  │   SERVER    │────►│  PIPELINE   │────►│    AGENT    │────►│     LLM     │   │
│  │  (axum)     │ ✓   │ (frame-based│ ✓   │ (GoldLoan)  │ ✓   │  (Ollama)   │   │
│  └─────────────┘     │  streaming) │     └──────┬──────┘     └──────┬──────┘   │
│         │            └─────────────┘            │                    │          │
│         │                                       │                    │          │
│    WebSocket ✓                            ┌─────┴─────┐         ┌────┴────┐    │
│    WebRTC ✗ (signaling only)              │           │         │         │    │
│         │                                 ▼           ▼         ▼         │    │
│         │                           ┌─────────┐ ┌─────────┐ ┌───────┐    │    │
│         │                           │  TOOLS  │ │   RAG   │ │ TEXT  │    │    │
│         │                           │(8 tools)│ │(hybrid) │ │PROCESS│    │    │
│         │                           └────┬────┘ └────┬────┘ └───┬───┘    │    │
│         │                                │           │          │         │    │
│         │                                │           │          │         │    │
│  ┌──────┴──────┐                         │           │          │         │    │
│  │  TRANSPORT  │ ✗ NOT USED              │           │          │         │    │
│  │  (WebRTC)   │                         │           │          │         │    │
│  └─────────────┘                         │           │          │         │    │
│                                          │           │          │         │    │
│  ┌─────────────┐                    ┌────┴────┐ ┌────┴────┐     │         │    │
│  │   CONFIG    │───────────────────►│ CONFIG  │ │ QUERY   │     │         │    │
│  │  (domain)   │ ✗ NOT WIRED       │ DEFAULT │ │ EXPAND  │     │         │    │
│  └─────────────┘  to tools          └─────────┘ └────┬────┘     │         │    │
│                                                      │          │         │    │
│                                                  ✗ NOT WIRED    │         │    │
│                                                  to retriever   │         │    │
│                                                                 │         │    │
│  ┌─────────────┐                                                │         │    │
│  │ PERSONAL-   │ ✗ Signals detected but                         │         │    │
│  │ IZATION     │   NOT USED in responses                        │         │    │
│  └─────────────┘                                                │         │    │
│                                                                 │         │    │
│  ┌─────────────┐                                                │         │    │
│  │ LLM ADAPTER │ ✗ Implemented but agent uses                   │         │    │
│  │ (core trait)│   LlmBackend directly                          │         │    │
│  └─────────────┘                                                │         │    │
│                                                                 │         │    │
│  ┌─────────────┐                                                │         │    │
│  │ TOOL CALLING│ ✗ generate_with_tools()                        │         │    │
│  │ (LLM)       │   ignores tools parameter                      │         │    │
│  └─────────────┘                                                │         │    │
│                                                                 │         │    │
│  ┌─────────────┐                                                │         │    │
│  │ FSM TRAIT   │ ✗ NOT DEFINED                                  │         │    │
│  │ (core)      │   Only enum exists                             │         │    │
│  └─────────────┘                                                │         │    │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Gap 1: Config → Tools

### What Should Happen
```
DomainConfigManager (config crate)
    │
    ├── Loaded at startup from config/domain.yaml
    ├── Contains: gold_loan, branches, competitors, prompts
    │
    └── Should inject into →
            │
            ├── EligibilityCheckTool (gold_loan rates, LTV)
            ├── SavingsCalculatorTool (competitor rates)
            ├── BranchLocatorTool (branch data)
            └── All other tools
```

### What Actually Happens
```
DomainConfigManager
    │
    ├── Loaded ✓
    ├── Validated ✓
    ├── Stored in AppState ✓
    │
    └── Tools use →
            │
            └── GoldLoanConfig::default() ← HARDCODED!
```

### Fix Location
- `crates/tools/src/gold_loan.rs:180-182`
- `crates/tools/src/registry.rs`
- `crates/server/src/state.rs`

---

## Gap 2: Query Expansion → Retriever

### What Should Happen
```
User Query ("sona loan chahiye")
    │
    └── QueryExpander
            │
            ├── Synonym expansion: "sona" → ["gold", "swarna"]
            ├── Transliteration: "सोना" ↔ "sona"
            ├── Domain terms: "gold loan" → ["sona karza"]
            │
            └── HybridRetriever.search(expanded_query)
```

### What Actually Happens
```
User Query
    │
    ├── Agent calls HybridRetriever.search(original_query) directly
    │   └── No expansion applied
    │
    └── QueryExpander exists but only wired to EnhancedRetriever
        └── EnhancedRetriever NOT USED by agent
```

### Fix Location
- `crates/rag/src/retriever.rs` (add expansion in search())
- `crates/agent/src/agent.rs:899`

---

## Gap 3: LLM Adapter → Agent

### What Should Happen
```
Agent
    │
    └── Uses Arc<dyn LanguageModel> (core trait)
            │
            └── LanguageModelAdapter (llm crate)
                    │
                    ├── Implements core::LanguageModel
                    ├── Handles type conversion
                    └── Wraps LlmBackend (Ollama/OpenAI)
```

### What Actually Happens
```
Agent
    │
    └── Uses Arc<dyn LlmBackend> directly
            │
            ├── Bypasses adapter
            ├── Bypasses core trait
            └── Type inconsistency with core crate
```

### Fix Location
- `crates/agent/src/agent.rs:9,139,181-186`

---

## Gap 4: Tool Calling → LLM

### What Should Happen
```
Agent wants tool call
    │
    └── LLM.generate_with_tools(request, tools)
            │
            ├── Tools injected in system prompt
            ├── LLM generates: [TOOL_CALL: {...}]
            ├── Response parsed for tool calls
            │
            └── Returns GenerateResponse { tool_calls: [...] }
```

### What Actually Happens
```
Agent wants tool call
    │
    ├── Agent detects intent → triggers tool directly
    │   └── Intent-based, not LLM-based
    │
    └── LLM.generate_with_tools(request, tools)
            │
            └── Ignores tools, calls regular generate()
```

### Fix Location
- `crates/llm/src/adapter.rs:138-146`
- `crates/llm/src/prompt.rs`

---

## Gap 5: Personalization → Response

### What Should Happen
```
User Input
    │
    └── PersonalizationEngine.process_input()
            │
            ├── Detects signals: Urgency, Confusion, PriceObjection
            ├── Updates PersonalizationContext
            │
            └── generate_instructions() →
                    │
                    └── Modifies response generation:
                        ├── Temperature adjustment
                        ├── Response length
                        ├── Tool triggering
                        └── Prompt guidance
```

### What Actually Happens
```
User Input
    │
    └── PersonalizationEngine.process_input()
            │
            ├── Detects signals ✓
            ├── Logs signals ✓
            │
            └── generate_instructions() →
                    │
                    └── Added to prompt IF signals exist
                        └── But rarely modifies behavior
```

### Fix Location
- `crates/agent/src/agent.rs:615-626,858-873`

---

## Gap 6: WebRTC → Pipeline

### What Should Happen
```
WebRTC Client
    │
    ├── POST /api/webrtc/:id/offer
    │       └── Server creates peer connection
    │
    ├── Audio flows via WebRTC track
    │       │
    │       └── WebRtcAudioSource (transport crate)
    │               │
    │               └── Decodes Opus → AudioFrame
    │                       │
    │                       └── VoicePipeline.process_audio()
    │
    └── Response audio
            │
            └── Pipeline → WebRtcAudioSink → Client
```

### What Actually Happens
```
WebRTC Client
    │
    ├── POST /api/webrtc/:id/offer
    │       └── Server creates peer connection ✓
    │       └── Returns SDP answer ✓
    │
    ├── ICE candidates exchanged ✓
    │
    └── Audio? → NOWHERE
            │
            └── WebRTC transport exists but audio not routed
```

### Fix Location
- `crates/server/src/webrtc.rs`
- Needs audio callback to pipeline

---

## Gap 7: ConversationFSM Trait → Implementation

### What Should Happen
```
Core Crate defines:
    │
    └── trait ConversationFSM
            │
            ├── state() → current stage
            ├── transition(event) → actions
            ├── checkpoint() → save state
            └── restore() → recover state

Agent Crate implements:
    │
    └── impl ConversationFSM for GoldLoanAgent
```

### What Actually Happens
```
Core Crate defines:
    │
    └── enum ConversationStage (7 states)
            │
            └── NO TRAIT DEFINED

Agent Crate:
    │
    └── StageManager (custom implementation)
            │
            └── Works, but not conforming to trait
```

### Fix Location
- Create `crates/core/src/traits/fsm.rs`

---

## Connection Matrix

| Source | Target | Expected | Actual | Gap |
|--------|--------|----------|--------|-----|
| Server | Pipeline | Audio flow | ✓ WebSocket, ✗ WebRTC | WebRTC audio |
| Pipeline | Agent | Transcript | ✓ Working | None |
| Agent | LLM | Generation | ✓ Working (via backend) | Should use adapter |
| Agent | Tools | Execution | ✓ Intent-based | Should add LLM tool_use |
| Agent | RAG | Retrieval | ✓ Working | Query expansion bypass |
| Config | Tools | Injection | ✗ Hardcoded | Full rewiring needed |
| Personalization | Agent | Signals | ✗ Logged only | Behavior mapping |
| Core Traits | Agent | FSM | ✗ No trait | Define trait |
| Transport | Server | Audio | ✗ Not used | Full integration |

---

## Integration Test Scenarios

After fixes, these should pass:

### Test 1: Tool Config Integration
```rust
#[tokio::test]
async fn test_tool_uses_domain_config() {
    let config = DomainConfigManager::from_file("config/domain.yaml").unwrap();
    config.config_mut().gold_loan.kotak_interest_rate = 9.0; // Custom rate

    let registry = create_registry_with_config(&config);
    let tool = registry.get("calculate_savings").unwrap();

    let result = tool.execute(json!({
        "current_loan_amount": 100000,
        "current_interest_rate": 18.0,
        "remaining_tenure_months": 12
    })).await.unwrap();

    // Should use 9.0% rate from config, not default 10.5%
    assert!(result["kotak_rate"].as_f64().unwrap() == 9.0);
}
```

### Test 2: Query Expansion in Retrieval
```rust
#[tokio::test]
async fn test_query_expansion_in_search() {
    let retriever = HybridRetriever::new(config).await.unwrap();

    // Search with Hindi term
    let results = retriever.search("sona loan chahiye", &store, None).await.unwrap();

    // Should find results for "gold loan" via expansion
    assert!(!results.is_empty());
    assert!(results.iter().any(|d| d.content.contains("gold loan")));
}
```

### Test 3: WebRTC Audio Flow
```rust
#[tokio::test]
async fn test_webrtc_audio_to_pipeline() {
    let state = create_test_state().await;
    let session_id = create_session(&state).await;

    // Send WebRTC offer
    let answer = handle_offer(session_id, VALID_SDP_OFFER, &state).await.unwrap();
    assert!(!answer.is_empty());

    // Simulate audio from WebRTC
    let session = state.sessions.get(session_id).unwrap();
    let webrtc = session.webrtc.read().unwrap().as_ref().unwrap();

    webrtc.inject_audio(&test_audio_samples()).await;

    // Verify pipeline received audio
    let events = session.pipeline.drain_events().await;
    assert!(events.iter().any(|e| matches!(e, PipelineEvent::VadStateChanged(_))));
}
```
