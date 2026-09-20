#!/usr/bin/env python3
"""Regenerate docs/std.md from the doc comments in std/*.nx.

    python scripts/std_docs.py

Each module's leading `//` comment block becomes its description; every
`pub fn` with the `///` line above it becomes a table row. Run it whenever a
std module changes; CI does not (yet) check the file is current.
"""
import os, re

root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
std = os.path.join(root, "std")
modules = sorted(f[:-3] for f in os.listdir(std) if f.endswith(".nx"))

out = [
    "# The standard library",
    "",
    "Modules written in Nexium and embedded in the compiler. `import std.<module>`",
    "makes it available as `<module>.function(...)`; nothing to install or link.",
    "The core containers (`List`, `String`, `Map`), formatting, and the `math`,",
    "`io`, `os`, `time`, `random`, `mem`, and `process` namespaces are compiler",
    "builtins and are documented in [`language.md`](language.md).",
    "",
    "Sources are in [`std/`](../std); each module carries its own `test` blocks,",
    "run by `nx test std/<module>.nx` and by `cargo test`. This file is generated",
    "by `scripts/std_docs.py` from the doc comments.",
    "",
    "| module | what |",
    "| --- | --- |",
]
descs = {}
for name in modules:
    lines = open(os.path.join(std, name + ".nx"), encoding="utf-8").read().splitlines()
    head = []
    for l in lines:
        if l.startswith("// "):
            head.append(l[3:].strip())
        elif l.startswith("//"):
            head.append("")
        else:
            break
    first = head[0].split(":", 1)[1].strip() if head and ":" in head[0] else (head[0] if head else "")
    descs[name] = (head, first)
    out.append(f"| [`std.{name}`](#std{name}) | {first} |")
out.append("")

for name in modules:
    lines = open(os.path.join(std, name + ".nx"), encoding="utf-8").read().splitlines()
    head, _ = descs[name]
    out.append(f"## std.{name}")
    out.append("")
    para = " ".join(h for h in head if h)
    out.append(para)
    out.append("")
    # types first
    types = [l for l in lines if l.startswith("pub struct ") or l.startswith("pub enum ")]
    if types:
        out.append("Types: " + ", ".join("`" + t.split()[2] + "`" for t in types))
        out.append("")
    out.append("| function | what it does |")
    out.append("| --- | --- |")
    for i, l in enumerate(lines):
        stripped = l.strip()
        if stripped.startswith("pub fn "):
            sig = stripped[len("pub fn "):]
            sig = re.sub(r"\s*\{.*$", "", sig).strip()
            doc = lines[i - 1].strip()[4:].strip() if i > 0 and lines[i - 1].strip().startswith("/// ") else ""
            # methods are indented inside an impl block
            if l.startswith("    "):
                sig = "(method) " + sig
            sig = sig.replace("|", "\\|")
            out.append(f"| `{sig}` | {doc} |")
    out.append("")

path = os.path.join(root, "docs", "std.md")
open(path, "w", encoding="utf-8", newline="\n").write("\n".join(out))
print(f"wrote {path}: {len(modules)} modules")
