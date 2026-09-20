# Tests

Tests live in the file with the code, run with one command, and are
checked by the same compiler with the same rules. A test that does not
compile is a failing test; a test that leaks is reported by `nx leaks`;
a test that could have run at compile time, can.

{{include topo/code/testing.nx}}

```bash
$ nx test topo/code/testing.nx
ok    freezing and boiling
ok    body temperature, approximately
ok    median of odd, even and empty
ok    errors
ok    text

5 passed, 0 failed
```

## `test` blocks

```nexium
test "freezing and boiling" {
    expect_eq(to_celsius(32.0), 0.0)
```

A `test "name" { }` block is a function with no arguments that `nx test`
runs, in file order, each in its own process state. `expect(cond)` and
`expect_eq(a, b)` are builtins; a failed expectation panics with the values
and the location, the test is reported as failed, and the run continues
with the next one. `nx run` ignores test blocks entirely, so they cost the
program nothing.

Inside a test, `try` is allowed (a test that hits an unexpected error fails
with the error's name), `defer` works, and any function in the file is
reachable, `pub` or not. Every module of the standard library is tested this
way, in its own file: `nx test std/strings.nx` runs the strings module's
tests.

## `std.testing`

The builtins cover most tests. `std.testing` adds the assertions that are
tedious to write by hand:

- `expect_approx(a, b, eps)` for floats.
- `expect_err(T, r)` and `expect_error(T, r, error.Name)` for results
  that should fail.
- `expect_contains(text, needle)` and `expect_lines(actual, expected)`,
  which names the first line that differs.
- `snapshot(name, actual)`: compares `actual` with
  `snapshots/<name>.txt`, creates the file the first time, and rewrites it
  when `NX_UPDATE_SNAPSHOTS=1` is set. Snapshot tests are how a program's
  whole output is kept honest with one line.

## The command

```bash
nx test file.nx                    # every test in the file
nx test file.nx --filter median    # only the tests whose name contains it
nx test file.nx --verbose          # with timings
```

The exit code is the number of failures, so CI sees a red run without
parsing anything.

## Compile-time tests

```nexium
comptime test "runs while compiling" {
    expect_eq(median([5][..]) orelse 0, 5)
}
```

The last chapter introduced these. A `comptime test` runs in the
interpreter during checking; it is not listed by `nx test` because it
already ran when the file was checked, and a failure is a compile error.
Use them for pure functions whose examples are worth stating next to the
code, and for facts about constants.

## What the test suite of this book looks like

The programs in these chapters are tested by the same harness that tests
the compiler: `nx run tests/run.nx -- topo` runs every one, compares its
output with the recorded `.expected` file the page shows, runs the `test`
blocks in the calculator and this chapter's file, builds the packages
example from its own directory, renders the GUI chapter's frame, and builds
this site. The harness is a Nexium program, `tests/run.nx`, which is worth
a read once you have finished the book: it is a 400-line example of process
control, threads and file handling in the language.

Next: [effects](15-effects.html), the feature that is Nexium's own.
