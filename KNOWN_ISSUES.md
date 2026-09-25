# Known issues

Open bugs, gaps and limitations that are known but not yet fixed. Each entry
says where it shows, how to reproduce it, and what the fix likely is. A fix
removes the entry and adds a line under `Fixed` in `CHANGELOG.md`, with a
regression test (an example with recorded output, a compile-fail case, or a
unit test).

Fixed bugs are not listed here; `CHANGELOG.md` and `git log` have them.

## Compiler

- **`import a.b.{X, Y}` does not parse.** The parser has a form that
  imports items by name (`self/parser.nx`, `parse_import`: a `.` then
  `{`), and the checker handles it (`has_name` imports), but the lexer
  reads `.{` as one token, the anonymous literal's opener, so written the
  natural way it fails with "expected a newline after declaration but
  found `.{`". The form is in no document (SPEC 11 has `import a.b` only),
  and `impl http.Transport for T` does not need it (it names the trait
  with its module). Fix, if the form is wanted: accept `DotLBrace` in
  `parse_import`, say so in SPEC 11, and add a spec case.

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
- **`nx fmt` glues a public generic struct's brace.** `pub struct
  Deque(T){` keeps no space before `{`, while `struct Input(T) {` (not
  `pub`) gets one: the rule that tells a generic literal (`Pair(i32){`)
  from a declaration finds `struct` before the name only when it begins
  the line. std.deque, std.heap and std.http's `Streaming` and `Client`
  are written the glued way because of it. Fix: look past `pub` (and the
  other declaration keywords) to `struct`/`enum` before the name, as the
  identifier rule does with `declared`, and respace the tree.
- **Formatter bar classification has no unit test.** `nx fmt` tells
  closure bars from bit-or per line (`self/fmt.nx`, `bar_role`); the tree-wide
  `--check` in CI is the only guard. Add cases for `|x| x | 1`, `a | b`,
  `f(|x| x)`, `Task(T, R)|`.

## Tests and CI

- **Suites that build files must pass `--out-dir`.** Two cases compiling
  the same source into `nx-out/` at once fail on Windows (the second write
  hits a mapped file). The harness (`tests/run.nx`) gives every case its
  own directory under `nx-out/cases/`; a new suite has to do the same by
  hand, nothing checks it.
- **`process` drains a child's output after its stdin is fully written.**
  A child that produces more than the pipe holds before reading its input
  can stall; feed such programs through files (`std.process` says so).
