#!/bin/sh
# The playground's compiler: the C of the current sources (not the seed's),
# built for wasm32-wasi. The runtime compiles as NX_WASM there: no processes,
# sockets, terminal or setjmp. The stack is the linker's (the page tells the
# interpreter how deep it may go, NX_PLAY_DEPTH in site/play.js).
#
#   sh site/play_build.sh [nx] [out.wasm]
set -e
nx=${1:-nx-out/bootstrap/nx2}
out=${2:-site/out/play/nx.wasm}
mkdir -p nx-out "$(dirname "$out")"
"$nx" emit-c self/nx.nx --mode safe > nx-out/play.c
zig cc -target wasm32-wasi -std=gnu11 -Os -s -w -fno-strict-aliasing \
    -o "$out" nx-out/play.c -lm -Wl,-z,stack-size=67108864
ls -l "$out"
