# Hello, world

Put this in a file called `hello.nx`:

{{include topo/code/hello.nx}}

and run it:

```bash
$ nx run hello.nx
hello, world
hello from Nexium, in 2026
```

That is the whole ceremony. Now, what happened.

## What `nx run` did

`nx run` read the file, checked it, wrote a C file for it, handed the C file
to the C compiler, and ran the executable that came out. On a warm machine
the whole thing takes well under a second; you will not notice the C
compiler unless you look for it. `nx build hello.nx` does everything but the
last step and leaves `nx-out/hello` (`hello.exe` on Windows) behind.

You can look at the C if you are curious:

```bash
nx emit-c hello.nx
```

It is readable. Every Nexium function becomes a C function; the runtime is a
single header pasted at the top. There is no virtual machine and no runtime
library to install: the executable depends on the C library of the platform
and nothing else.

## Reading the program

```nexium
fn main() {
```

A program starts at `main`. This one takes no arguments and returns nothing;
later chapters use `fn main() -> !void` (it can fail with an error) and
`fn main() -> u8` (it chooses its exit code).

```nexium
    println("hello, world", .{})
```

`println` prints a line. The second argument is always there: it is the list
of values to format, and here there are none. `.{}` is an anonymous list of
values; you will see the same spelling wherever a function takes "some
values": struct literals use `.{ .x = 1 }` for the same reason.

```nexium
    let name = "Nexium"
    let year = 2026
```

`let` binds a name to a value, once. `name` is text; `year` is an integer,
and since nothing says otherwise it is an `i64`. Types are inferred inside
functions and written out on function signatures, so a function's contract is
always visible.

```nexium
    println("hello from {}, in {}", .{name, year})
```

Each `{}` in the format string takes the next value. The formatting is
checked when the program is compiled: a `{}` without a value, or a value the
formatter cannot print, is a compile error, not a surprise at run time.

Statements end at the end of the line. There are no semicolons, unless you
want two statements on one line (`a += 1; b += 1`), and a line that ends in
an operator or an open bracket continues on the next.

## Comments

```nexium
// a line comment

/// A doc comment: it attaches to the declaration below it, and
/// `nx doc` turns it into documentation.
fn documented() { }
```

## Two things to know before the next chapter

**Names.** Types are `PascalCase`, functions and variables `snake_case`,
constants `SCREAMING_SNAKE_CASE`. The compiler does not enforce this, but
everything you read will follow it.

**`nx fmt`.** There is one formatting and a tool that applies it:
`nx fmt hello.nx` rewrites the file in place; `nx fmt hello.nx --check` only
reports. The formatter never joins or splits your lines, it settles spacing,
so it is safe to run on anything.

Next: [your first script](03-first-script.html), a program that does
something useful.
