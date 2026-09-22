# Contributing to Nexium

Thanks for taking a look. This document covers how to build, test, and propose
changes.

Start with `docs/architecture.md` for a map of the compiler.

## Building

```bash
sh bootstrap/build.sh              # the compiler in Nexium from the C seed, no Rust: nx-out/bootstrap/nx2
nx-out/bootstrap/nx2 run tests/run.nx   # every suite through it (the harness is a Nexium program)
nx-out/bootstrap/nx2 run tests/run.nx -- spec fmt   # only these suites
```

`nx` needs a C compiler. It looks for `zig` on your `PATH` and uses `zig cc`;
pass `--cc clang` (or `--cc gcc`) to use another one. The harness rebuilds
the compiler when anything under `self/`, `std/`, `runtime/` or the seed is
newer than it, and skips the parts of the `ship` suite whose tools (`cargo`,
`npm`) are not installed.

## Layout

| path | what lives there |
| --- | --- |
| `self/lexer.nx`, `self/parser.nx` | syntax |
| `self/check.nx`, `self/cimport.nx` | name resolution, type checking, monomorphization, effects, the compile-time interpreter, C header import |
| `self/cgen.nx` | the C backend |
| `self/nx.nx` | the `nx` driver: build, run, test, check, emit-c, tir |
| `self/fmt.nx`, `self/doc.nx`, `self/tools.nx`, `self/size.nx`, `self/manifest.nx`, `self/ship.nx`, `self/lsp.nx`, `self/repl.nx` | the tools: formatter, docs, reports, packages, `ship`, the language server, the REPL |
| `bootstrap/` | the C seed `nx.c` and the build scripts |
| `runtime/nx_rt.h` | the C runtime, embedded into generated code |
| `examples/` | programs with `.expected` output, run by the tests |
| `tests/run.nx` | the test harness |
| `tests/spec/` | the specification's conformance cases, one per claim, with recorded output |
| `tests/compile_fail/` | programs that must be rejected, with `// EXPECT:` lines |

## Adding a test

- A behaviour that should work: add or extend a program in `examples/`, run it
  with `nx run`, and save the output as `examples/<name>.expected` (the
  harness compares stdout+stderr). Add the name to the list in
  `suite_examples` in `tests/run.nx` if it is a new file. A claim the
  specification makes gets a case in `tests/spec/` instead, picked up by name.
- A program that must be rejected: add `tests/compile_fail/<name>.nx` with one
  or more `// EXPECT: <substring of the diagnostic>` lines at the top.

## Where the compiler is

The compiler is `self/` (lexer, parser, checker, C emitter, driver, the
tools), written in Nexium; that is where every change goes. A machine that
has no `nx` builds one from `bootstrap/nx.c`, the C the compiler emits for
itself (decision 90). A change to `self/*.nx` may implement a new feature
but may not use it until the seed knows it: regenerate the seed in the same
change (`nx emit-c self/nx.nx --mode safe > bootstrap/nx.c`) when `self/`
needs a builtin or a form the seed lacks.

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
validated in CI with [patchnotes](https://pypi.org/project/patchnotes/).
Repositories that use [pre-commit](https://pre-commit.com) can run `nx fmt`
and `nx check` on every commit through the hooks in
`.pre-commit-hooks.yaml`; `CITATION.cff` is how to cite the language. The
wheels reach PyPI through the `PyPI` workflow and a trusted publisher
registered on PyPI (owner `Londopy`, repository `nexium`, workflow
`pypi.yml`, environment `pypi`); a release whose upload failed is published
again by running that workflow by hand with the tag. Add a
line under `Unreleased` in the right category (`Added`, `Changed`,
`Deprecated`, `Removed`, `Fixed`, `Security`) with your change.

## Style

- Nexium: `nx fmt` clean; four-space indentation, one statement per line.
- Diagnostics name the constraint violated, the location, and where the
  constraint was declared (archived spec, section 18). A new error message
  should follow that shape.

## Pull requests

Keep them focused. Describe what changed and why, link the issue, and make
sure `nx run tests/run.nx` passes on your machine. CI runs on Windows, Linux,
and macOS.

Every pull request has to pass, before it can merge:

- the tests on the three platforms, the editor-support build, `nx fmt
  --check` on the tree and `nx check` on the compiler (`ci.yml`);
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

## What runs on a push

The `CI` workflow (the bootstrap and every harness suite on three
platforms, the fuzzer, the sanitizers, the editor files, the lint) skips
a push or pull request that changes only prose and pictures (`*.md`,
`docs/i18n/`, `topo/img/`, `assets/`, `linguist/`). `CI (prose)` runs
instead: it validates `CHANGELOG.md` and reports the other required
checks as nothing to build, so a documentation pull request can merge;
`Pages` builds and deploys the site from the Markdown. A change to a
`.nx`, `.c`, `.h` or workflow file runs everything. `[skip ci]` in a
commit message skips every workflow, the site's deploy included, so it is
for the rare push that should not be published at all; GitHub reads the
marker anywhere in the message, so a message that merely mentions it
skips too (spell it out, "the skip marker", when writing about it).

## Releasing

1. Move the `Unreleased` entries in `CHANGELOG.md` under a new
   `## [x.y.z] - YYYY-MM-DD` heading and add its compare link at the bottom.
   `patchnotes validate CHANGELOG.md` must pass.
2. Set the same version and release name in `self/nx.nx` (`VERSION`,
   `RELEASE_NAME`) and regenerate the seed:
   `nx emit-c self/nx.nx --mode safe > bootstrap/nx.c`.
3. Tag and push: `git tag vx.y.z && git push origin vx.y.z`. The release
   workflow refuses a tag that does not match `self/nx.nx`, builds `nx` from
   the seed for Windows, Linux, and macOS, and publishes a GitHub release
   with the changelog section as its notes.
4. Land what the workflows could not. The release workflow commits the
   package-manager manifests (`Formula/`, `bucket/`, `installers/`) to
   `main`, and the Bench workflow the numbers; a protected `main` refuses
   those pushes, so the commits wait on `packaging/vx.y.z` and `numbers`
   and each job's notice says so. Land them with
   `git fetch origin packaging/vx.y.z && git cherry-pick FETCH_HEAD && git push origin main`
   (the same for `numbers`; the site reads the numbers page from that
   branch either way).
