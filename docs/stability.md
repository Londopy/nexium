# Stability

What a version number promises, from 1.0 on. The short form: a program that
builds with 1.x builds, and does the same thing, with every later 1.y.

## Versions

Nexium follows [semantic versioning](https://semver.org). `MAJOR.MINOR.PATCH`,
and each release has a name from [release-names.md](release-names.md).

- A **patch** (1.0.1) fixes bugs. It changes no program's meaning except
  where the old meaning was a bug listed in `KNOWN_ISSUES.md` or the
  changelog's `Fixed` section.
- A **minor** (1.1.0) adds. Everything that was accepted is still accepted
  and means the same; what was rejected may become accepted.
- A **major** (2.0.0) may remove what a minor deprecated, after the cycle
  below, and never anything else.

The language changes only by addition after 1.0 (ROADMAP, "Past 1.0"). A
change that would alter the meaning of an existing program waits for a
major, however small.

## What a minor version may do

- Add syntax that was an error before (a new keyword only when it was
  reserved in `SPEC.md`; `KEYWORDS` in `self/lexer.nx` is the list).
- Add builtins, methods on builtin types, modules and functions to the
  standard library, effects only as reserved names becoming real, and
  fields to `artifact` blocks.
- Accept more programs: a stricter check may be relaxed, a limitation in
  `SPEC.md` may be lifted.
- Improve diagnostics, the generated C, the runtime, the tools' output.
  The text of a diagnostic is not stable; its presence is.
- Add a platform, a target, an editor integration.

## What a minor version may not do

- Remove or rename anything a program can name: syntax, a builtin, a
  standard-library function, a field, an effect, a command-line option.
- Change the effects a standard-library function has. A function that
  became impure would break every `pure fn` calling it.
- Change the output or exit code of a program in `examples/` or
  `tests/spec/`, except a bug fix listed as such.
- Change the shape of what `nx ship` produces for a given source: the C
  header's declarations, the Python and Rust signatures, the npm typings.
  New artifacts may appear next to them.
- Change the on-disk formats a program reads: `nexium.toml`, `nexium.lock`.

`nx fmt` may change its output in a minor: formatting is a tool, not a
language feature. Such a change is listed in the changelog.

## Deprecation

A feature that has to go is deprecated first:

1. A minor version marks it: `nx check` warns at every use, with the
   replacement named; `SPEC.md` and the changelog say so.
2. It stays for at least two more minor versions and six months. Programs
   using it keep building and keep meaning the same.
3. The next major removes it. Where the replacement is mechanical, `nx fix`
   rewrites the program.

## `nx fix`

`nx fix FILE...` rewrites source for the deprecations the running compiler
knows how to migrate, in place, and reports what it changed. It never
changes meaning and never touches what it cannot migrate mechanically; those
uses stay as warnings. At 1.0 there are no deprecations, and `nx fix` says
so.

## Not covered

- The generated C and `runtime/nx_rt.h` are artifacts, not interfaces.
  Programs do not include the runtime header; `nx ship` gives them a header
  of their own.
- The typed IR that `nx tir` prints exists for the compiler's own tests.
- The comptime interpreter's step budget, stack limits, and the exact
  bytes `nx size` attributes to each declaration.
- Behaviour that `SPEC.md` calls undefined or unspecified.

## The C toolchain

`nx` is tested with the Zig version CI pins (`ZIG_VERSION` in
`.github/workflows/ci.yml`, 0.14.1 today), which the Windows installer and
the install script also install. Other Zig versions and other C compilers
(`--cc gcc`, `--cc clang`) are expected to work: the C that `nx` emits is
`gnu11` with no compiler extensions beyond `__attribute__` guards, and the
runtime is one plain-C header. A bug that appears with one compiler and not
another is a bug in `nx`; `KNOWN_ISSUES.md` lists any that are open. When
the pinned Zig version moves, the changelog says so and `nx doctor` shows
what an installation has.

## Platforms

Which operating systems and architectures are supported, and how far, is
in [platforms.md](platforms.md).
