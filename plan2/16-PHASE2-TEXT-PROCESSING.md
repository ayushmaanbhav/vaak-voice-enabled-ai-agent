# Phase 2: Text Processing Pipeline Implementation

> **Priority:** P0 (Critical for Banking)
> **Duration:** 3 weeks
> **Dependencies:** Phase 1 (Core Traits)
> **Required For:** Production deployment

---

## Overview

This phase creates the `text_processing` crate with four major components:
1. **Grammar Correction** - Fix STT errors with domain context
2. **Translation** - Translate-Think-Translate pattern
3. **PII Detection** - Protect sensitive Indian data
4. **Compliance Checking** - Banking regulatory requirements

---

## 1. Crate Structure

### Create New Crate

```bash
# Create crate directory
mkdir -p voice-agent-rust/crates/text_processing/src/{grammar,translation,pii,compliance,simplify}
```

### Target Structure

```
crates/text_processing/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── grammar/
    │   ├── mod.rs
    │   ├── llm_corrector.rs
    │   └── noop.rs
    ├── translation/
    │   ├── mod.rs
    │   ├── indictrans.rs
    │   ├── grpc.rs
    │   └── detect.rs
    ├── pii/
    │   ├── mod.rs
    │   ├── detector.rs
    │   ├── patterns.rs
    │   └── redactor.rs
    ├── compliance/
    │   ├── mod.rs
    │   ├── checker.rs
    │   └── rules.rs
    └── simplify/
        ├── mod.rs
        └── tts_prep.rs
```

---

## 2. Cargo.toml

```toml
[package]
name = "text_processing"
version = "0.1.0"
edition = "2021"
description = "Text processing pipeline for voice agent"

[dependencies]
# Core types
voice-agent-core = { path = "../core" }

# Async
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
futures = "0.3"

# ONNX for translation
ort = { version = "2.0.0-rc.9", features = ["cuda", "coreml"] }

# NLP
regex = "1"
unicode-segmentation = "1.10"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# gRPC fallback
tonic = { version = "0.12", optional = true }

# Logging
tracing = "0.1"

# Error handling
thiserror = "1"
anyhow = "1"

[features]
default = []
grpc = ["tonic"]
```

---

## 3. Grammar Correction

### 3.1 grammar/mod.rs

```rust
//! Grammar correction module

mod llm_corrector;
mod noop;

pub use llm_corrector::LLMGrammarCorrector;
pub use noop::NoopCorrector;

use voice_agent_core::{GrammarCorrector, DomainContext};
use std::sync::Arc;

/// Create grammar corrector based on config
pub fn create_corrector(
    config: &GrammarConfig,
    llm: Option<Arc<dyn voice_agent_core::LanguageModel>>,
) -> Arc<dyn GrammarCorrector> {
    match config.provider {
        GrammarProvider::Llm => {
            if let Some(llm) = llm {
                Arc::new(LLMGrammarCorrector::new(llm, &config.domain))
            } else {
                tracing::warn!("LLM not available, using noop corrector");
                Arc::new(NoopCorrector)
            }
        }
        GrammarProvider::Disabled => Arc::new(NoopCorrector),
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GrammarConfig {
    pub provider: GrammarProvider,
    pub domain: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GrammarProvider {
    Llm,
    Disabled,
}

impl Default for GrammarConfig {
    fn default() -> Self {
        Self {
            provider: GrammarProvider::Disabled,
            domain: "gold_loan".to_string(),
            temperature: 0.1,
            max_tokens: 256,
        }
    }
}
```

### 3.2 grammar/llm_corrector.rs

```rust
//! LLM-based grammar correction

use async_trait::async_trait;
use futures::Stream;
use std::sync::Arc;
use voice_agent_core::{
    GrammarCorrector, DomainContext, LanguageModel,
    GenerateRequest, Message, Role, Result,
};

/// Grammar corrector using LLM
pub struct LLMGrammarCorrector {
    llm: Arc<dyn LanguageModel>,
    domain_context: DomainContext,
}

impl LLMGrammarCorrector {
    pub fn new(llm: Arc<dyn LanguageModel>, domain: &str) -> Self {
        let domain_context = match domain {
            "gold_loan" => DomainContext::gold_loan(),
            _ => DomainContext::default(),
        };

        Self { llm, domain_context }
    }

    /// Build grammar correction prompt
    fn build_prompt(&self, text: &str) -> String {
        format!(r#"You are a speech-to-text error corrector for a {} conversation.

DOMAIN VOCABULARY:
{}

COMMON PHRASES:
{}

RULES:
1. Fix obvious transcription errors
2. Preserve proper nouns and numbers exactly
3. Keep the meaning identical
4. Output ONLY the corrected text, nothing else
5. If text is already correct, output it unchanged
6. Handle Hindi-English code-switching naturally

INPUT: {}
CORRECTED:"#,
            self.domain_context.domain,
            self.domain_context.vocabulary.join(", "),
            self.domain_context.phrases.join("\n"),
            text,
        )
    }
}

#[async_trait]
impl GrammarCorrector for LLMGrammarCorrector {
    async fn correct(&self, text: &str, context: &DomainContext) -> Result<String> {
        let prompt = self.build_prompt(text);

        let request = GenerateRequest {
            messages: vec![Message {
                role: Role::User,
                content: prompt,
            }],
            max_tokens: Some(256),
            temperature: Some(0.1),
            stream: false,
            ..Default::default()
        };

        let response = self.llm.generate(request).await?;
        Ok(response.text.trim().to_string())
    }

    fn correct_stream<'a>(
        &'a self,
        text_stream: impl Stream<Item = String> + Send + 'a,
        context: &DomainContext,
    ) -> Box<dyn Stream<Item = Result<String>> + Send + Unpin + 'a> {
        use futures::StreamExt;

        let corrector = self.clone();
        let ctx = context.clone();

        Box::new(
            text_stream
                .then(move |text| {
                    let c = corrector.clone();
                    let ctx = ctx.clone();
                    async move { c.correct(&text, &ctx).await }
                })
                .boxed()
        )
    }

    fn is_enabled(&self) -> bool {
        true
    }
}

impl Clone for LLMGrammarCorrector {
    fn clone(&self) -> Self {
        Self {
            llm: self.llm.clone(),
            domain_context: self.domain_context.clone(),
        }
    }
}
```

### 3.3 grammar/noop.rs

```rust
//! No-op grammar corrector (pass-through)

use async_trait::async_trait;
use futures::Stream;
use voice_agent_core::{GrammarCorrector, DomainContext, Result};

/// Pass-through corrector that does nothing
pub struct NoopCorrector;

#[async_trait]
impl GrammarCorrector for NoopCorrector {
    async fn correct(&self, text: &str, _context: &DomainContext) -> Result<String> {
        Ok(text.to_string())
    }

    fn correct_stream<'a>(
        &'a self,
        text_stream: impl Stream<Item = String> + Send + 'a,
        _context: &DomainContext,
    ) -> Box<dyn Stream<Item = Result<String>> + Send + Unpin + 'a> {
        use futures::StreamExt;
        Box::new(text_stream.map(Ok).boxed())
    }

    fn is_enabled(&self) -> bool {
        false
    }
}
```

---

## 4. Translation

### 4.1 translation/mod.rs

```rust
//! Translation module with IndicTrans2 support

mod indictrans;
mod grpc;
mod detect;

pub use indictrans::IndicTranslator;
pub use grpc::GrpcTranslator;
pub use detect::ScriptDetector;

use voice_agent_core::{Translator, Language};
use std::sync::Arc;
use std::path::Path;

/// Create translator based on config
pub async fn create_translator(config: &TranslationConfig) -> Result<Arc<dyn Translator>, TranslationError> {
    match config.provider {
        TranslationProvider::Onnx => {
            match IndicTranslator::new(config).await {
                Ok(translator) => Ok(Arc::new(translator)),
                Err(e) if config.fallback_to_grpc => {
                    tracing::warn!("ONNX translator failed, falling back to gRPC: {}", e);
                    Ok(Arc::new(GrpcTranslator::new(&config.grpc_endpoint).await?))
                }
                Err(e) => Err(e),
            }
        }
        TranslationProvider::Grpc => {
            Ok(Arc::new(GrpcTranslator::new(&config.grpc_endpoint).await?))
        }
        TranslationProvider::Disabled => {
            Ok(Arc::new(NoopTranslator))
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TranslationConfig {
    pub provider: TranslationProvider,
    pub onnx_model_path: Option<String>,
    pub tokenizer_path: Option<String>,
    pub grpc_endpoint: String,
    pub fallback_to_grpc: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TranslationProvider {
    Onnx,
    Grpc,
    Disabled,
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            provider: TranslationProvider::Disabled,
            onnx_model_path: None,
            tokenizer_path: None,
            grpc_endpoint: "http://localhost:50051".to_string(),
            fallback_to_grpc: true,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error("ONNX model load failed: {0}")]
    OnnxLoadError(String),
    #[error("gRPC connection failed: {0}")]
    GrpcError(String),
    #[error("Translation failed: {0}")]
    TranslationFailed(String),
    #[error("Unsupported language pair: {from} -> {to}")]
    UnsupportedPair { from: String, to: String },
}

/// No-op translator
struct NoopTranslator;

#[async_trait::async_trait]
impl Translator for NoopTranslator {
    async fn translate(&self, text: &str, _from: Language, _to: Language) -> voice_agent_core::Result<String> {
        Ok(text.to_string())
    }

    async fn detect_language(&self, _text: &str) -> voice_agent_core::Result<Language> {
        Ok(Language::English)
    }

    fn translate_stream<'a>(
        &'a self,
        text_stream: impl futures::Stream<Item = String> + Send + 'a,
        _from: Language,
        _to: Language,
    ) -> Box<dyn futures::Stream<Item = voice_agent_core::Result<String>> + Send + Unpin + 'a> {
        use futures::StreamExt;
        Box::new(text_stream.map(Ok).boxed())
    }

    fn supports_pair(&self, _from: Language, _to: Language) -> bool {
        false
    }

    fn name(&self) -> &str {
        "noop"
    }
}
```

### 4.2 translation/indictrans.rs

```rust
//! IndicTrans2 ONNX translator

use async_trait::async_trait;
use futures::Stream;
use ort::{Session, SessionBuilder, GraphOptimizationLevel};
use std::path::Path;
use voice_agent_core::{Translator, Language, Result};
use super::{TranslationConfig, TranslationError, ScriptDetector};

/// IndicTrans2 translator using ONNX
pub struct IndicTranslator {
    encoder: Session,
    decoder: Session,
    tokenizer: IndicTransTokenizer,
    script_detector: ScriptDetector,
}

impl IndicTranslator {
    pub async fn new(config: &TranslationConfig) -> std::result::Result<Self, TranslationError> {
        let model_path = config.onnx_model_path.as_ref()
            .ok_or_else(|| TranslationError::OnnxLoadError("Model path not specified".into()))?;

        // Load encoder
        let encoder = SessionBuilder::new()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_model_from_file(format!("{}/encoder.onnx", model_path))
            .map_err(|e| TranslationError::OnnxLoadError(e.to_string()))?;

        // Load decoder
        let decoder = SessionBuilder::new()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_model_from_file(format!("{}/decoder.onnx", model_path))
            .map_err(|e| TranslationError::OnnxLoadError(e.to_string()))?;

        // Load tokenizer
        let tokenizer_path = config.tokenizer_path.as_ref()
            .ok_or_else(|| TranslationError::OnnxLoadError("Tokenizer path not specified".into()))?;
        let tokenizer = IndicTransTokenizer::load(tokenizer_path)
            .map_err(|e| TranslationError::OnnxLoadError(e.to_string()))?;

        Ok(Self {
            encoder,
            decoder,
            tokenizer,
            script_detector: ScriptDetector::new(),
        })
    }

    /// Supported language pairs
    fn supported_pairs() -> &'static [(Language, Language)] {
        &[
            // Indic to English
            (Language::Hindi, Language::English),
            (Language::Tamil, Language::English),
            (Language::Telugu, Language::English),
            (Language::Bengali, Language::English),
            (Language::Marathi, Language::English),
            (Language::Gujarati, Language::English),
            (Language::Kannada, Language::English),
            (Language::Malayalam, Language::English),
            (Language::Punjabi, Language::English),
            (Language::Odia, Language::English),
            // English to Indic
            (Language::English, Language::Hindi),
            (Language::English, Language::Tamil),
            (Language::English, Language::Telugu),
            (Language::English, Language::Bengali),
            (Language::English, Language::Marathi),
            (Language::English, Language::Gujarati),
            (Language::English, Language::Kannada),
            (Language::English, Language::Malayalam),
            (Language::English, Language::Punjabi),
            (Language::English, Language::Odia),
        ]
    }
}

#[async_trait]
impl Translator for IndicTranslator {
    async fn translate(&self, text: &str, from: Language, to: Language) -> Result<String> {
        // Skip if same language
        if from == to {
            return Ok(text.to_string());
        }

        // Check if pair is supported
        if !self.supports_pair(from, to) {
            return Err(voice_agent_core::Error::msg(format!(
                "Unsupported language pair: {:?} -> {:?}",
                from, to
            )));
        }

        // Tokenize
        let tokens = self.tokenizer.encode(text, from, to)?;

        // Run encoder
        let encoder_output = self.encoder.run(ort::inputs![tokens]?)?;

        // Run decoder
        let decoder_output = self.decoder.run(encoder_output)?;

        // Decode tokens
        let output_tokens = decoder_output[0].try_extract::<i64>()?;
        let translated = self.tokenizer.decode(&output_tokens.view().to_slice().unwrap(), to)?;

        Ok(translated)
    }

    async fn detect_language(&self, text: &str) -> Result<Language> {
        Ok(self.script_detector.detect(text))
    }

    fn translate_stream<'a>(
        &'a self,
        text_stream: impl Stream<Item = String> + Send + 'a,
        from: Language,
        to: Language,
    ) -> Box<dyn Stream<Item = Result<String>> + Send + Unpin + 'a> {
        use futures::StreamExt;

        Box::new(
            text_stream
                .then(move |text| async move {
                    self.translate(&text, from, to).await
                })
                .boxed()
        )
    }

    fn supports_pair(&self, from: Language, to: Language) -> bool {
        Self::supported_pairs().contains(&(from, to))
    }

    fn name(&self) -> &str {
        "indictrans2_onnx"
    }
}

/// IndicTrans2 tokenizer
struct IndicTransTokenizer {
    // Tokenizer implementation
}

impl IndicTransTokenizer {
    fn load(_path: &str) -> std::result::Result<Self, String> {
        // TODO: Implement tokenizer loading
        Ok(Self {})
    }

    fn encode(&self, _text: &str, _from: Language, _to: Language) -> Result<ndarray::Array2<i64>> {
        // TODO: Implement encoding
        todo!("Implement tokenizer encoding")
    }

    fn decode(&self, _tokens: &[i64], _lang: Language) -> Result<String> {
        // TODO: Implement decoding
        todo!("Implement tokenizer decoding")
    }
}
```

### 4.3 translation/detect.rs

```rust
//! Script and language detection

use voice_agent_core::{Language, Script};
use std::collections::HashMap;

/// Script-based language detector
pub struct ScriptDetector {
    script_map: HashMap<Script, Language>,
}

impl ScriptDetector {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert(Script::Devanagari, Language::Hindi);
        map.insert(Script::Tamil, Language::Tamil);
        map.insert(Script::Telugu, Language::Telugu);
        map.insert(Script::Kannada, Language::Kannada);
        map.insert(Script::Malayalam, Language::Malayalam);
        map.insert(Script::Bengali, Language::Bengali);
        map.insert(Script::Gujarati, Language::Gujarati);
        map.insert(Script::Gurmukhi, Language::Punjabi);
        map.insert(Script::Odia, Language::Odia);
        map.insert(Script::Arabic, Language::Urdu);
        map.insert(Script::Latin, Language::English);

        Self { script_map: map }
    }

    /// Detect language from text
    pub fn detect(&self, text: &str) -> Language {
        let script = self.detect_script(text);
        self.script_map.get(&script).copied().unwrap_or(Language::English)
    }

    /// Detect dominant script in text
    fn detect_script(&self, text: &str) -> Script {
        let mut counts: HashMap<Script, usize> = HashMap::new();

        for c in text.chars() {
            let script = self.char_to_script(c);
            if script != Script::Latin || c.is_ascii_alphabetic() {
                *counts.entry(script).or_insert(0) += 1;
            }
        }

        counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(script, _)| script)
            .unwrap_or(Script::Latin)
    }

    /// Map character to script
    fn char_to_script(&self, c: char) -> Script {
        let code = c as u32;
        match code {
            0x0000..=0x007F => Script::Latin,
            0x0900..=0x097F => Script::Devanagari,
            0x0980..=0x09FF => Script::Bengali,
            0x0A00..=0x0A7F => Script::Gurmukhi,
            0x0A80..=0x0AFF => Script::Gujarati,
            0x0B00..=0x0B7F => Script::Odia,
            0x0B80..=0x0BFF => Script::Tamil,
            0x0C00..=0x0C7F => Script::Telugu,
            0x0C80..=0x0CFF => Script::Kannada,
            0x0D00..=0x0D7F => Script::Malayalam,
            0x0600..=0x06FF => Script::Arabic,
            _ => Script::Latin,
        }
    }
}

impl Default for ScriptDetector {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## 5. PII Detection

### 5.1 pii/mod.rs

```rust
//! PII detection and redaction module

mod detector;
mod patterns;
mod redactor;

pub use detector::HybridPIIDetector;
pub use patterns::IndianPIIPatterns;
pub use redactor::PIIRedactor as PIIRedactorImpl;

use voice_agent_core::PIIRedactor;
use std::sync::Arc;

/// Create PII detector based on config
pub fn create_detector(config: &PIIConfig) -> Arc<dyn PIIRedactor> {
    match config.provider {
        PIIProvider::Hybrid => Arc::new(HybridPIIDetector::new(&config.entities)),
        PIIProvider::Regex => Arc::new(HybridPIIDetector::regex_only(&config.entities)),
        PIIProvider::Disabled => Arc::new(NoopDetector),
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PIIConfig {
    pub provider: PIIProvider,
    pub entities: Vec<String>,
    pub strategy: voice_agent_core::RedactionStrategy,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PIIProvider {
    Hybrid,
    Regex,
    Disabled,
}

impl Default for PIIConfig {
    fn default() -> Self {
        Self {
            provider: PIIProvider::Regex,
            entities: vec![
                "Aadhaar".to_string(),
                "PAN".to_string(),
                "PhoneNumber".to_string(),
                "Email".to_string(),
            ],
            strategy: voice_agent_core::RedactionStrategy::PartialMask { visible_chars: 4 },
        }
    }
}

/// No-op detector
struct NoopDetector;

#[async_trait::async_trait]
impl PIIRedactor for NoopDetector {
    async fn detect(&self, _text: &str) -> voice_agent_core::Result<Vec<voice_agent_core::PIIEntity>> {
        Ok(vec![])
    }

    async fn redact(&self, text: &str, _strategy: &voice_agent_core::RedactionStrategy) -> voice_agent_core::Result<String> {
        Ok(text.to_string())
    }

    fn supported_types(&self) -> &[voice_agent_core::PIIType] {
        &[]
    }
}
```

### 5.2 pii/patterns.rs

```rust
//! India-specific PII regex patterns

use regex::Regex;
use voice_agent_core::PIIType;
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Compiled PII patterns for India
pub static INDIAN_PII_PATTERNS: Lazy<HashMap<PIIType, Regex>> = Lazy::new(|| {
    let mut patterns = HashMap::new();

    // Aadhaar: 12 digits, often with spaces (XXXX XXXX XXXX)
    patterns.insert(
        PIIType::Aadhaar,
        Regex::new(r"\b\d{4}\s?\d{4}\s?\d{4}\b").unwrap(),
    );

    // PAN: 5 letters, 4 digits, 1 letter (ABCDE1234F)
    patterns.insert(
        PIIType::PAN,
        Regex::new(r"\b[A-Z]{5}[0-9]{4}[A-Z]\b").unwrap(),
    );

    // Indian phone: +91 or 0 followed by 10 digits starting with 6-9
    patterns.insert(
        PIIType::PhoneNumber,
        Regex::new(r"(?:\+91[\-\s]?)?[0]?[6-9]\d{9}").unwrap(),
    );

    // Email
    patterns.insert(
        PIIType::Email,
        Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
    );

    // IFSC: 4 letters, 0, 6 alphanumeric (SBIN0001234)
    patterns.insert(
        PIIType::IFSC,
        Regex::new(r"\b[A-Z]{4}0[A-Z0-9]{6}\b").unwrap(),
    );

    // Bank Account: 9-18 digits
    patterns.insert(
        PIIType::BankAccount,
        Regex::new(r"\b\d{9,18}\b").unwrap(),
    );

    // Voter ID: 3 letters followed by 7 digits
    patterns.insert(
        PIIType::VoterId,
        Regex::new(r"\b[A-Z]{3}\d{7}\b").unwrap(),
    );

    // Driving License: State code + 13 alphanumeric
    patterns.insert(
        PIIType::DrivingLicense,
        Regex::new(r"\b[A-Z]{2}\d{2}\s?\d{4}\s?\d{7}\b").unwrap(),
    );

    // Passport: Letter + 7 digits
    patterns.insert(
        PIIType::Passport,
        Regex::new(r"\b[A-Z]\d{7}\b").unwrap(),
    );

    patterns
});

/// Get pattern for PII type
pub fn get_pattern(pii_type: PIIType) -> Option<&'static Regex> {
    INDIAN_PII_PATTERNS.get(&pii_type)
}

/// Check if text matches any PII pattern
pub fn find_matches(text: &str, pii_type: PIIType) -> Vec<(usize, usize, String)> {
    if let Some(pattern) = get_pattern(pii_type) {
        pattern
            .find_iter(text)
            .map(|m| (m.start(), m.end(), m.as_str().to_string()))
            .collect()
    } else {
        vec![]
    }
}
```

### 5.3 pii/detector.rs

```rust
//! Hybrid PII detector (regex + NER)

use async_trait::async_trait;
use voice_agent_core::{PIIRedactor, PIIEntity, PIIType, RedactionStrategy, Result};
use super::patterns::{INDIAN_PII_PATTERNS, find_matches};
use std::collections::HashSet;

/// Hybrid PII detector using regex and optional NER
pub struct HybridPIIDetector {
    enabled_types: HashSet<PIIType>,
    use_ner: bool,
}

impl HybridPIIDetector {
    /// Create with both regex and NER
    pub fn new(entity_names: &[String]) -> Self {
        Self {
            enabled_types: parse_entity_names(entity_names),
            use_ner: true,
        }
    }

    /// Create with regex only
    pub fn regex_only(entity_names: &[String]) -> Self {
        Self {
            enabled_types: parse_entity_names(entity_names),
            use_ner: false,
        }
    }

    /// Detect using regex patterns
    fn detect_regex(&self, text: &str) -> Vec<PIIEntity> {
        let mut entities = Vec::new();

        for pii_type in &self.enabled_types {
            if let Some(pattern) = INDIAN_PII_PATTERNS.get(pii_type) {
                for capture in pattern.find_iter(text) {
                    entities.push(PIIEntity {
                        pii_type: *pii_type,
                        text: capture.as_str().to_string(),
                        start: capture.start(),
                        end: capture.end(),
                        confidence: 0.95, // Regex matches are high confidence
                    });
                }
            }
        }

        entities
    }

    /// Detect using NER (for names, addresses)
    async fn detect_ner(&self, _text: &str) -> Vec<PIIEntity> {
        // TODO: Implement NER-based detection using rust-bert
        vec![]
    }

    /// Merge and deduplicate detections
    fn merge_detections(&self, mut entities: Vec<PIIEntity>) -> Vec<PIIEntity> {
        // Sort by start position
        entities.sort_by_key(|e| e.start);

        // Remove overlaps, keeping higher confidence
        let mut result = Vec::new();
        for entity in entities {
            if let Some(last) = result.last_mut() {
                if entity.start < last.end {
                    // Overlap - keep higher confidence
                    if entity.confidence > last.confidence {
                        *last = entity;
                    }
                    continue;
                }
            }
            result.push(entity);
        }

        result
    }
}

#[async_trait]
impl PIIRedactor for HybridPIIDetector {
    async fn detect(&self, text: &str) -> Result<Vec<PIIEntity>> {
        let mut entities = self.detect_regex(text);

        if self.use_ner {
            entities.extend(self.detect_ner(text).await);
        }

        Ok(self.merge_detections(entities))
    }

    async fn redact(&self, text: &str, strategy: &RedactionStrategy) -> Result<String> {
        let entities = self.detect(text).await?;

        let mut result = text.to_string();

        // Apply redactions in reverse order to preserve indices
        for entity in entities.into_iter().rev() {
            let replacement = match strategy {
                RedactionStrategy::Mask => "[REDACTED]".to_string(),
                RedactionStrategy::TypeMask => format!("[{:?}]", entity.pii_type),
                RedactionStrategy::PartialMask { visible_chars } => {
                    partial_mask(&entity.text, *visible_chars)
                }
                RedactionStrategy::Remove => String::new(),
                RedactionStrategy::Synthesize => synthesize_fake(entity.pii_type),
            };

            result.replace_range(entity.start..entity.end, &replacement);
        }

        Ok(result)
    }

    fn supported_types(&self) -> &[PIIType] {
        // Return as static slice
        static TYPES: &[PIIType] = &[
            PIIType::Aadhaar,
            PIIType::PAN,
            PIIType::PhoneNumber,
            PIIType::Email,
            PIIType::IFSC,
            PIIType::BankAccount,
        ];
        TYPES
    }
}

/// Parse entity names to PIIType
fn parse_entity_names(names: &[String]) -> HashSet<PIIType> {
    names
        .iter()
        .filter_map(|name| match name.to_lowercase().as_str() {
            "aadhaar" => Some(PIIType::Aadhaar),
            "pan" => Some(PIIType::PAN),
            "phone" | "phonenumber" => Some(PIIType::PhoneNumber),
            "email" => Some(PIIType::Email),
            "ifsc" => Some(PIIType::IFSC),
            "bankaccount" => Some(PIIType::BankAccount),
            "voterid" => Some(PIIType::VoterId),
            "passport" => Some(PIIType::Passport),
            _ => None,
        })
        .collect()
}

/// Partial mask: 98****1234
fn partial_mask(text: &str, visible: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= visible * 2 {
        return "*".repeat(chars.len());
    }

    let prefix: String = chars[..visible].iter().collect();
    let suffix: String = chars[chars.len() - visible..].iter().collect();
    let middle = "*".repeat(chars.len() - visible * 2);

    format!("{}{}{}", prefix, middle, suffix)
}

/// Generate fake replacement
fn synthesize_fake(pii_type: PIIType) -> String {
    match pii_type {
        PIIType::PhoneNumber => "9999999999".to_string(),
        PIIType::Email => "user@example.com".to_string(),
        PIIType::Aadhaar => "XXXX XXXX XXXX".to_string(),
        PIIType::PAN => "XXXXX0000X".to_string(),
        _ => "[REDACTED]".to_string(),
    }
}
```

---

## 6. Compliance Checking

### 6.1 compliance/mod.rs

```rust
//! Compliance checking module

mod checker;
mod rules;

pub use checker::RuleBasedComplianceChecker;
pub use rules::{ComplianceRules, load_rules};

use voice_agent_core::ComplianceChecker;
use std::sync::Arc;
use std::path::Path;

/// Create compliance checker
pub fn create_checker(config: &ComplianceConfig) -> Arc<dyn ComplianceChecker> {
    match config.provider {
        ComplianceProvider::RuleBased => {
            match RuleBasedComplianceChecker::from_config(&config.rules_file) {
                Ok(checker) => Arc::new(checker),
                Err(e) => {
                    tracing::error!("Failed to load compliance rules: {}", e);
                    Arc::new(NoopChecker)
                }
            }
        }
        ComplianceProvider::Disabled => Arc::new(NoopChecker),
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ComplianceConfig {
    pub provider: ComplianceProvider,
    pub rules_file: String,
    pub strict_mode: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComplianceProvider {
    RuleBased,
    Disabled,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            provider: ComplianceProvider::Disabled,
            rules_file: "compliance.toml".to_string(),
            strict_mode: false,
        }
    }
}

/// No-op checker
struct NoopChecker;

#[async_trait::async_trait]
impl ComplianceChecker for NoopChecker {
    async fn check(&self, _text: &str) -> voice_agent_core::Result<voice_agent_core::ComplianceResult> {
        Ok(voice_agent_core::ComplianceResult {
            is_compliant: true,
            violations: vec![],
            required_additions: vec![],
            suggested_rewrites: vec![],
        })
    }

    async fn make_compliant(&self, text: &str) -> voice_agent_core::Result<String> {
        Ok(text.to_string())
    }

    fn rules_version(&self) -> &str {
        "noop"
    }
}
```

### 6.2 compliance/checker.rs

```rust
//! Rule-based compliance checker

use async_trait::async_trait;
use regex::Regex;
use voice_agent_core::{
    ComplianceChecker, ComplianceResult, ComplianceViolation, Severity, SuggestedRewrite, Result,
};
use super::rules::ComplianceRules;
use std::path::Path;

/// Rule-based compliance checker
pub struct RuleBasedComplianceChecker {
    rules: ComplianceRules,
    forbidden_patterns: Vec<Regex>,
    version: String,
}

impl RuleBasedComplianceChecker {
    pub fn from_config(path: &str) -> std::result::Result<Self, String> {
        let rules = super::rules::load_rules(path)?;

        let forbidden_patterns = rules
            .forbidden_phrases
            .iter()
            .filter_map(|phrase| {
                Regex::new(&format!(r"(?i)\b{}\b", regex::escape(phrase))).ok()
            })
            .collect();

        Ok(Self {
            rules,
            forbidden_patterns,
            version: "1.0".to_string(),
        })
    }
}

#[async_trait]
impl ComplianceChecker for RuleBasedComplianceChecker {
    async fn check(&self, text: &str) -> Result<ComplianceResult> {
        let mut violations = Vec::new();
        let mut required_additions = Vec::new();

        // Check forbidden phrases
        for (i, pattern) in self.forbidden_patterns.iter().enumerate() {
            if let Some(m) = pattern.find(text) {
                violations.push(ComplianceViolation {
                    rule_id: format!("FORBIDDEN_{}", i),
                    description: format!(
                        "Forbidden phrase detected: '{}'",
                        &self.rules.forbidden_phrases[i]
                    ),
                    severity: Severity::Critical,
                    text_span: (m.start(), m.end()),
                });
            }
        }

        // Check claims requiring disclaimers
        for rule in &self.rules.claims_requiring_disclaimer {
            if let Ok(pattern) = Regex::new(&rule.pattern) {
                if pattern.is_match(text) && !text.contains(&rule.disclaimer) {
                    required_additions.push(rule.disclaimer.clone());
                }
            }
        }

        // Check rate accuracy
        if let Ok(rate_pattern) = Regex::new(r"(\d+(?:\.\d+)?)\s*%") {
            if let Some(rate_match) = rate_pattern.find(text) {
                let rate_str = rate_match.as_str().trim_end_matches('%').trim();
                if let Ok(rate) = rate_str.parse::<f32>() {
                    if rate < self.rules.rate_rules.min_rate || rate > self.rules.rate_rules.max_rate {
                        violations.push(ComplianceViolation {
                            rule_id: "RATE_ACCURACY".to_string(),
                            description: format!(
                                "Rate {}% outside valid range ({}-{}%)",
                                rate,
                                self.rules.rate_rules.min_rate,
                                self.rules.rate_rules.max_rate
                            ),
                            severity: Severity::Error,
                            text_span: (rate_match.start(), rate_match.end()),
                        });
                    }
                }
            }
        }

        Ok(ComplianceResult {
            is_compliant: violations.iter().all(|v| v.severity != Severity::Critical),
            violations,
            required_additions,
            suggested_rewrites: Vec::new(),
        })
    }

    async fn make_compliant(&self, text: &str) -> Result<String> {
        let result = self.check(text).await?;

        if result.is_compliant && result.required_additions.is_empty() {
            return Ok(text.to_string());
        }

        let mut compliant_text = text.to_string();

        // Remove critical violations (replace with safe text)
        for violation in result.violations.iter().filter(|v| v.severity == Severity::Critical) {
            compliant_text.replace_range(
                violation.text_span.0..violation.text_span.1,
                "[content removed]",
            );
        }

        // Add required disclaimers
        for addition in &result.required_additions {
            compliant_text.push(' ');
            compliant_text.push_str(addition);
        }

        Ok(compliant_text)
    }

    fn rules_version(&self) -> &str {
        &self.version
    }
}
```

---

## 7. Checklist

### 7.1 Crate Setup
- [ ] Create `crates/text_processing/Cargo.toml`
- [ ] Create `crates/text_processing/src/lib.rs`
- [ ] Add to workspace `Cargo.toml`

### 7.2 Grammar Correction
- [ ] Implement `LLMGrammarCorrector`
- [ ] Implement `NoopCorrector`
- [ ] Add factory function
- [ ] Add unit tests

### 7.3 Translation
- [ ] Implement `IndicTranslator` (ONNX)
- [ ] Implement `GrpcTranslator` (fallback)
- [ ] Implement `ScriptDetector`
- [ ] Add tokenizer implementation
- [ ] Add factory function
- [ ] Add unit tests

### 7.4 PII Detection
- [ ] Implement `HybridPIIDetector`
- [ ] Add India-specific regex patterns
- [ ] Implement redaction strategies
- [ ] Add factory function
- [ ] Add unit tests

### 7.5 Compliance
- [ ] Implement `RuleBasedComplianceChecker`
- [ ] Create compliance rules format
- [ ] Create sample `compliance.toml`
- [ ] Add factory function
- [ ] Add unit tests

### 7.6 Integration
- [ ] Create unified `TextProcessingPipeline`
- [ ] Wire into agent flow
- [ ] Add configuration loading
- [ ] Add integration tests

---

*This phase is critical for banking deployment. PII protection and compliance are regulatory requirements.*
