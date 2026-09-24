" Vim syntax file for Nexium (.nx)
" Install: see editors/vim/README.md. The rules follow the Sublime and
" TextMate grammars in the same directory tree; keep the three in step.

if exists("b:current_syntax")
  finish
endif

syn case match

" ---------------------------------------------------------------- operators
" (first on purpose: of two rules matching at one column Vim uses the one
" defined last, so `//`, `!effect`, `?T` and the rest must come after this)
syn match nexiumOperator "|>\|->\|=>\|\.\.=\?\|[+*-]%\|[+*-]|\|[-+*/%&|^~<>=!?]"

" ---------------------------------------------------------------- comments
syn match nexiumComment "//.*$" contains=nexiumTodo,@Spell
syn match nexiumDocComment "///\%(/\)\@!.*$" contains=nexiumTodo,@Spell
syn match nexiumDocComment "//!.*$" contains=nexiumTodo,@Spell

" ---------------------------------------------------------------- keywords
syn keyword nexiumKeyword fn let var const struct enum record ref class trait impl
syn keyword nexiumKeyword pub import comptime unsafe type distinct where into
syn keyword nexiumKeyword artifact test export as in dyn weak extern derive layout own
syn match nexiumKeyword /\<bench\>\ze\s\+"/
syn keyword nexiumConditional if else match
syn keyword nexiumRepeat for while step parallel
syn keyword nexiumStatement break continue return defer errdefer using unreachable
syn keyword nexiumException try catch orelse
syn keyword nexiumOperatorWord and or
syn keyword nexiumConstant true false null undefined
syn keyword nexiumTodo TODO FIXME XXX NOTE contained

" ---------------------------------------------------------------- types
syn keyword nexiumPrimitive i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 isize usize
syn keyword nexiumPrimitive f32 f64 bool char void never Self
syn keyword nexiumCollection List String Map
syn match nexiumType "\<[A-Z][A-Za-z0-9_]*\>"

" ---------------------------------------------------------------- effects and intrinsics
syn match nexiumEffect "!\%(allocates\|refcounts\|blocks\|shared_mutable\|nondeterministic\|panics\|ffi\|unbounded_stack\)\>"
syn match nexiumIntrinsic "@\%(typeName\|sizeOf\|truncate\|errorName\|embedFile\|weak\|refCount\|cImport\|cstr\)\>"
syn match nexiumNamespace "\<\%(math\|io\|os\|process\|time\|random\|mem\|slice\|utf8\|ascii\|fmt\|context\|alloc\)\.\ze[a-zA-Z_]"
syn match nexiumErrorKeyword "\<error\>" contained
syn match nexiumError "\<error\.[A-Za-z_][A-Za-z0-9_]*\>" contains=nexiumErrorKeyword
syn match nexiumVariant "\%([A-Za-z0-9_)\]]\)\@<!\.[A-Z][A-Za-z0-9_]*\>"
syn match nexiumLabel "\<[a-z_][A-Za-z0-9_]*:\ze\s*\%({\|for\|while\)"
syn match nexiumLabel ":[a-z_][A-Za-z0-9_]*\>"

" ---------------------------------------------------------------- literals
syn match nexiumEscape "\\\%(n\|r\|t\|0\|\\\|\"\|'\|x[0-9A-Fa-f]\{2}\|u{[0-9A-Fa-f]\+}\)" contained
syn match nexiumPlaceholder "{[^{}]*}\|{{\|}}" contained
syn region nexiumString start=+"+ skip=+\\\\\|\\"+ end=+"+ contains=nexiumEscape,nexiumPlaceholder
syn region nexiumRawString start=+r"+ end=+"+
syn region nexiumBytes start=+b"+ skip=+\\\\\|\\"+ end=+"+ contains=nexiumEscape
syn match nexiumChar "'\%(\\\%(n\|r\|t\|0\|\\\|\"\|'\|x[0-9A-Fa-f]\{2}\|u{[0-9A-Fa-f]\+}\)\|[^'\\]\)'"
syn match nexiumNumber "\<0[xX][0-9A-Fa-f_]\+\%(\.[0-9A-Fa-f_]\+\)\?\%([pP][+-]\?[0-9_]\+\)\?\>"
syn match nexiumNumber "\<0[oO][0-7_]\+\>"
syn match nexiumNumber "\<0[bB][01_]\+\>"
syn match nexiumNumber "\<[0-9][0-9_]*\%(\.[0-9][0-9_]*\)\?\%([eE][+-]\?[0-9_]\+\)\?\>"

" ---------------------------------------------------------------- binary patterns
syn match nexiumBinaryModifier "/\%(big\|little\|native\|signed\|unsigned\|float\|utf8\|bytes\)\%(-\%(big\|little\|native\|signed\|unsigned\|float\|utf8\|bytes\)\)*\>" contained
syn keyword nexiumBinaryModifier bytes contained
syn region nexiumBinary matchgroup=nexiumBinaryDelim start="<<\%([=<]\)\@!" end=">>" contains=nexiumBinaryModifier,nexiumNumber,nexiumString,nexiumChar,nexiumBytes,nexiumIdent
syn match nexiumIdent "\<[a-z_][A-Za-z0-9_]*\>" contained

" ---------------------------------------------------------------- declarations and calls
syn match nexiumFunction "\<fn\s\+\zs[A-Za-z_][A-Za-z0-9_]*"
syn match nexiumCall "\<[a-z_][A-Za-z0-9_]*\ze("
syn match nexiumBuiltin "\<\%(println\|print\|eprintln\|format\|expect\|expect_eq\|panic\)\ze("

hi def link nexiumComment Comment
hi def link nexiumDocComment SpecialComment
hi def link nexiumTodo Todo
hi def link nexiumKeyword Keyword
hi def link nexiumConditional Conditional
hi def link nexiumRepeat Repeat
hi def link nexiumStatement Statement
hi def link nexiumException Exception
hi def link nexiumOperatorWord Operator
hi def link nexiumConstant Constant
hi def link nexiumPrimitive Type
hi def link nexiumCollection Type
hi def link nexiumType Type
hi def link nexiumEffect PreProc
hi def link nexiumIntrinsic Macro
hi def link nexiumNamespace Identifier
hi def link nexiumErrorKeyword Keyword
hi def link nexiumError Constant
hi def link nexiumVariant Constant
hi def link nexiumLabel Label
hi def link nexiumEscape SpecialChar
hi def link nexiumPlaceholder Special
hi def link nexiumString String
hi def link nexiumRawString String
hi def link nexiumBytes String
hi def link nexiumChar Character
hi def link nexiumNumber Number
hi def link nexiumBinaryDelim Delimiter
hi def link nexiumBinaryModifier StorageClass
hi def link nexiumFunction Function
hi def link nexiumCall Function
hi def link nexiumBuiltin Function
hi def link nexiumOperator Operator

let b:current_syntax = "nexium"
