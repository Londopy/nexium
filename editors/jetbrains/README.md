# Nexium in JetBrains IDEs

IntelliJ IDEA, CLion, PyCharm, RustRover, WebStorm and the rest read
TextMate bundles and, with a plugin, speak the Language Server Protocol, so
Nexium needs no plugin of its own.

## Highlighting

The VS Code extension in [`../vscode`](../vscode) is a TextMate bundle.
*Settings → Editor → TextMate Bundles → +* and choose that directory (a
clone of the repository, or the unzipped `.vsix` from a release: rename it
to `.zip` and take the `extension/` folder inside). `.nx` files are then
highlighted with the IDE's color scheme mapping of TextMate scopes.

## The language server

[LSP4IJ](https://plugins.jetbrains.com/plugin/23257-lsp4ij) (free, from
Red Hat) adds language servers to every JetBrains IDE. After installing it:
*Settings → Languages & Frameworks → Language Servers → +*

- Name: `Nexium`
- Command: `nx lsp`
- Mappings → File name patterns: `*.nx`, language id `nexium`

`nx` must be on the `PATH` (or give the full path in the command). The
server provides diagnostics on every change, hover with inferred effects,
go to definition, completion and rename.

IntelliJ IDEA Ultimate's own LSP API is available only to plugins, which
is why LSP4IJ is the route here.

## Running nx

*Settings → Tools → External Tools → +*: program `nx`, arguments
`run "$FilePath$"`, working directory `$FileDir$`. The tool appears under
*Tools → External Tools* and can be given a shortcut in the keymap.
