#!/usr/bin/env python3
"""Regenerate the Unicode tables at the end of std/text.nx.

    python scripts/unicode_tables.py [UNICORE_DIR]

The tables come from the Unicode Character Database as Perl compiles it
into its `unicore` directory, which every Perl carries (Git for Windows'
does, at Unicode 16.0.0): the grapheme cluster break map with
Extended_Pictographic and Indic_Conjunct_Break folded into it (To/GCB.pl),
East Asian width (To/Ea.pl), the general category (To/Gc.pl), and the case
mappings and case folding with their multi-character cases (To/Uc.pl,
To/Lc.pl, To/Cf.pl). Without an argument the directory is found through
`perl -MConfig`. The block between the two marker lines in std/text.nx is
replaced; nothing else in the file is touched.
"""
import os, re, subprocess, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TEXT = os.path.join(ROOT, "std", "text.nx")
BEGIN = "// ---------------------------------------------- generated: Unicode tables"
END = "// ---------------------------------------------- end of the generated tables"


def unicore_dir():
    if len(sys.argv) > 1:
        return sys.argv[1]
    out = subprocess.run(["perl", "-MConfig", "-e", "print $Config{privlib}"], capture_output=True, text=True, check=True).stdout.strip()
    # Git for Windows' Perl answers with its own POSIX path
    if os.name == "nt" and out.startswith("/"):
        git = subprocess.run(["git", "--exec-path"], capture_output=True, text=True, check=True).stdout.strip()
        base = git.split("/mingw64/")[0] if "/mingw64/" in git else os.path.dirname(os.path.dirname(git))
        out = base + "/usr" + out[len("/usr"):] if out.startswith("/usr") else base + out
    return os.path.join(out, "unicore")


def read_map(path):
    """(format, missing, [(lo, hi, value)], {cp: [cps]} specials) of a To/*.pl file."""
    text = open(path, encoding="utf-8").read()
    fmt = re.search(r"\{'format'\} = '([a-z]+)'", text).group(1)
    missing = re.search(r"\{'missing'\} = '([^']*)'", text).group(1)
    specials = {}
    for m in re.finditer(r"# U\+([0-9A-F]+) => ([0-9A-F ]+)", text):
        specials[int(m.group(1), 16)] = [int(x, 16) for x in m.group(2).split()]
    body = text.split("return <<'END';\n", 1)[1].split("\nEND\n", 1)[0]
    rows = []
    for line in body.splitlines():
        parts = line.split("\t")
        lo = int(parts[0], 16)
        hi = int(parts[1], 16) if parts[1] else lo
        rows.append((lo, hi, parts[2]))
    return fmt, missing, rows, specials


def version(dir):
    v = open(os.path.join(dir, "version")).read().strip()
    return v


# ------------------------------------------------------------------ grapheme breaks

GB_KINDS = {
    "Other": 0, "CR": 1, "LF": 2, "Control": 3, "Extend": 4, "ZWJ": 5,
    "Regional_Indicator": 6, "Prepend": 7, "SpacingMark": 8, "L": 9, "V": 10,
    "T": 11, "LV": 12, "LVT": 13, "ExtPict_XX": 14, "InCB_Consonant_XX": 15,
    "InCB_Extend_EX": 16, "InCB_Linker_EX": 17,
}


def merged(rows):
    """Adjacent ranges with the same value as one."""
    out = []
    for lo, hi, v in sorted(rows):
        if out and out[-1][2] == v and out[-1][1] + 1 == lo:
            out[-1] = (out[-1][0], hi, v)
        else:
            out.append((lo, hi, v))
    return out


def grapheme_rows(dir):
    _, missing, rows, _ = read_map(os.path.join(dir, "To", "GCB.pl"))
    assert missing == "Other", missing
    kept = []
    for lo, hi, v in rows:
        assert v in GB_KINDS, v
        # Hangul syllables are computed: LV every 28th from U+AC00, LVT between
        if v in ("LV", "LVT"):
            assert 0xAC00 <= lo and hi <= 0xD7A3, (hex(lo), v)
            continue
        # ASCII is decided in code
        if hi < 0x80:
            continue
        kept.append((max(lo, 0x80), hi, GB_KINDS[v]))
    return merged(kept)


# ------------------------------------------------------------------ widths

def width_rows(dir):
    _, _, gc, _ = read_map(os.path.join(dir, "To", "Gc.pl"))
    _, _, ea, _ = read_map(os.path.join(dir, "To", "Ea.pl"))
    _, _, gcb, _ = read_map(os.path.join(dir, "To", "GCB.pl"))
    zero = []
    for lo, hi, v in gc:
        if v in ("Mn", "Me", "Cf"):
            for a, b in split_out(lo, hi, 0xAD):
                zero.append((a, b, 0))
    # Hangul vowels and finals join the syllable before them
    for lo, hi, v in gcb:
        if v in ("V", "T"):
            zero.append((lo, hi, 0))
    wide = [(lo, hi, 0) for lo, hi, v in ea if v in ("W", "F")]
    zero = [(lo, hi) for lo, hi, _ in merged(zero)]
    wide = [(lo, hi) for lo, hi, _ in merged(wide)]
    # a combining mark stays zero width where the width table calls it wide
    return zero, wide


def split_out(lo, hi, cp):
    """The range without one code point in it."""
    if cp < lo or cp > hi:
        return [(lo, hi)]
    out = []
    if lo < cp:
        out.append((lo, cp - 1))
    if cp < hi:
        out.append((cp + 1, hi))
    return out


# ------------------------------------------------------------------ case

def simple_map(dir, name):
    """{cp: mapped} of an 'ax' map: each code point of a range maps to its
    value plus its distance from the range's start."""
    fmt, missing, rows, specials = read_map(os.path.join(dir, "To", name + ".pl"))
    assert fmt == "ax" and missing == "0", (name, fmt, missing)
    out = {}
    for lo, hi, v in rows:
        base = int(v, 16)
        for cp in range(lo, hi + 1):
            out[cp] = base + (cp - lo)
    return out, specials


def runs(m):
    """[(lo, hi, delta, step)]: code points `lo, lo + step, ..., hi` each map
    to themselves plus `delta`, step 1 or 2 (the alternating pairs of Latin
    Extended and Cyrillic)."""
    cps = sorted(m)
    out = []
    for cp in cps:
        d = m[cp] - cp
        if out:
            lo, hi, delta, step = out[-1]
            if delta == d and step == 1 and cp == hi + 1:
                out[-1] = (lo, cp, d, 1)
                continue
            if delta == d and cp == hi + 2 and (step == 2 or lo == hi):
                out[-1] = (lo, cp, d, 2)
                continue
        out.append((cp, cp, d, 1))
    # the ranges must not overlap, so a binary search finds one
    for a, b in zip(out, out[1:]):
        assert a[1] < b[0], (a, b)
    # nothing sits in a step-2 range's gaps
    keys = set(cps)
    for lo, hi, _, step in out:
        if step == 2:
            for cp in range(lo + 1, hi, 2):
                assert cp not in keys, hex(cp)
    return out


def specials_table(specials):
    cps = sorted(specials)
    at = [0]
    to = []
    for cp in cps:
        to.extend(specials[cp])
        at.append(len(to))
    return cps, at, to


# ------------------------------------------------------------------ writing

def nx_array(name, typ, values, per_line, fmt):
    lines = []
    for i in range(0, len(values), per_line):
        lines.append("    " + ", ".join(fmt(v) for v in values[i:i + per_line]) + ",")
    body = "\n".join(lines)
    return f"const {name}: [{len(values)}]{typ} = [\n{body}\n]\n"


def hx(v):
    return "0x%X" % v


def main():
    dir = unicore_dir()
    ver = version(dir)
    gb = grapheme_rows(dir)
    zero, wide = width_rows(dir)
    lower, lower_sp = simple_map(dir, "Lc")
    upper, upper_sp = simple_map(dir, "Uc")
    fold, fold_sp = simple_map(dir, "Cf")
    # folding differs from lower-casing for few code points: keep those
    fold_diff = {cp: to for cp, to in fold.items() if lower.get(cp, cp) != to}
    for cp, to in lower.items():
        if cp not in fold and to != cp:
            fold_diff[cp] = cp
    lo_runs = runs(lower)
    up_runs = runs(upper)

    parts = [BEGIN, f"// by scripts/unicode_tables.py from the Unicode Character Database {ver};", "// run it again rather than editing these.", ""]
    parts.append(f'pub const UNICODE_VERSION: []u8 = "{ver}"\n')
    parts.append("// grapheme cluster break classes of the code points from U+0080 on, the\n// Hangul syllables aside (GB_* above)")
    parts.append(nx_array("GB_LO", "u32", [r[0] for r in gb], 8, hx))
    parts.append(nx_array("GB_HI", "u32", [r[1] for r in gb], 8, hx))
    parts.append(nx_array("GB_KIND", "u8", [r[2] for r in gb], 24, str))
    parts.append("// no columns: nonspacing and enclosing marks, format characters but the\n// soft hyphen, Hangul vowels and finals")
    parts.append(nx_array("ZERO_LO", "u32", [r[0] for r in zero], 8, hx))
    parts.append(nx_array("ZERO_HI", "u32", [r[1] for r in zero], 8, hx))
    parts.append("// two columns: East Asian wide and fullwidth")
    parts.append(nx_array("WIDE_LO", "u32", [r[0] for r in wide], 8, hx))
    parts.append(nx_array("WIDE_HI", "u32", [r[1] for r in wide], 8, hx))
    for name, rs in (("LOWER", lo_runs), ("UPPER", up_runs)):
        parts.append(f"// the simple {name.lower()}case mapping: LO, LO + STEP, ... HI each add DELTA")
        parts.append(nx_array(f"{name}_LO", "u32", [r[0] for r in rs], 8, hx))
        parts.append(nx_array(f"{name}_HI", "u32", [r[1] for r in rs], 8, hx))
        parts.append(nx_array(f"{name}_DELTA", "i32", [r[2] for r in rs], 12, str))
        parts.append(nx_array(f"{name}_STEP", "u8", [r[3] for r in rs], 24, str))
    fd = sorted(fold_diff.items())
    parts.append("// where case folding differs from the simple lowercase mapping")
    parts.append(nx_array("FOLD_CP", "u32", [c for c, _ in fd], 8, hx))
    parts.append(nx_array("FOLD_TO", "u32", [t for _, t in fd], 8, hx))
    for name, sp in (("UPPER", upper_sp), ("LOWER", lower_sp), ("FOLD", fold_sp)):
        cps, at, to = specials_table(sp)
        parts.append(f"// {name.lower()} mappings to more than one code point: CP[i] becomes TO[AT[i]..AT[i + 1]]")
        parts.append(nx_array(f"{name}_SPECIAL_CP", "u32", cps, 8, hx))
        parts.append(nx_array(f"{name}_SPECIAL_AT", "u16", at, 16, str))
        parts.append(nx_array(f"{name}_SPECIAL_TO", "u32", to, 8, hx))
    parts.append(END)
    block = "\n".join(parts) + "\n"

    src = open(TEXT, encoding="utf-8").read()
    if BEGIN in src:
        head = src.split(BEGIN, 1)[0]
        tail = src.split(END + "\n", 1)[1]
        src = head + block + tail
    else:
        src = src.rstrip("\n") + "\n\n" + block
    with open(TEXT, "w", encoding="utf-8", newline="\n") as f:
        f.write(src)
    print(f"wrote {TEXT}: Unicode {ver}, {len(gb)} grapheme ranges, {len(zero)} zero-width and {len(wide)} wide ranges, "
          f"{len(lo_runs)} lower and {len(up_runs)} upper runs, {len(fd)} folding differences, "
          f"{len(upper_sp)}/{len(lower_sp)}/{len(fold_sp)} special upper/lower/fold mappings, {len(block)} bytes")


if __name__ == "__main__":
    main()
