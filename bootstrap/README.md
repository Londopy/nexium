# bootstrap

How `nx` is built on a machine that has no `nx`.

- `nx.c` — the seed: the C that the compiler written in Nexium (`self/`)
  emits for itself, as of the last release, in `safe` mode. Any C compiler
  builds it. It is regenerated at each release, and whenever `self/` needs
  a builtin or a language feature the seed does not understand, by the
  current compiler (`nx emit-c self/nx.nx --mode safe > bootstrap/nx.c`);
  otherwise it may be a little behind `self/` (decision 90).
- `build.sh`, `build.ps1` — the three stages: the seed builds `nx0`, `nx0`
  builds the current `self/nx.nx` into `nx1`, and `nx1` must rebuild itself
  to the same C (`nx2`). The result is `nx-out/bootstrap/nx2`: `nx1` was
  linked against the runtime header the seed carries, `nx2` against the one
  in `runtime/`, so a runtime change is live in `nx2`.
- `rust/` — the first compiler, written in Rust, frozen at 0.7 semantics.
  It still builds (`cargo build`); every tool it held is now in `self/` and
  the test harness is `tests/run.nx`. It is deleted at 1.0. Language changes
  do not go in it.

`nx run tests/run.nx` runs the same chain and every suite through `nx2`.
CI runs `build.sh` on the three platforms with no Rust toolchain installed.
