# Voice Agent Backend - Comprehensive Code Review Findings

## Executive Summary

This document presents a comprehensive deep-dive analysis of the `voice-agent/backend` codebase, focusing on making it **config-driven and domain-agnostic** so that new business use cases can be onboarded by defining YAML configs without code changes.

**Key Finding**: The codebase has strong foundational architecture with good trait-based abstractions, but contains significant **gold loan domain-specific hardcoding** that prevents true domain agnosticism.

---

## Table of Contents

1. [Critical Issues - Domain Leakage](#1-critical-issues---domain-leakage)
2. [Crate-by-Crate Analysis](#2-crate-by-crate-analysis)
3. [Hardcoded Values Inventory](#3-hardcoded-values-inventory)
4. [Trait Abstraction Recommendations](#4-trait-abstraction-recommendations)
5. [Code Organization Issues](#5-code-organization-issues)
6. [Concurrency & Performance Issues](#6-concurrency--performance-issues)
7. [Recommended Refactoring Plan](#7-recommended-refactoring-plan)

---

## 1. Critical Issues - Domain Leakage

### 1.1 Gold Loan Logic in Generic Code

| Location | Issue | Impact |
|----------|-------|--------|
| `agent/src/agent.rs:104-110` | "Kotak Mahindra Bank" and "Gold Loan Advisor" hardcoded in persona | Cannot reuse for other products |
| `agent/src/agent.rs:740-761` | Slot names (`gold_weight`, `gold_purity`) hardcoded | Cannot handle car/home loans |
| `agent/src/dst/slots.rs:66-106` | `ConversationGoal` enum is gold-loan specific | Blocks other product domains |
| `agent/src/persuasion.rs:312-340` | Competitor names (Muthoot, Manappuram, IIFL) hardcoded | Market-specific, not configurable |
| `core/src/customer.rs:250-285` | `CustomerSegment` enum hardcoded with gold-loan types | Leaks domain into core |
| `core/src/domain_context.rs:44-127` | `gold_loan()` method with hardcoded vocabulary | Forces gold-loan context |
| `tools/src/gold_loan/tools.rs:1599-1649` | Competitor data hardcoded in CompetitorComparisonTool | Cannot change without recompile |

### 1.2 Config Duplication

```
config/default.yaml    contains gold_loan settings
config/domain.yaml     also contains gold_loan settings (DUPLICATED)
config/domains/gold_loan/domain.yaml  (third location)
```

**Impact**: Changes must be made in multiple places; easy to miss one.

---

## 2. Crate-by-Crate Analysis

### 2.1 `crates/agent` (2,847 lines in agent.rs alone)

**Issues**:
- **Single Responsibility Violation**: `GoldLoanAgent` has 17 fields handling: conversation, tools, LLM, RAG, personalization, translation, persuasion, dialogue state, lead scoring
- **File Too Large**: `agent.rs` is 2,847 lines - should be split into 5-6 modules
- **Hardcoded Intent-to-Tool Mapping**: `maybe_call_tool()` at lines 1453-1506 hardcodes gold-loan intents
- **Persona Goal Duplicated 3 Times**: Lines 104-110, 279-285, 368-374

**Recommended Split**:
```
agent/src/
├── agent.rs (500 lines max - core orchestration)
├── tool_orchestrator.rs (tool calling, arg building)
├── prompt_builder.rs (LLM prompt construction)
├── lead_scorer_connector.rs (lead scoring integration)
├── dst_connector.rs (dialogue state integration)
└── personalization_connector.rs (personalization integration)
```

### 2.2 `crates/config`

**Strengths**:
- Well-structured 3-level hierarchy (base → domain → sub-configs)
- View pattern (`AgentDomainView`, `LlmDomainView`, `ToolsDomainView`) separates concerns
- 14 YAML config files for domain customization

**Issues**:
- Default domain hardcoded to "gold_loan" at `master.rs:182-183`
- Slot names hardcoded in `slots.rs:85, 95` (`"gold_purity"`, `"current_lender"`)
- Branch filtering hardcoded to `gold_loan_available` at `branches.rs:66-70`
- No trait-based validation pattern

### 2.3 `crates/tools`

**Strengths**:
- `Tool` trait from core provides MCP compatibility
- Factory functions support dependency injection

**Issues**:
- **3 Registry Creation Functions**: `create_default_registry()`, `create_registry_with_integrations()`, `create_registry_with_persistence()` - divergence risk
- **Helper Method Duplication**: `get_rate()`, `get_ltv()` copied across 3 tools
- **Phone Validation Duplicated**: Lines 410-412 and 1183-1185
- **Tool Definitions Not in Config**: Names/schemas hardcoded in Rust

### 2.4 `crates/core`

**Strengths**:
- `LanguageModel`, `SpeechToText`, `TextToSpeech`, `Retriever` traits are domain-agnostic
- `VoiceActivityDetector` trait well-designed

**Issues**:
- **Domain Leakage**: `CustomerSegment` enum at `customer.rs:5-21` is gold-loan specific
- **DomainContext Hardcoded**: `domain_context.rs:44-127` contains gold-loan vocabulary
- **Feature/Objection Enums**: `adaptation.rs:14-43, 88-109` are product-specific
- **SegmentAdapter Not Trait-Based**: Concrete struct, not pluggable

### 2.5 `crates/pipeline` & `crates/server`

**Strengths**:
- STT/TTS/VAD all expose async traits with factory functions
- `SessionStore` trait allows pluggable persistence
- ProcessorChain is generic and composable

**Issues**:
- `Session` struct directly embeds `GoldLoanAgent` - no agent trait
- WebSocket handler calls `session.agent.process_stream()` directly

### 2.6 `crates/llm` & `crates/rag`

**Excellent Architecture**:
- Provider-agnostic with factory pattern (Claude, Ollama, OpenAI, Azure)
- `EnhancedRetriever` uses composition: hybrid + agentic + expander + booster
- Query expansion supports synonyms, transliteration, domain terms
- Cross-lingual support with Devanagari detection

### 2.7 `crates/text_processing`

**Strengths**:
- Pluggable translation backends (Candle, ONNX, Noop)
- Trait-based `Translator`, `GrammarCorrector`, `PIIRedactor`

**Issues**:
- Language-to-IndicTrans code mapping duplicated in two files (candle_indictrans2.rs:1043-1069, indictrans2.rs:64-90)
- 100+ abbreviations hardcoded in lazy-static HashMap
- Custom abbreviations recompile regex on every call (performance issue)

---

## 3. Hardcoded Values Inventory

### 3.1 Business Logic Hardcoding

| Value | Location | Should Be |
|-------|----------|-----------|
| "Kotak Mahindra Bank" | agent.rs:106, 281, 370 | `config.brand.bank_name` |
| "Gold Loan Advisor" | agent.rs:106, 281, 370 | `config.persona.role` |
| "Priya" | agent.rs:99, agent_config.rs:103 | `config.persona.name` |
| Interest rates (10.5%, 11.5%, 9.5%) | constants.rs:12-33 | Load from domain YAML |
| LTV 75% | constants.rs:47-53 | Load from domain YAML |
| Gold price 7500/gram | constants.rs:56-72, tools.rs:815 | Load from domain YAML |
| Competitor rates (Muthoot 18%, etc.) | persuasion.rs:628-664, tools.rs:1599-1649 | Load from competitors.yaml |
| Lead scoring thresholds (30, 60, 80) | lead_scoring.rs:30-47 | Load from scoring.yaml |

### 3.2 Language/Localization Hardcoding

| Value | Location | Should Be |
|-------|----------|-----------|
| "hi" default language | pipeline.rs:217 | Environment/config |
| "hi-female-1" default voice | pipeline.rs:218 | Environment/config |
| Urgency keywords (English + Hindi) | lead_scoring.rs:352-365 | Config file |
| Month names (English only) | numbers.rs:398-414 | i18n resource file |
| Banking abbreviations (100+) | abbreviations.rs:11-144 | Config file with hot-reload |

### 3.3 Slot/Intent Hardcoding

| Slot Name | Location | Issue |
|-----------|----------|-------|
| `gold_weight`, `gold_purity` | agent.rs:740-761 | Cannot handle other collateral |
| `current_lender` | slots.rs:95 | Gold-loan specific |
| Intent mappings | agent.rs:1453-1506 | Cannot add new intents without code |
| ConversationGoal enum | dst/slots.rs:66-106 | Hardcoded goals |

---

## 4. Trait Abstraction Recommendations

### 4.1 Missing Traits (Should Be Created)

```rust
// 1. Generic Agent Trait (to replace hardcoded GoldLoanAgent)
pub trait DomainAgent: Send + Sync {
    type State: Send + Sync;
    async fn process(&self, input: &str) -> Result<String, AgentError>;
    async fn process_stream(&self, input: &str) -> Result<mpsc::Receiver<String>, AgentError>;
    fn domain_state(&self) -> &Self::State;
    fn stage(&self) -> Box<dyn ConversationStage>;
}

// 2. Configurable Conversation Stage
pub trait ConversationStage: Send + Sync + Debug {
    fn name(&self) -> &str;
    fn guidance(&self) -> &str;
    fn suggested_questions(&self) -> Vec<&str>;
    fn rag_context_fraction(&self) -> f32;
    fn transitions(&self) -> Vec<&str>;
}

// 3. Configurable Conversation Goal
pub trait ConversationGoal: Send + Sync {
    fn name(&self) -> &str;
    fn required_slots(&self) -> &[&str];
    fn optional_slots(&self) -> &[&str];
    fn completion_action(&self) -> &str;
}

// 4. Pluggable Signal Detection Strategy
pub trait SignalDetector: Send + Sync {
    fn detect(&self, text: &str, timing: &TurnTiming) -> Vec<BehaviorSignal>;
}

// 5. Pluggable Segment Adapter
pub trait SegmentAdapter: Send + Sync {
    fn adapt_response(&self, segment: &str, base_response: &str) -> String;
    fn get_value_propositions(&self, segment: &str) -> Vec<String>;
    fn handle_objection(&self, objection: &str, segment: &str) -> String;
}

// 6. Tool Factory Trait
pub trait ToolFactory: Send + Sync {
    fn create(&self, config: &ToolsDomainView, deps: &Dependencies) -> Arc<dyn Tool>;
}
```

### 4.2 Existing Traits That Are Good

| Trait | Location | Status |
|-------|----------|--------|
| `LanguageModel` | core/src/traits/llm.rs | Excellent - provider agnostic |
| `SpeechToText` | core/src/traits/speech.rs | Excellent |
| `TextToSpeech` | core/src/traits/speech.rs | Excellent |
| `VoiceActivityDetector` | core/src/traits/speech.rs | Good |
| `Retriever` | core/src/traits/retriever.rs | Good |
| `Tool` | core/src/traits/tool.rs | Good (MCP compatible) |
| `SessionStore` | server/src/session.rs | Good (pluggable persistence) |
| `Translator` | core/src/traits/text_processing.rs | Good |

---

## 5. Code Organization Issues

### 5.1 Files That Are Too Large

| File | Lines | Recommended Split |
|------|-------|-------------------|
| `agent/src/agent.rs` | 2,847 | 5-6 modules |
| `tools/src/gold_loan/tools.rs` | 1,813 | 1 file per tool + shared utils |
| `config/src/settings.rs` | 1,129 | Split by concern (server, pipeline, etc.) |
| `config/src/domain/views.rs` | 818 | Keep (view logic is cohesive) |
| `llm/src/backend.rs` | 903 | Split by provider (ollama, openai, claude) |

### 5.2 Code That Should Move

| Code | Current Location | Should Be |
|------|------------------|-----------|
| Slot-to-fact mapping | agent.rs:740-761 | `agent/src/slot_extractor.rs` or config |
| Intent-to-tool mapping | agent.rs:1453-1506 | `agent/src/tool_mapper.rs` or config |
| Tool argument building | agent.rs:1513-1611, 1661-1704 (duplicated) | `tools/src/argument_builder.rs` |
| Competitor data | tools.rs:1599-1649 | `config/domains/gold_loan/competitors.yaml` |
| Urgency keywords | lead_scoring.rs:352-365 | Domain config |
| Phone validation | tools.rs:410-412, 1183-1185 | `tools/src/validation.rs` |

### 5.3 Code Duplication

| Pattern | Occurrences | Files |
|---------|-------------|-------|
| Rate/LTV helpers | 3x | tools.rs:35-80, 197-208, 1525-1536 |
| Phone validation | 2x | tools.rs:410, 1183 |
| Registry creation | 3x | registry.rs:201, 416, 545 |
| Persona goal hardcoding | 3x | agent.rs:104, 279, 368 |
| Language-to-code mapping | 2x | candle_indictrans2.rs:1043, indictrans2.rs:64 |

---

## 6. Concurrency & Performance Issues

### 6.1 Lock Contention

| Issue | Location | Impact |
|-------|----------|--------|
| Multiple fine-grained locks in TTS hot path | tts/streaming.rs:251-293 | 3+ lock acquisitions per frame |
| Lock held during ONNX inference | tts/streaming.rs:340-347 | Blocks threads for seconds |
| Single RwLock for 10,000 sessions | session.rs:442-448 | Cleanup causes pause |
| Context vector cloned on every generation | llm/backend.rs:202, 209 | Memory pressure |

### 6.2 Memory Allocations in Hot Paths

| Issue | Location | Fix |
|-------|----------|-----|
| `toLowerCase()` per objection check | adaptation.rs:114 | Use case-insensitive search |
| Embedding cloned on cache hit | rag/cache.rs:94-112 | Return Arc<Vec<f32>> |
| Session ID list clones all IDs | session.rs:622-623 | Add limit parameter |
| Vec allocated for expired sessions | session.rs:605-611 | Use drain_filter |

### 6.3 Async Anti-Patterns

| Issue | Location | Fix |
|-------|----------|-----|
| `block_in_place()` in hot path | tts/streaming.rs:312-314 | Make truly async |
| Guard held across await | voice_session.rs:492-506 | Drop before await |
| Background task not awaited | session.rs:480-514 | Return JoinHandle |

---

## 7. Recommended Refactoring Plan

### Phase 1: Critical Domain Abstraction (High Priority)

1. **Create `DomainAgent` trait** to replace hardcoded `GoldLoanAgent`
2. **Move stage/goal definitions to YAML** using `StagesConfig`, `SlotsConfig`
3. **Extract competitor data** from code to `competitors.yaml`
4. **Parameterize persona** - remove hardcoded "Kotak", "Priya", "Gold Loan Advisor"
5. **Create `ConversationGoal` trait** loaded from config

### Phase 2: Code Organization (Medium Priority)

6. **Split `agent.rs`** into focused modules (tool_orchestrator, prompt_builder, etc.)
7. **Create unified tool factory** with builder pattern
8. **Extract shared helpers** (rate/LTV lookups, phone validation)
9. **Remove duplicate code** (persona goal, language-to-code mappings)

### Phase 3: Performance & Concurrency (Medium Priority)

10. **Group TTS state into single Mutex** instead of fine-grained locks
11. **Use DashMap** for SessionManager instead of single RwLock
12. **Cache compiled regexes** for custom abbreviations
13. **Use Arc<Vec<f32>>** for embedding cache to avoid clones

### Phase 4: Config Consolidation (Lower Priority)

14. **Eliminate config duplication** between default.yaml and domain.yaml
15. **Move abbreviations to config file** with hot-reload
16. **Make language/voice defaults configurable** via environment

---

## Appendix A: Files to Modify for Domain Agnosticism

```
# Critical files (must change for any new domain)
agent/src/agent.rs              # Remove "Kotak", "Gold Loan Advisor"
agent/src/dst/slots.rs          # Make ConversationGoal configurable
agent/src/persuasion.rs         # Load competitor data from config
core/src/customer.rs            # Extract CustomerSegment to config
core/src/domain_context.rs      # Remove gold_loan() method
tools/src/gold_loan/tools.rs    # Move competitor data to config

# Configuration files to keep/extend
config/domains/gold_loan/domain.yaml
config/domains/gold_loan/slots.yaml
config/domains/gold_loan/stages.yaml
config/domains/gold_loan/competitors.yaml
config/domains/gold_loan/scoring.yaml
```

---

## Appendix B: Proposed Config-Only Domain Onboarding

```yaml
# config/domains/car_loan/domain.yaml
domain_id: car_loan
display_name: "Kotak Car Loan"
brand:
  bank_name: "Kotak Mahindra Bank"
  agent_name: "Rahul"
  helpline: "1800-266-2666"

constants:
  interest_rates:
    tiers:
      - max_amount: 500000
        rate: 9.5
      - max_amount: 2500000
        rate: 8.5
  ltv_percent: 85.0
  loan_limits:
    min: 100000
    max: 50000000

competitors:
  hdfc_car:
    display_name: "HDFC Car Loan"
    typical_rate: 9.75
# ... etc
```

With this config-driven approach, onboarding a new domain (car loan, home loan, personal loan) would require:
1. Creating `config/domains/{domain_id}/` directory
2. Defining YAML configs (domain.yaml, slots.yaml, stages.yaml, etc.)
3. No code changes required

---

*Generated: 2026-01-06*
*Analysis by: Claude Code with 8 parallel exploration agents*
