//! Feature Provider trait for domain-agnostic feature management
//!
//! This module provides a domain-agnostic interface for managing product features
//! and their display across different customer segments. All feature definitions
//! are loaded from configuration (features.yaml).
//!
//! # P20 FIX: Replaces hardcoded Feature enum in personalization/adaptation.rs
//!
//! The previous Feature enum had domain-specific variants with hardcoded
//! display strings. This trait enables fully config-driven features with
//! variable substitution, making the system truly domain-agnostic.
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_core::traits::FeatureProvider;
//!
//! // Provider is created from domain config
//! let provider = config_bridge.feature_provider();
//!
//! // Get feature display name with variable substitution
//! let name = provider.feature_display_name("women_benefits", "en", &variables);
//! // Returns configured display name with variables substituted
//!
//! // Get features for a segment
//! let features = provider.features_for_segment("women");
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Feature ID type alias for domain-agnostic feature references
pub type FeatureId = String;

/// Well-known feature IDs (convenience constants, not exhaustive)
///
/// These are common feature IDs that may be used across domains.
/// Domain-specific features should be defined in config, not here.
pub mod feature_ids {
    pub const LOW_RATES: &str = "low_rates";
    pub const QUICK_PROCESS: &str = "quick_process";
    pub const SECURITY: &str = "security";
    pub const TRANSPARENCY: &str = "transparency";
    pub const FLEXIBILITY: &str = "flexibility";
    pub const DIGITAL: &str = "digital";
    pub const RELATIONSHIP_MANAGER: &str = "relationship_manager";
    pub const HIGHER_LIMITS: &str = "higher_limits";
    pub const NO_HIDDEN_CHARGES: &str = "no_hidden_charges";
    pub const REGULATORY_COMPLIANT: &str = "regulatory_compliant";
    pub const ZERO_FORECLOSURE: &str = "zero_foreclosure";
    pub const DOORSTEP_SERVICE: &str = "doorstep_service";
    pub const SPECIAL_PROGRAM: &str = "special_program";
}

/// Feature display information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDisplay {
    /// Feature ID
    pub id: String,
    /// Display name (with variables substituted)
    pub display_name: String,
    /// Localized display name (e.g., Hindi)
    pub localized_name: Option<String>,
    /// Short description
    pub description: Option<String>,
    /// Icon identifier (for UI)
    pub icon: Option<String>,
    /// Badge text (e.g., "exclusive", "new")
    pub badge: Option<String>,
}

/// Feature emphasis for a segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturePriority {
    /// Feature ID
    pub feature_id: String,
    /// Priority order (1 = highest)
    pub priority: u8,
    /// Custom display text for this segment (optional)
    pub segment_text: Option<String>,
}

/// Value proposition for segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentValueProposition {
    /// The proposition text (with variables substituted)
    pub text: String,
    /// Language code
    pub language: String,
}

/// Variables for template substitution
pub type VariableMap = HashMap<String, String>;

/// Trait for feature definition
///
/// Defines a single feature with its display properties and segment overrides.
pub trait FeatureDefinition: Send + Sync {
    /// Feature ID (e.g., "low_rates", "women_benefits")
    fn id(&self) -> &str;

    /// Get display name for a language
    ///
    /// Returns the display name with variables substituted.
    fn display_name(&self, language: &str, variables: &VariableMap) -> String;

    /// Get description for a language
    fn description(&self, language: &str, variables: &VariableMap) -> Option<String>;

    /// Check if feature is enabled
    fn is_enabled(&self) -> bool;

    /// Get icon identifier
    fn icon(&self) -> Option<&str>;

    /// Get badge text
    fn badge(&self) -> Option<&str>;

    /// Get segment-specific override text
    fn segment_override(&self, segment_id: &str, language: &str, variables: &VariableMap)
        -> Option<String>;
}

/// Trait for providing features from config
///
/// This trait replaces the hardcoded Feature enum. All feature definitions,
/// display names, and segment mappings come from configuration.
pub trait FeatureProvider: Send + Sync {
    /// Get all feature IDs defined in config
    fn all_feature_ids(&self) -> Vec<String>;

    /// Get feature display name with variable substitution
    ///
    /// # Arguments
    /// * `feature_id` - The feature identifier (e.g., "women_benefits")
    /// * `language` - Language code (e.g., "en", "hi")
    /// * `variables` - Variables for substitution (e.g., {{special_program_name}})
    ///
    /// # Returns
    /// The display name with variables substituted, or None if feature not found
    fn feature_display_name(
        &self,
        feature_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Option<String>;

    /// Get feature description
    fn feature_description(
        &self,
        feature_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Option<String>;

    /// Get full feature display info
    fn feature_display(
        &self,
        feature_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Option<FeatureDisplay>;

    /// Get features for a customer segment
    ///
    /// Returns feature IDs in priority order for the given segment.
    fn features_for_segment(&self, segment_id: &str) -> Vec<String>;

    /// Get features with priorities for a segment
    fn features_with_priority(&self, segment_id: &str) -> Vec<FeaturePriority>;

    /// Get top N features for a segment
    fn top_features(&self, segment_id: &str, n: usize) -> Vec<String> {
        self.features_for_segment(segment_id)
            .into_iter()
            .take(n)
            .collect()
    }

    /// Get value propositions for a segment
    ///
    /// Returns localized value proposition texts with variables substituted.
    fn value_propositions(
        &self,
        segment_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Vec<String>;

    /// Check if a feature is enabled
    fn is_feature_enabled(&self, feature_id: &str) -> bool;

    /// Get segment-specific feature text
    ///
    /// Some features have different display text per segment.
    fn segment_feature_text(
        &self,
        feature_id: &str,
        segment_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Option<String>;
}

/// Config-driven feature definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFeatureDefinition {
    /// Feature ID
    pub id: String,
    /// Display names by language
    #[serde(default)]
    pub display_name: HashMap<String, String>,
    /// Descriptions by language
    #[serde(default)]
    pub description: HashMap<String, String>,
    /// Whether feature is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Icon identifier
    #[serde(default)]
    pub icon: Option<String>,
    /// Badge text
    #[serde(default)]
    pub badge: Option<String>,
    /// Segment-specific overrides
    #[serde(default)]
    pub segment_overrides: HashMap<String, SegmentFeatureOverride>,
}

fn default_true() -> bool {
    true
}

/// Segment-specific feature override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentFeatureOverride {
    /// Priority for this segment (lower = higher priority)
    #[serde(default = "default_priority")]
    pub priority: u8,
    /// Custom display text by language
    #[serde(default)]
    pub custom_text: HashMap<String, String>,
}

fn default_priority() -> u8 {
    50
}

impl FeatureDefinition for ConfigFeatureDefinition {
    fn id(&self) -> &str {
        &self.id
    }

    fn display_name(&self, language: &str, variables: &VariableMap) -> String {
        let template = self
            .display_name
            .get(language)
            .or_else(|| self.display_name.get("en"))
            .cloned()
            .unwrap_or_else(|| self.id.clone());

        substitute_variables(&template, variables)
    }

    fn description(&self, language: &str, variables: &VariableMap) -> Option<String> {
        self.description
            .get(language)
            .or_else(|| self.description.get("en"))
            .map(|s| substitute_variables(s, variables))
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    fn badge(&self) -> Option<&str> {
        self.badge.as_deref()
    }

    fn segment_override(
        &self,
        segment_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Option<String> {
        self.segment_overrides.get(segment_id).and_then(|o| {
            o.custom_text
                .get(language)
                .or_else(|| o.custom_text.get("en"))
                .map(|s| substitute_variables(s, variables))
        })
    }
}

/// Substitute variables in a template string
///
/// Variables are in the format {{variable_name}}.
pub fn substitute_variables(template: &str, variables: &VariableMap) -> String {
    let mut result = template.to_string();
    for (key, value) in variables {
        let pattern = format!("{{{{{}}}}}", key);
        result = result.replace(&pattern, value);
    }
    result
}

/// Config-driven feature provider
///
/// Aggregates multiple `ConfigFeatureDefinition` instances and implements
/// the `FeatureProvider` trait for domain-agnostic feature management.
#[derive(Debug, Clone)]
pub struct ConfigFeatureProvider {
    /// All feature definitions (feature_id -> definition)
    features: HashMap<String, ConfigFeatureDefinition>,
    /// Segment to feature mappings (segment_id -> feature_ids in priority order)
    segment_features: HashMap<String, Vec<String>>,
    /// Value propositions by segment and language
    value_propositions: HashMap<String, HashMap<String, Vec<String>>>,
    /// Default segment ID
    default_segment: String,
}

impl ConfigFeatureProvider {
    /// Create from config structures
    pub fn new(
        features: Vec<ConfigFeatureDefinition>,
        segment_features: HashMap<String, Vec<String>>,
        value_propositions: HashMap<String, HashMap<String, Vec<String>>>,
    ) -> Self {
        let features_map = features
            .into_iter()
            .map(|f| (f.id.clone(), f))
            .collect();

        Self {
            features: features_map,
            segment_features,
            value_propositions,
            default_segment: "default".to_string(),
        }
    }

    /// Create with default segment
    pub fn with_default_segment(mut self, segment: &str) -> Self {
        self.default_segment = segment.to_string();
        self
    }

    /// Get a feature definition by ID
    pub fn get_feature(&self, feature_id: &str) -> Option<&ConfigFeatureDefinition> {
        self.features.get(feature_id)
    }
}

impl FeatureProvider for ConfigFeatureProvider {
    fn all_feature_ids(&self) -> Vec<String> {
        self.features.keys().cloned().collect()
    }

    fn feature_display_name(
        &self,
        feature_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Option<String> {
        self.features
            .get(feature_id)
            .filter(|f| f.enabled)
            .map(|f| f.display_name(language, variables))
    }

    fn feature_description(
        &self,
        feature_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Option<String> {
        self.features
            .get(feature_id)
            .filter(|f| f.enabled)
            .and_then(|f| f.description(language, variables))
    }

    fn feature_display(
        &self,
        feature_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Option<FeatureDisplay> {
        self.features.get(feature_id).filter(|f| f.enabled).map(|f| {
            FeatureDisplay {
                id: f.id.clone(),
                display_name: f.display_name(language, variables),
                localized_name: if language != "en" {
                    Some(f.display_name(language, variables))
                } else {
                    None
                },
                description: f.description(language, variables),
                icon: f.icon.clone(),
                badge: f.badge.clone(),
            }
        })
    }

    fn features_for_segment(&self, segment_id: &str) -> Vec<String> {
        self.segment_features
            .get(segment_id)
            .or_else(|| self.segment_features.get(&self.default_segment))
            .cloned()
            .unwrap_or_default()
    }

    fn features_with_priority(&self, segment_id: &str) -> Vec<FeaturePriority> {
        self.features_for_segment(segment_id)
            .into_iter()
            .enumerate()
            .map(|(idx, feature_id)| {
                let segment_text = self
                    .features
                    .get(&feature_id)
                    .and_then(|f| {
                        f.segment_overrides
                            .get(segment_id)
                            .and_then(|o| o.custom_text.get("en").cloned())
                    });

                FeaturePriority {
                    feature_id,
                    priority: (idx + 1) as u8,
                    segment_text,
                }
            })
            .collect()
    }

    fn value_propositions(
        &self,
        segment_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Vec<String> {
        self.value_propositions
            .get(segment_id)
            .or_else(|| self.value_propositions.get(&self.default_segment))
            .and_then(|by_lang| by_lang.get(language).or_else(|| by_lang.get("en")))
            .map(|props| {
                props
                    .iter()
                    .map(|p| substitute_variables(p, variables))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn is_feature_enabled(&self, feature_id: &str) -> bool {
        self.features
            .get(feature_id)
            .map(|f| f.enabled)
            .unwrap_or(false)
    }

    fn segment_feature_text(
        &self,
        feature_id: &str,
        segment_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Option<String> {
        self.features.get(feature_id).and_then(|f| {
            f.segment_override(segment_id, language, variables)
                .or_else(|| Some(f.display_name(language, variables)))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_substitution() {
        let template = "{{program_name}} with {{benefit}}";
        let mut vars = HashMap::new();
        vars.insert("program_name".to_string(), "Shakti Gold".to_string());
        vars.insert("benefit".to_string(), "0.25% lower rates".to_string());

        let result = substitute_variables(template, &vars);
        assert_eq!(result, "Shakti Gold with 0.25% lower rates");
    }

    #[test]
    fn test_config_feature_definition() {
        let mut display_name = HashMap::new();
        display_name.insert("en".to_string(), "{{program_name}} Benefits".to_string());
        display_name.insert("hi".to_string(), "{{program_name}}".to_string());

        let feature = ConfigFeatureDefinition {
            id: "women_benefits".to_string(),
            display_name,
            description: HashMap::new(),
            enabled: true,
            icon: Some("female".to_string()),
            badge: Some("exclusive".to_string()),
            segment_overrides: HashMap::new(),
        };

        let mut vars = HashMap::new();
        vars.insert("program_name".to_string(), "Shakti Gold".to_string());

        assert_eq!(
            feature.display_name("en", &vars),
            "Shakti Gold Benefits"
        );
        assert_eq!(feature.display_name("hi", &vars), "Shakti Gold");
        assert!(feature.is_enabled());
        assert_eq!(feature.icon(), Some("female"));
    }
}
