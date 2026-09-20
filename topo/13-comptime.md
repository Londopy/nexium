# Compile time

Some of a program's work does not depend on its input: a lookup table, the
contents of a data file, a value that follows from a constant. Nexium runs
that work when the program is compiled, in the language itself. There is no
macro language and no template metalanguage; `comptime` runs ordinary
functions in an interpreter before the C compiler ever sees the program.

{{include topo/code/comptime.nx}}

{{output topo/code/comptime.expected}}

## `comptime` and `const`

```nexium
const CRC_TABLE: [256]u32 = comptime make_crc_table()
```

`comptime expr` evaluates the expression during compilation and replaces it
with its value. `make_crc_table` is a normal function; it could be called at
run time too. Here its 256 entries become a static table in the executable,
computed once, by the compiler, and `crc32` at run time does two lookups per
byte. `const` declarations are evaluated at compile time whether or not you
write `comptime`, so the keyword is for the places you want to be explicit,
or for an expression inside a function (`let x = comptime fib(30)`).

The interpreter runs the whole language: loops, structs, enums, `List`,
`Map`, `String`, calls into the standard library. What it refuses is the
outside world: no files, no clock, no randomness, no foreign calls, no
mutable globals, so that compiling a program is a pure function of its
sources and the same everywhere. A step budget turns an infinite loop into
a compile error instead of a hang. (The REPL of chapter 4 runs the same
interpreter with the world switched on.)

## `@embedFile`

```nexium
const ROUTES = @embedFile("data/routes.csv")
```

The file's bytes, at compile time, as a `[]u8`; the path is relative to the
source file. The data is part of the executable, so the program has no file
to find at run time, and `comptime` code can read it: `ROUTE_COUNT` is
counted by the compiler.

## `comptime test`

```nexium
comptime test "the table starts the way every CRC-32 table does" {
    expect_eq(CRC_TABLE[1], 0x77073096)
}
```

A test that runs while the program is being checked. A failure is a compile
error pointing at the expectation. It costs nothing at run time and cannot
be forgotten, which makes it the right place for the facts a table or a
constant must satisfy.

## The builtins

`@typeName(T)`, `@sizeOf(T)`, `@truncate(T, x)`, `@errorName(e)`,
`@embedFile(path)`, `@weak(x)`, `@refCount(x)`, `@cImport(header)`,
`@cstr(literal)`: the `@` names are the operations that need the compiler's
knowledge rather than a library. There are nine, and they are all listed in
the [reference](../docs/language.html#standard-library-builtins).

## What this is for

Tables (CRC, sine, colour palettes), embedded assets (a font, a shader, a
default configuration), configuration checked at compile time (`comptime
test` that the embedded data is well-formed), and any computation whose
inputs are all known before the program runs. `comptime` is also how
generics are spelled: `fn largest(comptime T: type, ...)` is a function
whose first argument is known at compile time, and every generic call is a
compile-time evaluation of the function's signature with that type.

Next: [tests](14-testing.html).
