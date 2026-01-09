# Comprehensive Backend Code Review - January 9, 2026

## Executive Summary

This document presents an exhaustive code review of the voice-agent backend, with the **primary goal of achieving true domain-agnosticism** - enabling new domain onboarding through YAML configs only.

### Overall Assessment: **6.5/10** (Good foundation, significant domain coupling remains)

| Dimension | Score | Status |
|-----------|-------|--------|
| Trait Design | 8.5/10 | Excellent - 206+ trait object usages |
| Pluggability | 8.5/10 | Excellent - Multiple backends swappable |
| Config System | 7.5/10 | Good structure, incomplete wiring |
| Domain Agnosticism | 5.0/10 | **CRITICAL** - 287+ hardcoded references |
| Code Quality (SRP) | 5.5/10 | Large files, duplication |
| Concurrency Safety | 8.0/10 | Proper patterns, minor concerns |
| Error Handling | 8.0/10 | Consistent thiserror usage |
| Performance | 6.5/10 | Acceptable, some lock contention |

### Critical Blockers for Domain-Agnostic Goal

| Blocker | Location | Severity |
|---------|----------|----------|
| 287+ domain-specific hardcoded references | Multiple crates | CRITICAL |
| NextBestAction enum hardcoded | dst/slots.rs:65-92 | CRITICAL |
| SMS templates hardcoded | tools/sms.rs:118-165 | CRITICAL |
| Document checklist hardcoded | tools/document_checklist.rs:84-165 | CRITICAL |
| Domain boost terms hardcoded | rag/domain_boost.rs:159-305 | HIGH |
| Slot mappings hardcoded | agent/processing.rs:92-98 | HIGH |
| Appointment purposes hardcoded | tools/appointment.rs:87-94 | HIGH |
| Branch fallback data embedded | tools/locations.rs:77-141 | HIGH |
| GoldLoanDialogueState still primary | dst/slots.rs | MEDIUM |

---

## Table of Contents

1. [Domain-Specific Hardcoding Analysis](#1-domain-specific-hardcoding-analysis)
2. [Trait Design & Factory Patterns](#2-trait-design--factory-patterns)
3. [Configuration Architecture](#3-configuration-architecture)
4. [Code Organization & SRP Violations](#4-code-organization--srp-violations)
5. [Concurrency & Performance](#5-concurrency--performance)
6. [Crate-by-Crate Analysis](#6-crate-by-crate-analysis)
7. [Prioritized Action Plan](#7-prioritized-action-plan)
8. [Generic Naming Recommendations](#8-generic-naming-recommendations)

---

## 1. Domain-Specific Hardcoding Analysis

### 1.1 Summary Statistics

| Category | Count | Location(s) |
|----------|-------|-------------|
| Gold terminology | 120+ | agent/, rag/, tools/, core/ |
| Kotak branding | 85+ | server/ptt.rs, agent/response.rs, memory/core.rs |
| Financial constants | 45+ | calculator.rs, customer.rs, persistence/ |
| Competitor data | 65+ | persuasion.rs, savings.rs, domain_boost.rs |
| Slot name references | 30+ | processing.rs, memory/, dst/ |
| Tool schema content | 20+ | tools/domain_tools/ |

**Total: 287+ hardcoded domain-specific references**

### 1.2 Critical Hardcoding (P0 - Blocks Domain Agnosticism)

#### Tools Crate - SMS Templates
**File:** `crates/tools/src/domain_tools/tools/sms.rs` (Lines 118-165)
```rust
// HARDCODED - Should be in sms_templates.yaml
format!(
    "Dear {}, your Gold Loan appointment is confirmed for {}. Please bring your gold and KYC documents. - Bank",
    customer_name, details
)
```
**Impact:** Cannot send SMS for non-gold-loan domains without code change.

#### Tools Crate - Document Checklist
**File:** `crates/tools/src/domain_tools/tools/document_checklist.rs` (Lines 84-165)
```rust
// HARDCODED - Should be in documents.yaml
json!({
    "document": "Gold Items",
    "notes": "Bring gold jewelry/items for valuation. Remove any non-gold attachments"
})
```
**Impact:** Document requirements are gold-loan specific.

#### Tools Crate - Appointment Purposes
**File:** `crates/tools/src/domain_tools/tools/appointment.rs` (Lines 87-94, 148-154)
```rust
// HARDCODED enum values
vec!["New Gold Loan", "Gold Loan Transfer", "Top-up", "Closure"]
```
**Impact:** Cannot schedule appointments for other domains.

#### RAG Crate - Domain Boost Terms
**File:** `crates/rag/src/domain_boost.rs` (Lines 159-305)
```rust
fn load_gold_loan_terms(&self) {
    // 146 lines of hardcoded gold loan domain terms
    DomainTerm { term: "gold loan".to_string(), boost: 2.0, ... }
    DomainTerm { term: "jewel loan".to_string(), boost: 1.8, ... }
    // Brand terms: kotak, muthoot, manappuram, iifl
}
```
**Impact:** RAG boosting is hardcoded for gold loans only.

#### Server Crate - Greeting Messages
**File:** `crates/server/src/ptt.rs` (Lines 751-776)
```rust
// HARDCODED greeting
"Hi, I'm your Kotak Gold Loan assistant"
```
**Impact:** Branding embedded in server code.

### 1.3 High Priority Hardcoding (P1)

#### Agent Crate - Slot Name Mappings
**File:** `crates/agent/src/agent/processing.rs` (Lines 92-98)
```rust
// HARDCODED slot aliases - should be in slots.yaml
"gold_weight" | "weight" => Some("gold_weight"),
"gold_purity" | "purity" | "karat" => Some("gold_purity"),
"loan_amount" | "amount" => Some("loan_amount"),
```
**Impact:** Cannot map slots for other domains without code change.

#### Agent Crate - Intent to Tool Mappings
**File:** `crates/agent/src/agent/tools.rs` (Lines 20-72)
```rust
// HARDCODED intent → tool mappings
"eligibility_check" => Some("check_eligibility"),
"switch_lender" => Some("calculate_savings"),
```
**Impact:** Intent-to-tool routing is hardcoded.

#### Agent Crate - Memory Compression Priorities
**File:** `crates/agent/src/memory/compressor.rs` (Lines 51-60)
```rust
priority_entities: vec![
    "gold_weight".to_string(),
    "loan_amount".to_string(),
    "gold_purity".to_string(),
]
```
**Impact:** Memory compression prioritizes gold-loan entities.

#### Core Crate - Customer Segmentation
**File:** `crates/core/src/customer.rs` (Lines 221, 258, 263, 271-273)
```rust
// HARDCODED thresholds and competitor names
let gold_value = gold_grams * 7500.0; // Hardcoded price
if gold_grams > 100.0 { ... } // Hardcoded threshold
["IIFL", "Muthoot", "Manappuram"] // Hardcoded competitors
```
**Impact:** Segmentation uses hardcoded gold-loan values.

### 1.4 What Config Exists But Is NOT Used

| Config File | Contains | Code Uses |
|-------------|----------|-----------|
| `tools/sms_templates.yaml` | SMS templates | Hardcoded strings |
| `tools/documents.yaml` | Document checklist | Hardcoded JSON |
| `segments.yaml` | Thresholds | Hardcoded values |
| `slots.yaml` | Purity factors | Hardcoded in customer.rs |

---

## 2. Trait Design & Factory Patterns

### 2.1 Trait Inventory

**Strong Traits (40+ total defined)**

| Category | Traits | Assessment |
|----------|--------|------------|
| Infrastructure | `Tool`, `ConversationFSM`, `LanguageModel`, `Retriever`, `ToolFactory` | Excellent |
| Speech | `SpeechToText`, `TextToSpeech`, `VoiceActivityDetector`, `AudioProcessor` | Excellent |
| Domain | `DomainCalculator`, `SlotSchema`, `ConversationGoalSchema`, `LeadScoringStrategy` | Good |
| Text | `GrammarCorrector`, `Translator`, `PIIRedactor`, `ComplianceChecker` | Excellent |
| Persistence | `SessionStore`, `GoldPriceService`, `AppointmentStore`, `SmsService` | Good |

**Assessment:** Trait design is **EXCELLENT** with 206+ trait object usages.

### 2.2 Factory Pattern Analysis

#### ToolFactory - Well Designed
**File:** `crates/tools/src/domain_tools/factory.rs`
```rust
pub trait ToolFactory: Send + Sync {
    fn create_tool(&self, name: &str) -> Result<Arc<dyn Tool>, ToolError>;
    fn create_all_tools(&self) -> Result<Vec<Arc<dyn Tool>>, ToolError>;
    fn available_tools(&self) -> &[ToolMetadata];
}
```
**Assessment:** Good design but static imports limit extensibility.

#### DomainBridge - Partially Fixed
**File:** `crates/config/src/domain/bridge.rs`
- Lines 58, 94, 160, 217, 252, 295 now return `Arc<dyn Trait>`
- Previously returned `impl Trait` (static dispatch)
- **Status:** Fixed per DOMAIN_AGNOSTIC_REFACTORING.md

#### Agent Initialization - Massive Duplication
**File:** `crates/agent/src/agent/mod.rs` (Lines 120-491)
- Three constructors: `new()`, `with_llm()`, `without_llm()`
- **370 lines of duplicated initialization code**
- Same blocks repeated 3 times:
  - Domain config initialization (lines 127-146)
  - Agentic retriever setup (lines 176-193)
  - Translator creation (lines 206-231)
  - Speculative executor (lines 237-258)

**Recommendation:** Extract to builder pattern:
```rust
pub struct DomainAgentBuilder {
    config: Arc<MasterDomainConfig>,
    llm: Option<Arc<dyn LanguageModel>>,
    retriever: Option<Arc<AgenticRetriever>>,
    // ...
}

impl DomainAgentBuilder {
    pub fn with_llm(mut self, llm: Arc<dyn LanguageModel>) -> Self { ... }
    pub fn build(self) -> Result<DomainAgent, Error> { ... }
}
```

### 2.3 Missing Traits

| Trait Needed | Purpose | Current State |
|--------------|---------|---------------|
| `BranchProvider` | Location data abstraction | Hardcoded in locations.rs |
| `DocumentProvider` | Document requirements | Hardcoded in document_checklist.rs |
| `SmsTemplateProvider` | SMS message templates | Hardcoded in sms.rs |
| `DomainTermProvider` | RAG boosting terms | Hardcoded in domain_boost.rs |
| `SlotExtractor` | Slot extraction from text | Method signature only, no impl |
| `ConfigValidator` | YAML validation at startup | Missing |

### 2.4 Unsafe Code Issue

**File:** `crates/agent/src/fsm_adapter.rs` (Lines 195-197)
```rust
// SAFETY: We just updated this, and RwLock ensures safe access
// This is a workaround for returning a reference to computed data
unsafe { &*(&*self.current_stage.read() as *const CoreStage) }
```
**Impact:** Design debt causing unsafe code to work around trait interface.
**Fix:** Redesign `ConversationFSM::state()` to return owned value or `Arc<T>`.

---

## 3. Configuration Architecture

### 3.1 Current Config Structure

```
config/domains/gold_loan/
├── domain.yaml          (716 lines) - Core business constants
├── slots.yaml           (~250 lines) - DST definitions
├── stages.yaml          (~150 lines) - Conversation flow
├── scoring.yaml         (~200 lines) - Lead scoring
├── objections.yaml      (~100 lines) - Objection handling
├── segments.yaml        (~100 lines) - Customer segments
├── goals.yaml           (~150 lines) - Intent-to-goal mapping
├── features.yaml        (~50 lines) - Feature flags
├── competitors.yaml     (~100 lines) - Competitor data
├── prompts/system.yaml  (~200 lines) - LLM prompts
├── tools/
│   ├── schemas.yaml     (~150 lines) - Tool definitions
│   ├── branches.yaml    (~100 lines) - Location data
│   ├── sms_templates.yaml (~80 lines) - SMS templates (NOT USED!)
│   └── documents.yaml   (~80 lines) - Document checklist (NOT USED!)
└── Total: ~3,242 lines of domain configuration
```

### 3.2 Config Coverage Analysis

| Component | Config Coverage | Code Changes Needed? |
|-----------|-----------------|---------------------|
| Brand info | 100% | No |
| Interest rates | 100% | No |
| LTV/constants | 100% | No |
| Competitors | 100% | No |
| Stage definitions | 100% | No |
| Prompts/templates | 95% | No |
| DST slots | 90% | Minor |
| Tool definitions | 90% | No |
| Objection handling | 90% | No |
| **SMS templates** | **0%** | **YES - hardcoded** |
| **Document checklist** | **0%** | **YES - hardcoded** |
| **Appointment purposes** | **0%** | **YES - hardcoded** |
| **RAG boost terms** | **0%** | **YES - hardcoded** |

### 3.3 Can New Domain Be Onboarded via YAML Only?

**Answer: 65-70% YES (Down from 70-75% in previous review)**

| Works via YAML | Still Requires Code |
|----------------|---------------------|
| Brand/product info | NextBestAction variants |
| Conversation stages | Tool business logic (SMS, docs) |
| Slot definitions | Domain boost terms |
| Objection responses | Slot name mappings |
| Prompts/templates | Intent-to-tool routing |
| Competitor data | Appointment purposes |
| Lead scoring rules | Memory compression priorities |

### 3.4 View Pattern Assessment

**Excellent architecture for config access:**

```rust
// Three specialized views for different crates
pub struct AgentDomainView { ... }  // For agent crate
pub struct LlmDomainView { ... }    // For LLM crate
pub struct ToolsDomainView { ... }  // For tools crate
```

**Issue:** Views are well-designed but tools don't fully use them for SMS/documents.

---

## 4. Code Organization & SRP Violations

### 4.1 Files Requiring Splitting (>500 lines)

| File | Lines | Issues | Recommendation |
|------|-------|--------|----------------|
| `text_processing/src/intent/mod.rs` | 1521 | Intent + slots + numerals | Split into 4 modules |
| `agent/src/dst/slots.rs` | 1377 | 60+ fields, 80+ match arms | HashMap-based storage |
| `server/src/ptt.rs` | 1316 | Audio + STT + markdown | Split by concern |
| `config/src/domain/views.rs` | 1291 | All view classes | One file per view |
| `agent/src/dst/mod.rs` | 1292 | DST + tracker mixed | Separate modules |
| `pipeline/src/orchestrator.rs` | 1233 | VAD+STT+TTS+LLM | Split into 4 modules |
| `llm/src/speculative.rs` | 1147 | Multiple modes | Acceptable |
| `agent/src/memory/mod.rs` | 1179 | AgenticMemory + compression | Split modules |
| `agent/src/agent/mod.rs` | 979 | Struct + init + tests | Extract builder |

### 4.2 GoldLoanDialogueState - 60+ Fields

**File:** `crates/agent/src/dst/slots.rs`

The same 16 slot names are pattern-matched in **5 different methods**:
- `mark_confirmed()` - 16 match arms
- `get_slot_value()` - 16 match arms
- `get_slot_with_confidence()` - 16 match arms
- `set_slot_value()` - 16 match arms
- `clear_slot()` - 16 match arms

**Impact:** 80+ nearly identical match arms.

**Fix:** Use HashMap-based slot storage:
```rust
pub struct DomainDialogueState {
    slots: HashMap<String, SlotValue>,
    metadata: StateMetadata,
}
```

### 4.3 DomainAgent - God Object

**File:** `crates/agent/src/agent/mod.rs`

**Dependencies managed (8 crates!):**
- Configuration
- Conversation context
- Tool execution
- LLM inference
- RAG pipeline
- Personalization
- Translation
- Memory management

**Fix:** Use composition pattern, extract subsystems.

### 4.4 Recommended Crate Extractions

1. **dialogue-state-tracking** - Extract `agent/dst/` (3400+ LOC)
2. **conversation-memory** - Extract `agent/memory/` (3750+ LOC)
3. **lead-scoring** - Extract `agent/lead_scoring.rs`

---

## 5. Concurrency & Performance

### 5.1 Concurrency Assessment

**Strengths:**
- Proper use of `parking_lot::RwLock` (3x faster than std)
- Correct `Arc<dyn Trait>` patterns throughout
- Channel-based event streaming
- No `Mutex<Arc>` anti-patterns

**Issues:**

| Issue | File | Line | Risk |
|-------|------|------|------|
| RwLock<HashMap<HashMap<Vec>>> | slots.rs | 423 | Lock contention |
| `.unwrap()` on locks | locations.rs | 77,84,90 | Panic on poison |
| Lock held during clone | backend.rs | 202,209,363 | Thread blocking |
| Static atomic counter | orchestrator.rs | 805-806 | Race if concurrent |

### 5.2 Performance Concerns

| Category | Count/Issue | Impact |
|----------|-------------|--------|
| Excessive cloning | 792 `.clone()` calls | HIGH |
| Regex recompilation | 8+ locations | MEDIUM |
| String reallocations | 20+ locations | MEDIUM |
| Lock contention in audio loop | orchestrator.rs | MEDIUM |

### 5.3 Highest Impact Fixes

1. **Replace excessive clones** - Use `Arc::clone()` for large contexts
2. **Pre-compile all regex** - Use `Lazy<Regex>` pattern everywhere
3. **Release Mutex locks** - Before `.await` points
4. **Use DashMap** - For high-concurrency HashMap patterns

---

## 6. Crate-by-Crate Analysis

### 6.1 Agent Crate (23 files)

| Aspect | Rating | Notes |
|--------|--------|-------|
| Domain Hardcoding | ⚠️ HIGH | Slot mappings, memory priorities |
| Trait Usage | ✅ GOOD | Uses ConversationContext, DialogueState traits |
| SRP | ⚠️ POOR | agent/mod.rs, dst/mod.rs too large |
| Factory Pattern | ⚠️ POOR | 370-line duplication |

**Key Files:**
- `agent/mod.rs` - Needs builder extraction
- `processing.rs` - Hardcoded slot mappings (lines 92-98)
- `tools.rs` - Hardcoded intent mappings (lines 20-72)
- `memory/compressor.rs` - Hardcoded priorities (lines 51-60)

### 6.2 Core Crate (35 files)

| Aspect | Rating | Notes |
|--------|--------|-------|
| Domain Hardcoding | ⚠️ MEDIUM | customer.rs thresholds |
| Trait Design | ✅ EXCELLENT | 15+ well-designed traits |
| SRP | ✅ GOOD | Clear separation |
| Documentation | ⚠️ FAIR | 40+ undocumented APIs |

**Key Files:**
- `customer.rs` - Hardcoded thresholds (lines 221, 258, 263)
- `domain/traits.rs` - Some deprecated traits still exported

### 6.3 Config Crate (22 files)

| Aspect | Rating | Notes |
|--------|--------|-------|
| Domain Hardcoding | ✅ LOW | Config is properly structured |
| View Pattern | ✅ EXCELLENT | Three specialized views |
| SRP | ⚠️ FAIR | views.rs (1291 lines) too large |
| Config Loading | ✅ GOOD | YAML-driven |

**Key Files:**
- `domain/views.rs` - Should split into separate view files
- `domain/bridge.rs` - Factory pattern fixed

### 6.4 Tools Crate (19 files)

| Aspect | Rating | Notes |
|--------|--------|-------|
| Domain Hardcoding | ⚠️ CRITICAL | SMS, documents, appointments |
| Factory Pattern | ✅ GOOD | ToolFactory implemented |
| SRP | ⚠️ FAIR | Tools do message formatting |
| Registry | ⚠️ FAIR | Hardcoded tool list |

**Critical Files:**
- `tools/sms.rs` - Hardcoded templates (lines 118-165)
- `tools/document_checklist.rs` - Hardcoded docs (lines 84-165)
- `tools/appointment.rs` - Hardcoded purposes (lines 87-94)
- `locations.rs` - Hardcoded branch fallback (lines 77-141)

### 6.5 Pipeline Crate (41 files)

| Aspect | Rating | Notes |
|--------|--------|-------|
| Domain Hardcoding | ✅ NONE | Fully domain-agnostic |
| Trait Usage | ✅ EXCELLENT | VadEngine, SttBackend, TtsBackend |
| SRP | ⚠️ FAIR | orchestrator.rs (1233 lines) |
| Pluggability | ✅ EXCELLENT | Multiple backends |

**Key Files:**
- `orchestrator.rs` - Needs splitting into 4 modules

### 6.6 LLM Crate (8 files)

| Aspect | Rating | Notes |
|--------|--------|-------|
| Domain Hardcoding | ✅ NONE | Fully domain-agnostic |
| Provider Abstraction | ✅ EXCELLENT | Claude, Ollama, OpenAI |
| Streaming | ✅ EXCELLENT | Proper channel handling |
| Tool Calling | ✅ EXCELLENT | Native support |

### 6.7 RAG Crate (18 files)

| Aspect | Rating | Notes |
|--------|--------|-------|
| Domain Hardcoding | ⚠️ HIGH | domain_boost.rs (146 lines) |
| Retrieval Design | ✅ EXCELLENT | Hybrid dense+sparse |
| Query Expansion | ✅ GOOD | Hindi/Hinglish support |
| Agentic Flow | ✅ GOOD | Multi-step retrieval |

**Critical File:**
- `domain_boost.rs` - 146 lines of hardcoded gold loan terms

### 6.8 Text Processing Crate (27 files)

| Aspect | Rating | Notes |
|--------|--------|-------|
| Domain Hardcoding | ✅ LOW | Generic text processing |
| SRP | ⚠️ FAIR | intent/mod.rs (1521 lines) |
| Pipeline Design | ✅ GOOD | Grammar→Translation→PII→Compliance |

**Key File:**
- `intent/mod.rs` - Should split into intent_detection.rs and slot_extraction.rs

### 6.9 Persistence Crate (9 files)

| Aspect | Rating | Notes |
|--------|--------|-------|
| Domain Hardcoding | ⚠️ MEDIUM | GoldPriceService naming |
| Abstraction | ✅ GOOD | Trait-based services |
| ScyllaDB | ✅ GOOD | Production-ready |

### 6.10 Server Crate (12 files)

| Aspect | Rating | Notes |
|--------|--------|-------|
| Domain Hardcoding | ⚠️ MEDIUM | ptt.rs greeting |
| Session Management | ✅ EXCELLENT | Trait-based storage |
| Graceful Shutdown | ✅ GOOD | Proper handling |

---

## 7. Prioritized Action Plan

### Phase 1: Critical Domain Blockers (Week 1)

| Task | Files | Effort | Impact |
|------|-------|--------|--------|
| Wire SMS templates from config | sms.rs | 4h | P0 |
| Wire document checklist from config | document_checklist.rs | 4h | P0 |
| Make appointment purposes config-driven | appointment.rs | 4h | P0 |
| Remove hardcoded branch fallback | locations.rs | 2h | P0 |
| Move domain boost terms to config | domain_boost.rs | 8h | P0 |
| Fix unsafe code in FSM adapter | fsm_adapter.rs | 4h | P0 |

### Phase 2: Slot/Intent Mappings (Week 2)

| Task | Files | Effort | Impact |
|------|-------|--------|--------|
| Make slot name mappings config-driven | processing.rs | 8h | P1 |
| Make intent-to-tool mappings config-driven | tools.rs | 4h | P1 |
| Make memory priorities config-driven | compressor.rs | 4h | P1 |
| Make NextBestAction config-driven | dst/slots.rs | 16h | P1 |
| Rename GoldLoanDialogueState | dst/slots.rs | 4h | P1 |
| Use config thresholds in customer.rs | customer.rs | 4h | P1 |

### Phase 3: Code Quality (Week 3)

| Task | Files | Effort | Impact |
|------|-------|--------|--------|
| Extract DomainAgentBuilder | agent/mod.rs | 8h | P2 |
| Split orchestrator.rs | pipeline/ | 8h | P2 |
| Split intent/mod.rs | text_processing/ | 8h | P2 |
| Split views.rs | config/ | 4h | P2 |
| HashMap-based slot storage | dst/slots.rs | 16h | P2 |
| Pre-compile regex patterns | Multiple | 4h | P2 |

### Phase 4: Architecture (Week 4-5)

| Task | Files | Effort | Impact |
|------|-------|--------|--------|
| Create BranchProvider trait | tools/ | 8h | P2 |
| Create DocumentProvider trait | tools/ | 8h | P2 |
| Create SmsTemplateProvider trait | tools/ | 4h | P2 |
| Create DomainTermProvider trait | rag/ | 8h | P2 |
| Extract DST to separate crate | agent/dst/ | 16h | P3 |
| Extract memory to separate crate | agent/memory/ | 16h | P3 |

---

## 8. Generic Naming Recommendations

### Rename/Abstract These Items

| Current Name | Generic Name | Location |
|--------------|--------------|----------|
| `GoldLoanDialogueState` | `DomainDialogueState` | dst/slots.rs |
| `gold_weight_grams` | `asset_quantity` | Multiple |
| `gold_purity` | `asset_quality_tier` | Multiple |
| `gold_price_per_gram` | `asset_unit_price` | Multiple |
| `calculate_loan_amount` | `calculate_offer_value` | calculator.rs |
| `branch_locator` | `location_finder` | tools/ |
| `gold_loan_tools` | `domain_tools` | tools/ (DONE) |
| `GoldPriceService` | `AssetPriceService` | persistence/ |
| `load_gold_loan_terms` | `load_domain_terms` | domain_boost.rs |

### Config Keys to Generalize

| Gold-Loan Specific | Generic |
|-------------------|---------|
| `interest_rates.gold_loan` | `interest_rates.primary` |
| `ltv_ratio` | `value_ratio` |
| `purity_factors` | `quality_multipliers` |
| `gold_price_per_gram` | `base_price_per_unit` |

---

## Appendix A: Metrics Summary

| Metric | Value |
|--------|-------|
| Total Rust files | 220+ |
| Total lines of code | ~50,000 |
| Crates | 11 |
| Traits defined | 40+ core |
| Trait object usages | 206+ |
| `.clone()` calls | 792 |
| Hardcoded domain refs | 287+ |
| P-FIX comments | 75+ |
| Files >500 lines | 10 |
| Files >1000 lines | 9 |
| YAML config lines | 3,242 |

---

## Appendix B: Files with Most Domain Hardcoding

| File | Hardcoded Items | Priority |
|------|-----------------|----------|
| `rag/src/domain_boost.rs` | 146 lines of terms | P0 |
| `tools/src/domain_tools/tools/sms.rs` | 47 lines of templates | P0 |
| `tools/src/domain_tools/tools/document_checklist.rs` | 81 lines | P0 |
| `agent/src/agent/processing.rs` | Slot mappings | P1 |
| `agent/src/agent/tools.rs` | Intent mappings | P1 |
| `agent/src/memory/compressor.rs` | Priority list | P1 |
| `core/src/customer.rs` | Thresholds | P1 |
| `server/src/ptt.rs` | Greeting | P1 |
| `agent/src/persuasion.rs` | Fallback handlers | P2 |

---

## Appendix C: Domain-Agnostic Verification Checklist

After completing Phase 1-2, verify:

- [ ] No hardcoded "Kotak" strings in Rust files
- [ ] No hardcoded "gold loan" in tool outputs
- [ ] No hardcoded interest rates in Rust files
- [ ] No hardcoded competitor names in Rust files
- [ ] SMS templates loaded from YAML
- [ ] Document checklist loaded from YAML
- [ ] Appointment purposes loaded from YAML
- [ ] Domain boost terms loaded from YAML
- [ ] Slot mappings loaded from YAML
- [ ] Intent-to-tool mappings loaded from YAML
- [ ] `DomainConfigManager` fully deprecated
- [ ] Build passes with `cargo check`
- [ ] All tests pass
- [ ] New domain folder can be created with only YAML files

---

*Generated by comprehensive code review - January 9, 2026*
