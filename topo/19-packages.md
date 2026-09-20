# Packages

A package is a directory with a `nexium.toml` and a `src/` folder of
modules. A program depends on one by path or by git tag, and imports its
modules like any other. There is no registry and no build script: a
dependency is source code the compiler reads.

## Modules first

Every `.nx` file is a module. `import foo` loads `foo.nx` next to the file
that says so, and its `pub` items are reached as `foo.item`. That is how
the GUI chapter found `gui/nexium_gui.nx` from `gui/counter.nx`, and it is
enough for a program that is a handful of files in one directory.

A package is the same idea with a boundary around it, so that a directory
of modules can be used by programs elsewhere without knowing its layout.

## A package and a program that uses it

The book's example is two directories:

```
topo/code/pkg/
  words/
    nexium.toml
    src/
      lib.nx        # `import words`
      cases.nx      # `import words.cases`
  app/
    nexium.toml     # depends on ../words
    main.nx
```

The package:

{{include topo/code/pkg/words/nexium.toml}}

{{include topo/code/pkg/words/src/lib.nx}}

{{include topo/code/pkg/words/src/cases.nx}}

Inside the package, `import cases` refers to the package's own
`src/cases.nx`, never to a file of the program that uses it, so two
packages may both have a module called `util`. `src/lib.nx` is what
`import words` means from outside; the other modules are reached as
`words.cases`. Only `pub` items cross the boundary.

The program:

{{include topo/code/pkg/app/nexium.toml}}

{{include topo/code/pkg/app/main.nx}}

```bash
$ cd topo/code/pkg/app
$ nx run main.nx
SUMMIT!
quiet
```

The compiler finds `nexium.toml` by walking up from the root source file,
reads the `[dependencies]` table, and resolves `import words` through it.

## `nx init`, `nx add`, `nx fetch`

```bash
nx init                                    # writes nexium.toml (and main.nx in an empty folder)
nx add words --path ../words               # a directory on disk
nx add greet --git https://github.com/someone/greet --tag v1.2.0
nx fetch                                   # clone the git dependencies, write nexium.lock
```

`nx add` records the dependency; `nx fetch` clones every git dependency,
transitively, into `nexium_modules/` next to the manifest with a shallow
checkout of the tag, and writes `nexium.lock` with the exact commit each one
resolved to. Commit the lock file; commit `nexium_modules/` too if you want
builds with no network (vendoring), or ignore it and fetch after cloning.

To publish a package: push the repository and tag it. Users depend on the
tag; the lock file pins the commit, so a moved tag changes nobody's build
until they fetch again.

## Testing a package

A package's tests are `test` blocks in its modules, run from its own
directory:

```bash
$ cd topo/code/pkg/words
$ nx test src/lib.nx
ok    shout

1 passed, 0 failed
```

## Rules worth knowing

- The dependency's name in the manifest is the identifier programs import.
- The first package to claim a name wins across the whole dependency graph;
  version conflicts are not resolved automatically, since there are no
  version ranges to resolve, only tags.
- `nexium.toml` is a small TOML subset: tables, strings, inline tables,
  arrays of strings, comments.
- A package may declare `artifact link { c_sources = [...] }` for C it
  ships with (the GUI does), and the program's build compiles it in.

The [packages document](../docs/packages.html) has the details.

Next: [shipping a library](20-shipping.html).
