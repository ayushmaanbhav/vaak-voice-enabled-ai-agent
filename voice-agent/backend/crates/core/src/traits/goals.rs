//! Conversation Goal Schema trait for dynamic goal definitions
//!
//! This module provides a domain-agnostic interface for conversation goals,
//! including required/optional slots, completion actions, and next-best-action logic.
//! All goal definitions are loaded from configuration.
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_core::traits::ConversationGoalSchema;
//!
//! // Schema is created from domain config
//! let schema = config_bridge.goal_schema();
//!
//! // Get next action for current goal
//! let action = schema.next_action_for_goal("balance_transfer", &filled_slots);
//! ```

use std::collections::HashMap;

/// Goal completion status
#[derive(Debug, Clone, PartialEq)]
pub enum GoalCompletionStatus {
    /// All required slots filled
    Complete,
    /// Some required slots missing
    Incomplete {
        missing_slots: Vec<String>,
        completion_percent: f32,
    },
    /// Goal not applicable
    NotApplicable,
}

/// Next action recommendation
#[derive(Debug, Clone)]
pub enum NextAction {
    /// Ask user for a specific slot
    AskForSlot {
        slot_name: String,
        prompt_template: String,
    },
    /// Call a tool with current slot values
    CallTool {
        tool_name: String,
        /// Slot values to pass as arguments
        arguments: HashMap<String, String>,
    },
    /// Confirm details before proceeding
    ConfirmDetails {
        summary: String,
    },
    /// Goal is complete, proceed to next stage
    Complete {
        completion_message: String,
    },
    /// Explain the process to the user
    ExplainProcess {
        explanation: String,
    },
    /// Offer to schedule appointment
    OfferAppointment,
    /// Transition to another goal
    TransitionGoal {
        new_goal_id: String,
    },
}

/// Goal definition trait
///
/// Defines a conversation goal with required slots, optional slots,
/// and completion actions. All loaded from config.
pub trait GoalDefinition: Send + Sync {
    /// Goal identifier (e.g., "balance_transfer", "eligibility_check")
    fn id(&self) -> &str;

    /// Human-readable display name
    fn display_name(&self) -> &str;

    /// Goal description
    fn description(&self) -> &str;

    /// Required slots that must be filled
    fn required_slots(&self) -> &[String];

    /// Optional slots for enhanced service
    fn optional_slots(&self) -> &[String];

    /// Tool to call when goal is complete
    fn completion_tool(&self) -> Option<&str>;

    /// Check if goal is complete given filled slots
    fn is_complete(&self, filled_slots: &HashMap<String, String>) -> bool {
        self.required_slots()
            .iter()
            .all(|slot| filled_slots.contains_key(slot))
    }

    /// Get missing required slots
    fn missing_slots(&self, filled_slots: &HashMap<String, String>) -> Vec<&str> {
        self.required_slots()
            .iter()
            .filter(|slot| !filled_slots.contains_key(*slot))
            .map(|s| s.as_str())
            .collect()
    }

    /// Get completion percentage
    fn completion_percentage(&self, filled_slots: &HashMap<String, String>) -> f32 {
        let required = self.required_slots();
        if required.is_empty() {
            return 100.0;
        }

        let filled_count = required
            .iter()
            .filter(|slot| filled_slots.contains_key(*slot))
            .count();

        (filled_count as f32 / required.len() as f32) * 100.0
    }

    /// Get completion status
    fn completion_status(&self, filled_slots: &HashMap<String, String>) -> GoalCompletionStatus {
        let missing = self.missing_slots(filled_slots);
        if missing.is_empty() {
            GoalCompletionStatus::Complete
        } else {
            GoalCompletionStatus::Incomplete {
                missing_slots: missing.iter().map(|s| s.to_string()).collect(),
                completion_percent: self.completion_percentage(filled_slots),
            }
        }
    }

    /// Priority level (lower = higher priority)
    fn priority(&self) -> u8 {
        50 // Default middle priority
    }
}

/// Conversation goal schema manager trait
///
/// Manages all goal definitions and provides intent-to-goal mapping
/// and next-best-action logic.
pub trait ConversationGoalSchema: Send + Sync {
    /// Get goal definition by ID
    fn get_goal(&self, id: &str) -> Option<&dyn GoalDefinition>;

    /// Get all goal IDs
    fn goal_ids(&self) -> Vec<&str>;

    /// Get all goal definitions
    fn all_goals(&self) -> Vec<&dyn GoalDefinition>;

    /// Map intent to goal
    ///
    /// Returns the goal ID for a detected intent.
    fn goal_for_intent(&self, intent: &str) -> Option<&str>;

    /// Get next-best-action for a goal
    ///
    /// Determines what the agent should do next based on:
    /// - Which slots are filled
    /// - Which slots are missing
    /// - Whether the goal is complete
    fn next_action_for_goal(
        &self,
        goal_id: &str,
        filled_slots: &HashMap<String, String>,
    ) -> NextAction;

    /// Suggest tool to call when goal is complete
    fn suggest_tool_for_goal(&self, goal_id: &str) -> Option<&str>;

    /// Get prompt template for asking a slot
    fn slot_prompt_template(&self, goal_id: &str, slot_name: &str) -> Option<&str>;

    /// Detect goal from filled slots
    ///
    /// Infers the most likely goal based on which slots have been filled.
    fn detect_goal_from_slots(&self, filled_slots: &HashMap<String, String>) -> Option<&str>;

    /// Get default goal (exploration)
    fn default_goal(&self) -> &str;
}

/// Config-driven goal definition
#[derive(Debug, Clone)]
pub struct ConfigGoalDefinition {
    id: String,
    display_name: String,
    description: String,
    required_slots: Vec<String>,
    optional_slots: Vec<String>,
    completion_tool: Option<String>,
    priority: u8,
    slot_prompts: HashMap<String, String>,
}

impl ConfigGoalDefinition {
    /// Create a new goal definition
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            description: description.into(),
            required_slots: Vec::new(),
            optional_slots: Vec::new(),
            completion_tool: None,
            priority: 50,
            slot_prompts: HashMap::new(),
        }
    }

    /// Set required slots
    pub fn with_required_slots(mut self, slots: Vec<String>) -> Self {
        self.required_slots = slots;
        self
    }

    /// Set optional slots
    pub fn with_optional_slots(mut self, slots: Vec<String>) -> Self {
        self.optional_slots = slots;
        self
    }

    /// Set completion tool
    pub fn with_completion_tool(mut self, tool: impl Into<String>) -> Self {
        self.completion_tool = Some(tool.into());
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Add slot prompt template
    pub fn with_slot_prompt(mut self, slot: impl Into<String>, prompt: impl Into<String>) -> Self {
        self.slot_prompts.insert(slot.into(), prompt.into());
        self
    }

    /// Get slot prompt
    pub fn get_slot_prompt(&self, slot: &str) -> Option<&str> {
        self.slot_prompts.get(slot).map(|s| s.as_str())
    }

    // NOTE: Domain-specific factory methods (balance_transfer, new_loan, eligibility_check,
    // branch_visit, lead_capture) have been removed. Use config-driven goals from
    // config/domains/{domain}/goals.yaml via DomainBridge instead.
    //
    // Only exploration() is kept as it's a generic default goal for all domains.

    /// Create exploration goal (default).
    /// This is a generic goal that applies to all domains.
    pub fn exploration() -> Self {
        Self::new("exploration", "Exploration", "General exploration of options")
            .with_priority(100) // Lowest priority
    }
}

impl GoalDefinition for ConfigGoalDefinition {
    fn id(&self) -> &str {
        &self.id
    }

    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn required_slots(&self) -> &[String] {
        &self.required_slots
    }

    fn optional_slots(&self) -> &[String] {
        &self.optional_slots
    }

    fn completion_tool(&self) -> Option<&str> {
        self.completion_tool.as_deref()
    }

    fn priority(&self) -> u8 {
        self.priority
    }
}

/// Config-driven goal schema implementation
pub struct ConfigGoalSchema {
    goals: HashMap<String, ConfigGoalDefinition>,
    intent_mapping: HashMap<String, String>,
    default_goal: String,
}

impl ConfigGoalSchema {
    /// Create a new goal schema
    pub fn new(goals: Vec<ConfigGoalDefinition>, intent_mapping: HashMap<String, String>) -> Self {
        let goal_map = goals
            .into_iter()
            .map(|g| (g.id.clone(), g))
            .collect();

        Self {
            goals: goal_map,
            intent_mapping,
            default_goal: "exploration".to_string(),
        }
    }

}

impl ConversationGoalSchema for ConfigGoalSchema {
    fn get_goal(&self, id: &str) -> Option<&dyn GoalDefinition> {
        self.goals.get(id).map(|g| g as &dyn GoalDefinition)
    }

    fn goal_ids(&self) -> Vec<&str> {
        self.goals.keys().map(|s| s.as_str()).collect()
    }

    fn all_goals(&self) -> Vec<&dyn GoalDefinition> {
        self.goals.values().map(|g| g as &dyn GoalDefinition).collect()
    }

    fn goal_for_intent(&self, intent: &str) -> Option<&str> {
        self.intent_mapping.get(intent).map(|s| s.as_str())
    }

    fn next_action_for_goal(
        &self,
        goal_id: &str,
        filled_slots: &HashMap<String, String>,
    ) -> NextAction {
        let Some(goal) = self.goals.get(goal_id) else {
            return NextAction::ExplainProcess {
                explanation: "Let me help you understand our services.".to_string(),
            };
        };

        // PRESERVED: Exact logic from slots.rs:108-174
        let missing = goal.missing_slots(filled_slots);

        if missing.is_empty() {
            // Goal complete - call tool
            if let Some(tool) = goal.completion_tool() {
                let mut arguments = HashMap::new();
                for slot in goal.required_slots() {
                    if let Some(value) = filled_slots.get(slot) {
                        arguments.insert(slot.clone(), value.clone());
                    }
                }
                for slot in goal.optional_slots() {
                    if let Some(value) = filled_slots.get(slot) {
                        arguments.insert(slot.clone(), value.clone());
                    }
                }
                return NextAction::CallTool {
                    tool_name: tool.to_string(),
                    arguments,
                };
            }
            return NextAction::Complete {
                completion_message: "I have all the information I need.".to_string(),
            };
        }

        // Ask for first missing slot
        let first_missing = missing[0];
        let prompt = goal
            .get_slot_prompt(first_missing)
            .unwrap_or("Could you please provide this information?")
            .to_string();

        NextAction::AskForSlot {
            slot_name: first_missing.to_string(),
            prompt_template: prompt,
        }
    }

    fn suggest_tool_for_goal(&self, goal_id: &str) -> Option<&str> {
        self.goals.get(goal_id).and_then(|g| g.completion_tool())
    }

    fn slot_prompt_template(&self, goal_id: &str, slot_name: &str) -> Option<&str> {
        self.goals
            .get(goal_id)
            .and_then(|g| g.get_slot_prompt(slot_name))
    }

    fn detect_goal_from_slots(&self, filled_slots: &HashMap<String, String>) -> Option<&str> {
        // Find goal with most matching required slots
        let mut best_goal: Option<&str> = None;
        let mut best_score = 0;

        for (id, goal) in &self.goals {
            let score = goal
                .required_slots()
                .iter()
                .filter(|s| filled_slots.contains_key(*s))
                .count();

            if score > best_score {
                best_score = score;
                best_goal = Some(id.as_str());
            }
        }

        best_goal
    }

    fn default_goal(&self) -> &str {
        &self.default_goal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a test goal using the builder pattern
    fn create_test_goal(id: &str, required_slots: Vec<&str>, tool: Option<&str>) -> ConfigGoalDefinition {
        let mut goal = ConfigGoalDefinition::new(id, id, format!("Test goal: {}", id))
            .with_required_slots(required_slots.into_iter().map(String::from).collect());
        if let Some(t) = tool {
            goal = goal.with_completion_tool(t);
        }
        goal
    }

    /// Create test goal schema with generic test goals
    fn test_schema() -> ConfigGoalSchema {
        let goals = vec![
            ConfigGoalDefinition::exploration(),
            create_test_goal("goal_a", vec!["slot_1", "slot_2"], Some("tool_a")),
            create_test_goal("goal_b", vec!["slot_1"], Some("tool_b")),
            create_test_goal("goal_c", vec!["slot_3"], None),
        ];

        let intent_mapping = [
            ("intent_a", "goal_a"),
            ("intent_a_alt", "goal_a"),
            ("intent_b", "goal_b"),
            ("intent_c", "goal_c"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

        ConfigGoalSchema::new(goals, intent_mapping)
    }

    #[test]
    fn test_goal_builder() {
        let goal = ConfigGoalDefinition::new("test_goal", "Test Goal", "A test goal")
            .with_required_slots(vec!["slot_a".to_string(), "slot_b".to_string()])
            .with_optional_slots(vec!["slot_c".to_string()])
            .with_completion_tool("test_tool")
            .with_priority(10)
            .with_slot_prompt("slot_a", "Please provide slot A");

        assert_eq!(goal.id(), "test_goal");
        assert_eq!(goal.required_slots(), &["slot_a", "slot_b"]);
        assert_eq!(goal.optional_slots(), &["slot_c"]);
        assert_eq!(goal.completion_tool(), Some("test_tool"));
        assert_eq!(goal.priority(), 10);
        assert_eq!(goal.get_slot_prompt("slot_a"), Some("Please provide slot A"));
    }

    #[test]
    fn test_goal_completion() {
        let goal = create_test_goal("test", vec!["required_slot"], Some("tool"));

        let mut slots = HashMap::new();
        assert!(!goal.is_complete(&slots));

        slots.insert("required_slot".to_string(), "value".to_string());
        assert!(goal.is_complete(&slots));
    }

    #[test]
    fn test_intent_mapping() {
        let schema = test_schema();

        assert_eq!(schema.goal_for_intent("intent_a"), Some("goal_a"));
        assert_eq!(schema.goal_for_intent("intent_a_alt"), Some("goal_a"));
        assert_eq!(schema.goal_for_intent("intent_b"), Some("goal_b"));
        assert_eq!(schema.goal_for_intent("unknown_intent"), None);
    }

    #[test]
    fn test_next_action_ask_slot() {
        let schema = test_schema();
        let slots = HashMap::new();

        let action = schema.next_action_for_goal("goal_b", &slots);
        assert!(matches!(action, NextAction::AskForSlot { slot_name, .. } if slot_name == "slot_1"));
    }

    #[test]
    fn test_next_action_call_tool() {
        let schema = test_schema();
        let mut slots = HashMap::new();
        slots.insert("slot_1".to_string(), "value".to_string());

        let action = schema.next_action_for_goal("goal_b", &slots);
        assert!(matches!(action, NextAction::CallTool { tool_name, .. } if tool_name == "tool_b"));
    }

    #[test]
    fn test_next_action_complete_no_tool() {
        let schema = test_schema();
        let mut slots = HashMap::new();
        slots.insert("slot_3".to_string(), "value".to_string());

        let action = schema.next_action_for_goal("goal_c", &slots);
        assert!(matches!(action, NextAction::Complete { .. }));
    }

    #[test]
    fn test_completion_percentage() {
        let goal = create_test_goal("test", vec!["slot_1", "slot_2"], None);
        let mut slots = HashMap::new();

        assert_eq!(goal.completion_percentage(&slots), 0.0);

        slots.insert("slot_1".to_string(), "value".to_string());
        assert_eq!(goal.completion_percentage(&slots), 50.0);

        slots.insert("slot_2".to_string(), "value".to_string());
        assert_eq!(goal.completion_percentage(&slots), 100.0);
    }

    #[test]
    fn test_exploration_goal() {
        let goal = ConfigGoalDefinition::exploration();
        assert_eq!(goal.id(), "exploration");
        assert_eq!(goal.priority(), 100); // Lowest priority
        assert!(goal.required_slots().is_empty());
    }

    #[test]
    fn test_default_goal() {
        let schema = test_schema();
        assert_eq!(schema.default_goal(), "exploration");
    }
}
