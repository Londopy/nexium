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
  file type with an icon and a "Run with Nexium" context entry, and install
  the VS Code extension if `code` is on the PATH.
- **Finish**: launch the interactive session, open the README, or open a
  console that runs `nx doctor`.
- **Start menu**: "Nexium <version> (64-bit)" opens the interactive session,
  so typing `nx` in the Windows search bar works like typing `python`.

Nothing else is required. Uninstall from Settings; it removes the files and
the PATH entry, and leaves programs you compiled alone.

Silent install for scripts: `nexium-<version>-setup-x64.exe /VERYSILENT /TASKS=addtopath`.

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

Variables: `NEXIUM_VERSION=v1.0.0` pins a release, `NEXIUM_HOME` changes the
directory, `NEXIUM_NO_MODIFY_PATH=1` leaves shell files alone,
`NEXIUM_NO_ZIG=1` never downloads Zig. Uninstall by deleting `~/.nexium` and
the three lines the script added.

## From source

Nothing but a C compiler is needed: `sh bootstrap/build.sh` (or
`.\bootstrap\build.ps1`) builds `nx0` from the C seed the compiler emits
for itself, `nx0` builds the compiler from `self/`, and the result rebuilds
itself to the same C; `nx-out/bootstrap/nx2` is the compiler. `CC` names
the C compiler for the build (default `zig cc`, `cc` on macOS); `NX_CC` or
`NX_ZIG` names the one `nx` itself runs. See `bootstrap/README.md`.

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

`nx doctor` prints which one is in effect and whether it runs. Cross-compiling
(`--target`) always uses Zig, since that is what makes it possible.

## An editor

Every editor gets the same language server: `nx lsp` speaks LSP over
stdio and gives diagnostics as you type, hover with inferred effects, go
to definition, completion and rename. The
[`editors/`](https://github.com/Londopy/nexium/tree/main/editors)
directory of the repository has the pieces for each one:

| editor | install |
| --- | --- |
| VS Code | the "Nexium" extension on the Marketplace, or the `.vsix` on every release (the Windows installer installs it when `code` is on the PATH) |
| Vim | `Plug 'Londopy/nexium', { 'rtp': 'editors/vim' }` |
| Neovim | the `editors/neovim` plugin: tree-sitter, LSP and the Vim files as fallback |
| Helix | append `editors/helix/languages.toml`, copy its queries, `hx --grammar fetch && hx --grammar build` |
| Zed | *Install Dev Extension* on `editors/zed` |
| Emacs | `editors/emacs/nexium-mode.el`; registers itself with Eglot and lsp-mode |
| Kate, KWrite, KDevelop, Qt Creator | copy `editors/kate/nexium.xml` into the KSyntaxHighlighting directory |
| JetBrains IDEs | `editors/vscode` as a TextMate bundle, `nx lsp` through LSP4IJ |
| Sublime Text | copy `editors/sublime/Nexium.sublime-syntax` into `Packages/User` |
| Notepad++ | copy `editors/notepad-plus-plus/nexium.udl.xml` into `%AppData%\Notepad++\userDefineLangs` |
| nano | include `editors/nano/nexium.nanorc` from `~/.nanorc` |

Each directory has a README with the details.

## Verifying downloads

Every release ships `SHA256SUMS.txt` and lists the same values on the release
page.

```sh
sha256sum -c SHA256SUMS.txt --ignore-missing      # Linux
shasum -a 256 -c SHA256SUMS.txt --ignore-missing   # macOS
```

```powershell
Get-FileHash .\nexium-1.0.0-setup-x64.exe -Algorithm SHA256
```

The install script verifies automatically and refuses a mismatch.
