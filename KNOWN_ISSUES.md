# Known issues

Open bugs, gaps and limitations that are known but not yet fixed. Each entry
says where it shows, how to reproduce it, and what the fix likely is. A fix
removes the entry and adds a line under `Fixed` in `CHANGELOG.md`, with a
regression test (an example with recorded output, a compile-fail case, or a
unit test).

Fixed bugs are not listed here; `CHANGELOG.md` and `git log` have them.

## Compiler

- **R1 rejects a slice into a loop item's buffer over borrowed storage.**
  `for s in xs { return s }` with `xs: []String` a parameter (or
  `return s.text` in `for s in self.sections` in a `self: *Self` method,
  found while writing QNI) fails: "this returns a slice into `s`, a local
  that is released when the function returns (region rule R1)". The slice
  is into the caller's element's heap buffer and outlives the call;
  `return self.sections[i].text` and `let t: []u8 = s.text; return t`
  pass. `if let` over a place, `for (k, v) in` and a `match` arm over a
  by-value parameter fail the same way. Fix: in `check_escaping_view`
  (self/check_views.nx), when `view_root` lands on a binding that views a
  value (`loop_item`, `if_let`, `tuple_view`; a pattern binding has no
  flag yet) and the view goes through a `String`, `List` or `Map` buffer,
  judge the binding's origins as the V1 branch does. Keep the error for
  `&x` or `s.arr[..]`: the binding is a stack copy, so those do dangle.

## Self-hosting

- **`nx tir --sigs` omits body-dependent facts** (error ids, alias types,
  and trait default methods materialized by calls) rather than being
  order-independent by construction. The full mode is complete; `--sigs`
  exists only as the first milestone of `self/check.nx`.

## Tools and editors

- **`nx fmt` collapses aligned trailing comments.** A struct whose fields
  carry comments aligned in one column (`kind: u32            // 0
  playing`) is rewritten with a single space before each comment, losing
  the alignment the author chose; found on statusmith's `Activity` and
  Point of Origin's `Level`. Fix: when consecutive lines end in a
  comment, keep the column of the first (or the widest code) for the
  run; a tree-wide `--check` guards it.
- **`nx fmt` writes `! (x)`.** A negated parenthesized condition
  `if !(a or b)` becomes `if ! (a or b)`: the no-space rule after `!`
  does not apply before `(`. Fix: `!` followed by `(` is tight, like `!x`.
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
