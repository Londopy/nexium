//! Lowering of function bodies, statements, and expressions to C.

use super::*;
use crate::ast::{BinOp, UnOp};
use crate::diag::Span;

impl Gen {
    pub fn st(&mut self) -> &mut FnState {
        self.cur.as_mut().unwrap()
    }

    pub fn line(&mut self, s: impl AsRef<str>) {
        let depth = self.body.len();
        let buf = self.body.last_mut().unwrap();
        for _ in 0..depth {
            buf.push_str("  ");
        }
        buf.push_str(s.as_ref());
        buf.push('\n');
    }

    pub fn push_buf(&mut self) {
        self.body.push(String::new());
    }
    pub fn pop_buf(&mut self) -> String {
        self.body.pop().unwrap()
    }

    pub fn loc(&self, span: Span) -> String {
        let file = self.opts.source_names.get(span.file as usize).cloned().unwrap_or_else(|| "?".into());
        let line = self.sm_line(span);
        format!("\"{}:{}\"", file.replace('\\', "/"), line)
    }

    fn sm_line(&self, span: Span) -> usize {
        // the line table is computed by the driver into sm_names as "file\nline-starts"; fall back to byte offset
        if let Some(starts) = self.line_starts.get(span.file as usize) {
            return match starts.binary_search(&span.start) {
                Ok(i) => i + 1,
                Err(i) => i,
            };
        }
        span.start as usize
    }

    fn local_name(&self, l: LocalId) -> String {
        self.cur.as_ref().unwrap().locals[l as usize].clone()
    }

    pub fn is_simple(&self, e: &TExpr) -> bool {
        match &e.kind {
            TExprKind::Local(_)
            | TExprKind::Global(_)
            | TExprKind::Const(_)
            | TExprKind::Int(_)
            | TExprKind::Float(_)
            | TExprKind::Bool(_)
            | TExprKind::Char(_)
            | TExprKind::Str(_)
            | TExprKind::Unit => true,
            TExprKind::Field { base, .. } | TExprKind::TupleField { base, .. } | TExprKind::RefField { base, .. } => self.is_simple(base),
            TExprKind::Deref(p) => self.is_simple(p),
            _ => false,
        }
    }

    /// Emit an expression, binding it to a temporary unless it is already simple.
    /// Emit an expression bound to a temporary unless it is already simple. A
    /// temporary that owns resources is released when the enclosing scope ends
    /// (the value is only borrowed by whatever reads it).
    pub fn simple(&mut self, e: &TExpr) -> String {
        let c = self.expr(e);
        if self.is_simple(e) {
            return c;
        }
        let ty = e.ty;
        let t = self.bind_tmp(&c, ty);
        if self.is_owned_temp(e) {
            self.register_drop(&t, ty);
        }
        t
    }

    /// Like `simple`, but the caller takes ownership of the temporary's contents.
    pub fn simple_owned(&mut self, e: &TExpr) -> String {
        let c = self.expr(e);
        if self.is_simple(e) {
            return c;
        }
        let ty = e.ty;
        self.bind_tmp(&c, ty)
    }

    pub fn bind_tmp(&mut self, c: &str, ty: TyId) -> String {
        let t = self.tmp();
        if self.is_void(ty) {
            self.line(format!("{};", c));
            return "0".into();
        }
        let cn = self.cty(ty);
        self.line(format!("{} {} = {};", cn, t, c));
        t
    }

    // ----- functions -----------------------------------------------------------

    pub fn emit_function(&mut self, inst: InstId) {
        let f = self.p.funcs[inst as usize].clone();
        let sig = self.fn_signature(inst, true);
        let locals: Vec<String> = f.locals.iter().enumerate().map(|(i, l)| local_c_name(&l.name, i as LocalId)).collect();
        let local_tys: Vec<TyId> = f.locals.iter().map(|l| l.ty).collect();
        self.cur = Some(FnState { inst, locals, local_tys, scopes: vec![], ret: f.ret, loop_labels: vec![], block_labels: vec![], is_test: f.is_test });
        self.body = vec![String::new()];
        self.tmp = 0;
        self.line("NX_UNUSED(c);");
        // closure captures
        if f.is_closure {
            let env = self.p.closure_envs.get(&inst).cloned().unwrap_or_default();
            let env_ty = self.closure_env_type(inst);
            self.line(format!("{}* _env = ({}*)_envp;", env_ty, env_ty));
            let n_params = f.params.len();
            for (i, (t, by_ref)) in env.iter().enumerate() {
                let lid = (n_params + i) as LocalId;
                let name = self.local_name(lid);
                if *by_ref {
                    let cn = self.cty(*t);
                    self.line(format!("{}* {} = _env->c{};", cn, name, i));
                } else {
                    let cn = self.cty(*t);
                    self.line(format!("{} {} = _env->c{};", cn, name, i));
                }
            }
        }
        let body = f.body.clone().unwrap();
        self.push_scope(false);
        // `own` parameters belong to this function now: drop them at exit (unless moved)
        for &p in &f.params {
            if f.locals[p as usize].owned {
                let name = self.local_name(p);
                self.register_drop(&name, f.locals[p as usize].ty);
            }
        }
        let v = self.block_value(&body);
        // tail value of the body: return it
        if let Some(v) = v {
            if !self.is_void(f.ret) {
                self.emit_return(Some(&v), body.span);
            }
        }
        // a function returning `!void` (or void) falls off the end successfully
        let rk = self.kind_of(f.ret);
        let void_payload = match rk {
            TyKind::ErrUnion(_, e) => self.is_void(e),
            _ => false,
        };
        if void_payload {
            self.emit_return(None, body.span);
        } else {
            self.pop_scope_emit();
        }
        if void_payload {
            self.st().scopes.pop();
        }
        let code = self.body.pop().unwrap();
        let _ = writeln!(self.funcs_out, "{} {{\n{}}}\n", sig, code);
        self.cur = None;
    }

    pub fn closure_env_type(&mut self, inst: InstId) -> String {
        let name = format!("nx_env_{}", inst);
        if self.type_names.values().any(|n| *n == name) {
            return name;
        }
        let env = self.p.closure_envs.get(&inst).cloned().unwrap_or_default();
        let mut fields = String::new();
        for (i, (t, by_ref)) in env.iter().enumerate() {
            let cn = self.cty(*t);
            let _ = write!(fields, " {}{} c{};", cn, if *by_ref { "*" } else { "" }, i);
        }
        if env.is_empty() {
            fields.push_str(" char _e;");
        }
        let _ = writeln!(self.types_out, "typedef struct {n} {{{f} }} {n};", n = name, f = fields);
        // record so we do not emit twice (use a synthetic key)
        let key = u32::MAX - inst;
        self.type_names.insert(key, name.clone());
        name
    }

    // ----- scopes ----------------------------------------------------------------

    fn push_scope(&mut self, is_loop_body: bool) {
        self.st().scopes.push(Scope { defers: vec![], drops: vec![], is_loop_body });
    }

    /// Emit the exit actions of the innermost scope and pop it.
    fn pop_scope_emit(&mut self) {
        let n = self.st().scopes.len() - 1;
        self.emit_scope_exit(n, false);
        self.st().scopes.pop();
    }

    /// Emit defers (and drops) for scope index `i`; `is_err` selects errdefers too.
    fn emit_scope_exit(&mut self, i: usize, is_err: bool) {
        let scope_defers = self.st().scopes[i].defers.clone();
        let drops = self.st().scopes[i].drops.clone();
        for (err_only, stmt) in scope_defers.iter().rev() {
            if *err_only && !is_err {
                continue;
            }
            self.stmt(stmt);
        }
        for (name, ty) in drops.iter().rev() {
            let d = self.drop_fn(*ty);
            self.line(format!("{}(c, &{});", d, name));
        }
    }

    /// Emit exits for all scopes from innermost down to (and including) index `to`.
    fn emit_exits_down_to(&mut self, to: usize, is_err: bool) {
        let n = self.st().scopes.len();
        for i in (to..n).rev() {
            self.emit_scope_exit(i, is_err);
        }
    }

    fn register_drop(&mut self, name: &str, ty: TyId) {
        if self.needs_drop(ty) {
            self.st().scopes.last_mut().unwrap().drops.push((name.to_string(), ty));
        }
    }

    // ----- statements ------------------------------------------------------------

    fn emit_return(&mut self, value: Option<&str>, _span: Span) {
        let ret = self.st().ret;
        let is_eu = matches!(self.kind_of(ret), TyKind::ErrUnion(..));
        let has_err_defers = self.st().scopes.iter().any(|s| s.defers.iter().any(|(e, _)| *e));
        match value {
            Some(v) if !self.is_void(ret) => {
                let cn = self.cty(ret);
                let t = self.tmp();
                self.line(format!("{} {} = {};", cn, t, v));
                if is_eu && has_err_defers {
                    self.line(format!("if ({}.err) {{", t));
                    self.body.push(String::new());
                    self.emit_exits_down_to(0, true);
                    let inner = self.pop_buf();
                    self.body.last_mut().unwrap().push_str(&inner);
                    self.line("} else {");
                    self.body.push(String::new());
                    self.emit_exits_down_to(0, false);
                    let inner = self.pop_buf();
                    self.body.last_mut().unwrap().push_str(&inner);
                    self.line("}");
                } else {
                    self.emit_exits_down_to(0, false);
                }
                self.line(format!("return {};", t));
            }
            _ => {
                if let Some(v) = value {
                    if v != "0" {
                        self.line(format!("{};", v));
                    }
                }
                self.emit_exits_down_to(0, false);
                if is_eu {
                    let cn = self.cty(ret);
                    self.line(format!("return ({}){{ .err = 0 }};", cn));
                } else {
                    self.line("return;");
                }
            }
        }
    }

    pub fn stmt(&mut self, s: &TStmt) {
        match s {
            TStmt::Let { local, init, .. } => {
                let name = self.local_name(*local);
                let ty = self.st().local_tys[*local as usize];
                let cn = self.cty(ty);
                match init {
                    Some(TExpr { kind: TExprKind::Undefined, .. }) => {
                        self.line(format!("{} {};", cn, name));
                        if self.opts.mode == BuildMode::Debug {
                            self.line(format!("memset(&{}, 0xAA, sizeof {});", name, name));
                        }
                    }
                    Some(e) => {
                        let v = self.expr_owned(e);
                        if self.is_void(ty) {
                            self.line(format!("{};", v));
                        } else {
                            self.line(format!("{} {} = {};", cn, name, v));
                        }
                    }
                    None => {
                        self.line(format!("{} {} = {{0}};", cn, name));
                    }
                }
                self.register_drop(&name, ty);
            }
            TStmt::Assign { target, op, value, span } => {
                match op {
                    None => {
                        let v = self.expr_owned(value);
                        let ty = target.ty;
                        if self.needs_drop(ty) {
                            let t = self.tmp();
                            let cn = self.cty(ty);
                            self.line(format!("{} {} = {};", cn, t, v));
                            let place = self.place(target);
                            let d = self.drop_fn(ty);
                            self.line(format!("{}(c, &({}));", d, place));
                            self.line(format!("{} = {};", place, t));
                        } else {
                            let place = self.place(target);
                            self.line(format!("{} = {};", place, v));
                        }
                    }
                    Some((bop, mode)) => {
                        let place = self.place(target);
                        // evaluate the place once
                        let pt = self.cty(target.ty);
                        let pp = self.tmp();
                        self.line(format!("{}* {} = &({});", pt, pp, place));
                        let v = self.expr(value);
                        let r = self.binop_code(*bop, *mode, &format!("(*{})", pp), &v, target.ty, *span, true);
                        self.line(format!("*{} = {};", pp, r));
                    }
                }
            }
            TStmt::Expr(e) => {
                let is_place_expr = self.is_place_expr(e);
                let v = self.expr(e);
                if self.is_void(e.ty) {
                    if v != "0" && !v.is_empty() {
                        self.line(format!("{};", v));
                    }
                } else if self.needs_drop(e.ty) && !is_place_expr {
                    // an owned temporary discarded immediately: drop it
                    let cn = self.cty(e.ty);
                    let t = self.tmp();
                    self.line(format!("{} {} = {};", cn, t, v));
                    let d = self.drop_fn(e.ty);
                    self.line(format!("{}(c, &{});", d, t));
                } else {
                    self.line(format!("(void)({});", v));
                }
            }
            TStmt::Return { value, span } => match value {
                Some(e) => {
                    let v = self.expr_owned(e);
                    self.emit_return(Some(&v), *span);
                }
                None => self.emit_return(None, *span),
            },
            TStmt::Break { label, value, .. } => {
                // labeled block?
                if let Some((_, brk_label, result_var)) = self.st().block_labels.iter().rev().find(|(l, _, _)| l == label).cloned() {
                    if let Some(v) = value {
                        let vc = self.expr_owned(v);
                        self.line(format!("{} = {};", result_var, vc));
                    }
                    let depth = self.block_scope_depth(*label);
                    self.emit_exits_down_to(depth, false);
                    self.line(format!("goto {};", brk_label));
                    return;
                }
                let depth = self.loop_scope_depth(*label);
                self.emit_exits_down_to(depth, false);
                self.line(format!("goto nx_brk_{};", label));
            }
            TStmt::Continue { label, .. } => {
                let depth = self.loop_scope_depth(*label);
                // the loop body scope is at depth (loop scope + 1); exit those above it
                self.emit_exits_down_to(depth + 1, false);
                self.line(format!("goto nx_cont_{};", label));
            }
            TStmt::Defer { body, .. } => {
                self.st().scopes.last_mut().unwrap().defers.push((false, (**body).clone()));
            }
            TStmt::ErrDefer { body, .. } => {
                self.st().scopes.last_mut().unwrap().defers.push((true, (**body).clone()));
            }
            TStmt::While { cond, body, els, label, .. } => {
                let depth = self.st().scopes.len();
                self.st().loop_labels.push((*label, depth));
                self.line("for (;;) {");
                self.push_buf();
                let c = self.expr(cond);
                self.line(format!("if (!({})) break;", c));
                self.push_scope(true);
                self.block_stmt(body);
                self.pop_scope_emit();
                let inner = self.pop_buf();
                self.body.last_mut().unwrap().push_str(&inner);
                self.line(format!("  nx_cont_{}: ;", label));
                self.line("}");
                self.st().loop_labels.pop();
                // the `else` sits between the loop and its break label: a false
                // condition falls into it, a `break` jumps over it
                if let Some(eb) = els {
                    self.line("{");
                    self.push_scope(true);
                    self.block_stmt(eb);
                    self.pop_scope_emit();
                    self.line("}");
                }
                self.line(format!("nx_brk_{}: ;", label));
            }
            TStmt::ForRange { var, start, end, step, body, label, .. } => {
                let s = self.expr(start);
                let e = self.simple(end);
                let name = self.local_name(*var);
                let ty = self.st().local_tys[*var as usize];
                let cn = self.cty(ty);
                let depth = self.st().scopes.len();
                self.st().loop_labels.push((*label, depth));
                let et = self.tmp();
                self.line(format!("{} {} = {};", cn, et, e));
                match step {
                    None => self.line(format!("for ({} {} = {}; {} < {}; {}++) {{", cn, name, s, name, et, name)),
                    Some(st) => {
                        let sv = self.simple(st);
                        let stt = self.tmp();
                        self.line(format!("{} {} = {};", cn, stt, sv));
                        // a constant step picks its direction at compile time
                        match st.kind {
                            TExprKind::Int(v) if v > 0 => self.line(format!("for ({} {} = {}; {} < {}; {} += {}) {{", cn, name, s, name, et, name, stt)),
                            TExprKind::Int(_) => self.line(format!("for ({} {} = {}; {} > {}; {} += {}) {{", cn, name, s, name, et, name, stt)),
                            _ => self.line(format!("for ({} {} = {}; {} > 0 ? {} < {} : {} > {}; {} += {}) {{", cn, name, s, stt, name, et, name, et, name, stt)),
                        }
                    }
                }
                self.push_buf();
                self.push_scope(true);
                self.block_stmt(body);
                self.pop_scope_emit();
                let inner = self.pop_buf();
                self.body.last_mut().unwrap().push_str(&inner);
                self.line(format!("  nx_cont_{}: ;", label));
                self.line("}");
                self.line(format!("nx_brk_{}: ;", label));
                self.st().loop_labels.pop();
            }
            TStmt::ForSlice { items, index, body, label, span, parallel: true } => {
                self.parallel_for(items, index, body, *label, *span);
            }
            TStmt::ForSlice { items, index, body, label, span, parallel: false } => {
                let mut slices = Vec::new();
                for (l, e) in items {
                    let sc = self.expr(e);
                    let st = self.bind_tmp(&sc, e.ty);
                    slices.push((*l, st));
                }
                let first = slices[0].1.clone();
                for (_, s) in slices.iter().skip(1) {
                    let loc = self.loc(*span);
                    self.line(format!("if ({}.len != {}.len) nx_panic(\"parallel iteration over slices of different lengths\", {});", s, first, loc));
                }
                let idx = match index {
                    Some(i) => self.local_name(*i),
                    None => self.tmp(),
                };
                let depth = self.st().scopes.len();
                self.st().loop_labels.push((*label, depth));
                self.line(format!("for (size_t {} = 0; {} < {}.len; {}++) {{", idx, idx, first, idx));
                self.push_buf();
                for (l, s) in &slices {
                    let name = self.local_name(*l);
                    let ty = self.st().local_tys[*l as usize];
                    let cn = self.cty(ty);
                    self.line(format!("{} {} = {}.ptr[{}];", cn, name, s, idx));
                }
                self.push_scope(true);
                self.block_stmt(body);
                self.pop_scope_emit();
                let inner = self.pop_buf();
                self.body.last_mut().unwrap().push_str(&inner);
                self.line(format!("  nx_cont_{}: ;", label));
                self.line("}");
                self.line(format!("nx_brk_{}: ;", label));
                self.st().loop_labels.pop();
            }
            TStmt::Block(b) => {
                self.line("{");
                self.push_buf();
                self.push_scope(false);
                self.block_stmt(b);
                self.pop_scope_emit();
                let inner = self.pop_buf();
                self.body.last_mut().unwrap().push_str(&inner);
                self.line("}");
            }
            TStmt::Using { body, .. } => {
                // the block sees a context whose allocator is the arena; releases inside are no-ops
                let ar = self.tmp();
                self.line("{");
                self.push_buf();
                self.line(format!("nx_arena {ar}; nx_ctx {ar}_ctx = nx_arena_begin(c, &{ar});", ar = ar));
                self.line("{");
                self.push_buf();
                self.line(format!("nx_ctx* c = &{}_ctx;", ar));
                self.push_scope(false);
                self.block_stmt(body);
                self.pop_scope_emit();
                let inner = self.pop_buf();
                self.body.last_mut().unwrap().push_str(&inner);
                self.line("}");
                self.line(format!("nx_arena_end(&{});", ar));
                let inner = self.pop_buf();
                self.body.last_mut().unwrap().push_str(&inner);
                self.line("}");
            }
            TStmt::Drop { local, .. } => {
                let name = self.local_name(*local);
                let ty = self.st().local_tys[*local as usize];
                let d = self.drop_fn(ty);
                self.line(format!("{}(c, &{});", d, name));
            }
        }
    }

    pub fn local_name_pub(&self, l: LocalId) -> String {
        self.local_name(l)
    }
    pub fn push_scope_pub(&mut self, is_loop_body: bool) {
        self.push_scope(is_loop_body)
    }
    pub fn pop_scope_emit_pub(&mut self) {
        self.pop_scope_emit()
    }
    pub fn block_stmt_pub(&mut self, b: &TBlock) {
        self.block_stmt(b)
    }

    fn loop_scope_depth(&self, label: LabelId) -> usize {
        self.cur.as_ref().unwrap().loop_labels.iter().rev().find(|(l, _)| *l == label).map(|(_, d)| *d).unwrap_or(0)
    }
    fn block_scope_depth(&self, label: LabelId) -> usize {
        // labeled blocks record their scope depth in loop_labels too
        self.cur.as_ref().unwrap().loop_labels.iter().rev().find(|(l, _)| *l == label).map(|(_, d)| *d).unwrap_or(0)
    }

    /// A temporary whose contents belong to the expression itself (not a view
    /// into a container that keeps ownership).
    fn is_owned_temp(&self, e: &TExpr) -> bool {
        !self.is_place_expr(e) && !crate::check::expr::is_borrowed_view(e)
    }

    fn is_place_expr(&self, e: &TExpr) -> bool {
        matches!(
            e.kind,
            TExprKind::Local(_) | TExprKind::Global(_) | TExprKind::Field { .. } | TExprKind::RefField { .. } | TExprKind::TupleField { .. } | TExprKind::Index { .. } | TExprKind::Deref(_)
        )
    }

    /// Emit a block's statements (no value) in the current scope.
    fn block_stmt(&mut self, b: &TBlock) {
        for s in &b.stmts {
            self.stmt(s);
        }
        if let Some(t) = &b.tail {
            let v = self.expr(t);
            if !self.is_void(t.ty) {
                self.line(format!("(void)({});", v));
            } else if v != "0" {
                self.line(format!("{};", v));
            }
        }
    }

    /// Emit a block and return its value expression (already bound to a temp if needed).
    fn block_value(&mut self, b: &TBlock) -> Option<String> {
        for s in &b.stmts {
            self.stmt(s);
        }
        match &b.tail {
            Some(t) => {
                let v = self.expr_owned(t);
                if self.is_void(t.ty) {
                    if v != "0" {
                        self.line(format!("{};", v));
                    }
                    None
                } else {
                    Some(v)
                }
            }
            None => None,
        }
    }

    /// Emit a block into a fresh C scope, assigning its value to `out` if given.
    fn block_into(&mut self, b: &TBlock, out: Option<&str>) {
        self.line("{");
        self.push_buf();
        self.push_scope(false);
        let v = self.block_value(b);
        if let (Some(out), Some(v)) = (out, v) {
            self.line(format!("{} = {};", out, v));
        }
        self.pop_scope_emit();
        let inner = self.pop_buf();
        self.body.last_mut().unwrap().push_str(&inner);
        self.line("}");
    }

    // ----- places ----------------------------------------------------------------

    /// C lvalue for a place expression.
    pub fn place(&mut self, e: &TExpr) -> String {
        match &e.kind {
            TExprKind::Local(l) => self.local_name(*l),
            TExprKind::Global(g) => self.p.globals[*g as usize].mangled.clone(),
            TExprKind::Field { base, idx } => {
                let b = self.place(base);
                let fname = self.field_name(base.ty, *idx);
                format!("{}.{}", b, fname)
            }
            TExprKind::RefField { base, idx } => {
                let b = self.simple(base);
                let fname = self.field_name(base.ty, *idx);
                format!("{}->{}", b, fname)
            }
            TExprKind::TupleField { base, idx } => {
                let b = self.place(base);
                format!("{}.f{}", b, idx)
            }
            TExprKind::Index { base, index, proven } => {
                let bt = self.res(base.ty);
                let i = self.expr(index);
                let loc = self.loc(e.span);
                match self.p.tys.kind(bt).clone() {
                    TyKind::Array(n, _) => {
                        let b = self.place(base);
                        if *proven || self.opts.mode == BuildMode::FastRelease {
                            format!("{}.v[{}]", b, i)
                        } else {
                            format!("{}.v[nx_idx({}, {}, {})]", b, i, n, loc)
                        }
                    }
                    TyKind::Slice(..) | TyKind::List(_) | TyKind::Str => {
                        let b = if self.is_place_expr(base) { self.place(base) } else { self.simple(base) };
                        if *proven || self.opts.mode == BuildMode::FastRelease {
                            format!("{}.ptr[{}]", b, i)
                        } else {
                            format!("{}.ptr[nx_idx({}, {}.len, {})]", b, i, b, loc)
                        }
                    }
                    _ => "0".into(),
                }
            }
            TExprKind::Deref(p) => {
                let pc = self.simple(p);
                format!("(*{})", pc)
            }
            TExprKind::ArrayToSlice(inner) | TExprKind::ListToSlice(inner) | TExprKind::StrToSlice(inner) => self.place(inner),
            _ => {
                // not a place: materialize
                self.simple(e)
            }
        }
    }

    pub fn field_name(&mut self, base_ty: TyId, idx: u32) -> String {
        let mut t = self.res(base_ty);
        while let TyKind::Ptr(_, inner) = self.p.tys.kind(t).clone() {
            t = self.res(inner);
        }
        match self.p.tys.kind(t).clone() {
            TyKind::Struct(d, _) => {
                let def = &self.p.structs[d as usize];
                if def.c_name.is_some() {
                    return def.fields[idx as usize].name.clone(); // C field names are used as declared
                }
                format!("{}_{}", sanitize_ident(&def.fields[idx as usize].name), idx)
            }
            _ => format!("f{}", idx),
        }
    }

    // ----- expressions -----------------------------------------------------------

    /// Emit an expression whose value becomes owned by a new location: a local of a
    /// resource type is moved (and zeroed at the source).
    pub fn expr_owned(&mut self, e: &TExpr) -> String {
        if let TExprKind::Local(l) = &e.kind {
            let ty = e.ty;
            let rt = self.res(ty);
            if self.needs_drop(ty) && !self.is_ref(rt) {
                let name = self.local_name(*l);
                let cn = self.cty(ty);
                let t = self.tmp();
                self.line(format!("{} {} = {}; memset(&{}, 0, sizeof {});", cn, t, name, name, name));
                return t;
            }
        }
        // `opt.?`, `opt orelse d`, `try res` on a local: the payload moves out, so
        // the source is zeroed afterwards (a zeroed optional is null, a zeroed
        // resource is empty) and its drop becomes a no-op
        if let TExprKind::Unwrap { expr: inner, .. } | TExprKind::OrElse { expr: inner, .. } | TExprKind::Try(inner) = &e.kind {
            if let TExprKind::Local(l) = &inner.kind {
                let rt = self.res(e.ty);
                if self.needs_drop(e.ty) && !self.is_ref(rt) {
                    let name = self.local_name(*l);
                    let cn = self.cty(e.ty);
                    let v = self.expr(e);
                    let t = self.tmp();
                    self.line(format!("{} {} = {}; memset(&{}, 0, sizeof {});", cn, t, v, name, name));
                    return t;
                }
            }
        }
        self.expr(e)
    }

    /// An owned value for a field of a literal, materialized now: a later field
    /// that moves the same local must not zero it before this one reads it.
    fn field_value(&mut self, x: &TExpr) -> String {
        let v = self.expr_owned(x);
        let t = self.tmp();
        let cn = self.cty(x.ty);
        self.line(format!("{} {} = {};", cn, t, v));
        t
    }

    pub fn expr(&mut self, e: &TExpr) -> String {
        match &e.kind {
            TExprKind::Int(v) => {
                let t = self.res(e.ty);
                match self.p.tys.kind(t).clone() {
                    TyKind::Int(it) => int_literal(*v, it),
                    TyKind::Distinct(d) => {
                        let u = self.res(self.p.distinct_underlying[&d]);
                        match self.p.tys.kind(u).clone() {
                            TyKind::Int(it) => int_literal(*v, it),
                            _ => float_literal(*v as f64),
                        }
                    }
                    TyKind::Float(_) => float_literal(*v as f64),
                    _ => format!("{}", v),
                }
            }
            TExprKind::Float(f) => float_literal(*f),
            TExprKind::Bool(b) => {
                if *b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            TExprKind::Char(c) => format!("{}u", c),
            TExprKind::Str(s) => {
                let lit = self.string_literal(s);
                format!("nx_lit({}, {})", lit, s.len())
            }
            TExprKind::Unit => "0".into(),
            TExprKind::Local(l) => self.local_name(*l),
            TExprKind::Global(g) => self.p.globals[*g as usize].mangled.clone(),
            TExprKind::Const(id) => self.p.consts[*id as usize].mangled.clone(),
            TExprKind::FnRef(i) | TExprKind::FnToFat(i) => {
                let th = self.thunk(*i);
                let cn = self.cty(e.ty);
                format!("(({}){{ {}, NULL }})", cn, th)
            }
            TExprKind::Field { base, idx } => {
                let b = self.expr(base);
                let fname = self.field_name(base.ty, *idx);
                format!("({}).{}", b, fname)
            }
            TExprKind::RefField { base, idx } => {
                let b = self.simple(base);
                let fname = self.field_name(base.ty, *idx);
                format!("({})->{}", b, fname)
            }
            TExprKind::TupleField { base, idx } => {
                let b = self.expr(base);
                format!("({}).f{}", b, idx)
            }
            TExprKind::Index { .. } | TExprKind::Deref(_) => self.place(e),
            TExprKind::SliceOp { base, start, end } => self.slice_op(base, start.as_deref(), end.as_deref(), e),
            TExprKind::AddrOf { expr, .. } => {
                if self.is_place_expr(expr) {
                    let p = self.place(expr);
                    format!("&({})", p)
                } else {
                    let t = self.simple(expr);
                    format!("&{}", t)
                }
            }
            TExprKind::Call { inst, args } => {
                let f = self.p.funcs[*inst as usize].clone();
                let mut a: Vec<String> = Vec::new();
                if !f.is_extern {
                    a.push("c".into());
                }
                for (i, x) in args.iter().enumerate() {
                    let owned_param = f.params.get(i).map(|&p| f.locals[p as usize].owned).unwrap_or(false);
                    let v = if owned_param {
                        // handed over: a local is moved (zeroed), a temporary is not dropped here
                        if matches!(x.kind, TExprKind::Local(_)) {
                            self.expr_owned(x)
                        } else {
                            self.simple_owned(x)
                        }
                    } else {
                        self.simple(x)
                    };
                    a.push(v);
                }
                let name = self.fn_c_name(*inst);
                let call = format!("{}({})", name, a.join(", "));
                if self.is_void(e.ty) {
                    call
                } else {
                    self.bind_tmp(&call, e.ty)
                }
            }
            TExprKind::CallPtr { callee, args } => {
                let f = self.simple(callee);
                let mut a = vec!["c".to_string(), format!("{}.env", f)];
                for x in args {
                    let v = self.simple(x);
                    a.push(v);
                }
                let call = format!("{}.fn({})", f, a.join(", "));
                if self.is_void(e.ty) {
                    call
                } else {
                    self.bind_tmp(&call, e.ty)
                }
            }
            TExprKind::Builtin { op, args, tys } => self.builtin(*op, args, tys, e),
            TExprKind::Unary { op, expr, mode } => {
                let v = self.expr(expr);
                match op {
                    UnOp::Not => format!("(!({}))", v),
                    UnOp::BitNot => {
                        let t = self.res(e.ty);
                        let cn = self.cty(t);
                        format!("(({})~({}))", cn, v)
                    }
                    UnOp::Neg => match mode {
                        ArithMode::Checked if self.opts.mode != BuildMode::FastRelease => {
                            let m = self.int_mangle(e.ty);
                            let loc = self.loc(e.span);
                            format!("nx_neg_{}({}, {})", m, v, loc)
                        }
                        _ => format!("(-({}))", v),
                    },
                    _ => v,
                }
            }
            TExprKind::Binary { op, lhs, rhs, mode, proven } => {
                let l = self.expr(lhs);
                let r = self.expr(rhs);
                self.binop_code(*op, *mode, &l, &r, lhs.ty, e.span, !*proven)
            }
            TExprKind::Logical { and, lhs, rhs } => {
                let l = self.expr(lhs);
                let t = self.tmp();
                self.line(format!("bool {} = {};", t, l));
                self.line(format!("if ({}{}) {{", if *and { "" } else { "!" }, t));
                self.push_buf();
                // temporaries the right side creates live in its C block, so
                // they are released there and not at the statement's end
                self.push_scope(false);
                let r = self.expr(rhs);
                self.line(format!("{} = {};", t, r));
                self.pop_scope_emit();
                let inner = self.pop_buf();
                self.body.last_mut().unwrap().push_str(&inner);
                self.line("}");
                t
            }
            TExprKind::Cast { expr, kind } => self.cast(expr, *kind, e),
            TExprKind::If { cond, then, els } => {
                let c = self.expr(cond);
                let out = if self.is_void(e.ty) {
                    None
                } else {
                    let t = self.tmp();
                    let cn = self.cty(e.ty);
                    self.line(format!("{} {};", cn, t));
                    Some(t)
                };
                self.line(format!("if ({})", c));
                self.block_into(then, out.as_deref());
                if let Some(eb) = els {
                    self.line("else");
                    self.block_into(eb, out.as_deref());
                }
                out.unwrap_or_else(|| "0".into())
            }
            TExprKind::IfCapture { cond, local, then, els } => {
                let owned = self.is_owned_temp(cond);
                let c = self.simple_owned(cond);
                let out = if self.is_void(e.ty) {
                    None
                } else {
                    let t = self.tmp();
                    let cn = self.cty(e.ty);
                    self.line(format!("{} {};", cn, t));
                    Some(t)
                };
                self.line(format!("if ({}.has) {{", c));
                self.push_buf();
                self.push_scope(false);
                let name = self.local_name(*local);
                let lt = self.st().local_tys[*local as usize];
                let cn = self.cty(lt);
                self.line(format!("{} {} = {}.val;", cn, name, c));
                if owned {
                    self.register_drop(&name, lt);
                }
                self.block_into(then, out.as_deref());
                self.pop_scope_emit();
                let inner = self.pop_buf();
                self.body.last_mut().unwrap().push_str(&inner);
                self.line("}");
                if let Some(eb) = els {
                    self.line("else");
                    self.block_into(eb, out.as_deref());
                }
                out.unwrap_or_else(|| "0".into())
            }
            TExprKind::Match { scrutinee, arms } => self.match_expr(scrutinee, arms, e),
            TExprKind::Block(b) => {
                if let Some(label) = b.label {
                    // labeled block: value set by `break :label v` or the tail
                    let brk = format!("nx_blk_{}_{}", label, self.tmp());
                    let out = if self.is_void(e.ty) {
                        "0".to_string()
                    } else {
                        let t = self.tmp();
                        let cn = self.cty(e.ty);
                        self.line(format!("{} {};", cn, t));
                        t
                    };
                    let depth = self.st().scopes.len();
                    self.st().loop_labels.push((label, depth));
                    self.st().block_labels.push((label, brk.clone(), out.clone()));
                    self.block_into(b, if out == "0" { None } else { Some(&out) });
                    self.st().block_labels.pop();
                    self.st().loop_labels.pop();
                    self.line(format!("{}: ;", brk));
                    return out;
                }
                if self.is_void(e.ty) {
                    self.block_into(b, None);
                    "0".into()
                } else {
                    let t = self.tmp();
                    let cn = self.cty(e.ty);
                    self.line(format!("{} {};", cn, t));
                    self.block_into(b, Some(&t));
                    t
                }
            }
            TExprKind::StructLit { fields } => {
                let cn = self.cty(e.ty);
                let mut parts = Vec::new();
                for (i, f) in fields {
                    let v = self.field_value(f);
                    let fname = self.field_name(e.ty, *i);
                    parts.push(format!(".{} = {}", fname, v));
                }
                if parts.is_empty() {
                    format!("(({}){{0}})", cn)
                } else {
                    format!("(({}){{ {} }})", cn, parts.join(", "))
                }
            }
            TExprKind::RefNew { fields } => {
                let cn = self.cty(e.ty);
                let t = self.tmp();
                self.line(format!("{} {} = ({})nx_alloc_bytes(c, sizeof({}_obj), _Alignof({}_obj));", cn, t, cn, cn, cn));
                self.line(format!("{}->rc = 1; {}->weak = 0;", t, t));
                for (i, f) in fields {
                    let v = self.field_value(f);
                    let fname = self.field_name(e.ty, *i);
                    self.line(format!("{}->{} = {};", t, fname, v));
                }
                t
            }
            TExprKind::EnumLit { variant, payload } => {
                let cn = self.cty(e.ty);
                if payload.is_empty() {
                    format!("(({}){{ .tag = {} }})", cn, variant)
                } else {
                    let mut parts = Vec::new();
                    for (i, p) in payload.iter().enumerate() {
                        let v = self.field_value(p);
                        parts.push(format!(".f{} = {}", i, v));
                    }
                    format!("(({}){{ .tag = {}, .u = {{ .v{} = {{ {} }} }} }})", cn, variant, variant, parts.join(", "))
                }
            }
            TExprKind::ArrayLit(elems) => {
                let cn = self.cty(e.ty);
                let mut parts = Vec::new();
                for x in elems {
                    parts.push(self.expr_owned(x));
                }
                if parts.is_empty() {
                    format!("(({}){{0}})", cn)
                } else {
                    format!("(({}){{ {{ {} }} }})", cn, parts.join(", "))
                }
            }
            TExprKind::ArrayRepeat { value, count } => {
                let cn = self.cty(e.ty);
                let v = self.simple(value);
                let t = self.tmp();
                self.line(format!("{} {}; for (size_t i = 0; i < {}; i++) {}.v[i] = {};", cn, t, count, t, v));
                t
            }
            TExprKind::TupleLit(elems) => {
                let cn = self.cty(e.ty);
                let mut parts = Vec::new();
                for (i, x) in elems.iter().enumerate() {
                    let v = self.field_value(x);
                    parts.push(format!(".f{} = {}", i, v));
                }
                format!("(({}){{ {} }})", cn, parts.join(", "))
            }
            TExprKind::Try(inner) => {
                let v = self.simple_owned(inner);
                let ret = self.st().ret;
                let rcn = self.cty(ret);
                self.line(format!("if ({}.err) {{", v));
                self.push_buf();
                self.emit_exits_down_to(0, true);
                self.line(format!("return ({}){{ .err = {}.err }};", rcn, v));
                let inner_c = self.pop_buf();
                self.body.last_mut().unwrap().push_str(&inner_c);
                self.line("}");
                if self.is_void(e.ty) {
                    "0".into()
                } else {
                    format!("{}.val", v)
                }
            }
            TExprKind::Catch { expr, err_local, handler } => {
                let v = self.simple_owned(expr);
                let out = if self.is_void(e.ty) {
                    None
                } else {
                    let t = self.tmp();
                    let cn = self.cty(e.ty);
                    self.line(format!("{} {};", cn, t));
                    Some(t)
                };
                self.line(format!("if ({}.err) {{", v));
                self.push_buf();
                self.push_scope(false);
                if let Some(l) = err_local {
                    let name = self.local_name(*l);
                    self.line(format!("uint32_t {} = {}.err;", name, v));
                }
                let h = self.expr_owned(handler);
                if let Some(o) = &out {
                    if !self.is_void(handler.ty) {
                        self.line(format!("{} = {};", o, h));
                    }
                } else if h != "0" {
                    self.line(format!("{};", h));
                }
                self.pop_scope_emit();
                let inner_c = self.pop_buf();
                self.body.last_mut().unwrap().push_str(&inner_c);
                match &out {
                    Some(o) => {
                        self.line(format!("}} else {{ {} = {}.val; }}", o, v));
                        o.clone()
                    }
                    None => {
                        self.line("}");
                        "0".into()
                    }
                }
            }
            TExprKind::OrElse { expr, default } => {
                let v = self.simple_owned(expr);
                let out = if self.is_void(e.ty) {
                    None
                } else {
                    let t = self.tmp();
                    let cn = self.cty(e.ty);
                    self.line(format!("{} {};", cn, t));
                    Some(t)
                };
                self.line(format!("if (!{}.has) {{", v));
                self.push_buf();
                let d = self.expr_owned(default);
                if let Some(o) = &out {
                    if !self.is_void(default.ty) {
                        self.line(format!("{} = {};", o, d));
                    }
                } else if d != "0" {
                    self.line(format!("{};", d));
                }
                let inner_c = self.pop_buf();
                self.body.last_mut().unwrap().push_str(&inner_c);
                match &out {
                    Some(o) => {
                        self.line(format!("}} else {{ {} = {}.val; }}", o, v));
                        o.clone()
                    }
                    None => {
                        self.line("}");
                        "0".into()
                    }
                }
            }
            TExprKind::Unwrap { expr, .. } => {
                let v = self.simple_owned(expr);
                let loc = self.loc(e.span);
                self.line(format!("if (!{}.has) nx_panic(\"unwrapped a null optional\", {});", v, loc));
                format!("{}.val", v)
            }
            TExprKind::OptWrap(inner) => {
                let cn = self.cty(e.ty);
                if self.is_void(inner.ty) {
                    let v = self.expr(inner);
                    if v != "0" {
                        self.line(format!("{};", v));
                    }
                    format!("(({}){{ .has = true }})", cn)
                } else {
                    let v = self.expr_owned(inner);
                    format!("(({}){{ .has = true, .val = {} }})", cn, v)
                }
            }
            TExprKind::OptNull => {
                let cn = self.cty(e.ty);
                format!("(({}){{ .has = false }})", cn)
            }
            TExprKind::ErrWrap(inner) => {
                let cn = self.cty(e.ty);
                if self.is_void(inner.ty) {
                    let v = self.expr(inner);
                    if v != "0" {
                        self.line(format!("{};", v));
                    }
                    format!("(({}){{ .err = 0 }})", cn)
                } else {
                    let v = self.expr_owned(inner);
                    format!("(({}){{ .err = 0, .val = {} }})", cn, v)
                }
            }
            TExprKind::ErrToUnion(inner) => {
                let cn = self.cty(e.ty);
                let v = self.simple(inner);
                format!("(({}){{ .err = {} }})", cn, v)
            }
            TExprKind::ErrVal(id) => {
                let t = self.res(e.ty);
                match self.p.tys.kind(t) {
                    TyKind::ErrUnion(..) => {
                        let cn = self.cty(t);
                        format!("(({}){{ .err = {}u }})", cn, id)
                    }
                    _ => format!("{}u", id),
                }
            }
            TExprKind::ArrayToSlice(inner) => {
                let cn = self.cty(e.ty);
                let it = self.res(inner.ty);
                let n = match self.p.tys.kind(it) {
                    TyKind::Array(n, _) => *n,
                    _ => 0,
                };
                if self.is_place_expr(inner) {
                    let p = self.place(inner);
                    format!("(({}){{ {}.v, {} }})", cn, p, n)
                } else {
                    let t = self.simple(inner);
                    format!("(({}){{ {}.v, {} }})", cn, t, n)
                }
            }
            TExprKind::ListToSlice(inner) => {
                let cn = self.cty(e.ty);
                let v = self.simple(inner);
                format!("(({}){{ {}.ptr, {}.len }})", cn, v, v)
            }
            TExprKind::StrToSlice(inner) => {
                let v = self.simple(inner);
                format!("nx_str_slice({})", v)
            }
            TExprKind::Closure { inst, captures } => {
                let env_ty = self.closure_env_type(*inst);
                let et = self.tmp();
                let mut parts = Vec::new();
                for (i, (l, by_ref)) in captures.iter().enumerate() {
                    let name = self.local_name(*l);
                    parts.push(format!(".c{} = {}{}", i, if *by_ref { "&" } else { "" }, name));
                }
                if parts.is_empty() {
                    parts.push(".{ _e = 0 }".to_string());
                    parts.clear();
                }
                if parts.is_empty() {
                    self.line(format!("{} {} = {{0}};", env_ty, et));
                } else {
                    self.line(format!("{} {} = {{ {} }};", env_ty, et, parts.join(", ")));
                }
                let name = self.fn_c_name(*inst);
                let cn = self.cty(e.ty);
                format!("(({}){{ {}, &{} }})", cn, name, et)
            }
            TExprKind::Unreachable => {
                let loc = self.loc(e.span);
                self.line(format!("nx_panic(\"reached unreachable code\", {});", loc));
                // never runs, but must be a value of the expression's type when that
                // type is an aggregate (an error union, a struct) rather than a scalar
                let cn = self.cty(e.ty);
                if cn == "void" {
                    "0".into()
                } else {
                    format!("(({}){{0}})", cn)
                }
            }
            TExprKind::Undefined => {
                let cn = self.cty(e.ty);
                format!("(({}){{0}})", cn)
            }
            TExprKind::BinConstruct { segments, target } => self.bin_construct(segments, target, e),
            TExprKind::Value(v) => {
                let ty = e.ty;
                match v {
                    Value::Int(_) | Value::Float(_) | Value::Bool(_) | Value::Char(_) => self.static_init(v, ty).unwrap_or_else(|| "0".into()),
                    Value::Str(s) => {
                        let lit = self.string_literal(s);
                        format!("nx_lit({}, {})", lit, s.len())
                    }
                    _ => {
                        let cn = self.cty(ty);
                        match self.static_init(v, ty) {
                            Some(init) => format!("(({}){})", cn, init),
                            None => "0".into(),
                        }
                    }
                }
            }
            TExprKind::TypeVal(_) => "0".into(),
            TExprKind::RecordCheck { value, checks, as_error } => {
                let v = self.simple_owned(value);
                let loc = self.loc(e.span);
                let out_cn = self.cty(e.ty);
                let out = self.tmp();
                self.line(format!("{} {};", out_cn, out));
                if *as_error {
                    self.line(format!("{}.err = 0;", out));
                }
                for (idx, ce, local) in checks {
                    let name = self.local_name(*local);
                    let lt = self.st().local_tys[*local as usize];
                    let cn = self.cty(lt);
                    let fname = self.field_name(value.ty, *idx);
                    self.line("{");
                    self.push_buf();
                    self.line(format!("{} {} = {}.{};", cn, name, v, fname));
                    let cond = self.expr(ce);
                    if *as_error {
                        let id = self.p.error_names.iter().position(|n| n == "InvalidRecord").map(|i| i + 1).unwrap_or(0);
                        self.line(format!("if (!({})) {}.err = {}u;", cond, out, id));
                    } else {
                        self.line(format!("if (!({})) nx_panic(\"record constraint violated\", {});", cond, loc));
                    }
                    let inner = self.pop_buf();
                    self.body.last_mut().unwrap().push_str(&inner);
                    self.line("}");
                }
                if *as_error {
                    self.line(format!("if ({}.err == 0) {}.val = {};", out, out, v));
                } else {
                    self.line(format!("{} = {};", out, v));
                }
                out
            }
            TExprKind::DynFrom { expr, vtable } => {
                let v = self.expr(expr);
                let vt = self.vtable_instance(*vtable);
                let cn = self.cty(e.ty);
                format!("(({}){{ (void*)({}), &{} }})", cn, v, vt)
            }
            TExprKind::DynCall { recv, method, args } => {
                let r = self.simple(recv);
                let tr = match self.kind_of(recv.ty) {
                    TyKind::Dyn(t, _) => t,
                    _ => 0,
                };
                let mname = sanitize_ident(&self.p.traits[tr as usize].methods[*method as usize].name);
                let mut a = vec!["c".to_string(), format!("{}.data", r)];
                for x in args {
                    let v = self.simple(x);
                    a.push(v);
                }
                let call = format!("{}.vt->{}({})", r, mname, a.join(", "));
                if self.is_void(e.ty) {
                    call
                } else {
                    self.bind_tmp(&call, e.ty)
                }
            }
            TExprKind::Retained(inner) => {
                let v = self.expr(inner);
                let cn = self.cty(e.ty);
                let rt = self.res(e.ty);
                if self.is_ref(rt) {
                    format!("(({})nx_retain({}))", cn, v)
                } else {
                    let vt = self.bind_tmp(&v, e.ty);
                    let f = self.clone_fn(e.ty);
                    format!("{}(c, &{})", f, vt)
                }
            }
        }
    }

    pub fn int_mangle(&mut self, t: TyId) -> String {
        let t = self.res(t);
        match self.p.tys.kind(t).clone() {
            TyKind::Int(i) => i.name().to_string(),
            TyKind::Distinct(d) => {
                let u = self.p.distinct_underlying[&d];
                self.int_mangle(u)
            }
            TyKind::Bool => "u8".into(),
            TyKind::Char => "u32".into(),
            _ => "i64".into(),
        }
    }

    pub fn binop_code(&mut self, op: BinOp, mode: ArithMode, l: &str, r: &str, ty: TyId, span: Span, checked: bool) -> String {
        let fast = self.opts.mode == BuildMode::FastRelease;
        if op.is_comparison() {
            let t = self.res(ty);
            let is_scalar = matches!(self.p.tys.kind(t), TyKind::Int(_) | TyKind::Float(_) | TyKind::Bool | TyKind::Char | TyKind::Ptr(..) | TyKind::ErrorSet(_) | TyKind::Distinct(_));
            if is_scalar {
                let d = match self.p.tys.kind(t).clone() {
                    TyKind::Distinct(d) => Some(d),
                    _ => None,
                };
                let scalar_under = d.map(|d| self.res(self.p.distinct_underlying[&d])).unwrap_or(t);
                if matches!(self.p.tys.kind(scalar_under), TyKind::Int(_) | TyKind::Float(_) | TyKind::Bool | TyKind::Char | TyKind::Ptr(..) | TyKind::ErrorSet(_)) {
                    return format!("(({}) {} ({}))", l, op.symbol(), r);
                }
            }
            if let TyKind::Opt(_) = self.p.tys.kind(t) {
                // compare with null: only `has` matters
                return match op {
                    BinOp::Eq => format!("(({}).has == ({}).has)", l, r),
                    _ => format!("(({}).has != ({}).has)", l, r),
                };
            }
            return match op {
                BinOp::Eq => self.eq_expr(t, l, r),
                BinOp::Ne => {
                    let e = self.eq_expr(t, l, r);
                    format!("(!{})", e)
                }
                _ => {
                    let c = self.cmp_expr(t, l, r);
                    format!("({} {} 0)", c, op.symbol())
                }
            };
        }
        let m = self.int_mangle(ty);
        let loc = self.loc(span);
        match mode {
            ArithMode::Float if op == BinOp::Rem => format!("fmod({}, {})", l, r),
            ArithMode::Float | ArithMode::Plain => match op {
                BinOp::Shl => format!("(({}) << ({}))", l, r),
                BinOp::Shr => format!("(({}) >> ({}))", l, r),
                _ => format!("(({}) {} ({}))", l, op.symbol(), r),
            },
            ArithMode::Wrap => match op {
                BinOp::AddWrap | BinOp::Add => format!("nx_addw_{}({}, {})", m, l, r),
                BinOp::SubWrap | BinOp::Sub => format!("nx_subw_{}({}, {})", m, l, r),
                _ => format!("nx_mulw_{}({}, {})", m, l, r),
            },
            ArithMode::Sat => match op {
                BinOp::AddSat | BinOp::Add => format!("nx_adds_{}({}, {})", m, l, r),
                BinOp::SubSat | BinOp::Sub => format!("nx_subs_{}({}, {})", m, l, r),
                _ => format!("nx_muls_{}({}, {})", m, l, r),
            },
            ArithMode::Checked => {
                if fast || !checked {
                    return match op {
                        BinOp::Shl => format!("(({}) << ({}))", l, r),
                        BinOp::Shr => format!("(({}) >> ({}))", l, r),
                        BinOp::Div | BinOp::Rem if !fast => {
                            let f = if op == BinOp::Div { "div" } else { "rem" };
                            format!("nx_{}_{}({}, {}, {})", f, m, l, r, loc)
                        }
                        _ => format!("(({}) {} ({}))", l, op.symbol(), r),
                    };
                }
                let f = match op {
                    BinOp::Add => "add",
                    BinOp::Sub => "sub",
                    BinOp::Mul => "mul",
                    BinOp::Div => "div",
                    BinOp::Rem => "rem",
                    BinOp::Shl => "shl",
                    BinOp::Shr => "shr",
                    _ => return format!("(({}) {} ({}))", l, op.symbol(), r),
                };
                if matches!(op, BinOp::Shl | BinOp::Shr) {
                    format!("nx_{}_{}({}, (uint32_t)({}), {})", f, m, l, r, loc)
                } else {
                    format!("nx_{}_{}({}, {}, {})", f, m, l, r, loc)
                }
            }
        }
    }

    fn cast(&mut self, inner: &TExpr, kind: CastKind, e: &TExpr) -> String {
        let v = self.expr(inner);
        let to = self.res(e.ty);
        let cn = self.cty(to);
        match kind {
            CastKind::IntToInt { checked } => {
                if checked && self.opts.mode != BuildMode::FastRelease {
                    let (lo, hi) = self.int_bounds(to);
                    let loc = self.loc(e.span);
                    format!("(({})nx_cast_check((nx_i128)({}), {}, {}, {}))", cn, v, lo, hi, loc)
                } else {
                    format!("(({})({}))", cn, v)
                }
            }
            CastKind::IntToFloat | CastKind::FloatToFloat | CastKind::Bits | CastKind::PtrToPtr => format!("(({})({}))", cn, v),
            CastKind::FloatToInt => {
                if self.opts.mode == BuildMode::FastRelease {
                    format!("(({})({}))", cn, v)
                } else {
                    let (lo, hi) = self.int_bounds(to);
                    let loc = self.loc(e.span);
                    format!("(({})nx_f2i((double)({}), {}, {}, {}))", cn, v, lo, hi, loc)
                }
            }
        }
    }

    fn int_bounds(&mut self, t: TyId) -> (String, String) {
        let t = self.res(t);
        match self.p.tys.kind(t).clone() {
            TyKind::Int(it) => (int_literal(it.min(), IntTy::I128), int_literal(it.max(), IntTy::I128)),
            TyKind::Char => ("0".into(), "0x10FFFF".into()),
            TyKind::Distinct(d) => {
                let u = self.p.distinct_underlying[&d];
                self.int_bounds(u)
            }
            _ => ("0".into(), "0".into()),
        }
    }

    fn slice_op(&mut self, base: &TExpr, start: Option<&TExpr>, end: Option<&TExpr>, e: &TExpr) -> String {
        let bt = self.res(base.ty);
        let cn = self.cty(e.ty);
        let (ptr, len) = match self.p.tys.kind(bt).clone() {
            TyKind::Array(n, _) => {
                let b = if self.is_place_expr(base) { self.place(base) } else { self.simple(base) };
                (format!("{}.v", b), n.to_string())
            }
            TyKind::Slice(..) | TyKind::List(_) | TyKind::Str => {
                let b = if self.is_place_expr(base) { self.place(base) } else { self.simple(base) };
                (format!("{}.ptr", b), format!("{}.len", b))
            }
            _ => ("0".into(), "0".into()),
        };
        let s = match start {
            Some(s) => self.simple(s),
            None => "0".into(),
        };
        let en = match end {
            Some(x) => self.simple(x),
            None => len.clone(),
        };
        let loc = self.loc(e.span);
        if self.opts.mode != BuildMode::FastRelease {
            self.line(format!("nx_slice_check({}, {}, {}, {});", s, en, len, loc));
        }
        format!("(({}){{ {} + {}, {} - {} }})", cn, ptr, s, en, s)
    }

    // ----- match -----------------------------------------------------------------

    fn match_expr(&mut self, scrutinee: &TExpr, arms: &[TArm], e: &TExpr) -> String {
        let owned = self.is_owned_temp(scrutinee);
        let s = self.simple_owned(scrutinee);
        let out = if self.is_void(e.ty) {
            None
        } else {
            let t = self.tmp();
            let cn = self.cty(e.ty);
            self.line(format!("{} {};", cn, t));
            Some(t)
        };
        let done = format!("nx_match_done_{}", self.tmp());
        let loc = self.loc(e.span);
        for arm in arms {
            self.line("{");
            self.push_buf();
            self.push_scope(false);
            let cond = self.pattern_test(&arm.pat, &s, scrutinee.ty);
            self.line(format!("if ({}) {{", cond));
            self.push_buf();
            self.pattern_bind(&arm.pat, &s, scrutinee.ty, owned);
            let guard_ok = match &arm.guard {
                Some(g) => {
                    let gc = self.expr(g);
                    self.line(format!("if ({}) {{", gc));
                    self.push_buf();
                    true
                }
                None => false,
            };
            let v = self.expr_owned(&arm.body);
            match &out {
                Some(o) if !self.is_void(arm.body.ty) => self.line(format!("{} = {};", o, v)),
                _ => {
                    if v != "0" {
                        self.line(format!("{};", v));
                    }
                }
            }
            self.pop_scope_emit_no_pop();
            self.line(format!("goto {};", done));
            if guard_ok {
                let inner = self.pop_buf();
                self.body.last_mut().unwrap().push_str(&inner);
                self.line("}");
            }
            let inner = self.pop_buf();
            self.body.last_mut().unwrap().push_str(&inner);
            self.line("}");
            self.st().scopes.pop();
            let inner = self.pop_buf();
            self.body.last_mut().unwrap().push_str(&inner);
            self.line("}");
        }
        self.line(format!("nx_panic(\"no match arm matched\", {});", loc));
        self.line(format!("{}: ;", done));
        out.unwrap_or_else(|| "0".into())
    }

    /// Emit the innermost scope's exits without popping (used before a goto out of an arm).
    fn pop_scope_emit_no_pop(&mut self) {
        let i = self.st().scopes.len() - 1;
        self.emit_scope_exit(i, false);
        // clear so the pop does not emit again
        self.st().scopes[i].drops.clear();
        self.st().scopes[i].defers.clear();
    }

    /// A C condition testing whether `s` (a simple expression of type `ty`) matches the pattern.
    fn pattern_test(&mut self, p: &TPat, s: &str, ty: TyId) -> String {
        let ty = self.res(ty);
        match p {
            TPat::Wild | TPat::Bind(_) => "1".into(),
            TPat::Int(v) => {
                let lit = match self.p.tys.kind(ty).clone() {
                    TyKind::Int(it) => int_literal(*v, it),
                    _ => format!("{}", v),
                };
                format!("({} == {})", s, lit)
            }
            TPat::Float(f) => format!("({} == {})", s, float_literal(*f)),
            TPat::Bool(b) => format!("({} == {})", s, b),
            TPat::Char(c) => format!("({} == {}u)", s, c),
            TPat::Str(lit) => {
                let l = self.string_literal(lit);
                let sl = match self.p.tys.kind(ty) {
                    TyKind::Str => format!("nx_str_slice({})", s),
                    _ => s.to_string(),
                };
                format!("nx_sl_eq({}, nx_lit({}, {}))", sl, l, lit.len())
            }
            TPat::Range { lo, hi, inclusive } => {
                let (l, h) = match self.p.tys.kind(ty).clone() {
                    TyKind::Int(it) => (int_literal(*lo, it), int_literal(*hi, it)),
                    _ => (format!("{}u", lo), format!("{}u", hi)),
                };
                format!("({} >= {} && {} {} {})", s, l, s, if *inclusive { "<=" } else { "<" }, h)
            }
            TPat::Variant { idx, args } => {
                let mut conds = vec![format!("{}.tag == {}", s, idx)];
                let vtys = self.p.enum_variant_tys.get(&ty).cloned().unwrap_or_default();
                for (i, a) in args.iter().enumerate() {
                    let at = vtys[*idx as usize][i];
                    let sub = format!("{}.u.v{}.f{}", s, idx, i);
                    let c = self.pattern_test(a, &sub, at);
                    if c != "1" {
                        conds.push(c);
                    }
                }
                format!("({})", conds.join(" && "))
            }
            TPat::Error(id) => match self.p.tys.kind(ty) {
                TyKind::ErrUnion(..) => format!("({}.err == {}u)", s, id),
                _ => format!("({} == {}u)", s, id),
            },
            TPat::Null => format!("(!{}.has)", s),
            TPat::Some(inner) => {
                let it = match self.p.tys.kind(ty).clone() {
                    TyKind::Opt(e) => e,
                    _ => ty,
                };
                let c = self.pattern_test(inner, &format!("{}.val", s), it);
                if c == "1" {
                    format!("({}.has)", s)
                } else {
                    format!("({}.has && {})", s, c)
                }
            }
            TPat::Ok(inner) => {
                let it = match self.p.tys.kind(ty).clone() {
                    TyKind::ErrUnion(_, e) => e,
                    _ => ty,
                };
                let c = self.pattern_test(inner, &format!("{}.val", s), it);
                if c == "1" {
                    format!("({}.err == 0)", s)
                } else {
                    format!("({}.err == 0 && {})", s, c)
                }
            }
            TPat::Or(alts) => {
                let cs: Vec<String> = alts.iter().map(|a| self.pattern_test(a, s, ty)).collect();
                format!("({})", cs.join(" || "))
            }
            TPat::Tuple(ps) => {
                let ts = match self.p.tys.kind(ty).clone() {
                    TyKind::Tuple(ts) => ts,
                    _ => vec![],
                };
                let mut conds = Vec::new();
                for (i, p) in ps.iter().enumerate() {
                    let c = self.pattern_test(p, &format!("{}.f{}", s, i), ts[i]);
                    if c != "1" {
                        conds.push(c);
                    }
                }
                if conds.is_empty() {
                    "1".into()
                } else {
                    format!("({})", conds.join(" && "))
                }
            }
            TPat::Binary(segs) => self.bin_pattern_test(segs, s, ty),
        }
    }

    /// Declare the pattern's bindings (the test has succeeded).
    fn pattern_bind(&mut self, p: &TPat, s: &str, ty: TyId, owned: bool) {
        let ty = self.res(ty);
        match p {
            TPat::Bind(l) => {
                let name = self.local_name(*l);
                let lt_raw = self.st().local_tys[*l as usize];
                let lt = self.res(lt_raw);
                // bound by reference (matching through a pointer): alias the payload
                if lt != ty {
                    if let TyKind::Ptr(_, inner) = self.p.tys.kind(lt).clone() {
                        if self.res(inner) == ty {
                            let cn = self.cty(lt);
                            self.line(format!("{} {} = &({});", cn, name, s));
                            return;
                        }
                    }
                }
                let cn = self.cty(ty);
                self.line(format!("{} {} = {};", cn, name, s));
                if owned {
                    self.register_drop(&name, ty);
                }
            }
            TPat::Variant { idx, args } => {
                let vtys = self.p.enum_variant_tys.get(&ty).cloned().unwrap_or_default();
                for (i, a) in args.iter().enumerate() {
                    let at = vtys[*idx as usize][i];
                    self.pattern_bind(a, &format!("{}.u.v{}.f{}", s, idx, i), at, owned);
                }
            }
            TPat::Some(inner) => {
                let it = match self.p.tys.kind(ty).clone() {
                    TyKind::Opt(e) => e,
                    _ => ty,
                };
                self.pattern_bind(inner, &format!("{}.val", s), it, owned);
            }
            TPat::Ok(inner) => {
                let it = match self.p.tys.kind(ty).clone() {
                    TyKind::ErrUnion(_, e) => e,
                    _ => ty,
                };
                self.pattern_bind(inner, &format!("{}.val", s), it, owned);
            }
            TPat::Tuple(ps) => {
                let ts = match self.p.tys.kind(ty).clone() {
                    TyKind::Tuple(ts) => ts,
                    _ => vec![],
                };
                for (i, p) in ps.iter().enumerate() {
                    self.pattern_bind(p, &format!("{}.f{}", s, i), ts[i], owned);
                }
            }
            TPat::Binary(segs) => self.bin_pattern_bind(segs, s, ty),
            _ => {}
        }
    }

    // ----- binary patterns ---------------------------------------------------

    fn bytes_of(&mut self, s: &str, ty: TyId) -> String {
        match self.kind_of(ty) {
            TyKind::Str => format!("nx_str_slice({})", s),
            TyKind::Array(n, _) => format!("nx_lit((const char*){}.v, {})", s, n),
            _ => s.to_string(),
        }
    }

    /// Binary pattern test: computes into a temporary flag; bindings are recomputed in bind.
    fn bin_pattern_test(&mut self, segs: &[TBinSeg], s: &str, ty: TyId) -> String {
        let buf = self.bytes_of(s, ty);
        let flag = self.tmp();
        self.line(format!("bool {} = false;", flag));
        self.line("do {");
        self.push_buf();
        let bit = self.tmp();
        self.line(format!("size_t {} = 0; NX_UNUSED({});", bit, bit));
        let total = format!("({}.len * 8)", buf);
        // static total size when all segments are constant
        let mut const_bits: u64 = 0;
        let all_const = segs.iter().all(|sg| matches!(sg.size, TBinSize::Bits(_)));
        if all_const {
            for sg in segs {
                if let TBinSize::Bits(b) = sg.size {
                    const_bits += b;
                }
            }
            self.line(format!("if ({} != {}) break;", total, const_bits));
        }
        for sg in segs {
            let size = match &sg.size {
                TBinSize::Bits(b) => b.to_string(),
                TBinSize::Expr(e) => {
                    // size expressions may use earlier bindings: bind them progressively
                    let v = self.expr(e);
                    format!("((size_t)({}))", v)
                }
                TBinSize::Rest => format!("({} - {})", total, bit),
            };
            let sz = self.tmp();
            self.line(format!("size_t {} = {};", sz, size));
            if !all_const {
                self.line(format!("if ({} + {} > {}) break;", bit, sz, total));
            }
            let is_bytes = matches!(self.kind_of(sg.ty), TyKind::Slice(..)) || matches!(sg.size, TBinSize::Rest);
            match &sg.kind {
                TBinSegKind::Bind(l) | TBinSegKind::Rest(l) => {
                    let name = self.local_name(*l);
                    if is_bytes {
                        self.line(format!("if (({} & 7) || ({} & 7)) break;", bit, sz));
                        self.line(format!("nx_sl_u8 {} = {{ {}.ptr + {} / 8, {} / 8 }};", name, buf, bit, sz));
                        if sg.utf8 {
                            self.line(format!("if (!nx_utf8_valid({})) break;", name));
                        }
                    } else {
                        let cn = self.cty(sg.ty);
                        let read = self.bits_read_expr(&buf, &bit, &sz, sg);
                        self.line(format!("{} {} = {};", cn, name, read));
                    }
                }
                TBinSegKind::Value(v) => {
                    if is_bytes {
                        let vc = self.simple(v);
                        self.line(format!("if (({} & 7) || {} != {}.len * 8) break;", bit, sz, vc));
                        self.line(format!("if (memcmp({}.ptr + {} / 8, {}.ptr, {}.len) != 0) break;", buf, bit, vc, vc));
                    } else {
                        let cn = self.cty(sg.ty);
                        let read = self.bits_read_expr(&buf, &bit, &sz, sg);
                        let vc = self.expr(v);
                        self.line(format!("if ((({})({})) != (({})({}))) break;", cn, read, cn, vc));
                    }
                }
            }
            self.line(format!("{} += {};", bit, sz));
        }
        if !all_const && !segs.iter().any(|sg| matches!(sg.size, TBinSize::Rest)) {
            self.line(format!("if ({} != {}) break;", bit, total));
        }
        self.line(format!("{} = true;", flag));
        let inner = self.pop_buf();
        self.body.last_mut().unwrap().push_str(&inner);
        self.line("} while (0);");
        flag
    }

    fn bits_read_expr(&mut self, buf: &str, bit: &str, sz: &str, sg: &TBinSeg) -> String {
        let raw = format!("nx_bits_read({}.ptr, {}, {})", buf, bit, sz);
        let raw = match sg.endian {
            Endian::Little => format!("nx_bswap({}, {} / 8)", raw, sz),
            Endian::Native => format!("(nx_is_little_endian() ? nx_bswap({}, {} / 8) : {})", raw, sz, raw),
            Endian::Big => raw,
        };
        if sg.float {
            if let TBinSize::Bits(32) = sg.size {
                return format!("({{ uint32_t _b = (uint32_t){}; float _f; memcpy(&_f, &_b, 4); _f; }})", raw);
            }
            return format!("({{ uint64_t _b = (uint64_t){}; double _f; memcpy(&_f, &_b, 8); _f; }})", raw);
        }
        if sg.signed {
            return format!("nx_sign_extend({}, {})", raw, sz);
        }
        raw
    }

    fn bin_pattern_bind(&mut self, segs: &[TBinSeg], s: &str, ty: TyId) {
        // bindings were declared inside the test's do-block; redo them in this scope
        let buf = self.bytes_of(s, ty);
        let bit = self.tmp();
        self.line(format!("size_t {} = 0; NX_UNUSED({});", bit, bit));
        let total = format!("({}.len * 8)", buf);
        for sg in segs {
            let size = match &sg.size {
                TBinSize::Bits(b) => b.to_string(),
                TBinSize::Expr(e) => {
                    let v = self.expr(e);
                    format!("((size_t)({}))", v)
                }
                TBinSize::Rest => format!("({} - {})", total, bit),
            };
            let sz = self.tmp();
            self.line(format!("size_t {} = {};", sz, size));
            let is_bytes = matches!(self.kind_of(sg.ty), TyKind::Slice(..)) || matches!(sg.size, TBinSize::Rest);
            if let TBinSegKind::Bind(l) | TBinSegKind::Rest(l) = &sg.kind {
                let name = self.local_name(*l);
                if is_bytes {
                    self.line(format!("nx_sl_u8 {} = {{ {}.ptr + {} / 8, {} / 8 }};", name, buf, bit, sz));
                } else {
                    let cn = self.cty(sg.ty);
                    let read = self.bits_read_expr(&buf, &bit, &sz, sg);
                    self.line(format!("{} {} = {};", cn, name, read));
                }
            }
            self.line(format!("{} += {};", bit, sz));
        }
    }

    fn bin_construct(&mut self, segs: &[TBinSeg], target: &TExpr, e: &TExpr) -> String {
        let buf = self.simple(target);
        let cn = self.cty(e.ty);
        let out = self.tmp();
        self.line(format!("{} {};", cn, out));
        self.line("{");
        self.push_buf();
        let bit = self.tmp();
        self.line(format!("size_t {} = 0;", bit));
        let total = format!("({}.len * 8)", buf);
        let fail = self.p.error_names.iter().position(|n| n == "BufferTooSmall").map(|i| i + 1).unwrap_or(0);
        self.line(format!("{}.err = 0;", out));
        self.line("do {");
        self.push_buf();
        for sg in segs {
            let is_bytes = matches!(self.kind_of(sg.ty), TyKind::Slice(..)) || matches!(sg.size, TBinSize::Rest);
            let v = match &sg.kind {
                TBinSegKind::Value(v) => self.simple(v),
                _ => "0".into(),
            };
            let size = match &sg.size {
                TBinSize::Bits(b) => b.to_string(),
                TBinSize::Expr(x) => {
                    let s = self.expr(x);
                    format!("((size_t)({}))", s)
                }
                TBinSize::Rest => format!("({}.len * 8)", v),
            };
            let sz = self.tmp();
            self.line(format!("size_t {} = {};", sz, size));
            self.line(format!("if ({} + {} > {}) {{ {}.err = {}u; break; }}", bit, sz, total, out, fail));
            if is_bytes {
                self.line(format!("if (({} & 7) || {} > {}.len * 8) {{ {}.err = {}u; break; }}", bit, sz, v, out, fail));
                self.line(format!("memcpy({}.ptr + {} / 8, {}.ptr, {} / 8);", buf, bit, v, sz));
            } else {
                let raw = if sg.float {
                    if let TBinSize::Bits(32) = sg.size {
                        format!("({{ float _f = (float)({}); uint32_t _b; memcpy(&_b, &_f, 4); (uint64_t)_b; }})", v)
                    } else {
                        format!("({{ double _f = (double)({}); uint64_t _b; memcpy(&_b, &_f, 8); _b; }})", v)
                    }
                } else {
                    format!("(uint64_t)({})", v)
                };
                let raw = match sg.endian {
                    Endian::Little => format!("nx_bswap({}, {} / 8)", raw, sz),
                    Endian::Native => format!("(nx_is_little_endian() ? nx_bswap({}, {} / 8) : {})", raw, sz, raw),
                    Endian::Big => raw,
                };
                self.line(format!("nx_bits_write({}.ptr, {}, {}, {});", buf, bit, sz, raw));
            }
            self.line(format!("{} += {};", bit, sz));
        }
        self.line(format!("{}.val = (nx_sl_u8){{ {}.ptr, {} / 8 }};", out, buf, bit));
        let inner = self.pop_buf();
        self.body.last_mut().unwrap().push_str(&inner);
        self.line("} while (0);");
        let inner = self.pop_buf();
        self.body.last_mut().unwrap().push_str(&inner);
        self.line("}");
        out
    }
}
