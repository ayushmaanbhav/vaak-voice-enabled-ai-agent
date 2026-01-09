# Comprehensive Backend Code Review - January 2025

> **Generated**: 2025-01-07
> **Codebase**: voice-agent/backend (202 Rust files, 92K+ lines)
> **Focus**: Domain-agnosticism, trait design, code quality, architecture

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Domain-Specific Hardcoding](#1-domain-specific-hardcoding-critical)
3. [Trait Design Analysis](#2-trait-design-analysis)
4. [Code Quality Issues](#3-code-quality-issues)
5. [Concurrency & Performance](#4-concurrency--performance)
6. [Recommended Architecture](#5-recommended-architecture)
7. [Implementation Phases](#6-implementation-phases)
8. [File Reference Index](#7-file-reference-index)

---

## Executive Summary

### Critical Issues Blocking Domain-Agnosticism

| Category | Count | Impact | Priority |
|----------|-------|--------|----------|
| Hardcoded business values | 60+ | Cannot onboard new domains | P0 |
| Brand-specific strings | 100+ | Kotak/gold-loan mentions everywhere | P0 |
| Concrete types instead of traits | 5 major | Low testability, tight coupling | P1 |
| Files >500 lines | 25+ | Maintenance nightmare | P2 |
| `unwrap()` calls | 1,012+ | Potential panics in production | P2 |
| Code duplication | 50+ patterns | Increases maintenance burden | P3 |

### Architecture Maturity Assessment

| Aspect | Score | Notes |
|--------|-------|-------|
| Core Traits (LLM, Tool, Pipeline) | 9/10 | Excellent abstraction |
| Domain-Agnostic Traits (P13 FIX) | 9/10 | Good: DomainCalculator, SlotSchema, etc. |
| Factory Pattern Usage | 6/10 | Partial: hardcoded tool registration |
| Dependency Injection | 5/10 | Optional views with fallbacks |
| Agent Trait Abstraction | 3/10 | Uses concrete types, not traits |
| File Organization | 4/10 | Many oversized monolithic files |

---

## 1. Domain-Specific Hardcoding (CRITICAL)

### 1.1 Financial Values in Code

These hardcoded values **MUST** be removed for domain-agnosticism:

| File | Line | Hardcoded Value | Current Code | Should Be |
|------|------|-----------------|--------------|-----------|
| `tools/src/gold_loan/tools.rs` | 38 | Interest rate 10.5% | `unwrap_or(10.5)` | `config.constants.interest_rates.base_rate` |
| `tools/src/gold_loan/tools.rs` | 44 | LTV 75% | `unwrap_or(75.0)` | `config.constants.ltv_percent` |
| `tools/src/gold_loan/tools.rs` | 71 | Gold price Rs.7500 | `weight * 7500.0` | `config.constants.gold_price_per_gram` |
| `tools/src/gold_loan/tools.rs` | 64-70 | Purity factors | `0.916, 0.75, 0.585` | `config.constants.purity_factors` |
| `tools/src/gold_loan/tools.rs` | 153-159 | Rate tiers | `<=100K, <=500K` | `config.constants.interest_rates.tiers` |
| `tools/src/gold_loan/tools.rs` | 200 | Interest rate | `unwrap_or(10.5)` | config |
| `tools/src/gold_loan/tools.rs` | 845-849 | Gold price | `unwrap_or(7500.0)` | config |
| `tools/src/gold_loan/tools.rs` | 1579 | Kotak rate | `unwrap_or(10.49)` | config |
| `tools/src/gold_loan/tools.rs` | 1585 | LTV | `unwrap_or(75.0)` | config |
| `config/src/constants.rs` | 16-33 | Interest rates | `11.5, 10.5, 9.5, 18.0` | DELETE - use YAML |
| `config/src/constants.rs` | 36-43 | Tier boundaries | `100K, 500K` | DELETE - use YAML |
| `config/src/constants.rs` | 47-52 | LTV values | `75.0, 70.0` | DELETE - use YAML |
| `config/src/constants.rs` | 56-72 | Gold/purity | `7500.0, 0.916...` | DELETE - use YAML |

**Anti-Pattern Identified**: Tools use `Option<Arc<ToolsDomainView>>` with fallback values:

```rust
// CURRENT (BAD)
pub struct EligibilityCheckTool {
    view: Option<Arc<ToolsDomainView>>,  // Optional!
}

fn get_rate(&self, amount: f64) -> f64 {
    self.view.as_ref()
        .map(|v| v.get_rate_for_amount(amount))
        .unwrap_or(10.5)  // HARDCODED FALLBACK
}

// REQUIRED (GOOD)
pub struct EligibilityCheckTool {
    view: Arc<ToolsDomainView>,  // Required, not optional
}

fn get_rate(&self, amount: f64) -> f64 {
    self.view.get_rate_for_amount(amount)  // No fallback
}
```

### 1.2 Brand & Company Names

All brand mentions must be templated:

| File | Line | Content | Fix |
|------|------|---------|-----|
| `agent/src/agent.rs` | 105-110 | `"Kotak Mahindra Bank as a Gold Loan Advisor"` | `{bank_name} as a {agent_role}` |
| `tools/src/gold_loan/tools.rs` | 217 | `"switching from NBFC to Kotak gold loan"` | `{bank_name} {product_name}` |
| `tools/src/gold_loan/tools.rs` | 749 | `"Kotak Mahindra Bank branches"` | `{bank_name} branches` |
| `tools/src/gold_loan/tools.rs` | 1266-1303 | SMS templates with Kotak branding | Use `sms_templates.yaml` |
| `core/src/domain_context.rs` | 95-97 | Bank vocabulary | Load from `config.vocabulary` |
| `agent/src/persuasion.rs` | 528 | `"Kotak Mahindra Bank...35 years"` | Use `config.prompts.trust_script` |
| `agent/src/persuasion.rs` | 463, 474 | `"50,000 customers...Muthoot and Manappuram"` | Use `config.prompts.social_proof` |
| `agent/src/persuasion.rs` | 484 | `"Processing fee is just 1%"` | Use `config.constants.processing_fee` |

**Total Brand Mentions**: 100+ across codebase

### 1.3 Competitor Names

| File | Lines | Competitors | Current | Should Load From |
|------|-------|-------------|---------|------------------|
| `agent/src/persuasion.rs` | 108-111 | Detection | `"muthoot", "manappuram", "iifl"` | `competitors.yaml` aliases |
| `agent/src/persuasion.rs` | 323-325 | Display names | Match statement | `competitors.yaml` display_name |
| `agent/src/persuasion.rs` | 352-386 | Comparison scripts | Hardcoded responses | `objections.yaml` or `prompts.yaml` |
| `tools/src/gold_loan/tools.rs` | 243-249 | Schema enum | `["Muthoot", "Manappuram"...]` | Dynamic from config |
| `tools/src/gold_loan/tools.rs` | 1606-1614 | Fallback rates | `Muthoot: 12.0%, HDFC: 10.5%` | REMOVE - no fallbacks |
| `tools/src/gold_loan/tools.rs` | 1660-1669 | Schema enum | Hardcoded | Dynamic from config |
| `core/src/domain_context.rs` | 140-150 | Vocabulary | Competitor names | `config.vocabulary.competitors` |

### 1.4 Product-Specific Logic

| File | Lines | Logic | Issue | Fix |
|------|-------|-------|-------|-----|
| `agent/src/dst/slots.rs` | ALL | `GoldLoanDialogueState` | Domain-specific struct | Rename to `DomainDialogueState`, use dynamic slots |
| `agent/src/dst/slots.rs` | 18-25 | Purity IDs | `K24, K22, K18, K14` | Load from config slot enums |
| `agent/src/dst/slots.rs` | 27-40 | `parse_purity_id()` | Hardcoded parsing | Use `SlotSchema.extract()` |
| `agent/src/dst/slots.rs` | 64-78 | `NextBestAction` | Gold-loan specific | Abstract to config-driven goals |
| `tools/src/gold_loan/tools.rs` | 1394-1422 | Document checklist | Aadhaar, PAN, etc. | Move to `documents.yaml` |
| `tools/src/gold_loan/tools.rs` | 1478-1521 | Loan types | balance_transfer, top_up | Move to `products.yaml` |
| `tools/src/gold_loan/utils.rs` | 16-44 | EMI calculation | Domain-specific | Use `DomainCalculator` trait |

---

## 2. Trait Design Analysis

### 2.1 Excellent Traits (Keep As-Is)

| Trait | File | Lines | Status | Notes |
|-------|------|-------|--------|-------|
| `LanguageModel` | `core/src/traits/llm.rs` | 25+ | EXCELLENT | Pluggable LLM backends (Claude, Ollama, etc.) |
| `Tool` | `core/src/traits/tool.rs` | 402+ | EXCELLENT | MCP-compatible with validation |
| `FrameProcessor` | `core/src/traits/pipeline.rs` | 284+ | GOOD | Pipeline stage composition |
| `DomainCalculator` | `core/src/traits/calculator.rs` | 96+ | EXCELLENT | Config-driven calculations |
| `LeadScoringStrategy` | `core/src/traits/scoring.rs` | 322+ | EXCELLENT | `ConfigLeadScoring` implementation |
| `SlotSchema` | `core/src/traits/slots.rs` | 160+ | EXCELLENT | Dynamic slot extraction |
| `CompetitorAnalyzer` | `core/src/traits/competitors.rs` | 317+ | EXCELLENT | Config-driven comparison |
| `ObjectionHandler` | `core/src/traits/objections.rs` | - | EXCELLENT | Config-driven responses |
| `SegmentDetector` | `core/src/traits/segments.rs` | - | EXCELLENT | Customer segmentation |
| `ConversationGoalSchema` | `core/src/traits/goals.rs` | - | EXCELLENT | Dynamic goals |

### 2.2 Missing Traits (Must Create)

| Needed Trait | Current Concrete Type | File:Line | Priority | Reason |
|--------------|----------------------|-----------|----------|--------|
| `ConversationContext` | `Arc<Conversation>` | `agent/src/agent.rs:57` | **CRITICAL** | Blocks testing, couples to impl |
| `DialogueState` | `RwLock<DialogueStateTracker>` | `agent/src/agent.rs:86` | **CRITICAL** | Can't mock DST |
| `PersuasionStrategy` | `PersuasionEngine` | `agent/src/agent.rs:79` | HIGH | Different domains need different persuasion |
| `ConversationMemory` | Concrete in `memory/mod.rs` | `agent/src/memory/` | HIGH | Memory strategy should be pluggable |
| `SpeculativeStrategy` | `SpeculativeExecutor` | `agent/src/agent.rs:82` | MEDIUM | Alternative speculation algorithms |
| `ToolRegistry` trait | Concrete `ToolRegistry` | `tools/src/registry.rs` | MEDIUM | Plugin discovery |

**Proposed Trait Definitions**:

```rust
// conversation.rs
pub trait ConversationContext: Send + Sync {
    fn add_turn(&mut self, role: TurnRole, text: String);
    fn get_history(&self) -> &[ConversationTurn];
    fn current_stage(&self) -> &str;
    fn stage_metadata(&self) -> Option<&serde_json::Value>;
}

// dst/mod.rs
pub trait DialogueState: Send + Sync {
    fn extract_slot(&mut self, name: &str, value: String, confidence: f32);
    fn get_slot(&self, name: &str) -> Option<&SlotValue>;
    fn has_required_slots(&self, goal: &str) -> bool;
    fn to_context_string(&self) -> String;
}

// persuasion.rs
pub trait PersuasionStrategy: Send + Sync {
    fn detect_objection(&self, text: &str) -> Option<ObjectionMatch>;
    fn generate_response(&self, objection: &ObjectionMatch) -> String;
    fn get_competitor_comparison(&self, competitor: &str) -> Option<String>;
}
```

### 2.3 Factory Pattern Issues

| Issue | Location | Impact | Fix |
|-------|----------|--------|-----|
| Hardcoded tool registration | `tools/src/registry.rs:201-249` | Can't add tools via config | Implement `ToolProvider` trait |
| Manual dependency wiring | `agent/src/agent.rs:94-200` | 150+ lines of constructor | Use builder or DI container |
| Optional view injection | `tools/src/gold_loan/tools.rs:22-34` | Fallback hardcoding | Make view required |

**Current Tool Registration (BAD)**:
```rust
// registry.rs:201-221
pub fn create_default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(EligibilityCheckTool::new());  // Hardcoded
    registry.register(SavingsCalculatorTool::new()); // Hardcoded
    // ... 8 more hardcoded registrations
    registry
}
```

**Proposed Plugin System (GOOD)**:
```rust
pub trait ToolProvider: Send + Sync {
    fn tool_ids(&self) -> Vec<&str>;
    fn create_tool(&self, id: &str, view: Arc<ToolsDomainView>) -> Option<Arc<dyn Tool>>;
}

pub fn create_registry_from_config(
    config: &MasterDomainConfig,
    providers: &[Arc<dyn ToolProvider>]
) -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    let view = Arc::new(ToolsDomainView::new(&config));

    for tool_config in &config.tools.enabled {
        for provider in providers {
            if let Some(tool) = provider.create_tool(&tool_config.name, view.clone()) {
                registry.register(tool);
                break;
            }
        }
    }
    registry
}
```

---

## 3. Code Quality Issues

### 3.1 Single Responsibility Violations

#### GoldLoanAgent (agent/src/agent.rs)

**Current State**: 2,727 lines with 13 distinct responsibilities

| Responsibility | Lines | Should Be |
|----------------|-------|-----------|
| Configuration | 54-92 | `AgentConfig` (exists) |
| Conversation management | 57 | `Arc<dyn ConversationContext>` |
| Tool orchestration | 58 | `Arc<dyn ToolExecutor>` |
| LLM interface | 59-60 | `Arc<dyn LanguageModel>` (exists) |
| RAG retrieval | 63-65 | `Arc<dyn Retriever>` (exists) |
| Prefetch caching | 67-68 | Extract to `PrefetchCache` |
| Personalization | 70-72 | `Arc<dyn PersonalizationEngine>` |
| Translation | 75-77 | `Arc<dyn Translator>` (exists) |
| Persuasion | 79 | `Arc<dyn PersuasionStrategy>` |
| Speculative execution | 82 | `Arc<dyn SpeculativeStrategy>` |
| Dialogue state | 86 | `Arc<dyn DialogueState>` |
| Lead scoring | 89 | `Arc<dyn LeadScoringStrategy>` (exists) |
| Domain config | 91 | `Arc<AgentDomainView>` (exists) |

#### Tools File (tools/src/gold_loan/tools.rs)

**Current State**: 1,857 lines with 10 tool implementations

| Tool | Lines | Should Be |
|------|-------|-----------|
| EligibilityCheckTool | 22-180 | `tools/src/domain/eligibility.rs` |
| SavingsCalculatorTool | 183-330 | `tools/src/domain/savings.rs` |
| LeadCaptureTool | 332-498 | `tools/src/domain/lead_capture.rs` |
| AppointmentSchedulerTool | 500-731 | `tools/src/domain/appointment.rs` |
| BranchLocatorTool | 733-803 | `tools/src/domain/branch.rs` |
| GetGoldPriceTool | 805-990 | `tools/src/domain/pricing.rs` |
| EscalateToHumanTool | 992-1152 | `tools/src/domain/escalation.rs` |
| SendSmsTool | 1154-1365 | `tools/src/domain/sms.rs` |
| DocumentChecklistTool | 1367-1559 | `tools/src/domain/documents.rs` |
| CompetitorComparisonTool | 1561-1796 | `tools/src/domain/competitor.rs` |

### 3.2 Files Over 500 Lines

| File | Lines | Issue | Priority |
|------|-------|-------|----------|
| `agent/src/agent.rs` | 2,727 | 13 responsibilities | P1 |
| `tools/src/gold_loan/tools.rs` | 1,857 | 10 tools in one file | P1 |
| `agent/src/dst/extractor.rs` | 1,601 | 13 pattern builders | P2 |
| `text_processing/src/intent/mod.rs` | 1,521 | Mixed concerns | P2 |
| `server/src/ptt.rs` | 1,316 | Pipeline in one file | P2 |
| `agent/src/memory/mod.rs` | 1,179 | Multiple memory types | P3 |
| `llm/src/speculative.rs` | 1,135 | Multiple strategies | P3 |
| `config/src/settings.rs` | 1,129 | Config definitions | P3 |
| `llm/src/backend.rs` | 1,121 | Provider implementations | P3 |
| `config/src/domain/views.rs` | 975 | 50+ pass-through methods | P3 |

### 3.3 Code Duplication

#### Pattern Builder Duplication

**File**: `agent/src/dst/extractor.rs:80-200`

13 functions with identical structure - should use generic `PatternBuilder<T>`.

#### Tool Schema Boilerplate

10 tools repeat identical schema construction - should use derive macro.

### 3.4 Error Handling Issues

| File | unwrap() Count | Severity |
|------|----------------|----------|
| `agent/src/dst/extractor.rs` | 170 | CRITICAL |
| `server/src/ptt.rs` | 29 | HIGH |
| `llm/src/prompt.rs` | 21 | HIGH |
| `pipeline/src/stt/indicconformer.rs` | 11 | MEDIUM |
| Other files (90+) | 780+ | MEDIUM |
| **TOTAL** | 1,012+ | - |

---

## 4. Concurrency & Performance

### 4.1 Lock Ordering Issues

**File**: `agent/src/agent.rs:68-89`

Multiple `RwLock`s without documented ordering - potential deadlock risk.

### 4.2 Global Mutable State

| Location | Issue | Fix |
|----------|-------|-----|
| `server/src/ptt.rs:67` | `static STT_POOL` | Inject via AppState |
| `pipeline/src/stt/indicconformer.rs:444` | `static DEBUG_LOGGED` | Use tracing span |

### 4.3 Performance Hotspots

| File | Lines | Issue | Impact |
|------|-------|-------|--------|
| `agent/src/dst/extractor.rs` | 80-200 | Regex compiled per call | Latency |
| `server/src/ptt.rs` | 127-131 | No pre-allocation | Memory churn |

---

## 5. Recommended Architecture

### 5.1 Config-Driven Domain Layer

```
config/domains/{domain_id}/
├── domain.yaml           # Core: brand, constants, vocabulary
├── competitors.yaml      # Competitor definitions with aliases
├── products.yaml         # Product variants and features
├── slots.yaml           # DST slot definitions
├── goals.yaml           # Conversation goals with required slots
├── segments.yaml        # Customer segments
├── scoring.yaml         # Lead scoring config
├── objections.yaml      # Objection patterns and responses
├── documents.yaml       # Required documents (NEW)
├── prompts/
│   ├── system.yaml      # System prompt template
│   └── scripts.yaml     # Persuasion scripts (NEW)
└── tools/
    ├── schemas.yaml     # Tool input schemas
    ├── branches.yaml    # Location data
    └── sms_templates.yaml
```

**Key Principle**: Code reads ZERO domain values directly. ALL via config views.

### 5.2 Trait-Based Agent

```rust
pub trait DomainAgent: Send + Sync {
    fn domain(&self) -> &DomainBridge;
    fn conversation(&self) -> Arc<dyn ConversationContext>;
    fn tools(&self) -> Arc<dyn ToolRegistry>;
    async fn process_turn(&self, input: &str) -> Result<AgentResponse>;
}
```

---

## 6. Implementation Phases

### Phase 1: Remove Hardcoding (P0)
- Remove all `unwrap_or(hardcoded_value)` patterns
- Make `ToolsDomainView` REQUIRED
- Delete `config/src/constants.rs`
- Template all brand names

### Phase 2: Abstract Concrete Types (P1)
- Create `ConversationContext` trait
- Create `DialogueState` trait
- Create `PersuasionStrategy` trait
- Rename `GoldLoanAgent` to `DomainAgent`

### Phase 3: Split Large Files (P2)
- Split `tools.rs` into 10 modules
- Split `agent.rs` into sub-modules
- Extract pattern builders

### Phase 4: Code Quality (P3)
- Convert Regex to `lazy_static!`
- Document lock ordering
- Replace `unwrap()` with proper errors

---

## 7. File Reference Index

### Critical Files

| File | Lines | Issues |
|------|-------|--------|
| `crates/tools/src/gold_loan/tools.rs` | 1,857 | 20+ hardcoded values |
| `crates/agent/src/agent.rs` | 2,727 | 13 responsibilities |
| `crates/agent/src/dst/slots.rs` | 1,207 | Domain-specific struct |
| `crates/agent/src/persuasion.rs` | ~600 | Hardcoded scripts |
| `crates/config/src/constants.rs` | ~80 | DELETE entire file |

### Good Reference Files

| File | Pattern |
|------|---------|
| `crates/core/src/traits/calculator.rs` | Config-driven trait |
| `crates/config/src/domain/bridge.rs` | DomainBridge pattern |
| `crates/config/src/domain/views.rs` | View separation |

---

## Acceptance Criteria for Domain-Agnosticism

- [ ] All hardcoded fallback values removed
- [ ] `ToolsDomainView` is required, not optional
- [ ] `GoldLoanAgent` renamed to `DomainAgent` with traits
- [ ] All brand/competitor mentions from config
- [ ] New domain onboardable via `config/domains/{domain}/` only
- [ ] Zero code changes for new domain

---

*Generated by comprehensive code review analysis - 2025-01-07*
