p = "scripts/31_translate_functions.py"
t = open(p).read()

old = '''            fn_path = os.path.join(fn_dir, f"{fn}.rs")
            prior = ""
            if args.resume and os.path.isfile(fn_path):
                prior = open(fn_path, encoding="utf-8", errors="replace").read()

            if prior.strip():
                code = prior
                att = 0
                ok = True
                first_try = None'''

new = '''            fn_path = os.path.join(fn_dir, f"{fn}.rs")
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
                code = None'''

assert old in t
t = t.replace(old, new)

# the original `else:` branch must become an `if code is None:` style flow
old2 = '''            else:
                prompt = t30.build_function_prompt(sig, ccallee, csrc, ctext_contract)'''
new2 = '''            if not prior.strip():
                prompt = t30.build_function_prompt(sig, ccallee, csrc, ctext_contract)'''
assert old2 in t
t = t.replace(old2, new2)

open(p, "w").write(t)
print("resume now only reuses candidates that compile")
