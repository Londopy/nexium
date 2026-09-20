# Base camp

A *topo* is the drawing a climber carries up a route: every pitch marked,
the hard moves named, the belays where they are. This one goes up Nexium.
It starts at the bottom, with a compiler on your machine and `hello, world`,
and ends with a neural network, a GUI, a network service and a library
shipped to C, Python and Node from one file. Every program in it is real: the
test suite runs each one and compares its output with what the page shows,
so the code you read is the code that ran.

Read the chapters in order the first time. Each one leans on the one before,
and each is short enough to finish in a sitting. The [language
reference](../docs/language.html) and the [standard library](../docs/std.html)
are the map for later, when you know what you are looking for.

## What Nexium is

Nexium is a systems language that compiles to C. It has:

- **Ownership without a borrow checker.** `List`, `String` and `Map` own
  their storage. Passing one by value moves it; a function borrows what it
  is handed; the compiler tells you when you use something after it moved.
  There are no lifetime annotations to write.
- **Automatic reference counting** for the objects you want shared
  (`ref class`), with `weak` for back edges, and no tracing garbage
  collector: things are freed the moment the last reference goes.
- **Effects the compiler infers**: whether a function allocates, blocks,
  can panic, touches shared state, calls foreign code. You can promise a
  function does not (`!allocates`, `!panics`) and the compiler holds you to
  it, naming the line, however deep in the call chain, that breaks the
  promise.
- **One source, every target.** `nx ship` turns a file into a C library
  with a header, a Python wheel, a Rust crate, an npm package or a command
  line tool with an installer.
- **Compile time is the language.** `comptime` runs the same interpreter
  the REPL uses, so lookup tables, embedded files and tests can run before
  the C compiler sees a line.

And it is written in itself: the compiler, its tools, the test harness and
the fuzzer are Nexium programs, and the compiler builds itself from the C it
emits. The whole toolchain is one binary that needs only a C compiler.

## Installing

Nexium needs a C compiler to turn its output into machine code. Zig's
`zig cc` is the one it expects, because it cross-compiles to every platform
out of the box; on macOS the system `cc` from the Xcode command line tools is
used instead.

**Windows.** Download `nexium-<version>-setup-x64.exe` from the
[latest release](https://github.com/Londopy/nexium/releases/latest) and run
it. It installs `nx`, a bundled Zig toolchain, the standard library, the
examples, this documentation and the VS Code extension, and can add `nx` to
your `PATH`. Nothing else is needed. A portable zip without the installer is
next to it.

**macOS and Linux.**

```bash
curl -fsSL https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.sh | sh
```

The script downloads the release for your platform, verifies it against the
published checksums, installs to `~/.nexium/bin`, downloads Zig on Linux when
no C compiler is found, and adds the directory to your `PATH`. On macOS
install the Xcode command line tools first (`xcode-select --install`).

**From source.** With nothing but a C compiler:

```bash
git clone https://github.com/Londopy/nexium
cd nexium
sh bootstrap/build.sh        # .\bootstrap\build.ps1 on Windows
```

This builds `nx0` from `bootstrap/nx.c`, the C the compiler emits for
itself, then has `nx0` build the compiler from its Nexium sources, then has
that compiler rebuild itself and checks the two agree byte for byte. The
result is `nx-out/bootstrap/nx2`. Put it on your `PATH` or call it by path.

## Checking the installation

Open a new terminal and run:

```bash
$ nx doctor
nx 1.0.1 (Annapurna: Rébuffat)
executable:  /home/you/.nexium/bin/nx
C compiler:  zig cc  (zig on PATH)
             0.14.1
std modules: args bytes fs http json lists net num process regex stream strings testing text thread time
everything works. Try: nx run examples/hello.nx
```

`nx doctor` says which C compiler `nx` will run and why (`--cc` on the
command line, the `NX_CC` or `NX_ZIG` environment variables, a Zig bundled
next to `nx`, the system compiler on macOS, or `zig` on the `PATH`, in that
order), and whether it answers. If it reports `not found`, install Zig from
[ziglang.org](https://ziglang.org/download/) and put it on your `PATH`, or
set `NX_CC=gcc` (or `clang`) to use a C compiler you already have.

## An editor

The VS Code extension gives syntax colouring, diagnostics as you type, hover
with inferred effects, go to definition, completion and rename; the Windows
installer installs it, and on other platforms it is the `.vsix` attached to
every release (*Extensions: Install from VSIX...*). It talks to `nx lsp`,
the language server that is part of the compiler, and so can every other
editor: the repository's `editors/` directory has Vim, Neovim, Helix, Zed,
Emacs, Kate, JetBrains, Sublime Text, Notepad++ and nano support, each with
a README, and [the install guide](../docs/install.md#an-editor) has the
one-line version of each.

None of this is required. A text editor and a terminal are enough for every
chapter.

## How to read the code in this book

Programs appear like this, with the file they come from named above the
code:

{{include topo/code/hello.nx}}

and their output like this:

{{output topo/code/hello.expected}}

The files are under `topo/code/` in the repository, so you can run any of
them yourself:

```bash
nx run topo/code/hello.nx
```

Some chapters show a program that does not compile, on purpose, next to the
message the compiler gives. Those are recorded too: if the compiler's words
change, the page changes with them.

## The route ahead

1. Hello, world: what `nx run` does.
2. Your first script: a real program that reads a file.
3. The REPL: the language at a prompt.
4. Values and types, functions and errors, structs and traits: the language
   itself.
5. Ownership: the chapter that makes Nexium Nexium.
6. Collections, then two projects: a calculator and a to-do list.
7. Binary patterns, compile time, tests, effects.
8. Threads and parallel loops, a network service, a GUI.
9. Packages, shipping a library to other languages.
10. A neural network from nothing.
11. The tools, and the summit.

Boots on. Next: [Hello, world](02-hello-world.html).
