//! IndicConformer Configuration
//!
//! Configuration structs for the IndicConformer STT model.

use super::super::decoder::DecoderConfig;
use voice_agent_core::SampleRate;

/// IndicConformer configuration
#[derive(Debug, Clone)]
pub struct IndicConformerConfig {
    /// Language code (hi, mr, bn, etc.)
    pub language: String,
    /// Sample rate (must be 16000)
    pub sample_rate: SampleRate,
    /// Number of mel frequency bins
    pub n_mels: usize,
    /// FFT window size in samples
    pub n_fft: usize,
    /// Hop length in samples
    pub hop_length: usize,
    /// Window length in samples
    pub win_length: usize,
    /// Chunk size in milliseconds for streaming
    pub chunk_ms: u32,
    /// Enable partial results
    pub enable_partials: bool,
    /// Partial emission interval (frames)
    pub partial_interval: usize,
    /// Decoder configuration
    pub decoder: DecoderConfig,
}

impl Default for IndicConformerConfig {
    fn default() -> Self {
        Self {
            language: "hi".to_string(),
            sample_rate: SampleRate::Hz16000,
            n_mels: 80,
            n_fft: 512,
            hop_length: 160, // 10ms at 16kHz
            win_length: 400, // 25ms at 16kHz
            // Chunk size: 500ms gives ~5 decoder frames after Conformer downsampling
            // (100ms chunks only produce 1 frame, insufficient for CTC decoding)
            chunk_ms: 500,
            enable_partials: true,
            partial_interval: 1, // Emit partials every chunk for responsive turn detection
            decoder: DecoderConfig::default(),
        }
    }
}

impl IndicConformerConfig {
    /// Create config for Hindi
    pub fn hindi() -> Self {
        Self::default()
    }

    /// Create config for Marathi
    pub fn marathi() -> Self {
        Self {
            language: "mr".to_string(),
            ..Self::default()
        }
    }

    /// Create config for Bengali
    pub fn bengali() -> Self {
        Self {
            language: "bn".to_string(),
            ..Self::default()
        }
    }

    /// Create config for Tamil
    pub fn tamil() -> Self {
        Self {
            language: "ta".to_string(),
            ..Self::default()
        }
    }

    /// Create config for Telugu
    pub fn telugu() -> Self {
        Self {
            language: "te".to_string(),
            ..Self::default()
        }
    }

    /// Set language
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    /// Set chunk size for streaming
    pub fn with_chunk_ms(mut self, chunk_ms: u32) -> Self {
        self.chunk_ms = chunk_ms;
        self
    }

    /// Enable/disable partial results
    pub fn with_partials(mut self, enable: bool) -> Self {
        self.enable_partials = enable;
        self
    }

    /// Set decoder config
    pub fn with_decoder(mut self, decoder: DecoderConfig) -> Self {
        self.decoder = decoder;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = IndicConformerConfig::default();
        assert_eq!(config.language, "hi");
        assert_eq!(config.n_mels, 80);
        assert_eq!(config.sample_rate, SampleRate::Hz16000);
    }

    #[test]
    fn test_language_configs() {
        assert_eq!(IndicConformerConfig::hindi().language, "hi");
        assert_eq!(IndicConformerConfig::marathi().language, "mr");
        assert_eq!(IndicConformerConfig::bengali().language, "bn");
        assert_eq!(IndicConformerConfig::tamil().language, "ta");
        assert_eq!(IndicConformerConfig::telugu().language, "te");
    }

    #[test]
    fn test_builder_pattern() {
        let config = IndicConformerConfig::default()
            .with_language("mr")
            .with_chunk_ms(1000)
            .with_partials(false);

        assert_eq!(config.language, "mr");
        assert_eq!(config.chunk_ms, 1000);
        assert!(!config.enable_partials);
    }
}
