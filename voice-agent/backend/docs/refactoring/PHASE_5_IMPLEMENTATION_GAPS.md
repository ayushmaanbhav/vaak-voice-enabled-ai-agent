# Phase 5: Implementation Gaps

**Priority:** P3 - Complete the platform
**Estimated Files:** 10 files
**Dependencies:** Phases 1-2 complete

---

## Overview

This phase addresses missing implementations and adds extensibility features:
1. Implement missing traits (AudioProcessor, Retriever)
2. Add ConfigValidator for YAML validation
3. Create ToolFactory trait for domain-agnostic tool creation
4. Add feature flags for optional functionality
5. Create domain onboarding documentation

---

## Task 5.1: Implement AudioProcessor Trait (or Remove)

### Problem
`core/src/traits/speech.rs:329-343` defines AudioProcessor but it's not implemented:
- No AEC (Acoustic Echo Cancellation)
- No NS (Noise Suppression)
- No AGC (Automatic Gain Control)

### Decision Required
Choose one:
- **Option A:** Implement with existing libraries
- **Option B:** Document as "client-side only" and remove trait

---

#### Option A: Implement with RNNoise

**File:** `crates/pipeline/src/audio_processor.rs` (NEW)

```rust
//! Audio processing implementation using RNNoise

use voice_agent_core::traits::AudioProcessor;
use nnnoiseless::DenoiseState;

/// RNNoise-based audio processor
pub struct RNNoiseProcessor {
    denoise_state: DenoiseState,
    sample_rate: u32,
}

impl RNNoiseProcessor {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            denoise_state: DenoiseState::new(),
            sample_rate,
        }
    }
}

impl AudioProcessor for RNNoiseProcessor {
    fn process(&mut self, audio: &[f32]) -> Vec<f32> {
        // RNNoise expects 480-sample frames at 48kHz
        let frame_size = DenoiseState::FRAME_SIZE;

        audio.chunks(frame_size)
            .flat_map(|chunk| {
                if chunk.len() == frame_size {
                    let mut output = vec![0.0; frame_size];
                    self.denoise_state.process_frame(&mut output, chunk);
                    output
                } else {
                    // Handle partial frames
                    let mut padded = chunk.to_vec();
                    padded.resize(frame_size, 0.0);
                    let mut output = vec![0.0; frame_size];
                    self.denoise_state.process_frame(&mut output, &padded);
                    output[..chunk.len()].to_vec()
                }
            })
            .collect()
    }

    fn supports_aec(&self) -> bool {
        false  // RNNoise doesn't do AEC
    }

    fn supports_noise_suppression(&self) -> bool {
        true
    }

    fn supports_agc(&self) -> bool {
        false  // RNNoise doesn't do AGC
    }
}

/// Passthrough processor (no-op)
pub struct PassthroughProcessor;

impl AudioProcessor for PassthroughProcessor {
    fn process(&mut self, audio: &[f32]) -> Vec<f32> {
        audio.to_vec()
    }

    fn supports_aec(&self) -> bool { false }
    fn supports_noise_suppression(&self) -> bool { false }
    fn supports_agc(&self) -> bool { false }
}
```

#### Option B: Document as Out of Scope

**File:** `crates/core/src/traits/speech.rs`

Update trait documentation:
```rust
/// Audio processing trait for preprocessing audio before STT.
///
/// # Implementation Status
///
/// **This trait is intentionally NOT implemented server-side.**
///
/// Audio preprocessing (AEC, noise suppression, AGC) should be handled
/// client-side using:
/// - Browser: Web Audio API + AudioWorklet
/// - Mobile: Platform-native audio processing
/// - Desktop: OS audio subsystem
///
/// Server-side processing adds latency and is generally inferior to
/// client-side processing which has access to the echo reference signal.
///
/// If server-side processing is required, see `PassthroughAudioProcessor`
/// in the pipeline crate for a no-op implementation.
#[deprecated(since = "0.2.0", note = "Use client-side audio processing")]
pub trait AudioProcessor: Send + Sync {
    // ...
}
```

---

## Task 5.2: Implement Retriever Trait

### Problem
`core/src/traits/retriever.rs` defines Retriever but no concrete implementations exist.

### Solution
Implement QdrantRetriever and VectorStoreRetriever.

---

#### 5.2.1 Create Qdrant Retriever

**File:** `crates/rag/src/retriever/qdrant.rs` (NEW)

```rust
//! Qdrant-based retriever implementation

use async_trait::async_trait;
use qdrant_client::prelude::*;
use voice_agent_core::traits::{Retriever, RetrieveOptions, RetrievedDocument};

pub struct QdrantRetriever {
    client: QdrantClient,
    collection_name: String,
    embedding_model: Box<dyn EmbeddingModel>,
}

impl QdrantRetriever {
    pub async fn new(
        url: &str,
        collection_name: &str,
        embedding_model: Box<dyn EmbeddingModel>,
    ) -> Result<Self, QdrantError> {
        let client = QdrantClient::from_url(url).build()?;

        // Ensure collection exists
        if !client.collection_exists(collection_name).await? {
            return Err(QdrantError::CollectionNotFound(collection_name.to_string()));
        }

        Ok(Self {
            client,
            collection_name: collection_name.to_string(),
            embedding_model,
        })
    }
}

#[async_trait]
impl Retriever for QdrantRetriever {
    async fn retrieve(
        &self,
        query: &str,
        options: RetrieveOptions,
    ) -> Result<Vec<RetrievedDocument>, RetrievalError> {
        // Generate embedding for query
        let embedding = self.embedding_model.embed(query).await?;

        // Search Qdrant
        let results = self.client
            .search_points(&SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: embedding,
                limit: options.top_k as u64,
                score_threshold: Some(options.min_score),
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await?;

        // Convert to RetrievedDocument
        let documents = results.result.into_iter()
            .map(|point| {
                let payload = point.payload;
                RetrievedDocument {
                    id: point.id.to_string(),
                    content: payload.get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    score: point.score,
                    metadata: payload.into_iter()
                        .filter(|(k, _)| k != "content")
                        .map(|(k, v)| (k, v.to_string()))
                        .collect(),
                }
            })
            .collect();

        Ok(documents)
    }

    async fn retrieve_agentic(
        &self,
        query: &str,
        options: RetrieveOptions,
        context: &str,
    ) -> Result<Vec<RetrievedDocument>, RetrievalError> {
        // Expand query using context
        let expanded_query = format!("{} {}", query, context);
        self.retrieve(&expanded_query, options).await
    }
}
```

#### 5.2.2 Create In-Memory Retriever for Testing

**File:** `crates/rag/src/retriever/memory.rs` (NEW)

```rust
//! In-memory retriever for testing and development

use async_trait::async_trait;
use voice_agent_core::traits::{Retriever, RetrieveOptions, RetrievedDocument};

pub struct InMemoryRetriever {
    documents: Vec<(String, Vec<f32>)>,  // (content, embedding)
    embedding_model: Box<dyn EmbeddingModel>,
}

impl InMemoryRetriever {
    pub fn new(embedding_model: Box<dyn EmbeddingModel>) -> Self {
        Self {
            documents: Vec::new(),
            embedding_model,
        }
    }

    pub async fn add_document(&mut self, content: &str) -> Result<(), RetrievalError> {
        let embedding = self.embedding_model.embed(content).await?;
        self.documents.push((content.to_string(), embedding));
        Ok(())
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        dot / (norm_a * norm_b)
    }
}

#[async_trait]
impl Retriever for InMemoryRetriever {
    async fn retrieve(
        &self,
        query: &str,
        options: RetrieveOptions,
    ) -> Result<Vec<RetrievedDocument>, RetrievalError> {
        let query_embedding = self.embedding_model.embed(query).await?;

        let mut scored: Vec<_> = self.documents.iter()
            .enumerate()
            .map(|(idx, (content, embedding))| {
                let score = Self::cosine_similarity(&query_embedding, embedding);
                (idx, content.clone(), score)
            })
            .filter(|(_, _, score)| *score >= options.min_score)
            .collect();

        scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        scored.truncate(options.top_k);

        Ok(scored.into_iter()
            .map(|(idx, content, score)| RetrievedDocument {
                id: idx.to_string(),
                content,
                score,
                metadata: HashMap::new(),
            })
            .collect())
    }
}
```

---

## Task 5.3: Create ConfigValidator

### Problem
No validation of YAML configs at startup. Invalid configs cause runtime errors.

---

#### 5.3.1 Create Validation Framework

**File:** `crates/config/src/domain/validator.rs` (NEW or update existing)

```rust
//! Configuration validation framework

use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Missing required field: {field} in {file}")]
    MissingField { file: String, field: String },

    #[error("Invalid value for {field}: {message}")]
    InvalidValue { field: String, message: String },

    #[error("Reference error: {reference} not found (referenced in {field})")]
    ReferenceError { field: String, reference: String },

    #[error("Duplicate key: {key} in {file}")]
    DuplicateKey { file: String, key: String },

    #[error("Schema violation: {message}")]
    SchemaViolation { message: String },
}

pub struct ConfigValidator {
    errors: Vec<ValidationError>,
    warnings: Vec<String>,
}

impl ConfigValidator {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn validate_master_config(
        &mut self,
        config: &MasterDomainConfig,
    ) -> Result<(), Vec<ValidationError>> {
        // Validate brand config
        self.validate_brand(&config.brand);

        // Validate slots
        self.validate_slots(&config.slots_config);

        // Validate goals reference valid slots
        self.validate_goals(&config.goals_config, &config.slots_config);

        // Validate stages
        self.validate_stages(&config.stages_config);

        // Validate competitors
        self.validate_competitors(&config.competitors_config);

        // Validate objections reference valid types
        self.validate_objections(&config.objections_config);

        // Cross-validate references
        self.validate_cross_references(config);

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    fn validate_brand(&mut self, brand: &BrandConfig) {
        if brand.bank_name.is_empty() {
            self.errors.push(ValidationError::MissingField {
                file: "domain.yaml".to_string(),
                field: "brand.bank_name".to_string(),
            });
        }

        if brand.agent_name.is_empty() {
            self.errors.push(ValidationError::MissingField {
                file: "domain.yaml".to_string(),
                field: "brand.agent_name".to_string(),
            });
        }
    }

    fn validate_slots(&mut self, slots: &SlotsConfig) {
        let mut seen_ids = HashSet::new();

        for (id, slot) in &slots.slots {
            // Check for duplicates
            if !seen_ids.insert(id) {
                self.errors.push(ValidationError::DuplicateKey {
                    file: "slots.yaml".to_string(),
                    key: id.clone(),
                });
            }

            // Validate slot type
            if slot.slot_type.is_none() {
                self.warnings.push(format!(
                    "Slot '{}' has no explicit type, defaulting to String",
                    id
                ));
            }

            // Validate enum values if type is Enum
            if let Some(SlotType::Enum) = slot.slot_type {
                if slot.allowed_values.is_none() || slot.allowed_values.as_ref().unwrap().is_empty() {
                    self.errors.push(ValidationError::InvalidValue {
                        field: format!("slots.{}", id),
                        message: "Enum slot must have allowed_values".to_string(),
                    });
                }
            }
        }
    }

    fn validate_goals(&mut self, goals: &GoalsConfig, slots: &SlotsConfig) {
        for (id, goal) in &goals.goals {
            // Validate required slots exist
            for slot_name in goal.required_slots.iter().flatten() {
                if !slots.slots.contains_key(slot_name) {
                    self.errors.push(ValidationError::ReferenceError {
                        field: format!("goals.{}.required_slots", id),
                        reference: slot_name.clone(),
                    });
                }
            }

            // Validate intent mappings
            for intent in goal.intents.iter().flatten() {
                if intent.is_empty() {
                    self.warnings.push(format!(
                        "Goal '{}' has empty intent in intents list",
                        id
                    ));
                }
            }
        }
    }

    fn validate_cross_references(&mut self, config: &MasterDomainConfig) {
        // Validate stage transitions reference valid stages
        let stage_ids: HashSet<_> = config.stages_config.stages.keys().collect();

        for (stage_id, stage) in &config.stages_config.stages {
            if let Some(next_stages) = &stage.possible_next_stages {
                for next in next_stages {
                    if !stage_ids.contains(next) {
                        self.errors.push(ValidationError::ReferenceError {
                            field: format!("stages.{}.possible_next_stages", stage_id),
                            reference: next.clone(),
                        });
                    }
                }
            }
        }

        // Validate competitor references in objections
        let competitor_ids: HashSet<_> = config.competitors_config.competitors.keys().collect();

        for (_, objection) in &config.objections_config.objections {
            if let Some(competitors) = &objection.mentioned_competitors {
                for comp in competitors {
                    if !competitor_ids.contains(comp) {
                        self.warnings.push(format!(
                            "Objection references unknown competitor: {}",
                            comp
                        ));
                    }
                }
            }
        }
    }

    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
}
```

#### 5.3.2 Add Validation to Startup

**File:** `crates/server/src/main.rs`

```rust
use voice_agent_config::domain::ConfigValidator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    let master_config = MasterDomainConfig::load(&domain_id, &config_dir)?;

    // Validate config
    let mut validator = ConfigValidator::new();
    if let Err(errors) = validator.validate_master_config(&master_config) {
        tracing::error!("Configuration validation failed:");
        for error in &errors {
            tracing::error!("  - {}", error);
        }
        return Err("Invalid configuration".into());
    }

    // Log warnings
    for warning in validator.warnings() {
        tracing::warn!("Config warning: {}", warning);
    }

    tracing::info!("Configuration validated successfully");

    // Continue with startup...
}
```

---

## Task 5.4: Add Feature Flags

### Problem
No way to enable/disable features without code changes.

---

#### 5.4.1 Create Feature Config

**File:** `config/domains/gold_loan/features.yaml` (update)

```yaml
# Feature flags for gold_loan domain
feature_flags:
  # Core features
  balance_transfer_enabled: true
  competitor_comparison_enabled: true
  doorstep_service_enabled: true

  # Segment detection
  segment_detection:
    enabled: true
    high_value_detection: true
    price_sensitive_detection: true
    trust_seeker_detection: true

  # Lead scoring
  lead_scoring:
    enabled: true
    escalation_triggers_enabled: true
    urgency_detection_enabled: true

  # Integration features
  crm_integration_enabled: false
  calendar_integration_enabled: false
  sms_integration_enabled: true

  # AI features
  speculative_execution_enabled: true
  rag_enabled: true

  # Compliance
  ai_disclosure_required: true
  recording_disclosure_required: true

  # Experimental
  experimental:
    multilingual_tts: false
    realtime_translation: false
```

#### 5.4.2 Create Feature Flag Service

**File:** `crates/config/src/feature_flags.rs` (NEW)

```rust
//! Feature flag service

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct FeatureFlags {
    #[serde(flatten)]
    flags: HashMap<String, FlagValue>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum FlagValue {
    Bool(bool),
    Nested(HashMap<String, FlagValue>),
}

impl FeatureFlags {
    pub fn is_enabled(&self, flag_path: &str) -> bool {
        let parts: Vec<&str> = flag_path.split('.').collect();
        self.get_flag_value(&parts)
            .map(|v| matches!(v, FlagValue::Bool(true)))
            .unwrap_or(false)
    }

    fn get_flag_value(&self, path: &[&str]) -> Option<&FlagValue> {
        if path.is_empty() {
            return None;
        }

        let mut current = self.flags.get(path[0])?;

        for part in &path[1..] {
            match current {
                FlagValue::Nested(map) => {
                    current = map.get(*part)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }
}

// Usage:
// if features.is_enabled("balance_transfer_enabled") { ... }
// if features.is_enabled("segment_detection.high_value_detection") { ... }
```

#### 5.4.3 Integrate Feature Flags

**File:** `crates/agent/src/agent/mod.rs`

```rust
impl DomainAgent {
    pub async fn process(&self, input: &str) -> Result<Response, AgentError> {
        let features = self.domain_view.feature_flags();

        // Check feature flag before processing
        if features.is_enabled("balance_transfer_enabled") {
            // Handle balance transfer logic
        }

        if features.is_enabled("segment_detection.enabled") {
            // Run segment detection
        }

        // ...
    }
}
```

---

## Task 5.5: Create Domain Onboarding Guide

**File:** `docs/DOMAIN_ONBOARDING.md` (NEW)

```markdown
# Domain Onboarding Guide

This guide explains how to add a new business domain to the voice agent platform.

## Prerequisites

- Rust 1.75+
- Access to configuration directory
- Understanding of the domain's business logic

## Step 1: Create Domain Directory Structure

```bash
mkdir -p config/domains/{domain_id}
mkdir -p config/domains/{domain_id}/tools
mkdir -p config/domains/{domain_id}/prompts
```

## Step 2: Create Required Configuration Files

### domain.yaml (Required)
```yaml
domain_id: "{domain_id}"
display_name: "Your Domain Name"

brand:
  bank_name: "Your Company"
  agent_name: "Agent Name"
  agent_role: "Role Description"
  helpline: "1800-XXX-XXXX"
  website: "https://..."

constants:
  # Domain-specific constants
  # Example for insurance:
  # premium_calculation_factor: 1.5
  # max_coverage: 10000000

interest_rates:  # Or equivalent for your domain
  base_rate: 10.0
  # ...
```

### slots.yaml (Required)
```yaml
slot_mappings:
  amount: "policy_amount"  # Map generic to domain-specific
  # ...

slots:
  policy_amount:
    type: number
    description: "Coverage amount"
    required: true
    min_value: 100000
    max_value: 10000000
  # Add all slots your domain needs
```

### goals.yaml (Required)
```yaml
goals:
  new_policy:
    display_name: "New Policy"
    required_slots:
      - customer_name
      - policy_amount
      - coverage_type
    intents:
      - policy_inquiry
      - new_policy_request
```

### stages.yaml (Required)
```yaml
stages:
  greeting:
    display_name: "Greeting"
    description: "Initial greeting"
    possible_next_stages:
      - discovery
  # Define your conversation flow
```

### competitors.yaml (Optional)
```yaml
competitors:
  competitor_1:
    name: "Competitor Name"
    type: "competitor_type"
    # ...
```

### objections.yaml (Optional)
```yaml
objections:
  price:
    responses:
      en: "Our pricing is competitive because..."
      hi: "..."
```

### prompts/system.yaml (Required)
```yaml
templates:
  base_persona: |
    You are {agent_name}, a {agent_role} at {bank_name}.
    # ...
```

### tools/schemas.yaml (Required)
```yaml
tools:
  check_eligibility:
    description: "Check customer eligibility"
    parameters:
      # Define JSON schema
```

## Step 3: Implement Domain-Specific Tools (If Needed)

If your domain needs custom tools beyond the generic ones:

```rust
// crates/tools/src/{domain_id}/mod.rs
pub mod tools;

// crates/tools/src/{domain_id}/tools/eligibility.rs
pub struct EligibilityTool {
    view: Arc<ToolsDomainView>,
}

impl Tool for EligibilityTool {
    // ...
}
```

## Step 4: Create Tool Factory

```rust
// crates/tools/src/{domain_id}/factory.rs
pub struct DomainToolFactory {
    view: Arc<ToolsDomainView>,
}

impl ToolFactory for DomainToolFactory {
    fn available_tools(&self) -> Vec<ToolMetadata> {
        // Return list of tools
    }

    fn create_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        // Create tool instances
    }
}
```

## Step 5: Register Domain

No code changes needed! The platform automatically discovers domains
based on the `config/domains/{domain_id}/` directory structure.

Set the domain at runtime:
```bash
DOMAIN_ID={domain_id} cargo run -p voice-agent-server
```

## Step 6: Validate Configuration

```bash
cargo run -p voice-agent-config --bin validate_config -- --domain {domain_id}
```

## Step 7: Test Domain

```bash
cargo test --workspace -- --test-threads=1 domain_{domain_id}
```

## Checklist

- [ ] domain.yaml created with brand info
- [ ] slots.yaml with all required slots
- [ ] goals.yaml with conversation goals
- [ ] stages.yaml with conversation flow
- [ ] prompts/system.yaml with LLM prompts
- [ ] tools/schemas.yaml with tool definitions
- [ ] Config validation passes
- [ ] All tests pass

## Common Issues

### "Slot not found" errors
Ensure all slots referenced in goals.yaml are defined in slots.yaml.

### "Invalid stage transition"
Check stages.yaml has valid `possible_next_stages` references.

### "Tool execution failed"
Verify tools/schemas.yaml has correct JSON schemas.
```

---

## Phase 5 Completion Checklist

- [ ] 5.1 AudioProcessor implemented or documented as out-of-scope
- [ ] 5.2.1 QdrantRetriever implemented
- [ ] 5.2.2 InMemoryRetriever implemented for testing
- [ ] 5.3.1 ConfigValidator framework created
- [ ] 5.3.2 Validation added to startup
- [ ] 5.4.1 Feature flags config created
- [ ] 5.4.2 Feature flag service implemented
- [ ] 5.4.3 Feature flags integrated into agent
- [ ] 5.5 Domain onboarding guide created

### Verification Commands
```bash
# Run config validation
cargo run -p voice-agent-config --bin validate_config -- --domain gold_loan

# Test retriever implementations
cargo test -p voice-agent-rag retriever

# Test feature flags
cargo test -p voice-agent-config feature_flags

# Validate new domain can be onboarded
mkdir -p config/domains/test_domain
# Create minimal configs
cargo run -p voice-agent-config --bin validate_config -- --domain test_domain
```

---

## Final Verification: Domain-Agnostic Test

After completing all 5 phases, verify domain-agnosticism:

```bash
# 1. Create a minimal test domain
./scripts/create_test_domain.sh

# 2. Run server with test domain
DOMAIN_ID=test_domain cargo run -p voice-agent-server

# 3. Verify no gold_loan references in logs
cargo run -p voice-agent-server 2>&1 | grep -i "gold_loan"  # Should return nothing

# 4. Check no hardcoded domain references in compiled binary
strings target/release/voice-agent-server | grep -i "gold_loan"  # Should return nothing (excluding configs)
```

**Success Criteria:**
- Server starts with any valid domain config
- No hardcoded domain references in Rust code
- All functionality driven by YAML configs
- New domain requires zero code changes
