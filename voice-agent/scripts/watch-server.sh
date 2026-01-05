#!/bin/bash
# Watch mode for voice agent - auto-rebuilds and restarts on file changes
# Usage: ./scripts/watch-server.sh

set -e

cd /home/vscode/goldloan-study/voice-agent/backend

# Environment setup
export LIBRARY_PATH="/home/vscode/goldloan-study/voice-agent/backend/onnxruntime/lib:$LIBRARY_PATH"
export LD_LIBRARY_PATH="/home/vscode/goldloan-study/voice-agent/backend/onnxruntime/lib:$LD_LIBRARY_PATH"
export ORT_LIB_LOCATION="/home/vscode/goldloan-study/voice-agent/backend/onnxruntime"
export ORT_PREFER_DYNAMIC_LINK=1

# Enable all debug logging
export RUST_LOG="debug,voice_agent=debug,voice_agent_pipeline=debug,voice_agent_server=debug,hyper=warn,h2=warn,tower=warn,tonic=warn"

# Install cargo-watch if not present
if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch..."
    cargo install cargo-watch
fi

echo "=== Voice Agent Watch Mode ==="
echo "RUST_LOG: $RUST_LOG"
echo ""
echo "Watching for changes in crates/..."
echo "Server will auto-restart on changes"
echo "Press Ctrl+C to stop"
echo ""

# Use cargo-watch to rebuild and restart on changes
# -x = execute command
# -w = watch directory
# -s = shell command (allows chaining)
# --no-vcs-ignores = don't use .gitignore (to catch config changes)
cargo watch \
    -w crates \
    -w config \
    -s 'pkill -f "target/release/voice-agent" 2>/dev/null || true; \
        cargo build --release && \
        echo "Starting server..." && \
        ./target/release/voice-agent &
        sleep 2 && \
        curl -s http://localhost:8081/health | jq -r .status'
