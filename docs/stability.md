# Stability

What a version number promises, from 1.0 on. The short form: a program that
builds with 1.x builds, and does the same thing, with every later 1.y.

## Versions

Nexium follows [semantic versioning](https://semver.org). `MAJOR.MINOR.PATCH`,
and each release has a name from [release-names.md](release-names.md).

- A **patch** (1.0.1) fixes bugs. It changes no program's meaning except
  where the old meaning was a bug listed in `KNOWN_ISSUES.md` or the
  changelog's `Fixed` section. It may also add what changes no meaning
  (a tool, a command, a builtin, a field of an `artifact` block, a way to
  install), as 1.0.1 to 1.0.3 did: the line between a patch and a minor
  is meaning, not size.
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

Undefined behaviour was never promised. A program that has it under
`SPEC.md` (the view cases of 5.6, a value leaving its arena in 5.7) may
become an error in a minor as the checker learns to see it; such a rule
arrives as a warning in one release and an error in the next, so the
program gets a release to run `nx fix`. 1.2 did this for the view rules
V1 to V5 (`SPEC.md` 5.6 and 5.7): they warned, `--strict` or `NX_STRICT=1`
made them errors, and since 1.3 they are errors for everyone. `--strict`
stays: it makes any warning an error, a deprecation's included.

## Deprecation

A feature that has to go is deprecated first:

1. A minor version marks it: `nx check` warns at every use, with the
   replacement named; `SPEC.md` and the changelog say so.
2. It stays for at least two more minor versions and six months. Programs
   using it keep building and keep meaning the same.
3. The next major removes it. Where the replacement is mechanical, `nx fix`
   rewrites the program.

## `nx fix`

`nx fix FILE...` rewrites source in place and reports what it changed:
the deprecations the running compiler knows how to migrate, and the edits
the checker offers for its errors where the fix is mechanical: rule V4's
`.clone()` where a value moves out from under a view of it, rule V5's
`@escape(...)` around a value kept past its arena, `_ = ` in front of a
value nothing uses. Each edit is tried on the program first and kept only
when the checker then reports fewer errors and none it did not report
before; what it cannot fix mechanically it leaves, and says how many
errors remain. `--check` reports without writing. It never changes what a
correct program means, and it makes no edit that would decide what a
program computes: where arithmetic may overflow in a `!panics` function,
wrapping (`+%`) and saturating (`+|`) are the programmer's choice
(decision 113). There are no deprecations yet.

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
