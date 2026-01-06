# Text Processing Pipeline

> Grammar correction, translation, compliance, and PII handling for voice agents
>
> **Design Goal:** Streaming sentence-by-sentence processing with <100ms overhead per stage

---

## Table of Contents

1. [Overview](#overview)
2. [Pipeline Architecture](#pipeline-architecture)
3. [Input Processing](#input-processing)
4. [Output Processing](#output-processing)
5. [Streaming Design](#streaming-design)
6. [Indian Language Considerations](#indian-language-considerations)
7. [Implementation](#implementation)
8. [Configuration](#configuration)
9. [Testing](#testing)

---

## Overview

### Why Text Processing?

Voice conversations require text transformations for:

| Stage | Problem | Solution |
|-------|---------|----------|
| **STT Output** | Transcription errors | Grammar correction |
| **LLM Input** | Indian language reasoning weak | Translate to English |
| **LLM Output** | English response | Translate to customer's language |
| **Before TTS** | Regulatory violations possible | Compliance check |
| **Before Logging** | Contains customer PII | PII redaction |
| **Before TTS** | Complex language | Simplify for pronunciation |

### Pipeline Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      TEXT PROCESSING PIPELINE                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  INPUT PIPELINE (After STT, Before LLM):                                    │
│                                                                             │
│  ┌──────────┐   ┌───────────────┐   ┌───────────────┐   ┌──────────┐       │
│  │   STT    │──►│    Grammar    │──►│   Translate   │──►│   LLM    │       │
│  │  Output  │   │   Correction  │   │   IN → EN     │   │  Input   │       │
│  └──────────┘   └───────────────┘   └───────────────┘   └──────────┘       │
│                                                                             │
│  OUTPUT PIPELINE (After LLM, Before TTS):                                   │
│                                                                             │
│  ┌──────────┐   ┌───────────────┐   ┌───────────────┐   ┌───────────────┐  │
│  │   LLM    │──►│   Translate   │──►│  Compliance   │──►│     PII       │  │
│  │  Output  │   │   EN → IN     │   │    Check      │   │   Redact      │  │
│  └──────────┘   └───────────────┘   └───────────────┘   └───────┬───────┘  │
│                                                                  │          │
│                                                                  ▼          │
│                                                 ┌───────────────────────┐   │
│                                                 │      Simplify         │   │
│                                                 │    (for TTS)          │   │
│                                                 └───────────┬───────────┘   │
│                                                             │               │
│                                                             ▼               │
│                                                       ┌──────────┐         │
│                                                       │   TTS    │         │
│                                                       │  Input   │         │
│                                                       └──────────┘         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Pipeline Architecture

### Composable Processors

Each stage is a `TextProcessor` that can be enabled/disabled:

```rust
/// Text processor trait
#[async_trait]
pub trait TextProcessor: Send + Sync + 'static {
    /// Process single text
    async fn process(&self, text: &str, context: &ProcessContext) -> Result<String>;

    /// Process stream of sentences
    fn process_stream(
        &self,
        input: impl Stream<Item = String> + Send + 'static,
        context: &ProcessContext,
    ) -> Box<dyn Stream<Item = Result<String>> + Send + Unpin>;

    /// Processor name for tracing
    fn name(&self) -> &'static str;

    /// Is this processor enabled?
    fn enabled(&self) -> bool {
        true
    }
}

/// Processing context
#[derive(Debug, Clone)]
pub struct ProcessContext {
    pub language: Language,
    pub domain: String,
    pub segment: Option<CustomerSegment>,
    pub conversation_id: String,
}
```

### Pipeline Composition

```rust
pub struct TextPipeline {
    processors: Vec<Box<dyn TextProcessor>>,
}

impl TextPipeline {
    pub fn new(processors: Vec<Box<dyn TextProcessor>>) -> Self {
        Self { processors }
    }

    /// Process text through all stages
    pub async fn process(&self, text: &str, context: &ProcessContext) -> Result<String> {
        let mut current = text.to_string();

        for processor in &self.processors {
            if !processor.enabled() {
                continue;
            }

            let span = tracing::info_span!(
                "text_processor",
                processor = processor.name()
            );
            let _guard = span.enter();

            let start = Instant::now();
            current = processor.process(&current, context).await?;

            tracing::debug!(
                duration_ms = start.elapsed().as_millis(),
                "Processor complete"
            );
        }

        Ok(current)
    }

    /// Process stream through all stages
    pub fn process_stream(
        &self,
        input: impl Stream<Item = String> + Send + 'static,
        context: &ProcessContext,
    ) -> impl Stream<Item = Result<String>> + Send {
        let mut current_stream: Box<dyn Stream<Item = Result<String>> + Send + Unpin> =
            Box::new(input.map(Ok));

        for processor in &self.processors {
            if !processor.enabled() {
                continue;
            }

            current_stream = processor.process_stream(
                current_stream.map(|r| r.unwrap_or_default()),
                context,
            );
        }

        current_stream
    }
}
```

---

## Input Processing

### Grammar Correction

**Purpose:** Fix STT transcription errors using domain knowledge.

**Why LLM-Based:**
- No mature Rust library for Indian languages
- Domain vocabulary matters (e.g., "gold Ion" → "gold loan")
- Context-aware corrections

```rust
pub struct LLMGrammarCorrector {
    llm: Arc<dyn LanguageModel>,
    domain_vocab: DomainVocabulary,
}

#[derive(Debug, Clone)]
pub struct DomainVocabulary {
    /// Words to preserve/correct to
    pub terms: Vec<String>,
    /// Common phrases
    pub phrases: Vec<String>,
    /// Common STT mistakes and corrections
    pub corrections: HashMap<String, String>,
}

impl LLMGrammarCorrector {
    fn build_prompt(&self, text: &str) -> String {
        format!(r#"
Fix transcription errors in this speech-to-text output.

DOMAIN VOCABULARY (preserve these terms):
{terms}

KNOWN CORRECTIONS:
{corrections}

RULES:
1. Fix obvious transcription errors
2. Preserve proper nouns and numbers
3. Keep meaning unchanged
4. Output ONLY the corrected text

INPUT: {text}
CORRECTED:"#,
            terms = self.domain_vocab.terms.join(", "),
            corrections = self.domain_vocab.corrections
                .iter()
                .map(|(k, v)| format!("{} → {}", k, v))
                .collect::<Vec<_>>()
                .join(", "),
            text = text,
        )
    }
}

#[async_trait]
impl TextProcessor for LLMGrammarCorrector {
    async fn process(&self, text: &str, context: &ProcessContext) -> Result<String> {
        // Skip if text is very short or already clean
        if text.len() < 5 {
            return Ok(text.to_string());
        }

        // Apply known corrections first (fast)
        let mut corrected = text.to_string();
        for (wrong, right) in &self.domain_vocab.corrections {
            corrected = corrected.replace(wrong, right);
        }

        // If no changes needed, skip LLM
        if self.looks_correct(&corrected) {
            return Ok(corrected);
        }

        // Use LLM for complex corrections
        let prompt = self.build_prompt(&corrected);
        let response = self.llm.generate(GenerateRequest {
            prompt,
            max_tokens: (text.len() * 2) as u32,
            temperature: 0.1,
            ..Default::default()
        }).await?;

        Ok(response.text.trim().to_string())
    }

    fn name(&self) -> &'static str {
        "grammar_corrector"
    }
}
```

**Example Corrections:**

| STT Output | Corrected | Reason |
|------------|-----------|--------|
| "Kotak se gold Ion lena hai" | "Kotak se gold loan lena hai" | Domain vocabulary |
| "byaaj dar kitna hai" | "byaaj dar kitna hai" | No change (correct) |
| "Muthooth se" | "Muthoot se" | Known correction |

### Translation (Indian Language → English)

**Purpose:** LLMs reason better in English. Translate before LLM processing.

**The "Translate-Think-Translate" Pattern:**

```
User (Hindi) → Translate(Hi→En) → LLM(English) → Translate(En→Hi) → TTS
```

```rust
pub struct InputTranslator {
    translator: Arc<dyn Translator>,
    target_language: Language,
}

#[async_trait]
impl TextProcessor for InputTranslator {
    async fn process(&self, text: &str, context: &ProcessContext) -> Result<String> {
        // Skip if already in target language
        if context.language == self.target_language {
            return Ok(text.to_string());
        }

        // Translate to English
        self.translator
            .translate(text, context.language, self.target_language)
            .await
    }

    fn name(&self) -> &'static str {
        "input_translator"
    }

    fn process_stream(
        &self,
        input: impl Stream<Item = String> + Send + 'static,
        context: &ProcessContext,
    ) -> Box<dyn Stream<Item = Result<String>> + Send + Unpin> {
        // Stream translation sentence by sentence
        let translator = self.translator.clone();
        let source = context.language;
        let target = self.target_language;

        Box::new(stream! {
            pin_mut!(input);
            while let Some(sentence) = input.next().await {
                if source == target {
                    yield Ok(sentence);
                } else {
                    yield translator.translate(&sentence, source, target).await;
                }
            }
        })
    }
}
```

---

## Output Processing

### Translation (English → Indian Language)

```rust
pub struct OutputTranslator {
    translator: Arc<dyn Translator>,
}

#[async_trait]
impl TextProcessor for OutputTranslator {
    async fn process(&self, text: &str, context: &ProcessContext) -> Result<String> {
        // Translate from English to customer's language
        if context.language == Language::English {
            return Ok(text.to_string());
        }

        self.translator
            .translate(text, Language::English, context.language)
            .await
    }

    fn name(&self) -> &'static str {
        "output_translator"
    }
}
```

### Compliance Checking

**Purpose:** Ensure agent responses comply with banking regulations.

```rust
pub struct ComplianceProcessor {
    checker: Arc<dyn ComplianceChecker>,
    strict_mode: bool,
}

#[async_trait]
impl TextProcessor for ComplianceProcessor {
    async fn process(&self, text: &str, context: &ProcessContext) -> Result<String> {
        let result = self.checker.check(text).await?;

        if result.is_compliant {
            // Add any required disclaimers
            if result.required_additions.is_empty() {
                return Ok(text.to_string());
            }

            let mut output = text.to_string();
            for addition in &result.required_additions {
                match addition.position {
                    AdditionPosition::End => {
                        output.push_str(" ");
                        output.push_str(&addition.text);
                    }
                    AdditionPosition::Beginning => {
                        output = format!("{} {}", addition.text, output);
                    }
                    AdditionPosition::AfterClaim => {
                        // Insert after the triggering phrase
                        // Implementation depends on tracking claim positions
                        output.push_str(" ");
                        output.push_str(&addition.text);
                    }
                }
            }
            return Ok(output);
        }

        // Handle violations
        for violation in &result.violations {
            match violation.severity {
                Severity::Critical => {
                    if self.strict_mode {
                        return Err(Error::ComplianceViolation(violation.clone()));
                    }
                    // Try to make compliant
                    return self.checker.make_compliant(text).await;
                }
                Severity::Error => {
                    return self.checker.make_compliant(text).await;
                }
                Severity::Warning => {
                    tracing::warn!(
                        violation = ?violation,
                        "Compliance warning (proceeding)"
                    );
                }
                Severity::Info => {}
            }
        }

        Ok(text.to_string())
    }

    fn name(&self) -> &'static str {
        "compliance_checker"
    }
}
```

**Compliance Rules Example:**

```yaml
# domains/gold_loan/compliance.yaml

forbidden_phrases:
  - "guaranteed approval"
  - "100% approval"
  - "no documentation required"
  - "beat any rate"

claims_requiring_disclaimer:
  - pattern: "save up to \\d+%"
    disclaimer: "Actual savings depend on your current loan terms."

  - pattern: "lowest rate"
    disclaimer: "Rates subject to eligibility and RBI guidelines."

rate_rules:
  min_rate: 10.0
  max_rate: 18.0
  # Alert if agent quotes outside this range

competitor_rules:
  # Can mention but not disparage
  allowed_comparisons:
    - "Our rate is X%, while [competitor] typically charges Y%"
  forbidden_comparisons:
    - "[competitor] has fraud"
    - "[competitor] is unsafe"
```

### PII Redaction

**Purpose:** Protect customer data in logs and analytics.

```rust
pub struct PIIProcessor {
    redactor: Arc<dyn PIIRedactor>,
    strategy: RedactionStrategy,
    log_mode: bool,  // True = redact for logging, False = for TTS
}

#[async_trait]
impl TextProcessor for PIIProcessor {
    async fn process(&self, text: &str, context: &ProcessContext) -> Result<String> {
        if self.log_mode {
            // Full redaction for logging
            self.redactor.redact(text, &self.strategy).await
        } else {
            // For TTS, we might want to keep some PII
            // (e.g., customer's name for personalization)
            let result = self.redactor.detect(text).await?;

            let mut redacted = text.to_string();
            for entity in result.into_iter().rev() {
                // Redact everything except names
                if entity.pii_type != PIIType::PersonName {
                    let replacement = match &self.strategy {
                        RedactionStrategy::PartialMask { visible_chars } => {
                            partial_mask(&entity.text, *visible_chars)
                        }
                        _ => "[REDACTED]".to_string(),
                    };
                    redacted.replace_range(entity.start..entity.end, &replacement);
                }
            }

            Ok(redacted)
        }
    }

    fn name(&self) -> &'static str {
        if self.log_mode {
            "pii_redactor_log"
        } else {
            "pii_redactor_tts"
        }
    }
}
```

### Text Simplification

**Purpose:** Make text easier for TTS to pronounce correctly.

```rust
pub struct TextSimplifier {
    number_converter: NumberToWords,
    abbreviation_expander: AbbreviationExpander,
    max_sentence_length: usize,
}

#[async_trait]
impl TextProcessor for TextSimplifier {
    async fn process(&self, text: &str, context: &ProcessContext) -> Result<String> {
        let mut result = text.to_string();

        // 1. Expand numbers
        // "₹5 lakh" → "rupees paanch lakh" (in Hindi)
        result = self.number_converter.convert(&result, context.language);

        // 2. Expand abbreviations
        // "LTV" → "Loan to Value"
        result = self.abbreviation_expander.expand(&result);

        // 3. Break long sentences
        if self.should_break(&result) {
            result = self.break_sentences(&result);
        }

        // 4. Remove special characters TTS can't handle
        result = self.clean_for_tts(&result);

        Ok(result)
    }

    fn name(&self) -> &'static str {
        "text_simplifier"
    }
}

impl TextSimplifier {
    fn should_break(&self, text: &str) -> bool {
        text.split_whitespace().count() > self.max_sentence_length
    }

    fn break_sentences(&self, text: &str) -> String {
        // Break at conjunctions and commas
        text.replace(", and ", ". ")
            .replace(", but ", ". ")
            .replace(", or ", ". ")
    }

    fn clean_for_tts(&self, text: &str) -> String {
        text.chars()
            .filter(|c| {
                c.is_alphanumeric() ||
                c.is_whitespace() ||
                ".,!?'\"()-₹%".contains(*c)
            })
            .collect()
    }
}
```

**Simplification Examples:**

| Before | After | Rule |
|--------|-------|------|
| "₹5,00,000" | "paanch lakh rupaye" | Number conversion (Hindi) |
| "LTV is 75%" | "Loan to Value is pachattar percent" | Abbreviation + number |
| "A, and B, and C" | "A. B. C" | Sentence breaking |

---

## Streaming Design

### Sentence-by-Sentence Processing

All processors support streaming for low latency:

```rust
/// Sentence accumulator for streaming
pub struct SentenceAccumulator {
    buffer: String,
    terminators: &'static [char],
}

impl SentenceAccumulator {
    pub fn new(language: Language) -> Self {
        let terminators = match language {
            Language::Hindi => &['.', '!', '?', '।'][..],
            Language::English => &['.', '!', '?'][..],
            _ => &['.', '!', '?', '।', '॥'][..],
        };

        Self {
            buffer: String::new(),
            terminators,
        }
    }

    /// Add chunk and extract complete sentences
    pub fn add(&mut self, chunk: &str) -> Vec<String> {
        self.buffer.push_str(chunk);

        let mut sentences = Vec::new();

        while let Some(pos) = self.find_sentence_end() {
            let sentence = self.buffer[..=pos].trim().to_string();
            if !sentence.is_empty() {
                sentences.push(sentence);
            }
            self.buffer = self.buffer[pos + 1..].to_string();
        }

        sentences
    }

    /// Flush remaining content
    pub fn flush(&mut self) -> Option<String> {
        let remaining = std::mem::take(&mut self.buffer);
        let trimmed = remaining.trim();
        if !trimmed.is_empty() {
            Some(trimmed.to_string())
        } else {
            None
        }
    }

    fn find_sentence_end(&self) -> Option<usize> {
        self.buffer
            .char_indices()
            .find(|(_, c)| self.terminators.contains(c))
            .map(|(i, _)| i)
    }
}

/// Streaming pipeline with sentence buffering
pub fn create_streaming_pipeline(
    processors: Vec<Box<dyn TextProcessor>>,
    context: ProcessContext,
) -> impl FnMut(String) -> Pin<Box<dyn Stream<Item = Result<String>> + Send>> {
    let mut accumulator = SentenceAccumulator::new(context.language);

    move |chunk: String| {
        let sentences = accumulator.add(&chunk);
        let ctx = context.clone();
        let procs = processors.clone();

        Box::pin(stream! {
            for sentence in sentences {
                let mut current = sentence;
                for processor in &procs {
                    current = processor.process(&current, &ctx).await?;
                }
                yield Ok(current);
            }
        })
    }
}
```

### Latency Optimization

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    STREAMING LATENCY OPTIMIZATION                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  WITHOUT STREAMING:                                                         │
│                                                                             │
│  LLM generates full response ──────────────────────────────────► Then TTS   │
│  [=================================] 800ms                        [====]    │
│                                                     Total: 800ms + TTS      │
│                                                                             │
│  WITH STREAMING (Sentence-by-Sentence):                                     │
│                                                                             │
│  LLM: [Sent1]─────[Sent2]─────[Sent3]───────────────►                      │
│         │           │           │                                           │
│         ▼           ▼           ▼                                           │
│  TTS: [===]       [===]       [===]                                         │
│         │           │           │                                           │
│         ▼           ▼           ▼                                           │
│  Play: [===]─────[===]─────[===]                                            │
│                                                                             │
│  Time to first audio: ~200ms (just first sentence)                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Indian Language Considerations

### Script Detection

```rust
pub fn detect_script(text: &str) -> Script {
    let mut script_counts: HashMap<Script, usize> = HashMap::new();

    for c in text.chars() {
        let script = match c {
            '\u{0900}'..='\u{097F}' => Script::Devanagari,
            '\u{0980}'..='\u{09FF}' => Script::Bengali,
            '\u{0A00}'..='\u{0A7F}' => Script::Gurmukhi,
            '\u{0A80}'..='\u{0AFF}' => Script::Gujarati,
            '\u{0B00}'..='\u{0B7F}' => Script::Odia,
            '\u{0B80}'..='\u{0BFF}' => Script::Tamil,
            '\u{0C00}'..='\u{0C7F}' => Script::Telugu,
            '\u{0C80}'..='\u{0CFF}' => Script::Kannada,
            '\u{0D00}'..='\u{0D7F}' => Script::Malayalam,
            'A'..='Z' | 'a'..='z' => Script::Latin,
            _ => continue,
        };
        *script_counts.entry(script).or_insert(0) += 1;
    }

    script_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(script, _)| script)
        .unwrap_or(Script::Latin)
}
```

### Number Conversion (Multilingual)

```rust
pub struct NumberToWords {
    converters: HashMap<Language, Box<dyn NumberConverter>>,
}

impl NumberToWords {
    pub fn convert(&self, text: &str, language: Language) -> String {
        let converter = self.converters
            .get(&language)
            .unwrap_or_else(|| self.converters.get(&Language::English).unwrap());

        // Find numbers and convert
        let re = Regex::new(r"₹?\d+(?:,\d{2,3})*(?:\.\d+)?%?").unwrap();

        re.replace_all(text, |caps: &regex::Captures| {
            let num_str = &caps[0];
            converter.convert(num_str)
        }).to_string()
    }
}

trait NumberConverter: Send + Sync {
    fn convert(&self, number: &str) -> String;
}

struct HindiNumberConverter;

impl NumberConverter for HindiNumberConverter {
    fn convert(&self, number: &str) -> String {
        // "₹5,00,000" → "paanch lakh rupaye"
        let clean = number
            .replace("₹", "")
            .replace(",", "")
            .replace("%", "");

        let value: f64 = clean.parse().unwrap_or(0.0);

        let words = if value >= 10_000_000.0 {
            format!("{} crore", self.to_hindi((value / 10_000_000.0) as i64))
        } else if value >= 100_000.0 {
            format!("{} lakh", self.to_hindi((value / 100_000.0) as i64))
        } else if value >= 1_000.0 {
            format!("{} hazaar", self.to_hindi((value / 1_000.0) as i64))
        } else {
            self.to_hindi(value as i64)
        };

        let prefix = if number.starts_with("₹") { "rupaye " } else { "" };
        let suffix = if number.ends_with("%") { " pratishat" } else { "" };

        format!("{}{}{}", prefix, words, suffix)
    }
}
```

---

## Implementation

### Complete Input Pipeline

```rust
pub fn create_input_pipeline(
    config: &TextProcessingConfig,
    llm: Arc<dyn LanguageModel>,
    translator: Arc<dyn Translator>,
) -> TextPipeline {
    let mut processors: Vec<Box<dyn TextProcessor>> = Vec::new();

    // 1. Grammar correction
    if config.input.grammar_correction.enabled {
        let domain_vocab = DomainVocabulary::from_config(&config.domain_vocab_path);
        processors.push(Box::new(LLMGrammarCorrector::new(
            llm.clone(),
            domain_vocab,
        )));
    }

    // 2. Translation to English
    if config.input.translation.enabled {
        processors.push(Box::new(InputTranslator::new(
            translator.clone(),
            Language::English,
        )));
    }

    TextPipeline::new(processors)
}

pub fn create_output_pipeline(
    config: &TextProcessingConfig,
    translator: Arc<dyn Translator>,
    compliance_checker: Arc<dyn ComplianceChecker>,
    pii_redactor: Arc<dyn PIIRedactor>,
) -> TextPipeline {
    let mut processors: Vec<Box<dyn TextProcessor>> = Vec::new();

    // 1. Translation from English
    if config.output.translation.enabled {
        processors.push(Box::new(OutputTranslator::new(translator.clone())));
    }

    // 2. Compliance check
    if config.output.compliance.enabled {
        processors.push(Box::new(ComplianceProcessor::new(
            compliance_checker.clone(),
            config.output.compliance.strict_mode,
        )));
    }

    // 3. PII redaction (for logs)
    // Note: Run twice - once for logs (full redaction), once for TTS (partial)
    if config.output.pii_redaction.enabled {
        processors.push(Box::new(PIIProcessor::new(
            pii_redactor.clone(),
            config.output.pii_redaction.strategy.clone(),
            false, // TTS mode
        )));
    }

    // 4. Text simplification
    if config.output.simplification.enabled {
        processors.push(Box::new(TextSimplifier::new(
            config.output.simplification.max_sentence_length,
        )));
    }

    TextPipeline::new(processors)
}
```

---

## Configuration

```toml
# domains/gold_loan/text_processing.toml

[input]
[input.grammar_correction]
enabled = true
provider = "llm"
model = "qwen2.5:7b-q4"
temperature = 0.1
max_tokens = 256

[input.grammar_correction.domain_vocab]
file = "knowledge/vocabulary.yaml"

[input.translation]
enabled = true
provider = "onnx"           # "onnx" | "grpc"
target_language = "en"
fallback_provider = "grpc"
grpc_endpoint = "http://localhost:50051"
onnx_model_path = "models/translation/indictrans2_en.onnx"

[output]
[output.translation]
enabled = true
provider = "onnx"
source_language = "en"
# target_language = auto (from context)

[output.compliance]
enabled = true
provider = "rule_based"     # "rule_based" | "llm" | "hybrid"
rules_file = "compliance.yaml"
strict_mode = true          # Fail on critical violations

[output.pii_redaction]
enabled = true
provider = "hybrid"         # "rustbert" | "regex" | "hybrid"
strategy = "partial_mask"
visible_chars = 4
entities = [
    "Aadhaar", "PAN", "PhoneNumber", "Email",
    "BankAccount", "CreditCard"
]

[output.simplification]
enabled = true
expand_abbreviations = true
normalize_numbers = true
max_sentence_length = 25    # Words

# Logging pipeline (separate, full redaction)
[logging]
pii_strategy = "mask"       # Full redaction for logs
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_grammar_correction() {
        let corrector = LLMGrammarCorrector::new_mock();
        let context = ProcessContext::default();

        let result = corrector
            .process("Kotak se gold Ion lena hai", &context)
            .await
            .unwrap();

        assert_eq!(result, "Kotak se gold loan lena hai");
    }

    #[tokio::test]
    async fn test_pii_redaction() {
        let redactor = HybridPIIRedactor::new().unwrap();
        let context = ProcessContext::default();

        let result = redactor
            .redact(
                "Mera Aadhaar 1234 5678 9012 hai",
                &RedactionStrategy::PartialMask { visible_chars: 4 },
            )
            .await
            .unwrap();

        assert!(result.contains("1234****9012"));
    }

    #[tokio::test]
    async fn test_compliance_check() {
        let checker = RuleBasedComplianceChecker::from_config(Path::new("test_rules.yaml")).unwrap();

        let result = checker
            .check("We guarantee 100% approval")
            .await
            .unwrap();

        assert!(!result.is_compliant);
        assert!(result.violations.iter().any(|v| v.severity == Severity::Critical));
    }

    #[tokio::test]
    async fn test_streaming_pipeline() {
        let pipeline = create_test_pipeline();
        let context = ProcessContext::default();

        let input = stream::iter(vec![
            "Hello. ".to_string(),
            "How are ".to_string(),
            "you?".to_string(),
        ]);

        let output: Vec<_> = pipeline
            .process_stream(input, &context)
            .collect()
            .await;

        assert_eq!(output.len(), 2); // Two sentences
        assert!(output[0].is_ok());
        assert!(output[1].is_ok());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_input_output_pipeline() {
    // Setup
    let input_pipeline = create_input_pipeline(&config, llm, translator);
    let output_pipeline = create_output_pipeline(&config, translator, compliance, pii);

    let context = ProcessContext {
        language: Language::Hindi,
        domain: "gold_loan".to_string(),
        segment: Some(CustomerSegment::P1HighValue),
        conversation_id: "test-123".to_string(),
    };

    // Test input pipeline
    let user_input = "Muthooth se kitna kam interest milega?";
    let processed_input = input_pipeline.process(user_input, &context).await.unwrap();

    // Should be translated to English
    assert!(processed_input.contains("interest") || processed_input.contains("Muthoot"));

    // Test output pipeline
    let llm_output = "You can save up to 5% compared to Muthoot. Our rate is 12%.";
    let processed_output = output_pipeline.process(llm_output, &context).await.unwrap();

    // Should be translated back to Hindi
    assert!(processed_output.contains("pratishat") || processed_output.contains("%"));
}
```

---

## References

- [AI4Bharat IndicTrans2](https://github.com/AI4Bharat/IndicTrans2)
- [rust-bert NER](https://docs.rs/rust-bert/latest/rust_bert/pipelines/ner/)
- [Harper Grammar Checker](https://github.com/Automattic/harper)
- [nlprule](https://github.com/bminixhofer/nlprule)
