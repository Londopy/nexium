//! Method calls, builtin functions, namespaces, and static members.

use super::*;
use crate::ast::*;
use crate::tir::*;
use crate::types::*;

fn peel_ptr(c: &Checker, mut t: TyId) -> TyId {
    t = c.tys.shallow(t);
    while let TyKind::Ptr(_, inner) = c.tys.kind(t).clone() {
        t = c.tys.shallow(inner);
    }
    t
}

impl<'a> Checker<'a> {
    fn check_args_n(&mut self, args: &[Expr], n: usize, what: &str, span: Span) -> bool {
        if args.len() != n {
            self.error(span, format!("`{}` takes {} argument(s) but {} were given", what, n, args.len()));
            for a in args {
                let _ = self.check_expr(a, None);
            }
            return false;
        }
        true
    }

    fn arg(&mut self, e: &Expr, ty: TyId, what: &str) -> TExpr {
        let te = self.check_expr(e, Some(ty));
        self.coerce_or_error(te, ty, what)
    }

    fn builtin(&mut self, op: Builtin, args: Vec<TExpr>, tys: Vec<TyId>, ty: TyId, span: Span) -> TExpr {
        self.mk(TExprKind::Builtin { op, args, tys }, ty, span)
    }

    /// Parse a format string and check placeholders against argument types.
    fn check_format(&mut self, fmt: &Expr, args: Option<&Expr>, span: Span) -> Option<(Vec<u8>, Vec<TExpr>)> {
        let fmt_bytes = match fmt {
            Expr::Lit { value: Lit::Str(s), .. } => s.clone(),
            _ => {
                self.error(fmt.span(), "the format string must be a string literal so it can be checked at compile time (section 7.4)");
                return None;
            }
        };
        // count placeholders
        let mut n = 0;
        let mut i = 0;
        let b = &fmt_bytes;
        let mut specs = Vec::new();
        while i < b.len() {
            if b[i] == b'{' {
                if i + 1 < b.len() && b[i + 1] == b'{' {
                    i += 2;
                    continue;
                }
                let start = i + 1;
                let mut j = start;
                while j < b.len() && b[j] != b'}' {
                    j += 1;
                }
                if j >= b.len() {
                    self.error(fmt.span(), "unterminated `{` in format string (write `{{` for a literal brace)");
                    return None;
                }
                specs.push(String::from_utf8_lossy(&b[start..j]).to_string());
                n += 1;
                i = j + 1;
            } else if b[i] == b'}' {
                if i + 1 < b.len() && b[i + 1] == b'}' {
                    i += 2;
                    continue;
                }
                self.error(fmt.span(), "unmatched `}` in format string (write `}}` for a literal brace)");
                return None;
            } else {
                i += 1;
            }
        }
        let arg_exprs: Vec<Expr> = match args {
            None => vec![],
            Some(Expr::TupleLit { elems, .. }) => elems.clone(),
            Some(Expr::StructLit { ty: None, fields, .. }) if fields.is_empty() => vec![],
            Some(other) => vec![other.clone()],
        };
        if arg_exprs.len() != n {
            self.error(span, format!("format string has {} placeholder(s) but {} argument(s) were given", n, arg_exprs.len()));
            return None;
        }
        let mut out = Vec::new();
        for (k, a) in arg_exprs.iter().enumerate() {
            let te = self.check_expr(a, None);
            let te = self.finalize_expr(te);
            let t = self.tys.resolve(te.ty, true);
            let mut te = te;
            let ok = match self.tys.kind(t).clone() {
                TyKind::Int(_) | TyKind::Float(_) | TyKind::Bool | TyKind::Char | TyKind::ErrorSet(_) => true,
                TyKind::Slice(_, e) => matches!(self.tys.kind(self.tys.shallow(e)), TyKind::Int(IntTy::U8)),
                TyKind::Str => {
                    let u8t = self.tys.u8();
                    let sl = self.tys.slice(false, u8t);
                    let sp = te.span;
                    te = TExpr { kind: TExprKind::StrToSlice(Box::new(te)), ty: sl, span: sp };
                    true
                }
                TyKind::Enum(d, _) => self.enums[d as usize].variants.iter().all(|v| matches!(v.payload, VariantPayloadDef::Unit)),
                TyKind::Distinct(d) => {
                    let u = self.distinct_underlying[&d];
                    let sp = te.span;
                    te = TExpr { kind: TExprKind::Cast { expr: Box::new(te), kind: CastKind::Bits }, ty: u, span: sp };
                    true
                }
                TyKind::Opt(inner) => matches!(self.tys.kind(self.tys.shallow(inner)), TyKind::Int(_) | TyKind::Float(_) | TyKind::Bool | TyKind::Char),
                TyKind::Ptr(..) => true,
                _ => false,
            };
            if !ok {
                let tn = self.type_name(t);
                self.error(a.span(), format!("cannot format a value of type `{}`; formatting supports numbers, bool, char, `[]u8`, `String`, unit enums, and errors", tn));
            }
            let spec = specs[k].trim_start_matches(':').to_string();
            let spec = &spec;
            if !spec.is_empty() {
                let valid = match spec.as_str() {
                    "x" | "X" | "b" | "o" | "e" | "c" => true,
                    s if s.starts_with('.') && s[1..].parse::<u32>().is_ok() => true,
                    s if s.starts_with('>') || s.starts_with('<') => true,
                    _ => false,
                };
                if !valid {
                    self.error(fmt.span(), format!("unknown format spec `{{{}}}`; supported: `{{}}`, `{{x}}`, `{{X}}`, `{{b}}`, `{{o}}`, `{{e}}`, `{{c}}`, `{{.N}}`, `{{>N}}`, `{{<N}}`", spec));
                }
            }
            out.push(te);
        }
        Some((fmt_bytes, out))
    }

    /// Builtin free functions: println, print, format, expect, ...
    pub fn check_builtin_fn(&mut self, name: &str, args: &[Expr], _expected: Option<TyId>, span: Span) -> Option<TExpr> {
        let void = self.tys.void();
        let u8t = self.tys.u8();
        let bytes = self.tys.slice(false, u8t);
        match name {
            "println" | "print" | "eprintln" | "format" => {
                if args.is_empty() || args.len() > 2 {
                    self.error(span, format!("`{}` takes a format string and an optional `.{{ }}` argument tuple", name));
                    return Some(self.error_expr(span));
                }
                let (fmt, fargs) = match self.check_format(&args[0], args.get(1), span) {
                    Some(x) => x,
                    None => return Some(self.error_expr(span)),
                };
                let mut all = vec![TExpr { kind: TExprKind::Str(fmt), ty: bytes, span: args[0].span() }];
                all.extend(fargs);
                let (op, ty) = match name {
                    "println" => (Builtin::Println, void),
                    "print" => (Builtin::Print, void),
                    "eprintln" => (Builtin::Eprintln, void),
                    _ => (Builtin::Format, self.tys.string()),
                };
                if op == Builtin::Format {
                    self.add_effect(Effects::ALLOCATES, span, "`format` builds an owned String");
                } else {
                    self.add_effect(Effects::BLOCKS, span, "writing to stdout may block");
                }
                Some(self.builtin(op, all, vec![], ty, span))
            }
            "expect" | "assert" => {
                if !self.check_args_n(args, 1, name, span) {
                    return Some(self.error_expr(span));
                }
                let bt = self.tys.bool();
                let c = self.arg(&args[0], bt, "condition");
                self.add_effect(Effects::PANICS, span, "a failed `expect` panics");
                let src = self.sm.file(span.file).map(|f| f.text[args[0].span().start as usize..args[0].span().end as usize].to_string()).unwrap_or_default();
                let msg = TExpr { kind: TExprKind::Str(src.into_bytes()), ty: bytes, span };
                Some(self.builtin(Builtin::Expect, vec![c, msg], vec![], void, span))
            }
            "expect_eq" => {
                if !self.check_args_n(args, 2, name, span) {
                    return Some(self.error_expr(span));
                }
                let fake = Expr::Binary { op: BinOp::Eq, lhs: Box::new(args[0].clone()), rhs: Box::new(args[1].clone()), span };
                let bt = self.tys.bool();
                let c = self.check_expr(&fake, Some(bt));
                self.add_effect(Effects::PANICS, span, "a failed `expect_eq` panics");
                let src = self.sm.file(span.file).map(|f| f.text[span.start as usize..span.end as usize].to_string()).unwrap_or_default();
                let msg = TExpr { kind: TExprKind::Str(src.into_bytes()), ty: bytes, span };
                Some(self.builtin(Builtin::Expect, vec![c, msg], vec![], void, span))
            }
            "panic" => {
                if !self.check_args_n(args, 1, name, span) {
                    return Some(self.error_expr(span));
                }
                let m = self.arg(&args[0], bytes, "panic message");
                self.add_effect(Effects::PANICS, span, "explicit panic");
                let never = self.tys.never();
                Some(self.builtin(Builtin::Panic, vec![m], vec![], never, span))
            }
            _ => None,
        }
    }

    /// `@name(args)` builtins.
    pub fn check_at_builtin(&mut self, name: &str, args: &[Expr], _expected: Option<TyId>, span: Span) -> TExpr {
        let u8t = self.tys.u8();
        let bytes = self.tys.slice(false, u8t);
        match name {
            "typeName" => {
                if !self.check_args_n(args, 1, "@typeName", span) {
                    return self.error_expr(span);
                }
                let te = self.check_expr(&args[0], None);
                let t = match te.kind {
                    TExprKind::TypeVal(t) => t,
                    _ => te.ty,
                };
                let t = self.tys.resolve(t, true);
                let n = self.type_name(t);
                self.mk(TExprKind::Str(n.into_bytes()), bytes, span)
            }
            "sizeOf" => {
                if !self.check_args_n(args, 1, "@sizeOf", span) {
                    return self.error_expr(span);
                }
                let te = self.check_expr(&args[0], None);
                let t = match te.kind {
                    TExprKind::TypeVal(t) => t,
                    _ => te.ty,
                };
                let t = self.tys.resolve(t, true);
                let s = self.size_of(t);
                let u = self.tys.usize();
                self.mk(TExprKind::Int(s as i128), u, span)
            }
            "truncate" => {
                if !self.check_args_n(args, 2, "@truncate", span) {
                    return self.error_expr(span);
                }
                let te = self.check_expr(&args[0], None);
                let t = match te.kind {
                    TExprKind::TypeVal(t) => t,
                    _ => {
                        self.error(args[0].span(), "the first argument of @truncate is the target type");
                        return self.error_expr(span);
                    }
                };
                let v = self.check_expr(&args[1], None);
                if !self.tys.is_integer(t) || !self.tys.is_integer(v.ty) {
                    self.error(span, "@truncate converts between integer types");
                }
                let vt = self.tys.resolve(v.ty, true);
                let v = TExpr { ty: vt, ..v };
                self.builtin(Builtin::Truncate, vec![v], vec![t], t, span)
            }
            "errorName" => {
                if !self.check_args_n(args, 1, "@errorName", span) {
                    return self.error_expr(span);
                }
                let e = self.check_expr(&args[0], None);
                let t = self.tys.shallow(e.ty);
                if !matches!(self.tys.kind(t), TyKind::ErrorSet(_)) {
                    let tn = self.type_name(t);
                    self.error(span, format!("@errorName takes an error value but found `{}`", tn));
                }
                self.builtin(Builtin::ErrorName, vec![e], vec![], bytes, span)
            }
            "embedFile" => {
                if !self.check_args_n(args, 1, "@embedFile", span) {
                    return self.error_expr(span);
                }
                let path = match &args[0] {
                    Expr::Lit { value: Lit::Str(s), .. } => String::from_utf8_lossy(s).to_string(),
                    _ => {
                        self.error(args[0].span(), "@embedFile takes a string literal path (a declared build input)");
                        return self.error_expr(span);
                    }
                };
                let module = self.cur().module;
                let dir = self.source_dirs.get(module as usize).cloned().unwrap_or_else(|| ".".into());
                let full = std::path::Path::new(&dir).join(&path);
                match std::fs::read(&full) {
                    Ok(data) => self.mk(TExprKind::Str(data), bytes, span),
                    Err(e) => {
                        self.error(span, format!("@embedFile: cannot read `{}`: {}", full.display(), e));
                        self.error_expr(span)
                    }
                }
            }
            "cstr" => {
                if !self.check_args_n(args, 1, "@cstr", span) {
                    return self.error_expr(span);
                }
                match &args[0] {
                    Expr::Lit { value: Lit::Str(s), .. } => {
                        let pt = self.tys.ptr(false, u8t);
                        let lit = self.mk(TExprKind::Str(s.clone()), bytes, span);
                        self.builtin(Builtin::PtrAdd, vec![lit], vec![], pt, span)
                    }
                    _ => {
                        self.error(args[0].span(), "@cstr takes a string literal and yields a NUL-terminated `*u8` for C");
                        self.error_expr(span)
                    }
                }
            }
            "weak" => {
                if !self.check_args_n(args, 1, "@weak", span) {
                    return self.error_expr(span);
                }
                let e = self.check_expr(&args[0], None);
                let t = self.tys.resolve(e.ty, true);
                if !self.is_ref_class(t) {
                    let tn = self.type_name(t);
                    self.error(span, format!("@weak takes a `ref class` value but found `{}`", tn));
                    return self.error_expr(span);
                }
                let wt = self.tys.intern(TyKind::Weak(t));
                self.add_effect(Effects::REFCOUNTS, span, "creating a weak reference touches the count");
                self.builtin(Builtin::Weak, vec![e], vec![t], wt, span)
            }
            "refCount" => {
                if !self.check_args_n(args, 1, "@refCount", span) {
                    return self.error_expr(span);
                }
                let e = self.check_expr(&args[0], None);
                let t = self.tys.resolve(e.ty, true);
                if !self.is_ref_class(t) {
                    self.error(span, "@refCount takes a `ref class` value");
                    return self.error_expr(span);
                }
                let u = self.tys.usize();
                self.builtin(Builtin::RefCount, vec![e], vec![], u, span)
            }
            _ => {
                self.error(span, format!("unknown builtin `@{}`; available: @typeName, @sizeOf, @truncate, @errorName, @embedFile, @weak, @refCount", name));
                for a in args {
                    let _ = self.check_expr(a, None);
                }
                self.error_expr(span)
            }
        }
    }

    pub fn check_namespace_member(&mut self, ns: &str, name: &str, span: Span, _expected: Option<TyId>) -> TExpr {
        let f64t = self.tys.float(FloatTy::F64);
        match (ns, name) {
            ("math", "PI") => self.mk(TExprKind::Float(std::f64::consts::PI), f64t, span),
            ("math", "E") => self.mk(TExprKind::Float(std::f64::consts::E), f64t, span),
            ("math", "TAU") => self.mk(TExprKind::Float(std::f64::consts::TAU), f64t, span),
            ("math", "INF") => self.mk(TExprKind::Float(f64::INFINITY), f64t, span),
            ("math", "NAN") => self.mk(TExprKind::Float(f64::NAN), f64t, span),
            _ if ns.starts_with("module:") => {
                let m: u32 = ns[7..].parse().unwrap();
                let saved_mod = self.cur().module;
                // resolve the item in the other module (must be pub)
                match self.items.get(&(m, name.to_string())).cloned() {
                    Some(ItemRef::Fn(fid)) => {
                        if !self.fns[fid as usize].decl.attrs.is_pub {
                            self.error(span, format!("function `{}` is private to its module", name));
                        }
                        self.cur().module = m;
                        let te = self.check_ident(name, span);
                        self.cur().module = saved_mod;
                        te
                    }
                    Some(_) => {
                        self.cur().module = m;
                        let te = self.check_ident(name, span);
                        self.cur().module = saved_mod;
                        te
                    }
                    None => {
                        let mname = self.module_names[m as usize].clone();
                        match self.cimport_unsupported.get(&(m, name.to_string())).cloned() {
                            Some(why) => self.error(span, format!("`{}` was imported from `{}` but cannot be used: {}", name, mname.trim_start_matches("cimport:"), why)),
                            None => self.error(span, format!("module `{}` has no item `{}`", mname, name)),
                        }
                        self.error_expr(span)
                    }
                }
            }
            _ => {
                self.error(span, format!("`{}` has no member `{}`", ns, name));
                self.error_expr(span)
            }
        }
    }

    pub fn check_namespace_call(&mut self, ns: &str, name: &str, args: &[Expr], expected: Option<TyId>, span: Span) -> TExpr {
        if let Some(rest) = ns.strip_prefix("module:") {
            let m: u32 = rest.parse().unwrap();
            match self.items.get(&(m, name.to_string())).cloned() {
                Some(ItemRef::Fn(fid)) => {
                    if !self.fns[fid as usize].decl.attrs.is_pub {
                        self.error(span, format!("function `{}` is private to its module", name));
                    }
                    return self.call_fn_def(fid, None, args, expected, span);
                }
                Some(ItemRef::Struct(_)) | Some(ItemRef::Enum(_)) => {
                    let saved = self.cur().module;
                    self.cur().module = m;
                    let tv = self.check_ident(name, span);
                    self.cur().module = saved;
                    if let TExprKind::TypeVal(t) = tv.kind {
                        // generic instantiation across modules is not supported; treat as type value call error
                        let tn = self.type_name(t);
                        self.error(span, format!("`{}` is a type; construct it with `{}{{ ... }}`", tn, tn));
                    }
                    return self.error_expr(span);
                }
                _ => {
                    let mname = self.module_names[m as usize].clone();
                    match self.cimport_unsupported.get(&(m, name.to_string())).cloned() {
                        Some(why) => self.error(span, format!("`{}` was imported from `{}` but cannot be called: {}", name, mname.trim_start_matches("cimport:"), why)),
                        None => self.error(span, format!("module `{}` has no function `{}`", mname, name)),
                    }
                    for a in args {
                        let _ = self.check_expr(a, None);
                    }
                    return self.error_expr(span);
                }
            }
        }
        let void = self.tys.void();
        let u8t = self.tys.u8();
        let bytes = self.tys.slice(false, u8t);
        let f64t = self.tys.float(FloatTy::F64);
        let i64t = self.tys.int(IntTy::I64);
        let u64t = self.tys.int(IntTy::U64);
        let string = self.tys.string();
        match ns {
            "math" => {
                let unary = |op: Builtin| Some(op);
                let op = match name {
                    "sqrt" => unary(Builtin::MathSqrt),
                    "abs" => unary(Builtin::MathAbs),
                    "floor" => unary(Builtin::MathFloor),
                    "ceil" => unary(Builtin::MathCeil),
                    "round" => unary(Builtin::MathRound),
                    "sin" => unary(Builtin::MathSin),
                    "cos" => unary(Builtin::MathCos),
                    "tan" => unary(Builtin::MathTan),
                    "exp" => unary(Builtin::MathExp),
                    "log" => unary(Builtin::MathLog),
                    "log2" => unary(Builtin::MathLog2),
                    _ => None,
                };
                if let Some(op) = op {
                    if !self.check_args_n(args, 1, &format!("math.{}", name), span) {
                        return self.error_expr(span);
                    }
                    let x = self.check_expr(&args[0], expected);
                    let t = self.tys.resolve(x.ty, true);
                    if op == Builtin::MathAbs {
                        if !self.tys.is_numeric(t) {
                            self.error(span, "math.abs takes a number");
                        }
                        return self.builtin(op, vec![x], vec![t], t, span);
                    }
                    if !self.tys.is_float(t) {
                        let tn = self.type_name(t);
                        self.error(span, format!("math.{} takes a float but found `{}`; convert with `as f64`", name, tn));
                    }
                    return self.builtin(op, vec![x], vec![t], t, span);
                }
                let binary = match name {
                    "min" => Some(Builtin::MathMin),
                    "max" => Some(Builtin::MathMax),
                    "pow" => Some(Builtin::MathPow),
                    "atan2" => Some(Builtin::MathAtan2),
                    _ => None,
                };
                if let Some(op) = binary {
                    if !self.check_args_n(args, 2, &format!("math.{}", name), span) {
                        return self.error_expr(span);
                    }
                    let a = self.check_expr(&args[0], expected);
                    let at = self.tys.resolve(a.ty, false);
                    let b = self.arg(&args[1], at, "second operand");
                    let t = self.tys.resolve(at, true);
                    if !self.tys.is_numeric(t) {
                        self.error(span, format!("math.{} takes numbers", name));
                    }
                    if matches!(op, Builtin::MathPow | Builtin::MathAtan2) && !self.tys.is_float(t) {
                        self.error(span, format!("math.{} takes floats", name));
                    }
                    return self.builtin(op, vec![a, b], vec![t], t, span);
                }
                if name == "clamp" {
                    if !self.check_args_n(args, 3, "math.clamp", span) {
                        return self.error_expr(span);
                    }
                    let a = self.check_expr(&args[0], expected);
                    let at = self.tys.resolve(a.ty, false);
                    let lo = self.arg(&args[1], at, "lower bound");
                    let hi = self.arg(&args[2], at, "upper bound");
                    let t = self.tys.resolve(at, true);
                    return self.builtin(Builtin::MathClamp, vec![a, lo, hi], vec![t], t, span);
                }
                self.error(span, format!("`math` has no function `{}`", name));
                self.error_expr(span)
            }
            "io" => match name {
                "read_file" => {
                    if !self.check_args_n(args, 1, "io.read_file", span) {
                        return self.error_expr(span);
                    }
                    let p = self.arg(&args[0], bytes, "path");
                    self.add_effect(Effects::BLOCKS, span, "file I/O blocks");
                    self.add_effect(Effects::ALLOCATES, span, "reading a file allocates its contents");
                    let r = self.tys.err_union(None, string);
                    self.builtin(Builtin::ReadFile, vec![p], vec![], r, span)
                }
                "write_file" => {
                    if !self.check_args_n(args, 2, "io.write_file", span) {
                        return self.error_expr(span);
                    }
                    let p = self.arg(&args[0], bytes, "path");
                    let d = self.arg(&args[1], bytes, "data");
                    self.add_effect(Effects::BLOCKS, span, "file I/O blocks");
                    let r = self.tys.err_union(None, void);
                    self.builtin(Builtin::WriteFile, vec![p, d], vec![], r, span)
                }
                "read_line" => {
                    if !self.check_args_n(args, 0, "io.read_line", span) {
                        return self.error_expr(span);
                    }
                    self.add_effect(Effects::BLOCKS, span, "reading stdin blocks");
                    self.add_effect(Effects::ALLOCATES, span, "reading a line allocates");
                    let o = self.tys.opt(string);
                    self.builtin(Builtin::ReadLine, vec![], vec![], o, span)
                }
                "append_file" => {
                    if !self.check_args_n(args, 2, "io.append_file", span) {
                        return self.error_expr(span);
                    }
                    let p = self.arg(&args[0], bytes, "path");
                    let d = self.arg(&args[1], bytes, "data");
                    self.add_effect(Effects::BLOCKS, span, "file I/O blocks");
                    let r = self.tys.err_union(None, void);
                    self.builtin(Builtin::AppendFile, vec![p, d], vec![], r, span)
                }
                "file_kind" => {
                    if !self.check_args_n(args, 1, "io.file_kind", span) {
                        return self.error_expr(span);
                    }
                    let p = self.arg(&args[0], bytes, "path");
                    self.add_effect(Effects::BLOCKS, span, "file I/O blocks");
                    let t = self.tys.int(IntTy::I32);
                    self.builtin(Builtin::FsKind, vec![p], vec![], t, span)
                }
                "file_size" | "file_modified" => {
                    if !self.check_args_n(args, 1, &format!("io.{}", name), span) {
                        return self.error_expr(span);
                    }
                    let p = self.arg(&args[0], bytes, "path");
                    self.add_effect(Effects::BLOCKS, span, "file I/O blocks");
                    let (op, t) = if name == "file_size" { (Builtin::FsSize, self.tys.int(IntTy::U64)) } else { (Builtin::FsModified, self.tys.int(IntTy::I64)) };
                    let r = self.tys.err_union(None, t);
                    self.builtin(op, vec![p], vec![], r, span)
                }
                "make_dir" | "remove_file" | "remove_dir" => {
                    if !self.check_args_n(args, 1, &format!("io.{}", name), span) {
                        return self.error_expr(span);
                    }
                    let p = self.arg(&args[0], bytes, "path");
                    self.add_effect(Effects::BLOCKS, span, "file I/O blocks");
                    let op = match name {
                        "make_dir" => Builtin::FsMkdir,
                        "remove_file" => Builtin::FsRemoveFile,
                        _ => Builtin::FsRemoveDir,
                    };
                    let r = self.tys.err_union(None, void);
                    self.builtin(op, vec![p], vec![], r, span)
                }
                "rename" => {
                    if !self.check_args_n(args, 2, "io.rename", span) {
                        return self.error_expr(span);
                    }
                    let a = self.arg(&args[0], bytes, "path");
                    let b = self.arg(&args[1], bytes, "new path");
                    self.add_effect(Effects::BLOCKS, span, "file I/O blocks");
                    let r = self.tys.err_union(None, void);
                    self.builtin(Builtin::FsRename, vec![a, b], vec![], r, span)
                }
                "list_dir" => {
                    if !self.check_args_n(args, 1, "io.list_dir", span) {
                        return self.error_expr(span);
                    }
                    let p = self.arg(&args[0], bytes, "path");
                    self.add_effect(Effects::BLOCKS, span, "file I/O blocks");
                    self.add_effect(Effects::ALLOCATES, span, "listing a directory allocates the names");
                    let l = self.tys.list(string);
                    let r = self.tys.err_union(None, l);
                    self.builtin(Builtin::FsListDir, vec![p], vec![], r, span)
                }
                "cwd" => {
                    if !self.check_args_n(args, 0, "io.cwd", span) {
                        return self.error_expr(span);
                    }
                    self.add_effect(Effects::ALLOCATES, span, "the working directory is copied into a String");
                    let r = self.tys.err_union(None, string);
                    self.builtin(Builtin::FsCwd, vec![], vec![], r, span)
                }
                "temp_dir" => {
                    if !self.check_args_n(args, 0, "io.temp_dir", span) {
                        return self.error_expr(span);
                    }
                    self.add_effect(Effects::ALLOCATES, span, "the directory name is copied into a String");
                    self.builtin(Builtin::FsTempDir, vec![], vec![], string, span)
                }
                _ => {
                    self.error(span, format!("`io` has no function `{}`; available: read_file, write_file, append_file, read_line, file_kind, file_size, file_modified, make_dir, remove_file, remove_dir, rename, list_dir, cwd, temp_dir", name));
                    self.error_expr(span)
                }
            },
            "os" | "process" => match name {
                "args" => {
                    if !self.check_args_n(args, 0, "os.args", span) {
                        return self.error_expr(span);
                    }
                    let t = self.tys.slice(false, bytes);
                    self.builtin(Builtin::Args, vec![], vec![], t, span)
                }
                "env" => {
                    if !self.check_args_n(args, 1, "os.env", span) {
                        return self.error_expr(span);
                    }
                    let n = self.arg(&args[0], bytes, "variable name");
                    self.add_effect(Effects::NONDETERMINISTIC, span, "the environment varies across runs");
                    let o = self.tys.opt(bytes);
                    self.builtin(Builtin::Env, vec![n], vec![], o, span)
                }
                "exit" => {
                    if !self.check_args_n(args, 1, "os.exit", span) {
                        return self.error_expr(span);
                    }
                    let i32t = self.tys.int(IntTy::I32);
                    let c = self.arg(&args[0], i32t, "exit code");
                    let never = self.tys.never();
                    self.builtin(Builtin::Exit, vec![c], vec![], never, span)
                }
                "run" => {
                    if !self.check_args_n(args, 1, "process.run", span) {
                        return self.error_expr(span);
                    }
                    let argv_t = self.tys.slice(false, bytes);
                    let argv = self.arg(&args[0], argv_t, "command and arguments");
                    self.add_effect(Effects::BLOCKS, span, "running a process waits for it to finish");
                    self.add_effect(Effects::NONDETERMINISTIC, span, "a child process can do anything");
                    let i32t = self.tys.int(IntTy::I32);
                    let r = self.tys.err_union(None, i32t);
                    self.builtin(Builtin::Run, vec![argv], vec![], r, span)
                }
                _ => {
                    self.error(span, format!("`{}` has no function `{}`; available: args, env, exit, run", ns, name));
                    self.error_expr(span)
                }
            },
            "time" => match name {
                "now" => {
                    if !self.check_args_n(args, 0, "time.now", span) {
                        return self.error_expr(span);
                    }
                    self.add_effect(Effects::NONDETERMINISTIC, span, "reading the clock");
                    self.builtin(Builtin::TimeNow, vec![], vec![], i64t, span)
                }
                "monotonic" => {
                    if !self.check_args_n(args, 0, "time.monotonic", span) {
                        return self.error_expr(span);
                    }
                    self.add_effect(Effects::NONDETERMINISTIC, span, "reading the clock");
                    self.builtin(Builtin::TimeMonotonic, vec![], vec![], u64t, span)
                }
                "sleep" => {
                    if !self.check_args_n(args, 1, "time.sleep", span) {
                        return self.error_expr(span);
                    }
                    let ms = self.arg(&args[0], u64t, "milliseconds");
                    self.add_effect(Effects::BLOCKS, span, "sleeping blocks");
                    self.builtin(Builtin::Sleep, vec![ms], vec![], void, span)
                }
                "utc_offset" => {
                    if !self.check_args_n(args, 1, "time.utc_offset", span) {
                        return self.error_expr(span);
                    }
                    let ms = self.arg(&args[0], i64t, "milliseconds since the epoch");
                    self.add_effect(Effects::NONDETERMINISTIC, span, "the local time zone varies across machines");
                    self.builtin(Builtin::TimeUtcOffset, vec![ms], vec![], i64t, span)
                }
                _ => {
                    self.error(span, format!("`time` has no function `{}`; available: now, monotonic, sleep, utc_offset", name));
                    self.error_expr(span)
                }
            },
            "random" => match name {
                "int" => {
                    if !self.check_args_n(args, 2, "random.int", span) {
                        return self.error_expr(span);
                    }
                    let lo = self.arg(&args[0], i64t, "lower bound");
                    let hi = self.arg(&args[1], i64t, "upper bound");
                    self.add_effect(Effects::NONDETERMINISTIC, span, "random numbers");
                    self.builtin(Builtin::RandomInt, vec![lo, hi], vec![], i64t, span)
                }
                "float" => {
                    if !self.check_args_n(args, 0, "random.float", span) {
                        return self.error_expr(span);
                    }
                    self.add_effect(Effects::NONDETERMINISTIC, span, "random numbers");
                    self.builtin(Builtin::RandomFloat, vec![], vec![], f64t, span)
                }
                "seed" => {
                    if !self.check_args_n(args, 1, "random.seed", span) {
                        return self.error_expr(span);
                    }
                    let s = self.arg(&args[0], u64t, "seed");
                    self.builtin(Builtin::RandomSeed, vec![s], vec![], void, span)
                }
                _ => {
                    self.error(span, format!("`random` has no function `{}`; available: int, float, seed", name));
                    self.error_expr(span)
                }
            },
            "mem" => match name {
                "copy" => {
                    if !self.check_args_n(args, 2, "mem.copy", span) {
                        return self.error_expr(span);
                    }
                    let d = self.check_expr(&args[0], None);
                    let dt = self.tys.resolve(d.ty, true);
                    let (m, e) = match self.tys.kind(dt).clone() {
                        TyKind::Slice(m, e) => (m, e),
                        _ => {
                            self.error(span, "mem.copy takes a `[]mut T` destination");
                            return self.error_expr(span);
                        }
                    };
                    if !m {
                        self.error(span, "mem.copy destination must be `[]mut T`");
                    }
                    let st = self.tys.slice(false, e);
                    let s = self.arg(&args[1], st, "source");
                    self.add_effect(Effects::PANICS, span, "mem.copy panics when the source is longer than the destination");
                    self.builtin(Builtin::SliceCopy, vec![d, s], vec![e], void, span)
                }
                _ => {
                    self.error(span, format!("`mem` has no function `{}`", name));
                    self.error_expr(span)
                }
            },
            _ => {
                self.error(span, format!("`{}` has no function `{}`", ns, name));
                for a in args {
                    let _ = self.check_expr(a, None);
                }
                self.error_expr(span)
            }
        }
    }

    /// `Type.member` without a call.
    pub fn check_static_member(&mut self, t: TyId, name: &str, span: Span, _expected: Option<TyId>) -> TExpr {
        let t = self.tys.shallow(t);
        match self.tys.kind(t).clone() {
            TyKind::Enum(d, _) => {
                let def = self.enums[d as usize].clone();
                match def.variants.iter().position(|v| v.name == name) {
                    Some(idx) => {
                        if !matches!(def.variants[idx].payload, VariantPayloadDef::Unit) {
                            self.error(span, format!("variant `{}.{}` carries a payload; construct it with `{}.{}(...)`", def.name, name, def.name, name));
                        }
                        self.note_used(t);
                        self.mk(TExprKind::EnumLit { variant: idx as u32, payload: vec![] }, t, span)
                    }
                    None => {
                        self.error(span, format!("enum `{}` has no variant `{}`", def.name, name));
                        self.error_expr(span)
                    }
                }
            }
            TyKind::ErrorSet(Some(s)) => {
                let set = self.error_sets[s as usize].clone();
                let id = self.error_id(name);
                if !set.ids.contains(&id) {
                    self.error(span, format!("error set `{}` has no member `{}`", set.name, name));
                }
                let et = self.tys.intern(TyKind::ErrorSet(Some(s)));
                self.mk(TExprKind::ErrVal(id), et, span)
            }
            TyKind::Struct(..) | TyKind::Distinct(_) => {
                // associated function as a value
                if let Some(fid) = self.find_inherent_method(t, name) {
                    let def = self.fns[fid as usize].clone();
                    if def.is_generic {
                        self.error(span, format!("`{}` is generic; call it directly", name));
                        return self.error_expr(span);
                    }
                    if def.decl.params.iter().any(|p| p.owned) {
                        self.error(span, format!("`{}` takes `own` parameters and cannot be used as a function value; call it directly", name));
                        return self.error_expr(span);
                    }
                    let inst = self.instantiate(fid, vec![], span);
                    let (ps, r) = self.fn_sig(inst);
                    let neg = self.funcs[inst as usize].declared_neg;
                    let ft = self.tys.intern(TyKind::Fn(ps, r, Effects::ALL.without(neg).0));
                    return self.mk(TExprKind::FnToFat(inst), ft, span);
                }
                let tn = self.type_name(t);
                self.error(span, format!("`{}` has no associated item `{}`", tn, name));
                self.error_expr(span)
            }
            _ => {
                let tn = self.type_name(t);
                self.error(span, format!("`{}` has no member `{}`", tn, name));
                self.error_expr(span)
            }
        }
    }

    /// Find a method in an inherent or trait impl for type `t`.
    pub fn find_inherent_method(&mut self, t: TyId, name: &str) -> Option<FnDefId> {
        let t = self.tys.resolve(t, true);
        // inherent first
        for i in 0..self.impls.len() {
            let im = self.impls[i].clone();
            if im.trait_name.is_some() {
                continue;
            }
            if let Some((_, fid)) = im.methods.iter().find(|(n, _)| n == name) {
                if self.match_impl_target(&im, t).is_some() {
                    return Some(*fid);
                }
            }
        }
        for i in 0..self.impls.len() {
            let im = self.impls[i].clone();
            if im.trait_name.is_none() {
                continue;
            }
            if let Some((_, fid)) = im.methods.iter().find(|(n, _)| n == name) {
                if self.match_impl_target(&im, t).is_some() {
                    return Some(*fid);
                }
            }
        }
        // trait default methods
        for i in 0..self.impls.len() {
            let im = self.impls[i].clone();
            if let Some(tn) = &im.trait_name {
                if self.match_impl_target(&im, t).is_some() {
                    if let Some(ItemRef::Trait(tid)) = self.lookup_item(im.module, tn) {
                        let tdef = self.traits[tid as usize].clone();
                        if let Some(m) = tdef.methods.iter().find(|m| m.name == name && m.body.is_some()) {
                            // materialize a default method as an impl method
                            let fid = self.fns.len() as FnDefId;
                            let type_params: Vec<String> =
                                m.params.iter().filter(|p| p.comptime && matches!(&p.ty, TypeExpr::Named { path, .. } if path.len() == 1 && path[0] == "type")).map(|p| p.name.clone()).collect();
                            let is_generic = !type_params.is_empty() || !im.type_params.is_empty();
                            self.fns.push(FnDef { decl: m.clone(), module: im.module, impl_id: Some(i as u32), is_generic, type_params });
                            self.impls[i].methods.push((name.to_string(), fid));
                            return Some(fid);
                        }
                    }
                }
            }
        }
        None
    }

    /// `Type.name(args)`
    pub fn check_static_call(&mut self, t: TyId, name: &str, args: &[Expr], expected: Option<TyId>, span: Span) -> TExpr {
        let t = self.tys.shallow(t);
        let void = self.tys.void();
        let usize_t = self.tys.usize();
        let u8t = self.tys.u8();
        let bytes = self.tys.slice(false, u8t);
        match self.tys.kind(t).clone() {
            TyKind::List(e) => match name {
                "new" => {
                    if !self.check_args_n(args, 0, "List.new", span) {
                        return self.error_expr(span);
                    }
                    self.builtin(Builtin::ListNew, vec![], vec![e], t, span)
                }
                "with_capacity" => {
                    if !self.check_args_n(args, 1, "List.with_capacity", span) {
                        return self.error_expr(span);
                    }
                    let n = self.arg(&args[0], usize_t, "capacity");
                    self.add_effect(Effects::ALLOCATES, span, "reserving list capacity allocates");
                    self.builtin(Builtin::ListWithCapacity, vec![n], vec![e], t, span)
                }
                "from" => {
                    if !self.check_args_n(args, 1, "List.from", span) {
                        return self.error_expr(span);
                    }
                    let st = self.tys.slice(false, e);
                    let s = self.arg(&args[0], st, "source slice");
                    self.add_effect(Effects::ALLOCATES, span, "copying into a List allocates");
                    self.builtin(Builtin::ListFromSlice, vec![s], vec![e], t, span)
                }
                _ => {
                    self.error(span, format!("`List` has no associated function `{}`; available: new, with_capacity, from", name));
                    self.error_expr(span)
                }
            },
            TyKind::Str => match name {
                "new" => {
                    if !self.check_args_n(args, 0, "String.new", span) {
                        return self.error_expr(span);
                    }
                    self.builtin(Builtin::StringNew, vec![], vec![], t, span)
                }
                "from" => {
                    if !self.check_args_n(args, 1, "String.from", span) {
                        return self.error_expr(span);
                    }
                    let s = self.arg(&args[0], bytes, "source");
                    self.add_effect(Effects::ALLOCATES, span, "building a String allocates");
                    self.builtin(Builtin::StringFrom, vec![s], vec![], t, span)
                }
                "with_capacity" => {
                    if !self.check_args_n(args, 1, "String.with_capacity", span) {
                        return self.error_expr(span);
                    }
                    let n = self.arg(&args[0], usize_t, "capacity");
                    self.add_effect(Effects::ALLOCATES, span, "reserving capacity allocates");
                    self.builtin(Builtin::StringWithCapacity, vec![n], vec![], t, span)
                }
                _ => {
                    self.error(span, format!("`String` has no associated function `{}`; available: new, from, with_capacity", name));
                    self.error_expr(span)
                }
            },
            TyKind::Map(k, v) => match name {
                "new" => {
                    if !self.check_args_n(args, 0, "Map.new", span) {
                        return self.error_expr(span);
                    }
                    self.builtin(Builtin::MapNew, vec![], vec![k, v], t, span)
                }
                _ => {
                    self.error(span, format!("`Map` has no associated function `{}`; available: new", name));
                    self.error_expr(span)
                }
            },
            TyKind::Enum(..) => self.check_variant_ctor(t, name, args, span),
            TyKind::Struct(d, _) => {
                let def = self.structs[d as usize].clone();
                if def.kind == StructKind::Record && name == "new" {
                    // validated construction: Record.new(.{ ... }) -> !Record
                    if !self.check_args_n(args, 1, "new", span) {
                        return self.error_expr(span);
                    }
                    match &args[0] {
                        Expr::StructLit { ty: None, fields, span: lsp } => {
                            self.record_new_mode = true;
                            let lit = self.check_struct_lit(None, fields, Some(t), *lsp);
                            self.record_new_mode = false;
                            return self.record_check(lit, t, true, span);
                        }
                        _ => {
                            self.error(args[0].span(), format!("`{}.new` takes an anonymous struct literal: `{}.new(.{{ .field = value }})`", def.name, def.name));
                            return self.error_expr(span);
                        }
                    }
                }
                if let Some(fid) = self.find_inherent_method(t, name) {
                    let fdef = self.fns[fid as usize].clone();
                    if fdef.decl.params.first().map(|p| p.name == "self").unwrap_or(false) {
                        self.error(span, format!("`{}` is a method; call it on a value: `value.{}(...)`", name, name));
                        return self.error_expr(span);
                    }
                    return self.call_fn_def(fid, None, args, expected, span);
                }
                self.error(span, format!("`{}` has no associated function `{}`", def.name, name));
                for a in args {
                    let _ = self.check_expr(a, None);
                }
                self.error_expr(span)
            }
            TyKind::Distinct(_) => {
                if let Some(fid) = self.find_inherent_method(t, name) {
                    return self.call_fn_def(fid, None, args, expected, span);
                }
                let tn = self.type_name(t);
                self.error(span, format!("`{}` has no associated function `{}`", tn, name));
                self.error_expr(span)
            }
            _ => {
                let tn = self.type_name(t);
                let _ = void;
                self.error(span, format!("`{}` has no associated function `{}`", tn, name));
                for a in args {
                    let _ = self.check_expr(a, None);
                }
                self.error_expr(span)
            }
        }
    }

    // ----- method calls on values --------------------------------------------------

    pub fn check_method_call(&mut self, receiver: &Expr, method: &str, args: &[Expr], expected: Option<TyId>, span: Span) -> TExpr {
        let recv = self.check_expr(receiver, None);
        let rt = self.tys.shallow(recv.ty);
        match self.tys.kind(rt).clone() {
            TyKind::Type => {
                if let TExprKind::TypeVal(t) = recv.kind {
                    return self.check_static_call(t, method, args, expected, span);
                }
            }
            TyKind::Namespace(ns) => return self.check_namespace_call(&ns, method, args, expected, span),
            _ => {}
        }
        if let TyKind::Dyn(trait_id, eff) = self.tys.kind(peel_ptr(self, rt)).clone() {
            let recv = self.auto_deref_all(recv);
            let tdef = self.traits[trait_id as usize].clone();
            let idx = match tdef.methods.iter().position(|m| m.name == method) {
                Some(i) => i,
                None => {
                    self.error(span, format!("trait `{}` has no method `{}`", tdef.name, method));
                    for a in args {
                        let _ = self.check_expr(a, None);
                    }
                    return self.error_expr(span);
                }
            };
            let (ps, ret, mutable) = match self.dyn_method_sig(trait_id, idx, span) {
                Some(s) => s,
                None => return self.error_expr(span),
            };
            if mutable && self.is_place(&recv) && !self.place_mutable(&recv) {
                self.error(span, format!("`{}` mutates its receiver, but this trait object is immutable", method));
            }
            if args.len() != ps.len() {
                self.error(span, format!("`{}` takes {} argument(s) but {} were given", method, ps.len(), args.len()));
                return self.error_expr(span);
            }
            let mut targs = Vec::new();
            for (a, &p) in args.iter().zip(ps.iter()) {
                let te = self.check_expr(a, Some(p));
                targs.push(self.coerce_or_error(te, p, "argument"));
            }
            // a dynamic call acquires every effect the trait object permits (E4)
            let permitted = Effects(eff);
            if !permitted.is_empty() {
                self.add_effect(permitted, span, "calling through a trait object acquires the effects its type permits");
            }
            return self.mk(TExprKind::DynCall { recv: Box::new(recv), method: idx as u32, args: targs }, ret, span);
        }
        let base_t = peel_ptr(self, rt);
        // user-defined methods first (they may shadow nothing builtin on user types)
        match self.tys.kind(base_t).clone() {
            TyKind::Struct(..) | TyKind::Enum(..) | TyKind::Distinct(_) => {
                if let Some(fid) = self.find_inherent_method(base_t, method) {
                    let fdef = self.fns[fid as usize].clone();
                    if !fdef.decl.params.first().map(|p| p.name == "self").unwrap_or(false) {
                        let tn = self.type_name(base_t);
                        self.error(span, format!("`{}` is an associated function, not a method; call it as `{}.{}(...)`", method, tn, method));
                        return self.error_expr(span);
                    }
                    return self.call_fn_def(fid, Some(recv), args, expected, span);
                }
                if self.is_ref_class(base_t) {
                    if let Some(te) = self.ref_method(recv.clone(), base_t, method, args, span) {
                        return te;
                    }
                }
            }
            _ => {}
        }
        // builtin methods
        let recv = self.auto_deref_all(recv);
        let t = self.tys.resolve(recv.ty, true);
        if let Some(te) = self.builtin_method(recv.clone(), t, method, args, expected, span) {
            return te;
        }
        let tn = self.type_name(t);
        let avail = self.available_methods(t);
        let hint = if avail.is_empty() { String::new() } else { format!("; available: {}", avail.join(", ")) };
        self.error(span, format!("no method `{}` on type `{}`{}", method, tn, hint));
        for a in args {
            let _ = self.check_expr(a, None);
        }
        self.error_expr(span)
    }

    fn auto_deref_all(&mut self, mut e: TExpr) -> TExpr {
        loop {
            let t = self.tys.shallow(e.ty);
            match self.tys.kind(t).clone() {
                TyKind::Ptr(_, inner) => {
                    let sp = e.span;
                    e = TExpr { kind: TExprKind::Deref(Box::new(e)), ty: inner, span: sp };
                }
                _ => return e,
            }
        }
    }

    fn available_methods(&self, t: TyId) -> Vec<&'static str> {
        match self.tys.kind(t) {
            TyKind::List(_) => {
                vec!["append", "pop", "clear", "clone", "last", "first", "insert", "remove", "swap_remove", "extend", "reserve", "items", "contains", "index_of", "sort", "reverse", "fill", "is_empty"]
            }
            TyKind::Str => vec![
                "append",
                "append_char",
                "clone",
                "clear",
                "pop",
                "bytes",
                "starts_with",
                "ends_with",
                "contains",
                "find",
                "trim",
                "split",
                "lines",
                "parse_int",
                "parse_float",
                "is_empty",
                "eq_ignore_case",
            ],
            TyKind::Map(..) => vec!["put", "get", "contains", "remove", "clear", "clone", "keys", "values", "is_empty"],
            TyKind::Slice(..) | TyKind::Array(..) => vec![
                "fill",
                "reverse",
                "sort",
                "contains",
                "index_of",
                "starts_with",
                "ends_with",
                "find",
                "trim",
                "split",
                "lines",
                "to_owned",
                "to_string",
                "parse_int",
                "parse_float",
                "is_empty",
                "copy_from",
                "eq_ignore_case",
            ],
            TyKind::Int(_) => vec!["abs", "min", "max", "checked_add", "checked_sub", "checked_mul", "to_string"],
            TyKind::Float(_) => vec!["abs", "sqrt", "floor", "ceil", "round", "min", "max", "pow", "to_string"],
            TyKind::Char => vec!["is_digit", "is_alpha", "is_space", "is_upper", "is_lower", "to_lower", "to_upper", "to_digit"],
            _ => vec![],
        }
    }

    fn ref_method(&mut self, recv: TExpr, t: TyId, method: &str, args: &[Expr], span: Span) -> Option<TExpr> {
        match method {
            "weak" => {
                if !self.check_args_n(args, 0, "weak", span) {
                    return Some(self.error_expr(span));
                }
                let wt = self.tys.intern(TyKind::Weak(t));
                self.add_effect(Effects::REFCOUNTS, span, "creating a weak reference");
                Some(self.builtin(Builtin::Weak, vec![recv], vec![t], wt, span))
            }
            _ => None,
        }
    }

    fn require_mut_recv(&mut self, recv: &TExpr, method: &str, span: Span) {
        if self.is_place(recv) && !self.place_mutable(recv) {
            let what = match &recv.kind {
                TExprKind::Local(l) => format!("`{}`", self.cur.as_ref().unwrap().locals[*l as usize].name),
                _ => "the receiver".to_string(),
            };
            self.error_note(span, format!("`{}` mutates its receiver, but {} is immutable", method, what), None, "declare it with `var`");
        }
    }

    fn builtin_method(&mut self, recv: TExpr, t: TyId, method: &str, args: &[Expr], expected: Option<TyId>, span: Span) -> Option<TExpr> {
        let void = self.tys.void();
        let bool_t = self.tys.bool();
        let usize_t = self.tys.usize();
        let u8t = self.tys.u8();
        let bytes = self.tys.slice(false, u8t);
        let string = self.tys.string();
        let k = self.tys.kind(t).clone();
        match k {
            TyKind::List(e) => {
                let te = match method {
                    "append" | "push" => {
                        if !self.check_args_n(args, 1, "append", span) {
                            return Some(self.error_expr(span));
                        }
                        self.require_mut_recv(&recv, "append", span);
                        let v = self.arg(&args[0], e, "element");
                        let v = self.take_ownership(v);
                        self.add_effect(Effects::ALLOCATES, span, "appending to a List may grow it");
                        self.builtin(Builtin::ListAppend, vec![recv, v], vec![e], void, span)
                    }
                    "pop" => {
                        if !self.check_args_n(args, 0, "pop", span) {
                            return Some(self.error_expr(span));
                        }
                        self.require_mut_recv(&recv, "pop", span);
                        let o = self.tys.opt(e);
                        self.builtin(Builtin::ListPop, vec![recv], vec![e], o, span)
                    }
                    "clear" => {
                        if !self.check_args_n(args, 0, "clear", span) {
                            return Some(self.error_expr(span));
                        }
                        self.require_mut_recv(&recv, "clear", span);
                        self.builtin(Builtin::ListClear, vec![recv], vec![e], void, span)
                    }
                    "clone" => {
                        if !self.check_args_n(args, 0, "clone", span) {
                            return Some(self.error_expr(span));
                        }
                        self.add_effect(Effects::ALLOCATES, span, "cloning a List allocates");
                        self.builtin(Builtin::ListClone, vec![recv], vec![e], t, span)
                    }
                    "last" | "first" => {
                        if !self.check_args_n(args, 0, method, span) {
                            return Some(self.error_expr(span));
                        }
                        let o = self.tys.opt(e);
                        let s = self.tys.slice(false, e);
                        let sl = TExpr { kind: TExprKind::ListToSlice(Box::new(recv)), ty: s, span };
                        let idx = if method == "last" { 1 } else { 0 };
                        let it = TExpr { kind: TExprKind::Int(idx), ty: usize_t, span };
                        self.builtin(Builtin::ListLast, vec![sl, it], vec![e], o, span)
                    }
                    "insert" => {
                        if !self.check_args_n(args, 2, "insert", span) {
                            return Some(self.error_expr(span));
                        }
                        self.require_mut_recv(&recv, "insert", span);
                        let i = self.arg(&args[0], usize_t, "index");
                        let v = self.arg(&args[1], e, "element");
                        let v = self.take_ownership(v);
                        self.add_effect(Effects::ALLOCATES, span, "inserting into a List may grow it");
                        self.add_effect(Effects::PANICS, span, "insert panics when the index is out of range");
                        self.builtin(Builtin::ListInsert, vec![recv, i, v], vec![e], void, span)
                    }
                    "remove" => {
                        if !self.check_args_n(args, 1, "remove", span) {
                            return Some(self.error_expr(span));
                        }
                        self.require_mut_recv(&recv, "remove", span);
                        let i = self.arg(&args[0], usize_t, "index");
                        self.add_effect(Effects::PANICS, span, "remove panics when the index is out of range");
                        self.builtin(Builtin::ListRemove, vec![recv, i], vec![e], e, span)
                    }
                    "swap_remove" => {
                        if !self.check_args_n(args, 1, "swap_remove", span) {
                            return Some(self.error_expr(span));
                        }
                        self.require_mut_recv(&recv, "swap_remove", span);
                        let i = self.arg(&args[0], usize_t, "index");
                        self.add_effect(Effects::PANICS, span, "swap_remove panics when the index is out of range");
                        self.builtin(Builtin::ListSwapRemove, vec![recv, i], vec![e], e, span)
                    }
                    "extend" => {
                        if !self.check_args_n(args, 1, "extend", span) {
                            return Some(self.error_expr(span));
                        }
                        self.require_mut_recv(&recv, "extend", span);
                        let st = self.tys.slice(false, e);
                        let s = self.arg(&args[0], st, "elements");
                        if self.needs_drop(e) {
                            self.error(span, "`extend` copies elements; the element type owns resources, so append clones instead");
                        }
                        self.add_effect(Effects::ALLOCATES, span, "extending a List may grow it");
                        self.builtin(Builtin::ListExtend, vec![recv, s], vec![e], void, span)
                    }
                    "reserve" => {
                        if !self.check_args_n(args, 1, "reserve", span) {
                            return Some(self.error_expr(span));
                        }
                        self.require_mut_recv(&recv, "reserve", span);
                        let n = self.arg(&args[0], usize_t, "capacity");
                        self.add_effect(Effects::ALLOCATES, span, "reserving allocates");
                        self.builtin(Builtin::ListReserve, vec![recv, n], vec![e], void, span)
                    }
                    "items" | "as_slice" => {
                        if !self.check_args_n(args, 0, method, span) {
                            return Some(self.error_expr(span));
                        }
                        let m = self.place_mutable(&recv);
                        let st = self.tys.slice(m, e);
                        TExpr { kind: TExprKind::ListToSlice(Box::new(recv)), ty: st, span }
                    }
                    "is_empty" => {
                        if !self.check_args_n(args, 0, "is_empty", span) {
                            return Some(self.error_expr(span));
                        }
                        let len = self.builtin(Builtin::Len, vec![recv], vec![], usize_t, span);
                        let zero = TExpr { kind: TExprKind::Int(0), ty: usize_t, span };
                        self.mk(TExprKind::Binary { op: BinOp::Eq, lhs: Box::new(len), rhs: Box::new(zero), mode: ArithMode::Plain, proven: true }, bool_t, span)
                    }
                    _ => {
                        // slice methods apply to lists through a view
                        let m = self.place_mutable(&recv);
                        let st = self.tys.slice(m, e);
                        let sl = TExpr { kind: TExprKind::ListToSlice(Box::new(recv)), ty: st, span };
                        return self.slice_method(sl, e, m, method, args, expected, span);
                    }
                };
                Some(te)
            }
            TyKind::Str => {
                let te = match method {
                    "append" => {
                        if !self.check_args_n(args, 1, "append", span) {
                            return Some(self.error_expr(span));
                        }
                        self.require_mut_recv(&recv, "append", span);
                        let s = self.arg(&args[0], bytes, "text");
                        self.add_effect(Effects::ALLOCATES, span, "appending to a String may grow it");
                        self.builtin(Builtin::StringAppend, vec![recv, s], vec![], void, span)
                    }
                    "append_char" => {
                        if !self.check_args_n(args, 1, "append_char", span) {
                            return Some(self.error_expr(span));
                        }
                        self.require_mut_recv(&recv, "append_char", span);
                        let ct = self.tys.char();
                        let c = self.arg(&args[0], ct, "character");
                        self.add_effect(Effects::ALLOCATES, span, "appending to a String may grow it");
                        self.builtin(Builtin::StringAppendChar, vec![recv, c], vec![], void, span)
                    }
                    "push_byte" => {
                        if !self.check_args_n(args, 1, "push_byte", span) {
                            return Some(self.error_expr(span));
                        }
                        self.require_mut_recv(&recv, "push_byte", span);
                        let u8t = self.tys.int(IntTy::U8);
                        let b = self.arg(&args[0], u8t, "byte");
                        self.add_effect(Effects::ALLOCATES, span, "appending to a String may grow it");
                        self.builtin(Builtin::StringPushByte, vec![recv, b], vec![], void, span)
                    }
                    "clone" | "to_string" => {
                        if !self.check_args_n(args, 0, method, span) {
                            return Some(self.error_expr(span));
                        }
                        self.add_effect(Effects::ALLOCATES, span, "cloning a String allocates");
                        self.builtin(Builtin::StringClone, vec![recv], vec![], string, span)
                    }
                    "clear" => {
                        if !self.check_args_n(args, 0, "clear", span) {
                            return Some(self.error_expr(span));
                        }
                        self.require_mut_recv(&recv, "clear", span);
                        self.builtin(Builtin::StringClear, vec![recv], vec![], void, span)
                    }
                    "pop" => {
                        if !self.check_args_n(args, 0, "pop", span) {
                            return Some(self.error_expr(span));
                        }
                        self.require_mut_recv(&recv, "pop", span);
                        let o = self.tys.opt(u8t);
                        self.builtin(Builtin::StringPop, vec![recv], vec![], o, span)
                    }
                    "bytes" | "as_slice" => {
                        if !self.check_args_n(args, 0, method, span) {
                            return Some(self.error_expr(span));
                        }
                        TExpr { kind: TExprKind::StrToSlice(Box::new(recv)), ty: bytes, span }
                    }
                    _ => {
                        let sl = TExpr { kind: TExprKind::StrToSlice(Box::new(recv)), ty: bytes, span };
                        return self.slice_method(sl, u8t, false, method, args, expected, span);
                    }
                };
                Some(te)
            }
            TyKind::Map(kt, vt) => {
                // lookups by content: a `String`-keyed map accepts `[]u8` keys
                let lookup_kt = if matches!(self.tys.kind(self.tys.shallow(kt)), TyKind::Str) { bytes } else { kt };
                let te = match method {
                    "put" | "insert" => {
                        if !self.check_args_n(args, 2, "put", span) {
                            return Some(self.error_expr(span));
                        }
                        self.require_mut_recv(&recv, "put", span);
                        let kk = self.arg(&args[0], kt, "key");
                        let kk = self.take_ownership(kk);
                        let vv = self.arg(&args[1], vt, "value");
                        let vv = self.take_ownership(vv);
                        self.add_effect(Effects::ALLOCATES, span, "inserting into a Map may grow it");
                        self.builtin(Builtin::MapPut, vec![recv, kk, vv], vec![kt, vt], void, span)
                    }
                    "get" => {
                        if !self.check_args_n(args, 1, "get", span) {
                            return Some(self.error_expr(span));
                        }
                        let kk = self.arg(&args[0], lookup_kt, "key");
                        let o = self.tys.opt(vt);
                        self.builtin(Builtin::MapGet, vec![recv, kk], vec![kt, vt], o, span)
                    }
                    "contains" => {
                        if !self.check_args_n(args, 1, "contains", span) {
                            return Some(self.error_expr(span));
                        }
                        let kk = self.arg(&args[0], lookup_kt, "key");
                        self.builtin(Builtin::MapContains, vec![recv, kk], vec![kt, vt], bool_t, span)
                    }
                    "remove" => {
                        if !self.check_args_n(args, 1, "remove", span) {
                            return Some(self.error_expr(span));
                        }
                        self.require_mut_recv(&recv, "remove", span);
                        let kk = self.arg(&args[0], lookup_kt, "key");
                        self.builtin(Builtin::MapRemove, vec![recv, kk], vec![kt, vt], bool_t, span)
                    }
                    "clear" => {
                        if !self.check_args_n(args, 0, "clear", span) {
                            return Some(self.error_expr(span));
                        }
                        self.require_mut_recv(&recv, "clear", span);
                        self.builtin(Builtin::MapClear, vec![recv], vec![kt, vt], void, span)
                    }
                    "clone" => {
                        if !self.check_args_n(args, 0, "clone", span) {
                            return Some(self.error_expr(span));
                        }
                        self.add_effect(Effects::ALLOCATES, span, "cloning a Map allocates");
                        self.builtin(Builtin::MapClone, vec![recv], vec![kt, vt], t, span)
                    }
                    "keys" => {
                        if !self.check_args_n(args, 0, "keys", span) {
                            return Some(self.error_expr(span));
                        }
                        self.add_effect(Effects::ALLOCATES, span, "collecting keys allocates");
                        let lt = self.tys.list(kt);
                        self.builtin(Builtin::MapKeys, vec![recv], vec![kt, vt], lt, span)
                    }
                    "values" => {
                        if !self.check_args_n(args, 0, "values", span) {
                            return Some(self.error_expr(span));
                        }
                        self.add_effect(Effects::ALLOCATES, span, "collecting values allocates");
                        let lt = self.tys.list(vt);
                        self.builtin(Builtin::MapValues, vec![recv], vec![kt, vt], lt, span)
                    }
                    "is_empty" => {
                        if !self.check_args_n(args, 0, "is_empty", span) {
                            return Some(self.error_expr(span));
                        }
                        let len = self.builtin(Builtin::Len, vec![recv], vec![], usize_t, span);
                        let zero = TExpr { kind: TExprKind::Int(0), ty: usize_t, span };
                        self.mk(TExprKind::Binary { op: BinOp::Eq, lhs: Box::new(len), rhs: Box::new(zero), mode: ArithMode::Plain, proven: true }, bool_t, span)
                    }
                    _ => return None,
                };
                Some(te)
            }
            TyKind::Slice(m, e) => self.slice_method(recv, e, m, method, args, expected, span),
            TyKind::Array(_, e) => {
                let m = self.place_mutable(&recv);
                let st = self.tys.slice(m, e);
                let sl = TExpr { kind: TExprKind::ArrayToSlice(Box::new(recv)), ty: st, span };
                self.slice_method(sl, e, m, method, args, expected, span)
            }
            TyKind::Int(it) => {
                let te = match method {
                    "abs" => {
                        if !self.check_args_n(args, 0, "abs", span) {
                            return Some(self.error_expr(span));
                        }
                        self.add_effect(Effects::PANICS, span, "abs overflows for the minimum value");
                        self.builtin(Builtin::MathAbs, vec![recv], vec![t], t, span)
                    }
                    "min" | "max" => {
                        if !self.check_args_n(args, 1, method, span) {
                            return Some(self.error_expr(span));
                        }
                        let b = self.arg(&args[0], t, "operand");
                        let op = if method == "min" { Builtin::MathMin } else { Builtin::MathMax };
                        self.builtin(op, vec![recv, b], vec![t], t, span)
                    }
                    "checked_add" | "checked_sub" | "checked_mul" => {
                        if !self.check_args_n(args, 1, method, span) {
                            return Some(self.error_expr(span));
                        }
                        let b = self.arg(&args[0], t, "operand");
                        let o = self.tys.opt(t);
                        let bop = match method {
                            "checked_add" => BinOp::Add,
                            "checked_sub" => BinOp::Sub,
                            _ => BinOp::Mul,
                        };
                        let _ = it;
                        // encoded as a builtin: tys[0]=t, args=[a,b], op selected by a marker int
                        let marker = TExpr {
                            kind: TExprKind::Int(match bop {
                                BinOp::Add => 0,
                                BinOp::Sub => 1,
                                _ => 2,
                            }),
                            ty: t,
                            span,
                        };
                        self.builtin(Builtin::Hash, vec![recv, b, marker], vec![t], o, span)
                    }
                    "to_string" => {
                        if !self.check_args_n(args, 0, "to_string", span) {
                            return Some(self.error_expr(span));
                        }
                        self.add_effect(Effects::ALLOCATES, span, "formatting into a String allocates");
                        self.builtin(Builtin::IntToStr, vec![recv], vec![t], string, span)
                    }
                    _ => return None,
                };
                Some(te)
            }
            TyKind::Float(_) => {
                let op = match method {
                    "abs" => Some(Builtin::MathAbs),
                    "sqrt" => Some(Builtin::MathSqrt),
                    "floor" => Some(Builtin::MathFloor),
                    "ceil" => Some(Builtin::MathCeil),
                    "round" => Some(Builtin::MathRound),
                    _ => None,
                };
                if let Some(op) = op {
                    if !self.check_args_n(args, 0, method, span) {
                        return Some(self.error_expr(span));
                    }
                    return Some(self.builtin(op, vec![recv], vec![t], t, span));
                }
                let te = match method {
                    "min" | "max" | "pow" => {
                        if !self.check_args_n(args, 1, method, span) {
                            return Some(self.error_expr(span));
                        }
                        let b = self.arg(&args[0], t, "operand");
                        let op = match method {
                            "min" => Builtin::MathMin,
                            "max" => Builtin::MathMax,
                            _ => Builtin::MathPow,
                        };
                        self.builtin(op, vec![recv, b], vec![t], t, span)
                    }
                    "to_string" => {
                        if !self.check_args_n(args, 0, "to_string", span) {
                            return Some(self.error_expr(span));
                        }
                        self.add_effect(Effects::ALLOCATES, span, "formatting into a String allocates");
                        self.builtin(Builtin::FloatToStr, vec![recv], vec![t], string, span)
                    }
                    _ => return None,
                };
                Some(te)
            }
            TyKind::Char => {
                let op = match method {
                    "is_digit" => Some((Builtin::CharIsDigit, bool_t)),
                    "is_alpha" => Some((Builtin::CharIsAlpha, bool_t)),
                    "is_space" => Some((Builtin::CharIsSpace, bool_t)),
                    "to_lower" => Some((Builtin::CharToLower, t)),
                    "to_upper" => Some((Builtin::CharToUpper, t)),
                    _ => None,
                };
                if let Some((op, rt)) = op {
                    if !self.check_args_n(args, 0, method, span) {
                        return Some(self.error_expr(span));
                    }
                    return Some(self.builtin(op, vec![recv], vec![], rt, span));
                }
                if method == "to_digit" {
                    if !self.check_args_n(args, 0, method, span) {
                        return Some(self.error_expr(span));
                    }
                    let u32t = self.tys.int(IntTy::U32);
                    let o = self.tys.opt(u32t);
                    return Some(self.builtin(Builtin::CharToDigit, vec![recv], vec![], o, span));
                }
                None
            }
            TyKind::Weak(inner) => match method {
                "upgrade" => {
                    if !self.check_args_n(args, 0, "upgrade", span) {
                        return Some(self.error_expr(span));
                    }
                    let o = self.tys.opt(inner);
                    self.add_effect(Effects::REFCOUNTS, span, "upgrading a weak reference retains");
                    Some(self.builtin(Builtin::Upgrade, vec![recv], vec![inner], o, span))
                }
                _ => None,
            },
            TyKind::Opt(_) => {
                self.error(span, format!("cannot call `{}` on an optional; unwrap it first with `orelse`, `.?`, or `if (x) |v|`", method));
                Some(self.error_expr(span))
            }
            TyKind::ErrUnion(..) => {
                self.error(span, format!("cannot call `{}` on an error union; use `try` first", method));
                Some(self.error_expr(span))
            }
            _ => None,
        }
    }

    fn slice_method(&mut self, recv: TExpr, e: TyId, mutable: bool, method: &str, args: &[Expr], _expected: Option<TyId>, span: Span) -> Option<TExpr> {
        let void = self.tys.void();
        let bool_t = self.tys.bool();
        let usize_t = self.tys.usize();
        let u8t = self.tys.u8();
        let bytes = self.tys.slice(false, u8t);
        let string = self.tys.string();
        let is_u8 = matches!(self.tys.kind(self.tys.shallow(e)), TyKind::Int(IntTy::U8));
        let te = match method {
            "fill" => {
                if !self.check_args_n(args, 1, "fill", span) {
                    return Some(self.error_expr(span));
                }
                if !mutable {
                    self.error(span, "`fill` needs a mutable slice (`[]mut T`)");
                }
                let v = self.arg(&args[0], e, "value");
                self.builtin(Builtin::SliceFill, vec![recv, v], vec![e], void, span)
            }
            "reverse" => {
                if !self.check_args_n(args, 0, "reverse", span) {
                    return Some(self.error_expr(span));
                }
                if !mutable {
                    self.error(span, "`reverse` needs a mutable slice (`[]mut T`)");
                }
                self.builtin(Builtin::SliceReverse, vec![recv], vec![e], void, span)
            }
            "sort" => {
                if !self.check_args_n(args, 0, "sort", span) {
                    return Some(self.error_expr(span));
                }
                if !mutable {
                    self.error(span, "`sort` needs a mutable slice (`[]mut T`)");
                }
                if !self.type_satisfies_trait(e, "Ord", 0) {
                    let tn = self.type_name(e);
                    self.error(span, format!("cannot sort elements of type `{}`; they must be ordered (derive(Ord))", tn));
                }
                self.builtin(Builtin::SliceSort, vec![recv], vec![e], void, span)
            }
            "contains" => {
                if !self.check_args_n(args, 1, "contains", span) {
                    return Some(self.error_expr(span));
                }
                if is_u8 && matches!(&args[0], Expr::Lit { value: Lit::Str(_), .. }) {
                    let n = self.arg(&args[0], bytes, "needle");
                    let ot = self.tys.opt(usize_t);
                    let f = self.builtin(Builtin::SliceFind, vec![recv, n], vec![e], ot, span);
                    let null = TExpr { kind: TExprKind::OptNull, ty: f.ty, span };
                    return Some(self.mk(TExprKind::Binary { op: BinOp::Ne, lhs: Box::new(f), rhs: Box::new(null), mode: ArithMode::Plain, proven: true }, bool_t, span));
                }
                let v = self.arg(&args[0], e, "value");
                if !self.type_satisfies_trait(e, "Eq", 0) {
                    let tn = self.type_name(e);
                    self.error(span, format!("cannot search for `{}` values; the element type must be comparable (derive(Eq))", tn));
                }
                self.builtin(Builtin::SliceContains, vec![recv, v], vec![e], bool_t, span)
            }
            "index_of" => {
                if !self.check_args_n(args, 1, "index_of", span) {
                    return Some(self.error_expr(span));
                }
                let v = self.arg(&args[0], e, "value");
                let o = self.tys.opt(usize_t);
                self.builtin(Builtin::SliceIndexOf, vec![recv, v], vec![e], o, span)
            }
            "copy_from" => {
                if !self.check_args_n(args, 1, "copy_from", span) {
                    return Some(self.error_expr(span));
                }
                if !mutable {
                    self.error(span, "`copy_from` needs a mutable slice (`[]mut T`)");
                }
                let st = self.tys.slice(false, e);
                let s = self.arg(&args[0], st, "source");
                self.add_effect(Effects::PANICS, span, "copy_from panics when the source is longer than the destination");
                self.builtin(Builtin::SliceCopy, vec![recv, s], vec![e], void, span)
            }
            "is_empty" => {
                if !self.check_args_n(args, 0, "is_empty", span) {
                    return Some(self.error_expr(span));
                }
                let len = self.builtin(Builtin::Len, vec![recv], vec![], usize_t, span);
                let zero = TExpr { kind: TExprKind::Int(0), ty: usize_t, span };
                self.mk(TExprKind::Binary { op: BinOp::Eq, lhs: Box::new(len), rhs: Box::new(zero), mode: ArithMode::Plain, proven: true }, bool_t, span)
            }
            "to_owned" | "to_list" => {
                if !self.check_args_n(args, 0, method, span) {
                    return Some(self.error_expr(span));
                }
                self.add_effect(Effects::ALLOCATES, span, "copying into a List allocates");
                let lt = self.tys.list(e);
                self.builtin(Builtin::ListFromSlice, vec![recv], vec![e], lt, span)
            }
            _ if is_u8 => match method {
                "starts_with" | "ends_with" => {
                    if !self.check_args_n(args, 1, method, span) {
                        return Some(self.error_expr(span));
                    }
                    let p = self.arg(&args[0], bytes, "prefix");
                    let op = if method == "starts_with" { Builtin::SliceStartsWith } else { Builtin::SliceEndsWith };
                    self.builtin(op, vec![recv, p], vec![e], bool_t, span)
                }
                "find" => {
                    if !self.check_args_n(args, 1, "find", span) {
                        return Some(self.error_expr(span));
                    }
                    let n = self.arg(&args[0], bytes, "needle");
                    let o = self.tys.opt(usize_t);
                    self.builtin(Builtin::SliceFind, vec![recv, n], vec![e], o, span)
                }
                "trim" => {
                    if !self.check_args_n(args, 0, "trim", span) {
                        return Some(self.error_expr(span));
                    }
                    self.builtin(Builtin::SliceTrim, vec![recv], vec![e], bytes, span)
                }
                "split" => {
                    if !self.check_args_n(args, 1, "split", span) {
                        return Some(self.error_expr(span));
                    }
                    let sep = self.arg(&args[0], bytes, "separator");
                    self.add_effect(Effects::ALLOCATES, span, "split collects the pieces into a List");
                    let lt = self.tys.list(bytes);
                    self.builtin(Builtin::SliceSplit, vec![recv, sep], vec![e], lt, span)
                }
                "lines" => {
                    if !self.check_args_n(args, 0, "lines", span) {
                        return Some(self.error_expr(span));
                    }
                    self.add_effect(Effects::ALLOCATES, span, "lines collects the pieces into a List");
                    let lt = self.tys.list(bytes);
                    self.builtin(Builtin::SliceLines, vec![recv], vec![e], lt, span)
                }
                "to_string" => {
                    if !self.check_args_n(args, 0, "to_string", span) {
                        return Some(self.error_expr(span));
                    }
                    self.add_effect(Effects::ALLOCATES, span, "building a String allocates");
                    self.builtin(Builtin::StringFrom, vec![recv], vec![], string, span)
                }
                "parse_int" => {
                    if !self.check_args_n(args, 1, "parse_int", span) {
                        return Some(self.error_expr(span));
                    }
                    let tv = self.check_expr(&args[0], None);
                    let target = match tv.kind {
                        TExprKind::TypeVal(t) => t,
                        _ => {
                            self.error(args[0].span(), "parse_int takes the integer type to parse, e.g. `s.parse_int(i32)`");
                            return Some(self.error_expr(span));
                        }
                    };
                    if !self.tys.is_integer(target) {
                        self.error(span, "parse_int needs an integer type");
                    }
                    let r = self.tys.err_union(None, target);
                    self.builtin(Builtin::SliceParseInt, vec![recv], vec![target], r, span)
                }
                "parse_float" => {
                    if !self.check_args_n(args, 0, "parse_float", span) {
                        return Some(self.error_expr(span));
                    }
                    let f64t = self.tys.float(FloatTy::F64);
                    let r = self.tys.err_union(None, f64t);
                    self.builtin(Builtin::SliceParseFloat, vec![recv], vec![f64t], r, span)
                }
                "eq_ignore_case" => {
                    if !self.check_args_n(args, 1, "eq_ignore_case", span) {
                        return Some(self.error_expr(span));
                    }
                    let o = self.arg(&args[0], bytes, "other");
                    self.builtin(Builtin::SliceEqIgnoreCase, vec![recv, o], vec![e], bool_t, span)
                }
                _ => return None,
            },
            _ => return None,
        };
        Some(te)
    }
}
