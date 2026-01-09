# Voice Agent Backend: Comprehensive Code Review & Domain-Agnostic Refactoring Plan

**Date:** 2026-01-07
**Goal:** Make the codebase truly domain-agnostic, config-driven, and production-ready

---

## Implementation Progress

| Phase | Task | Status |
|-------|------|--------|
| 1.1 | Fix DomainBridge factory pattern (`Arc<dyn Trait>`) | ✅ Complete |
| 1.2 | Clean domain/traits.rs (removed superseded traits) | ✅ Complete |
| 1.3 | Wire customer.rs to use config thresholds | ✅ Complete |
| 1.4-1.6 | Missing traits verification | ✅ Complete (already exist) |
| 2.1 | Deprecate gold-loan factory methods | ✅ Complete |
| 2.2 | Create generic ToolFactory interface | ✅ Complete |
| 3.1 | Split indicconformer.rs into modules | Deferred |
| 3.2 | Consolidate regex patterns into shared module | Deferred |
| 3.3 | Move DST extractor to text_processing crate | Deferred |

### Changes Made (2026-01-07 - Session 2)

**Phase 2.1: Deprecated Gold-Loan Factory Methods (18 total)**

1. **slots.rs** - Deprecated 4 methods:
   - `ConfigSlotDefinition::asset_quantity()`
   - `ConfigSlotDefinition::asset_quality()`
   - `ConfigSlotDefinition::loan_amount()`
   - `ConfigSlotDefinition::phone_number()`

2. **goals.rs** - Deprecated 5 methods:
   - `ConfigGoalDefinition::balance_transfer()`
   - `ConfigGoalDefinition::new_loan()`
   - `ConfigGoalDefinition::eligibility_check()`
   - `ConfigGoalDefinition::branch_visit()`
   - `ConfigGoalDefinition::lead_capture()`

3. **competitors.rs** - Deprecated 8 methods:
   - `CompetitorInfo::muthoot()`, `manappuram()`, `iifl()`, `hdfc()`
   - `CompetitorInfo::sbi()`, `icici()`, `federal()`, `local_jeweler()`

**Phase 2.2: Created ToolFactory Interface**

1. **New trait: `ToolFactory`** (`core/traits/tool_factory.rs`)
   - Domain-agnostic interface for tool creation
   - `create_tool(name)` - create single tool
   - `create_all_tools()` - create all domain tools
   - `create_tools_by_category()` - create by category
   - `ToolFactoryRegistry` - multi-domain factory registry

2. **New implementation: `GoldLoanToolFactory`** (`tools/gold_loan/factory.rs`)
   - Implements `ToolFactory` for gold loan domain
   - Creates all 10 tools with proper metadata
   - Supports optional CRM/calendar integrations

3. **New function: `create_registry_with_factory()`** (`tools/registry.rs`)
   - Factory-based registry creation (PREFERRED method)
   - Domain-agnostic - works with any `ToolFactory` implementation

### Changes Made (2026-01-07 - Session 1)

1. **DomainBridge now returns `Arc<dyn Trait>`** - All 6 factory methods updated for polymorphism
2. **Superseded traits removed from domain/traits.rs** - Added documentation notes explaining migration
3. **New `SegmentationConfig` struct** - Config-driven thresholds for segmentation
4. **`CustomerProfile::infer_segment_with_config()`** - New config-driven method
5. **`SegmentDetector::from_config()`** - New constructor accepting config
6. **`PersonalizationContext::for_profile_with_config()`** - New config-driven method
7. **Deprecated old methods** - `infer_segment()` marked deprecated with migration guidance

---

## Executive Summary

The voice-agent backend has **excellent architectural foundations** with strong separation of concerns. However, there are **critical issues** preventing true domain-agnosticism:

1. **Domain-specific code in core traits** - Gold loan factory methods hardcoded in "generic" traits
2. **Missing trait implementations** - AudioProcessor, Retriever, SlotExtraction are non-functional
3. **Factory pattern issues** - Uses `impl Trait` (static dispatch) instead of `Arc<dyn Trait>` (dynamic)
4. **Code duplication** - Regex patterns duplicated across crates
5. **Monster files** - Several files exceed 1000+ lines with SRP violations

**Target Outcome:** Onboard any business domain by ONLY defining YAML configs.

---

## Part 1: Critical Design Flaws

### 1.1 Factory Pattern Anti-Pattern (CRITICAL)

**File:** `/crates/config/src/domain/bridge.rs`

**Current Implementation (BROKEN):**
```rust
pub fn calculator(&self) -> impl DomainCalculator {
    ConfigDrivenCalculator::new(...)  // Returns CONCRETE type!
}
```

**Problems:**
- Lines 58, 94, 160, 217, 252, 295 all return `impl Trait`
- Cannot swap implementations at runtime
- Cannot use mock implementations in tests
- Cannot serialize/deserialize trait objects
- No polymorphism possible

**Required Fix:**
```rust
pub fn calculator(&self) -> Arc<dyn DomainCalculator> {
    Arc::new(ConfigDrivenCalculator::new(...))
}
```

| Method | Line | Current | Should Be |
|--------|------|---------|-----------|
| `calculator()` | 58 | `impl DomainCalculator` | `Arc<dyn DomainCalculator>` |
| `lead_scoring()` | 94 | `impl LeadScoringStrategy` | `Arc<dyn LeadScoringStrategy>` |
| `competitor_analyzer()` | 160 | `impl CompetitorAnalyzer` | `Arc<dyn CompetitorAnalyzer>` |
| `segment_detector()` | 217 | `impl SegmentDetector` | `Arc<dyn SegmentDetector>` |
| `objection_handler()` | 252 | `impl ObjectionHandler` | `Arc<dyn ObjectionHandler>` |
| `goal_schema()` | 295 | `impl ConversationGoalSchema` | `Arc<dyn ConversationGoalSchema>` |

---

### 1.2 Missing Trait Implementations (CRITICAL)

#### AudioProcessor - DECLARED BUT NOT IMPLEMENTED

**File:** `/crates/core/src/traits/speech.rs:329-343`

```rust
/// P2-4 FIX: Implementation Status
/// This trait is defined but NOT YET IMPLEMENTED.
```

**Missing implementations:**
- Acoustic Echo Cancellation (AEC)
- Noise Suppression (NS)
- Automatic Gain Control (AGC)

**Current Workaround:** `PassthroughAudioProcessor` (no-op) in `/crates/pipeline/src/adapters.rs`

**Impact:** Audio quality issues in production, especially with speaker phone.

#### Retriever - NO IMPLEMENTATIONS

**File:** `/crates/core/src/traits/retriever.rs`

**Status:** Interface only, no concrete implementations found in codebase

**Impact:** RAG system is non-functional

#### SlotSchema::extract_slots() - METHOD NOT IMPLEMENTED

**File:** `/crates/core/src/traits/slots.rs:174-178`

**Status:** Method signature exists, zero implementations anywhere

**Impact:** Cannot extract slots from user utterances via trait system

---

### 1.3 Duplicate Trait Definitions

**ObjectionHandler - DEFINED TWICE:**

| Location | File | Lines | Status |
|----------|------|-------|--------|
| Core Traits | `/crates/core/src/traits/objections.rs` | 136-167 | PROPER: Has ConfigObjectionHandler |
| Domain Legacy | `/crates/core/src/domain/traits.rs` | 111-123 | DEAD CODE: No implementations |

**CustomerSegment vs SegmentDetector:**

| Location | File | Lines | Status |
|----------|------|-------|--------|
| SegmentDetector | `/crates/core/src/traits/segments.rs` | 116-154 | PROPER: Full-featured |
| CustomerSegment | `/crates/core/src/domain/traits.rs` | 95-107 | LEGACY: Remove |

**Action:** Remove dead code from `/crates/core/src/domain/traits.rs`

---

### 1.4 SRP Violations in Traits

#### LeadScoringStrategy (Too Many Concerns)

**File:** `/crates/core/src/traits/scoring.rs:322-359`

```rust
// Currently conflates:
fn calculate_breakdown(&self, ...) -> ScoreBreakdown;  // Scoring logic
fn check_escalation_triggers(&self, ...) -> Vec<...>; // Escalation detection
fn urgency_keywords(&self, ...) -> Vec<&str>;         // Language-specific keywords
fn thresholds(&self) -> ...;                          // Config access
```

**Should split into:**
- `LeadScorer` - score calculation only
- `EscalationDetector` - escalation triggers
- `KeywordProvider` - language keywords

#### ConversationGoalSchema (Too Large)

**File:** `/crates/core/src/traits/goals.rs:146-186`

**Conflating:**
- Goal definitions (lines 73-140)
- Intent mapping (line 159)
- Next-best-action logic (lines 167-171)
- Tool suggestions (line 174)

---

## Part 2: Domain-Specific Code in Core Traits (CRITICAL)

### 2.1 Gold Loan Factory Methods in "Generic" Traits

These factory methods should NOT exist in core traits:

#### Calculator Trait
**File:** `/crates/core/src/traits/calculator.rs`
- Line 131: "For gold loan: weight_grams × price_per_gram × purity_factor"
- `get_quality_factor()` (line 162) - only makes sense for gold purity

#### Slots Trait
**File:** `/crates/core/src/traits/slots.rs:276-402`
- `asset_quantity()` - explicitly for "gold weight"
- `asset_quality()` - hardcoded `K24`, `K22`, `K18`, `K14` (gold karats)
- `loan_amount()` - labeled "Desired loan amount in rupees"

#### Goals Trait
**File:** `/crates/core/src/traits/goals.rs:255-341`
- `balance_transfer()` - gold loan specific
- `new_loan()` - uses "asset_quantity", "asset_quality"
- `eligibility_check()` - gold loan eligibility
- `branch_visit()` - gold loan branches

#### Objections Trait
**File:** `/crates/core/src/traits/objections.rs:216-440`
- Line 234: "my gold", "my gold ornaments"
- Line 273: "gold security concern"
- Line 362: "Muthoot or Manappuram" (gold loan competitors)

#### Competitors Trait
**File:** `/crates/core/src/traits/competitors.rs:133-278`
- `muthoot()` - gold loan NBFC with exact rate 12.0%
- `manappuram()` - gold loan competitor
- `local_jeweler()` - gold loan context

#### Segments Trait
**File:** `/crates/core/src/traits/segments.rs:221-231`
- "100 gram" text pattern (gold specific)
- `asset_quantity` threshold

**Resolution:** Move ALL factory methods to config module. Core traits should only have generic constructors.

---

### 2.2 Hardcoded Thresholds in Customer Module

**File:** `/crates/core/src/customer.rs`

| Line | Hardcoded Value | Description | Config Location |
|------|-----------------|-------------|-----------------|
| 221 | `7500.0` | Gold price per gram | domain.yaml:41 |
| 258 | `100.0` | High-value gold threshold | segments.yaml:14 |
| 263 | `500_000.0` | High-value amount threshold | segments.yaml:16 |
| 235-238 | Purity factors | 22K=0.916, 18K=0.75 | slots.yaml:67-80 |
| 271-273 | NBFC names | IIFL, Muthoot, Manappuram | segments.yaml:62-66 |

**Problem:** Config values exist but aren't used! Code uses hardcoded fallbacks.

**Fix:** Refactor `CustomerProfile::infer_segment()` to accept `SegmentsConfig` parameter.

---

## Part 3: Code Organization Issues

### 3.1 Monster Files (SRP Violations)

| File | Lines | Issues |
|------|-------|--------|
| `/crates/pipeline/src/stt/indicconformer.rs` | 1639 | Combines: audio preprocessing, ONNX backend, Candle backend, mel filterbank, CTC decoding |
| `/crates/text_processing/src/intent/mod.rs` | 1521 | Combines: intent definitions, 50+ regex patterns, Hindi numeral conversion, slot extraction |
| `/crates/agent/src/dst/extractor.rs` | 1480 | DUPLICATE regex patterns (same as intent/mod.rs) |
| `/crates/agent/src/dst/slots.rs` | 1377 | Slot definitions + utilities |
| `/crates/server/src/ptt.rs` | 1316 | Audio processing, STT pooling, markdown stripping, base64 |
| `/crates/pipeline/src/orchestrator.rs` | 1232 | Audio pipeline orchestration |
| `/crates/config/src/settings.rs` | 1129 | Monolithic config |
| `/crates/agent/src/conversation.rs` | 1117 | 939 lines of impl blocks |

**Split Recommendations:**

1. **indicconformer.rs** → 4 files:
   - `ort_backend.rs` - ONNX Runtime implementation
   - `candle_backend.rs` - Candle implementation
   - `mel_filterbank.rs` - Audio preprocessing
   - `decoder.rs` - CTC decoder

2. **intent/mod.rs** → 4 files:
   - `detector.rs` - IntentDetector
   - `patterns.rs` - Regex definitions
   - `indic.rs` - Indic numeral conversion
   - `types.rs` - Intent/Slot types

---

### 3.2 Code Duplication (CRITICAL)

**Regex Patterns Duplicated in TWO Places:**

| Pattern Type | Location 1 | Location 2 |
|--------------|------------|------------|
| Loan amount (crore/lakh) | `agent/dst/extractor.rs:42` | `text_processing/intent/mod.rs:307` |
| Weight patterns (grams) | `agent/dst/extractor.rs` | `text_processing/intent/mod.rs` |
| Phone patterns | `agent/dst/extractor.rs` | `text_processing/intent/mod.rs` |
| Hindi multipliers | `agent/dst/extractor.rs` | `text_processing/intent/mod.rs` |

**Fix:** Create shared `PatternRegistry` in `text_processing` crate.

---

### 3.3 Wrong Crate Boundaries

| Code | Current Location | Should Be |
|------|------------------|-----------|
| Slot extraction (pure text processing) | `agent/src/dst/extractor.rs` | `text_processing/src/slots/extractor.rs` |
| DST patterns | `agent/src/dst/` | `text_processing/src/slots/` |
| Gold loan tools | `tools/src/gold_loan/` | Config-driven generic tools |

---

### 3.4 Performance Issues

#### Regex Compilation at Runtime
**File:** `/crates/text_processing/src/intent/mod.rs:110-111`

```rust
// ANTI-PATTERN: Compiles 50+ regex in constructor
pub fn new(config: IntentConfig) -> Self {
    let slot_patterns = compile_slot_patterns(); // Called every time!
}
```

**Fix:** Use `Lazy<Regex>` pattern (like in `agent/dst/extractor.rs`).

#### Excessive String Allocations
- **2567 instances of `.to_string()`** across codebase
- Many could use `Cow<str>` or references

#### Arc Overuse
**File:** `/crates/agent/src/agent/mod.rs:81-99`

```rust
// 6 separate Arc fields - each adds atomic overhead
pub(crate) conversation: Arc<dyn ConversationContext>,
pub(crate) tools: Arc<ToolRegistry>,
pub(crate) llm: Option<Arc<dyn LanguageModel>>,
pub(crate) agentic_retriever: Option<Arc<AgenticRetriever>>,
pub(crate) vector_store: Option<Arc<VectorStore>>,
pub(crate) translator: Option<Arc<dyn Translator>>,
```

**Fix:** Consider `Arc<DomainAgentInner>` with all fields inside.

---

## Part 4: Missing Abstractions

### 4.1 Traits That Should Exist

| Trait | Purpose | Current Status |
|-------|---------|----------------|
| `ToolFactory` | Plugin architecture for tools | Missing - hard-wired in registry |
| `IntentClassifier` | ML-based intent classification | Missing - only regex-based |
| `IntentToGoalMapper` | Pluggable intent→goal mapping | Embedded in GoalSchema |
| `SlotValidator` | Separate validation logic | Embedded in SlotDefinition |
| `SlotExtractor` | Separate extraction logic | Not implemented |
| `ConfigValidator` | Validate YAML configs at startup | Missing |

### 4.2 Missing Feature Flags

| Behavior | Current Status | Recommendation |
|----------|----------------|----------------|
| Balance transfer mode | Hardcoded in DST | `balance_transfer_enabled` flag |
| High-value detection | Always on | `segment_detection.high_value_enabled` |
| Competitor comparison | Always on | `competitor_comparison_enabled` |
| Shakti Gold program | Hardcoded | `shakti_gold_program_enabled` |

---

## Part 5: Trait Inventory

### All Traits Defined (42 Total)

**Core Infrastructure (6):**
- `LanguageModel`, `SpeechToText`, `TextToSpeech`, `VoiceActivityDetector`, `AudioProcessor`, `Retriever`

**Text Processing (5):**
- `GrammarCorrector`, `Translator`, `PIIRedactor`, `ComplianceChecker`, `TextProcessor`

**Conversation (3):**
- `ConversationFSM`, `FrameProcessor`, `Tool`

**Domain-Agnostic (9):**
- `DomainCalculator`, `SlotSchema`, `ConversationGoalSchema`, `SegmentDetector`
- `ObjectionHandler`, `LeadScoringStrategy`, `CompetitorAnalyzer`, `SlotDefinition`, `GoalDefinition`

**Config-Driven Implementations (8):**
- `ConfigDrivenCalculator`, `ConfigSlotDefinition`, `ConfigGoalDefinition`, `ConfigGoalSchema`
- `ConfigSegmentDetector`, `ConfigObjectionHandler`, `ConfigLeadScoring`, `ConfigCompetitorAnalyzer`

**Persistence (5):**
- `SessionStore`, `GoldPriceService`, `AppointmentStore`, `SmsService`, `AuditLog`

**Tools (4):**
- `ToolExecutor`, `CrmIntegration`, `CalendarIntegration`, `ResourceProvider`

**Legacy/Duplicate (2):**
- `CustomerSegment` (duplicate of SegmentDetector)
- `ObjectionHandler` (domain) - duplicate

---

## Part 6: Refactoring Roadmap

### Phase 1: Critical Fixes (Week 1)

#### P1.1 Fix DomainBridge Factory Pattern
**Files:** `/crates/config/src/domain/bridge.rs`

Change all methods to return `Arc<dyn Trait>`:
```rust
pub fn calculator(&self) -> Arc<dyn DomainCalculator> {
    Arc::new(ConfigDrivenCalculator::new(...))
}
```

#### P1.2 Remove Duplicate Traits
**File:** `/crates/core/src/domain/traits.rs`

Delete:
- `CustomerSegment` trait (use SegmentDetector)
- `ObjectionHandler` trait (use core/traits version)

#### P1.3 Move Config Values to Runtime
**File:** `/crates/core/src/customer.rs`

Refactor `infer_segment()` to accept `SegmentsConfig`:
```rust
pub fn infer_segment(&self, config: &SegmentsConfig) -> Option<CustomerSegment> {
    let threshold = config.high_value_threshold; // From config, not hardcoded
}
```

### Phase 2: Domain Decoupling (Week 2)

#### P2.1 Move Domain Factory Methods
Move all gold-loan-specific factory methods from core traits to config module:

| From | To |
|------|-----|
| `traits/slots.rs::asset_quantity()` | `config/domain/slots.rs` |
| `traits/goals.rs::balance_transfer()` | `config/domain/goals.rs` |
| `traits/competitors.rs::muthoot()` | Load from YAML only |
| `traits/objections.rs::safety()` | Load from YAML only |
| `traits/segments.rs::high_value()` | Load from YAML only |

#### P2.2 Create Generic Tool Interface
**File:** `/crates/tools/src/lib.rs`

```rust
pub trait ToolFactory: Send + Sync {
    fn create_tool(&self, name: &str, config: &ToolConfig) -> Option<Arc<dyn Tool>>;
    fn available_tools(&self) -> Vec<&str>;
}
```

### Phase 3: Code Cleanup (Week 3)

#### P3.1 Split Monster Files
- `indicconformer.rs` → 4 modules
- `intent/mod.rs` → 4 modules
- `extractor.rs` → merge with text_processing

#### P3.2 Consolidate Patterns
Create `/crates/text_processing/src/patterns/`:
- `currency.rs` - Amount patterns
- `phone.rs` - Phone patterns
- `indic.rs` - Indic numeral utilities

#### P3.3 Fix DST Crate Boundary
Move `/crates/agent/src/dst/extractor.rs` → `/crates/text_processing/src/slots/`

### Phase 4: Implementation Gaps (Week 4)

#### P4.1 Implement AudioProcessor
Choose implementation: `rnnoise-c` or `webrtc-audio-processing-sys`

#### P4.2 Implement Retriever Trait
Create concrete implementation for RAG system

#### P4.3 Implement extract_slots()
Wire `SlotSchema::extract_slots()` to pattern-based extraction

### Phase 5: Advanced (Future)

- Split LeadScoringStrategy into smaller traits
- Add ConfigValidator for YAML validation at startup
- Create DI container for service location
- Add comprehensive mock implementations for testing

---

## Part 7: File Reference Quick Look

### Files to Modify (By Priority)

**Critical (Week 1):**
| File | Line(s) | Issue |
|------|---------|-------|
| `config/src/domain/bridge.rs` | 58-331 | Change to Arc<dyn Trait> |
| `core/src/customer.rs` | 221, 258, 263 | Use config thresholds |
| `core/src/domain/traits.rs` | ALL | Remove duplicate traits |

**High (Week 2):**
| File | Line(s) | Issue |
|------|---------|-------|
| `core/src/traits/slots.rs` | 276-402 | Move factory methods |
| `core/src/traits/goals.rs` | 255-341 | Move factory methods |
| `core/src/traits/competitors.rs` | 133-278 | Move factory methods |
| `core/src/traits/objections.rs` | 216-440 | Move factory methods |

**Medium (Week 3):**
| File | Lines | Issue |
|------|-------|-------|
| `pipeline/src/stt/indicconformer.rs` | 1639 | Split into modules |
| `text_processing/src/intent/mod.rs` | 1521 | Split into modules |
| `agent/src/dst/extractor.rs` | 1480 | Move to text_processing |

---

## Part 8: Config Structure (For Reference)

### Current Domain Config Files
```
config/domains/gold_loan/
├── domain.yaml         # Business constants, vocabulary
├── features.yaml       # Feature definitions per segment
├── goals.yaml          # Conversation goals
├── segments.yaml       # Segment thresholds
├── slots.yaml          # Slot definitions with patterns
├── stages.yaml         # Conversation stages
├── scoring.yaml        # Lead scoring rules
├── objections.yaml     # Objection handling
├── competitors.yaml    # Competitor info
└── prompts/
    └── system.yaml     # LLM system prompts
```

### Values in Config But NOT Used in Code

| Config File | Value | Code Location | Issue |
|-------------|-------|---------------|-------|
| `segments.yaml:14-17` | numeric_thresholds | `customer.rs:258,263` | Hardcoded instead |
| `segments.yaml:62-66` | current_lender values | `customer.rs:271-273` | Hardcoded instead |
| `slots.yaml:67-80` | purity_factors | `customer.rs:235-238` | Hardcoded instead |

---

## Conclusion

The architecture is fundamentally sound but suffers from **domain leakage** into core traits. The primary work is:

1. **Factory pattern fix** - Enable polymorphism via `Arc<dyn Trait>`
2. **Move domain code** - Factory methods from traits to config module
3. **Use existing configs** - Stop hardcoding values that are already in YAML
4. **Split large files** - Improve maintainability
5. **Implement missing traits** - AudioProcessor, Retriever, extract_slots()

After this refactoring, onboarding a new domain will require ONLY:
```
config/domains/{new_domain}/
├── domain.yaml
├── goals.yaml
├── slots.yaml
└── ...
```

No Rust code changes needed.
