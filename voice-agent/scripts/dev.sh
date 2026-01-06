#!/bin/bash
#
# Voice Agent Development Script
#
# Starts all required services for local development:
# - Qdrant vector database
# - Whisper STT service
# - IndicF5 TTS service
# - Rust backend (with cargo-watch for hot reload)
# - Frontend dev server
#
# Usage:
#   ./scripts/dev.sh          # Start all services
#   ./scripts/dev.sh backend  # Start only backend services
#   ./scripts/dev.sh frontend # Start only frontend
#
# Press Ctrl+C to stop all services

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BACKEND_DIR="$PROJECT_ROOT/backend"
FRONTEND_DIR="$PROJECT_ROOT/frontend"
SERVICES_DIR="$BACKEND_DIR/services"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Track PIDs for cleanup
PIDS=()

cleanup() {
    log_info "Shutting down services..."
    for pid in "${PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
        fi
    done
    # Kill any orphaned processes
    pkill -f "qdrant" 2>/dev/null || true
    pkill -f "whisper_service.py" 2>/dev/null || true
    pkill -f "tts_service.py" 2>/dev/null || true
    pkill -f "cargo watch" 2>/dev/null || true
    pkill -f "voice-agent-server" 2>/dev/null || true
    log_success "All services stopped"
    exit 0
}

trap cleanup SIGINT SIGTERM

# Environment variables
export RUST_LOG="${RUST_LOG:-info,voice_agent=debug}"
export RUST_BACKTRACE=1

# ONNX Runtime environment
export ORT_LIB_LOCATION="$BACKEND_DIR/onnxruntime"
export ORT_PREFER_DYNAMIC_LINK=1
export LD_LIBRARY_PATH="$BACKEND_DIR/onnxruntime/lib:$LD_LIBRARY_PATH"
export LIBRARY_PATH="$BACKEND_DIR/onnxruntime/lib:$LIBRARY_PATH"

# Qdrant
export QDRANT_URL="${QDRANT_URL:-http://localhost:6333}"

# Service ports
QDRANT_PORT=6333
WHISPER_PORT=8091
TTS_PORT=8092
BACKEND_PORT=8080
FRONTEND_PORT=5173

check_port() {
    local port=$1
    if lsof -i ":$port" >/dev/null 2>&1; then
        return 0  # Port in use
    else
        return 1  # Port free
    fi
}

wait_for_port() {
    local port=$1
    local name=$2
    local max_wait=30
    local waited=0

    while ! check_port "$port"; do
        if [ $waited -ge $max_wait ]; then
            log_error "$name did not start within ${max_wait}s"
            return 1
        fi
        sleep 1
        waited=$((waited + 1))
    done
    log_success "$name is ready on port $port"
}

start_qdrant() {
    if check_port $QDRANT_PORT; then
        log_info "Qdrant already running on port $QDRANT_PORT"
        return 0
    fi

    log_info "Starting Qdrant..."

    # Check for docker
    if command -v docker &>/dev/null && docker ps &>/dev/null; then
        docker run -d --rm \
            --name qdrant-dev \
            -p $QDRANT_PORT:6333 \
            -p 6334:6334 \
            -v "$PROJECT_ROOT/storage/qdrant:/qdrant/storage" \
            qdrant/qdrant:latest \
            >/dev/null 2>&1 || {
                # Container might already exist
                docker start qdrant-dev 2>/dev/null || true
            }
        wait_for_port $QDRANT_PORT "Qdrant"
    else
        log_warn "Docker not available, assuming Qdrant is running externally"
    fi
}

start_whisper() {
    if check_port $WHISPER_PORT; then
        log_info "Whisper service already running on port $WHISPER_PORT"
        return 0
    fi

    log_info "Starting Whisper STT service..."

    cd "$SERVICES_DIR"

    # Activate venv if exists
    if [ -d "whisper-venv" ]; then
        source whisper-venv/bin/activate
    fi

    python3 whisper_service.py 2>&1 | sed 's/^/[whisper] /' &
    PIDS+=($!)

    wait_for_port $WHISPER_PORT "Whisper STT"
}

start_tts() {
    if check_port $TTS_PORT; then
        log_info "TTS service already running on port $TTS_PORT"
        return 0
    fi

    log_info "Starting IndicF5 TTS service..."

    cd "$SERVICES_DIR"

    # Check if indicf5 is installed
    if ! python3 -c "import indicf5" 2>/dev/null; then
        log_warn "IndicF5 not installed. TTS will be unavailable."
        log_warn "Install with: pip install git+https://github.com/ai4bharat/IndicF5.git"
        return 0
    fi

    python3 tts_service.py 2>&1 | sed 's/^/[tts] /' &
    PIDS+=($!)

    wait_for_port $TTS_PORT "IndicF5 TTS"
}

start_backend() {
    if check_port $BACKEND_PORT; then
        log_warn "Port $BACKEND_PORT in use, killing existing process..."
        fuser -k $BACKEND_PORT/tcp 2>/dev/null || true
        sleep 1
    fi

    log_info "Starting Rust backend with cargo-watch..."

    cd "$BACKEND_DIR"

    # Check if cargo-watch is installed
    if ! command -v cargo-watch &>/dev/null; then
        log_warn "cargo-watch not installed, using cargo run instead"
        log_warn "Install cargo-watch for hot reload: cargo install cargo-watch"

        cargo run --package voice-agent-server 2>&1 | sed 's/^/[backend] /' &
        PIDS+=($!)
    else
        cargo watch -x "run --package voice-agent-server" 2>&1 | sed 's/^/[backend] /' &
        PIDS+=($!)
    fi

    wait_for_port $BACKEND_PORT "Backend"
}

start_frontend() {
    if check_port $FRONTEND_PORT; then
        log_info "Frontend already running on port $FRONTEND_PORT"
        return 0
    fi

    log_info "Starting frontend dev server..."

    cd "$FRONTEND_DIR"
    npm run dev 2>&1 | sed 's/^/[frontend] /' &
    PIDS+=($!)

    wait_for_port $FRONTEND_PORT "Frontend"
}

print_banner() {
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║${NC}     🎙️  Voice Agent Development Server  🎙️     ${GREEN}║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════╝${NC}"
    echo ""
}

print_urls() {
    echo ""
    echo -e "${GREEN}Services running:${NC}"
    echo -e "  📊 Qdrant:    http://localhost:$QDRANT_PORT"
    echo -e "  🎤 Whisper:   http://localhost:$WHISPER_PORT"
    echo -e "  🔊 TTS:       http://localhost:$TTS_PORT"
    echo -e "  ⚙️  Backend:   http://localhost:$BACKEND_PORT"
    echo -e "  🌐 Frontend:  http://localhost:$FRONTEND_PORT"
    echo ""
    echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}"
    echo ""
}

# Main
print_banner

case "${1:-all}" in
    backend)
        start_qdrant
        start_whisper
        start_tts
        start_backend
        ;;
    frontend)
        start_frontend
        ;;
    all|*)
        start_qdrant
        start_whisper
        start_tts
        start_backend
        start_frontend
        ;;
esac

print_urls

# Wait for all processes
wait
