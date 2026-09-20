# Nexium for Vim

Syntax highlighting, file type detection, comment and indent settings, and
`:NxRun` / `:NxTest` / `:NxCheck` / `:NxFmt` for the current file. `:make`
runs `nx check` and fills the quickfix list from its diagnostics.

## Install

With a plugin manager, point it at this directory of the repository:

```vim
" vim-plug
Plug 'Londopy/nexium', { 'rtp': 'editors/vim' }
```

```lua
-- lazy.nvim (Neovim; see ../neovim for tree-sitter and LSP as well)
{ "Londopy/nexium", config = function() vim.opt.rtp:append(vim.fn.stdpath("data") .. "/lazy/nexium/editors/vim") end }
```

By hand: copy `syntax/`, `ftdetect/`, `ftplugin/` and `indent/` into
`~/.vim/` (or `~/vimfiles/` on Windows).

## The language server

`nx lsp` speaks the Language Server Protocol over stdio. With
[vim-lsp](https://github.com/prabirshrestha/vim-lsp):

```vim
if executable('nx')
  au User lsp_setup call lsp#register_server({
    \ 'name': 'nexium',
    \ 'cmd': {server_info->['nx', 'lsp']},
    \ 'allowlist': ['nexium'],
    \ })
endif
```

With [coc.nvim](https://github.com/neoclide/coc.nvim), in `coc-settings.json`:

```json
{ "languageserver": { "nexium": { "command": "nx", "args": ["lsp"], "filetypes": ["nexium"] } } }
```

With [ALE](https://github.com/dense-analysis/ale):

```vim
let g:ale_linters = { 'nexium': ['nexium_lsp'] }
call ale#linter#Define('nexium', {
  \ 'name': 'nexium_lsp', 'lsp': 'stdio', 'executable': 'nx',
  \ 'command': '%e lsp', 'project_root': '.' })
```

The server provides diagnostics on every change, hover with inferred
effects, go to definition, completion and rename.
