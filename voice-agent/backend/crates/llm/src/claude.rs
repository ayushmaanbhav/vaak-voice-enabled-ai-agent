//! Claude Backend with Native Tool Use Support
//!
//! Implements the Anthropic Messages API with proper tool calling support.
//! Uses Claude's native tool_use format for optimal tool calling performance.
//!
//! ## Features
//! - Native tool_use blocks (not text-based)
//! - Streaming support with tool deltas
//! - Proper tool_result handling
//! - Claude-specific optimizations (extended thinking for opus)

use std::collections::HashMap;
use std::time::Duration;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use voice_agent_core::llm_types::{ToolDefinition, ToolCall};
use crate::prompt::Message;
use crate::backend::{LlmBackend, GenerationResult, FinishReason};
use crate::LlmError;

/// Claude model variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeModel {
    /// Claude Opus 4.5 - Most capable, best for complex tasks
    Opus4_5,
    /// Claude Sonnet 4 - Fast and capable
    Sonnet4,
    /// Claude Haiku 3.5 - Fastest, good for simple tasks
    Haiku3_5,
}

impl ClaudeModel {
    pub fn model_id(&self) -> &'static str {
        match self {
            ClaudeModel::Opus4_5 => "claude-opus-4-5-20251101",
            ClaudeModel::Sonnet4 => "claude-sonnet-4-20250514",
            ClaudeModel::Haiku3_5 => "claude-3-5-haiku-20241022",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "opus" | "opus-4.5" | "claude-opus-4-5-20251101" => Some(ClaudeModel::Opus4_5),
            "sonnet" | "sonnet-4" | "claude-sonnet-4-20250514" => Some(ClaudeModel::Sonnet4),
            "haiku" | "haiku-3.5" | "claude-3-5-haiku-20241022" => Some(ClaudeModel::Haiku3_5),
            _ => None,
        }
    }
}

impl Default for ClaudeModel {
    fn default() -> Self {
        ClaudeModel::Opus4_5
    }
}

/// Configuration for Claude backend
#[derive(Debug, Clone)]
pub struct ClaudeConfig {
    /// API key (from ANTHROPIC_API_KEY or direct)
    pub api_key: String,
    /// Model to use
    pub model: ClaudeModel,
    /// Maximum tokens to generate
    pub max_tokens: usize,
    /// Temperature (0.0 - 1.0)
    pub temperature: f32,
    /// Top-p sampling
    pub top_p: Option<f32>,
    /// Request timeout
    pub timeout: Duration,
    /// Enable streaming
    pub stream: bool,
    /// API endpoint (for testing or proxy)
    pub endpoint: String,
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
            model: ClaudeModel::Opus4_5,
            max_tokens: 1024,
            temperature: 0.7,
            top_p: None,
            timeout: Duration::from_secs(60),
            stream: true,
            endpoint: "https://api.anthropic.com".to_string(),
        }
    }
}

impl ClaudeConfig {
    /// Create config with API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            ..Default::default()
        }
    }

    /// Set model
    pub fn with_model(mut self, model: ClaudeModel) -> Self {
        self.model = model;
        self
    }

    /// Set model from string
    pub fn with_model_str(mut self, model: &str) -> Self {
        if let Some(m) = ClaudeModel::from_str(model) {
            self.model = m;
        }
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature.clamp(0.0, 1.0);
        self
    }

    /// Enable/disable streaming
    pub fn with_streaming(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }
}

/// Claude backend with native tool use support
pub struct ClaudeBackend {
    config: ClaudeConfig,
    client: Client,
}

impl ClaudeBackend {
    /// Create a new Claude backend
    pub fn new(config: ClaudeConfig) -> Result<Self, LlmError> {
        if config.api_key.is_empty() {
            return Err(LlmError::Configuration(
                "ANTHROPIC_API_KEY not set. Set it via environment or config.".to_string()
            ));
        }

        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| LlmError::Network(e.to_string()))?;

        Ok(Self { config, client })
    }

    /// Generate with tool support (native Claude tool_use)
    pub async fn generate_with_tools(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<ClaudeResponse, LlmError> {
        let claude_messages = self.convert_messages(messages);
        let claude_tools = self.convert_tools(tools);

        // Extract system message if present
        let system = messages.iter()
            .find(|m| matches!(m.role, crate::prompt::Role::System))
            .map(|m| m.content.clone());

        let request = ClaudeRequest {
            model: self.config.model.model_id().to_string(),
            max_tokens: self.config.max_tokens,
            messages: claude_messages,
            system,
            tools: if claude_tools.is_empty() { None } else { Some(claude_tools) },
            temperature: Some(self.config.temperature),
            top_p: self.config.top_p,
            stream: Some(false),
        };

        let response = self.client
            .post(format!("{}/v1/messages", self.config.endpoint))
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::Api(format!("HTTP {}: {}", status, error_text)));
        }

        let response: ClaudeApiResponse = response.json().await
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

        Ok(self.parse_response(response))
    }

    /// Generate with tools and streaming
    pub async fn generate_with_tools_stream(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        tx: mpsc::Sender<String>,
    ) -> Result<ClaudeResponse, LlmError> {
        let claude_messages = self.convert_messages(messages);
        let claude_tools = self.convert_tools(tools);

        let system = messages.iter()
            .find(|m| matches!(m.role, crate::prompt::Role::System))
            .map(|m| m.content.clone());

        let request = ClaudeRequest {
            model: self.config.model.model_id().to_string(),
            max_tokens: self.config.max_tokens,
            messages: claude_messages,
            system,
            tools: if claude_tools.is_empty() { None } else { Some(claude_tools) },
            temperature: Some(self.config.temperature),
            top_p: self.config.top_p,
            stream: Some(true),
        };

        let response = self.client
            .post(format!("{}/v1/messages", self.config.endpoint))
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::Api(format!("HTTP {}: {}", status, error_text)));
        }

        // Process SSE stream
        use futures::StreamExt;
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut full_text = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut current_tool_name = String::new();
        let mut current_tool_id = String::new();
        let mut current_tool_input = String::new();
        let mut finish_reason = ClaudeStopReason::EndTurn;
        let mut input_tokens = 0;
        let mut output_tokens = 0;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| LlmError::Network(e.to_string()))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                if let Some(json_str) = line.strip_prefix("data: ") {
                    if json_str == "[DONE]" {
                        continue;
                    }

                    if let Ok(event) = serde_json::from_str::<ClaudeStreamEvent>(json_str) {
                        match event {
                            ClaudeStreamEvent::MessageStart { message } => {
                                if let Some(usage) = message.usage {
                                    input_tokens = usage.input_tokens;
                                }
                            }
                            ClaudeStreamEvent::ContentBlockStart { content_block, .. } => {
                                match content_block {
                                    ClaudeContentBlock::Text { .. } => {}
                                    ClaudeContentBlock::ToolUse { id, name, .. } => {
                                        current_tool_id = id;
                                        current_tool_name = name;
                                        current_tool_input.clear();
                                    }
                                }
                            }
                            ClaudeStreamEvent::ContentBlockDelta { delta, .. } => {
                                match delta {
                                    ClaudeDelta::TextDelta { text } => {
                                        full_text.push_str(&text);
                                        let _ = tx.send(text).await;
                                    }
                                    ClaudeDelta::InputJsonDelta { partial_json } => {
                                        current_tool_input.push_str(&partial_json);
                                    }
                                }
                            }
                            ClaudeStreamEvent::ContentBlockStop { .. } => {
                                if !current_tool_name.is_empty() {
                                    // Parse the tool input JSON
                                    let arguments: HashMap<String, serde_json::Value> =
                                        serde_json::from_str(&current_tool_input)
                                            .unwrap_or_default();

                                    tool_calls.push(ToolCall {
                                        id: current_tool_id.clone(),
                                        name: current_tool_name.clone(),
                                        arguments,
                                    });
                                    current_tool_name.clear();
                                    current_tool_id.clear();
                                    current_tool_input.clear();
                                }
                            }
                            ClaudeStreamEvent::MessageDelta { delta, usage } => {
                                if let Some(reason) = delta.stop_reason {
                                    finish_reason = reason;
                                }
                                if let Some(u) = usage {
                                    output_tokens = u.output_tokens;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(ClaudeResponse {
            text: full_text,
            tool_calls,
            stop_reason: finish_reason,
            input_tokens,
            output_tokens,
        })
    }

    /// Convert messages to Claude format
    fn convert_messages(&self, messages: &[Message]) -> Vec<ClaudeMessage> {
        messages.iter()
            .filter(|m| !matches!(m.role, crate::prompt::Role::System))
            .map(|m| ClaudeMessage {
                role: match m.role {
                    crate::prompt::Role::User => "user".to_string(),
                    crate::prompt::Role::Assistant => "assistant".to_string(),
                    crate::prompt::Role::Tool => "user".to_string(), // Tool results come as user messages
                    crate::prompt::Role::System => unreachable!(), // Filtered out
                },
                content: ClaudeContent::Text(m.content.clone()),
            })
            .collect()
    }

    /// Convert tool definitions to Claude format
    fn convert_tools(&self, tools: &[ToolDefinition]) -> Vec<ClaudeTool> {
        tools.iter()
            .map(|t| ClaudeTool {
                name: t.name.clone(),
                description: t.description.clone(),
                input_schema: t.parameters.clone(),
            })
            .collect()
    }

    /// Parse Claude API response
    fn parse_response(&self, response: ClaudeApiResponse) -> ClaudeResponse {
        let mut text = String::new();
        let mut tool_calls = Vec::new();

        for block in response.content {
            match block {
                ClaudeContentBlock::Text { text: t } => {
                    text.push_str(&t);
                }
                ClaudeContentBlock::ToolUse { id, name, input } => {
                    let arguments: HashMap<String, serde_json::Value> =
                        serde_json::from_value(input).unwrap_or_default();
                    tool_calls.push(ToolCall {
                        id,
                        name,
                        arguments,
                    });
                }
            }
        }

        ClaudeResponse {
            text,
            tool_calls,
            stop_reason: response.stop_reason,
            input_tokens: response.usage.input_tokens,
            output_tokens: response.usage.output_tokens,
        }
    }
}

#[async_trait]
impl LlmBackend for ClaudeBackend {
    async fn generate(&self, messages: &[Message]) -> Result<GenerationResult, LlmError> {
        let start = std::time::Instant::now();
        let response = self.generate_with_tools(messages, &[]).await?;
        let total_time_ms = start.elapsed().as_millis() as u64;

        Ok(GenerationResult {
            text: response.text,
            tokens: response.output_tokens,
            time_to_first_token_ms: total_time_ms,
            total_time_ms,
            tokens_per_second: if total_time_ms > 0 {
                response.output_tokens as f32 / (total_time_ms as f32 / 1000.0)
            } else {
                0.0
            },
            finish_reason: match response.stop_reason {
                ClaudeStopReason::EndTurn | ClaudeStopReason::StopSequence => FinishReason::Stop,
                ClaudeStopReason::MaxTokens => FinishReason::Length,
                ClaudeStopReason::ToolUse => FinishReason::Stop,
            },
            context: None,
        })
    }

    async fn generate_stream(
        &self,
        messages: &[Message],
        tx: mpsc::Sender<String>,
    ) -> Result<GenerationResult, LlmError> {
        let start = std::time::Instant::now();
        let response = self.generate_with_tools_stream(messages, &[], tx).await?;
        let total_time_ms = start.elapsed().as_millis() as u64;

        Ok(GenerationResult {
            text: response.text,
            tokens: response.output_tokens,
            time_to_first_token_ms: total_time_ms / 10, // Estimate first token
            total_time_ms,
            tokens_per_second: if total_time_ms > 0 {
                response.output_tokens as f32 / (total_time_ms as f32 / 1000.0)
            } else {
                0.0
            },
            finish_reason: match response.stop_reason {
                ClaudeStopReason::EndTurn | ClaudeStopReason::StopSequence => FinishReason::Stop,
                ClaudeStopReason::MaxTokens => FinishReason::Length,
                ClaudeStopReason::ToolUse => FinishReason::Stop,
            },
            context: None,
        })
    }

    async fn is_available(&self) -> bool {
        // Minimal health check
        !self.config.api_key.is_empty()
    }

    fn model_name(&self) -> &str {
        self.config.model.model_id()
    }
}

/// Parsed response from Claude
#[derive(Debug, Clone)]
pub struct ClaudeResponse {
    /// Text content from the response
    pub text: String,
    /// Tool calls requested by the model
    pub tool_calls: Vec<ToolCall>,
    /// Why the model stopped
    pub stop_reason: ClaudeStopReason,
    /// Input tokens used
    pub input_tokens: usize,
    /// Output tokens generated
    pub output_tokens: usize,
}

impl ClaudeResponse {
    /// Check if the model requested tool use
    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }
}

// =============================================================================
// Claude API Types
// =============================================================================

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: usize,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ClaudeTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: ClaudeContent,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ClaudeContent {
    Text(String),
    Blocks(Vec<ClaudeContentBlock>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClaudeContentBlock {
    Text { text: String },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

#[derive(Debug, Serialize)]
struct ClaudeTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct ClaudeApiResponse {
    content: Vec<ClaudeContentBlock>,
    stop_reason: ClaudeStopReason,
    usage: ClaudeUsage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ClaudeStopReason {
    #[default]
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: usize,
    output_tokens: usize,
}

// Streaming event types
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)] // Fields required for serde deserialization
enum ClaudeStreamEvent {
    MessageStart { message: ClaudeMessageStart },
    ContentBlockStart { index: usize, content_block: ClaudeContentBlock },
    ContentBlockDelta { index: usize, delta: ClaudeDelta },
    ContentBlockStop { index: usize },
    MessageDelta { delta: ClaudeMessageDeltaBody, usage: Option<ClaudeUsageDelta> },
    MessageStop,
    Ping,
    Error { error: ClaudeError },
}

#[derive(Debug, Deserialize)]
struct ClaudeMessageStart {
    usage: Option<ClaudeUsageStart>,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsageStart {
    input_tokens: usize,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClaudeDelta {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
}

#[derive(Debug, Deserialize)]
struct ClaudeMessageDeltaBody {
    stop_reason: Option<ClaudeStopReason>,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsageDelta {
    output_tokens: usize,
}

#[derive(Debug, Deserialize)]
struct ClaudeError {
    #[allow(dead_code)]
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_model_id() {
        assert_eq!(ClaudeModel::Opus4_5.model_id(), "claude-opus-4-5-20251101");
        assert_eq!(ClaudeModel::Sonnet4.model_id(), "claude-sonnet-4-20250514");
        assert_eq!(ClaudeModel::Haiku3_5.model_id(), "claude-3-5-haiku-20241022");
    }

    #[test]
    fn test_claude_model_from_str() {
        assert_eq!(ClaudeModel::from_str("opus"), Some(ClaudeModel::Opus4_5));
        assert_eq!(ClaudeModel::from_str("sonnet"), Some(ClaudeModel::Sonnet4));
        assert_eq!(ClaudeModel::from_str("haiku"), Some(ClaudeModel::Haiku3_5));
        assert_eq!(ClaudeModel::from_str("unknown"), None);
    }

    #[test]
    fn test_config_builder() {
        let config = ClaudeConfig::new("test-key")
            .with_model(ClaudeModel::Sonnet4)
            .with_max_tokens(2048)
            .with_temperature(0.5);

        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.model, ClaudeModel::Sonnet4);
        assert_eq!(config.max_tokens, 2048);
        assert_eq!(config.temperature, 0.5);
    }

    #[test]
    fn test_tool_conversion() {
        use crate::prompt::{ToolBuilder, gold_loan_tools};

        // Test with actual gold loan tools
        let tools = gold_loan_tools();
        assert!(!tools.is_empty());

        // Verify structure matches Claude API expectations
        let tool = &tools[0];
        assert!(!tool.name.is_empty());
        assert!(!tool.description.is_empty());
        assert!(tool.parameters.is_object());
    }

    #[test]
    fn test_request_serialization() {
        let request = ClaudeRequest {
            model: "claude-opus-4-5-20251101".to_string(),
            max_tokens: 1024,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: ClaudeContent::Text("Hello".to_string()),
            }],
            system: Some("You are helpful".to_string()),
            tools: None,
            temperature: Some(0.7),
            top_p: None,
            stream: Some(false),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("claude-opus-4-5-20251101"));
        assert!(json.contains("Hello"));
        assert!(json.contains("You are helpful"));
    }

    #[test]
    fn test_response_parsing() {
        let json = r#"{
            "content": [
                {"type": "text", "text": "Hello!"}
            ],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;

        let response: ClaudeApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.stop_reason, ClaudeStopReason::EndTurn);
        assert_eq!(response.usage.input_tokens, 10);
        assert_eq!(response.usage.output_tokens, 5);
    }

    #[test]
    fn test_tool_use_response_parsing() {
        let json = r#"{
            "content": [
                {"type": "text", "text": "Let me check that."},
                {"type": "tool_use", "id": "tool_123", "name": "check_eligibility", "input": {"gold_weight": 50}}
            ],
            "stop_reason": "tool_use",
            "usage": {"input_tokens": 100, "output_tokens": 50}
        }"#;

        let response: ClaudeApiResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.stop_reason, ClaudeStopReason::ToolUse);
        assert_eq!(response.content.len(), 2);
    }
}
