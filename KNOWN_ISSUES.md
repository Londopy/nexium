# Known issues

Open bugs, gaps and limitations that are known but not yet fixed. Each entry
says where it shows, how to reproduce it, and what the fix likely is. A fix
removes the entry and adds a line under `Fixed` in `CHANGELOG.md`, with a
regression test (an example with recorded output, a compile-fail case, or a
unit test).

Fixed bugs are not listed here; `CHANGELOG.md` and `git log` have them.

## Compiler

- **An integer literal under `as` is typed `i64` first, and wraps.**
  `(0xffffffffffffffff as u128) * (0xffffffffffffffff as u128)` panics
  "integer overflow in `*`": the literal is emitted as an `int64_t`, so
  -1, which `as u128` widens to 2^128-1. `0xffffffffffffffff as i128`
  gives -1 (`as f64` gives -1.0), `-9223372036854775809 as i128` gives
  9223372036854775807, and `0x1ffffffffffffffff as u128` fails in the C
  compiler. Found while writing QNI. `check_cast` (self/check_exprs.nx)
  checks the operand with no expected type, and `Types.resolve`
  (self/check.nx) defaults the open literal to `i64` with no range check;
  `let x = 0xffffffffffffffff` prints -1 the same way. Fix: in
  `check_cast`, check a literal operand, or `-` on one, with the target as
  its expected type (`300 as u8` then fails to compile, as
  `let z: u8 = 300` does); and range-check literals that fall back to
  `i64`, taking `-lit` as one literal so that
  `let y = -9223372036854775808` (a run-time panic today) is accepted.
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
- **A `u128` above `i128`'s maximum is formatted as a negative number.**
  With `let a: u128 = 0xffffffffffffffff` and `let p = a * a`,
  `println("{} {x}", .{p, p})` prints `-36893488147419103231
  -1ffffffffffffffff`, and `0 -% 1` prints `-1`; `format`, `to_string()`,
  widths and `?u128` do the same. Found while writing QNI. `write_value` in
  self/cgen.nx writes every integer as `nx_w_int(sink, (nx_i128)(v), ...)`,
  so bit 127 becomes a sign. The lexer (`number`, self/lexer.nx) keeps a
  literal as its `u128`'s `to_string()`, so 2^128-1 is rejected as
  "literal `-1`", and 2^127 becomes text that `check_lit`
  (self/check_exprs.nx) cannot parse, which its `catch { 0 }` turns into 0
  without an error, for an `i128` too (`check_lit_pattern` in
  self/check_match.nx has the same `catch`). `nx play` holds integers as
  `i128`, so `a * a` panics. Fix: an `nx_w_uint` taking an `nx_u128` in
  runtime/nx_rt.h, called by `write_value` for `u128`; `check_lit` must
  then reject what it cannot parse rather than use 0, and the interpreter
  needs a `u128` value.
- **A call through `dyn` ignores `own` parameters.** With `trait Sink {
  fn put(self: *mut Self, own s: String) }` and `d: dyn Sink`, the calls
  `d.put(s)` and `d.put(String.from("x"))` hand the String to the callee
  and still drop it in the caller: a double free, 0xC0000374 on Windows
  (found while writing QNI). A `ref class` argument is not retained: the
  callee gets a reference with no count of its own. The checker does not
  mark `s` moved, so reading it after the call compiles, and an impl
  whose `own` differs from the trait's is accepted. Fix: have
  `dyn_method_sig` (self/check_calls.nx) report the trait's `own` flags,
  call `take_ownership` on those arguments in `check_method_call`, pass
  them in `dyn_call` (self/cgen.nx) with `expr_owned` for a local or `x.?`
  on one and `simple_owned` otherwise, not `self.simple(x)`, and have
  `vtable_for` (which pairs impl and trait methods by name only) reject
  differing `own`.
- **A call through `dyn Trait` fails in C when nothing coerces to it.**
  Declaring or importing a function over a trait object breaks the build
  when no code converts a value to that `dyn` (found while writing QNI):
  ```
  trait Area { fn area(self: *Self) -> i64 }
  fn g9(d: dyn Area) -> i64 { return d.area() }
  ```
  `nx check` passes; `nx build`, `run` and `test` fail in the C compiler
  unless the method is void and takes no arguments ("incompatible type
  'void'", or "too many arguments to function call"). `emit_vtable_type`
  in `self/cgen.nx` takes the field types from a vtable instance and, with
  none, emits `void (*area)(nx_ctx*, void*)`. Fix: build the fields from
  the trait's declared signatures, as `dyn_method_sig` in
  `self/check_calls.nx` does, and check impl methods against the trait so
  the `vtable_instance` thunks still match (an impl returning `i32` for an
  `i64` builds today).

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
