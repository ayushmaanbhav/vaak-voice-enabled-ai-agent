# Architecture vs Implementation: Comprehensive Gap Analysis

> **Status:** CRITICAL - Documentation significantly out of sync with implementation
> **Analysis Date:** 2024-12-28
> **Scope:** Full ARCHITECTURE_v2.md vs voice-agent-rust/ codebase
> **Reviewed By:** 8 specialized analysis agents

---

## Executive Summary

The **ARCHITECTURE_v2.md** document describes an **aspirational design** that has been **partially implemented**. This analysis reveals significant gaps across all major components.

### Implementation Completeness by Component

| Component | Documented | Implemented | Gap Severity |
|-----------|------------|-------------|--------------|
| **Core Traits** | 9 traits | 0 as documented | CRITICAL |
| **Pipeline Architecture** | Frame-based processors | Simplified orchestrator | CRITICAL |
| **Text Processing** | Full pipeline | None | CRITICAL |
| **RAG System** | 5-step agentic + timing | Partial (3 steps) | HIGH |
| **Core Types** | 8 types | 4 with discrepancies | HIGH |
| **Domain Config** | TOML/YAML based | All hardcoded | CRITICAL |
| **Personalization** | Full engine | 15% basic | CRITICAL |
| **Fallback Patterns** | 4 model fallbacks | 1 wired up | HIGH |
| **Crate Structure** | 11 crates | 9 crates (different) | MEDIUM |

**Overall Architecture Alignment: ~25%**

---

## 1. Core Traits Gap Analysis

### ARCHITECTURE_v2.md Defines 9 Traits (lines 340-460)

| Trait | Doc Location | Exists? | Actual Name | Signature Match |
|-------|--------------|---------|-------------|-----------------|
| `SpeechToText` | crates/core/src/traits/ | NO | `SttBackend` | Different methods |
| `TextToSpeech` | crates/core/src/traits/ | NO | `TtsBackend` | Different methods |
| `LanguageModel` | crates/core/src/traits/ | NO | `LlmBackend` | Different methods |
| `Retriever` | crates/core/src/traits/ | NO | Concrete struct | No trait |
| `GrammarCorrector` | crates/core/src/traits/ | NO | - | NOT IMPLEMENTED |
| `Translator` | crates/core/src/traits/ | NO | - | NOT IMPLEMENTED |
| `PIIRedactor` | crates/core/src/traits/ | NO | - | NOT IMPLEMENTED |
| `ComplianceChecker` | crates/core/src/traits/ | NO | - | NOT IMPLEMENTED |
| `FrameProcessor` | crates/pipeline/src/ | NO | - | NOT IMPLEMENTED |

### Traits That Actually Exist (Not Documented)

| Trait | Location | Purpose |
|-------|----------|---------|
| `SttBackend` | pipeline/src/stt/mod.rs | Speech-to-text |
| `TtsBackend` | pipeline/src/tts/mod.rs | Text-to-speech |
| `LlmBackend` | llm/src/backend.rs | LLM inference |
| `VadEngine` | pipeline/src/vad/mod.rs | Voice activity detection |
| `Tool` | tools/src/mcp.rs | Tool execution |
| `Transport` | transport/src/traits.rs | Network transport |
| `AudioSource` | transport/src/traits.rs | Audio input |
| `AudioSink` | transport/src/traits.rs | Audio output |

### Critical Finding
The documented `crates/core/src/traits/mod.rs` **does not exist**. The core crate only contains:
- `audio.rs`, `conversation.rs`, `customer.rs`, `error.rs`, `lib.rs`, `transcript.rs`

---

## 2. Pipeline Architecture Gap Analysis

### Documented Architecture (lines 558-896)

```
Frame → Processor1 → Frame → Processor2 → Frame → Processor3
         (tokio task)        (tokio task)        (tokio task)
         via channel         via channel         via channel
```

### Actual Architecture

```
AudioFrame → VoicePipeline.process_audio()
              ├─ VAD (sequential)
              ├─ TurnDetection (sequential)
              ├─ STT (sequential)
              └─ emit PipelineEvent
```

### Frame Enum Comparison

| Documented Frame Variant | Implemented? | Notes |
|-------------------------|--------------|-------|
| AudioInput(AudioFrame) | NO | - |
| AudioOutput(AudioFrame) | NO | - |
| TranscriptPartial | YES | As `PipelineEvent::PartialTranscript` |
| TranscriptFinal | YES | As `PipelineEvent::FinalTranscript` |
| GrammarCorrected | NO | Text processing not implemented |
| Translated | NO | Translation not implemented |
| ComplianceChecked | NO | Compliance not implemented |
| PIIRedacted | NO | PII not implemented |
| LLMChunk | NO | - |
| LLMComplete | NO | - |
| ToolCall | NO | Exists elsewhere, not in pipeline |
| ToolResult | NO | Exists elsewhere, not in pipeline |
| UserSpeaking | PARTIAL | In TurnState, not Frame |
| UserSilence | NO | - |
| BargeIn | YES | As `PipelineEvent::BargeIn` |
| EndOfTurn | NO | - |
| StateChange | NO | - |
| Error | YES | As `PipelineEvent::Error` |
| Metrics | NO | - |

### Missing Pipeline Components

1. **FrameProcessor trait** - Does not exist
2. **Processor chain orchestration** - Uses monolithic VoicePipeline instead
3. **Channel-based inter-processor communication** - Not implemented
4. **LLMToTTSStreamer** - Not implemented as documented
5. **InterruptHandler** with modes - Simplified barge-in only

---

## 3. Text Processing Pipeline Gap Analysis

### Status: COMPLETELY MISSING

The architecture describes a comprehensive text processing pipeline (lines 900-1565):

```
INPUT:  STT → Grammar → Translate(IN→EN) → LLM
OUTPUT: LLM → Translate(EN→IN) → Compliance → PII → Simplify → TTS
```

### Reality

```
INPUT:  STT → Agent (direct)
OUTPUT: LLM → TTS (direct)
```

### Component Status

| Component | Documented | Implemented | Location |
|-----------|------------|-------------|----------|
| `text_processing/` crate | YES | NO | Does not exist |
| `LLMGrammarCorrector` | YES | NO | - |
| `DomainContext` | YES | NO | - |
| `IndicTranslator` | YES | NO | - |
| `HybridPIIDetector` | YES | NO | - |
| `RuleBasedComplianceChecker` | YES | NO | - |
| `text_processing.toml` | YES | NO | - |
| `compliance.toml` | YES | NO | - |

### Impact

1. **No Grammar Correction** - STT errors pass directly to LLM
2. **No Translation** - Cannot use "Translate-Think-Translate" pattern
3. **No PII Protection** - Aadhaar, PAN, phone numbers unprotected
4. **No Compliance** - Bank regulatory requirements not enforced
5. **No Simplification** - TTS may struggle with complex text

---

## 4. RAG Implementation Gap Analysis

### 5-Step Agentic RAG Workflow

| Step | Documented | Implemented | Notes |
|------|------------|-------------|-------|
| 1. Intent Classification | YES | PARTIAL | No RAG integration |
| 2. Parallel Retrieval | YES | MOSTLY | No stage-aware filter |
| 3. Sufficiency Check | LLM-based | Heuristic only | No semantic evaluation |
| 4. Reranking | YES | YES | Fully implemented |
| 5. Context Sizing | Per-stage | NO | Hardcoded budgets |

### RAG Timing Strategies

| Strategy | Documented | Implemented | Notes |
|----------|------------|-------------|-------|
| `Sequential` | YES | Default behavior | Not explicit strategy |
| `PrefetchAsync` | YES | PARTIAL | Method exists, not wired to VAD |
| `ParallelInject` | YES | NO | - |
| `RAGTimingMode` enum | YES | NO | - |

### Missing RAG Features

1. **Stage-aware context sizing** - All stages use same 4096 token budget
2. **VAD → Prefetch integration** - prefetch() exists but never called from VAD
3. **LLM sufficiency checking** - Uses simple heuristics instead
4. **Intent-driven retrieval** - Intent detection not connected to document filtering

---

## 5. Core Types Gap Analysis

### AudioFrame

| Field | Documented | Actual |
|-------|------------|--------|
| data | `Vec<i16>` | `samples: Arc<[f32]>` |
| sample_rate | `u32` | `SampleRate` enum |
| channels | `u8` | `Channels` enum |
| timestamp_ms | `u64` | `timestamp: Instant` |
| - | - | `sequence: u64` (NEW) |
| - | - | `duration: Duration` (NEW) |
| - | - | `vad_probability: Option<f32>` (NEW) |
| - | - | `is_speech: bool` (NEW) |
| - | - | `energy_db: f32` (NEW) |

### TranscriptFrame → TranscriptResult

| Documented | Actual |
|------------|--------|
| `TranscriptFrame` | `TranscriptResult` (different name) |
| `language: Language` | `language: Option<String>` |
| `words: Vec<WordTiming>` | `words: Vec<WordTimestamp>` |

### Language Enum

| Documented | Actual |
|------------|--------|
| 22 Indian languages | 3 variants only (Hindi, English, Hinglish) |
| Location: `crates/core` | Location: `crates/pipeline/src/tts/g2p.rs` |

### VoiceConfig

| Documented | Actual |
|------------|--------|
| Full struct | **DOES NOT EXIST** |

### ConversationState vs ConversationStage

The codebase has **two different enums** instead of one:

**ConversationState** (agent/conversation.rs):
```rust
pub enum ConversationState {
    Active,
    Paused,
    Ended,
}
```

**ConversationStage** (core/conversation.rs):
```rust
pub enum ConversationStage {
    Greeting,
    Discovery,
    Qualification,    // Not "NeedsAnalysis"
    Presentation,     // Not "Pitch"
    ObjectionHandling,
    Closing,
    Farewell,         // NEW
}
```

### CustomerSegment

| Documented | Actual |
|------------|--------|
| `P1HighValue` | `HighValue` |
| `P2TrustSeeker` | `TrustSeeker` |
| `P3Shakti` | `Women` |
| `P4YoungPro` | `Professional` |
| `Unknown` | NOT present |
| - | `FirstTime` (NEW) |
| - | `PriceSensitive` (NEW) |

---

## 6. Domain Configuration Gap Analysis

### Design Principle Violation

**Documented (line 69-75):**
> "CONFIGURABILITY OVER CODE"
> "Domain logic lives in TOML/YAML, not Rust"
> "New vertical = new config directory, zero code changes"

**Reality:** All domain logic is **hardcoded in Rust**

### Domain Structure Comparison

| Documented | Actual |
|------------|--------|
| `domains/gold_loan/knowledge/` | Does not exist |
| `domains/gold_loan/prompts/` | Prompts hardcoded in Rust |
| `domains/gold_loan/segments.toml` | CustomerSegment enum hardcoded |
| `domains/gold_loan/tools.toml` | Tools hardcoded in Rust |
| `domains/gold_loan/compliance.toml` | Does not exist |
| `domains/gold_loan/experiments.toml` | Does not exist |

### What's Actually Configurable

| Component | Configurable? | Location |
|-----------|--------------|----------|
| Branch data | YES | data/branches.json |
| Vocabulary | YES | data/gold_loan_vocab.txt |
| Customer segments | NO | Hardcoded enum |
| Prompts | NO | Hardcoded strings |
| Tools | NO | Hardcoded structs |
| Business logic | NO | gold_loan.rs defaults |
| Compliance rules | NO | Not implemented |
| Experiments | NO | Not implemented |

**Configurability Score: ~5%**

---

## 7. Personalization Engine Gap Analysis

### Documented Components (lines 285-290)

| Component | Documented | Implemented |
|-----------|------------|-------------|
| `personalization/` crate | YES | NO (does not exist) |
| `segments.rs` | YES | PARTIAL (in core/customer.rs) |
| `strategy.rs` | YES | NO |
| `disclosure.rs` | YES | NO |

### Feature Status

| Feature | Documented | Implemented |
|---------|------------|-------------|
| Segment detection | YES | PARTIAL (basic heuristics) |
| Segment-specific messaging | YES | Defined but never used |
| Dynamic warmth adjustment | YES | Static PersonaConfig only |
| Natural AI disclosure | YES | Static greeting only |
| Psychology guardrails | YES | NO |
| Persuasion strategy | YES | NO |

### Evidence of Partial Implementation

```rust
// Exists but never called:
CustomerSegment::TrustSeeker.key_messages() => [...]
CustomerSegment::TrustSeeker.suggested_warmth() => 0.95

// PromptBuilder accepts segment but doesn't use key_messages:
.with_customer(name, segment, history) // segment added as static text only
```

---

## 8. Fallback Patterns Gap Analysis

### Documented Fallbacks (lines 226-235)

| Primary | Fallback | Status |
|---------|----------|--------|
| IndicConformer | Whisper | NOT WIRED UP |
| IndicF5 | Piper | NOT WIRED UP |
| IndicTrans2 ONNX | gRPC | NOT IMPLEMENTED |
| Qwen2.5 | Ollama | Single backend only |

### Actually Implemented Fallbacks

| Component | Fallback | Location |
|-----------|----------|----------|
| Silero VAD | Energy-based | agent/voice_session.rs:190-208 |
| LLM summarization | Simple summary | agent/memory.rs:297-306 |
| LLM backend | Optional (None) | agent/agent.rs:134-138 |

### Critical Gap

**STT and TTS have no runtime fallbacks** despite both engines existing:
- `SttEngine::Whisper` and `SttEngine::IndicConformer` exist
- `TtsEngine::Piper` and `TtsEngine::IndicF5` exist
- But: selection is config-based, not runtime fallback

---

## 9. Crate Structure Gap Analysis

### Documented vs Actual

| Crate | Documented | Exists | Notes |
|-------|------------|--------|-------|
| core | YES | YES | Missing traits module |
| config | YES | YES | - |
| pipeline | YES | YES | Merged speech functionality |
| speech | YES | NO | Merged into pipeline |
| text_processing | YES | NO | Not implemented |
| rag | YES | YES | - |
| agent | YES | YES | - |
| personalization | YES | NO | Partial in core/agent |
| llm | YES | YES | - |
| experiments | YES | NO | Not implemented |
| server | YES | YES | - |
| tools | NO | YES | NEW (not documented) |
| transport | NO | YES | NEW (not documented) |

---

## 10. Recommendations by Priority

### P0: Critical - Update or Implement

1. **Sync documentation with reality** - Update ARCHITECTURE_v2.md to reflect actual implementation
2. **Implement SlotExtractor trait** - For pluggable multilingual support
3. **Implement Translator trait** - For Translate-Think-Translate approach
4. **Wire up STT/TTS fallbacks** - Critical for production reliability

### P1: High - Before Production

5. **Implement stage-aware context sizing** - Different RAG budgets per stage
6. **Wire up VAD → Prefetch** - Reduce latency with predictive retrieval
7. **Add compliance checking** - Bank regulatory requirement
8. **Add PII detection** - Data protection requirement

### P2: Medium - Post-Launch

9. **Implement Frame-based pipeline** - Or document why simplified is preferred
10. **Add grammar correction** - Improve STT quality
11. **Make domain config YAML-based** - Support new verticals without code changes
12. **Implement personalization engine** - Use existing segment data

### P3: Low - Enhancement

13. **Add experiment framework** - A/B testing capability
14. **Align type/enum names** - Match P1/P2/P3/P4 naming
15. **Complete Language enum** - Support all 22 languages in core

---

## Appendix A: Files That Don't Exist (Referenced in Docs)

```
crates/core/src/traits/mod.rs
crates/text_processing/
crates/personalization/
crates/experiments/
crates/speech/
domains/
config/default.yaml
```

## Appendix B: Undocumented Files/Crates

```
crates/tools/
crates/transport/
data/branches.json
data/gold_loan_vocab.txt
```

---

*This analysis was performed by 8 specialized code review agents examining ~15,000 lines of Rust code against the ARCHITECTURE_v2.md specification.*
