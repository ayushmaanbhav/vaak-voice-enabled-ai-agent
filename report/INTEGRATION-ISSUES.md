# Integration Issues - End-to-End Analysis

## Critical Integration Gaps

This document identifies where components are disconnected or not properly wired together.

---

## 1. Transport Crate Completely Isolated

### Status: CRITICAL (1,500+ LOC unused)

**The Problem:**
- Transport crate has complete WebRTC implementation
- Server crate does NOT depend on transport crate
- No WebRTC signaling endpoints exist
- All WebRTC code is dead

**Evidence:**
```toml
# server/Cargo.toml - MISSING:
voice-agent-transport = { path = "../transport" }
```

**Impact:**
- Mobile clients cannot use low-latency WebRTC
- Fallback mechanism never activated
- Development effort wasted

**Files Affected:**
- `transport/src/webrtc.rs` (843 lines) - UNUSED
- `transport/src/session.rs` (288 lines) - UNUSED
- `transport/src/codec.rs` (374 lines) - UNUSED

---

## 2. Frame Processors Disconnected from Orchestrator

### Status: HIGH (Architectural mismatch)

**The Problem:**
- Pipeline spec describes frame-based architecture
- Orchestrator uses traditional state machine
- SentenceDetector, TtsProcessor, InterruptHandler exist but aren't called

**Evidence:**
```rust
// orchestrator.rs - Uses state machine:
match *self.state.lock() {
    PipelineState::Listening => { ... }
    PipelineState::Speaking => { ... }
}

// processors/chain.rs - Has full implementation but orchestrator doesn't use it:
pub struct ProcessorChain { ... }
```

**Impact:**
- Sentence-level streaming not used
- Advanced interrupt handling not active
- Spec/implementation mismatch

**Files Affected:**
- `pipeline/src/processors/chain.rs` - EXISTS, not wired
- `pipeline/src/processors/sentence_detector.rs` - EXISTS, not wired
- `pipeline/src/processors/tts_processor.rs` - EXISTS, not wired

---

## 3. PersuasionEngine Created but Not Invoked

### Status: HIGH (Feature incomplete)

**The Problem:**
- PersuasionEngine is initialized
- ObjectionType detection exists
- But `handle_objection()` never called in conversation flow

**Evidence:**
```rust
// agent.rs - Engine exists:
pub struct GoldLoanAgent {
    persuasion: PersuasionEngine,  // Created but unused
}

// agent.rs - generate_response() - NO objection handling branch
```

**Impact:**
- Objections detected but not handled specially
- Persuasion techniques not applied
- Sales effectiveness reduced

---

## 4. LlmBackend vs LanguageModel Trait Mismatch

### Status: MEDIUM (Type incompatibility)

**The Problem:**
- Core defines `LanguageModel` trait
- LLM crate implements `LlmBackend` trait (different signatures)
- No adapter layer exists

**Evidence:**
```rust
// core/traits/llm.rs:
#[async_trait]
pub trait LanguageModel {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse, Error>;
}

// llm/backend.rs:
#[async_trait]
pub trait LlmBackend {
    async fn generate(&self, messages: &[Message]) -> Result<GenerationResult, LlmError>;
}
```

**Impact:**
- Cannot use LLM implementations with core trait
- Prevents clean dependency injection
- Limits composability

---

## 5. Text Processing Traits Defined but Missing Implementations

### Status: MEDIUM (Partial implementations)

**Traits in Core with Missing/Partial Implementations:**

| Trait | Implementation Status |
|-------|----------------------|
| GrammarCorrector | ✅ LLMGrammarCorrector exists |
| Translator | ✅ CandleIndicTrans2Translator exists |
| PIIRedactor | ✅ HybridPIIDetector exists |
| ComplianceChecker | ✅ RuleBasedComplianceChecker exists |

**Note:** After deep analysis, these DO exist in text_processing crate. The initial concern was unfounded.

---

## 6. Configuration Files Missing

### Status: MEDIUM (Runtime initialization issues)

**The Problem:**
- Settings loader references YAML files
- Files don't exist in repository
- Relies on environment variables as fallback

**Expected Files:**
```
config/
├── default.yaml       # MISSING
├── development.yaml   # MISSING
├── staging.yaml       # MISSING
├── production.yaml    # MISSING
└── domain.yaml        # MISSING
```

**Impact:**
- Configuration not version-controlled
- Deployment requires manual setup
- Settings validation doesn't catch missing files

---

## 7. A/B Testing Framework Missing

### Status: LOW (Feature gap)

**The Problem:**
- FeatureFlags exist (binary toggles)
- No experiments.rs for A/B testing
- No variant bucketing or percentage rollouts

**Expected but Missing:**
```rust
// experiments.rs
pub struct Experiment {
    id: String,
    variants: Vec<Variant>,
    rollout_percentage: f32,
}
```

**Impact:**
- Cannot test features incrementally
- No data-driven optimization
- All-or-nothing feature launches

---

## 8. RAG Not Integrated with Pipeline

### Status: MEDIUM (Manual integration only)

**The Problem:**
- RAG retrieval works standalone
- Pipeline doesn't call RAG directly
- Agent manually orchestrates RAG calls

**Current Flow:**
```
User Speech → STT → Agent → [Manual RAG call] → LLM → TTS
```

**Spec Flow:**
```
User Speech → STT → [Prefetch RAG on partial] → Turn Complete → LLM+RAG → TTS
```

**Impact:**
- Prefetch optimization not in pipeline
- Latency higher than spec target
- Manual orchestration in agent

---

## 9. Personalization Limited to Agent

### Status: LOW (Good design, limited scope)

**The Problem:**
- PersonalizationEngine well-implemented
- Only used in agent.rs
- Could benefit pipeline (TTS voice selection, etc.)

**Current:**
- Agent uses personalization for prompt engineering
- Pipeline ignores customer segment

**Potential:**
- TTS voice selection based on persona
- VAD sensitivity based on user patterns
- Response length based on segment

---

## 10. Session Touch Bug - FIXED

**Status:** Verified FIXED

The session touch implementation now correctly updates timestamp:
```rust
async fn touch(&self, id: &str) -> Result<(), ServerError> {
    if let Some(meta) = self.metadata.write().get_mut(id) {
        meta.last_activity_ms = Instant::now();  // Correctly updated
    }
    Ok(())
}
```

---

## Integration Matrix

| Component A | Component B | Integration Status |
|-------------|-------------|-------------------|
| Server | Transport | ❌ NOT CONNECTED |
| Orchestrator | Processors | ❌ NOT CONNECTED |
| Agent | PersuasionEngine | ⚠️ CREATED, NOT USED |
| LLM | Core Traits | ⚠️ DIFFERENT TRAITS |
| Pipeline | RAG | ⚠️ MANUAL ONLY |
| Config | YAML Files | ⚠️ FILES MISSING |
| Agent | Personalization | ✅ CONNECTED |
| Agent | Tools | ✅ CONNECTED |
| Pipeline | STT/TTS | ✅ CONNECTED |
| Server | WebSocket | ✅ CONNECTED |

---

## Priority for Integration Fixes

### Must Fix (P0):
1. Wire Transport to Server (or remove unused code)
2. Integrate PersuasionEngine in Agent

### Should Fix (P1):
3. Create LanguageModel adapter
4. Create configuration files

### Nice to Have (P2):
5. Wire Frame Processors (or document limitation)
6. Add A/B testing framework
7. Expand Personalization to Pipeline
