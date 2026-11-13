import re

p = "scripts/30_translate.py"
t = open(p).read()

old = '''    for u in units:
        name = u["unit"]
        fns = u["functions"]
        print(f"\\n=== {name} ({len(fns)} fns) ===", flush=True)
'''
new = '''    for u in units:
        name = u["unit"]
        fns = u["functions"]
        print(f"\\n=== {name} ({len(fns)} fns) ===", flush=True)

        # ---- resume: reuse an existing candidate that already compiles ----
        cand_path = os.path.join(cand_dir, f"{name}.rs")
        if args.resume and os.path.isfile(cand_path):
            prior = open(cand_path, encoding="utf-8", errors="replace").read()
            prior_code = "\\n".join(ln for ln in prior.splitlines()
                                    if not ln.startswith("//"))
            if prior_code.strip():
                ext = sorted({c for c in u["calls_earlier_units"] + u["calls_later_units"]
                              if c in sigs and c not in fns})
                write_scratch(os.path.join(out_dir, "scratch"), prior_code, sigs, ext)
                pok, _ = cargo_check(os.path.join(out_dir, "scratch"))
                if pok:
                    code_by_unit[name] = prior_code
                    translated_so_far.append(name)
                    results.append({
                        "unit": name, "model": args.model,
                        "prompt_sha256": "resumed", "prompt_tokens": 0,
                        "completion_tokens": 0, "wall_seconds": 0,
                        "has_code": True, "pub_normalised": None,
                        "compiles_first_try": None, "compiles_after_repair": True,
                        "attempts": 0, "first_try_errors": [], "history": [],
                        "resumed": True,
                    })
                    print(f"  resumed from a previous candidate that compiles")
                    continue
'''
assert old in t
t = t.replace(old, new)

t = t.replace('''    ap.add_argument("--max-attempts", type=int, default=5,''',
              '''    ap.add_argument("--resume", action="store_true",
                    help="reuse an existing candidate for a unit when it already compiles")
    ap.add_argument("--max-attempts", type=int, default=5,''')

# a resumed unit has no first-try information; exclude it from that metric
t = t.replace('''    n_pass = sum(1 for r in results if r["compiles_first_try"])''',
              '''    fresh = [r for r in results if not r.get("resumed")]
    n_pass = sum(1 for r in fresh if r["compiles_first_try"])''')
t = t.replace('''    n_final = sum(1 for r in results if r["compiles_after_repair"])''',
              '''    n_final = sum(1 for r in fresh if r["compiles_after_repair"])''')
t = t.replace('''        "compile_first_try_rate": f"{n_pass}/{len(results)}",''',
              '''        "compile_first_try_rate": f"{n_pass}/{len(fresh)}",
        "units_attempted": len(fresh),
        "units_resumed": len(results) - len(fresh),''')
t = t.replace('''        "compiles_after_repair_rate": f"{n_final}/{len(results)}",''',
              '''        "compiles_after_repair_rate": f"{n_final}/{len(fresh)}",''')
t = t.replace('''        "total_model_calls": sum(r["attempts"] for r in results),''',
              '''        "total_model_calls": sum(r["attempts"] for r in results),''')
t = t.replace('''    print(f"\\n=== {args.model}: compile-first-try {n_pass}/{len(results)}; "
          f"after repair {n_final}/{len(results)}; "''',
              '''    print(f"\\n=== {args.model}: compile-first-try {n_pass}/{len(fresh)}; "
          f"after repair {n_final}/{len(fresh)}; "''')

open(p, "w").write(t)
print("resume support added")
