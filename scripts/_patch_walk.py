p = "scripts/30_translate.py"
t = open(p).read()

WALK_EXAMPLE = '''- The C code's `input_pointer`, `output_pointer`, `input_end` are NOT Rust
  pointers. Use a `usize` index into the buffer, or push into a `Vec<u8>`:
  ```c
  while (input_pointer < input_end) { c = *input_pointer; input_pointer++; }
  *output_pointer = c; output_pointer++;
  ```
  ```rust
  let mut i = start;
  while i < end {
      let c = input_buffer.content[i];
      i += 1;
  }
  output.push(c);
  ```
  A `usize` index into a slice is the Rust way to walk a C pointer.
'''

old = '''- The contract's methods need `()`: `item.child()` not `item.child`.

Reply with exactly ONE ```rust code block containing ONLY this function."""'''

new = '''- The contract's methods need `()`: `item.child()` not `item.child`.
''' + WALK_EXAMPLE + '''
Reply with exactly ONE ```rust code block containing ONLY this function."""'''

assert old in t
t = t.replace(old, new)

# the same guidance in the unit-level prompt
old2 = '''- The C `hooks` struct fields exist in Rust (`hooks.allocate`, `hooks.deallocate`,
  `hooks.reallocate`), but they take and return `Vec<u8>`, not raw pointers:
  use `hooks.alloc(size)`, `hooks.dealloc(buf)` and `hooks.realloc(buf, size, keep)`.

Reply with exactly one ```rust code block containing the Rust implementations of the functions above."""'''
new2 = '''- The C `hooks` struct fields exist in Rust (`hooks.allocate`, `hooks.deallocate`,
  `hooks.reallocate`), but they take and return `Vec<u8>`, not raw pointers:
  use `hooks.alloc(size)`, `hooks.dealloc(buf)` and `hooks.realloc(buf, size, keep)`.
''' + WALK_EXAMPLE + '''
Reply with exactly one ```rust code block containing the Rust implementations of the functions above."""'''
assert old2 in t
t = t.replace(old2, new2)

# a stronger checklist entry
t = t.replace('''2. NO `unsafe`. NO raw pointers. NO `libc`, `CString`, `mem::transmute`, `std::ptr`.''',
              '''2. NO `unsafe`. NO raw pointers. NO `libc`, `CString`, `mem::transmute`, `std::ptr`.
   A C `char *` that walks a buffer becomes a `usize` index, NOT a Rust pointer.''')

open(p, "w").write(t)
print("pointer-walking idiom added to both prompts")
