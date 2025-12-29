# Voice Agent Rust - Plan2 Documentation Index

> **Purpose:** Master navigation document for the comprehensive code review and implementation plan
> **Last Updated:** 2024-12-28
> **Total Documents:** 15

---

## Quick Navigation

| Category | Documents | Description |
|----------|-----------|-------------|
| Overview | 00 | Executive summary with production readiness |
| Issue Tracking | 01-02 | Critical and high-priority fixes |
| Analysis Reports | 10-13 | Deep-dive analysis and gap identification |
| Implementation Specs | 14-18 | Detailed implementation roadmap and phase specs |

---

## Document Catalog

### Executive Summary
| File | Title | Key Content |
|------|-------|-------------|
| [00-EXECUTIVE-SUMMARY.md](./00-EXECUTIVE-SUMMARY.md) | Executive Summary | Production readiness (45%), priority matrix, architecture concerns |

### Issue Tracking (By Priority)
| File | Title | Issues | Est. Effort |
|------|-------|--------|-------------|
| [01-CRITICAL-FIXES.md](./01-CRITICAL-FIXES.md) | P0 Critical Fixes | 23 ship blockers | 2 weeks |
| [02-HIGH-PRIORITY-FIXES.md](./02-HIGH-PRIORITY-FIXES.md) | P1 High Priority | 31 pre-launch items | 2 weeks |

### Deep Analysis Reports
| File | Title | Key Findings |
|------|-------|--------------|
| [10-MULTILINGUAL-SUPPORT-PLAN.md](./10-MULTILINGUAL-SUPPORT-PLAN.md) | 22-Language Support Plan | Native approach for all Indian languages |
| [11-TESTING-GAPS.md](./11-TESTING-GAPS.md) | Testing Coverage Gaps | Unit/integration test gaps per crate |
| [12-APPROACH-COMPARISON.md](./12-APPROACH-COMPARISON.md) | Architecture Approaches | Approach 1 vs 2 with pluggable design |
| [13-ARCHITECTURE-GAP-ANALYSIS.md](./13-ARCHITECTURE-GAP-ANALYSIS.md) | **CRITICAL** Gap Analysis | ~75% of ARCHITECTURE_v2.md unimplemented |

### Implementation Roadmap (6 Phases)
| File | Phase | Scope | Dependencies |
|------|-------|-------|--------------|
| [14-IMPLEMENTATION-ROADMAP.md](./14-IMPLEMENTATION-ROADMAP.md) | Overview | 6-phase plan, 8-12 weeks total | None |
| [15-PHASE1-CORE-TRAITS.md](./15-PHASE1-CORE-TRAITS.md) | Phase 1 | Core traits: 9 async traits, 22 languages | None |
| [16-PHASE2-TEXT-PROCESSING.md](./16-PHASE2-TEXT-PROCESSING.md) | Phase 2 | Text processing crate: grammar, translation, PII | Phase 1 |
| [17-PHASE3-PIPELINE.md](./17-PHASE3-PIPELINE.md) | Phase 3 | Frame-based pipeline architecture | Phase 1, 2 |
| [18-PHASE4-6-REMAINING.md](./18-PHASE4-6-REMAINING.md) | Phases 4-6 | RAG, Personalization, Domain Config | Phase 1-3 |

---

## Reading Order

### For Understanding Current State
1. **00-EXECUTIVE-SUMMARY.md** - Start here for overview
2. **13-ARCHITECTURE-GAP-ANALYSIS.md** - Critical: understand what's missing
3. **01-CRITICAL-FIXES.md** - Immediate blockers
4. **02-HIGH-PRIORITY-FIXES.md** - Pre-launch requirements

### For Implementation Planning
1. **14-IMPLEMENTATION-ROADMAP.md** - 6-phase overview
2. **15-PHASE1-CORE-TRAITS.md** - Foundation (start here)
3. **16-PHASE2-TEXT-PROCESSING.md** - Text pipeline
4. **17-PHASE3-PIPELINE.md** - Frame processing
5. **18-PHASE4-6-REMAINING.md** - RAG, Personalization, Config

### For Specific Concerns
- **Multilingual support:** 10-MULTILINGUAL-SUPPORT-PLAN.md
- **Testing gaps:** 11-TESTING-GAPS.md
- **Architecture decisions:** 12-APPROACH-COMPARISON.md

---

## Implementation Phases Summary

```
Phase 1: Core Traits (Week 1-2)
├── 9 async traits (SpeechToText, TextToSpeech, LanguageModel, etc.)
├── Language enum (22 Indian languages)
├── Supporting types (AudioFrame, TranscriptFrame, PIIEntity, etc.)
└── Adapter implementations for existing backends

Phase 2: Text Processing (Week 2-3)
├── crates/text_processing/ (new crate)
├── GrammarCorrector (LLM-based)
├── Translator (IndicTrans2 ONNX + gRPC fallback)
├── PIIRedactor (hybrid: regex + model)
└── ComplianceChecker (rule-based)

Phase 3: Pipeline Architecture (Week 3-5)
├── Frame enum (AudioInput, Transcript, LLMChunk, BargeIn, etc.)
├── FrameProcessor trait implementations
├── Channel-based pipeline orchestrator
├── SentenceDetector with Indic terminators
└── InterruptHandler (SentenceBoundary, Immediate, WordBoundary)

Phase 4: RAG Enhancements (Week 5-6)
├── RAG timing strategies (Sequential, PrefetchAsync, ParallelInject)
├── Stage-aware context sizing (ContextBudget)
├── Pipeline integration via RagFrame
└── A/B testing infrastructure

Phase 5: Personalization Engine (Week 6-7)
├── crates/personalization/ (new crate)
├── SegmentDetector (rule + ML hybrid)
├── PersuasionStrategy per segment
├── DisclosureHandler (compliance)
└── PromptCustomizer

Phase 6: Domain Configuration (Week 7-8)
├── domains/ directory structure
├── DomainConfig schema (TOML/YAML)
├── DomainLoader with hot reload
├── Tera template engine integration
└── Multi-domain routing
```

---

## Key Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Production Readiness | 45% | 95% |
| Architecture Alignment | 25% | 95% |
| Language Support | 3 | 22 |
| Test Coverage | ~40% | 80% |
| Critical Issues | 23 | 0 |
| High Priority Issues | 31 | 0 |

---

## File Organization

```
plan2/
├── INDEX.md                         # THIS FILE - Master navigation
│
├── 00-EXECUTIVE-SUMMARY.md          # Overview & production readiness
│
├── [01-02] Issue Tracking
│   ├── 01-CRITICAL-FIXES.md         # P0 - Ship blockers
│   └── 02-HIGH-PRIORITY-FIXES.md    # P1 - Pre-launch
│
├── [10-13] Analysis Reports
│   ├── 10-MULTILINGUAL-SUPPORT-PLAN.md   # 22 language plan
│   ├── 11-TESTING-GAPS.md                # Test coverage
│   ├── 12-APPROACH-COMPARISON.md         # Architecture options
│   └── 13-ARCHITECTURE-GAP-ANALYSIS.md   # CRITICAL: Doc vs impl gaps
│
└── [14-18] Implementation Roadmap
    ├── 14-IMPLEMENTATION-ROADMAP.md      # 6-phase overview
    ├── 15-PHASE1-CORE-TRAITS.md          # Traits & types
    ├── 16-PHASE2-TEXT-PROCESSING.md      # Text pipeline crate
    ├── 17-PHASE3-PIPELINE.md             # Frame architecture
    └── 18-PHASE4-6-REMAINING.md          # RAG, Personalization, Config
```

---

## Cross-References

### Architecture Document References
- **ARCHITECTURE_v2.md** - The aspirational architecture (gap analyzed in 13)
- **docs/MODELS.md** - ML model specifications
- **docs/STT_INTEGRATION.md** - Speech-to-text details
- **docs/TTS_INTEGRATION.md** - Text-to-speech details

### Code References by Phase
| Phase | Primary Files to Create/Modify |
|-------|-------------------------------|
| 1 | `crates/core/src/traits/*.rs` |
| 2 | `crates/text_processing/` (new) |
| 3 | `crates/pipeline/src/orchestrator.rs`, `crates/pipeline/src/frame.rs` |
| 4 | `crates/rag/src/timing.rs`, `crates/rag/src/context_budget.rs` |
| 5 | `crates/personalization/` (new) |
| 6 | `domains/`, `crates/core/src/config/domain.rs` |

---

## Notes

1. **Numbering Gap (03-09):** Reserved for future detailed reviews per crate
2. **Implementation Order:** Must follow phase dependencies (1→2→3→4/5/6)
3. **Parallel Work:** Phases 4, 5, 6 can proceed in parallel after Phase 3
4. **Testing:** Each phase should include comprehensive tests before proceeding

---

*Generated: 2024-12-28 | Voice Agent Rust Plan2 Index*
