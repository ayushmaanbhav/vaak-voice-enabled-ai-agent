# Multilingual Support: Approach Comparison & Pluggable Design

> **Status:** Design Document
> **Purpose:** Compare native vs translate-first approaches with pluggable architecture
> **Outcome:** Switchable implementations via configuration

---

## Executive Summary

Based on agent analysis of the codebase, we identified:

| Component | Pluggable? | Gap |
|-----------|------------|-----|
| **LLM Backend** | ✅ Trait-based | None |
| **RAG Retriever** | ✅ Builder pattern | None |
| **Tool Registry** | ✅ Plugin pattern | None |
| **Slot Extraction** | ❌ Hardcoded | **CRITICAL** |
| **Text Processing Pipeline** | ❌ Not implemented | **CRITICAL** |
| **Translation** | ❌ Not implemented | **CRITICAL** |

**Key Finding:** The documented `TextProcessor` trait and translation pipeline exist only in docs, not in code. Slot extraction in `IntentDetector` is hardcoded with no trait interface.

---

## Two Approaches Compared

### Approach 1: Native Language Processing (Current Plan)

```
User speaks Tamil: "ஐந்து லட்சம் loan வேண்டும்"
    ↓
STT (IndicConformer) → "ஐந்து லட்சம் loan வேண்டும்"
    ↓
Script Detection → Tamil
    ↓
Native Slot Extraction:
  - Normalize numerals: ௫ → 5
  - Match multiplier: லட்சம் → 100,000
  - Extract amount: 500,000
    ↓
Intent + Slots → LLM (multilingual prompt)
    ↓
Response in Tamil
```

**Pros:**
- No translation latency
- Preserves nuance and cultural context
- Works offline (no translation model)

**Cons:**
- Need patterns for all 22 languages
- Complex regex maintenance
- Harder to test/debug

---

### Approach 2: Translate-Think-Translate (Architecture Doc Pattern)

```
User speaks Tamil: "ஐந்து லட்சம் loan வேண்டும்"
    ↓
STT (IndicConformer) → "ஐந்து லட்சம் loan வேண்டும்"
    ↓
Translate (Tamil → English): "I need five lakh loan"
    ↓
Simple English Slot Extraction:
  - Match: "five lakh" → 500,000
  - Extract amount: 500,000
    ↓
Intent + Slots → LLM (English - better reasoning)
    ↓
Response in English: "Your eligibility is..."
    ↓
Translate (English → Tamil): "உங்கள் தகுதி..."
    ↓
TTS in Tamil
```

**Pros:**
- Simple English-only extraction patterns
- LLM reasons better in English
- Single codebase for all languages
- Easier to test/debug

**Cons:**
- Added latency (~100-200ms per translation)
- Translation errors can compound
- Requires translation model (IndicTrans2)

---

## Pluggable Architecture Design

Following the existing codebase patterns (strategy enums, traits, config), here's the proposed design:

### 1. Strategy Enum (Config-Driven)

```rust
// crates/config/src/agent.rs

/// Multilingual processing strategy
#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MultilingualStrategy {
    /// Process in native language (Approach 1)
    /// - Use language-specific patterns for slot extraction
    /// - LLM prompt in native language or English
    #[default]
    Native,

    /// Translate to English first (Approach 2)
    /// - Translate input to English before processing
    /// - Use English patterns for slot extraction
    /// - Translate response back to user's language
    TranslateFirst,

    /// Hybrid: Try native, fallback to translate
    /// - Attempt native extraction first
    /// - If confidence low, translate and retry
    Hybrid {
        confidence_threshold: f32,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct MultilingualConfig {
    /// Processing strategy
    pub strategy: MultilingualStrategy,

    /// Translation provider (for TranslateFirst/Hybrid)
    pub translation_provider: TranslationProvider,

    /// Fallback to native if translation fails
    pub fallback_to_native: bool,

    /// Languages to always use native processing
    pub native_only_languages: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TranslationProvider {
    #[default]
    Onnx,
    Grpc,
    Disabled,
}
```

### 2. Slot Extractor Trait (New)

```rust
// crates/agent/src/extraction/mod.rs

use async_trait::async_trait;

/// Core trait for slot extraction - enables pluggable implementations
#[async_trait]
pub trait SlotExtractor: Send + Sync {
    /// Extract all slots from text
    async fn extract_slots(
        &self,
        text: &str,
        language: &str,
        context: &ExtractionContext,
    ) -> Result<HashMap<String, SlotValue>, ExtractionError>;

    /// Extract specific slot type
    async fn extract_slot(
        &self,
        text: &str,
        slot_type: SlotType,
        language: &str,
    ) -> Result<Option<SlotValue>, ExtractionError>;

    /// Name for logging/metrics
    fn name(&self) -> &'static str;

    /// Supported languages
    fn supported_languages(&self) -> &[&str];
}

#[derive(Debug, Clone)]
pub struct ExtractionContext {
    pub domain: String,
    pub conversation_stage: Option<String>,
    pub previous_slots: HashMap<String, SlotValue>,
}

#[derive(Debug, Clone)]
pub enum SlotValue {
    Amount(f64),
    Phone(String),
    Text(String),
    Number(i64),
    Date(String),
    Location(String),
}
```

### 3. Native Extractor Implementation (Approach 1)

```rust
// crates/agent/src/extraction/native.rs

/// Native language slot extraction (Approach 1)
/// Uses language-specific patterns without translation
pub struct NativeSlotExtractor {
    /// Patterns per language
    patterns: HashMap<String, Vec<CompiledSlotPattern>>,
    /// Indic numeral normalizer
    numeral_normalizer: IndicNumeralNormalizer,
    /// Multilingual multiplier words
    multiplier_words: MultilingualMultipliers,
}

impl NativeSlotExtractor {
    pub fn new() -> Self {
        Self {
            patterns: compile_all_language_patterns(),
            numeral_normalizer: IndicNumeralNormalizer::new(),
            multiplier_words: MultilingualMultipliers::new(),
        }
    }
}

#[async_trait]
impl SlotExtractor for NativeSlotExtractor {
    async fn extract_slots(
        &self,
        text: &str,
        language: &str,
        context: &ExtractionContext,
    ) -> Result<HashMap<String, SlotValue>, ExtractionError> {
        // 1. Normalize Indic numerals to ASCII
        let normalized = self.numeral_normalizer.normalize(text);

        // 2. Get language-specific patterns (fallback to generic)
        let patterns = self.patterns
            .get(language)
            .or_else(|| self.patterns.get("generic"))
            .ok_or(ExtractionError::UnsupportedLanguage(language.into()))?;

        // 3. Extract using patterns
        let mut slots = HashMap::new();

        for pattern in patterns {
            if let Some(value) = pattern.extract(&normalized)? {
                slots.insert(pattern.slot_name.clone(), value);
            }
        }

        // 4. Extract amounts using multilingual multipliers
        if let Some(amount) = self.multiplier_words.extract_amount(&normalized, language)? {
            slots.insert("loan_amount".into(), SlotValue::Amount(amount));
        }

        Ok(slots)
    }

    fn name(&self) -> &'static str {
        "native_slot_extractor"
    }

    fn supported_languages(&self) -> &[&str] {
        &["hi", "ta", "te", "bn", "kn", "ml", "gu", "mr", "pa", "or", "en"]
    }
}
```

### 4. Translate-First Extractor Implementation (Approach 2)

```rust
// crates/agent/src/extraction/translate_first.rs

/// Translate-then-extract slot extraction (Approach 2)
/// Translates to English, extracts with simple patterns, translates back
pub struct TranslateFirstExtractor {
    /// Translator (IndicTrans2 via ONNX or gRPC)
    translator: Arc<dyn Translator>,
    /// Simple English-only extractor
    english_extractor: EnglishSlotExtractor,
    /// Cache for repeated translations
    translation_cache: RwLock<LruCache<String, String>>,
}

impl TranslateFirstExtractor {
    pub fn new(translator: Arc<dyn Translator>) -> Self {
        Self {
            translator,
            english_extractor: EnglishSlotExtractor::new(),
            translation_cache: RwLock::new(LruCache::new(1000)),
        }
    }
}

#[async_trait]
impl SlotExtractor for TranslateFirstExtractor {
    async fn extract_slots(
        &self,
        text: &str,
        language: &str,
        context: &ExtractionContext,
    ) -> Result<HashMap<String, SlotValue>, ExtractionError> {
        // Skip translation if already English
        if language == "en" {
            return self.english_extractor.extract_slots(text, "en", context).await;
        }

        // 1. Check translation cache
        let cache_key = format!("{}:{}", language, text);
        if let Some(cached) = self.translation_cache.read().get(&cache_key) {
            return self.english_extractor.extract_slots(cached, "en", context).await;
        }

        // 2. Translate to English
        let english_text = self.translator
            .translate(text, language, "en")
            .await
            .map_err(|e| ExtractionError::TranslationFailed(e.to_string()))?;

        // 3. Cache translation
        self.translation_cache.write().put(cache_key, english_text.clone());

        // 4. Extract from English text
        self.english_extractor.extract_slots(&english_text, "en", context).await
    }

    fn name(&self) -> &'static str {
        "translate_first_extractor"
    }

    fn supported_languages(&self) -> &[&str] {
        // Supports all IndicTrans2 languages
        &["hi", "ta", "te", "bn", "kn", "ml", "gu", "mr", "pa", "or",
          "as", "brx", "doi", "kok", "ks", "mai", "mni", "ne", "sa", "sat", "sd", "ur"]
    }
}
```

### 5. Hybrid Extractor (Best of Both)

```rust
// crates/agent/src/extraction/hybrid.rs

/// Hybrid extractor: try native first, fallback to translate
pub struct HybridExtractor {
    native: NativeSlotExtractor,
    translate_first: TranslateFirstExtractor,
    confidence_threshold: f32,
}

#[async_trait]
impl SlotExtractor for HybridExtractor {
    async fn extract_slots(
        &self,
        text: &str,
        language: &str,
        context: &ExtractionContext,
    ) -> Result<HashMap<String, SlotValue>, ExtractionError> {
        // 1. Try native extraction first
        let native_result = self.native.extract_slots(text, language, context).await;

        match native_result {
            Ok(slots) if self.has_high_confidence(&slots) => {
                tracing::debug!("Native extraction succeeded with high confidence");
                Ok(slots)
            }
            Ok(slots) => {
                // Low confidence - try translate-first
                tracing::debug!("Native extraction low confidence, trying translate-first");
                match self.translate_first.extract_slots(text, language, context).await {
                    Ok(translated_slots) => {
                        // Merge results, preferring translated for amounts
                        Ok(self.merge_slots(slots, translated_slots))
                    }
                    Err(_) => Ok(slots), // Fallback to native result
                }
            }
            Err(_) => {
                // Native failed - try translate-first
                self.translate_first.extract_slots(text, language, context).await
            }
        }
    }

    fn name(&self) -> &'static str {
        "hybrid_extractor"
    }

    fn supported_languages(&self) -> &[&str] {
        self.translate_first.supported_languages() // Widest coverage
    }
}
```

### 6. Factory Function (Config-Driven Selection)

```rust
// crates/agent/src/extraction/mod.rs

/// Create slot extractor based on configuration
pub fn create_extractor(
    config: &MultilingualConfig,
    translator: Option<Arc<dyn Translator>>,
) -> Result<Arc<dyn SlotExtractor>, ExtractionError> {
    match config.strategy {
        MultilingualStrategy::Native => {
            Ok(Arc::new(NativeSlotExtractor::new()))
        }

        MultilingualStrategy::TranslateFirst => {
            let translator = translator
                .ok_or(ExtractionError::TranslatorRequired)?;
            Ok(Arc::new(TranslateFirstExtractor::new(translator)))
        }

        MultilingualStrategy::Hybrid { confidence_threshold } => {
            let translator = translator
                .ok_or(ExtractionError::TranslatorRequired)?;
            Ok(Arc::new(HybridExtractor::new(
                NativeSlotExtractor::new(),
                TranslateFirstExtractor::new(translator),
                confidence_threshold,
            )))
        }
    }
}
```

### 7. Configuration Example

```yaml
# config/default.yaml

agent:
  multilingual:
    # Options: native, translate_first, hybrid
    strategy: hybrid

    # For translate_first and hybrid
    translation_provider: onnx  # or grpc

    # Fallback behavior
    fallback_to_native: true

    # Languages that should always use native
    native_only_languages: ["en", "hi"]

    # Hybrid-specific config
    hybrid:
      confidence_threshold: 0.7

  # Translation model paths (for ONNX provider)
  translation:
    onnx_model_path: "models/indictrans2/indic_en.onnx"
    tokenizer_path: "models/indictrans2/tokenizer"

    # gRPC fallback
    grpc_endpoint: "http://localhost:50051"
```

---

## Integration with Existing IntentDetector

### Current Code (Hardcoded)

```rust
// crates/agent/src/intent.rs:422-437 (CURRENT)

pub fn extract_slots(&self, text: &str) -> HashMap<String, String> {
    let mut slots = HashMap::new();
    // Hardcoded extraction logic...
}
```

### Proposed Change (Pluggable)

```rust
// crates/agent/src/intent.rs (PROPOSED)

pub struct IntentDetector {
    intents: RwLock<Vec<Intent>>,
    /// Pluggable slot extractor
    slot_extractor: Arc<dyn SlotExtractor>,
}

impl IntentDetector {
    pub fn new(slot_extractor: Arc<dyn SlotExtractor>) -> Self {
        Self {
            intents: RwLock::new(Vec::new()),
            slot_extractor,
        }
    }

    pub async fn extract_slots(
        &self,
        text: &str,
        language: &str,
    ) -> Result<HashMap<String, SlotValue>, IntentError> {
        let context = ExtractionContext::default();
        self.slot_extractor
            .extract_slots(text, language, &context)
            .await
            .map_err(IntentError::from)
    }
}
```

---

## Translator Trait (For Approach 2)

```rust
// crates/core/src/translator.rs (NEW)

#[async_trait]
pub trait Translator: Send + Sync {
    /// Translate text from source to target language
    async fn translate(
        &self,
        text: &str,
        from: &str,
        to: &str,
    ) -> Result<String, TranslationError>;

    /// Check if language pair is supported
    fn supports_pair(&self, from: &str, to: &str) -> bool;

    /// Provider name
    fn name(&self) -> &'static str;
}

// ONNX Implementation
pub struct OnnxTranslator {
    session: ort::Session,
    tokenizer: Tokenizer,
}

#[async_trait]
impl Translator for OnnxTranslator {
    async fn translate(&self, text: &str, from: &str, to: &str) -> Result<String, TranslationError> {
        // IndicTrans2 ONNX inference
        let tokens = self.tokenizer.encode(text, from, to)?;
        let output = self.session.run(tokens)?;
        self.tokenizer.decode(output)
    }

    fn name(&self) -> &'static str {
        "onnx_indictrans2"
    }
}

// gRPC Fallback Implementation
pub struct GrpcTranslator {
    client: IndicTransClient,
}

#[async_trait]
impl Translator for GrpcTranslator {
    async fn translate(&self, text: &str, from: &str, to: &str) -> Result<String, TranslationError> {
        self.client.translate(text, from, to).await
    }

    fn name(&self) -> &'static str {
        "grpc_indictrans2"
    }
}
```

---

## Comparison Matrix

| Aspect | Approach 1 (Native) | Approach 2 (Translate-First) | Hybrid |
|--------|---------------------|------------------------------|--------|
| **Latency** | ✅ Fastest | ⚠️ +100-200ms | ⚠️ Variable |
| **Accuracy** | ⚠️ Needs 22 lang patterns | ✅ English patterns only | ✅ Best of both |
| **Maintenance** | ❌ Complex | ✅ Simple | ⚠️ Medium |
| **Offline** | ✅ Works | ❌ Needs translation model | ⚠️ Partial |
| **LLM Quality** | ⚠️ Multilingual prompts | ✅ English reasoning | ✅ Best |
| **Cultural Nuance** | ✅ Preserved | ⚠️ May lose | ✅ Native fallback |
| **Testing** | ❌ 22 language tests | ✅ English tests only | ⚠️ Both |

---

## Implementation Priority

### Phase 1: Core Traits (P0)
1. Create `SlotExtractor` trait
2. Create `Translator` trait
3. Add `MultilingualStrategy` enum to config
4. Refactor `IntentDetector` to accept injectable extractor

### Phase 2: Implementations (P1)
1. Implement `NativeSlotExtractor` (move existing code)
2. Implement `EnglishSlotExtractor` (simplified)
3. Implement `OnnxTranslator` (IndicTrans2)
4. Implement `TranslateFirstExtractor`

### Phase 3: Integration (P1)
1. Wire up factory function in agent initialization
2. Add config loading for multilingual strategy
3. Add metrics for extraction method used

### Phase 4: Hybrid & Testing (P2)
1. Implement `HybridExtractor`
2. Add A/B testing infrastructure
3. Create test suites for all 22 languages

---

## Files to Create/Modify

### New Files
```
crates/agent/src/extraction/
├── mod.rs              # Trait + factory
├── native.rs           # NativeSlotExtractor
├── translate_first.rs  # TranslateFirstExtractor
├── hybrid.rs           # HybridExtractor
└── english.rs          # EnglishSlotExtractor

crates/core/src/
├── translator.rs       # Translator trait
├── indic_numerals.rs   # Numeral normalization (from plan)
└── script_detect.rs    # Script detection (from plan)

crates/config/src/
└── multilingual.rs     # MultilingualConfig
```

### Modified Files
```
crates/agent/src/intent.rs      # Accept SlotExtractor via DI
crates/agent/src/agent.rs       # Wire up extractor factory
crates/config/src/agent.rs      # Add MultilingualConfig
crates/config/src/lib.rs        # Export new config
```

---

## Testing Strategy

```rust
#[tokio::test]
async fn test_approach_switching() {
    // Test that we can switch approaches via config
    let native_config = MultilingualConfig {
        strategy: MultilingualStrategy::Native,
        ..Default::default()
    };

    let translate_config = MultilingualConfig {
        strategy: MultilingualStrategy::TranslateFirst,
        ..Default::default()
    };

    let translator = Arc::new(MockTranslator::new());

    let native_extractor = create_extractor(&native_config, None).unwrap();
    let translate_extractor = create_extractor(&translate_config, Some(translator)).unwrap();

    let tamil_text = "ஐந்து லட்சம் loan வேண்டும்";

    // Both should extract the same amount
    let native_slots = native_extractor.extract_slots(tamil_text, "ta", &ctx).await.unwrap();
    let translate_slots = translate_extractor.extract_slots(tamil_text, "ta", &ctx).await.unwrap();

    assert_eq!(
        native_slots.get("loan_amount"),
        translate_slots.get("loan_amount")
    );
}
```

---

*This document provides a pluggable architecture that allows switching between approaches via configuration, following the existing patterns in the codebase (strategy enums, traits, factory functions).*
