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
`fast` mode: `fib` 1.02×, `nbody` 0.96×, `sieve` 1.00×. The gap is `words`,
maps and strings: 2.0× C on Windows and 1.5× on Linux, from 3.0× and 1.9×
at 1.5.0. `safe` mode costs 1.5× on `fib`, the overflow checks on every
call; nothing measurable on the other three.

**Next.** A cache of small freed blocks in the allocator (measured, see
below), then a benchmark that uses `ref class`, so the ARC elision has a
number to move.

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
  already caches them. Awaiting a decision on which to do.

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
