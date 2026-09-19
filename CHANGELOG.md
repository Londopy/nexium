# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
The file is validated in CI with [patchnotes](https://pypi.org/project/patchnotes/).

## [Unreleased]

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

[Unreleased]: https://github.com/Londopy/nexium/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/Londopy/nexium/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/Londopy/nexium/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Londopy/nexium/releases/tag/v0.1.0
