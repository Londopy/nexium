#!/usr/bin/env python3
"""Wheels and npm packages that carry the compiler, from a release's archives.

    python scripts/pypi_npm.py v1.0.3 artifacts/

For every `nx-<tag>-<target>.tar.gz` or `.zip` in the directory:

- a wheel `nexium_lang-<version>-py3-none-<platform>.whl`, the package
  `nexium_lang` with the binary under `bin/` and a console script `nx` that
  hands over to it, so `pip install nexium-lang` gives `nx`;
- an npm platform package `@nexium-lang/<os>-<cpu>` with the binary and
  `os`/`cpu` fields, plus one `nexium-lang` package whose `bin/nx.js` finds
  the platform package (the shape esbuild uses: the platform packages are
  optional dependencies, npm installs the one that fits).

The wheels and the `npm pack` tarballs land in the same directory, so the
release attaches them; the release job uploads them to PyPI and npm when
`PYPI_TOKEN` and `NPM_TOKEN` are set. The names follow decision 91:
`nexium` is taken on both registries by unrelated projects.
"""
import base64, hashlib, io, json, os, shutil, subprocess, sys, tarfile, zipfile

REPO = "Londopy/nexium"
tag, art_dir = sys.argv[1], sys.argv[2]
version = tag.lstrip("v")
root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
summary = "The Nexium language: a compiler that emits C and ships libraries, packages and tools"

# target -> (wheel platform tag, npm os, npm cpu, binary name)
TARGETS = {
    "x86_64-pc-windows-msvc": ("win_amd64", "win32", "x64", "nx.exe"),
    "aarch64-pc-windows-msvc": ("win_arm64", "win32", "arm64", "nx.exe"),
    "aarch64-apple-darwin": ("macosx_11_0_arm64", "darwin", "arm64", "nx"),
    "x86_64-unknown-linux-gnu": ("manylinux_2_17_x86_64", "linux", "x64", "nx"),
    "aarch64-unknown-linux-gnu": ("manylinux_2_17_aarch64", "linux", "arm64", "nx"),
}

def read_archive(path, binary):
    """The binary and the three text files inside a release archive."""
    out = {}
    wanted = {binary, "README.md", "LICENSE", "CHANGELOG.md"}
    if path.endswith(".zip"):
        with zipfile.ZipFile(path) as z:
            for n in z.namelist():
                base = n.split("/")[-1]
                if base in wanted and "/" not in n.strip("./"):
                    out[base] = z.read(n)
    else:
        with tarfile.open(path) as t:
            for m in t.getmembers():
                base = m.name.split("/")[-1]
                if m.isfile() and base in wanted and m.name.lstrip("./").count("/") == 0:
                    out[base] = t.extractfile(m).read()
    if binary not in out:
        sys.exit(f"pypi_npm.py: no {binary} in {path}")
    return out

# ------------------------------------------------------------------- wheels
INIT_PY = '''"""The Nexium language: the compiler, nx, as a Python package.

`pip install nexium-lang` puts `nx` on the PATH of the environment; this
module hands over to the binary in its `bin/` directory.
"""
import os
import subprocess
import sys

def binary():
    return os.path.join(os.path.dirname(os.path.abspath(__file__)), "bin", "nx.exe" if os.name == "nt" else "nx")

def main():
    b = binary()
    if os.name == "nt":
        raise SystemExit(subprocess.call([b] + sys.argv[1:]))
    os.execv(b, [b] + sys.argv[1:])

if __name__ == "__main__":
    main()
'''

def record_line(name, data):
    digest = base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b"=").decode()
    return f"{name},sha256={digest},{len(data)}"

def wheel(target, files):
    plat, _, _, binary = TARGETS[target]
    name = f"nexium_lang-{version}-py3-none-{plat}.whl"
    dist = f"nexium_lang-{version}.dist-info"
    metadata = "\n".join([
        "Metadata-Version: 2.1", "Name: nexium-lang", f"Version: {version}", f"Summary: {summary}",
        f"Home-page: https://londopy.github.io/nexium/", "Author: Londopy", "License: MIT",
        f"Project-URL: Source, https://github.com/{REPO}", f"Project-URL: Changelog, https://github.com/{REPO}/blob/main/CHANGELOG.md",
        "Requires-Python: >=3.8", "Classifier: License :: OSI Approved :: MIT License",
        "Classifier: Programming Language :: Other", "Classifier: Topic :: Software Development :: Compilers",
        "Description-Content-Type: text/markdown", "", files["README.md"].decode("utf-8", "replace"),
    ]) + "\n"
    entries = [
        ("nexium_lang/__init__.py", INIT_PY.encode(), 0o644),
        (f"nexium_lang/bin/{binary}", files[binary], 0o755),
        ("nexium_lang/LICENSE", files["LICENSE"], 0o644),
        (f"{dist}/METADATA", metadata.encode(), 0o644),
        (f"{dist}/WHEEL", f"Wheel-Version: 1.0\nGenerator: scripts/pypi_npm.py\nRoot-Is-Purelib: false\nTag: py3-none-{plat}\n".encode(), 0o644),
        (f"{dist}/entry_points.txt", b"[console_scripts]\nnx = nexium_lang:main\n", 0o644),
    ]
    record = "\n".join(record_line(n, d) for n, d, _ in entries) + f"\n{dist}/RECORD,,\n"
    entries.append((f"{dist}/RECORD", record.encode(), 0o644))
    path = os.path.join(art_dir, name)
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as z:
        for n, d, mode in entries:
            info = zipfile.ZipInfo(n, date_time=(2020, 1, 1, 0, 0, 0))
            info.external_attr = (mode | 0o100000) << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(info, d)
    print("wrote", name)
    return name

# ---------------------------------------------------------------------- npm
NX_JS = '''#!/usr/bin/env node
// nexium-lang: hands over to the compiler in the platform package that
// matches this machine (an optional dependency npm picked at install time).
"use strict";
const { spawnSync } = require("child_process");
const key = process.platform + "-" + process.arch;
const packages = %PACKAGES%;
const name = packages[key];
if (!name) {
  console.error("nexium-lang: no binary is built for " + key + "; see https://londopy.github.io/nexium/docs/install.html for the other ways in");
  process.exit(1);
}
let bin;
try {
  bin = require.resolve(name + "/" + (process.platform === "win32" ? "nx.exe" : "nx"));
} catch (e) {
  console.error("nexium-lang: the platform package " + name + " is not installed; optional dependencies must be allowed (npm install without --no-optional)");
  process.exit(1);
}
const r = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
if (r.error) { console.error("nexium-lang: " + r.error.message); process.exit(1); }
process.exit(r.status === null ? 1 : r.status);
'''

def npm_packages(built):
    """The platform packages for the archives seen, and the main package."""
    npm = shutil.which("npm") or shutil.which("npm.cmd")
    if not npm:
        print("npm is not installed: the npm packages are not built")
        return []
    work = os.path.join(art_dir, "npm-work")
    shutil.rmtree(work, ignore_errors=True)
    tarballs = []
    mapping = {}
    for target, files in built.items():
        _, os_name, cpu, binary = TARGETS[target]
        pkg = f"@nexium-lang/{os_name}-{cpu}"
        mapping[f"{os_name}-{cpu}"] = pkg
        d = os.path.join(work, f"{os_name}-{cpu}")
        os.makedirs(d)
        with open(os.path.join(d, binary), "wb") as fh:
            fh.write(files[binary])
        os.chmod(os.path.join(d, binary), 0o755)
        for n in ("LICENSE", "README.md"):
            with open(os.path.join(d, n), "wb") as fh:
                fh.write(files[n])
        meta = {
            "name": pkg, "version": version, "description": f"The Nexium compiler for {os_name} {cpu}, used by the nexium-lang package",
            "license": "MIT", "repository": {"type": "git", "url": f"https://github.com/{REPO}"},
            "homepage": "https://londopy.github.io/nexium/", "os": [os_name], "cpu": [cpu], "files": [binary, "LICENSE", "README.md"],
        }
        with open(os.path.join(d, "package.json"), "w", encoding="utf-8", newline="\n") as fh:
            json.dump(meta, fh, indent=2)
            fh.write("\n")
        tarballs.append(pack(npm, d))
    d = os.path.join(work, "nexium-lang")
    os.makedirs(os.path.join(d, "bin"))
    with open(os.path.join(d, "bin", "nx.js"), "w", encoding="utf-8", newline="\n") as fh:
        fh.write(NX_JS.replace("%PACKAGES%", json.dumps(mapping, indent=2)))
    any_files = next(iter(built.values()))
    for n in ("LICENSE", "README.md"):
        with open(os.path.join(d, n), "wb") as fh:
            fh.write(any_files[n])
    meta = {
        "name": "nexium-lang", "version": version, "description": summary, "license": "MIT",
        "repository": {"type": "git", "url": f"https://github.com/{REPO}"}, "homepage": "https://londopy.github.io/nexium/",
        "bugs": f"https://github.com/{REPO}/issues", "keywords": ["nexium", "compiler", "language", "c"],
        "bin": {"nx": "bin/nx.js"}, "files": ["bin", "LICENSE", "README.md"],
        "optionalDependencies": {pkg: version for pkg in mapping.values()},
        "engines": {"node": ">=16"},
    }
    with open(os.path.join(d, "package.json"), "w", encoding="utf-8", newline="\n") as fh:
        json.dump(meta, fh, indent=2)
        fh.write("\n")
    tarballs.append(pack(npm, d))
    shutil.rmtree(work, ignore_errors=True)
    return tarballs

def pack(npm, d):
    out = subprocess.run([npm, "pack", "--silent", "--pack-destination", os.path.abspath(art_dir)], cwd=d, capture_output=True, text=True, encoding="utf-8")
    if out.returncode != 0:
        sys.exit(f"npm pack failed in {d}:\n{out.stderr}")
    name = out.stdout.strip().splitlines()[-1]
    print("wrote", name)
    return name

# -------------------------------------------------------------------- main
built = {}
for f in sorted(os.listdir(art_dir)):
    for target, (_, _, _, binary) in TARGETS.items():
        if f in (f"nx-{tag}-{target}.tar.gz", f"nx-{tag}-{target}.zip"):
            built[target] = read_archive(os.path.join(art_dir, f), binary)
if not built:
    sys.exit(f"pypi_npm.py: no nx-{tag}-<target> archive in {art_dir}")
wheels = [wheel(t, files) for t, files in built.items()]
tarballs = npm_packages(built)
print(f"{len(wheels)} wheel(s), {len(tarballs)} npm package(s) for {tag}")
