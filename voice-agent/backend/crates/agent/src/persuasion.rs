//! Domain-Agnostic Persuasion Engine
//!
//! Implements config-driven objection handling for any domain:
//! - Acknowledge: Validate customer concerns
//! - Reframe: Present alternative perspective
//! - Evidence: Provide supporting facts/data
//! - Value Proposition: Articulate key benefits
//!
//! ## Architecture
//!
//! The persuasion engine is fully config-driven:
//! - Objection types defined in `objections.yaml` (not hardcoded enum)
//! - Detection patterns loaded from config
//! - Response templates support brand variable substitution
//! - Competitor data from `competitors.yaml`
//! - Value propositions from `segments.yaml`
//!
//! ## Usage
//!
//! ```ignore
//! // Create from domain config (recommended)
//! let engine = PersuasionEngine::from_view(&domain_view);
//!
//! // Detect and handle objection
//! if let Some(response) = engine.handle_objection(user_text, Language::English) {
//!     println!("{}", response.full_response());
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use voice_agent_config::domain::AgentDomainView;
use voice_agent_config::{ActionContext, ObjectionsConfig};
use voice_agent_core::Language;

// =============================================================================
// Objection Types - String-Based (Domain-Agnostic)
// =============================================================================

/// Objection ID - string-based identifier for objection types
///
/// Objection types are defined in config/domains/{domain}/objections.yaml.
/// Common IDs include: "safety", "interest_rate", "process_complexity", etc.
/// New domains can define their own objection types without code changes.
pub type ObjectionId = String;

/// Common objection IDs (reference - actual types come from config)
pub mod objection_ids {
    pub const SAFETY: &str = "safety";
    pub const INTEREST_RATE: &str = "interest_rate";
    pub const PROCESS_COMPLEXITY: &str = "process_complexity";
    pub const NEED_TIME: &str = "need_time";
    pub const CURRENT_LENDER_SATISFACTION: &str = "current_lender_satisfaction";
    pub const HIDDEN_CHARGES: &str = "hidden_charges";
    pub const DOCUMENTATION: &str = "documentation";
    pub const TRUST_ISSUES: &str = "trust_issues";
    pub const OTHER: &str = "other";
}

// =============================================================================
// Backward Compatibility - Deprecated ObjectionType Enum
// =============================================================================

/// Types of objections (DEPRECATED - use string-based ObjectionId)
///
/// This enum is kept for backward compatibility only.
/// New code should use string-based objection IDs from config.
#[deprecated(note = "Use string-based ObjectionId from config instead")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectionType {
    Safety,
    InterestRate,
    GoldSecurity,
    ProcessComplexity,
    NeedTime,
    CurrentLenderSatisfaction,
    HiddenCharges,
    Documentation,
    TrustIssues,
    Other,
}

#[allow(deprecated)]
impl ObjectionType {
    /// Convert to string ID
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Safety => "safety",
            Self::InterestRate => "interest_rate",
            Self::GoldSecurity => "gold_security",
            Self::ProcessComplexity => "process_complexity",
            Self::NeedTime => "need_time",
            Self::CurrentLenderSatisfaction => "current_lender_satisfaction",
            Self::HiddenCharges => "hidden_charges",
            Self::Documentation => "documentation",
            Self::TrustIssues => "trust_issues",
            Self::Other => "other",
        }
    }

    /// Parse from string ID
    pub fn from_str(s: &str) -> Self {
        match s {
            "safety" => Self::Safety,
            "interest_rate" => Self::InterestRate,
            "gold_security" => Self::GoldSecurity,
            "process_complexity" => Self::ProcessComplexity,
            "need_time" => Self::NeedTime,
            "current_lender_satisfaction" => Self::CurrentLenderSatisfaction,
            "hidden_charges" => Self::HiddenCharges,
            "documentation" => Self::Documentation,
            "trust_issues" => Self::TrustIssues,
            _ => Self::Other,
        }
    }

    /// Detect from text (DEPRECATED - use ObjectionDetector)
    #[deprecated(note = "Use ObjectionDetector::detect() for config-driven detection")]
    pub fn detect(_text: &str) -> Self {
        // Fallback - returns Other, use ObjectionDetector instead
        Self::Other
    }
}

// =============================================================================
// PersuasionStrategy Trait (Domain-Agnostic)
// =============================================================================

/// Trait for persuasion/objection handling abstraction
///
/// This trait allows domain-agnostic agents to work with any persuasion
/// implementation. Uses string-based objection IDs for flexibility.
pub trait PersuasionStrategy: Send + Sync {
    /// Handle an objection and return appropriate response
    ///
    /// Detects the objection type from text using config patterns,
    /// then returns the appropriate response with brand substitution.
    fn handle_objection(&self, text: &str, language: Language) -> Option<ObjectionResponse>;

    /// Get response for a specific objection type (by string ID)
    fn get_response_by_id(
        &self,
        objection_id: &str,
        language: Language,
    ) -> Option<ObjectionResponse>;

    /// Get response for a specific objection type (deprecated enum)
    #[allow(deprecated)]
    fn get_response(
        &self,
        objection_type: ObjectionType,
        language: Language,
    ) -> Option<ObjectionResponse> {
        self.get_response_by_id(objection_type.as_str(), language)
    }

    /// Get value proposition for customer segment
    fn get_value_proposition(&self, segment: &str) -> Option<ValueProposition>;

    /// Get competitor comparison data
    fn get_competitor_comparison(&self, competitor: &str) -> Option<CompetitorComparison>;

    /// Calculate savings for switching from a competitor
    fn calculate_switch_savings(
        &self,
        competitor: &str,
        amount: f64,
    ) -> Option<SwitchSavings>;

    /// Generate a full persuasion script for a scenario
    fn generate_script(
        &self,
        objection_id: &str,
        language: Language,
        customer_segment: Option<&str>,
    ) -> PersuasionScript;

    /// Get all available objection types (from config)
    fn available_objection_types(&self) -> Vec<&str>;

    /// Detect objection type from text using config patterns
    fn detect_objection(&self, text: &str, language: Language) -> Option<String>;
}

// =============================================================================
// Data Structures
// =============================================================================

/// Objection handling response with acknowledge-reframe-evidence pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectionResponse {
    /// The objection type ID
    pub objection_id: String,
    /// Acknowledgment of the concern (validates customer feeling)
    pub acknowledge: String,
    /// Reframe to shift perspective
    pub reframe: String,
    /// Evidence/facts to support the reframe
    pub evidence: String,
    /// Call to action
    pub call_to_action: String,
}

impl ObjectionResponse {
    /// Create a new objection response
    pub fn new(
        objection_id: impl Into<String>,
        acknowledge: impl Into<String>,
        reframe: impl Into<String>,
        evidence: impl Into<String>,
        call_to_action: impl Into<String>,
    ) -> Self {
        Self {
            objection_id: objection_id.into(),
            acknowledge: acknowledge.into(),
            reframe: reframe.into(),
            evidence: evidence.into(),
            call_to_action: call_to_action.into(),
        }
    }

    /// Get full response text
    pub fn full_response(&self) -> String {
        format!(
            "{} {} {} {}",
            self.acknowledge, self.reframe, self.evidence, self.call_to_action
        )
    }

    /// Apply brand substitution to all fields
    pub fn with_brand_context(&self, context: &ActionContext) -> Self {
        Self {
            objection_id: self.objection_id.clone(),
            acknowledge: context.substitute(&self.acknowledge),
            reframe: context.substitute(&self.reframe),
            evidence: context.substitute(&self.evidence),
            call_to_action: context.substitute(&self.call_to_action),
        }
    }
}

/// Value proposition for a customer segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueProposition {
    /// Segment ID
    pub segment_id: String,
    /// Headline benefit
    pub headline: String,
    /// Supporting points
    pub points: Vec<String>,
    /// Differentiator from competition
    pub differentiator: String,
    /// Social proof (testimonial, statistic)
    pub social_proof: String,
}

/// Comparison with a competitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitorComparison {
    /// Competitor ID
    pub competitor_id: String,
    /// Competitor display name
    pub name: String,
    /// Their typical rate
    pub their_rate: f64,
    /// Our rate
    pub our_rate: f64,
    /// Monthly savings per unit amount (e.g., per lakh)
    pub monthly_savings_per_unit: f64,
    /// Unit amount for savings calculation
    pub savings_unit_amount: f64,
    /// Additional advantages we offer
    pub our_advantages: Vec<String>,
}

/// Savings calculation for switching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchSavings {
    /// Competitor being compared
    pub competitor: String,
    /// Amount being compared
    pub amount: f64,
    /// Monthly savings
    pub monthly_savings: f64,
    /// Annual savings
    pub annual_savings: f64,
    /// Rate difference
    pub rate_difference: f64,
}

/// Full persuasion script for a scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersuasionScript {
    /// The objection being addressed
    pub objection_id: String,
    /// Language of the script
    pub language: Language,
    /// Customer segment (if applicable)
    pub segment: Option<String>,
    /// Opening response
    pub opening: String,
    /// Main persuasion points
    pub body: Vec<String>,
    /// Closing/call to action
    pub closing: String,
}

// =============================================================================
// ObjectionDetector - Config-Driven Detection
// =============================================================================

/// Detects objection types from text using config patterns
pub struct ObjectionDetector {
    /// Patterns by objection ID and language
    patterns: HashMap<String, HashMap<String, Vec<String>>>,
}

impl ObjectionDetector {
    /// Create from objections config
    pub fn from_config(config: &ObjectionsConfig) -> Self {
        let mut patterns = HashMap::new();

        for (objection_id, objection) in &config.objections {
            let mut lang_patterns = HashMap::new();
            for (lang, pats) in &objection.patterns {
                lang_patterns.insert(lang.clone(), pats.clone());
            }
            patterns.insert(objection_id.clone(), lang_patterns);
        }

        Self { patterns }
    }

    /// Detect objection type from text
    ///
    /// Returns the objection ID with highest pattern match score.
    pub fn detect(&self, text: &str, language: Language) -> Option<String> {
        let lower = text.to_lowercase();
        let lang_key = match language {
            Language::English => "en",
            Language::Hindi => "hi",
            _ => "en",
        };

        let mut best_match: Option<(String, usize)> = None;

        for (objection_id, lang_patterns) in &self.patterns {
            // Check primary language
            if let Some(patterns) = lang_patterns.get(lang_key) {
                let matches = patterns.iter().filter(|p| lower.contains(&p.to_lowercase())).count();
                if matches > 0 {
                    if best_match.as_ref().map_or(true, |(_, count)| matches > *count) {
                        best_match = Some((objection_id.clone(), matches));
                    }
                }
            }

            // Also check all languages for cross-language detection (e.g., Hindi words in English text)
            for (_, patterns) in lang_patterns {
                let matches = patterns.iter().filter(|p| lower.contains(&p.to_lowercase())).count();
                if matches > 0 {
                    if best_match.as_ref().map_or(true, |(_, count)| matches > *count) {
                        best_match = Some((objection_id.clone(), matches));
                    }
                }
            }
        }

        best_match.map(|(id, _)| id)
    }

    /// Get available objection types
    pub fn objection_types(&self) -> Vec<&str> {
        self.patterns.keys().map(|s| s.as_str()).collect()
    }
}

// =============================================================================
// PersuasionEngine - Main Implementation
// =============================================================================

/// Domain-agnostic persuasion engine
///
/// Loads all configuration from domain config files:
/// - Objection types and responses from `objections.yaml`
/// - Competitor data from `competitors.yaml`
/// - Value propositions from `segments.yaml`
pub struct PersuasionEngine {
    /// Objection handlers by (objection_id, language)
    handlers: HashMap<(String, Language), ObjectionResponse>,
    /// Value propositions by segment ID
    value_propositions: HashMap<String, ValueProposition>,
    /// Competitor comparison data by competitor ID
    competition_data: HashMap<String, CompetitorComparison>,
    /// Objection detector
    detector: ObjectionDetector,
    /// Brand context for variable substitution
    brand_context: ActionContext,
    /// Our base rate (for savings calculations)
    our_base_rate: f64,
}

impl PersuasionEngine {
    /// Create from domain view (recommended)
    ///
    /// Loads all objection handlers, competitor data, and value propositions
    /// from the domain configuration.
    pub fn from_view(view: &Arc<AgentDomainView>) -> Self {
        let mut engine = Self {
            handlers: HashMap::new(),
            value_propositions: HashMap::new(),
            competition_data: HashMap::new(),
            detector: ObjectionDetector::from_config(view.objections_config()),
            brand_context: ActionContext {
                tool_name: None,
                slot_id: None,
                slot_display: None,
                company_name: view.company_name().to_string(),
                product_name: view.product_name().to_string(),
                agent_name: view.agent_name().to_string(),
            },
            our_base_rate: view.our_rate_for_amount(500_000.0), // Get base rate from config
        };

        // Load objection handlers from config
        engine.load_handlers_from_config(view);

        // Load competitor data from config
        engine.load_competition_from_config(view);

        // Load value propositions from segments config
        engine.load_value_propositions_from_config(view);

        engine
    }

    /// Create with default/empty state (for testing)
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            value_propositions: HashMap::new(),
            competition_data: HashMap::new(),
            detector: ObjectionDetector { patterns: HashMap::new() },
            brand_context: ActionContext::default(),
            our_base_rate: 9.5,
        }
    }

    /// Load objection handlers from config
    fn load_handlers_from_config(&mut self, view: &Arc<AgentDomainView>) {
        let config = view.objections_config();

        for (objection_id, objection) in &config.objections {
            // Load English response
            if let Some(response) = &objection.responses.get("en") {
                self.handlers.insert(
                    (objection_id.clone(), Language::English),
                    ObjectionResponse::new(
                        objection_id,
                        &response.acknowledge,
                        &response.reframe,
                        &response.evidence,
                        &response.call_to_action,
                    ),
                );
            }

            // Load Hindi response
            if let Some(response) = &objection.responses.get("hi") {
                self.handlers.insert(
                    (objection_id.clone(), Language::Hindi),
                    ObjectionResponse::new(
                        objection_id,
                        &response.acknowledge,
                        &response.reframe,
                        &response.evidence,
                        &response.call_to_action,
                    ),
                );
            }
        }

        tracing::debug!(
            objection_count = config.objections.len(),
            handler_count = self.handlers.len(),
            "Loaded objection handlers from config"
        );
    }

    /// Load competitor data from config
    fn load_competition_from_config(&mut self, view: &Arc<AgentDomainView>) {
        let competitors = view.competitors_config();

        for (competitor_id, competitor) in &competitors.competitors {
            let their_rate = competitor.typical_rate;
            let our_rate = self.our_base_rate;

            // Calculate monthly savings per lakh
            let rate_diff = their_rate - our_rate;
            let monthly_savings_per_lakh = (rate_diff / 100.0 / 12.0) * 100_000.0;

            let advantages = competitor.weaknesses.clone();

            self.competition_data.insert(
                competitor_id.clone(),
                CompetitorComparison {
                    competitor_id: competitor_id.clone(),
                    name: competitor.display_name.clone(),
                    their_rate,
                    our_rate,
                    monthly_savings_per_unit: monthly_savings_per_lakh,
                    savings_unit_amount: 100_000.0,
                    our_advantages: advantages,
                },
            );
        }

        tracing::debug!(
            competitor_count = self.competition_data.len(),
            "Loaded competitor data from config"
        );
    }

    /// Load value propositions from segments config
    fn load_value_propositions_from_config(&mut self, view: &Arc<AgentDomainView>) {
        let segments = view.segments_config();

        for (segment_id, segment) in &segments.segments {
            // Get value props for English (or any first available language)
            let value_props = segment
                .value_props
                .get("en")
                .or_else(|| segment.value_props.values().next())
                .cloned()
                .unwrap_or_default();

            // Build value proposition from segment data
            let vp = ValueProposition {
                segment_id: segment_id.clone(),
                headline: segment.display_name.clone(),
                points: segment.features.clone(),
                // Use first value prop as differentiator, or description
                differentiator: value_props
                    .first()
                    .cloned()
                    .unwrap_or_else(|| segment.description.clone()),
                social_proof: if value_props.len() > 1 {
                    value_props[1].clone()
                } else {
                    format!("Priority {} segment", segment.priority)
                },
            };

            self.value_propositions.insert(segment_id.clone(), vp);
        }

        tracing::debug!(
            segment_count = self.value_propositions.len(),
            "Loaded value propositions from segments config"
        );
    }
}

impl Default for PersuasionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PersuasionStrategy for PersuasionEngine {
    fn handle_objection(&self, text: &str, language: Language) -> Option<ObjectionResponse> {
        // Detect objection type from text
        let objection_id = self.detect_objection(text, language)?;

        // Get response and apply brand context
        self.get_response_by_id(&objection_id, language)
    }

    fn get_response_by_id(
        &self,
        objection_id: &str,
        language: Language,
    ) -> Option<ObjectionResponse> {
        self.handlers
            .get(&(objection_id.to_string(), language))
            .or_else(|| {
                // Fallback to English if language-specific not found
                self.handlers.get(&(objection_id.to_string(), Language::English))
            })
            .map(|r| r.with_brand_context(&self.brand_context))
    }

    fn get_value_proposition(&self, segment: &str) -> Option<ValueProposition> {
        self.value_propositions.get(segment).cloned()
    }

    fn get_competitor_comparison(&self, competitor: &str) -> Option<CompetitorComparison> {
        // Try exact match first
        if let Some(comp) = self.competition_data.get(competitor) {
            return Some(comp.clone());
        }

        // Try lowercase match
        let lower = competitor.to_lowercase();
        for (id, comp) in &self.competition_data {
            if id.to_lowercase() == lower || comp.name.to_lowercase().contains(&lower) {
                return Some(comp.clone());
            }
        }

        None
    }

    fn calculate_switch_savings(
        &self,
        competitor: &str,
        amount: f64,
    ) -> Option<SwitchSavings> {
        let comp = self.get_competitor_comparison(competitor)?;

        let rate_diff = comp.their_rate - comp.our_rate;
        let monthly_savings = (rate_diff / 100.0 / 12.0) * amount;
        let annual_savings = monthly_savings * 12.0;

        Some(SwitchSavings {
            competitor: comp.name.clone(),
            amount,
            monthly_savings,
            annual_savings,
            rate_difference: rate_diff,
        })
    }

    fn generate_script(
        &self,
        objection_id: &str,
        language: Language,
        customer_segment: Option<&str>,
    ) -> PersuasionScript {
        let response = self.get_response_by_id(objection_id, language);
        let value_prop = customer_segment.and_then(|s| self.get_value_proposition(s));

        let opening = response
            .as_ref()
            .map(|r| r.acknowledge.clone())
            .unwrap_or_else(|| "I understand your concern.".to_string());

        let mut body = Vec::new();

        if let Some(ref r) = response {
            body.push(r.reframe.clone());
            body.push(r.evidence.clone());
        }

        if let Some(ref vp) = value_prop {
            body.push(vp.differentiator.clone());
        }

        let closing = response
            .as_ref()
            .map(|r| r.call_to_action.clone())
            .unwrap_or_else(|| "How can I help you further?".to_string());

        PersuasionScript {
            objection_id: objection_id.to_string(),
            language,
            segment: customer_segment.map(|s| s.to_string()),
            opening,
            body,
            closing,
        }
    }

    fn available_objection_types(&self) -> Vec<&str> {
        self.detector.objection_types()
    }

    fn detect_objection(&self, text: &str, language: Language) -> Option<String> {
        self.detector.detect(text, language)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_objection_response_creation() {
        let response = ObjectionResponse::new(
            "safety",
            "I understand",
            "Actually...",
            "Here's proof",
            "Want to continue?",
        );

        assert_eq!(response.objection_id, "safety");
        assert!(response.full_response().contains("I understand"));
    }

    #[test]
    fn test_brand_substitution() {
        let response = ObjectionResponse::new(
            "test",
            "{brand.bank_name} understands",
            "{brand.product_name} is great",
            "Evidence",
            "Call {brand.bank_name}",
        );

        let context = ActionContext {
            tool_name: None,
            slot_id: None,
            slot_display: None,
            company_name: "TestBank".to_string(),
            product_name: "TestProduct".to_string(),
            agent_name: "Agent".to_string(),
        };

        let substituted = response.with_brand_context(&context);
        assert!(substituted.acknowledge.contains("TestBank"));
        assert!(substituted.reframe.contains("TestProduct"));
    }

    #[test]
    fn test_detector_patterns() {
        let mut patterns = HashMap::new();
        let mut safety_patterns = HashMap::new();
        safety_patterns.insert("en".to_string(), vec!["safe".to_string(), "trust".to_string()]);
        safety_patterns.insert("hi".to_string(), vec!["bharosa".to_string()]);
        patterns.insert("safety".to_string(), safety_patterns);

        let detector = ObjectionDetector { patterns };

        assert_eq!(
            detector.detect("Is it safe?", Language::English),
            Some("safety".to_string())
        );
        assert_eq!(
            detector.detect("mujhe bharosa nahi", Language::Hindi),
            Some("safety".to_string())
        );
    }

    #[test]
    fn test_switch_savings_calculation() {
        let mut engine = PersuasionEngine::new();
        engine.competition_data.insert(
            "competitor".to_string(),
            CompetitorComparison {
                competitor_id: "competitor".to_string(),
                name: "Competitor Co".to_string(),
                their_rate: 18.0,
                our_rate: 10.0,
                monthly_savings_per_unit: 666.67,
                savings_unit_amount: 100_000.0,
                our_advantages: vec!["Better service".to_string()],
            },
        );

        let savings = engine.calculate_switch_savings("competitor", 500_000.0).unwrap();
        assert_eq!(savings.rate_difference, 8.0);
        assert!(savings.monthly_savings > 0.0);
        assert!(savings.annual_savings > savings.monthly_savings);
    }

    #[test]
    #[allow(deprecated)]
    fn test_objection_type_backward_compat() {
        assert_eq!(ObjectionType::Safety.as_str(), "safety");
        assert_eq!(ObjectionType::from_str("interest_rate"), ObjectionType::InterestRate);
        assert_eq!(ObjectionType::from_str("unknown"), ObjectionType::Other);
    }

    #[test]
    fn test_value_proposition() {
        let mut engine = PersuasionEngine::new();
        engine.value_propositions.insert(
            "premium".to_string(),
            ValueProposition {
                segment_id: "premium".to_string(),
                headline: "Premium benefits".to_string(),
                points: vec!["Point 1".to_string()],
                differentiator: "Unique".to_string(),
                social_proof: "1000+ customers".to_string(),
            },
        );

        let vp = engine.get_value_proposition("premium").unwrap();
        assert_eq!(vp.headline, "Premium benefits");
    }

    #[test]
    fn test_generate_script() {
        let mut engine = PersuasionEngine::new();
        engine.handlers.insert(
            ("test".to_string(), Language::English),
            ObjectionResponse::new("test", "Ack", "Reframe", "Evidence", "CTA"),
        );

        let script = engine.generate_script("test", Language::English, None);
        assert_eq!(script.objection_id, "test");
        assert_eq!(script.opening, "Ack");
        assert!(script.body.contains(&"Reframe".to_string()));
    }
}
