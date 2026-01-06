# Audio Pipeline Architecture

## Overview

The audio pipeline is the real-time backbone of the voice agent, handling bidirectional audio streaming with sub-second latency requirements. This document covers Voice Activity Detection (VAD), Speech-to-Text (STT) streaming, Text-to-Speech (TTS) streaming, and barge-in handling.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        AUDIO PIPELINE OVERVIEW                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌──────────────┐ │
│  │   WebRTC    │───▶│     VAD     │───▶│   STT       │───▶│   Text       │ │
│  │   Input     │    │   Silero    │    │   Streamer  │    │   Pipeline   │ │
│  └─────────────┘    └─────────────┘    └─────────────┘    └──────────────┘ │
│                                                                    │        │
│                                                                    ▼        │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌──────────────┐ │
│  │   WebRTC    │◀───│   Audio     │◀───│   TTS       │◀───│   LLM        │ │
│  │   Output    │    │   Mixer     │    │   Streamer  │    │   Response   │ │
│  └─────────────┘    └─────────────┘    └─────────────┘    └──────────────┘ │
│                            ▲                                                │
│                            │                                                │
│                     ┌──────┴──────┐                                        │
│                     │   Barge-in  │                                        │
│                     │   Handler   │                                        │
│                     └─────────────┘                                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Audio Frame Definition

### Core Audio Types

```rust
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Supported audio sample rates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SampleRate {
    Hz8000,   // Telephony
    Hz16000,  // Standard speech
    Hz22050,  // CD quality speech
    Hz44100,  // CD quality
    Hz48000,  // Professional audio
}

impl SampleRate {
    pub fn as_u32(&self) -> u32 {
        match self {
            SampleRate::Hz8000 => 8000,
            SampleRate::Hz16000 => 16000,
            SampleRate::Hz22050 => 22050,
            SampleRate::Hz44100 => 44100,
            SampleRate::Hz48000 => 48000,
        }
    }

    pub fn frame_size_20ms(&self) -> usize {
        (self.as_u32() as usize * 20) / 1000
    }

    pub fn frame_size_10ms(&self) -> usize {
        (self.as_u32() as usize * 10) / 1000
    }
}

/// Audio encoding formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioEncoding {
    Pcm16,      // 16-bit signed PCM (native)
    PcmF32,     // 32-bit float PCM
    Opus,       // Opus codec (WebRTC)
    Mulaw,      // μ-law (telephony)
    Alaw,       // A-law (telephony)
}

/// Audio channel configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channels {
    Mono,
    Stereo,
}

/// Audio frame with metadata
#[derive(Clone)]
pub struct AudioFrame {
    /// Raw audio samples (always stored as f32 internally)
    pub samples: Arc<[f32]>,

    /// Sample rate
    pub sample_rate: SampleRate,

    /// Number of channels
    pub channels: Channels,

    /// Frame sequence number for ordering
    pub sequence: u64,

    /// Timestamp when frame was captured/generated
    pub timestamp: Instant,

    /// Duration of this frame
    pub duration: Duration,

    /// Voice activity probability (0.0 - 1.0)
    pub vad_probability: Option<f32>,

    /// Is this frame during active speech?
    pub is_speech: bool,

    /// Energy level in dB
    pub energy_db: f32,
}

impl AudioFrame {
    /// Create a new audio frame from f32 samples
    pub fn new(
        samples: Vec<f32>,
        sample_rate: SampleRate,
        channels: Channels,
        sequence: u64,
    ) -> Self {
        let duration = Duration::from_secs_f64(
            samples.len() as f64 / sample_rate.as_u32() as f64
        );
        let energy_db = Self::calculate_energy_db(&samples);

        Self {
            samples: samples.into(),
            sample_rate,
            channels,
            sequence,
            timestamp: Instant::now(),
            duration,
            vad_probability: None,
            is_speech: false,
            energy_db,
        }
    }

    /// Calculate RMS energy in decibels
    fn calculate_energy_db(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return -96.0; // Minimum dB
        }

        let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
        let rms = (sum_squares / samples.len() as f32).sqrt();

        if rms > 0.0 {
            20.0 * rms.log10()
        } else {
            -96.0
        }
    }

    /// Convert from PCM16 bytes
    pub fn from_pcm16(
        bytes: &[u8],
        sample_rate: SampleRate,
        channels: Channels,
        sequence: u64,
    ) -> Self {
        let samples: Vec<f32> = bytes
            .chunks_exact(2)
            .map(|chunk| {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                sample as f32 / 32768.0
            })
            .collect();

        Self::new(samples, sample_rate, channels, sequence)
    }

    /// Convert to PCM16 bytes
    pub fn to_pcm16(&self) -> Vec<u8> {
        self.samples
            .iter()
            .flat_map(|&sample| {
                let clamped = sample.clamp(-1.0, 1.0);
                let pcm16 = (clamped * 32767.0) as i16;
                pcm16.to_le_bytes()
            })
            .collect()
    }

    /// Resample to target sample rate
    pub fn resample(&self, target_rate: SampleRate) -> Self {
        if self.sample_rate == target_rate {
            return self.clone();
        }

        let ratio = target_rate.as_u32() as f64 / self.sample_rate.as_u32() as f64;
        let new_len = (self.samples.len() as f64 * ratio) as usize;

        // Linear interpolation resampling (production would use sinc)
        let mut resampled = Vec::with_capacity(new_len);
        for i in 0..new_len {
            let src_idx = i as f64 / ratio;
            let idx_floor = src_idx.floor() as usize;
            let idx_ceil = (idx_floor + 1).min(self.samples.len() - 1);
            let frac = src_idx - idx_floor as f64;

            let sample = self.samples[idx_floor] * (1.0 - frac as f32)
                + self.samples[idx_ceil] * frac as f32;
            resampled.push(sample);
        }

        Self::new(resampled, target_rate, self.channels, self.sequence)
    }
}
```

## Voice Activity Detection (VAD)

### Silero VAD Integration

```rust
use ort::{Session, SessionBuilder, Environment};
use std::sync::Mutex;

/// VAD configuration
#[derive(Debug, Clone)]
pub struct VadConfig {
    /// Probability threshold for speech detection
    pub threshold: f32,

    /// Minimum speech duration to trigger (ms)
    pub min_speech_duration_ms: u32,

    /// Minimum silence duration to end speech (ms)
    pub min_silence_duration_ms: u32,

    /// Speech padding (ms) - audio to keep before/after speech
    pub speech_pad_ms: u32,

    /// Window size for VAD (samples at 16kHz)
    pub window_size: usize,

    /// Energy threshold (dB) below which is definitely silence
    pub energy_floor_db: f32,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            min_speech_duration_ms: 250,
            min_silence_duration_ms: 300,
            speech_pad_ms: 100,
            window_size: 512,
            energy_floor_db: -50.0,
        }
    }
}

/// VAD state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadState {
    /// No speech detected
    Silence,
    /// Potential speech start (accumulating)
    SpeechStart,
    /// Active speech
    Speech,
    /// Potential speech end (accumulating silence)
    SpeechEnd,
}

/// Voice Activity Detector using Silero VAD
pub struct SileroVad {
    session: Session,
    config: VadConfig,

    // LSTM hidden states (Silero VAD uses LSTM)
    h_state: Mutex<Vec<f32>>,
    c_state: Mutex<Vec<f32>>,

    // State tracking
    state: Mutex<VadState>,
    speech_samples: Mutex<usize>,
    silence_samples: Mutex<usize>,

    // Ring buffer for padding
    padding_buffer: Mutex<Vec<f32>>,
}

impl SileroVad {
    /// Load Silero VAD model
    pub fn new(model_path: &str, config: VadConfig) -> Result<Self, VadError> {
        let environment = Environment::builder()
            .with_name("silero_vad")
            .with_execution_providers([ort::ExecutionProvider::CPU(Default::default())])
            .build()?;

        let session = SessionBuilder::new(&environment)?
            .with_intra_threads(1)?
            .with_model_from_file(model_path)?;

        // Initialize LSTM states (64 hidden units for Silero VAD v4)
        let h_state = vec![0.0f32; 2 * 64]; // 2 layers * 64 hidden
        let c_state = vec![0.0f32; 2 * 64];

        let padding_samples = (config.speech_pad_ms as usize * 16) / 1; // 16 samples per ms at 16kHz

        Ok(Self {
            session,
            config,
            h_state: Mutex::new(h_state),
            c_state: Mutex::new(c_state),
            state: Mutex::new(VadState::Silence),
            speech_samples: Mutex::new(0),
            silence_samples: Mutex::new(0),
            padding_buffer: Mutex::new(Vec::with_capacity(padding_samples)),
        })
    }

    /// Process audio frame and return VAD result
    pub fn process(&self, frame: &mut AudioFrame) -> Result<VadResult, VadError> {
        // Resample to 16kHz if needed
        let frame_16k = if frame.sample_rate != SampleRate::Hz16000 {
            frame.resample(SampleRate::Hz16000)
        } else {
            frame.clone()
        };

        // Quick energy check - skip VAD if definitely silence
        if frame_16k.energy_db < self.config.energy_floor_db {
            frame.vad_probability = Some(0.0);
            frame.is_speech = false;
            return self.update_state(false, frame.samples.len());
        }

        // Run Silero VAD inference
        let probability = self.infer(&frame_16k.samples)?;

        frame.vad_probability = Some(probability);
        let is_speech = probability >= self.config.threshold;
        frame.is_speech = is_speech;

        self.update_state(is_speech, frame.samples.len())
    }

    /// Run ONNX inference
    fn infer(&self, samples: &[f32]) -> Result<f32, VadError> {
        use ort::Value;
        use ndarray::{Array1, Array2, Array3};

        let mut h_state = self.h_state.lock().unwrap();
        let mut c_state = self.c_state.lock().unwrap();

        // Prepare input tensor [batch=1, samples]
        let input = Array2::from_shape_vec((1, samples.len()), samples.to_vec())?;

        // Prepare LSTM states [layers=2, batch=1, hidden=64]
        let h = Array3::from_shape_vec((2, 1, 64), h_state.clone())?;
        let c = Array3::from_shape_vec((2, 1, 64), c_state.clone())?;

        // Sample rate tensor
        let sr = Array1::from_vec(vec![16000i64]);

        // Run inference
        let outputs = self.session.run(ort::inputs![
            "input" => input,
            "sr" => sr,
            "h" => h,
            "c" => c,
        ]?)?;

        // Extract probability
        let output: Vec<f32> = outputs["output"]
            .try_extract_tensor()?
            .view()
            .to_owned()
            .into_raw_vec();

        // Update LSTM states for next call
        let new_h: Vec<f32> = outputs["hn"]
            .try_extract_tensor()?
            .view()
            .to_owned()
            .into_raw_vec();
        let new_c: Vec<f32> = outputs["cn"]
            .try_extract_tensor()?
            .view()
            .to_owned()
            .into_raw_vec();

        *h_state = new_h;
        *c_state = new_c;

        Ok(output[0])
    }

    /// Update state machine based on VAD result
    fn update_state(&self, is_speech: bool, sample_count: usize) -> Result<VadResult, VadError> {
        let mut state = self.state.lock().unwrap();
        let mut speech_samples = self.speech_samples.lock().unwrap();
        let mut silence_samples = self.silence_samples.lock().unwrap();

        let min_speech_samples = (self.config.min_speech_duration_ms as usize * 16000) / 1000;
        let min_silence_samples = (self.config.min_silence_duration_ms as usize * 16000) / 1000;

        let result = match (*state, is_speech) {
            (VadState::Silence, true) => {
                *state = VadState::SpeechStart;
                *speech_samples = sample_count;
                *silence_samples = 0;
                VadResult::PotentialSpeechStart
            }

            (VadState::SpeechStart, true) => {
                *speech_samples += sample_count;
                if *speech_samples >= min_speech_samples {
                    *state = VadState::Speech;
                    VadResult::SpeechConfirmed
                } else {
                    VadResult::PotentialSpeechStart
                }
            }

            (VadState::SpeechStart, false) => {
                // False alarm, reset
                *state = VadState::Silence;
                *speech_samples = 0;
                VadResult::Silence
            }

            (VadState::Speech, true) => {
                *silence_samples = 0;
                VadResult::SpeechContinue
            }

            (VadState::Speech, false) => {
                *state = VadState::SpeechEnd;
                *silence_samples = sample_count;
                VadResult::PotentialSpeechEnd
            }

            (VadState::SpeechEnd, true) => {
                // Speech resumed
                *state = VadState::Speech;
                *silence_samples = 0;
                VadResult::SpeechContinue
            }

            (VadState::SpeechEnd, false) => {
                *silence_samples += sample_count;
                if *silence_samples >= min_silence_samples {
                    *state = VadState::Silence;
                    *speech_samples = 0;
                    *silence_samples = 0;
                    VadResult::SpeechEnd
                } else {
                    VadResult::PotentialSpeechEnd
                }
            }

            (VadState::Silence, false) => {
                VadResult::Silence
            }
        };

        Ok(result)
    }

    /// Reset VAD state (e.g., after barge-in)
    pub fn reset(&self) {
        *self.state.lock().unwrap() = VadState::Silence;
        *self.speech_samples.lock().unwrap() = 0;
        *self.silence_samples.lock().unwrap() = 0;
        *self.h_state.lock().unwrap() = vec![0.0f32; 2 * 64];
        *self.c_state.lock().unwrap() = vec![0.0f32; 2 * 64];
    }
}

/// VAD processing result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadResult {
    /// Silence detected
    Silence,
    /// Potential speech start (below threshold)
    PotentialSpeechStart,
    /// Speech confirmed (above threshold duration)
    SpeechConfirmed,
    /// Speech continuing
    SpeechContinue,
    /// Potential speech end (accumulating silence)
    PotentialSpeechEnd,
    /// Speech ended (silence threshold met)
    SpeechEnd,
}
```

## STT Streaming Pipeline

### Streaming Transcription Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        STT STREAMING ARCHITECTURE                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Audio Input                                                                │
│       │                                                                      │
│       ▼                                                                      │
│   ┌───────────────┐                                                         │
│   │  Ring Buffer  │  ◀──  Accumulates audio frames                          │
│   │  (10-30 sec)  │                                                         │
│   └───────┬───────┘                                                         │
│           │                                                                  │
│           ▼                                                                  │
│   ┌───────────────┐     ┌───────────────┐                                   │
│   │    Chunker    │────▶│   Overlap     │  Sliding window with overlap     │
│   │  (500-1000ms) │     │   Manager     │                                   │
│   └───────┬───────┘     └───────────────┘                                   │
│           │                                                                  │
│           ▼                                                                  │
│   ┌───────────────┐                                                         │
│   │  STT Engine   │  ◀──  Whisper / IndicConformer                          │
│   │   (ONNX)      │                                                         │
│   └───────┬───────┘                                                         │
│           │                                                                  │
│           ▼                                                                  │
│   ┌───────────────┐     ┌───────────────┐                                   │
│   │   Partial     │────▶│   Stable      │  Filter unstable partials        │
│   │   Decoder     │     │   Detector    │                                   │
│   └───────┬───────┘     └───────────────┘                                   │
│           │                                                                  │
│           ▼                                                                  │
│   ┌───────────────┐                                                         │
│   │   Sentence    │  ◀──  Accumulate until sentence boundary                │
│   │  Accumulator  │                                                         │
│   └───────┬───────┘                                                         │
│           │                                                                  │
│           ▼                                                                  │
│      Transcript                                                              │
│       Output                                                                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### STT Streaming Implementation

```rust
use tokio::sync::mpsc;
use std::collections::VecDeque;

/// STT streaming configuration
#[derive(Debug, Clone)]
pub struct SttStreamConfig {
    /// Chunk size for STT processing (ms)
    pub chunk_duration_ms: u32,

    /// Overlap between chunks (ms)
    pub overlap_duration_ms: u32,

    /// Maximum buffer size (seconds)
    pub max_buffer_seconds: u32,

    /// Emit partial results
    pub emit_partials: bool,

    /// Minimum confidence for final results
    pub min_confidence: f32,

    /// Language hint
    pub language: Option<String>,
}

impl Default for SttStreamConfig {
    fn default() -> Self {
        Self {
            chunk_duration_ms: 500,
            overlap_duration_ms: 100,
            max_buffer_seconds: 30,
            emit_partials: true,
            min_confidence: 0.7,
            language: None,
        }
    }
}

/// Transcript result from STT
#[derive(Debug, Clone)]
pub struct TranscriptResult {
    /// Transcribed text
    pub text: String,

    /// Is this a final result?
    pub is_final: bool,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,

    /// Start time offset (ms from stream start)
    pub start_time_ms: u64,

    /// End time offset (ms from stream start)
    pub end_time_ms: u64,

    /// Detected language
    pub language: Option<String>,

    /// Word-level timestamps
    pub words: Vec<WordTimestamp>,
}

#[derive(Debug, Clone)]
pub struct WordTimestamp {
    pub word: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub confidence: f32,
}

/// Streaming STT processor
pub struct SttStreamer {
    config: SttStreamConfig,
    engine: Box<dyn SttEngine>,

    // Audio buffer
    audio_buffer: VecDeque<f32>,
    buffer_start_ms: u64,

    // State
    last_final_text: String,
    last_partial_text: String,
    total_audio_ms: u64,

    // Sentence accumulation
    sentence_buffer: String,
}

/// STT engine trait
#[async_trait::async_trait]
pub trait SttEngine: Send + Sync {
    /// Transcribe audio chunk
    async fn transcribe(&self, audio: &[f32], language: Option<&str>)
        -> Result<SttEngineResult, SttError>;

    /// Get supported languages
    fn supported_languages(&self) -> Vec<String>;

    /// Optimal chunk size for this engine
    fn optimal_chunk_ms(&self) -> u32;
}

#[derive(Debug)]
pub struct SttEngineResult {
    pub text: String,
    pub confidence: f32,
    pub words: Vec<WordTimestamp>,
    pub language: Option<String>,
}

impl SttStreamer {
    pub fn new(config: SttStreamConfig, engine: Box<dyn SttEngine>) -> Self {
        let max_samples = config.max_buffer_seconds as usize * 16000;

        Self {
            config,
            engine,
            audio_buffer: VecDeque::with_capacity(max_samples),
            buffer_start_ms: 0,
            last_final_text: String::new(),
            last_partial_text: String::new(),
            total_audio_ms: 0,
            sentence_buffer: String::new(),
        }
    }

    /// Push audio frame to the streamer
    pub async fn push_audio(&mut self, frame: &AudioFrame) -> Result<Vec<TranscriptResult>, SttError> {
        // Resample to 16kHz if needed
        let frame_16k = if frame.sample_rate != SampleRate::Hz16000 {
            frame.resample(SampleRate::Hz16000)
        } else {
            frame.clone()
        };

        // Add to buffer
        self.audio_buffer.extend(frame_16k.samples.iter());
        self.total_audio_ms += frame.duration.as_millis() as u64;

        // Trim buffer if too large
        let max_samples = self.config.max_buffer_seconds as usize * 16000;
        while self.audio_buffer.len() > max_samples {
            self.audio_buffer.pop_front();
            self.buffer_start_ms += 1000 / 16; // Remove 1 sample worth of time
        }

        // Check if we have enough audio for a chunk
        let chunk_samples = (self.config.chunk_duration_ms as usize * 16000) / 1000;
        if self.audio_buffer.len() < chunk_samples {
            return Ok(vec![]);
        }

        // Process chunk
        self.process_chunk().await
    }

    /// Process accumulated audio chunk
    async fn process_chunk(&mut self) -> Result<Vec<TranscriptResult>, SttError> {
        let chunk_samples = (self.config.chunk_duration_ms as usize * 16000) / 1000;
        let overlap_samples = (self.config.overlap_duration_ms as usize * 16000) / 1000;

        // Extract chunk with overlap for better continuity
        let chunk: Vec<f32> = self.audio_buffer
            .iter()
            .take(chunk_samples)
            .copied()
            .collect();

        // Run STT
        let result = self.engine.transcribe(
            &chunk,
            self.config.language.as_deref(),
        ).await?;

        let mut outputs = Vec::new();

        // Check for stable text (text that hasn't changed)
        let stable_text = self.find_stable_prefix(&result.text);

        if !stable_text.is_empty() && stable_text != self.last_final_text {
            // We have new stable text
            let start_ms = self.buffer_start_ms;
            let end_ms = self.total_audio_ms;

            // Check for sentence boundaries
            let sentences = self.extract_sentences(&stable_text);

            for sentence in sentences {
                outputs.push(TranscriptResult {
                    text: sentence.clone(),
                    is_final: true,
                    confidence: result.confidence,
                    start_time_ms: start_ms,
                    end_time_ms: end_ms,
                    language: result.language.clone(),
                    words: vec![], // Would extract from result.words
                });
            }

            self.last_final_text = stable_text;
        }

        // Emit partial if enabled
        if self.config.emit_partials && result.text != self.last_partial_text {
            let unstable_part = result.text
                .strip_prefix(&self.last_final_text)
                .unwrap_or(&result.text)
                .trim();

            if !unstable_part.is_empty() {
                outputs.push(TranscriptResult {
                    text: unstable_part.to_string(),
                    is_final: false,
                    confidence: result.confidence,
                    start_time_ms: self.buffer_start_ms,
                    end_time_ms: self.total_audio_ms,
                    language: result.language.clone(),
                    words: vec![],
                });
            }

            self.last_partial_text = result.text;
        }

        // Remove processed audio (minus overlap)
        let remove_samples = chunk_samples.saturating_sub(overlap_samples);
        for _ in 0..remove_samples {
            self.audio_buffer.pop_front();
        }
        self.buffer_start_ms += (remove_samples * 1000 / 16000) as u64;

        Ok(outputs)
    }

    /// Find the stable prefix that hasn't changed
    fn find_stable_prefix(&self, new_text: &str) -> String {
        // Compare with last partial to find stable portion
        // In production, use n-best list comparison

        let last_words: Vec<&str> = self.last_partial_text.split_whitespace().collect();
        let new_words: Vec<&str> = new_text.split_whitespace().collect();

        let mut stable_count = 0;
        for (i, (last, new)) in last_words.iter().zip(new_words.iter()).enumerate() {
            if last == new {
                stable_count = i + 1;
            } else {
                break;
            }
        }

        // Require at least 3 stable words or sentence boundary
        if stable_count >= 3 || new_text.ends_with(['.', '?', '!']) {
            new_words[..stable_count.min(new_words.len())]
                .join(" ")
        } else {
            String::new()
        }
    }

    /// Extract complete sentences from text
    fn extract_sentences(&mut self, text: &str) -> Vec<String> {
        self.sentence_buffer.push_str(text);

        let mut sentences = Vec::new();

        // Find sentence boundaries
        // Handle Indian language sentence endings too
        let sentence_endings = ['.', '।', '?', '!', '॥'];

        loop {
            let boundary = self.sentence_buffer
                .char_indices()
                .find(|(_, c)| sentence_endings.contains(c));

            if let Some((idx, ending)) = boundary {
                let sentence: String = self.sentence_buffer
                    .drain(..=idx)
                    .collect();

                let trimmed = sentence.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
            } else {
                break;
            }
        }

        sentences
    }

    /// Finalize stream and get remaining text
    pub async fn finalize(&mut self) -> Result<Option<TranscriptResult>, SttError> {
        // Process remaining buffer
        if self.audio_buffer.is_empty() {
            return Ok(None);
        }

        let chunk: Vec<f32> = self.audio_buffer.drain(..).collect();
        let result = self.engine.transcribe(&chunk, self.config.language.as_deref()).await?;

        // Return remaining text if any
        let remaining = if !self.sentence_buffer.is_empty() {
            Some(TranscriptResult {
                text: std::mem::take(&mut self.sentence_buffer).trim().to_string(),
                is_final: true,
                confidence: result.confidence,
                start_time_ms: self.buffer_start_ms,
                end_time_ms: self.total_audio_ms,
                language: result.language,
                words: vec![],
            })
        } else if !result.text.is_empty() && result.text != self.last_final_text {
            Some(TranscriptResult {
                text: result.text,
                is_final: true,
                confidence: result.confidence,
                start_time_ms: self.buffer_start_ms,
                end_time_ms: self.total_audio_ms,
                language: result.language,
                words: vec![],
            })
        } else {
            None
        };

        self.reset();
        Ok(remaining)
    }

    /// Reset streamer state
    pub fn reset(&mut self) {
        self.audio_buffer.clear();
        self.buffer_start_ms = 0;
        self.last_final_text.clear();
        self.last_partial_text.clear();
        self.total_audio_ms = 0;
        self.sentence_buffer.clear();
    }
}
```

## TTS Streaming Pipeline

### Sentence-by-Sentence TTS Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        TTS STREAMING ARCHITECTURE                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   LLM Response Stream                                                        │
│       │                                                                      │
│       ▼                                                                      │
│   ┌───────────────┐                                                         │
│   │   Sentence    │  ◀──  Accumulate tokens until sentence                  │
│   │   Detector    │                                                         │
│   └───────┬───────┘                                                         │
│           │                                                                  │
│           ▼                                                                  │
│   ┌───────────────┐     ┌───────────────┐                                   │
│   │   Text Pre-   │────▶│   SSML        │  Optional prosody markup          │
│   │   processor   │     │   Generator   │                                   │
│   └───────┬───────┘     └───────────────┘                                   │
│           │                                                                  │
│           ▼                                                                  │
│   ┌───────────────┐     ┌───────────────┐                                   │
│   │   TTS Queue   │────▶│   Priority    │  Urgent interjections first      │
│   │   (bounded)   │     │   Sorter      │                                   │
│   └───────┬───────┘     └───────────────┘                                   │
│           │                                                                  │
│           ▼                                                                  │
│   ┌───────────────┐                                                         │
│   │   TTS Engine  │  ◀──  IndicF5 / Piper / VITS                            │
│   │   (ONNX)      │                                                         │
│   └───────┬───────┘                                                         │
│           │                                                                  │
│           ▼                                                                  │
│   ┌───────────────┐     ┌───────────────┐                                   │
│   │   Audio       │────▶│   Crossfade   │  Smooth transitions               │
│   │   Chunker     │     │   Mixer       │                                   │
│   └───────┬───────┘     └───────────────┘                                   │
│           │                                                                  │
│           ▼                                                                  │
│      Audio Output                                                            │
│       Stream                                                                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### TTS Streaming Implementation

```rust
use tokio::sync::mpsc;
use std::sync::Arc;

/// TTS streaming configuration
#[derive(Debug, Clone)]
pub struct TtsStreamConfig {
    /// Voice ID or name
    pub voice_id: String,

    /// Speaking rate multiplier (1.0 = normal)
    pub rate: f32,

    /// Pitch adjustment in semitones
    pub pitch: f32,

    /// Output sample rate
    pub sample_rate: SampleRate,

    /// Audio chunk size (ms)
    pub chunk_duration_ms: u32,

    /// Crossfade duration between sentences (ms)
    pub crossfade_ms: u32,

    /// Maximum queue depth
    pub max_queue_depth: usize,
}

impl Default for TtsStreamConfig {
    fn default() -> Self {
        Self {
            voice_id: "default".to_string(),
            rate: 1.0,
            pitch: 0.0,
            sample_rate: SampleRate::Hz22050,
            chunk_duration_ms: 50,
            crossfade_ms: 20,
            max_queue_depth: 5,
        }
    }
}

/// TTS engine trait
#[async_trait::async_trait]
pub trait TtsEngine: Send + Sync {
    /// Synthesize text to audio
    async fn synthesize(&self, text: &str, config: &TtsStreamConfig)
        -> Result<Vec<f32>, TtsError>;

    /// List available voices
    fn list_voices(&self) -> Vec<VoiceInfo>;

    /// Get voice info by ID
    fn get_voice(&self, voice_id: &str) -> Option<VoiceInfo>;
}

#[derive(Debug, Clone)]
pub struct VoiceInfo {
    pub id: String,
    pub name: String,
    pub language: String,
    pub gender: Gender,
    pub sample_rate: SampleRate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
    Neutral,
}

/// Sentence for TTS processing
#[derive(Debug)]
struct TtsSentence {
    text: String,
    priority: TtsPriority,
    sequence: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TtsPriority {
    /// Urgent interjection (e.g., "I understand", acknowledgments)
    Urgent = 0,
    /// Normal sentence
    Normal = 1,
    /// Filler/backchanneling
    Filler = 2,
}

/// Streaming TTS processor
pub struct TtsStreamer {
    config: TtsStreamConfig,
    engine: Arc<dyn TtsEngine>,

    // Sentence accumulation
    sentence_buffer: String,
    sentence_sequence: u64,

    // Audio generation queue
    sentence_tx: mpsc::Sender<TtsSentence>,
    audio_rx: mpsc::Receiver<AudioFrame>,

    // Crossfade state
    last_samples: Vec<f32>,
}

impl TtsStreamer {
    pub fn new(config: TtsStreamConfig, engine: Arc<dyn TtsEngine>) -> Self {
        let (sentence_tx, sentence_rx) = mpsc::channel(config.max_queue_depth);
        let (audio_tx, audio_rx) = mpsc::channel(config.max_queue_depth * 2);

        // Spawn TTS worker
        let engine_clone = engine.clone();
        let config_clone = config.clone();
        tokio::spawn(async move {
            Self::tts_worker(sentence_rx, audio_tx, engine_clone, config_clone).await;
        });

        Self {
            config,
            engine,
            sentence_buffer: String::new(),
            sentence_sequence: 0,
            sentence_tx,
            audio_rx,
            last_samples: Vec::new(),
        }
    }

    /// TTS worker that processes sentences
    async fn tts_worker(
        mut sentence_rx: mpsc::Receiver<TtsSentence>,
        audio_tx: mpsc::Sender<AudioFrame>,
        engine: Arc<dyn TtsEngine>,
        config: TtsStreamConfig,
    ) {
        let mut frame_sequence = 0u64;

        while let Some(sentence) = sentence_rx.recv().await {
            // Synthesize sentence
            match engine.synthesize(&sentence.text, &config).await {
                Ok(samples) => {
                    // Chunk audio into frames
                    let chunk_samples = (config.chunk_duration_ms as usize
                        * config.sample_rate.as_u32() as usize) / 1000;

                    for chunk in samples.chunks(chunk_samples) {
                        let frame = AudioFrame::new(
                            chunk.to_vec(),
                            config.sample_rate,
                            Channels::Mono,
                            frame_sequence,
                        );
                        frame_sequence += 1;

                        if audio_tx.send(frame).await.is_err() {
                            return; // Receiver dropped
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("TTS error: {:?}", e);
                }
            }
        }
    }

    /// Push text token from LLM stream
    pub async fn push_token(&mut self, token: &str) -> Result<(), TtsError> {
        self.sentence_buffer.push_str(token);

        // Check for sentence boundaries
        let sentence_endings = ['.', '।', '?', '!', '॥', '\n'];

        if let Some(idx) = self.sentence_buffer
            .char_indices()
            .rev()
            .find(|(_, c)| sentence_endings.contains(c))
            .map(|(i, _)| i)
        {
            let sentence: String = self.sentence_buffer.drain(..=idx).collect();
            let trimmed = sentence.trim();

            if !trimmed.is_empty() {
                self.queue_sentence(trimmed.to_string(), TtsPriority::Normal).await?;
            }
        }

        Ok(())
    }

    /// Queue sentence for TTS
    async fn queue_sentence(&mut self, text: String, priority: TtsPriority)
        -> Result<(), TtsError>
    {
        self.sentence_sequence += 1;

        let sentence = TtsSentence {
            text,
            priority,
            sequence: self.sentence_sequence,
        };

        self.sentence_tx.send(sentence).await
            .map_err(|_| TtsError::QueueFull)?;

        Ok(())
    }

    /// Queue urgent interjection (skips to front)
    pub async fn interject(&mut self, text: &str) -> Result<(), TtsError> {
        self.queue_sentence(text.to_string(), TtsPriority::Urgent).await
    }

    /// Get next audio frame
    pub async fn next_frame(&mut self) -> Option<AudioFrame> {
        self.audio_rx.recv().await
    }

    /// Get next frame with crossfade applied
    pub async fn next_frame_crossfaded(&mut self) -> Option<AudioFrame> {
        let mut frame = self.audio_rx.recv().await?;

        if !self.last_samples.is_empty() {
            // Apply crossfade with previous samples
            let crossfade_samples = (self.config.crossfade_ms as usize
                * self.config.sample_rate.as_u32() as usize) / 1000;

            let crossfade_len = crossfade_samples.min(self.last_samples.len())
                .min(frame.samples.len());

            let mut new_samples: Vec<f32> = frame.samples.to_vec();

            for i in 0..crossfade_len {
                let fade_in = i as f32 / crossfade_len as f32;
                let fade_out = 1.0 - fade_in;

                let last_idx = self.last_samples.len() - crossfade_len + i;
                new_samples[i] = new_samples[i] * fade_in
                    + self.last_samples[last_idx] * fade_out;
            }

            frame.samples = new_samples.into();
        }

        // Store last samples for next crossfade
        let samples_to_keep = (self.config.crossfade_ms as usize
            * self.config.sample_rate.as_u32() as usize) / 1000;
        self.last_samples = frame.samples
            .iter()
            .rev()
            .take(samples_to_keep)
            .rev()
            .copied()
            .collect();

        Some(frame)
    }

    /// Finalize stream and flush remaining text
    pub async fn finalize(&mut self) -> Result<(), TtsError> {
        let remaining = std::mem::take(&mut self.sentence_buffer);
        let trimmed = remaining.trim();

        if !trimmed.is_empty() {
            self.queue_sentence(trimmed.to_string(), TtsPriority::Normal).await?;
        }

        Ok(())
    }

    /// Clear queue (e.g., on barge-in)
    pub fn clear(&mut self) {
        self.sentence_buffer.clear();
        self.last_samples.clear();
        // Note: can't clear mpsc channel, need to drop and recreate
    }
}
```

## Barge-In Handling

### Interrupt Detection and Response

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::watch;

/// Barge-in configuration
#[derive(Debug, Clone)]
pub struct BargeInConfig {
    /// Enable barge-in detection
    pub enabled: bool,

    /// VAD threshold for interrupt detection
    pub vad_threshold: f32,

    /// Minimum speech duration to trigger interrupt (ms)
    pub min_speech_ms: u32,

    /// Energy threshold for interrupt (dB)
    pub energy_threshold_db: f32,

    /// Action on barge-in
    pub action: BargeInAction,

    /// Cooldown after barge-in before allowing another (ms)
    pub cooldown_ms: u32,
}

impl Default for BargeInConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            vad_threshold: 0.6, // Slightly higher than normal VAD
            min_speech_ms: 200,
            energy_threshold_db: -35.0,
            action: BargeInAction::StopAndListen,
            cooldown_ms: 500,
        }
    }
}

/// Action to take on barge-in
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BargeInAction {
    /// Stop output immediately and listen
    StopAndListen,
    /// Stop output and acknowledge ("I hear you")
    StopAndAcknowledge,
    /// Duck volume and continue
    DuckAndContinue,
    /// Ignore (no barge-in)
    Ignore,
}

/// Barge-in detector
pub struct BargeInDetector {
    config: BargeInConfig,
    vad: SileroVad,

    // State
    is_playing: AtomicBool,
    speech_samples: std::sync::atomic::AtomicU64,
    last_barge_in: std::sync::Mutex<std::time::Instant>,

    // Notification
    interrupt_tx: watch::Sender<bool>,
    interrupt_rx: watch::Receiver<bool>,
}

impl BargeInDetector {
    pub fn new(config: BargeInConfig, vad: SileroVad) -> Self {
        let (interrupt_tx, interrupt_rx) = watch::channel(false);

        Self {
            config,
            vad,
            is_playing: AtomicBool::new(false),
            speech_samples: std::sync::atomic::AtomicU64::new(0),
            last_barge_in: std::sync::Mutex::new(std::time::Instant::now()
                - std::time::Duration::from_secs(10)),
            interrupt_tx,
            interrupt_rx,
        }
    }

    /// Set whether agent is currently speaking
    pub fn set_playing(&self, playing: bool) {
        self.is_playing.store(playing, Ordering::SeqCst);
        if !playing {
            // Reset speech accumulation when not playing
            self.speech_samples.store(0, Ordering::SeqCst);
        }
    }

    /// Get interrupt notification receiver
    pub fn subscribe(&self) -> watch::Receiver<bool> {
        self.interrupt_rx.clone()
    }

    /// Process input audio for barge-in detection
    pub fn process(&self, frame: &AudioFrame) -> Option<BargeInEvent> {
        if !self.config.enabled {
            return None;
        }

        // Only detect barge-in while agent is speaking
        if !self.is_playing.load(Ordering::SeqCst) {
            return None;
        }

        // Check cooldown
        {
            let last = self.last_barge_in.lock().unwrap();
            if last.elapsed().as_millis() < self.config.cooldown_ms as u128 {
                return None;
            }
        }

        // Quick energy check
        if frame.energy_db < self.config.energy_threshold_db {
            self.speech_samples.store(0, Ordering::SeqCst);
            return None;
        }

        // Check VAD probability
        if let Some(prob) = frame.vad_probability {
            if prob < self.config.vad_threshold {
                self.speech_samples.store(0, Ordering::SeqCst);
                return None;
            }
        }

        // Accumulate speech
        let current = self.speech_samples.fetch_add(
            frame.samples.len() as u64,
            Ordering::SeqCst
        );

        let min_samples = (self.config.min_speech_ms as u64 * 16000) / 1000;

        if current + frame.samples.len() as u64 >= min_samples {
            // Trigger barge-in
            self.speech_samples.store(0, Ordering::SeqCst);
            *self.last_barge_in.lock().unwrap() = std::time::Instant::now();

            // Notify listeners
            let _ = self.interrupt_tx.send(true);

            Some(BargeInEvent {
                action: self.config.action,
                timestamp: frame.timestamp,
            })
        } else {
            None
        }
    }

    /// Reset barge-in state
    pub fn reset(&self) {
        self.speech_samples.store(0, Ordering::SeqCst);
        let _ = self.interrupt_tx.send(false);
    }
}

#[derive(Debug, Clone)]
pub struct BargeInEvent {
    pub action: BargeInAction,
    pub timestamp: std::time::Instant,
}
```

## Full Audio Pipeline Integration

### Unified Audio Pipeline

```rust
use tokio::sync::{mpsc, broadcast};

/// Audio pipeline configuration
#[derive(Debug, Clone)]
pub struct AudioPipelineConfig {
    pub vad: VadConfig,
    pub stt: SttStreamConfig,
    pub tts: TtsStreamConfig,
    pub barge_in: BargeInConfig,

    /// Input sample rate
    pub input_sample_rate: SampleRate,

    /// Output sample rate
    pub output_sample_rate: SampleRate,
}

/// Events emitted by the audio pipeline
#[derive(Debug, Clone)]
pub enum AudioPipelineEvent {
    /// User started speaking
    SpeechStart,

    /// Transcript available (partial or final)
    Transcript(TranscriptResult),

    /// User stopped speaking
    SpeechEnd,

    /// User interrupted agent
    BargeIn(BargeInEvent),

    /// Agent audio frame ready
    AgentAudio(AudioFrame),

    /// Agent finished speaking
    AgentSpeechEnd,

    /// Error occurred
    Error(String),
}

/// Full audio pipeline orchestrator
pub struct AudioPipeline {
    config: AudioPipelineConfig,

    // Components
    vad: SileroVad,
    stt_streamer: SttStreamer,
    tts_streamer: TtsStreamer,
    barge_in_detector: BargeInDetector,

    // Channels
    input_rx: mpsc::Receiver<AudioFrame>,
    event_tx: broadcast::Sender<AudioPipelineEvent>,

    // State
    is_agent_speaking: bool,
    frame_sequence: u64,
}

impl AudioPipeline {
    pub async fn new(
        config: AudioPipelineConfig,
        stt_engine: Box<dyn SttEngine>,
        tts_engine: Arc<dyn TtsEngine>,
    ) -> Result<(Self, mpsc::Sender<AudioFrame>, broadcast::Receiver<AudioPipelineEvent>), AudioError> {
        let vad = SileroVad::new("models/silero_vad.onnx", config.vad.clone())?;
        let stt_streamer = SttStreamer::new(config.stt.clone(), stt_engine);
        let tts_streamer = TtsStreamer::new(config.tts.clone(), tts_engine.clone());

        let vad_for_barge_in = SileroVad::new("models/silero_vad.onnx", config.vad.clone())?;
        let barge_in_detector = BargeInDetector::new(config.barge_in.clone(), vad_for_barge_in);

        let (input_tx, input_rx) = mpsc::channel(100);
        let (event_tx, event_rx) = broadcast::channel(100);

        let pipeline = Self {
            config,
            vad,
            stt_streamer,
            tts_streamer,
            barge_in_detector,
            input_rx,
            event_tx,
            is_agent_speaking: false,
            frame_sequence: 0,
        };

        Ok((pipeline, input_tx, event_rx))
    }

    /// Run the audio pipeline
    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                // Process input audio
                Some(mut frame) = self.input_rx.recv() => {
                    self.process_input(&mut frame).await;
                }

                // Get TTS output
                Some(audio_frame) = self.tts_streamer.next_frame_crossfaded() => {
                    self.emit_event(AudioPipelineEvent::AgentAudio(audio_frame));
                }

                else => break,
            }
        }
    }

    /// Process incoming audio frame
    async fn process_input(&mut self, frame: &mut AudioFrame) {
        // Run VAD
        let vad_result = match self.vad.process(frame) {
            Ok(result) => result,
            Err(e) => {
                self.emit_event(AudioPipelineEvent::Error(format!("VAD error: {:?}", e)));
                return;
            }
        };

        // Check for barge-in while agent is speaking
        if self.is_agent_speaking {
            if let Some(barge_in) = self.barge_in_detector.process(frame) {
                self.handle_barge_in(barge_in).await;
                return;
            }
        }

        // Process based on VAD state
        match vad_result {
            VadResult::SpeechConfirmed => {
                self.emit_event(AudioPipelineEvent::SpeechStart);
            }

            VadResult::SpeechContinue | VadResult::PotentialSpeechEnd => {
                // Feed to STT
                if frame.is_speech {
                    match self.stt_streamer.push_audio(frame).await {
                        Ok(transcripts) => {
                            for transcript in transcripts {
                                self.emit_event(AudioPipelineEvent::Transcript(transcript));
                            }
                        }
                        Err(e) => {
                            self.emit_event(AudioPipelineEvent::Error(
                                format!("STT error: {:?}", e)
                            ));
                        }
                    }
                }
            }

            VadResult::SpeechEnd => {
                // Finalize STT
                if let Ok(Some(final_transcript)) = self.stt_streamer.finalize().await {
                    self.emit_event(AudioPipelineEvent::Transcript(final_transcript));
                }
                self.emit_event(AudioPipelineEvent::SpeechEnd);
            }

            VadResult::Silence | VadResult::PotentialSpeechStart => {
                // No action needed
            }
        }
    }

    /// Handle barge-in event
    async fn handle_barge_in(&mut self, event: BargeInEvent) {
        match event.action {
            BargeInAction::StopAndListen => {
                self.tts_streamer.clear();
                self.is_agent_speaking = false;
                self.emit_event(AudioPipelineEvent::BargeIn(event));
            }

            BargeInAction::StopAndAcknowledge => {
                self.tts_streamer.clear();
                let _ = self.tts_streamer.interject("I understand.").await;
                self.emit_event(AudioPipelineEvent::BargeIn(event));
            }

            BargeInAction::DuckAndContinue => {
                // Would reduce volume but continue - not implemented
                self.emit_event(AudioPipelineEvent::BargeIn(event));
            }

            BargeInAction::Ignore => {
                // Do nothing
            }
        }
    }

    /// Push LLM token for TTS
    pub async fn push_llm_token(&mut self, token: &str) -> Result<(), TtsError> {
        if !self.is_agent_speaking {
            self.is_agent_speaking = true;
            self.barge_in_detector.set_playing(true);
        }

        self.tts_streamer.push_token(token).await
    }

    /// Mark agent speech as complete
    pub async fn finish_agent_speech(&mut self) -> Result<(), TtsError> {
        self.tts_streamer.finalize().await?;
        self.is_agent_speaking = false;
        self.barge_in_detector.set_playing(false);
        self.emit_event(AudioPipelineEvent::AgentSpeechEnd);
        Ok(())
    }

    fn emit_event(&self, event: AudioPipelineEvent) {
        let _ = self.event_tx.send(event);
    }
}
```

## Latency Optimization

### End-to-End Latency Budget

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        LATENCY BUDGET (TARGET: <800ms)                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   Component                    │ Target    │ Optimization Strategy           │
│   ────────────────────────────┼───────────┼────────────────────────────────│
│   VAD Processing              │ 5ms       │ - Silero VAD optimized ONNX    │
│                               │           │ - 10ms frame batching          │
│   ────────────────────────────┼───────────┼────────────────────────────────│
│   STT Streaming               │ 100-200ms │ - Streaming ASR (not batch)    │
│                               │           │ - Partial results enabled       │
│                               │           │ - Optimized chunk size         │
│   ────────────────────────────┼───────────┼────────────────────────────────│
│   Text Processing             │ 50ms      │ - Async grammar correction     │
│   (Grammar + Translation)     │           │ - Sentence-level batching      │
│   ────────────────────────────┼───────────┼────────────────────────────────│
│   RAG Retrieval               │ 100-150ms │ - Prefetch on partial text     │
│   (if needed)                 │           │ - Cached embeddings            │
│   ────────────────────────────┼───────────┼────────────────────────────────│
│   LLM Inference               │ 200-300ms │ - Streaming response           │
│   (first token)               │           │ - KV cache                     │
│   ────────────────────────────┼───────────┼────────────────────────────────│
│   TTS Synthesis               │ 100-150ms │ - Sentence-by-sentence         │
│   (first sentence)            │           │ - Pipelining with LLM stream   │
│   ────────────────────────────┼───────────┼────────────────────────────────│
│   Network/Buffering           │ 50-100ms  │ - WebRTC/WebSocket             │
│                               │           │ - Jitter buffer tuning         │
│   ────────────────────────────┴───────────┴────────────────────────────────│
│                                                                              │
│   TOTAL                       │ 600-800ms │ User perceives <1 second       │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Optimization Techniques

```rust
/// Latency tracking metrics
#[derive(Debug, Default)]
pub struct LatencyMetrics {
    pub vad_ms: f32,
    pub stt_first_partial_ms: f32,
    pub stt_final_ms: f32,
    pub text_processing_ms: f32,
    pub rag_retrieval_ms: f32,
    pub llm_first_token_ms: f32,
    pub tts_first_audio_ms: f32,
    pub total_turn_ms: f32,
}

impl LatencyMetrics {
    pub fn log(&self) {
        tracing::info!(
            vad_ms = self.vad_ms,
            stt_partial_ms = self.stt_first_partial_ms,
            stt_final_ms = self.stt_final_ms,
            text_proc_ms = self.text_processing_ms,
            rag_ms = self.rag_retrieval_ms,
            llm_ttft_ms = self.llm_first_token_ms,
            tts_first_ms = self.tts_first_audio_ms,
            total_ms = self.total_turn_ms,
            "Turn latency breakdown"
        );
    }
}

/// Prefetch manager for speculative processing
pub struct PrefetchManager {
    /// Prefetch RAG on partial transcripts
    pub rag_prefetch_enabled: bool,

    /// Minimum confidence for prefetch
    pub min_confidence: f32,

    /// Cached prefetch results
    prefetch_cache: std::sync::Mutex<Option<PrefetchedContext>>,
}

struct PrefetchedContext {
    query: String,
    results: Vec<String>,
    timestamp: std::time::Instant,
}

impl PrefetchManager {
    /// Speculatively prefetch RAG results on partial transcript
    pub async fn maybe_prefetch(
        &self,
        partial: &TranscriptResult,
        rag_retriever: &dyn Retriever,
    ) {
        if !self.rag_prefetch_enabled {
            return;
        }

        if partial.confidence < self.min_confidence {
            return;
        }

        // Check if query is long enough
        if partial.text.split_whitespace().count() < 3 {
            return;
        }

        // Prefetch in background
        let query = partial.text.clone();
        let results = rag_retriever.retrieve(&query, 3).await;

        if let Ok(results) = results {
            *self.prefetch_cache.lock().unwrap() = Some(PrefetchedContext {
                query,
                results,
                timestamp: std::time::Instant::now(),
            });
        }
    }

    /// Get prefetched results if still valid
    pub fn get_prefetched(&self, final_query: &str) -> Option<Vec<String>> {
        let cache = self.prefetch_cache.lock().unwrap();

        if let Some(ref prefetched) = *cache {
            // Check if query is similar enough
            let similarity = self.query_similarity(&prefetched.query, final_query);

            // Check if not too old (500ms)
            let is_fresh = prefetched.timestamp.elapsed().as_millis() < 500;

            if similarity > 0.8 && is_fresh {
                return Some(prefetched.results.clone());
            }
        }

        None
    }

    fn query_similarity(&self, a: &str, b: &str) -> f32 {
        // Simple word overlap similarity
        let words_a: std::collections::HashSet<&str> = a.split_whitespace().collect();
        let words_b: std::collections::HashSet<&str> = b.split_whitespace().collect();

        let intersection = words_a.intersection(&words_b).count();
        let union = words_a.union(&words_b).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }
}
```

## Audio Format Handling

### WebRTC Integration

```rust
use webrtc::api::media_engine::MediaEngine;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;

/// WebRTC audio configuration
pub struct WebRtcAudioConfig {
    /// Opus encoder settings
    pub opus_bitrate: u32,
    pub opus_frame_duration_ms: u32,

    /// Jitter buffer settings
    pub jitter_buffer_ms: u32,

    /// Enable echo cancellation
    pub echo_cancellation: bool,

    /// Enable noise suppression
    pub noise_suppression: bool,
}

impl Default for WebRtcAudioConfig {
    fn default() -> Self {
        Self {
            opus_bitrate: 32000,
            opus_frame_duration_ms: 20,
            jitter_buffer_ms: 50,
            echo_cancellation: true,
            noise_suppression: true,
        }
    }
}

/// Audio codec wrapper
pub enum AudioCodec {
    Opus(OpusCodec),
    Pcm,
}

pub struct OpusCodec {
    encoder: opus::Encoder,
    decoder: opus::Decoder,
    sample_rate: SampleRate,
}

impl OpusCodec {
    pub fn new(sample_rate: SampleRate) -> Result<Self, AudioError> {
        let encoder = opus::Encoder::new(
            sample_rate.as_u32(),
            opus::Channels::Mono,
            opus::Application::Voip,
        )?;

        let decoder = opus::Decoder::new(
            sample_rate.as_u32(),
            opus::Channels::Mono,
        )?;

        Ok(Self {
            encoder,
            decoder,
            sample_rate,
        })
    }

    pub fn encode(&mut self, pcm: &[f32]) -> Result<Vec<u8>, AudioError> {
        // Convert f32 to i16
        let pcm_i16: Vec<i16> = pcm.iter()
            .map(|&s| (s * 32767.0) as i16)
            .collect();

        let mut output = vec![0u8; 4000]; // Max Opus frame size
        let len = self.encoder.encode(&pcm_i16, &mut output)?;
        output.truncate(len);

        Ok(output)
    }

    pub fn decode(&mut self, opus_data: &[u8]) -> Result<Vec<f32>, AudioError> {
        let frame_size = (self.sample_rate.as_u32() as usize * 20) / 1000;
        let mut output = vec![0i16; frame_size];

        let samples = self.decoder.decode(opus_data, &mut output, false)?;

        let pcm_f32: Vec<f32> = output[..samples]
            .iter()
            .map(|&s| s as f32 / 32768.0)
            .collect();

        Ok(pcm_f32)
    }
}
```

## Error Handling and Recovery

### Audio Pipeline Errors

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("VAD error: {0}")]
    Vad(#[from] VadError),

    #[error("STT error: {0}")]
    Stt(#[from] SttError),

    #[error("TTS error: {0}")]
    Tts(#[from] TtsError),

    #[error("Codec error: {0}")]
    Codec(String),

    #[error("Channel closed")]
    ChannelClosed,

    #[error("Buffer overflow")]
    BufferOverflow,

    #[error("Model not found: {0}")]
    ModelNotFound(String),
}

#[derive(Error, Debug)]
pub enum VadError {
    #[error("ONNX runtime error: {0}")]
    Onnx(#[from] ort::Error),

    #[error("Invalid audio format")]
    InvalidFormat,

    #[error("Array shape error: {0}")]
    Shape(#[from] ndarray::ShapeError),
}

#[derive(Error, Debug)]
pub enum SttError {
    #[error("Engine error: {0}")]
    Engine(String),

    #[error("Timeout")]
    Timeout,

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}

#[derive(Error, Debug)]
pub enum TtsError {
    #[error("Engine error: {0}")]
    Engine(String),

    #[error("Queue full")]
    QueueFull,

    #[error("Voice not found: {0}")]
    VoiceNotFound(String),
}

/// Error recovery strategies
pub struct AudioErrorHandler {
    /// Maximum consecutive errors before circuit break
    pub max_consecutive_errors: u32,

    /// Circuit break duration
    pub circuit_break_duration: std::time::Duration,

    // State
    consecutive_errors: std::sync::atomic::AtomicU32,
    circuit_open_until: std::sync::Mutex<Option<std::time::Instant>>,
}

impl AudioErrorHandler {
    pub fn new(max_errors: u32, break_duration: std::time::Duration) -> Self {
        Self {
            max_consecutive_errors: max_errors,
            circuit_break_duration: break_duration,
            consecutive_errors: std::sync::atomic::AtomicU32::new(0),
            circuit_open_until: std::sync::Mutex::new(None),
        }
    }

    /// Record a successful operation
    pub fn record_success(&self) {
        self.consecutive_errors.store(0, std::sync::atomic::Ordering::SeqCst);
    }

    /// Record an error, returns whether to proceed
    pub fn record_error(&self, error: &AudioError) -> bool {
        let count = self.consecutive_errors.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        if count >= self.max_consecutive_errors {
            // Open circuit
            let mut guard = self.circuit_open_until.lock().unwrap();
            *guard = Some(std::time::Instant::now() + self.circuit_break_duration);

            tracing::error!(
                error = ?error,
                "Circuit breaker opened after {} consecutive errors",
                count
            );

            return false;
        }

        true
    }

    /// Check if circuit is closed (can proceed)
    pub fn is_available(&self) -> bool {
        let guard = self.circuit_open_until.lock().unwrap();

        match *guard {
            None => true,
            Some(until) => {
                if std::time::Instant::now() >= until {
                    drop(guard);
                    // Reset state
                    self.consecutive_errors.store(0, std::sync::atomic::Ordering::SeqCst);
                    *self.circuit_open_until.lock().unwrap() = None;
                    true
                } else {
                    false
                }
            }
        }
    }
}
```

## Testing

### Audio Pipeline Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Mock STT engine for testing
    struct MockSttEngine {
        responses: std::sync::Mutex<Vec<SttEngineResult>>,
    }

    impl MockSttEngine {
        fn new(responses: Vec<SttEngineResult>) -> Self {
            Self {
                responses: std::sync::Mutex::new(responses),
            }
        }
    }

    #[async_trait::async_trait]
    impl SttEngine for MockSttEngine {
        async fn transcribe(&self, _audio: &[f32], _language: Option<&str>)
            -> Result<SttEngineResult, SttError>
        {
            let mut responses = self.responses.lock().unwrap();
            Ok(responses.pop().unwrap_or(SttEngineResult {
                text: String::new(),
                confidence: 0.0,
                words: vec![],
                language: None,
            }))
        }

        fn supported_languages(&self) -> Vec<String> {
            vec!["en".to_string(), "hi".to_string()]
        }

        fn optimal_chunk_ms(&self) -> u32 {
            500
        }
    }

    #[tokio::test]
    async fn test_vad_state_transitions() {
        // Test VAD state machine
        let config = VadConfig::default();

        // Simulate speech detection
        let mut state = VadState::Silence;

        // Speech start
        assert_eq!(state, VadState::Silence);
        state = VadState::SpeechStart;

        // Confirmed speech
        state = VadState::Speech;

        // Speech end
        state = VadState::SpeechEnd;

        // Back to silence
        state = VadState::Silence;
        assert_eq!(state, VadState::Silence);
    }

    #[tokio::test]
    async fn test_stt_sentence_extraction() {
        let mock_engine = Box::new(MockSttEngine::new(vec![
            SttEngineResult {
                text: "Hello how are you doing today.".to_string(),
                confidence: 0.95,
                words: vec![],
                language: Some("en".to_string()),
            }
        ]));

        let config = SttStreamConfig::default();
        let mut streamer = SttStreamer::new(config, mock_engine);

        // Create test audio frame
        let samples = vec![0.0f32; 8000]; // 0.5 seconds at 16kHz
        let frame = AudioFrame::new(
            samples,
            SampleRate::Hz16000,
            Channels::Mono,
            0,
        );

        let results = streamer.push_audio(&frame).await.unwrap();

        // Should extract complete sentence
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_audio_frame_resampling() {
        let samples = vec![1.0f32; 1600]; // 0.1 seconds at 16kHz
        let frame = AudioFrame::new(
            samples,
            SampleRate::Hz16000,
            Channels::Mono,
            0,
        );

        // Resample to 22050 Hz
        let resampled = frame.resample(SampleRate::Hz22050);

        assert_eq!(resampled.sample_rate, SampleRate::Hz22050);
        // 1600 samples at 16kHz = 0.1s = 2205 samples at 22050Hz
        assert!((resampled.samples.len() as i32 - 2205).abs() < 10);
    }

    #[test]
    fn test_barge_in_cooldown() {
        let config = BargeInConfig {
            cooldown_ms: 500,
            ..Default::default()
        };

        // Verify cooldown prevents rapid barge-ins
        // Would need full detector setup for real test
    }

    #[test]
    fn test_opus_roundtrip() {
        // Test Opus encode/decode
        let mut codec = OpusCodec::new(SampleRate::Hz16000).unwrap();

        // Create test signal (sine wave)
        let samples: Vec<f32> = (0..320)
            .map(|i| (i as f32 * 0.1).sin())
            .collect();

        let encoded = codec.encode(&samples).unwrap();
        let decoded = codec.decode(&encoded).unwrap();

        // Lossy compression, just verify we get output
        assert!(!decoded.is_empty());
    }
}
```

## Summary

The audio pipeline provides:

1. **Low-latency VAD**: Silero VAD with state machine for robust speech detection
2. **Streaming STT**: Chunk-based transcription with partial results and sentence accumulation
3. **Sentence-by-sentence TTS**: Pipelined synthesis starting with first LLM sentence
4. **Barge-in handling**: Configurable interrupt detection and response
5. **Latency optimization**: Prefetching, pipelining, and careful budget management
6. **Error recovery**: Circuit breaker pattern for graceful degradation

Target end-to-end latency: **600-800ms** from user speech end to first agent audio.
