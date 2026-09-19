//! Statements and blocks.

use super::*;
use crate::ast::*;
use crate::tir::*;
use crate::types::*;

impl<'a> Checker<'a> {
    pub fn push_scope(&mut self) {
        self.cur().scopes.push(Vec::new());
    }
    pub fn pop_scope(&mut self) {
        self.cur().scopes.pop();
    }

    pub fn declare_local(&mut self, name: &str, ty: TyId, mutable: bool, span: Span) -> LocalId {
        let cur = self.cur();
        let id = cur.locals.len() as LocalId;
        cur.locals.push(Local { name: name.to_string(), ty, mutable, span, is_param: false, owned: false });
        cur.scopes.last_mut().unwrap().push((name.to_string(), ScopeEntry { local: id, auto_deref: false }));
        id
    }

    fn new_label(&mut self, name: Option<String>, is_loop: bool, break_ty: Option<TyId>) -> LabelId {
        let cur = self.cur();
        let id = cur.next_label;
        cur.next_label += 1;
        let depth = cur.scopes.len();
        cur.labels.push(LabelInfo { name, id, is_loop, break_ty, scope_depth: depth });
        id
    }

    pub fn check_block(&mut self, b: &Block, expected: Option<TyId>, label: Option<LabelId>) -> TBlock {
        self.check_block_inner(b, expected, label, None)
    }

    pub fn check_block_labeled(&mut self, b: &Block, expected: Option<TyId>, label: Option<String>) -> TBlock {
        match label {
            Some(name) => {
                let ty = expected.unwrap_or_else(|| self.tys.fresh_infer());
                let lid = self.new_label(Some(name), false, Some(ty));
                let tb = self.check_block_inner(b, Some(ty), Some(lid), Some(ty));
                self.cur().labels.pop();
                tb
            }
            None => self.check_block_inner(b, expected, None, None),
        }
    }

    fn check_block_inner(&mut self, b: &Block, expected: Option<TyId>, label: Option<LabelId>, label_ty: Option<TyId>) -> TBlock {
        self.push_scope();
        let mut stmts = Vec::new();
        for s in &b.stmts {
            if let Some(ts) = self.check_stmt(s) {
                stmts.push(ts);
            }
        }
        let mut tail = None;
        let mut ty = self.tys.void();
        if let Some(t) = &b.tail {
            let want = expected.or(label_ty);
            let te = self.check_expr(t, want);
            let tt = self.tys.shallow(te.ty);
            let is_void_expected = matches!(want.map(|w| self.tys.kind(self.tys.shallow(w)).clone()), Some(TyKind::Void));
            if is_void_expected && !matches!(self.tys.kind(tt), TyKind::Void | TyKind::Never) {
                // a trailing expression in a void context must be a statement-like value
                self.check_unused(&te);
                stmts.push(TStmt::Expr(te));
            } else if matches!(self.tys.kind(tt), TyKind::Void) {
                stmts.push(TStmt::Expr(te));
            } else {
                let te = match want {
                    Some(w) if !is_void_expected => self.coerce_or_error(te, w, "block value"),
                    _ => te,
                };
                let te = self.take_ownership(te);
                ty = te.ty;
                tail = Some(Box::new(te));
            }
        }
        if tail.is_none() {
            // a block that diverges has type never
            let diverges = stmts.iter().any(|s| self.stmt_diverges(s));
            if diverges {
                ty = self.tys.never();
            } else if let Some(lt) = label_ty {
                // labeled block without a tail: value comes from `break :label v`
                ty = lt;
            }
        }
        if let (Some(lt), Some(_)) = (label_ty, &tail) {
            ty = lt;
        }
        self.pop_scope();
        TBlock { stmts, tail, label, ty, span: b.span }
    }

    /// Report an unused non-void value in statement position.
    fn check_unused(&mut self, te: &TExpr) {
        if self.repl_mode && self.cur().fn_name == "main" {
            return; // the REPL prints it
        }
        let t = self.tys.shallow(te.ty);
        match self.tys.kind(t).clone() {
            TyKind::Void | TyKind::Never => {}
            TyKind::ErrUnion(_, inner) => {
                let iv = matches!(self.tys.kind(self.tys.shallow(inner)), TyKind::Void);
                let tn = self.type_name(t);
                self.error_note(
                    te.span,
                    format!("unhandled error: this expression has type `{}`", tn),
                    None,
                    if iv { "use `try` to propagate it, `catch` to handle it, or `_ = ` to discard it explicitly" } else { "use `try`, `catch`, or discard it with `_ = `" },
                );
            }
            _ => {
                let tn = self.type_name(t);
                self.error_note(te.span, format!("unused value of type `{}`", tn), None, "discard it explicitly with `_ = ...` (section 4.1)");
            }
        }
    }

    pub fn check_stmt(&mut self, s: &Stmt) -> Option<TStmt> {
        match s {
            Stmt::Let { mutable, name, ty, init, span } => {
                let declared = ty.as_ref().map(|t| {
                    let cur = self.cur.as_ref().unwrap();
                    let (g, st, m) = (cur.generics.clone(), cur.self_ty, cur.module);
                    self.resolve_type(t, &g, st, m)
                });
                let init_e = match init {
                    Some(e) => {
                        let te = self.check_expr(e, declared);
                        let te = match declared {
                            Some(d) => self.coerce_or_error(te, d, &format!("initializer of `{}`", name)),
                            None => te,
                        };
                        let tt = self.tys.shallow(te.ty);
                        if matches!(self.tys.kind(tt), TyKind::Void) {
                            self.error(te.span, format!("`{}` would have type `void`; a `let` must bind a value", name));
                        }
                        if matches!(self.tys.kind(tt), TyKind::Type) {
                            self.error(te.span, "types cannot be bound with `let`; use `type Name = ...` or a `comptime` parameter");
                        }
                        Some(self.take_ownership(te))
                    }
                    None => {
                        if declared.is_none() {
                            self.error(*span, format!("`{}` needs a type or an initializer", name));
                        }
                        if !*mutable {
                            self.error(*span, format!("`let {}` without an initializer can never be assigned; use `var`", name));
                        }
                        None
                    }
                };
                let lty = match (declared, &init_e) {
                    (Some(d), _) => d,
                    (None, Some(e)) => e.ty,
                    (None, None) => self.tys.void(),
                };
                // range fact for literal-initialized immutable locals
                let range = init_e.as_ref().and_then(|e| self.expr_range(e));
                let id = self.declare_local(name, lty, *mutable, *span);
                if let (false, Some(r)) = (*mutable, range) {
                    self.cur().ranges.insert(id, r);
                }
                if self.needs_drop(lty) && !self.is_ref_class(lty) {
                    // owning locals are dropped at scope exit; no effect unless refs involved
                }
                if self.is_ref_class(lty) {
                    self.add_effect(Effects::REFCOUNTS, *span, "holding a reference releases it at scope exit");
                }
                Some(TStmt::Let { local: id, init: init_e, span: *span })
            }
            Stmt::Assign { target, op, value, span } => {
                let t = self.check_expr(target, None);
                if !self.is_place(&t) {
                    self.error(target.span(), "the left side of an assignment must be a variable, field, element, or dereference");
                    let _ = self.check_expr(value, None);
                    return None;
                }
                if !self.place_mutable(&t) {
                    let what = match &t.kind {
                        TExprKind::Local(l) => {
                            let n = self.cur.as_ref().unwrap().locals[*l as usize].name.clone();
                            format!("`{}` is immutable; declare it with `var`", n)
                        }
                        TExprKind::Index { base, .. } => {
                            let bt = self.type_name(base.ty);
                            format!("cannot assign through an immutable `{}`", bt)
                        }
                        TExprKind::Deref(p) => {
                            let pt = self.type_name(p.ty);
                            format!("cannot assign through `{}`; a `*mut` pointer is required", pt)
                        }
                        _ => "cannot assign to an immutable location".to_string(),
                    };
                    self.error(target.span(), what);
                }
                if matches!(t.kind, TExprKind::Global(_)) {
                    self.add_effect(Effects::SHARED_MUTABLE, *span, "writing a mutable global");
                }
                let tt = t.ty;
                match op {
                    None => {
                        let v = self.check_expr(value, Some(tt));
                        let v = self.coerce_or_error(v, tt, "assignment");
                        let v = self.take_ownership(v);
                        // re-assigning a moved local revives it
                        if let TExprKind::Local(l) = &t.kind {
                            let l = *l;
                            self.cur().moved.remove(&l);
                            self.cur().ranges.remove(&l);
                        }
                        Some(TStmt::Assign { target: t, op: None, value: v, span: *span })
                    }
                    Some(bop) => {
                        // compound: type check as a binary op on (target, value)
                        let fake = Expr::Binary { op: *bop, lhs: Box::new(target.clone()), rhs: Box::new(value.clone()), span: *span };
                        let be = self.check_expr(&fake, Some(tt));
                        let (mode, v) = match be.kind {
                            TExprKind::Binary { rhs, mode, .. } => (mode, *rhs),
                            _ => return None,
                        };
                        if let TExprKind::Local(l) = &t.kind {
                            let l = *l;
                            self.cur().ranges.remove(&l);
                        }
                        Some(TStmt::Assign { target: t, op: Some((*bop, mode)), value: v, span: *span })
                    }
                }
            }
            Stmt::Expr(e) => {
                let void = self.tys.void();
                let te = self.check_expr(e, Some(void));
                self.check_unused(&te);
                Some(TStmt::Expr(te))
            }
            Stmt::Return { value, span } => {
                if self.cur().in_defer {
                    self.error(*span, "`return` is not allowed inside `defer`");
                }
                if self.cur().in_parallel > 0 {
                    self.error(*span, "`return` is not allowed inside a `for parallel` body");
                }
                let ret = self.cur().ret;
                let ret_r = self.tys.shallow(ret);
                let ret_is_void = matches!(self.tys.kind(ret_r), TyKind::Void) || matches!(self.tys.kind(ret_r), TyKind::ErrUnion(_, e) if matches!(self.tys.kind(self.tys.shallow(*e)), TyKind::Void));
                match value {
                    None => {
                        if !ret_is_void {
                            let tn = self.type_name(ret);
                            self.error(*span, format!("`return` without a value in a function returning `{}`", tn));
                        }
                        Some(TStmt::Return { value: None, span: *span })
                    }
                    Some(v) => {
                        let te = self.check_expr(v, Some(ret));
                        let tt = self.tys.shallow(te.ty);
                        if ret_is_void && matches!(self.tys.kind(tt), TyKind::Void) {
                            return Some(TStmt::Return { value: Some(te), span: *span });
                        }
                        let te = self.coerce_or_error(te, ret, "return value");
                        self.check_escaping_view(&te, *span);
                        let te = self.take_ownership(te);
                        Some(TStmt::Return { value: Some(te), span: *span })
                    }
                }
            }
            Stmt::Break { label, value, span } => {
                if self.cur().in_parallel > 0 {
                    self.error(*span, "`break` is not allowed inside a `for parallel` body (other workers cannot be stopped); use `continue`");
                }
                let info = self.find_label(label.as_deref(), true, *span)?;
                let v = match value {
                    Some(v) => {
                        let want = info.break_ty;
                        if info.is_loop {
                            self.error(*span, "loops do not produce values; use a labeled block for `break :label value`");
                        }
                        let te = self.check_expr(v, want);
                        let te = match want {
                            Some(w) => self.coerce_or_error(te, w, "break value"),
                            None => te,
                        };
                        Some(self.take_ownership(te))
                    }
                    None => {
                        if !info.is_loop {
                            if let Some(bt) = info.break_ty {
                                let void = self.tys.void();
                                if !self.unify(bt, void) {
                                    self.error(*span, "this labeled block produces a value; `break :label` needs one");
                                }
                            }
                        }
                        None
                    }
                };
                Some(TStmt::Break { label: info.id, value: v, span: *span })
            }
            Stmt::Continue { label, span } => {
                let info = self.find_label(label.as_deref(), false, *span)?;
                if !info.is_loop {
                    self.error(*span, "`continue` needs a loop label");
                }
                Some(TStmt::Continue { label: info.id, span: *span })
            }
            Stmt::Defer { body, span } | Stmt::ErrDefer { body, span } => {
                let is_err = matches!(s, Stmt::ErrDefer { .. });
                if is_err {
                    let ret = self.cur().ret;
                    if !matches!(self.tys.kind(self.tys.shallow(ret)), TyKind::ErrUnion(..)) {
                        self.error(*span, "`errdefer` is only meaningful in a function that returns an error union");
                    }
                }
                let saved = self.cur().in_defer;
                self.cur().in_defer = true;
                self.push_scope();
                let tb = self.check_stmt(body);
                self.pop_scope();
                self.cur().in_defer = saved;
                let tb = tb?;
                if let TStmt::Expr(e) = &tb {
                    let _ = e;
                }
                if is_err {
                    Some(TStmt::ErrDefer { body: Box::new(tb), span: *span })
                } else {
                    Some(TStmt::Defer { body: Box::new(tb), span: *span })
                }
            }
            Stmt::While { cond, body, label, span } => {
                let bt = self.tys.bool();
                let c = self.check_expr(cond, Some(bt));
                let c = self.coerce_or_error(c, bt, "while condition");
                let lid = self.new_label(label.clone(), true, None);
                self.cur().loop_depth += 1;
                let void = self.tys.void();
                let tb = self.check_block(body, Some(void), None);
                self.cur().loop_depth -= 1;
                self.cur().labels.pop();
                Some(TStmt::While { cond: c, body: tb, label: lid, span: *span })
            }
            Stmt::For { iter, bindings, body, label, parallel, span } => self.check_for(iter, bindings, body, label.clone(), *parallel, *span),
            Stmt::Using { strategy, body, span } => {
                let void = self.tys.void();
                let tb = self.check_block(body, Some(void), None);
                self.add_effect(Effects::ALLOCATES, *span, "installing an arena allocates its first chunk");
                Some(TStmt::Using { strategy: strategy.clone(), body: tb, span: *span })
            }
            Stmt::Unsafe { body, span } => {
                self.cur().unsafe_depth += 1;
                let void = self.tys.void();
                let tb = self.check_block(body, Some(void), None);
                self.cur().unsafe_depth -= 1;
                let _ = span;
                Some(TStmt::Block(tb))
            }
            Stmt::Discard { value, span } => {
                let te = self.check_expr(value, None);
                let t = self.tys.shallow(te.ty);
                if matches!(self.tys.kind(t), TyKind::Void) {
                    self.warn(*span, "`_ =` on a void expression has no effect");
                }
                // discarding an owned resource drops it immediately
                Some(TStmt::Expr(te))
            }
        }
    }

    /// Region rule R1, conservatively: a returned slice or pointer must not be a
    /// view into a local variable (array, List, String, or a struct holding
    /// one) that is released when the function returns.
    pub fn check_escaping_view(&mut self, te: &TExpr, span: Span) {
        let t = self.tys.resolve(te.ty, false);
        let is_view = |c: &Self, t: TyId| -> bool {
            let t = c.tys.shallow(t);
            match c.tys.kind(t).clone() {
                TyKind::Slice(..) | TyKind::Ptr(..) => true,
                TyKind::Opt(e) | TyKind::ErrUnion(_, e) => matches!(c.tys.kind(c.tys.shallow(e)), TyKind::Slice(..) | TyKind::Ptr(..)),
                _ => false,
            }
        };
        if !is_view(self, t) {
            return;
        }
        if let Some((local, what)) = self.view_root(te) {
            let cur = self.cur.as_ref().unwrap();
            let l = &cur.locals[local as usize];
            if l.is_param {
                return; // views into parameters outlive the call (they belong to the caller)
            }
            let lt = self.tys.resolve(l.ty, false);
            let owns = matches!(
                self.tys.kind(lt).clone(),
                TyKind::Array(..)
                    | TyKind::List(_)
                    | TyKind::Str
                    | TyKind::Map(..)
                    | TyKind::Struct(..)
                    | TyKind::Tuple(_)
                    | TyKind::Enum(..)
                    | TyKind::Int(_)
                    | TyKind::Float(_)
                    | TyKind::Bool
                    | TyKind::Char
            );
            if owns {
                let name = l.name.clone();
                let lspan = l.span;
                self.error_note(
                    span,
                    format!("this returns a {} into `{}`, a local that is released when the function returns (region rule R1)", what, name),
                    Some(lspan),
                    "declared here; return an owned value (`.clone()`, `.to_owned()`) or take the storage as a parameter",
                );
            }
        }
    }

    /// The local a view expression borrows from, if any, and what kind of view it is.
    fn view_root(&self, e: &TExpr) -> Option<(LocalId, &'static str)> {
        match &e.kind {
            TExprKind::ArrayToSlice(inner) | TExprKind::ListToSlice(inner) | TExprKind::StrToSlice(inner) => self.place_root(inner).map(|l| (l, "slice")),
            TExprKind::SliceOp { base, .. } => match &base.kind {
                TExprKind::ArrayToSlice(inner) | TExprKind::ListToSlice(inner) | TExprKind::StrToSlice(inner) => self.place_root(inner).map(|l| (l, "slice")),
                _ => self.place_root(base).map(|l| (l, "slice")),
            },
            TExprKind::AddrOf { expr, .. } => self.place_root(expr).map(|l| (l, "pointer")),
            TExprKind::OptWrap(inner) | TExprKind::ErrWrap(inner) => self.view_root(inner),
            TExprKind::Block(b) => b.tail.as_ref().and_then(|t| self.view_root(t)),
            TExprKind::If { then, els, .. } => then.tail.as_ref().and_then(|t| self.view_root(t)).or_else(|| els.as_ref().and_then(|b| b.tail.as_ref()).and_then(|t| self.view_root(t))),
            _ => None,
        }
    }

    /// The root local of a place expression (`x`, `x.f`, `x[i]`, `x.0`), not through pointers.
    fn place_root(&self, e: &TExpr) -> Option<LocalId> {
        match &e.kind {
            TExprKind::Local(l) => Some(*l),
            TExprKind::Field { base, .. } | TExprKind::TupleField { base, .. } | TExprKind::Index { base, .. } => {
                // indexing a slice reaches memory the slice borrows, not the local itself
                let bt = self.tys.shallow(base.ty);
                if matches!(self.tys.kind(bt), TyKind::Slice(..) | TyKind::Ptr(..)) {
                    return None;
                }
                self.place_root(base)
            }
            _ => None,
        }
    }

    fn find_label(&mut self, name: Option<&str>, is_break: bool, span: Span) -> Option<LabelInfo> {
        let cur = self.cur.as_ref().unwrap();
        let found = match name {
            Some(n) => cur.labels.iter().rev().find(|l| l.name.as_deref() == Some(n)).cloned(),
            None => cur.labels.iter().rev().find(|l| l.is_loop).cloned(),
        };
        match found {
            Some(l) => Some(l),
            None => {
                let what = if is_break { "break" } else { "continue" };
                match name {
                    Some(n) => self.error(span, format!("no enclosing loop or block labeled `{}`", n)),
                    None => self.error(span, format!("`{}` outside of a loop", what)),
                }
                None
            }
        }
    }

    fn check_for(&mut self, iter: &ForIter, bindings: &[String], body: &Block, label: Option<String>, parallel: bool, span: Span) -> Option<TStmt> {
        match iter {
            ForIter::Range { start, end } => {
                if bindings.len() != 1 {
                    self.error(span, "a range loop binds exactly one variable: `for (a..b) |i|`");
                }
                let hint = self.tys.usize();
                let s = self.check_expr(start, None);
                let st = self.tys.shallow(s.ty);
                let s = if matches!(self.tys.kind(st), TyKind::Infer(_)) {
                    // untyped literal start: take the end's type, defaulting to usize
                    let e = self.check_expr(end, Some(hint));
                    let et = self.tys.resolve(e.ty, false);
                    let s = self.coerce_or_error(s, et, "range start");
                    (s, e)
                } else {
                    let e = self.check_expr(end, Some(st));
                    let e = self.coerce_or_error(e, st, "range end");
                    (s, e)
                };
                let (s, e) = s;
                let it = self.tys.resolve(s.ty, true);
                if !self.tys.is_integer(it) {
                    let tn = self.type_name(it);
                    self.error(span, format!("range bounds must be integers but found `{}`", tn));
                }
                self.push_scope();
                let var = self.declare_local(&bindings[0], it, false, span);
                // range fact: start <= i < end
                if let (Some((slo, _)), Some((_, ehi))) = (self.expr_range(&s), self.expr_range(&e)) {
                    if ehi > slo {
                        self.cur().ranges.insert(var, (slo, ehi - 1));
                    }
                }
                // `for (0..xs.len) |i|` => i indexes xs safely
                if let TExprKind::Builtin { op: Builtin::Len, args, .. } = &e.kind {
                    if let TExprKind::Local(sl) = &args[0].kind {
                        let sl = *sl;
                        if matches!(s.kind, TExprKind::Int(0)) {
                            self.cur().index_of.insert(var, sl);
                        }
                    }
                }
                let lid = self.new_label(label, true, None);
                self.cur().loop_depth += 1;
                let void = self.tys.void();
                let tb = self.check_block(body, Some(void), None);
                self.cur().loop_depth -= 1;
                self.cur().labels.pop();
                self.pop_scope();
                if parallel {
                    self.error(span, "`for parallel` iterates slices: write `for parallel (items) |x, i| { ... }`");
                }
                Some(TStmt::ForRange { var, start: s, end: e, body: tb, label: lid, span })
            }
            ForIter::Items(items) => {
                // bindings: one per item, plus an optional trailing index
                let n_items = items.len();
                if !(bindings.len() == n_items || bindings.len() == n_items + 1) {
                    self.error(span, format!("this loop iterates {} value(s) and may bind an index; expected {} or {} names but found {}", n_items, n_items, n_items + 1, bindings.len()));
                    return None;
                }
                let mut checked = Vec::new();
                for it in items {
                    let te = self.check_expr(it, None);
                    let mut te = te;
                    let mut t = self.tys.shallow(te.ty);
                    while let TyKind::Ptr(_, inner) = self.tys.kind(t).clone() {
                        te = TExpr { kind: TExprKind::Deref(Box::new(te)), ty: inner, span: it.span() };
                        t = self.tys.shallow(inner);
                    }
                    let elem = match self.tys.kind(t).clone() {
                        TyKind::Array(_, e) | TyKind::Slice(_, e) | TyKind::List(e) => e,
                        TyKind::Str => self.tys.u8(),
                        TyKind::Map(k, _) => {
                            // iterate keys
                            let kt = self.tys.slice(false, k);
                            let ks = self.mk(TExprKind::Builtin { op: Builtin::MapKeys, args: vec![te], tys: vec![k] }, kt, it.span());
                            self.add_effect(Effects::ALLOCATES, it.span(), "iterating a Map collects its keys");
                            checked.push((ks, k));
                            continue;
                        }
                        _ => {
                            let tn = self.type_name(t);
                            self.error(it.span(), format!("cannot iterate a value of type `{}`; expected an array, slice, List, String, or Map", tn));
                            return None;
                        }
                    };
                    // convert arrays/lists/strings to slices for uniform iteration
                    let te = match self.tys.kind(t).clone() {
                        TyKind::Array(..) => {
                            let m = self.place_mutable(&te);
                            let st = self.tys.slice(m, elem);
                            TExpr { kind: TExprKind::ArrayToSlice(Box::new(te)), ty: st, span: it.span() }
                        }
                        TyKind::List(_) => {
                            let m = self.place_mutable(&te);
                            let st = self.tys.slice(m, elem);
                            TExpr { kind: TExprKind::ListToSlice(Box::new(te)), ty: st, span: it.span() }
                        }
                        TyKind::Str => {
                            let st = self.tys.slice(false, elem);
                            TExpr { kind: TExprKind::StrToSlice(Box::new(te)), ty: st, span: it.span() }
                        }
                        _ => te,
                    };
                    checked.push((te, elem));
                }
                self.push_scope();
                let mut item_locals = Vec::new();
                for (i, (te, elem)) in checked.into_iter().enumerate() {
                    let l = self.declare_local(&bindings[i], elem, false, span);
                    item_locals.push((l, te));
                }
                let index = if bindings.len() == n_items + 1 {
                    let u = self.tys.usize();
                    let l = self.declare_local(&bindings[n_items], u, false, span);
                    // the index is in range for every iterated local slice
                    for (_, te) in &item_locals {
                        if let TExprKind::Local(sl) = &te.kind {
                            let sl = *sl;
                            self.cur().index_of.insert(l, sl);
                        }
                        if let TExprKind::ListToSlice(inner) | TExprKind::ArrayToSlice(inner) = &te.kind {
                            if let TExprKind::Local(sl) = &inner.kind {
                                let sl = *sl;
                                self.cur().index_of.insert(l, sl);
                            }
                        }
                    }
                    Some(l)
                } else {
                    None
                };
                let lid = self.new_label(label, true, None);
                self.cur().loop_depth += 1;
                let void = self.tys.void();
                // a parallel body must not touch shared mutable state (spec 7.2)
                let w_before = self.cur().witnesses.len();
                let c_before = self.cur().callees.len();
                if parallel {
                    self.cur().in_parallel += 1;
                }
                let tb = self.check_block(body, Some(void), None);
                if parallel {
                    self.cur().in_parallel -= 1;
                    let ws: Vec<(Effects, Span, String)> = self.cur().witnesses[w_before..].to_vec();
                    for (e, sp, why) in ws {
                        if e.contains(Effects::SHARED_MUTABLE) {
                            self.error_note(sp, "a `for parallel` body must not have the `shared_mutable` effect", Some(span), format!("introduced here: {}", why));
                        }
                    }
                    let cs: Vec<(InstId, Span)> = self.cur().callees[c_before..].to_vec();
                    self.parallel_obligations.extend(cs);
                    self.add_effect(Effects::BLOCKS, span, "`for parallel` joins its worker threads");
                }
                self.cur().loop_depth -= 1;
                self.cur().labels.pop();
                self.pop_scope();
                Some(TStmt::ForSlice { items: item_locals, index, body: tb, label: lid, span, parallel })
            }
        }
    }
}
