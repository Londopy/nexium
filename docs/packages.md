# Packages

A package is a directory with a `nexium.toml` manifest and a `src/` folder
of modules. Programs use packages as dependencies; dependencies come from a
git tag or a directory on disk. No registry is needed.

## A program with dependencies

```sh
nx init                     # writes nexium.toml (and main.nx when the folder is empty)
nx add greet --git https://github.com/someone/greet --tag v1.2.0
nx add sdk --git https://github.com/someone/app --tag v2.0.0 --dir nexium   # a package in a directory of a repository
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
sdk = { git = "https://github.com/someone/app", tag = "v2.0.0", dir = "nexium" }
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

A package may live in a directory of a larger repository, an SDK beside
the app it belongs to: the dependency names it with `dir = "nexium"`, and
the package's `nexium.toml` and `src/` sit in that directory. The lock
pins the repository's commit as for any git source.

To publish, push the repository and tag it (`git tag v1.2.0 && git push
--tags`). Users depend on the tag. Bump the tag for every release; the lock
file pins the commit, so moving a tag does not change anyone's build until
they run `nx update`.

## Packages you can `nx add` today

There is no registry yet (it is on the roadmap, under *Ecosystem*); this
table is the registry until then. A package is any git repository with a
`nexium.toml` at its root and a tag to pin:

```sh
nx add words --git https://github.com/Londopy/nexium --tag v1.1.0
```

| package | what it is | add it with |
| --- | --- | --- |
| `words` (in this repository, `topo/code/pkg/words`) | the Topo's example package: a few functions over words, the layout to copy | a `path` dependency on a checkout, or copy the two files |
| `app` (in this repository, `topo/code/pkg/app`) | the program that depends on `words`; the shape of a program with dependencies | the same |
| `discord_rpc` ([statusmith](https://github.com/Londopy/statusmith), its `nexium/` directory) | Discord Rich Presence over the local pipe: `connect`, `set_activity`, `clear`, `close`; Windows; [the page](discord.html) | `nx add discord_rpc --git https://github.com/Londopy/statusmith --tag sdk-v0.1.0 --dir nexium` |

The standard library's next modules land in `std/`, not here; the packages
people write are still mostly in their programs.

To be listed, open a pull request adding a row: the repository, one line
of what it does, the tag to pin. The package has to build with the
current release (`nx check` on its `src/`), carry a license, and keep its
name a valid identifier.

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
