//! C backend: lowers the typed IR to a single C translation unit.

pub mod builtins;
pub mod exports;
pub mod expr;
pub mod parallel;

use crate::ast::StructKind;
use crate::check::{Program, StructDef, VariantPayloadDef};
use crate::effects::Effects;
use crate::tir::*;
use crate::types::*;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

pub const RUNTIME_H: &str = include_str!("../../runtime/nx_rt.h");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildMode {
    Debug,
    SafeRelease,
    FastRelease,
    SmallRelease,
}

#[derive(Clone, Debug)]
pub struct GenOptions {
    pub mode: BuildMode,
    pub entry: Entry,
    pub source_names: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    /// a CLI binary with `main`
    Main,
    /// a test runner
    Tests,
    /// a library: exports only, no main
    Library,
}

pub struct Scope {
    pub defers: Vec<(bool, TStmt)>,
    pub drops: Vec<(String, TyId)>,
    pub is_loop_body: bool,
}

pub struct FnState {
    pub inst: InstId,
    pub locals: Vec<String>,
    pub local_tys: Vec<TyId>,
    pub scopes: Vec<Scope>,
    pub ret: TyId,
    pub loop_labels: Vec<(LabelId, usize)>,
    pub block_labels: Vec<(LabelId, String, String)>,
    pub is_test: bool,
}

pub struct Gen {
    pub p: Program,
    pub opts: GenOptions,
    pub types_out: String,
    pub fwd_out: String,
    pub protos_out: String,
    pub helpers_out: String,
    pub data_out: String,
    pub funcs_out: String,
    pub type_names: HashMap<TyId, String>,
    pub emitted_types: HashSet<TyId>,
    pub emitted_names: HashSet<String>,
    pub drop_fns: HashMap<TyId, String>,
    pub eq_fns: HashMap<TyId, String>,
    pub cmp_fns: HashMap<TyId, String>,
    pub thunks: HashMap<InstId, String>,
    pub fn_types: HashMap<(Vec<TyId>, TyId), String>,
    pub tmp: u32,
    pub cur: Option<FnState>,
    pub body: Vec<String>,
    pub sm_names: Vec<String>,
    pub str_lits: HashMap<Vec<u8>, String>,
    pub errors: Vec<String>,
    pub line_starts: Vec<Vec<u32>>,
    pub thunks_by_key: HashMap<String, String>,
    pub lib_name: String,
}

pub fn c_escape_bytes(b: &[u8]) -> String {
    let mut s = String::from("\"");
    for (i, &c) in b.iter().enumerate() {
        match c {
            b'"' => s.push_str("\\\""),
            b'\\' => s.push_str("\\\\"),
            b'\n' => s.push_str("\\n"),
            b'\r' => s.push_str("\\r"),
            b'\t' => s.push_str("\\t"),
            32..=126 => s.push(c as char),
            _ => {
                let _ = write!(s, "\\{:03o}", c);
                // avoid ambiguity with following digits: octal escapes are fixed width
                let _ = i;
            }
        }
    }
    s.push('"');
    s
}

impl Gen {
    pub fn new(p: Program, opts: GenOptions) -> Gen {
        Gen {
            p,
            opts,
            types_out: String::new(),
            fwd_out: String::new(),
            protos_out: String::new(),
            helpers_out: String::new(),
            data_out: String::new(),
            funcs_out: String::new(),
            type_names: HashMap::new(),
            emitted_types: HashSet::new(),
            emitted_names: HashSet::new(),
            drop_fns: HashMap::new(),
            eq_fns: HashMap::new(),
            cmp_fns: HashMap::new(),
            thunks: HashMap::new(),
            fn_types: HashMap::new(),
            tmp: 0,
            cur: None,
            body: Vec::new(),
            sm_names: Vec::new(),
            str_lits: HashMap::new(),
            errors: Vec::new(),
            line_starts: Vec::new(),
            thunks_by_key: HashMap::new(),
            lib_name: "nexium".into(),
        }
    }

    pub fn tmp(&mut self) -> String {
        self.tmp += 1;
        format!("_t{}", self.tmp)
    }

    pub fn res(&mut self, t: TyId) -> TyId {
        self.p.tys.resolve(t, true)
    }

    /// Emit a typedef once per C name (distinct Nexium types may share one C type).
    pub fn typedef(&mut self, name: &str, text: String) {
        if self.emitted_names.insert(name.to_string()) {
            self.types_out.push_str(&text);
            self.types_out.push('\n');
        }
    }

    pub fn kind_of(&mut self, t: TyId) -> TyKind {
        let t = self.res(t);
        self.p.tys.kind(t).clone()
    }

    // ----- type names ----------------------------------------------------------

    pub fn struct_def(&self, t: TyId) -> Option<&StructDef> {
        match self.p.tys.kind(t) {
            TyKind::Struct(d, _) => Some(&self.p.structs[*d as usize]),
            _ => None,
        }
    }

    pub fn is_ref(&self, t: TyId) -> bool {
        matches!(self.struct_def(t), Some(d) if d.kind == StructKind::RefClass)
    }

    pub fn is_void(&self, t: TyId) -> bool {
        matches!(self.p.tys.kind(self.p.tys.shallow(t)), TyKind::Void | TyKind::Never)
    }

    /// A short mangled name usable inside identifiers.
    pub fn mangle(&mut self, t: TyId) -> String {
        let t = self.res(t);
        match self.p.tys.kind(t).clone() {
            TyKind::Int(i) => i.name().to_string(),
            TyKind::Float(FloatTy::F32) => "f32".into(),
            TyKind::Float(FloatTy::F64) => "f64".into(),
            TyKind::Bool => "bool".into(),
            TyKind::Char => "char".into(),
            TyKind::Void | TyKind::Never => "void".into(),
            TyKind::Struct(d, args) | TyKind::Enum(d, args) => {
                let base = match self.p.tys.kind(t) {
                    TyKind::Struct(..) => self.p.structs[d as usize].name.clone(),
                    _ => self.p.enums[d as usize].name.clone(),
                };
                if args.is_empty() {
                    base
                } else {
                    let a: Vec<String> = args.iter().map(|&x| self.mangle(x)).collect();
                    format!("{}__{}", base, a.join("_"))
                }
            }
            TyKind::Array(n, e) => format!("arr{}_{}", n, self.mangle(e)),
            TyKind::Slice(_, e) => format!("sl_{}", self.mangle(e)),
            TyKind::Ptr(_, e) => format!("p_{}", self.mangle(e)),
            TyKind::Opt(e) => format!("opt_{}", self.mangle(e)),
            TyKind::ErrUnion(_, e) => format!("eu_{}", self.mangle(e)),
            TyKind::Fn(ps, r, _) => {
                let a: Vec<String> = ps.iter().map(|&x| self.mangle(x)).collect();
                format!("fn_{}_{}", a.join("_"), self.mangle(r))
            }
            TyKind::Tuple(ts) => {
                let a: Vec<String> = ts.iter().map(|&x| self.mangle(x)).collect();
                format!("tup_{}", a.join("_"))
            }
            TyKind::Distinct(d) => {
                let u = self.p.distinct_underlying[&d];
                self.mangle(u)
            }
            TyKind::Weak(e) => format!("weak_{}", self.mangle(e)),
            TyKind::List(e) => format!("list_{}", self.mangle(e)),
            TyKind::Str => "string".into(),
            TyKind::Map(k, v) => format!("map_{}_{}", self.mangle(k), self.mangle(v)),
            TyKind::ErrorSet(_) => "err".into(),
            TyKind::Dyn(t, _) => format!("dyn_{}", self.p.traits[t as usize].name),
            _ => "unk".into(),
        }
    }

    /// The C type name for a Nexium type, emitting its definition if needed.
    pub fn cty(&mut self, t: TyId) -> String {
        let t = self.res(t);
        if let Some(n) = self.type_names.get(&t) {
            return n.clone();
        }
        let name = match self.p.tys.kind(t).clone() {
            TyKind::Int(i) => i.c_name().to_string(),
            TyKind::Float(FloatTy::F32) => "float".into(),
            TyKind::Float(FloatTy::F64) => "double".into(),
            TyKind::Bool => "bool".into(),
            TyKind::Char => "uint32_t".into(),
            TyKind::Void | TyKind::Never => "void".into(),
            TyKind::Type | TyKind::Namespace(_) | TyKind::Infer(_) | TyKind::IntLit | TyKind::FloatLit | TyKind::Param(_) | TyKind::Closure(_) => "void".into(),
            TyKind::ErrorSet(_) => "uint32_t".into(),
            TyKind::Distinct(d) => {
                let u = self.p.distinct_underlying[&d];
                self.cty(u)
            }
            TyKind::Ptr(_, e) => {
                let en = self.cty_ptr_target(e);
                format!("{}*", en)
            }
            TyKind::Weak(e) => {
                let en = self.cty(e);
                en
            }
            TyKind::Str => "nx_string".into(),
            TyKind::Dyn(tr, _) => {
                let name = format!("nx_dyn_{}", self.p.traits[tr as usize].name);
                self.type_names.insert(t, name.clone());
                self.emit_vtable_type(tr);
                name
            }
            TyKind::Map(..) => "nx_map".into(),
            TyKind::Slice(_, e) => {
                let m = self.mangle(t);
                let name = format!("nx_{}", m);
                if matches!(self.p.tys.kind(self.p.tys.shallow(e)), TyKind::Int(IntTy::U8)) {
                    "nx_sl_u8".to_string()
                } else {
                    let en = self.cty_ptr_target(e);
                    self.type_names.insert(t, name.clone());
                    let _ = writeln!(self.types_out, "typedef struct {n} {{ {e}* ptr; size_t len; }} {n};", n = name, e = en);
                    name
                }
            }
            TyKind::List(e) => {
                let m = self.mangle(t);
                let name = format!("nx_{}", m);
                let en = self.cty_ptr_target(e);
                self.type_names.insert(t, name.clone());
                let _ = writeln!(self.types_out, "typedef struct {n} {{ {e}* ptr; size_t len; size_t cap; struct nx_arena* ar; }} {n};", n = name, e = en);
                name
            }
            TyKind::Array(n, e) => {
                let m = self.mangle(t);
                let name = format!("nx_{}", m);
                let en = self.cty(e);
                self.type_names.insert(t, name.clone());
                let _ = writeln!(self.types_out, "typedef struct {n} {{ {e} v[{c}]; }} {n};", n = name, e = en, c = n.max(1));
                name
            }
            TyKind::Opt(e) => {
                let m = self.mangle(t);
                let name = format!("nx_{}", m);
                self.type_names.insert(t, name.clone());
                if self.is_void(e) {
                    let _ = writeln!(self.types_out, "typedef struct {n} {{ bool has; }} {n};", n = name);
                } else {
                    let en = self.cty(e);
                    let _ = writeln!(self.types_out, "typedef struct {n} {{ bool has; {e} val; }} {n};", n = name, e = en);
                }
                name
            }
            TyKind::ErrUnion(_, e) => {
                let m = self.mangle(t);
                let name = format!("nx_{}", m);
                self.type_names.insert(t, name.clone());
                if self.is_void(e) {
                    let _ = writeln!(self.types_out, "typedef struct {n} {{ uint32_t err; }} {n};", n = name);
                } else {
                    let en = self.cty(e);
                    let _ = writeln!(self.types_out, "typedef struct {n} {{ uint32_t err; {e} val; }} {n};", n = name, e = en);
                }
                name
            }
            TyKind::Tuple(ts) => {
                let m = self.mangle(t);
                let name = format!("nx_{}", m);
                self.type_names.insert(t, name.clone());
                let mut fields = String::new();
                for (i, &e) in ts.iter().enumerate() {
                    let en = self.cty(e);
                    let _ = write!(fields, " {} f{};", en, i);
                }
                self.typedef(&name, format!("typedef struct {n} {{{f} }} {n};", n = name, f = fields));
                name
            }
            TyKind::Fn(ps, r, _) => {
                let key = (ps.clone(), r);
                if let Some(n) = self.fn_types.get(&key) {
                    let n = n.clone();
                    self.type_names.insert(t, n.clone());
                    return n;
                }
                let m = self.mangle(t);
                let name = format!("nx_{}", m);
                self.type_names.insert(t, name.clone());
                self.fn_types.insert(key, name.clone());
                let rn = self.cty(r);
                let mut params = String::from("nx_ctx*, void*");
                for &pt in &ps {
                    let pn = self.cty(pt);
                    let _ = write!(params, ", {}", pn);
                }
                let _ = writeln!(self.types_out, "typedef struct {n} {{ {r} (*fn)({p}); void* env; }} {n};", n = name, r = rn, p = params);
                name
            }
            TyKind::Struct(d, _) => {
                let def = self.p.structs[d as usize].clone();
                // a struct imported from a C header is declared by that header, under its own name
                if let Some(cn) = &def.c_name {
                    self.type_names.insert(t, cn.clone());
                    return cn.clone();
                }
                let m = self.mangle(t);
                let name = format!("nx_{}", m);
                self.type_names.insert(t, name.clone());
                if def.kind == StructKind::RefClass {
                    // pointer to an object with a refcount header
                    let _ = writeln!(self.fwd_out, "typedef struct {n}_obj {n}_obj; typedef {n}_obj* {n};", n = name);
                    let ftys = self.p.struct_field_tys.get(&t).cloned().unwrap_or_default();
                    let mut fields = String::new();
                    for (i, (f, &ft)) in def.fields.iter().zip(ftys.iter()).enumerate() {
                        let fn_ = self.cty(ft);
                        let _ = write!(fields, " {} {}_{};", fn_, sanitize_ident(&f.name), i);
                    }
                    let _ = writeln!(self.types_out, "struct {n}_obj {{ size_t rc; size_t weak;{f} }};", n = name, f = fields);
                } else {
                    let ftys = self.p.struct_field_tys.get(&t).cloned().unwrap_or_default();
                    let mut fields = String::new();
                    for (i, (f, &ft)) in def.fields.iter().zip(ftys.iter()).enumerate() {
                        let fn_ = self.cty(ft);
                        let _ = write!(fields, " {} {}_{};", fn_, sanitize_ident(&f.name), i);
                    }
                    if def.fields.is_empty() {
                        fields.push_str(" char _empty;");
                    }
                    self.typedef(&name, format!("typedef struct {n} {{{f} }} {n};", n = name, f = fields));
                }
                name
            }
            TyKind::Enum(d, _) => {
                let def = self.p.enums[d as usize].clone();
                let m = self.mangle(t);
                let name = format!("nx_{}", m);
                self.type_names.insert(t, name.clone());
                let vtys = self.p.enum_variant_tys.get(&t).cloned().unwrap_or_default();
                let mut union = String::new();
                let mut any = false;
                for (vi, v) in def.variants.iter().enumerate() {
                    let tys = vtys.get(vi).cloned().unwrap_or_default();
                    if tys.is_empty() {
                        continue;
                    }
                    any = true;
                    let mut fields = String::new();
                    for (i, &ft) in tys.iter().enumerate() {
                        let fn_ = self.cty(ft);
                        let _ = write!(fields, " {} f{};", fn_, i);
                    }
                    let _ = write!(union, " struct {{{} }} v{};", fields, vi);
                    let _ = v;
                }
                if any {
                    let _ = writeln!(self.types_out, "typedef struct {n} {{ uint32_t tag; union {{{u} }} u; }} {n};", n = name, u = union);
                } else {
                    let _ = writeln!(self.types_out, "typedef struct {n} {{ uint32_t tag; }} {n};", n = name);
                }
                name
            }
        };
        self.type_names.insert(t, name.clone());
        name
    }

    /// C type used as a pointer target (structs may be incomplete when only pointed to).
    fn cty_ptr_target(&mut self, e: TyId) -> String {
        self.cty(e)
    }

    pub fn needs_drop(&mut self, t: TyId) -> bool {
        let t = self.res(t);
        match self.p.tys.kind(t).clone() {
            TyKind::List(_) | TyKind::Str | TyKind::Map(..) | TyKind::Weak(_) => true,
            TyKind::Struct(d, _) => {
                if self.p.structs[d as usize].kind == StructKind::RefClass {
                    return true;
                }
                let f = self.p.struct_field_tys.get(&t).cloned().unwrap_or_default();
                f.iter().any(|&e| self.needs_drop(e))
            }
            TyKind::Enum(..) => {
                let v = self.p.enum_variant_tys.get(&t).cloned().unwrap_or_default();
                v.iter().flatten().any(|&e| self.needs_drop(e))
            }
            TyKind::Array(_, e) | TyKind::Opt(e) | TyKind::ErrUnion(_, e) => self.needs_drop(e),
            TyKind::Tuple(ts) => ts.iter().any(|&e| self.needs_drop(e)),
            _ => false,
        }
    }

    /// Name of the drop function for a type (generated on demand): `void f(nx_ctx*, T*)`.
    pub fn drop_fn(&mut self, t: TyId) -> String {
        let t = self.res(t);
        if let Some(n) = self.drop_fns.get(&t) {
            return n.clone();
        }
        let m = self.mangle(t);
        let name = format!("nx_drop_{}", m);
        self.drop_fns.insert(t, name.clone());
        let cn = self.cty(t);
        let _ = writeln!(self.protos_out, "static void {}(nx_ctx* c, {}* v);", name, cn);
        let mut body = String::new();
        match self.p.tys.kind(t).clone() {
            TyKind::List(e) => {
                if self.needs_drop(e) {
                    let ed = self.drop_fn(e);
                    let _ = writeln!(body, "  for (size_t i = 0; i < v->len; i++) {}(c, &v->ptr[i]);", ed);
                }
                let en = self.cty(e);
                let _ = writeln!(body, "  nx_list_free(c, (nx_rawlist*)v, sizeof({}));", en);
            }
            TyKind::Str => {
                let _ = writeln!(body, "  nx_str_free(c, v);");
            }
            TyKind::Map(k, vt) => {
                let kd = if self.needs_drop(k) { Some(self.drop_fn(k)) } else { None };
                let vd = if self.needs_drop(vt) { Some(self.drop_fn(vt)) } else { None };
                if kd.is_some() || vd.is_some() {
                    let kn = self.cty(k);
                    let vn = self.cty(vt);
                    let _ = writeln!(body, "  {{ size_t i = 0; void* kp; void* vp; while (nx_map_next(v, &i, &kp, &vp)) {{");
                    if let Some(kd) = kd {
                        let _ = writeln!(body, "    {}(c, ({}*)kp);", kd, kn);
                    }
                    if let Some(vd) = vd {
                        let _ = writeln!(body, "    {}(c, ({}*)vp);", vd, vn);
                    }
                    let _ = writeln!(body, "  }} }}");
                }
                let _ = writeln!(body, "  nx_map_free_storage(c, v);");
            }
            TyKind::Weak(e) => {
                let en = self.cty(e);
                let _ = writeln!(body, "  if (*v) {{ nx_obj_header* h = (nx_obj_header*)*v; h->weak--; if (h->rc == 0 && h->weak == 0) nx_free_bytes(c, *v, sizeof({}_obj)); *v = NULL; }}", en);
            }
            TyKind::Struct(d, _) => {
                let def = self.p.structs[d as usize].clone();
                let ftys = self.p.struct_field_tys.get(&t).cloned().unwrap_or_default();
                if def.kind == StructKind::RefClass {
                    let _ = writeln!(body, "  {}_obj* o = *v; if (!o) return;", cn);
                    let _ = writeln!(body, "  if (--o->rc == 0) {{");
                    for (i, (f, &ft)) in def.fields.iter().zip(ftys.iter()).enumerate() {
                        if self.needs_drop(ft) {
                            let fd = self.drop_fn(ft);
                            let _ = writeln!(body, "    {}(c, &o->{}_{});", fd, sanitize_ident(&f.name), i);
                        }
                    }
                    let _ = writeln!(body, "    if (o->weak == 0) nx_free_bytes(c, o, sizeof({}_obj));", cn);
                    let _ = writeln!(body, "  }}\n  *v = NULL;");
                } else {
                    for (i, (f, &ft)) in def.fields.iter().zip(ftys.iter()).enumerate() {
                        if self.needs_drop(ft) {
                            let fd = self.drop_fn(ft);
                            let _ = writeln!(body, "  {}(c, &v->{}_{});", fd, sanitize_ident(&f.name), i);
                        }
                    }
                }
            }
            TyKind::Enum(..) => {
                let vtys = self.p.enum_variant_tys.get(&t).cloned().unwrap_or_default();
                let _ = writeln!(body, "  switch (v->tag) {{");
                for (vi, tys) in vtys.iter().enumerate() {
                    let mut s = String::new();
                    for (i, &ft) in tys.iter().enumerate() {
                        if self.needs_drop(ft) {
                            let fd = self.drop_fn(ft);
                            let _ = writeln!(s, "    {}(c, &v->u.v{}.f{});", fd, vi, i);
                        }
                    }
                    if !s.is_empty() {
                        let _ = writeln!(body, "  case {}:\n{}    break;", vi, s);
                    }
                }
                let _ = writeln!(body, "  default: break;\n  }}");
            }
            TyKind::Array(n, e) => {
                let ed = self.drop_fn(e);
                let _ = writeln!(body, "  for (size_t i = 0; i < {}; i++) {}(c, &v->v[i]);", n, ed);
            }
            TyKind::Opt(e) => {
                let ed = self.drop_fn(e);
                let _ = writeln!(body, "  if (v->has) {}(c, &v->val);", ed);
            }
            TyKind::ErrUnion(_, e) => {
                let ed = self.drop_fn(e);
                let _ = writeln!(body, "  if (v->err == 0) {}(c, &v->val);", ed);
            }
            TyKind::Tuple(ts) => {
                for (i, &e) in ts.iter().enumerate() {
                    if self.needs_drop(e) {
                        let ed = self.drop_fn(e);
                        let _ = writeln!(body, "  {}(c, &v->f{});", ed, i);
                    }
                }
            }
            _ => {}
        }
        let _ = writeln!(self.helpers_out, "static void {}(nx_ctx* c, {}* v) {{\n  NX_UNUSED(c);\n{}}}", name, cn, body);
        name
    }

    /// Equality function for derived Eq types: `bool f(const T* a, const T* b)`.
    pub fn eq_fn(&mut self, t: TyId) -> String {
        let t = self.res(t);
        if let Some(n) = self.eq_fns.get(&t) {
            return n.clone();
        }
        let m = self.mangle(t);
        let name = format!("nx_eq_{}", m);
        self.eq_fns.insert(t, name.clone());
        let cn = self.cty(t);
        let _ = writeln!(self.protos_out, "static bool {}(const {}* a, const {}* b);", name, cn, cn);
        let mut body = String::new();
        match self.p.tys.kind(t).clone() {
            TyKind::Struct(d, _) => {
                let def = self.p.structs[d as usize].clone();
                let ftys = self.p.struct_field_tys.get(&t).cloned().unwrap_or_default();
                for (i, (f, &ft)) in def.fields.iter().zip(ftys.iter()).enumerate() {
                    let fname = format!("{}_{}", sanitize_ident(&f.name), i);
                    let cmp = self.eq_expr(ft, &format!("a->{}", fname), &format!("b->{}", fname));
                    let _ = writeln!(body, "  if (!({})) return false;", cmp);
                }
                let _ = writeln!(body, "  return true;");
            }
            TyKind::Enum(..) => {
                let vtys = self.p.enum_variant_tys.get(&t).cloned().unwrap_or_default();
                let _ = writeln!(body, "  if (a->tag != b->tag) return false;\n  switch (a->tag) {{");
                for (vi, tys) in vtys.iter().enumerate() {
                    if tys.is_empty() {
                        continue;
                    }
                    let mut s = String::new();
                    for (i, &ft) in tys.iter().enumerate() {
                        let cmp = self.eq_expr(ft, &format!("a->u.v{}.f{}", vi, i), &format!("b->u.v{}.f{}", vi, i));
                        let _ = writeln!(s, "    if (!({})) return false;", cmp);
                    }
                    let _ = writeln!(body, "  case {}:\n{}    break;", vi, s);
                }
                let _ = writeln!(body, "  default: break;\n  }}\n  return true;");
            }
            TyKind::Array(n, e) => {
                let cmp = self.eq_expr(e, "a->v[i]", "b->v[i]");
                let _ = writeln!(body, "  for (size_t i = 0; i < {}; i++) if (!({})) return false;\n  return true;", n, cmp);
            }
            TyKind::Tuple(ts) => {
                for (i, &e) in ts.iter().enumerate() {
                    let cmp = self.eq_expr(e, &format!("a->f{}", i), &format!("b->f{}", i));
                    let _ = writeln!(body, "  if (!({})) return false;", cmp);
                }
                let _ = writeln!(body, "  return true;");
            }
            TyKind::Opt(e) => {
                let cmp = self.eq_expr(e, "a->val", "b->val");
                let _ = writeln!(body, "  if (a->has != b->has) return false;\n  return !a->has || ({});", cmp);
            }
            _ => {
                let _ = writeln!(body, "  return memcmp(a, b, sizeof *a) == 0;");
            }
        }
        let _ = writeln!(self.helpers_out, "static bool {}(const {}* a, const {}* b) {{\n{}}}", name, cn, cn, body);
        name
    }

    /// C expression comparing two values of type `t` given lvalue expressions.
    pub fn eq_expr(&mut self, t: TyId, a: &str, b: &str) -> String {
        let t = self.res(t);
        match self.p.tys.kind(t).clone() {
            TyKind::Int(_) | TyKind::Float(_) | TyKind::Bool | TyKind::Char | TyKind::ErrorSet(_) | TyKind::Ptr(..) => format!("({}) == ({})", a, b),
            TyKind::Distinct(d) => {
                let u = self.p.distinct_underlying[&d];
                self.eq_expr(u, a, b)
            }
            TyKind::Str => format!("nx_sl_eq(nx_str_slice({}), nx_str_slice({}))", a, b),
            TyKind::Slice(_, e) => {
                if matches!(self.p.tys.kind(self.p.tys.shallow(e)), TyKind::Int(IntTy::U8)) {
                    format!("nx_sl_eq({}, {})", a, b)
                } else {
                    let f = self.slice_eq_fn(e);
                    format!("{}({}, {})", f, a, b)
                }
            }
            TyKind::Struct(d, _) if self.p.structs[d as usize].kind == StructKind::RefClass => format!("({}) == ({})", a, b),
            _ => {
                let f = self.eq_fn(t);
                format!("{}(&({}), &({}))", f, a, b)
            }
        }
    }

    fn slice_eq_fn(&mut self, e: TyId) -> String {
        let st = self.p.tys.slice(false, e);
        let st = self.res(st);
        if let Some(n) = self.eq_fns.get(&st) {
            return n.clone();
        }
        let m = self.mangle(st);
        let name = format!("nx_eq_{}", m);
        self.eq_fns.insert(st, name.clone());
        let cn = self.cty(st);
        let cmp = self.eq_expr(e, "a.ptr[i]", "b.ptr[i]");
        let _ = writeln!(self.protos_out, "static bool {}({} a, {} b);", name, cn, cn);
        let _ = writeln!(
            self.helpers_out,
            "static bool {}({} a, {} b) {{\n  if (a.len != b.len) return false;\n  for (size_t i = 0; i < a.len; i++) if (!({})) return false;\n  return true;\n}}",
            name, cn, cn, cmp
        );
        name
    }

    /// Three-way comparison for ordered types: `int f(const T* a, const T* b)`.
    pub fn cmp_expr(&mut self, t: TyId, a: &str, b: &str) -> String {
        let t = self.res(t);
        match self.p.tys.kind(t).clone() {
            TyKind::Int(_) | TyKind::Float(_) | TyKind::Bool | TyKind::Char => format!("(({}) < ({}) ? -1 : (({}) > ({}) ? 1 : 0))", a, b, a, b),
            TyKind::Distinct(d) => {
                let u = self.p.distinct_underlying[&d];
                self.cmp_expr(u, a, b)
            }
            TyKind::Str => format!("nx_sl_cmp(nx_str_slice({}), nx_str_slice({}))", a, b),
            TyKind::Slice(..) => format!("nx_sl_cmp({}, {})", a, b),
            TyKind::Enum(..) => {
                let f = self.cmp_fn(t);
                format!("{}(&({}), &({}))", f, a, b)
            }
            _ => {
                let f = self.cmp_fn(t);
                format!("{}(&({}), &({}))", f, a, b)
            }
        }
    }

    pub fn cmp_fn(&mut self, t: TyId) -> String {
        let t = self.res(t);
        if let Some(n) = self.cmp_fns.get(&t) {
            return n.clone();
        }
        let m = self.mangle(t);
        let name = format!("nx_cmp_{}", m);
        self.cmp_fns.insert(t, name.clone());
        let cn = self.cty(t);
        let _ = writeln!(self.protos_out, "static int {}(const {}* a, const {}* b);", name, cn, cn);
        let mut body = String::new();
        match self.p.tys.kind(t).clone() {
            TyKind::Struct(d, _) => {
                let def = self.p.structs[d as usize].clone();
                let ftys = self.p.struct_field_tys.get(&t).cloned().unwrap_or_default();
                for (i, (f, &ft)) in def.fields.iter().zip(ftys.iter()).enumerate() {
                    let fname = format!("{}_{}", sanitize_ident(&f.name), i);
                    let c = self.cmp_expr(ft, &format!("a->{}", fname), &format!("b->{}", fname));
                    let _ = writeln!(body, "  {{ int r = {}; if (r) return r; }}", c);
                }
                let _ = writeln!(body, "  return 0;");
            }
            TyKind::Enum(..) => {
                let vtys = self.p.enum_variant_tys.get(&t).cloned().unwrap_or_default();
                let _ = writeln!(body, "  if (a->tag != b->tag) return a->tag < b->tag ? -1 : 1;\n  switch (a->tag) {{");
                for (vi, tys) in vtys.iter().enumerate() {
                    if tys.is_empty() {
                        continue;
                    }
                    let mut s = String::new();
                    for (i, &ft) in tys.iter().enumerate() {
                        let c = self.cmp_expr(ft, &format!("a->u.v{}.f{}", vi, i), &format!("b->u.v{}.f{}", vi, i));
                        let _ = writeln!(s, "    {{ int r = {}; if (r) return r; }}", c);
                    }
                    let _ = writeln!(body, "  case {}:\n{}    break;", vi, s);
                }
                let _ = writeln!(body, "  default: break;\n  }}\n  return 0;");
            }
            TyKind::Tuple(ts) => {
                for (i, &e) in ts.iter().enumerate() {
                    let c = self.cmp_expr(e, &format!("a->f{}", i), &format!("b->f{}", i));
                    let _ = writeln!(body, "  {{ int r = {}; if (r) return r; }}", c);
                }
                let _ = writeln!(body, "  return 0;");
            }
            TyKind::Array(n, e) => {
                let c = self.cmp_expr(e, "a->v[i]", "b->v[i]");
                let _ = writeln!(body, "  for (size_t i = 0; i < {}; i++) {{ int r = {}; if (r) return r; }}\n  return 0;", n, c);
            }
            _ => {
                let _ = writeln!(body, "  return memcmp(a, b, sizeof *a);");
            }
        }
        let _ = writeln!(self.helpers_out, "static int {}(const {}* a, const {}* b) {{\n{}}}", name, cn, cn, body);
        name
    }

    // ----- functions -------------------------------------------------------------

    pub fn fn_c_name(&self, inst: InstId) -> String {
        let f = &self.p.funcs[inst as usize];
        if f.is_extern {
            f.name.clone()
        } else {
            f.mangled.clone()
        }
    }

    pub fn fn_signature(&mut self, inst: InstId, with_names: bool) -> String {
        let f = self.p.funcs[inst as usize].clone();
        let ret = if self.is_void(f.ret) { "void".to_string() } else { self.cty(f.ret) };
        let mut params: Vec<String> = Vec::new();
        if !f.is_extern {
            params.push(if with_names { "nx_ctx* c".into() } else { "nx_ctx*".into() });
        }
        if f.is_closure {
            params.push(if with_names { "void* _envp".into() } else { "void*".into() });
        }
        let n_params = f.params.len();
        for (i, &p) in f.params.iter().enumerate() {
            if f.is_closure && i >= n_params {
                break;
            }
            let pt = f.locals[p as usize].ty;
            let cn = self.cty(pt);
            if with_names {
                params.push(format!("{} {}", cn, local_c_name(&f.locals[p as usize].name, p)));
            } else {
                params.push(cn);
            }
        }
        let name = self.fn_c_name(inst);
        if f.is_variadic {
            params.push("...".into());
        }
        let linkage = if f.is_extern {
            "extern "
        } else if f.export.is_some() && self.opts.entry == Entry::Library {
            "static "
        } else {
            "static "
        };
        format!("{}{} {}({})", linkage, ret, name, if params.is_empty() { "void".into() } else { params.join(", ") })
    }

    /// The vtable struct of a trait and the fat pointer type over it.
    pub fn emit_vtable_type(&mut self, tr: DefId) {
        let tname = self.p.traits[tr as usize].name.clone();
        let key = format!("vtable:{}", tr);
        if self.thunks_by_key.contains_key(&key) {
            return;
        }
        self.thunks_by_key.insert(key, tname.clone());
        let methods = self.p.traits[tr as usize].methods.clone();
        let mut fields = String::new();
        // method signatures are taken from any vtable instance; without one, emit generic pointers
        let sample = self.p.vtables.iter().find(|(t, _, _)| *t == tr).map(|(_, _, insts)| insts.clone());
        for (i, m) in methods.iter().enumerate() {
            let sig = match &sample {
                Some(insts) if i < insts.len() => {
                    let f = self.p.funcs[insts[i] as usize].clone();
                    let ret = if self.is_void(f.ret) { "void".to_string() } else { self.cty(f.ret) };
                    let mut params = vec!["nx_ctx*".to_string(), "void*".to_string()];
                    for &p in f.params.iter().skip(1) {
                        let cn = self.cty(f.locals[p as usize].ty);
                        params.push(cn);
                    }
                    format!("{} (*{})({})", ret, sanitize_ident(&m.name), params.join(", "))
                }
                _ => format!("void (*{})(nx_ctx*, void*)", sanitize_ident(&m.name)),
            };
            let _ = write!(fields, " {};", sig);
        }
        let _ = writeln!(self.fwd_out, "typedef struct nx_vt_{n} nx_vt_{n};\ntypedef struct nx_dyn_{n} {{ void* data; const nx_vt_{n}* vt; }} nx_dyn_{n};", n = tname);
        let _ = writeln!(self.types_out, "struct nx_vt_{n} {{{f} }};", n = tname, f = fields);
    }

    /// The vtable instance for (trait, type): thunks adapt `void* self` to the receiver type.
    pub fn vtable_instance(&mut self, vt: u32) -> String {
        let (tr, ty, insts) = self.p.vtables[vt as usize].clone();
        let key = format!("vtinst:{}", vt);
        if let Some(n) = self.thunks_by_key.get(&key) {
            return n.clone();
        }
        let tname = self.p.traits[tr as usize].name.clone();
        let name = format!("nx_vt_{}_{}", tname, self.mangle(ty));
        self.thunks_by_key.insert(key, name.clone());
        self.emit_vtable_type(tr);
        let mut entries = Vec::new();
        for (i, &inst) in insts.iter().enumerate() {
            let f = self.p.funcs[inst as usize].clone();
            let thunk = format!("nx_vtthunk_{}_{}", vt, i);
            let ret = if self.is_void(f.ret) { "void".to_string() } else { self.cty(f.ret) };
            let mut params = vec!["nx_ctx* c".to_string(), "void* self".to_string()];
            let mut args = vec!["c".to_string()];
            let recv_ty = f.locals[f.params[0] as usize].ty;
            let recv_c = self.cty(recv_ty);
            let tcn = self.cty(ty);
            match self.kind_of(recv_ty) {
                TyKind::Ptr(..) => args.push(format!("({})self", recv_c)),
                _ => args.push(format!("*({}*)self", tcn)),
            }
            for (k, &p) in f.params.iter().enumerate().skip(1) {
                let cn = self.cty(f.locals[p as usize].ty);
                params.push(format!("{} a{}", cn, k));
                args.push(format!("a{}", k));
            }
            let target = self.fn_c_name(inst);
            let _ = writeln!(self.helpers_out, "static {} {}({}) {{ {}{}({}); }}", ret, thunk, params.join(", "), if ret == "void" { "" } else { "return " }, target, args.join(", "));
            entries.push(format!(".{} = {}", sanitize_ident(&self.p.traits[tr as usize].methods[i].name), thunk));
        }
        let _ = writeln!(self.helpers_out, "static const nx_vt_{} {} = {{ {} }};", tname, name, entries.join(", "));
        name
    }

    /// Thunk adapting a plain function to the fat function-value calling convention.
    pub fn thunk(&mut self, inst: InstId) -> String {
        if let Some(n) = self.thunks.get(&inst) {
            return n.clone();
        }
        let f = self.p.funcs[inst as usize].clone();
        let name = format!("nx_thunk_{}", inst);
        self.thunks.insert(inst, name.clone());
        let ret = if self.is_void(f.ret) { "void".to_string() } else { self.cty(f.ret) };
        let mut params = vec!["nx_ctx* c".to_string(), "void* env".to_string()];
        let mut args = vec!["c".to_string()];
        if f.is_closure {
            args.push("env".into());
        }
        for (i, &p) in f.params.iter().enumerate() {
            let cn = self.cty(f.locals[p as usize].ty);
            params.push(format!("{} a{}", cn, i));
            args.push(format!("a{}", i));
        }
        if f.is_extern {
            args.remove(0);
        }
        let target = self.fn_c_name(inst);
        let _ = writeln!(self.protos_out, "static {} {}({});", ret, name, params.join(", "));
        let _ = writeln!(self.helpers_out, "static {} {}({}) {{ NX_UNUSED(env); {}{}({}); }}", ret, name, params.join(", "), if ret == "void" { "" } else { "return " }, target, args.join(", "));
        name
    }

    // ----- program ---------------------------------------------------------------

    pub fn generate(mut self) -> Result<String, Vec<String>> {
        // error names
        let mut names = String::from("static const char* const nx_error_names[] = { \"(ok)\"");
        for n in &self.p.error_names {
            let _ = write!(names, ", \"{}\"", n);
        }
        names.push_str(" };\nNX_INLINE const char* nx_error_name(uint32_t e) { return e <= sizeof(nx_error_names)/sizeof(*nx_error_names) - 1 ? nx_error_names[e] : \"(unknown error)\"; }\n");
        self.data_out.push_str(&names);

        // consts and globals
        for i in 0..self.p.consts.len() {
            let c = self.p.consts[i].clone();
            if let Some((ty, te)) = &c.resolved {
                if let TExprKind::Value(v) = &te.kind {
                    let ty = *ty;
                    let cn = self.cty(ty);
                    match self.static_init(v, ty) {
                        Some(init) => {
                            let _ = writeln!(self.data_out, "static const {} {} = {};", cn, c.mangled, init);
                        }
                        None => {}
                    }
                }
            }
        }
        for i in 0..self.p.globals.len() {
            let g = self.p.globals[i].clone();
            if let Some(ty) = g.ty {
                let cn = self.cty(ty);
                let init = match &g.init {
                    Some(TExpr { kind: TExprKind::Value(v), .. }) => self.static_init(v, ty),
                    _ => None,
                };
                let align = g.align.map(|a| format!(" __attribute__((aligned({})))", a)).unwrap_or_default();
                match init {
                    Some(init) => {
                        let _ = writeln!(self.data_out, "static {} {}{} = {};", cn, g.mangled, align, init);
                    }
                    None => {
                        let _ = writeln!(self.data_out, "static {} {}{};", cn, g.mangled, align);
                    }
                }
            }
        }

        // prototypes for all functions with bodies (and externs)
        let n = self.p.funcs.len();
        for i in 0..n {
            let f = &self.p.funcs[i];
            if f.body.is_none() && !f.is_extern {
                continue;
            }
            if f.cimport {
                continue; // declared by the included header
            }
            if f.is_extern {
                let sig = self.fn_signature(i as InstId, false);
                let _ = writeln!(self.protos_out, "{};", sig);
            } else {
                let sig = self.fn_signature(i as InstId, false);
                let _ = writeln!(self.protos_out, "{};", sig);
            }
        }
        // bodies
        for i in 0..n {
            if self.p.funcs[i].body.is_some() {
                self.emit_function(i as InstId);
            }
        }
        // exports and entry
        let mut tail = String::new();
        if self.opts.entry == Entry::Library || !self.p.exports.is_empty() {
            for &e in &self.p.exports.clone() {
                let w = self.export_wrapper(e);
                tail.push_str(&w);
            }
        }
        match self.opts.entry {
            Entry::Main => tail.push_str(&self.main_wrapper()),
            Entry::Tests => tail.push_str(&self.test_runner()),
            Entry::Library => {}
        }
        if !self.errors.is_empty() {
            return Err(self.errors);
        }
        let mut out = String::new();
        out.push_str("/* generated by nx */\n");
        let _ = writeln!(
            out,
            "#define NX_MODE_{}",
            match self.opts.mode {
                BuildMode::Debug => "DEBUG",
                BuildMode::SafeRelease => "SAFE",
                BuildMode::FastRelease => "FAST",
                BuildMode::SmallRelease => "SMALL",
            }
        );
        out.push_str(RUNTIME_H);
        out.push_str("\n/* ---- imported C headers ---- */\n");
        for (h, sys) in &self.p.cimport_headers {
            if *sys {
                let _ = writeln!(out, "#include <{}>", h);
            } else {
                let _ = writeln!(out, "#include \"{}\"", h);
            }
        }
        out.push_str("\n/* ---- forward declarations ---- */\n");
        out.push_str(&self.fwd_out);
        out.push_str("\n/* ---- types ---- */\n");
        {
            let mut seen: HashSet<&str> = HashSet::new();
            for line in self.types_out.lines() {
                if seen.insert(line) {
                    out.push_str(line);
                    out.push('\n');
                }
            }
        }
        out.push_str("\n/* ---- data ---- */\n");
        out.push_str(&self.data_out);
        out.push_str("\n/* ---- prototypes ---- */\n");
        out.push_str(&self.protos_out);
        out.push_str("\n/* ---- helpers ---- */\n");
        out.push_str(&self.helpers_out);
        out.push_str("\n/* ---- functions ---- */\n");
        out.push_str(&self.funcs_out);
        out.push_str("\n/* ---- entry ---- */\n");
        out.push_str(&tail);
        Ok(out)
    }

    /// A C static initializer for a compile-time value.
    pub fn static_init(&mut self, v: &Value, ty: TyId) -> Option<String> {
        let ty = self.res(ty);
        let k = self.p.tys.kind(ty).clone();
        Some(match (v, k) {
            (Value::Int(i), TyKind::Int(it)) => int_literal(*i, it),
            (Value::Int(i), TyKind::Char) => format!("{}u", i),
            (Value::Int(i), TyKind::Float(_)) => float_literal(*i as f64),
            (Value::Char(c), TyKind::Char) => format!("{}u", c),
            (Value::Char(c), TyKind::Int(it)) => int_literal(*c as i128, it),
            (Value::Float(f), TyKind::Float(_)) => float_literal(*f),
            (Value::Bool(b), TyKind::Bool) => {
                if *b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            (v, TyKind::Distinct(d)) => {
                let u = self.p.distinct_underlying[&d];
                return self.static_init(v, u);
            }
            (Value::Str(s), TyKind::Slice(..)) => {
                let lit = self.string_literal(s);
                format!("{{ (uint8_t*){}, {} }}", lit, s.len())
            }
            (Value::Array(a), TyKind::Array(_, e)) => {
                let mut parts = Vec::new();
                for x in a {
                    parts.push(self.static_init(x, e)?);
                }
                format!("{{ {{ {} }} }}", parts.join(", "))
            }
            (Value::Str(s), TyKind::Array(_, _)) => {
                let parts: Vec<String> = s.iter().map(|b| b.to_string()).collect();
                format!("{{ {{ {} }} }}", parts.join(", "))
            }
            (Value::Struct(fs), TyKind::Struct(d, _)) => {
                let def = self.p.structs[d as usize].clone();
                if def.kind == StructKind::RefClass {
                    return None;
                }
                let ftys = self.p.struct_field_tys.get(&ty).cloned().unwrap_or_default();
                let mut parts = Vec::new();
                for (i, (f, &ft)) in def.fields.iter().zip(ftys.iter()).enumerate() {
                    let v = fs.get(i)?;
                    let init = self.static_init(v, ft)?;
                    parts.push(format!(".{}_{} = {}", sanitize_ident(&f.name), i, init));
                }
                format!("{{ {} }}", parts.join(", "))
            }
            (Value::Tuple(vs), TyKind::Tuple(ts)) => {
                let mut parts = Vec::new();
                for (i, (v, &t)) in vs.iter().zip(ts.iter()).enumerate() {
                    parts.push(format!(".f{} = {}", i, self.static_init(v, t)?));
                }
                format!("{{ {} }}", parts.join(", "))
            }
            (Value::Enum(vi, payload), TyKind::Enum(..)) => {
                let vtys = self.p.enum_variant_tys.get(&ty).cloned().unwrap_or_default();
                let tys = vtys.get(*vi as usize).cloned().unwrap_or_default();
                if tys.is_empty() {
                    format!("{{ .tag = {} }}", vi)
                } else {
                    let mut parts = Vec::new();
                    for (i, (v, &t)) in payload.iter().zip(tys.iter()).enumerate() {
                        parts.push(format!(".f{} = {}", i, self.static_init(v, t)?));
                    }
                    format!("{{ .tag = {}, .u = {{ .v{} = {{ {} }} }} }}", vi, vi, parts.join(", "))
                }
            }
            (Value::Opt(None), TyKind::Opt(_)) => "{ .has = false }".into(),
            (Value::Opt(Some(v)), TyKind::Opt(e)) => format!("{{ .has = true, .val = {} }}", self.static_init(v, e)?),
            (Value::Err(id), TyKind::ErrUnion(..)) => format!("{{ .err = {} }}", id),
            (Value::Ok(v), TyKind::ErrUnion(_, e)) => {
                if self.is_void(e) {
                    "{ .err = 0 }".into()
                } else {
                    format!("{{ .err = 0, .val = {} }}", self.static_init(v, e)?)
                }
            }
            (Value::Err(id), TyKind::ErrorSet(_)) => format!("{}u", id),
            (Value::Void, _) => "0".into(),
            (Value::Undefined, _) => "{0}".into(),
            _ => {
                self.errors.push(format!("cannot emit a compile-time value of type `{}` as static data", self.p_type_name(ty)));
                return None;
            }
        })
    }

    pub fn p_type_name(&self, t: TyId) -> String {
        format!("{:?}", self.p.tys.kind(self.p.tys.shallow(t)))
    }

    /// Interned string literal: returns a C expression of type `const char*`.
    pub fn string_literal(&mut self, s: &[u8]) -> String {
        if let Some(n) = self.str_lits.get(s) {
            return n.clone();
        }
        let name = format!("nx_str_{}", self.str_lits.len());
        let esc = c_escape_bytes(s);
        let _ = writeln!(self.data_out, "static const char {}[{}] = {};", name, s.len() + 1, esc);
        self.str_lits.insert(s.to_vec(), name.clone());
        name
    }
}

pub fn sanitize_ident(s: &str) -> String {
    let mut out: String = s.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    if out.is_empty() {
        out.push('_');
    }
    out
}

pub fn local_c_name(name: &str, id: LocalId) -> String {
    format!("{}_{}", sanitize_ident(name), id)
}

pub fn int_literal(v: i128, it: IntTy) -> String {
    match it {
        IntTy::I128 => {
            if v == i64::MIN as i128 {
                "((nx_i128)INT64_MIN)".to_string()
            } else if v > i64::MIN as i128 && v <= i64::MAX as i128 {
                format!("((nx_i128){}LL)", v)
            } else {
                let hi = (v >> 64) as i64;
                let lo = v as u64;
                format!("(((nx_i128){}LL << 64) | (nx_u128){}ULL)", hi, lo)
            }
        }
        IntTy::U128 => {
            if v <= u64::MAX as i128 {
                format!("((nx_u128){}ULL)", v)
            } else {
                let hi = (v >> 64) as u64;
                let lo = v as u64;
                format!("(((nx_u128){}ULL << 64) | (nx_u128){}ULL)", hi, lo)
            }
        }
        _ if it.is_signed() => {
            if v == i64::MIN as i128 {
                "INT64_MIN".to_string()
            } else {
                format!("(({}){}LL)", it.c_name(), v)
            }
        }
        _ => format!("(({}){}ULL)", it.c_name(), v),
    }
}

pub fn float_literal(f: f64) -> String {
    if f.is_nan() {
        "NAN".into()
    } else if f.is_infinite() {
        if f > 0.0 {
            "INFINITY".into()
        } else {
            "(-INFINITY)".into()
        }
    } else {
        format!("{:?}", f)
    }
}

pub fn effects_of(p: &Program, inst: InstId) -> Effects {
    p.funcs[inst as usize].effects
}

pub fn variant_payload_len(def: &crate::check::EnumDef, vi: usize) -> usize {
    match &def.variants[vi].payload {
        VariantPayloadDef::Unit => 0,
        VariantPayloadDef::Tuple(t) => t.len(),
        VariantPayloadDef::Struct(f) => f.len(),
    }
}
