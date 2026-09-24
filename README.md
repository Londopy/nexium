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
  <a href="https://londopy.github.io/nexium/"><img alt="Docs" src="https://img.shields.io/badge/docs-londopy.github.io%2Fnexium-5b4bd6"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue"></a>
  <a href="https://ziglang.org/download/"><img alt="Zig" src="https://img.shields.io/badge/backend-zig%20cc-f7a41d?logo=zig&logoColor=white"></a>
  <img alt="Platforms" src="https://img.shields.io/badge/platforms-windows%20%7C%20linux%20%7C%20macos-2b3a55">
</p>

<p align="center">
  <a href="https://pypi.org/project/nexium-lang/"><img alt="PyPI" src="https://img.shields.io/pypi/v/nexium-lang?logo=pypi&logoColor=white&label=pip"></a>
  <a href="https://www.npmjs.com/package/nexium-lang"><img alt="npm" src="https://img.shields.io/npm/v/nexium-lang?logo=npm&logoColor=white&label=npm"></a>
  <a href="https://open-vsx.org/extension/Londopy/nexium"><img alt="Open VSX" src="https://img.shields.io/open-vsx/v/Londopy/nexium?label=Open%20VSX&color=a60ee5"></a>
  <a href="https://community.chocolatey.org/packages/nexium"><img alt="Chocolatey" src="https://img.shields.io/chocolatey/v/nexium?label=choco&color=80b5e3"></a>
  <a href="https://github.com/Londopy/nexium/pkgs/container/nexium"><img alt="Docker" src="https://img.shields.io/badge/docker-ghcr.io%2Flondopy%2Fnexium-2496ed?logo=docker&logoColor=white"></a>
  <a href="https://codespaces.new/Londopy/nexium"><img alt="Open in GitHub Codespaces" src="https://img.shields.io/badge/codespaces-open-24292f?logo=github"></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/Londopy/nexium"><img alt="OpenSSF Scorecard" src="https://api.scorecard.dev/projects/github.com/Londopy/nexium/badge"></a>
</p>

<p align="center">
  <b>Nexium is a language complete enough to build everything in, that is also the best thing to adopt for one piece of something else.</b>
</p>

<p align="center">
  <a href="https://londopy.github.io/nexium/"><b>Documentation &amp; the Topo tutorial &rarr; londopy.github.io/nexium</b></a><br>
  <sub><a href="https://londopy.github.io/nexium/topo/01-base-camp.html">Start with the Topo</a> &middot; <a href="https://londopy.github.io/nexium/docs/language.html">Language reference</a> &middot; <a href="https://londopy.github.io/nexium/docs/install.html">Install</a> &middot; <a href="https://londopy.github.io/nexium/docs/std.html">Standard library</a> &middot; <a href="https://londopy.github.io/nexium/docs/embedding.html">Embedding</a></sub>
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

**Windows, from PowerShell**: `irm https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.ps1 | iex`
(the portable build, verified, on the PATH; no wizard).

**pip or npm**: `pip install nexium-lang` ([PyPI](https://pypi.org/project/nexium-lang/)) or `npm install -g nexium-lang` ([npm](https://www.npmjs.com/package/nexium-lang)); the binary, per platform; a C compiler as usual.

**Docker**: `docker run --rm -v "$PWD":/work ghcr.io/londopy/nexium run hello.nx`
(Debian; `:alpine` too; amd64 and arm64).

**Chocolatey and winget**: `choco install nexium` ([the package](https://community.chocolatey.org/packages/nexium)) and `winget install Londopy.Nexium`, each once its registry has approved the first version ([the status](docs/install.md#where-to-get-it)).

**Debian, RPM, Nix, mise**: every release attaches `.deb` and `.rpm` packages (`sudo dpkg -i nexium_1.2.1_amd64.deb`); `nix run github:Londopy/nexium` builds it from the one C file; `mise use -g "ubi:Londopy/nexium[exe=nx]"` installs the release binary. Every asset carries signed provenance: `gh attestation verify nx --owner Londopy`. [All the roads](docs/install.md#where-to-get-it).

**In the browser**: [open the repository in a Codespace](https://codespaces.new/Londopy/nexium) and `nx run examples/hello.nx` runs in a minute, nothing installed.

**Homebrew and Scoop**: the repository is its own tap and bucket.

```bash
brew tap londopy/tap https://github.com/Londopy/nexium && brew install londopy/tap/nexium
```

```powershell
scoop bucket add londopy https://github.com/Londopy/scoop-bucket && scoop install nexium
```

Then, in a new console, `nx doctor` shows what will be used. All the details,
including verifying checksums and every environment variable, are in
[docs/install.md](docs/install.md).

Or build from source with nothing but a C compiler (Zig on the PATH, or
`CC`), which builds the compiler written in Nexium from its C seed:

```bash
git clone https://github.com/Londopy/nexium && cd nexium && sh bootstrap/build.sh
```

The result is `nx-out/bootstrap/nx2` (`build.ps1` on Windows). Then:

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
- [Stability](docs/stability.md) and [platforms](docs/platforms.md): what a version promises, the deprecation cycle, `nx fix`, the tiers.
- [The Topo](https://londopy.github.io/nexium/topo/01-base-camp.html): the tutorial, from installing the compiler to a neural network, a GUI and a shipped library, with exercises the compiler grades (on the page, or `nx topo` in the terminal); the source is [`topo/`](topo/). All of the above, rendered, is at [londopy.github.io/nexium](https://londopy.github.io/nexium/).
- [Installing](docs/install.md): the Windows installer, the macOS/Linux script, source builds, checksums, and how `nx` finds a C compiler.
- [Packages](docs/packages.md): `nexium.toml`, `nx add`, `nx fetch`, git or path dependencies, the lock file.
- [Standard library](docs/std.md): the modules written in Nexium (`std.strings`, `std.lists`, `std.bytes`, `std.num`, `std.json`, `std.args`, `std.fs`, `std.time`, `std.regex`, `std.text`, `std.testing`, `std.stream`, `std.net`, `std.http`, `std.thread`, `std.process`).
- [nexium-gui](docs/gui.md): the immediate-mode GUI library and how to write a widget.
- [Releasing your program](docs/releasing-your-program.md): binaries for three platforms from a tag, installers optional.
- [Editor support](editors): VS Code, Vim, Neovim, Helix, Zed, Emacs, Kate, JetBrains, Sublime Text, Notepad++, nano, and `nx lsp` for the rest.
- [Linguist](linguist): the ready-to-apply pull request that will make GitHub recognize `.nx` once the usage bar is met.
- [Translations](docs/i18n): this README in six languages; the language reference and the architecture tour in Spanish, Chinese, and Japanese.
- [Release names](docs/release-names.md): every release is a place on a mountain; the scheme, the ledger, and the names still to use.
- [Decisions](DECISIONS.md): every call made where the specification was open.
- [Known issues](KNOWN_ISSUES.md): open bugs, gaps and limitations, with repros.

## In the wild

Programs and packages outside this repository that are written in Nexium:

| project | what Nexium does there |
| --- | --- |
| [statusmith](https://github.com/Londopy/statusmith), Discord Rich Presence from the tray | its SDK is a Nexium package: `nx add discord_rpc --git https://github.com/Londopy/statusmith --tag sdk-v0.1.0 --dir nexium` sets a presence from any Nexium program ([the page](docs/discord.md)) |
| [Point of Origin](https://github.com/Londopy/point-of-origin), a platformer where the puzzle is the ground | the whole build is Nexium: `build.nx` drives the Odin simulation's DLL, `tools/bindgen.nx` reads the Odin exports and writes the C# bindings Unity calls so the two sides cannot drift, `tools/levels.nx` compiles the level maps into the JSON the game loads (every level grown by the same simulation, so it is solvable by construction), `tools/chapters.nx` writes the docs from them |
| [QNI](https://github.com/Londopy/qni), net reminders, check-in help and a net control tutorial for the Cal Poly Amateur Radio Club's Discord (W6BHZ) | the whole program is Nexium: slash commands and buttons answered over webhooks, with no bot user and no permissions; every request checked for Discord's Ed25519 signature before anything else is read (through nxtls); net cards, a practice mode for net control, and net logs in the officers' own sheet format; tested end to end against a fake Discord |
| [nxtls](https://github.com/Londopy/nxtls), cryptography in pure Nexium | SHA-2, HMAC, HKDF with TLS 1.3's labels, and Ed25519 verification, with no C and no `unsafe`, tested against the standards' published vectors; a package: `nx add nxtls --git https://github.com/Londopy/nxtls --tag v0.1.0`; a TLS 1.3 client is the plan |

Using Nexium somewhere? Open an issue or a pull request and it goes here.

## Commands

| command | what it does |
| --- | --- |
| `nx build file.nx` | compile to an executable (or object when there is no `main`) |
| `nx run file.nx` | build and run; `--watch` runs again whenever a file of the program changes |
| `nx test file.nx [filter]` | run the `test "..."` blocks; `--watch` too |
| `nx check file.nx` | type-check and report effect violations |
| `nx effects file.nx` | print the inferred effects of every function |
| `nx explain file.nx f effect` | why `f` has the effect: the calls that carry it in, down to the primitive, as a tree |
| `nx audit file.nx` | list `unsafe` blocks and mutable globals; `--lock` writes the effects lockfile, `--check` fails on a gained effect |
| `nx ship file.nx` | produce every declared `artifact` |
| `nx emit-c file.nx` | print the generated C |
| `nx tir file.nx [--sigs]` | the checked program as S-expressions (the compiler's own tests read it) |
| `nx fmt file.nx [--check]` | canonical formatting |
| `nx bench file.nx` | measure the `bench "name" { }` blocks: an optimized build, the median time per iteration |
| `nx fix file.nx` | make the checker's mechanical fixes (`.clone()`, `@escape(...)`, `_ = `) and migrate deprecated forms; see [docs/stability.md](docs/stability.md) |
| `nx doc file.nx` | HTML documentation with inferred effects |
| `nx size file.nx` | attribute binary bytes to declarations |
| `nx layout file.nx [Type...]` | offsets, sizes and padding of a struct or enum, and the order by alignment that would shrink it |
| `nx upgrade` | the latest release in place of this executable, verified; `--check` only reports |
| `nx install [DIR]` | this copy, with what sits beside it, into the user's place and onto the PATH (the portable zip installing itself) |
| `nx refcounts file.nx` | every retain and release site |
| `nx leaks file.nx` | run with allocation tracking and report leaks |
| `nx lsp` | language server over stdio |
| `nx doctor` | which C compiler will be used, and whether the installation works |
| `nx repl`, or just `nx` | an interactive session: type code, see values, keep bindings |
| `nx play [file.nx]` | check a whole program and run it in the interpreter, nothing compiled; stdin when no file (the command the site's playground runs) |
| `nx topo [<chapter>\|check\|hint\|solution\|quiz]` | the Topo's exercises in the terminal, graded by the compiler, progress kept |

Options: `--mode debug|safe|fast|small`, `--target x86_64-linux-gnu` (any
target `zig cc` knows), `--cpu baseline|native|<name>` (baseline by default,
so a binary runs on any machine of its architecture), `--out-dir`,
`--keep-c`, `--cc`, `--strict` (warnings are errors), `--sanitize
address,undefined` (the C compiler's sanitizers; `address` needs gcc or
clang), and for C interop `-I`, `--link`, `--link-path`, `--c-source`.

## Status

**1.0: language-stable, early ecosystem.** The language changes only by
addition under the [stability policy](docs/stability.md); the compiler is
written in Nexium and builds itself; every example, spec case and tutorial
program runs in CI on three platforms, under the sanitizers and the fuzzer.
What 1.0 is not yet, and where each is answered, is the first section of
[the roadmap](ROADMAP.md): memory safety is the view rules of 1.2, errors
since 1.3 (`nx fix` makes the mechanical fixes), there are no benchmark
numbers beyond [the numbers page](docs/numbers.md) (four programs in five
languages on one runner, regenerated weekly), and the ecosystem is one maintainer, sixteen standard
library modules and two projects outside the tree (statusmith's
[Discord Rich Presence SDK](docs/discord.md) and Point of Origin's build
tooling, [above](#in-the-wild)). [`KNOWN_ISSUES.md`](KNOWN_ISSUES.md) lists every open bug
with its fix; [`DECISIONS.md`](DECISIONS.md) every call made where the
specification was open.

## Release names

A major version is a mountain, in the order the fourteen 8000-metre peaks
were first climbed; the versions under it are the climb: camps, routes and
faces for minor versions, the first-ascent expedition's members for
patches, `Summit` for `X.0.0`. The 0.x line is the approach and the camps
of Annapurna, the first 8000er climbed (1950), so 1.0.0 is
`Annapurna: Summit`; 0.7.0, where the compiler started building itself, is
`Annapurna: Camp V`, the last camp before the summit push. The name is in
the changelog, the release title and `nx version`;
[docs/release-names.md](docs/release-names.md) has the rule, the ledger,
and the mountains still to climb.

## Self-hosting

The compiler is written in Nexium, under [`self/`](self), and builds itself.
A machine with no `nx` builds one from [`bootstrap/nx.c`](bootstrap/nx.c),
the C the compiler emits for itself, with any C compiler and no Rust:

```sh
sh bootstrap/build.sh     # nx.c -> nx0; nx0 builds self/nx.nx -> nx1; nx1 rebuilds itself to the same C -> nx2
```

| stage | file | lines |
| --- | --- | --- |
| lexer | [`self/lexer.nx`](self/lexer.nx) | tokens |
| parser | [`self/parser.nx`](self/parser.nx) | an id-arena syntax tree |
| checker | [`self/check.nx`](self/check.nx), `self/check_*.nx`, [`self/cimport.nx`](self/cimport.nx) | types, effects, ownership, generics, the compile-time interpreter, C header import, every diagnostic |
| C emitter | [`self/cgen.nx`](self/cgen.nx) | one C file per program |
| driver | [`self/nx.nx`](self/nx.nx) | build, run, test, check, emit-c, tir; the standard library embedded |
| tools | [`self/fmt.nx`](self/fmt.nx), [`self/doc.nx`](self/doc.nx), [`self/tools.nx`](self/tools.nx), [`self/size.nx`](self/size.nx), [`self/manifest.nx`](self/manifest.nx), [`self/ship.nx`](self/ship.nx), [`self/lsp.nx`](self/lsp.nx), [`self/repl.nx`](self/repl.nx) | the formatter, the documentation generator, the reports, packages, `ship`, the language server, the REPL |

Every example, every spec case and every compile-fail case runs through the
bootstrapped compiler, driven by a test harness that is itself a Nexium
program (`nx run tests/run.nx`), in CI on three platforms with no Rust
toolchain at all. The first compiler, in Rust, drove the port and was
deleted at 1.0 (decision 90).

## Languages in the repository

Non-blank lines of code, excluding build output, dependencies, and generated
files (`bootstrap/nx.c`, the tree-sitter parser, `gui/font.bin`, lock files):

| language | lines | share | what it is |
| --- | --- | --- | --- |
| Nexium | 38,374 | 85.4% | the compiler and its tools (27,100 lines under `self/`), the standard library, the test harness and the fuzzer, the examples, the tutorial's programs, the GUI, the site generator, four benchmarks |
| C | 2,925 | 6.5% | the runtime `nx_rt.h`, the GUI window layer, vendored test C, a benchmark |
| Python | 1,063 | 2.4% | the release scripts (notes, package manifests, wheels and npm packages, the std docs), the benchmark runner, a benchmark |
| editor files | 1,028 | 2.3% | tree-sitter queries, Emacs Lisp, Vim script, Lua for Neovim, and the 25 lines of Rust that Zed requires of an extension |
| JavaScript, TypeScript | 550 | 1.2% | the VS Code extension and the tree-sitter grammar |
| Inno Setup, shell, PowerShell | 777 | 1.7% | the Windows installer script, `install.sh`, `install.ps1`, the Chocolatey scripts |
| Rust, Go, Ruby | 236 | 0.5% | one benchmark each in Rust and Go, and the Homebrew formula |

There is no Rust in the compiler: the first compiler drove the port and
was deleted at 1.0 (decision 90). The Rust that remains is the glue of the
Zed extension, which Zed compiles to WebAssembly, and one benchmark
program written to be measured against, beside its Go twin. Zig is not in
the table because there is no Zig source in the tree: `zig cc` is the C
compiler `nx` runs (bundled by the Windows installer, downloaded by the
install script), the same way a C compiler is used and not written.

## Layout

```
bootstrap/      the C seed the compiler is built from, and the build scripts
runtime/        nx_rt.h, embedded into every generated C file
std/            the standard library in Nexium, embedded in the compiler
self/           the compiler in Nexium, stage by stage
gui/            nexium-gui: immediate-mode GUI in Nexium, demo, and the C platform layer
editors/        VS Code extension, tree-sitter grammar, and the files for ten more editors
examples/       programs with recorded output, run by the tests
topo/           the tutorial: chapters, and the programs they show (run by the tests)
site/           the documentation site generator, a Nexium program
tests/          the harness (run.nx), the spec conformance suite (tests/spec) and compile-fail cases
docs/           how it works, language reference, embedding guide, i18n/ translations
bench/          four programs in five languages behind the numbers page
installers/     the Windows installer script, install.sh and install.ps1, the winget and Chocolatey manifests
docker/         the compiler images for ghcr.io (Debian and Alpine)
Formula/, bucket/  this repository as a Homebrew tap and a Scoop bucket (written at each release)
scripts/        release notes, package manifests, wheels and npm packages, the std docs
assets/         logo, banner and the social preview
nexium-spec.txt          the design
nexium-systems-spec.txt  the archived systems language; sections 4 to 9 are the syntax reference
DECISIONS.md    decisions made where the specification was open
KNOWN_ISSUES.md open bugs and limitations; fixes move to the changelog
```

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Bugs and proposals go through GitHub
issues; a language change must name the hard constraint in section 3 of the
specification that it serves. Pull requests pass the tests on three
platforms, the formatters, a changelog check and the [Contributor License
Agreement](CLA.md) before they merge; you keep your copyright.

## License

MIT. Copyright (c) 2026 Londopy.

<p align="center"><img src="https://raw.githubusercontent.com/Londopy/nexium/main/assets/logo.svg" alt="" width="48"></p>
