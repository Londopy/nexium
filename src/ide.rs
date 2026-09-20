//! Editor features behind `nx lsp`: go to definition, completion, rename.
//!
//! These work from the token stream and the parsed modules, not the typed
//! program, so they answer while the code is half-written. Names resolve
//! the way the language does: locals in the enclosing function first (the
//! last `let`, `var`, parameter or capture before the cursor), then the
//! file's items, then imported modules and the std modules; `alias.name`
//! looks inside the imported module; `value.method` finds a method of that
//! name in any `impl`.

use crate::ast::{self, Item, Module};
use crate::diag::{SourceMap, Span};
use crate::lexer::{Lexer, Tok, Token};

/// Where a name is defined: a file of the source map and a span in it.
#[derive(Clone, Debug)]
pub struct DefLoc {
    pub file: u32,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Completion {
    pub label: String,
    /// an LSP CompletionItemKind
    pub kind: u32,
    pub detail: String,
}

pub const KIND_METHOD: u32 = 2;
pub const KIND_FUNCTION: u32 = 3;
pub const KIND_FIELD: u32 = 5;
pub const KIND_VARIABLE: u32 = 6;
pub const KIND_MODULE: u32 = 9;
pub const KIND_ENUM: u32 = 13;
pub const KIND_KEYWORD: u32 = 14;
pub const KIND_CONSTANT: u32 = 21;
pub const KIND_STRUCT: u32 = 22;
pub const KIND_TYPE: u32 = 25;

/// The builtin namespaces and what they offer (kept in step with the checker).
pub const NAMESPACES: &[(&str, &[&str])] = &[
    ("math", &["PI", "E", "TAU", "INF", "NAN", "sqrt", "abs", "floor", "ceil", "round", "sin", "cos", "tan", "exp", "log", "log2", "min", "max", "pow", "atan2", "clamp"]),
    (
        "io",
        &[
            "read_file",
            "write_file",
            "append_file",
            "read_line",
            "file_kind",
            "file_size",
            "file_modified",
            "make_dir",
            "remove_file",
            "remove_dir",
            "rename",
            "list_dir",
            "cwd",
            "temp_dir",
            "open",
            "read",
            "write",
            "flush",
            "close",
        ],
    ),
    ("os", &["args", "env", "environ", "exit"]),
    ("process", &["run", "exec", "last_stdout", "last_stderr"]),
    ("time", &["now", "monotonic", "sleep", "utc_offset"]),
    ("random", &["int", "float", "seed"]),
    ("net", &["connect", "listen", "accept", "send", "recv", "close", "peer", "local", "resolve", "udp_bind", "send_to", "recv_from", "last_peer"]),
    ("thread", &["start", "join", "count"]),
    ("sync", &["mutex_new", "lock", "unlock", "mutex_free", "cond_new", "wait", "signal", "broadcast", "cond_free"]),
    ("mem", &["copy"]),
];

const PRIMITIVES: &[&str] =
    &["i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32", "f64", "bool", "char", "void", "never", "type", "error", "String", "List", "Map"];

const COMMON_METHODS: &[&str] = &[
    "append",
    "append_char",
    "push_byte",
    "pop",
    "clear",
    "clone",
    "len",
    "is_empty",
    "last",
    "first",
    "insert",
    "remove",
    "swap_remove",
    "extend",
    "reserve",
    "items",
    "contains",
    "index_of",
    "sort",
    "reverse",
    "fill",
    "get",
    "put",
    "keys",
    "values",
    "starts_with",
    "ends_with",
    "find",
    "trim",
    "split",
    "lines",
    "to_string",
    "parse_int",
    "parse_float",
    "eq_ignore_case",
    "bytes",
    "to_owned",
    "copy_from",
    "abs",
    "min",
    "max",
    "to_lower",
    "to_upper",
    "is_digit",
    "is_alpha",
    "is_space",
];

fn lex(text: &str) -> Vec<Token> {
    Lexer::new(text, 0).lex().0
}

fn ident_at(toks: &[Token], off: usize) -> Option<(usize, String)> {
    toks.iter().enumerate().find_map(|(i, t)| match &t.tok {
        Tok::Ident(s) if (t.span.start as usize) <= off && off <= (t.span.end as usize) => Some((i, s.clone())),
        _ => None,
    })
}

/// `a.b` where `b` is the token at `i`: the qualifier `a`.
fn qualifier(toks: &[Token], i: usize) -> Option<String> {
    if i >= 2 && matches!(toks[i - 1].tok, Tok::Dot) {
        if let Tok::Ident(q) = &toks[i - 2].tok {
            return Some(q.clone());
        }
    }
    None
}

fn item_name_span(item: &Item) -> Option<(&str, Span, u32)> {
    match item {
        Item::Fn(f) => Some((&f.name, f.span, KIND_FUNCTION)),
        Item::Struct(s) => Some((&s.name, s.span, KIND_STRUCT)),
        Item::Enum(e) => Some((&e.name, e.span, KIND_ENUM)),
        Item::Trait(t) => Some((&t.name, t.span, KIND_TYPE)),
        Item::Const(c) => Some((&c.name, c.span, KIND_CONSTANT)),
        Item::Global(g) => Some((&g.name, g.span, KIND_VARIABLE)),
        Item::TypeAlias(t) => Some((&t.name, t.span, KIND_TYPE)),
        Item::ErrorSet(e) => Some((&e.name, e.span, KIND_ENUM)),
        _ => None,
    }
}

fn item_is_pub(item: &Item) -> bool {
    match item {
        Item::Fn(f) => f.attrs.is_pub,
        Item::Struct(s) => s.attrs.is_pub,
        Item::Enum(e) => e.attrs.is_pub,
        Item::Trait(t) => t.attrs.is_pub,
        Item::Const(c) => c.attrs.is_pub,
        Item::Global(g) => g.attrs.is_pub,
        Item::TypeAlias(t) => t.attrs.is_pub,
        Item::ErrorSet(e) => e.attrs.is_pub,
        _ => false,
    }
}

/// The function (or method) of the root module whose span holds `off`.
fn enclosing_fn(root: &Module, off: usize) -> Option<&ast::FnDecl> {
    let mut all: Vec<&ast::FnDecl> = Vec::new();
    for item in &root.items {
        match item {
            Item::Fn(f) => all.push(f),
            Item::Impl(im) => all.extend(im.methods.iter()),
            Item::Trait(t) => all.extend(t.methods.iter()),
            _ => {}
        }
    }
    all.into_iter().filter(|f| (f.span.start as usize) <= off && off <= (f.span.end as usize)).min_by_key(|f| f.span.end - f.span.start)
}

/// Local definitions visible at `off` inside `f`: parameters, and the
/// `let`/`var`/capture bindings whose token comes before the cursor.
fn locals_before(f: &ast::FnDecl, toks: &[Token], off: usize) -> Vec<(String, Span)> {
    let mut out: Vec<(String, Span)> = f.params.iter().map(|p| (p.name.clone(), p.span)).collect();
    let start = f.span.start as usize;
    for (i, t) in toks.iter().enumerate() {
        let s = t.span.start as usize;
        if s < start || s >= off {
            continue;
        }
        if let Tok::Ident(name) = &t.tok {
            if i == 0 {
                continue;
            }
            let prev = &toks[i - 1].tok;
            // `let x`, `var x`, `if let x =`, `for x, i in`, `for parallel x in`,
            // and closure or `catch` captures `|x|`
            let declared = matches!(prev, Tok::Ident(k) if k == "let" || k == "var" || k == "for" || k == "parallel")
                || matches!(prev, Tok::Pipe | Tok::Comma if in_capture(toks, i))
                || matches!(prev, Tok::Comma if in_for_header(toks, i));
            if declared {
                out.push((name.clone(), t.span));
            }
        }
    }
    out
}

/// Is the identifier at `i` inside a `|...|` capture list?
fn in_capture(toks: &[Token], i: usize) -> bool {
    // walk back to the nearest `|` on this logical line
    let mut j = i;
    while j > 0 {
        j -= 1;
        match &toks[j].tok {
            Tok::Pipe => return true,
            Tok::Newline | Tok::LBrace | Tok::RBrace => return false,
            _ => {}
        }
    }
    false
}

/// Is the identifier at `i` among the bindings of a `for` header, before `in`?
fn in_for_header(toks: &[Token], i: usize) -> bool {
    let mut j = i;
    while j > 0 {
        j -= 1;
        match &toks[j].tok {
            Tok::Ident(k) if k == "for" => return true,
            Tok::Ident(k) if k == "in" => return false,
            Tok::Ident(_) | Tok::Comma => {}
            _ => return false,
        }
    }
    false
}

fn module_index(names: &[String], target: &str) -> Option<usize> {
    names.iter().position(|n| n == target || n.ends_with(&format!(".{}", target)))
}

/// The module an import alias refers to.
fn alias_module(root: &Module, names: &[String], alias: &str) -> Option<usize> {
    for item in &root.items {
        if let Item::Import(im) = item {
            let a = im.alias.clone().unwrap_or_else(|| im.path.last().cloned().unwrap_or_default());
            if a == alias {
                return module_index(names, &im.path.join("."));
            }
        }
    }
    None
}

fn find_item(m: &Module, name: &str, pub_only: bool) -> Option<DefLoc> {
    for item in &m.items {
        if let Some((n, span, _)) = item_name_span(item) {
            if n == name && (!pub_only || item_is_pub(item)) {
                return Some(DefLoc { file: span.file, span });
            }
        }
    }
    None
}

fn find_method(modules: &[Module], name: &str) -> Option<DefLoc> {
    for m in modules {
        for item in &m.items {
            if let Item::Impl(im) = item {
                if let Some(f) = im.methods.iter().find(|f| f.name == name) {
                    return Some(DefLoc { file: f.span.file, span: f.span });
                }
            }
        }
    }
    None
}

/// Where the name under the cursor is defined.
pub fn definition(modules: &[Module], names: &[String], text: &str, off: usize) -> Option<DefLoc> {
    let toks = lex(text);
    let (i, name) = ident_at(&toks, off)?;
    let root = modules.first()?;
    if let Some(q) = qualifier(&toks, i) {
        if let Some(mi) = alias_module(root, names, &q) {
            return find_item(&modules[mi], &name, true);
        }
        if NAMESPACES.iter().any(|(n, _)| *n == q) {
            return None;
        }
        // `Type.assoc()` or `value.method()`
        if let Some(d) = find_method(modules, &name) {
            return Some(d);
        }
        return find_item(root, &name, false);
    }
    if let Some(f) = enclosing_fn(root, off) {
        let locals = locals_before(f, &toks, off);
        if let Some((_, span)) = locals.iter().rev().find(|(n, _)| *n == name) {
            return Some(DefLoc { file: 0, span: *span });
        }
    }
    if let Some(d) = find_item(root, &name, false) {
        return Some(d);
    }
    // `import a.{x}` and the std modules, by name
    for m in modules.iter().skip(1) {
        if let Some(d) = find_item(m, &name, true) {
            return Some(d);
        }
    }
    find_method(modules, &name)
}

/// Completions at `off` (the identifier being typed ends there).
pub fn completions(modules: &[Module], names: &[String], text: &str, off: usize) -> Vec<Completion> {
    let bytes = text.as_bytes();
    let mut start = off.min(bytes.len());
    while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        start -= 1;
    }
    let prefix = &text[start..off.min(bytes.len())];
    let mut before = start;
    while before > 0 && bytes[before - 1] == b' ' {
        before -= 1;
    }
    let after_dot = before > 0 && bytes[before - 1] == b'.';
    let mut out: Vec<Completion> = Vec::new();
    let mut push = |label: &str, kind: u32, detail: &str| {
        if label.starts_with(prefix) && !out.iter().any(|c| c.label == label) {
            out.push(Completion { label: label.to_string(), kind, detail: detail.to_string() });
        }
    };
    let root = match modules.first() {
        Some(r) => r,
        None => return out,
    };
    if after_dot {
        // the word before the dot
        let mut q_end = before - 1;
        while q_end > 0 && bytes[q_end - 1] == b' ' {
            q_end -= 1;
        }
        let mut q_start = q_end;
        while q_start > 0 && (bytes[q_start - 1].is_ascii_alphanumeric() || bytes[q_start - 1] == b'_') {
            q_start -= 1;
        }
        let q = &text[q_start..q_end];
        if let Some((_, members)) = NAMESPACES.iter().find(|(n, _)| *n == q) {
            for m in members.iter() {
                push(m, KIND_FUNCTION, &format!("{}.{}", q, m));
            }
            return out;
        }
        if let Some(mi) = alias_module(root, names, q) {
            for item in &modules[mi].items {
                if let Some((n, _, kind)) = item_name_span(item) {
                    if item_is_pub(item) {
                        push(n, kind, &names[mi]);
                    }
                }
            }
            return out;
        }
        if q == "error" {
            for m in modules {
                for item in &m.items {
                    if let Item::ErrorSet(e) = item {
                        for n in &e.names {
                            push(n, KIND_CONSTANT, &format!("error set {}", e.name));
                        }
                    }
                }
            }
            for n in ["OutOfMemory", "Panic", "NotFound", "IoError", "InvalidInput", "Timeout", "ConnectionRefused", "Truncated", "Overflow", "InvalidUtf8", "BufferTooSmall", "InvalidRecord"] {
                push(n, KIND_CONSTANT, "builtin error");
            }
            return out;
        }
        // fields and methods declared anywhere, then the builtin methods
        for m in modules {
            for item in &m.items {
                match item {
                    Item::Struct(s) => {
                        for f in &s.fields {
                            push(&f.name, KIND_FIELD, &format!("field of {}", s.name));
                        }
                    }
                    Item::Impl(im) => {
                        for f in &im.methods {
                            push(&f.name, KIND_METHOD, "method");
                        }
                    }
                    Item::Enum(e) => {
                        for v in &e.variants {
                            push(&v.name, KIND_ENUM, &format!("variant of {}", e.name));
                        }
                    }
                    _ => {}
                }
            }
        }
        for m in COMMON_METHODS {
            push(m, KIND_METHOD, "builtin method");
        }
        return out;
    }
    if let Some(f) = enclosing_fn(root, off) {
        let toks = lex(text);
        for (n, _) in locals_before(f, &toks, off) {
            push(&n, KIND_VARIABLE, "local");
        }
    }
    for item in &root.items {
        if let Some((n, _, kind)) = item_name_span(item) {
            push(n, kind, "this file");
        }
        if let Item::Import(im) = item {
            let a = im.alias.clone().unwrap_or_else(|| im.path.last().cloned().unwrap_or_default());
            push(&a, KIND_MODULE, &format!("import {}", im.path.join(".")));
            if let Some(ns) = &im.names {
                for n in ns {
                    push(n, KIND_FUNCTION, &format!("from {}", im.path.join(".")));
                }
            }
        }
    }
    for (n, _) in NAMESPACES {
        push(n, KIND_MODULE, "builtin namespace");
    }
    for p in PRIMITIVES {
        push(p, KIND_TYPE, "type");
    }
    for k in crate::lexer::KEYWORDS {
        push(k, KIND_KEYWORD, "keyword");
    }
    for b in ["println", "print", "eprintln", "format", "expect", "expect_eq", "panic", "assert"] {
        push(b, KIND_FUNCTION, "builtin");
    }
    out
}

/// The spans in `text` to rewrite when renaming the identifier at `off`:
/// every occurrence inside the enclosing function for a local, or in the
/// whole file otherwise.
pub fn rename_spans(modules: &[Module], text: &str, off: usize) -> Vec<Span> {
    let toks = lex(text);
    let (i, name) = match ident_at(&toks, off) {
        Some(x) => x,
        None => return vec![],
    };
    if crate::lexer::is_keyword(&name) {
        return vec![];
    }
    let root = match modules.first() {
        Some(r) => r,
        None => return vec![],
    };
    let mut range: Option<(usize, usize)> = None;
    if qualifier(&toks, i).is_none() {
        if let Some(f) = enclosing_fn(root, off) {
            if locals_before(f, &toks, off).iter().any(|(n, _)| *n == name) {
                range = Some((f.span.start as usize, f.span.end as usize));
            }
        }
    }
    toks.iter()
        .filter(|t| matches!(&t.tok, Tok::Ident(s) if *s == name))
        .filter(|t| range.map(|(a, b)| (t.span.start as usize) >= a && (t.span.end as usize) <= b).unwrap_or(true))
        .map(|t| t.span)
        .collect()
}

/// A file's 0-based line and column for a byte offset.
pub fn line_col(sm: &SourceMap, file: u32, offset: u32) -> (usize, usize) {
    let sp = Span { file, start: offset, end: offset };
    match sm.line_col(sp) {
        Some((l, c)) => (l - 1, c - 1),
        None => (0, 0),
    }
}
