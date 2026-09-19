# The Nexium Language Specification

Version 0.2, describing Nexium as implemented by `nx` 0.2.1. This document is
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
`module.item`. Fields are visible to any code that can see the struct.
Struct fields are separated by commas or newlines and may carry defaults
(`verbose: u8 = 0`).

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
function with an `own` parameter cannot be used as a function value.

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

### 5.6 Regions

Rule R1 is enforced: a function may not return a slice or a pointer into
one of its own locals. Views into parameters are allowed, because the
caller owns that storage. A view stored into an outer variable is not
tracked (**planned**: R2 to R4).

### 5.7 Allocation scopes

`using arena { ... }` installs a bump allocator for the block. Values
created inside are allocated from the arena, their releases are no-ops, and
the arena is freed as a whole at the end of the block. Containers created
outside the block keep using the heap when they grow inside it, so
collecting results into an outer `List`, `String`, or `Map` is safe. Values
created inside must not escape the block (not checked). `pool` and `stack`
strategies are **planned**.

## 6. Expressions and statements

### 6.1 Operators

Arithmetic `+ - * / %` traps on overflow (a defined panic). Wrapping
`+% -% *%` and saturating `+| -| *|` never fail. Bitwise `& | ^ ~ << >>`.
Comparison `== != < <= > >=` on numbers, chars, bools, `[]u8`, `String`,
unit enums, and types that `derive(Eq)` / `derive(Ord)`. Logical `and`,
`or`, `!`, short-circuiting. `x |> f(a)` is `f(x, a)`.

Compound assignment: `= += -= *= /= %= &= |= ^= <<= >>= +%= -%= *%=`.

### 6.2 Control flow

- `if (c) a else b` is an expression when both branches have a type. An
  `if` without `else` is a statement. Without braces, the body is one
  statement, so `if (c) x = 1 else x = 2` and `if (c) return v` are valid.
- `if (opt) |v| { } else { }` unwraps an optional.
- `while (c) { }`, `for (items) |x| { }`, `for (items) |x, i| { }`,
  `for (a, b) |x, y| { }` (lengths must match), `for (lo..hi) |i| { }`.
  Iteration works over arrays, slices, lists, strings, and map keys.
  Loop bodies take braces.
- `break`, `continue`, `return`, each optionally with a label:
  `outer: for (...) |a| { ... continue :outer }`. A labeled block yields a
  value with `break :label value`.
- `match v { pattern => expr, ... }` (section 7).
- Blocks are expressions whose value is their trailing expression.

### 6.3 Errors and optionals in expressions

`try e` propagates an error from an `E!T` to the enclosing function, which
must return an error union. `e catch |err| handler` handles it; the handler
is an expression of `T`'s type, or a jump (`return`, `break`, `continue`).
`opt orelse default` and `opt orelse return x` unwrap or jump. `opt.?`
unwraps and panics on null. `defer stmt` runs at scope exit; `errdefer stmt`
only when the scope exits through an error.

### 6.4 Closures

`|[captures] params| -> R { body }`. Captures are explicit: `[x]` copies,
`[&x]` and `[&mut x]` take references. A closure coerces to `fn(...) -> R`
and to `fn(...) -> R !bounds` when it satisfies the bounds.

### 6.5 Formatting

`println(fmt, .{args})`, `print`, `eprintln`, and `format(...) -> String`
take a literal format string with placeholders `{}` `{x}` `{X}` `{b}` `{o}`
`{e}` `{c}` `{:.N}` `{>N}` `{<N}`; `{{` and `}}` are literal braces.
Formatting is compiled: each placeholder becomes a typed write.

## 7. Patterns

`match` is exhaustive for enums and bools; other scrutinee types need a
`_ =>` arm. Patterns: literals (integers, chars, strings, bools, negative
literals), integer ranges `1..=9`, enum variants `.Variant(p, q)` or
`.Variant { field: p }`, `null` and a binding on optionals (the binding is
the payload), `error.Name` and a binding on error unions (the binding is
the success value), tuples, wildcards, or-patterns `.A | .B`, and guards
`pattern if cond`.

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
| `unbounded_stack` | reserved |

Extern functions are assumed to have every effect. A negative bound on a
signature (`!allocates`) or on a function type is checked after inference;
the diagnostic names the site that introduced the effect through the call
chain, with the sentence recorded when the effect was added. Calling
through a function value acquires the effects its type permits (all of
them, minus the type's bounds).

### 9.1 Discharging `panics`

`panics` is not added when the compiler can prove the operation cannot
fail: indexing with the loop index of a `for` over the same slice, indexing
with a compile-time-known index into a known length, arithmetic whose
operand ranges (from types, literals, and dominating `if` guards) fit the
result type. Everything else contributes `panics`.

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
`bytes`, `num`, `json`, `args` (see `docs/std.md`). The builtin namespaces
`math`, `io`, `os`, `process`, `time`, `random`, `mem` are always in scope
and need no import. **planned**: `std.fs`, `std.time`, `std.regex`,
`std.net`, `std.http`, packages with a manifest (ROADMAP phases 1 to 3).

The builtin methods of `List`, `String`, `Map`, slices, integers, floats,
and chars are listed in `docs/language.md`. They are implemented in the
compiler because ownership, effects, and the runtime need them there
(**decided**, 65).

## 12. Compile-time evaluation

`comptime expr` evaluates in an interpreter over the typed IR. Allowed:
pure computation, collections, calls to Nexium functions. Forbidden: I/O,
clocks, randomness, foreign calls, globals. A step budget turns a runaway
evaluation into a compile error. `const` initializers, `@embedFile`, record
checks on literals, and `comptime test` blocks run here. Intrinsics:
`@typeName(T) @sizeOf(T) @truncate(T, x) @errorName(e) @embedFile(path)
@weak(x) @refCount(x) @cImport(header) @cstr(literal)`.

## 13. Concurrency

`for parallel (items) |x, i| { ... }` runs the body over the index range on
a pool of hardware threads. The body may not have the `shared_mutable`
effect (directly or through calls), may not `return` or `break`, and
writes results through a mutable slice indexed by `i`. A panic in a worker
is re-raised in the caller after every worker finishes. The loop has the
`blocks` effect. Data races through captured mutable locals are the
programmer's responsibility (**decided**, 39).

**planned** (ROADMAP phase 2): threads, channels with move semantics,
mutexes as `ref class` values gated by `shared_mutable`. Async is an open
decision; the default answer is threads without colored functions.

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
a static archive, and a C header. `artifact python { name = "pkg" }`
produces a ctypes-based package and a wheel. `artifact rustlib { name =
"crate" }` produces a Cargo crate with `extern "C"` declarations, `#[repr(C)]`
structs, and safe wrappers returning `Result<T, NexiumError>`. `artifact
cli` names the executable. **planned**: `node`, `installer`.

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

The compiler is 22.7k lines of Rust, emits C, and embeds the runtime
(`runtime/nx_rt.h`) and the standard library into itself. The self-hosted
compiler under `self/` replaces it stage by stage (ROADMAP phase 4).

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
| std: strings, lists, bytes, num, json, args | implemented, tested |
| regions | R1 only |
| layouts `packed`, `soa`; strategies `pool`, `stack` | planned |
| std: fs, time, regex, net, http | planned (phases 1 and 2) |
| threads, channels; async decision | planned (phase 2) |
| packages, registry, `node`, `installer` | planned (phase 3) |
| self-hosting | lexer done; parser next (phase 4) |
