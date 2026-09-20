# Shipping a library

The one-line summary of why Nexium exists: write a function once, and
call it from the project you already have, whatever that project is
written in. `nx ship` turns a file into a C library with a header, a
Python package and wheel, a Rust crate, an npm package, a command line tool
with an installer, whichever of those the file asks for.

{{include topo/code/hasher.nx}}

Run it and it is a program:

{{output topo/code/hasher.expected}}

Ship it and it is a library:

```bash
$ nx ship topo/code/hasher.nx
shipped 5 artifact file(s) for x86_64-linux:
  nx-out/hasher/libhasher.so
  nx-out/hasher/libhasher.a
  nx-out/hasher/hasher.h
  nx-out/hasher/hasher-0.1.0-py3-none-linux_x86_64.whl
  nx-out/hasher/node/package.json
```

## `export(c)`

`export(c)` on a function puts it on the boundary with the C calling
convention. The parameters and results that can cross are integers, floats,
`bool`, `char`, slices of those (a slice arrives as a pointer and a length),
and `layout(c)` structs of scalars; a result may also be `void` or an error
union of one of those. Slices cannot be returned, since nothing on the other
side knows when to free them; write into a caller-provided `[]mut T`
instead.

## The header, and what effects buy

```c
/* FNV-1a over the bytes: the same answer in C, Python and Node. */
/* effects: none; cannot panic, so it returns its value directly */
uint32_t checksum(const uint8_t* data, size_t data_len);

/* The mean of a slice of doubles; `!panics` is proven, so the C signature is the natural one. */
/* effects: none; cannot panic, so it returns its value directly */
double mean(const double* xs, size_t xs_len);

/* A function that can fail crosses the boundary as a status code plus an out-parameter. */
/* effects: panics */
int32_t divide(int64_t a, int64_t b, int64_t* out);
```

The doc comments travel with the functions, and so does the effect system
(chapter 15). `checksum` and `mean` are proven not to panic, so they get
their natural C signatures. `divide` can fail (the error union) and can
panic (`a / b` with `b == -1` and `a` at the minimum overflows), so it
returns a status: `0` for success with the value in `*out`, otherwise an
error code that `hasher_error_name(code)` turns into `"DivideByZero"` or
`"Panic"`, with `hasher_last_panic()` holding the message. A Nexium panic
inside a library never aborts the host process; it comes back as a code.
That is the promise the specification calls S3, and the effect system is
how it is kept without a runtime check on every call.

## From Python

```python
import hasher
hasher.checksum(b"hello")            # 1335831723
hasher.mean([1.0, 2.0, 6.0])         # 3.0
hasher.divide(84, 2)                 # 42
hasher.divide(1, 0)                  # raises hasher.NexiumError: DivideByZero
```

`pip install nx-out/hasher/hasher-*.whl`, or add `nx-out/hasher/python` to
`sys.path` while developing. The package is `ctypes` over the shared
library with `.pyi` stubs, so an editor completes it; a status code becomes
an exception, a panic a `hasher.NexiumPanic` with the message; a `[]f64`
parameter takes a list, an `array('d')` or a NumPy array without copying.

## From C, Rust and Node

C: include the header, link the static or shared library. Rust: `artifact
rustlib { name = "hasher" }` adds a crate with a build script that links the
archive, `#[repr(C)]` structs and safe wrappers (`hasher::divide(1, 0)` is a
`Result`). Node: `artifact node { name = "hasher" }` is an npm package over
the shared library through `koffi`; a status becomes a thrown error with
`errorName`, `Float64Array` and `BigInt` cross as you would hope. The
[embedding guide](../docs/embedding.html) has each of these in full.

## Programs and installers

```nexium
artifact cli { name = "todo" }
artifact installer { name = "Todo", publisher = "You", version = "1.0.0", license = "LICENSE", files = ["assets"], add_to_path = true }
```

`artifact cli` names the executable `nx ship` builds; `artifact installer`
adds an Inno Setup script (and runs it, when Inno Setup is installed) on
Windows, and an `install.sh` with a tarball elsewhere. The to-do list of
chapter 11 becomes a proper tool with those two lines.

## Cross-compiling and releasing

`nx ship file.nx --target aarch64-linux-gnu` builds for another platform
from the machine you are on, because `zig cc` carries every target's C
library. The [releasing guide](../docs/releasing-your-program.html) has a
GitHub Actions template that builds a program for Windows, Linux and macOS
from a tag and attaches the artifacts to a release; it is the workflow this
compiler's own releases use.

## Rules of the boundary

- No initialization call is needed; each exported call sets up what it
  needs and tears it down.
- A program that declares an embeddable artifact may not have mutable
  globals, so two hosts loading the same library cannot interfere.
- The header's declarations are stable across minor versions of Nexium
  for a given source ([stability](../docs/stability.html)).

Next: [a neural network](21-project-neural-network.html).
