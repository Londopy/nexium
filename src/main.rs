#![allow(dead_code)]
// dead_code: several fields and helpers are reserved for features listed in DECISIONS.md item 27.
mod ast;
mod cgen;
mod check;
mod cimport;
mod comptime;
mod diag;
mod doc;
mod effects;
mod fmt;
mod lexer;
mod lsp;
mod parser;
mod repl;
mod report;
mod ship;
mod size;
mod stdlib;
mod tir;
mod types;

use cgen::{BuildMode, Entry, Gen, GenOptions};
use check::{ArtifactValue, Program};
use diag::SourceMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn usage() -> ! {
    eprintln!(
        "nx {v} - the Nexium compiler

usage:
  nx build <file.nx> [options]     compile to an executable (or library when no main)
  nx run <file.nx> [-- args]       compile and run
  nx test <file.nx> [filter]       run `test \"...\"` blocks
  nx check <file.nx>               type-check and report effects violations only
  nx effects <file.nx>             report the inferred effects of every function
  nx audit <file.nx> [--globals]   list unsafe blocks and mutable globals
  nx refcounts <file.nx>           list every retain and release site
  nx size <file.nx>                attribute binary bytes to declarations
  nx fmt <file.nx> [--check]       canonical formatting in place (--check: report only)
  nx doc <file.nx> [-o dir]        static HTML documentation
  nx lsp                           language server over stdio (diagnostics, hover)
  nx leaks <file.nx> [-- args]     run in debug mode with allocation tracking, report leaks at exit
  nx ship <file.nx>                produce every declared artifact (cabi, python, cli)
  nx emit-c <file.nx>              print the generated C
  nx parse <file.nx>               dump the syntax tree
  nx tokens <file.nx>              dump the token stream (start end KIND payload)
  nx repl                          interactive session (also: `nx` with no arguments)
  nx doctor                        show which C compiler nx will use and whether it works
  nx version

options:
  --mode debug|safe|fast|small     build mode (default: safe for build/ship, debug for run/test)
  --target <triple>                cross-compile (any target zig cc supports, e.g. x86_64-linux-gnu)
  -o <path>                        output path
  --out-dir <dir>                  artifact directory (default: nx-out)
  --keep-c                         keep the generated C next to the output
  --cc <path>                      C compiler to use (default: `zig cc`; the system `cc` for native builds on macOS; NX_CC overrides)
  -I <dir>                         header search path for @cImport and vendored C
  --link <lib>, --link-path <dir>  link a C library (`artifact link {{ libs = [...] }}` does the same)
  --c-source <file.c>              compile a C source into the program (vendored C, spec 17.1)
",
        v = VERSION
    );
    exit(2)
}

struct Opts {
    file: PathBuf,
    mode: BuildMode,
    target: Option<String>,
    out: Option<PathBuf>,
    out_dir: PathBuf,
    keep_c: bool,
    cc: Option<String>,
    rest: Vec<String>,
    defines: Vec<String>,
    include_dirs: Vec<String>,
    link_libs: Vec<String>,
    link_paths: Vec<String>,
    c_sources: Vec<String>,
}

fn parse_opts(args: &[String]) -> Opts {
    let mut o = Opts {
        file: PathBuf::new(),
        mode: BuildMode::SafeRelease,
        target: None,
        out: None,
        out_dir: PathBuf::from("nx-out"),
        keep_c: false,
        cc: None,
        rest: Vec::new(),
        defines: Vec::new(),
        include_dirs: Vec::new(),
        link_libs: Vec::new(),
        link_paths: Vec::new(),
        c_sources: Vec::new(),
    };
    let mut i = 0;
    let mut file_set = false;
    let mut mode_set = false;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "--mode" => {
                i += 1;
                o.mode = match args.get(i).map(|s| s.as_str()) {
                    Some("debug") => BuildMode::Debug,
                    Some("safe") => BuildMode::SafeRelease,
                    Some("fast") => BuildMode::FastRelease,
                    Some("small") => BuildMode::SmallRelease,
                    _ => {
                        eprintln!("error: --mode needs debug|safe|fast|small");
                        exit(2)
                    }
                };
                mode_set = true;
            }
            "--target" => {
                i += 1;
                o.target = args.get(i).cloned();
            }
            "-o" => {
                i += 1;
                o.out = args.get(i).map(PathBuf::from);
            }
            "--out-dir" => {
                i += 1;
                o.out_dir = args.get(i).map(PathBuf::from).unwrap_or(o.out_dir);
            }
            "--keep-c" => o.keep_c = true,
            "-I" => {
                i += 1;
                if let Some(d) = args.get(i) {
                    o.include_dirs.push(d.clone());
                }
            }
            "--link" | "-l" => {
                i += 1;
                if let Some(l) = args.get(i) {
                    o.link_libs.push(l.clone());
                }
            }
            "--link-path" | "-L" => {
                i += 1;
                if let Some(l) = args.get(i) {
                    o.link_paths.push(l.clone());
                }
            }
            "--c-source" => {
                i += 1;
                if let Some(f) = args.get(i) {
                    o.c_sources.push(f.clone());
                }
            }
            "--cc" => {
                i += 1;
                o.cc = args.get(i).cloned();
            }
            "--" => {
                o.rest.extend(args[i + 1..].iter().cloned());
                break;
            }
            _ if !file_set => {
                o.file = PathBuf::from(a);
                file_set = true;
            }
            _ => o.rest.push(a.clone()),
        }
        i += 1;
    }
    if !file_set {
        eprintln!("error: a source file is required");
        usage();
    }
    if !mode_set {
        o.mode = BuildMode::Debug; // callers override for build/ship
    }
    o
}

struct Loaded {
    sm: SourceMap,
    modules: Vec<ast::Module>,
    names: Vec<String>,
    dirs: Vec<String>,
}

/// Load the root file and every module it imports (files next to it).
/// The directory a source file lives in, as a string; "." for a bare file name
/// (whose `parent()` is the empty path, not `None`).
fn dir_of(p: &Path) -> String {
    match p.parent() {
        Some(d) if !d.as_os_str().is_empty() => d.to_string_lossy().to_string(),
        _ => ".".into(),
    }
}

fn load(root: &Path) -> Result<Loaded, ()> {
    let mut sm = SourceMap::default();
    let mut modules = Vec::new();
    let mut names = Vec::new();
    let mut dirs = Vec::new();
    let root_dir = PathBuf::from(dir_of(root));
    let root_name = root.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "main".into());
    let mut queue: Vec<(String, PathBuf)> = vec![(root_name, root.to_path_buf())];
    let mut seen: HashMap<String, ()> = HashMap::new();
    let mut had_error = false;
    let mut std_pending: Vec<String> = Vec::new();
    while let Some((name, path)) = queue.pop() {
        if seen.contains_key(&name) {
            continue;
        }
        seen.insert(name.clone(), ());
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error: cannot read {}: {}", path.display(), e);
                return Err(());
            }
        };
        let display = path.to_string_lossy().to_string();
        let file = sm.add(display, text.clone());
        let (toks, ldiags) = lexer::Lexer::new(&text, file).lex();
        for d in &ldiags {
            eprint!("{}", sm.render(d));
        }
        let mut p = parser::Parser::new(toks, file);
        let m = p.parse_module();
        for d in &p.diags {
            eprint!("{}", sm.render(d));
        }
        if !ldiags.is_empty() || !p.diags.is_empty() {
            had_error = true;
        }
        for item in &m.items {
            if let ast::Item::Import(im) = item {
                if im.path[0] == "std" {
                    // std modules written in Nexium are embedded in the compiler; the
                    // rest of `std.*` are builtin namespaces. They load after the files
                    // so the root stays module 0.
                    if im.path.len() == 2 && stdlib::source(&im.path[1]).is_some() {
                        let mname = im.path.join(".");
                        if !std_pending.contains(&mname) {
                            std_pending.push(mname);
                        }
                    }
                    continue;
                }
                let rel: PathBuf = im.path.iter().collect();
                let candidate = root_dir.join(&rel).with_extension("nx");
                if candidate.exists() {
                    queue.push((im.path.join("."), candidate));
                }
            }
        }
        names.push(name);
        dirs.push(dir_of(&path));
        modules.push(m);
    }
    for mname in std_pending {
        let short = mname.trim_start_matches("std.").to_string();
        let src = stdlib::source(&short).unwrap();
        let file = sm.add(format!("<std>/{}.nx", short), src.to_string());
        let (toks, ldiags) = lexer::Lexer::new(src, file).lex();
        let mut p = parser::Parser::new(toks, file);
        let m = p.parse_module();
        for d in ldiags.iter().chain(p.diags.iter()) {
            eprint!("{}", sm.render(d));
            had_error = true;
        }
        names.push(mname);
        dirs.push("std".into());
        modules.push(m);
    }
    if had_error {
        return Err(());
    }
    Ok(Loaded { sm, modules, names, dirs })
}

/// `artifact link { c_sources = [...], libs = [...], include = [...] }` declared in the sources.
struct LinkInfo {
    c_sources: Vec<String>,
    libs: Vec<String>,
    include: Vec<String>,
    lib_paths: Vec<String>,
}

/// The OS of the build target: "windows", "linux", "macos", or "other".
fn target_os(target: &Option<String>) -> &'static str {
    match target {
        Some(t) => {
            if t.contains("windows") {
                "windows"
            } else if t.contains("linux") {
                "linux"
            } else if t.contains("macos") || t.contains("darwin") {
                "macos"
            } else {
                "other"
            }
        }
        None => {
            if cfg!(windows) {
                "windows"
            } else if cfg!(target_os = "macos") {
                "macos"
            } else if cfg!(target_os = "linux") {
                "linux"
            } else {
                "other"
            }
        }
    }
}

/// Collect `artifact link { ... }` from every module. Paths are relative to the
/// module that declares them; `libs_windows`, `libs_linux`, `libs_macos` apply
/// only when building for that OS.
fn link_info(loaded: &Loaded, opts: &Opts) -> LinkInfo {
    let mut li = LinkInfo { c_sources: vec![], libs: vec![], include: vec![], lib_paths: vec![] };
    let os = target_os(&opts.target);
    for (mi, m) in loaded.modules.iter().enumerate() {
        let root_dir = loaded.dirs.get(mi).cloned().unwrap_or_else(|| ".".into());
        for item in &m.items {
            if let ast::Item::Artifact(a) = item {
                if a.kind != "link" {
                    continue;
                }
                for (k, v, _) in &a.fields {
                    let vals: Vec<String> = match v {
                        ast::Expr::ArrayLit { elems, .. } => {
                            elems.iter().filter_map(|e| if let ast::Expr::Lit { value: ast::Lit::Str(s), .. } = e { Some(String::from_utf8_lossy(s).to_string()) } else { None }).collect()
                        }
                        ast::Expr::Lit { value: ast::Lit::Str(s), .. } => vec![String::from_utf8_lossy(s).to_string()],
                        _ => vec![],
                    };
                    let rel = |x: &String| Path::new(&root_dir).join(x).to_string_lossy().to_string();
                    match k.as_str() {
                        "c_sources" => li.c_sources.extend(vals.iter().map(rel)),
                        "libs" => li.libs.extend(vals),
                        "libs_windows" if os == "windows" => li.libs.extend(vals),
                        "libs_linux" if os == "linux" => li.libs.extend(vals),
                        "libs_macos" if os == "macos" => li.libs.extend(vals),
                        "include" => li.include.extend(vals.iter().map(rel)),
                        "lib_paths" => li.lib_paths.extend(vals.iter().map(rel)),
                        _ => {}
                    }
                }
            }
        }
    }
    li
}

fn check(loaded: &Loaded, opts: &Opts) -> Result<Program, ()> {
    let mut c = check::Checker::new(&loaded.sm);
    c.source_dirs = loaded.dirs.clone();
    let li = link_info(loaded, opts);
    let cc = cc_command(opts);
    let mut cc_vec = vec![cc.program.clone()];
    cc_vec.extend(cc.args.clone());
    let mut include_dirs: Vec<String> = loaded.dirs.clone();
    include_dirs.extend(opts.include_dirs.clone());
    include_dirs.extend(li.include.clone());
    c.cimport_opts = Some(cimport::ImportOptions { cc: cc_vec, target: opts.target.clone(), include_dirs, windows: is_windows_target(&opts.target) });
    let (res, diags) = c.check_program(&loaded.modules, &loaded.names);
    let mut errors = 0;
    for d in &diags {
        eprint!("{}", loaded.sm.render(d));
        if d.level == diag::Level::Error {
            errors += 1;
        }
    }
    if errors > 0 {
        eprintln!("{} error(s)", errors);
    }
    res
}

fn line_starts(sm: &SourceMap) -> Vec<Vec<u32>> {
    sm.files
        .iter()
        .map(|f| {
            let mut v = vec![0u32];
            for (i, b) in f.text.bytes().enumerate() {
                if b == b'\n' {
                    v.push(i as u32 + 1);
                }
            }
            v
        })
        .collect()
}

fn generate(prog: Program, loaded: &Loaded, mode: BuildMode, entry: Entry, lib_name: &str) -> Result<(String, Vec<cgen::exports::ExportInfo>), ()> {
    let opts = GenOptions { mode, entry, source_names: loaded.sm.files.iter().map(|f| f.name.clone()).collect() };
    let mut g = Gen::new(prog, opts);
    g.line_starts = line_starts(&loaded.sm);
    g.lib_name = lib_name.to_string();
    let exports: Vec<u32> = g.p.exports.clone();
    let mut infos = Vec::new();
    for e in exports {
        if let Some(i) = g.export_info(e) {
            infos.push(i);
        }
    }
    let helpers = if g.p.exports.is_empty() { String::new() } else { g.library_helpers() };
    match g.generate() {
        Ok(mut c) => {
            c.push_str(&helpers);
            Ok((c, infos))
        }
        Err(errs) => {
            for e in errs {
                eprintln!("error: {}", e);
            }
            Err(())
        }
    }
}

fn host_target() -> String {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    };
    let arch = if cfg!(target_arch = "aarch64") { "aarch64" } else { "x86_64" };
    format!("{}-{}", arch, os)
}

fn is_windows_target(t: &Option<String>) -> bool {
    match t {
        Some(t) => t.contains("windows"),
        None => cfg!(target_os = "windows"),
    }
}
fn is_macos_target(t: &Option<String>) -> bool {
    match t {
        Some(t) => t.contains("macos") || t.contains("darwin"),
        None => cfg!(target_os = "macos"),
    }
}

struct CcInvocation {
    program: String,
    args: Vec<String>,
}

/// A Zig installed next to `nx`: `<dir>/zig/zig` (the Windows installer) or
/// `<dir>/../zig/zig` (`~/.nexium/bin/nx` with `~/.nexium/zig`).
fn bundled_zig() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let name = if cfg!(windows) { "zig.exe" } else { "zig" };
    for cand in [dir.join("zig").join(name), dir.join("..").join("zig").join(name)] {
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

/// Which C compiler to run and why, in order: `--cc`, `NX_CC`, `NX_ZIG`, a Zig
/// bundled next to `nx`, the system compiler for a native build on macOS
/// (zig 0.14 cannot read the libSystem stubs of Xcode 16.3+ SDKs), then `zig`
/// on the PATH, which also cross-compiles.
fn cc_command_why(opts: &Opts) -> (CcInvocation, &'static str) {
    if let Some(cc) = &opts.cc {
        let mut parts = cc.split_whitespace();
        let program = parts.next().unwrap_or("cc").to_string();
        return (CcInvocation { program, args: parts.map(|s| s.to_string()).collect() }, "--cc");
    }
    if let Some(cc) = std::env::var("NX_CC").ok().filter(|s| !s.trim().is_empty()) {
        let mut parts = cc.split_whitespace();
        let program = parts.next().unwrap_or("cc").to_string();
        return (CcInvocation { program, args: parts.map(|s| s.to_string()).collect() }, "NX_CC");
    }
    if let Some(z) = std::env::var("NX_ZIG").ok().filter(|s| !s.trim().is_empty()) {
        return (CcInvocation { program: z, args: vec!["cc".into()] }, "NX_ZIG");
    }
    if let Some(z) = bundled_zig() {
        return (CcInvocation { program: z.to_string_lossy().to_string(), args: vec!["cc".into()] }, "bundled with nx");
    }
    if cfg!(target_os = "macos") && opts.target.is_none() {
        return (CcInvocation { program: "cc".into(), args: vec![] }, "system compiler (macOS)");
    }
    (CcInvocation { program: "zig".into(), args: vec!["cc".into()] }, "zig on PATH")
}

fn cc_command(opts: &Opts) -> CcInvocation {
    cc_command_why(opts).0
}

/// `nx doctor`: what this installation will use, and whether it works.
fn cmd_doctor() -> i32 {
    println!("nx {}", VERSION);
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("nx"));
    println!("executable:  {}", exe.display());
    let opts = Opts {
        file: PathBuf::new(),
        mode: BuildMode::Debug,
        target: None,
        out: None,
        out_dir: PathBuf::from("nx-out"),
        keep_c: false,
        cc: None,
        rest: vec![],
        defines: vec![],
        include_dirs: vec![],
        link_libs: vec![],
        link_paths: vec![],
        c_sources: vec![],
    };
    let (cc, why) = cc_command_why(&opts);
    let shown = if cc.args.is_empty() { cc.program.clone() } else { format!("{} {}", cc.program, cc.args.join(" ")) };
    println!("C compiler:  {}  ({})", shown, why);
    let probe = if cc.program.ends_with("zig") || cc.program.ends_with("zig.exe") { vec!["version"] } else { vec!["--version"] };
    let ok = match Command::new(&cc.program).args(&probe).output() {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            println!("             {}", text.lines().next().unwrap_or("").trim());
            true
        }
        Ok(o) => {
            println!("             does not run: {}", String::from_utf8_lossy(&o.stderr).lines().next().unwrap_or("").trim());
            false
        }
        Err(e) => {
            println!("             not found: {}", e);
            false
        }
    };
    println!("std modules: {}", stdlib::MODULES.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(", "));
    if let Some(dir) = exe.parent() {
        let on_path = std::env::var_os("PATH").map(|p| std::env::split_paths(&p).any(|d| d == dir)).unwrap_or(false);
        println!("on PATH:     {}", if on_path { "yes" } else { "no (add the executable's directory to PATH to run `nx` from anywhere)" });
    }
    if !ok {
        println!();
        println!("no working C compiler. Options:");
        println!("  - Windows: run the installer from https://github.com/Londopy/nexium/releases (it bundles Zig)");
        println!("  - macOS:   xcode-select --install");
        println!("  - Linux:   the install script (installers/install.sh) downloads Zig, or install zig / gcc");
        println!("  - any:     put zig on the PATH, or set NX_ZIG=/path/to/zig or NX_CC=gcc");
        return 1;
    }
    println!();
    println!("everything works. Try: nx run examples/hello.nx");
    0
}

fn compile_c(opts: &Opts, c_path: &Path, out: &Path, kind: &str) -> Result<(), ()> {
    let cc = cc_command(opts);
    let mut cmd = Command::new(&cc.program);
    cmd.args(&cc.args);
    cmd.arg("-std=gnu11");
    match opts.mode {
        BuildMode::Debug => {
            cmd.arg("-O0");
            cmd.arg("-g");
        }
        BuildMode::SafeRelease => {
            cmd.arg("-O2");
        }
        BuildMode::FastRelease => {
            cmd.arg("-O2");
        }
        BuildMode::SmallRelease => {
            cmd.arg("-Os");
        }
    }
    cmd.args(["-w", "-fno-strict-aliasing"]);
    for d in &opts.defines {
        cmd.arg(format!("-D{}", d));
    }
    let li = load(&opts.file).map(|l| link_info(&l, opts)).unwrap_or(LinkInfo { c_sources: vec![], libs: vec![], include: vec![], lib_paths: vec![] });
    for d in opts.include_dirs.iter().chain(li.include.iter()) {
        cmd.arg(format!("-I{}", d));
    }
    cmd.arg(format!("-I{}", dir_of(&opts.file)));
    for src in opts.c_sources.iter().chain(li.c_sources.iter()) {
        cmd.arg(src);
    }
    for d in opts.link_paths.iter().chain(li.lib_paths.iter()) {
        cmd.arg(format!("-L{}", d));
    }
    for l in opts.link_libs.iter().chain(li.libs.iter()) {
        cmd.arg(format!("-l{}", l));
    }
    if let Some(t) = &opts.target {
        cmd.arg("-target");
        cmd.arg(t);
    }
    if kind == "object" {
        // an object of the generated file alone
        let mut cmd2 = Command::new(&cc.program);
        cmd2.args(&cc.args);
        cmd2.args(["-std=gnu11", "-w", "-fno-strict-aliasing", "-c"]);
        match opts.mode {
            BuildMode::Debug => {
                cmd2.arg("-O0");
            }
            BuildMode::SmallRelease => {
                cmd2.arg("-Os");
            }
            _ => {
                cmd2.arg("-O2");
            }
        }
        if let Some(t) = &opts.target {
            cmd2.arg("-target");
            cmd2.arg(t);
        }
        for d in opts.include_dirs.iter().chain(li.include.iter()) {
            cmd2.arg(format!("-I{}", d));
        }
        cmd2.arg(format!("-I{}", dir_of(&opts.file)));
        cmd2.arg(c_path).arg("-o").arg(out);
        return match cmd2.status() {
            Ok(st) if st.success() => Ok(()),
            _ => {
                eprintln!("error: C compilation failed");
                Err(())
            }
        };
    }
    match kind {
        "shared" => {
            cmd.arg("-shared");
            cmd.arg("-DNX_BUILD_SHARED");
            if !is_windows_target(&opts.target) {
                cmd.arg("-fPIC");
            }
        }
        "object" => {
            cmd.arg("-c");
        }
        _ => {}
    }
    cmd.arg(c_path);
    cmd.arg("-o");
    cmd.arg(out);
    if !is_windows_target(&opts.target) {
        // libm is part of libSystem on macOS; a separate -lm there makes zig skip
        // its implicit libc link on machines without an SDK. Ask for libc explicitly.
        if !is_macos_target(&opts.target) {
            cmd.arg("-lm");
        }
        cmd.arg("-lc");
    }
    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot run the C compiler `{}`: {}\n  run `nx doctor` for options (the installers bundle Zig; or put zig on the PATH, or set NX_CC)", cc.program, e);
            return Err(());
        }
    };
    if !status.success() {
        eprintln!("error: C compilation failed (this is a compiler bug; run `nx emit-c` to inspect the output)");
        return Err(());
    }
    Ok(())
}

fn write_c(opts: &Opts, stem: &str, c: &str) -> Result<PathBuf, ()> {
    std::fs::create_dir_all(&opts.out_dir).map_err(|e| eprintln!("error: cannot create {}: {}", opts.out_dir.display(), e))?;
    let path = opts.out_dir.join(format!("{}.c", stem));
    std::fs::write(&path, c).map_err(|e| eprintln!("error: cannot write {}: {}", path.display(), e))?;
    let rt = opts.out_dir.join("nx_rt.h");
    let _ = std::fs::write(&rt, cgen::RUNTIME_H);
    Ok(path)
}

fn exe_name(stem: &str, target: &Option<String>) -> String {
    if is_windows_target(target) {
        format!("{}.exe", stem)
    } else {
        stem.to_string()
    }
}

fn stem_of(p: &Path) -> String {
    p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "out".into())
}

fn cmd_build(opts: &Opts, run: bool) -> Result<PathBuf, ()> {
    let loaded = load(&opts.file)?;
    let prog = check(&loaded, opts)?;
    let stem = stem_of(&opts.file);
    let has_main = prog.main.is_some();
    let entry = if has_main { Entry::Main } else { Entry::Library };
    if !has_main && !run {
        eprintln!("note: no `main` found; building a library object");
    }
    let (c, _infos) = generate(prog, &loaded, opts.mode, entry, &stem)?;
    let c_path = write_c(opts, &stem, &c)?;
    let out = match &opts.out {
        Some(o) => o.clone(),
        None => opts.out_dir.join(if has_main { exe_name(&stem, &opts.target) } else { format!("{}.o", stem) }),
    };
    compile_c(opts, &c_path, &out, if has_main { "exe" } else { "object" })?;
    if !opts.keep_c {
        let _ = std::fs::remove_file(&c_path);
    }
    Ok(out)
}

fn cmd_run(opts: &Opts) -> i32 {
    let exe = match cmd_build(opts, true) {
        Ok(e) => e,
        Err(()) => return 1,
    };
    let exe = if exe.is_relative() { std::env::current_dir().unwrap().join(exe) } else { exe };
    match Command::new(&exe).args(&opts.rest).status() {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("error: cannot run {}: {}", exe.display(), e);
            1
        }
    }
}

fn cmd_test(opts: &Opts) -> i32 {
    let loaded = match load(&opts.file) {
        Ok(l) => l,
        Err(()) => return 1,
    };
    let prog = match check(&loaded, opts) {
        Ok(p) => p,
        Err(()) => return 1,
    };
    if prog.tests.is_empty() {
        println!("no tests found");
        return 0;
    }
    // compile-time tests run in the interpreter; runtime tests are compiled
    let stem = stem_of(&opts.file);
    let (c, _) = match generate(prog, &loaded, opts.mode, Entry::Tests, &stem) {
        Ok(x) => x,
        Err(()) => return 1,
    };
    let c_path = match write_c(opts, &format!("{}_test", stem), &c) {
        Ok(p) => p,
        Err(()) => return 1,
    };
    let out = opts.out_dir.join(exe_name(&format!("{}_test", stem), &opts.target));
    if compile_c(opts, &c_path, &out, "exe").is_err() {
        return 1;
    }
    if !opts.keep_c {
        let _ = std::fs::remove_file(&c_path);
    }
    let exe = if out.is_relative() { std::env::current_dir().unwrap().join(out) } else { out };
    match Command::new(&exe).args(&opts.rest).status() {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("error: cannot run tests: {}", e);
            1
        }
    }
}

fn cmd_effects(opts: &Opts) -> i32 {
    let loaded = match load(&opts.file) {
        Ok(l) => l,
        Err(()) => return 1,
    };
    let prog = match check(&loaded, opts) {
        Ok(p) => p,
        Err(()) => return 1,
    };
    let mut tys = prog.tys;
    let mut rows: Vec<(String, String)> = Vec::new();
    let sm = &loaded.sm;
    let names = |t: &mut types::TyTable, ty: types::TyId| -> String { type_display(t, ty, &prog.structs, &prog.enums, &prog.aliases) };
    for f in &prog.funcs {
        if f.body.is_none() || f.is_closure || f.is_test {
            continue;
        }
        let params: Vec<String> = f
            .params
            .iter()
            .map(|&p| {
                let l = &f.locals[p as usize];
                format!("{}: {}", l.name, names(&mut tys, l.ty))
            })
            .collect();
        let ret = names(&mut tys, f.ret);
        let eff = f.effects;
        let sig = format!("{}fn {}({}) -> {}", if f.is_pub { "pub " } else { "" }, f.name, params.join(", "), ret);
        let effs = if eff.is_empty() { "(pure)".to_string() } else { eff.render() };
        let loc = sm.location(f.span);
        rows.push((format!("{}  {}", sig, effs), loc));
    }
    println!("inferred effects ({} functions)\n", rows.len());
    for (r, loc) in rows {
        println!("{}\n    at {}", r, loc);
    }
    0
}

fn type_display(tys: &mut types::TyTable, t: types::TyId, structs: &[check::StructDef], enums: &[check::EnumDef], aliases: &[check::AliasDef]) -> String {
    use types::*;
    let t = tys.resolve(t, true);
    match tys.kind(t).clone() {
        TyKind::Int(i) => i.name().into(),
        TyKind::Float(FloatTy::F32) => "f32".into(),
        TyKind::Float(FloatTy::F64) => "f64".into(),
        TyKind::Bool => "bool".into(),
        TyKind::Char => "char".into(),
        TyKind::Void => "void".into(),
        TyKind::Never => "never".into(),
        TyKind::Struct(d, args) => {
            let n = structs[d as usize].name.clone();
            if args.is_empty() {
                n
            } else {
                format!("{}({})", n, args.iter().map(|&a| type_display(tys, a, structs, enums, aliases)).collect::<Vec<_>>().join(", "))
            }
        }
        TyKind::Enum(d, args) => {
            let n = enums[d as usize].name.clone();
            if args.is_empty() {
                n
            } else {
                format!("{}({})", n, args.iter().map(|&a| type_display(tys, a, structs, enums, aliases)).collect::<Vec<_>>().join(", "))
            }
        }
        TyKind::Array(n, e) => format!("[{}]{}", n, type_display(tys, e, structs, enums, aliases)),
        TyKind::Slice(m, e) => format!("[]{}{}", if m { "mut " } else { "" }, type_display(tys, e, structs, enums, aliases)),
        TyKind::Ptr(m, e) => format!("*{}{}", if m { "mut " } else { "" }, type_display(tys, e, structs, enums, aliases)),
        TyKind::Opt(e) => format!("?{}", type_display(tys, e, structs, enums, aliases)),
        TyKind::ErrUnion(_, e) => format!("!{}", type_display(tys, e, structs, enums, aliases)),
        TyKind::Fn(ps, r, _) => {
            format!("fn({}) -> {}", ps.iter().map(|&a| type_display(tys, a, structs, enums, aliases)).collect::<Vec<_>>().join(", "), type_display(tys, r, structs, enums, aliases))
        }
        TyKind::Tuple(ts) => format!("({})", ts.iter().map(|&a| type_display(tys, a, structs, enums, aliases)).collect::<Vec<_>>().join(", ")),
        TyKind::Distinct(d) => aliases[d as usize].name.clone(),
        TyKind::Weak(e) => format!("weak {}", type_display(tys, e, structs, enums, aliases)),
        TyKind::List(e) => format!("List({})", type_display(tys, e, structs, enums, aliases)),
        TyKind::Str => "String".into(),
        TyKind::Map(k, v) => format!("Map({}, {})", type_display(tys, k, structs, enums, aliases), type_display(tys, v, structs, enums, aliases)),
        TyKind::ErrorSet(_) => "error".into(),
        other => format!("{:?}", other),
    }
}

fn cmd_refcounts(opts: &Opts) -> i32 {
    let loaded = match load(&opts.file) {
        Ok(l) => l,
        Err(()) => return 1,
    };
    let mut prog = match check(&loaded, opts) {
        Ok(p) => p,
        Err(()) => return 1,
    };
    let sites = report::refcount_sites(&mut prog);
    println!("reference count traffic: {} site(s)\n", sites.len());
    for s in &sites {
        println!("{}  {}\n    in {}", loaded.sm.location(s.span), s.what, s.func);
    }
    if sites.is_empty() {
        println!("this program performs no reference counting: every function qualifies for `!refcounts` (spec 5.3)");
    }
    0
}

/// Check a document from memory (the editor's buffer) and return diagnostics as (line, col) pairs.
fn analyze_text(path: &str, text: &str) -> (Vec<lsp::Diagnostic>, Option<(Loaded, Program)>) {
    let opts = Opts {
        file: PathBuf::from(path),
        mode: BuildMode::Debug,
        target: None,
        out: None,
        out_dir: PathBuf::from("nx-out"),
        keep_c: false,
        cc: None,
        rest: vec![],
        defines: vec![],
        include_dirs: vec![],
        link_libs: vec![],
        link_paths: vec![],
        c_sources: vec![],
    };
    let mut sm = SourceMap::default();
    let file = sm.add(path.to_string(), text.to_string());
    let (toks, ldiags) = lexer::Lexer::new(text, file).lex();
    let mut p = parser::Parser::new(toks, file);
    let m = p.parse_module();
    let mut all: Vec<diag::Diag> = ldiags;
    all.extend(p.diags.clone());
    let mut result = None;
    if all.is_empty() {
        let dir = dir_of(Path::new(path));
        // imported modules are loaded from disk
        let mut loaded = Loaded { sm, modules: vec![m], names: vec![Path::new(path).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "main".into())], dirs: vec![dir.clone()] };
        for item in loaded.modules[0].items.clone() {
            if let ast::Item::Import(im) = item {
                if im.path[0] == "std" {
                    if im.path.len() == 2 {
                        if let Some(src) = stdlib::source(&im.path[1]) {
                            let f = loaded.sm.add(format!("<std>/{}.nx", im.path[1]), src.to_string());
                            let (tk, _) = lexer::Lexer::new(src, f).lex();
                            let mut pp = parser::Parser::new(tk, f);
                            loaded.modules.push(pp.parse_module());
                            loaded.names.push(im.path.join("."));
                            loaded.dirs.push("std".into());
                        }
                    }
                    continue;
                }
                let rel: PathBuf = im.path.iter().collect();
                let candidate = Path::new(&dir).join(&rel).with_extension("nx");
                if let Ok(t) = std::fs::read_to_string(&candidate) {
                    let f = loaded.sm.add(candidate.to_string_lossy().to_string(), t.clone());
                    let (tk, _) = lexer::Lexer::new(&t, f).lex();
                    let mut pp = parser::Parser::new(tk, f);
                    let mm = pp.parse_module();
                    loaded.modules.push(mm);
                    loaded.names.push(im.path.join("."));
                    loaded.dirs.push(dir_of(&candidate));
                }
            }
        }
        let mut c = check::Checker::new(&loaded.sm);
        c.source_dirs = loaded.dirs.clone();
        let cc = cc_command(&opts);
        let mut cc_vec = vec![cc.program.clone()];
        cc_vec.extend(cc.args.clone());
        c.cimport_opts = Some(cimport::ImportOptions { cc: cc_vec, target: None, include_dirs: loaded.dirs.clone(), windows: cfg!(windows) });
        let (res, diags) = c.check_program(&loaded.modules, &loaded.names);
        all.extend(diags);
        if let Ok(prog) = res {
            result = Some((loaded, prog));
        }
        let sm_ref = match &result {
            Some((l, _)) => &l.sm,
            None => {
                // rebuild a map for rendering when checking failed
                return (to_lsp_diags(&all, &SourceMap { files: vec![] }, path, text), None);
            }
        };
        return (to_lsp_diags(&all, sm_ref, path, text), result);
    }
    (to_lsp_diags(&all, &sm, path, text), None)
}

/// Hover for a file that does not check: the signature as written, without effects.
fn hover_from_syntax(text: &str, off: usize) -> Option<lsp::HoverInfo> {
    let (toks, _) = lexer::Lexer::new(text, 0).lex();
    let mut p = parser::Parser::new(toks, 0);
    let m = p.parse_module();
    let mut fns: Vec<&ast::FnDecl> = vec![];
    for item in &m.items {
        match item {
            ast::Item::Fn(f) => fns.push(f),
            ast::Item::Impl(i) => fns.extend(i.methods.iter()),
            _ => {}
        }
    }
    let mut best: Option<&ast::FnDecl> = None;
    for f in fns {
        if (f.span.start as usize) <= off && off <= (f.span.end as usize) && best.map(|b| f.span.end - f.span.start < b.span.end - b.span.start).unwrap_or(true) {
            best = Some(f);
        }
    }
    let f = best?;
    let end = f.body.as_ref().map(|b| b.span.start as usize).unwrap_or(f.span.end as usize);
    let sig = text.get(f.span.start as usize..end)?.trim().trim_end_matches('{').trim();
    Some(lsp::HoverInfo { text: format!("```nexium\n{}\n```\nfile has errors; effects not inferred", sig) })
}

/// One line per token for `nx tokens`: `start end KIND [payload]`. This is the
/// oracle the self-hosted lexer is checked against, so the format is fixed.
fn token_line(t: &lexer::Token, text: &str) -> String {
    use lexer::Tok;
    let dbg = format!("{:?}", t.tok);
    let kind = dbg.split('(').next().unwrap_or(&dbg).to_string();
    let payload = match &t.tok {
        Tok::Ident(s) => s.clone(),
        Tok::Int(v) => v.to_string(),
        Tok::Float(_) => text[t.span.start as usize..t.span.end as usize].to_string(),
        Tok::Str(b) | Tok::Bytes(b) => b.iter().map(|x| format!("{:02x}", x)).collect::<String>(),
        Tok::Char(c) => c.to_string(),
        Tok::Doc(s) | Tok::Comment(s) => s.clone(),
        _ => String::new(),
    };
    if payload.is_empty() {
        format!("{} {} {}", t.span.start, t.span.end, kind)
    } else {
        format!("{} {} {} {}", t.span.start, t.span.end, kind, payload)
    }
}

/// One line of the C compiler in use, for banners.
pub fn compiler_summary() -> String {
    let opts = Opts {
        file: PathBuf::new(),
        mode: BuildMode::Debug,
        target: None,
        out: None,
        out_dir: PathBuf::from("nx-out"),
        keep_c: false,
        cc: None,
        rest: vec![],
        defines: vec![],
        include_dirs: vec![],
        link_libs: vec![],
        link_paths: vec![],
        c_sources: vec![],
    };
    let (cc, why) = cc_command_why(&opts);
    let base = Path::new(&cc.program).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or(cc.program.clone());
    if why.starts_with(&base) {
        why.to_string()
    } else {
        format!("{} via {}", base, why)
    }
}

/// Check a synthetic REPL program and run its new statements. Errors come back
/// rendered, ready to print.
pub fn repl_check(text: &str, start: usize, seed: Vec<(String, tir::Value)>) -> Result<tir::ReplOutcome, String> {
    let mut sm = SourceMap::default();
    let file = sm.add("<repl>".to_string(), text.to_string());
    let (toks, ldiags) = lexer::Lexer::new(text, file).lex();
    let mut p = parser::Parser::new(toks, file);
    let m = p.parse_module();
    let mut all: Vec<diag::Diag> = ldiags;
    all.extend(p.diags.clone());
    if !all.is_empty() {
        return Err(all.iter().map(|d| sm.render(d)).collect::<String>());
    }
    let cwd = std::env::current_dir().map(|d| d.to_string_lossy().to_string()).unwrap_or_else(|_| ".".into());
    let mut modules = vec![m];
    let mut names = vec!["main".to_string()];
    let mut dirs = vec![cwd.clone()];
    let mut std_pending: Vec<String> = Vec::new();
    for item in modules[0].items.clone() {
        if let ast::Item::Import(im) = item {
            if im.path[0] == "std" {
                if im.path.len() == 2 && stdlib::source(&im.path[1]).is_some() && !std_pending.contains(&im.path.join(".")) {
                    std_pending.push(im.path.join("."));
                }
                continue;
            }
            let rel: PathBuf = im.path.iter().collect();
            let candidate = Path::new(&cwd).join(&rel).with_extension("nx");
            if let Ok(t) = std::fs::read_to_string(&candidate) {
                let f = sm.add(candidate.to_string_lossy().to_string(), t.clone());
                let (tk, _) = lexer::Lexer::new(&t, f).lex();
                let mut pp = parser::Parser::new(tk, f);
                modules.push(pp.parse_module());
                names.push(im.path.join("."));
                dirs.push(dir_of(&candidate));
            }
        }
    }
    for mname in std_pending {
        let short = mname.trim_start_matches("std.").to_string();
        let src = stdlib::source(&short).unwrap();
        let f = sm.add(format!("<std>/{}.nx", short), src.to_string());
        let (tk, _) = lexer::Lexer::new(src, f).lex();
        let mut pp = parser::Parser::new(tk, f);
        modules.push(pp.parse_module());
        names.push(mname);
        dirs.push("std".into());
    }
    let mut c = check::Checker::new(&sm);
    c.source_dirs = dirs;
    c.repl_mode = true;
    c.repl_request = Some((start, seed));
    let (res, diags) = c.check_program(&modules, &names);
    match res {
        Ok(prog) => Ok(prog.repl.unwrap_or_default()),
        Err(()) => Err(diags.iter().filter(|d| d.level == diag::Level::Error).map(|d| sm.render(d)).collect::<String>()),
    }
}

fn offset_to_pos(text: &str, off: usize) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;
    for (i, ch) in text.char_indices() {
        if i >= off {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += ch.len_utf16();
        }
    }
    (line, col)
}

fn to_lsp_diags(diags: &[diag::Diag], sm: &SourceMap, path: &str, text: &str) -> Vec<lsp::Diagnostic> {
    let mut out = Vec::new();
    for d in diags {
        // only diagnostics in the open file (file 0 or a file with this path)
        let in_file = d.span.file == 0 || sm.file(d.span.file).map(|f| f.name == path).unwrap_or(false);
        if !in_file {
            continue;
        }
        let (line, col) = offset_to_pos(text, d.span.start as usize);
        let (end_line, end_col) = offset_to_pos(text, d.span.end.max(d.span.start + 1) as usize);
        let mut message = d.msg.clone();
        for (_, n) in &d.notes {
            message.push_str("\n");
            message.push_str(n);
        }
        out.push(lsp::Diagnostic {
            line,
            col,
            end_line,
            end_col,
            severity: match d.level {
                diag::Level::Error => 1,
                diag::Level::Warning => 2,
                diag::Level::Note => 3,
            },
            message,
        });
    }
    out
}

fn cmd_lsp() -> i32 {
    let backend = lsp::Backend {
        diagnostics: Box::new(|path, text| analyze_text(path, text).0),
        hover: Box::new(|path, text, line, col| {
            let (_, result) = analyze_text(path, text);
            // byte offset of the position
            let mut off = 0;
            for (i, l) in text.split('\n').enumerate() {
                if i == line {
                    off += col.min(l.len());
                    break;
                }
                off += l.len() + 1;
            }
            let (loaded, prog) = match result {
                Some(r) => r,
                None => return hover_from_syntax(text, off),
            };
            // the function whose span contains the offset (in the root file)
            let mut tys = prog.tys;
            let mut best: Option<&tir::TFunc> = None;
            for f in &prog.funcs {
                if f.span.file == 0 && (f.span.start as usize) <= off && off <= (f.span.end as usize) && f.body.is_some() && !f.is_closure {
                    if best.map(|b| f.span.end - f.span.start < b.span.end - b.span.start).unwrap_or(true) {
                        best = Some(f);
                    }
                }
            }
            let f = best?;
            let params: Vec<String> = f
                .params
                .iter()
                .map(|&p| {
                    let l = &f.locals[p as usize];
                    format!("{}: {}", l.name, type_display(&mut tys, l.ty, &prog.structs, &prog.enums, &prog.aliases))
                })
                .collect();
            let ret = type_display(&mut tys, f.ret, &prog.structs, &prog.enums, &prog.aliases);
            let eff = if f.effects.is_empty() { "pure (no effects)".to_string() } else { format!("effects: {}", f.effects.render()) };
            let _ = loaded;
            Some(lsp::HoverInfo { text: format!("```nexium\nfn {}({}) -> {}\n```\n{}", f.name, params.join(", "), ret, eff) })
        }),
    };
    lsp::run(backend);
    0
}

fn cmd_fmt(opts: &Opts) -> i32 {
    let check_only = opts.rest.iter().any(|a| a == "--check");
    let mut files = vec![opts.file.clone()];
    files.extend(opts.rest.iter().filter(|a| !a.starts_with("--")).map(PathBuf::from));
    let mut changed = 0;
    for f in files {
        let text = match std::fs::read_to_string(&f) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error: cannot read {}: {}", f.display(), e);
                return 1;
            }
        };
        let formatted = match fmt::format_source(&text) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {}: {}", f.display(), e);
                return 1;
            }
        };
        if formatted != text {
            changed += 1;
            if check_only {
                println!("would reformat {}", f.display());
            } else if let Err(e) = std::fs::write(&f, &formatted) {
                eprintln!("error: cannot write {}: {}", f.display(), e);
                return 1;
            } else {
                println!("formatted {}", f.display());
            }
        }
    }
    if check_only && changed > 0 {
        return 1;
    }
    0
}

fn cmd_doc(opts: &Opts) -> i32 {
    let loaded = match load(&opts.file) {
        Ok(l) => l,
        Err(()) => return 1,
    };
    let prog = match check(&loaded, opts) {
        Ok(p) => p,
        Err(()) => return 1,
    };
    let stem = stem_of(&opts.file);
    let html = doc::generate(&stem, &loaded.modules, &loaded.names, &prog);
    let dir = opts.out.clone().unwrap_or_else(|| opts.out_dir.join("doc"));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("error: cannot create {}: {}", dir.display(), e);
        return 1;
    }
    let path = dir.join(format!("{}.html", stem));
    if let Err(e) = std::fs::write(&path, html) {
        eprintln!("error: cannot write {}: {}", path.display(), e);
        return 1;
    }
    println!("wrote {}", path.display());
    0
}

fn cmd_size(opts: &Opts) -> i32 {
    let loaded = match load(&opts.file) {
        Ok(l) => l,
        Err(()) => return 1,
    };
    let prog = match check(&loaded, opts) {
        Ok(p) => p,
        Err(()) => return 1,
    };
    let stem = stem_of(&opts.file);
    let entry = if prog.main.is_some() { Entry::Main } else { Entry::Library };
    let mut symbols: Vec<(String, String)> =
        prog.funcs.iter().filter(|f| f.body.is_some()).map(|f| (f.mangled.clone(), if f.targs.is_empty() { f.name.clone() } else { format!("{} (instance)", f.name) })).collect();
    for f in prog.funcs.iter().filter(|f| f.export.is_some()) {
        symbols.push((f.name.clone(), format!("{} (export wrapper)", f.name)));
    }
    symbols.push(("main".into(), "main (entry point)".into()));
    let (c, _) = match generate(prog, &loaded, opts.mode, entry, &stem) {
        Ok(x) => x,
        Err(()) => return 1,
    };
    let c_path = match write_c(opts, &format!("{}_size", stem), &c) {
        Ok(p) => p,
        Err(()) => return 1,
    };
    let obj = opts.out_dir.join(format!("{}_size.o", stem));
    let mut o = Opts {
        file: opts.file.clone(),
        mode: opts.mode,
        target: opts.target.clone(),
        out: None,
        out_dir: opts.out_dir.clone(),
        keep_c: true,
        cc: opts.cc.clone(),
        rest: vec![],
        defines: opts.defines.clone(),
        include_dirs: opts.include_dirs.clone(),
        link_libs: opts.link_libs.clone(),
        link_paths: opts.link_paths.clone(),
        c_sources: opts.c_sources.clone(),
    };
    o.defines.push("NX_SIZE_SECTIONS".into());
    if compile_c_sections(&o, &c_path, &obj).is_err() {
        return 1;
    }
    let _ = std::fs::remove_file(&c_path);
    let bytes = match std::fs::read(&obj) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: cannot read {}: {}", obj.display(), e);
            return 1;
        }
    };
    let _ = std::fs::remove_file(&obj);
    let sections = match size::section_sizes(&bytes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {}", e);
            return 1;
        }
    };
    let (entries, runtime, total) = size::attribute(&sections, &symbols);
    println!("{:>9}  {:<5} declaration", "bytes", "kind");
    for e in &entries {
        println!("{:>9}  {:<5} {}", e.bytes, e.kind, e.symbol);
    }
    println!("{:>9}  {:<5} runtime and libc glue", runtime, "");
    println!(
        "{:>9}        total ({} mode; the linker drops unreferenced sections)",
        total,
        match opts.mode {
            BuildMode::Debug => "debug",
            BuildMode::SafeRelease => "safe",
            BuildMode::FastRelease => "fast",
            BuildMode::SmallRelease => "small",
        }
    );
    0
}

/// Compile to an object with one section per function and datum.
fn compile_c_sections(opts: &Opts, c_path: &Path, out: &Path) -> Result<(), ()> {
    let cc = cc_command(opts);
    let mut cmd = Command::new(&cc.program);
    cmd.args(&cc.args);
    cmd.args(["-std=gnu11", "-w", "-fno-strict-aliasing", "-ffunction-sections", "-fdata-sections", "-c"]);
    match opts.mode {
        BuildMode::Debug => cmd.arg("-O0"),
        BuildMode::SmallRelease => cmd.arg("-Os"),
        _ => cmd.arg("-O2"),
    };
    if let Some(t) = &opts.target {
        cmd.arg("-target");
        cmd.arg(t);
    }
    cmd.arg(c_path).arg("-o").arg(out);
    match cmd.status() {
        Ok(s) if s.success() => Ok(()),
        Ok(_) => {
            eprintln!("error: C compilation failed");
            Err(())
        }
        Err(e) => {
            eprintln!("error: cannot run the C compiler `{}`: {}", cc.program, e);
            Err(())
        }
    }
}

fn cmd_audit(opts: &Opts) -> i32 {
    let loaded = match load(&opts.file) {
        Ok(l) => l,
        Err(()) => return 1,
    };
    let mut unsafe_sites: Vec<(diag::Span, usize)> = Vec::new();
    for m in &loaded.modules {
        for item in &m.items {
            audit_item(item, &mut unsafe_sites);
        }
    }
    let sm = &loaded.sm;
    println!("unsafe blocks: {}", unsafe_sites.len());
    for (sp, lines) in &unsafe_sites {
        println!("  {}  ({} line(s))", sm.location(*sp), lines);
    }
    let globals_flag = opts.rest.iter().any(|a| a == "--globals");
    let mut globals = Vec::new();
    for m in &loaded.modules {
        for item in &m.items {
            if let ast::Item::Global(g) = item {
                globals.push((g.name.clone(), g.span));
            }
        }
    }
    println!("mutable globals: {}", globals.len());
    for (n, sp) in &globals {
        println!("  {}  at {}", n, sm.location(*sp));
    }
    if globals_flag && !globals.is_empty() {
        println!("note: a project declaring a `cabi`, `python`, `node`, `rustlib`, or `shared` artifact rejects any function that reaches these (S2)");
    }
    let has_error = check(&loaded, opts).is_err();
    if has_error {
        1
    } else {
        0
    }
}

fn audit_item(item: &ast::Item, out: &mut Vec<(diag::Span, usize)>) {
    match item {
        ast::Item::Fn(f) => {
            if let Some(b) = &f.body {
                audit_block(b, out);
            }
        }
        ast::Item::Impl(im) => {
            for m in &im.methods {
                if let Some(b) = &m.body {
                    audit_block(b, out);
                }
            }
        }
        ast::Item::Trait(t) => {
            for m in &t.methods {
                if let Some(b) = &m.body {
                    audit_block(b, out);
                }
            }
        }
        ast::Item::Test(t) => audit_block(&t.body, out),
        _ => {}
    }
}

fn audit_block(b: &ast::Block, out: &mut Vec<(diag::Span, usize)>) {
    for s in &b.stmts {
        audit_stmt(s, out);
    }
    if let Some(t) = &b.tail {
        audit_expr(t, out);
    }
}

fn audit_stmt(s: &ast::Stmt, out: &mut Vec<(diag::Span, usize)>) {
    use ast::Stmt::*;
    match s {
        Unsafe { body, span } => {
            out.push((*span, body.stmts.len() + body.tail.is_some() as usize));
            audit_block(body, out);
        }
        Let { init: Some(e), .. } | Expr(e) | Return { value: Some(e), .. } | Break { value: Some(e), .. } | Discard { value: e, .. } => audit_expr(e, out),
        Assign { target, value, .. } => {
            audit_expr(target, out);
            audit_expr(value, out);
        }
        Defer { body, .. } | ErrDefer { body, .. } => audit_stmt(body, out),
        While { cond, body, .. } => {
            audit_expr(cond, out);
            audit_block(body, out);
        }
        Using { body, .. } => audit_block(body, out),
        For { body, .. } => audit_block(body, out),
        _ => {}
    }
}

fn audit_expr(e: &ast::Expr, out: &mut Vec<(diag::Span, usize)>) {
    use ast::Expr::*;
    match e {
        Unsafe { body, span } => {
            out.push((*span, body.stmts.len() + body.tail.is_some() as usize));
            audit_block(body, out);
        }
        Block(b) => audit_block(b, out),
        If { cond, then, els, .. } => {
            audit_expr(cond, out);
            audit_block(then, out);
            if let Some(x) = els {
                audit_expr(x, out);
            }
        }
        IfCapture { cond, then, els, .. } => {
            audit_expr(cond, out);
            audit_block(then, out);
            if let Some(x) = els {
                audit_expr(x, out);
            }
        }
        Match { scrutinee, arms, .. } => {
            audit_expr(scrutinee, out);
            for a in arms {
                audit_expr(&a.body, out);
            }
        }
        Call { callee, args, .. } => {
            audit_expr(callee, out);
            for a in args {
                audit_expr(a, out);
            }
        }
        MethodCall { receiver, args, .. } => {
            audit_expr(receiver, out);
            for a in args {
                audit_expr(a, out);
            }
        }
        Binary { lhs, rhs, .. } | Pipe { lhs, rhs, .. } => {
            audit_expr(lhs, out);
            audit_expr(rhs, out);
        }
        Unary { expr, .. } | Try { expr, .. } | Unwrap { expr, .. } | Deref { expr, .. } | Comptime { expr, .. } | Cast { expr, .. } => audit_expr(expr, out),
        Catch { expr, handler, .. } => {
            audit_expr(expr, out);
            audit_expr(handler, out);
        }
        OrElse { expr, default, .. } => {
            audit_expr(expr, out);
            audit_expr(default, out);
        }
        Closure(c) => audit_expr(&c.body, out),
        StructLit { fields, .. } => {
            for (_, v, _) in fields {
                audit_expr(v, out);
            }
        }
        ArrayLit { elems, .. } | TupleLit { elems, .. } => {
            for v in elems {
                audit_expr(v, out);
            }
        }
        _ => {}
    }
}

fn artifact_str(a: &check::ArtifactInfo, key: &str) -> Option<String> {
    a.fields.iter().find(|(k, _)| k == key).and_then(|(_, v)| match v {
        ArtifactValue::Str(s) | ArtifactValue::Ident(s) => Some(s.clone()),
        _ => None,
    })
}

fn cmd_ship(opts: &Opts) -> i32 {
    let loaded = match load(&opts.file) {
        Ok(l) => l,
        Err(()) => return 1,
    };
    let prog = match check(&loaded, opts) {
        Ok(p) => p,
        Err(()) => return 1,
    };
    if prog.artifacts.is_empty() {
        eprintln!("error: no `artifact` declarations found; declare e.g. `artifact cabi {{ name = \"mylib\" }}` or `artifact python {{ name = \"mylib\" }}`");
        return 1;
    }
    let stem = stem_of(&opts.file);
    let artifacts = prog.artifacts.clone();
    let mut produced = Vec::new();
    // one library build serves cabi + python; the cli build is separate
    let lib_arts: Vec<&check::ArtifactInfo> = artifacts.iter().filter(|a| matches!(a.kind.as_str(), "cabi" | "python" | "shared" | "rustlib")).collect();
    let cli_arts: Vec<&check::ArtifactInfo> = artifacts.iter().filter(|a| matches!(a.kind.as_str(), "cli" | "app")).collect();
    for a in &artifacts {
        if !matches!(a.kind.as_str(), "cabi" | "python" | "shared" | "cli" | "app" | "lib" | "rustlib" | "link") {
            eprintln!("note: artifact `{}` is not produced by this version of nx (supported: cabi, python, shared, cli); skipped", a.kind);
        }
    }
    let target = opts.target.clone().unwrap_or_else(host_target);
    if !lib_arts.is_empty() {
        let lib_name = lib_arts.iter().filter_map(|a| artifact_str(a, "name")).next().unwrap_or_else(|| stem.clone());
        let version = lib_arts.iter().filter_map(|a| artifact_str(a, "version")).next().unwrap_or_else(|| "0.1.0".into());
        if prog.exports.is_empty() {
            eprintln!("error: library artifacts need at least one `export(c)` function");
            return 1;
        }
        let (c, infos) = match generate(prog.clone_for_ship(), &loaded, if opts.mode == BuildMode::Debug { BuildMode::SafeRelease } else { opts.mode }, Entry::Library, &lib_name) {
            Ok(x) => x,
            Err(()) => return 1,
        };
        let dir = opts.out_dir.join(&lib_name);
        if std::fs::create_dir_all(&dir).is_err() {
            eprintln!("error: cannot create {}", dir.display());
            return 1;
        }
        let c_path = dir.join(format!("{}.c", lib_name));
        if std::fs::write(&c_path, &c).is_err() {
            eprintln!("error: cannot write {}", c_path.display());
            return 1;
        }
        let header = ship::c_header(&lib_name, &infos);
        let h_path = dir.join(format!("{}.h", lib_name));
        let _ = std::fs::write(&h_path, header);
        // shared library
        let shared_name = if is_windows_target(&opts.target) {
            format!("{}.dll", lib_name)
        } else if is_macos_target(&opts.target) {
            format!("lib{}.dylib", lib_name)
        } else {
            format!("lib{}.so", lib_name)
        };
        let shared_path = dir.join(&shared_name);
        let mut lib_opts = Opts {
            file: opts.file.clone(),
            mode: if opts.mode == BuildMode::Debug { BuildMode::SafeRelease } else { opts.mode },
            target: opts.target.clone(),
            out: None,
            out_dir: opts.out_dir.clone(),
            keep_c: true,
            cc: opts.cc.clone(),
            rest: vec![],
            defines: opts.defines.clone(),
            include_dirs: opts.include_dirs.clone(),
            link_libs: opts.link_libs.clone(),
            link_paths: opts.link_paths.clone(),
            c_sources: opts.c_sources.clone(),
        };
        // the shared library is built in its own directory: on Windows the linker also
        // writes an import library named <name>.lib there, which must not be confused
        // with the static archive produced below
        let shared_dir = dir.join("shared");
        let _ = std::fs::create_dir_all(&shared_dir);
        let shared_build = shared_dir.join(&shared_name);
        if compile_c(&lib_opts, &c_path, &shared_build, "shared").is_err() {
            return 1;
        }
        if std::fs::copy(&shared_build, &shared_path).is_err() {
            eprintln!("error: cannot place {}", shared_path.display());
            return 1;
        }
        produced.push(shared_path.clone());
        // static object; on Windows it targets the MSVC ABI so Rust and MSVC users can link it
        let obj_path = dir.join(if is_windows_target(&opts.target) { format!("{}.obj", lib_name) } else { format!("{}.o", lib_name) });
        lib_opts.keep_c = true;
        if is_windows_target(&opts.target) && opts.target.is_none() && opts.cc.is_none() {
            lib_opts.target = Some("x86_64-windows-msvc".into());
        }
        if compile_c(&lib_opts, &c_path, &obj_path, "object").is_ok() {
            // archive with zig ar when available
            let ar_path = dir.join(if is_windows_target(&opts.target) { format!("{}.lib", lib_name) } else { format!("lib{}.a", lib_name) });
            let _ = std::fs::remove_file(&ar_path);
            let cc = cc_command(opts);
            let ok = if cc.program == "zig" {
                Command::new("zig").args(["ar", "rcs"]).arg(&ar_path).arg(&obj_path).status().map(|s| s.success()).unwrap_or(false)
            } else {
                Command::new("ar").args(["rcs"]).arg(&ar_path).arg(&obj_path).status().map(|s| s.success()).unwrap_or(false)
            };
            if ok {
                produced.push(ar_path);
            }
        }
        produced.push(h_path);
        // rust crate wrapping the static library
        if lib_arts.iter().any(|a| a.kind == "rustlib") {
            let ar_name = if is_windows_target(&opts.target) { format!("{}.lib", lib_name) } else { format!("lib{}.a", lib_name) };
            let crate_dir = dir.join("rust");
            let _ = std::fs::create_dir_all(crate_dir.join("src"));
            let _ = std::fs::create_dir_all(crate_dir.join("lib"));
            if let Ok(bytes) = std::fs::read(dir.join(&ar_name)) {
                let _ = std::fs::write(crate_dir.join("lib").join(&ar_name), bytes);
            }
            for (path, content) in ship::rust_crate(&lib_name, &version, &infos, &ar_name) {
                let p = crate_dir.join(&path);
                if let Err(e) = std::fs::write(&p, content) {
                    eprintln!("error: cannot write {}: {}", p.display(), e);
                    return 1;
                }
            }
            produced.push(crate_dir.join("Cargo.toml"));
        }
        // python package + wheel
        if lib_arts.iter().any(|a| a.kind == "python") {
            let py_name = lib_arts.iter().filter(|a| a.kind == "python").filter_map(|a| artifact_str(a, "name")).next().unwrap_or_else(|| lib_name.clone()).replace('-', "_");
            let pkg_dir = dir.join("python").join(&py_name);
            if std::fs::create_dir_all(&pkg_dir).is_err() {
                eprintln!("error: cannot create {}", pkg_dir.display());
                return 1;
            }
            let module = ship::python_module(&lib_name, &version, &infos, &shared_name);
            let _ = std::fs::write(pkg_dir.join("__init__.py"), &module);
            let stub = ship::python_stub(&infos);
            let _ = std::fs::write(pkg_dir.join("__init__.pyi"), &stub);
            let _ = std::fs::write(pkg_dir.join("py.typed"), "");
            let lib_bytes = std::fs::read(&shared_path).unwrap_or_default();
            let _ = std::fs::write(pkg_dir.join(&shared_name), &lib_bytes);
            let _ = std::fs::write(dir.join("python").join("pyproject.toml"), format!("[project]\nname = \"{}\"\nversion = \"{}\"\nrequires-python = \">=3.8\"\n\n[build-system]\nrequires = [\"setuptools\"]\nbuild-backend = \"setuptools.build_meta\"\n\n[tool.setuptools]\npackages = [\"{}\"]\n\n[tool.setuptools.package-data]\n\"{}\" = [\"*.dll\", \"*.so\", \"*.dylib\", \"*.pyi\", \"py.typed\"]\n", py_name, version, py_name, py_name));
            let files = vec![
                (format!("{}/__init__.py", py_name), module.into_bytes()),
                (format!("{}/__init__.pyi", py_name), stub.into_bytes()),
                (format!("{}/py.typed", py_name), Vec::new()),
                (format!("{}/{}", py_name, shared_name), lib_bytes),
            ];
            let tag = ship::platform_tag(&target);
            match ship::build_wheel(&dir, &py_name, &version, &tag, &files, &format!("{} (built with Nexium)", py_name)) {
                Ok(w) => produced.push(w),
                Err(e) => {
                    eprintln!("error: cannot write wheel: {}", e);
                    return 1;
                }
            }
        }
    }
    if !cli_arts.is_empty() {
        let name = cli_arts.iter().filter_map(|a| artifact_str(a, "name")).next().unwrap_or_else(|| stem.clone());
        let mut o = Opts {
            file: opts.file.clone(),
            mode: if opts.mode == BuildMode::Debug { BuildMode::SafeRelease } else { opts.mode },
            target: opts.target.clone(),
            out: Some(opts.out_dir.join(&name).join(exe_name(&name, &opts.target))),
            out_dir: opts.out_dir.join(&name),
            keep_c: false,
            cc: opts.cc.clone(),
            rest: vec![],
            defines: opts.defines.clone(),
            include_dirs: opts.include_dirs.clone(),
            link_libs: opts.link_libs.clone(),
            link_paths: opts.link_paths.clone(),
            c_sources: opts.c_sources.clone(),
        };
        let _ = std::fs::create_dir_all(&o.out_dir);
        match cmd_build(&o, false) {
            Ok(p) => produced.push(p),
            Err(()) => return 1,
        }
        o.keep_c = false;
    }
    println!("shipped {} artifact file(s) for {}:", produced.len(), target);
    for p in produced {
        println!("  {}", p.display());
    }
    0
}

impl Program {
    fn clone_for_ship(self) -> Program {
        self
    }
}

fn main() {
    // Deeply nested source (long else-if chains, big match arms) recurses deeply
    // in the checker and the C emitter; run on a thread with a generous stack,
    // as every compiler that walks trees recursively does.
    let child = std::thread::Builder::new().stack_size(512 << 20).spawn(real_main).expect("spawn compiler thread");
    let code = child.join().unwrap_or(101);
    exit(code);
}

fn real_main() -> i32 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        // like `python`: no arguments at a terminal opens the REPL
        if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
            return repl::run();
        }
        usage();
    }
    let cmd = args[0].as_str();
    let rest = &args[1..];
    let code = match cmd {
        "version" | "--version" | "-V" => {
            println!("nx {}", VERSION);
            0
        }
        "doctor" => cmd_doctor(),
        "repl" => repl::run(),
        "tokens" => {
            let o = parse_opts(rest);
            match std::fs::read_to_string(&o.file) {
                Ok(text) => {
                    let (toks, diags) = lexer::Lexer::new(&text, 0).lex();
                    for t in &toks {
                        println!("{}", token_line(t, &text));
                    }
                    for d in &diags {
                        eprintln!("error at {}: {}", d.span.start, d.msg);
                    }
                    if diags.is_empty() {
                        0
                    } else {
                        1
                    }
                }
                Err(e) => {
                    eprintln!("error: cannot read {}: {}", o.file.display(), e);
                    1
                }
            }
        }
        "parse" => {
            let o = parse_opts(rest);
            match load(&o.file) {
                Ok(l) => {
                    for m in &l.modules {
                        println!("{:#?}", m);
                    }
                    0
                }
                Err(()) => 1,
            }
        }
        "check" => {
            let o = parse_opts(rest);
            match load(&o.file).and_then(|l| check(&l, &o).map(|_| ())) {
                Ok(()) => {
                    println!("ok");
                    0
                }
                Err(()) => 1,
            }
        }
        "build" => {
            let mut o = parse_opts(rest);
            if !rest.iter().any(|a| a == "--mode") {
                o.mode = BuildMode::SafeRelease;
            }
            match cmd_build(&o, false) {
                Ok(p) => {
                    println!("built {}", p.display());
                    0
                }
                Err(()) => 1,
            }
        }
        "run" => {
            let o = parse_opts(rest);
            cmd_run(&o)
        }
        "test" => {
            let o = parse_opts(rest);
            cmd_test(&o)
        }
        "effects" => {
            let o = parse_opts(rest);
            cmd_effects(&o)
        }
        "refcounts" => {
            let o = parse_opts(rest);
            cmd_refcounts(&o)
        }
        "fmt" => {
            let o = parse_opts(rest);
            cmd_fmt(&o)
        }
        "lsp" => cmd_lsp(),
        "doc" => {
            let o = parse_opts(rest);
            cmd_doc(&o)
        }
        "size" => {
            let mut o = parse_opts(rest);
            if !rest.iter().any(|a| a == "--mode") {
                o.mode = BuildMode::SmallRelease;
            }
            cmd_size(&o)
        }
        "leaks" => {
            let mut o = parse_opts(rest);
            o.mode = BuildMode::Debug;
            o.defines.push("NX_LEAK_CHECK".into());
            cmd_run(&o)
        }
        "audit" => {
            let o = parse_opts(rest);
            cmd_audit(&o)
        }
        "ship" => {
            let mut o = parse_opts(rest);
            if !rest.iter().any(|a| a == "--mode") {
                o.mode = BuildMode::SafeRelease;
            }
            cmd_ship(&o)
        }
        "emit-c" => {
            let o = parse_opts(rest);
            match load(&o.file) {
                Ok(l) => match check(&l, &o) {
                    Ok(p) => {
                        let has_main = p.main.is_some();
                        let has_tests = !p.tests.is_empty() && !has_main;
                        let entry = if has_main {
                            Entry::Main
                        } else if has_tests {
                            Entry::Tests
                        } else {
                            Entry::Library
                        };
                        let stem = stem_of(&o.file);
                        match generate(p, &l, o.mode, entry, &stem) {
                            Ok((c, _)) => {
                                print!("{}", c);
                                0
                            }
                            Err(()) => 1,
                        }
                    }
                    Err(()) => 1,
                },
                Err(()) => 1,
            }
        }
        "help" | "--help" | "-h" => usage(),
        _ => {
            eprintln!("error: unknown command `{}`", cmd);
            usage()
        }
    };
    code
}
