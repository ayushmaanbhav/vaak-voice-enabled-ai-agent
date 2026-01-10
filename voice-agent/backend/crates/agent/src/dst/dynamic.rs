//! Dynamic Dialogue State (Domain-Agnostic)
//!
//! Implements a config-driven dialogue state that works with any domain.
//! Slots, goals, and validation are all defined in YAML config files.
//!
//! This replaces hardcoded domain-specific state implementations
//! with a fully dynamic implementation.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use voice_agent_config::domain::{GoalDefinition, SlotDefinition, SlotsConfig};

use super::{DialogueStateTrait, NextBestAction, SlotValue, DEFAULT_GOAL};

/// Dynamic dialogue state that loads slot definitions from config
///
/// This uses a HashMap for all slots, making it fully domain-agnostic.
/// Slot definitions, goals, and validation rules come from YAML config.
///
/// # Example
/// ```ignore
/// use voice_agent_config::domain::SlotsConfig;
/// use voice_agent_agent::dst::DynamicDialogueState;
///
/// let config = SlotsConfig::load("config/domains/my_domain/slots.yaml")?;
/// let mut state = DynamicDialogueState::from_config(Arc::new(config));
///
/// state.set_slot_value("customer_name", "Rahul", 0.9);
/// assert_eq!(state.get_slot_value("customer_name"), Some("Rahul".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicDialogueState {
    /// All slot values stored in a HashMap
    slots: HashMap<String, SlotValue>,

    /// Slots pending confirmation
    pending_slots: HashSet<String>,

    /// Confirmed slots
    confirmed_slots: HashSet<String>,

    /// Primary detected intent
    primary_intent: Option<String>,

    /// Intent confidence
    intent_confidence: f32,

    /// Secondary intents detected
    secondary_intents: Vec<String>,

    /// Current conversation goal ID
    conversation_goal: String,

    /// Whether goal has been explicitly confirmed
    goal_confirmed: bool,

    /// Turn at which goal was set
    goal_set_turn: usize,

    /// Slot configuration (not serialized - provided externally)
    #[serde(skip)]
    config: Option<Arc<SlotsConfig>>,
}

impl Default for DynamicDialogueState {
    fn default() -> Self {
        Self {
            slots: HashMap::new(),
            pending_slots: HashSet::new(),
            confirmed_slots: HashSet::new(),
            primary_intent: None,
            intent_confidence: 0.0,
            secondary_intents: Vec::new(),
            conversation_goal: DEFAULT_GOAL.to_string(),
            goal_confirmed: false,
            goal_set_turn: 0,
            config: None,
        }
    }
}

impl DynamicDialogueState {
    /// Create a new empty dynamic state
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from slots configuration
    ///
    /// The config provides slot definitions, goals, and intent mappings.
    pub fn from_config(config: Arc<SlotsConfig>) -> Self {
        Self {
            config: Some(config),
            ..Default::default()
        }
    }

    /// Set the configuration (useful after deserialization)
    pub fn set_config(&mut self, config: Arc<SlotsConfig>) {
        self.config = Some(config);
    }

    /// Get the slots configuration
    pub fn config(&self) -> Option<&SlotsConfig> {
        self.config.as_ref().map(|c| c.as_ref())
    }

    /// Get slot definition from config
    pub fn get_slot_definition(&self, slot_name: &str) -> Option<&SlotDefinition> {
        self.config.as_ref().and_then(|c| c.get_slot(slot_name))
    }

    /// Get goal definition from config
    pub fn get_goal_definition(&self, goal_id: &str) -> Option<&GoalDefinition> {
        self.config.as_ref().and_then(|c| c.get_goal(goal_id))
    }

    /// Get required slots for a goal from config
    pub fn required_slots_for_goal(&self, goal_id: &str) -> Vec<&str> {
        self.get_goal_definition(goal_id)
            .map(|g| g.required_slots.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get optional slots for a goal from config
    pub fn optional_slots_for_goal(&self, goal_id: &str) -> Vec<&str> {
        self.get_goal_definition(goal_id)
            .map(|g| g.optional_slots.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get completion action for a goal from config
    pub fn completion_action_for_goal(&self, goal_id: &str) -> Option<&str> {
        self.get_goal_definition(goal_id)
            .and_then(|g| g.completion_action.as_deref())
    }

    /// Map intent to goal using config
    pub fn goal_for_intent(&self, intent: &str) -> Option<&str> {
        self.config.as_ref().and_then(|c| c.goal_for_intent(intent))
    }

    /// Get all slot names defined in config
    pub fn defined_slots(&self) -> Vec<&str> {
        self.config
            .as_ref()
            .map(|c| c.slots.keys().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    // ====== Customer Information (common across domains) ======

    /// Get customer name (convenience accessor)
    pub fn customer_name(&self) -> Option<&str> {
        self.slots.get("customer_name").map(|v| v.value.as_str())
    }

    /// Get phone number (convenience accessor)
    pub fn phone_number(&self) -> Option<&str> {
        self.slots.get("phone_number").map(|v| v.value.as_str())
    }

    /// Get location (convenience accessor)
    pub fn location(&self) -> Option<&str> {
        self.slots.get("location").map(|v| v.value.as_str())
    }

    // ====== Intent Tracking ======

    /// Get primary intent
    pub fn primary_intent_value(&self) -> Option<&str> {
        self.primary_intent.as_deref()
    }

    /// Get intent confidence
    pub fn intent_confidence(&self) -> f32 {
        self.intent_confidence
    }

    /// Get secondary intents
    pub fn secondary_intents(&self) -> &[String] {
        &self.secondary_intents
    }

    // ====== Goal Tracking ======

    /// Check if goal is confirmed (explicit) vs inferred
    pub fn is_goal_confirmed(&self) -> bool {
        self.goal_confirmed
    }

    /// Get the turn at which the goal was set
    pub fn goal_set_turn(&self) -> usize {
        self.goal_set_turn
    }

    /// Check if we have complete contact info
    pub fn has_complete_contact(&self) -> bool {
        self.slots.contains_key("customer_name") && self.slots.contains_key("phone_number")
    }

    // ====== State Management ======

    /// Get slots pending confirmation with their values
    pub fn slots_needing_confirmation(&self) -> Vec<(&str, String)> {
        self.pending_slots
            .iter()
            .filter_map(|slot_name| {
                self.get_slot_value(slot_name)
                    .map(|value| (slot_name.as_str(), value))
            })
            .collect()
    }

    /// Generate a confirmation prompt for pending slots
    pub fn pending_confirmation_prompt(&self) -> Option<String> {
        let pending = self.slots_needing_confirmation();
        if pending.is_empty() {
            return None;
        }

        let formatted: Vec<String> = pending
            .iter()
            .map(|(slot, value)| {
                let display_name = slot.replace('_', " ");
                format!("{}: {}", display_name, value)
            })
            .collect();

        Some(format!("Please confirm: {}", formatted.join(", ")))
    }

    /// Calculate completion percentage for a goal
    pub fn completion_for_goal(&self, goal_id: &str) -> f32 {
        let required = self.required_slots_for_goal(goal_id);
        let optional = self.optional_slots_for_goal(goal_id);

        if required.is_empty() && optional.is_empty() {
            return 1.0; // Goals without requirements are always complete
        }

        let required_filled = required
            .iter()
            .filter(|s| self.get_slot_value(s).is_some())
            .count();
        let optional_filled = optional
            .iter()
            .filter(|s| self.get_slot_value(s).is_some())
            .count();

        if required.is_empty() {
            // Only optional slots
            optional_filled as f32 / optional.len() as f32
        } else if optional.is_empty() {
            // Only required slots
            required_filled as f32 / required.len() as f32
        } else {
            // Weight: required slots 70%, optional 30%
            (required_filled as f32 / required.len() as f32) * 0.7
                + (optional_filled as f32 / optional.len() as f32) * 0.3
        }
    }

    /// Get missing required slots for current goal
    pub fn missing_required_slots(&self) -> Vec<&str> {
        self.required_slots_for_goal(&self.conversation_goal)
            .into_iter()
            .filter(|s| self.get_slot_value(s).is_none())
            .collect()
    }

    /// Check if current goal is complete (all required slots filled)
    pub fn is_goal_complete(&self) -> bool {
        self.missing_required_slots().is_empty()
    }
}

// =============================================================================
// DialogueStateTrait Implementation
// =============================================================================

impl DialogueStateTrait for DynamicDialogueState {
    fn primary_intent(&self) -> Option<&str> {
        self.primary_intent.as_deref()
    }

    fn get_slot_value(&self, slot_name: &str) -> Option<String> {
        self.slots.get(slot_name).map(|v| v.value.clone())
    }

    fn set_slot_value(&mut self, slot_name: &str, value: &str, confidence: f32) {
        let slot_value = SlotValue::new(value, confidence, 0);
        self.slots.insert(slot_name.to_string(), slot_value);
    }

    fn clear_slot(&mut self, slot_name: &str) {
        self.slots.remove(slot_name);
        self.pending_slots.remove(slot_name);
        self.confirmed_slots.remove(slot_name);
    }

    fn filled_slots(&self) -> Vec<&str> {
        self.slots.keys().map(|s| s.as_str()).collect()
    }

    fn pending_slots(&self) -> &HashSet<String> {
        &self.pending_slots
    }

    fn confirmed_slots(&self) -> &HashSet<String> {
        &self.confirmed_slots
    }

    fn mark_pending(&mut self, slot_name: &str) {
        self.confirmed_slots.remove(slot_name);
        self.pending_slots.insert(slot_name.to_string());
    }

    fn mark_confirmed(&mut self, slot_name: &str) {
        self.pending_slots.remove(slot_name);
        self.confirmed_slots.insert(slot_name.to_string());

        // Update the slot value's confirmed flag
        if let Some(slot) = self.slots.get_mut(slot_name) {
            slot.confirm();
        }
    }

    fn goal_id(&self) -> &str {
        &self.conversation_goal
    }

    fn set_goal(&mut self, goal_id: &str, turn: usize) {
        // Only update if it's a meaningful change (not downgrading to exploration)
        if goal_id != DEFAULT_GOAL || self.conversation_goal == DEFAULT_GOAL {
            self.conversation_goal = goal_id.to_string();
            self.goal_set_turn = turn;
        }
    }

    fn confirm_goal(&mut self, goal_id: &str, turn: usize) {
        self.conversation_goal = goal_id.to_string();
        self.goal_confirmed = true;
        self.goal_set_turn = turn;
    }

    fn should_auto_capture_lead(&self) -> bool {
        // Don't duplicate if already in lead capture mode
        if self.conversation_goal == "lead_capture" {
            return false;
        }

        // Capture lead if we have both name and phone collected
        self.has_complete_contact()
    }

    fn to_context_string(&self) -> String {
        let mut parts = Vec::new();

        // Common customer info
        if let Some(name) = self.customer_name() {
            parts.push(format!("Customer: {}", name));
        }
        if let Some(phone) = self.phone_number() {
            parts.push(format!("Phone: {}", phone));
        }
        if let Some(loc) = self.location() {
            parts.push(format!("Location: {}", loc));
        }

        // All other slots
        for (slot_name, slot_value) in &self.slots {
            // Skip already handled slots
            if ["customer_name", "phone_number", "location"].contains(&slot_name.as_str()) {
                continue;
            }
            let display_name = slot_name.replace('_', " ");
            // Capitalize first letter
            let display_name = display_name
                .chars()
                .enumerate()
                .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
                .collect::<String>();
            parts.push(format!("{}: {}", display_name, slot_value.value));
        }

        // Intent
        if let Some(intent) = self.primary_intent() {
            parts.push(format!("Intent: {}", intent));
        }

        if parts.is_empty() {
            "No information collected yet.".to_string()
        } else {
            parts.join("\n")
        }
    }

    fn to_full_context_string(&self) -> String {
        let mut output = String::new();

        // Collected information
        output.push_str("# Customer Information\n");
        output.push_str(&self.to_context_string());
        output.push_str("\n\n");

        // Goal info
        output.push_str(&format!("# Current Goal: {}\n", self.conversation_goal));

        // Missing slots
        let missing = self.missing_required_slots();
        if !missing.is_empty() {
            output.push_str(&format!(
                "# Missing Required: {}\n",
                missing.join(", ")
            ));
        }

        output
    }

    fn update_intent(&mut self, intent: &str, confidence: f32) {
        // If we already have this intent, just update confidence
        if self.primary_intent.as_deref() == Some(intent) {
            self.intent_confidence = confidence;
            return;
        }

        // Move current primary to secondary if exists
        if let Some(ref prev) = self.primary_intent {
            if !self.secondary_intents.contains(prev) {
                self.secondary_intents.push(prev.clone());
            }
        }

        self.primary_intent = Some(intent.to_string());
        self.intent_confidence = confidence;
    }

    fn get_slot_with_confidence(&self, slot_name: &str) -> Option<&SlotValue> {
        self.slots.get(slot_name)
    }

    fn next_best_action(&self) -> NextBestAction {
        // Check if we're in exploration mode
        if self.conversation_goal == DEFAULT_GOAL || self.conversation_goal.is_empty() {
            return NextBestAction::DiscoverIntent;
        }

        // Get missing required slots for current goal
        let missing = self.missing_required_slots();
        if !missing.is_empty() {
            // Ask for the first missing slot
            return NextBestAction::AskFor(missing[0].to_string());
        }

        // All required slots filled - check completion action
        if let Some(action) = self.completion_action_for_goal(&self.conversation_goal) {
            return NextBestAction::CallTool(action.to_string());
        }

        // Special cases based on goal
        match self.conversation_goal.as_str() {
            "branch_visit" => NextBestAction::OfferAppointment,
            "lead_capture" => NextBestAction::CaptureLead,
            _ => {
                // Default: discover more intent
                NextBestAction::DiscoverIntent
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Arc<SlotsConfig> {
        let yaml = r#"
slots:
  customer_name:
    type: string
    description: "Customer name"
  phone_number:
    type: string
    description: "Phone number"
  gold_weight:
    type: number
    description: "Gold weight in grams"
  loan_amount:
    type: number
    description: "Loan amount"
  current_lender:
    type: string
    description: "Current lender"

goals:
  exploration:
    description: "Just exploring"
    required_slots: []
  balance_transfer:
    description: "Transfer loan"
    required_slots:
      - current_lender
      - loan_amount
    completion_action: calculate_savings
  eligibility_check:
    description: "Check eligibility"
    required_slots:
      - gold_weight
    completion_action: check_eligibility
  lead_capture:
    description: "Capture lead"
    required_slots:
      - customer_name
      - phone_number
    completion_action: capture_lead

intent_mapping:
  balance_transfer:
    - switch_lender
    - balance_transfer
  eligibility_check:
    - eligibility_check
    - loan_inquiry
"#;
        Arc::new(serde_yaml::from_str(yaml).unwrap())
    }

    #[test]
    fn test_dynamic_state_creation() {
        let state = DynamicDialogueState::new();
        assert!(state.customer_name().is_none());
        assert!(state.filled_slots().is_empty());
        assert_eq!(state.goal_id(), DEFAULT_GOAL);
    }

    #[test]
    fn test_dynamic_state_from_config() {
        let config = create_test_config();
        let state = DynamicDialogueState::from_config(config);

        // Should have access to defined slots
        let defined = state.defined_slots();
        assert!(defined.contains(&"customer_name"));
        assert!(defined.contains(&"gold_weight"));
    }

    #[test]
    fn test_slot_set_and_get() {
        let mut state = DynamicDialogueState::new();

        state.set_slot_value("customer_name", "Rahul", 0.9);
        state.set_slot_value("loan_amount", "500000", 0.85);

        assert_eq!(state.customer_name(), Some("Rahul"));
        assert_eq!(state.get_slot_value("loan_amount"), Some("500000".to_string()));
    }

    #[test]
    fn test_slot_confirmation() {
        let mut state = DynamicDialogueState::new();

        state.set_slot_value("gold_weight", "50", 0.8);
        state.mark_pending("gold_weight");

        assert!(state.pending_slots().contains("gold_weight"));
        assert!(!state.confirmed_slots().contains("gold_weight"));

        state.mark_confirmed("gold_weight");

        assert!(!state.pending_slots().contains("gold_weight"));
        assert!(state.confirmed_slots().contains("gold_weight"));
    }

    #[test]
    fn test_goal_tracking() {
        let mut state = DynamicDialogueState::new();
        assert_eq!(state.goal_id(), "exploration");

        state.set_goal("balance_transfer", 1);
        assert_eq!(state.goal_id(), "balance_transfer");
        assert!(!state.is_goal_confirmed());

        state.confirm_goal("new_loan", 2);
        assert_eq!(state.goal_id(), "new_loan");
        assert!(state.is_goal_confirmed());
    }

    #[test]
    fn test_required_slots_from_config() {
        let config = create_test_config();
        let state = DynamicDialogueState::from_config(config);

        let required = state.required_slots_for_goal("balance_transfer");
        assert!(required.contains(&"current_lender"));
        assert!(required.contains(&"loan_amount"));
    }

    #[test]
    fn test_missing_required_slots() {
        let config = create_test_config();
        let mut state = DynamicDialogueState::from_config(config);

        state.set_goal("balance_transfer", 0);

        // Should need both slots
        let missing = state.missing_required_slots();
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&"current_lender"));
        assert!(missing.contains(&"loan_amount"));

        // P23 FIX: Use generic provider name - actual values come from domain config
        state.set_slot_value("current_lender", "competitor_1", 0.9);
        let missing = state.missing_required_slots();
        assert_eq!(missing.len(), 1);
        assert!(missing.contains(&"loan_amount"));

        // Fill both slots
        state.set_slot_value("loan_amount", "500000", 0.9);
        assert!(state.missing_required_slots().is_empty());
        assert!(state.is_goal_complete());
    }

    #[test]
    fn test_next_best_action() {
        let config = create_test_config();
        let mut state = DynamicDialogueState::from_config(config);

        // Exploration mode -> discover intent
        assert_eq!(state.next_best_action(), NextBestAction::DiscoverIntent);

        // Set goal with missing slots -> ask for slot
        state.set_goal("balance_transfer", 0);
        match state.next_best_action() {
            NextBestAction::AskFor(slot) => {
                assert!(slot == "current_lender" || slot == "loan_amount");
            }
            _ => panic!("Expected AskFor action"),
        }

        // P23 FIX: Use generic provider name - actual values come from domain config
        state.set_slot_value("current_lender", "competitor_1", 0.9);
        state.set_slot_value("loan_amount", "500000", 0.9);
        assert_eq!(
            state.next_best_action(),
            NextBestAction::CallTool("calculate_savings".to_string())
        );
    }

    #[test]
    fn test_goal_for_intent() {
        let config = create_test_config();
        let state = DynamicDialogueState::from_config(config);

        assert_eq!(state.goal_for_intent("switch_lender"), Some("balance_transfer"));
        assert_eq!(state.goal_for_intent("eligibility_check"), Some("eligibility_check"));
        assert_eq!(state.goal_for_intent("unknown_intent"), None);
    }

    #[test]
    fn test_completion_percentage() {
        let config = create_test_config();
        let mut state = DynamicDialogueState::from_config(config);

        // No slots filled
        assert_eq!(state.completion_for_goal("balance_transfer"), 0.0);

        // P23 FIX: Use generic provider name - actual values come from domain config
        state.set_slot_value("current_lender", "competitor_1", 0.9);
        assert!((state.completion_for_goal("balance_transfer") - 0.5).abs() < 0.01);

        // Both required slots
        state.set_slot_value("loan_amount", "500000", 0.9);
        assert_eq!(state.completion_for_goal("balance_transfer"), 1.0);
    }

    #[test]
    fn test_context_string() {
        let mut state = DynamicDialogueState::new();

        state.set_slot_value("customer_name", "Rahul", 0.9);
        state.set_slot_value("loan_amount", "500000", 0.9);

        let context = state.to_context_string();
        assert!(context.contains("Rahul"));
        assert!(context.contains("Loan amount: 500000"));
    }

    #[test]
    fn test_intent_update() {
        let mut state = DynamicDialogueState::new();

        state.update_intent("loan_inquiry", 0.9);
        assert_eq!(state.primary_intent(), Some("loan_inquiry"));

        state.update_intent("eligibility_check", 0.85);
        assert_eq!(state.primary_intent(), Some("eligibility_check"));
        assert!(state.secondary_intents().contains(&"loan_inquiry".to_string()));
    }

    #[test]
    fn test_auto_capture_lead() {
        let mut state = DynamicDialogueState::new();

        assert!(!state.should_auto_capture_lead());

        state.set_slot_value("customer_name", "Rahul", 0.9);
        assert!(!state.should_auto_capture_lead()); // Still missing phone

        state.set_slot_value("phone_number", "9876543210", 0.9);
        assert!(state.should_auto_capture_lead()); // Now has both

        // But not if already in lead_capture mode
        state.set_goal("lead_capture", 0);
        assert!(!state.should_auto_capture_lead());
    }

    #[test]
    fn test_clear_slot() {
        let mut state = DynamicDialogueState::new();

        state.set_slot_value("customer_name", "Rahul", 0.9);
        state.mark_confirmed("customer_name");

        assert!(state.customer_name().is_some());
        assert!(state.confirmed_slots().contains("customer_name"));

        state.clear_slot("customer_name");

        assert!(state.customer_name().is_none());
        assert!(!state.confirmed_slots().contains("customer_name"));
    }
}
