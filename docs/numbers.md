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

Measured 2026-11-02 on GitHub Actions 1000012410, Linux-7.0.0-1017-azure-x86_64-with-glibc2.39, x86_64; 7 runs each, three for a program over five seconds.

| program | nexium safe | nexium fast | c | rust | go | python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `fib` | 1.391 | 0.849 | 0.515 | 0.847 | 1.563 | 39.278 |
| `nbody` | 0.754 | 0.754 | 0.682 | 0.768 | 0.863 | 61.584 |
| `sieve` | 0.701 | 0.737 | 0.561 | 0.541 | 0.572 | 5.762 |
| `words` | 0.991 | 0.987 | 0.737 | 1.158 | 1.483 | 4.478 |
| `trees` | 0.893 | 0.857 | 0.607 | 0.620 | 1.216 | 14.981 |

The same, as multiples of C's time (1.00 is as fast as C):

| program | nexium safe | nexium fast | rust | go | python |
| --- | ---: | ---: | ---: | ---: | ---: |
| `fib` | 2.70 | 1.65 | 1.64 | 3.03 | 76.19 |
| `nbody` | 1.11 | 1.11 | 1.13 | 1.27 | 90.35 |
| `sieve` | 1.25 | 1.31 | 0.96 | 1.02 | 10.28 |
| `words` | 1.34 | 1.34 | 1.57 | 2.01 | 6.07 |
| `trees` | 1.47 | 1.41 | 1.02 | 2.00 | 24.70 |

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
