//! Prompt Template Configuration
//!
//! Defines config-driven prompt templates for LLM interactions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Prompts configuration loaded from prompts/system.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsConfig {
    /// Main system prompt template
    #[serde(default)]
    pub system_prompt: String,
    /// Persona trait templates
    #[serde(default)]
    pub persona_traits: HashMap<String, String>,
    /// Language style descriptions
    #[serde(default)]
    pub language_styles: HashMap<String, String>,
    /// Key facts template
    #[serde(default)]
    pub key_facts_template: String,
    /// RAG context injection template
    #[serde(default)]
    pub rag_context_template: String,
    /// Customer profile template
    #[serde(default)]
    pub customer_profile_template: String,
    /// Stage guidance wrapper template
    #[serde(default)]
    pub stage_guidance_template: String,
    /// Tool injection template
    #[serde(default)]
    pub tool_injection_template: String,
    /// Response templates by scenario and language
    #[serde(default)]
    pub response_templates: HashMap<String, HashMap<String, String>>,
    /// Error response templates
    #[serde(default)]
    pub error_templates: HashMap<String, HashMap<String, String>>,
}

impl Default for PromptsConfig {
    fn default() -> Self {
        Self {
            system_prompt: String::new(),
            persona_traits: HashMap::new(),
            language_styles: HashMap::new(),
            key_facts_template: String::new(),
            rag_context_template: String::new(),
            customer_profile_template: String::new(),
            stage_guidance_template: String::new(),
            tool_injection_template: String::new(),
            response_templates: HashMap::new(),
            error_templates: HashMap::new(),
        }
    }
}

impl PromptsConfig {
    /// Load from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, PromptsConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            PromptsConfigError::FileNotFound(path.as_ref().display().to_string(), e.to_string())
        })?;

        serde_yaml::from_str(&content).map_err(|e| PromptsConfigError::ParseError(e.to_string()))
    }

    /// Get language style description
    pub fn language_style(&self, language: &str) -> &str {
        self.language_styles
            .get(language)
            .map(|s| s.as_str())
            .unwrap_or("English")
    }

    /// Get persona trait by key
    pub fn persona_trait(&self, key: &str) -> Option<&str> {
        self.persona_traits.get(key).map(|s| s.as_str())
    }

    /// Get response template for a scenario and language
    pub fn response_template(&self, scenario: &str, language: &str) -> Option<&str> {
        self.response_templates
            .get(scenario)
            .and_then(|lang_map| lang_map.get(language).map(|s| s.as_str()))
    }

    /// Get error template for a scenario and language
    pub fn error_template(&self, scenario: &str, language: &str) -> Option<&str> {
        self.error_templates
            .get(scenario)
            .and_then(|lang_map| lang_map.get(language).map(|s| s.as_str()))
    }

    /// Build persona traits string from config values
    pub fn build_persona_traits(&self, warmth: f32, empathy: f32, formality: f32, urgency: f32) -> String {
        let mut traits = Vec::new();

        if warmth > 0.7 {
            if let Some(t) = self.persona_trait("warmth_high") {
                traits.push(t.to_string());
            }
        }
        if empathy > 0.8 {
            if let Some(t) = self.persona_trait("empathy_high") {
                traits.push(t.to_string());
            }
        }
        if formality < 0.4 {
            if let Some(t) = self.persona_trait("formality_low") {
                traits.push(t.to_string());
            }
        } else if formality > 0.7 {
            if let Some(t) = self.persona_trait("formality_high") {
                traits.push(t.to_string());
            }
        } else if let Some(t) = self.persona_trait("formality_medium") {
            traits.push(t.to_string());
        }
        if urgency > 0.6 {
            if let Some(t) = self.persona_trait("urgency_high") {
                traits.push(t.to_string());
            }
        }

        traits.join("\n")
    }

    /// Build system prompt with substitutions
    pub fn build_system_prompt(
        &self,
        agent_name: &str,
        bank_name: &str,
        persona_traits: &str,
        language: &str,
        key_facts: &str,
        helpline: &str,
    ) -> String {
        let language_style = self.language_style(language);

        self.system_prompt
            .replace("{agent_name}", agent_name)
            .replace("{bank_name}", bank_name)
            .replace("{persona_traits}", persona_traits)
            .replace("{language_style}", language_style)
            .replace("{key_facts}", key_facts)
            .replace("{helpline}", helpline)
    }

    /// Build RAG context message
    pub fn build_rag_context(&self, context: &str) -> String {
        if context.is_empty() {
            return String::new();
        }
        self.rag_context_template.replace("{context}", context)
    }

    /// Build stage guidance message
    pub fn build_stage_guidance(&self, guidance: &str) -> String {
        if guidance.is_empty() {
            return String::new();
        }
        self.stage_guidance_template.replace("{guidance}", guidance)
    }
}

/// Errors when loading prompts configuration
#[derive(Debug)]
pub enum PromptsConfigError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for PromptsConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => {
                write!(f, "Prompts config not found at {}: {}", path, err)
            }
            Self::ParseError(err) => write!(f, "Failed to parse prompts config: {}", err),
        }
    }
}

impl std::error::Error for PromptsConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompts_deserialization() {
        let yaml = r#"
system_prompt: "You are {agent_name} from {bank_name}."

persona_traits:
  warmth_high: "- Warm and welcoming"
  empathy_high: "- Highly empathetic"

language_styles:
  en: "English"
  hi: "Hindi-English (Hinglish)"

response_templates:
  greeting:
    en: "Hello, I'm {agent_name}!"
    hi: "नमस्ते, मैं {agent_name} हूं!"
"#;
        let config: PromptsConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.system_prompt.contains("{agent_name}"));
        assert_eq!(config.language_style("en"), "English");
        assert_eq!(config.language_style("hi"), "Hindi-English (Hinglish)");
        assert_eq!(
            config.response_template("greeting", "en"),
            Some("Hello, I'm {agent_name}!")
        );
    }

    #[test]
    fn test_build_system_prompt() {
        let mut config = PromptsConfig::default();
        config.system_prompt = "You are {agent_name} from {bank_name}. Language: {language_style}".to_string();
        config.language_styles.insert("en".to_string(), "English".to_string());

        let result = config.build_system_prompt(
            "Priya",
            "Kotak Bank",
            "",
            "en",
            "",
            "",
        );

        assert!(result.contains("Priya"));
        assert!(result.contains("Kotak Bank"));
        assert!(result.contains("English"));
    }

    #[test]
    fn test_build_persona_traits() {
        let mut config = PromptsConfig::default();
        config.persona_traits.insert("warmth_high".to_string(), "- Warm".to_string());
        config.persona_traits.insert("empathy_high".to_string(), "- Empathetic".to_string());
        config.persona_traits.insert("formality_medium".to_string(), "- Balanced".to_string());

        let traits = config.build_persona_traits(0.8, 0.9, 0.5, 0.5);
        assert!(traits.contains("Warm"));
        assert!(traits.contains("Empathetic"));
        assert!(traits.contains("Balanced"));
    }
}
