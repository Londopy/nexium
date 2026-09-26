# Translations

English is the source of truth; these mirror it by file name.

| language | README | language reference | how it works |
| --- | --- | --- | --- |
| English | [README.md](../../README.md) | [language.md](../language.md) | [architecture.md](../architecture.md) |
| Español | [es/README.md](es/README.md) | [es/language.md](es/language.md) | [es/architecture.md](es/architecture.md) |
| 简体中文 | [zh-CN/README.md](zh-CN/README.md) | [zh-CN/language.md](zh-CN/language.md) | [zh-CN/architecture.md](zh-CN/architecture.md) |
| 日本語 | [ja/README.md](ja/README.md) | [ja/language.md](ja/language.md) | [ja/architecture.md](ja/architecture.md) |
| 한국어 | [ko/README.md](ko/README.md) | | |
| Français | [fr/README.md](fr/README.md) | | |
| Deutsch | [de/README.md](de/README.md) | | |

Not yet translated: `embedding.md`, `gui.md`, `releasing-your-program.md`,
`DECISIONS.md`, `CONTRIBUTING.md`. Code, commands, error messages, and
identifiers stay in English everywhere, because they are what you type and
what the compiler prints; only the comments inside code blocks are translated.

To add or update a translation: copy the English file, keep every code block
byte for byte apart from its comments, translate the prose, and open a pull
request. The link checker in CI (`scripts/check_links.py`) verifies that every
relative link resolves, and that every `#anchor` names a heading of the page it
points into: a translated heading makes a translated anchor.
