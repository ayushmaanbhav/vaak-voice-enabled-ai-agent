#!/usr/bin/env python3
"""
Automated audio pipeline test script.
Sends audio file to WebSocket and captures responses.
"""

import asyncio
import json
import struct
import sys
import wave
from pathlib import Path

try:
    import websockets
except ImportError:
    print("Installing websockets...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "websockets", "-q"])
    import websockets

try:
    import numpy as np
except ImportError:
    print("Installing numpy...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "numpy", "-q"])
    import numpy as np

# Configuration
BACKEND_URL = "ws://localhost:8080"
TARGET_SAMPLE_RATE = 16000
CHUNK_SIZE_MS = 80  # Send 80ms chunks like browser
TIMEOUT_SECONDS = 30


def load_and_resample_audio(audio_path: str) -> bytes:
    """Load audio file and resample to 16kHz mono PCM."""
    with wave.open(audio_path, 'rb') as wf:
        sample_rate = wf.getframerate()
        n_channels = wf.getnchannels()
        sample_width = wf.getsampwidth()
        n_frames = wf.getnframes()

        print(f"[Audio] Loading: {audio_path}")
        print(f"[Audio] Original: {sample_rate}Hz, {n_channels}ch, {sample_width*8}bit, {n_frames} frames")

        raw_data = wf.readframes(n_frames)

    # Convert to numpy array
    if sample_width == 2:
        audio = np.frombuffer(raw_data, dtype=np.int16)
    else:
        raise ValueError(f"Unsupported sample width: {sample_width}")

    # Convert stereo to mono
    if n_channels == 2:
        audio = audio.reshape(-1, 2).mean(axis=1).astype(np.int16)

    # Resample if needed
    if sample_rate != TARGET_SAMPLE_RATE:
        # Simple resampling using linear interpolation
        duration = len(audio) / sample_rate
        target_samples = int(duration * TARGET_SAMPLE_RATE)
        indices = np.linspace(0, len(audio) - 1, target_samples)
        audio = np.interp(indices, np.arange(len(audio)), audio).astype(np.int16)
        print(f"[Audio] Resampled to: {TARGET_SAMPLE_RATE}Hz, {len(audio)} samples")

    return audio.tobytes()


async def create_session() -> str:
    """Create a new session via HTTP API."""
    import urllib.request
    import json

    req = urllib.request.Request(
        f"http://localhost:8080/api/sessions",
        method="POST",
        headers={"Content-Type": "application/json"},
        data=json.dumps({}).encode()
    )

    with urllib.request.urlopen(req) as resp:
        data = json.loads(resp.read().decode())
        session_id = data.get("session_id") or data.get("id")
        print(f"[Session] Created: {session_id}")
        return session_id


async def test_audio_pipeline(audio_path: str):
    """Send audio to WebSocket and capture responses."""

    # Load audio
    pcm_data = load_and_resample_audio(audio_path)
    samples_per_chunk = int(TARGET_SAMPLE_RATE * CHUNK_SIZE_MS / 1000)
    bytes_per_chunk = samples_per_chunk * 2  # 16-bit = 2 bytes

    total_chunks = len(pcm_data) // bytes_per_chunk
    duration_ms = len(pcm_data) / 2 / TARGET_SAMPLE_RATE * 1000
    print(f"[Audio] Will send {total_chunks} chunks ({duration_ms:.0f}ms total)")

    # Create session
    session_id = await create_session()

    # Connect to WebSocket
    ws_url = f"{BACKEND_URL}/ws/{session_id}"
    print(f"[WS] Connecting to: {ws_url}")

    responses = []
    transcripts = []

    async with websockets.connect(ws_url) as ws:
        print("[WS] Connected!")

        # Task to receive messages
        async def receiver():
            try:
                async for msg in ws:
                    if isinstance(msg, str):
                        data = json.loads(msg)
                        msg_type = data.get("type", "unknown")

                        if msg_type == "transcript":
                            text = data.get("text", "")
                            is_final = data.get("is_final", False)
                            print(f"[Transcript] {'FINAL' if is_final else 'partial'}: {text}")
                            transcripts.append(data)
                        elif msg_type == "response":
                            print(f"[Response] {data.get('text', '')[:100]}...")
                            responses.append(data)
                        elif msg_type == "response_audio":
                            print(f"[Audio Response] Received audio chunk")
                        elif msg_type == "status":
                            print(f"[Status] state={data.get('state')} stage={data.get('stage')}")
                        elif msg_type == "session_info":
                            print(f"[Session] ID confirmed: {data.get('session_id')}")
                        else:
                            print(f"[{msg_type}] {str(data)[:100]}")
            except websockets.exceptions.ConnectionClosed:
                print("[WS] Connection closed")

        # Start receiver task
        recv_task = asyncio.create_task(receiver())

        # Send audio chunks with realistic timing
        print(f"\n[Sending audio...]")
        for i in range(total_chunks):
            chunk_start = i * bytes_per_chunk
            chunk_end = chunk_start + bytes_per_chunk
            chunk = pcm_data[chunk_start:chunk_end]

            # Send as binary
            await ws.send(chunk)

            if i % 10 == 0:
                print(f"  Sent chunk {i}/{total_chunks}")

            # Simulate real-time playback
            await asyncio.sleep(CHUNK_SIZE_MS / 1000)

        print(f"\n[Audio sent! Sending silence frames to trigger turn completion...]")

        # Send silence frames to trigger VAD end-of-speech detection
        # Need more silence because STT processing is ~100ms/chunk and threshold is 500ms
        silence_samples = int(TARGET_SAMPLE_RATE * 3.0)  # 3 seconds of silence
        silence_bytes = bytes(silence_samples * 2)  # 16-bit = 2 bytes per sample
        silence_chunks = len(silence_bytes) // bytes_per_chunk
        print(f"  Sending {silence_chunks} silence chunks")

        for i in range(silence_chunks):
            chunk_start = i * bytes_per_chunk
            chunk_end = chunk_start + bytes_per_chunk
            chunk = silence_bytes[chunk_start:chunk_end]
            await ws.send(chunk)
            await asyncio.sleep(CHUNK_SIZE_MS / 1000)

        print(f"\n[Silence sent! Waiting for processing...]")

        # Wait for responses with timeout
        try:
            await asyncio.wait_for(asyncio.sleep(10), timeout=TIMEOUT_SECONDS)
        except asyncio.TimeoutError:
            pass

        recv_task.cancel()
        try:
            await recv_task
        except asyncio.CancelledError:
            pass

    # Summary
    print("\n" + "="*60)
    print("SUMMARY")
    print("="*60)
    print(f"Transcripts received: {len(transcripts)}")
    for t in transcripts:
        print(f"  - {'[FINAL]' if t.get('is_final') else '[partial]'} {t.get('text', '')}")
    print(f"Responses received: {len(responses)}")
    for r in responses:
        print(f"  - {r.get('text', '')[:100]}")

    return len(transcripts) > 0 or len(responses) > 0


async def main():
    # Default audio file
    audio_path = sys.argv[1] if len(sys.argv) > 1 else \
        "/home/vscode/goldloan-study/voice-agent/backend/models/tts/IndicF5/samples/namaste.wav"

    if not Path(audio_path).exists():
        print(f"Error: Audio file not found: {audio_path}")
        sys.exit(1)

    success = await test_audio_pipeline(audio_path)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    asyncio.run(main())
