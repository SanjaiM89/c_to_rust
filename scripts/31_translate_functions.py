#!/usr/bin/env python3
"""
31_translate_functions.py - Phase 3, function granularity.

The unit-granularity run (30_translate.py) measured 0/26 compile-first-try:
a 4B model asked for six functions at once makes several *local* mistakes and
cannot recover from them within five attempts.

This driver lowers the generation granularity to ONE function at a time, with
a compile-feedback repair loop per function, and then assembles the functions
back into the unit. The schedule is unchanged - the units are still the
scheduling and context-bundle unit - only the size of a single generation is
reduced.

The unit that a function belongs to still supplies:
  * the contract (types, constants, helpers)
  * the target signature
  * the ALREADY TRANSLATED Rust code of the functions it calls
  * the C source of that one function
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import contract as contract_mod  # noqa: E402
from llm_client import LLMConfig, ask  # noqa: E402
import importlib.util  # noqa: E402

_spec = importlib.util.spec_from_file_location(
    "t30", os.path.join(ROOT := os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                        "scripts", "30_translate.py"))
t30 = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(t30)

A = os.path.join(ROOT, "artifacts", "analysis")


def read_json(p):
    with open(p) as fh:
        return json.load(fh)


def contract_for_unit(csrc: str, fns: list[str], ctext: dict) -> str:
    need = ["consts", "libc_helpers"]
    if any("parse_buffer" in ctext[f]["params"][i]["type"]
           for f in fns for i in range(len(ctext[f]["params"]))) or "parse_buffer" in csrc:
        need.append("parsebuf")
    if "printbuffer" in csrc or "PrintBuffer" in csrc:
        need.append("printbuf")
    if "internal_hooks" in csrc or "Hooks" in csrc:
        need += ["hooks", "init_hooks", "try_alloc"]
    if "global_error" in csrc or "GetErrorPtr" in fns:
        need.append("error_api")
    return contract_mod.contract_for(need)


def callee_context(callees: list[str], fn_code: dict[str, str],
                   sigs: dict) -> str:
    """The already translated Rust of the callees (code when short, else the signature)."""
    out = []
    for c in callees:
        code = fn_code.get(c, "")
        if code and code.count("\n") <= 30:
            out.append(code.rstrip())
        else:
            out.append(sigs[c]["rust_signature"] + ";")
    return "\n\n".join(out)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--port", default="8080")
    ap.add_argument("--max-attempts", type=int, default=5)
    ap.add_argument("--only", default="")
    ap.add_argument("--resume", action="store_true")
    args = ap.parse_args()

    sched = read_json(os.path.join(A, "schedule.json"))
    sigs = read_json(os.path.join(A, "target_signatures.json"))
    cg = read_json(os.path.join(A, "callgraph.json"))
    ctext = cg["functions"]
    edges = cg["edges"]

    units = sched["units"]
    if args.only:
        want = set(args.only.split(","))
        units = [u for u in units if u["unit"] in want]

    out_dir = os.path.join(ROOT, args.out)
    cand_dir = os.path.join(out_dir, "candidates")
    fn_dir = os.path.join(out_dir, "functions")
    os.makedirs(cand_dir, exist_ok=True)
    os.makedirs(fn_dir, exist_ok=True)

    cfg = LLMConfig(backend="local", model=args.model,
                    url=f"http://127.0.0.1:{args.port}/v1/chat/completions",
                    temperature=0.0, max_tokens=2048)

    fn_code: dict[str, str] = {}
    results = []
    unit_order: list[str] = []
    unit_code: dict[str, str] = {}
    scratch = os.path.join(out_dir, "scratch")

    for u in units:
        name = u["unit"]
        fns = u["functions"]
        print(f"\n=== {name} ({len(fns)} fns) ===", flush=True)
        unit_order.append(name)

        for fn in fns:
            sig = sigs[fn]["rust_signature"]
            csrc = t30.extract_function_text(fn) if hasattr(t30, "extract_function_text") else ""
            if not csrc:
                # reuse the analyzer's extractor
                import importlib.util as _iu
                _s = _iu.spec_from_file_location("an", os.path.join(ROOT, "scripts", "20_analyze.py"))
                an = _iu.module_from_spec(_s)
                _s.loader.exec_module(an)
                csrc = an.extract_function_text(fn)

            # callees: everything this function calls that is not itself
            callees = sorted({c for c in edges.get(fn, []) if c != fn})
            # only callees that have already been translated (leaf-first order)
            known = [c for c in callees if c in fn_code]
            ccallee = callee_context(known, fn_code, sigs)

            ctext_contract = contract_for_unit(csrc, [fn], ctext)

            fn_path = os.path.join(fn_dir, f"{fn}.rs")
            prior = ""
            if args.resume and os.path.isfile(fn_path):
                prior = open(fn_path, encoding="utf-8", errors="replace").read()

            # only resume a candidate that actually compiles
            if prior.strip():
                t30.write_scratch(scratch, prior, sigs,
                                  [c for c in callees if c not in fn_code])
                _ok, _ = t30.cargo_check(scratch)
                if _ok:
                    code = prior
                    att = 0
                    ok = True
                    first_try = None
                else:
                    prior = ""   # broken candidate: translate it again

            if not prior.strip():
                code = None
            if not prior.strip():
                prompt = t30.build_function_prompt(sig, ccallee, csrc, ctext_contract)
                t0 = time.time()
                r = ask(prompt, system=t30.SYSTEM, cfg=cfg, tag=f"fn:{fn}")
                code = t30.extract_rust(r.text)
                if code:
                    code, _ = t30.normalise_pub(code)
                att = 1
                first_try = None
                ok = False
                if code:
                    t30.write_scratch(scratch, code, sigs, [c for c in callees if c not in fn_code])
                    ok, log = t30.cargo_check(scratch)
                    first_try = ok
                while code and not ok and att < args.max_attempts:
                    rp = t30.build_func_repair_prompt(code, log, sig, "unit_under_test.rs")
                    r2 = ask(rp, system=t30.SYSTEM, cfg=cfg, tag=f"fnrep:{fn}")
                    new_code = t30.extract_rust(r2.text)
                    if not new_code:
                        break
                    new_code, _ = t30.normalise_pub(new_code)
                    att += 1
                    t30.write_scratch(scratch, new_code, sigs,
                                      [c for c in callees if c not in fn_code])
                    ok, log = t30.cargo_check(scratch)
                    code = new_code
                if code:
                    with open(fn_path, "w") as fh:
                        fh.write(code + "\n")

            fn_code[fn] = code or ""
            results.append({
                "unit": name, "function": fn, "model": args.model,
                "attempts": att, "compiles_first_try": first_try,
                "compiles": ok, "rust_signature": sig,
            })
            print(f"  {fn:<34} attempts={att} first_try={first_try} compiles={ok}", flush=True)

        # assemble the unit from its functions
        unit_src = "\n\n".join(fn_code.get(f, "") for f in fns if fn_code.get(f))
        unit_code[name] = unit_src
        with open(os.path.join(cand_dir, f"{name}.rs"), "w") as fh:
            fh.write(f"// model: {args.model} (function granularity)\n")
            fh.write(unit_src + "\n")

    # final assembly
    t30.write_crate(out_dir, unit_order, unit_code)
    ok, log = t30.cargo_check(out_dir)

    fresh = [r for r in results if r["attempts"] > 0]
    n_first = sum(1 for r in fresh if r["compiles_first_try"])
    n_ok = sum(1 for r in fresh if r["compiles"])
    summary = {
        "model": args.model,
        "granularity": "function",
        "out": args.out,
        "functions": len(results),
        "functions_attempted": len(fresh),
        "compile_first_try": f"{n_first}/{len(fresh)}",
        "compiles_after_repair": f"{n_ok}/{len(fresh)}",
        "total_model_calls": sum(r["attempts"] for r in results),
        "final_crate_compiles": ok,
        "results": results,
    }
    with open(os.path.join(out_dir, "translation_results_functions.json"), "w") as fh:
        json.dump(summary, fh, indent=2)
    print(f"\n=== {args.model} (function granularity): compile-first-try "
          f"{n_first}/{len(fresh)}; after repair {n_ok}/{len(fresh)}; "
          f"model calls {summary['total_model_calls']}; "
          f"final crate compiles: {ok} ===")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
