(function_item
  "fn" @context
  name: (identifier) @name) @item

(function_signature
  "fn" @context
  name: (identifier) @name) @item

(extern_function
  "fn" @context
  name: (identifier) @name) @item

(struct_item
  "struct" @context
  name: (identifier) @name) @item

(struct_item
  "record" @context
  name: (identifier) @name) @item

(struct_item
  "ref" @context
  "class" @context
  name: (identifier) @name) @item

(enum_item
  "enum" @context
  name: (identifier) @name) @item

(trait_item
  "trait" @context
  name: (identifier) @name) @item

(impl_item
  "impl" @context
  type: (_) @name) @item

(type_alias
  "type" @context
  name: (identifier) @name) @item

(error_set
  "error" @context
  name: (identifier) @name) @item

(const_item
  "const" @context
  name: (identifier) @name) @item

(test_item
  "test" @context
  name: (string_literal) @name) @item

(bench_item
  "bench" @context
  name: (string_literal) @name) @item
