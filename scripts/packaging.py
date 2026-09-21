#!/usr/bin/env python3
"""Write the package-manager manifests for a release from its checksums.

    python scripts/packaging.py v1.0.2 artifacts/SHA256SUMS.txt

- `Formula/nexium.rb`: this repository is a Homebrew tap
  (`brew tap londopy/tap https://github.com/Londopy/nexium`). The release
  build where there is one (Apple Silicon, Linux x86-64 and aarch64); on any
  other machine the formula builds the compiler from its one C file,
  `bootstrap/nx.c`, out of the source tarball of the tag.
- `bucket/nexium.json`: this repository is a Scoop bucket, and the manifest
  installs by URL as well. The portable zip, without Zig.
- `installers/winget/`: the three winget manifests for `Londopy.Nexium`,
  the Inno Setup installer. They are submitted to microsoft/winget-pkgs by
  hand (a new package needs a person's pull request); until then they
  install with `winget install --manifest installers/winget`.

The release workflow runs this after the release is published and commits
the result to main. The source tarball's checksum is computed from GitHub
unless NX_SOURCE_SHA256 is set (tests set it).
"""
import hashlib, json, os, subprocess, sys, urllib.request
from datetime import date

REPO = "Londopy/nexium"
tag, sums_path = sys.argv[1], sys.argv[2]
version = tag.lstrip("v")
root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
download = f"https://github.com/{REPO}/releases/download/{tag}"

sums = {}
for line in open(sums_path, encoding="utf-8"):
    parts = line.split()
    if len(parts) == 2:
        sums[parts[1]] = parts[0]

def sha(name):
    if name not in sums:
        sys.exit(f"packaging.py: {name} is not in {sums_path}")
    return sums[name]

mac_arm = f"nx-{tag}-aarch64-apple-darwin.tar.gz"
linux_x64 = f"nx-{tag}-x86_64-unknown-linux-gnu.tar.gz"
linux_arm = f"nx-{tag}-aarch64-unknown-linux-gnu.tar.gz"
win_x64 = f"nx-{tag}-x86_64-pc-windows-msvc.zip"
win_arm = f"nx-{tag}-aarch64-pc-windows-msvc.zip"
setup = f"nexium-{version}-setup-x64.exe"

source_url = f"https://github.com/{REPO}/archive/refs/tags/{tag}.tar.gz"
source_sha = os.environ.get("NX_SOURCE_SHA256")
if not source_sha:
    h = hashlib.sha256()
    with urllib.request.urlopen(source_url) as r:
        for chunk in iter(lambda: r.read(1 << 20), b""):
            h.update(chunk)
    source_sha = h.hexdigest()

try:
    release_date = subprocess.run(["git", "log", "-1", "--format=%cs", tag], capture_output=True, text=True, cwd=root).stdout.strip() or str(date.today())
except OSError:
    release_date = str(date.today())

def write(rel, text):
    path = os.path.join(root, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    print("wrote", rel)

# ------------------------------------------------------------------ Homebrew
write("Formula/nexium.rb", f'''# The Nexium language, for Homebrew. Written by scripts/packaging.py at each
# release; this repository is the tap:
#
#     brew tap londopy/tap https://github.com/{REPO}
#     brew install londopy/tap/nexium
#
# The release build where there is one (Apple Silicon, Linux x86-64 and
# aarch64); on any other machine, the compiler's own C, bootstrap/nx.c, built
# from the source tarball by the system compiler.
class Nexium < Formula
  desc "Nexium language: a compiler that emits C and ships libraries, packages and tools"
  homepage "https://londopy.github.io/nexium/"
  url "{source_url}"
  sha256 "{source_sha}"
  license "MIT"

  on_macos do
    on_arm do
      url "{download}/{mac_arm}"
      sha256 "{sha(mac_arm)}"
    end
  end

  on_linux do
    on_intel do
      url "{download}/{linux_x64}"
      sha256 "{sha(linux_x64)}"
    end
    on_arm do
      url "{download}/{linux_arm}"
      sha256 "{sha(linux_arm)}"
    end
  end

  def install
    unless File.exist?("nx")
      # no release for this machine: the one C file
      libs = OS.mac? ? [] : ["-lm", "-lc", "-lpthread"]
      system ENV.cc, "-std=gnu11", "-O2", "-w", "-fno-strict-aliasing", "-o", "nx", "bootstrap/nx.c", *libs
    end
    bin.install "nx"
    pkgshare.install "examples", "std", "docs"
    doc.install "README.md", "CHANGELOG.md"
  end

  def caveats
    <<~EOS
      nx builds programs with a C compiler: the Xcode command line tools on
      macOS (xcode-select --install), gcc, clang or zig on Linux; `nx doctor`
      says which one it found. The examples are in #{{pkgshare}}/examples.
    EOS
  end

  test do
    (testpath/"hello.nx").write "fn main() {{ println(\\"hi\\", .{{}}) }}\\n"
    assert_match "hi", shell_output("#{{bin}}/nx run hello.nx")
  end
end
''')

# --------------------------------------------------------------------- Scoop
scoop = {
    "version": version,
    "description": "The Nexium language: a compiler that emits C and ships libraries, packages and tools",
    "homepage": "https://londopy.github.io/nexium/",
    "license": "MIT",
    "notes": [
        "nx builds programs with a C compiler: Zig on the PATH (scoop install zig), or the full installer from the Releases page bundles one.",
        "nx doctor says what it found. The examples are next to nx.exe.",
    ],
    "architecture": {
        "64bit": {"url": f"{download}/{win_x64}", "hash": sha(win_x64)},
        "arm64": {"url": f"{download}/{win_arm}", "hash": sha(win_arm)},
    },
    "bin": "nx.exe",
    "suggest": {"C compiler": ["zig"]},
    "checkver": {"github": f"https://github.com/{REPO}"},
    "autoupdate": {
        "architecture": {
            "64bit": {"url": f"https://github.com/{REPO}/releases/download/v$version/nx-v$version-x86_64-pc-windows-msvc.zip"},
            "arm64": {"url": f"https://github.com/{REPO}/releases/download/v$version/nx-v$version-aarch64-pc-windows-msvc.zip"},
        },
        "hash": {"url": f"https://github.com/{REPO}/releases/download/v$version/SHA256SUMS.txt"},
    },
}
write("bucket/nexium.json", json.dumps(scoop, indent=4) + "\n")

# -------------------------------------------------------------------- winget
ident = "Londopy.Nexium"
write(f"installers/winget/{ident}.yaml", f'''# yaml-language-server: $schema=https://aka.ms/winget-manifest.version.1.6.0.schema.json
# Written by scripts/packaging.py at each release.
PackageIdentifier: {ident}
PackageVersion: {version}
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.6.0
''')
write(f"installers/winget/{ident}.installer.yaml", f'''# yaml-language-server: $schema=https://aka.ms/winget-manifest.installer.1.6.0.schema.json
PackageIdentifier: {ident}
PackageVersion: {version}
InstallerType: inno
Scope: user
InstallModes:
  - interactive
  - silent
  - silentWithProgress
InstallerSwitches:
  Silent: /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /TASKS=addtopath
  SilentWithProgress: /SILENT /SUPPRESSMSGBOXES /NORESTART /TASKS=addtopath
UpgradeBehavior: install
ProductCode: '{{B7E4C2F1-7A5D-4B7E-9D1C-3E2A9F0C5E11}}_is1'
ReleaseDate: {release_date}
Installers:
  - Architecture: x64
    InstallerUrl: {download}/{setup}
    InstallerSha256: {sha(setup).upper()}
ManifestType: installer
ManifestVersion: 1.6.0
''')
write(f"installers/winget/{ident}.locale.en-US.yaml", f'''# yaml-language-server: $schema=https://aka.ms/winget-manifest.defaultLocale.1.6.0.schema.json
PackageIdentifier: {ident}
PackageVersion: {version}
PackageLocale: en-US
Publisher: Londopy
PublisherUrl: https://github.com/Londopy
PublisherSupportUrl: https://github.com/{REPO}/issues
PackageName: Nexium
PackageUrl: https://londopy.github.io/nexium/
License: MIT
LicenseUrl: https://github.com/{REPO}/blob/main/LICENSE
Copyright: Copyright (c) Londopy
ShortDescription: "The Nexium language: a compiler that emits C and ships libraries, packages and tools"
Description: |-
  Nexium compiles to native code through C, has reference counting without a
  tracing collector, a checked effect system that says whether a function
  allocates, blocks or can panic, and a compiler that turns one source tree
  into a C library, a Python wheel, a Rust crate, an npm package or a command
  line tool. The installer bundles the Zig toolchain nx uses as its C
  compiler, the standard library, the examples, the docs and the VS Code
  extension.
Moniker: nexium
Tags:
  - c
  - compiler
  - language
  - nexium
  - programming
ReleaseNotesUrl: https://github.com/{REPO}/releases/tag/{tag}
ManifestType: defaultLocale
ManifestVersion: 1.6.0
''')
