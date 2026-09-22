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
#   NEXIUM_FROM_SOURCE=1       build nx from bootstrap/nx.c with the C compiler on
#                              the machine instead of downloading a release
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
# the release built for this machine, or nothing: then the one C file below
case "$os" in
  Darwin) case "$arch" in arm64|aarch64) target="aarch64-apple-darwin" ;; *) target="" ;; esac ;;
  Linux)  case "$arch" in x86_64|amd64) target="x86_64-unknown-linux-gnu" ;; aarch64|arm64) target="aarch64-unknown-linux-gnu" ;; *) target="" ;; esac ;;
  MINGW*|MSYS*|CYGWIN*) die "on Windows, use the installer from the Releases page" ;;
  *) target="" ;;
esac
[ -z "${NEXIUM_FROM_SOURCE:-}" ] || target=""

# resolve the release tag
if [ "$VERSION" = "latest" ]; then
  VERSION="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1)"
  if [ -z "$VERSION" ]; then
    [ -z "$target" ] || die "could not find the latest release"
    VERSION=main
  fi
fi
base="https://github.com/$REPO/releases/download/$VERSION"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# The one C file: bootstrap/nx.c is the C the compiler emits for itself, as
# of the release, with the standard library inside. Any C compiler builds it,
# and the result is the whole nx, on any Unix the C compiler runs on. This is
# the road when no release is built for the machine, when the download
# fails, or when NEXIUM_FROM_SOURCE=1 asks for it.
build_from_seed() {
  cc_cmd=""
  if have cc; then cc_cmd="cc"; elif have gcc; then cc_cmd="gcc"; elif have clang; then cc_cmd="clang"; elif have zig; then cc_cmd="zig cc"; fi
  [ -n "$cc_cmd" ] || die "no release is built for $os/$arch and there is no C compiler to build one; install cc, gcc, clang or zig and run this again"
  say "building nx from the one C file, bootstrap/nx.c at $VERSION, with $cc_cmd"
  curl -fsSL "https://raw.githubusercontent.com/$REPO/$VERSION/bootstrap/nx.c" -o "$tmp/nx.c" || die "could not download bootstrap/nx.c at $VERSION"
  case "$cc_cmd" in
    *zig*) march="-mcpu=baseline" ;;
    *) case "$arch" in x86_64|amd64) march="-march=x86-64" ;; *) march="" ;; esac ;;
  esac
  case "$os" in Darwin) libs="" ;; *) libs="-lm -lc -lpthread" ;; esac
  $cc_cmd -std=gnu11 -O2 -w -fno-strict-aliasing $march -o "$tmp/nx" "$tmp/nx.c" $libs || die "$cc_cmd could not build bootstrap/nx.c"
  mkdir -p "$HOME_DIR/bin" "$HOME_DIR/share"
  install -m 755 "$tmp/nx" "$HOME_DIR/bin/nx"
  say "installed nx $VERSION (built from bootstrap/nx.c) to $HOME_DIR/bin/nx; the examples and docs are at https://github.com/$REPO"
}

asset="nx-$VERSION-$target.tar.gz"
if [ -z "$target" ]; then
  [ -n "${NEXIUM_FROM_SOURCE:-}" ] || say "no release is built for $os/$arch"
  build_from_seed
elif ! curl -fsSL "$base/$asset" -o "$tmp/$asset"; then
  say "$asset is not in release $VERSION"
  build_from_seed
else
  say "downloaded $asset"
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
  # the manual page and the completions: where man and the shells look in a home directory
  if [ -f "$tmp/nx.1" ]; then mkdir -p "$HOME_DIR/share/man/man1" && cp "$tmp/nx.1" "$HOME_DIR/share/man/man1/nx.1"; fi
  if [ -d "$tmp/completions" ]; then
    mkdir -p "$HOME_DIR/share/completions" && cp "$tmp/completions/"* "$HOME_DIR/share/completions/"
    [ -d "$HOME/.local/share/bash-completion/completions" ] && cp "$tmp/completions/nx.bash" "$HOME/.local/share/bash-completion/completions/nx" 2>/dev/null
    [ -d "$HOME/.config/fish/completions" ] && cp "$tmp/completions/nx.fish" "$HOME/.config/fish/completions/nx.fish" 2>/dev/null
  fi
  say "installed nx $VERSION to $HOME_DIR/bin/nx"
fi

# a C compiler: nx looks for a bundled zig next to itself first, then the
# system compiler on macOS, then zig on the PATH, then cc, gcc or clang
compiler=""
if [ -x "$HOME_DIR/zig/zig" ]; then compiler="bundled zig"
elif [ "$os" = "Darwin" ] && have cc; then compiler="system cc"
elif have zig; then compiler="zig on PATH"
elif have cc || have gcc || have clang; then compiler="the system compiler on the PATH"
fi
if [ -z "$compiler" ] && [ "${NEXIUM_NO_ZIG:-}" != "1" ]; then
  # The checksums of the 0.14.1 archives, from ziglang.org/download/index.json,
  # kept here so the download is checked against a number this script
  # carries rather than one fetched from the same host at the same moment.
  zsum=""
  case "$target" in
    x86_64-unknown-linux-gnu) zig_name="zig-x86_64-linux-$ZIG_VERSION"; zsum="24aeeec8af16c381934a6cd7d95c807a8cb2cf7df9fa40d359aa884195c4716c" ;;
    aarch64-unknown-linux-gnu) zig_name="zig-aarch64-linux-$ZIG_VERSION"; zsum="f7a654acc967864f7a050ddacfaa778c7504a0eca8d2b678839c21eea47c992b" ;;
    aarch64-apple-darwin) zig_name="zig-aarch64-macos-$ZIG_VERSION"; zsum="39f3dc5e79c22088ce878edc821dedb4ca5a1cd9f5ef915e9b3cc3053e8faefa" ;;
  esac
  [ "$ZIG_VERSION" = "0.14.1" ] || zsum=""
  say "no C compiler found; downloading Zig $ZIG_VERSION into $HOME_DIR/zig"
  curl -fsSL "https://ziglang.org/download/$ZIG_VERSION/$zig_name.tar.xz" -o "$tmp/zig.tar.xz"
  if [ -z "$zsum" ] && have python3; then
    # another version: the index is the only source of its checksum
    zsum="$(curl -fsSL https://ziglang.org/download/index.json | python3 -c 'import json,sys; d=json.load(sys.stdin)["'"$ZIG_VERSION"'"]; want={"x86_64-unknown-linux-gnu":"x86_64-linux","aarch64-unknown-linux-gnu":"aarch64-linux","aarch64-apple-darwin":"aarch64-macos"}["'"$target"'"]; print(d[want]["shasum"])' 2>/dev/null || true)"
  fi
  if [ -n "$zsum" ]; then
    if have sha256sum; then zact="$(sha256sum "$tmp/zig.tar.xz" | awk '{print $1}')"; else zact="$(shasum -a 256 "$tmp/zig.tar.xz" | awk '{print $1}')"; fi
    [ "$zact" = "$zsum" ] || die "checksum mismatch for the Zig download"
    say "zig checksum ok"
  else
    say "warning: could not verify the Zig download (no checksum for Zig $ZIG_VERSION on $target)"
  fi
  rm -rf "$HOME_DIR/zig"
  mkdir -p "$HOME_DIR/zig"
  tar -xJf "$tmp/zig.tar.xz" -C "$HOME_DIR/zig" --strip-components=1
  compiler="bundled zig"
fi
[ -n "$compiler" ] || say "warning: no C compiler found; install Zig (https://ziglang.org/download) or a system compiler"

# PATH
line="export PATH=\"$HOME_DIR/bin:\$PATH\""
if [ "${NEXIUM_NO_MODIFY_PATH:-}" != "1" ]; then
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
say "completions: nx completions bash|zsh|fish (copies are in $HOME_DIR/share/completions); the manual page: man $HOME_DIR/share/man/man1/nx.1"
"$HOME_DIR/bin/nx" doctor || true
