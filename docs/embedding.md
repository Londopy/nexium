# Embedding Nexium in Python and C

The seam is the product (spec section 4). This is the afternoon: write one
function, ship it, call it from the project you already have.

## 1. Write and export

```
pub fn checksum(data: []u8) -> u32 export(c) {
    var h: u32 = 2166136261
    for b in data {
        h ^= b as u32
        h *%= 16777619
    }
    return h
}

pub fn dot(a: []f64, b: []f64) -> !f64 export(c) {
    if a.len != b.len { return error.InvalidInput }
    var acc: f64 = 0.0
    for x, y in a, b { acc += x * y }
    return acc
}

artifact cabi   { name = "ropesim" }
artifact python { name = "ropesim", version = "0.1.0" }
```

Exportable parameter types: integers, floats, `bool`, `char`, slices of those
(`[]f64` arrives as a pointer and a length), and `layout(c)` structs of
scalars. Return types: scalars, `layout(c)` structs, `void`, or an error union
of one of those. Slices cannot be returned (their lifetime cannot be expressed
across the boundary); write into a caller-provided `[]mut T` instead.

## 2. Ship

```
nx ship ropesim.nx                 # host target
nx ship ropesim.nx --target x86_64-linux-gnu
```

Output in `nx-out/ropesim/`:

| file | purpose |
| --- | --- |
| `ropesim.h` | C header with one prototype per export |
| `ropesim.dll` / `libropesim.so` / `libropesim.dylib` | shared library |
| `ropesim.lib` / `libropesim.a` | static library |
| `ropesim.c` | the generated C, for inspection |
| `python/ropesim/` | importable package (`__init__.py`, `.pyi` stubs, the shared library) |
| `ropesim-0.1.0-py3-none-<platform>.whl` | installable wheel |

## 3. The C ABI

The effect signature decides the shape (S3: a panic never crosses the
boundary):

```c
/* effects: none; cannot panic, so it returns its value directly */
uint32_t checksum(const uint8_t* data, size_t data_len);

/* effects: none */
int32_t dot(const double* a, size_t a_len, const double* b, size_t b_len, double* out);

/* effects: panics */
int32_t divide(int64_t a, int64_t b, int64_t* out);

const char* ropesim_error_name(int32_t code);
const char* ropesim_last_panic(void);
```

- A function that returns a plain value and is proven `!panics` has a direct
  signature.
- Every other export returns an `int32_t` status: `0` on success, otherwise
  an error code. `ropesim_error_name(code)` gives the error's name. A Nexium
  panic inside the call is caught at the boundary and reported as the code
  named `"Panic"`; `ropesim_last_panic()` returns the message and location
  for the current thread.
- No initialization is required (S1); each call builds its context on the
  stack. No process-global state is created (S2): the compiler rejects an
  export that reaches a mutable global when an embeddable artifact is
  declared.
- Symbols are not mangled (S4). `[]T` becomes `(const T*, size_t)`;
  `[]mut T` becomes `(T*, size_t)`.

Link statically (`ropesim.lib` / `libropesim.a`) or load the shared library.
The static library has no runtime dependency beyond libc and libm.

## 4. From Python

```python
import array, ropesim

ropesim.checksum(b"hello")                    # 1335831723
pos = array.array("d", [0.1, 0.2, 0.3])
ropesim.simulate(pos, 0.01, 100)              # []mut f64 writes back into `pos`
ropesim.dot([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]) # 32.0

try:
    ropesim.dot([1.0], [1.0, 2.0])
except ropesim.NexiumError as e:
    print(e.name, e.code)                     # InvalidInput 9

try:
    ropesim.divide(1, 0)
except ropesim.NexiumPanic as e:
    print(e.message)                          # division by zero (at ropesim.nx:31)
```

The package uses `ctypes` over the shared library: one wheel per platform,
any Python 3.8+ interpreter (the "stable ABI" default of spec 4.2). A
read-only slice parameter accepts any buffer (`bytes`, `bytearray`,
`array.array`, `memoryview`, a NumPy array), by pointer when the item
format matches and by copy otherwise, or any sequence, by copy. A `[]mut T`
parameter accepts only a writable, contiguous buffer of the matching item
format (`bytearray`, `array.array`, a writable `memoryview`, a NumPy
array) and writes through to it; a list, a read-only buffer, a
differently-typed array or a strided view is a `TypeError`, because the
writes would otherwise be lost in a copy. `layout(c)` structs become
`ctypes.Structure` subclasses with the same field names. Type stubs
(`__init__.pyi`) are included for editors.

Install the wheel with `pip install nx-out/ropesim/ropesim-0.1.0-py3-none-*.whl`,
or add `nx-out/ropesim/python` to `sys.path` during development.

## 5. From Rust

Declare `artifact rustlib { name = "ropesim" }` and `nx ship` writes a crate
to `nx-out/ropesim/rust/` with the static archive in `lib/`, a `build.rs`
that links it, `extern "C"` declarations, `#[repr(C)]` structs, and safe
wrappers:

```rust
// Cargo.toml: ropesim = { path = "nx-out/ropesim/rust" }
let mut pos = [0.1, 0.2, 0.3];
ropesim::simulate(&mut pos, 0.01, 100);          // &mut [f64] -> pointer and length
ropesim::dot(&[1.0, 2.0], &[3.0, 4.0]);          // Result<f64, NexiumError>
ropesim::divide(1, 0);                           // Err(NexiumError::Panic { message })
```

Slices become `&[T]` / `&mut [T]`, status-returning exports become
`Result<T, NexiumError>` with `Error { code, name }` and `Panic { message }`
variants, and `!panics` exports are plain functions. On Windows the archive is
compiled for the MSVC ABI so it links into the default Rust toolchain.
