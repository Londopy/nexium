//! Expression checking.

use super::*;
use crate::ast::*;
use crate::tir::*;
use crate::types::*;

impl<'a> Checker<'a> {
    // ----- type predicates ----------------------------------------------------

    /// Does a value of this type own resources that must be released at scope exit?
    pub fn needs_drop(&mut self, t: TyId) -> bool {
        let t = self.tys.shallow(t);
        match self.tys.kind(t).clone() {
            TyKind::List(_) | TyKind::Str | TyKind::Map(..) | TyKind::Weak(_) => true,
            TyKind::Struct(d, _) => {
                if self.structs[d as usize].kind == StructKind::RefClass {
                    return true;
                }
                let f = self.struct_field_types(t);
                f.iter().any(|&e| self.needs_drop(e))
            }
            TyKind::Enum(..) => {
                let v = self.enum_variant_types(t);
                v.iter().flatten().any(|&e| self.needs_drop(e))
            }
            TyKind::Array(_, e) | TyKind::Opt(e) | TyKind::ErrUnion(_, e) => self.needs_drop(e),
            TyKind::Tuple(ts) => ts.iter().any(|&e| self.needs_drop(e)),
            _ => false,
        }
    }

    /// Types whose copies are made by retaining: ref classes, optionals of them, weak refs.
    pub fn is_ref_like(&self, t: TyId) -> bool {
        let t = self.tys.shallow(t);
        match self.tys.kind(t).clone() {
            TyKind::Opt(e) | TyKind::Weak(e) => self.is_ref_like(e),
            _ => self.is_ref_class(t),
        }
    }

    pub fn is_place(&self, e: &TExpr) -> bool {
        match &e.kind {
            TExprKind::Local(_) | TExprKind::Global(_) => true,
            TExprKind::Field { base, .. } | TExprKind::TupleField { base, .. } => self.is_place(base),
            TExprKind::RefField { .. } => true,
            TExprKind::Index { base, .. } => {
                let bt = self.tys.shallow(base.ty);
                match self.tys.kind(bt) {
                    TyKind::Slice(..) => true,
                    _ => self.is_place(base),
                }
            }
            TExprKind::Deref(_) => true,
            _ => false,
        }
    }

    /// Can this place be assigned through?
    pub fn place_mutable(&self, e: &TExpr) -> bool {
        match &e.kind {
            TExprKind::Local(l) => self.cur.as_ref().unwrap().locals[*l as usize].mutable,
            TExprKind::Global(_) => true,
            TExprKind::Field { base, .. } | TExprKind::TupleField { base, .. } => self.place_mutable(base),
            TExprKind::RefField { .. } => true,
            TExprKind::Index { base, .. } => {
                let bt = self.tys.shallow(base.ty);
                match self.tys.kind(bt) {
                    TyKind::Slice(m, _) => *m,
                    _ => self.place_mutable(base),
                }
            }
            TExprKind::Deref(p) => {
                let pt = self.tys.shallow(p.ty);
                matches!(self.tys.kind(pt), TyKind::Ptr(true, _))
            }
            _ => false,
        }
    }

    /// Consume a value into a new owner: locals of resource types are moved,
    /// fields cannot be moved out of, parameters are borrowed.
    /// The moved-set at this point, for branch-aware move tracking.
    pub fn moved_snapshot(&mut self) -> (HashSet<LocalId>, HashMap<LocalId, Span>) {
        let c = self.cur();
        (c.moved.clone(), c.moved_spans.clone())
    }
    pub fn moved_restore(&mut self, s: &(HashSet<LocalId>, HashMap<LocalId, Span>)) {
        let c = self.cur();
        c.moved = s.0.clone();
        c.moved_spans = s.1.clone();
    }
    /// After a branch: anything moved in it counts as moved from here on.
    pub fn moved_merge(&mut self, s: &(HashSet<LocalId>, HashMap<LocalId, Span>)) {
        let c = self.cur();
        for l in &s.0 {
            c.moved.insert(*l);
            if let Some(sp) = s.1.get(l) {
                c.moved_spans.entry(*l).or_insert(*sp);
            }
        }
    }

    pub fn take_ownership(&mut self, e: TExpr) -> TExpr {
        let t = self.tys.resolve(e.ty, false);
        if !self.needs_drop(t) {
            return e;
        }
        if self.is_ref_like(t) {
            // references (and optionals/weak refs of them) are copied by retaining
            if !matches!(e.kind, TExprKind::RefNew { .. } | TExprKind::Call { .. } | TExprKind::Retained(_)) {
                self.add_effect(Effects::REFCOUNTS, e.span, "copying a reference retains it");
                let ty = e.ty;
                let span = e.span;
                return TExpr { kind: TExprKind::Retained(Box::new(e)), ty, span };
            }
            return e;
        }
        // `return local` into a `?T` or `!T` wraps the local; the move is the local's
        if let TExprKind::OptWrap(inner) | TExprKind::ErrWrap(inner) = &e.kind {
            if matches!(inner.kind, TExprKind::Local(_)) {
                let inner = (**inner).clone();
                let inner = self.take_ownership(inner);
                let (ty, span) = (e.ty, e.span);
                return match e.kind {
                    TExprKind::OptWrap(_) => TExpr { kind: TExprKind::OptWrap(Box::new(inner)), ty, span },
                    _ => TExpr { kind: TExprKind::ErrWrap(Box::new(inner)), ty, span },
                };
            }
        }
        match &e.kind {
            TExprKind::Local(l) => {
                let l = *l;
                let cur = self.cur.as_ref().unwrap();
                let local = &cur.locals[l as usize];
                if local.is_param && !local.owned {
                    let n = local.name.clone();
                    let tn = self.type_name(t);
                    self.error_note(e.span, format!("cannot move `{}` out of a parameter; parameters are borrowed", n), None, format!("use `{}.clone()` to take an owned copy of the `{}`", n, tn));
                } else if local.loop_item {
                    let n = local.name.clone();
                    let tn = self.type_name(t);
                    self.error_note(
                        e.span,
                        format!("cannot move `{}` out of a loop; a loop variable is a view of the element", n),
                        None,
                        format!("use `{}.clone()` to take an owned copy of the `{}`", n, tn),
                    );
                } else {
                    let cur = self.cur.as_mut().unwrap();
                    cur.moved.insert(l);
                    cur.moved_spans.insert(l, e.span);
                }
                e
            }
            // `opt.?`, `opt orelse d`, `try res` on a local move the whole local
            TExprKind::Unwrap { expr: inner, .. } | TExprKind::OrElse { expr: inner, .. } | TExprKind::Try(inner) if matches!(inner.kind, TExprKind::Local(_)) => {
                let l = match inner.kind {
                    TExprKind::Local(l) => l,
                    _ => unreachable!(),
                };
                let cur = self.cur.as_ref().unwrap();
                let local = &cur.locals[l as usize];
                if local.is_param && !local.owned {
                    let n = local.name.clone();
                    let tn = self.type_name(t);
                    self.error_note(e.span, format!("cannot move the `{}` out of the parameter `{}`; parameters are borrowed", tn, n), None, "use `.clone()` to take an owned copy");
                } else if local.loop_item {
                    let n = local.name.clone();
                    let tn = self.type_name(t);
                    self.error_note(e.span, format!("cannot move the `{}` out of the loop variable `{}`; it is a view of the element", tn, n), None, "use `.clone()` to take an owned copy");
                } else {
                    let cur = self.cur.as_mut().unwrap();
                    cur.moved.insert(l);
                    cur.moved_spans.insert(l, e.span);
                }
                e
            }
            _ if is_borrowed_view(&e) => {
                let tn = self.type_name(t);
                self.error_note(e.span, format!("cannot move a `{}` out of a field or element", tn), None, "use `.clone()` for an owned copy");
                e
            }
            _ => e,
        }
    }

    // ----- coercion ---------------------------------------------------------------

    /// Try to coerce `te` to `target`. Returns Err(original) when impossible.
    pub fn coerce(&mut self, te: TExpr, target: TyId) -> Result<TExpr, TExpr> {
        let from = self.tys.shallow(te.ty);
        let to = self.tys.shallow(target);
        if from == to {
            return Ok(te);
        }
        let kf = self.tys.kind(from).clone();
        let kt = self.tys.kind(to).clone();
        let ptr_to_owned = match &kf {
            TyKind::Ptr(_, i) => matches!(self.tys.kind(self.tys.shallow(*i)), TyKind::Str | TyKind::List(_)),
            _ => false,
        };
        // never coerces to anything
        if matches!(kf, TyKind::Never) {
            return Ok(TExpr { ty: target, ..te });
        }
        match (&kf, &kt) {
            // literal / inference variables
            (TyKind::Infer(_), _) | (_, TyKind::Infer(_)) => {
                if self.unify(from, to) {
                    return Ok(TExpr { ty: target, ..te });
                }
                // a literal into `?T`: pin it to T, then wrap
                if let TyKind::Opt(inner) = kt {
                    if let Ok(inner_e) = self.coerce(te.clone(), inner) {
                        let span = inner_e.span;
                        return Ok(TExpr { kind: TExprKind::OptWrap(Box::new(inner_e)), ty: target, span });
                    }
                }
                // a literal into `!T`: the same, on the success side
                if let TyKind::ErrUnion(_, inner) = kt {
                    if let Ok(inner_e) = self.coerce(te.clone(), inner) {
                        let span = inner_e.span;
                        return Ok(TExpr { kind: TExprKind::ErrWrap(Box::new(inner_e)), ty: target, span });
                    }
                }
                return Err(te);
            }
            // *String / *List(T) -> []u8 / []T: a pointer to an owning value reads as a view of it
            (TyKind::Ptr(_, inner), TyKind::Slice(..)) if ptr_to_owned => {
                let inner = *inner;
                let span = te.span;
                let d = TExpr { kind: TExprKind::Deref(Box::new(te.clone())), ty: inner, span };
                if let Ok(v) = self.coerce(d, target) {
                    return Ok(v);
                }
                return Err(te);
            }
            // ?_ -> ?T: a `null` (or wrapped literal) whose inner type is still open
            (TyKind::Opt(a), TyKind::Opt(b)) => {
                let (a, b) = (*a, *b);
                if self.unify(a, b) {
                    return Ok(TExpr { ty: target, ..te });
                }
                return Err(te);
            }
            // T -> ?T
            (_, TyKind::Opt(inner)) => {
                let inner = *inner;
                if let Ok(inner_e) = self.coerce(te.clone(), inner) {
                    let span = inner_e.span;
                    return Ok(TExpr { kind: TExprKind::OptWrap(Box::new(inner_e)), ty: target, span });
                }
                return Err(te);
            }
            // error value -> E!T
            (TyKind::ErrorSet(_), TyKind::ErrUnion(set, _)) => {
                if let TExprKind::ErrVal(id) = te.kind {
                    if let Some(s) = set {
                        if !self.error_sets[*s as usize].ids.contains(&id) {
                            let ename = self.error_names[(id - 1) as usize].clone();
                            let sname = self.error_sets[*s as usize].name.clone();
                            self.error(te.span, format!("error `{}` is not a member of error set `{}`", ename, sname));
                        }
                    }
                }
                if matches!(te.kind, TExprKind::ErrVal(_)) {
                    return Ok(TExpr { ty: target, ..te });
                }
                let span = te.span;
                return Ok(TExpr { kind: TExprKind::ErrToUnion(Box::new(te)), ty: target, span });
            }
            // T -> !T
            (_, TyKind::ErrUnion(_, inner)) => {
                let inner = *inner;
                if let Ok(inner_e) = self.coerce(te.clone(), inner) {
                    let span = inner_e.span;
                    return Ok(TExpr { kind: TExprKind::ErrWrap(Box::new(inner_e)), ty: target, span });
                }
                return Err(te);
            }
            // [N]T -> []T   (place) ; *[N]T -> []T
            (TyKind::Array(_, e1), TyKind::Slice(m, e2)) => {
                if self.unify(*e1, *e2) {
                    if *m && !self.place_mutable(&te) && self.is_place(&te) {
                        return Err(te);
                    }
                    let span = te.span;
                    return Ok(TExpr { kind: TExprKind::ArrayToSlice(Box::new(te)), ty: target, span });
                }
                return Err(te);
            }
            (TyKind::Ptr(pm, inner), TyKind::Slice(m, e2)) => {
                if let TyKind::Array(_, e1) = self.tys.kind(self.tys.shallow(*inner)).clone() {
                    if self.unify(e1, *e2) && (!*m || *pm) {
                        let span = te.span;
                        let deref = TExpr { kind: TExprKind::Deref(Box::new(te)), ty: *inner, span };
                        return Ok(TExpr { kind: TExprKind::ArrayToSlice(Box::new(deref)), ty: target, span });
                    }
                }
                return Err(te);
            }
            // List(T) -> []T, []mut T
            (TyKind::List(e1), TyKind::Slice(m, e2)) => {
                if self.unify(*e1, *e2) {
                    if *m && self.is_place(&te) && !self.place_mutable(&te) {
                        return Err(te);
                    }
                    let span = te.span;
                    return Ok(TExpr { kind: TExprKind::ListToSlice(Box::new(te)), ty: target, span });
                }
                return Err(te);
            }
            // String -> []u8
            (TyKind::Str, TyKind::Slice(m, e2)) => {
                if matches!(self.tys.kind(self.tys.shallow(*e2)), TyKind::Int(IntTy::U8)) {
                    if *m && self.is_place(&te) && !self.place_mutable(&te) {
                        return Err(te);
                    }
                    let span = te.span;
                    return Ok(TExpr { kind: TExprKind::StrToSlice(Box::new(te)), ty: target, span });
                }
                return Err(te);
            }
            // []mut T -> []T
            (TyKind::Slice(true, e1), TyKind::Slice(false, e2)) => {
                if self.unify(*e1, *e2) {
                    return Ok(TExpr { ty: target, ..te });
                }
                return Err(te);
            }
            // *mut T -> *T
            (TyKind::Ptr(true, e1), TyKind::Ptr(false, e2)) => {
                if self.unify(*e1, *e2) {
                    return Ok(TExpr { ty: target, ..te });
                }
                return Err(te);
            }
            // *T -> dyn Trait (a fat pointer; the pointee must implement the trait)
            (TyKind::Ptr(pm, inner), TyKind::Dyn(trait_id, eff)) => {
                let inner_t = self.tys.resolve(*inner, true);
                let tname = self.traits[*trait_id as usize].name.clone();
                if !self.type_satisfies_trait(inner_t, &tname, 0) {
                    return Err(te);
                }
                if let Some(vt) = self.vtable_for(*trait_id, inner_t, te.span) {
                    // every method must satisfy the trait object's negative bounds
                    let need_neg = Effects::ALL.without(Effects(*eff));
                    if !need_neg.is_empty() {
                        for inst in self.vtables[vt as usize].2.clone() {
                            self.fn_value_obligations.push((inst, need_neg, te.span));
                        }
                    }
                    let _ = pm;
                    let span = te.span;
                    return Ok(TExpr { kind: TExprKind::DynFrom { expr: Box::new(te), vtable: vt }, ty: target, span });
                }
                return Err(te);
            }
            // fn reference -> fn value
            (TyKind::Fn(p1, r1, ef1), TyKind::Fn(p2, r2, ef2)) => {
                if p1.len() == p2.len() {
                    let mut ok = self.unify(*r1, *r2);
                    for (a, b) in p1.iter().zip(p2.iter()) {
                        ok &= self.unify(*a, *b);
                    }
                    if ok {
                        // the target may require the absence of effects the value has
                        let need_neg = Effects::ALL.without(Effects(*ef2));
                        if !need_neg.is_empty() {
                            match &te.kind {
                                TExprKind::FnToFat(inst) | TExprKind::Closure { inst, .. } => {
                                    self.fn_value_obligations.push((*inst, need_neg, te.span));
                                }
                                _ => {
                                    if !Effects(*ef1).without(Effects(*ef2)).is_empty() {
                                        return Err(te);
                                    }
                                }
                            }
                        }
                        return Ok(TExpr { ty: target, ..te });
                    }
                }
                return Err(te);
            }
            _ => {}
        }
        if self.unify(from, to) {
            return Ok(TExpr { ty: target, ..te });
        }
        Err(te)
    }

    pub fn coerce_or_error(&mut self, te: TExpr, target: TyId, what: &str) -> TExpr {
        match self.coerce(te, target) {
            Ok(e) => e,
            Err(e) => {
                let got = self.type_name(e.ty);
                let want = self.type_name(target);
                let mut hint = String::new();
                let f = self.tys.shallow(e.ty);
                let t = self.tys.shallow(target);
                match (self.tys.kind(f).clone(), self.tys.kind(t).clone()) {
                    (TyKind::Int(_), TyKind::Int(_)) | (TyKind::Int(_), TyKind::Float(_)) | (TyKind::Float(_), TyKind::Int(_)) | (TyKind::Float(_), TyKind::Float(_)) => {
                        hint = format!("; numeric conversions are explicit: write `... as {}`", want);
                    }
                    (TyKind::Distinct(_), _) | (_, TyKind::Distinct(_)) => {
                        hint = format!("; distinct types convert explicitly: write `... as {}`", want);
                    }
                    (TyKind::ErrUnion(..), _) => {
                        hint = "; use `try` to propagate the error or `catch` to handle it".into();
                    }
                    (TyKind::Opt(_), _) => {
                        hint = "; unwrap the optional with `orelse`, `.?`, or `if (x) |v|`".into();
                    }
                    (TyKind::Slice(_, _), TyKind::Str) => {
                        hint = "; build an owned String with `String.from(...)`".into();
                    }
                    _ => {}
                }
                self.error(e.span, format!("type mismatch in {}: expected `{}` but found `{}`{}", what, want, got, hint));
                TExpr { ty: target, ..e }
            }
        }
    }

    pub fn finalize_expr(&mut self, te: TExpr) -> TExpr {
        let ty = self.tys.resolve(te.ty, true);
        TExpr { ty, ..te }
    }

    pub fn mk(&mut self, kind: TExprKind, ty: TyId, span: Span) -> TExpr {
        TExpr { kind, ty, span }
    }

    pub fn error_expr(&mut self, span: Span) -> TExpr {
        let t = self.tys.never();
        TExpr { kind: TExprKind::Unreachable, ty: t, span }
    }

    // ----- expressions --------------------------------------------------------

    pub fn check_expr(&mut self, e: &Expr, expected: Option<TyId>) -> TExpr {
        match e {
            Expr::Lit { value, span } => self.check_lit(value, expected, *span),
            Expr::Ident { name, span } => self.check_ident(name, *span),
            Expr::Field { base, name, span } => self.check_field(base, name, *span, expected),
            Expr::ImplicitVariant { name, args, span } => self.check_implicit_variant(name, args, expected, *span),
            Expr::Index { base, index, span } => self.check_index(base, index, *span),
            Expr::SliceOp { base, start, end, span } => self.check_slice_op(base, start.as_deref(), end.as_deref(), *span),
            Expr::Call { callee, args, span } => self.check_call(callee, args, expected, *span),
            Expr::MethodCall { receiver, method, args, span } => self.check_method_call(receiver, method, args, expected, *span),
            Expr::Unary { op, expr, span } => self.check_unary(*op, expr, expected, *span),
            Expr::Binary { op, lhs, rhs, span } => self.check_binary(*op, lhs, rhs, expected, *span),
            Expr::Pipe { lhs, rhs, span } => self.check_pipe(lhs, rhs, expected, *span),
            Expr::Try { expr, span } => self.check_try(expr, *span),
            Expr::Catch { expr, binding, handler, span } => self.check_catch(expr, binding.as_deref(), handler, expected, *span),
            Expr::OrElse { expr, default, span } => self.check_orelse(expr, default, expected, *span),
            Expr::Unwrap { expr, span } => {
                let inner = self.check_expr(expr, None);
                let t = self.tys.shallow(inner.ty);
                match self.tys.kind(t).clone() {
                    TyKind::Opt(e) => {
                        self.add_effect(Effects::PANICS, *span, "`.?` panics when the optional is null");
                        self.mk(TExprKind::Unwrap { expr: Box::new(inner), proven: false }, e, *span)
                    }
                    _ => {
                        let tn = self.type_name(t);
                        self.error(*span, format!("`.?` requires an optional but found `{}`", tn));
                        self.error_expr(*span)
                    }
                }
            }
            Expr::Deref { expr, span } => {
                let inner = self.check_expr(expr, None);
                let t = self.tys.shallow(inner.ty);
                match self.tys.kind(t).clone() {
                    TyKind::Ptr(_, e) => self.mk(TExprKind::Deref(Box::new(inner)), e, *span),
                    _ => {
                        let tn = self.type_name(t);
                        self.error(*span, format!("`.*` requires a pointer but found `{}`", tn));
                        self.error_expr(*span)
                    }
                }
            }
            Expr::If { cond, then, els, span } => self.check_if(cond, then, els.as_deref(), expected, *span),
            Expr::IfCapture { cond, binding, then, els, span } => self.check_if_capture(cond, binding, then, els.as_deref(), expected, *span),
            Expr::Match { scrutinee, arms, span } => self.check_match(scrutinee, arms, expected, *span),
            Expr::Block(b) => {
                let label = b.label.clone();
                let tb = self.check_block_labeled(b, expected, label);
                let ty = tb.ty;
                let span = tb.span;
                TExpr { kind: TExprKind::Block(tb), ty, span }
            }
            Expr::StructLit { ty, fields, span } => self.check_struct_lit(ty.as_ref(), fields, expected, *span),
            Expr::ArrayLit { elems, span } => self.check_array_lit(elems, expected, *span),
            Expr::TupleLit { elems, span } => self.check_tuple_lit(elems, expected, *span),
            Expr::Closure(c) => self.check_closure(c, expected),
            Expr::Range { span, .. } => {
                self.error(*span, "a range is only valid in `for (a..b)` or in slicing `x[a..b]`");
                self.error_expr(*span)
            }
            Expr::Cast { expr, ty, span } => self.check_cast(expr, ty, *span),
            Expr::Null { span } => {
                let t = match expected.map(|t| self.tys.shallow(t)) {
                    Some(t) if matches!(self.tys.kind(t), TyKind::Opt(_)) => t,
                    Some(t) if matches!(self.tys.kind(t), TyKind::Infer(_)) => {
                        let v = self.tys.fresh_infer();
                        let o = self.tys.opt(v);
                        self.unify(t, o);
                        o
                    }
                    _ => {
                        let v = self.tys.fresh_infer();
                        self.tys.opt(v)
                    }
                };
                self.mk(TExprKind::OptNull, t, *span)
            }
            Expr::Undefined { span } => match expected {
                Some(t) => self.mk(TExprKind::Undefined, t, *span),
                None => {
                    self.error(*span, "`undefined` needs a type from context, e.g. `var buf: [1500]u8 = undefined`");
                    self.error_expr(*span)
                }
            },
            Expr::Unreachable { span } => {
                self.add_effect(Effects::PANICS, *span, "`unreachable` panics if reached");
                let t = self.tys.never();
                self.mk(TExprKind::Unreachable, t, *span)
            }
            Expr::ErrorLit { name, span } => {
                let id = self.error_id(name);
                let t = self.tys.intern(TyKind::ErrorSet(None));
                let te = self.mk(TExprKind::ErrVal(id), t, *span);
                match expected {
                    Some(exp) => match self.coerce(te, exp) {
                        Ok(e) => e,
                        Err(e) => e,
                    },
                    None => te,
                }
            }
            Expr::Builtin { name, args, span } => self.check_at_builtin(name, args, expected, *span),
            Expr::BinConstruct { segments, target, span } => self.check_bin_construct(segments, target, *span),
            Expr::Comptime { expr, span } => {
                let saved = self.cur().is_comptime;
                self.cur().is_comptime = true;
                let inner = self.check_expr(expr, expected);
                self.cur().is_comptime = saved;
                let inner = self.finalize_expr(inner);
                match crate::comptime::eval_const_expr(self, &inner) {
                    Some(v) => self.mk(TExprKind::Value(v), inner.ty, *span),
                    None => {
                        self.error(*span, "this expression cannot be evaluated at compile time");
                        inner
                    }
                }
            }
            Expr::TypeVal { ty, span } => {
                let cur = self.cur.as_ref().unwrap();
                let (g, s, m) = (cur.generics.clone(), cur.self_ty, cur.module);
                let t = self.resolve_type(ty, &g, s, m);
                let tt = self.tys.type_ty();
                self.mk(TExprKind::TypeVal(t), tt, *span)
            }
            Expr::Unsafe { body, span } => {
                self.cur().unsafe_depth += 1;
                let tb = self.check_block(body, expected, None);
                self.cur().unsafe_depth -= 1;
                let ty = tb.ty;
                TExpr { kind: TExprKind::Block(tb), ty, span: *span }
            }
        }
    }

    fn check_lit(&mut self, value: &Lit, expected: Option<TyId>, span: Span) -> TExpr {
        let exp = expected.map(|t| self.tys.shallow(t));
        match value {
            Lit::Int(v) => {
                let v = *v as i128;
                if let Some(t) = exp {
                    match self.tys.kind(t).clone() {
                        TyKind::Int(it) => {
                            if v < it.min() || v > it.max() {
                                self.error(span, format!("literal `{}` does not fit in `{}` (range {} to {})", v, it.name(), it.min(), it.max()));
                            }
                            return self.mk(TExprKind::Int(v), t, span);
                        }
                        TyKind::Float(_) => return self.mk(TExprKind::Float(v as f64), t, span),
                        TyKind::Infer(n) if self.tys.infer_kinds[n as usize] == InferKind::Float => {
                            return self.mk(TExprKind::Float(v as f64), t, span);
                        }
                        TyKind::Infer(n) if self.tys.infer_kinds[n as usize] == InferKind::Integer => {
                            return self.mk(TExprKind::Int(v), t, span);
                        }
                        TyKind::Distinct(d) => {
                            let u = self.distinct_underlying[&d];
                            if self.tys.is_integer(u) {
                                return self.mk(TExprKind::Int(v), t, span);
                            }
                            if self.tys.is_float(u) {
                                return self.mk(TExprKind::Float(v as f64), t, span);
                            }
                        }
                        _ => {}
                    }
                }
                let t = self.tys.int_lit();
                self.mk(TExprKind::Int(v), t, span)
            }
            Lit::Float(f) => {
                if let Some(t) = exp {
                    match self.tys.kind(t).clone() {
                        TyKind::Float(_) => return self.mk(TExprKind::Float(*f), t, span),
                        TyKind::Distinct(d) => {
                            let u = self.distinct_underlying[&d];
                            if self.tys.is_float(u) {
                                return self.mk(TExprKind::Float(*f), t, span);
                            }
                        }
                        _ => {}
                    }
                }
                let t = self.tys.float_lit();
                self.mk(TExprKind::Float(*f), t, span)
            }
            Lit::Str(s) | Lit::Bytes(s) => {
                let t = self.tys.str_slice();
                let te = self.mk(TExprKind::Str(s.clone()), t, span);
                // `[N]u8` from a string literal when the expected type is an array of matching length
                if let Some(t) = exp {
                    if let TyKind::Array(n, e) = self.tys.kind(t).clone() {
                        if n as usize == s.len() && matches!(self.tys.kind(self.tys.shallow(e)), TyKind::Int(IntTy::U8)) {
                            let u8t = self.tys.u8();
                            let elems = s.iter().map(|&b| TExpr { kind: TExprKind::Int(b as i128), ty: u8t, span }).collect();
                            return self.mk(TExprKind::ArrayLit(elems), t, span);
                        }
                    }
                }
                te
            }
            Lit::Char(c) => {
                if let Some(t) = exp {
                    if let TyKind::Int(it) = self.tys.kind(t).clone() {
                        if (*c as i128) <= it.max() {
                            return self.mk(TExprKind::Int(*c as i128), t, span);
                        }
                        self.error(span, format!("character literal does not fit in `{}`", it.name()));
                    }
                }
                let t = self.tys.char();
                self.mk(TExprKind::Char(*c), t, span)
            }
            Lit::Bool(b) => {
                let t = self.tys.bool();
                self.mk(TExprKind::Bool(*b), t, span)
            }
        }
    }

    pub fn check_ident(&mut self, name: &str, span: Span) -> TExpr {
        if let Some(entry) = self.cur().lookup(name) {
            let cur = self.cur.as_ref().unwrap();
            let local = &cur.locals[entry.local as usize];
            let ty = local.ty;
            if cur.moved.contains(&entry.local) {
                let msp = cur.moved_spans.get(&entry.local).copied();
                let n = name.to_string();
                self.error_note(span, format!("use of `{}` after it was moved", n), msp, "value moved here; use `.clone()` if both places need it");
            }
            let e = self.mk(TExprKind::Local(entry.local), ty, span);
            if entry.auto_deref {
                let inner = match self.tys.kind(self.tys.shallow(ty)).clone() {
                    TyKind::Ptr(_, e) => e,
                    _ => ty,
                };
                return self.mk(TExprKind::Deref(Box::new(e)), inner, span);
            }
            return e;
        }
        let module = self.cur().module;
        match self.lookup_item(module, name) {
            Some(ItemRef::Fn(fid)) => {
                let def = self.fns[fid as usize].clone();
                if def.is_generic {
                    self.error(span, format!("`{}` is generic; call it with arguments so its type parameters can be inferred", name));
                    return self.error_expr(span);
                }
                if def.decl.params.iter().any(|p| p.owned) {
                    self.error(span, format!("`{}` takes `own` parameters and cannot be used as a function value; call it directly", name));
                    return self.error_expr(span);
                }
                let inst = self.instantiate(fid, vec![], span);
                let (ps, r) = self.fn_sig(inst);
                let f = &self.funcs[inst as usize];
                let neg = f.declared_neg;
                let ft = self.tys.intern(TyKind::Fn(ps, r, Effects::ALL.without(neg).0));
                self.mk(TExprKind::FnToFat(inst), ft, span)
            }
            Some(ItemRef::Const(id)) => match self.resolve_const(id) {
                Some((ty, te)) => match &te.kind {
                    TExprKind::Value(v) if matches!(v, Value::Int(_) | Value::Float(_) | Value::Bool(_) | Value::Char(_) | Value::Str(_)) => TExpr { kind: te.kind.clone(), ty, span },
                    _ => self.mk(TExprKind::Const(id), ty, span),
                },
                None => self.error_expr(span),
            },
            Some(ItemRef::Global(id)) => {
                let ty = self.globals[id as usize].ty.unwrap_or_else(|| self.tys.void());
                if self.cur().unsafe_depth == 0 && !self.cur().is_comptime {
                    let n = name.to_string();
                    self.error_note(
                        span,
                        format!("access to mutable global `{}` requires an `unsafe` block", n),
                        Some(self.globals[id as usize].span),
                        "mutable globals may be touched from any thread; wrap the access in `unsafe { }` (section 8.4)",
                    );
                }
                let fname = self.cur().fn_name.clone();
                self.global_uses.push((id, span, fname));
                self.add_effect(Effects::SHARED_MUTABLE, span, "access to a mutable global");
                self.mk(TExprKind::Global(id), ty, span)
            }
            Some(ItemRef::Struct(id)) => {
                let def = &self.structs[id as usize];
                if !def.type_params.is_empty() {
                    let n = def.name.clone();
                    let ps = def.type_params.join(", ");
                    self.error(span, format!("`{}` is generic; write `{}({})`", n, n, ps));
                    return self.error_expr(span);
                }
                let t = self.tys.intern(TyKind::Struct(id, vec![]));
                self.note_used(t);
                let tt = self.tys.type_ty();
                self.mk(TExprKind::TypeVal(t), tt, span)
            }
            Some(ItemRef::Enum(id)) => {
                let def = &self.enums[id as usize];
                if !def.type_params.is_empty() {
                    let n = def.name.clone();
                    self.error(span, format!("`{}` is generic; write `{}(...)`", n, n));
                    return self.error_expr(span);
                }
                let t = self.tys.intern(TyKind::Enum(id, vec![]));
                self.note_used(t);
                let tt = self.tys.type_ty();
                self.mk(TExprKind::TypeVal(t), tt, span)
            }
            Some(ItemRef::Alias(id)) => {
                let t = self.resolve_alias(id);
                let tt = self.tys.type_ty();
                self.mk(TExprKind::TypeVal(t), tt, span)
            }
            Some(ItemRef::ErrorSet(id)) => {
                let t = self.tys.intern(TyKind::ErrorSet(Some(id)));
                let tt = self.tys.type_ty();
                self.mk(TExprKind::TypeVal(t), tt, span)
            }
            Some(ItemRef::Trait(_)) => {
                self.error(span, format!("`{}` is a trait and cannot be used as a value", name));
                self.error_expr(span)
            }
            Some(ItemRef::Module(m)) => {
                let t = self.tys.intern(TyKind::Namespace(format!("module:{}", m)));
                self.mk(TExprKind::Unit, t, span)
            }
            None => {
                if let Some(&t) = self.cur().generics.get(name) {
                    let tt = self.tys.type_ty();
                    return self.mk(TExprKind::TypeVal(t), tt, span);
                }
                if name == "String" {
                    let t = self.tys.string();
                    let tt = self.tys.type_ty();
                    return self.mk(TExprKind::TypeVal(t), tt, span);
                }
                if matches!(
                    name,
                    "List"
                        | "Map"
                        | "math"
                        | "io"
                        | "os"
                        | "time"
                        | "random"
                        | "context"
                        | "process"
                        | "utf8"
                        | "ascii"
                        | "mem"
                        | "slice"
                        | "fmt"
                        | "test"
                        | "alloc"
                        | "Ordering"
                        | "net"
                        | "thread"
                        | "sync"
                ) {
                    let t = self.tys.intern(TyKind::Namespace(name.to_string()));
                    return self.mk(TExprKind::Unit, t, span);
                }
                if matches!(name, "println" | "print" | "eprintln" | "format" | "expect" | "expect_eq" | "panic" | "assert") {
                    self.error(span, format!("`{}` is a builtin function; call it directly", name));
                    return self.error_expr(span);
                }
                self.error(span, format!("cannot find `{}` in this scope", name));
                self.error_expr(span)
            }
        }
    }

    fn check_field(&mut self, base: &Expr, name: &str, span: Span, expected: Option<TyId>) -> TExpr {
        let b = self.check_expr(base, None);
        let bt = self.tys.shallow(b.ty);
        // static access on a type or namespace
        match self.tys.kind(bt).clone() {
            TyKind::Type => {
                if let TExprKind::TypeVal(t) = b.kind {
                    return self.check_static_member(t, name, span, expected);
                }
            }
            TyKind::Namespace(ns) => return self.check_namespace_member(&ns, name, span, expected),
            _ => {}
        }
        self.field_access(b, name, span)
    }

    /// Field access on a value (with auto-deref through pointers).
    pub fn field_access(&mut self, mut b: TExpr, name: &str, span: Span) -> TExpr {
        let mut bt = self.tys.shallow(b.ty);
        // auto-deref pointers
        while let TyKind::Ptr(_, inner) = self.tys.kind(bt).clone() {
            b = self.mk(TExprKind::Deref(Box::new(b)), inner, span);
            bt = self.tys.shallow(inner);
        }
        match self.tys.kind(bt).clone() {
            TyKind::Struct(d, _) => {
                let def = self.structs[d as usize].clone();
                let ftys = self.struct_field_types(bt);
                if let Some(idx) = def.fields.iter().position(|f| f.name == name) {
                    let fty = ftys[idx];
                    if def.kind == StructKind::RefClass {
                        return self.mk(TExprKind::RefField { base: Box::new(b), idx: idx as u32 }, fty, span);
                    }
                    return self.mk(TExprKind::Field { base: Box::new(b), idx: idx as u32 }, fty, span);
                }
                // method reference without call? not supported as a value
                let dn = def.name.clone();
                let mut hint = String::new();
                if let Some(close) = def.fields.iter().map(|f| f.name.as_str()).find(|f| similar(f, name)) {
                    hint = format!("; did you mean `{}`?", close);
                }
                self.error(span, format!("`{}` has no field `{}`{}", dn, name, hint));
                self.error_expr(span)
            }
            TyKind::Tuple(ts) => match name.parse::<usize>() {
                Ok(i) if i < ts.len() => self.mk(TExprKind::TupleField { base: Box::new(b), idx: i as u32 }, ts[i], span),
                _ => {
                    self.error(span, format!("tuple has no field `{}`", name));
                    self.error_expr(span)
                }
            },
            TyKind::Slice(..) | TyKind::Array(..) | TyKind::List(_) | TyKind::Str | TyKind::Map(..) => match name {
                "len" => {
                    let u = self.tys.usize();
                    self.mk(TExprKind::Builtin { op: Builtin::Len, args: vec![b], tys: vec![] }, u, span)
                }
                "ptr" if matches!(self.tys.kind(bt), TyKind::Slice(..)) => {
                    if self.cur().unsafe_depth == 0 {
                        self.error(span, "`.ptr` of a slice is a raw pointer and requires an `unsafe` block");
                    }
                    let (m, e) = match self.tys.kind(bt).clone() {
                        TyKind::Slice(m, e) => (m, e),
                        _ => unreachable!(),
                    };
                    let pt = self.tys.ptr(m, e);
                    self.mk(TExprKind::Builtin { op: Builtin::PtrAdd, args: vec![b], tys: vec![] }, pt, span)
                }
                _ => {
                    let tn = self.type_name(bt);
                    self.error(span, format!("`{}` has no field `{}` (methods are called with parentheses)", tn, name));
                    self.error_expr(span)
                }
            },
            TyKind::Opt(_) => {
                let tn = self.type_name(bt);
                self.error(span, format!("cannot access field `{}` of optional `{}`; unwrap it first with `orelse`, `.?`, or `if (x) |v|`", name, tn));
                self.error_expr(span)
            }
            TyKind::ErrUnion(..) => {
                let tn = self.type_name(bt);
                self.error(span, format!("cannot access field `{}` of `{}`; use `try` first", name, tn));
                self.error_expr(span)
            }
            TyKind::Enum(d, _) => {
                let dn = self.enums[d as usize].name.clone();
                self.error(span, format!("`{}` is an enum; use `match` to access variant payloads", dn));
                self.error_expr(span)
            }
            _ => {
                let tn = self.type_name(bt);
                self.error(span, format!("`{}` has no field `{}`", tn, name));
                self.error_expr(span)
            }
        }
    }

    fn check_implicit_variant(&mut self, name: &str, args: &[Expr], expected: Option<TyId>, span: Span) -> TExpr {
        let exp = match expected {
            Some(t) => self.tys.shallow(t),
            None => {
                self.error(span, format!("cannot infer the enum type of `.{}`; write `EnumName.{}`", name, name));
                return self.error_expr(span);
            }
        };
        // peel optionals / error unions
        let (target, wrap) = match self.tys.kind(exp).clone() {
            TyKind::Opt(inner) => (self.tys.shallow(inner), Some(exp)),
            TyKind::ErrUnion(_, inner) => (self.tys.shallow(inner), Some(exp)),
            _ => (exp, None),
        };
        match self.tys.kind(target).clone() {
            TyKind::Enum(..) => {
                let v = self.check_variant_ctor(target, name, args, span);
                match wrap {
                    Some(w) => self.coerce_or_error(v, w, "enum value"),
                    None => v,
                }
            }
            _ => {
                let tn = self.type_name(exp);
                self.error(span, format!("`.{}` needs an enum type from context, but the expected type is `{}`", name, tn));
                self.error_expr(span)
            }
        }
    }

    pub fn check_variant_ctor(&mut self, enum_ty: TyId, name: &str, args: &[Expr], span: Span) -> TExpr {
        let d = match self.tys.kind(enum_ty).clone() {
            TyKind::Enum(d, _) => d,
            _ => unreachable!(),
        };
        let def = self.enums[d as usize].clone();
        let vtys = self.enum_variant_types(enum_ty);
        let idx = match def.variants.iter().position(|v| v.name == name) {
            Some(i) => i,
            None => {
                let dn = def.name.clone();
                self.error(span, format!("enum `{}` has no variant `{}`", dn, name));
                return self.error_expr(span);
            }
        };
        let want = &vtys[idx];
        if args.len() != want.len() {
            let dn = def.name.clone();
            self.error(span, format!("variant `{}.{}` takes {} value(s) but {} were given", dn, name, want.len(), args.len()));
            return self.error_expr(span);
        }
        let want = want.clone();
        let mut payload = Vec::new();
        for (a, &t) in args.iter().zip(want.iter()) {
            let te = self.check_expr(a, Some(t));
            let te = self.coerce_or_error(te, t, "enum variant payload");
            let te = self.take_ownership(te);
            payload.push(te);
        }
        self.note_used(enum_ty);
        self.mk(TExprKind::EnumLit { variant: idx as u32, payload }, enum_ty, span)
    }

    fn check_index(&mut self, base: &Expr, index: &Expr, span: Span) -> TExpr {
        let b = self.check_expr(base, None);
        let mut b = b;
        let mut bt = self.tys.shallow(b.ty);
        while let TyKind::Ptr(_, inner) = self.tys.kind(bt).clone() {
            b = self.mk(TExprKind::Deref(Box::new(b)), inner, span);
            bt = self.tys.shallow(inner);
        }
        let elem = match self.tys.kind(bt).clone() {
            TyKind::Array(_, e) | TyKind::Slice(_, e) | TyKind::List(e) => e,
            TyKind::Str => self.tys.u8(),
            TyKind::Map(k, v) => {
                // map[key] -> ?V ; String-keyed maps accept []u8 keys
                let lk = if matches!(self.tys.kind(self.tys.shallow(k)), TyKind::Str) { self.tys.str_slice() } else { k };
                let key = self.check_expr(index, Some(lk));
                let key = self.coerce_or_error(key, lk, "map key");
                let ot = self.tys.opt(v);
                return self.mk(TExprKind::Builtin { op: Builtin::MapGet, args: vec![b, key], tys: vec![] }, ot, span);
            }
            _ => {
                let tn = self.type_name(bt);
                self.error(span, format!("cannot index a value of type `{}`", tn));
                return self.error_expr(span);
            }
        };
        let usize_t = self.tys.usize();
        let i = self.check_expr(index, Some(usize_t));
        let i = self.coerce_or_error(i, usize_t, "index");
        let proven = self.index_proven(&b, &i);
        if !proven {
            self.add_effect(Effects::PANICS, span, "index may be out of bounds");
        }
        self.mk(TExprKind::Index { base: Box::new(b), index: Box::new(i), proven }, elem, span)
    }

    /// Range analysis for indexing (E6): literal indices into arrays, loop index
    /// variables over the same slice, and guarded locals.
    fn index_proven(&mut self, base: &TExpr, index: &TExpr) -> bool {
        let bt = self.tys.shallow(base.ty);
        if let TyKind::Array(n, _) = self.tys.kind(bt).clone() {
            if let Some((lo, hi)) = self.expr_range(index) {
                return lo >= 0 && hi < n as i128;
            }
        }
        if let (TExprKind::Local(bl), TExprKind::Local(il)) = (&base.kind, &index.kind) {
            let cur = self.cur.as_ref().unwrap();
            if cur.index_of.get(il) == Some(bl) {
                return true;
            }
        }
        false
    }

    /// Best-effort interval for an integer expression.
    pub fn expr_range(&self, e: &TExpr) -> Option<(i128, i128)> {
        match &e.kind {
            TExprKind::Int(v) => Some((*v, *v)),
            TExprKind::Char(c) => Some((*c as i128, *c as i128)),
            TExprKind::Bool(b) => Some((*b as i128, *b as i128)),
            TExprKind::Value(Value::Int(v)) => Some((*v, *v)),
            TExprKind::Local(l) => {
                let cur = self.cur.as_ref()?;
                if let Some(r) = cur.ranges.get(l) {
                    return Some(*r);
                }
                self.type_range(cur.locals[*l as usize].ty)
            }
            TExprKind::Cast { expr, kind: CastKind::IntToInt { .. } } => {
                let inner = self.expr_range(expr)?;
                let tr = self.type_range(e.ty)?;
                Some((inner.0.max(tr.0), inner.1.min(tr.1)))
            }
            TExprKind::Binary { op, lhs, rhs, .. } => {
                let a = self.expr_range(lhs)?;
                let b = self.expr_range(rhs)?;
                let r = match op {
                    // ranges are i128 and a u64 * u64 range exceeds it: saturate, never overflow
                    BinOp::Add | BinOp::AddWrap | BinOp::AddSat => (a.0.saturating_add(b.0), a.1.saturating_add(b.1)),
                    BinOp::Sub | BinOp::SubWrap | BinOp::SubSat => (a.0.saturating_sub(b.1), a.1.saturating_sub(b.0)),
                    BinOp::Mul | BinOp::MulWrap | BinOp::MulSat => {
                        let c = [a.0.saturating_mul(b.0), a.0.saturating_mul(b.1), a.1.saturating_mul(b.0), a.1.saturating_mul(b.1)];
                        (*c.iter().min().unwrap(), *c.iter().max().unwrap())
                    }
                    BinOp::Rem if b.0 > 0 => (0.max(a.0.min(0)), (b.1 - 1).min(a.1.max(0))),
                    BinOp::Div if b.0 > 0 => (a.0.min(0).min(a.0 / b.0), a.1.max(0).max(a.1 / b.0)),
                    BinOp::BitAnd if b.0 >= 0 && a.0 >= 0 => (0, a.1.min(b.1)),
                    BinOp::Shr if a.0 >= 0 && b.0 >= 0 => (a.0 >> b.1.min(127), a.1 >> b.0.min(127)),
                    _ => return self.type_range(e.ty),
                };
                match self.type_range(e.ty) {
                    Some(tr) if matches!(op, BinOp::AddWrap | BinOp::SubWrap | BinOp::MulWrap | BinOp::AddSat | BinOp::SubSat | BinOp::MulSat) => Some((r.0.max(tr.0), r.1.min(tr.1))),
                    _ => Some(r),
                }
            }
            TExprKind::Builtin { op: Builtin::Len, args, .. } => {
                if let TyKind::Array(n, _) = self.tys.kind(self.tys.shallow(args[0].ty)) {
                    return Some((*n as i128, *n as i128));
                }
                Some((0, i128::MAX))
            }
            _ => self.type_range(e.ty),
        }
    }

    pub fn type_range(&self, t: TyId) -> Option<(i128, i128)> {
        match self.tys.kind(self.tys.shallow(t)) {
            TyKind::Int(i) => Some((i.min(), i.max())),
            TyKind::Bool => Some((0, 1)),
            TyKind::Char => Some((0, 0x10FFFF)),
            _ => None,
        }
    }

    fn check_slice_op(&mut self, base: &Expr, start: Option<&Expr>, end: Option<&Expr>, span: Span) -> TExpr {
        let b = self.check_expr(base, None);
        let mut b = b;
        let mut bt = self.tys.shallow(b.ty);
        while let TyKind::Ptr(_, inner) = self.tys.kind(bt).clone() {
            b = self.mk(TExprKind::Deref(Box::new(b)), inner, span);
            bt = self.tys.shallow(inner);
        }
        let (mutable, elem) = match self.tys.kind(bt).clone() {
            TyKind::Array(_, e) => (self.place_mutable(&b), e),
            TyKind::Slice(m, e) => (m, e),
            TyKind::List(e) => (self.place_mutable(&b), e),
            TyKind::Str => (self.place_mutable(&b), self.tys.u8()),
            _ => {
                let tn = self.type_name(bt);
                self.error(span, format!("cannot slice a value of type `{}`", tn));
                return self.error_expr(span);
            }
        };
        let usize_t = self.tys.usize();
        let s = match start {
            Some(s) => {
                let e = self.check_expr(s, Some(usize_t));
                Some(Box::new(self.coerce_or_error(e, usize_t, "slice start")))
            }
            None => None,
        };
        let e = match end {
            Some(x) => {
                let e = self.check_expr(x, Some(usize_t));
                Some(Box::new(self.coerce_or_error(e, usize_t, "slice end")))
            }
            None => None,
        };
        self.add_effect(Effects::PANICS, span, "slicing checks its bounds and panics when they are out of range");
        let st = self.tys.slice(mutable, elem);
        self.mk(TExprKind::SliceOp { base: Box::new(b), start: s, end: e }, st, span)
    }

    fn check_unary(&mut self, op: UnOp, expr: &Expr, expected: Option<TyId>, span: Span) -> TExpr {
        match op {
            UnOp::AddrOf | UnOp::AddrOfMut => {
                let mutable = op == UnOp::AddrOfMut;
                // `&arr` with a slice expected: coerce
                let inner_expected = match expected.map(|t| self.tys.shallow(t)) {
                    Some(t) => match self.tys.kind(t).clone() {
                        TyKind::Ptr(_, e) => Some(e),
                        _ => None,
                    },
                    None => None,
                };
                let inner = self.check_expr(expr, inner_expected);
                if !self.is_place(&inner) {
                    // taking the address of a temporary is allowed: materialize it
                }
                if mutable && self.is_place(&inner) && !self.place_mutable(&inner) {
                    self.error(span, "cannot take a mutable reference to an immutable value; declare it with `var`");
                }
                let it = inner.ty;
                let pt = self.tys.ptr(mutable, it);
                self.mk(TExprKind::AddrOf { expr: Box::new(inner), mutable }, pt, span)
            }
            UnOp::Not => {
                let bt = self.tys.bool();
                let inner = self.check_expr(expr, Some(bt));
                let inner = self.coerce_or_error(inner, bt, "operand of `!`");
                self.mk(TExprKind::Unary { op, expr: Box::new(inner), mode: ArithMode::Plain }, bt, span)
            }
            UnOp::Neg => {
                let inner = self.check_expr(expr, expected);
                let t = self.tys.shallow(inner.ty);
                if self.tys.is_float(t) {
                    return self.mk(TExprKind::Unary { op, expr: Box::new(inner), mode: ArithMode::Float }, t, span);
                }
                if self.tys.is_integer(t) {
                    if let Some(it) = self.tys.as_int(t) {
                        if !it.is_signed() {
                            self.error(span, format!("cannot negate a value of unsigned type `{}`", it.name()));
                        }
                    }
                    // -MIN overflows
                    let proven = matches!(self.expr_range(&inner), Some((lo, _)) if self.type_range(t).map(|r| lo > r.0).unwrap_or(false));
                    if !proven {
                        self.add_effect(Effects::PANICS, span, "negation overflows for the minimum value");
                    }
                    return self.mk(TExprKind::Unary { op, expr: Box::new(inner), mode: ArithMode::Checked }, t, span);
                }
                if let TyKind::Distinct(d) = self.tys.kind(t).clone() {
                    let u = self.distinct_underlying[&d];
                    let mode = if self.tys.is_float(u) { ArithMode::Float } else { ArithMode::Checked };
                    return self.mk(TExprKind::Unary { op, expr: Box::new(inner), mode }, t, span);
                }
                let tn = self.type_name(t);
                self.error(span, format!("cannot negate a value of type `{}`", tn));
                self.error_expr(span)
            }
            UnOp::BitNot => {
                let inner = self.check_expr(expr, expected);
                let t = self.tys.shallow(inner.ty);
                if !self.tys.is_integer(t) {
                    let tn = self.type_name(t);
                    self.error(span, format!("`~` requires an integer but found `{}`", tn));
                }
                self.mk(TExprKind::Unary { op, expr: Box::new(inner), mode: ArithMode::Plain }, t, span)
            }
        }
    }

    fn check_binary(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, expected: Option<TyId>, span: Span) -> TExpr {
        if op.is_logical() {
            let bt = self.tys.bool();
            let l = self.check_expr(lhs, Some(bt));
            let l = self.coerce_or_error(l, bt, "operand of a logical operator");
            let r = self.check_expr(rhs, Some(bt));
            let r = self.coerce_or_error(r, bt, "operand of a logical operator");
            return self.mk(TExprKind::Logical { and: op == BinOp::And, lhs: Box::new(l), rhs: Box::new(r) }, bt, span);
        }
        if op.is_comparison() {
            return self.check_comparison(op, lhs, rhs, span);
        }
        // arithmetic / bitwise: operands share a type
        let hint = if op.is_arith() { expected } else { None };
        let l = self.check_expr(lhs, hint);
        let lt = self.tys.shallow(l.ty);
        // shifts: rhs is any integer
        if matches!(op, BinOp::Shl | BinOp::Shr) {
            let r = self.check_expr(rhs, None);
            let rt = self.tys.shallow(r.ty);
            if !self.tys.is_integer(lt) || !self.tys.is_integer(rt) {
                self.error(span, "shift operands must be integers");
            }
            let bits = self.tys.as_int(lt).map(|i| i.bits() as i128);
            let proven = match (self.expr_range(&r), bits) {
                (Some((lo, hi)), Some(b)) => lo >= 0 && hi < b,
                (Some((lo, hi)), None) => lo >= 0 && hi < 64,
                _ => false,
            };
            if !proven {
                self.add_effect(Effects::PANICS, span, "shift amount may exceed the bit width");
            }
            let res = self.tys.resolve(lt, false);
            return self.mk(TExprKind::Binary { op, lhs: Box::new(l), rhs: Box::new(r), mode: ArithMode::Checked, proven }, res, span);
        }
        let r = self.check_expr(rhs, Some(lt));
        let r = match self.coerce(r, lt) {
            Ok(r) => r,
            Err(r) => {
                // maybe the lhs was a literal and the rhs decides
                let rt = r.ty;
                match self.coerce(l.clone(), rt) {
                    Ok(_) => {
                        let ln = self.type_name(lt);
                        let rn = self.type_name(rt);
                        self.error(span, format!("mismatched operand types `{}` and `{}` for `{}`; numeric conversions are explicit (`as`)", ln, rn, op.symbol()));
                        return self.error_expr(span);
                    }
                    Err(_) => {
                        let ln = self.type_name(lt);
                        let rn = self.type_name(rt);
                        self.error(span, format!("mismatched operand types `{}` and `{}` for `{}`", ln, rn, op.symbol()));
                        return self.error_expr(span);
                    }
                }
            }
        };
        let t = self.tys.shallow(l.ty);
        let under = match self.tys.kind(t).clone() {
            TyKind::Distinct(d) => self.distinct_underlying[&d],
            _ => t,
        };
        let is_int = self.tys.is_integer(under);
        let is_float = self.tys.is_float(under);
        let is_bool = matches!(self.tys.kind(self.tys.shallow(under)), TyKind::Bool);
        if op.is_bitwise() && !(is_int || is_bool) {
            let tn = self.type_name(t);
            self.error(span, format!("bitwise `{}` requires integer operands but found `{}`", op.symbol(), tn));
            return self.error_expr(span);
        }
        if op.is_arith() && !(is_int || is_float) {
            let tn = self.type_name(t);
            let mut msg = format!("cannot apply `{}` to values of type `{}`", op.symbol(), tn);
            if matches!(self.tys.kind(t), TyKind::Str | TyKind::Slice(..)) {
                msg.push_str("; strings are joined with `.append(...)` or `format(...)`");
            }
            self.error(span, msg);
            return self.error_expr(span);
        }
        let mode = if is_float {
            if matches!(op, BinOp::AddWrap | BinOp::SubWrap | BinOp::MulWrap | BinOp::AddSat | BinOp::SubSat | BinOp::MulSat) {
                self.error(span, "wrapping and saturating operators apply to integers only");
            }
            ArithMode::Float
        } else if is_bool {
            ArithMode::Plain
        } else {
            match op {
                BinOp::AddWrap | BinOp::SubWrap | BinOp::MulWrap => ArithMode::Wrap,
                BinOp::AddSat | BinOp::SubSat | BinOp::MulSat => ArithMode::Sat,
                BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor => ArithMode::Plain,
                _ => ArithMode::Checked,
            }
        };
        let mut proven = true;
        if mode == ArithMode::Checked {
            // overflow: provable when operand ranges fit the result type
            let tmp = TExpr { kind: TExprKind::Binary { op, lhs: Box::new(l.clone()), rhs: Box::new(r.clone()), mode, proven: false }, ty: t, span };
            let fits = match (self.expr_range_raw(&tmp), self.type_range(under)) {
                (Some((lo, hi)), Some((tlo, thi))) => lo >= tlo && hi <= thi,
                _ => false,
            };
            let div_ok = match op {
                BinOp::Div | BinOp::Rem => matches!(self.expr_range(&r), Some((lo, hi)) if lo > 0 || hi < 0),
                _ => true,
            };
            proven = fits && div_ok;
            if !proven {
                let why = match op {
                    BinOp::Div | BinOp::Rem => "division may divide by zero or overflow",
                    _ => "arithmetic may overflow (use `+%` to wrap or `+|` to saturate)",
                };
                self.add_effect(Effects::PANICS, span, why);
            }
        }
        let res = self.tys.resolve(t, false);
        self.mk(TExprKind::Binary { op, lhs: Box::new(l), rhs: Box::new(r), mode, proven }, res, span)
    }

    /// Like expr_range but does not clamp a checked operation to its type (used to decide overflow).
    fn expr_range_raw(&self, e: &TExpr) -> Option<(i128, i128)> {
        if let TExprKind::Binary { op, lhs, rhs, .. } = &e.kind {
            let a = self.expr_range(lhs)?;
            let b = self.expr_range(rhs)?;
            return Some(match op {
                BinOp::Add => (a.0.checked_add(b.0)?, a.1.checked_add(b.1)?),
                BinOp::Sub => (a.0.checked_sub(b.1)?, a.1.checked_sub(b.0)?),
                BinOp::Mul => {
                    let c = [a.0.checked_mul(b.0)?, a.0.checked_mul(b.1)?, a.1.checked_mul(b.0)?, a.1.checked_mul(b.1)?];
                    (*c.iter().min().unwrap(), *c.iter().max().unwrap())
                }
                BinOp::Div if b.0 > 0 => (a.0.min(0), a.1.max(0)),
                BinOp::Rem if b.0 > 0 => (a.0.min(0), (b.1 - 1).min(a.1.max(0))),
                _ => return None,
            });
        }
        self.expr_range(e)
    }

    fn check_comparison(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, span: Span) -> TExpr {
        let bt = self.tys.bool();
        let l = self.check_expr(lhs, None);
        let lt = self.tys.shallow(l.ty);
        let r = self.check_expr(rhs, Some(lt));
        let (l, r) = match self.coerce(r, lt) {
            Ok(r) => (l, r),
            Err(r) => {
                let rt = r.ty;
                match self.coerce(l, rt) {
                    Ok(l) => (l, r),
                    Err(l) => {
                        let ln = self.type_name(l.ty);
                        let rn = self.type_name(r.ty);
                        self.error(span, format!("cannot compare `{}` with `{}`", ln, rn));
                        return self.error_expr(span);
                    }
                }
            }
        };
        let t = self.tys.shallow(l.ty);
        let under = match self.tys.kind(t).clone() {
            TyKind::Distinct(d) => self.tys.shallow(self.distinct_underlying[&d]),
            _ => t,
        };
        let k = self.tys.kind(under).clone();
        let ordered = matches!(op, BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge);
        match k {
            TyKind::Int(_) | TyKind::Float(_) | TyKind::Char | TyKind::Infer(_) => {}
            TyKind::Bool | TyKind::Ptr(..) | TyKind::ErrorSet(_) if !ordered => {}
            TyKind::Slice(_, e) => {
                // content comparison of byte slices (== and !=), or lexicographic for u8
                let is_u8 = matches!(self.tys.kind(self.tys.shallow(e)), TyKind::Int(IntTy::U8));
                if !is_u8 && ordered {
                    let tn = self.type_name(t);
                    self.error(span, format!("cannot order values of type `{}`", tn));
                }
                if !is_u8 && !self.type_satisfies_trait(e, "Eq", 0) {
                    let tn = self.type_name(t);
                    self.error(span, format!("cannot compare slices of `{}`; the element type must derive Eq", tn));
                }
                let cmp = self.mk(TExprKind::Builtin { op: Builtin::SliceEq, args: vec![l, r], tys: vec![e] }, bt, span);
                return self.finish_compare(op, cmp, span);
            }
            TyKind::Str => {
                let u8t = self.tys.u8();
                let sl = self.tys.slice(false, u8t);
                let l = TExpr { kind: TExprKind::StrToSlice(Box::new(l)), ty: sl, span };
                let r = TExpr { kind: TExprKind::StrToSlice(Box::new(r)), ty: sl, span };
                let cmp = self.mk(TExprKind::Builtin { op: Builtin::SliceEq, args: vec![l, r], tys: vec![u8t] }, bt, span);
                return self.finish_compare(op, cmp, span);
            }
            TyKind::Enum(d, _) => {
                let all_unit = self.enums[d as usize].variants.iter().all(|v| matches!(v.payload, VariantPayloadDef::Unit));
                let derived = self.enums[d as usize].derives.iter().any(|x| x == "Eq" || x == "Ord");
                if !(all_unit || derived) {
                    let dn = self.enums[d as usize].name.clone();
                    self.error(span, format!("enum `{}` has payloads; add `derive(Eq)` to compare it, or use `match`", dn));
                }
                if ordered && !self.enums[d as usize].derives.iter().any(|x| x == "Ord") {
                    let dn = self.enums[d as usize].name.clone();
                    self.error(span, format!("enum `{}` is not ordered; add `derive(Ord)`", dn));
                }
            }
            TyKind::Struct(d, _) => {
                let want = if ordered { "Ord" } else { "Eq" };
                if self.structs[d as usize].kind == StructKind::RefClass && !ordered {
                    // reference identity
                    let cmp = self.mk(TExprKind::Builtin { op: Builtin::RefEq, args: vec![l, r], tys: vec![] }, bt, span);
                    return self.finish_compare(op, cmp, span);
                }
                if !self.type_satisfies_trait(t, want, 0) {
                    let dn = self.structs[d as usize].name.clone();
                    self.error(span, format!("`{}` cannot be compared with `{}`; add `derive({})` to the struct", dn, op.symbol(), want));
                }
            }
            TyKind::Opt(inner) if !ordered => {
                // comparing with null
                if matches!(r.kind, TExprKind::OptNull) || matches!(l.kind, TExprKind::OptNull) {
                    let _ = inner;
                } else {
                    self.error(span, "compare optionals against `null`, or unwrap them first");
                }
            }
            _ => {
                let tn = self.type_name(t);
                self.error(span, format!("cannot compare values of type `{}` with `{}`", tn, op.symbol()));
            }
        }
        self.mk(TExprKind::Binary { op, lhs: Box::new(l), rhs: Box::new(r), mode: ArithMode::Plain, proven: true }, bt, span)
    }

    fn finish_compare(&mut self, op: BinOp, cmp: TExpr, span: Span) -> TExpr {
        let bt = self.tys.bool();
        match op {
            BinOp::Eq => cmp,
            BinOp::Ne => self.mk(TExprKind::Unary { op: UnOp::Not, expr: Box::new(cmp), mode: ArithMode::Plain }, bt, span),
            _ => {
                // ordered comparison of byte slices: SliceEq builtin with tys[1] marker is replaced by a compare builtin
                if let TExprKind::Builtin { args, tys, .. } = cmp.kind {
                    let it = self.tys.int(IntTy::I32);
                    let c = self.mk(TExprKind::Builtin { op: Builtin::SliceFind, args, tys: vec![tys[0], it] }, it, span);
                    // SliceFind with two tys acts as memcmp-style compare (documented in codegen)
                    let zero = self.mk(TExprKind::Int(0), it, span);
                    return self.mk(TExprKind::Binary { op, lhs: Box::new(c), rhs: Box::new(zero), mode: ArithMode::Plain, proven: true }, bt, span);
                }
                cmp
            }
        }
    }

    fn check_pipe(&mut self, lhs: &Expr, rhs: &Expr, expected: Option<TyId>, span: Span) -> TExpr {
        // x |> f(a)  ==  f(x, a)
        match rhs {
            Expr::Call { callee, args, .. } => {
                let mut new_args = vec![lhs.clone()];
                new_args.extend(args.iter().cloned());
                self.check_call(callee, &new_args, expected, span)
            }
            Expr::MethodCall { receiver, method, args, .. } => {
                let mut new_args = vec![lhs.clone()];
                new_args.extend(args.iter().cloned());
                self.check_method_call(receiver, method, &new_args, expected, span)
            }
            Expr::Ident { .. } | Expr::Field { .. } => self.check_call(rhs, &[lhs.clone()], expected, span),
            _ => {
                self.error(rhs.span(), "the right side of `|>` must be a call, e.g. `x |> f(a)`");
                self.error_expr(span)
            }
        }
    }

    fn check_try(&mut self, expr: &Expr, span: Span) -> TExpr {
        let ret = self.cur().ret;
        let ret_r = self.tys.shallow(ret);
        let fn_set = match self.tys.kind(ret_r).clone() {
            TyKind::ErrUnion(s, _) => s,
            _ => {
                let tn = self.type_name(ret);
                self.error_note(span, "`try` is only valid in a function that returns an error union", None, format!("this function returns `{}`; change it to `!{}`", tn, tn));
                let _ = self.check_expr(expr, None);
                return self.error_expr(span);
            }
        };
        // expected: an error union of unknown payload
        let inner = self.check_expr(expr, None);
        let it = self.tys.shallow(inner.ty);
        match self.tys.kind(it).clone() {
            TyKind::ErrUnion(set, payload) => {
                if let (Some(fs), Some(es)) = (fn_set, set) {
                    if fs != es {
                        let fsn = self.error_sets[fs as usize].name.clone();
                        let fs_ids = self.error_sets[fs as usize].ids.clone();
                        let es_ids = self.error_sets[es as usize].ids.clone();
                        let missing: Vec<String> = es_ids.iter().filter(|i| !fs_ids.contains(i)).map(|i| self.error_names[(*i - 1) as usize].clone()).collect();
                        if !missing.is_empty() {
                            self.error(span, format!("`try` would propagate errors not in `{}`: {}", fsn, missing.join(", ")));
                        }
                    }
                }
                self.mk(TExprKind::Try(Box::new(inner)), payload, span)
            }
            _ => {
                let tn = self.type_name(it);
                self.error(span, format!("`try` requires an error union but found `{}`", tn));
                inner
            }
        }
    }

    fn check_catch(&mut self, expr: &Expr, binding: Option<&str>, handler: &Expr, expected: Option<TyId>, span: Span) -> TExpr {
        let inner = self.check_expr(expr, None);
        let it = self.tys.shallow(inner.ty);
        let (set, payload) = match self.tys.kind(it).clone() {
            TyKind::ErrUnion(s, p) => (s, p),
            _ => {
                let tn = self.type_name(it);
                self.error(span, format!("`catch` requires an error union but found `{}`", tn));
                return self.error_expr(span);
            }
        };
        let _ = expected;
        self.push_scope();
        let err_local = binding.map(|b| {
            let et = self.tys.intern(TyKind::ErrorSet(set));
            self.declare_local(b, et, false, span)
        });
        let moved_before = self.moved_snapshot();
        let h = self.check_expr(handler, Some(payload));
        if matches!(self.tys.kind(self.tys.shallow(h.ty)), TyKind::Never) {
            self.moved_restore(&moved_before);
        }
        let h = self.coerce_or_error(h, payload, "catch handler");
        self.pop_scope();
        self.mk(TExprKind::Catch { expr: Box::new(inner), err_local, handler: Box::new(h) }, payload, span)
    }

    fn check_orelse(&mut self, expr: &Expr, default: &Expr, expected: Option<TyId>, span: Span) -> TExpr {
        let hint = expected.map(|t| self.tys.opt(t));
        let inner = self.check_expr(expr, hint);
        let it = self.tys.shallow(inner.ty);
        let payload = match self.tys.kind(it).clone() {
            TyKind::Opt(p) => p,
            _ => {
                let tn = self.type_name(it);
                self.error(span, format!("`orelse` requires an optional but found `{}`", tn));
                return self.error_expr(span);
            }
        };
        // a default that diverges (`orelse return x`) moves nothing for the
        // code after it, the same as a diverging `if` branch
        let moved_before = self.moved_snapshot();
        let d = self.check_expr(default, Some(payload));
        if matches!(self.tys.kind(self.tys.shallow(d.ty)), TyKind::Never) {
            self.moved_restore(&moved_before);
        }
        let d = self.coerce_or_error(d, payload, "orelse default");
        let p = self.tys.resolve(payload, false);
        self.mk(TExprKind::OrElse { expr: Box::new(inner), default: Box::new(d) }, p, span)
    }

    fn check_if(&mut self, cond: &Expr, then: &Block, els: Option<&Expr>, expected: Option<TyId>, span: Span) -> TExpr {
        let bt = self.tys.bool();
        let c = self.check_expr(cond, Some(bt));
        let c = self.coerce_or_error(c, bt, "if condition");
        // guard-based range narrowing: `if (x < N)` / `if (x <= N)` / `if (x >= N)`
        let narrowing = self.guard_narrowing(&c);
        self.push_scope();
        for (l, r) in &narrowing {
            self.cur().ranges.insert(*l, *r);
        }
        let moved_before = self.moved_snapshot();
        let then_expected = if els.is_some() { expected } else { Some(self.tys.void()) };
        let tb = self.check_block(then, then_expected, None);
        self.pop_scope();
        let moved_then = self.moved_snapshot();
        let then_diverges = matches!(self.tys.kind(self.tys.shallow(tb.ty)), TyKind::Never);
        if then_diverges && els.is_none() {
            // `if (c) return x` moved nothing for the code after it
            self.moved_restore(&moved_before);
        }
        let then_ty = tb.ty;
        match els {
            None => {
                let void = self.tys.void();
                if !matches!(self.tys.kind(self.tys.shallow(then_ty)), TyKind::Void | TyKind::Never) {
                    let tn = self.type_name(then_ty);
                    self.error(tb.span, format!("an `if` without `else` produces no value, but this block has type `{}`; discard the value with `_ =`", tn));
                }
                self.mk(TExprKind::If { cond: Box::new(c), then: tb, els: None }, void, span)
            }
            Some(e) => {
                // the else branch starts from the moves before the `if`
                self.moved_restore(&moved_before);
                let hint = if matches!(self.tys.kind(self.tys.shallow(then_ty)), TyKind::Never) { expected } else { Some(then_ty) };
                let eb = match e {
                    Expr::Block(b) => self.check_block(b, hint, None),
                    other => {
                        let te = self.check_expr(other, hint);
                        let sp = te.span;
                        let ty = te.ty;
                        TBlock { stmts: vec![], tail: Some(Box::new(te)), label: None, ty, span: sp }
                    }
                };
                let else_diverges = matches!(self.tys.kind(self.tys.shallow(eb.ty)), TyKind::Never);
                let moved_else = self.moved_snapshot();
                self.moved_restore(&moved_before);
                if !then_diverges {
                    self.moved_merge(&moved_then);
                }
                if !else_diverges {
                    self.moved_merge(&moved_else);
                }
                let (tb, eb, ty) = self.join_branches(tb, eb, expected);
                self.mk(TExprKind::If { cond: Box::new(c), then: tb, els: Some(eb) }, ty, span)
            }
        }
    }

    /// Unify two branch blocks to a common type, coercing tails as needed.
    pub fn join_branches(&mut self, mut a: TBlock, mut b: TBlock, expected: Option<TyId>) -> (TBlock, TBlock, TyId) {
        let at = self.tys.shallow(a.ty);
        let bt = self.tys.shallow(b.ty);
        let a_never = matches!(self.tys.kind(at), TyKind::Never);
        let b_never = matches!(self.tys.kind(bt), TyKind::Never);
        let target = if a_never && b_never {
            self.tys.never()
        } else if a_never {
            expected.filter(|&t| self.coerce_tail_ok(&b, t)).unwrap_or(bt)
        } else if b_never {
            expected.filter(|&t| self.coerce_tail_ok(&a, t)).unwrap_or(at)
        } else {
            // try expected first, then a's type, then b's
            let mut t = None;
            if let Some(e) = expected {
                if self.coerce_tail_ok(&a, e) && self.coerce_tail_ok(&b, e) {
                    t = Some(e);
                }
            }
            match t {
                Some(t) => t,
                None => {
                    if self.coerce_tail_ok(&b, at) {
                        at
                    } else if self.coerce_tail_ok(&a, bt) {
                        bt
                    } else {
                        let an = self.type_name(at);
                        let bn = self.type_name(bt);
                        self.error(b.span, format!("`if` branches have incompatible types: `{}` and `{}`", an, bn));
                        at
                    }
                }
            }
        };
        self.coerce_block_tail(&mut a, target);
        self.coerce_block_tail(&mut b, target);
        (a, b, target)
    }

    fn coerce_tail_ok(&mut self, b: &TBlock, t: TyId) -> bool {
        match &b.tail {
            Some(tail) => {
                // probe without committing diagnostics
                let saved = self.diags.len();
                let ok = self.coerce(*tail.clone(), t).is_ok();
                self.diags.truncate(saved);
                ok
            }
            None => matches!(self.tys.kind(self.tys.shallow(b.ty)), TyKind::Never | TyKind::Void) && (matches!(self.tys.kind(self.tys.shallow(t)), TyKind::Void) || self.block_diverges(b)),
        }
    }

    pub fn coerce_block_tail(&mut self, b: &mut TBlock, t: TyId) {
        if let Some(tail) = b.tail.take() {
            let c = self.coerce_or_error(*tail, t, "block value");
            b.tail = Some(Box::new(c));
        }
        b.ty = t;
    }

    /// Extract range facts from a boolean guard on a local.
    fn guard_narrowing(&self, c: &TExpr) -> Vec<(LocalId, (i128, i128))> {
        let mut out = Vec::new();
        if let TExprKind::Binary { op, lhs, rhs, .. } = &c.kind {
            if let (TExprKind::Local(l), Some((rlo, rhi))) = (&lhs.kind, self.expr_range(rhs)) {
                let cur = self.expr_range(lhs).unwrap_or((i128::MIN / 2, i128::MAX / 2));
                let r = match op {
                    BinOp::Lt => Some((cur.0, cur.1.min(rhi - 1))),
                    BinOp::Le => Some((cur.0, cur.1.min(rhi))),
                    BinOp::Gt => Some((cur.0.max(rlo + 1), cur.1)),
                    BinOp::Ge => Some((cur.0.max(rlo), cur.1)),
                    BinOp::Eq => Some((cur.0.max(rlo), cur.1.min(rhi))),
                    _ => None,
                };
                if let Some(r) = r {
                    if r.0 <= r.1 {
                        out.push((*l, r));
                    }
                }
            }
        }
        if let TExprKind::Logical { and: true, lhs, rhs } = &c.kind {
            out.extend(self.guard_narrowing(lhs));
            out.extend(self.guard_narrowing(rhs));
        }
        out
    }

    fn check_if_capture(&mut self, cond: &Expr, binding: &str, then: &Block, els: Option<&Expr>, expected: Option<TyId>, span: Span) -> TExpr {
        let c = self.check_expr(cond, None);
        let ct = self.tys.shallow(c.ty);
        let payload = match self.tys.kind(ct).clone() {
            TyKind::Opt(p) => p,
            _ => {
                let tn = self.type_name(ct);
                self.error(cond.span(), format!("`if (x) |v|` requires an optional but found `{}`", tn));
                return self.error_expr(span);
            }
        };
        self.push_scope();
        let local = self.declare_local(binding, payload, false, span);
        let moved_before = self.moved_snapshot();
        let then_expected = if els.is_some() { expected } else { Some(self.tys.void()) };
        let tb = self.check_block(then, then_expected, None);
        self.pop_scope();
        let moved_then = self.moved_snapshot();
        let then_diverges = matches!(self.tys.kind(self.tys.shallow(tb.ty)), TyKind::Never);
        if then_diverges && els.is_none() {
            // `if (c) return x` moved nothing for the code after it
            self.moved_restore(&moved_before);
        }
        let then_ty = tb.ty;
        match els {
            None => {
                let void = self.tys.void();
                if !matches!(self.tys.kind(self.tys.shallow(then_ty)), TyKind::Void | TyKind::Never) {
                    self.error(tb.span, "an `if` without `else` produces no value");
                }
                self.mk(TExprKind::IfCapture { cond: Box::new(c), local, then: tb, els: None }, void, span)
            }
            Some(e) => {
                // the else branch starts from the moves before the `if`
                self.moved_restore(&moved_before);
                let hint = if matches!(self.tys.kind(self.tys.shallow(then_ty)), TyKind::Never) { expected } else { Some(then_ty) };
                let eb = match e {
                    Expr::Block(b) => self.check_block(b, hint, None),
                    other => {
                        let te = self.check_expr(other, hint);
                        let sp = te.span;
                        let ty = te.ty;
                        TBlock { stmts: vec![], tail: Some(Box::new(te)), label: None, ty, span: sp }
                    }
                };
                let else_diverges = matches!(self.tys.kind(self.tys.shallow(eb.ty)), TyKind::Never);
                let moved_else = self.moved_snapshot();
                self.moved_restore(&moved_before);
                if !then_diverges {
                    self.moved_merge(&moved_then);
                }
                if !else_diverges {
                    self.moved_merge(&moved_else);
                }
                let (tb, eb, ty) = self.join_branches(tb, eb, expected);
                self.mk(TExprKind::IfCapture { cond: Box::new(c), local, then: tb, els: Some(eb) }, ty, span)
            }
        }
    }

    pub fn check_struct_lit(&mut self, ty: Option<&TypeExpr>, fields: &[(String, Expr, Span)], expected: Option<TyId>, span: Span) -> TExpr {
        let t = match ty {
            Some(te) => {
                // `Enum.Variant{ ... }` struct-payload variant
                if let TypeExpr::Named { path, args, .. } = te {
                    if path.len() == 2 && args.is_empty() {
                        let module = self.cur().module;
                        if let Some(ItemRef::Enum(eid)) = self.lookup_item(module, &path[0]) {
                            let et = self.tys.intern(TyKind::Enum(eid, vec![]));
                            return self.check_variant_struct_lit(et, &path[1], fields, span);
                        }
                    }
                }
                let cur = self.cur.as_ref().unwrap();
                let (g, s, m) = (cur.generics.clone(), cur.self_ty, cur.module);
                self.resolve_type(te, &g, s, m)
            }
            None => match expected {
                Some(t) => {
                    let t = self.tys.shallow(t);
                    match self.tys.kind(t).clone() {
                        TyKind::Opt(inner) | TyKind::ErrUnion(_, inner) => {
                            let v = self.check_struct_lit(None, fields, Some(inner), span);
                            return self.coerce_or_error(v, t, "struct literal");
                        }
                        _ => t,
                    }
                }
                None => {
                    self.error(span, "cannot infer the type of this anonymous struct literal; name the type: `Point{ ... }`");
                    return self.error_expr(span);
                }
            },
        };
        let t = self.tys.shallow(t);
        let d = match self.tys.kind(t).clone() {
            TyKind::Struct(d, _) => d,
            _ => {
                let tn = self.type_name(t);
                if fields.is_empty() && matches!(self.tys.kind(t), TyKind::Void) {
                    return self.mk(TExprKind::Unit, t, span);
                }
                self.error(span, format!("`{}` is not a struct; struct literal syntax does not apply", tn));
                return self.error_expr(span);
            }
        };
        let def = self.structs[d as usize].clone();
        let ftys = self.struct_field_types(t);
        let mut out: Vec<(u32, TExpr)> = Vec::new();
        let mut seen = vec![false; def.fields.len()];
        for (name, value, fspan) in fields {
            let idx = match def.fields.iter().position(|f| &f.name == name) {
                Some(i) => i,
                None => {
                    let dn = def.name.clone();
                    self.error(*fspan, format!("`{}` has no field `{}`", dn, name));
                    continue;
                }
            };
            if seen[idx] {
                self.error(*fspan, format!("field `{}` is given twice", name));
                continue;
            }
            seen[idx] = true;
            let fty = ftys[idx];
            let v = self.check_expr(value, Some(fty));
            let v = self.coerce_or_error(v, fty, &format!("field `{}`", name));
            let v = self.take_ownership(v);
            out.push((idx as u32, v));
        }
        for (i, f) in def.fields.iter().enumerate() {
            if !seen[i] {
                match &f.default {
                    Some(dflt) => {
                        let fty = ftys[i];
                        let v = self.check_expr(dflt, Some(fty));
                        let v = self.coerce_or_error(v, fty, &format!("default of field `{}`", f.name));
                        out.push((i as u32, v));
                    }
                    None => {
                        let dn = def.name.clone();
                        let fname = f.name.clone();
                        self.error(span, format!("missing field `{}` in literal of `{}`", fname, dn));
                    }
                }
            }
        }
        out.sort_by_key(|(i, _)| *i);
        self.note_used(t);
        let lit = match def.kind {
            StructKind::RefClass => {
                self.add_effect(Effects::ALLOCATES, span, "constructing a `ref class` allocates");
                self.add_effect(Effects::REFCOUNTS, span, "a `ref class` value is reference counted");
                self.mk(TExprKind::RefNew { fields: out }, t, span)
            }
            _ => self.mk(TExprKind::StructLit { fields: out }, t, span),
        };
        if def.kind == StructKind::Record && !self.record_new_mode {
            return self.record_check(lit, t, false, span);
        }
        lit
    }

    /// Wrap a record value in its constraint checks (compile-time where possible).
    pub fn record_check(&mut self, lit: TExpr, t: TyId, as_error: bool, span: Span) -> TExpr {
        let d = match self.tys.kind(t).clone() {
            TyKind::Struct(d, _) => d,
            _ => return lit,
        };
        let def = self.structs[d as usize].clone();
        let ftys = self.struct_field_types(t);
        let mut checks = Vec::new();
        let mut all_static_ok = true;
        for (i, f) in def.fields.iter().enumerate() {
            if let Some(c) = &f.constraint {
                self.push_scope();
                let vl = self.declare_local("value", ftys[i], false, f.span);
                let bt = self.tys.bool();
                let ce = self.check_expr(c, Some(bt));
                let ce = self.coerce_or_error(ce, bt, "record constraint");
                self.pop_scope();
                // static evaluation when the field value is a literal
                let field_val = match &lit.kind {
                    TExprKind::StructLit { fields } => fields.iter().find(|(j, _)| *j as usize == i).map(|(_, v)| v.clone()),
                    _ => None,
                };
                let mut static_result = None;
                if let (Some(fv), false) = (field_val, as_error) {
                    if let Some(v) = crate::comptime::eval_const_expr(self, &fv) {
                        static_result = crate::comptime::eval_with_local(self, &ce, vl, v);
                    }
                }
                match static_result {
                    Some(Value::Bool(true)) => {}
                    Some(Value::Bool(false)) => {
                        let dn = def.name.clone();
                        let fname = f.name.clone();
                        let csrc = self.sm.file(f.span.file).map(|fl| fl.text[c.span().start as usize..c.span().end as usize].to_string()).unwrap_or_default();
                        self.error_note(span, format!("record `{}` constraint violated: field `{}` must satisfy `{}`", dn, fname, csrc), Some(f.span), "constraint declared here");
                        all_static_ok = false;
                    }
                    _ => {
                        all_static_ok = false;
                        checks.push((i as u32, ce, vl));
                    }
                }
            }
        }
        if checks.is_empty() {
            let _ = all_static_ok;
            if as_error {
                let et = self.tys.err_union(None, t);
                return self.mk(TExprKind::ErrWrap(Box::new(lit)), et, span);
            }
            return lit;
        }
        if as_error {
            let et = self.tys.err_union(None, t);
            self.mk(TExprKind::RecordCheck { value: Box::new(lit), checks, as_error: true }, et, span)
        } else {
            self.add_effect(Effects::PANICS, span, "a record constraint that cannot be checked at compile time panics at runtime if violated (use `.new(...)` for an error instead)");
            self.mk(TExprKind::RecordCheck { value: Box::new(lit), checks, as_error: false }, t, span)
        }
    }

    fn check_variant_struct_lit(&mut self, et: TyId, variant: &str, fields: &[(String, Expr, Span)], span: Span) -> TExpr {
        let d = match self.tys.kind(et).clone() {
            TyKind::Enum(d, _) => d,
            _ => unreachable!(),
        };
        let def = self.enums[d as usize].clone();
        let vtys = self.enum_variant_types(et);
        let idx = match def.variants.iter().position(|v| v.name == variant) {
            Some(i) => i,
            None => {
                self.error(span, format!("enum `{}` has no variant `{}`", def.name, variant));
                return self.error_expr(span);
            }
        };
        let fdefs = match &def.variants[idx].payload {
            VariantPayloadDef::Struct(fs) => fs.clone(),
            _ => {
                self.error(span, format!("variant `{}.{}` does not have named fields", def.name, variant));
                return self.error_expr(span);
            }
        };
        let mut payload: Vec<Option<TExpr>> = vec![None; fdefs.len()];
        for (name, value, fspan) in fields {
            match fdefs.iter().position(|f| &f.name == name) {
                Some(i) => {
                    let fty = vtys[idx][i];
                    let v = self.check_expr(value, Some(fty));
                    let v = self.coerce_or_error(v, fty, &format!("field `{}`", name));
                    let v = self.take_ownership(v);
                    payload[i] = Some(v);
                }
                None => self.error(*fspan, format!("variant `{}.{}` has no field `{}`", def.name, variant, name)),
            }
        }
        let mut out = Vec::new();
        for (i, p) in payload.into_iter().enumerate() {
            match p {
                Some(v) => out.push(v),
                None => {
                    self.error(span, format!("missing field `{}` in `{}.{}`", fdefs[i].name, def.name, variant));
                    return self.error_expr(span);
                }
            }
        }
        self.note_used(et);
        self.mk(TExprKind::EnumLit { variant: idx as u32, payload: out }, et, span)
    }

    fn check_array_lit(&mut self, elems: &[Expr], expected: Option<TyId>, span: Span) -> TExpr {
        let exp = expected.map(|t| self.tys.shallow(t));
        let (elem_hint, want_slice, want_list) = match exp.map(|t| self.tys.kind(t).clone()) {
            Some(TyKind::Array(n, e)) => {
                if n as usize != elems.len() {
                    self.error(span, format!("expected an array of {} elements but the literal has {}", n, elems.len()));
                }
                (Some(e), false, false)
            }
            Some(TyKind::Slice(_, e)) => (Some(e), true, false),
            Some(TyKind::List(e)) => (Some(e), false, true),
            _ => (None, false, false),
        };
        let elem_ty = elem_hint.unwrap_or_else(|| self.tys.fresh_infer());
        let mut out = Vec::new();
        for e in elems {
            let te = self.check_expr(e, Some(elem_ty));
            let te = self.coerce_or_error(te, elem_ty, "array element");
            let te = self.take_ownership(te);
            out.push(te);
        }
        if elems.is_empty() && elem_hint.is_none() {
            self.error(span, "cannot infer the element type of an empty array literal; annotate it, e.g. `let xs: [0]i32 = []`");
        }
        let at = self.tys.array(elems.len() as u64, elem_ty);
        let lit = self.mk(TExprKind::ArrayLit(out), at, span);
        if want_slice {
            let st = exp.unwrap();
            return self.mk(TExprKind::ArrayToSlice(Box::new(lit)), st, span);
        }
        if want_list {
            let lt = exp.unwrap();
            self.add_effect(Effects::ALLOCATES, span, "building a List allocates");
            let st = self.tys.slice(false, elem_ty);
            let sl = self.mk(TExprKind::ArrayToSlice(Box::new(lit)), st, span);
            return self.mk(TExprKind::Builtin { op: Builtin::ListFromSlice, args: vec![sl], tys: vec![elem_ty] }, lt, span);
        }
        lit
    }

    fn check_tuple_lit(&mut self, elems: &[Expr], expected: Option<TyId>, span: Span) -> TExpr {
        if elems.is_empty() {
            let v = self.tys.void();
            return self.mk(TExprKind::Unit, v, span);
        }
        let hints: Vec<Option<TyId>> = match expected.map(|t| self.tys.kind(self.tys.shallow(t)).clone()) {
            Some(TyKind::Tuple(ts)) if ts.len() == elems.len() => ts.iter().map(|&t| Some(t)).collect(),
            _ => vec![None; elems.len()],
        };
        let mut out = Vec::new();
        let mut tys = Vec::new();
        for (e, h) in elems.iter().zip(hints) {
            let te = self.check_expr(e, h);
            let te = match h {
                Some(t) => self.coerce_or_error(te, t, "tuple element"),
                None => te,
            };
            let te = self.take_ownership(te);
            tys.push(te.ty);
            out.push(te);
        }
        let tt = self.tys.tuple(tys);
        self.mk(TExprKind::TupleLit(out), tt, span)
    }

    fn check_cast(&mut self, expr: &Expr, ty: &TypeExpr, span: Span) -> TExpr {
        let cur = self.cur.as_ref().unwrap();
        let (g, s, m) = (cur.generics.clone(), cur.self_ty, cur.module);
        let target = self.resolve_type(ty, &g, s, m);
        let inner = self.check_expr(expr, None);
        let from = self.tys.resolve(inner.ty, true);
        let to = self.tys.shallow(target);
        let kf = self.tys.kind(from).clone();
        let kt = self.tys.kind(to).clone();
        let unwrap_distinct = |c: &Self, t: TyId| -> TyId {
            match c.tys.kind(t).clone() {
                TyKind::Distinct(d) => c.tys.shallow(c.distinct_underlying[&d]),
                _ => t,
            }
        };
        let uf = unwrap_distinct(self, from);
        let ut = unwrap_distinct(self, to);
        let kuf = self.tys.kind(uf).clone();
        let kut = self.tys.kind(ut).clone();
        // distinct <-> underlying (representation preserving)
        if (matches!(kf, TyKind::Distinct(_)) || matches!(kt, TyKind::Distinct(_))) && uf == ut {
            return self.mk(TExprKind::Cast { expr: Box::new(inner), kind: CastKind::Bits }, to, span);
        }
        let kind = match (&kuf, &kut) {
            (TyKind::Int(a), TyKind::Int(b)) => {
                let fits = a.min() >= b.min() && a.max() <= b.max();
                let proven = fits || matches!(self.expr_range(&inner), Some((lo, hi)) if lo >= b.min() && hi <= b.max());
                if !proven {
                    self.add_effect(Effects::PANICS, span, "narrowing cast panics when the value does not fit (use `@truncate` to wrap)");
                }
                CastKind::IntToInt { checked: !proven }
            }
            (TyKind::Int(_), TyKind::Float(_)) => CastKind::IntToFloat,
            (TyKind::Float(_), TyKind::Int(_)) => {
                self.add_effect(Effects::PANICS, span, "float to integer cast panics when out of range");
                CastKind::FloatToInt
            }
            (TyKind::Float(_), TyKind::Float(_)) => CastKind::FloatToFloat,
            (TyKind::Bool, TyKind::Int(_)) => CastKind::Bits,
            (TyKind::Char, TyKind::Int(b)) => {
                if b.max() < 0x10FFFF {
                    self.add_effect(Effects::PANICS, span, "char to narrow integer cast panics when out of range");
                    CastKind::IntToInt { checked: true }
                } else {
                    CastKind::Bits
                }
            }
            (TyKind::Int(_), TyKind::Char) => {
                self.add_effect(Effects::PANICS, span, "integer to char cast panics for invalid scalar values");
                CastKind::IntToInt { checked: true }
            }
            (TyKind::Enum(d, _), TyKind::Int(_)) => {
                if !self.enums[*d as usize].variants.iter().all(|v| matches!(v.payload, VariantPayloadDef::Unit)) {
                    self.error(span, "only enums without payloads convert to integers");
                }
                CastKind::Bits
            }
            (TyKind::Ptr(..), TyKind::Ptr(..)) => {
                if self.cur().unsafe_depth == 0 {
                    self.error(span, "pointer casts require an `unsafe` block");
                }
                CastKind::PtrToPtr
            }
            (TyKind::Int(_), TyKind::Ptr(..)) | (TyKind::Ptr(..), TyKind::Int(IntTy::Usize)) => {
                if self.cur().unsafe_depth == 0 {
                    self.error(span, "casting between pointers and integers requires an `unsafe` block");
                }
                CastKind::PtrToPtr
            }
            _ => {
                if self.unify(from, to) {
                    return inner;
                }
                let fname = self.type_name(from);
                let tname = self.type_name(to);
                self.error(span, format!("cannot cast `{}` to `{}`", fname, tname));
                return self.error_expr(span);
            }
        };
        self.mk(TExprKind::Cast { expr: Box::new(inner), kind }, to, span)
    }

    // ----- calls ------------------------------------------------------------------

    pub fn check_call(&mut self, callee: &Expr, args: &[Expr], expected: Option<TyId>, span: Span) -> TExpr {
        // builtin free functions
        if let Expr::Ident { name, .. } = callee {
            if self.cur().lookup(name).is_none() {
                let module = self.cur().module;
                if self.lookup_item(module, name).is_none() {
                    if let Some(te) = self.check_builtin_fn(name, args, expected, span) {
                        return te;
                    }
                    if self.cur().generics.get(name).is_none() && !matches!(name.as_str(), "List" | "Map" | "String" | "Allocator") {
                        self.error(span, format!("cannot find function `{}`", name));
                        for a in args {
                            let _ = self.check_expr(a, None);
                        }
                        return self.error_expr(span);
                    }
                }
                if let Some(ItemRef::Fn(fid)) = self.lookup_item(module, name) {
                    return self.call_fn_def(fid, None, args, expected, span);
                }
            }
        }
        // `pkg.func(...)` or `Type.assoc(...)` or `Enum.Variant(...)`
        if let Expr::Field { base, name, .. } = callee {
            let b = self.check_expr(base, None);
            let bt = self.tys.shallow(b.ty);
            match self.tys.kind(bt).clone() {
                TyKind::Type => {
                    if let TExprKind::TypeVal(t) = b.kind {
                        return self.check_static_call(t, name, args, expected, span);
                    }
                }
                TyKind::Namespace(ns) => return self.check_namespace_call(&ns, name, args, expected, span),
                _ => {
                    // calling a field of function type
                    let f = self.field_access(b, name, span);
                    return self.call_value(f, args, span);
                }
            }
        }
        // generic type instantiation used as a value: `List(i32)` -> type value
        if let Expr::Ident { name, .. } = callee {
            let cur_mod = self.cur().module;
            if matches!(name.as_str(), "List" | "Map" | "String") || matches!(self.lookup_item(cur_mod, name), Some(ItemRef::Struct(_)) | Some(ItemRef::Enum(_))) {
                let mut targs = Vec::new();
                for a in args {
                    match a {
                        Expr::TypeVal { ty, .. } => targs.push(ty.clone()),
                        Expr::Ident { name: n, span: s } => targs.push(TypeExpr::Named { path: vec![n.clone()], args: vec![], span: *s }),
                        Expr::Call { .. } | Expr::Field { .. } | Expr::TupleLit { .. } => match expr_to_type_expr(a) {
                            Some(te) => targs.push(te),
                            None => {
                                self.error(a.span(), "expected a type argument");
                                return self.error_expr(span);
                            }
                        },
                        _ => {
                            self.error(a.span(), format!("`{}` takes type arguments, e.g. `{}(i32)`", name, name));
                            return self.error_expr(span);
                        }
                    }
                }
                let te = TypeExpr::Named { path: vec![name.clone()], args: targs, span };
                let cur = self.cur.as_ref().unwrap();
                let (g, s, m) = (cur.generics.clone(), cur.self_ty, cur.module);
                let t = self.resolve_type(&te, &g, s, m);
                let tt = self.tys.type_ty();
                return self.mk(TExprKind::TypeVal(t), tt, span);
            }
        }
        let f = self.check_expr(callee, None);
        // calling a type value: struct construction from a tuple? not supported
        let ft = self.tys.shallow(f.ty);
        if matches!(self.tys.kind(ft), TyKind::Type) {
            if let TExprKind::TypeVal(t) = f.kind {
                let tn = self.type_name(t);
                self.error(span, format!("`{}` is a type; construct it with `{}{{ .field = value }}`", tn, tn));
                return self.error_expr(span);
            }
        }
        self.call_value(f, args, span)
    }

    /// Call a value of function type.
    pub fn call_value(&mut self, f: TExpr, args: &[Expr], span: Span) -> TExpr {
        let ft = self.tys.shallow(f.ty);
        let (params, ret, eff) = match self.tys.kind(ft).clone() {
            TyKind::Fn(p, r, e) => (p, r, Effects(e)),
            _ => {
                let tn = self.type_name(ft);
                self.error(span, format!("cannot call a value of type `{}`", tn));
                for a in args {
                    let _ = self.check_expr(a, None);
                }
                return self.error_expr(span);
            }
        };
        if args.len() != params.len() {
            self.error(span, format!("this function takes {} argument(s) but {} were given", params.len(), args.len()));
            return self.error_expr(span);
        }
        let mut targs = Vec::new();
        for (a, &p) in args.iter().zip(params.iter()) {
            let te = self.check_expr(a, Some(p));
            let te = self.coerce_or_error(te, p, "argument");
            targs.push(te);
        }
        // direct calls to known functions avoid the indirection
        if let TExprKind::FnToFat(inst) = f.kind {
            self.cur().callees.push((inst, span));
            return self.mk(TExprKind::Call { inst, args: targs }, ret, span);
        }
        // calling through a function value: acquire its declared effects
        if !eff.is_empty() {
            self.add_effect(eff, span, "calling through a function value acquires the effects its type permits");
        }
        self.mk(TExprKind::CallPtr { callee: Box::new(f), args: targs }, ret, span)
    }

    /// Call a function definition, inferring generic arguments from the call.
    pub fn call_fn_def(&mut self, fid: FnDefId, self_arg: Option<TExpr>, args: &[Expr], expected: Option<TyId>, span: Span) -> TExpr {
        let def = self.fns[fid as usize].clone();
        let mut generics: HashMap<String, TyId> = HashMap::new();
        let mut self_ty = None;
        // impl type params from the receiver type
        let mut impl_targs: Vec<TyId> = Vec::new();
        if let Some(impl_id) = def.impl_id {
            let im = self.impls[impl_id as usize].clone();
            if let Some(sa) = &self_arg {
                // receiver type (peel pointer)
                let mut rt = self.tys.resolve(sa.ty, true);
                if let TyKind::Ptr(_, inner) = self.tys.kind(rt).clone() {
                    rt = inner;
                }
                if let Some(b) = self.match_impl_target(&im, rt) {
                    for p in &im.type_params {
                        let t = b.get(p).copied().unwrap_or_else(|| self.tys.fresh_infer());
                        generics.insert(p.clone(), t);
                        impl_targs.push(t);
                    }
                }
            } else {
                for p in &im.type_params {
                    let t = self.tys.fresh_infer();
                    generics.insert(p.clone(), t);
                    impl_targs.push(t);
                }
            }
            self_ty = Some(self.resolve_type(&im.target, &generics, None, im.module));
        }
        // explicit vs inferred type params
        let n_type_params = def.type_params.len();
        let value_params: Vec<&Param> = def.decl.params.iter().filter(|p| !(p.comptime && def.type_params.contains(&p.name))).collect();
        let n_self = if self_arg.is_some() { 1 } else { 0 };
        let explicit = args.len() == value_params.len() + n_type_params - n_self && n_type_params > 0 && args.iter().take(n_type_params).all(|a| self.is_type_arg(a));
        let mut arg_iter = args.iter();
        let mut fn_targs: Vec<TyId> = Vec::new();
        if explicit {
            for p in &def.type_params {
                let a = arg_iter.next().unwrap();
                let te = self.check_expr(a, None);
                let t = match te.kind {
                    TExprKind::TypeVal(t) => t,
                    _ => {
                        self.error(a.span(), "expected a type argument");
                        self.tys.void()
                    }
                };
                generics.insert(p.clone(), t);
                fn_targs.push(t);
            }
        } else {
            for p in &def.type_params {
                let t = self.tys.fresh_infer();
                generics.insert(p.clone(), t);
                fn_targs.push(t);
            }
        }
        let rest: Vec<&Expr> = arg_iter.collect();
        let variadic = def.decl.variadic;
        if variadic && rest.len() + n_self < value_params.len() || !variadic && rest.len() + n_self != value_params.len() {
            let n = def.decl.name.clone();
            let mut msg = format!("`{}` takes {} argument(s) but {} were given", n, value_params.len() - n_self, rest.len());
            if n_type_params > 0 && !explicit {
                msg.push_str(&format!(" (type parameters {} are inferred from the arguments)", def.type_params.join(", ")));
            }
            self.error(span, msg);
            for a in rest {
                let _ = self.check_expr(a, None);
            }
            return self.error_expr(span);
        }
        // check args against param types with generics as inference vars
        let mut targs_e: Vec<TExpr> = Vec::new();
        let mut params_iter = value_params.iter();
        if let Some(sa) = self_arg {
            let p = params_iter.next().unwrap();
            let pt = self.resolve_type(&p.ty, &generics, self_ty, def.module);
            let sa = self.adjust_receiver(sa, pt, span);
            targs_e.push(sa);
        }
        // return type hint helps infer generics from the expected type
        if let (Some(exp), Some(r)) = (expected, &def.decl.ret) {
            let rt = self.resolve_type(r, &generics, self_ty, def.module);
            let saved = self.diags.len();
            let _ = self.unify(rt, exp);
            self.diags.truncate(saved);
        }
        let n_fixed = value_params.len() - n_self;
        for (a, p) in rest.iter().zip(params_iter) {
            let pt = self.resolve_type(&p.ty, &generics, self_ty, def.module);
            let te = self.check_expr(a, Some(pt));
            let te = self.coerce_or_error(te, pt, &format!("argument `{}`", p.name));
            // an `own` parameter takes the argument: the caller's local is moved
            let te = if p.owned { self.take_ownership(te) } else { te };
            targs_e.push(te);
        }
        // extra arguments of a C variadic: untyped literals take C's default promotions
        for a in rest.iter().skip(n_fixed) {
            let te = self.check_expr(a, None);
            let t = self.tys.shallow(te.ty);
            let te = match self.tys.kind(t).clone() {
                TyKind::Infer(_) if self.tys.is_integer(t) => {
                    let i32t = self.tys.int(IntTy::I32);
                    self.coerce_or_error(te, i32t, "variadic argument")
                }
                TyKind::Infer(_) if self.tys.is_float(t) => {
                    let f64t = self.tys.float(FloatTy::F64);
                    self.coerce_or_error(te, f64t, "variadic argument")
                }
                TyKind::Int(_) | TyKind::Float(_) | TyKind::Bool | TyKind::Char | TyKind::Ptr(..) => te,
                _ => {
                    let tn = self.type_name(t);
                    self.error(a.span(), format!("a value of type `{}` cannot be passed to a C variadic parameter; use scalars or pointers", tn));
                    te
                }
            };
            targs_e.push(te);
        }
        // all generics must be resolved
        let mut all_targs: Vec<TyId> = Vec::new();
        for t in impl_targs.iter().chain(fn_targs.iter()) {
            let r = self.tys.resolve(*t, true);
            if self.tys.contains_infer(r) {
                let n = def.decl.name.clone();
                self.error(span, format!("cannot infer all type parameters of `{}`; pass them explicitly", n));
                return self.error_expr(span);
            }
            all_targs.push(r);
        }
        // where-clause bounds
        for w in &def.decl.wheres {
            if let Some(&t) = generics.get(&w.param) {
                let t = self.tys.resolve(t, true);
                for b in &w.bounds {
                    if !self.type_satisfies_trait(t, b, def.module) {
                        let tn = self.type_name(t);
                        let n = def.decl.name.clone();
                        self.error_note(span, format!("`{}` requires `{}: {}`, but `{}` does not implement `{}`", n, w.param, b, tn, b), Some(w.span), "bound declared here");
                    }
                }
            }
        }
        let inst = self.instantiate(fid, all_targs, span);
        let (_, ret) = self.fn_sig(inst);
        self.cur().callees.push((inst, span));
        if def.decl.extern_c {
            if self.cur().unsafe_depth == 0 {
                let n = def.decl.name.clone();
                self.error(span, format!("calling the foreign function `{}` requires an `unsafe` block", n));
            }
            self.add_effect(Effects::FFI, span, "call to a foreign function");
        }
        let _ = expected;
        self.mk(TExprKind::Call { inst, args: targs_e }, ret, span)
    }

    fn is_type_arg(&mut self, a: &Expr) -> bool {
        match a {
            Expr::TypeVal { .. } => true,
            Expr::Ident { name, .. } => {
                if self.cur().lookup(name).is_some() {
                    return false;
                }
                if self.cur().generics.contains_key(name) {
                    return true;
                }
                let m = self.cur().module;
                matches!(self.lookup_item(m, name), Some(ItemRef::Struct(_)) | Some(ItemRef::Enum(_)) | Some(ItemRef::Alias(_))) || matches!(name.as_str(), "String")
            }
            Expr::Call { callee, .. } => {
                matches!(&**callee, Expr::Ident { name, .. } if matches!(name.as_str(), "List" | "Map")) || {
                    if let Expr::Ident { name, .. } = &**callee {
                        let m = self.cur().module;
                        matches!(self.lookup_item(m, name), Some(ItemRef::Struct(_)) | Some(ItemRef::Enum(_)))
                    } else {
                        false
                    }
                }
            }
            _ => false,
        }
    }

    /// Make the receiver expression match the declared `self` parameter type:
    /// take `&`/`&mut` for pointer receivers, deref for by-value receivers.
    pub fn adjust_receiver(&mut self, recv: TExpr, param_ty: TyId, span: Span) -> TExpr {
        let pt = self.tys.shallow(param_ty);
        let rt = self.tys.shallow(recv.ty);
        match (self.tys.kind(pt).clone(), self.tys.kind(rt).clone()) {
            (TyKind::Ptr(m, _), TyKind::Ptr(rm, _)) => {
                if m && !rm {
                    self.error(span, "this method mutates its receiver, but the receiver is an immutable pointer");
                }
                recv
            }
            (TyKind::Ptr(m, _), _) => {
                if m && self.is_place(&recv) && !self.place_mutable(&recv) {
                    let what = match &recv.kind {
                        TExprKind::Local(l) => format!("`{}`", self.cur.as_ref().unwrap().locals[*l as usize].name),
                        _ => "the receiver".to_string(),
                    };
                    self.error_note(span, format!("this method mutates its receiver, but {} is immutable", what), None, "declare it with `var` instead of `let`");
                }
                let rty = recv.ty;
                let p = self.tys.ptr(m, rty);
                TExpr { kind: TExprKind::AddrOf { expr: Box::new(recv), mutable: m }, ty: p, span }
            }
            (_, TyKind::Ptr(_, inner)) => {
                let _ = inner;
                TExpr { kind: TExprKind::Deref(Box::new(recv)), ty: param_ty, span }
            }
            _ => recv,
        }
    }

    // ----- closures --------------------------------------------------------------

    fn check_closure(&mut self, c: &ClosureExpr, expected: Option<TyId>) -> TExpr {
        let exp_sig = match expected.map(|t| self.tys.kind(self.tys.shallow(t)).clone()) {
            Some(TyKind::Fn(ps, r, ef)) => Some((ps, r, ef)),
            _ => None,
        };
        let cur = self.cur.as_ref().unwrap();
        let (g, s, m) = (cur.generics.clone(), cur.self_ty, cur.module);
        // parameter types
        let mut ptys = Vec::new();
        for (i, p) in c.params.iter().enumerate() {
            let t = match &p.ty {
                TypeExpr::Infer { .. } => match &exp_sig {
                    Some((ps, _, _)) if i < ps.len() => ps[i],
                    _ => {
                        self.error(p.span, format!("closure parameter `{}` needs a type annotation", p.name));
                        self.tys.void()
                    }
                },
                te => self.resolve_type(te, &g, s, m),
            };
            ptys.push(t);
        }
        let ret = match &c.ret {
            Some(r) => self.resolve_type(r, &g, s, m),
            None => match &exp_sig {
                Some((_, r, _)) => *r,
                None => self.tys.fresh_infer(),
            },
        };
        if let Some((ps, _, _)) = &exp_sig {
            if ps.len() != ptys.len() {
                self.error(c.span, format!("expected a closure with {} parameter(s) but it has {}", ps.len(), ptys.len()));
            }
        }
        // captures
        let mut captures: Vec<(LocalId, bool, TyId)> = Vec::new();
        let mut cap_list = Vec::new();
        for cap in &c.captures {
            match self.cur().lookup(&cap.name) {
                Some(entry) => {
                    let lt = self.cur.as_ref().unwrap().locals[entry.local as usize].ty;
                    if cap.by_ref && cap.mutable && !self.cur.as_ref().unwrap().locals[entry.local as usize].mutable {
                        self.error(cap.span, format!("cannot capture `{}` by mutable reference; declare it with `var`", cap.name));
                    }
                    if !cap.by_ref {
                        let _lt2 = lt;
                        // by-value capture of a resource type moves it
                        let le = TExpr { kind: TExprKind::Local(entry.local), ty: lt, span: cap.span };
                        let _ = self.take_ownership(le);
                    }
                    captures.push((entry.local, cap.by_ref, lt));
                    cap_list.push((entry.local, cap.by_ref));
                }
                None => self.error(cap.span, format!("cannot capture `{}`: no such local", cap.name)),
            }
        }
        // create the instance
        let id = self.funcs.len() as InstId;
        self.closure_counter += 1;
        let mut locals = Vec::new();
        let mut params = Vec::new();
        for (i, p) in c.params.iter().enumerate() {
            let lid = locals.len() as LocalId;
            locals.push(Local { name: p.name.clone(), ty: ptys[i], mutable: false, span: p.span, is_param: true, owned: false, loop_item: false });
            params.push(lid);
        }
        let env_tys: Vec<(TyId, bool)> = captures.iter().map(|(_, r, t)| (*t, *r)).collect();
        let fname = self.cur().fn_name.clone();
        let f = TFunc {
            name: format!("closure in {}", fname),
            mangled: format!("nx_closure_{}", id),
            def: None,
            targs: vec![],
            params,
            ret,
            locals,
            body: None,
            own_effects: Effects::NONE,
            witnesses: Vec::new(),
            callees: Vec::new(),
            effects: Effects::NONE,
            declared_neg: Effects::NONE,
            declared_pos: Effects::NONE,
            declared_spans: Vec::new(),
            export: None,
            is_extern: false,
            is_variadic: false,
            cimport: false,
            is_pub: false,
            is_test: false,
            test_comptime: false,
            is_closure: true,
            closure_env: None,
            span: c.span,
            module: m,
            moved: HashSet::new(),
            takes_ctx: true,
        };
        self.funcs.push(f);
        self.closure_envs.insert(id, env_tys);
        self.check_closure_body((id, c.clone(), captures, g, s, m));
        let ef = match &exp_sig {
            Some((_, _, ef)) => *ef,
            None => Effects::ALL.0,
        };
        if ef != Effects::ALL.0 {
            self.fn_value_obligations.push((id, Effects::ALL.without(Effects(ef)), c.span));
        }
        let ft = self.tys.intern(TyKind::Fn(ptys, ret, ef));
        self.mk(TExprKind::Closure { inst: id, captures: cap_list }, ft, c.span)
    }

    pub fn check_closure_body(&mut self, item: (InstId, ClosureExpr, Vec<(LocalId, bool, TyId)>, HashMap<String, TyId>, Option<TyId>, u32)) {
        let (inst, c, captures, generics, self_ty, module) = item;
        let f = self.funcs[inst as usize].clone();
        let mut ctx = FnCtx::new(f.ret, module, &f.name);
        ctx.generics = generics;
        ctx.self_ty = self_ty;
        ctx.locals = f.locals.clone();
        for &p in &f.params {
            let name = ctx.locals[p as usize].name.clone();
            ctx.scopes[0].push((name, ScopeEntry { local: p, auto_deref: false }));
        }
        // captured variables become locals: by-value copies, or pointers auto-dereferenced
        for (i, (outer_local, by_ref, ty)) in captures.iter().enumerate() {
            let _ = outer_local;
            let name = c.captures[i].name.clone();
            let lid = ctx.locals.len() as LocalId;
            let mutable = c.captures[i].mutable;
            let lty = if *by_ref { self.tys.ptr(mutable, *ty) } else { *ty };
            ctx.locals.push(Local { name: format!("cap_{}", name), ty: lty, mutable: mutable || !*by_ref, span: c.captures[i].span, is_param: true, owned: false, loop_item: false });
            ctx.scopes[0].push((name, ScopeEntry { local: lid, auto_deref: *by_ref }));
        }
        let saved = self.cur.take();
        self.cur = Some(ctx);
        let ret = f.ret;
        let body_block = match &*c.body {
            Expr::Block(b) => b.clone(),
            other => Block { stmts: vec![], tail: Some(Box::new(other.clone())), label: None, span: other.span() },
        };
        let ret_hint = if self.tys.contains_infer(ret) { None } else { Some(ret) };
        let tb = self.check_block(&body_block, ret_hint, None);
        // infer the return type from the body when it was not annotated
        if self.tys.contains_infer(ret) {
            let bt = tb.ty;
            let void = self.tys.void();
            let target = if tb.tail.is_some() {
                bt
            } else if self.block_diverges(&tb) {
                ret
            } else {
                void
            };
            if !self.tys.contains_infer(target) {
                let _ = self.unify(ret, target);
            }
        }
        let tb = self.finalize_block_as_body(tb, ret);
        let ctx = self.cur.take().unwrap();
        self.cur = saved;
        let func = &mut self.funcs[inst as usize];
        func.locals = ctx.locals;
        func.body = Some(tb);
        func.own_effects = ctx.own_effects;
        func.witnesses = ctx.witnesses;
        func.callees = ctx.callees;
        func.moved = ctx.moved;
        func.ret = self.tys.resolve(ret, true);
    }
}

/// An expression that denotes storage owned by something else: a field, an
/// element, a dereference, or a lookup that returns a view (`map.get`,
/// `list.last`). Moving such a value into a new owner needs `.clone()`.
pub fn is_borrowed_view(e: &TExpr) -> bool {
    match &e.kind {
        TExprKind::Field { .. } | TExprKind::RefField { .. } | TExprKind::TupleField { .. } | TExprKind::Index { .. } | TExprKind::Deref(_) => true,
        TExprKind::Builtin { op: Builtin::MapGet | Builtin::ListLast, .. } => true,
        TExprKind::OrElse { expr, .. } | TExprKind::Unwrap { expr, .. } | TExprKind::Try(expr) => is_borrowed_view(expr),
        // `return xs[i]` into a `?T` or `!T` wraps the element first; the view is still moved
        TExprKind::OptWrap(expr) | TExprKind::ErrWrap(expr) => is_borrowed_view(expr),
        _ => false,
    }
}

fn similar(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let (la, lb) = (a.len(), b.len());
    if la.abs_diff(lb) > 2 {
        return false;
    }
    // simple edit distance
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = if a[i - 1].eq_ignore_ascii_case(&b[j - 1]) { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        prev = cur;
    }
    prev[b.len()] <= 2
}

/// Interpret an expression written in type-argument position as a type.
pub fn expr_to_type_expr(e: &Expr) -> Option<TypeExpr> {
    match e {
        Expr::Ident { name, span } => Some(TypeExpr::Named { path: vec![name.clone()], args: vec![], span: *span }),
        Expr::TypeVal { ty, .. } => Some(ty.clone()),
        Expr::Field { base, name, span } => {
            let mut t = expr_to_type_expr(base)?;
            if let TypeExpr::Named { path, span: s, .. } = &mut t {
                path.push(name.clone());
                *s = *span;
                return Some(t);
            }
            None
        }
        Expr::Call { callee, args, span } => {
            let mut t = expr_to_type_expr(callee)?;
            let targs: Option<Vec<TypeExpr>> = args.iter().map(expr_to_type_expr).collect();
            if let TypeExpr::Named { args: ta, span: s, .. } = &mut t {
                *ta = targs?;
                *s = *span;
                return Some(t);
            }
            None
        }
        // `List((A, B))`: a tuple expression in type-argument position is a tuple type
        Expr::TupleLit { elems, span } => {
            let ts: Option<Vec<TypeExpr>> = elems.iter().map(expr_to_type_expr).collect();
            Some(TypeExpr::Tuple { elems: ts?, span: *span })
        }
        _ => None,
    }
}
