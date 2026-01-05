#!/bin/bash
# Start the voice agent backend server with ONNX runtime

BACKEND_DIR="/home/vscode/goldloan-study/voice-agent/backend"
cd "$BACKEND_DIR"

export LIBRARY_PATH="$BACKEND_DIR/onnxruntime/lib"
export LD_LIBRARY_PATH="$BACKEND_DIR/onnxruntime/lib:$LD_LIBRARY_PATH"
export RUST_LOG="${RUST_LOG:-debug}"

# Kill existing server
pkill -f "target/release/voice-agent" 2>/dev/null || true
sleep 1

# Start server
echo "Starting voice-agent-server..."
./target/release/voice-agent 2>&1 | tee /tmp/voice-agent-server.log
