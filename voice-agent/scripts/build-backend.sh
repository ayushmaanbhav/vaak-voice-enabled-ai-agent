#!/bin/bash
# Build the voice agent backend with ONNX runtime

BACKEND_DIR="/home/vscode/goldloan-study/voice-agent/backend"
cd "$BACKEND_DIR"

export LIBRARY_PATH="$BACKEND_DIR/onnxruntime/lib"
export ORT_LIB_LOCATION="$BACKEND_DIR/onnxruntime"
export ORT_PREFER_DYNAMIC_LINK=1

echo "Building voice-agent-server..."
cargo build --release
