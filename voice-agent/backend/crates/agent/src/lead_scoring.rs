//! Lead Scoring Module for Voice Agent
//!
//! Implements predictive lead scoring based on research best practices:
//! - Signal-based scoring (urgency, price sensitivity, trust, engagement)
//! - MQL (Marketing Qualified Lead) vs SQL (Sales Qualified Lead) classification
//! - Conversion probability estimation
//! - Auto-escalation triggers
//!
//! # P20 FIX: Config-Driven Classification
//!
//! Lead classification rules now come from `lead_scoring.yaml` config:
//! - SQL/MQL criteria are defined in config, not hardcoded
//! - Signal weights are config-driven
//! - Escalation triggers are config-driven
//!
//! # Research References
//! - Section 10: Sales Conversion & Customer Support from SMALL_MODEL_AGENT_RESEARCH.md
//! - Lead scoring models (MQL → SQL → Opportunity)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lead qualification level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadQualification {
    /// Cold lead - just exploring, low intent
    Cold,
    /// Warm lead - showing interest, gathering information
    Warm,
    /// Hot lead - high intent, ready to act
    Hot,
    /// Qualified lead - all criteria met, ready for conversion
    Qualified,
}

impl LeadQualification {
    /// Get the minimum score threshold for this qualification level
    ///
    /// **DEPRECATED**: Use config-driven thresholds via `from_score_with_config()`.
    pub fn min_score(&self) -> u32 {
        match self {
            LeadQualification::Cold => 0,
            LeadQualification::Warm => 30,
            LeadQualification::Hot => 60,
            LeadQualification::Qualified => 80,
        }
    }

    /// Create from score using default thresholds
    ///
    /// **DEPRECATED**: Use `from_score_with_config()` for config-driven thresholds.
    pub fn from_score(score: u32) -> Self {
        match score {
            0..=29 => LeadQualification::Cold,
            30..=59 => LeadQualification::Warm,
            60..=79 => LeadQualification::Hot,
            _ => LeadQualification::Qualified,
        }
    }

    /// P21 FIX: Create from score using config-driven thresholds
    ///
    /// Thresholds come from lead_scoring.yaml qualification_thresholds section.
    pub fn from_score_with_config(score: u32, config: &voice_agent_config::ScoringConfig) -> Self {
        let thresholds = &config.qualification_thresholds;
        if score >= thresholds.qualified {
            LeadQualification::Qualified
        } else if score >= thresholds.hot {
            LeadQualification::Hot
        } else if score >= thresholds.warm {
            LeadQualification::Warm
        } else {
            LeadQualification::Cold
        }
    }
}

/// Lead classification (MQL vs SQL)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadClassification {
    /// Not yet qualified
    Unqualified,
    /// Marketing Qualified Lead - meets marketing criteria
    MQL,
    /// Sales Qualified Lead - meets sales criteria, ready for conversion
    SQL,
}

impl LeadClassification {
    /// Determine classification from signals using config-driven rules
    ///
    /// P20 FIX: This is the preferred method - uses rules from lead_scoring.yaml
    ///
    /// # Arguments
    /// * `signals` - The lead signals to evaluate
    /// * `config` - Classification config from lead_scoring.yaml
    ///
    /// # Example Config
    /// ```yaml
    /// classification:
    ///   sql:
    ///     required_flags:
    ///       - has_urgency_signal
    ///       - provided_contact_info
    ///       - has_specific_requirements
    ///   mql:
    ///     thresholds:
    ///       engagement_turns: 3
    ///     any_of:
    ///       - asked_about_rates
    ///       - asked_for_comparison
    /// ```
    pub fn from_signals_with_config(
        signals: &LeadSignals,
        config: &ClassificationConfig,
    ) -> Self {
        // Check SQL criteria from config
        if config.sql_criteria_met(signals) {
            return LeadClassification::SQL;
        }

        // Check MQL criteria from config
        if config.mql_criteria_met(signals) {
            return LeadClassification::MQL;
        }

        LeadClassification::Unqualified
    }
    // P21 FIX: Removed deprecated from_signals() method
    // Use from_signals_with_config() for config-driven classification
}

/// P20 FIX: Config-driven classification rules
///
/// Loaded from lead_scoring.yaml classification section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassificationConfig {
    /// SQL classification criteria
    #[serde(default)]
    pub sql: SqlCriteria,
    /// MQL classification criteria
    #[serde(default)]
    pub mql: MqlCriteria,
}

/// SQL (Sales Qualified Lead) criteria
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SqlCriteria {
    /// Required flags - ALL must be true for SQL classification
    #[serde(default)]
    pub required_flags: Vec<String>,
}

/// MQL (Marketing Qualified Lead) criteria
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MqlCriteria {
    /// Thresholds - all must be met
    #[serde(default)]
    pub thresholds: HashMap<String, u32>,
    /// Any-of flags - at least one must be true
    #[serde(default)]
    pub any_of: Vec<String>,
}

impl ClassificationConfig {
    /// Check if SQL criteria are met
    pub fn sql_criteria_met(&self, signals: &LeadSignals) -> bool {
        // All required flags must be true
        self.sql.required_flags.iter().all(|flag| {
            signals.get_flag_value(flag)
        })
    }

    /// Check if MQL criteria are met
    pub fn mql_criteria_met(&self, signals: &LeadSignals) -> bool {
        // All thresholds must be met
        let thresholds_met = self.mql.thresholds.iter().all(|(field, threshold)| {
            signals.get_numeric_value(field) >= *threshold
        });

        // At least one any_of flag must be true (if any_of is specified)
        let any_of_met = self.mql.any_of.is_empty() ||
            self.mql.any_of.iter().any(|flag| signals.get_flag_value(flag));

        thresholds_met && any_of_met
    }
}

impl LeadSignals {
    /// Get boolean flag value by name (for config-driven classification)
    pub fn get_flag_value(&self, flag_name: &str) -> bool {
        match flag_name {
            "has_urgency_signal" => self.has_urgency_signal,
            "provided_contact_info" => self.provided_contact_info,
            "has_specific_requirements" => self.has_specific_requirements,
            "asked_about_rates" => self.asked_about_rates,
            "asked_for_comparison" => self.asked_for_comparison,
            "mentioned_other_banks" => self.mentioned_other_banks,
            "provided_asset_details" => self.provided_asset_details,
            "provided_loan_amount" => self.provided_loan_amount,
            "expressed_intent_to_proceed" => self.expressed_intent_to_proceed,
            "requested_callback" => self.requested_callback,
            "requested_branch_visit" => self.requested_branch_visit,
            "requested_human_agent" => self.requested_human_agent,
            "expressed_disinterest" => self.expressed_disinterest,
            "has_existing_relationship" => self.has_existing_relationship,
            "price_sensitive" => self.price_sensitive,
            "objected_to_price" => self.objected_to_price,
            "mentioned_competitor_preference" => self.mentioned_competitor_preference,
            _ => {
                tracing::warn!(flag = %flag_name, "Unknown flag in classification config");
                false
            }
        }
    }

    /// Get numeric value by name (for config-driven thresholds)
    pub fn get_numeric_value(&self, field_name: &str) -> u32 {
        match field_name {
            "engagement_turns" => self.engagement_turns,
            "questions_asked" => self.questions_asked,
            "objections_raised" => self.objections_raised,
            "objections_resolved" => self.objections_resolved,
            "urgency_keywords_count" => self.urgency_keywords_count,
            "conversation_stalled_turns" => self.conversation_stalled_turns,
            _ => {
                tracing::warn!(field = %field_name, "Unknown numeric field in classification config");
                0
            }
        }
    }
}

/// Signals collected during conversation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LeadSignals {
    // Urgency indicators
    pub has_urgency_signal: bool,
    pub urgency_keywords_count: u32,

    // Price sensitivity
    pub price_sensitive: bool,
    pub asked_about_rates: bool,
    pub asked_for_comparison: bool,
    pub objected_to_price: bool,

    // Trust indicators
    pub trust_level: TrustLevel,
    pub has_existing_relationship: bool,
    pub mentioned_other_banks: bool,

    // Engagement metrics
    pub engagement_turns: u32,
    pub questions_asked: u32,
    pub objections_raised: u32,
    pub objections_resolved: u32,

    // Information provided
    pub provided_contact_info: bool,
    pub provided_asset_details: bool,
    pub provided_loan_amount: bool,
    pub has_specific_requirements: bool,

    // Intent signals
    pub expressed_intent_to_proceed: bool,
    pub requested_callback: bool,
    pub requested_branch_visit: bool,
    pub requested_human_agent: bool,

    // Negative signals
    pub expressed_disinterest: bool,
    pub mentioned_competitor_preference: bool,
    pub conversation_stalled_turns: u32,
}

/// Trust level indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TrustLevel {
    #[default]
    Unknown,
    Low,
    Medium,
    High,
}

impl TrustLevel {
    /// Get trust score using default hardcoded values
    ///
    /// **DEPRECATED**: Use `score_with_config()` for config-driven values.
    pub fn score(&self) -> u32 {
        match self {
            TrustLevel::Unknown => 0,
            TrustLevel::Low => 5,
            TrustLevel::Medium => 10,
            TrustLevel::High => 15,
        }
    }

    /// P21 FIX: Get trust score using config-driven values
    pub fn score_with_config(&self, config: &voice_agent_config::ScoringConfig) -> u32 {
        config.trust_score(self.as_str())
    }

    /// Get string representation for config lookup
    pub fn as_str(&self) -> &'static str {
        match self {
            TrustLevel::Unknown => "unknown",
            TrustLevel::Low => "low",
            TrustLevel::Medium => "medium",
            TrustLevel::High => "high",
        }
    }
}

/// Auto-escalation trigger reasons
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EscalationTrigger {
    /// Too many objections detected
    ExcessiveObjections { count: u32, threshold: u32 },
    /// Conversation stalled without progress
    ConversationStalled { turns: u32, threshold: u32 },
    /// High-value loan amount requires human attention
    HighValueLoan { amount: f64, threshold: f64 },
    /// Customer explicitly frustrated
    CustomerFrustration,
    /// Customer requested human agent
    CustomerRequested,
    /// Complex query beyond AI capability
    ComplexQuery,
    /// Compliance-sensitive topic
    ComplianceSensitive,
}

/// Lead score with breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadScore {
    /// Total score (0-100)
    pub total: u32,
    /// Qualification level
    pub qualification: LeadQualification,
    /// MQL/SQL classification
    pub classification: LeadClassification,
    /// Estimated conversion probability (0.0-1.0)
    pub conversion_probability: f32,
    /// Score breakdown by category
    pub breakdown: ScoreBreakdown,
    /// Active escalation triggers
    pub escalation_triggers: Vec<EscalationTrigger>,
    /// Recommendation for next action
    pub recommendation: LeadRecommendation,
}

/// Score breakdown by category
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    /// Urgency score (0-25)
    pub urgency: u32,
    /// Engagement score (0-25)
    pub engagement: u32,
    /// Information completeness score (0-25)
    pub information: u32,
    /// Intent strength score (0-25)
    pub intent: u32,
    /// Penalty from negative signals
    pub penalty: i32,
}

/// Recommended next action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadRecommendation {
    /// Continue AI conversation
    ContinueConversation,
    /// Push for appointment scheduling
    PushForAppointment,
    /// Offer callback from human agent
    OfferCallback,
    /// Escalate to human immediately
    EscalateNow { reason: String },
    /// Send follow-up SMS/email
    SendFollowUp,
    /// Mark as low priority
    LowPriority,
}

/// Lead Scoring Engine
pub struct LeadScoringEngine {
    /// Configuration
    config: LeadScoringConfig,
    /// Current signals
    signals: LeadSignals,
    /// Score history (for trend analysis)
    score_history: Vec<u32>,
    /// P20 FIX: Optional config-driven classifier
    /// When set, uses config-driven MQL/SQL classification instead of hardcoded rules
    classifier: Option<std::sync::Arc<dyn voice_agent_core::traits::LeadClassifier>>,
    /// P20 FIX: Optional classification config for config-driven rules
    classification_config: Option<ClassificationConfig>,
    /// P20 FIX: Optional scoring config for config-driven scoring values
    /// When set, uses urgency keywords, signal weights, etc. from config
    scoring_config: Option<std::sync::Arc<voice_agent_config::ScoringConfig>>,
}

/// Configuration for lead scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadScoringConfig {
    /// Objection threshold for auto-escalation
    pub max_objections_before_escalate: u32,
    /// Stalled turns threshold for auto-escalation
    pub max_stalled_turns: u32,
    /// High-value loan threshold (in rupees)
    pub high_value_loan_threshold: f64,
    /// Weight multipliers for score categories
    pub weights: ScoreWeights,
}

/// Weights for different score categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreWeights {
    pub urgency: f32,
    pub engagement: f32,
    pub information: f32,
    pub intent: f32,
}

impl Default for LeadScoringConfig {
    fn default() -> Self {
        Self {
            max_objections_before_escalate: 3,
            max_stalled_turns: 5,
            high_value_loan_threshold: 1_000_000.0, // 10 lakh
            weights: ScoreWeights {
                urgency: 1.0,
                engagement: 1.0,
                information: 1.0,
                intent: 1.0,
            },
        }
    }
}

impl LeadScoringEngine {
    /// Create a new lead scoring engine
    pub fn new() -> Self {
        Self {
            config: LeadScoringConfig::default(),
            signals: LeadSignals::default(),
            score_history: Vec::new(),
            classifier: None,
            classification_config: None,
            scoring_config: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: LeadScoringConfig) -> Self {
        Self {
            config,
            signals: LeadSignals::default(),
            score_history: Vec::new(),
            classifier: None,
            classification_config: None,
            scoring_config: None,
        }
    }

    /// P20 FIX: Create with config-driven classifier
    ///
    /// Uses the LeadClassifier trait for MQL/SQL classification instead of hardcoded rules.
    /// Get the classifier from `AgentDomainView::lead_classifier()`.
    ///
    /// ```ignore
    /// let classifier = domain_view.lead_classifier();
    /// let engine = LeadScoringEngine::with_classifier(classifier);
    /// ```
    pub fn with_classifier(
        classifier: std::sync::Arc<dyn voice_agent_core::traits::LeadClassifier>,
    ) -> Self {
        Self {
            config: LeadScoringConfig::default(),
            signals: LeadSignals::default(),
            score_history: Vec::new(),
            classifier: Some(classifier),
            classification_config: None,
            scoring_config: None,
        }
    }

    /// P20 FIX: Create with classification config
    ///
    /// Uses ClassificationConfig for MQL/SQL rules from lead_scoring.yaml.
    pub fn with_classification_config(classification_config: ClassificationConfig) -> Self {
        Self {
            config: LeadScoringConfig::default(),
            signals: LeadSignals::default(),
            score_history: Vec::new(),
            classifier: None,
            classification_config: Some(classification_config),
            scoring_config: None,
        }
    }

    /// P20 FIX: Create with full scoring config from domain config
    ///
    /// Uses ScoringConfig for urgency keywords, signal weights, etc.
    pub fn with_scoring_config(
        scoring_config: std::sync::Arc<voice_agent_config::ScoringConfig>,
    ) -> Self {
        Self {
            config: LeadScoringConfig::default(),
            signals: LeadSignals::default(),
            score_history: Vec::new(),
            classifier: None,
            classification_config: None,
            scoring_config: Some(scoring_config),
        }
    }

    /// P20 FIX: Set the classifier after construction
    pub fn set_classifier(
        &mut self,
        classifier: std::sync::Arc<dyn voice_agent_core::traits::LeadClassifier>,
    ) {
        self.classifier = Some(classifier);
    }

    /// P20 FIX: Set the classification config after construction
    pub fn set_classification_config(&mut self, config: ClassificationConfig) {
        self.classification_config = Some(config);
    }

    /// P20 FIX: Set the scoring config after construction
    pub fn set_scoring_config(
        &mut self,
        scoring_config: std::sync::Arc<voice_agent_config::ScoringConfig>,
    ) {
        self.scoring_config = Some(scoring_config);
    }

    /// Get current signals (read-only)
    pub fn signals(&self) -> &LeadSignals {
        &self.signals
    }

    /// Get mutable signals for updates
    pub fn signals_mut(&mut self) -> &mut LeadSignals {
        &mut self.signals
    }

    /// Update signals from detected intent
    ///
    /// P20 FIX: Uses config-driven intent mappings when scoring_config is set.
    /// Falls back to hardcoded mappings if no config is available.
    pub fn update_from_intent(&mut self, intent: &str, slots: &HashMap<String, String>) {
        // P20 FIX: Try config-driven intent mapping first
        if let Some(scoring_config) = self.scoring_config.clone() {
            if let Some(mapping) = scoring_config.get_intent_signals(intent) {
                // Apply signals from config
                for signal in &mapping.signals {
                    self.apply_signal(signal);
                }
                // Check slot conditions from config
                for slot_check in &mapping.slot_checks {
                    if slot_check.slots.iter().any(|s| slots.contains_key(s)) {
                        for signal in &slot_check.signals {
                            self.apply_signal(signal);
                        }
                    }
                }
                // Also apply slot-to-signal mappings
                self.apply_slot_signals_from_config(&scoring_config, slots);
                return;
            }
            // Intent not in config - apply default engagement and check slots
            self.signals.engagement_turns += 1;
            self.apply_slot_signals_from_config(&scoring_config, slots);
            return;
        }

        // P21 FIX: Minimal domain-agnostic fallback for when no scoring_config is set
        // This fallback uses only generic intent categories that apply to ANY domain
        // All domain-specific intent handling should be in lead_scoring.yaml intent_signal_mappings
        match intent {
            // Domain-agnostic: scheduling intents
            "schedule_visit" | "schedule_callback" | "schedule_appointment" => {
                self.signals.expressed_intent_to_proceed = true;
                self.signals.requested_branch_visit = intent == "schedule_visit";
                self.signals.requested_callback = intent == "schedule_callback";
            }
            // Domain-agnostic: objection handling
            "objection" => {
                self.signals.objections_raised += 1;
            }
            // Domain-agnostic: agreement/resolution
            "affirmative" | "agreement" | "confirmation" => {
                if self.signals.objections_raised > self.signals.objections_resolved {
                    self.signals.objections_resolved += 1;
                }
            }
            // Domain-agnostic: negative responses
            "negative" | "rejection" | "decline" => {
                self.signals.expressed_disinterest = true;
            }
            // Domain-agnostic: escalation request
            "escalate" | "human_agent" | "speak_to_human" => {
                self.signals.requested_human_agent = true;
            }
            // Domain-agnostic: questions indicate engagement
            "question" | "clarification" => {
                self.signals.questions_asked += 1;
                self.signals.engagement_turns += 1;
            }
            // All other intents: count as engagement only
            _ => {
                self.signals.engagement_turns += 1;
            }
        }

        // P21 FIX: Domain-agnostic slot signal detection
        // Uses canonical slot names that should be defined in any domain's slots.yaml
        // Contact info slots (universal across all domains)
        if slots.contains_key("phone_number") || slots.contains_key("customer_name")
            || slots.contains_key("phone") || slots.contains_key("email") {
            self.signals.provided_contact_info = true;
        }

        // Asset/collateral slots (using generic canonical names from slots.yaml)
        // Domain configs should map their specific names to these canonical names via slot_aliases
        if slots.contains_key("asset_quantity") || slots.contains_key("asset_quality")
            || slots.contains_key("collateral_weight") || slots.contains_key("collateral_variant")
            || slots.contains_key("collateral_value") {
            self.signals.provided_asset_details = true;
        }

        // Offer/amount slots (universal across financial domains)
        if slots.contains_key("offer_amount") || slots.contains_key("requested_amount") {
            self.signals.provided_loan_amount = true;
            self.signals.has_specific_requirements = true;
        }
    }

    /// P20 FIX: Apply a signal by name from config
    fn apply_signal(&mut self, signal: &str) {
        match signal {
            // Increment signals
            "increment_engagement_turns" => self.signals.engagement_turns += 1,
            "increment_questions_asked" => self.signals.questions_asked += 1,
            "increment_objections_raised" => self.signals.objections_raised += 1,
            "resolve_objection_if_pending" => {
                if self.signals.objections_raised > self.signals.objections_resolved {
                    self.signals.objections_resolved += 1;
                }
            }
            // Boolean signals
            "has_urgency_signal" => self.signals.has_urgency_signal = true,
            "asked_about_rates" => self.signals.asked_about_rates = true,
            "asked_for_comparison" => self.signals.asked_for_comparison = true,
            "mentioned_other_banks" => self.signals.mentioned_other_banks = true,
            "provided_contact_info" => self.signals.provided_contact_info = true,
            "provided_asset_details" => self.signals.provided_asset_details = true,
            "provided_loan_amount" => self.signals.provided_loan_amount = true,
            "has_specific_requirements" => self.signals.has_specific_requirements = true,
            "expressed_intent_to_proceed" => self.signals.expressed_intent_to_proceed = true,
            "requested_callback" => self.signals.requested_callback = true,
            "requested_branch_visit" => self.signals.requested_branch_visit = true,
            "requested_human_agent" => self.signals.requested_human_agent = true,
            "expressed_disinterest" => self.signals.expressed_disinterest = true,
            _ => {
                tracing::debug!(signal = %signal, "Unknown signal in config, ignoring");
            }
        }
    }

    /// P20 FIX: Apply slot-to-signal mappings from config
    fn apply_slot_signals_from_config(
        &mut self,
        scoring_config: &std::sync::Arc<voice_agent_config::ScoringConfig>,
        slots: &HashMap<String, String>,
    ) {
        for slot_name in slots.keys() {
            if let Some(signal) = scoring_config.get_slot_signal(slot_name) {
                self.apply_signal(signal);
            }
        }
    }

    /// Update urgency signal from text analysis
    ///
    /// P20 FIX: Uses config-driven urgency keywords when scoring_config is set.
    /// Falls back to hardcoded keywords if no config is available.
    pub fn update_urgency(&mut self, text: &str) {
        let text_lower = text.to_lowercase();

        // P20 FIX: Use config-driven urgency keywords when available
        if let Some(ref scoring_config) = self.scoring_config {
            // Get all urgency keywords from config (all languages)
            let keywords = scoring_config.all_urgency_keywords();
            for keyword in keywords {
                if text_lower.contains(keyword) {
                    self.signals.has_urgency_signal = true;
                    self.signals.urgency_keywords_count += 1;
                }
            }
            return;
        }

        // Fallback: hardcoded keywords (deprecated - prefer config)
        let urgency_keywords = [
            "urgent",
            "urgently",
            "immediately",
            "today",
            "now",
            "asap",
            "emergency",
            "jaldi",
            "abhi",
            "turant",
            "aaj",
            "foran",
        ];

        for keyword in &urgency_keywords {
            if text_lower.contains(keyword) {
                self.signals.has_urgency_signal = true;
                self.signals.urgency_keywords_count += 1;
            }
        }
    }

    /// Update trust level based on conversation
    pub fn update_trust(&mut self, positive_signal: bool) {
        self.signals.trust_level = match (self.signals.trust_level, positive_signal) {
            (TrustLevel::Unknown, true) => TrustLevel::Low,
            (TrustLevel::Unknown, false) => TrustLevel::Unknown,
            (TrustLevel::Low, true) => TrustLevel::Medium,
            (TrustLevel::Low, false) => TrustLevel::Low,
            (TrustLevel::Medium, true) => TrustLevel::High,
            (TrustLevel::Medium, false) => TrustLevel::Low,
            (TrustLevel::High, true) => TrustLevel::High,
            (TrustLevel::High, false) => TrustLevel::Medium,
        };
    }

    /// Mark conversation as stalled (no meaningful progress)
    pub fn mark_stalled(&mut self) {
        self.signals.conversation_stalled_turns += 1;
    }

    /// Reset stall counter (progress made)
    pub fn reset_stall(&mut self) {
        self.signals.conversation_stalled_turns = 0;
    }

    /// Calculate current lead score
    ///
    /// P21 FIX: Uses config-driven scoring values when `scoring_config` is set.
    pub fn calculate_score(&mut self) -> LeadScore {
        let breakdown = self.calculate_breakdown();
        let total = self.calculate_total(&breakdown);

        // Track score history
        self.score_history.push(total);

        // P21 FIX: Use config-driven qualification thresholds when available
        let qualification = if let Some(ref config) = self.scoring_config {
            LeadQualification::from_score_with_config(total, config)
        } else {
            LeadQualification::from_score(total)
        };

        // P20 FIX: Use config-driven classification when available
        let classification = self.classify_lead();

        let conversion_probability = self.estimate_conversion_probability(total, &classification);
        let escalation_triggers = self.check_escalation_triggers();
        let recommendation = self.generate_recommendation(&qualification, &escalation_triggers);

        LeadScore {
            total,
            qualification,
            classification,
            conversion_probability,
            breakdown,
            escalation_triggers,
            recommendation,
        }
    }

    /// P20 FIX: Classify lead using config-driven rules when available
    ///
    /// Priority:
    /// 1. LeadClassifier trait (if set via with_classifier())
    /// 2. ClassificationConfig (if set via with_classification_config())
    /// 3. Legacy hardcoded rules (fallback)
    fn classify_lead(&self) -> LeadClassification {
        // Priority 1: Use trait-based classifier if available
        if let Some(ref classifier) = self.classifier {
            // Create a simple signals adapter for the trait
            let signals_impl = SimpleLeadSignalsAdapter::from(&self.signals);
            let class = classifier.classify(&signals_impl);
            return match class {
                voice_agent_core::traits::LeadClass::SQL => LeadClassification::SQL,
                voice_agent_core::traits::LeadClass::MQL => LeadClassification::MQL,
                voice_agent_core::traits::LeadClass::Unqualified => LeadClassification::Unqualified,
            };
        }

        // Priority 2: Use classification config if available
        if let Some(ref config) = self.classification_config {
            return LeadClassification::from_signals_with_config(&self.signals, config);
        }

        // P21 FIX: No fallback to deprecated hardcoded rules - config is required
        tracing::warn!(
            "No classification config provided - using Unqualified as default. \
             Configure lead_scoring.yaml with classification rules for production use."
        );
        LeadClassification::Unqualified
    }

    /// Calculate score breakdown
    ///
    /// P21 FIX: Uses config-driven scoring values when `scoring_config` is set.
    /// Falls back to hardcoded defaults if no config is available.
    fn calculate_breakdown(&self) -> ScoreBreakdown {
        let signals = &self.signals;

        // P21 FIX: Use config values if available, otherwise use defaults
        if let Some(ref scoring_config) = self.scoring_config {
            return self.calculate_breakdown_from_config(signals, scoring_config);
        }

        // Fallback: hardcoded values (deprecated - prefer config)
        // Urgency score (0-25)
        let urgency = {
            let mut score = 0u32;
            if signals.has_urgency_signal {
                score += 10;
            }
            score += signals.urgency_keywords_count.min(3) * 5;
            score.min(25)
        };

        // Engagement score (0-25)
        let engagement = {
            let mut score = 0u32;
            score += signals.engagement_turns.min(5) * 3;
            score += signals.questions_asked.min(3) * 2;
            if signals.asked_about_rates {
                score += 3;
            }
            if signals.asked_for_comparison {
                score += 3;
            }
            score.min(25)
        };

        // Information completeness score (0-25)
        let information = {
            let mut score = 0u32;
            if signals.provided_contact_info {
                score += 8;
            }
            if signals.provided_asset_details {
                score += 8;
            }
            if signals.provided_loan_amount {
                score += 5;
            }
            if signals.has_specific_requirements {
                score += 4;
            }
            score.min(25)
        };

        // Intent strength score (0-25)
        let intent = {
            let mut score = 0u32;
            if signals.expressed_intent_to_proceed {
                score += 15;
            }
            if signals.requested_callback {
                score += 5;
            }
            if signals.requested_branch_visit {
                score += 8;
            }
            score += signals.trust_level.score();
            score.min(25)
        };

        // Penalty from negative signals
        let penalty = {
            let mut p = 0i32;
            if signals.expressed_disinterest {
                p -= 15;
            }
            if signals.mentioned_competitor_preference {
                p -= 10;
            }
            if signals.requested_human_agent {
                p -= 5; // Slight penalty, might indicate frustration
            }
            // Unresolved objections
            let unresolved = signals.objections_raised.saturating_sub(signals.objections_resolved);
            p -= (unresolved * 3) as i32;
            p
        };

        ScoreBreakdown {
            urgency,
            engagement,
            information,
            intent,
            penalty,
        }
    }

    /// P21 FIX: Calculate score breakdown using config-driven values
    ///
    /// All scoring weights, thresholds, and penalties come from lead_scoring.yaml.
    fn calculate_breakdown_from_config(
        &self,
        signals: &LeadSignals,
        config: &std::sync::Arc<voice_agent_config::ScoringConfig>,
    ) -> ScoreBreakdown {
        let urgency_cfg = &config.urgency;
        let engagement_cfg = &config.engagement;
        let info_cfg = &config.information;
        let intent_cfg = &config.intent;
        let penalty_cfg = &config.penalties;

        // Urgency score (0-max_score from config)
        let urgency = {
            let mut score = 0u32;
            if signals.has_urgency_signal {
                score += urgency_cfg.has_signal_score;
            }
            score += signals.urgency_keywords_count.min(urgency_cfg.max_keywords) * urgency_cfg.per_keyword_score;
            score.min(urgency_cfg.max_score)
        };

        // Engagement score (0-max_score from config)
        let engagement = {
            let mut score = 0u32;
            score += signals.engagement_turns.min(engagement_cfg.max_turns) * engagement_cfg.per_turn_score;
            score += signals.questions_asked.min(engagement_cfg.max_questions) * engagement_cfg.per_question_score;
            if signals.asked_about_rates {
                score += engagement_cfg.rates_inquiry_score;
            }
            if signals.asked_for_comparison {
                score += engagement_cfg.comparison_score;
            }
            score.min(engagement_cfg.max_score)
        };

        // Information completeness score (0-max_score from config)
        let information = {
            let mut score = 0u32;
            if signals.provided_contact_info {
                score += info_cfg.contact_info_score;
            }
            if signals.provided_asset_details {
                score += info_cfg.asset_details_score;
            }
            if signals.provided_loan_amount {
                score += info_cfg.loan_amount_score;
            }
            if signals.has_specific_requirements {
                score += info_cfg.specific_requirements_score;
            }
            score.min(info_cfg.max_score)
        };

        // Intent strength score (0-max_score from config)
        let intent = {
            let mut score = 0u32;
            if signals.expressed_intent_to_proceed {
                score += intent_cfg.intent_to_proceed_score;
            }
            if signals.requested_callback {
                score += intent_cfg.callback_request_score;
            }
            if signals.requested_branch_visit {
                score += intent_cfg.branch_visit_score;
            }
            // Use config-driven trust score
            score += signals.trust_level.score_with_config(config);
            score.min(intent_cfg.max_score)
        };

        // Penalty from negative signals (config-driven)
        let penalty = {
            let mut p = 0i32;
            if signals.expressed_disinterest {
                p += penalty_cfg.disinterest; // Already negative in config
            }
            if signals.mentioned_competitor_preference {
                p += penalty_cfg.competitor_preference; // Already negative in config
            }
            if signals.requested_human_agent {
                p += penalty_cfg.human_request; // Already negative in config
            }
            // Unresolved objections
            let unresolved = signals.objections_raised.saturating_sub(signals.objections_resolved);
            p += (unresolved as i32) * penalty_cfg.per_unresolved_objection; // Already negative in config
            p
        };

        ScoreBreakdown {
            urgency,
            engagement,
            information,
            intent,
            penalty,
        }
    }

    /// Calculate total score from breakdown
    fn calculate_total(&self, breakdown: &ScoreBreakdown) -> u32 {
        let weights = &self.config.weights;

        let weighted_sum = (breakdown.urgency as f32 * weights.urgency)
            + (breakdown.engagement as f32 * weights.engagement)
            + (breakdown.information as f32 * weights.information)
            + (breakdown.intent as f32 * weights.intent);

        let total_with_penalty = weighted_sum as i32 + breakdown.penalty;
        total_with_penalty.max(0).min(100) as u32
    }

    /// Estimate conversion probability
    ///
    /// P21 FIX: Uses config-driven multipliers when `scoring_config` is set.
    /// Falls back to hardcoded values if no config is available.
    fn estimate_conversion_probability(&self, score: u32, classification: &LeadClassification) -> f32 {
        // Base probability from score
        let base = (score as f32) / 100.0;

        // P21 FIX: Use config values if available
        if let Some(ref scoring_config) = self.scoring_config {
            let conv = &scoring_config.conversion_multipliers;

            // Adjust based on classification (config-driven)
            let classification_multiplier = match classification {
                LeadClassification::Unqualified => conv.unqualified,
                LeadClassification::MQL => conv.mql,
                LeadClassification::SQL => conv.sql,
            };

            // Adjust for positive/negative signals (config-driven)
            let signal_adjustment = if self.signals.expressed_intent_to_proceed {
                conv.intent_to_proceed
            } else if self.signals.expressed_disinterest {
                conv.disinterest
            } else {
                1.0
            };

            return (base * classification_multiplier * signal_adjustment).min(conv.max_probability);
        }

        // Fallback: hardcoded values (deprecated - prefer config)
        // Adjust based on classification
        let classification_multiplier = match classification {
            LeadClassification::Unqualified => 0.5,
            LeadClassification::MQL => 0.8,
            LeadClassification::SQL => 1.2,
        };

        // Adjust for positive/negative signals
        let signal_adjustment = if self.signals.expressed_intent_to_proceed {
            1.2
        } else if self.signals.expressed_disinterest {
            0.3
        } else {
            1.0
        };

        (base * classification_multiplier * signal_adjustment).min(0.95)
    }

    /// Check for auto-escalation triggers
    ///
    /// P21 FIX: Uses config-driven thresholds from scoring_config.escalation when available,
    /// falling back to internal config defaults only if scoring_config is not set.
    fn check_escalation_triggers(&self) -> Vec<EscalationTrigger> {
        let mut triggers = Vec::new();
        let signals = &self.signals;

        // P21 FIX: Get thresholds from scoring_config if available, otherwise use internal defaults
        let (max_objections, max_stalled) = if let Some(scoring_config) = &self.scoring_config {
            (
                scoring_config.escalation.max_objections,
                scoring_config.escalation.max_stalled_turns,
            )
        } else {
            (
                self.config.max_objections_before_escalate,
                self.config.max_stalled_turns,
            )
        };

        // Check objection threshold
        let unresolved_objections = signals.objections_raised.saturating_sub(signals.objections_resolved);
        if unresolved_objections >= max_objections {
            triggers.push(EscalationTrigger::ExcessiveObjections {
                count: unresolved_objections,
                threshold: max_objections,
            });
        }

        // Check stalled conversation
        if signals.conversation_stalled_turns >= max_stalled {
            triggers.push(EscalationTrigger::ConversationStalled {
                turns: signals.conversation_stalled_turns,
                threshold: max_stalled,
            });
        }

        // Check for customer request
        if signals.requested_human_agent {
            triggers.push(EscalationTrigger::CustomerRequested);
        }

        triggers
    }

    /// Check if loan amount triggers high-value escalation
    ///
    /// P21 FIX: Uses config-driven threshold from scoring_config.escalation when available.
    pub fn check_high_value_loan(&mut self, amount: f64) -> Option<EscalationTrigger> {
        // P21 FIX: Get threshold from scoring_config if available, otherwise use internal default
        let threshold = if let Some(scoring_config) = &self.scoring_config {
            scoring_config.escalation.high_value_threshold
        } else {
            self.config.high_value_loan_threshold
        };

        if amount >= threshold {
            Some(EscalationTrigger::HighValueLoan { amount, threshold })
        } else {
            None
        }
    }

    /// Generate recommendation based on current state
    fn generate_recommendation(
        &self,
        qualification: &LeadQualification,
        triggers: &[EscalationTrigger],
    ) -> LeadRecommendation {
        // Check escalation triggers first
        for trigger in triggers {
            match trigger {
                EscalationTrigger::CustomerRequested => {
                    return LeadRecommendation::EscalateNow {
                        reason: "Customer requested human agent".to_string(),
                    };
                }
                EscalationTrigger::ExcessiveObjections { count, .. } => {
                    return LeadRecommendation::EscalateNow {
                        reason: format!("{} unresolved objections", count),
                    };
                }
                EscalationTrigger::ConversationStalled { turns, .. } => {
                    return LeadRecommendation::OfferCallback;
                }
                EscalationTrigger::HighValueLoan { amount, .. } => {
                    return LeadRecommendation::EscalateNow {
                        reason: format!("High-value loan: ₹{:.0}", amount),
                    };
                }
                _ => {}
            }
        }

        // Recommendation based on qualification level
        match qualification {
            LeadQualification::Cold => {
                if self.signals.expressed_disinterest {
                    LeadRecommendation::LowPriority
                } else {
                    LeadRecommendation::ContinueConversation
                }
            }
            LeadQualification::Warm => {
                if self.signals.asked_about_rates || self.signals.asked_for_comparison {
                    LeadRecommendation::ContinueConversation
                } else {
                    LeadRecommendation::SendFollowUp
                }
            }
            LeadQualification::Hot => LeadRecommendation::PushForAppointment,
            LeadQualification::Qualified => {
                if self.signals.requested_callback || self.signals.requested_branch_visit {
                    LeadRecommendation::PushForAppointment
                } else {
                    LeadRecommendation::OfferCallback
                }
            }
        }
    }

    /// Get score trend (positive = improving, negative = declining)
    pub fn score_trend(&self) -> i32 {
        if self.score_history.len() < 2 {
            return 0;
        }

        let recent = self.score_history.iter().rev().take(3);
        let scores: Vec<_> = recent.cloned().collect();

        if scores.len() >= 2 {
            scores[0] as i32 - scores[scores.len() - 1] as i32
        } else {
            0
        }
    }

    /// Reset for new conversation
    pub fn reset(&mut self) {
        self.signals = LeadSignals::default();
        self.score_history.clear();
    }
}

impl Default for LeadScoringEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// P20 FIX: Adapter to convert LeadSignals to LeadSignalsTrait
///
/// This bridges the agent's LeadSignals struct to the core trait interface,
/// enabling config-driven classification through the LeadClassifier trait.
struct SimpleLeadSignalsAdapter {
    flags: HashMap<String, bool>,
    thresholds: HashMap<String, u32>,
}

impl SimpleLeadSignalsAdapter {
    fn from(signals: &LeadSignals) -> Self {
        let mut flags = HashMap::new();
        flags.insert("has_urgency_signal".to_string(), signals.has_urgency_signal);
        flags.insert("provided_contact_info".to_string(), signals.provided_contact_info);
        flags.insert("has_specific_requirements".to_string(), signals.has_specific_requirements);
        flags.insert("asked_about_rates".to_string(), signals.asked_about_rates);
        flags.insert("asked_for_comparison".to_string(), signals.asked_for_comparison);
        flags.insert("mentioned_other_banks".to_string(), signals.mentioned_other_banks);
        flags.insert("provided_asset_details".to_string(), signals.provided_asset_details);
        flags.insert("provided_loan_amount".to_string(), signals.provided_loan_amount);
        flags.insert("expressed_intent_to_proceed".to_string(), signals.expressed_intent_to_proceed);
        flags.insert("requested_callback".to_string(), signals.requested_callback);
        flags.insert("requested_branch_visit".to_string(), signals.requested_branch_visit);
        flags.insert("requested_human_agent".to_string(), signals.requested_human_agent);
        flags.insert("expressed_disinterest".to_string(), signals.expressed_disinterest);
        flags.insert("has_existing_relationship".to_string(), signals.has_existing_relationship);
        flags.insert("price_sensitive".to_string(), signals.price_sensitive);
        flags.insert("objected_to_price".to_string(), signals.objected_to_price);
        flags.insert("mentioned_competitor_preference".to_string(), signals.mentioned_competitor_preference);

        let mut thresholds = HashMap::new();
        thresholds.insert("engagement_turns".to_string(), signals.engagement_turns);
        thresholds.insert("questions_asked".to_string(), signals.questions_asked);
        thresholds.insert("objections_raised".to_string(), signals.objections_raised);
        thresholds.insert("objections_resolved".to_string(), signals.objections_resolved);
        thresholds.insert("urgency_keywords_count".to_string(), signals.urgency_keywords_count);
        thresholds.insert("conversation_stalled_turns".to_string(), signals.conversation_stalled_turns);

        Self { flags, thresholds }
    }
}

impl voice_agent_core::traits::LeadSignalsTrait for SimpleLeadSignalsAdapter {
    fn has_signal(&self, signal_name: &str) -> bool {
        // Check boolean flags first
        if let Some(&value) = self.flags.get(signal_name) {
            return value;
        }
        // Check numeric thresholds (> 0 means signal is active)
        if let Some(&value) = self.thresholds.get(signal_name) {
            return value > 0;
        }
        false
    }

    fn get_numeric_signal(&self, signal_name: &str) -> Option<u32> {
        self.thresholds.get(signal_name).copied()
    }

    fn active_signals(&self) -> Vec<&str> {
        let mut active = Vec::new();
        for (name, &value) in &self.flags {
            if value {
                active.push(name.as_str());
            }
        }
        for (name, &value) in &self.thresholds {
            if value > 0 {
                active.push(name.as_str());
            }
        }
        active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lead_qualification_from_score() {
        assert_eq!(LeadQualification::from_score(0), LeadQualification::Cold);
        assert_eq!(LeadQualification::from_score(29), LeadQualification::Cold);
        assert_eq!(LeadQualification::from_score(30), LeadQualification::Warm);
        assert_eq!(LeadQualification::from_score(59), LeadQualification::Warm);
        assert_eq!(LeadQualification::from_score(60), LeadQualification::Hot);
        assert_eq!(LeadQualification::from_score(79), LeadQualification::Hot);
        assert_eq!(LeadQualification::from_score(80), LeadQualification::Qualified);
        assert_eq!(LeadQualification::from_score(100), LeadQualification::Qualified);
    }

    #[test]
    fn test_lead_scoring_basic() {
        let mut engine = LeadScoringEngine::new();

        // Initial score should be low
        let score = engine.calculate_score();
        assert_eq!(score.qualification, LeadQualification::Cold);
        assert_eq!(score.classification, LeadClassification::Unqualified);
    }

    #[test]
    fn test_lead_scoring_with_engagement() {
        let mut engine = LeadScoringEngine::new();

        // Simulate strong engagement
        engine.signals_mut().engagement_turns = 5;
        engine.signals_mut().asked_about_rates = true;
        engine.signals_mut().asked_for_comparison = true;
        engine.signals_mut().provided_contact_info = true;
        engine.signals_mut().questions_asked = 3;

        let score = engine.calculate_score();
        // engagement: min(5*3 + 3*2 + 3 + 3, 25) = 25
        // information: 8 (contact info)
        // Total: 33
        assert!(score.total >= 30, "Score {} should be >= 30", score.total);
        assert_eq!(score.classification, LeadClassification::MQL);
    }

    #[test]
    fn test_lead_scoring_sql_classification() {
        let mut engine = LeadScoringEngine::new();

        // SQL criteria: urgency + contact + specific requirements + engagement
        engine.signals_mut().has_urgency_signal = true;
        engine.signals_mut().urgency_keywords_count = 2;
        engine.signals_mut().provided_contact_info = true;
        engine.signals_mut().has_specific_requirements = true;
        engine.signals_mut().provided_loan_amount = true;
        engine.signals_mut().provided_asset_details = true;
        engine.signals_mut().expressed_intent_to_proceed = true;
        engine.signals_mut().requested_branch_visit = true;
        engine.signals_mut().engagement_turns = 5;
        engine.signals_mut().trust_level = TrustLevel::High;

        let score = engine.calculate_score();
        assert_eq!(score.classification, LeadClassification::SQL);
        // urgency: 10 + 2*5 = 20
        // engagement: min(5*3, 25) = 15
        // information: 8 + 8 + 5 + 4 = 25
        // intent: 15 + 8 + 15 = 38 -> capped at 25
        // Total: 20 + 15 + 25 + 25 = 85
        assert!(score.total >= 60, "Score {} should be >= 60", score.total);
    }

    #[test]
    fn test_auto_escalation_objections() {
        let mut engine = LeadScoringEngine::new();

        // Raise objections without resolving
        engine.signals_mut().objections_raised = 4;
        engine.signals_mut().objections_resolved = 1;

        let score = engine.calculate_score();
        assert!(score.escalation_triggers.iter().any(|t| {
            matches!(t, EscalationTrigger::ExcessiveObjections { .. })
        }));
    }

    #[test]
    fn test_auto_escalation_stalled() {
        let mut engine = LeadScoringEngine::new();

        // Stalled conversation
        for _ in 0..5 {
            engine.mark_stalled();
        }

        let score = engine.calculate_score();
        assert!(score.escalation_triggers.iter().any(|t| {
            matches!(t, EscalationTrigger::ConversationStalled { .. })
        }));
    }

    #[test]
    fn test_high_value_loan_escalation() {
        let mut engine = LeadScoringEngine::new();

        // High-value loan
        let trigger = engine.check_high_value_loan(1_500_000.0);
        assert!(trigger.is_some());
        assert!(matches!(trigger, Some(EscalationTrigger::HighValueLoan { .. })));
    }

    #[test]
    fn test_urgency_detection() {
        let mut engine = LeadScoringEngine::new();

        engine.update_urgency("I need the loan urgently today");
        assert!(engine.signals().has_urgency_signal);
        assert!(engine.signals().urgency_keywords_count >= 2);
    }

    #[test]
    fn test_intent_updates() {
        let mut engine = LeadScoringEngine::new();
        let mut slots = HashMap::new();
        slots.insert("loan_amount".to_string(), "500000".to_string());

        engine.update_from_intent("loan_inquiry", &slots);
        assert!(engine.signals().provided_loan_amount);
        assert!(engine.signals().has_specific_requirements);
    }

    #[test]
    fn test_negative_signals_penalty() {
        let mut engine = LeadScoringEngine::new();

        // Add positive signals first
        engine.signals_mut().engagement_turns = 5;
        engine.signals_mut().provided_contact_info = true;
        let positive_score = engine.calculate_score().total;

        // Add negative signals
        engine.signals_mut().expressed_disinterest = true;
        engine.signals_mut().objections_raised = 2;
        let negative_score = engine.calculate_score().total;

        assert!(negative_score < positive_score);
    }

    #[test]
    fn test_score_trend() {
        let mut engine = LeadScoringEngine::new();

        // Build up engagement over multiple calculations
        engine.calculate_score(); // Low score

        engine.signals_mut().engagement_turns = 3;
        engine.calculate_score(); // Medium score

        engine.signals_mut().provided_contact_info = true;
        engine.signals_mut().expressed_intent_to_proceed = true;
        engine.calculate_score(); // Higher score

        let trend = engine.score_trend();
        assert!(trend > 0, "Score trend should be positive");
    }
}
