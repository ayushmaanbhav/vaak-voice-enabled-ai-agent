//! Objection Handling Configuration
//!
//! Defines config-driven objection detection patterns and responses.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Objections configuration loaded from objections.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectionsConfig {
    /// Objection definitions keyed by objection type
    #[serde(default)]
    pub objections: HashMap<String, ObjectionDefinition>,
    /// Default objection for unrecognized concerns
    #[serde(default)]
    pub default_objection: Option<ObjectionDefinition>,
}

impl Default for ObjectionsConfig {
    fn default() -> Self {
        Self {
            objections: HashMap::new(),
            default_objection: None,
        }
    }
}

impl ObjectionsConfig {
    /// Load from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ObjectionsConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            ObjectionsConfigError::FileNotFound(path.as_ref().display().to_string(), e.to_string())
        })?;

        serde_yaml::from_str(&content)
            .map_err(|e| ObjectionsConfigError::ParseError(e.to_string()))
    }

    /// Get objection definition by type
    pub fn get_objection(&self, objection_type: &str) -> Option<&ObjectionDefinition> {
        self.objections.get(objection_type)
    }

    /// Get all objection type names
    pub fn objection_types(&self) -> Vec<&str> {
        self.objections.keys().map(|s| s.as_str()).collect()
    }

    /// Detect objection type from text
    pub fn detect_objection(&self, text: &str, language: &str) -> Option<&str> {
        let text_lower = text.to_lowercase();

        for (objection_type, definition) in &self.objections {
            if let Some(patterns) = definition.patterns.get(language) {
                for pattern in patterns {
                    if text_lower.contains(&pattern.to_lowercase()) {
                        return Some(objection_type.as_str());
                    }
                }
            }
            // Fallback to English patterns if language-specific not found
            if language != "en" {
                if let Some(patterns) = definition.patterns.get("en") {
                    for pattern in patterns {
                        if text_lower.contains(&pattern.to_lowercase()) {
                            return Some(objection_type.as_str());
                        }
                    }
                }
            }
        }

        None
    }

    /// Get response for an objection type and language
    pub fn get_response(&self, objection_type: &str, language: &str) -> Option<&ObjectionResponse> {
        self.objections
            .get(objection_type)
            .and_then(|def| {
                def.responses
                    .get(language)
                    .or_else(|| def.responses.get("en")) // Fallback to English
            })
    }

    /// Get default response for unrecognized objections
    pub fn get_default_response(&self, language: &str) -> Option<&ObjectionResponse> {
        self.default_objection.as_ref().and_then(|def| {
            def.responses
                .get(language)
                .or_else(|| def.responses.get("en"))
        })
    }

    /// Build full response text from components
    pub fn build_full_response(&self, objection_type: &str, language: &str) -> Option<String> {
        self.get_response(objection_type, language)
            .map(|r| r.full_response())
    }
}

/// Definition for a single objection type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectionDefinition {
    /// Human-readable display name
    #[serde(default)]
    pub display_name: String,
    /// Description of the objection
    #[serde(default)]
    pub description: String,
    /// Detection patterns by language
    #[serde(default)]
    pub patterns: HashMap<String, Vec<String>>,
    /// Response components by language
    #[serde(default)]
    pub responses: HashMap<String, ObjectionResponse>,
}

/// Objection response components (acknowledge-reframe-evidence-CTA pattern)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectionResponse {
    /// Acknowledgment of the concern (validates customer feeling)
    #[serde(default)]
    pub acknowledge: String,
    /// Reframe to shift perspective
    #[serde(default)]
    pub reframe: String,
    /// Evidence/facts to support the reframe
    #[serde(default)]
    pub evidence: String,
    /// Call to action
    #[serde(default)]
    pub call_to_action: String,
}

impl ObjectionResponse {
    /// Build full response from components
    pub fn full_response(&self) -> String {
        format!(
            "{} {} {} {}",
            self.acknowledge, self.reframe, self.evidence, self.call_to_action
        )
    }

    /// Get acknowledgment only
    pub fn acknowledge_only(&self) -> &str {
        &self.acknowledge
    }

    /// Get acknowledge + reframe
    pub fn acknowledge_and_reframe(&self) -> String {
        format!("{} {}", self.acknowledge, self.reframe)
    }
}

/// Errors when loading objections configuration
#[derive(Debug)]
pub enum ObjectionsConfigError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for ObjectionsConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => {
                write!(f, "Objections config not found at {}: {}", path, err)
            }
            Self::ParseError(err) => write!(f, "Failed to parse objections config: {}", err),
        }
    }
}

impl std::error::Error for ObjectionsConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_objections_deserialization() {
        let yaml = r#"
objections:
  safety:
    display_name: "Safety Concerns"
    description: "Customer worried about safety"
    patterns:
      en:
        - "safe"
        - "trust"
      hi:
        - "bharosa"
    responses:
      en:
        acknowledge: "I understand your concern."
        reframe: "Banks are actually safer."
        evidence: "We've never lost gold."
        call_to_action: "Would you like to learn more?"
      hi:
        acknowledge: "मैं समझता हूं।"
        reframe: "बैंक ज़्यादा सुरक्षित है।"
        evidence: "हमने कभी सोना नहीं खोया।"
        call_to_action: "क्या आप और जानना चाहेंगे?"
"#;
        let config: ObjectionsConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.objections.len(), 1);

        let safety = config.get_objection("safety").unwrap();
        assert_eq!(safety.display_name, "Safety Concerns");
        assert_eq!(safety.patterns.get("en").unwrap().len(), 2);

        let en_response = safety.responses.get("en").unwrap();
        assert!(en_response.acknowledge.contains("understand"));
    }

    #[test]
    fn test_detect_objection() {
        let yaml = r#"
objections:
  safety:
    patterns:
      en:
        - "safe"
        - "trust"
  rate:
    patterns:
      en:
        - "interest"
        - "rate"
"#;
        let config: ObjectionsConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.detect_objection("is it safe?", "en"), Some("safety"));
        assert_eq!(
            config.detect_objection("what's the interest rate?", "en"),
            Some("rate")
        );
        assert_eq!(config.detect_objection("hello", "en"), None);
    }

    #[test]
    fn test_full_response() {
        let response = ObjectionResponse {
            acknowledge: "I understand.".to_string(),
            reframe: "Let me explain.".to_string(),
            evidence: "We have proof.".to_string(),
            call_to_action: "Want to continue?".to_string(),
        };

        assert_eq!(
            response.full_response(),
            "I understand. Let me explain. We have proof. Want to continue?"
        );
    }
}
