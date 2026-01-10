//! Agent traits for abstraction and testability
//!
//! P1-1 FIX: Create Agent trait to enable:
//! - Easy testing with mock implementations
//! - Alternative agent implementations
//! - Clean separation of interface from implementation

use async_trait::async_trait;
use tokio::sync::{broadcast, mpsc};
use voice_agent_core::Language;

use crate::agent_config::AgentEvent;
use crate::conversation::EndReason;
use crate::stage::ConversationStage;
use crate::AgentError;

/// Core agent trait for conversational AI agents
///
/// This trait defines the essential interface for any conversational agent.
/// Implementations handle user input, maintain conversation state, and
/// generate appropriate responses.
///
/// # Example
///
/// ```ignore
/// struct MockAgent;
///
/// #[async_trait]
/// impl Agent for MockAgent {
///     async fn process(&self, input: &str) -> Result<String, AgentError> {
///         Ok(format!("Echo: {}", input))
///     }
///     // ... other methods
/// }
/// ```
#[async_trait]
pub trait Agent: Send + Sync {
    /// Process user input and generate a response
    ///
    /// This is the main entry point for agent interaction. The agent will:
    /// 1. Detect intent from the input
    /// 2. Update conversation state
    /// 3. Optionally retrieve context via RAG
    /// 4. Generate a response using LLM
    /// 5. Handle any tool calls
    ///
    /// # Arguments
    /// * `input` - The user's input text (may be empty for initial greeting)
    ///
    /// # Returns
    /// The agent's response text, or an error
    async fn process(&self, input: &str) -> Result<String, AgentError>;

    /// Process user input with streaming response
    ///
    /// Like `process()`, but returns a channel that yields translated
    /// sentences as they are ready. This enables lower latency TTS by
    /// starting synthesis before the full response is complete.
    ///
    /// # Arguments
    /// * `input` - The user's input text
    ///
    /// # Returns
    /// A receiver that yields translated response sentences
    async fn process_stream(&self, input: &str) -> Result<mpsc::Receiver<String>, AgentError>;

    /// Get the current conversation stage
    fn stage(&self) -> ConversationStage;

    /// Get the user's preferred language
    fn user_language(&self) -> Language;

    /// Subscribe to agent events
    ///
    /// Returns a broadcast receiver for events like:
    /// - Thinking (processing started)
    /// - Response (response generated)
    /// - Tool calls
    /// - Stage transitions
    fn subscribe(&self) -> broadcast::Receiver<AgentEvent>;

    /// Get the agent's name
    fn name(&self) -> &str;

    /// End the conversation
    ///
    /// # Arguments
    /// * `reason` - Why the conversation is ending
    fn end(&self, reason: EndReason);
}

/// Agent with prefetch capabilities for lower latency
///
/// Extends the base Agent trait with methods for prefetching context
/// based on partial transcripts, enabling speculative RAG retrieval.
#[async_trait]
pub trait PrefetchingAgent: Agent {
    /// Prefetch context based on partial transcript
    ///
    /// Called when partial STT results are available to speculatively
    /// start RAG retrieval before the user finishes speaking.
    ///
    /// # Arguments
    /// * `partial_transcript` - Partial transcript from STT
    /// * `confidence` - Confidence score (0.0 - 1.0)
    ///
    /// # Returns
    /// `true` if prefetch was started
    async fn prefetch_on_partial(&self, partial_transcript: &str, confidence: f32) -> bool;

    /// Start prefetch in background task
    ///
    /// Non-blocking version that spawns a background task.
    fn prefetch_background(&self, partial_transcript: String, confidence: f32);

    /// Clear any cached prefetch results
    fn clear_prefetch_cache(&self);
}

/// Agent with personalization capabilities
pub trait PersonalizableAgent: Agent {
    /// Set customer profile for personalization
    fn set_customer_profile(&self, profile: &voice_agent_core::CustomerProfile);

    /// Set customer name
    fn set_customer_name(&self, name: impl Into<String>);

    /// Set customer segment (enum-based - deprecated)
    ///
    /// Use `set_segment_id` instead for config-driven segment support.
    fn set_customer_segment(&self, segment: voice_agent_core::CustomerSegment);

    /// P25 FIX: Set customer segment by config-driven ID
    ///
    /// This method accepts a string segment ID from config (e.g., "high_value",
    /// "trust_seeker", "women", "professional") and uses config-driven persona
    /// lookup instead of the hardcoded enum-based approach.
    ///
    /// # Arguments
    /// * `segment_id` - Segment ID as defined in segments.yaml
    fn set_segment_id(&self, segment_id: impl Into<String>);
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test that traits are object-safe
    fn _assert_agent_object_safe(_: &dyn Agent) {}

    // Note: PrefetchingAgent is NOT object-safe due to prefetch_background
    // taking String by value. This is intentional as we don't need dyn dispatch for it.
}
