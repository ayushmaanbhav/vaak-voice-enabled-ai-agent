# Phase 4: Performance & Safety

**Priority:** P2 - Production readiness
**Estimated Files:** 20 files
**Dependencies:** None (can run in parallel with Phase 3)

---

## Overview

This phase addresses:
1. Fix concurrency issues and race conditions
2. Replace dangerous unwrap() calls with proper error handling
3. Optimize performance bottlenecks
4. Add proper resource cleanup

---

## Task 4.1: Fix WebSocket Concurrency Issues

### Problem
`server/src/websocket.rs` has multiple concurrency problems:
- 25+ `.lock().await` calls causing contention
- No graceful shutdown for spawned tasks
- No backpressure handling
- 19 instances of `unwrap()` on serialization

---

#### 4.1.1 Replace unwrap() with Proper Error Handling

**File:** `crates/server/src/websocket.rs`

**Find all JSON serialization unwraps:**
```bash
grep -n "serde_json::to_string.*unwrap()" crates/server/src/websocket.rs
```

**Replace pattern (19 instances):**

**Before:**
```rust
let json = serde_json::to_string(&msg).unwrap();
sender.lock().await.send(Message::Text(json)).await;
```

**After:**
```rust
match serde_json::to_string(&msg) {
    Ok(json) => {
        if let Err(e) = sender.lock().await.send(Message::Text(json)).await {
            tracing::warn!("Failed to send WebSocket message: {}", e);
            break;
        }
    }
    Err(e) => {
        tracing::error!("Failed to serialize message: {}", e);
        // Send error response to client
        let error_msg = json!({"error": "Internal serialization error"});
        let _ = sender.lock().await
            .send(Message::Text(error_msg.to_string()))
            .await;
    }
}
```

**Create helper function:**
```rust
async fn send_json<T: Serialize>(
    sender: &Arc<Mutex<SplitSink<WebSocket, Message>>>,
    msg: &T,
) -> Result<(), WebSocketError> {
    let json = serde_json::to_string(msg)
        .map_err(|e| WebSocketError::Serialization(e.to_string()))?;

    sender.lock().await
        .send(Message::Text(json))
        .await
        .map_err(|e| WebSocketError::Send(e.to_string()))
}
```

#### 4.1.2 Add Graceful Task Shutdown

**Current (lines 723-727):**
```rust
audio_task.abort();
event_task.abort();
if let Some(task) = pipeline_event_task {
    task.abort();
}
```

**Replace with graceful shutdown:**
```rust
use tokio::sync::oneshot;
use tokio::time::{timeout, Duration};

// At task spawn, create shutdown channels
let (audio_shutdown_tx, audio_shutdown_rx) = oneshot::channel::<()>();
let (event_shutdown_tx, event_shutdown_rx) = oneshot::channel::<()>();

// In audio task
let audio_task = tokio::spawn(async move {
    tokio::select! {
        _ = audio_shutdown_rx => {
            tracing::info!("Audio task received shutdown signal");
        }
        result = audio_processing_loop() => {
            if let Err(e) = result {
                tracing::error!("Audio task error: {}", e);
            }
        }
    }
});

// Graceful shutdown with timeout
async fn shutdown_tasks(
    audio_task: JoinHandle<()>,
    event_task: JoinHandle<()>,
    audio_shutdown_tx: oneshot::Sender<()>,
    event_shutdown_tx: oneshot::Sender<()>,
) {
    // Signal shutdown
    let _ = audio_shutdown_tx.send(());
    let _ = event_shutdown_tx.send(());

    // Wait with timeout
    let shutdown_timeout = Duration::from_secs(5);

    if timeout(shutdown_timeout, audio_task).await.is_err() {
        tracing::warn!("Audio task did not shutdown gracefully, aborting");
    }

    if timeout(shutdown_timeout, event_task).await.is_err() {
        tracing::warn!("Event task did not shutdown gracefully, aborting");
    }
}
```

#### 4.1.3 Add Backpressure to Audio Channel

**Current (line 137):**
```rust
let (audio_tx, mut audio_rx) = mpsc::channel::<Vec<u8>>(100);
```

**Replace with backpressure handling:**
```rust
use tokio::sync::mpsc;

const AUDIO_CHANNEL_CAPACITY: usize = 100;
const AUDIO_CHANNEL_HIGH_WATERMARK: usize = 80;

let (audio_tx, mut audio_rx) = mpsc::channel::<Vec<u8>>(AUDIO_CHANNEL_CAPACITY);

// Wrapper that handles backpressure
struct AudioSender {
    inner: mpsc::Sender<Vec<u8>>,
    dropped_frames: AtomicUsize,
}

impl AudioSender {
    async fn send(&self, data: Vec<u8>) -> Result<(), AudioError> {
        // Check capacity before sending
        if self.inner.capacity() < (AUDIO_CHANNEL_CAPACITY - AUDIO_CHANNEL_HIGH_WATERMARK) {
            // Channel is getting full - log warning
            tracing::warn!(
                "Audio channel backpressure: {} frames buffered, dropped {} total",
                AUDIO_CHANNEL_CAPACITY - self.inner.capacity(),
                self.dropped_frames.load(Ordering::Relaxed)
            );
        }

        // Try to send without blocking
        match self.inner.try_send(data) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.dropped_frames.fetch_add(1, Ordering::Relaxed);
                Err(AudioError::ChannelFull)
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                Err(AudioError::ChannelClosed)
            }
        }
    }
}
```

#### 4.1.4 Reduce Lock Contention

**Problem:** Multiple locks acquired in sequence in hot paths.

**Current pattern:**
```rust
// Acquiring 3 locks in sequence
let rate_limiter = rate_limiter.lock().await;
let pipeline = pipeline.lock().await;
let sender = sender.lock().await;
```

**Refactor to single state struct:**
```rust
struct WebSocketState {
    pipeline: AudioPipeline,
    rate_limiter: RateLimiter,
    // Other mutable state
}

// Single lock for all state
let state = Arc::new(RwLock::new(WebSocketState::new()));

// Read-heavy operations use read lock
let state = state.read().await;

// Write operations use write lock (less frequent)
let mut state = state.write().await;
```

**For sender, use dedicated send task:**
```rust
// Create dedicated sender task
let (send_tx, mut send_rx) = mpsc::channel::<Message>(100);

let sender_task = tokio::spawn(async move {
    while let Some(msg) = send_rx.recv().await {
        if sender.send(msg).await.is_err() {
            break;
        }
    }
});

// Other tasks just send to channel (no lock needed)
send_tx.send(Message::Text(json)).await?;
```

---

## Task 4.2: Fix ptt.rs Regex Unwraps

**File:** `crates/server/src/ptt.rs`

### Problem
Lines 30-38 have 8 regex patterns using `unwrap()` at static initialization.

**Current:**
```rust
static RE_HEADERS: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?m)^#{1,6}\s*").unwrap()
});
```

### Solution
Use compile-time regex validation or handle errors gracefully.

**Option A: Use regex literal (compile-time validation)**
```rust
use regex::Regex;

// These patterns are known-good at compile time
static RE_HEADERS: Lazy<Regex> = Lazy::new(|| {
    // Pattern is validated - unwrap is safe
    Regex::new(r"(?m)^#{1,6}\s*").expect("RE_HEADERS is invalid regex")
});
```

**Option B: Validate at startup**
```rust
/// Validate all regex patterns at startup
pub fn validate_regex_patterns() -> Result<(), RegexError> {
    // Force initialization of all lazy patterns
    let _ = &*RE_HEADERS;
    let _ = &*RE_BOLD_ASTERISK;
    let _ = &*RE_BOLD_UNDERSCORE;
    // ... etc
    Ok(())
}

// Call in main.rs
if let Err(e) = ptt::validate_regex_patterns() {
    tracing::error!("Invalid regex pattern: {}", e);
    std::process::exit(1);
}
```

---

## Task 4.3: Fix TranscriptAccumulator Thread Safety

**File:** `crates/core/src/transcript.rs`

### Problem
`TranscriptAccumulator` (lines 135-261) has mutable state but no synchronization.

**Current:**
```rust
pub struct TranscriptAccumulator {
    stable_text: String,
    unstable_text: String,
    words: Vec<WordInfo>,
    // ... mutable fields
}
```

### Solution A: Make it explicitly single-threaded
```rust
/// Transcript accumulator - NOT thread-safe
///
/// This struct must only be accessed from a single thread.
/// For concurrent access, wrap in `Arc<Mutex<TranscriptAccumulator>>`.
#[derive(Debug)]
pub struct TranscriptAccumulator {
    // ... fields
}

impl !Sync for TranscriptAccumulator {}
impl !Send for TranscriptAccumulator {}
```

### Solution B: Add internal synchronization
```rust
use parking_lot::RwLock;

/// Thread-safe transcript accumulator
pub struct TranscriptAccumulator {
    inner: RwLock<TranscriptAccumulatorInner>,
}

struct TranscriptAccumulatorInner {
    stable_text: String,
    unstable_text: String,
    words: Vec<WordInfo>,
}

impl TranscriptAccumulator {
    pub fn process(&self, result: &TranscriptResult) -> Option<String> {
        let mut inner = self.inner.write();
        // ... existing logic
    }

    pub fn stable_text(&self) -> String {
        self.inner.read().stable_text.clone()
    }
}
```

---

## Task 4.4: Optimize Token Estimation

**File:** `crates/llm/src/prompt.rs`

### Problem
Lines 670-691: Token estimation recomputes for every call without caching.

**Current:**
```rust
pub fn estimate_tokens(&self) -> usize {
    self.messages.iter()
        .map(|m| {
            let grapheme_count = m.content.graphemes(true).count();
            let devanagari_count = m.content.chars()
                .filter(|c| ('\u{0900}'..='\u{097F}').contains(c))
                .count();
            // ... expensive computation
        })
        .sum()
}
```

### Solution: Add caching
```rust
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MessageHistory {
    messages: Vec<Message>,
    cached_token_count: AtomicUsize,
    cache_valid: AtomicBool,
}

impl MessageHistory {
    pub fn add_message(&mut self, msg: Message) {
        self.messages.push(msg);
        self.cache_valid.store(false, Ordering::Release);
    }

    pub fn estimate_tokens(&self) -> usize {
        // Check cache first
        if self.cache_valid.load(Ordering::Acquire) {
            return self.cached_token_count.load(Ordering::Relaxed);
        }

        // Compute
        let count = self.messages.iter()
            .map(|m| estimate_message_tokens(m))
            .sum();

        // Update cache
        self.cached_token_count.store(count, Ordering::Relaxed);
        self.cache_valid.store(true, Ordering::Release);

        count
    }
}

fn estimate_message_tokens(msg: &Message) -> usize {
    use unicode_segmentation::UnicodeSegmentation;

    let grapheme_count = msg.content.graphemes(true).count();
    let devanagari_count = msg.content.chars()
        .filter(|c| ('\u{0900}'..='\u{097F}').contains(c))
        .count();

    // Devanagari uses ~2 tokens per grapheme, Latin ~0.25
    let devanagari_tokens = devanagari_count * 2;
    let latin_tokens = (grapheme_count - devanagari_count) / 4;

    devanagari_tokens + latin_tokens + 4  // +4 for message overhead
}
```

---

## Task 4.5: Optimize Message Cloning in Speculative Execution

**File:** `crates/llm/src/speculative.rs`

### Problem
Lines 203-207: Messages cloned for speculative execution.

**Current:**
```rust
let messages_for_llm = messages.to_vec();  // Expensive clone
```

### Solution: Use Arc
```rust
use std::sync::Arc;

pub struct SpeculativeExecutor {
    // ... fields
}

impl SpeculativeExecutor {
    pub async fn execute(
        &self,
        messages: Arc<Vec<Message>>,  // Changed from &[Message]
    ) -> Result<Response, LlmError> {
        let messages_clone = Arc::clone(&messages);  // Cheap clone

        let llm_handle = tokio::spawn(async move {
            self.llm.generate(&messages_clone).await
        });

        // ...
    }
}
```

---

## Task 4.6: Add Timeout to Agent Processing

**File:** `crates/server/src/websocket.rs`

### Problem
Line 338: No timeout on `process_stream()`.

**Current:**
```rust
match session.agent.process_stream(&processed_input).await {
    // Could block indefinitely
}
```

### Solution: Add timeout
```rust
use tokio::time::{timeout, Duration};

const AGENT_PROCESSING_TIMEOUT: Duration = Duration::from_secs(30);

match timeout(AGENT_PROCESSING_TIMEOUT, session.agent.process_stream(&processed_input)).await {
    Ok(Ok(response)) => {
        // Normal processing
    }
    Ok(Err(e)) => {
        tracing::error!("Agent processing error: {}", e);
        send_error_response(&sender, "Processing error").await;
    }
    Err(_) => {
        tracing::error!("Agent processing timed out after {:?}", AGENT_PROCESSING_TIMEOUT);
        send_error_response(&sender, "Request timed out").await;
    }
}
```

---

## Task 4.7: Fix Double Mutex Wrapping in STT

**File:** `crates/pipeline/src/stt/mod.rs`

### Problem
Lines 203-204: STT backend has double mutex wrapping.

**Current:**
```rust
let backend = IndicConformerBackend::new(path, config)?;
Ok(Arc::new(parking_lot::Mutex::new(backend)))  // Double wrapping if backend has internal mutex
```

### Solution: Audit and remove unnecessary wrapping
```rust
// Check if IndicConformerBackend already has internal synchronization
impl IndicConformerBackend {
    // If methods take &self (not &mut self), no external mutex needed
    pub fn transcribe(&self, audio: &[f32]) -> Result<String, Error> {
        // Internal state protected by internal locks
    }
}

// Factory should return appropriate wrapper
pub fn create_stt_backend(...) -> Result<Arc<dyn SttBackend>, PipelineError> {
    let backend = IndicConformerBackend::new(path, config)?;

    // If backend is already Sync+Send, just wrap in Arc
    Ok(Arc::new(backend))

    // Only add Mutex if backend needs external synchronization
    // Ok(Arc::new(parking_lot::Mutex::new(backend)))
}
```

---

## Task 4.8: Add Proper Resource Cleanup

### 4.8.1 Session Cleanup on WebSocket Close

**File:** `crates/server/src/websocket.rs`

**Add cleanup handler:**
```rust
// At end of handle_socket()
async fn cleanup_session(
    session_id: &str,
    state: &AppState,
) {
    tracing::info!("Cleaning up session {}", session_id);

    // Persist session state if enabled
    if let Some(session) = state.sessions.get(session_id) {
        if let Err(e) = state.sessions.persist_session(&session).await {
            tracing::error!("Failed to persist session on close: {}", e);
        }
    }

    // Update session as inactive
    if let Some(mut metadata) = state.sessions.get_metadata(session_id).await.ok().flatten() {
        metadata.active = false;
        // Store updated metadata
    }

    // Log session summary
    if let Some(session) = state.sessions.get(session_id) {
        tracing::info!(
            "Session {} closed: {} turns, stage: {}",
            session_id,
            session.turn_count(),
            session.stage()
        );
    }
}
```

### 4.8.2 Pipeline Resource Cleanup

**File:** `crates/pipeline/src/orchestrator.rs`

**Add Drop implementation:**
```rust
impl Drop for AudioPipeline {
    fn drop(&mut self) {
        tracing::debug!("Dropping AudioPipeline, cleaning up resources");

        // Stop any running tasks
        if let Some(vad_task) = self.vad_task.take() {
            vad_task.abort();
        }

        // Release model handles
        // (ONNX sessions will be dropped automatically)

        tracing::debug!("AudioPipeline cleanup complete");
    }
}
```

---

## Task 4.9: Add Health Metrics

**File:** `crates/server/src/metrics.rs`

**Add performance metrics:**
```rust
use metrics::{counter, gauge, histogram};

pub fn record_websocket_connection() {
    counter!("websocket_connections_total").increment(1);
    gauge!("websocket_connections_active").increment(1.0);
}

pub fn record_websocket_disconnection() {
    gauge!("websocket_connections_active").decrement(1.0);
}

pub fn record_audio_frame_processed(duration_ms: f64) {
    histogram!("audio_frame_processing_ms").record(duration_ms);
}

pub fn record_llm_latency(duration_ms: f64) {
    histogram!("llm_response_latency_ms").record(duration_ms);
}

pub fn record_dropped_audio_frames(count: u64) {
    counter!("audio_frames_dropped_total").increment(count);
}

pub fn record_agent_processing_timeout() {
    counter!("agent_processing_timeouts_total").increment(1);
}
```

---

## Phase 4 Completion Checklist

- [ ] 4.1.1 All JSON serialization unwraps replaced with error handling
- [ ] 4.1.2 Graceful task shutdown implemented
- [ ] 4.1.3 Audio channel backpressure handling added
- [ ] 4.1.4 Lock contention reduced in WebSocket handler
- [ ] 4.2 ptt.rs regex patterns validated/handled
- [ ] 4.3 TranscriptAccumulator thread safety documented/fixed
- [ ] 4.4 Token estimation caching implemented
- [ ] 4.5 Message cloning optimized with Arc
- [ ] 4.6 Agent processing timeout added
- [ ] 4.7 Double mutex wrapping fixed
- [ ] 4.8.1 Session cleanup implemented
- [ ] 4.8.2 Pipeline resource cleanup implemented
- [ ] 4.9 Health metrics added

### Verification Commands
```bash
# Check for remaining unwraps in critical paths
grep -rn "\.unwrap()" crates/server/src/websocket.rs

# Check for potential deadlocks (multiple locks in same function)
grep -rn "\.lock()\.await" crates/server/src/websocket.rs | wc -l

# Run stress test
cargo test --release websocket_stress_test

# Check metrics endpoint
curl http://localhost:3000/metrics
```

---

## Performance Benchmarks

After Phase 4, measure:

| Metric | Target | How to Measure |
|--------|--------|----------------|
| WebSocket latency p99 | <50ms | Prometheus histogram |
| Audio frame drop rate | <0.1% | Counter metric |
| LLM response p99 | <5s | Prometheus histogram |
| Memory per session | <50MB | Process metrics |
| Concurrent sessions | >100 | Load test |

```bash
# Run load test
cargo run --release --bin load_test -- --sessions 100 --duration 60s
```
