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

## In editors

The per-editor directories next to this one carry the configuration and a
copy of the queries in that editor's scope names:

- [`../neovim`](../neovim): a plugin that registers the parser with
  nvim-treesitter and the language server with Neovim's LSP client;
- [`../helix`](../helix): `languages.toml` with the grammar source, and
  queries with Helix's scopes (plus indents and text objects);
- [`../zed`](../zed): an extension that declares the grammar and runs
  `nx lsp`.

The tree-sitter grammar is fetched from this repository with
`subpath`/`path` = `editors/tree-sitter-nexium`.

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
