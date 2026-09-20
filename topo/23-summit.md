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

The [roadmap](../docs/roadmap.html) is public and specific. The next few
themes:

- **1.1**, the ergonomics the compiler wanted while it was being written in
  itself: `a?.b`, tuple destructuring, `derive(Clone)`, iterators, slice
  patterns, more proofs so fewer functions carry `panics`.
- **1.2**, memory safety without a garbage collector: the view rules, so that
  a slice or pointer cannot outlive its storage and `unsafe` marks
  everything the compiler cannot prove, without a borrow checker's
  annotations. Chapter 8 said where the 1.0 compiler stops; 1.2 is where it
  stops stopping.
- **1.3**, the toolchain grown up: incremental builds, a semantic language
  server, `nx bench`, sanitizers.
- **1.4**, the standard library people stop supplementing; **1.5**,
  platforms (WebAssembly, more GUI backends); **1.6**, the runtime that
  release builds deserve.

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
`self/check.nx` has the answer, and after these chapters you can read it.

Thanks for climbing.
