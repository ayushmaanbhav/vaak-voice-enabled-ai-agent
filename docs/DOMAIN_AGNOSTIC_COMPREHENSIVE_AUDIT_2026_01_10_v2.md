# Domain-Agnostic Architecture Comprehensive Audit

**Date:** 2026-01-10
**Version:** 2.0
**Status:** Deep Analysis Complete

---

## Executive Summary

This document provides a comprehensive audit of the voice agent backend codebase for domain-agnostic architecture compliance. The goal is to ensure the system can onboard new business domains **purely through YAML configuration** without code changes.

### Current State Assessment
- **Overall Domain-Agnostic Score:** 75-80%
- **Trait System Maturity:** 8.5/10
- **Config Coverage:** ~75%
- **Files with Hardcoded Domain Logic:** 80+
- **Critical Issues:** 15
- **High Priority Issues:** 25

---

## Part 1: Hardcoded Domain-Specific Terms

### 1.1 CRITICAL - Business Logic Hardcoding

#### Gold Purity/Quality Factors
| File | Lines | Hardcoded Values | Impact |
|------|-------|------------------|--------|
| `core/src/traits/calculator.rs` | 422-425 | K24=1.0, K22=0.916, K18=0.75, K14=0.585 | Breaks non-gold domains |
| `text_processing/src/slot_extraction/mod.rs` | 189-194 | PURITY_24K, PURITY_22K, PURITY_18K, PURITY_14K regex | Gold-specific patterns |
| `text_processing/src/intent/mod.rs` | 684-695 | "18K", "22K", "24K" enum | Limited to gold karats |
| `text_processing/src/entities/mod.rs` | 404-414 | Validation range 10-24 | Gold karat range |

#### Interest Rate Thresholds
| File | Lines | Hardcoded Values | Impact |
|------|-------|------------------|--------|
| `core/src/traits/calculator.rs` | 411, 475-478 | 500_000.0 (5 lakh threshold) | Gold loan specific tiers |
| `core/src/customer.rs` | 351, 381, 397, 459 | 500_000.0 high-value threshold | Fixed amount regardless of domain |

#### Competitor Default Rates
| File | Lines | Hardcoded Values | Impact |
|------|-------|------------------|--------|
| `core/src/traits/competitors.rs` | 286-287 | NBFC=18.0%, informal=24.0% | Gold loan market rates |
| `core/src/traits/competitors.rs` | 300, 351 | default_unknown_rate=18.0 | Assumes gold loan market |

### 1.2 HIGH - Specific Competitor Names

| File | Lines | Hardcoded Names |
|------|-------|-----------------|
| `text_processing/src/compliance/checker.rs` | 273-275 | "Muthoot", "Manappuram", "Kotak" |
| `text_processing/src/intent/mod.rs` | 1215-1222 | "muthoot", "manappuram", "iifl" patterns |
| `config/src/domain/vocabulary.rs` | 195-211 | "KMBL", "KMB", "MFL" abbreviations |
| `text_processing/src/grammar/llm_corrector.rs` | 62 | "kotuk" -> "Kotak" correction |

### 1.3 MEDIUM - Location & Currency

#### City Patterns (50+ Indian cities hardcoded)
| File | Lines | Count |
|------|-------|-------|
| `text_processing/src/slot_extraction/mod.rs` | 147-150 | 50+ cities |
| `text_processing/src/slot_extraction/mod.rs` | 737-763 | Duplicate list |
| `text_processing/src/intent/mod.rs` | 700-711 | 9 major cities |

#### Currency Units
| File | Lines | Hardcoded Values |
|------|-------|------------------|
| `text_processing/src/slot_extraction/mod.rs` | 66-72 | crore, lakh, thousand, ₹, rs |
| `text_processing/src/entities/mod.rs` | 175-178 | INR, rupees, lakh, crore |
| `text_processing/src/entities/mod.rs` | 180-183 | Hindi: करोड़, लाख, हज़ार |

#### Hindi Numbers (Duplicated)
| File | Lines | Issue |
|------|-------|-------|
| `text_processing/src/entities/mod.rs` | 431-454 | 18+ Hindi numbers in match |
| `text_processing/src/intent/mod.rs` | 1032-1058 | EXACT DUPLICATE |

### 1.4 LOW - Weight Units & Loan Purposes

#### Weight Patterns
| File | Lines | Hardcoded Values |
|------|-------|------------------|
| `text_processing/src/slot_extraction/mod.rs` | 76-81 | grams, tola, ग्राम, तोला |
| `text_processing/src/intent/mod.rs` | 641-654 | 11.66 tola conversion factor |
| `text_processing/src/entities/mod.rs` | 351-354 | 11660 milligrams per tola |

#### Loan Purpose Keywords
| File | Lines | Hardcoded Purposes |
|------|-------|-------------------|
| `text_processing/src/slot_extraction/mod.rs` | 125-135 | business, medical, education, wedding, etc. |
| `text_processing/src/slot_extraction/mod.rs` | 694-730 | Duplicate purpose extraction |

---

## Part 2: Config Structure & Wiring Analysis

### 2.1 Current Config Files (23 total)

| Config File | Status | Coverage |
|------------|--------|----------|
| `domain.yaml` | GOOD | Brand, rates, limits, competitors |
| `slots.yaml` | GOOD | Slot definitions, aliases, parsing |
| `stages.yaml` | PARTIAL | Missing context_budget, rag_fraction |
| `features.yaml` | GOOD | Feature priorities per segment |
| `competitors.yaml` | GOOD | All competitors with rates |
| `compliance.yaml` | GOOD | Rate rules, forbidden phrases |
| `objections.yaml` | GOOD | Objection patterns and responses |
| `segments.yaml` | GOOD | Customer segment definitions |
| `lead_scoring.yaml` | PARTIAL | Missing scoring_weights section |
| `tools/schemas.yaml` | GOOD | Tool definitions |
| `adaptation.yaml` | GOOD | Variable substitution |
| `extraction_patterns.yaml` | EXISTS | NOT WIRED to code |
| `vocabulary.yaml` | EXISTS | PARTIALLY WIRED |
| `entity_types.yaml` | EXISTS | NOT FULLY WIRED |
| `signals.yaml` | EXISTS | NOT FULLY WIRED |

### 2.2 Config Gaps Identified

#### Missing Config Sections
```yaml
# lead_scoring.yaml - MISSING
scoring_weights:
  urgency: { base_signal: 10, per_keyword: 5 }
  engagement: { per_turn: 3, per_question: 2 }
  information: { contact_info: 8, asset_details: 8 }
  intent: { proceed: 15, callback: 5, visit: 8 }
  penalties: { disinterest: -15, competitor_pref: -10 }

urgency_keywords:
  immediate: { en: [...], hi: [...] }
  short_term: { en: [...], hi: [...] }

slot_signal_mappings:
  contact_info: { slots: [...], signal: "provided_contact_info" }
```

```yaml
# stages.yaml - MISSING FIELDS
stages:
  greeting:
    context_budget_tokens: 1024  # MISSING
    rag_context_fraction: 0.0   # MISSING
    history_turns_to_keep: 0    # MISSING
    valid_transitions: [...]     # MISSING
```

```yaml
# domain.yaml - MISSING
currency:
  field_suffix: "inr"  # For JSON output naming

high_value_thresholds:
  amount: 500000
  weight: 100
```

### 2.3 Wiring Issues

| Component | Config Exists | Wired | Issue |
|-----------|--------------|-------|-------|
| Purity patterns | extraction_patterns.yaml | NO | Still uses hardcoded regex |
| City patterns | extraction_patterns.yaml | NO | 50+ cities hardcoded |
| Purpose patterns | extraction_patterns.yaml | NO | Uses static patterns |
| Hindi numbers | vocabulary.yaml | NO | Match statement instead |
| Lead scoring weights | lead_scoring.yaml | NO | All weights hardcoded |
| Stage budgets | stages.yaml | NO | Match statement |
| Currency suffix | domain.yaml | NO | _inr hardcoded |

---

## Part 3: Trait System Analysis

### 3.1 Trait Inventory (22 traits)

#### Mature Traits (Config-Driven)
| Trait | File | Status |
|-------|------|--------|
| DomainCalculator | traits/calculator.rs | MATURE |
| CompetitorAnalyzer | traits/competitors.rs | MATURE |
| FeatureProvider | traits/feature_provider.rs | MATURE |
| ObjectionProvider | traits/objection_provider.rs | MATURE |
| LeadScoringStrategy | traits/scoring.rs | MATURE |
| LeadClassifier | traits/lead_classifier.rs | MATURE |
| ToolArgumentProvider | traits/tool_arguments.rs | MATURE |
| SignalProvider | traits/signals.rs | MATURE |
| EntityTypeProvider | traits/entity_types.rs | MATURE |
| ToolFactory | traits/tool_factory.rs | MATURE |

#### Needs Review
| Trait | File | Issue |
|-------|------|-------|
| ConversationFSM | - | No stage config integration |
| ComplianceChecker | compliance/checker.rs | Partial hardcoding |
| Translator/GrammarCorrector | - | Domain patterns |

### 3.2 Missing Traits

| Proposed Trait | Purpose | Priority |
|---------------|---------|----------|
| ExtractionPatternProvider | Config-driven slot extraction | HIGH |
| ConversationFlowProvider | Config-driven stage management | HIGH |
| ValidationProvider | Config-driven validation rules | MEDIUM |
| ResponseBuilder | Config-driven response templates | MEDIUM |

### 3.3 Architecture Score: 8.5/10

| Dimension | Score | Notes |
|-----------|-------|-------|
| Trait Genericity | 9/10 | Excellent abstraction |
| Config-Driven Design | 7/10 | Gaps in wiring |
| Separation of Concerns | 8/10 | Some god objects |
| Extensibility | 8/10 | Needs flow abstraction |
| Test Coverage | 9/10 | Good mock implementations |
| Documentation | 9/10 | P13/P20/P23 markers helpful |

---

## Part 4: Tool Implementation Analysis

### 4.1 Output Field Naming (Currency Coupling)

| File | Lines | Hardcoded Fields |
|------|-------|------------------|
| `tools/savings.rs` | 171-178 | current_emi_inr, our_emi_inr, monthly_emi_savings_inr |
| `tools/eligibility.rs` | 182-186 | collateral_value_inr, max_loan_amount_inr |
| `tools/price.rs` | 175, 198 | price_per_gram_inr, estimated_values_inr |

### 4.2 Currency Symbol Hardcoding

| File | Lines | Issue |
|------|-------|-------|
| `tools/price.rs` | 224, 233, 286 | Hardcoded ₹ |
| `tools/eligibility.rs` | 150, 155, 166, 168 | Hardcoded ₹ |
| `tools/competitor.rs` | 177, 204 | Hardcoded ₹ |

### 4.3 Tool Status Summary

| Tool | Config-Driven | Issues |
|------|--------------|--------|
| SavingsCalculator | 85% | Field naming, currency |
| EligibilityCheck | 85% | Field naming, currency |
| GetPrice | 80% | Field naming, currency |
| CompetitorComparison | 90% | Currency symbol |
| BranchLocator | 95% | Minimal issues |

---

## Part 5: Agent Processing Analysis

### 5.1 Lead Scoring (ALL weights hardcoded)

| Category | File:Lines | Hardcoded Values |
|----------|-----------|------------------|
| Urgency | lead_scoring.rs:835-842 | 10, 5, 25 |
| Engagement | lead_scoring.rs:849-854 | 3, 2, 3, 3, 25 |
| Information | lead_scoring.rs:860-873 | 8, 8, 5, 4, 25 |
| Intent | lead_scoring.rs:876-889 | 15, 5, 8, 25 |
| Penalties | lead_scoring.rs:893-908 | -15, -10, -5, -3 |
| Multipliers | lead_scoring.rs:1064-1077 | 0.5, 0.8, 1.2, 0.3 |

### 5.2 Stage Configuration (ALL match statements)

| Property | File:Lines | Values |
|----------|-----------|--------|
| Context Budget | stage.rs:147-156 | 1024-3584 tokens |
| RAG Fraction | stage.rs:163-173 | 0.0-0.4 |
| History Turns | stage.rs:178-188 | 0-6 turns |
| Valid Transitions | stage.rs:254-286 | Per-stage arrays |
| Suggested Questions | stage.rs:217-250 | Per-stage arrays |

### 5.3 Slot Mappings (Hardcoded aliases)

| File | Lines | Mappings |
|------|-------|----------|
| processing.rs | 113-123 | gold_weight->asset_quantity, gold_purity->asset_quality |
| tools.rs | 198-225 | asset_quantity->collateral_weight, asset_quality->collateral_variant |
| stage.rs | 582-588 | Fallback aliases |

---

## Part 6: Remediation Plan

### Sprint 1: Critical Business Logic
1. Wire quality tier factors to extraction_patterns.yaml
2. Wire interest rate thresholds to domain.yaml
3. Wire competitor default rates to competitors.yaml
4. Wire ALL lead scoring weights to lead_scoring.yaml
5. Wire stage configuration to stages.yaml

### Sprint 2: Text Processing Wiring
1. Create ExtractionPatternProvider trait
2. Wire slot_extraction patterns to config
3. Consolidate Hindi number handling
4. Wire competitor patterns to config

### Sprint 3: Agent Processing Wiring
1. Wire urgency keywords to config
2. Wire slot-to-signal mappings to config
3. Wire currency field naming to config

### Sprint 4: Cleanup & New Traits
1. Remove duplicate code
2. Replace hardcoded currency symbols
3. Create new traits
4. Move fallback responses to config

---

## Part 7: New Config Sections Required

### lead_scoring.yaml Extensions
```yaml
scoring_weights:
  urgency:
    base_signal: 10
    per_keyword: 5
    max_keywords: 3
    max_score: 25
  engagement:
    per_turn: 3
    per_question: 2
    rates_inquiry: 3
    comparison_request: 3
    max_score: 25
  information:
    contact_info: 8
    asset_details: 8
    loan_amount: 5
    specific_requirements: 4
    max_score: 25
  intent:
    proceed_intent: 15
    callback_request: 5
    branch_visit: 8
    max_score: 25
  penalties:
    disinterest: -15
    competitor_preference: -10
    human_request: -5
    per_unresolved_objection: -3

conversion_multipliers:
  unqualified: 0.5
  mql: 0.8
  sql: 1.2
  intent_proceed: 1.2
  intent_disinterest: 0.3

urgency_keywords:
  immediate:
    en: ["urgent", "urgently", "immediately", "today", "now", "asap", "emergency"]
    hi: ["jaldi", "abhi", "turant", "aaj", "foran"]
  short_term:
    en: ["this week", "few days", "soon", "quickly"]
    hi: ["is hafte", "jald", "thode din"]

slot_signal_mappings:
  contact_info:
    slots: ["phone_number", "customer_name", "phone", "email"]
    signal: "provided_contact_info"
  asset_details:
    slots: ["asset_quantity", "asset_quality", "collateral_weight", "collateral_variant"]
    signal: "provided_asset_details"
  loan_amount:
    slots: ["offer_amount", "requested_amount"]
    signals: ["provided_loan_amount", "has_specific_requirements"]
```

### stages.yaml Extensions
```yaml
stages:
  greeting:
    context_budget_tokens: 1024
    rag_context_fraction: 0.0
    history_turns_to_keep: 0
    valid_transitions: ["discovery", "farewell"]
    suggested_questions:
      en: ["How are you doing today?", "Is this a good time to talk?"]
      hi: ["Aap kaise hain?", "Kya yeh sahi samay hai baat karne ka?"]
  discovery:
    context_budget_tokens: 2048
    rag_context_fraction: 0.15
    history_turns_to_keep: 3
    valid_transitions: ["qualification", "presentation", "objection_handling", "farewell"]
  # ... all stages
```

### domain.yaml Extensions
```yaml
currency:
  code: "INR"
  symbol: "₹"
  field_suffix: "inr"
  name: "Indian Rupees"

high_value_thresholds:
  amount: 500000
  weight: 100

competitors:
  defaults:
    unknown_rate: 18.0
    by_type:
      nbfc: 18.0
      informal: 24.0
      bank: 12.0
      cooperative: 15.0
```

---

## Part 8: Files to Modify (Complete List)

### Critical Priority
- `crates/core/src/traits/calculator.rs`
- `crates/core/src/customer.rs`
- `crates/core/src/traits/competitors.rs`
- `crates/agent/src/lead_scoring.rs`
- `crates/agent/src/stage.rs`

### High Priority
- `crates/text_processing/src/slot_extraction/mod.rs`
- `crates/text_processing/src/intent/mod.rs`
- `crates/text_processing/src/entities/mod.rs`
- `crates/agent/src/dst/slots.rs`
- `crates/tools/src/domain_tools/tools/savings.rs`
- `crates/tools/src/domain_tools/tools/eligibility.rs`
- `crates/tools/src/domain_tools/tools/price.rs`

### Medium Priority
- `crates/agent/src/agent/response.rs`
- `crates/tools/src/domain_tools/tools/competitor.rs`
- `crates/text_processing/src/compliance/checker.rs`
- `crates/agent/src/memory/compressor.rs`

### New Files to Create
- `crates/core/src/traits/extraction.rs`
- `crates/core/src/traits/conversation_flow.rs`
- `crates/core/src/traits/validation.rs`
- `config/domains/gold_loan/memory.yaml`

---

## Part 9: Success Criteria

1. **Zero hardcoded domain terms** in business logic
2. **All patterns configurable** via YAML
3. **New domain onboarding** requires only YAML files
4. **Existing tests pass** with no behavioral changes
5. **Architecture score** improves from 8.5/10 to 9.5/10

---

## Appendix A: Detailed File-by-File Findings

### text_processing/src/slot_extraction/mod.rs

| Lines | Issue | Severity | Config |
|-------|-------|----------|--------|
| 66-72 | Amount multipliers | MEDIUM | extraction_patterns.yaml |
| 76-81 | Weight patterns | MEDIUM | extraction_patterns.yaml |
| 125-135 | Purpose patterns | HIGH | extraction_patterns.yaml |
| 138-143 | Repayment types | HIGH | extraction_patterns.yaml |
| 145-150 | City patterns | HIGH | extraction_patterns.yaml |
| 189-194 | Purity patterns | CRITICAL | extraction_patterns.yaml |
| 544 | Tola conversion 11.66 | MEDIUM | extraction_patterns.yaml |
| 694-730 | Duplicate purpose | MEDIUM | Remove duplicate |
| 737-763 | Duplicate cities | MEDIUM | Remove duplicate |

### text_processing/src/intent/mod.rs

| Lines | Issue | Severity | Config |
|-------|-------|----------|--------|
| 641-654 | Weight patterns | MEDIUM | extraction_patterns.yaml |
| 684-695 | Purity enum | CRITICAL | extraction_patterns.yaml |
| 700-711 | City patterns | HIGH | extraction_patterns.yaml |
| 1032-1058 | Hindi numbers (dup) | MEDIUM | vocabulary.yaml |
| 1214-1225 | Competitor patterns | HIGH | competitors.yaml |

### agent/src/lead_scoring.rs

| Lines | Issue | Severity | Config |
|-------|-------|----------|--------|
| 579-616 | Intent signal mapping | HIGH | lead_scoring.yaml |
| 618-638 | Slot signal mapping | HIGH | lead_scoring.yaml |
| 707-720 | Urgency keywords | HIGH | lead_scoring.yaml |
| 835-842 | Urgency weights | CRITICAL | lead_scoring.yaml |
| 849-854 | Engagement weights | CRITICAL | lead_scoring.yaml |
| 860-873 | Information weights | CRITICAL | lead_scoring.yaml |
| 876-889 | Intent weights | CRITICAL | lead_scoring.yaml |
| 893-908 | Penalty values | CRITICAL | lead_scoring.yaml |
| 1064-1077 | Conversion multipliers | HIGH | lead_scoring.yaml |

### agent/src/stage.rs

| Lines | Issue | Severity | Config |
|-------|-------|----------|--------|
| 147-156 | Context budget | HIGH | stages.yaml |
| 163-173 | RAG fraction | HIGH | stages.yaml |
| 178-188 | History turns | HIGH | stages.yaml |
| 217-250 | Suggested questions | MEDIUM | stages.yaml |
| 254-286 | Valid transitions | HIGH | stages.yaml |
| 582-588 | Fallback aliases | MEDIUM | slots.yaml |

---

## Appendix B: Config Loading Flow

```
Config Files (YAML)
    ├── domain.yaml
    ├── slots.yaml
    ├── stages.yaml
    ├── features.yaml
    ├── competitors.yaml
    ├── lead_scoring.yaml
    ├── extraction_patterns.yaml  ← NOT WIRED
    ├── vocabulary.yaml           ← PARTIALLY WIRED
    └── ...
        ↓
MasterDomainConfig::load() (config/src/master.rs)
        ↓
AgentDomainView / ToolsDomainView
        ↓
Trait Implementations (ConfigDriven*)
        ↓
Agent / Tools / TextProcessing
        ↓
Runtime Behavior
```

**Issue:** extraction_patterns.yaml and vocabulary.yaml exist but are not fully wired to text_processing crate.

---

*Document generated from deep analysis of voice-agent backend codebase*
