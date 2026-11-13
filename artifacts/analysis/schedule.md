# Translation schedule — cJSON v1.7.19 → Rust

Units: **14** · functions: **113** · call edges: **162** · leaf-first violations: **0**

Units are ordered **leaf-first**: a unit's callees are always in *earlier* units, so the already-translated Rust signature of every callee is known when a unit is translated. Recursive groups (strongly connected components) are translated together inside a single unit.

## Structs

| struct | file:line | fields |
|---|---|---|
| `error` | cJSON.c:88 | `json: const unsigned char *`, `position: size_t` |
| `internal_hooks` | cJSON.c:156 | `allocate: void * (*)(size_t size)`, `deallocate: void (*)(void *pointer)`, `reallocate: void * (*)(void *pointer, size_t size)` |
| `parse_buffer` | cJSON.c:289 | `content: const unsigned char *`, `length: size_t`, `offset: size_t`, `depth: size_t`, `hooks: internal_hooks` |
| `printbuffer` | cJSON.c:473 | `buffer: unsigned char *`, `length: size_t`, `offset: size_t`, `depth: size_t`, `noalloc: cJSON_bool`, `format: cJSON_bool`, `hooks: internal_hooks` |
| `cJSON` | cJSON.h:103 | `next: struct cJSON *`, `prev: struct cJSON *`, `child: struct cJSON *`, `type: int`, `valuestring: char *`, `valueint: int`, `valuedouble: double`, `string: char *` |
| `cJSON_Hooks` | cJSON.h:125 | `malloc_fn: void * (*)(size_t sz)`, `free_fn: void (*)(void *ptr)` |

## Globals

| global | type | file:line |
|---|---|---|
| `global_error` | `error` | cJSON.c:92 |
| `global_hooks` | `internal_hooks` | cJSON.c:186 |

## Unit order

| # | unit | fns | cyclic | calls earlier | calls later |
|---|---|---|---|---|---|
| 0 | `U01_hooks_alloc` | 6 | - | 0 | 0 |
| 1 | `U02_print_buffer` | 2 | - | 0 | 0 |
| 2 | `U03_number_parse` | 4 | - | 0 | 0 |
| 3 | `U04_string_parse` | 3 | - | 0 | 0 |
| 4 | `U05_string_print` | 2 | - | 1 | 0 |
| 5 | `U06_value_parse` | 9 | yes | 4 | 0 |
| 6 | `U07_value_print` | 9 | yes | 6 | 0 |
| 7 | `U08_accessors` | 22 | - | 0 | 0 |
| 8 | `U09_create` | 20 | - | 4 | 0 |
| 9 | `U10_add_item` | 16 | - | 14 | 0 |
| 10 | `U11_detach_replace` | 13 | - | 8 | 0 |
| 11 | `U12_duplicate` | 2 | - | 3 | 0 |
| 12 | `U13_minify` | 4 | - | 0 | 0 |
| 13 | `U14_compare` | 1 | - | 2 | 0 |

## 0. `U01_hooks_alloc` — allocator hooks, strdup, node lifecycle

Functions (6): `cJSON_Delete`, `cJSON_InitHooks`, `cJSON_New_Item`, `cJSON_free`, `cJSON_malloc`, `cJSON_strdup`

### Signatures and pointer roles

| function | C signature | returns | parameters (role) |
|---|---|---|---|
| `cJSON_Delete` | `void (cJSON *) cJSON_Delete(cJSON * item)` | raw-pointer | `item: cJSON *` **owned-consumed** |
| `cJSON_InitHooks` | `void (cJSON_Hooks *) cJSON_InitHooks(cJSON_Hooks * hooks)` | raw-pointer | `hooks: cJSON_Hooks *` **borrowed-mutable** |
| `cJSON_New_Item` | `cJSON *(const internal_hooks *const) cJSON_New_Item(const internal_hooks *const hooks)` | owning-pointer | `hooks: const internal_hooks *const` **borrowed** |
| `cJSON_free` | `void (void *) cJSON_free(void * object)` | raw-pointer | `object: void *` **borrowed-mutable** |
| `cJSON_malloc` | `void *(size_t) cJSON_malloc(size_t size)` | raw-pointer | `size: size_t` **by-value** |
| `cJSON_strdup` | `unsigned char *(const unsigned char *, const internal_hooks *const) cJSON_strdup(const unsigned char * string, const internal_hooks *const hooks)` | owning-pointer | `string: const unsigned char *` **borrowed**, `hooks: const internal_hooks *const` **borrowed** |

### Macros this unit must expand

| macro | strategy | C expansion | Rust guidance |
|---|---|---|---|
| `CJSON_PUBLIC` | drop | `(function-like)` | C linkage decoration; has no Rust equivalent. Rust items are exported with `pub` and the C ABI is added separately. |
| `cJSON_IsReference` | inline-const | `256` | const CJSON_IS_REFERENCE: u8 = 256; |
| `cJSON_StringIsConst` | inline-const | `512` | const CJSON_STRING_IS_CONST: u8 = 512; |

**Calls in earlier units:** none

**Calls in later units:** none

## 1. `U02_print_buffer` — printbuffer growth and offset bookkeeping

Functions (2): `ensure`, `update_offset`

### Signatures and pointer roles

| function | C signature | returns | parameters (role) |
|---|---|---|---|
| `ensure` | `unsigned char *(printbuffer *const, size_t) ensure(printbuffer *const p, size_t needed)` | borrowed-pointer | `p: printbuffer *const` **borrowed-mutable**, `needed: size_t` **by-value** |
| `update_offset` | `void (printbuffer *const) update_offset(printbuffer *const buffer)` | borrowed-pointer | `buffer: printbuffer *const` **borrowed-mutable** |

**Calls in earlier units:** none

**Calls in later units:** none

## 2. `U03_number_parse` — number scanning and decimal point

Functions (4): `cJSON_SetNumberHelper`, `compare_double`, `get_decimal_point`, `parse_number`

### Signatures and pointer roles

| function | C signature | returns | parameters (role) |
|---|---|---|---|
| `cJSON_SetNumberHelper` | `double (cJSON *, double) cJSON_SetNumberHelper(cJSON * object, double number)` | raw-pointer | `object: cJSON *` **tree-handle**, `number: double` **by-value** |
| `compare_double` | `cJSON_bool (double, double) compare_double(double a, double b)` | by-value | `a: double` **by-value**, `b: double` **by-value** |
| `get_decimal_point` | `unsigned char (void) get_decimal_point()` | by-value | — |
| `parse_number` | `cJSON_bool (cJSON *const, parse_buffer *const) parse_number(cJSON *const item, parse_buffer *const input_buffer)` | borrowed-pointer | `item: cJSON *const` **borrowed-mutable**, `input_buffer: parse_buffer *const` **borrowed-mutable** |

### Macros this unit must expand

| macro | strategy | C expansion | Rust guidance |
|---|---|---|---|
| `CJSON_PUBLIC` | drop | `(function-like)` | C linkage decoration; has no Rust equivalent. Rust items are exported with `pub` and the C ABI is added separately. |
| `buffer_at_offset` | inline-fn | `(function-like)` | inline as `&buffer.content[buffer.offset..]`; the C version can run past the end - the Rust version must not |
| `cJSON_Number` | inline-const | `(1 << 3)` | const CJSON_NUMBER: u8 = 1 << 3; |
| `can_access_at_index` | rust-helper | `(function-like)` | becomes `buffer.can_access_at_index(i) -> bool` with a checked bounds test (this is a Kani proof target) |
| `false` | inline-const | `((cJSON_bool)0)` | Rust's `false` |
| `true` | inline-const | `((cJSON_bool)1)` | Rust's `true` |

**Calls in earlier units:** none

**Calls in later units:** none

## 3. `U04_string_parse` — hex4 decoding, UTF-16 -> UTF-8, string parsing

Functions (3): `parse_hex4`, `parse_string`, `utf16_literal_to_utf8`

### Signatures and pointer roles

| function | C signature | returns | parameters (role) |
|---|---|---|---|
| `parse_hex4` | `unsigned int (const unsigned char *const) parse_hex4(const unsigned char *const input)` | borrowed-pointer | `input: const unsigned char *const` **borrowed** |
| `parse_string` | `cJSON_bool (cJSON *const, parse_buffer *const) parse_string(cJSON *const item, parse_buffer *const input_buffer)` | borrowed-pointer | `item: cJSON *const` **borrowed-mutable**, `input_buffer: parse_buffer *const` **borrowed-mutable** |
| `utf16_literal_to_utf8` | `unsigned char (const unsigned char *const, const unsigned char *const, unsigned char **) utf16_literal_to_utf8(const unsigned char *const input_pointer, const unsigned char *const input_end, unsigned char ** output_pointer)` | borrowed-pointer | `input_pointer: const unsigned char *const` **borrowed**, `input_end: const unsigned char *const` **borrowed**, `output_pointer: unsigned char **` **output-parameter** |

### Macros this unit must expand

| macro | strategy | C expansion | Rust guidance |
|---|---|---|---|
| `buffer_at_offset` | inline-fn | `(function-like)` | inline as `&buffer.content[buffer.offset..]`; the C version can run past the end - the Rust version must not |
| `cJSON_String` | inline-const | `(1 << 4)` | const CJSON_STRING: u8 = 1 << 4; |
| `false` | inline-const | `((cJSON_bool)0)` | Rust's `false` |
| `true` | inline-const | `((cJSON_bool)1)` | Rust's `true` |

**Calls in earlier units:** none

**Calls in later units:** none

## 4. `U05_string_print` — string escaping for output

Functions (2): `print_string`, `print_string_ptr`

### Signatures and pointer roles

| function | C signature | returns | parameters (role) |
|---|---|---|---|
| `print_string` | `cJSON_bool (const cJSON *const, printbuffer *const) print_string(const cJSON *const item, printbuffer *const p)` | borrowed-pointer | `item: const cJSON *const` **borrowed-mutable**, `p: printbuffer *const` **borrowed-mutable** |
| `print_string_ptr` | `cJSON_bool (const unsigned char *const, printbuffer *const) print_string_ptr(const unsigned char *const input, printbuffer *const output_buffer)` | borrowed-pointer | `input: const unsigned char *const` **borrowed-mutable**, `output_buffer: printbuffer *const` **borrowed-mutable** |

### Macros this unit must expand

| macro | strategy | C expansion | Rust guidance |
|---|---|---|---|
| `false` | inline-const | `((cJSON_bool)0)` | Rust's `false` |
| `true` | inline-const | `((cJSON_bool)1)` | Rust's `true` |

**Calls in earlier units:** `ensure`

**Calls in later units:** none

## 5. `U06_value_parse` — value/array/object parsing and the public parse entry points

Functions (9): `buffer_skip_whitespace`, `cJSON_Parse`, `cJSON_ParseWithLength`, `cJSON_ParseWithLengthOpts`, `cJSON_ParseWithOpts`, `parse_array`, `parse_object`, `parse_value`, `skip_utf8_bom`

> **Mutually recursive — must be translated as one unit.** SCC: `parse_array`, `parse_object`, `parse_value`

### Signatures and pointer roles

| function | C signature | returns | parameters (role) |
|---|---|---|---|
| `buffer_skip_whitespace` | `parse_buffer *(parse_buffer *const) buffer_skip_whitespace(parse_buffer *const buffer)` | borrowed-pointer | `buffer: parse_buffer *const` **borrowed-mutable** |
| `cJSON_Parse` | `cJSON *(const char *) cJSON_Parse(const char * value)` | owning-pointer | `value: const char *` **borrowed** |
| `cJSON_ParseWithLength` | `cJSON *(const char *, size_t) cJSON_ParseWithLength(const char * value, size_t buffer_length)` | owning-pointer | `value: const char *` **borrowed**, `buffer_length: size_t` **by-value** |
| `cJSON_ParseWithLengthOpts` | `cJSON *(const char *, size_t, const char **, cJSON_bool) cJSON_ParseWithLengthOpts(const char * value, size_t buffer_length, const char ** return_parse_end, cJSON_bool require_null_terminated)` | owning-pointer | `value: const char *` **borrowed**, `buffer_length: size_t` **by-value**, `return_parse_end: const char **` **output-parameter**, `require_null_terminated: cJSON_bool` **by-value** |
| `cJSON_ParseWithOpts` | `cJSON *(const char *, const char **, cJSON_bool) cJSON_ParseWithOpts(const char * value, const char ** return_parse_end, cJSON_bool require_null_terminated)` | owning-pointer | `value: const char *` **borrowed**, `return_parse_end: const char **` **output-parameter**, `require_null_terminated: cJSON_bool` **by-value** |
| `parse_array` | `cJSON_bool (cJSON *const, parse_buffer *const) parse_array(cJSON *const item, parse_buffer *const input_buffer)` | borrowed-pointer | `item: cJSON *const` **borrowed-mutable**, `input_buffer: parse_buffer *const` **borrowed-mutable** |
| `parse_object` | `cJSON_bool (cJSON *const, parse_buffer *const) parse_object(cJSON *const item, parse_buffer *const input_buffer)` | borrowed-pointer | `item: cJSON *const` **borrowed-mutable**, `input_buffer: parse_buffer *const` **borrowed-mutable** |
| `parse_value` | `cJSON_bool (cJSON *const, parse_buffer *const) parse_value(cJSON *const item, parse_buffer *const input_buffer)` | borrowed-pointer | `item: cJSON *const` **borrowed-mutable**, `input_buffer: parse_buffer *const` **borrowed-mutable** |
| `skip_utf8_bom` | `parse_buffer *(parse_buffer *const) skip_utf8_bom(parse_buffer *const buffer)` | borrowed-pointer | `buffer: parse_buffer *const` **borrowed-mutable** |

### Macros this unit must expand

| macro | strategy | C expansion | Rust guidance |
|---|---|---|---|
| `CJSON_NESTING_LIMIT` | inline-const | `1000` | pub const CJSON_NESTING_LIMIT: usize = 1000;  // parse recursion depth limit - MUST be preserved exactly |
| `CJSON_PUBLIC` | drop | `(function-like)` | C linkage decoration; has no Rust equivalent. Rust items are exported with `pub` and the C ABI is added separately. |
| `buffer_at_offset` | inline-fn | `(function-like)` | inline as `&buffer.content[buffer.offset..]`; the C version can run past the end - the Rust version must not |
| `cJSON_Array` | inline-const | `(1 << 5)` | const CJSON_ARRAY: u8 = 1 << 5; |
| `cJSON_False` | inline-const | `(1 << 0)` | const CJSON_FALSE: u8 = 1 << 0; |
| `cJSON_NULL` | inline-const | `(1 << 2)` | const CJSON_NULL: u8 = 1 << 2; |
| `cJSON_Number` | inline-const | `(1 << 3)` | const CJSON_NUMBER: u8 = 1 << 3; |
| `cJSON_Object` | inline-const | `(1 << 6)` | const CJSON_OBJECT: u8 = 1 << 6; |
| `cJSON_Raw` | inline-const | `(1 << 7)` | const CJSON_RAW: u8 = 1 << 7; |
| `cJSON_String` | inline-const | `(1 << 4)` | const CJSON_STRING: u8 = 1 << 4; |
| `cJSON_True` | inline-const | `(1 << 1)` | const CJSON_TRUE: u8 = 1 << 1; |
| `can_access_at_index` | rust-helper | `(function-like)` | becomes `buffer.can_access_at_index(i) -> bool` with a checked bounds test (this is a Kani proof target) |
| `can_read` | rust-helper | `(function-like)` | becomes `buffer.can_read(n) -> bool` with a checked bounds test (this is a Kani proof target) |
| `cannot_access_at_index` | rust-helper | `(function-like)` | becomes `!buffer.can_access_at_index(i)` |
| `false` | inline-const | `((cJSON_bool)0)` | Rust's `false` |
| `true` | inline-const | `((cJSON_bool)1)` | Rust's `true` |

**Calls in earlier units:** `cJSON_Delete`, `cJSON_New_Item`, `parse_number`, `parse_string`

**Calls in later units:** none

## 6. `U07_value_print` — value/array/object printing and the public print entry points

Functions (9): `cJSON_Print`, `cJSON_PrintBuffered`, `cJSON_PrintPreallocated`, `cJSON_PrintUnformatted`, `print`, `print_array`, `print_number`, `print_object`, `print_value`

> **Mutually recursive — must be translated as one unit.** SCC: `print_array`, `print_object`, `print_value`

### Signatures and pointer roles

| function | C signature | returns | parameters (role) |
|---|---|---|---|
| `cJSON_Print` | `char *(const cJSON *) cJSON_Print(const cJSON * item)` | owning-pointer | `item: const cJSON *` **borrowed** |
| `cJSON_PrintBuffered` | `char *(const cJSON *, int, cJSON_bool) cJSON_PrintBuffered(const cJSON * item, int prebuffer, cJSON_bool fmt)` | owning-pointer | `item: const cJSON *` **borrowed**, `prebuffer: int` **by-value**, `fmt: cJSON_bool` **by-value** |
| `cJSON_PrintPreallocated` | `cJSON_bool (cJSON *, char *, const int, const cJSON_bool) cJSON_PrintPreallocated(cJSON * item, char * buffer, const int length, const cJSON_bool format)` | borrowed-pointer | `item: cJSON *` **tree-handle**, `buffer: char *` **borrowed-mutable**, `length: const int` **by-value**, `format: const cJSON_bool` **by-value** |
| `cJSON_PrintUnformatted` | `char *(const cJSON *) cJSON_PrintUnformatted(const cJSON * item)` | owning-pointer | `item: const cJSON *` **borrowed** |
| `print` | `unsigned char *(const cJSON *const, cJSON_bool, const internal_hooks *const) print(const cJSON *const item, cJSON_bool format, const internal_hooks *const hooks)` | owning-pointer | `item: const cJSON *const` **borrowed**, `format: cJSON_bool` **by-value**, `hooks: const internal_hooks *const` **borrowed** |
| `print_array` | `cJSON_bool (const cJSON *const, printbuffer *const) print_array(const cJSON *const item, printbuffer *const output_buffer)` | borrowed-pointer | `item: const cJSON *const` **borrowed-mutable**, `output_buffer: printbuffer *const` **borrowed-mutable** |
| `print_number` | `cJSON_bool (const cJSON *const, printbuffer *const) print_number(const cJSON *const item, printbuffer *const output_buffer)` | borrowed-pointer | `item: const cJSON *const` **borrowed-mutable**, `output_buffer: printbuffer *const` **borrowed-mutable** |
| `print_object` | `cJSON_bool (const cJSON *const, printbuffer *const) print_object(const cJSON *const item, printbuffer *const output_buffer)` | borrowed-pointer | `item: const cJSON *const` **borrowed-mutable**, `output_buffer: printbuffer *const` **borrowed-mutable** |
| `print_value` | `cJSON_bool (const cJSON *const, printbuffer *const) print_value(const cJSON *const item, printbuffer *const output_buffer)` | borrowed-pointer | `item: const cJSON *const` **borrowed-mutable**, `output_buffer: printbuffer *const` **borrowed-mutable** |

### Macros this unit must expand

| macro | strategy | C expansion | Rust guidance |
|---|---|---|---|
| `CJSON_PUBLIC` | drop | `(function-like)` | C linkage decoration; has no Rust equivalent. Rust items are exported with `pub` and the C ABI is added separately. |
| `cJSON_Array` | inline-const | `(1 << 5)` | const CJSON_ARRAY: u8 = 1 << 5; |
| `cJSON_False` | inline-const | `(1 << 0)` | const CJSON_FALSE: u8 = 1 << 0; |
| `cJSON_NULL` | inline-const | `(1 << 2)` | const CJSON_NULL: u8 = 1 << 2; |
| `cJSON_Number` | inline-const | `(1 << 3)` | const CJSON_NUMBER: u8 = 1 << 3; |
| `cJSON_Object` | inline-const | `(1 << 6)` | const CJSON_OBJECT: u8 = 1 << 6; |
| `cJSON_Raw` | inline-const | `(1 << 7)` | const CJSON_RAW: u8 = 1 << 7; |
| `cJSON_String` | inline-const | `(1 << 4)` | const CJSON_STRING: u8 = 1 << 4; |
| `cJSON_True` | inline-const | `(1 << 1)` | const CJSON_TRUE: u8 = 1 << 1; |
| `cjson_min` | inline-fn | `(function-like)` | inline as Rust `a.min(b)` |
| `false` | inline-const | `((cJSON_bool)0)` | Rust's `false` |
| `isinf` | inline-fn | `(function-like)` | Rust's `f64::is_infinite()` |
| `isnan` | inline-fn | `(function-like)` | Rust's `f64::is_nan()` |
| `true` | inline-const | `((cJSON_bool)1)` | Rust's `true` |

**Calls in earlier units:** `compare_double`, `ensure`, `get_decimal_point`, `print_string`, `print_string_ptr`, `update_offset`

**Calls in later units:** none

## 7. `U08_accessors` — read-only accessors and type predicates

Functions (22): `cJSON_GetArrayItem`, `cJSON_GetArraySize`, `cJSON_GetErrorPtr`, `cJSON_GetNumberValue`, `cJSON_GetObjectItem`, `cJSON_GetObjectItemCaseSensitive`, `cJSON_GetStringValue`, `cJSON_HasObjectItem`, `cJSON_IsArray`, `cJSON_IsBool`, `cJSON_IsFalse`, `cJSON_IsInvalid`, `cJSON_IsNull`, `cJSON_IsNumber`, `cJSON_IsObject`, `cJSON_IsRaw`, `cJSON_IsString`, `cJSON_IsTrue`, `cJSON_Version`, `case_insensitive_strcmp`, `get_array_item`, `get_object_item`

### Signatures and pointer roles

| function | C signature | returns | parameters (role) |
|---|---|---|---|
| `cJSON_GetArrayItem` | `cJSON *(const cJSON *, int) cJSON_GetArrayItem(const cJSON * array, int index)` | borrowed-pointer | `array: const cJSON *` **borrowed**, `index: int` **by-value** |
| `cJSON_GetArraySize` | `int (const cJSON *) cJSON_GetArraySize(const cJSON * array)` | borrowed-pointer | `array: const cJSON *` **borrowed** |
| `cJSON_GetErrorPtr` | `const char *(void) cJSON_GetErrorPtr()` | borrowed-pointer | — |
| `cJSON_GetNumberValue` | `double (const cJSON *const) cJSON_GetNumberValue(const cJSON *const item)` | borrowed-pointer | `item: const cJSON *const` **borrowed** |
| `cJSON_GetObjectItem` | `cJSON *(const cJSON *const, const char *const) cJSON_GetObjectItem(const cJSON *const object, const char *const string)` | borrowed-pointer | `object: const cJSON *const` **borrowed**, `string: const char *const` **borrowed** |
| `cJSON_GetObjectItemCaseSensitive` | `cJSON *(const cJSON *const, const char *const) cJSON_GetObjectItemCaseSensitive(const cJSON *const object, const char *const string)` | borrowed-pointer | `object: const cJSON *const` **borrowed**, `string: const char *const` **borrowed** |
| `cJSON_GetStringValue` | `char *(const cJSON *const) cJSON_GetStringValue(const cJSON *const item)` | borrowed-pointer | `item: const cJSON *const` **borrowed** |
| `cJSON_HasObjectItem` | `cJSON_bool (const cJSON *, const char *) cJSON_HasObjectItem(const cJSON * object, const char * string)` | borrowed-pointer | `object: const cJSON *` **borrowed**, `string: const char *` **borrowed** |
| `cJSON_IsArray` | `cJSON_bool (const cJSON *const) cJSON_IsArray(const cJSON *const item)` | borrowed-pointer | `item: const cJSON *const` **borrowed** |
| `cJSON_IsBool` | `cJSON_bool (const cJSON *const) cJSON_IsBool(const cJSON *const item)` | borrowed-pointer | `item: const cJSON *const` **borrowed** |
| `cJSON_IsFalse` | `cJSON_bool (const cJSON *const) cJSON_IsFalse(const cJSON *const item)` | borrowed-pointer | `item: const cJSON *const` **borrowed** |
| `cJSON_IsInvalid` | `cJSON_bool (const cJSON *const) cJSON_IsInvalid(const cJSON *const item)` | borrowed-pointer | `item: const cJSON *const` **borrowed** |
| `cJSON_IsNull` | `cJSON_bool (const cJSON *const) cJSON_IsNull(const cJSON *const item)` | borrowed-pointer | `item: const cJSON *const` **borrowed** |
| `cJSON_IsNumber` | `cJSON_bool (const cJSON *const) cJSON_IsNumber(const cJSON *const item)` | borrowed-pointer | `item: const cJSON *const` **borrowed** |
| `cJSON_IsObject` | `cJSON_bool (const cJSON *const) cJSON_IsObject(const cJSON *const item)` | borrowed-pointer | `item: const cJSON *const` **borrowed** |
| `cJSON_IsRaw` | `cJSON_bool (const cJSON *const) cJSON_IsRaw(const cJSON *const item)` | borrowed-pointer | `item: const cJSON *const` **borrowed** |
| `cJSON_IsString` | `cJSON_bool (const cJSON *const) cJSON_IsString(const cJSON *const item)` | borrowed-pointer | `item: const cJSON *const` **borrowed** |
| `cJSON_IsTrue` | `cJSON_bool (const cJSON *const) cJSON_IsTrue(const cJSON *const item)` | borrowed-pointer | `item: const cJSON *const` **borrowed** |
| `cJSON_Version` | `const char *(void) cJSON_Version()` | borrowed-pointer | — |
| `case_insensitive_strcmp` | `int (const unsigned char *, const unsigned char *) case_insensitive_strcmp(const unsigned char * string1, const unsigned char * string2)` | borrowed-pointer | `string1: const unsigned char *` **borrowed**, `string2: const unsigned char *` **borrowed** |
| `get_array_item` | `cJSON *(const cJSON *, size_t) get_array_item(const cJSON * array, size_t index)` | borrowed-pointer | `array: const cJSON *` **borrowed**, `index: size_t` **by-value** |
| `get_object_item` | `cJSON *(const cJSON *const, const char *const, const cJSON_bool) get_object_item(const cJSON *const object, const char *const name, const cJSON_bool case_sensitive)` | borrowed-pointer | `object: const cJSON *const` **borrowed**, `name: const char *const` **borrowed**, `case_sensitive: const cJSON_bool` **by-value** |

### Macros this unit must expand

| macro | strategy | C expansion | Rust guidance |
|---|---|---|---|
| `CJSON_PUBLIC` | drop | `(function-like)` | C linkage decoration; has no Rust equivalent. Rust items are exported with `pub` and the C ABI is added separately. |
| `CJSON_VERSION_MAJOR` | inline-const | `1` | pub const CJSON_VERSION_MAJOR: u32 = 1; |
| `CJSON_VERSION_MINOR` | inline-const | `7` | pub const CJSON_VERSION_MINOR: u32 = 7; |
| `CJSON_VERSION_PATCH` | inline-const | `19` | pub const CJSON_VERSION_PATCH: u32 = 19; |
| `NAN` | inline-fn | `(__builtin_nanf (""))` | Rust's `f64::NAN` |
| `cJSON_Array` | inline-const | `(1 << 5)` | const CJSON_ARRAY: u8 = 1 << 5; |
| `cJSON_False` | inline-const | `(1 << 0)` | const CJSON_FALSE: u8 = 1 << 0; |
| `cJSON_Invalid` | inline-const | `(0)` | const CJSON_INVALID: u8 = 0; |
| `cJSON_NULL` | inline-const | `(1 << 2)` | const CJSON_NULL: u8 = 1 << 2; |
| `cJSON_Number` | inline-const | `(1 << 3)` | const CJSON_NUMBER: u8 = 1 << 3; |
| `cJSON_Object` | inline-const | `(1 << 6)` | const CJSON_OBJECT: u8 = 1 << 6; |
| `cJSON_Raw` | inline-const | `(1 << 7)` | const CJSON_RAW: u8 = 1 << 7; |
| `cJSON_String` | inline-const | `(1 << 4)` | const CJSON_STRING: u8 = 1 << 4; |
| `cJSON_True` | inline-const | `(1 << 1)` | const CJSON_TRUE: u8 = 1 << 1; |
| `false` | inline-const | `((cJSON_bool)0)` | Rust's `false` |
| `true` | inline-const | `((cJSON_bool)1)` | Rust's `true` |

**Calls in earlier units:** none

**Calls in later units:** none

## 8. `U09_create` — node constructors

Functions (20): `cJSON_CreateArray`, `cJSON_CreateArrayReference`, `cJSON_CreateBool`, `cJSON_CreateDoubleArray`, `cJSON_CreateFalse`, `cJSON_CreateFloatArray`, `cJSON_CreateIntArray`, `cJSON_CreateNull`, `cJSON_CreateNumber`, `cJSON_CreateObject`, `cJSON_CreateObjectReference`, `cJSON_CreateRaw`, `cJSON_CreateString`, `cJSON_CreateStringArray`, `cJSON_CreateStringReference`, `cJSON_CreateTrue`, `cJSON_SetValuestring`, `cast_away_const`, `create_reference`, `suffix_object`

### Signatures and pointer roles

| function | C signature | returns | parameters (role) |
|---|---|---|---|
| `cJSON_CreateArray` | `cJSON *(void) cJSON_CreateArray()` | owning-pointer | — |
| `cJSON_CreateArrayReference` | `cJSON *(const cJSON *) cJSON_CreateArrayReference(const cJSON * child)` | owning-pointer | `child: const cJSON *` **borrowed** |
| `cJSON_CreateBool` | `cJSON *(cJSON_bool) cJSON_CreateBool(cJSON_bool boolean)` | owning-pointer | `boolean: cJSON_bool` **by-value** |
| `cJSON_CreateDoubleArray` | `cJSON *(const double *, int) cJSON_CreateDoubleArray(const double * numbers, int count)` | owning-pointer | `numbers: const double *` **borrowed**, `count: int` **by-value** |
| `cJSON_CreateFalse` | `cJSON *(void) cJSON_CreateFalse()` | owning-pointer | — |
| `cJSON_CreateFloatArray` | `cJSON *(const float *, int) cJSON_CreateFloatArray(const float * numbers, int count)` | owning-pointer | `numbers: const float *` **borrowed**, `count: int` **by-value** |
| `cJSON_CreateIntArray` | `cJSON *(const int *, int) cJSON_CreateIntArray(const int * numbers, int count)` | owning-pointer | `numbers: const int *` **borrowed**, `count: int` **by-value** |
| `cJSON_CreateNull` | `cJSON *(void) cJSON_CreateNull()` | owning-pointer | — |
| `cJSON_CreateNumber` | `cJSON *(double) cJSON_CreateNumber(double num)` | owning-pointer | `num: double` **by-value** |
| `cJSON_CreateObject` | `cJSON *(void) cJSON_CreateObject()` | owning-pointer | — |
| `cJSON_CreateObjectReference` | `cJSON *(const cJSON *) cJSON_CreateObjectReference(const cJSON * child)` | owning-pointer | `child: const cJSON *` **borrowed** |
| `cJSON_CreateRaw` | `cJSON *(const char *) cJSON_CreateRaw(const char * raw)` | owning-pointer | `raw: const char *` **borrowed** |
| `cJSON_CreateString` | `cJSON *(const char *) cJSON_CreateString(const char * string)` | owning-pointer | `string: const char *` **borrowed** |
| `cJSON_CreateStringArray` | `cJSON *(const char *const *, int) cJSON_CreateStringArray(const char *const * strings, int count)` | owning-pointer | `strings: const char *const *` **output-parameter**, `count: int` **by-value** |
| `cJSON_CreateStringReference` | `cJSON *(const char *) cJSON_CreateStringReference(const char * string)` | owning-pointer | `string: const char *` **borrowed** |
| `cJSON_CreateTrue` | `cJSON *(void) cJSON_CreateTrue()` | owning-pointer | — |
| `cJSON_SetValuestring` | `char *(cJSON *, const char *) cJSON_SetValuestring(cJSON * object, const char * valuestring)` | borrowed-pointer | `object: cJSON *` **tree-handle**, `valuestring: const char *` **borrowed** |
| `cast_away_const` | `void *(const void *) cast_away_const(const void * string)` | borrowed-pointer | `string: const void *` **borrowed** |
| `create_reference` | `cJSON *(const cJSON *, const internal_hooks *const) create_reference(const cJSON * item, const internal_hooks *const hooks)` | borrowed-pointer | `item: const cJSON *` **borrowed**, `hooks: const internal_hooks *const` **borrowed** |
| `suffix_object` | `void (cJSON *, cJSON *) suffix_object(cJSON * prev, cJSON * item)` | raw-pointer | `prev: cJSON *` **tree-handle**, `item: cJSON *` **tree-handle** |

### Macros this unit must expand

| macro | strategy | C expansion | Rust guidance |
|---|---|---|---|
| `CJSON_PUBLIC` | drop | `(function-like)` | C linkage decoration; has no Rust equivalent. Rust items are exported with `pub` and the C ABI is added separately. |
| `cJSON_Array` | inline-const | `(1 << 5)` | const CJSON_ARRAY: u8 = 1 << 5; |
| `cJSON_False` | inline-const | `(1 << 0)` | const CJSON_FALSE: u8 = 1 << 0; |
| `cJSON_IsReference` | inline-const | `256` | const CJSON_IS_REFERENCE: u8 = 256; |
| `cJSON_NULL` | inline-const | `(1 << 2)` | const CJSON_NULL: u8 = 1 << 2; |
| `cJSON_Number` | inline-const | `(1 << 3)` | const CJSON_NUMBER: u8 = 1 << 3; |
| `cJSON_Object` | inline-const | `(1 << 6)` | const CJSON_OBJECT: u8 = 1 << 6; |
| `cJSON_Raw` | inline-const | `(1 << 7)` | const CJSON_RAW: u8 = 1 << 7; |
| `cJSON_String` | inline-const | `(1 << 4)` | const CJSON_STRING: u8 = 1 << 4; |
| `cJSON_True` | inline-const | `(1 << 1)` | const CJSON_TRUE: u8 = 1 << 1; |

**Calls in earlier units:** `cJSON_Delete`, `cJSON_New_Item`, `cJSON_free`, `cJSON_strdup`

**Calls in later units:** none

## 9. `U10_add_item` — attaching nodes to arrays and objects

Functions (16): `add_item_to_array`, `add_item_to_object`, `cJSON_AddArrayToObject`, `cJSON_AddBoolToObject`, `cJSON_AddFalseToObject`, `cJSON_AddItemReferenceToArray`, `cJSON_AddItemReferenceToObject`, `cJSON_AddItemToArray`, `cJSON_AddItemToObject`, `cJSON_AddItemToObjectCS`, `cJSON_AddNullToObject`, `cJSON_AddNumberToObject`, `cJSON_AddObjectToObject`, `cJSON_AddRawToObject`, `cJSON_AddStringToObject`, `cJSON_AddTrueToObject`

### Signatures and pointer roles

| function | C signature | returns | parameters (role) |
|---|---|---|---|
| `add_item_to_array` | `cJSON_bool (cJSON *, cJSON *) add_item_to_array(cJSON * array, cJSON * item)` | raw-pointer | `array: cJSON *` **tree-handle**, `item: cJSON *` **tree-handle** |
| `add_item_to_object` | `cJSON_bool (cJSON *const, const char *const, cJSON *const, const internal_hooks *const, const cJSON_bool) add_item_to_object(cJSON *const object, const char *const string, cJSON *const item, const internal_hooks *const hooks, const cJSON_bool constant_key)` | borrowed-pointer | `object: cJSON *const` **borrowed**, `string: const char *const` **borrowed**, `item: cJSON *const` **borrowed**, `hooks: const internal_hooks *const` **borrowed**, `constant_key: const cJSON_bool` **by-value** |
| `cJSON_AddArrayToObject` | `cJSON *(cJSON *const, const char *const) cJSON_AddArrayToObject(cJSON *const object, const char *const name)` | borrowed-pointer | `object: cJSON *const` **borrowed**, `name: const char *const` **borrowed** |
| `cJSON_AddBoolToObject` | `cJSON *(cJSON *const, const char *const, const cJSON_bool) cJSON_AddBoolToObject(cJSON *const object, const char *const name, const cJSON_bool boolean)` | borrowed-pointer | `object: cJSON *const` **borrowed**, `name: const char *const` **borrowed**, `boolean: const cJSON_bool` **by-value** |
| `cJSON_AddFalseToObject` | `cJSON *(cJSON *const, const char *const) cJSON_AddFalseToObject(cJSON *const object, const char *const name)` | borrowed-pointer | `object: cJSON *const` **borrowed**, `name: const char *const` **borrowed** |
| `cJSON_AddItemReferenceToArray` | `cJSON_bool (cJSON *, cJSON *) cJSON_AddItemReferenceToArray(cJSON * array, cJSON * item)` | raw-pointer | `array: cJSON *` **tree-handle**, `item: cJSON *` **tree-handle** |
| `cJSON_AddItemReferenceToObject` | `cJSON_bool (cJSON *, const char *, cJSON *) cJSON_AddItemReferenceToObject(cJSON * object, const char * string, cJSON * item)` | borrowed-pointer | `object: cJSON *` **tree-handle**, `string: const char *` **borrowed**, `item: cJSON *` **tree-handle** |
| `cJSON_AddItemToArray` | `cJSON_bool (cJSON *, cJSON *) cJSON_AddItemToArray(cJSON * array, cJSON * item)` | raw-pointer | `array: cJSON *` **owned-transferred**, `item: cJSON *` **owned-transferred** |
| `cJSON_AddItemToObject` | `cJSON_bool (cJSON *, const char *, cJSON *) cJSON_AddItemToObject(cJSON * object, const char * string, cJSON * item)` | borrowed-pointer | `object: cJSON *` **owned-transferred**, `string: const char *` **borrowed**, `item: cJSON *` **owned-transferred** |
| `cJSON_AddItemToObjectCS` | `cJSON_bool (cJSON *, const char *, cJSON *) cJSON_AddItemToObjectCS(cJSON * object, const char * string, cJSON * item)` | borrowed-pointer | `object: cJSON *` **owned-transferred**, `string: const char *` **borrowed**, `item: cJSON *` **owned-transferred** |
| `cJSON_AddNullToObject` | `cJSON *(cJSON *const, const char *const) cJSON_AddNullToObject(cJSON *const object, const char *const name)` | borrowed-pointer | `object: cJSON *const` **borrowed**, `name: const char *const` **borrowed** |
| `cJSON_AddNumberToObject` | `cJSON *(cJSON *const, const char *const, const double) cJSON_AddNumberToObject(cJSON *const object, const char *const name, const double number)` | borrowed-pointer | `object: cJSON *const` **borrowed**, `name: const char *const` **borrowed**, `number: const double` **by-value** |
| `cJSON_AddObjectToObject` | `cJSON *(cJSON *const, const char *const) cJSON_AddObjectToObject(cJSON *const object, const char *const name)` | borrowed-pointer | `object: cJSON *const` **borrowed**, `name: const char *const` **borrowed** |
| `cJSON_AddRawToObject` | `cJSON *(cJSON *const, const char *const, const char *const) cJSON_AddRawToObject(cJSON *const object, const char *const name, const char *const raw)` | borrowed-pointer | `object: cJSON *const` **borrowed**, `name: const char *const` **borrowed**, `raw: const char *const` **borrowed** |
| `cJSON_AddStringToObject` | `cJSON *(cJSON *const, const char *const, const char *const) cJSON_AddStringToObject(cJSON *const object, const char *const name, const char *const string)` | borrowed-pointer | `object: cJSON *const` **borrowed**, `name: const char *const` **borrowed**, `string: const char *const` **borrowed** |
| `cJSON_AddTrueToObject` | `cJSON *(cJSON *const, const char *const) cJSON_AddTrueToObject(cJSON *const object, const char *const name)` | borrowed-pointer | `object: cJSON *const` **borrowed**, `name: const char *const` **borrowed** |

### Macros this unit must expand

| macro | strategy | C expansion | Rust guidance |
|---|---|---|---|
| `CJSON_PUBLIC` | drop | `(function-like)` | C linkage decoration; has no Rust equivalent. Rust items are exported with `pub` and the C ABI is added separately. |
| `cJSON_Invalid` | inline-const | `(0)` | const CJSON_INVALID: u8 = 0; |
| `cJSON_StringIsConst` | inline-const | `512` | const CJSON_STRING_IS_CONST: u8 = 512; |
| `false` | inline-const | `((cJSON_bool)0)` | Rust's `false` |
| `true` | inline-const | `((cJSON_bool)1)` | Rust's `true` |

**Calls in earlier units:** `cJSON_CreateArray`, `cJSON_CreateBool`, `cJSON_CreateFalse`, `cJSON_CreateNull`, `cJSON_CreateNumber`, `cJSON_CreateObject`, `cJSON_CreateRaw`, `cJSON_CreateString`, `cJSON_CreateTrue`, `cJSON_Delete`, `cJSON_strdup`, `cast_away_const`, `create_reference`, `suffix_object`

**Calls in later units:** none

## 10. `U11_detach_replace` — detaching, deleting and replacing nodes

Functions (13): `cJSON_DeleteItemFromArray`, `cJSON_DeleteItemFromObject`, `cJSON_DeleteItemFromObjectCaseSensitive`, `cJSON_DetachItemFromArray`, `cJSON_DetachItemFromObject`, `cJSON_DetachItemFromObjectCaseSensitive`, `cJSON_DetachItemViaPointer`, `cJSON_InsertItemInArray`, `cJSON_ReplaceItemInArray`, `cJSON_ReplaceItemInObject`, `cJSON_ReplaceItemInObjectCaseSensitive`, `cJSON_ReplaceItemViaPointer`, `replace_item_in_object`

### Signatures and pointer roles

| function | C signature | returns | parameters (role) |
|---|---|---|---|
| `cJSON_DeleteItemFromArray` | `void (cJSON *, int) cJSON_DeleteItemFromArray(cJSON * array, int which)` | raw-pointer | `array: cJSON *` **owned-consumed**, `which: int` **by-value** |
| `cJSON_DeleteItemFromObject` | `void (cJSON *, const char *) cJSON_DeleteItemFromObject(cJSON * object, const char * string)` | borrowed-pointer | `object: cJSON *` **owned-consumed**, `string: const char *` **borrowed** |
| `cJSON_DeleteItemFromObjectCaseSensitive` | `void (cJSON *, const char *) cJSON_DeleteItemFromObjectCaseSensitive(cJSON * object, const char * string)` | borrowed-pointer | `object: cJSON *` **owned-consumed**, `string: const char *` **borrowed** |
| `cJSON_DetachItemFromArray` | `cJSON *(cJSON *, int) cJSON_DetachItemFromArray(cJSON * array, int which)` | owning-pointer | `array: cJSON *` **tree-handle**, `which: int` **by-value** |
| `cJSON_DetachItemFromObject` | `cJSON *(cJSON *, const char *) cJSON_DetachItemFromObject(cJSON * object, const char * string)` | owning-pointer | `object: cJSON *` **tree-handle**, `string: const char *` **borrowed** |
| `cJSON_DetachItemFromObjectCaseSensitive` | `cJSON *(cJSON *, const char *) cJSON_DetachItemFromObjectCaseSensitive(cJSON * object, const char * string)` | owning-pointer | `object: cJSON *` **tree-handle**, `string: const char *` **borrowed** |
| `cJSON_DetachItemViaPointer` | `cJSON *(cJSON *, cJSON *const) cJSON_DetachItemViaPointer(cJSON * parent, cJSON *const item)` | owning-pointer | `parent: cJSON *` **tree-handle**, `item: cJSON *const` **borrowed** |
| `cJSON_InsertItemInArray` | `cJSON_bool (cJSON *, int, cJSON *) cJSON_InsertItemInArray(cJSON * array, int which, cJSON * newitem)` | raw-pointer | `array: cJSON *` **owned-transferred**, `which: int` **by-value**, `newitem: cJSON *` **owned-transferred** |
| `cJSON_ReplaceItemInArray` | `cJSON_bool (cJSON *, int, cJSON *) cJSON_ReplaceItemInArray(cJSON * array, int which, cJSON * newitem)` | raw-pointer | `array: cJSON *` **owned-transferred**, `which: int` **by-value**, `newitem: cJSON *` **owned-transferred** |
| `cJSON_ReplaceItemInObject` | `cJSON_bool (cJSON *, const char *, cJSON *) cJSON_ReplaceItemInObject(cJSON * object, const char * string, cJSON * newitem)` | borrowed-pointer | `object: cJSON *` **owned-transferred**, `string: const char *` **borrowed**, `newitem: cJSON *` **owned-transferred** |
| `cJSON_ReplaceItemInObjectCaseSensitive` | `cJSON_bool (cJSON *, const char *, cJSON *) cJSON_ReplaceItemInObjectCaseSensitive(cJSON * object, const char * string, cJSON * newitem)` | borrowed-pointer | `object: cJSON *` **owned-transferred**, `string: const char *` **borrowed**, `newitem: cJSON *` **owned-transferred** |
| `cJSON_ReplaceItemViaPointer` | `cJSON_bool (cJSON *const, cJSON *const, cJSON *) cJSON_ReplaceItemViaPointer(cJSON *const parent, cJSON *const item, cJSON * replacement)` | borrowed-pointer | `parent: cJSON *const` **borrowed**, `item: cJSON *const` **borrowed**, `replacement: cJSON *` **owned-transferred** |
| `replace_item_in_object` | `cJSON_bool (cJSON *, const char *, cJSON *, cJSON_bool) replace_item_in_object(cJSON * object, const char * string, cJSON * replacement, cJSON_bool case_sensitive)` | borrowed-pointer | `object: cJSON *` **tree-handle**, `string: const char *` **borrowed**, `replacement: cJSON *` **tree-handle**, `case_sensitive: cJSON_bool` **by-value** |

### Macros this unit must expand

| macro | strategy | C expansion | Rust guidance |
|---|---|---|---|
| `CJSON_PUBLIC` | drop | `(function-like)` | C linkage decoration; has no Rust equivalent. Rust items are exported with `pub` and the C ABI is added separately. |
| `cJSON_StringIsConst` | inline-const | `512` | const CJSON_STRING_IS_CONST: u8 = 512; |
| `false` | inline-const | `((cJSON_bool)0)` | Rust's `false` |
| `true` | inline-const | `((cJSON_bool)1)` | Rust's `true` |

**Calls in earlier units:** `add_item_to_array`, `cJSON_Delete`, `cJSON_GetObjectItem`, `cJSON_GetObjectItemCaseSensitive`, `cJSON_free`, `cJSON_strdup`, `get_array_item`, `get_object_item`

**Calls in later units:** none

## 11. `U12_duplicate` — deep copy

Functions (2): `cJSON_Duplicate`, `cJSON_Duplicate_rec`

### Signatures and pointer roles

| function | C signature | returns | parameters (role) |
|---|---|---|---|
| `cJSON_Duplicate` | `cJSON *(const cJSON *, cJSON_bool) cJSON_Duplicate(const cJSON * item, cJSON_bool recurse)` | owning-pointer | `item: const cJSON *` **borrowed**, `recurse: cJSON_bool` **by-value** |
| `cJSON_Duplicate_rec` | `cJSON *(const cJSON *, size_t, cJSON_bool) cJSON_Duplicate_rec(const cJSON * item, size_t depth, cJSON_bool recurse)` | owning-pointer | `item: const cJSON *` **borrowed**, `depth: size_t` **by-value**, `recurse: cJSON_bool` **by-value** |

### Macros this unit must expand

| macro | strategy | C expansion | Rust guidance |
|---|---|---|---|
| `CJSON_CIRCULAR_LIMIT` | inline-const | `10000` | pub const CJSON_CIRCULAR_LIMIT: usize = 10000;  // duplicate/compare cycle limit - MUST be preserved exactly |
| `CJSON_PUBLIC` | drop | `(function-like)` | C linkage decoration; has no Rust equivalent. Rust items are exported with `pub` and the C ABI is added separately. |
| `cJSON_IsReference` | inline-const | `256` | const CJSON_IS_REFERENCE: u8 = 256; |
| `cJSON_StringIsConst` | inline-const | `512` | const CJSON_STRING_IS_CONST: u8 = 512; |
| `true` | inline-const | `((cJSON_bool)1)` | Rust's `true` |

**Calls in earlier units:** `cJSON_Delete`, `cJSON_New_Item`, `cJSON_strdup`

**Calls in later units:** none

## 12. `U13_minify` — whitespace/comment stripping

Functions (4): `cJSON_Minify`, `minify_string`, `skip_multiline_comment`, `skip_oneline_comment`

### Signatures and pointer roles

| function | C signature | returns | parameters (role) |
|---|---|---|---|
| `cJSON_Minify` | `void (char *) cJSON_Minify(char * json)` | raw-pointer | `json: char *` **borrowed-mutable** |
| `minify_string` | `void (char **, char **) minify_string(char ** input, char ** output)` | raw-pointer | `input: char **` **output-parameter**, `output: char **` **output-parameter** |
| `skip_multiline_comment` | `void (char **) skip_multiline_comment(char ** input)` | raw-pointer | `input: char **` **output-parameter** |
| `skip_oneline_comment` | `void (char **) skip_oneline_comment(char ** input)` | raw-pointer | `input: char **` **output-parameter** |

### Macros this unit must expand

| macro | strategy | C expansion | Rust guidance |
|---|---|---|---|
| `CJSON_PUBLIC` | drop | `(function-like)` | C linkage decoration; has no Rust equivalent. Rust items are exported with `pub` and the C ABI is added separately. |
| `static_strlen` | inline-fn | `(function-like)` | C's `sizeof(literal) - 1`; in Rust use `literal.len()` on a `&str` |

**Calls in earlier units:** none

**Calls in later units:** none

## 13. `U14_compare` — deep structural comparison (used by cJSON_Utils)

Functions (1): `cJSON_Compare`

### Signatures and pointer roles

| function | C signature | returns | parameters (role) |
|---|---|---|---|
| `cJSON_Compare` | `cJSON_bool (const cJSON *const, const cJSON *const, const cJSON_bool) cJSON_Compare(const cJSON *const a, const cJSON *const b, const cJSON_bool case_sensitive)` | borrowed-pointer | `a: const cJSON *const` **borrowed**, `b: const cJSON *const` **borrowed**, `case_sensitive: const cJSON_bool` **by-value** |

### Macros this unit must expand

| macro | strategy | C expansion | Rust guidance |
|---|---|---|---|
| `CJSON_PUBLIC` | drop | `(function-like)` | C linkage decoration; has no Rust equivalent. Rust items are exported with `pub` and the C ABI is added separately. |
| `cJSON_Array` | inline-const | `(1 << 5)` | const CJSON_ARRAY: u8 = 1 << 5; |
| `cJSON_ArrayForEach` | rust-helper | `for(element = (array != NULL) ? (array)->child : NULL; element != NULL; element = element->next)` | becomes an iterator: `node.children()` returning `impl Iterator<Item = &Node>` - do NOT port the for-loop macro |
| `cJSON_False` | inline-const | `(1 << 0)` | const CJSON_FALSE: u8 = 1 << 0; |
| `cJSON_NULL` | inline-const | `(1 << 2)` | const CJSON_NULL: u8 = 1 << 2; |
| `cJSON_Number` | inline-const | `(1 << 3)` | const CJSON_NUMBER: u8 = 1 << 3; |
| `cJSON_Object` | inline-const | `(1 << 6)` | const CJSON_OBJECT: u8 = 1 << 6; |
| `cJSON_Raw` | inline-const | `(1 << 7)` | const CJSON_RAW: u8 = 1 << 7; |
| `cJSON_String` | inline-const | `(1 << 4)` | const CJSON_STRING: u8 = 1 << 4; |
| `cJSON_True` | inline-const | `(1 << 1)` | const CJSON_TRUE: u8 = 1 << 1; |
| `false` | inline-const | `((cJSON_bool)0)` | Rust's `false` |
| `true` | inline-const | `((cJSON_bool)1)` | Rust's `true` |

**Calls in earlier units:** `compare_double`, `get_object_item`

**Calls in later units:** none

## What the analysis could NOT resolve

### recursive-scc

these functions are mutually recursive and MUST be translated together

- parse_array / parse_object / parse_value
- print_array / print_object / print_value

### analysis-limitation

pointer roles are heuristic (syntax + known-function table); the aliasing/ownership semantics of `cJSON` trees are NOT proven by static analysis - they are established empirically in Phase 4

### analysis-limitation

the call graph contains only direct calls; indirect calls through function pointers (hooks, localeconv) are not resolved

- add_item_to_object
- cJSON_InitHooks
- cJSON_New_Item
- cJSON_strdup
- create_reference
- print
