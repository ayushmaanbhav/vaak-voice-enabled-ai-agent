# Dead Code Analysis - Voice Agent Rust

## Summary

Total dead/unused code identified: **~2,000 lines**

| Category | Lines | Severity | Action |
|----------|-------|----------|--------|
| Unused Transport Crate | ~1,500 | HIGH | Integrate or remove |
| Disconnected Processors | ~500 | MEDIUM | Wire or document |
| Intentionally Dead (allow markers) | ~100 | LOW | Keep (documented) |
| Deprecated Functions | ~30 | LOW | Remove |

---

## 1. Transport Crate - ENTIRELY UNUSED (~1,500 lines)

### Status: CRITICAL

The entire transport crate is disconnected from the server:

| File | Lines | Purpose | Status |
|------|-------|---------|--------|
| `transport/src/webrtc.rs` | 843 | Full WebRTC with ICE/DTLS | UNUSED |
| `transport/src/session.rs` | 288 | Transport session manager | UNUSED |
| `transport/src/codec.rs` | 374 | Opus encoder/decoder | UNUSED |
| `transport/src/websocket.rs` | ~50 | WebSocket stub | UNUSED (stub) |

### Why It's Dead:
```toml
# server/Cargo.toml - does NOT include:
voice-agent-transport = { path = "../transport" }
```

### Recommendation:
**Option A:** Integrate - Add WebRTC signaling endpoints to server
**Option B:** Remove - Delete transport crate (significant effort lost)

---

## 2. Frame Processors - DISCONNECTED (~500 lines)

### Status: MEDIUM (Exists but not wired)

| File | Lines | Purpose | Integration |
|------|-------|---------|-------------|
| `pipeline/src/processors/chain.rs` | 515 | Processor orchestration | NOT CALLED |
| `pipeline/src/processors/sentence_detector.rs` | 490 | LLM→Sentence | NOT CALLED |
| `pipeline/src/processors/tts_processor.rs` | 400 | Sentence→TTS | NOT CALLED |
| `pipeline/src/processors/interrupt_handler.rs` | 540 | Barge-in handling | NOT CALLED |

### Why It's "Dead":
The orchestrator uses a traditional state machine instead of the frame-based pipeline. These processors are well-implemented but never instantiated.

### Evidence:
```rust
// orchestrator.rs - NO processor chain usage:
pub struct VoicePipeline {
    vad: Arc<VoiceActivityDetector>,
    turn_detector: Arc<HybridTurnDetector>,
    stt: Arc<Mutex<StreamingStt>>,
    tts: Arc<StreamingTts>,
    // NO: processor_chain: ProcessorChain
}
```

### Recommendation:
Either:
1. Wire processors into orchestrator
2. Document as "advanced usage" framework
3. Remove if truly not needed

---

## 3. Intentionally Dead Code (Allowed)

These are marked with `#[allow(dead_code)]` and are intentional:

### RAG Crate
```rust
// reranker.rs:579 - Early exit strategy (ONNX limitation)
#[allow(dead_code)]
fn should_exit(&self, layer_output: &LayerOutput, config: &ExitConfig) -> bool {
    // ~55 lines - Never called because ONNX doesn't support per-layer output
}
```
**Reason:** Documented limitation - kept for potential future Candle implementation.

### LLM Crate
```rust
// backend.rs:905-920 - OpenAI response fields
#[allow(dead_code)]
struct OpenAIUsage {
    prompt_tokens: usize,
    total_tokens: usize,
}
```
**Reason:** Required for API response deserialization schema.

### Transport Crate
```rust
// websocket.rs:50 - Stub implementation
#[allow(dead_code)]
pub struct WebSocketTransport { ... }
```
**Reason:** Real WebSocket is in server crate; this is interface placeholder.

### Persistence Crate
```rust
// Various timestamp fields
#[allow(dead_code)]
timestamp: std::time::Instant,
```
**Reason:** Prepared for future TTL-based cache eviction.

---

## 4. Deprecated Functions (~30 lines)

### Intent Detection
```rust
// intent.rs:872-882
#[deprecated(note = "Use extract_slots() which uses compiled regex patterns")]
fn extract_slot_value(&self, text: &str, slot_name: &str) -> Option<String>
```
**Action:** Remove in next cleanup cycle.

### Old Extract Methods
```rust
// intent.rs:885-915
#[allow(dead_code)]
fn extract_number_before(text: &str) -> Option<f64>
```
**Action:** Remove (replaced by compiled patterns).

---

## 5. Partially Implemented Features

These aren't dead code but are incomplete:

### gRPC Translator Stub
```rust
// text_processing/translation/grpc.rs:125-143
async fn call_service(&self, text: &str, ...) -> Result<String> {
    // TODO: Implement actual HTTP client call
    Ok(text.to_string())  // Returns original!
}
```
**Status:** Stubbed, returns original text.
**Impact:** Falls back gracefully to Candle translator.

### AudioProcessor Trait
```rust
// core/traits/speech.rs
#[async_trait]
pub trait AudioProcessor: Send + Sync {
    async fn process(&self, frame: &mut AudioFrame) -> Result<AudioFrame, Error>;
}
```
**Status:** Trait defined, no implementations.
**Impact:** Echo cancellation/noise suppression flags are placeholders.

---

## Code Quality Metrics

### Dead Code by Crate

| Crate | Dead Lines | % of Crate | Notes |
|-------|------------|------------|-------|
| transport | ~1,500 | 100% | Entire crate unused |
| pipeline/processors | ~500 | ~30% | Framework exists, not wired |
| rag | ~55 | ~2% | should_exit() dead |
| intent | ~30 | ~2% | Deprecated methods |
| Others | ~0 | 0% | Clean |

### Cleanup Priority

1. **HIGH:** Decide on transport crate (integrate or remove)
2. **MEDIUM:** Document processor framework (or wire it)
3. **LOW:** Remove deprecated intent methods
4. **KEEP:** Allowed dead code (intentional, documented)

---

## Recommendations

### Immediate Actions
1. Add comment in transport crate explaining status
2. Add comment in orchestrator explaining why processors not used
3. Remove deprecated extract_slot_value()

### Short-term (If Integrating Transport)
1. Add WebRTC endpoints to server
2. Add transport crate dependency
3. Wire signaling flow

### Long-term (If Removing Transport)
1. Archive WebRTC code in separate branch
2. Remove transport crate from workspace
3. Update documentation

---

## How to Find Dead Code

```bash
# Find allow(dead_code) markers:
grep -rn "#\[allow(dead_code)\]" crates/

# Find unused warnings during build:
cargo build --workspace 2>&1 | grep "warning.*never used"

# Use cargo-udeps for dependency analysis:
cargo +nightly udeps --workspace
```
