//! Push-to-Talk API Endpoint
//!
//! Simplified audio processing without VAD/turn detection.
//! User records audio, sends it, and receives response.
//!
//! Flow:
//! 1. Receive audio (webm/opus, base64 encoded)
//! 2. Convert to PCM 16kHz mono
//! 3. STT via Rust IndicConformer
//! 4. LLM processing via GoldLoanAgent (with RAG + tools)
//! 5. TTS response generation
//! 6. Return transcripts and audio

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use parking_lot::Mutex;

use crate::state::AppState;
use voice_agent_pipeline::stt::{IndicConformerStt, IndicConformerConfig};

/// Faster-whisper HTTP service URL
const WHISPER_SERVICE_URL: &str = "http://127.0.0.1:8091";

/// Request for push-to-talk processing
#[derive(Debug, Deserialize)]
pub struct PttRequest {
    /// Base64 encoded audio data
    pub audio: String,
    /// Audio format (webm, wav, etc.)
    pub audio_format: String,
    /// Language code (hi, en, ta, etc.)
    pub language: String,
}

/// Response from push-to-talk processing
#[derive(Debug, Serialize)]
pub struct PttResponse {
    /// User's transcribed text (raw from STT)
    pub user_text: String,
    /// User's text after grammar correction
    pub user_text_corrected: Option<String>,
    /// User's text before translation (if translated)
    pub user_text_original: Option<String>,
    /// Assistant's response text (in user's language)
    pub assistant_text: String,
    /// Assistant's text before translation (if translated)
    pub assistant_text_original: Option<String>,
    /// Base64 encoded audio response
    pub audio_response: Option<String>,
    /// Audio response format
    pub audio_format: Option<String>,
    /// Processing metrics
    pub metrics: PttMetrics,
    /// Current processing phase (for UI status updates)
    pub phase: String,
}

/// Processing metrics with per-phase timing
#[derive(Debug, Serialize, Default)]
pub struct PttMetrics {
    pub stt_ms: u64,
    pub grammar_ms: u64,
    pub llm_ms: u64,
    pub tts_ms: u64,
    pub total_ms: u64,
}

/// Lazy-initialized IndicConformer STT instance (for Indian languages)
static STT_INSTANCE: once_cell::sync::Lazy<Mutex<Option<IndicConformerStt>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

/// Check if language is English
fn is_english(language: &str) -> bool {
    matches!(language.to_lowercase().as_str(), "en" | "english")
}

/// Call faster-whisper HTTP service for English STT
async fn transcribe_with_whisper(audio: &[f32], language: &str) -> Result<String, String> {
    // Convert f32 samples to bytes
    let audio_bytes: Vec<u8> = audio
        .iter()
        .flat_map(|&f| f.to_le_bytes())
        .collect();

    let audio_b64 = BASE64.encode(&audio_bytes);

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/transcribe", WHISPER_SERVICE_URL))
        .json(&serde_json::json!({
            "audio": audio_b64,
            "language": language,
            "sample_rate": 16000
        }))
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| format!("Whisper service request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Whisper service error {}: {}", status, body));
    }

    let result: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse whisper response: {}", e))?;

    let text = result["text"].as_str().unwrap_or("").to_string();
    let proc_time = result["processing_time_seconds"].as_f64().unwrap_or(0.0);

    tracing::info!(
        "Faster-whisper transcribed in {:.2}s: '{}'",
        proc_time,
        if text.len() > 100 { &text[..100] } else { &text }
    );

    Ok(text)
}

/// Get or initialize IndicConformer STT for Indian languages
fn get_indicconformer_stt(language: &str) -> Result<(), String> {
    let mut stt_guard = STT_INSTANCE.lock();
    if stt_guard.is_none() {
        tracing::info!("Initializing IndicConformer STT for PTT...");
        let config = IndicConformerConfig {
            language: language.to_string(),
            ..Default::default()
        };
        let model_dir = std::path::Path::new("models/stt/indicconformer");
        match IndicConformerStt::new(model_dir, config) {
            Ok(stt) => {
                *stt_guard = Some(stt);
                tracing::info!("IndicConformer STT initialized for PTT");
            }
            Err(e) => {
                return Err(format!("Failed to initialize STT: {}", e));
            }
        }
    }
    Ok(())
}

/// Handle push-to-talk request
pub async fn handle_ptt(
    State(state): State<AppState>,
    Json(request): Json<PttRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let mut metrics = PttMetrics::default();

    tracing::info!(
        "PTT request: language={}, audio_format={}, audio_size={}",
        request.language,
        request.audio_format,
        request.audio.len()
    );

    // 1. Decode audio from base64
    let audio_bytes = match BASE64.decode(&request.audio) {
        Ok(bytes) => bytes,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Invalid base64 audio: {}", e) })),
            );
        }
    };

    // 2. Convert audio to PCM 16kHz mono f32
    let pcm_f32 = match convert_to_pcm_f32(&audio_bytes, &request.audio_format).await {
        Ok(pcm) => pcm,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Audio conversion failed: {}", e) })),
            );
        }
    };

    tracing::info!("Converted audio to PCM f32: {} samples", pcm_f32.len());

    // 3. Initialize STT and run based on language
    let stt_start = std::time::Instant::now();
    let use_english = is_english(&request.language);

    let stt_text = if use_english {
        // Use faster-whisper HTTP service for English
        match transcribe_with_whisper(&pcm_f32, &request.language).await {
            Ok(text) => text,
            Err(e) => {
                tracing::error!("Faster-whisper STT error: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": format!("Whisper STT failed: {}", e) })),
                );
            }
        }
    } else {
        // Use IndicConformer for Indian languages
        if let Err(e) = get_indicconformer_stt(&request.language) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            );
        }

        let stt_guard = STT_INSTANCE.lock();
        let stt = stt_guard.as_ref().unwrap();

        match stt.process(&pcm_f32) {
            Ok(Some(result)) => result.text,
            Ok(None) => String::new(),
            Err(e) => {
                tracing::error!("IndicConformer STT error: {}", e);
                String::new()
            }
        }
    };
    metrics.stt_ms = stt_start.elapsed().as_millis() as u64;

    tracing::info!("STT result ({}): '{}'", if use_english { "faster-whisper" } else { "IndicConformer" }, stt_text);

    if stt_text.is_empty() {
        let no_speech_msg = if use_english {
            "I didn't hear anything. Please speak again."
        } else {
            "मुझे कुछ सुनाई नहीं दिया। कृपया फिर से बोलें।"
        };
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "user_text": "",
                "user_text_corrected": null,
                "user_text_original": null,
                "assistant_text": no_speech_msg,
                "assistant_text_original": if use_english { serde_json::Value::Null } else { serde_json::json!("I didn't hear anything. Please speak again.") },
                "audio_response": null,
                "metrics": metrics,
                "phase": "complete"
            })),
        );
    }

    // 4. Deterministic phonetic error correction (SymSpell + confusion rules)
    let grammar_start = std::time::Instant::now();
    let (corrected, corrections) = state.phonetic_corrector.correct(&stt_text);
    let corrected_text = if !corrections.is_empty() {
        tracing::info!(
            original = stt_text.as_str(),
            corrected = corrected.as_str(),
            corrections = ?corrections.iter().map(|c| format!("{}->{}({})", c.original, c.corrected, c.rule)).collect::<Vec<_>>(),
            "Phonetic correction applied"
        );
        Some(corrected)
    } else {
        None
    };
    metrics.grammar_ms = grammar_start.elapsed().as_millis() as u64;

    // Use corrected text for LLM if available
    let text_for_llm = corrected_text.as_ref().unwrap_or(&stt_text);

    // 5. Call LLM via Agent pipeline (with RAG + tools)
    let llm_start = std::time::Instant::now();
    let llm_response = match process_with_agent(&state, text_for_llm, &request.language).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("Agent processing failed: {}", e);
            // Fallback to basic acknowledgment
            format_fallback_response(text_for_llm, &request.language)
        }
    };
    metrics.llm_ms = llm_start.elapsed().as_millis() as u64;

    tracing::info!("LLM response: '{}'", llm_response);

    // 6. Generate TTS (placeholder - return null for now)
    let tts_start = std::time::Instant::now();
    let audio_response: Option<String> = None;
    metrics.tts_ms = tts_start.elapsed().as_millis() as u64;

    metrics.total_ms = start.elapsed().as_millis() as u64;

    // 7. Return response
    let response = PttResponse {
        user_text: stt_text.clone(),
        user_text_corrected: corrected_text,
        user_text_original: None,
        assistant_text: llm_response,
        assistant_text_original: None,
        audio_response,
        audio_format: None,
        metrics,
        phase: "complete".to_string(),
    };

    (StatusCode::OK, Json(serde_json::to_value(response).unwrap()))
}

/// Convert audio to PCM 16kHz mono f32 samples
async fn convert_to_pcm_f32(audio_bytes: &[u8], format: &str) -> Result<Vec<f32>, String> {
    match format {
        "webm" | "opus" => convert_webm_to_pcm_f32(audio_bytes).await,
        "wav" => extract_wav_pcm_f32(audio_bytes),
        "pcm" => {
            // Assume PCM16 little-endian
            let samples: Vec<f32> = audio_bytes
                .chunks_exact(2)
                .map(|chunk| {
                    let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                    sample as f32 / 32768.0
                })
                .collect();
            Ok(samples)
        }
        _ => Err(format!("Unsupported audio format: {}", format)),
    }
}

/// Convert WebM/Opus to PCM f32 using ffmpeg
async fn convert_webm_to_pcm_f32(webm_bytes: &[u8]) -> Result<Vec<f32>, String> {
    use tokio::process::Command;

    // Create temp file for input
    let input_path = format!("/tmp/ptt_input_{}.webm", std::process::id());
    let output_path = format!("/tmp/ptt_output_{}.raw", std::process::id());

    // Write input file
    tokio::fs::write(&input_path, webm_bytes)
        .await
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    // Run ffmpeg to convert to raw PCM16
    let output = Command::new("ffmpeg")
        .args([
            "-y",
            "-i", &input_path,
            "-ar", "16000",
            "-ac", "1",
            "-f", "s16le",
            "-acodec", "pcm_s16le",
            &output_path,
        ])
        .output()
        .await
        .map_err(|e| format!("FFmpeg failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("FFmpeg error: {}", stderr);
        // Cleanup
        let _ = tokio::fs::remove_file(&input_path).await;
        let _ = tokio::fs::remove_file(&output_path).await;
        return Err(format!("FFmpeg conversion failed: {}", stderr));
    }

    // Read output as PCM16 bytes
    let pcm_bytes = tokio::fs::read(&output_path)
        .await
        .map_err(|e| format!("Failed to read output: {}", e))?;

    // Cleanup
    let _ = tokio::fs::remove_file(&input_path).await;
    let _ = tokio::fs::remove_file(&output_path).await;

    // Convert PCM16 to f32
    let samples: Vec<f32> = pcm_bytes
        .chunks_exact(2)
        .map(|chunk| {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            sample as f32 / 32768.0
        })
        .collect();

    Ok(samples)
}

/// Extract PCM f32 data from WAV file
fn extract_wav_pcm_f32(wav_bytes: &[u8]) -> Result<Vec<f32>, String> {
    // Simple WAV header parsing
    if wav_bytes.len() < 44 {
        return Err("WAV file too short".to_string());
    }

    // Find 'data' chunk
    let mut pos = 12; // Skip RIFF header
    while pos + 8 < wav_bytes.len() {
        let chunk_id = &wav_bytes[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            wav_bytes[pos + 4],
            wav_bytes[pos + 5],
            wav_bytes[pos + 6],
            wav_bytes[pos + 7],
        ]) as usize;

        if chunk_id == b"data" {
            let data_start = pos + 8;
            let data_end = data_start + chunk_size;
            if data_end <= wav_bytes.len() {
                // Convert PCM16 to f32
                let pcm_bytes = &wav_bytes[data_start..data_end];
                let samples: Vec<f32> = pcm_bytes
                    .chunks_exact(2)
                    .map(|chunk| {
                        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                        sample as f32 / 32768.0
                    })
                    .collect();
                return Ok(samples);
            }
        }

        pos += 8 + chunk_size;
    }

    Err("Could not find data chunk in WAV".to_string())
}

/// Process user input through the full Agent pipeline (LLM + RAG + tools)
async fn process_with_agent(state: &AppState, user_text: &str, language: &str) -> Result<String, String> {
    use voice_agent_agent::AgentConfig;

    // Create agent config with user's language
    let mut config = AgentConfig::default();
    config.language = language.to_string();

    // Create a new session with full integration (RAG + tools)
    let session = state
        .sessions
        .create_with_full_integration(
            config,
            state.vector_store.clone(),
            Some(state.tools.clone()),
        )
        .map_err(|e| format!("Failed to create session: {}", e))?;

    tracing::info!(
        session_id = %session.id,
        language = ?language,
        "Created PTT session for agent processing"
    );

    // Process through agent pipeline
    let response = session
        .agent
        .process(user_text)
        .await
        .map_err(|e| format!("Agent processing failed: {}", e))?;

    // Clean up session (PTT is stateless per-request)
    state.sessions.remove(&session.id);

    Ok(response)
}

/// Fallback response when agent processing fails
fn format_fallback_response(user_text: &str, language: &str) -> String {
    if language == "hi" {
        format!("आपने कहा: '{}'. कृपया थोड़ी देर बाद पुनः प्रयास करें।", user_text)
    } else {
        format!("You said: '{}'. Please try again in a moment.", user_text)
    }
}

/// Health check for PTT service
pub async fn ptt_health() -> impl IntoResponse {
    // Check if STT model exists
    let model_path = std::path::Path::new("models/stt/indicconformer/assets/encoder.onnx");
    let stt_ok = model_path.exists();

    if stt_ok {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ok",
                "stt_backend": "rust_indicconformer",
                "model_path": model_path.to_string_lossy()
            })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "error",
                "error": "STT model not found",
                "expected_path": model_path.to_string_lossy()
            })),
        )
    }
}
