#!/bin/sh
# Nexium installer for macOS and Linux.
#
#   curl -fsSL https://raw.githubusercontent.com/Londopy/nexium/main/installers/install.sh | sh
#
# Installs nx into ~/.nexium/bin (or $NEXIUM_HOME/bin), verifies the download
# against the release's SHA256SUMS.txt, makes sure a C compiler is available
# (the system compiler on macOS; on Linux, a bundled Zig is downloaded when
# none is found), and adds the bin directory to your PATH in your shell's
# startup file. Nothing else on the system is touched.
#
# Environment:
#   NEXIUM_VERSION=v0.1.0      install a specific release (default: latest)
#   NEXIUM_HOME=/opt/nexium    install somewhere else (default: ~/.nexium)
#   NEXIUM_NO_MODIFY_PATH=1    do not edit shell startup files
#   NEXIUM_NO_ZIG=1            never download Zig
set -eu

REPO="Londopy/nexium"
VERSION="${NEXIUM_VERSION:-latest}"
HOME_DIR="${NEXIUM_HOME:-$HOME/.nexium}"
ZIG_VERSION="0.14.1"

say() { printf '%s\n' "$*"; }
die() { printf 'install.sh: %s\n' "$*" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

have curl || die "curl is required"
have tar || die "tar is required"

os="$(uname -s)"
arch="$(uname -m)"
case "$os" in
  Darwin) case "$arch" in arm64|aarch64) target="aarch64-apple-darwin" ;; *) die "macOS on $arch is not built yet; build from source with cargo install nexium" ;; esac ;;
  Linux)  case "$arch" in x86_64|amd64) target="x86_64-unknown-linux-gnu" ;; *) die "Linux on $arch is not built yet; build from source with cargo install nexium" ;; esac ;;
  *) die "unsupported OS: $os (use the Windows installer from the Releases page)" ;;
esac

# resolve the release tag
if [ "$VERSION" = "latest" ]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1)"
  [ -n "$VERSION" ] || die "could not find the latest release"
fi
base="https://github.com/$REPO/releases/download/$VERSION"
asset="nx-$VERSION-$target.tar.gz"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
say "downloading $asset"
curl -fsSL "$base/$asset" -o "$tmp/$asset"
curl -fsSL "$base/SHA256SUMS.txt" -o "$tmp/SHA256SUMS.txt" || die "SHA256SUMS.txt missing from release $VERSION"

# verify
expected="$(grep " $asset\$" "$tmp/SHA256SUMS.txt" | awk '{print $1}')"
[ -n "$expected" ] || die "no checksum for $asset in SHA256SUMS.txt"
if have sha256sum; then actual="$(sha256sum "$tmp/$asset" | awk '{print $1}')"
elif have shasum; then actual="$(shasum -a 256 "$tmp/$asset" | awk '{print $1}')"
else die "need sha256sum or shasum to verify the download"; fi
[ "$actual" = "$expected" ] || die "checksum mismatch for $asset (expected $expected, got $actual)"
say "checksum ok"

# install
mkdir -p "$HOME_DIR/bin" "$HOME_DIR/share"
tar -xzf "$tmp/$asset" -C "$tmp"
install -m 755 "$tmp/nx" "$HOME_DIR/bin/nx"
for f in README.md LICENSE CHANGELOG.md; do [ -f "$tmp/$f" ] && cp "$tmp/$f" "$HOME_DIR/share/"; done
for d in examples std docs; do [ -d "$tmp/$d" ] && { rm -rf "$HOME_DIR/share/$d"; cp -R "$tmp/$d" "$HOME_DIR/share/$d"; }; done
say "installed nx $VERSION to $HOME_DIR/bin/nx"

# a C compiler: nx looks for a bundled zig next to itself first, then the system
compiler=""
if [ -x "$HOME_DIR/zig/zig" ]; then compiler="bundled zig"
elif [ "$os" = "Darwin" ] && have cc; then compiler="system cc"
elif have zig; then compiler="zig on PATH"
elif have cc; then compiler="system cc"
fi
if [ -z "$compiler" ] && [ "${NEXIUM_NO_ZIG:-}" != "1" ]; then
  case "$target" in
    x86_64-unknown-linux-gnu) zig_name="zig-x86_64-linux-$ZIG_VERSION" ;;
    aarch64-apple-darwin) zig_name="zig-aarch64-macos-$ZIG_VERSION" ;;
  esac
  say "no C compiler found; downloading Zig $ZIG_VERSION into $HOME_DIR/zig"
  curl -fsSL "https://ziglang.org/download/$ZIG_VERSION/$zig_name.tar.xz" -o "$tmp/zig.tar.xz"
  if have python3; then
    zsum="$(curl -fsSL https://ziglang.org/download/index.json | python3 -c 'import json,sys; d=json.load(sys.stdin)["'"$ZIG_VERSION"'"]; k=[k for k in d if k.replace("-","_") in ("x86_64_linux","aarch64_macos") and "'"$target"'".startswith(k.split("-")[0])][0]; print(d[k]["shasum"])' 2>/dev/null || true)"
    if [ -n "$zsum" ]; then
      if have sha256sum; then zact="$(sha256sum "$tmp/zig.tar.xz" | awk '{print $1}')"; else zact="$(shasum -a 256 "$tmp/zig.tar.xz" | awk '{print $1}')"; fi
      [ "$zact" = "$zsum" ] || die "checksum mismatch for the Zig download"
      say "zig checksum ok"
    else
      say "warning: could not verify the Zig download (index.json unavailable)"
    fi
  fi
  rm -rf "$HOME_DIR/zig"
  mkdir -p "$HOME_DIR/zig"
  tar -xJf "$tmp/zig.tar.xz" -C "$HOME_DIR/zig" --strip-components=1
  compiler="bundled zig"
fi
[ -n "$compiler" ] || say "warning: no C compiler found; install Zig (https://ziglang.org/download) or a system compiler"

# PATH
if [ "${NEXIUM_NO_MODIFY_PATH:-}" != "1" ]; then
  line="export PATH=\"$HOME_DIR/bin:\$PATH\""
  added=""
  for rc in "$HOME/.profile" "$HOME/.bashrc" "$HOME/.zshrc"; do
    [ -f "$rc" ] || continue
    if ! grep -qs "$HOME_DIR/bin" "$rc"; then
      printf '\n# Nexium\n%s\n' "$line" >> "$rc"
      added="$added $rc"
    fi
  done
  [ -f "$HOME/.profile" ] || { printf '\n# Nexium\n%s\n' "$line" >> "$HOME/.profile"; added="$added $HOME/.profile"; }
  [ -z "$added" ] || say "added $HOME_DIR/bin to PATH in:$added"
fi

say ""
say "done. Open a new shell (or run: $line) and try:"
say "  nx doctor"
say "  nx run $HOME_DIR/share/examples/hello.nx"
"$HOME_DIR/bin/nx" doctor || true
