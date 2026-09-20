//! The language server over stdio: diagnostics, hover, go to definition,
//! completion and rename, driven the way an editor drives them.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn frame(body: &str) -> Vec<u8> {
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body).into_bytes()
}

/// Send the requests in order, then read everything the server writes.
fn session(requests: &[String]) -> String {
    // the server under test is the one written in Nexium, built by the examples harness (nx-out/bootstrap/nx2)
    let nx2 = root().join("nx-out").join("bootstrap").join(if cfg!(windows) { "nx2.exe" } else { "nx2" });
    let server = if nx2.exists() { nx2 } else { PathBuf::from(env!("CARGO_BIN_EXE_nx")) };
    let mut child = Command::new(server).arg("lsp").env("NX_ZIG", "zig").current_dir(root()).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().expect("start nx lsp");
    {
        let stdin = child.stdin.as_mut().unwrap();
        for r in requests {
            stdin.write_all(&frame(r)).unwrap();
        }
        stdin.write_all(&frame(r#"{"jsonrpc":"2.0","id":99,"method":"shutdown"}"#)).unwrap();
        stdin.write_all(&frame(r#"{"jsonrpc":"2.0","method":"exit"}"#)).unwrap();
    }
    let mut out = String::new();
    child.stdout.take().unwrap().read_to_string(&mut out).unwrap();
    child.wait().unwrap();
    out
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

#[test]
fn definition_completion_and_rename() {
    let dir = root().join("nx-out").join("lsp-test");
    std::fs::create_dir_all(&dir).unwrap();
    let helper = dir.join("helper.nx");
    std::fs::write(&helper, "pub fn twice(x: i32) -> i32 { return x * 2 }\npub const LIMIT: i32 = 10\n").unwrap();
    let main = dir.join("main.nx");
    let text = "import helper\n\nstruct Point { x: i32, y: i32 }\n\nfn origin() -> Point { return Point{ .x = 0, .y = 0 } }\n\nfn main() {\n    let total = helper.twice(3)\n    let p = origin()\n    println(\"{} {}\", .{total + p.x, helper.LIMIT})\n    let again = total\n    for item, idx in [1, 2] { println(\"{}\", .{item + idx}) }\n}\n";
    std::fs::write(&main, text).unwrap();
    let uri = format!("file:///{}", main.to_string_lossy().replace('\\', "/").replace(':', "%3A"));
    let open = format!(r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"{}","languageId":"nexium","version":1,"text":"{}"}}}}}}"#, uri, json_escape(text));
    // `helper.twice` on line 7: the identifier `twice` starts at column 23
    let def_twice = format!(r#"{{"jsonrpc":"2.0","id":1,"method":"textDocument/definition","params":{{"textDocument":{{"uri":"{}"}},"position":{{"line":7,"character":24}}}}}}"#, uri);
    // `origin()` on line 8, column 12
    let def_origin = format!(r#"{{"jsonrpc":"2.0","id":2,"method":"textDocument/definition","params":{{"textDocument":{{"uri":"{}"}},"position":{{"line":8,"character":13}}}}}}"#, uri);
    // `total` on line 10 (`let again = total`), column 16: a local
    let def_local = format!(r#"{{"jsonrpc":"2.0","id":3,"method":"textDocument/definition","params":{{"textDocument":{{"uri":"{}"}},"position":{{"line":10,"character":17}}}}}}"#, uri);
    // completion right after `helper.` on line 9 (inside the println arguments), with nothing typed yet
    let comp_module = format!(r#"{{"jsonrpc":"2.0","id":4,"method":"textDocument/completion","params":{{"textDocument":{{"uri":"{}"}},"position":{{"line":9,"character":43}}}}}}"#, uri);
    // completion after `p.` on line 9, column 33 (start of `x`)
    let comp_field = format!(r#"{{"jsonrpc":"2.0","id":5,"method":"textDocument/completion","params":{{"textDocument":{{"uri":"{}"}},"position":{{"line":9,"character":33}}}}}}"#, uri);
    // rename the local `total` from its use on line 10
    let rename = format!(r#"{{"jsonrpc":"2.0","id":6,"method":"textDocument/rename","params":{{"textDocument":{{"uri":"{}"}},"position":{{"line":10,"character":17}},"newName":"sum"}}}}"#, uri);
    // `idx` used on line 11 (`item + idx`), column 52: a `for` binding
    let def_for = format!(r#"{{"jsonrpc":"2.0","id":7,"method":"textDocument/definition","params":{{"textDocument":{{"uri":"{}"}},"position":{{"line":11,"character":53}}}}}}"#, uri);
    let out = session(&[r#"{"jsonrpc":"2.0","id":0,"method":"initialize","params":{}}"#.into(), open, def_twice, def_origin, def_local, comp_module, comp_field, rename, def_for]);

    assert!(out.contains("\"definitionProvider\":true"), "capabilities: {}", out);
    // each response is one frame; find them by id
    let resp = |id: u32| -> String {
        let key = format!("\"id\":{},", id);
        let start = out.find(&key).or_else(|| out.find(&format!("\"id\":{}}}", id))).unwrap_or_else(|| panic!("no response {} in {}", id, out));
        let end = out[start..].find("Content-Length").map(|e| start + e).unwrap_or(out.len());
        out[start..end].to_string()
    };
    let r1 = resp(1);
    assert!(r1.contains("helper.nx") && r1.contains("\"line\":0"), "definition of helper.twice: {}", r1);
    let r2 = resp(2);
    assert!(r2.contains("main.nx") && r2.contains("\"line\":4"), "definition of origin: {}", r2);
    let r3 = resp(3);
    assert!(r3.contains("\"line\":7"), "definition of the local total: {}", r3);
    let r4 = resp(4);
    assert!(r4.contains("\"label\":\"LIMIT\"") && r4.contains("\"label\":\"twice\""), "module completion: {}", r4);
    let r5 = resp(5);
    assert!(r5.contains("\"label\":\"x\"") && r5.contains("\"label\":\"y\""), "field completion: {}", r5);
    let r6 = resp(6);
    assert_eq!(r6.matches("\"newText\":\"sum\"").count(), 3, "rename edits: {}", r6);
    // `idx` resolves to its binding in the `for` header on the same line, column 14
    let r7 = resp(7);
    assert!(r7.contains("\"line\":11") && r7.contains("\"character\":14"), "definition of for binding: {}", r7);
}
