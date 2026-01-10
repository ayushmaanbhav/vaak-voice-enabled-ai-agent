# Backend Code Review: Domain-Agnostic Architecture & Quality Fixes

**Status: IN PROGRESS**
**Last Updated: 2026-01-09**

## Changes Implemented

### Completed (P0 - Safety Fixes)
- [x] `crates/tools/src/domain_tools/locations.rs` - Replaced `std::sync::RwLock` with `parking_lot::RwLock`, removed `.unwrap()` calls
- [x] `crates/tools/src/domain_tools/locations.rs` - Added config-driven file paths with `VOICE_AGENT_DATA_DIR` env var support
- [x] `crates/llm/src/backend.rs` - Already using `parking_lot::Mutex` (no changes needed)

### Completed (P1 - Domain-Agnostic)
- [x] `crates/core/src/customer.rs` - Deprecated `DEFAULT_ASSET_PRICE_PER_UNIT` and variant factors, added `estimated_collateral_value_with_config()`
- [x] `crates/core/src/customer.rs` - Made `CustomerSegment` display names and messages domain-agnostic
- [x] `crates/core/src/customer.rs` - Made `SegmentDetector` support config-driven patterns via `with_config()`
- [x] `crates/core/src/financial.rs` - NEW: Consolidated EMI calculation into single source of truth
- [x] `crates/core/src/traits/calculator.rs` - Updated to use shared `financial::calculate_emi()`
- [x] `crates/core/src/traits/competitors.rs` - Updated to use shared `financial::calculate_emi()`
- [x] `crates/tools/src/domain_tools/utils.rs` - Re-exports from `voice_agent_core::financial`

### Remaining Work
- [ ] Remove segment presets in `segments.rs`
- [ ] Remove hardcoded competitor patterns in `intent/mod.rs`
- [ ] Remove hardcoded prompts in `prompt.rs`
- [ ] Rename gold-specific tool parameters
- [ ] Add asset config section to domain.yaml
- [ ] Wire new configs to views

---

## Executive Summary

Comprehensive review of `/home/vscode/goldloan-study/voice-agent/backend/` reveals a well-architected codebase with **60+ issues** requiring attention. The system has made significant progress toward domain-agnosticism through `MasterDomainConfig` and domain views, but hardcoded gold loan specific code remains in several locations.

**Core Goal**: Make the codebase truly config-driven so onboarding a new domain requires only YAML configs.

---

## Critical Findings Summary

### Architecture Strengths (Already Done Well)
- 12 focused Rust crates with clear responsibilities
- `MasterDomainConfig` with domain views (`AgentDomainView`, `LlmDomainView`, `ToolsDomainView`)
- `ToolFactory` trait - excellent factory pattern implementation
- Most business logic uses generic names (`asset_price_per_unit`, `variant_factors`)
- Trait-driven design with `Arc<dyn Trait>` for polymorphism

### Critical Issues Found

| Category | Count | Severity |
|----------|-------|----------|
| Hardcoded Domain Constants | 5 | CRITICAL |
| Hardcoded Domain Strings/Patterns | 15+ | CRITICAL |
| Config Values Not Wired | 8 | HIGH |
| Lock `.unwrap()` (panic risk) | 7 | HIGH |
| EMI Calculation Duplicated | 3 places | HIGH |
| Segment Presets Hardcoded | 7 | MEDIUM |
| Large Files (>1000 lines) | 10 | MEDIUM |
| TODOs/Incomplete Code | 3 | LOW |

---

## Phase 1: Safety Fixes (P0 - Immediate)

### 1.1 Replace `.unwrap()` on Lock Operations

**Files**:
- `crates/tools/src/domain_tools/locations.rs:71, 77, 85`
- `crates/llm/src/backend.rs:202, 209, 291, 296, 363, 444`

**Fix**: Replace `std::sync::RwLock` with `parking_lot::RwLock` (already used elsewhere in codebase). It never poisons and returns guard directly.

```rust
// Before (panics if poisoned):
*BRANCH_DATA.write().unwrap() = branches;

// After (parking_lot):
*BRANCH_DATA.write() = branches;
```

### 1.2 Fix Hardcoded File Paths

**File**: `crates/tools/src/domain_tools/locations.rs:40-44`

**Fix**: Use environment variable or config for data directory:
```rust
fn default_data_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(data_dir) = std::env::var("VOICE_AGENT_DATA_DIR") {
        paths.push(PathBuf::from(data_dir).join("branches.json"));
    }
    // ... fallbacks
}
```

---

## Phase 2: Domain-Agnostic Refactoring (P1 - High Priority)

### 2.1 Remove Hardcoded Constants

**File**: `crates/core/src/customer.rs`

| Line | Hardcoded Value | Fix |
|------|-----------------|-----|
| 245 | `DEFAULT_ASSET_PRICE_PER_UNIT = 7500.0` | Load from `asset.default_price_per_unit` in config |
| 262-264 | Purity factors (24K=1.0, 22K=0.916) | Load from `asset.variants` in config |
| 418-419 | Hindi gold patterns | Load from `vocabulary.yaml` |
| 439-443 | Hindi number terms | Load from config |
| 456-473 | Objection patterns | Already in `objections.yaml` - wire properly |

### 2.2 Remove Hardcoded Segment Presets

**File**: `crates/core/src/traits/segments.rs:217-432`

Remove these 7 preset factory methods and load ALL segments from config:
- `high_value()` - hardcodes 100g gold, 500000 loan
- `trust_seeker()` - hardcodes muthoot, manappuram, iifl
- `price_sensitive()` - hardcodes EMI, interest keywords
- `urgent_need()`
- `balance_transfer()`
- `first_time()`
- `business_owner()`

**Fix**: Delete preset methods. All segments come from `config/domains/{domain}/segments.yaml`.

### 2.3 Remove Hardcoded Competitor Patterns

**File**: `crates/text_processing/src/intent/mod.rs:575-596`

```rust
// REMOVE this hardcoded list:
let lender_patterns = vec![
    CompiledSlotPattern { name: "muthoot".to_string(), ... },
    CompiledSlotPattern { name: "manappuram".to_string(), ... },
    CompiledSlotPattern { name: "iifl".to_string(), ... },
];
```

**Fix**: Load competitor detection patterns from `config/domains/{domain}/competitors.yaml`.

### 2.4 Remove Hardcoded Prompts

**File**: `crates/llm/src/prompt.rs`

| Lines | Issue | Fix |
|-------|-------|-----|
| 245-285 | "RBI-regulated bank", product facts | Use `prompts/system.yaml` templates |
| 713-774 | "Kotak Mahindra Bank" in greetings | Use `{company_name}` placeholder |

### 2.5 Rename Gold-Specific Tool Parameters

**File**: `crates/tools/src/domain_tools/tools/eligibility.rs:90-110`

```rust
// RENAME:
"gold_weight_grams" -> "collateral_quantity"
"gold_purity" -> "collateral_variant"
```

### 2.6 Remove Hardcoded Scoring Values

**File**: `crates/core/src/traits/scoring.rs:273-285`

```rust
// REMOVE these hardcoded scores:
info_gold_weight_score: 8,
info_gold_purity_score: 5,
```

**Fix**: Rename to generic terms and load from `config/domains/{domain}/scoring.yaml`.

---

## Phase 3: Code Deduplication (P1)

### 3.1 Consolidate EMI Calculation

**Duplicated in 3 files**:
1. `crates/core/src/traits/calculator.rs:260-274` - uses `powi(i32)`
2. `crates/core/src/traits/competitors.rs:263-271` - uses `powi(i32)`
3. `crates/tools/src/domain_tools/utils.rs:16-33` - uses `powf(f64)`

**Fix**: Create single source of truth:

```rust
// NEW FILE: crates/core/src/financial.rs
pub fn calculate_emi(principal: f64, annual_rate_percent: f64, tenure_months: i64) -> f64 {
    if tenure_months <= 0 || principal <= 0.0 { return 0.0; }
    let monthly_rate = annual_rate_percent / 100.0 / 12.0;
    if monthly_rate <= 0.0 { return principal / tenure_months as f64; }
    let n = tenure_months as i32;
    let factor = (1.0 + monthly_rate).powi(n);
    principal * monthly_rate * factor / (factor - 1.0)
}
```

Then delete duplicates and use this single function everywhere.

---

## Phase 4: New Config Structure

### 4.1 Add Asset Configuration

**File**: `config/domains/gold_loan/domain.yaml`

Add new `asset` section:
```yaml
asset:
  type: "gold"
  display_name: "Gold"
  display_name_hi: "सोना"
  unit: "grams"
  default_price_per_unit: 7500.0
  variants:
    "24K":
      factor: 1.0
      aliases: ["24k", "24 karat", "pure"]
    "22K":
      factor: 0.916
      aliases: ["22k", "22 karat", "standard"]
    "18K":
      factor: 0.75
      aliases: ["18k", "18 karat"]
  default_variant: "22K"
```

### 4.2 Add Competitor Detection Patterns

**File**: `config/domains/gold_loan/competitors.yaml`

Add detection patterns:
```yaml
competitors:
  muthoot:
    display_name: "Muthoot Finance"
    detection_patterns:
      - "\\b(?i)muthoot\\b"
      - "\\b(?i)mutut\\b"
    aliases: ["muthoot", "muthut"]
```

### 4.3 Wire New Configs

**Files to update**:
- `crates/config/src/domain/master.rs` - Add `AssetConfig` struct
- `crates/config/src/domain/views.rs` - Add asset config methods to views
- `crates/server/src/state.rs` - Wire new config to components

---

## Phase 5: New Traits (P2)

### 5.1 Asset Configuration Trait

**New File**: `crates/core/src/traits/asset.rs`

```rust
pub trait AssetConfig: Send + Sync {
    fn asset_type(&self) -> &str;
    fn price_per_unit(&self) -> f64;
    fn variant_factor(&self, variant_id: &str) -> f64;
    fn parse_variant(&self, input: &str) -> Option<&str>;
    fn calculate_value(&self, quantity: f64, variant_id: &str) -> f64;
}
```

### 5.2 Competitor Detection Trait

**New File**: `crates/core/src/traits/competitor_detector.rs`

```rust
pub trait CompetitorDetector: Send + Sync {
    fn extract_competitor(&self, text: &str) -> Option<String>;
    fn competitor_patterns(&self) -> Vec<(String, String)>;
}
```

---

## Phase 6: Large File Splitting (P2)

| File | Lines | Recommended Split |
|------|-------|-------------------|
| `text_processing/src/intent/mod.rs` | 1534 | detector.rs, patterns.rs, extraction.rs, currency.rs |
| `config/src/domain/views.rs` | 1299 | types.rs, loader.rs, rendering.rs |
| `llm/src/speculative.rs` | 1147 | cache.rs, predictor.rs, executor.rs |
| `llm/src/backend.rs` | 1121 | claude.rs, ollama.rs, openai.rs |
| `agent/src/conversation.rs` | 1117 | turn.rs, context.rs, history.rs |

---

## Phase 7: Performance Optimizations (P3)

### 7.1 Reduce Cloning in Hot Paths

**Files**: `segments.rs`, `goals.rs`

- Use `Cow<'_, str>` instead of `String` where ownership not needed
- Use `&[T]` slices instead of cloning `Vec<T>`
- Use `Arc<T>` for shared read-only data

### 7.2 Lazy Initialization

For regex compilation, use `once_cell::sync::Lazy`:
```rust
static COMPILED_PATTERNS: Lazy<Vec<CompiledPattern>> = Lazy::new(|| compile_all_patterns());
```

---

## Phase 8: Cleanup (P3)

### 8.1 Remove Deprecated Code

**File**: `crates/llm/src/prompt.rs:288` - Remove `system_prompt()` after migrating callers

### 8.2 Address TODOs

- `pipeline/src/tts/mod.rs:221` - Piper ONNX backend
- `pipeline/src/tts/mod.rs:228` - ParlerTts ONNX backend
- `pipeline/src/stt/mod.rs:227` - Wav2Vec2 backend

---

## Critical Files for Implementation

| Priority | File | Changes |
|----------|------|---------|
| P0 | `crates/tools/src/domain_tools/locations.rs` | Lock safety, path handling |
| P1 | `crates/core/src/customer.rs` | Remove hardcoded constants (7500.0, purity factors) |
| P1 | `crates/core/src/traits/segments.rs` | Remove 7 preset factory methods |
| P1 | `crates/text_processing/src/intent/mod.rs` | Remove hardcoded competitor patterns |
| P1 | `crates/llm/src/prompt.rs` | Remove hardcoded prompts |
| P1 | `crates/core/src/traits/calculator.rs` | Consolidate EMI calculation |
| P1 | `crates/core/src/traits/competitors.rs` | Remove duplicate EMI, use config |
| P1 | `crates/tools/src/domain_tools/utils.rs` | Remove duplicate EMI |
| P1 | `crates/config/src/domain/master.rs` | Add AssetConfig |
| P1 | `crates/config/src/domain/views.rs` | Add asset config methods |
| P2 | `config/domains/gold_loan/domain.yaml` | Add asset section |
| P2 | `config/domains/gold_loan/competitors.yaml` | Add detection patterns |

---

## Verification Steps

### 1. Config Validation
```bash
# Create config validator
cargo run --bin config-validator -- --domain gold_loan
```

### 2. No-Hardcode Check
```bash
# Search for remaining domain-specific terms in core code
grep -r "gold\|loan\|muthoot\|kotak\|7500" crates/core/src/ --include="*.rs"
grep -r "gold\|loan\|muthoot\|kotak\|7500" crates/agent/src/ --include="*.rs"
```

### 3. Test with Alternate Domain
Create `config/domains/test_domain/` with different values and verify no gold-specific behavior leaks through.

### 4. Run Tests
```bash
cargo test --workspace
```

---

## New Domain Onboarding Checklist

After refactoring, onboarding a new domain (e.g., `vehicle_loan`) requires only:

```
config/domains/vehicle_loan/
├── domain.yaml          # Core + asset config
├── slots.yaml           # DST slots
├── segments.yaml        # Customer segments
├── competitors.yaml     # Competitor data
├── objections.yaml      # Objection handlers
├── goals.yaml           # Conversation goals
├── features.yaml        # Feature flags
├── prompts/system.yaml  # LLM prompts
└── tools/schemas.yaml   # Tool definitions
```

Then: `DOMAIN_ID=vehicle_loan cargo run`

---

## Implementation Order

1. **Week 1**: Safety fixes (P0) - Lock safety, path handling
2. **Week 2**: Create new config structures and traits
3. **Week 3**: Remove hardcoded values from customer.rs, segments.rs
4. **Week 4**: Remove hardcoded competitor patterns, prompts
5. **Week 5**: Consolidate EMI calculation, wire configs
6. **Week 6**: File splitting, performance optimizations
7. **Week 7**: Verification, documentation, cleanup
