; Nexium highlighting for Helix. Same shape as the nvim-treesitter queries in
; ../../../tree-sitter-nexium/queries, with Helix's scope names.

; ---------------------------------------------------------------- comments
(comment) @comment.line
(doc_comment) @comment.block.documentation

; ---------------------------------------------------------------- keywords
"fn" @keyword.function
[
  "struct" "enum" "record" "ref" "class" "trait" "impl" "type" "error" "artifact"
] @keyword.storage.type
[
  "let" "var" "const" "own" "distinct" "dyn" "weak" "extern" "export" "comptime"
] @keyword.storage.modifier
["import" "using"] @keyword.control.import
["if" "else" "match"] @keyword.control.conditional
["for" "in" "while" "step" "parallel"] @keyword.control.repeat
["return" "break" "continue"] @keyword.control.return
["try" "catch" "orelse" "defer" "errdefer"] @keyword.control.exception
["test" "bench" "where" "unsafe"] @keyword
["as" "into"] @keyword.operator
(visibility) @keyword.storage.modifier

(for_bindings (identifier) @variable)
(if_expression binding: (identifier) @variable)

; ---------------------------------------------------------------- literals
(integer_literal) @constant.numeric.integer
(float_literal) @constant.numeric.float
(string_literal) @string
(bytes_literal) @string
(char_literal) @constant.character
(boolean_literal) @constant.builtin.boolean
(null_literal) @constant.builtin
(undefined_literal) @constant.builtin
(unreachable_expression) @keyword.control.exception

; ---------------------------------------------------------------- definitions
(function_item name: (identifier) @function)
(function_signature name: (identifier) @function)
(extern_function name: (identifier) @function)
(struct_item name: (identifier) @type)
(enum_item name: (identifier) @type)
(trait_item name: (identifier) @type)
(type_alias name: (identifier) @type)
(error_set name: (identifier) @type)
(const_item name: (identifier) @constant)
(global_item name: (identifier) @variable)
(variant name: (identifier) @type.enum.variant)
(field_declaration name: (identifier) @variable.other.member)
(parameter name: (identifier) @variable.parameter)
(type_parameters (identifier) @type.parameter)
(attribute name: (identifier) @attribute)
(effect (identifier) @attribute)
(label (identifier) @label)
(label_ref (identifier) @label)
(test_item name: (string_literal) @string.special)

; ---------------------------------------------------------------- uses
(call_expression function: (identifier) @function)
(call_expression function: (field_expression field: (identifier) @function.method))
(builtin_call name: (identifier) @function.builtin)
(field_expression field: (identifier) @variable.other.member)
(field_initializer name: (identifier) @variable.other.member)
(enum_literal variant: (identifier) @type.enum.variant)
(enum_pattern variant: (identifier) @type.enum.variant)
(error_value (identifier) @constant)
(error_pattern (identifier) @constant)
(struct_literal type: (identifier) @type)
(import_item path: (module_path (identifier) @namespace))

; ---------------------------------------------------------------- types
(primary_type (identifier) @type)
(path_type (identifier) @type)
(generic_type (identifier) @type)
(error_union_type (identifier) @type)
((primary_type (identifier) @type.builtin)
  (#match? @type.builtin "^(i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize|f32|f64|bool|char|void|never|type|String|List|Map|Self|error)$"))

; ---------------------------------------------------------------- operators and punctuation
[
  "+" "-" "*" "/" "%" "+%" "-%" "*%" "+|" "-|" "*|"
  "==" "!=" "<" "<=" ">" ">=" "and" "or" "!" "~" "&" "|" "^" "<<" ">>"
  "=" "+=" "-=" "*=" "/=" "%=" "&=" "|=" "^=" "<<=" ">>=" "+%=" "-%=" "*%=" "+|=" "-|=" "*|="
  ".." "..=" "|>" "->" "=>" "?" ".?" "?." ".*" "@"
] @operator

["(" ")" "[" "]" "{" "}" ".{" "<<" ">>"] @punctuation.bracket
["," ":" "." ";"] @punctuation.delimiter

; ---------------------------------------------------------------- fallbacks
((identifier) @variable.builtin
  (#match? @variable.builtin "^(self|Self)$"))
((identifier) @namespace
  (#match? @namespace "^(math|io|os|time|random|mem|process|net|thread|sync|std)$"))
(identifier) @variable
