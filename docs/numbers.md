# The numbers

Four programs written the same way in five languages, timed on one machine
on one day: the median wall time of several runs, in seconds, smaller is
better. Each runs for about a second in the compiled languages, so starting
the process is noise rather than the measurement. What each program
measures, the rules, and how to run them yourself are in
[`bench/`](https://github.com/Londopy/nexium/tree/main/bench); the Bench
workflow regenerates this page on a GitHub runner weekly and at every tag,
and fails when Nexium's time, as a multiple of C's in the same run, grows
by a quarter.

Measured 2026-10-04 on GitHub Actions 1000010811, Linux-6.17.0-1022-azure-x86_64-with-glibc2.39, x86_64; 7 runs each, three for a program over five seconds.

| program | nexium safe | nexium fast | c | rust | go | python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `fib` | 1.524 | 0.898 | 0.514 | 0.846 | 1.562 | 39.508 |
| `nbody` | 0.754 | 0.753 | 0.681 | 0.766 | 0.862 | 61.477 |
| `sieve` | 0.644 | 0.599 | 0.436 | 0.435 | 0.477 | 5.529 |
| `words` | 1.480 | 1.438 | 0.720 | 1.161 | 1.445 | 4.478 |

The same, as multiples of C's time (1.00 is as fast as C):

| program | nexium safe | nexium fast | rust | go | python |
| --- | ---: | ---: | ---: | ---: | ---: |
| `fib` | 2.96 | 1.75 | 1.65 | 3.04 | 76.79 |
| `nbody` | 1.11 | 1.11 | 1.13 | 1.27 | 90.33 |
| `sieve` | 1.48 | 1.37 | 1.00 | 1.09 | 12.69 |
| `words` | 2.05 | 2.00 | 1.61 | 2.01 | 6.22 |

Nexium is built twice: `safe` keeps its overflow and bounds checks (what `nx
bench` and `nx ship` build unless told otherwise), `fast` (`--mode fast`) leaves
them out. C is built with `-O2`, Rust with `-O` (which keeps its
bounds checks), Go and Python as they come.

| tool | version |
| --- | --- |
| nexium | nx 1.5.0 (Everest: Base Camp) |
| c | gcc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0 |
| rust | rustc 1.99.0 (b940084d7 2026-09-28) |
| go | go version go1.23.12 linux/amd64 |
| python | Python 3.12.3 |
