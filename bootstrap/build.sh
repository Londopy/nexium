#!/bin/sh
# Build `nx` from source with no Rust: a C compiler builds nx0 from the
# seed nx.c (the C the Nexium compiler emits for itself, as of the last
# release), nx0 builds self/nx.nx into nx1, and nx1 must rebuild itself to
# the same C. Run from the repository root:
#
#     sh bootstrap/build.sh            # -> nx-out/bootstrap/nx1 (nx1.exe on Windows)
#     CC="zig cc" sh bootstrap/build.sh
#
# CC defaults to `zig cc`, or `cc` on macOS (zig 0.14 cannot link against the
# current Xcode SDK). Set NX_CC or NX_ZIG for the compiler nx1 itself runs.
set -e
out=nx-out/bootstrap
mkdir -p "$out"
case "$(uname -s)" in
  Darwin) CC="${CC:-cc}"; libs="" ;;
  MINGW*|MSYS*|CYGWIN*|Windows_NT) CC="${CC:-zig cc}"; libs="-lws2_32"; exe=.exe ;;
  *) CC="${CC:-zig cc}"; libs="-lm -lc" ;;
esac
echo "stage 0: $CC builds nx0 from bootstrap/nx.c"
$CC -std=gnu11 -O2 -w -fno-strict-aliasing -o "$out/nx0$exe" bootstrap/nx.c $libs
echo "stage 1: nx0 builds self/nx.nx"
"$out/nx0$exe" build self/nx.nx --mode safe -o "$out/nx1$exe" --out-dir "$out"
echo "stage 2: nx1 emits itself, $CC builds nx2, nx2 emits itself"
"$out/nx1$exe" emit-c self/nx.nx --mode safe > "$out/nx1.c"
$CC -std=gnu11 -O2 -w -fno-strict-aliasing -o "$out/nx2$exe" "$out/nx1.c" $libs
"$out/nx2$exe" emit-c self/nx.nx --mode safe > "$out/nx2.c"
if cmp -s "$out/nx1.c" "$out/nx2.c"; then
  echo "fixed point: nx1 and nx2 emit the same C"
else
  echo "nx1 and nx2 emit different C" >&2; exit 1
fi
if ! cmp -s bootstrap/nx.c "$out/nx1.c"; then
  echo "note: bootstrap/nx.c is behind self/; at the release: cp $out/nx1.c bootstrap/nx.c"
fi
echo "built $out/nx1$exe"
