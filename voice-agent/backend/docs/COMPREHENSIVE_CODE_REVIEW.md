# Comprehensive Backend Code Review

**Date:** 2026-01-06
**Scope:** voice-agent/backend/ (~60,000+ LOC across 12 crates)
**Focus:** Domain-agnosticism, design patterns, code quality, architecture

---

## Executive Summary

The voice-agent backend is a **sophisticated conversational AI platform** with excellent foundational architecture but **significant domain coupling to gold loans/Kotak Bank**. While 75% of the codebase follows good design patterns, approximately **200+ hardcoded domain-specific strings** and **2,500+ lines of domain-specific logic in core modules** prevent true domain-agnosticism.

### Key Metrics

| Metric | Value |
|--------|-------|
| Total Lines of Code | ~60,000+ |
| Crates Analyzed | 12 |
| Hardcoded Domain Strings | 200+ |
| Files Requiring Refactor | 35+ |
| Estimated Refactor Effort | 80-120 hours |

### Overall Health Score: **6.5/10**

| Aspect | Score | Notes |
|--------|-------|-------|
| Architecture | 8/10 | Excellent trait-based design, clean layering |
| Domain Separation | 4/10 | Core has too much gold-loan code |
| Config-Driven | 7/10 | Good YAML structure, but fallbacks hardcoded |
| Code Quality | 7/10 | Good patterns, some duplication |
| Performance | 7/10 | Lock contention issues identified |
| Maintainability | 6/10 | Mixed - good in some areas, tight coupling in others |

---

## Table of Contents

1. [Critical Findings](#1-critical-findings)
2. [Crate-by-Crate Analysis](#2-crate-by-crate-analysis)
3. [Domain-Specific Hardcoding Inventory](#3-domain-specific-hardcoding-inventory)
4. [Design Pattern Issues](#4-design-pattern-issues)
5. [Performance & Concurrency Issues](#5-performance--concurrency-issues)
6. [Recommended Refactoring Plan](#6-recommended-refactoring-plan)
7. [File Reference Guide](#7-file-reference-guide)

---

## 1. Critical Findings

### 1.1 Domain Coupling in Core Modules

**Problem:** The `core` crate (abstraction layer) contains 2,500+ lines of gold-loan-specific code that should be in domain config or separate crates.

**Affected Files:**
- `core/src/traits/calculator.rs:265-310` - `gold_loan_defaults()`
- `core/src/traits/goals.rs:397-450` - Hardcoded gold loan goals
- `core/src/traits/objections.rs:494-550` - Kotak-specific objection handling
- `core/src/traits/competitors.rs:407-505` - Muthoot/Manappuram hardcoded
- `core/src/personalization/adaptation.rs:110-410` - 65% gold-loan specific

### 1.2 Missing Abstract Factory for Domains

**Problem:** No `DomainAgentFactory` trait exists to create domain-specific agents from config.

**Current State:**
```rust
// Only gold loan agent exists
pub struct GoldLoanAgent { ... }

// Should be:
pub trait DomainAgentFactory {
    fn create(session_id: &str, config: AgentConfig) -> Box<dyn Agent>;
}
```

### 1.3 Enum-Based Domain Logic (Anti-Pattern)

**Problem:** Business domain concepts are hardcoded as Rust enums instead of config-driven data.

**Examples:**
- `ConversationGoal` enum with `BalanceTransfer`, `NewLoan` variants
- `ObjectionType` enum with `GoldSafety`, `CurrentLenderSatisfaction`
- `Feature` enum with `RbiRegulated`, `ZeroForeclosure`, `DoorstepService`

**Impact:** Adding new products requires code changes.

### 1.4 Hardcoded Brand References

**Problem:** "Kotak Mahindra Bank" appears in 25+ code locations.

**Locations:**
- `agent/src/agent.rs:106, 281, 370` - Constructor persona strings
- `llm/src/prompt.rs` - System prompt templates
- `tools/src/gold_loan/tools.rs:1266-1298` - SMS templates
- `persistence/src/sms.rs:127, 137` - SMS message bodies

### 1.5 Rate Calculation Hardcoding

**Problem:** Interest rates, LTV, gold price hardcoded as code constants.

| Value | Locations | Should Be |
|-------|-----------|-----------|
| 11.5%, 10.5%, 9.5% rates | `calculator.rs:270-280`, `tools.rs` | `domain.yaml` |
| 75% LTV | `calculator.rs:262`, `sms.rs:136` | `domain.yaml` |
| 7500.0 gold price | `calculator.rs:263, 303`, `customer.rs:221` | API + config |
| Competitor rates | `competitors.rs:137-189` | `competitors.yaml` |

---

## 2. Crate-by-Crate Analysis

### 2.1 Config Crate (7,018 LOC)

**Purpose:** Configuration loading and domain abstraction

**Strengths:**
- Excellent `DomainBridge` factory pattern (`bridge.rs`)
- Clean `MasterDomainConfig` with hierarchical YAML loading
- Good view separation (`AgentDomainView`, `LlmDomainView`, `ToolsDomainView`)

**Issues:**

| Issue | File:Line | Severity |
|-------|-----------|----------|
| `gold_loan_defaults()` in defaults | `master.rs:205` | HIGH |
| Hardcoded doorstep cities | `branches.rs:152-162` | HIGH |
| `gold_loan_branches()` method name | `branches.rs:66-72` | MEDIUM |
| Hardcoded competitor rates | `competitors.rs:177-187` | HIGH |
| `views.rs` too large (938 lines) | `views.rs:1-938` | MEDIUM |

**Recommendations:**
1. Move all `gold_loan_defaults()` to YAML
2. Rename `gold_loan_branches()` → `branches_with_product(product)`
3. Split `views.rs` into `agent_view.rs`, `llm_view.rs`, `tools_view.rs`

### 2.2 Agent Crate (16,252 LOC)

**Purpose:** Conversational AI agent implementation

**Strengths:**
- Good trait abstraction (`Agent`, `PrefetchingAgent`, `PersonalizableAgent`)
- Config-driven LLM/RAG initialization
- Domain view override capability via `with_domain_view()`

**Issues:**

| Issue | File:Line | Severity |
|-------|-----------|----------|
| Constructor duplication (303 lines) | `agent.rs:94-426` | HIGH |
| "Kotak Mahindra Bank" 3x | `agent.rs:106, 281, 370` | HIGH |
| Hardcoded slot mapping | `agent.rs:762-778` | HIGH |
| Intent→tool mapping hardcoded | `agent.rs:1471-1524` | HIGH |
| Competitor patterns hardcoded | `dst/extractor.rs:139-171` | HIGH |
| `ConversationGoal` enum not trait | `dst/slots.rs:86-94` | MEDIUM |

**Recommendations:**
1. Use builder pattern for constructors
2. Load intent→tool mapping from config
3. Convert `ConversationGoal` to trait + config
4. Extract competitor patterns to YAML

### 2.3 Core Crate (13,916 LOC)

**Purpose:** Domain-agnostic abstractions and traits

**Strengths:**
- Excellent speech/LLM/retriever traits
- Good `DomainContext` design
- Clean pipeline/FSM abstractions

**Critical Issues:**

| Issue | File:Line | Severity |
|-------|-----------|----------|
| `gold_loan_defaults()` in calculator | `calculator.rs:265` | CRITICAL |
| `gold_loan_defaults()` in goals | `goals.rs:397` | CRITICAL |
| `gold_loan_defaults()` in objections | `objections.rs:494` | CRITICAL |
| Feature enum hardcoding | `adaptation.rs:13-43` | HIGH |
| Objection detection hardcoded | `adaptation.rs:111-178` | HIGH |
| "Kotak" in SegmentAdapter | `adaptation.rs:238-410` | HIGH |

**Recommendations:**
1. **REMOVE all `gold_loan_defaults()` from core** - move to config crate
2. Convert `Feature` and `Objection` enums to config-driven data
3. Create `DomainConfigProvider` trait
4. Move `personalization/adaptation.rs` domain logic to config

### 2.4 Tools Crate (2,142 LOC)

**Purpose:** MCP-compatible tool implementations

**Strengths:**
- Good `ToolsDomainView` injection pattern
- Clean tool registry with hot-reload support
- Proper factory methods (`create_registry_with_view`)

**Issues:**

| Issue | File:Line | Severity |
|-------|-----------|----------|
| Tool fallback values hardcoded | `tools.rs:35-73` | HIGH |
| Document checklist hardcoded | `tools.rs:1441-1559` | MEDIUM |
| No tool enable/disable config | N/A | MEDIUM |

**Recommendations:**
1. Remove hardcoded fallback values - require config injection
2. Move document checklists to YAML
3. Add `tools.yaml` for enable/disable per tool

### 2.5 Pipeline Crate (9,397 LOC)

**Purpose:** STT, TTS, VAD, audio processing

**Strengths:**
- Excellent factory patterns for backends
- Good VAD lock consolidation (P0 FIX)
- Clean processor chain architecture

**Issues:**

| Issue | File:Line | Severity |
|-------|-----------|----------|
| "Kotak Gold Loan" in LLM default | `orchestrator.rs:105-108` | HIGH |
| Domain entities in decoder | `decoder.rs:416` | MEDIUM |
| Lock contention in TTS | `streaming.rs:138-144` | MEDIUM |
| Unsafe pointer in TTS | `tts/mod.rs:132` | MEDIUM |

**Recommendations:**
1. Move LLM system prompt to config
2. Load STT domain vocabulary from config
3. Consolidate TTS Mutex fields

### 2.6 Server Crate (5,483 LOC)

**Purpose:** HTTP/WebSocket server

**Strengths:**
- **Clean domain separation achieved** (P12 refactor success)
- Proper `MasterDomainConfig` integration
- Session store abstraction

**Issues:**

| Issue | File:Line | Severity |
|-------|-----------|----------|
| Cleanup task never started | `main.rs` | CRITICAL |
| Sender Mutex contention | `websocket.rs:107` | HIGH |
| Session persistence incomplete | `main.rs` | HIGH |
| Phonetic corrector hardcoded | `state.rs:69` | MEDIUM |

**Recommendations:**
1. Add `state.sessions.start_cleanup_task()` call
2. Refactor sender to use channels instead of Mutex
3. Persist session on state transitions

### 2.7 LLM Crate

**Purpose:** LLM client abstraction

**Strengths:**
- Multi-provider support (Ollama, OpenAI, Claude)
- Good speculative execution config
- Clean streaming implementation

**Issues:**

| Issue | File:Line | Severity |
|-------|-----------|----------|
| "Gold Loan specialist" role hardcoded | `prompt.rs` | MEDIUM |
| Stage guidance text hardcoded | `prompt.rs:425-445` | MEDIUM |

**Recommendations:**
1. Load persona role from config
2. Load stage guidance from `stages.yaml`

### 2.8 RAG Crate (9,010 LOC)

**Purpose:** Retrieval-Augmented Generation

**Strengths:**
- Sophisticated agentic retrieval
- Good query expansion and domain boosting
- Proper caching with LRU

**Issues:**

| Issue | File:Line | Severity |
|-------|-----------|----------|
| `gold_loan()` factory only | `query_expansion.rs:160-171` | CRITICAL |
| Collection name hardcoded | `vector_store.rs:39` | HIGH |
| Gold loan stopwords | `query_expansion.rs:101-144` | HIGH |
| LLM prompts with gold loan | `agentic.rs:614, 634` | HIGH |

**Recommendations:**
1. Create parameterized `QueryExpander::new(config)`
2. Make collection name configurable
3. Load stopwords from domain config

### 2.9 Text Processing Crate

**Purpose:** Translation, grammar, compliance

**Strengths:**
- Good IndicTrans2 implementation
- Clean compliance checker abstraction

**Issues:**

| Issue | File:Line | Severity |
|-------|-----------|----------|
| Grammar default domain | `grammar/mod.rs:56` | MEDIUM |
| Kotak phonetic corrections | `phonetic_corrector.rs:171-178` | MEDIUM |

---

## 3. Domain-Specific Hardcoding Inventory

### 3.1 Brand Names (25+ locations)

```
"Kotak Mahindra Bank" - agent.rs, prompt.rs, persuasion.rs, sms.rs, tools.rs
"Kotak" - phonetic_corrector.rs, domain_boost.rs, agentic.rs
```

### 3.2 Competitor Names (115+ references)

```
Muthoot Finance, Manappuram Gold Loan, IIFL Gold Loan, HDFC Bank,
SBI, Federal Bank, ICICI Bank, Axis Bank, PNB
```

**Locations:** `competitors.rs`, `extractor.rs`, `persuasion.rs`, `tools.rs`

### 3.3 Business Constants

| Constant | Value | Hardcoded In |
|----------|-------|--------------|
| Interest rates | 9.5%, 10.5%, 11.5% | calculator.rs, tools.rs |
| LTV | 75% | calculator.rs, sms.rs |
| Gold price | 7500.0 INR/gram | calculator.rs, customer.rs, tools.rs |
| Min loan | 10,000 INR | calculator.rs |
| Max loan | 25,000,000 INR | calculator.rs |
| Processing fee | 1% | calculator.rs |

### 3.4 Domain Vocabulary

| Category | Examples | Should Be |
|----------|----------|-----------|
| Slot names | `gold_weight`, `gold_purity`, `current_lender` | Config slots.yaml |
| Goals | `BalanceTransfer`, `NewLoan`, `EligibilityCheck` | Config goals.yaml |
| Features | `RbiRegulated`, `ZeroForeclosure`, `DoorstepService` | Config features.yaml |
| Objections | `GoldSafety`, `CurrentLenderSatisfaction` | Config objections.yaml |

---

## 4. Design Pattern Issues

### 4.1 Missing Patterns

| Pattern | Purpose | Current State |
|---------|---------|---------------|
| Domain Factory | Create domain-specific agents | Not implemented |
| Strategy Pattern | Slot schemas, scoring | Hardcoded enums |
| Plugin Architecture | Tool registration | Partial (registry exists) |
| Configuration Provider | Centralized config access | Scattered defaults |

### 4.2 Anti-Patterns Found

| Anti-Pattern | Location | Impact |
|--------------|----------|--------|
| God Object | `GoldLoanAgent` (55 fields) | Hard to test, maintain |
| Enum for domain data | `ConversationGoal`, `Feature` | Requires code changes |
| Duplicate code | Agent constructors (303 lines) | Maintenance burden |
| Mixed concerns | Config types with business logic | SRP violation |
| Hardcoded fallbacks | Tool implementations | Silent config bypass |

### 4.3 Good Patterns Found

| Pattern | Location | Quality |
|---------|----------|---------|
| Bridge Pattern | `DomainBridge` | Excellent |
| Factory Method | `create_stt_backend()`, `create_tts_backend()` | Excellent |
| Adapter Pattern | `LanguageModelAdapter` | Good |
| Builder Pattern | `ToolBuilder`, `ProcessorChainBuilder` | Good |
| View Pattern | `AgentDomainView`, `ToolsDomainView` | Excellent |

---

## 5. Performance & Concurrency Issues

### 5.1 Lock Contention

| Location | Issue | Impact | Fix |
|----------|-------|--------|-----|
| `websocket.rs:107` | Sender wrapped in Mutex | Message blocking | Use channels |
| `streaming.rs:138-144` | 3 separate Mutex fields | Lock overhead | Consolidate |
| `sentence_detector.rs` | 3 separate Mutex fields | Lock overhead | Consolidate |

### 5.2 Memory Issues

| Issue | Location | Impact |
|-------|----------|--------|
| Dual translation models | `candle_indictrans2.rs` | ~400MB memory |
| LRU cache cloning | `rag/cache.rs` | Unnecessary allocations |
| Domain terms reload | `domain_boost.rs` | Repeated HashMap creation |

### 5.3 Async/Await Issues

| Issue | Location | Fix |
|-------|----------|-----|
| Lock held across await | `websocket.rs:230` | Split lock scope |
| Spawned task not tracked | `websocket.rs:335` | Bind to session lifetime |
| Sync ONNX in async context | `orchestrator.rs:846` | Documented limitation |

---

## 6. Recommended Refactoring Plan

### Phase 1: Critical (Blocks Reusability) - 40 hours

1. **Remove `gold_loan_defaults()` from core** (8h)
   - Move to config crate with YAML backing
   - Files: calculator.rs, goals.rs, objections.rs, competitors.rs, scoring.rs

2. **Create DomainConfigProvider trait** (4h)
   - Single factory for all domain traits
   - Wire to server startup

3. **Parameterize brand/competitor names** (6h)
   - Extract to `config.brand.bank_name`
   - Load competitors from YAML only (no code fallback)

4. **Convert enums to config-driven data** (12h)
   - `ConversationGoal` → trait + YAML
   - `Feature` → data struct + YAML
   - `ObjectionType` → trait + YAML

5. **Fix agent constructor duplication** (4h)
   - Use builder pattern
   - Single initialization path

6. **Fix server cleanup task** (1h)
   - Add `start_cleanup_task()` call

7. **Add session state persistence** (5h)
   - Persist on stage transitions
   - Persist on significant state changes

### Phase 2: Important (Maintainability) - 30 hours

1. **Split views.rs** (4h)
   - `agent_view.rs`, `llm_view.rs`, `tools_view.rs`

2. **Consolidate Mutex fields** (6h)
   - TTS streaming state
   - Sentence detector state

3. **Parameterize RAG** (8h)
   - Collection name configurable
   - Domain stopwords from config
   - Query expansion factory

4. **Tool configuration** (6h)
   - Enable/disable per tool
   - Remove hardcoded fallbacks
   - Document checklists to YAML

5. **Pipeline domain extraction** (6h)
   - LLM system prompt configurable
   - STT vocabulary from config

### Phase 3: Nice-to-Have - 20 hours

1. **Performance optimizations** (8h)
   - WebSocket sender channels
   - Cache improvements
   - Lazy domain term loading

2. **Config validation** (6h)
   - Schema validation on load
   - Cross-field validation
   - Required field checks

3. **Multi-domain testing** (6h)
   - Test fixtures for non-gold-loan domain
   - Integration tests for config switching

---

## 7. File Reference Guide

### Critical Files (Immediate Attention)

| File | Lines | Issue |
|------|-------|-------|
| `core/src/traits/calculator.rs` | 548 | `gold_loan_defaults()` |
| `core/src/personalization/adaptation.rs` | 633 | 65% hardcoded |
| `agent/src/agent.rs` | 2,747 | Constructor duplication, brand hardcoding |
| `agent/src/dst/slots.rs` | 1,558 | Enum-based goals |
| `rag/src/query_expansion.rs` | 547 | `gold_loan()` factory only |
| `server/src/main.rs` | 313 | Missing cleanup task |

### Well-Designed Files (Reference)

| File | Lines | Pattern |
|------|-------|---------|
| `config/src/domain/bridge.rs` | 353 | Excellent adapter pattern |
| `config/src/domain/views.rs` | 938 | Good view separation |
| `pipeline/src/vad/magicnet.rs` | 657 | Consolidated Mutex |
| `pipeline/src/processors/chain.rs` | 505 | Channel-based pipeline |

### Config Files

| File | Purpose | Quality |
|------|---------|---------|
| `config/domains/gold_loan/domain.yaml` | Business constants | Good |
| `config/domains/gold_loan/slots.yaml` | DST slots | Good |
| `config/domains/gold_loan/competitors.yaml` | Competitor data | Good |
| `config/domains/gold_loan/objections.yaml` | Objection handling | Good |
| `config/default.yaml` | Infrastructure | Good |

---

## Conclusion

The voice-agent backend has a **solid architectural foundation** with excellent trait-based abstractions at the infrastructure level. However, achieving true **domain-agnosticism** requires:

1. **Moving ~2,500 lines of domain code** out of core
2. **Converting 4 enums to traits** with config backing
3. **Removing 200+ hardcoded strings** to configuration
4. **Creating domain factory pattern** for multi-tenant support

With the recommended refactoring, a new domain (e.g., car loans, personal loans) could be onboarded by:
1. Creating `config/domains/{domain_id}/` directory
2. Adding YAML files (domain.yaml, slots.yaml, competitors.yaml, etc.)
3. Setting `DOMAIN_ID` environment variable
4. **Zero code changes required**

---

*Report generated by Claude Code analysis - 2026-01-06*
