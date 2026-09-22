# The site's API

The site is static files, and so is its API: JSON documents written by
`site/build.nx` at every deploy, under `https://londopy.github.io/nexium/api/`.
No key, no rate limit beyond GitHub Pages', CORS open, and each document
is regenerated from the repository, so it says what the tree says.

| document | what it holds |
| --- | --- |
| [`api/index.json`](https://londopy.github.io/nexium/api/index.json) | the list below, as data |
| [`api/latest.json`](https://londopy.github.io/nexium/api/latest.json) | the current release: version, name, date, tag, the download URL of every asset, the checksums file, the registry names |
| [`api/releases.json`](https://londopy.github.io/nexium/api/releases.json) | every release in the changelog: version, name, date, the release page |
| [`api/std.json`](https://londopy.github.io/nexium/api/std.json) | the standard library: every module, every public function with its signature and its doc comment |
| [`api/topo.json`](https://londopy.github.io/nexium/api/topo.json) | the Topo: every chapter with its title and page, and its exercises with their kinds |
| [`api/commands.json`](https://londopy.github.io/nexium/api/commands.json) | the commands and options of `nx`, one line each (the table `nx completions` and `nx man` come from) |

Uses it was made for: an editor extension or a tool checking for a newer
release (`latest.json` is what `nx doctor` asks GitHub for, without the
API rate limit), an install script that wants the asset URLs without
guessing them, a package repository's bot, a completion or documentation
tool that wants the standard library's index, and a course tracker
reading the Topo's chapters.

```bash
curl -s https://londopy.github.io/nexium/api/latest.json | jq .version
curl -s https://londopy.github.io/nexium/api/std.json | jq '.modules[] | select(.name == "fs") | .functions[].name'
```

The documents are versioned by the release they describe (`latest.json`
carries `version`); the shape of each may gain fields in a minor and loses
none until a major, the same rule as the language.
