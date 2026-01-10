# Domain-Agnostic Architecture Deep Analysis

**Date:** 2026-01-09
**Purpose:** Comprehensive audit of domain-specific content that must be abstracted for true domain-agnostic operation
**Goal:** Enable new business onboarding purely through YAML configuration

---

## Executive Summary

The codebase has a **well-designed domain-agnostic architecture** with config-driven components, but **critical domain leakage** exists in several modules that prevents true domain-agnostic operation. This document identifies all hardcoded domain content and provides actionable recommendations.

### Overall Assessment

| Category | Status | Impact |
|----------|--------|--------|
| Core Traits | ✅ Excellent | Fully generic |
| Config Loading | ✅ Excellent | DomainBridge pattern works well |
| YAML Configs | ✅ Good | Proper separation in domains/ folder |
| Memory Compressor | ❌ Critical | Hardcoded keywords block reuse |
| DST Module | ❌ Critical | Gold-specific purity functions |
| Persuasion Module | ⚠️ Medium | Hardcoded "lakh" currency unit |
| Python Services | ⚠️ Medium | Whisper prompts contain domain terms |
| Test Data | ℹ️ Low | Domain examples in tests (acceptable) |

---

## Part 1: Critical Domain Leakage - Rust Code

### 1.1 Memory Compressor (CRITICAL)

**File:** `/voice-agent/backend/crates/agent/src/memory/compressor.rs`

#### Hardcoded Keyword List (Lines 148-157)

```rust
for kw in &[
    "gold", "loan", "rate", "interest", "emi", "branch", "weight",
    "gram", "grams", "tola", "purity", "karat", "22k", "24k", "18k",
    "sona", "karj", "byaj", "rin", "gehne", "jewelry",
    "lakh", "crore", "rupees", "amount", "kotak", "muthoot", "manappuram",
    "eligibility", "document", "disbursal", "repayment", "tenure",
] {
    domain_keywords.insert(kw.to_string());
}
```

**Impact:** Cannot reuse memory compressor for insurance, credit cards, or other domains.

#### Entity Pattern Mappings (Lines 159-180)

| Line | Entity Type | Hardcoded Values |
|------|-------------|------------------|
| 162-165 | `asset_quantity` | "gram", "grams", "gm", "tola" |
| 167-170 | `amount` | "lakh", "crore", "rupees", "rs" |
| 172-175 | `asset_quality` | "22k", "24k", "18k", "karat" |
| 177-180 | `competitor` | "muthoot", "manappuram", "iifl", "hdfc", "sbi" |

#### Intent Keywords (Lines 427-434)

```rust
let intent_keywords: HashMap<&str, Vec<&str>> = [
    ("rate_inquiry", vec!["rate", "interest", "byaj", "percent"]),
    ("loan_inquiry", vec!["loan", "borrow", "karj", "rin"]),
    ("competitor", vec!["muthoot", "manappuram", "better", "compare"]),
    // ...
].into_iter().collect();
```

#### DST Display Mappings (Lines 489-502)

```rust
let display_key = match k.as_str() {
    "asset_quantity" | "gold_weight" | "weight" => "Asset",
    "asset_quality" | "gold_purity" | "purity" => "Quality",
    "competitor" | "current_lender" => "Competitor",
    // ...
};
```

**Recommendation:** Create `memory_config.yaml` per domain with:
- `keywords`: Domain-specific priority terms
- `entity_patterns`: Entity type to pattern mappings
- `intent_keywords`: Intent detection keywords
- `display_mappings`: Slot name to display label mappings

---

### 1.2 DST Slots Module (CRITICAL)

**File:** `/voice-agent/backend/crates/agent/src/dst/slots.rs`

#### Purity Functions (Lines 8-47)

```rust
pub type PurityId = String;

pub mod purity_ids {
    pub const K24: &str = "24k";
    pub const K22: &str = "22k";
    pub const K18: &str = "18k";
    pub const K14: &str = "14k";
    pub const UNKNOWN: &str = "unknown";
}

pub fn parse_purity_id(text: &str) -> &'static str {
    let lower = text.to_lowercase();
    if lower.contains("24") { purity_ids::K24 }
    else if lower.contains("22") { purity_ids::K22 }
    // ... hardcoded gold karat parsing
}

pub fn format_purity_display(purity_id: &str) -> &'static str {
    match purity_id {
        "24k" => "24 karat",
        "22k" => "22 karat",
        // ... hardcoded gold karat formatting
    }
}
```

#### Public Exports (Lines 49-52 in mod.rs)

```rust
pub use slots::{
    PurityId, purity_ids, parse_purity_id, format_purity_display,
};
```

**Impact:** These gold-specific functions are exported as public API, making the entire DST module gold-loan specific.

**Recommendation:**
1. Rename to generic `QualityId`, `quality_tiers`
2. Load quality tier definitions from config
3. Create `parse_quality_id()` that reads patterns from config
4. Create `format_quality_display()` that reads labels from config

---

### 1.3 Persuasion Module (MEDIUM)

**File:** `/voice-agent/backend/crates/agent/src/persuasion.rs`

#### Hardcoded Currency Unit (Lines 500-514)

```rust
// Line 270: /// Monthly savings per unit amount (e.g., per lakh)
// Line 500: // Calculate monthly savings per lakh
let monthly_savings_per_lakh = (rate_diff / 100.0 / 12.0) * 100_000.0;
// Line 514:
savings_unit_amount: 100_000.0,  // 1 lakh hardcoded
```

#### Hardcoded Amounts (Lines 424, 447)

```rust
our_base_rate: view.our_rate_for_amount(500_000.0),  // 5 lakh
our_base_rate: 9.5,  // Default rate
```

**Impact:** Cannot use for international markets with different currency units.

**Recommendation:**
1. Load `savings_unit_amount` from config
2. Load `default_base_rate` from config
3. Load currency display format from config (lakh vs thousand)

---

### 1.4 Prompt Module (MEDIUM - Deprecated)

**File:** `/voice-agent/backend/crates/llm/src/prompt.rs`

#### Deprecated System Prompt (Lines 299-338)

```rust
let agent_name = "Priya";       // Hardcoded
let company_name = "Kotak";     // Hardcoded

let system_content = format!(
    "You are {agent_name}, a helpful voice assistant for {company_name} gold loan services."
);
```

#### ProductFacts Defaults (Lines 180-202)

```rust
impl Default for ProductFacts {
    fn default() -> Self {
        Self {
            our_rate: 10.5,           // Kotak rate
            nbfc_rate_low: 18.0,      // Competitor rate
            nbfc_rate_high: 24.0,     // Competitor rate
            ltv_percent: 75.0,        // Gold loan LTV
        }
    }
}
```

**Status:** Marked as deprecated, has config-driven replacement `system_prompt_from_config()`.

**Recommendation:** Remove deprecated code after migration verification.

---

### 1.5 Conversation Module (MEDIUM)

**File:** `/voice-agent/backend/crates/agent/src/conversation.rs`

#### Intent-to-Stage Mappings (Lines 608-735)

```rust
"loan_inquiry" | "eligibility_query" => match current {
    ConversationStage::Greeting => Some(ConversationStage::Discovery),
    // ...
},
"interest_rate_query" => match current {
    ConversationStage::Greeting | ConversationStage::Discovery => {
        Some(ConversationStage::Presentation)
    },
},
```

**Impact:** Gold loan intents hardcoded in stage transitions.

**Recommendation:** Load intent-to-stage mappings from `stages.yaml` config.

---

## Part 2: Domain Leakage - Python Code

### 2.1 Whisper Service (MEDIUM)

**File:** `/voice-agent/backend/services/whisper_service.py`

#### Gold Loan Prompts (Lines 43-70)

```python
GOLD_LOAN_PROMPT = """
Kotak Mahindra Bank gold loan
Interest rate for gold loan
Muthoot Finance
Manappuram
IIFL
EMI calculator
LTV ratio
Processing fee
foreclosure charges
Doorstep gold loan service
"""

GOLD_LOAN_PROMPT_HINDI = """
Kotak gold loan ke baare mein batao
Muthoot
Manappuram
"""
```

**Impact:** STT contextual biasing is hardcoded for gold loan vocabulary.

**Recommendation:** Load vocabulary prompts from domain config file.

---

### 2.2 RAG Knowledge Loader (MEDIUM)

**File:** `/voice-agent/backend/scripts/load_rag_knowledge.py`

#### Hardcoded Collection (Line 12)

```python
COLLECTION_NAME = "gold_loan_knowledge"
```

#### Hardcoded Knowledge (Lines 96-250)

- Interest rates: 11.5%, 10.5%, 9.5%
- Competitor rates: Muthoot 18%, Manappuram 19%, IIFL 17.5%
- Loan amounts: Rs 10,000 to Rs 2.5 crore
- LTV: 75%
- Processing fee: 1%

**Recommendation:** Load all knowledge content from YAML files in knowledge/ directory.

---

### 2.3 Knowledge YAML Files

**Location:** `/voice-agent/backend/knowledge/`

| File | Content |
|------|---------|
| `rates.yaml` | Interest rates, competitor comparisons |
| `products.yaml` | Kotak Standard Gold, Shakti Gold, etc. |
| `competitors.yaml` | Muthoot, Manappuram, IIFL details |
| `eligibility.yaml` | Age 21-65, gold purity 18K+, LTV 75% |
| `faqs.yaml` | Gold loan Q&A |
| `process.yaml` | Application process |
| `branches.yaml` | Kotak branch locations |
| `safety.yaml` | Vault security, RBI compliance |

**Status:** ✅ Properly externalized to config (good).

---

## Part 3: Well-Architected Areas

### 3.1 Core Traits (Excellent)

All business logic traits are fully generic:

| Trait | File | Status |
|-------|------|--------|
| `DomainCalculator` | `core/src/traits/calculator.rs` | ✅ Config-driven |
| `SlotSchema` | `core/src/traits/slots.rs` | ✅ Config-driven |
| `ConversationGoalSchema` | `core/src/traits/goals.rs` | ✅ Config-driven |
| `LeadScoringStrategy` | `core/src/traits/scoring.rs` | ✅ Config-driven |
| `CompetitorAnalyzer` | `core/src/traits/competitors.rs` | ✅ Config-driven |
| `ObjectionHandler` | `core/src/traits/objections.rs` | ✅ Config-driven |
| `SegmentDetector` | `core/src/traits/segments.rs` | ✅ Config-driven |

### 3.2 Configuration Loading (Excellent)

**DomainBridge Pattern:** `config/src/domain/bridge.rs`

```
YAML Config Files
    ↓
MasterDomainConfig (loads all YAML)
    ↓
DomainBridge (converts config to trait implementations)
    ↓
Arc<dyn DomainCalculator>
Arc<dyn LeadScoringStrategy>
Arc<dyn CompetitorAnalyzer>
...
```

### 3.3 Domain Config Structure (Good)

```
config/
├── base/
│   └── defaults.yaml           # Domain-agnostic base
├── domains/
│   └── gold_loan/
│       ├── domain.yaml         # Core rates, brand, competitors
│       ├── slots.yaml          # DST slot definitions
│       ├── stages.yaml         # Conversation flow
│       ├── scoring.yaml        # Lead scoring
│       ├── objections.yaml     # Objection handling
│       ├── competitors.yaml    # Competitor details
│       ├── segments.yaml       # Customer segments
│       ├── goals.yaml          # Goals and actions
│       ├── intent_tool_mappings.yaml
│       ├── prompts/
│       │   └── system.yaml
│       └── tools/
│           ├── schemas.yaml
│           ├── branches.yaml
│           ├── documents.yaml
│           ├── responses.yaml
│           └── sms_templates.yaml
```

---

## Part 4: Inventory of All Domain Terms Found

### 4.1 Financial Product Terms

| Term | Occurrences | Files |
|------|-------------|-------|
| gold loan | 500+ | RAG, knowledge, tests |
| interest rate | 200+ | calculator, config, tests |
| EMI | 100+ | calculator, tools |
| LTV | 50+ | calculator, config |
| processing fee | 30+ | config, tools |
| foreclosure | 20+ | config, compliance |
| disbursal | 15+ | config, stages |

### 4.2 Competitor Names

| Competitor | Occurrences | Critical Files |
|------------|-------------|----------------|
| Kotak | 100+ | prompts, config, tests |
| Muthoot | 50+ | **compressor.rs:153**, config |
| Manappuram | 40+ | **compressor.rs:153**, config |
| IIFL | 30+ | **compressor.rs:179**, config |
| HDFC | 20+ | compressor.rs, config |
| SBI | 15+ | compressor.rs, config |

### 4.3 Gold-Specific Terms

| Term | Occurrences | Critical Files |
|------|-------------|----------------|
| karat/carat | 30+ | **slots.rs:14-47**, config |
| 22k/24k/18k | 50+ | **compressor.rs:150**, slots.rs |
| gram/grams | 40+ | **compressor.rs:149**, tests |
| tola | 10+ | **compressor.rs:149**, config |
| purity | 30+ | **slots.rs:8-47**, config |
| hallmark | 10+ | config, knowledge |

### 4.4 Indian Currency Terms

| Term | Occurrences | Critical Files |
|------|-------------|----------------|
| lakh | 30+ | **persuasion.rs:502**, compressor |
| crore | 20+ | **compressor.rs:152**, config |
| rupees/Rs | 40+ | compressor, config |

### 4.5 Hindi Financial Terms

| Term | Meaning | Files |
|------|---------|-------|
| sona | gold | compressor.rs:151 |
| karj | loan | compressor.rs:151 |
| byaj | interest | compressor.rs:151 |
| rin | loan | compressor.rs:151 |
| gehne | jewelry | compressor.rs:151 |

---

## Part 5: Action Plan for True Domain-Agnostic Operation

### Priority 1: Critical (Blocks Domain Reuse)

#### 1.1 Refactor Memory Compressor

**Create:** `config/domains/{domain}/memory.yaml`

```yaml
# memory.yaml
keywords:
  - gold
  - loan
  - rate
  # ... loaded from config

entity_patterns:
  asset_quantity:
    - gram
    - grams
    - tola
  asset_quality:
    - 22k
    - 24k
    - karat
  competitor:
    - muthoot
    - manappuram
    - iifl

intent_keywords:
  rate_inquiry:
    - rate
    - interest
    - byaj
  loan_inquiry:
    - loan
    - borrow
    - karj

display_mappings:
  asset_quantity: "Asset"
  asset_quality: "Quality"
  competitor: "Competitor"
```

**Change:** `ExtractiveCompressor::new()` to accept config.

#### 1.2 Refactor DST Purity Functions

**Current:** Hardcoded `PurityId`, `purity_ids`, `parse_purity_id()`, `format_purity_display()`

**Target:** Generic `QualityId` system loaded from config:

```yaml
# slots.yaml
quality_tiers:
  - id: "24k"
    display: "24 karat"
    patterns: ["24", "24k", "24 karat", "pure"]
  - id: "22k"
    display: "22 karat"
    patterns: ["22", "22k", "22 karat"]
  - id: "18k"
    display: "18 karat"
    patterns: ["18", "18k", "18 karat"]
```

#### 1.3 Refactor Persuasion Module

**Add to domain config:**

```yaml
# domain.yaml
currency:
  savings_unit_amount: 100000  # 1 lakh
  savings_unit_name: "lakh"
  symbol: "Rs"

defaults:
  base_rate: 9.5
  reference_amount: 500000
```

### Priority 2: Important (Code Clarity)

#### 2.1 Move Intent-Stage Mappings to Config

**Add to stages.yaml:**

```yaml
intent_transitions:
  loan_inquiry:
    from: [greeting, discovery]
    to: qualification
  interest_rate_query:
    from: [greeting, discovery]
    to: presentation
  competitor_reference:
    from: [discovery, qualification, presentation]
    to: objection_handling
```

#### 2.2 Move Whisper Vocabulary to Config

**Create:** `config/domains/{domain}/stt.yaml`

```yaml
vocabulary_prompts:
  en:
    - "Kotak Mahindra Bank gold loan"
    - "Interest rate for gold loan"
    - "Muthoot Finance"
    - "Manappuram"
  hi:
    - "Kotak gold loan ke baare mein batao"
    - "Muthoot"
    - "Manappuram"
```

#### 2.3 Remove Deprecated Code

- Delete `system_prompt()` in prompt.rs (lines 299-338)
- Delete `ProductFacts::default()` hardcoded values
- Update tests to use config-driven approach

### Priority 3: Nice-to-Have

#### 3.1 Update Test Data

- Replace `gold_loan_calculator()` with `test_calculator()`
- Use generic slot names in test assertions
- Add domain-agnostic test fixtures

#### 3.2 Documentation Updates

- Update comments referencing "gold loan domain" to "domain-specific"
- Add architecture diagrams showing config flow
- Create onboarding guide for new domains

---

## Part 6: New Domain Onboarding Checklist

After implementing Priority 1 changes, onboarding a new domain (e.g., "auto_loan") requires:

### Required Files

```
config/domains/auto_loan/
├── domain.yaml              # Brand, rates, constants
├── slots.yaml               # Slot definitions + quality_tiers
├── stages.yaml              # Conversation flow + intent_transitions
├── competitors.yaml         # Competitor details
├── goals.yaml               # Goals and actions
├── scoring.yaml             # Lead scoring thresholds
├── objections.yaml          # Objection handling
├── memory.yaml              # Keywords, entity patterns (NEW)
├── stt.yaml                 # Vocabulary prompts (NEW)
├── intent_tool_mappings.yaml
├── prompts/
│   └── system.yaml
└── tools/
    ├── schemas.yaml
    └── responses.yaml
```

### Environment Setup

```bash
export DOMAIN_ID=auto_loan
cargo run
```

### No Code Changes Required For:

- ✅ Interest rates and loan limits
- ✅ Processing fees and collateral valuation factors
- ✅ Competitor information and comparison
- ✅ Conversation stages and transitions
- ✅ Lead scoring thresholds
- ✅ Tool schemas and responses
- ✅ Prompt templates and system instructions
- ✅ Slot definitions and extraction patterns
- ✅ Customer segments and features
- ✅ STT vocabulary biasing
- ✅ Memory compressor keywords

### Code Changes Still Required For:

- ❌ New tool implementations (Rust code)
- ❌ New RAG retrieval strategies
- ❌ Custom ML model changes
- ❌ New authentication methods

---

## Part 7: Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    DOMAIN-AGNOSTIC LAYER                        │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Core Traits: DomainCalculator, SlotSchema, Goals, etc. │   │
│  └─────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Pipeline: STT, TTS, VAD, LLM, RAG                      │   │
│  └─────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Memory Compressor (needs config) ⚠️                     │   │
│  └─────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  DST Module (needs refactor) ⚠️                          │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    CONFIGURATION BRIDGE                          │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  DomainBridge: YAML → Arc<dyn Trait>                    │   │
│  └─────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  MasterDomainConfig: Load all YAML files                │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    DOMAIN CONFIGURATION                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │  gold_loan/  │  │  auto_loan/  │  │  insurance/  │         │
│  │  domain.yaml │  │  domain.yaml │  │  domain.yaml │         │
│  │  slots.yaml  │  │  slots.yaml  │  │  slots.yaml  │         │
│  │  stages.yaml │  │  stages.yaml │  │  stages.yaml │         │
│  │  ...         │  │  ...         │  │  ...         │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

---

## Conclusion

The codebase is **85% domain-agnostic** with excellent trait-based abstractions and config-driven components. However, **critical domain leakage** in 3 modules blocks true domain-agnostic operation:

1. **Memory Compressor** - Hardcoded keywords and entity patterns
2. **DST Slots** - Hardcoded purity/quality functions
3. **Persuasion** - Hardcoded currency unit

Implementing the Priority 1 changes (~2-3 days of work) will enable:
- Pure YAML-based domain onboarding
- No code changes for new business verticals
- Reusable infrastructure across loan types, insurance, and other products

---

## Appendix: File Reference

### Critical Files Requiring Changes

| File | Lines | Issue |
|------|-------|-------|
| `crates/agent/src/memory/compressor.rs` | 148-180, 427-434, 489-502 | Hardcoded keywords/patterns |
| `crates/agent/src/dst/slots.rs` | 8-47 | Hardcoded purity functions |
| `crates/agent/src/persuasion.rs` | 270, 500-514 | Hardcoded lakh unit |
| `services/whisper_service.py` | 43-70 | Hardcoded vocabulary |
| `crates/llm/src/prompt.rs` | 180-202, 299-338 | Deprecated hardcoded prompts |
| `crates/agent/src/conversation.rs` | 608-735 | Hardcoded intent mappings |

### Well-Architected Files (No Changes Needed)

| File | Purpose |
|------|---------|
| `crates/core/src/traits/*.rs` | All domain-agnostic traits |
| `crates/config/src/domain/bridge.rs` | Config to trait conversion |
| `config/domains/gold_loan/*.yaml` | Properly externalized config |
| `crates/config/src/settings.rs` | Generic settings loading |
