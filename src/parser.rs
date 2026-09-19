//! Recursive-descent parser for Nexium.

use crate::ast::*;
use crate::diag::{Diag, Span};
use crate::lexer::{Tok, Token};

pub const EFFECT_NAMES: &[&str] = &["allocates", "refcounts", "blocks", "shared_mutable", "nondeterministic", "panics", "ffi", "unbounded_stack"];

pub const PRIMITIVE_TYPES: &[&str] = &["i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "isize", "usize", "f32", "f64", "bool", "char", "void", "never", "type", "anytype"];

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
    file: u32,
    pub diags: Vec<Diag>,
    no_struct_lit: bool,
}

type PResult<T> = Result<T, ()>;

impl Parser {
    pub fn new(toks: Vec<Token>, file: u32) -> Self {
        Parser { toks, pos: 0, file, diags: Vec::new(), no_struct_lit: false }
    }

    // ----- token helpers -------------------------------------------------

    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }
    fn peek_at(&self, n: usize) -> &Tok {
        let i = (self.pos + n).min(self.toks.len() - 1);
        &self.toks[i].tok
    }
    fn span(&self) -> Span {
        self.toks[self.pos].span
    }
    fn prev_span(&self) -> Span {
        self.toks[self.pos.saturating_sub(1)].span
    }
    fn bump(&mut self) -> Token {
        let t = self.toks[self.pos].clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }
    fn at(&self, t: &Tok) -> bool {
        self.peek() == t
    }
    fn at_ident(&self, s: &str) -> bool {
        matches!(self.peek(), Tok::Ident(x) if x == s)
    }
    fn eat(&mut self, t: &Tok) -> bool {
        if self.at(t) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn eat_ident(&mut self, s: &str) -> bool {
        if self.at_ident(s) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn skip_newlines(&mut self) {
        while self.at(&Tok::Newline) || self.at(&Tok::Semi) {
            self.bump();
        }
    }
    fn error<T>(&mut self, span: Span, msg: impl Into<String>) -> PResult<T> {
        self.diags.push(Diag::error(span, msg));
        Err(())
    }
    fn expect(&mut self, t: &Tok) -> PResult<Token> {
        if self.at(t) {
            Ok(self.bump())
        } else {
            let got = self.peek().describe();
            let span = self.span();
            self.error(span, format!("expected {} but found {}", t.describe(), got))
        }
    }
    fn expect_ident(&mut self) -> PResult<(String, Span)> {
        match self.peek().clone() {
            Tok::Ident(s) if !crate::lexer::is_keyword(&s) => {
                let sp = self.span();
                self.bump();
                Ok((s, sp))
            }
            other => {
                let span = self.span();
                self.error(span, format!("expected an identifier but found {}", other.describe()))
            }
        }
    }
    fn expect_kw(&mut self, kw: &str) -> PResult<Span> {
        if self.at_ident(kw) {
            Ok(self.bump().span)
        } else {
            let got = self.peek().describe();
            let span = self.span();
            self.error(span, format!("expected `{}` but found {}", kw, got))
        }
    }
    /// Recover to the next top-level item boundary.
    fn recover_item(&mut self) {
        loop {
            match self.peek() {
                Tok::Eof => return,
                Tok::Newline => {
                    self.bump();
                    if self.at_item_start() {
                        return;
                    }
                }
                _ => {
                    self.bump();
                }
            }
        }
    }
    fn at_item_start(&self) -> bool {
        matches!(self.peek(), Tok::Ident(s) if matches!(s.as_str(),
            "fn" | "pub" | "struct" | "enum" | "record" | "ref" | "trait" | "impl" | "const" | "var" | "type"
            | "import" | "test" | "artifact" | "error" | "extern" | "comptime"))
            || matches!(self.peek(), Tok::Doc(_))
    }
    fn recover_stmt(&mut self) {
        let mut depth = 0i32;
        loop {
            match self.peek() {
                Tok::Eof => return,
                Tok::Newline if depth <= 0 => return,
                Tok::LBrace | Tok::DotLBrace => {
                    depth += 1;
                    self.bump();
                }
                Tok::RBrace => {
                    if depth <= 0 {
                        return;
                    }
                    depth -= 1;
                    self.bump();
                }
                _ => {
                    self.bump();
                }
            }
        }
    }

    // ----- module ---------------------------------------------------------

    pub fn parse_module(&mut self) -> Module {
        let mut items = Vec::new();
        loop {
            self.skip_newlines();
            if self.at(&Tok::Eof) {
                break;
            }
            match self.parse_item() {
                Ok(Some(item)) => items.push(item),
                Ok(None) => {}
                Err(()) => self.recover_item(),
            }
        }
        Module { items, file: self.file }
    }

    fn parse_docs(&mut self) -> Vec<String> {
        let mut docs = Vec::new();
        loop {
            match self.peek().clone() {
                Tok::Doc(s) => {
                    docs.push(s);
                    self.bump();
                }
                Tok::Newline if !docs.is_empty() => {
                    self.bump();
                }
                _ => break,
            }
        }
        docs
    }

    fn parse_item(&mut self) -> PResult<Option<Item>> {
        let doc = self.parse_docs();
        let start = self.span();
        let is_pub = self.eat_ident("pub");
        let attrs = Attrs { is_pub, doc };
        let item = match self.peek().clone() {
            Tok::Ident(s) => match s.as_str() {
                "fn" => Item::Fn(self.parse_fn(attrs, false)?),
                "extern" => {
                    self.bump();
                    Item::Fn(self.parse_fn(attrs, true)?)
                }
                "struct" => Item::Struct(self.parse_struct(attrs, StructKind::Struct)?),
                "record" => Item::Struct(self.parse_struct(attrs, StructKind::Record)?),
                "ref" => {
                    self.bump();
                    self.expect_kw("class")?;
                    // parse_struct expects to consume the keyword; we already did. Use a helper.
                    Item::Struct(self.parse_struct_body(attrs, StructKind::RefClass, start)?)
                }
                "enum" => Item::Enum(self.parse_enum(attrs)?),
                "trait" => Item::Trait(self.parse_trait(attrs)?),
                "impl" => Item::Impl(self.parse_impl()?),
                "const" => Item::Const(self.parse_const(attrs)?),
                "var" => Item::Global(self.parse_global(attrs)?),
                "type" => Item::TypeAlias(self.parse_type_alias(attrs)?),
                "error" => Item::ErrorSet(self.parse_error_set(attrs)?),
                "import" => Item::Import(self.parse_import()?),
                "test" => Item::Test(self.parse_test(false)?),
                "comptime" if matches!(self.peek_at(1), Tok::Ident(t) if t == "test") => {
                    self.bump();
                    Item::Test(self.parse_test(true)?)
                }
                "artifact" => Item::Artifact(self.parse_artifact()?),
                _ => {
                    let sp = self.span();
                    return self.error(sp, format!("expected a declaration but found `{}`", s));
                }
            },
            other => {
                let sp = self.span();
                return self.error(sp, format!("expected a declaration but found {}", other.describe()));
            }
        };
        // items end at a newline or EOF
        if !(self.at(&Tok::Newline) || self.at(&Tok::Eof) || self.at(&Tok::RBrace)) {
            let sp = self.span();
            let got = self.peek().describe();
            return self.error(sp, format!("expected a newline after declaration but found {}", got));
        }
        Ok(Some(item))
    }

    fn parse_type_params(&mut self) -> PResult<Vec<String>> {
        // `(T, U)` immediately after a type name in a declaration
        let mut out = Vec::new();
        if self.at(&Tok::LParen) {
            self.bump();
            loop {
                if self.at(&Tok::RParen) {
                    break;
                }
                let (n, _) = self.expect_ident()?;
                out.push(n);
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(&Tok::RParen)?;
        }
        Ok(out)
    }

    fn parse_effect_bounds(&mut self) -> Vec<EffectBound> {
        let mut out = Vec::new();
        loop {
            if self.at(&Tok::Bang) {
                if let Tok::Ident(n) = self.peek_at(1).clone() {
                    if EFFECT_NAMES.contains(&n.as_str()) {
                        let sp = self.span();
                        self.bump();
                        let sp2 = self.bump().span;
                        out.push(EffectBound { negative: true, name: n, span: sp.to(sp2) });
                        continue;
                    }
                }
                break;
            }
            if let Tok::Ident(n) = self.peek().clone() {
                if EFFECT_NAMES.contains(&n.as_str()) {
                    let sp = self.bump().span;
                    out.push(EffectBound { negative: false, name: n, span: sp });
                    continue;
                }
            }
            break;
        }
        out
    }

    fn parse_where_bounds(&mut self, param: String, span: Span) -> PResult<WhereClause> {
        // after `where`: `T: Ord + Eq + !blocks`
        let (p, _) = self.expect_ident()?;
        if p != param {
            let sp = self.prev_span();
            return self.error(sp, format!("where clause names `{}` but the parameter is `{}`", p, param));
        }
        self.expect(&Tok::Colon)?;
        let mut bounds = Vec::new();
        let mut effect_bounds = Vec::new();
        loop {
            if self.at(&Tok::Bang) {
                let sp = self.span();
                self.bump();
                let (n, sp2) = self.expect_ident()?;
                effect_bounds.push(EffectBound { negative: true, name: n, span: sp.to(sp2) });
            } else {
                let (n, _) = self.expect_ident()?;
                bounds.push(n);
            }
            if !self.eat(&Tok::Plus) {
                break;
            }
        }
        Ok(WhereClause { param, bounds, effect_bounds, span })
    }

    fn parse_params(&mut self, wheres: &mut Vec<WhereClause>) -> PResult<Vec<Param>> {
        self.expect(&Tok::LParen)?;
        let mut params = Vec::new();
        loop {
            if self.at(&Tok::RParen) {
                break;
            }
            let start = self.span();
            let comptime = self.eat_ident("comptime");
            // `own` is a modifier only when a parameter name follows it
            let owned = self.at_ident("own") && matches!(self.peek_at(1), Tok::Ident(_)) && self.eat_ident("own");
            let (name, nsp) = self.expect_ident()?;
            if owned && name == "self" {
                return self.error(nsp, "`self` cannot be an owned parameter; receivers are borrowed (`*Self` or `*mut Self`)");
            }
            self.expect(&Tok::Colon)?;
            let ty = self.parse_type()?;
            if self.eat_ident("where") {
                let w = self.parse_where_bounds(name.clone(), nsp)?;
                wheres.push(w);
            }
            let span = start.to(self.prev_span());
            params.push(Param { name, ty, comptime, owned, span });
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RParen)?;
        Ok(params)
    }

    fn parse_fn(&mut self, attrs: Attrs, extern_c: bool) -> PResult<FnDecl> {
        let start = self.expect_kw("fn")?;
        let (name, name_span) = self.expect_ident()?;
        let mut wheres = Vec::new();
        let params = self.parse_params(&mut wheres)?;
        let ret = if self.eat(&Tok::Arrow) { Some(self.parse_type()?) } else { None };
        let effects = self.parse_effect_bounds();
        while self.eat_ident("where") {
            let sp = self.prev_span();
            let pname = match self.peek() {
                Tok::Ident(s) => s.clone(),
                _ => String::new(),
            };
            let w = self.parse_where_bounds(pname, sp)?;
            wheres.push(w);
        }
        let mut export = None;
        if self.eat_ident("export") {
            self.expect(&Tok::LParen)?;
            let (abi, _) = self.expect_ident()?;
            self.expect(&Tok::RParen)?;
            export = Some(abi);
        }
        let body = if self.at(&Tok::LBrace) { Some(self.parse_block(None)?) } else { None };
        let span = start.to(self.prev_span());
        Ok(FnDecl { attrs, name, params, ret, effects, wheres, export, extern_c, variadic: false, cimport_header: None, body, span, name_span })
    }

    fn parse_struct(&mut self, attrs: Attrs, kind: StructKind) -> PResult<StructDecl> {
        let start = self.bump().span; // struct / record
        self.parse_struct_body(attrs, kind, start)
    }

    fn parse_struct_body(&mut self, attrs: Attrs, kind: StructKind, start: Span) -> PResult<StructDecl> {
        let (name, _) = self.expect_ident()?;
        let type_params = self.parse_type_params()?;
        let mut layout = Layout::Default;
        let mut derives = Vec::new();
        loop {
            if self.eat_ident("layout") {
                self.expect(&Tok::LParen)?;
                let (l, lsp) = self.expect_ident()?;
                layout = match l.as_str() {
                    "c" => Layout::C,
                    "packed" => Layout::Packed,
                    _ => return self.error(lsp, format!("unknown layout `{}`; expected `c` or `packed`", l)),
                };
                self.expect(&Tok::RParen)?;
            } else if self.eat_ident("derive") {
                self.expect(&Tok::LParen)?;
                loop {
                    if self.at(&Tok::RParen) {
                        break;
                    }
                    let (d, _) = self.expect_ident()?;
                    derives.push(d);
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.expect(&Tok::RParen)?;
            } else if self.eat_ident("soa") {
                // accepted and ignored in this implementation
            } else {
                break;
            }
        }
        let fields = self.parse_fields()?;
        let span = start.to(self.prev_span());
        Ok(StructDecl { attrs, kind, name, type_params, layout, derives, fields, span })
    }

    fn parse_fields(&mut self) -> PResult<Vec<Field>> {
        self.expect(&Tok::LBrace)?;
        let mut fields = Vec::new();
        loop {
            self.skip_newlines();
            let doc = self.parse_docs();
            self.skip_newlines();
            if self.at(&Tok::RBrace) {
                break;
            }
            let start = self.span();
            let (name, _) = self.expect_ident()?;
            self.expect(&Tok::Colon)?;
            let ty = self.parse_type()?;
            let default = if self.eat(&Tok::Eq) { Some(self.parse_expr()?) } else { None };
            let constraint = if self.eat_ident("where") { Some(self.parse_expr()?) } else { None };
            let span = start.to(self.prev_span());
            fields.push(Field { name, ty, default, constraint, doc, span });
            if !(self.eat(&Tok::Comma) || self.at(&Tok::Newline) || self.at(&Tok::RBrace)) {
                let sp = self.span();
                let got = self.peek().describe();
                return self.error(sp, format!("expected `,` or newline after field but found {}", got));
            }
        }
        self.expect(&Tok::RBrace)?;
        Ok(fields)
    }

    fn parse_enum(&mut self, attrs: Attrs) -> PResult<EnumDecl> {
        let start = self.expect_kw("enum")?;
        let (name, _) = self.expect_ident()?;
        let type_params = self.parse_type_params()?;
        let mut derives = Vec::new();
        if self.eat_ident("derive") {
            self.expect(&Tok::LParen)?;
            loop {
                if self.at(&Tok::RParen) {
                    break;
                }
                let (d, _) = self.expect_ident()?;
                derives.push(d);
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(&Tok::RParen)?;
        }
        self.expect(&Tok::LBrace)?;
        let mut variants = Vec::new();
        loop {
            self.skip_newlines();
            let _doc = self.parse_docs();
            self.skip_newlines();
            if self.at(&Tok::RBrace) {
                break;
            }
            let vstart = self.span();
            let (vname, _) = self.expect_ident()?;
            let payload = if self.at(&Tok::LParen) {
                self.bump();
                let mut tys = Vec::new();
                loop {
                    if self.at(&Tok::RParen) {
                        break;
                    }
                    tys.push(self.parse_type()?);
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.expect(&Tok::RParen)?;
                VariantPayload::Tuple(tys)
            } else if self.at(&Tok::LBrace) {
                VariantPayload::Struct(self.parse_fields()?)
            } else {
                VariantPayload::Unit
            };
            let span = vstart.to(self.prev_span());
            variants.push(Variant { name: vname, payload, span });
            if !(self.eat(&Tok::Comma) || self.at(&Tok::Newline) || self.at(&Tok::RBrace)) {
                let sp = self.span();
                let got = self.peek().describe();
                return self.error(sp, format!("expected `,` or newline after enum variant but found {}", got));
            }
        }
        self.expect(&Tok::RBrace)?;
        let span = start.to(self.prev_span());
        Ok(EnumDecl { attrs, name, type_params, variants, derives, span })
    }

    fn parse_trait(&mut self, attrs: Attrs) -> PResult<TraitDecl> {
        let start = self.expect_kw("trait")?;
        let (name, _) = self.expect_ident()?;
        self.expect(&Tok::LBrace)?;
        let mut assoc_types = Vec::new();
        let mut methods = Vec::new();
        loop {
            self.skip_newlines();
            let doc = self.parse_docs();
            self.skip_newlines();
            if self.at(&Tok::RBrace) {
                break;
            }
            if self.eat_ident("type") {
                let (n, _) = self.expect_ident()?;
                assoc_types.push(n);
            } else {
                let f = self.parse_fn(Attrs { is_pub: true, doc }, false)?;
                methods.push(f);
            }
            if !(self.at(&Tok::Newline) || self.at(&Tok::RBrace)) {
                let sp = self.span();
                return self.error(sp, "expected a newline after trait member");
            }
        }
        self.expect(&Tok::RBrace)?;
        let span = start.to(self.prev_span());
        Ok(TraitDecl { attrs, name, assoc_types, methods, span })
    }

    fn parse_impl(&mut self) -> PResult<ImplDecl> {
        let start = self.expect_kw("impl")?;
        let type_params = self.parse_type_params()?;
        // `impl Trait for Type` or `impl Type`
        let first = self.parse_type()?;
        let (trait_name, target) = if self.eat_ident("for") {
            let tn = match &first {
                TypeExpr::Named { path, args, .. } if path.len() == 1 && args.is_empty() => path[0].clone(),
                _ => return self.error(first.span(), "expected a trait name before `for`"),
            };
            (Some(tn), self.parse_type()?)
        } else {
            (None, first)
        };
        self.expect(&Tok::LBrace)?;
        let mut assoc_types = Vec::new();
        let mut methods = Vec::new();
        loop {
            self.skip_newlines();
            let doc = self.parse_docs();
            self.skip_newlines();
            if self.at(&Tok::RBrace) {
                break;
            }
            let is_pub = self.eat_ident("pub");
            if self.eat_ident("type") {
                let (n, _) = self.expect_ident()?;
                self.expect(&Tok::Eq)?;
                let t = self.parse_type()?;
                assoc_types.push((n, t));
            } else {
                let f = self.parse_fn(Attrs { is_pub, doc }, false)?;
                methods.push(f);
            }
            if !(self.at(&Tok::Newline) || self.at(&Tok::RBrace)) {
                let sp = self.span();
                return self.error(sp, "expected a newline after impl member");
            }
        }
        self.expect(&Tok::RBrace)?;
        let span = start.to(self.prev_span());
        Ok(ImplDecl { trait_name, target, type_params, assoc_types, methods, span })
    }

    fn parse_const(&mut self, attrs: Attrs) -> PResult<ConstDecl> {
        let start = self.expect_kw("const")?;
        let (name, _) = self.expect_ident()?;
        let ty = if self.eat(&Tok::Colon) { Some(self.parse_type()?) } else { None };
        self.expect(&Tok::Eq)?;
        let value = self.parse_expr()?;
        let span = start.to(self.prev_span());
        Ok(ConstDecl { attrs, name, ty, value, span })
    }

    fn parse_global(&mut self, attrs: Attrs) -> PResult<GlobalDecl> {
        let start = self.expect_kw("var")?;
        let (name, _) = self.expect_ident()?;
        self.expect(&Tok::Colon)?;
        let ty = self.parse_type()?;
        let mut align = None;
        if self.eat_ident("align") {
            self.expect(&Tok::LParen)?;
            match self.bump().tok {
                Tok::Int(v) => align = Some(v as u64),
                _ => {
                    let sp = self.prev_span();
                    return self.error(sp, "expected an integer alignment");
                }
            }
            self.expect(&Tok::RParen)?;
        }
        self.expect(&Tok::Eq)?;
        let value = if self.eat_ident("undefined") { None } else { Some(self.parse_expr()?) };
        let span = start.to(self.prev_span());
        Ok(GlobalDecl { attrs, name, ty, value, align, span })
    }

    fn parse_type_alias(&mut self, attrs: Attrs) -> PResult<TypeAliasDecl> {
        let start = self.expect_kw("type")?;
        let (name, _) = self.expect_ident()?;
        self.expect(&Tok::Eq)?;
        let distinct = self.eat_ident("distinct");
        let ty = self.parse_type()?;
        let span = start.to(self.prev_span());
        Ok(TypeAliasDecl { attrs, name, distinct, ty, span })
    }

    fn parse_error_set(&mut self, attrs: Attrs) -> PResult<ErrorSetDecl> {
        let start = self.expect_kw("error")?;
        let (name, _) = self.expect_ident()?;
        self.expect(&Tok::LBrace)?;
        let mut names = Vec::new();
        loop {
            self.skip_newlines();
            if self.at(&Tok::RBrace) {
                break;
            }
            let (n, _) = self.expect_ident()?;
            names.push(n);
            if !(self.eat(&Tok::Comma) || self.at(&Tok::Newline) || self.at(&Tok::RBrace)) {
                let sp = self.span();
                return self.error(sp, "expected `,` or newline in error set");
            }
        }
        self.expect(&Tok::RBrace)?;
        let span = start.to(self.prev_span());
        Ok(ErrorSetDecl { attrs, name, names, span })
    }

    fn parse_import(&mut self) -> PResult<ImportDecl> {
        let start = self.expect_kw("import")?;
        let mut path = Vec::new();
        let mut names = None;
        let (first, _) = self.expect_ident()?;
        path.push(first);
        while self.eat(&Tok::Dot) {
            if self.at(&Tok::LBrace) {
                self.bump();
                let mut ns = Vec::new();
                loop {
                    self.skip_newlines();
                    if self.at(&Tok::RBrace) {
                        break;
                    }
                    let (n, _) = self.expect_ident()?;
                    ns.push(n);
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.skip_newlines();
                self.expect(&Tok::RBrace)?;
                names = Some(ns);
                break;
            }
            let (n, _) = self.expect_ident()?;
            path.push(n);
        }
        let alias = if self.eat_ident("as") { Some(self.expect_ident()?.0) } else { None };
        let span = start.to(self.prev_span());
        Ok(ImportDecl { path, alias, names, span })
    }

    fn parse_test(&mut self, comptime: bool) -> PResult<TestDecl> {
        let start = self.expect_kw("test")?;
        let name = match self.bump().tok {
            Tok::Str(s) => String::from_utf8_lossy(&s).to_string(),
            _ => {
                let sp = self.prev_span();
                return self.error(sp, "expected a string naming the test");
            }
        };
        let body = self.parse_block(None)?;
        let span = start.to(self.prev_span());
        Ok(TestDecl { name, comptime, body, span })
    }

    fn parse_artifact(&mut self) -> PResult<ArtifactDecl> {
        let start = self.expect_kw("artifact")?;
        let (kind, _) = self.expect_ident()?;
        self.expect(&Tok::LBrace)?;
        let mut fields = Vec::new();
        loop {
            self.skip_newlines();
            if self.at(&Tok::RBrace) {
                break;
            }
            let (k, ksp) = self.expect_ident()?;
            self.expect(&Tok::Eq)?;
            let v = self.parse_expr()?;
            fields.push((k, v, ksp));
            if !(self.eat(&Tok::Comma) || self.at(&Tok::Newline) || self.at(&Tok::RBrace)) {
                let sp = self.span();
                return self.error(sp, "expected `,` or newline in artifact declaration");
            }
        }
        self.expect(&Tok::RBrace)?;
        let span = start.to(self.prev_span());
        Ok(ArtifactDecl { kind, fields, span })
    }

    // ----- types ----------------------------------------------------------

    pub fn parse_type(&mut self) -> PResult<TypeExpr> {
        let start = self.span();
        let t = match self.peek().clone() {
            Tok::LBracket => {
                self.bump();
                if self.eat(&Tok::RBracket) {
                    let mutable = self.eat_ident("mut");
                    let elem = self.parse_type()?;
                    TypeExpr::Slice { mutable, elem: Box::new(elem), span: start.to(self.prev_span()) }
                } else {
                    let len = if self.at_ident("_") {
                        let sp = self.bump().span;
                        Expr::Ident { name: "_".into(), span: sp }
                    } else {
                        self.parse_expr()?
                    };
                    self.expect(&Tok::RBracket)?;
                    let elem = self.parse_type()?;
                    TypeExpr::Array { len: Box::new(len), elem: Box::new(elem), span: start.to(self.prev_span()) }
                }
            }
            Tok::Star => {
                self.bump();
                let mutable = self.eat_ident("mut");
                let elem = self.parse_type()?;
                TypeExpr::Ptr { mutable, elem: Box::new(elem), span: start.to(self.prev_span()) }
            }
            Tok::Question => {
                self.bump();
                let elem = self.parse_type()?;
                TypeExpr::Optional { elem: Box::new(elem), span: start.to(self.prev_span()) }
            }
            Tok::Bang => {
                self.bump();
                let elem = self.parse_type()?;
                TypeExpr::ErrorUnion { set: None, elem: Box::new(elem), span: start.to(self.prev_span()) }
            }
            Tok::LParen => {
                self.bump();
                let mut elems = Vec::new();
                loop {
                    if self.at(&Tok::RParen) {
                        break;
                    }
                    elems.push(self.parse_type()?);
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.expect(&Tok::RParen)?;
                TypeExpr::Tuple { elems, span: start.to(self.prev_span()) }
            }
            Tok::Ident(s) => match s.as_str() {
                "fn" => {
                    self.bump();
                    self.expect(&Tok::LParen)?;
                    let mut params = Vec::new();
                    loop {
                        if self.at(&Tok::RParen) {
                            break;
                        }
                        // allow `name: T` or just `T`
                        if matches!(self.peek(), Tok::Ident(_)) && self.peek_at(1) == &Tok::Colon {
                            self.bump();
                            self.bump();
                        }
                        params.push(self.parse_type()?);
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    self.expect(&Tok::RParen)?;
                    let ret = if self.eat(&Tok::Arrow) { self.parse_type()? } else { TypeExpr::named("void", self.prev_span()) };
                    let effects = self.parse_effect_bounds();
                    TypeExpr::Fn { params, ret: Box::new(ret), effects, span: start.to(self.prev_span()) }
                }
                "weak" => {
                    self.bump();
                    let elem = self.parse_type()?;
                    TypeExpr::Weak { elem: Box::new(elem), span: start.to(self.prev_span()) }
                }
                "dyn" => {
                    self.bump();
                    let (n, _) = self.expect_ident()?;
                    let effects = self.parse_effect_bounds();
                    TypeExpr::Dyn { trait_name: n, effects, span: start.to(self.prev_span()) }
                }
                "_" => {
                    self.bump();
                    TypeExpr::Infer { span: start }
                }
                _ if crate::lexer::is_keyword(&s) && s != "type" && s != "error" => {
                    return self.error(start, format!("expected a type but found keyword `{}`", s));
                }
                _ => {
                    let mut path = vec![s.clone()];
                    self.bump();
                    while self.at(&Tok::Dot) && matches!(self.peek_at(1), Tok::Ident(_)) {
                        self.bump();
                        let (n, _) = self.expect_ident()?;
                        path.push(n);
                    }
                    let mut args = Vec::new();
                    if self.at(&Tok::LParen) && !PRIMITIVE_TYPES.contains(&s.as_str()) {
                        self.bump();
                        loop {
                            if self.at(&Tok::RParen) {
                                break;
                            }
                            args.push(self.parse_type_or_const_arg()?);
                            if !self.eat(&Tok::Comma) {
                                break;
                            }
                        }
                        self.expect(&Tok::RParen)?;
                    }
                    TypeExpr::Named { path, args, span: start.to(self.prev_span()) }
                }
            },
            other => return self.error(start, format!("expected a type but found {}", other.describe())),
        };
        // `Set!T` error union with explicit set
        if self.at(&Tok::Bang) {
            let is_effect = matches!(self.peek_at(1), Tok::Ident(n) if EFFECT_NAMES.contains(&n.as_str()));
            let starts_type = matches!(self.peek_at(1), Tok::Ident(_) | Tok::LBracket | Tok::Star | Tok::Question | Tok::LParen);
            if !is_effect && starts_type {
                if let TypeExpr::Named { .. } = t {
                    self.bump();
                    let elem = self.parse_type()?;
                    return Ok(TypeExpr::ErrorUnion { set: Some(Box::new(t)), elem: Box::new(elem), span: start.to(self.prev_span()) });
                }
            }
        }
        Ok(t)
    }

    /// Generic argument: a type, or an integer constant (for `fixed(1, 15)` style).
    fn parse_type_or_const_arg(&mut self) -> PResult<TypeExpr> {
        if let Tok::Int(v) = self.peek().clone() {
            let sp = self.bump().span;
            return Ok(TypeExpr::Named { path: vec![format!("#{}", v)], args: vec![], span: sp });
        }
        self.parse_type()
    }

    // ----- statements -----------------------------------------------------

    pub fn parse_block(&mut self, label: Option<String>) -> PResult<Block> {
        let start = self.expect(&Tok::LBrace)?.span;
        let saved = self.no_struct_lit;
        self.no_struct_lit = false;
        let mut stmts: Vec<Stmt> = Vec::new();
        let mut tail: Option<Box<Expr>> = None;
        loop {
            self.skip_newlines();
            if self.at(&Tok::RBrace) {
                break;
            }
            if self.at(&Tok::Eof) {
                self.no_struct_lit = saved;
                return self.error(start, "unterminated block");
            }
            match self.parse_stmt() {
                Ok(stmt) => {
                    // statement terminator; a final expression before `}` is the block's value
                    let mut k = 0;
                    while matches!(self.peek_at(k), Tok::Newline | Tok::Semi) {
                        k += 1;
                    }
                    if self.peek_at(k) == &Tok::RBrace {
                        for _ in 0..k {
                            self.bump();
                        }
                    }
                    if self.at(&Tok::RBrace) {
                        if let Stmt::Expr(e) = stmt {
                            tail = Some(Box::new(e));
                        } else {
                            stmts.push(stmt);
                        }
                        break;
                    }
                    if !(self.at(&Tok::Newline) || self.at(&Tok::Semi)) {
                        let sp = self.span();
                        let got = self.peek().describe();
                        self.diags.push(Diag::error(sp, format!("expected a newline after statement but found {}", got)));
                        self.recover_stmt();
                    }
                    stmts.push(stmt);
                }
                Err(()) => self.recover_stmt(),
            }
        }
        let end = self.expect(&Tok::RBrace)?.span;
        self.no_struct_lit = saved;
        Ok(Block { stmts, tail, label, span: start.to(end) })
    }

    fn parse_stmt(&mut self) -> PResult<Stmt> {
        let start = self.span();
        // labels: `name: for/while/{`
        if let Tok::Ident(l) = self.peek().clone() {
            if self.peek_at(1) == &Tok::Colon && !crate::lexer::is_keyword(&l) {
                if let Tok::Ident(k) = self.peek_at(2).clone() {
                    if k == "for" || k == "while" {
                        self.bump();
                        self.bump();
                        return self.parse_loop_stmt(Some(l), start);
                    }
                }
                if self.peek_at(2) == &Tok::LBrace {
                    self.bump();
                    self.bump();
                    let b = self.parse_block(Some(l))?;
                    return Ok(Stmt::Expr(Expr::Block(b)));
                }
            }
        }
        match self.peek().clone() {
            Tok::Ident(s) => match s.as_str() {
                "let" | "var" => {
                    let mutable = s == "var";
                    self.bump();
                    let (name, _) = self.expect_ident()?;
                    let ty = if self.eat(&Tok::Colon) { Some(self.parse_type()?) } else { None };
                    let init = if self.eat(&Tok::Eq) { Some(self.parse_expr()?) } else { None };
                    Ok(Stmt::Let { mutable, name, ty, init, span: start.to(self.prev_span()) })
                }
                "const" => {
                    // local const: treated as an immutable let
                    self.bump();
                    let (name, _) = self.expect_ident()?;
                    let ty = if self.eat(&Tok::Colon) { Some(self.parse_type()?) } else { None };
                    self.expect(&Tok::Eq)?;
                    let init = Some(self.parse_expr()?);
                    Ok(Stmt::Let { mutable: false, name, ty, init, span: start.to(self.prev_span()) })
                }
                "return" => {
                    self.bump();
                    let value = if self.at(&Tok::Newline) || self.at(&Tok::RBrace) || self.at(&Tok::Semi) { None } else { Some(self.parse_expr()?) };
                    Ok(Stmt::Return { value, span: start.to(self.prev_span()) })
                }
                "break" => {
                    self.bump();
                    let label = if self.at(&Tok::Colon) {
                        self.bump();
                        Some(self.expect_ident()?.0)
                    } else {
                        None
                    };
                    let value = if self.at(&Tok::Newline) || self.at(&Tok::RBrace) || self.at(&Tok::Semi) { None } else { Some(self.parse_expr()?) };
                    Ok(Stmt::Break { label, value, span: start.to(self.prev_span()) })
                }
                "continue" => {
                    self.bump();
                    let label = if self.at(&Tok::Colon) {
                        self.bump();
                        Some(self.expect_ident()?.0)
                    } else {
                        None
                    };
                    Ok(Stmt::Continue { label, span: start.to(self.prev_span()) })
                }
                "using" => {
                    self.bump();
                    let (strategy, ssp) = self.expect_ident()?;
                    if strategy != "arena" {
                        return self.error(ssp, format!("unknown allocation strategy `{}`; available: arena", strategy));
                    }
                    let body = self.parse_block(None)?;
                    Ok(Stmt::Using { strategy, body, span: start.to(self.prev_span()) })
                }
                "defer" => {
                    self.bump();
                    let body = self.parse_stmt()?;
                    Ok(Stmt::Defer { body: Box::new(body), span: start.to(self.prev_span()) })
                }
                "errdefer" => {
                    self.bump();
                    let body = self.parse_stmt()?;
                    Ok(Stmt::ErrDefer { body: Box::new(body), span: start.to(self.prev_span()) })
                }
                "while" | "for" => self.parse_loop_stmt(None, start),
                "unsafe" if self.peek_at(1) == &Tok::LBrace => {
                    self.bump();
                    let body = self.parse_block(None)?;
                    Ok(Stmt::Unsafe { body, span: start.to(self.prev_span()) })
                }
                "_" if self.peek_at(1) == &Tok::Eq => {
                    self.bump();
                    self.bump();
                    let value = self.parse_expr()?;
                    Ok(Stmt::Discard { value, span: start.to(self.prev_span()) })
                }
                _ => self.parse_expr_or_assign(start),
            },
            _ => self.parse_expr_or_assign(start),
        }
    }

    fn parse_expr_or_assign(&mut self, start: Span) -> PResult<Stmt> {
        let e = self.parse_expr()?;
        let op = match self.peek() {
            Tok::Eq => Some(None),
            Tok::PlusEq => Some(Some(BinOp::Add)),
            Tok::MinusEq => Some(Some(BinOp::Sub)),
            Tok::StarEq => Some(Some(BinOp::Mul)),
            Tok::SlashEq => Some(Some(BinOp::Div)),
            Tok::PercentEq => Some(Some(BinOp::Rem)),
            Tok::AmpEq => Some(Some(BinOp::BitAnd)),
            Tok::PipeEq => Some(Some(BinOp::BitOr)),
            Tok::CaretEq => Some(Some(BinOp::BitXor)),
            Tok::ShlEq => Some(Some(BinOp::Shl)),
            Tok::ShrEq => Some(Some(BinOp::Shr)),
            Tok::PlusPctEq => Some(Some(BinOp::AddWrap)),
            Tok::MinusPctEq => Some(Some(BinOp::SubWrap)),
            Tok::StarPctEq => Some(Some(BinOp::MulWrap)),
            _ => None,
        };
        if let Some(op) = op {
            self.bump();
            self.skip_newlines();
            let value = self.parse_expr()?;
            return Ok(Stmt::Assign { target: e, op, value, span: start.to(self.prev_span()) });
        }
        Ok(Stmt::Expr(e))
    }

    fn parse_loop_stmt(&mut self, label: Option<String>, start: Span) -> PResult<Stmt> {
        let kw = self.bump();
        let kw_name = match kw.tok {
            Tok::Ident(s) => s,
            _ => unreachable!(),
        };
        if kw_name == "while" {
            let cond = self.parse_expr_no_struct()?;
            let body = self.parse_body("the `while` condition")?;
            let els = if self.eat_ident("else") { Some(self.parse_body("`else`")?) } else { None };
            return Ok(Stmt::While { cond, body, els, label, span: start.to(self.prev_span()) });
        }
        // for [parallel] x, i in items { }   /   for i in lo..hi [step s] { }
        let parallel = self.eat_ident("parallel");
        let mut bindings = Vec::new();
        loop {
            let (n, _) = self.expect_ident()?;
            bindings.push(n);
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        if !self.at_ident("in") {
            let sp = self.span();
            let got = self.peek().describe();
            return self.error(sp, format!("expected `in` after the loop variable(s) but found {}", got));
        }
        self.bump();
        let first = self.parse_expr_no_struct()?;
        let iter = if self.eat(&Tok::DotDot) {
            let end = self.parse_expr_no_struct()?;
            let step = if self.eat_ident("step") { Some(self.parse_expr_no_struct()?) } else { None };
            ForIter::Range { start: first, end, step }
        } else {
            let mut items = vec![first];
            while self.eat(&Tok::Comma) {
                items.push(self.parse_expr_no_struct()?);
            }
            ForIter::Items(items)
        };
        let body = self.parse_body("the `for` header")?;
        Ok(Stmt::For { iter, bindings, body, label, parallel, span: start.to(self.prev_span()) })
    }

    // ----- expressions ----------------------------------------------------

    pub fn parse_expr(&mut self) -> PResult<Expr> {
        self.parse_pipe()
    }

    fn parse_expr_no_struct(&mut self) -> PResult<Expr> {
        let saved = self.no_struct_lit;
        self.no_struct_lit = true;
        let r = self.parse_expr();
        self.no_struct_lit = saved;
        r
    }

    /// If the next token is a newline followed by a continuation operator, consume the newline.
    fn continuation(&mut self, ops: &[Tok]) -> bool {
        if self.at(&Tok::Newline) {
            let next = self.peek_at(1).clone();
            let cont = ops.contains(&next) || matches!(&next, Tok::Ident(s) if s == "catch" || s == "orelse" || s == "and" || s == "or");
            if cont {
                self.bump();
                return true;
            }
            return false;
        }
        true
    }

    fn parse_pipe(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_catch()?;
        loop {
            self.continuation(&[Tok::PipeGt]);
            if self.at(&Tok::PipeGt) {
                self.bump();
                self.skip_newlines();
                let rhs = self.parse_catch()?;
                let span = lhs.span().to(rhs.span());
                lhs = Expr::Pipe { lhs: Box::new(lhs), rhs: Box::new(rhs), span };
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_catch(&mut self) -> PResult<Expr> {
        let mut e = self.parse_or()?;
        loop {
            self.continuation(&[]);
            if self.at_ident("catch") {
                self.bump();
                let binding = if self.eat(&Tok::Pipe) {
                    let (n, _) = self.expect_ident()?;
                    self.expect(&Tok::Pipe)?;
                    Some(n)
                } else {
                    None
                };
                let handler = self.parse_or_or_jump()?;
                let span = e.span().to(handler.span());
                e = Expr::Catch { expr: Box::new(e), binding, handler: Box::new(handler), span };
            } else if self.at_ident("orelse") {
                self.bump();
                let default = self.parse_or_or_jump()?;
                let span = e.span().to(default.span());
                e = Expr::OrElse { expr: Box::new(e), default: Box::new(default), span };
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn parse_or(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_and()?;
        loop {
            self.continuation(&[]);
            if self.at_ident("or") {
                self.bump();
                self.skip_newlines();
                let rhs = self.parse_and()?;
                let span = lhs.span().to(rhs.span());
                lhs = Expr::Binary { op: BinOp::Or, lhs: Box::new(lhs), rhs: Box::new(rhs), span };
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_cmp()?;
        loop {
            self.continuation(&[]);
            if self.at_ident("and") {
                self.bump();
                self.skip_newlines();
                let rhs = self.parse_cmp()?;
                let span = lhs.span().to(rhs.span());
                lhs = Expr::Binary { op: BinOp::And, lhs: Box::new(lhs), rhs: Box::new(rhs), span };
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_cmp(&mut self) -> PResult<Expr> {
        let lhs = self.parse_bitor()?;
        let op = match self.peek() {
            Tok::EqEq => BinOp::Eq,
            Tok::Ne => BinOp::Ne,
            Tok::Lt => BinOp::Lt,
            Tok::Le => BinOp::Le,
            Tok::Gt => BinOp::Gt,
            Tok::Ge => BinOp::Ge,
            _ => return Ok(lhs),
        };
        self.bump();
        self.skip_newlines();
        let rhs = self.parse_bitor()?;
        let span = lhs.span().to(rhs.span());
        let e = Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs), span };
        if matches!(self.peek(), Tok::EqEq | Tok::Ne | Tok::Lt | Tok::Le | Tok::Gt | Tok::Ge) {
            let sp = self.span();
            return self.error(sp, "comparison operators cannot be chained; use `and`");
        }
        Ok(e)
    }

    fn binary_level(&mut self, next: fn(&mut Self) -> PResult<Expr>, ops: &[(Tok, BinOp)]) -> PResult<Expr> {
        let mut lhs = next(self)?;
        loop {
            let mut found = None;
            for (t, op) in ops {
                if self.at(t) {
                    found = Some(*op);
                    break;
                }
            }
            match found {
                Some(op) => {
                    self.bump();
                    self.skip_newlines();
                    let rhs = next(self)?;
                    let span = lhs.span().to(rhs.span());
                    lhs = Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs), span };
                }
                None => break,
            }
        }
        Ok(lhs)
    }

    fn parse_bitor(&mut self) -> PResult<Expr> {
        self.binary_level(Self::parse_bitxor, &[(Tok::Pipe, BinOp::BitOr)])
    }
    fn parse_bitxor(&mut self) -> PResult<Expr> {
        self.binary_level(Self::parse_bitand, &[(Tok::Caret, BinOp::BitXor)])
    }
    fn parse_bitand(&mut self) -> PResult<Expr> {
        self.binary_level(Self::parse_shift, &[(Tok::Amp, BinOp::BitAnd)])
    }
    fn parse_shift(&mut self) -> PResult<Expr> {
        self.binary_level(Self::parse_additive, &[(Tok::LtLt, BinOp::Shl), (Tok::GtGt, BinOp::Shr)])
    }
    fn parse_additive(&mut self) -> PResult<Expr> {
        self.binary_level(
            Self::parse_multiplicative,
            &[(Tok::Plus, BinOp::Add), (Tok::Minus, BinOp::Sub), (Tok::PlusPct, BinOp::AddWrap), (Tok::MinusPct, BinOp::SubWrap), (Tok::PlusPipe, BinOp::AddSat), (Tok::MinusPipe, BinOp::SubSat)],
        )
    }
    fn parse_multiplicative(&mut self) -> PResult<Expr> {
        self.binary_level(Self::parse_cast, &[(Tok::Star, BinOp::Mul), (Tok::Slash, BinOp::Div), (Tok::Percent, BinOp::Rem), (Tok::StarPct, BinOp::MulWrap), (Tok::StarPipe, BinOp::MulSat)])
    }

    fn parse_cast(&mut self) -> PResult<Expr> {
        let mut e = self.parse_unary()?;
        while self.at_ident("as") {
            self.bump();
            let ty = self.parse_type()?;
            let span = e.span().to(ty.span());
            e = Expr::Cast { expr: Box::new(e), ty, span };
        }
        Ok(e)
    }

    fn parse_unary(&mut self) -> PResult<Expr> {
        let start = self.span();
        let op = match self.peek() {
            Tok::Minus => Some(UnOp::Neg),
            Tok::Bang => Some(UnOp::Not),
            Tok::Tilde => Some(UnOp::BitNot),
            Tok::Amp => {
                self.bump();
                let mutable = self.eat_ident("mut");
                let e = self.parse_unary()?;
                let span = start.to(e.span());
                return Ok(Expr::Unary { op: if mutable { UnOp::AddrOfMut } else { UnOp::AddrOf }, expr: Box::new(e), span });
            }
            Tok::Ident(s) if s == "try" => {
                self.bump();
                let e = self.parse_unary()?;
                let span = start.to(e.span());
                return Ok(Expr::Try { expr: Box::new(e), span });
            }
            Tok::Ident(s) if s == "comptime" => {
                self.bump();
                let e = self.parse_unary()?;
                let span = start.to(e.span());
                return Ok(Expr::Comptime { expr: Box::new(e), span });
            }
            _ => None,
        };
        if let Some(op) = op {
            self.bump();
            let e = self.parse_unary()?;
            let span = start.to(e.span());
            // fold negative literals
            if let (UnOp::Neg, Expr::Lit { value: Lit::Float(f), .. }) = (op, &e) {
                return Ok(Expr::Lit { value: Lit::Float(-f), span });
            }
            return Ok(Expr::Unary { op, expr: Box::new(e), span });
        }
        self.parse_postfix()
    }

    fn parse_args(&mut self) -> PResult<Vec<Expr>> {
        self.expect(&Tok::LParen)?;
        let saved = self.no_struct_lit;
        self.no_struct_lit = false;
        let r = self.parse_args_inner();
        self.no_struct_lit = saved;
        r
    }

    fn parse_args_inner(&mut self) -> PResult<Vec<Expr>> {
        let mut args = Vec::new();
        loop {
            if self.at(&Tok::RParen) {
                break;
            }
            args.push(self.parse_arg()?);
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RParen)?;
        Ok(args)
    }

    /// An argument may be a type (generic instantiation, builtin argument).
    fn parse_arg(&mut self) -> PResult<Expr> {
        let start = self.span();
        let is_type_start = match self.peek() {
            Tok::LBracket => {
                (matches!(self.peek_at(1), Tok::RBracket) && matches!(self.peek_at(2), Tok::Ident(_) | Tok::LBracket | Tok::Star | Tok::Question))
                    || (matches!(self.peek_at(1), Tok::Int(_)) && matches!(self.peek_at(2), Tok::RBracket) && matches!(self.peek_at(3), Tok::Ident(_) | Tok::LBracket | Tok::Star | Tok::Question))
            }
            Tok::Star | Tok::Question => true,
            Tok::Ident(s) => (s == "fn" && self.peek_at(1) == &Tok::LParen) || s == "weak" || s == "dyn",
            _ => false,
        };
        if is_type_start {
            let ty = self.parse_type()?;
            return Ok(Expr::TypeVal { span: start.to(self.prev_span()), ty });
        }
        self.parse_expr()
    }

    fn parse_postfix(&mut self) -> PResult<Expr> {
        let mut e = self.parse_primary()?;
        loop {
            // method chains may continue on the next line
            if self.at(&Tok::Newline) && matches!(self.peek_at(1), Tok::Dot) && matches!(self.peek_at(2), Tok::Ident(_)) {
                self.bump();
            }
            match self.peek().clone() {
                Tok::LParen => {
                    let args = self.parse_args()?;
                    let span = e.span().to(self.prev_span());
                    e = Expr::Call { callee: Box::new(e), args, span };
                }
                Tok::LBracket => {
                    self.bump();
                    if self.at(&Tok::DotDot) {
                        self.bump();
                        let end = if self.at(&Tok::RBracket) { None } else { Some(Box::new(self.parse_expr()?)) };
                        self.expect(&Tok::RBracket)?;
                        let span = e.span().to(self.prev_span());
                        e = Expr::SliceOp { base: Box::new(e), start: None, end, span };
                        continue;
                    }
                    let idx = self.parse_expr()?;
                    if self.eat(&Tok::DotDot) {
                        let end = if self.at(&Tok::RBracket) { None } else { Some(Box::new(self.parse_expr()?)) };
                        self.expect(&Tok::RBracket)?;
                        let span = e.span().to(self.prev_span());
                        e = Expr::SliceOp { base: Box::new(e), start: Some(Box::new(idx)), end, span };
                    } else {
                        self.expect(&Tok::RBracket)?;
                        let span = e.span().to(self.prev_span());
                        e = Expr::Index { base: Box::new(e), index: Box::new(idx), span };
                    }
                }
                Tok::Dot => {
                    self.bump();
                    match self.peek().clone() {
                        Tok::Ident(name) => {
                            self.bump();
                            if self.at(&Tok::LParen) {
                                let args = self.parse_args()?;
                                let span = e.span().to(self.prev_span());
                                e = Expr::MethodCall { receiver: Box::new(e), method: name, args, span };
                            } else {
                                let span = e.span().to(self.prev_span());
                                e = Expr::Field { base: Box::new(e), name, span };
                            }
                        }
                        Tok::Int(i) => {
                            self.bump();
                            let span = e.span().to(self.prev_span());
                            e = Expr::Field { base: Box::new(e), name: i.to_string(), span };
                        }
                        other => {
                            let sp = self.span();
                            return self.error(sp, format!("expected a field name after `.` but found {}", other.describe()));
                        }
                    }
                }
                Tok::DotStar => {
                    self.bump();
                    let span = e.span().to(self.prev_span());
                    e = Expr::Deref { expr: Box::new(e), span };
                }
                Tok::DotQuestion => {
                    self.bump();
                    let span = e.span().to(self.prev_span());
                    e = Expr::Unwrap { expr: Box::new(e), span };
                }
                Tok::LBrace if !self.no_struct_lit && self.is_type_path(&e) => {
                    let ty = self.expr_to_type(&e)?;
                    let fields = self.parse_struct_lit_fields()?;
                    let span = e.span().to(self.prev_span());
                    e = Expr::StructLit { ty: Some(ty), fields, span };
                }
                _ => break,
            }
        }
        Ok(e)
    }

    fn is_type_path(&self, e: &Expr) -> bool {
        match e {
            // `Point{ ... }`; a lowercase name is accepted after a module path (`c.cv_point{ ... }`)
            Expr::Ident { name, .. } => name.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false),
            Expr::Field { base, .. } => self.is_path(base),
            Expr::Call { callee, args, .. } => {
                // `List(i32){ ... }` generic instantiation literal
                self.is_type_path(callee) && args.iter().all(|a| matches!(a, Expr::TypeVal { .. } | Expr::Ident { .. }))
            }
            _ => false,
        }
    }
    fn is_path(&self, e: &Expr) -> bool {
        match e {
            Expr::Ident { .. } => true,
            Expr::Field { base, .. } => self.is_path(base),
            _ => false,
        }
    }

    fn expr_to_type(&mut self, e: &Expr) -> PResult<TypeExpr> {
        match e {
            Expr::Ident { name, span } => Ok(TypeExpr::Named { path: vec![name.clone()], args: vec![], span: *span }),
            Expr::Field { base, name, span } => {
                let mut t = self.expr_to_type(base)?;
                if let TypeExpr::Named { path, .. } = &mut t {
                    path.push(name.clone());
                }
                if let TypeExpr::Named { span: s, .. } = &mut t {
                    *s = *span;
                }
                Ok(t)
            }
            Expr::Call { callee, args, span } => {
                let mut t = self.expr_to_type(callee)?;
                let mut targs = Vec::new();
                for a in args {
                    targs.push(self.expr_to_type(a)?);
                }
                if let TypeExpr::Named { args: ta, span: s, .. } = &mut t {
                    *ta = targs;
                    *s = *span;
                }
                Ok(t)
            }
            Expr::TypeVal { ty, .. } => Ok(ty.clone()),
            _ => self.error(e.span(), "expected a type"),
        }
    }

    fn parse_struct_lit_fields(&mut self) -> PResult<Vec<(String, Expr, Span)>> {
        // at `{`
        self.expect(&Tok::LBrace)?;
        let mut fields = Vec::new();
        loop {
            self.skip_newlines();
            if self.at(&Tok::RBrace) {
                break;
            }
            let start = self.span();
            self.expect(&Tok::Dot)?;
            let (name, _) = self.expect_ident()?;
            self.expect(&Tok::Eq)?;
            self.skip_newlines();
            let value = self.parse_expr()?;
            fields.push((name, value, start.to(self.prev_span())));
            self.skip_newlines();
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.skip_newlines();
        self.expect(&Tok::RBrace)?;
        Ok(fields)
    }

    fn parse_primary(&mut self) -> PResult<Expr> {
        let start = self.span();
        match self.peek().clone() {
            Tok::Int(v) => {
                self.bump();
                return Ok(Expr::Lit { value: Lit::Int(v), span: start });
            }
            Tok::Float(v) => {
                self.bump();
                return Ok(Expr::Lit { value: Lit::Float(v), span: start });
            }
            Tok::Str(s) => {
                self.bump();
                return Ok(Expr::Lit { value: Lit::Str(s), span: start });
            }
            Tok::Bytes(s) => {
                self.bump();
                return Ok(Expr::Lit { value: Lit::Bytes(s), span: start });
            }
            Tok::Char(c) => {
                self.bump();
                return Ok(Expr::Lit { value: Lit::Char(c), span: start });
            }
            Tok::LParen => {
                self.bump();
                if self.at(&Tok::RParen) {
                    self.bump();
                    return Ok(Expr::TupleLit { elems: vec![], span: start.to(self.prev_span()) });
                }
                // inside parentheses a struct literal is unambiguous again
                let saved = self.no_struct_lit;
                self.no_struct_lit = false;
                let r = self.parse_paren_rest(start);
                self.no_struct_lit = saved;
                return r;
            }
            Tok::LBracket => {
                let saved = self.no_struct_lit;
                self.no_struct_lit = false;
                let r = self.parse_bracket_rest(start);
                self.no_struct_lit = saved;
                return r;
            }
            _ => {}
        }
        self.parse_primary_rest(start)
    }

    fn parse_paren_rest(&mut self, start: Span) -> PResult<Expr> {
        {
            {
                let first = self.parse_expr()?;
                if self.eat(&Tok::Comma) {
                    let mut elems = vec![first];
                    loop {
                        if self.at(&Tok::RParen) {
                            break;
                        }
                        elems.push(self.parse_expr()?);
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    self.expect(&Tok::RParen)?;
                    return Ok(Expr::TupleLit { elems, span: start.to(self.prev_span()) });
                }
                self.expect(&Tok::RParen)?;
                Ok(first)
            }
        }
    }

    fn parse_bracket_rest(&mut self, start: Span) -> PResult<Expr> {
        {
            {
                self.bump();
                // `[]T` slice type in expression position, or `[]` the empty array literal
                if self.at(&Tok::RBracket) {
                    if matches!(self.peek_at(1), Tok::Ident(_) | Tok::LBracket | Tok::Star | Tok::Question) && !matches!(self.peek_at(1), Tok::Ident(s) if crate::lexer::is_keyword(s)) {
                        self.pos -= 1;
                        let ty = self.parse_type()?;
                        return Ok(Expr::TypeVal { span: start.to(self.prev_span()), ty });
                    }
                    self.bump();
                    return Ok(Expr::ArrayLit { elems: vec![], span: start.to(self.prev_span()) });
                }
                let mut elems = Vec::new();
                loop {
                    self.skip_newlines();
                    if self.at(&Tok::RBracket) {
                        break;
                    }
                    elems.push(self.parse_expr()?);
                    self.skip_newlines();
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.skip_newlines();
                self.expect(&Tok::RBracket)?;
                // `[3]i32` array type
                if elems.len() == 1
                    && matches!(self.peek(), Tok::Ident(_) | Tok::LBracket | Tok::Star | Tok::Question)
                    && !self.at_ident("as")
                    && !self.at_ident("and")
                    && !self.at_ident("or")
                    && !self.at_ident("catch")
                    && !self.at_ident("orelse")
                    && !self.at_ident("into")
                {
                    let elem = self.parse_type()?;
                    let len = elems.pop().unwrap();
                    let span = start.to(self.prev_span());
                    return Ok(Expr::TypeVal { ty: TypeExpr::Array { len: Box::new(len), elem: Box::new(elem), span }, span });
                }
                Ok(Expr::ArrayLit { elems, span: start.to(self.prev_span()) })
            }
        }
    }

    fn parse_primary_rest(&mut self, start: Span) -> PResult<Expr> {
        match self.peek().clone() {
            Tok::DotLBrace => {
                self.bump();
                self.skip_newlines();
                // anonymous struct `.{ .a = 1 }` or tuple `.{ 1, 2 }`
                if self.at(&Tok::Dot) && matches!(self.peek_at(1), Tok::Ident(_)) && self.peek_at(2) == &Tok::Eq {
                    self.pos -= 1;
                    // re-enter through the field parser: it expects `{`; emulate by adjusting
                    // (DotLBrace was one token, so we handle inline instead)
                    self.pos += 1;
                    let mut fields = Vec::new();
                    loop {
                        self.skip_newlines();
                        if self.at(&Tok::RBrace) {
                            break;
                        }
                        let fstart = self.span();
                        self.expect(&Tok::Dot)?;
                        let (name, _) = self.expect_ident()?;
                        self.expect(&Tok::Eq)?;
                        self.skip_newlines();
                        let value = self.parse_expr()?;
                        fields.push((name, value, fstart.to(self.prev_span())));
                        self.skip_newlines();
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    self.skip_newlines();
                    self.expect(&Tok::RBrace)?;
                    return Ok(Expr::StructLit { ty: None, fields, span: start.to(self.prev_span()) });
                }
                let mut elems = Vec::new();
                loop {
                    self.skip_newlines();
                    if self.at(&Tok::RBrace) {
                        break;
                    }
                    elems.push(self.parse_expr()?);
                    self.skip_newlines();
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.skip_newlines();
                self.expect(&Tok::RBrace)?;
                Ok(Expr::TupleLit { elems, span: start.to(self.prev_span()) })
            }
            Tok::Dot => {
                // `.Variant` or `.Variant(args)`
                self.bump();
                let (name, _) = self.expect_ident()?;
                let args = if self.at(&Tok::LParen) { self.parse_args()? } else { Vec::new() };
                Ok(Expr::ImplicitVariant { name, args, span: start.to(self.prev_span()) })
            }
            Tok::LBrace => {
                let b = self.parse_block(None)?;
                Ok(Expr::Block(b))
            }
            Tok::Pipe => self.parse_closure(),
            Tok::At => {
                self.bump();
                let (name, _) = self.expect_ident()?;
                let args = self.parse_args()?;
                Ok(Expr::Builtin { name, args, span: start.to(self.prev_span()) })
            }
            Tok::LtLt => {
                let segments = self.parse_bin_segments()?;
                self.expect_kw("into")?;
                let target = self.parse_unary()?;
                let span = start.to(self.prev_span());
                Ok(Expr::BinConstruct { segments, target: Box::new(target), span })
            }
            Tok::Star | Tok::Question => {
                let ty = self.parse_type()?;
                Ok(Expr::TypeVal { span: start.to(self.prev_span()), ty })
            }
            Tok::Ident(s) => match s.as_str() {
                "true" => {
                    self.bump();
                    Ok(Expr::Lit { value: Lit::Bool(true), span: start })
                }
                "false" => {
                    self.bump();
                    Ok(Expr::Lit { value: Lit::Bool(false), span: start })
                }
                "null" => {
                    self.bump();
                    Ok(Expr::Null { span: start })
                }
                "undefined" => {
                    self.bump();
                    Ok(Expr::Undefined { span: start })
                }
                "unreachable" => {
                    self.bump();
                    Ok(Expr::Unreachable { span: start })
                }
                "if" => self.parse_if(),
                "match" => self.parse_match(),
                "unsafe" => {
                    self.bump();
                    let body = self.parse_block(None)?;
                    Ok(Expr::Unsafe { body, span: start.to(self.prev_span()) })
                }
                "error" => {
                    self.bump();
                    self.expect(&Tok::Dot)?;
                    let (name, _) = self.expect_ident()?;
                    Ok(Expr::ErrorLit { name, span: start.to(self.prev_span()) })
                }
                "fn" if self.peek_at(1) == &Tok::LParen => {
                    let ty = self.parse_type()?;
                    Ok(Expr::TypeVal { span: start.to(self.prev_span()), ty })
                }
                "weak" | "dyn" => {
                    let ty = self.parse_type()?;
                    Ok(Expr::TypeVal { span: start.to(self.prev_span()), ty })
                }
                _ if PRIMITIVE_TYPES.contains(&s.as_str()) => {
                    self.bump();
                    Ok(Expr::TypeVal { ty: TypeExpr::named(&s, start), span: start })
                }
                _ if crate::lexer::is_keyword(&s) => self.error(start, format!("unexpected keyword `{}` in expression", s)),
                _ if self.peek_at(1) == &Tok::Colon && self.peek_at(2) == &Tok::LBrace => {
                    // labeled block expression: `outer: { ... break :outer v ... }`
                    self.bump();
                    self.bump();
                    let b = self.parse_block(Some(s))?;
                    Ok(Expr::Block(b))
                }
                _ => {
                    self.bump();
                    Ok(Expr::Ident { name: s, span: start })
                }
            },
            other => self.error(start, format!("expected an expression but found {}", other.describe())),
        }
    }

    fn parse_closure(&mut self) -> PResult<Expr> {
        let start = self.expect(&Tok::Pipe)?.span;
        let mut captures = Vec::new();
        if self.eat(&Tok::LBracket) {
            loop {
                if self.at(&Tok::RBracket) {
                    break;
                }
                let cstart = self.span();
                let by_ref = self.eat(&Tok::Amp);
                let mutable = by_ref && self.eat_ident("mut");
                let (name, _) = self.expect_ident()?;
                captures.push(Capture { name, by_ref, mutable, span: cstart.to(self.prev_span()) });
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(&Tok::RBracket)?;
        }
        let mut params = Vec::new();
        loop {
            if self.at(&Tok::Pipe) {
                break;
            }
            let pstart = self.span();
            let (name, _) = self.expect_ident()?;
            let ty = if self.eat(&Tok::Colon) { self.parse_type()? } else { TypeExpr::Infer { span: pstart } };
            params.push(Param { name, ty, comptime: false, owned: false, span: pstart.to(self.prev_span()) });
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::Pipe)?;
        let ret = if self.eat(&Tok::Arrow) { Some(self.parse_type()?) } else { None };
        let body = if self.at(&Tok::LBrace) { Expr::Block(self.parse_block(None)?) } else { self.parse_expr()? };
        let span = start.to(self.prev_span());
        Ok(Expr::Closure(ClosureExpr { captures, params, ret, body: Box::new(body), span }))
    }

    /// An expression, or a `return`/`break`/`continue` statement wrapped as a block expression.
    /// An `orelse` / `catch` right-hand side: an `or`-level expression, or a
    /// jump (`return`, `break`, `continue`) wrapped as a diverging block.
    fn parse_or_or_jump(&mut self) -> PResult<Expr> {
        if matches!(self.peek(), Tok::Ident(s) if s == "return" || s == "break" || s == "continue") {
            let start = self.span();
            let stmt = self.parse_stmt()?;
            let sp = start.to(self.prev_span());
            return Ok(Expr::Block(Block { stmts: vec![stmt], tail: None, label: None, span: sp }));
        }
        self.parse_or()
    }

    fn parse_expr_or_jump(&mut self) -> PResult<Expr> {
        let start = self.span();
        let stmt = self.parse_stmt()?;
        match stmt {
            Stmt::Expr(e) => Ok(e),
            other => {
                let sp = start.to(self.prev_span());
                Ok(Expr::Block(Block { stmts: vec![other], tail: None, label: None, span: sp }))
            }
        }
    }

    /// A block after a control-flow header; the braces are required.
    fn parse_body(&mut self, after: &str) -> PResult<Block> {
        if self.at(&Tok::LBrace) {
            return self.parse_block(None);
        }
        let sp = self.span();
        let got = self.peek().describe();
        self.error(sp, format!("expected `{{` after {} but found {}; bodies always take braces (`if c {{ return v }}`)", after, got))
    }

    /// `if c { } else if d { } else { }` and `if let v = opt { } else { }`.
    fn parse_if(&mut self) -> PResult<Expr> {
        let start = self.expect_kw("if")?;
        let binding = if self.eat_ident("let") {
            let (n, _) = self.expect_ident()?;
            self.expect(&Tok::Eq)?;
            Some(n)
        } else {
            None
        };
        let cond = self.parse_expr_no_struct()?;
        if self.at(&Tok::Pipe) && matches!(self.peek_at(1), Tok::Ident(_)) && self.peek_at(2) == &Tok::Pipe {
            let sp = self.span();
            return self.error(sp, "`if (opt) |v|` is now written `if let v = opt { ... }`");
        }
        let then = self.parse_body("the `if` condition")?;
        // allow `else` on the next line
        let els = if self.at_ident("else") || (self.at(&Tok::Newline) && matches!(self.peek_at(1), Tok::Ident(s) if s == "else")) {
            self.skip_newlines();
            self.bump();
            if self.at_ident("if") {
                Some(Box::new(self.parse_if()?))
            } else {
                Some(Box::new(Expr::Block(self.parse_body("`else`")?)))
            }
        } else {
            None
        };
        let span = start.to(self.prev_span());
        match binding {
            Some(b) => Ok(Expr::IfCapture { cond: Box::new(cond), binding: b, then, els, span }),
            None => Ok(Expr::If { cond: Box::new(cond), then, els, span }),
        }
    }

    fn parse_match(&mut self) -> PResult<Expr> {
        let start = self.expect_kw("match")?;
        let scrutinee = self.parse_expr_no_struct()?;
        self.expect(&Tok::LBrace)?;
        let mut arms = Vec::new();
        loop {
            self.skip_newlines();
            if self.at(&Tok::RBrace) {
                break;
            }
            let astart = self.span();
            let pat = self.parse_pattern()?;
            let guard = if self.eat_ident("if") { Some(self.parse_expr()?) } else { None };
            self.expect(&Tok::FatArrow)?;
            self.skip_newlines();
            let body = if self.at(&Tok::LBrace) { Expr::Block(self.parse_block(None)?) } else { self.parse_expr_or_jump()? };
            arms.push(MatchArm { pat, guard, body, span: astart.to(self.prev_span()) });
            if !(self.eat(&Tok::Comma) || self.at(&Tok::Newline) || self.at(&Tok::RBrace)) {
                let sp = self.span();
                let got = self.peek().describe();
                return self.error(sp, format!("expected `,` or newline after match arm but found {}", got));
            }
        }
        self.expect(&Tok::RBrace)?;
        let span = start.to(self.prev_span());
        Ok(Expr::Match { scrutinee: Box::new(scrutinee), arms, span })
    }

    // ----- patterns -------------------------------------------------------

    fn parse_pattern(&mut self) -> PResult<Pattern> {
        let first = self.parse_pattern_atom()?;
        if self.at(&Tok::Pipe) {
            let mut alts = vec![first];
            while self.eat(&Tok::Pipe) {
                alts.push(self.parse_pattern_atom()?);
            }
            let span = alts[0].span().to(alts.last().unwrap().span());
            return Ok(Pattern::Or { alts, span });
        }
        Ok(first)
    }

    fn parse_pattern_atom(&mut self) -> PResult<Pattern> {
        let start = self.span();
        match self.peek().clone() {
            Tok::Int(_) | Tok::Float(_) | Tok::Str(_) | Tok::Bytes(_) | Tok::Char(_) | Tok::Minus => {
                let negative = self.eat(&Tok::Minus);
                let value = match self.bump().tok {
                    Tok::Int(v) => Lit::Int(v),
                    Tok::Float(v) => Lit::Float(v),
                    Tok::Str(s) => Lit::Str(s),
                    Tok::Bytes(s) => Lit::Bytes(s),
                    Tok::Char(c) => Lit::Char(c),
                    _ => return self.error(start, "expected a literal pattern"),
                };
                if self.at(&Tok::DotDot) {
                    self.bump();
                    let inclusive = self.eat(&Tok::Eq);
                    let lo = Expr::Lit { value, span: start };
                    let lo = if negative { Expr::Unary { op: UnOp::Neg, expr: Box::new(lo), span: start } } else { lo };
                    let hi = self.parse_unary()?;
                    let span = start.to(self.prev_span());
                    return Ok(Pattern::Range { lo: Box::new(lo), hi: Box::new(hi), inclusive, span });
                }
                Ok(Pattern::Lit { value, negative, span: start.to(self.prev_span()) })
            }
            Tok::LtLt => {
                let segments = self.parse_bin_segments()?;
                Ok(Pattern::Binary { segments, span: start.to(self.prev_span()) })
            }
            Tok::LParen => {
                self.bump();
                let mut elems = Vec::new();
                loop {
                    if self.at(&Tok::RParen) {
                        break;
                    }
                    elems.push(self.parse_pattern()?);
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.expect(&Tok::RParen)?;
                Ok(Pattern::Tuple { elems, span: start.to(self.prev_span()) })
            }
            Tok::Dot => {
                self.bump();
                let (name, _) = self.expect_ident()?;
                let args = self.parse_pattern_args()?;
                Ok(Pattern::Variant { path: vec![name], args, span: start.to(self.prev_span()) })
            }
            Tok::Ident(s) => match s.as_str() {
                "_" => {
                    self.bump();
                    Ok(Pattern::Wildcard { span: start })
                }
                "else" => {
                    self.bump();
                    Ok(Pattern::Wildcard { span: start })
                }
                "null" => {
                    self.bump();
                    Ok(Pattern::Null { span: start })
                }
                "true" => {
                    self.bump();
                    Ok(Pattern::Lit { value: Lit::Bool(true), negative: false, span: start })
                }
                "false" => {
                    self.bump();
                    Ok(Pattern::Lit { value: Lit::Bool(false), negative: false, span: start })
                }
                "error" => {
                    self.bump();
                    self.expect(&Tok::Dot)?;
                    let (name, _) = self.expect_ident()?;
                    Ok(Pattern::Error { name, span: start.to(self.prev_span()) })
                }
                _ if crate::lexer::is_keyword(&s) => self.error(start, format!("unexpected keyword `{}` in pattern", s)),
                _ => {
                    self.bump();
                    if self.at(&Tok::Dot) {
                        // `Enum.Variant(...)` path pattern
                        let mut path = vec![s];
                        while self.eat(&Tok::Dot) {
                            let (n, _) = self.expect_ident()?;
                            path.push(n);
                        }
                        let args = self.parse_pattern_args()?;
                        return Ok(Pattern::Variant { path, args, span: start.to(self.prev_span()) });
                    }
                    if self.at(&Tok::LParen) {
                        let args = self.parse_pattern_args()?;
                        return Ok(Pattern::Variant { path: vec![s], args, span: start.to(self.prev_span()) });
                    }
                    Ok(Pattern::Binding { name: s, span: start })
                }
            },
            other => self.error(start, format!("expected a pattern but found {}", other.describe())),
        }
    }

    fn parse_pattern_args(&mut self) -> PResult<Vec<Pattern>> {
        let mut args = Vec::new();
        if self.at(&Tok::LParen) {
            self.bump();
            loop {
                if self.at(&Tok::RParen) {
                    break;
                }
                args.push(self.parse_pattern()?);
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(&Tok::RParen)?;
        }
        Ok(args)
    }

    // ----- binary patterns ------------------------------------------------

    /// Segment size: a unary expression with `*` products, so `/modifiers` stay free.
    fn parse_bin_size(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_unary()?;
        while self.at(&Tok::Star) {
            self.bump();
            let rhs = self.parse_unary()?;
            let span = lhs.span().to(rhs.span());
            lhs = Expr::Binary { op: BinOp::Mul, lhs: Box::new(lhs), rhs: Box::new(rhs), span };
        }
        Ok(lhs)
    }

    fn parse_bin_segments(&mut self) -> PResult<Vec<BinSegment>> {
        self.expect(&Tok::LtLt)?;
        let mut segs = Vec::new();
        loop {
            self.skip_newlines();
            if self.at(&Tok::GtGt) {
                break;
            }
            let start = self.span();
            let value = match self.peek().clone() {
                Tok::Ident(name) if !crate::lexer::is_keyword(&name) && !matches!(self.peek_at(1), Tok::LParen | Tok::Dot | Tok::LBracket) => {
                    self.bump();
                    // `name:bytes` handled below once size is known
                    BinSegValue::Bind(name)
                }
                _ => {
                    let e = self.parse_bin_size()?;
                    BinSegValue::Expr(Box::new(e))
                }
            };
            let mut size = None;
            let mut value = value;
            if self.eat(&Tok::Colon) {
                if self.at_ident("bytes") {
                    self.bump();
                    value = match value {
                        BinSegValue::Bind(n) => BinSegValue::Rest(n),
                        _ => return self.error(start, "`bytes` requires a binding name"),
                    };
                } else {
                    size = Some(Box::new(self.parse_bin_size()?));
                }
            }
            let mut modifiers = Vec::new();
            if self.eat(&Tok::Slash) {
                loop {
                    let (m, msp) = self.expect_ident()?;
                    if !matches!(m.as_str(), "big" | "little" | "native" | "signed" | "unsigned" | "float" | "utf8" | "bytes") {
                        return self.error(msp, format!("unknown binary segment modifier `{}`", m));
                    }
                    if m == "bytes" {
                        value = match value {
                            BinSegValue::Bind(n) => BinSegValue::Rest(n),
                            other => other,
                        };
                    } else {
                        modifiers.push(m);
                    }
                    if !self.eat(&Tok::Minus) {
                        break;
                    }
                }
            }
            segs.push(BinSegment { value, size, modifiers, span: start.to(self.prev_span()) });
            self.skip_newlines();
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.skip_newlines();
        self.expect(&Tok::GtGt)?;
        Ok(segs)
    }
}
