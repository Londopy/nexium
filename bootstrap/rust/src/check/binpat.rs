//! Binary pattern matching and construction (section 6).

use super::*;
use crate::ast::*;
use crate::tir::*;
use crate::types::*;

/// Section 6.1: size arithmetic is computed at pointer width regardless of the
/// operand bindings' widths. Every identifier leaf is widened to `usize`.
fn widen_size_expr(e: &Expr) -> Expr {
    match e {
        Expr::Ident { span, .. } => Expr::Cast { expr: Box::new(e.clone()), ty: TypeExpr::named("usize", *span), span: *span },
        Expr::Binary { op, lhs, rhs, span } => Expr::Binary { op: *op, lhs: Box::new(widen_size_expr(lhs)), rhs: Box::new(widen_size_expr(rhs)), span: *span },
        Expr::Unary { op, expr, span } => Expr::Unary { op: *op, expr: Box::new(widen_size_expr(expr)), span: *span },
        other => other.clone(),
    }
}

impl<'a> Checker<'a> {
    fn seg_endian(&mut self, seg: &BinSegment) -> (Endian, bool, bool, bool) {
        let mut endian = Endian::Big;
        let (mut signed, mut float, mut utf8) = (false, false, false);
        for m in &seg.modifiers {
            match m.as_str() {
                "big" => endian = Endian::Big,
                "little" => endian = Endian::Little,
                "native" => endian = Endian::Native,
                "signed" => signed = true,
                "unsigned" => signed = false,
                "float" => float = true,
                "utf8" => utf8 = true,
                _ => {}
            }
        }
        (endian, signed, float, utf8)
    }

    /// Resolve a segment's size into bits, in the current scope (earlier bindings visible).
    fn seg_size(&mut self, seg: &BinSegment, default_bits: u64) -> TBinSize {
        match &seg.size {
            None => TBinSize::Bits(default_bits),
            Some(e) => {
                let m = self.cur().module;
                let g = self.cur().generics.clone();
                if let Some(v) = self.const_eval_int(e, m, &g) {
                    if v <= 0 {
                        self.error(e.span(), "segment size must be positive");
                        return TBinSize::Bits(8);
                    }
                    return TBinSize::Bits(v as u64);
                }
                let u = self.tys.usize();
                let widened = widen_size_expr(e);
                let te = self.check_expr(&widened, Some(u));
                let te = self.coerce_or_error(te, u, "segment size");
                // size arithmetic is always checked (6.1)
                TBinSize::Expr(Box::new(te))
            }
        }
    }

    fn int_type_for_bits(&mut self, bits: u64, signed: bool) -> TyId {
        let it = match (bits, signed) {
            (0..=8, false) => IntTy::U8,
            (0..=8, true) => IntTy::I8,
            (9..=16, false) => IntTy::U16,
            (9..=16, true) => IntTy::I16,
            (17..=32, false) => IntTy::U32,
            (17..=32, true) => IntTy::I32,
            (_, false) => IntTy::U64,
            (_, true) => IntTy::I64,
        };
        self.tys.int(it)
    }

    pub fn check_bin_pattern(&mut self, segments: &[BinSegment], span: Span) -> Vec<TBinSeg> {
        let mut out = Vec::new();
        let u8t = self.tys.u8();
        let bytes_t = self.tys.slice(false, u8t);
        for (i, seg) in segments.iter().enumerate() {
            let (endian, signed, float, utf8) = self.seg_endian(seg);
            match &seg.value {
                BinSegValue::Rest(name) => {
                    if i != segments.len() - 1 {
                        self.error(seg.span, "a `bytes` segment consumes the remainder and must be last");
                    }
                    let l = self.declare_local(name, bytes_t, false, seg.span);
                    out.push(TBinSeg { kind: TBinSegKind::Rest(l), size: TBinSize::Rest, endian, signed, float, utf8, ty: bytes_t, span: seg.span });
                }
                BinSegValue::Bind(name) => {
                    let default_bits = if float { 64 } else { 8 };
                    let size = self.seg_size(seg, default_bits);
                    let ty = match &size {
                        TBinSize::Bits(b) => {
                            if float {
                                match b {
                                    32 => self.tys.float(FloatTy::F32),
                                    64 => self.tys.float(FloatTy::F64),
                                    _ => {
                                        self.error(seg.span, "float segments are 32 or 64 bits");
                                        self.tys.float(FloatTy::F64)
                                    }
                                }
                            } else if utf8 || *b > 64 {
                                if b % 8 != 0 {
                                    self.error(seg.span, "byte-slice segments need a size that is a multiple of 8 bits");
                                }
                                bytes_t
                            } else {
                                self.int_type_for_bits(*b, signed)
                            }
                        }
                        TBinSize::Expr(_) => bytes_t,
                        TBinSize::Rest => bytes_t,
                    };
                    let l = self.declare_local(name, ty, false, seg.span);
                    out.push(TBinSeg { kind: TBinSegKind::Bind(l), size, endian, signed, float, utf8, ty, span: seg.span });
                }
                BinSegValue::Expr(e) => {
                    // literal to match
                    match &**e {
                        Expr::Lit { value: Lit::Str(s), .. } | Expr::Lit { value: Lit::Bytes(s), .. } => {
                            if seg.size.is_some() {
                                self.error(seg.span, "string segments take their size from the literal");
                            }
                            let te = self.check_expr(e, None);
                            let bits = (s.len() * 8) as u64;
                            out.push(TBinSeg { kind: TBinSegKind::Value(Box::new(te)), size: TBinSize::Bits(bits), endian, signed, float, utf8, ty: bytes_t, span: seg.span });
                        }
                        _ => {
                            let size = self.seg_size(seg, 8);
                            let ty = match &size {
                                TBinSize::Bits(b) if float => {
                                    if *b == 32 {
                                        self.tys.float(FloatTy::F32)
                                    } else {
                                        self.tys.float(FloatTy::F64)
                                    }
                                }
                                TBinSize::Bits(b) if *b <= 64 => self.int_type_for_bits(*b, signed),
                                _ => {
                                    self.error(seg.span, "literal segments must be at most 64 bits");
                                    self.tys.int(IntTy::U64)
                                }
                            };
                            let te = self.check_expr(e, Some(ty));
                            let te = self.coerce_or_error(te, ty, "binary segment literal");
                            if let (TExprKind::Int(v), TBinSize::Bits(b)) = (&te.kind, &size) {
                                if *b < 64 && !signed && (*v < 0 || *v >= (1i128 << b)) {
                                    self.error(seg.span, format!("literal `{}` does not fit in {} bits", v, b));
                                }
                            }
                            out.push(TBinSeg { kind: TBinSegKind::Value(Box::new(te)), size, endian, signed, float, utf8, ty, span: seg.span });
                        }
                    }
                }
            }
        }
        let _ = span;
        out
    }

    pub fn check_bin_construct(&mut self, segments: &[BinSegment], target: &Expr, span: Span) -> TExpr {
        let u8t = self.tys.u8();
        let bytes_t = self.tys.slice(false, u8t);
        let mut_bytes = self.tys.slice(true, u8t);
        let t = self.check_expr(target, Some(mut_bytes));
        let t = self.coerce_or_error(t, mut_bytes, "construction target (needs `[]mut u8`)");
        let mut out = Vec::new();
        for seg in segments {
            let (endian, signed, float, utf8) = self.seg_endian(seg);
            let value_expr: Expr = match &seg.value {
                BinSegValue::Bind(name) => Expr::Ident { name: name.clone(), span: seg.span },
                BinSegValue::Rest(name) => Expr::Ident { name: name.clone(), span: seg.span },
                BinSegValue::Expr(e) => (**e).clone(),
            };
            let is_rest = matches!(seg.value, BinSegValue::Rest(_)) || seg.modifiers.iter().any(|m| m == "bytes");
            if is_rest {
                let te = self.check_expr(&value_expr, Some(bytes_t));
                let te = self.coerce_or_error(te, bytes_t, "byte segment");
                out.push(TBinSeg { kind: TBinSegKind::Value(Box::new(te)), size: TBinSize::Rest, endian, signed, float, utf8, ty: bytes_t, span: seg.span });
                continue;
            }
            // string literal: whole literal
            if let Expr::Lit { value: Lit::Str(s), .. } | Expr::Lit { value: Lit::Bytes(s), .. } = &value_expr {
                let te = self.check_expr(&value_expr, None);
                let bits = (s.len() * 8) as u64;
                out.push(TBinSeg { kind: TBinSegKind::Value(Box::new(te)), size: TBinSize::Bits(bits), endian, signed, float, utf8, ty: bytes_t, span: seg.span });
                continue;
            }
            let default_bits = if float { 64 } else { 8 };
            let size = self.seg_size(seg, default_bits);
            let te = self.check_expr(&value_expr, None);
            let vt = self.tys.resolve(te.ty, true);
            let kind = self.tys.kind(vt).clone();
            let te = match kind {
                TyKind::Int(_) | TyKind::Char | TyKind::Bool => te,
                TyKind::Float(FloatTy::F32) => {
                    if !matches!(size, TBinSize::Bits(32)) && !float {
                        self.error(seg.span, "an f32 segment needs `:32/float`");
                    }
                    te
                }
                TyKind::Float(FloatTy::F64) => {
                    if !float {
                        self.error(seg.span, "float values need the `/float` modifier");
                    }
                    te
                }
                TyKind::Slice(_, e) if matches!(self.tys.kind(self.tys.shallow(e)), TyKind::Int(IntTy::U8)) => {
                    // a byte slice with an explicit size in bits (must be a multiple of 8)
                    out.push(TBinSeg { kind: TBinSegKind::Value(Box::new(te)), size, endian, signed, float, utf8, ty: bytes_t, span: seg.span });
                    continue;
                }
                TyKind::Str => {
                    let te = TExpr { kind: TExprKind::StrToSlice(Box::new(te)), ty: bytes_t, span: seg.span };
                    out.push(TBinSeg { kind: TBinSegKind::Value(Box::new(te)), size, endian, signed, float, utf8, ty: bytes_t, span: seg.span });
                    continue;
                }
                _ => {
                    let tn = self.type_name(vt);
                    self.error(seg.span, format!("cannot write a value of type `{}` into a binary; use integers, floats, or `[]u8`", tn));
                    te
                }
            };
            out.push(TBinSeg { kind: TBinSegKind::Value(Box::new(te)), size, endian, signed, float, utf8, ty: vt, span: seg.span });
        }
        let ret = self.tys.err_union(None, bytes_t);
        self.mk(TExprKind::BinConstruct { segments: out, target: Box::new(t) }, ret, span)
    }
}
