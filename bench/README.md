# bench: the numbers

Four programs, each written the same way in Nexium, C, Rust, Go and Python:
the same algorithm, the same data, the same output line. Each runs for about
a second in the compiled languages, long enough that starting the process
is noise rather than the measurement (Python takes 3 to 35 seconds).
`run.py` builds them (Nexium twice, in `safe` mode with its overflow and
bounds checks and in `fast` mode without; C with `-O2`, Rust with `-O`, Go
and Python as they come), runs each several times (three times when a run
takes over five seconds), checks that every language prints the same
answer, and keeps the median wall time.

| program | what it measures | prints |
| --- | --- | --- |
| `fib` | calls: `fib(42)` the naive way | `267914296` |
| `nbody` | floats: five bodies, 10,000,000 steps of a symplectic integrator | the energy, nine decimals |
| `sieve` | arrays: primes below 100,000,000 | `5761455` |
| `words` | hashing and strings: 10,000,000 generated words counted in a map | distinct and total |

```sh
python bench/run.py                       # the table, on this machine
python bench/run.py --runs 7 --out bench/results/latest.json --page docs/numbers.md
python bench/run.py --check bench/results/latest.json   # fails when Nexium, against C, got 25% slower
```

The check compares Nexium's time on each program as a multiple of C's time
in the same run, so a slower or faster runner cancels out; a baseline that
timed other sizes of the programs (its `suite`) is not compared. The Bench
workflow runs this on a GitHub runner weekly and at every tag, commits
`bench/results/latest.json` and `docs/numbers.md` (to `main`, or to the
branch `numbers` when `main` is protected; the site reads the page from
there and the next run measures against it), and fails on a regression. A
number here is a median on one machine on one day; the page names both.

The rule for a program: no tricks a reader would not write by hand in
that language, the standard library where the language has one, one file
each. A program that is faster because its language's compiler is smarter
is a fair result; one that is faster because it was rewritten to be is
not.
