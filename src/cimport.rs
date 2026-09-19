//! `@cImport("header.h")`: direct C header import (spec section 9, archived 14).
//!
//! The header is preprocessed by the C compiler (`zig cc -E`), so include
//! guards, conditionals, and object-like macros are handled by the real
//! preprocessor. The declarations that come out are read by a small C
//! declaration parser covering the subset a binding needs: functions,
//! typedefs of scalars and pointers, structs of scalars/pointers/arrays, enums,
//! and integer/float/string macros. Anything else is imported as an opaque
//! declaration that is an error to use, and the diagnostic names it.

use crate::ast::*;
use crate::diag::Span;
use std::collections::HashMap;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct CFunction {
    pub name: String,
    pub params: Vec<(String, TypeExpr)>,
    pub ret: TypeExpr,
    pub variadic: bool,
}

/// A constant read from an enum or an object-like macro.
#[derive(Clone, Debug, PartialEq)]
pub enum CConst {
    Int(i128),
    Float(f64),
    Str(Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct CImport {
    pub header: String,
    pub is_system: bool,
    pub functions: Vec<CFunction>,
    /// (Nexium name, C spelling such as `struct tag` or a typedef name, fields)
    pub structs: Vec<(String, String, Vec<(String, TypeExpr)>)>,
    pub consts: Vec<(String, CConst)>,
    pub aliases: Vec<(String, TypeExpr)>,
    /// declarations that could not be translated, with the reason
    pub unsupported: HashMap<String, String>,
}

pub struct ImportOptions {
    pub cc: Vec<String>,
    pub target: Option<String>,
    pub include_dirs: Vec<String>,
    pub windows: bool,
}

fn run_pp(opts: &ImportOptions, source: &str, extra: &[&str]) -> Result<String, String> {
    let dir = std::env::temp_dir().join(format!("nx-cimport-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("probe.c");
    std::fs::write(&path, source).map_err(|e| e.to_string())?;
    let mut cmd = Command::new(&opts.cc[0]);
    cmd.args(&opts.cc[1..]);
    cmd.args(["-E", "-w"]);
    cmd.args(extra);
    if let Some(t) = &opts.target {
        cmd.arg("-target");
        cmd.arg(t);
    }
    for d in &opts.include_dirs {
        cmd.arg(format!("-I{}", d));
    }
    cmd.arg(&path);
    let out = cmd.output().map_err(|e| format!("cannot run the C compiler: {}", e))?;
    let _ = std::fs::remove_file(&path);
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let first = err.lines().find(|l| l.contains("error")).unwrap_or("").trim().to_string();
        return Err(format!("the C preprocessor failed: {}", first));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

pub fn import(header: &str, opts: &ImportOptions, span: Span) -> Result<CImport, String> {
    let is_system = !opts.include_dirs.iter().any(|d| std::path::Path::new(d).join(header).exists());
    let include = if is_system { format!("#include <{}>\n", header) } else { format!("#include \"{}\"\n", header) };
    let text = run_pp(opts, &include, &["-P"])?;
    let macros = run_pp(opts, &include, &["-dM"])?;
    let mut ci = CImport { header: header.to_string(), is_system, functions: Vec::new(), structs: Vec::new(), consts: Vec::new(), aliases: Vec::new(), unsupported: HashMap::new() };
    let mut p = CParser { toks: tokenize(&text), pos: 0, typedefs: HashMap::new(), windows: opts.windows, span };
    p.parse_all(&mut ci);
    parse_macros(&macros, &mut ci);
    Ok(ci)
}

// ----- tokens ------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum CTok {
    Ident(String),
    Num(String),
    Str(Vec<u8>),
    Punct(String),
}

fn tokenize(src: &str) -> Vec<CTok> {
    let b = src.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i < b.len() {
        let c = b[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if c == b'#' {
            // line markers / directives that survived: skip the line
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            let s = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            out.push(CTok::Ident(src[s..i].to_string()));
            continue;
        }
        if c.is_ascii_digit() || (c == b'.' && i + 1 < b.len() && b[i + 1].is_ascii_digit()) {
            let s = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'.' || ((b[i] == b'+' || b[i] == b'-') && matches!(b[i - 1], b'e' | b'E' | b'p' | b'P'))) {
                i += 1;
            }
            out.push(CTok::Num(src[s..i].to_string()));
            continue;
        }
        if c == b'"' || (c == b'L' && i + 1 < b.len() && b[i + 1] == b'"') {
            if c == b'L' {
                i += 1;
            }
            i += 1;
            let mut s = Vec::new();
            while i < b.len() && b[i] != b'"' {
                if b[i] == b'\\' && i + 1 < b.len() {
                    i += 1;
                    s.push(match b[i] {
                        b'n' => b'\n',
                        b't' => b'\t',
                        b'r' => b'\r',
                        b'0' => 0,
                        other => other,
                    });
                } else {
                    s.push(b[i]);
                }
                i += 1;
            }
            i += 1;
            out.push(CTok::Str(s));
            continue;
        }
        if c == b'\'' {
            i += 1;
            let s = i;
            while i < b.len() && b[i] != b'\'' {
                if b[i] == b'\\' {
                    i += 1;
                }
                i += 1;
            }
            let lit = &src[s..i.min(b.len())];
            i += 1;
            let v = if lit.starts_with('\\') { lit.as_bytes().get(1).copied().unwrap_or(0) as u32 } else { lit.chars().next().map(|c| c as u32).unwrap_or(0) };
            out.push(CTok::Num(v.to_string()));
            continue;
        }
        // punctuation (longest first)
        let three = src.get(i..i + 3).unwrap_or("");
        let two = src.get(i..i + 2).unwrap_or("");
        if three == "..." || three == "<<=" || three == ">>=" {
            out.push(CTok::Punct(three.into()));
            i += 3;
            continue;
        }
        if matches!(two, "->" | "++" | "--" | "<<" | ">>" | "<=" | ">=" | "==" | "!=" | "&&" | "||" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "##") {
            out.push(CTok::Punct(two.into()));
            i += 2;
            continue;
        }
        out.push(CTok::Punct((c as char).to_string()));
        i += 1;
    }
    out
}

// ----- declarations ---------------------------------------------------------

struct CParser {
    toks: Vec<CTok>,
    pos: usize,
    typedefs: HashMap<String, Result<TypeExpr, String>>,
    windows: bool,
    span: Span,
}

const SKIP_WORDS: &[&str] = &[
    "extern",
    "static",
    "inline",
    "__inline",
    "__inline__",
    "__forceinline",
    "__restrict",
    "__restrict__",
    "restrict",
    "volatile",
    "__volatile__",
    "_Noreturn",
    "__extension__",
    "__cdecl",
    "__stdcall",
    "__fastcall",
    "__thiscall",
    "__vectorcall",
    "__attribute__",
    "__declspec",
    "__asm__",
    "__asm",
    "asm",
    "_Nullable",
    "_Nonnull",
    "__unaligned",
    "__w64",
    "__ptr32",
    "__ptr64",
    "register",
    "auto",
    "__thread",
    "_Thread_local",
];

impl CParser {
    fn peek(&self) -> Option<&CTok> {
        self.toks.get(self.pos)
    }
    fn is_punct(&self, p: &str) -> bool {
        matches!(self.peek(), Some(CTok::Punct(x)) if x == p)
    }

    /// Collect one top-level declaration's tokens (up to `;` at depth 0, or a
    /// function body's closing brace), with attributes and noise removed.
    fn next_decl(&mut self) -> Option<(Vec<CTok>, Option<Vec<CTok>>)> {
        let mut toks = Vec::new();
        let mut body: Option<Vec<CTok>> = None;
        let mut depth = 0i32;
        let mut paren = 0i32;
        let mut saw_body_brace = false;
        while let Some(t) = self.toks.get(self.pos).cloned() {
            self.pos += 1;
            match &t {
                CTok::Punct(p) if p == "{" => {
                    depth += 1;
                    if depth == 1 {
                        let is_type = toks.iter().any(|x| matches!(x, CTok::Ident(s) if s == "struct" || s == "union" || s == "enum"));
                        if !is_type {
                            saw_body_brace = true;
                        }
                        if is_type {
                            body = Some(Vec::new());
                            continue;
                        }
                    }
                }
                CTok::Punct(p) if p == "}" => {
                    depth -= 1;
                    if depth == 0 {
                        if saw_body_brace {
                            return Some((toks, None)); // function definition: body dropped
                        }
                        continue;
                    }
                }
                CTok::Punct(p) if p == ";" && depth == 0 && paren == 0 => return Some((toks, body)),
                CTok::Punct(p) if p == "(" => paren += 1,
                CTok::Punct(p) if p == ")" => paren -= 1,
                _ => {}
            }
            if depth >= 1 {
                if let Some(b) = &mut body {
                    if !(depth == 1 && matches!(&t, CTok::Punct(p) if p == "{")) {
                        b.push(t.clone());
                    }
                }
                if saw_body_brace {
                    continue;
                }
            }
            if depth == 0 || body.is_none() {
                toks.push(t);
            }
        }
        if toks.is_empty() {
            None
        } else {
            Some((toks, body))
        }
    }

    fn parse_all(&mut self, ci: &mut CImport) {
        let mut seen = std::collections::HashSet::new();
        while let Some((raw, body)) = self.next_decl() {
            let toks = strip_noise(&raw);
            if toks.is_empty() {
                continue;
            }
            let _ = self.decl(&toks, body.as_deref(), ci, &mut seen);
        }
    }

    fn decl(&mut self, toks: &[CTok], body: Option<&[CTok]>, ci: &mut CImport, seen: &mut std::collections::HashSet<String>) -> Option<()> {
        let first = match &toks[0] {
            CTok::Ident(s) => s.as_str(),
            _ => return None,
        };
        if first == "typedef" {
            let rest = &toks[1..];
            // typedef struct tag { ... } Name;   /  typedef struct tag Name;  / typedef int Name;
            if matches!(rest.first(), Some(CTok::Ident(s)) if s == "struct" || s == "union" || s == "enum") {
                let kind = if let CTok::Ident(s) = &rest[0] { s.clone() } else { unreachable!() };
                let name = match rest.last() {
                    Some(CTok::Ident(n)) => n.clone(),
                    _ => return None,
                };
                if name == "struct" || name == "union" || name == "enum" {
                    return None;
                }
                let tag = match rest.get(1) {
                    Some(CTok::Ident(t)) if t != &name => Some(t.clone()),
                    _ => None,
                };
                if kind == "enum" {
                    if let Some(b) = body {
                        self.enum_body(b, ci);
                    }
                    let i32t = TypeExpr::named("i32", self.span);
                    self.typedefs.insert(name.clone(), Ok(i32t.clone()));
                    ci.aliases.push((name, i32t));
                    return Some(());
                }
                if kind == "union" {
                    self.typedefs.insert(name.clone(), Err("unions are not supported".into()));
                    ci.unsupported.insert(name, "C unions are not supported".into());
                    return Some(());
                }
                let fields = match body {
                    Some(b) => self.struct_fields(b),
                    None => match tag.as_ref().and_then(|t| self.typedefs.get(&format!("struct {}", t)).cloned()) {
                        Some(Ok(TypeExpr::Named { path, .. })) => {
                            // alias of an already imported struct
                            let target = path[0].clone();
                            let te = TypeExpr::named(&target, self.span);
                            self.typedefs.insert(name.clone(), Ok(te.clone()));
                            ci.aliases.push((name, te));
                            return Some(());
                        }
                        _ => Ok(Vec::new()), // opaque struct: usable through pointers
                    },
                };
                // fields we cannot translate make the struct opaque (usable through
                // pointers, like a forward declaration), not unusable: Apple's FILE
                // holds function pointers and every stdio function takes FILE*
                let f = fields.unwrap_or_default();
                if seen.insert(name.clone()) {
                    ci.structs.push((name.clone(), name.clone(), f));
                }
                let te = TypeExpr::named(&name, self.span);
                self.typedefs.insert(name.clone(), Ok(te.clone()));
                if let Some(t) = tag {
                    self.typedefs.insert(format!("struct {}", t), Ok(te));
                }
                return Some(());
            }
            // function pointer typedef: typedef ret (*Name)(...)
            if rest.iter().any(|t| matches!(t, CTok::Punct(p) if p == "(")) {
                if let Some(pos) = rest.iter().position(|t| matches!(t, CTok::Punct(p) if p == "(")) {
                    if let Some(CTok::Ident(n)) = rest.get(pos + 2) {
                        self.typedefs.insert(n.clone(), Err("function pointer types are not supported".into()));
                        ci.unsupported.insert(n.clone(), "function pointer typedefs are not supported".into());
                    }
                }
                return Some(());
            }
            let name = match rest.last() {
                Some(CTok::Ident(n)) => n.clone(),
                _ => return None,
            };
            let ty = self.ctype(&rest[..rest.len() - 1]);
            match ty {
                Ok(t) => {
                    self.typedefs.insert(name.clone(), Ok(t.clone()));
                    ci.aliases.push((name, t));
                }
                Err(why) => {
                    self.typedefs.insert(name.clone(), Err(why.clone()));
                    ci.unsupported.insert(name, why);
                }
            }
            return Some(());
        }
        if first == "struct" || first == "union" {
            if let (Some(CTok::Ident(tag)), Some(b)) = (toks.get(1), body) {
                if first == "union" {
                    self.typedefs.insert(format!("struct {}", tag), Err("unions are not supported".into()));
                    return Some(());
                }
                let tag = tag.clone();
                // untranslatable fields: opaque, see the typedef case above
                let f = self.struct_fields(b).unwrap_or_default();
                if seen.insert(tag.clone()) {
                    ci.structs.push((tag.clone(), format!("struct {}", tag), f));
                }
                self.typedefs.insert(format!("struct {}", tag), Ok(TypeExpr::named(&tag, self.span)));
            }
            return Some(());
        }
        if first == "enum" {
            if let Some(b) = body {
                self.enum_body(b, ci);
            }
            return Some(());
        }
        // function: ... name ( params )
        let lp = toks.iter().position(|t| matches!(t, CTok::Punct(p) if p == "("))?;
        if lp == 0 {
            return None;
        }
        let name = match &toks[lp - 1] {
            CTok::Ident(n) => n.clone(),
            _ => return None,
        };
        if !matches!(toks.last(), Some(CTok::Punct(p)) if p == ")") {
            return None;
        }
        if !seen.insert(name.clone()) {
            return None;
        }
        let ret = match self.ctype(&toks[..lp - 1]) {
            Ok(t) => t,
            Err(why) => {
                ci.unsupported.insert(name, format!("return type: {}", why));
                return Some(());
            }
        };
        let params_toks = &toks[lp + 1..toks.len() - 1];
        let mut params = Vec::new();
        let mut variadic = false;
        for (k, p) in split_commas(params_toks).iter().enumerate() {
            if p.is_empty() {
                continue;
            }
            if matches!(p.as_slice(), [CTok::Punct(x)] if x == "...") {
                variadic = true;
                continue;
            }
            if matches!(p.as_slice(), [CTok::Ident(x)] if x == "void") {
                continue;
            }
            // trailing identifier is the parameter name unless it is a type word
            let (ty_toks, pname) = match p.last() {
                Some(CTok::Ident(n)) if !is_type_word(n) && !self.typedefs.contains_key(n) && p.len() > 1 => (&p[..p.len() - 1], n.clone()),
                _ => (&p[..], format!("arg{}", k)),
            };
            // array parameter `int a[]` is a pointer
            let mut ty_toks: Vec<CTok> = ty_toks.to_vec();
            if let Some(bpos) = ty_toks.iter().position(|t| matches!(t, CTok::Punct(x) if x == "[")) {
                ty_toks.truncate(bpos);
                ty_toks.push(CTok::Punct("*".into()));
            }
            match self.ctype(&ty_toks) {
                Ok(t) => params.push((pname, t)),
                Err(why) => {
                    ci.unsupported.insert(name.clone(), format!("parameter `{}`: {}", pname, why));
                    return Some(());
                }
            }
        }
        ci.functions.push(CFunction { name, params, ret, variadic });
        Some(())
    }

    fn enum_body(&mut self, b: &[CTok], ci: &mut CImport) {
        let mut next: i128 = 0;
        for item in split_commas(b) {
            if item.is_empty() {
                continue;
            }
            let name = match &item[0] {
                CTok::Ident(n) => n.clone(),
                _ => continue,
            };
            if item.len() > 2 {
                if let Some(v) = eval_const(&item[2..], ci) {
                    next = v;
                }
            }
            ci.consts.push((name, CConst::Int(next)));
            next += 1;
        }
    }

    fn struct_fields(&mut self, b: &[CTok]) -> Result<Vec<(String, TypeExpr)>, String> {
        let mut fields = Vec::new();
        for decl in b.split(|t| matches!(t, CTok::Punct(p) if p == ";")) {
            let decl = strip_noise(decl);
            if decl.is_empty() {
                continue;
            }
            if decl.iter().any(|t| matches!(t, CTok::Punct(p) if p == ":" || p == "(" || p == "{")) {
                return Err("bit-fields, nested definitions, and function pointer fields are not supported".into());
            }
            // int a, b;  char name[16];
            let mut base_end = decl.len();
            // find where declarators start: after the last type word/pointer
            let mut names: Vec<(String, usize, Vec<CTok>)> = Vec::new();
            for part in split_commas(&decl) {
                let _ = part;
            }
            // simple approach: base type = tokens up to the first identifier that is followed by `,` `[` or end
            let mut i = 0;
            while i < decl.len() {
                if let CTok::Ident(n) = &decl[i] {
                    let next = decl.get(i + 1);
                    let is_name = !is_type_word(n)
                        && !self.typedefs.contains_key(n)
                        && !matches!(n.as_str(), "struct" | "union" | "enum" | "const" | "unsigned" | "signed")
                        && (next.is_none() || matches!(next, Some(CTok::Punct(p)) if p == "," || p == "["));
                    if is_name {
                        base_end = base_end.min(i);
                        // array suffix
                        let mut dims = Vec::new();
                        let mut j = i + 1;
                        while matches!(decl.get(j), Some(CTok::Punct(p)) if p == "[") {
                            let mut k = j + 1;
                            let mut inner = Vec::new();
                            while k < decl.len() && !matches!(&decl[k], CTok::Punct(p) if p == "]") {
                                inner.push(decl[k].clone());
                                k += 1;
                            }
                            dims.push(inner);
                            j = k + 1;
                        }
                        names.push((n.clone(), i, dims.into_iter().flatten().collect()));
                        i = j;
                        continue;
                    }
                }
                i += 1;
            }
            if names.is_empty() {
                return Err("could not read a field declaration".into());
            }
            let base = self.ctype(&decl[..base_end])?;
            for (n, _, dims) in names {
                let mut t = base.clone();
                if !dims.is_empty() {
                    let len = eval_const_simple(&dims).ok_or_else(|| format!("array field `{}` has a non-constant length", n))?;
                    t = TypeExpr::Array { len: Box::new(Expr::Lit { value: Lit::Int(len as u128), span: self.span }), elem: Box::new(t), span: self.span };
                }
                fields.push((n, t));
            }
        }
        Ok(fields)
    }

    /// Translate a C type (specifiers plus pointer stars) into a Nexium type.
    fn ctype(&mut self, toks: &[CTok]) -> Result<TypeExpr, String> {
        let mut words: Vec<String> = Vec::new();
        let mut stars = 0;
        // `const T*` points at immutable data: the innermost pointer becomes `*T`
        let mut const_target = false;
        for t in toks {
            match t {
                CTok::Ident(s) if s == "const" || s == "__const" => {
                    if stars == 0 {
                        const_target = true;
                    }
                }
                CTok::Ident(s) if s == "volatile" => {}
                CTok::Ident(s) => words.push(s.clone()),
                CTok::Punct(p) if p == "*" => stars += 1,
                CTok::Punct(p) if p == "[" || p == "]" => {}
                CTok::Punct(p) => return Err(format!("unexpected `{}` in a type", p)),
                CTok::Num(_) => {}
                CTok::Str(_) => return Err("unexpected string in a type".into()),
            }
        }
        if words.is_empty() {
            return Err("missing type".into());
        }
        let sp = self.span;
        let base: TypeExpr = if words[0] == "struct" || words[0] == "union" || words[0] == "enum" {
            let tag = words.get(1).ok_or("missing tag")?.clone();
            if words[0] == "enum" {
                TypeExpr::named("i32", sp)
            } else {
                match self.typedefs.get(&format!("struct {}", tag)) {
                    Some(Ok(t)) => t.clone(),
                    Some(Err(e)) => return Err(e.clone()),
                    None => {
                        // forward-declared: opaque, only usable through pointers
                        if stars == 0 {
                            return Err(format!("struct {} is opaque here", tag));
                        }
                        TypeExpr::named("u8", sp)
                    }
                }
            }
        } else {
            let key = words.join(" ");
            let win = self.windows;
            let scalar = match key.as_str() {
                "void" => Some("void"),
                "char" | "signed char" | "unsigned char" => Some("u8"),
                "short" | "short int" | "signed short" | "signed short int" => Some("i16"),
                "unsigned short" | "unsigned short int" => Some("u16"),
                "int" | "signed" | "signed int" => Some("i32"),
                "unsigned" | "unsigned int" => Some("u32"),
                "long" | "long int" | "signed long" | "signed long int" => Some(if win { "i32" } else { "i64" }),
                "unsigned long" | "unsigned long int" => Some(if win { "u32" } else { "u64" }),
                "long long" | "long long int" | "signed long long" | "signed long long int" => Some("i64"),
                "unsigned long long" | "unsigned long long int" => Some("u64"),
                "float" => Some("f32"),
                "double" => Some("f64"),
                "long double" => None,
                "_Bool" | "bool" => Some("bool"),
                "size_t" | "uintptr_t" | "rsize_t" => Some("usize"),
                "ssize_t" | "ptrdiff_t" | "intptr_t" | "intmax_t" => Some("isize"),
                "uintmax_t" => Some("u64"),
                "int8_t" => Some("i8"),
                "int16_t" => Some("i16"),
                "int32_t" => Some("i32"),
                "int64_t" => Some("i64"),
                "uint8_t" => Some("u8"),
                "uint16_t" => Some("u16"),
                "uint32_t" => Some("u32"),
                "uint64_t" => Some("u64"),
                "wchar_t" => Some(if win { "u16" } else { "u32" }),
                "__int64" => Some("i64"),
                "unsigned __int64" => Some("u64"),
                "__int32" => Some("i32"),
                "__int128" => Some("i128"),
                _ => None,
            };
            match scalar {
                Some(s) => TypeExpr::named(s, sp),
                None if words.len() == 1 => match self.typedefs.get(&words[0]) {
                    Some(Ok(t)) => t.clone(),
                    Some(Err(e)) => return Err(e.clone()),
                    None => return Err(format!("unknown type `{}`", words[0])),
                },
                None => return Err(format!("unsupported type `{}`", key)),
            }
        };
        let mut t = base;
        let is_void = matches!(&t, TypeExpr::Named { path, .. } if path[0] == "void");
        for k in 0..stars {
            if k == 0 && is_void {
                t = TypeExpr::named("u8", sp); // void* is a pointer to bytes
            }
            t = TypeExpr::Ptr { mutable: !(k == 0 && const_target), elem: Box::new(t), span: sp };
        }
        Ok(t)
    }
}

fn is_type_word(s: &str) -> bool {
    matches!(
        s,
        "void"
            | "char"
            | "short"
            | "int"
            | "long"
            | "float"
            | "double"
            | "signed"
            | "unsigned"
            | "_Bool"
            | "bool"
            | "const"
            | "struct"
            | "union"
            | "enum"
            | "size_t"
            | "int8_t"
            | "int16_t"
            | "int32_t"
            | "int64_t"
            | "uint8_t"
            | "uint16_t"
            | "uint32_t"
            | "uint64_t"
            | "uintptr_t"
            | "intptr_t"
            | "ptrdiff_t"
            | "ssize_t"
            | "wchar_t"
    )
}

/// Remove attributes, calling conventions, and storage-class words.
fn strip_noise(toks: &[CTok]) -> Vec<CTok> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        match &toks[i] {
            CTok::Ident(s) if s == "__attribute__" || s == "__attribute" || s == "__declspec" || s == "__asm__" || s == "__asm" || s == "asm" || s == "_Alignas" || s == "__alignof__" => {
                // skip the parenthesized group
                i += 1;
                if matches!(toks.get(i), Some(CTok::Punct(p)) if p == "(") {
                    let mut depth = 0;
                    while i < toks.len() {
                        match &toks[i] {
                            CTok::Punct(p) if p == "(" => depth += 1,
                            CTok::Punct(p) if p == ")" => {
                                depth -= 1;
                                if depth == 0 {
                                    i += 1;
                                    break;
                                }
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                }
            }
            CTok::Ident(s) if SKIP_WORDS.contains(&s.as_str()) => i += 1,
            t => {
                out.push(t.clone());
                i += 1;
            }
        }
    }
    out
}

fn split_commas(toks: &[CTok]) -> Vec<Vec<CTok>> {
    let mut out = vec![Vec::new()];
    let mut depth = 0;
    for t in toks {
        match t {
            CTok::Punct(p) if p == "(" || p == "[" || p == "{" => depth += 1,
            CTok::Punct(p) if p == ")" || p == "]" || p == "}" => depth -= 1,
            CTok::Punct(p) if p == "," && depth == 0 => {
                out.push(Vec::new());
                continue;
            }
            _ => {}
        }
        out.last_mut().unwrap().push(t.clone());
    }
    out
}

fn parse_c_int(s: &str) -> Option<i128> {
    let t = s.trim_end_matches(|c: char| matches!(c, 'u' | 'U' | 'l' | 'L'));
    if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        return i128::from_str_radix(h, 16).ok();
    }
    if t.len() > 1 && t.starts_with('0') && t.chars().all(|c| c.is_ascii_digit()) {
        return i128::from_str_radix(&t[1..], 8).ok();
    }
    t.parse().ok()
}

fn eval_const_simple(toks: &[CTok]) -> Option<i128> {
    match toks {
        [CTok::Num(n)] => parse_c_int(n),
        [CTok::Punct(p), CTok::Num(n)] if p == "-" => parse_c_int(n).map(|v| -v),
        [CTok::Punct(l), inner @ .., CTok::Punct(r)] if l == "(" && r == ")" => eval_const_simple(inner),
        _ => None,
    }
}

/// Constant expressions in enums and macros: integers, negation, parentheses,
/// references to earlier constants, and `a << b`, `a | b`, `a + b`.
fn eval_const(toks: &[CTok], ci: &CImport) -> Option<i128> {
    let toks: Vec<CTok> = toks.iter().filter(|t| !matches!(t, CTok::Punct(p) if p == "(" || p == ")")).cloned().collect();
    let val = |t: &CTok| -> Option<i128> {
        match t {
            CTok::Num(n) => parse_c_int(n),
            CTok::Ident(n) => ci.consts.iter().find(|(k, _)| k == n).and_then(|(_, l)| if let CConst::Int(v) = l { Some(*v) } else { None }),
            _ => None,
        }
    };
    match toks.as_slice() {
        [a] => val(a),
        [CTok::Punct(p), a] if p == "-" => val(a).map(|v| -v),
        [CTok::Punct(p), a] if p == "~" => val(a).map(|v| !v),
        [a, CTok::Punct(op), b] => {
            let (x, y) = (val(a)?, val(b)?);
            Some(match op.as_str() {
                "<<" => x << y,
                ">>" => x >> y,
                "|" => x | y,
                "&" => x & y,
                "+" => x + y,
                "-" => x - y,
                "*" => x * y,
                _ => return None,
            })
        }
        _ => None,
    }
}

/// Object-like macros with literal values from `-dM` output.
fn parse_macros(text: &str, ci: &mut CImport) {
    let mut defs: HashMap<String, String> = HashMap::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("#define ") {
            let mut it = rest.splitn(2, ' ');
            let name = it.next().unwrap_or("").to_string();
            let value = it.next().unwrap_or("").trim().to_string();
            if name.contains('(') || name.is_empty() {
                continue;
            }
            defs.insert(name, value);
        }
    }
    let mut names: Vec<&String> = defs.keys().collect();
    names.sort();
    for name in names {
        if name.starts_with("__") && !name.starts_with("__STDC") {
            continue;
        }
        let mut value = defs[name].clone();
        // follow one level of indirection (INT_MAX -> __INT_MAX__)
        for _ in 0..3 {
            if let Some(v) = defs.get(value.trim_matches(|c| c == '(' || c == ')')) {
                value = v.clone();
            } else {
                break;
            }
        }
        let v = value.trim().trim_matches(|c| c == '(' || c == ')').trim();
        if v.is_empty() {
            continue;
        }
        let lit = if v.starts_with('"') && v.ends_with('"') && v.len() >= 2 {
            Some(CConst::Str(v[1..v.len() - 1].as_bytes().to_vec()))
        } else if let Some(i) = parse_c_int(v) {
            Some(CConst::Int(i))
        } else if let Some(i) = v.strip_prefix('-').and_then(parse_c_int) {
            Some(CConst::Int(-i))
        } else {
            let f = v.trim_end_matches(|c: char| matches!(c, 'f' | 'F' | 'l' | 'L'));
            f.parse::<f64>().ok().filter(|_| f.contains('.') || f.contains('e') || f.contains('E')).map(CConst::Float)
        };
        if let Some(l) = lit {
            if !ci.consts.iter().any(|(n, _)| n == name) {
                ci.consts.push((name.clone(), l));
            }
        }
    }
}
