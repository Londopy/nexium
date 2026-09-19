# Nexium roadmap

Where the language is, where it goes, and in what order. Dates are not
promises; the order is. Each phase ends when its exit criteria hold, and
every item lands with tests, docs, and a decision entry when it settles a
design question. This file is updated when a phase closes.

The spec's one sentence is the compass: *a language complete enough to build
everything in, that is also the best thing to adopt for one piece of
something else.* The second half is ahead of the first, on purpose.

## Now: 0.4.0

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
- Self-hosting: the lexer, verified byte-for-byte against the Rust one.
- nexium-gui: an immediate-mode GUI in Nexium on a 200-line C window layer.

Numbers: 22.7k lines of Rust (the compiler), 3.0k of Nexium, 96 std
functions, 19 examples and 19 compile-fail cases, 7 integration test groups.

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
- `node` artifact: `nx ship` to an npm package (N-API through the C ABI),
  since Python and Node together cover most "one native piece" needs.
- `installer` artifact: `nx ship` producing an installer for a Nexium
  program, reusing this repository's Inno and script templates.
- Editor support: tree-sitter grammar for Neovim, Helix, and Zed; the
  language server gains completion, go-to-definition, and rename.
- Linguist PR, when the usage bar is met.

Exit: a second person's package is used by a third person's program.

## Phase 4: the compiler in Nexium (0.6 to 1.0)

Goal: `nx` built by `nx`. Runs in parallel with phases 1 to 3; each stage is
checked against the Rust compiler on identical inputs.

- Parser (`self/parser.nx`): id-arena AST, dumped in the shape of
  `nx parse`, diffed over every example and std module. Needs the tuple
  and enum machinery the JSON module proved.
- Checker (`self/check.nx`): the largest stage. Types interned in a `List`,
  effects by fixpoint, ownership, monomorphization. Verified by the
  compile-fail suite producing identical messages.
- C emitter (`self/cgen.nx`): byte-identical C for every example.
- Driver, tools, and the std embedding in Nexium.
- Bootstrap: Rust `nx` builds `nx1`; `nx1` builds `nx2`; `nx1` and `nx2`
  produce identical output. Then the Rust compiler moves to `bootstrap/`,
  kept for building the first Nexium compiler on a fresh machine.

Exit: `cargo` is no longer needed to build `nx` from a release tarball.

## Phase 5: 1.0

A 1.0 means the language stops changing under people's feet.

- The specification in `SPEC.md` is complete and every section has tests.
- Regions: the full rule set (R1 to R4 in the archived design) or an
  explicit decision to keep R1 only, with the escape cases documented.
- Layouts: `packed` and `soa` implemented or removed from the spec.
- Allocation strategies: `pool` and `stack` implemented or removed.
- Stability policy: what a minor version may change, deprecation cycle,
  `nx fix` for mechanical migrations.
- Platforms: Linux aarch64 and Windows arm64 in releases; a tier list.
- Security: a disclosure process is in `SECURITY.md` already; add fuzzing
  of the parser and the binary pattern engine to CI.

## Always

- Every release is verified on three platforms by CI before it is tagged.
- Every language change names the spec constraint it serves and lands with
  an example or a compile-fail case.
- Every open design call goes in `DECISIONS.md` the day it is made.
- The Nexium share of the repository rises every phase and is never
  padded.

## Not planned

- A garbage collector. Reference counting with `weak` is the design.
- A native backend. C is the backend; Zig or a system compiler is the
  toolchain, and the installers make that invisible.
- Async/await as colored functions, unless phase 2 finds a design that fits
  the effect system.
- Exceptions. Error unions and panics are the two failure modes, and panics
  never cross an export boundary.
