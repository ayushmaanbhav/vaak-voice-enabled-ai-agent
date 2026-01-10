//! Response adaptation based on customer segment
//!
//! Adapts responses to match customer expectations:
//! - Feature emphasis based on segment priorities
//! - Objection handling strategies
//! - Value proposition customization
//! - Competitive positioning
//!
//! # Config-Driven Design
//!
//! All content is loaded from YAML configuration files:
//! - `features.yaml` - Feature definitions and segment mappings
//! - `objections.yaml` - Objection patterns and responses
//! - `segments.yaml` - Customer segment definitions
//!
//! Use string-based IDs (SegmentId, FeatureId, ObjectionId) loaded from config via
//! `SegmentDetector`, `FeatureProvider`, and `ObjectionProvider` traits.
//!
//! ## Migration from CustomerSegment enum
//!
//! ```ignore
//! // OLD: Enum-based
//! let features = adapter.get_features_for_segment(CustomerSegment::HighValue);
//!
//! // NEW: Config-driven with SegmentId
//! let features = adapter.get_feature_ids("high_value");
//! ```

use crate::CustomerSegment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Config-Driven Types
// ============================================================================

/// Segment identifier (config-driven)
///
/// Use string IDs from config instead of CustomerSegment enum.
/// Segment definitions and detection come from segments.yaml.
pub type SegmentId = String;

/// Feature identifier (config-driven)
///
/// Use string IDs from config instead of enum variants.
/// Feature display names and properties come from features.yaml.
pub type FeatureId = String;

/// Objection identifier (config-driven)
///
/// Use string IDs from config instead of enum variants.
/// Objection detection and responses come from objections.yaml.
pub type ObjectionId = String;

/// Well-known feature IDs (for reference, actual values in config)
///
/// These constants provide compile-time references to common feature IDs.
/// The actual display names and properties are loaded from config.
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
    pub const REGULATOR_CERTIFIED: &str = "rbi_regulated";
    pub const ZERO_FORECLOSURE: &str = "zero_foreclosure";
    pub const DOORSTEP_SERVICE: &str = "doorstep_service";
    pub const SPECIAL_PROGRAM_BENEFITS: &str = "women_benefits";
}

/// Well-known objection IDs (for reference, actual detection in config)
///
/// These constants provide compile-time references to common objection IDs.
/// The actual detection patterns and responses are loaded from config.
pub mod objection_ids {
    pub const COLLATERAL_SAFETY: &str = "collateral_safety";
    pub const BETTER_RATES_ELSEWHERE: &str = "better_rates_elsewhere";
    pub const TOO_MUCH_PAPERWORK: &str = "too_much_paperwork";
    pub const DONT_WANT_TO_SWITCH: &str = "dont_want_to_switch";
    pub const NEEDS_TIME: &str = "needs_time";
    pub const TRUST_ISSUES: &str = "trust_issues";
    pub const EXPECTS_HIDDEN_CHARGES: &str = "expects_hidden_charges";
    pub const TOO_SLOW: &str = "too_slow";
    pub const NO_NEARBY_BRANCH: &str = "no_nearby_branch";
    pub const EXISTING_COMMITMENTS: &str = "existing_loans";
}

// ============================================================================
// Response Types
// ============================================================================

/// Objection handling response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectionResponse {
    /// Acknowledgment phrase
    pub acknowledgment: String,
    /// Main response
    pub response: String,
    /// Follow-up question (optional)
    pub follow_up: Option<String>,
    /// Feature to highlight (feature ID)
    pub highlight_feature: FeatureId,
}

// ============================================================================
// Configuration Types
// ============================================================================

/// Configuration data for SegmentAdapter
///
/// Use this struct to load segment configuration from YAML files.
///
/// Example YAML:
/// ```yaml
/// segment_features:
///   high_value: ["relationship_manager", "higher_limits"]
///   trust_seeker: ["security", "transparency"]
/// value_propositions:
///   high_value:
///     - "Exclusive rates and priority processing"
///     - "Dedicated relationship manager"
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SegmentAdapterConfig {
    /// Feature IDs per segment (segment_id -> [feature_id, ...])
    #[serde(default)]
    pub segment_features: HashMap<String, Vec<String>>,
    /// Value propositions per segment (segment_id -> [proposition, ...])
    #[serde(default)]
    pub value_propositions: HashMap<String, Vec<String>>,
    /// Objection responses (objection_id -> response data)
    #[serde(default)]
    pub objection_responses: HashMap<String, ObjectionResponseConfig>,
}

/// Objection response configuration (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectionResponseConfig {
    /// Segment this response applies to
    pub segment: String,
    /// Acknowledgment phrase
    pub acknowledgment: String,
    /// Main response
    pub response: String,
    /// Follow-up question (optional)
    pub follow_up: Option<String>,
    /// Feature to highlight (feature_id)
    pub highlight_feature: String,
}

// ============================================================================
// Segment Adapter
// ============================================================================

/// Segment adapter for response customization
///
/// Create via `from_config()` with domain-specific content loaded from YAML files.
///
/// ## Usage
///
/// ```ignore
/// let config = SegmentAdapterConfig::from_domain_config(&domain_config);
/// let adapter = SegmentAdapter::from_config(config);
/// ```
pub struct SegmentAdapter {
    /// Feature IDs per segment (segment_id -> [feature_id, ...])
    segment_feature_ids: HashMap<String, Vec<String>>,
    /// Value propositions per segment (segment_id -> [proposition, ...])
    value_propositions: HashMap<String, Vec<String>>,
    /// Objection responses ((segment_id, objection_id) -> response)
    objection_responses: HashMap<(String, String), ObjectionResponse>,
}

impl SegmentAdapter {
    /// Create an empty segment adapter
    ///
    /// Returns an adapter with no features, propositions, or responses loaded.
    /// Useful for unit tests or when config loading is deferred.
    pub fn empty() -> Self {
        Self {
            segment_feature_ids: HashMap::new(),
            value_propositions: HashMap::new(),
            objection_responses: HashMap::new(),
        }
    }

    /// Create a segment adapter from configuration
    ///
    /// Loads all domain-specific content (features, value propositions, objection responses)
    /// from the provided configuration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = SegmentAdapterConfig {
    ///     segment_features: features_config.segment_features.clone(),
    ///     value_propositions: features_config.value_propositions.clone(),
    ///     objection_responses: objections_data,
    /// };
    /// let adapter = SegmentAdapter::from_config(config);
    /// ```
    pub fn from_config(config: SegmentAdapterConfig) -> Self {
        let mut adapter = Self::empty();

        // Store segment features
        adapter.segment_feature_ids = config.segment_features;

        // Store value propositions
        adapter.value_propositions = config.value_propositions;

        // Load objection responses
        for (objection_id, response_config) in config.objection_responses {
            adapter.objection_responses.insert(
                (response_config.segment.clone(), objection_id),
                ObjectionResponse {
                    acknowledgment: response_config.acknowledgment,
                    response: response_config.response,
                    follow_up: response_config.follow_up,
                    highlight_feature: response_config.highlight_feature,
                },
            );
        }

        adapter
    }

    /// Validate that config was properly loaded
    ///
    /// Returns Ok(()) if adapter has content, Err with details if empty.
    pub fn validate(&self) -> Result<(), String> {
        let mut issues = Vec::new();

        if self.segment_feature_ids.is_empty() {
            issues.push("No segment features loaded");
        }

        if self.value_propositions.is_empty() {
            issues.push("No value propositions loaded");
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "SegmentAdapter config validation failed: {}. \
                 Ensure features.yaml and adaptation.yaml are properly configured.",
                issues.join(", ")
            ))
        }
    }

    // ========================================================================
    // Feature Access
    // ========================================================================

    /// Get feature IDs for a segment
    ///
    /// Returns feature IDs as strings for use with FeatureProvider trait.
    pub fn get_feature_ids(&self, segment_id: &str) -> Vec<String> {
        self.segment_feature_ids
            .get(segment_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get feature IDs for a CustomerSegment enum
    pub fn get_features_for_segment(&self, segment: CustomerSegment) -> Vec<String> {
        self.get_feature_ids(&segment_to_id(segment))
    }

    /// Get top N feature IDs for a segment
    pub fn get_top_feature_ids(&self, segment_id: &str, n: usize) -> Vec<String> {
        self.get_feature_ids(segment_id).into_iter().take(n).collect()
    }

    // ========================================================================
    // Value Propositions
    // ========================================================================

    /// Get value propositions for a segment by ID
    ///
    /// Returns raw propositions which may contain {{variable}} placeholders.
    pub fn get_value_propositions(&self, segment_id: &str) -> Vec<String> {
        self.value_propositions
            .get(segment_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get value propositions for a CustomerSegment enum
    pub fn get_value_propositions_for_segment(&self, segment: CustomerSegment) -> Vec<String> {
        self.get_value_propositions(&segment_to_id(segment))
    }

    // ========================================================================
    // Objection Handling
    // ========================================================================

    /// Get objection response
    pub fn get_objection_response(
        &self,
        segment_id: &str,
        objection_id: &str,
    ) -> Option<&ObjectionResponse> {
        self.objection_responses
            .get(&(segment_id.to_string(), objection_id.to_string()))
    }

    /// Get objection response for CustomerSegment enum
    pub fn get_objection_response_for_segment(
        &self,
        segment: CustomerSegment,
        objection_id: &str,
    ) -> Option<&ObjectionResponse> {
        self.get_objection_response(&segment_to_id(segment), objection_id)
    }

    /// Handle objection and return formatted response
    pub fn handle_objection(
        &self,
        segment_id: &str,
        objection_id: &str,
        customer_name: Option<&str>,
    ) -> Option<String> {
        let response = self.get_objection_response(segment_id, objection_id)?;

        let mut result = response.acknowledgment.clone();
        result.push(' ');
        result.push_str(&response.response);

        if let Some(follow_up) = &response.follow_up {
            result.push(' ');
            if let Some(name) = customer_name {
                result.push_str(&format!("{}, {}", name, follow_up));
            } else {
                result.push_str(follow_up);
            }
        }

        Some(result)
    }

    // ========================================================================
    // Metadata
    // ========================================================================

    /// Get all segment IDs that have features configured
    pub fn configured_segment_ids(&self) -> Vec<String> {
        self.segment_feature_ids.keys().cloned().collect()
    }

    /// Check if a segment has any features configured
    pub fn has_features(&self, segment_id: &str) -> bool {
        self.segment_feature_ids
            .get(segment_id)
            .map(|f| !f.is_empty())
            .unwrap_or(false)
    }

    // ========================================================================
    // Modification
    // ========================================================================

    /// Add feature ID for a segment
    pub fn add_feature(&mut self, segment_id: &str, feature_id: String) {
        self.segment_feature_ids
            .entry(segment_id.to_string())
            .or_default()
            .push(feature_id);
    }

    /// Add value proposition for a segment
    pub fn add_value_proposition(&mut self, segment_id: &str, proposition: String) {
        self.value_propositions
            .entry(segment_id.to_string())
            .or_default()
            .push(proposition);
    }

    /// Add objection response
    pub fn add_objection_response(
        &mut self,
        segment_id: &str,
        objection_id: &str,
        response: ObjectionResponse,
    ) {
        self.objection_responses
            .insert((segment_id.to_string(), objection_id.to_string()), response);
    }
}

impl Default for SegmentAdapter {
    fn default() -> Self {
        Self::empty()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert CustomerSegment enum to string ID
///
/// # Deprecated
///
/// Use `CustomerSegment::to_segment_id()` directly or string-based `SegmentId`.
/// This function is retained for backward compatibility.
///
/// ## Migration
/// ```ignore
/// // OLD
/// let id = segment_to_id(segment);
///
/// // NEW
/// let id = segment.to_segment_id();
/// // Or use SegmentId directly: "high_value"
/// ```
pub fn segment_to_id(segment: CustomerSegment) -> SegmentId {
    segment.to_segment_id()
}

/// Parse segment ID string to CustomerSegment enum
///
/// # Deprecated
///
/// Use `CustomerSegment::from_segment_id()` directly.
/// For config-defined segments not in the enum, the result will be None.
///
/// ## Migration
/// ```ignore
/// // OLD
/// let segment = parse_segment_id(id);
///
/// // NEW
/// let segment = CustomerSegment::from_segment_id(id);
/// // Or use SegmentId directly without converting to enum
/// ```
pub fn parse_segment_id(id: &str) -> Option<CustomerSegment> {
    CustomerSegment::from_segment_id(id)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_to_id() {
        assert_eq!(segment_to_id(CustomerSegment::HighValue), "high_value");
        assert_eq!(segment_to_id(CustomerSegment::TrustSeeker), "trust_seeker");
        assert_eq!(segment_to_id(CustomerSegment::Women), "women");
    }

    #[test]
    fn test_parse_segment_id() {
        assert_eq!(parse_segment_id("high_value"), Some(CustomerSegment::HighValue));
        assert_eq!(parse_segment_id("trust_seeker"), Some(CustomerSegment::TrustSeeker));
        assert_eq!(parse_segment_id("unknown"), None);
    }

    #[test]
    fn test_empty_adapter() {
        let adapter = SegmentAdapter::empty();
        assert!(adapter.get_feature_ids("high_value").is_empty());
        assert!(adapter.get_value_propositions("high_value").is_empty());
        assert!(adapter.validate().is_err());
    }

    #[test]
    fn test_from_config() {
        let mut config = SegmentAdapterConfig::default();
        config.segment_features.insert(
            "high_value".to_string(),
            vec!["relationship_manager".to_string(), "higher_limits".to_string()],
        );
        config.value_propositions.insert(
            "high_value".to_string(),
            vec!["Exclusive rates".to_string()],
        );

        let adapter = SegmentAdapter::from_config(config);

        let features = adapter.get_feature_ids("high_value");
        assert_eq!(features.len(), 2);
        assert!(features.contains(&"relationship_manager".to_string()));

        let propositions = adapter.get_value_propositions("high_value");
        assert_eq!(propositions.len(), 1);
        assert_eq!(propositions[0], "Exclusive rates");
    }

    #[test]
    fn test_objection_response() {
        let mut config = SegmentAdapterConfig::default();
        config.segment_features.insert("trust_seeker".to_string(), vec!["security".to_string()]);
        config.value_propositions.insert("trust_seeker".to_string(), vec!["Safe service".to_string()]);
        config.objection_responses.insert(
            "collateral_safety".to_string(),
            ObjectionResponseConfig {
                segment: "trust_seeker".to_string(),
                acknowledgment: "I understand your concern.".to_string(),
                response: "Your assets are fully insured.".to_string(),
                follow_up: Some("Would you like details?".to_string()),
                highlight_feature: "security".to_string(),
            },
        );

        let adapter = SegmentAdapter::from_config(config);

        let response = adapter.get_objection_response("trust_seeker", "collateral_safety");
        assert!(response.is_some());
        let response = response.unwrap();
        assert_eq!(response.acknowledgment, "I understand your concern.");
        assert_eq!(response.highlight_feature, "security");
    }

    #[test]
    fn test_handle_objection() {
        let mut config = SegmentAdapterConfig::default();
        config.segment_features.insert("trust_seeker".to_string(), vec![]);
        config.value_propositions.insert("trust_seeker".to_string(), vec![]);
        config.objection_responses.insert(
            "safety".to_string(),
            ObjectionResponseConfig {
                segment: "trust_seeker".to_string(),
                acknowledgment: "I understand.".to_string(),
                response: "It's safe.".to_string(),
                follow_up: Some("Questions?".to_string()),
                highlight_feature: "security".to_string(),
            },
        );

        let adapter = SegmentAdapter::from_config(config);

        let result = adapter.handle_objection("trust_seeker", "safety", Some("Priya"));
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.contains("I understand."));
        assert!(result.contains("It's safe."));
        assert!(result.contains("Priya"));
    }

    #[test]
    fn test_get_features_for_segment() {
        let mut config = SegmentAdapterConfig::default();
        config.segment_features.insert(
            "high_value".to_string(),
            vec!["feature1".to_string(), "feature2".to_string()],
        );
        config.value_propositions.insert("high_value".to_string(), vec![]);

        let adapter = SegmentAdapter::from_config(config);

        let features = adapter.get_features_for_segment(CustomerSegment::HighValue);
        assert_eq!(features.len(), 2);
    }

    #[test]
    fn test_validate() {
        // Empty adapter fails validation
        let adapter = SegmentAdapter::empty();
        assert!(adapter.validate().is_err());

        // Adapter with content passes validation
        let mut config = SegmentAdapterConfig::default();
        config.segment_features.insert("test".to_string(), vec!["f1".to_string()]);
        config.value_propositions.insert("test".to_string(), vec!["p1".to_string()]);

        let adapter = SegmentAdapter::from_config(config);
        assert!(adapter.validate().is_ok());
    }

    #[test]
    fn test_feature_ids_constants() {
        // Verify feature ID constants are correct
        assert_eq!(feature_ids::LOW_RATES, "low_rates");
        assert_eq!(feature_ids::SECURITY, "security");
        assert_eq!(feature_ids::QUICK_PROCESS, "quick_process");
    }

    #[test]
    fn test_objection_ids_constants() {
        // Verify objection ID constants are correct
        assert_eq!(objection_ids::COLLATERAL_SAFETY, "collateral_safety");
        assert_eq!(objection_ids::BETTER_RATES_ELSEWHERE, "better_rates_elsewhere");
        assert_eq!(objection_ids::NEEDS_TIME, "needs_time");
    }

    #[test]
    fn test_add_feature() {
        let mut adapter = SegmentAdapter::empty();
        adapter.add_feature("high_value", "new_feature".to_string());

        let features = adapter.get_feature_ids("high_value");
        assert_eq!(features.len(), 1);
        assert_eq!(features[0], "new_feature");
    }

    #[test]
    fn test_add_value_proposition() {
        let mut adapter = SegmentAdapter::empty();
        adapter.add_value_proposition("high_value", "Great service".to_string());

        let props = adapter.get_value_propositions("high_value");
        assert_eq!(props.len(), 1);
        assert_eq!(props[0], "Great service");
    }
}
