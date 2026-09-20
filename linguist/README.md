# Getting GitHub to recognize Nexium

GitHub's language detection, syntax highlighting on github.com, and the
language bar all come from [Linguist](https://github.com/github-linguist/linguist).
Adding Nexium is one pull request there. This folder holds everything that
PR needs, ready to apply, so that submitting it is a few commands once the
usage requirement is met.

## The one requirement we cannot prepare in advance

Linguist only adds languages that are already used on GitHub. Their check is
a code search for the extension across repositories that are not the
language's own, and the bar is on the order of 200 repositories, judged case
by case. A PR before that is closed with "come back when it is used", so the
order is:

1. Release Nexium and get programs written in it into public repositories
   (every user's project counts; `docs/releasing-your-program.md` and the
   `setup-nexium` action exist to make that easy).
2. When a search like `path:*.nx fn` returns a few hundred repositories that
   are not `Londopy/nexium`, submit the PR below.

Until then, `.gitattributes` in this repository maps `.nx` to Zig so files
are at least highlighted, and the README carries the honest language table.

## What the PR contains

| piece | where it comes from |
| --- | --- |
| `lib/linguist/languages.yml` entry | [`languages.yml.snippet`](languages.yml.snippet) |
| grammar submodule under `vendor/grammars/` | this repository; Linguist's `script/add-grammar` scans it and finds `editors/vscode/syntaxes/nexium.tmLanguage.json` |
| `samples/Nexium/*.nx` | real programs from `examples/`, copied by `apply.py` |
| a heuristic for `.nx` | not needed today: no language in Linguist uses `.nx`; `apply.py` adds one anyway so a future collision is already handled |
| a language id | assigned by Linguist's own `script/update-ids` |

The color is the brand teal `#4FD1C5`. Linguist requires colors to be
distinct; the nearest existing ones (MLIR `#5EC8DB`, Elm `#60B5CC`) are far
enough apart.

## Submitting

```bash
git clone https://github.com/github-linguist/linguist
cd linguist
script/bootstrap
git checkout -b add-nexium
python ../nexium/linguist/apply.py .        # edits languages.yml, adds samples and a heuristic
script/add-grammar https://github.com/Londopy/nexium
script/update-ids
bundle exec rake samples
bundle exec rake test
```

Then commit and open the PR. Its description should contain:

- a link to the language: https://github.com/Londopy/nexium
- the usage search link and its count
- the grammar's license (MIT, in `editors/vscode/LICENSE`)
- one sentence on why `.nx` is safe: no existing language claims it

Linguist's contributing guide is the authority if anything here has changed:
https://github.com/github-linguist/linguist/blob/main/CONTRIBUTING.md
