# Known issues

Open bugs, gaps and limitations that are known but not yet fixed. Each entry
says where it shows, how to reproduce it, and what the fix likely is. A fix
removes the entry and adds a line under `Fixed` in `CHANGELOG.md`, with a
regression test (an example with recorded output, a compile-fail case, or a
unit test).

Fixed bugs are not listed here; `CHANGELOG.md` and `git log` have them.

## Compiler

- **A contained panic leaks what the call acquired.** An export's wrapper
  (`self/cgen.nx`, the `NX_EXPORT` shape) sets a boundary and `nx_panic`
  longjmps to it, so every drop between the panic and the boundary is
  skipped: a `String` of 1 MiB built and then divided by zero leaks 1 MiB
  per call, and a file opened before the panic keeps its slot in the
  runtime's table (64 of them). The host survives, as S3 promises, but a
  request handler that keeps failing grows without bound. Reproduce:
  ship a function that allocates and panics, call it 100 times from
  Python, watch RSS. Planned fix, in the runtime: every export call gets
  a tracker; the call's allocator wraps the default one and records live
  allocations in a hash set (a spinlock, since a `for parallel` body
  copies the context to other threads), `nx_file_open`, socket opens and
  mutex locks register with the boundary in the same way, and the
  `setjmp` branch releases everything still registered before returning
  the panic status (a normal return only frees the table). Exports
  cannot return heap values or reach globals (S1, S2), so nothing
  allocated during a panicked call is reachable afterwards; thread and
  parallel boundaries keep leaking on a panic, since what a thread
  allocates can escape through `shared_mutable`. Regression test: a
  shipped library whose export opens a file and panics, called more times
  than the file table holds, then an open that must succeed; and the CI
  Python step measuring peak RSS over 100 panicking calls. The same
  mechanism is the first half of the roadmap's 1.2 (a panic releases
  what it owned), and until it lands `docs/embedding.md` says so.
- **Compile-time recursion is limited to 32 nested calls.** The
  compile-time interpreter recurses on the compiler's own stack, and each
  nested call costs about 165 KiB of it (the interpreter's large functions
  declare every temporary at function scope in the generated C), so a
  `comptime` call chain 100 deep overflowed a 16 MiB stack and crashed
  the compiler (found by the fuzzer, which turned `fib` into an unbounded
  recursion). The interpreter now stops at 32 nested calls with a
  diagnostic; `fib(30)` is fine (its depth is 30), a recursive descent
  parser at compile time may not be. Planned fix: `artifact cli { stack
  = "1G" }`, a runtime `nx_run_on_stack` that runs `main` on a thread with
  that reservation, and the compiler declaring it for itself, after which
  the limit becomes a few thousand; separately, `self/cgen.nx` scoping
  temporaries to their blocks would shrink every frame.
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
