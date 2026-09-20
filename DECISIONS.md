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
