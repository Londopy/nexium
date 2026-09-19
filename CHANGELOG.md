# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
The file is validated in CI with [patchnotes](https://pypi.org/project/patchnotes/).

## [Unreleased]

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
- Translations under `docs/i18n/`: the README in Spanish, Chinese, Japanese,
  Korean, French, and German; the language reference and architecture tour in
  Spanish, Chinese, and Japanese.

[Unreleased]: https://github.com/Londopy/nexium/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Londopy/nexium/releases/tag/v0.1.0
