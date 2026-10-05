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

Measured 2026-10-05 on GitHub Actions 1000010997, Linux-6.17.0-1022-azure-x86_64-with-glibc2.39, x86_64; 7 runs each, three for a program over five seconds.

| program | nexium safe | nexium fast | c | rust | go | python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `fib` | 1.708 | 1.005 | 0.502 | 0.946 | 1.736 | 37.093 |
| `nbody` | 0.845 | 0.845 | 0.760 | 0.867 | 0.913 | 62.134 |
| `sieve` | 0.660 | 0.633 | 0.535 | 0.546 | 0.595 | 5.468 |
| `words` | 1.249 | 1.229 | 0.741 | 1.254 | 1.462 | 4.593 |

The same, as multiples of C's time (1.00 is as fast as C):

| program | nexium safe | nexium fast | rust | go | python |
| --- | ---: | ---: | ---: | ---: | ---: |
| `fib` | 3.40 | 2.00 | 1.88 | 3.46 | 73.85 |
| `nbody` | 1.11 | 1.11 | 1.14 | 1.20 | 81.70 |
| `sieve` | 1.23 | 1.18 | 1.02 | 1.11 | 10.23 |
| `words` | 1.69 | 1.66 | 1.69 | 1.97 | 6.20 |

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
