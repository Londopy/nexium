# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
The file is validated in CI with [patchnotes](https://pypi.org/project/patchnotes/).

## [Unreleased]

## [0.6.1] - 2026-09-19

### Added

- `nx tir`: the checked program as S-expressions, the oracle for the
  self-hosted checker; `--sigs` prints declarations and signatures only.
- `self/check.nx`, the checker in Nexium: module loading, declarations,
  type interning, signatures (`cargo test` diffs `--sigs` over 41 sources),
  and function bodies: statements, expressions, calls, builtin methods and
  namespaces, matches and patterns, casts, coercions, ownership moves and
  the range analysis. It produces the oracle's exact typed IR for 35
  sources, itself among them; generics, closures, trait objects, binary
  patterns, compile-time calls and C imports remain.

### Fixed

- A C compilation that fails because Zig's shared cache was being written by
  another `nx` at the same time ("failed to check cache") is retried, so
  parallel builds on Windows no longer fail at random.
- The formatter spaces bit-or like the other operators (`a | b`, not
  `a| b`) and no longer glues `-> !List(T) {`; closure bars are classified
  per line so a bit-or inside a closure body is not taken for its closing bar.
- A `String` built for the right side of `and`/`or` produced C that did not
  compile (its release was emitted outside the block that declared it).
- `i128` range bounds overflowed inside the compiler; an `i128` literal
  beyond `i64` was emitted as undefined C for the minimum; `parse_int`
  rejected values above 2^65.
- A range fact from an `if` guard (`if i == 1 { ... }`) outlived its block
  and could prove a later `xs[i]` in range, eliding its bounds check.
- `return xs[i]` into a `?T` or `!T` moved the element without a clone, and
  a diverging `orelse` default or `catch` handler marked its operand moved.
- Floats print as the shortest text that reads back exactly
  (`3.141592653589793`, not `3.1415926535897931`).
- The tree-sitter parser is regenerated for 0.6.0 (its version is embedded).

## [0.6.0] - 2026-09-19

### Changed

- **Breaking: control flow drops its parentheses and bodies always take
  braces** (decision 87). `if c { }`, `while c { }`, `for x, i in items { }`,
  `for i in lo..hi step s { }`, `for parallel x in items { }`, and
  `if let v = opt { }` replace `if (c)`, `while (c)`, `for (items) |x, i|`,
  and `if (opt) |v|`. The one-statement forms become braced one-liners:
  `if c { return v }`, `let m = if a > b { a } else { b }`. A struct literal
  in a condition needs parentheses (`if (Point{ .x = 1 }) == p { }`).
  Match-arm guards are `pat if cond =>`. `catch |e|` and closure
  parameters are unchanged.
- `nx fmt` migrates 0.5 sources to the new syntax as part of formatting;
  `nx fmt --migrate-only` upgrades the syntax and leaves the layout alone.
  The migration edits by token span, keeps comments and blank lines, and
  is idempotent, so running it on a mixed tree is safe.
- The formatter puts a space before the body brace of any control head
  (`if k == Kind.Defer {`), where before an uppercase name would have been
  glued to `{` as a struct literal.
- The language server resolves `for` bindings and `if let` bindings to
  their declaration.
- The tree-sitter grammar, the VS Code and Sublime syntaxes, every example,
  the standard library, the GUI, the self-hosting sources, and all
  documentation (including the translations) use the new syntax.
- Moving a `String` or `List` out of a `for` loop variable is a compile
  error (`use x.clone()`); it silently produced a double free before. A
  loop variable is a view of the element.

### Added

- `self/parser.nx`: the parser written in Nexium now parses the 0.6 syntax
  and prints the same tree as `nx sexp` for every example, std module, GUI
  and self-hosting source; `cargo test` diffs the two (roadmap phase 4).

### Fixed

- `unreachable` as the last statement of a function returning an error
  union or struct produced C that did not compile.
- A program whose imported module also defines `main` (as `self/lexer.nx`
  does, for running the lexer on its own) took the wrong entry point.
- `nx` left one Zig cache directory behind per process under
  `nx-out/.zig-cache`; it is removed when the command finishes, and stale
  ones are swept.

## [0.5.0] - 2026-09-19

### Added

- Packages: a `nexium.toml` manifest with `[dependencies]` from a git tag
  (`{ git = "...", tag = "..." }`) or a directory (`{ path = "..." }`);
  `import dep` loads the dependency's `src/lib.nx` and `import dep.module`
  its `src/module.nx`; a package's own imports stay inside the package.
  `nx init` writes a manifest, `nx add` records and fetches a dependency,
  `nx fetch` clones every git dependency (transitively) into
  `nexium_modules/` and writes `nexium.lock` with the resolved commits.
  See `docs/packages.md`.
- A tree-sitter grammar (`editors/tree-sitter-nexium`) with highlight
  queries for Neovim, Helix and Zed; it parses every example, std module
  and self-hosted source without an error node, and CI keeps it that way.
- The language server gained go to definition (functions, types, constants,
  locals, imported and package modules), completion (module members after
  `alias.`, fields, methods, variants and errors after `.`, names in
  scope), and rename (a local within its function, an item across the
  file). They work from the parsed source, so they answer in files that do
  not type-check yet.
- `artifact installer`: `nx ship` produces an installer for a program. On
  Windows an Inno Setup script (compiled to `<Name>-<version>-setup-x64.exe`
  when Inno Setup 6 is installed) with license page, per-user or
  all-users install, Start menu entry, optional PATH entry and uninstaller;
  on Linux and macOS an `install.sh` with `--prefix` and `--uninstall` plus
  a tarball. Listed `files` are copied next to the program.
- `artifact node { name = "pkg" }`: `nx ship` produces an npm package for an
  exported library: `index.js` calling the shared library through koffi
  (no build step, no node-gyp), `index.d.ts` typings, `package.json`.
  Slices take typed arrays, arrays or strings; error unions throw
  `NexiumError` and panics throw `NexiumPanic` with the message.

### Fixed

- Parallel builds on Windows could fail inside Zig's own cache ("failed to
  check cache ... file_open Unexpected") when several `zig cc` processes
  started at once; each `nx` process now gives Zig its own cache under the
  output directory unless `ZIG_LOCAL_CACHE_DIR` is set.

## [0.4.0] - 2026-09-19

### Added

- Sockets in the runtime: `net.connect`, `listen`, `accept`, `send`, `recv`,
  `close`, `peer`, `local`, `resolve`, `udp_bind`, `send_to`, `recv_from`,
  `last_peer`; blocking, with per-call timeouts, on Winsock and BSD sockets.
  New errors `Timeout` and `ConnectionRefused`.
- `std.net`: `TcpStream` (connect with timeout, send, recv, recv_all, peer,
  buffered `reader()`/`writer()`), `TcpListener` (bind, accept with
  timeout, port), `UdpSocket` (bind, send_to, recv_from), `parse_addr`,
  `port_of`, `resolve`. `std.stream` readers and writers work over sockets.
- `std.http`: a client (`get`, `post`, `request` with headers; HTTP/1.1,
  Content-Length and chunked bodies, up to five redirects) and a server
  (`Server.bind`, `serve`, `serve_one`, `Router` with exact and `/*` routes,
  `serve_static`, `Request.param`/`header`, response helpers `text`, `html`,
  `json`, `redirect`, `not_found`). Plain `http://`; TLS is left to
  `@cImport`.
- `examples/service.nx`: an HTTP service and a client in one program, the
  phase 2 exit example; `examples/errors_more.nx` covers the fixes below.
- Threads in the runtime (`thread.start`, `thread.join`, `thread.count`,
  `sync.mutex_new`/`lock`/`unlock`/`mutex_free`, `sync.cond_new`/`wait`/
  `signal`/`broadcast`/`cond_free`) and `std.thread` on top: `spawn` returns
  a `Thread(T, R)` whose `join` yields the function's result, `run` returns
  a `Worker(T)` for functions without one, `Channel(T)` (`send`, `recv`,
  `try_recv`, `close`), `Mutex(T)` (`lock` returns `*mut T`, `unlock`). A
  panic inside a thread is re-raised by `join`. Starting a thread carries
  the `nondeterministic` and `shared_mutable` effects.
- `own` is accepted on parameters of methods in generic `impl` blocks.
- `process.exec(argv, stdin, cwd)` runs a program with stdin fed, a working
  directory, and stdout/stderr captured (`process.last_stdout`,
  `process.last_stderr`); `std.process` wraps it as `run`, `run_with`,
  `shell` returning an `Output` with `code`, `stdout`, `stderr`.

### Fixed

- Free functions with the same name in two imported modules (`fs.copy` and
  `stream.copy`) collided in the generated C.
- Returning a caught error value (`catch |e| { return e }`) from a function
  returning `!T` produced a bare error id instead of an error union.
- An untyped integer literal now coerces into `!T` (`return 7` in a
  function returning `!i32`).
- The formatter kept the space in `-> http.Response {` (a dotted type before
  a block is not a struct literal).

## [0.3.0] - 2026-09-19

### Added

- `std.fs`: `exists`, `is_file`, `is_dir`, `size`, `modified`, `read`,
  `read_lines`, `write`, `append`, `copy`, `list` (sorted), `make_dir`,
  `make_dirs`, `remove`, `remove_all`, `rename`, `walk`, `cwd`, `temp_dir`,
  `temp_path`, and the path helpers `join`, `parent`, `base_name`, `stem`,
  `extension`, `with_extension`, `normalize`, `is_absolute`. Under it, new
  `io` primitives in the runtime: `append_file`, `file_kind`, `file_size`,
  `file_modified`, `make_dir`, `remove_file`, `remove_dir`, `rename`,
  `list_dir`, `cwd`, `temp_dir`; they also work at the REPL.
- `std.time`: `DateTime` (`utc`, `local`, `with_offset`, `now_utc`,
  `now_local`, `date`, `parse_iso`, `to_ms`, `weekday`, `day_of_year`,
  `iso`, `format` with `%Y %m %d %H %M %S %3 %z %a %b %j`), `Duration`
  (`seconds` ... `days`, `between`, `since`, `text` such as `1h 02m`),
  `Stopwatch` on the monotonic clock, `is_leap`, `days_in_month`. Under it,
  `time.utc_offset(ms)` in the runtime (0 at the REPL).
- `std.regex`: a Pike VM (no backtracking, linear time) with classes,
  `\d \w \s \b`, anchors, groups and `(?:...)`, alternation, greedy and
  lazy repeats including `{n,m}`; `compile`, `find`, `find_at`, `find_all`,
  `is_match`, `replace_all` with `$1` references, `split`, and `Match.group`.
- A local that was moved out can be assigned again; the assignment
  re-initializes it instead of being reported as a use after move.
- `for (a..b step s) |i|` walks a range with a step; a negative step counts
  down (the loop variable must be signed). `while (c) { } else { }` runs the
  else block when the condition turns false, but not after a `break`.
- `match` exhaustiveness now uses the full matrix algorithm, so tuples of
  enums, nested optionals and enum payloads are checked precisely instead of
  demanding a catch-all arm.
- `nx test FILE --filter NAME` runs the tests whose names contain NAME.
- `std.text`: UTF-8 by code point (`decode_at`, `chars`, `char_count`,
  `char_at`, `slice`, `truncate`, `reverse`, `encode`, `is_valid`), terminal
  `width` (wide and zero-width aware), and case mapping for ASCII, Latin-1,
  Latin Extended-A, Greek and Cyrillic (`to_upper`, `to_lower`,
  `eq_ignore_case`).
- `std.testing`: `approx`, `expect_approx`, `is_err`, `expect_err`,
  `expect_error`, `expect_contains`, `expect_lines` (names the first
  differing line) and file snapshots (`snapshot`, `snapshot_in`;
  `NX_UPDATE_SNAPSHOTS=1` rewrites them).
- Embedded std modules can import each other; `error` is a type name (the
  anonymous error set), and `own` is accepted on generic parameters.
- `nx test` no longer runs the tests of imported std modules.
- `std.stream`: buffered `Reader` (`open`, `stdin`, `read_line`, `read`,
  `read_all`) and `Writer` (`open`, `append`, `stdout`, `stderr`, `write`,
  `write_line`, `flush`, `close`) plus `copy`, over new runtime file handles
  (`io.open`, `io.read`, `io.write`, `io.flush`, `io.close`; handles 1 to 3
  are the standard streams).
- `os.environ()` lists the environment; `args.env_map()` turns it into a
  `Map(String, String)`.
- `nx test --verbose` prints timings and the tests a filter skipped;
  `nx doc std.fs` and `nx test std.regex` accept an embedded module by name.
- `examples/tool.nx`: a log scanner (walk a tree, parse timestamps, filter
  by a date window and a regex, tally by level) in 142 lines, the phase 1
  exit example of the roadmap; runs on `examples/data/logs` by default.

### Fixed

- A struct, tuple or enum literal that read a local in one field and moved
  it in a later field saw the already-zeroed value; field values are now
  materialized in source order.

## [0.2.1] - 2026-09-19

### Added

- `SPEC.md`, the language specification as implemented, and `ROADMAP.md`.
- `nx repl`, and `nx` with no arguments at a terminal: an interactive session
  on the compiler's interpreter, with bindings kept across lines, the real
  diagnostics, I/O, and `:load`. The Windows installer adds a Start menu
  entry that opens it, and offers to launch it when setup finishes.
- The compile-time interpreter now evaluates `Map`, the mutating `List`,
  `String` and slice methods (`insert`, `remove`, `sort`, `split`, `trim`,
  `parse_int`, ...), `expect_eq`, `assert`, UTF-8 helpers and the reference
  class operations, and pointers stay valid across calls, so `std.json` and
  the other std modules run at the prompt. A statement the interpreter
  cannot evaluate is reported with the position of the failing expression.

### Fixed

- Building a program by its bare file name (`nx run app.nx` from inside its
  directory) dropped the C sources declared in `artifact link`: the empty
  parent directory produced a lone `-I` that swallowed the next argument.
- `opt.?`, `opt orelse d` and `try res` on a local holding an owning value
  now move the local: it is no longer dropped a second time at scope end
  (this crashed GUI programs on exit), and a later use is reported as a use
  after move.

## [0.2.0] - 2026-09-19

### Added

- `own` parameters: `fn f(own s: String)` takes ownership of its argument
  (moved at the call site, dropped or moved on by the callee).
- `orelse return v` and `catch |e| return v`: jumps on the right of `orelse`
  and `catch`.
- `String.push_byte(b)` appends one raw byte.
- Integer and float literals coerce into `?T`.
- `examples/own.nx` and five compile-fail cases for the `own` rules.
- The standard library in Nexium: `import std.strings`, `std.lists`,
  `std.bytes`, `std.num`, `std.json`, `std.args` (96 functions with tests),
  embedded in the compiler; `docs/std.md` lists them; `examples/stdlib.nx`
  uses them.
- Tuple types as type arguments: `List((A, B))`.
- A Windows installer (`nexium-<version>-setup-x64.exe`): license, overview,
  per-user or all-users, components (bundled Zig, std and examples, docs, VS
  Code extension), PATH and `.nx` file-type tasks, `nx doctor` on finish.
- An install script for macOS and Linux (`installers/install.sh`) that
  verifies checksums and sets up a C compiler.
- `nx doctor`, and `nx` finds a Zig bundled next to itself (`NX_ZIG` too).
- Releases carry `SHA256SUMS.txt`, per-file checksums, and install
  instructions in the notes; the tarballs include examples, std, and docs.
- Recursive types through `List`: `enum Json { Arr(List(Json)) }`.
- Matching through a pointer (`match p.*`) binds owning payloads by reference,
  so `match v.* { .Arr(items) => items.append(x) }` mutates in place.
- `*String` and `*List(T)` coerce to `[]u8` and `[]T`; `null` coerces into a
  nested optional such as `!?T`.

### Changed

- Moves are tracked per branch: a value moved in one `if` branch or `match`
  arm stays usable in the others.
- An unbraced `if` or `else` body is one statement, so `if (c) x = 1` works.
- `self/lexer.nx` moves token text with `own` instead of cloning it.

### Fixed

- A branch that diverges (`if (c) return x`) no longer marks what it moved as
  moved afterwards.
- Two modules each defining a type of the same name (`Parser` in `std.json`
  and `std.args`) collided in the generated C; type names now carry their
  module. A user function named like a runtime identifier (`fn string`)
  no longer collides either.
- `nx fmt` spacing after `-> List(T)`, after a closing closure bar, and
  between an `if` condition and a parenthesized body.

## [0.1.0] - 2026-09-18

First public release.

### Added

- The `nx` compiler: `build`, `run`, `test`, `check`, `effects`, `audit`, `ship`, `emit-c`, `parse`.
- Language: structs, records with `where` constraints, enums with payloads, `ref class`
  with reference counting and `weak`, distinct types, traits with default methods,
  generic functions and types through `comptime T: type`, closures with explicit
  captures, error unions with `try`/`catch`, `defer`/`errdefer`, optionals,
  labeled blocks and loops, the pipe operator, binary pattern matching and
  construction, compile-time evaluation of constants, `test` blocks.
- Effects: `allocates`, `refcounts`, `blocks`, `shared_mutable`, `nondeterministic`,
  `panics`, `ffi`, inferred for every function and enforced against negative bounds,
  with diagnostics naming the site that introduces an effect.
- Standard library builtins: `List`, `String`, `Map`, slices, formatting, `math`,
  `io`, `os`, `time`, `random`, `mem`.
- Artifacts: `cabi` (static library, shared library, C header) and `python`
  (ctypes package and wheel) from one source tree with `nx ship`; `cli` binaries.
- Embedded mode: reachable mutable globals are rejected when an embeddable
  artifact is declared (S2); panics never cross the export boundary (S3).
- Examples under `examples/`, compile-fail cases under `tests/compile_fail/`.
- `dyn Trait` trait objects with effect bounds, `for parallel` over a thread pool,
  `using arena { }` allocation scopes, `comptime test`.
- `@cImport("header.h")` direct C header import with variadic calls, `@cstr`, and
  vendored C through `artifact link { c_sources = [...] }`.
- `rustlib` artifact: a generated Cargo crate with safe wrappers.
- Tools: `nx fmt`, `nx doc`, `nx size`, `nx refcounts`, `nx leaks`, `nx lsp`.
- A conservative region check: returning a view into a local is an error.
- `process.run(argv) -> !i32`, `nx tokens`, and `self/lexer.nx`: the first stage
  of the self-hosted compiler, verified against the Rust lexer in `cargo test`.
- Logo and banner under `assets/`; `docs/architecture.md` explains how the compiler works.
- `gui/`: nexium-gui, an immediate-mode GUI written in Nexium (software rasterizer,
  bitmap font, buttons, checkboxes, sliders, text fields) over a 200-line C
  platform layer; Win32 window backend, offscreen rendering everywhere.
- `artifact link` accepts `libs_windows`, `libs_linux`, `libs_macos`; paths in a
  link artifact resolve relative to the module that declares it.
- Editor support: a VS Code extension (grammar, `nx lsp` client, run command,
  packaged as a `.vsix` on every release) and a Sublime Text syntax.
- `setup-nexium` GitHub Action and a release workflow template for programs
  written in Nexium (`docs/releasing-your-program.md`).
- `linguist/`: the prepared GitHub Linguist entry, samples, heuristic, and apply script.
- `@cImport`: structs with untranslatable fields are opaque types instead of
  errors, so `FILE*` works with Apple's libc.
- Native macOS builds use the system `cc` (zig 0.14 cannot link against the
  Xcode 26 SDK); `NX_CC` selects the C compiler without a flag.
- Translations under `docs/i18n/`: the README in Spanish, Chinese, Japanese,
  Korean, French, and German; the language reference and architecture tour in
  Spanish, Chinese, and Japanese.

[Unreleased]: https://github.com/Londopy/nexium/compare/v0.6.1...HEAD
[0.6.1]: https://github.com/Londopy/nexium/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/Londopy/nexium/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/Londopy/nexium/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/Londopy/nexium/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/Londopy/nexium/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/Londopy/nexium/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/Londopy/nexium/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Londopy/nexium/releases/tag/v0.1.0
