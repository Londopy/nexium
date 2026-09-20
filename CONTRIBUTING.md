# Contributing to Nexium

Thanks for taking a look. This document covers how to build, test, and propose
changes.

Start with `docs/architecture.md` for a map of the compiler.

## Building

```bash
cargo build            # the compiler, target/debug/nx
cargo test             # unit tests, every example against its recorded output,
                       # compile-fail cases, and a `nx ship` round trip
```

`nx` needs a C compiler. It looks for `zig` on your `PATH` and uses `zig cc`;
pass `--cc clang` (or `--cc gcc`) to use another one. The integration tests
skip the parts that need a C compiler when `zig` is missing, so `cargo test`
still passes on a machine without it, but you will want it installed.

## Layout

| path | what lives there |
| --- | --- |
| `src/lexer.rs`, `src/parser.rs`, `src/ast.rs` | syntax |
| `src/check/` | name resolution, type checking, monomorphization, effects |
| `src/comptime.rs` | the compile-time interpreter over the typed IR |
| `src/cgen/` | the C backend and the export boundary |
| `src/ship.rs` | headers, Python packages, wheels |
| `src/main.rs` | the `nx` driver |
| `runtime/nx_rt.h` | the C runtime, embedded into generated code |
| `examples/` | programs with `.expected` output, run by the tests |
| `tests/compile_fail/` | programs that must be rejected, with `// EXPECT:` lines |

## Adding a test

- A behaviour that should work: add or extend a program in `examples/`, run it
  with `nx run`, and save the output as `examples/<name>.expected` (the
  integration test compares stdout+stderr). Add the name to the list in
  `tests/examples.rs` if it is a new file.
- A program that must be rejected: add `tests/compile_fail/<name>.nx` with one
  or more `// EXPECT: <substring of the diagnostic>` lines at the top.

## Language changes

The specification is the authority. A proposal to change the language must
name which hard constraint in section 3 of `nexium-spec.txt` it serves, list
the alternatives considered, and state the migration cost (spec 17.6). Open an
issue with the `language` label before writing code; small compiler fixes do
not need a proposal.

Decisions already taken where the specification was silent are recorded in
`DECISIONS.md`. If you disagree with one, open an issue that references its
number.

## Known issues

Open bugs and limitations live in `KNOWN_ISSUES.md`, each with a repro and
the likely fix. Add what you find there; when you fix one, remove its entry,
add a `Fixed` line to the changelog, and land a regression test with it.

## Changelog

`CHANGELOG.md` follows [Keep a Changelog](https://keepachangelog.com/) and is
validated in CI with [patchnotes](https://pypi.org/project/patchnotes/). Add a
line under `Unreleased` in the right category (`Added`, `Changed`,
`Deprecated`, `Removed`, `Fixed`, `Security`) with your change.

## Style

- Rust: `cargo fmt` and `cargo clippy` clean.
- Nexium examples: four-space indentation, one statement per line.
- Diagnostics name the constraint violated, the location, and where the
  constraint was declared (archived spec, section 18). A new error message
  should follow that shape.

## Pull requests

Keep them focused. Describe what changed and why, link the issue, and make
sure `cargo test` passes on your machine. CI runs on Windows, Linux, and macOS.

Every pull request has to pass, before it can merge:

- the tests on the three platforms, the editor-support build, `cargo fmt`
  and `clippy` (`ci.yml`);
- the changelog validation, and a check that the pull request adds a line
  to `CHANGELOG.md` under `Unreleased` (label it `no changelog` when there is
  nothing to say: a typo, a CI tweak);
- `nx fmt --check` on every `.nx` file it touches;
- the CLA check (below).

When every required check has passed, the pull request gets the `ready to
merge` label (and loses it when a new push restarts the checks). The
maintainer reviews every change (`.github/CODEOWNERS`); `main` only takes
what these checks have passed.

## AI-assisted contributions

You may use AI tools to help write a contribution. Three rules, which the
CLA makes binding:

- **Say so in the pull request.** The template has a line for it: which
  tool, and what it did (drafted the change, wrote the tests, explained the
  code, translated the docs). "None" is a fine answer.
- **You are the author.** Read and understand everything the tool produced
  before you submit it; you are answerable for it in review exactly as for
  code you typed. A pull request whose author cannot explain a change is
  sent back. Commits and files name people, not tools: an AI tool is never
  listed as an author or co-author.
- **No laundering.** Do not submit output that reproduces someone else's
  work under a license incompatible with MIT, or that you have no right to
  license. If a tool hands you a recognizable chunk of another project,
  leave it out.

An undisclosed AI-assisted contribution is closed when discovered, whatever
its quality. Disclosed ones are reviewed like any other: the tests are the
bar.

## Contributor License Agreement

Your first pull request gets a comment from the CLA check asking you to
accept [`CLA.md`](CLA.md) by replying with one sentence. You keep the
copyright in your work; the agreement gives the project a license to
distribute it under the MIT License (or another OSI-approved license, never
a proprietary one) and a patent license for what your contribution
necessarily uses, and it records that the work is yours to give, AI-assisted
or not. Companies contributing on behalf of employees can open an issue
naming the covered accounts instead.

## Releasing

1. Move the `Unreleased` entries in `CHANGELOG.md` under a new
   `## [x.y.z] - YYYY-MM-DD` heading and add its compare link at the bottom.
   `patchnotes validate CHANGELOG.md` must pass.
2. Set the same version in `Cargo.toml` and run `cargo build` so
   `Cargo.lock` follows.
3. Tag and push: `git tag vx.y.z && git push origin vx.y.z`. The release
   workflow refuses a tag that does not match `Cargo.toml`, builds `nx` for
   Windows, Linux, and macOS, and publishes a GitHub release with the
   changelog section as its notes.
4. Optionally `cargo publish` for `cargo install nexium`.
