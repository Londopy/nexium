# Nexium roadmap

Where the language is, where it goes, and in what order. Dates are not
promises; the order is. Each phase ends when its exit criteria hold, and
every item lands with tests, docs, and a decision entry when it settles a
design question. This file is updated when a phase closes.

The spec's one sentence is the compass: *a language complete enough to build
everything in, that is also the best thing to adopt for one piece of
something else.* The second half is ahead of the first, on purpose.

## Now: 0.7.0

What exists and is verified on Windows, Linux, and macOS:

- The language: structs, records, enums, `ref class` with ARC and `weak`,
  distinct types, traits and `dyn Trait` with effect bounds, generics by
  monomorphization, closures with explicit captures, error unions, optionals,
  `defer`/`errdefer`, labeled blocks, `own` parameters, branch-aware moves,
  binary pattern matching and construction, `comptime`, `for parallel`,
  `using arena`, recursive types through `List`, matching through pointers.
- Effects: inferred for every function, checked against negative bounds,
  `panics` discharged by proof, effects deciding the C ABI of exports.
- Interop: `@cImport` of C headers, vendored C, `nx ship` to C, Python
  wheels, and Rust crates.
- Standard library in Nexium: `strings`, `lists`, `bytes`, `num`, `json`,
  `args`, embedded in the compiler.
- Tools: `build run test check effects audit ship emit-c tokens fmt doc size
  refcounts leaks lsp doctor`.
- Distribution: a Windows installer with bundled Zig, a macOS/Linux install
  script with checksum verification, a VS Code extension, a Sublime syntax,
  a GitHub Action, and a release template for Nexium programs.
- Self-hosting: the lexer, parser, checker, C emitter and driver in
  Nexium, verified byte-for-byte against the Rust ones on every source;
  the driver builds itself.
- nexium-gui: an immediate-mode GUI in Nexium on a 200-line C window layer.

Numbers: 29.4k lines of Rust (the compiler), 18.7k of Nexium, 190 std
functions, 32 examples, 32 spec conformance cases and 41 compile-fail
cases, 19 integration tests, verified on three platforms by CI.

## Phase 1: a language you can write your tools in (0.3)

Goal: someone replaces a Python or Rust command-line tool with Nexium and
misses nothing for the tool's own needs.

- `std.fs`: list directories, stat, create and remove, paths (join, parent,
  extension, absolute), temp files. Runtime section for the platform calls.
- `std.time`: wall-clock breakdown, formatting, durations, monotonic timers.
- `std.regex`: a backtracking-free engine (Thompson or Pike VM) in Nexium,
  with the usual syntax subset; a stress test of the language by design.
- `std.text`: UTF-8 iteration by code point, grapheme-agnostic width,
  case mapping beyond ASCII where cheap.
- `std.testing` conveniences: `expect_err`, approximate float comparison,
  snapshot files.
- `io`: buffered readers and writers, stdin as a stream, environment
  variables as a map.
- Language: `while` with `else`, `match` on tuples of enums, integer
  ranges in `for` with steps, `String` slicing helpers, `?T` chaining
  (`a?.b`) if the spec's ergonomics survive review.
- Tools: `nx fmt` gains line joining for trailing commas only; `nx doc`
  covers std; `nx test --filter` and `--verbose`.
- Done ahead of the phase: `nx repl` (the interactive session on the
  interpreter) and a Start menu entry that opens it.

Exit: `examples/tool.nx`, a real utility (a log grep with regex, dates, and
file walking) under 300 lines, runs on all three platforms from the release.
Status: done except `?T` chaining (`a?.b`, waiting on the spec review) and
the `nx fmt` trailing-comma rule. `std.fs`, `std.time`, `std.regex`,
`std.text`, `std.testing`, `std.stream`, range steps, `while`/`else`, full
match exhaustiveness, `nx test --filter`/`--verbose`, `os.environ`, and
`examples/tool.nx` (142 lines) shipped in 0.3.0.

## Phase 2: a language that talks to the world (0.4)

Goal: services and clients without leaving the language.

- Sockets in the runtime: TCP and UDP, blocking, with timeouts; the
  `blocks` effect on every call.
- `std.net`: addresses, DNS lookup, TCP client and server helpers.
- `std.http`: a client (HTTP/1.1, chunked, redirects) and a small server
  (routing, static files) on top of sockets. TLS through `@cImport` of a C
  library (`libtls` or OpenSSL), documented, optional.
- Status: sockets, `std.net`, `std.http`, `std.thread` (threads, channels,
  mutexes) and `examples/service.nx` are done; TLS stays optional through
  `@cImport`. The async decision is taken: threads and channels, no colored
  functions (decision 82). `std.process` captures output, feeds stdin and
  sets the working directory. Phase 2 is complete; POSIX signals are left
  for a program that needs them.
- Threads (spec 7.2): spawn, join, channels with move semantics, `Mutex`
  as a `ref class`; `shared_mutable` becomes the effect that gates them.
- Process: `process.run` gains stdin/stdout capture, environment, working
  directory; signals on POSIX.
- Decision needed: async. The spec does not have it. Phase 2 decides between
  blocking threads only (simple, matches the effect system) and a structured
  concurrency design. Default answer unless a strong case appears: threads
  and channels, no colored functions.

Exit: `examples/server.nx`, an HTTP server serving JSON from a directory,
and `examples/client.nx` fetching and parsing a URL, both under test.

## Phase 3: a language other people can build on (0.5)

Goal: sharing code beyond a sibling file.

- Packages: a `nexium.toml` manifest, `import pkg.module`, versioned
  dependencies fetched from git tags first (no registry required), lock
  file, vendoring. Status: done (`nx init`, `nx add`, `nx fetch`,
  `docs/packages.md`).
- `nx publish` and a minimal registry (static index in a git repository,
  the way early Cargo worked), only once there are packages to publish.
- `node` artifact: `nx ship` to an npm package, since Python and Node
  together cover most "one native piece" needs. Done, through koffi rather
  than N-API (decision 86).
- `installer` artifact: `nx ship` producing an installer for a Nexium
  program, reusing this repository's Inno and script templates. Done.
- Editor support: tree-sitter grammar for Neovim, Helix, and Zed, and the
  language server's completion, go-to-definition and rename: done.
- Linguist PR, when the usage bar is met.
- Status (0.5.0): everything above is done except `nx publish` and the
  registry, which wait for packages to exist, and the Linguist PR.

Exit: a second person's package is used by a third person's program.

## Phase 4: the compiler in Nexium (0.6 to 1.0)

0.6 opens with the syntax settled for 1.0 (decision 87): `if c { }`,
`while c { }`, `for x in items { }`, `if let v = opt { }`, braces always,
no semicolons. `nx fmt` migrates 0.5 sources. Nothing else about the
surface syntax is planned to change before 1.0.

Goal: `nx` built by `nx`. Runs in parallel with phases 1 to 3; each stage is
checked against the Rust compiler on identical inputs.

- Parser (`self/parser.nx`): id-arena AST, dumped in the shape of
  `nx sexp`, diffed over every example and std module. Done in 0.6.0:
  identical on all 46 sources, enforced by `cargo test`.
- Checker (`self/check.nx`): the largest stage. Types interned in a `List`,
  effects by fixpoint, ownership, monomorphization, dumped in the shape of
  `nx tir` and diffed over every source. Done so far: declarations and
  signatures (`--sigs`), then bodies: statements, expressions, calls,
  builtins, matches and patterns, casts, moves, ranges, generics, closures,
  records, trait objects, binary patterns, the compile-time interpreter,
  the diagnostics passes (effect bounds, exhaustiveness, escaping views),
  `@cImport` (`self/cimport.nx`). Done: identical on every source (54,
  `check.nx` itself included), every compile-fail case rejected with the
  same messages.
- C emitter (`self/cgen.nx`): byte-identical C for every example. Done:
  identical on every source in the tree, its own included.
- Driver, tools, and the std embedding in Nexium. Done: `self/nx.nx`
  (build, run, test, check, emit-c, tir) with the standard library
  embedded; the other tools (fmt, doc, lsp, ship, packages) stay in Rust
  for now.
- Bootstrap: Rust `nx` builds `nx1`; `nx1` builds `nx2`; `nx1` and `nx2`
  produce identical output. Done for the emitter (`cargo test` runs the
  three stages); the driver's tools come next, then the Rust compiler
  moves to `bootstrap/`, kept for building the first Nexium compiler on a
  fresh machine.

- Status (0.7.0): syntax settled; lexer, parser, checker, C emitter and
  driver done and bootstrapped: `nx` written in Nexium builds itself. Left
  for the exit: moving the Rust compiler to `bootstrap/` and building the
  release tarball's `nx` from the shipped C. Porting the other tools (fmt,
  doc, lsp, ship, packages) is not required for 1.0; see past 1.0.

Exit: `cargo` is no longer needed to build `nx` from a release tarball.

## Phase 5: 1.0

A 1.0 means the language stops changing under people's feet.

- The specification in `SPEC.md` is complete and every section has tests.
  Done: `tests/spec` holds one recorded program per claim in sections 2
  to 14, diffed through every self-hosting stage; the artifact and
  toolchain sections (15, 16) are covered by the ship tests.
- Regions: the full rule set (R1 to R4 in the archived design) or an
  explicit decision to keep R1 only, with the escape cases documented.
  Done: R1 only (decision 88), the uncovered cases listed in SPEC 5.6.
- Layouts: `packed` and `soa` implemented or removed from the spec. Done:
  removed (decision 88); the spellings are errors.
- Allocation strategies: `pool` and `stack` implemented or removed. Done:
  removed (decision 88).
- Stability policy: what a minor version may change, deprecation cycle,
  `nx fix` for mechanical migrations.
- Platforms: Linux aarch64 and Windows arm64 in releases; a tier list.
- Security: a disclosure process is in `SECURITY.md` already; add fuzzing
  of the parser and the binary pattern engine to CI.
- C toolchain support: which Zig versions `nx` is tested with, in CI and
  in the docs. CI pins 0.14.1; 0.16 differs in what its debug build traps
  and in which Apple SDKs it links against, and 0.7.0 found bugs that only
  one of them showed.

## Past 1.0

After 1.0 the language changes only by addition, under the stability policy
of phase 5. Each minor version has a theme; a bullet moves into a version
when it has a decision entry and a test plan. The compass sentence still
decides the order: what the compiler and the tools in `self/` and `std/`
needed first, what other people's programs need next. Everything here was
found by writing Nexium, not by reading other languages' feature lists.

### 1.1: the language the compiler wanted

Ergonomics the self-hosted compiler paid for by hand.

- `?T` chaining: `a?.b` is `null` when `a` is (deferred since phase 1).
- Tuple destructuring: `let (a, b) = pair`, in `for` bindings too.
- `derive(Clone)` for structs and enums whose fields all clone, the way
  drops are derived (KNOWN_ISSUES: every deep copy in `check.nx` is a hand
  written function).
- Iterators: `for x in v` over any value with a `next(self: *mut Self) -> ?T`
  method, so a user type iterates like a slice; `Map` iteration over
  entries `(k, v)` as well as keys.
- Slice patterns: `[first, rest..]`, `[a, b]`, and `x @ pat` bindings in
  `match`.
- Range facts through `while` conditions and `else` branches (KNOWN_ISSUES),
  and proofs for `x / y` after `y != 0`, `.?` after `x != null`, and slice
  bounds after `if s.len >= n`, so fewer functions carry `panics`.
- Format width from a value (`{:>w}`) and named placeholders with an
  anonymous literal (`format("{x}", .{ .x = 1 })`).
- Intrinsics still missing from a systems language: `@bitCast`, `@min`,
  `@max`, `@alignOf`, `@target()` (os, arch, pointer width) for compile-time
  `if` in std.
- `unbounded_stack`: the spec says reserved. Either a recursion-depth
  proof (a function is bounded when every recursive call is on a strictly
  smaller argument) or removal, the way decision 88 settled the others.

Exit: the compile-time interpreter and `check.nx` lose their hand-written
copies and index workarounds; `nx audit self/check.nx` reports fewer
`panics` than in 1.0.

### 1.2: the toolchain in Nexium

The Rust crate becomes a bootstrap seed and nothing else.

- `fmt`, `doc`, `lsp`, `ship`, packages, the REPL and the migrator in
  Nexium, each diffed against the Rust tool until it retires.
- Incremental builds: one C file per module, compiled separately and
  cached by content hash, so a one-line change does not recompile a
  100k-line translation unit; parallel checking of independent modules.
- `#line` directives in the generated C, so a debugger shows `.nx` lines,
  and `nx debug` launching lldb or gdb with formatters for `List`,
  `String`, `Map`, slices and optionals.
- A semantic language server: hover, diagnostics, go-to-definition and
  rename backed by the checker's typed IR, not the parser (decision 84 was
  syntactic on purpose, for 0.5).
- `nx fix` applies the compiler's own hints: `+%`/`+|` where the note
  suggests it, `.clone()`, `_ =`, the migrations.
- `nx bench`: `bench "name" { }` blocks with warmup, iterations and
  medians, in the same file as tests.
- `nx build --sanitize address,undefined` through the C compiler, and
  `nx test --sanitize` in this repository's CI.
- Conditional compilation: `if comptime @target().os == "windows" { }` in
  std replaces the runtime's `#ifdef`s one by one.

Exit: `cargo` is used only to build the bootstrap seed; every tool a user
runs is a Nexium program.

### 1.3: a standard library people stop supplementing

- Collections: `Set(T)`, `Deque(T)`, `std.sort` with comparators and
  stable sort, `std.heap` (priority queue), binary search on sorted slices.
- `std.path` (split off from `std.fs`), `std.env` (config files, XDG and
  AppData directories), `std.csv`, `std.toml` (the manifest parser leaves
  Rust), `std.base64`, `std.hash` (FNV, SipHash for `Map`, SHA-256 for
  checksums), `std.uuid`, `std.log` with levels and structured fields.
- `std.http` client with redirects, timeouts and streaming bodies; TLS
  through the platform (SChannel, Security.framework, OpenSSL where the
  system has it) so `https` works without vendoring a library.
- `std.text`: grapheme clusters and case mapping tables, `chars()` over
  scalars, width for terminal alignment.
- `std.time`: time zones from the platform database, ISO 8601 parsing
  in both directions, `Duration` arithmetic.
- `std.process`: pipes as streams, signals, exit codes by name.
- `std.thread`: `select` over channels, scoped threads that are joined
  when the block ends (no handle can escape), atomics in `sync`.
- `std.testing`: property-based tests (`check(gen, fn)`) with shrinking,
  the same driver the fuzzers use.

Exit: `examples/tool.nx`, `service.nx` and the self-hosted compiler import
nothing they had to write themselves.

### 1.4: platforms

- WebAssembly: `--target wasm32-wasi` and `wasm32-freestanding`, the
  runtime's process, socket and thread code behind `@target()`, and an
  `artifact wasm` producing a `.wasm` with a JavaScript loader. The
  self-hosted compiler compiled to wasm is the playground: `nx` running in
  a browser, no server.
- nexium-gui: X11 and Wayland, Cocoa backends beside Win32; the demo runs
  on all three and in the browser through a canvas backend.
- Static Linux binaries (musl), FreeBSD, and the tier list extended;
  Linux aarch64 and Windows arm64 promoted to tier 1 when CI runs them.
- Cross-compilation matrix in `nx ship`: every target the C toolchain
  supports, from one machine, tested in CI for the tier-1 set.
- Embedded targets: `-Os` builds without the runtime's file, socket and
  thread parts (`@target().os == "none"`), the first program on a
  microcontroller.

Exit: a Nexium program runs in the browser, on a Raspberry Pi and on the
three desktops from one source, and the docs say which combinations CI
proves.

### 1.5: the runtime that release builds deserve

- ARC elision: retain/release pairs the checker proves redundant (a `ref
  class` passed down and back within one function) are not emitted; the
  `refcounts` effect reports the ones that remain.
- Bounds-check elimination in loops from the range analysis; `nx audit`
  shows which checks survive in a hot function.
- `Map`: open addressing with a hash chosen per key type, iteration order
  documented; small-string optimization for `String`; `List` growth policy
  documented and tunable per `using` block.
- `for parallel`: work stealing, a chunk size heuristic, and nested
  parallel loops that share one pool.
- Compile time: the checker's monomorphization cache, and preprocessed
  `@cImport` headers cached by hash.
- A benchmark suite (`bench/`) against C, Rust, Go and Python on the
  programs the examples already implement, run by CI on a fixed runner
  with results in the repository, so a regression is a failing check.

Exit: every example in release mode is within a documented factor of its
C counterpart, and the factor does not grow between releases.

### 2.0 candidates: questions the spec review should settle

Additions large enough to deserve a spec version of their own. Each is a
decision entry first, an implementation second, and none is promised.

- Errors that carry data: `error Parse { Bad{ line: u32 } }` with payloads
  in `catch |e|` and `match`, or the position that error sets stay names
  and context travels beside them.
- Operator overloading through traits (`Add`, `Index`, `Eq` is derived
  already) for numeric types written in Nexium, or the position that
  operators mean what they mean for builtins only.
- Named arguments, or the position that an anonymous literal parameter
  (`f(.{ .width = 3 })`) is the language's way.
- Generic traits and associated types (`trait Container(T)`,
  `trait Iterator { type Item }`), which 1.1's iterator protocol may force.
- Visibility levels (`pub(package)`) and re-exports (`pub import`).
- An `unsafe` audit: `nx audit --unsafe` listing every `unsafe` block, its
  reason comment (mandatory), and the foreign calls under it.

### Ecosystem, in parallel with all of the above

- The book: a chapter per spec section, each with a program; the tour is
  chapter one. `nx doc` renders HTML for the book and for std, and the
  docs site is built by CI from the repository.
- `nx new <template>`: cli, service, library, gui, wasm.
- Editors: Neovim and Helix configurations (the tree-sitter grammar and
  the language server exist), an Emacs mode, a JetBrains plugin when the
  language server is semantic.
- The registry and `nx publish` (decision 27's last item), seeded with
  the packages that leave the tree: the HTTP client, TLS, TOML, CSV,
  SQLite through `@cImport`, a CLI parser richer than `std.args`.
- Governance: an RFC process for additions (the decision log becomes
  public proposals with a comment period), a release calendar, and the
  stability policy applied to `std` (what a std module may change).
- Translations of the book and the reference (docs/i18n exists for the
  README, language and architecture pages).

## Always

- Every release is verified on three platforms by CI before it is tagged.
- Every language change names the spec constraint it serves and lands with
  an example or a compile-fail case.
- Every open design call goes in `DECISIONS.md` the day it is made.
- The Nexium share of the repository rises every phase and is never
  padded.

## Not planned

- A garbage collector. Reference counting with `weak` is the design.
- `async`/`await`. Threads, channels and blocking calls are the concurrency
  model (decision 82); an event loop can be a library.
- A native backend. C is the backend; Zig or a system compiler is the
  toolchain, and the installers make that invisible.
- Async/await as colored functions, unless phase 2 finds a design that fits
  the effect system.
- Exceptions. Error unions and panics are the two failure modes, and panics
  never cross an export boundary.
