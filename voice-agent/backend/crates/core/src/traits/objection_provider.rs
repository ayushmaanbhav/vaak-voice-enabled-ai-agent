//! Objection Provider trait for domain-agnostic objection handling
//!
//! This module provides a domain-agnostic interface for detecting customer objections
//! and generating appropriate responses. All objection definitions are loaded from
//! configuration (objections.yaml).
//!
//! # P20 FIX: Replaces hardcoded Objection enum in personalization/adaptation.rs
//!
//! The previous Objection enum had domain-specific variants like `GoldSafety`
//! with hardcoded detection keywords. This trait enables fully config-driven
//! objection detection with localized patterns and responses.
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_core::traits::ObjectionProvider;
//!
//! // Provider is created from domain config
//! let provider = config_bridge.objection_provider();
//!
//! // Detect objection from user text
//! let result = provider.detect_objection("Is my gold safe with you?", "en", &competitors);
//! if let Some((objection_id, confidence)) = result {
//!     // Get ACRE response
//!     let response = provider.get_acre_response(&objection_id, "en", &variables);
//! }
//! ```

use super::feature_provider::{substitute_variables, VariableMap};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Objection ID type alias for domain-agnostic objection references
pub type ObjectionId = String;

/// Well-known objection IDs (convenience constants, not exhaustive)
///
/// These use generic names that work across domains.
/// Domain-specific objection names are defined in config with aliases.
pub mod objection_ids {
    /// Concerns about collateral safety (was "gold_safety")
    pub const COLLATERAL_SAFETY: &str = "collateral_safety";
    /// Competitor offers better rates
    pub const BETTER_RATES_ELSEWHERE: &str = "better_rates_elsewhere";
    /// Too much paperwork/documentation
    pub const TOO_MUCH_PAPERWORK: &str = "too_much_paperwork";
    /// Doesn't want to switch providers
    pub const DONT_WANT_TO_SWITCH: &str = "dont_want_to_switch";
    /// Needs time to think/decide
    pub const NEEDS_TIME: &str = "needs_time";
    /// Trust issues with the company
    pub const TRUST_ISSUES: &str = "trust_issues";
    /// Expects hidden charges
    pub const EXPECTS_HIDDEN_CHARGES: &str = "expects_hidden_charges";
    /// Process is too slow
    pub const TOO_SLOW: &str = "too_slow";
    /// No nearby branch/location
    pub const NO_NEARBY_BRANCH: &str = "no_nearby_branch";
    /// Already has existing loans
    pub const EXISTING_LOANS: &str = "existing_loans";
}

/// Objection detection result
#[derive(Debug, Clone)]
pub struct ObjectionDetection {
    /// Objection ID
    pub objection_id: String,
    /// Detection confidence (0.0 - 1.0)
    pub confidence: f32,
    /// Matched patterns that triggered detection
    pub matched_patterns: Vec<String>,
    /// Language of detected patterns
    pub language: String,
}

/// ACRE response framework
///
/// Acknowledge, Clarify, Respond, Engage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcreResponseParts {
    /// Acknowledge the customer's concern
    pub acknowledge: String,
    /// Clarify/reframe the concern (optional)
    pub clarify: Option<String>,
    /// Respond with information/evidence
    pub respond: String,
    /// Engage with a follow-up question
    pub engage: Option<String>,
}

impl AcreResponseParts {
    /// Build full response from parts
    pub fn full_response(&self) -> String {
        let mut parts = vec![self.acknowledge.clone()];
        if let Some(ref c) = self.clarify {
            parts.push(c.clone());
        }
        parts.push(self.respond.clone());
        if let Some(ref e) = self.engage {
            parts.push(e.clone());
        }
        parts.join(" ")
    }
}

/// Objection response with context
#[derive(Debug, Clone)]
pub struct ObjectionResponse {
    /// Objection ID
    pub objection_id: String,
    /// ACRE response parts
    pub acre: AcreResponseParts,
    /// Feature to highlight in response
    pub highlight_feature: Option<String>,
    /// Segment this response is tailored for
    pub segment: Option<String>,
}

/// Detection pattern for an objection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionPattern {
    /// Simple keyword/phrase patterns
    #[serde(default)]
    pub patterns: Vec<String>,
    /// Regex patterns (optional, for complex matching)
    #[serde(default)]
    pub regex_patterns: Vec<String>,
    /// Confidence boost for specific pattern matches
    #[serde(default)]
    pub confidence_boosts: Vec<PatternBoost>,
}

/// Confidence boost for specific patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternBoost {
    /// Pattern to match
    pub pattern: String,
    /// Boost amount (0.0 - 0.5)
    pub boost: f32,
}

/// Trait for objection definition
///
/// Defines a single objection with detection patterns and responses.
pub trait ObjectionDefinitionTrait: Send + Sync {
    /// Objection ID (e.g., "collateral_safety", "better_rates_elsewhere")
    fn id(&self) -> &str;

    /// Aliases for backward compatibility (e.g., ["gold_safety"] for "collateral_safety")
    fn aliases(&self) -> Vec<&str>;

    /// Display name for the objection
    fn display_name(&self, language: &str) -> String;

    /// Detection patterns by language
    fn patterns(&self, language: &str) -> Vec<&str>;

    /// Check if text matches this objection
    fn matches(&self, text: &str, language: &str, competitor_names: &[String]) -> bool;

    /// Get match confidence
    fn match_confidence(&self, text: &str, language: &str, competitor_names: &[String]) -> f32;

    /// Get ACRE response for a language
    fn acre_response(&self, language: &str, variables: &VariableMap) -> Option<AcreResponseParts>;

    /// Get segment-specific response
    fn segment_response(
        &self,
        segment_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Option<AcreResponseParts>;

    /// Get feature to highlight when handling this objection
    fn highlight_feature(&self) -> Option<&str>;
}

/// Trait for providing objection handling from config
///
/// This trait replaces the hardcoded Objection enum. All objection definitions,
/// detection patterns, and responses come from configuration.
pub trait ObjectionProvider: Send + Sync {
    /// Get all objection IDs defined in config
    fn all_objection_ids(&self) -> Vec<String>;

    /// Detect objection from user text
    ///
    /// # Arguments
    /// * `text` - User's text to analyze
    /// * `language` - Language code (e.g., "en", "hi")
    /// * `competitor_names` - Known competitor names for detection
    ///
    /// # Returns
    /// Tuple of (objection_id, confidence) if objection detected
    fn detect_objection(
        &self,
        text: &str,
        language: &str,
        competitor_names: &[String],
    ) -> Option<(String, f32)>;

    /// Detect all matching objections (for disambiguation)
    fn detect_all_objections(
        &self,
        text: &str,
        language: &str,
        competitor_names: &[String],
    ) -> Vec<ObjectionDetection>;

    /// Get ACRE response for an objection
    ///
    /// # Arguments
    /// * `objection_id` - The objection identifier
    /// * `language` - Language code
    /// * `variables` - Variables for substitution (e.g., {{company_name}})
    fn get_acre_response(
        &self,
        objection_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Option<AcreResponseParts>;

    /// Get segment-specific ACRE response
    fn get_segment_response(
        &self,
        objection_id: &str,
        segment_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Option<AcreResponseParts>;

    /// Build full response for objection
    fn build_response(
        &self,
        objection_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Option<String> {
        self.get_acre_response(objection_id, language, variables)
            .map(|acre| acre.full_response())
    }

    /// Get full objection response with context
    fn get_objection_response(
        &self,
        objection_id: &str,
        language: &str,
        segment_id: Option<&str>,
        variables: &VariableMap,
    ) -> Option<ObjectionResponse>;

    /// Get feature to highlight for an objection
    fn highlight_feature(&self, objection_id: &str) -> Option<String>;

    /// Resolve objection ID from alias
    ///
    /// Allows using legacy names like "gold_safety" which maps to "collateral_safety"
    fn resolve_alias(&self, alias_or_id: &str) -> Option<String>;
}

/// Config-driven objection definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigObjectionDef {
    /// Objection ID
    pub id: String,
    /// Aliases for backward compatibility
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Display names by language
    #[serde(default)]
    pub display_name: HashMap<String, String>,
    /// Detection patterns by language
    #[serde(default)]
    pub detection: HashMap<String, DetectionPattern>,
    /// ACRE responses by language
    #[serde(default)]
    pub responses: HashMap<String, ConfigAcreResponse>,
    /// Segment-specific responses
    #[serde(default)]
    pub segment_responses: HashMap<String, HashMap<String, ConfigAcreResponse>>,
    /// Feature to highlight
    #[serde(default)]
    pub highlight_feature: Option<String>,
    /// Whether objection is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Config ACRE response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigAcreResponse {
    pub acknowledge: String,
    #[serde(default)]
    pub clarify: Option<String>,
    pub respond: String,
    #[serde(default)]
    pub engage: Option<String>,
}

impl ConfigAcreResponse {
    /// Convert to AcreResponseParts with variable substitution
    pub fn to_parts(&self, variables: &VariableMap) -> AcreResponseParts {
        AcreResponseParts {
            acknowledge: substitute_variables(&self.acknowledge, variables),
            clarify: self
                .clarify
                .as_ref()
                .map(|s| substitute_variables(s, variables)),
            respond: substitute_variables(&self.respond, variables),
            engage: self
                .engage
                .as_ref()
                .map(|s| substitute_variables(s, variables)),
        }
    }
}

impl ObjectionDefinitionTrait for ConfigObjectionDef {
    fn id(&self) -> &str {
        &self.id
    }

    fn aliases(&self) -> Vec<&str> {
        self.aliases.iter().map(|s| s.as_str()).collect()
    }

    fn display_name(&self, language: &str) -> String {
        self.display_name
            .get(language)
            .or_else(|| self.display_name.get("en"))
            .cloned()
            .unwrap_or_else(|| self.id.clone())
    }

    fn patterns(&self, language: &str) -> Vec<&str> {
        self.detection
            .get(language)
            .or_else(|| self.detection.get("en"))
            .map(|d| d.patterns.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    fn matches(&self, text: &str, language: &str, _competitor_names: &[String]) -> bool {
        let lower = text.to_lowercase();
        let patterns = self.patterns(language);
        patterns.iter().any(|p| lower.contains(&p.to_lowercase()))
    }

    fn match_confidence(&self, text: &str, language: &str, competitor_names: &[String]) -> f32 {
        if !self.matches(text, language, competitor_names) {
            return 0.0;
        }

        let lower = text.to_lowercase();
        let mut confidence = 0.5; // Base confidence

        // Count pattern matches
        let patterns = self.patterns(language);
        let match_count = patterns
            .iter()
            .filter(|p| lower.contains(&p.to_lowercase()))
            .count();
        confidence += (match_count as f32 * 0.1).min(0.3);

        // Apply boosts from config
        if let Some(detection) = self
            .detection
            .get(language)
            .or_else(|| self.detection.get("en"))
        {
            for boost in &detection.confidence_boosts {
                if lower.contains(&boost.pattern.to_lowercase()) {
                    confidence += boost.boost;
                }
            }
        }

        confidence.min(1.0)
    }

    fn acre_response(&self, language: &str, variables: &VariableMap) -> Option<AcreResponseParts> {
        self.responses
            .get(language)
            .or_else(|| self.responses.get("en"))
            .map(|r| r.to_parts(variables))
    }

    fn segment_response(
        &self,
        segment_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Option<AcreResponseParts> {
        self.segment_responses
            .get(segment_id)
            .and_then(|by_lang| by_lang.get(language).or_else(|| by_lang.get("en")))
            .map(|r| r.to_parts(variables))
    }

    fn highlight_feature(&self) -> Option<&str> {
        self.highlight_feature.as_deref()
    }
}

/// Config-driven objection provider
///
/// Aggregates multiple `ConfigObjectionDef` instances and implements
/// the `ObjectionProvider` trait for domain-agnostic objection handling.
#[derive(Debug, Clone)]
pub struct ConfigObjectionProvider {
    /// All objection definitions (objection_id -> definition)
    objections: HashMap<String, ConfigObjectionDef>,
    /// Alias mappings (alias -> canonical_id)
    aliases: HashMap<String, String>,
}

impl ConfigObjectionProvider {
    /// Create from config structures
    pub fn new(objections: Vec<ConfigObjectionDef>) -> Self {
        let mut aliases = HashMap::new();
        let mut objections_map = HashMap::new();

        for obj in objections {
            if obj.enabled {
                // Register aliases
                for alias in &obj.aliases {
                    aliases.insert(alias.clone(), obj.id.clone());
                }
                objections_map.insert(obj.id.clone(), obj);
            }
        }

        Self {
            objections: objections_map,
            aliases,
        }
    }

    /// Get an objection definition by ID
    pub fn get_objection(&self, objection_id: &str) -> Option<&ConfigObjectionDef> {
        let canonical = self.resolve_canonical_id(objection_id);
        self.objections.get(canonical)
    }

    /// Resolve alias to canonical ID
    fn resolve_canonical_id<'a>(&'a self, alias_or_id: &'a str) -> &'a str {
        self.aliases
            .get(alias_or_id)
            .map(|s| s.as_str())
            .unwrap_or(alias_or_id)
    }
}

impl ObjectionProvider for ConfigObjectionProvider {
    fn all_objection_ids(&self) -> Vec<String> {
        self.objections.keys().cloned().collect()
    }

    fn detect_objection(
        &self,
        text: &str,
        language: &str,
        competitor_names: &[String],
    ) -> Option<(String, f32)> {
        self.detect_all_objections(text, language, competitor_names)
            .into_iter()
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
            .map(|d| (d.objection_id, d.confidence))
    }

    fn detect_all_objections(
        &self,
        text: &str,
        language: &str,
        competitor_names: &[String],
    ) -> Vec<ObjectionDetection> {
        self.objections
            .values()
            .filter_map(|obj| {
                let confidence = obj.match_confidence(text, language, competitor_names);
                if confidence > 0.0 {
                    let patterns = obj.patterns(language);
                    let lower = text.to_lowercase();
                    let matched: Vec<String> = patterns
                        .into_iter()
                        .filter(|p| lower.contains(&p.to_lowercase()))
                        .map(|s| s.to_string())
                        .collect();

                    Some(ObjectionDetection {
                        objection_id: obj.id.clone(),
                        confidence,
                        matched_patterns: matched,
                        language: language.to_string(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn get_acre_response(
        &self,
        objection_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Option<AcreResponseParts> {
        self.get_objection(objection_id)
            .and_then(|obj| obj.acre_response(language, variables))
    }

    fn get_segment_response(
        &self,
        objection_id: &str,
        segment_id: &str,
        language: &str,
        variables: &VariableMap,
    ) -> Option<AcreResponseParts> {
        self.get_objection(objection_id)
            .and_then(|obj| obj.segment_response(segment_id, language, variables))
    }

    fn get_objection_response(
        &self,
        objection_id: &str,
        language: &str,
        segment_id: Option<&str>,
        variables: &VariableMap,
    ) -> Option<ObjectionResponse> {
        let obj = self.get_objection(objection_id)?;

        // Try segment-specific response first
        let acre = if let Some(seg) = segment_id {
            obj.segment_response(seg, language, variables)
                .or_else(|| obj.acre_response(language, variables))
        } else {
            obj.acre_response(language, variables)
        }?;

        Some(ObjectionResponse {
            objection_id: obj.id.clone(),
            acre,
            highlight_feature: obj.highlight_feature.clone(),
            segment: segment_id.map(|s| s.to_string()),
        })
    }

    fn highlight_feature(&self, objection_id: &str) -> Option<String> {
        self.get_objection(objection_id)
            .and_then(|obj| obj.highlight_feature.clone())
    }

    fn resolve_alias(&self, alias_or_id: &str) -> Option<String> {
        let canonical = self.resolve_canonical_id(alias_or_id);
        if self.objections.contains_key(canonical) {
            Some(canonical.to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_objection_detection() {
        let mut detection = HashMap::new();
        detection.insert(
            "en".to_string(),
            DetectionPattern {
                patterns: vec!["safe".to_string(), "security".to_string()],
                regex_patterns: vec![],
                confidence_boosts: vec![PatternBoost {
                    pattern: "gold.*safe".to_string(),
                    boost: 0.2,
                }],
            },
        );

        let objection = ConfigObjectionDef {
            id: "collateral_safety".to_string(),
            aliases: vec!["gold_safety".to_string()],
            display_name: HashMap::new(),
            detection,
            responses: HashMap::new(),
            segment_responses: HashMap::new(),
            highlight_feature: Some("security".to_string()),
            enabled: true,
        };

        assert!(objection.matches("Is my gold safe?", "en", &[]));
        assert!(objection.matches("I have security concerns", "en", &[]));
        assert!(!objection.matches("What is the interest rate?", "en", &[]));

        let confidence = objection.match_confidence("Is my gold safe?", "en", &[]);
        assert!(confidence > 0.5);
    }

    #[test]
    fn test_acre_response_substitution() {
        let response = ConfigAcreResponse {
            acknowledge: "I understand your concern about {{collateral_type}} safety.".to_string(),
            clarify: None,
            respond: "Your {{collateral_type}} is stored in {{company_name}}'s secure vaults."
                .to_string(),
            engage: Some("Would you like to visit our branch?".to_string()),
        };

        let mut vars = HashMap::new();
        vars.insert("collateral_type".to_string(), "gold".to_string());
        vars.insert("company_name".to_string(), "Kotak Bank".to_string());

        let parts = response.to_parts(&vars);
        assert_eq!(
            parts.acknowledge,
            "I understand your concern about gold safety."
        );
        assert!(parts.respond.contains("Kotak Bank"));
    }
}
