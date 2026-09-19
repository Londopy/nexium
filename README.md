<p align="center">
  <img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/banner.svg" alt="Nexium" width="880">
</p>

<p align="center">
  <b>English</b> ·
  <a href="docs/i18n/es/README.md">Español</a> ·
  <a href="docs/i18n/zh-CN/README.md">简体中文</a> ·
  <a href="docs/i18n/ja/README.md">日本語</a> ·
  <a href="docs/i18n/ko/README.md">한국어</a> ·
  <a href="docs/i18n/fr/README.md">Français</a> ·
  <a href="docs/i18n/de/README.md">Deutsch</a>
</p>

<p align="center">
  <a href="https://github.com/Londopy/nexium/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/Londopy/nexium/ci.yml?branch=main&label=CI&logo=githubactions&logoColor=white"></a>
  <a href="https://github.com/Londopy/nexium/releases"><img alt="Release" src="https://img.shields.io/github/v/release/Londopy/nexium?logo=github&color=8b7cf6"></a>
  <a href="https://crates.io/crates/nexium"><img alt="crates.io" src="https://img.shields.io/crates/v/nexium?logo=rust&color=4fd1c5"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue"></a>
  <a href="https://ziglang.org/download/"><img alt="Zig" src="https://img.shields.io/badge/backend-zig%20cc-f7a41d?logo=zig&logoColor=white"></a>
  <img alt="Platforms" src="https://img.shields.io/badge/platforms-windows%20%7C%20linux%20%7C%20macos-2b3a55">
</p>

<p align="center">
  <b>A language complete enough to build everything in, that is also the best thing to adopt for one piece of something else.</b>
</p>

Nexium compiles to native code through C, has automatic reference counting
without a tracing collector, a machine-checked effect system that says whether
a function allocates, blocks, or can panic, and a compiler that turns one
source tree into a C library, a Python wheel, a Rust crate, or a command line
tool.

<table>
<tr>
<td width="50%" valign="top">

**One file**

```
fn checksum(data: []u8) -> u32 export(c) {
    var h: u32 = 2166136261
    for b in data {
        h ^= b as u32
        h *%= 16777619
    }
    return h
}

artifact cabi   { name = "hasher" }
artifact python { name = "hasher" }
```

</td>
<td width="50%" valign="top">

**Every target**

```
$ nx ship hasher.nx
shipped 4 artifact file(s) for x86_64-windows:
  nx-out/hasher/hasher.dll
  nx-out/hasher/hasher.lib
  nx-out/hasher/hasher.h
  nx-out/hasher/hasher-0.1.0-py3-none-win_amd64.whl
```

```python
>>> import hasher
>>> hasher.checksum(b"hello")
1335831723
```

</td>
</tr>
</table>

The effect signature decides the C ABI: `checksum` is proven `!panics`, so it
gets a plain `uint32_t checksum(const uint8_t*, size_t)`. A function that can
fail returns a status code, and a Nexium panic inside it is converted at the
boundary rather than aborting the host process.

## Highlights

| | |
| --- | --- |
| 🧾 **Effects, inferred and checked** | `allocates` `refcounts` `blocks` `shared_mutable` `nondeterministic` `panics` `ffi`. Declare `!allocates` and the compiler points at the exact line that would break it, through calls. |
| 🧠 **Ownership without a borrow checker** | Collections move, `.clone()` copies, `ref class` values are reference counted, `weak` breaks cycles. Use after move is a compile error. |
| 🔬 **Binary patterns** | `<<version:4, ihl:4, len:16/big, rest:bytes>>` matches and builds packets with checked sizes. |
| 🧵 **Parallel loops, arenas, trait objects** | `for parallel`, `using arena { }`, `dyn Trait !allocates`. |
| 🔌 **C without bindings** | `@cImport("header.h")` reads the header directly; `artifact link` compiles vendored C into the program. |
| 📦 **Ship from one source** | `nx ship` produces C headers and libraries, Python wheels, and Rust crates with safe wrappers. |
| 🖼 **A GUI, in Nexium** | [`gui/`](gui): an immediate-mode GUI (buttons, sliders, text fields) with a software rasterizer and bitmap font, all Nexium over a 200-line C window layer. |
| 🛠 **Tooling in the box** | `fmt`, `doc`, `lsp`, `size`, `leaks`, `refcounts`, `effects`, `audit`. Zero dependencies. |

## Install

**Windows**: download and run the installer from the
[Releases](https://github.com/Londopy/nexium/releases) page. It installs
`nx`, a bundled Zig toolchain (the C compiler `nx` uses), the standard
library, examples, docs, and the VS Code extension, and adds `nx` to your
PATH. Nothing else to install.

**macOS and Linux**:

```bash
curl -fsSL https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.sh | sh
```

It verifies the download against the release checksums, installs to
`~/.nexium`, sets up a C compiler (the Xcode tools on macOS; Zig is downloaded
on Linux when nothing is found), and adds `nx` to your PATH.

Then, in a new console, `nx doctor` shows what will be used. All the details,
including verifying checksums and every environment variable, are in
[docs/install.md](docs/install.md).

Or build from source with Rust 1.75 or newer (you provide the C compiler:
Zig on the PATH, or `NX_CC`):

```bash
cargo install nexium
```

```bash
cargo install --git https://github.com/Londopy/nexium
```

Then:

```bash
nx run examples/hello.nx
```

## A tour

```
struct Point derive(Eq) { x: f64, y: f64 }

enum Shape { Circle(f64), Rect { w: f64, h: f64 }, Empty }

fn area(s: Shape) -> f64 {
    match s {
        .Circle(r) => math.PI * r * r,
        .Rect(w, h) => w * h,
        .Empty => 0.0,
    }
}

error ParseError { Empty, NotANumber }

fn parse_num(text: []u8) -> ParseError!i64 {
    if text.len == 0 { return error.Empty }
    var total: i64 = 0
    for c in text {
        if c < '0' or c > '9' { return error.NotANumber }
        total = total * 10 + (c - '0') as i64
    }
    return total
}

fn max(comptime T: type where T: Ord, a: T, b: T) -> T {
    return if a > b { a } else { b }
}

fn main() -> !void {
    let n = try parse_num("1234")
    let bad = parse_num("12x") catch |e| {
        println("caught {}", .{e})
        -1
    }
    var xs = List(i32).new()
    for i in 0..10 { xs.append((i * i) as i32) }
    let found = outer: {
        for x, i in xs { if x > 30 { break :outer i as i64 } }
        -1
    }
    println("{} {} {} {} {}", .{n, bad, max(3, 9), xs.len, found})
}
```

<details>
<summary><b>Binary pattern matching</b></summary>

```
fn parse_ipv4(packet: []u8) -> Net!Ipv4 {
    match packet {
        <<version:4, ihl:4, dscp:6, ecn:2, total_len:16/big,
          id:16, flags:3, frag_off:13, ttl:8, proto:8,
          checksum:16, src:32, dst:32, rest:bytes>> => {
            return Ipv4{ .version = version, .ihl = ihl, .total_len = total_len,
                         .ttl = ttl, .proto = proto, .src = src, .dst = dst }
        }
        _ => return error.Truncated,
    }
}

let written = try <<4:4, 5:4, 0:8, 1500:16/big, "ab">> into buf[..]
```

</details>

<details>
<summary><b>Effects are inferred and checked</b></summary>

```
fn hot(xs: []i32) -> i32 !allocates !panics {
    var list = List(i32).new()
    list.append(1)
    return helper(xs) + list.len as i32
}
```

```
error: function `hot` is declared `!allocates` but has the `allocates` effect
  --> examples/effects_bad.nx:7:26
  note: the effect is introduced here: appending to a List may grow it
  --> examples/effects_bad.nx:9:5
```

</details>

<details>
<summary><b>Calling C is a header import away</b></summary>

```
const libc = @cImport("string.h")
const cv = @cImport("cvendor.h")
artifact link { c_sources = ["cvendor.c"] }

unsafe { println("{}", .{libc.strlen(@cstr("hello"))}) }
```

No binding generator, no build step: the header is the single source of truth
for layout, and foreign calls carry the `ffi` effect.

</details>

<details>
<summary><b>Parallel loops and arenas</b></summary>

```
for parallel p, i in positions {
    out[i] = integrate(p)          // no shared_mutable allowed in here
}

using arena {
    var scratch = List(Frame).new()   // bump allocated, freed all at once
    ...
}
```

</details>

**Documentation**

- [Specification](SPEC.md): the language as implemented, with planned parts marked.
- [Roadmap](ROADMAP.md): phases, exit criteria, and what is not planned.
- [How Nexium works](docs/architecture.md): the pipeline from source to binary, effects inference, ownership, the runtime, and shipping.
- [Language reference](docs/language.md): every construct the compiler implements.
- [Embedding](docs/embedding.md): calling shipped libraries from Python, Rust, and C.
- [The interactive session](docs/repl.md): `nx` at a prompt, like `python`.
- [Installing](docs/install.md): the Windows installer, the macOS/Linux script, source builds, checksums, and how `nx` finds a C compiler.
- [Packages](docs/packages.md): `nexium.toml`, `nx add`, `nx fetch`, git or path dependencies, the lock file.
- [Standard library](docs/std.md): the modules written in Nexium (`std.strings`, `std.lists`, `std.bytes`, `std.num`, `std.json`, `std.args`, `std.fs`, `std.time`, `std.regex`, `std.text`, `std.testing`, `std.stream`, `std.net`, `std.http`, `std.thread`, `std.process`).
- [nexium-gui](docs/gui.md): the immediate-mode GUI library and how to write a widget.
- [Releasing your program](docs/releasing-your-program.md): binaries for three platforms from a tag, installers optional.
- [Editor support](editors): VS Code extension, Sublime syntax, LSP.
- [Linguist](linguist): the ready-to-apply pull request that will make GitHub recognize `.nx` once the usage bar is met.
- [Translations](docs/i18n): this README in six languages; the language reference and the architecture tour in Spanish, Chinese, and Japanese.
- [Decisions](DECISIONS.md): every call made where the specification was open.

## Commands

| command | what it does |
| --- | --- |
| `nx build file.nx` | compile to an executable (or object when there is no `main`) |
| `nx run file.nx` | build and run |
| `nx test file.nx [filter]` | run the `test "..."` blocks |
| `nx check file.nx` | type-check and report effect violations |
| `nx effects file.nx` | print the inferred effects of every function |
| `nx audit file.nx` | list `unsafe` blocks and mutable globals |
| `nx ship file.nx` | produce every declared `artifact` |
| `nx emit-c file.nx` | print the generated C |
| `nx tokens file.nx` | dump the token stream (the self-hosting oracle) |
| `nx fmt file.nx [--check]` | canonical formatting |
| `nx doc file.nx` | HTML documentation with inferred effects |
| `nx size file.nx` | attribute binary bytes to declarations |
| `nx refcounts file.nx` | every retain and release site |
| `nx leaks file.nx` | run with allocation tracking and report leaks |
| `nx lsp` | language server over stdio |
| `nx doctor` | which C compiler will be used, and whether the installation works |
| `nx repl`, or just `nx` | an interactive session: type code, see values, keep bindings |

Options: `--mode debug|safe|fast|small`, `--target x86_64-linux-gnu` (any
target `zig cc` knows), `--out-dir`, `--keep-c`, `--cc`, and for C interop
`-I`, `--link`, `--link-path`, `--c-source`.

## Status

This is the first implementation of the design in `nexium-spec.txt`. It is
complete enough to write real programs (see [`examples/`](examples)) and to
ship a Python, Rust, or C component from one file. Trait objects, parallel
loops, arena scopes, direct C header import, and the tooling are all in. Still
early: the standard library is a fraction of section 16, and region checking
covers only returned views. [`DECISIONS.md`](DECISIONS.md) lists every call
made where the specification was open, for review, and item 27 lists what is
left.

## Self-hosting

The compiler is Rust today. The Nexium version of it grows under
[`self/`](self), one stage at a time, each checked against the Rust compiler
on the same inputs:

| stage | file | oracle | status |
| --- | --- | --- | --- |
| lexer | [`self/lexer.nx`](self/lexer.nx) | `nx tokens` | ✅ matches on every example and on itself |
| parser | | `nx parse` | next |
| checker | | `nx check`, the compile-fail suite | |
| C emitter | | `nx emit-c` | |

`cargo test` builds `self/lexer.nx` with the Rust compiler and diffs its output
against the oracle.

## Languages in the repository

Non-blank lines of code, excluding build output, dependencies, and generated
files (`gui/font.bin`, lock files):

| language | lines | share | what it is |
| --- | --- | --- | --- |
| Rust | 22,788 | 82.1% | the `nx` compiler |
| Nexium | 3,690 | 13.3% | the standard library, examples, the self-hosted lexer, nexium-gui, tests |
| C | 1,127 | 4.1% | the runtime `nx_rt.h` and the GUI window layer |
| JavaScript, TypeScript | 159 | 0.6% | the VS Code extension |

The Nexium share grows with every self-hosting stage; the Rust share is the
bootstrap compiler and will one day be zero.

## Layout

```
src/            the compiler (lexer, parser, checker, comptime, C backend, driver)
runtime/        nx_rt.h, embedded into every generated C file
std/            the standard library in Nexium, embedded in the compiler
self/           the compiler in Nexium, stage by stage
gui/            nexium-gui: immediate-mode GUI in Nexium, demo, and the C platform layer
editors/        VS Code extension and Sublime Text syntax
examples/       programs with recorded output, run by `cargo test`
tests/          integration tests and compile-fail cases
docs/           how it works, language reference, embedding guide, i18n/ translations
assets/         logo and banner
nexium-spec.txt          the design
nexium-systems-spec.txt  the archived systems language; sections 4 to 9 are the syntax reference
DECISIONS.md    decisions made where the specification was open
```

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Bugs and proposals go through GitHub
issues; a language change must name the hard constraint in section 3 of the
specification that it serves.

## License

MIT. Copyright (c) 2026 Londopy.

<p align="center"><img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/logo.svg" alt="" width="48"></p>
