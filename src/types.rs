//! Type representation. Types are interned; `TyId` compares by identity.

use std::collections::HashMap;

pub type TyId = u32;
pub type DefId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IntTy {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    Isize,
    Usize,
}

impl IntTy {
    pub fn is_signed(self) -> bool {
        matches!(self, IntTy::I8 | IntTy::I16 | IntTy::I32 | IntTy::I64 | IntTy::I128 | IntTy::Isize)
    }
    pub fn bits(self) -> u32 {
        match self {
            IntTy::I8 | IntTy::U8 => 8,
            IntTy::I16 | IntTy::U16 => 16,
            IntTy::I32 | IntTy::U32 => 32,
            IntTy::I64 | IntTy::U64 | IntTy::Isize | IntTy::Usize => 64,
            IntTy::I128 | IntTy::U128 => 128,
        }
    }
    pub fn min(self) -> i128 {
        if self.is_signed() {
            -(1i128 << (self.bits() - 1))
        } else {
            0
        }
    }
    pub fn max(self) -> i128 {
        if self.is_signed() {
            (1i128 << (self.bits() - 1)) - 1
        } else if self.bits() == 128 {
            i128::MAX
        } else {
            (1i128 << self.bits()) - 1
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            IntTy::I8 => "i8",
            IntTy::I16 => "i16",
            IntTy::I32 => "i32",
            IntTy::I64 => "i64",
            IntTy::I128 => "i128",
            IntTy::U8 => "u8",
            IntTy::U16 => "u16",
            IntTy::U32 => "u32",
            IntTy::U64 => "u64",
            IntTy::U128 => "u128",
            IntTy::Isize => "isize",
            IntTy::Usize => "usize",
        }
    }
    pub fn c_name(self) -> &'static str {
        match self {
            IntTy::I8 => "int8_t",
            IntTy::I16 => "int16_t",
            IntTy::I32 => "int32_t",
            IntTy::I64 => "int64_t",
            IntTy::I128 => "__int128",
            IntTy::U8 => "uint8_t",
            IntTy::U16 => "uint16_t",
            IntTy::U32 => "uint32_t",
            IntTy::U64 => "uint64_t",
            IntTy::U128 => "unsigned __int128",
            IntTy::Isize => "intptr_t",
            IntTy::Usize => "size_t",
        }
    }
    pub fn from_name(s: &str) -> Option<IntTy> {
        Some(match s {
            "i8" => IntTy::I8,
            "i16" => IntTy::I16,
            "i32" => IntTy::I32,
            "i64" => IntTy::I64,
            "i128" => IntTy::I128,
            "u8" => IntTy::U8,
            "u16" => IntTy::U16,
            "u32" => IntTy::U32,
            "u64" => IntTy::U64,
            "u128" => IntTy::U128,
            "isize" => IntTy::Isize,
            "usize" => IntTy::Usize,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FloatTy {
    F32,
    F64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TyKind {
    Int(IntTy),
    Float(FloatTy),
    Bool,
    Char,
    Void,
    Never,
    /// a struct, record, or ref class; args are generic arguments
    Struct(DefId, Vec<TyId>),
    Enum(DefId, Vec<TyId>),
    Array(u64, TyId),
    Slice(bool, TyId),
    Ptr(bool, TyId),
    Opt(TyId),
    /// error union; the set is None for the inferred/global set
    ErrUnion(Option<DefId>, TyId),
    Fn(Vec<TyId>, TyId, u32),
    Tuple(Vec<TyId>),
    Distinct(DefId),
    Weak(TyId),
    List(TyId),
    Str,
    Map(TyId, TyId),
    /// a compile-time type value
    Type,
    /// a value of an error set (or of the global error set when None)
    ErrorSet(Option<DefId>),
    /// unresolved inference variable
    Infer(u32),
    /// untyped integer literal awaiting context
    IntLit,
    FloatLit,
    /// a closure with a unique identity
    Closure(u32),
    /// generic parameter inside an uninstantiated body (only used for display)
    Param(String),
    /// a module namespace value (e.g. `math`)
    Namespace(String),
    /// a trait object: fat pointer to a value implementing the trait, with the
    /// effects its methods are permitted (E4/E7)
    Dyn(DefId, u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InferKind {
    Any,
    Integer,
    Float,
}

#[derive(Default)]
pub struct TyTable {
    kinds: Vec<TyKind>,
    map: HashMap<TyKind, TyId>,
    pub subst: Vec<Option<TyId>>,
    pub infer_kinds: Vec<InferKind>,
    next_infer: u32,
}

impl TyTable {
    pub fn intern(&mut self, k: TyKind) -> TyId {
        if let Some(&id) = self.map.get(&k) {
            return id;
        }
        let id = self.kinds.len() as TyId;
        self.kinds.push(k.clone());
        self.map.insert(k, id);
        id
    }
    pub fn kind(&self, id: TyId) -> &TyKind {
        &self.kinds[id as usize]
    }
    pub fn fresh_infer(&mut self) -> TyId {
        self.fresh_infer_kind(InferKind::Any)
    }
    pub fn fresh_infer_kind(&mut self, kind: InferKind) -> TyId {
        let n = self.next_infer;
        self.next_infer += 1;
        self.subst.push(None);
        self.infer_kinds.push(kind);
        self.intern(TyKind::Infer(n))
    }
    pub fn infer_kind(&self, t: TyId) -> Option<InferKind> {
        match self.kind(self.shallow(t)) {
            TyKind::Infer(n) => Some(self.infer_kinds[*n as usize]),
            _ => None,
        }
    }

    pub fn int(&mut self, i: IntTy) -> TyId {
        self.intern(TyKind::Int(i))
    }
    pub fn float(&mut self, f: FloatTy) -> TyId {
        self.intern(TyKind::Float(f))
    }
    pub fn bool(&mut self) -> TyId {
        self.intern(TyKind::Bool)
    }
    pub fn void(&mut self) -> TyId {
        self.intern(TyKind::Void)
    }
    pub fn never(&mut self) -> TyId {
        self.intern(TyKind::Never)
    }
    pub fn char(&mut self) -> TyId {
        self.intern(TyKind::Char)
    }
    pub fn usize(&mut self) -> TyId {
        self.intern(TyKind::Int(IntTy::Usize))
    }
    pub fn u8(&mut self) -> TyId {
        self.intern(TyKind::Int(IntTy::U8))
    }
    pub fn slice(&mut self, m: bool, t: TyId) -> TyId {
        self.intern(TyKind::Slice(m, t))
    }
    pub fn ptr(&mut self, m: bool, t: TyId) -> TyId {
        self.intern(TyKind::Ptr(m, t))
    }
    pub fn opt(&mut self, t: TyId) -> TyId {
        self.intern(TyKind::Opt(t))
    }
    pub fn err_union(&mut self, set: Option<DefId>, t: TyId) -> TyId {
        self.intern(TyKind::ErrUnion(set, t))
    }
    pub fn array(&mut self, n: u64, t: TyId) -> TyId {
        self.intern(TyKind::Array(n, t))
    }
    pub fn tuple(&mut self, ts: Vec<TyId>) -> TyId {
        self.intern(TyKind::Tuple(ts))
    }
    pub fn str_slice(&mut self) -> TyId {
        let u = self.u8();
        self.slice(false, u)
    }
    pub fn list(&mut self, t: TyId) -> TyId {
        self.intern(TyKind::List(t))
    }
    pub fn string(&mut self) -> TyId {
        self.intern(TyKind::Str)
    }
    pub fn type_ty(&mut self) -> TyId {
        self.intern(TyKind::Type)
    }
    pub fn int_lit(&mut self) -> TyId {
        self.fresh_infer_kind(InferKind::Integer)
    }
    pub fn float_lit(&mut self) -> TyId {
        self.fresh_infer_kind(InferKind::Float)
    }

    /// Follow inference bindings to the representative type (shallow).
    pub fn shallow(&self, mut t: TyId) -> TyId {
        loop {
            match self.kind(t) {
                TyKind::Infer(n) => match self.subst[*n as usize] {
                    Some(b) => t = b,
                    None => return t,
                },
                _ => return t,
            }
        }
    }

    /// Fully resolve a type, substituting bound inference variables and defaulting
    /// unconstrained literals when `default_lits` is set.
    pub fn resolve(&mut self, t: TyId, default_lits: bool) -> TyId {
        let t = self.shallow(t);
        let k = self.kind(t).clone();
        match k {
            TyKind::Infer(n) => {
                if default_lits {
                    match self.infer_kinds[n as usize] {
                        InferKind::Integer => {
                            let r = self.int(IntTy::I64);
                            self.subst[n as usize] = Some(r);
                            r
                        }
                        InferKind::Float => {
                            let r = self.float(FloatTy::F64);
                            self.subst[n as usize] = Some(r);
                            r
                        }
                        InferKind::Any => t,
                    }
                } else {
                    t
                }
            }
            TyKind::IntLit => {
                if default_lits {
                    self.int(IntTy::I64)
                } else {
                    t
                }
            }
            TyKind::FloatLit => {
                if default_lits {
                    self.float(FloatTy::F64)
                } else {
                    t
                }
            }
            TyKind::Struct(d, args) => {
                let args = args.into_iter().map(|a| self.resolve(a, default_lits)).collect();
                self.intern(TyKind::Struct(d, args))
            }
            TyKind::Enum(d, args) => {
                let args = args.into_iter().map(|a| self.resolve(a, default_lits)).collect();
                self.intern(TyKind::Enum(d, args))
            }
            TyKind::Array(n, e) => {
                let e = self.resolve(e, default_lits);
                self.array(n, e)
            }
            TyKind::Slice(m, e) => {
                let e = self.resolve(e, default_lits);
                self.slice(m, e)
            }
            TyKind::Ptr(m, e) => {
                let e = self.resolve(e, default_lits);
                self.ptr(m, e)
            }
            TyKind::Opt(e) => {
                let e = self.resolve(e, default_lits);
                self.opt(e)
            }
            TyKind::Weak(e) => {
                let e = self.resolve(e, default_lits);
                self.intern(TyKind::Weak(e))
            }
            TyKind::ErrUnion(s, e) => {
                let e = self.resolve(e, default_lits);
                self.err_union(s, e)
            }
            TyKind::Fn(ps, r, ef) => {
                let ps = ps.into_iter().map(|p| self.resolve(p, default_lits)).collect();
                let r = self.resolve(r, default_lits);
                self.intern(TyKind::Fn(ps, r, ef))
            }
            TyKind::Tuple(ts) => {
                let ts = ts.into_iter().map(|p| self.resolve(p, default_lits)).collect();
                self.tuple(ts)
            }
            TyKind::List(e) => {
                let e = self.resolve(e, default_lits);
                self.list(e)
            }
            TyKind::Map(k, v) => {
                let k = self.resolve(k, default_lits);
                let v = self.resolve(v, default_lits);
                self.intern(TyKind::Map(k, v))
            }
            _ => t,
        }
    }

    pub fn contains_infer(&self, t: TyId) -> bool {
        let t = self.shallow(t);
        match self.kind(t) {
            TyKind::Infer(_) | TyKind::IntLit | TyKind::FloatLit => true,
            TyKind::Struct(_, args) | TyKind::Enum(_, args) | TyKind::Tuple(args) => args.iter().any(|&a| self.contains_infer(a)),
            TyKind::Array(_, e) | TyKind::Slice(_, e) | TyKind::Ptr(_, e) | TyKind::Opt(e) | TyKind::Weak(e) | TyKind::ErrUnion(_, e) | TyKind::List(e) => self.contains_infer(*e),
            TyKind::Map(k, v) => self.contains_infer(*k) || self.contains_infer(*v),
            TyKind::Fn(ps, r, _) => ps.iter().any(|&p| self.contains_infer(p)) || self.contains_infer(*r),
            _ => false,
        }
    }

    pub fn is_integer(&self, t: TyId) -> bool {
        match self.kind(self.shallow(t)) {
            TyKind::Int(_) | TyKind::IntLit => true,
            TyKind::Infer(n) => self.infer_kinds[*n as usize] == InferKind::Integer,
            _ => false,
        }
    }
    pub fn is_float(&self, t: TyId) -> bool {
        match self.kind(self.shallow(t)) {
            TyKind::Float(_) | TyKind::FloatLit => true,
            TyKind::Infer(n) => self.infer_kinds[*n as usize] == InferKind::Float,
            _ => false,
        }
    }
    pub fn is_numeric(&self, t: TyId) -> bool {
        self.is_integer(t) || self.is_float(t)
    }
    pub fn as_int(&self, t: TyId) -> Option<IntTy> {
        match self.kind(self.shallow(t)) {
            TyKind::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Substitute generic parameter placeholders (by name) with concrete types.
    pub fn substitute(&mut self, t: TyId, map: &HashMap<String, TyId>) -> TyId {
        let k = self.kind(t).clone();
        match k {
            TyKind::Param(name) => *map.get(&name).unwrap_or(&t),
            TyKind::Struct(d, args) => {
                let args = args.into_iter().map(|a| self.substitute(a, map)).collect();
                self.intern(TyKind::Struct(d, args))
            }
            TyKind::Enum(d, args) => {
                let args = args.into_iter().map(|a| self.substitute(a, map)).collect();
                self.intern(TyKind::Enum(d, args))
            }
            TyKind::Array(n, e) => {
                let e = self.substitute(e, map);
                self.array(n, e)
            }
            TyKind::Slice(m, e) => {
                let e = self.substitute(e, map);
                self.slice(m, e)
            }
            TyKind::Ptr(m, e) => {
                let e = self.substitute(e, map);
                self.ptr(m, e)
            }
            TyKind::Opt(e) => {
                let e = self.substitute(e, map);
                self.opt(e)
            }
            TyKind::Weak(e) => {
                let e = self.substitute(e, map);
                self.intern(TyKind::Weak(e))
            }
            TyKind::ErrUnion(s, e) => {
                let e = self.substitute(e, map);
                self.err_union(s, e)
            }
            TyKind::Fn(ps, r, ef) => {
                let ps = ps.into_iter().map(|p| self.substitute(p, map)).collect();
                let r = self.substitute(r, map);
                self.intern(TyKind::Fn(ps, r, ef))
            }
            TyKind::Tuple(ts) => {
                let ts = ts.into_iter().map(|p| self.substitute(p, map)).collect();
                self.tuple(ts)
            }
            TyKind::List(e) => {
                let e = self.substitute(e, map);
                self.list(e)
            }
            TyKind::Map(k, v) => {
                let k = self.substitute(k, map);
                let v = self.substitute(v, map);
                self.intern(TyKind::Map(k, v))
            }
            _ => t,
        }
    }
}
