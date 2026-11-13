#!/usr/bin/env bash
# Phase 1 (gate): run the UNMODIFIED upstream C test suite against the Rust implementation.
#
# c2rust emits `#[no_mangle] pub unsafe extern "C"` for the public API, so the
# upstream Unity test binaries can be linked directly against the Rust staticlib.
# This is a much stronger claim than "we ported the tests": the *same* test
# sources that upstream ships are executed against the Rust code.
#
# The three cJSON_Utils-dependent tests link cJSON_Utils.c (compiled from C);
# that object only *calls* cJSON_* symbols, which resolve to the Rust library.
# cJSON_Utils itself remains outside the translation scope.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"

VENDOR="$ROOT/vendor/cJSON"
CRATE="$ROOT/artifacts/baseline/c2rust"
WORK="$ROOT/artifacts/baseline/ctest"
LOGDIR="$ROOT/logs"
RESULT="$ROOT/artifacts/baseline/ctest_results.json"

UNITY_TESTS="parse_examples parse_number parse_hex4 parse_string parse_array parse_object
parse_value print_string print_number print_array print_object print_value misc_tests
parse_with_opts compare_tests cjson_add readme_examples minify_tests"
UTILS_TESTS="json_patch_tests old_utils_tests misc_utils_tests"

mkdir -p "$WORK"
rm -f "$WORK"/*.o "$WORK"/t_* 2>/dev/null

echo "=== 1. build Rust staticlib (c2rust baseline) ==="
cd "$CRATE"
cargo +nightly-2023-04-15 build --release 2>&1 | tail -3
LIBDIR="$CRATE/target/release"
ls -la "$LIBDIR"/libc2rust.a || { echo "FATAL: no staticlib"; exit 1; }

echo "=== 2. compile Unity + cJSON_Utils (C) ==="
cd "$VENDOR"
gcc -c -O1 -w -I tests/unity/src -o "$WORK/unity.o" tests/unity/src/unity.c || exit 1
gcc -c -O1 -w -I . -o "$WORK/cJSON_Utils.o" cJSON_Utils.c || exit 1

echo "=== 3. compile + link each upstream test against the RUST library ==="
cd "$VENDOR/tests"
declare -A NEEDS_UTILS
for t in $UTILS_TESTS; do NEEDS_UTILS[$t]=1; done

PASS=0; FAIL=0; FAILED=""
for t in $UNITY_TESTS $UTILS_TESTS; do
  gcc -c -O1 -w -I . -I unity/src -I .. -o "$WORK/$t.o" "$t.c" 2> "$LOGDIR/ctest_compile_$t.err"
  if [ $? -ne 0 ]; then
    echo "  [COMPILE-FAIL] $t"; FAIL=$((FAIL+1)); FAILED="$FAILED $t"; continue
  fi
  EXTRA=""
  [ -n "${NEEDS_UTILS[$t]:-}" ] && EXTRA="$WORK/cJSON_Utils.o"
  gcc -o "$WORK/t_$t" "$WORK/$t.o" "$WORK/unity.o" $EXTRA "$LIBDIR/libc2rust.a" -lm 2> "$LOGDIR/ctest_link_$t.err"
  if [ $? -ne 0 ]; then
    echo "  [LINK-FAIL] $t"; FAIL=$((FAIL+1)); FAILED="$FAILED $t"; continue
  fi
  "$WORK/t_$t" > "$LOGDIR/ctest_run_$t.log" 2>&1
  rc=$?
  if [ $rc -eq 0 ]; then
    echo "  [PASS] $t"; PASS=$((PASS+1))
  else
    echo "  [FAIL] $t (exit $rc)"; FAIL=$((FAIL+1)); FAILED="$FAILED $t"
  fi
done

TOTAL=$((PASS+FAIL))
echo
echo "=== RESULT: $PASS/$TOTAL upstream C test binaries pass against the Rust baseline ==="
[ -n "$FAILED" ] && echo "failing:$FAILED"

python3 - "$RESULT" "$PASS" "$FAIL" "$TOTAL" "$FAILED" <<'PY'
import json, sys
out, p, f, t, failed = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4]), sys.argv[5].split()
json.dump({"passed": p, "failed": f, "total": t, "failing": failed,
           "note": "upstream Unity test binaries, compiled unmodified from C, linked against the Rust staticlib"},
          open(out, "w"), indent=2)
PY
exit $([ "$FAIL" -eq 0 ] && echo 0 || echo 1)
