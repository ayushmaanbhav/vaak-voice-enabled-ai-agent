#!/usr/bin/env python3
"""
Compare Python and Rust STT implementations.

This script:
1. Loads test audio
2. Runs Python STT (ground truth)
3. Sends audio to Rust server for STT
4. Compares the results
"""

import sys
sys.path.insert(0, 'services/stt')

import soundfile as sf
import numpy as np
import requests
import json
import base64
import io
import wave
import time

# Load and preprocess audio
print("Loading audio...")
audio, sr = sf.read("models/stt/hindi-quantized/file.wav")
if audio.ndim > 1:
    audio = audio.mean(axis=1)
audio = audio.astype(np.float32)

# Resample to 16kHz
if sr != 16000:
    from librosa import resample
    audio = resample(audio, orig_sr=sr, target_sr=16000)
    sr = 16000

print(f"Audio: {len(audio)} samples at {sr}Hz ({len(audio)/sr:.2f}s)")

# Use only first 5 seconds for faster testing
audio = audio[:80000]  # 5 seconds
print(f"Using first 5s: {len(audio)} samples")

# Test 1: Python STT
print("\n=== Python STT ===")
from indicconformer_backend import IndicConformerCTC
model = IndicConformerCTC(language='hi')
start = time.time()
python_result = model.transcribe(audio, sr)
python_time = time.time() - start
print(f"Result: {python_result}")
print(f"Time: {python_time:.2f}s")

# Test 2: Rust STT (via health check to initialize, then transcribe endpoint)
print("\n=== Rust STT ===")

# Create WAV bytes
wav_buffer = io.BytesIO()
with wave.open(wav_buffer, 'wb') as wav_file:
    wav_file.setnchannels(1)
    wav_file.setsampwidth(2)
    wav_file.setframerate(16000)
    wav_file.writeframes((audio * 32767).astype(np.int16).tobytes())
wav_bytes = wav_buffer.getvalue()
audio_b64 = base64.b64encode(wav_bytes).decode()

# Check if server is running
try:
    health = requests.get("http://localhost:8080/health", timeout=5)
    if health.ok:
        print("Server is healthy")
    else:
        print("Server not healthy, skipping Rust test")
        sys.exit(0)
except Exception as e:
    print(f"Server not running: {e}")
    sys.exit(0)

# Send to Rust /api/ptt/process (with longer timeout)
start = time.time()
try:
    resp = requests.post(
        "http://localhost:8080/api/ptt/process",
        json={
            "audio": audio_b64,
            "audio_format": "wav",
            "language": "hi"
        },
        headers={"Content-Type": "application/json"},
        timeout=120  # 2 minute timeout for LLM processing
    )
    rust_time = time.time() - start
    
    if resp.ok:
        result = resp.json()
        print(f"Result (user_text): {result.get('user_text', 'N/A')}")
        print(f"Time: {rust_time:.2f}s")
        
        # Compare
        print("\n=== Comparison ===")
        print(f"Python: {python_result}")
        print(f"Rust:   {result.get('user_text', 'N/A')}")
    else:
        print(f"Error: {resp.status_code}")
        print(resp.text[:500])
except Exception as e:
    print(f"Error: {e}")

