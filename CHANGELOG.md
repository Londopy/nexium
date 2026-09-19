# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
The file is validated in CI with [patchnotes](https://pypi.org/project/patchnotes/).

## [Unreleased]

### Added

- `SPEC.md`, the language specification as implemented, and `ROADMAP.md`.

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

[Unreleased]: https://github.com/Londopy/nexium/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/Londopy/nexium/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Londopy/nexium/releases/tag/v0.1.0
