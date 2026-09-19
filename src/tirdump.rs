//! `nx tir`: the checked program as S-expressions, the oracle for the
//! self-hosted checker (`self/check.nx`), the way `nx sexp` is for the
//! parser. Types print structurally (never by table index) so two checkers
//! that intern in a different order still agree. `--sigs` prints only the
//! declarations and function signatures, the first milestone of the port.

use crate::ast::{BinOp, Layout, StructKind, UnOp};
use crate::check::{Program, VariantPayloadDef};
use crate::diag::Span;
use crate::effects::Effects;
use crate::tir::*;
use crate::types::{FloatTy, TyId, TyKind, TyTable};

struct Out<'a> {
    p: &'a Program,
    tys: TyTable,
    s: String,
    depth: usize,
}

impl<'a> Out<'a> {
    fn line(&mut self, text: &str) {
        for _ in 0..self.depth {
            self.s.push_str("  ");
        }
        self.s.push_str(text);
        self.s.push('\n');
    }
    fn open(&mut self, text: &str) {
        self.line(text);
        self.depth += 1;
    }
    fn close(&mut self) {
        self.depth -= 1;
        self.line(")");
    }

    fn ty(&mut self, t: TyId) -> String {
        let t = self.tys.resolve(t, true);
        match self.tys.kind(t).clone() {
            TyKind::Int(i) => i.name().into(),
            TyKind::Float(FloatTy::F32) => "f32".into(),
            TyKind::Float(FloatTy::F64) => "f64".into(),
            TyKind::Bool => "bool".into(),
            TyKind::Char => "char".into(),
            TyKind::Void => "void".into(),
            TyKind::Never => "never".into(),
            TyKind::Struct(d, args) => {
                let n = self.p.structs[d as usize].name.clone();
                if args.is_empty() {
                    n
                } else {
                    format!("{}({})", n, self.tys_list(&args))
                }
            }
            TyKind::Enum(d, args) => {
                let n = self.p.enums[d as usize].name.clone();
                if args.is_empty() {
                    n
                } else {
                    format!("{}({})", n, self.tys_list(&args))
                }
            }
            TyKind::Array(n, e) => format!("[{}]{}", n, self.ty(e)),
            TyKind::Slice(m, e) => format!("[]{}{}", if m { "mut " } else { "" }, self.ty(e)),
            TyKind::Ptr(m, e) => format!("*{}{}", if m { "mut " } else { "" }, self.ty(e)),
            TyKind::Opt(e) => format!("?{}", self.ty(e)),
            TyKind::ErrUnion(_, e) => format!("!{}", self.ty(e)),
            TyKind::Fn(ps, r, ef) => {
                let neg = Effects(ef).render_negative();
                format!("fn({}) -> {}{}", self.tys_list(&ps), self.ty(r), if neg.is_empty() { String::new() } else { format!(" {}", neg) })
            }
            TyKind::Tuple(ts) => format!("({})", self.tys_list(&ts)),
            TyKind::Distinct(d) => self.p.aliases[d as usize].name.clone(),
            TyKind::Weak(e) => format!("weak {}", self.ty(e)),
            TyKind::List(e) => format!("List({})", self.ty(e)),
            TyKind::Str => "String".into(),
            TyKind::Map(k, v) => format!("Map({}, {})", self.ty(k), self.ty(v)),
            TyKind::Type => "type".into(),
            TyKind::ErrorSet(_) => "error".into(),
            TyKind::Closure(n) => format!("closure#{}", n),
            TyKind::Param(n) => n,
            TyKind::Namespace(n) => format!("namespace {}", n),
            TyKind::Dyn(t, ef) => {
                let neg = Effects(ef).render_negative();
                format!("dyn {}{}", self.p.traits[t as usize].name, if neg.is_empty() { String::new() } else { format!(" {}", neg) })
            }
            TyKind::Infer(_) => "_".into(),
            TyKind::IntLit => "{integer}".into(),
            TyKind::FloatLit => "{float}".into(),
        }
    }

    fn tys_list(&mut self, ts: &[TyId]) -> String {
        ts.iter().map(|&t| self.ty(t)).collect::<Vec<_>>().join(", ")
    }
}

fn sp(s: Span) -> String {
    format!("@{}:{}", s.start, s.end)
}

fn effects(e: Effects) -> String {
    format!("({})", e.names().join(" "))
}

fn quote(b: &[u8]) -> String {
    let mut s = String::from("\"");
    for &c in b {
        match c {
            b'"' => s.push_str("\\\""),
            b'\\' => s.push_str("\\\\"),
            b'\n' => s.push_str("\\n"),
            b'\r' => s.push_str("\\r"),
            b'\t' => s.push_str("\\t"),
            0x20..=0x7e => s.push(c as char),
            _ => s.push_str(&format!("\\x{:02x}", c)),
        }
    }
    s.push('"');
    s
}

fn binop(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Rem => "%",
        BinOp::AddWrap => "+%",
        BinOp::SubWrap => "-%",
        BinOp::MulWrap => "*%",
        BinOp::AddSat => "+|",
        BinOp::SubSat => "-|",
        BinOp::MulSat => "*|",
        BinOp::BitAnd => "&",
        BinOp::BitOr => "|",
        BinOp::BitXor => "^",
        BinOp::Shl => "<<",
        BinOp::Shr => ">>",
        BinOp::Eq => "==",
        BinOp::Ne => "!=",
        BinOp::Lt => "<",
        BinOp::Le => "<=",
        BinOp::Gt => ">",
        BinOp::Ge => ">=",
        BinOp::And => "and",
        BinOp::Or => "or",
    }
}

fn unop(op: UnOp) -> &'static str {
    match op {
        UnOp::Neg => "-",
        UnOp::Not => "!",
        UnOp::BitNot => "~",
        UnOp::AddrOf => "&",
        UnOp::AddrOfMut => "&mut",
    }
}

fn mode(m: ArithMode) -> &'static str {
    match m {
        ArithMode::Checked => "checked",
        ArithMode::Wrap => "wrap",
        ArithMode::Sat => "sat",
        ArithMode::Float => "float",
        ArithMode::Plain => "plain",
    }
}

fn cast(k: CastKind) -> String {
    match k {
        CastKind::IntToInt { checked } => format!("int_to_int checked={}", checked),
        CastKind::IntToFloat => "int_to_float".into(),
        CastKind::FloatToInt => "float_to_int".into(),
        CastKind::FloatToFloat => "float_to_float".into(),
        CastKind::Bits => "bits".into(),
        CastKind::PtrToPtr => "ptr_to_ptr".into(),
    }
}

fn opt<T>(v: &Option<T>) -> &'static str {
    if v.is_some() {
        "yes"
    } else {
        "no"
    }
}

pub fn program(p: &Program, sigs_only: bool) -> String {
    let mut o = Out { p, tys: p.tys.clone(), s: String::new(), depth: 0 };
    o.open("(program");
    o.open("(modules");
    for (i, m) in p.modules.iter().enumerate() {
        o.line(&format!("(module {} {})", i, m));
    }
    o.close();
    o.open("(structs");
    for (i, s) in p.structs.iter().enumerate() {
        let kind = match s.kind {
            StructKind::Struct => "struct",
            StructKind::Record => "record",
            StructKind::RefClass => "ref",
        };
        let layout = match s.layout {
            Layout::Default => "default",
            Layout::C => "c",
            Layout::Packed => "packed",
        };
        o.open(&format!(
            "(struct {} {} {} kind={} layout={} module={} pub={} tparams=({}) derives=({})",
            i,
            s.name,
            sp(s.span),
            kind,
            layout,
            s.module,
            s.is_pub,
            s.type_params.join(" "),
            s.derives.join(" ")
        ));
        for f in &s.fields {
            o.line(&format!("(field {} {} default={} constraint={})", f.name, sp(f.span), opt(&f.default), opt(&f.constraint)));
        }
        o.close();
    }
    o.close();
    o.open("(enums");
    for (i, e) in p.enums.iter().enumerate() {
        o.open(&format!("(enum {} {} {} module={} tparams=({}) derives=({})", i, e.name, sp(e.span), e.module, e.type_params.join(" "), e.derives.join(" ")));
        for v in &e.variants {
            let payload = match &v.payload {
                VariantPayloadDef::Unit => "unit".to_string(),
                VariantPayloadDef::Tuple(ts) => format!("tuple {}", ts.len()),
                VariantPayloadDef::Struct(fs) => format!("struct ({})", fs.iter().map(|f| f.name.clone()).collect::<Vec<_>>().join(" ")),
            };
            o.line(&format!("(variant {} {} {})", v.name, sp(v.span), payload));
        }
        o.close();
    }
    o.close();
    o.open("(errors");
    for (i, e) in p.error_names.iter().enumerate() {
        o.line(&format!("(error {} {})", i + 1, e));
    }
    o.close();
    o.open("(aliases");
    for (i, a) in p.aliases.iter().enumerate() {
        let t = a.resolved.map(|t| o.ty(t)).unwrap_or_else(|| "-".into());
        o.line(&format!("(alias {} {} {} distinct={} module={} ty={})", i, a.name, sp(a.span), a.distinct, a.module, t));
    }
    o.close();
    o.open("(traits");
    for (i, t) in p.traits.iter().enumerate() {
        o.open(&format!("(trait {} {} {} module={} assoc=({})", i, t.name, sp(t.span), t.module, t.assoc_types.join(" ")));
        for m in &t.methods {
            o.line(&format!("(method {} {} params={} body={})", m.name, sp(m.span), m.params.len(), opt(&m.body)));
        }
        o.close();
    }
    o.close();
    o.open("(consts");
    for (i, c) in p.consts.iter().enumerate() {
        let t = c.resolved.as_ref().map(|(t, _)| o.ty(*t)).unwrap_or_else(|| "-".into());
        o.line(&format!("(const {} {} {} module={} pub={} mangled={} ty={})", i, c.name, sp(c.span), c.module, c.is_pub, c.mangled, t));
    }
    o.close();
    o.open("(globals");
    for (i, g) in p.globals.iter().enumerate() {
        let t = g.ty.map(|t| o.ty(t)).unwrap_or_else(|| "-".into());
        o.line(&format!("(global {} {} {} module={} mangled={} ty={} init={})", i, g.name, sp(g.span), g.module, g.mangled, t, opt(&g.init)));
    }
    o.close();
    o.open("(fns");
    // signatures come in declaration order, so the numbering that body
    // checking assigns (instances are created as calls are met) plays no part
    let mut order: Vec<usize> = (0..p.funcs.len()).collect();
    if sigs_only {
        order.sort_by_key(|&i| (p.funcs[i].module, p.funcs[i].span.start, p.funcs[i].span.end));
    }
    for i in order {
        let f = &p.funcs[i];
        if sigs_only && (f.is_closure || !f.targs.is_empty()) {
            // closures and generic instances only exist once bodies are checked
            continue;
        }
        let ret = o.ty(f.ret);
        let targs = o.tys_list(&f.targs);
        // a mangled name that ends in the instance number is decided by body checking too
        let instance_named = f.mangled.ends_with(&format!("_{}", i));
        let index = if sigs_only { "-".to_string() } else { i.to_string() };
        let mangled = if sigs_only && instance_named { "-".to_string() } else { f.mangled.clone() };
        o.open(&format!(
            "(fn {} {} {} mangled={} module={} ret={} targs=({}) export={} extern={} variadic={} pub={} test={} ctest={} closure={} body={}",
            index,
            f.name,
            sp(f.span),
            mangled,
            f.module,
            ret,
            targs,
            f.export.clone().unwrap_or_else(|| "-".into()),
            f.is_extern,
            f.is_variadic,
            f.is_pub,
            f.is_test,
            f.test_comptime,
            f.is_closure,
            opt(&f.body)
        ));
        let mut params = Vec::new();
        for &pi in &f.params {
            let l = &f.locals[pi as usize];
            let t = o.ty(l.ty);
            params.push(format!("({} {} own={})", l.name, t, l.owned));
        }
        o.line(&format!("(params {})", params.join(" ")));
        o.line(&format!("(declared neg={} pos={})", effects(f.declared_neg), effects(f.declared_pos)));
        if !sigs_only {
            o.line(&format!("(effects own={} all={})", effects(f.own_effects), effects(f.effects)));
            o.open("(locals");
            for (li, l) in f.locals.iter().enumerate() {
                let t = o.ty(l.ty);
                o.line(&format!("(local {} {} {} {} mut={} param={} own={} loop={})", li, l.name, sp(l.span), t, l.mutable, l.is_param, l.owned, l.loop_item));
            }
            o.close();
            if let Some(b) = &f.body {
                block(&mut o, b);
            }
        }
        o.close();
    }
    o.close();
    o.close();
    o.s
}

fn block(o: &mut Out, b: &TBlock) {
    let t = o.ty(b.ty);
    let label = b.label.map(|l| l.to_string()).unwrap_or_else(|| "-".into());
    o.open(&format!("(block {} :{} label={}", sp(b.span), t, label));
    for s in &b.stmts {
        stmt(o, s);
    }
    if let Some(e) = &b.tail {
        o.open("(tail");
        expr(o, e);
        o.close();
    }
    o.close();
}

fn stmt(o: &mut Out, s: &TStmt) {
    match s {
        TStmt::Let { local, init, span } => {
            o.open(&format!("(let {} local={}", sp(*span), local));
            if let Some(e) = init {
                expr(o, e);
            }
            o.close();
        }
        TStmt::Assign { target, op, value, span } => {
            let op = op.map(|(b, m)| format!("{}= {}", binop(b), mode(m))).unwrap_or_else(|| "=".into());
            o.open(&format!("(assign {} op={}", sp(*span), op));
            expr(o, target);
            expr(o, value);
            o.close();
        }
        TStmt::Expr(e) => {
            o.open("(expr");
            expr(o, e);
            o.close();
        }
        TStmt::Return { value, span } => {
            o.open(&format!("(return {}", sp(*span)));
            if let Some(e) = value {
                expr(o, e);
            }
            o.close();
        }
        TStmt::Break { label, value, span } => {
            o.open(&format!("(break {} label={}", sp(*span), label));
            if let Some(e) = value {
                expr(o, e);
            }
            o.close();
        }
        TStmt::Continue { label, span } => o.line(&format!("(continue {} label={})", sp(*span), label)),
        TStmt::Defer { body, span } => {
            o.open(&format!("(defer {}", sp(*span)));
            stmt(o, body);
            o.close();
        }
        TStmt::ErrDefer { body, span } => {
            o.open(&format!("(errdefer {}", sp(*span)));
            stmt(o, body);
            o.close();
        }
        TStmt::While { cond, body, els, label, span } => {
            o.open(&format!("(while {} label={}", sp(*span), label));
            expr(o, cond);
            block(o, body);
            if let Some(e) = els {
                o.open("(else");
                block(o, e);
                o.close();
            }
            o.close();
        }
        TStmt::ForRange { var, start, end, step, body, label, span } => {
            o.open(&format!("(for_range {} var={} label={}", sp(*span), var, label));
            expr(o, start);
            expr(o, end);
            if let Some(s) = step {
                o.open("(step");
                expr(o, s);
                o.close();
            }
            block(o, body);
            o.close();
        }
        TStmt::ForSlice { items, index, body, label, span, parallel } => {
            let idx = index.map(|i| i.to_string()).unwrap_or_else(|| "-".into());
            o.open(&format!("(for_slice {} index={} label={} parallel={}", sp(*span), idx, label, parallel));
            for (l, e) in items {
                o.open(&format!("(item local={}", l));
                expr(o, e);
                o.close();
            }
            block(o, body);
            o.close();
        }
        TStmt::Block(b) => block(o, b),
        TStmt::Drop { local, span } => o.line(&format!("(drop {} local={})", sp(*span), local)),
        TStmt::Using { strategy, body, span } => {
            o.open(&format!("(using {} {}", sp(*span), strategy));
            block(o, body);
            o.close();
        }
    }
}

fn exprs(o: &mut Out, es: &[TExpr]) {
    for e in es {
        expr(o, e);
    }
}

fn expr(o: &mut Out, e: &TExpr) {
    let t = o.ty(e.ty);
    let head = |name: &str| format!("({} {} :{}", name, sp(e.span), t);
    match &e.kind {
        TExprKind::Int(v) => o.line(&format!("{} {})", head("int"), v)),
        TExprKind::Float(v) => o.line(&format!("{} {:?})", head("float"), v)),
        TExprKind::Bool(v) => o.line(&format!("{} {})", head("bool"), v)),
        TExprKind::Char(v) => o.line(&format!("{} {})", head("char"), v)),
        TExprKind::Str(s) => o.line(&format!("{} {})", head("str"), quote(s))),
        TExprKind::Unit => o.line(&format!("{})", head("unit"))),
        TExprKind::Local(l) => o.line(&format!("{} {})", head("local"), l)),
        TExprKind::Global(d) => o.line(&format!("{} {})", head("global"), d)),
        TExprKind::Const(d) => o.line(&format!("{} {})", head("const"), d)),
        TExprKind::FnRef(i) => o.line(&format!("{} {})", head("fnref"), i)),
        TExprKind::Field { base, idx } => {
            o.open(&format!("{} {}", head("field"), idx));
            expr(o, base);
            o.close();
        }
        TExprKind::RefField { base, idx } => {
            o.open(&format!("{} {}", head("ref_field"), idx));
            expr(o, base);
            o.close();
        }
        TExprKind::TupleField { base, idx } => {
            o.open(&format!("{} {}", head("tuple_field"), idx));
            expr(o, base);
            o.close();
        }
        TExprKind::Index { base, index, proven } => {
            o.open(&format!("{} proven={}", head("index"), proven));
            expr(o, base);
            expr(o, index);
            o.close();
        }
        TExprKind::SliceOp { base, start, end } => {
            o.open(&head("slice"));
            expr(o, base);
            o.line(&format!("(start {})", opt(start)));
            if let Some(s) = start {
                expr(o, s);
            }
            o.line(&format!("(end {})", opt(end)));
            if let Some(s) = end {
                expr(o, s);
            }
            o.close();
        }
        TExprKind::Deref(x) => {
            o.open(&head("deref"));
            expr(o, x);
            o.close();
        }
        TExprKind::AddrOf { expr: x, mutable } => {
            o.open(&format!("{} mut={}", head("addr"), mutable));
            expr(o, x);
            o.close();
        }
        TExprKind::Call { inst, args } => {
            o.open(&format!("{} {}", head("call"), inst));
            exprs(o, args);
            o.close();
        }
        TExprKind::CallPtr { callee, args } => {
            o.open(&head("call_ptr"));
            expr(o, callee);
            exprs(o, args);
            o.close();
        }
        TExprKind::Builtin { op, args, tys } => {
            let tl = o.tys_list(tys);
            o.open(&format!("{} {:?} tys=({})", head("builtin"), op, tl));
            exprs(o, args);
            o.close();
        }
        TExprKind::Unary { op, expr: x, mode: m } => {
            o.open(&format!("{} {} {}", head("unary"), unop(*op), mode(*m)));
            expr(o, x);
            o.close();
        }
        TExprKind::Binary { op, lhs, rhs, mode: m, proven } => {
            o.open(&format!("{} {} {} proven={}", head("binary"), binop(*op), mode(*m), proven));
            expr(o, lhs);
            expr(o, rhs);
            o.close();
        }
        TExprKind::Logical { and, lhs, rhs } => {
            o.open(&format!("{} {}", head("logical"), if *and { "and" } else { "or" }));
            expr(o, lhs);
            expr(o, rhs);
            o.close();
        }
        TExprKind::Cast { expr: x, kind } => {
            o.open(&format!("{} {}", head("cast"), cast(*kind)));
            expr(o, x);
            o.close();
        }
        TExprKind::If { cond, then, els } => {
            o.open(&head("if"));
            expr(o, cond);
            block(o, then);
            if let Some(b) = els {
                o.open("(else");
                block(o, b);
                o.close();
            }
            o.close();
        }
        TExprKind::IfCapture { cond, local, then, els } => {
            o.open(&format!("{} local={}", head("if_let"), local));
            expr(o, cond);
            block(o, then);
            if let Some(b) = els {
                o.open("(else");
                block(o, b);
                o.close();
            }
            o.close();
        }
        TExprKind::Match { scrutinee, arms } => {
            o.open(&head("match"));
            expr(o, scrutinee);
            for a in arms {
                o.open(&format!("(arm {} guard={}", sp(a.span), opt(&a.guard)));
                pat(o, &a.pat);
                if let Some(g) = &a.guard {
                    expr(o, g);
                }
                expr(o, &a.body);
                o.close();
            }
            o.close();
        }
        TExprKind::Block(b) => {
            o.open(&head("block_expr"));
            block(o, b);
            o.close();
        }
        TExprKind::StructLit { fields } => {
            o.open(&head("struct_lit"));
            for (i, f) in fields {
                o.open(&format!("(init {}", i));
                expr(o, f);
                o.close();
            }
            o.close();
        }
        TExprKind::RefNew { fields } => {
            o.open(&head("ref_new"));
            for (i, f) in fields {
                o.open(&format!("(init {}", i));
                expr(o, f);
                o.close();
            }
            o.close();
        }
        TExprKind::EnumLit { variant, payload } => {
            o.open(&format!("{} {}", head("enum_lit"), variant));
            exprs(o, payload);
            o.close();
        }
        TExprKind::ArrayLit(es) => {
            o.open(&head("array_lit"));
            exprs(o, es);
            o.close();
        }
        TExprKind::ArrayRepeat { value, count } => {
            o.open(&format!("{} {}", head("array_repeat"), count));
            expr(o, value);
            o.close();
        }
        TExprKind::TupleLit(es) => {
            o.open(&head("tuple_lit"));
            exprs(o, es);
            o.close();
        }
        TExprKind::Try(x) => {
            o.open(&head("try"));
            expr(o, x);
            o.close();
        }
        TExprKind::Catch { expr: x, err_local, handler } => {
            let l = err_local.map(|l| l.to_string()).unwrap_or_else(|| "-".into());
            o.open(&format!("{} err={}", head("catch"), l));
            expr(o, x);
            expr(o, handler);
            o.close();
        }
        TExprKind::OrElse { expr: x, default } => {
            o.open(&head("orelse"));
            expr(o, x);
            expr(o, default);
            o.close();
        }
        TExprKind::Unwrap { expr: x, proven } => {
            o.open(&format!("{} proven={}", head("unwrap"), proven));
            expr(o, x);
            o.close();
        }
        TExprKind::OptWrap(x) => {
            o.open(&head("opt_wrap"));
            expr(o, x);
            o.close();
        }
        TExprKind::OptNull => o.line(&format!("{})", head("null"))),
        TExprKind::ErrWrap(x) => {
            o.open(&head("err_wrap"));
            expr(o, x);
            o.close();
        }
        TExprKind::ErrToUnion(x) => {
            o.open(&head("err_to_union"));
            expr(o, x);
            o.close();
        }
        TExprKind::ErrVal(v) => o.line(&format!("{} {})", head("err_val"), v)),
        TExprKind::ArrayToSlice(x) => {
            o.open(&head("array_to_slice"));
            expr(o, x);
            o.close();
        }
        TExprKind::ListToSlice(x) => {
            o.open(&head("list_to_slice"));
            expr(o, x);
            o.close();
        }
        TExprKind::StrToSlice(x) => {
            o.open(&head("str_to_slice"));
            expr(o, x);
            o.close();
        }
        TExprKind::Closure { inst, captures } => {
            let caps = captures.iter().map(|(l, r)| format!("({} ref={})", l, r)).collect::<Vec<_>>().join(" ");
            o.line(&format!("{} {} captures=({}))", head("closure"), inst, caps));
        }
        TExprKind::FnToFat(i) => o.line(&format!("{} {})", head("fn_to_fat"), i)),
        TExprKind::Unreachable => o.line(&format!("{})", head("unreachable"))),
        TExprKind::Undefined => o.line(&format!("{})", head("undefined"))),
        TExprKind::BinConstruct { segments, target } => {
            o.open(&head("bin_construct"));
            for s in segments {
                seg(o, s);
            }
            o.open("(into");
            expr(o, target);
            o.close();
            o.close();
        }
        TExprKind::Value(v) => o.line(&format!("{} {})", head("value"), value(v))),
        TExprKind::TypeVal(t) => {
            let n = o.ty(*t);
            o.line(&format!("{} {})", head("type_val"), n));
        }
        TExprKind::RecordCheck { value: v, checks, as_error } => {
            o.open(&format!("{} as_error={}", head("record_check"), as_error));
            expr(o, v);
            for (f, c, l) in checks {
                o.open(&format!("(check field={} local={}", f, l));
                expr(o, c);
                o.close();
            }
            o.close();
        }
        TExprKind::Retained(x) => {
            o.open(&head("retained"));
            expr(o, x);
            o.close();
        }
        TExprKind::DynFrom { expr: x, vtable } => {
            o.open(&format!("{} vtable={}", head("dyn_from"), vtable));
            expr(o, x);
            o.close();
        }
        TExprKind::DynCall { recv, method, args } => {
            o.open(&format!("{} method={}", head("dyn_call"), method));
            expr(o, recv);
            exprs(o, args);
            o.close();
        }
    }
}

fn value(v: &Value) -> String {
    match v {
        Value::Int(i) => format!("(int {})", i),
        Value::Float(f) => format!("(float {:?})", f),
        Value::Bool(b) => format!("(bool {})", b),
        Value::Char(c) => format!("(char {})", c),
        Value::Str(s) | Value::OwnedStr(s) => format!("(str {})", quote(s)),
        Value::Void => "(void)".into(),
        Value::Array(a) | Value::List(a) | Value::Tuple(a) | Value::Struct(a) => format!("(seq {})", a.iter().map(value).collect::<Vec<_>>().join(" ")),
        Value::Enum(i, a) => format!("(enum {} {})", i, a.iter().map(value).collect::<Vec<_>>().join(" ")),
        Value::Opt(None) => "(null)".into(),
        Value::Opt(Some(b)) => format!("(some {})", value(b)),
        Value::Ok(b) => format!("(ok {})", value(b)),
        Value::Err(e) => format!("(err {})", e),
        Value::Fn(i) => format!("(fn {})", i),
        Value::Type(_) => "(type)".into(),
        Value::Map(ps) => format!("(map {})", ps.iter().map(|(k, v)| format!("{} {}", value(k), value(v))).collect::<Vec<_>>().join(" ")),
        Value::Undefined => "(undefined)".into(),
        Value::Never => "(never)".into(),
        Value::Ptr(..) => "(ptr)".into(),
    }
}

fn pat(o: &mut Out, p: &TPat) {
    match p {
        TPat::Wild => o.line("(pat _)"),
        TPat::Bind(l) => o.line(&format!("(pat bind {})", l)),
        TPat::Int(v) => o.line(&format!("(pat int {})", v)),
        TPat::Float(v) => o.line(&format!("(pat float {:?})", v)),
        TPat::Bool(v) => o.line(&format!("(pat bool {})", v)),
        TPat::Str(s) => o.line(&format!("(pat str {})", quote(s))),
        TPat::Char(c) => o.line(&format!("(pat char {})", c)),
        TPat::Range { lo, hi, inclusive } => o.line(&format!("(pat range {} {} inclusive={})", lo, hi, inclusive)),
        TPat::Variant { idx, args } => {
            o.open(&format!("(pat variant {}", idx));
            for a in args {
                pat(o, a);
            }
            o.close();
        }
        TPat::Error(e) => o.line(&format!("(pat error {})", e)),
        TPat::Null => o.line("(pat null)"),
        TPat::Some(b) => {
            o.open("(pat some");
            pat(o, b);
            o.close();
        }
        TPat::Ok(b) => {
            o.open("(pat ok");
            pat(o, b);
            o.close();
        }
        TPat::Or(ps) => {
            o.open("(pat or");
            for a in ps {
                pat(o, a);
            }
            o.close();
        }
        TPat::Tuple(ps) => {
            o.open("(pat tuple");
            for a in ps {
                pat(o, a);
            }
            o.close();
        }
        TPat::Binary(segs) => {
            o.open("(pat binary");
            for s in segs {
                seg(o, s);
            }
            o.close();
        }
    }
}

fn seg(o: &mut Out, s: &TBinSeg) {
    let t = o.ty(s.ty);
    let kind = match &s.kind {
        TBinSegKind::Bind(l) => format!("bind {}", l),
        TBinSegKind::Rest(l) => format!("rest {}", l),
        TBinSegKind::Value(_) => "value".into(),
    };
    let size = match &s.size {
        TBinSize::Bits(n) => format!("bits {}", n),
        TBinSize::Expr(_) => "expr".into(),
        TBinSize::Rest => "rest".into(),
    };
    let endian = match s.endian {
        Endian::Big => "big",
        Endian::Little => "little",
        Endian::Native => "native",
    };
    o.open(&format!("(seg {} :{} {} size={} endian={} signed={} float={} utf8={}", sp(s.span), t, kind, size, endian, s.signed, s.float, s.utf8));
    if let TBinSegKind::Value(e) = &s.kind {
        expr(o, e);
    }
    if let TBinSize::Expr(e) = &s.size {
        expr(o, e);
    }
    o.close();
}
