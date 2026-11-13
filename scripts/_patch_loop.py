import re

p = "scripts/30_translate.py"
t = open(p).read()

old = '''        if code:
            write_crate(out_dir, translated_so_far + [name],
                        {**code_by_unit, name: code})
            ok, log = cargo_check(out_dir)
            errs = error_lines(log)
            first_try_ok = ok
            first_try_errors = errs[:20]
            best_err = len(errs)

            while not ok and attempts < args.max_attempts:
                rp = build_repair_prompt(code, log, signatures, mod_file)
                t1 = time.time()
                r2 = ask(rp, system=SYSTEM, cfg=cfg, tag=f"repair:{name}")
                d1 = time.time() - t1
                new_code = extract_rust(r2.text)
                if not new_code:
                    history.append({"attempt": attempts + 1, "error": "no code block in reply",
                                    "wall_seconds": round(d1, 1)})
                    break
                new_code, _ = normalise_pub(new_code)
                attempts += 1
                history.append({"attempt": attempts, "prompt_sha256": r2.prompt_sha256,
                                "prompt_tokens": r2.usage.get("prompt_tokens"),
                                "completion_tokens": r2.usage.get("completion_tokens"),
                                "wall_seconds": round(d1, 1)})
                write_crate(out_dir, translated_so_far + [name],
                            {**code_by_unit, name: new_code})
                ok2, log2 = cargo_check(out_dir)
                errs2 = error_lines(log2)
                if len(errs2) < best_err:
                    best_err = len(errs2)
                    best_code = new_code
                code = new_code
                log = log2
                ok = ok2
                if ok:
                    break

            # keep the best candidate we saw
            if best_code and best_err < len(error_lines(log)):
                code = best_code
                write_crate(out_dir, translated_so_far + [name],
                            {**code_by_unit, name: code})
                ok, log = cargo_check(out_dir)
'''

new = '''        # The unit is checked in ISOLATION (contract + stubs for its external
        # callees). Otherwise a failure in an earlier unit would make every
        # later unit look broken.
        scratch = os.path.join(out_dir, "scratch")
        ext_callees = sorted({c for c in callee_names if c not in fns})

        def check_unit(candidate: str) -> tuple[bool, str]:
            write_scratch(scratch, candidate, sigs, ext_callees)
            return cargo_check(scratch)

        if code:
            ok, log = check_unit(code)
            errs = error_lines(log)
            first_try_ok = ok
            first_try_errors = errs[:20]
            best_err = len(errs)

            while not ok and attempts < args.max_attempts:
                rp = build_repair_prompt(code, log, signatures, mod_file)
                t1 = time.time()
                r2 = ask(rp, system=SYSTEM, cfg=cfg, tag=f"repair:{name}")
                d1 = time.time() - t1
                new_code = extract_rust(r2.text)
                if not new_code:
                    history.append({"attempt": attempts + 1, "error": "no code block in reply",
                                    "wall_seconds": round(d1, 1)})
                    break
                new_code, _ = normalise_pub(new_code)
                attempts += 1
                history.append({"attempt": attempts, "prompt_sha256": r2.prompt_sha256,
                                "prompt_tokens": r2.usage.get("prompt_tokens"),
                                "completion_tokens": r2.usage.get("completion_tokens"),
                                "wall_seconds": round(d1, 1)})
                ok2, log2 = check_unit(new_code)
                errs2 = error_lines(log2)
                if len(errs2) < best_err:
                    best_err = len(errs2)
                    best_code = new_code
                code = new_code
                log = log2
                ok = ok2
                if ok:
                    break

            # keep the best candidate we saw
            if best_code and best_err < len(error_lines(log)):
                code = best_code
                ok, log = check_unit(code)
'''
assert old in t, "pattern not found"
t = t.replace(old, new)

# the module file name in the scratch crate is unit_under_test.rs
t = t.replace('mod_file = "u" + re.sub(r"[^0-9a-zA-Z]+", "_", name).strip("_").lower() + ".rs"',
              'mod_file = "unit_under_test.rs"')

# the final assembly check must also be reported separately
t = t.replace('''        # ---- compile-first-try check: assemble everything up to this unit ----
        code_by_unit[name] = code or ""
        write_crate(out_dir, translated_so_far + [name], code_by_unit)''',
              '''        code_by_unit[name] = code or ""''')

open(p, "w").write(t)
print("main loop rewired to isolated unit checks")
