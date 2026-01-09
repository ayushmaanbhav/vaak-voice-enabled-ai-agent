//! Lead Scoring Strategy trait for qualification and scoring
//!
//! This module provides a domain-agnostic interface for lead scoring,
//! including category-based scoring, qualification levels, MQL/SQL classification,
//! and escalation triggers. All thresholds are loaded from configuration.
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_core::traits::LeadScoringStrategy;
//!
//! // Strategy is created from domain config
//! let scorer = config_bridge.lead_scoring();
//!
//! // Calculate score from signals
//! let breakdown = scorer.calculate_breakdown(&signals);
//! let level = scorer.qualification_level(breakdown.total());
//! ```

/// Lead qualification level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QualificationLevel {
    /// Score 0-29: Not yet interested
    Cold,
    /// Score 30-59: Shows some interest
    Warm,
    /// Score 60-79: Actively interested
    Hot,
    /// Score 80+: Ready to convert
    Qualified,
}

impl QualificationLevel {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Cold => "Cold",
            Self::Warm => "Warm",
            Self::Hot => "Hot",
            Self::Qualified => "Qualified",
        }
    }

    /// Get priority (higher = more urgent)
    pub fn priority(&self) -> u8 {
        match self {
            Self::Cold => 1,
            Self::Warm => 2,
            Self::Hot => 3,
            Self::Qualified => 4,
        }
    }
}

/// Lead classification (MQL vs SQL)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeadClassification {
    /// Unqualified lead
    Unqualified,
    /// Marketing Qualified Lead
    MQL,
    /// Sales Qualified Lead
    SQL,
}

impl LeadClassification {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Unqualified => "Unqualified",
            Self::MQL => "Marketing Qualified Lead",
            Self::SQL => "Sales Qualified Lead",
        }
    }
}

/// Escalation trigger types
#[derive(Debug, Clone)]
pub enum EscalationTrigger {
    /// Too many unresolved objections
    ExcessiveObjections {
        count: u32,
        threshold: u32,
    },
    /// Conversation not progressing
    ConversationStalled {
        turns: u32,
        threshold: u32,
    },
    /// High-value opportunity
    HighValueOpportunity {
        amount: f64,
        threshold: f64,
    },
    /// Customer showing frustration
    CustomerFrustration,
    /// Customer explicitly requested human
    CustomerRequested,
    /// Query too complex for AI
    ComplexQuery,
    /// Compliance-sensitive topic
    ComplianceSensitive,
}

impl EscalationTrigger {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ExcessiveObjections { .. } => "Excessive Objections",
            Self::ConversationStalled { .. } => "Conversation Stalled",
            Self::HighValueOpportunity { .. } => "High Value Opportunity",
            Self::CustomerFrustration => "Customer Frustration",
            Self::CustomerRequested => "Customer Requested",
            Self::ComplexQuery => "Complex Query",
            Self::ComplianceSensitive => "Compliance Sensitive",
        }
    }
}

/// Score breakdown by category
#[derive(Debug, Clone, Default)]
pub struct ScoreBreakdown {
    /// Urgency score (0-25)
    pub urgency: u32,
    /// Engagement score (0-25)
    pub engagement: u32,
    /// Information completeness score (0-25)
    pub information: u32,
    /// Intent strength score (0-25)
    pub intent: u32,
    /// Penalty points (negative)
    pub penalty: i32,
}

impl ScoreBreakdown {
    /// Calculate total score (capped at 0-100)
    pub fn total(&self) -> u32 {
        let raw = self.urgency as i32 + self.engagement as i32 + self.information as i32 + self.intent as i32 + self.penalty;
        raw.max(0).min(100) as u32
    }

    /// Check if any score is at maximum
    pub fn has_max_score(&self) -> bool {
        self.urgency >= 25 || self.engagement >= 25 || self.information >= 25 || self.intent >= 25
    }
}

/// Lead signals collected during conversation
pub trait LeadSignals: Send + Sync {
    // Urgency signals
    fn has_urgency_signal(&self) -> bool;
    fn urgency_keyword_count(&self) -> u32;

    // Engagement signals
    fn engagement_turns(&self) -> u32;
    fn questions_asked(&self) -> u32;
    fn asked_about_rates(&self) -> bool;
    fn asked_for_comparison(&self) -> bool;

    // Objection signals
    fn objections_raised(&self) -> u32;
    fn objections_resolved(&self) -> u32;
    fn unresolved_objections(&self) -> u32 {
        self.objections_raised().saturating_sub(self.objections_resolved())
    }

    // Information signals
    fn provided_contact_info(&self) -> bool;
    fn provided_asset_details(&self) -> bool;
    fn provided_loan_amount(&self) -> bool;
    fn has_specific_requirements(&self) -> bool;

    // Intent signals
    fn expressed_intent_to_proceed(&self) -> bool;
    fn requested_callback(&self) -> bool;
    fn requested_branch_visit(&self) -> bool;

    // Negative signals
    fn expressed_disinterest(&self) -> bool;
    fn mentioned_competitor_preference(&self) -> bool;
    fn requested_human(&self) -> bool;

    // Trust level
    fn trust_level(&self) -> &str;

    // Loan amount for high-value detection
    fn loan_amount(&self) -> Option<f64>;
}

/// Scoring configuration
#[derive(Debug, Clone)]
pub struct ScoringConfig {
    // Qualification thresholds
    pub cold_max: u32,
    pub warm_max: u32,
    pub hot_max: u32,
    pub qualified_min: u32,

    // Urgency weights
    pub urgency_has_signal_score: u32,
    pub urgency_keyword_score: u32,
    pub urgency_max_keywords: u32,
    pub urgency_max_score: u32,

    // Engagement weights
    pub engagement_per_turn_score: u32,
    pub engagement_max_turns: u32,
    pub engagement_per_question_score: u32,
    pub engagement_max_questions: u32,
    pub engagement_rates_inquiry_score: u32,
    pub engagement_comparison_score: u32,
    pub engagement_max_score: u32,

    // Information weights
    pub info_contact_score: u32,
    pub info_asset_details_score: u32,
    pub info_loan_amount_score: u32,
    pub info_specific_requirements_score: u32,
    pub info_max_score: u32,

    // Intent weights
    pub intent_to_proceed_score: u32,
    pub intent_callback_score: u32,
    pub intent_branch_visit_score: u32,
    pub intent_max_score: u32,

    // Penalties
    pub penalty_disinterest: i32,
    pub penalty_competitor_preference: i32,
    pub penalty_human_request: i32,
    pub penalty_per_unresolved_objection: i32,

    // Escalation thresholds
    pub escalation_max_objections: u32,
    pub escalation_max_stalled_turns: u32,
    pub escalation_high_value_threshold: f64,

    // MQL criteria
    pub mql_min_engagement_turns: u32,

    // Urgency keywords
    pub urgency_keywords_en: Vec<String>,
    pub urgency_keywords_hi: Vec<String>,
}

impl Default for ScoringConfig {
    /// PRESERVED: Exact values from lead_scoring.rs and scoring.yaml
    fn default() -> Self {
        Self {
            // Thresholds: 0-29 Cold, 30-59 Warm, 60-79 Hot, 80+ Qualified
            cold_max: 29,
            warm_max: 59,
            hot_max: 79,
            qualified_min: 80,

            // Urgency (max 25)
            urgency_has_signal_score: 10,
            urgency_keyword_score: 5,
            urgency_max_keywords: 3,
            urgency_max_score: 25,

            // Engagement (max 25)
            engagement_per_turn_score: 3,
            engagement_max_turns: 5,
            engagement_per_question_score: 2,
            engagement_max_questions: 3,
            engagement_rates_inquiry_score: 3,
            engagement_comparison_score: 3,
            engagement_max_score: 25,

            // Information (max 25)
            info_contact_score: 8,
            info_asset_details_score: 8,
            info_loan_amount_score: 5,
            info_specific_requirements_score: 4,
            info_max_score: 25,

            // Intent (max 25)
            intent_to_proceed_score: 15,
            intent_callback_score: 5,
            intent_branch_visit_score: 8,
            intent_max_score: 25,

            // Penalties
            penalty_disinterest: -15,
            penalty_competitor_preference: -10,
            penalty_human_request: -5,
            penalty_per_unresolved_objection: -3,

            // Escalation
            escalation_max_objections: 3,
            escalation_max_stalled_turns: 5,
            escalation_high_value_threshold: 1_000_000.0,

            // MQL
            mql_min_engagement_turns: 3,

            // Keywords
            urgency_keywords_en: vec![
                "urgent".to_string(),
                "urgently".to_string(),
                "immediately".to_string(),
                "today".to_string(),
                "now".to_string(),
                "asap".to_string(),
                "emergency".to_string(),
            ],
            urgency_keywords_hi: vec![
                "जल्दी".to_string(),
                "अभी".to_string(),
                "तुरंत".to_string(),
                "आज".to_string(),
                "फोरन".to_string(),
            ],
        }
    }
}

/// Lead scoring strategy trait
///
/// Calculates lead scores and classifications based on conversation signals.
pub trait LeadScoringStrategy: Send + Sync {
    /// Calculate score breakdown from signals
    fn calculate_breakdown(&self, signals: &dyn LeadSignals) -> ScoreBreakdown;

    /// Calculate total score (0-100)
    fn calculate_total(&self, signals: &dyn LeadSignals) -> u32 {
        self.calculate_breakdown(signals).total()
    }

    /// Determine qualification level from score
    fn qualification_level(&self, score: u32) -> QualificationLevel;

    /// Determine MQL/SQL classification
    fn classify(&self, signals: &dyn LeadSignals) -> LeadClassification;

    /// Estimate conversion probability (0.0 - 1.0)
    fn conversion_probability(
        &self,
        score: u32,
        classification: LeadClassification,
        signals: &dyn LeadSignals,
    ) -> f32;

    /// Check for escalation triggers
    fn check_escalation_triggers(
        &self,
        signals: &dyn LeadSignals,
    ) -> Vec<EscalationTrigger>;

    /// Get urgency keywords for language
    fn urgency_keywords(&self, language: &str) -> Vec<&str>;

    /// Get qualification thresholds (cold_max, warm_max, hot_max, qualified_min)
    fn thresholds(&self) -> (u32, u32, u32, u32);

    /// Get scoring configuration
    fn config(&self) -> &ScoringConfig;
}

/// Config-driven lead scoring implementation
pub struct ConfigLeadScoring {
    config: ScoringConfig,
}

impl ConfigLeadScoring {
    /// Create a new scorer with the given configuration
    pub fn new(config: ScoringConfig) -> Self {
        Self { config }
    }

}

impl LeadScoringStrategy for ConfigLeadScoring {
    fn calculate_breakdown(&self, signals: &dyn LeadSignals) -> ScoreBreakdown {
        let cfg = &self.config;

        // PRESERVED: Exact scoring algorithm from lead_scoring.rs:166-198

        // Urgency (max 25)
        let mut urgency = 0u32;
        if signals.has_urgency_signal() {
            urgency += cfg.urgency_has_signal_score;
        }
        let keyword_score = signals.urgency_keyword_count().min(cfg.urgency_max_keywords) * cfg.urgency_keyword_score;
        urgency = (urgency + keyword_score).min(cfg.urgency_max_score);

        // Engagement (max 25)
        let mut engagement = 0u32;
        let turn_score = signals.engagement_turns().min(cfg.engagement_max_turns) * cfg.engagement_per_turn_score;
        engagement += turn_score;
        let question_score = signals.questions_asked().min(cfg.engagement_max_questions) * cfg.engagement_per_question_score;
        engagement += question_score;
        if signals.asked_about_rates() {
            engagement += cfg.engagement_rates_inquiry_score;
        }
        if signals.asked_for_comparison() {
            engagement += cfg.engagement_comparison_score;
        }
        engagement = engagement.min(cfg.engagement_max_score);

        // Information (max 25)
        let mut information = 0u32;
        if signals.provided_contact_info() {
            information += cfg.info_contact_score;
        }
        if signals.provided_asset_details() {
            information += cfg.info_asset_details_score;
        }
        if signals.provided_loan_amount() {
            information += cfg.info_loan_amount_score;
        }
        if signals.has_specific_requirements() {
            information += cfg.info_specific_requirements_score;
        }
        information = information.min(cfg.info_max_score);

        // Intent (max 25)
        let mut intent = 0u32;
        if signals.expressed_intent_to_proceed() {
            intent += cfg.intent_to_proceed_score;
        }
        if signals.requested_callback() {
            intent += cfg.intent_callback_score;
        }
        if signals.requested_branch_visit() {
            intent += cfg.intent_branch_visit_score;
        }
        intent = intent.min(cfg.intent_max_score);

        // Penalties
        let mut penalty = 0i32;
        if signals.expressed_disinterest() {
            penalty += cfg.penalty_disinterest;
        }
        if signals.mentioned_competitor_preference() {
            penalty += cfg.penalty_competitor_preference;
        }
        if signals.requested_human() {
            penalty += cfg.penalty_human_request;
        }
        penalty += (signals.unresolved_objections() as i32) * cfg.penalty_per_unresolved_objection;

        ScoreBreakdown {
            urgency,
            engagement,
            information,
            intent,
            penalty,
        }
    }

    fn qualification_level(&self, score: u32) -> QualificationLevel {
        // PRESERVED: Exact thresholds from lead_scoring.rs:43-46
        if score <= self.config.cold_max {
            QualificationLevel::Cold
        } else if score <= self.config.warm_max {
            QualificationLevel::Warm
        } else if score <= self.config.hot_max {
            QualificationLevel::Hot
        } else {
            QualificationLevel::Qualified
        }
    }

    fn classify(&self, signals: &dyn LeadSignals) -> LeadClassification {
        // PRESERVED: Exact criteria from lead_scoring.rs:62-81

        // SQL criteria: has_urgency AND provided_contact AND has_specific_requirements
        if signals.has_urgency_signal()
            && signals.provided_contact_info()
            && signals.has_specific_requirements()
        {
            return LeadClassification::SQL;
        }

        // MQL criteria: engagement_turns >= 3 AND (asked_about_rates OR asked_for_comparison)
        if signals.engagement_turns() >= self.config.mql_min_engagement_turns
            && (signals.asked_about_rates() || signals.asked_for_comparison())
        {
            return LeadClassification::MQL;
        }

        LeadClassification::Unqualified
    }

    fn conversion_probability(
        &self,
        score: u32,
        classification: LeadClassification,
        _signals: &dyn LeadSignals,
    ) -> f32 {
        // Base probability from score
        let base = (score as f32) / 100.0;

        // Adjust by classification
        let multiplier = match classification {
            LeadClassification::SQL => 1.5,
            LeadClassification::MQL => 1.2,
            LeadClassification::Unqualified => 0.8,
        };

        (base * multiplier).min(1.0)
    }

    fn check_escalation_triggers(
        &self,
        signals: &dyn LeadSignals,
    ) -> Vec<EscalationTrigger> {
        let cfg = &self.config;
        let mut triggers = Vec::new();

        // PRESERVED: Exact escalation logic from lead_scoring.rs:147-164

        // Excessive objections
        if signals.unresolved_objections() > cfg.escalation_max_objections {
            triggers.push(EscalationTrigger::ExcessiveObjections {
                count: signals.unresolved_objections(),
                threshold: cfg.escalation_max_objections,
            });
        }

        // Conversation stalled
        if signals.engagement_turns() > cfg.escalation_max_stalled_turns
            && !signals.expressed_intent_to_proceed()
        {
            triggers.push(EscalationTrigger::ConversationStalled {
                turns: signals.engagement_turns(),
                threshold: cfg.escalation_max_stalled_turns,
            });
        }

        // High-value opportunity
        if let Some(amount) = signals.loan_amount() {
            if amount > cfg.escalation_high_value_threshold {
                triggers.push(EscalationTrigger::HighValueOpportunity {
                    amount,
                    threshold: cfg.escalation_high_value_threshold,
                });
            }
        }

        // Customer requested human
        if signals.requested_human() {
            triggers.push(EscalationTrigger::CustomerRequested);
        }

        triggers
    }

    fn urgency_keywords(&self, language: &str) -> Vec<&str> {
        match language {
            "en" => self.config.urgency_keywords_en.iter().map(|s| s.as_str()).collect(),
            "hi" => self.config.urgency_keywords_hi.iter().map(|s| s.as_str()).collect(),
            _ => self.config.urgency_keywords_en.iter().map(|s| s.as_str()).collect(),
        }
    }

    fn thresholds(&self) -> (u32, u32, u32, u32) {
        (
            self.config.cold_max,
            self.config.warm_max,
            self.config.hot_max,
            self.config.qualified_min,
        )
    }

    fn config(&self) -> &ScoringConfig {
        &self.config
    }
}

/// Simple signals implementation for testing
#[derive(Debug, Clone, Default)]
pub struct SimpleLeadSignals {
    pub has_urgency: bool,
    pub urgency_keywords: u32,
    pub turns: u32,
    pub questions: u32,
    pub rates_inquiry: bool,
    pub comparison_inquiry: bool,
    pub objections: u32,
    pub resolved_objections: u32,
    pub contact_provided: bool,
    pub asset_details: bool,
    pub amount_provided: bool,
    pub specific_requirements: bool,
    pub intent_to_proceed: bool,
    pub callback_requested: bool,
    pub visit_requested: bool,
    pub disinterest: bool,
    pub competitor_preference: bool,
    pub human_requested: bool,
    pub trust: String,
    pub amount: Option<f64>,
}

impl LeadSignals for SimpleLeadSignals {
    fn has_urgency_signal(&self) -> bool { self.has_urgency }
    fn urgency_keyword_count(&self) -> u32 { self.urgency_keywords }
    fn engagement_turns(&self) -> u32 { self.turns }
    fn questions_asked(&self) -> u32 { self.questions }
    fn asked_about_rates(&self) -> bool { self.rates_inquiry }
    fn asked_for_comparison(&self) -> bool { self.comparison_inquiry }
    fn objections_raised(&self) -> u32 { self.objections }
    fn objections_resolved(&self) -> u32 { self.resolved_objections }
    fn provided_contact_info(&self) -> bool { self.contact_provided }
    fn provided_asset_details(&self) -> bool { self.asset_details }
    fn provided_loan_amount(&self) -> bool { self.amount_provided }
    fn has_specific_requirements(&self) -> bool { self.specific_requirements }
    fn expressed_intent_to_proceed(&self) -> bool { self.intent_to_proceed }
    fn requested_callback(&self) -> bool { self.callback_requested }
    fn requested_branch_visit(&self) -> bool { self.visit_requested }
    fn expressed_disinterest(&self) -> bool { self.disinterest }
    fn mentioned_competitor_preference(&self) -> bool { self.competitor_preference }
    fn requested_human(&self) -> bool { self.human_requested }
    fn trust_level(&self) -> &str { &self.trust }
    fn loan_amount(&self) -> Option<f64> { self.amount }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create test scorer with default configuration
    fn test_scorer() -> ConfigLeadScoring {
        ConfigLeadScoring::new(ScoringConfig::default())
    }

    #[test]
    fn test_qualification_thresholds() {
        let scorer = test_scorer();

        // PRESERVED: Exact threshold boundaries
        assert_eq!(scorer.qualification_level(0), QualificationLevel::Cold);
        assert_eq!(scorer.qualification_level(29), QualificationLevel::Cold);
        assert_eq!(scorer.qualification_level(30), QualificationLevel::Warm);
        assert_eq!(scorer.qualification_level(59), QualificationLevel::Warm);
        assert_eq!(scorer.qualification_level(60), QualificationLevel::Hot);
        assert_eq!(scorer.qualification_level(79), QualificationLevel::Hot);
        assert_eq!(scorer.qualification_level(80), QualificationLevel::Qualified);
        assert_eq!(scorer.qualification_level(100), QualificationLevel::Qualified);
    }

    #[test]
    fn test_urgency_scoring() {
        let scorer = test_scorer();
        let mut signals = SimpleLeadSignals::default();

        // No urgency
        let breakdown = scorer.calculate_breakdown(&signals);
        assert_eq!(breakdown.urgency, 0);

        // Has signal: +10
        signals.has_urgency = true;
        let breakdown = scorer.calculate_breakdown(&signals);
        assert_eq!(breakdown.urgency, 10);

        // Add keywords: +5 each (max 3)
        signals.urgency_keywords = 3;
        let breakdown = scorer.calculate_breakdown(&signals);
        assert_eq!(breakdown.urgency, 25); // 10 + (3 * 5) = 25
    }

    #[test]
    fn test_engagement_scoring() {
        let scorer = test_scorer();
        let mut signals = SimpleLeadSignals::default();

        signals.turns = 5; // 5 * 3 = 15
        signals.questions = 3; // 3 * 2 = 6
        signals.rates_inquiry = true; // +3
        signals.comparison_inquiry = true; // +3

        let breakdown = scorer.calculate_breakdown(&signals);
        // Total: 15 + 6 + 3 + 3 = 27, capped at 25
        assert_eq!(breakdown.engagement, 25);
    }

    #[test]
    fn test_information_scoring() {
        let scorer = test_scorer();
        let mut signals = SimpleLeadSignals::default();

        signals.contact_provided = true; // +8
        signals.asset_details = true; // +8
        signals.amount_provided = true; // +5
        signals.specific_requirements = true; // +4

        let breakdown = scorer.calculate_breakdown(&signals);
        // Total: 8 + 8 + 5 + 4 = 25
        assert_eq!(breakdown.information, 25);
    }

    #[test]
    fn test_penalties() {
        let scorer = test_scorer();
        let mut signals = SimpleLeadSignals::default();

        signals.disinterest = true; // -15
        signals.competitor_preference = true; // -10
        signals.objections = 2; // 2 unresolved * -3 = -6

        let breakdown = scorer.calculate_breakdown(&signals);
        assert_eq!(breakdown.penalty, -31);
    }

    #[test]
    fn test_sql_classification() {
        let scorer = test_scorer();
        let mut signals = SimpleLeadSignals::default();

        signals.has_urgency = true;
        signals.contact_provided = true;
        signals.specific_requirements = true;

        assert_eq!(scorer.classify(&signals), LeadClassification::SQL);
    }

    #[test]
    fn test_mql_classification() {
        let scorer = test_scorer();
        let mut signals = SimpleLeadSignals::default();

        signals.turns = 4;
        signals.rates_inquiry = true;

        assert_eq!(scorer.classify(&signals), LeadClassification::MQL);
    }

    #[test]
    fn test_escalation_triggers() {
        let scorer = test_scorer();
        let mut signals = SimpleLeadSignals::default();

        signals.objections = 5;
        signals.amount = Some(2_000_000.0);
        signals.human_requested = true;

        let triggers = scorer.check_escalation_triggers(&signals);
        assert!(triggers.len() >= 3);
    }

    #[test]
    fn test_total_score_capping() {
        let scorer = test_scorer();
        let mut signals = SimpleLeadSignals::default();

        // Max out everything
        signals.has_urgency = true;
        signals.urgency_keywords = 5;
        signals.turns = 10;
        signals.questions = 5;
        signals.rates_inquiry = true;
        signals.comparison_inquiry = true;
        signals.contact_provided = true;
        signals.asset_details = true;
        signals.amount_provided = true;
        signals.specific_requirements = true;
        signals.intent_to_proceed = true;
        signals.callback_requested = true;
        signals.visit_requested = true;

        let breakdown = scorer.calculate_breakdown(&signals);
        assert_eq!(breakdown.total(), 100);
    }
}
