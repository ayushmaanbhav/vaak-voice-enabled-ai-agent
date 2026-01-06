# Voice Agent Backend - Deep Dive Analysis Report

**Date:** December 30, 2025
**Scope:** voice-agent/backend Rust crates + configuration files
**Analysis Method:** 5 parallel investigation agents covering constants, configs, traits, types, and wiring

---

## Executive Summary

The voice-agent backend has a **well-designed architecture** with centralized configuration management and proper trait abstractions. However, several critical issues were discovered:

| Category | Status | Critical Issues |
|----------|--------|----------------|
| Constants Centralization | PARTIAL | 15+ constant types duplicated across files |
| Config Struct Ownership | GOOD | Single source in `config` crate |
| Config Loading | **BROKEN** | `load_settings()` never called in main.rs |
| Trait Implementation | GOOD | 13/14 traits fully implemented |
| Type Definitions | NEEDS WORK | 5 critical duplicate types across crates |

---

## 1. Constants and Magic Values Analysis

### 1.1 Critical Duplications Found

#### PCM Audio Conversion (INCONSISTENT - HIGH PRIORITY)
Duplicated across 5+ files with **inconsistent values**:
- Value `32768.0` used for division (normalize PCM16 to f32)
- Value `32767.0` used for multiplication (to_pcm16 conversion)

| File | Issue |
|------|-------|
| `core/src/audio.rs:190,203` | Uses 32768.0 |
| `transport/src/codec.rs:81,203,226` | Mixed usage |
| `server/src/websocket.rs:174,306,430` | Mixed usage |

#### Business Constants - Interest Rates
Despite centralization in `config/src/constants.rs`, rates are still hardcoded in 20+ locations:

| Rate | Centralized Location | Duplicate Count | Key Duplicates |
|------|---------------------|-----------------|----------------|
| 11.5% (Tier 1) | constants.rs:18 | 4 | product.rs, adaptation.rs |
| 10.5% (Tier 2) | constants.rs:22 | 10+ | prompt.rs, persuasion.rs, agent.rs |
| 9.5% (Tier 3) | constants.rs:25 | 8+ | persuasion.rs, agent.rs |
| 18.0-24.0% (NBFC) | constants.rs:31-32 | 15+ | scattered everywhere |

#### Timeout Values (SCATTERED)
```
30s timeout: llm/backend.rs, transport/webrtc.rs (NOT using constants)
60s timeout: llm/claude.rs (NOT using constants)
WebRTC ICE timeouts: HARDCODED in webrtc.rs (5s, 25s, 2s - NOT in constants!)
```

#### Context Token Limit CONFLICT
```
config/src/constants.rs:117  -> MAX_CONTEXT_TOKENS: 2048
rag/src/context.rs:248       -> max_context_tokens: 32768
```
**CRITICAL:** 16x difference between constants definition and actual usage!

### 1.2 Missing from Constants Module
- 14K purity factor (0.585) - 22K and 18K exist but 14K missing
- Turn detection timing values (200-1000ms range)
- WebRTC ICE timeout configuration
- VAD frame count constants (min_speech_frames, min_silence_frames)

---

## 2. Configuration Architecture Analysis

### 2.1 Config Crate Structure (WELL ORGANIZED)

```
config/src/
├── lib.rs          # Re-exports all public types
├── settings.rs     # Master Settings struct (44 config types)
├── agent.rs        # AgentConfig, PersonaConfig, LlmConfig, MemoryConfig
├── pipeline.rs     # PipelineConfig, VadConfig, SttConfig, TtsConfig
├── gold_loan.rs    # GoldLoanConfig, TieredRates, PurityFactors
├── constants.rs    # Centralized constants (P1 FIX)
├── domain.rs       # DomainConfig, DomainConfigManager
├── branch.rs       # BranchConfig, Branch
├── product.rs      # ProductConfig, ProductVariant
├── competitor.rs   # CompetitorConfig, Competitor
└── prompts.rs      # PromptTemplates, SystemPrompt
```

### 2.2 Config Loading - CRITICAL BUG

**Location:** `server/src/main.rs:14`

```rust
// CURRENT (BROKEN):
let config = Settings::default();

// SHOULD BE:
let config = load_settings(env.as_deref())?;
```

**Impact:**
- All configuration files (`config/default.yaml`, `config/production.yaml`) are **IGNORED**
- Environment variables (`VOICE_AGENT__*`) have no effect
- System runs on hardcoded Rust defaults only

### 2.3 Config Files Present (But Unused)

| File | Purpose | Status |
|------|---------|--------|
| `config/default.yaml` | Comprehensive defaults | EXISTS but UNUSED |
| `config/production.yaml` | Production overrides | EXISTS but UNUSED |
| `config/domain.yaml` | Domain-specific config | LOADED CORRECTLY |

### 2.4 Local Config Definitions (Problematic)

| File | Struct | Issue |
|------|--------|-------|
| `llm/src/backend.rs` | `LlmConfig` | Shadows central `config::LlmConfig` |
| `llm/src/factory.rs` | `LlmProviderConfig` | Local definition |
| `llm/src/claude.rs` | `ClaudeConfig` | Should use central config |
| `llm/src/speculative.rs` | `SpeculativeConfig` | Local definition |

---

## 3. Trait Implementation Analysis

### 3.1 Core Traits Status (14 Total)

| Trait | Module | Implemented | Adapter |
|-------|--------|-------------|---------|
| SpeechToText | speech.rs | YES | SttAdapter |
| TextToSpeech | speech.rs | YES | TtsAdapter |
| VoiceActivityDetector | speech.rs | YES | MagicNetVAD wrapper |
| AudioProcessor | speech.rs | PARTIAL | Passthrough, NoiseSuppressor |
| LanguageModel | llm.rs | YES | LanguageModelAdapter |
| Retriever | retriever.rs | YES | EnhancedRetriever |
| Tool | tool.rs | YES | Multiple domain tools |
| FrameProcessor | pipeline.rs | YES | 4 implementations |
| ConversationFSM | fsm.rs | YES | StageManagerAdapter |
| GrammarCorrector | text_processing.rs | YES | LLMGrammarCorrector, Noop |
| Translator | text_processing.rs | YES | CandleIndicTrans2, ONNX, Noop |
| PIIRedactor | text_processing.rs | YES | HybridPIIDetector |
| ComplianceChecker | text_processing.rs | YES | RuleBasedChecker, Noop |
| TextProcessor | text_processing.rs | YES | TextProcessingPipeline |

### 3.2 AudioProcessor - Incomplete Implementation

**Status:** Trait defined but missing implementations

| Component | Status |
|-----------|--------|
| Passthrough | IMPLEMENTED |
| Noise Suppression | IMPLEMENTED (P2-1 FIX) |
| AEC (Echo Cancellation) | DEFERRED - browser-side |
| AGC (Automatic Gain Control) | DEFERRED - browser-side |

### 3.3 Traits Outside Core (Organization Issue)

| Trait | Location | Should Be |
|-------|----------|-----------|
| `ToolExecutor` | tools/src/registry.rs | core/src/traits/tool.rs |
| `LlmBackend` | llm/src/backend.rs | Internal only (OK) |

---

## 4. Type Definitions Analysis

### 4.1 Critical Duplicate Types

#### Message Struct (CRITICAL)
```rust
// core/src/llm_types.rs:103 - COMPLETE
pub struct Message {
    pub role: Role,
    pub content: String,
    pub name: Option<String>,        // MISSING in llm version
    pub tool_call_id: Option<String>, // MISSING in llm version
}

// llm/src/prompt.rs:37 - INCOMPLETE DUPLICATE
pub struct Message {
    pub role: Role,
    pub content: String,
}
```

#### Role Enum (CRITICAL)
Identical enum defined in both:
- `core/src/llm_types.rs:159`
- `llm/src/prompt.rs:16`

#### ConversationContext (INCOMPATIBLE)
```rust
// core/src/traits/retriever.rs:223
pub struct ConversationContext {
    pub recent_turns: Vec<ConversationTurn>,
    pub intent: Option<String>,
    pub stage: ConversationStage,
    pub entities: HashMap<String, Value>,
}

// rag/src/agentic.rs:55 - DIFFERENT STRUCTURE
pub struct ConversationContext {
    pub summary: String,
    pub stage: Stage,
    pub entities: Vec<(String, String)>, // Different type!
}
```

Adapter workaround exists in `rag/src/adapter.rs:11-15`:
```rust
use voice_agent_core::ConversationContext as CoreContext;
use crate::agentic::ConversationContext as RagContext;
```

#### Document Struct (INCOMPATIBLE)
```rust
// core/src/traits/retriever.rs:177
pub struct Document {
    pub content: String,  // Field name
    ...
}

// rag/src/vector_store.rs:67
pub struct Document {
    pub text: String,     // Different field name!
    ...
}
```

#### Language Enum
- `core/src/language.rs:11` - 23 Indian languages + English
- `pipeline/src/tts/g2p.rs:71` - Hindi, English, Hinglish only

**Note:** G2P version is intentional limited scope but should be documented.

### 4.2 Duplicate Summary

| Type | Severity | Core Location | Duplicate Location |
|------|----------|---------------|-------------------|
| Message | CRITICAL | core/llm_types.rs | llm/prompt.rs |
| Role | CRITICAL | core/llm_types.rs | llm/prompt.rs |
| ConversationContext | CRITICAL | core/retriever.rs | rag/agentic.rs |
| Document | HIGH | core/retriever.rs | rag/vector_store.rs |
| Language | MEDIUM | core/language.rs | pipeline/g2p.rs |
| ErrorCode | MEDIUM | core/error.rs | core/tool.rs |

---

## 5. Configuration Wiring Analysis

### 5.1 Proper Wiring (Working)

| Component | Config Source | Wiring Method |
|-----------|--------------|---------------|
| Rate Limiting | Settings.server.rate_limit | Via AppState |
| RAG | Settings.rag | If enabled flag |
| Persistence | Settings.persistence | If enabled flag |
| Domain Config | domain.yaml | DomainConfigManager |
| Tools | DomainConfig.gold_loan | Explicit passing |

### 5.2 Broken/Missing Wiring

| Component | Issue |
|-----------|-------|
| Main Settings | Uses `Settings::default()` not `load_settings()` |
| Environment Config | `env` field initialized as `None` |
| Hot Reload | Exists but unusable without proper env |
| Model Paths | Validates but models still required at runtime |

---

## 6. File References

### Critical Files Requiring Changes

| Priority | File Path | Issue |
|----------|-----------|-------|
| P0 | `server/src/main.rs:14` | Config not loaded from files |
| P0 | `rag/src/context.rs:248` | Wrong context token limit |
| P1 | `llm/src/prompt.rs` | Duplicate Message, Role types |
| P1 | `rag/src/agentic.rs` | Duplicate ConversationContext |
| P1 | `core/src/audio.rs` | Hardcoded PCM constants |
| P1 | `transport/src/webrtc.rs` | Hardcoded ICE timeouts |
| P2 | `agent/src/persuasion.rs` | Hardcoded interest rates in strings |
| P2 | `pipeline/src/turn_detection/hybrid.rs` | Hardcoded timing values |
| P2 | `rag/src/vector_store.rs` | Incompatible Document struct |

### Config Crate Files (Reference)

```
voice-agent/backend/crates/config/src/
├── lib.rs            # Main exports
├── settings.rs       # Settings + load_settings()
├── constants.rs      # Centralized constants
├── domain.rs         # DomainConfigManager
├── agent.rs          # Agent/LLM config
├── pipeline.rs       # Audio pipeline config
├── gold_loan.rs      # Business logic config
├── branch.rs         # Branch/location config
├── product.rs        # Product variants
├── competitor.rs     # Competitor analysis
└── prompts.rs        # Conversation templates
```

---

## 7. Metrics Summary

| Metric | Value |
|--------|-------|
| Total Rust Files Analyzed | 85+ |
| Core Traits Defined | 14 |
| Traits Fully Implemented | 13 |
| Config Struct Types | 44 |
| Critical Duplicate Types | 5 |
| Hardcoded Constant Locations | 50+ |
| Files Needing Updates | 25+ |

---

## Appendix: Agent Investigation Summaries

### Agent 1: Constants Duplication
Found 15 categories of duplicated/scattered constants including PCM conversion values, interest rates, timeout values, and confidence thresholds.

### Agent 2: Config Struct Centralization
Confirmed config crate is well-organized with proper re-exports, but found LLM crate defines shadow configs.

### Agent 3: Trait Implementation Wiring
All 14 core traits have implementations. AudioProcessor is partial (missing AEC/AGC). Adapter pattern used correctly.

### Agent 4: Duplicate Type Definitions
Found 5 critical type duplicates: Message, Role, ConversationContext, Document, Language. Some require conversion adapters.

### Agent 5: Config Usage and Wiring
Discovered main.rs uses `Settings::default()` instead of loading from YAML files - making config files dead code.
