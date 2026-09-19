//! Abstract syntax tree for Nexium.

use crate::diag::Span;

pub type P<T> = Box<T>;

#[derive(Clone, Debug)]
pub struct Module {
    pub items: Vec<Item>,
    pub file: u32,
}

#[derive(Clone, Debug)]
pub enum Item {
    Fn(FnDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Trait(TraitDecl),
    Impl(ImplDecl),
    Const(ConstDecl),
    Global(GlobalDecl),
    TypeAlias(TypeAliasDecl),
    ErrorSet(ErrorSetDecl),
    Import(ImportDecl),
    Test(TestDecl),
    Artifact(ArtifactDecl),
}

#[derive(Clone, Debug)]
pub struct Attrs {
    pub is_pub: bool,
    pub doc: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct EffectBound {
    pub negative: bool,
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub ty: TypeExpr,
    pub comptime: bool,
    /// `own name: T`: the callee takes ownership (the argument is moved)
    pub owned: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct WhereClause {
    pub param: String,
    pub bounds: Vec<String>, // trait names
    pub effect_bounds: Vec<EffectBound>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FnDecl {
    pub attrs: Attrs,
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Option<TypeExpr>,
    pub effects: Vec<EffectBound>,
    pub wheres: Vec<WhereClause>,
    pub export: Option<String>, // export(c)
    pub extern_c: bool,
    /// C variadic (`...`) function; only from `@cImport`
    pub variadic: bool,
    /// the header this declaration was imported from
    pub cimport_header: Option<String>,
    pub body: Option<Block>,
    pub span: Span,
    pub name_span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructKind {
    Struct,
    Record,
    RefClass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Default,
    C,
    Packed,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub ty: TypeExpr,
    pub default: Option<Expr>,
    pub constraint: Option<Expr>, // record `where` clause; `value` refers to the field
    pub doc: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct StructDecl {
    pub attrs: Attrs,
    pub kind: StructKind,
    pub name: String,
    pub type_params: Vec<String>,
    pub layout: Layout,
    pub derives: Vec<String>,
    pub fields: Vec<Field>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Variant {
    pub name: String,
    pub payload: VariantPayload,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum VariantPayload {
    Unit,
    Tuple(Vec<TypeExpr>),
    Struct(Vec<Field>),
}

#[derive(Clone, Debug)]
pub struct EnumDecl {
    pub attrs: Attrs,
    pub name: String,
    pub type_params: Vec<String>,
    pub variants: Vec<Variant>,
    pub derives: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct TraitDecl {
    pub attrs: Attrs,
    pub name: String,
    pub assoc_types: Vec<String>,
    pub methods: Vec<FnDecl>, // bodies optional (defaults)
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ImplDecl {
    pub trait_name: Option<String>,
    pub target: TypeExpr,
    pub type_params: Vec<String>,
    pub assoc_types: Vec<(String, TypeExpr)>,
    pub methods: Vec<FnDecl>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ConstDecl {
    pub attrs: Attrs,
    pub name: String,
    pub ty: Option<TypeExpr>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct GlobalDecl {
    pub attrs: Attrs,
    pub name: String,
    pub ty: TypeExpr,
    pub value: Option<Expr>, // None => undefined
    pub align: Option<u64>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct TypeAliasDecl {
    pub attrs: Attrs,
    pub name: String,
    pub distinct: bool,
    pub ty: TypeExpr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ErrorSetDecl {
    pub attrs: Attrs,
    pub name: String,
    pub names: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ImportDecl {
    pub path: Vec<String>,
    pub alias: Option<String>,
    pub names: Option<Vec<String>>, // import a.b.{x, y}
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct TestDecl {
    pub name: String,
    pub comptime: bool,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ArtifactDecl {
    pub kind: String, // cabi, python, rustlib, node, lib, cli, app, shared, installer
    pub fields: Vec<(String, Expr, Span)>,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Types

#[derive(Clone, Debug)]
pub enum TypeExpr {
    /// A named type, possibly generic: `Point`, `List(i32)`, `pkg.Type`
    Named {
        path: Vec<String>,
        args: Vec<TypeExpr>,
        span: Span,
    },
    Array {
        len: P<Expr>,
        elem: P<TypeExpr>,
        span: Span,
    },
    Slice {
        mutable: bool,
        elem: P<TypeExpr>,
        span: Span,
    },
    Ptr {
        mutable: bool,
        elem: P<TypeExpr>,
        span: Span,
    },
    Optional {
        elem: P<TypeExpr>,
        span: Span,
    },
    ErrorUnion {
        set: Option<P<TypeExpr>>,
        elem: P<TypeExpr>,
        span: Span,
    },
    Fn {
        params: Vec<TypeExpr>,
        ret: P<TypeExpr>,
        effects: Vec<EffectBound>,
        span: Span,
    },
    Tuple {
        elems: Vec<TypeExpr>,
        span: Span,
    },
    Weak {
        elem: P<TypeExpr>,
        span: Span,
    },
    Dyn {
        trait_name: String,
        effects: Vec<EffectBound>,
        span: Span,
    },
    Infer {
        span: Span,
    },
}

impl TypeExpr {
    pub fn span(&self) -> Span {
        match self {
            TypeExpr::Named { span, .. }
            | TypeExpr::Array { span, .. }
            | TypeExpr::Slice { span, .. }
            | TypeExpr::Ptr { span, .. }
            | TypeExpr::Optional { span, .. }
            | TypeExpr::ErrorUnion { span, .. }
            | TypeExpr::Fn { span, .. }
            | TypeExpr::Tuple { span, .. }
            | TypeExpr::Weak { span, .. }
            | TypeExpr::Dyn { span, .. }
            | TypeExpr::Infer { span } => *span,
        }
    }
    pub fn named(name: &str, span: Span) -> TypeExpr {
        TypeExpr::Named { path: vec![name.to_string()], args: vec![], span }
    }
}

// ---------------------------------------------------------------------------
// Statements

#[derive(Clone, Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    /// trailing expression producing the block's value (no newline after it)
    pub tail: Option<P<Expr>>,
    pub label: Option<String>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Let {
        mutable: bool,
        name: String,
        ty: Option<TypeExpr>,
        init: Option<Expr>,
        span: Span,
    },
    Assign {
        target: Expr,
        op: Option<BinOp>,
        value: Expr,
        span: Span,
    },
    Expr(Expr),
    Return {
        value: Option<Expr>,
        span: Span,
    },
    Break {
        label: Option<String>,
        value: Option<Expr>,
        span: Span,
    },
    Continue {
        label: Option<String>,
        span: Span,
    },
    Defer {
        body: P<Stmt>,
        span: Span,
    },
    ErrDefer {
        body: P<Stmt>,
        span: Span,
    },
    While {
        cond: Expr,
        body: Block,
        label: Option<String>,
        span: Span,
    },
    For {
        iter: ForIter,
        bindings: Vec<String>,
        body: Block,
        label: Option<String>,
        parallel: bool,
        span: Span,
    },
    Unsafe {
        body: Block,
        span: Span,
    },
    Discard {
        value: Expr,
        span: Span,
    },
    /// `using arena { ... }`: install an allocation strategy for the block
    Using {
        strategy: String,
        body: Block,
        span: Span,
    },
}

#[derive(Clone, Debug)]
pub enum ForIter {
    Range { start: Expr, end: Expr },
    Items(Vec<Expr>), // parallel iteration when more than one
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Let { span, .. }
            | Stmt::Assign { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::Break { span, .. }
            | Stmt::Continue { span, .. }
            | Stmt::Defer { span, .. }
            | Stmt::ErrDefer { span, .. }
            | Stmt::While { span, .. }
            | Stmt::For { span, .. }
            | Stmt::Unsafe { span, .. }
            | Stmt::Using { span, .. }
            | Stmt::Discard { span, .. } => *span,
            Stmt::Expr(e) => e.span(),
        }
    }
}

// ---------------------------------------------------------------------------
// Expressions

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    AddWrap,
    SubWrap,
    MulWrap,
    AddSat,
    SubSat,
    MulSat,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl BinOp {
    pub fn is_comparison(self) -> bool {
        matches!(self, BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge)
    }
    pub fn is_logical(self) -> bool {
        matches!(self, BinOp::And | BinOp::Or)
    }
    pub fn is_arith(self) -> bool {
        matches!(self, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem | BinOp::AddWrap | BinOp::SubWrap | BinOp::MulWrap | BinOp::AddSat | BinOp::SubSat | BinOp::MulSat)
    }
    pub fn is_bitwise(self) -> bool {
        matches!(self, BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr)
    }
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::AddWrap => "+%",
            BinOp::SubWrap => "-%",
            BinOp::MulWrap => "*%",
            BinOp::AddSat => "+|",
            BinOp::SubSat => "-|",
            BinOp::MulSat => "*|",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::BitXor => "^",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::And => "and",
            BinOp::Or => "or",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
    BitNot,
    AddrOf,
    AddrOfMut,
}

#[derive(Clone, Debug)]
pub enum Lit {
    Int(u128),
    Float(f64),
    Str(Vec<u8>),
    Bytes(Vec<u8>),
    Char(u32),
    Bool(bool),
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pat: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Capture {
    pub name: String,
    pub by_ref: bool,
    pub mutable: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ClosureExpr {
    pub captures: Vec<Capture>,
    pub params: Vec<Param>,
    pub ret: Option<TypeExpr>,
    pub body: P<Expr>,
    pub span: Span,
}

/// One segment of a binary pattern or construction: `name:size/mods` or a literal.
#[derive(Clone, Debug)]
pub struct BinSegment {
    pub value: BinSegValue,
    pub size: Option<P<Expr>>, // bits; None => default (8 for ints, all for `bytes`)
    pub modifiers: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum BinSegValue {
    Bind(String),  // pattern: binds a name
    Expr(P<Expr>), // construction: an expression; pattern: a literal expression to match
    Rest(String),  // `name:bytes`
}

#[derive(Clone, Debug)]
pub enum Expr {
    Lit {
        value: Lit,
        span: Span,
    },
    Ident {
        name: String,
        span: Span,
    },
    /// `a.b.c` where the head is an identifier; resolved by the checker to a field access,
    /// enum variant, module path, or associated function.
    Field {
        base: P<Expr>,
        name: String,
        span: Span,
    },
    /// `.Variant` with the enum inferred from context, or `.Variant(args)`
    ImplicitVariant {
        name: String,
        args: Vec<Expr>,
        span: Span,
    },
    Index {
        base: P<Expr>,
        index: P<Expr>,
        span: Span,
    },
    SliceOp {
        base: P<Expr>,
        start: Option<P<Expr>>,
        end: Option<P<Expr>>,
        span: Span,
    },
    Call {
        callee: P<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    MethodCall {
        receiver: P<Expr>,
        method: String,
        args: Vec<Expr>,
        span: Span,
    },
    Unary {
        op: UnOp,
        expr: P<Expr>,
        span: Span,
    },
    Binary {
        op: BinOp,
        lhs: P<Expr>,
        rhs: P<Expr>,
        span: Span,
    },
    /// `x |> f(a)` == `f(x, a)`
    Pipe {
        lhs: P<Expr>,
        rhs: P<Expr>,
        span: Span,
    },
    Try {
        expr: P<Expr>,
        span: Span,
    },
    Catch {
        expr: P<Expr>,
        binding: Option<String>,
        handler: P<Expr>,
        span: Span,
    },
    OrElse {
        expr: P<Expr>,
        default: P<Expr>,
        span: Span,
    },
    Unwrap {
        expr: P<Expr>,
        span: Span,
    },
    Deref {
        expr: P<Expr>,
        span: Span,
    },
    If {
        cond: P<Expr>,
        then: Block,
        els: Option<P<Expr>>,
        span: Span,
    },
    /// `if (opt) |v| { } else { }`
    IfCapture {
        cond: P<Expr>,
        binding: String,
        then: Block,
        els: Option<P<Expr>>,
        span: Span,
    },
    Match {
        scrutinee: P<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Block(Block),
    /// `Name{ .x = 1 }` or `.{ .x = 1 }` (name = None)
    StructLit {
        ty: Option<TypeExpr>,
        fields: Vec<(String, Expr, Span)>,
        span: Span,
    },
    /// `.{ a, b }` positional tuple, or `[a, b, c]` array
    ArrayLit {
        elems: Vec<Expr>,
        span: Span,
    },
    TupleLit {
        elems: Vec<Expr>,
        span: Span,
    },
    Closure(ClosureExpr),
    Range {
        start: P<Expr>,
        end: P<Expr>,
        span: Span,
    },
    Cast {
        expr: P<Expr>,
        ty: TypeExpr,
        span: Span,
    },
    Null {
        span: Span,
    },
    Undefined {
        span: Span,
    },
    Unreachable {
        span: Span,
    },
    ErrorLit {
        name: String,
        span: Span,
    },
    /// `@name(args)`
    Builtin {
        name: String,
        args: Vec<Expr>,
        span: Span,
    },
    /// `<<...>> into buf` construction
    BinConstruct {
        segments: Vec<BinSegment>,
        target: P<Expr>,
        span: Span,
    },
    Comptime {
        expr: P<Expr>,
        span: Span,
    },
    /// A type used in expression position (generic argument, `@typeName(i32)`)
    TypeVal {
        ty: TypeExpr,
        span: Span,
    },
    Unsafe {
        body: Block,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Lit { span, .. }
            | Expr::Ident { span, .. }
            | Expr::Field { span, .. }
            | Expr::ImplicitVariant { span, .. }
            | Expr::Index { span, .. }
            | Expr::SliceOp { span, .. }
            | Expr::Call { span, .. }
            | Expr::MethodCall { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Pipe { span, .. }
            | Expr::Try { span, .. }
            | Expr::Catch { span, .. }
            | Expr::OrElse { span, .. }
            | Expr::Unwrap { span, .. }
            | Expr::Deref { span, .. }
            | Expr::If { span, .. }
            | Expr::IfCapture { span, .. }
            | Expr::Match { span, .. }
            | Expr::StructLit { span, .. }
            | Expr::ArrayLit { span, .. }
            | Expr::TupleLit { span, .. }
            | Expr::Range { span, .. }
            | Expr::Cast { span, .. }
            | Expr::Null { span }
            | Expr::Undefined { span }
            | Expr::Unreachable { span }
            | Expr::ErrorLit { span, .. }
            | Expr::Builtin { span, .. }
            | Expr::BinConstruct { span, .. }
            | Expr::Comptime { span, .. }
            | Expr::TypeVal { span, .. }
            | Expr::Unsafe { span, .. } => *span,
            Expr::Block(b) => b.span,
            Expr::Closure(c) => c.span,
        }
    }
}

// ---------------------------------------------------------------------------
// Patterns

#[derive(Clone, Debug)]
pub enum Pattern {
    Wildcard {
        span: Span,
    },
    Binding {
        name: String,
        span: Span,
    },
    Lit {
        value: Lit,
        negative: bool,
        span: Span,
    },
    /// `.Variant`, `.Variant(p, q)`, `Enum.Variant(...)`
    Variant {
        path: Vec<String>,
        args: Vec<Pattern>,
        span: Span,
    },
    /// `error.Name`
    Error {
        name: String,
        span: Span,
    },
    Null {
        span: Span,
    },
    /// `else` in an error match, or `_`
    Range {
        lo: P<Expr>,
        hi: P<Expr>,
        inclusive: bool,
        span: Span,
    },
    Or {
        alts: Vec<Pattern>,
        span: Span,
    },
    Binary {
        segments: Vec<BinSegment>,
        span: Span,
    },
    Tuple {
        elems: Vec<Pattern>,
        span: Span,
    },
}

impl Pattern {
    pub fn span(&self) -> Span {
        match self {
            Pattern::Wildcard { span }
            | Pattern::Binding { span, .. }
            | Pattern::Lit { span, .. }
            | Pattern::Variant { span, .. }
            | Pattern::Error { span, .. }
            | Pattern::Null { span }
            | Pattern::Range { span, .. }
            | Pattern::Or { span, .. }
            | Pattern::Binary { span, .. }
            | Pattern::Tuple { span, .. } => *span,
        }
    }
}
