# Decisions made while building the first compiler

This log records the calls made during the initial implementation that the
specification left open, or where the implementation deliberately narrows the
design for a first release. Each one is reviewable; none is load-bearing for
the architecture. "Spec" means `nexium-spec.txt`; "archived" means
`nexium-systems-spec.txt` sections 4 through 9.

## Toolchain and backend

1. **Compiler written in Rust, C emitted as the backend, compiled with `zig cc`.**
   The archived design (section 19) picks C emission for portability; `zig cc`
   gives a C compiler, a static archiver, and cross-compilation to any target
   with no toolchain install (spec section 9). `--cc` overrides the compiler.
2. **One translation unit per build.** The runtime header is embedded into the
   generated C. No separate runtime library to link, satisfying S6.
3. **Hidden context parameter.** Every internal Nexium function takes
   `nx_ctx*` (allocator, RNG state, argv, stdio). This is the "implicit
   context" of the archived section 15. Exported functions create a context
   on the stack per call, so S1 (no initialization) holds. `for parallel`
   would need an executor in this context and is not implemented yet.
4. **Thread-local panic boundary.** The only static in the runtime is a
   `_Thread_local` pointer to the current panic boundary. Two Nexium
   libraries in one process each carry their own copy (S2).

## Syntax choices where the two documents were silent

5. **Struct literals are `Point{ .x = 1 }` and anonymous `.{ .x = 1 }`**,
   following the archived 4.1 example. Enum variants are `.Variant(args)` with
   the type inferred, or `Enum.Variant(args)`.
6. **Conditions take parentheses** (`if (x)`, `while (x)`, `for (xs) |x|`),
   matching every example in the archived document. This also makes `Name{`
   unambiguous.
7. **Optionals**: `null` is the empty value, `.?` unwraps (panics),
   `x orelse d`, `if (x) |v| { }`. A bare binding in a `match` on an optional
   binds the payload; `null` matches the empty case.
8. **Numeric casts are `value as T`**, checked at runtime when narrowing
   (contributes `panics` unless the range analysis proves it). `@truncate(T, x)`
   wraps. Distinct types convert with `as` in both directions.
9. **Logical operators are `and`, `or`, `!`.** Bitwise are `& | ^ ~ << >>`.
10. **Compile-time builtins use `@`:** `@typeName`, `@sizeOf`, `@truncate`,
    `@errorName`, `@embedFile`, `@weak`, `@refCount`.
11. **`impl Type { }` and `impl Trait for Type { }`** declare methods; the
    receiver is the first parameter named `self` with type `Self`, `*Self`,
    or `*mut Self`. Generic impls are `impl(T) Pair(T) { }`.
12. **Statement termination** is the newline. A line beginning with `|>`,
    `.method(`, `catch`, `orelse`, `and`, or `or` continues the previous
    expression; a line ending with a binary operator continues too.
13. **Error sets** are declared `error Name { A, B }`; `error.Name` values
    belong to the global set unless a set is named in the return type
    (`ParseError!T`).

## Semantics narrowed for the first release

14. **Pointers `*T`/`*mut T` are safe references** created with `&x`/`&mut x`
    and dereferenced with `.*`. The archived section 12 lists raw pointer
    dereference as unsafe; here only pointer casts, integer/pointer casts,
    `.ptr` of a slice, foreign calls, and mutable globals require `unsafe`.
    Reason: `self: *mut Self` receivers would otherwise force `unsafe` into
    every method.
15. **Regions are checked conservatively.** Rule R1 is enforced for the case
    that matters most: a function may not return a slice or pointer into one
    of its own locals (arrays, Lists, Strings, structs); parameters are
    exempt because their storage belongs to the caller. Views stored into
    outer variables are not tracked (archived O1/O2 leave the notation open).
    Exported functions may not return slices at all.
16. **Ownership of collections** (`List`, `String`, `Map`, and structs that
    contain them): `let y = x` moves; a later use of `x` is a compile error;
    moving out of a field or element is an error (`.clone()` instead);
    parameters are borrowed, so moving a parameter into a new owner is an
    error. Owned values are released at scope exit; that release is the only
    automatic action at scope exit (H7). Codegen zeroes moved-from locals so
    the release is always safe.
17. **`ref class` values copy by retaining**; `?Node` and `weak Node` too.
    Passing to a function does not retain (borrow). Cycles leak, as the spec
    says; `weak` breaks them.
18. **Effects are inferred for every function** (not only within a module);
    negative bounds are enforced with diagnostics naming the introducing
    site. Calling through a function value acquires every effect its type
    permits (all of them unless the type says `fn(...) !allocates`).
19. **`panics` range analysis** follows E6's five sources: type ranges,
    literals, dominating `if` guards on locals, `for` iteration facts
    (`for (xs) |x, i|` proves `xs[i]`; `for (0..xs.len) |i|` proves `xs[i]`
    on the same local), and constant loop bounds. Anything else contributes
    `panics`.
20. **Records**: a literal with compile-time-known field values is checked
    at compile time (a violation is a compile error); a literal with runtime
    values panics if violated; `Record.new(.{ ... })` returns
    `error.InvalidRecord` instead.
21. **`println` has the `blocks` effect** (archived section 15: reaching for
    stdout blocks) and does not allocate. `format` allocates and returns a
    `String`.
22. **The export ABI depends on the effect signature.** An export that returns
    a plain value and is proven `!panics` gets a direct C signature. Any
    other export returns `int32_t` status and writes its value through an out
    pointer; a panic becomes the `Panic` status (S3). The Python wrapper
    raises `NexiumError` / `NexiumPanic`.
23. **Python artifact uses ctypes** over the shared library rather than a
    CPython extension module; this is the "stable ABI by default" of 4.2 taken
    literally (one wheel per platform, any interpreter version). `abi = native`
    is accepted and ignored for now.
24. **Test blocks return `!void`** so `try` works inside them. `nx test`
    compiles a runner; `comptime test` is parsed but runs at runtime for now.
25. **Integer literals default to `i64`, floats to `f64`** when nothing
    constrains them. `for (0..n)` takes the type of `n`.
26. **`main` may return `void`, `!void`, or an integer exit code.** An error
    returned from `main` prints `error: Name` and exits 1; a panic prints the
    location and exits 101.
27. **Not implemented in this release** (reported as errors or notes rather
    than silently ignored): `soa` layout, `packed` layout, `node` and
    `installer` artifacts, `nx publish`, a registry, and the `pool`/`stack`
    allocation strategies. Everything else listed in the first draft of this
    item now exists; see items 31 to 44.

## Second pass: the gaps

31. **`comptime test`** runs in the compile-time interpreter during `nx check`;
    a failing expectation is a compile error at the `expect` line.
32. **`nx refcounts`** walks the typed IR and lists every retain, release,
    weak creation, and upgrade site with the function it is in.
33. **`nx leaks`** runs the program in debug mode with a tracking allocator
    whose counters live in the context (no globals); a nonzero live count at
    exit is reported with total and peak bytes. Every example is leak-free.
34. **`using arena { ... }`** installs a bump allocator for the block and
    frees it all at once at the end. Every `List`, `String`, and `Map`
    remembers the arena it was created in (one pointer per container), so a
    container created outside the block keeps its storage on the heap even
    when it grows inside the block, and the arena forwards frees and reallocs
    of memory it does not own to the parent allocator. What is still not
    checked: a value *created inside* the block that is stored outside it
    (same status as regions).
35. **`nx fmt`** is token based and line preserving: indentation, spacing,
    trailing whitespace, and blank-line runs are canonical; lines are never
    joined or split. `cargo test` checks that every example is canonical.
36. **`nx doc`** emits one HTML page per program with signatures, fields, and
    the inferred effect set of each public function.
37. **`nx size`** compiles with one section per function and reads section
    sizes back from the object file (ELF and COFF; Mach-O is reported as
    unsupported), attributing bytes to declarations, glue, and runtime.
38. **`dyn Trait`** is a fat pointer `{data, vtable}` created by coercing
    `*T`/`*mut T`; vtables are generated per (trait, type) with thunks that
    adapt the receiver. `dyn Trait !allocates` checks every implementation's
    inferred effects at the coercion site. `Self` may only appear in receiver
    position for a trait used as an object.
39. **`for parallel (items) |x, i| { }`** extracts the body into a worker
    function that reaches outer locals through an environment of pointers,
    and the runtime splits the index range across hardware threads. The
    checker rejects `shared_mutable` in the body (directly or through calls),
    and rejects `return` and `break` inside it; a panic in a worker is
    re-raised in the caller after all workers finish. Data races through
    captured mutable locals are the programmer's responsibility, exactly as
    the spec's "free of `shared_mutable`" rule implies.
40. **`@cImport("header.h")`** runs the C compiler's preprocessor and reads
    functions (including variadics), typedefs, structs of scalars/pointers/
    arrays, enums, and integer/float/string macros. `const T*` maps to `*T`,
    other pointers to `*mut T`, `void*` to `*mut u8`, C `long` to i32 on
    Windows and i64 elsewhere. A struct with fields that cannot be translated
    (function pointers, bit-fields, nested definitions) becomes an opaque type
    usable through pointers, because that is how Apple's `FILE` is defined
    and every stdio function takes `FILE*`. Unsupported declarations (unions,
    function-pointer typedefs, function-like macros such as `stdout`) are
    imported as names that explain themselves when used. Imported structs keep their C
    spelling in the generated code, so the header stays the single source of
    truth for layout. `@cstr("...")` gives a NUL-terminated `*u8`.
41. **Vendored C** is declared with `artifact link { c_sources = [...],
    libs = [...], include = [...], lib_paths = [...] }` (spec 17.1), and the
    same is available as `--c-source`, `--link`, `--link-path`, `-I`.
42. **`rustlib` artifact**: a Cargo crate with a build script linking the
    static archive, `extern "C"` declarations, `#[repr(C)]` structs, and safe
    wrappers returning `Result<T, NexiumError>` (with a `Panic` variant). On
    Windows the static archive is compiled for the MSVC ABI so it links into
    Rust and MSVC programs.
43. **`nx lsp`** is a stdio language server with diagnostics on open/change/
    save and hover over functions (signature plus inferred effects). It has no
    dependencies; JSON handling is in-tree. When a file has errors, hover
    falls back to the parsed signature without effects.
44. **Region rule R1** is enforced conservatively (item 15).

## Toward self-hosting

45. **`process.run(argv) -> !i32`** spawns without a shell (CreateProcess on
    Windows with CommandLineToArgv-compatible quoting, `posix_spawnp`
    elsewhere), waits, and returns the exit code. stdout/stderr are flushed
    first so parent and child output interleave deterministically. A signal
    death reports as 128 plus the signal number.
46. **`nx tokens`** is the lexer oracle: `start end KIND [payload]` per token,
    with integers in decimal, floats as their source text, strings and byte
    strings as hex of the unescaped bytes, chars as code points. The format is
    frozen because `self/lexer.nx` reproduces it.
47. **The compiler runs on a 512 MB thread.** A long else-if chain in
    `self/lexer.nx` overflowed the default main-thread stack in the checker;
    tree-walking compilers recurse, so the work moves to a big-stack thread
    instead of rewriting every walk iteratively.
48. **Program output is binary on Windows** (`_setmode(_O_BINARY)` on stdout
    and stderr) so the same program prints the same bytes on every platform.
    The oracle diff depends on it.
49. **The AST will be an id-arena.** Nodes live in a `List` and refer to each
    other by index (`examples/tree.nx`), which needs no recursive types, no
    pointers, and frees in one release. Language gaps found writing the lexer,
    to close before the parser: parameters cannot be taken by value (moving a
    `String` into a struct field costs a `.clone()`), there is no raw-byte
    push on `String` (`append_char` encodes), and `return` is a statement, so
    `orelse return` is not available. All three are closed in items 61 to 63.

## nexium-gui

50. **The GUI is immediate mode over a software framebuffer.** Widgets are
    function calls that return what happened this frame, state lives in the
    caller's variables (passed as `*mut`), and drawing is Nexium code writing
    into a `List(u32)`. No retained widget tree, no callbacks, no closures: the
    style that suits a language with explicit ownership, and the one egui
    proved. The cost is CPU rendering, fine at 640x440 and 60 Hz.
51. **The platform layer is the only C**, about 200 lines: open a window, pump
    events into a queue, blit a framebuffer, sleep, time. Win32 today; other
    platforms compile a stub whose `gp_open` returns 0, so the library, its
    tests, and offscreen rendering work everywhere and only the window is
    missing. X11 and Cocoa backends are the same eight functions.
52. **Text is a bitmap font embedded with `@embedFile`**, generated once from
    Pillow's own free bitmap font into `gui/font.bin` (8x13 cells, ASCII).
    Non-ASCII is skipped. A vector font is a later problem.
53. **Widget identity is call order.** Each widget takes the next id in the
    frame; hot, active, and focus are ids. Conditional widgets shift the ids
    of everything after them, which is the standard immediate-mode trade-off.
54. **`libs_windows`/`libs_linux`/`libs_macos`** exist because zig's gnu
    target links user32 but not gdi32 by default, `#pragma comment(lib)` is
    ignored there, and a plain `libs` entry would break the Linux build.

## Editors and releases

55. **One TextMate grammar, two spellings.** VS Code takes JSON and Sublime
    takes YAML with the same regexes and scope names, kept in step by hand;
    a generator would be more machinery than the two files. Tree-sitter, which
    Neovim, Helix, and Zed want, is a separate grammar and a later job.
56. **The extension is thin.** Highlighting is the grammar; everything
    semantic comes from `nx lsp`, so the editor never disagrees with the
    compiler and the extension has one dependency (the LSP client library).
57. **GitHub highlighting borrows Zig's grammar** through `.gitattributes`
    until Linguist accepts Nexium, which requires usage in public
    repositories first. Zig's syntax is the closest match.
58. **Installers stay outside `nx`.** `.dmg`, `.msi`, and `.deb` are made by
    the platform tools in CI from the binary `nx build` produced; the
    release template carries those jobs, off by default. The spec's
    `installer` artifact remains unimplemented on purpose.

## Translations

59. **Translations live in `docs/i18n/<lang>/` and mirror the English
    files by name.** English is the source of truth; a translated page that
    falls behind is still linked, and its header switcher always offers the
    English original. Code, command names, error messages, and identifiers
    are never translated, because they are what the reader will type and
    what the compiler will print; only the comments inside code blocks are. The README is translated into six
    languages; the two documents people read first, the language reference and
    the architecture tour, into three. The rest follows as the English text
    settles.

## Language gaps closed before the parser

61. **`own` parameters.** `fn f(own s: String)` takes ownership: the caller's
    argument is moved (a local is zeroed after the copy so its scope-exit
    drop is a no-op; a temporary is handed over without a caller-side drop),
    and the callee owns it: mutable, movable into a struct or another call,
    dropped at return otherwise. `own` is a contextual modifier, only
    recognized before a parameter name. It is rejected on receivers, on
    copy types (where it would mean nothing), on exported functions (the
    host cannot hand over ownership), and a function with `own` parameters
    cannot become a function value because `fn(String)` says nothing about
    ownership. The spec is silent on the syntax; `own` reads as the
    counterpart of "borrowed" and does not collide with any identifier in
    the examples.
62. **Moves are branch-aware.** The moved-set is snapshotted before an `if`
    or `match`, each branch or arm starts from that snapshot, and afterwards
    the union counts as moved. Before this, moving a String in the `then`
    branch made it unusable in `else`, which the lexer hit at once.
63. **Jumps in expression position, narrowly.** `return`, `break`, and
    `continue` are still statements, but the right-hand side of `orelse` and
    `catch` accepts one (wrapped as a diverging block), and an unbraced `if`
    body or `else` body is parsed as a statement, so assignments and jumps
    work there without braces. `String.push_byte(u8)` appends a raw byte;
    `append_char` keeps encoding code points. Integer and float literals
    coerce into `?T`.

## The standard library

64. **`std/` is Nexium source embedded in the compiler.** `import std.strings`
    loads `std/strings.nx` from the `nx` binary the way the runtime header is
    embedded: nothing to install, one artifact, and the library is versioned
    with the compiler that compiles it. Std modules load after the program's
    own files so the root stays module 0.
65. **What stays builtin.** `List`, `String`, `Map`, slices, formatting, and
    the platform namespaces need the compiler for ownership, effects, and the
    C runtime, and stay in Rust. Everything that is plain Nexium over those
    (`strings`, `lists`, `bytes`, `num` today) lives in `std/`, readable and
    testable with `nx test`. Module names never collide with the builtin
    namespaces, so `import std.math` still means the builtin.
66. **Tuple types are legal type arguments** (`List((A, B))`), which
    `lists.zip` needed. A tuple literal in type-argument position is read as
    a tuple type.

67. **Recursive types through `List`.** `enum Json { Arr(List(Json)) }` is
    legal: a List holds a pointer, so the C backend forward-declares a struct
    or enum reached through a List or slice and defines it after the current
    type. Definitions that are only ever forward-declared are emitted before
    the functions. This is what `std.json` needed.
68. **Matching through a pointer binds owning payloads by reference.** In
    `match p.* { .Arr(items) => ... }` with `p: *mut Json`, `items` is a
    `*mut List(Json)` aliasing the payload, so appending mutates the value in
    place and returning `&items[i]` is a pointer into the caller's data, not
    into a local. Scalars are still copied. The rule is only active for
    `match p.*`; a match on a value binds values as before.
69. **`fn string(...)` may not become `nx_string`.** User function names are
    mangled with `nx_`, which the runtime also uses; the backend appends `_fn`
    to any user name that collides with a runtime identifier instead of
    reserving the names in the language. Struct and enum names outside the
    root module are prefixed with their module index for the same reason:
    `std.json` and `std.args` each have a `Parser`.

## Installers

70. **The Windows installer bundles Zig.** Python's installer ships its
    runtime; Nexium's ships its C toolchain, so "install and run" needs
    nothing else. `nx` looks for `zig` next to itself before the PATH, so
    the bundled copy wins without configuration, and the component can be
    unchecked by people who keep their own Zig. Inno Setup was chosen over
    WiX because it gives the license, overview, components, tasks, and
    finish pages with no XML and is preinstalled on GitHub's runners.
71. **macOS and Linux get a script, not a package.** `install.sh` verifies
    the checksum, installs under `~/.nexium`, and edits the shell startup
    files; `.pkg` and `.deb` would need signing or repositories to be worth
    more than that. Zig is downloaded only on Linux when no compiler exists,
    because on macOS the Xcode tools are the compiler that works.
72. **Zig stays required by the design, not by the user.** As long as C is
    the backend, a C compiler is needed to build programs; the installers
    make that invisible. Dropping the requirement entirely means a native
    backend, which is self-hosting stage 4 territory and not planned yet.

## The interactive session

73. **The REPL is the compile-time interpreter with a persistent program.**
    Each line is appended to a synthetic module and the whole thing is
    re-checked, so the prompt and the compiler can never disagree; only the
    new statements run, against values kept by name from earlier lines. The
    interpreter is allowed I/O in this mode and nowhere else. Re-checking
    everything per line is cheap at REPL scale and simpler than an
    incremental checker; values are plain data, so they survive the type
    table being rebuilt. Compile-only features (`@cImport`, `for parallel`,
    `using arena`) are reported, not emulated.

74. **Interpreter pointers name a call frame.** A `Value::Ptr` is
    (frame, local, path): the frame index says which suspended caller's
    environment holds the local, so a pointer passed into a function still
    points at the caller's variable, and `match p.*` binds owning payloads
    as pointers into that place, exactly as the compiled program does. Paths
    step into enum payloads, optionals and error unions (index 0) as well as
    fields and elements. `Map` is an insertion-ordered list of pairs in the
    interpreter; it only needs to be correct, not fast. When a statement
    cannot be evaluated the message names the source position of the first
    expression that failed, because "cannot be evaluated" without a location
    was the most common dead end in the REPL.

## Compiler selection

60. **Native macOS builds use the system compiler.** zig 0.14.1 cannot read
    the libSystem text stubs shipped with Xcode 16.3 and later, so `zig cc`
    on a current Mac links a program with no libc at all (`_puts` undefined
    in a hello world). Rather than pin CI to an old runner and leave users
    stuck, `nx` picks `cc` for native macOS builds and `zig cc` everywhere
    else and for every cross-compile. `--cc` and the `NX_CC` environment
    variable override both.

## Repository

28. License: MIT, copyright Londopy.
29. Changelog follows Keep a Changelog and is validated in CI with
    `patchnotes` (strict mode).
30. CI runs on Windows, Linux, and macOS: `cargo test` builds the compiler,
    runs every example against its recorded output, and checks the
    compile-fail cases. Zig is installed in CI to provide the C compiler.
