//! IndicConformer STT Core Implementation
//!
//! Contains the main IndicConformerStt struct and its implementation.

use ndarray::Array2;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::Path;

#[cfg(feature = "onnx")]
use ndarray::Array3;

#[cfg(feature = "onnx")]
use ort::{session::builder::GraphOptimizationLevel, session::Session, value::Tensor};

// Candle ONNX imports (pure Rust ONNX support)
#[cfg(feature = "candle-onnx")]
use candle_core::{Device, Tensor as CandleTensor};
#[cfg(feature = "candle-onnx")]
use candle_onnx::onnx::ModelProto;

// Import from parent stt module
use super::super::decoder::{DecoderConfig, EnhancedDecoder};
use super::super::vocab::Vocabulary;
use super::super::SttBackend;
use crate::PipelineError;
use voice_agent_core::{TranscriptResult, WordTimestamp};

// Import from sibling modules
use super::config::IndicConformerConfig;
use super::mel::MelFilterbank;

/// Mutable state for IndicConformer
struct IndicConformerState {
    /// Audio buffer for accumulating samples
    audio_buffer: Vec<f32>,
    /// Frame counter for partial emission
    frame_count: usize,
    /// Words detected with timestamps
    words: Vec<WordTimestamp>,
    /// Start timestamp
    start_time_ms: u64,
    /// Previous encoder hidden state for streaming (if using RNN-T)
    encoder_state: Option<Array2<f32>>,
    /// P0 FIX: Running sum of frame confidences for averaging
    confidence_sum: f32,
    /// P0 FIX: Number of frames processed (for averaging)
    confidence_count: usize,
    /// P0 FIX: Per-word confidence accumulator
    current_word_confidence: f32,
    /// P0 FIX: Frame count for current word
    current_word_frames: usize,
    /// P0 FIX: Total audio frames processed (for timestamp calculation)
    total_audio_frames: usize,
}

/// IndicConformer STT implementation
pub struct IndicConformerStt {
    // ORT backend (external ONNX Runtime)
    #[cfg(feature = "onnx")]
    encoder_session: Mutex<Session>,
    #[cfg(feature = "onnx")]
    decoder_session: Mutex<Session>,
    #[cfg(feature = "onnx")]
    post_net_session: Option<Mutex<Session>>,

    // Candle ONNX backend (pure Rust)
    #[cfg(feature = "candle-onnx")]
    encoder_model: ModelProto,
    #[cfg(feature = "candle-onnx")]
    decoder_model: ModelProto,
    #[cfg(feature = "candle-onnx")]
    post_net_model: Option<ModelProto>,
    #[cfg(feature = "candle-onnx")]
    device: Device,

    config: IndicConformerConfig,
    vocabulary: Vocabulary,
    decoder: EnhancedDecoder,
    mel_filterbank: MelFilterbank,
    /// Language mask to constrain decoder output to target language tokens
    language_mask: Vec<bool>,
    state: Mutex<IndicConformerState>,
}

impl IndicConformerStt {
    /// Create a new IndicConformer STT from model directory
    ///
    /// Expected directory structure:
    /// - assets/encoder.onnx
    /// - assets/ctc_decoder.onnx
    /// - assets/joint_post_net_{lang}.onnx
    /// - assets/vocab.json
    #[cfg(feature = "onnx")]
    pub fn new(
        model_dir: impl AsRef<Path>,
        config: IndicConformerConfig,
    ) -> Result<Self, PipelineError> {
        let model_dir = model_dir.as_ref();
        let assets_dir = model_dir.join("assets");

        // Load encoder
        let encoder_path = assets_dir.join("encoder.onnx");
        let encoder_session = Self::load_session(&encoder_path)?;

        // Load CTC decoder
        let decoder_path = assets_dir.join("ctc_decoder.onnx");
        let decoder_session = Self::load_session(&decoder_path)?;

        // Load language-specific post-net (optional)
        let post_net_path = assets_dir.join(format!("joint_post_net_{}.onnx", config.language));
        let post_net_session = if post_net_path.exists() {
            Some(Self::load_session(&post_net_path)?)
        } else {
            None
        };

        // Load vocabulary (per-language, 257 tokens)
        let vocab_path = assets_dir.join("vocab.json");
        let vocabulary = Self::load_vocab(&vocab_path, &config.language)?;

        // Load language mask - CRITICAL for correct decoding
        // CTC decoder outputs 5633 joint tokens, mask filters to 257 per-language tokens
        let language_mask = Self::load_language_mask(&assets_dir, &config.language)?;
        tracing::info!(
            language = %config.language,
            mask_size = language_mask.len(),
            enabled_tokens = language_mask.iter().filter(|&&x| x).count(),
            "IndicConformer: Loaded language mask for joint vocab filtering"
        );

        // Create decoder with vocabulary and correct blank_id
        // After masking, we have 257 tokens: indices 0-255 regular + 256 blank
        let mut decoder_config = config.decoder.clone();
        decoder_config.blank_id = 256; // Blank token at index 256 in filtered output
        let decoder = EnhancedDecoder::new(vocabulary.clone().into_tokens(), decoder_config);

        // Create mel filterbank
        let mel_filterbank = MelFilterbank::new(
            config.sample_rate.as_u32() as usize,
            config.n_fft,
            config.n_mels,
        );

        Ok(Self {
            encoder_session: Mutex::new(encoder_session),
            decoder_session: Mutex::new(decoder_session),
            post_net_session: post_net_session.map(Mutex::new),
            config,
            vocabulary,
            decoder,
            mel_filterbank,
            language_mask,
            state: Mutex::new(IndicConformerState {
                audio_buffer: Vec::new(),
                frame_count: 0,
                words: Vec::new(),
                start_time_ms: 0,
                encoder_state: None,
                confidence_sum: 0.0,
                confidence_count: 0,
                current_word_confidence: 0.0,
                current_word_frames: 0,
                total_audio_frames: 0,
            }),
        })
    }

    /// Create IndicConformer using candle-onnx (pure Rust ONNX)
    ///
    /// This uses Huggingface Candle's ONNX support which doesn't require
    /// external ONNX Runtime libraries.
    #[cfg(feature = "candle-onnx")]
    pub fn new(
        model_dir: impl AsRef<Path>,
        config: IndicConformerConfig,
    ) -> Result<Self, PipelineError> {
        let model_dir = model_dir.as_ref();
        let assets_dir = model_dir.join("assets");

        tracing::info!(
            model_dir = %model_dir.display(),
            assets_dir = %assets_dir.display(),
            "IndicConformer: Starting model loading with candle-onnx"
        );

        // Load ONNX models using candle-onnx
        // Use encoder_inline.onnx which has all weights embedded (not external data)
        let encoder_path = assets_dir.join("encoder_inline.onnx");
        tracing::debug!(path = %encoder_path.display(), "IndicConformer: Loading encoder...");
        let encoder_model = candle_onnx::read_file(&encoder_path)
            .map_err(|e| {
                tracing::error!(error = %e, path = %encoder_path.display(), "IndicConformer: Failed to load encoder");
                PipelineError::Model(format!("Failed to load encoder: {}", e))
            })?;
        tracing::info!("IndicConformer: Encoder loaded successfully");

        let decoder_path = assets_dir.join("ctc_decoder.onnx");
        tracing::debug!(path = %decoder_path.display(), "IndicConformer: Loading CTC decoder...");
        let decoder_model = candle_onnx::read_file(&decoder_path)
            .map_err(|e| {
                tracing::error!(error = %e, path = %decoder_path.display(), "IndicConformer: Failed to load CTC decoder");
                PipelineError::Model(format!("Failed to load CTC decoder: {}", e))
            })?;
        tracing::info!("IndicConformer: CTC decoder loaded successfully");

        // Load language-specific post-net (optional)
        let post_net_path = assets_dir.join(format!("joint_post_net_{}.onnx", config.language));
        let post_net_model = if post_net_path.exists() {
            Some(candle_onnx::read_file(&post_net_path)
                .map_err(|e| PipelineError::Model(format!("Failed to load post-net: {}", e)))?)
        } else {
            None
        };

        // Load vocabulary (per-language, 257 tokens)
        let vocab_path = assets_dir.join("vocab.json");
        let vocabulary = Self::load_vocab(&vocab_path, &config.language)?;

        // Load language mask - CRITICAL for correct decoding
        // CTC decoder outputs 5633 joint tokens, mask filters to 257 per-language tokens
        let language_mask = Self::load_language_mask(&assets_dir, &config.language)?;
        tracing::info!(
            language = %config.language,
            mask_size = language_mask.len(),
            enabled_tokens = language_mask.iter().filter(|&&x| x).count(),
            "IndicConformer: Loaded language mask for joint vocab filtering (candle)"
        );

        // Create decoder with vocabulary and correct blank_id
        // After masking, we have 257 tokens: indices 0-255 regular + 256 blank
        let mut decoder_config = config.decoder.clone();
        decoder_config.blank_id = 256; // Blank token at index 256 in filtered output
        let decoder = EnhancedDecoder::new(vocabulary.clone().into_tokens(), decoder_config);

        // Create mel filterbank
        let mel_filterbank = MelFilterbank::new(
            config.sample_rate.as_u32() as usize,
            config.n_fft,
            config.n_mels,
        );

        // Use CPU device (can be extended to support CUDA/Metal)
        let device = Device::Cpu;

        tracing::info!(
            language = %config.language,
            encoder = %encoder_path.display(),
            "Loaded IndicConformer models using candle-onnx (pure Rust)"
        );

        Ok(Self {
            encoder_model,
            decoder_model,
            post_net_model,
            device,
            config,
            vocabulary,
            decoder,
            mel_filterbank,
            language_mask,
            state: Mutex::new(IndicConformerState {
                audio_buffer: Vec::new(),
                frame_count: 0,
                words: Vec::new(),
                start_time_ms: 0,
                encoder_state: None,
                confidence_sum: 0.0,
                confidence_count: 0,
                current_word_confidence: 0.0,
                current_word_frames: 0,
                total_audio_frames: 0,
            }),
        })
    }

    /// Create IndicConformer without ONNX (stub for testing)
    #[cfg(not(any(feature = "onnx", feature = "candle-onnx")))]
    pub fn new(
        _model_dir: impl AsRef<Path>,
        config: IndicConformerConfig,
    ) -> Result<Self, PipelineError> {
        Self::simple(config)
    }

    /// Create a simple stub for testing
    #[cfg(not(any(feature = "onnx", feature = "candle-onnx")))]
    pub fn simple(config: IndicConformerConfig) -> Result<Self, PipelineError> {
        let vocabulary = Vocabulary::default_indicconformer();
        let decoder = EnhancedDecoder::simple(config.decoder.clone());
        let mel_filterbank = MelFilterbank::new(
            config.sample_rate.as_u32() as usize,
            config.n_fft,
            config.n_mels,
        );

        Ok(Self {
            config,
            vocabulary,
            decoder,
            mel_filterbank,
            language_mask: Vec::new(), // No mask for simple/stub mode
            state: Mutex::new(IndicConformerState {
                audio_buffer: Vec::new(),
                frame_count: 0,
                words: Vec::new(),
                start_time_ms: 0,
                encoder_state: None,
                confidence_sum: 0.0,
                confidence_count: 0,
                current_word_confidence: 0.0,
                current_word_frames: 0,
                total_audio_frames: 0,
            }),
        })
    }

    #[cfg(feature = "onnx")]
    fn load_session(path: &Path) -> Result<Session, PipelineError> {
        Session::builder()
            .map_err(|e| PipelineError::Model(e.to_string()))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| PipelineError::Model(e.to_string()))?
            .with_intra_threads(2)
            .map_err(|e| PipelineError::Model(e.to_string()))?
            .commit_from_file(path)
            .map_err(|e| PipelineError::Model(format!("Failed to load {}: {}", path.display(), e)))
    }

    fn load_vocab(path: &Path, language: &str) -> Result<Vocabulary, PipelineError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PipelineError::Io(format!("Failed to read vocab: {}", e)))?;

        let vocab_map: HashMap<String, Vec<String>> = serde_json::from_str(&content)
            .map_err(|e| PipelineError::Stt(format!("Failed to parse vocab: {}", e)))?;

        // IndicConformer CTC decoder outputs per-language vocabulary (257 tokens)
        // Load only the target language's tokens (indices 0-255) + blank "|" (index 256)
        let lang_tokens = vocab_map.get(language)
            .ok_or_else(|| PipelineError::Stt(format!("Language '{}' not found in vocab", language)))?;

        // The vocab file has the "|" blank token at the end of each language's list
        // We need exactly 257 tokens: indices 0-255 for regular tokens, 256 for blank
        let mut tokens: Vec<String> = lang_tokens.iter().take(256).cloned().collect();
        tokens.push("|".to_string()); // Blank token at index 256

        tracing::info!(
            language = %language,
            vocab_size = tokens.len(),
            expected_size = 257,
            "IndicConformer: Loaded per-language vocabulary"
        );

        Ok(Vocabulary::from_tokens(tokens))
    }

    /// Load language mask from language_masks.json
    ///
    /// The language mask constrains decoder output to only tokens valid for the target language.
    fn load_language_mask(assets_dir: &Path, language: &str) -> Result<Vec<bool>, PipelineError> {
        let mask_path = assets_dir.join("language_masks.json");

        if !mask_path.exists() {
            tracing::warn!("language_masks.json not found, decoder will use all tokens");
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&mask_path)
            .map_err(|e| PipelineError::Io(format!("Failed to read language masks: {}", e)))?;

        let masks: HashMap<String, Vec<bool>> = serde_json::from_str(&content)
            .map_err(|e| PipelineError::Stt(format!("Failed to parse language masks: {}", e)))?;

        let mask = masks.get(language).cloned().unwrap_or_else(|| {
            tracing::warn!(language = %language, "No mask found for language, using all tokens");
            Vec::new()
        });

        tracing::info!(
            language = %language,
            mask_size = mask.len(),
            true_count = mask.iter().filter(|&&x| x).count(),
            "IndicConformer: Loaded language mask"
        );

        Ok(mask)
    }

    /// Filter logits using language mask and re-normalize with log_softmax
    ///
    /// This is CRITICAL for correct decoding:
    /// 1. CTC decoder outputs 5633 joint vocab logprobs
    /// 2. We filter to only the 257 tokens for target language
    /// 3. Re-normalize with log_softmax after filtering
    /// 4. Return filtered logprobs for CTC decoding
    fn filter_and_normalize_logits(logits: &[f32], mask: &[bool]) -> Vec<f32> {
        // Debug: Log on first call only (using static counter would need mutex)
        static DEBUG_LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        let should_log = !DEBUG_LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed);

        if mask.is_empty() {
            if should_log {
                tracing::warn!("filter_and_normalize_logits: mask is empty! Returning as-is.");
            }
            return logits.to_vec();
        }

        if should_log {
            tracing::info!(
                logits_len = logits.len(),
                mask_len = mask.len(),
                "filter_and_normalize_logits: input sizes"
            );
        }

        // Step 1: Filter to only masked (allowed) tokens
        let filtered: Vec<f32> = logits.iter()
            .zip(mask.iter())
            .filter_map(|(&logit, &allowed)| if allowed { Some(logit) } else { None })
            .collect();

        if filtered.is_empty() {
            if should_log {
                tracing::error!("filter_and_normalize_logits: No tokens passed filter!");
            }
            return vec![0.0; 257];
        }

        if should_log {
            // Find argmax in filtered
            let (argmax_idx, argmax_val) = filtered.iter().enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or((0, &0.0));
            tracing::info!(
                filtered_len = filtered.len(),
                argmax_idx = argmax_idx,
                argmax_val = %argmax_val,
                first_5 = ?filtered.iter().take(5).collect::<Vec<_>>(),
                "filter_and_normalize_logits: filtered result"
            );
        }

        // Step 2: Apply log_softmax to re-normalize probabilities
        let max_logit = filtered.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let exp_sum: f32 = filtered.iter().map(|&x| (x - max_logit).exp()).sum();
        let log_sum_exp = max_logit + exp_sum.ln();

        let result: Vec<f32> = filtered.iter().map(|&x| x - log_sum_exp).collect();

        if should_log {
            let (argmax_idx, argmax_val) = result.iter().enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or((0, &0.0));
            tracing::info!(
                result_len = result.len(),
                argmax_idx = argmax_idx,
                argmax_val = %argmax_val,
                "filter_and_normalize_logits: after log_softmax"
            );
        }

        result
    }

    /// Legacy apply_language_mask for backward compatibility (masks to -inf)
    #[allow(dead_code)]
    fn apply_language_mask(logits: &mut [f32], mask: &[bool]) {
        if mask.is_empty() || mask.len() != logits.len() {
            return;
        }

        for (i, &allowed) in mask.iter().enumerate() {
            if !allowed {
                logits[i] = f32::NEG_INFINITY;
            }
        }
    }

    /// P0 FIX: Extract confidence from model logits using softmax
    ///
    /// Computes the probability of the predicted token by applying softmax
    /// to the logits and returning the max probability.
    fn extract_confidence_from_logits(logits: &[f32]) -> f32 {
        if logits.is_empty() {
            return 0.5; // Default for empty logits
        }

        // Find max logit for numerical stability (log-sum-exp trick)
        let max_logit = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        // Compute softmax denominator: sum(exp(logit - max))
        let exp_sum: f32 = logits.iter().map(|&x| (x - max_logit).exp()).sum();

        if exp_sum == 0.0 {
            return 0.5; // Avoid division by zero
        }

        // Find the maximum probability (confidence of best prediction)
        let max_prob = logits
            .iter()
            .map(|&x| (x - max_logit).exp() / exp_sum)
            .fold(0.0f32, f32::max);

        // Clamp to valid probability range
        max_prob.clamp(0.0, 1.0)
    }

    /// P0 FIX: Calculate word timestamp based on frame timing
    ///
    /// Uses actual frame-level timing instead of naive character-based heuristic.
    fn calculate_word_timestamp(
        &self,
        word: &str,
        start_frame: usize,
        end_frame: usize,
    ) -> (u64, u64) {
        // Each frame represents hop_length samples at sample_rate Hz
        let frame_duration_ms = (self.config.hop_length as f64
            / self.config.sample_rate.as_u32() as f64
            * 1000.0) as u64;

        let start_ms = start_frame as u64 * frame_duration_ms;
        let end_ms = end_frame as u64 * frame_duration_ms;

        // Ensure end is at least start + minimum word duration
        let min_word_duration = (word.chars().count() as u64 * 30).max(100); // At least 30ms per char, min 100ms
        let adjusted_end = end_ms.max(start_ms + min_word_duration);

        (start_ms, adjusted_end)
    }

    /// Get chunk size in samples
    fn chunk_samples(&self) -> usize {
        self.config.sample_rate.as_u32() as usize * self.config.chunk_ms as usize / 1000
    }

    /// Process audio samples
    pub fn process(&self, audio: &[f32]) -> Result<Option<TranscriptResult>, PipelineError> {
        let mut state = self.state.lock();
        state.audio_buffer.extend_from_slice(audio);

        let chunk_size = self.chunk_samples();
        let buffer_len = state.audio_buffer.len();

        // DIAGNOSTIC: Log buffer accumulation
        if buffer_len % (chunk_size * 5) < audio.len() {
            tracing::debug!(
                buffer_len = buffer_len,
                chunk_size = chunk_size,
                audio_in = audio.len(),
                total_frames = state.total_audio_frames,
                "IndicConformer: Audio buffer status"
            );
        }

        if buffer_len < chunk_size {
            return Ok(None);
        }

        // Process full chunks
        let mut chunks_processed = 0;
        while state.audio_buffer.len() >= chunk_size {
            let chunk: Vec<f32> = state.audio_buffer.drain(..chunk_size).collect();
            tracing::debug!(
                chunk_size = chunk_size,
                remaining_buffer = state.audio_buffer.len(),
                "IndicConformer: About to process chunk"
            );
            drop(state);

            self.process_chunk_internal(&chunk)?;
            chunks_processed += 1;

            tracing::debug!(
                chunks_processed = chunks_processed,
                "IndicConformer: Chunk processed, re-acquiring state lock"
            );
            state = self.state.lock();
            tracing::debug!(
                buffer_len = state.audio_buffer.len(),
                "IndicConformer: State lock acquired"
            );
        }

        // DIAGNOSTIC: Log chunk processing
        tracing::debug!(
            chunks_processed = chunks_processed,
            frame_count = state.frame_count,
            partial_interval = self.config.partial_interval,
            enable_partials = self.config.enable_partials,
            words_detected = state.words.len(),
            "IndicConformer: Chunk processing complete"
        );

        if self.config.enable_partials && state.frame_count >= self.config.partial_interval {
            state.frame_count = 0;
            drop(state);
            let partial = self.get_partial();

            // DIAGNOSTIC: Log partial emission
            match &partial {
                Some(p) => tracing::info!(
                    text = %p.text,
                    confidence = p.confidence,
                    words = p.words.len(),
                    "IndicConformer: Emitting partial transcript"
                ),
                None => tracing::debug!("IndicConformer: No partial to emit (decoder has no text)"),
            }
            return Ok(partial);
        }

        Ok(None)
    }

    /// Process a single audio chunk
    #[cfg(feature = "onnx")]
    fn process_chunk_internal(&self, audio: &[f32]) -> Result<(), PipelineError> {
        let start_time = std::time::Instant::now();

        // Extract mel spectrogram - returns flat vec in [time, n_mels] order
        let mel = self.mel_filterbank.extract(audio);
        let mel_time = start_time.elapsed();
        let n_frames = mel.len() / self.config.n_mels;

        // Model expects [batch, n_mels, time] shape
        // mel is in row-major [time][n_mels] order, need to transpose to [n_mels][time]
        // Build the transposed array directly for correct memory layout
        let n_mels = self.config.n_mels;
        let mut transposed = vec![0.0f32; mel.len()];
        for t in 0..n_frames {
            for m in 0..n_mels {
                // mel[t * n_mels + m] -> transposed[m * n_frames + t]
                transposed[m * n_frames + t] = mel[t * n_mels + m];
            }
        }

        // Create array with shape [1, n_mels, time]
        let mel_input = Array3::from_shape_vec((1, n_mels, n_frames), transposed)
            .map_err(|e| PipelineError::Stt(format!("Failed to create mel tensor: {}", e)))?;

        // Create length tensor [batch_size] - required by encoder
        let length_input = ndarray::Array1::from_vec(vec![n_frames as i64]);

        // Run encoder - convert ndarray to ort Tensor
        let encoder_start = std::time::Instant::now();
        let mut encoder = self.encoder_session.lock();

        // Create tensors (ort 2.0 API)
        let mel_tensor = Tensor::from_array(mel_input)
            .map_err(|e| PipelineError::Model(e.to_string()))?;
        let length_tensor = Tensor::from_array(length_input)
            .map_err(|e| PipelineError::Model(e.to_string()))?;

        let encoder_outputs = encoder
            .run(ort::inputs![
                "audio_signal" => mel_tensor,
                "length" => length_tensor,
            ])
            .map_err(|e| PipelineError::Model(format!("Encoder failed: {}", e)))?;
        let encoder_time = encoder_start.elapsed();

        // Get encoder output - model uses "outputs" not "encoded"
        let encoded = encoder_outputs
            .get("outputs")
            .ok_or_else(|| PipelineError::Model("Missing 'outputs' from encoder".to_string()))?
            .try_extract_array::<f32>()
            .map_err(|e| PipelineError::Model(e.to_string()))?;

        // Run CTC decoder
        let decoder_start = std::time::Instant::now();
        let mut decoder = self.decoder_session.lock();

        let encoded_tensor = Tensor::from_array(encoded.to_owned())
            .map_err(|e| PipelineError::Model(e.to_string()))?;

        let decoder_outputs = decoder
            .run(ort::inputs![
                "encoder_output" => encoded_tensor,
            ])
            .map_err(|e| PipelineError::Model(format!("Decoder failed: {}", e)))?;
        let decoder_time = decoder_start.elapsed();

        tracing::debug!(
            mel_ms = mel_time.as_millis() as u64,
            encoder_ms = encoder_time.as_millis() as u64,
            decoder_ms = decoder_time.as_millis() as u64,
            total_ms = start_time.elapsed().as_millis() as u64,
            "IndicConformer: ONNX inference timing"
        );

        // Get logits - try various output names (model uses "logprobs")
        let logits = if let Some(output) = decoder_outputs.get("logprobs") {
            output.try_extract_array::<f32>()
                .map_err(|e| PipelineError::Model(e.to_string()))?
        } else if let Some(output) = decoder_outputs.get("logits") {
            output.try_extract_array::<f32>()
                .map_err(|e| PipelineError::Model(e.to_string()))?
        } else if let Some(output) = decoder_outputs.get("log_probs") {
            output.try_extract_array::<f32>()
                .map_err(|e| PipelineError::Model(e.to_string()))?
        } else {
            return Err(PipelineError::Model("Missing logits output".to_string()));
        };

        let shape = logits.shape().to_vec();

        // Process each frame through enhanced decoder
        if shape.len() >= 2 {
            let n_frames = shape[1];
            let vocab_size = if shape.len() > 2 { shape[2] } else { shape[1] };

            // DIAGNOSTIC: Log decoder input dimensions
            let frame_count = self.state.lock().frame_count;
            if frame_count % 5 == 0 {
                tracing::debug!(
                    n_frames = n_frames,
                    vocab_size = vocab_size,
                    shape = ?shape,
                    chunk = frame_count,
                    "IndicConformer: Processing decoder output frames"
                );
            }

            for frame_idx in 0..n_frames {
                let raw_logits: Vec<f32> = if shape.len() > 2 {
                    (0..vocab_size)
                        .map(|v| logits[[0, frame_idx, v]])
                        .collect()
                } else {
                    (0..vocab_size)
                        .map(|v| logits[[frame_idx, v]])
                        .collect()
                };

                // CRITICAL: Filter 5633 joint vocab → 257 per-language tokens + re-normalize
                let frame_logits = Self::filter_and_normalize_logits(&raw_logits, &self.language_mask);

                // P0 FIX: Extract actual confidence from filtered logits
                let frame_confidence = Self::extract_confidence_from_logits(&frame_logits);

                // DIAGNOSTIC: Log top token for first few frames of each chunk
                if frame_idx < 3 && frame_count % 3 == 0 {
                    let top_idx = frame_logits.iter().enumerate()
                        .filter(|(_, v)| v.is_finite())
                        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    let top_token = self.vocabulary.get_token(top_idx as u32).map(|s| s.to_string()).unwrap_or_else(|| format!("?{}", top_idx));
                    tracing::trace!(
                        chunk = frame_count,
                        frame = frame_idx,
                        top_token = %top_token,
                        top_idx = top_idx,
                        confidence = format!("{:.2}", frame_confidence),
                        "IndicConformer: Frame token"
                    );
                }

                // Update running confidence stats
                {
                    let mut state = self.state.lock();
                    state.confidence_sum += frame_confidence;
                    state.confidence_count += 1;
                    state.current_word_confidence += frame_confidence;
                    state.current_word_frames += 1;
                    state.total_audio_frames += 1;
                }

                if let Some(word) = self.decoder.process_frame(&frame_logits)? {
                    tracing::info!(word = %word, "IndicConformer: Word detected by decoder");
                    self.add_word_with_confidence(&word, frame_confidence);
                }
            }
        }

        let mut state = self.state.lock();
        state.frame_count += 1;

        // DIAGNOSTIC: Log current decoder state periodically
        if state.frame_count % 5 == 0 {
            let current_text = self.decoder.current_best();
            tracing::debug!(
                chunk = state.frame_count,
                total_frames = state.total_audio_frames,
                words_detected = state.words.len(),
                current_text = %current_text,
                "IndicConformer: Chunk complete"
            );
        }

        Ok(())
    }

    /// Process a single audio chunk using candle-onnx
    #[cfg(feature = "candle-onnx")]
    fn process_chunk_internal(&self, audio: &[f32]) -> Result<(), PipelineError> {
        use std::collections::HashMap;

        let frame_count = self.state.lock().frame_count;
        if frame_count % 50 == 0 {
            tracing::debug!(
                audio_len = audio.len(),
                frame = frame_count,
                "IndicConformer: Processing audio chunk"
            );
        }

        // Extract mel spectrogram
        let mel = self.mel_filterbank.extract(audio);
        let n_frames = mel.len() / self.config.n_mels;

        if frame_count % 50 == 0 {
            tracing::debug!(
                mel_len = mel.len(),
                n_frames = n_frames,
                n_mels = self.config.n_mels,
                "IndicConformer: Extracted mel spectrogram"
            );
        }

        // Create input tensor [batch, n_mels, time] - ONNX model expects this shape order
        // The mel spectrogram is extracted as [time, n_mels], so we need to transpose
        let mel_tensor = Tensor::from_vec(
            mel.clone(),
            (n_frames, self.config.n_mels),
            &self.device,
        )
        .and_then(|t| t.transpose(0, 1)) // [n_mels, time]
        .and_then(|t| t.unsqueeze(0))    // [1, n_mels, time]
        .map_err(|e| {
            tracing::error!(error = %e, "IndicConformer: Failed to create mel tensor");
            PipelineError::Stt(format!("Failed to create mel tensor: {}", e))
        })?;

        // Create length tensor (required by encoder)
        let length_tensor = Tensor::from_vec(
            vec![n_frames as i64],
            (1,),
            &self.device,
        )
        .map_err(|e| PipelineError::Stt(format!("Failed to create length tensor: {}", e)))?;

        // Run encoder
        let mut encoder_inputs = HashMap::new();
        encoder_inputs.insert("audio_signal".to_string(), mel_tensor);
        encoder_inputs.insert("length".to_string(), length_tensor);

        let encoder_outputs = candle_onnx::simple_eval(&self.encoder_model, encoder_inputs)
            .map_err(|e| {
                tracing::error!(error = %e, "IndicConformer: Encoder inference failed");
                PipelineError::Model(format!("Encoder inference failed: {}", e))
            })?;

        // Get encoder output (model uses "outputs" not "encoded")
        let encoded = encoder_outputs
            .get("outputs")
            .ok_or_else(|| {
                let keys: Vec<_> = encoder_outputs.keys().collect();
                tracing::error!(available_keys = ?keys, "IndicConformer: Missing encoder output 'outputs'");
                PipelineError::Model(format!("Missing 'outputs' from encoder. Available: {:?}", keys))
            })?;

        // Run CTC decoder
        let mut decoder_inputs = HashMap::new();
        decoder_inputs.insert("encoder_output".to_string(), encoded.clone());

        let decoder_outputs = candle_onnx::simple_eval(&self.decoder_model, decoder_inputs)
            .map_err(|e| PipelineError::Model(format!("CTC decoder inference failed: {}", e)))?;

        // Get logprobs (model uses "logprobs")
        let logits = decoder_outputs
            .get("logprobs")
            .or_else(|| decoder_outputs.get("logits"))
            .or_else(|| decoder_outputs.get("log_probs"))
            .ok_or_else(|| {
                let keys: Vec<_> = decoder_outputs.keys().collect();
                tracing::error!(available_keys = ?keys, "IndicConformer: Missing decoder output");
                PipelineError::Model(format!("Missing logprobs from decoder. Available: {:?}", keys))
            })?;

        // Get shape and convert to Vec for processing
        let shape = logits.dims();
        let logits_flat: Vec<f32> = logits
            .flatten_all()
            .map_err(|e| PipelineError::Model(format!("Failed to flatten logits: {}", e)))?
            .to_vec1()
            .map_err(|e| PipelineError::Model(format!("Failed to convert logits: {}", e)))?;

        // Process each frame through enhanced decoder
        if shape.len() >= 2 {
            let n_output_frames = shape[1];
            let vocab_size = if shape.len() > 2 { shape[2] } else { shape[1] };

            for frame_idx in 0..n_output_frames {
                let frame_start = if shape.len() > 2 {
                    frame_idx * vocab_size
                } else {
                    frame_idx * vocab_size
                };
                let frame_end = frame_start + vocab_size;

                let raw_logits: Vec<f32> = if frame_end <= logits_flat.len() {
                    logits_flat[frame_start..frame_end].to_vec()
                } else {
                    continue; // Skip incomplete frames
                };

                // CRITICAL: Filter 5633 joint vocab → 257 per-language tokens + re-normalize
                let frame_logits = Self::filter_and_normalize_logits(&raw_logits, &self.language_mask);

                // Extract confidence from filtered logits
                let frame_confidence = Self::extract_confidence_from_logits(&frame_logits);

                // Update running confidence stats
                {
                    let mut state = self.state.lock();
                    state.confidence_sum += frame_confidence;
                    state.confidence_count += 1;
                    state.current_word_confidence += frame_confidence;
                    state.current_word_frames += 1;
                    state.total_audio_frames += 1;
                }

                if let Some(word) = self.decoder.process_frame(&frame_logits)? {
                    self.add_word_with_confidence(&word, frame_confidence);
                }
            }
        }

        let mut state = self.state.lock();
        state.frame_count += 1;
        Ok(())
    }

    /// Process a single audio chunk (stub when neither ONNX backend is enabled)
    #[cfg(not(any(feature = "onnx", feature = "candle-onnx")))]
    fn process_chunk_internal(&self, _audio: &[f32]) -> Result<(), PipelineError> {
        let mut state = self.state.lock();
        state.frame_count += 1;
        Ok(())
    }

    /// P0 FIX: Add a detected word with actual confidence from model
    fn add_word_with_confidence(&self, word: &str, last_frame_confidence: f32) {
        let mut state = self.state.lock();

        // P0 FIX: Calculate word confidence as average of frames for this word
        let word_confidence = if state.current_word_frames > 0 {
            (state.current_word_confidence / state.current_word_frames as f32).clamp(0.0, 1.0)
        } else {
            last_frame_confidence.clamp(0.0, 1.0)
        };

        // P0 FIX: Calculate timestamps based on actual frame positions
        let frame_duration_ms = (self.config.hop_length as f64
            / self.config.sample_rate.as_u32() as f64
            * 1000.0) as u64;

        // Calculate word boundaries based on frames processed
        let word_end_frame = state.total_audio_frames;
        let word_start_frame = word_end_frame.saturating_sub(state.current_word_frames);

        let word_start = state.start_time_ms + (word_start_frame as u64 * frame_duration_ms);
        let word_end = state.start_time_ms + (word_end_frame as u64 * frame_duration_ms);

        // Ensure minimum duration
        let min_duration = (word.chars().count() as u64 * 30).max(100);
        let adjusted_end = word_end.max(word_start + min_duration);

        state.words.push(WordTimestamp {
            word: word.trim().to_string(),
            start_ms: word_start,
            end_ms: adjusted_end,
            confidence: word_confidence,
        });

        // Reset per-word accumulators
        state.current_word_confidence = 0.0;
        state.current_word_frames = 0;
    }

    /// Legacy add_word for backward compatibility (uses default confidence)
    #[allow(dead_code)]
    fn add_word(&self, word: &str) {
        self.add_word_with_confidence(word, 0.5)
    }

    /// Get current partial result
    fn get_partial(&self) -> Option<TranscriptResult> {
        let text = self.decoder.current_best();
        if text.is_empty() {
            return None;
        }

        let state = self.state.lock();
        let words = state.words.clone();
        let start_ms = state.start_time_ms;
        let end_ms = words.last().map(|w| w.end_ms).unwrap_or(start_ms);

        // P0 FIX: Calculate actual confidence as average of all frames
        let confidence = if state.confidence_count > 0 {
            (state.confidence_sum / state.confidence_count as f32).clamp(0.0, 1.0)
        } else {
            // Fallback: average word confidences if available
            if !words.is_empty() {
                words.iter().map(|w| w.confidence).sum::<f32>() / words.len() as f32
            } else {
                0.5 // Neutral confidence when no data
            }
        };

        Some(TranscriptResult {
            text,
            is_final: false,
            confidence,
            start_time_ms: start_ms,
            end_time_ms: end_ms,
            language: Some(self.config.language.clone()),
            words,
        })
    }

    /// Finalize and get final result
    pub fn finalize(&self) -> TranscriptResult {
        // DIAGNOSTIC: Log finalize entry
        let finalize_start = std::time::Instant::now();

        // Process remaining audio
        let remaining: Vec<f32> = {
            let mut state = self.state.lock();
            state.audio_buffer.drain(..).collect()
        };

        let remaining_len = remaining.len();
        if !remaining.is_empty() {
            let chunk_size = self.chunk_samples();
            let mut padded = remaining;
            padded.resize(chunk_size, 0.0);
            let _ = self.process_chunk_internal(&padded);
        }

        let text = self.decoder.finalize();
        let state = self.state.lock();
        let words = state.words.clone();
        let start_ms = state.start_time_ms;
        let end_ms = words.last().map(|w| w.end_ms).unwrap_or(start_ms);

        // P0 FIX: Calculate actual final confidence as average of all frames
        let confidence = if state.confidence_count > 0 {
            (state.confidence_sum / state.confidence_count as f32).clamp(0.0, 1.0)
        } else {
            // Fallback: average word confidences if available
            if !words.is_empty() {
                words.iter().map(|w| w.confidence).sum::<f32>() / words.len() as f32
            } else {
                0.5 // Neutral confidence when no data
            }
        };

        // DIAGNOSTIC: Log finalize results
        tracing::info!(
            text = %text,
            text_len = text.len(),
            words_count = words.len(),
            confidence = format!("{:.2}", confidence),
            total_frames = state.total_audio_frames,
            confidence_count = state.confidence_count,
            remaining_audio = remaining_len,
            finalize_ms = finalize_start.elapsed().as_millis() as u64,
            "IndicConformer: Finalize complete"
        );

        TranscriptResult {
            text,
            is_final: true,
            confidence,
            start_time_ms: start_ms,
            end_time_ms: end_ms,
            language: Some(self.config.language.clone()),
            words,
        }
    }

    /// Reset STT state
    pub fn reset(&self) {
        let mut state = self.state.lock();
        state.audio_buffer.clear();
        state.frame_count = 0;
        state.words.clear();
        state.start_time_ms = 0;
        state.encoder_state = None;
        // P0 FIX: Reset confidence tracking state
        state.confidence_sum = 0.0;
        state.confidence_count = 0;
        state.current_word_confidence = 0.0;
        state.current_word_frames = 0;
        state.total_audio_frames = 0;
        self.decoder.reset();
    }

    /// Set start time for word timestamps
    pub fn set_start_time(&self, time_ms: u64) {
        self.state.lock().start_time_ms = time_ms;
    }

    /// Add entities to boost in decoder
    pub fn add_entities(&self, entities: impl IntoIterator<Item = impl AsRef<str>>) {
        self.decoder.add_entities(entities);
    }

    /// Get vocabulary
    pub fn vocabulary(&self) -> &Vocabulary {
        &self.vocabulary
    }

    /// Get supported languages
    pub fn supported_languages() -> Vec<&'static str> {
        vec![
            "as",  // Assamese
            "bn",  // Bengali
            "brx", // Bodo
            "doi", // Dogri
            "gu",  // Gujarati
            "hi",  // Hindi
            "kn",  // Kannada
            "kok", // Konkani
            "ks",  // Kashmiri
            "mai", // Maithili
            "ml",  // Malayalam
            "mni", // Manipuri
            "mr",  // Marathi
            "ne",  // Nepali
            "or",  // Odia
            "pa",  // Punjabi
            "sa",  // Sanskrit
            "sat", // Santali
            "sd",  // Sindhi
            "ta",  // Tamil
            "te",  // Telugu
            "ur",  // Urdu
        ]
    }
}

#[async_trait::async_trait]
impl SttBackend for IndicConformerStt {
    async fn process_chunk(
        &mut self,
        audio: &[f32],
    ) -> Result<Option<TranscriptResult>, PipelineError> {
        IndicConformerStt::process(self, audio)
    }

    async fn finalize(&mut self) -> Result<TranscriptResult, PipelineError> {
        Ok(IndicConformerStt::finalize(self))
    }

    fn reset(&mut self) {
        IndicConformerStt::reset(self);
    }

    fn partial(&self) -> Option<&TranscriptResult> {
        None // Partials are returned through process_chunk()
    }

    fn process(&mut self, audio: &[f32]) -> Result<Option<TranscriptResult>, PipelineError> {
        IndicConformerStt::process(self, audio)
    }

    fn finalize_sync(&mut self) -> TranscriptResult {
        IndicConformerStt::finalize(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voice_agent_core::SampleRate;

    #[test]
    fn test_supported_languages() {
        let languages = IndicConformerStt::supported_languages();
        assert!(languages.contains(&"hi"));
        assert!(languages.contains(&"mr"));
        assert!(languages.contains(&"bn"));
    }

    #[cfg(not(any(feature = "onnx", feature = "candle-onnx")))]
    #[test]
    fn test_indicconformer_simple() {
        let config = IndicConformerConfig::default();
        let stt = IndicConformerStt::simple(config).unwrap();
        // Default vocabulary has 8000 tokens (placeholder)
        assert_eq!(stt.vocabulary().len(), 8000);
    }
}
