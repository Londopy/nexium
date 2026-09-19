//! End-to-end tests: every program in `examples/` must build and reproduce its
//! recorded output; every file in `tests/compile_fail/` must be rejected with
//! the diagnostic named on its `// EXPECT:` line.

use std::path::{Path, PathBuf};
use std::process::Command;

fn nx() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_nx"))
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn have_cc() -> bool {
    Command::new("zig").arg("version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn normalize(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\\', "/")
}

fn run_example(name: &str, subcommand: &str) {
    let src = PathBuf::from("examples").join(format!("{}.nx", name)); // relative: diagnostics print this path
    let expected_path = root().join("examples").join(format!("{}.expected", name));
    let out_dir = std::env::temp_dir().join(format!("nx-test-{}-{}", name, std::process::id()));
    let out = Command::new(nx()).arg(subcommand).arg(&src).arg("--out-dir").arg(&out_dir).current_dir(root()).output().expect("run nx");
    let got = normalize(&format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr)));
    let expected = normalize(&std::fs::read_to_string(&expected_path).unwrap_or_default());
    let _ = std::fs::remove_dir_all(&out_dir);
    assert_eq!(got.trim(), expected.trim(), "output of examples/{}.nx differs", name);
}

#[test]
fn examples_reproduce_recorded_output() {
    if !have_cc() {
        eprintln!("skipping: zig not found");
        return;
    }
    for name in ["hello", "tour", "binary", "ownership", "generics", "control", "ctest", "arena", "dyn", "parallel", "cimport", "process", "tree", "own", "stdlib", "json", "guard_scope", "records"] {
        run_example(name, "run");
    }
    run_example("tests", "test");
}

#[test]
fn compile_fail_cases_are_rejected() {
    let dir = root().join("tests").join("compile_fail");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir).expect("compile_fail dir").filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|x| x == "nx").unwrap_or(false)).collect();
    entries.sort();
    assert!(!entries.is_empty());
    for path in entries {
        let text = std::fs::read_to_string(&path).unwrap();
        let expects: Vec<&str> = text.lines().filter_map(|l| l.strip_prefix("// EXPECT:")).map(|s| s.trim()).collect();
        assert!(!expects.is_empty(), "{} has no // EXPECT: line", path.display());
        let out = Command::new(nx()).arg("check").arg(&path).current_dir(root()).output().expect("run nx");
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
    let out = Command::new(nx()).arg("ship").arg(PathBuf::from("examples").join("ropesim.nx")).arg("--out-dir").arg(&out_dir).current_dir(root()).output().expect("run nx ship");
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

/// The self-hosted lexer (self/lexer.nx) must produce exactly the token stream
/// of the Rust lexer for every example, for its own source, and with the same
/// exit code.
#[test]
fn self_hosted_lexer_matches_oracle() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let exe = root.join("nx-out").join(if cfg!(windows) { "self_lexer.exe" } else { "self_lexer" });
    let build = std::process::Command::new(env!("CARGO_BIN_EXE_nx")).args(["build", "self/lexer.nx", "-o"]).arg(&exe).current_dir(root).output().expect("run nx");
    assert!(build.status.success(), "building self/lexer.nx failed:\n{}", String::from_utf8_lossy(&build.stderr));
    let mut files: Vec<std::path::PathBuf> =
        std::fs::read_dir(root.join("examples")).unwrap().filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|x| x == "nx").unwrap_or(false)).collect();
    files.push(root.join("self").join("lexer.nx"));
    files.sort();
    for f in files {
        let oracle = std::process::Command::new(env!("CARGO_BIN_EXE_nx")).arg("tokens").arg(&f).output().unwrap();
        let mine = std::process::Command::new(&exe).arg(&f).output().unwrap();
        assert_eq!(String::from_utf8_lossy(&oracle.stdout), String::from_utf8_lossy(&mine.stdout), "token stream differs for {}", f.display());
        assert_eq!(oracle.status.code(), mine.status.code(), "exit code differs for {}", f.display());
    }
}

/// The parser written in Nexium prints the same tree as `nx sexp` for every
/// example, std module, GUI and self-hosting source (phase 4 of the roadmap).
#[test]
fn self_hosted_parser_matches_oracle() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let exe = root.join("nx-out").join(if cfg!(windows) { "self_parser.exe" } else { "self_parser" });
    let build = std::process::Command::new(env!("CARGO_BIN_EXE_nx")).args(["build", "self/parser.nx", "-o"]).arg(&exe).current_dir(root).output().expect("run nx");
    assert!(
        build.status.success(),
        "building self/parser.nx failed:
{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    for dir in ["examples", "std", "gui", "self", "tests"] {
        files.extend(std::fs::read_dir(root.join(dir)).unwrap().filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|x| x == "nx").unwrap_or(false)));
    }
    files.sort();
    assert!(files.len() > 40, "expected the whole tree, found {} files", files.len());
    for f in files {
        let oracle = std::process::Command::new(env!("CARGO_BIN_EXE_nx")).arg("sexp").arg(&f).output().unwrap();
        let mine = std::process::Command::new(&exe).arg(&f).output().unwrap();
        assert_eq!(String::from_utf8_lossy(&oracle.stdout), String::from_utf8_lossy(&mine.stdout), "parse tree differs for {}", f.display());
        assert_eq!(oracle.status.code(), mine.status.code(), "exit code differs for {}", f.display());
    }
}

/// The checker written in Nexium, stage 1: declarations and signatures match
/// `nx tir --sigs` for every source the Rust checker accepts, except those
/// that import C headers (`@cImport` is a later stage).
#[test]
fn self_hosted_checker_matches_signatures() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    // its own output directory: the bodies test compiles the same file at the same time
    let out_dir = root.join("nx-out").join("self_check_sigs");
    let exe = out_dir.join(if cfg!(windows) { "self_check.exe" } else { "self_check" });
    let build = std::process::Command::new(env!("CARGO_BIN_EXE_nx")).args(["build", "self/check.nx", "-o"]).arg(&exe).arg("--out-dir").arg(&out_dir).current_dir(root).output().expect("run nx");
    assert!(
        build.status.success(),
        "building self/check.nx failed:
{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    for dir in ["examples", "std", "gui", "self", "tests"] {
        files.extend(std::fs::read_dir(root.join(dir)).unwrap().filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|x| x == "nx").unwrap_or(false)));
    }
    files.sort();
    let mut compared = 0;
    for f in files {
        let oracle = std::process::Command::new(env!("CARGO_BIN_EXE_nx")).arg("tir").arg(&f).arg("--sigs").current_dir(root).output().unwrap();
        if !oracle.status.success() {
            // a deliberately failing example: nothing to compare
            continue;
        }
        let expected = String::from_utf8_lossy(&oracle.stdout).to_string();
        if expected.contains("(module ") && expected.contains(" cimport:") {
            // a C header somewhere in the import graph
            continue;
        }
        let mine = std::process::Command::new(&exe).arg(&f).arg("--sigs").current_dir(root).output().unwrap();
        assert!(
            mine.status.success(),
            "self/check.nx rejected {}:
{}",
            f.display(),
            String::from_utf8_lossy(&mine.stderr)
        );
        assert_eq!(expected, String::from_utf8_lossy(&mine.stdout), "signatures differ for {}", f.display());
        compared += 1;
    }
    assert!(compared > 35, "expected the whole tree, compared {} files", compared);
}

/// The checker written in Nexium, stage 2: the full typed IR matches `nx tir`
/// on every source it already covers (dyn, binary patterns,
/// comptime calls and C imports are the remaining stages). The list only
/// grows.
#[test]
fn self_hosted_checker_matches_bodies() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let out_dir = root.join("nx-out").join("self_check_bodies");
    let exe = out_dir.join(if cfg!(windows) { "self_check.exe" } else { "self_check" });
    let build = std::process::Command::new(env!("CARGO_BIN_EXE_nx")).args(["build", "self/check.nx", "-o"]).arg(&exe).arg("--out-dir").arg(&out_dir).current_dir(root).output().expect("run nx");
    assert!(
        build.status.success(),
        "building self/check.nx failed:
{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let files = [
        "examples/arena.nx",
        "examples/control.nx",
        "examples/ctest.nx",
        "examples/errors_more.nx",
        "examples/generics.nx",
        "examples/guard_scope.nx",
        "examples/hello.nx",
        "examples/json.nx",
        "examples/loops_more.nx",
        "examples/moves_again.nx",
        "examples/optional_move.nx",
        "examples/orelse_return.nx",
        "examples/own.nx",
        "examples/ownership.nx",
        "examples/parallel.nx",
        "examples/process.nx",
        "examples/records.nx",
        "examples/ropesim.nx",
        "examples/service.nx",
        "examples/stdlib.nx",
        "examples/tests.nx",
        "examples/tool.nx",
        "examples/tour.nx",
        "examples/tree.nx",
        "self/check.nx",
        "self/lexer.nx",
        "self/parser.nx",
        "std/args.nx",
        "std/bytes.nx",
        "std/fs.nx",
        "std/http.nx",
        "std/json.nx",
        "std/lists.nx",
        "std/net.nx",
        "std/num.nx",
        "std/process.nx",
        "std/regex.nx",
        "std/stream.nx",
        "std/strings.nx",
        "std/testing.nx",
        "std/text.nx",
        "std/thread.nx",
        "std/time.nx",
    ];
    for f in files {
        let path = root.join(f);
        let oracle = std::process::Command::new(env!("CARGO_BIN_EXE_nx")).arg("tir").arg(&path).current_dir(root).output().unwrap();
        assert!(
            oracle.status.success(),
            "nx tir rejected {}:
{}",
            f,
            String::from_utf8_lossy(&oracle.stderr)
        );
        let mine = std::process::Command::new(&exe).arg(&path).current_dir(root).output().unwrap();
        assert!(
            mine.status.success(),
            "self/check.nx rejected {}:
{}",
            f,
            String::from_utf8_lossy(&mine.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&oracle.stdout), String::from_utf8_lossy(&mine.stdout), "typed IR differs for {}", f);
    }
}

/// The GUI library's headless tests (rasterizer and widget interaction) must pass;
/// this also compiles gui/platform.c on every platform (the non-Windows stub included).
#[test]
fn gui_headless_tests_pass() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_nx")).args(["test", "gui/nexium_gui.nx"]).current_dir(root).output().expect("run nx");
    assert!(
        out.status.success(),
        "gui tests failed:
{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let shot = root.join("nx-out").join("demo-shot.ppm");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_nx")).args(["run", "gui/demo.nx", "--", "--shot"]).arg(&shot).current_dir(root).output().expect("run nx");
    assert!(
        out.status.success(),
        "offscreen demo failed:
{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(std::fs::metadata(&shot).map(|m| m.len() > 640 * 440 * 3).unwrap_or(false), "screenshot not written");
    // the same build from inside the directory, with a bare file name: the
    // declared C source must still be compiled in
    let out =
        std::process::Command::new(env!("CARGO_BIN_EXE_nx")).args(["build", "demo.nx", "--out-dir"]).arg(root.join("nx-out").join("demo-bare")).current_dir(root.join("gui")).output().expect("run nx");
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
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_nx")).arg("test").arg(&f).current_dir(root).output().unwrap();
        assert!(out.status.success(), "std tests failed for {}:\n{}{}", f.display(), String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        let fmt = std::process::Command::new(env!("CARGO_BIN_EXE_nx")).arg("fmt").arg(&f).arg("--check").current_dir(root).output().unwrap();
        assert!(fmt.status.success(), "{} is not canonically formatted", f.display());
    }
}

/// The REPL evaluates lines, keeps bindings, accepts items, reports errors, and prints.
#[test]
fn repl_session() {
    use std::io::Write;
    let script = "let x = 2\nx * 21\nfn sq(n: i32) -> i32 { return n * n }\nsq(x)\nprintln(\"hi {}\", .{x})\nimport std.strings\nstrings.to_upper(\"ok\")\nlet s = String.from(\"a\")\ns\nundefined_name\nvar xs = List(i32).new()\nxs.append(7)\nxs\n:quit\n";
    let mut child =
        std::process::Command::new(env!("CARGO_BIN_EXE_nx")).arg("repl").stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().unwrap();
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

#[test]
fn examples_are_canonically_formatted() {
    // `nx fmt --check` must pass on every example: formatting is idempotent and the sources are canonical
    let dir = root().join("examples");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|x| x == "nx").unwrap_or(false)).collect();
    files.sort();
    let mut cmd = Command::new(nx());
    cmd.arg("fmt");
    for f in &files {
        cmd.arg(f);
    }
    cmd.arg("--check");
    let out = cmd.current_dir(root()).output().expect("run nx fmt");
    assert!(
        out.status.success(),
        "unformatted examples:
{}",
        String::from_utf8_lossy(&out.stdout)
    );
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
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_nx")).args(["run", "main.nx"]).current_dir(base.join("app")).output().expect("run nx");
    assert!(out.status.success(), "package program failed:\n{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "hello, world!");
    let fetch = std::process::Command::new(env!("CARGO_BIN_EXE_nx")).arg("fetch").current_dir(base.join("app")).output().expect("run nx fetch");
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
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_nx")).args(["ship", "greeter.nx"]).env("ISCC", "").current_dir(&base).output().expect("run nx ship");
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
