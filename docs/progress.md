# Progress

What is being worked on, in detail: by theme, newest first, each entry
dated, with its commit, what changed, what it measured and what is still
open. The [roadmap](../ROADMAP.md) says what is planned and the
[changelog](../CHANGELOG.md) what shipped; this is the work in between.
Numbers are from one machine on one day unless they name the numbers page.

## 1.6: the runtime that release builds deserve

Everest: Khumbu Icefall. Under way on `main` since 1.5.0.

**Where it stands.** With the same C compiler on both sides (zig cc on
Windows, gcc on Linux), three of the four benchmarks run at C's speed in
`fast` mode: `fib` 0.99×, `nbody` 0.87×, `sieve` 0.93×. The gap is `words`,
maps and strings: 1.6× C on Windows (0.66 s against 0.42), and on Linux
under WSL between 1.6× and 1.8× depending on the run (0.82 s against 0.45
on the last one), from 3.0× and 1.9× at 1.5.0. `safe` mode costs 1.5× on
`fib`, the overflow checks on every call; nothing measurable on the other
three. The fifth benchmark, `trees` (binary trees of `ref class` nodes),
stands at 1.3× C in both modes on Windows: the ARC elision item's number,
about a tenth of it the retain and release of each child `count` binds.

**Next.** The ARC elision: the retain and release pairs the checker can
prove redundant, which `trees` makes twice per inner node in `count`.

### 2026-10-09, the `trees` benchmark (6e7f407)

- A fifth benchmark for the numbers page, binary trees of `ref class`
  nodes (decision 127): for every second depth from 4 to 16, 2^(21−depth)
  trees built, counted and dropped, a stretch tree of depth 18 first and a
  long-lived one of depth 17 kept; 174,754 trees, 29,971,806 nodes;
  `bench/trees.{nx,c,rs,go,py}`, the same line from all five.
- Measured on Windows with `bench/run.py` (zig cc 0.14.1 for Nexium and C,
  rustc 1.98, go 1.26, Python 3.13; medians of five runs): Nexium safe
  1.07 s, fast 1.08, C 0.85, Rust 0.94, Go 0.50 (its collector against a
  `malloc` and a `free` per node), Python 13.3. Nexium at 1.3× C in both
  modes. On Linux under WSL with gcc, where `malloc` is quicker, C takes
  0.42 s and Nexium `fast` 0.55 (1.3× again).
- `nx leaks bench/trees.nx`: none, 29,971,806 allocations, peak 25,165,776
  bytes: the stretch tree alone, 524,287 nodes of 48 bytes (two words of
  counts and two 16-byte optionals, where C's node is 16 bytes), dropped
  before the long-lived tree is made. `nx refcounts bench/trees.nx`: eleven
  sites, four of them in `count`, the retain of each child bound with `let`
  and its release at the scope's exit, twice per inner node (the two on the
  leaf literal's `null`s are a report quirk, in `KNOWN_ISSUES.md`); `make`
  moves its fresh children into the node and retains nothing. Those pairs
  are the ARC elision's target and this is its before: bound with `if let`,
  which borrows, the program ran at 1.26× C against 1.39× in one alternating
  pass, so the elision is worth about a tenth here.
- The Bench workflow's next run adds the row (a program the baseline lacks
  is not compared, `bench/run.py` line 158); `docs/numbers.md`,
  `bench/results/latest.json` and the README's dated Speed table follow
  from it. The README and its six translations count five benchmarks.

### 2026-10-08, small blocks cached in the allocator (7203e50)

- Freed blocks of up to 64 bytes and a little over are kept, in four
  classes 16 bytes apart, up to 64 per class, per thread, and reused before
  `malloc` is asked (decision 126); a thread the runtime started, and an
  exported call, return their blocks when they end. The classes end where
  the C library's steps do: measured on 64-bit Windows and glibc, a block
  of 24, 40, 56 or 72 bytes costs their allocators what one of 16, 32, 48
  or 64 does, so the classes end there and a cached block costs no more
  than its request would; elsewhere they end at the multiples of 16.
  Small-string optimization is dropped, the reason in the roadmap.
- Measured, the same C compiler on both sides, medians of nine alternating
  runs: `words` 0.87 s → 0.66 s on Windows with zig cc (2.1× C → 1.6×, C
  at 0.42 s); Linux with gcc under WSL unchanged, 0.82 s before and 0.82
  after in one pass (C at 0.45 s; WSL's figures move between passes, 0.70 s
  against 0.43 in an earlier one, so only a pass's own before and after
  compare). On Windows the compiler emits its own C in 0.58 s where it took
  0.70. `fib`, `nbody` and `sieve`, which allocate nothing in their loops,
  stay at C's speed (0.99×, 0.87×, 0.93×).
- Tests: the spec case `s5_small_blocks`, compiled, and under `nx leaks`
  with the old runtime's counts to the allocation. CI's sanitizer job
  builds with the cache compiled out, so the cache itself was run under
  gcc's AddressSanitizer and UBSan in WSL with `-DNX_SMALL_CACHE=1`, over
  all 80 spec cases, locally only; the case also runs clean there with the
  cache off, and under UBSan with it on.
- Found on the way, logged in `KNOWN_ISSUES.md`: `nx leaks` miscounts what
  a thread allocated and the joiner releases (`s13_concurrency` shows it),
  and a debug build does not fill a released container buffer with `0xDD`.
  Fixed beside it: `bootstrap\build.ps1`'s stage 2, broken since the script
  was written.

### 2026-10-04, `Map.get_or_put` (61ce3be)

- `m.get_or_put(key, default)`: the value of a key as a pointer, put in
  with the default when the key is new, in one lookup (decision 125). The
  `words` benchmark counts with it: `counts.get_or_put(word, 0).* += 1`.
- Measured: `words` 1.16 s → 0.77 s on Windows (3.0× C → 2.0×); on Linux
  with gcc 0.64 s → 0.65 s, no change (glibc's allocator was not the cost
  there).
- Two bugs it exposed and fixed: a compound assignment's target was
  checked twice and the interpreter evaluated it twice (`slot(&mut xs,
  i).* += 1` called `slot` twice under `nx play`); a view bound by `let`
  counted as taken at the `let` rather than where its value was made.
- Open: small-string optimization, the roadmap's next item for `String`,
  does not fit the language: a `String` is passed by copy and a slice into
  it may outlive the call, which is fine while the bytes are on the heap
  and shared, and wrong once they live inside the copy. A cache of small
  freed blocks in the allocator was measured instead: `words` 0.78 s →
  0.59 s on Windows (2.0× C → 1.5×), nothing on Linux, whose C library
  already caches them. Decided 2026-10-08: the cache, as the roadmap's
  next item; small-string optimization is dropped. Landed, the entry above.

### 2026-10-04, faster short strings (694b36c)

- An integer that fits in 64 bits is written with 64-bit arithmetic and a
  constant base of 10; every digit was a 128-bit division, a library call.
- A string's first block holds 16 bytes, where it held 4 and the next
  append grew it at once: `format("w{}", .{k})` allocates once, not twice.
- Measured: `words` 2.26 s → 1.39 s on Windows with zig cc, 0.91 s → 0.73 s
  on Linux with gcc; C takes 0.48 s on both.
- A spec case for the boundaries of integer formatting
  (`s4_int_format_edges`): the largest `u64` and the smallest `i64` by the
  64-bit path, 2⁶⁴ and past it by the 128-bit one, every base.

### 2026-10-04, a known issue logged (1826e50)

- Zig 0.16 fails inside clang on every `wasm32-wasi` build ("undefined tag
  symbol cannot be weak", compiling wasi-libc's own `setjmp` runtime).
  Native builds work with 0.16; every build works with the pinned 0.14.1.
  In `KNOWN_ISSUES.md`, not fixed.

## Outside the roadmap

### 2026-10-07, the mark (bcccdba, 6d99635, 8089dd5)

- A new mark: Fuji over a lake, the lake reflecting the mountain as `nx`;
  indigo on washi by day, the same scene by night under dark mode. One
  script, `scripts/make_assets.py`, draws every asset from it: the logo,
  the README's light and dark banners, the social preview, the icon at
  every size, the VS Code icon, the installer's wizard images, a version
  composed for a circle (Discord), and a small version with no letters for
  browser tabs and title bars.
- The site and the Topo take the mark's palette, day and night.

### 2026-10-04, the docs brought up to 1.5 (01b08be, 186f3b5, a58a805)

- The README and its six translations describe 1.5 (WebAssembly, the
  32-bit downloads, ARM, CI on five machines), with the line counts
  recounted; the Topo's chapters 15, 16, 18 and 23 no longer promise what
  shipped or describe the compiler as it was at 1.0; `docs/discord.md`
  and `docs/packages.md` name nexium-discord 0.2.0 and nxtls 0.5.0.
- A private project is no longer named in the repository.

## 1.5: platforms

Everest: Base Camp. Shipped as 1.5.0 on 2026-10-04; the
[changelog](../CHANGELOG.md#150---2026-10-04) has the whole of it, the
[roadmap](../ROADMAP.md#15-platforms) what moved to 1.6, 1.7 and 1.8.

### 2026-10-04, the 32-bit downloads (df478c3) and the release

- Every release carries `nx` for 32-bit Windows and Linux and for ARMv7
  Linux, static on Linux; the install scripts pick them by the system
  rather than the CPU and build from source on a CPU without SSE2 or NEON;
  Scoop and the AUR list them. The release workflow's new step was run
  locally, as written, before the tag.
- Released the same day; the registries (PyPI, npm, Open VSX, Chocolatey,
  Docker) published; the Scoop bucket and the winget request moved to
  1.5.0.
