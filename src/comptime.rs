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
}

/// Evaluate an expression with no locals. Reports compile-time panics as errors.
pub fn eval_const_expr(c: &mut Checker, e: &TExpr) -> Option<Value> {
    let mut it = Interp { c, steps: 0, panic: None, budget_exceeded: false, pending: None };
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
    let mut it = Interp { c, steps: 0, panic: None, budget_exceeded: false, pending: None };
    let mut env = Env::new();
    env.insert(local, v);
    it.eval(e, &mut env)
}

/// Run a function instance at compile time with the given arguments.
pub fn call_instance(c: &mut Checker, inst: InstId, args: Vec<Value>) -> Result<Option<Value>, (String, Span)> {
    let mut it = Interp { c, steps: 0, panic: None, budget_exceeded: false, pending: None };
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
        let (flow, v) = self.exec_block(body, &mut env)?;
        Some(match flow {
            Flow::Return(v) => v,
            _ => v.unwrap_or(Value::Void),
        })
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
            TStmt::While { cond, body, label, .. } => {
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
                        Flow::Break(l, _) if l == *label => break,
                        Flow::Continue(l) if l == *label => continue,
                        other => return Some(other),
                    }
                }
                Some(Flow::Next)
            }
            TStmt::ForRange { var, start, end, body, label, .. } => {
                let s = self.eval(start, env)?.as_int()?;
                let e = self.eval(end, env)?.as_int()?;
                let mut i = s;
                while i < e {
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
                    i += 1;
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

    /// Resolve a place expression to a root local and a path of indices.
    fn place_path(&mut self, e: &TExpr, env: &mut Env) -> Option<(LocalId, Vec<usize>)> {
        match &e.kind {
            TExprKind::Local(l) => Some((*l, vec![])),
            TExprKind::Field { base, idx } | TExprKind::TupleField { base, idx } | TExprKind::RefField { base, idx } => {
                let (l, mut p) = self.place_path(base, env)?;
                p.push(*idx as usize);
                Some((l, p))
            }
            TExprKind::Index { base, index, .. } => {
                let i = self.eval(index, env)?.as_int()?;
                let (l, mut p) = self.place_path(base, env)?;
                p.push(i as usize);
                Some((l, p))
            }
            TExprKind::Deref(p) => match self.eval(p, env)? {
                Value::Ptr(l, path) => Some((l, path)),
                _ => None,
            },
            TExprKind::ListToSlice(inner) | TExprKind::ArrayToSlice(inner) | TExprKind::StrToSlice(inner) => self.place_path(inner, env),
            _ => None,
        }
    }

    fn assign(&mut self, target: &TExpr, v: Value, env: &mut Env) -> Option<()> {
        let (l, path) = self.place_path(target, env)?;
        let root = env.get_mut(&l)?;
        let mut cur = root;
        for (k, &step) in path.iter().enumerate() {
            let last = k == path.len() - 1;
            let next: &mut Value = match cur {
                Value::Array(a) | Value::List(a) | Value::Tuple(a) | Value::Struct(a) => a.get_mut(step)?,
                Value::Enum(_, a) => a.get_mut(step)?,
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
                Value::Ptr(l, path) => {
                    let root = env.get(&l)?.clone();
                    self.read_path(&root, &path)
                }
                _ => None,
            },
            TExprKind::AddrOf { expr, .. } => {
                let (l, path) = self.place_path(expr, env)?;
                Some(Value::Ptr(l, path))
            }
            TExprKind::Call { inst, args } => {
                let mut vs = Vec::new();
                for a in args {
                    vs.push(self.eval(a, env)?);
                }
                self.call(*inst, vs)
            }
            TExprKind::CallPtr { callee, args } => {
                let f = self.eval(callee, env)?;
                let mut vs = Vec::new();
                for a in args {
                    vs.push(self.eval(a, env)?);
                }
                match f {
                    Value::Fn(i) => self.call(i, vs),
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
                for arm in arms {
                    if self.match_pat(&arm.pat, &v, env)? {
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

    fn match_pat(&mut self, p: &TPat, v: &Value, env: &mut Env) -> Option<bool> {
        Some(match p {
            TPat::Wild => true,
            TPat::Bind(l) => {
                env.insert(*l, v.clone());
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
                    for (a, pv) in args.iter().zip(payload.iter()) {
                        if !self.match_pat(a, pv, env)? {
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
                Value::Opt(Some(x)) => self.match_pat(inner, x, env)?,
                _ => false,
            },
            TPat::Ok(inner) => match v {
                Value::Ok(x) => self.match_pat(inner, x, env)?,
                Value::Err(_) => false,
                other => self.match_pat(inner, other, env)?,
            },
            TPat::Or(alts) => {
                for a in alts {
                    if self.match_pat(a, v, env)? {
                        return Some(true);
                    }
                }
                false
            }
            TPat::Tuple(ps) => match v {
                Value::Tuple(vs) if vs.len() == ps.len() => {
                    for (a, pv) in ps.iter().zip(vs.iter()) {
                        if !self.match_pat(a, pv, env)? {
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
                TBinSize::Expr(e) => self.eval(e, env)?.as_int()? as usize * 8,
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
            Builtin::Len => Some(Value::Int(seq_len(&vs[0])? as i128)),
            Builtin::ListNew | Builtin::ListWithCapacity => Some(Value::List(vec![])),
            Builtin::ListFromSlice => match &vs[0] {
                Value::Array(a) => Some(Value::List(a.clone())),
                Value::Str(s) => Some(Value::List(s.iter().map(|b| Value::Int(*b as i128)).collect())),
                _ => None,
            },
            Builtin::ListAppend => {
                let (l, path) = self.place_path(&args[0], env)?;
                let v = vs[1].clone();
                let root = env.get_mut(&l)?;
                let mut cur = root;
                for &s in &path {
                    cur = match cur {
                        Value::Array(a) | Value::List(a) | Value::Struct(a) | Value::Tuple(a) => a.get_mut(s)?,
                        _ => return None,
                    };
                }
                match cur {
                    Value::List(a) => a.push(v),
                    Value::Undefined => *cur = Value::List(vec![v]),
                    _ => return None,
                }
                Some(Value::Void)
            }
            Builtin::ListPop => {
                let (l, path) = self.place_path(&args[0], env)?;
                let root = env.get_mut(&l)?;
                let mut cur = root;
                for &s in &path {
                    cur = match cur {
                        Value::Array(a) | Value::List(a) | Value::Struct(a) | Value::Tuple(a) => a.get_mut(s)?,
                        _ => return None,
                    };
                }
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
            Builtin::StringAppend | Builtin::StringAppendChar => {
                let (l, path) = self.place_path(&args[0], env)?;
                let add: Vec<u8> = match &vs[1] {
                    Value::Str(s) | Value::OwnedStr(s) => s.clone(),
                    Value::Char(c) => {
                        let mut b = [0u8; 4];
                        char::from_u32(*c)?.encode_utf8(&mut b).as_bytes().to_vec()
                    }
                    _ => return None,
                };
                let root = env.get_mut(&l)?;
                let mut cur = root;
                for &s in &path {
                    cur = match cur {
                        Value::Array(a) | Value::List(a) | Value::Struct(a) | Value::Tuple(a) => a.get_mut(s)?,
                        _ => return None,
                    };
                }
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
