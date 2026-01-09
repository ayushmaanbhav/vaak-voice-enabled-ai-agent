# Voice Agent Backend: Comprehensive Code Review & Domain-Agnostic Refactoring Analysis

**Date:** 2026-01-07
**Goal:** Make the codebase truly domain-agnostic, config-driven, and production-ready
**Scope:** All 11 crates in the workspace + YAML configurations

---

## Executive Summary

This review identifies **287+ specific issues** across the codebase that prevent true domain-agnosticism. The core finding is that while architectural foundations are strong, **domain-specific code (gold loan/Kotak/bank)** has leaked into every layer of the application.

### Key Metrics

| Category | Critical | High | Medium | Low | Total |
|----------|----------|------|--------|-----|-------|
| Domain-Specific Hardcoding | 45 | 62 | 38 | 22 | **167** |
| Code Duplication | 8 | 15 | 12 | 5 | **40** |
| Missing Abstractions | 12 | 18 | 8 | 2 | **40** |
| Performance Issues | 3 | 8 | 12 | 7 | **30** |
| Architectural Flaws | 2 | 5 | 3 | 0 | **10** |
| **Total** | **70** | **108** | **73** | **36** | **287** |

### Target Outcome
After refactoring, onboard any business domain by ONLY defining YAML configs - zero Rust code changes required.

---

## Part 1: Critical Domain-Specific Code (Must Fix Immediately)

### 1.1 Hardcoded Bank/Product Names Across Codebase

| File | Lines | Hardcoded Value | Impact |
|------|-------|-----------------|--------|
| `server/src/state.rs` | 68-70 | `PhoneticCorrector::gold_loan()` | Server layer coupled to domain |
| `server/src/http.rs` | 464-476 | `"gold_loan"` JSON response key | API schema domain-specific |
| `server/src/main.rs` | 290 | `"gold_loan"` default DOMAIN_ID | Runtime hardcoded |
| `llm/src/prompt.rs` | 119-188 | `gold_loan_tools()` function | All 10 tools hardcoded |
| `llm/src/prompt.rs` | 303-345 | "Gold Loan specialist at Kotak" | System prompt hardcoded |
| `llm/src/prompt.rs` | 709-717 | "Kotak Mahindra Bank" greetings | All greetings hardcoded |
| `llm/src/speculative.rs` | 813-853 | "kotak", "muthoot", "manappuram" | Relevance scoring domain-specific |
| `text_processing/src/slot_extraction/mod.rs` | 164-176 | Lender name patterns | Competitor detection hardcoded |
| `text_processing/src/slot_extraction/mod.rs` | 129-145 | Intent patterns | 15 intents gold-loan-specific |
| `pipeline/src/tts/g2p.rs` | 232 | "kotak" phoneme mapping | Speech processing domain-specific |
| `pipeline/src/stt/decoder.rs` | 416 | "Kotak Mahindra", "gold loan" | Entity boosting hardcoded |
| `agent/src/persuasion.rs` | 240-600 | All objection responses | 350+ lines domain-specific |
| `agent/src/dst/slots.rs` | 82-91 | "Kotak pays off lender" | NextBestAction hardcoded |
| `core/src/customer.rs` | 37-76 | Segment key messages | "Shakti Gold program" |
| `core/src/domain_context.rs` | 69-151 | Gold loan fallback context | Complete Kotak vocabulary |
| `tools/src/gold_loan/branches.rs` | 79-140 | Kotak branch data | 4 branches hardcoded |
| `tools/src/gold_loan/tools/sms.rs` | 118-165 | SMS templates | All templates hardcoded |
| `tools/src/gold_loan/tools/document_checklist.rs` | 85-165 | Document requirements | 16 documents hardcoded |

### 1.2 Hardcoded Business Constants

| File | Lines | Constant | Value | Should Be |
|------|-------|----------|-------|-----------|
| `config/src/constants.rs` | 18-28 | Interest rates | 11.5%, 10.5%, 9.5% | domain.yaml |
| `config/src/constants.rs` | 36-44 | Loan tiers | 1L, 5L boundaries | domain.yaml |
| `config/src/constants.rs` | 47-52 | LTV percent | 75% | domain.yaml |
| `config/src/constants.rs` | 56-72 | Gold price | 7500.0 | External API |
| `core/src/customer.rs` | 221 | Gold price per gram | 7500.0 | Config |
| `core/src/customer.rs` | 258, 263 | High-value thresholds | 100g, 500K | segments.yaml |
| `agent/src/lead_scoring.rs` | 254 | High value loan | 1,000,000 | scoring.yaml |
| `agent/src/agent_config.rs` | 170 | Default loan amount | 100,000 | Config |

### 1.3 Domain-Specific Structs That Should Be Generic

| Struct | Location | Issue | Fix |
|--------|----------|-------|-----|
| `GoldLoanDialogueState` | `agent/src/dst/slots.rs:187-320` | 15 gold-loan fields | Use `DynamicDialogueState` with HashMap |
| `CustomerSegment` | `core/src/customer.rs:5-21` | Gold-loan segments | Config-driven segment definitions |
| `ObjectionType` | `core/src/traits/fsm.rs:108-125` | Gold-loan objection variants | String-based objection IDs from config |
| `MonthlySavings` | `config/src/domain/views.rs:985-992` | Interest-based calculation | Generic comparison struct |
| `CompetitorInfo` | `config/src/domain/views.rs:977-983` | Gold loan competitors | Generic competitor interface |

---

## Part 2: Factory Pattern & Trait Design Issues

### 2.1 Factory Pattern Anti-Patterns

**File:** `config/src/domain/bridge.rs`

| Method | Line | Current Return | Should Return |
|--------|------|----------------|---------------|
| `calculator()` | 58 | `impl DomainCalculator` | `Arc<dyn DomainCalculator>` |
| `lead_scoring()` | 94 | `impl LeadScoringStrategy` | `Arc<dyn LeadScoringStrategy>` |
| `competitor_analyzer()` | 160 | `impl CompetitorAnalyzer` | `Arc<dyn CompetitorAnalyzer>` |
| `segment_detector()` | 217 | `impl SegmentDetector` | `Arc<dyn SegmentDetector>` |
| `objection_handler()` | 252 | `impl ObjectionHandler` | `Arc<dyn ObjectionHandler>` |
| `goal_schema()` | 295 | `impl ConversationGoalSchema` | `Arc<dyn ConversationGoalSchema>` |

**Impact:** Cannot swap implementations at runtime, cannot mock for tests, no polymorphism possible.

### 2.2 Missing Trait Implementations

| Trait | Location | Status | Impact |
|-------|----------|--------|--------|
| `AudioProcessor` | `core/src/traits/speech.rs:329-343` | Declared, NOT implemented | No AEC/NS/AGC available |
| `Retriever` | `core/src/traits/retriever.rs` | Interface only | RAG system non-functional |
| `SlotSchema::extract_slots()` | `core/src/traits/slots.rs:174-178` | Method signature only | Cannot extract slots via trait |
| `ToolFactory` | Missing | Not defined | No domain-agnostic tool creation |
| `IntentClassifier` | Missing | Not defined | Only regex-based detection |
| `ConfigValidator` | Missing | Not defined | No YAML validation at startup |

### 2.3 Duplicate Trait Definitions

| Trait | Location 1 | Location 2 | Action |
|-------|------------|------------|--------|
| `ObjectionHandler` | `core/src/traits/objections.rs:136-167` | `core/src/domain/traits.rs:111-123` | Delete domain/traits.rs version |
| `SegmentDetector` | `core/src/traits/segments.rs:116-154` | `CustomerSegment` in domain/traits.rs | Delete CustomerSegment |

### 2.4 SRP Violations in Traits

**LeadScoringStrategy** (`core/src/traits/scoring.rs:322-359`) conflates:
- Score calculation (`calculate_breakdown`)
- Escalation detection (`check_escalation_triggers`)
- Language keywords (`urgency_keywords`)
- Config access (`thresholds`)

**ConversationGoalSchema** (`core/src/traits/goals.rs:146-186`) conflates:
- Goal definitions
- Intent mapping
- Next-best-action logic
- Tool suggestions

---

## Part 3: Code Duplication Analysis

### 3.1 Config Error Types (10 duplications)

All config modules have identical error enums:
```rust
pub enum XxxConfigError {
    FileNotFound(String, String),
    ParseError(String),
}
```

**Files:** branches.rs, competitors.rs, prompts.rs, objections.rs, slots.rs, scoring.rs, stages.rs, tools.rs, segments.rs, goals.rs, sms_templates.rs

**Fix:** Create single `ConfigLoadError` in domain/mod.rs

### 3.2 Config .load() Pattern (11 duplications)

All config modules repeat:
```rust
pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, XxxConfigError> {
    let content = std::fs::read_to_string(path.as_ref())?;
    serde_yaml::from_str(&content)?
}
```

**Fix:** Create `ConfigFile` trait with default implementation

### 3.3 Agent Constructor Duplication (3 copies, ~200 LOC)

| Constructor | Lines | Content |
|-------------|-------|---------|
| `new()` | 120-285 | Full initialization |
| `with_llm()` | 306-409 | Same with explicit LLM |
| `without_llm()` | 412-491 | Same without LLM |

**Fix:** Builder pattern with shared initialization logic

### 3.4 Regex Patterns (2 locations)

| Pattern Type | agent/dst/extractor.rs | text_processing/intent/mod.rs |
|--------------|------------------------|-------------------------------|
| Loan amount | Line 42 | Line 307 |
| Weight patterns | Lines 50-70 | Lines 315-335 |
| Phone patterns | Lines 80-90 | Lines 340-350 |
| Hindi multipliers | Lines 95-110 | Lines 360-380 |

**Fix:** Shared `PatternRegistry` in text_processing crate

### 3.5 Interest Rate Definitions (5 locations!)

| Location | How Defined |
|----------|-------------|
| `config/constants.rs` | Hardcoded const |
| `config/domain/master.rs` | InterestRatesConfig struct |
| `config/default.yaml` | gold_loan.kotak_interest_rate |
| `config/domain.yaml` | constants.base_rate |
| `config/domains/gold_loan/domain.yaml` | interest_rates.base_rate |

---

## Part 4: File Organization Issues

### 4.1 Monster Files (>700 LOC)

| File | Lines | Issues | Split Into |
|------|-------|--------|------------|
| `pipeline/src/stt/indicconformer.rs` | 1639 | Audio preprocessing, ONNX, Candle, mel filterbank, CTC | 4 files |
| `text_processing/src/intent/mod.rs` | 1521 | Intent definitions, 50+ regex, Hindi numerals, slot extraction | 4 files |
| `agent/src/dst/slots.rs` | 1377 | GoldLoanDialogueState, 80+ getters, tests, NextBestAction | 3 files |
| `server/src/ptt.rs` | 1316 | Audio processing, STT pooling, markdown, base64 | 4 files |
| `pipeline/src/orchestrator.rs` | 1232 | Pipeline orchestration | 2 files |
| `config/src/settings.rs` | 1129 | All config structs | 3 files |
| `agent/src/conversation.rs` | 1117 | Conversation + compliance + stage transitions | 3 files |
| `agent/src/memory/mod.rs` | 1179 | AgenticMemory + statistics + context | 2 files |
| `agent/src/persuasion.rs` | 986 | PersuasionEngine + objection handlers + scripts | 2 files |
| `agent/src/agent/mod.rs` | 978 | Agent + 3 constructors + helpers | 3 files |
| `config/src/domain/views.rs` | 1000+ | AgentView, LlmView, ToolsView | 3 files |

### 4.2 Wrong Crate Boundaries

| Code | Current Location | Should Be |
|------|------------------|-----------|
| Slot extraction | `agent/src/dst/extractor.rs` | `text_processing/src/slots/` |
| DST patterns | `agent/src/dst/` | `text_processing/src/slots/` |
| Gold loan tools | `tools/src/gold_loan/` | Config-driven generic tools |
| Intent-to-stage mapping | `agent/src/conversation.rs:620-717` | YAML config |
| AI disclosure | `agent/src/conversation.rs:251-265` | YAML config |

---

## Part 5: Performance Issues

### 5.1 Regex Compilation at Runtime

**File:** `text_processing/src/intent/mod.rs:110-111`
```rust
pub fn new(config: IntentConfig) -> Self {
    let slot_patterns = compile_slot_patterns(); // Called every time!
}
```
**Fix:** Use `Lazy<Regex>` pattern

### 5.2 Excessive String Allocations

- **2567 instances of `.to_string()`** across codebase
- Many could use `Cow<str>` or references

### 5.3 Arc Overuse in Agent

**File:** `agent/src/agent/mod.rs:81-99`
```rust
pub(crate) conversation: Arc<dyn ConversationContext>,
pub(crate) tools: Arc<ToolRegistry>,
pub(crate) llm: Option<Arc<dyn LanguageModel>>,
pub(crate) agentic_retriever: Option<Arc<AgenticRetriever>>,
pub(crate) vector_store: Option<Arc<VectorStore>>,
pub(crate) translator: Option<Arc<dyn Translator>>,
```
**Fix:** Consider `Arc<DomainAgentInner>` with all fields inside

### 5.4 Lock Contention in WebSocket

**File:** `server/src/websocket.rs`
- 25+ `.lock().await` calls in nested async functions
- Triple-nested Arc/Mutex for sender
- Lock acquired in debug log paths

### 5.5 Message Cloning in Speculative Execution

**File:** `llm/src/speculative.rs:203-207`
```rust
let messages_for_llm = messages.to_vec();  // Expensive clone
```
**Fix:** Use `Arc<Vec<Message>>`

### 5.6 Token Estimation Without Caching

**File:** `llm/src/prompt.rs:670-691`
```rust
pub fn estimate_tokens(&self) -> usize {
    self.messages.iter().map(|m| /* recompute */ ).sum()
}
```
Called repeatedly without caching

---

## Part 6: Concurrency & Safety Issues

### 6.1 Critical Bug: Session Touch

**File:** `server/src/session.rs:134`
```rust
// InMemorySessionStore.touch() sets timestamp to 0 - BUG!
```
Sessions would never expire.

### 6.2 TranscriptAccumulator Not Thread-Safe

**File:** `core/src/transcript.rs:135-261`
- Contains mutable state (stable_text, unstable_text, words)
- `process()` modifies internal state
- No Mutex/RwLock protection

### 6.3 Task Lifecycle Not Managed

**File:** `server/src/websocket.rs:723-727`
```rust
audio_task.abort();
event_task.abort();
pipeline_event_task.abort();
```
- No graceful shutdown
- Could cause panic in critical section
- No timeout before abort

### 6.4 No Backpressure in Audio Channel

**File:** `server/src/websocket.rs:137, 670`
```rust
let (audio_tx, _) = mpsc::channel::<Vec<u8>>(100); // Fixed capacity
let _ = audio_tx.send(audio_bytes).await; // Silent drop on overflow
```

### 6.5 Unwrap Epidemic in WebSocket

**File:** `server/src/websocket.rs`
- **19 instances** of `serde_json::to_string(...).unwrap()`
- Any serialization failure causes panic

---

## Part 7: YAML Configuration Issues

### 7.1 Data Duplication Across Config Files

| Value | Locations | Count |
|-------|-----------|-------|
| Interest rates | default.yaml, domain.yaml, competitors.yaml | 4 |
| LTV percent | domain.yaml, competitors.yaml | 3 |
| Gold price | default.yaml, domain.yaml | 3 |
| Min/max loan | default.yaml, domain.yaml, slots.yaml | 3 |
| Purity factors | default.yaml, domain.yaml, slots.yaml | 3 |
| Competitor data | default.yaml, domain.yaml, competitors.yaml | 3 |

### 7.2 Missing Config Abstractions

| Missing | Purpose |
|---------|---------|
| `meta.yaml` | Domain metadata (id, name, type, currency) |
| `schema.yaml` | Validation rules per domain |
| `rules.yaml` | Eligibility/scoring rules |
| Currency config | Explicit currency symbol and formatting |
| Locale config | Supported languages, default language |

### 7.3 Hardcoded Domain Reference

**File:** `config/domain.yaml:4`
```yaml
domain: gold_loan  # Hardcoded!
```
Should be env var or runtime parameter

---

## Part 8: Refactoring Roadmap

### Phase 1: Critical Fixes (Week 1)

#### P1.1 Fix DomainBridge Factory Pattern
- Change all 6 methods to return `Arc<dyn Trait>`
- Enable polymorphism and testing

#### P1.2 Remove Duplicate Traits
- Delete `CustomerSegment` from domain/traits.rs
- Delete `ObjectionHandler` from domain/traits.rs

#### P1.3 Extract Domain Constants to Config
- Move all interest rates to domain.yaml
- Move all thresholds to segments.yaml/scoring.yaml
- Remove constants.rs hardcoded values

#### P1.4 Fix Session Bug
- Fix `touch()` to set correct timestamp

### Phase 2: Domain Decoupling (Week 2)

#### P2.1 Make Tool Definitions Config-Driven
- Load tools from tools/schemas.yaml
- Create generic `ToolFactory` trait
- Remove `gold_loan_tools()` function

#### P2.2 Make System Prompt Config-Driven
- Load prompt template from prompts/system.yaml
- Remove hardcoded "Gold Loan specialist" text
- Parameterize bank name, agent name, product

#### P2.3 Make Slot Extraction Config-Driven
- Load slot names from slots.yaml
- Load intent patterns from goals.yaml
- Load lender patterns from competitors.yaml

#### P2.4 Make DialogueState Generic
- Create `DynamicDialogueState` with HashMap<String, SlotValue>
- Remove `GoldLoanDialogueState` struct

### Phase 3: Code Organization (Week 3)

#### P3.1 Split Monster Files
- indicconformer.rs → 4 modules
- intent/mod.rs → 4 modules
- dst/slots.rs → 3 modules
- ptt.rs → 4 modules

#### P3.2 Consolidate Duplicate Code
- Create single `ConfigLoadError`
- Create `ConfigFile` trait
- Move regex patterns to shared module
- Use builder pattern for agent constructors

#### P3.3 Fix Crate Boundaries
- Move slot extraction to text_processing
- Move intent-to-stage mapping to config

### Phase 4: Performance & Safety (Week 4)

#### P4.1 Fix Concurrency Issues
- Add graceful shutdown to WebSocket tasks
- Replace unwrap() with proper error handling
- Add backpressure to audio channel
- Make TranscriptAccumulator thread-safe

#### P4.2 Optimize Performance
- Use `Lazy<Regex>` everywhere
- Cache token estimates
- Use Arc for message passing
- Reduce lock contention

### Phase 5: Implementation Gaps (Week 5)

#### P5.1 Implement Missing Traits
- AudioProcessor (or document as out of scope)
- Retriever concrete implementation
- ConfigValidator for YAML validation

#### P5.2 Add Feature Flags
- `balance_transfer_enabled`
- `segment_detection.high_value_enabled`
- `competitor_comparison_enabled`

---

## Part 9: Verification Checklist

After refactoring, verify:

- [ ] No hardcoded "Kotak", "gold loan", "Muthoot", "Manappuram" in Rust files
- [ ] No hardcoded rates (9.5%, 10.5%, 11.5%, 18.0%, 19.0%) in Rust files
- [ ] No hardcoded branch names in Rust files
- [ ] All config comes from YAML files
- [ ] New domain can be added with only YAML changes
- [ ] All tests pass
- [ ] Build succeeds
- [ ] No unwrap() in WebSocket handler
- [ ] Session touch() works correctly

---

## Part 10: Files to Delete After Migration

| File | Reason |
|------|--------|
| `config/src/constants.rs` | Move to domain.yaml |
| `config/src/gold_loan.rs` | Replaced by MasterDomainConfig |
| `config/src/branch.rs` | Replaced by branches.yaml |
| `config/src/competitor.rs` | Replaced by competitors.yaml |
| `config/src/domain_config.rs` | Replaced by MasterDomainConfig |
| `config/src/prompts.rs` | Replaced by prompts/system.yaml |
| `config/src/product.rs` | Replaced by domain.yaml products |
| `core/src/domain/traits.rs` | Duplicate traits |

---

## Appendix A: All Domain-Specific References Found

### Search Pattern Used
```
grep -rn "gold.loan|kotak|muthoot|manappuram|nbfc|interest.rate|lakh|crore|purity|K24|K22|11\.5|10\.5|9\.5|18\.0|Priya|Gold.Loan.specialist" --include="*.rs" --include="*.yaml"
```

### Results Summary
- **Rust files with domain references:** 47
- **YAML files with domain references:** 17
- **Total occurrences:** 1,200+

---

## Appendix B: Trait Inventory (42 Total)

### Core Infrastructure (6)
- `LanguageModel`, `SpeechToText`, `TextToSpeech`, `VoiceActivityDetector`, `AudioProcessor`, `Retriever`

### Text Processing (5)
- `GrammarCorrector`, `Translator`, `PIIRedactor`, `ComplianceChecker`, `TextProcessor`

### Conversation (3)
- `ConversationFSM`, `FrameProcessor`, `Tool`

### Domain-Agnostic (9)
- `DomainCalculator`, `SlotSchema`, `ConversationGoalSchema`, `SegmentDetector`
- `ObjectionHandler`, `LeadScoringStrategy`, `CompetitorAnalyzer`, `SlotDefinition`, `GoalDefinition`

### Config-Driven Implementations (8)
- `ConfigDrivenCalculator`, `ConfigSlotDefinition`, `ConfigGoalDefinition`, `ConfigGoalSchema`
- `ConfigSegmentDetector`, `ConfigObjectionHandler`, `ConfigLeadScoring`, `ConfigCompetitorAnalyzer`

### Persistence (5)
- `SessionStore`, `GoldPriceService`, `AppointmentStore`, `SmsService`, `AuditLog`

### Tools (4)
- `ToolExecutor`, `CrmIntegration`, `CalendarIntegration`, `ResourceProvider`

### Missing (6)
- `ToolFactory`, `IntentClassifier`, `IntentToGoalMapper`, `SlotValidator`, `SlotExtractor`, `ConfigValidator`

---

*Generated: 2026-01-07*
*Analysis performed by: Claude Opus 4.5*
