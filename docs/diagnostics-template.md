# Per-feature diagnostics

The section every feature's entry in the specification fills in before the
feature is built (template v0.1). It serves the roadmap's line
[errors you can fix alone](../ROADMAP.md#errors-you-can-fix-alone): an
intermediate programmer understands and fixes any problem in their own
Nexium code from the message and its page, without reading the compiler
or the standard library.

## How to use

Copy the FEATURE DIAGNOSTICS block below into every feature entry in the
spec, and fill it in BEFORE implementing the feature. Write the errors
first. If you can't write a clear error for a mistake, change the
feature, not the message. A feature isn't done until its DONE CHECKLIST
is complete.

## Placeholders

- `<angle brackets>`: you fill these in when writing the spec.
- `{curly braces}`: the compiler fills these in from the user's code.
  After a failed build, `nx explain` uses that build's details. Run cold,
  it falls back to the minimal example.

## Registry rules

- Every diagnostic has a code: NX plus 4 digits, assigned in order from
  one registry file.
- Codes are never reused or renumbered, even after they're retired.
- Errors, warnings, and runtime panics all draw from the same registry.
- The compiler can't construct a diagnostic without a registered code.
  Its own types enforce this, not code review.
- Every message ends by pointing to `nx explain NX####`.

## Message rules

- Lead with what happened in the user's code, using their names.
- Point at the line to change. If that's not where the problem was
  caught, show both locations and the chain between them.
- Use plain words for an intermediate programmer. No compiler internals.
- Every technical term in a message must have a concept page.
- Only show a fix after the compiler has re-checked that the patched code
  compiles.
- One mistake produces one error. No cascades.

## The block

```text
======================================================================
FEATURE DIAGNOSTICS: <feature name>
======================================================================

Spec section:  <section number>
Status:        <draft | reviewed | implemented | verified>

CLARITY GATE (answer before implementing)
  Can every mistake below be explained in plain words, to an
  intermediate programmer, without compiler internals?   <yes | no>
  If no:  <which mistake, and what change to the feature would fix it>
  A "no" blocks implementation until the design changes.

LIKELY MISTAKES
  Start with the 3 most common. Copy the entry below once per mistake.
  List each here as:  NX<####>  <kind>  <title>

----------------------------------------------------------------------
NX<####>  <title: one plain sentence, using {slots}>
----------------------------------------------------------------------

Kind:     <error | warning | runtime panic>
Mistake:  <what the programmer did, in one sentence>

PROVENANCE
  Caught at:       <where the compiler or runtime notices the problem>
  Actual mistake:  <where the programmer needs to make the change>
  Chain:           <the facts connecting the two, e.g. "{name} is
                   {type_a} because of line {line_a}, but line
                   {line_b} needs {type_b}">
  If the two locations differ, the message must show both, plus the
  chain.

MESSAGE (exact text, rendered for the minimal example)
  <header with code, labeled source lines, help with any fix, and the
   `nx explain` pointer>

EXPLAIN PAGE (what `nx explain NX<####>` prints)
  What happened:      <1-2 sentences about the user's code, in {slots}>
  Why it's an error:  <the language rule in plain words, plus its spec
                      section>
  Minimal example:    <smallest program that triggers exactly this code>
  Fixes:
    1. <fix>
       When to choose it:  <situation where this is the right fix>
       Auto-applicable:    <yes | no>
    2. <fix>
       When to choose it:  <situation>
       Auto-applicable:    <yes | no>
  Related codes:      <NX####, NX####>
  Terms used:         <term -> concept page, one per line>

TESTS
  Trigger:   The minimal example produces exactly NX<####> at the
             expected location, and no other diagnostics.
  Mutation:  <one-token breaks of known-good programs that must produce
             this code, each with its expected location>
  Fixes:     Every auto-applicable fix, applied to the minimal example,
             compiles cleanly.
  Human:     <n> intermediate programmers, message and explain page
             only.
             Median time to fix:       <mm:ss>
             Fixed without searching:  <x of n>
             Bar:                      <x of n, set before testing>
             Below the bar: rewrite the message or page, then retest.
             Run these in batches; one session can cover many codes.

HISTORY
  Introduced:  <version>
  Changed:     <version: what changed>
  Retired:     <version, or "active">. A retired code keeps its page,
               with a note on what replaced it.

DONE CHECKLIST
  [ ] Clarity gate is "yes" for every mistake
  [ ] Every mistake has a registered code
  [ ] Every message shows the actual mistake, not just where it's caught
  [ ] Every explain page field is filled in
  [ ] Every term used has a concept page
  [ ] Trigger, mutation, and fix tests pass in CI
  [ ] Human test meets the bar
  [ ] Warnings and runtime panics from this feature have entries too
```

## Worked example

Variable bindings, in Nexium's syntax: `let` declares a binding that
can't change and `var` one that can (specification 5.1). The codes are
placeholders until the registry assigns real ones. Today the compiler
says `` `count` is immutable; declare it with `var` `` at the assignment,
without a code or the declaration's line, and warns about neither NX0043
nor NX0044.

```text
FEATURE DIAGNOSTICS: Variable bindings

Spec section:  5.1
Status:        draft

CLARITY GATE (answer before implementing)
  Can every mistake below be explained in plain words, to an
  intermediate programmer, without compiler internals?   yes

LIKELY MISTAKES
  NX0042  error    can't assign to {name} because it's immutable
  NX0043  warning  {name} is declared with `var` but never changes
  NX0044  warning  {name} is declared but never used
  Only NX0042 is filled in below.

----------------------------------------------------------------------
NX0042  can't assign to {name} because it's immutable
----------------------------------------------------------------------

Kind:     error
Mistake:  Assigned a new value to a binding declared with `let`.

PROVENANCE
  Caught at:       the assignment, {file}:{line_b}
  Actual mistake:  the declaration, {file}:{line_a}, `let` where
                   `var` was meant
  Chain:           "{name} can't change because line {line_a} declares
                   it with `let`, and line {line_b} assigns to it"

MESSAGE (exact text, rendered for the minimal example)
  error[NX0042]: can't assign to `count` because it's immutable
    --> src/main.nx:3:5
     |
   2 |     let count = 0
     |     --------- declared here with `let`, so it can't change
   3 |     count = count + 1
     |     ^^^^^^^^^^^^^^^^^ this assigns a new value to it
     |
  help: declare it with `var` so it can change
     |
   2 |     var count = 0
     |     ~~~
     = full explanation: `nx explain NX0042`

EXPLAIN PAGE (what `nx explain NX0042` prints)
  What happened:
    Line {line_b} assigns a new value to {name}, but line {line_a}
    declares {name} with `let`, so its value can't change.
  Why it's an error:
    A binding declared with `let` can't change; one that changes is
    declared with `var`, so anyone reading the declaration knows
    whether the value ever changes. (Specification 5.1)
  Minimal example:
    fn main() {
        let count = 0
        count = count + 1
    }
  Fixes:
    1. Declare {name} with `var` on line {line_a}.
       When to choose it:  the value really does need to change.
       Auto-applicable:    yes
    2. Put the new value in a new binding instead of changing {name}.
       When to choose it:  you only need the new value, and the old
                           one can stay as it is.
       Auto-applicable:    no
  Related codes:
    NX0043  {name} is declared with `var` but never changes
  Terms used:
    binding    -> concepts/bindings
    immutable  -> concepts/mutability

TESTS
  Trigger:   minimal example -> exactly NX0042 at 3:5, nothing else
  Mutation:  tests/good/counter.nx: change `var` to `let` on line 2
             -> NX0042 at 5:9
  Fixes:     fix 1 applied to the minimal example compiles cleanly
  Human:     5 intermediate programmers, message and explain page
             only.
             Median time to fix:       <mm:ss>
             Fixed without searching:  <x of 5>
             Bar:                      4 of 5

HISTORY
  Introduced:  0.1
  Changed:     none
  Retired:     active

DONE CHECKLIST
  In progress. Still open: NX0043 and NX0044 entries, concept pages,
  CI tests, human test.
```
