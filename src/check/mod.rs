//! Semantic analysis: name resolution, type checking, monomorphization,
//! effect collection. Produces the typed IR in `crate::tir`.

pub mod binpat;
pub mod effects;
pub mod expr;
pub mod method;
pub mod pattern;
pub mod stmt;

use crate::ast::*;
use crate::diag::{Diag, SourceMap, Span};
use crate::effects::Effects;
use crate::tir::*;
use crate::types::*;
use std::collections::{HashMap, HashSet};

pub type FnDefId = u32;

#[derive(Clone, Debug)]
pub struct FieldDef {
    pub name: String,
    pub ty_expr: TypeExpr,
    pub default: Option<Expr>,
    pub constraint: Option<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct StructDef {
    pub name: String,
    pub kind: StructKind,
    pub layout: Layout,
    pub type_params: Vec<String>,
    pub fields: Vec<FieldDef>,
    pub derives: Vec<String>,
    pub module: u32,
    pub span: Span,
    pub is_pub: bool,
    /// for structs imported from C: how the C type is spelled
    pub c_name: Option<String>,
}

#[derive(Clone, Debug)]
pub enum VariantPayloadDef {
    Unit,
    Tuple(Vec<TypeExpr>),
    Struct(Vec<FieldDef>),
}

#[derive(Clone, Debug)]
pub struct VariantDef {
    pub name: String,
    pub payload: VariantPayloadDef,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub variants: Vec<VariantDef>,
    pub derives: Vec<String>,
    pub module: u32,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FnDef {
    pub decl: FnDecl,
    pub module: u32,
    /// for methods: the impl this belongs to
    pub impl_id: Option<u32>,
    pub is_generic: bool,
    /// names of comptime type parameters, in order
    pub type_params: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ImplDef {
    pub trait_name: Option<String>,
    pub target: TypeExpr,
    pub type_params: Vec<String>,
    pub methods: Vec<(String, FnDefId)>,
    pub assoc_types: Vec<(String, TypeExpr)>,
    pub module: u32,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct TraitDef {
    pub name: String,
    pub methods: Vec<FnDecl>,
    pub assoc_types: Vec<String>,
    pub module: u32,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ConstDef {
    pub name: String,
    pub ty_expr: Option<TypeExpr>,
    pub value: Expr,
    pub module: u32,
    pub span: Span,
    pub resolved: Option<(TyId, TExpr)>,
    pub in_progress: bool,
    pub is_pub: bool,
    pub mangled: String,
}

#[derive(Clone, Debug)]
pub struct GlobalDef {
    pub name: String,
    pub ty_expr: TypeExpr,
    pub value: Option<Expr>,
    pub module: u32,
    pub span: Span,
    pub ty: Option<TyId>,
    pub init: Option<TExpr>,
    pub mangled: String,
    pub align: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct AliasDef {
    pub name: String,
    pub distinct: bool,
    pub ty_expr: TypeExpr,
    pub module: u32,
    pub resolved: Option<TyId>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ErrorSetDef {
    pub name: String,
    pub ids: Vec<u32>,
    pub module: u32,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ItemRef {
    Struct(DefId),
    Enum(DefId),
    Fn(FnDefId),
    Const(DefId),
    Global(DefId),
    Alias(DefId),
    ErrorSet(DefId),
    Trait(DefId),
    Module(u32),
}

#[derive(Clone, Debug)]
pub struct ArtifactInfo {
    pub kind: String,
    pub fields: Vec<(String, ArtifactValue)>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ArtifactValue {
    Str(String),
    Ident(String),
    List(Vec<ArtifactValue>),
    Bool(bool),
    Int(i128),
}

/// A scope entry: name -> local, with auto-deref for by-reference captures.
#[derive(Clone, Copy, Debug)]
pub struct ScopeEntry {
    pub local: LocalId,
    pub auto_deref: bool,
}

#[derive(Clone, Debug)]
pub struct LabelInfo {
    pub name: Option<String>,
    pub id: LabelId,
    pub is_loop: bool,
    pub break_ty: Option<TyId>,
    pub scope_depth: usize,
}

/// Per-function checking state.
pub struct FnCtx {
    pub locals: Vec<Local>,
    pub scopes: Vec<Vec<(String, ScopeEntry)>>,
    pub ret: TyId,
    pub labels: Vec<LabelInfo>,
    pub own_effects: Effects,
    pub witnesses: Vec<(Effects, Span, String)>,
    pub callees: Vec<(InstId, Span)>,
    pub moved: HashSet<LocalId>,
    pub moved_spans: HashMap<LocalId, Span>,
    /// inside a `match p.*`: bind owning payloads as `*T` / `*mut T` (the pointer's mutability)
    pub pat_by_ref: Option<bool>,
    pub generics: HashMap<String, TyId>,
    pub self_ty: Option<TyId>,
    pub unsafe_depth: u32,
    pub module: u32,
    /// known integer ranges for locals (range analysis, E6)
    pub ranges: HashMap<LocalId, (i128, i128)>,
    /// index variables that are known to be in range for a given slice local
    pub index_of: HashMap<LocalId, LocalId>,
    pub is_comptime: bool,
    pub in_defer: bool,
    pub in_parallel: u32,
    pub fn_name: String,
    pub next_label: u32,
    pub loop_depth: u32,
}

impl FnCtx {
    pub fn new(ret: TyId, module: u32, name: &str) -> FnCtx {
        FnCtx {
            locals: Vec::new(),
            scopes: vec![Vec::new()],
            ret,
            labels: Vec::new(),
            own_effects: Effects::NONE,
            witnesses: Vec::new(),
            callees: Vec::new(),
            moved: HashSet::new(),
            moved_spans: HashMap::new(),
            pat_by_ref: None,
            generics: HashMap::new(),
            self_ty: None,
            unsafe_depth: 0,
            module,
            ranges: HashMap::new(),
            index_of: HashMap::new(),
            is_comptime: false,
            in_defer: false,
            in_parallel: 0,
            fn_name: name.to_string(),
            next_label: 0,
            loop_depth: 0,
        }
    }
    pub fn lookup(&self, name: &str) -> Option<ScopeEntry> {
        for scope in self.scopes.iter().rev() {
            for (n, e) in scope.iter().rev() {
                if n == name {
                    return Some(*e);
                }
            }
        }
        None
    }
}

pub struct Program {
    /// set by `nx repl`: the outcome of running the new statements
    pub repl: Option<crate::tir::ReplOutcome>,
    pub funcs: Vec<TFunc>,
    pub structs: Vec<StructDef>,
    pub enums: Vec<EnumDef>,
    pub consts: Vec<ConstDef>,
    pub globals: Vec<GlobalDef>,
    pub error_names: Vec<String>,
    pub artifacts: Vec<ArtifactInfo>,
    pub tys: TyTable,
    pub main: Option<InstId>,
    pub tests: Vec<InstId>,
    pub struct_field_tys: HashMap<TyId, Vec<TyId>>,
    pub enum_variant_tys: HashMap<TyId, Vec<Vec<TyId>>>,
    pub closure_envs: HashMap<InstId, Vec<(TyId, bool)>>,
    pub exports: Vec<InstId>,
    pub aliases: Vec<AliasDef>,
    pub distinct_underlying: HashMap<DefId, TyId>,
    pub modules: Vec<String>,
    pub used_tys: Vec<TyId>,
    /// vtables: (trait, concrete type, method instances in trait declaration order)
    pub vtables: Vec<(DefId, TyId, Vec<InstId>)>,
    pub traits: Vec<TraitDef>,
    /// headers imported with `@cImport`: (name, is_system)
    pub cimport_headers: Vec<(String, bool)>,
}

pub struct Checker<'a> {
    pub sm: &'a SourceMap,
    pub tys: TyTable,
    pub diags: Vec<Diag>,
    pub structs: Vec<StructDef>,
    pub enums: Vec<EnumDef>,
    pub fns: Vec<FnDef>,
    pub impls: Vec<ImplDef>,
    pub traits: Vec<TraitDef>,
    pub consts: Vec<ConstDef>,
    pub globals: Vec<GlobalDef>,
    pub aliases: Vec<AliasDef>,
    pub error_sets: Vec<ErrorSetDef>,
    pub items: HashMap<(u32, String), ItemRef>,
    pub module_names: Vec<String>,
    pub module_imports: HashMap<(u32, String), (u32, Option<String>)>,
    pub error_names: Vec<String>,
    pub error_ids: HashMap<String, u32>,
    pub funcs: Vec<TFunc>,
    pub inst_map: HashMap<(FnDefId, Vec<TyId>), InstId>,
    pub queue: Vec<(InstId, FnDefId, Vec<TyId>)>,
    pub artifacts: Vec<ArtifactInfo>,
    pub main: Option<InstId>,
    pub tests: Vec<InstId>,
    pub struct_field_tys: HashMap<TyId, Vec<TyId>>,
    pub enum_variant_tys: HashMap<TyId, Vec<Vec<TyId>>>,
    pub closure_envs: HashMap<InstId, Vec<(TyId, bool)>>,
    pub distinct_underlying: HashMap<DefId, TyId>,
    pub exports: Vec<InstId>,
    pub cur: Option<FnCtx>,
    pub embedded_mode: bool,
    pub closure_counter: u32,
    pub used_tys: Vec<TyId>,
    /// obligations: closure/function value must satisfy negative bounds
    pub fn_value_obligations: Vec<(InstId, Effects, Span)>,
    /// pending closure bodies: (inst, closure ast, captures, generics, self ty, module)
    pub closure_queue: Vec<(InstId, ClosureExpr, Vec<(LocalId, bool, TyId)>, HashMap<String, TyId>, Option<TyId>, u32)>,
    pub global_uses: Vec<(DefId, Span, String)>,
    pub inst_generics: HashMap<InstId, (HashMap<String, TyId>, Option<TyId>)>,
    pub vtables: Vec<(DefId, TyId, Vec<InstId>)>,
    /// calls made inside `for parallel` bodies: (callee, call site); checked for `shared_mutable`
    pub parallel_obligations: Vec<(InstId, Span)>,
    pub source_dirs: Vec<String>,
    pub cimport_opts: Option<crate::cimport::ImportOptions>,
    pub cimport_headers: Vec<(String, bool)>,
    pub cimport_unsupported: HashMap<(u32, String), String>,
    /// `nx repl`: unused values in `main` are shown instead of rejected, and the
    /// interpreter may perform I/O
    pub repl_mode: bool,
    /// `nx repl`: run `main`'s statements from this index with these seed bindings
    pub repl_request: Option<(usize, Vec<(String, Value)>)>,
    pub record_new_mode: bool,
}

impl<'a> Checker<'a> {
    pub fn new(sm: &'a SourceMap) -> Self {
        Checker {
            sm,
            tys: TyTable::default(),
            diags: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            fns: Vec::new(),
            impls: Vec::new(),
            traits: Vec::new(),
            consts: Vec::new(),
            globals: Vec::new(),
            aliases: Vec::new(),
            error_sets: Vec::new(),
            items: HashMap::new(),
            module_names: Vec::new(),
            module_imports: HashMap::new(),
            error_names: Vec::new(),
            error_ids: HashMap::new(),
            funcs: Vec::new(),
            inst_map: HashMap::new(),
            queue: Vec::new(),
            artifacts: Vec::new(),
            main: None,
            tests: Vec::new(),
            struct_field_tys: HashMap::new(),
            enum_variant_tys: HashMap::new(),
            closure_envs: HashMap::new(),
            distinct_underlying: HashMap::new(),
            exports: Vec::new(),
            cur: None,
            embedded_mode: false,
            closure_counter: 0,
            used_tys: Vec::new(),
            fn_value_obligations: Vec::new(),
            closure_queue: Vec::new(),
            global_uses: Vec::new(),
            inst_generics: HashMap::new(),
            vtables: Vec::new(),
            parallel_obligations: Vec::new(),
            source_dirs: Vec::new(),
            cimport_opts: None,
            cimport_headers: Vec::new(),
            cimport_unsupported: HashMap::new(),
            repl_mode: false,
            repl_request: None,
            record_new_mode: false,
        }
    }

    // ----- diagnostics ------------------------------------------------------

    pub fn error(&mut self, span: Span, msg: impl Into<String>) {
        self.diags.push(Diag::error(span, msg));
    }
    pub fn error_note(&mut self, span: Span, msg: impl Into<String>, nspan: Option<Span>, note: impl Into<String>) {
        self.diags.push(Diag::error(span, msg).note(nspan, note));
    }
    pub fn warn(&mut self, span: Span, msg: impl Into<String>) {
        self.diags.push(Diag::warning(span, msg));
    }
    pub fn has_errors(&self) -> bool {
        self.diags.iter().any(|d| d.level == crate::diag::Level::Error)
    }

    pub fn cur(&mut self) -> &mut FnCtx {
        self.cur.as_mut().expect("no function context")
    }

    pub fn add_effect(&mut self, e: Effects, span: Span, why: &str) {
        let cur = self.cur.as_mut().expect("no function context");
        if !cur.own_effects.contains(e) || cur.witnesses.iter().all(|(w, _, _)| !w.contains(e)) {
            cur.witnesses.push((e, span, why.to_string()));
        }
        cur.own_effects = cur.own_effects.union(e);
    }

    // ----- errors -------------------------------------------------------------

    pub fn error_id(&mut self, name: &str) -> u32 {
        if let Some(&id) = self.error_ids.get(name) {
            return id;
        }
        self.error_names.push(name.to_string());
        let id = self.error_names.len() as u32; // 1-based
        self.error_ids.insert(name.to_string(), id);
        id
    }

    // ----- collection -----------------------------------------------------

    pub fn collect(&mut self, modules: &[Module], names: &[String]) {
        self.module_names = names.to_vec();
        // `const c = @cImport("header.h")` becomes a synthetic module
        for m in modules {
            for item in &m.items {
                if let Item::Const(c) = item {
                    if let Expr::Builtin { name, args, span } = &c.value {
                        if name == "cImport" {
                            self.collect_cimport(&c.name, args, *span, m.file);
                        }
                    }
                }
            }
        }
        for m in modules {
            for item in &m.items {
                if let Item::Const(c) = item {
                    if matches!(&c.value, Expr::Builtin { name, .. } if name == "cImport") {
                        continue;
                    }
                }
                self.collect_item(item, m.file);
            }
        }
        // reserve a few well-known errors so ids are stable
        for e in ["OutOfMemory", "Panic", "InvalidRecord", "Truncated", "Overflow", "InvalidUtf8", "NotFound", "IoError", "InvalidInput", "BufferTooSmall", "Timeout", "ConnectionRefused"] {
            self.error_id(e);
        }
    }

    fn collect_cimport(&mut self, const_name: &str, args: &[Expr], span: Span, module: u32) {
        let header = match args.first() {
            Some(Expr::Lit { value: Lit::Str(s), .. }) if args.len() == 1 => String::from_utf8_lossy(s).to_string(),
            _ => {
                self.error(span, "@cImport takes one string literal: the header to import, e.g. `@cImport(\"stdio.h\")`");
                return;
            }
        };
        let opts = match &self.cimport_opts {
            Some(o) => o,
            None => {
                self.error(span, "@cImport is not available in this context");
                return;
            }
        };
        let ci = match crate::cimport::import(&header, opts, span) {
            Ok(ci) => ci,
            Err(e) => {
                self.error(span, format!("@cImport(\"{}\"): {}", header, e));
                return;
            }
        };
        let mid = self.module_names.len() as u32;
        self.module_names.push(format!("cimport:{}", header));
        self.cimport_headers.push((header.clone(), ci.is_system));
        let attrs = Attrs { is_pub: true, doc: vec![] };
        for (name, c_name, fields) in &ci.structs {
            let id = self.structs.len() as DefId;
            self.structs.push(StructDef {
                name: name.clone(),
                kind: StructKind::Struct,
                layout: Layout::C,
                type_params: vec![],
                fields: fields.iter().map(|(n, t)| FieldDef { name: n.clone(), ty_expr: t.clone(), default: None, constraint: None, span }).collect(),
                derives: vec![],
                module: mid,
                span,
                is_pub: true,
                c_name: Some(c_name.clone()),
            });
            self.items.insert((mid, name.clone()), ItemRef::Struct(id));
        }
        for (name, ty) in &ci.aliases {
            if self.items.contains_key(&(mid, name.clone())) {
                continue;
            }
            let id = self.aliases.len() as DefId;
            self.aliases.push(AliasDef { name: name.clone(), distinct: false, ty_expr: ty.clone(), module: mid, resolved: None, span });
            self.items.insert((mid, name.clone()), ItemRef::Alias(id));
        }
        for f in &ci.functions {
            if self.items.contains_key(&(mid, f.name.clone())) {
                continue;
            }
            let decl = FnDecl {
                attrs: attrs.clone(),
                name: f.name.clone(),
                params: f.params.iter().map(|(n, t)| Param { name: n.clone(), ty: t.clone(), comptime: false, owned: false, span }).collect(),
                ret: Some(f.ret.clone()),
                effects: vec![],
                wheres: vec![],
                export: None,
                extern_c: true,
                variadic: f.variadic,
                cimport_header: Some(header.clone()),
                body: None,
                span,
                name_span: span,
            };
            let fid = self.fns.len() as FnDefId;
            self.fns.push(FnDef { decl, module: mid, impl_id: None, is_generic: false, type_params: vec![] });
            self.items.insert((mid, f.name.clone()), ItemRef::Fn(fid));
        }
        for (name, lit) in &ci.consts {
            if self.items.contains_key(&(mid, name.clone())) {
                continue;
            }
            let id = self.consts.len() as DefId;
            let mangled = format!("nxc_{}_{}", name, id);
            // C integer constants are `int` unless they need more bits
            use crate::cimport::CConst;
            let (ty_expr, value) = match lit {
                CConst::Int(v) => {
                    let ty = if *v >= i32::MIN as i128 && *v <= i32::MAX as i128 {
                        "i32"
                    } else if *v >= i64::MIN as i128 && *v <= i64::MAX as i128 {
                        "i64"
                    } else {
                        "u64"
                    };
                    let mag = Expr::Lit { value: Lit::Int(v.unsigned_abs()), span };
                    let e = if *v < 0 { Expr::Unary { op: UnOp::Neg, expr: Box::new(mag), span } } else { mag };
                    (Some(TypeExpr::named(ty, span)), e)
                }
                CConst::Float(f) => (Some(TypeExpr::named("f64", span)), Expr::Lit { value: Lit::Float(*f), span }),
                CConst::Str(b) => (None, Expr::Lit { value: Lit::Str(b.clone()), span }),
            };
            self.consts.push(ConstDef { name: name.clone(), ty_expr, value, module: mid, span, resolved: None, in_progress: false, is_pub: true, mangled });
            self.items.insert((mid, name.clone()), ItemRef::Const(id));
        }
        for (name, why) in &ci.unsupported {
            self.cimport_unsupported.insert((mid, name.clone()), why.clone());
        }
        self.items.insert((module, const_name.to_string()), ItemRef::Module(mid));
    }

    fn define(&mut self, module: u32, name: &str, r: ItemRef, span: Span) {
        if self.items.contains_key(&(module, name.to_string())) {
            self.error(span, format!("`{}` is defined more than once in this module", name));
            return;
        }
        self.items.insert((module, name.to_string()), r);
    }

    fn collect_item(&mut self, item: &Item, module: u32) {
        match item {
            Item::Fn(f) => {
                let id = self.fns.len() as FnDefId;
                let type_params: Vec<String> =
                    f.params.iter().filter(|p| p.comptime && matches!(&p.ty, TypeExpr::Named { path, .. } if path.len() == 1 && path[0] == "type")).map(|p| p.name.clone()).collect();
                let is_generic = !type_params.is_empty() || f.params.iter().any(|p| p.comptime);
                self.fns.push(FnDef { decl: f.clone(), module, impl_id: None, is_generic, type_params });
                self.define(module, &f.name, ItemRef::Fn(id), f.name_span);
            }
            Item::Struct(s) => {
                let id = self.structs.len() as DefId;
                self.structs.push(StructDef {
                    name: s.name.clone(),
                    kind: s.kind,
                    layout: s.layout,
                    type_params: s.type_params.clone(),
                    fields: s.fields.iter().map(|f| FieldDef { name: f.name.clone(), ty_expr: f.ty.clone(), default: f.default.clone(), constraint: f.constraint.clone(), span: f.span }).collect(),
                    derives: s.derives.clone(),
                    module,
                    span: s.span,
                    is_pub: s.attrs.is_pub,
                    c_name: None,
                });
                self.define(module, &s.name, ItemRef::Struct(id), s.span);
            }
            Item::Enum(e) => {
                let id = self.enums.len() as DefId;
                self.enums.push(EnumDef {
                    name: e.name.clone(),
                    type_params: e.type_params.clone(),
                    variants: e
                        .variants
                        .iter()
                        .map(|v| VariantDef {
                            name: v.name.clone(),
                            payload: match &v.payload {
                                VariantPayload::Unit => VariantPayloadDef::Unit,
                                VariantPayload::Tuple(ts) => VariantPayloadDef::Tuple(ts.clone()),
                                VariantPayload::Struct(fs) => VariantPayloadDef::Struct(
                                    fs.iter().map(|f| FieldDef { name: f.name.clone(), ty_expr: f.ty.clone(), default: f.default.clone(), constraint: f.constraint.clone(), span: f.span }).collect(),
                                ),
                            },
                            span: v.span,
                        })
                        .collect(),
                    derives: e.derives.clone(),
                    module,
                    span: e.span,
                });
                self.define(module, &e.name, ItemRef::Enum(id), e.span);
            }
            Item::Trait(t) => {
                let id = self.traits.len() as DefId;
                self.traits.push(TraitDef { name: t.name.clone(), methods: t.methods.clone(), assoc_types: t.assoc_types.clone(), module, span: t.span });
                self.define(module, &t.name, ItemRef::Trait(id), t.span);
            }
            Item::Impl(im) => {
                let impl_id = self.impls.len() as u32;
                let mut methods = Vec::new();
                for m in &im.methods {
                    let fid = self.fns.len() as FnDefId;
                    let type_params: Vec<String> =
                        m.params.iter().filter(|p| p.comptime && matches!(&p.ty, TypeExpr::Named { path, .. } if path.len() == 1 && path[0] == "type")).map(|p| p.name.clone()).collect();
                    let is_generic = !type_params.is_empty() || !im.type_params.is_empty() || m.params.iter().any(|p| p.comptime);
                    self.fns.push(FnDef { decl: m.clone(), module, impl_id: Some(impl_id), is_generic, type_params });
                    methods.push((m.name.clone(), fid));
                }
                self.impls.push(ImplDef {
                    trait_name: im.trait_name.clone(),
                    target: im.target.clone(),
                    type_params: im.type_params.clone(),
                    methods,
                    assoc_types: im.assoc_types.clone(),
                    module,
                    span: im.span,
                });
            }
            Item::Const(c) => {
                let id = self.consts.len() as DefId;
                let mangled = format!("nxc_{}_{}", c.name, id);
                self.consts.push(ConstDef {
                    name: c.name.clone(),
                    ty_expr: c.ty.clone(),
                    value: c.value.clone(),
                    module,
                    span: c.span,
                    resolved: None,
                    in_progress: false,
                    is_pub: c.attrs.is_pub,
                    mangled,
                });
                self.define(module, &c.name, ItemRef::Const(id), c.span);
            }
            Item::Global(g) => {
                let id = self.globals.len() as DefId;
                let mangled = format!("nxg_{}_{}", g.name, id);
                self.globals.push(GlobalDef { name: g.name.clone(), ty_expr: g.ty.clone(), value: g.value.clone(), module, span: g.span, ty: None, init: None, mangled, align: g.align });
                self.define(module, &g.name, ItemRef::Global(id), g.span);
            }
            Item::TypeAlias(a) => {
                let id = self.aliases.len() as DefId;
                self.aliases.push(AliasDef { name: a.name.clone(), distinct: a.distinct, ty_expr: a.ty.clone(), module, resolved: None, span: a.span });
                self.define(module, &a.name, ItemRef::Alias(id), a.span);
            }
            Item::ErrorSet(e) => {
                let id = self.error_sets.len() as DefId;
                let ids = e.names.iter().map(|n| self.error_id(n)).collect();
                self.error_sets.push(ErrorSetDef { name: e.name.clone(), ids, module, span: e.span });
                self.define(module, &e.name, ItemRef::ErrorSet(id), e.span);
            }
            Item::Import(im) => {
                let target = im.path.join(".");
                let target_mod = self.module_names.iter().position(|n| *n == target || n.ends_with(&format!("/{}", target.replace('.', "/"))) || *n == target.replace('.', "/"));
                match target_mod {
                    Some(mid) => {
                        let mid = mid as u32;
                        if let Some(names) = &im.names {
                            for n in names {
                                self.module_imports.insert((module, n.clone()), (mid, Some(n.clone())));
                            }
                        } else {
                            let alias = im.alias.clone().unwrap_or_else(|| im.path.last().unwrap().clone());
                            self.module_imports.insert((module, alias), (mid, None));
                        }
                    }
                    None => {
                        // std modules are builtin namespaces
                        let last = im.path.last().unwrap().clone();
                        if im.path[0] == "std" || matches!(last.as_str(), "math" | "io" | "os" | "time" | "random" | "fmt" | "utf8" | "ascii" | "mem" | "slice" | "process" | "test" | "alloc" | "net")
                        {
                            // nothing to do; namespaces resolve by name
                        } else {
                            self.error(im.span, format!("cannot find module `{}`; expected a file `{}.nx` next to this one", target, target.replace('.', "/")));
                        }
                    }
                }
            }
            Item::Test(t) => {
                // tests are functions with no parameters returning !void
                let fid = self.fns.len() as FnDefId;
                let decl = FnDecl {
                    attrs: Attrs { is_pub: false, doc: vec![] },
                    name: format!("{}:{}", if t.comptime { "ctest" } else { "test" }, t.name),
                    params: vec![],
                    ret: Some(TypeExpr::ErrorUnion { set: None, elem: Box::new(TypeExpr::named("void", t.span)), span: t.span }),
                    effects: vec![],
                    wheres: vec![],
                    export: None,
                    extern_c: false,
                    variadic: false,
                    cimport_header: None,
                    body: Some(t.body.clone()),
                    span: t.span,
                    name_span: t.span,
                };
                self.fns.push(FnDef { decl, module, impl_id: None, is_generic: false, type_params: vec![] });
                let _ = fid;
            }
            Item::Artifact(a) => {
                let mut fields = Vec::new();
                for (k, v, sp) in &a.fields {
                    match Self::artifact_value(v) {
                        Some(av) => fields.push((k.clone(), av)),
                        None => self.error(*sp, format!("artifact field `{}` must be a string, identifier, list, or boolean", k)),
                    }
                }
                self.artifacts.push(ArtifactInfo { kind: a.kind.clone(), fields, span: a.span });
            }
        }
    }

    fn artifact_value(e: &Expr) -> Option<ArtifactValue> {
        Some(match e {
            Expr::Lit { value: Lit::Str(s), .. } => ArtifactValue::Str(String::from_utf8_lossy(s).to_string()),
            Expr::Lit { value: Lit::Bool(b), .. } => ArtifactValue::Bool(*b),
            Expr::Lit { value: Lit::Int(i), .. } => ArtifactValue::Int(*i as i128),
            Expr::Ident { name, .. } => ArtifactValue::Ident(name.clone()),
            Expr::Field { base, name, .. } => {
                let mut b = match Self::artifact_value(base)? {
                    ArtifactValue::Ident(s) => s,
                    _ => return None,
                };
                b.push('.');
                b.push_str(name);
                ArtifactValue::Ident(b)
            }
            Expr::ArrayLit { elems, .. } => ArtifactValue::List(elems.iter().map(Self::artifact_value).collect::<Option<Vec<_>>>()?),
            _ => return None,
        })
    }

    // ----- lookup -----------------------------------------------------------

    pub fn lookup_item(&self, module: u32, name: &str) -> Option<ItemRef> {
        if let Some(r) = self.items.get(&(module, name.to_string())) {
            return Some(r.clone());
        }
        if let Some((mid, item)) = self.module_imports.get(&(module, name.to_string())) {
            return match item {
                Some(n) => self.items.get(&(*mid, n.clone())).cloned(),
                None => Some(ItemRef::Module(*mid)),
            };
        }
        None
    }

    // ----- type resolution --------------------------------------------------

    pub fn type_name(&self, t: TyId) -> String {
        let t = self.tys.shallow(t);
        match self.tys.kind(t).clone() {
            TyKind::Int(i) => i.name().to_string(),
            TyKind::Float(FloatTy::F32) => "f32".into(),
            TyKind::Float(FloatTy::F64) => "f64".into(),
            TyKind::Bool => "bool".into(),
            TyKind::Char => "char".into(),
            TyKind::Void => "void".into(),
            TyKind::Never => "never".into(),
            TyKind::Struct(d, args) => {
                let n = self.structs[d as usize].name.clone();
                if args.is_empty() {
                    n
                } else {
                    format!("{}({})", n, args.iter().map(|a| self.type_name(*a)).collect::<Vec<_>>().join(", "))
                }
            }
            TyKind::Enum(d, args) => {
                let n = self.enums[d as usize].name.clone();
                if args.is_empty() {
                    n
                } else {
                    format!("{}({})", n, args.iter().map(|a| self.type_name(*a)).collect::<Vec<_>>().join(", "))
                }
            }
            TyKind::Array(n, e) => format!("[{}]{}", n, self.type_name(e)),
            TyKind::Slice(m, e) => format!("[]{}{}", if m { "mut " } else { "" }, self.type_name(e)),
            TyKind::Ptr(m, e) => format!("*{}{}", if m { "mut " } else { "" }, self.type_name(e)),
            TyKind::Opt(e) => format!("?{}", self.type_name(e)),
            TyKind::ErrUnion(s, e) => match s {
                Some(s) => format!("{}!{}", self.error_sets[s as usize].name, self.type_name(e)),
                None => format!("!{}", self.type_name(e)),
            },
            TyKind::Fn(ps, r, ef) => {
                let eff = Effects(ef);
                let neg = eff.render_negative();
                format!("fn({}) -> {}{}", ps.iter().map(|p| self.type_name(*p)).collect::<Vec<_>>().join(", "), self.type_name(r), if neg.is_empty() { String::new() } else { format!(" {}", neg) })
            }
            TyKind::Tuple(ts) => format!("({})", ts.iter().map(|p| self.type_name(*p)).collect::<Vec<_>>().join(", ")),
            TyKind::Distinct(d) => self.aliases[d as usize].name.clone(),
            TyKind::Weak(e) => format!("weak {}", self.type_name(e)),
            TyKind::List(e) => format!("List({})", self.type_name(e)),
            TyKind::Str => "String".into(),
            TyKind::Map(k, v) => format!("Map({}, {})", self.type_name(k), self.type_name(v)),
            TyKind::Type => "type".into(),
            TyKind::ErrorSet(None) => "error".into(),
            TyKind::ErrorSet(Some(s)) => self.error_sets[s as usize].name.clone(),
            TyKind::Infer(n) => match self.tys.infer_kinds[n as usize] {
                InferKind::Integer => "{integer}".into(),
                InferKind::Float => "{float}".into(),
                InferKind::Any => "_".into(),
            },
            TyKind::IntLit => "{integer}".into(),
            TyKind::FloatLit => "{float}".into(),
            TyKind::Closure(_) => "closure".into(),
            TyKind::Param(n) => n,
            TyKind::Namespace(n) => format!("namespace {}", n),
            TyKind::Dyn(t, ef) => {
                let neg = Effects(ef).render_negative();
                format!("dyn {}{}", self.traits[t as usize].name, if neg.is_empty() { String::new() } else { format!(" {}", neg) })
            }
        }
    }

    /// Resolve a type expression. `generics` maps comptime type parameter names.
    pub fn resolve_type(&mut self, te: &TypeExpr, generics: &HashMap<String, TyId>, self_ty: Option<TyId>, module: u32) -> TyId {
        match te {
            TypeExpr::Infer { .. } => self.tys.fresh_infer(),
            TypeExpr::Array { len, elem, span } => {
                let e = self.resolve_type(elem, generics, self_ty, module);
                if let Expr::Ident { name, .. } = &**len {
                    if name == "_" {
                        // inferred length: only valid with an initializer; use a marker
                        self.error(*span, "array length `_` can only be inferred from an initializer; write the length explicitly");
                        return self.tys.array(0, e);
                    }
                }
                match self.const_eval_int(len, module, generics) {
                    Some(n) if n >= 0 => self.tys.array(n as u64, e),
                    Some(_) => {
                        self.error(*span, "array length must not be negative");
                        self.tys.array(0, e)
                    }
                    None => {
                        self.error(len.span(), "array length must be a compile-time constant integer");
                        self.tys.array(0, e)
                    }
                }
            }
            TypeExpr::Slice { mutable, elem, .. } => {
                let e = self.resolve_type(elem, generics, self_ty, module);
                self.tys.slice(*mutable, e)
            }
            TypeExpr::Ptr { mutable, elem, .. } => {
                let e = self.resolve_type(elem, generics, self_ty, module);
                self.tys.ptr(*mutable, e)
            }
            TypeExpr::Optional { elem, .. } => {
                let e = self.resolve_type(elem, generics, self_ty, module);
                self.tys.opt(e)
            }
            TypeExpr::Weak { elem, span } => {
                let e = self.resolve_type(elem, generics, self_ty, module);
                if !self.is_ref_class(e) {
                    self.error(*span, "`weak` applies only to `ref class` types");
                }
                self.tys.intern(TyKind::Weak(e))
            }
            TypeExpr::ErrorUnion { set, elem, span } => {
                let e = self.resolve_type(elem, generics, self_ty, module);
                let set_id = match set {
                    None => None,
                    Some(s) => match &**s {
                        TypeExpr::Named { path, .. } if path.len() == 1 => match self.lookup_item(module, &path[0]) {
                            Some(ItemRef::ErrorSet(id)) => Some(id),
                            _ => {
                                self.error(*span, format!("`{}` is not an error set", path[0]));
                                None
                            }
                        },
                        _ => {
                            self.error(*span, "expected an error set name before `!`");
                            None
                        }
                    },
                };
                self.tys.err_union(set_id, e)
            }
            TypeExpr::Fn { params, ret, effects, .. } => {
                let ps: Vec<TyId> = params.iter().map(|p| self.resolve_type(p, generics, self_ty, module)).collect();
                let r = self.resolve_type(ret, generics, self_ty, module);
                let mut eff = Effects::ALL;
                for b in effects {
                    if let Some(e) = Effects::from_name(&b.name) {
                        if b.negative {
                            eff = eff.without(e);
                        }
                    }
                }
                self.tys.intern(TyKind::Fn(ps, r, eff.0))
            }
            TypeExpr::Tuple { elems, .. } => {
                let ts: Vec<TyId> = elems.iter().map(|p| self.resolve_type(p, generics, self_ty, module)).collect();
                if ts.is_empty() {
                    self.tys.void()
                } else {
                    self.tys.tuple(ts)
                }
            }
            TypeExpr::Dyn { trait_name, effects, span } => match self.lookup_item(module, trait_name) {
                Some(ItemRef::Trait(id)) => {
                    let mut eff = Effects::ALL;
                    for b in effects {
                        if let Some(e) = Effects::from_name(&b.name) {
                            if b.negative {
                                eff = eff.without(e);
                            }
                        }
                    }
                    self.tys.intern(TyKind::Dyn(id, eff.0))
                }
                _ => {
                    self.error(*span, format!("cannot find trait `{}`", trait_name));
                    self.tys.void()
                }
            },
            TypeExpr::Named { path, args, span } => self.resolve_named_type(path, args, *span, generics, self_ty, module),
        }
    }

    pub fn is_ref_class(&self, t: TyId) -> bool {
        match self.tys.kind(self.tys.shallow(t)) {
            TyKind::Struct(d, _) => self.structs[*d as usize].kind == StructKind::RefClass,
            _ => false,
        }
    }

    fn resolve_named_type(&mut self, path: &[String], args: &[TypeExpr], span: Span, generics: &HashMap<String, TyId>, self_ty: Option<TyId>, module: u32) -> TyId {
        let name = &path[0];
        if path.len() == 1 {
            if name == "Self" {
                return match self_ty {
                    Some(t) => t,
                    None => {
                        self.error(span, "`Self` is only valid inside an impl or trait");
                        self.tys.void()
                    }
                };
            }
            if let Some(&t) = generics.get(name) {
                return t;
            }
            if let Some(t) = self.primitive_type(name) {
                if !args.is_empty() {
                    self.error(span, format!("`{}` takes no type arguments", name));
                }
                return t;
            }
            match name.as_str() {
                "List" => {
                    if args.len() != 1 {
                        self.error(span, "`List` takes one type argument: List(T)");
                        return self.tys.void();
                    }
                    let e = self.resolve_type(&args[0], generics, self_ty, module);
                    return self.tys.list(e);
                }
                "Map" => {
                    if args.len() != 2 {
                        self.error(span, "`Map` takes two type arguments: Map(K, V)");
                        return self.tys.void();
                    }
                    let k = self.resolve_type(&args[0], generics, self_ty, module);
                    let v = self.resolve_type(&args[1], generics, self_ty, module);
                    if !self.is_hashable_key(k) {
                        self.error(args[0].span(), format!("`{}` cannot be a Map key; keys must be integers, bool, char, `[]u8`, or `String`", self.type_name(k)));
                    }
                    return self.tys.intern(TyKind::Map(k, v));
                }
                "String" => return self.tys.string(),
                "Allocator" => return self.tys.intern(TyKind::Namespace("Allocator".into())),
                _ => {}
            }
        }
        // module-qualified `mod.Type`
        let (module, name) = if path.len() == 2 {
            match self.lookup_item(module, &path[0]) {
                Some(ItemRef::Module(m)) => (m, &path[1]),
                Some(ItemRef::Enum(_)) | Some(ItemRef::Struct(_)) => {
                    // `Enum.Variant` used as a type is an error
                    self.error(span, format!("`{}` is not a type", path.join(".")));
                    return self.tys.void();
                }
                _ => {
                    self.error(span, format!("cannot find module `{}`", path[0]));
                    return self.tys.void();
                }
            }
        } else if path.len() > 2 {
            self.error(span, "type paths may have at most one module qualifier");
            return self.tys.void();
        } else {
            (module, name)
        };
        match self.lookup_item(module, name) {
            Some(ItemRef::Struct(id)) => {
                let def = &self.structs[id as usize];
                let n_params = def.type_params.len();
                if args.len() != n_params {
                    let dn = def.name.clone();
                    self.error(span, format!("`{}` expects {} type argument(s) but {} were given", dn, n_params, args.len()));
                    return self.tys.void();
                }
                let targs: Vec<TyId> = args.iter().map(|a| self.resolve_type(a, generics, self_ty, module)).collect();
                let t = self.tys.intern(TyKind::Struct(id, targs));
                self.note_used(t);
                t
            }
            Some(ItemRef::Enum(id)) => {
                let def = &self.enums[id as usize];
                let n_params = def.type_params.len();
                if args.len() != n_params {
                    let dn = def.name.clone();
                    self.error(span, format!("`{}` expects {} type argument(s) but {} were given", dn, n_params, args.len()));
                    return self.tys.void();
                }
                let targs: Vec<TyId> = args.iter().map(|a| self.resolve_type(a, generics, self_ty, module)).collect();
                let t = self.tys.intern(TyKind::Enum(id, targs));
                self.note_used(t);
                t
            }
            Some(ItemRef::Alias(id)) => self.resolve_alias(id),
            Some(ItemRef::ErrorSet(id)) => self.tys.intern(TyKind::ErrorSet(Some(id))),
            Some(ItemRef::Trait(_)) => {
                self.error(span, format!("`{}` is a trait, not a type; use it as a bound: `comptime T: type where T: {}`", name, name));
                self.tys.void()
            }
            Some(_) => {
                self.error(span, format!("`{}` is not a type", name));
                self.tys.void()
            }
            None => {
                self.error(span, format!("cannot find type `{}`", name));
                self.tys.void()
            }
        }
    }

    pub fn note_used(&mut self, t: TyId) {
        if !self.used_tys.contains(&t) {
            self.used_tys.push(t);
        }
    }

    pub fn is_hashable_key(&self, k: TyId) -> bool {
        let k = self.tys.shallow(k);
        match self.tys.kind(k) {
            TyKind::Int(_) | TyKind::Bool | TyKind::Char | TyKind::Str => true,
            TyKind::Slice(_, e) => matches!(self.tys.kind(*e), TyKind::Int(IntTy::U8)),
            TyKind::Infer(_) => true,
            _ => false,
        }
    }

    pub fn resolve_alias(&mut self, id: DefId) -> TyId {
        if let Some(t) = self.aliases[id as usize].resolved {
            return t;
        }
        let a = self.aliases[id as usize].clone();
        let under = self.resolve_type(&a.ty_expr, &HashMap::new(), None, a.module);
        let t = if a.distinct {
            let d = self.tys.intern(TyKind::Distinct(id));
            self.distinct_underlying.insert(id, under);
            d
        } else {
            under
        };
        self.aliases[id as usize].resolved = Some(t);
        t
    }

    pub fn primitive_type(&mut self, name: &str) -> Option<TyId> {
        if let Some(i) = IntTy::from_name(name) {
            return Some(self.tys.int(i));
        }
        Some(match name {
            "f32" => self.tys.float(FloatTy::F32),
            "f64" => self.tys.float(FloatTy::F64),
            "bool" => self.tys.bool(),
            "char" => self.tys.char(),
            "void" => self.tys.void(),
            "never" => self.tys.never(),
            "error" => self.tys.intern(TyKind::ErrorSet(None)),
            "type" => self.tys.type_ty(),
            "anytype" => self.tys.fresh_infer(),
            _ => return None,
        })
    }

    /// Evaluate a simple constant integer expression (array lengths, generic const args).
    pub fn const_eval_int(&mut self, e: &Expr, module: u32, generics: &HashMap<String, TyId>) -> Option<i128> {
        match e {
            Expr::Lit { value: Lit::Int(v), .. } => Some(*v as i128),
            Expr::Unary { op: UnOp::Neg, expr, .. } => self.const_eval_int(expr, module, generics).map(|v| -v),
            Expr::Binary { op, lhs, rhs, .. } => {
                let a = self.const_eval_int(lhs, module, generics)?;
                let b = self.const_eval_int(rhs, module, generics)?;
                Some(match op {
                    BinOp::Add => a + b,
                    BinOp::Sub => a - b,
                    BinOp::Mul => a * b,
                    BinOp::Div => {
                        if b == 0 {
                            return None;
                        } else {
                            a / b
                        }
                    }
                    BinOp::Rem => {
                        if b == 0 {
                            return None;
                        } else {
                            a % b
                        }
                    }
                    BinOp::Shl => a << b,
                    BinOp::Shr => a >> b,
                    BinOp::BitAnd => a & b,
                    BinOp::BitOr => a | b,
                    BinOp::BitXor => a ^ b,
                    _ => return None,
                })
            }
            Expr::Ident { name, .. } => {
                if let Some(t) = generics.get(name) {
                    // const generic argument encoded as a `#N` named type
                    if let TyKind::Param(p) = self.tys.kind(*t).clone() {
                        if let Some(rest) = p.strip_prefix('#') {
                            return rest.parse().ok();
                        }
                    }
                }
                match self.lookup_item(module, name) {
                    Some(ItemRef::Const(id)) => {
                        let (_, te) = self.resolve_const(id)?;
                        match te.kind {
                            TExprKind::Int(v) => Some(v),
                            TExprKind::Value(Value::Int(v)) => Some(v),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            Expr::Comptime { expr, .. } => self.const_eval_int(expr, module, generics),
            Expr::Builtin { name, args, .. } if name == "sizeOf" && args.len() == 1 => {
                if let Expr::TypeVal { ty, .. } = &args[0] {
                    let t = self.resolve_type(ty, generics, None, module);
                    return Some(self.size_of(t) as i128);
                }
                None
            }
            _ => None,
        }
    }

    /// Byte size of a type under the C backend's layout (best effort; used for `@sizeOf`).
    pub fn size_of(&mut self, t: TyId) -> u64 {
        let t = self.tys.shallow(t);
        match self.tys.kind(t).clone() {
            TyKind::Int(i) => (i.bits() / 8) as u64,
            TyKind::Float(FloatTy::F32) => 4,
            TyKind::Float(FloatTy::F64) => 8,
            TyKind::Bool => 1,
            TyKind::Char => 4,
            TyKind::Void | TyKind::Never => 0,
            TyKind::Array(n, e) => n * self.size_of(e),
            TyKind::Slice(..) => 16,
            TyKind::Ptr(..) | TyKind::Weak(_) => 8,
            TyKind::Opt(e) => {
                let s = self.size_of(e);
                let a = self.align_of(e);
                round_up(s + 1, a)
            }
            TyKind::ErrUnion(_, e) => {
                let s = self.size_of(e);
                let a = self.align_of(e).max(4);
                round_up(round_up(4, self.align_of(e)) + s, a)
            }
            TyKind::Fn(..) => 16,
            TyKind::Tuple(ts) => {
                let mut off = 0;
                let mut maxa = 1;
                for e in ts {
                    let a = self.align_of(e);
                    maxa = maxa.max(a);
                    off = round_up(off, a) + self.size_of(e);
                }
                round_up(off, maxa)
            }
            TyKind::Distinct(d) => {
                let u = self.distinct_underlying[&d];
                self.size_of(u)
            }
            TyKind::List(_) | TyKind::Str => 24,
            TyKind::Map(..) => 40,
            TyKind::Struct(d, _) => {
                if self.structs[d as usize].kind == StructKind::RefClass {
                    return 8;
                }
                let ftys = self.struct_field_types(t);
                let mut off = 0;
                let mut maxa = 1;
                for e in ftys {
                    let a = self.align_of(e);
                    maxa = maxa.max(a);
                    off = round_up(off, a) + self.size_of(e);
                }
                round_up(off, maxa)
            }
            TyKind::Enum(..) => {
                let vtys = self.enum_variant_types(t);
                let mut maxs = 0;
                let mut maxa = 4;
                for v in vtys {
                    let mut off = 0;
                    for e in v {
                        let a = self.align_of(e);
                        maxa = maxa.max(a);
                        off = round_up(off, a) + self.size_of(e);
                    }
                    maxs = maxs.max(off);
                }
                round_up(round_up(4, maxa) + maxs, maxa)
            }
            _ => 8,
        }
    }

    pub fn align_of(&mut self, t: TyId) -> u64 {
        let t = self.tys.shallow(t);
        match self.tys.kind(t).clone() {
            TyKind::Int(i) => ((i.bits() / 8) as u64).min(16),
            TyKind::Float(FloatTy::F32) => 4,
            TyKind::Float(FloatTy::F64) => 8,
            TyKind::Bool => 1,
            TyKind::Char => 4,
            TyKind::Void | TyKind::Never => 1,
            TyKind::Array(_, e) => self.align_of(e),
            TyKind::Opt(e) => self.align_of(e),
            TyKind::ErrUnion(_, e) => self.align_of(e).max(4),
            TyKind::Tuple(ts) => ts.iter().map(|&e| self.align_of(e)).max().unwrap_or(1),
            TyKind::Distinct(d) => {
                let u = self.distinct_underlying[&d];
                self.align_of(u)
            }
            TyKind::Struct(d, _) => {
                if self.structs[d as usize].kind == StructKind::RefClass {
                    return 8;
                }
                let ftys = self.struct_field_types(t);
                ftys.iter().map(|&e| self.align_of(e)).max().unwrap_or(1)
            }
            TyKind::Enum(..) => {
                let vtys = self.enum_variant_types(t);
                vtys.iter().flatten().map(|&e| self.align_of(e)).max().unwrap_or(1).max(4)
            }
            _ => 8,
        }
    }

    /// Field types of an instantiated struct type.
    pub fn struct_field_types(&mut self, t: TyId) -> Vec<TyId> {
        let t = self.tys.shallow(t);
        if let Some(v) = self.struct_field_tys.get(&t) {
            return v.clone();
        }
        let (d, args) = match self.tys.kind(t).clone() {
            TyKind::Struct(d, args) => (d, args),
            _ => return vec![],
        };
        let def = self.structs[d as usize].clone();
        let mut generics = HashMap::new();
        for (p, a) in def.type_params.iter().zip(args.iter()) {
            generics.insert(p.clone(), *a);
        }
        // guard against recursion through value types
        self.struct_field_tys.insert(t, vec![]);
        let tys: Vec<TyId> = def.fields.iter().map(|f| self.resolve_type(&f.ty_expr, &generics, Some(t), def.module)).collect();
        // value-type recursion check (a struct containing itself by value is infinite)
        if def.kind != StructKind::RefClass {
            for (f, &ft) in def.fields.iter().zip(tys.iter()) {
                if self.contains_by_value(ft, t) {
                    self.error(f.span, format!("struct `{}` contains itself by value through field `{}`; use `?*{}`, a `ref class`, or `List`", def.name, f.name, def.name));
                }
            }
        }
        self.struct_field_tys.insert(t, tys.clone());
        tys
    }

    fn contains_by_value(&mut self, t: TyId, target: TyId) -> bool {
        let t = self.tys.shallow(t);
        if t == target {
            return true;
        }
        match self.tys.kind(t).clone() {
            TyKind::Array(_, e) | TyKind::Opt(e) | TyKind::ErrUnion(_, e) => self.contains_by_value(e, target),
            TyKind::Tuple(ts) => ts.iter().any(|&e| self.contains_by_value(e, target)),
            TyKind::Struct(d, _) if self.structs[d as usize].kind != StructKind::RefClass => {
                if self.struct_field_tys.get(&t).is_none() {
                    // not yet computed: compute (may recurse); the guard entry prevents infinite loops
                    let f = self.struct_field_types(t);
                    return f.iter().any(|&e| self.contains_by_value(e, target));
                }
                let f = self.struct_field_tys[&t].clone();
                f.iter().any(|&e| self.contains_by_value(e, target))
            }
            TyKind::Enum(..) => {
                if let Some(v) = self.enum_variant_tys.get(&t).cloned() {
                    return v.iter().flatten().any(|&e| self.contains_by_value(e, target));
                }
                let v = self.enum_variant_types(t);
                v.iter().flatten().any(|&e| self.contains_by_value(e, target))
            }
            _ => false,
        }
    }

    pub fn enum_variant_types(&mut self, t: TyId) -> Vec<Vec<TyId>> {
        let t = self.tys.shallow(t);
        if let Some(v) = self.enum_variant_tys.get(&t) {
            return v.clone();
        }
        let (d, args) = match self.tys.kind(t).clone() {
            TyKind::Enum(d, args) => (d, args),
            _ => return vec![],
        };
        let def = self.enums[d as usize].clone();
        let mut generics = HashMap::new();
        for (p, a) in def.type_params.iter().zip(args.iter()) {
            generics.insert(p.clone(), *a);
        }
        self.enum_variant_tys.insert(t, vec![]);
        let tys: Vec<Vec<TyId>> = def
            .variants
            .iter()
            .map(|v| match &v.payload {
                VariantPayloadDef::Unit => vec![],
                VariantPayloadDef::Tuple(ts) => ts.iter().map(|te| self.resolve_type(te, &generics, Some(t), def.module)).collect(),
                VariantPayloadDef::Struct(fs) => fs.iter().map(|f| self.resolve_type(&f.ty_expr, &generics, Some(t), def.module)).collect(),
            })
            .collect();
        for (v, vt) in def.variants.iter().zip(tys.iter()) {
            for &ft in vt {
                if self.contains_by_value(ft, t) {
                    self.error(v.span, format!("enum `{}` contains itself by value in variant `{}`; use a pointer, `ref class`, or `List`", def.name, v.name));
                }
            }
        }
        self.enum_variant_tys.insert(t, tys.clone());
        tys
    }

    // ----- unification --------------------------------------------------------

    pub fn unify(&mut self, a: TyId, b: TyId) -> bool {
        let a = self.tys.shallow(a);
        let b = self.tys.shallow(b);
        if a == b {
            return true;
        }
        let ka = self.tys.kind(a).clone();
        let kb = self.tys.kind(b).clone();
        match (&ka, &kb) {
            (TyKind::Infer(n), _) => self.bind_infer(*n, b),
            (_, TyKind::Infer(n)) => self.bind_infer(*n, a),
            (TyKind::Never, _) | (_, TyKind::Never) => true,
            (TyKind::Struct(d1, a1), TyKind::Struct(d2, a2)) | (TyKind::Enum(d1, a1), TyKind::Enum(d2, a2)) => {
                d1 == d2 && a1.len() == a2.len() && a1.iter().zip(a2.iter()).all(|(&x, &y)| self.unify(x, y))
            }
            (TyKind::Array(n1, e1), TyKind::Array(n2, e2)) => n1 == n2 && self.unify(*e1, *e2),
            (TyKind::Slice(m1, e1), TyKind::Slice(m2, e2)) => m1 == m2 && self.unify(*e1, *e2),
            (TyKind::Ptr(m1, e1), TyKind::Ptr(m2, e2)) => m1 == m2 && self.unify(*e1, *e2),
            (TyKind::Opt(e1), TyKind::Opt(e2)) | (TyKind::List(e1), TyKind::List(e2)) | (TyKind::Weak(e1), TyKind::Weak(e2)) => self.unify(*e1, *e2),
            (TyKind::ErrUnion(s1, e1), TyKind::ErrUnion(s2, e2)) => (s1 == s2 || s1.is_none() || s2.is_none()) && self.unify(*e1, *e2),
            (TyKind::Map(k1, v1), TyKind::Map(k2, v2)) => self.unify(*k1, *k2) && self.unify(*v1, *v2),
            (TyKind::Fn(p1, r1, ef1), TyKind::Fn(p2, r2, ef2)) => {
                p1.len() == p2.len() && p1.iter().zip(p2.iter()).all(|(&x, &y)| self.unify(x, y)) && self.unify(*r1, *r2) && (ef1 == ef2 || Effects(*ef1).contains(Effects(*ef2)))
            }
            (TyKind::Tuple(t1), TyKind::Tuple(t2)) => t1.len() == t2.len() && t1.iter().zip(t2.iter()).all(|(&x, &y)| self.unify(x, y)),
            (TyKind::ErrorSet(_), TyKind::ErrorSet(_)) => true,
            (TyKind::Dyn(t1, e1), TyKind::Dyn(t2, e2)) => t1 == t2 && Effects(*e2).contains(Effects(*e1)),
            _ => false,
        }
    }

    fn bind_infer(&mut self, n: u32, target: TyId) -> bool {
        let kind = self.tys.infer_kinds[n as usize];
        let target = self.tys.shallow(target);
        if let TyKind::Infer(m) = self.tys.kind(target).clone() {
            if m == n {
                return true;
            }
            let tk = self.tys.infer_kinds[m as usize];
            let merged = match (kind, tk) {
                (InferKind::Any, k) | (k, InferKind::Any) => k,
                (a, b) if a == b => a,
                _ => return false,
            };
            self.tys.infer_kinds[m as usize] = merged;
            self.tys.subst[n as usize] = Some(target);
            return true;
        }
        let ok = match kind {
            InferKind::Any => true,
            InferKind::Integer => matches!(self.tys.kind(target), TyKind::Int(_)),
            InferKind::Float => matches!(self.tys.kind(target), TyKind::Float(_)),
        };
        if !ok {
            return false;
        }
        self.tys.subst[n as usize] = Some(target);
        true
    }

    // ----- functions ------------------------------------------------------------

    /// Get or create an instance of a function definition with the given type arguments.
    pub fn instantiate(&mut self, fid: FnDefId, targs: Vec<TyId>, span: Span) -> InstId {
        let targs: Vec<TyId> = targs.iter().map(|&t| self.tys.resolve(t, true)).collect();
        if let Some(&id) = self.inst_map.get(&(fid, targs.clone())) {
            return id;
        }
        let def = self.fns[fid as usize].clone();
        let id = self.funcs.len() as InstId;
        self.inst_map.insert((fid, targs.clone()), id);
        // build generic env
        let mut generics: HashMap<String, TyId> = HashMap::new();
        let mut self_ty = None;
        let mut idx = 0;
        if let Some(impl_id) = def.impl_id {
            let im = self.impls[impl_id as usize].clone();
            for p in &im.type_params {
                generics.insert(p.clone(), targs.get(idx).copied().unwrap_or_else(|| self.tys.void()));
                idx += 1;
            }
            self_ty = Some(self.resolve_type(&im.target, &generics, None, im.module));
        }
        for p in &def.type_params {
            generics.insert(p.clone(), targs.get(idx).copied().unwrap_or_else(|| self.tys.void()));
            idx += 1;
        }
        // const comptime params (non-type) come after
        let mut locals = Vec::new();
        let mut params = Vec::new();
        for p in &def.decl.params {
            if p.comptime && def.type_params.contains(&p.name) {
                continue;
            }
            let ty = self.resolve_type(&p.ty, &generics, self_ty, def.module);
            let lid = locals.len() as LocalId;
            // in a generic function the instantiation decides whether `own` matters
            if p.owned && !self.needs_drop(ty) && def.type_params.is_empty() {
                let tn = self.type_name(ty);
                self.error(p.span, format!("`own` applies to owning types (`List`, `String`, `Map`, or structs holding them); `{}` is copied anyway", tn));
            }
            locals.push(Local { name: p.name.clone(), ty, mutable: p.owned, span: p.span, is_param: true, owned: p.owned });
            params.push(lid);
        }
        let ret = match &def.decl.ret {
            Some(r) => self.resolve_type(r, &generics, self_ty, def.module),
            None => self.tys.void(),
        };
        let mut declared_neg = Effects::NONE;
        let mut declared_pos = Effects::NONE;
        let mut declared_spans = Vec::new();
        for b in &def.decl.effects {
            if let Some(e) = Effects::from_name(&b.name) {
                if b.negative {
                    declared_neg = declared_neg.union(e);
                } else {
                    declared_pos = declared_pos.union(e);
                }
                declared_spans.push((e, b.span));
            }
        }
        let base_name = match def.impl_id {
            Some(_) => {
                let st = self_ty.map(|t| self.type_name(t)).unwrap_or_default();
                format!("{}_{}", sanitize(&st), def.decl.name)
            }
            None => def.decl.name.clone(),
        };
        let mangled = if def.decl.extern_c {
            def.decl.name.clone()
        } else if def.decl.name.starts_with("test:") {
            format!("nx_test_{}", id)
        } else if targs.is_empty() && def.impl_id.is_none() {
            format!("nx_{}", sanitize(&base_name))
        } else {
            format!("nx_{}_{}", sanitize(&base_name), id)
        };
        let is_test = def.decl.name.starts_with("test:") || def.decl.name.starts_with("ctest:");
        let test_comptime = def.decl.name.starts_with("ctest:");
        let f = TFunc {
            name: def.decl.name.clone(),
            mangled,
            def: Some(fid),
            targs: targs.clone(),
            params,
            ret,
            locals,
            body: None,
            own_effects: Effects::NONE,
            witnesses: Vec::new(),
            callees: Vec::new(),
            effects: Effects::NONE,
            declared_neg,
            declared_pos,
            declared_spans,
            export: def.decl.export.clone(),
            is_extern: def.decl.extern_c,
            is_variadic: def.decl.variadic,
            cimport: def.decl.cimport_header.is_some(),
            is_pub: def.decl.attrs.is_pub,
            is_test,
            test_comptime,
            is_closure: false,
            closure_env: None,
            span: def.decl.span,
            module: def.module,
            moved: HashSet::new(),
            takes_ctx: !def.decl.extern_c,
        };
        self.funcs.push(f);
        if def.decl.export.is_some() {
            self.exports.push(id);
            if !targs.is_empty() {
                self.error(span, "generic functions cannot be exported");
            }
            if def.decl.params.iter().any(|p| p.owned) {
                self.error(span, "exported functions cannot take `own` parameters; the host language cannot hand over ownership");
            }
        }
        // an imported std module's tests belong to the compiler's own suite,
        // not to every program that imports it
        let from_std = self.module_names.get(def.module as usize).map(|n| n.starts_with("std.")).unwrap_or(false);
        if is_test && !test_comptime && !(from_std && def.module != 0) {
            self.tests.push(id);
        }
        // store generics for the body check
        self.queue.push((id, fid, targs));
        self.inst_generics.insert(id, (generics, self_ty));
        id
    }

    pub fn fn_sig(&self, inst: InstId) -> (Vec<TyId>, TyId) {
        let f = &self.funcs[inst as usize];
        (f.params.iter().map(|&p| f.locals[p as usize].ty).collect(), f.ret)
    }

    /// Run the instantiation queue until empty.
    pub fn drain_queue(&mut self) {
        loop {
            if let Some((inst, fid, _targs)) = self.queue.pop() {
                self.check_fn_body(inst, fid);
                continue;
            }
            if let Some(item) = self.closure_queue.pop() {
                self.check_closure_body(item);
                continue;
            }
            break;
        }
    }

    // ----- top level -------------------------------------------------------------

    pub fn check_program(mut self, modules: &[Module], names: &[String]) -> (Result<Program, ()>, Vec<Diag>) {
        self.collect(modules, names);
        self.embedded_mode = self.artifacts.iter().any(|a| matches!(a.kind.as_str(), "cabi" | "python" | "rustlib" | "node" | "shared"));

        // validate impls: coherence and trait method presence
        self.check_impls();

        // resolve globals and consts eagerly
        for i in 0..self.consts.len() {
            let _ = self.resolve_const(i as DefId);
        }
        for i in 0..self.globals.len() {
            self.resolve_global(i as DefId);
        }

        // instantiate every non-generic free function, test, and export
        let n = self.fns.len();
        for fid in 0..n {
            let def = self.fns[fid].clone();
            if def.is_generic || def.impl_id.is_some() {
                continue;
            }
            let id = self.instantiate(fid as FnDefId, vec![], def.decl.span);
            if def.decl.name == "main" {
                self.main = Some(id);
            }
        }
        // non-generic impl methods on non-generic types are instantiated eagerly too,
        // so that `nx effects` can report them
        for fid in 0..n {
            let def = self.fns[fid].clone();
            if def.impl_id.is_some() && !def.is_generic {
                let im = self.impls[def.impl_id.unwrap() as usize].clone();
                if im.type_params.is_empty() {
                    self.instantiate(fid as FnDefId, vec![], def.decl.span);
                }
            }
        }
        self.drain_queue();

        // compile-time tests run in the interpreter; a failure is a compile error
        let ctests: Vec<InstId> = (0..self.funcs.len() as InstId).filter(|&i| self.funcs[i as usize].test_comptime).collect();
        for t in ctests {
            let name = self.funcs[t as usize].name.trim_start_matches("ctest:").to_string();
            let span = self.funcs[t as usize].span;
            match crate::comptime::call_instance(&mut self, t, vec![]) {
                Ok(Some(Value::Err(id))) => {
                    let e = self.error_names[(id - 1) as usize].clone();
                    self.error(span, format!("comptime test \"{}\" failed with error.{}", name, e));
                }
                Ok(Some(_)) => {}
                Ok(None) => self.error(span, format!("comptime test \"{}\" could not be evaluated at compile time (only pure computation runs at compile time)", name)),
                Err((msg, sp)) => self.error_note(sp, format!("comptime test \"{}\" panicked: {}", name, msg), Some(span), "test declared here"),
            }
        }

        // effects
        self.propagate_effects();
        self.check_effect_bounds();
        self.check_embedded_constraints();

        let repl = match self.repl_request.take() {
            Some((start, seed)) if !self.has_errors() => Some(crate::comptime::run_repl(&mut self, start, seed)),
            _ => None,
        };
        if self.has_errors() {
            return (Err(()), self.diags);
        }
        let prog = Program {
            repl,
            funcs: self.funcs,
            structs: self.structs,
            enums: self.enums,
            consts: self.consts,
            globals: self.globals,
            error_names: self.error_names,
            artifacts: self.artifacts,
            tys: self.tys,
            main: self.main,
            tests: self.tests,
            struct_field_tys: self.struct_field_tys,
            enum_variant_tys: self.enum_variant_tys,
            closure_envs: self.closure_envs,
            exports: self.exports,
            aliases: self.aliases,
            distinct_underlying: self.distinct_underlying,
            modules: self.module_names,
            used_tys: self.used_tys,
            vtables: self.vtables,
            traits: self.traits,
            cimport_headers: self.cimport_headers,
        };
        (Ok(prog), self.diags)
    }

    fn check_impls(&mut self) {
        for i in 0..self.impls.len() {
            let im = self.impls[i].clone();
            if let Some(tn) = &im.trait_name {
                let trait_id = match self.lookup_item(im.module, tn) {
                    Some(ItemRef::Trait(t)) => t,
                    _ => {
                        self.error(im.span, format!("cannot find trait `{}`", tn));
                        continue;
                    }
                };
                let tdef = self.traits[trait_id as usize].clone();
                for m in &tdef.methods {
                    if m.body.is_none() && !im.methods.iter().any(|(n, _)| n == &m.name) {
                        self.error_note(im.span, format!("impl of `{}` is missing method `{}`", tn, m.name), Some(m.span), "required by this trait method");
                    }
                }
                for (n, fid) in &im.methods {
                    if !tdef.methods.iter().any(|m| &m.name == n) {
                        let sp = self.fns[*fid as usize].decl.name_span;
                        self.error(sp, format!("`{}` is not a method of trait `{}`", n, tn));
                    }
                }
                for at in &tdef.assoc_types {
                    if !im.assoc_types.iter().any(|(n, _)| n == at) {
                        self.error(im.span, format!("impl of `{}` is missing associated type `{}`", tn, at));
                    }
                }
            }
        }
    }

    pub fn resolve_const(&mut self, id: DefId) -> Option<(TyId, TExpr)> {
        if let Some(r) = &self.consts[id as usize].resolved {
            return Some(r.clone());
        }
        if self.consts[id as usize].in_progress {
            let sp = self.consts[id as usize].span;
            self.error(sp, "constant depends on itself");
            return None;
        }
        self.consts[id as usize].in_progress = true;
        let c = self.consts[id as usize].clone();
        let saved = self.cur.take();
        let void = self.tys.void();
        let mut ctx = FnCtx::new(void, c.module, &format!("const {}", c.name));
        ctx.is_comptime = true;
        self.cur = Some(ctx);
        let expected = c.ty_expr.as_ref().map(|t| self.resolve_type(t, &HashMap::new(), None, c.module));
        let te = self.check_expr(&c.value, expected);
        let te = match expected {
            Some(t) => self.coerce_or_error(te, t, "constant initializer"),
            None => te,
        };
        let te = self.finalize_expr(te);
        let ty = self.tys.resolve(te.ty, true);
        // evaluate at compile time
        let te = match crate::comptime::eval_const_expr(self, &te) {
            Some(v) => TExpr { kind: TExprKind::Value(v), ty, span: te.span },
            None => {
                self.error(te.span, "constant initializer must be evaluable at compile time");
                te
            }
        };
        self.cur = saved;
        self.consts[id as usize].in_progress = false;
        self.consts[id as usize].resolved = Some((ty, te.clone()));
        Some((ty, te))
    }

    fn resolve_global(&mut self, id: DefId) {
        let g = self.globals[id as usize].clone();
        let ty = self.resolve_type(&g.ty_expr, &HashMap::new(), None, g.module);
        self.globals[id as usize].ty = Some(ty);
        if let Some(v) = &g.value {
            let saved = self.cur.take();
            let void = self.tys.void();
            let mut ctx = FnCtx::new(void, g.module, &format!("global {}", g.name));
            ctx.is_comptime = true;
            self.cur = Some(ctx);
            let te = self.check_expr(v, Some(ty));
            let te = self.coerce_or_error(te, ty, "global initializer");
            let te = self.finalize_expr(te);
            let te = match crate::comptime::eval_const_expr(self, &te) {
                Some(val) => TExpr { kind: TExprKind::Value(val), ty, span: te.span },
                None => {
                    self.error(te.span, "global initializer must be evaluable at compile time (globals are initialized at compile time or not at all)");
                    te
                }
            };
            self.cur = saved;
            self.globals[id as usize].init = Some(te);
        }
    }

    fn check_embedded_constraints(&mut self) {
        // S2: in embedded mode, any reachable mutable global is an error.
        if !self.embedded_mode {
            return;
        }
        let uses = self.global_uses.clone();
        let mut reported = HashSet::new();
        for (gid, span, fname) in uses {
            if reported.insert(gid) {
                let g = &self.globals[gid as usize];
                let gname = g.name.clone();
                let gspan = g.span;
                self.error_note(
                    span,
                    format!("mutable global `{}` is reachable from `{}`, but this project declares an embeddable artifact (S2: no process-global state)", gname, fname),
                    Some(gspan),
                    "declared here; keep the state in a handle the host holds, or in the context",
                );
            }
        }
    }

    // ----- function bodies -----------------------------------------------------

    pub fn check_fn_body(&mut self, inst: InstId, fid: FnDefId) {
        let saved_cur = self.cur.take();
        self.check_fn_body_inner(inst, fid);
        self.cur = saved_cur;
    }

    fn check_fn_body_inner(&mut self, inst: InstId, fid: FnDefId) {
        let def = self.fns[fid as usize].clone();
        let (generics, self_ty) = self.inst_generics.get(&inst).cloned().unwrap_or_default();
        let f = self.funcs[inst as usize].clone();
        let body = match &def.decl.body {
            Some(b) => b.clone(),
            None => {
                if !def.decl.extern_c && def.impl_id.is_none() {
                    self.error(def.decl.span, format!("function `{}` has no body", def.decl.name));
                }
                return;
            }
        };
        let mut ctx = FnCtx::new(f.ret, def.module, &def.decl.name);
        ctx.generics = generics;
        ctx.self_ty = self_ty;
        ctx.locals = f.locals.clone();
        for &p in &f.params {
            let name = ctx.locals[p as usize].name.clone();
            ctx.scopes[0].push((name, ScopeEntry { local: p, auto_deref: false }));
        }
        // where-clause trait bounds
        for w in &def.decl.wheres {
            if let Some(&t) = ctx.generics.get(&w.param) {
                for b in &w.bounds {
                    if !self.type_satisfies_trait(t, b, def.module) {
                        let tn = self.type_name(t);
                        self.error_note(w.span, format!("type `{}` does not implement trait `{}`", tn, b), None, format!("required by the bound on `{}`", w.param));
                    }
                }
            }
        }
        self.cur = Some(ctx);
        let ret = f.ret;
        let tb = self.check_block(&body, Some(ret), None);
        let tb = self.finalize_block_as_body(tb, ret);
        let ctx = self.cur.take().unwrap();
        let func = &mut self.funcs[inst as usize];
        func.locals = ctx.locals;
        func.body = Some(tb);
        func.own_effects = ctx.own_effects;
        func.witnesses = ctx.witnesses;
        func.callees = ctx.callees;
        func.moved = ctx.moved;
    }

    /// Check a function body block against the return type: the tail value (if any)
    /// must coerce to the return type, and a non-void function must return on all paths.
    pub fn finalize_block_as_body(&mut self, mut tb: TBlock, ret: TyId) -> TBlock {
        let ret_r = self.tys.resolve(ret, true);
        let is_void_ret = matches!(self.tys.kind(ret_r), TyKind::Void) || matches!(self.tys.kind(ret_r), TyKind::ErrUnion(_, e) if matches!(self.tys.kind(self.tys.shallow(*e)), TyKind::Void));
        if let Some(tail) = tb.tail.take() {
            let tail = *tail;
            let tt = self.tys.resolve(tail.ty, false);
            if is_void_ret && matches!(self.tys.kind(tt), TyKind::Void | TyKind::Never) {
                tb.stmts.push(TStmt::Expr(tail));
            } else if is_void_ret {
                // discard-with-value: unused value
                let tn = self.type_name(tt);
                self.error(tail.span, format!("this value of type `{}` is unused; the function returns void. Discard it with `_ = ...` or return it", tn));
                tb.stmts.push(TStmt::Expr(tail));
            } else {
                let sp = tail.span;
                let v = self.coerce_or_error(tail, ret, "return value");
                tb.stmts.push(TStmt::Return { value: Some(v), span: sp });
            }
        } else if !is_void_ret && !self.block_diverges(&tb) {
            let tn = self.type_name(ret);
            self.error(tb.span, format!("function must return a value of type `{}` on every path; add a `return` or a tail expression", tn));
        }
        tb
    }

    pub fn block_diverges(&self, b: &TBlock) -> bool {
        if let Some(t) = &b.tail {
            if self.expr_diverges(t) {
                return true;
            }
        }
        b.stmts.iter().any(|s| self.stmt_diverges(s))
    }

    pub fn stmt_diverges(&self, s: &TStmt) -> bool {
        match s {
            TStmt::Return { .. } | TStmt::Break { .. } | TStmt::Continue { .. } => true,
            TStmt::Expr(e) => self.expr_diverges(e),
            TStmt::Let { init: Some(e), .. } => self.expr_diverges(e),
            TStmt::Block(b) | TStmt::Using { body: b, .. } => self.block_diverges(b),
            TStmt::While { cond, .. } => matches!(cond.kind, TExprKind::Bool(true)) && !Self::block_has_break(s),
            _ => false,
        }
    }

    fn block_has_break(s: &TStmt) -> bool {
        fn walk_block(b: &TBlock, depth: u32) -> bool {
            b.stmts.iter().any(|s| walk(s, depth)) || b.tail.as_ref().map(|t| walk_expr(t, depth)).unwrap_or(false)
        }
        fn walk(s: &TStmt, depth: u32) -> bool {
            match s {
                TStmt::Break { .. } => depth == 0,
                TStmt::While { body, .. } | TStmt::ForRange { body, .. } | TStmt::ForSlice { body, .. } => walk_block(body, depth + 1),
                TStmt::Block(b) => walk_block(b, depth),
                TStmt::Expr(e) | TStmt::Let { init: Some(e), .. } => walk_expr(e, depth),
                _ => false,
            }
        }
        fn walk_expr(e: &TExpr, depth: u32) -> bool {
            match &e.kind {
                TExprKind::If { then, els, .. } => walk_block(then, depth) || els.as_ref().map(|b| walk_block(b, depth)).unwrap_or(false),
                TExprKind::IfCapture { then, els, .. } => walk_block(then, depth) || els.as_ref().map(|b| walk_block(b, depth)).unwrap_or(false),
                TExprKind::Block(b) => walk_block(b, depth),
                TExprKind::Match { arms, .. } => arms.iter().any(|a| walk_expr(&a.body, depth)),
                _ => false,
            }
        }
        match s {
            TStmt::While { body, .. } => walk_block(body, 0),
            _ => false,
        }
    }

    pub fn expr_diverges(&self, e: &TExpr) -> bool {
        let t = self.tys.shallow(e.ty);
        if matches!(self.tys.kind(t), TyKind::Never) {
            return true;
        }
        match &e.kind {
            TExprKind::Block(b) => self.block_diverges(b),
            TExprKind::If { then, els: Some(els), .. } => self.block_diverges(then) && self.block_diverges(els),
            TExprKind::Match { arms, .. } => !arms.is_empty() && arms.iter().all(|a| self.expr_diverges(&a.body)),
            _ => false,
        }
    }

    /// Does the type satisfy a trait bound?  Primitive types satisfy the builtin
    /// traits Eq, Ord, Hash, Format; user types satisfy them through derive or impl.
    pub fn type_satisfies_trait(&mut self, t: TyId, trait_name: &str, module: u32) -> bool {
        let t = self.tys.resolve(t, true);
        match trait_name {
            "Eq" | "Ord" | "Hash" | "Format" | "Copy" => match self.tys.kind(t).clone() {
                TyKind::Int(_) | TyKind::Float(_) | TyKind::Bool | TyKind::Char => return true,
                TyKind::Slice(_, e) if trait_name != "Copy" => {
                    if trait_name == "Ord" {
                        return matches!(self.tys.kind(self.tys.shallow(e)), TyKind::Int(IntTy::U8));
                    }
                    return self.type_satisfies_trait(e, trait_name, module);
                }
                TyKind::Str if trait_name != "Copy" => return true,
                TyKind::Struct(d, _) => {
                    if self.structs[d as usize].derives.iter().any(|x| x == trait_name) {
                        return true;
                    }
                }
                TyKind::Enum(d, _) => {
                    if self.enums[d as usize].derives.iter().any(|x| x == trait_name) {
                        return true;
                    }
                    if trait_name != "Copy" && self.enums[d as usize].variants.iter().all(|v| matches!(v.payload, VariantPayloadDef::Unit)) {
                        return true;
                    }
                }
                TyKind::Distinct(d) => {
                    let u = self.distinct_underlying[&d];
                    return self.type_satisfies_trait(u, trait_name, module);
                }
                _ => {}
            },
            _ => {}
        }
        self.find_trait_impl(t, trait_name, module).is_some()
    }

    /// Find an impl of `trait_name` for type `t`; returns the impl index and bound type params.
    pub fn find_trait_impl(&mut self, t: TyId, trait_name: &str, _module: u32) -> Option<(u32, HashMap<String, TyId>)> {
        for i in 0..self.impls.len() {
            let im = self.impls[i].clone();
            if im.trait_name.as_deref() != Some(trait_name) {
                continue;
            }
            if let Some(b) = self.match_impl_target(&im, t) {
                return Some((i as u32, b));
            }
        }
        None
    }

    /// Match an impl's target type pattern against a concrete type, binding impl type params.
    pub fn match_impl_target(&mut self, im: &ImplDef, t: TyId) -> Option<HashMap<String, TyId>> {
        let t = self.tys.resolve(t, true);
        let mut bindings = HashMap::new();
        if self.match_type_pattern(&im.target, t, &im.type_params, &mut bindings, im.module) {
            Some(bindings)
        } else {
            None
        }
    }

    fn match_type_pattern(&mut self, pat: &TypeExpr, t: TyId, params: &[String], bindings: &mut HashMap<String, TyId>, module: u32) -> bool {
        let t = self.tys.shallow(t);
        match pat {
            TypeExpr::Named { path, args, .. } if path.len() == 1 && params.contains(&path[0]) => {
                if let Some(&b) = bindings.get(&path[0]) {
                    return b == t;
                }
                bindings.insert(path[0].clone(), t);
                true
            }
            TypeExpr::Named { path, args, .. } if path.len() == 1 && !args.is_empty() => {
                let name = &path[0];
                match (name.as_str(), self.tys.kind(t).clone()) {
                    ("List", TyKind::List(e)) if args.len() == 1 => self.match_type_pattern(&args[0], e, params, bindings, module),
                    ("Map", TyKind::Map(k, v)) if args.len() == 2 => self.match_type_pattern(&args[0], k, params, bindings, module) && self.match_type_pattern(&args[1], v, params, bindings, module),
                    (_, TyKind::Struct(d, targs)) => {
                        matches!(self.lookup_item(module, name), Some(ItemRef::Struct(id)) if id == d)
                            && targs.len() == args.len()
                            && args.iter().zip(targs.iter()).all(|(a, &ta)| self.match_type_pattern(a, ta, params, bindings, module))
                    }
                    (_, TyKind::Enum(d, targs)) => {
                        matches!(self.lookup_item(module, name), Some(ItemRef::Enum(id)) if id == d)
                            && targs.len() == args.len()
                            && args.iter().zip(targs.iter()).all(|(a, &ta)| self.match_type_pattern(a, ta, params, bindings, module))
                    }
                    _ => false,
                }
            }
            TypeExpr::Slice { mutable, elem, .. } => match self.tys.kind(t).clone() {
                TyKind::Slice(m, e) if m == *mutable => self.match_type_pattern(elem, e, params, bindings, module),
                _ => false,
            },
            TypeExpr::Ptr { mutable, elem, .. } => match self.tys.kind(t).clone() {
                TyKind::Ptr(m, e) if m == *mutable => self.match_type_pattern(elem, e, params, bindings, module),
                _ => false,
            },
            TypeExpr::Optional { elem, .. } => match self.tys.kind(t).clone() {
                TyKind::Opt(e) => self.match_type_pattern(elem, e, params, bindings, module),
                _ => false,
            },
            TypeExpr::Array { len, elem, .. } => match self.tys.kind(t).clone() {
                TyKind::Array(n, e) => {
                    let want = self.const_eval_int(len, module, &HashMap::new());
                    want == Some(n as i128) && self.match_type_pattern(elem, e, params, bindings, module)
                }
                _ => false,
            },
            _ => {
                // concrete type: resolve and compare
                let saved = self.diags.len();
                let pt = self.resolve_type(pat, &HashMap::new(), None, module);
                self.diags.truncate(saved);
                let pt = self.tys.resolve(pt, true);
                pt == t
            }
        }
    }
}

impl<'a> Checker<'a> {
    /// Build (or reuse) the vtable of trait `trait_id` for concrete type `t`.
    /// Returns None when `t` does not implement the trait.
    pub fn vtable_for(&mut self, trait_id: DefId, t: TyId, span: Span) -> Option<u32> {
        let t = self.tys.resolve(t, true);
        if let Some(i) = self.vtables.iter().position(|(tr, ty, _)| *tr == trait_id && *ty == t) {
            return Some(i as u32);
        }
        let tdef = self.traits[trait_id as usize].clone();
        let tname = tdef.name.clone();
        let _ = self.find_trait_impl(t, &tname, tdef.module)?;
        let mut insts = Vec::new();
        for m in &tdef.methods {
            let fid = match self.find_inherent_method(t, &m.name) {
                Some(f) => f,
                None => {
                    let tn = self.type_name(t);
                    self.error(span, format!("`{}` implements `{}` but has no method `{}`", tn, tname, m.name));
                    return None;
                }
            };
            let def = self.fns[fid as usize].clone();
            if def.is_generic {
                self.error(span, format!("method `{}` is generic and cannot be called through `dyn {}`", m.name, tname));
                return None;
            }
            // impl type params from the concrete type
            let mut targs = Vec::new();
            if let Some(impl_id) = def.impl_id {
                let im = self.impls[impl_id as usize].clone();
                if let Some(b) = self.match_impl_target(&im, t) {
                    for p in &im.type_params {
                        targs.push(*b.get(p).unwrap_or(&t));
                    }
                }
            }
            let inst = self.instantiate(fid, targs, span);
            insts.push(inst);
        }
        self.vtables.push((trait_id, t, insts));
        Some((self.vtables.len() - 1) as u32)
    }

    /// The signature of trait method `idx` as seen through a trait object:
    /// parameter types after `self`, and the return type.
    pub fn dyn_method_sig(&mut self, trait_id: DefId, idx: usize, span: Span) -> Option<(Vec<TyId>, TyId, bool)> {
        let tdef = self.traits[trait_id as usize].clone();
        let m = tdef.methods.get(idx)?.clone();
        let self_param = m.params.first().filter(|p| p.name == "self")?;
        let mutable = matches!(&self_param.ty, TypeExpr::Ptr { mutable: true, .. });
        // Self is only meaningful in receiver position for a trait object
        let placeholder = self.tys.intern(TyKind::Param("Self".into()));
        let mut ps = Vec::new();
        for p in m.params.iter().skip(1) {
            let t = self.resolve_type(&p.ty, &HashMap::new(), Some(placeholder), tdef.module);
            if self.type_mentions(t, placeholder) {
                self.error(span, format!("method `{}` uses `Self` in a parameter and cannot be called through `dyn {}`", m.name, tdef.name));
                return None;
            }
            ps.push(t);
        }
        let ret = match &m.ret {
            Some(r) => self.resolve_type(r, &HashMap::new(), Some(placeholder), tdef.module),
            None => self.tys.void(),
        };
        if self.type_mentions(ret, placeholder) {
            self.error(span, format!("method `{}` returns `Self` and cannot be called through `dyn {}`", m.name, tdef.name));
            return None;
        }
        Some((ps, ret, mutable))
    }

    fn type_mentions(&self, t: TyId, needle: TyId) -> bool {
        let t = self.tys.shallow(t);
        if t == needle {
            return true;
        }
        match self.tys.kind(t).clone() {
            TyKind::Struct(_, args) | TyKind::Enum(_, args) | TyKind::Tuple(args) => args.iter().any(|&a| self.type_mentions(a, needle)),
            TyKind::Array(_, e) | TyKind::Slice(_, e) | TyKind::Ptr(_, e) | TyKind::Opt(e) | TyKind::Weak(e) | TyKind::ErrUnion(_, e) | TyKind::List(e) => self.type_mentions(e, needle),
            TyKind::Map(k, v) => self.type_mentions(k, needle) || self.type_mentions(v, needle),
            TyKind::Fn(ps, r, _) => ps.iter().any(|&p| self.type_mentions(p, needle)) || self.type_mentions(r, needle),
            _ => false,
        }
    }
}

pub fn sanitize(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else if c == '(' || c == ',' || c == ')' || c == ' ' || c == '[' || c == ']' || c == '*' || c == '?' || c == '!' || c == '.' {
            if !out.ends_with('_') {
                out.push('_');
            }
        }
    }
    out.trim_end_matches('_').to_string()
}

pub fn round_up(v: u64, a: u64) -> u64 {
    if a == 0 {
        v
    } else {
        (v + a - 1) / a * a
    }
}
