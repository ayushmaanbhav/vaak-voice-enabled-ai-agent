//! Frame processors for the pipeline
//!
//! This module contains FrameProcessor implementations for:
//! - SentenceDetector: Detects sentence boundaries from LLM chunks
//! - InterruptHandler: Handles barge-in with configurable modes
//! - ProcessorChain: Channel-based chain connecting processors

mod sentence_detector;
mod interrupt_handler;
mod chain;

pub use sentence_detector::{SentenceDetector, SentenceDetectorConfig};
pub use interrupt_handler::{InterruptHandler, InterruptMode, InterruptHandlerConfig};
pub use chain::{ProcessorChain, ProcessorChainBuilder};
