# How Nexium works

This is the guided tour of the compiler for people who want to understand or
change it. The language reference is [`language.md`](language.md); calling
Nexium from other languages is [`embedding.md`](embedding.md); every judgment
call made where the specification was open is in
[`DECISIONS.md`](../DECISIONS.md).

## One compiler, written in itself

The compiler is written in Nexium under `self/`: `lexer.nx`, `parser.nx`,
the checker (`check.nx` and the `check_*.nx` modules beside it, with
`cimport.nx`), `cgen.nx` and the driver `nx.nx`, one stage each below, then
the tools (`fmt.nx`, `doc.nx`, `tools.nx`, `size.nx`, `manifest.nx`,
`ship.nx`, `ship_node.nx`, `installer.nx`, `lsp.nx`, `repl.nx`, `topo.nx`,
`completions.nx`). It builds itself from the C seed in `bootstrap/`
(decision 90): the first compiler, in Rust, drove the port and left at 1.0.

## The one-paragraph version

`nx` reads `.nx` source, checks it, and writes one C file. That C file includes `runtime/nx_rt.h` (900 lines of
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

### 1. Lexer (`self/lexer.nx`)

Bytes in, tokens out. Two decisions here shape everything downstream:

- **Newlines are tokens.** A statement ends at a newline, so the lexer emits
  `Newline` and collapses runs of them. Inside `(` and `[` newlines are
  suppressed, which is what lets an argument list span lines. Inside `{ }`
  they are kept, because a block's statements are separated by them.
- **`<<` and `>>` are `LtLt` and `GtGt`, never shift operators.** The parser
  decides from context whether they open a binary pattern or shift bits.

Identifiers, keywords, and the names of builtins are all `Ident`. Keywords are
recognised by the parser (`KEYWORDS` is the list). Non-ASCII outside strings
and comments is an error. On request the lexer keeps `//` comments as tokens;
the formatter needs them, the parser never sees them.

### 2. Parser (`self/parser.nx`)

Recursive descent, one function per grammar rule, producing a `Tree`: an
id-arena of `Node`s (kind, children, a name, a text, three slots `x`, `y`,
`z`) with the module's items on top: functions, structs, enums, traits,
impls, constants, globals, imports, tests, artifacts. Doc comments are kept
by index into `Tree.docs`. Conditions are bare (`if c {`; a struct literal
there needs parentheses), struct literals are `Point{ .x = 1 }`, anonymous ones are `.{ .x = 1 }`, captures on
closures are explicit `|[x, &mut y] a: i32|`.

The parser is where the continuation rules live: a line that starts with
`|>`, `.method(`, `catch`, `orelse`, `and`, or `or` joins the previous line,
and so does a line ending in a binary operator or an open bracket.

### 3. Checker (`self/check.nx` and `self/check_*.nx`)

The largest part of the compiler, about 15,000 lines. It turns the syntax
tree into the typed IR (`Tir`, another id-arena, `TKind` per node) and
produces every diagnostic. Its state is one struct, `Checker`, in
`check.nx`, with the core: types, definitions, loading, instances and the
passes over the whole program, the generic environment, ownership,
unification and ranges. The rest of its methods sit in modules of their
own, each an `impl Checker` block (decision 110), and `check.nx` imports
them all, which is what makes them part of the program:

| module | what it checks |
| --- | --- |
| `check_stmts.nx` | bodies, blocks, statements, loops |
| `check_views.nx` | views and their origins, the rules V1 to V5 |
| `check_exprs.nx` | expressions, function calls, `if`, casts |
| `check_fields.nx` | fields, literals, indexing, `try`, `catch`, `orelse` |
| `check_calls.nx` | methods, static members, the builtin namespaces |
| `check_match.nx` | `match`, patterns, exhaustiveness |
| `check_builtins.nx` | the `@` builtins, layout |
| `check_closures.nx` | closures |
| `check_print.nx` | the typed IR as text, for `nx tir` |
| `check_interp.nx` | the compile-time interpreter (section 4) |

The pieces:

**Types are interned** (`Types`). A type is an index into a table of `Ty`;
two types are equal exactly when their indices are equal. Inference
variables are entries too, so `let x = 0` gives `x` an integer variable that
is resolved by the first use that pins it, or defaults to `i64`.

**Generics are monomorphized.** `fn max(comptime T: type, a: T, b: T)` is
checked once per distinct `T` it is called with, producing one `Inst` per
combination (`instantiate`). There is no runtime representation of a type
parameter. Trait bounds (`where T: Ord`) are checked at instantiation, and
`impl` blocks are matched structurally.

**Ownership is a per-function flow analysis.** `List`, `String`, and `Map`
values are owned: assigning or passing one by value moves it, and the checker
marks the source local as moved (`FnCtx.moved`, snapshotted and merged per
branch). Using it again is the `use after move` error; moving out of a
field, an element, a loop variable or an `if let` binding over a place is
rejected because the container would be left half-owned. Parameters are
borrowed, so a callee cannot move them. `ref class` values are pointers
with a reference count and copy freely; the checker only tracks that they
need `refcounts`.

**Effects are inferred by fixpoint** (`propagate_effects`). While checking a
body the checker records `own_effects`, the effects the function performs
directly, each with a *witness*: the position and a sentence explaining it
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

The lattice is eight bits (`EFF_*`): `allocates`, `refcounts`, `blocks`,
`shared_mutable`, `nondeterministic`, `panics`, `ffi`, and
`unbounded_stack`, the last from the call graph: a function on a cycle
(Tarjan's components in `mark_recursion`) or calling through a function
value or a `dyn` carries it.

**Patterns** (`check_match`) compile `match` to decision trees with
exhaustiveness checking for enums and bools. **Binary patterns** turn
`<<len:16/little, payload:len*8, rest:bytes>>` into a sequence of checked
bit reads whose sizes may depend on earlier bindings.

**Views** are tracked by their origins (`self/check_views.nx`): every local knows the storage the views it holds point into and
when they were taken, and the rules V1 to V5 of `SPEC.md` 5.6 and 5.7 are
checked at the use that would read released storage, as warnings in 1.2
(`--strict` makes them errors). Returning a slice or pointer into a local
of the function (rule R1) has been an error since 1.0.

**Trait objects** get a vtable per (trait, type) pair, generated as thunks
that adapt the receiver; `dyn Shape !allocates` is a distinct type and every
implementation coerced into it must satisfy the bound.

**`@cImport`** (`self/cimport.nx`) runs `zig cc -E` on the header, parses the
declarations that come out with a small C declaration parser, and injects a
synthetic module. The generated C uses the header's own type names, so the
header stays the single source of truth for layout.

### 4. Compile-time evaluation (`self/check_interp.nx`)

An interpreter over the typed IR, values in `CV`. Its `it_*` functions are
methods of the checker declared in a module of their own (decision 110), so
the checker calls them as its own and they read its tables directly. `comptime expr`, `const`
initialisers, `@embedFile`, and `comptime test` blocks run here during
checking. It allows pure computation, collections, and calls to Nexium
functions, forbids I/O, clocks, randomness, foreign calls, and globals, and
has a step budget so a runaway evaluation is a compile error rather than a
hang. A failing `comptime test` is reported at its `expect` line like any
other error. The same interpreter runs `nx repl` (`self/repl.nx`), where
`repl_mode` lets it talk to the world, and `nx play`, the playground's
runner, where `play_mode` refuses what a page cannot do.

### 5. C backend (`self/cgen.nx`)

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
- **`for parallel`** (`parallel_for`) extracts the body into a worker
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

One header, embedded into the compiler with `@embedFile` and pasted at the
top of every generated file. Its sections: slices, the allocator interface,
panics, the default (malloc) allocator with optional leak tracking, arenas,
the parallel-for thread pool, lists, strings, formatting, hash maps,
reference counting, binary pattern helpers, checked arithmetic, and the
platform bits (file I/O, time, process spawning) for Windows and POSIX.

Reference counting is a two-word header (`rc`, `weak`) in front of every
`ref class` object. `nx_retain` is an inlined increment in the runtime; the
release is generated per class by the backend (`drop_fn` in `cgen.nx`),
because it has to drop the object's own fields when the count reaches zero.
A `weak` reference keeps the allocation alive but not the object, and
`upgrade()` fails once `rc` hits zero.

Everything is `static inline`, so the C compiler sees the whole program at
once and unused runtime functions cost nothing.

### 7. Driver and C compiler (`self/nx.nx`)

`nx build` writes the C file into `nx-out/`, invokes `zig cc` with flags for
the build mode (`debug`, `safe`, `fast`, `small`), the target triple, and any
`artifact link` inputs, and deletes the C unless `--keep-c` is given. The same
path serves `run`, `test` (a generated test runner is the entry point),
`leaks` (adds `-DNX_LEAK_CHECK`), and `size` (adds `-ffunction-sections` and
reads the object back).

`zig cc` is the default because it cross-compiles out of the box:
`--target aarch64-linux-gnu` from a Windows machine just works. `--cc clang`
or `--cc gcc` are accepted when cross-compiling is not needed.

### 8. Shipping (`self/ship.nx`, `ship_node.nx`, `installer.nx`; `export_wrapper` in `cgen.nx`)

`nx ship` reads the `artifact` declarations and produces:

- **`cabi`**: a shared library, a static archive, and a C header.
- **`python`**: a ctypes-based package and a wheel.
- **`rustlib`**: a Cargo crate with a build script that links the archive,
  `extern "C"` declarations, `#[repr(C)]` structs, and safe wrappers.
- **`node`**: an npm package over the shared library.
- **`cli`**: an executable, and **`installer`**: an Inno Setup script or
  an install script with the program's files.

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
| `nx effects` | each instance's `effects` after propagation |
| `nx refcounts` | every `Retain`/`Release`/`Weak`/`Upgrade` node, with its function |
| `nx audit` | `unsafe` blocks and globals |
| `nx doc` | doc comments, signatures, and effects, rendered to HTML |
| `nx lsp` | diagnostics from a full check on every edit, hover and code lenses (the panic proof from the recorded witness) from the checked instance; definition, completion and rename from the token stream and parsed modules (`self/lsp.nx`), so they answer while the code has errors |
| `nx repl` | the interpreter, line by line, over a program that is re-checked whole |
| `nx size` | section sizes of the object file mapped back to declarations |
| `nx layout` | the checker's `size_of`/`align_of` walked field by field: offsets, padding, the total, and the reordering by alignment that would shrink a struct |
| `nx fmt` | the token stream only; it never joins or splits lines |

## Self-hosting

The compiler is written in Nexium under `self/`, one file per stage, and
builds itself: `bootstrap/nx.c` is the C it emits for itself, any C compiler
turns that into `nx0`, `nx0` builds `self/nx.nx` into `nx1`, and `nx1` must
emit the same C again (`nx2`), which is the compiler under test and the one
releases ship (`bootstrap/build.sh`, `bootstrap/README.md`). The syntax tree
and the typed IR are id-arenas, nodes in a `List` referring to each other by
index (see `examples/tree.nx`), which needs no recursive types and frees in
one release. Every stage was validated during the port by diffing its output
against the first compiler on every source in the tree; the test harness
that now drives the suites is itself a Nexium program, `tests/run.nx`.

## Where to look when something goes wrong

| symptom | start here |
| --- | --- |
| a program parses but should not, or the reverse | `self/parser.nx`, then `tests/compile_fail/` for the expected message |
| a type error that seems wrong | `check_expr` in `self/check_exprs.nx`, `check_method_call` in `self/check_calls.nx` |
| an effect that should or should not be there | the witness in `own_effects`; grep `add_effect` in `self/check*.nx` |
| a leak in `nx leaks` | the scope stack in `self/cgen.nx` (`register_drop`); every owned temporary must be registered |
| generated C that does not compile | `nx emit-c file.nx --keep-c` and read `nx-out/file.c`; the runtime helper it calls is in `runtime/nx_rt.h` |
| a crash inside `for parallel` or `using arena` | `parallel_for` in `self/cgen.nx` and the arena section of the runtime |
