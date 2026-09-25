# Nexium values in lldb, loaded by `nx debug`
# (lldb -o "command script import nx_lldb.py").
#
# The same as nx_gdb.py for lldb: String and []u8 as text, List(T) and []T
# as their elements, ?T as its value or null, !T as its value or the error's
# name, Map(K, V) as its entries (debug builds name each map type
# `nx_dmap_K_V` and keep its key and value types in `nx_dkv_K_V`). A
# function's locals keep the index the compiler appends in C (`total_3`):
# `frame variable` lists them.

import lldb

LIMIT = 200  # elements, entries and characters shown at most


def _member(valobj, name):
    return valobj.GetNonSyntheticValue().GetChildMemberWithName(name)


def _read(valobj, ptr, n):
    if n <= 0 or ptr == 0:
        return b""
    err = lldb.SBError()
    data = valobj.GetProcess().ReadMemory(ptr, n, err)
    return data if err.Success() else b""


def text_summary(valobj, internal_dict):
    """String and []u8: the bytes as text."""
    ptr = _member(valobj, "ptr").GetValueAsUnsigned(0)
    n = _member(valobj, "len").GetValueAsUnsigned(0)
    s = _read(valobj, ptr, min(n, LIMIT * 4)).decode("utf-8", "replace")
    s = s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")
    return '"' + s + '"' + ("..." if n > LIMIT * 4 else "")


class NoChildren:
    """String and []u8 show as text, not as a pointer and a length."""

    def __init__(self, valobj, internal_dict):
        pass

    def update(self):
        return False

    def num_children(self):
        return 0

    def get_child_index(self, name):
        return -1

    def get_child_at_index(self, index):
        return None


class ElementsProvider:
    """List(T) and []T: their elements."""

    def __init__(self, valobj, internal_dict):
        self.valobj = valobj
        self.update()

    def update(self):
        self.ptr = _member(self.valobj, "ptr")
        self.len = _member(self.valobj, "len").GetValueAsUnsigned(0)
        self.et = self.ptr.GetType().GetPointeeType()
        self.size = self.et.GetByteSize()
        return False

    def num_children(self):
        return min(self.len, LIMIT)

    def get_child_index(self, name):
        try:
            return int(name.lstrip("[").rstrip("]"))
        except ValueError:
            return -1

    def get_child_at_index(self, index):
        addr = self.ptr.GetValueAsUnsigned(0) + index * self.size
        return self.valobj.CreateValueFromAddress("[%d]" % index, addr, self.et)


def elements_summary(valobj, internal_dict):
    name = valobj.GetType().GetCanonicalType().GetName()
    what = "List" if "nx_list_" in name else "slice"
    return "%s of %d" % (what, _member(valobj, "len").GetValueAsUnsigned(0))


def optional_summary(valobj, internal_dict):
    """?T: its value, or null."""
    if not _member(valobj, "has").GetValueAsUnsigned(0):
        return "null"
    val = _member(valobj, "val")
    if not val.IsValid():
        return "(some)"
    return val.GetSummary() or val.GetValue() or "(some)"


def error_union_summary(valobj, internal_dict):
    """!T: its value, or `error.Name`."""
    err = _member(valobj, "err").GetValueAsUnsigned(0)
    if err == 0:
        val = _member(valobj, "val")
        if not val.IsValid():
            return "(ok)"
        return val.GetSummary() or val.GetValue() or "(ok)"
    names = valobj.GetTarget().FindFirstGlobalVariable("nx_error_names")
    name = names.GetChildAtIndex(err).GetSummary() if names.IsValid() else None
    return "error." + name.strip('"') if name else "error #%d" % err


class MapProvider:
    """Map(K, V): an entry per key, named by the key."""

    def __init__(self, valobj, internal_dict):
        self.valobj = valobj
        self.update()

    def update(self):
        self.slots = []
        self.kt = self.vt = None
        name = self.valobj.GetType().GetName()
        if name.startswith("nx_dmap_"):
            kv = self.valobj.GetTarget().FindFirstType("nx_dkv_" + name[len("nx_dmap_"):])
            if kv.IsValid():
                kv = kv.GetCanonicalType()
                # the descriptor holds pointers: a map's value may be the
                # struct holding it
                self.kt = kv.GetFieldAtIndex(0).GetType().GetPointeeType()
                self.vt = kv.GetFieldAtIndex(1).GetType().GetPointeeType()
        # the entries, in the order their keys were put; a removed one is
        # not live until the map packs them
        used = _member(self.valobj, "used").GetValueAsUnsigned(0)
        self.ksize = _member(self.valobj, "ksize").GetValueAsUnsigned(0)
        self.vsize = _member(self.valobj, "vsize").GetValueAsUnsigned(0)
        self.keys = _member(self.valobj, "keys").GetValueAsUnsigned(0)
        self.vals = _member(self.valobj, "vals").GetValueAsUnsigned(0)
        live = _read(self.valobj, _member(self.valobj, "live").GetValueAsUnsigned(0), used)
        self.slots = [i for i in range(len(live)) if live[i] != 0][:LIMIT]
        return False

    def num_children(self):
        return len(self.slots) if self.kt is not None else 0

    def get_child_index(self, name):
        return -1

    def get_child_at_index(self, index):
        slot = self.slots[index]
        key = self.valobj.CreateValueFromAddress("key", self.keys + slot * self.ksize, self.kt)
        shown = key.GetSummary() or key.GetValue() or "?"
        return self.valobj.CreateValueFromAddress("[%s]" % shown, self.vals + slot * self.vsize, self.vt)


def map_summary(valobj, internal_dict):
    return "Map of %d" % _member(valobj, "len").GetValueAsUnsigned(0)


def __lldb_init_module(debugger, internal_dict):
    run = debugger.HandleCommand
    run("type summary add -F nx_lldb.text_summary nx_string nx_sl_u8")
    run("type synthetic add -l nx_lldb.NoChildren nx_string nx_sl_u8")
    run('type synthetic add -l nx_lldb.ElementsProvider -x "^nx_(list|sl)_"')
    run('type summary add -e -F nx_lldb.elements_summary -x "^nx_(list|sl)_"')
    run('type summary add -F nx_lldb.optional_summary -x "^nx_opt_"')
    run('type summary add -F nx_lldb.error_union_summary -x "^nx_eu_"')
    run('type synthetic add -l nx_lldb.MapProvider -x "^nx_dmap_"')
    run('type summary add -e -F nx_lldb.map_summary -x "^nx_dmap_"')
