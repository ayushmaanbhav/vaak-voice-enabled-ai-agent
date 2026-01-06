//! Dialogue State Tracking (DST) for Gold Loan Conversations
//!
//! Implements domain-specific dialogue state tracking based on LDST and ACL 2024 research.
//!
//! # Features
//!
//! - Domain-slot based state tracking
//! - Multi-turn slot value tracking
//! - Slot correction handling ("actually, it's 50 grams, not 40")
//! - Slot confirmation tracking
//! - Confidence-based slot updates
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_agent::dst::{DialogueStateTracker, GoldLoanDialogueState};
//! use voice_agent_text_processing::intent::IntentDetector;
//!
//! let detector = IntentDetector::new();
//! let mut tracker = DialogueStateTracker::new();
//!
//! // User: "I want a gold loan of 5 lakh"
//! let intent = detector.detect("I want a gold loan of 5 lakh");
//! tracker.update(&intent);
//!
//! // State now contains loan_amount = 500000
//! assert_eq!(tracker.state().loan_amount(), Some(500000.0));
//! ```

pub mod slots;
pub mod extractor;

pub use slots::{GoldLoanDialogueState, GoldPurity, SlotValue, UrgencyLevel, ConversationGoal, NextBestAction};
pub use extractor::SlotExtractor;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use voice_agent_text_processing::intent::{DetectedIntent, Slot};

/// Dialogue State Tracker for Gold Loan conversations
pub struct DialogueStateTracker {
    /// Current dialogue state
    state: GoldLoanDialogueState,
    /// History of state changes
    history: Vec<StateChange>,
    /// Slot extractor for enhanced extraction
    extractor: SlotExtractor,
    /// Configuration
    config: DstConfig,
}

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

impl DialogueStateTracker {
    /// Create a new dialogue state tracker
    pub fn new() -> Self {
        Self {
            state: GoldLoanDialogueState::new(),
            history: Vec::new(),
            extractor: SlotExtractor::new(),
            config: DstConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: DstConfig) -> Self {
        Self {
            state: GoldLoanDialogueState::new(),
            history: Vec::new(),
            extractor: SlotExtractor::new(),
            config,
        }
    }

    /// Get current dialogue state
    pub fn state(&self) -> &GoldLoanDialogueState {
        &self.state
    }

    /// Get mutable state
    pub fn state_mut(&mut self) -> &mut GoldLoanDialogueState {
        &mut self.state
    }

    /// Get state change history
    pub fn history(&self) -> &[StateChange] {
        &self.history
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
        // Look for correction patterns in the new slots
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
                        // This looks like a correction
                        tracing::debug!(
                            slot = slot_name,
                            old = ?recent.new_value,
                            new = new_value,
                            "Detected slot correction"
                        );

                        // Apply with correction source (higher priority)
                        self.update_slot(
                            slot_name,
                            new_value,
                            new_slot.confidence.max(0.9), // Boost confidence for corrections
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
            // Check if we have high confidence
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

    /// Check if all required slots for an intent are filled
    pub fn is_intent_complete(&self, intent: &str) -> bool {
        match intent {
            "eligibility_check" => self.state.gold_weight_grams().is_some(),
            "switch_lender" => self.state.current_lender().is_some(),
            "schedule_visit" => self.state.location().is_some(),
            "send_sms" => self.state.phone_number().is_some(),
            _ => true, // Most intents don't have required slots
        }
    }

    /// Get missing required slots for an intent
    pub fn missing_slots_for_intent(&self, intent: &str) -> Vec<&str> {
        match intent {
            "eligibility_check" => {
                let mut missing = Vec::new();
                if self.state.gold_weight_grams().is_none() {
                    missing.push("gold_weight");
                }
                missing
            }
            "switch_lender" => {
                let mut missing = Vec::new();
                if self.state.current_lender().is_none() {
                    missing.push("current_lender");
                }
                missing
            }
            "schedule_visit" => {
                let mut missing = Vec::new();
                if self.state.location().is_none() {
                    missing.push("location");
                }
                missing
            }
            "send_sms" => {
                let mut missing = Vec::new();
                if self.state.phone_number().is_none() {
                    missing.push("phone_number");
                }
                missing
            }
            _ => Vec::new(),
        }
    }

    /// Generate a prompt context from current state
    pub fn state_context(&self) -> String {
        self.state.to_context_string()
    }

    /// Generate full context including goal information
    pub fn full_context(&self) -> String {
        self.state.to_full_context_string()
    }

    /// Get current conversation goal
    pub fn conversation_goal(&self) -> ConversationGoal {
        self.state.conversation_goal()
    }

    /// Update goal from detected intent
    pub fn update_goal_from_intent(&mut self, intent: &str, turn: usize) {
        self.state.update_goal_from_intent(intent, turn);
    }

    /// Get the next best action based on current goal and slots
    pub fn get_next_action(&self) -> NextBestAction {
        self.state.get_next_action()
    }

    /// Check if we should proactively trigger a tool
    pub fn should_trigger_tool(&self) -> Option<String> {
        self.state.should_trigger_tool()
    }

    /// Get the goal context for LLM prompt injection
    pub fn goal_context(&self) -> String {
        self.state.goal_context()
    }

    /// Get missing required slots for current goal
    pub fn missing_required_slots(&self) -> Vec<&'static str> {
        self.state.missing_required_slots()
    }

    /// Get goal completion percentage
    pub fn goal_completion(&self) -> f32 {
        self.state.goal_completion()
    }

    /// Check if we should auto-capture lead (when we have contact info during any goal)
    pub fn should_auto_capture_lead(&self) -> bool {
        self.state.should_auto_capture_lead()
    }

    /// Reset the tracker
    pub fn reset(&mut self) {
        self.state = GoldLoanDialogueState::new();
        self.history.clear();
    }
}

impl Default for DialogueStateTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voice_agent_text_processing::intent::IntentDetector;

    #[test]
    fn test_tracker_creation() {
        let tracker = DialogueStateTracker::new();
        assert!(tracker.state().customer_name().is_none());
        assert!(tracker.history().is_empty());
    }

    #[test]
    fn test_slot_update() {
        let mut tracker = DialogueStateTracker::new();

        tracker.update_slot("customer_name", "Rahul", 0.9, ChangeSource::UserUtterance, 0);

        assert_eq!(tracker.state().customer_name(), Some("Rahul"));
        assert_eq!(tracker.history().len(), 1);
    }

    #[test]
    fn test_intent_update() {
        let mut tracker = DialogueStateTracker::new();
        let detector = IntentDetector::new();

        let intent = detector.detect("I want a gold loan of 5 lakh");
        tracker.update(&intent);

        assert_eq!(tracker.state().primary_intent(), Some("loan_inquiry"));
        assert!(tracker.state().loan_amount().is_some());
    }

    #[test]
    fn test_slot_correction() {
        let mut tracker = DialogueStateTracker::with_config(DstConfig {
            enable_corrections: true,
            ..Default::default()
        });

        // Initial value
        tracker.update_slot("gold_weight", "40", 0.8, ChangeSource::UserUtterance, 0);
        assert_eq!(tracker.state().get_slot_value("gold_weight"), Some("40".to_string()));

        // Correction
        tracker.update_slot("gold_weight", "50", 0.9, ChangeSource::Correction, 1);
        assert_eq!(tracker.state().get_slot_value("gold_weight"), Some("50".to_string()));
        assert_eq!(tracker.history().len(), 2);
    }

    #[test]
    fn test_confirmation_tracking() {
        let mut tracker = DialogueStateTracker::with_config(DstConfig {
            auto_confirm_confidence: 0.95,
            ..Default::default()
        });

        // Low confidence - should be pending
        tracker.update_slot("loan_amount", "500000", 0.8, ChangeSource::UserUtterance, 0);
        assert!(tracker.state().pending_slots().contains(&"loan_amount".to_string()));

        // Confirm
        tracker.confirm_slot("loan_amount");
        assert!(tracker.state().confirmed_slots().contains(&"loan_amount".to_string()));
    }

    #[test]
    fn test_auto_confirmation() {
        let mut tracker = DialogueStateTracker::with_config(DstConfig {
            auto_confirm_confidence: 0.9,
            ..Default::default()
        });

        // High confidence - should auto-confirm
        tracker.update_slot("loan_amount", "500000", 0.95, ChangeSource::UserUtterance, 0);
        assert!(tracker.state().confirmed_slots().contains(&"loan_amount".to_string()));
    }

    #[test]
    fn test_missing_slots_detection() {
        let tracker = DialogueStateTracker::new();

        let missing = tracker.missing_slots_for_intent("eligibility_check");
        assert!(missing.contains(&"gold_weight"));
    }

    #[test]
    fn test_intent_completeness() {
        let mut tracker = DialogueStateTracker::new();

        assert!(!tracker.is_intent_complete("eligibility_check"));

        tracker.update_slot("gold_weight", "50", 0.9, ChangeSource::UserUtterance, 0);
        assert!(tracker.is_intent_complete("eligibility_check"));
    }

    #[test]
    fn test_state_context() {
        let mut tracker = DialogueStateTracker::new();

        tracker.update_slot("customer_name", "Rahul", 0.9, ChangeSource::UserUtterance, 0);
        tracker.update_slot("loan_amount", "500000", 0.9, ChangeSource::UserUtterance, 1);

        let context = tracker.state_context();
        assert!(context.contains("Rahul"));
        // Loan amount is formatted as "5.0 lakh" in context
        assert!(context.contains("5.0 lakh") || context.contains("500000"));
    }
}
