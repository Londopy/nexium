# tree-sitter-nexium

A [tree-sitter](https://tree-sitter.github.io/) grammar for Nexium, with
highlight queries. It parses every example, standard library module and
self-hosted source in the repository without an error node.

The grammar is `grammar.js`; the generated parser is checked in under
`src/`, so editors need no Node toolchain. Regenerate after editing the
grammar:

```sh
npx tree-sitter-cli generate
```

Queries: `queries/highlights.scm`. Scopes follow the nvim-treesitter and
Helix conventions (`@function`, `@type`, `@keyword.control`, ...).

## Neovim (nvim-treesitter)

Add the parser to your config and install it:

```lua
local parsers = require("nvim-treesitter.parsers").get_parser_configs()
parsers.nexium = {
  install_info = {
    url = "https://github.com/Londopy/nexium",
    location = "editors/tree-sitter-nexium",
    files = { "src/parser.c" },
    branch = "main",
  },
  filetype = "nexium",
}
vim.filetype.add({ extension = { nx = "nexium" } })
```

Then `:TSInstall nexium` and copy `queries/highlights.scm` to
`~/.config/nvim/queries/nexium/highlights.scm` (or point `runtimepath` at a
copy of the `queries` directory under a `queries/nexium` folder).

## Helix

In `languages.toml`:

```toml
[[language]]
name = "nexium"
scope = "source.nexium"
file-types = ["nx"]
comment-token = "//"
indent = { tab-width = 4, unit = "    " }
language-servers = ["nexium-lsp"]

[language-server.nexium-lsp]
command = "nx"
args = ["lsp"]

[[grammar]]
name = "nexium"
source = { git = "https://github.com/Londopy/nexium", rev = "main", subpath = "editors/tree-sitter-nexium" }
```

Then `hx --grammar fetch && hx --grammar build`, and copy
`queries/highlights.scm` to `~/.config/helix/runtime/queries/nexium/`.

## Zed

Zed extensions declare grammars in `extension.toml`:

```toml
[grammars.nexium]
repository = "https://github.com/Londopy/nexium"
path = "editors/tree-sitter-nexium"
rev = "main"
```

with a `languages/nexium/config.toml` naming the language and the `nx`
extension, and the queries copied to `languages/nexium/highlights.scm`.

## Checking the grammar

```sh
npx tree-sitter-cli parse ../../examples/*.nx ../../std/*.nx --quiet --stat
npx tree-sitter-cli query queries/highlights.scm ../../examples/tour.nx
```

The parse step needs a C compiler on the `PATH`; the CLI looks for `cc`
(`clang` or `gcc`). On Windows with only Zig, build the parser yourself and
point the CLI at it:

```sh
zig cc -shared -O2 -I src -o lib/nexium.dll src/parser.c
TREE_SITTER_LIBDIR=$PWD/lib npx tree-sitter-cli parse ../../examples/*.nx --quiet --stat
```
