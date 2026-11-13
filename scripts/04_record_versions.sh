#!/usr/bin/env bash
# Record every toolchain version used by this project -> artifacts/toolchain_versions.txt
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
OUT="$ROOT/artifacts/toolchain_versions.txt"

{
  echo "# toolchain versions - $(date -Is)"
  echo "# host: $(uname -srm)  distro: $(. /etc/os-release 2>/dev/null && echo "$PRETTY_NAME")"
  echo
  echo "[system]"
  echo "gcc          : $(gcc --version 2>/dev/null | head -1)"
  echo "clang        : $(clang --version 2>/dev/null | head -1)"
  echo "llvm-config  : $(llvm-config --version 2>/dev/null)"
  echo "cmake        : $(cmake --version 2>/dev/null | head -1)"
  echo "ninja        : $(ninja --version 2>/dev/null)"
  echo "python3      : $(python3 --version 2>&1)"
  echo "git          : $(git --version 2>/dev/null)"
  echo
  echo "[rust]"
  echo "rustc        : $(rustc --version 2>/dev/null)"
  echo "cargo        : $(cargo --version 2>/dev/null)"
  echo "rustup       : $(rustup --version 2>/dev/null)"
  echo "toolchains   : $(rustup toolchain list 2>/dev/null | tr '\n' ' ')"
  echo "clippy       : $(cargo clippy --version 2>/dev/null)"
  echo "miri         : $(cargo +nightly miri --version 2>/dev/null | head -1)"
  echo "rustfmt      : $(cargo fmt --version 2>/dev/null)"
  echo
  echo "[cargo-installed tools]"
  echo "c2rust       : $(c2rust --version 2>/dev/null | head -1)"
  echo "cargo-fuzz   : $(cargo fuzz --version 2>/dev/null)"
  echo "kani         : $(cargo kani --version 2>/dev/null | head -1)"
  echo
  echo "[local model]"
  echo "llama-server : $(cat "$ROOT/logs/llama-server.log" 2>/dev/null | grep -o 'b12336-[a-f0-9]*' | head -1)"
  echo "models       :"
  ls -la "$ROOT"/models/*.gguf 2>/dev/null | awk '{printf "                %s  %.2f GiB\n", $NF, $5/1073741824}'
  echo
  echo "[target]"
  echo "cJSON tag    : $(cd "$ROOT/vendor/cJSON" 2>/dev/null && git describe --tags 2>/dev/null)"
  echo "cJSON commit : $(cd "$ROOT/vendor/cJSON" 2>/dev/null && git rev-parse HEAD 2>/dev/null)"
  echo "cJSON.c loc  : $(wc -l < "$ROOT/vendor/cJSON/cJSON.c" 2>/dev/null)"
  echo "cJSON.h loc  : $(wc -l < "$ROOT/vendor/cJSON/cJSON.h" 2>/dev/null)"
  echo
  echo "[gpu]"
  nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv,noheader 2>/dev/null | sed 's/^/                /'
} > "$OUT"

cat "$OUT"
