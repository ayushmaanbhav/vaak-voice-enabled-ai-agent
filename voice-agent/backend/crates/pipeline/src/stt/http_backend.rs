//! HTTP STT Backend - Calls external Python STT service
//!
//! This backend sends audio to a Python sidecar service that handles
//! IndicConformer inference with proper language mask decoding.
//!
//! ## Why Python?
//! The native Rust IndicConformer implementation had issues with:
//! - Mel spectrogram preprocessing mismatches
//! - Joint vocabulary vs per-language decoding
//! - Language mask filtering
//!
//! The Python service uses the reference ai4bharat implementation which
//! is known to work correctly.

use crate::PipelineError;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use voice_agent_core::TranscriptResult;

/// HTTP STT Backend configuration
#[derive(Debug, Clone)]
pub struct HttpSttConfig {
    /// Base URL of the Python STT service
    pub url: String,
    /// Language code (e.g., "hi" for Hindi)
    pub language: String,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Minimum audio length to process (samples at 16kHz)
    pub min_audio_samples: usize,
    /// Audio buffer size before sending (samples at 16kHz)
    pub buffer_size: usize,
}

impl Default for HttpSttConfig {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:8090".to_string(),
            language: "hi".to_string(),
            timeout_ms: 30000,
            min_audio_samples: 1600, // 100ms minimum
            buffer_size: 16000,      // 1 second buffer
        }
    }
}

/// Response from the Python STT service
#[derive(Debug, Deserialize)]
struct SttResponse {
    text: String,
    confidence: f32,
    language: String,
    backend: String,
    #[serde(default)]
    error: Option<String>,
}

/// HTTP STT Backend
///
/// Buffers audio and sends to Python service for transcription.
/// This is a non-streaming backend - audio is accumulated and sent
/// when finalize() is called or buffer is full.
pub struct HttpSttBackend {
    config: HttpSttConfig,
    client: reqwest::blocking::Client,
    audio_buffer: Vec<f32>,
    current_partial: Option<TranscriptResult>,
    start_time_ms: u64,
    utterance_start: Option<Instant>,
}

impl HttpSttBackend {
    /// Create a new HTTP STT backend
    pub fn new(config: HttpSttConfig) -> Result<Self, PipelineError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|e| PipelineError::Model(format!("Failed to create HTTP client: {}", e)))?;

        // Check if service is available
        let health_url = format!("{}/health", config.url);
        match client.get(&health_url).send() {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!(
                    "HTTP STT backend connected to {} (language: {})",
                    config.url,
                    config.language
                );
            }
            Ok(resp) => {
                tracing::warn!(
                    "HTTP STT service returned status {} - proceeding anyway",
                    resp.status()
                );
            }
            Err(e) => {
                tracing::warn!("HTTP STT service not reachable: {} - will retry on first request", e);
            }
        }

        Ok(Self {
            config,
            client,
            audio_buffer: Vec::with_capacity(48000), // 3 seconds
            current_partial: None,
            start_time_ms: 0,
            utterance_start: None,
        })
    }

    /// Create with default config
    pub fn new_default() -> Result<Self, PipelineError> {
        Self::new(HttpSttConfig::default())
    }

    /// Create with custom URL
    pub fn new_with_url(url: impl Into<String>, language: impl Into<String>) -> Result<Self, PipelineError> {
        Self::new(HttpSttConfig {
            url: url.into(),
            language: language.into(),
            ..Default::default()
        })
    }

    /// Set start time for timestamps
    pub fn set_start_time(&mut self, time_ms: u64) {
        self.start_time_ms = time_ms;
    }

    /// Send audio to Python service and get transcription
    fn transcribe_audio(&self, audio: &[f32]) -> Result<SttResponse, PipelineError> {
        if audio.len() < self.config.min_audio_samples {
            return Ok(SttResponse {
                text: String::new(),
                confidence: 0.0,
                language: self.config.language.clone(),
                backend: "http".to_string(),
                error: Some("Audio too short".to_string()),
            });
        }

        // Convert float32 audio to PCM16 bytes
        let pcm16: Vec<i16> = audio.iter().map(|&s| (s * 32767.0) as i16).collect();
        let pcm_bytes: Vec<u8> = pcm16
            .iter()
            .flat_map(|&s| s.to_le_bytes())
            .collect();

        let url = format!("{}/transcribe", self.config.url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "audio/pcm")
            .header("X-Language", &self.config.language)
            .body(pcm_bytes)
            .send()
            .map_err(|e| PipelineError::Model(format!("HTTP STT request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(PipelineError::Model(format!(
                "HTTP STT service returned error: {}",
                response.status()
            )));
        }

        let result: SttResponse = response
            .json()
            .map_err(|e| PipelineError::Model(format!("Failed to parse STT response: {}", e)))?;

        if let Some(error) = &result.error {
            tracing::warn!("STT service returned error: {}", error);
        }

        Ok(result)
    }
}

impl super::SttBackend for HttpSttBackend {
    fn process(&mut self, audio: &[f32]) -> Result<Option<TranscriptResult>, PipelineError> {
        if self.utterance_start.is_none() {
            self.utterance_start = Some(Instant::now());
        }

        // Add audio to buffer
        self.audio_buffer.extend_from_slice(audio);

        // If buffer is large enough, send for transcription
        if self.audio_buffer.len() >= self.config.buffer_size {
            let response = self.transcribe_audio(&self.audio_buffer)?;

            if !response.text.is_empty() {
                let elapsed = self.utterance_start.map(|s| s.elapsed().as_millis() as u64).unwrap_or(0);

                let partial = TranscriptResult {
                    text: response.text,
                    is_final: false,
                    confidence: response.confidence,
                    start_time_ms: self.start_time_ms,
                    end_time_ms: self.start_time_ms + elapsed,
                    language: Some(response.language),
                    words: vec![],
                };

                self.current_partial = Some(partial.clone());
                return Ok(Some(partial));
            }
        }

        Ok(None)
    }

    fn finalize_sync(&mut self) -> TranscriptResult {
        let elapsed = self.utterance_start.map(|s| s.elapsed().as_millis() as u64).unwrap_or(0);

        if self.audio_buffer.is_empty() {
            return TranscriptResult {
                text: String::new(),
                is_final: true,
                confidence: 0.0,
                start_time_ms: self.start_time_ms,
                end_time_ms: self.start_time_ms + elapsed,
                language: Some(self.config.language.clone()),
                words: vec![],
            };
        }

        // Transcribe remaining audio
        match self.transcribe_audio(&self.audio_buffer) {
            Ok(response) => {
                let result = TranscriptResult {
                    text: response.text,
                    is_final: true,
                    confidence: response.confidence,
                    start_time_ms: self.start_time_ms,
                    end_time_ms: self.start_time_ms + elapsed,
                    language: Some(response.language),
                    words: vec![],
                };

                // Clear buffer
                self.audio_buffer.clear();
                self.utterance_start = None;

                result
            }
            Err(e) => {
                tracing::error!("Failed to finalize STT: {}", e);

                self.audio_buffer.clear();
                self.utterance_start = None;

                TranscriptResult {
                    text: String::new(),
                    is_final: true,
                    confidence: 0.0,
                    start_time_ms: self.start_time_ms,
                    end_time_ms: self.start_time_ms + elapsed,
                    language: Some(self.config.language.clone()),
                    words: vec![],
                }
            }
        }
    }

    fn reset(&mut self) {
        self.audio_buffer.clear();
        self.current_partial = None;
        self.utterance_start = None;
    }

    fn partial(&self) -> Option<&TranscriptResult> {
        self.current_partial.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = HttpSttConfig::default();
        assert_eq!(config.url, "http://127.0.0.1:8090");
        assert_eq!(config.language, "hi");
        assert_eq!(config.timeout_ms, 30000);
    }
}
