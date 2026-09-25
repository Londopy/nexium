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

Measured 2026-09-25 on GitHub Actions 1000009041, Linux-6.17.0-1022-azure-x86_64-with-glibc2.39, x86_64; 7 runs each, three for a program over five seconds.

| program | nexium safe | nexium fast | c | rust | go | python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `fib` | 1.322 | 0.778 | 0.389 | 0.778 | 1.343 | 28.869 |
| `nbody` | 0.656 | 0.655 | 0.590 | 0.677 | 0.708 | 48.540 |
| `sieve` | 0.620 | 0.601 | 0.482 | 0.526 | 0.529 | 4.295 |
| `words` | 1.163 | 1.115 | 0.571 | 0.974 | 1.122 | 3.596 |

The same, as multiples of C's time (1.00 is as fast as C):

| program | nexium safe | nexium fast | rust | go | python |
| --- | ---: | ---: | ---: | ---: | ---: |
| `fib` | 3.40 | 2.00 | 2.00 | 3.45 | 74.21 |
| `nbody` | 1.11 | 1.11 | 1.15 | 1.20 | 82.24 |
| `sieve` | 1.29 | 1.25 | 1.09 | 1.10 | 8.91 |
| `words` | 2.04 | 1.95 | 1.71 | 1.97 | 6.30 |

Nexium is built twice: `safe` keeps its overflow and bounds checks (what `nx
bench` builds, and `nx build --mode safe`), `fast` leaves them out (what `nx ship`
builds for a release). C is built with `-O2`, Rust with `-O` (which keeps its
bounds checks), Go and Python as they come.

| tool | version |
| --- | --- |
| nexium | nx 1.3.1 (Annapurna: Couzy) |
| c | gcc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0 |
| rust | rustc 1.98.1 (48a229cea 2026-09-01) |
| go | go version go1.23.12 linux/amd64 |
| python | Python 3.12.3 |
