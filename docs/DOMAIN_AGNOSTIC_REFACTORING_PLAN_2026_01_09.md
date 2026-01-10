# Domain-Agnostic Voice Agent Refactoring Plan

**Date:** 2026-01-09
**Status:** Ready for Implementation
**Goal:** Enable new domain onboarding by ONLY defining YAML config files

---

## Executive Summary

The voice agent backend has been partially refactored through P0-P19 FIX phases, but critical hardcoded domain-specific content remains that prevents true domain agnosticism. This document provides a comprehensive analysis and implementation plan.

---

## Part 1: Detailed Analysis of Hardcoded Domain Terms

### 1.1 CRITICAL: Hardcoded Strings in adaptation.rs

| Line | Content | Severity |
|------|---------|----------|
| 61 | `Feature::WomenBenefits => "Shakti Gold Benefits"` | CRITICAL |
| 80 | `Feature::WomenBenefits => "Shakti Gold"` | CRITICAL |
| 517 | `"Shakti Gold program with 0.25% lower interest for women"` | CRITICAL |

### 1.2 HIGH: Hardcoded Enums

**Feature Enum (adaptation.rs:16-43):**
```rust
pub enum Feature {
    LowRates,
    QuickProcess,
    Security,
    Transparency,
    Flexibility,
    Digital,
    RelationshipManager,
    HigherLimits,
    NoHiddenCharges,
    RbiRegulated,
    ZeroForeclosure,
    DoorstepService,
    WomenBenefits,  // DOMAIN-SPECIFIC - "Shakti Gold"
}
```

**Objection Enum (adaptation.rs:88-109):**
```rust
pub enum Objection {
    GoldSafety,           // DOMAIN-SPECIFIC - should be "CollateralSafety"
    BetterRatesElsewhere,
    TooMuchPaperwork,
    DontWantToSwitch,
    NeedsTime,
    TrustIssues,
    ExpectsHiddenCharges,
    TooSlow,
    NoNearbyBranch,
    ExistingLoans,
}
```

### 1.3 MEDIUM: Hardcoded Fallback Logic

**tools.rs (lines 38-87):** Intent-to-tool fallback mappings
```rust
match intent.intent.as_str() {
    "eligibility_check" => { /* hardcoded */ }
    "switch_lender" => { /* hardcoded */ }
    // ... more hardcoded mappings
}
```

**tools.rs (lines 133-166):** Tool default values
```rust
if name == "check_eligibility" && !args.contains_key("collateral_variant") {
    args.insert("collateral_variant", defaults.default_gold_purity);
}
```

**lead_scoring.rs (lines 64-81):** Classification rules
```rust
if signals.has_urgency_signal
    && signals.provided_contact_info
    && signals.has_specific_requirements
{
    return LeadClassification::SQL;
}
```

### 1.4 LOW: Domain-Specific Method Names

| Method | File | Should Be |
|--------|------|-----------|
| `calculate_gold_value()` | views.rs | `calculate_asset_value()` |
| `gold_loan_branches()` | views.rs | `service_branches()` |

### 1.5 Grep Results Summary

Domain terms found in crates (excluding tests and comments):

| Term | Count | Primary Locations |
|------|-------|------------------|
| "gold loan" | 80+ | Tests, comments, docs |
| "Shakti" | 5 | adaptation.rs (CRITICAL) |
| "Kotak" | 10+ | Tests, docs |
| "Muthoot" | 15+ | Tests, competitor examples |
| "IIFL" | 8+ | Tests, competitor examples |

---

## Part 2: Current Config Architecture

### 2.1 Config File Structure
```
config/
├── default.yaml                      # App defaults
├── production.yaml                   # Production overrides
└── domains/gold_loan/
    ├── domain.yaml                   # Core domain config (888 lines)
    ├── slots.yaml                    # DST slot definitions
    ├── intents.yaml                  # Intent definitions
    ├── entities.yaml                 # NER entity types
    ├── stages.yaml                   # Conversation stages
    ├── goals.yaml                    # Conversation goals
    ├── scoring.yaml                  # Lead scoring rules
    ├── segments.yaml                 # Customer segmentation
    ├── features.yaml                 # Product features
    ├── compliance.yaml               # Regulatory rules
    ├── vocabulary.yaml               # Domain vocabulary
    ├── objections.yaml               # Objection handling
    ├── competitors.yaml              # Competitor data
    ├── lead_scoring.yaml             # Lead scoring config
    ├── intent_tool_mappings.yaml     # Intent -> Tool routing
    ├── prompts/system.yaml           # LLM system prompts
    └── tools/
        ├── schemas.yaml              # Tool schemas
        ├── responses.yaml            # Tool response templates
        ├── documents.yaml            # Document checklists
        ├── branches.yaml             # Branch locations
        └── sms_templates.yaml        # SMS templates
```

### 2.2 Rust Config Modules
```
crates/config/src/
├── lib.rs                            # Module exports
├── settings.rs                       # Load application settings
├── agent.rs                          # Agent configuration
├── constants.rs                      # Domain-agnostic constants
└── domain/
    ├── mod.rs                        # Domain module exports
    ├── master.rs                     # MasterDomainConfig (890 lines)
    ├── bridge.rs                     # DomainBridge trait adapters
    ├── views.rs                      # Crate-specific views (1200+ lines)
    ├── slots.rs, intents.rs, etc.    # Individual config modules
```

### 2.3 View Pattern (Already Implemented)
- `AgentDomainView` - For agent crate
- `ToolsDomainView` - For tools crate
- `LlmDomainView` - For LLM crate
- `RagDomainView` - For RAG crate

---

## Part 3: Existing Trait Architecture

### 3.1 Well-Designed Traits (25+)
```rust
// Core traits
pub trait LanguageModel
pub trait SpeechToText
pub trait TextToSpeech
pub trait Retriever
pub trait Tool
pub trait ToolFactory

// Domain-agnostic business logic (P13 FIX)
pub trait DomainCalculator
pub trait SlotSchema
pub trait ConversationGoalSchema
pub trait LeadScoringStrategy
pub trait SegmentDetector
pub trait ObjectionHandler
pub trait CompetitorAnalyzer
```

### 3.2 Missing Traits (Need to Create)
```rust
pub trait FeatureProvider       // Replace Feature enum
pub trait ObjectionProvider     // Replace Objection enum
pub trait ToolArgumentProvider  // Config-driven tool defaults
pub trait LeadClassifier        // Config-driven MQL/SQL rules
pub trait StageTransitionResolver // Config-driven stage transitions
```

---

## Part 4: Implementation Plan

### Phase 1: Create New Traits (Non-Breaking)

**Files to create:**
1. `crates/core/src/traits/feature_provider.rs`
2. `crates/core/src/traits/objection_provider.rs`
3. `crates/core/src/traits/tool_arguments.rs`
4. `crates/core/src/traits/lead_classifier.rs`

**Trait Definitions:**

```rust
// FeatureProvider
pub trait FeatureProvider: Send + Sync {
    fn feature_display_name(&self, id: &str, lang: &str) -> Option<String>;
    fn features_for_segment(&self, segment_id: &str) -> Vec<String>;
    fn all_feature_ids(&self) -> Vec<String>;
}

// ObjectionProvider
pub trait ObjectionProvider: Send + Sync {
    fn detect_objection(&self, text: &str, lang: &str) -> Option<(String, f32)>;
    fn get_response(&self, id: &str, lang: &str) -> Option<ObjectionResponse>;
    fn all_objection_ids(&self) -> Vec<String>;
}

// ToolArgumentProvider
pub trait ToolArgumentProvider: Send + Sync {
    fn get_tool_defaults(&self, tool: &str) -> HashMap<String, Value>;
    fn get_argument_mapping(&self, tool: &str) -> HashMap<String, String>;
    fn resolve_tool_for_intent(&self, intent: &str, slots: &[&str]) -> Option<String>;
}

// LeadClassifier
pub trait LeadClassifier: Send + Sync {
    fn classify(&self, signals: &LeadSignals) -> LeadClassification;
    fn qualification_level(&self, score: u32) -> LeadQualification;
}
```

### Phase 2: Config Schema Updates

**New file: `config/domains/gold_loan/adaptation.yaml`**
```yaml
schema_version: "1.0"

# Domain-specific variable definitions
variables:
  special_program_name: "Shakti Gold"
  special_program_benefit: "0.25% lower interest for women"
  collateral_type: "gold"

# Segment-specific adaptations
segment_adaptations:
  women:
    primary_features: ["women_benefits", "security"]
    special_program:
      enabled: true
      name: "{{special_program_name}}"
      benefit: "{{special_program_benefit}}"
```

**Enhanced features.yaml:**
```yaml
features:
  women_benefits:
    display_name:
      en: "{{special_program_name}} Benefits"
      hi: "{{special_program_name}}"
    description:
      en: "Special benefits for women customers"
    enabled: true

segment_features:
  women: ["women_benefits", "security", "flexibility"]

value_propositions:
  women:
    en:
      - "{{special_program_name}} with {{special_program_benefit}}"
```

**Enhanced objections.yaml:**
```yaml
objections:
  collateral_safety:  # Renamed from gold_safety
    aliases: ["gold_safety"]  # Backward compat
    display_name:
      en: "Safety Concerns"
    detection:
      patterns:
        en: ["safe", "security", "trust"]
        hi: ["bharosa", "suraksha"]
    responses:
      en:
        acknowledge: "I understand your concern about {{collateral_type}} safety."
        respond: "Your {{collateral_type}} is stored in secure vaults..."
```

**Enhanced lead_scoring.yaml:**
```yaml
classification:
  sql:
    required_signals:
      - has_urgency_signal
      - provided_contact_info
      - has_specific_requirements
  mql:
    required_signals:
      - engagement_turns: 3
    any_of:
      - asked_about_rates
      - asked_for_comparison
```

### Phase 3: Refactor adaptation.rs

1. **Remove Feature enum** (lines 16-83)
2. **Remove Objection enum** (lines 88-205)
3. **Remove load_defaults() method** (lines 408-534)
4. **Make from_config() the only constructor**

```rust
// REPLACE hardcoded enum with type alias
pub type FeatureId = String;
pub type ObjectionId = String;

// Feature IDs as constants (for convenience, not enforcement)
pub mod feature_ids {
    pub const LOW_RATES: &str = "low_rates";
    pub const WOMEN_BENEFITS: &str = "women_benefits";
    // ... etc
}
```

### Phase 4: Refactor tools.rs

1. **Remove hardcoded fallback mappings** (lines 35-87)
2. **Remove hardcoded tool defaults** (lines 133-166, 356-386)
3. **Fail fast when config missing**

```rust
// BEFORE: Hardcoded fallback
.or_else(|| {
    match intent.intent.as_str() {
        "eligibility_check" => Some("check_eligibility".to_string()),
        // ...
    }
})

// AFTER: Config-only
let tool_name = self.domain_view
    .as_ref()
    .ok_or_else(|| AgentError::ConfigError("DomainView required"))?
    .resolve_tool_for_intent(&intent.intent, &slots);
```

### Phase 5: Rename Methods

**In views.rs:**
```rust
// Rename
calculate_gold_value() → calculate_asset_value()
gold_loan_branches() → service_branches()

// Add deprecated alias for backward compat
#[deprecated(note = "Use calculate_asset_value")]
pub fn calculate_gold_value(&self, w: f64, v: &str) -> f64 {
    self.calculate_asset_value(w, v)
}
```

### Phase 6: Add Config Validation

**New file: `crates/config/src/validation.rs`**
```rust
pub struct ConfigValidator;

impl ConfigValidator {
    pub fn validate(config: &MasterDomainConfig) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if config.features.features.is_empty() {
            errors.push(ValidationError::MissingRequired("features"));
        }
        if config.objections.objections.is_empty() {
            errors.push(ValidationError::MissingRequired("objections"));
        }
        // ... more validations

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

---

## Part 5: Files Summary

### Files to Create
| File | Purpose |
|------|---------|
| `crates/core/src/traits/feature_provider.rs` | FeatureProvider trait |
| `crates/core/src/traits/objection_provider.rs` | ObjectionProvider trait |
| `crates/core/src/traits/tool_arguments.rs` | ToolArgumentProvider trait |
| `crates/core/src/traits/lead_classifier.rs` | LeadClassifier trait |
| `crates/config/src/validation.rs` | Config validation |
| `config/domains/gold_loan/adaptation.yaml` | Adaptation config |

### Files to Modify
| File | Changes |
|------|---------|
| `crates/core/src/personalization/adaptation.rs` | Remove enums, use config |
| `crates/agent/src/agent/tools.rs` | Remove hardcoded fallbacks |
| `crates/agent/src/lead_scoring.rs` | Config-driven classification |
| `crates/config/src/domain/views.rs` | Rename methods, add traits |
| `crates/config/src/domain/master.rs` | Add validation |
| `crates/core/src/traits/mod.rs` | Export new traits |

---

## Part 6: Verification

### Unit Tests
```bash
cargo test --workspace
```

### No Hardcoded Strings Check
```bash
# Should return empty (excluding tests/comments)
grep -rn "Shakti\|Kotak\|Muthoot" crates/*/src/*.rs | grep -v test | grep -v "//"
```

### Domain Onboarding Test
```bash
# Create minimal new domain
mkdir -p config/domains/test_domain
# Copy and modify YAML files
# Start with new domain
DOMAIN_ID=test_domain cargo run
```

---

## Part 7: Success Criteria

1. ✅ New domain can be created with ONLY YAML config files
2. ✅ No domain-specific strings in Rust source (except deprecated code, tests, comments)
3. ✅ All existing tests pass
4. ✅ Config validation catches missing sections
5. ✅ Variable substitution works ({{company_name}}, etc.)
6. ✅ Segment-specific overrides work

---

## Appendix: Architecture Quality Scorecard

| Dimension | Before | After (Target) |
|-----------|--------|----------------|
| Trait Coverage | 9/10 | 10/10 |
| Domain Agnosticism | 7/10 | 9.5/10 |
| Config-Driven Design | 8/10 | 9.5/10 |
| Backward Compatibility | 9/10 | 9/10 |
| Fail-Fast Behavior | 5/10 | 9/10 |
| Testability | 9/10 | 9.5/10 |

**Overall: B+ (8.2/10) → A (9.3/10)**
