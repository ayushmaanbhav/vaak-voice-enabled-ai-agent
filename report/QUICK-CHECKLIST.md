# Voice Agent Rust - Quick Reference Checklist

## P0 Critical Issues (Block Production)

- [x] **Register 3 missing tools** - `tools/src/registry.rs` ✅ FIXED
  - GetGoldPriceTool, EscalateToHumanTool, SendSmsTool now registered

- [x] **Create audit_log table** - `persistence/src/schema.rs` ✅ FIXED
  - RBI compliance table added with 7-year retention

- [x] **Integrate PersuasionEngine** - `agent/src/agent.rs` ✅ FIXED
  - Objection detection and handling now in generate_response()

- [x] **Fix EMI calculation** - `tools/src/gold_loan.rs` ✅ FIXED
  - Proper EMI formula added alongside simple interest

- [ ] **Wire Transport to Server** - `server/Cargo.toml`, `server/src/http.rs`
  - Or document as removed feature (1,500 LOC dead)

- [x] **Create LanguageModel adapter** - `llm/src/adapter.rs` ✅ FIXED
  - LanguageModelAdapter bridges LlmBackend to core LanguageModel trait

---

## P1 High Priority

- [ ] Add HTTP rate limiting
- [ ] Create configuration YAML files
- [ ] Implement real SMS gateway (replace simulated)
- [ ] Add OpenAI backend retry logic

---

## P2 Medium Priority

- [ ] Wire Frame Processors to Orchestrator (or document limitation)
- [ ] Add TextSimplifier for TTS
- [ ] Create experiments.rs for A/B testing
- [ ] Add missing metrics (HTTP latency, WS connections)

---

## Component Status Summary

| Component | Status | Critical Issue |
|-----------|--------|----------------|
| Pipeline/VAD | ✅ Working | - |
| Pipeline/STT | ✅ Working | - |
| Pipeline/TTS | ✅ Working | - |
| Pipeline/Orchestrator | ⚠️ Simplified | Processors disconnected |
| Agent/VoiceSession | ✅ Excellent | - |
| Agent/Memory | ✅ Complete | - |
| Agent/Persuasion | ⚠️ Exists | NOT INVOKED |
| RAG/Retrieval | ✅ Working | - |
| RAG/Reranker | ⚠️ Cascaded only | Early-exit non-functional |
| LLM/Backends | ✅ Complete | - |
| LLM/Traits | ⚠️ Different | No LanguageModel impl |
| Text Processing | ✅ 95% | Missing TextSimplifier |
| Tools | ⚠️ 5/8 registered | 3 P0 tools missing |
| Persistence | ⚠️ Missing table | audit_log not created |
| Server/HTTP | ✅ Complete | - |
| Server/WebSocket | ✅ Working | - |
| Server/WebRTC | ❌ UNUSED | Transport not integrated |

---

## Quick Commands

```bash
# Build all
cd voice-agent/backend && cargo build --workspace

# Run tests
cargo test --workspace

# Check for warnings
cargo clippy --workspace

# Run server (development)
cargo run -p voice-agent-server

# Check database schema
# (requires ScyllaDB running)
cqlsh -e "DESC KEYSPACE voice_agent"
```

---

## Key Files to Review

| Purpose | File |
|---------|------|
| Entry point | `crates/server/src/main.rs` |
| HTTP routes | `crates/server/src/http.rs` |
| Voice session | `crates/agent/src/voice_session.rs` |
| Tool registry | `crates/tools/src/registry.rs` |
| DB schema | `crates/persistence/src/schema.rs` |
| Config loading | `crates/config/src/settings.rs` |

---

## Risk Summary

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| EMI miscalculation | HIGH | Customer complaints | Fix formula immediately |
| Audit failure | HIGH | Regulatory issue | Add table to schema |
| Tools unavailable | HIGH | Feature broken | Register in registry |
| WebRTC unused | MEDIUM | Dev effort wasted | Integrate or remove |
| Latency targets missed | MEDIUM | UX degradation | Wire prefetch, processors |
