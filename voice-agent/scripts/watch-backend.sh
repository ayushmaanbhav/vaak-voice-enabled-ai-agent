#!/bin/bash
# Watch and auto-reload the voice agent backend

BACKEND_DIR="/home/vscode/goldloan-study/voice-agent/backend"
cd "$BACKEND_DIR"

export LIBRARY_PATH="$BACKEND_DIR/onnxruntime/lib"
export LD_LIBRARY_PATH="$BACKEND_DIR/onnxruntime/lib:$LD_LIBRARY_PATH"
export ORT_LIB_LOCATION="$BACKEND_DIR/onnxruntime"
export ORT_PREFER_DYNAMIC_LINK=1
export RUST_LOG="${RUST_LOG:-debug}"

# Kill existing server
pkill -f "target/release/voice-agent" 2>/dev/null || true
pkill -f "target/debug/voice-agent" 2>/dev/null || true
sleep 1

echo "Starting cargo-watch (auto-reload on file changes)..."
echo "Logs will be in /tmp/voice-agent-server.log"

# Use cargo-watch to rebuild and run on changes
# -x run: execute 'cargo run'
# -w crates: watch crates directory
# -c: clear screen before each run
cargo watch -x "run --release" -w crates 2>&1 | tee /tmp/voice-agent-server.log
