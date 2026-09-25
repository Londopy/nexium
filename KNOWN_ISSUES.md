# Known issues

Open bugs, gaps and limitations that are known but not yet fixed. Each entry
says where it shows, how to reproduce it, and what the fix likely is. A fix
removes the entry and adds a line under `Fixed` in `CHANGELOG.md`, with a
regression test (an example with recorded output, a compile-fail case, or a
unit test).

Fixed bugs are not listed here; `CHANGELOG.md` and `git log` have them.

## Compiler

- **An owned if-expression passed straight to a `[]u8` parameter is freed
  before the call reads it.** With `fn show(s: []u8)`,
  `show(if n > 2 { format("big {}", .{n}) } else { String.from("small") })`
  prints garbage on Linux (glibc hands the block straight back) and
  happens to work on Windows: the branch's owned String is dropped at the
  end of its branch, so the slice the call gets points into freed memory.
  Binding it first (`let t = if ...`), a format argument and `return` are
  all fine. Found by QNI's tests, which CI builds against glibc; still in
  1.3.0. Fix: an if-expression whose value is an owned temporary coerced
  to a slice argument keeps that temporary alive until the call returns,
  like any other owned argument.
- **`println` of an empty `String` passes NULL to `memcpy`.**
  `println("[{}]", .{String.new()})` traps in a debug build against glibc
  ("null pointer passed as argument 2, which is declared to never be
  null", in `nx_w`): the file sink copies with `memcpy(s->buf + s->n, p,
  n)` even when `n` is 0, and an empty String's pointer is NULL. It is
  undefined behaviour on every platform; `format` into a String is fine,
  because `nx_str_append` checks `if (n)`. Found by nxtls's live probe;
  still in 1.3.0. Reproduce: `nx build x.nx --target x86_64-linux-gnu`,
  run it on Linux. Fix: return from `nx_w` when `n == 0`.
- **`starts_with` an empty prefix with a NULL pointer calls `memcmp(a,
  NULL, 0)`.** `"abc".starts_with(String.new())` traps the same way, in
  `nx_sl_starts_with`, which lacks the `p.len == 0` guard that
  `nx_sl_ends_with` has. A `""` literal has a real pointer, so only an
  empty String (or a slice of one) shows it. Found by QNI, whose settings
  have prefixes that can be empty; still in 1.3.0. Fix: the same guard as
  `nx_sl_ends_with`.

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

## Packaging

- **The x86_64 Linux binary needs glibc 2.34 but claims 2.17.** The
  release builds it natively with `zig cc` on `ubuntu-latest`, which
  targets the runner's glibc (2.39), so the binary needs
  `__libc_start_main` and `pthread_create` at `GLIBC_2.34` (and `stat` at
  2.33, `pow` at 2.29). The wheel is tagged `manylinux_2_17_x86_64`, so pip
  installs it on Ubuntu 20.04, Debian 11 or Amazon Linux 2, where `nx`
  fails with "GLIBC_2.34 not found"; the `.deb` and `install.sh` have the
  same floor. Colab and anything on glibc 2.34 or later are unaffected.
  Reproduce: build the seed as the release does (`zig cc -target
  x86_64-linux-gnu.2.39 -std=gnu11 -O2 bootstrap/nx.c -lm -lc`) and list
  the versions it needs (`objdump -T nx | grep -o 'GLIBC_[0-9.]*' | sort
  -uV`). Fix: `-target x86_64-linux-gnu.2.17` for the release's Linux
  build (it links, and needs nothing past 2.17; the aarch64 cross build
  already targets 2.17), and a release step beside the AVX check that
  fails when the highest version needed is past 2.17.

## Tests and CI

- **Suites that build files must pass `--out-dir`.** Two cases compiling
  the same source into `nx-out/` at once fail on Windows (the second write
  hits a mapped file). The harness (`tests/run.nx`) gives every case its
  own directory under `nx-out/cases/`; a new suite has to do the same by
  hand, nothing checks it.
- **`process` drains a child's output after its stdin is fully written.**
  A child that produces more than the pipe holds before reading its input
  can stall; feed such programs through files (`std.process` says so).
