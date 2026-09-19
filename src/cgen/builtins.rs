//! Lowering of builtin operations (collections, formatting, math, platform).

use super::*;

impl Gen {
    fn err_id(&self, name: &str) -> usize {
        self.p.error_names.iter().position(|n| n == name).map(|i| i + 1).unwrap_or(0)
    }

    /// Deep-copy glue for resource types: `T f(nx_ctx*, const T*)`.
    pub fn clone_fn(&mut self, t: TyId) -> String {
        let t = self.res(t);
        let key = format!("clone:{}", t);
        if let Some(n) = self.thunks_by_key.get(&key) {
            return n.clone();
        }
        let m = self.mangle(t);
        let name = format!("nx_clone_{}", m);
        self.thunks_by_key.insert(key, name.clone());
        let cn = self.cty(t);
        let _ = writeln!(self.protos_out, "static {} {}(nx_ctx* c, const {}* v);", cn, name, cn);
        let mut body = String::new();
        match self.p.tys.kind(t).clone() {
            TyKind::Str => {
                let _ = writeln!(body, "  return nx_str_from(c, nx_str_slice(*v));");
            }
            TyKind::List(e) => {
                let en = self.cty(e);
                if self.needs_drop(e) {
                    let ec = self.clone_fn(e);
                    let _ = writeln!(body, "  {} r = {{0}}; r.ar = c->arena; nx_list_reserve(c, (nx_rawlist*)&r, sizeof({}), _Alignof({}), v->len);", cn, en, en);
                    let _ = writeln!(body, "  for (size_t i = 0; i < v->len; i++) r.ptr[i] = {}(c, &v->ptr[i]);\n  r.len = v->len; return r;", ec);
                } else {
                    let _ = writeln!(
                        body,
                        "  nx_rawlist r = nx_list_clone_raw(c, (const nx_rawlist*)v, sizeof({}), _Alignof({})); {} o; o.ptr = ({}*)r.ptr; o.len = r.len; o.cap = r.cap; o.ar = r.ar; return o;",
                        en, en, cn, en
                    );
                }
            }
            TyKind::Map(k, vt) => {
                let _ = writeln!(body, "  nx_map r = nx_map_clone_raw(c, v);");
                if self.needs_drop(k) || self.needs_drop(vt) {
                    let kn = self.cty(k);
                    let vn = self.cty(vt);
                    let _ = writeln!(body, "  {{ size_t i = 0; void* kp; void* vp; while (nx_map_next(&r, &i, &kp, &vp)) {{");
                    if self.needs_drop(k) {
                        let kc = self.clone_fn(k);
                        let _ = writeln!(body, "    {{ {} nk = {}(c, ({}*)kp); memcpy(kp, &nk, sizeof nk); }}", kn, kc, kn);
                    }
                    if self.needs_drop(vt) {
                        let vc = self.clone_fn(vt);
                        let _ = writeln!(body, "    {{ {} nv = {}(c, ({}*)vp); memcpy(vp, &nv, sizeof nv); }}", vn, vc, vn);
                    }
                    let _ = writeln!(body, "  }} }}");
                }
                let _ = writeln!(body, "  return r;");
            }
            TyKind::Struct(d, _) => {
                let def = self.p.structs[d as usize].clone();
                if def.kind == StructKind::RefClass {
                    let _ = writeln!(body, "  return ({})nx_retain(*v);", cn);
                } else {
                    let ftys = self.p.struct_field_tys.get(&t).cloned().unwrap_or_default();
                    let _ = writeln!(body, "  {} r = *v;", cn);
                    for (i, (f, &ft)) in def.fields.iter().zip(ftys.iter()).enumerate() {
                        if self.needs_drop(ft) {
                            let fc = self.clone_fn(ft);
                            let _ = writeln!(body, "  r.{n}_{i} = {fc}(c, &v->{n}_{i});", n = sanitize_ident(&f.name), i = i, fc = fc);
                        }
                    }
                    let _ = writeln!(body, "  return r;");
                }
            }
            TyKind::Weak(_) => {
                let _ = writeln!(body, "  return ({})nx_weak_new(*v);", cn);
            }
            TyKind::Opt(e) => {
                let ec = self.clone_fn(e);
                let _ = writeln!(body, "  {} r = *v; if (v->has) r.val = {}(c, &v->val); return r;", cn, ec);
            }
            TyKind::Array(n, e) => {
                let ec = self.clone_fn(e);
                let _ = writeln!(body, "  {} r; for (size_t i = 0; i < {}; i++) r.v[i] = {}(c, &v->v[i]); return r;", cn, n, ec);
            }
            TyKind::Tuple(ts) => {
                let _ = writeln!(body, "  {} r = *v;", cn);
                for (i, &e) in ts.iter().enumerate() {
                    if self.needs_drop(e) {
                        let ec = self.clone_fn(e);
                        let _ = writeln!(body, "  r.f{} = {}(c, &v->f{});", i, ec, i);
                    }
                }
                let _ = writeln!(body, "  return r;");
            }
            TyKind::Enum(..) => {
                let vtys = self.p.enum_variant_tys.get(&t).cloned().unwrap_or_default();
                let _ = writeln!(body, "  {} r = *v;\n  switch (v->tag) {{", cn);
                for (vi, tys) in vtys.iter().enumerate() {
                    let mut s = String::new();
                    for (i, &ft) in tys.iter().enumerate() {
                        if self.needs_drop(ft) {
                            let fc = self.clone_fn(ft);
                            let _ = writeln!(s, "    r.u.v{vi}.f{i} = {fc}(c, &v->u.v{vi}.f{i});", vi = vi, i = i, fc = fc);
                        }
                    }
                    if !s.is_empty() {
                        let _ = writeln!(body, "  case {}:\n{}    break;", vi, s);
                    }
                }
                let _ = writeln!(body, "  default: break;\n  }}\n  return r;");
            }
            _ => {
                let _ = writeln!(body, "  return *v;");
            }
        }
        let _ = writeln!(self.helpers_out, "static {} {}(nx_ctx* c, const {}* v) {{\n  NX_UNUSED(c);\n{}}}", cn, name, cn, body);
        name
    }

    /// Copy a value into a new owner: clone glue for resource types, plain copy otherwise.
    fn copy_value(&mut self, v: &str, t: TyId) -> String {
        if self.needs_drop(t) {
            let f = self.clone_fn(t);
            format!("{}(c, &({}))", f, v)
        } else {
            v.to_string()
        }
    }

    fn map_key_kind(&mut self, k: TyId) -> u32 {
        match self.kind_of(k) {
            TyKind::Slice(..) => 1,
            TyKind::Str => 2,
            _ => 0,
        }
    }

    fn write_value(&mut self, sink: &str, v: &str, t: TyId, spec: &str) {
        let t = self.res(t);
        let (mut width, mut left) = (0i32, false);
        let mut base = 10;
        let mut prec = -1i32;
        let mut exp = false;
        let mut as_char = false;
        let spec = spec.trim_start_matches(':');
        match spec {
            "x" => base = 16,
            "X" => base = 17,
            "b" => base = 2,
            "o" => base = 8,
            "e" => exp = true,
            "c" => as_char = true,
            s if s.starts_with('.') => prec = s[1..].parse().unwrap_or(-1),
            s if s.starts_with('>') => width = s[1..].parse().unwrap_or(0),
            s if s.starts_with('<') => {
                width = s[1..].parse().unwrap_or(0);
                left = true;
            }
            _ => {}
        }
        match self.p.tys.kind(t).clone() {
            TyKind::Int(_) => {
                if as_char {
                    self.line(format!("nx_w_char({}, (uint32_t)({}));", sink, v));
                } else {
                    self.line(format!("nx_w_int({}, (nx_i128)({}), {}, {}, {});", sink, v, base, width, left));
                }
            }
            TyKind::Char => self.line(format!("nx_w_char({}, {});", sink, v)),
            TyKind::Float(_) => self.line(format!("nx_w_float({}, (double)({}), {}, {}, {}, {});", sink, v, prec, exp, width, left)),
            TyKind::Bool => self.line(format!("nx_w_bool({}, {});", sink, v)),
            TyKind::Slice(..) => {
                if width > 0 {
                    self.line(format!("nx_w_pad({}, (const char*)({}).ptr, ({}).len, {}, {});", sink, v, v, width, left));
                } else {
                    self.line(format!("nx_w_sl({}, {});", sink, v));
                }
            }
            TyKind::Str => self.line(format!("nx_w_sl({}, nx_str_slice({}));", sink, v)),
            TyKind::ErrorSet(_) => self.line(format!("nx_w_cstr({}, nx_error_name({}));", sink, v)),
            TyKind::Enum(d, _) => {
                let names = self.enum_names_table(t);
                let n = self.p.enums[d as usize].variants.len();
                self.line(format!("nx_w_cstr({}, ({}).tag < {} ? {}[({}).tag] : \"?\");", sink, v, n, names, v));
            }
            TyKind::Opt(inner) => {
                let tmp = self.tmp();
                let cn = self.cty(t);
                self.line(format!("{} {} = {};", cn, tmp, v));
                self.line(format!("if (!{}.has) nx_w_cstr({}, \"null\"); else {{", tmp, sink));
                self.push_buf();
                self.write_value(sink, &format!("{}.val", tmp), inner, spec);
                let inner_c = self.pop_buf();
                self.body.last_mut().unwrap().push_str(&inner_c);
                self.line("}");
            }
            TyKind::Ptr(..) => self.line(format!("nx_w_int({}, (nx_i128)(uintptr_t)({}), 16, 0, false);", sink, v)),
            _ => self.line(format!("nx_w_cstr({}, \"?\");", sink)),
        }
    }

    fn enum_names_table(&mut self, t: TyId) -> String {
        let key = format!("enumnames:{}", t);
        if let Some(n) = self.thunks_by_key.get(&key) {
            return n.clone();
        }
        let m = self.mangle(t);
        let name = format!("nx_enum_names_{}", m);
        self.thunks_by_key.insert(key, name.clone());
        let d = match self.p.tys.kind(t) {
            TyKind::Enum(d, _) => *d,
            _ => 0,
        };
        let names: Vec<String> = self.p.enums[d as usize].variants.iter().map(|v| format!("\"{}\"", v.name)).collect();
        let _ = writeln!(self.data_out, "static const char* const {}[] = {{ {} }};", name, names.join(", "));
        name
    }

    /// Emit formatted output of `args` (fmt string first) into the sink variable.
    fn emit_format(&mut self, sink: &str, args: &[TExpr]) {
        let fmt = match &args[0].kind {
            TExprKind::Str(s) => s.clone(),
            _ => vec![],
        };
        let mut lit = Vec::new();
        let mut ai = 1;
        let mut i = 0;
        let flush = |g: &mut Gen, lit: &mut Vec<u8>| {
            if !lit.is_empty() {
                let l = g.string_literal(lit);
                g.line(format!("nx_w({}, (const uint8_t*){}, {});", sink, l, lit.len()));
                lit.clear();
            }
        };
        while i < fmt.len() {
            let c = fmt[i];
            if c == b'{' {
                if i + 1 < fmt.len() && fmt[i + 1] == b'{' {
                    lit.push(b'{');
                    i += 2;
                    continue;
                }
                let mut j = i + 1;
                while j < fmt.len() && fmt[j] != b'}' {
                    j += 1;
                }
                let spec = String::from_utf8_lossy(&fmt[i + 1..j]).to_string();
                flush(self, &mut lit);
                if let Some(a) = args.get(ai) {
                    let v = self.simple(a);
                    let t = a.ty;
                    self.write_value(sink, &v, t, &spec);
                }
                ai += 1;
                i = j + 1;
            } else if c == b'}' && i + 1 < fmt.len() && fmt[i + 1] == b'}' {
                lit.push(b'}');
                i += 2;
            } else {
                lit.push(c);
                i += 1;
            }
        }
        flush(self, &mut lit);
    }

    pub fn builtin(&mut self, op: Builtin, args: &[TExpr], tys: &[TyId], e: &TExpr) -> String {
        let loc = self.loc(e.span);
        let fast = self.opts.mode == BuildMode::FastRelease;
        match op {
            Builtin::Print | Builtin::Println | Builtin::Eprintln => {
                let s = self.tmp();
                let stream = if op == Builtin::Eprintln { "c->err" } else { "c->out" };
                self.line(format!("nx_sink {} = nx_sink_file(c, {});", s, stream));
                self.emit_format(&format!("&{}", s), args);
                if op != Builtin::Print {
                    self.line(format!("nx_w(&{}, (const uint8_t*)\"\\n\", 1);", s));
                }
                self.line(format!("nx_sink_flush(&{});", s));
                "0".into()
            }
            Builtin::Format => {
                let out = self.tmp();
                let s = self.tmp();
                self.line(format!("nx_string {} = {{0}}; {}.ar = c->arena;", out, out));
                self.line(format!("nx_sink {} = nx_sink_str(c, &{});", s, out));
                self.emit_format(&format!("&{}", s), args);
                out
            }
            Builtin::IntToStr | Builtin::FloatToStr => {
                let out = self.tmp();
                let s = self.tmp();
                self.line(format!("nx_string {} = {{0}}; {}.ar = c->arena;", out, out));
                self.line(format!("nx_sink {} = nx_sink_str(c, &{});", s, out));
                let v = self.simple(&args[0]);
                self.write_value(&format!("&{}", s), &v, args[0].ty, "");
                out
            }
            Builtin::Expect => {
                let c = self.simple(&args[0]);
                let msg = match &args[1].kind {
                    TExprKind::Str(s) => String::from_utf8_lossy(s).to_string(),
                    _ => String::new(),
                };
                let esc = c_escape_bytes(format!("expectation failed: {}", msg).as_bytes());
                self.line(format!("if (!({})) nx_panic({}, {});", c, esc, loc));
                "0".into()
            }
            Builtin::Panic => {
                let m = self.simple(&args[0]);
                self.line(format!("{{ char _pb[256]; snprintf(_pb, sizeof _pb, \"%.*s\", (int)({}.len > 255 ? 255 : {}.len), (const char*){}.ptr); nx_panic(_pb, {}); }}", m, m, m, loc));
                "0".into()
            }
            Builtin::Len => {
                let t = self.res(args[0].ty);
                match self.p.tys.kind(t).clone() {
                    TyKind::Array(n, _) => {
                        let _ = self.simple(&args[0]);
                        format!("((size_t){})", n)
                    }
                    _ => {
                        let v = self.simple(&args[0]);
                        format!("(({}).len)", v)
                    }
                }
            }
            Builtin::PtrAdd => {
                if let TExprKind::Str(s) = &args[0].kind {
                    let lit = self.string_literal(s);
                    return format!("((uint8_t*){})", lit);
                }
                let v = self.simple(&args[0]);
                format!("(({}).ptr)", v)
            }
            Builtin::ListNew => {
                let cn = self.cty(e.ty);
                format!("(({}){{NULL, 0, 0, c->arena}})", cn)
            }
            Builtin::ListWithCapacity => {
                let cn = self.cty(e.ty);
                let en = self.cty(tys[0]);
                let n = self.simple(&args[0]);
                let t = self.tmp();
                self.line(format!("{} {} = {{0}}; {}.ar = c->arena; nx_list_reserve(c, (nx_rawlist*)&{}, sizeof({}), _Alignof({}), {});", cn, t, t, t, en, en, n));
                t
            }
            Builtin::ListFromSlice => {
                let cn = self.cty(e.ty);
                let en = self.cty(tys[0]);
                let s = self.simple(&args[0]);
                let t = self.tmp();
                self.line(format!("{} {} = {{0}}; {}.ar = c->arena; nx_list_reserve(c, (nx_rawlist*)&{}, sizeof({}), _Alignof({}), {}.len);", cn, t, t, t, en, en, s));
                if self.needs_drop(tys[0]) {
                    let cf = self.clone_fn(tys[0]);
                    self.line(format!("for (size_t i = 0; i < {}.len; i++) {}.ptr[i] = {}(c, &{}.ptr[i]);", s, t, cf, s));
                } else {
                    self.line(format!("if ({}.len) memcpy({}.ptr, {}.ptr, {}.len * sizeof({}));", s, t, s, s, en));
                }
                self.line(format!("{}.len = {}.len;", t, s));
                t
            }
            Builtin::ListAppend => {
                let en = self.cty(tys[0]);
                let v = self.expr_owned(&args[1]);
                let vt = self.bind_tmp(&v, tys[0]);
                let l = self.place(&args[0]);
                let lp = self.tmp();
                let ln = self.cty(args[0].ty);
                self.line(format!("{}* {} = &({});", ln, lp, l));
                self.line(format!("if ({}->len == {}->cap) nx_list_grow(c, (nx_rawlist*){}, sizeof({}), _Alignof({}), {}->len + 1);", lp, lp, lp, en, en, lp));
                self.line(format!("{}->ptr[{}->len++] = {};", lp, lp, vt));
                "0".into()
            }
            Builtin::ListPop => {
                let l = self.place(&args[0]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                self.line(format!("{} {}; if (({}).len) {{ {}.has = true; {}.val = ({}).ptr[--({}).len]; }} else {}.has = false;", cn, t, l, t, t, l, l, t));
                t
            }
            Builtin::ListClear => {
                let l = self.place(&args[0]);
                if self.needs_drop(tys[0]) {
                    let d = self.drop_fn(tys[0]);
                    self.line(format!("for (size_t i = 0; i < ({}).len; i++) {}(c, &({}).ptr[i]);", l, d, l));
                }
                self.line(format!("({}).len = 0;", l));
                "0".into()
            }
            Builtin::ListClone | Builtin::StringClone | Builtin::MapClone => {
                let v = self.simple(&args[0]);
                let f = self.clone_fn(e.ty);
                format!("{}(c, &{})", f, v)
            }
            Builtin::ListLast => {
                let s = self.simple(&args[0]);
                let last = matches!(args[1].kind, TExprKind::Int(1));
                let cn = self.cty(e.ty);
                let t = self.tmp();
                let idx = if last { format!("{}.len - 1", s) } else { "0".to_string() };
                self.line(format!("{} {}; if ({}.len) {{ {}.has = true; {}.val = {}.ptr[{}]; }} else {}.has = false;", cn, t, s, t, t, s, idx, t));
                t
            }
            Builtin::ListInsert => {
                let en = self.cty(tys[0]);
                let i = self.simple(&args[1]);
                let v = self.expr_owned(&args[2]);
                let vt = self.bind_tmp(&v, tys[0]);
                let l = self.place(&args[0]);
                let ln = self.cty(args[0].ty);
                let lp = self.tmp();
                self.line(format!("{}* {} = &({});", ln, lp, l));
                self.line(format!("if ({} > {}->len) nx_panic_bounds({}, {}->len, {});", i, lp, i, lp, loc));
                self.line(format!("if ({}->len == {}->cap) nx_list_grow(c, (nx_rawlist*){}, sizeof({}), _Alignof({}), {}->len + 1);", lp, lp, lp, en, en, lp));
                self.line(format!("memmove({}->ptr + {} + 1, {}->ptr + {}, ({}->len - {}) * sizeof({}));", lp, i, lp, i, lp, i, en));
                self.line(format!("{}->ptr[{}] = {}; {}->len++;", lp, i, vt, lp));
                "0".into()
            }
            Builtin::ListRemove | Builtin::ListSwapRemove => {
                let en = self.cty(tys[0]);
                let i = self.simple(&args[1]);
                let l = self.place(&args[0]);
                let ln = self.cty(args[0].ty);
                let lp = self.tmp();
                let t = self.tmp();
                self.line(format!("{}* {} = &({});", ln, lp, l));
                self.line(format!("if ({} >= {}->len) nx_panic_bounds({}, {}->len, {});", i, lp, i, lp, loc));
                self.line(format!("{} {} = {}->ptr[{}];", en, t, lp, i));
                if op == Builtin::ListRemove {
                    self.line(format!("memmove({}->ptr + {}, {}->ptr + {} + 1, ({}->len - {} - 1) * sizeof({}));", lp, i, lp, i, lp, i, en));
                } else {
                    self.line(format!("{}->ptr[{}] = {}->ptr[{}->len - 1];", lp, i, lp, lp));
                }
                self.line(format!("{}->len--;", lp));
                t
            }
            Builtin::ListExtend => {
                let en = self.cty(tys[0]);
                let s = self.simple(&args[1]);
                let l = self.place(&args[0]);
                let ln = self.cty(args[0].ty);
                let lp = self.tmp();
                self.line(format!("{}* {} = &({});", ln, lp, l));
                self.line(format!("nx_list_reserve(c, (nx_rawlist*){}, sizeof({}), _Alignof({}), {}.len);", lp, en, en, s));
                self.line(format!("if ({}.len) memcpy({}->ptr + {}->len, {}.ptr, {}.len * sizeof({})); {}->len += {}.len;", s, lp, lp, s, s, en, lp, s));
                "0".into()
            }
            Builtin::ListReserve => {
                let en = self.cty(tys[0]);
                let n = self.simple(&args[1]);
                let l = self.place(&args[0]);
                self.line(format!("nx_list_reserve(c, (nx_rawlist*)&({}), sizeof({}), _Alignof({}), {});", l, en, en, n));
                "0".into()
            }
            Builtin::ListItems => {
                let v = self.simple(&args[0]);
                v
            }
            Builtin::StringNew | Builtin::StringWithCapacity => {
                let t = self.tmp();
                self.line(format!("nx_string {} = {{0}}; {}.ar = c->arena;", t, t));
                if op == Builtin::StringWithCapacity {
                    let n = self.simple(&args[0]);
                    self.line(format!("nx_str_reserve(c, &{}, {});", t, n));
                }
                t
            }
            Builtin::StringFrom => {
                let s = self.simple(&args[0]);
                let t = self.tmp();
                self.line(format!("nx_string {} = nx_str_from(c, {});", t, s));
                t
            }
            Builtin::StringAppend => {
                let s = self.simple(&args[1]);
                let l = self.place(&args[0]);
                self.line(format!("nx_str_append(c, &({}), {}.ptr, {}.len);", l, s, s));
                "0".into()
            }
            Builtin::StringAppendChar => {
                let ch = self.simple(&args[1]);
                let l = self.place(&args[0]);
                self.line(format!("nx_str_append_char(c, &({}), {});", l, ch));
                "0".into()
            }
            Builtin::StringPushByte => {
                let b = self.simple(&args[1]);
                let l = self.place(&args[0]);
                self.line(format!("{{ uint8_t _b = (uint8_t)({}); nx_str_append(c, &({}), &_b, 1); }}", b, l));
                "0".into()
            }
            Builtin::StringClear => {
                let l = self.place(&args[0]);
                self.line(format!("({}).len = 0;", l));
                "0".into()
            }
            Builtin::StringPop => {
                let l = self.place(&args[0]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                self.line(format!("{} {}; if (({}).len) {{ {}.has = true; {}.val = ({}).ptr[--({}).len]; }} else {}.has = false;", cn, t, l, t, t, l, l, t));
                t
            }
            Builtin::StringBytes => {
                let v = self.simple(&args[0]);
                format!("nx_str_slice({})", v)
            }
            Builtin::MapNew => {
                let kn = self.cty(tys[0]);
                let vn = if self.is_void(tys[1]) { "char".to_string() } else { self.cty(tys[1]) };
                let kind = self.map_key_kind(tys[0]);
                format!("nx_map_new(c, sizeof({}), sizeof({}), {})", kn, vn, kind)
            }
            Builtin::MapPut => {
                let kn = self.cty(tys[0]);
                let vn = self.cty(tys[1]);
                let k = self.expr_owned(&args[1]);
                let kt = self.bind_tmp(&k, tys[0]);
                let v = self.expr_owned(&args[2]);
                let vt = self.bind_tmp(&v, tys[1]);
                let m = self.place(&args[0]);
                let ok = self.tmp();
                let ov = self.tmp();
                self.line(format!("{} {}; {} {};", kn, ok, vn, ov));
                self.line(format!("if (nx_map_put(c, &({}), &{}, &{}, &{}, &{})) {{", m, kt, vt, ok, ov));
                if self.needs_drop(tys[0]) {
                    let d = self.drop_fn(tys[0]);
                    self.line(format!("  {}(c, &{});", d, ok));
                }
                if self.needs_drop(tys[1]) {
                    let d = self.drop_fn(tys[1]);
                    self.line(format!("  {}(c, &{});", d, ov));
                }
                self.line("}");
                "0".into()
            }
            Builtin::MapGet => {
                let kn = self.cty(tys[0]);
                let vn = self.cty(tys[1]);
                let m = self.simple(&args[0]);
                let k = self.simple(&args[1]);
                let kt = self.bind_tmp(&k, args[1].ty);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                let p = self.tmp();
                let _ = kn;
                self.line(format!("{} {}; {}* {} = ({}*)nx_map_get(&({}), &{});", cn, t, vn, p, vn, m, kt));
                self.line(format!("if ({}) {{ {}.has = true; {}.val = *{}; }} else {}.has = false;", p, t, t, p, t));
                t
            }
            Builtin::MapContains => {
                let m = self.simple(&args[0]);
                let k = self.simple(&args[1]);
                let kt = self.bind_tmp(&k, args[1].ty);
                format!("(nx_map_get(&({}), &{}) != NULL)", m, kt)
            }
            Builtin::MapRemove => {
                let kn = self.cty(tys[0]);
                let vn = self.cty(tys[1]);
                let k = self.simple(&args[1]);
                let kt = self.bind_tmp(&k, args[1].ty);
                let m = self.place(&args[0]);
                let ok = self.tmp();
                let ov = self.tmp();
                let r = self.tmp();
                self.line(format!("{} {}; {} {}; bool {} = nx_map_remove(&({}), &{}, &{}, &{});", kn, ok, vn, ov, r, m, kt, ok, ov));
                if self.needs_drop(tys[0]) || self.needs_drop(tys[1]) {
                    self.line(format!("if ({}) {{", r));
                    if self.needs_drop(tys[0]) {
                        let d = self.drop_fn(tys[0]);
                        self.line(format!("  {}(c, &{});", d, ok));
                    }
                    if self.needs_drop(tys[1]) {
                        let d = self.drop_fn(tys[1]);
                        self.line(format!("  {}(c, &{});", d, ov));
                    }
                    self.line("}");
                }
                r
            }
            Builtin::MapClear => {
                let m = self.place(&args[0]);
                let d = self.drop_fn(args[0].ty);
                self.line(format!("{}(c, &({}));", d, m));
                "0".into()
            }
            Builtin::MapKeys | Builtin::MapValues => {
                let m = self.simple(&args[0]);
                let is_keys = op == Builtin::MapKeys;
                let et = if is_keys { tys[0] } else { tys[1] };
                let en = self.cty(et);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                self.line(format!("{} {} = {{0}}; {}.ar = c->arena; nx_list_reserve(c, (nx_rawlist*)&{}, sizeof({}), _Alignof({}), ({}).len);", cn, t, t, t, en, en, m));
                self.line(format!("{{ size_t i = 0; void* kp; void* vp; while (nx_map_next(&({}), &i, &kp, &vp)) {{", m));
                let src = format!("(*({}*){})", en, if is_keys { "kp" } else { "vp" });
                let cp = self.copy_value(&src, et);
                self.line(format!("  {}.ptr[{}.len++] = {};", t, t, cp));
                self.line("} }");
                t
            }
            Builtin::SliceEq => {
                let a = self.simple(&args[0]);
                let b = self.simple(&args[1]);
                let st = args[0].ty;
                self.eq_expr(st, &a, &b)
            }
            Builtin::SliceFind if tys.len() == 2 => {
                let a = self.simple(&args[0]);
                let b = self.simple(&args[1]);
                format!("nx_sl_cmp({}, {})", a, b)
            }
            Builtin::SliceFind => {
                let a = self.simple(&args[0]);
                let n = self.simple(&args[1]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                self.line(format!("{} {}; {{ size_t _i; {}.has = nx_sl_find({}, {}, &_i); {}.val = _i; }}", cn, t, t, a, n, t));
                t
            }
            Builtin::SliceCopy => {
                let en = self.cty(tys[0]);
                let d = self.simple(&args[0]);
                let s = self.simple(&args[1]);
                if !fast {
                    self.line(format!("if ({}.len > {}.len) nx_panic(\"copy source is longer than the destination\", {});", s, d, loc));
                }
                self.line(format!("if ({}.len) memmove({}.ptr, {}.ptr, {}.len * sizeof({}));", s, d, s, s, en));
                "0".into()
            }
            Builtin::SliceFill => {
                let d = self.simple(&args[0]);
                let v = self.simple(&args[1]);
                self.line(format!("for (size_t i = 0; i < {}.len; i++) {}.ptr[i] = {};", d, d, v));
                "0".into()
            }
            Builtin::SliceReverse => {
                let en = self.cty(tys[0]);
                let d = self.simple(&args[0]);
                self.line(format!(
                    "for (size_t i = 0; i + 1 < {}.len; i++, {}.len > 0) {{ if (i >= {}.len - 1 - i) break; {} _x = {}.ptr[i]; {}.ptr[i] = {}.ptr[{}.len - 1 - i]; {}.ptr[{}.len - 1 - i] = _x; }}",
                    d, d, d, en, d, d, d, d, d, d
                ));
                "0".into()
            }
            Builtin::SliceSort => {
                let en = self.cty(tys[0]);
                let d = self.simple(&args[0]);
                let cmpf = self.qsort_cmp(tys[0]);
                self.line(format!("if ({}.len > 1) qsort({}.ptr, {}.len, sizeof({}), {});", d, d, d, en, cmpf));
                "0".into()
            }
            Builtin::SliceContains | Builtin::SliceIndexOf => {
                let a = self.simple(&args[0]);
                let v = self.simple(&args[1]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                let eq = self.eq_expr(tys[0], &format!("{}.ptr[_i]", a), &v);
                if op == Builtin::SliceContains {
                    self.line(format!("bool {} = false; for (size_t _i = 0; _i < {}.len; _i++) if ({}) {{ {} = true; break; }}", t, a, eq, t));
                } else {
                    self.line(format!("{} {}; {}.has = false; for (size_t _i = 0; _i < {}.len; _i++) if ({}) {{ {}.has = true; {}.val = _i; break; }}", cn, t, t, a, eq, t, t));
                }
                t
            }
            Builtin::SliceStartsWith => {
                let a = self.simple(&args[0]);
                let b = self.simple(&args[1]);
                format!("nx_sl_starts_with({}, {})", a, b)
            }
            Builtin::SliceEndsWith => {
                let a = self.simple(&args[0]);
                let b = self.simple(&args[1]);
                format!("nx_sl_ends_with({}, {})", a, b)
            }
            Builtin::SliceEqIgnoreCase => {
                let a = self.simple(&args[0]);
                let b = self.simple(&args[1]);
                format!("nx_sl_eq_ignore_case({}, {})", a, b)
            }
            Builtin::SliceTrim => {
                let a = self.simple(&args[0]);
                format!("nx_sl_trim({})", a)
            }
            Builtin::SliceSplit | Builtin::SliceLines => {
                let a = self.simple(&args[0]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                self.line(format!("{} {} = {{0}}; {}.ar = c->arena;", cn, t, t));
                let sep = if op == Builtin::SliceSplit { self.simple(&args[1]) } else { "nx_lit(\"\\n\", 1)".to_string() };
                self.line(format!("{{ size_t _s = 0; for (;;) {{ nx_sl_u8 _rest = {{ {}.ptr + _s, {}.len - _s }}; size_t _i; bool _f = {}.len && nx_sl_find(_rest, {}, &_i); nx_sl_u8 _piece = {{ _rest.ptr, _f ? _i : _rest.len }};", a, a, sep, sep));
                if op == Builtin::SliceLines {
                    self.line("  if (_piece.len && _piece.ptr[_piece.len - 1] == '\\r') _piece.len--;");
                    self.line(format!("  if (!_f && _piece.len == 0 && {}.len) break;", a));
                }
                self.line(format!("  if ({}.len == {}.cap) nx_list_grow(c, (nx_rawlist*)&{}, sizeof(nx_sl_u8), _Alignof(nx_sl_u8), {}.len + 1);", t, t, t, t));
                self.line(format!("  {}.ptr[{}.len++] = _piece; if (!_f) break; _s += _i + {}.len; }} }}", t, t, sep));
                t
            }
            Builtin::SliceParseInt => {
                let a = self.simple(&args[0]);
                let cn = self.cty(e.ty);
                let target = self.cty(tys[0]);
                let (lo, hi) = self.int_bounds_pub(tys[0]);
                let t = self.tmp();
                let inv = self.err_id("InvalidInput");
                let ovf = self.err_id("Overflow");
                self.line(format!(
                    "{} {}; {{ nx_i128 _v = 0; int _r = nx_parse_int({}, {}, {}, &_v); {}.err = _r == 0 ? 0 : (_r == 1 ? {}u : {}u); if (_r == 0) {}.val = ({})_v; }}",
                    cn, t, a, lo, hi, t, inv, ovf, t, target
                ));
                t
            }
            Builtin::SliceParseFloat => {
                let a = self.simple(&args[0]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                let inv = self.err_id("InvalidInput");
                self.line(format!("{} {}; {{ double _v = 0; bool _ok = nx_parse_float({}, &_v); {}.err = _ok ? 0 : {}u; {}.val = _v; }}", cn, t, a, t, inv, t));
                t
            }
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
                let x = self.simple(&args[0]);
                let is_f32 = matches!(self.kind_of(tys[0]), TyKind::Float(FloatTy::F32));
                let f = match op {
                    Builtin::MathSqrt => "sqrt",
                    Builtin::MathFloor => "floor",
                    Builtin::MathCeil => "ceil",
                    Builtin::MathRound => "round",
                    Builtin::MathSin => "sin",
                    Builtin::MathCos => "cos",
                    Builtin::MathTan => "tan",
                    Builtin::MathExp => "exp",
                    Builtin::MathLog => "log",
                    _ => "log2",
                };
                if is_f32 {
                    format!("{}f({})", f, x)
                } else {
                    format!("{}({})", f, x)
                }
            }
            Builtin::MathAbs => {
                let x = self.simple(&args[0]);
                let t = self.res(tys[0]);
                match self.p.tys.kind(t).clone() {
                    TyKind::Float(FloatTy::F32) => format!("fabsf({})", x),
                    TyKind::Float(_) => format!("fabs({})", x),
                    _ => {
                        let m = self.int_mangle(t);
                        format!("nx_abs_{}({}, {})", m, x, loc)
                    }
                }
            }
            Builtin::MathMin | Builtin::MathMax => {
                let a = self.simple(&args[0]);
                let b = self.simple(&args[1]);
                if op == Builtin::MathMin {
                    format!("(({}) < ({}) ? ({}) : ({}))", a, b, a, b)
                } else {
                    format!("(({}) > ({}) ? ({}) : ({}))", a, b, a, b)
                }
            }
            Builtin::MathClamp => {
                let a = self.simple(&args[0]);
                let lo = self.simple(&args[1]);
                let hi = self.simple(&args[2]);
                format!("(({}) < ({}) ? ({}) : (({}) > ({}) ? ({}) : ({})))", a, lo, lo, a, hi, hi, a)
            }
            Builtin::MathPow => {
                let a = self.simple(&args[0]);
                let b = self.simple(&args[1]);
                format!("pow({}, {})", a, b)
            }
            Builtin::MathAtan2 => {
                let a = self.simple(&args[0]);
                let b = self.simple(&args[1]);
                format!("atan2({}, {})", a, b)
            }
            Builtin::Hash => {
                // checked arithmetic returning an optional
                let a = self.simple(&args[0]);
                let b = self.simple(&args[1]);
                let marker = match args[2].kind {
                    TExprKind::Int(m) => m,
                    _ => 0,
                };
                let f = match marker {
                    0 => "add",
                    1 => "sub",
                    _ => "mul",
                };
                let cn = self.cty(e.ty);
                let en = self.cty(tys[0]);
                let t = self.tmp();
                self.line(format!("{} {}; {{ {} _r; {}.has = !__builtin_{}_overflow({}, {}, &_r); {}.val = _r; }}", cn, t, en, t, f, a, b, t));
                t
            }
            Builtin::Truncate => {
                let v = self.simple(&args[0]);
                let cn = self.cty(e.ty);
                let ut = match self.kind_of(e.ty) {
                    TyKind::Int(it) => match it.bits() {
                        8 => "uint8_t",
                        16 => "uint16_t",
                        32 => "uint32_t",
                        64 => "uint64_t",
                        _ => "nx_u128",
                    },
                    _ => "uint64_t",
                };
                format!("(({})({})({}))", cn, ut, v)
            }
            Builtin::ErrorName => {
                let v = self.simple(&args[0]);
                format!("nx_lit(nx_error_name({}), strlen(nx_error_name({})))", v, v)
            }
            Builtin::Weak => {
                let v = self.simple(&args[0]);
                let cn = self.cty(e.ty);
                format!("(({})nx_weak_new({}))", cn, v)
            }
            Builtin::Upgrade => {
                let v = self.simple(&args[0]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                self.line(format!("{} {}; {}.has = nx_weak_upgrade({}); {}.val = {};", cn, t, t, v, t, v));
                t
            }
            Builtin::RefCount => {
                let v = self.simple(&args[0]);
                format!("(({})->rc)", v)
            }
            Builtin::RefEq => {
                let a = self.simple(&args[0]);
                let b = self.simple(&args[1]);
                format!("(({}) == ({}))", a, b)
            }
            Builtin::CharIsDigit => {
                let v = self.simple(&args[0]);
                format!("(({}) >= '0' && ({}) <= '9')", v, v)
            }
            Builtin::CharIsAlpha => {
                let v = self.simple(&args[0]);
                format!("((({}) >= 'a' && ({}) <= 'z') || (({}) >= 'A' && ({}) <= 'Z'))", v, v, v, v)
            }
            Builtin::CharIsSpace => {
                let v = self.simple(&args[0]);
                format!("(({}) == ' ' || ({}) == '\\t' || ({}) == '\\n' || ({}) == '\\r')", v, v, v, v)
            }
            Builtin::CharToLower => {
                let v = self.simple(&args[0]);
                format!("((({}) >= 'A' && ({}) <= 'Z') ? ({}) + 32 : ({}))", v, v, v, v)
            }
            Builtin::CharToUpper => {
                let v = self.simple(&args[0]);
                format!("((({}) >= 'a' && ({}) <= 'z') ? ({}) - 32 : ({}))", v, v, v, v)
            }
            Builtin::CharToDigit => {
                let v = self.simple(&args[0]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                self.line(format!("{} {}; {}.has = ({}) >= '0' && ({}) <= '9'; {}.val = ({}) - '0';", cn, t, t, v, v, t, v));
                t
            }
            Builtin::ReadFile => {
                let p = self.simple(&args[0]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                let io = self.err_id("IoError");
                self.line(format!("{} {}; {{ nx_string _s; if (nx_read_file(c, {}, &_s)) {{ {}.err = 0; {}.val = _s; }} else {}.err = {}u; }}", cn, t, p, t, t, t, io));
                t
            }
            Builtin::WriteFile | Builtin::AppendFile => {
                let p = self.simple(&args[0]);
                let d = self.simple(&args[1]);
                let cn = self.cty(e.ty);
                let io = self.err_id("IoError");
                let f = if op == Builtin::WriteFile { "nx_write_file" } else { "nx_append_file" };
                format!("(({}){{ .err = {}({}, {}) ? 0 : {}u }})", cn, f, p, d, io)
            }
            Builtin::FsKind => {
                let p = self.simple(&args[0]);
                format!("nx_fs_kind({})", p)
            }
            Builtin::FsSize | Builtin::FsModified => {
                let p = self.simple(&args[0]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                let (nf, io) = (self.err_id("NotFound"), self.err_id("IoError"));
                let pick = if op == Builtin::FsSize { "(uint64_t)_sz" } else { "_mt" };
                self.line(format!(
                    "{} {}; {{ int64_t _sz = 0, _mt = 0; int32_t _r = nx_fs_stat({}, &_sz, &_mt); if (_r == 0) {{ {}.err = 0; {}.val = {}; }} else {}.err = _r == 1 ? {}u : {}u; }}",
                    cn, t, p, t, t, pick, t, nf, io
                ));
                t
            }
            Builtin::FsMkdir | Builtin::FsRemoveFile | Builtin::FsRemoveDir => {
                let p = self.simple(&args[0]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                let (nf, io) = (self.err_id("NotFound"), self.err_id("IoError"));
                let f = match op {
                    Builtin::FsMkdir => "nx_fs_mkdir",
                    Builtin::FsRemoveFile => "nx_fs_remove_file",
                    _ => "nx_fs_remove_dir",
                };
                self.line(format!("{} {}; {{ int32_t _r = {}({}); {}.err = _r == 0 ? 0 : _r == 1 ? {}u : {}u; }}", cn, t, f, p, t, nf, io));
                t
            }
            Builtin::FsRename => {
                let a = self.simple(&args[0]);
                let b = self.simple(&args[1]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                let (nf, io) = (self.err_id("NotFound"), self.err_id("IoError"));
                self.line(format!("{} {}; {{ int32_t _r = nx_fs_rename({}, {}); {}.err = _r == 0 ? 0 : _r == 1 ? {}u : {}u; }}", cn, t, a, b, t, nf, io));
                t
            }
            Builtin::FsListDir => {
                let p = self.simple(&args[0]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                let (nf, io) = (self.err_id("NotFound"), self.err_id("IoError"));
                self.line(format!(
                    "{} {}; {{ nx_rawlist _l; int32_t _r = nx_fs_list_dir(c, {}, &_l); if (_r == 0) {{ {}.err = 0; memcpy(&{}.val, &_l, sizeof _l); }} else {}.err = _r == 1 ? {}u : {}u; }}",
                    cn, t, p, t, t, t, nf, io
                ));
                t
            }
            Builtin::FsCwd => {
                let cn = self.cty(e.ty);
                let t = self.tmp();
                let io = self.err_id("IoError");
                self.line(format!("{} {}; {{ nx_string _s; if (nx_fs_cwd(c, &_s)) {{ {}.err = 0; {}.val = _s; }} else {}.err = {}u; }}", cn, t, t, t, t, io));
                t
            }
            Builtin::FsTempDir => "nx_fs_temp_dir(c)".into(),
            Builtin::FileOpen => {
                let p = self.simple(&args[0]);
                let md = self.simple(&args[1]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                let (nf, io) = (self.err_id("NotFound"), self.err_id("IoError"));
                self.line(format!("{} {}; {{ int64_t _h = nx_file_open({}, {}); if (_h >= 0) {{ {}.err = 0; {}.val = _h; }} else {}.err = _h == -1 ? {}u : {}u; }}", cn, t, p, md, t, t, t, nf, io));
                t
            }
            Builtin::FileRead => {
                let h = self.simple(&args[0]);
                let n = self.simple(&args[1]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                let io = self.err_id("IoError");
                self.line(format!("{} {}; {{ nx_string _s; if (nx_file_read(c, {}, {}, &_s)) {{ {}.err = 0; {}.val = _s; }} else {}.err = {}u; }}", cn, t, h, n, t, t, t, io));
                t
            }
            Builtin::FileWrite => {
                let h = self.simple(&args[0]);
                let d = self.simple(&args[1]);
                let cn = self.cty(e.ty);
                let io = self.err_id("IoError");
                format!("(({}){{ .err = nx_file_write({}, {}) ? 0 : {}u }})", cn, h, d, io)
            }
            Builtin::FileFlush | Builtin::FileClose => {
                let h = self.simple(&args[0]);
                let cn = self.cty(e.ty);
                let io = self.err_id("IoError");
                let f = if op == Builtin::FileFlush { "nx_file_flush" } else { "nx_file_close" };
                format!("(({}){{ .err = {}({}) ? 0 : {}u }})", cn, f, h, io)
            }
            Builtin::Environ => {
                let cn = self.cty(e.ty);
                let t = self.tmp();
                self.line(format!("{} {}; {{ nx_rawlist _l; nx_environ(c, &_l); memcpy(&{}, &_l, sizeof _l); }}", cn, t, t));
                t
            }
            Builtin::ReadLine => {
                let cn = self.cty(e.ty);
                let t = self.tmp();
                self.line(format!("{} {}; {{ nx_string _s; {}.has = nx_read_line(c, &_s); if ({}.has) {}.val = _s; }}", cn, t, t, t, t));
                t
            }
            Builtin::Args => {
                let cn = self.cty(e.ty);
                let t = self.tmp();
                self.line(format!("{} {}; {{ size_t _n; nx_sl_u8* _a = nx_args(c, &_n); {}.ptr = _a; {}.len = _n; }}", cn, t, t, t));
                t
            }
            Builtin::Env => {
                let n = self.simple(&args[0]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                self.line(format!("{} {}; {{ char _nb[256]; size_t _l = {}.len < 255 ? {}.len : 255; memcpy(_nb, {}.ptr, _l); _nb[_l] = 0; const char* _v = getenv(_nb); {}.has = _v != NULL; if (_v) {{ {}.val.ptr = (uint8_t*)_v; {}.val.len = strlen(_v); }} }}", cn, t, n, n, n, t, t, t));
                t
            }
            Builtin::Exit => {
                let v = self.simple(&args[0]);
                self.line(format!("exit((int)({}));", v));
                "0".into()
            }
            Builtin::Run => {
                let argv = self.simple(&args[0]);
                let cn = self.cty(e.ty);
                let t = self.tmp();
                let io = self.err_id("IoError");
                self.line(format!("{} {}; {{ int _code = 0; if (nx_run(c, {}.ptr, {}.len, &_code)) {{ {}.err = 0; {}.val = _code; }} else {}.err = {}u; }}", cn, t, argv, argv, t, t, t, io));
                t
            }
            Builtin::TimeNow => "nx_time_now_ms()".into(),
            Builtin::TimeUtcOffset => {
                let v = self.simple(&args[0]);
                format!("nx_time_utc_offset_min({})", v)
            }
            Builtin::TimeMonotonic => "nx_time_monotonic_ns()".into(),
            Builtin::Sleep => {
                let v = self.simple(&args[0]);
                self.line(format!("nx_sleep_ms({});", v));
                "0".into()
            }
            Builtin::RandomInt => {
                let a = self.simple(&args[0]);
                let b = self.simple(&args[1]);
                format!("nx_random_int(c, {}, {}, {})", a, b, loc)
            }
            Builtin::RandomFloat => "nx_random_float(c)".into(),
            Builtin::RandomSeed => {
                let v = self.simple(&args[0]);
                self.line(format!("c->rng = {}; c->rng_seeded = true;", v));
                "0".into()
            }
            _ => {
                self.errors.push(format!("builtin {:?} is not supported by the C backend", op));
                "0".into()
            }
        }
    }

    pub fn int_bounds_pub(&mut self, t: TyId) -> (String, String) {
        let t = self.res(t);
        match self.p.tys.kind(t).clone() {
            TyKind::Int(it) => (int_literal(it.min(), IntTy::I128), int_literal(it.max(), IntTy::I128)),
            TyKind::Distinct(d) => {
                let u = self.p.distinct_underlying[&d];
                self.int_bounds_pub(u)
            }
            _ => ("0".into(), "0".into()),
        }
    }

    fn qsort_cmp(&mut self, t: TyId) -> String {
        let t = self.res(t);
        let key = format!("qsort:{}", t);
        if let Some(n) = self.thunks_by_key.get(&key) {
            return n.clone();
        }
        let m = self.mangle(t);
        let name = format!("nx_qcmp_{}", m);
        self.thunks_by_key.insert(key, name.clone());
        let cn = self.cty(t);
        let cmp = self.cmp_expr(t, &format!("(*(const {}*)a)", cn), &format!("(*(const {}*)b)", cn));
        let _ = writeln!(self.protos_out, "static int {}(const void* a, const void* b);", name);
        let _ = writeln!(self.helpers_out, "static int {}(const void* a, const void* b) {{ return {}; }}", name, cmp);
        name
    }
}
