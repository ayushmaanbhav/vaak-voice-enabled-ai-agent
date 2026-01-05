//! Ollama Embeddings
//!
//! Uses Ollama's embedding API for generating dense vectors.
//!
//! ## Qwen3-Embedding Instruction Format
//!
//! For optimal retrieval performance, queries should use the instruction format:
//! `Instruct: <task>\nQuery:<query>`
//!
//! Documents do NOT need instructions - use plain text.
//! See: https://huggingface.co/Qwen/Qwen3-Embedding-0.6B

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::RagError;

/// Default retrieval instruction for Qwen3-Embedding
/// This instruction is used to prefix queries for optimal embedding performance.
pub const DEFAULT_RETRIEVAL_INSTRUCTION: &str =
    "Given a user query about banking products or gold loans, retrieve relevant information that answers the query";

/// Ollama embedding configuration
#[derive(Debug, Clone)]
pub struct OllamaEmbeddingConfig {
    /// Ollama API endpoint
    pub endpoint: String,
    /// Model name
    pub model: String,
    /// Embedding dimension
    pub embedding_dim: usize,
}

impl Default for OllamaEmbeddingConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:11434".to_string(),
            model: "qwen3-embedding:0.6b".to_string(),
            embedding_dim: 1024,
        }
    }
}

/// Request to Ollama embedding API
#[derive(Debug, Serialize)]
struct EmbedRequest {
    model: String,
    input: String,
}

/// Response from Ollama embedding API
#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

/// Ollama embedder
pub struct OllamaEmbedder {
    client: Client,
    config: OllamaEmbeddingConfig,
}

impl OllamaEmbedder {
    /// Create a new Ollama embedder
    pub fn new(config: OllamaEmbeddingConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Create with default config
    pub fn default_qwen3() -> Self {
        Self::new(OllamaEmbeddingConfig::default())
    }

    /// Embed a single text (for documents - no instruction prefix)
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, RagError> {
        self.embed_raw(text).await
    }

    /// Embed a query with instruction prefix for optimal retrieval
    ///
    /// Uses the Qwen3-Embedding instruction format:
    /// `Instruct: <task>\nQuery:<query>`
    pub async fn embed_query(&self, query: &str) -> Result<Vec<f32>, RagError> {
        self.embed_query_with_instruction(query, DEFAULT_RETRIEVAL_INSTRUCTION)
            .await
    }

    /// Embed a query with a custom instruction
    pub async fn embed_query_with_instruction(
        &self,
        query: &str,
        instruction: &str,
    ) -> Result<Vec<f32>, RagError> {
        let formatted = format!("Instruct: {}\nQuery:{}", instruction, query);
        self.embed_raw(&formatted).await
    }

    /// Raw embedding without any formatting
    async fn embed_raw(&self, text: &str) -> Result<Vec<f32>, RagError> {
        let request = EmbedRequest {
            model: self.config.model.clone(),
            input: text.to_string(),
        };

        let url = format!("{}/api/embed", self.config.endpoint);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| RagError::Embedding(format!("Ollama request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(RagError::Embedding(format!(
                "Ollama embedding failed: {} - {}",
                status, text
            )));
        }

        let embed_response: EmbedResponse = response
            .json()
            .await
            .map_err(|e| RagError::Embedding(format!("Failed to parse Ollama response: {}", e)))?;

        embed_response
            .embeddings
            .into_iter()
            .next()
            .ok_or_else(|| RagError::Embedding("No embedding returned".to_string()))
    }

    /// Embed multiple texts
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, RagError> {
        let mut embeddings = Vec::with_capacity(texts.len());
        for text in texts {
            embeddings.push(self.embed(text).await?);
        }
        Ok(embeddings)
    }

    /// Get embedding dimension
    pub fn dim(&self) -> usize {
        self.config.embedding_dim
    }

    /// Get model name
    pub fn model(&self) -> &str {
        &self.config.model
    }
}

/// Thread-safe async embedder wrapper
pub struct AsyncOllamaEmbedder {
    inner: Arc<OllamaEmbedder>,
}

impl AsyncOllamaEmbedder {
    pub fn new(config: OllamaEmbeddingConfig) -> Self {
        Self {
            inner: Arc::new(OllamaEmbedder::new(config)),
        }
    }

    pub fn default_qwen3() -> Self {
        Self {
            inner: Arc::new(OllamaEmbedder::default_qwen3()),
        }
    }

    /// Embed document text (no instruction prefix)
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, RagError> {
        self.inner.embed(text).await
    }

    /// Embed query with instruction prefix for optimal retrieval
    pub async fn embed_query(&self, query: &str) -> Result<Vec<f32>, RagError> {
        self.inner.embed_query(query).await
    }

    /// Embed query with custom instruction
    pub async fn embed_query_with_instruction(
        &self,
        query: &str,
        instruction: &str,
    ) -> Result<Vec<f32>, RagError> {
        self.inner
            .embed_query_with_instruction(query, instruction)
            .await
    }

    pub fn dim(&self) -> usize {
        self.inner.dim()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = OllamaEmbeddingConfig::default();
        assert_eq!(config.model, "qwen3-embedding:0.6b");
        assert_eq!(config.embedding_dim, 1024);
    }
}
