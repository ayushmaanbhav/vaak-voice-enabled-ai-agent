# Issues Summary - Quick Reference

## P0 - Critical (System Broken)

| ID | Issue | File | Line | Status |
|----|-------|------|------|--------|
| P0-1 | Config files not loaded - uses defaults only | server/src/main.rs | 14 | **FIXED** |
| P0-2 | Context token limit conflict (2048 vs 32768) | rag/src/context.rs | 248 | Open |

## P1 - High Priority

| ID | Issue | File(s) | Impact |
|----|-------|---------|--------|
| P1-1 | Message/Role types duplicated | llm/src/prompt.rs | Missing fields in LLM messages |
| P1-2 | PCM constants inconsistent (32767 vs 32768) | core/audio.rs, transport/codec.rs, server/websocket.rs | Audio conversion errors |
| P1-3 | WebRTC ICE timeouts hardcoded | transport/src/webrtc.rs:432-434 | Cannot configure timeouts |
| P1-4 | Turn detection timing hardcoded | pipeline/src/turn_detection/hybrid.rs:68-71 | Cannot tune turn detection |
| P1-5 | 14K purity factor missing | config/src/constants.rs | Incomplete purity support |

## P2 - Code Quality

| ID | Issue | File(s) | Impact |
|----|-------|---------|--------|
| P2-1 | ConversationContext duplicated (incompatible) | core/retriever.rs, rag/agentic.rs | Requires adapter conversion |
| P2-2 | Document struct field mismatch (content vs text) | core/retriever.rs, rag/vector_store.rs | Type incompatibility |
| P2-3 | Interest rates hardcoded in strings | llm/prompt.rs, agent/persuasion.rs | Hard to update rates |
| P2-4 | LLM crate has shadow config structs | llm/src/backend.rs, factory.rs, claude.rs | Config confusion |
| P2-5 | VAD frame counts hardcoded | pipeline/src/vad/magicnet.rs:59-60 | Cannot tune VAD |

## P3 - Polish

| ID | Issue | File(s) | Impact |
|----|-------|---------|--------|
| P3-1 | ToolExecutor trait in wrong crate | tools/src/registry.rs | Should be in core |
| P3-2 | G2P Language enum undocumented | pipeline/src/tts/g2p.rs:71 | Confusion with core Language |
| P3-3 | Missing config validation tests | config/src/ | No automated verification |

---

## Constants Duplication Count

| Constant Type | Centralized | Still Duplicated In |
|---------------|-------------|---------------------|
| Interest rates (11.5%, 10.5%, 9.5%) | YES | 20+ files |
| NBFC rates (18-24%) | YES | 15+ files |
| PCM conversion (32767/32768) | NO | 5 files |
| Sample rate (16000) | YES | 6 files |
| Timeouts (30s, 60s) | YES | 8+ files |
| Confidence thresholds (0.4-0.95) | PARTIAL | 10+ files |
| LTV ratios (75%, 70%) | YES | 8+ files |

---

## Type Duplication Matrix

| Type | core/ | llm/ | rag/ | pipeline/ | Severity |
|------|-------|------|------|-----------|----------|
| Message | llm_types.rs | prompt.rs | - | - | CRITICAL |
| Role | llm_types.rs | prompt.rs | - | - | CRITICAL |
| ConversationContext | retriever.rs | - | agentic.rs | - | CRITICAL |
| Document | retriever.rs | - | vector_store.rs | - | HIGH |
| Language | language.rs | - | - | g2p.rs | MEDIUM |
| ErrorCode | error.rs, tool.rs | - | - | - | MEDIUM |

---

## Quick Stats

- **Total Issues:** 16
- **P0 Critical:** 2
- **P1 High:** 5
- **P2 Medium:** 5
- **P3 Low:** 3
- **Files Affected:** 25+
- **Duplicate Types:** 6
- **Scattered Constants:** 50+ locations
