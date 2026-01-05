#!/bin/bash
# Development server with auto-rebuild and restart
# Usage: ./scripts/dev-server.sh

set -e

cd /home/vscode/goldloan-study/voice-agent/backend

# Environment setup
export LIBRARY_PATH="$PWD/onnxruntime/lib:$LIBRARY_PATH"
export LD_LIBRARY_PATH="$PWD/onnxruntime/lib:$LD_LIBRARY_PATH"
export ORT_LIB_LOCATION="$PWD/onnxruntime"
export ORT_PREFER_DYNAMIC_LINK=1
export RUST_LOG="${RUST_LOG:-info}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to kill existing server
kill_server() {
    echo -e "${YELLOW}Stopping server...${NC}"
    pkill -f "target/release/voice-agent" 2>/dev/null || true
    sleep 1
}

# Function to build
build() {
    echo -e "${YELLOW}Building...${NC}"
    if cargo build --release 2>&1; then
        echo -e "${GREEN}Build successful${NC}"
        return 0
    else
        echo -e "${RED}Build failed${NC}"
        return 1
    fi
}

# Function to start server
start_server() {
    echo -e "${YELLOW}Starting server...${NC}"
    ./target/release/voice-agent &
    SERVER_PID=$!
    sleep 2

    if curl -s http://localhost:8081/health | grep -q "healthy"; then
        echo -e "${GREEN}Server started (PID: $SERVER_PID)${NC}"
        echo -e "${GREEN}Health: $(curl -s http://localhost:8081/health | jq -r .status)${NC}"
        return 0
    else
        echo -e "${RED}Server failed to start${NC}"
        return 1
    fi
}

# Function for rebuild and restart
rebuild() {
    kill_server
    if build; then
        start_server
    fi
}

# Main
echo -e "${GREEN}=== Voice Agent Dev Server ===${NC}"
echo "RUST_LOG: $RUST_LOG"
echo ""

# Initial build and start
rebuild

echo ""
echo -e "${GREEN}Server running. Commands:${NC}"
echo "  r - rebuild and restart"
echo "  l - view logs (tail -f /tmp/backend.log)"
echo "  k - kill server"
echo "  q - quit"
echo ""

# Wait for commands
while true; do
    read -n1 -r cmd
    case $cmd in
        r|R)
            echo ""
            rebuild
            ;;
        l|L)
            echo ""
            tail -100 /tmp/backend-debug.log 2>/dev/null || echo "No logs found"
            ;;
        k|K)
            echo ""
            kill_server
            ;;
        q|Q)
            echo ""
            kill_server
            echo -e "${GREEN}Goodbye!${NC}"
            exit 0
            ;;
        *)
            ;;
    esac
done
