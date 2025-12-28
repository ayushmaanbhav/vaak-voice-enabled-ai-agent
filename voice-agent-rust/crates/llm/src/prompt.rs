//! Prompt Building and Management
//!
//! Constructs prompts for the gold loan voice agent.

use std::fmt;
use serde::{Deserialize, Serialize};

// P0 FIX: Re-export PersonaConfig from config crate (single source of truth)
pub use voice_agent_config::PersonaConfig;

/// Message role
///
/// P2 FIX: Added Tool role for function calling support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    /// Tool/function response role
    Tool,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::System => write!(f, "system"),
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::Tool => write!(f, "tool"),
        }
    }
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }

    /// P2 FIX: Added tool() constructor for function calling responses.
    pub fn tool(content: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
        }
    }
}

/// Prompt builder for gold loan agent
pub struct PromptBuilder {
    messages: Vec<Message>,
    persona: PersonaConfig,
}

// PersonaConfig is now imported from voice_agent_config (see re-export above)

impl PromptBuilder {
    /// Create a new prompt builder
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            persona: PersonaConfig::default(),
        }
    }

    /// Set persona configuration
    pub fn with_persona(mut self, persona: PersonaConfig) -> Self {
        self.persona = persona;
        self
    }

    /// Build system prompt for gold loan agent
    pub fn system_prompt(mut self, language: &str) -> Self {
        let persona_traits = self.build_persona_traits();

        let system = format!(
            r#"You are {name}, a friendly and knowledgeable Gold Loan specialist at Kotak Mahindra Bank.

## Your Persona
{traits}

## Your Role
- Help customers understand gold loan products and benefits
- Guide customers through the Switch & Save program
- Answer questions about interest rates, LTV, and documentation
- Address concerns and objections with empathy
- Collect lead information when appropriate

## Communication Guidelines
- Speak naturally and conversationally
- Use {language_style} language naturally
- Keep responses concise (2-3 sentences max for voice)
- Ask one question at a time
- Acknowledge customer concerns before addressing them
- Use the customer's name when known

## Key Product Information
- Interest rates: Starting from 10.5% (vs 18-24% NBFC rates)
- LTV: Up to 75% of gold value
- Processing: Same-day disbursement
- Safety: RBI-regulated bank with insured vault storage
- Bridge loan available for seamless transfer

## Response Format
Respond naturally as if speaking on a phone call. Do not use bullet points, headers, or markdown formatting. Keep responses brief and conversational."#,
            name = self.persona.name,
            traits = persona_traits,
            language_style = if language == "hi" { "Hindi-English (Hinglish)" } else { "English" },
        );

        self.messages.push(Message::system(system));
        self
    }

    /// Build persona traits string
    fn build_persona_traits(&self) -> String {
        let mut traits = Vec::new();

        if self.persona.warmth > 0.7 {
            traits.push("- Warm and welcoming in your tone");
        }
        if self.persona.empathy > 0.8 {
            traits.push("- Highly empathetic, understanding customer concerns");
        }
        if self.persona.formality < 0.4 {
            traits.push("- Casual and friendly, like talking to a trusted friend");
        } else if self.persona.formality > 0.7 {
            traits.push("- Professional and respectful");
        } else {
            traits.push("- Balanced between professional and approachable");
        }
        if self.persona.urgency > 0.6 {
            traits.push("- Gently convey time-sensitive opportunities");
        }

        traits.join("\n")
    }

    /// Add RAG context
    pub fn with_context(mut self, context: &str) -> Self {
        if !context.is_empty() {
            let context_msg = format!(
                "## Relevant Information\n{}\n\nUse this information to answer the customer's question if relevant.",
                context
            );
            self.messages.push(Message::system(context_msg));
        }
        self
    }

    /// Add customer profile
    pub fn with_customer(mut self, name: Option<&str>, segment: Option<&str>, history: Option<&str>) -> Self {
        let mut profile_parts = Vec::new();

        if let Some(n) = name {
            profile_parts.push(format!("Customer name: {}", n));
        }
        if let Some(s) = segment {
            profile_parts.push(format!("Segment: {}", s));
        }
        if let Some(h) = history {
            profile_parts.push(format!("History: {}", h));
        }

        if !profile_parts.is_empty() {
            let profile = format!("## Customer Profile\n{}", profile_parts.join("\n"));
            self.messages.push(Message::system(profile));
        }
        self
    }

    /// Add conversation history
    pub fn with_history(mut self, history: &[Message]) -> Self {
        self.messages.extend(history.iter().cloned());
        self
    }

    /// Add current user message
    pub fn user_message(mut self, message: &str) -> Self {
        self.messages.push(Message::user(message));
        self
    }

    /// Add stage guidance
    pub fn with_stage_guidance(mut self, stage: &str) -> Self {
        let guidance = match stage {
            "greeting" => "Warmly greet the customer and introduce yourself. Build rapport before discussing products.",
            "discovery" => "Ask open questions to understand their gold loan needs and current situation with competitors.",
            "qualification" => "Assess their eligibility and readiness to switch. Understand loan amount and timeline.",
            "presentation" => "Present Kotak's gold loan benefits, focusing on their specific needs and concerns.",
            "objection_handling" => "Address concerns with empathy. Use social proof and guarantees to build confidence.",
            "closing" => "Summarize benefits and guide them to next steps. Create urgency if appropriate.",
            "farewell" => "Thank them warmly and confirm next steps. Leave the door open for future conversations.",
            _ => "",
        };

        if !guidance.is_empty() {
            self.messages.push(Message::system(format!("## Current Stage Guidance\n{}", guidance)));
        }
        self
    }

    /// Build final message list
    pub fn build(self) -> Vec<Message> {
        self.messages
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Estimate token count
    pub fn estimate_tokens(&self) -> usize {
        self.messages
            .iter()
            .map(|m| m.content.len() / 4) // Rough estimate
            .sum()
    }
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick response templates
pub struct ResponseTemplates;

impl ResponseTemplates {
    /// Greeting template
    pub fn greeting(name: &str, language: &str) -> String {
        if language == "hi" {
            format!("Namaste! Main {} hoon, Kotak Mahindra Bank se. Aapki madad karne ke liye yahan hoon.", name)
        } else {
            format!("Hello! I'm {} from Kotak Mahindra Bank. I'm here to help you today.", name)
        }
    }

    /// Acknowledgment
    pub fn acknowledge(language: &str) -> String {
        if language == "hi" {
            "Ji bilkul, main samajh sakti hoon.".to_string()
        } else {
            "I understand, that makes sense.".to_string()
        }
    }

    /// Clarification request
    pub fn clarify(language: &str) -> String {
        if language == "hi" {
            "Kya aap thoda aur bata sakte hain?".to_string()
        } else {
            "Could you tell me a bit more about that?".to_string()
        }
    }

    /// Transition to next topic
    pub fn transition(language: &str) -> String {
        if language == "hi" {
            "Achha, ab main aapko batati hoon...".to_string()
        } else {
            "Great, let me tell you about...".to_string()
        }
    }

    /// Closing
    pub fn closing(language: &str) -> String {
        if language == "hi" {
            "Dhanyavaad aapka samay dene ke liye. Koi bhi sawal ho toh zaroor call karein.".to_string()
        } else {
            "Thank you for your time. Please feel free to call if you have any questions.".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_prompt_builder() {
        let messages = PromptBuilder::new()
            .system_prompt("en")
            .user_message("What is your interest rate?")
            .build();

        assert!(messages.len() >= 2);
        assert_eq!(messages[0].role, Role::System);
    }

    #[test]
    fn test_with_context() {
        let messages = PromptBuilder::new()
            .system_prompt("en")
            .with_context("Interest rate is 10.5%")
            .user_message("Tell me about rates")
            .build();

        // Should have system prompt, context, and user message
        assert!(messages.len() >= 3);
    }

    #[test]
    fn test_templates() {
        let greeting = ResponseTemplates::greeting("Priya", "hi");
        assert!(greeting.contains("Namaste"));

        let greeting_en = ResponseTemplates::greeting("Priya", "en");
        assert!(greeting_en.contains("Hello"));
    }
}
