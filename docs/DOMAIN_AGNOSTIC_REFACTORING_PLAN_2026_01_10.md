# Domain-Agnostic Architecture Refactoring Plan

**Date:** 2026-01-10
**Status:** Planning Complete - Awaiting Implementation Approval

---

## Executive Summary

This document provides a comprehensive analysis and refactoring plan to achieve **true domain-agnosticism** in the voice-agent backend. The goal is to enable onboarding new businesses/use-cases by only defining YAML configs, with zero code changes required.

### Current Assessment Score: **7.5/10 Domain-Agnostic**

---

## Part 1: Audit Results

### 1.1 What's Already Well-Architected

| Component | Status | Details |
|-----------|--------|---------|
| Rust Code | **Excellent** | Zero hardcoded domain terms (gold, loan, kotak, etc.) |
| Business Calculations | **Excellent** | Config-driven via `DomainCalculator` trait |
| Config Loading | **Good** | 19/23 configs properly loaded |
| Trait Abstractions | **Good** | 20+ traits for business logic |
| Variable Substitution | **Good** | `{{company_name}}`, `{{product_name}}` working |
| Slot System | **Excellent** | Generic canonical names with aliases |
| Feature System | **Excellent** | Config-driven with segment overrides |

### 1.2 Critical Gaps Identified

| Issue | Severity | Location |
|-------|----------|----------|
| 4 unused config files | **HIGH** | intents.yaml, vocabulary.yaml, entities.yaml, lead_scoring.yaml |
| Hardcoded tool metadata | **HIGH** | `crates/tools/src/domain_tools/factory.rs:81-164` |
| Factory not used in registry | **MEDIUM** | `crates/tools/src/registry.rs:212-241` |
| Dual ObjectionHandler traits | **MEDIUM** | Legacy + new coexist |
| Gold-specific field names | **LOW** | `crates/persistence/src/gold_price.rs` |

---

## Part 2: Unused Config Files Analysis

### 2.1 intents.yaml - NOT LOADED

**Location:** `config/domains/gold_loan/intents.yaml`

**Content:**
- 10+ intent definitions with examples
- Required/optional slots per intent
- Intent descriptions

**Impact:** Intent detection may use hardcoded patterns instead of config-driven definitions.

**Fix:** Load via `IntentsConfig` struct (already exists, just not wired).

### 2.2 vocabulary.yaml - NOT LOADED

**Location:** `config/domains/gold_loan/vocabulary.yaml`

**Content:**
- Domain-specific abbreviations
- Competitor abbreviations
- Domain terms with ASR boost factors
- Phonetic corrections
- Hindi number words

**Impact:** ASR boosting and phonetic corrections may not be fully config-driven.

**Fix:** Create `VocabularyConfig` struct and load in `master.rs`.

### 2.3 entities.yaml - NOT LOADED

**Location:** `config/domains/gold_loan/entities.yaml`

**Content:**
- Entity type definitions (asset_quantity, asset_quality, offer_amount, etc.)
- Categories (Asset, Financial, Provider, Customer)
- Extraction priority order
- Display formats

**Impact:** Entity extraction priority and display formatting not configurable.

**Fix:** Create `EntitiesConfig` struct and load in `master.rs`.

### 2.4 lead_scoring.yaml - NOT LOADED

**Location:** `config/domains/gold_loan/lead_scoring.yaml`

**Content:**
- MQL/SQL classification criteria
- Escalation triggers
- Intent-to-signal mappings
- Urgency keywords by language
- Signal weights

**Impact:** Lead classification rules may be hardcoded in `lead_scoring.rs`.

**Fix:** Extend `ScoringConfig` to include classification rules.

---

## Part 3: Tool System Hardcoding Analysis

### 3.1 Current State

**File:** `crates/tools/src/domain_tools/factory.rs`

Lines 81-164 contain hardcoded tool metadata:
```rust
fn available_tools(&self) -> Vec<ToolMetadata> {
    vec![
        ToolMetadata {
            name: "check_eligibility".to_string(),
            display_name: "Eligibility Check".to_string(),
            description: "Check if customer is eligible...".to_string(),
            category: "calculation".to_string(),
            requires_domain_config: true,
            requires_integrations: false,
        },
        // ... 9 more hardcoded tools
    ]
}
```

**File:** `crates/tools/src/registry.rs`

Lines 212-241 manually instantiate tools:
```rust
registry.register(EligibilityCheckTool::new(view.clone()));
registry.register(SavingsCalculatorTool::new(view.clone()));
// ... manual for each tool
```

### 3.2 Solution: Config-Driven Tools

**Step 1:** Extend `tools/schemas.yaml` with metadata section:
```yaml
tools:
  check_eligibility:
    name: check_eligibility
    description: "Check eligibility based on collateral value"
    category: "calculation"
    metadata:
      display_name: "Eligibility Check"
      icon: "calculator"
      requires_domain_config: true
      requires_integrations: false
      timeout_secs: 30
      enabled: true
      aliases: []
```

**Step 2:** Update `DomainToolFactory::available_tools()`:
```rust
fn available_tools(&self) -> Vec<ToolMetadata> {
    self.view.tools_config().tools.values()
        .filter(|t| t.enabled.unwrap_or(true))
        .map(|t| t.to_tool_metadata())
        .collect()
}
```

**Step 3:** Create factory-based registry:
```rust
pub fn create_registry_from_factory(factory: &dyn ToolFactory) -> Result<ToolRegistry, ToolFactoryError> {
    let mut registry = ToolRegistry::new();
    for meta in factory.available_tools() {
        registry.register_boxed(factory.create_tool(&meta.name)?);
    }
    Ok(registry)
}
```

---

## Part 4: Persistence Layer Naming

### 4.1 Current State

**File:** `crates/persistence/src/gold_price.rs`

```rust
pub struct AssetPrice {
    pub price_per_gram: f64,
    pub price_24k: f64,     // Gold-specific!
    pub price_22k: f64,     // Gold-specific!
    pub price_18k: f64,     // Gold-specific!
    pub source: String,
    pub updated_at: DateTime<Utc>,
}
```

### 4.2 Solution: Generic Tier Names

```rust
pub struct AssetPrice {
    pub price_per_gram: f64,

    #[serde(alias = "price_24k")]
    pub price_tier_1: f64,

    #[serde(alias = "price_22k")]
    pub price_tier_2: f64,

    #[serde(alias = "price_18k")]
    pub price_tier_3: f64,

    pub source: String,
    pub updated_at: DateTime<Utc>,
}

impl AssetPrice {
    // Legacy accessors for backward compatibility
    #[inline]
    pub fn price_24k(&self) -> f64 { self.price_tier_1 }
    #[inline]
    pub fn price_22k(&self) -> f64 { self.price_tier_2 }
    #[inline]
    pub fn price_18k(&self) -> f64 { self.price_tier_3 }
}
```

---

## Part 5: Dual Trait Issue

### 5.1 Current State

Two objection handling traits exist:
1. **Legacy:** `ObjectionHandler` in `crates/core/src/traits/objections.rs`
2. **New:** `ObjectionProvider` in `crates/core/src/traits/objection_provider.rs`

### 5.2 Solution

1. Mark `ObjectionHandler` as `#[deprecated]`
2. Migrate all consumers to `ObjectionProvider`
3. Remove legacy trait after deprecation period

---

## Part 6: Implementation Plan

### Phase 1: Config Wiring (Priority: HIGH)

| Task | File | Effort |
|------|------|--------|
| Create VocabularyConfig | `config/src/domain/vocabulary.rs` | Medium |
| Create EntitiesConfig | `config/src/domain/entities.rs` | Medium |
| Wire IntentsConfig | `config/src/domain/master.rs` | Low |
| Extend ScoringConfig | `config/src/domain/scoring.rs` | Medium |
| Add view accessors | `config/src/domain/views.rs` | Low |
| Wire to consumers | Various | Medium |

### Phase 2: Tool System (Priority: HIGH)

| Task | File | Effort |
|------|------|--------|
| Add ToolSchemaMetadata | `config/src/domain/tools.rs` | Low |
| Update schemas.yaml | `config/domains/gold_loan/tools/schemas.yaml` | Medium |
| Refactor factory | `tools/src/domain_tools/factory.rs` | Medium |
| Add factory-based registry | `tools/src/registry.rs` | Medium |

### Phase 3: Persistence Naming (Priority: MEDIUM)

| Task | File | Effort |
|------|------|--------|
| Rename AssetPrice fields | `persistence/src/gold_price.rs` | Low |
| Add serde aliases | Same | Low |
| Add legacy accessors | Same | Low |
| Update price tool | `tools/src/domain_tools/tools/price.rs` | Low |

### Phase 4: Cleanup (Priority: LOW)

| Task | File | Effort |
|------|------|--------|
| Deprecate ObjectionHandler | `core/src/traits/objections.rs` | Low |
| Deprecate manual registry | `tools/src/registry.rs` | Low |
| Update tests | Various | Medium |

---

## Part 7: File Modification Summary

### New Files to Create

| File | Purpose |
|------|---------|
| `crates/config/src/domain/vocabulary.rs` | VocabularyConfig struct and loader |
| `crates/config/src/domain/entities.rs` | EntitiesConfig struct and loader |

### Files to Modify

| File | Changes |
|------|---------|
| `crates/config/src/domain/mod.rs` | Add new module exports |
| `crates/config/src/domain/master.rs` | Add fields, loading code for 4 configs |
| `crates/config/src/domain/views.rs` | Add accessor methods |
| `crates/config/src/domain/tools.rs` | Add ToolSchemaMetadata struct |
| `crates/config/src/domain/scoring.rs` | Add ClassificationConfig |
| `crates/tools/src/domain_tools/factory.rs` | Load metadata from config |
| `crates/tools/src/registry.rs` | Add factory-based creation |
| `crates/persistence/src/gold_price.rs` | Rename fields, add aliases |
| `crates/tools/src/domain_tools/tools/price.rs` | Use new field names |
| `crates/core/src/traits/objections.rs` | Deprecate legacy trait |
| `config/domains/gold_loan/tools/schemas.yaml` | Add metadata to all tools |

---

## Part 8: Verification Plan

### Build Verification
```bash
cd voice-agent/backend
cargo build --all-features
cargo clippy --all-features -- -D warnings
```

### Test Verification
```bash
cargo test --all-features
```

### Runtime Verification
1. Start server: `DOMAIN_ID=gold_loan cargo run -p voice-agent-server`
2. Check logs for config loading (no warnings)
3. Test tool invocations via WebSocket
4. Verify price calculations return correct values

### Domain-Agnostic Verification
1. Create `config/domains/test_domain/` with all YAML files
2. Modify domain-specific values (company name, rates, etc.)
3. Start with `DOMAIN_ID=test_domain`
4. Verify system works without code changes

---

## Part 9: Success Criteria

After implementation:

- [ ] All 23 YAML config files are loaded and wired
- [ ] Zero hardcoded tool metadata in Rust code
- [ ] Tool registry created via factory pattern
- [ ] `AssetPrice` uses generic tier names (`price_tier_1`, etc.)
- [ ] Legacy `ObjectionHandler` trait deprecated
- [ ] `cargo build --all-features` succeeds
- [ ] `cargo test --all-features` passes
- [ ] New domain can be onboarded with only YAML config files

---

## Part 10: Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Breaking existing functionality | Low | High | Serde aliases, legacy accessors |
| Database compatibility | Low | Medium | Use SQL column aliases |
| Test failures | Medium | Low | Update tests incrementally |
| Config loading errors | Low | Medium | Graceful fallbacks, good error messages |

---

## Appendix: Config File to Struct Mapping

### Currently Wired (19 files)

| YAML File | Rust Struct | Status |
|-----------|-------------|--------|
| domain.yaml | MasterDomainConfig | Wired |
| slots.yaml | SlotsConfig | Wired |
| stages.yaml | StagesConfig | Wired |
| scoring.yaml | ScoringConfig | Wired |
| tools/schemas.yaml | ToolsConfig | Wired |
| intent_tool_mappings.yaml | (merged) | Wired |
| prompts/system.yaml | PromptsConfig | Wired |
| objections.yaml | ObjectionsConfig | Wired |
| tools/branches.yaml | BranchesConfig | Wired |
| tools/sms_templates.yaml | SmsTemplatesConfig | Wired |
| competitors.yaml | CompetitorsConfig | Wired |
| segments.yaml | SegmentsConfig | Wired |
| goals.yaml | GoalsConfig | Wired |
| features.yaml | FeaturesConfig | Wired |
| tools/documents.yaml | DocumentsConfig | Wired |
| tools/responses.yaml | ToolResponsesConfig | Wired |
| compliance.yaml | ComplianceConfig | Wired |
| adaptation.yaml | AdaptationConfig | Wired |
| extraction_patterns.yaml | ExtractionPatternsConfig | Wired |

### Not Wired (4 files) - TO BE FIXED

| YAML File | Rust Struct | Status |
|-----------|-------------|--------|
| intents.yaml | IntentsConfig | EXISTS, not loaded |
| vocabulary.yaml | VocabularyConfig | TO CREATE |
| entities.yaml | EntitiesConfig | TO CREATE |
| lead_scoring.yaml | (extend ScoringConfig) | TO EXTEND |
