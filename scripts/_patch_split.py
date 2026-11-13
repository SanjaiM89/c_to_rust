import re

# ------------------------------------------------------------------ #
# 1. analyzer: split large units into leaf-ordered chunks
# ------------------------------------------------------------------ #
p = "scripts/20_analyze.py"
s = open(p).read()

old = '''    # ---- partition into areas; verify against the call graph ----
    area_of: dict[str, str] = {}'''
new = '''    # ---- partition into areas; verify against the call graph ----
    area_of: dict[str, str] = {}'''
assert old in s

# insert the splitting logic right after AREAS are seeded
old2 = '''    order = [a[0] for a in AREAS]
    pos = {a: i for i, a in enumerate(order)}'''
new2 = '''    # ---- split oversized units -------------------------------------------------
    # A 4B model cannot emit 20 correct functions in one answer. Units larger
    # than MAX_UNIT_FUNCTIONS are split into chunks, ordered so that a
    # function's callees inside the same area always land in an EARLIER chunk.
    # Splitting a unit never breaks the leaf-first property of the schedule.
    MAX_UNIT_FUNCTIONS = 6

    def order_within(area: str, members: list[str]) -> list[str]:
        """Topologically order the members of one area by the internal call graph."""
        member_set = set(members)
        indeg = {m: 0 for m in members}
        adj = {m: [] for m in members}
        for m in members:
            for c in graph.get(m, ()):
                if c in member_set:
                    adj[c].append(m)      # edge c -> m (c is a callee of m)
                    indeg[m] += 1
        ready = sorted([m for m in members if indeg[m] == 0])
        out: list[str] = []
        while ready:
            m = ready.pop(0)
            out.append(m)
            for nxt in adj[m]:
                indeg[nxt] -= 1
                if indeg[nxt] == 0:
                    ready.append(nxt)
                    ready.sort()
        # any member left over is part of a cycle inside the area
        out += sorted(m for m in members if m not in out)
        return out

    new_areas: list[tuple[str, str, list[str]]] = []
    for area, desc, fs in AREAS:
        members = sorted(set(fs) & set(funcs))
        if not members:
            continue
        if len(members) <= MAX_UNIT_FUNCTIONS:
            new_areas.append((area, desc, members))
            continue
        ordered = order_within(area, members)
        chunks = [ordered[i:i + MAX_UNIT_FUNCTIONS]
                  for i in range(0, len(ordered), MAX_UNIT_FUNCTIONS)]
        base = re.sub(r"^U(\\d+)", r"U\\1", area)
        for i, chunk in enumerate(chunks):
            suffix = "" if len(chunks) == 1 else f"_p{i + 1}"
            new_areas.append((base + suffix,
                              desc + (f" (part {i + 1} of {len(chunks)})" if len(chunks) > 1 else ""),
                              chunk))
    AREAS = new_areas

    area_of: dict[str, str] = {}
    dup_seed: dict[str, list[str]] = defaultdict(list)
    for area, _desc, fs in AREAS:
        for f in fs:
            dup_seed[f].append(area)
            area_of.setdefault(f, area)
    seed_conflicts = {f: a for f, a in dup_seed.items() if len(a) > 1 and f in funcs}

    order = [a[0] for a in AREAS]
    pos = {a: i for i, a in enumerate(order)}'''
assert old2 in s
s = s.replace(old2, new2)

# the original seeding block must not run twice
old3 = '''    area_of: dict[str, str] = {}
    dup_seed: dict[str, list[str]] = defaultdict(list)
    for area, _desc, fs in AREAS:
        for f in fs:
            dup_seed[f].append(area)
            area_of.setdefault(f, area)
    seed_conflicts = {f: a for f, a in dup_seed.items() if len(a) > 1 and f in funcs}
    unassigned = sorted(set(funcs) - set(area_of))'''
new3 = '''    unassigned = sorted(set(funcs) - set(area_of))'''
assert old3 in s
s = s.replace(old3, new3)

open(p, "w").write(s)
print("analyzer: units are now split into <=6-function chunks")
