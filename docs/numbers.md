# The numbers

Five programs written the same way in five languages, timed on one machine
on one day: the median wall time of several runs, in seconds, smaller is
better. Each runs for about a second in the compiled languages, so starting
the process is noise rather than the measurement. What each program
measures, the rules, and how to run them yourself are in
[`bench/`](https://github.com/Londopy/nexium/tree/main/bench); the Bench
workflow regenerates this page on a GitHub runner weekly and at every tag,
and fails when Nexium's time, as a multiple of C's in the same run, grows
by a quarter.

Measured 2026-10-19 on GitHub Actions 1000011680, Linux-6.17.0-1022-azure-x86_64-with-glibc2.39, x86_64; 7 runs each, three for a program over five seconds.

| program | nexium safe | nexium fast | c | rust | go | python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `fib` | 1.086 | 0.734 | 0.389 | 0.734 | 1.345 | 28.948 |
| `nbody` | 0.655 | 0.655 | 0.591 | 0.672 | 0.708 | 49.377 |
| `sieve` | 0.632 | 0.663 | 0.544 | 0.545 | 0.577 | 4.334 |
| `words` | 0.933 | 0.922 | 0.585 | 0.942 | 1.132 | 3.700 |
| `trees` | 0.740 | 0.730 | 0.417 | 0.486 | 0.917 | 17.528 |

The same, as multiples of C's time (1.00 is as fast as C):

| program | nexium safe | nexium fast | rust | go | python |
| --- | ---: | ---: | ---: | ---: | ---: |
| `fib` | 2.79 | 1.88 | 1.89 | 3.46 | 74.38 |
| `nbody` | 1.11 | 1.11 | 1.14 | 1.20 | 83.60 |
| `sieve` | 1.16 | 1.22 | 1.00 | 1.06 | 7.96 |
| `words` | 1.60 | 1.58 | 1.61 | 1.94 | 6.33 |
| `trees` | 1.77 | 1.75 | 1.17 | 2.20 | 42.03 |

Nexium is built twice: `safe` keeps its overflow and bounds checks (what `nx
bench` and `nx ship` build unless told otherwise), `fast` (`--mode fast`) leaves
them out. C is built with `-O2`, Rust with `-O` (which keeps its
bounds checks), Go and Python as they come.

| tool | version |
| --- | --- |
| nexium | nx 1.5.0 (Everest: Base Camp) |
| c | gcc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0 |
| rust | rustc 1.99.1 (58cf8f9c3 2026-10-13) |
| go | go version go1.23.12 linux/amd64 |
| python | Python 3.12.3 |
