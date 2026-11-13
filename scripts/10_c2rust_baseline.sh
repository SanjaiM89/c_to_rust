#!/usr/bin/env bash
# Phase 1: produce the c2rust "faithful" baseline crate from vendor/cJSON/cJSON.c.
#
# c2rust is the deterministic (no-LLM) translation. Its output is the reference
# "faithful pass": mechanically correct, maximally unsafe. Every later metric is
# measured against it.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"

SRC="$ROOT/vendor/cJSON/cJSON.c"
HDR="$ROOT/vendor/cJSON/cJSON.h"
OUT="$ROOT/artifacts/baseline/c2rust"
LOG="$ROOT/logs/10_c2rust.log"

rm -rf "$OUT"; mkdir -p "$OUT"

echo "=== c2rust transpile ==="
cd "$ROOT/vendor/cJSON"
c2rust transpile \
  -o "$OUT" \
  --emit-build-files \
  --emit-modules \
  --overwrite-existing \
  "$SRC" -- -I "$(dirname "$HDR")" -DCJSON_HIDE_SYMBOLS 2>&1 | tee "$LOG" | tail -20

echo "=== emitted files ==="
find "$OUT" -name '*.rs' -o -name 'Cargo.toml' | sed "s|$OUT|artifacts/baseline/c2rust|" | sort
