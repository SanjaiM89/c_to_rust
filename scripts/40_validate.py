#!/usr/bin/env python3
"""
40_validate.py - Phase 4: the evidence ladder.

Level 1  unit tests        - the crate's own test suite (if present)
Level 2  differential      - identical bytes to the C and the Rust, transcripts
                            compared byte for byte
Level 3  miri / kani      - UB detection and bounded proofs

Every divergence is written to artifacts/validation/divergences.json with the
failing input and the first differing line, so it can be classified.
"""
from __future__ import annotations

import argparse
import json
import os
import random
import shutil
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HARNESS = os.path.join(ROOT, "harness")
CORPUS = os.path.join(ROOT, "artifacts", "corpus")
OUT = os.path.join(ROOT, "artifacts", "validation")


def run(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


def build_drivers(crate: str) -> tuple[str | None, str | None]:
    """Build the C driver and the Rust driver. Returns (c_bin, rust_bin)."""
    c_bin = os.path.join(HARNESS, "diff_c")
    r_bin = os.path.join(HARNESS, "diff_rust")
    p = run(["gcc", "-O1", "-w", "-I", os.path.join(ROOT, "vendor", "cJSON"),
             os.path.join(HARNESS, "differential_c.c"),
             os.path.join(ROOT, "vendor", "cJSON", "cJSON.c"), "-o", c_bin])
    if p.returncode != 0:
        print("C driver build failed:\n" + p.stderr[-2000:])
        return None, None

    # the Rust driver is compiled into the crate as an extra binary
    os.makedirs(os.path.join(crate, "src", "bin"), exist_ok=True)
    shutil.copy(os.path.join(HARNESS, "differential_rust.rs"),
                os.path.join(crate, "src", "bin", "differential_rust.rs"))
    env = dict(os.environ)
    env["PATH"] = os.path.expanduser("~/.cargo/bin") + ":" + env.get("PATH", "")
    p = run(["cargo", "build", "--release", "--bin", "differential_rust"],
            cwd=crate, env=env, timeout=1800)
    if p.returncode != 0:
        print("Rust driver build failed:\n" + (p.stdout + p.stderr)[-4000:])
        return c_bin, None
    built = os.path.join(crate, "target", "release", "differential_rust")
    shutil.copy(built, r_bin)
    return c_bin, r_bin


def transcript(binary: str, data: bytes, timeout: int = 20) -> tuple[str | None, str]:
    try:
        p = subprocess.run([binary], input=data, capture_output=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return None, "TIMEOUT"
    if p.returncode != 0:
        return None, f"EXIT {p.returncode}: {p.stderr[:400]}"
    return p.stdout.decode("utf-8", "replace"), ""


def first_diff(a: str, b: str) -> dict:
    la, lb = a.splitlines(), b.splitlines()
    for i in range(max(len(la), len(lb))):
        x = la[i] if i < len(la) else "<missing>"
        y = lb[i] if i < len(lb) else "<missing>"
        if x != y:
            return {"line": i + 1, "c": x[:400], "rust": y[:400]}
    return {}


def differential(c_bin: str, r_bin: str, inputs: list[tuple[str, bytes]],
                 limit: int = 0) -> dict:
    diverged = []
    agreed = 0
    errors = []
    for name, data in inputs:
        if limit and len(diverged) >= limit:
            break
        ct, cerr = transcript(c_bin, data)
        rt, rerr = transcript(r_bin, data)
        if cerr or rerr:
            errors.append({"input": name, "c_error": cerr, "rust_error": rerr})
            continue
        if ct == rt:
            agreed += 1
        else:
            d = {"input": name, "size": len(data), "diff": first_diff(ct, rt)}
            d["c_excerpt"] = ct[:1500]
            d["rust_excerpt"] = rt[:1500]
            diverged.append(d)
    return {"agreed": agreed, "diverged": diverged, "errors": errors}


def mutate(data: bytes, rng: random.Random) -> bytes:
    """A small, deterministic mutation operator set."""
    if not data:
        return bytes(rng.randrange(256) for _ in range(rng.randrange(0, 24)))
    b = bytearray(data)
    for _ in range(rng.randrange(1, 5)):
        op = rng.randrange(6)
        if op == 0 and b:                       # flip a byte
            i = rng.randrange(len(b))
            b[i] ^= 1 << rng.randrange(8)
        elif op == 1 and b:                     # delete a byte
            del b[rng.randrange(len(b))]
        elif op == 2:                           # insert a byte
            b.insert(rng.randrange(len(b) + 1), rng.randrange(256))
        elif op == 3 and b:                     # truncate
            del b[rng.randrange(len(b)):]
        elif op == 4 and b:                     # duplicate a slice
            i = rng.randrange(len(b))
            j = min(len(b), i + rng.randrange(1, 8))
            b[i:i] = b[i:j]
        else:                                   # splice in a JSON token
            tok = rng.choice([b"{", b"}", b"[", b"]", b'"', b"\\", b":", b",",
                              b"null", b"true", b"false", b"0", b"-1", b"1e999",
                              b"\\u0041", b"\\ud800", b"\x00", b"\xff", b"\xc0\xaf"])
            b[rng.randrange(len(b) + 1):rng.randrange(len(b) + 1)] = tok
    return bytes(b)


def fuzz(c_bin: str, r_bin: str, seconds: float, seed: int,
         corpus_dir: str, out_dir: str) -> dict:
    """Deterministic, mutation-based differential fuzzing with a fixed budget."""
    rng = random.Random(seed)
    seeds: list[bytes] = []
    for root, _dirs, files in os.walk(corpus_dir):
        for f in sorted(files):
            p = os.path.join(root, f)
            try:
                with open(p, "rb") as fh:
                    seeds.append(fh.read())
            except OSError:
                pass
    if not seeds:
        seeds = [b"{}", b"[]", b'{"a":1}']

    os.makedirs(out_dir, exist_ok=True)
    t0 = time.time()
    n = 0
    diverged = []
    agreed = 0
    errors = 0
    budget = int(seconds)
    while time.time() - t0 < budget:
        base = seeds[rng.randrange(len(seeds))]
        data = mutate(base, rng)
        n += 1
        ct, cerr = transcript(c_bin, data, timeout=10)
        rt, rerr = transcript(r_bin, data, timeout=10)
        if cerr or rerr:
            errors += 1
            with open(os.path.join(out_dir, f"error_{n:06d}.bin"), "wb") as fh:
                fh.write(data)
            continue
        if ct == rt:
            agreed += 1
            # keep interesting inputs that the C accepted
            if "STATUS ok" in ct and len(data) < 200:
                with open(os.path.join(out_dir, f"ok_{n:06d}.bin"), "wb") as fh:
                    fh.write(data)
        else:
            d = {"n": n, "size": len(data), "diff": first_diff(ct, rt),
                 "c_excerpt": ct[:1200], "rust_excerpt": rt[:1200]}
            diverged.append(d)
            with open(os.path.join(out_dir, f"diverge_{n:06d}.bin"), "wb") as fh:
                fh.write(data)
            if len(diverged) >= 50:
                break
    return {"seconds": round(time.time() - t0, 1), "executions": n,
            "agreed": agreed, "diverged": len(diverged), "errors": errors,
            "divergences": diverged[:50], "corpus_dir": out_dir}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--crate", default="artifacts/final")
    ap.add_argument("--fuzz-seconds", type=float, default=60.0)
    ap.add_argument("--seed", type=int, default=20261113)
    ap.add_argument("--skip-fuzz", action="store_true")
    args = ap.parse_args()

    os.makedirs(OUT, exist_ok=True)
    crate = os.path.join(ROOT, args.crate)

    print("[*] building differential drivers ...", flush=True)
    c_bin, r_bin = build_drivers(crate)
    if not c_bin or not r_bin:
        print("drivers could not be built - differential level skipped")
        with open(os.path.join(OUT, "validation.json"), "w") as fh:
            json.dump({"status": "drivers_failed"}, fh, indent=2)
        return 1

    # ---- level 2a: the seed corpus ----
    inputs = []
    for root, _d, files in os.walk(os.path.join(CORPUS, "seed")):
        for f in sorted(files):
            p = os.path.join(root, f)
            with open(p, "rb") as fh:
                inputs.append((os.path.relpath(p, CORPUS), fh.read()))
    print(f"[*] level 2a: differential over {len(inputs)} seed inputs ...", flush=True)
    res = differential(c_bin, r_bin, inputs)
    print(f"    agreed: {res['agreed']}  diverged: {len(res['diverged'])}  "
          f"errors: {len(res['errors'])}")

    # ---- level 2b: differential fuzzing ----
    fuzz_res = None
    if not args.skip_fuzz:
        print(f"[*] level 2b: differential fuzzing for {args.fuzz_seconds}s "
              f"(seed {args.seed}) ...", flush=True)
        fuzz_res = fuzz(c_bin, r_bin, args.fuzz_seconds, args.seed,
                        os.path.join(CORPUS, "seed"),
                        os.path.join(CORPUS, "fuzz"))
        print(f"    executions: {fuzz_res['executions']}  agreed: {fuzz_res['agreed']}  "
              f"diverged: {fuzz_res['diverged']}  errors: {fuzz_res['errors']}")

    out = {
        "crate": args.crate,
        "level2_seed": {"inputs": len(inputs), "agreed": res["agreed"],
                        "diverged": len(res["diverged"]), "errors": len(res["errors"]),
                        "divergences": res["diverged"]},
        "level2_fuzz": fuzz_res,
    }
    with open(os.path.join(OUT, "divergences.json"), "w") as fh:
        json.dump(out, fh, indent=2)
    print(f"\nwritten: artifacts/validation/divergences.json")
    return 0


if __name__ == "__main__":
    sys.exit(main())
