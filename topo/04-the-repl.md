# The REPL

Type `nx` with no arguments at a terminal, or `nx repl` anywhere, and you
get a prompt. What you type is Nexium, checked by the compiler and run by
its interpreter, one line at a time:

```
$ nx
Nexium 1.4.1 (nx repl, zig on PATH) on linux
Type :help for commands, :quit to exit.
> let xs = [3, 1, 2]
> xs
[3]i64: [3, 1, 2]
> let total = xs[0] + xs[1] + xs[2]
> total
i64: 6
> fn twice(x: i64) -> i64 { return x * 2 }
> twice(total)
i64: 12
> import std.strings
> strings.to_upper("route")
String: "ROUTE"
> var s = String.from("nex")
> s.append("ium")
> s
String: "nexium"
```

An expression on its own prints its type and its value. `let` and `var`
bindings stay for the rest of the session. Functions, structs, enums and
imports are accepted like in a file. A line whose brackets are not closed
continues on the next prompt, so a function body can span several lines.

## The prompt is the compiler

There is no second implementation of the language behind the prompt. Each
line becomes part of a program: declarations at the top level, everything
else inside a `main`. The whole program is re-checked on every line, so the
prompt reports exactly what a file would:

```
> let t = twice("no")
error: type mismatch in argument `x`: expected `i64` but found `[]u8`
   | let t = twice("no")
   |               ^
```

The caret is under what you typed, the way a file's diagnostic points at
its line; a line that stops in the middle of a statement (`let x = 1 +`)
says so. `nx` commands belong to the terminal: `nx upgrade` typed at the
prompt is answered as one, not run.

A line that fails to check is dropped; the session continues with what it
had. Only the new statements are executed, by the compile-time interpreter
(the same one `comptime` uses, chapter 13), against the values kept from
earlier lines. A line that panics is reported and dropped in the same way.

Because the prompt runs the interpreter and not compiled code, some things
are out of reach: calls into C through `@cImport`, `for parallel`, `using
arena`, and artifacts. The message names the expression it could not
evaluate. Everything else, including files, the clock and `println`, works.

## Commands

| command | effect |
| --- | --- |
| `:help` | the summary |
| `:quit`, `:q`, `exit` | leave |
| `:vars` | the kept bindings, with their types and values |
| `:items` | the declared functions, types and imports |
| `:load FILE` | add the items of a file to the session |
| `:reset` | start over |

```
> :vars
xs: [3]i64: [3, 1, 2]
total: i64: 6
s: String: "nexium"
> :items
fn twice(x: i64) -> i64 { return x * 2 }
import std.strings
```

`:load` is the way to poke at a program you are writing: load its file,
then call its functions from the prompt with whatever arguments you like.

## When to use it

For trying an expression, checking what a standard library function
returns, or working out a type error in the small. Speed at the prompt is
interpreter speed, fine for that and not for measuring anything: measure
with `nx run`.

Next: [values and types](05-values-and-types.html).
