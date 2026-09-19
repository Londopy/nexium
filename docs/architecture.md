# How Nexium works

This is the guided tour of the compiler for people who want to understand or
change it. The language reference is [`language.md`](language.md); calling
Nexium from other languages is [`embedding.md`](embedding.md); every judgment
call made where the specification was open is in
[`DECISIONS.md`](../DECISIONS.md).

## The one-paragraph version

`nx` is a Rust program with no dependencies. It reads `.nx` source, checks it,
and writes one C file. That C file includes `runtime/nx_rt.h` (900 lines of
plain C embedded in the compiler binary) and is handed to `zig cc`, which is
Clang with a cross-compiling libc bundled in. There is no garbage collector,
no virtual machine, and no runtime library to install: the output is a native
executable, shared library, or static archive that depends only on the C
library of the target.

```
 hello.nx ──lex──▶ tokens ──parse──▶ AST ──check──▶ typed IR ──cgen──▶ hello.c ──zig cc──▶ hello.exe
                                            │
                                            └──comptime──▶ constant values, comptime tests
```

## The pipeline, stage by stage

### 1. Lexer (`src/lexer.rs`)

Bytes in, tokens out. Two decisions here shape everything downstream:

- **Newlines are tokens.** A statement ends at a newline, so the lexer emits
  `Newline` and collapses runs of them. Inside `(` and `[` newlines are
  suppressed, which is what lets an argument list span lines. Inside `{ }`
  they are kept, because a block's statements are separated by them.
- **`<<` and `>>` are `LtLt` and `GtGt`, never shift operators.** The parser
  decides from context whether they open a binary pattern or shift bits.

Identifiers, keywords, and the names of builtins are all `Ident`. Keywords are
recognised by the parser. Non-ASCII outside strings and comments is an error.

`nx tokens file.nx` dumps the stream in a fixed format; it is the oracle the
self-hosted lexer in `self/lexer.nx` is checked against.

### 2. Parser (`src/parser.rs`, `src/ast.rs`)

Recursive descent, one function per grammar rule, producing an `ast::Module`
of items: functions, structs, enums, traits, impls, constants, globals,
imports, tests, artifacts. Conditions are parenthesised (`if (c)`), struct
literals are `Point{ .x = 1 }`, anonymous ones are `.{ .x = 1 }`, captures on
closures are explicit `|[x, &mut y] a: i32|`.

The parser is where the continuation rules live: a line that starts with
`|>`, `.method(`, `catch`, `orelse`, `and`, or `or` joins the previous line,
and so does a line ending in a binary operator or an open bracket.

### 3. Checker (`src/check/`)

The largest part of the compiler, about 7,500 lines. It turns the AST into
the typed IR in `src/tir.rs` and produces every diagnostic. The pieces:

**Types are interned** (`src/types.rs`). A `TyId` is an index into a table
of `TyKind`; two types are equal exactly when their ids are equal. Inference
variables are `TyKind`s too, so `let x = 0` gives `x` an integer variable
that is resolved by the first use that pins it, or defaults to `i64`.

**Generics are monomorphized.** `fn max(comptime T: type, a: T, b: T)` is
checked once per distinct `T` it is called with, producing one `TFunc`
instance per combination (`instantiate` in `check/mod.rs`). There is no
runtime representation of a type parameter. Trait bounds (`where T: Ord`) are
checked at instantiation, and `impl` blocks are matched structurally.

**Ownership is a per-function flow analysis.** `List`, `String`, and `Map`
values are owned: assigning or passing one by value moves it, and the checker
marks the source local as moved (`TFunc::moved`). Using it again is the
`use after move` error; moving out of a field or an element is rejected
because the container would be left half-owned. Parameters are borrowed, so a
callee cannot move them. `ref class` values are pointers with a reference
count and copy freely; the checker only tracks that they need `refcounts`.

**Effects are inferred by fixpoint** (`check/effects.rs`). While checking a
body the checker records `own_effects`, the effects the function performs
directly, each with a *witness*: the span and a sentence explaining it
("appending to a List may grow it"). It also records every callee. Then

```
effects(f) = own(f) ∪ ⋃ effects(callee)      iterated to a fixpoint
```

over all instances. Extern functions are assumed to do everything. A negative
bound such as `!allocates` on a signature, or on a function type, is checked
after propagation; the diagnostic walks the call graph to the function that
introduced the effect and prints its witness, so the error names the exact
line at the bottom of the call chain. `panics` is discharged where the
checker can prove the operation cannot fail: indexing with the loop index of
a `for` over the same slice, comptime-known indices, arithmetic whose operand
ranges fit.

The lattice is eight bits in `src/effects.rs`: `allocates`, `refcounts`,
`blocks`, `shared_mutable`, `nondeterministic`, `panics`, `ffi`, and
`unbounded_stack` (reserved).

**Patterns** (`check/pattern.rs`) compile `match` to decision trees with
exhaustiveness checking for enums and bools. **Binary patterns**
(`check/binpat.rs`) turn `<<len:16/little, payload:len*8, rest:bytes>>` into
a sequence of checked bit reads whose sizes may depend on earlier bindings.

**Regions** are checked conservatively: returning a slice or pointer into a
local of the function is an error (rule R1).

**Trait objects** get a vtable per (trait, type) pair, generated as thunks
that adapt the receiver; `dyn Shape !allocates` is a distinct type and every
implementation coerced into it must satisfy the bound.

**`@cImport`** (`src/cimport.rs`) runs `zig cc -E` on the header, parses the
declarations that come out with a small C declaration parser, and injects a
synthetic module. The generated C uses the header's own type names, so the
header stays the single source of truth for layout.

### 4. Compile-time evaluation (`src/comptime.rs`)

An interpreter over the typed IR. `comptime expr`, `const` initialisers,
`@embedFile`, and `comptime test` blocks run here during checking. It allows
pure computation, collections, and calls to Nexium functions, forbids I/O,
clocks, randomness, foreign calls, and globals, and has a step budget so a
runaway evaluation is a compile error rather than a hang. A failing
`comptime test` is reported at its `expect` line like any other error.

### 5. C backend (`src/cgen/`)

The typed IR is lowered to one C translation unit. A few conventions explain
most of what you see in `nx emit-c` output:

- **Every function takes a hidden `nx_ctx* c`.** The context carries the
  allocator, the current arena, stdout/stderr, the RNG state, argv, and the
  leak-tracking counters. There are no globals in the runtime, which is what
  makes a shipped library safe to load into a host process.
- **Owned values are dropped at scope exit.** The backend keeps a scope stack;
  each `Let` of an owned type registers a drop, and leaving the scope (normally,
  by `return`, `break`, or an error) emits the drops in reverse order together
  with any `defer`s. Temporaries passed to calls are registered too.
- **Checked arithmetic is a call.** `x * 7` becomes
  `nx_mul_i64(x, 7, "hello.nx:3")`; the location string is what a panic
  prints. Wrapping (`*%`) and saturating (`*|`) forms map to plain C or clamped
  helpers.
- **Panics are `longjmp`.** A thread-local `nx_boundary` holds a `jmp_buf`;
  `nx_panic` fills in the message and location and jumps to the nearest
  boundary, which the entry point or an export wrapper installed.
- **`for parallel`** (`cgen/parallel.rs`) extracts the body into a worker
  function that reaches the enclosing locals through a struct of pointers,
  and the runtime splits the index range across hardware threads. A worker's
  panic is captured and re-raised in the caller after all workers finish.
- **`using arena`** swaps a bump allocator into a copy of the context for the
  block. Containers remember the arena they were created in, so an outer
  `List` that grows inside the block still lives on the heap.

This is `examples/hello.nx` after lowering:

```c
static void nx_main(nx_ctx* c) {
  nx_sink _t1 = nx_sink_file(c, c->out);
  nx_w(&_t1, (const uint8_t*)nx_str_0, 13);
  nx_w(&_t1, (const uint8_t*)"\n", 1);
  nx_sink_flush(&_t1);
  int64_t x_0 = nx_mul_i64(((int64_t)6LL), ((int64_t)7LL), "examples/hello.nx:3");
  ...
}
```

Formatting is compiled, not interpreted: `println("x = {}", .{x})` becomes
direct writes to a sink, one per placeholder, with the argument's type known.

### 6. The runtime (`runtime/nx_rt.h`)

One header, embedded into the compiler with `include_str!` and pasted at the
top of every generated file. Its sections: slices, the allocator interface,
panics, the default (malloc) allocator with optional leak tracking, arenas,
the parallel-for thread pool, lists, strings, formatting, hash maps,
reference counting, binary pattern helpers, checked arithmetic, and the
platform bits (file I/O, time, process spawning) for Windows and POSIX.

Reference counting is a two-word header (`rc`, `weak`) in front of every
`ref class` object. `nx_retain` is an inlined increment in the runtime; the
release is generated per class by the backend (`drop_fn` in `cgen/mod.rs`),
because it has to drop the object's own fields when the count reaches zero.
A `weak` reference keeps the allocation alive but not the object, and
`upgrade()` fails once `rc` hits zero.

Everything is `static inline`, so the C compiler sees the whole program at
once and unused runtime functions cost nothing.

### 7. Driver and C compiler (`src/main.rs`)

`nx build` writes the C file into `nx-out/`, invokes `zig cc` with flags for
the build mode (`debug`, `safe`, `fast`, `small`), the target triple, and any
`artifact link` inputs, and deletes the C unless `--keep-c` is given. The same
path serves `run`, `test` (a generated test runner is the entry point),
`leaks` (adds `-DNX_LEAK_CHECK`), and `size` (adds `-ffunction-sections` and
reads the object back).

`zig cc` is the default because it cross-compiles out of the box:
`--target aarch64-linux-gnu` from a Windows machine just works. `--cc clang`
or `--cc gcc` are accepted when cross-compiling is not needed.

### 8. Shipping (`src/ship.rs`, `src/cgen/exports.rs`)

`nx ship` reads the `artifact` declarations and produces:

- **`cabi`**: a shared library, a static archive, and a C header.
- **`python`**: a ctypes-based package and a wheel.
- **`rustlib`**: a Cargo crate with a build script that links the archive,
  `extern "C"` declarations, `#[repr(C)]` structs, and safe wrappers.
- **`cli`**: an executable.

The export boundary is where effects pay off. An exported function that is
proven `!panics` and does not return an error union gets its natural C
signature. Any other export returns an `int32_t` status and delivers its value
through an out-parameter; the wrapper installs a panic boundary, so a Nexium
panic inside a library becomes an error code in the host instead of an abort.
Mutable globals are rejected in any program that declares an embeddable
artifact, so two hosts loading the same library cannot interfere.

## The tools

All of them are views over the same typed IR, which is why they agree with
the compiler:

| tool | what it reads |
| --- | --- |
| `nx effects` | `TFunc::effects` after propagation |
| `nx refcounts` | every `Retain`/`Release`/`Weak`/`Upgrade` node, with its function |
| `nx audit` | `unsafe` blocks and globals |
| `nx doc` | doc comments, signatures, and effects, rendered to HTML |
| `nx lsp` | diagnostics from a full check on every edit, hover from `TFunc` |
| `nx size` | section sizes of the object file mapped back to declarations |
| `nx fmt` | the token stream only; it never joins or splits lines |

## Self-hosting

The compiler is being rewritten in Nexium under `self/`, one stage at a
time, each stage validated by diffing its output against the Rust compiler on
the same input. The lexer is done and is part of `cargo test`. The parser is
next; its AST will be an id-arena, nodes in a `List` referring to each other
by index (see `examples/tree.nx`), which needs no recursive types and frees
in one release. When all four stages exist, the Rust `nx` compiles the Nexium
`nx` once, that binary compiles its own source again, and if the two outputs
match byte for byte the language builds itself.

## Where to look when something goes wrong

| symptom | start here |
| --- | --- |
| a program parses but should not, or the reverse | `src/parser.rs`, then `tests/compile_fail/` for the expected message |
| a type error that seems wrong | `src/check/expr.rs` (expressions) or `src/check/method.rs` (method and builtin calls) |
| an effect that should or should not be there | the witness in `own_effects`; grep `add_effect` in `src/check/` |
| a leak in `nx leaks` | the scope stack in `src/cgen/expr.rs`; every owned temporary must be registered |
| generated C that does not compile | `nx emit-c file.nx --keep-c` and read `nx-out/file.c`; the runtime helper it calls is in `runtime/nx_rt.h` |
| a crash inside `for parallel` or `using arena` | `src/cgen/parallel.rs` and the arena section of the runtime |
