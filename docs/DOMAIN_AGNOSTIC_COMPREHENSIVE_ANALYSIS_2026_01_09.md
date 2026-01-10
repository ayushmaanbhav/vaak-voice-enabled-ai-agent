# Domain-Agnostic Voice Agent: Comprehensive Architecture Analysis

**Date:** 2026-01-09
**Version:** 1.0
**Objective:** Ensure the voice agent is truly domain-agnostic, config-driven, and ready for multi-domain onboarding via YAML configuration.

---

## Executive Summary

The voice-agent/backend codebase has undergone significant refactoring (P13, P16, P18 fixes) toward domain-agnosticism. The architecture demonstrates **excellent design patterns** with comprehensive abstraction layers. However, **600+ hardcoded gold loan references** remain distributed across config files, source code, and knowledge bases.

### Overall Assessment: **8.5/10**

| Aspect | Score | Status |
|--------|-------|--------|
| Config Architecture | 95% | Excellent |
| Trait Abstractions | 95% | Excellent |
| Domain Isolation | 85% | Good (some leakage) |
| Config Wiring | 80% | Mostly complete |
| Intent Patterns | 60% | Needs work |
| Memory Integration | 70% | Partially wired |

### Critical Constraint Reminder
> **Cannot change content/strings/text/context** - only add placeholders or move to config.

---

## Table of Contents

1. [Current Architecture Overview](#1-current-architecture-overview)
2. [Hardcoded Domain Content Inventory](#2-hardcoded-domain-content-inventory)
3. [Trait Structure & Abstraction Analysis](#3-trait-structure--abstraction-analysis)
4. [Config Wiring Status](#4-config-wiring-status)
5. [Business Logic Coupling Analysis](#5-business-logic-coupling-analysis)
6. [Prompts & Templates Analysis](#6-prompts--templates-analysis)
7. [Priority Recommendations](#7-priority-recommendations)
8. [Implementation Roadmap](#8-implementation-roadmap)
9. [File Reference Index](#9-file-reference-index)

---

## 1. Current Architecture Overview

### 1.1 Config Directory Structure

```
voice-agent/backend/
├── config/
│   └── domains/
│       └── gold_loan/                    # Domain-specific configs
│           ├── domain.yaml               # Core business constants & branding
│           ├── slots.yaml                # Dialogue state slots & extraction
│           ├── intents.yaml              # Intent definitions
│           ├── entities.yaml             # Entity extraction patterns
│           ├── stages.yaml               # Conversation stages
│           ├── goals.yaml                # Conversation goals
│           ├── scoring.yaml              # Lead scoring rules
│           ├── compliance.yaml           # Regulatory rules
│           ├── vocabulary.yaml           # ASR corrections
│           ├── prompts/system.yaml       # System prompts
│           ├── intent_tool_mappings.yaml # Intent→Tool routes
│           ├── competitors.yaml          # Competitor data
│           ├── features.yaml             # Product features
│           ├── objections.yaml           # Objection handling
│           ├── segments.yaml             # Customer segments
│           ├── lead_scoring.yaml         # Lead scoring config
│           └── tools/                    # Tool-specific configs
│               ├── schemas.yaml
│               ├── responses.yaml
│               ├── documents.yaml
│               ├── branches.yaml
│               └── sms_templates.yaml
```

### 1.2 Config Loading Hierarchy

```
Priority Order:
1. Environment Variables (VOICE_AGENT_ prefix)
2. Domain YAML files (config/domains/{domain_id}/*)
3. Base defaults (config/base/defaults.yaml) [NOT YET IMPLEMENTED]
4. Code defaults
```

### 1.3 Crate Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  CONFIG CRATE (domain/master.rs)                             │
│  - MasterDomainConfig: Unified loader                        │
│  - Merges all YAML files for a domain                        │
│  - Validates configuration                                   │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│  CRATE-SPECIFIC VIEWS (domain/views.rs)                      │
│                                                              │
│  AgentDomainView ──┐                                         │
│  LlmDomainView   ──├─► Each crate gets only what it needs    │
│  ToolsDomainView ──┘                                         │
└────────────────────┬────────────────────────────────────────┘
                     │
      ┌──────────────┼──────────────┬────────────────────┐
      ▼              ▼              ▼                    ▼
  ┌────────┐  ┌──────────┐  ┌──────────────┐  ┌──────────────┐
  │ AGENT  │  │   LLM    │  │    TOOLS     │  │     RAG      │
  │ CRATE  │  │  CRATE   │  │    CRATE     │  │    CRATE     │
  └────────┘  └──────────┘  └──────────────┘  └──────────────┘
```

---

## 2. Hardcoded Domain Content Inventory

### 2.1 Severity Classification

| Severity | Description | Count | Action |
|----------|-------------|-------|--------|
| **P0** | Blocks domain reuse entirely | ~165 | Must fix |
| **P1** | Significant coupling | ~85 | High priority |
| **P2** | Moderate coupling | ~150 | Medium priority |
| **P3** | Low impact / appropriate | ~200+ | Acceptable |

### 2.2 P0 - Critical Hardcoding (Must Fix)

#### 2.2.1 LLM Prompts - Hardcoded Domain Terms

| File | Line | Hardcoded Content | Issue |
|------|------|-------------------|-------|
| `crates/llm/src/prompt.rs` | 320 | `"gold loan services"` | Should be `{product_name} services` |
| `crates/agent/src/memory/core.rs` | 681 | Test: `contains("kotak")` | Indicates prior hardcoding |

#### 2.2.2 Intent Detection - Hardcoded Patterns

| File | Line | Hardcoded Content | Issue |
|------|------|-------------------|-------|
| `crates/text_processing/src/intent/mod.rs` | 162 | `("muthoot", "Muthoot Finance", r"(?i)\b(muthoot)\b")` | Competitor pattern hardcoded |
| `crates/text_processing/src/intent/mod.rs` | 1215 | Muthoot regex pattern | Should load from config |
| `crates/text_processing/src/intent/mod.rs` | 1313 | `("iifl", "IIFL", r"(?i)\b(iifl|ii\s*fl)\b")` | Should load from config |
| `crates/text_processing/src/slot_extraction/mod.rs` | 118-149 | `INTENT_PATTERNS` static | 9+ patterns hardcoded |

#### 2.2.3 Entity Extraction - Hardcoded Lenders

| File | Line | Hardcoded Content | Issue |
|------|------|-------------------|-------|
| `crates/text_processing/src/entities/mod.rs` | 622 | `vec!["Muthoot".to_string(), "IIFL".to_string()]` | Hardcoded lender list |
| `crates/text_processing/src/entities/mod.rs` | 627 | Test data with competitors | Should be config-driven |

#### 2.2.4 Compliance - Hardcoded Names

| File | Line | Hardcoded Content | Issue |
|------|------|-------------------|-------|
| `crates/text_processing/src/compliance/checker.rs` | 273 | `"Muthoot".to_string()` | Competitor in compliance |
| `crates/text_processing/src/compliance/checker.rs` | 275 | `"Kotak".to_string()` | Brand in compliance |

#### 2.2.5 Config Defaults - Domain-Specific Values

| File | Line | Hardcoded Content | Issue |
|------|------|-------------------|-------|
| `config/default.yaml` | 77 | `kotak_interest_rate: 10.5` | Should be in domain config |
| `config/default.yaml` | 97-99 | `competitor_rates: muthoot: 18.0, iifl: 17.5` | Should be in domain config |

### 2.3 P1 - High Priority Hardcoding

#### 2.3.1 Tool Implementations

| File | Line | Hardcoded Content | Issue |
|------|------|-------------------|-------|
| `crates/tools/src/domain_tools/tools/savings.rs` | 84 | `"Muthoot".into()` | Competitor in tool code |
| `crates/tools/src/domain_tools/tools/savings.rs` | 86 | `"IIFL".into()` | Competitor in tool code |

#### 2.3.2 Persuasion Engine - Objection IDs

| File | Line | Hardcoded Content | Issue |
|------|------|-------------------|-------|
| `crates/agent/src/persuasion.rs` | 49-59 | `GOLD_SECURITY` constant | Domain-specific objection |

#### 2.3.3 Core Traits - Example Code

| File | Line | Hardcoded Content | Issue |
|------|------|-------------------|-------|
| `crates/core/src/traits/competitors.rs` | 16 | `analyzer.get_rate("muthoot")` | Example with hardcoded name |
| `crates/core/src/traits/competitors.rs` | 132 | Comment lists competitors | Documentation with names |

#### 2.3.4 Data Files

| File | Content | Issue |
|------|---------|-------|
| `data/branches.json` | 30+ Kotak branch entries | Bank-specific data |
| `data/gold_loan_vocab.txt` | 125 gold loan terms | Domain vocabulary |

### 2.4 P2 - Medium Priority Hardcoding

#### 2.4.1 Memory Compressor Defaults

| File | Line | Hardcoded Content | Issue |
|------|------|-------------------|-------|
| `crates/agent/src/memory/compressor.rs` | 37-68 | `priority_entities` with gold terms | Should load from config |
| `crates/agent/src/memory/mod.rs` | Various | Memory watermarks hardcoded | Should be in domain.yaml |

#### 2.4.2 Compliance Rules Defaults

| File | Line | Hardcoded Content | Issue |
|------|------|-------------------|-------|
| `crates/text_processing/src/compliance/rules.rs` | 40-63 | `min_rate: 7.0, max_rate: 24.0` | Gold loan defaults |

#### 2.4.3 FSM Stage Messages

| File | Line | Hardcoded Content | Issue |
|------|------|-------------------|-------|
| `crates/agent/src/fsm_adapter.rs` | 138-170 | Generic fallback messages | Should be configurable |

#### 2.4.4 AI Disclosure Messages

| File | Line | Hardcoded Content | Issue |
|------|------|-------------------|-------|
| `crates/agent/src/conversation.rs` | 250-265 | 9 language AI disclosures | Should be in compliance.yaml |

### 2.5 P3 - Acceptable / Low Priority

- **Knowledge Base Files** (`knowledge/*.yaml`) - 250+ references, appropriate tier
- **Test Data** - Domain-specific assertions for regression testing
- **Documentation Comments** - Non-blocking
- **Config Examples** - Appropriate in domain-specific config files

---

## 3. Trait Structure & Abstraction Analysis

### 3.1 Core Traits Inventory (16+ Traits)

#### Domain-Agnostic Infrastructure Traits
| Trait | File | Status | Notes |
|-------|------|--------|-------|
| `SpeechToText` | `crates/stt/src/lib.rs` | ✅ Excellent | Provider-agnostic |
| `TextToSpeech` | `crates/tts/src/lib.rs` | ✅ Excellent | Provider-agnostic |
| `LanguageModel` | `crates/llm/src/lib.rs` | ✅ Excellent | Ollama/Claude/OpenAI |
| `Tool` | `crates/tools/src/lib.rs` | ✅ Excellent | MCP-compatible |
| `ToolFactory` | `crates/tools/src/factory.rs` | ✅ Excellent | Factory pattern |
| `Retriever` | `crates/rag/src/lib.rs` | ✅ Excellent | Metadata filtering |
| `FrameProcessor` | `crates/pipeline/src/lib.rs` | ✅ Excellent | 15+ frame types |
| `ConversationFSM` | `crates/agent/src/fsm.rs` | ✅ Excellent | State machine |

#### Business Logic Traits (Crown Jewels)
| Trait | File | Status | Notes |
|-------|------|--------|-------|
| `DomainCalculator` | `crates/core/src/traits/calculator.rs` | ✅ Excellent | All formulas parameterized |
| `SlotSchema` | `crates/config/src/domain/slots.rs` | ✅ Excellent | Language-specific patterns |
| `ConversationGoalSchema` | `crates/config/src/domain/goals.rs` | ✅ Excellent | Config-driven NBA logic |
| `LeadScoringStrategy` | `crates/core/src/traits/scoring.rs` | ✅ Excellent | Cold/Warm/Hot/Qualified |
| `ObjectionHandler` | `crates/core/src/traits/objections.rs` | ✅ Excellent | ACRE response patterns |
| `CompetitorAnalyzer` | `crates/core/src/traits/competitors.rs` | ✅ Excellent | Rate comparison logic |
| `SegmentDetector` | `crates/core/src/traits/segments.rs` | ✅ Excellent | Customer segmentation |

### 3.2 Design Patterns Identified

| Pattern | Implementation | Quality |
|---------|---------------|---------|
| **Factory** | ToolFactory + ToolFactoryRegistry | ✅ Excellent |
| **Strategy** | LeadScoringStrategy, PersuasionStrategy, DomainCalculator | ✅ Excellent |
| **Builder** | ConfigGoalDefinition, ConfigObjectionDefinition, CompetitorInfo | ✅ Excellent |
| **Registry** | ToolFactoryRegistry for multi-domain support | ✅ Excellent |
| **Bridge** | DomainBridge (config → trait adapter) | ✅ Excellent |
| **Chain of Responsibility** | FrameProcessor pipeline | ✅ Excellent |

### 3.3 Missing Abstraction Layers

#### High Priority
| Abstraction | Purpose | Complexity |
|-------------|---------|------------|
| Configuration Trait | Hot-reload support | Medium |
| Unified Error Handling | Consistent error types | Low |
| Feature Flags Trait | Conditional compilation | Medium |

#### Medium Priority
| Abstraction | Purpose | Complexity |
|-------------|---------|------------|
| Integration Registry | CRM, Calendar, SMS providers | Medium |
| Metrics/Analytics | Domain-agnostic telemetry | Medium |
| Generic Caching | Multi-backend cache trait | Low |

---

## 4. Config Wiring Status

### 4.1 Properly Wired (✅)

| Config Section | Source | Destination | Status |
|----------------|--------|-------------|--------|
| Domain Constants | `domain.yaml` | AgentDomainView → Agent | ✅ Complete |
| Slots | `slots.yaml` | SlotsConfig → DST → Memory | ✅ Complete |
| Phonetic Corrections | `domain.yaml` | PhoneticCorrector | ✅ Complete |
| Stage Management | `stages.yaml` | StageConfigProvider | ✅ Complete |
| Intent-to-Goal Mapping | `goals.yaml` | ConversationGoalSchema | ✅ Complete |
| Slot Aliases | `slots.yaml` | canonical_fact_key() | ✅ Complete (P16) |
| Quality Tier Parsing | `slots.yaml` | parse_quality_tier() | ✅ Complete (P18) |
| Objection Handling | `objections.yaml` | PersuasionEngine | ✅ Complete |
| Competitor Data | `domain.yaml` | CompetitorAnalyzer | ✅ Complete |

### 4.2 Partially Wired (⚠️)

| Config Section | Source | Issue |
|----------------|--------|-------|
| Query Expansion | `domain.yaml` | Defined but RAG usage unclear |
| Domain Boost (RAG) | `domain.yaml` | Config exists, wiring not verified |
| Memory Compressor | `domain.yaml` | Config loaded but defaults used |

### 4.3 Not Wired (❌)

| Config Section | Source | Issue |
|----------------|--------|-------|
| Currency Config | `domain.yaml` | Loaded but never used |
| Slot Display Mappings | `slots.yaml` | 9 mappings defined, never accessed |
| Memory Capacity | `domain.yaml` | Uses hardcoded 4000/8000 tokens |
| Language Formatting | N/A | No config for number/currency per language |

### 4.4 Missing Infrastructure

| Item | Description | Impact |
|------|-------------|--------|
| Base Defaults Layer | `config/base/defaults.yaml` not implemented | Each domain must specify everything |
| Config Inheritance | No parent → child config merging | Duplication risk |
| Startup Validation | Required sections not validated | Silent failures |

---

## 5. Business Logic Coupling Analysis

### 5.1 Calculation Logic

| Component | Location | Coupling Level | Recommendation |
|-----------|----------|----------------|----------------|
| Interest Rate Tiers | `domain.yaml:20-50` | Config-driven | ✅ Good |
| LTV Calculation | `domain.yaml` | Config-driven | ✅ Good |
| Purity Factors | `domain.yaml` | Config-driven | ✅ Good |
| Slot Rate Lookup | `slots.rs:174-181` | Config-driven | ✅ Good |

### 5.2 Intent Handling

| Component | Location | Coupling Level | Recommendation |
|-----------|----------|----------------|----------------|
| Intent Patterns | `slot_extraction/mod.rs:118-149` | **HARDCODED** | ❌ Move to config |
| Competitor Patterns | `intent/mod.rs:162,1215,1313` | **HARDCODED** | ❌ Move to config |
| Intent-Tool Mapping | `intent_tool_mappings.yaml` | Config-driven | ✅ Good |

### 5.3 Compliance Rules

| Component | Location | Coupling Level | Recommendation |
|-----------|----------|----------------|----------------|
| Rate Bounds | `rules.rs:40-63` | Hardcoded defaults | ⚠️ Use config |
| Forbidden Phrases | `compliance.yaml` | Config-driven | ✅ Good |
| Disclaimers | `compliance.yaml` | Config-driven | ✅ Good |

### 5.4 Lead Scoring

| Component | Location | Coupling Level | Recommendation |
|-----------|----------|----------------|----------------|
| Signal Weights | `lead_scoring.yaml` | Config-driven | ✅ Good |
| Thresholds | `scoring.yaml` | Config-driven | ✅ Good |
| High-Value Threshold | `scoring.yaml:19-22` | Domain-specific value | ⚠️ Document |

---

## 6. Prompts & Templates Analysis

### 6.1 Already Config-Driven (✅)

| Template Type | Location | Status |
|---------------|----------|--------|
| System Prompts | `prompts/system.yaml` | ✅ Brand variables supported |
| Stage Guidance | `stages.yaml` | ✅ Per-stage templates |
| Objection Responses | `objections.yaml` | ✅ ACRE pattern |
| SMS Templates | `tools/sms_templates.yaml` | ✅ 11 templates, bilingual |
| Fallback Responses | Via `stage_fallback_response()` | ✅ Config-driven |

### 6.2 Needs Migration (❌)

| Template Type | Location | Issue |
|---------------|----------|-------|
| FSM Stage Messages | `fsm_adapter.rs:138-170` | Generic fallbacks hardcoded |
| AI Disclosure | `conversation.rs:250-265` | 9 languages hardcoded |
| SMS Sender ID | `sms_templates.yaml:89` | "KOTKBK" hardcoded |
| Context Headers | `response.rs` headers | "## Relevant Information" etc. |

### 6.3 Brand Variables in Use

| Variable | Usage |
|----------|-------|
| `{agent_name}` | Agent persona name |
| `{bank_name}` / `{company_name}` | Company branding |
| `{product_name}` | Product name (e.g., "Gold Loan") |
| `{helpline}` | Customer helpline number |
| `{brand.bank_name}` | Alternative notation |

---

## 7. Priority Recommendations

### 7.1 P0 - Critical (Block Multi-Domain)

| # | Issue | File | Fix |
|---|-------|------|-----|
| 1 | Intent patterns hardcoded | `slot_extraction/mod.rs:118-149` | Move `INTENT_PATTERNS` to YAML |
| 2 | Competitor patterns hardcoded | `intent/mod.rs:162,1215,1313` | Load from `domain.yaml` competitors |
| 3 | Lender list hardcoded | `entities/mod.rs:622` | Load from config |
| 4 | LLM prompt has "gold loan" | `llm/src/prompt.rs:320` | Use `{product_name}` |
| 5 | Compliance checker names | `compliance/checker.rs:273-275` | Load from config |

### 7.2 P1 - High Priority

| # | Issue | File | Fix |
|---|-------|------|-----|
| 6 | Savings tool competitors | `tools/savings.rs:84-86` | Use domain view |
| 7 | `GOLD_SECURITY` objection | `persuasion.rs:49-59` | Make all objection IDs config-driven |
| 8 | Memory compressor defaults | `compressor.rs:37-68` | Load from `domain.yaml` |
| 9 | AI disclosure messages | `conversation.rs:250-265` | Move to `compliance.yaml` |
| 10 | Base defaults missing | N/A | Create `config/base/defaults.yaml` |

### 7.3 P2 - Medium Priority

| # | Issue | File | Fix |
|---|-------|------|-----|
| 11 | FSM stage messages | `fsm_adapter.rs:138-170` | Add to `stages.yaml` |
| 12 | Rate validation defaults | `rules.rs:40-63` | Load from config |
| 13 | Currency config unused | `domain.yaml` | Wire to formatting |
| 14 | Memory capacity hardcoded | `memory/mod.rs` | Use config values |
| 15 | SMS sender ID | `sms_templates.yaml:89` | Move to brand config |

### 7.4 P3 - Nice to Have

| # | Issue | File | Fix |
|---|-------|------|-----|
| 16 | Slot display mappings | `slots.yaml` | Wire to UI |
| 17 | Query expansion verify | `domain.yaml` → RAG | Verify wiring |
| 18 | Config startup validation | N/A | Add required checks |
| 19 | Trait documentation | N/A | Add architecture docs |

---

## 8. Implementation Roadmap

### Phase 1: Critical Path (P0 Fixes)

```
Week 1: Intent & Entity Extraction
├─ Move INTENT_PATTERNS to config YAML
├─ Load competitor patterns from domain.yaml
├─ Make lender extraction config-driven
└─ Update tests to use config fixtures

Week 2: Prompts & Compliance
├─ Parameterize LLM prompt templates
├─ Move compliance checker names to config
└─ Add brand variable substitution tests
```

### Phase 2: High Priority (P1 Fixes)

```
Week 3: Tools & Persuasion
├─ Refactor savings tool to use domain view
├─ Make objection IDs fully config-driven
└─ Wire memory compressor to domain config

Week 4: Infrastructure
├─ Create config/base/defaults.yaml
├─ Move AI disclosure to compliance.yaml
└─ Add config inheritance mechanism
```

### Phase 3: Polish (P2 Fixes)

```
Week 5: Remaining Hardcoding
├─ FSM stage messages to config
├─ Rate validation defaults from config
├─ Wire currency config to formatting
└─ Memory capacity from config

Week 6: Validation & Testing
├─ Add config startup validation
├─ Multi-domain integration tests
└─ Documentation updates
```

---

## 9. File Reference Index

### 9.1 Config Files

| File | Purpose | Domain-Specific Content |
|------|---------|------------------------|
| `config/domains/gold_loan/domain.yaml` | Core constants, branding, competitors | Yes - appropriate |
| `config/domains/gold_loan/slots.yaml` | Slot definitions, extraction patterns | Yes - appropriate |
| `config/domains/gold_loan/stages.yaml` | Conversation stages, guidance | Yes - appropriate |
| `config/domains/gold_loan/objections.yaml` | Objection handling templates | Yes - appropriate |
| `config/domains/gold_loan/compliance.yaml` | Regulatory rules | Yes - appropriate |
| `config/domains/gold_loan/intent_tool_mappings.yaml` | Intent → tool routing | Yes - appropriate |
| `config/domains/gold_loan/tools/sms_templates.yaml` | SMS message templates | Yes - appropriate |

### 9.2 Source Files with Hardcoding

| File | Lines | Issue Type |
|------|-------|------------|
| `crates/text_processing/src/slot_extraction/mod.rs` | 118-149 | Intent patterns |
| `crates/text_processing/src/intent/mod.rs` | 162, 1215, 1313 | Competitor patterns |
| `crates/text_processing/src/entities/mod.rs` | 622, 627 | Lender list |
| `crates/text_processing/src/compliance/checker.rs` | 273-275 | Brand names |
| `crates/text_processing/src/compliance/rules.rs` | 40-63 | Rate defaults |
| `crates/llm/src/prompt.rs` | 320 | "gold loan" text |
| `crates/agent/src/persuasion.rs` | 49-59 | Objection IDs |
| `crates/agent/src/memory/compressor.rs` | 37-68 | Priority entities |
| `crates/agent/src/conversation.rs` | 250-265 | AI disclosure |
| `crates/agent/src/fsm_adapter.rs` | 138-170 | Stage messages |
| `crates/tools/src/domain_tools/tools/savings.rs` | 84-86 | Competitor names |

### 9.3 Key Abstraction Files

| File | Purpose |
|------|---------|
| `crates/config/src/domain/master.rs` | MasterDomainConfig loader |
| `crates/config/src/domain/views.rs` | Crate-specific view pattern |
| `crates/config/src/domain/bridge.rs` | Config → trait adapter |
| `crates/config/src/domain/slots.rs` | Slot schema with aliases |
| `crates/core/src/traits/*.rs` | Domain-agnostic trait definitions |

---

## Appendix A: Config Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│  YAML CONFIG FILES                                           │
│  config/domains/{domain_id}/*.yaml                           │
└────────────────────┬────────────────────────────────────────┘
                     │ load & merge
                     ▼
┌─────────────────────────────────────────────────────────────┐
│  MasterDomainConfig (master.rs)                              │
│  - Single unified config object                              │
│  - Validated at startup                                      │
└────────────────────┬────────────────────────────────────────┘
                     │ create views
                     ▼
┌─────────────────────────────────────────────────────────────┐
│  Domain Views                                                │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐    │
│  │ AgentView   │ │ LlmView     │ │ ToolsView           │    │
│  │ - stages    │ │ - prompts   │ │ - schemas           │    │
│  │ - scoring   │ │ - brand     │ │ - responses         │    │
│  │ - slots     │ │ - constants │ │ - mappings          │    │
│  └─────────────┘ └─────────────┘ └─────────────────────┘    │
└────────────────────┬────────────────────────────────────────┘
                     │ inject
                     ▼
┌─────────────────────────────────────────────────────────────┐
│  RUNTIME COMPONENTS                                          │
│  Agent, DST, LLM, Tools, RAG, Compliance                    │
└─────────────────────────────────────────────────────────────┘
```

---

## Appendix B: Multi-Domain Onboarding Checklist

To onboard a new domain (e.g., `car_loan`), create:

```bash
config/domains/car_loan/
├── domain.yaml           # Brand, constants, competitors
├── slots.yaml            # Slots with extraction patterns
├── intents.yaml          # Domain-specific intents
├── stages.yaml           # Conversation flow
├── goals.yaml            # Conversation goals
├── objections.yaml       # Objection handling
├── compliance.yaml       # Regulatory rules
├── scoring.yaml          # Lead scoring weights
├── tools/
│   ├── schemas.yaml      # Tool input schemas
│   ├── responses.yaml    # Tool response templates
│   └── sms_templates.yaml
└── prompts/
    └── system.yaml       # System prompts
```

**No code changes required** once P0/P1 fixes are complete.

---

## Appendix C: Test Assertions Verifying Domain-Agnosticism

The codebase includes tests that verify no hardcoded domain terms leak through:

| File | Line | Assertion |
|------|------|-----------|
| `stage_config.rs` | 303 | `assert!(!guidance.contains("Kotak"));` |
| `stage_config.rs` | 357-358 | Asserts questions don't mention "kotak" |
| `dst/slots.rs` | 353 | `assert!(!instruction.contains("Kotak"));` |
| `memory/core.rs` | 681 | `assert!(!persona.role.to_lowercase().contains("kotak"));` |
| `persistence/sms.rs` | 374 | `assert!(!msg.contains("Kotak"));` |

These tests serve as regression guards against re-introducing domain-specific hardcoding.

---

**Document Status:** Complete
**Next Review:** After P0/P1 fixes implemented
**Maintainer:** Architecture Team
