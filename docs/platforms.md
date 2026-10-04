# Platforms

Where `nx` runs, where the programs it builds run, and how much each is
tested. Tiers are promises about testing, not about code: the compiler
emits C that differs between targets only where the program asks about
the target (`@target()`, `@sizeOf`), and Zig links it for any target Zig
knows.

## Tier 1: built, tested, released

Every commit runs the whole test harness here; every release ships a
binary.

| target | runs in CI as | notes |
| --- | --- | --- |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | glibc; the install script downloads Zig when no C compiler is found |
| `x86_64-pc-windows-msvc` | `windows-latest` | Windows 10 or later; the installer bundles Zig |
| `aarch64-apple-darwin` | `macos-latest` | Apple Silicon; the system `cc` links (Zig 0.14 cannot read the current Xcode SDK), Zig preprocesses `@cImport` |
| `aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` | glibc; the install script picks it on an ARM Linux, a 64-bit Raspberry Pi OS among them |
| `aarch64-pc-windows-msvc` | `windows-11-arm` | Windows 11 on ARM; a zip, no installer |

The release builds the two ARM binaries on its Linux runner, from the same
C with Zig (`aarch64-linux-gnu`, `aarch64-windows-gnu`); CI runs the harness
on ARM machines. They were tier 2 until GitHub's arm64 runners (1.5).

## Tier 2: built, released, tested in part

Every release ships a binary, and CI runs, for every commit, what the
compiler builds for the target: the spec cases, the examples, the standard
library's tests, the Topo's programs and the compile-fail cases. It also
builds the compiler for the target, which must emit the same C as the
64-bit one and, natively, build itself. The rest of the harness, the
tools' suites, runs on tier 1 only; a tier-2 target moves up when a runner
runs the whole of it.

| target | runs in CI as | notes |
| --- | --- | --- |
| `i686-pc-windows-msvc` | 32-bit programs on `windows-latest` | a zip, no installer; Scoop's `32bit`, and the PowerShell one-liner picks it on 32-bit Windows |
| `i686-unknown-linux-musl` | 32-bit programs on `ubuntu-latest` | static, so any distribution; the install script picks it on a 32-bit x86 system |
| `armv7-unknown-linux-musleabihf` | QEMU's user mode on `ubuntu-latest` | static; a Raspberry Pi 2 or later (or a Zero 2) on a 32-bit OS, which the install script picks it for |

`i686` is x86 with SSE2, a Pentium 4 or later, as in Rust's `i686`
targets; `armv7` is ARMv7-A with NEON (zig's `arm`). The install script
builds from source on a CPU without them. The release builds the three on
its Linux runner, from C emitted for each target (the C asserts every
`@sizeOf` at the target's width), with Zig; the Linux ones link musl
statically.

## Tested, not yet released

`riscv32-linux-musl` is tested as tier 2 is, under QEMU on the Linux
runner, and no release carries a binary for it: `nx build --target
riscv32-linux-musl` makes one. Programs also build for the 32-bit targets
with glibc (`i686-linux-gnu`, `armv7-linux-gnueabihf`) and for the CPU of
the Raspberry Pi 1 and Zero (`armv6-linux-gnueabihf`, whose programs every
Pi runs in 32-bit mode), which CI does not run.

WebAssembly is tested the same way. `nx build --target wasm32-wasi` makes
a `.wasm` module, and `nx run` and `nx test` run it under Node.js's WASI
(Node.js on the PATH): the program sees its working directory and a `/tmp`
of its own beside the module, and nothing else of the machine. CI runs the
spec cases, the examples, the standard library's tests and the Topo's
programs as modules under Node.js on Linux, and loads a library built with
`artifact wasm` into a page in headless Chrome. WASI (preview 1) has no
processes, sockets or threads: `std.process` and `std.net` fail as a system
that refuses them would, `for parallel` and `std.thread` run their work in
place, and a program's stack is the engine's (`artifact cli { stack }` has
no thread to give it). A panic reaches its boundary through the legacy form
of WebAssembly's exception-handling proposal, which every major browser and
Node.js run; a WASI runtime without it refuses the module. On Windows,
Node.js's WASI cannot list a directory (`fd_readdir` is not implemented
there), so `fs.list` fails; on Linux and macOS it works.
`wasm32-freestanding`, with no WASI at all, waits for the embedded targets:
the runtime needs a C library.

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
