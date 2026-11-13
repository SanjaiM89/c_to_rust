#!/usr/bin/env python3
"""
metrics.py - safety / size metrics for a Rust crate.

Used for the Phase 1 baseline table and re-used in Phase 6 for the final
comparison, so the two tables are guaranteed to be measured the same way.

Usage: metrics.py <crate-dir> <label> [--json out.json]
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import sys

# raw pointer type occurrences: *const T / *mut T in a type position
RE_PTR = re.compile(r"\*\s*(?:const\s+|mut\s+)+[A-Za-z_][A-Za-z0-9_:<>]*")
RE_PTR_BARE = re.compile(r"\*\s*[A-Za-z_][A-Za-z0-9_]*")
RE_UNSAFE_BLOCK = re.compile(r"\bunsafe\s*\{")
RE_UNSAFE_FN = re.compile(r"\bunsafe\s+(?:extern\s+\"[^\"]*\"\s+)?fn\b")
RE_UNSAFE_IMPL = re.compile(r"\bunsafe\s+impl\b")
RE_UNSAFE_TRAIT = re.compile(r"\bunsafe\s+trait\b")


def rust_files(crate: str) -> list[str]:
    out = []
    for root, dirs, files in os.walk(crate):
        dirs[:] = [d for d in dirs if d not in ("target", ".git", "fuzz")]
        for f in files:
            if f.endswith(".rs"):
                out.append(os.path.join(root, f))
    return sorted(out)


def count_unsafe(src: str) -> dict:
    return {
        "unsafe_blocks": len(RE_UNSAFE_BLOCK.findall(src)),
        "unsafe_fns": len(RE_UNSAFE_FN.findall(src)),
        "unsafe_impls": len(RE_UNSAFE_IMPL.findall(src)),
        "unsafe_traits": len(RE_UNSAFE_TRAIT.findall(src)),
    }


def count_pointers(src: str) -> dict:
    """Count raw-pointer *types* (not dereference expressions).

    `*const T` / `*mut T` are the two forms Rust can express, so counting those
    is exact. A bare `*T` is not valid Rust in type position, so any remaining
    `*` is a dereference and is counted separately for transparency.
    """
    const = len(re.findall(r"\*\s*const\s+", src))
    mut = len(re.findall(r"\*\s*mut\s+", src))
    # raw pointers used as struct fields (the ones that actually drive ownership)
    fields = len(re.findall(r"^\s*(?:pub\s+)?[A-Za-z_][A-Za-z0-9_]*\s*:\s*\*", src, re.M))
    derefs = len(re.findall(r"\(\s*\*", src)) + len(re.findall(r"\*\s*[A-Za-z_][A-Za-z0-9_]*\s*\.\s*", src))
    return {"ptr_const": const, "ptr_mut": mut, "ptr_fields": fields, "deref_sites": derefs}


def clippy(crate: str, toolchain: str | None = None) -> dict:
    env = dict(os.environ)
    env["PATH"] = os.path.expanduser("~/.cargo/bin") + ":" + env.get("PATH", "")
    cmd = ["cargo"] + ([f"+{toolchain}"] if toolchain else []) + ["clippy", "--all-targets", "--message-format=json"]
    try:
        p = subprocess.run(cmd, cwd=crate, capture_output=True, text=True, env=env, timeout=1200)
    except Exception as e:
        return {"error": str(e), "warnings": -1, "errors": -1, "by_code": {}}
    seen: set[tuple] = set()
    warnings = 0
    errors = 0
    codes: dict[str, int] = {}
    for line in p.stdout.splitlines():
        try:
            m = json.loads(line)
        except Exception:
            continue
        if m.get("reason") != "compiler-message":
            continue
        msg = m.get("message", {})
        lvl = msg.get("level")
        if lvl not in ("warning", "error"):
            continue
        code = (msg.get("code") or {}).get("code")
        spans = msg.get("spans") or []
        f = spans[0].get("file_name", "?") if spans else "?"
        ln = spans[0].get("line_start", 0) if spans else 0
        key = (lvl, code, f, ln)
        if key in seen:
            continue          # the same lint is reported once per target; count it once
        seen.add(key)
        if lvl == "warning":
            warnings += 1
            if code:
                codes[code] = codes.get(code, 0) + 1
        else:
            errors += 1
            if code:
                codes["ERROR:" + code] = codes.get("ERROR:" + code, 0) + 1
    res = {"warnings": warnings, "errors": errors, "by_code": codes}
    if p.returncode != 0 and warnings == 0 and errors == 0:
        res["error"] = "clippy produced no diagnostics but exited " + str(p.returncode) + \
                       " - treating as a TOOL FAILURE, not a clean run"
        res["stderr_tail"] = p.stderr[-500:]
    return res


def main() -> None:
    crate = os.path.abspath(sys.argv[1])
    label = sys.argv[2] if len(sys.argv) > 2 else os.path.basename(crate)
    out_json = None
    if "--json" in sys.argv:
        out_json = sys.argv[sys.argv.index("--json") + 1]
    toolchain = os.environ.get("METRICS_TOOLCHAIN") or None

    files = rust_files(crate)
    total_lines = 0
    code_lines = 0
    agg_u = {"unsafe_blocks": 0, "unsafe_fns": 0, "unsafe_impls": 0, "unsafe_traits": 0}
    agg_p = {"ptr_const": 0, "ptr_mut": 0, "ptr_fields": 0, "deref_sites": 0}
    per_file = {}

    for f in files:
        src = open(f, encoding="utf-8", errors="replace").read()
        lines = src.splitlines()
        n = len(lines)
        total_lines += n
        # non-blank, non-comment lines
        c = 0
        in_block = False
        for ln in lines:
            s = ln.strip()
            if in_block:
                if "*/" in s:
                    in_block = False
                continue
            if s.startswith("/*"):
                if "*/" not in s:
                    in_block = True
                continue
            if not s or s.startswith("//"):
                continue
            c += 1
        code_lines += c
        u = count_unsafe(src)
        p = count_pointers(src)
        for k in agg_u:
            agg_u[k] += u[k]
        for k in agg_p:
            agg_p[k] += p[k]
        per_file[os.path.relpath(f, crate)] = {
            "lines": n, "code_lines": c, **u, **p,
        }

    total_unsafe = agg_u["unsafe_blocks"] + agg_u["unsafe_fns"] + agg_u["unsafe_impls"] + agg_u["unsafe_traits"]
    total_ptr = agg_p["ptr_const"] + agg_p["ptr_mut"]

    res = {
        "label": label,
        "crate": crate,
        "files": len(files),
        "total_lines": total_lines,
        "code_lines": code_lines,
        "unsafe_total": total_unsafe,
        "raw_pointers_total": total_ptr,
        "unsafe": agg_u,
        "pointers": agg_p,
        "clippy": clippy(crate, toolchain),
        "per_file": per_file,
    }

    print(f"crate                : {crate}")
    print(f"rust files           : {res['files']}")
    print(f"total lines          : {res['total_lines']}")
    print(f"code lines (no cmt)  : {res['code_lines']}")
    print(f"unsafe blocks        : {agg_u['unsafe_blocks']}")
    print(f"unsafe fns           : {agg_u['unsafe_fns']}")
    print(f"unsafe impls/traits  : {agg_u['unsafe_impls'] + agg_u['unsafe_traits']}")
    print(f"unsafe TOTAL         : {total_unsafe}")
    print(f"raw pointers const   : {agg_p['ptr_const']}")
    print(f"raw pointers mut     : {agg_p['ptr_mut']}")
    print(f"raw ptr struct fields: {agg_p['ptr_fields']}")
    print(f"deref sites (info)   : {agg_p['deref_sites']}")
    print(f"raw pointers TOTAL   : {total_ptr}")
    c = res["clippy"]
    print(f"clippy warnings      : {c.get('warnings')}  errors: {c.get('errors')}")
    if c.get("by_code"):
        print("clippy by code       : " + ", ".join(f"{k}={v}" for k, v in sorted(c["by_code"].items(), key=lambda x: -x[1])))

    if out_json:
        os.makedirs(os.path.dirname(out_json), exist_ok=True)
        with open(out_json, "w") as fh:
            json.dump(res, fh, indent=2)
        print(f"\nwritten: {out_json}")


if __name__ == "__main__":
    main()
