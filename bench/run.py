#!/usr/bin/env python3
"""Build and time the benchmark programs in every language; the medians.

    python bench/run.py [--runs N] [--out results.json] [--page docs/numbers.md]
                        [--check baseline.json] [--nx PATH] [--only fib,nbody]

Every language must print the same answer for a program, or the run fails.
`--check` fails when Nexium's median on any program is more than a quarter
slower than the baseline's. `--page` renders the results as the site's
numbers page. The machine, the compiler versions and the date go in the
JSON, so a number is never without its context.
"""
import argparse, json, os, platform, shutil, statistics, subprocess, sys, time
from datetime import date

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BENCH = os.path.join(ROOT, "bench")
OUT = os.path.join(ROOT, "nx-out", "bench")
PROGRAMS = ["fib", "nbody", "sieve", "words"]
EXE = ".exe" if os.name == "nt" else ""

ap = argparse.ArgumentParser()
ap.add_argument("--runs", type=int, default=5)
ap.add_argument("--out")
ap.add_argument("--page")
ap.add_argument("--check")
ap.add_argument("--nx", default=None)
ap.add_argument("--only", default=None)
args = ap.parse_args()
os.makedirs(OUT, exist_ok=True)
programs = args.only.split(",") if args.only else PROGRAMS

def which(*names):
    for n in names:
        p = shutil.which(n)
        if p:
            return p
    return None

def run(cmd, cwd=None):
    r = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, encoding="utf-8", errors="replace")
    if r.returncode != 0:
        sys.exit(f"bench: {' '.join(cmd)} failed:\n{r.stdout}{r.stderr}")
    return r.stdout

def version(cmd):
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")
        return (r.stdout or r.stderr).strip().splitlines()[0]
    except Exception:
        return "?"

nx = os.path.abspath(args.nx) if args.nx else (which("nx") or os.path.join(ROOT, "nx-out", "bootstrap", "nx2" + EXE))
cc = which("gcc", "cc", "clang")
cc_cmd = [cc] if cc else ([which("zig"), "cc"] if which("zig") else None)
rustc = which("rustc")
go = which("go")
py = sys.executable

def agree(outputs):
    """The same answer from every language: token by token, exactly for
    integers and words, within one percent for floating-point numbers (the
    order of a few hundred thousand chaotic steps differs by the rounding of
    each compiler; a wrong program differs by far more)."""
    rows = [o.split() for o in outputs]
    if any(len(r) != len(rows[0]) for r in rows):
        return False
    for col in zip(*rows):
        if len(set(col)) == 1:
            continue
        try:
            vals = [float(t) for t in col]
        except ValueError:
            return False
        if not all("." in t or "e" in t.lower() for t in col):
            return False
        lo, hi = min(vals), max(vals)
        if hi - lo > 0.01 * max(abs(lo), abs(hi), 1e-9):
            return False
    return True

# --- build
tools = {}
def build(lang, prog):
    src = os.path.join(BENCH, f"{prog}.{ {'nexium': 'nx', 'c': 'c', 'rust': 'rs', 'go': 'go', 'python': 'py'}[lang] }")
    exe = os.path.join(OUT, f"{prog}-{lang}{EXE}")
    if lang == "nexium":
        run([nx, "build", src, "--mode", "fast", "-o", exe, "--out-dir", os.path.join(OUT, "nx-" + prog)])
    elif lang == "c":
        run(cc_cmd + ["-O2", "-o", exe, src, "-lm"])
    elif lang == "rust":
        run([rustc, "-O", "-o", exe, src])
    elif lang == "go":
        run([go, "build", "-o", exe, src], cwd=BENCH)
    elif lang == "python":
        return [py, src]
    return [exe]

languages = [("nexium", True), ("c", cc_cmd is not None), ("rust", rustc is not None), ("go", go is not None), ("python", True)]
tools["nexium"] = version([nx, "version"])
tools["c"] = version(cc_cmd + ["--version"]) if cc_cmd else "not found"
tools["rust"] = version([rustc, "--version"]) if rustc else "not found"
tools["go"] = version([go, "version"]) if go else "not found"
tools["python"] = version([py, "--version"])

results = {}
for prog in programs:
    answers = {}
    results[prog] = {}
    for lang, present in languages:
        if not present:
            continue
        cmd = build(lang, prog)
        times = []
        for _ in range(args.runs):
            t0 = time.perf_counter()
            r = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")
            times.append(time.perf_counter() - t0)
            if r.returncode != 0:
                sys.exit(f"bench: {prog} in {lang} failed:\n{r.stderr}")
            answers[lang] = r.stdout.strip()
        results[prog][lang] = round(statistics.median(times), 4)
        print(f"{prog:6} {lang:7} {results[prog][lang]:8.3f} s   {answers[lang]}")
    if not agree(list(answers.values())):
        sys.exit(f"bench: {prog}: the languages disagree: {answers}")

machine = {"os": platform.platform(), "cpu": platform.processor() or platform.machine(), "date": str(date.today()),
           "runner": os.environ.get("RUNNER_NAME") or os.environ.get("ImageOS") or platform.node()}
doc = {"machine": machine, "tools": tools, "runs": args.runs, "results": results}

if args.check and os.path.exists(args.check):
    base = json.load(open(args.check, encoding="utf-8"))
    slow = []
    for prog in programs:
        was = base.get("results", {}).get(prog, {}).get("nexium")
        now = results[prog].get("nexium")
        if was and now and now > was * 1.25:
            slow.append(f"{prog}: {now:.3f} s, was {was:.3f} s")
    if slow:
        sys.exit("bench: Nexium got slower than the baseline by more than a quarter:\n  " + "\n  ".join(slow))
    print(f"no regression against {args.check}")

if args.out:
    os.makedirs(os.path.dirname(os.path.abspath(args.out)), exist_ok=True)
    with open(args.out, "w", encoding="utf-8", newline="\n") as fh:
        json.dump(doc, fh, indent=2)
        fh.write("\n")
    print("wrote", args.out)

if args.page:
    langs = [l for l, _ in languages]
    lines = ["# The numbers", "",
             "Four small programs written the same way in five languages, timed on one",
             "machine on one day: the median wall time of several runs, in seconds,",
             "smaller is better. What each program measures, the rules, and how to run",
             "them yourself are in [`bench/`](https://github.com/Londopy/nexium/tree/main/bench);",
             "the Bench workflow regenerates this page on a GitHub runner weekly and at",
             "every tag, and fails when Nexium gets a quarter slower than the last run.", "",
             f"Measured {machine['date']} on {machine['runner']}, {machine['os']}, {machine['cpu']}; {args.runs} runs each.", "",
             "| program | " + " | ".join(langs) + " |", "| --- | " + " | ".join("---:" for _ in langs) + " |"]
    for prog in programs:
        row = [f"`{prog}`"]
        for l in langs:
            v = results[prog].get(l)
            row.append(f"{v:.3f}" if v is not None else "n/a")
        lines.append("| " + " | ".join(row) + " |")
    lines += ["", "Nexium is built in `fast` mode (no overflow or bounds checks, as `nx ship` builds a",
              "release), C with `-O2`, Rust with `-O`, Go and Python as they come.", "",
              "| tool | version |", "| --- | --- |"]
    for l in langs:
        lines.append(f"| {l} | {tools[l]} |")
    lines.append("")
    with open(args.page, "w", encoding="utf-8", newline="\n") as fh:
        fh.write("\n".join(lines))
    print("wrote", args.page)
