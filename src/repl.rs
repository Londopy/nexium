//! `nx repl`: an interactive session on the compile-time interpreter.
//!
//! Every line is added to a synthetic program (items at the top level,
//! statements inside a synthetic `main`), the whole program is re-checked so
//! the prompt gets the compiler's real diagnostics, and only the new
//! statements are executed against the values kept from earlier lines. In
//! REPL mode the interpreter may perform I/O, which `comptime` never may.

use crate::lexer::{Lexer, Tok};
use crate::tir::Value;
use std::io::{self, BufRead, IsTerminal, Write};

const ITEM_KEYWORDS: &[&str] = &["fn", "pub", "struct", "enum", "record", "ref", "trait", "impl", "import", "type", "error", "test", "extern", "artifact", "comptime"];

struct Session {
    items: Vec<String>,
    stmts: Vec<String>,
    bindings: Vec<(String, Value)>,
    shown: Vec<(String, String)>,
}

impl Session {
    fn source(&self, extra_item: Option<&str>, extra_stmt: Option<&str>) -> String {
        let mut s = String::new();
        for it in &self.items {
            s.push_str(it);
            s.push('\n');
        }
        if let Some(it) = extra_item {
            s.push_str(it);
            s.push('\n');
        }
        s.push_str("fn main() -> !void {\n");
        for st in &self.stmts {
            s.push_str(st);
            s.push('\n');
        }
        if let Some(st) = extra_stmt {
            s.push_str(st);
            s.push('\n');
        }
        s.push_str("    return\n}\n");
        s
    }
}

/// A value that can be carried to the next line: no compile-time-only handles.
fn keepable(v: &Value) -> bool {
    match v {
        Value::Type(_) | Value::Fn(_) | Value::Ptr(..) => false,
        Value::Array(xs) | Value::Tuple(xs) | Value::Struct(xs) | Value::List(xs) | Value::Enum(_, xs) => xs.iter().all(keepable),
        Value::Opt(Some(b)) | Value::Ok(b) => keepable(b),
        Value::Map(kv) => kv.iter().all(|(k, v)| keepable(k) && keepable(v)),
        _ => true,
    }
}

/// Is the text a complete input? Unbalanced brackets or an unterminated
/// string mean the user is still typing.
fn complete(text: &str) -> bool {
    let (toks, diags) = Lexer::new(text, 0).lex();
    if diags.iter().any(|d| d.msg.contains("unterminated")) {
        return false;
    }
    let mut depth: i32 = 0;
    for t in &toks {
        match t.tok {
            Tok::LParen | Tok::LBracket | Tok::LBrace | Tok::DotLBrace => depth += 1,
            Tok::RParen | Tok::RBracket | Tok::RBrace => depth -= 1,
            _ => {}
        }
    }
    depth <= 0
}

fn is_item(text: &str) -> bool {
    let (toks, _) = Lexer::new(text, 0).lex();
    match toks.first().map(|t| &t.tok) {
        Some(Tok::Ident(s)) => ITEM_KEYWORDS.contains(&s.as_str()) || (s == "const" && toks.iter().any(|t| matches!(t.tok, Tok::At))),
        _ => false,
    }
}

fn help() {
    println!("Type Nexium code; each line is checked by the compiler and run.");
    println!("  let x = 2 * 21          bind a value (kept for later lines)");
    println!("  x + 1                   evaluate and print an expression");
    println!("  fn f(n: i32) -> i32 {{ return n * 2 }}");
    println!("  import std.json         then json.parse(\"[1, 2]\")");
    println!("Lines with open brackets continue on the next prompt.");
    println!("Commands:");
    println!("  :help          this text            :quit, :q      exit");
    println!("  :vars          the kept bindings    :items         the declared items");
    println!("  :load FILE     add a file's items    :reset         start over");
    println!("Not available at the prompt: @cImport, for parallel, using arena, artifacts.");
}

pub fn run() -> i32 {
    let stdin = io::stdin();
    let interactive = stdin.is_terminal();
    if interactive && (cfg!(not(windows)) || std::env::var_os("WT_SESSION").is_some()) {
        print!("\x1b]0;Nexium {}\x07", crate::VERSION);
    }
    let compiler = crate::compiler_summary();
    println!("Nexium {} (nx repl, {}) on {}", crate::VERSION, compiler, std::env::consts::OS);
    println!("Type :help for commands, :quit to exit.");
    let mut session = Session { items: vec![], stmts: vec![], bindings: vec![], shown: vec![] };
    let mut pending = String::new();
    loop {
        if interactive {
            print!("{}", if pending.is_empty() { "> " } else { "... " });
            let _ = io::stdout().flush();
        }
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => {
                if interactive {
                    println!();
                }
                return 0;
            }
            Ok(_) => {}
            Err(_) => return 1,
        }
        let line = line.trim_end_matches(['\n', '\r']);
        if pending.is_empty() {
            let t = line.trim();
            if t.is_empty() {
                continue;
            }
            if let Some(cmd) = t.strip_prefix(':') {
                let mut parts = cmd.splitn(2, ' ');
                match parts.next().unwrap_or("") {
                    "q" | "quit" | "exit" => return 0,
                    "help" | "h" | "?" => help(),
                    "reset" => {
                        session = Session { items: vec![], stmts: vec![], bindings: vec![], shown: vec![] };
                        println!("reset");
                    }
                    "vars" => {
                        if session.shown.is_empty() {
                            println!("(no bindings)");
                        }
                        for (n, v) in &session.shown {
                            println!("{}: {}", n, v);
                        }
                    }
                    "items" => {
                        if session.items.is_empty() {
                            println!("(no items)");
                        }
                        for it in &session.items {
                            println!("{}", it.lines().next().unwrap_or(""));
                        }
                    }
                    "load" => match std::fs::read_to_string(parts.next().unwrap_or("").trim()) {
                        Ok(text) => {
                            let src = session.source(Some(&text), None);
                            match crate::repl_check(&src, session.stmts.len(), session.bindings.clone()) {
                                Ok(_) => {
                                    session.items.push(text);
                                    println!("loaded");
                                }
                                Err(d) => eprint!("{}", d),
                            }
                        }
                        Err(e) => eprintln!("cannot read: {}", e),
                    },
                    other => println!("unknown command :{}; try :help", other),
                }
                continue;
            }
            if t == "exit" || t == "quit" {
                return 0;
            }
        }
        if !pending.is_empty() {
            pending.push('\n');
        }
        pending.push_str(line);
        if !complete(&pending) {
            continue;
        }
        let input = std::mem::take(&mut pending);
        if is_item(&input) {
            let src = session.source(Some(&input), None);
            match crate::repl_check(&src, session.stmts.len(), session.bindings.clone()) {
                Ok(_) => session.items.push(input),
                Err(d) => eprint!("{}", d),
            }
            continue;
        }
        let src = session.source(None, Some(&input));
        match crate::repl_check(&src, session.stmts.len(), session.bindings.clone()) {
            Ok(out) => {
                if let Some(msg) = out.panic {
                    eprintln!("panic: {}", msg);
                    continue;
                }
                session.stmts.push(input);
                let kept: Vec<(String, Value)> = out.bindings.into_iter().filter(|(_, v)| keepable(v)).collect();
                session.shown = out.shown.into_iter().filter(|(n, _)| kept.iter().any(|(k, _)| k == n)).collect();
                session.bindings = kept;
                if let Some((ty, val)) = out.printed {
                    println!("{}: {}", ty, val);
                }
            }
            Err(d) => eprint!("{}", d),
        }
    }
}
