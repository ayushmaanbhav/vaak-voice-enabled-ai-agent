//! Voice Activity Detection
//!
//! Provides two VAD implementations:
//! - MagicNet: Custom VAD with 10ms frames and mel filterbank features
//! - Silero: Production-ready VAD using raw audio input (recommended)

mod magicnet;
mod silero;

pub use magicnet::{VadConfig, VadResult, VadState, VoiceActivityDetector};
pub use silero::{SileroConfig, SileroVad};

use crate::PipelineError;
use voice_agent_core::AudioFrame;

/// VAD engine trait for pluggable implementations
///
/// Returns (VadState, probability, VadResult) tuple to provide full context
/// for downstream processing (turn detection, barge-in, etc.)
pub trait VadEngine: Send + Sync {
    /// Process a single audio frame
    /// Returns (current_state, speech_probability, detailed_result)
    fn process_frame(&self, frame: &mut AudioFrame) -> Result<(VadState, f32, VadResult), PipelineError>;

    /// Reset VAD state
    fn reset(&self);

    /// Get current state
    fn state(&self) -> VadState;
}
