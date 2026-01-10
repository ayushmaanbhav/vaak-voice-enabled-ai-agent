//! Text Processing Pipeline for Voice Agent
//!
//! This crate provides text processing capabilities:
//! - **Grammar Correction**: Fix STT errors while preserving domain vocabulary
//! - **Translation**: Translate between Indian languages (Translate-Think-Translate)
//! - **PII Detection**: Detect and redact sensitive Indian data (Aadhaar, PAN, etc.)
//! - **Compliance Checking**: Ensure banking regulatory compliance
//! - **Intent Detection**: Detect user intents and extract slots (P1-2 FIX: moved from agent)
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_text_processing::{TextProcessingPipeline, TextProcessingConfig};
//!
//! let config = TextProcessingConfig::default();
//! let pipeline = TextProcessingPipeline::new(config)?;
//!
//! // Process text through the pipeline
//! let result = pipeline.process("mujhe gol lone chahiye").await?;
//! println!("Processed: {}", result.text);
//! ```

pub mod compliance;
pub mod entities;
pub mod grammar;
pub mod hindi; // P2.2 FIX: Shared Hindi language utilities
pub mod intent; // P1-2 FIX: Intent detection moved from agent crate
pub mod pii;
pub mod sentiment; // P2-1 FIX: Sentiment analysis for customer emotion detection
pub mod simplifier; // P2 FIX: Text simplifier for TTS
pub mod slot_extraction; // P3-3 FIX: Slot extraction moved from agent/dst
pub mod translation; // P2-5 FIX: Loan entity extraction

mod error;
mod pipeline;

pub use error::{Result, TextProcessingError};
pub use pipeline::{ProcessedText, TextProcessingConfig, TextProcessingPipeline};

// Re-export key types
pub use compliance::{ComplianceConfig, ComplianceProvider, RuleBasedComplianceChecker};
pub use grammar::{GrammarConfig, GrammarProvider, LLMGrammarCorrector, NoopCorrector};
pub use pii::{HybridPIIDetector, IndianPIIPatterns, PIIConfig, PIIProvider};
pub use simplifier::{AbbreviationExpander, NumberToWords, TextSimplifier, TextSimplifierConfig};
pub use translation::{ScriptDetector, TranslationConfig, TranslationProvider};
// P1-2 FIX: Intent detection exports
pub use intent::{DetectedIntent, Intent, IntentDetector, Slot, SlotType};
// P2-1 FIX: Sentiment analysis exports
pub use sentiment::{Sentiment, SentimentAnalyzer, SentimentConfig, SentimentResult};
// P2-5 FIX: Loan entity extraction exports
pub use entities::{Currency, Duration, EntityExtractor, ExtractedEntities, Percentage, Weight};
// P3-3 FIX: Slot extraction exports (moved from agent/dst)
pub use slot_extraction::SlotExtractor;
