#!/usr/bin/env bash
# Phase 0 exit-gate: local model smoke test.
#   - proves the model answers
#   - proves full GPU offload (all layers in VRAM)
#   - measures prompt-eval (tok/s) and generation (tok/s)
# Usage: scripts/03_smoke_llm.sh [port]
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${1:-8080}"
OUT="$ROOT/logs/smoke_llm.json"
URL="http://127.0.0.1:$PORT/v1/chat/completions"

python3 - "$URL" "$OUT" "$PORT" <<'PY'
import json, sys, time, urllib.request, subprocess

url, out, port = sys.argv[1], sys.argv[2], sys.argv[3]

def vram():
    try:
        r = subprocess.run(["nvidia-smi","--query-gpu=memory.used","--format=csv,noheader,nounits"],
                           capture_output=True, text=True, timeout=20)
        return int(r.stdout.strip().split("\n")[0])
    except Exception:
        return -1

before = vram()
body = {
    "model": "local",
    "temperature": 0.0,
    "max_tokens": 256,
    "messages": [
        {"role":"system","content":"You are a precise C-to-Rust translation assistant. Output only code."},
        {"role":"user","content":
         "Translate this C function to idiomatic safe Rust. Output only the Rust code, no commentary.\n\n"
         "```c\n"
         "static int add(int a, int b) { return a + b; }\n"
         "```"}
    ],
}
t0 = time.time()
req = urllib.request.Request(url, data=json.dumps(body).encode(),
                             headers={"Content-Type":"application/json","Authorization":"Bearer local"})
with urllib.request.urlopen(req, timeout=600) as r:
    resp = json.load(r)
dt = time.time() - t0
after = vram()

u = resp.get("usage", {})
prompt_tok = u.get("prompt_tokens", 0)
gen_tok   = u.get("completion_tokens", 0)
timings = resp.get("timings") or {}

text = resp["choices"][0]["message"]["content"]

rec = {
  "model": resp.get("model"),
  "wall_seconds": round(dt, 3),
  "usage": u,
  "timings": timings,
  "prompt_tok_per_s": round(timings.get("prompt_per_second", prompt_tok/dt if dt else 0), 2),
  "gen_tok_per_s": round(timings.get("predicted_per_second", gen_tok/dt if dt else 0), 2),
  "vram_used_mib_before": before,
  "vram_used_mib_after": after,
  "vram_delta_mib": after - before,
  "reply": text,
}
with open(out, "w") as fh:
    json.dump(rec, fh, indent=2)
print(json.dumps({k:v for k,v in rec.items() if k!="reply"}, indent=2))
print("--- reply ---")
print(text[:600])
PY
