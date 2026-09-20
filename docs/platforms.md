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
