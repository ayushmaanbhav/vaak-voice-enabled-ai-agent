//! Domain Abstraction Traits
//!
//! Generic traits for domain-agnostic agent behavior.
//! These traits define interfaces without any specific domain knowledge
//! (e.g., no "gold loan" terminology in core).
//!
//! Domain-specific implementations are provided by the config crate
//! based on YAML configuration files.

use std::collections::{HashMap, HashSet};

/// Generic identifiers used across domains
pub type StageId = String;
pub type SegmentId = String;
pub type ObjectionId = String;
pub type FeatureId = String;
pub type SlotId = String;
pub type ToolId = String;

/// Generic customer signals for segment matching
/// Domain config defines which keys are relevant (e.g., "gold_weight_grams")
#[derive(Debug, Clone, Default)]
pub struct CustomerSignals {
    /// Numeric values extracted from conversation
    pub numeric_values: HashMap<String, f64>,
    /// Text values extracted from conversation
    pub text_values: HashMap<String, String>,
    /// Boolean flags detected
    pub flags: HashSet<String>,
}

impl CustomerSignals {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_numeric(&mut self, key: impl Into<String>, value: f64) {
        self.numeric_values.insert(key.into(), value);
    }

    pub fn set_text(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.text_values.insert(key.into(), value.into());
    }

    pub fn set_flag(&mut self, flag: impl Into<String>) {
        self.flags.insert(flag.into());
    }

    pub fn get_numeric(&self, key: &str) -> Option<f64> {
        self.numeric_values.get(key).copied()
    }

    pub fn get_text(&self, key: &str) -> Option<&str> {
        self.text_values.get(key).map(|s| s.as_str())
    }

    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.contains(flag)
    }
}

/// Pattern type for matching text
#[derive(Debug, Clone)]
pub enum PatternType {
    /// Regular expression pattern
    Regex,
    /// Exact keyword match
    Keyword,
    /// Fuzzy/phonetic match
    Fuzzy,
}

/// Generic pattern for matching user input
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Pattern type
    pub pattern_type: PatternType,
    /// Pattern value (regex string, keyword, etc.)
    pub value: String,
    /// Language code (e.g., "en", "hi")
    pub language: Option<String>,
}

/// Response template with variable placeholders
#[derive(Debug, Clone)]
pub struct ResponseTemplate {
    /// Template text with {placeholders}
    pub template: String,
    /// Language code
    pub language: String,
}

// NOTE: The following traits have been superseded by better implementations:
// - CustomerSegment trait → use core::traits::SegmentDetector instead
// - ObjectionHandler trait → use core::traits::ObjectionHandler instead
// - ConversationStage trait → use config/domain/stages.rs StageDefinition instead
//
// These traits were part of an earlier design iteration. The current architecture
// uses config-driven implementations via DomainBridge (see config/domain/bridge.rs).
//
// For segment detection: use ConfigSegmentDetector via bridge.segment_detector()
// For objection handling: use ConfigObjectionHandler via bridge.objection_handler()
// For conversation stages: use StagesConfig loaded from stages.yaml

/// Generic slot definition for dialogue state tracking
#[derive(Debug, Clone)]
pub struct SlotDefinition {
    /// Slot identifier
    pub slot_id: SlotId,
    /// Slot type
    pub slot_type: SlotType,
    /// Extraction patterns
    pub patterns: Vec<Pattern>,
    /// Validation rules
    pub validation: Option<SlotValidation>,
    /// Unit conversions (e.g., "tola" -> grams multiplier)
    pub unit_conversions: HashMap<String, f64>,
}

/// Slot types for DST
#[derive(Debug, Clone)]
pub enum SlotType {
    /// String value
    String,
    /// Numeric value
    Number { min: Option<f64>, max: Option<f64> },
    /// Enumerated value
    Enum { values: Vec<String> },
    /// Date value
    Date,
    /// Boolean value
    Boolean,
}

/// Slot validation rules
#[derive(Debug, Clone)]
pub struct SlotValidation {
    /// Required field
    pub required: bool,
    /// Regex pattern for validation
    pub pattern: Option<String>,
    /// Custom validation function name
    pub validator: Option<String>,
}

/// Goal definition for DST
#[derive(Debug, Clone)]
pub struct GoalDefinition {
    /// Goal identifier
    pub goal_id: String,
    /// Required slots for this goal
    pub required_slots: Vec<SlotId>,
    /// Optional slots for this goal
    pub optional_slots: Vec<SlotId>,
    /// Action to take when goal is complete
    pub completion_action: Option<String>,
}

/// Scoring weights for lead qualification
#[derive(Debug, Clone)]
pub struct ScoringWeights {
    /// Weight for urgency signals
    pub urgency: f32,
    /// Weight for engagement level
    pub engagement: f32,
    /// Weight for information provided
    pub information: f32,
    /// Weight for intent signals
    pub intent: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            urgency: 25.0,
            engagement: 25.0,
            information: 25.0,
            intent: 25.0,
        }
    }
}

/// Scoring thresholds
#[derive(Debug, Clone)]
pub struct ScoringThresholds {
    /// High-value amount threshold
    pub high_value_amount: f64,
    /// Minimum engagement turns for MQL
    pub min_engagement_turns: usize,
}

// NOTE: DomainView trait has been superseded by concrete view implementations:
// - AgentDomainView (config/domain/views.rs) - for agent crate
// - LlmDomainView (config/domain/views.rs) - for llm crate
// - ToolsDomainView (config/domain/views.rs) - for tools crate
//
// These concrete views provide type-safe access to domain configuration
// loaded from MasterDomainConfig. See CONFIG_CONSOLIDATION_PLAN.md for details.
