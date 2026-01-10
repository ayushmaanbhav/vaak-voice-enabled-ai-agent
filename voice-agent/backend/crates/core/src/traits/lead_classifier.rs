//! Lead Classifier trait for config-driven lead qualification
//!
//! This module provides a domain-agnostic interface for classifying leads
//! (MQL/SQL) based on configurable rules. All classification criteria
//! are loaded from configuration (lead_scoring.yaml).
//!
//! # P20 FIX: Replaces hardcoded classification in lead_scoring.rs
//!
//! The previous implementation had hardcoded MQL/SQL criteria:
//! ```ignore
//! if signals.has_urgency_signal
//!     && signals.provided_contact_info
//!     && signals.has_specific_requirements
//! {
//!     return LeadClassification::SQL;
//! }
//! ```
//!
//! This trait enables fully config-driven classification rules.
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_core::traits::LeadClassifier;
//!
//! // Classifier is created from domain config
//! let classifier = config_bridge.lead_classifier();
//!
//! // Classify lead based on signals
//! let classification = classifier.classify(&signals);
//! let qualification = classifier.qualification_level(score);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lead classification types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeadClass {
    /// Not yet qualified
    Unqualified,
    /// Marketing Qualified Lead
    MQL,
    /// Sales Qualified Lead
    SQL,
}

impl Default for LeadClass {
    fn default() -> Self {
        Self::Unqualified
    }
}

/// Lead qualification levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualificationLevel {
    Cold,
    Warm,
    Hot,
    Qualified,
}

impl Default for QualificationLevel {
    fn default() -> Self {
        Self::Cold
    }
}

/// Signals collected during conversation for lead scoring
pub trait LeadSignalsTrait: Send + Sync {
    /// Check if a signal flag is set
    fn has_signal(&self, signal_name: &str) -> bool;

    /// Get numeric signal value
    fn get_numeric_signal(&self, signal_name: &str) -> Option<u32>;

    /// Get all active signal names
    fn active_signals(&self) -> Vec<&str>;
}

/// Simple implementation of LeadSignalsTrait
#[derive(Debug, Clone, Default)]
pub struct SimpleLeadSignalsImpl {
    /// Boolean signal flags
    pub flags: HashMap<String, bool>,
    /// Numeric signal values
    pub values: HashMap<String, u32>,
}

impl SimpleLeadSignalsImpl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_flag(&mut self, name: &str, value: bool) {
        self.flags.insert(name.to_string(), value);
    }

    pub fn set_value(&mut self, name: &str, value: u32) {
        self.values.insert(name.to_string(), value);
    }
}

impl LeadSignalsTrait for SimpleLeadSignalsImpl {
    fn has_signal(&self, signal_name: &str) -> bool {
        self.flags.get(signal_name).copied().unwrap_or(false)
    }

    fn get_numeric_signal(&self, signal_name: &str) -> Option<u32> {
        self.values.get(signal_name).copied()
    }

    fn active_signals(&self) -> Vec<&str> {
        self.flags
            .iter()
            .filter(|(_, v)| **v)
            .map(|(k, _)| k.as_str())
            .collect()
    }
}

/// Classification rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRule {
    /// All of these signal flags must be true
    #[serde(default)]
    pub required_flags: Vec<String>,
    /// At least one of these signal flags must be true
    #[serde(default)]
    pub any_of_flags: Vec<String>,
    /// Numeric thresholds (signal_name -> min_value)
    #[serde(default)]
    pub thresholds: HashMap<String, u32>,
}

impl ClassificationRule {
    /// Check if rule matches the signals
    pub fn matches(&self, signals: &dyn LeadSignalsTrait) -> bool {
        // Check required flags (all must be true)
        let has_required = self
            .required_flags
            .iter()
            .all(|f| signals.has_signal(f));

        if !has_required {
            return false;
        }

        // Check any_of flags (at least one must be true, or list is empty)
        let has_any = self.any_of_flags.is_empty()
            || self.any_of_flags.iter().any(|f| signals.has_signal(f));

        if !has_any {
            return false;
        }

        // Check thresholds
        let meets_thresholds = self.thresholds.iter().all(|(signal, min_value)| {
            signals
                .get_numeric_signal(signal)
                .map(|v| v >= *min_value)
                .unwrap_or(false)
        });

        meets_thresholds
    }
}

/// Qualification level threshold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualificationThreshold {
    /// Minimum score for this level
    pub min_score: u32,
    /// Maximum score for this level
    pub max_score: u32,
}

/// Escalation trigger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationTriggerConfig {
    /// Trigger ID
    pub id: String,
    /// Signal to check
    pub signal: String,
    /// Threshold value (for numeric signals)
    #[serde(default)]
    pub threshold: Option<u32>,
    /// Priority (lower = higher priority)
    #[serde(default = "default_priority")]
    pub priority: u8,
    /// Display message
    pub message: String,
}

fn default_priority() -> u8 {
    50
}

/// Escalation trigger result
#[derive(Debug, Clone)]
pub struct EscalationTriggerResult {
    pub trigger_id: String,
    pub priority: u8,
    pub message: String,
}

/// Trait for config-driven lead classification
///
/// This trait replaces hardcoded MQL/SQL classification rules.
/// All rules are loaded from configuration.
pub trait LeadClassifier: Send + Sync {
    /// Classify lead based on signals
    ///
    /// Returns SQL, MQL, or Unqualified based on config rules.
    fn classify(&self, signals: &dyn LeadSignalsTrait) -> LeadClass;

    /// Get qualification level from score
    fn qualification_level(&self, score: u32) -> QualificationLevel;

    /// Get score thresholds for each qualification level
    fn qualification_thresholds(&self) -> HashMap<QualificationLevel, (u32, u32)>;

    /// Check if escalation should be triggered
    fn should_escalate(&self, signals: &dyn LeadSignalsTrait) -> Option<EscalationTriggerResult>;

    /// Get all escalation triggers
    fn escalation_triggers(&self) -> Vec<&EscalationTriggerConfig>;

    /// Get classification rules
    fn sql_rule(&self) -> &ClassificationRule;
    fn mql_rule(&self) -> &ClassificationRule;
}

/// Config-driven lead classifier
#[derive(Debug, Clone)]
pub struct ConfigLeadClassifier {
    /// SQL classification rule
    sql_rule: ClassificationRule,
    /// MQL classification rule
    mql_rule: ClassificationRule,
    /// Qualification level thresholds
    qualification_thresholds: HashMap<String, QualificationThreshold>,
    /// Escalation triggers
    escalation_triggers: Vec<EscalationTriggerConfig>,
}

impl ConfigLeadClassifier {
    /// Create from config structures
    pub fn new(
        sql_rule: ClassificationRule,
        mql_rule: ClassificationRule,
        qualification_thresholds: HashMap<String, QualificationThreshold>,
        escalation_triggers: Vec<EscalationTriggerConfig>,
    ) -> Self {
        Self {
            sql_rule,
            mql_rule,
            qualification_thresholds,
            escalation_triggers,
        }
    }

    /// Create with default thresholds
    pub fn with_default_thresholds(
        sql_rule: ClassificationRule,
        mql_rule: ClassificationRule,
    ) -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert(
            "cold".to_string(),
            QualificationThreshold {
                min_score: 0,
                max_score: 29,
            },
        );
        thresholds.insert(
            "warm".to_string(),
            QualificationThreshold {
                min_score: 30,
                max_score: 59,
            },
        );
        thresholds.insert(
            "hot".to_string(),
            QualificationThreshold {
                min_score: 60,
                max_score: 79,
            },
        );
        thresholds.insert(
            "qualified".to_string(),
            QualificationThreshold {
                min_score: 80,
                max_score: 100,
            },
        );

        Self {
            sql_rule,
            mql_rule,
            qualification_thresholds: thresholds,
            escalation_triggers: Vec::new(),
        }
    }
}

impl LeadClassifier for ConfigLeadClassifier {
    fn classify(&self, signals: &dyn LeadSignalsTrait) -> LeadClass {
        // Check SQL first (higher priority)
        if self.sql_rule.matches(signals) {
            return LeadClass::SQL;
        }

        // Check MQL
        if self.mql_rule.matches(signals) {
            return LeadClass::MQL;
        }

        LeadClass::Unqualified
    }

    fn qualification_level(&self, score: u32) -> QualificationLevel {
        for (level, threshold) in &self.qualification_thresholds {
            if score >= threshold.min_score && score <= threshold.max_score {
                return match level.as_str() {
                    "cold" => QualificationLevel::Cold,
                    "warm" => QualificationLevel::Warm,
                    "hot" => QualificationLevel::Hot,
                    "qualified" => QualificationLevel::Qualified,
                    _ => QualificationLevel::Cold,
                };
            }
        }
        QualificationLevel::Cold
    }

    fn qualification_thresholds(&self) -> HashMap<QualificationLevel, (u32, u32)> {
        let mut result = HashMap::new();
        for (level, threshold) in &self.qualification_thresholds {
            let qual = match level.as_str() {
                "cold" => QualificationLevel::Cold,
                "warm" => QualificationLevel::Warm,
                "hot" => QualificationLevel::Hot,
                "qualified" => QualificationLevel::Qualified,
                _ => continue,
            };
            result.insert(qual, (threshold.min_score, threshold.max_score));
        }
        result
    }

    fn should_escalate(&self, signals: &dyn LeadSignalsTrait) -> Option<EscalationTriggerResult> {
        let mut triggers: Vec<_> = self
            .escalation_triggers
            .iter()
            .filter(|t| {
                if let Some(threshold) = t.threshold {
                    signals
                        .get_numeric_signal(&t.signal)
                        .map(|v| v >= threshold)
                        .unwrap_or(false)
                } else {
                    signals.has_signal(&t.signal)
                }
            })
            .collect();

        // Sort by priority (lower = higher priority)
        triggers.sort_by_key(|t| t.priority);

        triggers.first().map(|t| EscalationTriggerResult {
            trigger_id: t.id.clone(),
            priority: t.priority,
            message: t.message.clone(),
        })
    }

    fn escalation_triggers(&self) -> Vec<&EscalationTriggerConfig> {
        self.escalation_triggers.iter().collect()
    }

    fn sql_rule(&self) -> &ClassificationRule {
        &self.sql_rule
    }

    fn mql_rule(&self) -> &ClassificationRule {
        &self.mql_rule
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_rule_matching() {
        let rule = ClassificationRule {
            required_flags: vec![
                "has_urgency_signal".to_string(),
                "provided_contact_info".to_string(),
            ],
            any_of_flags: vec![],
            thresholds: HashMap::new(),
        };

        let mut signals = SimpleLeadSignalsImpl::new();
        signals.set_flag("has_urgency_signal", true);
        signals.set_flag("provided_contact_info", true);

        assert!(rule.matches(&signals));

        signals.set_flag("provided_contact_info", false);
        assert!(!rule.matches(&signals));
    }

    #[test]
    fn test_any_of_flags() {
        let rule = ClassificationRule {
            required_flags: vec!["engagement".to_string()],
            any_of_flags: vec!["asked_about_rates".to_string(), "asked_for_comparison".to_string()],
            thresholds: HashMap::new(),
        };

        let mut signals = SimpleLeadSignalsImpl::new();
        signals.set_flag("engagement", true);
        signals.set_flag("asked_about_rates", true);

        assert!(rule.matches(&signals));

        signals.set_flag("asked_about_rates", false);
        assert!(!rule.matches(&signals)); // Neither any_of flag is true

        signals.set_flag("asked_for_comparison", true);
        assert!(rule.matches(&signals)); // One any_of flag is true
    }

    #[test]
    fn test_threshold_matching() {
        let mut thresholds = HashMap::new();
        thresholds.insert("engagement_turns".to_string(), 3);

        let rule = ClassificationRule {
            required_flags: vec![],
            any_of_flags: vec![],
            thresholds,
        };

        let mut signals = SimpleLeadSignalsImpl::new();
        signals.set_value("engagement_turns", 2);
        assert!(!rule.matches(&signals));

        signals.set_value("engagement_turns", 3);
        assert!(rule.matches(&signals));

        signals.set_value("engagement_turns", 5);
        assert!(rule.matches(&signals));
    }

    #[test]
    fn test_lead_classification() {
        let sql_rule = ClassificationRule {
            required_flags: vec![
                "has_urgency_signal".to_string(),
                "provided_contact_info".to_string(),
                "has_specific_requirements".to_string(),
            ],
            any_of_flags: vec![],
            thresholds: HashMap::new(),
        };

        let mut mql_thresholds = HashMap::new();
        mql_thresholds.insert("engagement_turns".to_string(), 3);

        let mql_rule = ClassificationRule {
            required_flags: vec![],
            any_of_flags: vec!["asked_about_rates".to_string(), "asked_for_comparison".to_string()],
            thresholds: mql_thresholds,
        };

        let classifier = ConfigLeadClassifier::with_default_thresholds(sql_rule, mql_rule);

        // Test SQL classification
        let mut sql_signals = SimpleLeadSignalsImpl::new();
        sql_signals.set_flag("has_urgency_signal", true);
        sql_signals.set_flag("provided_contact_info", true);
        sql_signals.set_flag("has_specific_requirements", true);
        assert_eq!(classifier.classify(&sql_signals), LeadClass::SQL);

        // Test MQL classification
        let mut mql_signals = SimpleLeadSignalsImpl::new();
        mql_signals.set_value("engagement_turns", 4);
        mql_signals.set_flag("asked_about_rates", true);
        assert_eq!(classifier.classify(&mql_signals), LeadClass::MQL);

        // Test Unqualified
        let unqualified = SimpleLeadSignalsImpl::new();
        assert_eq!(classifier.classify(&unqualified), LeadClass::Unqualified);
    }

    #[test]
    fn test_qualification_level() {
        let classifier = ConfigLeadClassifier::with_default_thresholds(
            ClassificationRule::default(),
            ClassificationRule::default(),
        );

        assert_eq!(classifier.qualification_level(15), QualificationLevel::Cold);
        assert_eq!(classifier.qualification_level(45), QualificationLevel::Warm);
        assert_eq!(classifier.qualification_level(70), QualificationLevel::Hot);
        assert_eq!(
            classifier.qualification_level(90),
            QualificationLevel::Qualified
        );
    }
}

impl Default for ClassificationRule {
    fn default() -> Self {
        Self {
            required_flags: vec![],
            any_of_flags: vec![],
            thresholds: HashMap::new(),
        }
    }
}
