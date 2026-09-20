#!/usr/bin/env python3
"""Write the GitHub release body and SHA256SUMS.txt for a set of assets.

    python scripts/release_notes.py v0.1.0 artifacts/ notes.md

Reads the matching CHANGELOG.md section with patchnotes, lists every asset
with its size and SHA-256, and adds install and verify instructions per
platform. SHA256SUMS.txt is written into the artifacts directory so it is
uploaded with the rest. The release's name (the italic line under the
version header, see docs/release-names.md) becomes the title, written to
title.txt next to the notes for the workflow to pick up.
"""
import hashlib, os, re, sys

import patchnotes

tag, art_dir, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
version = tag.lstrip("v")

# --- changelog section
cl = patchnotes.parse_file("CHANGELOG.md")
rel = cl.get_version(version) or cl.latest()
sections = {}
for e in rel.entries:
    sections.setdefault(str(e.change_type).split(".")[-1].title(), []).append(e.text)

# --- the release's name: `*Mountain: Place* — why` under the version header
raw = open("CHANGELOG.md", encoding="utf-8").read()
name, why = "", ""
m = re.search(r"^## \[%s\][^\n]*\n\s*\*([^*]+)\*(?: — (.*))?$" % re.escape(version), raw, re.M)
if m:
    name, why = m.group(1).strip(), (m.group(2) or "").strip()
title = f"Nexium {tag} — {name}" if name else f"Nexium {tag}"
open(os.path.join(os.path.dirname(out_path) or ".", "title.txt"), "w", encoding="utf-8", newline="\n").write(title + "\n")

# --- assets and checksums
assets = sorted(f for f in os.listdir(art_dir) if os.path.isfile(os.path.join(art_dir, f)) and f != "SHA256SUMS.txt")
sums = []
for f in assets:
    h = hashlib.sha256()
    with open(os.path.join(art_dir, f), "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    sums.append((f, h.hexdigest(), os.path.getsize(os.path.join(art_dir, f))))
with open(os.path.join(art_dir, "SHA256SUMS.txt"), "w", newline="\n") as fh:
    for f, digest, _ in sums:
        fh.write(f"{digest}  {f}\n")

def human(n):
    for unit in ("B", "KB", "MB", "GB"):
        if n < 1024 or unit == "GB":
            return f"{n:.0f} {unit}" if unit == "B" else f"{n:.1f} {unit}"
        n /= 1024

def find(sub):
    return next((f for f, _, _ in sums if sub in f), None)

win_setup = find("setup-x64.exe")
win_zip = find("windows-msvc.zip")
mac = find("apple-darwin.tar.gz")
linux = find("linux-gnu.tar.gz")
vsix = find(".vsix")

out = [f"# {title}", ""]
if name:
    out += [f"*{name}* — {why}" if why else f"*{name}*", ""]
for kind in ("Added", "Changed", "Fixed", "Removed", "Deprecated", "Security"):
    if kind in sections:
        out.append(f"### {kind}")
        out.append("")
        out.extend(f"- {t}" for t in sections[kind])
        out.append("")

out += ["## Install", ""]
if win_setup:
    out += [f"**Windows**: run [`{win_setup}`]({win_setup}). It installs `nx`, a bundled Zig toolchain (the C compiler `nx` uses), the standard library, examples, docs, and the VS Code extension, and can add `nx` to your PATH. No other install is needed. A portable zip without the installer is `{win_zip}`.", ""]
out += ["**macOS and Linux**:", "", "```sh", "curl -fsSL https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.sh | sh", "```", "",
        f"This downloads `{mac}` or `{linux}`, verifies it against `SHA256SUMS.txt`, installs to `~/.nexium/bin`, downloads Zig on Linux when no C compiler is present (macOS uses the Xcode command line tools), and adds the directory to your PATH. Set `NEXIUM_VERSION={tag}` to pin this release.", "",
        "**From source**: `sh bootstrap/build.sh` with a C compiler (Zig, or `cc` on macOS) builds the compiler from its C seed; no Rust is needed.", "",
        f"**VS Code**: install `{vsix}` with *Extensions: Install from VSIX...*, or let the Windows installer do it." if vsix else "", ""]
out += ["Then, in a new console:", "", "```", "nx doctor", "nx run examples/hello.nx", "```", ""]

out += ["## Files", "", "| file | size | SHA-256 |", "| --- | --- | --- |"]
for f, digest, size in sums:
    out.append(f"| [`{f}`]({f}) | {human(size)} | `{digest}` |")
out += ["", "`SHA256SUMS.txt` holds the same values. Verify a download with:", "",
        "```sh", "sha256sum -c SHA256SUMS.txt --ignore-missing      # Linux", "shasum -a 256 -c SHA256SUMS.txt --ignore-missing   # macOS", "```", "",
        "```powershell", "Get-FileHash .\\" + (win_setup or "nx.zip") + " -Algorithm SHA256   # Windows, compare with the table", "```", ""]
out += ["## Requirements", "", "- Windows 10 or later, x64. macOS on Apple Silicon. Linux x86_64 with glibc.",
        "- A C compiler is needed to build programs: the Windows installer and the macOS/Linux script take care of it. Otherwise put [Zig](https://ziglang.org/download/) on your PATH, or set `NX_CC`.", ""]
open(out_path, "w", encoding="utf-8", newline="\n").write("\n".join(out))
print(f"wrote {out_path} and SHA256SUMS.txt for {len(sums)} assets")
