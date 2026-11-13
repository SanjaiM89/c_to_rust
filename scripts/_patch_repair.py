p = "scripts/30_translate.py"
t = open(p).read()

# 1. restore the mangled build_repair_prompt
bad = '''def build_crate(code: str, log: str, sigs_txt: str, mod_file: str) -> str:
    """Only show the errors that belong to THIS unit.'''
good = '''def build_repair_prompt(code: str, log: str, sigs_txt: str, mod_file: str) -> str:
    """Only show the errors that belong to THIS unit.'''
assert bad in t
t = t.replace(bad, good)

# 2. add the function-granularity prompt templates at module level
assert "FUNC_PROMPT = " not in t
templates = '''
FUNC_PROMPT = """Translate ONE C function to safe, idiomatic Rust.

## 1. Rust contract - ALREADY WRITTEN, do not redefine these types
```rust
{contract}
```
The contract is EXHAUSTIVE: these are the only types, fields, constants and
helper functions that exist.

## 2. The function to implement (exact signature required)
```rust
{signature};
```

## 3. Rust code of the functions it calls (already translated - call them as they are)
```rust
{callee_code}
```

## 4. The C source of that one function (the behaviour you must preserve)
```c
{csource}
```

## 5. How to convert the C idioms
The Rust types keep the C field names, so most code translates directly:
- `item->type` -> `item.type_` ; `item->valuestring` -> `item.valuestring` ;
  `item->valueint` -> `item.valueint` ; `item->valuedouble` -> `item.valuedouble` ;
  `item->string` -> `item.string`.
- `item->child` -> `item.children.first()` (or `item.child()`).
- `item->next` -> the NEXT element of the `children` Vec. There is no `next` field.
- `item->prev` -> the PREVIOUS element of the `children` Vec.
- `array->child = item` (append) -> `array.children.push(item)`.
- `strcpy(dst, "lit")` -> `dst.copy_from_slice(b"lit")`.
- `strlen(s)` -> `s.len()`.
- `memcpy(d, s, n)` -> `d[..n].copy_from_slice(&s[..n])`.
- `'0'` (a C char literal) -> `b'0'` (a Rust u8). Same for every other literal.
- `INT_MAX` -> `i32::MAX as usize`.
- `NULL` -> `None`; `x == NULL` -> `x.is_none()` ONLY for an `Option`. A `&[u8]`
  parameter is never `None`, so never test it for null.
- A `char *` that the function allocates and returns -> `Option<Vec<u8>>`.
- A `char *` that the function writes into -> `&mut Vec<u8>`.
- `cJSON_Delete(item)` frees memory: in Rust nothing has to be done, the `Node`
  is dropped automatically, so those calls simply disappear.
- The C `hooks` struct fields exist in Rust (`hooks.allocate`, `hooks.deallocate`,
  `hooks.reallocate`), but they take and return `Vec<u8>`, not raw pointers:
  use `hooks.alloc(size)`, `hooks.dealloc(buf)` and `hooks.realloc(buf, size, keep)`.
- A C bitmask test `x & FLAG` -> `x & FLAG != 0`. There is no `.contains()` on an integer.
- The contract's methods need `()`: `item.child()` not `item.child`.

Reply with exactly ONE ```rust code block containing ONLY this function."""

FUNC_REPAIR = """Your Rust function does not compile. Fix it.

## Compiler errors
```
{errors}
```

## Your current code
```rust
{code}
```

## The exact signature you must keep
```rust
{signature};
```

Rules for the fix:
- Fix every error. Do not leave any error in place.
- Keep the behaviour of the C original unchanged.
- Keep the same name and signature.
- If the error says a field or method does not exist, the contract does not
  have it: use the Rust equivalent instead (`Vec` methods, `Option`, slices).
- Reply with the COMPLETE corrected function in exactly one ```rust code block."""


'''
anchor = "def read_json(p):"
assert anchor in t
t = t.replace(anchor, templates.lstrip("\n") + anchor, 1)

open(p, "w").write(t)
print("30_translate.py repaired")
