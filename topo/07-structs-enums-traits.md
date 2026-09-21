# Structs, enums and traits

Three ways to make a type: a `struct` holds several values at once, an
`enum` holds one of several cases, and a `trait` names what a set of types
can do. With generics, they are the whole type system a program needs.

{{include topo/code/shapes.nx}}

{{output topo/code/shapes.expected}}

## Structs and methods

```nexium
struct Point derive(Eq) { x: f64, y: f64 }
```

A struct is a value: assigning one copies its fields, and two `Point`s are
two `Point`s. Fields are separated by commas or newlines and may have
defaults (`verbose: bool = false`). `derive(Eq)` makes `==` work field by
field; `derive(Ord)` adds `<` and friends, and `derive(Hash)` lets the type
be a map key. `derive(Clone)` gives it `.clone()`, a deep copy field by
field, once every field can clone.

Methods live in an `impl` block. The receiver is spelled out:

- `fn origin() -> Point`: no receiver, an *associated function*, called as
  `Point.origin()`.
- `fn distance(self: *Self, ...)`: a method that reads the struct; `Self`
  is the type of the `impl`.
- `fn shift(self: *mut Self, ...)`: a method that changes it, which only a
  `var` (or something reached through `*mut`) can call.

`p.distance(&o)` passes a pointer to `o` because the parameter is `*Point`;
`p.shift(1.0, 1.0)` takes `&mut p` for you. Field access through a pointer
needs no arrow: `self.x` and `other.x` both just work.

## Enums

```nexium
enum Shape {
    Circle(f64),
    Rect { w: f64, h: f64 },
    Triangle(f64, f64, f64),
    Empty,
}
```

An enum value is exactly one of its cases, and a case may carry data: a
tuple of values (`Circle(f64)`), named fields (`Rect { w, h }`), or nothing
(`Empty`). Construct with `Shape.Circle(1.0)` or `Shape.Rect{ .w = 2.0, .h
= 3.0 }`. The only way to look inside is `match`:

```nexium
    match s.* {
        .Circle(r) => math.PI * r * r,
        .Rect(w, h) => w * h,
        ...
        .Empty => 0.0,
    }
```

`.Circle(r)` binds the payload. The match must cover every case; leave one
out and the compiler names it. That is the point of enums over "an integer
tag and some fields": adding a case later makes every match that forgot it
a compile error, not a silent fall-through.

`s.*` reads through the pointer `s: *Shape`. Matching through a pointer
binds the payloads by reference, which matters when they own something (a
`List` inside a case is not copied; chapter 8).

An enum whose cases are all empty (a plain "one of these") casts to an
integer with `as` and back with a `match`.

## Traits and `dyn`

```nexium
trait Named {
    fn name(self: *Self) -> String
}

impl Named for Point { ... }
impl Named for Shape { ... }
```

A trait is a list of method signatures. `impl Trait for Type` provides them,
and from then on `p.name()` works on a `Point`. Traits are how generic code
states what it needs (`where T: Ord` below), and how one function can accept
different types at run time:

```nexium
fn describe(thing: dyn Named) {
    println("this is a {}", .{thing.name()})
}
```

`dyn Named` is a *trait object*: a pointer to any value whose type
implements `Named`, plus a table of that type's methods. `describe(&p)` and
`describe(&shapes[0])` pass a `Point` and a `Shape` through the same
parameter, and the call dispatches at run time. A `List(dyn Named)` holds a
mixed collection. Trait objects carry effects too: `dyn Named !allocates`
accepts only implementations that do not allocate (chapter 15).

## Generics

```nexium
fn largest(comptime T: type where T: Ord, items: []T) -> ?T {
```

A generic function takes a type as a `comptime` parameter. `where T: Ord`
says the function will compare values, so only ordered types are accepted;
the call names the type, `largest(i32, numbers[..])`, and the compiler
generates one version of the function per distinct type it is called with
(monomorphization: no boxing, no runtime type information). Generic structs
look the same: `Pair(T)`, constructed as `Pair(i32){ ... }`, with methods in
`impl(T) Pair(T)`.

The standard library's `std.lists` is written this way (`lists.map(i32,
i32, xs[..], double)`), which is why its functions take the element type
first.

## Other declarations

Two you will meet in later chapters and in other people's code:

```nexium
type Meters = distinct f64       // a new type with f64's representation and no implicit conversion
record Dose { mg: f64 where value > 0.0 }   // a struct whose invariants are checked when it is built
ref class Node { value: i32, next: ?Node }  // a reference-counted object (chapter 8)
```

Next: [ownership](08-ownership.html), the chapter that makes the language
what it is.
