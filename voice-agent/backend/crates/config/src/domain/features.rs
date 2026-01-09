//! Features configuration
//!
//! Defines features to emphasize per customer segment and value propositions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Feature ID - string-based identifier
pub type FeatureId = String;

/// Features configuration loaded from features.yaml
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeaturesConfig {
    /// Feature definitions
    #[serde(default)]
    pub features: HashMap<String, FeatureDefinition>,

    /// Segment to feature priority mappings
    #[serde(default)]
    pub segment_features: HashMap<String, Vec<String>>,

    /// Value propositions per segment
    #[serde(default)]
    pub value_propositions: HashMap<String, Vec<String>>,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            features: HashMap::new(),
            segment_features: HashMap::new(),
            value_propositions: HashMap::new(),
        }
    }
}

/// Feature definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeatureDefinition {
    /// Display names by language
    pub display_name: HashMap<String, String>,

    /// Description
    #[serde(default)]
    pub description: String,
}

impl FeaturesConfig {
    /// Get display name for a feature in a specific language
    pub fn display_name(&self, feature_id: &str, language: &str) -> Option<&str> {
        self.features.get(feature_id).and_then(|f| {
            f.display_name
                .get(language)
                .or_else(|| f.display_name.get("en"))
                .map(|s| s.as_str())
        })
    }

    /// Get features for a segment
    pub fn features_for_segment(&self, segment_id: &str) -> Vec<&str> {
        self.segment_features
            .get(segment_id)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get top N features for a segment
    pub fn top_features_for_segment(&self, segment_id: &str, n: usize) -> Vec<&str> {
        self.features_for_segment(segment_id)
            .into_iter()
            .take(n)
            .collect()
    }

    /// Get value propositions for a segment
    pub fn value_propositions_for_segment(&self, segment_id: &str) -> Vec<&str> {
        self.value_propositions
            .get(segment_id)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get value propositions with rate substitution
    pub fn value_propositions_with_rate(
        &self,
        segment_id: &str,
        our_best_rate: f64,
    ) -> Vec<String> {
        self.value_propositions
            .get(segment_id)
            .map(|v| {
                v.iter()
                    .map(|s| s.replace("{our_best_rate}", &format!("{:.1}", our_best_rate)))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a feature exists
    pub fn has_feature(&self, feature_id: &str) -> bool {
        self.features.contains_key(feature_id)
    }

    /// Get all feature IDs
    pub fn feature_ids(&self) -> Vec<&str> {
        self.features.keys().map(|s| s.as_str()).collect()
    }

    /// Get all segment IDs that have feature mappings
    pub fn segment_ids(&self) -> Vec<&str> {
        self.segment_features.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> FeaturesConfig {
        let yaml = r#"
features:
  low_rates:
    display_name:
      en: "Competitive Interest Rates"
      hi: "Kam Byaj Dar"
    description: "Low interest rates"
  security:
    display_name:
      en: "Bank-Grade Security"
      hi: "Surakshit"
    description: "Security and safety"

segment_features:
  price_sensitive:
    - low_rates
    - security
  trust_seeker:
    - security

value_propositions:
  price_sensitive:
    - "Starting at {our_best_rate}% - among the lowest rates"
    - "Zero foreclosure charges"
"#;
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn test_display_name() {
        let config = sample_config();

        assert_eq!(
            config.display_name("low_rates", "en"),
            Some("Competitive Interest Rates")
        );
        assert_eq!(
            config.display_name("low_rates", "hi"),
            Some("Kam Byaj Dar")
        );
        // Falls back to en
        assert_eq!(
            config.display_name("low_rates", "ta"),
            Some("Competitive Interest Rates")
        );
        assert_eq!(config.display_name("unknown", "en"), None);
    }

    #[test]
    fn test_features_for_segment() {
        let config = sample_config();

        let features = config.features_for_segment("price_sensitive");
        assert_eq!(features, vec!["low_rates", "security"]);

        let features = config.features_for_segment("trust_seeker");
        assert_eq!(features, vec!["security"]);

        let features = config.features_for_segment("unknown");
        assert!(features.is_empty());
    }

    #[test]
    fn test_top_features() {
        let config = sample_config();

        let features = config.top_features_for_segment("price_sensitive", 1);
        assert_eq!(features, vec!["low_rates"]);
    }

    #[test]
    fn test_value_propositions_with_rate() {
        let config = sample_config();

        let props = config.value_propositions_with_rate("price_sensitive", 9.5);
        assert_eq!(props.len(), 2);
        assert!(props[0].contains("9.5%"));
    }
}
