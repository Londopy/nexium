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

Measured 2026-09-28 on GitHub Actions 1000010261, Linux-6.17.0-1022-azure-x86_64-with-glibc2.39, x86_64; 7 runs each, three for a program over five seconds.

| program | nexium safe | nexium fast | c | rust | go | python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `fib` | 1.524 | 0.899 | 0.515 | 0.898 | 1.562 | 39.577 |
| `nbody` | 0.754 | 0.755 | 0.681 | 0.776 | 0.864 | 62.056 |
| `sieve` | 0.673 | 0.627 | 0.464 | 0.509 | 0.496 | 5.519 |
| `words` | 1.460 | 1.448 | 0.729 | 1.190 | 1.494 | 4.572 |

The same, as multiples of C's time (1.00 is as fast as C):

| program | nexium safe | nexium fast | rust | go | python |
| --- | ---: | ---: | ---: | ---: | ---: |
| `fib` | 2.96 | 1.74 | 1.74 | 3.03 | 76.80 |
| `nbody` | 1.11 | 1.11 | 1.14 | 1.27 | 91.14 |
| `sieve` | 1.45 | 1.35 | 1.10 | 1.07 | 11.90 |
| `words` | 2.00 | 1.99 | 1.63 | 2.05 | 6.27 |

Nexium is built twice: `safe` keeps its overflow and bounds checks (what `nx
bench` and `nx ship` build unless told otherwise), `fast` (`--mode fast`) leaves
them out. C is built with `-O2`, Rust with `-O` (which keeps its
bounds checks), Go and Python as they come.

| tool | version |
| --- | --- |
| nexium | nx 1.4.1 (Annapurna: Ang Tharkay) |
| c | gcc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0 |
| rust | rustc 1.98.1 (48a229cea 2026-09-01) |
| go | go version go1.23.12 linux/amd64 |
| python | Python 3.12.3 |
