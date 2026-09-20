//! End-to-end tests. The compiler under test is `nx` written in Nexium
//! (`self/nx.nx`), built with no Rust involved: a C compiler builds `nx0`
//! from the seed `bootstrap/nx.c`, and `nx0` builds the current sources
//! (decision 90). Every program in `examples/` and `tests/spec/` must
//! reproduce its recorded output through it, every file in
//! `tests/compile_fail/` must be rejected with the diagnostic on its
//! `// EXPECT:` line, and the compiler must rebuild itself to a fixed point.
//! The Rust binary (`bootstrap/rust/`, frozen) drives only the tools that
//! are still written in Rust: `ship`, `fmt`, the REPL, packages, installers.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// The frozen Rust binary: the tools that are still in Rust.
fn nx() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_nx"))
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn have_cc() -> bool {
    Command::new("zig").arg("version").output().map(|o| o.status.success()).unwrap_or(false)
}

/// The C compiler that links programs on this host: zig, except on macOS,
/// where zig 0.14 cannot link against the current Xcode SDK and the drivers
/// use the system compiler. Preprocessing (`@cImport`) always goes through
/// zig, on every host, so that every run sees the same headers.
fn host_cc() -> Command {
    if cfg!(target_os = "macos") {
        Command::new("cc")
    } else {
        let mut c = Command::new("zig");
        c.arg("cc");
        c
    }
}

/// Point a self-hosted driver at the linker `host_cc` names.
fn driver_cc(cmd: &mut Command) -> &mut Command {
    if cfg!(target_os = "macos") {
        cmd.env("NX_CC", "cc")
    } else {
        cmd.env("NX_ZIG", "zig")
    }
}

fn exe(name: &str) -> String {
    if cfg!(windows) {
        format!("{}.exe", name)
    } else {
        name.to_string()
    }
}

/// The compiler under test, built once per test process with no Rust:
/// `bootstrap/nx.c` -> `nx0` (a C compiler), `nx0` builds `self/nx.nx` ->
/// `nx1`. Every language suite runs through `nx1`.
fn nx_self() -> PathBuf {
    static NX2: OnceLock<PathBuf> = OnceLock::new();
    NX2.get_or_init(|| {
        let out_dir = root().join("nx-out").join("bootstrap");
        let _ = std::fs::create_dir_all(&out_dir);
        let nx0 = out_dir.join(exe("nx0"));
        build_c(&root().join("bootstrap").join("nx.c"), &nx0, "the seed bootstrap/nx.c");
        let nx1 = out_dir.join(exe("nx1"));
        let out = driver_cc(&mut Command::new(&nx0)).args(["build", "self/nx.nx", "--mode", "safe", "-o"]).arg(&nx1).arg("--out-dir").arg(&out_dir).current_dir(root()).output().expect("run nx0");
        assert!(out.status.success(), "the seed compiler could not build self/nx.nx:\n{}", String::from_utf8_lossy(&out.stderr));
        // nx1 runs on the runtime the seed carries; nx2, built from nx1's own C, runs on the one in the tree
        let emit = driver_cc(&mut Command::new(&nx1)).args(["emit-c", "self/nx.nx", "--mode", "safe"]).current_dir(root()).output().expect("run nx1");
        assert!(emit.status.success(), "nx1 could not emit self/nx.nx:\n{}", String::from_utf8_lossy(&emit.stderr));
        let c1 = out_dir.join("nx1.c");
        std::fs::write(&c1, &emit.stdout).unwrap();
        let nx2 = out_dir.join(exe("nx2"));
        build_c(&c1, &nx2, "nx1.c");
        nx2
    })
    .clone()
}

/// The host C compiler builds one emitted file into an executable.
fn build_c(c_file: &Path, exe_path: &Path, what: &str) {
    let mut cc = host_cc();
    cc.args(["-std=gnu11", "-O2", "-w", "-fno-strict-aliasing", "-o"]).arg(exe_path).arg(c_file);
    if cfg!(windows) {
        cc.arg("-lws2_32");
    } else if !cfg!(target_os = "macos") {
        cc.args(["-lm", "-lc"]);
    }
    let out = cc.current_dir(root()).output().expect("run the C compiler");
    assert!(out.status.success(), "{} did not build:\n{}", what, String::from_utf8_lossy(&out.stderr));
}

/// A command for the compiler under test, run from the repository root with
/// the host's C compiler. Each test gets its own Zig cache directory: the
/// shared one is not safe against several `zig cc` starting at once on
/// Windows, and the tests run in parallel.
fn nxs() -> Command {
    let mut c = Command::new(nx_self());
    driver_cc(&mut c);
    let test = std::thread::current().name().unwrap_or("main").replace("::", "_");
    c.env("ZIG_LOCAL_CACHE_DIR", root().join("nx-out").join("zig-cache").join(test));
    c.current_dir(root());
    c
}

fn normalize(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\\', "/")
}

/// Two oracle outputs must match; on a mismatch, name the first line that
/// differs (the whole texts run to megabytes, which CI logs drop).
fn assert_same_text(expected: &str, got: &str, what: &str) {
    if expected == got {
        return;
    }
    let (el, gl): (Vec<&str>, Vec<&str>) = (expected.lines().collect(), got.lines().collect());
    let first = el.iter().zip(gl.iter()).position(|(a, b)| a != b).unwrap_or(el.len().min(gl.len()));
    panic!(
        "{} at line {} ({} vs {} lines):
  oracle: {}
  mine:   {}",
        what,
        first + 1,
        el.len(),
        gl.len(),
        el.get(first).unwrap_or(&"<end>"),
        gl.get(first).unwrap_or(&"<end>")
    );
}

fn run_example(name: &str, subcommand: &str) {
    let src = PathBuf::from("examples").join(format!("{}.nx", name)); // relative: diagnostics print this path
    let expected_path = root().join("examples").join(format!("{}.expected", name));
    let out_dir = std::env::temp_dir().join(format!("nx-test-{}-{}", name, std::process::id()));
    let out = nxs().arg(subcommand).arg(&src).arg("--out-dir").arg(&out_dir).output().expect("run nx");
    let got = normalize(&format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)));
    let expected = normalize(&std::fs::read_to_string(&expected_path).unwrap_or_default());
    let _ = std::fs::remove_dir_all(&out_dir);
    assert_same_text(expected.trim(), got.trim(), &format!("output of examples/{}.nx differs", name));
}

#[test]
fn examples_reproduce_recorded_output() {
    if !have_cc() {
        eprintln!("skipping: zig not found");
        return;
    }
    for name in [
        "hello",
        "tour",
        "binary",
        "ownership",
        "generics",
        "control",
        "ctest",
        "arena",
        "dyn",
        "parallel",
        "cimport",
        "process",
        "tree",
        "own",
        "stdlib",
        "json",
        "guard_scope",
        "records",
        "binary_sizes",
    ] {
        run_example(name, "run");
    }
    run_example("tests", "test");
}

/// The specification's conformance cases: `tests/spec/<section>_*.nx`, one
/// per claim SPEC.md makes, each with its recorded output and exit code
/// (`// EXIT: n`, 0 when absent).
#[test]
fn spec_cases_reproduce_recorded_output() {
    if !have_cc() {
        eprintln!("skipping: zig not found");
        return;
    }
    let dir = root().join("tests").join("spec");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir).expect("spec dir").filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|x| x == "nx").unwrap_or(false)).collect();
    entries.sort();
    assert!(!entries.is_empty());
    let out_dir = std::env::temp_dir().join(format!("nx-spec-{}", std::process::id()));
    for path in entries {
        let text = std::fs::read_to_string(&path).unwrap();
        let want_code: i32 = text.lines().find_map(|l| l.strip_prefix("// EXIT:")).map(|s| s.trim().parse().unwrap()).unwrap_or(0);
        let rel = PathBuf::from("tests").join("spec").join(path.file_name().unwrap());
        let out = nxs().arg("run").arg(&rel).arg("--out-dir").arg(&out_dir).output().expect("run nx");
        let got = normalize(&format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)));
        let expected = normalize(&std::fs::read_to_string(path.with_extension("expected")).unwrap_or_default());
        assert_same_text(expected.trim(), got.trim(), &format!("output of {} differs", rel.display()));
        assert_eq!(out.status.code().unwrap_or(-1), want_code, "exit code of {} differs", rel.display());
    }
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn compile_fail_cases_are_rejected() {
    if !have_cc() {
        eprintln!("skipping: zig not found");
        return;
    }
    let dir = root().join("tests").join("compile_fail");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir).expect("compile_fail dir").filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|x| x == "nx").unwrap_or(false)).collect();
    entries.sort();
    assert!(!entries.is_empty());
    for path in entries {
        let text = std::fs::read_to_string(&path).unwrap();
        let expects: Vec<&str> = text.lines().filter_map(|l| l.strip_prefix("// EXPECT:")).map(|s| s.trim()).collect();
        assert!(!expects.is_empty(), "{} has no // EXPECT: line", path.display());
        let rel = PathBuf::from("tests").join("compile_fail").join(path.file_name().unwrap());
        let out = nxs().arg("check").arg(&rel).output().expect("run nx");
        assert!(!out.status.success(), "{} was accepted but should fail", path.display());
        let diag = normalize(&String::from_utf8_lossy(&out.stderr));
        for e in expects {
            assert!(diag.contains(e), "{}: expected diagnostic containing `{}`, got:\n{}", path.display(), e, diag);
        }
    }
}

#[test]
fn ship_produces_library_and_header() {
    if !have_cc() {
        eprintln!("skipping: zig not found");
        return;
    }
    let out_dir = std::env::temp_dir().join(format!("nx-ship-{}", std::process::id()));
    let out = nxs().arg("ship").arg(PathBuf::from("examples").join("ropesim.nx")).arg("--out-dir").arg(&out_dir).output().expect("run nx ship");
    assert!(out.status.success(), "ship failed: {}", String::from_utf8_lossy(&out.stderr));
    let lib = out_dir.join("ropesim");
    assert!(lib.join("ropesim.h").exists());
    assert!(lib.join("python").join("ropesim").join("__init__.py").exists());
    let has_wheel = std::fs::read_dir(&lib).unwrap().any(|e| e.unwrap().path().extension().map(|x| x == "whl").unwrap_or(false));
    assert!(has_wheel, "no wheel produced");
    // the generated Rust crate must build
    let crate_dir = lib.join("rust");
    assert!(crate_dir.join("Cargo.toml").exists(), "no rust crate produced");
    let build = Command::new("cargo").arg("build").arg("-q").current_dir(&crate_dir).output().expect("run cargo");
    assert!(build.status.success(), "generated crate does not build: {}", String::from_utf8_lossy(&build.stderr));
    // the npm package: generated files present, and when npm is around, it runs
    let node_dir = lib.join("node");
    assert!(node_dir.join("index.js").exists() && node_dir.join("index.d.ts").exists() && node_dir.join("package.json").exists(), "no node package produced");
    let dts = std::fs::read_to_string(node_dir.join("index.d.ts")).unwrap();
    assert!(dts.contains("export function dot(a: Float64Array | ArrayLike<number>, b: Float64Array | ArrayLike<number>): number;"), "typings: {}", dts);
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    if Command::new(npm).arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
        let install = Command::new(npm).args(["install", "--silent", "--no-audit", "--no-fund"]).current_dir(&node_dir).output().expect("run npm");
        if install.status.success() {
            let script = "const m = require('.'); const pos = new Float64Array([0, 1, 2]); const r = m.simulate(pos, 0.1, 3); let err = ''; try { m.dot([1, 2], [1]); } catch (e) { err = e.errorName; } let panic = ''; try { m.divide(1n, 0n); } catch (e) { panic = e.name; } console.log(JSON.stringify({ dot: m.dot([1, 2, 3], [4, 5, 6]), sum: m.checksum('hello') > 0, moved: pos[1] !== 1, r: r > 0, err, panic, div: String(m.divide(84n, 2n)) }));";
            let run = Command::new("node").args(["-e", script]).current_dir(&node_dir).output().expect("run node");
            let out = String::from_utf8_lossy(&run.stdout);
            assert!(run.status.success(), "node package failed: {}", String::from_utf8_lossy(&run.stderr));
            assert_eq!(out.trim(), r#"{"dot":32,"sum":true,"moved":true,"r":true,"err":"InvalidInput","panic":"NexiumPanic","div":"42"}"#);
        } else {
            eprintln!("skipping the node run: npm install failed (offline?)");
        }
    }
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[allow(dead_code)]
fn exists(p: &Path) -> bool {
    p.exists()
}

/// The compiler rebuilds itself to a fixed point with no Rust involved:
/// `nx1` (built by the seed) emits the C of `self/nx.nx`, a C compiler
/// builds `nx2` from it, and `nx2` emits the same C. The seed itself is
/// that C as of the last release; a note says when it is due for
/// regeneration (`nx emit-c self/nx.nx --mode safe > bootstrap/nx.c`).
#[test]
fn bootstrap_reaches_a_fixed_point() {
    if !have_cc() {
        eprintln!("skipping: zig not found");
        return;
    }
    let out_dir = root().join("nx-out").join("bootstrap");
    // nx_self() built nx2 from nx1's C; nx2 must emit that same C
    let c1 = std::fs::read(out_dir.join("nx1.c")).unwrap();
    let again = nxs().args(["emit-c", "self/nx.nx", "--mode", "safe"]).output().unwrap();
    assert!(again.status.success(), "nx2 could not emit self/nx.nx:\n{}", String::from_utf8_lossy(&again.stderr));
    assert_same_text(&String::from_utf8_lossy(&c1), &String::from_utf8_lossy(&again.stdout), "nx1 and nx2 emit different C for self/nx.nx");
    // nx2 is a working compiler, not just a matching emitter
    let hello = nxs().args(["run", "examples/hello.nx", "--out-dir"]).arg(&out_dir).output().unwrap();
    assert!(hello.status.success(), "nx2 could not run hello:\n{}", String::from_utf8_lossy(&hello.stderr));
    let expected = std::fs::read_to_string(root().join("examples").join("hello.expected")).unwrap();
    assert_same_text(normalize(&expected).trim(), normalize(&String::from_utf8_lossy(&hello.stdout)).trim(), "hello through nx2");
    let seed = std::fs::read(root().join("bootstrap").join("nx.c")).unwrap();
    if seed != c1 {
        eprintln!("note: bootstrap/nx.c is behind self/: regenerate it at the release (nx emit-c self/nx.nx --mode safe > bootstrap/nx.c)");
    }
}

/// The GUI library's headless tests (rasterizer and widget interaction) must pass;
/// this also compiles gui/platform.c on every platform (the non-Windows stub included).
#[test]
fn gui_headless_tests_pass() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = nxs().args(["test", "gui/nexium_gui.nx"]).output().expect("run nx");
    assert!(
        out.status.success(),
        "gui tests failed:
{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let shot = root.join("nx-out").join("demo-shot.ppm");
    let out = nxs().args(["run", "gui/demo.nx", "--", "--shot"]).arg(&shot).output().expect("run nx");
    assert!(
        out.status.success(),
        "offscreen demo failed:
{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(std::fs::metadata(&shot).map(|m| m.len() > 640 * 440 * 3).unwrap_or(false), "screenshot not written");
    // the same build from inside the directory, with a bare file name: the
    // declared C source must still be compiled in
    let out = driver_cc(&mut Command::new(nx_self())).args(["build", "demo.nx", "--out-dir"]).arg(root.join("nx-out").join("demo-bare")).current_dir(root.join("gui")).output().expect("run nx");
    assert!(
        out.status.success(),
        "build with a bare file name failed:
{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Every module of the standard library (written in Nexium, embedded in the
/// compiler) passes its own tests and is canonically formatted.
#[test]
fn std_modules_pass_their_tests() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files: Vec<std::path::PathBuf> =
        std::fs::read_dir(root.join("std")).unwrap().filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|x| x == "nx").unwrap_or(false)).collect();
    files.sort();
    assert!(!files.is_empty());
    for f in files {
        let out = nxs().arg("test").arg(&f).output().unwrap();
        assert!(out.status.success(), "std tests failed for {}:\n{}{}", f.display(), String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        let fmt = nxs().arg("fmt").arg(&f).arg("--check").output().unwrap();
        assert!(fmt.status.success(), "{} is not canonically formatted", f.display());
    }
}

/// The REPL evaluates lines, keeps bindings, accepts items, reports errors, and prints.
#[test]
fn repl_session() {
    use std::io::Write;
    let script = "let x = 2\nx * 21\nfn sq(n: i32) -> i32 { return n * n }\nsq(x)\nprintln(\"hi {}\", .{x})\nimport std.strings\nstrings.to_upper(\"ok\")\nlet s = String.from(\"a\")\ns\nundefined_name\nvar xs = List(i32).new()\nxs.append(7)\nxs\n:quit\n";
    let mut child = nxs().arg("repl").stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(script.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "repl exited with {:?}\n{}", out.status, stderr);
    for expected in ["i64: 42", "i32: 4", "hi 2", "String: \"OK\"", "String: \"a\"", "List(i32): [7]"] {
        assert!(stdout.contains(expected), "missing {:?} in:\n{}", expected, stdout);
    }
    assert!(stderr.contains("undefined_name"), "the error for an unknown name was not reported:\n{}", stderr);
}

/// `nx fmt --check` passes on every source in the tree: formatting is
/// idempotent and the sources are canonical. The formatter in Nexium
/// (`self/fmt.nx`) is also held to the frozen Rust one while that exists:
/// both must produce the same text for every source after its layout is
/// disturbed.
#[test]
fn examples_are_canonically_formatted() {
    if !have_cc() {
        eprintln!("skipping: zig not found");
        return;
    }
    let mut files: Vec<PathBuf> = Vec::new();
    for dir in ["examples", "std", "gui", "self", "tests/spec", "tests/compile_fail"] {
        files.extend(std::fs::read_dir(root().join(dir)).unwrap().filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|x| x == "nx").unwrap_or(false)));
    }
    files.sort();
    let mut cmd = nxs();
    cmd.arg("fmt");
    for f in &files {
        cmd.arg(f);
    }
    cmd.arg("--check");
    let out = cmd.output().expect("run nx fmt");
    assert!(
        out.status.success(),
        "unformatted examples:
{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let scratch = root().join("nx-out").join("fmt-oracle");
    let _ = std::fs::create_dir_all(&scratch);
    for f in &files {
        let src = std::fs::read_to_string(f).unwrap();
        let disturbed: String = src.lines().map(|l| l.trim().replace(", ", " , ").replace(" = ", "=")).collect::<Vec<_>>().join("\n");
        let a = scratch.join("a.nx");
        let b = scratch.join("b.nx");
        std::fs::write(&a, &disturbed).unwrap();
        std::fs::write(&b, &disturbed).unwrap();
        let ra = Command::new(nx()).arg("fmt").arg(&a).current_dir(root()).output().unwrap();
        let rb = nxs().arg("fmt").arg(&b).output().unwrap();
        assert_eq!(ra.status.code(), rb.status.code(), "fmt exit codes differ for {}", f.display());
        assert_same_text(&std::fs::read_to_string(&a).unwrap(), &std::fs::read_to_string(&b).unwrap(), &format!("the two formatters disagree on {}", f.display()));
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

/// Packages: a manifest with a path dependency, `import dep` (src/lib.nx),
/// `import dep.module`, a package importing its own sibling, and a
/// transitive dependency.
#[test]
fn packages_resolve_path_dependencies() {
    let root = root();
    let base = root.join("nx-out").join("pkg-test");
    let _ = std::fs::remove_dir_all(&base);
    let write = |rel: &str, text: &str| {
        let p = base.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    };
    write("words/nexium.toml", "[package]\nname = \"words\"\nversion = \"0.1.0\"\n");
    write("words/src/lib.nx", "pub fn planet() -> []u8 { return \"world\" }\n");
    write("greet/nexium.toml", "[package]\nname = \"greet\"\nversion = \"0.1.0\"\n\n[dependencies]\nwords = { path = \"../words\" }\n");
    write("greet/src/lib.nx", "import util\nimport words\npub fn hello() -> String { return util.wrap(words.planet()) }\n");
    write("greet/src/util.nx", "pub fn wrap(s: []u8) -> String { var out = String.from(\"hello, \"); out.append(s); return out }\n");
    write("greet/src/extra.nx", "pub fn punct() -> []u8 { return \"!\" }\n");
    write("app/nexium.toml", "[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[dependencies]\ngreet = { path = \"../greet\" }\n");
    write("app/main.nx", "import greet\nimport greet.extra\nfn main() {\n    println(\"{}{}\", .{greet.hello(), extra.punct()})\n}\n");
    let out = driver_cc(&mut Command::new(nx_self())).args(["run", "main.nx"]).current_dir(base.join("app")).output().expect("run nx");
    assert!(out.status.success(), "package program failed:\n{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "hello, world!");
    let fetch = Command::new(nx_self()).arg("fetch").current_dir(base.join("app")).output().expect("run nx fetch");
    assert!(fetch.status.success(), "nx fetch failed:\n{}", String::from_utf8_lossy(&fetch.stderr));
    let lock = std::fs::read_to_string(base.join("app").join("nexium.lock")).unwrap();
    assert!(lock.contains("greet\tpath\t../greet"), "lock file: {}", lock);
    assert!(lock.contains("words\tpath"), "transitive dependency missing from the lock: {}", lock);
}

/// `artifact installer`: `nx ship` writes an Inno Setup script on Windows and
/// an install script elsewhere, next to the built program, listing its files.
#[test]
fn installer_artifact_writes_scripts() {
    let root = root();
    let base = root.join("nx-out").join("installer-test");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("assets")).unwrap();
    std::fs::write(
        base.join("assets").join("data.txt"),
        "data
",
    )
    .unwrap();
    std::fs::write(
        base.join("LICENSE"),
        "MIT
",
    )
    .unwrap();
    std::fs::write(
        base.join("README.md"),
        "# Greeter
",
    )
    .unwrap();
    std::fs::write(
        base.join("greeter.nx"),
        "artifact cli { name = \"greeter\" }
artifact installer { name = \"Greeter\", publisher = \"Londopy\", version = \"1.0.0\", license = \"LICENSE\", readme = \"README.md\", files = [\"assets\"], add_to_path = true }
fn main() { println(\"hi\", .{}) }
",
    )
    .unwrap();
    // an installed Inno Setup would also compile the script; the script itself is what this checks
    let out = driver_cc(&mut Command::new(nx_self())).args(["ship", "greeter.nx"]).env("ISCC", "").current_dir(&base).output().expect("run nx ship");
    assert!(
        out.status.success(),
        "nx ship failed:
{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let dir = base.join("nx-out").join("greeter");
    assert!(dir.join("assets").join("data.txt").exists(), "files were not copied next to the program");
    if cfg!(windows) {
        let iss = std::fs::read_to_string(dir.join("Greeter.iss")).expect("Greeter.iss");
        assert!(iss.contains("AppName=Greeter") && iss.contains("AppVersion=1.0.0") && iss.contains("addtopath") && iss.contains(r"assets\*"), "script: {}", iss);
        assert!(iss.contains("AppId={{"), "the app id must escape its brace for Inno: {}", iss);
    } else {
        let sh = std::fs::read_to_string(dir.join("install.sh")).expect("install.sh");
        assert!(sh.contains("copy_tree 'assets'") && sh.contains("exe='greeter'"), "script: {}", sh);
        assert!(dir.join("greeter-1.0.0-linux.tar.gz").exists() || dir.join("greeter-1.0.0-macos.tar.gz").exists(), "no archive in {}", dir.display());
    }
}
