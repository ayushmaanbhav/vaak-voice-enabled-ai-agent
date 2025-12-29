# P1 High Priority Fixes - Pre-Launch Required

> **Priority:** HIGH
> **Estimated Effort:** 2 weeks
> **Impact:** User experience, reliability

---

## 1. SlmFirst Latency Cliffs

**Location:** `crates/llm/src/speculative.rs:154-243`
**Severity:** HIGH
**Effort:** 8 hours

### Problem
```
Scenario: Simple query, SLM times out or fails quality check

Timeline:
- Complexity check: 5ms
- SLM attempt: 200ms (timeout)
- LLM fallback: 300ms
- Total: 505ms+ (exceeds 500ms budget!)
```

### Solution: Adaptive Timeout + Quality Streaming
```rust
fn adaptive_timeout(&self, messages: &[Message]) -> Duration {
    let last_msg = messages.last().map(|m| &m.content).unwrap_or(&"");

    // Fast queries get shorter timeout
    if last_msg.len() < 20 { Duration::from_millis(100) }
    else if last_msg.contains("interest") { Duration::from_millis(150) }
    else { Duration::from_millis(180) }  // Never exceed to leave room for LLM
}

// Or: Start LLM in parallel, cancel if SLM succeeds
async fn execute_with_fallback(&self, messages: &[Message]) -> Result<...> {
    let slm_fut = self.slm.generate(messages);
    let llm_fut = self.llm.generate(messages);

    // Start both, take first success
    let (result, _) = tokio::select! {
        slm = slm_fut => {
            if self.is_acceptable(&slm?) { (slm, "slm") }
            else { (llm_fut.await, "llm") }
        }
        llm = llm_fut => (llm, "llm")
    };
}
```

---

## 2. FSM Missing Transitions

**Location:** `crates/agent/src/conversation.rs:262-296`
**Severity:** HIGH
**Effort:** 4 hours

### Problem
Only 4 intent-to-stage transitions implemented:
```rust
match intent.intent.as_str() {
    "farewell" => Some(Farewell),
    "objection" => Some(ObjectionHandling),
    "schedule_visit" if current == Presentation => Some(Closing),
    "affirmative" if current == Closing => Some(Farewell),
    _ => None,  // <-- ALL OTHER INTENTS IGNORED
}
```

### Missing Transitions
| Intent | Expected Transition |
|--------|---------------------|
| `negative` | Backtrack or Farewell |
| `interest_rate` | Stay or Qualification |
| `eligibility_check` | Qualification |
| `complaint` | Escalation (new stage) |
| `loan_inquiry` | Presentation |

### Solution
```rust
fn intent_based_transition(&self, intent: &Intent, current: ConversationStage) -> Option<ConversationStage> {
    match (intent.intent.as_str(), current) {
        // Existing transitions
        ("farewell", _) => Some(Farewell),
        ("objection", s) if s != ObjectionHandling => Some(ObjectionHandling),
        ("schedule_visit", Presentation) => Some(Closing),
        ("affirmative", Closing) => Some(Farewell),

        // NEW transitions
        ("negative", Presentation | Closing) => {
            // User not interested - try objection handling
            Some(ObjectionHandling)
        }
        ("negative", ObjectionHandling) => {
            // After objection handling, still negative = farewell
            Some(Farewell)
        }
        ("interest_rate" | "eligibility_check", Discovery) => {
            Some(Qualification)
        }
        ("loan_inquiry", Qualification | Discovery) => {
            Some(Presentation)
        }
        ("complaint" | "escalation", _) => {
            // Add Escalation stage to FSM
            Some(Escalation)
        }
        _ => None,
    }
}
```

---

## 3. RRF Fusion Weighting Bug

**Location:** `crates/rag/src/retriever.rs:225-251`
**Severity:** HIGH
**Effort:** 2 hours

### Problem
Weight applied DURING RRF instead of after:
```rust
// Current (WRONG)
let weighted = rrf_score * self.config.dense_weight;  // Applied per-doc
```

RRF scores should be accumulated then weighted at fusion.

### Solution
```rust
fn compute_rrf_fusion(&self, dense: &[SearchResult], sparse: &[SearchResult]) -> Vec<SearchResult> {
    let mut scores: HashMap<String, (f32, f32)> = HashMap::new();

    // Compute raw RRF scores (no weighting yet)
    for (rank, result) in dense.iter().enumerate() {
        let rrf = 1.0 / (self.config.rrf_k + rank as f32 + 1.0);
        scores.entry(result.id.clone())
            .or_insert((0.0, 0.0))
            .0 += rrf;
    }

    for (rank, result) in sparse.iter().enumerate() {
        let rrf = 1.0 / (self.config.rrf_k + rank as f32 + 1.0);
        scores.entry(result.id.clone())
            .or_insert((0.0, 0.0))
            .1 += rrf;
    }

    // Apply weights at fusion time
    let mut fused: Vec<_> = scores.into_iter()
        .map(|(id, (dense_score, sparse_score))| {
            let combined = dense_score * self.config.dense_weight
                         + sparse_score * (1.0 - self.config.dense_weight);
            (id, combined)
        })
        .collect();

    fused.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    // ...
}
```

---

## 4. Authentication Disabled by Default

**Location:** `crates/config/src/settings.rs:154-185`
**Severity:** HIGH
**Effort:** 1 hour

### Problem
```rust
impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,  // <-- DISABLED BY DEFAULT
            api_key: None,
        }
    }
}
```

### Solution
```rust
impl Default for AuthConfig {
    fn default() -> Self {
        // Check environment to determine default
        let is_production = std::env::var("ENVIRONMENT")
            .map(|v| v == "production")
            .unwrap_or(false);

        Self {
            enabled: is_production,  // Enabled in production
            api_key: std::env::var("API_KEY").ok(),
        }
    }
}

// Add validation at startup
impl Settings {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server.auth.enabled && self.server.auth.api_key.is_none() {
            return Err(ConfigError::Message(
                "Auth enabled but no API key configured".into()
            ));
        }
        // ... other validations
    }
}
```

---

## 5. WebRTC Audio Tracks Stubbed

**Location:** `crates/transport/src/webrtc.rs:321-360`
**Severity:** HIGH
**Effort:** 24+ hours

### Problem
```rust
// Audio track handler is stub
struct DummyAudioSource;  // Used as fallback

// No actual audio track processing
// No RTP packet handling
// No DTLS setup completion
```

### Options

**Option A: Complete WebRTC (24+ hours)**
- Implement ICE gathering
- Complete DTLS-SRTP handshake
- Handle RTP audio packets
- Implement jitter buffer

**Option B: Use Existing Library (8 hours)**
```toml
# Replace custom webrtc with proven library
webrtc = "0.10"  # pion-rs bindings
# or
libwebrtc-rs = "0.1"  # Google's libwebrtc
```

**Recommendation:** Option B - Use existing library. Custom WebRTC is extremely complex.

---

## 6. Sparse Search Blocks Async

**Location:** `crates/rag/src/retriever.rs:186-192`
**Severity:** MEDIUM-HIGH
**Effort:** 1 hour

### Problem
```rust
let sparse_future = async {
    if self.sparse_index.is_some() {
        self.search_sparse(query)  // <-- BLOCKING BM25 call
    }
};
```

Tantivy BM25 is CPU-intensive but runs on async thread.

### Solution
```rust
let sparse_future = async {
    if let Some(index) = &self.sparse_index {
        let idx = index.clone();
        let q = query.to_string();
        let k = top_k;

        tokio::task::spawn_blocking(move || {
            idx.search(&q, Some(k))
        }).await?
    } else {
        Ok(Vec::new())
    }
};
```

---

## 7. Memory Summarization Fire-and-Forget

**Location:** `crates/agent/src/agent.rs:277-281`
**Severity:** MEDIUM-HIGH
**Effort:** 4 hours

### Problem
```rust
tokio::spawn(async move {
    if let Err(e) = memory.summarize_pending_async().await {
        tracing::debug!("Memory summarization skipped: {}", e);
        // Error just logged, not handled!
    }
});
```

If LLM fails repeatedly, memory bloats until context overflow.

### Solution
```rust
// Track consecutive failures
static SUMMARIZATION_FAILURES: AtomicU32 = AtomicU32::new(0);

pub async fn process(&mut self, input: &str) -> Result<AgentResponse, AgentError> {
    // ... existing logic ...

    // Synchronous watermark check
    let memory = self.conversation.memory();
    if memory.working_size() > self.config.memory_watermark {
        // Don't spawn, do sync summarization
        match memory.summarize_pending_async().await {
            Ok(_) => {
                SUMMARIZATION_FAILURES.store(0, Ordering::Relaxed);
            }
            Err(e) => {
                let failures = SUMMARIZATION_FAILURES.fetch_add(1, Ordering::Relaxed) + 1;
                if failures > 3 {
                    // Emergency: truncate oldest turns
                    memory.emergency_truncate();
                    SUMMARIZATION_FAILURES.store(0, Ordering::Relaxed);
                }
                tracing::warn!("Summarization failed {}/3: {}", failures, e);
            }
        }
    }
}
```

---

## 8. Linear Resampling Quality

**Location:** `crates/core/src/audio.rs:215-238`
**Severity:** MEDIUM
**Effort:** 2 hours

### Problem
Using linear interpolation instead of proper resampler:
```rust
let sample = self.samples[idx_floor] * (1.0 - frac as f32)
    + self.samples[idx_ceil] * frac as f32;
```

Rubato is in dependencies but not used.

### Solution
```rust
use rubato::{SincFixedIn, Resampler, InterpolationType, WindowFunction};

pub fn resample(&self, target_rate: SampleRate) -> Result<Self, AudioError> {
    if self.sample_rate == target_rate {
        return Ok(self.clone());
    }

    let ratio = target_rate.as_hz() as f64 / self.sample_rate.as_hz() as f64;

    let mut resampler = SincFixedIn::<f32>::new(
        ratio,
        2.0,  // max relative ratio change
        InterpolationType::Cubic,
        256,  // chunk size
        1,    // channels
    )?;

    let input = vec![self.samples.clone()];
    let output = resampler.process(&input, None)?;

    Ok(Self {
        samples: output[0].clone(),
        sample_rate: target_rate,
        ..self.clone()
    })
}
```

---

## 9. Cascaded Pre-filter Threshold Too Low

**Location:** `crates/rag/src/reranker.rs:356-368`
**Severity:** MEDIUM
**Effort:** 1 hour

### Problem
```rust
prefilter_threshold = 0.1  // Only 10% keyword overlap required
max_full_model_docs = 10   // Take top 10 anyway
```

For 20 candidates, ~18-19 pass the 0.1 threshold. Minimal speedup.

### Solution
```rust
// Adjust defaults for meaningful pre-filtering
impl Default for EarlyExitConfig {
    fn default() -> Self {
        Self {
            prefilter_threshold: 0.4,  // 40% overlap required (was 0.1)
            max_full_model_docs: 5,    // Top 5 only (was 10)
            confidence_threshold: 0.85,
            // ...
        }
    }
}
```

**Expected impact:** 50% speedup (5 full model runs instead of 10-18)

---

## 10. CORS Unsafe Defaults

**Location:** `crates/server/src/http.rs:76-117`
**Severity:** MEDIUM-HIGH
**Effort:** 1 hour

### Problem
Three fallback levels create confusion:
1. `enabled: false` → Permissive (all origins)
2. `origins: []` → Hardcoded localhost:3000
3. Configured → Uses list

### Solution
```rust
pub fn cors_layer(config: &ServerConfig) -> Result<CorsLayer, ConfigError> {
    let cors_config = &config.cors;

    // Fail fast in production with bad config
    if is_production() && !cors_config.enabled {
        return Err(ConfigError::Message(
            "CORS must be configured in production".into()
        ));
    }

    if is_production() && cors_config.origins.is_empty() {
        return Err(ConfigError::Message(
            "Explicit CORS origins required in production".into()
        ));
    }

    // Only allow permissive in development
    if !cors_config.enabled {
        tracing::warn!("CORS disabled - development mode only!");
        return Ok(CorsLayer::permissive());
    }

    // Validate and apply configured origins
    // ...
}
```

---

## Summary: P1 High Priority Fixes

| # | Issue | Effort | Owner | Status |
|---|-------|--------|-------|--------|
| 1 | SlmFirst latency cliffs | 8h | LLM | [ ] |
| 2 | FSM missing transitions | 4h | Agent | [ ] |
| 3 | RRF fusion weighting | 2h | RAG | [ ] |
| 4 | Auth disabled by default | 1h | Security | [ ] |
| 5 | WebRTC stubbed | 24h+ | Transport | [ ] |
| 6 | Sparse search blocking | 1h | RAG | [ ] |
| 7 | Memory fire-and-forget | 4h | Agent | [ ] |
| 8 | Linear resampling | 2h | Pipeline | [ ] |
| 9 | Pre-filter threshold | 1h | RAG | [ ] |
| 10 | CORS unsafe defaults | 1h | Security | [ ] |

**Total Effort: ~48+ hours (2 dev weeks)**

---

*Next: See 03-MEDIUM-PRIORITY-FIXES.md for P2 issues*
