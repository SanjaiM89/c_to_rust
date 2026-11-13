#!/usr/bin/env python3
"""
translate_library.py - Automated, one-command C-to-safe-Rust translation pipeline.

Usage:
    python3 scripts/translate_library.py <path_to_c_dir_or_file> [--out <crate_dir>]
"""
import argparse
import os
import re
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "scripts"))
import llm_client

SYSTEM_PROMPT = """You are an expert systems programmer translating C code into safe, idiomatic, compilable Rust (edition 2021).
Rules:
1. Output ONLY compilable Rust code enclosed in a single ```rust ... ``` code block.
2. No raw pointers (*const, *mut). Use Rust slices (&[u8], &str) or owned types (String, Vec<u8>).
3. Zero unsafe code. No `unsafe` blocks.
4. Maintain exact behavioral equivalence with the C implementation.
5. Provide all needed imports and types so the code compiles standalone.
"""

def extract_rust_code(text: str) -> str:
    m = re.findall(r"```(?:rust)?\s*\n(.*?)```", text, re.DOTALL)
    if m:
        return m[-1].strip()
    return text.strip()

def run_cargo_check(crate_dir: str) -> tuple[bool, str]:
    env = dict(os.environ)
    env["PATH"] = os.path.expanduser("~/.cargo/bin") + ":" + env.get("PATH", "")
    p = subprocess.run(
        ["cargo", "check", "--message-format=short"],
        cwd=crate_dir,
        capture_output=True,
        text=True,
        env=env
    )
    if p.returncode == 0:
        return True, ""
    return False, p.stdout + p.stderr

def run_cargo_test(crate_dir: str) -> tuple[bool, str]:
    env = dict(os.environ)
    env["PATH"] = os.path.expanduser("~/.cargo/bin") + ":" + env.get("PATH", "")
    p = subprocess.run(
        ["cargo", "test"],
        cwd=crate_dir,
        capture_output=True,
        text=True,
        env=env
    )
    if p.returncode == 0:
        return True, p.stdout
    return False, p.stdout + p.stderr

def main():
    parser = argparse.ArgumentParser(description="Automated C to Rust translator")
    parser.add_argument("source", help="Directory or .c file of the C library to translate")
    parser.add_argument("--out", help="Target Rust crate directory", default=None)
    parser.add_argument("--max-retries", type=int, default=5, help="Max repair attempts")
    args = parser.parse_args()

    src_path = os.path.abspath(args.source)
    c_files = []
    h_files = []

    if os.path.isdir(src_path):
        for root, _, files in os.walk(src_path):
            if "target" in root or ".git" in root or "tests" in root or "examples" in root or "fuzz" in root:
                continue
            for f in files:
                p = os.path.join(root, f)
                content = open(p, errors="ignore").read()
                if f.endswith(".c"):
                    if "main(" not in content and "LLVMFuzzer" not in content:
                        c_files.append(p)
                elif f.endswith(".h"):
                    h_files.append(p)
    else:
        c_files.append(src_path)

    if not c_files:
        print(f"Error: No library .c files found in {src_path}")
        sys.exit(1)

    lib_name = os.path.basename(src_path).replace("-", "_").replace(" ", "_")
    if lib_name.endswith(".c"):
        lib_name = lib_name[:-2]

    out_dir = args.out
    if not out_dir:
        out_dir = os.path.join(src_path if os.path.isdir(src_path) else os.path.dirname(src_path), f"{lib_name}-rs")

    os.makedirs(os.path.join(out_dir, "src"), exist_ok=True)
    cargo_toml = os.path.join(out_dir, "Cargo.toml")
    crate_pkg_name = os.path.basename(out_dir).replace("-", "_")
    if not os.path.exists(cargo_toml):
        with open(cargo_toml, "w") as f:
            f.write(f"""[package]
name = "{crate_pkg_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
""")

    print(f"[*] Translating C library: {src_path}")
    print(f"    Target crate: {out_dir}")
    print(f"    C source files: {[os.path.basename(f) for f in c_files]}")
    print(f"    Header files: {[os.path.basename(f) for f in h_files]}")

    c_content = ""
    for h in h_files:
        c_content += f"// --- Header: {os.path.basename(h)} ---\n" + open(h, errors="ignore").read() + "\n\n"
    for c in c_files:
        c_content += f"// --- Source: {os.path.basename(c)} ---\n" + open(c, errors="ignore").read() + "\n\n"

    lib_rs = os.path.join(out_dir, "src", "lib.rs")

    prompt = f"""{SYSTEM_PROMPT}

Translate the following C library into safe, idiomatic Rust:

```c
{c_content}
```

Write the complete `src/lib.rs` implementation:
"""
    print("[*] Generating Rust translation via local LLM ...", flush=True)
    t0 = time.time()
    res = llm_client.ask(prompt, cfg=llm_client.LLMConfig(temperature=0.1, max_tokens=4096))
    rust_code = extract_rust_code(res.text)

    with open(lib_rs, "w") as f:
        f.write("// Translated automatically by translate_library.py\n#![allow(unused, dead_code)]\n" + rust_code + "\n")

    print(f"    Generated {len(rust_code.splitlines())} lines of Rust in {time.time()-t0:.1f}s")
    print("[*] Running cargo check ...", flush=True)

    ok, err = run_cargo_check(out_dir)
    attempt = 0
    while not ok and attempt < args.max_retries:
        attempt += 1
        print(f"[!] Compiler error on attempt {attempt}/{args.max_retries}. Repairing ...", flush=True)
        repair_prompt = f"""{SYSTEM_PROMPT}

The previous Rust translation failed `cargo check` with the following error:

```
{err[:2000]}
```

Here is the current Rust code:
```rust
{rust_code}
```

Please fix all compilation errors and provide the updated complete Rust code.
"""
        res = llm_client.ask(repair_prompt, cfg=llm_client.LLMConfig(temperature=0.1, max_tokens=4096))
        rust_code = extract_rust_code(res.text)
        with open(lib_rs, "w") as f:
            f.write("// Translated automatically by translate_library.py\n#![allow(unused, dead_code)]\n" + rust_code + "\n")
        ok, err = run_cargo_check(out_dir)

    if ok:
        print("[+] cargo check PASSED! Clean compilation.")
    else:
        print(f"[-] cargo check failed after {args.max_retries} attempts.")
        print(err[-1000:])
        sys.exit(1)

    print("[*] Generating test suite ...", flush=True)
    tests_dir = os.path.join(out_dir, "tests")
    os.makedirs(tests_dir, exist_ok=True)
    test_rs = os.path.join(tests_dir, "integration_test.rs")

    test_prompt = f"""Write a comprehensive unit test suite in Rust for this crate:
```rust
{rust_code}
```
Use standard `#[test]` functions that verify normal operation and edge cases. Output ONLY a ```rust ... ``` block.
"""
    test_res = llm_client.ask(test_prompt, cfg=llm_client.LLMConfig(temperature=0.1, max_tokens=2048))
    test_code = extract_rust_code(test_res.text)
    with open(test_rs, "w") as f:
        f.write(f"use {crate_pkg_name}::*;\n" + test_code + "\n")

    print("[*] Running cargo test ...", flush=True)
    tok, tout = run_cargo_test(out_dir)
    if tok:
        print("[+] cargo test PASSED!")
        for line in tout.splitlines():
            if "test result:" in line or "running" in line:
                print(f"    {line}")
    else:
        print("[-] cargo test failed:")
        print(tout[-1000:])

    print(f"\n[+] Translation complete! Crate ready at: {out_dir}")

if __name__ == "__main__":
    main()
