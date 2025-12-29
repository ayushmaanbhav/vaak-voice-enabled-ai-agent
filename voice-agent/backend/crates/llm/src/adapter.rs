//! P0 FIX: Language Model adapter
//!
//! Bridges the LlmBackend trait to the core LanguageModel trait,
//! allowing LLM backends to be used where LanguageModel is expected.

use std::pin::Pin;
use std::sync::Arc;
use async_trait::async_trait;
use futures::Stream;
use tokio::sync::mpsc;

use voice_agent_core::{
    LanguageModel,
    GenerateRequest, GenerateResponse, StreamChunk, ToolDefinition,
    Result, Error,
    llm_types::{FinishReason as CoreFinishReason, TokenUsage},
};

use crate::backend::{LlmBackend, FinishReason as BackendFinishReason};

/// Adapter that wraps an LlmBackend to implement the core LanguageModel trait.
///
/// This allows using any LlmBackend implementation (Ollama, OpenAI, Claude)
/// where the core LanguageModel trait is expected.
///
/// # Example
///
/// ```ignore
/// let backend = OllamaBackend::new(config)?;
/// let language_model: Box<dyn LanguageModel> = Box::new(
///     LanguageModelAdapter::new(backend)
/// );
/// ```
pub struct LanguageModelAdapter {
    backend: Arc<dyn LlmBackend>,
    model_name: String,
}

impl LanguageModelAdapter {
    /// Create a new adapter wrapping an LlmBackend
    pub fn new<B: LlmBackend + 'static>(backend: B) -> Self {
        let model_name = backend.model_name().to_string();
        Self {
            backend: Arc::new(backend),
            model_name,
        }
    }

    /// Create from an Arc'd backend
    pub fn from_arc(backend: Arc<dyn LlmBackend>) -> Self {
        let model_name = backend.model_name().to_string();
        Self { backend, model_name }
    }

    /// Convert core messages to backend messages
    fn convert_messages(request: &GenerateRequest) -> Vec<crate::prompt::Message> {
        request.messages.iter().map(|m| {
            crate::prompt::Message {
                role: match m.role {
                    voice_agent_core::llm_types::Role::System => crate::prompt::Role::System,
                    voice_agent_core::llm_types::Role::User => crate::prompt::Role::User,
                    voice_agent_core::llm_types::Role::Assistant => crate::prompt::Role::Assistant,
                    voice_agent_core::llm_types::Role::Tool => crate::prompt::Role::User, // Map tool to user
                },
                content: m.content.clone(),
            }
        }).collect()
    }

    /// Convert backend finish reason to core finish reason
    fn convert_finish_reason(reason: BackendFinishReason) -> CoreFinishReason {
        match reason {
            BackendFinishReason::Stop => CoreFinishReason::Stop,
            BackendFinishReason::Length => CoreFinishReason::Length,
            BackendFinishReason::Error => CoreFinishReason::Error,
            BackendFinishReason::Cancelled => CoreFinishReason::Error,
        }
    }
}

#[async_trait]
impl LanguageModel for LanguageModelAdapter {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse> {
        let messages = Self::convert_messages(&request);

        match self.backend.generate(&messages).await {
            Ok(result) => {
                Ok(GenerateResponse {
                    text: result.text,
                    finish_reason: Self::convert_finish_reason(result.finish_reason),
                    usage: Some(TokenUsage::new(0, result.tokens as u32)),
                    tool_calls: Vec::new(),
                })
            }
            Err(e) => Err(Error::Llm(format!("LLM generation failed: {}", e))),
        }
    }

    fn generate_stream<'a>(
        &'a self,
        request: GenerateRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send + 'a>> {
        let messages = Self::convert_messages(&request);
        let backend = self.backend.clone();

        Box::pin(async_stream::stream! {
            let (tx, mut rx) = mpsc::channel::<String>(100);

            // Spawn the streaming task
            let stream_task = tokio::spawn(async move {
                backend.generate_stream(&messages, tx).await
            });

            // Yield chunks as they arrive
            while let Some(text) = rx.recv().await {
                yield Ok(StreamChunk::text(text));
            }

            // Wait for task completion and yield final chunk
            match stream_task.await {
                Ok(Ok(result)) => {
                    yield Ok(StreamChunk {
                        delta: String::new(),
                        is_final: true,
                        finish_reason: Some(Self::convert_finish_reason(result.finish_reason)),
                    });
                }
                Ok(Err(e)) => {
                    yield Err(Error::Llm(format!("Stream error: {}", e)));
                }
                Err(e) => {
                    yield Err(Error::Llm(format!("Task join error: {}", e)));
                }
            }
        })
    }

    async fn generate_with_tools(
        &self,
        request: GenerateRequest,
        tools: &[ToolDefinition],
    ) -> Result<GenerateResponse> {
        // P0-3: Implement tool calling for Ollama via text injection
        // For Ollama, we inject tool definitions into the prompt and parse [TOOL_CALL: ...]
        // Claude backend has native tool support via ClaudeBackend::generate_with_tools

        if tools.is_empty() {
            return self.generate(request).await;
        }

        // Build prompt with tool definitions injected
        let mut messages = Self::convert_messages(&request);

        // Add tool definitions to system prompt
        let tool_prompt = crate::prompt::PromptBuilder::new()
            .with_tools(tools)
            .build();

        // Prepend tool definitions to messages
        if let Some(tool_msg) = tool_prompt.first() {
            messages.insert(0, crate::prompt::Message {
                role: crate::prompt::Role::System,
                content: tool_msg.content.clone(),
            });
        }

        match self.backend.generate(&messages).await {
            Ok(result) => {
                // Parse tool calls from response text
                let tool_calls: Vec<voice_agent_core::llm_types::ToolCall> = crate::prompt::parse_tool_call(&result.text)
                    .map(|tc| voice_agent_core::llm_types::ToolCall {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: tc.name,
                        arguments: tc.arguments.as_object()
                            .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                            .unwrap_or_default(),
                    })
                    .into_iter()
                    .collect();

                let finish_reason = if !tool_calls.is_empty() {
                    CoreFinishReason::ToolCalls
                } else {
                    Self::convert_finish_reason(result.finish_reason)
                };

                Ok(GenerateResponse {
                    text: result.text,
                    finish_reason,
                    usage: Some(TokenUsage::new(0, result.tokens as u32)),
                    tool_calls,
                })
            }
            Err(e) => Err(Error::Llm(format!("LLM generation failed: {}", e))),
        }
    }

    async fn is_available(&self) -> bool {
        self.backend.is_available().await
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn context_size(&self) -> usize {
        // Default context size, could be made configurable
        4096
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        self.backend.estimate_tokens(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock backend for testing
    struct MockBackend {
        response: String,
    }

    impl MockBackend {
        fn new(response: &str) -> Self {
            Self { response: response.to_string() }
        }
    }

    #[async_trait]
    impl LlmBackend for MockBackend {
        async fn generate(&self, _messages: &[crate::prompt::Message]) -> std::result::Result<crate::backend::GenerationResult, crate::LlmError> {
            Ok(crate::backend::GenerationResult {
                text: self.response.clone(),
                tokens: 10,
                time_to_first_token_ms: 50,
                total_time_ms: 100,
                tokens_per_second: 100.0,
                finish_reason: BackendFinishReason::Stop,
                context: None,
            })
        }

        async fn generate_stream(
            &self,
            _messages: &[crate::prompt::Message],
            tx: mpsc::Sender<String>,
        ) -> std::result::Result<crate::backend::GenerationResult, crate::LlmError> {
            // Send response in chunks
            for word in self.response.split_whitespace() {
                let _ = tx.send(format!("{} ", word)).await;
            }
            Ok(crate::backend::GenerationResult {
                text: self.response.clone(),
                tokens: 10,
                time_to_first_token_ms: 50,
                total_time_ms: 100,
                tokens_per_second: 100.0,
                finish_reason: BackendFinishReason::Stop,
                context: None,
            })
        }

        async fn is_available(&self) -> bool {
            true
        }

        fn model_name(&self) -> &str {
            "mock-model"
        }
    }

    #[tokio::test]
    async fn test_adapter_generate() {
        let backend = MockBackend::new("Hello, world!");
        let adapter = LanguageModelAdapter::new(backend);

        let request = GenerateRequest::new("You are helpful")
            .with_user_message("Hi");

        let response = adapter.generate(request).await.unwrap();
        assert_eq!(response.text, "Hello, world!");
        assert_eq!(response.finish_reason, CoreFinishReason::Stop);
    }

    #[tokio::test]
    async fn test_adapter_is_available() {
        let backend = MockBackend::new("test");
        let adapter = LanguageModelAdapter::new(backend);
        assert!(adapter.is_available().await);
    }

    #[test]
    fn test_adapter_model_name() {
        let backend = MockBackend::new("test");
        let adapter = LanguageModelAdapter::new(backend);
        assert_eq!(adapter.model_name(), "mock-model");
    }
}
