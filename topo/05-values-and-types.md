# Values and types

This chapter is the ground the rest stands on: numbers, text, arrays,
conditions and loops. One program shows all of it; the sections after it
walk through the parts.

{{include topo/code/values.nx}}

{{output topo/code/values.expected}}

## Bindings

`let` binds a name once; `var` binds a name you can assign to again. Neither
needs a type when the initializer gives one away, and both take one when it
matters: `let small: u8 = 200`. Prefer `let`; a reader learns something
from every `var` that remains.

## Integers

Nexium has the fixed-width integers a systems language needs: `i8` to
`i128`, `u8` to `u128`, and `isize`/`usize` for sizes and indices. There is
no implicit conversion between widths, not even widening: `a + b` needs `a`
and `b` to have the same type, and `x as u32` says out loud where a width
changes. A literal takes the type its context asks for and is `i64` when
nothing asks.

The arithmetic operators **trap on overflow**: `200 + 100` in a `u8` is a
panic with a message and a location, never a silent wrap to 44. When wrapping
is what you mean, say so: `+%`, `-%`, `*%` wrap; `+|`, `-|`, `*|` saturate at
the type's limits. A hash function wants `*%`; a volume control wants `+|`.
Chapter 15 shows how the compiler notices when the trapping form cannot
actually trap, so that most arithmetic costs nothing.

Literals: `42`, `0xFF`, `0o755`, `0b1010_1100`, `3_000_000_000` (the
underscores are for you).

## Floats, booleans, characters, text

`f64` is the default float, `f32` the other one. `bool` is `true` or
`false`; `and`, `or` and `!` combine them and are spelled as words. `char` is
one Unicode scalar value, `'n'` or `'\n'` or `'é'`, and compares with the
integer types that can hold it.

Text is bytes. A string literal `"Nexium"` has type `[]u8`, a *slice* of
bytes that must be valid UTF-8; `text.len` is the byte length, 6 here. The
owning counterpart, `String`, comes in chapter 9, and `std.text` handles the
cases where a byte is not a character.

`{:.3}` in the format string rounds a float to three decimals; the other
placeholders are in the [reference](../docs/language.html#standard-library-builtins).

## Casts

`x as T` converts between number types, between an integer and a `char`,
between a `bool` and an integer, and between a unit enum and an integer.
Narrowing is checked: `300 as u8` panics unless the compiler can prove the
value fits, which it often can (a loop index over a slice, a value that was
just compared). `@truncate(u8, x)` is the cast that wraps instead. `7.9 as
i32` truncates toward zero.

## Arrays, slices, tuples

`[2, 3, 5, 7, 11]` is an array: five `i64`s, a fixed size that is part of
its type (`[5]i64`), a value that is copied when assigned. `primes[1..4]` is
a slice of it: a pointer and a length, a view that owns nothing. Functions
almost always take slices, so that arrays, `List`s and pieces of either all
fit; `primes[..]` is the slice of the whole array.

A tuple groups a few values of different types: `(3, "three")` has type
`(i64, []u8)` and fields `.0` and `.1`.

## `if`, blocks, `match`

`if` is an expression when it has an `else`, so it can sit on the right of
a `let`. Conditions take no parentheses; bodies always take braces, even for
one statement, which keeps `if c { return v }` unambiguous on one line.

A block is an expression whose value is its last line without a `return`,
which is how `squared` above gets its value. A labeled block, `search: {
... }`, is left early with `break :search value`; that is the idiom for "the
first thing that matches, or a default".

`match` compares a value against patterns in order and takes the first arm
that fits. Integers match literals and ranges (`2..=9` is inclusive); text
matches text; the `_` arm takes whatever is left. Matches on enums and
booleans must be exhaustive, and the compiler says which case is missing.
Chapter 7 uses that with enums, chapter 12 with binary data.

## Loops

`while cond { }` and `for` in four forms:

```nexium
for x in items { }               // arrays, slices, lists, strings, map keys
for x, i in items { }            // with the index
for i in 0..10 { }               // a range: 0 to 9
for i in 0..10 step 3 { }        // 0, 3, 6, 9; `for i in 10..0 step -1` counts down
```

`break` and `continue` do what they do everywhere; a labeled loop
(`outer: for ...`) is left with `break :outer`. `while c { } else { }` runs
the `else` when the condition turns false, not after a `break`, which is the
"search finished without finding" case.

The loop variable of a `for` over a collection is a *view* of the element,
not a copy you own; that matters for owning element types and is one of the
rules of chapter 8.

Next: [functions and errors](06-functions-and-errors.html).
