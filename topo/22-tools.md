# The tools

One binary, `nx`, and inside it everything the book has used and a few
things it has not. All of them are views over the same compiler, which is
why they agree with each other, and all of them are Nexium programs in the
compiler's `self/` directory.

## Building and running

| command | what it does |
| --- | --- |
| `nx run file.nx [-- args]` | build and run |
| `nx build file.nx [-o out]` | build an executable (or an object file when there is no `main`) |
| `nx test file.nx [--filter s] [--verbose]` | run the `test` blocks |
| `nx check file.nx` | check without building; the diagnostics only |
| `nx emit-c file.nx` | print the generated C |

`--mode debug` (the default: checks on, symbols in), `safe` (optimized,
checks on), `fast` (optimized, some checks proven away), `small` (size
first). `--target <triple>` cross-compiles; `--cc clang` names a C
compiler; `--keep-c` leaves the C file in `nx-out/`.

## Looking at a program

**`nx effects`** prints the inferred effects of every function (chapter
15). **`nx audit`** lists every `unsafe` block and mutable global: the
places the compiler's promises stop. **`nx refcounts`** lists every place a
reference is retained or released:

```
$ nx refcounts topo/code/ownership.nx
reference count traffic: 13 site(s)

topo/code/ownership.nx:61:5  release at scope exit
    in main
topo/code/ownership.nx:61:17  allocation with count 1
    in main
topo/code/ownership.nx:61:43  retain (copy of a reference)
    in main
```

**`nx leaks`** runs a debug build with every allocation counted and reports
what was alive at exit. **`nx size`** builds in small mode and attributes the
executable's bytes to declarations, which answers "what is this
binary made of":

```
$ nx size topo/code/hello.nx
    bytes  kind  declaration
     1025  code  main (entry point)
      256  data  (section .tls$$nx_tls_last_panic)
       73  data  (section .rdata)
       38  data  (string literals)
     1072        runtime and libc glue
     2472        total (small mode; the linker drops unreferenced sections)
```

Two and a half kilobytes of program for `hello, world`, which is what a
runtime that is a header of `static inline` functions buys.

## Formatting and documentation

**`nx fmt file.nx`** rewrites the file in the one canonical layout;
`--check` only reports, and is what CI runs on every file in this
repository. The formatter settles spacing and never joins or splits lines,
so the shape of your code stays yours.

**`nx doc file.nx`** renders the `///` comments, signatures and inferred
effects of a file to HTML, one page per file; the [standard library
reference](../docs/std.html) is built from the same comments.

**`nx fix file.nx`** applies the mechanical migrations the compiler knows;
there are none in 1.0, and the [stability policy](../docs/stability.html)
says how one would be introduced.

## The editor and the prompt

**`nx lsp`** is the language server: diagnostics on every edit, hover with
inferred effects, go to definition, completion and rename. The VS Code
extension in the repository starts it; any editor that speaks the protocol
can. **`nx repl`** (chapter 4) is the prompt.

## Shipping

**`nx ship file.nx`** (chapter 20) produces the file's declared artifacts;
`--target` cross-compiles them. **`nx init`**, **`nx add`** and **`nx
fetch`** (chapter 19) manage a manifest.

## The installation

**`nx version`** prints the version and its release name. **`nx doctor`**
prints which C compiler will run and why, whether it answers, the standard
library modules, and whether `nx` is on the `PATH`; it is the first thing to
run when a build does something odd.

## Under the hood

Every one of these is a file in the compiler's source: `self/fmt.nx`,
`self/doc.nx`, `self/tools.nx` (effects, audit, refcounts), `self/size.nx`,
`self/lsp.nx`, `self/repl.nx`, `self/ship.nx`, `self/manifest.nx`, and the
driver `self/nx.nx` that dispatches to them. The [architecture
tour](../docs/architecture.html) walks through the compiler itself, stage
by stage; the tools are the shorter reads, and `self/tools.nx` at a hundred
lines is the shortest.

Next: [the summit](23-summit.html).
