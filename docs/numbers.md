# The numbers

Four small programs written the same way in five languages, timed on one
machine on one day: the median wall time of several runs, in seconds,
smaller is better. What each program measures, the rules, and how to run
them yourself are in [`bench/`](https://github.com/Londopy/nexium/tree/main/bench);
the Bench workflow regenerates this page on a GitHub runner weekly and at
every tag, and fails when Nexium gets a quarter slower than the last run.

Measured 2026-09-20 on MSI, Windows-11-10.0.26200-SP0, Intel64 Family 6 Model 198 Stepping 2, GenuineIntel; 3 runs each.

| program | nexium | c | rust | go | python |
| --- | ---: | ---: | ---: | ---: | ---: |
| `fib` | 0.026 | 0.025 | 0.010 | 0.013 | 0.184 |
| `nbody` | 0.030 | 0.029 | 0.015 | 0.018 | 0.664 |
| `sieve` | 0.051 | 0.045 | 0.033 | 0.034 | 0.340 |
| `words` | 0.402 | 0.117 | 0.216 | 0.153 | 0.505 |

Nexium is built in `fast` mode (no overflow or bounds checks, as `nx ship` builds a
release), C with `-O2`, Rust with `-O`, Go and Python as they come.

| tool | version |
| --- | --- |
| nexium | nx 1.0.3 (Annapurna: Schatz) |
| c | clang version 21.1.0 |
| rust | rustc 1.98.0 (88d9e12ae 2026-08-18) |
| go | go version go1.26.7 windows/amd64 |
| python | Python 3.13.15 |
