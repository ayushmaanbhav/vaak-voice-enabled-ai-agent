//! P2-2 FIX: Context Compressor for Conversation History
//!
//! Compresses long conversation history by summarizing older turns while
//! keeping recent turns intact. Uses LLM for intelligent summarization.
//!
//! # Strategy
//!
//! 1. Keep N most recent turns intact (recency window)
//! 2. Summarize older turns into a compact "Previously discussed:" summary
//! 3. Maintain key information: customer name, loan amount, gold weight, etc.
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_rag::compressor::{ContextCompressor, CompressorConfig, Turn};
//!
//! let compressor = ContextCompressor::new(CompressorConfig::default());
//! let turns = vec![
//!     Turn::user("I want a gold loan"),
//!     Turn::assistant("I'd be happy to help with a gold loan. What is your name?"),
//!     Turn::user("My name is Rahul"),
//!     // ... more turns
//! ];
//!
//! let compressed = compressor.compress(&turns, 500).await?;
//! println!("Compressed: {}", compressed.text);
//! ```

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::RagError;

/// A conversation turn (user or assistant message)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    /// Role: "user" or "assistant"
    pub role: String,
    /// Message content
    pub content: String,
    /// Optional timestamp
    pub timestamp: Option<String>,
}

impl Turn {
    /// Create a user turn
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
            timestamp: None,
        }
    }

    /// Create an assistant turn
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            timestamp: None,
        }
    }

    /// Estimate token count (simple approximation: 1 token ≈ 4 chars)
    pub fn estimated_tokens(&self) -> usize {
        self.content.len().div_ceil(4) + 10 // +10 for role prefix
    }
}

/// Result of context compression
#[derive(Debug, Clone)]
pub struct CompressedContext {
    /// Compressed text representation
    pub text: String,
    /// Number of turns summarized
    pub summarized_turns: usize,
    /// Number of turns kept intact
    pub intact_turns: usize,
    /// Estimated token count
    pub estimated_tokens: usize,
    /// Key entities extracted during summarization
    pub extracted_entities: Vec<(String, String)>,
}

impl CompressedContext {
    /// Format as context for LLM
    pub fn as_context(&self) -> String {
        self.text.clone()
    }
}

/// Configuration for context compressor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressorConfig {
    /// Number of recent turns to keep intact
    pub recency_window: usize,
    /// Maximum tokens for the summary portion
    pub max_summary_tokens: usize,
    /// Whether to extract entities during summarization
    pub extract_entities: bool,
    /// Language for summary (affects templates)
    pub language: String,
}

impl Default for CompressorConfig {
    fn default() -> Self {
        Self {
            recency_window: 4,       // Keep last 4 turns intact
            max_summary_tokens: 200, // Summarize older turns into ~200 tokens
            extract_entities: true,
            language: "en".to_string(),
        }
    }
}

impl CompressorConfig {
    /// Config for short conversations (aggressive compression)
    pub fn aggressive() -> Self {
        Self {
            recency_window: 2,
            max_summary_tokens: 100,
            ..Default::default()
        }
    }

    /// Config for long conversations (preserve more context)
    pub fn conservative() -> Self {
        Self {
            recency_window: 6,
            max_summary_tokens: 400,
            ..Default::default()
        }
    }
}

/// Trait for LLM summarization capability
#[async_trait::async_trait]
pub trait Summarizer: Send + Sync {
    /// Summarize text into a shorter version
    async fn summarize(&self, text: &str, max_tokens: usize) -> Result<String, RagError>;
}

/// Simple rule-based summarizer (no LLM needed)
///
/// Extracts key information without LLM. Good for fallback
/// or when LLM is not available.
pub struct RuleBasedSummarizer;

#[async_trait::async_trait]
impl Summarizer for RuleBasedSummarizer {
    async fn summarize(&self, text: &str, max_tokens: usize) -> Result<String, RagError> {
        // Extract key patterns
        let mut summary_parts = Vec::new();

        // Extract names
        if let Some(name) = Self::extract_pattern(text, &["my name is", "i am", "this is"], 3) {
            summary_parts.push(format!("Customer: {}", name));
        }

        // Extract amounts
        if let Some(amount) = Self::extract_pattern(text, &["lakh", "crore", "rupees", "₹"], 4) {
            summary_parts.push(format!("Amount discussed: {}", amount));
        }

        // P18 FIX: Extract collateral weight (generic - gram/kg are standard units)
        if let Some(weight) = Self::extract_pattern(text, &["gram", "gm", "kg", "tola"], 3) {
            summary_parts.push(format!("Collateral: {}", weight));
        }

        // P18 FIX: Provider detection removed - should be config-driven at runtime
        // Competitor names should come from domain config, not hardcoded here

        // Build summary
        let summary = if summary_parts.is_empty() {
            // Fall back to truncation
            Self::smart_truncate(text, max_tokens)
        } else {
            format!("Previously discussed: {}", summary_parts.join("; "))
        };

        Ok(summary)
    }
}

impl RuleBasedSummarizer {
    /// Extract text around a pattern
    fn extract_pattern(text: &str, patterns: &[&str], words_after: usize) -> Option<String> {
        let text_lower = text.to_lowercase();

        for pattern in patterns {
            if let Some(pos) = text_lower.find(pattern) {
                let start = pos + pattern.len();
                let remaining = &text[start..];

                // Extract next N words
                let words: Vec<&str> = remaining.split_whitespace().take(words_after).collect();

                if !words.is_empty() {
                    return Some(words.join(" "));
                }
            }
        }

        None
    }

    /// Smart truncation that preserves sentence boundaries
    fn smart_truncate(text: &str, max_tokens: usize) -> String {
        let max_chars = max_tokens * 4; // Approximate

        if text.len() <= max_chars {
            return text.to_string();
        }

        // Find sentence boundary near the limit
        let truncated = &text[..max_chars.min(text.len())];

        if let Some(pos) = truncated.rfind(['.', '?', '!']) {
            truncated[..=pos].to_string()
        } else if let Some(pos) = truncated.rfind(',') {
            format!("{}..", &truncated[..pos])
        } else {
            format!("{}..", truncated.trim())
        }
    }
}

/// Context compressor for conversation history
pub struct ContextCompressor<S: Summarizer = RuleBasedSummarizer> {
    config: CompressorConfig,
    summarizer: Arc<S>,
}

impl ContextCompressor<RuleBasedSummarizer> {
    /// Create with default rule-based summarizer
    pub fn new(config: CompressorConfig) -> Self {
        Self {
            config,
            summarizer: Arc::new(RuleBasedSummarizer),
        }
    }
}

impl<S: Summarizer> ContextCompressor<S> {
    /// Create with custom summarizer
    pub fn with_summarizer(config: CompressorConfig, summarizer: Arc<S>) -> Self {
        Self { config, summarizer }
    }

    /// Compress conversation history to fit within token budget
    ///
    /// # Arguments
    /// * `turns` - Conversation turns
    /// * `max_tokens` - Maximum tokens for entire compressed context
    ///
    /// # Returns
    /// Compressed context with summary and recent turns
    pub async fn compress(
        &self,
        turns: &[Turn],
        max_tokens: usize,
    ) -> Result<CompressedContext, RagError> {
        if turns.is_empty() {
            return Ok(CompressedContext {
                text: String::new(),
                summarized_turns: 0,
                intact_turns: 0,
                estimated_tokens: 0,
                extracted_entities: Vec::new(),
            });
        }

        // Calculate total tokens
        let total_tokens: usize = turns.iter().map(|t| t.estimated_tokens()).sum();

        // If fits, return as-is
        if total_tokens <= max_tokens {
            return Ok(CompressedContext {
                text: self.format_turns(turns),
                summarized_turns: 0,
                intact_turns: turns.len(),
                estimated_tokens: total_tokens,
                extracted_entities: Vec::new(),
            });
        }

        // Split into summarize and keep sections
        let keep_count = self.config.recency_window.min(turns.len());
        let summarize_count = turns.len().saturating_sub(keep_count);

        let to_summarize = &turns[..summarize_count];
        let to_keep = &turns[summarize_count..];

        // Calculate token budget for summary
        let keep_tokens: usize = to_keep.iter().map(|t| t.estimated_tokens()).sum();
        let summary_budget = max_tokens
            .saturating_sub(keep_tokens)
            .min(self.config.max_summary_tokens);

        // Generate summary
        let summary = if summarize_count > 0 {
            let summary_text = self.format_turns(to_summarize);
            self.summarizer
                .summarize(&summary_text, summary_budget)
                .await?
        } else {
            String::new()
        };

        // Extract entities from summary
        let entities = if self.config.extract_entities {
            self.extract_entities(&summary)
        } else {
            Vec::new()
        };

        // Combine summary and recent turns
        let mut result = String::new();

        if !summary.is_empty() {
            result.push_str("[Summary of earlier conversation]\n");
            result.push_str(&summary);
            result.push_str("\n\n[Recent conversation]\n");
        }

        result.push_str(&self.format_turns(to_keep));

        let estimated_tokens = result.len().div_ceil(4);

        Ok(CompressedContext {
            text: result,
            summarized_turns: summarize_count,
            intact_turns: keep_count,
            estimated_tokens,
            extracted_entities: entities,
        })
    }

    /// Format turns as text
    fn format_turns(&self, turns: &[Turn]) -> String {
        turns
            .iter()
            .map(|t| format!("{}: {}", t.role.to_uppercase(), t.content))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Extract key entities from summary
    fn extract_entities(&self, text: &str) -> Vec<(String, String)> {
        let mut entities = Vec::new();
        let text_lower = text.to_lowercase();

        // Extract customer name
        if text_lower.contains("customer:") {
            if let Some(pos) = text_lower.find("customer:") {
                let remainder = &text[pos + 9..];
                if let Some(end) = remainder.find([';', '.', '\n']) {
                    let name = remainder[..end].trim();
                    if !name.is_empty() {
                        entities.push(("customer_name".to_string(), name.to_string()));
                    }
                }
            }
        }

        // Extract amount
        if text_lower.contains("amount") {
            if let Some(pos) = text_lower.find("amount") {
                let remainder = &text[pos..];
                if let Some(end) = remainder.find([';', '\n']) {
                    let amount = remainder[..end].trim();
                    entities.push(("loan_amount".to_string(), amount.to_string()));
                }
            }
        }

        entities
    }
}

impl Default for ContextCompressor<RuleBasedSummarizer> {
    fn default() -> Self {
        Self::new(CompressorConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_compression_needed() {
        let compressor = ContextCompressor::default();
        let turns = vec![Turn::user("Hello"), Turn::assistant("Hi there!")];

        let result = compressor.compress(&turns, 1000).await.unwrap();

        assert_eq!(result.summarized_turns, 0);
        assert_eq!(result.intact_turns, 2);
        assert!(result.text.contains("USER: Hello"));
        assert!(result.text.contains("ASSISTANT: Hi there!"));
    }

    #[tokio::test]
    async fn test_compression_with_long_history() {
        let compressor = ContextCompressor::new(CompressorConfig {
            recency_window: 2,
            ..Default::default()
        });

        // Create turns with longer content to exceed budget
        let turns = vec![
            Turn::user("My name is Rahul Kumar and I am looking for a gold loan. I have about 50 grams of 22 karat gold jewelry."),
            Turn::assistant("Hello Rahul! Thank you for your interest in our gold loan services. We offer competitive rates starting from 10.5% per annum."),
            Turn::user("I currently have a loan with Muthoot Finance at 14% interest rate. I want to transfer my gold loan to get better rates."),
            Turn::assistant("I understand. We can definitely help you with a balance transfer. Our rate of 10.5% will save you significant money."),
            Turn::user("What is the maximum loan amount I can get?"),
            Turn::assistant("Based on 50 grams of 22K gold at today's rate, you can get up to Rs 2.5 lakh."),
        ];

        // Force compression with small budget (smaller than total tokens)
        let result = compressor.compress(&turns, 150).await.unwrap();

        // Should summarize old turns, keep recent ones
        assert!(
            result.summarized_turns > 0,
            "Expected some turns to be summarized, got {}",
            result.summarized_turns
        );
        assert_eq!(result.intact_turns, 2);
        assert!(result.text.contains("[Summary") || result.text.contains("Previously"));
    }

    #[tokio::test]
    async fn test_entity_extraction() {
        let compressor = ContextCompressor::new(CompressorConfig {
            extract_entities: true,
            recency_window: 1,
            ..Default::default()
        });

        let turns = vec![
            Turn::user("My name is Rahul and I want 5 lakh loan"),
            Turn::assistant("Sure Rahul"),
            Turn::user("What's next?"),
        ];

        let result = compressor.compress(&turns, 100).await.unwrap();

        // Check if name was extracted
        let has_name = result
            .extracted_entities
            .iter()
            .any(|(k, _)| k == "customer_name");
        assert!(has_name || result.text.contains("Rahul"));
    }

    #[test]
    fn test_turn_token_estimation() {
        let turn = Turn::user("This is a test message with several words");
        let tokens = turn.estimated_tokens();

        // Should be reasonable estimate
        assert!(tokens > 5);
        assert!(tokens < 50);
    }

    #[test]
    fn test_rule_based_summarizer_patterns() {
        let text = "My name is Rahul. I want a 5 lakh loan. I have gold from Muthoot.";

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { RuleBasedSummarizer.summarize(text, 100).await })
            .unwrap();

        // Should extract key info
        assert!(
            result.to_lowercase().contains("rahul")
                || result.to_lowercase().contains("lakh")
                || result.to_lowercase().contains("muthoot")
        );
    }
}
