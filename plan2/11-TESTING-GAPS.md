# Testing Coverage Gaps & Plan

> **Current Coverage:** ~25-30% estimated
> **Target Coverage:** 70%+ for production
> **Priority:** P1-P2

---

## Current State Summary

| Crate | Unit Tests | Integration | ONNX Tests | Benchmarks | Hindi Tests |
|-------|------------|-------------|------------|------------|-------------|
| Pipeline | 22 | 0 | 0 | 0 | 0 |
| LLM | 14 | 0 | N/A | 0 | 0 |
| RAG | 12 | 0 | 0 | 0 | 0 |
| Agent | 18 | 1 | N/A | 0 | 0 |
| Tools | 15 | 0 | N/A | 0 | 0 |
| Core | 10 | 0 | N/A | 0 | 0 |
| Server | 0 | 0 | N/A | 0 | 0 |
| Transport | 0 | 0 | N/A | 0 | 0 |

---

## Critical Missing Tests

### 1. ONNX Model Loading & Inference
```rust
#[tokio::test]
#[ignore = "requires ONNX models"]
async fn test_vad_onnx_inference() {
    let vad = VoiceActivityDetector::new(VadConfig::default())?;
    let frame = AudioFrame::from_samples(&[0.1f32; 160], SampleRate::Hz16000);
    let (state, prob) = vad.process_frame(&mut frame.clone())?;
    assert!(prob >= 0.0 && prob <= 1.0);
}

#[tokio::test]
#[ignore = "requires ONNX models"]
async fn test_stt_indicconformer() {
    let stt = StreamingStt::indicconformer(SttConfig::default())?;
    let audio = load_test_audio("test_data/hindi_greeting.wav");
    let transcript = stt.transcribe(&audio).await?;
    assert!(transcript.text.contains("नमस्ते") || transcript.text.contains("namaste"));
}
```

### 2. End-to-End Pipeline Integration
```rust
#[tokio::test]
async fn test_full_pipeline_flow() {
    let pipeline = Pipeline::new(PipelineConfig::default());
    let audio_stream = simulate_audio_stream("Hello, I need a gold loan");

    let results: Vec<_> = pipeline.process_stream(audio_stream).collect().await;

    assert!(results.iter().any(|r| matches!(r, PipelineEvent::TranscriptFinal(_))));
    assert!(results.iter().any(|r| matches!(r, PipelineEvent::AgentResponse(_))));
}
```

### 3. Latency Benchmarks
```rust
#[bench]
fn bench_vad_frame_processing(b: &mut Bencher) {
    let vad = VoiceActivityDetector::new(VadConfig::default()).unwrap();
    let frame = AudioFrame::from_samples(&[0.1f32; 160], SampleRate::Hz16000);

    b.iter(|| {
        let mut f = frame.clone();
        vad.process_frame(&mut f).unwrap()
    });
}

#[bench]
fn bench_end_to_end_latency(b: &mut Bencher) {
    // Target: <500ms
    let agent = GoldLoanAgent::new("bench", AgentConfig::default());

    b.iter(|| {
        let start = Instant::now();
        let _ = tokio_test::block_on(agent.process("What is the interest rate?"));
        start.elapsed()
    });
}
```

### 4. Hindi/Devanagari Tests
```rust
#[test]
fn test_hindi_slot_extraction() {
    let extractor = SlotExtractor::new();
    let slots = extractor.extract("मुझे पांच लाख का लोन चाहिए");
    assert_eq!(slots.get("loan_amount"), Some(&500_000.0));
}

#[test]
fn test_devanagari_numeral_conversion() {
    assert_eq!(devanagari_to_ascii("५००००"), "50000");
    assert_eq!(devanagari_to_ascii("१२३"), "123");
}

#[test]
fn test_hindi_intent_detection() {
    let classifier = IntentClassifier::new();
    let intent = classifier.classify("ब्याज दर क्या है");
    assert_eq!(intent.intent, "interest_rate");
}
```

### 5. Concurrent Access Tests
```rust
#[tokio::test]
async fn test_concurrent_sessions() {
    let server = spawn_test_server().await;

    let handles: Vec<_> = (0..10)
        .map(|i| {
            tokio::spawn(async move {
                let session_id = format!("session-{}", i);
                let client = TestClient::new(&session_id);
                client.send("Hello").await?;
                client.send("I need a gold loan").await?;
                client.send("100 grams gold").await?;
                Ok::<_, anyhow::Error>(())
            })
        })
        .collect();

    for h in handles {
        h.await.unwrap().unwrap();
    }
}
```

### 6. Error Recovery Tests
```rust
#[tokio::test]
async fn test_llm_timeout_recovery() {
    let backend = MockLlmBackend::with_latency(Duration::from_secs(5));
    let executor = SpeculativeExecutor::new(backend.clone(), backend);

    let result = executor.execute(&[Message::user("test")]).await;

    // Should fallback or timeout gracefully
    assert!(result.is_ok() || matches!(result, Err(LlmError::Timeout)));
}

#[tokio::test]
async fn test_rag_empty_results() {
    let retriever = HybridRetriever::new(empty_vector_store());
    let results = retriever.search("nonexistent query", 10).await?;

    assert!(results.is_empty());
    // Should not panic or error
}
```

---

## Test Data Requirements

### Audio Files Needed
```
test_data/
├── audio/
│   ├── hindi_greeting.wav         # "नमस्ते"
│   ├── hindi_loan_inquiry.wav     # "मुझे गोल्ड लोन चाहिए"
│   ├── english_greeting.wav       # "Hello, good morning"
│   ├── hinglish_mixed.wav         # "Mujhe 5 lakh ka loan chahiye"
│   ├── noise_only.wav             # Background noise (no speech)
│   └── silence.wav                # Pure silence
├── transcripts/
│   └── expected_outputs.json      # Ground truth transcriptions
└── conversations/
    └── multi_turn_scenario.json   # Full conversation flows
```

### ONNX Models for Testing
```
test_models/
├── vad/
│   └── silero_vad.onnx            # Small VAD model
├── stt/
│   └── whisper_tiny.onnx          # Tiny STT for fast tests
├── embeddings/
│   └── minilm.onnx                # Small embedding model
└── reranker/
    └── cross_encoder_small.onnx   # Small reranker
```

---

## Test Categories by Priority

### P0: Must Have Before Production
- [ ] ONNX model loading succeeds
- [ ] Pipeline doesn't panic on edge cases
- [ ] Memory doesn't leak in long conversations
- [ ] Session cleanup works
- [ ] Hindi slot extraction works

### P1: Should Have
- [ ] Latency benchmarks pass (<500ms)
- [ ] Concurrent session handling
- [ ] LLM fallback works
- [ ] RAG returns relevant results
- [ ] Tool calculations are correct

### P2: Nice to Have
- [ ] 80%+ code coverage
- [ ] Chaos testing (network failures)
- [ ] Load testing (1000 concurrent)
- [ ] Fuzzing for security

---

## CI/CD Integration

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run unit tests
        run: cargo test --all-features

  integration-tests:
    runs-on: ubuntu-latest
    needs: unit-tests
    services:
      redis:
        image: redis:7
        ports:
          - 6379:6379
      qdrant:
        image: qdrant/qdrant
        ports:
          - 6333:6333
    steps:
      - uses: actions/checkout@v4
      - name: Download test models
        run: ./scripts/download_test_models.sh
      - name: Run integration tests
        run: cargo test --test '*' -- --ignored

  benchmarks:
    runs-on: ubuntu-latest
    needs: unit-tests
    steps:
      - uses: actions/checkout@v4
      - name: Run benchmarks
        run: cargo bench -- --save-baseline main
      - name: Check latency regression
        run: ./scripts/check_latency.sh

  hindi-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run Hindi-specific tests
        run: cargo test hindi -- --test-threads=1
```

---

## Effort Estimate

| Category | Tests Needed | Effort |
|----------|--------------|--------|
| ONNX Integration | 15 | 16h |
| Pipeline E2E | 10 | 12h |
| Latency Benchmarks | 8 | 8h |
| Hindi/Devanagari | 20 | 16h |
| Concurrent Access | 5 | 8h |
| Error Recovery | 10 | 8h |
| Server HTTP | 15 | 12h |
| WebSocket/WebRTC | 10 | 16h |

**Total: ~85 tests, ~96 hours (2-3 weeks)**

---

*This plan addresses the testing gaps identified across all crates.*
