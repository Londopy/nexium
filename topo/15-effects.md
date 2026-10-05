# Effects

Every function in a Nexium program carries a set of *effects*: the things
it might do besides compute its result. The compiler infers them, you can
see them, and you can forbid them on a signature. When a promise is broken,
the diagnostic walks the call chain to the line responsible. No other part
of the language does more to make a program's behaviour visible, and no
other part costs less: an effect is a fact about the code, not a check at
run time.

{{include topo/code/effects.nx}}

{{output topo/code/effects.expected}}

## The eight effects

| effect | the function may... |
| --- | --- |
| `allocates` | take memory from the heap: build a `String`, grow a `List` |
| `refcounts` | retain or release a `ref class` reference |
| `blocks` | wait: I/O, sockets, sleeping, joining a thread, locking |
| `shared_mutable` | read or write a mutable global, or lock |
| `nondeterministic` | depend on the clock, randomness, the environment, a thread's timing |
| `panics` | stop the program: an index that may be out of bounds, checked arithmetic that may overflow |
| `ffi` | call foreign code through `@cImport` |
| `unbounded_stack` | recurse without a bound the compiler can see, or call through a function value |

`nx effects file.nx` prints the inferred set for every function:

```
$ nx effects topo/code/effects.nx
inferred effects (7 functions)

fn mean(xs: []f64) -> f64  (pure)
    at topo/code/effects.nx:3:1
fn dot(a: []f64, b: []f64) -> f64  (pure)
    at topo/code/effects.nx:11:1
fn describe(xs: []f64) -> String  allocates
    at topo/code/effects.nx:18:1
fn apply(xs: []mut f64, f: fn(f64) -> f64 !allocates !panics) -> void  refcounts blocks shared_mutable nondeterministic ffi unbounded_stack
    at topo/code/effects.nx:23:1
fn halve(x: f64) -> f64  (pure)
    at topo/code/effects.nx:27:1
fn hash(data: []u8) -> u32  (pure)
    at topo/code/effects.nx:30:1
fn main() -> void  allocates refcounts blocks shared_mutable nondeterministic ffi unbounded_stack
    at topo/code/effects.nx:39:1
```

A function's effects are its own, plus the effects of everything it calls,
computed as a fixpoint over the whole program. `describe` allocates because
`format` does. `main` has everything but `panics`: `describe` brings
`allocates`, and `apply` the rest, because it calls through a function
value: the value's type promises `!allocates !panics` and nothing else, so
the call is assumed to do anything else it could. Nothing in the program
can panic (the section on proofs below says why), so neither can `main`. A
call into foreign code is assumed to do everything.

## Promises

```nexium
fn mean(xs: []f64) -> f64 !allocates !panics {
```

A negative bound on a signature is checked. Break it and the compiler
says exactly how:

{{include topo/code/effects_fails.nx}}

{{output topo/code/effects_fails.expected}}

The first note points at the call inside `tag` that brings the effect in;
the second at the exact line that introduces it (the `format` call inside
`label`) and the reason, however far down the chain. Rename `label` to
something in another module, five calls deep, and the notes still lead you
to the line.

Bounds go on function types too. `apply` accepts only functions that are
`!allocates !panics`; passing one that could panic is a type error at the
call site, and a closure has to satisfy the same bounds. `dyn Trait
!allocates` is the same idea for trait objects (chapter 7): a `List(dyn
Shape !allocates)` can only hold implementations that keep the promise.

## Proofs: why `hash` is pure

`hash` uses `^=` and `*%=`: the exclusive-or cannot overflow and the
wrapping multiply is defined to wrap, so nothing in it can panic. `mean`
indexes nothing and divides by a count it checked is not zero. The
compiler works this out; you do not annotate anything.

The `panics` effect is *discharged by proof* where the compiler can see
that an operation cannot fail: indexing with the loop variable of a `for`
over the same slice, an index the compiler knows is in range, arithmetic
whose operands have known ranges, a division whose divisor was compared
against zero on the way in, an index a comparison with the length guards.
`dot` indexes `b[i]` with an index drawn from `a`, which proves nothing
about `b`; the guard `i < b.len` around it does, so `dot` is pure. Delete
the guard and `dot` carries `panics`, and
`nx explain topo/code/effects.nx dot panics` answers "index may be out of
bounds" at `b[i]`. That is what the effect is for: it tells you.

## Why this matters

- **Libraries.** A function without `panics` gets its natural C signature
  when shipped (chapter 20); the caller in Python or C need not handle a
  failure that cannot happen. A function without `allocates` or `blocks`
  can be called from an audio callback or an interrupt.
- **Reading code.** `nx effects` on a stranger's module is a summary a
  comment could never be trusted to be.
- **Refactoring.** Put `!allocates` on the hot loop, and any change that
  sneaks an allocation in fails to compile, with the line.
- **Embedding.** A program that declares an embeddable artifact is
  forbidden mutable globals altogether, so two hosts loading the same
  library cannot interfere.

`nx audit file.nx` is the effect system's sibling for the things it
excludes: it lists every `unsafe` block and every mutable global, the two
places a program steps outside what the compiler proves.

Next: [threads and parallel loops](16-concurrency.html).
