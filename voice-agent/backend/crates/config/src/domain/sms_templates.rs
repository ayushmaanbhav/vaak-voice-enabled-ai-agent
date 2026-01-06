//! SMS Templates Configuration
//!
//! Defines SMS message templates loaded from YAML for the SendSmsTool.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// SMS templates configuration loaded from sms_templates.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsTemplatesConfig {
    /// SMS templates keyed by type, then by language
    #[serde(default)]
    pub templates: HashMap<String, HashMap<String, String>>,
    /// SMS configuration settings
    #[serde(default)]
    pub config: SmsConfig,
}

impl Default for SmsTemplatesConfig {
    fn default() -> Self {
        Self {
            templates: HashMap::new(),
            config: SmsConfig::default(),
        }
    }
}

impl SmsTemplatesConfig {
    /// Load from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, SmsTemplatesConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            SmsTemplatesConfigError::FileNotFound(
                path.as_ref().display().to_string(),
                e.to_string(),
            )
        })?;

        serde_yaml::from_str(&content)
            .map_err(|e| SmsTemplatesConfigError::ParseError(e.to_string()))
    }

    /// Get template by type and language
    pub fn get_template(&self, template_type: &str, language: &str) -> Option<&str> {
        self.templates
            .get(template_type)
            .and_then(|langs| {
                langs
                    .get(language)
                    .or_else(|| langs.get(&self.config.default_language))
            })
            .map(|s| s.as_str())
    }

    /// Get all template types
    pub fn template_types(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }

    /// Build message from template with placeholder substitution
    pub fn build_message(
        &self,
        template_type: &str,
        language: &str,
        placeholders: &HashMap<String, String>,
    ) -> Option<String> {
        let template = self.get_template(template_type, language)?;
        let mut message = template.to_string();

        for (key, value) in placeholders {
            message = message.replace(&format!("{{{}}}", key), value);
        }

        Some(message)
    }

    /// Check if template type is transactional
    pub fn is_transactional(&self, template_type: &str) -> bool {
        self.config
            .categories
            .transactional
            .contains(&template_type.to_string())
    }

    /// Check if template type is promotional
    pub fn is_promotional(&self, template_type: &str) -> bool {
        self.config
            .categories
            .promotional
            .contains(&template_type.to_string())
    }
}

/// SMS configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsConfig {
    /// Maximum message length
    #[serde(default = "default_max_length")]
    pub max_length: usize,
    /// Unicode message max length
    #[serde(default = "default_unicode_max_length")]
    pub unicode_max_length: usize,
    /// Default language
    #[serde(default = "default_language")]
    pub default_language: String,
    /// Sender ID
    #[serde(default)]
    pub sender_id: String,
    /// Template categories
    #[serde(default)]
    pub categories: SmsCategories,
}

fn default_max_length() -> usize {
    160
}

fn default_unicode_max_length() -> usize {
    70
}

fn default_language() -> String {
    "en".to_string()
}

impl Default for SmsConfig {
    fn default() -> Self {
        Self {
            max_length: default_max_length(),
            unicode_max_length: default_unicode_max_length(),
            default_language: default_language(),
            sender_id: String::new(),
            categories: SmsCategories::default(),
        }
    }
}

/// SMS template categories for compliance
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmsCategories {
    #[serde(default)]
    pub transactional: Vec<String>,
    #[serde(default)]
    pub promotional: Vec<String>,
    #[serde(default)]
    pub service: Vec<String>,
}

/// Errors when loading SMS templates configuration
#[derive(Debug)]
pub enum SmsTemplatesConfigError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for SmsTemplatesConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => {
                write!(f, "SMS templates config not found at {}: {}", path, err)
            }
            Self::ParseError(err) => write!(f, "Failed to parse SMS templates config: {}", err),
        }
    }
}

impl std::error::Error for SmsTemplatesConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sms_templates_deserialization() {
        let yaml = r#"
templates:
  appointment_confirmation:
    en: "Dear {customer_name}, your appointment is confirmed."
    hi: "प्रिय {customer_name}, आपकी अपॉइंटमेंट कन्फर्म है।"
config:
  max_length: 160
  default_language: "en"
"#;
        let config: SmsTemplatesConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.templates.len(), 1);
        assert!(config.templates.contains_key("appointment_confirmation"));
    }

    #[test]
    fn test_get_template() {
        let mut templates = HashMap::new();
        let mut langs = HashMap::new();
        langs.insert("en".to_string(), "Hello {name}".to_string());
        langs.insert("hi".to_string(), "नमस्ते {name}".to_string());
        templates.insert("greeting".to_string(), langs);

        let config = SmsTemplatesConfig {
            templates,
            config: SmsConfig::default(),
        };

        assert_eq!(config.get_template("greeting", "en"), Some("Hello {name}"));
        assert_eq!(
            config.get_template("greeting", "hi"),
            Some("नमस्ते {name}")
        );
        // Fallback to default language
        assert_eq!(config.get_template("greeting", "fr"), Some("Hello {name}"));
    }

    #[test]
    fn test_build_message() {
        let mut templates = HashMap::new();
        let mut langs = HashMap::new();
        langs.insert(
            "en".to_string(),
            "Hello {name}, your appointment is on {date}".to_string(),
        );
        templates.insert("appointment".to_string(), langs);

        let config = SmsTemplatesConfig {
            templates,
            config: SmsConfig::default(),
        };

        let mut placeholders = HashMap::new();
        placeholders.insert("name".to_string(), "John".to_string());
        placeholders.insert("date".to_string(), "Jan 15".to_string());

        let message = config.build_message("appointment", "en", &placeholders);
        assert_eq!(
            message,
            Some("Hello John, your appointment is on Jan 15".to_string())
        );
    }
}
