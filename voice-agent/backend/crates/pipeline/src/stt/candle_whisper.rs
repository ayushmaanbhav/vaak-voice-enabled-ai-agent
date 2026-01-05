//! Candle Whisper STT Backend
//!
//! Native Rust Whisper implementation using Candle for English STT.
//! Based on: https://github.com/huggingface/candle/tree/main/candle-examples/examples/whisper

#[cfg(feature = "candle")]
mod candle_impl {
    use crate::PipelineError;
    use candle_core::{Device, Tensor, D};
    use candle_nn::VarBuilder;
    use candle_transformers::models::whisper::{self as m, audio, Config};
    use tokenizers::Tokenizer;
    use voice_agent_core::TranscriptResult;

    const SAMPLE_RATE: usize = 16000;

    /// Whisper model size
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum WhisperModel {
        Tiny,
        Base,
        #[default]
        Small,
        Medium,
        Large,
        LargeV2,
        LargeV3,
    }

    /// Whisper configuration
    #[derive(Debug, Clone)]
    pub struct CandleWhisperConfig {
        /// Path to model directory
        pub model_path: std::path::PathBuf,
        /// Model size
        pub model_size: WhisperModel,
        /// Device (CPU/CUDA)
        pub device: Device,
        /// Language (None for auto-detect)
        pub language: Option<String>,
        /// Task: transcribe or translate
        pub task: String,
    }

    impl Default for CandleWhisperConfig {
        fn default() -> Self {
            Self {
                model_path: std::path::PathBuf::from("models/stt/whisper-small"),
                model_size: WhisperModel::Small,
                device: Device::Cpu,
                language: Some("en".to_string()),
                task: "transcribe".to_string(),
            }
        }
    }

    /// Candle Whisper STT
    pub struct CandleWhisperStt {
        model: m::model::Whisper,
        config: Config,
        tokenizer: Tokenizer,
        user_config: CandleWhisperConfig,
        mel_filters: Vec<f32>,
        audio_buffer: Vec<f32>,
    }

    impl CandleWhisperStt {
        /// Create a new Whisper STT instance
        pub fn new(user_config: CandleWhisperConfig) -> Result<Self, PipelineError> {
            let model_path = &user_config.model_path;

            if !model_path.exists() {
                return Err(PipelineError::Model(format!(
                    "Whisper model not found at {:?}",
                    model_path
                )));
            }

            tracing::info!("Loading Whisper model from {:?}", model_path);

            // Load model config from JSON
            let config_path = model_path.join("config.json");
            let config_str = std::fs::read_to_string(&config_path)
                .map_err(|e| PipelineError::Model(format!("Failed to read config.json: {}", e)))?;
            let config: Config = serde_json::from_str(&config_str)
                .map_err(|e| PipelineError::Model(format!("Failed to parse config.json: {}", e)))?;

            // Load weights
            let weights_path = model_path.join("model.safetensors");
            let device = &user_config.device;

            let vb = unsafe {
                VarBuilder::from_mmaped_safetensors(&[weights_path], m::DTYPE, device)
                    .map_err(|e| PipelineError::Model(format!("Failed to load weights: {}", e)))?
            };

            // Build model
            let model = m::model::Whisper::load(&vb, config.clone())
                .map_err(|e| PipelineError::Model(format!("Failed to build Whisper model: {}", e)))?;

            // Load tokenizer
            let tokenizer_path = model_path.join("tokenizer.json");
            let tokenizer = Tokenizer::from_file(&tokenizer_path)
                .map_err(|e| PipelineError::Model(format!("Failed to load tokenizer: {}", e)))?;

            // Load mel filters
            let mel_bytes = if config.num_mel_bins == 128 {
                include_bytes!("mel_filters_128.bytes").as_slice()
            } else {
                include_bytes!("mel_filters.bytes").as_slice()
            };
            let mut mel_filters = vec![0f32; mel_bytes.len() / 4];
            <byteorder::LittleEndian as byteorder::ByteOrder>::read_f32_into(mel_bytes, &mut mel_filters);

            tracing::info!(
                "Whisper model loaded successfully (mel_bins={})",
                config.num_mel_bins
            );

            Ok(Self {
                model,
                config,
                tokenizer,
                user_config,
                mel_filters,
                audio_buffer: Vec::new(),
            })
        }

        /// Transcribe audio
        pub fn transcribe(&mut self, audio: &[f32]) -> Result<TranscriptResult, PipelineError> {
            if audio.is_empty() {
                return Ok(TranscriptResult {
                    text: String::new(),
                    is_final: true,
                    confidence: 0.0,
                    start_time_ms: 0,
                    end_time_ms: 0,
                    language: self.user_config.language.clone(),
                    words: vec![],
                });
            }

            let device = &self.user_config.device;

            // Convert to mel spectrogram using candle's audio module
            let mel = audio::pcm_to_mel(&self.config, audio, &self.mel_filters);
            let mel_len = mel.len();
            let n_mels = self.config.num_mel_bins;

            let mel = Tensor::from_slice(&mel, (1, n_mels, mel_len / n_mels), device)
                .map_err(|e| PipelineError::Model(format!("Mel tensor failed: {}", e)))?;

            // Encode
            let encoder_output = self
                .model
                .encoder
                .forward(&mel, true)
                .map_err(|e| PipelineError::Model(format!("Encoder failed: {}", e)))?;

            // Decode with greedy search
            let sot_token = 50258u32; // <|startoftranscript|>
            let transcribe_token = 50359u32; // <|transcribe|>
            let notimestamps_token = 50363u32; // <|notimestamps|>
            let eot_token = 50257u32; // <|endoftext|>

            let lang_token = if let Some(ref lang) = self.user_config.language {
                match lang.as_str() {
                    "en" => 50259u32,
                    "hi" => 50276u32,
                    _ => 50259u32, // Default to English
                }
            } else {
                50259u32
            };

            let mut tokens: Vec<u32> = vec![sot_token, lang_token, transcribe_token, notimestamps_token];
            let max_tokens = 224;

            for _ in 0..max_tokens {
                let tokens_tensor = Tensor::new(tokens.as_slice(), device)
                    .map_err(|e| PipelineError::Model(format!("Token tensor failed: {}", e)))?
                    .unsqueeze(0)
                    .map_err(|e| PipelineError::Model(format!("Unsqueeze failed: {}", e)))?;

                let logits = self
                    .model
                    .decoder
                    .forward(&tokens_tensor, &encoder_output, true)
                    .map_err(|e| PipelineError::Model(format!("Decoder failed: {}", e)))?;

                let (_, seq_len, _) = logits.dims3()
                    .map_err(|e| PipelineError::Model(format!("Dims failed: {}", e)))?;

                let last_logits = logits
                    .get_on_dim(1, seq_len - 1)
                    .map_err(|e| PipelineError::Model(format!("Get on dim failed: {}", e)))?
                    .squeeze(0)
                    .map_err(|e| PipelineError::Model(format!("Squeeze failed: {}", e)))?;

                let next_token = last_logits
                    .argmax(D::Minus1)
                    .map_err(|e| PipelineError::Model(format!("Argmax failed: {}", e)))?
                    .to_scalar::<u32>()
                    .map_err(|e| PipelineError::Model(format!("Scalar failed: {}", e)))?;

                if next_token == eot_token || next_token >= 50257 {
                    break;
                }

                tokens.push(next_token);
            }

            // Decode tokens to text (skip special tokens at start)
            let text_tokens: Vec<u32> = tokens.into_iter().skip(4).collect();
            let text = self
                .tokenizer
                .decode(&text_tokens, true)
                .map_err(|e| PipelineError::Model(format!("Decode failed: {}", e)))?;

            let duration_ms = (audio.len() as u64 * 1000) / SAMPLE_RATE as u64;

            Ok(TranscriptResult {
                text: text.trim().to_string(),
                is_final: true,
                confidence: 0.9,
                start_time_ms: 0,
                end_time_ms: duration_ms,
                language: self.user_config.language.clone(),
                words: vec![],
            })
        }

        /// Process audio chunk (accumulates in buffer)
        pub fn process(&mut self, audio: &[f32]) -> Result<Option<TranscriptResult>, PipelineError> {
            self.audio_buffer.extend_from_slice(audio);

            // Only process if we have at least 1 second
            if self.audio_buffer.len() < SAMPLE_RATE {
                return Ok(None);
            }

            // Return partial every 2 seconds
            if self.audio_buffer.len() >= SAMPLE_RATE * 2 {
                // Take buffer to avoid borrow conflict (transcribe needs &mut self)
                let buffer = std::mem::take(&mut self.audio_buffer);
                let result = self.transcribe(&buffer)?;
                return Ok(Some(result));
            }

            Ok(None)
        }

        /// Finalize and get final transcript
        pub fn finalize(&mut self) -> TranscriptResult {
            if self.audio_buffer.is_empty() {
                return TranscriptResult {
                    text: String::new(),
                    is_final: true,
                    confidence: 0.0,
                    start_time_ms: 0,
                    end_time_ms: 0,
                    language: self.user_config.language.clone(),
                    words: vec![],
                };
            }

            let audio = std::mem::take(&mut self.audio_buffer);
            match self.transcribe(&audio) {
                Ok(mut result) => {
                    result.is_final = true;
                    result
                }
                Err(e) => {
                    tracing::error!("Whisper finalize failed: {}", e);
                    TranscriptResult {
                        text: String::new(),
                        is_final: true,
                        confidence: 0.0,
                        start_time_ms: 0,
                        end_time_ms: 0,
                        language: self.user_config.language.clone(),
                        words: vec![],
                    }
                }
            }
        }

        /// Reset state
        pub fn reset(&mut self) {
            self.audio_buffer.clear();
        }
    }
}

#[cfg(feature = "candle")]
pub use candle_impl::*;

// Stub when candle feature is disabled
#[cfg(not(feature = "candle"))]
#[derive(Debug, Clone, Default)]
pub struct CandleWhisperConfig {
    pub model_path: std::path::PathBuf,
}

#[cfg(not(feature = "candle"))]
pub struct CandleWhisperStt;

#[cfg(not(feature = "candle"))]
impl CandleWhisperStt {
    pub fn new(_config: CandleWhisperConfig) -> Result<Self, crate::PipelineError> {
        Err(crate::PipelineError::Model(
            "Candle feature not enabled".to_string(),
        ))
    }
}

#[cfg(not(feature = "candle"))]
#[derive(Debug, Clone, Copy, Default)]
pub enum WhisperModel {
    #[default]
    Small,
}
