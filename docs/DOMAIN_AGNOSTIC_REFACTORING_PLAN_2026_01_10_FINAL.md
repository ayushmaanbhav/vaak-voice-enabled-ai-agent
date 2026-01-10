# Domain-Agnostic Voice Agent Refactoring Plan

**Date**: 2026-01-10
**Status**: Comprehensive Analysis Complete
**Goal**: Enable domain onboarding via YAML configs only, with zero Rust code changes

---

## Executive Summary

This plan addresses critical hardcoded components preventing true domain-agnosticism in the voice-agent backend. The codebase has achieved ~70% config-driven design, but several blockers remain that require code changes to onboard new domains.

---

## Critical Issues Identified

### Issue 1: CustomerSegment Enum (CRITICAL BLOCKER)

**Location**: `crates/core/src/customer.rs:8-21`

**Problem**: Hardcoded enum with 6 variants, while `segments.yaml` defines 7 different segments.

| Enum Variants | Config Segments | Status |
|---------------|-----------------|--------|
| HighValue | high_value | Aligned |
| TrustSeeker | trust_seeker | Aligned |
| FirstTime | first_time | Aligned |
| PriceSensitive | price_sensitive | Aligned |
| Women | - | In enum, NOT in config |
| Professional | - | In enum, NOT in config |
| - | balance_transfer | In config, NOT in enum |
| - | urgent_need | In config, NOT in enum |
| - | business_owner | In config, NOT in enum |

**Impact**: ~101 usages across 8 files:
- `crates/core/src/customer.rs` (~60 usages)
- `crates/core/src/personalization/adaptation.rs` (~20 usages)
- `crates/core/src/personalization/persona.rs` (~15 usages)
- `crates/core/src/personalization/mod.rs` (~12 usages)
- `crates/agent/src/agent/mod.rs` (3 usages)

**Key Functions with Hardcoded Matches**:
- `CustomerSegment::display_name()` - lines 25-34
- `CustomerSegment::key_messages()` - lines 40-79
- `CustomerSegment::suggested_warmth()` - lines 82-91
- `segment_to_id()` - adaptation.rs:391-400
- `parse_segment_id()` - adaptation.rs:403-413

---

### Issue 2: Persona System Hardcoding (CRITICAL)

**Location**: `crates/core/src/personalization/persona.rs`

| Component | Lines | Problem |
|-----------|-------|---------|
| Tone enum | 16-26 | 4 hardcoded variants |
| Tone::greeting_prefix() | 30-36 | Hardcoded English phrases |
| Tone::closing_phrase() | 40-46 | Hardcoded English phrases |
| LanguageComplexity enum | 51-62 | 3 hardcoded variants |
| ResponseUrgency enum | 65-78 | 4 hardcoded variants |
| Persona::for_segment() | 132-207 | Hardcoded match with 6 segment mappings |
| system_prompt_instructions() | 246-330 | Hardcoded instruction strings |

**Persona::for_segment() Hardcoded Values**:

| Segment | Persona Name | Tone | Warmth | Empathy | Hinglish |
|---------|--------------|------|--------|---------|----------|
| HighValue | premium_advisor | Formal | 0.9 | 0.8 | false |
| TrustSeeker | trust_builder | Professional | 0.95 | 0.95 | true |
| FirstTime | helpful_guide | Friendly | 0.9 | 0.85 | true |
| PriceSensitive | value_expert | Professional | 0.7 | 0.6 | false |
| Women | shakti_advisor | Friendly | 0.95 | 0.9 | true |
| Professional | smart_advisor | Professional | 0.75 | 0.65 | false |

---

### Issue 3: Tool Parameter Aliases (HIGH)

**Hardcoded Legacy Parameter Names**:

| File | Lines | Hardcoded Alias |
|------|-------|-----------------|
| eligibility.rs | 109-110 | `"gold_weight_grams"` |
| eligibility.rs | 116-117 | `"gold_purity"` |
| price.rs | 114 | `"purity"` |
| price.rs | 115 | `"weight_grams"` |

**Hardcoded Defaults**:

| File | Lines | Hardcoded Value |
|------|-------|-----------------|
| competitor.rs | 119-127 | Default loan_amount: 100000, tenure: 12 |
| savings.rs | 73 | Rate range: 10.0-30.0 |
| branch_locator.rs | 61 | Max results: 5 |
| lead_capture.rs | 104 | Interest level: "Medium" |
| document_checklist.rs | 305, 310 | Customer type: "individual", existing: false |

---

### Issue 4: Enum-Based Key Messages & Warmth (MEDIUM)

**Location**: `crates/core/src/customer.rs:40-91`

- `key_messages()` - 6 hardcoded match arms with domain-specific messages like "Gold stored in secure bank vaults"
- `suggested_warmth()` - 6 hardcoded warmth values

These should be loaded from `segments.yaml` which already has `value_props` that could replace key_messages.

---

## Implementation Plan

### Phase 1: Extend segments.yaml with Persona Config

**File**: `config/domains/gold_loan/segments.yaml`

Add persona configuration to each segment:

```yaml
segments:
  high_value:
    display_name: "High Value Customer"
    priority: 1

    # NEW: Persona configuration (from persona.rs:134-145)
    persona:
      name: "premium_advisor"
      tone: "formal"
      warmth: 0.9
      empathy: 0.8
      language_complexity: "sophisticated"
      urgency: "efficient"
      use_customer_name: true
      acknowledge_emotions: true
      use_hinglish: false
      max_response_words: 80

    # NEW: Key messages (from customer.rs:42-47)
    key_messages:
      en:
        - "Dedicated relationship manager"
        - "Priority processing"
        - "Higher limits"
        - "Exclusive rates"
      hi:
        - "समर्पित रिलेशनशिप मैनेजर"
        - "प्राथमिकता प्रसंस्करण"

    detection: {...}
    features: [...]
    value_props: {...}

  # Add missing segments from enum
  women:
    display_name: "Women"
    priority: 3
    persona:
      name: "shakti_advisor"
      tone: "friendly"
      warmth: 0.95
      empathy: 0.9
      language_complexity: "simple"
      urgency: "relaxed"
      use_customer_name: true
      acknowledge_emotions: true
      use_hinglish: true
      max_response_words: 55
    detection:
      text_patterns:
        en: ["woman", "lady", "mahila", "shakti"]
    features: ["shakti_program", "preferential_rates"]
    key_messages:
      en:
        - "Special programs available"
        - "Preferential rates"
        - "Dedicated service centers"

  professional:
    display_name: "Young Professional"
    priority: 4
    persona:
      name: "smart_advisor"
      tone: "professional"
      warmth: 0.75
      empathy: 0.65
      language_complexity: "moderate"
      urgency: "efficient"
      use_customer_name: false
      acknowledge_emotions: false
      use_hinglish: false
      max_response_words: 45
    detection:
      text_patterns:
        en: ["professional", "office", "salary", "corporate"]
    features: ["quick_digital_process", "mobile_app"]
    key_messages:
      en:
        - "Quick digital process"
        - "Mobile app tracking"
        - "Instant approval"
```

---

### Phase 2: Create New Config File - personas.yaml

**File**: `config/domains/gold_loan/personas.yaml`

```yaml
# Tone configurations with localized phrases
tones:
  formal:
    greeting_prefix:
      en: "Respected"
      hi: "आदरणीय"
    closing_phrase:
      en: "Thank you for your valuable time."
      hi: "आपके कीमती समय के लिए धन्यवाद।"
    instructions:
      en: "Use formal, respectful language. Address with honorifics."

  professional:
    greeting_prefix:
      en: "Dear"
      hi: "प्रिय"
    closing_phrase:
      en: "Thank you for considering us."
      hi: "हमें चुनने के लिए धन्यवाद।"
    instructions:
      en: "Use professional but warm language. Be clear and helpful."

  friendly:
    greeting_prefix:
      en: "Hi"
      hi: "नमस्ते"
    closing_phrase:
      en: "Thanks! Let me know if you need anything else."
      hi: "धन्यवाद! कुछ और चाहिए तो बताइए।"
    instructions:
      en: "Use friendly, approachable language."

  casual:
    greeting_prefix:
      en: "Hey"
      hi: "अरे"
    closing_phrase:
      en: "Cool, just ping me if you need help!"
    instructions:
      en: "Use casual, relaxed language."

# Instruction thresholds for dynamic generation
warmth_thresholds:
  - min: 0.8
    instruction: "Be very warm and welcoming. Express genuine care."
  - min: 0.6
    instruction: "Maintain a warm and helpful tone."
  - min: 0.0
    instruction: "Keep responses focused and factual."

empathy_thresholds:
  - min: 0.8
    instruction: "Show strong empathy. Acknowledge concerns explicitly."
  - min: 0.5
    instruction: "Show understanding when customer expresses concerns."
  - min: 0.0
    instruction: "Maintain professional neutrality."

complexity_levels:
  simple:
    instruction: "Use simple words and short sentences. Avoid jargon."
  moderate:
    instruction: "Use clear language. Explain terms briefly if needed."
  sophisticated:
    instruction: "Technical terms are acceptable. Detailed explanations welcome."

urgency_levels:
  relaxed:
    instruction: "Take time to explain thoroughly. No rush."
  normal:
    instruction: "Maintain natural conversational pace."
  efficient:
    instruction: "Be concise. Value customer's time."
  urgent:
    instruction: "Quick responses. Focus on key information."

# Hinglish guidance
hinglish_config:
  enabled_instruction: "Feel free to use common Hindi words/phrases if the customer uses them (e.g., 'ji', 'bilkul', 'zaroor')."
  disabled_instruction: "Use only English or fully Hindi based on customer preference."
```

---

### Phase 3: Add Tool Parameter Config

**File**: `config/domains/gold_loan/tools/schemas.yaml`

Add parameter aliases and defaults:

```yaml
# Parameter aliases for backward compatibility
parameter_aliases:
  collateral_weight:
    - "gold_weight_grams"
    - "weight_grams"
    - "weight"
  collateral_variant:
    - "gold_purity"
    - "purity"
    - "quality"
  offer_amount:
    - "loan_amount"
    - "amount"

# Tool-specific defaults (move from hardcoded values)
tool_defaults:
  compare_lenders:
    loan_amount: 100000
    tenure_months: 12
  savings_calculator:
    interest_rate_min: 10.0
    interest_rate_max: 30.0
  branch_locator:
    max_results: 5
  lead_capture:
    interest_level_default: "Medium"
  document_checklist:
    customer_type_default: "individual"
    existing_customer_default: false
```

---

### Phase 4: Rust Code Changes

#### 4.1 Create SegmentProvider Trait

**File**: `crates/core/src/traits/segment_provider.rs` (NEW)

```rust
//! Config-driven segment provider trait

use std::collections::HashMap;

/// Segment ID type - replaces CustomerSegment enum
pub type SegmentId = String;

/// Well-known segment IDs (convenience constants, not exhaustive)
pub mod segment_ids {
    pub const HIGH_VALUE: &str = "high_value";
    pub const TRUST_SEEKER: &str = "trust_seeker";
    pub const FIRST_TIME: &str = "first_time";
    pub const PRICE_SENSITIVE: &str = "price_sensitive";
    pub const WOMEN: &str = "women";
    pub const PROFESSIONAL: &str = "professional";
    pub const BALANCE_TRANSFER: &str = "balance_transfer";
    pub const URGENT_NEED: &str = "urgent_need";
    pub const BUSINESS_OWNER: &str = "business_owner";
}

/// Persona configuration from YAML
#[derive(Debug, Clone)]
pub struct PersonaConfig {
    pub name: String,
    pub tone: String,
    pub warmth: f32,
    pub empathy: f32,
    pub language_complexity: String,
    pub urgency: String,
    pub use_customer_name: bool,
    pub acknowledge_emotions: bool,
    pub use_hinglish: bool,
    pub max_response_words: usize,
}

/// Trait for segment providers - load from config
pub trait SegmentProvider: Send + Sync {
    fn all_segment_ids(&self) -> Vec<SegmentId>;
    fn get_segment(&self, id: &str) -> Option<&SegmentConfig>;
    fn default_segment(&self) -> SegmentId;
    fn persona_config(&self, segment_id: &str) -> Option<&PersonaConfig>;
    fn key_messages(&self, segment_id: &str, lang: &str) -> Vec<String>;
    fn priority(&self, segment_id: &str) -> u32;
}
```

#### 4.2 Create PersonaProvider Trait

**File**: `crates/core/src/traits/persona_provider.rs` (NEW)

```rust
//! Config-driven persona provider trait

use crate::personalization::Persona;
use std::collections::HashMap;

/// Tone configuration from YAML
#[derive(Debug, Clone)]
pub struct ToneConfig {
    pub greeting_prefix: HashMap<String, String>,
    pub closing_phrase: HashMap<String, String>,
    pub instructions: HashMap<String, String>,
}

/// Trait for persona providers - load from config
pub trait PersonaProvider: Send + Sync {
    fn tone_config(&self, tone_id: &str) -> Option<&ToneConfig>;
    fn build_instructions(&self, persona: &Persona, language: &str) -> String;
    fn greeting_prefix(&self, tone: &str, lang: &str) -> Option<String>;
    fn closing_phrase(&self, tone: &str, lang: &str) -> Option<String>;
}
```

#### 4.3 Deprecate CustomerSegment Enum

**File**: `crates/core/src/customer.rs`

```rust
/// DEPRECATED: Use SegmentId and SegmentProvider trait instead.
/// This enum remains for backward compatibility during migration.
#[deprecated(since = "2.0.0", note = "Use SegmentId from traits::segment_provider")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CustomerSegment {
    HighValue,
    TrustSeeker,
    FirstTime,
    PriceSensitive,
    Women,
    Professional,
}

impl CustomerSegment {
    /// Convert to string-based segment ID
    pub fn to_segment_id(&self) -> SegmentId {
        match self {
            Self::HighValue => "high_value".to_string(),
            Self::TrustSeeker => "trust_seeker".to_string(),
            Self::FirstTime => "first_time".to_string(),
            Self::PriceSensitive => "price_sensitive".to_string(),
            Self::Women => "women".to_string(),
            Self::Professional => "professional".to_string(),
        }
    }

    /// Try to create from segment ID
    pub fn from_segment_id(id: &str) -> Option<Self> {
        match id {
            "high_value" => Some(Self::HighValue),
            "trust_seeker" => Some(Self::TrustSeeker),
            "first_time" => Some(Self::FirstTime),
            "price_sensitive" => Some(Self::PriceSensitive),
            "women" => Some(Self::Women),
            "professional" => Some(Self::Professional),
            _ => None, // New segments from config won't map to enum
        }
    }
}
```

#### 4.4 Update Persona System

**File**: `crates/core/src/personalization/persona.rs`

```rust
impl Persona {
    /// Create persona from config (replaces for_segment match)
    pub fn from_config(config: &PersonaConfig) -> Self {
        Self {
            name: config.name.clone(),
            tone: Tone::from_str(&config.tone).unwrap_or_default(),
            warmth: config.warmth,
            empathy: config.empathy,
            language_complexity: LanguageComplexity::from_str(&config.language_complexity)
                .unwrap_or_default(),
            urgency: ResponseUrgency::from_str(&config.urgency).unwrap_or_default(),
            use_customer_name: config.use_customer_name,
            acknowledge_emotions: config.acknowledge_emotions,
            use_hinglish: config.use_hinglish,
            max_response_words: config.max_response_words,
        }
    }

    /// DEPRECATED: Use PersonaProvider::persona_for_segment() instead
    #[deprecated(since = "2.0.0", note = "Use Persona::from_config() with SegmentProvider")]
    pub fn for_segment(segment: CustomerSegment) -> Self {
        // Keep existing implementation for backward compatibility
        match segment { ... }
    }
}

impl Tone {
    /// Parse from string (for config loading)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "formal" => Some(Self::Formal),
            "professional" => Some(Self::Professional),
            "friendly" => Some(Self::Friendly),
            "casual" => Some(Self::Casual),
            _ => None,
        }
    }
}
```

#### 4.5 Update Tool Implementations

**File**: `crates/tools/src/domain_tools/tools/eligibility.rs`

```rust
// BEFORE (hardcoded):
let weight: f64 = input
    .get("collateral_weight")
    .or_else(|| input.get("gold_weight_grams"))  // Hardcoded!
    .and_then(|v| v.as_f64())
    .ok_or_else(|| ToolError::invalid_params("collateral_weight is required"))?;

// AFTER (config-driven):
let weight: f64 = self.view
    .get_param_with_aliases(&input, "collateral_weight")
    .and_then(|v| v.as_f64())
    .ok_or_else(|| ToolError::invalid_params("collateral_weight is required"))?;
```

---

## Files To Modify (Ordered by Dependency)

### Tier 1: Config Files (No Code Dependencies)
| File | Action | Priority |
|------|--------|----------|
| `config/domains/gold_loan/segments.yaml` | Extend with persona, key_messages, add missing segments | P0 |
| `config/domains/gold_loan/personas.yaml` | NEW: tone configs, thresholds | P0 |
| `config/domains/gold_loan/tools/schemas.yaml` | Add parameter_aliases, tool_defaults | P0 |

### Tier 2: Config Crate (Parse New Configs)
| File | Action | Priority |
|------|--------|----------|
| `crates/config/src/domain/segments.rs` | Add PersonaConfig, KeyMessages parsing | P1 |
| `crates/config/src/domain/personas.rs` | NEW: Parse personas.yaml | P1 |
| `crates/config/src/domain/mod.rs` | Export personas module | P1 |
| `crates/config/src/domain/views.rs` | Add get_param_with_aliases() | P1 |

### Tier 3: Core Crate (New Traits)
| File | Action | Priority |
|------|--------|----------|
| `crates/core/src/traits/segment_provider.rs` | NEW: SegmentProvider trait | P2 |
| `crates/core/src/traits/persona_provider.rs` | NEW: PersonaProvider trait | P2 |
| `crates/core/src/traits/mod.rs` | Export new traits | P2 |
| `crates/core/src/lib.rs` | Re-export SegmentId | P2 |

### Tier 4: Core Crate (Update Existing)
| File | Lines to Change | Action | Priority |
|------|-----------------|--------|----------|
| `crates/core/src/customer.rs` | 8-91 | Deprecate enum, update methods | P3 |
| `crates/core/src/personalization/persona.rs` | 132-207, 246-330 | Add from_config(), config-driven instructions | P3 |
| `crates/core/src/personalization/adaptation.rs` | 391-413 | Use SegmentId, remove hardcoded mappings | P3 |
| `crates/core/src/personalization/mod.rs` | 66, 97, 131 | Update PersonalizationContext | P3 |

### Tier 5: Tools Crate
| File | Lines to Change | Action | Priority |
|------|-----------------|--------|----------|
| `crates/tools/src/domain_tools/tools/eligibility.rs` | 109-120 | Use config aliases | P4 |
| `crates/tools/src/domain_tools/tools/price.rs` | 114-115 | Use config aliases | P4 |
| `crates/tools/src/domain_tools/tools/competitor.rs` | 119-127 | Use config defaults | P4 |
| `crates/tools/src/domain_tools/tools/savings.rs` | 73 | Use config constraints | P4 |

### Tier 6: Agent Crate
| File | Lines to Change | Action | Priority |
|------|-----------------|--------|----------|
| `crates/agent/src/agent/mod.rs` | 632, 763 | Use SegmentId | P5 |

---

## Verification Steps

### 1. Config Validation
```bash
# Verify configs parse correctly
cargo test --package config -- --test-threads=1
```

### 2. Unit Tests
```bash
# Run core crate tests
cargo test --package core

# Run tools crate tests
cargo test --package tools
```

### 3. Integration Test
```bash
# Run full agent tests
cargo test --package agent -- --test-threads=1
```

### 4. Manual Verification
- Start the agent with gold_loan domain
- Verify segment detection still works for all 9 segments
- Verify persona adaptation responds correctly
- Verify tools accept both old and new parameter names

### 5. Domain Switch Test (Critical for Domain-Agnostic Goal)
- Create minimal test domain config:
  ```
  config/domains/test_domain/
  ├── domain.yaml
  ├── segments.yaml
  ├── personas.yaml
  └── tools/schemas.yaml
  ```
- Set `DOMAIN_ID=test_domain`
- Verify agent starts with new domain
- Verify NO gold-loan specific behavior leaks through

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Breaking existing behavior | Keep deprecated CustomerSegment enum for one release cycle |
| Old integrations fail | Tool parameter aliases ensure backward compatibility |
| Large refactor scope | Each tier can be merged independently as separate PRs |
| Runtime errors | Add config validation at startup to catch missing fields |

---

## Success Criteria

- [ ] No hardcoded domain-specific terms in Rust source code (except deprecation stubs)
- [ ] All 9 segment definitions come from segments.yaml (including women, professional)
- [ ] All persona configurations come from segments.yaml and personas.yaml
- [ ] Tool parameters support config-driven aliases (no hardcoded gold_weight_grams)
- [ ] Tool defaults come from config (no hardcoded 100000, 12, etc.)
- [ ] New domain can be onboarded with ONLY YAML config changes
- [ ] All existing tests pass
- [ ] No regression in agent behavior with gold_loan domain

---

## Appendix: Current vs Target State

### CustomerSegment

| Current (Enum) | Target (Config) |
|----------------|-----------------|
| 6 hardcoded variants | Any number from segments.yaml |
| Rust code change to add segment | YAML change only |
| key_messages() in Rust | key_messages in YAML |
| suggested_warmth() in Rust | persona.warmth in YAML |

### Persona

| Current | Target |
|---------|--------|
| for_segment() match statement | from_config() loading from YAML |
| Hardcoded tone phrases | Localized phrases in personas.yaml |
| 6 segment mappings | Unlimited from config |

### Tools

| Current | Target |
|---------|--------|
| Hardcoded "gold_weight_grams" alias | Config-driven parameter_aliases |
| Hardcoded defaults (100000, 12) | tool_defaults in config |
| Hardcoded constraints (10.0-30.0) | Constraint ranges in config |
