# Nexium language reference

This describes what the `nx` compiler implements today. Section numbers refer
to `nexium-spec.txt` (the design) and, where marked "archived", to
`nexium-systems-spec.txt` sections 4 to 9 (the syntax reference).

## Files and modules

A file is a module. `import foo.bar` loads `foo/bar.nx` next to the root file;
its `pub` items are reached as `bar.item`. `import std.math` (or any std
module) is accepted but not required: the builtin namespaces `math`, `io`,
`os`, `time`, `random`, and `mem` are always in scope.

## Lexical structure (archived 4)

- `//` comments; `///` doc comments attach to the next declaration.
- Identifiers are ASCII. Non-ASCII outside strings and comments is an error.
- Integers: `42`, `0xFF`, `0o755`, `0b1010_1100`. Floats: `3.14`, `1e-9`,
  `0x1.8p3`. Strings `"text"` (must be UTF-8), raw strings `r"..."`, byte
  strings `b"\x00\xff"`, chars `'a'` (a Unicode scalar, type `char`).
- Statements end at a newline. A line that starts with `|>`, `.method(`,
  `catch`, `orelse`, `and`, or `or` continues the previous line, as does a
  line that ends with a binary operator or an open bracket.
- Conventions: types `PascalCase`, functions and variables `snake_case`,
  constants `SCREAMING_SNAKE_CASE`.

## Declarations

```
fn name(a: T, b: U) -> R effects { ... }         // effects: e.g. !allocates !panics
pub fn f(x: i32) -> i32 export(c) { ... }        // exported with the C ABI
extern fn puts(s: *u8) -> i32                    // foreign; calling needs `unsafe`
struct Point derive(Eq, Ord) { x: f64, y: f64 }
struct Frame layout(c) { id: u32, len: u16 }     // C layout, usable across the boundary
struct Pair(T) { a: T, b: T }                    // generic
record Dose { mg: f64 where value > 0.0 }        // validated data
ref class Node { value: i32, next: ?Node }       // reference counted
enum Shape { Circle(f64), Rect { w: f64, h: f64 }, Empty }
type Meters = distinct f64                       // no implicit conversion
type Bytes = []u8                                // alias
error ParseError { Empty, NotANumber }
trait Shape { fn area(self: *Self) -> f64 }
impl Shape for Circle { fn area(self: *Self) -> f64 { ... } }
impl Point { fn origin() -> Point { ... } }
impl(T) Pair(T) { fn swap(self: *mut Self) { ... } }
const TABLE: [256]u8 = comptime build_table()
var counter: u32 = 0                             // mutable global; access needs `unsafe`
test "name" { ... }
artifact cabi { name = "lib", exports = [f] }
```

Fields inside `{ }` are separated by commas or newlines. A field may have a
default (`verbose: u8 = 0`).

## Types (archived 5)

| syntax | meaning |
| --- | --- |
| `i8 … i128`, `u8 … u128`, `isize`, `usize` | integers; no implicit conversion between widths |
| `f32`, `f64`, `bool`, `char`, `void`, `never` | primitives |
| `[N]T` | array, `N` a compile-time constant |
| `[]T`, `[]mut T` | slice: pointer plus length; `[]u8` is text |
| `*T`, `*mut T` | pointer to one value (`&x`, `&mut x`, `p.*`) |
| `?T` | optional; `null` is the empty value |
| `!T`, `Set!T` | error union |
| `List(T)`, `String`, `Map(K, V)` | owning collections (values, section 5.3) |
| `fn(A, B) -> R !effects` | function value (closures and functions coerce to it) |
| `(A, B)` | tuple; fields `.0`, `.1` |
| `weak T` | weak reference to a `ref class` |

Integer literals take the type the context asks for and default to `i64`;
floats default to `f64`. A char literal fits any integer type that can hold it
(`c == 'a'` with `c: u8`).

Casts: `x as T` between numbers (narrowing is checked at runtime unless the
range is proven), between a distinct type and its representation, `char` and
integers, `bool` to integer, a unit enum to integer. `@truncate(T, x)` wraps.

## Values, bindings, ownership

`let x = e` binds immutably, `var x = e` mutably. Mutability is shallow
(archived 5.10): a `let` holding `*mut T` still mutates through it.

Collections own a heap buffer (5.3). `let b = a` moves `a`; using `a`
afterwards is a compile error; `a.clone()` copies. Parameters are borrowed:
a function receiving a `List` reads it; to mutate, take `*mut List(T)`.
`List(T)` and `String` coerce to `[]T` / `[]u8` when passed where a slice is
expected. Moving out of a field or element is an error. Owned values are
released when their scope ends; that is the only automatic action at scope
exit (H7).

`ref class` values are references; copying one retains it and the object is
freed when the last reference goes (5.1). Cycles leak; use `weak` for back
edges (`@weak(x)` or `x.weak()`, then `w.upgrade()`).

## Expressions

- Arithmetic `+ - * / %` trap on overflow (a defined panic). Wrapping `+% -% *%`
  and saturating `+| -| *|` never fail. Bitwise `& | ^ ~ << >>`. Comparison
  `== != < <= > >=` on numbers, chars, bools, `[]u8`, `String`, unit enums,
  and types that `derive(Eq)` / `derive(Ord)`. Logical `and`, `or`, `!`.
- `x |> f(a)` is `f(x, a)`.
- `if (c) a else b` is an expression; `if (opt) |v| { } else { }` unwraps.
- `match v { pat => expr, ... }` on integers (literals, ranges `1..=9`),
  strings, bools, chars, enums (`.Variant(p)`), optionals (`null`, binding),
  error unions (`error.Name`, binding), tuples, and byte slices (binary
  patterns). Enum and bool matches must be exhaustive; others need `_ =>`.
- Blocks are expressions whose value is the final expression. A labeled block
  yields through `break :label value`.
- `try e` propagates an error; `e catch |err| handler`; `opt orelse default`;
  `opt.?` unwraps (panics on null).
- `defer stmt` runs at scope exit, `errdefer stmt` only when the scope exits
  through an error; both in reverse order of registration.
- Closures: `|[captures] params| -> R { body }`. Captures are explicit:
  `[x]` copies, `[&x]` and `[&mut x]` take references.
- `unsafe { }` is required for mutable globals, foreign calls, pointer casts,
  and `.ptr` of a slice.
- `comptime expr` evaluates at compile time; `@embedFile("path")` embeds a
  declared build input as `[]u8`.

## Statements and loops

```
while (cond) { }
for (items) |x| { }               // arrays, slices, lists, strings, map keys
for (items) |x, i| { }            // with index
for (a, b) |x, y| { }             // lockstep; lengths must match
for (0..n) |i| { }
outer: for (...) |a| { for (...) |b| { continue :outer } }
break, continue, return
_ = expr                          // explicit discard; unused values are errors
```

## Binary patterns (archived 6)

```
match packet {
    <<version:4, ihl:4, total_len:16/big, rest:bytes>> => ...
    <<0x1b, '[', 'A', rest:bytes>> => Key.Up
    <<len:16/little, payload:len*8, rest:bytes>> => payload
    _ => ...
}
let written = try <<4:4, 5:4, 0:8, 1500:16/big, "ab">> into buf[..]
```

Segments are `name:size/modifiers` or literals. Sizes are bits (default 8);
`bytes` consumes the rest. Modifiers: `big` (default), `little`, `native`,
`signed`, `unsigned`, `float`, `utf8`. A binding with a constant size up to 64
bits is an integer of the smallest fitting width; larger or dynamic sizes bind
a `[]u8` view. Size arithmetic runs at pointer width and is always checked.
Construction targets a `[]mut u8` buffer and returns `![]u8` (the written
prefix), failing with `error.BufferTooSmall`.

## Effects (section 8)

`allocates refcounts blocks shared_mutable nondeterministic panics ffi`

Effects are inferred for every function. A negative bound on a signature
(`!allocates`) is checked; the diagnostic points at the site that introduced
the effect, through calls. Function types may carry negative bounds
(`fn(i32) -> i32 !allocates`); a closure or function coerced to such a type
must satisfy them. `panics` is discharged by proof (E6): indexing with a loop
index over the same slice, comptime-known indices, arithmetic whose operand
ranges fit, and guarded locals do not contribute. `nx effects file.nx` prints
the inferred set per function.

## Standard library (builtins)

- `println(fmt, .{args})`, `print`, `eprintln`, `format(...) -> String`.
  Placeholders: `{}`, `{x}`, `{X}`, `{b}`, `{o}`, `{e}`, `{c}`, `{:.N}`,
  `{>N}`, `{<N}`; `{{` and `}}` are literal braces.
- `expect(cond)`, `expect_eq(a, b)`, `panic(msg)`.
- `List(T)`: `new`, `with_capacity`, `from`, `append`, `pop`, `clear`, `clone`,
  `last`, `first`, `insert`, `remove`, `swap_remove`, `extend`, `reserve`,
  `items`, `is_empty`, `len`, plus slice methods.
- `String`: `new`, `from`, `with_capacity`, `append`, `append_char`, `clone`,
  `clear`, `pop`, `bytes`, `len`, plus `[]u8` methods.
- `Map(K, V)` (keys: integers, bool, char, `[]u8`, `String`): `new`, `put`,
  `get`, `contains`, `remove`, `clear`, `clone`, `keys`, `values`, `len`,
  `m[key]`; `for (m) |k|` iterates keys.
- Slices: `len`, `fill`, `reverse`, `sort`, `contains`, `index_of`,
  `copy_from`, `to_owned`, `is_empty`; `[]u8` also `starts_with`,
  `ends_with`, `find`, `trim`, `split`, `lines`, `to_string`, `parse_int(T)`,
  `parse_float`, `eq_ignore_case`.
- Integers: `abs`, `min`, `max`, `checked_add/sub/mul` (return `?T`),
  `to_string`. Floats: `abs`, `sqrt`, `floor`, `ceil`, `round`, `min`, `max`,
  `pow`, `to_string`. Chars: `is_digit`, `is_alpha`, `is_space`, `to_lower`,
  `to_upper`, `to_digit`.
- `math`: `PI E TAU INF NAN`, `sqrt abs floor ceil round sin cos tan exp log
  log2 min max pow atan2 clamp`.
- `io.read_file(path) -> !String`, `io.write_file(path, bytes) -> !void`,
  `io.read_line() -> ?String`.
- `os.args() -> [][]u8`, `os.env(name) -> ?[]u8`, `os.exit(code)`,
  `process.run(argv: [][]u8) -> !i32` (spawns, waits, returns the exit code;
  `error.IoError` when the program cannot be started).
- `time.now() -> i64` (ms since the epoch), `time.monotonic() -> u64` (ns),
  `time.sleep(ms)`.
- `random.int(lo, hi)`, `random.float()`, `random.seed(n)`.
- `mem.copy(dst, src)`.
- `@typeName(T)`, `@sizeOf(T)`, `@truncate(T, x)`, `@errorName(e)`,
  `@embedFile(path)`, `@weak(x)`, `@refCount(x)`, `@cImport(header)`,
  `@cstr(literal)`.

Predefined errors: `OutOfMemory Panic InvalidRecord Truncated Overflow
InvalidUtf8 NotFound IoError InvalidInput BufferTooSmall`. Any `error.Name`
creates a new one.

## Entry points

`fn main()`, `fn main() -> !void`, or `fn main() -> u8`. An error from `main`
prints `error: Name` and exits with 1; a panic prints its location and exits
with 101. `test "name" { }` blocks run with `nx test`.

## Trait objects

`dyn Trait` is a fat pointer made from `*T` or `*mut T` where `T` implements
the trait: `let s: dyn Shape = &circle`, `List(dyn Shape)`, `[]dyn Shape`.
Calls dispatch through a vtable and acquire every effect the object type
permits; `dyn Shape !allocates !blocks` is a distinct type that only
implementations satisfying those bounds coerce into. A trait used as an object
may mention `Self` only in receiver position.

## Parallel loops

`for parallel (items) |x, i| { ... }` runs the body over the index range on a
thread pool (spec 7.2). The body may not have the `shared_mutable` effect,
may not `return` or `break` (use `continue`), and writes results through a
mutable slice indexed by `i`. A panic in a worker is re-raised in the caller
after all workers finish. The loop carries the `blocks` effect (it joins).

## Allocation scopes

`using arena { ... }` installs a bump allocator for the block: values created
inside come from the arena, their releases are no-ops, and the whole arena is
freed when the block ends (spec 5.1, "replaceable at any scope"). Containers
created outside the block keep using the heap when they grow inside it, so
collecting results into an outer `List`, `String`, or `Map` is safe. Values
created inside must not escape the block.

## Calling C

```
const libc = @cImport("string.h")
const cv = @cImport("cvendor.h")               // found next to the source file
artifact link { c_sources = ["cvendor.c"], libs = [], include = [], libs_windows = ["gdi32"] }

unsafe {
    let n = libc.strlen(@cstr("hello"))
    let p = cv.cv_point{ .x = 1.0, .y = 2.0 }
    _ = cstdio.printf(@cstr("%d\n"), 42)      // variadics take scalars and pointers
}
```

`@cImport` runs the C preprocessor and imports functions, typedefs, structs,
enums, and literal macros. `const T*` becomes `*T`, other pointers `*mut T`,
`void*` becomes `*mut u8`. Foreign calls need `unsafe` and carry the `ffi`
effect. Declarations that cannot be translated (unions, bit-fields, function
pointers, function-like macros) are named in the error when used.

## Compile-time tests

`comptime test "name" { ... }` runs in the interpreter during checking; a
failure is a compile error pointing at the expectation.

## Regions

A function may not return a slice or pointer into one of its own locals
(rule R1); views into parameters are fine because the caller owns them. A view
stored into an outer variable is not tracked.

## Not implemented yet

`soa` and `packed` layouts, `node` and `installer` artifacts, `nx publish`
and the registry, `pool`/`stack` allocation strategies, and region checking
beyond rule R1.
