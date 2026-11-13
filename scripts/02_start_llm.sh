#!/usr/bin/env bash
# Phase 0: start the local llama.cpp server (CUDA backend) and wait until healthy.
# Usage: scripts/02_start_llm.sh <model-gguf> [ctx] [port]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODEL="${1:?usage: 02_start_llm.sh <model.gguf> [ctx] [port] [parallel]}"
CTX="${2:-8192}"
PORT="${3:-8080}"
NPAR="${4:-1}"
BIN="$ROOT/tools/llama-cuda/llama-b12336/llama-server"
LOG="$ROOT/logs/llama-server.log"
PIDFILE="$ROOT/logs/llama-server.pid"

# stop any previous instance
if [ -f "$PIDFILE" ] && kill -0 "$(cat "$PIDFILE")" 2>/dev/null; then
  kill "$(cat "$PIDFILE")" 2>/dev/null || true
  sleep 3
fi

cd "$(dirname "$BIN")"
nohup ./llama-server \
  --model "$MODEL" \
  --host 127.0.0.1 --port "$PORT" \
  -ngl 99 -c "$CTX" -b 2048 -ub 512 -np "$NPAR" \
  -lv 5 \
  --jinja --no-webui \
  --api-key local \
  > "$LOG" 2>&1 &
echo $! > "$PIDFILE"

for i in $(seq 1 120); do
  if curl -sf "http://127.0.0.1:$PORT/health" > /dev/null 2>&1; then
    echo "llama-server ready on port $PORT (pid $(cat $PIDFILE))"
    exit 0
  fi
  sleep 1
done
echo "ERROR: server did not become healthy in 120s" >&2
tail -30 "$LOG" >&2
exit 1
