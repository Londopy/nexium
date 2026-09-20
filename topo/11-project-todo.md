# Project: a to-do list

A command line tool with subcommands, options, a JSON file it reads and
writes, and exit codes. Small enough to read in one go, shaped like the
tools people actually keep.

{{include topo/code/todo.nx}}

With no arguments it runs a session against a temporary file, so the book
can show it working:

{{output topo/code/todo.expected}}

`no item #9` came out on standard error and is shown last for that reason;
in a terminal it appears in order.

## Arguments with `std.args`

```nexium
    var p = args.Parser.new(os.args())
    let given = p.option("--file", "-f")
    let rest = p.rest()
```

`args.Parser` answers questions about the command line: `flag` for
`--verbose`, `option` for `--file PATH` (or `--file=PATH`, or `-f PATH`),
`int_option` for a number, and `rest()` for whatever was not consumed, which
is where the subcommand and its words are. Ask about the options first, then
take the rest.

## The file

```nexium
fn load(path: []u8) -> !List(Item) {
```

`std.fs` reads the file, `std.json` parses it. `json.parse` returns a `Json`
value, an enum much like the calculator's tokens: `Null`, `Bool`, `Num`,
`Str`, `Arr`, `Obj`. The accessors `json.at`, `json.get`, `json.as_str`,
`json.as_bool` each return an optional, and `orelse continue` skips an entry
that is not shaped as expected instead of crashing on it. A file that is not
JSON at all is an error from `parse`, which `try` turns into the tool's exit
with `error: InvalidInput`.

Writing goes the other way: `json.array()`, `json.object()`, `json.set`,
`json.push` build a value, `json.pretty(&doc, 2)` renders it with two-space
indentation, `fs.write` replaces the file. The whole file is rewritten on
every change, which is right for a list of a few hundred lines and wrong for
a database; this is a to-do list.

## Exit codes

```nexium
fn main() -> !u8 {
```

A `main` that returns `u8` chooses the process's exit code: 0 for success,
2 for a usage mistake, 1 for "no such item". Scripts and other programs can
branch on it. `!u8` means it can also fail with an error, in which case the
code is 1 and the name is printed.

`run` separates the *decision* about the exit code (its `i32` result) from
the *failures* it did not decide (`!` on the return type, from `load` and
`save`). That split, a status for expected outcomes and an error for the
rest, is one to copy.

## Things to try

- `todo edit N TEXT`, and `todo done N` toggling instead of setting.
- Keep the file in the user's home directory: `os.env("HOME")`
  (`USERPROFILE` on Windows), joined with `fs.join`.
- Print dates: `std.time` has `now_local()` and `iso()`.
- Ship it: chapter 20 turns any program with a `main` into an installer.

Next: [binary patterns](12-binary-patterns.html).
