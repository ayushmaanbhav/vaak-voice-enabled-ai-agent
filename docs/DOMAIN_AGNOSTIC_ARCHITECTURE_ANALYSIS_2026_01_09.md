# Domain-Agnostic Architecture Analysis

**Date:** 2026-01-09
**Scope:** voice-agent/backend codebase
**Objective:** Ensure true domain-agnosticism for multi-business onboarding via YAML configs

---

## Executive Summary

This document provides a comprehensive analysis of the voice-agent backend codebase to identify all domain-specific (gold loan/banking/finance) code that needs to be abstracted behind configuration-driven layers. The goal is to enable onboarding new businesses/use cases by simply defining YAML configs without code changes.

### Current State
- **Partially Refactored**: Many areas have been refactored with P0-P16 fixes
- **Config-Driven Architecture**: Good foundation exists with domain configs, views, and traits
- **Remaining Issues**: ~47 files still contain hardcoded domain-specific terms
- **Test Data**: Significant domain-specific content in tests and examples

### Key Findings
| Category | Files Affected | Severity |
|----------|----------------|----------|
| Hardcoded Business Logic | 12 | HIGH |
| Domain-Specific Slot/Intent Names | 8 | MEDIUM |
| Competitor Names in Code | 15 | LOW |
| Test Data with Domain Terms | 25+ | LOW |
| Database Schema (gold_prices) | 3 | MEDIUM |

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Domain-Specific Code Inventory](#2-domain-specific-code-inventory)
3. [Hardcoded Business Logic](#3-hardcoded-business-logic)
4. [Slot and Intent Names](#4-slot-and-intent-names)
5. [Competitor References](#5-competitor-references)
6. [Database Schema Issues](#6-database-schema-issues)
7. [Config Structure Analysis](#7-config-structure-analysis)
8. [Feature Flags Implementation](#8-feature-flags-implementation)
9. [Recommendations](#9-recommendations)
10. [Action Items](#10-action-items)
11. [Appendix: File Reference](#appendix-file-reference)

---

## 1. Architecture Overview

### Current Config-Driven Architecture

```
voice-agent/backend/config/
├── default.yaml                    # Global defaults
├── production.yaml                 # Production overrides
├── base/defaults.yaml              # Domain-agnostic base
├── domains/
│   └── gold_loan/
│       ├── domain.yaml             # Master domain config
│       ├── intents.yaml            # Intent definitions
│       ├── slots.yaml              # DST slot definitions
│       ├── stages.yaml             # Conversation flow
│       ├── objections.yaml         # Objection handling
│       ├── competitors.yaml        # Competitor intelligence
│       ├── segments.yaml           # Customer segmentation
│       ├── features.yaml           # Product features
│       ├── scoring.yaml            # Lead scoring
│       ├── vocabulary.yaml         # Domain vocabulary
│       ├── prompts/system.yaml     # LLM prompts
│       └── tools/
│           ├── schemas.yaml        # Tool JSON schemas
│           ├── documents.yaml      # Document requirements
│           ├── branches.yaml       # Branch data
│           └── sms_templates.yaml  # SMS templates
└── rag.toml                        # RAG configuration
```

### View Pattern (Good Design)

```rust
// Each crate accesses config through specialized views:
AgentDomainView    → agent crate (conversation, DST, stages)
LlmDomainView      → llm crate (prompts, templates)
ToolsDomainView    → tools crate (tool schemas, business logic)
```

### Domain-Agnostic Naming Convention (Partially Applied)

| Domain-Specific | Generic Name | Status |
|-----------------|--------------|--------|
| `gold_weight` | `asset_quantity` | PARTIAL |
| `gold_purity` | `asset_quality_tier` | PARTIAL |
| `loan_amount` | `offer_amount` | NOT APPLIED |
| `bank_name` | `company_name` | DONE (P16) |
| `GoldPrice` | `AssetPrice` | PARTIAL (alias) |
| `GoldPurity` | `AssetVariant` | PARTIAL (alias) |

---

## 2. Domain-Specific Code Inventory

### 2.1 Core Crate (`voice-agent-core`)

**File: `crates/core/src/customer.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 124 | Serde alias | `alias = "gold_weight"` | Config-driven aliases |
| 128 | Serde alias | `alias = "gold_purity"` | Config-driven aliases |
| 300, 352 | Hardcoded threshold | `100 units collateral` | Move to config |
| 307, 369 | Hardcoded threshold | `500,000 (5 lakhs)` | Move to config |
| 502-522 | Price sensitivity patterns | Hindi: "byaj dar" | Config-driven patterns |
| 757-811 | Test data | "150 gram gold", "Is my gold safe?" | Parameterize tests |

**File: `crates/core/src/traits/calculator.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 476-482 | Rate tiers | Tier 1-3 with 1L, 5L boundaries | Config-driven tiers |
| 140, 161 | Function names | `test_gold_loan_calculator` | Generic naming |

**File: `crates/core/src/traits/competitors.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 16, 19, 58 | Example competitor | `"muthoot"` in docs | Use generic examples |
| 132 | Factory methods | Lists specific NBFCs | Config-driven factory |

**File: `crates/core/src/personalization/adaptation.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 35-36 | Feature enums | `RbiRegulated`, `ZeroForeclosure` | Config-driven features |
| 61 | Product name | `"Shakti Gold"` | From config |
| 89-91 | Objection enum | `GoldSafety` | Generic `AssetSecurity` |
| 323-326 | Marketing copy | "RBI regulated bank" | Config templates |
| 342, 351 | Hardcoded rates | "9.5%", "0.25% women's discount" | From config |

### 2.2 Tools Crate (`voice-agent-tools`)

**File: `crates/tools/src/domain_tools/tools/eligibility.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 15 | Tool name | `"check_eligibility"` | Config-driven tool name |
| 110 | Legacy alias | `"gold_weight_grams"` | Config aliases |
| 116 | Legacy alias | `"gold_purity"` | Config aliases |
| 137 | Response key | `"gold_value_inr"` | Generic `"asset_value"` |
| 147 | Response text | "loan up to ₹{} at {}%" | Config template |

**File: `crates/tools/src/domain_tools/tools/savings.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 56 | Description | "switching from NBFC to our gold loan" | Config description |
| 86 | Enum value | `"IIFL".into()` | From competitors config |
| 152 | Response text | Hardcoded savings message | Config template |

**File: `crates/tools/src/domain_tools/tools/price.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 14 | Tool name | `"get_price"` | Config-driven |
| 155-163 | Purity descriptions | "Pure gold (99.9%)" | From variants config |
| 190, 195 | Response text | "Current gold price is ₹{}" | Config template |

**File: `crates/tools/src/registry.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 217 | Comment | "Register gold loan tools" | Generic comment |
| 467, 520-521 | Service name | `gold_price_service` | `asset_price_service` |
| 678, 695, 714 | Tool assertions | `"get_gold_price"` | Config tool names |

### 2.3 Text Processing Crate

**File: `crates/text_processing/src/intent/mod.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 654, 660, 666 | Competitor patterns | `muthoot`, `manappuram`, `iifl` | Config patterns |
| 618-634 | Weight patterns | "grams", "tola" (11.66 multiplier) | Config units |
| 678-693 | Purity patterns | "22k", "24k", "18k" | Config variants |

**File: `crates/text_processing/src/entities/mod.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 112 | Field name | `gold_weight: Option<Weight>` | `asset_quantity` |
| 120 | Field name | `gold_purity: Option<u8>` | `asset_variant` |
| 122 | Field name | `current_lender` | OK (generic enough) |

**File: `crates/text_processing/src/slot_extraction/mod.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 73 | Pattern | "gold\|sona\|सोना" | Config vocabulary |
| 148-161 | Intent patterns | Domain-specific intents | Config intents |
| 787 | Context keywords | "gold", "kotak", "muthoot" | Config keywords |

**File: `crates/text_processing/src/sentiment/mod.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 7, 98 | Comments | "gold loan context" | Generic "domain context" |
| 321-357 | Patterns | "lower interest", "kam interest" | Config patterns |

**File: `crates/text_processing/src/compliance/rules.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 58-59 | Rate rules | `min_rate: 7.0, max_rate: 24.0` | Config compliance |
| 163-165 | Competitor list | Muthoot, IIFL, HDFC, SBI, ICICI | Config competitors |

### 2.4 Agent Crate

**File: `crates/agent/src/agent_config.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 167 | Default | `default_city: "Mumbai"` | Config default |
| 168 | Default | `default_gold_purity: "22K"` | Config default |
| 169 | Default | `default_competitor_rate: 18.0` | Config default |
| 170 | Default | `default_loan_amount: 100_000` | Config default |

**File: `crates/agent/src/agent/tools.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 38-82 | Intent mappings | Hardcoded intent→tool | Config mappings |
| 100-133 | Tool defaults | `gold_purity`, `loan_amount` | Config defaults |
| 265-281 | Slot mappings | `gold_weight`, `gold_purity` | Config mappings |

**File: `crates/agent/src/stage.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 340 | Stage requirement | `"current_lender"` | Config requirements |
| 349 | Stage requirement | `"gold_weight"` | Config requirements |

**File: `crates/agent/src/persuasion.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 49-59 | Objection types | `"gold_security"`, `"interest_rate"` | Config objections |
| 759 | Test data | `18.0` competitor rate | Config test data |

**File: `crates/agent/src/lead_scoring.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 110 | Signal | `provided_gold_details` | `provided_asset_details` |
| 345 | Slot check | `"gold_weight"`, `"gold_purity"` | Config slot names |

**File: `crates/agent/src/memory/compressor.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 52-57 | Field names | `gold_weight`, `gold_purity` | Config field names |
| 477-482 | Entity mapping | "Gold", "Amount", "Lender" | Config entities |

### 2.5 Persistence Crate

**File: `crates/persistence/src/gold_price.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 24-37 | Struct fields | `price_24k`, `price_22k`, `price_18k` | Config variants |
| 40 | Type alias | `GoldPrice = AssetPrice` | Remove alias |
| 81 | Type alias | `GoldPurity = AssetVariant` | Remove alias |
| 164-165 | Purity factors | `0.916`, `0.75` | Config factors |
| 182, 219, 250, 314 | Table names | `gold_price_latest`, `gold_prices` | `asset_prices` |

**File: `crates/persistence/src/schema.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 79-94 | Table | `gold_prices` | `asset_prices` |
| 104-119 | Table | `gold_price_latest` | `asset_price_latest` |
| 162-164 | Comment | "RBI compliance" | Generic "regulatory" |

**File: `crates/persistence/src/lib.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 43 | Hardcoded price | `7500.0` | Config base price |

### 2.6 LLM Crate

**Status: MOSTLY DOMAIN-AGNOSTIC** ✓

The LLM crate has been well-refactored:
- `prompt.rs`: Uses `system_prompt_from_config()` instead of hardcoded prompts
- `speculative.rs`: Domain terms loaded from config (lines 85-86)
- No hardcoded gold loan/banking terms found

### 2.7 Server Crate

**File: `crates/server/src/main.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 73-78 | Service name | `gold_price_service` | `asset_price_service` |
| 299, 308 | Env var | `DOMAIN_ID=gold_loan` | OK (config value) |

**File: `crates/server/src/http.rs`**
| Line | Issue | Current | Recommendation |
|------|-------|---------|----------------|
| 463 | Response key | `"current_price_per_unit"` | OK (generic) |

---

## 3. Hardcoded Business Logic

### 3.1 Critical Hardcoded Values

| Location | Value | Purpose | Action |
|----------|-------|---------|--------|
| `customer.rs:300` | 100 units | High-value collateral threshold | Move to `segments.yaml` |
| `customer.rs:307` | 500,000 | High-value loan threshold | Move to `segments.yaml` |
| `calculator.rs:476-482` | Tier boundaries | Rate tier amounts (1L, 5L) | Move to `domain.yaml` |
| `gold_price.rs:164-165` | 0.916, 0.75 | Purity multipliers | Move to `domain.yaml` |
| `gold_price.rs:379` | 0.75 | LTV ratio | Move to `domain.yaml` |
| `lib.rs:43` | 7500.0 | Base asset price | Move to `domain.yaml` |
| `agent_config.rs:169` | 18.0 | Competitor rate default | Move to `competitors.yaml` |

### 3.2 Hardcoded Text/Templates

| Location | Content | Action |
|----------|---------|--------|
| `eligibility.rs:147` | "You are eligible for a loan..." | Move to `prompts/` |
| `savings.rs:152` | "By switching to {bank}..." | Move to `prompts/` |
| `price.rs:190-195` | "Current gold price is ₹{}" | Move to `prompts/` |
| `adaptation.rs:323-326` | "RBI regulated bank" | Move to `features.yaml` |
| `adaptation.rs:342` | "9.5% - among the lowest" | Move to `domain.yaml` |

---

## 4. Slot and Intent Names

### 4.1 Current Domain-Specific Slot Names

| Current Name | Generic Alternative | Files Affected |
|--------------|---------------------|----------------|
| `gold_weight` | `asset_quantity` | 8 files |
| `gold_purity` | `asset_quality_tier` | 7 files |
| `loan_amount` | `offer_amount` | 6 files |
| `gold_weight_grams` | `asset_quantity_units` | 3 files |
| `current_lender` | `current_provider` | 5 files |

### 4.2 Domain-Specific Intent Names

| Current Intent | Generic Alternative |
|----------------|---------------------|
| `gold_price_inquiry` | `asset_price_inquiry` |
| `eligibility_check` | `qualification_check` |
| `switch_lender` | `switch_provider` |
| `gold_price` | `asset_price` |
| `closure_inquiry` | `service_closure_inquiry` |

### 4.3 Recommended Slot Configuration

```yaml
# slots.yaml (domain-agnostic naming with display aliases)
slots:
  asset_quantity:
    display_name:
      gold_loan: "Gold Weight"
      car_loan: "Vehicle Value"
      insurance: "Coverage Amount"
    type: number
    unit:
      gold_loan: grams
      car_loan: INR
    extraction_patterns: ${domain.vocabulary.quantity_patterns}

  asset_quality_tier:
    display_name:
      gold_loan: "Gold Purity"
      car_loan: "Vehicle Condition"
    type: enum
    values: ${domain.asset_variants}
```

---

## 5. Competitor References

### 5.1 Hardcoded Competitor Names

| Competitor | Files | Lines | Action |
|------------|-------|-------|--------|
| Muthoot | 15 | ~45 | Move to `competitors.yaml` |
| Manappuram | 8 | ~20 | Move to `competitors.yaml` |
| IIFL | 10 | ~25 | Move to `competitors.yaml` |
| Kotak | 6 | ~12 | Brand config |
| HDFC | 4 | ~8 | Move to `competitors.yaml` |
| SBI | 4 | ~8 | Move to `competitors.yaml` |

### 5.2 Current Config Structure (Good)

```yaml
# competitors.yaml
competitors:
  muthoot:
    display_name: "Muthoot Finance"
    aliases: ["muthoot", "muthut", "muthood"]
    type: nbfc
    typical_rate: 18.0
    weaknesses: [...]
```

### 5.3 Required Changes

1. Remove hardcoded competitor patterns from `intent/mod.rs:654-666`
2. Load competitor regex patterns from config
3. Remove competitor names from `compliance/rules.rs:163-165`

---

## 6. Database Schema Issues

### 6.1 Domain-Specific Table Names

| Current Table | Generic Table | Migration Required |
|---------------|---------------|-------------------|
| `gold_prices` | `asset_prices` | YES |
| `gold_price_latest` | `asset_price_latest` | YES |

### 6.2 Domain-Specific Column Names

| Current Column | Generic Column |
|----------------|----------------|
| `price_24k` | `price_tier_1` or configurable |
| `price_22k` | `price_tier_2` |
| `price_18k` | `price_tier_3` |
| `price_per_gram` | `price_per_unit` |

### 6.3 Recommended Schema Migration

```sql
-- Rename tables
ALTER TABLE gold_prices RENAME TO asset_prices;
ALTER TABLE gold_price_latest RENAME TO asset_price_latest;

-- Rename columns (or use views for backwards compatibility)
-- Option 1: Direct rename
ALTER TABLE asset_prices RENAME COLUMN price_per_gram TO price_per_unit;

-- Option 2: Create views for backwards compatibility
CREATE VIEW gold_prices AS SELECT * FROM asset_prices;
```

---

## 7. Config Structure Analysis

### 7.1 Well-Designed Config Areas ✓

1. **Brand Configuration**: `company_name`, `agent_name`, `helpline` - fully configurable
2. **Interest Rate Tiers**: Defined in `domain.yaml` with boundaries
3. **Competitor Intelligence**: Full competitor profiles in `competitors.yaml`
4. **Conversation Stages**: `stages.yaml` with transitions and requirements
5. **Objection Handling**: `objections.yaml` with patterns and responses
6. **Vocabulary/Phonetic Corrections**: Comprehensive in `vocabulary.yaml`

### 7.2 Config Gaps (Need Addition)

1. **Tool Response Templates**: Add to `tools/responses.yaml`
2. **Slot Name Aliases**: Add `aliases` field to `slots.yaml`
3. **Intent-to-Tool Mappings**: Add `intent_tool_mappings.yaml`
4. **Entity Type Mappings**: Add to `entities.yaml`
5. **Compliance Rules**: Add `compliance.yaml` with rate limits, forbidden phrases
6. **Database Schema Config**: Add `schema.yaml` for table/column naming

### 7.3 Recommended New Config Files

```yaml
# config/domains/{domain}/intent_tool_mappings.yaml
mappings:
  eligibility_check: check_eligibility
  price_inquiry: get_price
  switch_provider: calculate_savings
  schedule_visit: find_locations

# config/domains/{domain}/tools/responses.yaml
templates:
  eligibility_success: "You are eligible for {product_name} up to ₹{max_amount} at {rate}%!"
  savings_comparison: "By switching to {company_name}, you can save ₹{monthly_savings}/month!"
  price_info: "Current {variant_name} price is ₹{price} per {unit}."

# config/domains/{domain}/entities.yaml
entity_types:
  asset_quantity:
    display_name: "Gold Weight"
    category: "Asset"
  asset_quality:
    display_name: "Gold Purity"
    category: "Asset"
  offer_amount:
    display_name: "Loan Amount"
    category: "Financial"

# config/domains/{domain}/compliance.yaml
rules:
  rate_bounds:
    min: 7.0
    max: 24.0
  forbidden_phrases:
    - "guaranteed approval"
    - "zero interest"
  competitor_comparison:
    allowed: true
    disparagement_forbidden: true
```

---

## 8. Feature Flags Implementation

### 8.1 Current Feature Flags (ML/Infrastructure)

| Feature | Purpose | Files |
|---------|---------|-------|
| `onnx` | ONNX Runtime inference | 12 files |
| `candle` | Native Rust ML | 18 files |
| `noise-suppression` | Audio preprocessing | 2 files |
| `webrtc` | WebRTC transport | 3 files |
| `telemetry` | OpenTelemetry | 1 file |

### 8.2 Missing: Domain Feature Flags

Currently, there are NO domain-specific feature flags. Recommended additions:

```toml
# Cargo.toml
[features]
default = []
# Domain features
gold_loan = []
car_loan = []
insurance = []
generic = []

# Regulatory features
rbi_compliance = []
irdai_compliance = []
sebi_compliance = []
```

### 8.3 Runtime Feature Flags (Config-Based)

```yaml
# domain.yaml
features:
  enable_competitor_comparison: true
  enable_savings_calculator: true
  enable_asset_price_lookup: true
  enable_doorstep_service: true
  regulatory_compliance: "rbi"  # rbi, irdai, sebi, none
```

---

## 9. Recommendations

### 9.1 Immediate Actions (P0)

1. **Create `intent_tool_mappings.yaml`** - Remove hardcoded mappings from `agent/tools.rs`
2. **Create `tools/responses.yaml`** - Remove hardcoded response templates
3. **Add slot aliases to `slots.yaml`** - Support `gold_weight` → `asset_quantity` mapping
4. **Move thresholds to `segments.yaml`** - High-value customer detection

### 9.2 Short-Term Actions (P1)

1. **Rename database tables** - `gold_prices` → `asset_prices` (with migration script)
2. **Refactor persistence crate** - Use generic `AssetPrice` throughout
3. **Load competitor patterns from config** - Remove from `intent/mod.rs`
4. **Create compliance config** - Move rate limits from `rules.rs`

### 9.3 Medium-Term Actions (P2)

1. **Generic slot naming throughout** - Replace all `gold_weight` with `asset_quantity`
2. **Domain feature flags** - Conditional compilation for domain-specific code
3. **Test parameterization** - Remove domain-specific test data
4. **Documentation update** - Remove gold loan references from README

### 9.4 Architecture Patterns

#### Pattern 1: Config-Driven Tool Factory
```rust
// Instead of:
match intent {
    "eligibility_check" => "check_eligibility",
    "gold_price" => "get_price",
}

// Use:
let tool = config.intent_tool_mappings.get(intent)?;
```

#### Pattern 2: Template-Based Responses
```rust
// Instead of:
format!("You are eligible for a loan up to ₹{}", amount)

// Use:
config.templates.render("eligibility_success", &context)
```

#### Pattern 3: Config-Driven Slot Names
```rust
// Instead of:
if slot_name == "gold_weight" || slot_name == "gold_purity" { ... }

// Use:
if config.asset_slots.contains(&slot_name) { ... }
```

---

## 10. Action Items

### 10.1 Config File Changes

| File | Action | Priority |
|------|--------|----------|
| `intent_tool_mappings.yaml` | CREATE | P0 |
| `tools/responses.yaml` | CREATE | P0 |
| `slots.yaml` | ADD aliases field | P0 |
| `entities.yaml` | CREATE | P1 |
| `compliance.yaml` | CREATE | P1 |
| `segments.yaml` | ADD thresholds | P0 |

### 10.2 Code Changes

| File | Change | Priority |
|------|--------|----------|
| `agent/tools.rs:38-82` | Load from config | P0 |
| `agent_config.rs:167-171` | Load from config | P0 |
| `eligibility.rs:147` | Use template | P0 |
| `savings.rs:152` | Use template | P0 |
| `price.rs:190-195` | Use template | P0 |
| `gold_price.rs` | Rename to `asset_price.rs` | P1 |
| `intent/mod.rs:654-666` | Load patterns from config | P1 |
| `customer.rs:300,307` | Load thresholds from config | P0 |
| `lead_scoring.rs:110,345` | Use config slot names | P1 |
| `compressor.rs:52-57` | Use config field names | P1 |

### 10.3 Database Migration

```sql
-- Migration: Rename gold-specific tables
-- File: migrations/V003__rename_asset_tables.sql

-- 1. Rename tables
ALTER TABLE IF EXISTS gold_prices RENAME TO asset_prices;
ALTER TABLE IF EXISTS gold_price_latest RENAME TO asset_price_latest;

-- 2. Create backwards-compatible views
CREATE OR REPLACE VIEW gold_prices AS SELECT * FROM asset_prices;
CREATE OR REPLACE VIEW gold_price_latest AS SELECT * FROM asset_price_latest;

-- 3. Update column comments
COMMENT ON COLUMN asset_prices.price_per_gram IS 'Price per unit (grams for gold, units for other assets)';
```

### 10.4 Test Updates

| Test File | Change |
|-----------|--------|
| `customer.rs:757-811` | Parameterize with config |
| `intent/mod.rs:1249-1269` | Use config test data |
| `slot_extraction/mod.rs:985-1310` | Use config competitors |
| `bench/*.rs` | Remove hardcoded "Muthoot" |

---

## Appendix: File Reference

### Files Requiring Changes (by Priority)

#### P0 - Critical (12 files)
1. `/home/vscode/goldloan-study/voice-agent/backend/crates/agent/src/agent/tools.rs`
2. `/home/vscode/goldloan-study/voice-agent/backend/crates/agent/src/agent_config.rs`
3. `/home/vscode/goldloan-study/voice-agent/backend/crates/tools/src/domain_tools/tools/eligibility.rs`
4. `/home/vscode/goldloan-study/voice-agent/backend/crates/tools/src/domain_tools/tools/savings.rs`
5. `/home/vscode/goldloan-study/voice-agent/backend/crates/tools/src/domain_tools/tools/price.rs`
6. `/home/vscode/goldloan-study/voice-agent/backend/crates/core/src/customer.rs`
7. `/home/vscode/goldloan-study/voice-agent/backend/crates/core/src/traits/calculator.rs`
8. `/home/vscode/goldloan-study/voice-agent/backend/crates/persistence/src/lib.rs`
9. `/home/vscode/goldloan-study/voice-agent/backend/crates/persistence/src/gold_price.rs`
10. `/home/vscode/goldloan-study/voice-agent/backend/crates/agent/src/stage.rs`
11. `/home/vscode/goldloan-study/voice-agent/backend/crates/agent/src/dst/dynamic.rs`
12. `/home/vscode/goldloan-study/voice-agent/backend/crates/agent/src/persuasion.rs`

#### P1 - Important (15 files)
1. `/home/vscode/goldloan-study/voice-agent/backend/crates/text_processing/src/intent/mod.rs`
2. `/home/vscode/goldloan-study/voice-agent/backend/crates/text_processing/src/entities/mod.rs`
3. `/home/vscode/goldloan-study/voice-agent/backend/crates/text_processing/src/slot_extraction/mod.rs`
4. `/home/vscode/goldloan-study/voice-agent/backend/crates/text_processing/src/sentiment/mod.rs`
5. `/home/vscode/goldloan-study/voice-agent/backend/crates/text_processing/src/compliance/rules.rs`
6. `/home/vscode/goldloan-study/voice-agent/backend/crates/core/src/personalization/adaptation.rs`
7. `/home/vscode/goldloan-study/voice-agent/backend/crates/core/src/traits/competitors.rs`
8. `/home/vscode/goldloan-study/voice-agent/backend/crates/tools/src/registry.rs`
9. `/home/vscode/goldloan-study/voice-agent/backend/crates/agent/src/lead_scoring.rs`
10. `/home/vscode/goldloan-study/voice-agent/backend/crates/agent/src/memory/compressor.rs`
11. `/home/vscode/goldloan-study/voice-agent/backend/crates/persistence/src/schema.rs`
12. `/home/vscode/goldloan-study/voice-agent/backend/crates/persistence/src/sms.rs`
13. `/home/vscode/goldloan-study/voice-agent/backend/crates/server/src/main.rs`
14. `/home/vscode/goldloan-study/voice-agent/backend/crates/server/src/state.rs`
15. `/home/vscode/goldloan-study/voice-agent/backend/crates/server/src/mcp_server.rs`

#### P2 - Low Priority (20+ files)
- Test files with domain-specific examples
- README and documentation
- Benchmark files

### Config Files (Current)
- `/home/vscode/goldloan-study/voice-agent/backend/config/default.yaml`
- `/home/vscode/goldloan-study/voice-agent/backend/config/domains/gold_loan/domain.yaml`
- `/home/vscode/goldloan-study/voice-agent/backend/config/domains/gold_loan/intents.yaml`
- `/home/vscode/goldloan-study/voice-agent/backend/config/domains/gold_loan/slots.yaml`
- `/home/vscode/goldloan-study/voice-agent/backend/config/domains/gold_loan/stages.yaml`
- `/home/vscode/goldloan-study/voice-agent/backend/config/domains/gold_loan/objections.yaml`
- `/home/vscode/goldloan-study/voice-agent/backend/config/domains/gold_loan/competitors.yaml`
- `/home/vscode/goldloan-study/voice-agent/backend/config/domains/gold_loan/segments.yaml`
- `/home/vscode/goldloan-study/voice-agent/backend/config/domains/gold_loan/scoring.yaml`
- `/home/vscode/goldloan-study/voice-agent/backend/config/domains/gold_loan/features.yaml`
- `/home/vscode/goldloan-study/voice-agent/backend/config/domains/gold_loan/tools/schemas.yaml`

---

## Conclusion

The voice-agent backend has a **solid foundation** for domain-agnostic architecture with the config-driven view pattern. However, approximately **47 files** still contain hardcoded domain-specific code that needs refactoring.

### Summary of Work Required

| Category | Files | Effort |
|----------|-------|--------|
| New Config Files | 4 | LOW |
| Code Refactoring (P0) | 12 | MEDIUM |
| Code Refactoring (P1) | 15 | HIGH |
| Database Migration | 1 | MEDIUM |
| Test Updates | 20+ | LOW |

### Target State

After completing all action items:
1. **Zero hardcoded domain terms** in production code
2. **100% config-driven** business logic
3. **New domain onboarding** via `config/domains/{new_domain}/` directory only
4. **Backwards compatibility** via type aliases and database views

---

*Document generated: 2026-01-09*
*Analysis performed by: Claude Code Agent*
