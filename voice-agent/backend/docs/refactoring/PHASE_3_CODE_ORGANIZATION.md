# Phase 3: Code Organization

**Priority:** P2 - Improve maintainability and reduce technical debt
**Estimated Files:** 40 files
**Dependencies:** Phase 1 complete, Phase 2 tasks 2.3-2.4 complete

---

## Overview

This phase focuses on code organization improvements:
1. Split monster files into manageable modules
2. Consolidate duplicate code
3. Fix crate boundaries
4. Extract common utilities

---

## Task 3.1: Split Monster Files

### 3.1.1 Split indicconformer.rs (1639 lines → 4 files)

**Current File:** `crates/pipeline/src/stt/indicconformer.rs`

**Create new module structure:**
```
crates/pipeline/src/stt/indicconformer/
├── mod.rs           # Public API, IndicConformerStt struct
├── ort_backend.rs   # ONNX Runtime implementation
├── candle_backend.rs # Candle implementation
├── mel_filterbank.rs # Audio preprocessing
└── decoder.rs       # CTC decoder
```

#### mod.rs (New)
```rust
//! IndicConformer Speech-to-Text
//!
//! Supports two backends:
//! - ONNX Runtime (production)
//! - Candle (native Rust, development)

mod ort_backend;
mod candle_backend;
mod mel_filterbank;
mod decoder;

pub use ort_backend::OrtIndicConformer;
pub use candle_backend::CandleIndicConformer;
pub use mel_filterbank::MelFilterbank;
pub use decoder::{CtcDecoder, DecoderConfig};

use crate::PipelineError;
use voice_agent_core::traits::SpeechToText;

/// Configuration for IndicConformer STT
#[derive(Debug, Clone)]
pub struct IndicConformerConfig {
    pub model_path: std::path::PathBuf,
    pub backend: Backend,
    pub sample_rate: u32,
    pub use_gpu: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Backend {
    OnnxRuntime,
    Candle,
}

/// IndicConformer STT engine
pub struct IndicConformerStt {
    inner: Box<dyn SpeechToText>,
    config: IndicConformerConfig,
}

impl IndicConformerStt {
    pub fn new(config: IndicConformerConfig) -> Result<Self, PipelineError> {
        let inner: Box<dyn SpeechToText> = match config.backend {
            Backend::OnnxRuntime => Box::new(OrtIndicConformer::new(&config)?),
            Backend::Candle => Box::new(CandleIndicConformer::new(&config)?),
        };
        Ok(Self { inner, config })
    }
}

impl SpeechToText for IndicConformerStt {
    fn transcribe(&self, audio: &[f32]) -> Result<String, PipelineError> {
        self.inner.transcribe(audio)
    }

    fn supported_languages(&self) -> Vec<&str> {
        self.inner.supported_languages()
    }
}
```

#### ort_backend.rs (Extract from lines 200-600)
```rust
//! ONNX Runtime backend for IndicConformer

use ort::{Session, SessionBuilder};
use super::{MelFilterbank, CtcDecoder, IndicConformerConfig};
use crate::PipelineError;

pub struct OrtIndicConformer {
    session: Session,
    mel_filterbank: MelFilterbank,
    decoder: CtcDecoder,
}

impl OrtIndicConformer {
    pub fn new(config: &IndicConformerConfig) -> Result<Self, PipelineError> {
        // ... ONNX session initialization
        // Extract from original lines 200-350
    }

    pub fn process_audio(&self, audio: &[f32]) -> Result<Vec<f32>, PipelineError> {
        // ... inference logic
        // Extract from original lines 350-500
    }
}
```

#### candle_backend.rs (Extract from lines 600-900)
```rust
//! Candle (native Rust) backend for IndicConformer

use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use super::{MelFilterbank, CtcDecoder, IndicConformerConfig};

pub struct CandleIndicConformer {
    model: ConformerModel,
    mel_filterbank: MelFilterbank,
    decoder: CtcDecoder,
    device: Device,
}

// Extract from original lines 600-900
```

#### mel_filterbank.rs (Extract from lines 900-1200)
```rust
//! Mel filterbank for audio preprocessing

use ndarray::{Array1, Array2};

pub struct MelFilterbank {
    n_fft: usize,
    n_mels: usize,
    sample_rate: u32,
    filterbank: Array2<f32>,
}

impl MelFilterbank {
    pub fn new(n_fft: usize, n_mels: usize, sample_rate: u32) -> Self {
        // Extract from original lines 900-1050
    }

    pub fn compute(&self, audio: &[f32]) -> Array2<f32> {
        // Extract from original lines 1050-1200
    }
}
```

#### decoder.rs (Extract from lines 1200-1639)
```rust
//! CTC decoder for converting model output to text

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DecoderConfig {
    pub blank_id: i32,
    pub beam_width: usize,
    pub language_model_weight: f32,
}

pub struct CtcDecoder {
    vocab: HashMap<i32, char>,
    config: DecoderConfig,
}

impl CtcDecoder {
    pub fn new(vocab_path: &std::path::Path, config: DecoderConfig) -> Self {
        // Extract from original lines 1200-1350
    }

    pub fn decode(&self, logits: &[f32]) -> String {
        // Extract from original lines 1350-1500
    }

    pub fn decode_with_beam_search(&self, logits: &[f32]) -> Vec<(String, f32)> {
        // Extract from original lines 1500-1639
    }
}
```

---

### 3.1.2 Split intent/mod.rs (1521 lines → 4 files)

**Current File:** `crates/text_processing/src/intent/mod.rs`

**Create new module structure:**
```
crates/text_processing/src/intent/
├── mod.rs          # Public API
├── detector.rs     # IntentDetector
├── patterns.rs     # Regex patterns
├── indic.rs        # Indic numeral conversion
└── types.rs        # Intent/Slot types
```

#### mod.rs (New)
```rust
//! Intent detection and slot extraction

mod detector;
mod patterns;
mod indic;
mod types;

pub use detector::IntentDetector;
pub use patterns::PatternRegistry;
pub use indic::IndicNumeralConverter;
pub use types::{Intent, DetectedIntent, IntentConfig};
```

#### types.rs (Extract type definitions)
```rust
//! Intent and slot type definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub name: String,
    pub confidence: f32,
    pub slots: Vec<DetectedSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedSlot {
    pub name: String,
    pub value: SlotValue,
    pub confidence: f32,
    pub source_span: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SlotValue {
    String(String),
    Number(f64),
    Boolean(bool),
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntentConfig {
    pub patterns_path: Option<std::path::PathBuf>,
    pub confidence_threshold: f32,
    pub max_intents: usize,
}

impl Default for IntentConfig {
    fn default() -> Self {
        Self {
            patterns_path: None,
            confidence_threshold: 0.7,
            max_intents: 3,
        }
    }
}
```

#### patterns.rs (Extract ~300 lines of regex patterns)
```rust
//! Regex pattern registry for intent detection

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

pub struct PatternRegistry {
    intent_patterns: HashMap<String, Vec<CompiledPattern>>,
    slot_patterns: HashMap<String, Vec<CompiledPattern>>,
}

struct CompiledPattern {
    regex: Regex,
    confidence: f32,
    capture_groups: Vec<String>,
}

impl PatternRegistry {
    /// Load patterns from config file
    pub fn from_config(path: &std::path::Path) -> Result<Self, Error> {
        // Load YAML and compile patterns
    }

    /// Create with default patterns (for backwards compatibility)
    pub fn default_patterns() -> Self {
        // Use lazy static patterns
    }

    pub fn match_intent(&self, text: &str) -> Vec<(String, f32)> {
        // Intent matching logic
    }

    pub fn extract_slots(&self, text: &str, intent: &str) -> Vec<DetectedSlot> {
        // Slot extraction logic
    }
}
```

#### indic.rs (Extract Hindi/Devanagari utilities)
```rust
//! Indic language utilities

use std::collections::HashMap;

/// Convert Indic numerals to Arabic numerals
pub struct IndicNumeralConverter {
    mappings: HashMap<char, char>,
}

impl IndicNumeralConverter {
    pub fn new() -> Self {
        let mut mappings = HashMap::new();
        // Devanagari digits
        mappings.insert('०', '0');
        mappings.insert('१', '1');
        mappings.insert('२', '2');
        // ... etc
        Self { mappings }
    }

    pub fn convert(&self, text: &str) -> String {
        text.chars()
            .map(|c| *self.mappings.get(&c).unwrap_or(&c))
            .collect()
    }
}

/// Hindi multiplier words
pub fn parse_hindi_multiplier(word: &str) -> Option<f64> {
    match word.to_lowercase().as_str() {
        "हज़ार" | "हजार" => Some(1000.0),
        "लाख" => Some(100_000.0),
        "करोड़" => Some(10_000_000.0),
        _ => None,
    }
}
```

#### detector.rs (Main IntentDetector, ~400 lines)
```rust
//! Intent detection engine

use super::{PatternRegistry, IndicNumeralConverter, Intent, IntentConfig};

pub struct IntentDetector {
    patterns: PatternRegistry,
    numeral_converter: IndicNumeralConverter,
    config: IntentConfig,
}

impl IntentDetector {
    pub fn new(config: IntentConfig) -> Self {
        let patterns = config.patterns_path
            .as_ref()
            .map(|p| PatternRegistry::from_config(p).unwrap_or_default())
            .unwrap_or_else(PatternRegistry::default_patterns);

        Self {
            patterns,
            numeral_converter: IndicNumeralConverter::new(),
            config,
        }
    }

    pub fn detect(&self, text: &str) -> Vec<Intent> {
        // Normalize text
        let normalized = self.normalize(text);

        // Match intents
        let intent_matches = self.patterns.match_intent(&normalized);

        // Extract slots for top intents
        intent_matches.into_iter()
            .take(self.config.max_intents)
            .filter(|(_, conf)| *conf >= self.config.confidence_threshold)
            .map(|(intent_name, confidence)| {
                let slots = self.patterns.extract_slots(&normalized, &intent_name);
                Intent { name: intent_name, confidence, slots }
            })
            .collect()
    }

    fn normalize(&self, text: &str) -> String {
        let converted = self.numeral_converter.convert(text);
        converted.to_lowercase()
    }
}
```

---

### 3.1.3 Split dst/slots.rs (1377 lines → 3 files)

**Current File:** `crates/agent/src/dst/slots.rs`

**Create new module structure:**
```
crates/agent/src/dst/
├── mod.rs              # Existing, update imports
├── slots.rs            # Slot definitions (reduced)
├── state.rs            # DynamicDialogueState (from Phase 2)
├── actions.rs          # NextBestAction logic
└── legacy_compat.rs    # GoldLoanDialogueState wrapper
```

#### slots.rs (Reduced to ~200 lines)
```rust
//! Slot type definitions and utilities

use serde::{Deserialize, Serialize};

/// Slot value with confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotEntry {
    pub value: SlotValue,
    pub confidence: f32,
    pub source: SlotSource,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SlotSource {
    UserUtterance,
    ToolResult,
    Inference,
    Default,
}

/// Slot validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Validate slot value against schema
pub fn validate_slot(
    value: &SlotValue,
    schema: &SlotDefinition,
) -> ValidationResult {
    let mut result = ValidationResult {
        is_valid: true,
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    // Type validation
    match (&value, &schema.slot_type) {
        (SlotValue::Number(n), SlotType::Number) => {
            if let Some(min) = schema.min_value {
                if *n < min {
                    result.is_valid = false;
                    result.errors.push(format!("Value {} below minimum {}", n, min));
                }
            }
            if let Some(max) = schema.max_value {
                if *n > max {
                    result.is_valid = false;
                    result.errors.push(format!("Value {} above maximum {}", n, max));
                }
            }
        }
        (SlotValue::String(s), SlotType::Enum) => {
            if let Some(allowed) = &schema.allowed_values {
                if !allowed.contains(s) {
                    result.is_valid = false;
                    result.errors.push(format!("Value '{}' not in allowed values", s));
                }
            }
        }
        _ => {}
    }

    result
}
```

#### actions.rs (Extract NextBestAction, ~300 lines)
```rust
//! Next Best Action recommendation engine

use super::DynamicDialogueState;
use voice_agent_config::domain::{GoalsConfig, SlotsConfig};

#[derive(Debug, Clone)]
pub struct NextBestAction {
    pub action_type: ActionType,
    pub priority: u8,
    pub prompt_suggestion: String,
    pub target_slot: Option<String>,
    pub tool_suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum ActionType {
    AskForSlot,
    ConfirmValue,
    SuggestTool,
    AdvanceStage,
    HandleObjection,
    Escalate,
}

pub struct ActionRecommender {
    goals_config: GoalsConfig,
    slots_config: SlotsConfig,
}

impl ActionRecommender {
    pub fn new(goals_config: GoalsConfig, slots_config: SlotsConfig) -> Self {
        Self { goals_config, slots_config }
    }

    pub fn recommend(&self, state: &DynamicDialogueState) -> Vec<NextBestAction> {
        let mut actions = Vec::new();

        // Check for missing required slots
        if let Some(intent) = &state.intent {
            if let Some(goal) = self.goals_config.goal_for_intent(intent) {
                let missing = state.missing_slots_for_goal(goal);
                for slot_name in missing {
                    if let Some(slot_def) = self.slots_config.get_slot(&slot_name) {
                        actions.push(NextBestAction {
                            action_type: ActionType::AskForSlot,
                            priority: slot_def.priority.unwrap_or(5),
                            prompt_suggestion: slot_def.prompt_template.clone()
                                .unwrap_or_else(|| format!("Please provide {}", slot_name)),
                            target_slot: Some(slot_name.to_string()),
                            tool_suggestion: None,
                        });
                    }
                }
            }
        }

        // Sort by priority
        actions.sort_by(|a, b| a.priority.cmp(&b.priority));
        actions
    }
}
```

---

### 3.1.4 Split ptt.rs (1316 lines → 4 files)

**Current File:** `crates/server/src/ptt.rs`

**Create new module structure:**
```
crates/server/src/ptt/
├── mod.rs              # Public API
├── audio.rs            # Audio processing
├── stt_pool.rs         # STT connection pooling
├── markdown.rs         # Markdown stripping
└── encoding.rs         # Base64 encoding utilities
```

---

## Task 3.2: Consolidate Duplicate Code

### 3.2.1 Create Shared PatternRegistry

**Problem:** Regex patterns duplicated in:
- `agent/src/dst/extractor.rs`
- `text_processing/src/intent/mod.rs`

**Solution:** Create shared `PatternRegistry` in text_processing crate.

**File:** `crates/text_processing/src/patterns/mod.rs` (NEW)

```rust
//! Shared pattern registry for slot/intent extraction
//!
//! Used by both intent detection and DST.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;

/// Amount pattern with multiplier
#[derive(Debug, Clone, Deserialize)]
pub struct AmountPattern {
    pub regex: String,
    pub multiplier: f64,
    #[serde(skip)]
    compiled: once_cell::sync::OnceCell<Regex>,
}

impl AmountPattern {
    pub fn compiled(&self) -> &Regex {
        self.compiled.get_or_init(|| {
            Regex::new(&self.regex).expect("Invalid regex pattern")
        })
    }
}

/// Shared pattern definitions
pub static AMOUNT_PATTERNS: Lazy<Vec<AmountPattern>> = Lazy::new(|| {
    vec![
        AmountPattern {
            regex: r"(?i)(\d+(?:\.\d+)?)\s*(?:crore|cr|करोड़)".to_string(),
            multiplier: 10_000_000.0,
            compiled: once_cell::sync::OnceCell::new(),
        },
        AmountPattern {
            regex: r"(?i)(\d+(?:\.\d+)?)\s*(?:lakh|lac|लाख)".to_string(),
            multiplier: 100_000.0,
            compiled: once_cell::sync::OnceCell::new(),
        },
        AmountPattern {
            regex: r"(?i)(\d+(?:\.\d+)?)\s*(?:thousand|k|हज़ार)".to_string(),
            multiplier: 1_000.0,
            compiled: once_cell::sync::OnceCell::new(),
        },
        AmountPattern {
            regex: r"(?:₹|rs\.?|rupees?)\s*(\d+(?:,\d+)*(?:\.\d+)?)".to_string(),
            multiplier: 1.0,
            compiled: once_cell::sync::OnceCell::new(),
        },
    ]
});

pub static PHONE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:\+91[\s-]?)?([6-9]\d{9})").unwrap()
});

/// Extract amount from text using shared patterns
pub fn extract_amount(text: &str) -> Option<(f64, f32)> {
    for pattern in AMOUNT_PATTERNS.iter() {
        if let Some(caps) = pattern.compiled().captures(text) {
            if let Some(num_str) = caps.get(1) {
                let clean = num_str.as_str().replace(",", "");
                if let Ok(num) = clean.parse::<f64>() {
                    return Some((num * pattern.multiplier, 0.9));
                }
            }
        }
    }
    None
}

/// Extract phone number from text
pub fn extract_phone(text: &str) -> Option<(String, f32)> {
    PHONE_PATTERN.captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| (m.as_str().to_string(), 0.95))
}
```

### 3.2.2 Update DST Extractor to Use Shared Patterns

**File:** `crates/agent/src/dst/extractor.rs`

```rust
// Remove duplicate pattern definitions
// Import from shared module
use voice_agent_text_processing::patterns::{extract_amount, extract_phone, AMOUNT_PATTERNS};

impl SlotExtractor {
    pub fn extract_amount(&self, text: &str) -> Option<(f64, f32)> {
        // Use shared function
        extract_amount(text)
    }

    pub fn extract_phone(&self, text: &str) -> Option<(String, f32)> {
        // Use shared function
        extract_phone(text)
    }
}
```

### 3.2.3 Create Phone Validator Utility

**Problem:** Phone validation duplicated in:
- `tools/src/gold_loan/tools/lead_capture.rs:91-93`
- `tools/src/gold_loan/tools/sms.rs:92-94`

**File:** `crates/tools/src/validation.rs` (NEW)

```rust
//! Common validation utilities for tools

use regex::Regex;
use once_cell::sync::Lazy;

static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[6-9]\d{9}$").unwrap()
});

/// Validate Indian mobile phone number
pub fn validate_indian_phone(phone: &str) -> Result<String, String> {
    let cleaned: String = phone.chars()
        .filter(|c| c.is_ascii_digit())
        .collect();

    // Remove country code if present
    let number = if cleaned.starts_with("91") && cleaned.len() == 12 {
        &cleaned[2..]
    } else {
        &cleaned
    };

    if PHONE_REGEX.is_match(number) {
        Ok(number.to_string())
    } else {
        Err(format!("Invalid phone number: {}", phone))
    }
}

/// Validate email address
pub fn validate_email(email: &str) -> Result<String, String> {
    static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
    });

    if EMAIL_REGEX.is_match(email) {
        Ok(email.to_lowercase())
    } else {
        Err(format!("Invalid email: {}", email))
    }
}
```

### 3.2.4 Create ID Generator Utility

**Problem:** UUID generation duplicated in multiple tools.

**File:** `crates/tools/src/id_generator.rs` (NEW)

```rust
//! ID generation utilities

use uuid::Uuid;

/// Generate prefixed ID
pub fn generate_id(prefix: &str) -> String {
    let uuid = Uuid::new_v4();
    let short = &uuid.to_string()[..8];
    format!("{}{}", prefix, short.to_uppercase())
}

/// ID prefixes for different entities
pub mod prefixes {
    pub const LEAD: &str = "LD";
    pub const APPOINTMENT: &str = "APT";
    pub const ESCALATION: &str = "ESC";
    pub const SMS: &str = "SMS";
    pub const SESSION: &str = "SES";
}

// Usage:
// let lead_id = generate_id(prefixes::LEAD);  // "LD1A2B3C4D"
```

---

## Task 3.3: Fix Crate Boundaries

### 3.3.1 Move Slot Extraction to text_processing

**Problem:** `agent/src/dst/extractor.rs` contains text processing logic.

**Solution:** Move to `text_processing/src/slot_extraction/`

**Steps:**
1. Create `crates/text_processing/src/slot_extraction/mod.rs`
2. Move extractor logic from agent crate
3. Update agent crate to use text_processing
4. Delete `agent/src/dst/extractor.rs`

**File:** `crates/text_processing/src/slot_extraction/mod.rs`
```rust
//! Slot extraction from user utterances

mod extractor;
mod patterns;

pub use extractor::{SlotExtractor, ConfigurableSlotExtractor, ExtractedSlots};
pub use patterns::ExtractionPatterns;
```

**Update agent/Cargo.toml:**
```toml
[dependencies]
voice-agent-text-processing = { path = "../text_processing" }
```

**Update agent/src/dst/mod.rs:**
```rust
// Remove local extractor
// mod extractor;  // DELETE

// Use from text_processing
use voice_agent_text_processing::slot_extraction::{SlotExtractor, ExtractedSlots};
```

### 3.3.2 Move Intent-to-Stage Mapping to Config

**Problem:** `agent/src/conversation.rs:620-717` has 100 lines of hardcoded intent→stage mappings.

**Solution:** Move to YAML config.

**File:** `config/domains/gold_loan/stages.yaml` (add section)

```yaml
# Add to existing stages.yaml
intent_stage_mappings:
  # Intent → Stage transitions
  greeting:
    from_stages: ["*"]
    to_stage: "discovery"

  loan_inquiry:
    from_stages: ["greeting", "discovery"]
    to_stage: "qualification"

  interest_rate_query:
    from_stages: ["discovery", "qualification"]
    to_stage: "presentation"

  eligibility_inquiry:
    from_stages: ["discovery", "qualification"]
    to_stage: "presentation"

  balance_transfer:
    from_stages: ["discovery", "qualification"]
    to_stage: "qualification"
    required_slots: ["current_lender"]

  objection_*:  # Wildcard for all objection intents
    from_stages: ["presentation"]
    to_stage: "objection_handling"

  appointment_request:
    from_stages: ["presentation", "objection_handling"]
    to_stage: "closing"

  farewell:
    from_stages: ["*"]
    to_stage: "farewell"
```

**Update StagesConfig:**
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct IntentStageMapping {
    pub from_stages: Vec<String>,  // "*" means any
    pub to_stage: String,
    pub required_slots: Option<Vec<String>>,
}

impl StagesConfig {
    pub fn next_stage_for_intent(
        &self,
        current_stage: &str,
        intent: &str,
    ) -> Option<&str> {
        self.intent_stage_mappings.get(intent)
            .filter(|mapping| {
                mapping.from_stages.contains(&"*".to_string()) ||
                mapping.from_stages.iter().any(|s| s == current_stage)
            })
            .map(|mapping| mapping.to_stage.as_str())
    }
}
```

### 3.3.3 Move AI Disclosure to Config

**Problem:** `agent/src/conversation.rs:251-265` has hardcoded AI disclosure in 8 languages.

**File:** `config/domains/gold_loan/compliance.yaml` (NEW)

```yaml
# Compliance-related configurations
ai_disclosure:
  required: true
  trigger: "start_of_conversation"

  messages:
    en: "I'm an AI assistant. This call may be recorded for quality purposes."
    hi: "Main ek AI assistant hoon. Yeh call quality ke liye record ho sakti hai."
    ta: "Naan oru AI udhaviyalar. Idhu tharam meipaatu kaaga pathivu seyyappadalam."
    te: "Nenu AI sahayakudu. Ee call quality kosam record avvachu."
    kn: "Naanu AI sahayaka. Ee call gunakke record aagabahudu."
    mr: "Mi ek AI sahayak ahe. Hi call quality sathi record hoU shakel."
    bn: "Ami ekti AI sahayak. Ei call quality'r jonno record hote pare."
    gu: "Hu ek AI sahayak chu. Aa call quality mate record thai shake che."

recording_disclosure:
  required: true
  messages:
    en: "This call is being recorded for training and quality purposes."
    hi: "Yeh call training aur quality ke liye record ki ja rahi hai."
```

**Update agent to use config:**
```rust
impl Conversation {
    fn get_ai_disclosure(&self, language: &str) -> String {
        self.compliance_config
            .ai_disclosure
            .messages
            .get(language)
            .or_else(|| self.compliance_config.ai_disclosure.messages.get("en"))
            .cloned()
            .unwrap_or_default()
    }
}
```

---

## Task 3.4: Extract Common Utilities

### 3.4.1 Create Shared Audio Utilities

**File:** `crates/core/src/audio/utils.rs` (NEW)

```rust
//! Common audio processing utilities

/// Resample audio to target sample rate
pub fn resample_linear(
    samples: &[f32],
    from_rate: u32,
    to_rate: u32,
) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }

    let ratio = to_rate as f64 / from_rate as f64;
    let new_len = (samples.len() as f64 * ratio) as usize;
    let mut output = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_idx = i as f64 / ratio;
        let idx0 = src_idx.floor() as usize;
        let idx1 = (idx0 + 1).min(samples.len() - 1);
        let frac = src_idx - idx0 as f64;

        let sample = samples[idx0] as f64 * (1.0 - frac) + samples[idx1] as f64 * frac;
        output.push(sample as f32);
    }

    output
}

/// Pad audio to frame size
pub fn pad_to_frame_size(samples: &[f32], frame_size: usize) -> Vec<f32> {
    if samples.len() >= frame_size {
        return samples.to_vec();
    }

    let mut padded = samples.to_vec();
    padded.resize(frame_size, 0.0);
    padded
}

/// Convert PCM16 bytes to f32 samples
pub fn pcm16_to_f32(bytes: &[u8]) -> Vec<f32> {
    bytes.chunks_exact(2)
        .map(|chunk| {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            sample as f32 / 32768.0
        })
        .collect()
}

/// Convert f32 samples to PCM16 bytes
pub fn f32_to_pcm16(samples: &[f32]) -> Vec<u8> {
    samples.iter()
        .flat_map(|&sample| {
            let clamped = sample.clamp(-1.0, 1.0);
            let i16_sample = (clamped * 32767.0) as i16;
            i16_sample.to_le_bytes()
        })
        .collect()
}
```

### 3.4.2 Create Time Utilities

**File:** `crates/core/src/utils/time.rs` (NEW)

```rust
//! Time-related utilities

use std::time::{SystemTime, UNIX_EPOCH};

/// Get current time in milliseconds since epoch
pub fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Get current time in seconds since epoch
pub fn current_time_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Format duration for display
pub fn format_duration_ms(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        let mins = ms / 60_000;
        let secs = (ms % 60_000) / 1000;
        format!("{}m {}s", mins, secs)
    }
}
```

---

## Phase 3 Completion Checklist

- [ ] 3.1.1 indicconformer.rs split into 4 files
- [ ] 3.1.2 intent/mod.rs split into 4 files
- [ ] 3.1.3 dst/slots.rs split into 3 files
- [ ] 3.1.4 ptt.rs split into 4 files
- [ ] 3.2.1 Shared PatternRegistry created
- [ ] 3.2.2 DST extractor uses shared patterns
- [ ] 3.2.3 Phone validator utility created
- [ ] 3.2.4 ID generator utility created
- [ ] 3.3.1 Slot extraction moved to text_processing
- [ ] 3.3.2 Intent-to-stage mapping moved to config
- [ ] 3.3.3 AI disclosure moved to config
- [ ] 3.4.1 Audio utilities extracted
- [ ] 3.4.2 Time utilities extracted

### Verification Commands
```bash
# Check all modules compile
cargo check --workspace

# Run tests
cargo test --workspace

# Check for remaining large files
find crates -name "*.rs" -exec wc -l {} \; | sort -rn | head -20

# Verify no duplicate patterns
grep -rn "crore|cr|करोड़" crates/ --include="*.rs" | wc -l  # Should be 1 location
```

---

## Dependencies for Phase 4

Phase 4 (Performance & Safety) can proceed independently.
