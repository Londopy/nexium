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
6. ~~**Conditions take parentheses** (`if (x)`, `while (x)`, `for (xs) |x|`),
   matching every example in the archived document. This also makes `Name{`
   unambiguous.~~ Superseded by 87 in 0.6.
7. **Optionals**: `null` is the empty value, `.?` unwraps (panics),
   `x orelse d`, `if let v = x { }` (was `if (x) |v| { }` before 87). A bare
   binding in a `match` on an optional binds the payload; `null` matches the
   empty case.
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
    than silently ignored): `nx publish` and a registry. `node` and
    `installer` artifacts exist since 0.5; `soa`, `packed`, `pool` and
    `stack` were removed from the language in item 88. Everything else
    listed in the first draft of this item now exists; see items 31 to 44.

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

## The standard library, phase 1

75. **File-system access is a thin set of `io` primitives plus a Nexium
    module.** The runtime knows only what needs the platform: kind, stat,
    make/remove a single directory, remove a file, rename, list one directory,
    cwd, temp dir, append. Everything else in `std.fs` (recursive create and
    remove, the walker, sorted listings, every path function) is Nexium, so
    it is portable by construction and readable by users. Primitives report
    `error.NotFound` or `error.IoError`, nothing finer, until a real program
    needs more. Paths are byte strings; both `/` and `\` separate components
    on every platform and results keep the separator the input used, because
    a tool written on Windows should produce paths its own users recognize.

76. **Dates are arithmetic; the platform only supplies an offset.** `std.time`
    breaks instants down with the civil-from-days algorithm in Nexium and
    carries a UTC offset in minutes on every `DateTime`, so UTC, fixed-offset
    and local times are one type. The one primitive, `time.utc_offset(ms)`,
    asks the C library for the local offset at that instant; there is no
    time zone database in the runtime and no zone names, because that is a
    large dependency for a rare need and `@cImport` can bring one in. The
    interpreter reports offset 0, so local time at the REPL is UTC.

77. **Regular expressions never backtrack.** `std.regex` compiles to a small
    instruction set and runs a Pike VM: every search is linear in the
    pattern times the text, so a user-supplied pattern cannot hang a
    program. The price is no back-references and byte-level matching (`.`
    is one byte); both can come later behind the same API. The engine is
    written in Nexium on purpose: it exercises recursive enums, owning
    payloads through pointers, and moves in loops, and it found two compiler
    bugs on its first run (literal field order, reassignment after a move).

78. **Text is bytes; code points are a library.** `String` and `[]u8` stay
    byte-oriented and `std.text` decodes UTF-8 on demand, so the core never
    pays for or argues about a character model. Case mapping covers only
    the alphabets whose upper/lower pairs are a fixed offset (ASCII, Latin-1,
    Latin Extended-A, Greek, Cyrillic); full Unicode tables would be larger
    than the rest of the standard library. Terminal width follows the
    wcwidth convention. A program's `nx test` runs its own tests and its
    file modules' tests, never the embedded std modules' tests, which the
    compiler's suite covers.

79. **Streams are integer handles plus a Nexium buffer.** The runtime keeps a
    small table of open `FILE*`s and hands out integers (1, 2, 3 are the
    standard streams), because a handle is trivially copyable, printable
    and needs no destructor; `std.stream` puts the 64 KB buffers, line
    splitting and flushing on top in Nexium. Nothing closes or flushes a
    Writer automatically: with no destructors in the language yet, an
    explicit `close` (or `defer w.flush()`) is the honest rule, stated in
    the module's doc comment.

## Talking to the world (phase 2)

80. **Sockets are blocking, with a timeout on every wait.** The runtime
    offers plain blocking TCP and UDP over Winsock and BSD sockets, and each
    call that can wait takes a timeout in milliseconds (0 = forever) that
    expires as `error.Timeout`. No event loop, no async: a tool or a small
    service is simplest as straight-line code, and threads (phase 2, later)
    cover concurrency when it is needed. A socket is an integer handle like
    a file, so `std.stream` buffers both with one Reader. Windows takes two
    seconds to refuse a loopback connection, which is why the tests give
    refusals a generous timeout.

81. **HTTP is a library, not a runtime feature, and it is single-threaded.**
    `std.http` is written on `std.net` and `std.stream` alone: a client that
    always sends `Connection: close` (one connection per request keeps the
    parser trivial and the behaviour obvious), and a server that answers one
    request at a time. Handlers are plain `fn(*Request) -> Response` values
    in a `Router`, so a service is a set of functions and no framework. TLS
    stays outside the standard library until a vetted C binding exists.
    Concurrency arrives with threads later in the phase; the API will not
    change when it does, because each connection is already independent.

82. **Threads, no async.** The concurrency model is blocking threads with
    channels and mutexes, and nothing else: no event loop, no colored
    functions, no `await`. The effect system already says which calls
    block, which is the information an async design would add, and a
    thread per connection or per job is what the programs this language
    targets need. The runtime primitive is deliberately untyped
    (`thread.start` takes any `fn(*mut T)` and a pointer); `std.thread`
    supplies the typed `Thread(T, R)`, `Worker(T)`, `Channel(T)` and
    `Mutex(T)` in Nexium, keeping every thread's argument in a one-element
    List so its address is stable while the Thread value moves. Refcounted
    `ref class` values are not thread-safe yet (their counts are not
    atomic); channels and mutexes are shared by pointer, and the spawner
    joins before the shared values go out of scope. Atomic counts and a
    `Mutex` as a `ref class` can come when a program needs them.

## Sharing code (phase 3)

83. **Packages are git tags first, a registry later.** A dependency is a
    repository at a tag or a directory on disk, checked out shallowly into
    `nexium_modules/` and pinned by commit in `nexium.lock`; that gives
    reproducible builds and vendoring with nothing to host. A package is a
    manifest plus `src/`; `import dep` is `src/lib.nx` so the common case
    needs no module name, and a package's own imports resolve inside it
    (the loader tags each module with its package and the checker looks
    for `pkg.name` first), so two packages may both have a `util`. Names
    are claimed first-come across the graph and versions are not
    resolved: with no registry there is no version to compare, and a
    conflict is an error a person should see. The TOML reader is a small
    subset written in the compiler, because the crate takes no
    dependencies.

84. **Editor navigation is syntactic.** Go to definition, completion and
    rename resolve names from the token stream and the parsed modules with
    the language's own scoping rules (locals of the enclosing function
    first, then the file, then imports), not from the typed program. The
    typed program only exists when the file checks, and an editor needs
    answers most while the file does not. The cost is precision at the
    edges: `value.method` finds a method of that name in any `impl`, and
    completion after a dot offers every field and method declared anywhere.
    Hover keeps using the typed program, because a signature with inferred
    effects is worth waiting for. When the file checks, the typed answers
    can replace the syntactic ones without changing the protocol.

85. **Installers are generated scripts for the platform's own tool.** The
    `installer` artifact writes an Inno Setup script on Windows and a POSIX
    shell script elsewhere, and only runs Inno's compiler when it is
    present. Writing the script is always possible and inspectable; the
    setup program needs a tool this compiler should not embed. The Windows
    script is the same shape as Nexium's own installer (PATH handling
    included) so the two stay correct together, and the app id is a hash of
    the program name so upgrades replace the previous install. A `.msi` or
    a `.deb` are left to the release template's optional jobs.

86. **The npm package is FFI, not N-API.** The `node` artifact generates
    plain JavaScript that loads the shared library through koffi, the same
    way the Python package uses ctypes: one shared library serves C,
    Python, Rust and Node, the consumer needs no compiler or node-gyp, and
    the generated code is readable. N-API would give a marginally faster
    call and no dependency, at the price of a C build on every install
    and a second copy of the export boundary to keep correct. 64-bit
    integers cross as BigInt, which is what JavaScript has for them.

## Syntax, second pass

87. **Control flow is `if c { }`, `while c { }`, `for x in items { }`;
    bodies always take braces.** Nexium is meant to read as a high-level
    language by default and a systems language only where it has to, and
    the C-family ceremony around a condition (`(`, `)`, then `{`) carried
    no information: the grammar already knows a condition follows `if`, and
    the brace already marks the body. The line, not a semicolon, ends a
    statement (decision 12), so the brace stays as the one structural
    boundary and indentation never becomes syntax. Consequences:
    - `if c { return v }` replaces the unbraced `if (c) return v`; a body is
      a block, always, so there is one form to read and no dangling-else
      question.
    - `for x, i in items { }`, `for i in lo..hi step s { }`, and
      `for parallel x in items { }` replace `for (items) |x, i|`. The
      capture spelling was Zig's; `in` is what the loop means, and
      `for items |x|` would have collided with bit-or.
    - `if let v = opt { }` replaces `if (opt) |v| { }`. `catch |e|` and
      closure parameters keep their bars: those are functions' parameters,
      not loop variables.
    - A condition is parsed without struct literals (Rust's rule), so
      `if p == Point{ .x = 1 } {` needs parentheses around the literal;
      inside `( )`, `[ ]`, and argument lists a literal is unambiguous again.
      `Name{` stays the literal spelling everywhere else, so decision 6's
      concern is answered by the parser rather than by parentheses.
    - `match` arms were already newline-separated with optional commas.
    - Migration is `nx fmt`: the formatter rewrites 0.5 syntax by token span
      (comments and blank lines survive; `--migrate-only` keeps the layout)
      and is idempotent on new sources. The compiler does not accept the
      old forms; a mixed codebase would have meant two grammars forever.
    This is the last planned change to the surface syntax before 1.0.
88. **The specification promises only what is implemented and tested.**
    Toward 1.0 (ROADMAP phase 5), the four features the spec still carried
    as "planned" are settled, all the same way:
    - Regions: rule R1 stays the only region rule. R2 to R4 of the archived
      design would need a notation for the lifetime of every stored view,
      and nothing in the tree (the compiler in Nexium included) has needed
      it; the uncovered cases are listed in the spec (5.6), and a debug
      build fills freed storage with a fixed byte so they do not pass by
      luck.
    - `packed` and `soa` layouts are removed. `layout(c)` is the one layout
      because the C ABI needs it; a packed struct is a binary pattern away
      (section 7.1), and a structure of arrays is separate lists. Both
      spellings were accepted and silently ignored, which item 27 had ruled
      out; they are errors now that name this decision.
    - `pool` and `stack` allocation strategies are removed. `using arena`
      covers the use the archived design had for them (a block's worth of
      allocations freed at once); `using pool` was always an error.
    A feature wanted later comes back through the stability policy, as an
    addition in a minor version, not as a promise in the spec.
89. **Releases are named after places on a mountain.** A major version is a
    mountain, in the order the fourteen 8000-metre peaks were first climbed
    (0.x and 1.x Annapurna, 2.x Everest, 3.x Nanga Parbat, 4.x K2, ...);
    `X.0.0` is `Mountain: Summit`; a minor version is a camp, route, face or
    feature of that mountain, chosen to fit the release with one line of
    why in the changelog; a patch is a member of the mountain's first-ascent
    expedition, in roster order, so nothing is forced onto a bug-fix
    release. The 0.x line is the approach and the camps of the first
    climb, so 1.0.0 is the top of the mountain the project has been on
    since 0.1.0. The mountain always comes first in the name
    (`Annapurna: Camp V`), so a name reads without knowing the scheme.
    After the 8000ers come the Seven Summits, then the great north faces
    of the Alps; a range beyond those is a new decision. The name appears
    under the version header in `CHANGELOG.md`, in the release title and
    tag message, and in `nx version`; `docs/release-names.md` is the
    reference and the ledger, with the pools of names per mountain. The
    0.x releases were named retroactively. The name fits the release; the
    release is never shaped to fit a name.
90. **The compiler in Nexium is the compiler; the Rust one is a frozen
    seed until 1.0, then gone.** From 0.8, a language change is made in
    `self/` and nowhere else. The Rust crate keeps 0.7 semantics, moves to
    `bootstrap/rust/`, stays buildable and in CI (it is still the binary
    users run until the tools are ported in 0.9), and is deleted at 1.0.
    The seed a fresh machine builds from is the C the Nexium compiler
    emits for itself, `bootstrap/nx.c`, regenerated at each release by the
    previous release's compiler: any C compiler builds `nx0` from it,
    `nx0` builds `self/nx.nx`, and the result must rebuild itself to the
    same C. The oracle comparisons that drove the port (`nx tokens`,
    `sexp`, `tir`, `emit-c` diffed between the two compilers) retire with
    it; the fixed point, the examples, the spec suite, the compile-fail
    cases and fuzzing are the independent check, and they are written
    once. Reasons: two implementations meant every fix twice (0.7's last
    day mirrored five) and a class of bugs that were only disagreements
    between them; a compiler in the language is the proof of the language
    people look for, and the largest program that exercises it; and after
    the port, a contributor needs to know Nexium, not Nexium and Rust. The
    constraint this adds: `self/*.nx` may implement a feature but may not
    use it until the next release's seed understands it.

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
91. Names for discovery: page titles and metadata say "Nexium language",
    since a search has other things called Nexium to get past; the
    compiler stays `nx`. Where a registry already has an unrelated
    `nexium` (PyPI, npm), the language's own packages are `nexium-lang`.
92. **Destructuring views a place and owns a temporary.** `let (a, b) = e`
    and `for (k, v) in pairs` follow `if let` (decision on bindings over a
    place): over a place the names are views of its elements, so the place
    keeps its value and a move out of a name is an error; over an owned
    value the names own the elements and the tuple itself, a hidden local,
    drops nothing. One rule for both, no partial moves to track, and each
    name is an ordinary `let` in the typed IR, so the emitter and the
    interpreter learned nothing new.
93. **`derive(Clone)` is explicit for structs and enums; everything else
    clones by structure.** A struct or enum gets `.clone()` only when it
    says so (a copy of a user type is a design decision, as `Eq` is), and
    then only when every field and payload clones: numbers, strings,
    slices, containers, optionals, tuples, arrays, weak references, other
    `Clone` types, and reference classes, which clone by retaining. A
    pointer, a trait object and a function value do not clone (a pointer
    is a view; clone what it points to). The emitter already derived the
    copy field by field for containers, so the checker gates and the
    backend's `clone_fn` serves; the compile-time interpreter copies the
    value it holds.
94. **An iterator is a value with `next(self: *mut Self) -> ?T`; no
    trait.** `for x in it` owns the iterator (a hidden mutable local) and
    is the `while true { if let x = it.next() { } else { break } }` it
    stands for, so a yielded value is the binding's own and every loop
    rule (`break`, `continue`, labels, the per-pass drop) holds without a
    new statement kind. One iterator per loop, no index (count in the
    body), no `for parallel` (a worker needs a slice to split). A map
    iterates its entries under a tuple binding, `for (k, v) in m`, the
    way it iterates keys: collected first, then walked.
95. **Slice patterns match by shape; the rest is a view; `@` names the
    whole.** `[a, b]`, `[first, rest..]` and `[.., last]` match slices and
    arrays (an array pattern names every element or ends in a rest, so
    the length is static). Elements bind as views of the slice's storage,
    never as owners, and the one `rest..` binds a `[]T` slice of the
    middle. Exhaustiveness is by length: the rest-arm with the fewest
    fixed elements must leave no shorter length without an exact arm,
    which makes `[]` plus `[x, rest..]` total without a `_`. `whole @
    pattern` binds the value (owned when the value is) while its parts
    match as views of it, so nothing is dropped twice.
96. **A guard proves its facts where it holds, and a fact about a `var`
    ends where the variable may change.** Facts flow into the `then` of an
    `if`, the body of a `while` (they hold at the top of every pass) and,
    negated, into the `else` of either; `and` proves both sides, `!` flips,
    `or` proves nothing certain. Beyond ranges, a guard proves `!= 0`,
    `!= null`, a slice's least length and an index below a slice's length,
    which discharge division, `.?`, indexing and slicing. A fact about a
    mutable local is dropped at an assignment, at a mutable borrow, and at
    the entry of any loop (every `var` at once: the loop may change any of
    them on any pass, and the check runs once), which is the conservative
    side of a bug 1.0.3 shipped with, a guard fact surviving into a loop
    that changed the variable. Lengths are tracked for slices only, whose
    length cannot change.
97. **Named format arguments are a mode of the literal, not of the
    placeholder.** `.{a, b}` is positional and `{x}` there is the hex
    spec as before; `.{ .x = a }` is named and every placeholder names
    an argument, `{x}` or `{x:>w}`, with a width taken from an integer
    argument by name. No mixing, and every named argument must be used.
    The checker rewrites either form to `{#index:spec}` for the emitter
    and the interpreter, which evaluate each argument once, in order,
    before the first write, so an argument written twice is computed
    once.
98. **`@target()` is a constant of the C build, not of the compile-time
    interpreter.** The seed and every emitted program are one C file for
    every platform, so the operating system, the architecture and the
    pointer width are decided when the C compiles: `@target()` is a
    tuple built from `nx_host_os()`, `nx_host_arch()` and `sizeof(void*)`,
    an `if` on it compiles both branches and the C compiler folds the
    dead one, and `comptime` cannot evaluate it. `@bitCast` reinterprets
    scalars of one size through `memcpy` (no pointers, no aggregates);
    `@min` and `@max` are the `min`/`max` methods as intrinsics that
    also carry the range of their operands, so `@min(i, n - 1)` proves an
    index; `@alignOf` reads the layout the checker already computes.
99. **`unbounded_stack` stays the call-graph effect, and there is no
    depth proof.** A recursive call on a strictly smaller argument bounds
    the depth by the argument, which is still a depth the caller cannot
    see: an exported function that carries the effect needs a stack sized
    for its input either way (section 15), and the discharge is what it
    has always been, a loop over an explicit stack. The roadmap's
    "reserved" was stale; the effect has been implemented and tested since
    0.9 (spec case `s9_unbounded_stack`, two compile-fail cases), and the
    1.1 theme leaves it as it is.
100. **Views have origins, not lifetimes.** The checker records, per
    local, the storage each view it holds points into (an *origin*: a
    local, a parameter's storage, a literal, a temporary, or the value's
    own heap storage) and when the view was taken. Origins flow through
    bindings, literals, calls (a result may point into any argument
    passed by reference or as a view, not into one moved to an `own`
    parameter), closures (their captures), and the `if let`, loop and
    pattern bindings, which view what they bind over. An `own` parameter
    is the callee's storage, released at return: a view into it is a view
    into a local. There are no lifetime annotations and no
    interprocedural analysis: a signature says all a caller needs, because
    a returned view can only point into the caller's own arguments (V1).
    The rules are checked at the use that would read released storage,
    which is where the fault is, so a view that is never used again costs
    nothing (liveness, not lifetimes). What the model cannot see is taken
    conservatively: a call's result may point into every non-scalar
    argument.
101. **Warnings first, `--strict` for those who want the errors now.** The
    checker gained a warning channel beside its errors: rendered the same
    with `warning:` in front, warning severity in the language server,
    and `--strict` or `NX_STRICT=1` turning them into errors. The view
    rules of 1.2 arrive this way, as `docs/stability.md` requires (a
    warning in one release, an error in the next); 1.3 makes them errors.
    The compiler, the standard library, the examples, the Topo and the
    tests build without a warning, and the harness runs the compile-fail
    cases marked `// STRICT` under `--strict`.
102. **V2 compares scope depths; a temporary is a hidden local of its
    block.** Every local knows the depth of the scope it was declared in,
    and a view of a local of a deeper scope stored into a shallower one is
    reported, as is a view of a temporary (the emitter releases
    temporaries at the end of the block, so a temporary is a hidden local
    of the current depth the rule compares like any other), a view stored
    into a global, and a view stored through a pointer parameter into the
    caller's storage. A view of a borrowed parameter may be stored
    anywhere but a global.
103. **V3 and V4 are events on the storage, matched against when the view
    was taken.** A growth, a clearing or a reassignment of a container and
    a move of a value are recorded with their position, and a read of a
    view taken before the event is reported. Both sides carry the first
    field on their path from the local (`lv.rows` against
    `lv.origins.append`), and meet only when one is the whole value or
    both name the same field: sibling fields of a struct are separate
    storage, and a level compiler that walks one list while filling
    another is the common shape. Inside a loop, a change after
    a read of a view taken before the loop is reported at the change,
    because the next pass reads it. A move on the line of the use is not
    an event: a literal or a return that holds the value and the view
    together keeps the storage alive. A view into heap storage a value
    owns that the same literal moves in (`Pair{ .first = name[..], .name =
    name }`) is a view into the literal's value, which travels with it;
    whether a part is heap storage (a `List`, `String` or `Map` buffer) or
    inline (an array, a struct's fields) decides that, because inline
    storage does not move with a move.
104. **V5 counts arena depth, and `@escape` is the one way out.** Every
    local knows the `using arena` nesting it was declared in and the
    nesting the value it holds was made in; a value made inside a block (a
    call's result, a literal, a clone, a container) kept in a place
    declared outside, handed to a global or to the caller, or returned, is
    reported. `@escape(v)` is a clone made by the allocator the block was
    entered with (the arena's parent context), so the copy survives the
    block; it takes a place, so what it copies is named and released as
    usual. `.clone()` inside the block allocates from the arena and does
    not escape. A call inside the block that returns a value it took from
    outside counts as made inside, because the model does not look into
    callees; `@escape` is the answer there, at the cost of a copy. A value
    taken out of a container (`pop`, `remove`) was made where the
    container's values were.
105. **A struct literal evaluates its initializers in the order written.**
    The checker checked moves in written order while the emitter and the
    interpreter evaluated in declaration order, so `Pair{ .first =
    name[..], .name = name }` with `name` declared first read a zeroed
    `name`; the view rules found it. The typed IR keeps the written order,
    with the field of each initializer beside it, and the defaults of the
    fields left out come last.
106. **A `return` out of `using arena` ends the arena.** The emitter ended
    an arena after its block's closing brace only, so a `return`, `break`
    or `continue` leaving the block leaked the arena's chunks (`nx leaks`
    showed 64 KiB live). The arena belongs to the block's scope now, whose
    exit actions (defers, drops, then the arena's end) run on every way
    out.
107. **An exercise is a file whose name says its kind, and one header
    serves the page and the terminal.** `topo/exercises/<chapter>/` holds
    a starter per exercise (`fill_`, `fix_`, `write_`, `test_`, `predict_`
    decide how it is checked), its recorded output where the output is
    the check, and its solution beside it; a chapter's `quiz.txt` holds
    questions with the right answer starred and a line of why. The
    starter's comment header (`// topo: kind`, the task, `// hint:` lines)
    is what both `site/build.nx` renders and `self/topo.nx` prints, so
    the two cannot drift, and the harness runs every solution, checks that
    every starter fails the way its kind says, and that the compiler
    embeds exactly the files in the directory. A prediction is graded from
    a `.guess` file rather than a prompt, so the terminal course needs no
    interaction beyond the quiz and the harness can walk it. Progress is
    a text file of `done <chapter>/<exercise>` lines in the terminal and a
    list in the browser's local storage on the site, and the two are not
    synchronized: nothing to sign into, and the site never sees the
    reader. The `???` hole is the marker because it is not Nexium: a
    starter with one cannot compile by accident.
108. **The playground runs the compiler's interpreter, not its C backend,
    and never prints what a compiled program would not.** A page on GitHub
    Pages has no server, so the compiler itself runs in the browser: the C
    of the current sources builds for `wasm32-wasi` with the runtime's
    `NX_WASM` branch (no processes, sockets, terminal or `setjmp`; each
    fails as a refusing operating system would), and `nx play` checks a
    program and runs `main` in the interpreter the REPL already had. A
    C compiler in the page is 1.5's work. The rule that makes the
    playground trustworthy is parity: a program prints exactly what its
    compiled self prints, or is refused with the line that needs a compiled
    run; a different answer is a bug. The harness enforces it over every
    recorded program, and holding the interpreter to it found eight places
    it was wrong, all fixed rather than listed as limits. What the
    interpreter cannot know (`@refCount`, the machine's target) it refuses
    rather than guesses; what it models differently but equivalently (a
    `for parallel` loop in order, a `[]mut` argument copied in and written
    back, `ref class` objects in a heap of their own) it keeps only where
    no program can tell. The page gives the interpreter a smaller call
    depth than a native thread, because the browser's stack holds fewer
    frames, so a deep recursion gets the interpreter's own message.
109. **A quiz question can ask the compiler, and then the compiler grades
    it.** A question in a chapter's `quiz.txt` may carry its program (`| `
    lines), what is asked of it (`ask: effects NAME`, `ask: output`, `ask:
    check`) and the compiler's own words for the answer (`says:` lines):
    for effects, the effect list `nx effects` prints and, per effect, the
    first reason `nx explain` gives; for a check, the line of the first
    diagnostic and its message; for output, what the program prints. The
    words are recorded rather than produced when the page is built, so the
    site needs no compiler to render a quiz; `nx topo verify` asks the real
    compiler every such question and fails when the starred choice or the
    recorded words differ from what it says now, and the harness runs it,
    so a quiz cannot outlive the language it is about. A wrong answer is
    met with the compiler's words, in the terminal (`nx topo quiz`) and on
    the page, where a click on a choice is graded at once. A question
    without `ask:` is an answer key as before; the two kinds mix in one quiz.
110. **A method is visible wherever its type is, a type's `impl` blocks may
    sit in any module that sees it, and a type has one method of a name.**
    Methods followed fields (visible to any code that can see the struct)
    in the implementation from the start, and an `impl` of a type from
    another module worked, but neither was written down, and a method
    defined twice, in two blocks of one module or in two modules, was
    accepted with the first definition silently winning every call. The
    rules are now the specification's: `pub` on a method marks the type's
    documented interface (`nx doc` lists it) rather than guarding it; the
    inherent methods of one type may be spread over blocks and modules,
    which is how a large type such as the compiler's `Checker` is split by
    responsibility without widening anything; and a second method of a
    name is an error at the second definition, a generic `impl(T)`
    conflicting with every instance of its head and `impl Pair(i32)` with
    `impl Pair(i32)` but not with `impl Pair(f64)`.
111. **`nx fix` keeps an edit only when the checker, run on the edited
    program, reports fewer errors and none it did not report before.**
    With the view rules errors in 1.3, the promise of 1.2 was that `nx fix`
    would insert `.clone()` where that is the fix. Where it is is narrower
    than the old advice suggested: a view's clone is the same view, so the
    only clone that fixes a view is of the value it points into, and that
    changes the view's type unless it is made where the value moves (rule
    V4: `take(s)` becomes `take(s.clone())`, and every type stays). Rule
    V5 has the other type-preserving edit, `@escape(...)` around the value
    kept past its arena. The checker offers these edits with the errors
    (`check.Fix`), and rather than trust each offer, `nx fix` tries it on
    the program in memory and keeps it only when the error count falls and
    no new error appears, then takes the next offer from a fresh check of
    the edited program. An offer that would not type-check, a clone of a
    struct that does not derive `Clone` for one, is dropped silently, and
    whatever is left is reported by count. V1, V2 and V3 get no edit: their
    fixes change a type or need a binding in the right scope, which is a
    decision for the programmer; the errors say what to do.
112. **A trait impl's methods have the signatures the trait declares.**
    Nothing checked it: impl and trait methods were paired by name only, so
    an impl could return `i32` where the trait declares `i64`, drop an
    `own`, or take `*Self` where the trait takes `*mut Self`, and a call
    through `dyn` or through generic code bound by the trait then ran the
    impl with the trait's types (a wrong value, or a double free where an
    `own` went missing). Now each method of `impl Trait for T` must have the
    trait's receiver, the same parameters with the same `own`, and the same
    return type, `Self` read as `T`; a generic impl is held to the receiver,
    the parameters' number and `own` (its types are checked where it is
    instantiated). This also lets the vtable type come from the trait's
    declaration alone, so a function over `dyn Trait` compiles in a program
    that never coerces a value to it. Programs this rejects were already
    wrong (specification 8.3).
113. **`nx fix` makes an edit only when it keeps what the program does;
    `+%` and `+|` are the programmer's to choose.** The 1.3 plan had `nx
    fix` apply every hint the compiler gives, and one of them names two
    operators: in a function declared `!panics`, arithmetic that may
    overflow is met with "use `+%` to wrap or `+|` to saturate". Either
    edit makes the error go, and so would pass decision 111's test, but
    each gives the overflowing case a different answer (a hash wants the
    wrap, a level or a count wants the clamp, and a program that wants
    neither wants a wider type or a range the checker can prove), so
    picking one would be `nx fix` deciding what the program computes. The
    edits it makes keep what the program does and only settle what the
    checker asks: V4's `.clone()` and V5's `@escape(...)` (111), and `_ = `
    in front of a value nothing uses, where the value is computed as
    before. `_ = ` is not offered for an error union (discarding an error
    is a decision too) nor inside an `if` with no `else` whose value is
    wanted, where the missing `else` is the mistake.
114. **`if comptime C` checks and builds only the branch `C` picks.**
    Platform code had one tool, `@target()` as a C constant, and with it
    both branches of an `if` were checked and compiled: a call to an API
    that one platform lacks failed to link on the others, which is why
    every platform difference lived in the runtime's `#ifdef`s. Now the
    whole condition after `if comptime` is evaluated while checking (the
    compile-time interpreter knows `@target()`: `--target`'s triple, or the
    machine compiling, in the words the runtime uses), and the branch not
    taken is parsed but never checked or built. The cost is Zig's: an
    error in a branch no build takes goes unreported until a build takes
    it, which is why CI builds for the three platforms. `comptime` after
    `if` takes the whole condition (`if comptime a == b`), where before it
    bound as a prefix operator and only folded `a`; nothing in the tree or
    in the packages known to use Nexium wrote that.
115. **When the program checks, the language server answers from the
    checker.** Decision 84 made navigation syntactic so it would answer
    while code is half-written, at the price of guessing: `value.method`
    went to a method of that name in any `impl`, a field access could not
    know its struct, and rename edited one file. Now, when the program
    checks, an index built from the typed IR (`self/lsp_index.nx`) says
    what each use of a local, function, method or field resolved to, with
    the span of the declaration's name: definition goes there, hover shows
    the local's or field's type or the function's signature and effects,
    and rename edits the declaration and every use in each of the
    program's own files (never std, a package or a C header). Where the
    program does not check, and for completion, types, enum variants and
    constants (which the IR does not keep by name), the syntactic answers
    of decision 84 stand. Rename sees what the checker checked: a name
    used only in an `if comptime` branch this build does not take, or in
    a generic function nothing instantiates, is not reached.
116. **A large program's debug build is a C file per module, each object
    reused while its C is unchanged.** The compiler's own debug build was
    one 190,000-line C file, eight and a half seconds to compile after any
    change. Now a debug build of a program of about 200 KB of source or
    more (`NX_UNITS=1` or `0` decides for any program) is generated as one
    C file per module (`cgen.generate_units`): each declares every
    function and the types it needs, defines its own module's functions
    with external linkage, and keeps its helpers and literals static; the
    root module's file defines the globals and holds the entry. The
    runtime's own state (the panic boundary, the file table, stdin's
    buffer, the console's modes) is declared by every file and defined by
    the root's (`NX_STATE` in `nx_rt.h`), where one file keeps it static.
    Each object is named by a hash of its C and of the command line that
    compiles it, so a build compiles again only the files whose C changed,
    on as many threads as there are cores (at most eight), and links.
    With a released (optimized) `nx`, a one-line change to the compiler
    rebuilds it in 0.99 seconds on Linux with gcc and 1.75 on Windows with
    zig cc, where the one C file took 7.8; of that, checking the whole
    program is 0.74, so "well under a second" everywhere waits for checking
    by module. `nx emit-c`, and so the seed, stays one file, as do
    optimized builds, whose optimizer sees across functions.
117. **A mountain runs from a line's `.5` to the next line's `.4`, and its
    summit is the major between.** Decision 89 made a major a mountain and
    its `X.0.0` the summit, which held for the first one only: the whole 0.x
    line was Annapurna's climb, so 1.0.0 stood on top. From Everest on, the
    first release on a mountain would have been its summit, and the plan
    then climbed the route from the bottom (2.1 at the Khumbu Icefall, the
    foot of the route, after 2.0 at the top). Now a mountain's climb starts
    at the `.5` of the line before, its minors go up the first-ascent route
    in order to the last place below the top for the release before the
    major, `X.0.0` is the summit, and the minors after it, up to `X.4`, take
    the mountain's other routes, faces, neighbours and descent by fit;
    patches take the first-ascent team of the mountain their minor is on. A
    late major means more camps; a major announced before the `.5` (its
    deprecations land earlier) starts the climb at that release, so none
    arrives without one. The mountain in a name then says which summit the
    line is climbing to or has just stood on, where before it said which
    major: `Everest: Base Camp` is 1.5.0. Every released name already fits
    (0.x the climb, 1.0.0 the summit, 1.1 to 1.3 Annapurna's other routes);
    the names pencilled in for 1.5 to 1.8 go back to Annapurna's pool, and
    Everest's route moves from after 2.0 to before it.
118. **A temporary a branch's value is made of belongs to the whole `if`,
    `match` or block expression.** SPEC 5.2 releases owned values when
    their scope ends without saying whose scope a temporary is. A plain
    expression's are the enclosing block's: `show(format(...))` and `let t:
    []u8 = format(...)` keep the String until the block ends. The
    temporaries a branch made for its value, and an owned `if let` capture
    or pattern binding, were the branch's, released when it closed, so a
    branch whose value was a view of one handed on freed memory:
    `show(if c { format(...) } else { ... })` read a released String, with
    no diagnostic, where the view rules promise that never happens. They
    are now the expression's: declared before it zeroed, so a branch that
    did not run releases nothing, and released with the block the
    expression is in, as a plain expression's are. A statement inside a
    branch keeps its own temporaries, released at the branch's end as
    before, so a loop in a branch does not pile them up. Found by QNI's
    tests against glibc (#15).
119. **`random.secure(buf)` fills a mutable byte slice from the operating
    system's generator, and returns `!void`.** Keys, tokens, UUIDs and a
    `Map` hashed against flooding need bytes an attacker cannot predict;
    `random.int` is splitmix64 seeded from the clock, and `random.seed`
    makes it repeatable on purpose. The secure source stays next to it in
    `random`, where people look, with the difference in its name. It fills
    a slice rather than returning a String, so code that must not allocate
    (a TLS record, a key schedule under `!allocates`) can use it; it
    returns an error rather than panicking because every other builtin
    that asks the operating system does, though it fails only where there
    is no generator at all (`IoError`). Each platform's own source: Windows
    BCryptGenRandom, looked up in bcrypt.dll at the first call so no program
    links another library; Linux the getrandom system call made directly,
    as glibc's wrapper would raise the floor past 2.17, falling back to
    `/dev/urandom`, checked to be a device, on kernels before 3.17; macOS
    and the BSDs `arc4random_buf`; WASI `getentropy`. It is
    `nondeterministic`, so compile-time evaluation refuses it; the REPL's
    interpreter does not run it yet, as it cannot write through a slice
    argument.
120. **HTTPS in std goes through a TLS slot: nxtls fills it first, the
    platform's TLS second.** `std.http`'s client speaks HTTP over any
    stream, and one interface turns a TCP connection into an encrypted
    one: connect with a deadline, send, receive, close, and whether the
    server ended cleanly (close_notify) or the connection was cut. nxtls,
    the TLS 1.3 client written in Nexium, fills it first: no C, the same
    behaviour on every platform, tested byte for byte against Python's
    `cryptography` and OpenSSL, and reviewed and fixed before this was
    decided (0.4.0). It speaks TLS 1.3 with ChaCha20-Poly1305 over X25519
    only, which every large host tried accepts (Discord, GitHub, Google,
    Cloudflare, 1.1.1.1, 8.8.8.8), and it stays a package that a program
    hands to the client. The platform's TLS fills the same slot second and
    is std's own `https`: SChannel, Security.framework, OpenSSL where the
    system has it. It reaches what nxtls cannot: www.echolink.org, probed
    on 2026-09-25, answers every TLS 1.3 ClientHello with
    handshake_failure and on TLS 1.2 takes only
    ECDHE-ECDSA-AES256-GCM-SHA384 over P-256. Neither alone: the platform
    alone puts C over three operating systems' APIs under every HTTPS
    call, with their certificate stores and their errors, and leaves
    nothing that behaves the same everywhere; nxtls alone would need TLS
    1.2, constant-time AES-GCM and constant-time P-256 key exchange
    written in Nexium, weeks of cryptography for the old servers the
    platform already reaches. This replaces the plan of 1.4's first draft,
    the platform's TLS alone.
