# The Nexium Language Specification

Version 1.0, describing Nexium as implemented by `nx` 1.0.0. This document is
normative for what the compiler does today; `nexium-spec.txt` is the
original design, `DECISIONS.md` records every call made where that design
was open, and `ROADMAP.md` says what comes next. Anything marked **planned**
is not implemented and is described so the design stays visible; anything
marked **decided** points to the decision that settled it.

Conformance is measured by the repository's tests: every clause below is
exercised by an example under `examples/`, a compile-fail case under
`tests/compile_fail/`, or a std module's own tests.

---

## 1. Goals and constraints

Nexium is a compiled language with three commitments that shape everything
below:

1. **Predictable memory without a collector.** Values own their storage,
   moves are explicit in the type rules, reference-counted classes cover
   shared ownership, and every release happens at a scope exit the reader
   can see.
2. **Effects are part of a function's type.** Whether a function allocates,
   blocks, panics, or touches shared state is inferred, checked against
   declared bounds, and visible in tools and in the C ABI of exports.
3. **One source tree ships anywhere.** A program becomes a native
   executable, a C library, a Python wheel, or a Rust crate from one file,
   and a panic never crosses the boundary into a host.

The hard constraints inherited from the design (spec section 3) and kept:
no runtime initialization for a shipped library (S1), no mutable globals in
an embeddable artifact (S2), panics converted at the export boundary (S3),
one translation unit per build (S6), and the release at scope exit as the
only automatic action at scope exit (H7).

## 2. Lexical structure

- Source is UTF-8. Identifiers are ASCII letters, digits, and `_`; a
  non-ASCII byte outside a string or comment is an error.
- `//` comments run to end of line. `///` doc comments attach to the next
  declaration and appear in `nx doc`.
- **Newlines terminate statements.** Inside `(` `)` and `[` `]` newlines are
  insignificant. A line continues the previous one when it starts with `|>`,
  `.method(`, `catch`, `orelse`, `and`, `or`, or when the previous line ends
  with a binary operator or an open bracket.
- Integer literals: decimal, `0x`, `0o`, `0b`, with `_` separators.
  Float literals: decimal with `.` or exponent, hexadecimal with `p`
  exponent. String literals `"..."` must be UTF-8; `r"..."` is raw;
  `b"..."` is bytes, any escapes; `'x'` is a `char` (a Unicode scalar).
  Escapes: `\n \r \t \0 \\ \" \' \xHH \u{H...}`.
- Keywords: `fn let var const struct enum record ref class trait impl pub
  import return if else for while match break continue try catch defer
  errdefer comptime unsafe error true false null undefined and or type
  distinct where into artifact test export unreachable as orelse in dyn
  weak parallel extern using`. `own` is a contextual modifier in parameter
  position only.
- `<<` and `>>` are always the binary-pattern delimiters at the lexical
  level; the parser decides between a pattern and a shift by context.

The token stream is fixed by `nx tokens` and reproduced exactly by the
self-hosted lexer (`self/lexer.nx`).

## 3. Program structure

A file is a module. Items are declarations at the top level:

```
fn name(params) -> R effects { ... }         // functions; effects are negative bounds
pub fn f(x: i32) -> i32 export(c) { ... }    // exported with the C ABI
extern fn puts(s: *u8) -> i32               // foreign declaration
struct Point derive(Eq, Ord) { x: f64, y: f64 }
struct Frame layout(c) { id: u32, len: u16 } // C layout; usable across the boundary
struct Pair(T) { a: T, b: T }               // generic
record Dose { mg: f64 where value > 0.0 }   // validated data
ref class Node { value: i32, next: ?Node }  // reference counted
enum Shape { Circle(f64), Rect { w: f64, h: f64 }, Empty }
type Meters = distinct f64                  // no implicit conversion
type Bytes = []u8                           // alias
error ParseError { Empty, NotANumber }      // an error set
trait Shape { fn area(self: *Self) -> f64 }
impl Shape for Circle { ... }               // trait implementation
impl Point { ... }                          // inherent methods
impl(T) Pair(T) { ... }                     // generic impl
const TABLE: [256]u8 = comptime build()     // compile-time constant
var counter: u32 = 0                        // mutable global (needs `unsafe` to touch)
test "name" { ... }                         // run by `nx test`
comptime test "name" { ... }                // run by the checker
artifact cabi { name = "lib", exports = [f] }
import foo.bar                              // foo/bar.nx next to the root file
import std.json                             // a std module embedded in the compiler
```

Visibility: `pub` items of an imported module are reached as
`module.item`. Fields are visible to any code that can see the struct,
and so are methods: `pub` on a method marks the type's documented
interface. A type's inherent methods may be declared in several `impl`
blocks, in any module that can see the type; a type has one method of a
name, so two blocks may not both define it.
Struct fields are separated by commas or newlines and may carry defaults
(`verbose: u8 = 0`). A struct literal's initializers are evaluated in the
order written, then the defaults of the fields it leaves out.

**Entry points.** `fn main()`, `fn main() -> !void`, or `fn main() -> u8`.
An error returned from `main` prints `error: Name` and exits with 1; a
panic prints its location and exits with 101; a `u8` is the exit code.

## 4. Types

| type | meaning |
| --- | --- |
| `i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 isize usize` | integers; no implicit conversion between widths |
| `f32 f64` | IEEE floats |
| `bool char void never` | `char` is a Unicode scalar; `never` is the type of a diverging expression |
| `[N]T` | array, `N` a compile-time constant; a value |
| `[]T`, `[]mut T` | slice: pointer and length; `[]u8` is text |
| `*T`, `*mut T` | pointer to one value; `&x`, `&mut x`, `p.*` |
| `?T` | optional; `null` is the absent value |
| `!T`, `Set!T` | error union: an error of the global set or of `Set`, or a `T` |
| `List(T)` `String` `Map(K, V)` | owning collections (section 5); `Map` keys are integers, `bool`, `char`, `[]u8`, or `String` |
| `(A, B, ...)` | tuple; fields `.0`, `.1`; legal as a type argument |
| `fn(A, B) -> R !effects` | function value; may carry negative effect bounds |
| `dyn Trait !effects` | trait object; a fat pointer (section 8.4) |
| `weak T` | weak reference to a `ref class` |
| `distinct T` | a new type with `T`'s representation and no implicit conversion |

**Type identity.** Types are structural for the built-in constructors and
nominal for structs, enums, records, classes, error sets, and distinct
types. Generic instantiations with equal arguments are the same type.

**Literals.** Integer literals take the type the context demands and default
to `i64`; float literals default to `f64`. A char literal is accepted where
any integer type that can hold it is expected. An integer or float literal
coerces into `?T`.

**Casts.** `x as T` converts between numeric types (narrowing is checked at
run time unless the range analysis proves it fits), between a distinct type
and its representation, between `char` and integers, from `bool` to
integer, and from a unit enum to an integer. `@truncate(T, x)` wraps.

**Coercions** (implicit, only these): `T` to `?T`; a value or error to
`E!T`; `List(T)` and `String` to `[]T` and `[]u8`; `*String` and
`*List(T)` to `[]u8` and `[]T`; `[N]T` and `*[N]T` to `[]T`; `*T`/`*mut T`
to `dyn Trait` when `T` implements the trait; a function or closure to a
matching `fn` type; `never` to anything.

**Recursive types** are legal through `List`, `Map`, pointers, and
`ref class`: `enum Json { Arr(List(Json)) }`. A struct may not contain
itself by value.

## 5. Values, ownership, and memory

### 5.1 Bindings

`let` binds immutably, `var` mutably. Mutability is shallow: a `let`
holding `*mut T` mutates through the pointer. An unused value is an error;
`_ = expr` discards explicitly.

`let (a, b) = e` binds one name per element of a tuple, `_` skipping one
(a skipped element is still dropped). Over an owned value (a call's
result, a literal, a moved local) the names own the elements; over a
place they are views of its elements, as `if let` binds over a place, so
the place keeps its value and moving out of a name is an error. `var`
destructures owned values only. `for (k, v) in pairs` binds the same way
over each item, and may still take an index: `for (k, v), i in pairs`.

### 5.2 Owning values and moves

`List`, `String`, `Map`, and any struct, enum, or tuple containing one are
*owning*. Passing or assigning an owning value moves it: the source is
unusable afterwards and using it is a compile error (`use after move`).
`.clone()` makes an independent copy. Moving out of a field or an element
is an error. Moves are tracked per branch: a value moved in one `if` branch
or `match` arm is still available in the others, and a branch that diverges
(`return`, `break`, a panic) moves nothing for the code after it.

Owned values are released when their scope ends, in reverse order of
declaration, together with `defer` statements. That release is the only
automatic action at scope exit.

### 5.3 Parameters

Parameters are borrowed: the callee may read them and may not move them. A
function that mutates a collection takes `*mut List(T)`. A parameter marked
`own` takes ownership (**decided**, 61): the argument is moved at the call
site, the callee may mutate it, move it on, or let it drop at return. `own`
is rejected on receivers, on non-owning types, on exported functions, and a
function with an `own` parameter cannot be used as a function value. An
`own` parameter is the callee's storage: a view into it is a view into a
local (5.6).

### 5.4 Reference classes

A `ref class` value is a pointer to a heap object with a reference count.
Copying it retains; the object is released when the count reaches zero and
its fields are released with it. Cycles are not collected; `weak` breaks
them: `@weak(x)` or `x.weak()` yields a `weak T`, and `w.upgrade()` returns
`?T`, null once the object is gone. `@refCount(x)` reads the count.

### 5.5 Pointers and safety

`*T` and `*mut T` are safe references created with `&` and `&mut` and
dereferenced with `.*`. Field access, method calls, indexing, and iteration
dereference automatically. `unsafe { }` is required for: reading or writing
a mutable global, calling a foreign function, casting between pointers or
between integers and pointers, and taking `.ptr` of a slice.

### 5.6 Views and their storage

A view (a slice, a pointer, a closure, or a value holding one) points into
storage it does not own. The checker knows where every view points (its
*origins*: locals, a parameter's storage, a literal, a temporary, the
value's own heap storage) and enforces five rules over them (**decided**,
100 to 104). In 1.2 they are warnings; `--strict` (or `NX_STRICT=1`) makes
them errors, and 1.3 makes them errors for everyone (`docs/stability.md`).
The archived region rules R2 to R4 are not adopted (**decided**, 88); the
view rules replace them.

- **V1.** A function may not return a view into its own locals, its
  temporaries or its `own` parameters, nor a value holding one. Views into
  borrowed parameters are allowed: the caller owns that storage. The direct
  case, a returned slice or pointer into a local, has been an error since
  1.0 as rule R1.
- **V2.** A view stored into a place (a variable, a field, an element, a
  global, a closure capture, the caller's storage through a pointer
  parameter) must not outlive the storage it points into: `out = s[..]`
  with `s` a local of an inner block, or a temporary of that statement.
- **V3.** A view into a `List`, `String` or `Map` is stale once the
  container grows, is cleared or is reassigned (`append`, `insert`, `put`,
  `extend`, `reserve`, `clear`, `=`). A use of the view after the change is
  reported, and so is a change inside a loop of a container a view taken
  before the loop reads in it.
- **V4.** A view into a value is stale once the value moves away; a use
  after the move is reported. A move on the line of the use is not: a
  literal or a `return` holding the value and the view together keeps the
  storage alive.
- **V5.** A value made inside `using arena { }` may not be kept past the
  block (5.7).

A view into heap storage a value owns (a slice of its `String` field) that
the same literal moves in is a view into the literal's value: it travels
with the value, and V3 and V4 then apply to that value. The rules report
at the use that would read released storage, naming the storage and the
moment it was released; a view that is not used again is not reported.
`.clone()` makes an independent copy where a view was kept. A `*mut`
obtained inside `unsafe` stays the programmer's responsibility. A debug
build fills freed storage with a fixed byte (`0xDD`), so a use the rules
miss reads garbage or panics on a length instead of yielding the old
contents by luck; such a use is a bug to report (`SECURITY.md`).

### 5.7 Allocation scopes

`using arena { ... }` installs a bump allocator for the block. Values
created inside are allocated from the arena, their releases are no-ops, and
the arena is freed as a whole at the end of the block, and on a `return`,
`break` or `continue` that leaves it. Containers created outside the block
keep using the heap when they grow inside it, so collecting numbers and
other values that own no heap storage into an outer `List`, `String`, or
`Map` is safe. A value made inside the block must not be kept past it
(rule V5 of 5.6): not returned, not assigned to a place declared outside,
not appended to an outer container, not handed to a global or to the
caller. `@escape(v)` is the way out: a copy of `v` (a variable, a field or
an element) made by the allocator the block was entered with, which lives
on after the block; `.clone()` inside the block allocates from the arena
and does not. A value a call returns inside the block counts as made
there. A debug build fills the arena's storage with `0xDD` when the block
ends. `arena` is the only allocation strategy; `pool` and `stack` are not
adopted (**decided**, 88).

## 6. Expressions and statements

### 6.1 Operators

Arithmetic `+ - * / %` traps on overflow (a defined panic). Wrapping
`+% -% *%` and saturating `+| -| *|` never fail. Bitwise `& | ^ ~ << >>`.
Comparison `== != < <= > >=` on numbers, chars, bools, `[]u8`, `String`,
unit enums, and types that `derive(Eq)` / `derive(Ord)`. Logical `and`,
`or`, `!`, short-circuiting. `x |> f(a)` is `f(x, a)`.

Compound assignment: `= += -= *= /= %= &= |= ^= <<= >>= +%= -%= *%=`.

### 6.2 Control flow

- `if c { a } else { b }` is an expression when both branches have a type.
  An `if` without `else` is a statement. The condition is a bare expression
  (no parentheses) and every body is a block, so a one-liner is
  `if c { return v }` or `let m = if a > b { a } else { b }`. A struct
  literal in a condition needs parentheses, `if (Point{ .x = 1 }) == p { }`,
  because the `{` would otherwise open the body.
- `if let v = opt { } else { }` unwraps an optional. When `opt` is a
  place (a local, a field, an element) the binding is a view of the
  payload, like a loop variable: it cannot be moved out of (`.clone()`
  it, or take the value with `opt.?`). When `opt` is an owned temporary,
  such as a call result, the binding owns the payload.
- `while c { }`, `for x in items { }`, `for x, i in items { }`,
  `for (k, v) in pairs { }`,
  `for x, y in a, b { }` (lengths must match), `for i in lo..hi { }`,
  `for i in lo..hi step s { }` (a negative step counts down and needs a
  signed loop variable; a zero step is an error). `while c { } else { }`
  runs the else block when the condition turns false, not after a `break`.
  Iteration works over arrays, slices, lists, strings, map keys, and map
  entries with a tuple binding (`for (k, v) in m { }`); and over any
  value with a `next(self: *mut Self) -> ?T` method, which the loop
  owns and drives until it yields `null` (`for x in (Counter{ .n = 3 }) { }`;
  one iterator, one binding, no index, no `parallel`). Loop bodies take
  braces.
- `break`, `continue`, `return`, each optionally with a label:
  `outer: for a in ... { ... continue :outer }`. A labeled block yields a
  value with `break :label value`. `else` may start the next line.
- `match v { pattern => expr, ... }` (section 7).
- Blocks are expressions whose value is their trailing expression.

### 6.3 Errors and optionals in expressions

`try e` propagates an error from an `E!T` to the enclosing function, which
must return an error union. `e catch |err| handler` handles it; the handler
is an expression of `T`'s type, or a jump (`return`, `break`, `continue`).
`opt orelse default` and `opt orelse return x` unwrap or jump. `opt.?`
unwraps and panics on null. `defer stmt` runs at scope exit; `errdefer stmt`
only when the scope exits through an error.

`opt?.field`, `opt?.method(args)`, and any postfix chain after `?.`, read
through an optional: `null` when `opt` is, otherwise the chain applied to
the payload, wrapped as an optional; a result that is an optional already
is not wrapped twice, so `a?.b?.c` chains, and `a?.name.len orelse 0`
reads as `(a?.name.len) orelse 0`. A chain over a place reads the payload
in place; over an owned temporary the payload belongs to the chain and is
dropped with it.

### 6.4 Closures

`|[captures] params| -> R { body }`. Captures are explicit: `[x]` copies,
`[&x]` and `[&mut x]` take references. A closure coerces to `fn(...) -> R`
and to `fn(...) -> R !bounds` when it satisfies the bounds.

### 6.5 Formatting

`println(fmt, .{args})`, `print`, `eprintln`, and `format(...) -> String`
take a literal format string with placeholders `{}` `{x}` `{X}` `{b}` `{o}`
`{e}` `{c}` `{:.N}` `{>N}` `{<N}`; `{{` and `}}` are literal braces.
With named arguments, `.{ .name = value, ... }`, every placeholder names
one, `{name}` or `{name:spec}`, an argument may be written more than
once, every argument must be used, and a width may come from an
integer argument: `{v:>w}` with `.w = 8`. A width pads any value written
as text: numbers, strings, booleans, and enum and error names; a char is
written as it is. Formatting is compiled: each placeholder becomes a typed
write.

## 7. Patterns

`match` must be exhaustive. Exhaustiveness is decided by the usual matrix
algorithm over constructors, so tuples of enums, nested optionals, enum
payloads and error sets are checked precisely; integers, strings and
other unbounded types need a `_ =>` arm. Patterns: literals (integers, chars, strings, bools, negative
literals), integer ranges `1..=9`, enum variants `.Variant(p, q)` (a struct
variant's fields bind positionally, in declaration order), `null` and a
binding on optionals (the binding is
the payload), `error.Name` and a binding on error unions (the binding is
the success value), tuples, slices by shape (`[]`, `[x]`, `[a, b]`,
`[first, rest..]`, `[.., last]`; one `rest..` binds a `[]T` view of the
middle, and an array pattern names every element or ends in a rest),
`whole @ pattern` (the value under a name while its parts match; the
parts are views of it), wildcards, or-patterns `.A | .B`, and guards
`pattern if cond`. Slice arms are exhaustive when the rest-arm with the
fewest elements leaves no shorter length uncovered (`[]` with
`[x, rest..]`); otherwise a slice match needs `_ =>`.

**Binding mode** (**decided**, 68): matching a value binds copies of the
payload. Matching through a pointer, `match p.*`, binds owning payloads as
pointers with the pointer's mutability (`*mut List(T)` when `p: *mut ...`),
so a match arm can mutate the payload in place or return a pointer into
it; scalars are copied.

### 7.1 Binary patterns

```
match packet {
    <<version:4, ihl:4, total_len:16/big, rest:bytes>> => ...
    <<0x1b, '[', 'A', rest:bytes>> => Key.Up
    <<len:16/little, payload:len*8, rest:bytes>> => payload
    _ => ...
}
let written = try <<4:4, 5:4, 0:8, 1500:16/big, "ab">> into buf[..]
```

A segment is `name:size/modifiers` or a literal. Sizes are in bits, default
8; `bytes` takes the rest. Modifiers: `big` (default), `little`, `native`,
`signed`, `unsigned`, `float`, `utf8`. A binding with a constant size up to
64 bits is an integer of the smallest width that fits; larger or dynamic
sizes bind a `[]u8` view. Size expressions may use earlier bindings and are
evaluated with checked arithmetic. Construction writes into a `[]mut u8`
and returns `![]u8` (the written prefix) or `error.BufferTooSmall`.

## 8. Functions, generics, traits

### 8.1 Functions

A function's signature is its parameters, its return type, and its negative
effect bounds. Functions are checked in any order and may be recursive.
Overloading does not exist; a method and a free function may share a name.

### 8.2 Generics

`fn max(comptime T: type where T: Ord, a: T, b: T) -> T`. A `comptime T:
type` parameter is inferred from the arguments or passed explicitly
(`max(i32, a, b)`). Every distinct set of type arguments produces one
instance (monomorphization); there is no run-time representation of a type
parameter. Bounds name traits (`where T: Ord`) and may add effect bounds.
Generic structs and impls take parameters the same way: `struct Pair(T)`,
`impl(T) Pair(T)`.

### 8.3 Traits and impls

A trait declares method signatures with receivers `self: *Self` or
`self: *mut Self`, and may give default bodies. `impl Trait for Type`
implements it; `impl Type` adds inherent methods; the receiver is the first
parameter named `self`. Method calls dereference pointers automatically.
`derive(Eq, Ord)` on a struct synthesizes comparisons over its fields.
`derive(Clone)` synthesizes `.clone()`, a deep copy field by field, for a
struct or an enum whose fields and payloads clone: numbers, `String`,
`List`, `Map`, slices, optionals, tuples, arrays, weak references, other
`Clone` types, and reference classes (a retained copy). A pointer, a
trait object or a function value does not clone, and the error names the
field. Tuples, optionals and arrays of clonable things clone without a
derive; `where T: Clone` bounds a type parameter the same way.

### 8.4 Trait objects

`dyn Trait` is a fat pointer made by coercing `*T` or `*mut T` where `T`
implements the trait. Calls dispatch through a vtable generated per (trait,
type). A call through `dyn Trait` acquires every effect permitted, unless
the type carries bounds: `dyn Shape !allocates !blocks` is a distinct type
and only implementations satisfying the bounds coerce into it. A trait used
as an object may mention `Self` only in receiver position.

## 9. Effects

The effect set of a function is inferred: what its own body does, unioned
with the effects of everything it calls, to a fixpoint over the program.
The lattice is:

| effect | introduced by |
| --- | --- |
| `allocates` | creating or growing a `List`, `String`, `Map`, a `ref class` object, `format`, reading a file |
| `refcounts` | retaining or releasing a `ref class` value |
| `blocks` | I/O, `time.sleep`, joining a `for parallel`, waiting on a process |
| `shared_mutable` | reading or writing a mutable global |
| `nondeterministic` | `random`, `time.now`, `os.env`, spawning a process |
| `panics` | any operation that can panic and is not proven safe (section 9.1) |
| `ffi` | calling a foreign function |
| `unbounded_stack` | recursion, direct or mutual: the function is on a cycle of the call graph, so its stack use depends on its input |

Extern functions are assumed to have every effect. A negative bound on a
signature (`!allocates`) or on a function type is checked after inference;
the diagnostic names the site that introduced the effect through the call
chain, with the sentence recorded when the effect was added. Calling
through a function value acquires the effects its type permits (all of
them, minus the type's bounds), `unbounded_stack` among them.

`unbounded_stack` is discharged by rewriting the recursion as a loop over
an explicit stack; there is no proof for it, because a depth bounded by an
integer parameter is still a depth the caller cannot see (**decided**,
99). It matters at the export boundary (section 15): a host calling an
exported function that carries it must give it a stack sized for the
input.

### 9.1 Discharging `panics`

`panics` is not added when the compiler can prove the operation cannot
fail: indexing with the loop index of a `for` over the same slice, indexing
with a compile-time-known index into a known length, arithmetic whose
operand ranges (from types, literals, and dominating guards) fit the
result type, `x / y` after `y != 0`, `x.?` after `x != null`, `s[i]` after
`i < s.len`, and `s[i]` or `s[a..b]` after `s.len >= n` keeps them within
`n`. A guard proves its facts in the branch where it holds: the `then`
of an `if`, the body of a `while` at the top of every pass, and, negated,
the `else` of either. A fact about a `var` ends when the variable is
assigned, borrowed mutably, or a loop that may change it begins; facts
about a `let` hold for its scope. Everything else contributes `panics`.

### 9.2 Effects and the export ABI

An exported function proven `!panics` whose return type is not an error
union gets a direct C signature. Every other export returns an `int32_t`
status (0 for success, an error code otherwise, a `Panic` code for a panic)
and delivers its value through an out parameter. A panic therefore never
unwinds into a host.

## 10. Errors and panics

Errors are values of error sets. `error Name { A, B }` declares a set;
`error.Name` creates a member of the global set. A function returning
`Set!T` may return only members of `Set`. Predefined: `OutOfMemory Panic
InvalidRecord Truncated Overflow InvalidUtf8 NotFound IoError InvalidInput
BufferTooSmall`. `@errorName(e)` gives the name as `[]u8`.

A panic is an unrecoverable failure: index out of range, integer overflow,
`.?` on null, `unreachable`, `panic(msg)`, `expect` failing, a record
constraint violated by a runtime literal. A panic prints its message and
source location and exits with 101 in a program, or becomes a status code at
an export boundary. There is no catching a panic inside Nexium.

Records: a literal with compile-time-known values is checked at compile
time; with runtime values it panics on violation; `Record.new(.{ ... })`
returns `error.InvalidRecord` instead.

## 11. Modules and the standard library

`import a.b` loads `a/b.nx` relative to the root file; the module's `pub`
items are `b.item`. `import std.name` loads a standard library module that
is written in Nexium and embedded in the compiler: `strings`, `lists`,
`bytes`, `num`, `json`, `args`, `fs`, `time`, `regex`, `text`, `testing`,
`stream`, `net`, `http`, `thread`, `process` (see `docs/std.md`); std
modules may import each other. The builtin namespaces `math`, `io`, `os`,
`process`, `time`, `random`, `mem`, `net`, `thread`, `sync` are always in
scope and need no import. `error` names the anonymous error set as a type.
Packages: `import dep` and `import dep.module` load a dependency named in
the program's `nexium.toml` (`src/lib.nx` and `src/module.nx` of the
package, from a path or a git repository); inside a package, imports
resolve to the package's own `src/`. See `docs/packages.md`. A registry is
not part of the language; `ROADMAP.md` has it under the ecosystem.

The builtin methods of `List`, `String`, `Map`, slices, integers, floats,
and chars are listed in `docs/language.md`. They are implemented in the
compiler because ownership, effects, and the runtime need them there
(**decided**, 65).

## 12. Compile-time evaluation

`comptime expr` evaluates in an interpreter over the typed IR. Allowed:
pure computation, collections, calls to Nexium functions. Forbidden: I/O,
clocks, randomness, foreign calls, globals. A step budget and a call-depth
limit (32 nested calls in 1.0; see `KNOWN_ISSUES.md`) turn a runaway
evaluation into a compile error. `const` initializers, `@embedFile`, record
checks on literals, and `comptime test` blocks run here. Intrinsics:
`@typeName(T) @sizeOf(T) @alignOf(T) @truncate(T, x) @bitCast(T, x)
@min(a, b) @max(a, b) @errorName(e) @embedFile(path) @weak(x)
@refCount(x) @cImport(header) @cstr(literal) @target() @escape(v)`. `@bitCast`
reinterprets the bytes of one scalar as another of the same size;
`@min` and `@max` take two numbers of one type and carry the range of
their operands; `@target()` is `(os, arch, pointer_bits)` (`"windows"`,
`"macos"`, `"linux"`, `"bsd"`; `"x86_64"`, `"aarch64"`, ...; 64 or 32),
a constant of the C build for `if` in platform code, not a value of the
compile-time interpreter.

## 13. Concurrency

`for parallel x, i in items { ... }` runs the body over the index range on
a pool of hardware threads. The body may not have the `shared_mutable`
effect (directly or through calls), may not `return` or `break`, and
writes results through a mutable slice indexed by `i`. A panic in a worker
is re-raised in the caller after every worker finishes. The loop has the
`blocks` effect. Data races through captured mutable locals are the
programmer's responsibility (**decided**, 39).

Threads: `thread.start(f, arg)` runs a `fn(*mut T) -> void` value on a new
thread with its own context and returns a handle; `thread.join` waits and
re-raises a panic from the thread. `sync.*` provides mutexes and condition
variables as handles. `std.thread` builds `Thread(T, R)`, `Worker(T)`,
`Channel(T)` and `Mutex(T)` on these. Starting a thread carries the
`nondeterministic` and `shared_mutable` effects; joining, locking and
waiting `block`. There is no async: blocking threads and channels are the
concurrency model (decision 82).

## 14. C interoperability

`const lib = @cImport("header.h")` runs the C preprocessor on the header
and imports functions (including variadics), typedefs, structs of scalars,
pointers and arrays, enums, and integer, float, and string macros. `const
T*` maps to `*T`, other pointers to `*mut T`, `void*` to `*mut u8`, C
`long` to `i32` on Windows and `i64` elsewhere. A struct whose fields cannot
be translated (function pointers, bit-fields, nested definitions) is opaque:
usable through pointers. Unions, function-pointer typedefs, and
function-like macros are reported when used. Imported structs keep their C
names in the generated code. Foreign calls need `unsafe` and carry `ffi`.
`@cstr("...")` yields a NUL-terminated `*u8`.

Vendored C: `artifact link { c_sources = [...], libs = [...], include =
[...], lib_paths = [...], libs_windows = [...], libs_linux = [...],
libs_macos = [...] }`, paths relative to the declaring module. The same is
available as `--c-source`, `--link`, `--link-path`, `-I`.

`struct X layout(c)` gives C layout and makes the struct usable across the
export boundary.

## 15. Artifacts and the ABI

`artifact cabi { name = "lib", exports = [...] }` produces a shared library,
a static archive, and a C header. A panic inside an exported call is caught
at the boundary, reported as a status, and releases what the call acquired
(allocations, open files, sockets, held locks) before the status returns. `artifact python { name = "pkg" }`
produces a ctypes-based package and a wheel. `artifact rustlib { name =
"crate" }` produces a Cargo crate with `extern "C"` declarations, `#[repr(C)]`
structs, and safe wrappers returning `Result<T, NexiumError>`. `artifact
cli` names the executable. `artifact installer { name, publisher, version,
url, license, readme, files, add_to_path }` produces an Inno Setup script
and setup program on Windows and an `install.sh` with a tarball elsewhere
(see `docs/releasing-your-program.md`). `artifact node { name = "pkg" }`
produces an npm package: `index.js` calling the shared library through
`koffi`, `index.d.ts`, `package.json`.

`artifact cli { name = "tool" }` names the executable; `stack = "1G"` (K, M,
G or a byte count) runs `main` on a thread reserving that much stack, so a
recursion deeper than the platform's default gets the room it declared.

Exported functions take and return scalars, `layout(c)` structs, and slices
(as pointer plus length); they cannot return slices into their own storage,
cannot take `own` parameters, and cannot be generic. A program that declares
an embeddable artifact may not have mutable globals (S2). Each exported call
creates its own context on the stack, so no initialization is needed (S1).

## 16. The toolchain

`nx build run test check effects audit ship emit-c tokens fmt doc size
refcounts leaks lsp doctor version`. Build modes `debug safe fast small`;
`--target` cross-compiles to any target the C toolchain supports. The C
compiler is chosen in this order: `--cc`, `NX_CC`, `NX_ZIG`, a Zig bundled
next to `nx`, the system compiler for native macOS builds, `zig` on the
PATH. `nx fmt` is canonical and line-preserving. `nx doctor` reports the
installation.

The compiler emits C and embeds the runtime (`runtime/nx_rt.h`) and the
standard library into itself. It is written in Nexium under `self/`
(lexer, parser, checker, C emitter, driver, the tools; 26k lines) and
builds itself from the C it emits for itself (`bootstrap/nx.c`); the first
compiler, in Rust, agreed with it byte for byte on every source in the tree
before it was deleted at 1.0. `nx run tests/run.nx` is the test harness.

## 17. Status summary

| area | status |
| --- | --- |
| lexical, syntax, types, ownership, moves, `own` | implemented, tested |
| effects: inference, bounds, proofs, ABI | implemented, tested |
| patterns incl. binary patterns | implemented, tested |
| generics, traits, `dyn Trait` with bounds | implemented, tested |
| `ref class`, `weak`, arenas, `for parallel` | implemented, tested |
| comptime, `comptime test` | implemented, tested |
| `@cImport`, vendored C, opaque structs | implemented, tested |
| artifacts: cabi, python, rustlib, cli | implemented, tested |
| std: strings, lists, bytes, num, json, args, fs, time, regex, text, testing, stream, net, http, process, thread | implemented, tested |
| regions | R1, decided (88); the uncovered cases are listed in 5.6 |
| layouts `packed`, `soa`; strategies `pool`, `stack` | not part of the language (88); `layout(c)` and `arena` are |
| threads, channels (`std.thread`); no async | done (0.4) |
| packages (path and git dependencies), `node`, `installer` artifacts | done (0.5) |
| self-hosting: lexer, parser, checker, C emitter, driver and every tool in Nexium | done (0.9); the compiler builds itself from its own C, and no Rust remains (1.0) |
| conformance: `tests/spec`, one recorded program per claim | sections 2 to 14 covered; 15 and 16 by the ship tests |
