# The summit

You have installed a compiler, written a script, used a prompt, learned
the values and the control flow, the errors and the optionals, the structs,
enums and traits, the ownership rules, the collections, built a calculator
and a to-do list, parsed binary data, computed at compile time, written
tests, read effects, run threads, served HTTP, drawn a window, made a
package, shipped a library to three other languages, and trained a neural
network. That is the language. There is no chapter on advanced features
that the rest of us were keeping from you.

## What to read now

- The [language reference](../docs/language.html): every construct, one
  paragraph each. It is the page to keep open while writing.
- The [standard library](../docs/std.html): every module and function, from
  the doc comments in `std/`.
- The [specification](../docs/spec.html): what the language promises, in
  the words the test suite is written against. Section 5 (ownership), 8
  (effects) and 12 (what undefined behaviour is and is not) are the ones
  that repay a careful read.
- [How Nexium works](../docs/architecture.html): the compiler stage by
  stage, for when you want to change it.
- [Stability](../docs/stability.html) and [platforms](../docs/platforms.html):
  what a version number means, which targets are tested.

## What is coming

The [roadmap](../docs/roadmap.html) is public and specific. Since 1.0 the
language has grown the ergonomics its compiler wanted while being written
in itself (1.1), memory safety without a garbage collector, the view rules
(1.2), a toolchain grown up: incremental builds, a semantic language
server, `nx bench`, sanitizers (1.3), a standard library people stop
supplementing (1.4), and its platforms: WebAssembly, 32-bit and ARM (1.5).
The next themes:

- **1.6**, the runtime that release builds deserve: fewer reference-count
  updates and bounds checks, faster maps, strings and parallel loops, each
  change measured against C on the [numbers page](../docs/numbers.html).
- **1.7**, the seam both ways: what chapter 20 shipped to Python and Rust
  grows to NumPy, exceptions and handles, error enums and `no_std`, more
  languages out, and Python and Rust called from Nexium.
- **1.8**, nexium-gui grown up on Linux and macOS as well, a web front end
  made in Nexium, and the apps built on them.
- **2.0**, the first breaking changes since 1.0, announced by a release
  that deprecates them two minors ahead.

Each release has a name from the mountain the project has been climbing,
and the [names document](../docs/release-names.html) explains the scheme;
this book is a topo because the releases are a route.

## Joining in

The repository is [github.com/Londopy/nexium](https://github.com/Londopy/nexium).
[Contributing](../docs/contributing.html) says how a change gets in: an
example with recorded output or a compile-fail case, a changelog line, a
language change needs the constraint it serves, and the [contributor license
agreement](../docs/cla.html) signed by a comment on the pull request. Bugs
go in the tracker; open ones the maintainers know about are listed in
[known issues](../docs/known-issues.html), and security reports have their
[own path](../docs/security.html).

The compiler is written in the language this book teaches, and its test
harness and fuzzer are too. If you want to know how a feature really works,
`self/check.nx` and the `check_*.nx` modules beside it have the answer, and
after these chapters you can read them.

Thanks for climbing.
