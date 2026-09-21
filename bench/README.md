# bench: the numbers

Four small programs, each written the same way in Nexium, C, Rust, Go and
Python: the same algorithm, the same data, the same output line. `run.py`
builds them (Nexium in `fast` mode, C with `-O2`, Rust with `-O`, Go as it
comes, Python as it comes), runs each several times, checks that every
language prints the same answer, and keeps the median wall time.

| program | what it measures | prints |
| --- | --- | --- |
| `fib` | calls: `fib(32)` the naive way | `2178309` |
| `nbody` | floats: five bodies, 200,000 steps of a symplectic integrator | the energy, nine decimals |
| `sieve` | arrays: primes below 10,000,000 | `664579` |
| `words` | hashing and strings: 2,000,000 generated words counted in a map | distinct and total |

```sh
python bench/run.py                       # the table, on this machine
python bench/run.py --runs 7 --out bench/results/latest.json --page docs/numbers.md
python bench/run.py --check bench/results/latest.json   # fails when Nexium got 25% slower
```

The Bench workflow runs this on a GitHub runner (the same kind each time)
weekly and at every tag, commits `bench/results/latest.json` and
`docs/numbers.md` (to `main`, or to the branch `numbers` when `main` is
protected; the site reads the page from there and the next run measures
against it), and fails on a regression. A number here is a median on one
machine on one day; the page names both.

The rule for a program: no tricks a reader would not write by hand in
that language, the standard library where the language has one, one file
each. A program that is faster because its language's compiler is smarter
is a fair result; one that is faster because it was rewritten to be is
not.
