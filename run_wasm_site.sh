#!/usr/bin/env bash
set -euo pipefail

# One-shot helper:
# 1) Build WASM from /core with cargo + wasm-bindgen
# 2) Serve /site via python http.server
#
# Default behavior runs the server in background.
# Use --fg to keep server in foreground.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CORE_DIR="$ROOT_DIR/core"
SITE_DIR="$ROOT_DIR/site"
PKG_DIR="$SITE_DIR/pkg"
PID_FILE="$ROOT_DIR/site_server.pid"
LOG_FILE="$ROOT_DIR/site_server.log"
PORT="${PORT:-8080}"
MODE="${1:---bg}"
TARGET_TRIPLE="wasm32-unknown-unknown"
PROFILE="${PROFILE:-release}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "Error: cargo is not installed."
  exit 1
fi

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "Error: wasm-bindgen CLI is not installed."
  echo "Install with: cargo install wasm-bindgen-cli"
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "Error: python3 is not installed."
  exit 1
fi

if ! rustup target list --installed | grep -qx "$TARGET_TRIPLE"; then
  echo "Error: Rust target '$TARGET_TRIPLE' is not installed."
  echo "Install with: rustup target add $TARGET_TRIPLE"
  exit 1
fi

build_wasm() {
  local wasm_file

  if [[ "$PROFILE" == "release" ]]; then
    echo "[1/2] Building WASM package (release)..."
    (
      cd "$CORE_DIR"
      cargo build --target "$TARGET_TRIPLE" --release
    )
    wasm_file="$CORE_DIR/target/$TARGET_TRIPLE/release/core.wasm"
  else
    echo "[1/2] Building WASM package (debug)..."
    (
      cd "$CORE_DIR"
      cargo build --target "$TARGET_TRIPLE"
    )
    wasm_file="$CORE_DIR/target/$TARGET_TRIPLE/debug/core.wasm"
  fi

  if [[ ! -f "$wasm_file" ]]; then
    echo "Error: expected build artifact not found at $wasm_file"
    exit 1
  fi

  rm -rf "$PKG_DIR"
  mkdir -p "$PKG_DIR"

  echo "[2/2] Generating web bindings..."
  wasm-bindgen \
    --target web \
    --out-dir "$PKG_DIR" \
    "$wasm_file"
}

stop_existing_server() {
  if [[ -f "$PID_FILE" ]]; then
    local existing_pid
    existing_pid="$(cat "$PID_FILE")"
    if kill -0 "$existing_pid" >/dev/null 2>&1; then
      echo "Stopping existing site server (pid $existing_pid)..."
      kill "$existing_pid" >/dev/null 2>&1 || true
      wait "$existing_pid" 2>/dev/null || true
    fi
    rm -f "$PID_FILE"
  fi
}

serve_site() {
  echo "Serving $SITE_DIR at http://127.0.0.1:$PORT"
  if [[ "$MODE" == "--fg" ]]; then
    (
      cd "$SITE_DIR"
      exec python3 -m http.server "$PORT"
    )
  else
    (
      cd "$SITE_DIR"
      nohup python3 -m http.server "$PORT" >"$LOG_FILE" 2>&1 &
      echo $! >"$PID_FILE"
    )
    echo "Background server started. PID: $(cat "$PID_FILE")"
    echo "Log: $LOG_FILE"
  fi
}

build_wasm
stop_existing_server
serve_site
