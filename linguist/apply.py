#!/usr/bin/env python3
"""Apply the Nexium language entry to a Linguist checkout.

    python apply.py /path/to/linguist

Inserts the entry from languages.yml.snippet into lib/linguist/languages.yml
in alphabetical order, copies sample programs into samples/Nexium/, and adds
a heuristic for `.nx` to lib/linguist/heuristics.yml. Idempotent.
"""
import os, re, shutil, sys

here = os.path.dirname(os.path.abspath(__file__))
repo = os.path.dirname(here)
if len(sys.argv) != 2:
    sys.exit(__doc__)
linguist = sys.argv[1]
langs = os.path.join(linguist, "lib", "linguist", "languages.yml")
heur = os.path.join(linguist, "lib", "linguist", "heuristics.yml")
if not os.path.exists(langs):
    sys.exit(f"not a linguist checkout: {linguist}")

# 1. languages.yml
entry = "".join(l for l in open(os.path.join(here, "languages.yml.snippet"), encoding="utf-8") if not l.startswith("#"))
text = open(langs, encoding="utf-8").read()
if re.search(r"^Nexium:$", text, re.M):
    print("languages.yml: Nexium already present")
else:
    # split into top-level blocks (a line starting at column 0 begins one)
    blocks = re.split(r"(?m)^(?=[^\s#])", text)
    head, blocks = blocks[0], blocks[1:]
    blocks.append(entry if entry.endswith("\n") else entry + "\n")
    blocks.sort(key=lambda b: b.split(":", 1)[0].strip('"').lower())
    open(langs, "w", encoding="utf-8", newline="\n").write(head + "".join(blocks))
    print("languages.yml: Nexium added")

# 2. samples
dst = os.path.join(linguist, "samples", "Nexium")
os.makedirs(dst, exist_ok=True)
for name in ["tour.nx", "binary.nx", "ownership.nx", "generics.nx", "cimport.nx", "parallel.nx"]:
    shutil.copy(os.path.join(repo, "examples", name), os.path.join(dst, name))
shutil.copy(os.path.join(repo, "self", "lexer.nx"), os.path.join(dst, "lexer.nx"))
print(f"samples: {len(os.listdir(dst))} files in samples/Nexium/")

# 3. heuristic: `.nx` is unclaimed today; this keeps Nexium winning if another
# language ever claims the extension. Nexium files declare `fn`, `struct`,
# `enum`, `trait`, `import`, `test`, or `artifact` at column 0.
rule = """- extensions: ['.nx']
  rules:
  - language: Nexium
    pattern: '^\\s*(fn|pub\\s+fn|struct|enum|trait|impl|import|test|artifact|error|const|var)\\s'
"""
h = open(heur, encoding="utf-8").read()
if "language: Nexium" in h:
    print("heuristics.yml: Nexium rule already present")
else:
    m = re.search(r"^disambiguations:\n", h, re.M)
    if not m:
        sys.exit("heuristics.yml: no disambiguations section")
    h = h[: m.end()] + rule + h[m.end():]
    open(heur, "w", encoding="utf-8", newline="\n").write(h)
    print("heuristics.yml: rule added")

print("next: script/add-grammar https://github.com/Londopy/nexium && script/update-ids && bundle exec rake test")
