# Domain-Agnostic Architecture Comprehensive Audit

**Date:** 2026-01-09
**Analysis Scope:** voice-agent/backend codebase
**Goal:** Ensure all domain-specific content (gold loan, Kotak, banking, etc.) is behind proper abstractions and config-driven

---

## Executive Summary

| Category | Status | Completion |
|----------|--------|------------|
| **Overall Domain Agnosticism** | GOOD | ~85% |
| **Trait Abstractions** | EXCELLENT | ~90% |
| **Config Loading Infrastructure** | EXCELLENT | ~95% |
| **Config Wiring (Actual Usage)** | NEEDS WORK | ~70% |
| **Lead Scoring** | CRITICAL GAP | ~40% |
| **Compliance Rules** | NOT WIRED | ~30% |
| **Text Processing Patterns** | PARTIAL | ~60% |
| **Prompt Templates** | NEEDS WORK | ~50% |

---

## 1. CRITICAL GAPS REQUIRING IMMEDIATE ATTENTION

### 1.1 Lead Scoring Weights & Thresholds (CRITICAL)

**Location:** `voice-agent/backend/crates/agent/src/lead_scoring.rs`

All scoring logic is hardcoded with magic numbers:

```rust
// Lines 38-56: Qualification thresholds
LeadQualification::Warm => 30,        // HARDCODED
LeadQualification::Hot => 60,         // HARDCODED
LeadQualification::Qualified => 80,   // HARDCODED

// Lines 415-426: Default config
max_objections_before_escalate: 3,    // HARDCODED
max_stalled_turns: 5,                 // HARDCODED
high_value_loan_threshold: 1_000_000.0, // HARDCODED (10 lakh)

// Lines 806-876: All point values
if signals.has_urgency_signal { score += 10; }           // +10 HARDCODED
score += signals.urgency_keywords_count.min(3) * 5;      // max 3 × 5 HARDCODED
if signals.provided_contact_info { score += 8; }         // +8 HARDCODED
if signals.expressed_intent_to_proceed { score += 15; }  // +15 HARDCODED
if signals.expressed_disinterest { p -= 15; }            // -15 HARDCODED
```

**Required Fix:** Move all to `lead_scoring.yaml`:
```yaml
scoring_weights:
  urgency_signal_bonus: 10
  urgency_keyword_max: 3
  urgency_keyword_multiplier: 5
  contact_info_bonus: 8
  intent_bonus: 15
  disinterest_penalty: -15
qualification_thresholds:
  cold_max: 29
  warm_max: 59
  hot_max: 79
```

---

### 1.2 DomainBridge Not Wired (CRITICAL)

**Location:** `voice-agent/backend/crates/config/src/domain/bridge.rs`

The elaborate DomainBridge pattern exists with methods like:
- `calculator()` → Returns `Arc<dyn DomainCalculator>`
- `competitor_analyzer()` → Returns `Arc<dyn CompetitorAnalyzer>`
- `lead_scoring()` → Returns `Arc<dyn LeadScoringStrategy>`

**Problem:** These are NEVER called in production code. Only tests use them.

**Evidence:**
```bash
grep -r "DomainBridge" --include="*.rs" | grep -v test | grep -v "//"
# Result: Only exports and definitions, no actual usage
```

**Required Fix:** Wire DomainBridge into agent creation in `agent/src/agent/mod.rs`

---

### 1.3 Agent Creation Uses Defaults (CRITICAL)

**Location:** `voice-agent/backend/crates/agent/src/agent/mod.rs:127-129`

```rust
// P15 FIX: Create domain config first, used for tools and persona
let domain_config = Arc::new(voice_agent_config::MasterDomainConfig::default());
let agent_view = Arc::new(voice_agent_config::AgentDomainView::new(domain_config.clone()));
```

**Problem:** Agent always uses `MasterDomainConfig::default()` instead of loaded config from AppState.

**Impact:** All domain-specific configuration is ignored at runtime.

**Required Fix:** Pass `Arc<MasterDomainConfig>` to `DomainAgent::new()` from AppState.

---

### 1.4 Compliance Config Not Loaded (CRITICAL)

**Locations:**
- Config: `voice-agent/backend/config/domains/gold_loan/compliance.yaml` (EXISTS, well-structured)
- Struct: `voice-agent/backend/crates/config/src/domain/master.rs` (NO compliance field)
- Runtime: `voice-agent/backend/crates/text_processing/src/compliance/rules.rs:133-193` (HARDCODED defaults used)

**Current Flow (Broken):**
```
compliance.yaml (perfect) → [NOT LOADED] → MasterDomainConfig (no field) → default_rules() HARDCODED
```

**Hardcoded in rules.rs:**
- Forbidden phrases: "guaranteed approval", "100% approval", "zero interest"
- Rate rules: min_rate: 7.0%, max_rate: 24.0%
- Disparaging words: "bad", "worst", "fraud", "cheat", "scam"

**Required Fix:**
1. Add `pub compliance: ComplianceConfig` to `MasterDomainConfig`
2. Load from `config/domains/{domain_id}/compliance.yaml`
3. Wire into TextProcessingPipeline creation

---

## 2. HIGH PRIORITY GAPS

### 2.1 Hardcoded Entity Patterns in Text Processing

**Location:** `voice-agent/backend/crates/text_processing/src/slot_extraction/mod.rs`

```rust
// Lines 189-194: Karat-specific patterns (gold-only)
static PURITY_24K: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)24\s*(?:k|karat|carat|kt)").unwrap());
static PURITY_22K: Lazy<Regex> = ...
static PURITY_18K: Lazy<Regex> = ...
static PURITY_PURE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)pure\s*gold").unwrap());

// Lines 737-745: Hardcoded Indian cities only
let cities = ["mumbai", "delhi", "bangalore", "bengaluru", "chennai", ...];

// Lines 695-719: Hardcoded loan purposes
(vec!["medical", "hospital", "treatment", ...], "medical"),
(vec!["education", "school", "college", ...], "education"),
```

**Required Fix:** Move all patterns to slots.yaml with:
```yaml
extraction_patterns:
  quality_tiers:
    - pattern: "(?i)24\\s*(?:k|karat|carat|kt)"
      value: "tier_1"
    - pattern: "(?i)22\\s*(?:k|karat|carat|kt)"
      value: "tier_2"
  locations:
    patterns: ["mumbai", "delhi", ...]  # Config-driven
```

---

### 2.2 Entity Quality Tier Validation Hardcoded

**Location:** `voice-agent/backend/crates/text_processing/src/entities/mod.rs:452-457`

```rust
// Hardcoded karat range (10-24) - only works for gold
if (10..=24).contains(&tier) {
    Some(tier)
} else {
    None
}
```

**Also hardcoded unit conversions (lines 393-398):**
```rust
// 1 tola = 11.66 grams (gold/jewelry specific)
```

**Required Fix:** Add to domain.yaml:
```yaml
collateral_config:
  quality_tier_range:
    min: 10
    max: 24
  unit_conversions:
    tola_to_grams: 11.66
```

---

### 2.3 Prompt Templates Have Hardcoded Content

**Location:** `voice-agent/backend/crates/llm/src/prompt.rs`

**Lines 540-547 (deprecated but still used):**
```rust
let guidance = match stage {
    "greeting" => "Focus on warm welcome and understanding the customer's needs.",
    "qualification" => "Gather information about collateral and loan requirements.", // "collateral" domain-specific
    "closing" => "Guide toward next steps and branch visit.", // "branch visit" domain-specific
    _ => "Be helpful and professional.",
};
```

**Lines 399-408 (key facts format):**
```rust
"- Interest rates: Starting from {:.1}% p.a. (vs {:.0}-{:.0}% competitor rates)\n\
 - LTV: Up to {:.0}% of collateral value\n\
 - Processing: Same-day disbursement\n\   // HARDCODED
 - Regulated financial institution with secure storage",  // HARDCODED
```

**Location:** `voice-agent/backend/crates/agent/src/agent/response.rs:463-518`

```rust
ConversationStage::Qualification => {
    if is_english {
        "What interest rate are you paying?".to_string()  // domain-specific
    } else {
        "Aapke paas kitna gold pledged hai abhi?"  // HARDCODED "gold pledged"
    }
},
ConversationStage::Closing => {
    "Shall I schedule an appointment for you? Visit nearest branch for gold valuation."
    // HARDCODED "branch", "gold valuation"
}
```

**Required Fix:** Move all to `prompts/system.yaml`:
```yaml
stage_fallback_responses:
  qualification:
    en: "Could you tell me about your current {{product_type}} situation?"
    hi: "Aap apni {{product_type}} ki sthiti ke baare mein batayein?"
key_facts_template: |
  - Interest rates: Starting from {{min_rate}}% p.a.
  - {{collateral_benefit}}: Up to {{ltv}}% of {{collateral_name}} value
  - Processing: {{processing_time}}
  - {{trust_statement}}
```

---

### 2.4 New Traits Not Fully Wired

**Location:** `voice-agent/backend/crates/core/src/traits/`

These NEW trait files exist but are NOT fully integrated:

| File | Status | Issue |
|------|--------|-------|
| `feature_provider.rs` | NEW (untracked) | Not wired into agent |
| `lead_classifier.rs` | NEW (untracked) | Not wired into DomainBridge |
| `objection_provider.rs` | NEW (untracked) | Not wired into persuasion engine |
| `tool_arguments.rs` | NEW (untracked) | Partially used |

**Required Fix:** Complete wiring of all P20 traits into production code paths.

---

### 2.5 adaptation.yaml Not Loaded

**Location:** `voice-agent/backend/config/domains/gold_loan/adaptation.yaml`

This file is NEW (shown in git status as `??`) and defines:
- Variable substitution mappings
- Segment-specific adaptations
- Personalization rules

**Problem:** Not loaded by `MasterDomainConfig::load()` - completely orphaned.

**Required Fix:** Add loading logic in master.rs similar to other optional configs.

---

## 3. MEDIUM PRIORITY GAPS

### 3.1 Hardcoded String Literals "gold_loan"

**Multiple Locations:**

```rust
// conversation.rs:41,43
.with_slot("loan_type", "gold_loan")
Some(&"gold_loan".to_string())

// core/src/traits/retriever.rs
.with_filter(MetadataFilter::eq("category", "gold_loan"))
```

**Required Fix:** Use `domain_context.domain_id()` or config value.

---

### 3.2 Views Missing Methods for Config Values

**Location:** `voice-agent/backend/crates/config/src/domain/views.rs`

Config values loaded but NOT exposed via view methods:
- `query_expansion` - Loaded in master.rs but no view method
- `domain_boost` - Loaded but not exposed
- `relevance_terms` - Loaded but never accessed

**Required Fix:** Add getter methods to `ToolsDomainView` and `AgentDomainView`.

---

### 3.3 Customer.rs Hardcoded Thresholds

**Location:** `voice-agent/backend/crates/core/src/customer.rs:348-349`

```rust
const DEFAULT_COLLATERAL_THRESHOLD: f64 = 100.0;  // grams - HARDCODED
const DEFAULT_AMOUNT_THRESHOLD: f64 = 500_000.0;  // INR (5 lakhs) - HARDCODED
```

**Required Fix:** Use `domain.yaml` `high_value` configuration instead.

---

### 3.4 Intent Patterns Static

**Location:** `voice-agent/backend/crates/text_processing/src/slot_extraction/mod.rs:152-186`

```rust
static INTENT_PATTERNS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| vec![
    (Regex::new(...), "balance_transfer"),
    (Regex::new(...), "price_inquiry"),
    // Intent IDs hardcoded
]);
```

**Required Fix:** Load from `intents.yaml` at runtime, not compile-time static.

---

## 4. WHAT'S WORKING WELL (Preserve)

### 4.1 Excellent Trait Architecture

All core traits are domain-agnostic:
- `LanguageModel` - Complete LLM abstraction
- `Tool` - Generic MCP-compatible interface
- `SpeechToText` / `TextToSpeech` - Pure audio I/O
- `Retriever` - Generic document retrieval
- `SlotSchema` - Fully config-driven slots
- `ConversationGoalSchema` - All goals from config
- `SegmentDetector` - Config-driven detection
- `ObjectionHandler` - ACRE framework generic
- `CompetitorAnalyzer` - Generic comparison

### 4.2 Config Loading Infrastructure

The `MasterDomainConfig::load()` properly handles:
- Base config (optional): `config/base/defaults.yaml`
- Domain config (required): `config/domains/{domain_id}/domain.yaml`
- 15+ specialized configs with fallback error handling

### 4.3 Domain-Agnostic Slot Names

Successfully renamed:
- `gold_weight` → `collateral_weight` (with backward-compatible alias)
- `gold_purity` → `collateral_variant` (with alias)
- `current_lender` → `current_provider` (with alias)

### 4.4 Brand Variable Substitution

Working substitution in prompts:
- `{agent_name}`, `{company_name}`, `{product_name}`, `{helpline}`
- `{brand.bank_name}`, `{brand.product_name}`
- `{{variable}}` template syntax

---

## 5. CONFIGURATION FILES STATUS

### 5.1 Files Status Matrix

| File | Loaded | Wired | Content Status |
|------|--------|-------|----------------|
| `domain.yaml` | YES | YES | Good - constants configurable |
| `slots.yaml` | YES | YES | Good - generic slot names |
| `features.yaml` | YES | PARTIAL | Good - template variables |
| `intents.yaml` | YES | YES | Excellent - domain-agnostic |
| `intent_tool_mappings.yaml` | YES | YES | Excellent - generic structure |
| `lead_scoring.yaml` | YES | PARTIAL | Good structure, code ignores weights |
| `objections.yaml` | YES | YES | Good - ACRE framework |
| `stages.yaml` | YES | YES | Medium - has hardcoded guidance |
| `prompts/system.yaml` | YES | PARTIAL | Poor - 70% hardcoded content |
| `adaptation.yaml` | NO | NO | Not loaded at all |
| `compliance.yaml` | NO | NO | Not loaded at all |
| `competitors.yaml` | YES | YES | Good |
| `segments.yaml` | YES | YES | Good |

### 5.2 Missing Config Fields in MasterDomainConfig

```rust
// These should be added to MasterDomainConfig
pub compliance: ComplianceConfig,     // NOT PRESENT
pub adaptation: AdaptationConfig,     // NOT PRESENT
```

---

## 6. RECOMMENDED REFACTORING PLAN

### Phase 1: Critical Wiring (Highest Priority)

1. **Wire Loaded Config to Agent**
   - Modify `DomainAgent::new()` to accept `Arc<MasterDomainConfig>`
   - Pass from AppState in server initialization
   - Remove `MasterDomainConfig::default()` calls

2. **Load Compliance Config**
   - Add `compliance: ComplianceConfig` field
   - Load `compliance.yaml` in `MasterDomainConfig::load()`
   - Wire into TextProcessingPipeline

3. **Externalize Lead Scoring Weights**
   - Add `scoring_weights` section to `lead_scoring.yaml`
   - Modify `calculate_*_score()` functions to use config
   - Remove all hardcoded point values

### Phase 2: Pattern Externalization (High Priority)

4. **Config-Driven Text Patterns**
   - Move quality tier patterns to slots.yaml
   - Move city lists to domain.yaml
   - Move purpose patterns to config

5. **Dynamic Intent Patterns**
   - Change from static to runtime-loaded
   - Load from intents.yaml at pipeline creation

6. **Prompt Template Separation**
   - Move all fallback responses to prompts/system.yaml
   - Use variable substitution everywhere
   - Remove deprecated `with_stage_guidance()`

### Phase 3: Complete Integration (Medium Priority)

7. **Wire DomainBridge**
   - Use `bridge.calculator()` in tools
   - Use `bridge.competitor_analyzer()` in persuasion
   - Use `bridge.lead_scoring()` in agent

8. **Load adaptation.yaml**
   - Add field to MasterDomainConfig
   - Wire into personalization engine

9. **Complete New Traits**
   - Finish FeatureProvider integration
   - Wire LeadClassifier
   - Wire ObjectionProvider

### Phase 4: Polish (Lower Priority)

10. **Remove String Literals**
    - Replace all `"gold_loan"` with config values
    - Use domain_context.domain_id()

11. **Add View Methods**
    - Expose query_expansion
    - Expose domain_boost
    - Expose relevance_terms

12. **Test Coverage**
    - Update tests to load from config instead of hardcoding
    - Add integration tests for domain switching

---

## 7. FILES REQUIRING CHANGES

### Rust Source Files

| File | Changes Needed | Priority |
|------|----------------|----------|
| `crates/config/src/domain/master.rs` | Add compliance, adaptation loading | P1 |
| `crates/agent/src/agent/mod.rs` | Accept config parameter, remove defaults | P1 |
| `crates/agent/src/lead_scoring.rs` | Use config for all weights/thresholds | P1 |
| `crates/text_processing/src/compliance/rules.rs` | Use loaded config | P1 |
| `crates/text_processing/src/slot_extraction/mod.rs` | Config-driven patterns | P2 |
| `crates/text_processing/src/entities/mod.rs` | Config-driven validation ranges | P2 |
| `crates/text_processing/src/intent/mod.rs` | Runtime-loaded patterns | P2 |
| `crates/llm/src/prompt.rs` | Remove hardcoded guidance | P2 |
| `crates/agent/src/agent/response.rs` | Use config for fallbacks | P2 |
| `crates/config/src/domain/views.rs` | Add missing getter methods | P3 |
| `crates/config/src/domain/bridge.rs` | Wire into production code | P3 |
| `crates/core/src/customer.rs` | Use config thresholds | P3 |

### Config Files

| File | Changes Needed | Priority |
|------|----------------|----------|
| `lead_scoring.yaml` | Add scoring_weights section | P1 |
| `compliance.yaml` | Already complete, needs loading | P1 |
| `adaptation.yaml` | Already complete, needs loading | P2 |
| `slots.yaml` | Add extraction_patterns section | P2 |
| `domain.yaml` | Add collateral_config section | P2 |
| `prompts/system.yaml` | Add stage_fallback_responses | P2 |

---

## 8. VERIFICATION CHECKLIST

After implementing fixes, verify:

- [ ] `cargo build` succeeds with no warnings about domain-specific terms
- [ ] `cargo test` passes with config-driven test data
- [ ] Agent initializes with loaded MasterDomainConfig (not defaults)
- [ ] Lead scores use config weights (change yaml, see score change)
- [ ] Compliance rules load from compliance.yaml
- [ ] Text processing uses config patterns (not hardcoded)
- [ ] Prompts use variable substitution (not hardcoded text)
- [ ] Can create a new domain folder and have system work with ONLY yaml changes

---

## 9. DOMAIN ONBOARDING REQUIREMENTS

After all fixes, onboarding a new domain (e.g., "car_loan") should require ONLY:

1. Create `config/domains/car_loan/` folder
2. Copy and modify yaml files:
   - `domain.yaml` - product name, rates, constants
   - `slots.yaml` - slot definitions for car-specific fields
   - `features.yaml` - product features
   - `objections.yaml` - car loan objections
   - `stages.yaml` - conversation flow
   - `prompts/system.yaml` - persona and guidance
   - `competitors.yaml` - car loan competitors
   - `compliance.yaml` - automotive lending regulations

3. NO CODE CHANGES REQUIRED

Current state: ~15-20 code changes still needed for new domain
Target state: 0 code changes needed

---

## 10. SUMMARY

The voice-agent backend has made excellent architectural progress toward domain-agnosticism (~85% complete). The trait system is well-designed and configuration infrastructure is solid.

**Critical gaps blocking true domain-agnosticism:**
1. Agent uses `MasterDomainConfig::default()` - config never reaches runtime
2. Lead scoring weights are 100% hardcoded - ignores config
3. Compliance config not loaded - uses hardcoded rules
4. Text processing patterns are static - not config-driven

**Estimated effort to achieve 100% domain-agnosticism:**
- Phase 1 (Critical): 2-3 days
- Phase 2 (Patterns): 2-3 days
- Phase 3 (Integration): 1-2 days
- Phase 4 (Polish): 1 day

**Total: ~6-9 development days**

After completing all phases, onboarding a new business domain will require ONLY yaml configuration files with zero code changes.
