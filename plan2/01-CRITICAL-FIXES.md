# P0 Critical Fixes - Must Address Before Production

> **Priority:** BLOCKING
> **Estimated Effort:** 2 weeks
> **Impact:** Ship blockers

---

## 1. ONNX Runtime Pre-Release Dependency

**Location:** `Cargo.toml:46`
**Severity:** CRITICAL
**Effort:** 2 hours

### Problem
```toml
ort = { version = "2.0.0-rc.9" }  # Pre-release, unstable API
```

### Risk
- Unresolved bugs and API instability
- May break in production with minimal notice
- Cannot be pinned precisely for reproducible builds
- Affects VAD, STT, TTS, reranker, embeddings

### Solution
```toml
# Option 1: Pin to stable release when available
ort = { version = "2.0.0" }

# Option 2: If stable not available, document risk
# Create fallback tests that verify ONNX behavior
```

### Validation
```bash
cargo audit
cargo test --features onnx
```

---

## 2. Redis Session Persistence (Stubbed)

**Location:** `crates/server/src/session.rs:73-127`
**Severity:** CRITICAL
**Effort:** 8 hours

### Problem
```rust
impl SessionStore for RedisSessionStore {
    async fn store_metadata(&self, session: &Session) -> Result<(), ServerError> {
        // TODO: Implement Redis SET with TTL
        Ok(())  // <-- NOT ACTUALLY PERSISTING
    }
}
```

### Risk
- All sessions lost on server restart
- No horizontal scaling possible
- Customer conversations disappear mid-call
- No recovery after crashes

### Solution
```rust
use deadpool_redis::{redis::AsyncCommands, Pool};

impl SessionStore for RedisSessionStore {
    async fn store_metadata(&self, session: &Session) -> Result<(), ServerError> {
        let mut conn = self.pool.get().await?;
        let key = format!("session:{}", session.id);
        let value = serde_json::to_string(&session.metadata)?;
        conn.set_ex(&key, value, self.ttl_secs).await?;
        Ok(())
    }

    async fn get_metadata(&self, session_id: &str) -> Result<Option<SessionMetadata>, ServerError> {
        let mut conn = self.pool.get().await?;
        let key = format!("session:{}", session_id);
        let value: Option<String> = conn.get(&key).await?;
        value.map(|v| serde_json::from_str(&v)).transpose()
    }
}
```

### Dependencies to Add
```toml
[dependencies]
deadpool-redis = "0.13"
```

---

## 3. Hindi/Devanagari Slot Extraction Missing

**Location:** `crates/agent/src/intent.rs:231-346`
**Severity:** CRITICAL
**Effort:** 16 hours

### Problem
Slot extraction patterns only have ASCII regex:
```rust
// Current - FAILS on Hindi input
CompiledSlotPattern {
    name: "lakh".to_string(),
    regex: Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(?:lakh|lac|lakhs)").unwrap(),
    // Won't match "लाख" (Devanagari)
}
```

### Hindi Input Failure
```
User: "पांच लाख रुपये का लोन चाहिए"  (5 lakh loan needed)
Expected: loan_amount = 500000
Actual: loan_amount = None → Tool call fails
```

### Solution
```rust
// Add Devanagari numeral patterns
lazy_static! {
    static ref DEVANAGARI_DIGITS: HashMap<char, u32> = [
        ('०', 0), ('१', 1), ('२', 2), ('३', 3), ('४', 4),
        ('५', 5), ('६', 6), ('७', 7), ('८', 8), ('९', 9),
    ].into_iter().collect();

    static ref HINDI_MULTIPLIERS: HashMap<&'static str, f64> = [
        ("लाख", 100_000.0),
        ("करोड़", 10_000_000.0),
        ("हज़ार", 1_000.0),
        ("lakh", 100_000.0),
        ("crore", 10_000_000.0),
    ].into_iter().collect();
}

fn extract_hindi_amount(text: &str) -> Option<f64> {
    // 1. Convert Devanagari digits to ASCII
    // 2. Match Hindi multiplier words
    // 3. Return computed amount
}
```

---

## 4. Gold Price Hardcoded (Stale Immediately)

**Location:** `crates/config/src/gold_loan.rs:96-98`
**Severity:** CRITICAL
**Effort:** 4 hours

### Problem
```rust
fn default_gold_price() -> f64 {
    7500.0 // INR per gram (stale within days!)
}
```

### Risk
- Gold price changes daily (sometimes 2-3x/day)
- All eligibility calculations incorrect
- Customer may be over/under-quoted on loan amount
- Competitor savings calculations wrong

### Solution
```rust
// New file: crates/tools/src/gold_price.rs
pub struct GoldPriceService {
    client: reqwest::Client,
    cache: RwLock<Option<(f64, Instant)>>,
    cache_ttl: Duration,
}

impl GoldPriceService {
    pub async fn get_price_per_gram(&self) -> Result<f64, ToolError> {
        // Check cache first
        if let Some((price, cached_at)) = self.cache.read().as_ref() {
            if cached_at.elapsed() < self.cache_ttl {
                return Ok(*price);
            }
        }

        // Fetch from API (MCX, GoldAPI.io, or Metals-API)
        let response = self.client
            .get("https://api.goldapi.io/v1/XAU/INR")
            .header("x-access-token", &self.api_key)
            .send()
            .await?;

        let data: GoldPriceResponse = response.json().await?;
        let price_per_gram = data.price / 31.1035; // Troy oz to gram

        // Update cache
        *self.cache.write() = Some((price_per_gram, Instant::now()));

        Ok(price_per_gram)
    }
}
```

---

## 5. SMS Confirmation False Positive

**Location:** `crates/tools/src/gold_loan.rs:493-496`
**Severity:** CRITICAL
**Effort:** 2 hours (stub) / 8 hours (real SMS)

### Problem
```rust
let result = json!({
    "confirmation_sent": true,  // CLAIMED but NOT sent!
    "message": format!(
        "...Confirmation SMS sent to {}.",
        phone
    )
});
```

### Risk
- Tells customer "SMS sent" but doesn't send
- Customer thinks appointment is booked but has no record
- Loss of trust, failed conversions

### Solution (Immediate Stub)
```rust
let result = json!({
    "confirmation_sent": false,  // Be honest
    "confirmation_pending": true,
    "message": format!(
        "Appointment scheduled for {} on {} at {}. You will receive confirmation shortly.",
        name, date, time
    )
});
```

### Solution (Real SMS via Twilio/MSG91)
```rust
pub async fn send_sms(phone: &str, message: &str) -> Result<bool, ToolError> {
    let client = TwilioClient::new(&config.account_sid, &config.auth_token);
    client.send_message(
        &config.from_number,
        phone,
        message
    ).await
    .map(|_| true)
    .map_err(|e| ToolError::Integration(e.to_string()))
}
```

---

## 6. Race Conditions in Pipeline Audio Buffer

**Location:** `crates/pipeline/src/vad/silero.rs:163-189`
**Severity:** CRITICAL
**Effort:** 4 hours

### Problem
```rust
let mut state = self.mutable.lock();
state.audio_buffer.extend_from_slice(&frame.samples);

if state.audio_buffer.len() >= self.config.chunk_size {
    let chunk: Vec<f32> = state.audio_buffer.drain(...).collect();
    drop(state);  // Release lock
    let speech_prob = self.compute_probability(&chunk)?;  // Re-acquire lock
    let mut state = self.mutable.lock();  // <-- RACE WINDOW
}
```

### Risk
Between `drop(state)` and re-acquiring, another thread could:
- Modify audio_buffer
- Cause data corruption
- Lead to incorrect VAD decisions or panics

### Solution
Don't release lock mid-processing:
```rust
fn process_frame(&self, frame: &mut AudioFrame) -> Result<(VadState, f32), PipelineError> {
    let mut state = self.mutable.lock();
    state.audio_buffer.extend_from_slice(&frame.samples);

    if state.audio_buffer.len() >= self.config.chunk_size {
        let chunk: Vec<f32> = state.audio_buffer.drain(..).collect();

        // Compute within lock scope using pre-allocated buffers
        let speech_prob = self.compute_probability_inner(&mut state, &chunk)?;

        self.update_state_inner(&mut state, speech_prob)?;
    }

    Ok((state.current_state, state.last_probability))
}
```

---

## 7. Streaming STT Not Actually Streaming

**Location:** `crates/pipeline/src/stt/indicconformer.rs:564-614`
**Severity:** HIGH (Latency)
**Effort:** 16 hours

### Problem
```rust
// Computes FULL mel spectrogram every chunk
fn extract_mel_spectrogram(&self, audio: &[f32]) -> Result<Array2<f32>, PipelineError> {
    // Manual DFT computation - O(n²) complexity
    // Recomputes all frames from scratch
}
```

### Impact
- Not true streaming (batched processing)
- 10-20x slower than FFT for 512-point transforms
- Latency budget of 500ms exceeded

### Solution
```rust
use rustfft::{FftPlanner, Fft};

struct StreamingMelExtractor {
    fft: Arc<dyn Fft<f32>>,
    mel_filterbank: Array2<f32>,
    overlap_buffer: Vec<f32>,
    hop_length: usize,
}

impl StreamingMelExtractor {
    fn process_chunk(&mut self, chunk: &[f32]) -> Vec<Array1<f32>> {
        // 1. Append to overlap buffer
        self.overlap_buffer.extend_from_slice(chunk);

        // 2. Process only new frames (sliding window)
        let mut mel_frames = Vec::new();
        while self.overlap_buffer.len() >= self.fft_size {
            let frame = &self.overlap_buffer[..self.fft_size];
            let mel = self.compute_mel_frame(frame);
            mel_frames.push(mel);

            // 3. Advance by hop_length (not full frame)
            self.overlap_buffer.drain(..self.hop_length);
        }

        mel_frames
    }
}
```

---

## 8. Tiered Rates Not Applied in Eligibility

**Location:** `crates/tools/src/gold_loan.rs:177-214`
**Severity:** HIGH (Business Logic)
**Effort:** 2 hours

### Problem
```rust
let result = json!({
    "interest_rate_percent": self.config.kotak_interest_rate,  // 10.5% always!
});
```

But tiered rates exist in config:
- <1L: 11.5%
- 1-5L: 10.5%
- >5L: 9.5%

### Solution
```rust
let max_loan = self.config.calculate_max_loan(gold_value);
let tiered_rate = self.config.get_tiered_rate(max_loan);  // Use this!

let result = json!({
    "interest_rate_percent": tiered_rate,
    // ...
});
```

---

## Summary: P0 Critical Fixes

| # | Issue | Effort | Owner | Status |
|---|-------|--------|-------|--------|
| 1 | ONNX pre-release | 2h | Infra | [ ] |
| 2 | Redis session persistence | 8h | Backend | [ ] |
| 3 | Hindi slot extraction | 16h | NLP | [ ] |
| 4 | Gold price API | 4h | Backend | [ ] |
| 5 | SMS false positive | 2h | Backend | [ ] |
| 6 | Pipeline race conditions | 4h | Pipeline | [ ] |
| 7 | True streaming STT | 16h | Pipeline | [ ] |
| 8 | Tiered rates in eligibility | 2h | Business | [ ] |

**Total Effort: ~54 hours (2 dev weeks)**

---

*Next: See 02-HIGH-PRIORITY-FIXES.md for P1 issues*
