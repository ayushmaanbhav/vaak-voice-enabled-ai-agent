# Domain-Agnostic Voice Agent: Comprehensive Analysis & Refactoring Plan

**Date**: 2026-01-10
**Status**: Ready for Implementation
**Target**: 100% Config-Driven Architecture

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current Architecture Analysis](#current-architecture-analysis)
3. [Findings by Component](#findings-by-component)
4. [Critical Issues](#critical-issues)
5. [Refactoring Plan](#refactoring-plan)
6. [Implementation Details](#implementation-details)
7. [Deprecated Code Inventory](#deprecated-code-inventory)
8. [Config File Inventory](#config-file-inventory)
9. [Verification Strategy](#verification-strategy)

---

## Executive Summary

### Current State
The voice agent backend is **~92% domain-agnostic** with excellent architectural foundations:
- 24 YAML config files for domain-specific data
- Well-structured trait system (`DomainCalculator`, `ToolFactory`, `Retriever`, etc.)
- Config-driven entity types, slots, intents, stages, compliance, scoring

### Remaining Issues (8%)
1. **ToolRegistry** hardcodes tool instantiation instead of using `ToolFactory`
2. **ScoreBreakdown** has fixed 4 categories (urgency, engagement, information, intent)
3. **CompetitorType** enum limited to 3 variants (Bank, Nbfc, Informal)
4. **Deprecated code** and legacy adapters still present
5. **Some configs** exist but aren't properly wired

### Goal
Enable new business domains to be onboarded purely through YAML configuration without any Rust code changes.

---

## Current Architecture Analysis

### Trait System Status

| Trait | Generic | Config-Driven | Status |
|-------|---------|---------------|--------|
| `DomainCalculator` | YES | YES | EXCELLENT |
| `ToolFactory` | YES | YES | EXCELLENT (but not used by registry) |
| `Retriever` | YES | N/A | EXCELLENT |
| `SlotSchema` | YES | YES | EXCELLENT |
| `FeatureProvider` | YES | YES | EXCELLENT |
| `ObjectionProvider` | YES | YES | EXCELLENT |
| `EntityTypeProvider` | YES | YES | EXCELLENT |
| `SignalProvider` | YES | YES | EXCELLENT |
| `ToolArgumentProvider` | YES | YES | EXCELLENT |
| `LeadScoringStrategy` | PARTIAL | NO | NEEDS REDESIGN |
| `CompetitorAnalyzer` | PARTIAL | NO | NEEDS REDESIGN |

### Config Coverage by Component

| Component | YAML Config | Rust Loader | View Access | Fully Wired |
|-----------|-------------|-------------|-------------|-------------|
| Entity Types | YES | YES | YES | YES |
| Slots | YES | YES | YES | YES |
| Intents | YES | YES | YES | YES |
| Tool Schemas | YES | YES | YES | PARTIAL |
| Tool Logic | NO | N/A | N/A | NO |
| Competitors | YES | YES | YES | YES |
| Prompts | YES | YES | YES | PARTIAL |
| Stages | YES | YES | YES | YES |
| Compliance | YES | YES | YES | YES |
| Lead Scoring | YES | YES | YES | PARTIAL |
| Segments | YES | YES | YES | PARTIAL |
| RAG Strategy | NO | NO | NO | NO |

---

## Findings by Component

### 1. Tool Registry (`crates/tools/src/registry.rs`)

**Problem**: Tools are instantiated directly in code, not via factory.

```rust
// CURRENT (lines 218-231) - HARDCODED
pub fn create_registry_with_view(view: Arc<ToolsDomainView>) -> ToolRegistry {
    registry.register(EligibilityCheckTool::new(view.clone()));
    registry.register(SavingsCalculatorTool::new(view.clone()));
    registry.register(GetGoldPriceTool::new(view.clone()));
    // ... 10 tools hardcoded
}
```

**Impact**: Cannot add new tools via config only.

### 2. Score Breakdown (`crates/core/src/traits/scoring.rs`)

**Problem**: Fixed struct fields limit scoring flexibility.

```rust
// CURRENT (lines 121-133) - HARDCODED CATEGORIES
pub struct ScoreBreakdown {
    pub urgency: u32,      // Fixed category 1
    pub engagement: u32,   // Fixed category 2
    pub information: u32,  // Fixed category 3
    pub intent: u32,       // Fixed category 4
    pub penalty: i32,
}
```

**Impact**: Different domains may need different scoring dimensions.

### 3. Competitor Type (`crates/core/src/traits/competitors.rs`)

**Problem**: Enum limits competitor types to 3 options.

```rust
// CURRENT (lines 25-33) - LIMITED ENUM
pub enum CompetitorType {
    Bank,      // Only 3 options
    Nbfc,
    Informal,
}
```

**Impact**: Cannot add "cooperative", "fintech", "government" types via config.

### 4. Text Processing Entities (`crates/text_processing/src/entities/mod.rs`)

**Status**: GOOD - Production code is domain-agnostic.

Deprecated methods exist but are properly marked:
- `gold_weight()` -> `collateral_weight`
- `gold_purity()` -> `collateral_quality`
- `current_lender()` -> `current_provider`
- `with_lenders()` -> `with_providers()`

### 5. Domain Calculator (`crates/core/src/traits/calculator.rs`)

**Status**: EXCELLENT - Fully generic with config-driven implementation.

All calculations use:
- `calculate_emi()` - standard amortization formula
- `calculate_asset_value()` - quantity * price * quality_factor
- `get_rate_for_amount()` - tiered rate lookup from config
- `get_quality_factor()` - from config quality tiers

---

## Critical Issues

### Issue 1: ToolRegistry Not Using ToolFactory

**Location**: `crates/tools/src/registry.rs:218-231`

**Current Flow**:
```
Config (tools/schemas.yaml) -> ToolsDomainView -> create_registry_with_view()
                                                        |
                                                        v
                                              [Hardcoded tool instantiation]
```

**Required Flow**:
```
Config (tools/schemas.yaml) -> DomainToolFactory -> create_registry_from_factory()
                                      |
                                      v
                             [Dynamic tool creation based on config]
```

### Issue 2: Hardcoded Score Categories

**Location**: `crates/core/src/traits/scoring.rs:121-133`

**Current**: 4 fixed categories with fixed weights
**Required**: Dynamic categories from `lead_scoring.yaml`

### Issue 3: Fixed CompetitorType Enum

**Location**: `crates/core/src/traits/competitors.rs:25-33`

**Current**: 3-variant enum
**Required**: String IDs from `entity_types.yaml` competitor_types section

---

## Refactoring Plan

### Phase 1: Tool Factory Implementation (HIGH PRIORITY)

#### 1.1 Create DomainToolFactory

**New File**: `crates/tools/src/factory.rs`

```rust
pub struct DomainToolFactory {
    view: Arc<ToolsDomainView>,
    calculator: Arc<dyn DomainCalculator>,
    integrations: ToolIntegrations,
}

impl ToolFactory for DomainToolFactory {
    fn create_tool(&self, name: &str) -> Result<Arc<dyn Tool>, ToolFactoryError> {
        let tool_config = self.view.get_tool(name)?;
        match tool_config.execution_type() {
            "calculation" => self.create_calculation_tool(tool_config),
            "lookup" => self.create_lookup_tool(tool_config),
            "integration" => self.create_integration_tool(tool_config),
            _ => self.create_generic_tool(tool_config),
        }
    }
}
```

#### 1.2 Extend Tool Config Schema

**File**: `config/domains/gold_loan/tools/schemas.yaml`

Add execution metadata to each tool:

```yaml
tools:
  check_eligibility:
    name: check_eligibility
    description: "Check loan eligibility..."
    execution:
      type: "calculation"
      calculator_method: "check_eligibility"
    parameters:
      - name: collateral_weight
        type: number
        aliases: ["gold_weight_grams", "asset_quantity"]
```

#### 1.3 Create Calculation Config

**New File**: `config/domains/gold_loan/calculations.yaml`

```yaml
calculations:
  check_eligibility:
    inputs:
      - name: quantity
        source: "param:collateral_weight"
      - name: quality
        source: "param:collateral_variant"
    steps:
      - name: collateral_value
        formula: "quantity * asset_price_per_unit * quality_factor(quality)"
      - name: max_loan
        formula: "collateral_value * ltv_percent / 100"
    outputs:
      eligible: "max_loan >= min_loan_amount"
      max_loan_amount_inr: "round(max_loan)"
```

#### 1.4 Update Registry

**File**: `crates/tools/src/registry.rs`

```rust
// NEW
pub fn create_registry_from_factory(
    factory: Arc<dyn ToolFactory>,
) -> Result<ToolRegistry, ToolFactoryError> {
    let mut registry = ToolRegistry::new();
    for tool in factory.create_all_tools()? {
        registry.register_boxed(tool);
    }
    Ok(registry)
}

// DEPRECATED
#[deprecated(since = "0.25.0", note = "Use create_registry_from_factory")]
pub fn create_registry_with_view(view: Arc<ToolsDomainView>) -> ToolRegistry {
    // Legacy implementation
}
```

### Phase 2: Dynamic Score Categories (HIGH PRIORITY)

#### 2.1 Update Scoring Structs

**File**: `crates/core/src/traits/scoring.rs`

```rust
// NEW: Dynamic categories
pub struct DynamicScoreBreakdown {
    pub categories: HashMap<String, u32>,
    pub penalty: i32,
    category_max: HashMap<String, u32>,
}

impl DynamicScoreBreakdown {
    pub fn total(&self, weights: &HashMap<String, f32>) -> u32 {
        let weighted: f32 = self.categories.iter()
            .map(|(k, v)| *v as f32 * weights.get(k).unwrap_or(&1.0))
            .sum();
        (weighted as i32 + self.penalty).max(0).min(100) as u32
    }

    // Legacy accessors for backward compatibility
    pub fn urgency(&self) -> u32 { self.get("urgency") }
    pub fn engagement(&self) -> u32 { self.get("engagement") }
}
```

#### 2.2 Update Lead Scoring Config

**File**: `config/domains/gold_loan/lead_scoring.yaml`

```yaml
score_categories:
  urgency:
    display_name: "Urgency"
    max_score: 25
    weight: 1.0
  engagement:
    display_name: "Engagement"
    max_score: 25
    weight: 1.0
  information:
    display_name: "Information"
    max_score: 25
    weight: 1.0
  intent:
    display_name: "Intent"
    max_score: 25
    weight: 1.0

category_signal_mappings:
  urgency:
    - signal: has_urgency_signal
      points: 10
    - signal: urgency_keywords_count
      points_per_unit: 5
      max_units: 3
  engagement:
    - signal: engagement_turns
      points_per_unit: 3
      max_units: 5
```

### Phase 3: Config-Driven CompetitorType (MEDIUM PRIORITY)

#### 3.1 Deprecate Enum

**File**: `crates/core/src/traits/competitors.rs`

```rust
#[deprecated(since = "0.2.0", note = "Use EntityTypeProvider")]
pub enum CompetitorType {
    Bank,
    Nbfc,
    Informal,
}

// NEW: String-based type
pub struct CompetitorInfo {
    pub id: String,
    pub display_name: String,
    pub competitor_type_id: String,  // "bank", "nbfc", "cooperative"
    // ...
}
```

#### 3.2 Wire EntityTypeProvider

Config already exists in `entity_types.yaml`:

```yaml
competitor_types:
  bank:
    display_name: "Bank"
    default_values:
      rate: 11.0
  nbfc:
    display_name: "NBFC"
    default_values:
      rate: 18.0
  cooperative:
    display_name: "Cooperative Bank"
    default_values:
      rate: 14.0
```

### Phase 4: Deprecated Code Removal (MEDIUM PRIORITY)

See [Deprecated Code Inventory](#deprecated-code-inventory) below.

### Phase 5: Config Consolidation (LOW PRIORITY)

1. Merge `extraction_patterns.yaml` into `slots.yaml`
2. Wire unused configs (`signals.yaml`, `goals.yaml`)
3. Move hardcoded fallbacks to config

---

## Implementation Details

### New Files to Create

| File | Purpose |
|------|---------|
| `crates/tools/src/factory.rs` | DomainToolFactory implementation |
| `crates/tools/src/calculation_context.rs` | Formula evaluation engine |
| `config/domains/gold_loan/calculations.yaml` | Calculation definitions |

### Files to Modify

| File | Changes |
|------|---------|
| `crates/tools/src/registry.rs` | Add factory-based creation |
| `crates/core/src/traits/scoring.rs` | Add DynamicScoreBreakdown |
| `crates/core/src/traits/competitors.rs` | Deprecate enum |
| `crates/config/src/domain/scoring.rs` | Add category config structs |
| `crates/agent/src/lead_scoring.rs` | Use dynamic categories |
| `config/domains/gold_loan/tools/schemas.yaml` | Add execution types |
| `config/domains/gold_loan/lead_scoring.yaml` | Add category definitions |

---

## Deprecated Code Inventory

### Methods to Remove

| File | Method | Replacement |
|------|--------|-------------|
| `persistence/src/gold_price.rs` | `price_24k()` | `price_for_tier("24K")` |
| `persistence/src/gold_price.rs` | `price_22k()` | `price_for_tier("22K")` |
| `persistence/src/gold_price.rs` | `price_18k()` | `price_for_tier("18K")` |
| `persistence/src/gold_price.rs` | `price_per_gram()` | `base_price_per_unit()` |
| `persistence/src/gold_price.rs` | `new_gold()` | `new()` |
| `text_processing/src/entities/mod.rs` | `gold_weight()` | `collateral_weight` field |
| `text_processing/src/entities/mod.rs` | `gold_purity()` | `collateral_quality` field |
| `text_processing/src/entities/mod.rs` | `current_lender()` | `current_provider` field |
| `text_processing/src/entities/mod.rs` | `with_lenders()` | `with_providers()` |
| `text_processing/src/entities/mod.rs` | `add_lenders()` | `add_providers()` |
| `text_processing/src/entities/mod.rs` | `extract_purity()` | `extract_quality_tier()` |
| `text_processing/src/entities/mod.rs` | `extract_lender()` | `extract_provider()` |
| `agent/src/memory/compressor.rs` | `with_defaults()` | `from_view()` |
| `agent/src/agent/mod.rs` | `new_with_defaults()` | `new()` |
| `config/src/domain/branches.rs` | `legacy_service_branches()` | `service_locations()` |
| `config/src/domain/views.rs` | `calculate_gold_value()` | `calculate_asset_value()` |

### Traits to Remove

| File | Trait | Replacement |
|------|-------|-------------|
| `core/src/traits/entity_types.rs` | `LegacyCompetitorTypeAdapter` | `EntityTypeProvider` |
| `core/src/traits/entity_types.rs` | `LegacySegmentAdapter` | `EntityTypeProvider` |
| `core/src/traits/signals.rs` | `LegacySignalAdapter` | `SignalProvider` |

### Type Aliases to Remove

| File | Alias | Replacement |
|------|-------|-------------|
| `persistence/src/gold_price.rs` | `GoldPrice` | `AssetPrice` |
| `persistence/src/gold_price.rs` | `GoldPriceService` | `AssetPriceService` |
| `persistence/src/gold_price.rs` | `SimulatedGoldPriceService` | `SimulatedAssetPriceService` |

### Serde Aliases to Remove

| File | Alias | Field |
|------|-------|-------|
| `config/src/domain/master.rs` | `bank_name` | `company_name` |
| `text_processing/src/entities/mod.rs` | `gold_weight` | `collateral_weight` |
| `text_processing/src/entities/mod.rs` | `gold_purity` | `collateral_quality` |
| `text_processing/src/entities/mod.rs` | `current_lender` | `current_provider` |

### Modules to Evaluate for Removal

| File | Reason |
|------|--------|
| `agent/src/memory_legacy.rs` | Check if `memory/mod.rs` covers all use cases |

---

## Config File Inventory

### Existing (24 files)

| File | Purpose | Status |
|------|---------|--------|
| `domain.yaml` | Master config | WIRED |
| `entity_types.yaml` | Entity type definitions | WIRED |
| `slots.yaml` | Slot definitions | WIRED |
| `intent_tool_mappings.yaml` | Intent to tool mappings | WIRED |
| `lead_scoring.yaml` | Lead scoring rules | WIRED |
| `compliance.yaml` | Compliance rules | WIRED |
| `prompts/system.yaml` | System prompts | WIRED |
| `objections.yaml` | Objection handling | WIRED |
| `stages.yaml` | Conversation stages | WIRED |
| `segments.yaml` | Customer segments | WIRED |
| `features.yaml` | Feature definitions | WIRED |
| `competitors.yaml` | Competitor data | WIRED |
| `adaptation.yaml` | Personalization | PARTIAL |
| `extraction_patterns.yaml` | Entity extraction | DUPLICATE |
| `vocabulary.yaml` | Domain vocabulary | WIRED |
| `signals.yaml` | Lead signals | CHECK WIRING |
| `goals.yaml` | Conversation goals | CHECK WIRING |
| `tools/schemas.yaml` | Tool definitions | WIRED |
| `tools/branches.yaml` | Branch locations | WIRED |
| `tools/documents.yaml` | Document lists | WIRED |
| `tools/responses.yaml` | Response templates | NOT WIRED |
| `tools/sms_templates.yaml` | SMS templates | WIRED |

### To Create

| File | Purpose |
|------|---------|
| `calculations.yaml` | Calculation formulas for tools |

### To Consolidate

| Source | Target | Action |
|--------|--------|--------|
| `extraction_patterns.yaml` asset quality | `slots.yaml` | Merge |
| `extraction_patterns.yaml` loan purposes | `slots.yaml` | Merge |
| `extraction_patterns.yaml` unit conversions | `slots.yaml` | Merge |

---

## Verification Strategy

### Build Verification
```bash
cd voice-agent/backend
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
```

### Runtime Verification
1. Start server with gold_loan domain
2. Verify tool registry loads from factory
3. Verify lead scoring uses dynamic categories
4. Verify competitor types from config

### Domain Agnostic Test
1. Create minimal `test_domain/` config folder
2. Define only essential YAML configs
3. Start agent with test domain
4. Verify basic conversation flow works

### Regression Testing
1. Run full test suite
2. Compare tool outputs before/after
3. Compare lead scores before/after
4. Verify no functional changes

---

## Success Criteria

1. **Zero hardcoded domain terms** in Rust production code
2. **All tools created via ToolFactory** from config
3. **Lead scoring categories** fully configurable
4. **Competitor types** defined in config, not enum
5. **No deprecated methods** or legacy adapters
6. **New domain** can be added with only YAML changes
7. **All existing tests pass** after refactoring
8. **Build succeeds** with no warnings

---

## Appendix: Code Snippets

### Current ToolRegistry (BEFORE)

```rust
// crates/tools/src/registry.rs:218-231
pub fn create_registry_with_view(view: Arc<ToolsDomainView>) -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(EligibilityCheckTool::new(view.clone()));
    registry.register(SavingsCalculatorTool::new(view.clone()));
    registry.register(GetGoldPriceTool::new(view.clone()));
    registry.register(CompetitorComparisonTool::new(view.clone()));
    registry.register(BranchLocatorTool::new());
    registry.register(DocumentChecklistTool::new(view.clone()));
    registry.register(AppointmentSchedulerTool::new(view.clone()));
    registry.register(SendSmsTool::new(view.clone()));
    registry.register(LeadCaptureTool::new());
    registry.register(EscalateToHumanTool::new());
    registry
}
```

### Factory-Based Registry (AFTER)

```rust
// crates/tools/src/registry.rs
pub fn create_registry_from_factory(
    factory: Arc<dyn ToolFactory>,
) -> Result<ToolRegistry, ToolFactoryError> {
    let mut registry = ToolRegistry::new();
    for tool in factory.create_all_tools()? {
        registry.register_boxed(tool);
    }
    tracing::info!(
        domain = factory.domain_name(),
        tool_count = registry.len(),
        "Created tool registry from factory"
    );
    Ok(registry)
}
```

### DomainToolFactory

```rust
// crates/tools/src/factory.rs
impl ToolFactory for DomainToolFactory {
    fn create_tool(&self, name: &str) -> Result<Arc<dyn Tool>, ToolFactoryError> {
        let config = self.view.get_tool(name)?;
        match config.execution_type() {
            "calculation" => Ok(Arc::new(ConfigDrivenCalculationTool::new(
                name.to_string(),
                self.view.clone(),
                self.calculator.clone(),
            ))),
            "lookup" => self.create_lookup_tool(config),
            "integration" => self.create_integration_tool(config),
            _ => self.create_generic_tool(config),
        }
    }
}
```

---

*Document generated: 2026-01-10*
