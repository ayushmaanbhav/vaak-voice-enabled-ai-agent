//! Dialogue State Tracking (DST) for Conversations
//!
//! Domain-agnostic dialogue state tracking based on LDST and ACL 2024 research.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                  DialogueStateTrait (trait)                 │
//! │  - get/set slot values, goal tracking, intent tracking      │
//! └─────────────────────────────────────────────────────────────┘
//!                               │
//!                               ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │              DynamicDialogueState (implementation)          │
//! │  - HashMap<String, SlotValue> for ALL slots                 │
//! │  - Loads slot definitions from config                       │
//! │  - Fully domain-agnostic                                    │
//! └─────────────────────────────────────────────────────────────┘
//!                               │
//!                               ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   DialogueStateTracker                      │
//! │  - Wraps DynamicDialogueState                               │
//! │  - History, corrections, confirmations                      │
//! │  - Config-driven slot validation                            │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use voice_agent_config::domain::SlotsConfig;
//! use voice_agent_agent::dst::{DynamicDialogueState, DialogueStateTracker};
//!
//! // Load config for your domain
//! let config = SlotsConfig::load("config/domains/my_domain/slots.yaml")?;
//! let state = DynamicDialogueState::from_config(Arc::new(config.clone()));
//! let mut tracker = DialogueStateTracker::new(state, Arc::new(config));
//!
//! // Use the tracker
//! tracker.update_slot("customer_name", "Rahul", 0.9, ChangeSource::UserUtterance, 0);
//! ```

pub mod slots;
pub mod dynamic;

// Core types from slots module
pub use slots::{
    SlotValue, UrgencyLevel, GoalId, NextBestAction, DEFAULT_GOAL,
    PurityId, purity_ids, parse_purity_id, format_purity_display,
};

// Primary dialogue state implementation
pub use dynamic::DynamicDialogueState;


// Re-export SlotExtractor from text_processing
pub use voice_agent_text_processing::SlotExtractor;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use voice_agent_text_processing::intent::{DetectedIntent, Slot};
use voice_agent_config::domain::AgentDomainView;

// =============================================================================
// DialogueStateTrait - The Abstraction
// =============================================================================

/// Trait for dialogue state abstraction
///
/// This trait defines the interface for dialogue state implementations.
/// The primary implementation is `DynamicDialogueState`.
pub trait DialogueStateTrait: Send + Sync {
    /// Get the primary detected intent
    fn primary_intent(&self) -> Option<&str>;

    /// Get a slot value by name
    fn get_slot_value(&self, slot_name: &str) -> Option<String>;

    /// Set a slot value with confidence
    fn set_slot_value(&mut self, slot_name: &str, value: &str, confidence: f32);

    /// Clear a slot value
    fn clear_slot(&mut self, slot_name: &str);

    /// Get all filled slot names
    fn filled_slots(&self) -> Vec<&str>;

    /// Get pending (unconfirmed) slot names
    fn pending_slots(&self) -> &HashSet<String>;

    /// Get confirmed slot names
    fn confirmed_slots(&self) -> &HashSet<String>;

    /// Mark a slot as pending confirmation
    fn mark_pending(&mut self, slot_name: &str);

    /// Mark a slot as confirmed
    fn mark_confirmed(&mut self, slot_name: &str);

    /// Get current goal ID
    fn goal_id(&self) -> &str;

    /// Set current goal
    fn set_goal(&mut self, goal_id: &str, turn: usize);

    /// Confirm goal (user explicitly stated it)
    fn confirm_goal(&mut self, goal_id: &str, turn: usize);

    /// Check if we should auto-capture lead
    fn should_auto_capture_lead(&self) -> bool;

    /// Generate context string for prompts
    fn to_context_string(&self) -> String;

    /// Generate full context including goal information
    fn to_full_context_string(&self) -> String;

    /// Update intent with confidence
    fn update_intent(&mut self, intent: &str, confidence: f32);

    /// Get slot value with confidence
    fn get_slot_with_confidence(&self, slot_name: &str) -> Option<&SlotValue>;

    /// Get next best action for current state
    fn next_best_action(&self) -> NextBestAction;
}

// Re-export as DialogueState for backward compatibility
pub use DialogueStateTrait as DialogueState;

// =============================================================================
// DialogueStateTracker - The Primary Tracker
// =============================================================================

/// Configuration for DST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DstConfig {
    /// Minimum confidence to accept a slot value
    pub min_slot_confidence: f32,
    /// Confidence threshold for auto-confirmation
    pub auto_confirm_confidence: f32,
    /// Enable correction detection
    pub enable_corrections: bool,
    /// Maximum turns to look back for corrections
    pub correction_lookback: usize,
}

impl Default for DstConfig {
    fn default() -> Self {
        Self {
            min_slot_confidence: 0.5,
            auto_confirm_confidence: 0.9,
            enable_corrections: true,
            correction_lookback: 3,
        }
    }
}

/// Record of a state change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Slot that changed
    pub slot_name: String,
    /// Old value
    pub old_value: Option<String>,
    /// New value
    pub new_value: Option<String>,
    /// Confidence of the change
    pub confidence: f32,
    /// Source of the change
    pub source: ChangeSource,
    /// Turn index
    pub turn_index: usize,
}

/// Source of a state change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeSource {
    /// Extracted from user utterance
    UserUtterance,
    /// User correction ("actually, it's...")
    Correction,
    /// System confirmation
    SystemConfirmation,
    /// External data (CRM, etc.)
    External,
}

/// Dialogue State Tracker
///
/// Wraps `DynamicDialogueState` and provides history tracking, corrections,
/// and config-driven slot validation.
pub struct DialogueStateTracker {
    /// Current dialogue state
    state: DynamicDialogueState,
    /// History of state changes
    history: Vec<StateChange>,
    /// Configuration
    config: DstConfig,
    /// Slots configuration for validation and mappings
    slots_config: Arc<voice_agent_config::domain::SlotsConfig>,
    /// Domain view for config-driven instructions (optional)
    domain_view: Option<Arc<AgentDomainView>>,
}

impl DialogueStateTracker {
    /// Create a new dialogue state tracker with config
    pub fn new(
        state: DynamicDialogueState,
        slots_config: Arc<voice_agent_config::domain::SlotsConfig>,
    ) -> Self {
        Self {
            state,
            history: Vec::new(),
            config: DstConfig::default(),
            slots_config,
            domain_view: None,
        }
    }

    /// Create with custom DST configuration
    pub fn with_dst_config(
        state: DynamicDialogueState,
        slots_config: Arc<voice_agent_config::domain::SlotsConfig>,
        dst_config: DstConfig,
    ) -> Self {
        Self {
            state,
            history: Vec::new(),
            config: dst_config,
            slots_config,
            domain_view: None,
        }
    }

    /// Create with default state from slots config
    pub fn from_config(slots_config: Arc<voice_agent_config::domain::SlotsConfig>) -> Self {
        let state = DynamicDialogueState::from_config(slots_config.clone());
        Self {
            state,
            history: Vec::new(),
            config: DstConfig::default(),
            slots_config,
            domain_view: None,
        }
    }

    /// Create with DST config only (uses default SlotsConfig)
    ///
    /// This is a convenience method for backward compatibility when only
    /// DST parameters are available. Prefer `from_config` or `with_dst_config`
    /// when SlotsConfig is available for proper domain-specific slot validation.
    pub fn with_tracking_config(dst_config: DstConfig) -> Self {
        let slots_config = Arc::new(voice_agent_config::domain::SlotsConfig::default());
        let state = DynamicDialogueState::from_config(slots_config.clone());
        Self {
            state,
            history: Vec::new(),
            config: dst_config,
            slots_config,
            domain_view: None,
        }
    }

    /// Create from domain config (extracts SlotsConfig)
    pub fn from_domain_config(
        domain_config: &voice_agent_config::MasterDomainConfig,
        dst_config: DstConfig,
    ) -> Self {
        let slots_config = Arc::new(domain_config.slots.clone());
        let state = DynamicDialogueState::from_config(slots_config.clone());
        Self {
            state,
            history: Vec::new(),
            config: dst_config,
            slots_config,
            domain_view: None,
        }
    }

    /// Set domain view for config-driven instructions
    pub fn with_domain_view(mut self, view: Arc<AgentDomainView>) -> Self {
        self.domain_view = Some(view);
        self
    }

    /// Set domain view (mutable reference version)
    pub fn set_domain_view(&mut self, view: Arc<AgentDomainView>) {
        self.domain_view = Some(view);
    }

    /// Get current dialogue state
    pub fn state(&self) -> &DynamicDialogueState {
        &self.state
    }

    /// Get mutable state
    pub fn state_mut(&mut self) -> &mut DynamicDialogueState {
        &mut self.state
    }

    /// Get state change history
    pub fn history(&self) -> &[StateChange] {
        &self.history
    }

    /// Get slots configuration
    pub fn slots_config(&self) -> &voice_agent_config::domain::SlotsConfig {
        &self.slots_config
    }

    /// Update state from detected intent
    pub fn update(&mut self, intent: &DetectedIntent) {
        let turn_index = self.history.len();

        // Check for corrections first
        if self.config.enable_corrections {
            self.detect_and_apply_corrections(&intent.slots, turn_index);
        }

        // Update from extracted slots
        for (slot_name, slot) in &intent.slots {
            if slot.confidence >= self.config.min_slot_confidence {
                if let Some(ref value) = slot.value {
                    self.update_slot(slot_name, value, slot.confidence, ChangeSource::UserUtterance, turn_index);
                }
            }
        }

        // Update primary intent
        self.state.update_intent(&intent.intent, intent.confidence);

        // Check for auto-confirmation
        self.check_auto_confirmations();
    }

    /// Update a specific slot
    pub fn update_slot(
        &mut self,
        slot_name: &str,
        value: &str,
        confidence: f32,
        source: ChangeSource,
        turn_index: usize,
    ) {
        let old_value = self.state.get_slot_value(slot_name);

        // Skip if value unchanged
        if old_value.as_ref().map(|v| v.as_str()) == Some(value) {
            return;
        }

        // Record change
        self.history.push(StateChange {
            timestamp: Utc::now(),
            slot_name: slot_name.to_string(),
            old_value: old_value.clone(),
            new_value: Some(value.to_string()),
            confidence,
            source,
            turn_index,
        });

        // Apply change to state
        self.state.set_slot_value(slot_name, value, confidence);

        // Mark as pending confirmation if not auto-confirmed
        if confidence < self.config.auto_confirm_confidence {
            self.state.mark_pending(slot_name);
        } else {
            self.state.mark_confirmed(slot_name);
        }

        tracing::debug!(
            slot = slot_name,
            old_value = ?old_value,
            new_value = value,
            confidence = confidence,
            "Slot updated"
        );
    }

    /// Confirm a slot value
    pub fn confirm_slot(&mut self, slot_name: &str) {
        self.state.mark_confirmed(slot_name);

        self.history.push(StateChange {
            timestamp: Utc::now(),
            slot_name: slot_name.to_string(),
            old_value: self.state.get_slot_value(slot_name),
            new_value: self.state.get_slot_value(slot_name),
            confidence: 1.0,
            source: ChangeSource::SystemConfirmation,
            turn_index: self.history.len(),
        });
    }

    /// Clear a slot value
    pub fn clear_slot(&mut self, slot_name: &str) {
        let old_value = self.state.get_slot_value(slot_name);
        self.state.clear_slot(slot_name);

        self.history.push(StateChange {
            timestamp: Utc::now(),
            slot_name: slot_name.to_string(),
            old_value,
            new_value: None,
            confidence: 1.0,
            source: ChangeSource::UserUtterance,
            turn_index: self.history.len(),
        });
    }

    /// Detect and apply corrections
    fn detect_and_apply_corrections(
        &mut self,
        new_slots: &HashMap<String, Slot>,
        turn_index: usize,
    ) {
        for (slot_name, new_slot) in new_slots {
            if let Some(ref new_value) = new_slot.value {
                // Check if this slot was recently set with a different value
                let recent_changes: Vec<_> = self.history
                    .iter()
                    .rev()
                    .take(self.config.correction_lookback)
                    .filter(|c| c.slot_name == *slot_name)
                    .collect();

                if let Some(recent) = recent_changes.first() {
                    if recent.new_value.as_ref() != Some(new_value) {
                        tracing::debug!(
                            slot = slot_name,
                            old = ?recent.new_value,
                            new = new_value,
                            "Detected slot correction"
                        );

                        self.update_slot(
                            slot_name,
                            new_value,
                            new_slot.confidence.max(0.9),
                            ChangeSource::Correction,
                            turn_index,
                        );
                    }
                }
            }
        }
    }

    /// Check and apply auto-confirmations
    fn check_auto_confirmations(&mut self) {
        let pending: Vec<String> = self.state.pending_slots().iter().cloned().collect();

        for slot_name in pending {
            if let Some(slot_value) = self.state.get_slot_with_confidence(&slot_name) {
                if slot_value.confidence >= self.config.auto_confirm_confidence {
                    self.state.mark_confirmed(&slot_name);
                }
            }
        }
    }

    /// Get slots that need confirmation
    pub fn slots_needing_confirmation(&self) -> Vec<&str> {
        self.state.pending_slots().iter().map(|s| s.as_str()).collect()
    }

    /// Get confirmed slots
    pub fn confirmed_slots(&self) -> Vec<&str> {
        self.state.confirmed_slots().iter().map(|s| s.as_str()).collect()
    }

    /// Check if all required slots for an intent are filled (config-driven)
    pub fn is_intent_complete(&self, intent: &str) -> bool {
        let goal_id = self.slots_config.goal_for_intent(intent).unwrap_or(intent);

        if let Some(goal) = self.slots_config.get_goal(goal_id) {
            return goal.required_slots.iter().all(|slot| {
                self.state.get_slot_value(slot).is_some()
            });
        }

        true
    }

    /// Get missing required slots for an intent (config-driven)
    pub fn missing_slots_for_intent(&self, intent: &str) -> Vec<&str> {
        let goal_id = self.slots_config.goal_for_intent(intent).unwrap_or(intent);

        if let Some(goal) = self.slots_config.get_goal(goal_id) {
            return goal.required_slots
                .iter()
                .filter(|slot| self.state.get_slot_value(slot).is_none())
                .map(|s| s.as_str())
                .collect();
        }

        Vec::new()
    }

    /// Generate a prompt context from current state
    pub fn state_context(&self) -> String {
        self.state.to_context_string()
    }

    /// Generate full context including goal information
    pub fn full_context(&self) -> String {
        self.state.to_full_context_string()
    }

    /// Get current conversation goal ID
    pub fn goal_id(&self) -> &str {
        self.state.goal_id()
    }

    /// Update goal from detected intent (config-driven)
    pub fn update_goal_from_intent(&mut self, intent: &str, turn: usize) {
        if let Some(goal_id) = self.slots_config.goal_for_intent(intent) {
            self.state.set_goal(goal_id, turn);
        } else if intent != "unknown" && intent != "exploration" {
            self.state.set_goal(intent, turn);
        }
    }

    /// Set goal explicitly
    pub fn set_goal(&mut self, goal_id: &str, turn: usize) {
        self.state.set_goal(goal_id, turn);
    }

    /// Confirm goal (user explicitly stated it)
    pub fn confirm_goal(&mut self, goal_id: &str, turn: usize) {
        self.state.confirm_goal(goal_id, turn);
    }

    /// Check if we should auto-capture lead
    pub fn should_auto_capture_lead(&self) -> bool {
        self.state.should_auto_capture_lead()
    }

    /// Get instruction for an action (config-driven if domain view available)
    pub fn instruction_for_action(&self, action: &NextBestAction, language: &str) -> String {
        if let Some(ref view) = self.domain_view {
            let action_type = match action {
                NextBestAction::ExplainProcess => "explain_process",
                NextBestAction::DiscoverIntent => "discover_intent",
                NextBestAction::OfferAppointment => "offer_appointment",
                NextBestAction::CaptureLead => "capture_lead",
                _ => "",
            };

            if !action_type.is_empty() {
                if let Some(instruction) = view.dst_instruction(action_type, language) {
                    return instruction.to_string();
                }
            }
        }

        action.to_instruction_default()
    }

    /// Get prompt to ask for a missing slot (config-driven)
    pub fn slot_prompt(&self, slot_name: &str, language: &str) -> String {
        if let Some(slot_def) = self.slots_config.get_slot(slot_name) {
            if !slot_def.description.is_empty() {
                let prefix = if language == "hi" { "कृपया बताएं" } else { "Please provide" };
                return format!("{} {}.", prefix, slot_def.description.to_lowercase());
            }
        }

        let slot_display = slot_name.replace('_', " ");
        if language == "hi" {
            format!("कृपया अपना {} बताएं।", slot_display)
        } else {
            format!("Please provide your {}.", slot_display)
        }
    }

    /// Get completion action for current goal
    pub fn completion_action_for_goal(&self, goal_id: &str) -> Option<&str> {
        self.slots_config
            .get_goal(goal_id)
            .and_then(|g| g.completion_action.as_deref())
    }

    /// Reset the tracker
    pub fn reset(&mut self) {
        self.state = DynamicDialogueState::from_config(self.slots_config.clone());
        self.history.clear();
    }
}

// =============================================================================
// DialogueStateTracking Trait (for generic code)
// =============================================================================

/// Trait for dialogue state tracking operations
///
/// Allows writing code that works with any tracker implementation.
pub trait DialogueStateTracking: Send + Sync {
    /// Type of dialogue state managed by this tracker
    type State: DialogueStateTrait;

    /// Get current dialogue state (immutable)
    fn state(&self) -> &Self::State;

    /// Get current dialogue state (mutable)
    fn state_mut(&mut self) -> &mut Self::State;

    /// Get state change history
    fn history(&self) -> &[StateChange];

    /// Update state from detected intent
    fn update(&mut self, intent: &DetectedIntent);

    /// Update a specific slot
    fn update_slot(
        &mut self,
        slot_name: &str,
        value: &str,
        confidence: f32,
        source: ChangeSource,
        turn_index: usize,
    );

    /// Confirm a slot value
    fn confirm_slot(&mut self, slot_name: &str);

    /// Clear a slot value
    fn clear_slot(&mut self, slot_name: &str);

    /// Get slots needing confirmation
    fn slots_needing_confirmation(&self) -> Vec<&str>;

    /// Get confirmed slots
    fn confirmed_slots(&self) -> Vec<&str>;

    /// Check if all required slots for an intent are filled
    fn is_intent_complete(&self, intent: &str) -> bool;

    /// Get missing required slots for an intent
    fn missing_slots_for_intent(&self, intent: &str) -> Vec<&str>;

    /// Generate prompt context from current state
    fn state_context(&self) -> String;

    /// Generate full context including goal information
    fn full_context(&self) -> String;

    /// Get current conversation goal ID
    fn goal_id(&self) -> &str;

    /// Update goal from detected intent
    fn update_goal_from_intent(&mut self, intent: &str, turn: usize);

    /// Set goal explicitly
    fn set_goal(&mut self, goal_id: &str, turn: usize);

    /// Confirm goal
    fn confirm_goal(&mut self, goal_id: &str, turn: usize);

    /// Check if we should auto-capture lead
    fn should_auto_capture_lead(&self) -> bool;

    /// Reset the tracker
    fn reset(&mut self);

    /// Get instruction for an action
    fn instruction_for_action(&self, action: &NextBestAction, language: &str) -> String;
}

impl DialogueStateTracking for DialogueStateTracker {
    type State = DynamicDialogueState;

    fn state(&self) -> &Self::State {
        &self.state
    }

    fn state_mut(&mut self) -> &mut Self::State {
        &mut self.state
    }

    fn history(&self) -> &[StateChange] {
        &self.history
    }

    fn update(&mut self, intent: &DetectedIntent) {
        DialogueStateTracker::update(self, intent)
    }

    fn update_slot(
        &mut self,
        slot_name: &str,
        value: &str,
        confidence: f32,
        source: ChangeSource,
        turn_index: usize,
    ) {
        DialogueStateTracker::update_slot(self, slot_name, value, confidence, source, turn_index)
    }

    fn confirm_slot(&mut self, slot_name: &str) {
        DialogueStateTracker::confirm_slot(self, slot_name)
    }

    fn clear_slot(&mut self, slot_name: &str) {
        DialogueStateTracker::clear_slot(self, slot_name)
    }

    fn slots_needing_confirmation(&self) -> Vec<&str> {
        DialogueStateTracker::slots_needing_confirmation(self)
    }

    fn confirmed_slots(&self) -> Vec<&str> {
        DialogueStateTracker::confirmed_slots(self)
    }

    fn is_intent_complete(&self, intent: &str) -> bool {
        DialogueStateTracker::is_intent_complete(self, intent)
    }

    fn missing_slots_for_intent(&self, intent: &str) -> Vec<&str> {
        DialogueStateTracker::missing_slots_for_intent(self, intent)
    }

    fn state_context(&self) -> String {
        DialogueStateTracker::state_context(self)
    }

    fn full_context(&self) -> String {
        DialogueStateTracker::full_context(self)
    }

    fn goal_id(&self) -> &str {
        DialogueStateTracker::goal_id(self)
    }

    fn update_goal_from_intent(&mut self, intent: &str, turn: usize) {
        DialogueStateTracker::update_goal_from_intent(self, intent, turn)
    }

    fn set_goal(&mut self, goal_id: &str, turn: usize) {
        DialogueStateTracker::set_goal(self, goal_id, turn)
    }

    fn confirm_goal(&mut self, goal_id: &str, turn: usize) {
        DialogueStateTracker::confirm_goal(self, goal_id, turn)
    }

    fn should_auto_capture_lead(&self) -> bool {
        DialogueStateTracker::should_auto_capture_lead(self)
    }

    fn reset(&mut self) {
        DialogueStateTracker::reset(self)
    }

    fn instruction_for_action(&self, action: &NextBestAction, language: &str) -> String {
        DialogueStateTracker::instruction_for_action(self, action, language)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Arc<voice_agent_config::domain::SlotsConfig> {
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
    description: "Asset weight in grams"
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
    fn test_tracker_creation() {
        let config = create_test_config();
        let tracker = DialogueStateTracker::from_config(config);
        assert!(tracker.state().customer_name().is_none());
        assert!(tracker.history().is_empty());
    }

    #[test]
    fn test_slot_update() {
        let config = create_test_config();
        let mut tracker = DialogueStateTracker::from_config(config);

        tracker.update_slot("customer_name", "Rahul", 0.9, ChangeSource::UserUtterance, 0);

        assert_eq!(tracker.state().customer_name(), Some("Rahul"));
        assert_eq!(tracker.history().len(), 1);
    }

    #[test]
    fn test_slot_correction() {
        let config = create_test_config();
        let state = DynamicDialogueState::from_config(config.clone());
        let mut tracker = DialogueStateTracker::with_dst_config(
            state,
            config,
            DstConfig {
                enable_corrections: true,
                ..Default::default()
            },
        );

        tracker.update_slot("gold_weight", "40", 0.8, ChangeSource::UserUtterance, 0);
        assert_eq!(tracker.state().get_slot_value("gold_weight"), Some("40".to_string()));

        tracker.update_slot("gold_weight", "50", 0.9, ChangeSource::Correction, 1);
        assert_eq!(tracker.state().get_slot_value("gold_weight"), Some("50".to_string()));
        assert_eq!(tracker.history().len(), 2);
    }

    #[test]
    fn test_confirmation_tracking() {
        let config = create_test_config();
        let state = DynamicDialogueState::from_config(config.clone());
        let mut tracker = DialogueStateTracker::with_dst_config(
            state,
            config,
            DstConfig {
                auto_confirm_confidence: 0.95,
                ..Default::default()
            },
        );

        tracker.update_slot("loan_amount", "500000", 0.8, ChangeSource::UserUtterance, 0);
        assert!(tracker.state().pending_slots().contains(&"loan_amount".to_string()));

        tracker.confirm_slot("loan_amount");
        assert!(tracker.state().confirmed_slots().contains(&"loan_amount".to_string()));
    }

    #[test]
    fn test_auto_confirmation() {
        let config = create_test_config();
        let state = DynamicDialogueState::from_config(config.clone());
        let mut tracker = DialogueStateTracker::with_dst_config(
            state,
            config,
            DstConfig {
                auto_confirm_confidence: 0.9,
                ..Default::default()
            },
        );

        tracker.update_slot("loan_amount", "500000", 0.95, ChangeSource::UserUtterance, 0);
        assert!(tracker.state().confirmed_slots().contains(&"loan_amount".to_string()));
    }

    #[test]
    fn test_missing_slots_detection() {
        let config = create_test_config();
        let tracker = DialogueStateTracker::from_config(config);

        let missing = tracker.missing_slots_for_intent("eligibility_check");
        assert!(missing.contains(&"gold_weight"));
    }

    #[test]
    fn test_intent_completeness() {
        let config = create_test_config();
        let mut tracker = DialogueStateTracker::from_config(config);

        assert!(!tracker.is_intent_complete("eligibility_check"));

        tracker.update_slot("gold_weight", "50", 0.9, ChangeSource::UserUtterance, 0);
        assert!(tracker.is_intent_complete("eligibility_check"));
    }

    #[test]
    fn test_state_context() {
        let config = create_test_config();
        let mut tracker = DialogueStateTracker::from_config(config);

        tracker.update_slot("customer_name", "Rahul", 0.9, ChangeSource::UserUtterance, 0);
        tracker.update_slot("loan_amount", "500000", 0.9, ChangeSource::UserUtterance, 1);

        let context = tracker.state_context();
        assert!(context.contains("Rahul"));
    }

    #[test]
    fn test_goal_from_intent() {
        let config = create_test_config();
        let mut tracker = DialogueStateTracker::from_config(config);

        tracker.update_goal_from_intent("switch_lender", 0);
        assert_eq!(tracker.goal_id(), "balance_transfer");
    }

    #[test]
    fn test_completion_action() {
        let config = create_test_config();
        let tracker = DialogueStateTracker::from_config(config);

        assert_eq!(
            tracker.completion_action_for_goal("balance_transfer"),
            Some("calculate_savings")
        );
    }
}
