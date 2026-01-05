#!/bin/bash
# Grep VAD-related logs from the server log

echo "Searching for VAD logs..."
grep -i "vad\|silero\|speech" /tmp/voice-agent-server.log | tail -100
