# Phase 3: Pipeline Architecture Implementation

> **Priority:** P1 (High)
> **Duration:** 2 weeks
> **Dependencies:** Phase 1 (Core Traits)
> **Required For:** Streaming, low-latency responses

---

## Overview

This phase transforms the current simplified `VoicePipeline` orchestrator into a proper Frame-based processor chain as documented in ARCHITECTURE_v2.md.

---

## 1. Current vs Target Architecture

### Current (Simplified)
```
AudioFrame → VoicePipeline.process_audio()
              ├─ VAD (sequential)
              ├─ TurnDetection (sequential)
              ├─ STT (sequential)
              └─ emit PipelineEvent
```

### Target (Frame-Based)
```
Frame → Processor1 → Frame → Processor2 → Frame → Processor3
         (tokio task)        (tokio task)        (tokio task)
         via channel         via channel         via channel
```

---

## 2. Files to Create/Modify

### New Files
```
crates/pipeline/src/
├── frame.rs                    # Frame enum
├── processor_context.rs        # ProcessorContext
├── pipeline_v2.rs              # New Pipeline orchestrator
├── builder.rs                  # PipelineBuilder
├── streaming/
│   ├── mod.rs
│   ├── sentence_detector.rs
│   ├── sentence_accumulator.rs
│   └── llm_tts_streamer.rs
├── interrupt/
│   ├── mod.rs
│   ├── handler.rs
│   └── config.rs
└── processors/
    ├── mod.rs
    ├── vad_processor.rs
    ├── stt_processor.rs
    ├── tts_processor.rs
    ├── grammar_processor.rs
    ├── translation_processor.rs
    ├── compliance_processor.rs
    └── pii_processor.rs
```

---

## 3. Frame Enum

### 3.1 frame.rs

```rust
//! Pipeline frame types

use std::time::Duration;
use voice_agent_core::{
    AudioFrame, TranscriptFrame, Language,
    ComplianceResult, ConversationState,
    ToolCall, ToolResult,
};

/// Pipeline frames (events)
///
/// Frames flow through processor chain, each processor
/// consuming and emitting frames.
#[derive(Debug, Clone)]
pub enum Frame {
    // === Audio frames ===
    /// Raw audio input from client
    AudioInput(AudioFrame),
    /// Synthesized audio output to client
    AudioOutput(AudioFrame),

    // === Speech frames ===
    /// Partial (interim) transcript
    TranscriptPartial(TranscriptFrame),
    /// Final transcript
    TranscriptFinal(TranscriptFrame),

    // === Text processing frames ===
    /// Grammar-corrected text
    GrammarCorrected(String),
    /// Translated text with source/target languages
    Translated {
        text: String,
        from: Language,
        to: Language,
    },
    /// Compliance check result
    ComplianceChecked {
        text: String,
        result: ComplianceResult,
    },
    /// PII-redacted text
    PIIRedacted(String),

    // === LLM frames ===
    /// Streaming LLM chunk
    LLMChunk(String),
    /// Complete LLM response
    LLMComplete(String),
    /// Tool call request
    ToolCall(ToolCall),
    /// Tool execution result
    ToolResult(ToolResult),

    // === Control frames ===
    /// User started speaking
    UserSpeaking,
    /// User silence detected
    UserSilence(Duration),
    /// User interrupted agent
    BargeIn {
        /// Word index where barge-in occurred
        at_word: Option<usize>,
    },
    /// End of conversation turn
    EndOfTurn,

    // === System frames ===
    /// Conversation state changed
    StateChange(ConversationState),
    /// Error occurred
    Error(PipelineError),
    /// Metrics event
    Metrics(MetricsEvent),

    // === Passthrough ===
    /// Custom frame for extensions
    Custom(String, serde_json::Value),
}

/// Pipeline error
#[derive(Debug, Clone)]
pub struct PipelineError {
    pub processor: String,
    pub message: String,
    pub recoverable: bool,
}

/// Metrics event
#[derive(Debug, Clone)]
pub struct MetricsEvent {
    pub processor: String,
    pub event: String,
    pub duration_ms: u64,
    pub metadata: std::collections::HashMap<String, String>,
}

impl Frame {
    /// Check if this is a terminal frame
    pub fn is_terminal(&self) -> bool {
        matches!(self, Frame::EndOfTurn | Frame::Error(_))
    }

    /// Get frame type name for logging
    pub fn type_name(&self) -> &'static str {
        match self {
            Frame::AudioInput(_) => "AudioInput",
            Frame::AudioOutput(_) => "AudioOutput",
            Frame::TranscriptPartial(_) => "TranscriptPartial",
            Frame::TranscriptFinal(_) => "TranscriptFinal",
            Frame::GrammarCorrected(_) => "GrammarCorrected",
            Frame::Translated { .. } => "Translated",
            Frame::ComplianceChecked { .. } => "ComplianceChecked",
            Frame::PIIRedacted(_) => "PIIRedacted",
            Frame::LLMChunk(_) => "LLMChunk",
            Frame::LLMComplete(_) => "LLMComplete",
            Frame::ToolCall(_) => "ToolCall",
            Frame::ToolResult(_) => "ToolResult",
            Frame::UserSpeaking => "UserSpeaking",
            Frame::UserSilence(_) => "UserSilence",
            Frame::BargeIn { .. } => "BargeIn",
            Frame::EndOfTurn => "EndOfTurn",
            Frame::StateChange(_) => "StateChange",
            Frame::Error(_) => "Error",
            Frame::Metrics(_) => "Metrics",
            Frame::Custom(name, _) => "Custom",
        }
    }
}
```

---

## 4. Processor Context

### 4.1 processor_context.rs

```rust
//! Processor context for sharing state

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use voice_agent_core::{ConversationContext, Language};

/// Shared context across processors
pub struct ProcessorContext {
    /// Session ID
    pub session_id: String,
    /// Detected input language
    pub input_language: Language,
    /// Target output language
    pub output_language: Language,
    /// Current conversation context
    pub conversation: Arc<RwLock<ConversationContext>>,
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Accumulated text for current turn
    pub turn_text: String,
    /// Whether agent is currently speaking
    pub agent_speaking: bool,
    /// Current word index in TTS output
    pub tts_word_index: usize,
}

impl ProcessorContext {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            input_language: Language::Hindi,
            output_language: Language::Hindi,
            conversation: Arc::new(RwLock::new(ConversationContext::default())),
            metadata: HashMap::new(),
            turn_text: String::new(),
            agent_speaking: false,
            tts_word_index: 0,
        }
    }

    /// Set metadata value
    pub fn set_meta(&mut self, key: &str, value: serde_json::Value) {
        self.metadata.insert(key.to_string(), value);
    }

    /// Get metadata value
    pub fn get_meta(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Reset for new turn
    pub fn new_turn(&mut self) {
        self.turn_text.clear();
        self.agent_speaking = false;
        self.tts_word_index = 0;
    }
}

impl Default for ProcessorContext {
    fn default() -> Self {
        Self::new("default".to_string())
    }
}
```

---

## 5. Pipeline Orchestrator

### 5.1 pipeline_v2.rs

```rust
//! Frame-based pipeline orchestrator

use std::sync::Arc;
use tokio::sync::{mpsc, broadcast};
use tracing::{debug, error, info, span, Level};
use voice_agent_core::FrameProcessor;
use crate::{Frame, ProcessorContext, PipelineError};

/// Frame-based pipeline
pub struct Pipeline {
    processors: Vec<Arc<dyn FrameProcessor>>,
    input_tx: mpsc::Sender<Frame>,
    output_rx: broadcast::Receiver<Frame>,
    shutdown_tx: broadcast::Sender<()>,
}

impl Pipeline {
    /// Create new pipeline with processors
    pub fn new(processors: Vec<Arc<dyn FrameProcessor>>) -> Self {
        let (input_tx, input_rx) = mpsc::channel(100);
        let (output_tx, output_rx) = broadcast::channel(100);
        let (shutdown_tx, _) = broadcast::channel(1);

        let pipeline = Self {
            processors,
            input_tx,
            output_rx,
            shutdown_tx,
        };

        pipeline
    }

    /// Start pipeline processing
    pub async fn run(&self, mut input_rx: mpsc::Receiver<Frame>) -> Result<(), PipelineError> {
        let n = self.processors.len();
        if n == 0 {
            return Ok(());
        }

        // Create channels between processors
        let mut channels: Vec<(mpsc::Sender<Frame>, mpsc::Receiver<Frame>)> = Vec::new();
        for _ in 0..n {
            channels.push(mpsc::channel(100));
        }

        // Output channel
        let (output_tx, _) = broadcast::channel(100);

        // Spawn processor tasks
        for (i, processor) in self.processors.iter().enumerate() {
            let processor = processor.clone();
            let name = processor.name();

            // Get input receiver
            let mut rx = if i == 0 {
                input_rx
            } else {
                channels[i - 1].1
            };

            // Get output sender
            let tx = if i == n - 1 {
                output_tx.clone()
            } else {
                channels[i].0.clone()
            };

            let mut shutdown_rx = self.shutdown_tx.subscribe();

            tokio::spawn(async move {
                let span = span!(Level::INFO, "processor", name = name);
                let _guard = span.enter();

                let mut context = ProcessorContext::default();

                loop {
                    tokio::select! {
                        Some(frame) = rx.recv() => {
                            let start = std::time::Instant::now();

                            match processor.process(frame, &mut context).await {
                                Ok(output_frames) => {
                                    for output in output_frames {
                                        if let Err(e) = tx.send(output) {
                                            error!("Failed to send frame: {}", e);
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Processor {} error: {}", name, e);
                                    let _ = tx.send(Frame::Error(PipelineError {
                                        processor: name.to_string(),
                                        message: e.to_string(),
                                        recoverable: true,
                                    }));
                                }
                            }

                            debug!(
                                processor = name,
                                duration_ms = start.elapsed().as_millis() as u64,
                                "frame processed"
                            );
                        }
                        _ = shutdown_rx.recv() => {
                            info!("Processor {} shutting down", name);
                            break;
                        }
                    }
                }
            });
        }

        Ok(())
    }

    /// Send frame to pipeline
    pub async fn send(&self, frame: Frame) -> Result<(), mpsc::error::SendError<Frame>> {
        self.input_tx.send(frame).await
    }

    /// Subscribe to output frames
    pub fn subscribe(&self) -> broadcast::Receiver<Frame> {
        self.output_rx.resubscribe()
    }

    /// Shutdown pipeline
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        self.shutdown();
    }
}
```

### 5.2 builder.rs

```rust
//! Pipeline builder

use std::sync::Arc;
use voice_agent_core::FrameProcessor;
use crate::Pipeline;

/// Builder for constructing pipelines
pub struct PipelineBuilder {
    processors: Vec<Arc<dyn FrameProcessor>>,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    /// Add a processor to the pipeline
    pub fn add<P: FrameProcessor + 'static>(mut self, processor: P) -> Self {
        self.processors.push(Arc::new(processor));
        self
    }

    /// Add a processor wrapped in Arc
    pub fn add_arc(mut self, processor: Arc<dyn FrameProcessor>) -> Self {
        self.processors.push(processor);
        self
    }

    /// Build the pipeline
    pub fn build(self) -> Pipeline {
        Pipeline::new(self.processors)
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create standard voice pipeline
pub fn create_voice_pipeline(
    vad: Arc<dyn FrameProcessor>,
    stt: Arc<dyn FrameProcessor>,
    grammar: Option<Arc<dyn FrameProcessor>>,
    translation_in: Option<Arc<dyn FrameProcessor>>,
    llm: Arc<dyn FrameProcessor>,
    translation_out: Option<Arc<dyn FrameProcessor>>,
    compliance: Option<Arc<dyn FrameProcessor>>,
    pii: Option<Arc<dyn FrameProcessor>>,
    tts: Arc<dyn FrameProcessor>,
) -> Pipeline {
    let mut builder = PipelineBuilder::new()
        .add_arc(vad)
        .add_arc(stt);

    if let Some(g) = grammar {
        builder = builder.add_arc(g);
    }

    if let Some(t) = translation_in {
        builder = builder.add_arc(t);
    }

    builder = builder.add_arc(llm);

    if let Some(t) = translation_out {
        builder = builder.add_arc(t);
    }

    if let Some(c) = compliance {
        builder = builder.add_arc(c);
    }

    if let Some(p) = pii {
        builder = builder.add_arc(p);
    }

    builder.add_arc(tts).build()
}
```

---

## 6. Sentence Streaming

### 6.1 streaming/sentence_detector.rs

```rust
//! Multilingual sentence boundary detection

use std::collections::HashSet;

/// Detects sentence boundaries across languages
pub struct SentenceDetector {
    terminators: HashSet<char>,
}

impl SentenceDetector {
    pub fn new() -> Self {
        Self {
            terminators: [
                '.', '!', '?',           // English/Latin
                '।',                      // Devanagari Danda (Hindi, Sanskrit, etc.)
                '॥',                      // Devanagari Double Danda
                '।',                      // Other Indic scripts
                '؟',                      // Arabic question mark (Urdu)
                '۔',                      // Arabic full stop (Urdu)
            ]
            .into_iter()
            .collect(),
        }
    }

    /// Find sentence boundary in text
    pub fn find_boundary(&self, text: &str) -> Option<usize> {
        text.char_indices()
            .find(|(_, c)| self.terminators.contains(c))
            .map(|(i, c)| i + c.len_utf8())
    }

    /// Check if character is sentence terminator
    pub fn is_terminator(&self, c: char) -> bool {
        self.terminators.contains(&c)
    }

    /// Add custom terminator
    pub fn add_terminator(&mut self, c: char) {
        self.terminators.insert(c);
    }
}

impl Default for SentenceDetector {
    fn default() -> Self {
        Self::new()
    }
}
```

### 6.2 streaming/sentence_accumulator.rs

```rust
//! Sentence accumulation for streaming

use super::SentenceDetector;

/// Accumulates text and emits complete sentences
pub struct SentenceAccumulator {
    buffer: String,
    detector: SentenceDetector,
}

impl SentenceAccumulator {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            detector: SentenceDetector::new(),
        }
    }

    /// Add chunk and get any complete sentences
    pub fn add(&mut self, chunk: &str) -> Vec<String> {
        self.buffer.push_str(chunk);

        let mut sentences = Vec::new();

        while let Some(boundary) = self.detector.find_boundary(&self.buffer) {
            let sentence = self.buffer[..boundary].trim().to_string();
            if !sentence.is_empty() {
                sentences.push(sentence);
            }
            self.buffer = self.buffer[boundary..].to_string();
        }

        sentences
    }

    /// Flush remaining buffer
    pub fn flush(&mut self) -> Option<String> {
        let remaining = std::mem::take(&mut self.buffer);
        let trimmed = remaining.trim();
        if !trimmed.is_empty() {
            Some(trimmed.to_string())
        } else {
            None
        }
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.trim().is_empty()
    }

    /// Get current buffer contents (for debugging)
    pub fn peek(&self) -> &str {
        &self.buffer
    }
}

impl Default for SentenceAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_sentences() {
        let mut acc = SentenceAccumulator::new();

        assert_eq!(acc.add("Hello world. "), vec!["Hello world."]);
        assert_eq!(acc.add("How are you? I am fine."), vec!["How are you?", "I am fine."]);
    }

    #[test]
    fn test_hindi_sentences() {
        let mut acc = SentenceAccumulator::new();

        assert_eq!(acc.add("नमस्ते। कैसे हो?"), vec!["नमस्ते।", "कैसे हो?"]);
    }

    #[test]
    fn test_streaming() {
        let mut acc = SentenceAccumulator::new();

        assert!(acc.add("Hello wor").is_empty());
        assert_eq!(acc.add("ld. Bye."), vec!["Hello world.", "Bye."]);
    }
}
```

### 6.3 streaming/llm_tts_streamer.rs

```rust
//! LLM to TTS streaming processor

use async_trait::async_trait;
use std::sync::Arc;
use voice_agent_core::{FrameProcessor, TextToSpeech, VoiceConfig, Result};
use crate::{Frame, ProcessorContext, SentenceAccumulator};

/// Streams LLM output to TTS sentence-by-sentence
pub struct LLMToTTSStreamer {
    tts: Arc<dyn TextToSpeech>,
    voice_config: VoiceConfig,
    accumulator: parking_lot::Mutex<SentenceAccumulator>,
}

impl LLMToTTSStreamer {
    pub fn new(tts: Arc<dyn TextToSpeech>, voice_config: VoiceConfig) -> Self {
        Self {
            tts,
            voice_config,
            accumulator: parking_lot::Mutex::new(SentenceAccumulator::new()),
        }
    }
}

#[async_trait]
impl FrameProcessor for LLMToTTSStreamer {
    async fn process(
        &self,
        frame: Frame,
        context: &mut ProcessorContext,
    ) -> Result<Vec<Frame>> {
        match frame {
            Frame::LLMChunk(chunk) => {
                let sentences = self.accumulator.lock().add(&chunk);
                let mut outputs = Vec::new();

                for sentence in sentences {
                    // Synthesize each sentence immediately
                    let audio = self.tts
                        .synthesize(&sentence, &self.voice_config)
                        .await?;
                    outputs.push(Frame::AudioOutput(audio));

                    // Update context
                    context.agent_speaking = true;
                    context.tts_word_index += sentence.split_whitespace().count();
                }

                Ok(outputs)
            }

            Frame::LLMComplete(text) => {
                let mut outputs = Vec::new();

                // Flush any remaining text
                if let Some(remaining) = self.accumulator.lock().flush() {
                    let audio = self.tts
                        .synthesize(&remaining, &self.voice_config)
                        .await?;
                    outputs.push(Frame::AudioOutput(audio));
                }

                outputs.push(Frame::EndOfTurn);
                context.agent_speaking = false;

                Ok(outputs)
            }

            // Pass through other frames
            other => Ok(vec![other]),
        }
    }

    fn name(&self) -> &'static str {
        "llm_to_tts_streamer"
    }
}
```

---

## 7. Interrupt Handling

### 7.1 interrupt/handler.rs

```rust
//! Interrupt (barge-in) handling

use std::time::{Duration, Instant};
use voice_agent_core::AudioFrame;

/// Interrupt detection modes
#[derive(Debug, Clone, Copy)]
pub enum InterruptMode {
    /// Stop at sentence boundary
    SentenceBoundary,
    /// Stop immediately (may clip)
    Immediate,
    /// Stop at word boundary
    WordBoundary,
}

/// Interrupt configuration
#[derive(Debug, Clone)]
pub struct InterruptConfig {
    pub mode: InterruptMode,
    pub vad_sensitivity: f32,
    pub min_speech_duration_ms: u64,
    pub silence_timeout_ms: u64,
}

impl Default for InterruptConfig {
    fn default() -> Self {
        Self {
            mode: InterruptMode::SentenceBoundary,
            vad_sensitivity: 0.5,
            min_speech_duration_ms: 200,
            silence_timeout_ms: 500,
        }
    }
}

/// Interrupt handler state
enum InterruptState {
    Idle,
    AgentSpeaking {
        start_time: Instant,
        current_sentence: String,
    },
    UserInterrupting {
        speech_start: Instant,
        accumulated_duration: Duration,
    },
}

/// Handles interrupt detection during agent speech
pub struct InterruptHandler {
    config: InterruptConfig,
    state: InterruptState,
}

/// Action to take on interrupt
pub enum InterruptAction {
    /// Stop immediately
    StopNow,
    /// Stop at sentence boundary
    StopAtSentence,
    /// Stop at word boundary
    StopAtWord,
    /// No action needed
    None,
}

impl InterruptHandler {
    pub fn new(config: InterruptConfig) -> Self {
        Self {
            config,
            state: InterruptState::Idle,
        }
    }

    /// Mark agent as speaking
    pub fn agent_started_speaking(&mut self) {
        self.state = InterruptState::AgentSpeaking {
            start_time: Instant::now(),
            current_sentence: String::new(),
        };
    }

    /// Mark agent as done speaking
    pub fn agent_stopped_speaking(&mut self) {
        self.state = InterruptState::Idle;
    }

    /// Process audio during agent speech
    ///
    /// Returns interrupt action if user speech detected
    pub fn process_audio(&mut self, is_speech: bool) -> InterruptAction {
        match (&mut self.state, is_speech) {
            // Agent speaking, user starts talking
            (InterruptState::AgentSpeaking { .. }, true) => {
                self.state = InterruptState::UserInterrupting {
                    speech_start: Instant::now(),
                    accumulated_duration: Duration::ZERO,
                };
                InterruptAction::None // Wait for minimum duration
            }

            // User continues speaking
            (InterruptState::UserInterrupting { speech_start, accumulated_duration }, true) => {
                *accumulated_duration = speech_start.elapsed();

                if *accumulated_duration >= Duration::from_millis(self.config.min_speech_duration_ms) {
                    match self.config.mode {
                        InterruptMode::Immediate => InterruptAction::StopNow,
                        InterruptMode::SentenceBoundary => InterruptAction::StopAtSentence,
                        InterruptMode::WordBoundary => InterruptAction::StopAtWord,
                    }
                } else {
                    InterruptAction::None
                }
            }

            // User stopped speaking (false positive)
            (InterruptState::UserInterrupting { .. }, false) => {
                self.state = InterruptState::AgentSpeaking {
                    start_time: Instant::now(),
                    current_sentence: String::new(),
                };
                InterruptAction::None
            }

            _ => InterruptAction::None,
        }
    }

    /// Check if currently handling interrupt
    pub fn is_interrupting(&self) -> bool {
        matches!(self.state, InterruptState::UserInterrupting { .. })
    }
}
```

---

## 8. Checklist

### 8.1 Frame System
- [ ] Create `frame.rs` with all Frame variants
- [ ] Create `processor_context.rs`
- [ ] Add PipelineError and MetricsEvent types

### 8.2 Pipeline Orchestrator
- [ ] Create `pipeline_v2.rs` with channel-based orchestration
- [ ] Create `builder.rs` for pipeline construction
- [ ] Add shutdown and lifecycle handling
- [ ] Add tracing spans per processor

### 8.3 Sentence Streaming
- [ ] Create `SentenceDetector` with Indic terminators
- [ ] Create `SentenceAccumulator`
- [ ] Create `LLMToTTSStreamer` processor
- [ ] Add tests for multilingual sentences

### 8.4 Interrupt Handling
- [ ] Create `InterruptConfig` with modes
- [ ] Create `InterruptHandler` state machine
- [ ] Integrate with VAD processor
- [ ] Add tests for interrupt scenarios

### 8.5 Standard Processors
- [ ] Create `VadProcessor` wrapper
- [ ] Create `SttProcessor` wrapper
- [ ] Create `TtsProcessor` wrapper
- [ ] Create text processing wrappers

### 8.6 Integration
- [ ] Create `create_voice_pipeline()` factory
- [ ] Migrate from old `VoicePipeline`
- [ ] Add integration tests
- [ ] Add benchmarks for latency

---

*This phase enables the key innovation: sentence-by-sentence streaming for sub-800ms latency.*
