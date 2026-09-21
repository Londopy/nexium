# Functions and errors

Nexium has no exceptions. A function that can fail says so in its return
type, the caller sees it there, and the compiler makes sure every failure is
either handled or passed on. Optionals do the same for "there might not be
one". This chapter is those two types and the five words that go with them:
`try`, `catch`, `orelse`, `defer`, `errdefer`.

{{include topo/code/errors.nx}}

{{output topo/code/errors.expected}}

## Functions

```nexium
fn parse_small(text: []u8) -> Parse!u8 {
```

A function names its parameters with their types and its return type after
`->`. There are no default arguments and no overloading: one name, one
signature. A function with no `->` returns nothing. Parameters are borrowed:
the caller keeps what it passed (chapter 8 has the whole rule).

Functions live at the top level of a file or inside an `impl` block
(chapter 7); there are no nested function declarations, closures fill that
role (`|[captures] x: i32| -> i32 { x + 1 }`). The order of declarations in
a file does not matter.

## Error sets and error unions

```nexium
error Parse { Empty, NotANumber, TooBig }
```

An `error` declaration names a set of errors. `Parse!u8` is an *error
union*: a `u8`, or one of `Parse`'s errors. `!u8` without a set name accepts
any error at all, which is the right type for `main` and for code that only
forwards what it gets. A named set is a contract: `parse_small` can fail
these three ways and no other, and `return error.Empty` is checked against
the set.

Errors are values with a name and no payload. `@errorName(e)` gives the name
as text; the predefined ones (`NotFound`, `IoError`, `InvalidInput`,
`Overflow`, ...) are what the builtins and the standard library return.

## `try`, `catch`, `match`

```nexium
    let x = try parse_small(a)
```

`try e` is "unwrap the success value, and if it is an error, return that
error from this function right now". It only compiles inside a function whose
return type can carry the error, which is how the compiler makes forgetting
impossible: an unhandled `!T` in statement position is an error too
(`unhandled error: this expression has type ...`).

```nexium
        let v = parse_small(input) catch |e| {
            println("{}: {}", .{input, @errorName(e)})
            continue
        }
```

`catch` handles the failure where it happens. The handler receives the error
as `|e|` and must produce a value of the success type, or leave: `continue`,
`break` and `return` are all allowed there, and `catch 0` is the short form
when a default is all you need. `match` on an error union has an arm per
named error and a binding arm for the success value; the compiler checks
that no member of the set is left out, and `else => ...` takes whichever
errors the other arms did not name.

## Optionals

```nexium
fn find_byte(haystack: []u8, needle: u8) -> ?usize {
```

`?usize` is a `usize` or `null`. `orelse` supplies a default, or leaves
(`orelse return null`, `orelse continue`); `if let i = opt { }` runs its
block only when there is a value; `opt.?` insists there is one and panics
otherwise, for the places where you have already checked. A number literal
where a `?T` is expected is wrapped for you, so `find_byte(...) orelse 99`
reads as it should.

And when what you want is a field or a method of the value inside, `?.`
reaches through: `user?.name` is `null` when `user` is and the name as an
optional otherwise, so `user?.name.len orelse 0` is one line where an `if
let` would be four.

## `defer` and `errdefer`

```nexium
    println("  open", .{})
    defer println("  close", .{})
    errdefer println("  (rolled back after an error)", .{})
```

`defer stmt` runs the statement when the enclosing scope ends, however it
ends: falling off the end, `return`, an error propagated by `try`. Several
`defer`s run in reverse order. It is how a file gets closed and a lock gets
released next to the line that opened or took it. `errdefer` runs only when
the scope is left through an error, for undoing partial work.

The program shows both: the second call fails inside `try`, the `errdefer`
fires, then the `defer` fires, then the error reaches `main`'s `catch`.

## Panics

A panic is not an error value. It is the program stopping with a message and
a location: an index out of bounds, an integer overflow, an `opt.?` on
`null`, a `panic("...")` you wrote, an `expect` in a test that failed. In a
plain program it exits with code 101; inside a library shipped with `nx ship`
it becomes an error code the host receives (chapter 20); the compiler tracks
which functions can panic at all (chapter 15).

The two mechanisms divide the world cleanly: errors are for what a correct
program expects to happen (a missing file, a malformed input), panics for
what it does not.

Next: [structs, enums and traits](07-structs-enums-traits.html).
