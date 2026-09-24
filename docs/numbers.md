# The numbers

Four small programs written the same way in five languages, timed on one
machine on one day: the median wall time of several runs, in seconds,
smaller is better. What each program measures, the rules, and how to run
them yourself are in [`bench/`](https://github.com/Londopy/nexium/tree/main/bench);
the Bench workflow regenerates this page on a GitHub runner weekly and at
every tag, and fails when Nexium gets a quarter slower than the last run.

Measured 2026-09-24 on GitHub Actions 1000008533, Linux-6.17.0-1022-azure-x86_64-with-glibc2.39, x86_64; 7 runs each.

| program | nexium | c | rust | go | python |
| --- | ---: | ---: | ---: | ---: | ---: |
| `fib` | 0.005 | 0.003 | 0.005 | 0.009 | 0.169 |
| `nbody` | 0.010 | 0.009 | 0.010 | 0.013 | 0.616 |
| `sieve` | 0.020 | 0.015 | 0.015 | 0.017 | 0.301 |
| `words` | 0.161 | 0.086 | 0.135 | 0.159 | 0.471 |

Nexium is built in `fast` mode (no overflow or bounds checks, as `nx ship` builds a
release), C with `-O2`, Rust with `-O`, Go and Python as they come.

| tool | version |
| --- | --- |
| nexium | nx 1.3.0 (Annapurna: North Face) |
| c | gcc (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0 |
| rust | rustc 1.98.1 (48a229cea 2026-09-01) |
| go | go version go1.23.12 linux/amd64 |
| python | Python 3.12.3 |
