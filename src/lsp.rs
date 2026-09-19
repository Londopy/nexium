//! `nx lsp`: a language server over stdio (JSON-RPC, LSP 3.x subset).
//!
//! Supported: initialize/shutdown, textDocument/didOpen, didChange, didSave,
//! didClose (publishing diagnostics on every change), and textDocument/hover
//! (function signatures with inferred effects, and local variable types are
//! not tracked). No dependencies: a small JSON reader/writer lives here.

use std::collections::HashMap;
use std::io::{self, BufRead, Write};

// ----- minimal JSON --------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(kv) => kv.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Num(n) => Some(*n),
            _ => None,
        }
    }
    pub fn as_arr(&self) -> Option<&Vec<Json>> {
        match self {
            Json::Arr(a) => Some(a),
            _ => None,
        }
    }
    pub fn write(&self, out: &mut String) {
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Json::Num(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    out.push_str(&format!("{}", *n as i64));
                } else {
                    out.push_str(&format!("{}", n));
                }
            }
            Json::Str(s) => write_str(s, out),
            Json::Arr(a) => {
                out.push('[');
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    v.write(out);
                }
                out.push(']');
            }
            Json::Obj(kv) => {
                out.push('{');
                for (i, (k, v)) in kv.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_str(k, out);
                    out.push(':');
                    v.write(out);
                }
                out.push('}');
            }
        }
    }
    pub fn to_string(&self) -> String {
        let mut s = String::new();
        self.write(&mut s);
        s
    }
}

fn write_str(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

struct JsonParser<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> JsonParser<'a> {
    fn ws(&mut self) {
        while self.i < self.b.len() && self.b[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn value(&mut self) -> Option<Json> {
        self.ws();
        let c = *self.b.get(self.i)?;
        match c {
            b'{' => {
                self.i += 1;
                let mut kv = Vec::new();
                loop {
                    self.ws();
                    if self.b.get(self.i) == Some(&b'}') {
                        self.i += 1;
                        break;
                    }
                    let k = match self.value()? {
                        Json::Str(s) => s,
                        _ => return None,
                    };
                    self.ws();
                    if self.b.get(self.i) != Some(&b':') {
                        return None;
                    }
                    self.i += 1;
                    let v = self.value()?;
                    kv.push((k, v));
                    self.ws();
                    match self.b.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b'}') => {
                            self.i += 1;
                            break;
                        }
                        _ => return None,
                    }
                }
                Some(Json::Obj(kv))
            }
            b'[' => {
                self.i += 1;
                let mut a = Vec::new();
                loop {
                    self.ws();
                    if self.b.get(self.i) == Some(&b']') {
                        self.i += 1;
                        break;
                    }
                    a.push(self.value()?);
                    self.ws();
                    match self.b.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b']') => {
                            self.i += 1;
                            break;
                        }
                        _ => return None,
                    }
                }
                Some(Json::Arr(a))
            }
            b'"' => {
                self.i += 1;
                let mut s = String::new();
                let mut buf = Vec::new();
                while self.i < self.b.len() && self.b[self.i] != b'"' {
                    if self.b[self.i] == b'\\' {
                        self.i += 1;
                        match self.b.get(self.i)? {
                            b'n' => buf.push(b'\n'),
                            b't' => buf.push(b'\t'),
                            b'r' => buf.push(b'\r'),
                            b'b' => buf.push(8),
                            b'f' => buf.push(12),
                            b'u' => {
                                let hex = std::str::from_utf8(self.b.get(self.i + 1..self.i + 5)?).ok()?;
                                let cp = u32::from_str_radix(hex, 16).ok()?;
                                self.i += 4;
                                let mut cp = cp;
                                // surrogate pairs
                                if (0xD800..0xDC00).contains(&cp) && self.b.get(self.i + 1..self.i + 3) == Some(b"\\u") {
                                    let hex2 = std::str::from_utf8(self.b.get(self.i + 3..self.i + 7)?).ok()?;
                                    let lo = u32::from_str_radix(hex2, 16).ok()?;
                                    cp = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                                    self.i += 6;
                                }
                                let ch = char::from_u32(cp).unwrap_or('\u{fffd}');
                                let mut tmp = [0u8; 4];
                                buf.extend_from_slice(ch.encode_utf8(&mut tmp).as_bytes());
                            }
                            other => buf.push(*other),
                        }
                        self.i += 1;
                    } else {
                        buf.push(self.b[self.i]);
                        self.i += 1;
                    }
                }
                self.i += 1;
                s.push_str(&String::from_utf8_lossy(&buf));
                Some(Json::Str(s))
            }
            b't' => {
                self.i += 4;
                Some(Json::Bool(true))
            }
            b'f' => {
                self.i += 5;
                Some(Json::Bool(false))
            }
            b'n' => {
                self.i += 4;
                Some(Json::Null)
            }
            _ => {
                let s = self.i;
                while self.i < self.b.len() && (self.b[self.i].is_ascii_digit() || matches!(self.b[self.i], b'-' | b'+' | b'.' | b'e' | b'E')) {
                    self.i += 1;
                }
                std::str::from_utf8(&self.b[s..self.i]).ok()?.parse::<f64>().ok().map(Json::Num)
            }
        }
    }
}

pub fn parse_json(s: &str) -> Option<Json> {
    JsonParser { b: s.as_bytes(), i: 0 }.value()
}

fn obj(kv: Vec<(&str, Json)>) -> Json {
    Json::Obj(kv.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}

// ----- server ------------------------------------------------------------------

pub struct Diagnostic {
    pub line: usize,
    pub col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub severity: u32,
    pub message: String,
}

pub struct HoverInfo {
    pub text: String,
}

/// Callbacks into the compiler, so this module stays free of compiler types.
pub struct Backend {
    pub diagnostics: Box<dyn Fn(&str, &str) -> Vec<Diagnostic>>,
    pub hover: Box<dyn Fn(&str, &str, usize, usize) -> Option<HoverInfo>>,
}

fn read_message(stdin: &mut impl BufRead) -> Option<String> {
    let mut len = 0usize;
    loop {
        let mut line = String::new();
        if stdin.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let t = line.trim_end();
        if t.is_empty() {
            break;
        }
        if let Some(v) = t.strip_prefix("Content-Length:") {
            len = v.trim().parse().ok()?;
        }
    }
    let mut buf = vec![0u8; len];
    stdin.read_exact(&mut buf).ok()?;
    Some(String::from_utf8_lossy(&buf).to_string())
}

fn send(out: &mut impl Write, msg: &Json) {
    let body = msg.to_string();
    let _ = write!(out, "Content-Length: {}\r\n\r\n{}", body.len(), body);
    let _ = out.flush();
}

fn uri_to_path(uri: &str) -> String {
    let p = uri.strip_prefix("file:///").or_else(|| uri.strip_prefix("file://")).unwrap_or(uri);
    let mut s = String::new();
    let b = p.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&p[i + 1..i + 3], 16) {
                s.push(v as char);
                i += 3;
                continue;
            }
        }
        s.push(b[i] as char);
        i += 1;
    }
    if cfg!(windows) {
        s = s.replace('/', "\\");
    } else if !s.starts_with('/') {
        s.insert(0, '/');
    }
    s
}

pub fn run(backend: Backend) {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut docs: HashMap<String, String> = HashMap::new();
    loop {
        let msg = match read_message(&mut reader) {
            Some(m) => m,
            None => break,
        };
        let json = match parse_json(&msg) {
            Some(j) => j,
            None => continue,
        };
        let method = json.get("method").and_then(|m| m.as_str()).unwrap_or("").to_string();
        let id = json.get("id").cloned();
        let params = json.get("params").cloned().unwrap_or(Json::Null);
        match method.as_str() {
            "initialize" => {
                let result = obj(vec![
                    (
                        "capabilities",
                        obj(vec![("textDocumentSync", obj(vec![("openClose", Json::Bool(true)), ("change", Json::Num(1.0)), ("save", Json::Bool(true))])), ("hoverProvider", Json::Bool(true))]),
                    ),
                    ("serverInfo", obj(vec![("name", Json::Str("nx".into())), ("version", Json::Str(env!("CARGO_PKG_VERSION").into()))])),
                ]);
                send(&mut out, &obj(vec![("jsonrpc", Json::Str("2.0".into())), ("id", id.unwrap_or(Json::Null)), ("result", result)]));
            }
            "initialized" => {}
            "shutdown" => {
                send(&mut out, &obj(vec![("jsonrpc", Json::Str("2.0".into())), ("id", id.unwrap_or(Json::Null)), ("result", Json::Null)]));
            }
            "exit" => break,
            "textDocument/didOpen" | "textDocument/didChange" | "textDocument/didSave" => {
                let td = params.get("textDocument").cloned().unwrap_or(Json::Null);
                let uri = td.get("uri").and_then(|u| u.as_str()).unwrap_or("").to_string();
                let text = if method == "textDocument/didOpen" {
                    td.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())
                } else if method == "textDocument/didChange" {
                    params.get("contentChanges").and_then(|c| c.as_arr()).and_then(|a| a.last()).and_then(|c| c.get("text")).and_then(|t| t.as_str()).map(|s| s.to_string())
                } else {
                    params.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())
                };
                if let Some(t) = text {
                    docs.insert(uri.clone(), t);
                }
                if let Some(text) = docs.get(&uri).cloned() {
                    let path = uri_to_path(&uri);
                    let diags = (backend.diagnostics)(&path, &text);
                    let items: Vec<Json> = diags
                        .iter()
                        .map(|d| {
                            obj(vec![
                                (
                                    "range",
                                    obj(vec![
                                        ("start", obj(vec![("line", Json::Num(d.line as f64)), ("character", Json::Num(d.col as f64))])),
                                        ("end", obj(vec![("line", Json::Num(d.end_line as f64)), ("character", Json::Num(d.end_col as f64))])),
                                    ]),
                                ),
                                ("severity", Json::Num(d.severity as f64)),
                                ("source", Json::Str("nx".into())),
                                ("message", Json::Str(d.message.clone())),
                            ])
                        })
                        .collect();
                    send(
                        &mut out,
                        &obj(vec![
                            ("jsonrpc", Json::Str("2.0".into())),
                            ("method", Json::Str("textDocument/publishDiagnostics".into())),
                            ("params", obj(vec![("uri", Json::Str(uri.clone())), ("diagnostics", Json::Arr(items))])),
                        ]),
                    );
                }
            }
            "textDocument/didClose" => {
                let uri = params.get("textDocument").and_then(|t| t.get("uri")).and_then(|u| u.as_str()).unwrap_or("").to_string();
                docs.remove(&uri);
                send(
                    &mut out,
                    &obj(vec![
                        ("jsonrpc", Json::Str("2.0".into())),
                        ("method", Json::Str("textDocument/publishDiagnostics".into())),
                        ("params", obj(vec![("uri", Json::Str(uri)), ("diagnostics", Json::Arr(vec![]))])),
                    ]),
                );
            }
            "textDocument/hover" => {
                let uri = params.get("textDocument").and_then(|t| t.get("uri")).and_then(|u| u.as_str()).unwrap_or("").to_string();
                let line = params.get("position").and_then(|p| p.get("line")).and_then(|l| l.as_f64()).unwrap_or(0.0) as usize;
                let col = params.get("position").and_then(|p| p.get("character")).and_then(|l| l.as_f64()).unwrap_or(0.0) as usize;
                let result = match docs.get(&uri) {
                    Some(text) => match (backend.hover)(&uri_to_path(&uri), text, line, col) {
                        Some(h) => obj(vec![("contents", obj(vec![("kind", Json::Str("markdown".into())), ("value", Json::Str(h.text))]))]),
                        None => Json::Null,
                    },
                    None => Json::Null,
                };
                send(&mut out, &obj(vec![("jsonrpc", Json::Str("2.0".into())), ("id", id.unwrap_or(Json::Null)), ("result", result)]));
            }
            _ => {
                if let Some(id) = id {
                    // unknown request: method not found
                    send(
                        &mut out,
                        &obj(vec![
                            ("jsonrpc", Json::Str("2.0".into())),
                            ("id", id),
                            ("error", obj(vec![("code", Json::Num(-32601.0)), ("message", Json::Str(format!("method not supported: {}", method)))])),
                        ]),
                    );
                }
            }
        }
    }
}
