# Editor support

| editor | what | where |
| --- | --- | --- |
| Visual Studio Code | extension: highlighting, diagnostics and hover via `nx lsp`, run command | [`vscode/`](vscode) (Marketplace: "Nexium", or the `.vsix` on Releases) |
| Sublime Text 3 and 4 | syntax definition | [`sublime/Nexium.sublime-syntax`](sublime/Nexium.sublime-syntax), copy into `Packages/User` |
| Neovim, Helix, Zed | tree-sitter grammar and highlight queries | [`tree-sitter-nexium/`](tree-sitter-nexium), install steps in its README |
| Any editor with LSP | diagnostics and hover | run `nx lsp` over stdio |
| GitHub | highlighting on github.com | `.gitattributes` maps `.nx` to Zig's grammar until Linguist knows Nexium |

The TextMate and Sublime grammars are the same rules in two formats, and the
tree-sitter grammar is a real parser of the language; when the language
gains syntax, change all three (CI checks that the tree-sitter grammar
parses every example and std module). Scopes follow the usual conventions so
every theme colors them.

Not yet: Vim's own syntax format.
