//! Config-Driven Stage System
//!
//! Provides stage guidance, questions, and parameters from domain config
//! instead of hardcoded values.
//!
//! ## Architecture
//!
//! The `StageConfigProvider` wraps `StagesConfig` to provide:
//! - Stage guidance text (domain-specific)
//! - Suggested questions (domain-specific)
//! - Context budget tokens
//! - RAG context fractions
//! - History turns to keep
//! - Valid transitions
//!
//! This enables true domain-agnostic stage management where all
//! domain-specific text comes from YAML configuration.

use std::sync::Arc;
use voice_agent_config::domain::StagesConfig;

use super::stage::ConversationStage;

/// Provides stage configuration from domain config
///
/// Wraps `StagesConfig` and provides typed access to stage parameters.
/// Falls back to sensible defaults when config is not available.
pub struct StageConfigProvider {
    config: Option<Arc<StagesConfig>>,
}

impl StageConfigProvider {
    /// Create with config
    pub fn new(config: Arc<StagesConfig>) -> Self {
        Self {
            config: Some(config),
        }
    }

    /// Create with no config (uses defaults)
    pub fn default_provider() -> Self {
        Self { config: None }
    }

    /// Get stage ID string for a ConversationStage
    fn stage_id(stage: ConversationStage) -> &'static str {
        match stage {
            ConversationStage::Greeting => "greeting",
            ConversationStage::Discovery => "discovery",
            ConversationStage::Qualification => "qualification",
            ConversationStage::Presentation => "presentation",
            ConversationStage::ObjectionHandling => "objection_handling",
            ConversationStage::Closing => "closing",
            ConversationStage::Farewell => "farewell",
        }
    }

    /// Get guidance text for a stage (from config or default)
    pub fn guidance(&self, stage: ConversationStage) -> String {
        if let Some(ref config) = self.config {
            if let Some(guidance) = config.get_guidance(Self::stage_id(stage)) {
                return guidance.to_string();
            }
        }
        // Generic fallback (no domain-specific references)
        Self::default_guidance(stage).to_string()
    }

    /// Get suggested questions for a stage (from config or default)
    pub fn suggested_questions(&self, stage: ConversationStage) -> Vec<String> {
        if let Some(ref config) = self.config {
            let questions = config.get_suggested_questions(Self::stage_id(stage));
            if !questions.is_empty() {
                return questions.into_iter().map(|s| s.to_string()).collect();
            }
        }
        // Generic fallback
        Self::default_questions(stage)
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get context budget tokens for a stage (from config or default)
    pub fn context_budget_tokens(&self, stage: ConversationStage) -> usize {
        if let Some(ref config) = self.config {
            return config.get_context_budget(Self::stage_id(stage));
        }
        Self::default_context_budget(stage)
    }

    /// Get RAG context fraction for a stage (from config or default)
    pub fn rag_context_fraction(&self, stage: ConversationStage) -> f32 {
        if let Some(ref config) = self.config {
            return config.get_rag_fraction(Self::stage_id(stage));
        }
        Self::default_rag_fraction(stage)
    }

    /// Get history turns to keep for a stage (from config or default)
    pub fn history_turns_to_keep(&self, stage: ConversationStage) -> usize {
        if let Some(ref config) = self.config {
            if let Some(stage_def) = config.get_stage(Self::stage_id(stage)) {
                return stage_def.history_turns_to_keep;
            }
        }
        Self::default_history_turns(stage)
    }

    /// Get valid transitions from a stage (from config or default)
    pub fn valid_transitions(&self, stage: ConversationStage) -> Vec<ConversationStage> {
        if let Some(ref config) = self.config {
            let transitions = config.get_transitions(Self::stage_id(stage));
            if !transitions.is_empty() {
                return transitions
                    .into_iter()
                    .filter_map(|s| Self::parse_stage(s))
                    .collect();
            }
        }
        Self::default_transitions(stage)
    }

    /// Check if transition is valid (from config or default)
    pub fn is_valid_transition(&self, from: ConversationStage, to: ConversationStage) -> bool {
        if let Some(ref config) = self.config {
            return config.is_valid_transition(Self::stage_id(from), Self::stage_id(to));
        }
        self.valid_transitions(from).contains(&to)
    }

    /// Parse stage ID string to ConversationStage
    fn parse_stage(stage_id: &str) -> Option<ConversationStage> {
        match stage_id {
            "greeting" => Some(ConversationStage::Greeting),
            "discovery" => Some(ConversationStage::Discovery),
            "qualification" => Some(ConversationStage::Qualification),
            "presentation" => Some(ConversationStage::Presentation),
            "objection_handling" => Some(ConversationStage::ObjectionHandling),
            "closing" => Some(ConversationStage::Closing),
            "farewell" => Some(ConversationStage::Farewell),
            _ => None,
        }
    }

    // =========================================================================
    // Default Values (Generic - No Domain-Specific References)
    // =========================================================================

    /// Default guidance (generic, no domain references)
    fn default_guidance(stage: ConversationStage) -> &'static str {
        match stage {
            ConversationStage::Greeting => {
                "Warmly greet the customer. Introduce yourself. Build initial rapport before discussing products."
            }
            ConversationStage::Discovery => {
                "Ask open questions to understand their needs. Learn about their current situation and pain points."
            }
            ConversationStage::Qualification => {
                "Assess eligibility and readiness. Understand timeline and decision-making process."
            }
            ConversationStage::Presentation => {
                "Present tailored benefits based on their needs. Focus on value and address their specific situation."
            }
            ConversationStage::ObjectionHandling => {
                "Address concerns with empathy. Use evidence and social proof. Don't be pushy."
            }
            ConversationStage::Closing => {
                "Summarize benefits and guide to next steps. Schedule appointment or capture contact info."
            }
            ConversationStage::Farewell => {
                "Thank warmly and confirm next steps. Leave door open for future conversations."
            }
        }
    }

    /// Default questions (generic, no domain references)
    fn default_questions(stage: ConversationStage) -> Vec<&'static str> {
        match stage {
            ConversationStage::Greeting => {
                vec!["How are you doing today?", "Is this a good time to talk?"]
            }
            ConversationStage::Discovery => vec![
                "Can you tell me about your current situation?",
                "What brought you to us today?",
                "What are you looking for?",
            ],
            ConversationStage::Qualification => vec![
                "What timeline are you working with?",
                "Are you the primary decision maker?",
            ],
            ConversationStage::Presentation => vec![
                "Would you like to know more about our offering?",
                "Can I show you how this could benefit you?",
            ],
            ConversationStage::ObjectionHandling => vec![
                "What concerns do you have?",
                "Is there anything holding you back?",
            ],
            ConversationStage::Closing => vec![
                "Would you like to proceed?",
                "Can I schedule a follow-up for you?",
            ],
            ConversationStage::Farewell => vec![
                "Is there anything else I can help with?",
                "Do you have any other questions?",
            ],
        }
    }

    /// Default context budget
    fn default_context_budget(stage: ConversationStage) -> usize {
        match stage {
            ConversationStage::Greeting => 1024,
            ConversationStage::Discovery => 2048,
            ConversationStage::Qualification => 2048,
            ConversationStage::Presentation => 3584,
            ConversationStage::ObjectionHandling => 3584,
            ConversationStage::Closing => 2560,
            ConversationStage::Farewell => 1024,
        }
    }

    /// Default RAG fraction
    fn default_rag_fraction(stage: ConversationStage) -> f32 {
        match stage {
            ConversationStage::Greeting => 0.0,
            ConversationStage::Discovery => 0.15,
            ConversationStage::Qualification => 0.2,
            ConversationStage::Presentation => 0.4,
            ConversationStage::ObjectionHandling => 0.35,
            ConversationStage::Closing => 0.2,
            ConversationStage::Farewell => 0.0,
        }
    }

    /// Default history turns
    fn default_history_turns(stage: ConversationStage) -> usize {
        match stage {
            ConversationStage::Greeting => 0,
            ConversationStage::Discovery => 3,
            ConversationStage::Qualification => 4,
            ConversationStage::Presentation => 5,
            ConversationStage::ObjectionHandling => 6,
            ConversationStage::Closing => 4,
            ConversationStage::Farewell => 2,
        }
    }

    /// Default transitions
    fn default_transitions(stage: ConversationStage) -> Vec<ConversationStage> {
        match stage {
            ConversationStage::Greeting => {
                vec![ConversationStage::Discovery, ConversationStage::Farewell]
            }
            ConversationStage::Discovery => vec![
                ConversationStage::Qualification,
                ConversationStage::Presentation,
                ConversationStage::ObjectionHandling,
                ConversationStage::Farewell,
            ],
            ConversationStage::Qualification => vec![
                ConversationStage::Presentation,
                ConversationStage::Discovery,
                ConversationStage::Farewell,
            ],
            ConversationStage::Presentation => vec![
                ConversationStage::ObjectionHandling,
                ConversationStage::Closing,
                ConversationStage::Farewell,
            ],
            ConversationStage::ObjectionHandling => vec![
                ConversationStage::Presentation,
                ConversationStage::Discovery,
                ConversationStage::Closing,
                ConversationStage::Farewell,
            ],
            ConversationStage::Closing => vec![
                ConversationStage::ObjectionHandling,
                ConversationStage::Farewell,
            ],
            ConversationStage::Farewell => vec![],
        }
    }
}

impl Default for StageConfigProvider {
    fn default() -> Self {
        Self::default_provider()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_provider() {
        let provider = StageConfigProvider::default_provider();

        // Should return generic guidance
        let guidance = provider.guidance(ConversationStage::Presentation);
        assert!(!guidance.contains("Kotak"));
        assert!(!guidance.contains("gold loan"));

        // Should return sensible defaults
        assert!(provider.context_budget_tokens(ConversationStage::Presentation) > 2000);
        assert!(provider.rag_context_fraction(ConversationStage::Presentation) > 0.3);
    }

    #[test]
    fn test_valid_transitions() {
        let provider = StageConfigProvider::default_provider();

        let transitions = provider.valid_transitions(ConversationStage::Greeting);
        assert!(transitions.contains(&ConversationStage::Discovery));
        assert!(transitions.contains(&ConversationStage::Farewell));
        assert!(!transitions.contains(&ConversationStage::Closing));
    }

    #[test]
    fn test_stage_id_roundtrip() {
        let stages = [
            ConversationStage::Greeting,
            ConversationStage::Discovery,
            ConversationStage::Qualification,
            ConversationStage::Presentation,
            ConversationStage::ObjectionHandling,
            ConversationStage::Closing,
            ConversationStage::Farewell,
        ];

        for stage in stages {
            let id = StageConfigProvider::stage_id(stage);
            let parsed = StageConfigProvider::parse_stage(id);
            assert_eq!(parsed, Some(stage));
        }
    }

    #[test]
    fn test_default_questions_generic() {
        let provider = StageConfigProvider::default_provider();

        for stage in [
            ConversationStage::Greeting,
            ConversationStage::Discovery,
            ConversationStage::Presentation,
        ] {
            let questions = provider.suggested_questions(stage);
            for q in &questions {
                assert!(
                    !q.to_lowercase().contains("gold"),
                    "Question should not mention gold: {}",
                    q
                );
                assert!(
                    !q.to_lowercase().contains("kotak"),
                    "Question should not mention kotak: {}",
                    q
                );
            }
        }
    }
}
