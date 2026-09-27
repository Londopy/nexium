# Platforms

Where `nx` runs, where the programs it builds run, and how much each is
tested. Tiers are promises about testing, not about code: the compiler
emits the same C for every target and Zig links it for any target Zig
knows.

## Tier 1: built, tested, released

Every commit runs the whole test harness here; every release ships a
binary.

| target | runs in CI as | notes |
| --- | --- | --- |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | glibc; the install script downloads Zig when no C compiler is found |
| `x86_64-pc-windows-msvc` | `windows-latest` | Windows 10 or later; the installer bundles Zig |
| `aarch64-apple-darwin` | `macos-latest` | Apple Silicon; the system `cc` links (Zig 0.14 cannot read the current Xcode SDK), Zig preprocesses `@cImport` |

## Tier 2: built, released, not tested

Every release cross-compiles these from the same C with Zig and attaches
them. Nothing runs them in CI; a report of a problem is a bug and is
treated like one on tier 1, but it may take a release to be noticed.

| target | how it is built |
| --- | --- |
| `aarch64-unknown-linux-gnu` | `zig cc -target aarch64-linux-gnu` on the Linux runner; the install script picks it on an ARM Linux |
| `aarch64-pc-windows-msvc` | `zig cc -target aarch64-windows-gnu` on the Linux runner; a zip, no installer |

A tier-2 target moves to tier 1 when GitHub offers a runner for it and the
harness passes there.

32-bit targets are tested more than they are released. CI builds and runs
the spec cases, the examples, the standard library's tests and the Topo's
programs for `i686-windows-gnu` and `i686-linux-musl` (a 32-bit program runs
on the 64-bit runners) and for `armv7-linux-musleabihf` and
`riscv32-linux-musl` (under QEMU's user-mode emulation on the Linux runner).
The compiler builds itself as a 32-bit x86 program, and built for ARMv7 and
RISC-V it runs under the emulator and emits the same C as on 64 bits. No
release carries a 32-bit binary yet: `nx build --target i686-linux-gnu` (or
`i686-windows-gnu`, `armv7-linux-gnueabihf`, `riscv32-linux-musl`) makes
one. `i686` is x86 with SSE2, a Pentium 4 or later, as in Rust's `i686`
targets; `armv7` is ARMv7-A with NEON (zig's `arm`), and
`armv6-linux-gnueabihf` is the CPU of the Raspberry Pi 1 and Zero, whose
programs every Pi runs in 32-bit mode.

## Tier 3: should work, unsupported

Anything else Zig can target: `nx build --target <triple>` cross-compiles a
program, and `bootstrap/build.sh` builds the compiler on any host with a C
compiler. Intel macOS is here because Zig 0.14 cannot link against the
current SDK from another host and no Intel runner remains in CI; it builds
from source with the system `cc`. Nothing is promised, and reports are
welcome.

## What the tiers cover

- The compiler itself: `nx build`, `run`, `test`, `check`, the tools, the
  REPL and the language server.
- The programs it builds, on the same target: executables, and the
  libraries `nx ship` produces for C, Python, Rust and Node.
- The GUI (`gui/`) on tier 1 in its headless mode; a window needs a
  display, which CI has not.
- Every binary, the compiler's and the programs', is built for the
  baseline of its architecture (plain x86-64 on x86-64) unless `--cpu`
  says otherwise, so it runs on any machine of that architecture; the
  release workflow refuses an x86-64 build that contains AVX
  instructions.

The C toolchain policy, which Zig versions are tested and what other C
compilers are expected to do, is in [stability.md](stability.md).
