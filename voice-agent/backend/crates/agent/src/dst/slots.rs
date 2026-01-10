//! Dialogue State Types and Utilities
//!
//! Core types for dialogue state tracking, shared across all domain implementations.
//! The actual dialogue state implementation is in `DynamicDialogueState`.
//!
//! P18 FIX: Quality tier system is now domain-agnostic. Use `quality_tier_ids` module
//! instead of domain-specific `purity_ids`. Parsing and display formatting are now
//! config-driven via `SlotsConfig::parse_quality_tier()` and `format_quality_display()`.

use serde::{Deserialize, Serialize};

// ============================================================================
// P18 FIX: Generic Quality Tier System (Domain-Agnostic)
// ============================================================================

/// Quality Tier ID - string-based identifier for asset quality (config-driven)
/// Example IDs: "tier_1", "tier_2", "tier_3", "tier_4"
/// Quality factors are defined in config/domains/{domain}/slots.yaml
pub type QualityTierId = String;

/// Generic quality tier IDs (domain-agnostic)
///
/// P18 FIX: Use these generic tier names instead of domain-specific names.
/// The actual display names and quality factors come from config.
pub mod quality_tier_ids {
    pub const TIER_1: &str = "tier_1";
    pub const TIER_2: &str = "tier_2";
    pub const TIER_3: &str = "tier_3";
    pub const TIER_4: &str = "tier_4";
    pub const UNKNOWN: &str = "unknown";
}

/// Goal ID - string-based goal identifier (config-driven)
///
/// Goals are defined in config/domains/{domain}/goals.yaml
/// Examples: "exploration", "balance_transfer", "new_loan", "eligibility_check"
pub type GoalId = String;

/// Default goal ID
pub const DEFAULT_GOAL: &str = "exploration";

/// Next best action for the agent
#[derive(Debug, Clone, PartialEq)]
pub enum NextBestAction {
    /// Call a specific tool
    CallTool(String),
    /// Ask for a specific slot
    AskFor(String),
    /// Offer to schedule an appointment
    OfferAppointment,
    /// Explain the process (e.g., balance transfer process)
    ExplainProcess,
    /// Discover customer intent first
    DiscoverIntent,
    /// Capture lead now
    CaptureLead,
}

impl NextBestAction {
    /// Get the action type name for template lookup
    pub fn action_type(&self) -> &'static str {
        match self {
            NextBestAction::CallTool(_) => "call_tool",
            NextBestAction::AskFor(_) => "ask_for",
            NextBestAction::OfferAppointment => "offer_appointment",
            NextBestAction::ExplainProcess => "explain_process",
            NextBestAction::DiscoverIntent => "discover_intent",
            NextBestAction::CaptureLead => "capture_lead",
        }
    }

    /// Get the slot or tool name associated with this action (if any)
    pub fn target(&self) -> Option<&str> {
        match self {
            NextBestAction::CallTool(tool) => Some(tool),
            NextBestAction::AskFor(slot) => Some(slot),
            _ => None,
        }
    }

    /// Convert to instruction for LLM using config-driven templates
    ///
    /// This is the preferred method - uses templates from goals.yaml config
    /// to generate domain-agnostic instructions.
    pub fn to_instruction_with_context(
        &self,
        templates: &voice_agent_config::ActionTemplatesConfig,
        context: &voice_agent_config::ActionContext,
        language: &str,
    ) -> String {
        // Build context with action-specific values
        let ctx = match self {
            NextBestAction::CallTool(tool) => context.clone().with_tool(tool),
            NextBestAction::AskFor(slot) => {
                let display = slot.replace('_', " ");
                context.clone().with_slot(slot, &display)
            }
            _ => context.clone(),
        };

        // Get template for this action type
        if let Some(template) = templates.get_template(self.action_type()) {
            template.render(language, &ctx)
        } else {
            self.to_instruction_default()
        }
    }

    /// Default fallback instructions (generic, no brand names)
    pub fn to_instruction_default(&self) -> String {
        match self {
            NextBestAction::CallTool(tool) => {
                format!("CALL the {} tool now with available information", tool)
            }
            NextBestAction::AskFor(slot) => {
                format!(
                    "ASK customer for their {} (required to proceed)",
                    slot.replace('_', " ")
                )
            }
            NextBestAction::OfferAppointment => {
                "OFFER to schedule a branch visit appointment".to_string()
            }
            NextBestAction::ExplainProcess => {
                "EXPLAIN the transfer process: we pay off current lender directly, no cash needed from customer".to_string()
            }
            NextBestAction::DiscoverIntent => {
                "ASK what brings them here today".to_string()
            }
            NextBestAction::CaptureLead => {
                "CAPTURE customer details for follow-up (name and phone)".to_string()
            }
        }
    }
}

/// Urgency level for requirement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UrgencyLevel {
    /// Immediate need (today/tomorrow)
    Immediate,
    /// Soon (within a week)
    Soon,
    /// Planning ahead (no specific timeline)
    Planning,
    /// Just exploring options
    Exploring,
}

impl UrgencyLevel {
    /// Parse from utterance context
    pub fn from_utterance(text: &str) -> Option<Self> {
        let lower = text.to_lowercase();

        // Immediate indicators
        if lower.contains("urgent") || lower.contains("today") || lower.contains("now")
            || lower.contains("immediately") || lower.contains("abhi") || lower.contains("turant")
            || lower.contains("aaj") || lower.contains("emergency")
        {
            return Some(UrgencyLevel::Immediate);
        }

        // Soon indicators
        if lower.contains("this week") || lower.contains("few days") || lower.contains("jaldi")
            || lower.contains("soon") || lower.contains("is hafte")
        {
            return Some(UrgencyLevel::Soon);
        }

        // Planning indicators
        if lower.contains("next month") || lower.contains("planning") || lower.contains("thinking")
            || lower.contains("soch") || lower.contains("agle mahine")
        {
            return Some(UrgencyLevel::Planning);
        }

        // Exploring indicators
        if lower.contains("just checking") || lower.contains("exploring") || lower.contains("options")
            || lower.contains("jaankari") || lower.contains("information")
        {
            return Some(UrgencyLevel::Exploring);
        }

        None
    }
}

impl std::fmt::Display for UrgencyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UrgencyLevel::Immediate => write!(f, "immediate"),
            UrgencyLevel::Soon => write!(f, "soon"),
            UrgencyLevel::Planning => write!(f, "planning"),
            UrgencyLevel::Exploring => write!(f, "exploring"),
        }
    }
}

/// A slot value with confidence and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotValue {
    /// The value as a string
    pub value: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Turn index when this was set
    pub turn_set: usize,
    /// Whether user confirmed this value
    pub confirmed: bool,
}

impl SlotValue {
    /// Create a new slot value
    pub fn new(value: impl Into<String>, confidence: f32, turn: usize) -> Self {
        Self {
            value: value.into(),
            confidence,
            turn_set: turn,
            confirmed: false,
        }
    }

    /// Mark as confirmed
    pub fn confirm(&mut self) {
        self.confirmed = true;
        self.confidence = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_tier_ids() {
        // Verify quality tier IDs are available
        assert_eq!(quality_tier_ids::TIER_1, "tier_1");
        assert_eq!(quality_tier_ids::TIER_2, "tier_2");
        assert_eq!(quality_tier_ids::TIER_3, "tier_3");
        assert_eq!(quality_tier_ids::TIER_4, "tier_4");
        assert_eq!(quality_tier_ids::UNKNOWN, "unknown");
    }

    #[test]
    fn test_urgency_detection() {
        assert_eq!(UrgencyLevel::from_utterance("I need it today"), Some(UrgencyLevel::Immediate));
        assert_eq!(UrgencyLevel::from_utterance("mujhe abhi chahiye"), Some(UrgencyLevel::Immediate));
        assert_eq!(UrgencyLevel::from_utterance("this week sometime"), Some(UrgencyLevel::Soon));
        assert_eq!(UrgencyLevel::from_utterance("just exploring options"), Some(UrgencyLevel::Exploring));
    }

    #[test]
    fn test_slot_value() {
        let mut slot = SlotValue::new("test", 0.8, 0);
        assert_eq!(slot.value, "test");
        assert_eq!(slot.confidence, 0.8);
        assert!(!slot.confirmed);

        slot.confirm();
        assert!(slot.confirmed);
        assert_eq!(slot.confidence, 1.0);
    }

    #[test]
    fn test_action_type_mapping() {
        assert_eq!(NextBestAction::CallTool("test".to_string()).action_type(), "call_tool");
        assert_eq!(NextBestAction::AskFor("slot".to_string()).action_type(), "ask_for");
        assert_eq!(NextBestAction::OfferAppointment.action_type(), "offer_appointment");
        assert_eq!(NextBestAction::ExplainProcess.action_type(), "explain_process");
        assert_eq!(NextBestAction::DiscoverIntent.action_type(), "discover_intent");
        assert_eq!(NextBestAction::CaptureLead.action_type(), "capture_lead");
    }

    #[test]
    fn test_action_target() {
        assert_eq!(NextBestAction::CallTool("my_tool".to_string()).target(), Some("my_tool"));
        assert_eq!(NextBestAction::AskFor("my_slot".to_string()).target(), Some("my_slot"));
        assert_eq!(NextBestAction::DiscoverIntent.target(), None);
    }

    #[test]
    fn test_default_instructions_no_brand() {
        let action = NextBestAction::ExplainProcess;
        let instruction = action.to_instruction_default();
        assert!(instruction.contains("EXPLAIN"));
        assert!(!instruction.contains("Kotak")); // No brand names
    }
}
