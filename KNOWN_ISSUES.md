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

- **A checkout of the compiler inside a package's directory builds as
  part of that package.** `nx` looks for `nexium.toml` from the working
  directory up, and Nexium's repository has none at its root, so
  `bootstrap/build.sh` run in a checkout that sits inside a package
  (nexium-discord's first CI did this) reaches stage 1 and stops: the
  outer package's dependencies are "not fetched". Fix: stop the search at
  a repository's root (a `.git`), or give the compiler's own build a way
  to say it is no package's (a flag, or a `nexium.toml` for the
  repository); a case in the bootstrap suite.

- **A WebAssembly build stops inside clang with Zig 0.16.** `nx build
  --target wasm32-wasi` and `artifact wasm` turn on WebAssembly's
  exception handling for the panics' `setjmp` (decision 123), and with it
  on, Zig 0.16 compiles wasi-libc's own `setjmp` runtime
  (`libc-top-half/musl/src/setjmp/wasm32/rt.c`), on which its clang 21
  fails with "error in backend: undefined tag symbol cannot be weak", and
  `nx` reports "C compilation failed". Native builds work with Zig 0.16,
  and every build works with the pinned 0.14.1, which the installer and
  the install script bring. Reproduce: Zig 0.16 first on the PATH and
  `nx build examples/hello.nx --target wasm32-wasi`, or the harness's
  `ship` suite. Fix: find what Zig 0.16 needs to build that runtime
  (`runtime/nx_wasm_sjlj.c` supplies it for 0.14, which leaves it out, and
  gives way to a libc that has it), and a CI job that builds for
  `wasm32-wasi` with the newest Zig beside the pinned one.

## Diagnostics

Places where a mistake in a program's own code is reported inside the
standard library, so that fixing it means reading std's source (the soft
goal of SPEC 1; the roadmap's "Errors you can fix alone").

- **A failed `std.testing` assertion points at std, not at the test.**
  `testing.expect_approx(total(xs) as f64, 7.0, 0.001)` in a test fails
  with `panic: expected 6.0 to be within 0.001 of 7.0` `at
  <std>/testing.nx:37`, the line in std that panicked; the test's own line
  is nowhere, so a test with several assertions does not say which one
  failed. The same holds for `expect_err`, `expect_error`,
  `expect_contains`, `expect_lines` and the snapshots. The builtin
  `expect` is right: it reports the test's line and the expression. Fix:
  a panic in a function that checks its caller's input reports the
  caller's location, as Rust's `#[track_caller]` does (a function
  attribute, or every `std.testing` function, whose panics take the call
  site as a hidden argument); a test whose failure is recorded with its
  own line.
- **A std function's check on its arguments panics at std's line.**
  `hash.siphash("too short", "hello")` stops with `panic: siphash takes a
  16-byte key` `at <std>/hash.nx:44`, and the program's call is not in
  the message. Fix: the same caller location, for the std functions that
  panic about their arguments, and a case with the caller's line
  recorded.
- **A generic std function refused for a type reports it three times.**
  `sort.is_sorted(Point, ps[..])`, where `Point` has no `Ord`, gives the
  right error at the call ("`is_sorted` requires `T: Ord`, but `Point`
  does not implement `Ord`"), then two more for the same mistake inside
  `<std>/sort.nx` (the bound at 165:27, the `<` at 168:12), and the one
  that says how to fix it ("add `derive(Ord)` to the struct") is the last.
  Fix: a call that fails a generic's `where` bound does not instantiate
  the generic with that type, so its body is not checked, and the first
  error carries the hint; a compile-fail case expecting the hint and one
  error.
- **A stack overflow is a bare crash.** A recursion deeper than the stack
  ends the program with the system's fault (a segmentation fault, exit 139,
  on Linux; `0xC00000FD` on Windows) and no message naming the overflow or
  the function that recursed. The fuzzer took one for a memory error: a
  mutant of `tests/spec/s3_entry_stack.nx` that lost its `artifact cli {
  stack }` overflowed (its mutants are no longer run). Fix: the runtime
  catches the overflow on an alternate stack (`sigaltstack` with a
  `SIGSEGV` handler on Linux and macOS, a vectored handler for
  `EXCEPTION_STACK_OVERFLOW` on Windows) and reports `panic: stack
  overflow` with the function, as a panic does; a spec case that
  overflows on purpose.

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
