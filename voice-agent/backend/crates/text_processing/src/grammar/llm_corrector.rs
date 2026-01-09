//! LLM-based grammar correction

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use std::sync::Arc;
use voice_agent_core::{
    DomainContext, GenerateRequest, GrammarCorrector, LanguageModel, Message, Result, Role,
};

/// Grammar corrector using LLM
pub struct LLMGrammarCorrector {
    llm: Arc<dyn LanguageModel>,
    domain_context: DomainContext,
    temperature: f32,
}

impl LLMGrammarCorrector {
    /// Create a new LLM grammar corrector
    ///
    /// Note: This uses a default DomainContext. For config-driven contexts,
    /// use `with_domain_context()`.
    pub fn new(llm: Arc<dyn LanguageModel>, domain: &str, temperature: f32) -> Self {
        // Default to empty context - callers should use with_domain_context for config-driven
        let domain_context = DomainContext::new(domain);
        Self::with_domain_context(llm, domain_context, temperature)
    }

    /// Create a new LLM grammar corrector with a pre-built DomainContext
    ///
    /// This is the preferred constructor for config-driven contexts.
    pub fn with_domain_context(
        llm: Arc<dyn LanguageModel>,
        domain_context: DomainContext,
        temperature: f32,
    ) -> Self {
        Self {
            llm,
            domain_context,
            temperature,
        }
    }

    /// Build grammar correction prompt
    fn build_prompt(&self, text: &str, context: &DomainContext) -> String {
        format!(
            r#"You are a speech-to-text error corrector for a {} conversation.

DOMAIN VOCABULARY (preserve these exact spellings):
{}

COMMON PHRASES (preserve):
{}

RULES:
1. Fix obvious transcription errors (homophones, mishearing)
2. Preserve proper nouns, bank names, and numbers exactly
3. Keep the meaning identical
4. Output ONLY the corrected text, nothing else
5. If text is already correct, output it unchanged
6. Handle Hindi-English code-switching naturally
7. Fix "gol lone" → "gold loan", "kotuk" → "Kotak", etc.

INPUT: {}
CORRECTED:"#,
            context.domain,
            context.vocabulary.join(", "),
            context.phrases.join("\n"),
            text,
        )
    }
}

#[async_trait]
impl GrammarCorrector for LLMGrammarCorrector {
    async fn correct(&self, text: &str, context: &DomainContext) -> Result<String> {
        // Skip very short text
        if text.trim().len() < 3 {
            return Ok(text.to_string());
        }

        let prompt = self.build_prompt(text, context);

        let request = GenerateRequest {
            messages: vec![Message {
                role: Role::User,
                content: prompt,
                name: None,
                tool_call_id: None,
            }],
            max_tokens: Some(256),
            temperature: Some(self.temperature),
            stream: false,
            ..Default::default()
        };

        let response = self.llm.generate(request).await?;
        let corrected = response.text.trim().to_string();

        // Sanity check: if correction is wildly different in length, keep original
        let len_ratio = corrected.len() as f32 / text.len() as f32;
        if !(0.5..=2.0).contains(&len_ratio) {
            tracing::warn!(
                "Grammar correction changed length significantly ({} -> {}), keeping original",
                text.len(),
                corrected.len()
            );
            return Ok(text.to_string());
        }

        Ok(corrected)
    }

    fn correct_stream<'a>(
        &'a self,
        text_stream: Pin<Box<dyn Stream<Item = String> + Send + 'a>>,
        context: &'a DomainContext,
    ) -> Pin<Box<dyn Stream<Item = Result<String>> + Send + 'a>> {
        use futures::StreamExt;

        let ctx = context.clone();
        Box::pin(text_stream.then(move |text| {
            let ctx = ctx.clone();
            async move { self.correct(&text, &ctx).await }
        }))
    }

    fn is_enabled(&self) -> bool {
        true
    }
}

impl Clone for LLMGrammarCorrector {
    fn clone(&self) -> Self {
        Self {
            llm: self.llm.clone(),
            domain_context: self.domain_context.clone(),
            temperature: self.temperature,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a test fixture for DomainContext
    fn test_context() -> DomainContext {
        DomainContext::from_config(
            "test",
            vec!["term1".to_string(), "term2".to_string()],
            vec!["phrase1".to_string()],
            vec![("LTV".to_string(), "Loan to Value".to_string())],
            vec!["PersonName".to_string()],
            vec!["CompetitorX".to_string()],
        )
    }

    #[test]
    fn test_domain_context_from_config() {
        let context = test_context();
        assert_eq!(context.domain, "test");
        assert!(context.vocabulary.contains(&"term1".to_string()));
        assert!(context.vocabulary.contains(&"term2".to_string()));
    }
}
