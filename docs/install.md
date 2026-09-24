# Installing Nexium

Every way to get `nx` onto a machine, what each one sets up, and how `nx`
finds its C compiler.

## Where to get it

Every release of `nx` reaches these places, most of them by the release
workflow itself:

| where | how | page |
| --- | --- | --- |
| GitHub | the installer, the archives, checksums, torrents | [releases](https://github.com/Londopy/nexium/releases) |
| PyPI | `pip install nexium-lang` | [pypi.org/project/nexium-lang](https://pypi.org/project/nexium-lang/) |
| npm | `npm install -g nexium-lang` | [npmjs.com/package/nexium-lang](https://www.npmjs.com/package/nexium-lang) |
| Docker | `docker run ghcr.io/londopy/nexium` | [the container image](https://github.com/Londopy/nexium/pkgs/container/nexium) |
| Homebrew | `brew install londopy/tap/nexium` | [the tap is this repository](https://github.com/Londopy/nexium/tree/main/Formula) |
| Scoop | `scoop bucket add londopy https://github.com/Londopy/scoop-bucket`, then `scoop install nexium` | [Londopy/scoop-bucket](https://github.com/Londopy/scoop-bucket), kept current by Scoop's own updater; the manifest is also [in this repository](https://github.com/Londopy/nexium/tree/main/bucket) |
| Chocolatey | `choco install nexium` | [community.chocolatey.org/packages/nexium](https://community.chocolatey.org/packages/nexium), from its moderators' approval of the first version on |
| winget | `winget install Londopy.Nexium` | [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs/tree/master/manifests/l/Londopy/Nexium), from the merge of [the first submission](https://github.com/microsoft/winget-pkgs/pull/438838) on |
| Debian, Ubuntu | `sudo dpkg -i nexium_<version>_amd64.deb` (also arm64) | [attached to each release](https://github.com/Londopy/nexium/releases/latest) |
| Fedora, RHEL, SUSE | `sudo rpm -i nexium-<version>.x86_64.rpm` (also aarch64) | [attached to each release](https://github.com/Londopy/nexium/releases/latest) |
| Nix | `nix run github:Londopy/nexium`, `nix profile install github:Londopy/nexium` | [`flake.nix`](https://github.com/Londopy/nexium/blob/main/flake.nix), built from the one C file |
| Arch Linux | `yay -S nexium-bin` (or any AUR helper) | [`installers/aur`](https://github.com/Londopy/nexium/tree/main/installers/aur), written at each release; on the AUR once its first push is made |
| mise, asdf | `mise use -g "ubi:Londopy/nexium[exe=nx]"` | the release binary through mise's `ubi` backend |
| GitHub Codespaces, dev containers | [open in a Codespace](https://codespaces.new/Londopy/nexium) | [`.devcontainer`](https://github.com/Londopy/nexium/tree/main/.devcontainer) on the Docker image |
| Colab, Jupyter | `!pip install -q nexium-lang` in a notebook cell | [three cells](#in-a-notebook-colab-and-jupyter), below, on the wheel from PyPI |
| Open VSX | the VS Code extension, for VSCodium, Cursor and the other forks | [open-vsx.org/extension/Londopy/nexium](https://open-vsx.org/extension/Londopy/nexium) |
| Visual Studio Marketplace | the VS Code extension | pending the publisher's token |

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
- **Already installed?** The page after the welcome then says which
  version is in which directory and offers the upgrade (a repair when it is
  the same version, a replacement when the installed one is newer), with
  the directory, components and tasks of the last install as the defaults,
  or the removal of the installed version, which runs its uninstaller and
  exits.
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
registered. Variables, set before the line: `NEXIUM_VERSION=v1.3.0` pins a
release, `NEXIUM_HOME` changes the directory, `NEXIUM_NO_MODIFY_PATH=1`
leaves the PATH alone, `NEXIUM_NO_ZIG=1` never downloads Zig. Remove it by
deleting the directory and the PATH entry.

## Portable mode and `nx install`

The portable zip on every release is a folder that runs from anywhere:
`nx.exe` (or `nx`), the standard library, the examples and the docs, and
nothing is written outside it except Zig's global cache in the user's
profile. A file named `portable` beside the executable moves that cache
beside it too (`cache/`), so a copy on a USB stick leaves nothing on the
host; `nx doctor` reports the mode. When the copy should stay, `nx
install` puts it in the user's place (`%LocalAppData%\Programs\Nexium`,
or `~/.nexium` with `bin/` and `share/`; `nx install DIR` names another),
with the zig, examples, std and docs beside it, and adds the directory to
the user's PATH unless `NEXIUM_NO_MODIFY_PATH=1`.

**Chocolatey**: `choco install nexium` from
[community.chocolatey.org/packages/nexium](https://community.chocolatey.org/packages/nexium)
(the Chocolatey workflow pushes each version when its release is made;
Chocolatey's moderators approve a package's first version by hand, so
1.2.0 is listed there before it is installable, and until it is approved
Chocolatey takes no further version, so 1.2.1 follows it through the
queue); each release also attaches the `.nupkg`, which installs with
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

Variables: `NEXIUM_VERSION=v1.3.0` pins a release, `NEXIUM_HOME` changes the
directory, `NEXIUM_NO_MODIFY_PATH=1` leaves shell files alone,
`NEXIUM_NO_ZIG=1` never downloads Zig, `NEXIUM_FROM_SOURCE=1` builds from
the one C file even when a release exists. Uninstall by deleting `~/.nexium`
and the three lines the script added.

## pip and npm

Every release carries the compiler as a wheel and as an npm package, both
named `nexium-lang` (`nexium` is taken on both registries by unrelated
projects). The wheels go to PyPI through [trusted
publishing](https://docs.pypi.org/trusted-publishers/): the `PyPI`
workflow (`.github/workflows/pypi.yml`) runs when a release is built and
uploads them with a short-lived token PyPI mints for that run, so there is
no secret to keep; PyPI knows the workflow by its file name, this
repository and the `pypi` environment. The npm packages go the same way
through the `npm` workflow (`.github/workflows/npm.yml`) with provenance:
`NPM_TOKEN` for a package's first publish, the trusted publisher
configured on each package's settings page after that (owner `Londopy`,
repository `nexium`, workflow `npm.yml`). The files are attached to the
release either way.

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

## In a notebook: Colab and Jupyter

The wheel is all a notebook needs: `pip install` puts `nx` on the PATH of
the notebook's Python, and a cell that starts with `!` runs it. In
[Google Colab](https://colab.research.google.com), three cells run a
program:

```
!pip install -q nexium-lang
```

```
%%writefile hello.nx
fn main() {
    println("hello from Colab", .{})
}
```

```
!nx run hello.nx
```

`%%writefile` must be the first line of its cell, with the program under
it in the same cell; the file lands in the notebook's working directory,
and `!nx test hello.nx` and `!nx bench hello.nx` work on it the same way.
Colab's machine is Linux on x86-64 with gcc and no zig, so `nx` builds with
gcc (step 7 of [How `nx` finds a C compiler](#how-nx-finds-a-c-compiler)).
Jupyter, JupyterLab and VS Code notebooks on your own machine run the same
cells with the C compiler `nx` finds there (`!nx doctor` says which).
The cells stay Python, with Nexium beside them through `!nx`; a Nexium
kernel is on the [roadmap](../ROADMAP.md#tools-only-this-language-can-have).

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
scoop bucket add londopy https://github.com/Londopy/scoop-bucket
scoop install nexium
```

The bucket's own updater reads the manifest's `checkver` and `autoupdate`
and moves it to each new release. Without adding a bucket,
`scoop install https://raw.githubusercontent.com/Londopy/nexium/main/bucket/nexium.json`
installs from the copy in this repository, which the release workflow
writes.

**Arch Linux**: the `nexium-bin` package (`installers/aur/PKGBUILD` and
`.SRCINFO`, written by the release workflow with the release's checksums)
is published on the AUR by a maintainer with an account there: a clone of
`ssh://aur@aur.archlinux.org/nexium-bin.git`, the two files copied in,
committed and pushed. Until then, `makepkg -si` in a checkout's
`installers/aur/` builds and installs it.

**winget**: `winget install Londopy.Nexium`, once
[the first submission](https://github.com/microsoft/winget-pkgs/pull/438838)
to [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) is
merged (a new package is a pull request there; later versions are one
`wingetcreate` command). The manifests are in `installers/winget/`, and
a checkout installs them directly:

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
curl -fsSL https://raw.githubusercontent.com/Londopy/nexium/v1.3.0/bootstrap/nx.c -o nx.c
cc -std=gnu11 -O2 -w -fno-strict-aliasing -o nx nx.c -lm -lpthread
```

(`main` in place of `v1.3.0` gives the seed of the next release, which may
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
stdio and gives diagnostics as you type, hover with types and inferred
effects, go to definition, completion and rename; when the program checks,
definition and rename follow what the checker resolved each name to (the
`scale` of the right `impl`, a field in every file that names it). The
[`editors/`](https://github.com/Londopy/nexium/tree/main/editors)
directory of the repository has the pieces for each one:

| editor | install |
| --- | --- |
| VS Code | the "Nexium" extension by Londopy on [Open VSX](https://open-vsx.org/extension/Londopy/nexium) (VSCodium, Cursor, Windsurf and the other forks install from it), on the Visual Studio Marketplace once its publisher token is set, and as the `.vsix` on every release (the Windows installer installs it when `code` is on the PATH) |
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

## Completions and the manual page

`nx completions bash|zsh|fish|powershell` prints the completion script for
a shell, generated from the compiler's own table of commands and options,
and `nx man` prints the manual page. Homebrew installs both; the install
script puts them under `~/.nexium/share/` (`man ~/.nexium/share/man/man1/nx.1`)
and copies the bash and fish scripts into place when those directories
exist; the release archives carry them as `nx.1` and `completions/`. By
hand:

```bash
nx completions bash > ~/.local/share/bash-completion/completions/nx
nx completions zsh > ~/.zsh/completions/_nx          # with that directory in fpath
nx completions fish > ~/.config/fish/completions/nx.fish
```

```powershell
Add-Content $PROFILE 'nx completions powershell | Out-String | Invoke-Expression'
```

## Upgrading

`nx doctor` says when a newer release exists, and `nx upgrade` installs it
in place: the archive for the machine is downloaded from the release,
verified against its `SHA256SUMS.txt` with the machine's checksum tool,
unpacked with `tar`, and moved over the running executable (which is set
aside as `nx.exe.old` on Windows until the next upgrade). A bundled Zig,
the examples and the docs beside `nx` stay as they are. `nx upgrade
--check` only reports. An `nx` that a package manager put in place is
upgraded the way it came: the installer (it offers the upgrade), `winget
upgrade Londopy.Nexium`, `scoop update nexium`, `brew upgrade nexium`,
`pip install -U nexium-lang`, `npm update -g nexium-lang`.

You do not have to ask. Once a day, after a command has done its work,
`nx` asks GitHub for the latest release and, when there is a newer one,
says so once, on stderr:

```
nx 1.3.1 is available (this is 1.3.0): `nx upgrade` installs it; https://github.com/Londopy/nexium/releases/latest
```

The REPL's banner names it too, and says to leave with `:quit` and run
`nx upgrade` in the terminal: it is a command, not Nexium code. The answer
is kept in `~/.nexium/update-check` (`%LocalAppData%\Nexium\update-check`
on Windows, `cache/update-check` beside a portable copy, or the file
`NX_UPDATE_CACHE` names), so no command waits for more than that one
question a day, and a machine without `curl` asks once and is quiet. A
notice can therefore lag a release by up to a day and name the one before
it; `nx upgrade` asks again and installs the newest.
`NX_NO_UPDATE_CHECK=1` silences the question and the notice;
`NX_OFFLINE=1` or a `CI` variable skips the question but not an answer
already cached. `nx doctor`, `nx upgrade`, `nx lsp` and the one-liners
never print it.

## Torrents

Every file of a release is also attached as a `.torrent`, one file each,
with its GitHub download as the web seed: a BitTorrent client fetches it
from GitHub when no peer has it and from peers when they do, so a release
stays reachable through a client when a direct download is slow or
blocked. The release notes list the magnet links (they carry the same web
seed) with a QR code of each, for a phone or a machine without a browser
on the page.

## Provenance

Every asset of a release (the archives, the installer, the wheels, the npm
packages, the `.deb` and `.rpm`, the extension, the Chocolatey package)
carries a signed provenance statement, made by GitHub when the release
workflow built it, that names the repository, the commit and the workflow
run. Verify one with the GitHub CLI:

```bash
gh attestation verify nx-v1.3.0-x86_64-unknown-linux-gnu.tar.gz --owner Londopy
```

It fails for a file that was not built by this repository's workflow, or
that was changed since. The wheels and the npm packages carry the same
kind of statement on PyPI and npm (`npm audit signatures`, PyPI's
"Verified details" box).

## Verifying downloads

Every release ships `SHA256SUMS.txt` and lists the same values on the release
page.

```sh
sha256sum -c SHA256SUMS.txt --ignore-missing      # Linux
shasum -a 256 -c SHA256SUMS.txt --ignore-missing   # macOS
```

```powershell
Get-FileHash .\nexium-1.3.0-setup-x64.exe -Algorithm SHA256
```

The install script verifies automatically and refuses a mismatch.
