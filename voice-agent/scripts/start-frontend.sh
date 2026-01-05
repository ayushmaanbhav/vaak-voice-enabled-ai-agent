#!/bin/bash
# Start the voice agent frontend dev server

FRONTEND_DIR="/home/vscode/goldloan-study/voice-agent/frontend"
cd "$FRONTEND_DIR"

echo "Starting frontend dev server..."
npm run dev
