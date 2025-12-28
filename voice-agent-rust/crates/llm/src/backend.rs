//! LLM Backend implementations
//!
//! Supports Ollama, Claude API, and OpenAI API.

use std::time::Duration;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::prompt::Message;
use crate::LlmError;

/// LLM configuration
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Model name/ID
    pub model: String,
    /// API endpoint
    pub endpoint: String,
    /// API key (optional)
    pub api_key: Option<String>,
    /// Maximum tokens to generate
    pub max_tokens: usize,
    /// Temperature
    pub temperature: f32,
    /// Top-p sampling
    pub top_p: f32,
    /// Request timeout
    pub timeout: Duration,
    /// Enable streaming
    pub stream: bool,
    /// P1 FIX: Maximum retry attempts for transient failures
    pub max_retries: u32,
    /// P1 FIX: Initial backoff duration (doubles each retry)
    pub initial_backoff: Duration,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            model: "qwen2.5:7b-instruct-q4_K_M".to_string(),
            endpoint: "http://localhost:11434".to_string(),
            api_key: None,
            max_tokens: 256,
            temperature: 0.7,
            top_p: 0.9,
            timeout: Duration::from_secs(30),
            stream: true,
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
        }
    }
}

/// LLM generation result
#[derive(Debug, Clone)]
pub struct GenerationResult {
    /// Generated text
    pub text: String,
    /// Tokens generated
    pub tokens: usize,
    /// Time to first token (ms)
    pub time_to_first_token_ms: u64,
    /// Total generation time (ms)
    pub total_time_ms: u64,
    /// Tokens per second
    pub tokens_per_second: f32,
    /// Finish reason
    pub finish_reason: FinishReason,
}

/// Finish reason
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinishReason {
    Stop,
    Length,
    Error,
    Cancelled,
}

/// LLM Backend trait
#[async_trait]
pub trait LlmBackend: Send + Sync {
    /// Generate a response
    async fn generate(&self, messages: &[Message]) -> Result<GenerationResult, LlmError>;

    /// Generate with streaming
    async fn generate_stream(
        &self,
        messages: &[Message],
        tx: mpsc::Sender<String>,
    ) -> Result<GenerationResult, LlmError>;

    /// Check if model is available
    async fn is_available(&self) -> bool;

    /// Get model name
    fn model_name(&self) -> &str;

    /// Estimate tokens
    fn estimate_tokens(&self, text: &str) -> usize {
        // Rough estimate: ~4 characters per token
        text.len() / 4
    }
}

/// Ollama backend
///
/// P2 FIX: Now derives Clone for better composability.
#[derive(Clone)]
pub struct OllamaBackend {
    client: Client,
    config: LlmConfig,
}

impl OllamaBackend {
    /// Create a new Ollama backend
    ///
    /// P1 FIX: Now returns Result instead of panicking on client creation failure.
    pub fn new(config: LlmConfig) -> Result<Self, LlmError> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| LlmError::Configuration(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// Build the API URL
    fn api_url(&self, path: &str) -> String {
        format!("{}/api{}", self.config.endpoint, path)
    }

    /// P1 FIX: Execute a single request (used by retry logic)
    async fn execute_request(&self, request: &OllamaChatRequest) -> Result<OllamaChatResponse, LlmError> {
        let response = self.client
            .post(self.api_url("/chat"))
            .json(request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error = response.text().await.unwrap_or_default();
            // 5xx errors are retryable, 4xx are not
            if status.is_server_error() {
                return Err(LlmError::Network(format!("Server error {}: {}", status, error)));
            }
            return Err(LlmError::Api(error));
        }

        response.json().await
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))
    }

    /// P1 FIX: Check if an error is retryable
    fn is_retryable(error: &LlmError) -> bool {
        matches!(error,
            LlmError::Network(_) |
            LlmError::Timeout
        )
    }
}

#[async_trait]
impl LlmBackend for OllamaBackend {
    /// Generate a response with retry logic for transient failures
    ///
    /// P1 FIX: Implements exponential backoff retry for network errors.
    async fn generate(&self, messages: &[Message]) -> Result<GenerationResult, LlmError> {
        let start = std::time::Instant::now();

        let request = OllamaChatRequest {
            model: self.config.model.clone(),
            messages: messages.iter().map(|m| m.into()).collect(),
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(self.config.temperature),
                top_p: Some(self.config.top_p),
                num_predict: Some(self.config.max_tokens as i32),
            }),
        };

        // P1 FIX: Retry loop with exponential backoff
        let mut last_error = None;
        let mut backoff = self.config.initial_backoff;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                tracing::warn!(
                    "LLM request failed, retrying in {:?} (attempt {}/{})",
                    backoff, attempt, self.config.max_retries
                );
                tokio::time::sleep(backoff).await;
                backoff *= 2; // Exponential backoff
            }

            match self.execute_request(&request).await {
                Ok(result) => {
                    let total_time = start.elapsed();
                    return Ok(GenerationResult {
                        text: result.message.content,
                        tokens: result.eval_count.unwrap_or(0) as usize,
                        time_to_first_token_ms: result.prompt_eval_duration.unwrap_or(0) / 1_000_000,
                        total_time_ms: total_time.as_millis() as u64,
                        tokens_per_second: result.eval_count.unwrap_or(0) as f32 /
                            (result.eval_duration.unwrap_or(1) as f32 / 1e9),
                        finish_reason: if result.done { FinishReason::Stop } else { FinishReason::Length },
                    });
                }
                Err(e) if Self::is_retryable(&e) => {
                    last_error = Some(e);
                }
                Err(e) => {
                    // Non-retryable error, fail immediately
                    return Err(e);
                }
            }
        }

        // All retries exhausted
        Err(last_error.unwrap_or_else(|| LlmError::Network("Max retries exceeded".to_string())))
    }

    async fn generate_stream(
        &self,
        messages: &[Message],
        tx: mpsc::Sender<String>,
    ) -> Result<GenerationResult, LlmError> {
        let start = std::time::Instant::now();
        let mut first_token_time = None;
        let mut total_tokens = 0;
        let mut full_response = String::new();

        let request = OllamaChatRequest {
            model: self.config.model.clone(),
            messages: messages.iter().map(|m| m.into()).collect(),
            stream: true,
            options: Some(OllamaOptions {
                temperature: Some(self.config.temperature),
                top_p: Some(self.config.top_p),
                num_predict: Some(self.config.max_tokens as i32),
            }),
        };

        let response = self.client
            .post(self.api_url("/chat"))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error = response.text().await.unwrap_or_default();
            return Err(LlmError::Api(error));
        }

        // Read streaming response
        let mut stream = response.bytes_stream();
        use futures::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);

            // Parse each line (NDJSON)
            for line in text.lines() {
                if line.is_empty() {
                    continue;
                }

                if let Ok(chunk_response) = serde_json::from_str::<OllamaStreamChunk>(line) {
                    if first_token_time.is_none() {
                        first_token_time = Some(start.elapsed());
                    }

                    let token = &chunk_response.message.content;
                    full_response.push_str(token);
                    total_tokens += 1;

                    // Send token to channel
                    if tx.send(token.clone()).await.is_err() {
                        // Channel closed, generation cancelled
                        return Ok(GenerationResult {
                            text: full_response,
                            tokens: total_tokens,
                            time_to_first_token_ms: first_token_time
                                .map(|t| t.as_millis() as u64)
                                .unwrap_or(0),
                            total_time_ms: start.elapsed().as_millis() as u64,
                            tokens_per_second: 0.0,
                            finish_reason: FinishReason::Cancelled,
                        });
                    }

                    if chunk_response.done {
                        break;
                    }
                }
            }
        }

        let total_time = start.elapsed();

        Ok(GenerationResult {
            text: full_response,
            tokens: total_tokens,
            time_to_first_token_ms: first_token_time
                .map(|t| t.as_millis() as u64)
                .unwrap_or(0),
            total_time_ms: total_time.as_millis() as u64,
            tokens_per_second: total_tokens as f32 / total_time.as_secs_f32(),
            finish_reason: FinishReason::Stop,
        })
    }

    async fn is_available(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.config.endpoint))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }
}

// Ollama API types
#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

impl From<&Message> for OllamaMessage {
    fn from(msg: &Message) -> Self {
        Self {
            role: msg.role.to_string(),
            content: msg.content.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessage,
    done: bool,
    #[serde(default)]
    eval_count: Option<u64>,
    #[serde(default)]
    eval_duration: Option<u64>,
    #[serde(default)]
    prompt_eval_duration: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OllamaStreamChunk {
    message: OllamaMessage,
    done: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::Role;

    #[test]
    fn test_config_default() {
        let config = LlmConfig::default();
        assert!(config.stream);
        assert_eq!(config.temperature, 0.7);
    }

    #[test]
    fn test_message_conversion() {
        let msg = Message {
            role: Role::User,
            content: "Hello".to_string(),
        };
        let ollama_msg: OllamaMessage = (&msg).into();
        assert_eq!(ollama_msg.role, "user");
        assert_eq!(ollama_msg.content, "Hello");
    }
}
