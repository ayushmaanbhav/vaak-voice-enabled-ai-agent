//! Persona Configuration
//!
//! Defines persona, tone, and communication style configurations loaded from YAML.
//!
//! DOMAIN-AGNOSTIC DESIGN:
//! - All tone phrases and instructions are config-driven (no hardcoded strings)
//! - Warmth/empathy thresholds generate dynamic instructions
//! - Supports localization (en, hi, hinglish)
//! - Adaptation rules for real-time persona adjustments

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Main personas configuration loaded from personas.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonasConfig {
    /// Tone configurations keyed by tone ID (formal, professional, friendly, casual)
    #[serde(default)]
    pub tones: HashMap<String, ToneConfig>,

    /// Warmth level thresholds for dynamic instruction generation
    #[serde(default)]
    pub warmth_thresholds: Vec<ThresholdConfig>,

    /// Empathy level thresholds for dynamic instruction generation
    #[serde(default)]
    pub empathy_thresholds: Vec<ThresholdConfig>,

    /// Language complexity level configurations
    #[serde(default)]
    pub complexity_levels: HashMap<String, ComplexityConfig>,

    /// Response urgency level configurations
    #[serde(default)]
    pub urgency_levels: HashMap<String, UrgencyConfig>,

    /// Hinglish (Hindi-English mix) configuration
    #[serde(default)]
    pub hinglish_config: HinglishConfig,

    /// Dynamic persona adaptation rules based on detected signals
    #[serde(default)]
    pub adaptation_rules: HashMap<String, AdaptationRule>,

    /// Response length guidelines
    #[serde(default)]
    pub response_length_guidelines: ResponseLengthGuidelines,

    /// Customer name usage guidelines
    #[serde(default)]
    pub name_usage: NameUsageConfig,

    /// Emotion acknowledgment phrases
    #[serde(default)]
    pub emotion_acknowledgment: EmotionAcknowledgmentConfig,
}

impl Default for PersonasConfig {
    fn default() -> Self {
        Self {
            tones: HashMap::new(),
            warmth_thresholds: Vec::new(),
            empathy_thresholds: Vec::new(),
            complexity_levels: HashMap::new(),
            urgency_levels: HashMap::new(),
            hinglish_config: HinglishConfig::default(),
            adaptation_rules: HashMap::new(),
            response_length_guidelines: ResponseLengthGuidelines::default(),
            name_usage: NameUsageConfig::default(),
            emotion_acknowledgment: EmotionAcknowledgmentConfig::default(),
        }
    }
}

impl PersonasConfig {
    /// Load from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, PersonasConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            PersonasConfigError::FileNotFound(path.as_ref().display().to_string(), e.to_string())
        })?;

        serde_yaml::from_str(&content)
            .map_err(|e| PersonasConfigError::ParseError(e.to_string()))
    }

    /// Get tone configuration by ID
    pub fn get_tone(&self, tone_id: &str) -> Option<&ToneConfig> {
        self.tones.get(tone_id)
    }

    /// Get greeting prefix for a tone in a specific language
    pub fn greeting_prefix(&self, tone_id: &str, language: &str) -> Option<&str> {
        self.tones
            .get(tone_id)
            .and_then(|tone| {
                tone.greeting_prefix
                    .get(language)
                    .or_else(|| tone.greeting_prefix.get("en"))
            })
            .map(|s| s.as_str())
    }

    /// Get closing phrase for a tone in a specific language
    pub fn closing_phrase(&self, tone_id: &str, language: &str) -> Option<&str> {
        self.tones
            .get(tone_id)
            .and_then(|tone| {
                tone.closing_phrase
                    .get(language)
                    .or_else(|| tone.closing_phrase.get("en"))
            })
            .map(|s| s.as_str())
    }

    /// Get tone instructions for a specific language
    pub fn tone_instructions(&self, tone_id: &str, language: &str) -> Option<&str> {
        self.tones
            .get(tone_id)
            .and_then(|tone| {
                tone.instructions
                    .get(language)
                    .or_else(|| tone.instructions.get("en"))
            })
            .map(|s| s.as_str())
    }

    /// Get warmth instruction for a given warmth level
    pub fn warmth_instruction(&self, warmth: f32, language: &str) -> Option<&str> {
        // Thresholds are sorted by min descending, find first match
        for threshold in &self.warmth_thresholds {
            if warmth >= threshold.min {
                return threshold
                    .instruction
                    .get(language)
                    .or_else(|| threshold.instruction.get("en"))
                    .map(|s| s.as_str());
            }
        }
        None
    }

    /// Get empathy instruction for a given empathy level
    pub fn empathy_instruction(&self, empathy: f32, language: &str) -> Option<&str> {
        for threshold in &self.empathy_thresholds {
            if empathy >= threshold.min {
                return threshold
                    .instruction
                    .get(language)
                    .or_else(|| threshold.instruction.get("en"))
                    .map(|s| s.as_str());
            }
        }
        None
    }

    /// Get complexity level instruction
    pub fn complexity_instruction(&self, level: &str, language: &str) -> Option<&str> {
        self.complexity_levels.get(level).and_then(|config| {
            config
                .instruction
                .get(language)
                .or_else(|| config.instruction.get("en"))
                .map(|s| s.as_str())
        })
    }

    /// Get urgency level instruction
    pub fn urgency_instruction(&self, level: &str, language: &str) -> Option<&str> {
        self.urgency_levels.get(level).and_then(|config| {
            config
                .instruction
                .get(language)
                .or_else(|| config.instruction.get("en"))
                .map(|s| s.as_str())
        })
    }

    /// Get hinglish instruction based on enabled flag
    pub fn hinglish_instruction(&self, enabled: bool, language: &str) -> Option<&str> {
        if enabled {
            self.hinglish_config
                .enabled_instruction
                .get(language)
                .or_else(|| self.hinglish_config.enabled_instruction.get("en"))
                .map(|s| s.as_str())
        } else {
            self.hinglish_config
                .disabled_instruction
                .get(language)
                .or_else(|| self.hinglish_config.disabled_instruction.get("en"))
                .map(|s| s.as_str())
        }
    }

    /// Get adaptation rule by signal type
    pub fn get_adaptation_rule(&self, signal: &str) -> Option<&AdaptationRule> {
        self.adaptation_rules.get(signal)
    }

    /// Get response length guideline for a max_words value
    pub fn response_length_guideline(&self, max_words: usize) -> Option<&str> {
        for range_guideline in &self.response_length_guidelines.by_max_words {
            if let [min, max] = range_guideline.range.as_slice() {
                if max_words >= *min && max_words <= *max {
                    return Some(&range_guideline.guideline);
                }
            }
        }
        None
    }

    /// Get emotion acknowledgment phrases for an emotion type
    pub fn emotion_phrases(&self, emotion: &str, language: &str) -> Vec<&str> {
        self.emotion_acknowledgment
            .enabled_phrases
            .get(emotion)
            .and_then(|phrases| {
                phrases
                    .get(language)
                    .or_else(|| phrases.get("en"))
            })
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Build complete persona instructions from config
    /// This replaces the hardcoded system_prompt_instructions() in persona.rs
    pub fn build_persona_instructions(
        &self,
        tone_id: &str,
        warmth: f32,
        empathy: f32,
        complexity: &str,
        urgency: &str,
        use_hinglish: bool,
        use_customer_name: bool,
        acknowledge_emotions: bool,
        max_response_words: usize,
        language: &str,
    ) -> String {
        let mut instructions = Vec::new();

        // Tone instruction
        if let Some(inst) = self.tone_instructions(tone_id, language) {
            instructions.push(inst.to_string());
        }

        // Warmth instruction
        if let Some(inst) = self.warmth_instruction(warmth, language) {
            instructions.push(inst.to_string());
        }

        // Empathy instruction
        if let Some(inst) = self.empathy_instruction(empathy, language) {
            instructions.push(inst.to_string());
        }

        // Complexity instruction
        if let Some(inst) = self.complexity_instruction(complexity, language) {
            instructions.push(inst.to_string());
        }

        // Urgency instruction
        if let Some(inst) = self.urgency_instruction(urgency, language) {
            instructions.push(inst.to_string());
        }

        // Hinglish instruction
        if let Some(inst) = self.hinglish_instruction(use_hinglish, language) {
            instructions.push(inst.to_string());
        }

        // Name usage instruction
        if use_customer_name {
            if let Some(inst) = self.name_usage.enabled_guidelines.get(language)
                .or_else(|| self.name_usage.enabled_guidelines.get("en"))
            {
                instructions.push(inst.clone());
            }
        } else if let Some(inst) = self.name_usage.disabled_guidelines.get(language)
            .or_else(|| self.name_usage.disabled_guidelines.get("en"))
        {
            instructions.push(inst.clone());
        }

        // Response length guideline
        if let Some(guideline) = self.response_length_guideline(max_response_words) {
            instructions.push(guideline.to_string());
        }

        // Emotion acknowledgment note
        if acknowledge_emotions {
            instructions.push("Acknowledge customer emotions when appropriate.".to_string());
        }

        instructions.join(" ")
    }

    /// Get all available tone IDs
    pub fn all_tone_ids(&self) -> Vec<&str> {
        self.tones.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a tone ID exists
    pub fn has_tone(&self, tone_id: &str) -> bool {
        self.tones.contains_key(tone_id)
    }
}

/// Tone configuration with localized phrases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToneConfig {
    /// Description of this tone
    #[serde(default)]
    pub description: String,

    /// Greeting prefix by language (e.g., "Respected", "Dear", "Hi")
    #[serde(default)]
    pub greeting_prefix: HashMap<String, String>,

    /// Closing phrase by language
    #[serde(default)]
    pub closing_phrase: HashMap<String, String>,

    /// System prompt instructions by language
    #[serde(default)]
    pub instructions: HashMap<String, String>,
}

/// Threshold configuration for warmth/empathy levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// Minimum value for this threshold (e.g., 0.9, 0.7, 0.5, 0.0)
    pub min: f32,

    /// Level name (e.g., "very_high", "high", "medium", "low")
    #[serde(default)]
    pub level: String,

    /// Instruction by language
    #[serde(default)]
    pub instruction: HashMap<String, String>,
}

/// Language complexity level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityConfig {
    /// Description of this complexity level
    #[serde(default)]
    pub description: String,

    /// Instruction by language
    #[serde(default)]
    pub instruction: HashMap<String, String>,

    /// Maximum sentence length recommendation
    #[serde(default)]
    pub max_sentence_length: Option<usize>,

    /// Terms to avoid at this complexity level
    #[serde(default)]
    pub avoid_terms: Vec<String>,
}

/// Response urgency level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrgencyConfig {
    /// Description of this urgency level
    #[serde(default)]
    pub description: String,

    /// Instruction by language
    #[serde(default)]
    pub instruction: HashMap<String, String>,

    /// Pace indicator (slow, normal, fast, very_fast)
    #[serde(default)]
    pub pace: String,

    /// Optional follow-up delay hint
    #[serde(default)]
    pub follow_up_delay_hint: Option<String>,
}

/// Hinglish (Hindi-English mix) configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HinglishConfig {
    /// Instructions when hinglish is enabled
    #[serde(default)]
    pub enabled_instruction: HashMap<String, String>,

    /// Instructions when hinglish is disabled
    #[serde(default)]
    pub disabled_instruction: HashMap<String, String>,

    /// Common hinglish phrases by category
    #[serde(default)]
    pub common_phrases: HashMap<String, Vec<String>>,
}

/// Dynamic persona adaptation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationRule {
    /// Warmth adjustment (added to current warmth)
    #[serde(default)]
    pub warmth_adjustment: Option<f32>,

    /// Empathy adjustment (added to current empathy)
    #[serde(default)]
    pub empathy_adjustment: Option<f32>,

    /// Override warmth to this value
    #[serde(default)]
    pub warmth_override: Option<f32>,

    /// Override empathy to this value
    #[serde(default)]
    pub empathy_override: Option<f32>,

    /// Override complexity level
    #[serde(default)]
    pub complexity_override: Option<String>,

    /// Override urgency level
    #[serde(default)]
    pub urgency_override: Option<String>,

    /// Additional instruction to append
    #[serde(default)]
    pub instruction_addition: Option<String>,
}

/// Response length guidelines
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseLengthGuidelines {
    /// Guidelines by max_words ranges
    #[serde(default)]
    pub by_max_words: Vec<RangeGuideline>,
}

/// Range-based guideline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeGuideline {
    /// Range [min, max]
    pub range: Vec<usize>,

    /// Guideline text
    pub guideline: String,
}

/// Customer name usage configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NameUsageConfig {
    /// Guidelines when name usage is enabled
    #[serde(default)]
    pub enabled_guidelines: HashMap<String, String>,

    /// Guidelines when name usage is disabled
    #[serde(default)]
    pub disabled_guidelines: HashMap<String, String>,

    /// Usage frequency hint
    #[serde(default)]
    pub frequency: Option<String>,

    /// Positions where name can be used
    #[serde(default)]
    pub positions: Vec<String>,
}

/// Emotion acknowledgment configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmotionAcknowledgmentConfig {
    /// Phrases by emotion type and language
    #[serde(default)]
    pub enabled_phrases: HashMap<String, HashMap<String, Vec<String>>>,
}

/// Errors when loading personas configuration
#[derive(Debug)]
pub enum PersonasConfigError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for PersonasConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => {
                write!(f, "Personas config not found at {}: {}", path, err)
            }
            Self::ParseError(err) => write!(f, "Failed to parse personas config: {}", err),
        }
    }
}

impl std::error::Error for PersonasConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personas_deserialization() {
        let yaml = r#"
tones:
  formal:
    description: "Highly formal tone"
    greeting_prefix:
      en: "Respected"
      hi: "आदरणीय"
    closing_phrase:
      en: "Thank you for your time."
    instructions:
      en: "Use formal language."

warmth_thresholds:
  - min: 0.8
    level: "high"
    instruction:
      en: "Be very warm."
  - min: 0.0
    level: "low"
    instruction:
      en: "Be professional."

complexity_levels:
  simple:
    instruction:
      en: "Use simple words."
"#;
        let config: PersonasConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.tones.contains_key("formal"));
        assert_eq!(config.warmth_thresholds.len(), 2);
        assert!(config.complexity_levels.contains_key("simple"));
    }

    #[test]
    fn test_greeting_prefix() {
        let yaml = r#"
tones:
  formal:
    greeting_prefix:
      en: "Respected"
      hi: "आदरणीय"
"#;
        let config: PersonasConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.greeting_prefix("formal", "en"), Some("Respected"));
        assert_eq!(config.greeting_prefix("formal", "hi"), Some("आदरणीय"));
        // Fallback to English
        assert_eq!(config.greeting_prefix("formal", "fr"), Some("Respected"));
    }

    #[test]
    fn test_warmth_instruction() {
        let yaml = r#"
warmth_thresholds:
  - min: 0.8
    instruction:
      en: "Very warm"
  - min: 0.5
    instruction:
      en: "Moderately warm"
  - min: 0.0
    instruction:
      en: "Professional"
"#;
        let config: PersonasConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.warmth_instruction(0.9, "en"), Some("Very warm"));
        assert_eq!(config.warmth_instruction(0.6, "en"), Some("Moderately warm"));
        assert_eq!(config.warmth_instruction(0.3, "en"), Some("Professional"));
    }

    #[test]
    fn test_build_persona_instructions() {
        let yaml = r#"
tones:
  professional:
    instructions:
      en: "Use professional language."

warmth_thresholds:
  - min: 0.7
    instruction:
      en: "Be warm."
  - min: 0.0
    instruction:
      en: "Be neutral."

empathy_thresholds:
  - min: 0.5
    instruction:
      en: "Show empathy."

complexity_levels:
  moderate:
    instruction:
      en: "Use clear language."

urgency_levels:
  normal:
    instruction:
      en: "Normal pace."
"#;
        let config: PersonasConfig = serde_yaml::from_str(yaml).unwrap();
        let instructions = config.build_persona_instructions(
            "professional",
            0.8,  // warmth
            0.6,  // empathy
            "moderate",
            "normal",
            false,  // use_hinglish
            true,   // use_customer_name
            true,   // acknowledge_emotions
            60,     // max_response_words
            "en",
        );

        assert!(instructions.contains("Use professional language."));
        assert!(instructions.contains("Be warm."));
        assert!(instructions.contains("Show empathy."));
        assert!(instructions.contains("Use clear language."));
        assert!(instructions.contains("Normal pace."));
    }
}
