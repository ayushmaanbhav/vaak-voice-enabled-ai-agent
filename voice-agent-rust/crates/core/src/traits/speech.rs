//! Speech processing traits

use async_trait::async_trait;
use std::pin::Pin;
use futures::Stream;
use crate::{AudioFrame, Language, VoiceConfig, VoiceInfo, Result};
use crate::transcript::TranscriptResult;

/// Alias for transcript frame (matches architecture doc naming)
pub type TranscriptFrame = TranscriptResult;

/// Speech-to-Text interface
///
/// Implementations:
/// - `IndicConformerStt` - AI4Bharat's multilingual STT (22 languages)
/// - `WhisperStt` - OpenAI Whisper (fallback)
///
/// # Example
///
/// ```ignore
/// let stt: Box<dyn SpeechToText> = Box::new(IndicConformerStt::new(config));
/// let transcript = stt.transcribe(&audio_frame).await?;
/// println!("Transcribed: {}", transcript.text);
/// ```
#[async_trait]
pub trait SpeechToText: Send + Sync + 'static {
    /// Transcribe a single audio frame
    ///
    /// # Arguments
    /// * `audio` - Audio frame to transcribe
    ///
    /// # Returns
    /// Transcript with text, confidence, and optional word timestamps
    async fn transcribe(&self, audio: &AudioFrame) -> Result<TranscriptFrame>;

    /// Stream transcription as audio arrives
    ///
    /// Returns partial transcripts followed by final transcript.
    /// Partial transcripts have `is_final = false`.
    ///
    /// # Arguments
    /// * `audio_stream` - Stream of audio frames
    ///
    /// # Returns
    /// Stream of transcript results (partial and final)
    fn transcribe_stream<'a>(
        &'a self,
        audio_stream: Pin<Box<dyn Stream<Item = AudioFrame> + Send + 'a>>,
    ) -> Pin<Box<dyn Stream<Item = Result<TranscriptFrame>> + Send + 'a>>;

    /// Get supported languages
    fn supported_languages(&self) -> &[Language];

    /// Get model name for logging
    fn model_name(&self) -> &str;

    /// Check if a specific language is supported
    fn supports_language(&self, lang: Language) -> bool {
        self.supported_languages().contains(&lang)
    }
}

/// Text-to-Speech interface
///
/// Implementations:
/// - `IndicF5Tts` - AI4Bharat's multilingual TTS (11 languages)
/// - `PiperTts` - Fast fallback TTS
///
/// # Example
///
/// ```ignore
/// let tts: Box<dyn TextToSpeech> = Box::new(IndicF5Tts::new(config));
/// let config = VoiceConfig::new(Language::Hindi);
/// let audio = tts.synthesize("नमस्ते", &config).await?;
/// ```
#[async_trait]
pub trait TextToSpeech: Send + Sync + 'static {
    /// Synthesize text to audio
    ///
    /// # Arguments
    /// * `text` - Text to synthesize
    /// * `config` - Voice configuration (language, speed, pitch)
    ///
    /// # Returns
    /// Audio frame with synthesized speech
    async fn synthesize(&self, text: &str, config: &VoiceConfig) -> Result<AudioFrame>;

    /// Stream synthesis sentence-by-sentence
    ///
    /// Enables low-latency response by starting audio before text is complete.
    /// Each yielded audio frame corresponds to one sentence.
    ///
    /// # Arguments
    /// * `text_stream` - Stream of text chunks (sentences)
    /// * `config` - Voice configuration
    ///
    /// # Returns
    /// Stream of audio frames
    fn synthesize_stream<'a>(
        &'a self,
        text_stream: Pin<Box<dyn Stream<Item = String> + Send + 'a>>,
        config: &'a VoiceConfig,
    ) -> Pin<Box<dyn Stream<Item = Result<AudioFrame>> + Send + 'a>>;

    /// Get available voices
    fn available_voices(&self) -> &[VoiceInfo];

    /// Get model name for logging
    fn model_name(&self) -> &str;

    /// Get default voice for a language
    fn default_voice(&self, lang: Language) -> Option<&VoiceInfo> {
        self.available_voices()
            .iter()
            .find(|v| v.language == lang)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct MockStt {
        languages: Vec<Language>,
    }

    #[async_trait]
    impl SpeechToText for MockStt {
        async fn transcribe(&self, _audio: &AudioFrame) -> Result<TranscriptFrame> {
            Ok(TranscriptResult {
                text: "Test transcription".to_string(),
                confidence: 0.95,
                is_final: true,
                ..Default::default()
            })
        }

        fn transcribe_stream<'a>(
            &'a self,
            _audio_stream: Pin<Box<dyn Stream<Item = AudioFrame> + Send + 'a>>,
        ) -> Pin<Box<dyn Stream<Item = Result<TranscriptFrame>> + Send + 'a>> {
            Box::pin(futures::stream::empty())
        }

        fn supported_languages(&self) -> &[Language] {
            &self.languages
        }

        fn model_name(&self) -> &str {
            "mock-stt"
        }
    }

    #[test]
    fn test_supports_language() {
        let stt = MockStt {
            languages: vec![Language::Hindi, Language::English],
        };
        assert!(stt.supports_language(Language::Hindi));
        assert!(!stt.supports_language(Language::Tamil));
    }
}
