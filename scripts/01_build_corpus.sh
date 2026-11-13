#!/usr/bin/env bash
# Phase 0: build the fuzz seed corpus.
#   - all JSON-ish files shipped with cJSON (tests/inputs, fuzzing/inputs, json-patch-tests)
#   - hand-written edge cases: deep nesting, huge numbers, invalid UTF-8, unterminated strings
# Output: artifacts/corpus/seed/*.json  (one case per file)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CJSON="$ROOT/vendor/cJSON"
OUT="$ROOT/artifacts/corpus/seed"
rm -rf "$OUT"; mkdir -p "$OUT"

# 1. upstream corpus
n_up=0
# prefix with the source subdir so same-named files from different dirs do not collide
for f in "$CJSON"/tests/inputs/* "$CJSON"/fuzzing/inputs/* "$CJSON"/tests/json-patch-tests/*.json; do
  [ -f "$f" ] || continue
  case "$f" in *.expected) continue;; esac
  rel="${f#$CJSON/}"
  tag="$(echo "${rel%%/*}" | tr '/' '_')"
  cp "$f" "$OUT/upstream_${tag}_$(basename "$f")"
  n_up=$((n_up+1))
done

# 2. hand-written edge cases
python3 - "$OUT" <<'PY'
import os, sys, json
out = sys.argv[1]
cases = {}

def nest(d):
    s = "1"
    for _ in range(d):
        s = "[%s]" % s
    return s

cases["deep_nest_64"]      = nest(64)
cases["deep_nest_256"]     = nest(256)
cases["deep_nest_512"]     = nest(512)
cases["deep_nest_1024"]    = nest(1024)
cases["deep_nest_2000"]    = nest(2000)          # cJSON has a recursion limit (1000)
cases["deep_obj_512"]      = "{" + '"a":{'*512 + '"b":1' + '}'*512 + "}"
cases["deep_obj_1500"]     = "{" + '"a":{'*1500 + '"b":1' + '}'*1500 + "}"

# huge / boundary numbers
cases["num_int_max_i64"]   = "9223372036854775807"
cases["num_int_min_i64"]   = "-9223372036854775808"
cases["num_int_over_i64"]  = "9223372036854775808"
cases["num_int_overflow2"] = "99999999999999999999999999999999999"
cases["num_float_huge"]    = "1e999"
cases["num_float_neg_huge"]= "-1e999"
cases["num_float_tiny"]    = "1e-999"
cases["num_leading_zero"]  = "0123"
cases["num_plus"]          = "+1"
cases["num_dot_lead"]      = ".5"
cases["num_trail_dot"]     = "5."
cases["num_exp_only"]      = "1e"
cases["num_hex"]           = "0x1F"
cases["num_inf"]           = "Infinity"
cases["num_nan"]           = "NaN"
cases["num_zero"]          = "0"
cases["num_neg_zero"]      = "-0"
cases["num_many_digits"]   = "1" + "0"*400

# strings / escapes
cases["str_unterminated"]      = '"abc'
cases["str_unterminated_esc"]  = '"abc\\'
cases["str_bad_escape"]        = '"\\q"'
cases["str_lone_surrogate"]    = '"\\ud800"'
cases["str_surrogate_pair"]    = '"\\ud83d\\ude00"'
cases["str_high_escape"]       = '"\\uD83D\\uDE00"'
cases["str_nul_escape"]        = '"a\\u0000b"'
cases["str_control_raw"]       = '"a\x01b"'
cases["str_multiline"]         = '"a\nb"'
cases["str_tabs"]              = '"a\tb"'
cases["str_backslash_end"]     = '"\\\\"'
cases["str_long"]              = '"' + "x"*5000 + '"'
cases["str_esc_slash"]         = '"a\\/b"'
cases["str_del_0x7f"]          = '"a\x7fb"'

# structural / whitespace
cases["ws_forms"]        = '  \t\r\n {  "a" : 1 , "b" : [ 1 , 2 ] }  \n '
cases["empty_obj"]       = "{}"
cases["empty_arr"]       = "[]"
cases["empty_ws"]        = "   \t\n  "
cases["trailing_comma"]  = '{"a":1,}'
cases["trailing_comma2"] = '[1,2,]'
cases["missing_colon"]   = '{"a" 1}'
cases["double_comma"]    = '[1,,2]'
cases["single_quotes"]   = "{'a':1}"
cases["bom"]             = "﻿" + '{"a":1}'
cases["nul_only"]        = b"\x00".decode("latin-1")
cases["nul_in_middle"]   = '{"a":1}\x00{"b":2}'

# invalid UTF-8 (raw bytes)
cases["utf8_bare_0xff"]  = b'{"a":"\xff"}'.decode("latin-1")
cases["utf8_bare_0xfe"]  = b'["\xfe\xff"]'.decode("latin-1")
cases["utf8_truncated"]  = b'"\xe4\xbd"'.decode("latin-1")
cases["utf8_overlong"]   = b'"\xc0\xaf"'.decode("latin-1")
cases["utf8_surrogate"]  = b'"\xed\xa0\x80"'.decode("latin-1")

# top-level scalars (cJSON parses bare values by default)
cases["bare_true"]   = "true"
cases["bare_false"]  = "false"
cases["bare_null"]   = "null"
cases["bare_number"] = "42"
cases["bare_string"] = '"hello"'
cases["bare_trunc"]  = "tr"

for name, content in cases.items():
    if isinstance(content, str):
        data = content.encode("latin-1", errors="replace")
    else:
        data = content
    with open(os.path.join(out, "edge_%s.json" % name), "wb") as fh:
        fh.write(data)

print("edge cases written: %d" % len(cases))
PY

# 3. minimal valid JSON sanity set (used by differential harness as control)
mkdir -p "$OUT/control"
printf '{}'                   > "$OUT/control/empty_object.json"
printf '[]'                   > "$OUT/control/empty_array.json"
printf '{"a":1}'              > "$OUT/control/simple_object.json"
printf '[1,2,3]'              > "$OUT/control/simple_array.json"
printf '"string"'             > "$OUT/control/simple_string.json"
printf '123'                  > "$OUT/control/simple_number.json"
printf 'true'                 > "$OUT/control/simple_true.json"
printf 'null'                 > "$OUT/control/simple_null.json"
printf '{"nested":{"arr":[1,{"x":[2,3]}]}}' > "$OUT/control/nested.json"

n=$(find "$OUT" -type f | wc -l)
echo "corpus files: $n  (upstream=$n_up, edge=$(find "$OUT" -name 'edge_*' | wc -l), control=$(find "$OUT/control" -type f | wc -l))"
