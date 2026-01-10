//! Persona Provider trait for config-driven persona management
//!
//! This module provides a domain-agnostic interface for managing personas,
//! tones, and communication styles. All definitions are loaded from configuration.
//!
//! # Design Goals
//!
//! - Replace hardcoded Tone::greeting_prefix(), closing_phrase()
//! - Replace hardcoded Persona::for_segment() match statement
//! - Replace hardcoded system_prompt_instructions()
//! - Enable runtime persona adaptation based on signals
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_core::traits::PersonaProvider;
//!
//! // Provider is created from domain config
//! let provider = config_bridge.persona_provider();
//!
//! // Get tone configuration
//! let greeting = provider.greeting_prefix("formal", "en");
//!
//! // Build persona instructions
//! let instructions = provider.build_instructions(&persona_config, "en");
//! ```

use std::collections::HashMap;

/// Segment ID type alias for clarity (String-based, config-driven)
pub type SegmentId = String;

/// Persona configuration from segment config
///
/// This replaces the hardcoded Persona::for_segment() match statement.
/// All values come from segments.yaml persona field.
#[derive(Debug, Clone)]
pub struct PersonaConfig {
    /// Persona name/identifier (e.g., "premium_advisor", "trust_builder")
    pub name: String,
    /// Tone identifier (e.g., "formal", "professional", "friendly", "casual")
    pub tone: String,
    /// Warmth level (0.0 = cold/factual, 1.0 = very warm/empathetic)
    pub warmth: f32,
    /// Empathy level (0.0 = neutral, 1.0 = highly empathetic)
    pub empathy: f32,
    /// Language complexity ("simple", "moderate", "sophisticated")
    pub language_complexity: String,
    /// Response urgency ("relaxed", "normal", "efficient", "urgent")
    pub urgency: String,
    /// Whether to use customer's name frequently
    pub use_customer_name: bool,
    /// Whether to acknowledge emotions
    pub acknowledge_emotions: bool,
    /// Whether to use Hindi words/phrases in English (Hinglish)
    pub use_hinglish: bool,
    /// Maximum response length preference (words)
    pub max_response_words: usize,
}

impl Default for PersonaConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            tone: "professional".to_string(),
            warmth: 0.8,
            empathy: 0.7,
            language_complexity: "moderate".to_string(),
            urgency: "normal".to_string(),
            use_customer_name: true,
            acknowledge_emotions: true,
            use_hinglish: false,
            max_response_words: 60,
        }
    }
}

/// Tone configuration from personas.yaml
#[derive(Debug, Clone)]
pub struct ToneConfig {
    /// Description of this tone
    pub description: String,
    /// Greeting prefix by language (e.g., "Respected", "Dear", "Hi")
    pub greeting_prefix: HashMap<String, String>,
    /// Closing phrase by language
    pub closing_phrase: HashMap<String, String>,
    /// System prompt instructions by language
    pub instructions: HashMap<String, String>,
}

impl ToneConfig {
    /// Get greeting prefix for a language, falling back to English
    pub fn greeting(&self, language: &str) -> Option<&str> {
        self.greeting_prefix
            .get(language)
            .or_else(|| self.greeting_prefix.get("en"))
            .map(|s| s.as_str())
    }

    /// Get closing phrase for a language, falling back to English
    pub fn closing(&self, language: &str) -> Option<&str> {
        self.closing_phrase
            .get(language)
            .or_else(|| self.closing_phrase.get("en"))
            .map(|s| s.as_str())
    }

    /// Get instructions for a language, falling back to English
    pub fn instruction(&self, language: &str) -> Option<&str> {
        self.instructions
            .get(language)
            .or_else(|| self.instructions.get("en"))
            .map(|s| s.as_str())
    }
}

/// Adaptation rule for dynamic persona adjustment
#[derive(Debug, Clone)]
pub struct AdaptationRule {
    /// Warmth adjustment (added to current warmth)
    pub warmth_adjustment: Option<f32>,
    /// Empathy adjustment (added to current empathy)
    pub empathy_adjustment: Option<f32>,
    /// Override warmth to this value
    pub warmth_override: Option<f32>,
    /// Override empathy to this value
    pub empathy_override: Option<f32>,
    /// Override complexity level
    pub complexity_override: Option<String>,
    /// Override urgency level
    pub urgency_override: Option<String>,
    /// Additional instruction to append
    pub instruction_addition: Option<String>,
}

/// Persona provider trait for config-driven persona management
///
/// This trait abstracts all persona-related operations that were previously
/// hardcoded in persona.rs. Implementations read from personas.yaml and
/// segments.yaml configurations.
pub trait PersonaProvider: Send + Sync {
    /// Get tone configuration by tone ID
    fn tone_config(&self, tone_id: &str) -> Option<&ToneConfig>;

    /// Get greeting prefix for a tone in a specific language
    ///
    /// Replaces hardcoded Tone::greeting_prefix()
    fn greeting_prefix(&self, tone_id: &str, language: &str) -> Option<&str>;

    /// Get closing phrase for a tone in a specific language
    ///
    /// Replaces hardcoded Tone::closing_phrase()
    fn closing_phrase(&self, tone_id: &str, language: &str) -> Option<&str>;

    /// Get tone instructions for a specific language
    fn tone_instructions(&self, tone_id: &str, language: &str) -> Option<&str>;

    /// Get warmth instruction based on warmth level
    fn warmth_instruction(&self, warmth: f32, language: &str) -> Option<String>;

    /// Get empathy instruction based on empathy level
    fn empathy_instruction(&self, empathy: f32, language: &str) -> Option<String>;

    /// Get language complexity instruction
    fn complexity_instruction(&self, level: &str, language: &str) -> Option<String>;

    /// Get urgency level instruction
    fn urgency_instruction(&self, level: &str, language: &str) -> Option<String>;

    /// Get hinglish instruction based on enabled flag
    fn hinglish_instruction(&self, enabled: bool, language: &str) -> Option<String>;

    /// Build complete persona instructions from config
    ///
    /// Replaces hardcoded system_prompt_instructions() in persona.rs
    fn build_instructions(&self, config: &PersonaConfig, language: &str) -> String;

    /// Get persona config for a segment ID
    ///
    /// Replaces hardcoded Persona::for_segment() match statement
    fn persona_for_segment(&self, segment_id: &str) -> Option<PersonaConfig>;

    /// Get key messages for a segment in a language
    ///
    /// Replaces hardcoded CustomerSegment::key_messages()
    fn key_messages(&self, segment_id: &str, language: &str) -> Vec<String>;

    /// Get suggested warmth for a segment
    ///
    /// Replaces hardcoded CustomerSegment::suggested_warmth()
    fn suggested_warmth(&self, segment_id: &str) -> f32;

    /// Get adaptation rule for a signal type (e.g., "frustration_detected")
    fn adaptation_rule(&self, signal: &str) -> Option<&AdaptationRule>;

    /// Apply adaptation rule to persona config
    fn apply_adaptation(&self, config: &mut PersonaConfig, signal: &str);

    /// Get all available tone IDs
    fn tone_ids(&self) -> Vec<&str>;

    /// Check if a tone ID exists
    fn has_tone(&self, tone_id: &str) -> bool;

    /// Get emotion acknowledgment phrases for an emotion type
    fn emotion_phrases(&self, emotion: &str, language: &str) -> Vec<String>;

    /// Get response length guideline for max_words
    fn response_length_guideline(&self, max_words: usize) -> Option<String>;
}

/// Config-driven persona provider implementation
///
/// Reads from PersonasConfig and SegmentsConfig to provide persona services.
pub struct ConfigPersonaProvider {
    tones: HashMap<String, ToneConfig>,
    warmth_thresholds: Vec<(f32, String)>,
    empathy_thresholds: Vec<(f32, String)>,
    complexity_instructions: HashMap<String, HashMap<String, String>>,
    urgency_instructions: HashMap<String, HashMap<String, String>>,
    hinglish_enabled: HashMap<String, String>,
    hinglish_disabled: HashMap<String, String>,
    adaptation_rules: HashMap<String, AdaptationRule>,
    segment_personas: HashMap<String, PersonaConfig>,
    segment_key_messages: HashMap<String, HashMap<String, Vec<String>>>,
    emotion_phrases: HashMap<String, HashMap<String, Vec<String>>>,
    response_guidelines: Vec<(usize, usize, String)>,
}

impl ConfigPersonaProvider {
    /// Create a new config persona provider
    pub fn new() -> Self {
        Self {
            tones: HashMap::new(),
            warmth_thresholds: Vec::new(),
            empathy_thresholds: Vec::new(),
            complexity_instructions: HashMap::new(),
            urgency_instructions: HashMap::new(),
            hinglish_enabled: HashMap::new(),
            hinglish_disabled: HashMap::new(),
            adaptation_rules: HashMap::new(),
            segment_personas: HashMap::new(),
            segment_key_messages: HashMap::new(),
            emotion_phrases: HashMap::new(),
            response_guidelines: Vec::new(),
        }
    }

    /// Add a tone configuration
    pub fn with_tone(mut self, id: impl Into<String>, config: ToneConfig) -> Self {
        self.tones.insert(id.into(), config);
        self
    }

    /// Add warmth threshold (min_value, instruction)
    pub fn with_warmth_threshold(mut self, min: f32, instruction: impl Into<String>) -> Self {
        self.warmth_thresholds.push((min, instruction.into()));
        // Keep sorted by min descending
        self.warmth_thresholds
            .sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        self
    }

    /// Add empathy threshold (min_value, instruction)
    pub fn with_empathy_threshold(mut self, min: f32, instruction: impl Into<String>) -> Self {
        self.empathy_thresholds.push((min, instruction.into()));
        // Keep sorted by min descending
        self.empathy_thresholds
            .sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        self
    }

    /// Add complexity level instruction
    pub fn with_complexity(
        mut self,
        level: impl Into<String>,
        language: impl Into<String>,
        instruction: impl Into<String>,
    ) -> Self {
        self.complexity_instructions
            .entry(level.into())
            .or_insert_with(HashMap::new)
            .insert(language.into(), instruction.into());
        self
    }

    /// Add urgency level instruction
    pub fn with_urgency(
        mut self,
        level: impl Into<String>,
        language: impl Into<String>,
        instruction: impl Into<String>,
    ) -> Self {
        self.urgency_instructions
            .entry(level.into())
            .or_insert_with(HashMap::new)
            .insert(language.into(), instruction.into());
        self
    }

    /// Add hinglish instructions
    pub fn with_hinglish(
        mut self,
        language: impl Into<String>,
        enabled: impl Into<String>,
        disabled: impl Into<String>,
    ) -> Self {
        let lang = language.into();
        self.hinglish_enabled.insert(lang.clone(), enabled.into());
        self.hinglish_disabled.insert(lang, disabled.into());
        self
    }

    /// Add adaptation rule
    pub fn with_adaptation_rule(mut self, signal: impl Into<String>, rule: AdaptationRule) -> Self {
        self.adaptation_rules.insert(signal.into(), rule);
        self
    }

    /// Add segment persona config
    pub fn with_segment_persona(
        mut self,
        segment_id: impl Into<String>,
        config: PersonaConfig,
    ) -> Self {
        self.segment_personas.insert(segment_id.into(), config);
        self
    }

    /// Add segment key messages
    pub fn with_segment_key_messages(
        mut self,
        segment_id: impl Into<String>,
        language: impl Into<String>,
        messages: Vec<String>,
    ) -> Self {
        self.segment_key_messages
            .entry(segment_id.into())
            .or_insert_with(HashMap::new)
            .insert(language.into(), messages);
        self
    }

    /// Add emotion phrases
    pub fn with_emotion_phrases(
        mut self,
        emotion: impl Into<String>,
        language: impl Into<String>,
        phrases: Vec<String>,
    ) -> Self {
        self.emotion_phrases
            .entry(emotion.into())
            .or_insert_with(HashMap::new)
            .insert(language.into(), phrases);
        self
    }

    /// Add response length guideline
    pub fn with_response_guideline(
        mut self,
        min_words: usize,
        max_words: usize,
        guideline: impl Into<String>,
    ) -> Self {
        self.response_guidelines
            .push((min_words, max_words, guideline.into()));
        self
    }
}

impl Default for ConfigPersonaProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl PersonaProvider for ConfigPersonaProvider {
    fn tone_config(&self, tone_id: &str) -> Option<&ToneConfig> {
        self.tones.get(tone_id)
    }

    fn greeting_prefix(&self, tone_id: &str, language: &str) -> Option<&str> {
        self.tones.get(tone_id).and_then(|t| t.greeting(language))
    }

    fn closing_phrase(&self, tone_id: &str, language: &str) -> Option<&str> {
        self.tones.get(tone_id).and_then(|t| t.closing(language))
    }

    fn tone_instructions(&self, tone_id: &str, language: &str) -> Option<&str> {
        self.tones.get(tone_id).and_then(|t| t.instruction(language))
    }

    fn warmth_instruction(&self, warmth: f32, language: &str) -> Option<String> {
        for (min, instruction) in &self.warmth_thresholds {
            if warmth >= *min {
                return Some(instruction.clone());
            }
        }
        None
    }

    fn empathy_instruction(&self, empathy: f32, language: &str) -> Option<String> {
        for (min, instruction) in &self.empathy_thresholds {
            if empathy >= *min {
                return Some(instruction.clone());
            }
        }
        None
    }

    fn complexity_instruction(&self, level: &str, language: &str) -> Option<String> {
        self.complexity_instructions.get(level).and_then(|langs| {
            langs
                .get(language)
                .or_else(|| langs.get("en"))
                .cloned()
        })
    }

    fn urgency_instruction(&self, level: &str, language: &str) -> Option<String> {
        self.urgency_instructions.get(level).and_then(|langs| {
            langs
                .get(language)
                .or_else(|| langs.get("en"))
                .cloned()
        })
    }

    fn hinglish_instruction(&self, enabled: bool, language: &str) -> Option<String> {
        if enabled {
            self.hinglish_enabled
                .get(language)
                .or_else(|| self.hinglish_enabled.get("en"))
                .cloned()
        } else {
            self.hinglish_disabled
                .get(language)
                .or_else(|| self.hinglish_disabled.get("en"))
                .cloned()
        }
    }

    fn build_instructions(&self, config: &PersonaConfig, language: &str) -> String {
        let mut instructions = Vec::new();

        // Tone instruction
        if let Some(inst) = self.tone_instructions(&config.tone, language) {
            instructions.push(inst.to_string());
        }

        // Warmth instruction
        if let Some(inst) = self.warmth_instruction(config.warmth, language) {
            instructions.push(inst);
        }

        // Empathy instruction
        if let Some(inst) = self.empathy_instruction(config.empathy, language) {
            instructions.push(inst);
        }

        // Complexity instruction
        if let Some(inst) = self.complexity_instruction(&config.language_complexity, language) {
            instructions.push(inst);
        }

        // Urgency instruction
        if let Some(inst) = self.urgency_instruction(&config.urgency, language) {
            instructions.push(inst);
        }

        // Hinglish instruction
        if let Some(inst) = self.hinglish_instruction(config.use_hinglish, language) {
            instructions.push(inst);
        }

        // Response length guideline
        if let Some(guideline) = self.response_length_guideline(config.max_response_words) {
            instructions.push(guideline);
        }

        // Emotion acknowledgment note
        if config.acknowledge_emotions {
            instructions.push("Acknowledge customer emotions when appropriate.".to_string());
        }

        instructions.join(" ")
    }

    fn persona_for_segment(&self, segment_id: &str) -> Option<PersonaConfig> {
        self.segment_personas.get(segment_id).cloned()
    }

    fn key_messages(&self, segment_id: &str, language: &str) -> Vec<String> {
        self.segment_key_messages
            .get(segment_id)
            .and_then(|langs| {
                langs
                    .get(language)
                    .or_else(|| langs.get("en"))
                    .cloned()
            })
            .unwrap_or_default()
    }

    fn suggested_warmth(&self, segment_id: &str) -> f32 {
        self.segment_personas
            .get(segment_id)
            .map(|p| p.warmth)
            .unwrap_or(0.8)
    }

    fn adaptation_rule(&self, signal: &str) -> Option<&AdaptationRule> {
        self.adaptation_rules.get(signal)
    }

    fn apply_adaptation(&self, config: &mut PersonaConfig, signal: &str) {
        if let Some(rule) = self.adaptation_rules.get(signal) {
            // Apply warmth changes
            if let Some(override_val) = rule.warmth_override {
                config.warmth = override_val;
            } else if let Some(adj) = rule.warmth_adjustment {
                config.warmth = (config.warmth + adj).clamp(0.0, 1.0);
            }

            // Apply empathy changes
            if let Some(override_val) = rule.empathy_override {
                config.empathy = override_val;
            } else if let Some(adj) = rule.empathy_adjustment {
                config.empathy = (config.empathy + adj).clamp(0.0, 1.0);
            }

            // Apply complexity override
            if let Some(ref complexity) = rule.complexity_override {
                config.language_complexity = complexity.clone();
            }

            // Apply urgency override
            if let Some(ref urgency) = rule.urgency_override {
                config.urgency = urgency.clone();
            }
        }
    }

    fn tone_ids(&self) -> Vec<&str> {
        self.tones.keys().map(|s| s.as_str()).collect()
    }

    fn has_tone(&self, tone_id: &str) -> bool {
        self.tones.contains_key(tone_id)
    }

    fn emotion_phrases(&self, emotion: &str, language: &str) -> Vec<String> {
        self.emotion_phrases
            .get(emotion)
            .and_then(|langs| {
                langs
                    .get(language)
                    .or_else(|| langs.get("en"))
                    .cloned()
            })
            .unwrap_or_default()
    }

    fn response_length_guideline(&self, max_words: usize) -> Option<String> {
        for (min, max, guideline) in &self.response_guidelines {
            if max_words >= *min && max_words <= *max {
                return Some(guideline.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_provider() -> ConfigPersonaProvider {
        ConfigPersonaProvider::new()
            .with_tone(
                "formal",
                ToneConfig {
                    description: "Formal tone".to_string(),
                    greeting_prefix: [("en".to_string(), "Respected".to_string())]
                        .into_iter()
                        .collect(),
                    closing_phrase: [("en".to_string(), "Thank you.".to_string())]
                        .into_iter()
                        .collect(),
                    instructions: [("en".to_string(), "Use formal language.".to_string())]
                        .into_iter()
                        .collect(),
                },
            )
            .with_warmth_threshold(0.8, "Be very warm.")
            .with_warmth_threshold(0.0, "Be professional.")
            .with_segment_persona(
                "high_value",
                PersonaConfig {
                    name: "premium_advisor".to_string(),
                    tone: "formal".to_string(),
                    warmth: 0.9,
                    empathy: 0.8,
                    ..Default::default()
                },
            )
            .with_segment_key_messages(
                "high_value",
                "en",
                vec!["Priority processing".to_string(), "Dedicated manager".to_string()],
            )
    }

    #[test]
    fn test_greeting_prefix() {
        let provider = test_provider();
        assert_eq!(provider.greeting_prefix("formal", "en"), Some("Respected"));
    }

    #[test]
    fn test_warmth_instruction() {
        let provider = test_provider();
        assert_eq!(
            provider.warmth_instruction(0.9, "en"),
            Some("Be very warm.".to_string())
        );
        assert_eq!(
            provider.warmth_instruction(0.5, "en"),
            Some("Be professional.".to_string())
        );
    }

    #[test]
    fn test_persona_for_segment() {
        let provider = test_provider();
        let persona = provider.persona_for_segment("high_value").unwrap();
        assert_eq!(persona.name, "premium_advisor");
        assert_eq!(persona.tone, "formal");
        assert_eq!(persona.warmth, 0.9);
    }

    #[test]
    fn test_key_messages() {
        let provider = test_provider();
        let messages = provider.key_messages("high_value", "en");
        assert_eq!(messages.len(), 2);
        assert!(messages.contains(&"Priority processing".to_string()));
    }

    #[test]
    fn test_build_instructions() {
        let provider = test_provider();
        let config = PersonaConfig {
            tone: "formal".to_string(),
            warmth: 0.9,
            ..Default::default()
        };
        let instructions = provider.build_instructions(&config, "en");
        assert!(instructions.contains("Use formal language."));
        assert!(instructions.contains("Be very warm."));
    }
}
