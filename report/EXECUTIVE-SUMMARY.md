# Voice Agent Code Review - Executive Summary

> **Review Date:** December 29, 2025
> **Codebase:** `/home/vscode/goldloan-study/voice-agent/backend`
> **Architecture Version:** 2.0
> **Documented Production Readiness:** 52%
> **Actual Production Readiness:** ~65% (Pipeline improved significantly)

---

## Overall Assessment

The voice agent codebase demonstrates **solid architecture** with proper separation of concerns across 12 Rust crates. The implementation is **more complete than documented** in some areas (pipeline is frame-based, not monolithic), but has **critical integration gaps** that prevent end-to-end functionality.

### Verdict: **GOOD FOUNDATION, CRITICAL WIRING ISSUES**

---

## Component Status Matrix

| # | Component | Documented Status | Actual Status | Gap Assessment |
|---|-----------|-------------------|---------------|----------------|
| 1 | **Core Traits (9)** | 9 traits | 8 traits + 1 misplaced | ConversationFSM MISSING |
| 2 | **22 Languages** | Full | Full | None |
| 3 | **Frame Pipeline** | Monolithic | Frame-based | BETTER than spec |
| 4 | **Sentence Streaming** | Detector only | Full streaming | BETTER than spec |
| 5 | **Text Processing** | 72% | 90%+ | Translation IMPLEMENTED |
| 6 | **RAG Hybrid** | 85% | 90% | Query expansion NOT wired |
| 7 | **Personalization** | Complete | Scaffold only | NOT auto-detected |
| 8 | **Session Persistence** | Stubbed | Partially stubbed | Redis stub, ScyllaDB impl |

---

## Critical Findings (P0)

| # | Issue | Location | Impact | Severity |
|---|-------|----------|--------|----------|
| 1 | **ConversationFSM trait NOT DEFINED** | core/src/traits/ | Cannot implement proper state machine | CRITICAL |
| 2 | **Tool trait in wrong crate** | tools/src/mcp.rs | Architecture inconsistency | HIGH |
| 3 | **LLM ToolDefinition type mismatch** | llm/prompt.rs vs core/llm_types.rs | Tool calling broken | CRITICAL |
| 4 | **Tools use hardcoded config** | tools/gold_loan.rs | Config changes ignored | HIGH |
| 5 | **LLM crate doesn't implement core::LanguageModel trait** | llm/backend.rs | Type mismatch, needs adapter | Medium |
| 6 | **WebRTC audio NOT connected** | server/webrtc.rs | Signaling only, no audio flow | HIGH |
| 7 | **Query expansion NOT wired** | rag/retriever.rs | Lower recall for Hindi queries | Medium |
| 8 | **Personalization signals detected but NOT used** | agent/agent.rs | No behavioral adaptation | Medium |

---

## Dead Code Inventory

| Component | Dead Code | Reason |
|-----------|-----------|--------|
| **Reranker** | `should_exit()`, `LayerOutput` | ONNX can't do layer-by-layer exit |
| **RAG** | Query expansion in HybridRetriever | Only in EnhancedRetriever adapter |
| **LLM** | `generate_with_tools()` | Accepts tools but ignores them |
| **Personalization** | Signal detection flow | Signals logged but not applied |
| **Transport** | Full WebRTC implementation | Server uses WebSocket only |
| **Tools** | CRM/Calendar integrations | Default registry uses stubs |

---

## Integration Gaps (Missing Connections)

### What's Connected (Working)
- Server HTTP API + Authentication + Metrics
- WebSocket audio stream pipeline
- Session management with GoldLoanAgent
- Agent tool execution (intent-based)
- LLM generation (Ollama/OpenAI)
- RAG retrieval with reranking
- TTS sentence streaming
- Hindi/Indic sentence terminators

### What's Disconnected (Not Wired)

```
┌─────────────────────────────────────────────────────────────────────┐
│                      DISCONNECTED COMPONENTS                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1. ConversationFSM Trait ──────────────────── NOT DEFINED          │
│     └── Only ConversationStage enum exists                          │
│                                                                     │
│  2. LLM Tool Calling ───────────────────────── STUBBED              │
│     └── generate_with_tools() ignores tools parameter               │
│                                                                     │
│  3. Domain Config → Tools ──────────────────── NOT WIRED            │
│     └── Tools use hardcoded GoldLoanConfig::default()               │
│                                                                     │
│  4. Query Expansion → Retriever ────────────── NOT WIRED            │
│     └── Only in EnhancedRetriever adapter (not used)                │
│                                                                     │
│  5. Personalization Signals → Behavior ─────── NOT WIRED            │
│     └── Signals detected but not used in responses                  │
│                                                                     │
│  6. WebRTC Transport → Audio Pipeline ──────── NOT WIRED            │
│     └── Signaling works, audio stays in transport crate             │
│                                                                     │
│  7. LanguageModelAdapter → Agent ───────────── NOT USED             │
│     └── Agent uses LlmBackend directly, bypassing adapter           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Crate-by-Crate Summary

| Crate | LOC | Status | Key Issues |
|-------|-----|--------|------------|
| **core** | ~3,000 | 90% | Missing ConversationFSM trait |
| **config** | ~1,200 | 95% | Working, hot-reload enabled |
| **pipeline** | ~2,500 | 90% | Frame-based, streaming works |
| **llm** | ~3,400 | 75% | Type mismatches, tools stubbed |
| **rag** | ~2,800 | 85% | Early-exit dead, query expansion not wired |
| **text_processing** | ~2,200 | 90% | Translation works (Candle IndicTrans2) |
| **tools** | ~2,500 | 70% | Uses hardcoded config, stubs default |
| **agent** | ~4,000 | 80% | FSM works, personalization scaffold |
| **server** | ~2,000 | 85% | WebSocket works, WebRTC partial |
| **transport** | ~1,500 | 40% | Implemented but not integrated |
| **persistence** | ~800 | 60% | ScyllaDB impl, Redis stub |

---

## Recommendations Priority

### Immediate (P0) - Blocks Production
1. Define `ConversationFSM` trait in core crate
2. Fix `ToolDefinition` type mismatch between core and llm crates
3. Wire domain config to tools (inject `DomainConfigManager`)
4. Implement LLM tool calling (parse tool responses)

### Short Term (P1) - Required for Full Feature
1. Wire query expansion in HybridRetriever
2. Use LanguageModelAdapter in Agent (not raw LlmBackend)
3. Connect WebRTC audio to pipeline
4. Wire personalization signals to prompt generation

### Medium Term (P2) - Production Hardening
1. Remove dead code (reranker should_exit, etc.)
2. Add proper error propagation in adapter layers
3. Implement session persistence with Redis/ScyllaDB
4. Add integration tests for full audio flow

---

## Architecture Compliance

| Principle | Documented | Implemented | Status |
|-----------|------------|-------------|--------|
| Configurability over code | TOML/YAML configs | Configs loaded | Partial (tools hardcoded) |
| Streaming by default | Every stage streams | Pipeline streams | YES |
| Experiment everything | A/B testable | Timing strategies exist | Partial |
| Fail gracefully | Fallbacks everywhere | Translation has fallback | YES |
| Privacy by design | On-premise, PII redact | PII implemented | YES |

---

## Test Coverage Summary

| Crate | Unit Tests | Integration Tests | Coverage |
|-------|------------|-------------------|----------|
| core | YES | - | Good |
| pipeline | YES | YES | Good |
| llm | YES | - | Moderate |
| rag | YES | - | Good |
| text_processing | YES | - | Good |
| agent | YES | YES (40+) | Excellent |
| tools | YES | - | Good |
| server | YES | - | Moderate |

---

## Files for Detailed Analysis

- [COMPONENT-ANALYSIS.md](./COMPONENT-ANALYSIS.md) - Deep dive per crate
- [DEAD-CODE-ANALYSIS.md](./DEAD-CODE-ANALYSIS.md) - All dead code with locations
- [FIX-PLAN.md](./FIX-PLAN.md) - Prioritized implementation plan
- [INTEGRATION-GAPS.md](./INTEGRATION-GAPS.md) - Missing connections with fixes

---

**Report Generated:** December 29, 2025
**Analysis Method:** Multi-agent parallel codebase exploration
**Crates Analyzed:** 12 workspace crates
**Total Lines of Code:** ~25,000+ LOC
