# Phase 1: Critical Fixes

**Priority:** P0 - Must complete before any other phases
**Estimated Files:** 15 files
**Dependencies:** None

---

## Overview

This phase addresses foundational issues that block all subsequent refactoring:
1. Fix factory pattern to enable polymorphism
2. Remove duplicate traits
3. Extract hardcoded constants to config
4. Fix critical bugs

---

## Task 1.1: Fix DomainBridge Factory Pattern

**File:** `crates/config/src/domain/bridge.rs`

### Problem
All factory methods return `impl Trait` (static dispatch) instead of `Arc<dyn Trait>` (dynamic dispatch). This prevents:
- Runtime implementation swapping
- Mock implementations for testing
- Polymorphic usage

### Changes Required

#### 1.1.1 Fix calculator() method

**Lines 58-93**

**Before:**
```rust
pub fn calculator(&self) -> impl DomainCalculator {
    ConfigDrivenCalculator::new(
        self.config.constants.gold_price_per_gram,
        self.config.constants.ltv_percent,
        self.config.interest_rates.tiers.clone(),
    )
}
```

**After:**
```rust
pub fn calculator(&self) -> Arc<dyn DomainCalculator> {
    Arc::new(ConfigDrivenCalculator::new(
        self.config.constants.gold_price_per_gram,
        self.config.constants.ltv_percent,
        self.config.interest_rates.tiers.clone(),
    ))
}
```

#### 1.1.2 Fix lead_scoring() method

**Lines 94-159**

**Before:**
```rust
pub fn lead_scoring(&self) -> impl LeadScoringStrategy {
    ConfigLeadScoring::new(...)
}
```

**After:**
```rust
pub fn lead_scoring(&self) -> Arc<dyn LeadScoringStrategy> {
    Arc::new(ConfigLeadScoring::new(...))
}
```

#### 1.1.3 Fix competitor_analyzer() method

**Lines 160-216**

**Before:**
```rust
pub fn competitor_analyzer(&self) -> impl CompetitorAnalyzer {
    ConfigCompetitorAnalyzer::new(...)
}
```

**After:**
```rust
pub fn competitor_analyzer(&self) -> Arc<dyn CompetitorAnalyzer> {
    Arc::new(ConfigCompetitorAnalyzer::new(...))
}
```

#### 1.1.4 Fix segment_detector() method

**Lines 217-251**

**Before:**
```rust
pub fn segment_detector(&self) -> impl SegmentDetector {
    ConfigSegmentDetector::new(...)
}
```

**After:**
```rust
pub fn segment_detector(&self) -> Arc<dyn SegmentDetector> {
    Arc::new(ConfigSegmentDetector::new(...))
}
```

#### 1.1.5 Fix objection_handler() method

**Lines 252-294**

**Before:**
```rust
pub fn objection_handler(&self) -> impl ObjectionHandler {
    ConfigObjectionHandler::new(...)
}
```

**After:**
```rust
pub fn objection_handler(&self) -> Arc<dyn ObjectionHandler> {
    Arc::new(ConfigObjectionHandler::new(...))
}
```

#### 1.1.6 Fix goal_schema() method

**Lines 295-331**

**Before:**
```rust
pub fn goal_schema(&self) -> impl ConversationGoalSchema {
    ConfigGoalSchema::new(...)
}
```

**After:**
```rust
pub fn goal_schema(&self) -> Arc<dyn ConversationGoalSchema> {
    Arc::new(ConfigGoalSchema::new(...))
}
```

### Update Import Statement

**Line 1-10** - Add Arc import:
```rust
use std::sync::Arc;
```

### Verification
```bash
cargo check -p voice-agent-config
cargo test -p voice-agent-config
```

---

## Task 1.2: Remove Duplicate Traits from domain/traits.rs

**File:** `crates/core/src/domain/traits.rs`

### Problem
This file contains duplicate trait definitions that are superseded by implementations in `core/src/traits/`:

| Duplicate Trait | Proper Location | Status |
|-----------------|-----------------|--------|
| `ObjectionHandler` (lines 111-123) | `core/src/traits/objections.rs` | Has ConfigObjectionHandler impl |
| `CustomerSegment` (lines 95-107) | `core/src/traits/segments.rs` | SegmentDetector is proper trait |

### Changes Required

#### 1.2.1 Remove ObjectionHandler duplicate

**Delete lines 111-123:**
```rust
// DELETE THIS ENTIRE BLOCK
pub trait ObjectionHandler: Send + Sync {
    fn handle_objection(
        &self,
        objection_type: &str,
        context: &ObjectionContext,
    ) -> Option<ObjectionResponse>;

    fn get_objection_types(&self) -> Vec<&str>;

    fn detect_objection(&self, text: &str) -> Option<String>;
}
```

#### 1.2.2 Remove CustomerSegment duplicate

**Delete lines 95-107:**
```rust
// DELETE THIS ENTIRE BLOCK
pub trait CustomerSegment: Send + Sync {
    fn segment_id(&self) -> &str;
    fn display_name(&self) -> &str;
    fn description(&self) -> &str;
    fn priority(&self) -> u8;
    fn matches(&self, profile: &CustomerProfile) -> bool;
}
```

#### 1.2.3 Add deprecation notice to file header

**Add at line 1:**
```rust
//! # Domain Traits (Legacy)
//!
//! **DEPRECATED:** Most traits in this file have been superseded by
//! config-driven implementations in `crates/core/src/traits/`.
//!
//! Use:
//! - `traits::objections::ObjectionHandler` instead of local ObjectionHandler
//! - `traits::segments::SegmentDetector` instead of CustomerSegment
//!
//! This file will be removed in a future version.
```

#### 1.2.4 Update any imports

Search for imports of these traits:
```bash
grep -rn "use.*domain::traits::ObjectionHandler" crates/
grep -rn "use.*domain::traits::CustomerSegment" crates/
```

Update any found imports to use the correct trait from `core/src/traits/`.

### Verification
```bash
cargo check -p voice-agent-core
cargo test -p voice-agent-core
```

---

## Task 1.3: Create Unified ConfigLoadError

**File:** `crates/config/src/domain/mod.rs`

### Problem
Each config module defines identical error types:
- `BranchesConfigError`
- `CompetitorsConfigError`
- `PromptsConfigError`
- etc. (11 total)

### Changes Required

#### 1.3.1 Add unified error type to mod.rs

**Add after existing imports:**
```rust
use std::path::Path;
use thiserror::Error;

/// Unified error type for all domain config loading
#[derive(Error, Debug)]
pub enum ConfigLoadError {
    #[error("Config file not found: {path} - {source}")]
    FileNotFound {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse config file {path}: {message}")]
    ParseError {
        path: String,
        message: String,
    },

    #[error("Validation error in {path}: {message}")]
    ValidationError {
        path: String,
        message: String,
    },
}

impl ConfigLoadError {
    pub fn file_not_found(path: impl AsRef<Path>, source: std::io::Error) -> Self {
        Self::FileNotFound {
            path: path.as_ref().display().to_string(),
            source,
        }
    }

    pub fn parse_error(path: impl AsRef<Path>, err: serde_yaml::Error) -> Self {
        Self::ParseError {
            path: path.as_ref().display().to_string(),
            message: err.to_string(),
        }
    }
}
```

#### 1.3.2 Add ConfigFile trait

```rust
/// Trait for config files that can be loaded from YAML
pub trait ConfigFile: Sized + serde::de::DeserializeOwned {
    /// Load config from a YAML file
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigLoadError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigLoadError::file_not_found(path, e))?;
        serde_yaml::from_str(&content)
            .map_err(|e| ConfigLoadError::parse_error(path, e))
    }

    /// Load config with validation
    fn load_validated<P: AsRef<Path>>(path: P) -> Result<Self, ConfigLoadError> {
        let config = Self::load(path.as_ref())?;
        // Subclasses can override validate()
        Ok(config)
    }
}
```

#### 1.3.3 Update branches.rs to use unified error

**File:** `crates/config/src/domain/branches.rs`

**Delete lines 176-193** (BranchesConfigError enum)

**Update load method:**
```rust
impl ConfigFile for BranchesConfig {}

// Remove the old load() method - trait provides it
```

#### 1.3.4 Update all other config files

Repeat for each file:
- `competitors.rs` - delete CompetitorsConfigError
- `prompts.rs` - delete PromptsConfigError
- `objections.rs` - delete ObjectionsConfigError
- `slots.rs` - delete SlotsConfigError
- `scoring.rs` - delete ScoringConfigError
- `stages.rs` - delete StagesConfigError
- `tools.rs` - delete ToolsConfigError
- `segments.rs` - delete SegmentsConfigError
- `goals.rs` - delete GoalsConfigError
- `sms_templates.rs` - delete SmsTemplatesConfigError

### Verification
```bash
cargo check -p voice-agent-config
cargo test -p voice-agent-config
```

---

## Task 1.4: Extract Hardcoded Constants to Config

**Files to modify:**
- `crates/config/src/constants.rs` (delete most constants)
- `config/domains/gold_loan/domain.yaml` (ensure values exist)

### Problem
`constants.rs` has hardcoded values that duplicate domain.yaml:
- Interest rates (11.5%, 10.5%, 9.5%)
- Loan tiers (100000, 500000)
- LTV percent (75%)
- Gold price (7500.0)
- Purity factors

### Changes Required

#### 1.4.1 Deprecate constants in constants.rs

**File:** `crates/config/src/constants.rs`

**Add deprecation to each constant group:**
```rust
// ============================================
// DEPRECATED: Use MasterDomainConfig instead
// These constants will be removed in v0.2.0
// ============================================

#[deprecated(since = "0.1.1", note = "Use MasterDomainConfig.interest_rates instead")]
pub mod interest_rates {
    pub const TIER_1_STANDARD: f64 = 11.5;
    pub const TIER_2_HEADLINE: f64 = 10.5;
    pub const TIER_3_PREMIUM: f64 = 9.5;
}

#[deprecated(since = "0.1.1", note = "Use MasterDomainConfig.constants instead")]
pub mod loan_tiers {
    pub const TIER_1_MAX: f64 = 100_000.0;
    pub const TIER_2_MAX: f64 = 500_000.0;
}

#[deprecated(since = "0.1.1", note = "Use MasterDomainConfig.constants.ltv_percent instead")]
pub const LTV_PERCENT: f64 = 75.0;

#[deprecated(since = "0.1.1", note = "Use external gold price service instead")]
pub const DEFAULT_GOLD_PRICE_PER_GRAM: f64 = 7500.0;
```

#### 1.4.2 Verify domain.yaml has all values

**File:** `config/domains/gold_loan/domain.yaml`

Ensure these sections exist:
```yaml
constants:
  gold_price_per_gram: 7500.0
  ltv_percent: 75.0
  processing_fee_percent: 1.0
  min_loan_amount: 10000.0
  max_loan_amount: 25000000.0
  purity_factors:
    K24: 1.0
    K22: 0.916
    K18: 0.75
    K14: 0.585

interest_rates:
  base_rate: 10.5
  tiers:
    - max_amount: 100000
      rate: 11.5
    - max_amount: 500000
      rate: 10.5
    - max_amount: null  # unlimited
      rate: 9.5
```

#### 1.4.3 Update code that uses constants

Search and replace:
```bash
grep -rn "constants::interest_rates::" crates/
grep -rn "constants::loan_tiers::" crates/
grep -rn "constants::LTV_PERCENT" crates/
grep -rn "constants::DEFAULT_GOLD_PRICE" crates/
```

For each occurrence, replace with config access via `MasterDomainConfig` or `ToolsDomainView`.

### Verification
```bash
cargo check --workspace
cargo test --workspace
```

---

## Task 1.5: Fix Session Touch Bug

**File:** `crates/server/src/session.rs`

### Problem
Line 134: `InMemorySessionStore.touch()` sets timestamp to 0 instead of current time.

### Changes Required

#### 1.5.1 Fix touch() implementation

**Find the touch() method** (around line 134):

**Before:**
```rust
fn touch(&self, id: &str) {
    if let Some(mut session) = self.sessions.get_mut(id) {
        session.metadata.last_activity_ms = 0; // BUG!
    }
}
```

**After:**
```rust
fn touch(&self, id: &str) {
    if let Some(mut session) = self.sessions.get_mut(id) {
        session.metadata.last_activity_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
    }
}
```

#### 1.5.2 Add test for touch()

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_touch_updates_timestamp() {
        let store = InMemorySessionStore::new();

        // Create session
        let session = store.create(...).await.unwrap();
        let initial_time = session.metadata.last_activity_ms;

        // Wait a bit
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Touch
        store.touch(&session.metadata.id);

        // Verify timestamp updated
        let updated = store.get_metadata(&session.metadata.id).await.unwrap().unwrap();
        assert!(updated.last_activity_ms > initial_time);
    }
}
```

### Verification
```bash
cargo test -p voice-agent-server session
```

---

## Task 1.6: Fix PhoneticCorrector Domain Hardcoding

**File:** `crates/server/src/state.rs`

### Problem
Line 68-70 hardcodes `PhoneticCorrector::gold_loan()`:
```rust
let phonetic_corrector = Arc::new(PhoneticCorrector::gold_loan());
```

### Changes Required

#### 1.6.1 Make PhoneticCorrector config-driven

**Before (line 68-70):**
```rust
let phonetic_corrector = Arc::new(PhoneticCorrector::gold_loan());
tracing::info!("Initialized deterministic phonetic corrector for gold_loan domain");
```

**After:**
```rust
let phonetic_corrector = Arc::new(
    PhoneticCorrector::from_domain_config(&master_domain_config)
        .unwrap_or_else(|_| PhoneticCorrector::default())
);
tracing::info!(
    "Initialized phonetic corrector for {} domain",
    master_domain_config.domain_id()
);
```

#### 1.6.2 Add from_domain_config() to PhoneticCorrector

**File:** `crates/text_processing/src/phonetic.rs` (or wherever PhoneticCorrector is defined)

```rust
impl PhoneticCorrector {
    /// Create corrector from domain config
    pub fn from_domain_config(config: &MasterDomainConfig) -> Result<Self, Error> {
        let mut corrector = Self::default();

        // Load domain-specific terms from config
        if let Some(vocabulary) = config.vocabulary() {
            for (term, phonetic) in vocabulary.phonetic_mappings() {
                corrector.add_mapping(term, phonetic);
            }
        }

        // Load brand names
        corrector.add_mapping(&config.brand().bank_name, /* phonetic */);
        corrector.add_mapping(&config.brand().agent_name, /* phonetic */);

        // Load competitor names
        for competitor in config.competitors().values() {
            corrector.add_mapping(&competitor.name, /* phonetic */);
        }

        Ok(corrector)
    }
}
```

### Verification
```bash
cargo check -p voice-agent-server
cargo test -p voice-agent-server
```

---

## Task 1.7: Fix Default DOMAIN_ID Hardcoding

**File:** `crates/server/src/main.rs`

### Problem
Line 290 hardcodes "gold_loan" as default:
```rust
let domain_id = std::env::var("DOMAIN_ID").unwrap_or_else(|_| "gold_loan".to_string());
```

### Changes Required

#### 1.7.1 Require DOMAIN_ID or use config

**Option A: Require environment variable**
```rust
let domain_id = std::env::var("DOMAIN_ID")
    .expect("DOMAIN_ID environment variable must be set");
```

**Option B: Read from config file**
```rust
let domain_id = config.domain_id
    .clone()
    .or_else(|| std::env::var("DOMAIN_ID").ok())
    .expect("domain_id must be set in config or DOMAIN_ID env var");
```

#### 1.7.2 Add domain_id to Settings

**File:** `crates/config/src/settings.rs`

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    // ... existing fields ...

    /// Domain identifier (e.g., "gold_loan", "auto_loan")
    #[serde(default)]
    pub domain_id: Option<String>,
}
```

#### 1.7.3 Update default.yaml

**File:** `config/default.yaml`

```yaml
# Domain configuration
domain_id: gold_loan  # Can be overridden by DOMAIN_ID env var
```

### Verification
```bash
# Test without env var (should use config)
cargo run -p voice-agent-server

# Test with env var
DOMAIN_ID=gold_loan cargo run -p voice-agent-server
```

---

## Phase 1 Completion Checklist

- [ ] 1.1 DomainBridge returns Arc<dyn Trait> for all 6 methods
- [ ] 1.2 Duplicate traits removed from domain/traits.rs
- [ ] 1.3 Unified ConfigLoadError created and adopted
- [ ] 1.4 Hardcoded constants deprecated, config values verified
- [ ] 1.5 Session touch() bug fixed with test
- [ ] 1.6 PhoneticCorrector uses config instead of hardcoded gold_loan()
- [ ] 1.7 DOMAIN_ID is configurable, not hardcoded

### Verification Commands
```bash
# Full workspace check
cargo check --workspace

# Run all tests
cargo test --workspace

# Check for remaining hardcoded references
grep -rn "gold_loan()" crates/server/
grep -rn "PhoneticCorrector::gold_loan" crates/
```

---

## Dependencies for Phase 2

Phase 2 (Domain Decoupling) depends on:
- Task 1.1 complete (factory pattern fixed)
- Task 1.4 complete (constants in config)

Phase 2 can start tasks 2.1-2.2 once Phase 1 is complete.
