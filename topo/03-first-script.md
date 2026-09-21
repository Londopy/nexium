# Your first script

A script is a program that reads something, works something out and prints
it. This one reads a text file and prints its ten most frequent words. It is
sixty lines, and it uses most of what a day-to-day program needs: arguments,
files, text, a map, a struct, a sort.

{{include topo/code/wordfreq.nx}}

Run it on the sample text that ships with this book, or on any file:

```bash
$ nx run topo/code/wordfreq.nx
$ nx run topo/code/wordfreq.nx -- notes.txt
```

{{output topo/code/wordfreq.expected}}

Arguments for the program go after `--`, so that `nx` does not read them as
its own.

## Piece by piece

```nexium
import std.fs
```

The standard library is a set of modules written in Nexium and embedded in
the compiler; `import std.fs` makes `fs.read`, `fs.write`, `fs.list` and the
rest available under `fs.`. There is nothing to install. The builtins you
used in the last chapter (`println`, `format`, the `io`, `os`, `math`
namespaces) need no import at all.

```nexium
fn main() -> !void {
```

`!void` says `main` may fail with an error. Inside such a function `try`
hands any error to the caller, and if `main` itself fails, the program prints
`error: <name>` and exits with code 1. Chapter 6 is about errors; for now,
`try fs.read(path)` reads "read the file, and if that fails, stop here".

```nexium
    let args = os.args()
    let path = if args.len > 1 { args[1] } else { "topo/code/data/hike.txt" }
```

`os.args()` is the command line; `args[0]` is the program. `if` is an
expression, so it can pick the value of a `let`.

```nexium
    var counts = Map(String, u32).new()
```

`var` declares a variable that can be reassigned or mutated; `let` does not.
A `Map` is a hash map; this one goes from `String` to `u32`. `List(T)`,
`String` and `Map(K, V)` are the three owning containers, and the type
parameters go in parentheses, not angle brackets.

```nexium
    for line in text[..].lines() {
        for raw in line.split(" ") {
```

`text` is a `String`, an owning buffer; `text[..]` is a *slice* of it, a view
of the bytes that does not own them. Most text operations are defined on the
view type `[]u8`, and a `String` becomes one with `[..]`. `lines()` and
`split()` return lists of views into the same bytes: no copying happens until
`clean` builds a fresh `String` for the lowered word.

```nexium
            let n = counts.get(word[..]) orelse 0
            counts.put(word, n + 1)
```

`get` returns an *optional*, `?u32`: the count, or `null` when the word is
new. `orelse` gives the default. Then the word goes into the map, and this
is the first sight of the rule the whole language turns on: `put` takes the
key by value, so `word` is *moved* into the map. The name `word` is unusable
after that line, and the compiler would refuse a later use of it. That is
fine here, because the loop is done with it. When you need to keep a value
you hand over, `.clone()` it. Chapter 8 makes this precise.

```nexium
struct Entry { count: u32, word: String }
```

A struct is a value with named fields. `Entry{ .count = 1, .word = w }`
makes one. The leading dots on field names are the same convention as
`.{...}`: "a field of the thing being built".

```nexium
fn sort_entries(xs: *mut List(Entry)) {
```

The parameter is a *mutable pointer* to a list. Functions borrow their
parameters: a plain `List(Entry)` parameter lets the function read the list
and nothing more, and the caller keeps it; `*mut` lets `sort_entries` change
it in place, and the caller still keeps it. (A function that wants to keep
the value says `own`; chapter 8.)

Slices of ordered types sort with one call, `xs[..].sort()`, but a struct is
not ordered until you say how, so this program sorts by hand with an
insertion sort. Small programs do that; chapter 9 shows the standard
library's helpers.

```nexium
        println("{>3}  {}", .{e.count, e.word})
```

`{>3}` right-aligns the value in three columns. The format language has a
handful of these: `{:.2}` for two decimals, `{x}` for hexadecimal, `{<8}` for
left alignment. Arguments can be named, `.{ .word = w, .cols = 3 }`, and
then the placeholders name them, `{word:>cols}`, with the width coming
from an argument. They are all in the [reference](../docs/language.html).

## What you have

A program that reads its arguments, reads a file, walks its text, counts with
a map, sorts a list of structs and prints a table, with no allocation you did
not ask for and no library outside the compiler. The next chapter puts the
same language at a prompt.

Next: [the REPL](04-the-repl.html).
