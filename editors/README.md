# Editor support

| editor | what | where |
| --- | --- | --- |
| Visual Studio Code | extension: highlighting, diagnostics, hover, go to definition, completion and rename via `nx lsp`, run command | [`vscode/`](vscode) (Marketplace: "Nexium", or the `.vsix` on Releases) |
| Vim | syntax, indent, `:NxRun` `:NxTest` `:NxCheck` `:NxFmt`, `:make` with quickfix; LSP via vim-lsp, coc or ALE | [`vim/`](vim) |
| Neovim | the Vim files plus tree-sitter parser registration, queries (highlights, folds, indents) and `nx lsp` through the built-in client | [`neovim/`](neovim) |
| Helix | language config, tree-sitter queries in Helix's scopes (highlights, indents, text objects), `nx lsp` | [`helix/`](helix) |
| Zed | extension: grammar, highlights, outline, brackets, indents, `nx lsp` | [`zed/`](zed) |
| Emacs | `nexium-mode`: font lock, indentation, `nexium-run` and friends, Eglot and lsp-mode registration | [`emacs/`](emacs) |
| Kate, KWrite, KDevelop, Qt Creator | KSyntaxHighlighting definition; LSP client settings for Kate | [`kate/`](kate) |
| JetBrains IDEs | the VS Code grammar as a TextMate bundle; `nx lsp` through LSP4IJ | [`jetbrains/`](jetbrains) |
| Sublime Text 3 and 4 | syntax definition | [`sublime/Nexium.sublime-syntax`](sublime/Nexium.sublime-syntax), copy into `Packages/User` |
| Notepad++ | User Defined Language | [`notepad-plus-plus/`](notepad-plus-plus) |
| nano | syntax file | [`nano/`](nano) |
| Any editor with LSP | diagnostics, hover, go to definition, completion, rename | run `nx lsp` over stdio |
| GitHub | highlighting on github.com | `.gitattributes` maps `.nx` to Zig's grammar until Linguist knows Nexium |

Three families of highlighting rules live here. The TextMate grammar
(`vscode/`) and the Sublime syntax are the same rules in two formats; the
Vim, Emacs, Kate, nano and Notepad++ files are hand-written from the same
token list; and the tree-sitter grammar (`tree-sitter-nexium/`) is a real
parser of the language, whose queries are copied into `neovim/`, `helix/`
and `zed/` with each editor's scope names. When the language gains syntax,
change them all; CI checks that the tree-sitter grammar parses every
example and std module, and that the query copies match.

The language server is the same everywhere: `nx lsp` speaks LSP over
stdio, and provides diagnostics on every change, hover with inferred
effects, go to definition, completion and rename.
