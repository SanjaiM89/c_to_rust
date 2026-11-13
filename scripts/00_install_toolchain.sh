#!/usr/bin/env bash
# Phase 0: install system toolchain (clang/llvm for c2rust + cargo-fuzz, etc.)
# sudo password is read from ~/.cache_sudo_pw (mode 600) so it never appears in argv.
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
PWFILE="$HOME/.cache_sudo_pw"

echo "=== [$(date -Is)] apt-get update ==="
sudo -S -p '' apt-get update -qq < "$PWFILE"

echo "=== [$(date -Is)] apt-get install ==="
sudo -S -p '' apt-get install -y --no-install-recommends \
  clang llvm libclang-dev llvm-dev \
  build-essential pkg-config \
  lld lldb \
  python3-pip python3-venv \
  unzip xz-utils zip \
  ca-certificates curl git \
  strace gdb valgrind \
  < "$PWFILE"

echo "=== [$(date -Is)] versions ==="
clang --version | head -2
llvm-config --version
cmake --version | head -1
echo "=== done ==="
