# Nexium for Visual Studio Code

Syntax highlighting for `.nx` files, plus diagnostics, hover, go to
definition, completion and rename through the
compiler's own language server (`nx lsp`).

## Install

From the Visual Studio Marketplace or Open VSX: search for "Nexium"
(publisher Londopy). The release workflow publishes each release there once
the publisher tokens are in place (see Publishing below).

From a release file: download `nexium-<version>.vsix` from the
[Releases](https://github.com/Londopy/nexium/releases) page, then in VS Code
run **Extensions: Install from VSIX...**.

For the language features, `nx` must be on your `PATH`, or set `nexium.nxPath`
in settings.

## What you get

- Highlighting for keywords, types, effects (`!allocates`), intrinsics
  (`@cImport`), binary patterns (`<<len:16/little, rest:bytes>>`), format
  placeholders, labels, and error names.
- Go to definition (F12) for functions, types, constants, locals and
  imported module members; completion after `.` for module members, fields,
  methods and error names, and for names in scope; rename (F2) of a local
  within its function or of an item across the file.
- Diagnostics on every edit and hover on functions showing the signature and
  the inferred effect set.
- A code lens above every function with its panic proof: `cannot panic`,
  or `panics: <why> at :<line>` naming the line the checker recorded as the
  witness, followed by the rest of its effects.
- **Nexium: Run Current File** in the command palette runs `nx run` in a
  terminal.
- **Nexium: Restart Language Server**.

## Settings

| setting | default | meaning |
| --- | --- | --- |
| `nexium.nxPath` | `nx` | path to the compiler |
| `nexium.enableLanguageServer` | `true` | start `nx lsp` |

## Building from source

```bash
cd editors/vscode
npm install
npm run compile
npx vsce package
```

`npx vsce package` writes `nexium-0.1.0.vsix`. Publishing to the Marketplace
is `npx vsce publish` with a publisher token.

## Publishing

The release workflow publishes the `.vsix` to the Visual Studio Marketplace
and to Open VSX when the repository has the tokens as secrets:

- `VSCE_PAT`: a personal access token from Azure DevOps with the
  Marketplace (Manage) scope, for the publisher `Londopy` created at
  https://marketplace.visualstudio.com/manage.
- `OVSX_PAT`: an access token from https://open-vsx.org (an Eclipse
  account), after the namespace is claimed once with
  `npx ovsx create-namespace Londopy -p <token>`.

Without a token the step says so and the release goes on. The extension is
packaged with the release's version (`vsce package <version>`), so each
release publishes once; `package.json` keeps its own version.
