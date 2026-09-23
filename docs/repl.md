# The interactive session

`nx` with no arguments at a terminal, or `nx repl` anywhere, opens a prompt:

```
Nexium 1.2.1 (nx repl, zig bundled with nx) on windows
Type :help for commands, :quit to exit.
> let xs = [3, 1, 2]
> import std.lists
> lists.max(i32, xs[..])
?i32: 3
> fn twice(x: i32) -> i32 { return x * 2 }
> twice(21)
i32: 42
> var s = String.from("nex")
> s.append("ium")
> s
String: "nexium"
```

## How it works

Each line becomes part of a program: declarations (`fn`, `struct`, `enum`,
`import`, ...) at the top level, everything else inside a `main`. The whole
program is re-checked on every line, so the prompt reports exactly what the
compiler would: type errors, effect violations, use after move. Only the new
statements are executed, by the compiler's interpreter (the same one that
runs `comptime`), against the values kept from earlier lines. A line that
fails to check is not kept; a line that panics is reported and not kept.

An expression on its own prints its type and value. `let` and `var`
bindings persist. Lines with unclosed brackets continue on the next prompt.

## The prompt

At a terminal the line colours as you type (keywords, strings, numbers,
types, comments), Tab completes the name or the member before the cursor
from what the language server's completion finds at that point (one
candidate goes in, a shared prefix goes in, several are listed), the
arrows and Home/End move, Up/Down walk the history, Ctrl-A/E jump to the
ends, Ctrl-U/K cut, Ctrl-L clears the screen, Ctrl-C drops the line and
Ctrl-D at an empty line leaves. The console is in raw mode only while a
line is typed; a statement runs with the console as it was. `NX_PLAIN=1`
asks for the plain reader (no editor), `NO_COLOR` keeps the editor
without the colours, and a pipe gets the plain reader by itself.

## Commands

| command | effect |
| --- | --- |
| `:help` | the summary |
| `:quit`, `:q`, `exit` | leave |
| `:vars` | the kept bindings and their values |
| `:items` | the declared items |
| `:load FILE` | add the items of a file to the session |
| `:undo` | take back the last line; the bindings return to what they were |
| `:save FILE` | write the session as a program `nx run` runs (values printed at the prompt become `_ = ...`) |
| `:effects EXPR` | the effects of an expression, with the kept bindings in scope: `:effects io.read_file("x")` |
| `:reset` | start over |

## One-liners: `nx -e` and `nx -p`

```sh
nx -p "2 * 21"                                   # 42
nx -e 'println("{}", .{strings.to_upper("hi")})'   # after a line: import std.strings
nx -e "fn f() -> i32 { return 7 }
println(\"{}\", .{f()})"
```

`nx -e CODE` runs the lines of `CODE` as the prompt would, one after
another, items and statements alike, and prints nothing but what the code
prints; `nx -p EXPR` prints the value of the last expression on its own,
the way a shell expects. The interpreter runs them, so nothing is
compiled; the exit code is 1 when a line does not check or the value is
missing.

## A whole program: `nx play`

```sh
nx play examples/hello.nx
nx play < program.nx
```

`nx play` checks a whole program the way `nx check` does, then runs its
`main` in the same interpreter, so nothing is compiled and no C compiler
is needed. It is the command the site's playground runs: the compiler is
built as WebAssembly (`site/play_build.sh`), and the page hands it the
code of an exercise or an example on stdin and shows what comes back.
The exit code is `main`'s own, 1 after diagnostics or an error returned
from `main`, and 2 after a panic or a program the interpreter cannot run.

A program prints the same interpreted as compiled, or says it cannot be
interpreted; it never prints something different. The harness holds it to
that: every spec case, example and Topo program runs both ways (the `play`
suite), and CI runs every exercise through the WebAssembly build and the
page's own script (`site/play_test.mjs`).

## Limits

The interpreter covers the language but not the platform: foreign calls
(`@cImport`, `extern`), threads, sockets, processes, mutable globals,
`@refCount` and `u128` values past `i128`'s maximum (the interpreter holds
integers as `i128`) need a compiled program (`nx run`), and the message
names the line of your program that led there. `for parallel` runs its iterations
in order. Values that only exist at compile time (function values,
pointers) are not kept between lines. Speed is interpreted speed, and a
run stops after 20 million steps; measure with `nx run`. In the page, a
recursion stops at 400 nested calls (the browser's stack is smaller than
a native thread's), there are no files to read, and `time.sleep` waits a
second at most.

In the session the program may do I/O (`println`, `io.read_file`,
`io.read_line`, `os.env`, `time.now`, `time.sleep`), which `comptime` in a
compiled program never may.
