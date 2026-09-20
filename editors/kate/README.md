# Nexium for Kate, KWrite, KDevelop and Qt Creator

`nexium.xml` is a [KSyntaxHighlighting](https://api.kde.org/frameworks/syntax-highlighting/html/)
definition: comments and doc comments, keywords, effects, intrinsics,
error names, variants, labels, strings with escapes and placeholders,
binary patterns, numbers, and brace folding. The C-style indenter applies.

## Install

Copy the file into the framework's user directory and restart the editor:

| platform | directory |
| --- | --- |
| Linux, BSD | `~/.local/share/org.kde.syntax-highlighting/syntax/` |
| Windows | `%USERPROFILE%\AppData\Local\org.kde.syntax-highlighting\syntax\` |
| macOS | `~/Library/Application Support/org.kde.syntax-highlighting/syntax/` |

Qt Creator reads the same directory for its generic highlighter
(*Preferences → Text Editor → Generic Highlighter*).

## The language server in Kate

Kate's LSP client takes new servers in *Settings → Configure Kate → LSP
Client → User Server Settings*:

```json
{
  "servers": {
    "nexium": {
      "command": ["nx", "lsp"],
      "highlightingModeRegex": "^Nexium$",
      "rootIndicationFileNames": ["nexium.toml"]
    }
  }
}
```

`nx` must be on the `PATH`. The server provides diagnostics on every change,
hover with inferred effects, go to definition, completion and rename.
