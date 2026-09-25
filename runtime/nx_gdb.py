# Nexium values in gdb, loaded by `nx debug` (gdb -x nx_gdb.py).
#
# The compiler writes C, so a Nexium value is a C struct in the debugger.
# These printers show the ones that would otherwise print as a pointer and
# a length: String and []u8 as text, List(T), []T and [N]T as their
# elements, ?T as its value or null, !T as its value or the error's name,
# Map(K, V) as its entries (debug builds name each map type `nx_dmap_K_V`
# and keep its key and value types in `nx_dkv_K_V`), and a struct's fields
# without the index the compiler appends in C (`x_0` shows as `x`). A
# function's locals keep that index (`total_3`): `info locals` lists them.

import re

import gdb

LIMIT = 200  # elements, entries and characters shown at most

_INDEXED = re.compile(r"^(.*)_(\d+)$")


def _read(ptr, n):
    if n <= 0 or int(ptr) == 0:
        return b""
    return bytes(gdb.selected_inferior().read_memory(int(ptr), n))


def _text(ptr, n):
    n = int(n)
    raw = _read(ptr, min(n, LIMIT * 4))
    s = raw.decode("utf-8", "replace")
    s = s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")
    return '"' + s + '"' + ("..." if n > LIMIT * 4 else "")


class TextPrinter:
    """String and []u8: the bytes as text."""

    def __init__(self, val):
        self.val = val

    def to_string(self):
        return _text(self.val["ptr"], self.val["len"])


class ElementsPrinter:
    """List(T) and []T: their elements."""

    def __init__(self, val, what):
        self.val = val
        self.what = what

    def to_string(self):
        return "%s of %d" % (self.what, int(self.val["len"]))

    def children(self):
        p = self.val["ptr"]
        for i in range(min(int(self.val["len"]), LIMIT)):
            yield "[%d]" % i, (p + i).dereference()

    def display_hint(self):
        return "array"


class ArrayPrinter:
    """[N]T: its elements, without the struct around them."""

    def __init__(self, val):
        self.val = val

    def to_string(self):
        return None

    def children(self):
        v = self.val["v"]
        lo, hi = v.type.range()
        for i in range(lo, min(hi + 1, LIMIT)):
            yield "[%d]" % i, v[i]

    def display_hint(self):
        return "array"


class OptionalPrinter:
    """?T: its value, or null."""

    def __init__(self, val):
        self.val = val

    def to_string(self):
        if not bool(self.val["has"]):
            return "null"
        try:
            return self.val["val"]
        except gdb.error:
            return "(some)"


class ErrorUnionPrinter:
    """!T: its value, or `error.Name`."""

    def __init__(self, val):
        self.val = val

    def to_string(self):
        err = int(self.val["err"])
        if err == 0:
            try:
                return self.val["val"]
            except gdb.error:
                return "(ok)"
        try:
            return "error." + gdb.parse_and_eval("nx_error_names")[err].string()
        except gdb.error:
            return "error #%d" % err


class MapPrinter:
    """Map(K, V): its entries, key => value."""

    def __init__(self, val, kv):
        self.val = val
        self.kv = kv

    def to_string(self):
        return "Map of %d" % int(self.val["len"])

    def children(self):
        if self.kv is None:
            return
        m = self.val
        # the entries, in the order their keys were put; a removed one is
        # not live until the map packs them
        used = int(m["used"])
        if used == 0:
            return
        # the descriptor holds pointers: a map's value may be the struct holding it
        fields = self.kv.strip_typedefs().fields()
        kt, vt = fields[0].type.target(), fields[1].type.target()
        ksize, vsize = int(m["ksize"]), int(m["vsize"])
        keys, vals = int(m["keys"]), int(m["vals"])
        live = _read(m["live"], used)
        shown = 0
        for i in range(used):
            if live[i] == 0:
                continue
            yield "k%d" % i, gdb.Value(keys + i * ksize).cast(kt.pointer()).dereference()
            yield "v%d" % i, gdb.Value(vals + i * vsize).cast(vt.pointer()).dereference()
            shown += 1
            if shown >= LIMIT:
                return

    def display_hint(self):
        return "map"


class TuplePrinter:
    """A tuple: its elements by position."""

    def __init__(self, val):
        self.val = val

    def to_string(self):
        return None

    def children(self):
        for i, f in enumerate(self.val.type.strip_typedefs().fields()):
            yield "[%d]" % i, self.val[f.name]


class StructPrinter:
    """A struct of the program: its fields by their Nexium names."""

    def __init__(self, val, fields):
        self.val = val
        self.fields = fields

    def to_string(self):
        return None

    def children(self):
        for f in self.fields:
            yield _INDEXED.match(f.name).group(1), self.val[f.name]


def _program_struct(t):
    """The fields of a struct the compiler made from the program's own, whose
    fields are all `name_<index>` in order; None for any other struct."""
    fields = t.fields()
    if not fields:
        return None
    for i, f in enumerate(fields):
        m = _INDEXED.match(f.name or "")
        if m is None or int(m.group(2)) != i:
            return None
    return fields


def _lookup(val):
    t = val.type
    if t.code == gdb.TYPE_CODE_TYPEDEF and (t.name or "").startswith("nx_dmap_"):
        try:
            kv = gdb.lookup_type("nx_dkv_" + t.name[len("nx_dmap_"):])
        except gdb.error:
            kv = None
        return MapPrinter(val, kv)
    s = t.strip_typedefs()
    if s.code != gdb.TYPE_CODE_STRUCT:
        return None
    tag = s.tag or s.name or ""
    if tag in ("nx_string", "nx_sl_u8"):
        return TextPrinter(val)
    if tag.startswith("nx_list_"):
        return ElementsPrinter(val, "List")
    if tag.startswith("nx_sl_"):
        return ElementsPrinter(val, "slice")
    if re.match(r"^nx_arr\d+_", tag):
        return ArrayPrinter(val)
    if tag.startswith("nx_opt_"):
        return OptionalPrinter(val)
    if tag.startswith("nx_eu_"):
        return ErrorUnionPrinter(val)
    if tag == "nx_map":
        return MapPrinter(val, None)
    if tag.startswith("nx_tup_"):
        return TuplePrinter(val)
    if tag.startswith("nx_"):
        fields = _program_struct(s)
        if fields is not None:
            return StructPrinter(val, fields)
    return None


gdb.pretty_printers.append(_lookup)
