#!/usr/bin/env python3
"""
Faster-Whisper STT Service

Fast CPU-optimized Whisper using CTranslate2 int8 quantization.
~4x faster than vanilla whisper on CPU.

Features:
- Domain-specific vocabulary biasing via initial_prompt
- Contextual biasing for gold loan terminology
- Based on research: https://arxiv.org/abs/2410.18363

Usage:
    pip install faster-whisper flask
    python whisper_service.py
"""

import os
import io
import time
import tempfile
import logging
from flask import Flask, request, jsonify
import numpy as np

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = Flask(__name__)

# Global model instance
model = None
# Use local model path (downloaded via git clone from HuggingFace)
# whisper-small is more accurate than tiny for domain-specific audio
MODEL_PATH = os.environ.get("WHISPER_MODEL_PATH", os.path.join(os.path.dirname(__file__), "faster-whisper-small"))

# =============================================================================
# Domain-Specific Vocabulary Biasing (Contextual Biasing)
# Based on: https://arxiv.org/abs/2410.18363
# The initial_prompt parameter guides Whisper towards domain vocabulary
# =============================================================================

# Gold Loan Domain Vocabulary - structured as realistic utterances
# This primes Whisper's decoder to expect these terms
GOLD_LOAN_PROMPT = """
Kotak Mahindra Bank gold loan. Interest rate for gold loan is competitive.
Tell me about gold loan eligibility. I need help regarding gold loan.
Balance transfer from Muthoot Finance. Manappuram gold loan comparison.
IIFL gold loan versus Kotak. Processing fee and foreclosure charges.
Gold loan EMI calculator. LTV ratio for gold ornaments.
Hallmark jewelry valuation. Purity of gold in carats.
Lakhs and crores loan amount. Documents required for gold loan.
Branch near me for gold loan. Doorstep gold loan service.
"""

# Hindi/Hinglish vocabulary prompt
GOLD_LOAN_PROMPT_HINDI = """
Kotak gold loan ke baare mein batao. Gold loan ka interest rate kya hai.
Mujhe gold loan chahiye. Sone ka loan kaise milega.
Balance transfer karna hai Muthoot se. Manappuram se transfer.
Processing fee kitni hai. Foreclosure charges kya hai.
EMI calculator. LTV kitna milega. Documents kya chahiye.
Nearest branch kahan hai. Doorstep service available hai.
"""

def get_domain_prompt(language: str = "en") -> str:
    """Get domain-specific prompt for vocabulary biasing."""
    if language in ("hi", "hindi"):
        return GOLD_LOAN_PROMPT_HINDI
    return GOLD_LOAN_PROMPT

def get_model():
    global model
    if model is None:
        from faster_whisper import WhisperModel
        logger.info(f"Loading faster-whisper model from: {MODEL_PATH}")
        # Use int8 quantization for CPU, runs on CPU by default
        model = WhisperModel(
            MODEL_PATH,
            device="cpu",
            compute_type="int8",
            cpu_threads=4,
            local_files_only=True
        )
        logger.info("Faster-whisper model loaded")
    return model


@app.route("/health", methods=["GET"])
def health():
    return jsonify({"status": "healthy", "model": MODEL_PATH})


@app.route("/transcribe", methods=["POST"])
def transcribe():
    """
    Transcribe audio to text.

    Expects JSON with:
    - audio: base64-encoded audio (PCM f32 mono 16kHz) or raw bytes
    - language: language code (default: "en")
    - sample_rate: sample rate (default: 16000)
    """
    start_time = time.time()

    try:
        data = request.get_json()
        if not data:
            return jsonify({"error": "No JSON data"}), 400

        # Get audio data
        import base64
        audio_b64 = data.get("audio")
        if not audio_b64:
            return jsonify({"error": "No audio data"}), 400

        audio_bytes = base64.b64decode(audio_b64)
        language = data.get("language", "en")
        sample_rate = data.get("sample_rate", 16000)

        # Convert bytes to float32 numpy array
        audio = np.frombuffer(audio_bytes, dtype=np.float32)

        logger.info(f"Received audio: {len(audio)} samples, {len(audio)/sample_rate:.2f}s, lang={language}")

        # Get model and transcribe
        whisper = get_model()

        # Get domain-specific prompt for vocabulary biasing
        # This significantly improves recognition of domain terms
        # See: https://arxiv.org/abs/2410.18363
        domain_prompt = get_domain_prompt(language)

        # Transcribe with faster-whisper using contextual biasing
        segments, info = whisper.transcribe(
            audio,
            language=language if language != "auto" else None,
            beam_size=5,
            vad_filter=True,  # Filter out silence
            vad_parameters=dict(
                min_silence_duration_ms=500,
                speech_pad_ms=400,
            ),
            # Domain vocabulary biasing via initial_prompt
            # This primes the decoder to expect gold loan terminology
            initial_prompt=domain_prompt,
            # Condition on previous text helps with consistency
            condition_on_previous_text=True,
            # Slightly lower temperature for more deterministic output
            temperature=0.0,
            # Compression ratio threshold to detect hallucinations
            compression_ratio_threshold=2.4,
            # Log probability threshold
            log_prob_threshold=-1.0,
            # No speech threshold
            no_speech_threshold=0.6,
        )

        # Collect all segments
        text_parts = []
        for segment in segments:
            text_parts.append(segment.text.strip())

        text = " ".join(text_parts)

        elapsed = time.time() - start_time
        logger.info(f"Transcribed in {elapsed:.2f}s: '{text[:100]}...' (lang={info.language}, prob={info.language_probability:.2f})")

        return jsonify({
            "text": text,
            "language": info.language,
            "language_probability": info.language_probability,
            "duration_seconds": info.duration,
            "processing_time_seconds": elapsed,
        })

    except Exception as e:
        logger.error(f"Transcription error: {e}", exc_info=True)
        return jsonify({"error": str(e)}), 500


if __name__ == "__main__":
    # Pre-load model
    logger.info("Pre-loading faster-whisper model...")
    get_model()
    logger.info("Starting whisper service on port 8091")
    app.run(host="0.0.0.0", port=8091, threaded=True)
