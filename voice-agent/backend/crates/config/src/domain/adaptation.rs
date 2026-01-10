//! Adaptation Configuration
//!
//! # P21 FIX: Config-driven personalization variables
//!
//! Defines adaptation rules and variables loaded from domain config YAML.
//! Enables domain-agnostic personalization by moving all domain-specific
//! terminology and segment adaptations to configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Root adaptation configuration loaded from adaptation.yaml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdaptationConfig {
    /// Schema version
    #[serde(default = "default_schema_version")]
    pub schema_version: String,

    /// Domain-specific variables for template substitution
    /// These can be referenced as {{variable_name}} in other config files
    #[serde(default)]
    pub variables: HashMap<String, String>,

    /// Segment-specific adaptations
    #[serde(default)]
    pub segment_adaptations: HashMap<String, SegmentAdaptation>,

    /// Collateral-specific terminology mappings
    /// Maps generic terms to domain-specific terms
    #[serde(default)]
    pub terminology: HashMap<String, String>,

    /// Feature flags for domain-specific behaviors
    #[serde(default)]
    pub enabled_features: HashMap<String, bool>,
}

fn default_schema_version() -> String {
    "1.0".to_string()
}

/// Segment-specific adaptation configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SegmentAdaptation {
    /// Special program configuration (if any)
    #[serde(default)]
    pub special_program: Option<SpecialProgram>,

    /// Primary features to emphasize for this segment
    #[serde(default)]
    pub primary_features: Vec<String>,

    /// Custom value propositions by language
    #[serde(default)]
    pub value_propositions: HashMap<String, Vec<String>>,
}

/// Special program configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialProgram {
    /// Whether this program is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Program identifier
    #[serde(default)]
    pub program_id: String,

    /// Program display name (can use {{variable}} substitution)
    #[serde(default)]
    pub name: String,

    /// Program benefit description (can use {{variable}} substitution)
    #[serde(default)]
    pub benefit: String,

    /// Discount percentage for this program
    #[serde(default)]
    pub discount_percent: f64,
}

impl AdaptationConfig {
    /// Load adaptation config from YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, AdaptationConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            AdaptationConfigError::FileNotFound(path.as_ref().display().to_string(), e.to_string())
        })?;

        serde_yaml::from_str(&content)
            .map_err(|e| AdaptationConfigError::ParseError(e.to_string()))
    }

    /// Get a variable value by name
    pub fn get_variable(&self, name: &str) -> Option<&str> {
        self.variables.get(name).map(|s| s.as_str())
    }

    /// Get terminology mapping
    pub fn get_terminology(&self, term: &str) -> Option<&str> {
        self.terminology.get(term).map(|s| s.as_str())
    }

    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.enabled_features.get(feature).copied().unwrap_or(false)
    }

    /// Get segment adaptation
    pub fn get_segment_adaptation(&self, segment_id: &str) -> Option<&SegmentAdaptation> {
        self.segment_adaptations.get(segment_id)
    }

    /// Get primary features for a segment
    pub fn get_segment_features(&self, segment_id: &str) -> Vec<&str> {
        self.segment_adaptations
            .get(segment_id)
            .map(|s| s.primary_features.iter().map(|f| f.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get value propositions for a segment in a specific language
    pub fn get_segment_value_propositions(&self, segment_id: &str, language: &str) -> Vec<&str> {
        self.segment_adaptations
            .get(segment_id)
            .and_then(|s| s.value_propositions.get(language))
            .map(|props| props.iter().map(|p| p.as_str()).collect())
            .unwrap_or_default()
    }

    /// Substitute variables in a template string
    /// Replaces {{variable_name}} with the corresponding variable value
    pub fn substitute_variables(&self, template: &str) -> String {
        let mut result = template.to_string();
        for (key, value) in &self.variables {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        result
    }
}

/// Errors during adaptation config loading
#[derive(Debug)]
pub enum AdaptationConfigError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for AdaptationConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => {
                write!(f, "Adaptation config not found at {}: {}", path, err)
            }
            Self::ParseError(err) => write!(f, "Failed to parse adaptation config: {}", err),
        }
    }
}

impl std::error::Error for AdaptationConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AdaptationConfig::default();
        assert!(config.variables.is_empty());
        assert!(config.segment_adaptations.is_empty());
    }

    #[test]
    fn test_variable_substitution() {
        let mut config = AdaptationConfig::default();
        config
            .variables
            .insert("product_name".to_string(), "Gold Loan".to_string());
        config
            .variables
            .insert("rate".to_string(), "9.5%".to_string());

        let template = "Get a {{product_name}} at just {{rate}}!";
        let result = config.substitute_variables(template);
        assert_eq!(result, "Get a Gold Loan at just 9.5%!");
    }

    #[test]
    fn test_feature_flag() {
        // P21 FIX: Use domain-agnostic feature names in tests
        let mut config = AdaptationConfig::default();
        config.enabled_features.insert("balance_transfer".to_string(), true);
        config.enabled_features.insert("premium_service".to_string(), false);

        assert!(config.is_feature_enabled("balance_transfer"));
        assert!(!config.is_feature_enabled("premium_service"));
        assert!(!config.is_feature_enabled("unknown_feature"));
    }
}
