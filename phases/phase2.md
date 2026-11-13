# Phase 2 — Analysis and scheduling

**Date:** 2026-11-13 · **Exit gate:** schedule file (units, order, context bundles) + note on what the analysis could not resolve

---

## Summary

Phase 2 is complete. The call graph, data-dependency graph, macro inventory, struct layouts
and global inventory for cJSON v1.7.19 were extracted and the translation was partitioned into
**14 units covering all 113 functions**, ordered **leaf-first with 0 violations**.

Two **recursive strongly connected components** were detected and are scheduled to be translated
together: `{parse_value, parse_array, parse_object}` and `{print_value, print_array, print_object}` —
exactly the mutual recursion the brief predicted would need joint treatment. All **38 local
preprocessor macros** were expanded or classified up front, including the three
`internal_malloc/free/realloc` macros that are *not* functions but aliases for `malloc/free/realloc`,
and the `can_access_at_index` / `can_read` bounds macros that later become Kani proof targets.

---

## Artifacts produced

| Artifact | Path | Content |
|---|---|---|
| Call graph + data deps | `artifacts/analysis/callgraph.json` | 113 functions, 162 edges, SCCs, per-function global/type usage |
| Macro inventory | `artifacts/analysis/macros.json` | 1448 macros classified; 38 local ones with expansion + Rust strategy |
| Structs & globals | `artifacts/analysis/structs.json` | 6 structs with exact fields and source lines; 2 globals |
| Schedule (machine) | `artifacts/analysis/schedule.json` | units, order, context bundles, pointer roles, violations |
| Schedule (readable) | `artifacts/analysis/schedule.md` | 530-line rendered schedule |
| Unresolved | `artifacts/analysis/unresolved.json` | what the analysis could not determine |
| Analyzer | `scripts/20_analyze.py`, `scripts/21_render_schedule.py` | reproducible |

---

## Metrics

| Metric | Value |
|---|---|
| Functions in `cJSON.c` | **113** (AST-derived, excludes 4 system `endian.h` inlines) |
| Call edges | **162** |
| Cyclic SCCs | **2** |
| Self-recursive functions | **0** |
| Translation units | **14** |
| Leaf-first ordering violations | **0** |
| Structs mapped | **6** (`cJSON`, `cJSON_Hooks`, `internal_hooks`, `parse_buffer`, `printbuffer`, `error`) |
| Globals mapped | **2** (`global_hooks`, `global_error`) |
| Macros seen by the preprocessor | **1448** (38 local, 1250 system, 160 system-constants) |

### Unit schedule (leaf-first)

| # | unit | fns | cyclic | calls earlier | calls later | key macros |
|---|---|---|---|---|---|---|
| 0 | `U01_hooks_alloc` | 6 | – | 0 | 0 | `cJSON_IsReference`, `cJSON_StringIsConst` |
| 1 | `U02_print_buffer` | 2 | – | 0 | 0 | – |
| 2 | `U03_number_parse` | 4 | – | 0 | 0 | `buffer_at_offset`, `can_access_at_index` |
| 3 | `U04_string_parse` | 3 | – | 0 | 0 | `buffer_at_offset`, `can_access_at_index` |
| 4 | `U05_string_print` | 2 | – | 1 | 0 | – |
| 5 | `U06_value_parse` | 9 | **yes** | 4 | 0 | `CJSON_NESTING_LIMIT`, `can_read` |
| 6 | `U07_value_print` | 9 | **yes** | 6 | 0 | `cjson_min`, `isinf`, `isnan` |
| 7 | `U08_accessors` | 22 | – | 0 | 0 | `CJSON_VERSION_*` |
| 8 | `U09_create` | 20 | – | 4 | 0 | `cJSON_IsReference` |
| 9 | `U10_add_item` | 16 | – | 14 | 0 | `cJSON_StringIsConst` |
| 10 | `U11_detach_replace` | 13 | – | 8 | 0 | `cJSON_StringIsConst` |
| 11 | `U12_duplicate` | 2 | – | 3 | 0 | `CJSON_CIRCULAR_LIMIT` |
| 12 | `U13_minify` | 4 | – | 0 | 0 | `static_strlen` |
| 13 | `U14_compare` | 1 | – | 0 | 0 | `cJSON_ArrayForEach` |

### Recursive components (translated together)

```
{parse_value, parse_array, parse_object}      -> U06_value_parse
{print_value, print_array, print_object}      -> U07_value_print
```

These are the two groups the brief flagged as hardest ("recursive printer"). They are kept inside
single units so the translator sees the whole recursion at once, and `cJSON_Delete`'s
self-recursion over `child` is visible inside `U01_hooks_alloc`.

### Macro handling (38 local macros, classified up front)

| strategy | count | examples |
|---|---|---|
| `inline-const` | 18 | `CJSON_NESTING_LIMIT` → `pub const CJSON_NESTING_LIMIT: usize = 1000;`, `cJSON_True` → `1 << 1` |
| `inline-fn` | 9 | `cjson_min` → `a.min(b)`, `internal_malloc` → owned allocation (no raw malloc wrapper) |
| `rust-helper` | 7 | `cJSON_ArrayForEach` → iterator, `can_access_at_index` → checked bounds (Kani target), `cJSON_SetIntValue` → setter method |
| `drop` | 4 | `CJSON_PUBLIC`, `CJSON_CDECL`, `CJSON_STDCALL`, header guard |

Each local macro carries its **actual preprocessor expansion** (obtained by planting a marker
probe through `clang -E`) and a concrete Rust instruction. Example, `can_read`:

```
#define can_read(buffer,size) ((buffer != NULL) && (((buffer)->offset + (size)) <= (buffer)->length))
-> strategy: rust-helper
-> note: becomes `buffer.can_read(n) -> bool` with a checked bounds test (this is a Kani proof target)
```

### Context bundle contents (per unit)

Each unit's bundle in `schedule.json` carries everything a translator needs without re-reading
the whole C file:

1. the unit's function list and their **C signatures**
2. **pointer roles** per parameter — `borrowed`, `borrowed-mutable`, `owning-pointer`,
   `tree-handle`, `owned-transferred`, `owned-consumed`, `output-parameter`, `length`
3. the **return role** — `owning-pointer` vs `borrowed-pointer` vs `raw-pointer`
4. the **struct layouts** it touches (fields + exact types)
5. the **globals** it touches
6. the **callees in earlier units** (whose Rust signatures will be known at translation time)
7. the **macros it must expand**, each with its C expansion and Rust instruction

---

## Method

* Functions, signatures, and the call graph come from **`clang -Xclang -ast-dump=json`**, not from
  regex guessing. The callee of every `CallExpr` is resolved through the AST's `referencedDecl`
  ids, so the 162 edges are exact.
* **Structs and globals are extracted from source text** instead. Reason: clang's JSON dump omits
  `loc.file` for nodes in the main file and points `includedFrom` at the *including* file, which
  made `ldiv_t` from `<stdlib.h>` look like it lived in `cJSON.c:77`. Source extraction is
  unambiguous, and comment stripping preserves byte offsets so line numbers stay exact
  (`cJSON.c:88`, `:156`, `:289`, `:473`; `cJSON.h:103`, `:125` — all verified against the source).
* **SCCs** via Tarjan's algorithm; **leaf-first order** via topological sort of the condensation.
* The partition into 14 semantic areas is **hand-seeded but machine-verified**: every call edge is
  checked to point from a later unit to an earlier unit. The first run produced **8 violations**;
  those were fixed by moving `compare_double` into `U03_number_parse` (it is used by both
  `print_number` and `cJSON_Compare`) and `suffix_object` / `cast_away_const` into `U09_create`
  (used by both the constructors and `U10_add_item`). The verifier now reports **0**.

---

## What the analysis could NOT resolve

Recorded in `artifacts/analysis/unresolved.json`:

1. **Recursive SCCs** — `{parse_value, parse_array, parse_object}` and
   `{print_value, print_array, print_object}` are mutually recursive. They are scheduled as single
   units, but the recursion itself cannot be broken into a leaf-first order.
2. **Pointer roles are heuristics**, derived from parameter syntax plus a table of known cJSON
   functions. The *aliasing* and *ownership* semantics of `cJSON` trees are **not proven** by
   static analysis. In particular the analyser does **not** decide whether
   `cJSON_AddItemToArray(array, item)` transfers ownership of `item`; that is encoded from the
   upstream contract, and it is **verified empirically in Phase 4**, not asserted here.
3. **Indirect calls are not in the call graph.** The hooks mechanism
   (`internal_hooks.allocate/deallocate/reallocate`), `localeconv`, and the `cJSON_Hooks` function
   pointers are called indirectly. The affected functions are
   `add_item_to_object`, `cJSON_InitHooks`, `cJSON_New_Item`, `cJSON_strdup`, `create_reference`,
   `print`. The consequence is that a unit may appear to have no callees while actually
   depending on a caller-supplied allocator — the context bundles therefore always include the
   `internal_hooks` layout.
4. **Three macros look like functions.** `internal_malloc`, `internal_free`, `internal_realloc` are
   `#define`s for `malloc`/`free`/`realloc`, not functions. An earlier analysis run wrongly
   assigned them to a unit; they are now classified as macros with strategy `inline-fn`.
5. **`cJSON_Utils` is not analysed.** It is out of translation scope, but `U14_compare`
   (`cJSON_Compare`) lives in `cJSON.c` in v1.7.19 and is used by `cJSON_Utils`.
6. **Layout offsets are not recorded.** The AST dump omits field offsets, so the bundles give
   field *order and types* but not byte offsets. This is sufficient, because the Rust structs are
   `#[repr(C)]` and Rust computes the identical layout.

---

## Open risks

1. **The 14 units are uneven.** `U08_accessors` has 22 functions and `U09_create` has 20, while
   `U02_print_buffer` has 2. Uneven units make the per-unit compile-rate metric noisy, and the
   two large units will produce long generations. Mitigation: the context bundle is what
   constrains the prompt, and the unit source is passed in full; if a unit proves too large for
   the 4 B model, it can be split further without changing the schedule's leaf-first property
   (a split of a unit preserves the ordering).
2. **The 4 B model has to translate 22 functions correctly in one pass** for `U08_accessors`. This
   is the most likely failure point of Phase 3 and is expected to need the Phase 5 repair loop.
3. **Two units own the recursion.** `U06`/`U07` are the units the brief predicted would be
   hardest. There is no Opus to escalate to, so the escalation path is
   *retry → split the unit → hand-write*. Recorded so the Phase 6 numbers can be interpreted.

---

## Go / no-go

**GO** for Phase 3.

Justification: the exit gate is met — `artifacts/analysis/schedule.json` and `schedule.md` contain
the units, their order and their full context bundles, and `unresolved.json` states plainly what
the analysis could not determine. The ordering is machine-verified (0 violations), the two
recursive components are identified and kept together, and every local macro is expanded or
classified with an explicit Rust instruction, so the translator never has to guess what a macro
means.

The risk carried into Phase 3 is model capability, not analysis quality: the partition and the
bundles are as good as static analysis can make them, and the remaining uncertainty about
ownership semantics is resolved by measurement in Phase 4.
