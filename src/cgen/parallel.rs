//! `for parallel` (spec 7.2): the loop body is extracted into a worker function
//! and run over index ranges by the runtime's thread pool. Every local the body
//! reads or writes is reached through an environment of pointers, so the body
//! compiles unchanged; the checker guarantees it has no `shared_mutable`
//! effect and no `return`/`break`.

use super::*;
use crate::diag::Span;
use std::collections::BTreeSet;

/// Locals a block uses that are declared outside of it.
pub fn free_locals(b: &TBlock) -> Vec<LocalId> {
    let mut used = BTreeSet::new();
    let mut declared = BTreeSet::new();
    walk_block(b, &mut used, &mut declared);
    used.difference(&declared).copied().collect()
}

fn walk_block(b: &TBlock, used: &mut BTreeSet<LocalId>, declared: &mut BTreeSet<LocalId>) {
    for s in &b.stmts {
        walk_stmt(s, used, declared);
    }
    if let Some(t) = &b.tail {
        walk_expr(t, used, declared);
    }
}

fn walk_stmt(s: &TStmt, used: &mut BTreeSet<LocalId>, declared: &mut BTreeSet<LocalId>) {
    match s {
        TStmt::Let { local, init, .. } => {
            declared.insert(*local);
            if let Some(e) = init {
                walk_expr(e, used, declared);
            }
        }
        TStmt::Assign { target, value, .. } => {
            walk_expr(target, used, declared);
            walk_expr(value, used, declared);
        }
        TStmt::Expr(e) | TStmt::Return { value: Some(e), .. } | TStmt::Break { value: Some(e), .. } => walk_expr(e, used, declared),
        TStmt::Defer { body, .. } | TStmt::ErrDefer { body, .. } => walk_stmt(body, used, declared),
        TStmt::While { cond, body, els, .. } => {
            walk_expr(cond, used, declared);
            walk_block(body, used, declared);
            if let Some(eb) = els {
                walk_block(eb, used, declared);
            }
        }
        TStmt::ForRange { var, start, end, step, body, .. } => {
            declared.insert(*var);
            walk_expr(start, used, declared);
            walk_expr(end, used, declared);
            if let Some(st) = step {
                walk_expr(st, used, declared);
            }
            walk_block(body, used, declared);
        }
        TStmt::ForSlice { items, index, body, .. } => {
            for (l, e) in items {
                declared.insert(*l);
                walk_expr(e, used, declared);
            }
            if let Some(i) = index {
                declared.insert(*i);
            }
            walk_block(body, used, declared);
        }
        TStmt::Block(b) | TStmt::Using { body: b, .. } => walk_block(b, used, declared),
        TStmt::Drop { local, .. } => {
            used.insert(*local);
        }
        _ => {}
    }
}

fn walk_pat(p: &TPat, declared: &mut BTreeSet<LocalId>) {
    match p {
        TPat::Bind(l) => {
            declared.insert(*l);
        }
        TPat::Variant { args, .. } | TPat::Or(args) | TPat::Tuple(args) => {
            for a in args {
                walk_pat(a, declared);
            }
        }
        TPat::Some(inner) | TPat::Ok(inner) => walk_pat(inner, declared),
        TPat::Binary(segs) => {
            for s in segs {
                if let TBinSegKind::Bind(l) | TBinSegKind::Rest(l) = &s.kind {
                    declared.insert(*l);
                }
            }
        }
        _ => {}
    }
}

fn walk_expr(e: &TExpr, used: &mut BTreeSet<LocalId>, declared: &mut BTreeSet<LocalId>) {
    match &e.kind {
        TExprKind::Local(l) => {
            used.insert(*l);
        }
        TExprKind::Field { base, .. } | TExprKind::RefField { base, .. } | TExprKind::TupleField { base, .. } => walk_expr(base, used, declared),
        TExprKind::Index { base, index, .. } => {
            walk_expr(base, used, declared);
            walk_expr(index, used, declared);
        }
        TExprKind::SliceOp { base, start, end } => {
            walk_expr(base, used, declared);
            if let Some(s) = start {
                walk_expr(s, used, declared);
            }
            if let Some(x) = end {
                walk_expr(x, used, declared);
            }
        }
        TExprKind::Deref(x)
        | TExprKind::Try(x)
        | TExprKind::OptWrap(x)
        | TExprKind::ErrWrap(x)
        | TExprKind::ErrToUnion(x)
        | TExprKind::ArrayToSlice(x)
        | TExprKind::ListToSlice(x)
        | TExprKind::StrToSlice(x)
        | TExprKind::Retained(x) => walk_expr(x, used, declared),
        TExprKind::AddrOf { expr, .. } | TExprKind::Unary { expr, .. } | TExprKind::Cast { expr, .. } | TExprKind::Unwrap { expr, .. } | TExprKind::DynFrom { expr, .. } => {
            walk_expr(expr, used, declared)
        }
        TExprKind::Call { args, .. } | TExprKind::Builtin { args, .. } | TExprKind::ArrayLit(args) | TExprKind::TupleLit(args) => {
            for a in args {
                walk_expr(a, used, declared);
            }
        }
        TExprKind::CallPtr { callee, args } => {
            walk_expr(callee, used, declared);
            for a in args {
                walk_expr(a, used, declared);
            }
        }
        TExprKind::DynCall { recv, args, .. } => {
            walk_expr(recv, used, declared);
            for a in args {
                walk_expr(a, used, declared);
            }
        }
        TExprKind::Binary { lhs, rhs, .. } | TExprKind::Logical { lhs, rhs, .. } => {
            walk_expr(lhs, used, declared);
            walk_expr(rhs, used, declared);
        }
        TExprKind::If { cond, then, els } => {
            walk_expr(cond, used, declared);
            walk_block(then, used, declared);
            if let Some(b) = els {
                walk_block(b, used, declared);
            }
        }
        TExprKind::IfCapture { cond, local, then, els } => {
            walk_expr(cond, used, declared);
            declared.insert(*local);
            walk_block(then, used, declared);
            if let Some(b) = els {
                walk_block(b, used, declared);
            }
        }
        TExprKind::Match { scrutinee, arms } => {
            walk_expr(scrutinee, used, declared);
            for a in arms {
                walk_pat(&a.pat, declared);
                if let Some(g) = &a.guard {
                    walk_expr(g, used, declared);
                }
                walk_expr(&a.body, used, declared);
            }
        }
        TExprKind::Block(b) => walk_block(b, used, declared),
        TExprKind::StructLit { fields } | TExprKind::RefNew { fields } => {
            for (_, f) in fields {
                walk_expr(f, used, declared);
            }
        }
        TExprKind::EnumLit { payload, .. } => {
            for p in payload {
                walk_expr(p, used, declared);
            }
        }
        TExprKind::Catch { expr, err_local, handler } => {
            walk_expr(expr, used, declared);
            if let Some(l) = err_local {
                declared.insert(*l);
            }
            walk_expr(handler, used, declared);
        }
        TExprKind::OrElse { expr, default } => {
            walk_expr(expr, used, declared);
            walk_expr(default, used, declared);
        }
        TExprKind::Closure { captures, .. } => {
            for (l, _) in captures {
                used.insert(*l);
            }
        }
        TExprKind::RecordCheck { value, checks, .. } => {
            walk_expr(value, used, declared);
            for (_, c, l) in checks {
                declared.insert(*l);
                walk_expr(c, used, declared);
            }
        }
        TExprKind::BinConstruct { segments, target } => {
            walk_expr(target, used, declared);
            for s in segments {
                if let TBinSegKind::Value(v) = &s.kind {
                    walk_expr(v, used, declared);
                }
                if let TBinSize::Expr(x) = &s.size {
                    walk_expr(x, used, declared);
                }
            }
        }
        _ => {}
    }
}

impl Gen {
    /// Lower a parallel slice loop: evaluate the slices, build the environment,
    /// emit the worker function, and dispatch through the runtime.
    pub fn parallel_for(&mut self, items: &[(LocalId, TExpr)], index: &Option<LocalId>, body: &TBlock, label: LabelId, span: Span) {
        // slices are evaluated once in the parent
        let mut slices = Vec::new();
        for (l, e) in items {
            let sc = self.expr(e);
            let st = self.bind_tmp(&sc, e.ty);
            slices.push((*l, st, e.ty));
        }
        let first = slices[0].1.clone();
        let loc = self.loc(span);
        for (_, s, _) in slices.iter().skip(1) {
            self.line(format!("if ({}.len != {}.len) nx_panic(\"parallel iteration over slices of different lengths\", {});", s, first, loc));
        }
        // free locals of the body become pointers in the environment
        let mut free = free_locals(body);
        let item_ids: Vec<LocalId> = items.iter().map(|(l, _)| *l).collect();
        free.retain(|l| !item_ids.contains(l) && Some(*l) != *index);
        let n = self.tmp();
        let env_ty = format!("nx_parenv_{}", n);
        let fn_name = format!("nx_par_{}", n);
        let mut fields = String::new();
        for (k, (_, _, ty)) in slices.iter().enumerate() {
            let cn = self.cty(*ty);
            let _ = write!(fields, " {} s{};", cn, k);
        }
        for &l in &free {
            let ty = self.st().local_tys[l as usize];
            let cn = self.cty(ty);
            let _ = write!(fields, " {}* l{};", cn, l);
        }
        let _ = writeln!(self.types_out, "typedef struct {n} {{{f} char _pad; }} {n};", n = env_ty, f = fields);
        let env = self.tmp();
        let mut inits = Vec::new();
        for (k, (_, s, _)) in slices.iter().enumerate() {
            inits.push(format!(".s{} = {}", k, s));
        }
        for &l in &free {
            let name = self.local_name_pub(l);
            inits.push(format!(".l{} = &({})", l, name));
        }
        self.line(format!("{} {} = {{ {} }};", env_ty, env, inits.join(", ")));
        self.line(format!("nx_parallel_for(c, {}.len, {}, &{}, {});", first, fn_name, env, loc));

        // emit the worker function with its own state, mapping free locals through the environment
        let saved_cur = self.cur.take();
        let saved_body = std::mem::take(&mut self.body);
        let saved_tmp = self.tmp;
        let parent = saved_cur.as_ref().unwrap();
        let mut locals = parent.locals.clone();
        for &l in &free {
            locals[l as usize] = format!("(*_penv->l{})", l);
        }
        self.cur = Some(FnState { inst: parent.inst, locals, local_tys: parent.local_tys.clone(), scopes: vec![], ret: parent.ret, loop_labels: vec![], block_labels: vec![], is_test: false });
        self.body = vec![String::new()];
        self.line(format!("{}* _penv = ({}*)_pv; NX_UNUSED(_penv);", env_ty, env_ty));
        let idx = match index {
            Some(i) => self.local_name_pub(*i),
            None => self.tmp(),
        };
        self.st().loop_labels.push((label, 0));
        self.line(format!("for (size_t {} = _b; {} < _e; {}++) {{", idx, idx, idx));
        self.push_buf();
        for (k, (l, _, _)) in slices.iter().enumerate() {
            let name = self.local_name_pub(*l);
            let ty = self.st().local_tys[*l as usize];
            let cn = self.cty(ty);
            self.line(format!("{} {} = _penv->s{}.ptr[{}];", cn, name, k, idx));
        }
        self.push_scope_pub(true);
        self.block_stmt_pub(body);
        self.pop_scope_emit_pub();
        let inner = self.pop_buf();
        self.body.last_mut().unwrap().push_str(&inner);
        self.line(format!("  nx_cont_{}: ;", label));
        self.line("}");
        let code = self.body.pop().unwrap();
        let _ = writeln!(self.protos_out, "static void {}(nx_ctx* c, void* _pv, size_t _b, size_t _e);", fn_name);
        let _ = writeln!(self.funcs_out, "static void {}(nx_ctx* c, void* _pv, size_t _b, size_t _e) {{\n  NX_UNUSED(c);\n{}}}\n", fn_name, code);
        self.cur = saved_cur;
        self.body = saved_body;
        self.tmp = saved_tmp.max(self.tmp);
    }
}
