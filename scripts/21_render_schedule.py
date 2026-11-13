#!/usr/bin/env python3
"""Render artifacts/analysis/schedule.json into a human-readable schedule.md."""
from __future__ import annotations

import json
import os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
A = os.path.join(ROOT, "artifacts", "analysis")

sched = json.load(open(os.path.join(A, "schedule.json")))
structs = json.load(open(os.path.join(A, "structs.json")))["structs"]
unres = json.load(open(os.path.join(A, "unresolved.json")))
cg = json.load(open(os.path.join(A, "callgraph.json")))

L = []
w = L.append

w("# Translation schedule — cJSON v1.7.19 → Rust")
w("")
w(f"Units: **{sched['unit_count']}** · functions: **{len(cg['functions'])}** · "
  f"call edges: **{sum(len(v) for v in cg['edges'].values())}** · "
  f"leaf-first violations: **{len(sched['order_violations'])}**")
w("")
w("Units are ordered **leaf-first**: a unit's callees are always in *earlier* units, "
  "so the already-translated Rust signature of every callee is known when a unit is "
  "translated. Recursive groups (strongly connected components) are translated together "
  "inside a single unit.")
w("")

w("## Structs")
w("")
w("| struct | file:line | fields |")
w("|---|---|---|")
for n, s in structs.items():
    fields = ", ".join(f"`{f['name']}: {f['type']}`" for f in s["fields"])
    w(f"| `{n}` | {s['file']}:{s['line']} | {fields} |")
w("")

w("## Globals")
w("")
w("| global | type | file:line |")
w("|---|---|---|")
for g in structs and json.load(open(os.path.join(A, "structs.json")))["globals"]:
    w(f"| `{g['name']}` | `{g['type']}` | {g['file']}:{g['line']} |")
w("")

w("## Unit order")
w("")
w("| # | unit | fns | cyclic | calls earlier | calls later |")
w("|---|---|---|---|---|---|")
for u in sched["units"]:
    w(f"| {u['index']} | `{u['unit']}` | {u['function_count']} | "
      f"{'yes' if u['in_cyclic_scc'] else '-'} | {len(u['calls_earlier_units'])} | "
      f"{len(u['calls_later_units'])} |")
w("")

for u in sched["units"]:
    w(f"## {u['index']}. `{u['unit']}` — {u['description']}")
    w("")
    w(f"Functions ({u['function_count']}): "
      + ", ".join(f"`{f}`" for f in u["functions"]))
    w("")
    if u["in_cyclic_scc"]:
        w("> **Mutually recursive — must be translated as one unit.** "
          f"SCC: {', '.join('`' + f + '`' for f in u['functions'] if f in ('parse_value','parse_array','parse_object','print_value','print_array','print_object'))}")
        w("")
    w("### Signatures and pointer roles")
    w("")
    w("| function | C signature | returns | parameters (role) |")
    w("|---|---|---|---|")
    for s in u["signatures"]:
        ps = ", ".join(f"`{p['name']}: {p['c_type']}` **{p['role']}**" for p in s["params"]) or "—"
        w(f"| `{s['name']}` | `{s['c_signature']}` | {s['return_role']} | {ps} |")
    w("")
    if u.get("macros_used"):
        w("### Macros this unit must expand")
        w("")
        w("| macro | strategy | C expansion | Rust guidance |")
        w("|---|---|---|---|")
        for m in u["macro_guidance"]:
            exp = (m["expansion"] or "(function-like)").replace("|", "\\|")
            note = m["rust_note"].replace("|", "\\|")
            w(f"| `{m['name']}` | {m['strategy']} | `{exp}` | {note} |")
        w("")
    w(f"**Calls in earlier units:** "
      + (", ".join(f"`{c}`" for c in u["calls_earlier_units"]) or "none"))
    w("")
    w(f"**Calls in later units:** "
      + (", ".join(f"`{c}`" for c in u["calls_later_units"]) or "none"))
    w("")

w("## What the analysis could NOT resolve")
w("")
for u in unres:
    w(f"### {u['kind']}")
    w("")
    w(u["detail"])
    w("")
    if u.get("items"):
        for it in u["items"]:
            if isinstance(it, dict):
                w(f"- `{it.get('caller','?')}` → `{it.get('callee','?')}` "
                  f"({it.get('caller_area','?')} → {it.get('callee_area','?')})")
            else:
                w(f"- {' / '.join(it) if isinstance(it, list) else it}")
        w("")

out = os.path.join(A, "schedule.md")
with open(out, "w") as fh:
    fh.write("\n".join(L))
print(f"written: {out}  ({len(L)} lines)")
