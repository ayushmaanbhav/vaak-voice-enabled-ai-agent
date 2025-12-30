# Voice Agent Backend - Fix Plan

**Date:** December 30, 2025
**Reference:** DEEP_DIVE_REPORT.md

---

## Priority Definitions

| Priority | Definition | Timeline Guidance |
|----------|------------|-------------------|
| **P0** | System broken/unusable without fix | Immediate |
| **P1** | Significant functionality impacted | High priority |
| **P2** | Code quality/maintainability issue | Medium priority |
| **P3** | Polish/optimization | Low priority |

---

## P0 - Critical Fixes (System Broken)

### P0-1: Config Loading Not Working

**File:** `server/src/main.rs:14`

**Current Code:**
```rust
let config = Settings::default();
```

**Fix:**
```rust
let env = std::env::var("VOICE_AGENT_ENV").ok();
let config = load_settings(env.as_deref()).unwrap_or_else(|e| {
    tracing::error!("Failed to load config: {}. Using defaults.", e);
    Settings::default()
});
```

**Impact:** All YAML config files will be loaded; environment variables will work.

---

### P0-2: Context Token Limit Conflict

**Files:**
- `config/src/constants.rs:117` - `MAX_CONTEXT_TOKENS: 2048`
- `rag/src/context.rs:248` - `max_context_tokens: 32768`

**Fix:** Update `rag/src/context.rs` to use centralized constant:
```rust
use voice_agent_config::constants::rag::MAX_CONTEXT_TOKENS;

// Replace hardcoded 32768 with:
max_context_tokens: MAX_CONTEXT_TOKENS,
```

**Or** if 32768 is correct, update constants.rs:
```rust
pub const MAX_CONTEXT_TOKENS: usize = 32768;
```

**Decision Required:** Which value is correct? 2048 or 32768?

---

## P1 - High Priority Fixes

### P1-1: Eliminate Message/Role Type Duplication

**Files:**
- `llm/src/prompt.rs` (duplicate)
- `core/src/llm_types.rs` (canonical)

**Fix:** In `llm/src/prompt.rs`, replace local definitions with re-exports:

```rust
// DELETE local Message and Role definitions
// ADD re-exports:
pub use voice_agent_core::llm_types::{Message, Role};
```

**Impact:** Ensures consistent Message struct with all fields (name, tool_call_id).

---

### P1-2: Centralize PCM Audio Conversion Constants

**Files to Update:**
- `core/src/audio.rs`
- `transport/src/codec.rs`
- `server/src/websocket.rs`

**Add to `config/src/constants.rs`:**
```rust
pub mod audio {
    /// PCM16 normalization divisor (for to_f32 conversion)
    pub const PCM16_NORMALIZE: f32 = 32768.0;

    /// PCM16 scaling multiplier (for to_pcm16 conversion)
    pub const PCM16_SCALE: f32 = 32767.0;

    /// PCM16 clamp range
    pub const PCM16_MIN: i16 = i16::MIN;
    pub const PCM16_MAX: i16 = i16::MAX;
}
```

**Update all files to use:**
```rust
use voice_agent_config::constants::audio::{PCM16_NORMALIZE, PCM16_SCALE};
```

---

### P1-3: Add Missing WebRTC ICE Timeout Constants

**File:** `transport/src/webrtc.rs:432-434`

**Current (hardcoded):**
```rust
disconnected: Duration::from_secs(5),
failed: Duration::from_secs(25),
keep_alive: Duration::from_secs(2),
```

**Add to `config/src/constants.rs`:**
```rust
pub mod webrtc {
    pub const DISCONNECT_TIMEOUT_SECS: u64 = 5;
    pub const FAILED_TIMEOUT_SECS: u64 = 25;
    pub const KEEPALIVE_INTERVAL_SECS: u64 = 2;
}
```

---

### P1-4: Add Turn Detection Constants

**File:** `pipeline/src/turn_detection/hybrid.rs:68-71`

**Current (hardcoded):**
```rust
base_silence: 500,
min_silence: 200,
max_silence: 1000,
min_speech: 200,
```

**Add to `config/src/constants.rs`:**
```rust
pub mod turn_detection {
    pub const BASE_SILENCE_MS: u64 = 500;
    pub const MIN_SILENCE_MS: u64 = 200;
    pub const MAX_SILENCE_MS: u64 = 1000;
    pub const MIN_SPEECH_MS: u64 = 200;
}
```

---

### P1-5: Add Missing 14K Purity Factor

**File:** `config/src/constants.rs`

**Add:**
```rust
pub mod gold_prices {
    // ... existing
    pub const PURITY_14K: f64 = 0.585;
}
```

---

## P2 - Code Quality Fixes

### P2-1: Consolidate ConversationContext Types

**Files:**
- `core/src/traits/retriever.rs:223` (canonical)
- `rag/src/agentic.rs:55` (duplicate)

**Options:**

**Option A:** Extend core type with optional fields
```rust
// In core/src/traits/retriever.rs
pub struct ConversationContext {
    pub recent_turns: Vec<ConversationTurn>,
    pub summary: Option<String>,  // ADD for RAG
    pub intent: Option<String>,
    pub stage: ConversationStage,
    pub entities: HashMap<String, Value>,
}
```

**Option B:** Keep separate types but rename RAG version
```rust
// In rag/src/agentic.rs
pub struct RagQueryContext {  // RENAME to avoid confusion
    pub summary: String,
    pub stage: Stage,
    pub entities: Vec<(String, String)>,
}
```

**Recommendation:** Option B - keeps separation of concerns clear.

---

### P2-2: Consolidate Document Types

**Files:**
- `core/src/traits/retriever.rs:177` - uses `content` field
- `rag/src/vector_store.rs:67` - uses `text` field

**Fix:** Standardize on `content` field name:

```rust
// In rag/src/vector_store.rs - CHANGE
pub struct Document {
    pub id: String,
    pub content: String,  // was "text"
    // ... rest same
}
```

**Update:** All references from `.text` to `.content`.

---

### P2-3: Extract Hardcoded Interest Rates from Strings

**Files:**
- `llm/src/prompt.rs`
- `agent/src/persuasion.rs`
- `agent/src/agent.rs`

**Pattern Change:**
```rust
// BEFORE (hardcoded in strings):
"Our rate is just 10.5% compared to NBFCs charging 18-24%"

// AFTER (use constants):
use voice_agent_config::constants::interest_rates;

format!(
    "Our rate is just {}% compared to NBFCs charging {}-{}%",
    interest_rates::TIER_2_HEADLINE,
    interest_rates::NBFC_TYPICAL_MIN,
    interest_rates::NBFC_TYPICAL_MAX
)
```

---

### P2-4: LLM Crate Config Cleanup

**Files:**
- `llm/src/backend.rs` - has local `LlmConfig`
- `llm/src/factory.rs` - has local `LlmProviderConfig`
- `llm/src/claude.rs` - has local `ClaudeConfig`

**Fix:** Import from config crate where possible, or clearly document why local definitions are needed.

---

### P2-5: Add VAD Frame Constants

**File:** `pipeline/src/vad/magicnet.rs:59-60`

**Add to `config/src/constants.rs`:**
```rust
pub mod vad {
    pub const MIN_SPEECH_FRAMES: u32 = 25;  // 250ms at 10ms frames
    pub const MIN_SILENCE_FRAMES: u32 = 30; // 300ms
}
```

---

## P3 - Polish/Optimization

### P3-1: Move ToolExecutor Trait to Core

**Current:** `tools/src/registry.rs`
**Move to:** `core/src/traits/tool.rs`

Enables external executor implementations and maintains trait consistency.

---

### P3-2: Document G2P Language Enum Scope

**File:** `pipeline/src/tts/g2p.rs:71`

**Add documentation:**
```rust
/// G2P-specific Language enum
///
/// Note: This is intentionally limited to languages supported by the G2P system.
/// For the full language enum, see `voice_agent_core::Language`.
pub enum G2PLanguage {
    Hindi,
    English,
    Hinglish,
}
```

---

### P3-3: Add Config Validation Tests

**New file:** `config/src/tests/validation.rs`

```rust
#[test]
fn test_constants_match_config_defaults() {
    let settings = Settings::default();

    // Verify interest rates
    assert_eq!(settings.gold_loan.tier1_rate,
               constants::interest_rates::TIER_1_STANDARD);

    // Verify timeouts
    assert_eq!(settings.server.timeout_secs,
               constants::timeouts::TOOL_DEFAULT_MS / 1000);

    // Verify audio constants
    assert_eq!(settings.pipeline.audio.sample_rate,
               constants::audio::SAMPLE_RATE);
}
```

---

## Implementation Checklist

### Phase 1: Critical Fixes (P0)
- [ ] Fix config loading in main.rs
- [ ] Resolve context token limit conflict

### Phase 2: High Priority (P1)
- [ ] Eliminate Message/Role duplication
- [ ] Centralize PCM audio constants
- [ ] Add WebRTC ICE timeout constants
- [ ] Add turn detection constants
- [ ] Add 14K purity factor

### Phase 3: Code Quality (P2)
- [ ] Consolidate ConversationContext types
- [ ] Consolidate Document types
- [ ] Extract hardcoded interest rates
- [ ] Clean up LLM crate configs
- [ ] Add VAD frame constants

### Phase 4: Polish (P3)
- [ ] Move ToolExecutor to core
- [ ] Document G2P Language enum
- [ ] Add validation tests

---

## Files Changed Summary

| Phase | Files Modified | Files Added |
|-------|---------------|-------------|
| P0 | 2 | 0 |
| P1 | 8 | 0 |
| P2 | 12 | 0 |
| P3 | 3 | 1 |
| **Total** | **25** | **1** |

---

## Risk Assessment

| Fix | Risk Level | Mitigation |
|-----|------------|------------|
| P0-1 Config Loading | LOW | Fallback to defaults exists |
| P0-2 Token Limit | MEDIUM | Test RAG performance after change |
| P1-1 Message Type | MEDIUM | Ensure all callers handle new fields |
| P2-1 ConversationContext | HIGH | Requires adapter updates |
| P2-2 Document Type | MEDIUM | Search-replace with testing |

---

## Testing Strategy

1. **After P0 fixes:**
   - Verify config files load correctly
   - Test environment variable overrides
   - Verify domain config still works

2. **After P1 fixes:**
   - Run full test suite
   - Verify LLM message handling
   - Test audio processing pipeline

3. **After P2 fixes:**
   - Integration tests for RAG
   - End-to-end conversation tests
   - Performance benchmarks

4. **After P3 fixes:**
   - Documentation review
   - Code coverage check
