#!/usr/bin/env python3
"""Check the relative links in the repository's Markdown.

    python scripts/check_links.py

Every tracked `.md` file is read, links in code blocks and code spans left
aside. A relative link must name a file or directory that exists, spelled
with the case it has (CI's file systems are case-sensitive), and an
`#anchor` into a Markdown file (or into the page itself) must match one of
its headings the way GitHub makes them anchors, or an explicit `<a id>`.
Links with a scheme (`https:`, `mailto:`) are not fetched, and a `.html`
link is the website's (site/build.nx makes those pages). Exits 1 and names
each broken link, with its file and line, when there is one.
"""
import os, re, subprocess, sys, unicodedata
from urllib.parse import unquote

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

INLINE = re.compile(r"!?\[(?:[^\[\]]|\[[^\[\]]*\])*\]\(\s*(<[^>]*>|[^()\s]*(?:\([^()\s]*\)[^()\s]*)*)(?:\s+(?:\"[^\"]*\"|'[^']*'))?\s*\)")
REFDEF = re.compile(r"^\s{0,3}\[[^\]]+\]:\s*(\S+)")
HEADING = re.compile(r"^(#{1,6})\s+(.*?)\s*#*\s*$")
HTML_ANCHOR = re.compile(r"<a\s+(?:[^>]*\s)?(?:id|name)\s*=\s*[\"']([^\"']+)[\"']", re.I)
SCHEME = re.compile(r"^[A-Za-z][A-Za-z0-9+.-]*:")


def tracked_markdown():
    out = subprocess.run(["git", "ls-files", "*.md"], cwd=ROOT, capture_output=True, text=True, check=True).stdout
    return [p for p in out.splitlines() if p]


def strip_code(lines):
    """The lines with fenced blocks blanked and code spans removed, numbering kept."""
    out = []
    fence = None
    for line in lines:
        m = re.match(r"^\s{0,3}(`{3,}|~{3,})", line)
        if fence:
            if m and m.group(1)[0] == fence[0] and len(m.group(1)) >= len(fence):
                fence = None
            out.append("")
            continue
        if m:
            fence = m.group(1)
            out.append("")
            continue
        out.append(re.sub(r"(`+)(?:(?!\1).)+?\1", "", line))
    return out


def heading_text(raw):
    """What GitHub shows of a heading: links as their text, code and emphasis bare."""
    t = re.sub(r"!?\[([^\]]*)\]\([^)]*\)", r"\1", raw)
    t = re.sub(r"<[^>]+>", "", t)
    t = t.replace("`", "")
    return t


def slug(text):
    """GitHub's anchor for a heading (github-slugger): lower case, spaces as
    hyphens, and every character that is not a letter, a digit, a mark, a
    hyphen or an underscore dropped."""
    out = []
    for c in text.lower():
        cat = unicodedata.category(c)
        if c in "-_" or cat[0] in "LNM":
            out.append(c)
        elif c == " ":
            out.append("-")
    return "".join(out)


_anchors = {}


def anchors_of(path):
    if path in _anchors:
        return _anchors[path]
    found = set()
    seen = {}
    try:
        lines = open(path, encoding="utf-8").read().splitlines()
    except (OSError, UnicodeDecodeError):
        _anchors[path] = found
        return found
    fence = None
    for line in lines:
        m = re.match(r"^\s{0,3}(`{3,}|~{3,})", line)
        if fence:
            if m and m.group(1)[0] == fence[0] and len(m.group(1)) >= len(fence):
                fence = None
            continue
        if m:
            fence = m.group(1)
            continue
        h = HEADING.match(line)
        if h:
            base = slug(heading_text(h.group(2)))
            n = seen.get(base, 0)
            seen[base] = n + 1
            found.add(base if n == 0 else f"{base}-{n}")
        for a in HTML_ANCHOR.findall(line):
            found.add(a)
    _anchors[path] = found
    return found


def exists_exactly(path):
    """Does the path exist with exactly this spelling, case included?"""
    rel = os.path.relpath(path, ROOT)
    if rel.startswith(".."):
        return os.path.exists(path)
    cur = ROOT
    for part in rel.replace("\\", "/").split("/"):
        if part in ("", "."):
            continue
        if part == "..":
            cur = os.path.dirname(cur)
            continue
        try:
            names = os.listdir(cur)
        except OSError:
            return False
        if part not in names:
            return False
        cur = os.path.join(cur, part)
    return True


def check_file(rel, problems):
    path = os.path.join(ROOT, rel)
    lines = open(path, encoding="utf-8").read().splitlines()
    for no, line in enumerate(strip_code(lines), 1):
        targets = [m.group(1) for m in INLINE.finditer(line)]
        d = REFDEF.match(line)
        if d:
            targets.append(d.group(1))
        for target in targets:
            if target.startswith("<") and target.endswith(">"):
                target = target[1:-1]
            if not target or SCHEME.match(target) or target.startswith("//"):
                continue
            file_part, _, anchor = target.partition("#")
            file_part = unquote(file_part.split("?", 1)[0])
            anchor = unquote(anchor)
            # a page of the website (the Topo's chapters link to each other
            # that way), made by site/build.nx: its build is where it resolves
            if file_part.endswith((".html", ".htm")):
                continue
            dest = os.path.normpath(os.path.join(os.path.dirname(path), file_part)) if file_part else path
            if file_part and not exists_exactly(dest):
                problems.append(f"{rel}:{no}: {target} (no such file)")
                continue
            if anchor and dest.endswith(".md") and os.path.isfile(dest):
                if anchor not in anchors_of(dest) and anchor.lower() not in anchors_of(dest):
                    problems.append(f"{rel}:{no}: {target} (no such heading)")


def main():
    problems = []
    files = tracked_markdown()
    for rel in files:
        check_file(rel, problems)
    for p in problems:
        print(p)
    print(f"{len(files)} Markdown files, {len(problems)} broken link(s)")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
