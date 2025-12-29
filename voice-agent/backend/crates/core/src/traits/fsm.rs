//! Conversation Finite State Machine trait
//!
//! Provides the interface for managing conversation state transitions,
//! checkpointing, and action generation.
//!
//! # Design Philosophy
//!
//! The FSM trait enables:
//! - Predictable conversation flow with clear state transitions
//! - Checkpointing for error recovery
//! - Event-driven architecture for loose coupling
//! - Metrics collection per state

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::conversation::ConversationStage;

/// Events that trigger state transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConversationEvent {
    // Lifecycle events
    /// Call started with optional customer ID
    CallStarted { customer_id: Option<String> },
    /// Call ended
    CallEnded,

    // Speech events
    /// User started speaking
    UserSpeaking,
    /// User silence detected
    UserSilence { duration_ms: u64 },
    /// Transcript available
    TranscriptReady { text: String, is_final: bool },

    // Agent events
    /// Agent generated response
    ResponseGenerated { text: String },
    /// Response delivered to user
    ResponseDelivered,

    // Intent events
    /// User intent detected
    UserIntent { intent: String, confidence: f32 },
    /// User agreement detected
    UserAgreement,
    /// User refusal detected
    UserRefusal { reason: Option<String> },
    /// User asked a question
    UserQuestion { topic: String },
    /// User raised an objection
    UserObjection { objection_type: ObjectionType },

    // Interrupt events
    /// User interrupted agent (barge-in)
    BargeIn { audio_position_ms: u64 },
    /// Stage timeout occurred
    Timeout { stage: String },

    // Tool events
    /// Tool execution requested
    ToolCallRequested { tool: String, params: serde_json::Value },
    /// Tool execution completed
    ToolResultReady { tool: String, result: serde_json::Value, success: bool },

    // Error events
    /// Error occurred
    Error { code: String, message: String },
}

impl ConversationEvent {
    /// Create a user intent event
    pub fn intent(name: impl Into<String>, confidence: f32) -> Self {
        Self::UserIntent {
            intent: name.into(),
            confidence,
        }
    }

    /// Create a transcript ready event
    pub fn transcript(text: impl Into<String>, is_final: bool) -> Self {
        Self::TranscriptReady {
            text: text.into(),
            is_final,
        }
    }

    /// Create a tool result event
    pub fn tool_result(tool: impl Into<String>, result: serde_json::Value, success: bool) -> Self {
        Self::ToolResultReady {
            tool: tool.into(),
            result,
            success,
        }
    }
}

/// Types of objections customers may raise
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectionType {
    /// Interest rate concerns
    Rate,
    /// Trust/safety concerns
    Trust,
    /// Timing not right
    Timing,
    /// Prefer current lender
    Competition,
    /// Process seems complicated
    Process,
    /// Concerned about gold safety
    Safety,
    /// Other/unspecified objection
    Other(String),
}

impl ObjectionType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Rate => "rate",
            Self::Trust => "trust",
            Self::Timing => "timing",
            Self::Competition => "competition",
            Self::Process => "process",
            Self::Safety => "safety",
            Self::Other(s) => s.as_str(),
        }
    }
}

/// Actions to execute after state transition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FSMAction {
    // Speech actions
    /// Start listening for user input
    StartListening,
    /// Stop listening
    StopListening,
    /// Start speaking the given text
    StartSpeaking { text: String },
    /// Stop speaking (on barge-in)
    StopSpeaking,

    // Context actions
    /// Load customer profile
    LoadCustomerProfile { customer_id: String },
    /// Update context value
    UpdateContext { key: String, value: serde_json::Value },
    /// Clear context value
    ClearContext { key: String },

    // Tool actions
    /// Execute a tool
    ExecuteTool { name: String, params: serde_json::Value },
    /// Prefetch data for anticipated tool use
    PrefetchTool { name: String, hint: Option<String> },

    // State actions
    /// Create checkpoint for recovery
    Checkpoint,
    /// End conversation with outcome
    EndConversation { outcome: ConversationOutcome },
    /// Escalate to human agent
    Escalate { to: String, reason: String },
    /// Schedule follow-up
    ScheduleFollowUp { reason: String, delay_hours: Option<u32> },

    // Metrics actions
    /// Record metric
    RecordMetric { name: String, value: f64, tags: HashMap<String, String> },
    /// Log event
    LogEvent { level: String, message: String },
}

impl FSMAction {
    /// Create a speak action
    pub fn speak(text: impl Into<String>) -> Self {
        Self::StartSpeaking { text: text.into() }
    }

    /// Create an execute tool action
    pub fn execute_tool(name: impl Into<String>, params: serde_json::Value) -> Self {
        Self::ExecuteTool {
            name: name.into(),
            params,
        }
    }

    /// Create an update context action
    pub fn update_context(key: impl Into<String>, value: serde_json::Value) -> Self {
        Self::UpdateContext {
            key: key.into(),
            value,
        }
    }

    /// Create an end conversation action
    pub fn end(outcome: ConversationOutcome) -> Self {
        Self::EndConversation { outcome }
    }
}

/// Final conversation outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationOutcome {
    /// Successfully converted (lead captured, appointment booked)
    Converted,
    /// Follow-up scheduled
    FollowUp,
    /// Customer declined
    Declined,
    /// Escalated to human
    Escalated,
    /// Error occurred
    Error,
    /// Customer hung up
    Abandoned,
}

/// FSM state checkpoint for recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FSMCheckpoint {
    /// Checkpoint index
    pub index: usize,
    /// Stage at checkpoint
    pub stage: ConversationStage,
    /// Context snapshot
    pub context: HashMap<String, serde_json::Value>,
    /// Timestamp
    pub timestamp_ms: u64,
    /// Turn count at checkpoint
    pub turn_count: usize,
}

/// FSM errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum FSMError {
    #[error("Invalid transition from {from:?} with event {event}")]
    InvalidTransition { from: ConversationStage, event: String },

    #[error("No checkpoint at index {0}")]
    NoCheckpoint(usize),

    #[error("Checkpoint restore failed: {0}")]
    RestoreFailed(String),

    #[error("State machine corrupted: {0}")]
    Corrupted(String),

    #[error("Context key not found: {0}")]
    ContextKeyNotFound(String),
}

/// Conversation Finite State Machine trait
///
/// Implementors manage conversation flow through discrete stages,
/// handling events and producing actions.
///
/// # Example Implementation
///
/// ```ignore
/// struct GoldLoanFSM {
///     stage: ConversationStage,
///     context: HashMap<String, Value>,
///     checkpoints: Vec<FSMCheckpoint>,
/// }
///
/// #[async_trait]
/// impl ConversationFSM for GoldLoanFSM {
///     fn state(&self) -> &ConversationStage {
///         &self.stage
///     }
///
///     async fn transition(&mut self, event: ConversationEvent) -> Result<Vec<FSMAction>, FSMError> {
///         match (&self.stage, &event) {
///             (ConversationStage::Greeting, ConversationEvent::UserIntent { intent, .. })
///                 if intent == "interested" => {
///                 self.stage = ConversationStage::Discovery;
///                 Ok(vec![FSMAction::speak("Great! Tell me about your gold...")])
///             }
///             _ => Ok(vec![])
///         }
///     }
///     // ... other methods
/// }
/// ```
#[async_trait]
pub trait ConversationFSM: Send + Sync + 'static {
    /// Get current conversation stage
    fn state(&self) -> &ConversationStage;

    /// Process an event and transition to new state
    ///
    /// Returns a list of actions to execute. The FSM implementation
    /// updates its internal state before returning.
    ///
    /// # Errors
    /// Returns FSMError::InvalidTransition if the event is not valid
    /// for the current state.
    async fn transition(&mut self, event: ConversationEvent) -> Result<Vec<FSMAction>, FSMError>;

    /// Check if a transition is valid without executing it
    fn can_transition(&self, event: &ConversationEvent) -> bool;

    /// Get all valid events for current state
    fn valid_events(&self) -> Vec<String>;

    /// Create a checkpoint of current state
    ///
    /// Checkpoints can be restored later for error recovery.
    fn checkpoint(&mut self) -> FSMCheckpoint;

    /// Restore to a previous checkpoint
    ///
    /// # Errors
    /// Returns FSMError::NoCheckpoint if the index doesn't exist.
    fn restore(&mut self, checkpoint_index: usize) -> Result<(), FSMError>;

    /// Get all checkpoints
    fn checkpoints(&self) -> &[FSMCheckpoint];

    /// Get context value by key
    fn get_context(&self, key: &str) -> Option<&serde_json::Value>;

    /// Set context value
    fn set_context(&mut self, key: &str, value: serde_json::Value);

    /// Get full context
    fn context(&self) -> &HashMap<String, serde_json::Value>;

    /// Get conversation metrics
    fn metrics(&self) -> FSMMetrics;

    /// Reset to initial state
    fn reset(&mut self);
}

/// FSM runtime metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FSMMetrics {
    /// Total turns in conversation
    pub turn_count: usize,
    /// Turns per stage
    pub stage_turns: HashMap<String, usize>,
    /// Time spent per stage (ms)
    pub stage_duration_ms: HashMap<String, u64>,
    /// Objections raised
    pub objections: Vec<ObjectionType>,
    /// Tools called
    pub tools_called: Vec<String>,
    /// Checkpoints created
    pub checkpoint_count: usize,
    /// Restores performed
    pub restore_count: usize,
}

/// Transition record for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRecord {
    /// Previous stage
    pub from: ConversationStage,
    /// New stage
    pub to: ConversationStage,
    /// Event that triggered transition
    pub event: String,
    /// Actions produced
    pub actions: Vec<String>,
    /// Timestamp
    pub timestamp_ms: u64,
    /// Confidence of transition decision
    pub confidence: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_event_creation() {
        let event = ConversationEvent::intent("interested", 0.95);
        match event {
            ConversationEvent::UserIntent { intent, confidence } => {
                assert_eq!(intent, "interested");
                assert!((confidence - 0.95).abs() < 0.001);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_fsm_action_creation() {
        let action = FSMAction::speak("Hello!");
        match action {
            FSMAction::StartSpeaking { text } => {
                assert_eq!(text, "Hello!");
            }
            _ => panic!("Wrong action type"),
        }
    }

    #[test]
    fn test_objection_type_as_str() {
        assert_eq!(ObjectionType::Rate.as_str(), "rate");
        assert_eq!(ObjectionType::Trust.as_str(), "trust");
        assert_eq!(ObjectionType::Other("custom".to_string()).as_str(), "custom");
    }

    #[test]
    fn test_conversation_outcome_serialization() {
        let outcome = ConversationOutcome::Converted;
        let json = serde_json::to_string(&outcome).unwrap();
        assert_eq!(json, "\"converted\"");
    }
}
