<div class="hero">
<img src="assets/banner.svg" alt="Nexium" style="max-width: 640px; width: 100%">
<p class="tag">Nexium is a language complete enough to build everything in, that is also the best thing to adopt for one piece of something else.</p>
</div>

Nexium compiles to native code through C, has automatic reference counting
without a tracing collector, a machine-checked effect system that says
whether a function allocates, blocks, or can panic, and a compiler that
turns one source tree into a C library, a Python wheel, a Rust crate, an
npm package, or a command line tool. The compiler is written in Nexium and
builds itself from the C it emits.

<div class="cards">
<div class="card"><h3><a href="topo/01-base-camp.html">The Topo</a></h3><p>The route up the mountain, one pitch at a time: install, hello world, the REPL, a calculator, ownership, effects, a GUI, a neural network, shipping a library. Every program in it is run by the test suite.</p></div>
<div class="card"><h3><a href="docs/language.html">The language reference</a></h3><p>Every construct the compiler implements, with the rules for types, ownership, effects, binary patterns, compile time and C interop.</p></div>
<div class="card"><h3><a href="docs/std.html">The standard library</a></h3><p>Every module and function, generated from the doc comments in <code>std/</code>.</p></div>
<div class="card"><h3><a href="docs/spec.html">The specification</a></h3><p>What Nexium promises: the document the conformance suite is written against, and what a version number means (<a href="docs/stability.html">stability</a>, <a href="docs/platforms.html">platforms</a>).</p></div>
</div>

## Install

<div class="two">
<div>

**Windows**: the installer from the [latest release](https://github.com/Londopy/nexium/releases/latest) installs `nx`, a bundled Zig toolchain, the standard library, the examples and the VS Code extension.

**macOS and Linux**:

```bash
curl -fsSL https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.sh | sh
```

</div>
<div>

**One C file**, on any Unix with a C compiler. It is the compiler's own C, standard library inside, and the result is the whole `nx`:

```bash
curl -fsSL https://raw.githubusercontent.com/Londopy/nexium/main/bootstrap/nx.c -o nx.c
cc -std=gnu11 -O2 -w -fno-strict-aliasing -o nx nx.c -lm -lpthread
```

**From source**, with nothing but a C compiler (Zig on the PATH, or `cc` on macOS):

```bash
git clone https://github.com/Londopy/nexium
cd nexium
sh bootstrap/build.sh
```

The compiler is `nx-out/bootstrap/nx2`. Also on [PyPI](https://pypi.org/project/nexium-lang/) (`pip install nexium-lang`), [npm](https://www.npmjs.com/package/nexium-lang), [Docker](https://github.com/Londopy/nexium/pkgs/container/nexium), Homebrew, Scoop, Chocolatey and winget, and the VS Code extension on [Open VSX](https://open-vsx.org/extension/Londopy/nexium): [where to get it](docs/install.html#where-to-get-it).

</div>
</div>

## One file, every target

```nexium
/// A hash of the bytes, the FNV-1a way.
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
artifact node   { name = "hasher" }
```

```bash
$ nx ship hasher.nx
  hasher.h  hasher.dll  hasher.lib          # C, from the same source
  python/hasher/  hasher-1.0-*.whl          # pip install
  node/                                     # npm install
```

`export(c)` puts a function on the boundary. Because the checker proved
`checksum` cannot panic, it gets its natural C signature; a function that
could panic returns a status and the host gets an error instead of an
abort. [Embedding](docs/embedding.html) has the whole story.

## What it feels like

{{include topo/code/centroid.nx}}

- **Ownership without a borrow checker.** `List`, `String` and `Map` own
  their storage; passing one moves it, a function borrows its parameters,
  and the compiler tells you when you used something after it moved.
- **Effects are inferred, bounds are checked.** Write `!allocates` or
  `!panics` on a signature and the compiler names the exact line, through
  any number of calls, that would break the promise.
- **Compile time is the same language.** `comptime` runs the interpreter
  the REPL uses; tables, tests and embedded files are computed before the
  C compiler sees them.
- **Binary patterns.** `<<len:16/little, payload:len*8, rest:bytes>>`
  parses a protocol in one `match`.

## The tools in the box

`nx build`, `run`, `test`, `check`, `fmt`, `doc`, `repl`, `lsp`, `ship`,
`size`, `leaks`, `effects`, `audit`, `refcounts`, `doctor`: one binary, no
dependencies beyond a C compiler. The language server is `nx lsp`, and
the repository's [`editors/`](https://github.com/Londopy/nexium/tree/main/editors)
has it wired into VS Code, Vim, Neovim, Helix, Zed, Emacs, Kate,
JetBrains, Sublime Text, Notepad++ and nano ([the install guide](docs/install.html#an-editor)
has the one-liners).

## In the wild

- [statusmith](https://github.com/Londopy/statusmith): Discord Rich Presence
  from the tray; its SDK is a Nexium package ([the page](docs/discord.html)).
- [Point of Origin](https://github.com/Londopy/point-of-origin): a platformer
  for a game jam, Odin and Unity underneath, and every build step written in
  Nexium: the native build, the C# bindings generated from the Odin exports,
  the levels compiled into the game's JSON, the docs written from them.
- [QNI](https://github.com/Londopy/qni): net reminders, check-in help and a
  net control tutorial for the Cal Poly Amateur Radio Club's Discord
  (W6BHZ), all Nexium; every request's Ed25519 signature is checked with
  nxtls before anything else is read.
- [nxtls](https://github.com/Londopy/nxtls): SHA-2, HMAC, HKDF and Ed25519
  verification in pure Nexium, no C and no `unsafe`, tested against the
  published vectors; a package ([the catalogue](docs/packages.html)).

Using Nexium somewhere? An issue or a pull request on
[GitHub](https://github.com/Londopy/nexium) puts it here.

## Where things are

- [The Topo](topo/01-base-camp.html), the guided route through the language.
- [The language reference](docs/language.html) and [the standard library](docs/std.html).
- [The specification](docs/spec.html), [the roadmap](docs/roadmap.html) and [the decisions](docs/decisions.html) that shaped the language.
- [Contributing](docs/contributing.html), [the changelog](docs/changelog.html), [release names](docs/release-names.html).
- [The site's API](docs/api.html): the current release, every release, the standard library, the Topo and the commands, as JSON.
- The source, on [GitHub](https://github.com/Londopy/nexium).
