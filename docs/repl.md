# The interactive session

`nx` with no arguments at a terminal, or `nx repl` anywhere, opens a prompt:

```
Nexium 0.4.0 (nx repl, zig bundled with nx) on windows
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

## Commands

| command | effect |
| --- | --- |
| `:help` | the summary |
| `:quit`, `:q`, `exit` | leave |
| `:vars` | the kept bindings and their values |
| `:items` | the declared items |
| `:load FILE` | add the items of a file to the session |
| `:reset` | start over |

## Limits

The interpreter covers the language but not the platform: `@cImport`
calls, `for parallel`, `using arena`, and artifacts need a compiled program
(`nx run`); the message names the expression that could not be evaluated.
Values that only exist at compile time (function values, pointers) are not
kept between lines. Speed is interpreted speed; measure
with `nx run`.

In the session the program may do I/O (`println`, `io.read_file`,
`io.read_line`, `os.env`, `time.now`, `time.sleep`), which `comptime` in a
compiled program never may.
