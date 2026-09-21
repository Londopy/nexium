# Installing Nexium

Every way to get `nx` onto a machine, what each one sets up, and how `nx`
finds its C compiler.

## Windows: the installer

Download `nexium-<version>-setup-x64.exe` from the
[Releases](https://github.com/Londopy/nexium/releases) page and run it. The
wizard offers:

- **For me or for all users.** Per-user needs no administrator rights and
  installs under `%LocalAppData%\Programs\Nexium`; all-users installs under
  `Program Files`.
- **Components**: the compiler (always), the bundled Zig toolchain
  (recommended; it is the C compiler and linker `nx` uses), the standard
  library sources and examples, the documentation, and the VS Code
  extension file.
- **Tasks**: add `nx` to the PATH (checked by default), register the `.nx`
  file type with an icon and a "Run with Nexium" context entry, add "Open
  Nexium console here" and "Open Nexium REPL here" to the right-click menu
  of a folder's background, add a "Nexium REPL" profile to Windows Terminal
  (offered checked when Terminal is installed; a JSON fragment, so your
  settings are not touched), and install the VS Code extension if `code`
  is on the PATH.
- **Finish**: launch the interactive session, open the README, or open a
  console that runs `nx doctor`.
- **Start menu**: "Nexium <version> (64-bit)" opens the interactive session,
  so typing `nx` in the Windows search bar works like typing `python`.

Nothing else is required. Uninstall from Settings; it removes the files and
the PATH entry, and leaves programs you compiled alone.

Silent install for scripts: `nexium-<version>-setup-x64.exe /VERYSILENT /TASKS=addtopath`.

## Windows: the PowerShell one-liner

```powershell
irm https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.ps1 | iex
```

It downloads the portable build for the machine (x64 or ARM64), verifies
it against the release's `SHA256SUMS.txt`, installs `nx.exe`, the standard
library, the examples and the docs to `%LocalAppData%\Programs\Nexium`,
adds that directory to the user's PATH, and downloads Zig beside it when
no C compiler is found. No wizard, no administrator rights, nothing
registered. Variables, set before the line: `NEXIUM_VERSION=v1.0.3` pins a
release, `NEXIUM_HOME` changes the directory, `NEXIUM_NO_MODIFY_PATH=1`
leaves the PATH alone, `NEXIUM_NO_ZIG=1` never downloads Zig. Remove it by
deleting the directory and the PATH entry.

**Chocolatey**: `choco install nexium` once the package is on
chocolatey.org (the release workflow pushes it when its key is set); each
release also attaches the `.nupkg`, which installs with
`choco install nexium --source .` from the directory it is in. The package
runs the installer silently with `nx` added to the PATH.

The installer refuses to start while another Nexium installer or
uninstaller is open, and names it. A running `nx.exe` holds the files
being replaced, so the installer closes it first (Windows' Restart
Manager; it is not started again). Every step of the run is logged, and
the log is kept as `install.log` next to `nx.exe`; the uninstaller takes
`/LOG="path"` to write one of its own.

A portable `nx-<version>-x86_64-pc-windows-msvc.zip` has the same files
without the installer or Zig; put its folder on the PATH and have Zig on the
PATH yourself.

## macOS and Linux: the install script

```sh
curl -fsSL https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.sh | sh
```

It downloads the release for your platform, verifies the archive against the
release's `SHA256SUMS.txt`, installs `nx` to `~/.nexium/bin` with the
examples, standard library sources, and docs under `~/.nexium/share`, and
adds the bin directory to your PATH in `~/.profile`, `~/.bashrc`, and
`~/.zshrc`. On macOS the system compiler from the Xcode command line tools is
used (`xcode-select --install` if missing). On Linux, when no compiler is
found, it downloads Zig into `~/.nexium/zig`.

When no release is built for the machine (an x86-64 Mac, a BSD, a RISC-V
board), or the download fails, the script builds `nx` from the one C file
below with the C compiler it finds (`cc`, `gcc`, `clang` or `zig`) and
installs that instead.

Variables: `NEXIUM_VERSION=v1.0.3` pins a release, `NEXIUM_HOME` changes the
directory, `NEXIUM_NO_MODIFY_PATH=1` leaves shell files alone,
`NEXIUM_NO_ZIG=1` never downloads Zig, `NEXIUM_FROM_SOURCE=1` builds from
the one C file even when a release exists. Uninstall by deleting `~/.nexium`
and the three lines the script added.

## pip and npm

Every release carries the compiler as a wheel and as an npm package, both
named `nexium-lang` (`nexium` is taken on both registries by unrelated
projects); the release workflow uploads them when its tokens are set, and
the files are attached to the release either way.

```sh
pip install nexium-lang        # nx on the PATH of the environment
npm install -g nexium-lang     # nx from npm; the platform package is an optional dependency
npx nexium-lang version        # or without installing
```

The wheel is per platform (Windows x64 and ARM64, macOS on Apple Silicon,
Linux x86-64 and aarch64, glibc) and holds the binary under
`nexium_lang/bin/`; the console script `nx` hands over to it. The npm
package is the shape esbuild uses: `nexium-lang` has `bin/nx.js`, which
runs the binary from `@nexium-lang/<os>-<cpu>`, the one optional
dependency npm installs for the machine. Neither carries Zig: `nx` needs a
C compiler as usual (`nx doctor` says what it found).

## Homebrew, Scoop and winget

The repository is its own Homebrew tap and Scoop bucket; the release
workflow regenerates the manifests with each release's checksums
(`scripts/packaging.py`).

**Homebrew**, on macOS and Linux. Apple Silicon and Linux get the release
build; an Intel Mac or another architecture builds the compiler from its
one C file:

```sh
brew tap londopy/tap https://github.com/Londopy/nexium
brew install londopy/tap/nexium
```

**Scoop**, on Windows: the portable build, without Zig (`scoop install zig`
beside it, or let `nx doctor` tell you what it found):

```powershell
scoop bucket add nexium https://github.com/Londopy/nexium
scoop install nexium
```

or, without adding the bucket,
`scoop install https://raw.githubusercontent.com/Londopy/nexium/main/bucket/nexium.json`.

**winget**: the manifests for `Londopy.Nexium` are in `installers/winget/`,
ready for [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs)
(a new package is a pull request there, made by a person). Until it is
accepted, a checkout installs them directly:

```powershell
winget settings --enable LocalManifestFiles    # once, as administrator
winget install --manifest installers\winget
```

## One C file

`bootstrap/nx.c` is the C the compiler emits for itself, as of the release,
with the standard library inside. Any C compiler builds it, and the result
is the whole `nx`, on any Unix the C compiler runs on, including machines
no release is built for:

```sh
curl -fsSL https://raw.githubusercontent.com/Londopy/nexium/v1.0.3/bootstrap/nx.c -o nx.c
cc -std=gnu11 -O2 -w -fno-strict-aliasing -o nx nx.c -lm -lpthread
```

(`main` in place of `v1.0.3` gives the seed of the next release, which may
be a little behind `self/`.) `nx` needs a C compiler at run time as well;
the one that built it will do. On Windows, `zig cc` builds it with
`-lws2_32` at the end, but the installer is the shorter road there.

## From source

Nothing but a C compiler is needed: `sh bootstrap/build.sh` (or
`.\bootstrap\build.ps1`) builds `nx0` from the C seed the compiler emits
for itself, `nx0` builds the compiler from `self/`, and the result rebuilds
itself to the same C; `nx-out/bootstrap/nx2` is the compiler. `CC` names
the C compiler for the build (default `zig cc`, `cc` on macOS); `NX_CC` or
`NX_ZIG` names the one `nx` itself runs. See `bootstrap/README.md`.

## Docker

`ghcr.io/londopy/nexium` is the compiler with its Zig toolchain, the
standard library, the examples and the docs, on Debian; `:alpine` is the
same on Alpine. Both come for amd64 and arm64, from `docker/`, built at
each release by the Docker workflow. The image's entry point is `nx`:

```sh
docker run --rm -v "$PWD":/work ghcr.io/londopy/nexium run hello.nx
docker run --rm ghcr.io/londopy/nexium doctor
docker run --rm -it --entrypoint sh ghcr.io/londopy/nexium    # a shell inside
```

Tags: `latest` and `debian` (the Debian image), `alpine`, and each with
the version in front (`1.0.4`, `1.0.4-alpine`). The examples are at
`/usr/local/share/nexium/examples` inside.

## In CI

The `Londopy/nexium/.github/actions/setup-nexium` action installs `nx` and Zig
on a GitHub runner; see [releasing-your-program.md](releasing-your-program.md).

## How `nx` finds a C compiler

In order, the first that applies wins:

1. `--cc <compiler>` on the command line.
2. `NX_CC` in the environment, for example `NX_CC=gcc` or `NX_CC="clang -fuse-ld=lld"`.
3. `NX_ZIG=/path/to/zig`.
4. A Zig next to `nx`: `<nx directory>/zig/zig` (the Windows installer) or
   `<nx directory>/../zig/zig` (`~/.nexium`).
5. On macOS, for native builds, the system `cc`.
6. `zig` on the PATH.
7. On macOS and Linux, when there is no zig: `cc`, `gcc` or `clang` on the
   PATH, the first found. A machine that built `nx` from the one C file has
   one.

`nx doctor` prints which one is in effect and whether it runs. Cross-compiling
(`--target`) always uses Zig, since that is what makes it possible.

`nx doctor` also asks GitHub for the latest release (through `curl`, three
seconds at most) and says when a newer one exists; `NX_OFFLINE=1` skips
the question, and a machine without `curl` or a connection is told so, not
failed.

## Which CPU a binary is built for

By default, the baseline of the machine's architecture: on x86-64 that is
plain x86-64 (SSE2), so a binary built on one machine runs on every 64-bit
x86 machine, which is what a release, a wheel or an installer needs. `zig cc`
on its own compiles for the CPU it runs on, and a compiler built on a
machine with AVX-512 crashed with an illegal instruction on one without it
(1.0.1). `--cpu native` (or `NX_CPU=native`) asks for this machine's CPU,
for a program that will only run here; `--cpu x86_64_v3` names one (Zig's
names with `zig cc`, `-march` names with gcc or clang). `nx doctor` prints
the choice, and `os.arch()` tells a program which architecture it runs on.

## An editor

Every editor gets the same language server: `nx lsp` speaks LSP over
stdio and gives diagnostics as you type, hover with inferred effects, go
to definition, completion and rename. The
[`editors/`](https://github.com/Londopy/nexium/tree/main/editors)
directory of the repository has the pieces for each one:

| editor | install |
| --- | --- |
| VS Code | the `.vsix` on every release (the Windows installer installs it when `code` is on the PATH); the "Nexium" extension on the Marketplace and on Open VSX once a release has been published there (the release workflow does it when the publisher tokens are set) |
| Vim | `Plug 'Londopy/nexium', { 'rtp': 'editors/vim' }` |
| Neovim | the `editors/neovim` plugin: tree-sitter, LSP and the Vim files as fallback |
| Helix | append `editors/helix/languages.toml`, copy its queries, `hx --grammar fetch && hx --grammar build` |
| Zed | *Install Dev Extension* on `editors/zed` |
| Emacs | `editors/emacs/nexium-mode.el`; registers itself with Eglot and lsp-mode |
| Kate, KWrite, KDevelop, Qt Creator | copy `editors/kate/nexium.xml` into the KSyntaxHighlighting directory |
| JetBrains IDEs | `editors/vscode` as a TextMate bundle, `nx lsp` through LSP4IJ |
| Sublime Text | copy `editors/sublime/Nexium.sublime-syntax` and `Nexium.sublime-build` into `Packages/User` (Ctrl+B runs the file) |
| Notepad++ | copy `editors/notepad-plus-plus/nexium.udl.xml` into `%AppData%\Notepad++\userDefineLangs` |
| nano | include `editors/nano/nexium.nanorc` from `~/.nanorc` |

Each directory has a README with the details.

## Torrents

Every file of a release is also attached as a `.torrent`, one file each,
with its GitHub download as the web seed: a BitTorrent client fetches it
from GitHub when no peer has it and from peers when they do, so a release
stays reachable through a client when a direct download is slow or
blocked. The release notes list the magnet links (they carry the same web
seed) with a QR code of each, for a phone or a machine without a browser
on the page.

## Verifying downloads

Every release ships `SHA256SUMS.txt` and lists the same values on the release
page.

```sh
sha256sum -c SHA256SUMS.txt --ignore-missing      # Linux
shasum -a 256 -c SHA256SUMS.txt --ignore-missing   # macOS
```

```powershell
Get-FileHash .\nexium-1.0.3-setup-x64.exe -Algorithm SHA256
```

The install script verifies automatically and refuses a mismatch.
