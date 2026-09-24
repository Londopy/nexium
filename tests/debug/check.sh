#!/bin/sh
# `nx debug` under gdb and lldb, for the CI job `debugger`: a breakpoint at
# the `println` of tests/debug/values.nx (a `.nx` line, through the `#line`
# directives), and each local shown by the formatters as a Nexium value
# (runtime/nx_gdb.py, runtime/nx_lldb.py).
#
#     sh tests/debug/check.sh path/to/nx
set -u
nx=$1
bad=0
mkdir -p nx-out

# expect <debugger> <output> <text>...: each text appears in the output
expect() {
    who=$1
    out=$2
    shift 2
    for want in "$@"; do
        if ! grep -qF -- "$want" "$out"; then
            echo "$who: missing: $want"
            bad=1
        fi
    done
}

if command -v gdb >/dev/null 2>&1; then
    NX_DEBUGGER="gdb -batch -x tests/debug/gdb.txt" "$nx" debug tests/debug/values.nx --out-dir nx-out/debug > nx-out/gdb.out 2>&1
    cat nx-out/gdb.out
    expect gdb nx-out/gdb.out 'values.nx:21' 'name_0 = "nexium"' 'xs_1 = List of 2 = {3, 4}' \
        'view_2 = slice of 2 = {3, 4}' 'm_3 = Map of 1 = {["one"] = 1}' 'maybe_4 = 7' 'none_5 = null' \
        'p_6 = {x = 1, y = 2}' 'arr_7 = {5, 6, 7}'
else
    echo "gdb: not installed"
    bad=1
fi

if command -v lldb >/dev/null 2>&1; then
    # Debian's lldb looks for lldb-server beside itself, not where it is
    srv=$(ls /usr/lib/llvm-*/bin/lldb-server 2>/dev/null | tail -1)
    if [ -n "$srv" ]; then export LLDB_DEBUGSERVER_PATH="$srv"; fi
    NX_DEBUGGER="lldb -b -s tests/debug/lldb.txt" "$nx" debug tests/debug/values.nx --out-dir nx-out/debug > nx-out/lldb.out 2>&1
    cat nx-out/lldb.out
    expect lldb nx-out/lldb.out 'values.nx:21' 'name_0 = "nexium"' 'xs_1 = List of 2' 'view_2 = slice of 2' \
        'm_3 = Map of 1' '["one"] = 1' 'maybe_4 = 7' 'none_5 = null'
else
    echo "lldb: not installed"
    bad=1
fi

exit $bad
