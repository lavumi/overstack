#!/usr/bin/env bash
set -euo pipefail

# One-shot helper:
# 1) Build WASM from /core
# 2) Serve /site via python http.server
#
# Default behavior runs the server in background.
# Use --fg to keep server in foreground.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CORE_DIR="$ROOT_DIR/core"
SITE_DIR="$ROOT_DIR/site"
PID_FILE="$ROOT_DIR/site_server.pid"
LOG_FILE="$ROOT_DIR/site_server.log"
PORT="${PORT:-8080}"
MODE="${1:---bg}"

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "Error: wasm-pack is not installed."
  echo "Install with: cargo install wasm-pack"
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "Error: python3 is not installed."
  exit 1
fi

echo "[1/2] Building WASM package..."
(
  cd "$CORE_DIR"
  wasm-pack build --target web --out-dir ../site/pkg
)

