//! Reports over the typed IR: `nx refcounts` (retain and release sites).

use crate::check::Program;
use crate::diag::Span;
use crate::tir::*;
use crate::types::TyKind;

pub struct RefSite {
    pub span: Span,
    pub what: &'static str,
    pub func: String,
}

/// Every place the generated code retains or releases a `ref class` reference.
pub fn refcount_sites(p: &mut Program) -> Vec<RefSite> {
    let mut out = Vec::new();
    let funcs = p.funcs.clone();
    for f in &funcs {
        if let Some(b) = &f.body {
            let mut w = Walker { p, out: &mut out, func: f.name.clone(), locals: f.locals.clone() };
            w.block(b);
        }
    }
    out
}

struct Walker<'a> {
    p: &'a mut Program,
    out: &'a mut Vec<RefSite>,
    func: String,
    locals: Vec<Local>,
}

impl<'a> Walker<'a> {
    fn is_ref_like(&mut self, t: crate::types::TyId) -> bool {
        let t = self.p.tys.resolve(t, true);
        match self.p.tys.kind(t).clone() {
            TyKind::Struct(d, _) => self.p.structs[d as usize].kind == crate::ast::StructKind::RefClass,
            TyKind::Opt(e) | TyKind::Weak(e) => self.is_ref_like(e),
            _ => false,
        }
    }

    fn holds_refs(&mut self, t: crate::types::TyId) -> bool {
        if self.is_ref_like(t) {
            return true;
        }
        let t = self.p.tys.resolve(t, true);
        match self.p.tys.kind(t).clone() {
            TyKind::List(e) | TyKind::Array(_, e) | TyKind::ErrUnion(_, e) => self.holds_refs(e),
            TyKind::Map(k, v) => self.holds_refs(k) || self.holds_refs(v),
            TyKind::Tuple(ts) => ts.iter().any(|&e| self.holds_refs(e)),
            TyKind::Struct(_, _) => {
                let f = self.p.struct_field_tys.get(&t).cloned().unwrap_or_default();
                f.iter().any(|&e| self.holds_refs(e))
            }
            TyKind::Enum(..) => {
                let v = self.p.enum_variant_tys.get(&t).cloned().unwrap_or_default();
                v.iter().flatten().any(|&e| self.holds_refs(e))
            }
            _ => false,
        }
    }

    fn site(&mut self, span: Span, what: &'static str) {
        let func = self.func.clone();
        self.out.push(RefSite { span, what, func });
    }

    fn block(&mut self, b: &TBlock) {
        for s in &b.stmts {
            self.stmt(s);
        }
        if let Some(t) = &b.tail {
            self.expr(t);
        }
    }

    fn stmt(&mut self, s: &TStmt) {
        match s {
            TStmt::Let { local, init, span } => {
                let ty = self.locals[*local as usize].ty;
                if self.holds_refs(ty) {
                    self.site(*span, "release at scope exit");
                }
                if let Some(e) = init {
                    self.expr(e);
                }
            }
            TStmt::Assign { target, value, span, .. } => {
                if self.holds_refs(target.ty) {
                    self.site(*span, "release of the previous value");
                }
                self.expr(target);
                self.expr(value);
            }
            TStmt::Expr(e) | TStmt::Return { value: Some(e), .. } | TStmt::Break { value: Some(e), .. } => self.expr(e),
            TStmt::Defer { body, .. } | TStmt::ErrDefer { body, .. } => self.stmt(body),
            TStmt::While { cond, body, .. } => {
                self.expr(cond);
                self.block(body);
            }
            TStmt::ForRange { start, end, body, .. } => {
                self.expr(start);
                self.expr(end);
                self.block(body);
            }
            TStmt::ForSlice { items, body, .. } => {
                for (_, e) in items {
                    self.expr(e);
                }
                self.block(body);
            }
            TStmt::Block(b) | TStmt::Using { body: b, .. } => self.block(b),
            _ => {}
        }
    }

    fn expr(&mut self, e: &TExpr) {
        match &e.kind {
            TExprKind::DynFrom { expr, .. } => self.expr(expr),
            TExprKind::DynCall { recv, args, .. } => {
                self.expr(recv);
                for a in args {
                    self.expr(a);
                }
            }
            TExprKind::Retained(inner) => {
                self.site(e.span, "retain (copy of a reference)");
                self.expr(inner);
            }
            TExprKind::RefNew { fields } => {
                self.site(e.span, "allocation with count 1");
                for (_, f) in fields {
                    self.expr(f);
                }
            }
            TExprKind::Builtin { op: Builtin::Weak, args, .. } => {
                self.site(e.span, "weak reference created");
                for a in args {
                    self.expr(a);
                }
            }
            TExprKind::Builtin { op: Builtin::Upgrade, args, .. } => {
                self.site(e.span, "retain on upgrade");
                for a in args {
                    self.expr(a);
                }
            }
            TExprKind::Builtin { args, .. } | TExprKind::Call { args, .. } | TExprKind::ArrayLit(args) | TExprKind::TupleLit(args) => {
                for a in args {
                    self.expr(a);
                }
            }
            TExprKind::CallPtr { callee, args } => {
                self.expr(callee);
                for a in args {
                    self.expr(a);
                }
            }
            TExprKind::Field { base, .. } | TExprKind::RefField { base, .. } | TExprKind::TupleField { base, .. } => self.expr(base),
            TExprKind::Index { base, index, .. } => {
                self.expr(base);
                self.expr(index);
            }
            TExprKind::SliceOp { base, start, end } => {
                self.expr(base);
                if let Some(s) = start {
                    self.expr(s);
                }
                if let Some(x) = end {
                    self.expr(x);
                }
            }
            TExprKind::Deref(x) | TExprKind::Try(x) | TExprKind::OptWrap(x) | TExprKind::ErrWrap(x) | TExprKind::ArrayToSlice(x) | TExprKind::ListToSlice(x) | TExprKind::StrToSlice(x) => self.expr(x),
            TExprKind::AddrOf { expr, .. } | TExprKind::Unary { expr, .. } | TExprKind::Cast { expr, .. } | TExprKind::Unwrap { expr, .. } => self.expr(expr),
            TExprKind::Binary { lhs, rhs, .. } | TExprKind::Logical { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            TExprKind::If { cond, then, els } => {
                self.expr(cond);
                self.block(then);
                if let Some(b) = els {
                    self.block(b);
                }
            }
            TExprKind::IfCapture { cond, then, els, .. } => {
                self.expr(cond);
                self.block(then);
                if let Some(b) = els {
                    self.block(b);
                }
            }
            TExprKind::Match { scrutinee, arms } => {
                self.expr(scrutinee);
                for a in arms {
                    if let Some(g) = &a.guard {
                        self.expr(g);
                    }
                    self.expr(&a.body);
                }
            }
            TExprKind::Block(b) => self.block(b),
            TExprKind::StructLit { fields } => {
                for (_, f) in fields {
                    self.expr(f);
                }
            }
            TExprKind::EnumLit { payload, .. } => {
                for p in payload {
                    self.expr(p);
                }
            }
            TExprKind::Catch { expr, handler, .. } => {
                self.expr(expr);
                self.expr(handler);
            }
            TExprKind::OrElse { expr, default } => {
                self.expr(expr);
                self.expr(default);
            }
            TExprKind::RecordCheck { value, checks, .. } => {
                self.expr(value);
                for (_, c, _) in checks {
                    self.expr(c);
                }
            }
            TExprKind::BinConstruct { segments, target } => {
                self.expr(target);
                for s in segments {
                    if let TBinSegKind::Value(v) = &s.kind {
                        self.expr(v);
                    }
                }
            }
            _ => {}
        }
    }
}
