p = "scripts/20_analyze.py"
s = open(p).read()

# 1. move the "unassigned" block to AFTER the splitting block
unassigned_block = '''    # ---- partition into areas; verify against the call graph ----
    unassigned = sorted(set(funcs) - set(area_of))
    # anything unassigned goes to the area of its first caller, else a catch-all
    if unassigned:
        fallback = AREAS[-1][0]
        for f in unassigned:
            area_of[f] = fallback

'''
assert unassigned_block in s
s = s.replace(unassigned_block, "    # ---- partition into areas; verify against the call graph ----\n")

anchor = '''    order = [a[0] for a in AREAS]
    pos = {a: i for i, a in enumerate(order)}'''
assert anchor in s
s = s.replace(anchor, '''    unassigned = sorted(set(funcs) - set(area_of))
    # anything unassigned goes to the last area as a catch-all
    if unassigned:
        fallback = AREAS[-1][0]
        for f in unassigned:
            area_of[f] = fallback

''' + anchor)

# 2. main() must mutate the module-level AREAS so the helpers see the split
old_main = '''def main() -> None:
    os.makedirs(OUT_DIR, exist_ok=True)'''
new_main = '''def main() -> None:
    global AREAS
    os.makedirs(OUT_DIR, exist_ok=True)'''
assert old_main in s
s = s.replace(old_main, new_main)

open(p, "w").write(s)
print("fixed ordering and AREAS scoping")
