//! Typed intermediate representation, produced by the checker and consumed by
//! the C backend and the compile-time interpreter.

use crate::ast::{BinOp, UnOp};
use crate::diag::Span;
use crate::effects::Effects;
use crate::types::{DefId, TyId};
use std::collections::HashSet;

pub type LocalId = u32;
pub type InstId = u32;
pub type LabelId = u32;

#[derive(Clone, Debug)]
pub struct Local {
    pub name: String,
    pub ty: TyId,
    pub mutable: bool,
    pub span: Span,
    pub is_param: bool,
    /// an `own` parameter: moved in by the caller, dropped here at scope exit
    pub owned: bool,
}

#[derive(Clone, Debug)]
pub struct TFunc {
    pub name: String,
    pub mangled: String,
    pub def: Option<DefId>,
    pub targs: Vec<TyId>,
    pub params: Vec<LocalId>,
    pub ret: TyId,
    pub locals: Vec<Local>,
    pub body: Option<TBlock>,
    /// effects the function itself performs (before propagation)
    pub own_effects: Effects,
    /// witnesses: which span introduced each effect (for diagnostics)
    pub witnesses: Vec<(Effects, Span, String)>,
    /// callees (instances) for effect propagation
    pub callees: Vec<(InstId, Span)>,
    /// effects after propagation
    pub effects: Effects,
    pub declared_neg: Effects,
    pub declared_pos: Effects,
    pub declared_spans: Vec<(Effects, Span)>,
    pub export: Option<String>,
    pub is_extern: bool,
    pub is_variadic: bool,
    pub cimport: bool,
    pub is_pub: bool,
    pub is_test: bool,
    pub test_comptime: bool,
    pub is_closure: bool,
    pub closure_env: Option<TyId>,
    pub span: Span,
    pub module: u32,
    pub moved: HashSet<LocalId>,
    pub takes_ctx: bool,
}

#[derive(Clone, Debug)]
pub struct TBlock {
    pub stmts: Vec<TStmt>,
    pub tail: Option<Box<TExpr>>,
    pub label: Option<LabelId>,
    pub ty: TyId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum TStmt {
    Let {
        local: LocalId,
        init: Option<TExpr>,
        span: Span,
    },
    Assign {
        target: TExpr,
        op: Option<(BinOp, ArithMode)>,
        value: TExpr,
        span: Span,
    },
    Expr(TExpr),
    Return {
        value: Option<TExpr>,
        span: Span,
    },
    Break {
        label: LabelId,
        value: Option<TExpr>,
        span: Span,
    },
    Continue {
        label: LabelId,
        span: Span,
    },
    Defer {
        body: Box<TStmt>,
        span: Span,
    },
    ErrDefer {
        body: Box<TStmt>,
        span: Span,
    },
    While {
        cond: TExpr,
        body: TBlock,
        label: LabelId,
        span: Span,
    },
    ForRange {
        var: LocalId,
        start: TExpr,
        end: TExpr,
        body: TBlock,
        label: LabelId,
        span: Span,
    },
    /// iterate one or more slices in lockstep; `index` receives the position
    ForSlice {
        items: Vec<(LocalId, TExpr)>,
        index: Option<LocalId>,
        body: TBlock,
        label: LabelId,
        span: Span,
        parallel: bool,
    },
    Block(TBlock),
    /// drop a local explicitly (inserted for scope-exit release when needed)
    Drop {
        local: LocalId,
        span: Span,
    },
    /// run the block with an arena installed as the context allocator
    Using {
        strategy: String,
        body: TBlock,
        span: Span,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArithMode {
    Checked,
    Wrap,
    Sat,
    Float,
    Plain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastKind {
    /// integer to integer; `checked` means the range must be verified at runtime
    IntToInt {
        checked: bool,
    },
    IntToFloat,
    FloatToInt,
    FloatToFloat,
    /// representation-preserving (distinct types, char/int, bool/int, enum/int)
    Bits,
    PtrToPtr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Builtin {
    Print,
    Println,
    Eprintln,
    Format,
    Expect,
    ExpectEq,
    Panic,
    TypeName,
    SizeOf,
    Truncate,
    ErrorName,
    Len,
    // collections
    ListNew,
    ListWithCapacity,
    ListAppend,
    ListPop,
    ListClear,
    ListClone,
    ListLast,
    ListInsert,
    ListRemove,
    ListExtend,
    ListReserve,
    ListItems,
    ListContains,
    ListSort,
    ListReverse,
    ListFill,
    ListSwapRemove,
    StringNew,
    StringFrom,
    StringWithCapacity,
    StringAppend,
    StringAppendChar,
    StringPushByte,
    StringClone,
    StringClear,
    StringPop,
    StringBytes,
    MapNew,
    MapPut,
    MapGet,
    MapContains,
    MapRemove,
    MapClear,
    MapClone,
    MapKeys,
    MapValues,
    // slices
    SliceEq,
    SliceCopy,
    SliceFill,
    SliceReverse,
    SliceSort,
    SliceContains,
    SliceIndexOf,
    SliceStartsWith,
    SliceEndsWith,
    SliceFind,
    SliceTrim,
    SliceSplit,
    SliceToOwned,
    SliceParseInt,
    SliceParseFloat,
    SliceLines,
    SliceEqIgnoreCase,
    // math
    MathSqrt,
    MathAbs,
    MathMin,
    MathMax,
    MathPow,
    MathFloor,
    MathCeil,
    MathRound,
    MathSin,
    MathCos,
    MathTan,
    MathExp,
    MathLog,
    MathLog2,
    MathAtan2,
    MathClamp,
    // refs
    Weak,
    Upgrade,
    Retain,
    Release,
    RefCount,
    Drop,
    // utf8 / char
    CharIsDigit,
    CharIsAlpha,
    CharIsSpace,
    CharToLower,
    CharToUpper,
    CharToDigit,
    Utf8Encode,
    Utf8Decode,
    Utf8Validate,
    // io/os (platform)
    ReadFile,
    WriteFile,
    AppendFile,
    FsKind,
    FsSize,
    FsModified,
    FsMkdir,
    FsRemoveFile,
    FsRemoveDir,
    FsRename,
    FsListDir,
    FsCwd,
    FsTempDir,
    ReadLine,
    Args,
    Env,
    Exit,
    Run,
    TimeNow,
    TimeMonotonic,
    Sleep,
    Random,
    RandomInt,
    RandomFloat,
    RandomSeed,
    // records
    RecordValidate,
    // misc
    Assert,
    Unreachable,
    SliceFromRaw,
    PtrAdd,
    IntToStr,
    FloatToStr,
    Hash,
    ParallelFor,
    AllocArena,
    ArenaReset,
    ArenaFree,
    WithAllocator,
    RefEq,
    ListFromSlice,
    Ordering,
}

#[derive(Clone, Debug)]
pub struct TExpr {
    pub kind: TExprKind,
    pub ty: TyId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum TExprKind {
    Int(i128),
    Float(f64),
    Bool(bool),
    Char(u32),
    /// `[]u8` pointing at static data
    Str(Vec<u8>),
    Unit,
    Local(LocalId),
    Global(DefId),
    Const(DefId),
    FnRef(InstId),
    Field {
        base: Box<TExpr>,
        idx: u32,
    },
    /// field of a ref class object (through the reference)
    RefField {
        base: Box<TExpr>,
        idx: u32,
    },
    TupleField {
        base: Box<TExpr>,
        idx: u32,
    },
    /// index into an array, slice, or list; `proven` means the bounds check is discharged
    Index {
        base: Box<TExpr>,
        index: Box<TExpr>,
        proven: bool,
    },
    SliceOp {
        base: Box<TExpr>,
        start: Option<Box<TExpr>>,
        end: Option<Box<TExpr>>,
    },
    Deref(Box<TExpr>),
    AddrOf {
        expr: Box<TExpr>,
        mutable: bool,
    },
    Call {
        inst: InstId,
        args: Vec<TExpr>,
    },
    CallPtr {
        callee: Box<TExpr>,
        args: Vec<TExpr>,
    },
    Builtin {
        op: Builtin,
        args: Vec<TExpr>,
        tys: Vec<TyId>,
    },
    Unary {
        op: UnOp,
        expr: Box<TExpr>,
        mode: ArithMode,
    },
    Binary {
        op: BinOp,
        lhs: Box<TExpr>,
        rhs: Box<TExpr>,
        mode: ArithMode,
        proven: bool,
    },
    /// short-circuit `and` / `or`
    Logical {
        and: bool,
        lhs: Box<TExpr>,
        rhs: Box<TExpr>,
    },
    Cast {
        expr: Box<TExpr>,
        kind: CastKind,
    },
    If {
        cond: Box<TExpr>,
        then: TBlock,
        els: Option<TBlock>,
    },
    IfCapture {
        cond: Box<TExpr>,
        local: LocalId,
        then: TBlock,
        els: Option<TBlock>,
    },
    Match {
        scrutinee: Box<TExpr>,
        arms: Vec<TArm>,
    },
    Block(TBlock),
    StructLit {
        fields: Vec<(u32, TExpr)>,
    },
    /// construct a ref class object on the heap
    RefNew {
        fields: Vec<(u32, TExpr)>,
    },
    EnumLit {
        variant: u32,
        payload: Vec<TExpr>,
    },
    ArrayLit(Vec<TExpr>),
    ArrayRepeat {
        value: Box<TExpr>,
        count: u64,
    },
    TupleLit(Vec<TExpr>),
    Try(Box<TExpr>),
    Catch {
        expr: Box<TExpr>,
        err_local: Option<LocalId>,
        handler: Box<TExpr>,
    },
    OrElse {
        expr: Box<TExpr>,
        default: Box<TExpr>,
    },
    Unwrap {
        expr: Box<TExpr>,
        proven: bool,
    },
    OptWrap(Box<TExpr>),
    OptNull,
    ErrWrap(Box<TExpr>),
    ErrVal(u32),
    ArrayToSlice(Box<TExpr>),
    ListToSlice(Box<TExpr>),
    StrToSlice(Box<TExpr>),
    Closure {
        inst: InstId,
        captures: Vec<(LocalId, bool)>,
    },
    /// coerce a plain function reference to a fat function value
    FnToFat(InstId),
    Unreachable,
    Undefined,
    BinConstruct {
        segments: Vec<TBinSeg>,
        target: Box<TExpr>,
    },
    /// an evaluated compile-time constant
    Value(Value),
    TypeVal(TyId),
    /// a record constraint check: evaluates `value` and checks each constraint
    RecordCheck {
        value: Box<TExpr>,
        checks: Vec<(u32, TExpr, LocalId)>,
        as_error: bool,
    },
    /// a `parallel for` body invoked through the executor
    /// `x.weak()` etc handled as Builtin
    Retained(Box<TExpr>),
    /// coerce a pointer to a trait object; `vtable` indexes Program.vtables
    DynFrom {
        expr: Box<TExpr>,
        vtable: u32,
    },
    /// call trait method `method` through a trait object's vtable
    DynCall {
        recv: Box<TExpr>,
        method: u32,
        args: Vec<TExpr>,
    },
}

#[derive(Clone, Debug)]
pub struct TArm {
    pub pat: TPat,
    pub guard: Option<TExpr>,
    pub body: TExpr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum TPat {
    Wild,
    Bind(LocalId),
    Int(i128),
    Float(f64),
    Bool(bool),
    Str(Vec<u8>),
    Char(u32),
    Range { lo: i128, hi: i128, inclusive: bool },
    Variant { idx: u32, args: Vec<TPat> },
    Error(u32),
    Null,
    Some(Box<TPat>),
    Ok(Box<TPat>),
    Or(Vec<TPat>),
    Tuple(Vec<TPat>),
    Binary(Vec<TBinSeg>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Endian {
    Big,
    Little,
    Native,
}

#[derive(Clone, Debug)]
pub enum TBinSize {
    Bits(u64),
    Expr(Box<TExpr>),
    Rest,
}

#[derive(Clone, Debug)]
pub enum TBinSegKind {
    Bind(LocalId),
    Rest(LocalId),
    /// literal to match (pattern) or value to write (construction)
    Value(Box<TExpr>),
}

#[derive(Clone, Debug)]
pub struct TBinSeg {
    pub kind: TBinSegKind,
    pub size: TBinSize,
    pub endian: Endian,
    pub signed: bool,
    pub float: bool,
    pub utf8: bool,
    pub ty: TyId,
    pub span: Span,
}

/// Compile-time values.
/// The result of executing the new statements of a REPL line.
#[derive(Clone, Debug, Default)]
pub struct ReplOutcome {
    /// every `let`/`var` binding of the synthetic main, by name, with its current value
    pub bindings: Vec<(String, Value)>,
    /// the same bindings rendered with their types, for display
    pub shown: Vec<(String, String)>,
    /// the value of a trailing expression statement: (type name, rendered value)
    pub printed: Option<(String, String)>,
    pub panic: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i128),
    Float(f64),
    Bool(bool),
    Char(u32),
    Str(Vec<u8>),
    Void,
    Array(Vec<Value>),
    Tuple(Vec<Value>),
    Struct(Vec<Value>),
    Enum(u32, Vec<Value>),
    Opt(Option<Box<Value>>),
    Err(u32),
    Ok(Box<Value>),
    Type(TyId),
    Undefined,
    /// owned collection values (comptime only)
    List(Vec<Value>),
    /// a Map as insertion-ordered pairs (comptime and REPL only)
    Map(Vec<(Value, Value)>),
    OwnedStr(Vec<u8>),
    Fn(InstId),
    Never,
    /// compile-time pointer: a local and a path of field/element indices into it
    /// a pointer to a place: (call frame, local, path of field/element indices)
    Ptr(u32, LocalId, Vec<usize>),
}

impl Value {
    pub fn as_int(&self) -> Option<i128> {
        match self {
            Value::Int(i) => Some(*i),
            Value::Char(c) => Some(*c as i128),
            Value::Bool(b) => Some(*b as i128),
            _ => None,
        }
    }
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
}
