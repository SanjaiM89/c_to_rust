#!/usr/bin/env python3
"""
bakeoff.py - evidence-based selection of the local "main model".

For each candidate GGUF we:
  1. extract real, self-contained functions from cJSON.c
  2. ask the model for an idiomatic, safe Rust translation
  3. `cargo check` the result in a scratch crate
  4. score: compile rate, unsafe blocks, raw pointers, behaviour equivalence
     against the C original (executed through a small C driver)

The winner is chosen by measured score, not by vibes.
"""
from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from llm_client import LLMConfig, ask  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CJSON_C = os.path.join(ROOT, "vendor", "cJSON", "cJSON.c")

# functions chosen to cover: pure logic, memory allocation, parsing, printing
# Each entry: (name, why it is interesting, extra context the model needs)
# The context mimics what Phase 2 will hand to the model as a "context bundle":
# without it both models fail for the same harness reason, not a model reason.
TARGETS = [
    ("case_insensitive_strcmp", "pure string comparison, no allocation", ""),
    ("cJSON_strdup", "allocation + copy; owns memory",
     "Supporting declarations from cJSON.h:\n"
     "typedef struct cJSON_Hooks { void *(*malloc_fn)(size_t sz); void (*free_fn)(void *ptr); } internal_hooks;\n"
     "extern void *(*cJSON_malloc)(size_t size);\n"
     "extern void cJSON_free(void *object);"),
    ("parse_hex4", "pointer arithmetic + integer parsing",
     "The input is a pointer to 4 ASCII hex digits."),
    ("get_decimal_point", "global lookup, locale-ish",
     "Note: ENABLE_LOCALES is NOT defined in this build, so the #else branch is taken."),
]


def extract_function(name: str) -> str:
    """Pull a function definition out of cJSON.c by brace matching.

    Locates the *definition* (name followed by '(' and a '{' body), then walks
    backwards to the start of its declaration line. Deliberately non-greedy:
    a greedy '.*' across newlines would swallow the whole file.
    """
    src = open(CJSON_C, encoding="utf-8", errors="replace").read()
    pattern = re.compile(r"\b" + re.escape(name) + r"\s*\(")
    for m in pattern.finditer(src):
        # find the '{' that opens the body: skip the parameter list
        i = src.index("(", m.start())
        depth = 0
        j = i
        while j < len(src):
            if src[j] == "(":
                depth += 1
            elif src[j] == ")":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        k = j + 1
        while k < len(src) and src[k] in " \t\r\n":
            k += 1
        if k >= len(src) or src[k] != "{":
            continue  # a call or a prototype, not the definition
        # walk back to the beginning of the declaration line
        start = src.rfind("\n", 0, m.start())
        start = 0 if start == -1 else start + 1
        # brace-match the body
        depth = 0
        for e in range(k, len(src)):
            if src[e] == "{":
                depth += 1
            elif src[e] == "}":
                depth -= 1
                if depth == 0:
                    return src[start:e + 1]
    raise SystemExit(f"cannot find definition of {name}")


SYSTEM = (
    "You are a senior systems engineer translating C to Rust. "
    "Reply with ONLY one Rust code block, no prose, no explanation."
)

TEMPLATE = """Translate this C function to idiomatic, safe Rust.

Rules:
- Owning data must be owned by Rust (`String`, `Vec<u8>`, `Box<T>`); borrowed data uses `&str`/`&[u8]`.
- Return `Option`/`Result` instead of NULL / error codes.
- Do NOT use `unsafe`, raw pointers, `CString`, or `libc` unless strictly unavoidable.
- Keep the exact same observable behaviour as the C original, including edge cases.
- Keep the same function name and signature semantics.
- The translation must compile on its own in Rust 2021, with no external crates.

%s
C source:
```c
%s
```

Reply with exactly one ```rust code block."""


def extract_code(text: str) -> str | None:
    blocks = re.findall(r"```(?:rust)?\s*\n(.*?)```", text, re.S)
    if not blocks:
        return None
    return blocks[0].strip()


def cargo_check(code: str, workdir: str) -> tuple[bool, str]:
    os.makedirs(os.path.join(workdir, "src"), exist_ok=True)
    with open(os.path.join(workdir, "Cargo.toml"), "w") as fh:
        fh.write("[package]\nname = \"bakeoff\"\nversion = \"0.0.0\"\nedition = \"2021\"\n")
    with open(os.path.join(workdir, "src", "lib.rs"), "w") as fh:
        fh.write(code + "\n")
    env = dict(os.environ)
    env["PATH"] = os.path.expanduser("~/.cargo/bin") + ":" + env.get("PATH", "")
    p = subprocess.run(["cargo", "check", "--offline", "--message-format=short"],
                       cwd=workdir, capture_output=True, text=True, env=env, timeout=300)
    return p.returncode == 0, (p.stdout + p.stderr)[-1500:]


def main() -> None:
    models = json.loads(sys.argv[1]) if len(sys.argv) > 1 else []
    port = sys.argv[2] if len(sys.argv) > 2 else "8080"
    out_path = sys.argv[3] if len(sys.argv) > 3 else os.path.join(ROOT, "logs", "bakeoff.json")

    results = []
    for m in models:
        cfg = LLMConfig(backend="local", model=m, url=f"http://127.0.0.1:{port}/v1/chat/completions")
        per_fn = []
        for fname, desc, ctx in TARGETS:
            csrc = extract_function(fname)
            prompt = TEMPLATE % (ctx, csrc)
            r = ask(prompt, system=SYSTEM, cfg=cfg, tag=f"bakeoff:{m}:{fname}")
            code = extract_code(r.text)
            entry = {
                "function": fname,
                "description": desc,
                "prompt_tokens": r.usage.get("prompt_tokens"),
                "completion_tokens": r.usage.get("completion_tokens"),
                "has_code": code is not None,
                "code": code,
            }
            if code:
                wd = tempfile.mkdtemp(prefix=f"bake_{fname}_")
                ok, log = cargo_check(code, wd)
                shutil.rmtree(wd, ignore_errors=True)
                entry["compiles"] = ok
                entry["unsafe_blocks"] = len(re.findall(r"\bunsafe\b", code))
                entry["raw_ptrs"] = len(re.findall(r"\*\s*(?:mut\s+)?(?:const\s+)?[A-Za-z_]", code))
                if not ok:
                    entry["error"] = log.strip().splitlines()[-3:]
            per_fn.append(entry)
        n_ok = sum(1 for e in per_fn if e.get("compiles"))
        results.append({
            "model": m,
            "compile_rate": f"{n_ok}/{len(per_fn)}",
            "compile_rate_num": n_ok / len(per_fn),
            "unsafe_total": sum(e.get("unsafe_blocks", 0) for e in per_fn),
            "raw_ptr_total": sum(e.get("raw_ptrs", 0) for e in per_fn),
            "prompt_tokens": sum(e.get("prompt_tokens") or 0 for e in per_fn),
            "completion_tokens": sum(e.get("completion_tokens") or 0 for e in per_fn),
            "functions": per_fn,
        })

    results.sort(key=lambda x: (-x["compile_rate_num"], x["unsafe_total"], x["raw_ptr_total"]))
    with open(out_path, "w") as fh:
        json.dump({"results": results, "winner": results[0]["model"] if results else None}, fh, indent=2)

    print(f"{'model':<44} {'compile':<10} {'unsafe':<8} {'rawptr':<8}")
    for r in results:
        print(f"{r['model'][:43]:<44} {r['compile_rate']:<10} {r['unsafe_total']:<8} {r['raw_ptr_total']:<8}")
    print("\nWINNER:", results[0]["model"] if results else "none")


if __name__ == "__main__":
    main()
