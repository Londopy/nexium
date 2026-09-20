# Nexium for Neovim

Tree-sitter highlighting, folds and indentation, the language server
(`nx lsp`) through Neovim's built-in LSP client, and the Vim files in
[`../vim`](../vim) as the fallback: regex syntax, indent script and the
`:NxRun` / `:NxTest` / `:NxCheck` / `:NxFmt` commands work before any
parser is installed.

## Install

With [lazy.nvim](https://github.com/folke/lazy.nvim):

```lua
{
  "Londopy/nexium",
  config = function(plugin)
    vim.opt.rtp:append(plugin.dir .. "/editors/neovim")
    require("nexium").setup()
  end,
}
```

Then `:TSInstall nexium` (needs a C compiler; nvim-treesitter master and
main are both supported). Without nvim-treesitter the Vim syntax applies.

`setup` takes `{ lsp = false }`, `{ treesitter = false }` and
`{ cmd = { "/path/to/nx", "lsp" } }`.

## What `setup` does

- adds `../vim` to the runtime path (syntax, indent, ftplugin commands);
- registers the parser with nvim-treesitter: the grammar lives in
  [`../tree-sitter-nexium`](../tree-sitter-nexium) of this repository and
  the queries in `queries/nexium/` here;
- configures the language server: `vim.lsp.config` + `vim.lsp.enable` on
  Neovim 0.11 and later, nvim-lspconfig when it is installed on older
  versions, otherwise `vim.lsp.start` on every Nexium buffer. The root
  is the directory with `nexium.toml` (or `.git`).

`:checkhealth vim.lsp` and `:checkhealth nvim-treesitter` show the result;
`nx` must be on the `PATH`.

## Queries

`queries/nexium/highlights.scm` is a copy of the tree-sitter grammar's
queries (nvim-treesitter scope names; CI checks the copy matches);
`folds.scm` folds blocks, structs, enums, traits, impls and matches;
`indents.scm` drives `=` and `o`.
