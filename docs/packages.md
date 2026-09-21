# Packages

A package is a directory with a `nexium.toml` manifest and a `src/` folder
of modules. Programs use packages as dependencies; dependencies come from a
git tag or a directory on disk. No registry is needed.

## A program with dependencies

```sh
nx init                     # writes nexium.toml (and main.nx when the folder is empty)
nx add greet --git https://github.com/someone/greet --tag v1.2.0
nx add local --path ../local
nx run main.nx
```

`nx add` records the dependency in the manifest and fetches it. The
manifest after those two commands:

```toml
[package]
name = "app"
version = "0.1.0"

[dependencies]
greet = { git = "https://github.com/someone/greet", tag = "v1.2.0" }
local = { path = "../local" }
```

In the program:

```nexium
import greet            // greet's src/lib.nx, used as `greet.hello()`
import greet.extra      // greet's src/extra.nx, used as `extra.punct()`
import local
```

`nx fetch` clones every git dependency (transitively) into
`nexium_modules/` next to the manifest with a shallow checkout, and writes
`nexium.lock` with the exact commit each one resolved to. When a lock file
exists, `nx fetch` checks each dependency out at the commit it names,
whatever the tag points at today, so a fresh clone of your repository
builds exactly what you built; `nx update` (all of them) or `nx update
name` resolves the tag again and rewrites the lock. Commit the lock file.
Commit `nexium_modules/` too if you want a build with no network at all
(vendoring); otherwise add it to `.gitignore` and run `nx fetch` after
cloning.

The compiler finds the manifest by walking up from the root source file, so
`nx run src/main.nx` and `nx run main.nx` both work.

## Writing a package

```
greet/
  nexium.toml
  src/
    lib.nx        # `import greet`
    extra.nx      # `import greet.extra`
    util.nx       # private helper: `import util` inside the package
```

Inside a package, `import util` refers to the package's own `src/util.nx`,
never to a file of the program using the package; two packages may both
have a `util` module. A package's own dependencies go in its manifest and
are fetched along with it. Only `pub` items are visible to importers.

To publish, push the repository and tag it (`git tag v1.2.0 && git push
--tags`). Users depend on the tag. Bump the tag for every release; the lock
file pins the commit, so moving a tag does not change anyone's build until
they run `nx update`.

## Rules

- A dependency's name in the manifest is the identifier programs import;
  it must be a valid identifier and must match the `name` in the
  dependency's own manifest for clarity, though the compiler uses the
  manifest key.
- The first package to claim a name wins across the whole dependency graph
  (a transitive dependency with a clashing name is skipped with the root's
  choice in force). Version conflicts are not resolved automatically.
- `nexium.toml` uses a small TOML subset: tables, strings, inline tables,
  arrays of strings, comments. `[package]` needs `name`; `version` is
  informational until a registry exists.
- The REPL and the language server do not load packages yet; `nx run`,
  `nx build`, `nx test` and `nx check` do.
