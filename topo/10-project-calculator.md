# Project: a calculator

Every language has to earn a calculator. This one handles `+ - * / ^`,
parentheses, unary minus, variables and assignment, reports five kinds of
mistake by name, and tests itself. It is a tokenizer, a recursive-descent
parser and an evaluator, which is also the skeleton of every interpreter
and compiler, including the one you are using.

{{include topo/code/calc.nx}}

Run it with no arguments for the built-in session, or give it lines:

```bash
$ nx run topo/code/calc.nx -- "r = 3" "pi * r ^ 2"
r = 3 => 3.0
pi * r ^ 2 => 28.274333882308138
```

{{output topo/code/calc.expected}}

## The tokenizer

```nexium
enum Tok { Num(f64), Name(String), Op(u8), LParen, RParen, End }
```

An enum with payloads is the natural token type: a number carries its
value, a name its text, an operator its character, and the brackets and the
end marker carry nothing. `tokenize` walks the bytes once and returns a
`List(Tok)`, or an error for a character it does not know, through the
`Calc` error set every function in the file shares.

A `String` inside a `Tok.Name` is owned by the token, and the token by the
list. Nothing is copied when the parser looks at it, because `atom` reads
tokens through a pointer, `let t = &c.toks[c.pos]`, and matches on `t.*`.

## The parser

Precedence is written as functions: `expression` handles `+` and `-` and
calls `term` for its operands; `term` handles `*` and `/` and calls
`power`; `power` handles `^` and recurses on itself for the right-hand
side, which makes `2 ^ 3 ^ 2` mean `2 ^ (3 ^ 2)`; `unary` handles the
minus; `atom` handles numbers, names and parentheses, calling `expression`
again for what is inside them. Each level only sees the operators it owns
and hands everything else down, which is all recursive descent is.

The parser and the evaluator are one pass: each function returns the value
of what it parsed. That is enough for a calculator; a compiler would build
a tree here and walk it later.

`Cursor` carries the token list and a position, and its methods are the
small vocabulary the grammar functions share: `peek_op`, `is_op`,
`at_end`. `Session` carries the variables. Both are passed as pointers,
`*Session` to read and `*mut Cursor` to advance, so the grammar functions
borrow them the whole way down and `evaluate` still owns them at the end.

## Errors, once more

```nexium
            return s.vars.get(n[..]) orelse return error.UnknownName
```

`orelse return error.UnknownName` is the idiom for "if there is no such
variable, this whole call fails". Note the two `return`s: the inner one is
what `orelse` does when the optional is `null`, the outer one returns the
value when it is not. Every error a user can cause has a name in the `Calc`
set, and `main` prints that name; a bug in the calculator itself would be a
panic instead, with a location.

## Tests

```nexium
test "precedence and parentheses" {
    expect_eq(try calc("1 + 2 * 3"), 7.0)
```

`test "name" { }` blocks live next to the code they test and run with `nx
test topo/code/calc.nx`:

```
ok    precedence and parentheses
ok    variables persist within a session
ok    errors are named

3 passed, 0 failed
```

`try` works inside a test: a test that hits an unexpected error fails with
its name. The third test checks an error by matching it in a `catch`
handler and returning early, then `expect(false)` catches the case where no
error came at all. Chapter 14 is about testing in full.

## Things to try

- Add `%` for remainder. It belongs in `term`.
- Add functions: `sqrt(2)`, `max(1, 2)`. A `Name` followed by `LParen` in
  `atom` is a call; `math.sqrt` and `math.max` are already there.
- Make it a real REPL: read lines with `io.read_line()` in a `while`
  loop until it returns `null`.

Next: [a project: the to-do list](11-project-todo.html), where a program
keeps state in a file.
