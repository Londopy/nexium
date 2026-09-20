//! Effect propagation (E1, E2) and negative-bound checking (E3, E4).

use super::*;
use crate::effects::Effects;
use crate::tir::*;

impl<'a> Checker<'a> {
    /// effects(f) = own(f) ∪ ⋃ effects(callee), to a fixpoint.
    pub fn propagate_effects(&mut self) {
        let n = self.funcs.len();
        for f in &mut self.funcs {
            f.effects = f.own_effects;
            if f.is_extern {
                f.effects = f.effects.union(Effects::FFI).union(Effects::BLOCKS).union(Effects::PANICS).union(Effects::ALLOCATES).union(Effects::NONDETERMINISTIC).union(Effects::SHARED_MUTABLE);
            }
        }
        let mut changed = true;
        while changed {
            changed = false;
            for i in 0..n {
                let mut e = self.funcs[i].effects;
                let callees = self.funcs[i].callees.clone();
                for (c, _) in callees {
                    e = e.union(self.funcs[c as usize].effects);
                }
                if e != self.funcs[i].effects {
                    self.funcs[i].effects = e;
                    changed = true;
                }
            }
        }
    }

    /// Find the function that introduces effect `e` on some call path from `inst`.
    fn find_introducer(&self, inst: InstId, e: Effects) -> Option<(InstId, Span, String)> {
        let mut visited = HashSet::new();
        let mut stack = vec![inst];
        while let Some(i) = stack.pop() {
            if !visited.insert(i) {
                continue;
            }
            let f = &self.funcs[i as usize];
            if f.own_effects.contains(e) {
                if let Some((_, sp, why)) = f.witnesses.iter().find(|(w, _, _)| w.contains(e)) {
                    return Some((i, *sp, why.clone()));
                }
                return Some((i, f.span, String::new()));
            }
            if f.is_extern {
                return Some((i, f.span, "foreign functions may do anything".into()));
            }
            for (c, _) in &f.callees {
                if self.funcs[*c as usize].effects.contains(e) {
                    stack.push(*c);
                }
            }
        }
        None
    }

    pub fn check_effect_bounds(&mut self) {
        let n = self.funcs.len();
        for i in 0..n {
            let f = self.funcs[i].clone();
            if f.body.is_none() {
                continue;
            }
            let violated = f.effects.intersect(f.declared_neg);
            for (name, e) in Effects::NAMES {
                if !violated.contains(*e) {
                    continue;
                }
                let bound_span = f.declared_spans.iter().find(|(x, _)| x.contains(*e)).map(|(_, s)| *s).unwrap_or(f.span);
                let msg = format!("function `{}` is declared `!{}` but has the `{}` effect", f.name, name, name);
                match self.find_introducer(i as InstId, *e) {
                    Some((intro, sp, why)) if intro == i as InstId => {
                        let d = Diag::error(bound_span, msg).note(Some(sp), format!("the effect is introduced here: {}", why));
                        self.diags.push(d);
                    }
                    Some((intro, sp, why)) => {
                        // where is it called from?
                        let call_span = self.first_call_site(i as InstId, *e).unwrap_or(f.span);
                        let iname = self.funcs[intro as usize].name.clone();
                        let d = Diag::error(bound_span, msg).note(Some(call_span), format!("this call acquires `{}`", name)).note(Some(sp), format!("introduced by `{}` here: {}", iname, why));
                        self.diags.push(d);
                    }
                    None => self.error(bound_span, msg),
                }
            }
        }
        // calls inside `for parallel` bodies must not mutate shared state
        let pobs = self.parallel_obligations.clone();
        for (inst, span) in pobs {
            let f = self.funcs[inst as usize].clone();
            if f.effects.contains(Effects::SHARED_MUTABLE) {
                let d = match self.find_introducer(inst, Effects::SHARED_MUTABLE) {
                    Some((_, sp, why)) => {
                        Diag::error(span, format!("`{}` is called inside a `for parallel` body but has the `shared_mutable` effect", f.name)).note(Some(sp), format!("introduced here: {}", why))
                    }
                    None => Diag::error(span, format!("`{}` is called inside a `for parallel` body but has the `shared_mutable` effect", f.name)),
                };
                self.diags.push(d);
            }
        }

        // closures / function values coerced to bounded function types
        let obligations = self.fn_value_obligations.clone();
        for (inst, neg, span) in obligations {
            let f = self.funcs[inst as usize].clone();
            let violated = f.effects.intersect(neg);
            for (name, e) in Effects::NAMES {
                if violated.contains(*e) {
                    let what = if f.is_closure { "this closure".to_string() } else { format!("`{}`", f.name) };
                    let d = match self.find_introducer(inst, *e) {
                        Some((_, sp, why)) => Diag::error(span, format!("{} has the `{}` effect, but the function type it is used as forbids it (`!{}`)", what, name, name))
                            .note(Some(sp), format!("introduced here: {}", why)),
                        None => Diag::error(span, format!("{} has the `{}` effect, but the function type it is used as forbids it", what, name)),
                    };
                    self.diags.push(d);
                }
            }
        }
    }

    fn first_call_site(&self, inst: InstId, e: Effects) -> Option<Span> {
        let f = &self.funcs[inst as usize];
        f.callees.iter().find(|(c, _)| self.funcs[*c as usize].effects.contains(e)).map(|(_, s)| *s)
    }
}
