import re

# ------------------------------------------------------------------ #
# contract: helpers the model kept trying to invent
# ------------------------------------------------------------------ #
p = "scripts/contract.py"
s = open(p).read()

old = '''    /// Append a child, mirroring the C `suffix_object` / append behaviour.
    pub fn push_child(&mut self, child: Node) {
        self.children.push(child);
    }
}'''
new = '''    /// Append a child, mirroring the C `suffix_object` / append behaviour.
    pub fn push_child(&mut self, child: Node) {
        self.children.push(child);
    }

    /// Iterate over the children, mirroring the C sibling loop
    /// `for (item = item->child; item != NULL; item = item->next)`.
    pub fn iter_children(&self) -> std::slice::Iter<'_, Node> {
        self.children.iter()
    }

    /// Iterate over the children mutably.
    pub fn iter_children_mut(&mut self) -> std::slice::IterMut<'_, Node> {
        self.children.iter_mut()
    }

    /// The child at `index`, mirroring the C `get_array_item`.
    pub fn child_at(&self, index: usize) -> Option<&Node> {
        self.children.get(index)
    }

    /// The child at `index`, mutably.
    pub fn child_at_mut(&mut self, index: usize) -> Option<&mut Node> {
        self.children.get_mut(index)
    }

    /// The position of the child equal to `other`, mirroring the C
    /// `cJSON_DetachItemViaPointer` search.
    pub fn position_of(&self, other: &Node) -> Option<usize> {
        self.children.iter().position(|c| c == other)
    }
}'''
assert old in s
s = s.replace(old, new)

# a libc helper the model kept reaching for
old2 = '''// ---------------------------------------------------------------------------
// error reporting
// ---------------------------------------------------------------------------'''
new2 = '''// ---------------------------------------------------------------------------
// small helpers the C code used from libc
// ---------------------------------------------------------------------------
//@BLOCK:libc_helpers
/// Lowercase an ASCII byte. The C code used `tolower` from libc.
pub fn to_lower(b: u8) -> u8 {
    b.to_ascii_lowercase()
}

/// Uppercase an ASCII byte. The C code used `toupper` from libc.
pub fn to_upper(b: u8) -> u8 {
    b.to_ascii_uppercase()
}

// ---------------------------------------------------------------------------
// error reporting
// ---------------------------------------------------------------------------'''
assert old2 in s
s = s.replace(old2, new2)

# expose the new block to contract_for()
old3 = '''    for name in ["consts", "node", "parsebuf", "printbuf", "hooks",
                 "init_hooks", "try_alloc", "error_api"]:'''
new3 = '''    for name in ["consts", "node", "parsebuf", "printbuf", "hooks",
                 "init_hooks", "try_alloc", "libc_helpers", "error_api"]:'''
assert old3 in s
s = s.replace(old3, new3)

open(p, "w").write(s)
print("contract: Node helpers + to_lower/to_upper added")

# ------------------------------------------------------------------ #
# prompt: worked examples + rules for the dominant error modes
# ------------------------------------------------------------------ #
p2 = "scripts/30_translate.py"
t = open(p2).read()

EXAMPLES = '''
WORKED_EXAMPLES = """## Worked examples (follow these patterns exactly)

### A. the C sibling loop becomes a Vec iteration
```c
static void print_children(const cJSON *item, printbuffer *p) {
    const cJSON *child;
    for (child = item->child; child != NULL; child = child->next) {
        print_value(child, p);
    }
}
```
```rust
pub fn print_children(item: &Node, p: &mut PrintBuffer) {
    for child in item.children.iter() {
        let _ = print_value(child, p);
    }
}
```
There is NO `next` field and NO `prev` field on `Node`. The children of a node
live in `node.children: Vec<Node>`.

### B. a NULL check on a reference disappears
```c
static cJSON_bool add_item(cJSON *object, cJSON *item) {
    if ((object == NULL) || (item == NULL)) { return false; }
    object->child = item;
    return true;
}
```
```rust
pub fn add_item(object: &mut Node, item: Node) -> bool {
    object.children.push(item);
    return true;
}
```
A `&mut Node` / `&Node` / `&[u8]` parameter can never be null, so its NULL
check simply disappears. `is_none()` is only valid on an `Option`.

### C. the type field is a bitmask
```c
if ((item->type & cJSON_IsReference) == 0) { ... }
```
```rust
if item.type_ & CJSON_IS_REFERENCE == 0 { ... }
```
Integers have no `contains()` method.

### D. bytes, not characters
```c
if ((*input >= 'A') && (*input <= 'Z')) { *output = (unsigned char)((*input) + 32); }
```
```rust
if b'A' <= *input && *input <= b'Z' {
    *output = to_lower(*input);
}
```
A C character literal `'A'` becomes the Rust byte literal `b'A'`.

### E. allocating a copy of a string
```c
copy = (char*)cJSON_strdup(valuestring, &global_hooks);
if (copy == NULL) { return NULL; }
```
```rust
let copy = cjson_strdup(valuestring, &global_hooks())?;
```
`global_hooks()` is a function that returns the installed hooks.

### F. reading a byte and advancing the cursor
```c
if ((buffer->offset + 1) >= buffer->length) { return false; }
c = buffer->content[buffer->offset];
buffer->offset++;
```
```rust
if buffer.offset + 1 >= buffer.length { return false; }
let c = buffer.content[buffer.offset];
buffer.offset += 1;
```

"""
'''

t = t.replace("PROMPT = \"\"\"Translate this C code to safe, idiomatic Rust.",
              EXAMPLES + "PROMPT = \"\"\"Translate this C code to safe, idiomatic Rust.")

# put the examples right after the contract section
old4 = '''## 2. Functions to implement (exact signatures required)'''
new4 = '''{examples}

## 2. Functions to implement (exact signatures required)'''
assert old4 in t
t = t.replace(old4, new4)

t = t.replace('''        prompt = PROMPT.format(contract=ctext_contract, signatures=signatures,''',
              '''        prompt = PROMPT.format(contract=ctext_contract, examples=WORKED_EXAMPLES,
                                signatures=signatures,''')

# contract block selection: always include the libc helpers
t = t.replace('''        need = ["consts"]''', '''        need = ["consts", "libc_helpers"]''')

# stronger checklist entries for the dominant error modes
t = t.replace('''9. The C `child`/`next`/`prev` chain is `children: Vec<Node>`.''',
              '''9. The C `child`/`next`/`prev` chain is `children: Vec<Node>`. There is NO
   `next` field, NO `prev` field and NO `child` field on `Node`.
   Iterate with `node.children.iter()`.''')
t = t.replace('''3. NO C pointer idioms: no `is_null()`, no `null_mut()`, no `as *mut`, no `NULL`.
   Rust has no null pointers. `Option` is `None`. A `Vec` is never null.''',
              '''3. NO C pointer idioms: no `is_null()`, no `null_mut()`, no `as *mut`, no `NULL`.
   Rust has no null pointers. `Option` is `None`. A `Vec` is never null.
   `is_none()` is ONLY valid on an `Option`. A `&Node`/`&mut Node`/`&[u8]`
   parameter is never null, so its NULL check disappears.''')

open(p2, "w").write(t)
print("prompt: worked examples + stronger rules added")
