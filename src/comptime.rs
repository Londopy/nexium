//! Compile-time evaluation: an interpreter over the typed IR (section 7).
//!
//! Permitted: pure computation, collections, calls to Nexium functions.
//! Not permitted: I/O, clocks, randomness, foreign calls, globals. Evaluation
//! carries a step budget; exceeding it is a compile error rather than a hang.

use crate::ast::{BinOp, UnOp};
use crate::check::Checker;
use crate::diag::Span;
use crate::tir::*;
use crate::types::*;
use std::collections::HashMap;

pub const STEP_BUDGET: u64 = 20_000_000;

type Env = HashMap<LocalId, Value>;

enum Flow {
    Next,
    Return(Value),
    Break(LabelId, Option<Value>),
    Continue(LabelId),
}

pub struct Interp<'c, 'a> {
    pub c: &'c mut Checker<'a>,
    steps: u64,
    pub panic: Option<(String, Span)>,
    pub budget_exceeded: bool,
    /// control flow (return/break/continue) escaping from inside an expression
    pending: Option<Flow>,
    /// the first expression the interpreter could not evaluate (for diagnostics)
    pub failed_at: Option<Span>,
    /// the environments of the callers of the running function, outermost
    /// first; a `Value::Ptr` names a local in one of these or in the current one
    frames: Vec<Env>,
    /// the function whose body is executing, for the types of its locals
    cur_fn: Option<InstId>,
    /// temporaries created for `&rvalue`, numbered down from the top of the id space
    temps: u32,
}

/// The root value a pointer refers to: a local in a suspended caller frame or
/// in the current environment (`fi == frames.len()`).
fn frame_root<'e>(frames: &'e mut [Env], env: &'e mut Env, fi: u32, l: LocalId) -> Option<&'e mut Value> {
    if fi as usize == frames.len() {
        env.get_mut(&l)
    } else {
        frames.get_mut(fi as usize)?.get_mut(&l)
    }
}

fn frame_get<'e>(frames: &'e [Env], env: &'e Env, fi: u32, l: LocalId) -> Option<&'e Value> {
    if fi as usize == frames.len() {
        env.get(&l)
    } else {
        frames.get(fi as usize)?.get(&l)
    }
}

/// Evaluate an expression with no locals. Reports compile-time panics as errors.
pub fn eval_const_expr(c: &mut Checker, e: &TExpr) -> Option<Value> {
    let mut it = Interp { c, steps: 0, panic: None, budget_exceeded: false, pending: None, failed_at: None, frames: vec![], cur_fn: None, temps: 0 };
    let mut env = Env::new();
    let r = it.eval(e, &mut env);
    let panic = it.panic.take();
    let exceeded = it.budget_exceeded;
    if let Some((msg, sp)) = panic {
        it.c.error(sp, format!("compile-time evaluation panicked: {}", msg));
    }
    if exceeded {
        it.c.error(e.span, format!("compile-time evaluation exceeded the step budget ({} steps)", STEP_BUDGET));
    }
    r
}

/// Evaluate with one local bound; silent on failure (used to probe record constraints).
pub fn eval_with_local(c: &mut Checker, e: &TExpr, local: LocalId, v: Value) -> Option<Value> {
    let mut it = Interp { c, steps: 0, panic: None, budget_exceeded: false, pending: None, failed_at: None, frames: vec![], cur_fn: None, temps: 0 };
    let mut env = Env::new();
    env.insert(local, v);
    it.eval(e, &mut env)
}

/// Run a function instance at compile time with the given arguments.
pub fn call_instance(c: &mut Checker, inst: InstId, args: Vec<Value>) -> Result<Option<Value>, (String, Span)> {
    let mut it = Interp { c, steps: 0, panic: None, budget_exceeded: false, pending: None, failed_at: None, frames: vec![], cur_fn: None, temps: 0 };
    let r = it.call(inst, args);
    if let Some(p) = it.panic.take() {
        return Err(p);
    }
    if it.budget_exceeded {
        return Err((format!("exceeded the compile-time step budget ({} steps)", STEP_BUDGET), it.c.funcs[inst as usize].span));
    }
    Ok(r)
}

fn int_range(c: &Checker, t: TyId) -> Option<(i128, i128)> {
    let t = c.tys.shallow(t);
    match c.tys.kind(t).clone() {
        TyKind::Int(i) => Some((i.min(), i.max())),
        TyKind::Distinct(d) => {
            let u = c.distinct_underlying[&d];
            int_range(c, u)
        }
        _ => None,
    }
}

fn bits_of(c: &Checker, t: TyId) -> Option<(u32, bool)> {
    let t = c.tys.shallow(t);
    match c.tys.kind(t).clone() {
        TyKind::Int(i) => Some((i.bits(), i.is_signed())),
        TyKind::Distinct(d) => {
            let u = c.distinct_underlying[&d];
            bits_of(c, u)
        }
        _ => None,
    }
}

fn wrap_to(v: i128, bits: u32, signed: bool) -> i128 {
    if bits >= 128 {
        return v;
    }
    let m = 1i128 << bits;
    let mut r = v.rem_euclid(m);
    if signed && r >= m / 2 {
        r -= m;
    }
    r
}

impl<'c, 'a> Interp<'c, 'a> {
    fn step(&mut self) -> bool {
        self.steps += 1;
        if self.steps > STEP_BUDGET {
            self.budget_exceeded = true;
            return false;
        }
        true
    }

    /// A zero-initialized value of the given type, used for `undefined` locals.
    /// Record a panic with a location and stop evaluation.
    fn panic_at(&mut self, msg: &str, sp: Span) -> Option<Value> {
        self.panic = Some((msg.to_string(), sp));
        None
    }

    pub fn default_value(&mut self, ty: TyId) -> Value {
        let ty = self.c.tys.resolve(ty, true);
        match self.c.tys.kind(ty).clone() {
            TyKind::Int(_) => Value::Int(0),
            TyKind::Float(_) => Value::Float(0.0),
            TyKind::Bool => Value::Bool(false),
            TyKind::Char => Value::Char(0),
            TyKind::Array(n, e) => {
                let d = self.default_value(e);
                Value::Array(vec![d; n as usize])
            }
            TyKind::Tuple(ts) => Value::Tuple(ts.iter().map(|&t| self.default_value(t)).collect()),
            TyKind::Struct(_, _) => {
                let f = self.c.struct_field_types(ty);
                Value::Struct(f.iter().map(|&t| self.default_value(t)).collect())
            }
            TyKind::Opt(_) => Value::Opt(None),
            TyKind::List(_) => Value::List(vec![]),
            TyKind::Str => Value::OwnedStr(vec![]),
            TyKind::Distinct(d) => {
                let u = self.c.distinct_underlying[&d];
                self.default_value(u)
            }
            _ => Value::Undefined,
        }
    }

    fn fail_panic(&mut self, msg: impl Into<String>, span: Span) -> Option<Value> {
        if self.panic.is_none() {
            self.panic = Some((msg.into(), span));
        }
        None
    }

    fn ensure_body(&mut self, inst: InstId) -> bool {
        if self.c.funcs[inst as usize].body.is_some() {
            return true;
        }
        if let Some(pos) = self.c.queue.iter().position(|(i, _, _)| *i == inst) {
            let (i, fid, _) = self.c.queue.remove(pos);
            self.c.check_fn_body(i, fid);
            return self.c.funcs[inst as usize].body.is_some();
        }
        if let Some(pos) = self.c.closure_queue.iter().position(|(i, ..)| *i == inst) {
            let item = self.c.closure_queue.remove(pos);
            self.c.check_closure_body(item);
            return self.c.funcs[inst as usize].body.is_some();
        }
        false
    }

    pub fn call(&mut self, inst: InstId, args: Vec<Value>) -> Option<Value> {
        if !self.step() {
            return None;
        }
        if !self.ensure_body(inst) {
            return None;
        }
        let f = self.c.funcs[inst as usize].clone();
        if f.is_extern {
            return None;
        }
        let mut env = Env::new();
        for (p, a) in f.params.iter().zip(args.into_iter()) {
            env.insert(*p, a);
        }
        let body = f.body.as_ref().unwrap();
        let outer = self.cur_fn.replace(inst);
        let r = self.exec_block(body, &mut env);
        self.cur_fn = outer;
        let (flow, v) = r?;
        Some(match flow {
            Flow::Return(v) => v,
            _ => v.unwrap_or(Value::Void),
        })
    }

    /// Call from inside a running function: the caller's environment is
    /// suspended on the frame stack so pointers into it stay valid.
    fn call_in(&mut self, env: &mut Env, inst: InstId, args: Vec<Value>) -> Option<Value> {
        let saved = std::mem::take(env);
        self.frames.push(saved);
        let r = self.call(inst, args);
        *env = self.frames.pop().unwrap_or_default();
        r
    }

    fn exec_block(&mut self, b: &TBlock, env: &mut Env) -> Option<(Flow, Option<Value>)> {
        let mut defers: Vec<&TStmt> = Vec::new();
        let mut result: Option<(Flow, Option<Value>)> = None;
        for s in &b.stmts {
            if let TStmt::Defer { body, .. } = s {
                defers.push(body);
                continue;
            }
            if let TStmt::ErrDefer { .. } = s {
                continue;
            }
            match self.exec_stmt(s, env)? {
                Flow::Next => {}
                other => {
                    result = Some((other, None));
                    break;
                }
            }
        }
        if result.is_none() {
            let v = match &b.tail {
                Some(t) => Some(self.eval(t, env)?),
                None => None,
            };
            result = match self.pending.take() {
                Some(f) => Some((f, None)),
                None => Some((Flow::Next, v)),
            };
        }
        for d in defers.iter().rev() {
            let _ = self.exec_stmt(d, env)?;
        }
        let (flow, v) = result.unwrap();
        // labeled block: a break targeting this block yields its value
        if let (Flow::Break(l, bv), Some(bl)) = (&flow, b.label) {
            if *l == bl {
                return Some((Flow::Next, bv.clone()));
            }
        }
        Some((flow, v))
    }

    fn exec_stmt(&mut self, s: &TStmt, env: &mut Env) -> Option<Flow> {
        let flow = self.exec_stmt_inner(s, env)?;
        if let Some(p) = self.pending.take() {
            return Some(p);
        }
        Some(flow)
    }

    fn exec_stmt_inner(&mut self, s: &TStmt, env: &mut Env) -> Option<Flow> {
        if !self.step() {
            return None;
        }
        match s {
            TStmt::Let { local, init, .. } => {
                let v = match init {
                    Some(TExpr { kind: TExprKind::Undefined, ty, .. }) => {
                        let ty = *ty;
                        self.default_value(ty)
                    }
                    Some(e) => self.eval(e, env)?,
                    None => Value::Undefined,
                };
                env.insert(*local, v);
                Some(Flow::Next)
            }
            TStmt::Assign { target, op, value, span } => {
                let v = self.eval(value, env)?;
                let v = match op {
                    None => v,
                    Some((bop, mode)) => {
                        let cur = self.eval(target, env)?;
                        self.binop(*bop, *mode, cur, v, target.ty, *span)?
                    }
                };
                self.assign(target, v, env)?;
                Some(Flow::Next)
            }
            TStmt::Expr(e) => {
                self.eval(e, env)?;
                Some(Flow::Next)
            }
            TStmt::Return { value, .. } => {
                let v = match value {
                    Some(e) => self.eval(e, env)?,
                    None => Value::Void,
                };
                Some(Flow::Return(v))
            }
            TStmt::Break { label, value, .. } => {
                let v = match value {
                    Some(e) => Some(self.eval(e, env)?),
                    None => None,
                };
                Some(Flow::Break(*label, v))
            }
            TStmt::Continue { label, .. } => Some(Flow::Continue(*label)),
            TStmt::Defer { body, .. } => self.exec_stmt(body, env),
            TStmt::ErrDefer { .. } => Some(Flow::Next),
            TStmt::While { cond, body, els, label, .. } => {
                let mut broke = false;
                loop {
                    if !self.step() {
                        return None;
                    }
                    let c = self.eval(cond, env)?;
                    if c != Value::Bool(true) {
                        break;
                    }
                    match self.exec_block(body, env)?.0 {
                        Flow::Next => {}
                        Flow::Break(l, _) if l == *label => {
                            broke = true;
                            break;
                        }
                        Flow::Continue(l) if l == *label => continue,
                        other => return Some(other),
                    }
                }
                if let (false, Some(eb)) = (broke, els) {
                    return Some(self.exec_block(eb, env)?.0);
                }
                Some(Flow::Next)
            }
            TStmt::ForRange { var, start, end, step, body, label, span } => {
                let s = self.eval(start, env)?.as_int()?;
                let e = self.eval(end, env)?.as_int()?;
                let st = match step {
                    Some(x) => self.eval(x, env)?.as_int()?,
                    None => 1,
                };
                if st == 0 {
                    self.panic = Some(("a range step cannot be zero".into(), *span));
                    return None;
                }
                let mut i = s;
                while if st > 0 { i < e } else { i > e } {
                    if !self.step() {
                        return None;
                    }
                    env.insert(*var, Value::Int(i));
                    match self.exec_block(body, env)?.0 {
                        Flow::Next => {}
                        Flow::Break(l, _) if l == *label => break,
                        Flow::Continue(l) if l == *label => {}
                        other => return Some(other),
                    }
                    i += st;
                }
                Some(Flow::Next)
            }
            TStmt::ForSlice { items, index, body, label, .. } => {
                let mut seqs = Vec::new();
                for (l, e) in items {
                    let v = self.eval(e, env)?;
                    let elems = match v {
                        Value::Array(a) | Value::List(a) => a,
                        Value::Str(s) | Value::OwnedStr(s) => s.into_iter().map(|b| Value::Int(b as i128)).collect(),
                        _ => return None,
                    };
                    seqs.push((*l, elems));
                }
                let n = seqs.iter().map(|(_, s)| s.len()).min().unwrap_or(0);
                if seqs.iter().any(|(_, s)| s.len() != n) {
                    return self.fail_panic("parallel iteration over slices of different lengths", body.span).map(|_| Flow::Next);
                }
                for i in 0..n {
                    if !self.step() {
                        return None;
                    }
                    for (l, s) in &seqs {
                        env.insert(*l, s[i].clone());
                    }
                    if let Some(ix) = index {
                        env.insert(*ix, Value::Int(i as i128));
                    }
                    match self.exec_block(body, env)?.0 {
                        Flow::Next => {}
                        Flow::Break(l, _) if l == *label => break,
                        Flow::Continue(l) if l == *label => {}
                        other => return Some(other),
                    }
                }
                Some(Flow::Next)
            }
            TStmt::Block(b) | TStmt::Using { body: b, .. } => Some(self.exec_block(b, env)?.0),
            TStmt::Drop { .. } => Some(Flow::Next),
        }
    }

    /// The place a mutating builtin acts on: the receiver place, following
    /// any pointer stored there (a `*mut List(T)` local names the list).
    fn recv_place(&mut self, e: &TExpr, env: &mut Env) -> Option<(u32, LocalId, Vec<usize>)> {
        let (mut fi, mut l, mut path) = self.place_path(e, env)?;
        loop {
            let root = frame_get(&self.frames, env, fi, l)?;
            match self.read_path(root, &path)? {
                Value::Ptr(f2, l2, p2) => {
                    fi = f2;
                    l = l2;
                    path = p2;
                }
                _ => return Some((fi, l, path)),
            }
        }
    }

    /// Resolve a place expression to a root local and a path of indices.
    fn place_path(&mut self, e: &TExpr, env: &mut Env) -> Option<(u32, LocalId, Vec<usize>)> {
        match &e.kind {
            TExprKind::Local(l) => Some((self.frames.len() as u32, *l, vec![])),
            TExprKind::Field { base, idx } | TExprKind::TupleField { base, idx } | TExprKind::RefField { base, idx } => {
                let (fi, l, mut p) = self.place_path(base, env)?;
                p.push(*idx as usize);
                Some((fi, l, p))
            }
            TExprKind::Index { base, index, .. } => {
                let i = self.eval(index, env)?.as_int()?;
                let (fi, l, mut p) = self.place_path(base, env)?;
                p.push(i as usize);
                Some((fi, l, p))
            }
            TExprKind::Deref(p) => match self.eval(p, env)? {
                Value::Ptr(fi, l, path) => Some((fi, l, path)),
                _ => None,
            },
            TExprKind::ListToSlice(inner) | TExprKind::ArrayToSlice(inner) | TExprKind::StrToSlice(inner) => self.place_path(inner, env),
            _ => None,
        }
    }

    fn assign(&mut self, target: &TExpr, v: Value, env: &mut Env) -> Option<()> {
        let (fi, l, path) = self.place_path(target, env)?;
        let root = frame_root(&mut self.frames, env, fi, l)?;
        let mut cur = root;
        for (k, &step) in path.iter().enumerate() {
            let last = k == path.len() - 1;
            let next: &mut Value = match cur {
                Value::Array(a) | Value::List(a) | Value::Tuple(a) | Value::Struct(a) => a.get_mut(step)?,
                Value::Enum(_, a) => a.get_mut(step)?,
                Value::Opt(Some(b)) | Value::Ok(b) if step == 0 => b.as_mut(),
                Value::OwnedStr(s) => {
                    if last {
                        *s.get_mut(step)? = v.as_int()? as u8;
                        return Some(());
                    }
                    return None;
                }
                _ => return None,
            };
            cur = next;
        }
        *cur = v;
        Some(())
    }

    fn read_path(&self, root: &Value, path: &[usize]) -> Option<Value> {
        let mut cur = root;
        for &step in path {
            cur = match cur {
                Value::Array(a) | Value::List(a) | Value::Tuple(a) | Value::Struct(a) => a.get(step)?,
                Value::Enum(_, a) => a.get(step)?,
                Value::Opt(Some(b)) | Value::Ok(b) if step == 0 => b.as_ref(),
                Value::OwnedStr(s) | Value::Str(s) => return Some(Value::Int(*s.get(step)? as i128)),
                _ => return None,
            };
        }
        Some(cur.clone())
    }

    fn escape(&mut self, flow: Flow) -> Option<Value> {
        self.pending = Some(flow);
        Some(Value::Never)
    }

    pub fn eval(&mut self, e: &TExpr, env: &mut Env) -> Option<Value> {
        let r = self.eval_inner(e, env);
        if r.is_none() && self.panic.is_none() && !self.budget_exceeded && self.failed_at.is_none() {
            self.failed_at = Some(e.span);
        }
        r
    }

    fn eval_inner(&mut self, e: &TExpr, env: &mut Env) -> Option<Value> {
        if !self.step() {
            return None;
        }
        if self.pending.is_some() {
            return Some(Value::Never);
        }
        match &e.kind {
            TExprKind::Int(v) => Some(Value::Int(*v)),
            TExprKind::Float(f) => Some(Value::Float(*f)),
            TExprKind::Bool(b) => Some(Value::Bool(*b)),
            TExprKind::Char(c) => Some(Value::Char(*c)),
            TExprKind::Str(s) => Some(Value::Str(s.clone())),
            TExprKind::Unit => Some(Value::Void),
            TExprKind::Value(v) => Some(v.clone()),
            TExprKind::Local(l) => env.get(l).cloned().or(Some(Value::Undefined)),
            TExprKind::Global(_) => None,
            TExprKind::Const(id) => {
                let (_, te) = self.c.resolve_const(*id)?;
                match te.kind {
                    TExprKind::Value(v) => Some(v),
                    _ => None,
                }
            }
            TExprKind::FnRef(i) | TExprKind::FnToFat(i) => Some(Value::Fn(*i)),
            TExprKind::Field { base, idx } | TExprKind::RefField { base, idx } | TExprKind::TupleField { base, idx } => {
                let b = self.eval(base, env)?;
                self.read_path(&b, &[*idx as usize])
            }
            TExprKind::Index { base, index, .. } => {
                let b = self.eval(base, env)?;
                let i = self.eval(index, env)?.as_int()?;
                let len = match &b {
                    Value::Array(a) | Value::List(a) => a.len(),
                    Value::Str(s) | Value::OwnedStr(s) => s.len(),
                    _ => return None,
                };
                if i < 0 || i as usize >= len {
                    return self.fail_panic(format!("index {} out of bounds for length {}", i, len), e.span);
                }
                self.read_path(&b, &[i as usize])
            }
            TExprKind::SliceOp { base, start, end } => {
                let b = self.eval(base, env)?;
                let s = match start {
                    Some(s) => self.eval(s, env)?.as_int()? as usize,
                    None => 0,
                };
                let len = match &b {
                    Value::Array(a) | Value::List(a) => a.len(),
                    Value::Str(x) | Value::OwnedStr(x) => x.len(),
                    _ => return None,
                };
                let en = match end {
                    Some(x) => self.eval(x, env)?.as_int()? as usize,
                    None => len,
                };
                if s > en || en > len {
                    return self.fail_panic(format!("slice bounds {}..{} out of range for length {}", s, en, len), e.span);
                }
                Some(match b {
                    Value::Array(a) | Value::List(a) => Value::Array(a[s..en].to_vec()),
                    Value::Str(x) | Value::OwnedStr(x) => Value::Str(x[s..en].to_vec()),
                    _ => return None,
                })
            }
            TExprKind::Deref(p) => match self.eval(p, env)? {
                Value::Ptr(fi, l, path) => {
                    let root = frame_get(&self.frames, env, fi, l)?.clone();
                    self.read_path(&root, &path)
                }
                _ => None,
            },
            TExprKind::AddrOf { expr, .. } => match self.place_path(expr, env) {
                Some((fi, l, path)) => Some(Value::Ptr(fi, l, path)),
                None => {
                    // the address of a temporary: give the value a slot of its own
                    let v = self.eval(expr, env)?;
                    let id = u32::MAX - self.temps;
                    self.temps += 1;
                    env.insert(id, v);
                    Some(Value::Ptr(self.frames.len() as u32, id, vec![]))
                }
            },
            TExprKind::Call { inst, args } => {
                let mut vs = Vec::new();
                for a in args {
                    vs.push(self.eval(a, env)?);
                }
                self.call_in(env, *inst, vs)
            }
            TExprKind::CallPtr { callee, args } => {
                let f = self.eval(callee, env)?;
                let mut vs = Vec::new();
                for a in args {
                    vs.push(self.eval(a, env)?);
                }
                match f {
                    Value::Fn(i) => self.call_in(env, i, vs),
                    _ => None,
                }
            }
            TExprKind::Builtin { op, args, tys } => self.eval_builtin(*op, args, tys, e, env),
            TExprKind::Unary { op, expr, mode } => {
                let v = self.eval(expr, env)?;
                match (op, v) {
                    (UnOp::Not, Value::Bool(b)) => Some(Value::Bool(!b)),
                    (UnOp::Neg, Value::Float(f)) => Some(Value::Float(-f)),
                    (UnOp::Neg, Value::Int(i)) => {
                        let r = -i;
                        if *mode == ArithMode::Checked {
                            if let Some((lo, hi)) = int_range(self.c, e.ty) {
                                if r < lo || r > hi {
                                    return self.fail_panic("integer overflow in negation", e.span);
                                }
                            }
                        }
                        Some(Value::Int(r))
                    }
                    (UnOp::BitNot, Value::Int(i)) => {
                        let (bits, signed) = bits_of(self.c, e.ty).unwrap_or((64, true));
                        Some(Value::Int(wrap_to(!i, bits, signed)))
                    }
                    (UnOp::BitNot, Value::Bool(b)) => Some(Value::Bool(!b)),
                    _ => None,
                }
            }
            TExprKind::Binary { op, lhs, rhs, mode, .. } => {
                let a = self.eval(lhs, env)?;
                let b = self.eval(rhs, env)?;
                let ty = lhs.ty;
                self.binop(*op, *mode, a, b, ty, e.span)
            }
            TExprKind::Logical { and, lhs, rhs } => {
                let a = self.eval(lhs, env)?.as_bool()?;
                if *and && !a {
                    return Some(Value::Bool(false));
                }
                if !*and && a {
                    return Some(Value::Bool(true));
                }
                let b = self.eval(rhs, env)?.as_bool()?;
                Some(Value::Bool(b))
            }
            TExprKind::Cast { expr, kind } => {
                let v = self.eval(expr, env)?;
                match kind {
                    CastKind::IntToInt { checked } => {
                        let i = v.as_int()?;
                        if let Some((lo, hi)) = int_range(self.c, e.ty) {
                            if i < lo || i > hi {
                                if *checked {
                                    return self.fail_panic(format!("value {} does not fit the target type", i), e.span);
                                }
                                let (bits, signed) = bits_of(self.c, e.ty)?;
                                return Some(Value::Int(wrap_to(i, bits, signed)));
                            }
                        }
                        if matches!(self.c.tys.kind(self.c.tys.shallow(e.ty)), TyKind::Char) {
                            return Some(Value::Char(i as u32));
                        }
                        Some(Value::Int(i))
                    }
                    CastKind::IntToFloat => Some(Value::Float(v.as_int()? as f64)),
                    CastKind::FloatToInt => {
                        let f = v.as_float()?;
                        let i = f.trunc() as i128;
                        if let Some((lo, hi)) = int_range(self.c, e.ty) {
                            if !f.is_finite() || i < lo || i > hi {
                                return self.fail_panic("float to integer cast out of range", e.span);
                            }
                        }
                        Some(Value::Int(i))
                    }
                    CastKind::FloatToFloat => Some(Value::Float(v.as_float()?)),
                    CastKind::Bits => match v {
                        Value::Bool(b) => Some(Value::Int(b as i128)),
                        Value::Char(c) => Some(Value::Int(c as i128)),
                        Value::Enum(i, _) => Some(Value::Int(i as i128)),
                        other => Some(other),
                    },
                    CastKind::PtrToPtr => None,
                }
            }
            TExprKind::If { cond, then, els } => {
                let c = self.eval(cond, env)?.as_bool()?;
                let b = if c {
                    then
                } else {
                    match els {
                        Some(b) => b,
                        None => return Some(Value::Void),
                    }
                };
                let (flow, v) = self.exec_block(b, env)?;
                match flow {
                    Flow::Next => Some(v.unwrap_or(Value::Void)),
                    other => self.escape(other),
                }
            }
            TExprKind::IfCapture { cond, local, then, els } => {
                let c = self.eval(cond, env)?;
                match c {
                    Value::Opt(Some(v)) => {
                        env.insert(*local, *v);
                        let (flow, v) = self.exec_block(then, env)?;
                        match flow {
                            Flow::Next => Some(v.unwrap_or(Value::Void)),
                            other => self.escape(other),
                        }
                    }
                    Value::Opt(None) => match els {
                        Some(b) => {
                            let (flow, v) = self.exec_block(b, env)?;
                            match flow {
                                Flow::Next => Some(v.unwrap_or(Value::Void)),
                                other => self.escape(other),
                            }
                        }
                        None => Some(Value::Void),
                    },
                    _ => None,
                }
            }
            TExprKind::Match { scrutinee, arms } => {
                let v = self.eval(scrutinee, env)?;
                // `match p.*`: payload bindings typed as pointers point into the place
                let place = match &scrutinee.kind {
                    TExprKind::Deref(_) => self.place_path(scrutinee, env),
                    _ => None,
                };
                for arm in arms {
                    if self.match_pat(&arm.pat, &v, place.as_ref(), env)? {
                        if let Some(g) = &arm.guard {
                            if !self.eval(g, env)?.as_bool()? {
                                continue;
                            }
                        }
                        return self.eval(&arm.body, env);
                    }
                }
                self.fail_panic("no match arm matched", e.span)
            }
            TExprKind::Block(b) => {
                let (flow, v) = self.exec_block(b, env)?;
                match flow {
                    Flow::Next => Some(v.unwrap_or(Value::Void)),
                    other => self.escape(other),
                }
            }
            TExprKind::StructLit { fields } | TExprKind::RefNew { fields } => {
                let n = fields.iter().map(|(i, _)| *i as usize + 1).max().unwrap_or(0);
                let mut vals = vec![Value::Undefined; n];
                for (i, f) in fields {
                    vals[*i as usize] = self.eval(f, env)?;
                }
                Some(Value::Struct(vals))
            }
            TExprKind::EnumLit { variant, payload } => {
                let mut vals = Vec::new();
                for p in payload {
                    vals.push(self.eval(p, env)?);
                }
                Some(Value::Enum(*variant, vals))
            }
            TExprKind::ArrayLit(elems) => {
                let mut vals = Vec::new();
                for x in elems {
                    vals.push(self.eval(x, env)?);
                }
                Some(Value::Array(vals))
            }
            TExprKind::ArrayRepeat { value, count } => {
                let v = self.eval(value, env)?;
                Some(Value::Array(vec![v; *count as usize]))
            }
            TExprKind::TupleLit(elems) => {
                let mut vals = Vec::new();
                for x in elems {
                    vals.push(self.eval(x, env)?);
                }
                Some(Value::Tuple(vals))
            }
            TExprKind::Try(inner) => match self.eval(inner, env)? {
                Value::Ok(v) => Some(*v),
                Value::Err(id) => self.escape(Flow::Return(Value::Err(id))),
                other => Some(other),
            },
            TExprKind::Catch { expr, err_local, handler } => match self.eval(expr, env)? {
                Value::Ok(v) => Some(*v),
                Value::Err(id) => {
                    if let Some(l) = err_local {
                        env.insert(*l, Value::Err(id));
                    }
                    self.eval(handler, env)
                }
                other => Some(other),
            },
            TExprKind::OrElse { expr, default } => match self.eval(expr, env)? {
                Value::Opt(Some(v)) => Some(*v),
                Value::Opt(None) => self.eval(default, env),
                _ => None,
            },
            TExprKind::Unwrap { expr, .. } => match self.eval(expr, env)? {
                Value::Opt(Some(v)) => Some(*v),
                Value::Opt(None) => self.fail_panic("unwrapped a null optional", e.span),
                _ => None,
            },
            TExprKind::OptWrap(inner) => Some(Value::Opt(Some(Box::new(self.eval(inner, env)?)))),
            TExprKind::OptNull => Some(Value::Opt(None)),
            TExprKind::ErrWrap(inner) => Some(Value::Ok(Box::new(self.eval(inner, env)?))),
            TExprKind::ErrToUnion(inner) => self.eval(inner, env),
            TExprKind::ErrVal(id) => Some(Value::Err(*id)),
            TExprKind::ArrayToSlice(inner) => match self.eval(inner, env)? {
                Value::Array(a) => Some(Value::Array(a)),
                Value::Str(s) => Some(Value::Str(s)),
                _ => None,
            },
            TExprKind::ListToSlice(inner) => match self.eval(inner, env)? {
                Value::List(a) | Value::Array(a) => Some(Value::Array(a)),
                _ => None,
            },
            TExprKind::StrToSlice(inner) => match self.eval(inner, env)? {
                Value::OwnedStr(s) | Value::Str(s) => Some(Value::Str(s)),
                _ => None,
            },
            TExprKind::Closure { .. } => None,
            TExprKind::Unreachable => self.fail_panic("reached `unreachable`", e.span),
            TExprKind::Undefined => Some(Value::Undefined),
            TExprKind::BinConstruct { .. } => None,
            TExprKind::TypeVal(t) => Some(Value::Type(*t)),
            TExprKind::RecordCheck { value, checks, as_error } => {
                let v = self.eval(value, env)?;
                for (idx, ce, local) in checks {
                    let fv = self.read_path(&v, &[*idx as usize])?;
                    env.insert(*local, fv);
                    let ok = self.eval(ce, env)?.as_bool()?;
                    if !ok {
                        if *as_error {
                            let id = self.c.error_id("InvalidRecord");
                            return Some(Value::Err(id));
                        }
                        return self.fail_panic("record constraint violated", e.span);
                    }
                }
                if *as_error {
                    Some(Value::Ok(Box::new(v)))
                } else {
                    Some(v)
                }
            }
            TExprKind::Retained(inner) => self.eval(inner, env),
            TExprKind::DynFrom { .. } | TExprKind::DynCall { .. } => None,
        }
    }

    fn match_pat(&mut self, p: &TPat, v: &Value, place: Option<&(u32, LocalId, Vec<usize>)>, env: &mut Env) -> Option<bool> {
        Some(match p {
            TPat::Wild => true,
            TPat::Bind(l) => {
                let by_ptr = match (place, self.cur_fn) {
                    (Some(_), Some(f)) => {
                        let lt = self.c.funcs[f as usize].locals[*l as usize].ty;
                        matches!(self.c.tys.kind(self.c.tys.shallow(lt)), TyKind::Ptr(..)) && !matches!(v, Value::Ptr(..))
                    }
                    _ => false,
                };
                match (by_ptr, place) {
                    (true, Some((fi, root, path))) => env.insert(*l, Value::Ptr(*fi, *root, path.clone())),
                    _ => env.insert(*l, v.clone()),
                };
                true
            }
            TPat::Int(i) => v.as_int() == Some(*i),
            TPat::Float(f) => v.as_float() == Some(*f),
            TPat::Bool(b) => *v == Value::Bool(*b),
            TPat::Char(c) => match v {
                Value::Char(x) => x == c,
                Value::Int(i) => *i == *c as i128,
                _ => false,
            },
            TPat::Str(s) => match v {
                Value::Str(x) | Value::OwnedStr(x) => x == s,
                _ => false,
            },
            TPat::Range { lo, hi, inclusive } => {
                let i = v.as_int()?;
                i >= *lo && (if *inclusive { i <= *hi } else { i < *hi })
            }
            TPat::Variant { idx, args } => match v {
                Value::Enum(i, payload) if i == idx => {
                    for (k, (a, pv)) in args.iter().zip(payload.iter()).enumerate() {
                        let sub = place.map(|(fi, l, p)| (*fi, *l, [p.as_slice(), &[k]].concat()));
                        if !self.match_pat(a, pv, sub.as_ref(), env)? {
                            return Some(false);
                        }
                    }
                    true
                }
                _ => false,
            },
            TPat::Error(id) => matches!(v, Value::Err(x) if x == id),
            TPat::Null => matches!(v, Value::Opt(None)),
            TPat::Some(inner) => match v {
                Value::Opt(Some(x)) => {
                    let sub = place.map(|(fi, l, p)| (*fi, *l, [p.as_slice(), &[0]].concat()));
                    self.match_pat(inner, x, sub.as_ref(), env)?
                }
                _ => false,
            },
            TPat::Ok(inner) => match v {
                Value::Ok(x) => {
                    let sub = place.map(|(fi, l, p)| (*fi, *l, [p.as_slice(), &[0]].concat()));
                    self.match_pat(inner, x, sub.as_ref(), env)?
                }
                Value::Err(_) => false,
                other => self.match_pat(inner, other, place, env)?,
            },
            TPat::Or(alts) => {
                for a in alts {
                    if self.match_pat(a, v, place, env)? {
                        return Some(true);
                    }
                }
                false
            }
            TPat::Tuple(ps) => match v {
                Value::Tuple(vs) if vs.len() == ps.len() => {
                    for (k, (a, pv)) in ps.iter().zip(vs.iter()).enumerate() {
                        let sub = place.map(|(fi, l, p)| (*fi, *l, [p.as_slice(), &[k]].concat()));
                        if !self.match_pat(a, pv, sub.as_ref(), env)? {
                            return Some(false);
                        }
                    }
                    true
                }
                _ => false,
            },
            TPat::Binary(segs) => {
                let bytes = match v {
                    Value::Str(b) | Value::OwnedStr(b) => b.clone(),
                    Value::Array(a) => a.iter().map(|x| x.as_int().unwrap_or(0) as u8).collect(),
                    _ => return None,
                };
                return self.match_binary(segs, &bytes, env);
            }
        })
    }

    fn match_binary(&mut self, segs: &[TBinSeg], bytes: &[u8], env: &mut Env) -> Option<bool> {
        let mut bit = 0usize;
        let total_bits = bytes.len() * 8;
        let read_bits = |bytes: &[u8], start: usize, n: usize| -> u128 {
            let mut v: u128 = 0;
            for i in 0..n {
                let b = start + i;
                let bitv = (bytes[b / 8] >> (7 - (b % 8))) & 1;
                v = (v << 1) | bitv as u128;
            }
            v
        };
        for seg in segs {
            let size_bits = match &seg.size {
                TBinSize::Bits(b) => *b as usize,
                TBinSize::Expr(e) => self.eval(e, env)?.as_int()? as usize,
                TBinSize::Rest => total_bits - bit,
            };
            if bit + size_bits > total_bits {
                return Some(false);
            }
            let is_bytes = matches!(self.c.tys.kind(self.c.tys.shallow(seg.ty)), TyKind::Slice(..)) || matches!(seg.size, TBinSize::Rest);
            if is_bytes {
                if bit % 8 != 0 || size_bits % 8 != 0 {
                    return Some(false);
                }
                let sl = bytes[bit / 8..bit / 8 + size_bits / 8].to_vec();
                match &seg.kind {
                    TBinSegKind::Bind(l) | TBinSegKind::Rest(l) => {
                        env.insert(*l, Value::Str(sl));
                    }
                    TBinSegKind::Value(e) => {
                        let want = match self.eval(e, env)? {
                            Value::Str(s) | Value::OwnedStr(s) => s,
                            _ => return None,
                        };
                        if want != sl {
                            return Some(false);
                        }
                    }
                }
            } else {
                let raw = read_bits(bytes, bit, size_bits);
                let raw = if seg.endian == Endian::Little && size_bits % 8 == 0 && size_bits > 8 {
                    let mut r: u128 = 0;
                    for i in 0..size_bits / 8 {
                        r |= ((raw >> (i * 8)) & 0xff) << ((size_bits / 8 - 1 - i) * 8);
                    }
                    r
                } else {
                    raw
                };
                let val = if seg.float {
                    Value::Float(if size_bits == 32 { f32::from_bits(raw as u32) as f64 } else { f64::from_bits(raw as u64) })
                } else if seg.signed && size_bits < 128 {
                    let m = 1i128 << size_bits;
                    let mut r = raw as i128;
                    if r >= m / 2 {
                        r -= m;
                    }
                    Value::Int(r)
                } else {
                    Value::Int(raw as i128)
                };
                match &seg.kind {
                    TBinSegKind::Bind(l) | TBinSegKind::Rest(l) => {
                        env.insert(*l, val);
                    }
                    TBinSegKind::Value(e) => {
                        let want = self.eval(e, env)?;
                        let eq = match (&want, &val) {
                            (Value::Float(a), Value::Float(b)) => a == b,
                            _ => want.as_int() == val.as_int(),
                        };
                        if !eq {
                            return Some(false);
                        }
                    }
                }
            }
            bit += size_bits;
        }
        Some(bit == total_bits || segs.iter().any(|s| matches!(s.size, TBinSize::Rest)))
    }

    fn binop(&mut self, op: BinOp, mode: ArithMode, a: Value, b: Value, ty: TyId, span: Span) -> Option<Value> {
        use BinOp::*;
        if op.is_comparison() {
            let r = match (&a, &b) {
                (Value::Float(x), Value::Float(y)) => x.partial_cmp(y),
                (Value::Str(x), Value::Str(y)) | (Value::OwnedStr(x), Value::OwnedStr(y)) | (Value::Str(x), Value::OwnedStr(y)) | (Value::OwnedStr(x), Value::Str(y)) => Some(x.cmp(y)),
                (Value::Opt(None), Value::Opt(None)) => Some(std::cmp::Ordering::Equal),
                (Value::Opt(_), Value::Opt(None)) | (Value::Opt(None), Value::Opt(_)) => Some(std::cmp::Ordering::Less),
                (Value::Enum(i, _), Value::Enum(j, _)) => Some(i.cmp(j)),
                (Value::Err(i), Value::Err(j)) => Some(i.cmp(j)),
                (Value::Struct(x), Value::Struct(y)) | (Value::Array(x), Value::Array(y)) => {
                    if x == y {
                        Some(std::cmp::Ordering::Equal)
                    } else {
                        Some(std::cmp::Ordering::Less)
                    }
                }
                _ => match (a.as_int(), b.as_int()) {
                    (Some(x), Some(y)) => Some(x.cmp(&y)),
                    _ => return None,
                },
            };
            let r = match r {
                Some(o) => o,
                None => return Some(Value::Bool(op == Ne)), // NaN
            };
            use std::cmp::Ordering::*;
            return Some(Value::Bool(match op {
                Eq => r == Equal,
                Ne => r != Equal,
                Lt => r == Less,
                Le => r != Greater,
                Gt => r == Greater,
                Ge => r != Less,
                _ => unreachable!(),
            }));
        }
        if let (Value::Bool(x), Value::Bool(y)) = (&a, &b) {
            return Some(Value::Bool(match op {
                BitAnd => x & y,
                BitOr => x | y,
                BitXor => x ^ y,
                _ => return None,
            }));
        }
        if mode == ArithMode::Float {
            let (x, y) = (a.as_float()?, b.as_float()?);
            return Some(Value::Float(match op {
                Add => x + y,
                Sub => x - y,
                Mul => x * y,
                Div => x / y,
                Rem => x % y,
                _ => return None,
            }));
        }
        let (x, y) = (a.as_int()?, b.as_int()?);
        let (bits, signed) = bits_of(self.c, ty).unwrap_or((64, true));
        let (lo, hi) = int_range(self.c, ty).unwrap_or((i64::MIN as i128, i64::MAX as i128));
        let raw = match op {
            Add | AddWrap | AddSat => x.checked_add(y),
            Sub | SubWrap | SubSat => x.checked_sub(y),
            Mul | MulWrap | MulSat => x.checked_mul(y),
            Div => {
                if y == 0 {
                    return self.fail_panic("division by zero", span);
                }
                x.checked_div(y)
            }
            Rem => {
                if y == 0 {
                    return self.fail_panic("remainder by zero", span);
                }
                x.checked_rem(y)
            }
            BitAnd => Some(x & y),
            BitOr => Some(x | y),
            BitXor => Some(x ^ y),
            Shl => {
                if y < 0 || y as u32 >= bits {
                    return self.fail_panic("shift amount exceeds the bit width", span);
                }
                Some(wrap_to(x << y, bits, signed))
            }
            Shr => {
                if y < 0 || y as u32 >= bits {
                    return self.fail_panic("shift amount exceeds the bit width", span);
                }
                Some(x >> y)
            }
            _ => None,
        };
        let r = match raw {
            Some(r) => r,
            None => return self.fail_panic("integer overflow", span),
        };
        Some(Value::Int(match mode {
            ArithMode::Checked => {
                if r < lo || r > hi {
                    return self.fail_panic(format!("integer overflow: {} {} {} does not fit", x, op.symbol(), y), span);
                }
                r
            }
            ArithMode::Wrap => wrap_to(r, bits, signed),
            ArithMode::Sat => r.clamp(lo, hi),
            _ => r,
        }))
    }

    fn eval_builtin(&mut self, op: Builtin, args: &[TExpr], _tys: &[TyId], e: &TExpr, env: &mut Env) -> Option<Value> {
        let mut vs = Vec::new();
        for a in args {
            vs.push(self.eval(a, env)?);
        }
        let seq_len = |v: &Value| -> Option<usize> {
            Some(match v {
                Value::Array(a) | Value::List(a) => a.len(),
                Value::Str(s) | Value::OwnedStr(s) => s.len(),
                _ => return None,
            })
        };
        match op {
            Builtin::Len if matches!(&vs[0], Value::Map(_)) => match &vs[0] {
                Value::Map(kv) => Some(Value::Int(kv.len() as i128)),
                _ => None,
            },
            Builtin::Len => Some(Value::Int(seq_len(&vs[0])? as i128)),
            // ---- lists (mutating through the receiver place)
            Builtin::ListClear => {
                let (fi, l, path) = self.recv_place(&args[0], env)?;
                match walk(frame_root(&mut self.frames, env, fi, l)?, &path)? {
                    Value::List(a) => a.clear(),
                    _ => return None,
                }
                Some(Value::Void)
            }
            Builtin::ListInsert => {
                let (fi, l, path) = self.recv_place(&args[0], env)?;
                let i = vs[1].as_int()? as usize;
                let v = vs[2].clone();
                let bad = match walk(frame_root(&mut self.frames, env, fi, l)?, &path)? {
                    Value::List(a) if i <= a.len() => {
                        a.insert(i, v);
                        false
                    }
                    Value::List(_) => true,
                    _ => return None,
                };
                if bad {
                    return self.panic_at("insert index out of range", e.span);
                }
                Some(Value::Void)
            }
            Builtin::ListRemove | Builtin::ListSwapRemove => {
                let (fi, l, path) = self.recv_place(&args[0], env)?;
                let i = vs[1].as_int()? as usize;
                let r = match walk(frame_root(&mut self.frames, env, fi, l)?, &path)? {
                    Value::List(a) if i < a.len() => Some(if op == Builtin::ListRemove { a.remove(i) } else { a.swap_remove(i) }),
                    Value::List(_) => None,
                    _ => return None,
                };
                match r {
                    Some(v) => Some(v),
                    None => self.panic_at("remove index out of range", e.span),
                }
            }
            Builtin::ListExtend => {
                let (fi, l, path) = self.recv_place(&args[0], env)?;
                let more: Vec<Value> = match &vs[1] {
                    Value::List(a) | Value::Array(a) => a.clone(),
                    Value::Str(s) | Value::OwnedStr(s) => s.iter().map(|b| Value::Int(*b as i128)).collect(),
                    _ => return None,
                };
                match walk(frame_root(&mut self.frames, env, fi, l)?, &path)? {
                    Value::List(a) => a.extend(more),
                    _ => return None,
                }
                Some(Value::Void)
            }
            Builtin::ListReserve => Some(Value::Void),
            Builtin::StringClear => {
                let (fi, l, path) = self.recv_place(&args[0], env)?;
                match walk(frame_root(&mut self.frames, env, fi, l)?, &path)? {
                    Value::OwnedStr(s) | Value::Str(s) => s.clear(),
                    _ => return None,
                }
                Some(Value::Void)
            }
            Builtin::StringPop => {
                let (fi, l, path) = self.recv_place(&args[0], env)?;
                match walk(frame_root(&mut self.frames, env, fi, l)?, &path)? {
                    Value::OwnedStr(s) | Value::Str(s) => Some(Value::Opt(s.pop().map(|b| Box::new(Value::Int(b as i128))))),
                    _ => None,
                }
            }
            // ---- maps
            Builtin::MapNew => Some(Value::Map(vec![])),
            Builtin::MapPut => {
                let (fi, l, path) = self.recv_place(&args[0], env)?;
                let (k, v) = (vs[1].clone(), vs[2].clone());
                match walk(frame_root(&mut self.frames, env, fi, l)?, &path)? {
                    Value::Map(kv) => match kv.iter_mut().find(|(kk, _)| key_eq(kk, &k)) {
                        Some(slot) => slot.1 = v,
                        None => kv.push((k, v)),
                    },
                    slot @ Value::Undefined => *slot = Value::Map(vec![(k, v)]),
                    _ => return None,
                }
                Some(Value::Void)
            }
            Builtin::MapGet => match &vs[0] {
                Value::Map(kv) => Some(Value::Opt(kv.iter().find(|(k, _)| key_eq(k, &vs[1])).map(|(_, v)| Box::new(v.clone())))),
                _ => None,
            },
            Builtin::MapContains => match &vs[0] {
                Value::Map(kv) => Some(Value::Bool(kv.iter().any(|(k, _)| key_eq(k, &vs[1])))),
                _ => None,
            },
            Builtin::MapRemove => {
                let (fi, l, path) = self.recv_place(&args[0], env)?;
                match walk(frame_root(&mut self.frames, env, fi, l)?, &path)? {
                    Value::Map(kv) => {
                        let before = kv.len();
                        kv.retain(|(k, _)| !key_eq(k, &vs[1]));
                        Some(Value::Bool(kv.len() != before))
                    }
                    _ => None,
                }
            }
            Builtin::MapClear => {
                let (fi, l, path) = self.recv_place(&args[0], env)?;
                match walk(frame_root(&mut self.frames, env, fi, l)?, &path)? {
                    Value::Map(kv) => kv.clear(),
                    _ => return None,
                }
                Some(Value::Void)
            }
            Builtin::MapKeys => match &vs[0] {
                Value::Map(kv) => Some(Value::List(kv.iter().map(|(k, _)| k.clone()).collect())),
                _ => None,
            },
            Builtin::MapValues => match &vs[0] {
                Value::Map(kv) => Some(Value::List(kv.iter().map(|(_, v)| v.clone()).collect())),
                _ => None,
            },
            // ---- slices
            Builtin::SliceCopy => {
                let (fi, l, path) = self.recv_place(&args[0], env)?;
                let src: Vec<Value> = seq_items(&vs[1])?;
                match walk(frame_root(&mut self.frames, env, fi, l)?, &path)? {
                    Value::List(a) | Value::Array(a) => {
                        for (d, s) in a.iter_mut().zip(src.into_iter()) {
                            *d = s;
                        }
                    }
                    Value::OwnedStr(s) | Value::Str(s) => {
                        for (d, v) in s.iter_mut().zip(src.into_iter()) {
                            *d = v.as_int()? as u8;
                        }
                    }
                    _ => return None,
                }
                Some(Value::Void)
            }
            Builtin::SliceFill => {
                let (fi, l, path) = self.recv_place(&args[0], env)?;
                let v = vs[1].clone();
                match walk(frame_root(&mut self.frames, env, fi, l)?, &path)? {
                    Value::List(a) | Value::Array(a) => {
                        for d in a.iter_mut() {
                            *d = v.clone();
                        }
                    }
                    Value::OwnedStr(s) | Value::Str(s) => {
                        let b = v.as_int()? as u8;
                        for d in s.iter_mut() {
                            *d = b;
                        }
                    }
                    _ => return None,
                }
                Some(Value::Void)
            }
            Builtin::SliceReverse => {
                let (fi, l, path) = self.recv_place(&args[0], env)?;
                match walk(frame_root(&mut self.frames, env, fi, l)?, &path)? {
                    Value::List(a) | Value::Array(a) => a.reverse(),
                    Value::OwnedStr(s) | Value::Str(s) => s.reverse(),
                    _ => return None,
                }
                Some(Value::Void)
            }
            Builtin::SliceSort => {
                let (fi, l, path) = self.recv_place(&args[0], env)?;
                match walk(frame_root(&mut self.frames, env, fi, l)?, &path)? {
                    Value::List(a) | Value::Array(a) => a.sort_by(value_cmp),
                    Value::OwnedStr(s) | Value::Str(s) => s.sort(),
                    _ => return None,
                }
                Some(Value::Void)
            }
            Builtin::SliceIndexOf => {
                let items = seq_items(&vs[0])?;
                Some(Value::Opt(items.iter().position(|x| key_eq(x, &vs[1])).map(|i| Box::new(Value::Int(i as i128)))))
            }
            Builtin::SliceFind => {
                let hay = bytes_of(&vs[0])?;
                let needle = bytes_of(&vs[1])?;
                let pos = if needle.is_empty() { Some(0) } else { hay.windows(needle.len()).position(|w| w == needle) };
                Some(Value::Opt(pos.map(|i| Box::new(Value::Int(i as i128)))))
            }
            Builtin::SliceTrim => {
                let s = bytes_of(&vs[0])?;
                let t = String::from_utf8_lossy(s).trim().to_string();
                Some(Value::OwnedStr(t.into_bytes()))
            }
            Builtin::SliceSplit => {
                let s = bytes_of(&vs[0])?;
                let sep = bytes_of(&vs[1])?;
                let mut out = Vec::new();
                if sep.is_empty() {
                    out.push(Value::OwnedStr(s.to_vec()));
                } else {
                    let mut start = 0;
                    let mut i = 0;
                    while i + sep.len() <= s.len() {
                        if &s[i..i + sep.len()] == sep {
                            out.push(Value::OwnedStr(s[start..i].to_vec()));
                            i += sep.len();
                            start = i;
                        } else {
                            i += 1;
                        }
                    }
                    out.push(Value::OwnedStr(s[start..].to_vec()));
                }
                Some(Value::List(out))
            }
            Builtin::SliceLines => {
                let s = bytes_of(&vs[0])?;
                Some(Value::List(s.split(|b| *b == b'\n').map(|l| Value::OwnedStr(l.strip_suffix(b"\r").unwrap_or(l).to_vec())).collect()))
            }
            Builtin::SliceToOwned => Some(match &vs[0] {
                Value::Str(s) => Value::OwnedStr(s.clone()),
                Value::Array(a) => Value::List(a.clone()),
                other => other.clone(),
            }),
            Builtin::SliceParseInt => {
                let s = String::from_utf8_lossy(bytes_of(&vs[0])?).to_string();
                Some(match s.trim().parse::<i128>() {
                    Ok(v) => Value::Ok(Box::new(Value::Int(v))),
                    Err(_) => Value::Err(self.c.error_id("InvalidInput")),
                })
            }
            Builtin::SliceParseFloat => {
                let s = String::from_utf8_lossy(bytes_of(&vs[0])?).to_string();
                Some(match s.trim().parse::<f64>() {
                    Ok(v) => Value::Ok(Box::new(Value::Float(v))),
                    Err(_) => Value::Err(self.c.error_id("InvalidInput")),
                })
            }
            Builtin::SliceEqIgnoreCase => Some(Value::Bool(bytes_of(&vs[0])?.eq_ignore_ascii_case(bytes_of(&vs[1])?))),
            // ---- checks and misc
            Builtin::ExpectEq => {
                if key_eq(&vs[0], &vs[1]) {
                    Some(Value::Void)
                } else {
                    self.panic_at(&format!("expectation failed: {} != {}", format_value(&vs[0]), format_value(&vs[1])), e.span)
                }
            }
            Builtin::Assert => {
                if vs[0] == Value::Bool(true) {
                    Some(Value::Void)
                } else {
                    self.panic_at("assertion failed", e.span)
                }
            }
            Builtin::Unreachable => self.panic_at("reached unreachable code", e.span),
            Builtin::TypeName => Some(Value::OwnedStr(self.c.type_name(*_tys.first()?).into_bytes())),
            Builtin::Utf8Validate => Some(Value::Bool(std::str::from_utf8(bytes_of(&vs[0])?).is_ok())),
            Builtin::Utf8Encode => {
                let cp = char::from_u32(vs[0].as_int()? as u32)?;
                let mut b = [0u8; 4];
                Some(Value::OwnedStr(cp.encode_utf8(&mut b).as_bytes().to_vec()))
            }
            Builtin::Utf8Decode => {
                let s = bytes_of(&vs[0])?;
                Some(Value::Opt(std::str::from_utf8(s).ok().and_then(|t| t.chars().next()).map(|c| Box::new(Value::Char(c as u32)))))
            }
            Builtin::TimeMonotonic if self.c.repl_mode => Some(Value::Int(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as i128).unwrap_or(0))),
            Builtin::RandomSeed if self.c.repl_mode => Some(Value::Void),
            Builtin::RandomInt if self.c.repl_mode => {
                let (lo, hi) = (vs[0].as_int()?, vs[1].as_int()?);
                let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as i128).unwrap_or(0);
                Some(Value::Int(if hi > lo { lo + (t.rem_euclid(hi - lo + 1)) } else { lo }))
            }
            Builtin::RandomFloat if self.c.repl_mode => {
                let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
                Some(Value::Float(t as f64 / 1e9))
            }
            // ---- reference classes are plain values in the interpreter
            Builtin::Retain | Builtin::Release | Builtin::Drop => Some(Value::Void),
            Builtin::Weak => Some(Value::Opt(Some(Box::new(vs[0].clone())))),
            Builtin::Upgrade => Some(match &vs[0] {
                Value::Opt(v) => Value::Opt(v.clone()),
                other => Value::Opt(Some(Box::new(other.clone()))),
            }),
            Builtin::RefCount => Some(Value::Int(1)),
            Builtin::RefEq => Some(Value::Bool(key_eq(&vs[0], &vs[1]))),
            Builtin::ListNew | Builtin::ListWithCapacity => Some(Value::List(vec![])),
            Builtin::ListFromSlice => match &vs[0] {
                Value::Array(a) => Some(Value::List(a.clone())),
                Value::Str(s) => Some(Value::List(s.iter().map(|b| Value::Int(*b as i128)).collect())),
                _ => None,
            },
            Builtin::ListAppend => {
                let (fi, l, path) = self.recv_place(&args[0], env)?;
                let v = vs[1].clone();
                let cur = walk(frame_root(&mut self.frames, env, fi, l)?, &path)?;
                match cur {
                    Value::List(a) => a.push(v),
                    Value::Undefined => *cur = Value::List(vec![v]),
                    _ => return None,
                }
                Some(Value::Void)
            }
            Builtin::ListPop => {
                let (fi, l, path) = self.recv_place(&args[0], env)?;
                let cur = walk(frame_root(&mut self.frames, env, fi, l)?, &path)?;
                match cur {
                    Value::List(a) => Some(Value::Opt(a.pop().map(Box::new))),
                    _ => None,
                }
            }
            Builtin::ListClone | Builtin::StringClone | Builtin::MapClone => Some(vs[0].clone()),
            Builtin::ListLast => match (&vs[0], vs[1].as_int()?) {
                (Value::Array(a), 1) => Some(Value::Opt(a.last().cloned().map(Box::new))),
                (Value::Array(a), _) => Some(Value::Opt(a.first().cloned().map(Box::new))),
                _ => None,
            },
            Builtin::ListItems => Some(vs[0].clone()),
            Builtin::StringNew | Builtin::StringWithCapacity => Some(Value::OwnedStr(vec![])),
            Builtin::StringFrom => match &vs[0] {
                Value::Str(s) | Value::OwnedStr(s) => Some(Value::OwnedStr(s.clone())),
                _ => None,
            },
            Builtin::StringBytes => match &vs[0] {
                Value::Str(s) | Value::OwnedStr(s) => Some(Value::Str(s.clone())),
                _ => None,
            },
            Builtin::StringAppend | Builtin::StringAppendChar | Builtin::StringPushByte => {
                let (fi, l, path) = self.recv_place(&args[0], env)?;
                let add: Vec<u8> = match &vs[1] {
                    Value::Str(s) | Value::OwnedStr(s) => s.clone(),
                    Value::Char(c) => {
                        let mut b = [0u8; 4];
                        char::from_u32(*c)?.encode_utf8(&mut b).as_bytes().to_vec()
                    }
                    Value::Int(i) => vec![*i as u8],
                    _ => return None,
                };
                let cur = walk(frame_root(&mut self.frames, env, fi, l)?, &path)?;
                match cur {
                    Value::OwnedStr(s) => s.extend(add),
                    Value::Undefined => *cur = Value::OwnedStr(add),
                    _ => return None,
                }
                Some(Value::Void)
            }
            Builtin::SliceEq => Some(Value::Bool(match (&vs[0], &vs[1]) {
                (Value::Str(a) | Value::OwnedStr(a), Value::Str(b) | Value::OwnedStr(b)) => a == b,
                (Value::Array(a), Value::Array(b)) => a == b,
                _ => return None,
            })),
            Builtin::SliceStartsWith | Builtin::SliceEndsWith => match (&vs[0], &vs[1]) {
                (Value::Str(a) | Value::OwnedStr(a), Value::Str(b) | Value::OwnedStr(b)) => Some(Value::Bool(if op == Builtin::SliceStartsWith { a.starts_with(b) } else { a.ends_with(b) })),
                _ => None,
            },
            Builtin::SliceContains => match &vs[0] {
                Value::Array(a) => Some(Value::Bool(a.contains(&vs[1]))),
                Value::Str(s) => Some(Value::Bool(s.contains(&(vs[1].as_int()? as u8)))),
                _ => None,
            },
            Builtin::Expect => {
                if vs[0] == Value::Bool(true) {
                    Some(Value::Void)
                } else {
                    let msg = match &vs[1] {
                        Value::Str(s) => String::from_utf8_lossy(s).to_string(),
                        _ => String::new(),
                    };
                    self.fail_panic(format!("expectation failed: {}", msg), e.span)
                }
            }
            Builtin::Panic => {
                let msg = match &vs[0] {
                    Value::Str(s) | Value::OwnedStr(s) => String::from_utf8_lossy(s).to_string(),
                    _ => "panic".into(),
                };
                self.fail_panic(msg, e.span)
            }
            Builtin::Truncate => {
                let (bits, signed) = bits_of(self.c, e.ty)?;
                Some(Value::Int(wrap_to(vs[0].as_int()?, bits, signed)))
            }
            Builtin::ErrorName => match &vs[0] {
                Value::Err(id) => Some(Value::Str(self.c.error_names[(*id - 1) as usize].clone().into_bytes())),
                _ => None,
            },
            Builtin::MathAbs => Some(match &vs[0] {
                Value::Int(i) => Value::Int(i.abs()),
                Value::Float(f) => Value::Float(f.abs()),
                _ => return None,
            }),
            Builtin::MathMin | Builtin::MathMax => {
                let is_min = op == Builtin::MathMin;
                Some(match (&vs[0], &vs[1]) {
                    (Value::Int(a), Value::Int(b)) => Value::Int(if is_min { *a.min(b) } else { *a.max(b) }),
                    (Value::Float(a), Value::Float(b)) => Value::Float(if is_min { a.min(*b) } else { a.max(*b) }),
                    _ => return None,
                })
            }
            Builtin::MathClamp => Some(match (&vs[0], &vs[1], &vs[2]) {
                (Value::Int(a), Value::Int(lo), Value::Int(hi)) => Value::Int((*a).clamp(*lo, *hi)),
                (Value::Float(a), Value::Float(lo), Value::Float(hi)) => Value::Float(a.clamp(*lo, *hi)),
                _ => return None,
            }),
            Builtin::MathSqrt
            | Builtin::MathFloor
            | Builtin::MathCeil
            | Builtin::MathRound
            | Builtin::MathSin
            | Builtin::MathCos
            | Builtin::MathTan
            | Builtin::MathExp
            | Builtin::MathLog
            | Builtin::MathLog2 => {
                let x = vs[0].as_float()?;
                Some(Value::Float(match op {
                    Builtin::MathSqrt => x.sqrt(),
                    Builtin::MathFloor => x.floor(),
                    Builtin::MathCeil => x.ceil(),
                    Builtin::MathRound => x.round(),
                    Builtin::MathSin => x.sin(),
                    Builtin::MathCos => x.cos(),
                    Builtin::MathTan => x.tan(),
                    Builtin::MathExp => x.exp(),
                    Builtin::MathLog => x.ln(),
                    _ => x.log2(),
                }))
            }
            Builtin::MathPow => Some(Value::Float(vs[0].as_float()?.powf(vs[1].as_float()?))),
            Builtin::MathAtan2 => Some(Value::Float(vs[0].as_float()?.atan2(vs[1].as_float()?))),
            Builtin::Hash => {
                // checked_add/sub/mul encoded with a marker
                let (a, b, m) = (vs[0].as_int()?, vs[1].as_int()?, vs[2].as_int()?);
                let r = match m {
                    0 => a.checked_add(b),
                    1 => a.checked_sub(b),
                    _ => a.checked_mul(b),
                };
                let range = int_range(self.c, args[0].ty)?;
                Some(Value::Opt(r.filter(|v| *v >= range.0 && *v <= range.1).map(|v| Box::new(Value::Int(v)))))
            }
            // in a REPL the program may talk to the world; `comptime` never may
            Builtin::Println | Builtin::Print | Builtin::Eprintln if self.c.repl_mode => {
                use std::io::Write;
                let mut out = format_values(&vs)?;
                if op != Builtin::Print {
                    out.push(b'\n');
                }
                if op == Builtin::Eprintln {
                    let _ = std::io::stderr().write_all(&out);
                } else {
                    let _ = std::io::stdout().write_all(&out);
                    let _ = std::io::stdout().flush();
                }
                Some(Value::Void)
            }
            Builtin::ReadFile if self.c.repl_mode => {
                let path = String::from_utf8_lossy(bytes_of(&vs[0])?).to_string();
                Some(match std::fs::read(&path) {
                    Ok(d) => Value::Ok(Box::new(Value::OwnedStr(d))),
                    Err(_) => Value::Err(self.c.error_id("IoError")),
                })
            }
            Builtin::WriteFile if self.c.repl_mode => {
                let path = String::from_utf8_lossy(bytes_of(&vs[0])?).to_string();
                Some(match std::fs::write(&path, bytes_of(&vs[1])?) {
                    Ok(()) => Value::Ok(Box::new(Value::Void)),
                    Err(_) => Value::Err(self.c.error_id("IoError")),
                })
            }
            Builtin::AppendFile if self.c.repl_mode => {
                let path = String::from_utf8_lossy(bytes_of(&vs[0])?).to_string();
                let data = bytes_of(&vs[1])?.to_vec();
                let r = std::fs::OpenOptions::new().append(true).create(true).open(&path).and_then(|mut f| std::io::Write::write_all(&mut f, &data));
                Some(match r {
                    Ok(()) => Value::Ok(Box::new(Value::Void)),
                    Err(_) => Value::Err(self.c.error_id("IoError")),
                })
            }
            Builtin::FsKind if self.c.repl_mode => {
                let path = String::from_utf8_lossy(bytes_of(&vs[0])?).to_string();
                Some(Value::Int(match std::fs::metadata(&path) {
                    Ok(m) if m.is_dir() => 2,
                    Ok(_) => 1,
                    Err(_) => 0,
                }))
            }
            Builtin::FsSize | Builtin::FsModified if self.c.repl_mode => {
                let path = String::from_utf8_lossy(bytes_of(&vs[0])?).to_string();
                Some(match std::fs::metadata(&path) {
                    Ok(m) if op == Builtin::FsSize => Value::Ok(Box::new(Value::Int(m.len() as i128))),
                    Ok(m) => {
                        let ms = m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_millis() as i128).unwrap_or(0);
                        Value::Ok(Box::new(Value::Int(ms)))
                    }
                    Err(e) => Value::Err(self.c.error_id(if e.kind() == std::io::ErrorKind::NotFound { "NotFound" } else { "IoError" })),
                })
            }
            Builtin::FsMkdir | Builtin::FsRemoveFile | Builtin::FsRemoveDir if self.c.repl_mode => {
                let path = String::from_utf8_lossy(bytes_of(&vs[0])?).to_string();
                let r = match op {
                    Builtin::FsMkdir => std::fs::create_dir(&path).or_else(|e| if e.kind() == std::io::ErrorKind::AlreadyExists { Ok(()) } else { Err(e) }),
                    Builtin::FsRemoveFile => std::fs::remove_file(&path),
                    _ => std::fs::remove_dir(&path),
                };
                Some(match r {
                    Ok(()) => Value::Ok(Box::new(Value::Void)),
                    Err(e) => Value::Err(self.c.error_id(if e.kind() == std::io::ErrorKind::NotFound { "NotFound" } else { "IoError" })),
                })
            }
            Builtin::FsRename if self.c.repl_mode => {
                let a = String::from_utf8_lossy(bytes_of(&vs[0])?).to_string();
                let b = String::from_utf8_lossy(bytes_of(&vs[1])?).to_string();
                Some(match std::fs::rename(&a, &b) {
                    Ok(()) => Value::Ok(Box::new(Value::Void)),
                    Err(e) => Value::Err(self.c.error_id(if e.kind() == std::io::ErrorKind::NotFound { "NotFound" } else { "IoError" })),
                })
            }
            Builtin::FsListDir if self.c.repl_mode => {
                let path = String::from_utf8_lossy(bytes_of(&vs[0])?).to_string();
                Some(match std::fs::read_dir(&path) {
                    Ok(rd) => Value::Ok(Box::new(Value::List(rd.filter_map(|e| e.ok()).map(|e| Value::OwnedStr(e.file_name().to_string_lossy().into_owned().into_bytes())).collect()))),
                    Err(e) => Value::Err(self.c.error_id(if e.kind() == std::io::ErrorKind::NotFound { "NotFound" } else { "IoError" })),
                })
            }
            Builtin::Environ if self.c.repl_mode => Some(Value::List(std::env::vars().map(|(k, v)| Value::OwnedStr(format!("{}={}", k, v).into_bytes())).collect())),
            Builtin::FsCwd if self.c.repl_mode => Some(match std::env::current_dir() {
                Ok(d) => Value::Ok(Box::new(Value::OwnedStr(d.to_string_lossy().into_owned().into_bytes()))),
                Err(_) => Value::Err(self.c.error_id("IoError")),
            }),
            Builtin::FsTempDir if self.c.repl_mode => {
                let d = std::env::temp_dir();
                let s = d.to_string_lossy().trim_end_matches(['/', '\\']).to_string();
                Some(Value::OwnedStr(s.into_bytes()))
            }
            Builtin::ReadLine if self.c.repl_mode => {
                let mut line = String::new();
                Some(match std::io::stdin().read_line(&mut line) {
                    Ok(n) if n > 0 => Value::Opt(Some(Box::new(Value::OwnedStr(line.trim_end_matches(['\n', '\r']).as_bytes().to_vec())))),
                    _ => Value::Opt(None),
                })
            }
            Builtin::Args if self.c.repl_mode => Some(Value::List(vec![Value::Str(b"nx".to_vec())])),
            Builtin::Env if self.c.repl_mode => {
                let name = String::from_utf8_lossy(bytes_of(&vs[0])?).to_string();
                Some(Value::Opt(std::env::var(&name).ok().map(|v| Box::new(Value::OwnedStr(v.into_bytes())))))
            }
            Builtin::TimeNow if self.c.repl_mode => Some(Value::Int(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i128).unwrap_or(0))),
            // the interpreter has no time zone database: local time is UTC at the prompt
            Builtin::TimeUtcOffset if self.c.repl_mode => Some(Value::Int(0)),
            Builtin::Sleep if self.c.repl_mode => {
                std::thread::sleep(std::time::Duration::from_millis(vs[0].as_int()?.max(0) as u64));
                Some(Value::Void)
            }
            Builtin::Exit if self.c.repl_mode => std::process::exit(vs[0].as_int()? as i32),
            Builtin::Format => {
                let fmt = match &vs[0] {
                    Value::Str(s) => s.clone(),
                    _ => return None,
                };
                let mut out = Vec::new();
                let mut ai = 1;
                let mut i = 0;
                while i < fmt.len() {
                    if fmt[i] == b'{' {
                        if i + 1 < fmt.len() && fmt[i + 1] == b'{' {
                            out.push(b'{');
                            i += 2;
                            continue;
                        }
                        let mut j = i + 1;
                        while j < fmt.len() && fmt[j] != b'}' {
                            j += 1;
                        }
                        let v = vs.get(ai)?;
                        ai += 1;
                        out.extend(format_value(v).into_bytes());
                        i = j + 1;
                    } else if fmt[i] == b'}' && i + 1 < fmt.len() && fmt[i + 1] == b'}' {
                        out.push(b'}');
                        i += 2;
                    } else {
                        out.push(fmt[i]);
                        i += 1;
                    }
                }
                Some(Value::OwnedStr(out))
            }
            Builtin::IntToStr | Builtin::FloatToStr => Some(Value::OwnedStr(format_value(&vs[0]).into_bytes())),
            Builtin::CharIsDigit => Some(Value::Bool(char::from_u32(vs[0].as_int()? as u32)?.is_ascii_digit())),
            Builtin::CharIsAlpha => Some(Value::Bool(char::from_u32(vs[0].as_int()? as u32)?.is_alphabetic())),
            Builtin::CharIsSpace => Some(Value::Bool(char::from_u32(vs[0].as_int()? as u32)?.is_whitespace())),
            Builtin::CharToLower => Some(Value::Char(char::from_u32(vs[0].as_int()? as u32)?.to_lowercase().next()? as u32)),
            Builtin::CharToUpper => Some(Value::Char(char::from_u32(vs[0].as_int()? as u32)?.to_uppercase().next()? as u32)),
            Builtin::CharToDigit => Some(Value::Opt(char::from_u32(vs[0].as_int()? as u32)?.to_digit(10).map(|d| Box::new(Value::Int(d as i128))))),
            _ => None,
        }
    }
}

/// Follow a place path (field and element indices) into a value.
fn walk<'v>(root: &'v mut Value, path: &[usize]) -> Option<&'v mut Value> {
    let mut cur = root;
    for &s in path {
        cur = match cur {
            Value::Array(a) | Value::List(a) | Value::Struct(a) | Value::Tuple(a) => a.get_mut(s)?,
            Value::Enum(_, a) => a.get_mut(s)?,
            Value::Opt(Some(b)) | Value::Ok(b) if s == 0 => b.as_mut(),
            _ => return None,
        };
    }
    Some(cur)
}

/// Equality that treats a string literal and an owned string alike.
fn key_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Str(x) | Value::OwnedStr(x), Value::Str(y) | Value::OwnedStr(y)) => x == y,
        (Value::List(x) | Value::Array(x), Value::List(y) | Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| key_eq(p, q)),
        _ => a == b,
    }
}

fn value_cmp(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x.cmp(y),
        (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(Equal),
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        (Value::Char(x), Value::Char(y)) => x.cmp(y),
        (Value::Str(x) | Value::OwnedStr(x), Value::Str(y) | Value::OwnedStr(y)) => x.cmp(y),
        (Value::Struct(x), Value::Struct(y)) | (Value::Tuple(x), Value::Tuple(y)) => x.iter().zip(y).map(|(p, q)| value_cmp(p, q)).find(|o| *o != Equal).unwrap_or(Equal),
        _ => Equal,
    }
}

/// The elements of an array, list, or byte string as values.
fn seq_items(v: &Value) -> Option<Vec<Value>> {
    match v {
        Value::Array(a) | Value::List(a) => Some(a.clone()),
        Value::Str(s) | Value::OwnedStr(s) => Some(s.iter().map(|b| Value::Int(*b as i128)).collect()),
        _ => None,
    }
}

fn bytes_of(v: &Value) -> Option<&[u8]> {
    match v {
        Value::Str(s) | Value::OwnedStr(s) => Some(s.as_slice()),
        _ => None,
    }
}

/// `format("...", .{args})` over evaluated values: the placeholder loop.
fn format_values(vs: &[Value]) -> Option<Vec<u8>> {
    let fmt = bytes_of(vs.first()?)?;
    let mut out = Vec::new();
    let mut ai = 1;
    let mut i = 0;
    while i < fmt.len() {
        if fmt[i] == b'{' {
            if i + 1 < fmt.len() && fmt[i + 1] == b'{' {
                out.push(b'{');
                i += 2;
                continue;
            }
            let mut j = i + 1;
            while j < fmt.len() && fmt[j] != b'}' {
                j += 1;
            }
            let v = vs.get(ai)?;
            ai += 1;
            out.extend(format_value(v).into_bytes());
            i = j + 1;
        } else if fmt[i] == b'}' && i + 1 < fmt.len() && fmt[i + 1] == b'}' {
            out.push(b'}');
            i += 2;
        } else {
            out.push(fmt[i]);
            i += 1;
        }
    }
    Some(out)
}

/// `nx repl`: run `main`'s statements from `start`, with earlier bindings seeded
/// by name, and report the bindings afterwards and the value of a trailing
/// expression statement.
pub fn run_repl(c: &mut Checker, start: usize, seed: Vec<(String, Value)>) -> crate::tir::ReplOutcome {
    let mut out = crate::tir::ReplOutcome::default();
    let inst = match c.main {
        Some(i) => i,
        None => return out,
    };
    let mut it = Interp { c, steps: 0, panic: None, budget_exceeded: false, pending: None, failed_at: None, frames: vec![], cur_fn: None, temps: 0 };
    if !it.ensure_body(inst) {
        return out;
    }
    let f = it.c.funcs[inst as usize].clone();
    it.cur_fn = Some(inst);
    let body = match &f.body {
        Some(b) => b.clone(),
        None => return out,
    };
    // the local each earlier name refers to at `start`
    let mut by_name: HashMap<String, LocalId> = HashMap::new();
    for s in body.stmts.iter().take(start) {
        if let TStmt::Let { local, .. } = s {
            by_name.insert(f.locals[*local as usize].name.clone(), *local);
        }
    }
    let mut env = Env::new();
    for (name, v) in seed {
        if let Some(l) = by_name.get(&name) {
            env.insert(*l, v);
        }
    }
    let mut last: Option<(TyId, Value)> = None;
    for s in body.stmts.iter().skip(start) {
        let r = match s {
            TStmt::Expr(e) => it.eval(e, &mut env).map(|v| {
                last = Some((e.ty, v));
                Flow::Next
            }),
            // the synthetic main ends with a bare `return`: the session is done
            TStmt::Return { value: None, .. } => break,
            other => {
                last = None;
                it.exec_stmt(other, &mut env)
            }
        };
        if let Some((msg, _)) = it.panic.take() {
            out.panic = Some(msg);
            return out;
        }
        if it.budget_exceeded {
            out.panic = Some(format!("exceeded the step budget ({} steps)", STEP_BUDGET));
            return out;
        }
        match r {
            Some(Flow::Next) => {}
            Some(Flow::Return(_)) => break,
            Some(_) => break,
            None => {
                let at = match it.failed_at {
                    Some(sp) => match it.c.sm.line_col(sp) {
                        Some((line, col)) => format!(" (at {}:{}:{})", it.c.sm.file(sp.file).map(|f| f.name.as_str()).unwrap_or("?"), line, col),
                        None => String::new(),
                    },
                    None => String::new(),
                };
                out.panic = Some(format!("this statement cannot be evaluated by the interpreter{}; foreign calls, parallel loops, and arenas need a compiled program", at));
                return out;
            }
        }
    }
    // bindings afterwards, later declarations winning
    let mut seen: HashMap<String, usize> = HashMap::new();
    for s in &body.stmts {
        if let TStmt::Let { local, .. } = s {
            if let Some(v) = env.get(local) {
                let name = f.locals[*local as usize].name.clone();
                match seen.get(&name) {
                    Some(&idx) => out.bindings[idx] = (name, v.clone()),
                    None => {
                        seen.insert(name.clone(), out.bindings.len());
                        out.bindings.push((name, v.clone()));
                    }
                }
            }
        }
    }
    for (name, v) in &out.bindings {
        let text = match by_name_all(&body, &f, name) {
            Some(l) => {
                let lt = it.c.tys.resolve(f.locals[l as usize].ty, true);
                format!("{}: {}", it.c.type_name(lt), format_value_typed(it.c, v, lt))
            }
            None => format_value(v),
        };
        out.shown.push((name.clone(), text));
    }
    if let Some((ty, v)) = last {
        let ty = it.c.tys.resolve(ty, true);
        let t = it.c.tys.shallow(ty);
        if !matches!(it.c.tys.kind(t), TyKind::Void | TyKind::Never) {
            let tn = it.c.type_name(ty);
            let text = format_value_typed(it.c, &v, ty);
            out.printed = Some((tn, text));
        }
    }
    out
}

/// The last local declared under `name` anywhere in the body.
fn by_name_all(body: &TBlock, f: &TFunc, name: &str) -> Option<LocalId> {
    let mut found = None;
    for s in &body.stmts {
        if let TStmt::Let { local, .. } = s {
            if f.locals[*local as usize].name == name {
                found = Some(*local);
            }
        }
    }
    found
}

/// Render a value the way source code would write it, using its type.
pub fn format_value_typed(c: &Checker, v: &Value, ty: TyId) -> String {
    let t = c.tys.shallow(ty);
    match (c.tys.kind(t).clone(), v) {
        (TyKind::Str, Value::Str(s)) | (TyKind::Str, Value::OwnedStr(s)) => format!("{:?}", String::from_utf8_lossy(s)),
        (TyKind::Slice(_, e), Value::Str(s)) | (TyKind::Slice(_, e), Value::OwnedStr(s)) if matches!(c.tys.kind(c.tys.shallow(e)), TyKind::Int(crate::types::IntTy::U8)) => {
            format!("{:?}", String::from_utf8_lossy(s))
        }
        (TyKind::Char, Value::Char(ch)) => format!("{:?}", char::from_u32(*ch).unwrap_or('?')),
        (TyKind::List(e), Value::List(xs))
        | (TyKind::Slice(_, e), Value::List(xs))
        | (TyKind::Array(_, e), Value::Array(xs))
        | (TyKind::Slice(_, e), Value::Array(xs))
        | (TyKind::List(e), Value::Array(xs)) => {
            format!("[{}]", xs.iter().map(|x| format_value_typed(c, x, e)).collect::<Vec<_>>().join(", "))
        }
        (TyKind::Map(k, v), Value::Map(kv)) => format!("{{{}}}", kv.iter().map(|(a, b)| format!("{}: {}", format_value_typed(c, a, k), format_value_typed(c, b, v))).collect::<Vec<_>>().join(", ")),
        (TyKind::Tuple(ts), Value::Tuple(xs)) => format!("({})", xs.iter().zip(ts.iter()).map(|(x, &t)| format_value_typed(c, x, t)).collect::<Vec<_>>().join(", ")),
        (TyKind::Opt(_), Value::Opt(None)) => "null".into(),
        (TyKind::Opt(e), Value::Opt(Some(b))) => format_value_typed(c, b, e),
        (TyKind::ErrUnion(_, e), Value::Ok(b)) => format_value_typed(c, b, e),
        (TyKind::ErrUnion(..), Value::Err(id)) => format!("error.{}", c.error_names.get((*id as usize).saturating_sub(1)).cloned().unwrap_or_else(|| "?".into())),
        (TyKind::Struct(d, _), Value::Struct(xs)) => {
            let def = &c.structs[d as usize];
            let ftys = c.struct_field_tys.get(&t).cloned().unwrap_or_default();
            let fields: Vec<String> = def
                .fields
                .iter()
                .zip(xs.iter())
                .enumerate()
                .map(|(i, (f, x))| format!(".{} = {}", f.name, ftys.get(i).map(|&ft| format_value_typed(c, x, ft)).unwrap_or_else(|| format_value(x))))
                .collect();
            format!("{}{{ {} }}", def.name, fields.join(", "))
        }
        (TyKind::Enum(d, _), Value::Enum(vi, xs)) => {
            let def = &c.enums[d as usize];
            let name = def.variants.get(*vi as usize).map(|v| v.name.clone()).unwrap_or_else(|| vi.to_string());
            if xs.is_empty() {
                format!(".{}", name)
            } else {
                let vtys = c.enum_variant_tys.get(&t).and_then(|v| v.get(*vi as usize).cloned()).unwrap_or_default();
                let args: Vec<String> = xs.iter().enumerate().map(|(i, x)| vtys.get(i).map(|&at| format_value_typed(c, x, at)).unwrap_or_else(|| format_value(x))).collect();
                format!(".{}({})", name, args.join(", "))
            }
        }
        (_, Value::Fn(_)) => "<function>".into(),
        (_, Value::Ptr(..)) => "<pointer>".into(),
        (_, Value::Type(t)) => c.type_name(*t),
        _ => format_value(v),
    }
}

pub fn format_value(v: &Value) -> String {
    match v {
        Value::Int(i) => i.to_string(),
        Value::Float(f) => format_float(*f),
        Value::Bool(b) => b.to_string(),
        Value::Char(c) => char::from_u32(*c).map(|c| c.to_string()).unwrap_or_default(),
        Value::Str(s) | Value::OwnedStr(s) => String::from_utf8_lossy(s).to_string(),
        Value::Void => String::new(),
        Value::Opt(None) => "null".into(),
        Value::Opt(Some(v)) => format_value(v),
        Value::Enum(i, _) => format!("variant {}", i),
        _ => "?".into(),
    }
}

pub fn format_float(f: f64) -> String {
    if f.is_finite() && f == f.trunc() && f.abs() < 1e16 {
        format!("{:.1}", f)
    } else {
        format!("{}", f)
    }
}
