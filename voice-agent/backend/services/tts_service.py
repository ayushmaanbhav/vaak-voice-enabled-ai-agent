#!/usr/bin/env python3
"""
Hybrid TTS Service

- Piper TTS for English (fast, ~0.1s)
- Indic Parler TTS for Hindi and other Indic languages (~10s with INT8)
"""

import io
import re
import time
import logging
import base64
import wave
from flask import Flask, request, jsonify
import numpy as np
import torch

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = Flask(__name__)

# Global model instances
piper_voice = None
parler_model = None
parler_tokenizer = None
device = None

# Paths
PIPER_MODEL_PATH = "/home/vscode/goldloan-study/voice-agent/backend/models/tts/piper/en_US-amy-medium.onnx"
PARLER_MODEL_PATH = "/home/vscode/goldloan-study/voice-agent/backend/models/tts/indic-parler-tts"

# Voice description for Parler (Indian languages)
PARLER_VOICE_DESC = "A female speaker with a clear Indian accent delivers the speech at a moderate pace with a calm and professional tone."

# Devanagari/Indic script detection pattern
INDIC_PATTERN = re.compile(r'[\u0900-\u097F\u0980-\u09FF\u0A00-\u0A7F\u0A80-\u0AFF\u0B00-\u0B7F\u0B80-\u0BFF\u0C00-\u0C7F\u0C80-\u0CFF\u0D00-\u0D7F]')


def detect_language(text: str) -> str:
    """Detect if text is English or Indic."""
    indic_chars = len(INDIC_PATTERN.findall(text))
    total_chars = len(text.replace(" ", ""))
    if total_chars == 0:
        return "en"
    indic_ratio = indic_chars / total_chars
    return "indic" if indic_ratio > 0.3 else "en"


def get_piper():
    """Load Piper TTS for English."""
    global piper_voice
    if piper_voice is None:
        from piper import PiperVoice
        logger.info(f"Loading Piper voice from: {PIPER_MODEL_PATH}")
        piper_voice = PiperVoice.load(PIPER_MODEL_PATH)
        logger.info("Piper voice loaded successfully")
    return piper_voice


def get_parler():
    """Load Parler TTS for Indic languages with INT8 quantization."""
    global parler_model, parler_tokenizer, device

    if parler_model is None:
        from parler_tts import ParlerTTSForConditionalGeneration
        from transformers import AutoTokenizer

        device = "cuda" if torch.cuda.is_available() else "cpu"
        logger.info(f"Loading Parler TTS on device: {device}")

        if device == "cuda":
            from transformers import BitsAndBytesConfig
            quantization_config = BitsAndBytesConfig(
                load_in_4bit=True,
                bnb_4bit_compute_dtype=torch.float16,
            )
            parler_model = ParlerTTSForConditionalGeneration.from_pretrained(
                PARLER_MODEL_PATH,
                local_files_only=True,
                quantization_config=quantization_config,
                device_map="auto",
            )
            logger.info("Parler loaded with 4-bit quantization (GPU)")
        else:
            parler_model = ParlerTTSForConditionalGeneration.from_pretrained(
                PARLER_MODEL_PATH,
                local_files_only=True,
            )
            # Apply INT8 dynamic quantization for CPU
            parler_model = torch.quantization.quantize_dynamic(
                parler_model,
                {torch.nn.Linear},
                dtype=torch.qint8
            )
            logger.info("Parler loaded with INT8 quantization (CPU)")

        parler_tokenizer = AutoTokenizer.from_pretrained(PARLER_MODEL_PATH, local_files_only=True)
        logger.info("Parler TTS loaded successfully")

    return parler_model, parler_tokenizer


def synthesize_piper(text: str) -> tuple:
    """Synthesize with Piper (English)."""
    voice = get_piper()

    # Generate audio using synthesize_wav
    audio_buffer = io.BytesIO()
    with wave.open(audio_buffer, 'wb') as wav_file:
        voice.synthesize_wav(text, wav_file)

    audio_buffer.seek(0)

    # Read back as numpy array
    with wave.open(audio_buffer, 'rb') as wav_file:
        frames = wav_file.readframes(wav_file.getnframes())
        audio = np.frombuffer(frames, dtype=np.int16)
        sample_rate = wav_file.getframerate()

    return audio, sample_rate


def synthesize_parler(text: str, description: str = None) -> tuple:
    """Synthesize with Parler (Indic languages)."""
    model, tokenizer = get_parler()

    if description is None:
        description = PARLER_VOICE_DESC

    input_ids = tokenizer(description, return_tensors="pt").input_ids.to(device)
    prompt_input_ids = tokenizer(text, return_tensors="pt").input_ids.to(device)

    with torch.no_grad():
        generation = model.generate(
            input_ids=input_ids,
            prompt_input_ids=prompt_input_ids,
        )

    audio = generation.cpu().numpy().squeeze()
    sample_rate = model.config.sampling_rate

    # Convert float to int16
    if audio.dtype in [np.float32, np.float64]:
        max_val = np.abs(audio).max()
        if max_val > 0:
            audio = audio / max_val * 0.9
        audio = (audio * 32767).astype(np.int16)

    return audio, sample_rate


def audio_to_wav_bytes(audio: np.ndarray, sample_rate: int) -> bytes:
    """Convert audio array to WAV bytes."""
    if audio.dtype in [np.float32, np.float64]:
        max_val = np.abs(audio).max()
        if max_val > 0:
            audio = audio / max_val * 0.9
        audio = (audio * 32767).astype(np.int16)

    buffer = io.BytesIO()
    with wave.open(buffer, 'wb') as wav_file:
        wav_file.setnchannels(1)
        wav_file.setsampwidth(2)
        wav_file.setframerate(sample_rate)
        wav_file.writeframes(audio.tobytes())

    return buffer.getvalue()


@app.route("/health", methods=["GET"])
def health():
    return jsonify({
        "status": "healthy",
        "engines": {
            "piper": {"model": "en_US-amy-medium", "loaded": piper_voice is not None},
            "parler": {"model": "indic-parler-tts", "loaded": parler_model is not None}
        }
    })


@app.route("/synthesize", methods=["POST"])
def synthesize():
    """
    Synthesize text to speech.

    Auto-detects language:
    - English -> Piper (fast)
    - Hindi/Indic -> Parler

    Request JSON:
    - text: Text to synthesize
    - language: (optional) Force "en" or "indic"
    - description: (optional) Voice description for Parler
    """
    start_time = time.time()

    try:
        data = request.get_json()
        if not data:
            return jsonify({"error": "No JSON data"}), 400

        text = data.get("text")
        if not text:
            return jsonify({"error": "No text provided"}), 400

        # Detect or use specified language
        language = data.get("language") or detect_language(text)
        description = data.get("description")

        # Truncate long texts (Piper handles long text well, Parler struggles)
        max_chars = 2000 if language == "en" else 500
        if len(text) > max_chars:
            text = text[:max_chars] + "..."
            logger.warning(f"Truncated text to {max_chars} chars")

        logger.info(f"TTS [{language}]: '{text[:60]}...'")

        # Synthesize based on language
        if language == "en":
            audio, sample_rate = synthesize_piper(text)
            engine = "piper"
        else:
            audio, sample_rate = synthesize_parler(text, description)
            engine = "parler"

        # Convert to WAV bytes
        wav_bytes = audio_to_wav_bytes(audio, sample_rate)
        audio_b64 = base64.b64encode(wav_bytes).decode('utf-8')

        elapsed = time.time() - start_time
        duration = len(audio) / sample_rate

        logger.info(f"TTS [{engine}] completed in {elapsed:.2f}s, audio: {duration:.2f}s")

        return jsonify({
            "audio": audio_b64,
            "format": "wav",
            "sample_rate": sample_rate,
            "duration_seconds": duration,
            "processing_time_seconds": elapsed,
            "engine": engine,
            "language": language,
        })

    except Exception as e:
        logger.error(f"TTS error: {e}", exc_info=True)
        return jsonify({"error": str(e)}), 500


if __name__ == "__main__":
    logger.info("Pre-loading Piper TTS (English)...")
    try:
        get_piper()
        logger.info("Starting hybrid TTS service on port 8092")
        logger.info("  - English: Piper (fast)")
        logger.info("  - Indic: Parler (INT8 quantized)")
        app.run(host="0.0.0.0", port=8092, threaded=True)
    except Exception as e:
        logger.error(f"Failed to start TTS service: {e}")
        raise
