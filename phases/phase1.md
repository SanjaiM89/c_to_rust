# Phase 1 — Baseline (c2rust, no LLM)

**Date:** 2026-11-13 · **Exit gate:** baseline compiles + passes the same tests · baseline metrics table exists

---

## Summary

Phase 1 is complete. `c2rust 0.22.1` translated `cJSON.c` into a **3265-line** Rust crate that
compiles and — the key result — passes the **unmodified upstream C test suite**: all **21/21**
upstream Unity test binaries, compiled from the original C sources and linked against the
Rust staticlib, pass. The baseline carries **136 `unsafe` items**, **599 raw pointers** and
**309 clippy warnings**, which is the "faithful but maximally unsafe" reference point every
later number is measured against.

The strongest evidence in this phase is that the tests were **not ported**. c2rust emits
`#[no_mangle] pub unsafe extern "C"` for the public API, so upstream's own test sources link
directly against the Rust library and are executed unmodified.

---

## Artifacts produced

| Artifact | Path |
|---|---|
| c2rust baseline crate | `artifacts/baseline/c2rust/` (`lib.rs`, `src/cJSON.rs`, `Cargo.toml`) |
| Baseline metrics | `artifacts/baseline/metrics_c2rust.json` |
| C-test link harness | `scripts/11_link_ctests.sh` |
| C-test result | `artifacts/baseline/ctest_results.json` |
| Per-test run logs | `logs/ctest_run_*.log` (21 files) |
| Transpile log | `logs/10_c2rust.log` |

---

## Metrics — baseline table

Measured by `scripts/metrics.py` (re-used verbatim in Phase 6 so both tables are comparable).

| Metric | c2rust baseline |
|---|---|
| Rust files | 3 |
| Total lines | 3265 |
| Code lines (no comments/blanks) | 3259 |
| **`unsafe` blocks** | 1 |
| **`unsafe fn`s** | 135 |
| **`unsafe impl`/`trait`** | 0 |
| **`unsafe` TOTAL** | **136** |
| Raw pointers `*const T` | 173 |
| Raw pointers `*mut T` | 426 |
| **Raw pointers TOTAL** | **599** |
| …of which are struct fields | 74 |
| Dereference sites (info only) | 627 |
| **clippy warnings** | **309** |
| **clippy errors** | **2** |

Top clippy lints on the baseline:

| lint | count | meaning |
|---|---|---|
| `clippy::needless_return` | 114 | C-style `return x;` everywhere |
| `clippy::missing_safety_doc` | 79 | `unsafe fn` without a `# Safety` section |
| `clippy::ptr_offset_with_cast` | 69 | `ptr as usize + n` pointer arithmetic — **the class of bug that causes UB** |
| `clippy::unnecessary_cast` | 24 | redundant casts from the C translation |
| `clippy::nonminimal_bool` | 14 | `x == 0` / `x != 0` instead of boolean ops |

**Test result: 21/21 upstream C test binaries pass.**

```
=== RESULT: 21/21 upstream C test binaries pass against the Rust baseline ===
  [PASS] parse_examples   [PASS] parse_number   [PASS] parse_hex4    [PASS] parse_string
  [PASS] parse_array      [PASS] parse_object   [PASS] parse_value   [PASS] print_string
  [PASS] print_number     [PASS] print_array    [PASS] print_object  [PASS] print_value
  [PASS] misc_tests       [PASS] parse_with_opts [PASS] compare_tests [PASS] cjson_add
  [PASS] readme_examples  [PASS] minify_tests
  [PASS] json_patch_tests [PASS] old_utils_tests [PASS] misc_utils_tests
```

### How the C tests were run against Rust

`scripts/11_link_ctests.sh`:

1. `cargo +nightly-2023-04-15 build --release` → `libc2rust.a` (c2rust pins this toolchain).
2. `unity.c` and `cJSON_Utils.c` are compiled **from C** with gcc.
3. Each upstream test `.c` is compiled **unmodified** from `vendor/cJSON/tests/` and linked:
   `gcc <test>.o unity.o [cJSON_Utils.o] libc2rust.a -lm`.

`cJSON_Utils.c` is linked as C on purpose: it *calls* `cJSON_*` symbols, which resolve to the
**Rust** implementation. So the utils-layer tests exercise Rust code through a C caller —
a genuine cross-language check. `cJSON_Utils` itself stays **outside** the translation scope;
it is test scaffolding only.

---

## Notes and caveats

1. **Toolchain skew.** The baseline must build on `nightly-2023-04-15` (c2rust's pinned
   toolchain) because it emits `#![feature(raw_ref_op)]`, which is `#![feature]`-gated and
   therefore rejected by stable. The final idiomatic crate will target stable 1.99.1.
   Consequently the baseline's clippy is **clippy 1.70-nightly** and the final crate's will be
   **clippy 1.99.1**. The lint *set* changed between them, so the warning counts are **not
   strictly apples-to-apples** — the per-lint breakdown is included so the comparison can be
   judged. This is recorded rather than hidden.
2. **`if_same_then_else` is an error (2×).** `src/cJSON.rs:705` — the C original has an `if`
   with two identical branches. This is a *faithful* translation of the C, so it is left in
   place for the baseline; the idiomatic crate must resolve it and the resolution must be
   checked against C behaviour (see Phase 4 divergence classification).
3. **`clippy::ptr_offset_with_cast` (69) is the real risk signal**, not the total warning
   count. Those are `(*p).field` and `ptr as usize + n` patterns — exactly the constructs
   that make Miri and Kani worth running. The metric to beat is 69 → 0.
4. **The baseline is not memory-safe.** 136 `unsafe` items and 599 raw pointers with manual
   `malloc`/`free` mean the baseline can exhibit UB that the C original also exhibits. It is
   a faithful *translation* baseline, not a correct-by-construction baseline. Any divergence
   found later must be checked against the C behaviour, not against the Rust baseline's
   behaviour.

---

## Open risks

1. **The baseline and the idiomatic crate are not API-compatible**, so the "link the C tests"
   trick works for the baseline but **cannot** work for the final crate: an idiomatic Rust API
   uses `&str`, `Option`, `Result` and ownership, not `*const c_char` and `NULL`. The Phase 4
   evidence for the final crate therefore has to come from the **differential harness** (C
   driver vs Rust driver on identical bytes) plus a Rust-native test port. The C-test-link
   trick is kept as a *baseline-only* gate. This is a planned, documented limitation — not a
   surprise discovered late.
2. **c2rust's pinned nightly is 3 years old** relative to stable 1.99.1. Anything the final
   crate does with modern Rust APIs has no c2rust counterpart, so the baseline is a floor, not
   a peer.
3. **`cJSON_Utils` is out of translation scope** but 3 of the 21 upstream tests depend on it.
   The Rust-side test suite for the final crate must reimplement the *behaviour* those tests
   assert, or the suite shrinks. Carried into Phase 2 as an open item.

---

## Go / no-go

**GO** for Phase 2.

Justification: the baseline compiles, and — more importantly — it passes the *unmodified*
upstream test suite, which establishes that c2rust's output is behaviourally faithful enough
to serve as the reference point for every later metric. The metrics table required by the
exit gate exists, is machine-readable, and is produced by a script that Phase 6 will re-run
on the final crate unchanged.

The risk carried forward is not about the baseline's correctness; it is that the strongest
baseline evidence technique (linking C tests) does not transfer to the final crate.
