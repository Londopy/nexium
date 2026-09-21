# Collections

Three owning containers, one view type, and a standard library that works
on the view. That is the whole collection story, and it fits in one
program.

{{include topo/code/collections.nx}}

{{output topo/code/collections.expected}}

## `List(T)`

A growable sequence that owns its elements. `List(i32).new()` is empty;
`with_capacity(n)` reserves; `List(u8).from("abc")` copies from a slice.
`append`, `insert(i, v)`, `pop() -> ?T`, `remove(i)`, `swap_remove(i)`,
`extend(slice)`, `clear`, `last() -> ?T`, `first()`, `len`, `is_empty`.
Index with `xs[i]`, which panics past the end (chapter 15 says when the
compiler can prove it cannot). A `List` is a slice wherever a slice is
wanted, so every slice method below is a `List` method too, and `xs[..]`
names the whole thing explicitly when you need to.

Appending an owning value moves it in (`kept.append(w.clone())` in the
ownership chapter); reading one out is `&xs[i]`, a borrow, because moving an
element out would leave a hole. `xs.pop()` is the way to take the last one.

## Slices

`[]T` is a pointer and a length: `len`, `contains`, `index_of`, `sort` (for
ordered element types), `reverse`, `fill`, `copy_from`, `to_owned` (a fresh
`List`), `is_empty`. `[]mut T` is the same view with permission to write
through it; a `var` array or list gives one, `let` gives the read-only kind.
Ranges cut sub-slices, `xs[1..3]`, `xs[2..]`, `xs[..2]`.

`std.lists` adds what a functional style wants, generic over the element
type: `filter`, `map`, `fold`, `any`, `all`, `find`, `position`, `min`,
`max`, `sum`, `zip`, `dedup`, `take`, `drop`. They take a function value as
the last argument; a plain function's name (`is_even`) is one, and so is a
closure `|[] x: i32| -> bool { x > 2 }` with its captures listed in the
brackets.

## `String` and `[]u8`

`String` owns text: `String.new()`, `String.from(slice)`, `append(slice)`,
`append_char(c)` (a code point, UTF-8 encoded), `push_byte(b)`, `clear`,
`pop`, `len`. A string literal is a `[]u8`, and every `[]u8` operation is
available on a `String` through `s[..]`: `split(sep)`, `lines()`, `trim()`,
`find(needle) -> ?usize`, `starts_with`, `ends_with`, `eq_ignore_case`,
`parse_int(T)`, `parse_float()`, `to_string()`.

The pieces `split` and `lines` return are views into the original bytes:
free to make, valid while the original lives. `std.strings` has `join`,
`replace`, `repeat`, `pad_left`, `to_upper`, `split_whitespace`,
`split_once` and the rest; `std.text` is for the cases where a character is
not a byte (`char_count`, `chars`, `width`, `to_upper` for non-ASCII
letters).

`format("{}", .{...})` builds a `String`; it is `println` without the
printing.

## `Map(K, V)`

A hash map. Keys may be integers, `bool`, `char`, text (`[]u8` or
`String`) or any struct that derives `Hash` and `Eq`. `put(k, v)` inserts
or replaces (and takes ownership of both), `get(k) -> ?V`, `m[k]` is the
same lookup, `contains`, `remove(k) -> ?V`, `keys()` and `values()` collect
into `List`s, `len`, `clear`. `for k in m` walks the keys and
`for (k, v) in m` the entries; the order is the map's own, not insertion
order, so sort the keys when the order shows.

A `Map(String, V)` is looked up with a `[]u8`: the key you store is owned,
the key you search with is a view, which is what `stock["rope"]` above
relies on.

## Arrays

`[N]T` has a length in its type and is a value: assigning it copies every
element, which is what you want for a small fixed table and not for
anything large. `var grid = [[0, 0, 0], [0, 0, 0]]` is `[2][3]i64`, indexed
`grid[r][c]`. An array coerces to a slice, so anything that takes `[]T`
takes `arr[..]`.

## Choosing

- A sequence that grows: `List`.
- A function parameter: a slice, always, unless the function keeps the
  data (then `own List`).
- Text you build: `String`; text you look at: `[]u8`.
- Lookup by key: `Map`.
- A fixed handful of things known at compile time: an array.

Next: [a project: the calculator](10-project-calculator.html).
