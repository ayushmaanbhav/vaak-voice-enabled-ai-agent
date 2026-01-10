# Domain-Agnostic Voice Agent - Comprehensive Analysis

**Date**: 2026-01-09
**Purpose**: Deep analysis of hardcoded domain-specific code and config wiring gaps

---

## Executive Summary

This document provides a comprehensive audit of the voice agent backend codebase to identify all domain-specific hardcoding that prevents true domain-agnostic operation. The goal is to enable onboarding new businesses by defining YAML configs only.

**Key Finding**: Significant config infrastructure already exists but is not fully utilized. The primary work is completing the wiring between existing config and code.

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Hardcoded Domain-Specific Code](#2-hardcoded-domain-specific-code)
3. [Config System Analysis](#3-config-system-analysis)
4. [Gap Analysis](#4-gap-analysis)
5. [Recommendations](#5-recommendations)

---

## 1. Architecture Overview

### 1.1 Config Loading Pipeline

```
YAML Files (config/domains/{domain_id}/)
    ↓
MasterDomainConfig (config/src/domain/master.rs)
    ↓
Domain Views (AgentDomainView, LlmDomainView, ToolsDomainView)
    ↓
Business Logic (agent, tools, text_processing crates)
```

### 1.2 Config Files Structure

The gold_loan domain has 23 YAML config files:

| File | Purpose | Status |
|------|---------|--------|
| `domain.yaml` | Brand, constants, competitors, vocabulary | Good |
| `stages.yaml` | Conversation stages, intent transitions | Has hardcoded strings |
| `slots.yaml` | Slot definitions (domain-agnostic names) | Excellent |
| `features.yaml` | Feature definitions per segment | Good |
| `segments.yaml` | Customer segment detection | Has hardcoded rates |
| `objections.yaml` | Objection handling | Excellent (uses variables) |
| `lead_scoring.yaml` | Lead qualification | Good |
| `adaptation.yaml` | Variable definitions | Needs more variables |
| `extraction_patterns.yaml` | Text extraction patterns | Missing repayment patterns |
| `compliance.yaml` | Regulatory rules | Missing AI disclosures |
| `competitors.yaml` | Competitor data | Has hardcoded rates |
| `intent_tool_mappings.yaml` | Intent to tool mappings | Good |
| `tools/*.yaml` | Tool schemas, branches, SMS | Good |
| `prompts/system.yaml` | LLM prompts | Needs review |

---

## 2. Hardcoded Domain-Specific Code

### 2.1 CRITICAL (P0) - Blocks Multi-Domain

#### 2.1.1 Intent-to-Stage Transitions (conversation.rs)

**Location**: `voice-agent/backend/crates/agent/src/conversation.rs:670-767`

**Issue**: 100-line hardcoded match statement for intent-to-stage transitions:

```rust
let new_stage = match intent.intent.as_str() {
    "greeting" if current == ConversationStage::Greeting => { ... },
    "loan_inquiry" | "eligibility_query" => match current { ... },
    "interest_rate_query" => match current { ... },
    // ... 15+ hardcoded intent names
};
```

**Config Exists**: `stages.yaml` lines 239-322 has `intent_transitions` section with identical mappings.

**Gap**: Code doesn't load from config.

#### 2.1.2 Hardcoded Company Name in Stages (stages.yaml)

**Location**: `config/domains/gold_loan/stages.yaml`

**Issues**:
- Line 13: `"Introduce yourself as a Kotak Gold Loan specialist"`
- Line 18: `"Thank you for your interest in Kotak Gold Loan!"`
- Line 87: `"Present Kotak's gold loan benefits"`

**Fix**: Use `{{company_short_name}}` and `{{product_name}}` variables.

#### 2.1.3 Hardcoded Rates in Stages (stages.yaml)

**Location**: `config/domains/gold_loan/stages.yaml:88-91`

**Issues**:
```yaml
- Competitive interest rates (starting from 10.49% p.a.)
- Up to 75% LTV
```

**Fix**: Use `{{our_standard_rate}}` and `{{ltv_percent}}` variables.

#### 2.1.4 AI Disclosure Messages (conversation.rs)

**Location**: `voice-agent/backend/crates/agent/src/conversation.rs:251-264`

**Issue**: 8 language-specific disclosure messages hardcoded in Rust:
```rust
static AI_DISCLOSURES: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("en", "This call may be recorded...");
    m.insert("hi", "इस कॉल को रिकॉर्ड किया जा सकता है...");
    // ... 6 more languages
});
```

**Fix**: Move to `compliance.yaml` with `ai_disclosures` section.

---

### 2.2 HIGH (P1) - Domain Coupling

#### 2.2.1 Urgency Keywords (lead_scoring.rs)

**Location**: `voice-agent/backend/crates/agent/src/lead_scoring.rs:732-745`

**Issue**:
```rust
let urgency_keywords = ["urgent", "urgently", "immediately", "today",
    "now", "asap", "emergency", "jaldi", "abhi", "turant", "aaj", "foran"];
```

**Note**: Config support exists (lines 719-728) but fallback to hardcoded list.

#### 2.2.2 Intent-to-Signal Mappings (lead_scoring.rs)

**Location**: `voice-agent/backend/crates/agent/src/lead_scoring.rs:605-649`

**Issue**: Hardcoded intent names for signal generation:
```rust
"loan_inquiry" | "eligibility_query" => { ... },
"interest_rate_query" => { ... },
"switch_lender" | "balance_transfer" => { ... },
```

#### 2.2.3 Repayment Patterns (slot_extraction/mod.rs)

**Location**: `voice-agent/backend/crates/text_processing/src/slot_extraction/mod.rs:137-175`

**Issue**: Static regex patterns for repayment type detection:
```rust
static REPAYMENT_PATTERNS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| vec![
    (Regex::new(r"(?i)(?:repay|payment\s+(?:option|method)|EMI...)"), "repayment_inquiry"),
]);
```

**Fix**: Add `repayment_types` section to `extraction_patterns.yaml`.

#### 2.2.4 Query Intent Classification (domain_boost.rs)

**Location**: `voice-agent/backend/crates/rag/src/domain_boost.rs:240-272`

**Issue**: `default_intent_patterns()` with hardcoded keywords for RAG boosting.

**Fix**: Add `intent_patterns` section to `domain.yaml` domain_boost config.

---

### 2.3 MEDIUM (P2) - Partial Config

#### 2.3.1 Hardcoded Rates in Segments (segments.yaml)

**Location**: `config/domains/gold_loan/segments.yaml`

**Issues**:
- Lines 46, 50-51: `"9.5% p.a."` in value propositions
- Lines 139, 144: Same

**Fix**: Use `{{our_best_rate}}` variable.

#### 2.3.2 Hardcoded Rates in Competitors (competitors.yaml)

**Location**: `config/domains/gold_loan/competitors.yaml`

**Issues**:
- Lines 223-224: `"Starting from 9.5% p.a. vs 12-24% at NBFCs"`
- Lines 251-252: `"Competitive interest rate from 10.49% p.a."`

**Fix**: Use rate variables.

#### 2.3.3 Memory Compression Slot Names (memory/mod.rs)

**Location**: `voice-agent/backend/crates/agent/src/memory/mod.rs:286-296`

**Issue**: Hardcoded list of slot names for display label lookup.

**Fix**: Use `view.all_slot_display_labels()` from config.

---

## 3. Config System Analysis

### 3.1 What's Working Well

#### Domain-Agnostic Slot Names
`slots.yaml` uses excellent generic naming:
```yaml
asset_quantity:    # not gold_weight_grams
asset_quality_tier: # not gold_purity
offer_amount:      # not loan_amount
```

#### Variable Substitution System
`adaptation.yaml` defines variables that can be used as `{{variable_name}}`:
```yaml
variables:
  collateral_type: "gold"
  special_program_name: "Shakti Gold"
  regulator_name: "RBI"
```

#### Objection Handling
`objections.yaml` uses brand variables properly:
```yaml
reframe: "{brand.bank_name} has state-of-the-art vaults..."
```

#### Tool Schemas
All tool implementations (eligibility.rs, price.rs, savings.rs) are fully config-driven with fallbacks only for missing config.

#### Competitor Data
All competitor information loaded from `competitors.yaml`, including rates, aliases, and comparison points.

### 3.2 What's Missing

#### Variables Not Defined
`adaptation.yaml` is missing:
- `company_name`, `company_short_name`
- `product_name`, `agent_role`
- `our_best_rate`, `our_standard_rate`
- `ltv_percent`, `switch_program_name`

#### Config Not Wired
- `stages.yaml` intent_transitions not loaded in `conversation.rs`
- `extraction_patterns.yaml` missing repayment patterns
- `domain.yaml` domain_boost missing intent_patterns
- `compliance.yaml` missing AI disclosures

---

## 4. Gap Analysis

### 4.1 Config vs Code Usage Matrix

| Config Section | Defined | Loaded | Used | Gap |
|---------------|---------|--------|------|-----|
| Brand constants | Yes | Yes | Yes | None |
| Interest rates | Yes | Yes | Yes | None |
| Competitors | Yes | Yes | Yes | None |
| Slot definitions | Yes | Yes | Yes | None |
| Intent transitions | Yes | Yes | **No** | Code ignores config |
| Objection responses | Yes | Yes | Yes | None |
| Lead scoring | Yes | Yes | Partial | Some hardcoded |
| Extraction patterns | Partial | Partial | Partial | Missing repayment |
| AI disclosures | **No** | No | Hardcoded | Need config |
| Stage guidance text | Yes | Yes | Yes | Has hardcoded strings |

### 4.2 Variable Substitution Coverage

| File | Uses Variables | Hardcoded Values |
|------|---------------|------------------|
| `objections.yaml` | Yes (`{brand.*}`) | None |
| `features.yaml` | Yes (`{{...}}`) | None |
| `stages.yaml` | **No** | Company name, rates |
| `segments.yaml` | **No** | Rates |
| `competitors.yaml` | **No** | Rates |
| `prompts/system.yaml` | Partial | Likely some |

---

## 5. Recommendations

### 5.1 Immediate Actions (P0)

1. **Wire intent transitions**: Modify `conversation.rs` to use `stages.yaml` config
2. **Add brand variables**: Add company_name, product_name to `adaptation.yaml`
3. **Replace hardcoded strings**: Update `stages.yaml` with variable references
4. **Move AI disclosures**: Add to `compliance.yaml` and load from config

### 5.2 Short-term Actions (P1)

1. **Repayment patterns**: Add to `extraction_patterns.yaml` and wire
2. **Query intent patterns**: Add to `domain.yaml` domain_boost section
3. **Ensure urgency keywords**: Always load from config, remove fallback
4. **Memory compression**: Use config for slot display labels

### 5.3 Medium-term Actions (P2)

1. **Rate variables**: Add and use in segments.yaml, competitors.yaml
2. **Validate config references**: Ensure all intent names match definitions
3. **Create domain template**: Document minimum config for new domain

### 5.4 Validation Checklist

When complete, these should pass:
```bash
# No hardcoded company names
grep -rn "Kotak" voice-agent/backend/crates/ --include="*.rs" | grep -v test

# No hardcoded rates in YAML (outside comments)
grep -rn "[0-9]\+\.[0-9]\+% p\.a\." config/domains/gold_loan/*.yaml | grep -v "^#"

# All intent names in config
# Run validation script to check intent coverage
```

---

## Appendix A: Files to Modify

### Rust Files

| File | Priority | Changes |
|------|----------|---------|
| `agent/src/conversation.rs` | P0 | Wire stages_config, load AI disclosures |
| `config/src/domain/master.rs` | P0 | Add substitute_all_variables() |
| `config/src/domain/views.rs` | P1 | Add all_slot_display_labels() |
| `config/src/domain/extraction_patterns.rs` | P1 | Add RepaymentTypesConfig |
| `text_processing/src/slot_extraction/mod.rs` | P1 | Load patterns from config |
| `rag/src/domain_boost.rs` | P1 | Load intent patterns from config |
| `agent/src/memory/mod.rs` | P2 | Use config for slot labels |
| `agent/src/lead_scoring.rs` | P2 | Remove hardcoded fallbacks |

### Config Files

| File | Priority | Changes |
|------|----------|---------|
| `adaptation.yaml` | P0 | Add brand, rate, program variables |
| `stages.yaml` | P0 | Replace hardcoded strings with variables |
| `compliance.yaml` | P0 | Add ai_disclosures section |
| `segments.yaml` | P2 | Replace hardcoded rates |
| `competitors.yaml` | P2 | Replace hardcoded rates |
| `extraction_patterns.yaml` | P1 | Add repayment_types section |
| `domain.yaml` | P1 | Add domain_boost.intent_patterns |

---

## Appendix B: New Config Schema Additions

### B.1 AI Disclosures (compliance.yaml)

```yaml
ai_disclosures:
  en: "This call may be recorded for quality and training purposes."
  hi: "इस कॉल को क्वालिटी और प्रशिक्षण के लिए रिकॉर्ड किया जा सकता है।"
  ta: "இந்த அழைப்பு தரம் மற்றும் பயிற்சி நோக்கங்களுக்காக பதிவு செய்யப்படலாம்."
  te: "ఈ కాల్ నాణ్యత మరియు శిక్షణ ప్రయోజనాల కోసం రికార్డ్ చేయబడవచ్చు."
  kn: "ಈ ಕರೆಯನ್ನು ಗುಣಮಟ್ಟ ಮತ್ತು ತರಬೇತಿ ಉದ್ದೇಶಗಳಿಗಾಗಿ ರೆಕಾರ್ಡ್ ಮಾಡಬಹುದು."
  ml: "ഈ കോൾ ഗുണനിലവാരത്തിനും പരിശീലനത്തിനും വേണ്ടി റെക്കോർഡ് ചെയ്യാവുന്നതാണ്."
  mr: "हा कॉल गुणवत्ता आणि प्रशिक्षण हेतूंसाठी रेकॉर्ड केला जाऊ शकतो."
  gu: "આ કોલ ગુણવત્તા અને તાલીમ હેતુઓ માટે રેકોર્ડ થઈ શકે છે."
```

### B.2 Repayment Types (extraction_patterns.yaml)

```yaml
repayment_types:
  categories:
    - id: emi
      display_name: "EMI"
      patterns:
        en: ["EMI", "monthly\\s+(?:payment|installment)", "equated"]
        hi: ["महीना", "किश्ते", "kishte", "mahina"]
    - id: bullet
      display_name: "Bullet Payment"
      patterns:
        en: ["bullet", "lump\\s*sum", "one\\s+time", "single\\s+payment"]
        hi: ["एकमुश्त", "ekmusht", "ek\\s+baar"]
    - id: overdraft
      display_name: "Overdraft"
      patterns:
        en: ["overdraft", "OD", "credit\\s+line", "flexible\\s+repay"]
    - id: interest_only
      display_name: "Interest Only"
      patterns:
        en: ["interest\\s+only", "only\\s+interest", "pay\\s+interest"]
        hi: ["sirf\\s+byaaj", "केवल\\s+ब्याज"]
```

### B.3 Intent Patterns for RAG Boost (domain.yaml)

```yaml
domain_boost:
  intent_patterns:
    rate_inquiry:
      keywords: ["interest", "rate", "byaj", "dar", "percent", "p.a."]
      boost: 1.8
    eligibility:
      keywords: ["eligible", "eligibility", "qualify", "criteria", "patrta"]
      boost: 1.7
    application:
      keywords: ["apply", "application", "process", "aavedan", "how to"]
      boost: 1.5
    amount:
      keywords: ["amount", "maximum", "how much", "kitna", "limit"]
      boost: 1.5
    branch:
      keywords: ["branch", "nearest", "location", "shakha", "office"]
      boost: 1.4
    documents:
      keywords: ["document", "papers", "kyc", "dastavez", "required"]
      boost: 1.4
    competitor:
      keywords: ["compare", "better", "switch", "transfer", "vs"]
      include_competitor_names: true
      boost: 1.3
    repayment:
      keywords: ["repay", "prepay", "foreclosure", "emi", "close", "settle"]
      boost: 1.4
```

---

## Appendix C: Variable Reference

### Brand Variables
| Variable | Purpose | Example |
|----------|---------|---------|
| `{{company_name}}` | Full company name | Kotak Mahindra Bank |
| `{{company_short_name}}` | Short name | Kotak |
| `{{product_name}}` | Product name | Gold Loan |
| `{{agent_role}}` | Agent description | Gold Loan specialist |

### Rate Variables
| Variable | Purpose | Example |
|----------|---------|---------|
| `{{our_best_rate}}` | Best promotional rate | 9.5 |
| `{{our_standard_rate}}` | Standard starting rate | 10.49 |
| `{{ltv_percent}}` | Loan-to-value ratio | 75 |
| `{{nbfc_rate_range}}` | Typical NBFC rates | 12-24 |

### Program Variables
| Variable | Purpose | Example |
|----------|---------|---------|
| `{{special_program_name}}` | Women's program | Shakti Gold |
| `{{switch_program_name}}` | Balance transfer | Switch & Save |
| `{{regulator_name}}` | Regulatory body | RBI |

---

*End of Analysis Document*
