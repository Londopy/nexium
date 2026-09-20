; Nexium indentation for nvim-treesitter
[
  (block)
  (field_list)
  (arguments)
  (parameters)
  (struct_literal)
  (array_literal)
  (match_expression)
  (match_arm)
  (parenthesized_expression)
] @indent.begin

[
  "}"
  ")"
  "]"
] @indent.end

[
  "}"
  ")"
  "]"
] @indent.branch

(comment) @indent.auto
(string_literal) @indent.auto
