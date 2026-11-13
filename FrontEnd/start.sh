#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$DIR"

PORT="${PORT:-3000}"

echo "================================================================="
echo "Starting c_to_rust Web FrontEnd"
echo "   Dashboard:     http://localhost:$PORT/"
echo "   Web VS Code:   http://localhost:$PORT/ide.html"
echo "   Telemetry:     http://localhost:$PORT/telemetry.html"
echo "================================================================="

export PORT="$PORT"
python3 server.py

