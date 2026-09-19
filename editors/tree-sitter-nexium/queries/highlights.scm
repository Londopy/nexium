; Nexium highlighting for tree-sitter (Neovim, Helix, Zed)

; ---------------------------------------------------------------- comments
(comment) @comment
(doc_comment) @comment.documentation

; ---------------------------------------------------------------- keywords
[
  "fn" "let" "var" "const" "struct" "enum" "record" "ref" "class" "trait" "impl"
  "type" "error" "test" "artifact" "import" "extern" "export" "comptime" "own"
  "distinct" "dyn" "weak" "where" "using" "unsafe" "defer" "errdefer"
] @keyword

[
  "if" "else" "for" "in" "while" "match" "break" "continue" "return" "step" "parallel"
] @keyword.control

(for_bindings (identifier) @variable)
(if_expression binding: (identifier) @variable)

[
  "try" "catch" "orelse" "into"
] @keyword.control.exception

"as" @keyword.operator
(visibility) @keyword.modifier

; ---------------------------------------------------------------- literals
(integer_literal) @number
(float_literal) @number.float
(string_literal) @string
(bytes_literal) @string
(char_literal) @character
(boolean_literal) @constant.builtin
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
(variant name: (identifier) @constructor)
(field_declaration name: (identifier) @property)
(parameter name: (identifier) @variable.parameter)
(type_parameters (identifier) @type)
(attribute name: (identifier) @attribute)
(effect (identifier) @attribute)
(label (identifier) @label)
(label_ref (identifier) @label)
(test_item name: (string_literal) @string.special)

; ---------------------------------------------------------------- uses
(call_expression function: (identifier) @function.call)
(call_expression function: (field_expression field: (identifier) @function.method))
(builtin_call name: (identifier) @function.builtin)
(field_expression field: (identifier) @property)
(field_initializer name: (identifier) @property)
(enum_literal variant: (identifier) @constructor)
(enum_pattern variant: (identifier) @constructor)
(error_value (identifier) @constant)
(error_pattern (identifier) @constant)
(struct_literal type: (identifier) @type)
(import_item path: (module_path (identifier) @module))

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
  ".." "..=" "|>" "->" "=>" "?" ".?" ".*" "@"
] @operator

["(" ")" "[" "]" "{" "}" ".{" "<<" ">>"] @punctuation.bracket
["," ":" "." ";"] @punctuation.delimiter

; ---------------------------------------------------------------- fallbacks
((identifier) @variable.builtin
  (#match? @variable.builtin "^(self|Self)$"))
((identifier) @module
  (#match? @module "^(math|io|os|time|random|mem|process|net|thread|sync|std)$"))
(identifier) @variable
