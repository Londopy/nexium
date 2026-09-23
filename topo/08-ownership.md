# Ownership

Every value in a Nexium program has exactly one owner, and the owner frees
it when its scope ends. That one sentence replaces a garbage collector, and
most of what a borrow checker does. This chapter is the rules, with the
compiler's messages for the cases it rejects.

{{include topo/code/ownership.nx}}

{{output topo/code/ownership.expected}}

## Values that own, values that do not

`List(T)`, `String` and `Map(K, V)` own a heap buffer. So does a struct
that contains one, a tuple that contains one, an enum case that carries one.
Everything else, numbers, `bool`, `char`, arrays of numbers, structs of
numbers, is a plain value that is copied when assigned and owns nothing
beyond its own bytes.

For an owning value, assignment is a **move**:

```nexium
    let b = a                          // `a` is moved into `b`
```

After this line `b` owns the list and `a` is a name that refers to nothing.
Using `a` again is a compile error, and the message says so:

{{include topo/code/ownership_fails.nx}}

{{output topo/code/ownership_fails.expected}}

The same happens when an owning value is passed by value to a function that
takes `own`, stored into a field, appended to a list or put into a map: the
value goes there, and the name that held it is done. Moves are tracked per
branch: a value moved inside one `if` branch is still available in the other
and counts as moved after the `if`.

When you need two, say so: `b.clone()` is a deep copy of the whole thing.
Nothing is copied behind your back; every allocation in a program is one
you can point at.

## Borrowing: pointers and views

A function should not have to own what it only wants to read. Parameters
are **borrowed**: passing `c` to `fn longest(words: *List(String))` lends it
a pointer, the function reads through it, and the caller still owns `c`
afterwards. A borrowed parameter cannot be moved out of, and cannot be
changed unless the pointer is `*mut`:

```nexium
fn shout(words: *mut List(String)) {   // may change the list; the caller keeps it
```

`&c` makes a `*List(String)`; `&mut c` makes a `*mut List(String)` and
needs `c` to be a `var`. Reading a field or calling a method through either
needs no special syntax; assigning a whole new value through a `*mut T` is
`p.* = value`.

A **slice** is the other kind of borrow: `text[0..4]` is a view of four
bytes of `text`, a pointer and a length, and it owns nothing. Slices are
what almost every function takes (`[]u8` for text, `[]T` for a sequence),
because an array, a `List`, a `String` and a piece of any of them all turn
into one for free. Two rules keep views honest:

- A view into a **local** may not be returned from the function (rule R1):
  the local dies when the function returns and the view would point at
  freed memory. The compiler says:

{{include topo/code/ownership_fails2.nx}}

{{output topo/code/ownership_fails2.expected}}

- A loop variable in `for w in c`, and the binding of `if let v = opt`
  over a stored optional, are views of the element: you may read them and
  clone them, not move them out. The message names the fix.
- The compiler also refuses a view kept past its storage: stored into a
  variable that outlives the local it points into, read after the `List`
  or `String` it points into grew, or after the value it points into
  moved away (rules V2 to V4 in `SPEC.md` 5.6; warnings in 1.2, errors
  since 1.3). The error names the storage, the line it died on, and the
  fix: take the view later, keep an owned copy (`.clone()` of the `String`
  or `List` itself; a view's clone is the same view), or move a clone
  instead of the value. `nx fix` makes that last edit for you.

Views into parameters may be returned, because the caller owns their
storage. What the rules cannot see, a `*mut` obtained in `unsafe` code,
stays the programmer's promise; a debug build fills freed memory with a
fixed byte so such a mistake fails loudly rather than quietly.

## `own`: taking a value on purpose

```nexium
fn make_token(kind: u8, own text: String) -> Token {
    return Token{ .kind = kind, .text = text }
}
```

`own` on a parameter says the function takes the value: the caller's name
is moved, no copy is made, and the function may move it on (into the struct
here) or let it drop when it returns. It is the right tool for constructors
and for anything that stores what it is given. A function with an `own`
parameter cannot be used as a function value, since the type of a function
value does not say who owns the argument.

## Scope exit

An owning value is freed when the scope that owns it ends, in reverse order
of declaration, and that is the only thing that happens automatically at
scope exit. There are no destructors to write; `defer` (chapter 6) is for
the actions that are not memory. Moving a value out of a scope (returning
it, storing it) hands the duty to the new owner. If you want to see it,
`nx leaks program.nx` runs a debug build that counts every allocation and
reports what was still alive at exit; for every program in this book the
answer is `none`.

## Reference counting: `ref class`

Sometimes a value really is shared: a node with several parents, a cache
every part of the program reads. For those, `ref class`:

```nexium
ref class Node { value: i32, next: ?Node, back: ?weak Node }
```

A `ref class` value is a reference to an object with a count. Copying it
(`let alias = first`) retains, the count goes up, and the object is freed
when the last reference is gone: deterministic, immediate, no tracing
collector pausing anything. `@refCount(x)` reads the count. Fields are
assigned through any reference (`first.next = second`), which is what makes
the type a class and not a struct.

The one thing counting cannot do is a cycle. Two objects that point at each
other keep each other alive forever, so a back edge is a `weak` reference:
`@weak(first)` (or `first.weak()`) does not count, and `w.upgrade()` gives
an `?Node` that is `null` once the object is gone. `nx leaks` reports the
cycles you forgot.

The effect system (chapter 15) tracks reference counting like everything
else: a function that copies a reference has the `refcounts` effect, and a
function that promises `!refcounts` cannot.

## The rules on one card

| you write | what happens |
| --- | --- |
| `let b = a` (owning) | move; `a` is gone |
| `let b = a.clone()` | deep copy; both live |
| `f(a)` with `fn f(x: List(T))` | borrow; `a` stays yours, `f` can only read |
| `f(&mut a)` with `fn f(x: *mut List(T))` | borrow; `f` may change it |
| `f(a)` with `fn f(own x: List(T))` | move; `f` keeps it |
| `a[i..j]`, `a[..]`, `for x in a` | a view; do not outlive `a`, do not move out |
| `let r = obj` (ref class) | retain; freed with the last reference |
| `@weak(obj)` | no retain; `upgrade()` when you need it |

Next: [collections](09-collections.html).
