//! Objection Handler trait for detecting and responding to customer objections
//!
//! This module provides a domain-agnostic interface for objection detection
//! using keyword patterns and ACRE (Acknowledge-Clarify-Respond-Engage) response building.
//! All objection definitions are loaded from configuration.
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_core::traits::ObjectionHandler;
//!
//! // Handler is created from domain config
//! let handler = config_bridge.objection_handler();
//!
//! // Detect objection from user text
//! if let Some(objection_type) = handler.detect_objection("I'm worried about safety", "en") {
//!     let response = handler.handle_objection(objection_type, "en", Some("Ramesh"));
//! }
//! ```

use std::collections::HashMap;

/// ACRE (Acknowledge-Clarify-Respond-Engage) response components
#[derive(Debug, Clone)]
pub struct AcreResponse {
    /// Acknowledge the customer's concern (validate feeling)
    pub acknowledge: String,
    /// Reframe/Clarify the concern (shift perspective)
    pub reframe: String,
    /// Evidence/Response with facts to support
    pub evidence: String,
    /// Call to action (next step)
    pub call_to_action: String,
}

impl AcreResponse {
    /// Create a new ACRE response
    pub fn new(
        acknowledge: impl Into<String>,
        reframe: impl Into<String>,
        evidence: impl Into<String>,
        call_to_action: impl Into<String>,
    ) -> Self {
        Self {
            acknowledge: acknowledge.into(),
            reframe: reframe.into(),
            evidence: evidence.into(),
            call_to_action: call_to_action.into(),
        }
    }

    /// Build full response text
    pub fn build(&self) -> String {
        format!(
            "{} {} {} {}",
            self.acknowledge, self.reframe, self.evidence, self.call_to_action
        )
    }

    /// Build response with customer name
    pub fn build_with_name(&self, name: Option<&str>) -> String {
        if let Some(n) = name {
            format!(
                "{}, {} {} {} {}",
                n, self.acknowledge, self.reframe, self.evidence, self.call_to_action
            )
        } else {
            self.build()
        }
    }
}

/// Objection detection match
#[derive(Debug, Clone)]
pub struct ObjectionMatch {
    /// Objection type ID
    pub objection_type: String,
    /// Match confidence (0.0 - 1.0)
    pub confidence: f32,
    /// Matched patterns
    pub matched_patterns: Vec<String>,
}

/// Objection definition trait
///
/// Defines an objection type with detection patterns and ACRE response.
pub trait ObjectionDefinition: Send + Sync {
    /// Objection type ID (e.g., "safety", "interest_rate")
    fn id(&self) -> &str;

    /// Human-readable display name
    fn display_name(&self) -> &str;

    /// Objection description
    fn description(&self) -> &str;

    /// Detection patterns for a language
    fn patterns(&self, language: &str) -> Vec<&str>;

    /// Get ACRE response for a language
    fn response(&self, language: &str) -> Option<&AcreResponse>;

    /// Check if text contains this objection
    fn matches(&self, text: &str, language: &str) -> bool {
        let lower_text = text.to_lowercase();
        self.patterns(language)
            .iter()
            .any(|p| lower_text.contains(&p.to_lowercase()))
    }

    /// Get match confidence
    fn match_confidence(&self, text: &str, language: &str) -> f32 {
        let lower_text = text.to_lowercase();
        let patterns = self.patterns(language);
        if patterns.is_empty() {
            return 0.0;
        }

        let matched = patterns
            .iter()
            .filter(|p| lower_text.contains(&p.to_lowercase()))
            .count();

        (matched as f32 / patterns.len() as f32).min(1.0)
    }

    /// Priority for detection (lower = check first)
    fn priority(&self) -> u8 {
        50
    }
}

/// Objection handler trait
///
/// Detects objections from text and provides appropriate responses.
pub trait ObjectionHandler: Send + Sync {
    /// Detect objection type from text
    ///
    /// Returns the ID of the detected objection type, or None if no objection.
    fn detect_objection(&self, text: &str, language: &str) -> Option<&str>;

    /// Detect all matching objections
    fn detect_all_objections(&self, text: &str, language: &str) -> Vec<ObjectionMatch>;

    /// Get objection definition by ID
    fn get_objection(&self, id: &str) -> Option<&dyn ObjectionDefinition>;

    /// Get all objection definitions
    fn all_objections(&self) -> Vec<&dyn ObjectionDefinition>;

    /// Get all objection type IDs
    fn objection_types(&self) -> Vec<&str>;

    /// Build response for detected objection
    fn handle_objection(
        &self,
        objection_type: &str,
        language: &str,
        customer_name: Option<&str>,
    ) -> Option<String>;

    /// Get ACRE response for objection
    fn get_acre_response(&self, objection_type: &str, language: &str) -> Option<&AcreResponse>;

    /// Get default response for unknown objections
    fn default_response(&self, language: &str) -> String;
}

/// Config-driven objection definition
#[derive(Debug, Clone)]
pub struct ConfigObjectionDefinition {
    id: String,
    display_name: String,
    description: String,
    patterns: HashMap<String, Vec<String>>,
    responses: HashMap<String, AcreResponse>,
    priority: u8,
}

impl ConfigObjectionDefinition {
    /// Create a new objection definition
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            description: description.into(),
            patterns: HashMap::new(),
            responses: HashMap::new(),
            priority: 50,
        }
    }

    /// Add detection patterns for a language
    pub fn with_patterns(mut self, language: &str, patterns: Vec<String>) -> Self {
        self.patterns.insert(language.to_string(), patterns);
        self
    }

    /// Add ACRE response for a language
    pub fn with_response(mut self, language: &str, response: AcreResponse) -> Self {
        self.responses.insert(language.to_string(), response);
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    // NOTE: Domain-specific factory methods (safety, interest_rate, gold_security,
    // process_complexity, need_time, current_lender_satisfaction, hidden_charges,
    // documentation, trust_issues) have been REMOVED.
    //
    // All objection definitions should come from config/domains/{domain}/objections.yaml
    // via DomainBridge. This makes the system truly domain-agnostic.
    //
    // To add objections for a new domain:
    // 1. Create config/domains/{domain}/objections.yaml
    // 2. Define patterns and ACRE responses in YAML
    // 3. Load via MasterDomainConfig.objections
    // 4. Access via DomainBridge.objection_handler()
}

impl ObjectionDefinition for ConfigObjectionDefinition {
    fn id(&self) -> &str {
        &self.id
    }

    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn patterns(&self, language: &str) -> Vec<&str> {
        self.patterns
            .get(language)
            .map(|p| p.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    fn response(&self, language: &str) -> Option<&AcreResponse> {
        self.responses.get(language)
    }

    fn priority(&self) -> u8 {
        self.priority
    }
}

/// Config-driven objection handler
pub struct ConfigObjectionHandler {
    objections: Vec<ConfigObjectionDefinition>,
    default_responses: HashMap<String, String>,
}

impl ConfigObjectionHandler {
    /// Create a new objection handler
    pub fn new(objections: Vec<ConfigObjectionDefinition>) -> Self {
        let mut sorted = objections;
        sorted.sort_by_key(|o| o.priority);

        Self {
            objections: sorted,
            default_responses: [
                ("en".to_string(), "I understand your concern. Let me address that for you.".to_string()),
                ("hi".to_string(), "मैं आपकी चिंता समझता हूं। मुझे इसका जवाब देने दीजिए।".to_string()),
            ].into_iter().collect(),
        }
    }

}

impl ObjectionHandler for ConfigObjectionHandler {
    fn detect_objection(&self, text: &str, language: &str) -> Option<&str> {
        for objection in &self.objections {
            if objection.matches(text, language) {
                return Some(objection.id());
            }
        }
        None
    }

    fn detect_all_objections(&self, text: &str, language: &str) -> Vec<ObjectionMatch> {
        self.objections
            .iter()
            .filter(|o| o.matches(text, language))
            .map(|o| ObjectionMatch {
                objection_type: o.id().to_string(),
                confidence: o.match_confidence(text, language),
                matched_patterns: Vec::new(),
            })
            .collect()
    }

    fn get_objection(&self, id: &str) -> Option<&dyn ObjectionDefinition> {
        self.objections
            .iter()
            .find(|o| o.id() == id)
            .map(|o| o as &dyn ObjectionDefinition)
    }

    fn all_objections(&self) -> Vec<&dyn ObjectionDefinition> {
        self.objections
            .iter()
            .map(|o| o as &dyn ObjectionDefinition)
            .collect()
    }

    fn objection_types(&self) -> Vec<&str> {
        self.objections.iter().map(|o| o.id()).collect()
    }

    fn handle_objection(
        &self,
        objection_type: &str,
        language: &str,
        customer_name: Option<&str>,
    ) -> Option<String> {
        self.get_acre_response(objection_type, language)
            .map(|r| r.build_with_name(customer_name))
    }

    fn get_acre_response(&self, objection_type: &str, language: &str) -> Option<&AcreResponse> {
        self.objections
            .iter()
            .find(|o| o.id() == objection_type)
            .and_then(|o| o.response(language))
    }

    fn default_response(&self, language: &str) -> String {
        self.default_responses
            .get(language)
            .cloned()
            .unwrap_or_else(|| "I understand your concern.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create generic test objection using builder pattern
    fn create_test_objection(
        id: &str,
        patterns_en: Vec<&str>,
        response_text: &str,
    ) -> ConfigObjectionDefinition {
        ConfigObjectionDefinition::new(id, id, format!("Test objection: {}", id))
            .with_patterns("en", patterns_en.into_iter().map(String::from).collect())
            .with_response(
                "en",
                AcreResponse::new(
                    "I understand your concern.",
                    "Let me address that.",
                    response_text,
                    "Would you like more information?",
                ),
            )
    }

    /// Create test objection handler with generic test objections
    fn test_handler() -> ConfigObjectionHandler {
        ConfigObjectionHandler::new(vec![
            create_test_objection("concern_a", vec!["worried", "concern", "scared"], "We take this seriously."),
            create_test_objection("concern_b", vec!["expensive", "costly", "high"], "Let me explain our value."),
            create_test_objection("concern_c", vec!["complicated", "difficult"], "We've simplified the process."),
            create_test_objection("concern_d", vec!["think", "later"], "Take your time to decide."),
        ])
    }

    #[test]
    fn test_objection_detection() {
        let handler = test_handler();

        assert_eq!(
            handler.detect_objection("I'm worried about this", "en"),
            Some("concern_a")
        );
        assert_eq!(
            handler.detect_objection("Is it too expensive?", "en"),
            Some("concern_b")
        );
    }

    #[test]
    fn test_acre_response() {
        let handler = test_handler();

        let response = handler.handle_objection("concern_a", "en", Some("Customer"));
        assert!(response.is_some());
        let text = response.unwrap();
        assert!(text.contains("Customer"));
        assert!(text.contains("understand"));
    }

    #[test]
    fn test_no_objection() {
        let handler = test_handler();

        assert_eq!(
            handler.detect_objection("Hello, how are you?", "en"),
            None
        );
    }

    #[test]
    fn test_all_objection_types() {
        let handler = test_handler();
        let types = handler.objection_types();

        assert_eq!(types.len(), 4);
        assert!(types.contains(&"concern_a"));
        assert!(types.contains(&"concern_b"));
        assert!(types.contains(&"concern_c"));
        assert!(types.contains(&"concern_d"));
    }

    #[test]
    fn test_multiple_objections() {
        let handler = test_handler();
        let matches = handler.detect_all_objections("I'm worried and it seems expensive", "en");

        assert!(matches.len() >= 2);
    }

    #[test]
    fn test_objection_builder() {
        let objection = ConfigObjectionDefinition::new("test", "Test", "Test objection")
            .with_patterns("en", vec!["keyword1".to_string(), "keyword2".to_string()])
            .with_patterns("hi", vec!["कीवर्ड".to_string()])
            .with_response(
                "en",
                AcreResponse::new("Ack", "Reframe", "Evidence", "CTA"),
            )
            .with_priority(10);

        assert_eq!(objection.id(), "test");
        assert_eq!(objection.patterns("en").len(), 2);
        assert_eq!(objection.patterns("hi").len(), 1);
        assert!(objection.response("en").is_some());
        assert_eq!(objection.priority(), 10);
    }

    #[test]
    fn test_default_response() {
        let handler = test_handler();
        let default = handler.default_response("en");
        assert!(!default.is_empty());
    }
}
