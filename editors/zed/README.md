# Nexium for Zed

A Zed extension: tree-sitter highlighting, outline, bracket matching and
indentation from the grammar in [`../tree-sitter-nexium`](../tree-sitter-nexium),
and diagnostics, hover, go to definition, completion and rename through
`nx lsp`.

## Install as a dev extension

Zed builds extensions from source (Rust to WebAssembly), so a Rust toolchain
with the `wasm32-wasip1` target is needed once:

```sh
rustup target add wasm32-wasip1
```

Then in Zed: *Extensions* (`zed: extensions`) → *Install Dev Extension* →
choose this directory (`editors/zed` in a clone of the repository). Zed
compiles `src/lib.rs`, fetches the grammar at the commit pinned in
`extension.toml`, and starts `nx lsp` from the `PATH` for every `.nx` file.

## What is here

| file | what |
| --- | --- |
| `extension.toml` | the extension, the grammar (repository, commit, `path`), the language server |
| `src/lib.rs` | the only Rust in the repository: tells Zed to run `nx lsp` |
| `languages/nexium/config.toml` | file suffix, comments, brackets, indentation |
| `languages/nexium/highlights.scm` | Zed's scope names (`@comment.doc`, `@variant`, `@enum`, ...) |
| `languages/nexium/outline.scm` | functions, structs, enums, traits, impls, tests in the outline panel |
| `languages/nexium/brackets.scm`, `indents.scm` | matching and indentation |

The grammar entry pins a commit of this repository (the 1.0.0 release); a
new pin is needed only when the grammar changes.
