# Known issues

Open bugs, gaps and limitations that are known but not yet fixed. Each entry
says where it shows, how to reproduce it, and what the fix likely is. A fix
removes the entry and adds a line under `Fixed` in `CHANGELOG.md`, with a
regression test (an example with recorded output, a compile-fail case, or a
unit test).

Fixed bugs are not listed here; `CHANGELOG.md` and `git log` have them.

## Compiler

- **No `.clone()` on user structs.** `List`, `String` and `Map` clone, but a
  struct holding them cannot be copied without writing a function by hand;
  code that needs a copy of a list element must return an index instead
  (`self/check.nx`, `find_label`). Likely fix: derive `clone` for value
  structs and enums whose fields all clone, the way drops are derived
  (decision pending).
- **Range facts stop at `if` guards.** A `while c < n { }` body and an
  `else` branch get no range facts from their conditions, so arithmetic
  and indexing there carry checks that a guard would have discharged. The
  `if` case is handled (and scoped to its block since 0.6.0). An
  optimization gap, not a soundness problem.

## Self-hosting

- **`nx tir --sigs` omits body-dependent facts** (error ids, alias types,
  and trait default methods materialized by calls) rather than being
  order-independent by construction. The full mode is complete; `--sigs`
  exists only as the first milestone of `self/check.nx`.

## Tools and editors

- **Formatter bar classification has no unit test.** `nx fmt` tells
  closure bars from bit-or per line (`self/fmt.nx`, `bar_role`); the tree-wide
  `--check` in CI is the only guard. Add cases for `|x| x | 1`, `a | b`,
  `f(|x| x)`, `Task(T, R)|`.
- **tree-sitter: a binary pattern after a braced arm parses as a shift.**
  In `tests/parse_smoke.nx`, `{ return total_len }` followed by a
  `<<...>>` arm on the next line is read as `{...} << ...`, because the
  grammar ignores newlines. Editors show one error there; the file is
  therefore not in CI's parse list. Fix: make the newline before `<<`
  significant in arm position, or require a comma after braced arms in
  the grammar.

## Tests and CI

- **Suites that build files must pass `--out-dir`.** Two cases compiling
  the same source into `nx-out/` at once fail on Windows (the second write
  hits a mapped file). The harness (`tests/run.nx`) gives every case its
  own directory under `nx-out/cases/`; a new suite has to do the same by
  hand, nothing checks it.
- **`process` drains a child's output after its stdin is fully written.**
  A child that produces more than the pipe holds before reading its input
  can stall; feed such programs through files (`std.process` says so).
