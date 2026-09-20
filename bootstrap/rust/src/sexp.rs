//! `nx sexp`: the syntax tree as S-expressions, one node per line.
//!
//! This is the oracle for the self-hosted parser (`self/parser.nx`): both
//! parsers print this format for the same file and the test diffs the
//! output. The format is deliberately plain so it can be written from
//! either language without a serializer:
//!
//! - a node is `(kind attr... child...)`, where attributes are `key=value`
//!   words (`name=main`, `mut=true`) and children are nested nodes;
//! - a span is `@start:end` right after the kind, byte offsets in the file;
//! - strings and byte literals are quoted with `\xNN` for anything that is
//!   not printable ASCII, so the output is a single line per node;
//! - lists of children under a heading are `(params ...)`, `(args ...)`, so
//!   an empty list is visible and the order is fixed;
//! - integers are decimal, floats use the shortest round-trip form.
//!
//! Nothing here is meant for people to read; `nx parse` remains the pretty
//! dump.

use crate::ast::*;
use crate::diag::Span;
use std::fmt::Write;

pub struct Out {
    buf: String,
    depth: usize,
    /// the source text, so float literals print as written
    src: Vec<u8>,
}

impl Out {
    fn line(&mut self, s: &str) {
        for _ in 0..self.depth {
            self.buf.push_str("  ");
        }
        self.buf.push_str(s);
        self.buf.push('\n');
    }
    fn open(&mut self, s: &str) {
        self.line(s);
        self.depth += 1;
    }
    fn close(&mut self) {
        self.depth -= 1;
        self.line(")");
    }
}

fn sp(s: Span) -> String {
    format!("@{}:{}", s.start, s.end)
}

pub fn quote(bytes: &[u8]) -> String {
    let mut s = String::from("\"");
    for &b in bytes {
        match b {
            b'"' => s.push_str("\\\""),
            b'\\' => s.push_str("\\\\"),
            0x20..=0x7e => s.push(b as char),
            _ => {
                let _ = write!(s, "\\x{:02x}", b);
            }
        }
    }
    s.push('"');
    s
}

fn opt<T>(v: &Option<T>) -> String {
    if v.is_some() {
        "yes".into()
    } else {
        "no".into()
    }
}

fn names(o: &mut Out, head: &str, v: &[String]) {
    o.line(&format!("({} {})", head, v.join(" ")));
}

pub fn module(m: &Module, src: &str) -> String {
    let mut o = Out { buf: String::new(), depth: 0, src: src.as_bytes().to_vec() };
    o.open("(module");
    for it in &m.items {
        item(&mut o, it);
    }
    o.close();
    o.buf
}

fn attrs(o: &mut Out, a: &Attrs) {
    o.line(&format!("(attrs pub={} docs={})", a.is_pub, a.doc.len()));
}

fn effects(o: &mut Out, head: &str, v: &[EffectBound]) {
    let parts: Vec<String> = v.iter().map(|e| format!("{}{}", if e.negative { "!" } else { "" }, e.name)).collect();
    o.line(&format!("({} {})", head, parts.join(" ")));
}

fn item(o: &mut Out, it: &Item) {
    match it {
        Item::Fn(f) => fn_decl(o, f),
        Item::Struct(s) => {
            let kind = match s.kind {
                StructKind::Struct => "struct",
                StructKind::Record => "record",
                StructKind::RefClass => "ref",
            };
            let layout = match s.layout {
                Layout::Default => "default",
                Layout::C => "c",
            };
            o.open(&format!("({} {} name={} layout={}", kind, sp(s.span), s.name, layout));
            attrs(o, &s.attrs);
            names(o, "tparams", &s.type_params);
            names(o, "derives", &s.derives);
            o.open("(fields");
            for f in &s.fields {
                field(o, f);
            }
            o.close();
            o.close();
        }
        Item::Enum(e) => {
            o.open(&format!("(enum {} name={}", sp(e.span), e.name));
            attrs(o, &e.attrs);
            names(o, "tparams", &e.type_params);
            names(o, "derives", &e.derives);
            o.open("(variants");
            for v in &e.variants {
                match &v.payload {
                    VariantPayload::Unit => o.line(&format!("(variant {} name={} unit)", sp(v.span), v.name)),
                    VariantPayload::Tuple(ts) => {
                        o.open(&format!("(variant {} name={} tuple", sp(v.span), v.name));
                        for t in ts {
                            ty(o, t);
                        }
                        o.close();
                    }
                    VariantPayload::Struct(fs) => {
                        o.open(&format!("(variant {} name={} struct", sp(v.span), v.name));
                        for f in fs {
                            field(o, f);
                        }
                        o.close();
                    }
                }
            }
            o.close();
            o.close();
        }
        Item::Trait(t) => {
            o.open(&format!("(trait {} name={}", sp(t.span), t.name));
            attrs(o, &t.attrs);
            names(o, "assoc", &t.assoc_types);
            for m in &t.methods {
                fn_decl(o, m);
            }
            o.close();
        }
        Item::Impl(i) => {
            o.open(&format!("(impl {} trait={}", sp(i.span), i.trait_name.clone().unwrap_or_else(|| "-".into())));
            names(o, "tparams", &i.type_params);
            ty(o, &i.target);
            for (n, t) in &i.assoc_types {
                o.open(&format!("(assoc name={}", n));
                ty(o, t);
                o.close();
            }
            for m in &i.methods {
                fn_decl(o, m);
            }
            o.close();
        }
        Item::Const(c) => {
            o.open(&format!("(const {} name={}", sp(c.span), c.name));
            attrs(o, &c.attrs);
            if let Some(t) = &c.ty {
                ty(o, t);
            } else {
                o.line("(notype)");
            }
            expr(o, &c.value);
            o.close();
        }
        Item::Global(g) => {
            o.open(&format!("(global {} name={} align={}", sp(g.span), g.name, g.align.map(|a| a.to_string()).unwrap_or_else(|| "-".into())));
            attrs(o, &g.attrs);
            ty(o, &g.ty);
            match &g.value {
                Some(v) => expr(o, v),
                None => o.line("(undefined)"),
            }
            o.close();
        }
        Item::TypeAlias(t) => {
            o.open(&format!("(type {} name={} distinct={}", sp(t.span), t.name, t.distinct));
            attrs(o, &t.attrs);
            ty(o, &t.ty);
            o.close();
        }
        Item::ErrorSet(e) => {
            o.open(&format!("(error {} name={}", sp(e.span), e.name));
            attrs(o, &e.attrs);
            names(o, "names", &e.names);
            o.close();
        }
        Item::Import(i) => {
            o.line(&format!(
                "(import {} path={} alias={} names={})",
                sp(i.span),
                i.path.join("."),
                i.alias.clone().unwrap_or_else(|| "-".into()),
                i.names.as_ref().map(|n| n.join(",")).unwrap_or_else(|| "-".into())
            ));
        }
        Item::Test(t) => {
            o.open(&format!("(test {} name={} comptime={}", sp(t.span), quote(t.name.as_bytes()), t.comptime));
            block(o, &t.body);
            o.close();
        }
        Item::Artifact(a) => {
            o.open(&format!("(artifact {} kind={}", sp(a.span), a.kind));
            for (k, v, s) in &a.fields {
                o.open(&format!("(field {} name={}", sp(*s), k));
                expr(o, v);
                o.close();
            }
            o.close();
        }
    }
}

fn fn_decl(o: &mut Out, f: &FnDecl) {
    o.open(&format!("(fn {} name={} export={} extern={} variadic={} body={}", sp(f.span), f.name, f.export.clone().unwrap_or_else(|| "-".into()), f.extern_c, f.variadic, opt(&f.body)));
    attrs(o, &f.attrs);
    o.open("(params");
    for p in &f.params {
        param(o, p);
    }
    o.close();
    match &f.ret {
        Some(t) => ty(o, t),
        None => o.line("(noret)"),
    }
    effects(o, "effects", &f.effects);
    for w in &f.wheres {
        o.open(&format!("(where {} param={}", sp(w.span), w.param));
        names(o, "bounds", &w.bounds);
        effects(o, "effects", &w.effect_bounds);
        o.close();
    }
    if let Some(b) = &f.body {
        block(o, b);
    }
    o.close();
}

fn param(o: &mut Out, p: &Param) {
    o.open(&format!("(param {} name={} comptime={} own={}", sp(p.span), p.name, p.comptime, p.owned));
    ty(o, &p.ty);
    o.close();
}

fn field(o: &mut Out, f: &Field) {
    o.open(&format!("(field {} name={} docs={}", sp(f.span), f.name, f.doc.len()));
    ty(o, &f.ty);
    if let Some(d) = &f.default {
        o.open("(default");
        expr(o, d);
        o.close();
    }
    if let Some(c) = &f.constraint {
        o.open("(where");
        expr(o, c);
        o.close();
    }
    o.close();
}

fn ty(o: &mut Out, t: &TypeExpr) {
    match t {
        TypeExpr::Named { path, args, span } => {
            if args.is_empty() {
                o.line(&format!("(tname {} path={})", sp(*span), path.join(".")));
            } else {
                o.open(&format!("(tname {} path={}", sp(*span), path.join(".")));
                for a in args {
                    ty(o, a);
                }
                o.close();
            }
        }
        TypeExpr::Array { len, elem, span } => {
            o.open(&format!("(tarray {}", sp(*span)));
            expr(o, len);
            ty(o, elem);
            o.close();
        }
        TypeExpr::Slice { mutable, elem, span } => {
            o.open(&format!("(tslice {} mut={}", sp(*span), mutable));
            ty(o, elem);
            o.close();
        }
        TypeExpr::Ptr { mutable, elem, span } => {
            o.open(&format!("(tptr {} mut={}", sp(*span), mutable));
            ty(o, elem);
            o.close();
        }
        TypeExpr::Optional { elem, span } => {
            o.open(&format!("(topt {}", sp(*span)));
            ty(o, elem);
            o.close();
        }
        TypeExpr::ErrorUnion { set, elem, span } => {
            o.open(&format!("(terr {} set={}", sp(*span), opt(set)));
            if let Some(s) = set {
                ty(o, s);
            }
            ty(o, elem);
            o.close();
        }
        TypeExpr::Fn { params, ret, effects: eff, span } => {
            o.open(&format!("(tfn {}", sp(*span)));
            o.open("(params");
            for p in params {
                ty(o, p);
            }
            o.close();
            ty(o, ret);
            effects(o, "effects", eff);
            o.close();
        }
        TypeExpr::Tuple { elems, span } => {
            o.open(&format!("(ttuple {}", sp(*span)));
            for e in elems {
                ty(o, e);
            }
            o.close();
        }
        TypeExpr::Weak { elem, span } => {
            o.open(&format!("(tweak {}", sp(*span)));
            ty(o, elem);
            o.close();
        }
        TypeExpr::Dyn { trait_name, effects: eff, span } => {
            o.open(&format!("(tdyn {} trait={}", sp(*span), trait_name));
            effects(o, "effects", eff);
            o.close();
        }
        TypeExpr::Infer { span } => o.line(&format!("(tinfer {})", sp(*span))),
    }
}

fn block(o: &mut Out, b: &Block) {
    o.open(&format!("(block {} label={}", sp(b.span), b.label.clone().unwrap_or_else(|| "-".into())));
    for s in &b.stmts {
        stmt(o, s);
    }
    if let Some(t) = &b.tail {
        o.open("(tail");
        expr(o, t);
        o.close();
    }
    o.close();
}

fn stmt(o: &mut Out, s: &Stmt) {
    match s {
        Stmt::Let { mutable, name, ty: t, init, span } => {
            o.open(&format!("(let {} name={} mut={}", sp(*span), name, mutable));
            match t {
                Some(t) => ty(o, t),
                None => o.line("(notype)"),
            }
            match init {
                Some(e) => expr(o, e),
                None => o.line("(noinit)"),
            }
            o.close();
        }
        Stmt::Assign { target, op, value, span } => {
            o.open(&format!("(assign {} op={}", sp(*span), op.map(|b| b.symbol().to_string()).unwrap_or_else(|| "=".into())));
            expr(o, target);
            expr(o, value);
            o.close();
        }
        Stmt::Expr(e) => {
            o.open("(expr");
            expr(o, e);
            o.close();
        }
        Stmt::Return { value, span } => {
            o.open(&format!("(return {}", sp(*span)));
            if let Some(v) = value {
                expr(o, v);
            }
            o.close();
        }
        Stmt::Break { label, value, span } => {
            o.open(&format!("(break {} label={}", sp(*span), label.clone().unwrap_or_else(|| "-".into())));
            if let Some(v) = value {
                expr(o, v);
            }
            o.close();
        }
        Stmt::Continue { label, span } => o.line(&format!("(continue {} label={})", sp(*span), label.clone().unwrap_or_else(|| "-".into()))),
        Stmt::Defer { body, span } => {
            o.open(&format!("(defer {}", sp(*span)));
            stmt(o, body);
            o.close();
        }
        Stmt::ErrDefer { body, span } => {
            o.open(&format!("(errdefer {}", sp(*span)));
            stmt(o, body);
            o.close();
        }
        Stmt::While { cond, body, els, label, span } => {
            o.open(&format!("(while {} label={}", sp(*span), label.clone().unwrap_or_else(|| "-".into())));
            expr(o, cond);
            block(o, body);
            if let Some(e) = els {
                o.open("(else");
                block(o, e);
                o.close();
            }
            o.close();
        }
        Stmt::For { iter, bindings, body, label, parallel, span } => {
            o.open(&format!("(for {} label={} parallel={}", sp(*span), label.clone().unwrap_or_else(|| "-".into()), parallel));
            names(o, "bindings", bindings);
            match iter {
                ForIter::Range { start, end, step } => {
                    o.open("(range");
                    expr(o, start);
                    expr(o, end);
                    if let Some(s) = step {
                        o.open("(step");
                        expr(o, s);
                        o.close();
                    }
                    o.close();
                }
                ForIter::Items(items) => {
                    o.open("(items");
                    for i in items {
                        expr(o, i);
                    }
                    o.close();
                }
            }
            block(o, body);
            o.close();
        }
        Stmt::Unsafe { body, span } => {
            o.open(&format!("(unsafe {}", sp(*span)));
            block(o, body);
            o.close();
        }
        Stmt::Discard { value, span } => {
            o.open(&format!("(discard {}", sp(*span)));
            expr(o, value);
            o.close();
        }
        Stmt::Using { strategy, body, span } => {
            o.open(&format!("(using {} strategy={}", sp(*span), strategy));
            block(o, body);
            o.close();
        }
    }
}

fn lit(o: &Out, v: &Lit, span: Span) -> String {
    match v {
        Lit::Int(i) => format!("int {}", i),
        Lit::Float(_) => {
            let text = o.src.get(span.start as usize..span.end as usize).map(|b| String::from_utf8_lossy(b).replace('_', "")).unwrap_or_default();
            format!("float {}", text)
        }
        Lit::Str(s) => format!("str {}", quote(s)),
        Lit::Bytes(s) => format!("bytes {}", quote(s)),
        Lit::Char(c) => format!("char {}", c),
        Lit::Bool(b) => format!("bool {}", b),
    }
}

fn unop(u: UnOp) -> &'static str {
    match u {
        UnOp::Neg => "-",
        UnOp::Not => "!",
        UnOp::BitNot => "~",
        UnOp::AddrOf => "&",
        UnOp::AddrOfMut => "&mut",
    }
}

fn segments(o: &mut Out, segs: &[BinSegment]) {
    for s in segs {
        let mods = if s.modifiers.is_empty() { "-".to_string() } else { s.modifiers.join(",") };
        match &s.value {
            BinSegValue::Bind(n) => o.open(&format!("(seg {} bind={} mods={} size={}", sp(s.span), n, mods, opt(&s.size))),
            BinSegValue::Rest(n) => o.open(&format!("(seg {} rest={} mods={} size={}", sp(s.span), n, mods, opt(&s.size))),
            BinSegValue::Expr(e) => {
                o.open(&format!("(seg {} expr mods={} size={}", sp(s.span), mods, opt(&s.size)));
                expr(o, e);
            }
        }
        if let Some(sz) = &s.size {
            o.open("(size");
            expr(o, sz);
            o.close();
        }
        o.close();
    }
}

fn expr(o: &mut Out, e: &Expr) {
    match e {
        Expr::Lit { value, span } => {
            let l = lit(o, value, *span);
            o.line(&format!("(lit {} {})", sp(*span), l));
        }
        Expr::Ident { name, span } => o.line(&format!("(ident {} name={})", sp(*span), name)),
        Expr::Field { base, name, span } => {
            o.open(&format!("(field {} name={}", sp(*span), name));
            expr(o, base);
            o.close();
        }
        Expr::ImplicitVariant { name, args, span } => {
            o.open(&format!("(variant {} name={}", sp(*span), name));
            for a in args {
                expr(o, a);
            }
            o.close();
        }
        Expr::Index { base, index, span } => {
            o.open(&format!("(index {}", sp(*span)));
            expr(o, base);
            expr(o, index);
            o.close();
        }
        Expr::SliceOp { base, start, end, span } => {
            o.open(&format!("(slice {} start={} end={}", sp(*span), opt(start), opt(end)));
            expr(o, base);
            if let Some(s) = start {
                expr(o, s);
            }
            if let Some(e) = end {
                expr(o, e);
            }
            o.close();
        }
        Expr::Call { callee, args, span } => {
            o.open(&format!("(call {}", sp(*span)));
            expr(o, callee);
            o.open("(args");
            for a in args {
                expr(o, a);
            }
            o.close();
            o.close();
        }
        Expr::MethodCall { receiver, method, args, span } => {
            o.open(&format!("(mcall {} name={}", sp(*span), method));
            expr(o, receiver);
            o.open("(args");
            for a in args {
                expr(o, a);
            }
            o.close();
            o.close();
        }
        Expr::Unary { op, expr: x, span } => {
            o.open(&format!("(unary {} op={}", sp(*span), unop(*op)));
            expr(o, x);
            o.close();
        }
        Expr::Binary { op, lhs, rhs, span } => {
            o.open(&format!("(binary {} op={}", sp(*span), op.symbol()));
            expr(o, lhs);
            expr(o, rhs);
            o.close();
        }
        Expr::Pipe { lhs, rhs, span } => {
            o.open(&format!("(pipe {}", sp(*span)));
            expr(o, lhs);
            expr(o, rhs);
            o.close();
        }
        Expr::Try { expr: x, span } => {
            o.open(&format!("(try {}", sp(*span)));
            expr(o, x);
            o.close();
        }
        Expr::Catch { expr: x, binding, handler, span } => {
            o.open(&format!("(catch {} bind={}", sp(*span), binding.clone().unwrap_or_else(|| "-".into())));
            expr(o, x);
            expr(o, handler);
            o.close();
        }
        Expr::OrElse { expr: x, default, span } => {
            o.open(&format!("(orelse {}", sp(*span)));
            expr(o, x);
            expr(o, default);
            o.close();
        }
        Expr::Unwrap { expr: x, span } => {
            o.open(&format!("(unwrap {}", sp(*span)));
            expr(o, x);
            o.close();
        }
        Expr::Deref { expr: x, span } => {
            o.open(&format!("(deref {}", sp(*span)));
            expr(o, x);
            o.close();
        }
        Expr::If { cond, then, els, span } => {
            o.open(&format!("(if {} else={}", sp(*span), opt(els)));
            expr(o, cond);
            block(o, then);
            if let Some(e) = els {
                expr(o, e);
            }
            o.close();
        }
        Expr::IfCapture { cond, binding, then, els, span } => {
            o.open(&format!("(ifcap {} bind={} else={}", sp(*span), binding, opt(els)));
            expr(o, cond);
            block(o, then);
            if let Some(e) = els {
                expr(o, e);
            }
            o.close();
        }
        Expr::Match { scrutinee, arms, span } => {
            o.open(&format!("(match {}", sp(*span)));
            expr(o, scrutinee);
            for a in arms {
                o.open(&format!("(arm {} guard={}", sp(a.span), opt(&a.guard)));
                pattern(o, &a.pat);
                if let Some(g) = &a.guard {
                    expr(o, g);
                }
                expr(o, &a.body);
                o.close();
            }
            o.close();
        }
        Expr::Block(b) => block(o, b),
        Expr::StructLit { ty: t, fields, span } => {
            o.open(&format!("(structlit {} type={}", sp(*span), opt(t)));
            if let Some(t) = t {
                ty(o, t);
            }
            for (n, v, s) in fields {
                o.open(&format!("(init {} name={}", sp(*s), n));
                expr(o, v);
                o.close();
            }
            o.close();
        }
        Expr::ArrayLit { elems, span } => {
            o.open(&format!("(array {}", sp(*span)));
            for x in elems {
                expr(o, x);
            }
            o.close();
        }
        Expr::TupleLit { elems, span } => {
            o.open(&format!("(tuple {}", sp(*span)));
            for x in elems {
                expr(o, x);
            }
            o.close();
        }
        Expr::Closure(c) => {
            o.open(&format!("(closure {} ret={}", sp(c.span), opt(&c.ret)));
            o.open("(captures");
            for cap in &c.captures {
                o.line(&format!("(capture {} name={} ref={} mut={})", sp(cap.span), cap.name, cap.by_ref, cap.mutable));
            }
            o.close();
            o.open("(params");
            for p in &c.params {
                param(o, p);
            }
            o.close();
            if let Some(t) = &c.ret {
                ty(o, t);
            }
            expr(o, &c.body);
            o.close();
        }
        Expr::Range { start, end, span } => {
            o.open(&format!("(range {}", sp(*span)));
            expr(o, start);
            expr(o, end);
            o.close();
        }
        Expr::Cast { expr: x, ty: t, span } => {
            o.open(&format!("(cast {}", sp(*span)));
            expr(o, x);
            ty(o, t);
            o.close();
        }
        Expr::Null { span } => o.line(&format!("(null {})", sp(*span))),
        Expr::Undefined { span } => o.line(&format!("(undefined {})", sp(*span))),
        Expr::Unreachable { span } => o.line(&format!("(unreachable {})", sp(*span))),
        Expr::ErrorLit { name, span } => o.line(&format!("(errorlit {} name={})", sp(*span), name)),
        Expr::Builtin { name, args, span } => {
            o.open(&format!("(builtin {} name={}", sp(*span), name));
            for a in args {
                expr(o, a);
            }
            o.close();
        }
        Expr::BinConstruct { segments: segs, target, span } => {
            o.open(&format!("(binbuild {}", sp(*span)));
            segments(o, segs);
            o.open("(into");
            expr(o, target);
            o.close();
            o.close();
        }
        Expr::Comptime { expr: x, span } => {
            o.open(&format!("(comptime {}", sp(*span)));
            expr(o, x);
            o.close();
        }
        Expr::TypeVal { ty: t, span } => {
            o.open(&format!("(typeval {}", sp(*span)));
            ty(o, t);
            o.close();
        }
        Expr::Unsafe { body, span } => {
            o.open(&format!("(unsafe {}", sp(*span)));
            block(o, body);
            o.close();
        }
    }
}

fn pattern(o: &mut Out, p: &Pattern) {
    match p {
        Pattern::Wildcard { span } => o.line(&format!("(pwild {})", sp(*span))),
        Pattern::Binding { name, span } => o.line(&format!("(pbind {} name={})", sp(*span), name)),
        Pattern::Lit { value, negative, span } => {
            // a negative literal's span covers the minus; the number itself follows it
            let lspan = if *negative { Span { file: span.file, start: span.start + 1, end: span.end } } else { *span };
            let l = lit(o, value, lspan);
            o.line(&format!("(plit {} neg={} {})", sp(*span), negative, l));
        }
        Pattern::Variant { path, args, span } => {
            o.open(&format!("(pvariant {} path={}", sp(*span), path.join(".")));
            for a in args {
                pattern(o, a);
            }
            o.close();
        }
        Pattern::Error { name, span } => o.line(&format!("(perror {} name={})", sp(*span), name)),
        Pattern::Null { span } => o.line(&format!("(pnull {})", sp(*span))),
        Pattern::Range { lo, hi, inclusive, span } => {
            o.open(&format!("(prange {} inclusive={}", sp(*span), inclusive));
            expr(o, lo);
            expr(o, hi);
            o.close();
        }
        Pattern::Or { alts, span } => {
            o.open(&format!("(por {}", sp(*span)));
            for a in alts {
                pattern(o, a);
            }
            o.close();
        }
        Pattern::Binary { segments: segs, span } => {
            o.open(&format!("(pbinary {}", sp(*span)));
            segments(o, segs);
            o.close();
        }
        Pattern::Tuple { elems, span } => {
            o.open(&format!("(ptuple {}", sp(*span)));
            for e in elems {
                pattern(o, e);
            }
            o.close();
        }
    }
}
