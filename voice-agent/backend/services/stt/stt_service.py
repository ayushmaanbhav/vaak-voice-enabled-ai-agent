#!/usr/bin/env python3
"""
Multi-language STT Service - Supports IndicConformer (Hindi/Indic) + Whisper (English/multilingual)

Extensible architecture:
- STTBackend base class for adding new backends
- IndicConformerBackend: Hindi and 21 other Indian languages
- WhisperBackend: English and multilingual fallback

Usage:
    python stt_service.py --port 8090

API:
    POST /transcribe
        Headers:
            Content-Type: audio/pcm (default) or audio/float32
            X-Language: hi|en|auto (default: auto)
        Body: raw audio bytes (16kHz mono)
        Response: {"text": "...", "confidence": 0.95, "language": "hi", "backend": "indicconformer"}

    GET /health
        Response: {"status": "ok", "backends": ["indicconformer", "whisper"]}
"""

import os
import sys
import json
import argparse
import numpy as np
from abc import ABC, abstractmethod
from http.server import HTTPServer, BaseHTTPRequestHandler
from typing import Optional, Tuple, Dict, List
from dataclasses import dataclass

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
BACKEND_DIR = os.path.join(SCRIPT_DIR, '..', '..')
MODELS_DIR = os.path.join(BACKEND_DIR, 'models')


@dataclass
class TranscriptionResult:
    text: str
    confidence: float
    language: str
    backend: str


class STTBackend(ABC):
    """Base class for STT backends. Extend this to add new backends."""

    @property
    @abstractmethod
    def name(self) -> str:
        """Backend identifier."""
        pass

    @property
    @abstractmethod
    def supported_languages(self) -> List[str]:
        """List of supported language codes."""
        pass

    @abstractmethod
    def transcribe(self, audio: np.ndarray, sample_rate: int = 16000) -> TranscriptionResult:
        """Transcribe audio to text."""
        pass

    @abstractmethod
    def is_available(self) -> bool:
        """Check if this backend is available (model files exist, deps installed)."""
        pass


class IndicConformerBackend(STTBackend):
    """
    IndicConformer CTC backend for Indian languages.
    Uses original ai4bharat model with language mask filtering.
    """

    INDIC_LANGUAGES = [
        'hi',   # Hindi
        'as',   # Assamese
        'bn',   # Bengali
        'brx',  # Bodo
        'doi',  # Dogri
        'gu',   # Gujarati
        'kn',   # Kannada
        'kok',  # Konkani
        'ks',   # Kashmiri
        'mai',  # Maithili
        'ml',   # Malayalam
        'mni',  # Manipuri
        'mr',   # Marathi
        'ne',   # Nepali
        'or',   # Odia
        'pa',   # Punjabi
        'sa',   # Sanskrit
        'sat',  # Santali
        'sd',   # Sindhi
        'ta',   # Tamil
        'te',   # Telugu
        'ur',   # Urdu
    ]

    def __init__(self, language: str = 'hi'):
        self._model = None
        self._language = language
        self._model_dir = os.path.join(MODELS_DIR, 'stt', 'indicconformer')

    @property
    def name(self) -> str:
        return 'indicconformer'

    @property
    def supported_languages(self) -> List[str]:
        return self.INDIC_LANGUAGES

    def is_available(self) -> bool:
        # Check for original model files
        assets_dir = os.path.join(self._model_dir, 'assets')
        encoder_path = os.path.join(assets_dir, 'encoder.onnx')
        decoder_path = os.path.join(assets_dir, 'ctc_decoder.onnx')
        vocab_path = os.path.join(assets_dir, 'vocab.json')
        mask_path = os.path.join(assets_dir, 'language_masks.json')
        return (os.path.exists(encoder_path) and os.path.exists(decoder_path) and
                os.path.exists(vocab_path) and os.path.exists(mask_path))

    def _load_model(self):
        if self._model is None:
            from indicconformer_backend import IndicConformerCTC
            self._model = IndicConformerCTC(model_dir=self._model_dir, language=self._language)
            print(f"[IndicConformer] Loaded model for language: {self._language}")

    def transcribe(self, audio: np.ndarray, sample_rate: int = 16000) -> TranscriptionResult:
        self._load_model()
        try:
            text = self._model.transcribe(audio, sample_rate)
            return TranscriptionResult(
                text=text.strip(),
                confidence=0.85,  # Model doesn't provide confidence
                language=self._language,
                backend=self.name
            )
        except Exception as e:
            print(f"[IndicConformer] Transcription error: {e}")
            import traceback
            traceback.print_exc()
            return TranscriptionResult(text='', confidence=0.0, language=self._language, backend=self.name)


class WhisperBackend(STTBackend):
    """
    OpenAI Whisper backend for English and multilingual.
    Uses faster-whisper for efficient inference.
    """

    def __init__(self, model_size: str = 'base'):
        self._model = None
        self._model_size = model_size

    @property
    def name(self) -> str:
        return 'whisper'

    @property
    def supported_languages(self) -> List[str]:
        # Whisper supports 99 languages, but we list common ones
        return ['en', 'hi', 'auto']

    def is_available(self) -> bool:
        try:
            import faster_whisper
            return True
        except ImportError:
            # Try regular whisper
            try:
                import whisper
                return True
            except ImportError:
                return False

    def _load_model(self):
        if self._model is None:
            try:
                # Try faster-whisper first (more efficient)
                from faster_whisper import WhisperModel
                self._model = WhisperModel(self._model_size, device='cpu', compute_type='int8')
                self._use_faster = True
                print(f"[Whisper] Loaded faster-whisper model: {self._model_size}")
            except ImportError:
                # Fall back to regular whisper
                import whisper
                self._model = whisper.load_model(self._model_size)
                self._use_faster = False
                print(f"[Whisper] Loaded whisper model: {self._model_size}")

    def transcribe(self, audio: np.ndarray, sample_rate: int = 16000) -> TranscriptionResult:
        self._load_model()
        try:
            if self._use_faster:
                segments, info = self._model.transcribe(audio, language='en')
                text = ' '.join([seg.text for seg in segments])
                confidence = info.language_probability if hasattr(info, 'language_probability') else 0.8
                language = info.language if hasattr(info, 'language') else 'en'
            else:
                result = self._model.transcribe(audio, language='en')
                text = result['text']
                confidence = 0.8
                language = result.get('language', 'en')

            return TranscriptionResult(
                text=text.strip(),
                confidence=confidence,
                language=language,
                backend=self.name
            )
        except Exception as e:
            print(f"[Whisper] Transcription error: {e}")
            import traceback
            traceback.print_exc()
            return TranscriptionResult(text='', confidence=0.0, language='en', backend=self.name)


class STTRouter:
    """
    Routes transcription requests to appropriate backend based on language.
    Extensible - just add new backends to the list.
    """

    def __init__(self):
        self.backends: Dict[str, STTBackend] = {}
        self._default_backend: Optional[str] = None

        # Register available backends
        self._register_backends()

    def _register_backends(self):
        """Register all available backends."""
        # Priority order: IndicConformer for Indic, Whisper for English/fallback
        backends_to_try = [
            IndicConformerBackend(),
            WhisperBackend(model_size='base'),
        ]

        for backend in backends_to_try:
            if backend.is_available():
                self.backends[backend.name] = backend
                print(f"[Router] Registered backend: {backend.name} (languages: {backend.supported_languages[:5]}...)")
                if self._default_backend is None:
                    self._default_backend = backend.name
            else:
                print(f"[Router] Backend not available: {backend.name}")

    def get_backend_for_language(self, language: str) -> Optional[STTBackend]:
        """Get the best backend for a given language."""
        # Check IndicConformer first for Indic languages
        if 'indicconformer' in self.backends:
            if language in self.backends['indicconformer'].supported_languages:
                return self.backends['indicconformer']

        # Use Whisper for English
        if language == 'en' and 'whisper' in self.backends:
            return self.backends['whisper']

        # Auto-detect: try IndicConformer first (primary use case), fall back to Whisper
        if language == 'auto':
            if 'indicconformer' in self.backends:
                return self.backends['indicconformer']
            if 'whisper' in self.backends:
                return self.backends['whisper']

        # Fall back to default
        if self._default_backend:
            return self.backends[self._default_backend]

        return None

    def transcribe(self, audio: np.ndarray, language: str = 'auto', sample_rate: int = 16000) -> TranscriptionResult:
        """Transcribe audio using appropriate backend."""
        backend = self.get_backend_for_language(language)
        if backend is None:
            return TranscriptionResult(
                text='',
                confidence=0.0,
                language=language,
                backend='none'
            )
        return backend.transcribe(audio, sample_rate)

    def available_backends(self) -> List[str]:
        return list(self.backends.keys())


def pcm16_to_float32(pcm_bytes: bytes) -> np.ndarray:
    """Convert PCM16 bytes to float32 numpy array."""
    samples = np.frombuffer(pcm_bytes, dtype=np.int16)
    return samples.astype(np.float32) / 32768.0


# Global router instance
_router: Optional[STTRouter] = None

def get_router() -> STTRouter:
    global _router
    if _router is None:
        _router = STTRouter()
    return _router


class STTHandler(BaseHTTPRequestHandler):
    """HTTP handler for STT requests."""

    def log_message(self, format, *args):
        print(f"[HTTP] {args[0]}")

    def send_json(self, data: dict, status: int = 200):
        body = json.dumps(data, ensure_ascii=False).encode('utf-8')
        self.send_response(status)
        self.send_header('Content-Type', 'application/json; charset=utf-8')
        self.send_header('Content-Length', len(body))
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(body)

    def do_OPTIONS(self):
        """Handle CORS preflight."""
        self.send_response(200)
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type, X-Language')
        self.end_headers()

    def do_GET(self):
        if self.path == '/health':
            router = get_router()
            self.send_json({
                'status': 'ok',
                'backends': router.available_backends(),
                'default_backend': router._default_backend
            })
        else:
            self.send_json({'error': 'Not found'}, 404)

    def do_POST(self):
        if self.path == '/transcribe':
            self._handle_transcribe()
        else:
            self.send_json({'error': 'Not found'}, 404)

    def _handle_transcribe(self):
        try:
            content_length = int(self.headers.get('Content-Length', 0))
            if content_length == 0:
                self.send_json({'error': 'No audio data'}, 400)
                return

            # Get language hint from header
            language = self.headers.get('X-Language', 'auto').lower()

            # Read audio
            audio_bytes = self.rfile.read(content_length)
            content_type = self.headers.get('Content-Type', 'audio/pcm')

            if 'float32' in content_type:
                audio = np.frombuffer(audio_bytes, dtype=np.float32)
            else:
                audio = pcm16_to_float32(audio_bytes)

            # Minimum length check (100ms at 16kHz)
            if len(audio) < 1600:
                self.send_json({
                    'text': '',
                    'confidence': 0.0,
                    'language': language,
                    'backend': 'none',
                    'error': 'Audio too short (min 100ms)'
                })
                return

            # Transcribe
            router = get_router()
            result = router.transcribe(audio, language=language)

            self.send_json({
                'text': result.text,
                'confidence': result.confidence,
                'language': result.language,
                'backend': result.backend
            })

        except Exception as e:
            print(f"[Error] {e}")
            import traceback
            traceback.print_exc()
            self.send_json({'error': str(e)}, 500)


def main():
    parser = argparse.ArgumentParser(description='Multi-language STT Service')
    parser.add_argument('--host', default='127.0.0.1', help='Host to bind to')
    parser.add_argument('--port', type=int, default=8090, help='Port to listen on')
    parser.add_argument('--preload', action='store_true', help='Preload models on startup')
    args = parser.parse_args()

    print("=" * 60)
    print("Multi-language STT Service")
    print("=" * 60)

    # Initialize router (loads backend info)
    router = get_router()

    if args.preload:
        print("\nPreloading models...")
        for name, backend in router.backends.items():
            try:
                # Trigger model load with dummy audio
                dummy = np.zeros(16000, dtype=np.float32)
                backend.transcribe(dummy)
                print(f"  {name}: loaded")
            except Exception as e:
                print(f"  {name}: failed - {e}")

    print(f"\nListening on http://{args.host}:{args.port}")
    print("\nEndpoints:")
    print("  GET  /health     - Health check")
    print("  POST /transcribe - Transcribe audio")
    print("       Headers: X-Language: hi|en|auto (default: auto)")
    print("       Body: PCM16 audio bytes (16kHz mono)")
    print("\nPress Ctrl+C to stop")
    print("=" * 60)

    server = HTTPServer((args.host, args.port), STTHandler)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down...")
        server.shutdown()


if __name__ == '__main__':
    main()
