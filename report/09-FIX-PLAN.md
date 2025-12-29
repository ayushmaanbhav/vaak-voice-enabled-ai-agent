# Detailed Fix Plan - Voice Agent Rust

## P0 Critical Fixes (Must Complete Before Production)

---

### Fix 1: Register Missing P0 Tools in Registry

**Issue:** GetGoldPriceTool, EscalateToHumanTool, SendSmsTool are implemented but NOT registered.

**Location:** `crates/tools/src/registry.rs`

**Current Code (line 341):**
```rust
pub fn create_default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(EligibilityCheckTool::new());
    registry.register(SavingsCalculatorTool::new());
    registry.register(LeadCaptureTool::new(None));
    registry.register(AppointmentSchedulerTool::new(None));
    registry.register(BranchLocatorTool::new());
    // Missing: GetGoldPriceTool, EscalateToHumanTool, SendSmsTool
    registry
}
```

**Fix:**
```rust
pub fn create_default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(EligibilityCheckTool::new());
    registry.register(SavingsCalculatorTool::new());
    registry.register(LeadCaptureTool::new(None));
    registry.register(AppointmentSchedulerTool::new(None));
    registry.register(BranchLocatorTool::new());
    // P0 FIX: Register missing tools
    registry.register(GetGoldPriceTool::new(None));  // Uses default GoldPriceService
    registry.register(EscalateToHumanTool::new());
    registry.register(SendSmsTool::new(None));  // Uses SimulatedSmsService
    registry
}

pub fn create_registry_with_integrations(config: IntegrationConfig) -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(EligibilityCheckTool::new());
    registry.register(SavingsCalculatorTool::new());
    registry.register(LeadCaptureTool::new(config.crm.clone()));
    registry.register(AppointmentSchedulerTool::new(config.calendar.clone()));
    registry.register(BranchLocatorTool::new());
    // P0 FIX: Register missing tools with integrations
    registry.register(GetGoldPriceTool::new(config.gold_price_service.clone()));
    registry.register(EscalateToHumanTool::new());
    registry.register(SendSmsTool::new(config.sms_service.clone()));
    registry
}
```

**Effort:** ~30 minutes
**Test:** Run `cargo test -p voice-agent-tools`

---

### Fix 2: Add Audit Log Table to Schema

**Issue:** audit.rs has complete implementation but schema.rs doesn't create the table.

**Location:** `crates/persistence/src/schema.rs`

**Add to create_tables() function:**
```rust
// P0 FIX: Create audit_log table for RBI compliance
session.query(
    format!(
        "CREATE TABLE IF NOT EXISTS {}.audit_log (
            partition_date TEXT,
            session_id TEXT,
            timestamp BIGINT,
            id UUID,
            event_type TEXT,
            actor_type TEXT,
            actor_id TEXT,
            resource_type TEXT,
            resource_id TEXT,
            action TEXT,
            outcome TEXT,
            details TEXT,
            previous_hash TEXT,
            hash TEXT,
            PRIMARY KEY ((partition_date, session_id), timestamp, id)
        ) WITH CLUSTERING ORDER BY (timestamp DESC, id DESC)
        AND default_time_to_live = 220752000",  // 7 years for banking compliance
        keyspace
    ),
    &[],
).await?;
```

**Effort:** ~15 minutes
**Test:** Run schema creation, verify table exists in ScyllaDB

---

### Fix 3: Integrate PersuasionEngine in Agent Process Flow

**Issue:** PersuasionEngine is created but never called during objection handling.

**Location:** `crates/agent/src/agent.rs`

**Add to process() method around line 680-700:**
```rust
// P0 FIX: Integrate PersuasionEngine for objection handling
if detected_intent.intent == "objection" ||
   self.conversation.stage() == ConversationStage::ObjectionHandling {

    // Detect objection type from user input
    if let Some(objection_type) = ObjectionType::detect(&english_input) {
        let persuasion_engine = PersuasionEngine::new();

        // Get objection response in user's language
        if let Some(response) = persuasion_engine.get_response(&objection_type, &self.user_language) {
            // Include persuasion guidance in LLM context
            builder = builder.with_context(&format!(
                "## Objection Handling Guidance\n\
                 Customer objection type: {:?}\n\
                 Acknowledge their concern: {}\n\
                 Reframe positively: {}\n\
                 Provide evidence: {}\n\
                 Suggested call-to-action: {}",
                objection_type,
                response.acknowledge,
                response.reframe,
                response.evidence,
                response.call_to_action
            ));
        }
    }
}
```

**Effort:** ~1 hour
**Test:** Test objection scenarios, verify persuasion responses included

---

### Fix 4: Correct EMI Calculation Formula

**Issue:** Uses simple interest instead of proper EMI formula.

**Location:** `crates/config/src/gold_loan.rs` (calculate_monthly_savings_tiered)

**Current (WRONG):**
```rust
let monthly_savings = loan_amount * (current_rate / 100.0 / 12.0)
                     - loan_amount * (kotak_rate / 100.0 / 12.0);
```

**Fix - Add EMI helper function:**
```rust
/// Calculate EMI using standard reducing balance formula
/// EMI = P × [r(1+r)^n] / [(1+r)^n - 1]
/// where P = principal, r = monthly rate, n = tenure in months
fn calculate_emi(principal: f64, annual_rate: f64, tenure_months: u32) -> f64 {
    if tenure_months == 0 || annual_rate <= 0.0 {
        return principal / tenure_months.max(1) as f64;
    }

    let monthly_rate = annual_rate / 100.0 / 12.0;
    let n = tenure_months as f64;

    let numerator = principal * monthly_rate * (1.0 + monthly_rate).powf(n);
    let denominator = (1.0 + monthly_rate).powf(n) - 1.0;

    if denominator.abs() < f64::EPSILON {
        return principal / n;
    }

    numerator / denominator
}

/// P0 FIX: Calculate monthly savings using proper EMI formula
pub fn calculate_monthly_savings_tiered(&self,
    loan_amount: f64,
    current_rate: f64,
    tenure_months: u32
) -> f64 {
    let kotak_rate = self.get_tiered_rate(loan_amount);

    let current_emi = calculate_emi(loan_amount, current_rate, tenure_months);
    let kotak_emi = calculate_emi(loan_amount, kotak_rate, tenure_months);

    // Return monthly savings (positive = customer saves)
    current_emi - kotak_emi
}
```

**Effort:** ~45 minutes
**Test:** Add unit tests with known EMI values, verify accuracy

---

### Fix 5: Create LanguageModel Adapter for LlmBackend

**Issue:** LLM crate's LlmBackend doesn't implement core's LanguageModel trait.

**Location:** Create new file `crates/llm/src/adapter.rs`

**New File:**
```rust
//! Adapter layer to bridge LlmBackend with core's LanguageModel trait

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

use voice_agent_core::traits::LanguageModel;
use voice_agent_core::llm_types::{GenerateRequest, GenerateResponse, StreamChunk, ToolCall};
use voice_agent_core::Error as CoreError;

use crate::{LlmBackend, Message, Role, LlmError};

/// Adapter that makes any LlmBackend implement LanguageModel
pub struct LanguageModelAdapter<B: LlmBackend> {
    backend: Arc<B>,
}

impl<B: LlmBackend> LanguageModelAdapter<B> {
    pub fn new(backend: Arc<B>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl<B: LlmBackend + 'static> LanguageModel for LanguageModelAdapter<B> {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse, CoreError> {
        // Convert GenerateRequest messages to LLM crate's Message format
        let messages: Vec<Message> = request.messages.iter().map(|m| {
            Message {
                role: match m.role.as_str() {
                    "system" => Role::System,
                    "user" => Role::User,
                    "assistant" => Role::Assistant,
                    "tool" => Role::Tool,
                    _ => Role::User,
                },
                content: m.content.clone(),
            }
        }).collect();

        // Call backend
        let result = self.backend.generate(&messages).await
            .map_err(|e| CoreError::Llm(e.to_string()))?;

        Ok(GenerateResponse {
            content: result.text,
            finish_reason: result.stop_reason,
            usage: None,  // Could map from result if available
            tool_calls: vec![],  // Parse from result.text if needed
        })
    }

    fn context_size(&self) -> usize {
        4096  // Default; could make configurable
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        self.backend.estimate_tokens(text)
    }
}

// Add to lib.rs exports:
// pub mod adapter;
// pub use adapter::LanguageModelAdapter;
```

**Effort:** ~1 hour
**Test:** Create integration test using adapter with Ollama backend

---

### Fix 6: Wire Transport Crate to Server

**Issue:** Transport crate is completely disconnected from server.

**Location:** Multiple files in `crates/server/`

**Step 1: Add dependency in `crates/server/Cargo.toml`:**
```toml
[dependencies]
voice-agent-transport = { path = "../transport" }
```

**Step 2: Add WebRTC signaling endpoints in `crates/server/src/http.rs`:**
```rust
use voice_agent_transport::{TransportSession, WebRtcConfig, SessionConfig};

// Add routes in create_router():
.route("/api/webrtc/offer", post(webrtc_offer))
.route("/api/webrtc/candidates/:session_id", post(webrtc_add_candidate))
.route("/api/webrtc/candidates/:session_id", get(webrtc_get_candidates))

// Handler functions:
async fn webrtc_offer(
    State(state): State<AppState>,
    Json(request): Json<WebRtcOfferRequest>,
) -> Result<Json<WebRtcAnswerResponse>, ServerError> {
    // Create transport session
    let config = SessionConfig {
        webrtc: Some(WebRtcConfig::default()),
        ..Default::default()
    };
    let mut transport = TransportSession::new(request.session_id.clone(), config);

    // Process SDP offer and get answer
    let answer = transport.connect(&request.sdp_offer).await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

    // Store transport in state (associate with session)
    state.transports.write().await.insert(request.session_id.clone(), transport);

    Ok(Json(WebRtcAnswerResponse { sdp_answer: answer }))
}

#[derive(Deserialize)]
struct WebRtcOfferRequest {
    session_id: String,
    sdp_offer: String,
}

#[derive(Serialize)]
struct WebRtcAnswerResponse {
    sdp_answer: String,
}
```

**Step 3: Add transport storage to AppState:**
```rust
pub struct AppState {
    // ... existing fields
    pub transports: Arc<RwLock<HashMap<String, TransportSession>>>,
}
```

**Effort:** ~4 hours
**Test:** Create WebRTC client test, verify signaling works

---

### Fix 7: Wire Frame Processors to Orchestrator (Optional)

**Issue:** Frame processors exist but orchestrator doesn't use them.

**Options:**
1. **Document as design decision** - Orchestrator is simplified demo, processors for advanced use
2. **Create ProcessorChain orchestrator** - Replace current state machine

**If choosing Option 2, add to `crates/pipeline/src/orchestrator.rs`:**
```rust
use crate::processors::{ProcessorChain, ProcessorChainBuilder, SentenceDetector, TtsProcessor, InterruptHandler};

impl VoicePipeline {
    /// Create a full processor-chain-based pipeline
    pub fn with_processors(config: PipelineConfig) -> Result<Self, PipelineError> {
        // Build processor chain
        let chain = ProcessorChainBuilder::new()
            .add(SentenceDetector::new(SentenceDetectorConfig::default()))
            .add(TtsProcessor::new(TtsProcessorConfig::default()))
            .add(InterruptHandler::new(InterruptHandlerConfig::default()))
            .build();

        // ... rest of initialization
    }
}
```

**Effort:** 8+ hours (significant refactor)
**Recommendation:** Document limitation for now, defer to Phase 3

---

## P1 High Priority Fixes

### Fix 8: Add HTTP Rate Limiting

**Location:** `crates/server/src/http.rs`

```rust
use tower::ServiceBuilder;
use tower_http::limit::RateLimitLayer;
use std::time::Duration;

// In create_router(), add before auth middleware:
.layer(
    ServiceBuilder::new()
        .layer(RateLimitLayer::new(100, Duration::from_secs(60)))  // 100 req/min
)
```

### Fix 9: Add Missing Configuration Files

Create `voice-agent/backend/config/default.yaml`:
```yaml
server:
  host: "0.0.0.0"
  port: 3000

pipeline:
  latency_budget_ms: 500
  vad:
    enabled: true
    model_type: silero

agent:
  language: hi
  persona:
    name: Kotak Assistant
    tone: professional

rag:
  dense_top_k: 20
  sparse_top_k: 20
  rerank_enabled: true
```

### Fix 10: Real SMS Integration (When Ready)

Replace SimulatedSmsService with actual provider (Twilio example):
```rust
pub struct TwilioSmsService {
    client: reqwest::Client,
    account_sid: String,
    auth_token: String,
    from_number: String,
}

#[async_trait]
impl SmsService for TwilioSmsService {
    async fn send(&self, to: &str, message: &str) -> Result<SmsResult, SmsError> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.account_sid
        );

        let response = self.client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&[
                ("To", to),
                ("From", &self.from_number),
                ("Body", message),
            ])
            .send()
            .await?;

        // ... handle response
    }
}
```

---

## P2 Medium Priority Fixes

### Fix 11: Add TextSimplifier for TTS

Create `crates/text_processing/src/simplifier.rs`:
```rust
/// Converts numbers and abbreviations to spoken form for TTS
pub struct TextSimplifier {
    language: Language,
}

impl TextSimplifier {
    pub fn simplify(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Convert numbers to words
        result = self.numbers_to_words(&result);

        // Expand abbreviations
        result = self.expand_abbreviations(&result);

        result
    }

    fn numbers_to_words(&self, text: &str) -> String {
        // Regex to find numbers
        let re = regex::Regex::new(r"\d+(\.\d+)?").unwrap();
        re.replace_all(text, |caps: &regex::Captures| {
            self.number_to_hindi_words(&caps[0])
        }).to_string()
    }

    fn number_to_hindi_words(&self, num_str: &str) -> String {
        // Implementation for Hindi number words
        // "12345" -> "बारह हज़ार तीन सौ पैंतालीस"
        // ...
    }
}
```

### Fix 12: Add A/B Testing Framework

Create `crates/config/src/experiments.rs`:
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Experiment {
    pub id: String,
    pub variants: Vec<ExperimentVariant>,
    pub active: bool,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExperimentVariant {
    pub name: String,
    pub percentage: u8,  // 0-100
    pub config_overrides: HashMap<String, serde_json::Value>,
}

pub struct ExperimentManager {
    experiments: Vec<Experiment>,
}

impl ExperimentManager {
    pub fn get_variant(&self, experiment_id: &str, user_id: &str) -> Option<&ExperimentVariant> {
        let experiment = self.experiments.iter().find(|e| e.id == experiment_id && e.active)?;

        // Consistent bucketing based on user_id hash
        let bucket = self.hash_to_bucket(user_id);

        let mut cumulative = 0;
        for variant in &experiment.variants {
            cumulative += variant.percentage;
            if bucket < cumulative {
                return Some(variant);
            }
        }
        None
    }
}
```

---

## Verification Checklist

After applying fixes:

- [ ] `cargo check --workspace` passes
- [ ] `cargo test --workspace` passes (all 200+ tests)
- [ ] `cargo clippy --workspace` has no errors
- [ ] New audit_log table exists in ScyllaDB
- [ ] All 8 tools appear in registry (verify with test)
- [ ] EMI calculation matches known values (add regression test)
- [ ] PersuasionEngine responses appear in objection handling
- [ ] WebRTC signaling endpoints respond (if implementing Fix 6)
- [ ] LanguageModel adapter compiles and tests pass

---

## Estimated Timeline

| Phase | Fixes | Effort | Duration |
|-------|-------|--------|----------|
| Phase 1 | Fixes 1-5 | ~4 hours | Day 1 |
| Phase 2 | Fixes 6-7 | ~8 hours | Day 2-3 |
| Phase 3 | Fixes 8-10 | ~4 hours | Day 4 |
| Phase 4 | Fixes 11-12 | ~8 hours | Day 5-7 |
| Testing | All | ~4 hours | Ongoing |

**Total Estimated Effort:** 2-3 weeks for full completion
**Minimum Viable:** Phase 1 (4 hours) for critical blockers only
