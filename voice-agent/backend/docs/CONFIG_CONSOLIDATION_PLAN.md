# Config Consolidation Plan: Eliminating Duplication

**Date:** 2026-01-06
**Status:** Analysis Complete, Implementation Pending
**Builds On:** `UNIFIED_REFACTORING_PLAN.md` (P1-P5 completed structurally, not wired)

---

## Executive Summary

The P1-P5 phases created YAML configs and Rust loader types but **did not wire them into the application**. We now have **two parallel config systems** that don't communicate:

| System | Status | Actually Used? |
|--------|--------|----------------|
| Legacy: `DomainConfigManager` | Working | Yes - AppState, tools, agent |
| New: `MasterDomainConfig` | Loaded | No - dropped after startup log |

**Result:** Duplication everywhere, hardcoded values still in use.

---

## Current State: Two Parallel Worlds

### System 1: Legacy (DomainConfigManager) - ACTIVE

```
crates/config/src/
├── domain_config.rs     → DomainConfigManager (wrapper)
├── gold_loan.rs         → GoldLoanConfig, TieredRates, CompetitorRates
├── branch.rs            → BranchConfig, Branch
├── competitor.rs        → CompetitorConfig, Competitor
├── prompts.rs           → PromptTemplates, SystemPromptConfig
├── product.rs           → ProductConfig, ProductVariant
└── constants.rs         → Hardcoded rates: 11.5%, 10.5%, 9.5%, 18.0%, 19.0%
```

**Used by:** `AppState`, `GoldLoanAgent`, tools, server handlers

### System 2: New (MasterDomainConfig) - ORPHANED

```
crates/config/src/domain/
├── master.rs            → MasterDomainConfig (loads YAML)
├── views.rs             → AgentDomainView, LlmDomainView, ToolsDomainView
├── slots.rs             → SlotsConfig, SlotDefinition
├── stages.rs            → StagesConfig, StageDefinition
├── scoring.rs           → ScoringConfig
├── tools.rs             → ToolsConfig, ToolSchema
├── prompts.rs           → PromptsConfig (different from prompts.rs!)
├── objections.rs        → ObjectionsConfig
├── branches.rs          → BranchesConfig, BranchEntry
├── competitors.rs       → CompetitorsConfig, CompetitorEntry
├── segments.rs          → SegmentsConfig
└── sms_templates.rs     → SmsTemplatesConfig

config/domains/gold_loan/
├── domain.yaml          → Brand, constants, competitors
├── slots.yaml           → DST slot definitions
├── stages.yaml          → Conversation flow
├── scoring.yaml         → Lead scoring weights
├── objections.yaml      → 9 objection types with responses
├── competitors.yaml     → Extended competitor data
├── segments.yaml        → Customer segmentation
└── tools/
    ├── schemas.yaml     → Tool JSON schemas
    ├── branches.yaml    → 20 branch locations
    └── sms_templates.yaml → SMS templates
```

**Used by:** Nothing (loaded in main.rs, immediately dropped)

---

## Duplication Map

### 1. Competitor Data (5 locations!)

| Location | Data | Struct |
|----------|------|--------|
| `gold_loan.rs:105-114` | `CompetitorRates { muthoot: 18.0, manappuram: 19.0 }` | Hardcoded |
| `competitor.rs:43-200` | Full competitor profiles with aliases | `Competitor` |
| `domain/competitors.rs` | YAML-loaded competitor profiles | `CompetitorEntry` |
| `domain/master.rs:78` | Inline competitor struct | `CompetitorEntry` |
| `domain.yaml:70-108` | YAML source of truth | N/A |

**All have the same rates!** But 4 different code paths.

### 2. Branch Data (3 locations)

| Location | Data | Struct |
|----------|------|--------|
| `branch.rs:1-300` | Hardcoded Mumbai, Delhi, etc. | `Branch`, `BranchConfig` |
| `domain/branches.rs` | YAML-loaded branches | `BranchEntry`, `BranchesConfig` |
| `tools/branches.yaml` | YAML source of truth (20 branches) | N/A |

### 3. Interest Rate Logic (4 locations)

| Location | Method | Logic |
|----------|--------|-------|
| `gold_loan.rs:296` | `get_tiered_rate(amount)` | Tier lookup |
| `domain_config.rs:436` | `get_interest_rate(amount)` | Wraps gold_loan |
| `master.rs:475` | `get_rate_for_amount(amount)` | Tier lookup from YAML |
| `views.rs:58,478` | `our_rate_for_amount(amount)` | Wraps master |

**Same algorithm, 4 implementations!**

### 4. Prompt/Greeting Logic (3 locations)

| Location | Method | Logic |
|----------|--------|-------|
| `ptt.rs:747` | `get_greeting(language)` | Hardcoded strings |
| `prompts.rs:668` | `get_greeting(hour, agent, name)` | Time-based template |
| `domain/prompts.rs` | `PromptsConfig` | YAML templates with placeholders |

### 5. Savings Calculation (1 location but fragmented)

| Location | Method | Dependencies |
|----------|--------|--------------|
| `competitor.rs:449` | `calculate_savings()` | Uses Kotak rate from GoldLoanConfig |
| `domain_config.rs:441` | `calculate_competitor_savings()` | Wraps competitor.rs |

**Problem:** No equivalent in new system - needs to be added to `ToolsDomainView`.

---

## Method Ownership Analysis

### Where Each Method Should Live (per UNIFIED_REFACTORING_PLAN.md)

| Method | Current Owner | Target Owner | Reason |
|--------|---------------|--------------|--------|
| `calculate_competitor_savings()` | `DomainConfigManager` | `ToolsDomainView` | Tools crate owns comparison logic |
| `find_branches_by_city()` | `DomainConfigManager` + `ToolsDomainView` | `ToolsDomainView` only | Remove from DomainConfigManager |
| `doorstep_available()` | `BranchConfig` + `DomainConfigManager` | `ToolsDomainView` | Tools crate owns branch operations |
| `get_system_prompt()` | `DomainConfigManager` | `LlmDomainView` | LLM crate owns prompts |
| `get_greeting()` | `ptt.rs` + `PromptTemplates` | `LlmDomainView` | LLM crate owns all prompts |
| `get_interest_rate()` | `DomainConfigManager` | `AgentDomainView.our_rate_for_amount()` | Agent needs rates for scoring |
| `get_competitor()` | `CompetitorConfig` | `ToolsDomainView.get_competitor()` | Tools crate owns competitor data |

---

## The Gap: What P5 Didn't Do

### main.rs Current State (P5)

```rust
// P5 loads config but doesn't wire it
let master_domain_config = load_master_domain_config("config");
tracing::info!("Loaded hierarchical domain configuration");  // Then dropped!

// AppState still uses legacy DomainConfigManager
AppState::with_domain_config(config.clone(), domain_config)
```

### main.rs Target State (P6)

```rust
// Load new config
let master_config = Arc::new(MasterDomainConfig::load(&domain_id, &config_dir)?);

// Create views for each crate
let agent_view = Arc::new(AgentDomainView::new(Arc::clone(&master_config)));
let llm_view = Arc::new(LlmDomainView::new(Arc::clone(&master_config)));
let tools_view = Arc::new(ToolsDomainView::new(Arc::clone(&master_config)));

// Wire into AppState (replace DomainConfigManager)
AppState::new(config)
    .with_master_domain_config(master_config)
    .with_agent_view(agent_view)
    .with_llm_view(llm_view)
    .with_tools_view(tools_view)
```

---

## Consolidation Strategy

### Option A: Replace Legacy with New (Clean but Breaking)

**Steps:**
1. Add missing methods to views (`calculate_competitor_savings`, etc.)
2. Replace `DomainConfigManager` in `AppState` with `MasterDomainConfig`
3. Update all callers to use views
4. Delete legacy config files

**Pros:** Clean codebase, single source of truth
**Cons:** Breaking changes, many files to update

### Option B: Bridge Pattern (Safe but Temporary)

**Steps:**
1. Make `DomainConfigManager` delegate to `MasterDomainConfig` internally
2. Keep API unchanged, switch implementation
3. Deprecate `DomainConfigManager` methods
4. Gradually migrate callers to views

**Pros:** No breaking changes, gradual migration
**Cons:** Temporary duplication, longer timeline

### Recommendation: Option A (with migration path)

1. P6: Wire `MasterDomainConfig` into `AppState` (add field, don't remove old)
2. P7: Add missing methods to views
3. P8: Update agent crate to use `AgentDomainView`
4. P9: Update tools crate to use `ToolsDomainView`
5. P10: Update llm crate to use `LlmDomainView`
6. P11: Remove legacy `DomainConfigManager`

---

## Detailed P6-P11 Implementation Plan

### P6: Wire MasterDomainConfig into AppState

**Files to modify:**
- `crates/server/src/state.rs` - Add `master_domain_config: Arc<MasterDomainConfig>`
- `crates/server/src/main.rs` - Pass config to AppState

**Changes:**
```rust
// state.rs
pub struct AppState {
    // Keep legacy for now
    pub domain_config: Arc<DomainConfigManager>,
    // Add new
    pub master_domain_config: Arc<MasterDomainConfig>,
}
```

### P7: Add Missing Methods to Views

**ToolsDomainView needs:**
```rust
impl ToolsDomainView {
    /// Calculate monthly savings vs competitor
    pub fn calculate_savings(&self, competitor: &str, loan_amount: f64) -> Option<MonthlySavings> {
        let competitor = self.config.competitors_config.find_by_name(competitor)?;
        let our_rate = self.config.get_rate_for_amount(loan_amount);
        let their_rate = competitor.typical_rate;

        let monthly_diff = loan_amount * (their_rate - our_rate) / 12.0 / 100.0;
        Some(MonthlySavings { monthly: monthly_diff, annual: monthly_diff * 12.0 })
    }

    /// Check if doorstep service available
    pub fn doorstep_available(&self, city: &str) -> bool {
        self.config.branches.doorstep_available(city)
    }
}
```

**LlmDomainView needs:**
```rust
impl LlmDomainView {
    /// Get greeting for language and time
    pub fn get_greeting(&self, language: &str, hour: u32) -> String {
        self.config.prompts.response_template("greeting", language)
            .map(|t| self.interpolate(t))
            .unwrap_or_else(|| self.default_greeting(language))
    }

    /// Build full system prompt with stage guidance
    pub fn build_system_prompt(&self, stage: &str, customer_name: Option<&str>) -> String {
        let traits = self.config.prompts.build_persona_traits(0.8, 0.8, 0.5, 0.5);
        self.config.prompts.build_system_prompt(
            &self.config.brand.agent_name,
            &self.config.brand.bank_name,
            &traits,
            "en",
            "", // key_facts
            &self.config.brand.helpline,
        )
    }
}
```

### P8: Update Agent Crate

**Files to modify:**
- `crates/agent/src/agent.rs` - Accept `AgentDomainView` in constructor
- `crates/agent/src/lead_scoring.rs` - Use view for thresholds
- `crates/agent/src/persuasion.rs` - Use view for objection responses
- `crates/agent/src/dst/extractor.rs` - Use view for patterns

**Example change:**
```rust
// Before (agent.rs)
impl GoldLoanAgent {
    pub fn new(/* ... */) -> Self {
        let high_value_threshold = 500_000.0; // hardcoded
    }
}

// After
impl GoldLoanAgent {
    pub fn new(agent_view: Arc<dyn AgentDomainView>, /* ... */) -> Self {
        let high_value_threshold = agent_view.high_value_amount_threshold();
    }
}
```

### P9: Update Tools Crate

**Files to modify:**
- `crates/tools/src/gold_loan/tools.rs` - Use `ToolsDomainView` for branches, rates
- `crates/tools/src/registry.rs` - Accept view in factory

**Example:**
```rust
// Before
impl CheckEligibilityTool {
    pub fn new(config: GoldLoanConfig) -> Self { /* ... */ }
}

// After
impl CheckEligibilityTool {
    pub fn new(view: Arc<ToolsDomainView>) -> Self {
        Self {
            ltv: view.ltv_percent(),
            rates: view.interest_rate_tiers(),
        }
    }
}
```

### P10: Update LLM Crate

**Files to modify:**
- `crates/llm/src/prompt.rs` - Use `LlmDomainView` for tool schemas, prompts

**Example:**
```rust
// Before
fn gold_loan_tools() -> Vec<ToolSchema> {
    vec![/* 10 hardcoded tools */]
}

// After
impl PromptBuilder {
    pub fn tools(&self) -> &[ToolSchema] {
        self.llm_view.tool_schemas()
    }
}
```

### P11: Remove Legacy Config

**Files to delete:**
- `crates/config/src/domain_config.rs` (DomainConfigManager)
- `crates/config/src/gold_loan.rs` (hardcoded rates)
- `crates/config/src/branch.rs` (hardcoded branches)
- `crates/config/src/competitor.rs` (hardcoded competitors)

**Files to update:**
- `crates/config/src/lib.rs` - Remove exports
- `crates/server/src/state.rs` - Remove `domain_config` field

---

## Files to Delete After Migration

| File | Reason |
|------|--------|
| `config/gold_loan.rs` | Replaced by `domain.yaml` + `MasterDomainConfig` |
| `config/branch.rs` | Replaced by `tools/branches.yaml` + `BranchesConfig` |
| `config/competitor.rs` | Replaced by `competitors.yaml` + `CompetitorsConfig` |
| `config/domain_config.rs` | Replaced by `MasterDomainConfig` + views |
| `config/prompts.rs` | Replaced by `prompts/system.yaml` + `PromptsConfig` |
| `config/product.rs` | Replaced by `domain.yaml` products section |

---

## Verification Checklist

After P11, verify:

- [ ] No hardcoded rates (9.5%, 10.5%, 11.5%, 18.0%, 19.0%) in Rust files
- [ ] No hardcoded branch names in Rust files
- [ ] No hardcoded competitor names in Rust files
- [ ] All config comes from YAML files
- [ ] `DomainConfigManager` fully removed
- [ ] Views are the only config access point
- [ ] Build passes with `cargo check`
- [ ] All tests pass

---

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Breaking existing code | Add new before removing old (P6-P10 parallel) |
| Missing methods in views | P7 adds all needed methods before migration |
| Test coverage gaps | Add integration tests comparing old/new values |
| Runtime errors | Feature flag `USE_NEW_CONFIG` for gradual rollout |

---

## Appendix: Hardcoded Values to Eliminate

Search regex: `11\.5|10\.5|9\.5|18\.0|19\.0|muthoot|manappuram|Kotak.*Bank|1800-266`

Files with hardcoded values (current count: 20+):
- `crates/core/src/customer.rs`
- `crates/core/src/personalization/`
- `crates/agent/src/persuasion.rs`
- `crates/agent/src/dst/extractor.rs`
- `crates/tools/src/gold_loan/tools.rs`
- `crates/llm/src/prompt.rs`
- `crates/server/src/ptt.rs`
- `crates/rag/src/domain_boost.rs`
- `crates/text_processing/src/intent/`
- ... and more

All should read from `MasterDomainConfig` via appropriate views after migration.

---

*Generated: 2026-01-06*
