# Nexium for Helix

Tree-sitter highlighting, indentation and text objects, plus diagnostics,
hover, go to definition, completion and rename through `nx lsp`.

## Install

1. Append [`languages.toml`](languages.toml) to `~/.config/helix/languages.toml`
   (`%AppData%\helix\languages.toml` on Windows). It declares the language,
   the server (`nx lsp`) and where the grammar comes from.
2. Copy the queries into Helix's runtime directory:

   ```sh
   mkdir -p ~/.config/helix/runtime/queries/nexium
   cp queries/nexium/*.scm ~/.config/helix/runtime/queries/nexium/
   ```

3. Fetch and build the grammar:

   ```sh
   hx --grammar fetch && hx --grammar build
   ```

`hx --health nexium` then lists the grammar, the queries and the server.
`nx` must be on the `PATH` for the language server to start.

The grammar entry pins a commit of this repository (the 1.0.0 release);
any later commit of `main` works, and a new pin is needed only when the
grammar changes.

## What the queries give

- `highlights.scm`: Helix's scope names (`keyword.control.conditional`,
  `constant.numeric.integer`, `variable.other.member`, `type.enum.variant`,
  ...), so every theme colors Nexium.
- `indents.scm`: blocks, field lists, argument lists and literals indent;
  closing brackets outdent.
- `textobjects.scm`: `mif`/`maf` select a function, `mit`/`mat` a struct,
  enum, trait or impl, `mia` an argument or parameter, `mic` a comment,
  `miT` a test.
