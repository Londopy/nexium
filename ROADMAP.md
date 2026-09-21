# Nexium roadmap

Where the language is, where it goes, and in what order. Dates are not
promises; the order is. Each phase ends when its exit criteria hold, and
every item lands with tests, docs, and a decision entry when it settles a
design question. This file is updated when a phase closes.

The spec's one sentence is the compass: *a language complete enough to build
everything in, that is also the best thing to adopt for one piece of
something else.* The second half is ahead of the first, on purpose.

## Now: 1.0.0

What exists and is verified on Windows, Linux, and macOS:

- The language: structs, records, enums, `ref class` with ARC and `weak`,
  distinct types, traits and `dyn Trait` with effect bounds, generics by
  monomorphization, closures with explicit captures, error unions, optionals,
  `defer`/`errdefer`, labeled blocks, `own` parameters, branch-aware moves,
  binary pattern matching and construction, `comptime`, `for parallel`,
  `using arena`, recursive types through `List`, matching through pointers.
- Effects: inferred for every function, checked against negative bounds,
  `panics` discharged by proof, effects deciding the C ABI of exports.
- Interop: `@cImport` of C headers, vendored C, `nx ship` to a C
  library, a Python wheel, a Rust crate, an npm package, an installer.
- Standard library in Nexium, sixteen modules embedded in the compiler:
  `strings`, `lists`, `bytes`, `num`, `json`, `args`, `fs`, `time`,
  `regex`, `text`, `testing`, `stream`, `net`, `http`, `thread`,
  `process`.
- Tools: `build run test check tir fix fmt effects audit refcounts doc
  ship size leaks version doctor lsp repl`, all written in Nexium.
- Distribution: a Windows installer with bundled Zig, a macOS/Linux install
  script with checksum verification, editor support for VS Code, Vim,
  Neovim, Helix, Zed, Emacs, Kate, JetBrains, Sublime Text, Notepad++ and
  nano, a GitHub Action, and a release template for Nexium programs.
- Self-hosting: the compiler is written in Nexium and builds itself from
  the C it emits (`bootstrap/nx.c`); no other compiler is involved.
- nexium-gui: an immediate-mode GUI in Nexium on a 200-line C window layer.
- Documentation: the site at londopy.github.io/nexium, built by a Nexium
  program from the repository's Markdown, and the Topo, a 23-chapter
  tutorial whose every program the tests run.

Numbers: 36.2k lines of Nexium (25.4k of them the compiler and its tools),
2.0k of C (the runtime and the GUI window layer), 307 std functions,
32 examples, 21 tutorial programs, 37 spec conformance cases and 49
compile-fail cases, 13 harness suites, verified on three platforms by CI
and under the sanitizers.

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
Status: done except the `nx fmt` trailing-comma rule (`?T` chaining,
`a?.b`, landed in 1.1). `std.fs`, `std.time`, `std.regex`,
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

- Status (1.0.0): the compiler, every tool, the test harness and the
  fuzzer are Nexium; releases ship the Nexium `nx` built from the C seed;
  the stability policy and the platform tiers are written; the Rust crate
  is gone.

The rest of the climb, one release each (decision 90):

- **0.8, the Sickle.** The compiler in Nexium is *the* compiler: language
  changes land in `self/` and only there. The Rust crate is frozen at 0.7
  semantics and moves to `bootstrap/rust/`; it still builds, and is still
  the binary users run, because the tools are in it. The seed for a fresh
  machine is the C the Nexium compiler emits for itself, checked in as
  `bootstrap/nx.c`: a C compiler builds `nx0`, `nx0` builds `self/nx.nx`,
  the result builds itself, and the two agree. CI does this without
  `cargo`. The language suites (examples, spec, compile-fail) run through
  the Nexium-built `nx`; the oracle diffs that scaffolded the port
  retire. First change made in one compiler only: `unbounded_stack`.
- **0.9, Summit Ridge.** The tools in Nexium: `fmt`, `doc`, `lsp`, `ship`,
  packages, the REPL, the migrator, the installer generator, `doctor`,
  `version`. The shipped `nx` becomes the Nexium one. Nothing else new:
  the stability policy, fuzzing, the tier list.
- **1.0, Summit.** `bootstrap/rust/` is deleted. The repository is Nexium,
  one generated C file, and the runtime header. Done.

Exit: `cargo` is no longer needed to build `nx` from a release tarball
(0.8), and no Rust is left in the repository (1.0). Both reached.

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
  `nx fix` for mechanical migrations. Done (0.9): `docs/stability.md`,
  `nx fix`.
- Platforms: Linux aarch64 and Windows arm64 in releases; a tier list.
  Done (0.9): `docs/platforms.md`; both cross-compiled in every release.
- Security: a disclosure process is in `SECURITY.md` already; add fuzzing
  of the parser and the binary pattern engine to CI. Done (0.9):
  `tests/fuzz.nx` on every push.
- C toolchain support: which Zig versions `nx` is tested with, in CI and
  in the docs. Done (0.9): `docs/stability.md`, "The C toolchain". CI pins
  0.14.1; 0.16 differs in what its debug build traps and in which Apple
  SDKs it links against, and 0.7.0 found bugs that only one of them showed.

Every item is done: 1.0 is the release that deletes `bootstrap/rust/`.

## Past 1.0

### What 1.0 is not yet

The four things a careful reader will hold against 1.0, and where this
roadmap answers each:

| weakness | where it is answered |
| --- | --- |
| Memory safety is not guaranteed: ownership without a borrow checker still lets a view outlive its storage (two were found in `std` and the compiler under AddressSanitizer while the tutorial was written) | 1.2, the whole theme; until it lands the docs say "memory-safe by default" nowhere |
| Maturity: writing one tutorial found eight compiler bugs; a stranger will find more | the hardening commitments under *Always* below: the corpus, the fuzz budget, the sanitizer job, a patch within a week of a fix |
| No performance numbers: it compiles through C, which is not the same as a table | the numbers page, pulled forward from 1.6 into 1.1 |
| No ecosystem: sixteen std modules, no registry, no third-party packages, one maintainer | 1.4 (std), the registry and the second-maintainer items under *Ecosystem*; `async` is answered under *Not planned*, errors with payloads under *2.0 candidates* |

After 1.0 the language changes only by addition, under the stability policy
of phase 5. Each minor version has a theme; a bullet moves into a version
when it has a decision entry and a test plan. The compass sentence still
decides the order: what the compiler and the tools in `self/` and `std/`
needed first, what other people's programs need next. Everything here was
found by writing Nexium, not by reading other languages' feature lists.

### Before anything: the open bugs, and 1.0.3

`KNOWN_ISSUES.md` is this roadmap's page zero: every open bug lives there
with a reproduction and the fix it needs, a fix removes the entry, adds a
`Fixed` line and a regression test, and the next patch release ships it.
Nothing below is started while a known bug that a user can hit sits
there. (1.0.2 was the hotfix for the release binaries, which were built
for the runner's CPU.) The four entries an outside review of 1.0.1
added shipped in 1.0.3:

1. A contained panic leaked what the call acquired: the export wrapper's
   `longjmp` skipped every drop between the panic and the boundary. A
   per-call tracker in the runtime now releases live allocations, open
   files, sockets and held locks on the panic path; it is also the
   first half of 1.2's "a panic releases what it owned".
2. `nexium.lock` was written and never read, so the lockfile pinned
   nothing: `nx fetch` honours it, `nx update` is what re-resolves.
3. The Python wrapper took a list for a `[]mut T` parameter and dropped
   the writes: mutable slices accept only writable, contiguous,
   matching buffers.
4. Effect notes pointed at the function rather than the recorded
   witness; they point at the witness, and the compile-fail cases pin
   the line and column.

With them, the words the same review found wrong, also done:
`SECURITY.md` said "memory-safe" in one sentence and listed safe-code
use-after-free cases in the next; until 1.2 the claim is
"deterministically memory-managed without a garbage collector, memory
safety complete in 1.2", everywhere. The README says "language-stable,
early ecosystem" near the top. And for discovery: "Nexium language" in
every page title, and `nexium-lang` for the language's own packages on
PyPI and npm when they come, since `nexium` is taken there by unrelated
projects (decision 91).

### First: the quick wins, in order

Everything below is described in a theme further down; this is the same
work sorted by how soon it can ship. None of it changes the language, so
each lands as a `1.0.x` patch or in 1.1 the day it is done, in roughly
this order. The themes after this section keep their order; this is the
queue in front of them.

**Hours each.**

0. `nx doctor` compiles and runs a one-line program before it says
   "everything works": the checker, the C emitter, the C compiler, the
   linker and the executable all have to answer, and an illegal
   instruction from the probe is named as such with the fix. 1.0.1's
   doctor reported a working installation on a machine where the
   compiler could not compile a line, because it had only asked the C
   compiler for its version. The harness runs the check. (Shipped in
   1.0.3.)
1. The installer refuses to run twice (a setup mutex naming the one
   already open), closes a running `nx` before replacing files, and
   writes a log; the uninstaller the same. (Done, for the compiler's
   installer and the ones `nx ship` writes.)
2. The one-C-file install on the front page of the docs, and
   `install.sh` falling back to `cc bootstrap/nx.c` when there is
   nothing to download. (Done; `NEXIUM_FROM_SOURCE=1` asks for it.)
3. The release workflow writes a `.torrent` with the assets as web seeds
   and puts the magnet link in the notes; a QR code beside it. (Done:
   one torrent per file, from 1.0.4 on.)
4. A Homebrew tap, a Scoop bucket and the winget manifest, from the
   archives the release already builds. (Done: the repository is the tap
   and the bucket, `scripts/packaging.py` writes all three at each
   release; the winget submission to microsoft/winget-pkgs is a pull
   request by hand.)
5. `nx doctor` says when a newer release exists. (Done.)
6. The VS Code extension on Open VSX beside the Marketplace. (Done in
   the release workflow: both publishes run when `VSCE_PAT` and
   `OVSX_PAT` are set; the tokens and the namespaces are the
   maintainer's to create, see `editors/vscode/README.md`.)
7. A `packages.md`: the packages people can `nx add` from git today.
   (Done, as a section of `docs/packages.md`: the honest list is the
   Topo's two example packages, and how to get a row.)
8. An "Open REPL here" folder entry and a Windows Terminal profile from
   the installer. (Done.)
9. `ghcr.io/londopy/nexium`: the compiler with Zig, on Alpine and Debian.
   (Done: `docker/`, published by the Docker workflow at each tag, for
   amd64 and arm64; CI builds and runs both on every push.)

**A day or two each.**

10. `install.ps1`, the PowerShell one-liner, and the Chocolatey package
    that wraps it. (Done; the package wraps the installer with its
    checksum, as Chocolatey's moderation asks, and the push needs the
    maintainer's `CHOCO_API_KEY`.)
11. REPL `:undo`, `:save` and `:load`; `:effects expr`. (Done; `:load`
    was there already.)
12. `nx -e` and `nx -p` one-liners on the REPL's compile cache. (Done,
    on the interpreter: nothing is compiled, so there is no cache to
    keep.)
13. `nx layout Type`: offsets, sizes, padding, the reordering. (Done:
    `nx layout FILE [Type...]`.)
14. `expect_snapshot` in `std.testing`. (Done, beside `snapshot`.)
15. `pip install nexium` and `npm install nexium`, wheels and packages
    that carry the binary. (Done as `nexium-lang`, decision 91:
    `scripts/pypi_npm.py` builds them from the release archives, the
    release attaches them and uploads when `PYPI_TOKEN` and `NPM_TOKEN`
    are set.)
16. The installer detects an installed version: upgrade, repair, remove,
    the previous choices as defaults. (Done in the compiler's installer;
    the ones `nx ship` writes keep Inno's own upgrade-in-place.)
17. `nx upgrade`, and `nxup` behind it later. (`nx upgrade` done: the
    release for the machine, verified, over the running executable; a
    separate `nxup` when one is needed.)
18. Portable mode, and `nx install` from a portable copy. (Done: a
    `portable` marker keeps Zig's cache beside nx, `nx install [DIR]`
    copies the folder into the user's place and onto the PATH.)
19. The effects lockfile (`nx audit --lock`) and the CI check. (Done:
    `file.effects.lock`, `--check` in CI on the shipped example.)
20. Panic proofs as code lenses in the language server. (Done.)
21. Colour as you type and completion in the REPL, from the server.
    (Done: a line editor on raw terminal input, the server's completion
    at the cursor, history; `NX_PLAIN=1` for the plain reader.)

**About a week each.**

22. `nx explain f effect`: the provenance tree. (Done.)
23. The numbers page: `bench/` in four languages, on a fixed runner,
    published. (Done: `bench/` in Nexium, C, Rust, Go and Python, the
    Bench workflow on a GitHub runner weekly and at tags, the medians on
    `docs/numbers.md`, a regression a failing job.)
24. `nx test --watch` and `nx run --watch` (file watching in the
    runtime). (Done, polling the program's files twice a second; a
    watcher in the runtime can replace the poll when one is wanted.)
25. The ownership trace, `nx run --trace own` and the REPL's `:own`.
26. The binary-pattern debugger, `nx bin`.
27. `:show` and `:plot` in the REPL: the inspector and chart windows on
    nexium-gui.
28. The installer's editor page, and the wizard in six languages.
29. Authenticode signing and macOS notarization in the release workflow
    (the certificates are the slow part).

Everything longer, in the order of the themes: memory safety (1.2),
incremental builds and the semantic language server (1.3), `std.tui`
and `nx topo` (1.4), the wasm playground (1.5), hot reload, profiling by
effect, the visual tools (1.6), the seam (1.7), nexium-gui grown up and
the Hut (1.8).

### 1.1: the language the compiler wanted

Ergonomics the self-hosted compiler paid for by hand.

- `?T` chaining: `a?.b` is `null` when `a` is (deferred since phase 1).
  (Done: the rest of a chain applies to the payload, `a?.name.len`.)
- Tuple destructuring: `let (a, b) = pair`, in `for` bindings too.
  (Done: views over a place, owners over a temporary, decision 92.)
- `derive(Clone)` for structs and enums whose fields all clone, the way
  drops are derived (KNOWN_ISSUES: every deep copy in `check.nx` is a hand
  written function). (Done, decision 93; `check.nx` lost `cv_clone` and
  `frames_clone`.)
- Iterators: `for x in v` over any value with a `next(self: *mut Self) -> ?T`
  method, so a user type iterates like a slice; `Map` iteration over
  entries `(k, v)` as well as keys. (Done, decision 94.)
- Slice patterns: `[first, rest..]`, `[a, b]`, and `x @ pat` bindings in
  `match`. (Done, decision 95.)
- Range facts through `while` conditions and `else` branches (KNOWN_ISSUES),
  and proofs for `x / y` after `y != 0`, `.?` after `x != null`, and slice
  bounds after `if s.len >= n`, so fewer functions carry `panics`. (Done,
  decision 96.)
- Format width from a value (`{:>w}`) and named placeholders with an
  anonymous literal (`format("{x}", .{ .x = 1 })`). (Done, decision 97.)
- Intrinsics still missing from a systems language: `@bitCast`, `@min`,
  `@max`, `@alignOf`, `@target()` (os, arch, pointer width) for compile-time
  `if` in std.
- `unbounded_stack`: the spec says reserved. Either a recursion-depth
  proof (a function is bounded when every recursive call is on a strictly
  smaller argument) or removal, the way decision 88 settled the others.
- The numbers page (pulled forward from 1.6): `bench/`, the programs the
  examples already implement written the same way in C, Rust, Go and
  Python, run by CI on a fixed runner, the medians published on the site
  with the compiler versions and the machine, and a regression a failing
  check. Before 1.2, so the memory-safety work is measured against a
  number rather than a feeling.

Exit: the compile-time interpreter and `check.nx` lose their hand-written
copies and index workarounds; `nx audit self/check.nx` reports fewer
`panics` than in 1.0.

### 1.2: memory safety without a garbage collector

The promise of section 12, no undefined behaviour in safe code, has four
holes that 5.6 and 5.7 hand to the programmer: a view stored past the
storage it points into, a view kept across a growth of its container, a
view into a value that is then moved or dropped, and a value that leaves
its arena. 1.2 closes them, so that ordinary safe Nexium code cannot
create a dangling view and `unsafe` marks everything the compiler cannot
prove: memory-safe by default, with explicit escape hatches, the
guarantee Rust gives, kept in Nexium's simpler shape.

Not a garbage collector. A GC would simplify the language and weaken
several of its defining promises at once: predictable destruction,
visible allocation behaviour, `!allocates`, embedding in C, Python, Rust
and Node processes, no substantial runtime. And not a borrow checker
with lifetime annotations either: the language keeps ownership without
one. What it keeps, and what it adds:

- Move semantics and destruction at scope exit stay as they are; so do
  reference counting for shared `ref class` objects and `weak` for cycles.
- The checker learns where every view points (its *origin*: a local, a
  parameter, a container, an arena) and, per function, whether the view is
  still used later (liveness, not lifetimes). The rules are decided one
  entry each and enforced by the same pass:
  - **V1** (R1 today): a function may not return a view into its own
    locals.
  - **V2, storing**: a view may be stored in a variable, a field, an
    element or a closure capture only when the storage it points into
    outlives that place; `out = s[..]` with `s` a local of an inner block
    is an error, and so is a closure capturing such a view and escaping.
  - **V3, growth**: while a view into a `List`, `String` or `Map` is live,
    the container may not be grown, cleared, reassigned or moved:
    `append`, `insert`, `put`, `clear`, `=` and passing it by value are
    errors until the view's last use. The same for `*mut T` pointers
    obtained with `&mut`.
  - **V4, moving**: a view into a value is dead once the value moves or
    drops; a use of the view after that is the `use after move` error the
    language already has, applied to views.
  - **V5, arenas**: a value allocated inside `using arena { }` may not be
    stored in a place that outlives the block: not returned, not assigned
    outward, not appended to an outer container. The decision entry
    settles how results leave a block (a heap copy made explicitly, or
    numbers and other values that carry no allocation).
- `unsafe` is required for what the checker cannot prove: raw pointers
  from `@cImport`, casts to `*mut`, a view the programmer knows outlives
  its origin. Every `unsafe` block in `std/` and `self/` carries a reason
  comment, and `nx audit` lists them with it.
- Threads and data races stay outside the promise (section 13): memory
  safety is not race freedom. The scoped threads of 1.4 (joined when the
  block ends, no handle escapes) remove the one way a thread could outlive
  the storage its argument points into.
- Leak detection stays. Reference cycles that `weak` does not break may
  still leak; a leak is not a memory-safety violation, and `nx leaks`
  reports it.
- The diagnostics name the origin and the moment it dies: "`out` keeps a
  view into `s`, which is dropped at the end of the block on line 12;
  clone it, or declare `s` where `out` lives". `nx fix` inserts the
  `.clone()` where that is the fix.

Rolling it out under the stability policy: undefined behaviour was never
promised, so a program that had it may become an error in a minor
(`docs/stability.md`). Each rule arrives as a warning in one release and
an error in the next, so a codebase gets a release to run `nx fix`.

Exit: sections 5.6 and 5.7 list no case that is the programmer's
responsibility; `SECURITY.md` extends its promise to views; every rule
has a compile-fail case and a spec case; the fuzzer gains a hunter that
compiles mutants in debug mode (freed storage filled with `0xDD`) and
runs them, so a dangling view that slips through is a finding rather than
luck.

### 1.3: the toolchain grown up

- First, `self/check.nx` split by responsibility (declarations, types,
  ownership and moves, effects, patterns, generics, the compile-time
  interpreter, diagnostics), even while every module still compiles into
  one unit: twelve thousand lines in one file is the ceiling the rest of
  this theme would otherwise hit.
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
  suggests it, `.clone()`, `_ =`, the view rules' fixes from 1.2.
- `nx bench`: `bench "name" { }` blocks with warmup, iterations and
  medians, in the same file as tests.
- `nx build --sanitize address,undefined` through the C compiler, and
  `nx test --sanitize` in this repository's CI.
- Conditional compilation: `if comptime @target().os == "windows" { }` in
  std replaces the runtime's `#ifdef`s one by one.

Exit: a one-line change rebuilds in well under a second on the compiler's
own sources, and the language server answers from the checker.

### 1.4: a standard library people stop supplementing

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

### 1.5: platforms

- WebAssembly: `--target wasm32-wasi` and `wasm32-freestanding`, the
  runtime's process, socket and thread code behind `@target()`, and an
  `artifact wasm` producing a `.wasm` with a JavaScript loader. The
  self-hosted compiler compiled to wasm is the playground: `nx` running in
  a browser, no server.
- nexium-gui: X11 and Wayland, Cocoa backends beside Win32; the demo runs
  on all three and in the browser through a canvas backend.
- Static Linux binaries (musl), FreeBSD, and the tier list extended;
  Linux aarch64 and Windows arm64 promoted to tier 1 when CI runs them.
- 32-bit architectures: `i686` Windows and Linux, `armv7` Linux (the
  Raspberry Pi OS that is still 32-bit, routers, older phones), `riscv32`
  and `thumb` alongside the embedded targets below. Everything in the
  language already has a width (`isize`/`usize` are the pointer's,
  `@sizeOf` is per target, C's `long` is already mapped per platform),
  so the work is in the places 64 bits were assumed: the runtime's
  handles and sizes (`int64_t` where `intptr_t` was meant, `size_t`
  arithmetic, `nx_time` and the file offsets staying 64-bit on purpose),
  the binary pattern engine's 64-bit segments on a 32-bit word, the
  compile-time interpreter evaluating `usize` at the *target's* width
  rather than the host's, `@target().pointer_width` for std, the C
  header import mapping `long` and `size_t` per target, the leak and
  size reports, and the ABI document's word about `usize` at the
  boundary. A `--target i686-linux-gnu` build of every spec case and
  every tutorial program in CI (32-bit executables run on the 64-bit
  runners), the compiler itself built and self-hosted as 32-bit, and
  the tiers extended: tier 2 for `i686` and `armv7`, tier 1 when a runner
  runs them.
- Cross-compilation matrix in `nx ship`: every target the C toolchain
  supports, from one machine, tested in CI for the tier-1 set.
- Embedded targets: `-Os` builds without the runtime's file, socket and
  thread parts (`@target().os == "none"`), the first program on a
  microcontroller.

Exit: a Nexium program runs in the browser, on a Raspberry Pi and on the
three desktops from one source, and the docs say which combinations CI
proves.

### 1.6: the runtime that release builds deserve

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
- The numbers page of 1.1 is the measure: each item above lands with its
  before-and-after row.

Exit: every example in release mode is within a documented factor of its
C counterpart, and the factor does not grow between releases.

### 1.7: the seam, both ways

Today a Nexium library leaves the tree through `nx ship`: a C header and
archive, a Python wheel over `ctypes`, a Rust crate with safe wrappers, an
npm package. What crosses is what C carries: integers, floats, slices,
`layout(c)` structs, a status code with an error name. Everything else
stays home, and the traffic runs one way. This theme widens the seam in
both directions, with the same rule as 1.0's spec section 15: a panic
never crosses, no initialization call, no process-global state, and a
symbol means the same thing from every language.

**Python, deeper.**

- Strings and bytes as values: `[]u8` and `String` parameters take `str`
  and `bytes`, returns come back as `str`; `[]String` as a list of them.
- The buffer protocol without copies: a `[]f64` parameter takes a NumPy
  array, `memoryview` or anything contiguous by pointer, and a `[]mut f64`
  writes through; a two-dimensional `[][]f64` view maps to a C-contiguous
  array with its shape. Today `array.array` and `bytearray` are the
  zero-copy cases and a sequence is copied.
- Records, enums and optionals: `layout(c)` structs arrive as dataclasses
  with the same fields, an `enum` as an `IntEnum`, `?T` as `None` or the
  value, an error set as an exception class per error name
  (`ropesim.InvalidInput`, a subclass of `NexiumError`), so `except` can
  name the one it handles.
- Objects: an export that returns a `ref class` hands Python an opaque
  handle whose `__del__` releases it; the class's `impl` methods become
  methods of the handle, so a stateful library (a parser, a simulation)
  is a Python class.
- Callbacks: a parameter of function type on an export becomes a Python
  callable; the compiler emits the `CFUNCTYPE` shape and the marshalling,
  and the call carries the `ffi` effect on the Nexium side.
- A second wheel shape, `artifact python { mode = "extension" }`: a
  CPython extension module against the limited API (`abi3`), one wheel
  per platform, calls an order of magnitude cheaper than `ctypes` for
  small functions; the `ctypes` wheel stays the default because it needs
  no Python headers to build.
- Python inside Nexium: `@cImport("Python.h")` works today; a `python`
  package (registry, not `std`) wraps the C API with typed conversions,
  `py.run`, `py.call(module, name, args)`, and the interpreter's lock as a
  `using` block, so a Nexium tool can call a Python library without
  shipping a wheel first.

**Rust, deeper.**

- Error sets as enums: `error Parse { Bad, Truncated }` becomes
  `ropesim::ParseError { Bad, Truncated }` and the export returns
  `Result<T, ParseError>`; `NexiumError` stays for panics and for exports
  that widen to `!T`.
- Strings: `&str` and `String` cross as `[]u8` and `String`; `&[u8]` as
  `[]u8`; a returned `String` arrives owned, freed by a `Drop` that calls
  the archive's free.
- Objects: a `ref class` return becomes an opaque struct with `Drop`, and
  its `impl` methods become inherent methods; `Send` is derived from the
  absence of `shared_mutable` on every method.
- `#![no_std]` crates when no export allocates (the runtime split so the
  archive's allocation, file, socket and thread parts are separable, which
  1.5's embedded targets need too), so a Nexium library can sit inside a
  Rust firmware.
- Source crates: the crate's `build.rs` rebuilds the archive from the
  `.nx` sources when `nx` is on the `PATH`, so `cargo build` after a
  change to the Nexium is enough, and the crate can be published to
  crates.io as source.
- Rust inside Nexium: `nx add` of a Rust crate that exposes `extern "C"`
  functions builds it with `cargo` into a static library, runs `cbindgen`
  for the header and binds it through `@cImport`, so a Nexium program can
  use a Rust crate without writing the bridge by hand.

**More languages out.** Each is an `artifact` kind with a wrapper in that
language's idiom over the same C ABI, a test in CI that calls `ropesim`
from it, and a chapter in the embedding guide:

- C++: the header compiles as C++ today; add `ropesim.hpp` with RAII
  wrappers over handles, `std::span` and `std::string_view` overloads, and
  results that follow `std::expected`.
- Go through cgo: a package with the header and archive and Go-typed
  wrappers, errors as `error` values with the Nexium name.
- Java and Kotlin through Panama (`jextract` over the header) with a
  small hand-written layer for slices and errors; JNI only where Panama
  is not available.
- C# through P/Invoke: a `DllImport` wrapper and a NuGet package with the
  native archive per runtime identifier.
- Ruby through `fiddle`, Lua as a C module, Swift through a module map
  over the header; Zig through `@cImport` of the header, which works today
  and needs a page.
- The browser: the `wasm` artifact of 1.5, with a TypeScript declaration
  file generated from the exports.

**The ABI itself.**

- `docs/abi.md`, versioned: the C shape of every crossing type, the
  status convention, who frees what, which thread may call what, and how
  a handle is retained and released, so a wrapper can be written for a
  language this list does not have.
- Buffers that leave: an export may return a `String` or a `List` of a
  crossing type; the header carries `<name>_free`, the wrappers call it,
  and the leak detector counts what the host never freed.
- `nx ship --abi-check`: the header and the export list are compared with
  the previous release's and a removed or narrowed export is reported, so
  the version number of a shipped library follows semver by construction.
- `@cImport` grows the C it accepts: object-like macros with values,
  function-like macros as inline functions, bitfields, variadic
  declarations, and a `[c]` section of `nexium.toml` that vendors and
  builds a C library (`nx add --c sqlite`) so `@cImport` finds it.

Exit: `ropesim` shipped to C++, Go, Java, C#, Ruby, Lua and the browser
and called from each in CI; the Python wheel takes a NumPy array without a
copy and raises `ropesim.InvalidInput`; the Rust crate's error enums come
from the Nexium error sets; an example calls a Rust crate and a Python
library from Nexium; `--abi-check` fails on a removed export.

### Tools only this language can have

The REPL was the first of these: a feature no systems language is
expected to have, cheap because the compiler was already there. Each item
below is the same shape, built on a fact the compiler already knows
(every function's effects, every move and drop, every binary pattern's
shape) and offered as a tool. Each is pencilled into the minor it fits;
the cheap ones may land earlier as patches to the tools, since none
changes the language.

**Answers from the effect system.**

- `nx explain f allocates` (1.3, done on the way): the provenance of an
  effect, the call chain from `f` down to the primitive that brings it
  in, as a tree.
- An effects lockfile (1.3, done on the way): `nx audit --lock` writes
  each function's effects to `file.effects.lock`; a change that adds an
  effect (a function starts allocating, can now panic) fails `nx audit
  --check`, and CI, until the lock is updated on purpose. Semver for
  behaviour.
- Proofs in the editor (1.3): the code lens above every function reads
  `cannot panic` or `panics: index at :12` (done on the way); hover on a
  slice index saying which fact proved the bound is still to come.
- Profile by effect (1.6): `nx run --trace allocates` instruments only the
  allocation sites, `--trace blocks` only the blocking calls, and prints
  a histogram per function: a profiler with no configuration, from the
  effect system.
- Record and replay (1.3): a function without `nondeterministic` is pure
  with respect to time, random and the environment, so `nx test --record`
  knows exactly which calls to capture and `--replay` makes a flaky test
  reproducible.

**Answers from ownership and layout.**

- `nx layout Type` (1.3): field offsets, sizes, padding bytes and the
  reordering that removes them, for any struct; `layout(c)` structs shown
  as the C compiler will see them.
- The ownership trace (1.3): `nx run --trace own` prints every move,
  clone, retain, release and drop with its source span, and the REPL's
  `:own` shows them for the line just entered. Ownership without a borrow
  checker, made visible.
- Ask the checker (1.3): `:facts x` in the REPL, and a hover in the
  editor, print the range facts and proofs the checker holds about a
  value at that point (`x < xs.len`, `y != 0`), so "why is this bounds
  check still here" has an answer.

**Binary patterns.**

- A binary-pattern debugger (1.3): `nx bin parse_png file.png` runs a
  `<<...>>` pattern from the source against a real file and prints which
  bytes bound to which field, and where a match failed.
- A binary workbench (1.4): the same as a terminal view, a hex dump on
  one side and a pattern on the other, the match updating as the pattern
  is edited; `:bin` in the REPL renders a pattern's bytes.

**Interactive, in the family of the REPL.**

- Hot reload (1.3): `nx run --hot` rebuilds on save and swaps the changed
  functions into the running program through a shared library, keeping
  the state. The effect system says which swaps are safe (a function
  without `shared_mutable` touches no global); the GUI library, being
  immediate mode, redraws with the new `frame` on the next tick, so a
  window is edited live the way a web page is.
- One-liners (1.1): `nx -e 'for l in io.lines() { ... }'` and `nx -p
  'expr'` for shell pipelines, on the REPL's compile cache, so Nexium
  takes the awk seat as well as the C seat.
- REPL sessions as files (1.1): `:undo` pops the last line (the session
  is a program the REPL recompiles), `:save f.nx` writes it as a script
  with the outputs as comments, `:load f.nx` resumes it, `:effects expr`
  prints the effects an expression carries.
- Watch mode (1.3): `nx test --watch` reruns on save only the tests whose
  dependencies changed, from the module graph the incremental build keeps;
  `nx run --watch` for programs.
- Snapshot tests (1.4): `expect_snapshot(value)` in `std.testing` writes
  and compares `.snap` files, the mechanism this repository uses for its
  own examples, offered to every project.
- A Jupyter kernel (ecosystem): the REPL behind the notebook protocol, so
  a Nexium cell runs beside Python ones; the effect chips as cell
  badges.
- The playground (1.5): the compiler compiled to WebAssembly puts a "run"
  button on every code block of the documentation site and the Topo.

### The clickable line: windows, wizards and the REPL

The installer wizard, the REPL and the GUI library are the parts of the
project people touch rather than read. This line grows them together:
each item is something to click, type into or look at, and each stands
on what exists (the REPL recompiles a session; nexium-gui draws in a
window or headless; the installer is a wizard already). Pencilled into
minors the same way as the tools above.

**The REPL you can look into** (1.1, 1.3).

- Colour as you type, completion and signatures from the language server,
  history search, multi-line editing, `:doc List.append` inline.
- `:show value` opens an inspector window: a struct as a tree, a `List`
  as rows, a `Map` as a table, live while the session continues, so a
  value is looked at rather than printed. `:plot xs` opens a chart window
  for a list of numbers; `:watch expr` re-evaluates and redraws on every
  line entered. The REPL and the GUI library, fused.

**Run from the editor, everywhere** (1.3). Ctrl+B runs the file, the
variant tests it, and a diagnostic is a click away, in every editor the
repository supports, the same way:

- Sublime Text: `Nexium.sublime-build` (done: run, test, check, build,
  effects; the `--> file:line:col` line is clickable). Vim and Neovim:
  `:make` and the `:Nx` commands (done). Emacs: `C-c C-r` and friends
  (done). Still to write: VS Code tasks with a problem matcher
  (`tasks.json` shipped by the extension, so Ctrl+Shift+B works without
  setup), a Zed `tasks.json`, Helix key bindings for `:sh nx run %`,
  Kate's build plugin target, JetBrains external tools and Notepad++
  NppExec scripts as files to import rather than steps to type.
- One machine-readable diagnostic format for all of them: `nx check
  --format short` prints `file:line:col: error: message` on one line
  (today's two-line `error: ... / --> file:line:col` needs a two-line
  matcher, which Sublime and Vim's `errorformat` manage and simpler
  problem matchers do not), and `--format json` for the tools that want
  the notes and the spans.
- The installer's editor page (below) installs the build systems with
  the syntaxes.

**The first five minutes** (1.3, 1.5).

- The installer's editor page: a checkbox per editor found on the
  machine (VS Code, Sublime Text, Vim and Neovim, Notepad++, Emacs, Kate,
  JetBrains and the rest), each putting the files from `editors/` where
  that editor looks: Sublime's syntax into `Packages\User`, Notepad++'s
  UDL into `userDefineLangs`, the Vim files into `vimfiles`, the VS Code
  `.vsix` through `code`; the boxes are ticked for the editors that are
  installed and greyed with a reason for the ones that are not. An
  "Open REPL here" entry in the folder context menu and a Windows Terminal
  profile; a finish page that opens the first chapter of the Topo. The
  install script does the same on macOS and Linux (`~/.config/sublime-text`,
  `~/.vim`, `~/.config/nvim`, `~/.emacs.d`), asking first.
- `nx upgrade`: fetches the new release, verifies the checksum, swaps
  itself; `nx doctor` says when one is available.
- A macOS `.pkg` and a Linux `.deb`/`.rpm` with the same pages (1.5), and
  manifests for `winget`, Homebrew and Scoop, so the install is one line
  where people already type them.

**The installer grown up** (1.3). The wizard is the first thing a Windows
user sees of the language, so it should behave like software people pay
for, in every situation it can find itself in:

- One at a time: a second setup started while one is running gets a
  message naming the one already open (a setup mutex) rather than two
  wizards racing for the same directory; the uninstaller the same.
- Knows what is there: detects an installed version and offers *Upgrade*
  (keeps the choices made last time: directory, PATH, components,
  editors), *Repair* and *Remove*; refuses to downgrade without saying
  so; notices a per-user install when installing for all users, and the
  reverse, and offers to remove the other first; reads the settings of a
  previous install for its defaults.
- Nothing in use: finds a running `nx`, REPL or language server and
  asks to close it (or lets the user do it) before files are replaced,
  with a restart-manager retry instead of a failed copy; warns when a
  terminal has `%LocalAppData%\Programs\Nexium` on its PATH and will not
  see the new `nx` until reopened.
- Checks before it starts: disk space including the bundled Zig, a
  pending reboot, a PATH already near the length limit, an antivirus
  quarantine of `zig.exe` after the copy (verifies the files it wrote and
  says which one is missing), and whether `code`, `git` and the editors
  of the editor page are present, so a box is unchecked with a reason
  rather than failing later.
- Choice of toolchain: bundle Zig (default), use a Zig already on the
  PATH, or download it at the end with a progress bar and a checksum;
  the compact type without Zig for people who have a C compiler.
- Speaks the reader's language: the wizard in the six languages of
  `docs/i18n`, chosen from the system locale.
- Signed: an Authenticode signature on the setup and on `nx.exe`, so
  SmartScreen shows the publisher rather than a warning; the release
  workflow signs from a certificate in the repository's secrets.
- Leaves a trail: a log under `%Temp%` named from the wizard's last
  page, an *Installation failed* page with the log and a link to file an
  issue, and rollback of a half-finished install.
- For administrators: `/VERYSILENT` with every page's choice as a
  parameter (`/COMPONENTS`, `/TASKS`, `/EDITORS`, `/DIR`, `/ALLUSERS`),
  an MSI wrapper for group policy, and an ARM64 setup beside the x64 one.
- After the install: *Check for updates* in the Start menu entry and a
  notice from `nx doctor`; uninstall that offers to keep `~/.nexium`
  (packages, REPL history) or remove everything; a repair that reinstalls
  only the files whose checksum differs.

**Every road in** (1.3 to 1.5). Today `nx` arrives by the Windows setup,
the install script, the portable archives, source, and the GitHub
Action. People install software the way their platform taught them, so
each road below is a manifest or a package built by the release
workflow from the same archives, in the order of how many people stand
on it. The community usually maintains the distribution-owned ones
(AUR, nixpkgs, Homebrew core); the project's job is a release that makes
that easy: stable URLs, checksums, and no post-install step.

- Windows: `winget install Londopy.Nexium`, Chocolatey, Scoop (a bucket
  in the repository), an MSI for group policy, an MSIX for the Microsoft
  Store, and the ARM64 setup.
- macOS: a Homebrew tap (`brew install londopy/tap/nexium`) then core,
  MacPorts, the signed and notarized `.pkg`, and a universal binary.
- Linux: `.deb` and `.rpm` with an apt and a dnf repository so upgrades
  come with the system's; a PPA, a Fedora COPR and an openSUSE OBS
  project; the AUR; Alpine (`apk`, musl builds of 1.5); Nix (a
  `flake.nix` in the repository, then nixpkgs) and Guix; Snap; Gentoo
  and Void templates. Flatpak and AppImage for the Hut, not the compiler.
- BSD: FreeBSD and OpenBSD ports once 1.5's tiers include them.
- Containers: `ghcr.io/londopy/nexium` (the compiler on Alpine and on
  Debian, with Zig), a dev container feature, so Codespaces and Gitpod
  have `nx` in one line, and a `setup-nexium`-style step for GitLab CI.
- Carried by other ecosystems: `pip install nexium` (a wheel that ships
  the binary, the way Zig and Ruff ship on PyPI) so a Python project's
  `nx ship` wheel builds in its own venv; `npm install nexium` and `npx
  nx` the way esbuild ships; conda-forge; Composer, RubyGems and Cargo
  carriers only when 1.7's artifacts for those languages exist.
- Version managers: an `asdf`/`mise` plugin, `pkgx`, and `nxup`, the
  project's own: several versions side by side, `nx +1.0.1 run`, a
  `nexium.toml` pin honoured automatically, `nxup update`; `nx upgrade`
  above is `nxup` for people with one version.
- Editors and services: the VS Code Marketplace and Open VSX (the
  extension is published to the first; the second is where VSCodium and
  Gitpod look), the Zed and Neovim registries once the language server
  is semantic, a pre-commit hook for `nx fmt`.
- Elsewhere: Termux on Android, the playground in the browser (no
  install at all), and a bootable "try it" image is not planned.
- For programs written in Nexium: every channel above that takes a
  binary becomes a target of the `installer` artifact (`nx ship
  --winget`, `--brew`, `--deb`, `--docker`), so a Nexium program reaches
  its users on the roads the compiler took.

**The niche roads** (1.3 onward). The ones people remember:

- The PowerShell line: `irm https://londopy.github.io/nexium/install.ps1 | iex`,
  the Windows twin of `curl | sh`, which does not exist yet: it downloads
  the portable zip or the setup, verifies the checksum, runs the setup
  silently (`/VERYSILENT /TASKS=addtopath`) or unzips to
  `%LocalAppData%\Programs\Nexium`, and fixes the PATH for the current
  session too. The Chocolatey package is this script inside a `.nupkg`,
  which is why Chocolatey installs feel like magic: `choco install` is
  PowerShell all the way down.
- One C file is the installer: the compiler is `bootstrap/nx.c`, so on a
  machine with nothing but a C compiler,
  `curl -fsSL .../bootstrap/nx.c | cc -x c - -o nx -lm -lpthread`
  installs Nexium. No other language can be installed by compiling one
  file; the docs should say so on the front page, and `install.sh`
  should fall back to it when there is no download.
- Torrents and magnets: the release workflow writes a `.torrent` per
  release with the GitHub assets as web seeds and publishes the magnet
  link in the notes, the way Linux distributions ship images; the
  checksums are in the file, so a swarm cannot serve a tampered
  binary. Also an IPFS pin of the same archives (`ipfs://`), for the
  people who have IPFS Companion.
- Self-installing `nx`: `nx install` from a portable copy performs the
  setup's tasks without the setup (PATH, file association, Start menu,
  the editor files), and `nx uninstall` reverses them; the portable zip
  and the setup become the same thing with a choice.
- Portable mode: a `portable` file next to `nx.exe` keeps packages, the
  REPL history and the Zig cache beside it, so a USB stick carries the
  whole toolchain and leaves nothing on the host.
- MSIX with an `.appinstaller` file: a link that installs and then
  updates itself from the release feed, no store account needed.
- `gh release download Londopy/nexium -p 'nx-*-x86_64-unknown-linux-gnu.tar.gz'`
  documented for the people who live in the GitHub CLI; Ansible, Chef
  and Puppet modules that wrap the install script and pin a version, for
  the people who install onto a hundred machines.
- A QR code on the release page and the site's install section that
  opens the install page on a phone, for Termux, and for the person
  standing at someone else's laptop.

**`nx topo`: the tutorial you can run** (1.4).

- An interactive tutorial runner: opens a chapter, shows its program, lets
  you edit and run it, checks the output against the recorded one, gives
  exercises with hints and keeps your progress. In the terminal first,
  and on the site once the playground (1.5) runs code in the page. The
  book becomes a course.

**The Topo as a course** (1.4 for the content, 1.5 for the page). The
chapters teach by showing; a course asks the reader to do. Every exercise
is a file in the repository the harness runs, so the course cannot rot:

- Exercises at the end of every chapter, three kinds, all checked by the
  compiler rather than by an answer key: *fill in the blank* (a program
  with a hole marked `???` that must compile and print the recorded
  output), *fix the error* (a program with a diagnostic the reader must
  clear; the Topo's `_fails.nx` files already are these), and *write it*
  (a spec of a function and a test file that must pass). Solutions ship
  beside them, folded away on the site.
- Quizzes the compiler grades: "does this function allocate?", "which
  line moves `s`?", "what does this print?", "which effect does `main`
  carry?", with the answer checked against `nx effects`, the ownership
  trace and the recorded output, so a wrong answer explains itself with
  the compiler's own words. Multiple choice on the page, the same
  questions as `nx topo quiz` in the terminal.
- Predict-the-output cards: the code shown, the output hidden until the
  reader commits to a guess.
- In the page, once the compiler runs in the browser (1.5): every code
  block editable with a *Run* button, the exercises and quizzes graded
  in place, a *reset* to the book's version, and a link that carries the
  reader's code (the playground's share URL). Until then the same
  exercises run in the terminal through `nx topo`.
- Progress: the route map of the Topo with each pitch ticked as its
  exercises pass, kept in the browser (nothing to sign into) and by
  `nx topo` in `~/.nexium`; a chapter shows what the next one needs.
- More chapters, each a project people actually want to write: a log
  analyser (text processing at scale), a game on nexium-gui, an HTTP
  server with routing and JSON, SQLite through `@cImport`, a tiny
  language (lexer, parser, interpreter: the compiler in miniature), a
  program for the browser (wasm, 1.5), "port a Python script", and
  "profile and speed up" once the numbers page exists. Each chapter ends
  with *common mistakes*, the diagnostics a newcomer meets first.

**The docs, more** (1.3 to 1.5).

- A reference page per std module generated by `nx doc` from the doc
  comments, with every function's signature, effects and an example that
  the harness runs, replacing the one long `docs/std.md`.
- The error index: every diagnostic the compiler can emit has a page with
  a wrong program, the message, why, and the fix; `nx explain E0042` (or
  the message's own words) opens it, and the language server links to
  it from the squiggle.
- A cookbook: "how do I..." recipes (read a file line by line, parse
  JSON, talk to a socket, call a C library, ship to Python), each a
  complete program the tests run; a page of Nexium next to Python, Rust,
  Go and C for the same twenty tasks, for the reader arriving from one
  of them.
- Guides that the reference cannot be: effects in depth (how inference
  works, how to discharge `panics`, what an `unsafe` block promises),
  ownership without a borrow checker (the rules, the diagnostics, the
  patterns), performance (what allocates, what the C looks like, `nx
  size` and `nx audit`), and the ABI (1.7).
- A style guide (what `nx fmt` decides and what it leaves to you), a
  glossary, an FAQ, and a "Nexium in five minutes" page for the reader
  who will not read the Topo.
- The site: search (a client-side index built by `site/build.nx`), a
  light/dark toggle beside the system setting, docs per release (`/1.0/`,
  `/latest/`), a printable Topo (one page, and a PDF and an EPUB built by
  CI), and "edit this page" on every page.

**`std.tui`** (1.4).

- Raw mode, key and mouse events, cells, colours and layout: the base for
  the binary workbench, `nx topo`, and `nx dash`, a project dashboard
  (tests, effects, size, leaks, live, keyboard-driven).

**Visual tools** (1.6).

- `nx size --gui`: the binary as a treemap by function, click to zoom.
- `nx leaks --gui`: the unreleased objects with their allocation stacks.
- `nx effects --gui`: the call graph with effect-coloured nodes; click one
  for the provenance `nx explain` prints.
- The ownership timeline: a run's moves, clones and drops on a scrubber,
  the source highlighted as it plays; chapter 8 of the Topo as an app.
- All built on nexium-gui: the library's own stress test.

**nexium-gui grows up** (1.8, Gangapurna).

- Anti-aliased TrueType text from a rasterizer in Nexium, Unicode,
  clipping and scrolling containers, tree, table, tabs and menus, multiple
  windows, the platform's text input (IME), high-DPI; the X11, Wayland
  and Cocoa backends of 1.5 underneath.
- `nx gui gallery`: every widget with its source beside it, clickable
  documentation.

**Apps** (1.8).

- The Hut, a small native IDE in nexium-gui: an editor with the language
  server, run and test buttons, an output pane, the effects panel, the
  inspector. The GUI library proves itself the way the compiler did, by
  building the thing that builds with it.
- The notebook: the GUI REPL with cells, results and charts, saved as an
  `.nx` script with the outputs as comments, so a notebook is a program.
- The playground with share links (1.5): a snippet is a URL.

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
- Editors: done for Vim, Neovim, Helix, Zed, Emacs, Kate, Notepad++,
  nano and JetBrains (through LSP4IJ) beside VS Code and Sublime Text; a
  JetBrains plugin of its own, and Marketplace listings for Zed and
  Neovim, when the language server is semantic (1.3).
- The registry and `nx publish` (decision 27's last item), seeded with
  the packages that leave the tree: the HTTP client, TLS, TOML, CSV,
  SQLite through `@cImport`, a CLI parser richer than `std.args`.
- Governance: an RFC process for additions (the decision log becomes
  public proposals with a comment period), a release calendar, and the
  stability policy applied to `std` (what a std module may change).
- A second maintainer, on purpose: `good first issue` labels kept
  stocked, "the compiler in an afternoon" (a guided read of `self/` in the
  order the architecture tour uses), every subsystem's owner named in
  `CONTRIBUTING.md`, and release rights shared before 1.4. A project with
  one maintainer is a project with a bus factor of one, and that is a
  weakness of the project, not of the language.
- Third-party packages before the registry: a `packages.md` list of
  packages people can `nx add` from git today, curated, so the ecosystem
  has a front door before it has infrastructure.
- Translations of the book and the reference (docs/i18n exists for the
  README, language and architecture pages).

## Always

- Every release is verified on three platforms by CI before it is tagged,
  under the sanitizers, and by the fuzzer (1,500 iterations on every push
  today; a nightly long run, its findings filed, is the next step).
- The corpus grows at least as fast as the language: every release adds
  programs to `examples/`, `topo/code/` and `tests/spec` shaped like the
  programs people write (a JSON tool, a server, an editor, a ray tracer),
  because the tutorial's twenty-one programs found eight bugs the spec
  suite had not.
- A bug reported from outside gets a patch release within a week of its
  fix; `KNOWN_ISSUES.md` lists what is open, with the workaround, so the
  reader never discovers a known bug the hard way.
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
