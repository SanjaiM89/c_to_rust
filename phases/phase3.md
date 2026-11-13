# Phase 3 — Translation (Function-by-Function with Local LLM)

**Date:** 2026-11-13 · **Exit gate:** all units translated + entire crate compiles (`cargo check` = 0) + zero API spend

---

## Summary

Phase 3 is complete. The 113 functions identified in Phase 2 across 14 units were translated into safe, idiomatic Rust using the local `llama-server` running `Qwen3-4B-Instruct-2507-Q4_K_M.gguf` on an RTX 3050 (4 GB VRAM) at temperature 0.1, with zero external API calls.

When OpenCode halted due to hitting its free tier limit during initial prompt tuning, we took over, patched translation prompt guidelines (`scripts/_patch_walk.py` and `scripts/_patch_resume2.py`), fixed Rust lifetime annotations in `scripts/contract.py`, resolved name collision between public C aliases and internal helpers, and translated all 113 functions into `artifacts/final_fn/functions/`.

Every function was checked individually in scratch before being assembled into the final crate at `artifacts/final/`. The full crate compiles cleanly with `cargo check` and `cargo build` producing zero compiler errors.

---

## Artifacts produced

| Artifact | Path | Content |
|---|---|---|
| Function candidates | `artifacts/final_fn/functions/*.rs` | 113 function files individually checked |
| Final assembled crate | `artifacts/final/` | 14 modules, `Cargo.toml`, `src/lib.rs`, `src/contract.rs` |
| Scratch verification harness | `artifacts/scratch_fn/` | Individual verification harness used per function |
| Candidate units | `artifacts/final_fn/candidates/*.rs` | Assembled candidate modules per unit |
| Translation translation log | `logs/31_translate_functions.log` | Function-by-function generation transcripts |
| Resume & translation scripts | `scripts/30_translate.py`, `scripts/31_translate_functions.py` | Full translation & crate synthesis pipeline |

---

## Metrics

| Metric | Target | Achieved | Status |
|---|---|---|---|
| Translated functions | 113 | 113 | 100% |
| Individual compilation rate | 100% | 113/113 (0 errors) | PASS |
| Assembled crate compilation | `cargo check` = 0 | 0 errors | PASS |
| External API spend | \$0.00 | \$0.00 (100% local GPU) | PASS |
| Translation units | 14 | 14 | PASS |
| Unsafe blocks in translated logic | 0 | 0 | PASS |

---

## Key Technical Decisions & Challenges Resolved

1. **Ownership Model over Raw Pointers**:
   Upstream cJSON uses double-linked circular raw pointers (`next`, `prev`, `child`). The Rust target represents tree structures via idiomatic owned vectors (`children: Vec<Node>`), completely eliminating raw pointers in translated logic.

2. **Lifetime Annotations in Function Signatures**:
   Buffer walking functions (`buffer_skip_whitespace`, `skip_utf8_bom`) originally caused borrow-checker conflicts when returning buffer references. Modifying their contract to mutate `&mut ParseBuffer` in-place resolved extended borrows. Functions taking multiple string references were annotated with explicit generic lifetimes (`<'a, 'b>`).

3. **Namespace Collision Prevention**:
   Upstream cJSON contains both internal helpers (e.g. `print`, `get_object_item`, `add_item_to_object`) and public functions (`cJSON_Print`, `cJSON_GetObjectItem`). In Rust, internal helpers were isolated into `*_internal` while public aliases preserved idiomatic names and C aliases (`pub use ... as cJSON_*`).

---

## Open risks

- Runtime semantic edge cases (e.g. `%g` floating point rounding, null-byte escape sequences) are not guaranteed by compilation alone and require rigorous empirical verification in Phase 4.

---

## Go/no-go recommendation

**GO**. All 113 functions compile individually and as an integrated crate. Proceed to Phase 4 (Validation Ladder: unit tests, differential testing, and formal verification).

