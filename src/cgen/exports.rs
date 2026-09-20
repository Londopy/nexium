//! Export boundary (section 4.1): C wrappers around exported functions, the CLI
//! entry point, and the test runner.
//!
//! An exported function whose return type is not an error union and which is
//! proven `!panics` gets a plain C signature. Every other export returns an
//! `int32_t` status (0 = ok, otherwise an error code; panics become the
//! `Panic` error) and delivers its value through an out-parameter, so a panic
//! never crosses the boundary (S3) and no runtime setup is required (S1).

use super::*;
use crate::ast::StructKind;

#[derive(Clone, Debug, PartialEq)]
pub enum ScalarKind {
    Int(IntTy),
    F32,
    F64,
    Bool,
    Char,
}

#[derive(Clone, Debug)]
pub enum ParamKind {
    Scalar(ScalarKind),
    Slice { elem_c: String, elem: ScalarKind, mutable: bool },
    Struct { c_name: String, fields: Vec<(String, ScalarKind, String)> },
}

#[derive(Clone, Debug)]
pub struct ExportParam {
    pub name: String,
    pub kind: ParamKind,
}

#[derive(Clone, Debug)]
pub struct ExportInfo {
    pub name: String,
    pub params: Vec<ExportParam>,
    /// None for void
    pub ret: Option<(String, ParamKind)>,
    pub status: bool,
    pub effects: String,
    pub doc: Vec<String>,
}

pub fn scalar_c(k: &ScalarKind) -> String {
    match k {
        ScalarKind::Int(i) => i.c_name().to_string(),
        ScalarKind::F32 => "float".into(),
        ScalarKind::F64 => "double".into(),
        ScalarKind::Bool => "bool".into(),
        ScalarKind::Char => "uint32_t".into(),
    }
}

impl Gen {
    fn scalar_kind(&mut self, t: TyId) -> Option<ScalarKind> {
        let t = self.res(t);
        Some(match self.p.tys.kind(t).clone() {
            TyKind::Int(i) => ScalarKind::Int(i),
            TyKind::Float(FloatTy::F32) => ScalarKind::F32,
            TyKind::Float(FloatTy::F64) => ScalarKind::F64,
            TyKind::Bool => ScalarKind::Bool,
            TyKind::Char => ScalarKind::Char,
            TyKind::Distinct(d) => {
                let u = self.p.distinct_underlying[&d];
                return self.scalar_kind(u);
            }
            _ => return None,
        })
    }

    fn param_kind(&mut self, t: TyId, what: &str, fname: &str) -> Option<ParamKind> {
        if let Some(s) = self.scalar_kind(t) {
            return Some(ParamKind::Scalar(s));
        }
        let t = self.res(t);
        match self.p.tys.kind(t).clone() {
            TyKind::Slice(m, e) => match self.scalar_kind(e) {
                Some(ek) => Some(ParamKind::Slice { elem_c: scalar_c(&ek), elem: ek, mutable: m }),
                None => {
                    self.errors.push(format!("export `{}`: {} is a slice of a non-scalar type, which has no C representation", fname, what));
                    None
                }
            },
            TyKind::Struct(d, _) => {
                let def = self.p.structs[d as usize].clone();
                if def.layout != crate::ast::Layout::C || def.kind == StructKind::RefClass {
                    self.errors.push(format!("export `{}`: {} has type `{}`, which needs `layout(c)` to cross the boundary", fname, what, def.name));
                    return None;
                }
                let ftys = self.p.struct_field_tys.get(&t).cloned().unwrap_or_default();
                let mut fields = Vec::new();
                for (i, (f, &ft)) in def.fields.iter().zip(ftys.iter()).enumerate() {
                    match self.scalar_kind(ft) {
                        Some(k) => fields.push((f.name.clone(), k, format!("{}_{}", sanitize_ident(&f.name), i))),
                        None => {
                            self.errors.push(format!("export `{}`: field `{}` of `{}` is not a scalar", fname, f.name, def.name));
                            return None;
                        }
                    }
                }
                let cn = self.cty(t);
                Some(ParamKind::Struct { c_name: cn, fields })
            }
            _ => {
                let tn = self.p_type_name_pretty(t);
                self.errors.push(format!("export `{}`: {} has type `{}`, which has no C representation (use scalars, slices of scalars, or `layout(c)` structs)", fname, what, tn));
                None
            }
        }
    }

    pub fn p_type_name_pretty(&mut self, t: TyId) -> String {
        let t = self.res(t);
        match self.p.tys.kind(t).clone() {
            TyKind::List(_) => "List".into(),
            TyKind::Str => "String".into(),
            TyKind::Map(..) => "Map".into(),
            TyKind::Opt(_) => "optional".into(),
            TyKind::ErrUnion(..) => "error union".into(),
            TyKind::Struct(d, _) => self.p.structs[d as usize].name.clone(),
            TyKind::Enum(d, _) => self.p.enums[d as usize].name.clone(),
            other => format!("{:?}", other),
        }
    }

    pub fn export_info(&mut self, inst: InstId) -> Option<ExportInfo> {
        let f = self.p.funcs[inst as usize].clone();
        let mut params = Vec::new();
        for &p in &f.params {
            let l = f.locals[p as usize].clone();
            let k = self.param_kind(l.ty, &format!("parameter `{}`", l.name), &f.name)?;
            params.push(ExportParam { name: l.name.clone(), kind: k });
        }
        let ret_t = self.res(f.ret);
        let (payload, is_eu) = match self.p.tys.kind(ret_t).clone() {
            TyKind::ErrUnion(_, e) => (e, true),
            _ => (ret_t, false),
        };
        let ret = if self.is_void(payload) {
            None
        } else {
            let k = self.param_kind(payload, "the return value", &f.name)?;
            if matches!(k, ParamKind::Slice { .. }) {
                self.errors.push(format!("export `{}`: returning a slice across the boundary is not allowed (its lifetime cannot be expressed); return through a caller-provided buffer", f.name));
                return None;
            }
            let cn = match &k {
                ParamKind::Scalar(s) => scalar_c(s),
                ParamKind::Struct { c_name, .. } => c_name.clone(),
                _ => unreachable!(),
            };
            Some((cn, k))
        };
        let status = is_eu || f.effects.contains(Effects::PANICS);
        let doc = self.p_fn_doc(inst);
        Some(ExportInfo { name: f.name.clone(), params, ret, status, effects: f.effects.render(), doc })
    }

    fn p_fn_doc(&self, inst: InstId) -> Vec<String> {
        let f = &self.p.funcs[inst as usize];
        let _ = f;
        Vec::new()
    }

    pub fn export_wrapper(&mut self, inst: InstId) -> String {
        let info = match self.export_info(inst) {
            Some(i) => i,
            None => return String::new(),
        };
        let f = self.p.funcs[inst as usize].clone();
        let mut params: Vec<String> = Vec::new();
        let mut args: Vec<String> = vec!["&ctx".into()];
        for (i, p) in info.params.iter().enumerate() {
            let l = f.locals[f.params[i] as usize].clone();
            match &p.kind {
                ParamKind::Scalar(s) => {
                    params.push(format!("{} {}", scalar_c(s), p.name));
                    let cn = self.cty(l.ty);
                    args.push(format!("({})({})", cn, p.name));
                }
                ParamKind::Slice { elem_c, mutable, .. } => {
                    params.push(format!("{}{}* {}", if *mutable { "" } else { "const " }, elem_c, p.name));
                    params.push(format!("size_t {}_len", p.name));
                    let cn = self.cty(l.ty);
                    args.push(format!("(({}){{ ({}*){}, {}_len }})", cn, elem_c, p.name, p.name));
                }
                ParamKind::Struct { c_name, .. } => {
                    params.push(format!("{} {}", c_name, p.name));
                    args.push(p.name.clone());
                }
            }
        }
        let target = self.fn_c_name(inst);
        let ret_t = self.res(f.ret);
        let is_eu = matches!(self.p.tys.kind(ret_t), TyKind::ErrUnion(..));
        let mut out = String::new();
        let panic_id = self.p.error_names.iter().position(|n| n == "Panic").map(|i| i + 1).unwrap_or(0);
        let _ = writeln!(out, "/* export: {}{} */", info.name, if info.effects.is_empty() { String::new() } else { format!(" ({})", info.effects) });
        if !info.status {
            let ret_c = info.ret.as_ref().map(|(c, _)| c.clone()).unwrap_or_else(|| "void".into());
            let _ = writeln!(out, "NX_EXPORT {} {}({}) {{", ret_c, info.name, if params.is_empty() { "void".to_string() } else { params.join(", ") });
            let _ = writeln!(out, "  nx_ctx ctx = nx_default_ctx(0, NULL);");
            if info.ret.is_some() {
                let _ = writeln!(out, "  return ({}){}({});", ret_c, target, args.join(", "));
            } else {
                let _ = writeln!(out, "  {}({});", target, args.join(", "));
            }
            let _ = writeln!(out, "}}");
        } else {
            let mut all = params.clone();
            if let Some((c, _)) = &info.ret {
                all.push(format!("{}* out", c));
            }
            let _ = writeln!(out, "NX_EXPORT int32_t {}({}) {{", info.name, if all.is_empty() { "void".to_string() } else { all.join(", ") });
            let _ = writeln!(out, "  nx_ctx ctx = nx_default_ctx(0, NULL);\n  nx_boundary _nxb; nx_boundary* prev = nx_tls_boundary; nx_tls_boundary = &_nxb;");
            let _ = writeln!(out, "  if (setjmp(_nxb.jb)) {{ nx_tls_boundary = prev; return {}; }}", panic_id);
            if is_eu {
                let rcn = self.cty(ret_t);
                let _ = writeln!(out, "  {} r = {}({});\n  nx_tls_boundary = prev;\n  if (r.err) return (int32_t)r.err;", rcn, target, args.join(", "));
                if info.ret.is_some() {
                    let _ = writeln!(out, "  if (out) *out = r.val;");
                }
            } else if info.ret.is_some() {
                let rcn = self.cty(ret_t);
                let _ = writeln!(out, "  {} r = {}({});\n  nx_tls_boundary = prev;\n  if (out) *out = r;", rcn, target, args.join(", "));
            } else {
                let _ = writeln!(out, "  {}({});\n  nx_tls_boundary = prev;", target, args.join(", "));
            }
            let _ = writeln!(out, "  return 0;\n}}");
        }
        out
    }

    /// Library-level helper exports: error names and the last panic message.
    pub fn library_helpers(&mut self) -> String {
        let lib = sanitize_ident(&self.lib_name);
        format!(
            "NX_EXPORT const char* {lib}_error_name(int32_t code) {{ return nx_error_name((uint32_t)code); }}\nNX_EXPORT const char* {lib}_last_panic(void) {{ return nx_tls_last_panic; }}\n",
            lib = lib
        )
    }

    pub fn main_wrapper(&mut self) -> String {
        let main = match self.p.main {
            Some(m) => m,
            None => {
                self.errors.push("no `main` function found; add `fn main() { ... }` or declare a library artifact".into());
                return String::new();
            }
        };
        let f = self.p.funcs[main as usize].clone();
        let target = self.fn_c_name(main);
        let ret_t = self.res(f.ret);
        let mut out = String::new();
        let _ = writeln!(out, "int main(int argc, char** argv) {{\n  nx_ctx ctx = nx_default_ctx(argc, argv);\n  nx_ctx_track_self(&ctx);\n  nx_boundary b; nx_tls_boundary = &b;\n  if (setjmp(b.jb)) {{ fflush(stdout); fprintf(stderr, \"panic: %s\\n  at %s\\n\", b.msg, b.loc); return 101; }}");
        if !f.params.is_empty() {
            self.errors.push("`main` takes no parameters; read arguments with `os.args()`".into());
        }
        match self.p.tys.kind(ret_t).clone() {
            TyKind::Void => {
                let _ = writeln!(
                    out,
                    "  {}(&ctx);\n  fflush(stdout);\n  nx_ctx_release(&ctx);
  nx_leak_report(&ctx);\n  return 0;",
                    target
                );
            }
            TyKind::ErrUnion(_, e) => {
                let rcn = self.cty(ret_t);
                let _ = writeln!(
                    out,
                    "  {} r = {}(&ctx);\n  fflush(stdout);\n  nx_ctx_release(&ctx);
  nx_leak_report(&ctx);\n  if (r.err) {{ fprintf(stderr, \"error: %s\\n\", nx_error_name(r.err)); return 1; }}",
                    rcn, target
                );
                if matches!(self.kind_of(e), TyKind::Int(_)) {
                    let _ = writeln!(out, "  return (int)r.val;");
                } else {
                    let _ = writeln!(out, "  return 0;");
                }
            }
            TyKind::Int(_) => {
                let _ = writeln!(
                    out,
                    "  int code = (int){}(&ctx);\n  fflush(stdout);\n  nx_ctx_release(&ctx);
  nx_leak_report(&ctx);\n  return code;",
                    target
                );
            }
            _ => {
                self.errors.push("`main` must return `void`, `!void`, or an integer exit code".into());
            }
        }
        let _ = writeln!(out, "}}");
        out
    }

    pub fn test_runner(&mut self) -> String {
        let tests = self.p.tests.clone();
        let mut out = String::new();
        let _ = writeln!(out, "int main(int argc, char** argv) {{\n  nx_ctx ctx = nx_default_ctx(argc, argv);\n  const char* filter = NULL; int verbose = 0;\n  for (int ai = 1; ai < argc; ai++) {{ if (strcmp(argv[ai], \"--verbose\") == 0) verbose = 1; else filter = argv[ai]; }}\n  int passed = 0, failed = 0, skipped = 0;\n  nx_boundary b; uint64_t t0 = 0;");
        for t in tests {
            let f = self.p.funcs[t as usize].clone();
            let name = f.name.trim_start_matches("test:").to_string();
            let target = self.fn_c_name(t);
            let rcn = self.cty(f.ret);
            let esc = c_escape_bytes(name.as_bytes());
            let _ = writeln!(out, "  if (!filter || strstr({esc}, filter)) {{\n    nx_tls_boundary = &b; t0 = nx_time_monotonic_ns();\n    if (setjmp(b.jb)) {{ failed++; printf(\"FAIL  %s\\n      panic: %s\\n      at %s\\n\", {esc}, b.msg, b.loc); }}\n    else {{ {rcn} r = {target}(&ctx); if (r.err) {{ failed++; printf(\"FAIL  %s\\n      error: %s\\n\", {esc}, nx_error_name(r.err)); }} else {{ passed++; if (verbose) printf(\"ok    %s  (%.1f ms)\\n\", {esc}, (double)(nx_time_monotonic_ns() - t0) / 1e6); else printf(\"ok    %s\\n\", {esc}); }} }}\n    nx_tls_boundary = NULL;\n  }} else {{ skipped++; if (verbose) printf(\"skip  %s\\n\", {esc}); }}", esc = esc, rcn = rcn, target = target);
        }
        let _ = writeln!(out, "  printf(\"\\n%d passed, %d failed%s\\n\", passed, failed, skipped ? \" (some skipped by filter)\" : \"\");\n  return failed ? 1 : 0;\n}}");
        out
    }
}
