#!/usr/bin/env python3
"""
STT Service - Multi-engine Speech-to-Text
Supports:
- Whisper (for English)
- IndicConformer (for Hindi and other Indic languages)

Runs on port 8090 by default.
"""

import os
import sys
import json
import numpy as np
from pathlib import Path
from typing import Optional
import logging

# Setup logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

# FastAPI imports
try:
    from fastapi import FastAPI, HTTPException
    from fastapi.responses import JSONResponse
    from pydantic import BaseModel
    import uvicorn
except ImportError:
    logger.error("FastAPI not installed. Run: pip install fastapi uvicorn")
    sys.exit(1)

# Whisper imports
whisper_model = None
whisper_processor = None

def load_whisper():
    """Load Whisper model for English STT"""
    global whisper_model, whisper_processor
    if whisper_model is not None:
        return True

    try:
        import torch
        from transformers import WhisperProcessor, WhisperForConditionalGeneration

        model_path = Path(__file__).parent.parent / "whisper-small"
        if not model_path.exists():
            model_path = Path("whisper-small")

        if not model_path.exists():
            logger.warning(f"Whisper model not found at {model_path}")
            return False

        logger.info(f"Loading Whisper from {model_path}")
        whisper_processor = WhisperProcessor.from_pretrained(str(model_path))
        whisper_model = WhisperForConditionalGeneration.from_pretrained(str(model_path))

        # Use GPU if available
        device = "cuda" if torch.cuda.is_available() else "cpu"
        whisper_model = whisper_model.to(device)
        logger.info(f"Whisper loaded on {device}")
        return True
    except Exception as e:
        logger.error(f"Failed to load Whisper: {e}")
        return False

# IndicConformer imports
indicconformer_model = None

def load_indicconformer():
    """Load IndicConformer for Hindi STT"""
    global indicconformer_model
    if indicconformer_model is not None:
        return True

    try:
        # Try to import ai4bharat's IndicConformer
        # This requires the ai4bharat-transliteration package
        logger.info("IndicConformer loading not implemented yet - using Whisper fallback")
        return False
    except Exception as e:
        logger.error(f"Failed to load IndicConformer: {e}")
        return False

# FastAPI app
app = FastAPI(title="STT Service", version="1.0.0")

class TranscribeRequest(BaseModel):
    audio: list[float]  # PCM f32 samples at 16kHz
    language: str = "en"
    sample_rate: int = 16000

class TranscribeResponse(BaseModel):
    text: str
    confidence: float
    language: str
    backend: str
    error: Optional[str] = None

@app.on_event("startup")
async def startup():
    """Load models on startup"""
    logger.info("Loading STT models...")
    load_whisper()
    load_indicconformer()
    logger.info("STT service ready")

@app.get("/health")
async def health():
    """Health check"""
    return {"status": "ok", "whisper": whisper_model is not None}

@app.post("/transcribe", response_model=TranscribeResponse)
async def transcribe(request: TranscribeRequest):
    """Transcribe audio to text"""
    try:
        import torch

        audio = np.array(request.audio, dtype=np.float32)
        language = request.language.lower()

        # Route based on language
        if language in ["en", "english"]:
            if whisper_model is None:
                if not load_whisper():
                    raise HTTPException(status_code=503, detail="Whisper model not available")

            # Process with Whisper
            device = next(whisper_model.parameters()).device

            # Prepare input
            input_features = whisper_processor(
                audio,
                sampling_rate=request.sample_rate,
                return_tensors="pt"
            ).input_features.to(device)

            # Generate
            with torch.no_grad():
                predicted_ids = whisper_model.generate(
                    input_features,
                    language="en",
                    task="transcribe",
                    max_length=448
                )

            # Decode
            text = whisper_processor.batch_decode(predicted_ids, skip_special_tokens=True)[0]

            return TranscribeResponse(
                text=text.strip(),
                confidence=0.9,  # Whisper doesn't provide confidence
                language="en",
                backend="whisper-small"
            )

        elif language in ["hi", "hindi"]:
            # Use IndicConformer for Hindi
            if indicconformer_model is None:
                # Fallback to Whisper with Hindi
                if whisper_model is None:
                    if not load_whisper():
                        raise HTTPException(status_code=503, detail="No STT model available")

                device = next(whisper_model.parameters()).device
                input_features = whisper_processor(
                    audio,
                    sampling_rate=request.sample_rate,
                    return_tensors="pt"
                ).input_features.to(device)

                with torch.no_grad():
                    predicted_ids = whisper_model.generate(
                        input_features,
                        language="hi",
                        task="transcribe",
                        max_length=448
                    )

                text = whisper_processor.batch_decode(predicted_ids, skip_special_tokens=True)[0]

                return TranscribeResponse(
                    text=text.strip(),
                    confidence=0.85,
                    language="hi",
                    backend="whisper-small-hindi-fallback"
                )

            # TODO: Use actual IndicConformer
            return TranscribeResponse(
                text="",
                confidence=0.0,
                language="hi",
                backend="indicconformer",
                error="IndicConformer not implemented"
            )

        else:
            # Try Whisper with auto language detection
            if whisper_model is None:
                if not load_whisper():
                    raise HTTPException(status_code=503, detail="No STT model available")

            device = next(whisper_model.parameters()).device
            input_features = whisper_processor(
                audio,
                sampling_rate=request.sample_rate,
                return_tensors="pt"
            ).input_features.to(device)

            with torch.no_grad():
                predicted_ids = whisper_model.generate(
                    input_features,
                    task="transcribe",
                    max_length=448
                )

            text = whisper_processor.batch_decode(predicted_ids, skip_special_tokens=True)[0]

            return TranscribeResponse(
                text=text.strip(),
                confidence=0.8,
                language=language,
                backend="whisper-small-auto"
            )

    except HTTPException:
        raise
    except Exception as e:
        logger.exception("Transcription failed")
        return TranscribeResponse(
            text="",
            confidence=0.0,
            language=request.language,
            backend="error",
            error=str(e)
        )

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description="STT Service")
    parser.add_argument("--port", type=int, default=8090, help="Port to listen on")
    parser.add_argument("--host", type=str, default="0.0.0.0", help="Host to bind to")
    args = parser.parse_args()

    logger.info(f"Starting STT service on {args.host}:{args.port}")
    uvicorn.run(app, host=args.host, port=args.port, log_level="info")
