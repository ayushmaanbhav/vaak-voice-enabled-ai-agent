# Optimized Voice Pipeline Architecture

> Research-driven, production-grade pipeline design for sub-500ms latency
>
> **Target:** 450-550ms E2E latency | Full-duplex capable | 22 Indian languages

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Architecture Taxonomy](#architecture-taxonomy)
3. [7-Layer Pipeline Architecture](#7-layer-pipeline-architecture)
4. [Latency Budget](#latency-budget)
5. [Component Designs](#component-designs)
6. [Research Sources](#research-sources)

---

## Executive Summary

### Key Innovations from Gap Analysis

| Gap | Solution | Impact |
|-----|----------|--------|
| **Semantic Turn Detection** | Turnsense SmolLM2-135M `<\|im_end\|>` detection | -200-600ms latency |
| **MagicNet VAD** | 10ms frames, causal conv + GRU | <15ms detection |
| **Enhanced STT** | Hallucination prevention, N-gram blocking | +10% WER improvement |
| **Speculative Execution** | Parallel SLM/LLM with EAGLE-style draft-verify | 2-5x speedup |
| **Early Exit Reranking** | PABEE-style layer-wise exit | 2-3.5x speedup |
| **Word-Level Barge-in** | 100-200ms TTS chunks with crossfade | Natural interruptions |
| **MCP Tools** | Industry-standard tool interface | Anthropic/OpenAI compatible |

### Target Latency Breakdown

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     OPTIMIZED LATENCY BUDGET (~450-550ms)                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Component                  │ Target    │ Technique                        │
│  ─────────────────────────────────────────────────────────────────────────  │
│  VAD + Turn Detection       │ 15-50ms   │ MagicNet 10ms + semantic hybrid  │
│  STT (streaming)            │ 100-150ms │ Partial results, prefetch RAG    │
│  RAG (speculative)          │ 50-100ms  │ Async prefetch, early exit CE    │
│  LLM (first token)          │ 150-200ms │ SLM race, KV cache, 4-bit quant  │
│  TTS (first audio)          │ 80-100ms  │ Word-level streaming             │
│  Network overhead           │ 50-100ms  │ WebRTC, persistent connections   │
│  ─────────────────────────────────────────────────────────────────────────  │
│  TOTAL                      │ 445-700ms │                                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Architecture Taxonomy

### Full-Duplex Survey (arXiv:2509.14515)

Research identifies two main approaches for simultaneous speaking/listening:

#### 1. Engineered Synchronization

| System | Latency | Approach |
|--------|---------|----------|
| **Voila** | 195ms | Separate STT/TTS with turn predictor |
| **LiveKit** | ~200ms | WebRTC + modular pipeline |
| **Pipecat** | ~250ms | Frame-based async pipeline |

**Pros:** Modular, easier to debug, proven components
**Cons:** Higher latency, complex coordination

#### 2. Learned Synchronization

| System | Latency | Approach |
|--------|---------|----------|
| **Moshi** | 160ms | Native audio-audio model |
| **GPT-4o Realtime** | ~200ms | End-to-end multimodal |
| **X-Talk** | ~180ms | Full-duplex transformer |

**Pros:** Lower latency, natural overlap handling
**Cons:** Less controllable, harder to debug, model-dependent

### Our Choice: Hybrid Engineered

We use **Engineered Synchronization** with:
- Speculative execution for latency reduction
- Semantic turn detection from learned models
- Word-level streaming for natural flow

---

## 7-Layer Pipeline Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      7-LAYER VOICE PIPELINE ARCHITECTURE                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  LAYER 7: TRANSPORT                                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  WebRTC │ WebSocket │ SIP Gateway                                    │   │
│  │  Opus codec │ Jitter buffer │ Echo cancellation                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │▲                                             │
│  LAYER 6: AUDIO PROCESSING   ││                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  MagicNet VAD (10ms) │ AGC │ Noise suppression │ Resampling          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │▲                                             │
│  LAYER 5: SPEECH RECOGNITION ││                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Streaming STT │ Enhanced decoder │ Semantic turn detection          │   │
│  │  IndicConformer │ Hallucination prevention │ HybridTurnDetector      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │▲                                             │
│  LAYER 4: TEXT PROCESSING    ││                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Grammar correction │ Translation (IN↔EN) │ PII redaction            │   │
│  │  Compliance check │ Entity extraction                                 │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │▲                                             │
│  LAYER 3: INTELLIGENCE       ││                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Speculative LLM (SLM race) │ Agentic RAG │ MCP Tools                │   │
│  │  Early-exit cross-encoder │ Context management │ Memory              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │▲                                             │
│  LAYER 2: AGENT LOGIC        ││                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Stage-based FSM │ Persuasion strategy │ Objection handling          │   │
│  │  Customer personalization │ Disclosure timing                         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │▲                                             │
│  LAYER 1: SPEECH SYNTHESIS   ││                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Word-level TTS │ Crossfade smoothing │ Barge-in handling            │   │
│  │  IndicF5 │ Prosody control │ SSML generation                         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Latency Budget

### Detailed Breakdown

| Stage | Component | Target | P95 Max | Optimization |
|-------|-----------|--------|---------|--------------|
| **Audio In** | WebRTC receive | 10ms | 20ms | Low-latency codec |
| **VAD** | MagicNet | 10ms | 15ms | 10ms frames, no lookahead |
| **STT** | IndicConformer | 100ms | 150ms | Streaming, partial results |
| **Turn Det** | HybridTurnDetector | 30ms | 50ms | SmolLM2-135M |
| **Grammar** | LLM-based | 30ms | 50ms | Async, sentence-level |
| **Translation** | IndicTrans2 | 40ms | 60ms | Cached, batched |
| **RAG Prefetch** | Qdrant+Tantivy | 50ms | 100ms | Speculative start on partial |
| **Rerank** | Early-exit CE | 20ms | 40ms | PABEE 2-3 layers |
| **LLM TTFT** | Speculative | 120ms | 180ms | SLM race, 4-bit quant |
| **TTS TTFA** | Word-level | 60ms | 100ms | First word streaming |
| **Audio Out** | WebRTC send | 10ms | 20ms | Low-latency buffer |
| **TOTAL** | | **480ms** | **785ms** | |

### Critical Path

```
User speech end
      │
      ├──► VAD detects silence (10ms)
      │
      ├──► Semantic turn detection (30ms) ──┬──► RAG prefetch starts (async)
      │                                      │
      ├──► STT finalizes (100ms) ────────────┼──► RAG results ready (~50ms)
      │                                      │
      ├──► Grammar + Translation (70ms) ─────┘
      │
      ├──► LLM first token (150ms) ──► SLM provides interim response
      │
      ├──► TTS first audio (80ms)
      │
      └──► User hears response (~450ms from speech end)
```

---

## Component Designs

### 1. Semantic Turn Detection (Gap 1)

```rust
// crates/pipeline/src/turn_detection/mod.rs

/// Turn state detected by semantic analysis
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TurnState {
    /// Complete thought, expecting response
    Finished,
    /// User explicitly asked to wait ("hold on", "ek minute")
    Wait,
    /// User paused but likely to continue
    Unfinished,
}

/// Configuration for semantic turn detector
#[derive(Debug, Clone)]
pub struct TurnDetectorConfig {
    /// ONNX model path (SmolLM2-135M-Instruct)
    pub model_path: PathBuf,
    /// Tokenizer path
    pub tokenizer_path: PathBuf,
    /// Probability threshold for <|im_end|> token
    pub end_token_threshold: f32,
    /// Maximum sequence length
    pub max_seq_len: usize,
    /// History turns to include
    pub history_turns: usize,
}

impl Default for TurnDetectorConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/turn_detection/smollm2-135m.onnx"),
            tokenizer_path: PathBuf::from("models/turn_detection/tokenizer.json"),
            end_token_threshold: 0.7,
            max_seq_len: 512,
            history_turns: 3,
        }
    }
}

/// Semantic turn detector using SLM
pub struct SemanticTurnDetector {
    session: Session,
    tokenizer: Tokenizer,
    config: TurnDetectorConfig,
    im_end_token_id: i64,
}

impl SemanticTurnDetector {
    pub fn new(config: TurnDetectorConfig) -> Result<Self, Error> {
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_model_from_file(&config.model_path)?;

        let tokenizer = Tokenizer::from_file(&config.tokenizer_path)?;

        // Find <|im_end|> token ID
        let im_end_token_id = tokenizer
            .token_to_id("<|im_end|>")
            .ok_or(Error::TokenNotFound)?;

        Ok(Self {
            session,
            tokenizer,
            config,
            im_end_token_id: im_end_token_id as i64,
        })
    }

    /// Detect if user turn is complete
    pub fn detect(&self, transcript: &str, history: &[Turn]) -> Result<(TurnState, f32), Error> {
        // Build chat-format input
        let mut input = String::new();

        // Add history
        for turn in history.iter().rev().take(self.config.history_turns).rev() {
            input.push_str(&format!(
                "<|im_start|>{}\n{}<|im_end|>\n",
                turn.role, turn.content
            ));
        }

        // Add current user message (without closing)
        input.push_str(&format!("<|im_start|>user\n{}", transcript));

        // Tokenize
        let encoding = self.tokenizer.encode(input, true)?;
        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&x| x as i64).collect();

        // Run inference
        let input_tensor = Array2::from_shape_vec(
            (1, input_ids.len()),
            input_ids.clone()
        )?;

        let outputs = self.session.run(ort::inputs![
            "input_ids" => input_tensor.view(),
        ]?)?;

        // Get logits for last position
        let logits: ArrayView3<f32> = outputs["logits"].try_extract_tensor()?;
        let last_logits = logits.slice(s![0, -1, ..]);

        // Softmax to get probabilities
        let max_logit = last_logits.fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        let exp_sum: f32 = last_logits.iter().map(|&x| (x - max_logit).exp()).sum();
        let probs: Vec<f32> = last_logits.iter()
            .map(|&x| (x - max_logit).exp() / exp_sum)
            .collect();

        // Check probability of <|im_end|>
        let im_end_prob = probs[self.im_end_token_id as usize];

        // Determine state
        let state = if im_end_prob >= self.config.end_token_threshold {
            TurnState::Finished
        } else if self.contains_wait_signal(transcript) {
            TurnState::Wait
        } else {
            TurnState::Unfinished
        };

        Ok((state, im_end_prob))
    }

    fn contains_wait_signal(&self, text: &str) -> bool {
        let wait_patterns = [
            "hold on", "wait", "ek minute", "ruko", "bas", "one second",
            "let me think", "sochne do", "abhi ruko",
        ];
        let lower = text.to_lowercase();
        wait_patterns.iter().any(|p| lower.contains(p))
    }
}

/// Hybrid VAD + Semantic turn detector
pub struct HybridTurnDetector {
    vad: Arc<VoiceActivityDetector>,
    semantic: SemanticTurnDetector,
    config: HybridConfig,
    state: Mutex<HybridState>,
}

#[derive(Debug, Clone)]
pub struct HybridConfig {
    /// Minimum silence before semantic check (ms)
    pub min_silence_ms: u64,
    /// Maximum silence before forced turn end (ms)
    pub max_silence_ms: u64,
    /// Use semantic detection
    pub semantic_enabled: bool,
}

impl HybridTurnDetector {
    pub async fn detect_turn_end(
        &self,
        transcript: &str,
        history: &[Turn],
        silence_duration_ms: u64,
    ) -> TurnDetectionResult {
        // Quick path: very long silence = definite end
        if silence_duration_ms >= self.config.max_silence_ms {
            return TurnDetectionResult {
                is_turn_end: true,
                confidence: 1.0,
                reason: TurnEndReason::MaxSilence,
            };
        }

        // If below minimum silence, not enough signal yet
        if silence_duration_ms < self.config.min_silence_ms {
            return TurnDetectionResult {
                is_turn_end: false,
                confidence: 0.0,
                reason: TurnEndReason::InsufficientSilence,
            };
        }

        // Semantic detection
        if self.config.semantic_enabled {
            match self.semantic.detect(transcript, history) {
                Ok((TurnState::Finished, confidence)) => {
                    return TurnDetectionResult {
                        is_turn_end: true,
                        confidence,
                        reason: TurnEndReason::SemanticComplete,
                    };
                }
                Ok((TurnState::Wait, confidence)) => {
                    return TurnDetectionResult {
                        is_turn_end: false,
                        confidence,
                        reason: TurnEndReason::ExplicitWait,
                    };
                }
                Ok((TurnState::Unfinished, confidence)) => {
                    // Continue waiting
                    return TurnDetectionResult {
                        is_turn_end: false,
                        confidence,
                        reason: TurnEndReason::SemanticIncomplete,
                    };
                }
                Err(e) => {
                    tracing::warn!("Semantic detection failed: {}", e);
                    // Fall back to silence-based
                }
            }
        }

        // Fallback: medium silence with question/statement heuristics
        let ends_with_question = transcript.trim().ends_with('?');
        let has_sentence_end = transcript.contains('.') || transcript.contains('।');

        if ends_with_question || has_sentence_end {
            TurnDetectionResult {
                is_turn_end: true,
                confidence: 0.8,
                reason: TurnEndReason::PunctuationHeuristic,
            }
        } else {
            TurnDetectionResult {
                is_turn_end: false,
                confidence: 0.5,
                reason: TurnEndReason::Uncertain,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TurnDetectionResult {
    pub is_turn_end: bool,
    pub confidence: f32,
    pub reason: TurnEndReason,
}

#[derive(Debug, Clone, Copy)]
pub enum TurnEndReason {
    MaxSilence,
    SemanticComplete,
    SemanticIncomplete,
    ExplicitWait,
    PunctuationHeuristic,
    InsufficientSilence,
    Uncertain,
}
```

### 2. MagicNet-Inspired VAD (Gap 2)

```rust
// crates/pipeline/src/vad/magicnet.rs

/// MagicNet-inspired VAD with 10ms frames and no future lookahead
/// Based on: "MagicNet: Semi-supervised Voice Activity Detection"
///
/// Architecture:
/// - Causal depth-separable convolutions
/// - GRU for temporal modeling
/// - 10ms frame size for low latency
pub struct VoiceActivityDetector {
    session: Session,
    config: VadConfig,
    gru_state: Array2<f32>,
    mel_filterbank: MelFilterbank,
    state: VadState,
    speech_frames: usize,
    silence_frames: usize,
}

#[derive(Debug, Clone)]
pub struct VadConfig {
    /// Speech probability threshold
    pub threshold: f32,
    /// Frame size in ms (10ms for low latency)
    pub frame_ms: u32,
    /// Minimum speech frames to confirm speech
    pub min_speech_frames: usize,
    /// Minimum silence frames to confirm silence
    pub min_silence_frames: usize,
    /// Number of mel filterbank bins
    pub n_mels: usize,
    /// Sample rate
    pub sample_rate: u32,
    /// GRU hidden size
    pub gru_hidden_size: usize,
    /// Energy floor (dB) for quick silence detection
    pub energy_floor_db: f32,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            frame_ms: 10,
            min_speech_frames: 25,  // 250ms
            min_silence_frames: 30, // 300ms
            n_mels: 40,
            sample_rate: 16000,
            gru_hidden_size: 64,
            energy_floor_db: -50.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VadState {
    Silence,
    SpeechStart,
    Speech,
    SpeechEnd,
}

impl VoiceActivityDetector {
    pub fn new(model_path: &Path, config: VadConfig) -> Result<Self, Error> {
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(1)?  // Single thread for consistent latency
            .with_model_from_file(model_path)?;

        let gru_state = Array2::zeros((1, config.gru_hidden_size));
        let mel_filterbank = MelFilterbank::new(
            config.sample_rate,
            config.frame_ms as usize * config.sample_rate as usize / 1000,
            config.n_mels,
        )?;

        Ok(Self {
            session,
            config,
            gru_state,
            mel_filterbank,
            state: VadState::Silence,
            speech_frames: 0,
            silence_frames: 0,
        })
    }

    /// Process a 10ms audio frame
    pub fn process_frame(&mut self, audio: &[f32]) -> Result<(VadState, f32), Error> {
        // Quick energy check
        let energy_db = calculate_energy_db(audio);
        if energy_db < self.config.energy_floor_db {
            return self.update_state(false, 0.0);
        }

        // Extract mel features
        let mel_features = self.mel_filterbank.compute(audio)?;

        // Prepare input tensor [batch=1, frames=1, features=n_mels]
        let input_tensor = Array3::from_shape_vec(
            (1, 1, self.config.n_mels),
            mel_features,
        )?;

        // Run inference with GRU state
        let outputs = self.session.run(ort::inputs![
            "mel_input" => input_tensor.view(),
            "gru_state_in" => self.gru_state.view(),
        ]?)?;

        // Extract speech probability
        let speech_prob: f32 = outputs["speech_prob"]
            .try_extract_tensor::<f32>()?
            .view()
            .iter()
            .next()
            .copied()
            .unwrap_or(0.0);

        // Update GRU state for next frame
        let new_gru_state: ArrayView2<f32> = outputs["gru_state_out"]
            .try_extract_tensor()?;
        self.gru_state.assign(&new_gru_state);

        let is_speech = speech_prob >= self.config.threshold;
        self.update_state(is_speech, speech_prob)
    }

    fn update_state(&mut self, is_speech: bool, probability: f32) -> Result<(VadState, f32), Error> {
        match (self.state, is_speech) {
            (VadState::Silence, true) => {
                self.speech_frames = 1;
                self.state = VadState::SpeechStart;
            }
            (VadState::SpeechStart, true) => {
                self.speech_frames += 1;
                if self.speech_frames >= self.config.min_speech_frames {
                    self.state = VadState::Speech;
                }
            }
            (VadState::SpeechStart, false) => {
                // False alarm
                self.speech_frames = 0;
                self.state = VadState::Silence;
            }
            (VadState::Speech, true) => {
                self.silence_frames = 0;
            }
            (VadState::Speech, false) => {
                self.silence_frames = 1;
                self.state = VadState::SpeechEnd;
            }
            (VadState::SpeechEnd, true) => {
                // Speech resumed
                self.silence_frames = 0;
                self.state = VadState::Speech;
            }
            (VadState::SpeechEnd, false) => {
                self.silence_frames += 1;
                if self.silence_frames >= self.config.min_silence_frames {
                    self.speech_frames = 0;
                    self.silence_frames = 0;
                    self.state = VadState::Silence;
                }
            }
            (VadState::Silence, false) => {}
        }

        Ok((self.state, probability))
    }

    pub fn reset(&mut self) {
        self.gru_state.fill(0.0);
        self.state = VadState::Silence;
        self.speech_frames = 0;
        self.silence_frames = 0;
    }
}

fn calculate_energy_db(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return -96.0;
    }
    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
    if rms > 0.0 { 20.0 * rms.log10() } else { -96.0 }
}

/// Mel filterbank for feature extraction
pub struct MelFilterbank {
    sample_rate: u32,
    n_fft: usize,
    n_mels: usize,
    filterbank: Array2<f32>,
    window: Vec<f32>,
}

impl MelFilterbank {
    pub fn new(sample_rate: u32, frame_size: usize, n_mels: usize) -> Result<Self, Error> {
        let n_fft = frame_size.next_power_of_two();

        // Hann window
        let window: Vec<f32> = (0..frame_size)
            .map(|i| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (frame_size - 1) as f32).cos()))
            .collect();

        // Create mel filterbank matrix
        let filterbank = create_mel_filterbank(sample_rate, n_fft, n_mels)?;

        Ok(Self {
            sample_rate,
            n_fft,
            n_mels,
            filterbank,
            window,
        })
    }

    pub fn compute(&self, audio: &[f32]) -> Result<Vec<f32>, Error> {
        // Apply window
        let windowed: Vec<f32> = audio.iter()
            .zip(&self.window)
            .map(|(a, w)| a * w)
            .collect();

        // Zero-pad to n_fft
        let mut padded = vec![0.0f32; self.n_fft];
        padded[..windowed.len()].copy_from_slice(&windowed);

        // FFT (using real-valued FFT)
        let spectrum = compute_magnitude_spectrum(&padded)?;

        // Apply mel filterbank
        let mel_energies = self.filterbank.dot(&Array1::from_vec(spectrum));

        // Log compression
        let log_mel: Vec<f32> = mel_energies.iter()
            .map(|&e| (e.max(1e-10)).ln())
            .collect();

        Ok(log_mel)
    }
}
```

### 3. Enhanced STT Decoder (Gap 3)

```rust
// crates/pipeline/src/stt/decoder.rs

/// Enhanced beam search decoder with hallucination prevention
/// Implements:
/// - Token suppression (silence/special tokens)
/// - N-gram repetition blocking
/// - Hallucination pattern detection
/// - Length normalization
pub struct EnhancedSttDecoder {
    config: DecoderConfig,
    suppress_tokens: HashSet<i64>,
    ngram_cache: HashMap<Vec<i64>, usize>,
    hallucination_patterns: Vec<Vec<i64>>,
}

#[derive(Debug, Clone)]
pub struct DecoderConfig {
    /// Beam width
    pub beam_size: usize,
    /// Temperature for sampling (lower = more deterministic)
    pub temperature: f32,
    /// Length penalty (alpha in length normalization)
    pub length_penalty: f32,
    /// Repetition penalty
    pub repetition_penalty: f32,
    /// N-gram size for blocking repetitions
    pub ngram_block_size: usize,
    /// Max consecutive silence tokens
    pub max_silence_tokens: usize,
    /// Patience for early stopping (number of non-improving steps)
    pub patience: usize,
    /// Hallucination detection threshold
    pub hallucination_threshold: f32,
}

impl Default for DecoderConfig {
    fn default() -> Self {
        Self {
            beam_size: 5,
            temperature: 0.0,  // Greedy by default
            length_penalty: 1.0,
            repetition_penalty: 1.2,
            ngram_block_size: 3,
            max_silence_tokens: 2,
            patience: 3,
            hallucination_threshold: 0.8,
        }
    }
}

impl EnhancedSttDecoder {
    pub fn new(config: DecoderConfig, tokenizer: &Tokenizer) -> Result<Self, Error> {
        // Build suppress token set
        let mut suppress_tokens = HashSet::new();

        // Suppress silence and special tokens
        for token in ["<|silence|>", "<|nospeech|>", "[BLANK]", "<blank>"] {
            if let Some(id) = tokenizer.token_to_id(token) {
                suppress_tokens.insert(id as i64);
            }
        }

        // Build hallucination patterns
        let hallucination_patterns = build_hallucination_patterns(tokenizer)?;

        Ok(Self {
            config,
            suppress_tokens,
            ngram_cache: HashMap::new(),
            hallucination_patterns,
        })
    }

    pub fn decode(
        &mut self,
        encoder_output: &Array3<f32>,
        tokenizer: &Tokenizer,
    ) -> Result<DecodingResult, Error> {
        self.ngram_cache.clear();

        let mut beams: Vec<BeamState> = vec![BeamState::new()];
        let mut best_finished: Option<BeamState> = None;
        let mut non_improving_steps = 0;

        for step in 0..self.config.max_length() {
            let mut new_beams = Vec::new();

            for beam in &beams {
                // Get logits for next token
                let logits = self.get_next_logits(encoder_output, &beam.tokens)?;

                // Apply modifications
                let modified_logits = self.modify_logits(&logits, &beam.tokens);

                // Get top-k candidates
                let candidates = self.get_top_k(&modified_logits, self.config.beam_size * 2);

                for (token_id, log_prob) in candidates {
                    // Skip suppressed tokens
                    if self.suppress_tokens.contains(&token_id) {
                        continue;
                    }

                    // Check N-gram blocking
                    if self.would_create_repeated_ngram(&beam.tokens, token_id) {
                        continue;
                    }

                    // Check hallucination patterns
                    if self.matches_hallucination_pattern(&beam.tokens, token_id) {
                        continue;
                    }

                    let mut new_beam = beam.clone();
                    new_beam.tokens.push(token_id);
                    new_beam.log_prob += log_prob;
                    new_beam.length += 1;

                    // Check if finished
                    if token_id == tokenizer.token_to_id("<|endoftext|>").unwrap() as i64 {
                        let score = self.compute_score(&new_beam);
                        if best_finished.as_ref().map_or(true, |b| score > self.compute_score(b)) {
                            best_finished = Some(new_beam.clone());
                        }
                    } else {
                        new_beams.push(new_beam);
                    }
                }
            }

            // Keep top beams
            new_beams.sort_by(|a, b| {
                self.compute_score(b).partial_cmp(&self.compute_score(a)).unwrap()
            });
            beams = new_beams.into_iter().take(self.config.beam_size).collect();

            // Early stopping with patience
            if beams.is_empty() {
                break;
            }

            let best_current = self.compute_score(&beams[0]);
            if let Some(ref finished) = best_finished {
                if self.compute_score(finished) >= best_current {
                    non_improving_steps += 1;
                    if non_improving_steps >= self.config.patience {
                        break;
                    }
                } else {
                    non_improving_steps = 0;
                }
            }
        }

        // Get best result
        let best = best_finished.or_else(|| beams.into_iter().next())
            .ok_or(Error::NoValidDecoding)?;

        // Post-process: remove hallucinations
        let cleaned_tokens = self.remove_hallucinations(&best.tokens)?;

        // Decode to text
        let text = tokenizer.decode(&cleaned_tokens, true)?;

        Ok(DecodingResult {
            text,
            tokens: cleaned_tokens,
            log_prob: best.log_prob,
            confidence: self.compute_confidence(&best),
        })
    }

    fn modify_logits(&self, logits: &Array1<f32>, context: &[i64]) -> Array1<f32> {
        let mut modified = logits.clone();

        // Apply repetition penalty
        for &token in context {
            if token >= 0 && (token as usize) < modified.len() {
                modified[token as usize] /= self.config.repetition_penalty;
            }
        }

        // Suppress tokens
        for &token in &self.suppress_tokens {
            if token >= 0 && (token as usize) < modified.len() {
                modified[token as usize] = f32::NEG_INFINITY;
            }
        }

        // Apply temperature
        if self.config.temperature > 0.0 {
            modified.mapv_inplace(|x| x / self.config.temperature);
        }

        modified
    }

    fn would_create_repeated_ngram(&self, tokens: &[i64], next_token: i64) -> bool {
        if tokens.len() < self.config.ngram_block_size {
            return false;
        }

        // Get the n-gram that would be created
        let start = tokens.len() - self.config.ngram_block_size + 1;
        let mut ngram: Vec<i64> = tokens[start..].to_vec();
        ngram.push(next_token);

        // Check if this n-gram exists earlier in the sequence
        for i in 0..=(tokens.len() - self.config.ngram_block_size) {
            let existing: Vec<i64> = tokens[i..i + self.config.ngram_block_size].to_vec();
            if existing == ngram[..self.config.ngram_block_size] {
                return true;
            }
        }

        false
    }

    fn matches_hallucination_pattern(&self, tokens: &[i64], next_token: i64) -> bool {
        for pattern in &self.hallucination_patterns {
            if tokens.len() >= pattern.len() - 1 {
                let context_start = tokens.len() - (pattern.len() - 1);
                let context = &tokens[context_start..];
                if context == &pattern[..pattern.len() - 1] && next_token == pattern[pattern.len() - 1] {
                    return true;
                }
            }
        }
        false
    }

    fn remove_hallucinations(&self, tokens: &[i64]) -> Result<Vec<i64>, Error> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            let mut matched = false;

            // Check for hallucination patterns
            for pattern in &self.hallucination_patterns {
                if i + pattern.len() <= tokens.len() {
                    if &tokens[i..i + pattern.len()] == pattern.as_slice() {
                        // Skip the hallucination pattern
                        i += pattern.len();
                        matched = true;
                        break;
                    }
                }
            }

            if !matched {
                result.push(tokens[i]);
                i += 1;
            }
        }

        Ok(result)
    }

    fn compute_score(&self, beam: &BeamState) -> f32 {
        // Length-normalized log probability
        beam.log_prob / (beam.length as f32).powf(self.config.length_penalty)
    }

    fn compute_confidence(&self, beam: &BeamState) -> f32 {
        // Convert log probability to confidence
        let avg_log_prob = beam.log_prob / beam.length as f32;
        avg_log_prob.exp().min(1.0)
    }
}

#[derive(Debug, Clone)]
struct BeamState {
    tokens: Vec<i64>,
    log_prob: f32,
    length: usize,
}

impl BeamState {
    fn new() -> Self {
        Self {
            tokens: Vec::new(),
            log_prob: 0.0,
            length: 0,
        }
    }
}

#[derive(Debug)]
pub struct DecodingResult {
    pub text: String,
    pub tokens: Vec<i64>,
    pub log_prob: f32,
    pub confidence: f32,
}

fn build_hallucination_patterns(tokenizer: &Tokenizer) -> Result<Vec<Vec<i64>>, Error> {
    // Common Whisper hallucinations
    let patterns = [
        "Thank you for watching.",
        "Thanks for watching!",
        "Please subscribe.",
        "Like and subscribe",
        "See you in the next video",
        "[Music]",
        "[Applause]",
        "...",  // Repeated dots
    ];

    let mut token_patterns = Vec::new();
    for pattern in patterns {
        let encoding = tokenizer.encode(pattern, false)?;
        token_patterns.push(encoding.get_ids().iter().map(|&x| x as i64).collect());
    }

    Ok(token_patterns)
}
```

### 4. Speculative LLM Executor (Gap 6)

```rust
// crates/llm/src/speculative.rs

/// Speculative execution modes for LLM inference
#[derive(Debug, Clone)]
pub enum SpeculativeMode {
    /// Run SLM first, switch to LLM if complex
    SlmFirst {
        slm_timeout_ms: u64,
        complexity_threshold: f32,
    },
    /// Race SLM and LLM in parallel, use first complete
    RaceParallel,
    /// Stream from SLM, verify/replace with LLM chunks
    HybridStreaming {
        switch_threshold_tokens: usize,
    },
    /// EAGLE-style draft-verify with tree attention
    DraftVerify {
        draft_length: usize,
        temperature: f32,
    },
}

/// Configuration for speculative execution
#[derive(Debug, Clone)]
pub struct SpeculativeConfig {
    pub mode: SpeculativeMode,
    /// SLM model (small, fast)
    pub slm_model: String,
    /// LLM model (large, quality)
    pub llm_model: String,
    /// Maximum tokens to generate
    pub max_tokens: usize,
    /// Acceptance threshold for draft tokens
    pub acceptance_threshold: f32,
}

impl Default for SpeculativeConfig {
    fn default() -> Self {
        Self {
            mode: SpeculativeMode::SlmFirst {
                slm_timeout_ms: 100,
                complexity_threshold: 0.7,
            },
            slm_model: "qwen2.5:1.5b-q4".to_string(),
            llm_model: "qwen2.5:7b-q4".to_string(),
            max_tokens: 256,
            acceptance_threshold: 0.8,
        }
    }
}

/// Speculative LLM executor with parallel SLM/LLM
pub struct SpeculativeLlmExecutor {
    config: SpeculativeConfig,
    slm_client: Arc<OllamaClient>,
    llm_client: Arc<OllamaClient>,
}

impl SpeculativeLlmExecutor {
    pub fn new(config: SpeculativeConfig, ollama_url: &str) -> Result<Self, Error> {
        let slm_client = Arc::new(OllamaClient::new(ollama_url)?);
        let llm_client = Arc::new(OllamaClient::new(ollama_url)?);

        Ok(Self {
            config,
            slm_client,
            llm_client,
        })
    }

    /// Generate response with speculative execution
    pub fn generate_stream(
        &self,
        prompt: &str,
        context: &ConversationContext,
    ) -> impl Stream<Item = Result<String, Error>> + '_ {
        let prompt = prompt.to_string();
        let context = context.clone();

        stream! {
            match &self.config.mode {
                SpeculativeMode::SlmFirst { slm_timeout_ms, complexity_threshold } => {
                    for await chunk in self.generate_slm_first(&prompt, &context, *slm_timeout_ms, *complexity_threshold) {
                        yield chunk;
                    }
                }
                SpeculativeMode::RaceParallel => {
                    for await chunk in self.generate_race_parallel(&prompt, &context) {
                        yield chunk;
                    }
                }
                SpeculativeMode::HybridStreaming { switch_threshold_tokens } => {
                    for await chunk in self.generate_hybrid_streaming(&prompt, &context, *switch_threshold_tokens) {
                        yield chunk;
                    }
                }
                SpeculativeMode::DraftVerify { draft_length, temperature } => {
                    for await chunk in self.generate_draft_verify(&prompt, &context, *draft_length, *temperature) {
                        yield chunk;
                    }
                }
            }
        }
    }

    /// SLM-first strategy: start with fast model, upgrade if needed
    fn generate_slm_first<'a>(
        &'a self,
        prompt: &'a str,
        context: &'a ConversationContext,
        timeout_ms: u64,
        complexity_threshold: f32,
    ) -> impl Stream<Item = Result<String, Error>> + 'a {
        stream! {
            let start = Instant::now();
            let mut slm_output = String::new();
            let mut token_count = 0;

            // Start SLM generation
            let slm_stream = self.slm_client.generate_stream(
                &self.config.slm_model,
                prompt,
            );
            pin_mut!(slm_stream);

            // Stream SLM tokens up to timeout
            while let Ok(Some(chunk)) = tokio::time::timeout(
                Duration::from_millis(10),
                slm_stream.next(),
            ).await {
                match chunk {
                    Ok(text) => {
                        slm_output.push_str(&text);
                        token_count += 1;
                        yield Ok(text);

                        // Check timeout
                        if start.elapsed().as_millis() > timeout_ms as u128 {
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("SLM error: {}", e);
                        break;
                    }
                }
            }

            // Assess complexity
            let complexity = self.assess_complexity(prompt, &slm_output);

            if complexity > complexity_threshold {
                // Switch to LLM for remainder
                tracing::info!("Switching to LLM (complexity: {})", complexity);

                let llm_stream = self.llm_client.generate_stream(
                    &self.config.llm_model,
                    &format!("{}\n{}", prompt, slm_output),
                );
                pin_mut!(llm_stream);

                while let Some(chunk) = llm_stream.next().await {
                    yield chunk;
                }
            } else {
                // Continue with SLM
                while let Some(chunk) = slm_stream.next().await {
                    yield chunk;
                }
            }
        }
    }

    /// Race parallel: run both models, use faster complete response
    fn generate_race_parallel<'a>(
        &'a self,
        prompt: &'a str,
        context: &'a ConversationContext,
    ) -> impl Stream<Item = Result<String, Error>> + 'a {
        stream! {
            let slm_task = tokio::spawn({
                let client = self.slm_client.clone();
                let model = self.config.slm_model.clone();
                let prompt = prompt.to_string();
                async move {
                    client.generate(&model, &prompt).await
                }
            });

            let llm_task = tokio::spawn({
                let client = self.llm_client.clone();
                let model = self.config.llm_model.clone();
                let prompt = prompt.to_string();
                async move {
                    client.generate(&model, &prompt).await
                }
            });

            // Wait for first to complete
            tokio::select! {
                result = slm_task => {
                    if let Ok(Ok(text)) = result {
                        yield Ok(text);
                        llm_task.abort();
                    }
                }
                result = llm_task => {
                    if let Ok(Ok(text)) = result {
                        yield Ok(text);
                        slm_task.abort();
                    }
                }
            }
        }
    }

    /// EAGLE-style draft-verify with speculative decoding
    fn generate_draft_verify<'a>(
        &'a self,
        prompt: &'a str,
        _context: &'a ConversationContext,
        draft_length: usize,
        _temperature: f32,
    ) -> impl Stream<Item = Result<String, Error>> + 'a {
        stream! {
            let mut generated = String::new();
            let mut current_prompt = prompt.to_string();

            loop {
                // Generate draft with SLM
                let draft = self.slm_client
                    .generate_n_tokens(&self.config.slm_model, &current_prompt, draft_length)
                    .await?;

                if draft.is_empty() {
                    break;
                }

                // Verify with LLM
                let verification_prompt = format!("{}{}", current_prompt, draft);
                let verification = self.llm_client
                    .verify_tokens(&self.config.llm_model, &verification_prompt, &draft, self.config.acceptance_threshold)
                    .await?;

                // Accept verified tokens
                let accepted = &draft[..verification.accepted_count];
                if accepted.is_empty() {
                    // No tokens accepted, generate one with LLM
                    let llm_token = self.llm_client
                        .generate_n_tokens(&self.config.llm_model, &current_prompt, 1)
                        .await?;
                    generated.push_str(&llm_token);
                    current_prompt.push_str(&llm_token);
                    yield Ok(llm_token);
                } else {
                    generated.push_str(accepted);
                    current_prompt.push_str(accepted);
                    yield Ok(accepted.to_string());
                }

                // Check for completion
                if generated.ends_with('\n') || generated.len() >= self.config.max_tokens * 4 {
                    break;
                }
            }
        }
    }

    fn assess_complexity(&self, prompt: &str, response: &str) -> f32 {
        // Simple heuristics for complexity
        let mut score = 0.0;

        // Question complexity
        if prompt.contains("why") || prompt.contains("explain") || prompt.contains("compare") {
            score += 0.3;
        }

        // Response uncertainty indicators
        let uncertainty_words = ["maybe", "perhaps", "might", "could", "not sure"];
        for word in uncertainty_words {
            if response.to_lowercase().contains(word) {
                score += 0.1;
            }
        }

        // Domain-specific complexity (gold loan)
        let complex_topics = ["RBI", "regulation", "compliance", "legal", "auction"];
        for topic in complex_topics {
            if prompt.to_lowercase().contains(topic) {
                score += 0.2;
            }
        }

        score.min(1.0)
    }
}
```

### 5. Early-Exit Cross-Encoder (Gap 5)

```rust
// crates/rag/src/reranker/early_exit.rs

/// Exit strategy for early-exit cross-encoder
#[derive(Debug, Clone)]
pub enum ExitStrategy {
    /// Exit when confidence exceeds threshold
    Confidence { threshold: f32 },
    /// Exit when k consecutive layers agree
    Patience { k: usize },
    /// Combination of confidence and patience
    Hybrid { confidence_threshold: f32, patience_k: usize },
    /// Exit based on similarity between layer outputs
    SimilarityBased { similarity_threshold: f32 },
}

/// Configuration for early-exit cross-encoder
#[derive(Debug, Clone)]
pub struct EarlyExitConfig {
    pub model_path: PathBuf,
    pub tokenizer_path: PathBuf,
    pub exit_strategy: ExitStrategy,
    pub max_seq_len: usize,
    pub min_exit_layer: usize,
}

impl Default for EarlyExitConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/reranker/bge-reranker-v2-m3.onnx"),
            tokenizer_path: PathBuf::from("models/reranker/tokenizer.json"),
            exit_strategy: ExitStrategy::Hybrid {
                confidence_threshold: 0.9,
                patience_k: 2,
            },
            max_seq_len: 512,
            min_exit_layer: 3,
        }
    }
}

/// Early-exit cross-encoder based on PABEE/DE³-BERT research
pub struct EarlyExitCrossEncoder {
    session: Session,
    tokenizer: Tokenizer,
    config: EarlyExitConfig,
    num_layers: usize,
    confidence_calibration: Option<ConfidenceCalibrator>,
}

impl EarlyExitCrossEncoder {
    pub fn new(config: EarlyExitConfig) -> Result<Self, Error> {
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_model_from_file(&config.model_path)?;

        let tokenizer = Tokenizer::from_file(&config.tokenizer_path)?;

        // Determine number of layers from model
        let num_layers = 12; // Typical for base models

        Ok(Self {
            session,
            tokenizer,
            config,
            num_layers,
            confidence_calibration: None,
        })
    }

    /// Rerank documents with early exit
    pub async fn rerank(
        &self,
        query: &str,
        documents: Vec<Document>,
    ) -> Result<Vec<RankedDocument>, Error> {
        let mut results = Vec::with_capacity(documents.len());

        for doc in documents {
            let (score, exit_layer, confidence) = self.score_with_early_exit(query, &doc.content)?;

            results.push(RankedDocument {
                document: doc,
                score,
                exit_layer,
                confidence,
            });
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }

    fn score_with_early_exit(
        &self,
        query: &str,
        document: &str,
    ) -> Result<(f32, usize, f32), Error> {
        // Tokenize query-document pair
        let encoding = self.tokenizer.encode((query, document), true)?;
        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&x| x as i64).collect();
        let attention_mask: Vec<i64> = encoding.get_attention_mask().iter().map(|&x| x as i64).collect();

        // Truncate if needed
        let seq_len = input_ids.len().min(self.config.max_seq_len);
        let input_ids = &input_ids[..seq_len];
        let attention_mask = &attention_mask[..seq_len];

        // Prepare tensors
        let input_tensor = Array2::from_shape_vec((1, seq_len), input_ids.to_vec())?;
        let mask_tensor = Array2::from_shape_vec((1, seq_len), attention_mask.to_vec())?;

        // Run inference with intermediate outputs
        let outputs = self.session.run(ort::inputs![
            "input_ids" => input_tensor.view(),
            "attention_mask" => mask_tensor.view(),
        ]?)?;

        // Get all layer logits
        let all_layer_logits = self.extract_layer_logits(&outputs)?;

        // Apply early exit strategy
        self.forward_with_early_exit(&all_layer_logits)
    }

    fn forward_with_early_exit(
        &self,
        layer_logits: &[f32],
    ) -> Result<(f32, usize, f32), Error> {
        let mut prev_predictions: Vec<i32> = Vec::new();
        let mut patience_count = 0;

        for layer in self.config.min_exit_layer..self.num_layers {
            let logits = layer_logits[layer];
            let prob = sigmoid(logits);
            let prediction = if prob > 0.5 { 1 } else { 0 };

            let confidence = if prob > 0.5 { prob } else { 1.0 - prob };

            // Apply exit strategy
            let should_exit = match &self.config.exit_strategy {
                ExitStrategy::Confidence { threshold } => {
                    confidence >= *threshold
                }
                ExitStrategy::Patience { k } => {
                    if !prev_predictions.is_empty() && prev_predictions.last() == Some(&prediction) {
                        patience_count += 1;
                    } else {
                        patience_count = 1;
                    }
                    patience_count >= *k
                }
                ExitStrategy::Hybrid { confidence_threshold, patience_k } => {
                    let confidence_met = confidence >= *confidence_threshold;

                    if !prev_predictions.is_empty() && prev_predictions.last() == Some(&prediction) {
                        patience_count += 1;
                    } else {
                        patience_count = 1;
                    }
                    let patience_met = patience_count >= *patience_k;

                    confidence_met || patience_met
                }
                ExitStrategy::SimilarityBased { similarity_threshold } => {
                    if layer > self.config.min_exit_layer {
                        let prev_logits = layer_logits[layer - 1];
                        let similarity = 1.0 - (logits - prev_logits).abs() / (logits.abs() + prev_logits.abs() + 1e-6);
                        similarity >= *similarity_threshold
                    } else {
                        false
                    }
                }
            };

            if should_exit {
                let calibrated_confidence = self.calibrate_confidence(confidence, layer);
                return Ok((prob, layer, calibrated_confidence));
            }

            prev_predictions.push(prediction);
        }

        // No early exit, use final layer
        let final_logits = layer_logits[self.num_layers - 1];
        let final_prob = sigmoid(final_logits);
        let final_confidence = if final_prob > 0.5 { final_prob } else { 1.0 - final_prob };

        Ok((final_prob, self.num_layers - 1, final_confidence))
    }

    fn extract_layer_logits(&self, outputs: &ort::SessionOutputs) -> Result<Vec<f32>, Error> {
        // Extract logits from each layer output
        let mut layer_logits = Vec::with_capacity(self.num_layers);

        for i in 0..self.num_layers {
            let output_name = format!("layer_{}_logits", i);
            if let Some(tensor) = outputs.get(&output_name) {
                let logits: f32 = tensor.try_extract_tensor::<f32>()?
                    .view()
                    .iter()
                    .next()
                    .copied()
                    .unwrap_or(0.0);
                layer_logits.push(logits);
            }
        }

        // If no per-layer outputs, use final logits repeated
        if layer_logits.is_empty() {
            let final_logits: f32 = outputs["logits"]
                .try_extract_tensor::<f32>()?
                .view()
                .iter()
                .next()
                .copied()
                .unwrap_or(0.0);
            layer_logits = vec![final_logits; self.num_layers];
        }

        Ok(layer_logits)
    }

    fn calibrate_confidence(&self, raw_confidence: f32, exit_layer: usize) -> f32 {
        if let Some(ref calibrator) = self.confidence_calibration {
            calibrator.calibrate(raw_confidence, exit_layer)
        } else {
            // Default: slight penalty for early exit
            let layer_factor = exit_layer as f32 / self.num_layers as f32;
            raw_confidence * (0.8 + 0.2 * layer_factor)
        }
    }
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

#[derive(Debug, Clone)]
pub struct RankedDocument {
    pub document: Document,
    pub score: f32,
    pub exit_layer: usize,
    pub confidence: f32,
}

/// Confidence calibrator based on validation data
pub struct ConfidenceCalibrator {
    layer_calibration: Vec<(f32, f32)>, // (slope, intercept) per layer
}

impl ConfidenceCalibrator {
    pub fn calibrate(&self, raw_confidence: f32, layer: usize) -> f32 {
        if layer < self.layer_calibration.len() {
            let (slope, intercept) = self.layer_calibration[layer];
            (raw_confidence * slope + intercept).clamp(0.0, 1.0)
        } else {
            raw_confidence
        }
    }
}
```

---

## Research Sources

### Academic Papers

| Paper | Key Finding | Application |
|-------|-------------|-------------|
| **Full-Duplex Survey (arXiv:2509.14515)** | Engineered vs Learned Synchronization taxonomy | Architecture choice |
| **Turnsense (LiveKit)** | SLM `<\|im_end\|>` detection for turn-taking | Semantic turn detection |
| **MagicNet** | Semi-supervised VAD with causal conv + GRU | Low-latency VAD |
| **PABEE** | Patience-based early exit for BERT | Early-exit reranking |
| **EAGLE** | Draft-verify speculative decoding | LLM acceleration |

### Industry Resources

| Source | Key Insight | Latency Target |
|--------|-------------|----------------|
| **Cresta Engineering Blog** | 78% failures in edge cases | Focus on P95/P99 |
| **Deepgram Voice AI** | 16% satisfaction drop per second >800ms | Sub-500ms critical |
| **LiveKit Realtime Voice** | 195ms with Voila architecture | Full-duplex achievable |
| **Hacker News (133ms agent)** | WebRTC + speculative = ultra-low latency | Aggressive optimization |

### Customer Experience Research

| Metric | Finding | Source |
|--------|---------|--------|
| **Latency tolerance** | 200-500ms expected, >1s = frustration | Human conversation norms |
| **ROI** | $3.50 return per $1 invested in voice UX | Industry studies |
| **Satisfaction** | 16% drop per additional second | Deepgram research |

---

## Implementation Priority

1. **Phase 1: Core Pipeline** (Weeks 1-4)
   - MagicNet VAD implementation
   - Streaming STT integration
   - Basic turn detection

2. **Phase 2: Intelligence** (Weeks 5-8)
   - HybridTurnDetector with semantic detection
   - Early-exit cross-encoder
   - Speculative LLM execution

3. **Phase 3: Polish** (Weeks 9-12)
   - Word-level TTS streaming
   - Barge-in handling
   - Latency optimization

4. **Phase 4: Production** (Weeks 13-16)
   - Load testing
   - Edge case handling
   - Monitoring and alerting
