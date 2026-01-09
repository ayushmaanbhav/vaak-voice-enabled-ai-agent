//! Segment Detector trait for customer segmentation
//!
//! This module provides a domain-agnostic interface for detecting customer segments
//! based on text patterns, numeric thresholds, and slot values. All segment definitions
//! are loaded from configuration.
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_core::traits::SegmentDetector;
//!
//! // Detector is created from domain config
//! let detector = config_bridge.segment_detector();
//!
//! // Detect segments from user context
//! let segments = detector.detect("I urgently need money for hospital", "en", &values, &slots);
//! let primary = detector.primary_segment(&segments);
//! ```

use std::collections::HashMap;

/// Segment detection match
#[derive(Debug, Clone)]
pub struct SegmentMatch {
    /// Segment ID
    pub segment_id: String,
    /// Match confidence (0.0 - 1.0)
    pub confidence: f32,
    /// Matched patterns or thresholds
    pub match_reasons: Vec<String>,
}

/// Value proposition for a segment
#[derive(Debug, Clone)]
pub struct ValueProposition {
    /// Short headline
    pub headline: String,
    /// Detailed description
    pub description: String,
    /// Language code
    pub language: String,
}

/// Feature emphasis for a segment
#[derive(Debug, Clone)]
pub struct FeatureEmphasis {
    /// Feature ID (e.g., "low_rates", "quick_process")
    pub feature_id: String,
    /// Display name
    pub display_name: String,
    /// Localized display name
    pub localized_name: String,
    /// Importance weight (higher = more important)
    pub weight: u8,
}

/// Segment definition trait
///
/// Defines a customer segment with detection rules, value propositions,
/// and feature emphasis. All loaded from config.
pub trait SegmentDefinition: Send + Sync {
    /// Segment ID (e.g., "high_value", "urgent_need")
    fn id(&self) -> &str;

    /// Human-readable display name
    fn display_name(&self) -> &str;

    /// Segment description
    fn description(&self) -> &str;

    /// Priority level (lower = higher priority)
    fn priority(&self) -> u8;

    /// Check if segment matches based on signals
    ///
    /// Returns true if this segment matches the given context.
    fn matches(
        &self,
        text: &str,
        language: &str,
        numeric_values: &HashMap<String, f64>,
        text_values: &HashMap<String, String>,
    ) -> bool;

    /// Get match confidence (0.0 - 1.0)
    fn match_confidence(
        &self,
        text: &str,
        language: &str,
        numeric_values: &HashMap<String, f64>,
        text_values: &HashMap<String, String>,
    ) -> f32;

    /// Get text patterns for detection
    fn text_patterns(&self, language: &str) -> Vec<&str>;

    /// Get numeric thresholds for detection
    fn numeric_thresholds(&self) -> &HashMap<String, f64>;

    /// Get slot value patterns for detection
    fn slot_patterns(&self) -> &HashMap<String, Vec<String>>;

    /// Get features to emphasize for this segment
    fn features(&self) -> Vec<&str>;

    /// Get value propositions for language
    fn value_propositions(&self, language: &str) -> Vec<&ValueProposition>;

    /// Get feature emphasis details
    fn feature_emphasis(&self) -> Vec<&FeatureEmphasis>;
}

/// Segment detector trait
///
/// Detects customer segments from conversation context.
pub trait SegmentDetector: Send + Sync {
    /// Detect all matching segments
    ///
    /// Returns segment IDs sorted by priority (highest priority first).
    fn detect(
        &self,
        text: &str,
        language: &str,
        numeric_values: &HashMap<String, f64>,
        text_values: &HashMap<String, String>,
    ) -> Vec<SegmentMatch>;

    /// Get primary segment (highest priority match)
    fn primary_segment(
        &self,
        text: &str,
        language: &str,
        numeric_values: &HashMap<String, f64>,
        text_values: &HashMap<String, String>,
    ) -> &str;

    /// Get segment definition by ID
    fn get_segment(&self, id: &str) -> Option<&dyn SegmentDefinition>;

    /// Get all segment definitions
    fn all_segments(&self) -> Vec<&dyn SegmentDefinition>;

    /// Get default segment ID
    fn default_segment(&self) -> &str;

    /// Get all segment IDs
    fn segment_ids(&self) -> Vec<&str>;

    /// Get features to emphasize for detected segments
    fn features_for_segments(&self, segment_ids: &[&str]) -> Vec<String>;

    /// Get value propositions for segment in language
    fn value_propositions(&self, segment_id: &str, language: &str) -> Vec<String>;
}

/// Config-driven segment definition
#[derive(Debug, Clone)]
pub struct ConfigSegmentDefinition {
    id: String,
    display_name: String,
    description: String,
    priority: u8,
    text_patterns: HashMap<String, Vec<String>>,
    numeric_thresholds: HashMap<String, f64>,
    slot_patterns: HashMap<String, Vec<String>>,
    features: Vec<String>,
    value_propositions: HashMap<String, Vec<ValueProposition>>,
    feature_emphasis: Vec<FeatureEmphasis>,
}

impl ConfigSegmentDefinition {
    /// Create a new segment definition
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        description: impl Into<String>,
        priority: u8,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            description: description.into(),
            priority,
            text_patterns: HashMap::new(),
            numeric_thresholds: HashMap::new(),
            slot_patterns: HashMap::new(),
            features: Vec::new(),
            value_propositions: HashMap::new(),
            feature_emphasis: Vec::new(),
        }
    }

    /// Add text patterns for a language
    pub fn with_text_patterns(mut self, language: &str, patterns: Vec<String>) -> Self {
        self.text_patterns.insert(language.to_string(), patterns);
        self
    }

    /// Add numeric thresholds
    pub fn with_numeric_thresholds(mut self, thresholds: HashMap<String, f64>) -> Self {
        self.numeric_thresholds = thresholds;
        self
    }

    /// Add slot patterns
    pub fn with_slot_patterns(mut self, patterns: HashMap<String, Vec<String>>) -> Self {
        self.slot_patterns = patterns;
        self
    }

    /// Add features to emphasize
    pub fn with_features(mut self, features: Vec<String>) -> Self {
        self.features = features;
        self
    }

    // === PRESERVED GOLD LOAN SEGMENTS ===

    /// Create high-value segment
    /// PRESERVED: gold_weight >= 100g OR loan_amount >= 500000
    pub fn high_value() -> Self {
        Self::new("high_value", "High Value", "High-value customer with significant assets", 1)
            .with_numeric_thresholds([
                ("asset_quantity".to_string(), 100.0),
                ("loan_amount".to_string(), 500_000.0),
            ].into_iter().collect())
            .with_text_patterns("en", vec![
                "lakh".to_string(),
                "crore".to_string(),
                "100 gram".to_string(),
                "large amount".to_string(),
                "significant".to_string(),
                "premium".to_string(),
            ])
            .with_text_patterns("hi", vec![
                "लाख".to_string(),
                "करोड़".to_string(),
                "सौ ग्राम".to_string(),
            ])
            .with_features(vec![
                "relationship_manager".to_string(),
                "priority_processing".to_string(),
                "premium_rates".to_string(),
                "dedicated_support".to_string(),
            ])
    }

    /// Create trust-seeker segment
    /// PRESERVED: NBFC lender detection + safety keywords
    pub fn trust_seeker() -> Self {
        Self::new("trust_seeker", "Trust Seeker", "Customer prioritizing safety and security", 2)
            .with_slot_patterns([
                ("current_lender".to_string(), vec![
                    "muthoot".to_string(),
                    "manappuram".to_string(),
                    "iifl".to_string(),
                    "local".to_string(),
                    "jeweler".to_string(),
                ]),
            ].into_iter().collect())
            .with_text_patterns("en", vec![
                "safe".to_string(),
                "security".to_string(),
                "trust".to_string(),
                "rbi".to_string(),
                "government".to_string(),
                "bank vault".to_string(),
                "worried".to_string(),
                "scared".to_string(),
            ])
            .with_text_patterns("hi", vec![
                "सुरक्षा".to_string(),
                "भरोसा".to_string(),
                "विश्वास".to_string(),
                "सरकारी".to_string(),
            ])
            .with_features(vec![
                "rbi_regulated".to_string(),
                "bank_security".to_string(),
                "vault_storage".to_string(),
                "insurance".to_string(),
            ])
    }

    /// Create price-sensitive segment
    /// PRESERVED: rate/interest/save keywords
    pub fn price_sensitive() -> Self {
        Self::new("price_sensitive", "Price Sensitive", "Customer focused on rates and savings", 3)
            .with_text_patterns("en", vec![
                "rate".to_string(),
                "interest".to_string(),
                "cheap".to_string(),
                "expensive".to_string(),
                "save".to_string(),
                "savings".to_string(),
                "compare".to_string(),
                "emi".to_string(),
                "monthly payment".to_string(),
            ])
            .with_text_patterns("hi", vec![
                "ब्याज".to_string(),
                "दर".to_string(),
                "सस्ता".to_string(),
                "महंगा".to_string(),
                "बचत".to_string(),
                "ईएमआई".to_string(),
                "किस्त".to_string(),
            ])
            .with_features(vec![
                "competitive_rates".to_string(),
                "savings_calculator".to_string(),
                "flexible_repayment".to_string(),
                "no_hidden_charges".to_string(),
            ])
    }

    /// Create urgent-need segment
    /// PRESERVED: urgent/emergency/today keywords
    pub fn urgent_need() -> Self {
        Self::new("urgent_need", "Urgent Need", "Customer with immediate financial need", 1)
            .with_text_patterns("en", vec![
                "urgent".to_string(),
                "emergency".to_string(),
                "immediately".to_string(),
                "today".to_string(),
                "right now".to_string(),
                "asap".to_string(),
                "quick".to_string(),
                "fast".to_string(),
                "hospital".to_string(),
                "medical".to_string(),
            ])
            .with_text_patterns("hi", vec![
                "जल्दी".to_string(),
                "तुरंत".to_string(),
                "आज".to_string(),
                "अभी".to_string(),
                "इमरजेंसी".to_string(),
                "अस्पताल".to_string(),
            ])
            .with_features(vec![
                "same_day_disbursement".to_string(),
                "minimal_documentation".to_string(),
                "quick_valuation".to_string(),
            ])
    }

    /// Create balance-transfer segment
    /// PRESERVED: existing loan detection
    pub fn balance_transfer() -> Self {
        Self::new("balance_transfer", "Balance Transfer", "Customer with existing loan elsewhere", 2)
            .with_text_patterns("en", vec![
                "already have".to_string(),
                "existing loan".to_string(),
                "current loan".to_string(),
                "transfer".to_string(),
                "switch".to_string(),
                "move".to_string(),
                "paying".to_string(),
                "emi".to_string(),
            ])
            .with_text_patterns("hi", vec![
                "पहले से".to_string(),
                "मौजूदा लोन".to_string(),
                "ट्रांसफर".to_string(),
                "बदलना".to_string(),
                "चल रहा".to_string(),
            ])
            .with_features(vec![
                "easy_transfer".to_string(),
                "rate_comparison".to_string(),
                "savings_calculation".to_string(),
                "quick_closure".to_string(),
            ])
    }

    /// Create first-time borrower segment
    /// PRESERVED: first time/new keywords
    pub fn first_time() -> Self {
        Self::new("first_time", "First Time Borrower", "Customer new to gold loans", 4)
            .with_text_patterns("en", vec![
                "first time".to_string(),
                "never".to_string(),
                "new to".to_string(),
                "how does".to_string(),
                "what is".to_string(),
                "explain".to_string(),
                "don't understand".to_string(),
            ])
            .with_text_patterns("hi", vec![
                "पहली बार".to_string(),
                "कभी नहीं".to_string(),
                "नया हूं".to_string(),
                "कैसे".to_string(),
                "क्या है".to_string(),
                "समझाइए".to_string(),
            ])
            .with_features(vec![
                "step_by_step_guidance".to_string(),
                "simple_process".to_string(),
                "support".to_string(),
            ])
    }

    /// Create business-owner segment
    /// PRESERVED: business/shop/capital keywords
    pub fn business_owner() -> Self {
        Self::new("business_owner", "Business Owner", "Customer needing business capital", 3)
            .with_text_patterns("en", vec![
                "business".to_string(),
                "shop".to_string(),
                "store".to_string(),
                "inventory".to_string(),
                "working capital".to_string(),
                "supplier".to_string(),
                "stock".to_string(),
                "company".to_string(),
            ])
            .with_text_patterns("hi", vec![
                "व्यापार".to_string(),
                "दुकान".to_string(),
                "कारोबार".to_string(),
                "माल".to_string(),
                "पूंजी".to_string(),
            ])
            .with_features(vec![
                "business_friendly".to_string(),
                "flexible_repayment".to_string(),
                "higher_limits".to_string(),
            ])
    }
}

impl SegmentDefinition for ConfigSegmentDefinition {
    fn id(&self) -> &str {
        &self.id
    }

    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn priority(&self) -> u8 {
        self.priority
    }

    fn matches(
        &self,
        text: &str,
        language: &str,
        numeric_values: &HashMap<String, f64>,
        text_values: &HashMap<String, String>,
    ) -> bool {
        let lower_text = text.to_lowercase();

        // Check numeric thresholds (any match triggers)
        for (key, threshold) in &self.numeric_thresholds {
            if let Some(value) = numeric_values.get(key) {
                if *value >= *threshold {
                    return true;
                }
            }
        }

        // Check text patterns
        if let Some(patterns) = self.text_patterns.get(language) {
            for pattern in patterns {
                if lower_text.contains(&pattern.to_lowercase()) {
                    return true;
                }
            }
        }

        // Check slot patterns
        for (slot_name, patterns) in &self.slot_patterns {
            if let Some(slot_value) = text_values.get(slot_name) {
                let lower_value = slot_value.to_lowercase();
                for pattern in patterns {
                    if lower_value.contains(&pattern.to_lowercase()) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn match_confidence(
        &self,
        text: &str,
        language: &str,
        numeric_values: &HashMap<String, f64>,
        text_values: &HashMap<String, String>,
    ) -> f32 {
        let mut score = 0.0f32;
        let mut max_score = 0.0f32;
        let lower_text = text.to_lowercase();

        // Numeric threshold matches
        for (key, threshold) in &self.numeric_thresholds {
            max_score += 1.0;
            if let Some(value) = numeric_values.get(key) {
                if *value >= *threshold {
                    score += 1.0;
                }
            }
        }

        // Text pattern matches
        if let Some(patterns) = self.text_patterns.get(language) {
            for pattern in patterns {
                max_score += 0.5;
                if lower_text.contains(&pattern.to_lowercase()) {
                    score += 0.5;
                }
            }
        }

        // Slot pattern matches
        for (slot_name, patterns) in &self.slot_patterns {
            if let Some(slot_value) = text_values.get(slot_name) {
                let lower_value = slot_value.to_lowercase();
                for pattern in patterns {
                    max_score += 1.0;
                    if lower_value.contains(&pattern.to_lowercase()) {
                        score += 1.0;
                    }
                }
            }
        }

        if max_score > 0.0 {
            (score / max_score).min(1.0)
        } else {
            0.0
        }
    }

    fn text_patterns(&self, language: &str) -> Vec<&str> {
        self.text_patterns
            .get(language)
            .map(|p| p.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    fn numeric_thresholds(&self) -> &HashMap<String, f64> {
        &self.numeric_thresholds
    }

    fn slot_patterns(&self) -> &HashMap<String, Vec<String>> {
        &self.slot_patterns
    }

    fn features(&self) -> Vec<&str> {
        self.features.iter().map(|s| s.as_str()).collect()
    }

    fn value_propositions(&self, language: &str) -> Vec<&ValueProposition> {
        self.value_propositions
            .get(language)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    fn feature_emphasis(&self) -> Vec<&FeatureEmphasis> {
        self.feature_emphasis.iter().collect()
    }
}

/// Config-driven segment detector
pub struct ConfigSegmentDetector {
    segments: Vec<ConfigSegmentDefinition>,
    default_segment: String,
}

impl ConfigSegmentDetector {
    /// Create a new segment detector
    pub fn new(segments: Vec<ConfigSegmentDefinition>, default_segment: impl Into<String>) -> Self {
        let mut sorted_segments = segments;
        // Sort by priority (lower = higher priority)
        sorted_segments.sort_by_key(|s| s.priority);

        Self {
            segments: sorted_segments,
            default_segment: default_segment.into(),
        }
    }

}

impl SegmentDetector for ConfigSegmentDetector {
    fn detect(
        &self,
        text: &str,
        language: &str,
        numeric_values: &HashMap<String, f64>,
        text_values: &HashMap<String, String>,
    ) -> Vec<SegmentMatch> {
        let mut matches = Vec::new();

        for segment in &self.segments {
            if segment.matches(text, language, numeric_values, text_values) {
                matches.push(SegmentMatch {
                    segment_id: segment.id().to_string(),
                    confidence: segment.match_confidence(text, language, numeric_values, text_values),
                    match_reasons: Vec::new(), // Could populate with specific matches
                });
            }
        }

        // Already sorted by priority (from constructor)
        matches
    }

    fn primary_segment(
        &self,
        text: &str,
        language: &str,
        numeric_values: &HashMap<String, f64>,
        text_values: &HashMap<String, String>,
    ) -> &str {
        for segment in &self.segments {
            if segment.matches(text, language, numeric_values, text_values) {
                return segment.id();
            }
        }
        &self.default_segment
    }

    fn get_segment(&self, id: &str) -> Option<&dyn SegmentDefinition> {
        self.segments
            .iter()
            .find(|s| s.id() == id)
            .map(|s| s as &dyn SegmentDefinition)
    }

    fn all_segments(&self) -> Vec<&dyn SegmentDefinition> {
        self.segments
            .iter()
            .map(|s| s as &dyn SegmentDefinition)
            .collect()
    }

    fn default_segment(&self) -> &str {
        &self.default_segment
    }

    fn segment_ids(&self) -> Vec<&str> {
        self.segments.iter().map(|s| s.id()).collect()
    }

    fn features_for_segments(&self, segment_ids: &[&str]) -> Vec<String> {
        let mut features = Vec::new();
        for id in segment_ids {
            if let Some(segment) = self.get_segment(id) {
                for feature in segment.features() {
                    if !features.contains(&feature.to_string()) {
                        features.push(feature.to_string());
                    }
                }
            }
        }
        features
    }

    fn value_propositions(&self, segment_id: &str, language: &str) -> Vec<String> {
        self.get_segment(segment_id)
            .map(|s| {
                s.value_propositions(language)
                    .iter()
                    .map(|v| v.headline.clone())
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create test segment detector with typical segments
    fn test_detector() -> ConfigSegmentDetector {
        ConfigSegmentDetector::new(
            vec![
                ConfigSegmentDefinition::urgent_need(),
                ConfigSegmentDefinition::high_value(),
                ConfigSegmentDefinition::balance_transfer(),
                ConfigSegmentDefinition::trust_seeker(),
                ConfigSegmentDefinition::price_sensitive(),
                ConfigSegmentDefinition::business_owner(),
                ConfigSegmentDefinition::first_time(),
            ],
            "first_time",
        )
    }

    #[test]
    fn test_urgent_need_detection() {
        let detector = test_detector();
        let values = HashMap::new();
        let slots = HashMap::new();

        let primary = detector.primary_segment("I urgently need money", "en", &values, &slots);
        assert_eq!(primary, "urgent_need");
    }

    #[test]
    fn test_high_value_by_amount() {
        let detector = test_detector();
        let mut values = HashMap::new();
        values.insert("loan_amount".to_string(), 600_000.0);
        let slots = HashMap::new();

        let primary = detector.primary_segment("I need a loan", "en", &values, &slots);
        assert_eq!(primary, "high_value");
    }

    #[test]
    fn test_trust_seeker_by_lender() {
        let detector = test_detector();
        let values = HashMap::new();
        let mut slots = HashMap::new();
        slots.insert("current_lender".to_string(), "muthoot".to_string());

        // Use text that doesn't trigger balance_transfer segment
        let primary = detector.primary_segment("I need a loan", "en", &values, &slots);
        assert_eq!(primary, "trust_seeker");
    }

    #[test]
    fn test_default_segment() {
        let detector = test_detector();
        let values = HashMap::new();
        let slots = HashMap::new();

        let primary = detector.primary_segment("hello", "en", &values, &slots);
        assert_eq!(primary, "first_time");
    }

    #[test]
    fn test_segment_features() {
        let detector = test_detector();
        let features = detector.features_for_segments(&["urgent_need"]);
        assert!(features.contains(&"same_day_disbursement".to_string()));
    }

    #[test]
    fn test_multiple_segments_detected() {
        let detector = test_detector();
        let mut values = HashMap::new();
        values.insert("loan_amount".to_string(), 600_000.0);
        let slots = HashMap::new();

        // "urgent" + high value amount should detect both
        let matches = detector.detect("I urgently need 6 lakh", "en", &values, &slots);
        assert!(matches.len() >= 2);
    }
}
