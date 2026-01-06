# Voice Agent Documentation Index

> Pure Rust Voice Agent for Gold Loan Sales
>
> **Version:** 2.0 (Post-Gap Analysis)
> **Updated:** December 2025

---

## Overview

This documentation describes a production-grade, pure Rust voice agent for Kotak Mahindra Bank's gold loan acquisition strategy. The system targets:

- **Latency:** <500ms end-to-end response time
- **Languages:** 22 Indian languages via IndicConformer/IndicF5
- **Scale:** 1000+ concurrent conversations
- **Domain:** Gold loan sales and customer service

---

## Documentation Map

```
docs/
├── INDEX.md                          # This file
├── ARCHITECTURE_v2.md                # High-level architecture
├── rust-ecosystem.md                 # Library decisions
├── core-traits.md                    # Core trait definitions
│
├── architecture/                     # Component documentation
│   ├── pipeline/
│   │   ├── audio-pipeline.md         # VAD, STT, TTS streaming
│   │   └── optimized-pipeline.md     # Gap implementations, research
│   ├── interfaces/
│   │   └── mcp-tools.md              # MCP tool interface
│   ├── agent/
│   │   └── agent-framework.md        # Stage-based agent
│   ├── rag/
│   │   └── agentic-rag-strategy.md   # RAG architecture
│   └── personalization/
│       └── customer-profiling.md     # Customer segmentation
│
├── deployment/
│   └── scaling-strategy.md           # Production deployment
│
├── experiments/
│   └── ab-testing-framework.md       # A/B testing
│
└── troubleshooting/                  # Issue reports & fixes
    ├── DEEP_DIVE_REPORT.md
    ├── FIX_PLAN.md
    └── ISSUES_SUMMARY.md
```

---

## Quick Start

### 1. Understand the Architecture

Start with [ARCHITECTURE_v2.md](./ARCHITECTURE_v2.md) for the high-level design.

### 2. Review Gap Implementations

The [optimized-pipeline.md](./architecture/pipeline/optimized-pipeline.md) documents all 9 gap implementations:

| Gap | Solution | Impact |
|-----|----------|--------|
| 1. Turn Detection | Semantic HybridTurnDetector | -200-600ms latency |
| 2. VAD | MagicNet-inspired 10ms frames | <15ms detection |
| 3. STT Accuracy | Enhanced decoder + hallucination prevention | +10% WER |
| 4. Pipeline | Low-latency orchestrator | <700ms E2E |
| 5. Reranking | Early-exit cross-encoder | 2-3.5x speedup |
| 6. LLM | Speculative SLM/LLM execution | 2-5x speedup |
| 7. Tools | MCP-compatible interface | Industry standard |
| 8. TTS | Word-level barge-in | Natural interruptions |
| 9. Agent | Stage-based framework | Production-ready |

### 3. Explore Component Details

| Component | Documentation |
|-----------|---------------|
| Audio Pipeline | [audio-pipeline.md](./architecture/pipeline/audio-pipeline.md) |
| Tool Interface | [mcp-tools.md](./architecture/interfaces/mcp-tools.md) |
| Agent Logic | [agent-framework.md](./architecture/agent/agent-framework.md) |
| RAG System | [agentic-rag-strategy.md](./architecture/rag/agentic-rag-strategy.md) |
| Rust Libraries | [rust-ecosystem.md](./rust-ecosystem.md) |

---

## Key Design Decisions

### 1. Cascaded vs End-to-End Architecture

**Decision:** Cascaded (STT → LLM → TTS) with speculative execution

**Rationale:**
- More controllable and debuggable
- Existing high-quality components (IndicConformer, IndicF5)
- Speculative execution closes latency gap with E2E

### 2. Turn Detection Strategy

**Decision:** Hybrid VAD + Semantic (Turnsense SmolLM2-135M)

**Rationale:**
- Reduces premature cutoffs by 200-600ms
- Small model (<135M params) adds only ~30ms
- Falls back to VAD silence for robustness

### 3. LLM Execution Strategy

**Decision:** Speculative SLM/LLM with EAGLE-style draft-verify

**Rationale:**
- SLM (Qwen 1.5B) handles 70% of queries directly
- LLM (Qwen 7B) for complex cases
- Draft-verify reduces LLM calls by 60%

### 4. Tool Interface

**Decision:** MCP (Model Context Protocol)

**Rationale:**
- Industry standard (Anthropic, OpenAI, Google)
- JSON Schema validation
- Future-proof for new models

### 5. Memory Architecture

**Decision:** Hierarchical (Working + Episodic + Semantic)

**Rationale:**
- Working memory for immediate context (8 turns)
- Episodic summaries for long conversations
- Semantic memory for key facts

---

## Latency Budget

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     TARGET: 450-550ms END-TO-END                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Stage                      │ Target   │ Technique                         │
│  ─────────────────────────────────────────────────────────────────────────  │
│  Audio receive              │ 10ms     │ WebRTC low-latency                │
│  VAD                        │ 10ms     │ MagicNet 10ms frames              │
│  STT streaming              │ 100ms    │ Partial results, prefetch         │
│  Turn detection             │ 30ms     │ SmolLM2-135M semantic             │
│  RAG (speculative)          │ 50ms     │ Prefetch on partial transcript    │
│  Cross-encoder rerank       │ 20ms     │ Early-exit PABEE                  │
│  LLM first token            │ 120ms    │ SLM race + speculative            │
│  TTS first audio            │ 60ms     │ Word-level streaming              │
│  Audio send                 │ 10ms     │ Low-latency buffer                │
│  ─────────────────────────────────────────────────────────────────────────  │
│  TOTAL (optimistic)         │ ~450ms   │                                   │
│  TOTAL (with network)       │ ~550ms   │                                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: Core Pipeline (Weeks 1-4)
- [ ] MagicNet VAD implementation
- [ ] Streaming STT integration
- [ ] Basic pipeline orchestration
- [ ] Simple turn detection (VAD-based)

### Phase 2: Intelligence (Weeks 5-8)
- [ ] HybridTurnDetector with semantic
- [ ] Early-exit cross-encoder
- [ ] Speculative LLM execution
- [ ] MCP tool integration

### Phase 3: Agent (Weeks 9-12)
- [ ] Stage-based FSM
- [ ] Hierarchical memory
- [ ] Persuasion engine
- [ ] Domain tools (calculator, locator)

### Phase 4: Polish (Weeks 13-16)
- [ ] Word-level TTS streaming
- [ ] Barge-in handling
- [ ] Load testing
- [ ] Production hardening

---

## Research Sources

### Academic Papers

| Paper | Contribution |
|-------|--------------|
| Full-Duplex Survey (arXiv:2509.14515) | Architecture taxonomy |
| Turnsense | Semantic turn detection |
| MagicNet | Low-latency VAD |
| PABEE | Early-exit inference |
| EAGLE | Speculative decoding |

### Industry References

| Source | Insight |
|--------|---------|
| LiveKit Realtime | 195ms full-duplex |
| Deepgram | 16% satisfaction drop/sec |
| Cresta | 78% failures in edge cases |
| Hacker News | 133ms voice agent achieved |

---

## Crate Structure

```
voice-agent-rust/
├── Cargo.toml                    # Workspace
├── crates/
│   ├── core/                     # Core traits and types
│   ├── config/                   # Configuration management
│   ├── pipeline/                 # Audio pipeline
│   │   ├── vad/                  # Voice activity detection
│   │   ├── turn_detection/       # Semantic turn detection
│   │   ├── stt/                  # Speech-to-text
│   │   └── tts/                  # Text-to-speech
│   ├── speech/                   # Speech processing utilities
│   ├── rag/                      # Retrieval and ranking
│   │   └── reranker/             # Early-exit cross-encoder
│   ├── llm/                      # LLM clients and speculative
│   ├── tools/                    # MCP tools
│   │   ├── mcp/                  # MCP protocol
│   │   └── domain/               # Domain-specific tools
│   ├── agent/                    # Agent framework
│   │   ├── stages/               # Conversation stages
│   │   ├── memory/               # Hierarchical memory
│   │   └── persuasion/           # Persuasion engine
│   ├── experiments/              # A/B testing
│   └── server/                   # HTTP/WebSocket server
└── models/                       # ONNX model files
```

---

## Risk Summary

### High Risk (Requires Validation)
- IndicConformer ONNX export - never verified at scale
- IndicF5 ONNX export - never verified at scale
- IndicTrans2 ONNX export - fairseq complexity

### Medium Risk (Have Fallbacks)
- sherpa-rs production stability → Whisper fallback
- Kalosm streaming reliability → Ollama fallback
- Semantic turn detection accuracy → VAD fallback

### Low Risk (Battle-Tested)
- tokio, axum ecosystem
- Qdrant, Tantivy
- ONNX Runtime (ort)

---

## Getting Started

1. **Read the docs** in order: ARCHITECTURE_v2.md → optimized-pipeline.md → component docs
2. **Review Rust ecosystem** to understand library choices
3. **Start with Phase 1** - core pipeline is foundation for everything else
4. **Test latency early** - measure E2E before adding features
5. **Plan fallbacks** - especially for Indian language models

---

## Contributing

When updating documentation:
1. Keep code examples in sync with actual implementation
2. Update this INDEX.md when adding new docs
3. Include latency impact for any new component
4. Document fallback strategies for risky components
