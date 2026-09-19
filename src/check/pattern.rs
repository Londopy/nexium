//! `match` expressions and patterns.

use super::*;
use crate::ast::*;
use crate::tir::*;
use crate::types::*;

impl<'a> Checker<'a> {
    pub fn check_match(&mut self, scrutinee: &Expr, arms: &[MatchArm], expected: Option<TyId>, span: Span) -> TExpr {
        let s = self.check_expr(scrutinee, None);
        let st = self.tys.resolve(s.ty, false);
        let s = TExpr { ty: st, ..s };
        if arms.is_empty() {
            self.error(span, "`match` needs at least one arm");
            return self.error_expr(span);
        }
        let mut tarms: Vec<(TPat, Option<TExpr>, TExpr, Span)> = Vec::new();
        let mut hint = expected;
        let moved_before = self.moved_snapshot();
        let mut moved_arms = Vec::new();
        for arm in arms {
            // each arm starts from the moves before the `match`
            self.moved_restore(&moved_before);
            self.push_scope();
            let pat = self.check_pattern(&arm.pat, st);
            let guard = arm.guard.as_ref().map(|g| {
                let bt = self.tys.bool();
                let ge = self.check_expr(g, Some(bt));
                self.coerce_or_error(ge, bt, "match guard")
            });
            let body = self.check_expr(&arm.body, hint);
            let bt = self.tys.shallow(body.ty);
            if hint.is_none() && !matches!(self.tys.kind(bt), TyKind::Never) {
                hint = Some(bt);
            }
            let body = self.take_ownership(body);
            self.pop_scope();
            tarms.push((pat, guard, body, arm.span));
            // a diverging arm cannot fall through: its moves do not count afterwards
            if !matches!(self.tys.kind(bt), TyKind::Never) {
                moved_arms.push(self.moved_snapshot());
            }
        }
        self.moved_restore(&moved_before);
        for m in &moved_arms {
            self.moved_merge(m);
        }
        // join arm types
        let bodies: Vec<TExpr> = tarms.iter().map(|(_, _, b, _)| b.clone()).collect();
        let (bodies, ty) = self.join_exprs(bodies, expected, span);
        let arms_out: Vec<TArm> = tarms.into_iter().zip(bodies).map(|((pat, guard, _, sp), body)| TArm { pat, guard, body, span: sp }).collect();
        self.check_exhaustive(st, &arms_out, span);
        self.mk(TExprKind::Match { scrutinee: Box::new(s), arms: arms_out }, ty, span)
    }

    /// Find a common type for a set of branch expressions and coerce each to it.
    pub fn join_exprs(&mut self, exprs: Vec<TExpr>, expected: Option<TyId>, span: Span) -> (Vec<TExpr>, TyId) {
        let never = self.tys.never();
        let void = self.tys.void();
        let mut candidates: Vec<TyId> = Vec::new();
        if let Some(e) = expected {
            candidates.push(e);
        }
        for e in &exprs {
            let t = self.tys.shallow(e.ty);
            if !matches!(self.tys.kind(t), TyKind::Never) {
                candidates.push(t);
            }
        }
        if candidates.is_empty() {
            return (exprs, never);
        }
        let mut target = None;
        for &cand in &candidates {
            let saved = self.diags.len();
            let ok = exprs.iter().all(|e| self.coerce(e.clone(), cand).is_ok());
            self.diags.truncate(saved);
            if ok {
                target = Some(cand);
                break;
            }
        }
        let target = match target {
            Some(t) => t,
            None => {
                // void is acceptable when every branch is a statement-like value
                let all_void = exprs.iter().all(|e| matches!(self.tys.kind(self.tys.shallow(e.ty)), TyKind::Void | TyKind::Never));
                if all_void {
                    void
                } else {
                    let names: Vec<String> = exprs.iter().map(|e| self.type_name(e.ty)).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
                    self.error(span, format!("match arms have incompatible types: {}", names.join(", ")));
                    candidates[0]
                }
            }
        };
        let out = exprs.into_iter().map(|e| self.coerce_or_error(e, target, "match arm")).collect();
        (out, target)
    }

    pub fn check_pattern(&mut self, pat: &Pattern, ty: TyId) -> TPat {
        let ty = self.tys.shallow(ty);
        let k = self.tys.kind(ty).clone();
        match pat {
            Pattern::Wildcard { .. } => TPat::Wild,
            Pattern::Binding { name, span } => {
                // a bare name on an optional binds its payload; on an error union, the success value
                match k {
                    TyKind::Opt(inner) => {
                        let l = self.declare_local(name, inner, false, *span);
                        TPat::Some(Box::new(TPat::Bind(l)))
                    }
                    TyKind::ErrUnion(_, inner) => {
                        let l = self.declare_local(name, inner, false, *span);
                        TPat::Ok(Box::new(TPat::Bind(l)))
                    }
                    _ => {
                        let l = self.declare_local(name, ty, false, *span);
                        TPat::Bind(l)
                    }
                }
            }
            Pattern::Lit { value, negative, span } => {
                // peel optionals / error unions
                match k {
                    TyKind::Opt(inner) => return TPat::Some(Box::new(self.check_pattern(pat, inner))),
                    TyKind::ErrUnion(_, inner) => return TPat::Ok(Box::new(self.check_pattern(pat, inner))),
                    _ => {}
                }
                let lit = self.check_lit_pattern(value, *negative, ty, *span);
                lit
            }
            Pattern::Null { span } => match k {
                TyKind::Opt(_) => TPat::Null,
                _ => {
                    let tn = self.type_name(ty);
                    self.error(*span, format!("`null` pattern on a non-optional value of type `{}`", tn));
                    TPat::Wild
                }
            },
            Pattern::Error { name, span } => {
                let id = self.error_id(name);
                match k {
                    TyKind::ErrUnion(set, _) | TyKind::ErrorSet(set) => {
                        if let Some(s) = set {
                            if !self.error_sets[s as usize].ids.contains(&id) {
                                let sn = self.error_sets[s as usize].name.clone();
                                self.error(*span, format!("`error.{}` is not a member of error set `{}`", name, sn));
                            }
                        }
                        TPat::Error(id)
                    }
                    _ => {
                        let tn = self.type_name(ty);
                        self.error(*span, format!("error pattern on a value of type `{}`, which is not an error union", tn));
                        TPat::Wild
                    }
                }
            }
            Pattern::Range { lo, hi, inclusive, span } => {
                match k {
                    TyKind::Opt(inner) => return TPat::Some(Box::new(self.check_pattern(pat, inner))),
                    _ => {}
                }
                if !self.tys.is_integer(ty) && !matches!(k, TyKind::Char) {
                    self.error(*span, "range patterns apply to integers and chars");
                    return TPat::Wild;
                }
                let m = self.cur().module;
                let g = self.cur().generics.clone();
                let lo_v = self.pattern_const_int(lo, m, &g);
                let hi_v = self.pattern_const_int(hi, m, &g);
                match (lo_v, hi_v) {
                    (Some(l), Some(h)) => TPat::Range { lo: l, hi: h, inclusive: *inclusive },
                    _ => {
                        self.error(*span, "range pattern bounds must be compile-time constants");
                        TPat::Wild
                    }
                }
            }
            Pattern::Or { alts, span } => {
                let mut out = Vec::new();
                for a in alts {
                    if let Pattern::Binding { span: bs, .. } = a {
                        self.error(*bs, "bindings are not allowed inside `|` patterns");
                    }
                    out.push(self.check_pattern(a, ty));
                }
                let _ = span;
                TPat::Or(out)
            }
            Pattern::Tuple { elems, span } => match k {
                TyKind::Tuple(ts) => {
                    if ts.len() != elems.len() {
                        self.error(*span, format!("tuple pattern has {} elements but the value has {}", elems.len(), ts.len()));
                        return TPat::Wild;
                    }
                    let ps = elems.iter().zip(ts.iter()).map(|(p, &t)| self.check_pattern(p, t)).collect();
                    TPat::Tuple(ps)
                }
                _ => {
                    let tn = self.type_name(ty);
                    self.error(*span, format!("tuple pattern on a value of type `{}`", tn));
                    TPat::Wild
                }
            },
            Pattern::Binary { segments, span } => {
                let is_bytes = match k {
                    TyKind::Slice(_, e) => matches!(self.tys.kind(self.tys.shallow(e)), TyKind::Int(IntTy::U8)),
                    TyKind::Str => true,
                    TyKind::Array(_, e) => matches!(self.tys.kind(self.tys.shallow(e)), TyKind::Int(IntTy::U8)),
                    _ => false,
                };
                if !is_bytes {
                    let tn = self.type_name(ty);
                    self.error(*span, format!("binary patterns match `[]u8` values, but this value has type `{}`", tn));
                    return TPat::Wild;
                }
                let segs = self.check_bin_pattern(segments, *span);
                TPat::Binary(segs)
            }
            Pattern::Variant { path, args, span } => {
                match k {
                    TyKind::Opt(inner) => return TPat::Some(Box::new(self.check_pattern(pat, inner))),
                    TyKind::ErrUnion(_, inner) => return TPat::Ok(Box::new(self.check_pattern(pat, inner))),
                    _ => {}
                }
                let (d, name) = match k {
                    TyKind::Enum(d, _) => (d, path.last().unwrap().clone()),
                    TyKind::Bool if path.len() == 1 && (path[0] == "true" || path[0] == "false") => {
                        return TPat::Bool(path[0] == "true");
                    }
                    _ => {
                        let tn = self.type_name(ty);
                        if path.len() == 1 && args.is_empty() {
                            self.error(*span, format!("`.{}` is an enum variant pattern but the value has type `{}`", path[0], tn));
                        } else {
                            self.error(*span, format!("variant pattern `{}` on a value of type `{}`", path.join("."), tn));
                        }
                        return TPat::Wild;
                    }
                };
                let def = self.enums[d as usize].clone();
                if path.len() == 2 {
                    let m = self.cur().module;
                    match self.lookup_item(m, &path[0]) {
                        Some(ItemRef::Enum(id)) if id == d => {}
                        _ => {
                            self.error(*span, format!("`{}` is not the enum `{}`", path[0], def.name));
                        }
                    }
                } else if path.len() > 2 {
                    self.error(*span, "variant patterns are `.Variant` or `Enum.Variant`");
                }
                let idx = match def.variants.iter().position(|v| v.name == name) {
                    Some(i) => i,
                    None => {
                        let mut hint = String::new();
                        if let Some(close) = def.variants.iter().map(|v| v.name.as_str()).find(|v| v.eq_ignore_ascii_case(&name)) {
                            hint = format!("; did you mean `.{}`?", close);
                        }
                        self.error(*span, format!("enum `{}` has no variant `{}`{}", def.name, name, hint));
                        return TPat::Wild;
                    }
                };
                let vtys = self.enum_variant_types(ty);
                let want = vtys[idx].clone();
                if args.is_empty() && !want.is_empty() {
                    // allow `.Variant` to match any payload
                    return TPat::Variant { idx: idx as u32, args: want.iter().map(|_| TPat::Wild).collect() };
                }
                if args.len() != want.len() {
                    self.error(*span, format!("variant `{}.{}` has {} value(s) but the pattern binds {}", def.name, name, want.len(), args.len()));
                    return TPat::Wild;
                }
                let ps = args.iter().zip(want.iter()).map(|(p, &t)| self.check_pattern(p, t)).collect();
                TPat::Variant { idx: idx as u32, args: ps }
            }
        }
    }

    fn pattern_const_int(&mut self, e: &Expr, module: u32, generics: &HashMap<String, TyId>) -> Option<i128> {
        match e {
            Expr::Lit { value: Lit::Char(c), .. } => Some(*c as i128),
            Expr::Unary { op: UnOp::Neg, expr, .. } => self.pattern_const_int(expr, module, generics).map(|v| -v),
            _ => self.const_eval_int(e, module, generics),
        }
    }

    fn check_lit_pattern(&mut self, value: &Lit, negative: bool, ty: TyId, span: Span) -> TPat {
        let k = self.tys.kind(ty).clone();
        let under = match k {
            TyKind::Distinct(d) => self.tys.kind(self.tys.shallow(self.distinct_underlying[&d])).clone(),
            other => other,
        };
        match value {
            Lit::Int(v) => {
                let v = if negative { -(*v as i128) } else { *v as i128 };
                match under {
                    TyKind::Int(it) => {
                        if v < it.min() || v > it.max() {
                            self.error(span, format!("literal `{}` does not fit in `{}`", v, it.name()));
                        }
                        TPat::Int(v)
                    }
                    TyKind::Infer(_) if self.tys.is_integer(ty) => TPat::Int(v),
                    TyKind::Float(_) => TPat::Float(v as f64),
                    TyKind::Char => TPat::Char(v as u32),
                    _ => {
                        let tn = self.type_name(ty);
                        self.error(span, format!("integer pattern on a value of type `{}`", tn));
                        TPat::Wild
                    }
                }
            }
            Lit::Float(f) => match under {
                TyKind::Float(_) => TPat::Float(if negative { -*f } else { *f }),
                _ => {
                    let tn = self.type_name(ty);
                    self.error(span, format!("float pattern on a value of type `{}`", tn));
                    TPat::Wild
                }
            },
            Lit::Str(s) | Lit::Bytes(s) => match under {
                TyKind::Slice(_, e) if matches!(self.tys.kind(self.tys.shallow(e)), TyKind::Int(IntTy::U8)) => TPat::Str(s.clone()),
                TyKind::Str => TPat::Str(s.clone()),
                _ => {
                    let tn = self.type_name(ty);
                    self.error(span, format!("string pattern on a value of type `{}`", tn));
                    TPat::Wild
                }
            },
            Lit::Char(c) => match under {
                TyKind::Char => TPat::Char(*c),
                TyKind::Int(it) => {
                    if (*c as i128) > it.max() {
                        self.error(span, format!("character does not fit in `{}`", it.name()));
                    }
                    TPat::Int(*c as i128)
                }
                _ => {
                    let tn = self.type_name(ty);
                    self.error(span, format!("character pattern on a value of type `{}`", tn));
                    TPat::Wild
                }
            },
            Lit::Bool(b) => match under {
                TyKind::Bool => TPat::Bool(*b),
                _ => {
                    let tn = self.type_name(ty);
                    self.error(span, format!("boolean pattern on a value of type `{}`", tn));
                    TPat::Wild
                }
            },
        }
    }

    fn pat_irrefutable(&self, p: &TPat) -> bool {
        match p {
            TPat::Wild | TPat::Bind(_) => true,
            TPat::Tuple(ps) => ps.iter().all(|p| self.pat_irrefutable(p)),
            TPat::Or(ps) => ps.iter().any(|p| self.pat_irrefutable(p)),
            _ => false,
        }
    }

    fn check_exhaustive(&mut self, st: TyId, arms: &[TArm], span: Span) {
        let unguarded: Vec<&TPat> = arms.iter().filter(|a| a.guard.is_none()).map(|a| &a.pat).collect();
        if unguarded.iter().any(|p| self.pat_irrefutable(p)) {
            return;
        }
        let k = self.tys.kind(self.tys.shallow(st)).clone();
        let mut flat: Vec<&TPat> = Vec::new();
        for p in &unguarded {
            match p {
                TPat::Or(ps) => flat.extend(ps.iter()),
                p => flat.push(p),
            }
        }
        let ok = match k {
            TyKind::Bool => flat.iter().any(|p| matches!(p, TPat::Bool(true))) && flat.iter().any(|p| matches!(p, TPat::Bool(false))),
            TyKind::Enum(d, _) => {
                let n = self.enums[d as usize].variants.len();
                (0..n).all(|i| flat.iter().any(|p| matches!(p, TPat::Variant { idx, args } if *idx as usize == i && args.iter().all(|a| self.pat_irrefutable(a)))))
            }
            TyKind::Opt(inner) => {
                let has_null = flat.iter().any(|p| matches!(p, TPat::Null));
                let some_pats: Vec<&TPat> = flat.iter().filter_map(|p| if let TPat::Some(q) = p { Some(&**q) } else { None }).collect();
                let some_ok = some_pats.iter().any(|p| self.pat_irrefutable(p)) || self.sub_exhaustive(inner, &some_pats);
                has_null && some_ok
            }
            TyKind::ErrUnion(set, inner) => {
                let ok_pats: Vec<&TPat> = flat.iter().filter_map(|p| if let TPat::Ok(q) = p { Some(&**q) } else { None }).collect();
                let ok_ok = ok_pats.iter().any(|p| self.pat_irrefutable(p)) || self.sub_exhaustive(inner, &ok_pats);
                let errs_ok = match set {
                    Some(s) => {
                        let ids = self.error_sets[s as usize].ids.clone();
                        ids.iter().all(|id| flat.iter().any(|p| matches!(p, TPat::Error(e) if e == id)))
                    }
                    None => false,
                };
                ok_ok && errs_ok
            }
            _ => false,
        };
        if !ok {
            let hint = match k {
                TyKind::Enum(d, _) => {
                    let missing: Vec<String> = self.enums[d as usize]
                        .variants
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| !flat.iter().any(|p| matches!(p, TPat::Variant { idx, args } if *idx as usize == *i && args.iter().all(|a| self.pat_irrefutable(a)))))
                        .map(|(_, v)| format!(".{}", v.name))
                        .collect();
                    format!("; not covered: {}", missing.join(", "))
                }
                TyKind::Slice(..) | TyKind::Str | TyKind::Array(..) => "; binary and string matches need a catch-all arm `_ =>` (section 6.2)".to_string(),
                TyKind::ErrUnion(..) => "; add `else => ...` to handle the remaining errors".to_string(),
                TyKind::Opt(_) => "; cover both `null` and a binding for the value".to_string(),
                _ => "; add a catch-all arm `_ =>`".to_string(),
            };
            self.error(span, format!("`match` is not exhaustive{}", hint));
        }
    }

    fn sub_exhaustive(&self, t: TyId, pats: &[&TPat]) -> bool {
        match self.tys.kind(self.tys.shallow(t)).clone() {
            TyKind::Bool => pats.iter().any(|p| matches!(p, TPat::Bool(true))) && pats.iter().any(|p| matches!(p, TPat::Bool(false))),
            TyKind::Enum(d, _) => {
                let n = self.enums[d as usize].variants.len();
                (0..n).all(|i| pats.iter().any(|p| matches!(p, TPat::Variant { idx, args } if *idx as usize == i && args.iter().all(|a| self.pat_irrefutable(a)))))
            }
            _ => false,
        }
    }
}
