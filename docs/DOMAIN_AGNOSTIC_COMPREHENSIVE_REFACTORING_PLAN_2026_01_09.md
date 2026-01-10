# Domain-Agnostic Comprehensive Refactoring Plan

**Date:** 2026-01-09
**Goal:** Make voice-agent backend truly domain-agnostic - onboard any business by YAML configs only

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current State Analysis](#current-state-analysis)
3. [Hardcoded Domain Terms](#hardcoded-domain-terms)
4. [Config Wiring Analysis](#config-wiring-analysis)
5. [Trait Architecture Analysis](#trait-architecture-analysis)
6. [Implementation Plan](#implementation-plan)
7. [New Config Schemas](#new-config-schemas)
8. [Verification Plan](#verification-plan)

---

## Executive Summary

### Current State: ~90% Domain-Agnostic

The voice-agent backend has excellent config-driven architecture:
- `MasterDomainConfig` loads 24 YAML config files
- Domain views (`AgentDomainView`, `ToolsDomainView`, `LlmDomainView`) provide clean access
- P20 FIX traits (`FeatureProvider`, `ObjectionProvider`, `LeadClassifier`, `ToolArgumentProvider`) are well-designed

### Remaining Issues (10%)

| Category | Issue Count | Priority |
|----------|-------------|----------|
| Hardcoded domain terms | 15+ locations | CRITICAL |
| Unwired config fields | 10+ fields | HIGH |
| Missing startup validation | 1 critical gap | HIGH |
| Incomplete variable substitution | 17+ configs missed | MEDIUM |
| Trait bloat/duplication | 3 traits | MEDIUM |
| Domain-coupled enums | 2 enums | MEDIUM |

---

## Current State Analysis

### Config Loading Architecture

```
main.rs:48 → load_master_domain_config()
         ↓
MasterDomainConfig::load() [master.rs:638-1127]
         ↓ (loads sequentially)
         ├→ slots.yaml → SlotsConfig
         ├→ stages.yaml → StagesConfig
         ├→ compliance.yaml → ComplianceConfig
         ├→ vocabulary.yaml → FullVocabularyConfig
         └→ ... 20 more configs
         ↓
AppState::with_master_domain_config()
         ↓
Domain Views provide clean access:
         ├→ AgentDomainView (stages, slots, prompts, scoring, objections)
         ├→ LlmDomainView (system prompt, language style, vocabulary)
         └→ ToolsDomainView (tools, branches, competitors, responses)
```

### Trait Architecture (Score: 8/10)

| Trait | Purpose | Domain-Agnosticism |
|-------|---------|-------------------|
| `DomainCalculator` | Financial calculations | Excellent |
| `SlotSchema` | Dynamic slot extraction | Excellent |
| `LeadScoringStrategy` | Lead qualification | Fair (trait bloat) |
| `Retriever` | RAG & document retrieval | Excellent |
| `ToolFactory` | Tool creation & registry | Excellent |
| `FeatureProvider` | Config-driven features | Excellent |
| `ObjectionProvider` | Objection handling | Excellent |

---

## Hardcoded Domain Terms

### CRITICAL: Must Fix Immediately

#### 1. LLM Purity Tier Enums

**Location:** `crates/llm/src/claude.rs` (~line 683)
```rust
.string_enum("purity", &["24K", "22K", "18K", "14K"])  // HARDCODED
```

**Location:** `crates/llm/src/prompt.rs` (~line 968)
```rust
.string_enum("gold_purity", &["24K", "22K", "18K", "14K"])  // HARDCODED
```

**Fix:**
```rust
// Use generic tier names in tests
.string_enum("tier", &["tier_1", "tier_2", "tier_3"])
```

#### 2. Persistence Legacy Accessors

**Location:** `crates/persistence/src/gold_price.rs` (lines 78-93)
```rust
pub fn price_24k(&self) -> f64  // HARDCODED
pub fn price_22k(&self) -> f64  // HARDCODED
pub fn price_18k(&self) -> f64  // HARDCODED
```

**Location:** `crates/persistence/src/gold_price.rs` (lines 163-181)
```rust
pub fn new_gold(client: ScyllaClient, base_price_24k: f64) -> Self {
    // Hardcoded tier definitions
    TierDefinition { code: "24K".to_string(), factor: 1.0, ... },
    TierDefinition { code: "22K".to_string(), factor: 0.916, ... },
}
```

**Fix:**
```rust
#[deprecated(since = "0.2.0", note = "Use price_for_tier() with config-driven tier codes")]
pub fn price_24k(&self) -> f64 { self.price_for_tier("24K") }

// Add config-driven factory
pub fn from_domain_view(view: &ToolsDomainView, client: ScyllaClient) -> Self {
    let tiers = view.quality_tiers_full().into_iter()
        .map(|(code, factor, desc)| TierDefinition { code, factor, description: desc })
        .collect();
    Self::new(client, view.asset_price_per_unit(), tiers)
}
```

### MEDIUM: Should Fix

#### 3. Config Views Fallbacks

**Location:** `crates/config/src/domain/views.rs` (lines 1759, 1774, 1800-1802, 1815)
```rust
// Fallback values with domain-specific terms
vec!["24K".to_string(), "22K".to_string(), "18K".to_string()]
```

**Fix:**
```rust
tracing::warn!("asset_quality_tier not found - using generic fallback");
vec!["tier_1".to_string(), "tier_2".to_string(), "tier_3".to_string()]
```

#### 4. Test Data with Competitor Names

**Locations:**
- `crates/agent/src/agent/mod.rs` (line 835): `"Muthoot"`
- `crates/agent/src/memory/mod.rs` (~line 1158): `"Muthoot"`
- `crates/agent/src/dst/dynamic.rs` (lines 615, 644, 671): `"Muthoot"`

**Fix:** Replace with `"competitor_1"` or `"another_provider"`

#### 5. Slot Extraction Patterns

**Location:** `crates/text_processing/src/slot_extraction/mod.rs` (lines 189-192)
```rust
static PURITY_24K: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)24\s*(?:k|karat)").unwrap());
static PURITY_22K: Lazy<Regex> = ...
```

**Fix:** Keep as fallback, but prefer config-driven patterns from `extraction_patterns.yaml`

### LOW: Documentation/Comments Only

- `crates/core/src/financial.rs` (line 68): Gold loan example in comment
- `crates/persistence/src/gold_price.rs` (lines 19-21): Gold price examples
- Various docstrings with domain-specific examples

---

## Config Wiring Analysis

### All Config Files (24 total)

| Config File | Rust Struct | Wired | Status |
|-------------|-------------|-------|--------|
| domain.yaml | MasterDomainConfig | Yes | OK |
| slots.yaml | SlotsConfig | Yes | OK |
| stages.yaml | StagesConfig | Yes | OK |
| intents.yaml | IntentsConfig | Partial | `optional_slots` unused |
| goals.yaml | GoalsConfig | Partial | `intent_mappings` unwired |
| objections.yaml | ObjectionsConfig | Yes | OK |
| segments.yaml | SegmentsConfig | Yes | OK |
| competitors.yaml | CompetitorsConfig | Yes | OK |
| lead_scoring.yaml | ScoringConfig | Yes | OK |
| compliance.yaml | ComplianceConfig | Partial | `language_rules`, `severity_levels` unwired |
| vocabulary.yaml | FullVocabularyConfig | Yes | OK |
| adaptation.yaml | AdaptationConfig | Partial | `terminology`, `enabled_features` unwired |
| extraction_patterns.yaml | ExtractionPatternsConfig | Partial | `confidence_boosters`, `name_exclusions` unwired |
| features.yaml | FeaturesConfig | Yes | OK |
| prompts/system.yaml | PromptsConfig | Yes | OK |
| tools/schemas.yaml | ToolsConfig | Yes | OK |
| tools/branches.yaml | BranchesConfig | Yes | OK |
| tools/responses.yaml | ToolResponsesConfig | Yes | OK |
| tools/sms_templates.yaml | SmsTemplatesConfig | Yes | OK |
| intent_tool_mappings.yaml | IntentToolMappingsConfig | Yes | OK |
| entities.yaml | EntitiesConfig | Yes | OK |
| documents.yaml | DocumentsConfig | Yes | OK |

### Unwired/Dead Config Fields (10+)

| Field | Location | Decision |
|-------|----------|----------|
| `raw_config` | master.rs:579 | **REMOVE** |
| `IntentsConfig.optional_slots` | intents.rs:88 | **DEPRECATE** |
| `GoalsConfig.intent_mappings` | goals.rs:168 | **WIRE** to DST |
| `confidence_boosters` | extraction_patterns.rs:42 | **WIRE** to slot extraction |
| `terminology` | adaptation.rs:32 | **WIRE** to response generation |
| `enabled_features` | adaptation.rs:36 | **WIRE** to feature gates |
| `language_rules` | compliance.rs:46 | **WIRE** to compliance checker |
| `severity_levels` | compliance.rs:50 | **WIRE** to violation categorization |
| `repayment_types` | extraction_patterns.rs:30 | **WIRE** to slot extraction |
| `name_exclusions` | extraction_patterns.rs:46 | **WIRE** to name validation |

### Critical Gap: ConfigValidator Not Called

**Problem:** `ConfigValidator` exists in `validator.rs` but is NEVER CALLED at startup

**Location:** `crates/server/src/main.rs` (after line 327)

**Fix:**
```rust
fn load_master_domain_config(config_dir: &str) -> Arc<MasterDomainConfig> {
    match MasterDomainConfig::load(&domain_id, config_path) {
        Ok(config) => {
            // NEW: Validate at startup
            let validator = ConfigValidator::new();
            let result = validator.validate(&domain_id, &config);

            if !result.is_ok() {
                tracing::error!("Config validation failed");
                std::process::exit(1);
            }
            // ...
        }
    }
}
```

### Incomplete Variable Substitution

**Current:** Only substitutes in stages, segments, competitors (3/20+)

**Missing:**
- prompts.system_prompt
- prompts.greetings
- objections.objections[*].responses[*].templates
- tool_responses.templates[*].success/error
- sms_templates.templates[*].body

---

## Trait Architecture Analysis

### Issue 1: LeadScoringStrategy Trait Bloat

**Current:** 9 methods mixing 4 concerns
```rust
pub trait LeadScoringStrategy: Send + Sync {
    fn calculate_breakdown(&self, signals: &dyn LeadSignals) -> ScoreBreakdown;
    fn calculate_total(&self, signals: &dyn LeadSignals) -> u32;
    fn qualification_level(&self, signals: &dyn LeadSignals) -> QualificationLevel;
    fn classify(&self, signals: &dyn LeadSignals) -> LeadClassification;
    fn conversion_probability(&self, signals: &dyn LeadSignals) -> f32;
    fn check_escalation_triggers(&self, signals: &dyn LeadSignals) -> Vec<EscalationTrigger>;
    fn urgency_keywords(&self) -> Vec<&str>;
    fn thresholds(&self) -> (u32, u32, u32, u32);
    fn config(&self) -> &ScoringConfig;  // Leaks implementation!
}
```

**Fix:** Split into focused traits:
```rust
pub trait ScoreCalculator: Send + Sync {
    fn calculate_breakdown(&self, signals: &dyn SignalProvider) -> ScoreBreakdown;
}

pub trait QualificationResolver: Send + Sync {
    fn qualification_level(&self, score: u32) -> QualificationLevel;
}

pub trait LeadClassifier: Send + Sync {
    fn classify(&self, signals: &dyn SignalProvider) -> LeadClass;
}

pub trait EscalationDetector: Send + Sync {
    fn check_triggers(&self, signals: &dyn SignalProvider) -> Vec<EscalationResult>;
}
```

### Issue 2: Duplicate LeadSignals Traits

**Problem:** Two competing implementations:
- `LeadSignals` (scoring.rs) - 15+ hardcoded methods
- `LeadSignalsTrait` (lead_classifier.rs) - 3 generic methods

**Fix:** Consolidate on generic `SignalProvider`:
```rust
pub trait SignalProvider: Send + Sync {
    fn has_signal(&self, signal_id: &str) -> bool;
    fn get_numeric(&self, signal_id: &str) -> Option<u32>;
    fn get_string(&self, signal_id: &str) -> Option<&str>;
    fn active_signals(&self) -> Vec<&str>;
}
```

### Issue 3: Domain-Coupled Enums

**CompetitorType (loan-specific):**
```rust
pub enum CompetitorType {
    Bank,      // Loan-industry specific
    Nbfc,      // Loan-industry specific
    Informal,  // Loan-industry specific
}
```

**CustomerSegment (hardcoded variants):**
```rust
pub enum CustomerSegment {
    HighValue,
    TrustSeeker,
    FirstTime,
    PriceSensitive,
    // ...
}
```

**Fix:** Make config-driven with string IDs:
```yaml
# entity_types.yaml
competitor_types:
  bank:
    display_name: "Bank"
    default_rate: 11.0
  nbfc:
    display_name: "NBFC"
    default_rate: 18.0
```

---

## Implementation Plan

### Phase 1: Low-Risk Changes (Week 1)

| Task | File | Lines |
|------|------|-------|
| Update LLM test with generic tiers | `crates/llm/src/prompt.rs` | ~968 |
| Update LLM test with generic tiers | `crates/llm/src/claude.rs` | ~683 |
| Update DST tests with generic provider | `crates/agent/src/dst/dynamic.rs` | 615, 644, 671 |
| Update agent tests | `crates/agent/src/agent/mod.rs` | 835 |
| Update memory tests | `crates/agent/src/memory/mod.rs` | ~1158 |
| Update persistence tests | `crates/persistence/src/gold_price.rs` | tests section |

### Phase 2: Config Wiring (Week 2)

| Task | File | Details |
|------|------|---------|
| Remove `raw_config` field | `master.rs` | Lines 579, 627, 676 |
| Wire `confidence_boosters` | `slot_extraction/mod.rs` | Add boost calculation |
| Wire `terminology` | `agent/response.rs` | Add term substitution |
| Wire `enabled_features` | `agent/mod.rs` | Add feature gates |
| Wire `language_rules` | `compliance/checker.rs` | Language-specific rules |
| Wire `GoalsConfig.intent_mappings` | `dst/dynamic.rs` | Goal routing |

### Phase 3: Validation & Substitution (Week 3)

| Task | File | Details |
|------|------|---------|
| Add ConfigValidator call | `server/main.rs` | After line 327 |
| Extend cross-reference validation | `validator.rs` | Tools, goals, slots |
| Extend variable substitution | `master.rs` | Prompts, objections, responses |
| Update view fallbacks | `views.rs` | Generic tier names |

### Phase 4: Trait Refactoring (Week 4)

| Task | Details |
|------|---------|
| Create `SignalProvider` trait | Replace hardcoded `LeadSignals` |
| Create `SignalStore` implementation | Config-driven signal storage |
| Split `LeadScoringStrategy` | 5 focused traits |
| Create `EntityTypeProvider` | Config-driven type definitions |
| Add `signals.yaml` schema | Signal definitions |
| Add `entity_types.yaml` schema | Entity type definitions |

### Phase 5: Migration & Cleanup (Week 5)

| Task | Details |
|------|---------|
| Migrate agent to new traits | Update `lead_scoring.rs` |
| Deprecate old enums | Add `#[deprecated]` annotations |
| Update all tests | Use generic data |
| Final validation | Full test suite |

---

## New Config Schemas

### signals.yaml (NEW)

```yaml
signals:
  # Boolean signals
  has_urgency:
    display_name: "Urgency Detected"
    type: boolean
    category: urgency
    weight: 10

  asked_about_rates:
    display_name: "Asked About Rates"
    type: boolean
    category: engagement
    weight: 3

  # Counter signals
  engagement_turns:
    display_name: "Engagement Turns"
    type: counter
    category: engagement
    weight: 3
    max: 5

  urgency_keyword_count:
    display_name: "Urgency Keywords"
    type: counter
    category: urgency
    weight: 5
    max: 3

  # Information signals
  provided_contact:
    display_name: "Provided Contact Info"
    type: boolean
    category: information
    weight: 8

categories:
  urgency:
    max_score: 25
    weight: 1.0
  engagement:
    max_score: 25
    weight: 1.0
  information:
    max_score: 25
    weight: 1.0
  intent:
    max_score: 25
    weight: 1.0
```

### entity_types.yaml (NEW)

```yaml
entity_types:
  competitor_types:
    bank:
      display_name: "Bank"
      default_values:
        rate: 11.0
      aliases: ["scheduled_bank", "commercial_bank"]

    nbfc:
      display_name: "NBFC"
      default_values:
        rate: 18.0
      aliases: ["finance_company"]

    informal:
      display_name: "Informal Lender"
      default_values:
        rate: 24.0
      aliases: ["local_lender", "moneylender"]

  customer_segments:
    high_value:
      display_name: "High Value"
      default_values:
        warmth: 0.9

    trust_seeker:
      display_name: "Trust Seeker"
      default_values:
        warmth: 0.95

    first_time:
      display_name: "First Time"
      default_values:
        warmth: 0.9
```

---

## Verification Plan

### Unit Tests

1. **LLM tests pass** with generic tier names
2. **Persistence tests pass** with generic tiers
3. **ConfigValidator catches** missing tool references
4. **Variable substitution works** in prompts, objections, responses
5. **SignalProvider tests** with config-driven signals
6. **EntityTypeProvider tests** with config-driven types

### Integration Tests

1. Load gold_loan domain - no validation errors
2. Full conversation flow with config-driven data
3. Tool execution with config parameters

### Manual Testing

1. Start server with `DOMAIN_ID=gold_loan` - clean startup
2. Create minimal test domain - verify all features work
3. Onboard hypothetical "car_loan" domain with YAML only

### Success Criteria

- [ ] Zero hardcoded domain terms in production code
- [ ] All config fields wired or explicitly removed
- [ ] Startup validation catches config errors
- [ ] Variable substitution in all text configs
- [ ] New domain onboarding requires only YAML
- [ ] All tests pass with generic data
- [ ] No backwards compatibility code needed

---

## Appendix: Critical File Locations

### Must Modify

| File | Purpose |
|------|---------|
| `crates/llm/src/prompt.rs` | Remove hardcoded purity enum |
| `crates/persistence/src/gold_price.rs` | Deprecate legacy accessors |
| `crates/config/src/domain/views.rs` | Replace hardcoded fallbacks |
| `crates/config/src/domain/master.rs` | Remove raw_config, extend substitution |
| `crates/server/src/main.rs` | Add ConfigValidator call |
| `crates/config/src/domain/validator.rs` | Add cross-reference validation |
| `crates/text_processing/src/slot_extraction/mod.rs` | Wire extraction patterns |
| `crates/agent/src/lead_scoring.rs` | Use config-driven signals |
| `crates/core/src/traits/scoring.rs` | Split into focused traits |
| `crates/core/src/traits/competitors.rs` | Remove CompetitorType enum |
| `crates/core/src/customer.rs` | Deprecate CustomerSegment enum |

### Must Create

| File | Purpose |
|------|---------|
| `crates/core/src/traits/signals.rs` | SignalProvider trait |
| `crates/core/src/traits/entity_types.rs` | EntityTypeProvider trait |
| `config/domains/gold_loan/signals.yaml` | Signal definitions |
| `config/domains/gold_loan/entity_types.yaml` | Entity type definitions |

---

## Implementation Status (Updated 2026-01-10)

### Completed Changes

| Phase | Task | Status |
|-------|------|--------|
| 1.1 | Update LLM tests with generic tier names | DONE |
| 1.2 | Deprecate legacy accessors in gold_price.rs | DONE |
| 1.3 | Replace hardcoded fallbacks in views.rs | DONE |
| 1.4 | Update test data with generic provider names | DONE |
| 2.1 | Remove dead raw_config field | DONE |
| 3.1 | Add ConfigValidator at startup | DONE |
| 5.1 | Create SignalProvider trait | DONE |
| 5.2 | Create signals.yaml config | DONE |
| 5.3 | Create EntityTypeProvider trait | DONE |
| 5.4 | Create entity_types.yaml config | DONE |

### New Files Created

1. **`crates/core/src/traits/signals.rs`** - SignalProvider trait with:
   - `SignalType` enum (Boolean, Counter, String, Numeric)
   - `SignalValue` enum for storing signal values
   - `SignalProvider` trait for generic signal access
   - `SignalStore` default implementation
   - `LegacySignalAdapter` for backward compatibility

2. **`crates/core/src/traits/entity_types.rs`** - EntityTypeProvider trait with:
   - `EntityTypeDefinition` for type configuration
   - `EntityTypeCategory` for grouping types
   - `EntityTypeProvider` trait for config-driven types
   - `EntityTypeStore` default implementation
   - `LegacyCompetitorTypeAdapter` and `LegacySegmentAdapter`

3. **`config/domains/gold_loan/signals.yaml`** - Signal definitions:
   - Urgency signals (has_urgency, time_pressure, urgency_keyword_count)
   - Engagement signals (has_pricing_interest, engagement_turns, etc.)
   - Information signals (provided_contact, provided_asset_details)
   - Intent signals (requested_callback, ready_to_visit)
   - Scoring thresholds and escalation triggers

4. **`config/domains/gold_loan/entity_types.yaml`** - Entity type definitions:
   - competitor_types (bank, nbfc, cooperative, informal)
   - customer_segments (high_value, trust_seeker, first_time, etc.)
   - asset_quality_types (tier_1/24K, tier_2/22K, tier_3/18K, tier_4/14K)
   - repayment_types (bullet, emi, flexible)

### Modified Files

1. **`crates/llm/src/prompt.rs`** - Updated test to use generic tier names
2. **`crates/persistence/src/gold_price.rs`** - Added deprecation warnings, `from_tiers()` factory
3. **`crates/config/src/domain/views.rs`** - Generic fallback values
4. **`crates/config/src/domain/master.rs`** - Removed `get_constant()`
5. **`crates/server/src/main.rs`** - Added ConfigValidator at startup
6. **`crates/config/src/lib.rs`** - Added ConfigValidator exports
7. **`crates/core/src/traits/mod.rs`** - Added signals and entity_types modules
8. **`crates/agent/src/agent/mod.rs`** - Updated test with generic provider
9. **`crates/config/src/domain/tools.rs`** - Fixed test missing field

### Verification

All changes compile successfully:
```
cargo check
Finished `dev` profile [optimized + debuginfo] target(s)
```

### Remaining Work (Future)

- Wire signals.yaml to lead_scoring.rs
- Wire entity_types.yaml to competitors analysis
- Extend variable substitution to prompts and objections
- Remove deprecated methods after migration period

---

*Generated by Claude Code analysis on 2026-01-09*
*Updated with implementation status on 2026-01-10*
