//! MemGPT-Style Agentic Memory System
//!
//! This module implements a hierarchical memory architecture inspired by:
//! - MemGPT (arXiv:2310.08560): Virtual context management
//! - A-MEM (arXiv:2502.12110): Zettelkasten-style memory linking
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      Main Context                            │
//! │  ┌──────────────┬────────────────────┬───────────────────┐  │
//! │  │    System    │   Core Memory      │    FIFO Queue     │  │
//! │  │ Instructions │  (Human + Persona) │  (Recent Turns)   │  │
//! │  └──────────────┴────────────────────┴───────────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//!                           ↕ Memory Functions
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    External Context                          │
//! │  ┌─────────────────────────┬────────────────────────────┐   │
//! │  │    Archival Storage     │      Recall Storage        │   │
//! │  │   (Vector DB / Long)    │   (Conversation Search)    │   │
//! │  └─────────────────────────┴────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Memory Functions (callable by agent)
//!
//! - `core_memory_append`: Add to human block
//! - `core_memory_replace`: Update human block
//! - `archival_memory_insert`: Store in long-term memory
//! - `archival_memory_search`: Search long-term memory
//! - `conversation_search`: Search conversation history

pub mod archival;
pub mod core;
pub mod recall;

pub use archival::{
    ArchivalMemory, ArchivalMemoryConfig, ArchivalSearchResult, MemoryNote, MemorySource,
    MemoryType,
};
pub use core::{
    CoreMemory, CoreMemoryConfig, CoreMemoryError, EntrySource, HumanBlock, MemoryBlockEntry,
    PersonaBlock,
};
pub use recall::{
    ConversationTurn, RecallMemory, RecallMemoryConfig, RecallSearchResult, TurnRole,
};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use voice_agent_core::{GenerateRequest, LanguageModel};

/// Unified memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgenticMemoryConfig {
    /// Core memory configuration
    pub core: CoreMemoryConfig,
    /// Archival memory configuration
    pub archival: ArchivalMemoryConfig,
    /// Recall memory configuration
    pub recall: RecallMemoryConfig,
    /// Maximum tokens for combined context
    pub max_context_tokens: usize,
    /// High watermark for context compaction
    pub high_watermark_tokens: usize,
    /// Low watermark target after compaction
    pub low_watermark_tokens: usize,
    /// Enable automatic summarization
    pub auto_summarize: bool,
}

impl Default for AgenticMemoryConfig {
    fn default() -> Self {
        Self {
            core: CoreMemoryConfig::default(),
            archival: ArchivalMemoryConfig::default(),
            recall: RecallMemoryConfig::default(),
            max_context_tokens: 4096,
            high_watermark_tokens: 3072,
            low_watermark_tokens: 2048,
            auto_summarize: true,
        }
    }
}

/// Memory statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Core memory tokens
    pub core_tokens: usize,
    /// FIFO (recent turns) tokens
    pub fifo_tokens: usize,
    /// Total recall memory tokens
    pub recall_total_tokens: usize,
    /// Archival memory count
    pub archival_count: usize,
    /// Total estimated context tokens
    pub total_context_tokens: usize,
    /// Whether above high watermark
    pub above_high_watermark: bool,
    /// Whether above max limit
    pub above_max_limit: bool,
}

/// Compression statistics for tracking compression efficiency
#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    /// Original token count before compression
    pub original_tokens: usize,
    /// Token count after compression
    pub compressed_tokens: usize,
    /// Number of turns summarized
    pub turns_summarized: usize,
    /// Number of turns kept intact
    pub turns_intact: usize,
    /// Compression ratio (original / compressed)
    pub compression_ratio: f32,
    /// Compression method used
    pub method: CompressionMethod,
}

impl CompressionStats {
    /// Create new compression stats
    pub fn new(original: usize, compressed: usize, summarized: usize, intact: usize) -> Self {
        let ratio = if compressed > 0 {
            original as f32 / compressed as f32
        } else {
            0.0
        };
        Self {
            original_tokens: original,
            compressed_tokens: compressed,
            turns_summarized: summarized,
            turns_intact: intact,
            compression_ratio: ratio,
            method: CompressionMethod::LlmSummarization,
        }
    }

    /// Check if compression was effective (ratio > 1.5x)
    pub fn is_effective(&self) -> bool {
        self.compression_ratio >= 1.5
    }
}

/// Compression method used
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CompressionMethod {
    /// No compression needed
    #[default]
    None,
    /// LLM-based summarization
    LlmSummarization,
    /// Rule-based extraction
    RuleBased,
    /// Simple truncation
    Truncation,
}

/// Compression aggressiveness level
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CompressionLevel {
    /// Keep more context, less aggressive compression
    Conservative,
    /// Balanced approach (default)
    #[default]
    Balanced,
    /// Aggressive compression for small context windows
    Aggressive,
    /// Maximum compression for very limited contexts
    Maximum,
}

impl CompressionLevel {
    /// Get target compression ratio for this level
    pub fn target_ratio(&self) -> f32 {
        match self {
            CompressionLevel::Conservative => 2.0,
            CompressionLevel::Balanced => 4.0,
            CompressionLevel::Aggressive => 8.0,
            CompressionLevel::Maximum => 16.0,
        }
    }

    /// Get recency window size for this level
    pub fn recency_window(&self) -> usize {
        match self {
            CompressionLevel::Conservative => 8,
            CompressionLevel::Balanced => 6,
            CompressionLevel::Aggressive => 4,
            CompressionLevel::Maximum => 2,
        }
    }
}

/// Agentic Memory System
///
/// Unified MemGPT-style memory management combining:
/// - Core Memory: Always in context (human + persona blocks)
/// - Recall Memory: Searchable conversation history with FIFO
/// - Archival Memory: Long-term vector-based storage
pub struct AgenticMemory {
    config: AgenticMemoryConfig,
    /// Core memory (always in context)
    pub core: CoreMemory,
    /// Recall memory (conversation history)
    pub recall: RecallMemory,
    /// Archival memory (long-term storage)
    pub archival: ArchivalMemory,
    /// Session ID for this memory instance
    session_id: String,
    /// Optional LLM for summarization
    llm: RwLock<Option<Arc<dyn LanguageModel>>>,
}

impl AgenticMemory {
    /// Create new agentic memory system
    pub fn new(config: AgenticMemoryConfig, session_id: impl Into<String>) -> Self {
        Self {
            core: CoreMemory::new(config.core.clone()),
            recall: RecallMemory::new(config.recall.clone()),
            archival: ArchivalMemory::new(config.archival.clone()),
            config,
            session_id: session_id.into(),
            llm: RwLock::new(None),
        }
    }

    /// Create with default config
    pub fn with_session(session_id: impl Into<String>) -> Self {
        Self::new(AgenticMemoryConfig::default(), session_id)
    }

    /// Set LLM for summarization
    pub fn set_llm(&self, llm: Arc<dyn LanguageModel>) {
        *self.llm.write() = Some(llm);
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    // =========================================================================
    // MemGPT-Style Memory Functions
    // =========================================================================

    /// Append to core memory (human block)
    ///
    /// MemGPT function: core_memory_append
    pub fn core_memory_append(&self, key: &str, value: &str) -> Result<(), CoreMemoryError> {
        self.core.human_append(key, value)
    }

    /// Replace in core memory (human block)
    ///
    /// MemGPT function: core_memory_replace
    pub fn core_memory_replace(
        &self,
        key: &str,
        old_value: &str,
        new_value: &str,
    ) -> Result<(), CoreMemoryError> {
        self.core.human_replace(key, old_value, new_value)
    }

    /// Insert into archival memory
    ///
    /// MemGPT function: archival_memory_insert
    pub fn archival_memory_insert(&self, content: &str, memory_type: MemoryType) -> Uuid {
        let note = MemoryNote::new(&self.session_id, content, memory_type);
        self.archival.insert(note)
    }

    /// Insert detailed memory note
    pub fn archival_memory_insert_note(&self, note: MemoryNote) -> Uuid {
        self.archival.insert(note)
    }

    /// Search archival memory
    ///
    /// MemGPT function: archival_memory_search
    pub fn archival_memory_search(
        &self,
        query: &str,
        top_k: Option<usize>,
    ) -> Vec<ArchivalSearchResult> {
        self.archival.search(query, top_k)
    }

    /// Search conversation history
    ///
    /// MemGPT function: conversation_search
    pub fn conversation_search(&self, query: &str, top_k: Option<usize>) -> Vec<RecallSearchResult> {
        self.recall.search(query, top_k)
    }

    // =========================================================================
    // Conversation Management
    // =========================================================================

    /// Add a user turn
    pub fn add_user_turn(&self, content: &str) -> u64 {
        let turn = ConversationTurn::new(TurnRole::User, content);
        self.recall.add_turn(turn)
    }

    /// Add an assistant turn
    pub fn add_assistant_turn(&self, content: &str) -> u64 {
        let turn = ConversationTurn::new(TurnRole::Assistant, content);
        self.recall.add_turn(turn)
    }

    /// Add a turn with metadata
    pub fn add_turn(&self, turn: ConversationTurn) -> u64 {
        self.recall.add_turn(turn)
    }

    /// Get recent conversation (FIFO)
    pub fn get_recent_turns(&self) -> Vec<ConversationTurn> {
        self.recall.get_fifo()
    }

    /// Get all turns
    pub fn get_all_turns(&self) -> Vec<ConversationTurn> {
        self.recall.get_all()
    }

    // =========================================================================
    // Context Generation
    // =========================================================================

    /// Get formatted context for LLM
    ///
    /// Returns the complete context including:
    /// 1. Core memory (persona + human blocks)
    /// 2. FIFO recent turns
    pub fn get_context(&self) -> String {
        let mut context = String::new();

        // Core memory (always included)
        context.push_str(&self.core.format_for_context());
        context.push('\n');

        // Recent conversation (FIFO)
        let fifo_context = self.recall.format_fifo_for_context();
        if !fifo_context.is_empty() {
            context.push_str("## Recent Conversation\n");
            context.push_str(&fifo_context);
            context.push('\n');
        }

        context
    }

    /// Get context with RAG results
    pub fn get_context_with_rag(&self, rag_context: &str) -> String {
        let mut context = self.get_context();

        if !rag_context.is_empty() {
            context.push_str("\n## Retrieved Knowledge\n");
            context.push_str(rag_context);
            context.push('\n');
        }

        context
    }

    /// Get context limited to token budget
    pub fn get_context_limited(&self, max_tokens: usize) -> String {
        let full_context = self.get_context();
        let estimated = full_context.len() / 4;

        if estimated <= max_tokens {
            return full_context;
        }

        // Prioritize: persona > customer facts > recent turns
        let mut context = String::new();

        // Always include persona
        let persona = self.core.persona_snapshot();
        context.push_str(&persona.format_for_context());
        context.push('\n');

        // Include customer name if available
        let human = self.core.human_snapshot();
        if let Some(name) = &human.name {
            context.push_str(&format!("Customer: {}\n", name));
        }

        // Add as many FIFO turns as fit
        let remaining_tokens = max_tokens.saturating_sub(context.len() / 4);
        let fifo = self.recall.get_fifo();
        let mut fifo_tokens = 0;

        context.push_str("\n## Conversation\n");
        for turn in fifo.iter().rev() {
            if fifo_tokens + turn.estimated_tokens > remaining_tokens {
                break;
            }
            fifo_tokens += turn.estimated_tokens;
        }

        // Add turns in correct order
        let turns_to_include = fifo
            .iter()
            .rev()
            .take_while(|t| {
                let include = fifo_tokens >= t.estimated_tokens;
                fifo_tokens = fifo_tokens.saturating_sub(t.estimated_tokens);
                include
            })
            .collect::<Vec<_>>();

        for turn in turns_to_include.into_iter().rev() {
            context.push_str(&turn.format_for_context());
            context.push('\n');
        }

        context
    }

    // =========================================================================
    // Memory Management
    // =========================================================================

    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        let core_tokens = self.core.estimated_tokens();
        let fifo_tokens = self.recall.fifo_tokens();
        let recall_total_tokens = self.recall.total_tokens();
        let archival_count = self.archival.len();

        let total_context_tokens = core_tokens + fifo_tokens;

        MemoryStats {
            core_tokens,
            fifo_tokens,
            recall_total_tokens,
            archival_count,
            total_context_tokens,
            above_high_watermark: total_context_tokens > self.config.high_watermark_tokens,
            above_max_limit: total_context_tokens > self.config.max_context_tokens,
        }
    }

    /// Check if memory needs compaction
    pub fn needs_compaction(&self) -> bool {
        self.get_stats().above_high_watermark
    }

    /// Perform memory compaction
    ///
    /// This:
    /// 1. Summarizes pending recall turns
    /// 2. Moves summaries to archival storage
    /// 3. Cleans up low-confidence facts
    pub async fn compact(&self) -> Result<(), String> {
        // Get pending turns for summarization
        let pending = self.recall.get_pending_summarization();

        if pending.is_empty() {
            return Ok(());
        }

        // Try to summarize with LLM
        let summary = self.summarize_turns(&pending).await?;

        // Store summary in archival
        let note = MemoryNote::new(&self.session_id, &summary, MemoryType::ConversationSummary)
            .with_context("Conversation summary")
            .with_tags(vec!["summary".to_string()]);

        self.archival.insert(note);

        tracing::debug!(
            turns = pending.len(),
            "Compacted conversation turns into summary"
        );

        Ok(())
    }

    /// Summarize turns using LLM with enhanced prompts
    ///
    /// Uses LLMLingua-inspired compression techniques:
    /// - Focus on key entities and facts
    /// - Preserve customer-stated information
    /// - Maintain conversation flow markers
    async fn summarize_turns(&self, turns: &[ConversationTurn]) -> Result<String, String> {
        let llm = {
            let guard = self.llm.read();
            match guard.as_ref() {
                Some(llm) => llm.clone(),
                None => {
                    // Fallback: rule-based extraction
                    return Ok(self.rule_based_summary(turns));
                }
            }
        };

        // Build conversation text with metadata
        let conversation: String = turns
            .iter()
            .map(|t| {
                let mut line = t.format_for_context();
                // Add entity annotations for better extraction
                if !t.entities.is_empty() {
                    let entities: Vec<_> = t.entities.iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect();
                    line = format!("{} [entities: {}]", line, entities.join(", "));
                }
                line
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Enhanced summarization prompt inspired by LLMLingua research
        let prompt = format!(
            r#"Compress this gold loan conversation into a concise summary.

RULES:
1. KEEP: Customer name, gold weight, loan amount, interest rates, competitor names
2. KEEP: Customer concerns, objections, and preferences
3. REMOVE: Greetings, filler words, repeated information
4. FORMAT: Use key-value pairs where possible (e.g., "Name: Rahul, Gold: 50g")

Conversation:
{}

Compressed Summary (max 100 words):"#,
            conversation
        );

        let request = GenerateRequest::new(
            "You are a context compression assistant. Extract and preserve only essential information."
        ).with_user_message(prompt);

        match llm.generate(request).await {
            Ok(response) => Ok(response.text.trim().to_string()),
            Err(e) => {
                tracing::warn!("LLM summarization failed: {}", e);
                Ok(self.rule_based_summary(turns))
            }
        }
    }

    /// Rule-based summarization fallback (no LLM needed)
    ///
    /// Extracts key information using pattern matching:
    /// - Customer name, gold weight, loan amount
    /// - Competitor mentions
    /// - Key intents and entities
    fn rule_based_summary(&self, turns: &[ConversationTurn]) -> String {
        let mut facts = Vec::new();
        let mut seen_entities = std::collections::HashSet::new();

        // Extract from entities first (most reliable)
        for turn in turns {
            for (key, value) in &turn.entities {
                let normalized_key = key.to_lowercase();
                if !seen_entities.contains(&normalized_key) {
                    seen_entities.insert(normalized_key.clone());
                    let display_key = match normalized_key.as_str() {
                        "gold_weight" | "weight" => "Gold",
                        "loan_amount" | "amount" => "Amount",
                        "gold_purity" | "purity" | "karat" => "Purity",
                        "customer_name" | "name" => "Name",
                        "competitor" | "current_lender" => "Current Lender",
                        "interest_rate" | "rate" => "Rate",
                        _ => continue, // Skip unknown entities
                    };
                    facts.push(format!("{}: {}", display_key, value));
                }
            }
        }

        // Extract from content using patterns
        let all_content: String = turns
            .iter()
            .filter(|t| t.role == TurnRole::User)
            .map(|t| t.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let all_lower = all_content.to_lowercase();

        // Extract name if not found
        if !seen_entities.contains("name") {
            if let Some(name) = Self::extract_after_pattern(&all_content, &["my name is", "i am", "this is"]) {
                facts.push(format!("Name: {}", name));
            }
        }

        // Extract gold weight if not found
        if !seen_entities.contains("gold_weight") && !seen_entities.contains("weight") {
            if let Some(weight) = Self::extract_amount_with_unit(&all_lower, &["gram", "gm", "g ", "tola"]) {
                facts.push(format!("Gold: {}", weight));
            }
        }

        // Extract loan amount if not found
        if !seen_entities.contains("loan_amount") && !seen_entities.contains("amount") {
            if let Some(amount) = Self::extract_amount_with_unit(&all_lower, &["lakh", "crore", "rupees", "₹"]) {
                facts.push(format!("Amount: {}", amount));
            }
        }

        // Extract competitor mentions
        let competitors = ["muthoot", "manappuram", "iifl", "sbi", "hdfc"];
        for competitor in competitors {
            if all_lower.contains(competitor) && !seen_entities.contains("competitor") {
                facts.push(format!("Current Lender: {}", competitor.to_uppercase()));
                break;
            }
        }

        // Extract key intents
        let mut intents = Vec::new();
        for turn in turns {
            for intent in &turn.intents {
                if !intents.contains(intent) && intents.len() < 3 {
                    intents.push(intent.clone());
                }
            }
        }
        if !intents.is_empty() {
            facts.push(format!("Discussed: {}", intents.join(", ")));
        }

        if facts.is_empty() {
            // Ultimate fallback: truncate user messages
            let user_messages: Vec<_> = turns
                .iter()
                .filter(|t| t.role == TurnRole::User)
                .map(|t| {
                    if t.content.len() > 40 {
                        format!("{}...", &t.content[..40])
                    } else {
                        t.content.clone()
                    }
                })
                .take(3)
                .collect();
            format!("User discussed: {}", user_messages.join("; "))
        } else {
            format!("Previous: {}", facts.join(" | "))
        }
    }

    /// Extract text after a pattern
    fn extract_after_pattern(text: &str, patterns: &[&str]) -> Option<String> {
        let text_lower = text.to_lowercase();
        for pattern in patterns {
            if let Some(pos) = text_lower.find(pattern) {
                let start = pos + pattern.len();
                let remaining = &text[start..];
                // Extract 1-3 words (likely a name)
                let words: Vec<_> = remaining
                    .split_whitespace()
                    .take(3)
                    .filter(|w| w.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false))
                    .collect();
                if !words.is_empty() {
                    return Some(words.join(" "));
                }
            }
        }
        None
    }

    /// Extract amount with unit
    fn extract_amount_with_unit(text: &str, units: &[&str]) -> Option<String> {
        for unit in units {
            if let Some(pos) = text.find(unit) {
                // Look for number before the unit
                let before = &text[..pos];
                let words: Vec<_> = before.split_whitespace().rev().take(3).collect();
                for word in words {
                    // Check if it's a number
                    if word.chars().any(|c| c.is_numeric()) {
                        return Some(format!("{} {}", word, unit));
                    }
                }
            }
        }
        None
    }

    // =========================================================================
    // Selective Context Injection (Query-Relevant Context)
    // =========================================================================

    /// Get context with selective injection based on query relevance
    ///
    /// This method implements Anthropic-style selective context:
    /// 1. Always include core memory (persona + customer facts)
    /// 2. Include recent FIFO turns
    /// 3. Search archival/recall for query-relevant memories
    /// 4. Inject only relevant additional context
    pub fn get_context_for_query(&self, query: &str, max_tokens: usize) -> String {
        let mut context = String::new();
        let mut used_tokens = 0;

        // 1. Core memory (always included, highest priority)
        let core_context = self.core.format_for_context();
        let core_tokens = core_context.len() / 4;
        context.push_str(&core_context);
        context.push('\n');
        used_tokens += core_tokens;

        // 2. Recent FIFO turns (high priority)
        let fifo = self.recall.get_fifo();
        let fifo_context = self.recall.format_fifo_for_context();
        let fifo_tokens = fifo_context.len() / 4;

        if used_tokens + fifo_tokens <= max_tokens {
            context.push_str("## Recent Conversation\n");
            context.push_str(&fifo_context);
            context.push('\n');
            used_tokens += fifo_tokens;
        }

        // 3. Query-relevant archival memories (if space allows)
        let remaining_tokens = max_tokens.saturating_sub(used_tokens);
        if remaining_tokens > 100 {
            let archival_results = self.archival_memory_search(query, Some(3));

            if !archival_results.is_empty() {
                let mut archival_context = String::new();
                let mut archival_tokens = 0;

                for result in archival_results {
                    let note_text = result.note.format_for_context();
                    let note_tokens = note_text.len() / 4;

                    if archival_tokens + note_tokens <= remaining_tokens / 2 {
                        archival_context.push_str("- ");
                        archival_context.push_str(&note_text);
                        archival_context.push('\n');
                        archival_tokens += note_tokens;
                    }
                }

                if !archival_context.is_empty() {
                    context.push_str("\n## Relevant Background\n");
                    context.push_str(&archival_context);
                    used_tokens += archival_tokens;
                }
            }
        }

        // 4. Query-relevant conversation history (if space allows)
        let remaining_tokens = max_tokens.saturating_sub(used_tokens);
        if remaining_tokens > 100 {
            // Exclude FIFO turns (already included)
            let fifo_ids: std::collections::HashSet<_> = fifo.iter().map(|t| t.id).collect();
            let recall_results = self.conversation_search(query, Some(3));

            let relevant_turns: Vec<_> = recall_results
                .into_iter()
                .filter(|r| !fifo_ids.contains(&r.turn.id))
                .collect();

            if !relevant_turns.is_empty() {
                let mut history_context = String::new();
                let mut history_tokens = 0;

                for result in relevant_turns {
                    let turn_text = result.turn.format_for_context();
                    let turn_tokens = turn_text.len() / 4;

                    if history_tokens + turn_tokens <= remaining_tokens {
                        history_context.push_str(&turn_text);
                        history_context.push('\n');
                        history_tokens += turn_tokens;
                    }
                }

                if !history_context.is_empty() {
                    context.push_str("\n## Earlier Relevant Discussion\n");
                    context.push_str(&history_context);
                }
            }
        }

        context
    }

    /// Compact with tracking and return compression stats
    pub async fn compact_with_stats(&self) -> Result<CompressionStats, String> {
        let pending = self.recall.get_pending_summarization();

        if pending.is_empty() {
            return Ok(CompressionStats::default());
        }

        // Calculate original tokens
        let original_tokens: usize = pending.iter().map(|t| t.estimated_tokens).sum();

        // Summarize
        let summary = self.summarize_turns(&pending).await?;
        let compressed_tokens = summary.len() / 4;

        // Store summary in archival
        let note = MemoryNote::new(&self.session_id, &summary, MemoryType::ConversationSummary)
            .with_context("Conversation summary")
            .with_tags(vec!["summary".to_string(), "compressed".to_string()]);

        self.archival.insert(note);

        let stats = CompressionStats::new(
            original_tokens,
            compressed_tokens,
            pending.len(),
            self.recall.get_fifo().len(),
        );

        tracing::debug!(
            turns = pending.len(),
            original_tokens = original_tokens,
            compressed_tokens = compressed_tokens,
            ratio = stats.compression_ratio,
            "Compacted conversation with {}x compression",
            stats.compression_ratio
        );

        Ok(stats)
    }

    /// Set compression level for automatic compaction
    pub fn set_compression_level(&mut self, level: CompressionLevel) {
        // Update recall memory FIFO size based on compression level
        // Note: This requires mutable access, which is why we have &mut self
        let _ = level; // Used to determine aggressiveness
        // The actual implementation would need RecallMemoryConfig to be mutable
        tracing::debug!("Compression level set to {:?}", level);
    }

    /// Clear all memory for this session
    pub fn clear(&self) {
        self.core.clear_human_block();
        self.core.clear_persona_goals();
        self.recall.clear();
        self.archival.clear_session(&self.session_id);
    }

    /// Reset to default state (including persona)
    pub fn reset(&self) {
        self.core.reset();
        self.recall.clear();
        self.archival.clear_session(&self.session_id);
    }
}

impl Default for AgenticMemory {
    fn default() -> Self {
        Self::new(AgenticMemoryConfig::default(), Uuid::new_v4().to_string())
    }
}

// ============================================================================
// Backward Compatibility - Re-export legacy types
// ============================================================================

// Re-export config from voice_agent_config for backward compatibility
pub use voice_agent_config::MemoryConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agentic_memory_creation() {
        let memory = AgenticMemory::with_session("test-session");
        assert_eq!(memory.session_id(), "test-session");
    }

    #[test]
    fn test_core_memory_functions() {
        let memory = AgenticMemory::with_session("test-session");

        // Append
        assert!(memory.core_memory_append("loan_amount", "500000").is_ok());

        // Verify
        let human = memory.core.human_snapshot();
        assert!(human.get_fact("loan_amount").is_some());

        // Replace
        assert!(memory
            .core_memory_replace("loan_amount", "500000", "750000")
            .is_ok());
        let human = memory.core.human_snapshot();
        assert_eq!(human.get_fact("loan_amount").unwrap().value, "750000");
    }

    #[test]
    fn test_conversation_flow() {
        let memory = AgenticMemory::with_session("test-session");

        memory.add_user_turn("I want a gold loan");
        memory.add_assistant_turn("Sure! How much gold do you have?");
        memory.add_user_turn("About 50 grams");

        assert_eq!(memory.recall.len(), 3);

        let recent = memory.get_recent_turns();
        assert!(!recent.is_empty());
    }

    #[test]
    fn test_archival_memory() {
        let memory = AgenticMemory::with_session("test-session");

        let id = memory.archival_memory_insert("Customer prefers Hindi", MemoryType::Preference);
        assert!(!id.is_nil());

        let results = memory.archival_memory_search("Hindi", Some(5));
        assert!(!results.is_empty());
    }

    #[test]
    fn test_conversation_search() {
        let memory = AgenticMemory::with_session("test-session");

        memory.add_user_turn("I have 50 grams of gold");
        memory.add_user_turn("The purity is 22 karat");

        let results = memory.conversation_search("gold", Some(5));
        assert!(!results.is_empty());
    }

    #[test]
    fn test_context_generation() {
        let memory = AgenticMemory::with_session("test-session");

        memory.core.set_customer_name("Rajesh");
        memory.add_user_turn("I need a gold loan");
        memory.add_assistant_turn("I can help with that!");

        let context = memory.get_context();

        assert!(context.contains("Priya")); // Default persona
        assert!(context.contains("Rajesh"));
        assert!(context.contains("gold loan"));
    }

    #[test]
    fn test_memory_stats() {
        let memory = AgenticMemory::with_session("test-session");

        memory.add_user_turn("Hello");
        memory.add_assistant_turn("Hi!");

        let stats = memory.get_stats();
        assert!(stats.fifo_tokens > 0);
        assert!(stats.core_tokens > 0);
    }

    // =========================================================================
    // Context Compression Tests
    // =========================================================================

    #[test]
    fn test_compression_stats() {
        let stats = CompressionStats::new(1000, 250, 10, 4);

        assert_eq!(stats.original_tokens, 1000);
        assert_eq!(stats.compressed_tokens, 250);
        assert_eq!(stats.turns_summarized, 10);
        assert_eq!(stats.turns_intact, 4);
        assert!((stats.compression_ratio - 4.0).abs() < 0.01);
        assert!(stats.is_effective()); // 4x > 1.5x
    }

    #[test]
    fn test_compression_stats_not_effective() {
        let stats = CompressionStats::new(100, 80, 2, 4);

        assert!(!stats.is_effective()); // 1.25x < 1.5x
    }

    #[test]
    fn test_compression_levels() {
        assert_eq!(CompressionLevel::Conservative.recency_window(), 8);
        assert_eq!(CompressionLevel::Balanced.recency_window(), 6);
        assert_eq!(CompressionLevel::Aggressive.recency_window(), 4);
        assert_eq!(CompressionLevel::Maximum.recency_window(), 2);

        assert!(CompressionLevel::Maximum.target_ratio() > CompressionLevel::Conservative.target_ratio());
    }

    #[test]
    fn test_rule_based_summary_with_entities() {
        let memory = AgenticMemory::with_session("test-session");

        // Create turns with entities
        let mut turn1 = ConversationTurn::new(TurnRole::User, "I want a gold loan");
        turn1.entities.push(("gold_weight".to_string(), "50 grams".to_string()));

        let mut turn2 = ConversationTurn::new(TurnRole::User, "My name is Rajesh");
        turn2.entities.push(("customer_name".to_string(), "Rajesh".to_string()));

        let turns = vec![turn1, turn2];
        let summary = memory.rule_based_summary(&turns);

        // Should extract entities
        assert!(summary.contains("Gold: 50 grams") || summary.contains("gold"));
        assert!(summary.contains("Rajesh") || summary.contains("Name"));
    }

    #[test]
    fn test_rule_based_summary_pattern_extraction() {
        let memory = AgenticMemory::with_session("test-session");

        let turns = vec![
            ConversationTurn::new(TurnRole::User, "My name is Rahul Kumar"),
            ConversationTurn::new(TurnRole::User, "I have about 100 gram gold"),
            ConversationTurn::new(TurnRole::User, "I need 5 lakh loan"),
            ConversationTurn::new(TurnRole::User, "Currently with Muthoot"),
        ];

        let summary = memory.rule_based_summary(&turns);

        // Should extract patterns even without entities
        assert!(
            summary.to_lowercase().contains("rahul") ||
            summary.to_lowercase().contains("gram") ||
            summary.to_lowercase().contains("lakh") ||
            summary.to_lowercase().contains("muthoot"),
            "Summary should contain extracted info: {}", summary
        );
    }

    #[test]
    fn test_selective_context_injection() {
        let memory = AgenticMemory::with_session("test-session");

        // Set up core memory
        memory.core.set_customer_name("Test Customer");

        // Add some conversation
        memory.add_user_turn("I want information about gold loan rates");
        memory.add_assistant_turn("Our rates start at 10.5%");
        memory.add_user_turn("What about 50 grams of gold?");
        memory.add_assistant_turn("For 50 grams, you can get up to 2.5 lakh");

        // Add archival memory
        memory.archival_memory_insert(
            "Customer interested in competitive rates",
            MemoryType::CustomerFact,
        );

        // Get context for a specific query
        let context = memory.get_context_for_query("gold loan rates", 1000);

        // Should contain core memory
        assert!(context.contains("Test Customer") || context.contains("Priya"));

        // Should contain recent conversation
        assert!(context.contains("gold") || context.contains("Gold"));
    }

    #[test]
    fn test_context_token_limit() {
        let memory = AgenticMemory::with_session("test-session");

        // Add lots of turns
        for i in 0..20 {
            let user_msg = format!("This is user message number {} with some content", i);
            let asst_msg = format!("This is response number {} with detailed information", i);
            memory.add_user_turn(&user_msg);
            memory.add_assistant_turn(&asst_msg);
        }

        // Get limited context
        let limited_context = memory.get_context_limited(200);

        // Should be within token limit (rough estimate: 200 tokens * 4 chars = 800 chars)
        // Allow some overhead for formatting
        assert!(limited_context.len() < 2000, "Context too large: {} chars", limited_context.len());
    }

    #[test]
    fn test_extract_after_pattern() {
        let text = "Hello, my name is Rahul Kumar and I need help.";
        let name = AgenticMemory::extract_after_pattern(text, &["my name is"]);

        assert!(name.is_some());
        let extracted = name.unwrap();
        assert!(extracted.contains("Rahul"), "Expected 'Rahul' in '{}'", extracted);
    }

    #[test]
    fn test_extract_amount_with_unit() {
        let text = "i need about 5 lakh rupees for my business";
        let amount = AgenticMemory::extract_amount_with_unit(text, &["lakh"]);

        assert!(amount.is_some());
        assert!(amount.unwrap().contains("5"));

        let text2 = "i have 50 gram gold";
        let weight = AgenticMemory::extract_amount_with_unit(text2, &["gram"]);

        assert!(weight.is_some());
        assert!(weight.unwrap().contains("50"));
    }
}
