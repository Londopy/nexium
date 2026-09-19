# Editor support

| editor | what | where |
| --- | --- | --- |
| Visual Studio Code | extension: highlighting, diagnostics and hover via `nx lsp`, run command | [`vscode/`](vscode) (Marketplace: "Nexium", or the `.vsix` on Releases) |
| Sublime Text 3 and 4 | syntax definition | [`sublime/Nexium.sublime-syntax`](sublime/Nexium.sublime-syntax), copy into `Packages/User` |
| Any editor with LSP | diagnostics and hover | run `nx lsp` over stdio |
| GitHub | highlighting on github.com | `.gitattributes` maps `.nx` to Zig's grammar until Linguist knows Nexium |

The two grammars are the same rules in two formats; when the language gains
syntax, change both. Scopes follow TextMate conventions so every theme colors
them.

Not yet: tree-sitter (Neovim, Helix, Zed) and Vim's own syntax format.
