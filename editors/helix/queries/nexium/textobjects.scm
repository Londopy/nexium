; Nexium text objects for Helix (mf, mif, mac, ...)
(function_item
  body: (_) @function.inside) @function.around
(struct_item
  body: (field_list) @class.inside) @class.around
(enum_item) @class.around
(trait_item) @class.around
(impl_item) @class.around
(parameters
  (parameter) @parameter.inside)
(arguments
  (_) @parameter.inside)
(comment) @comment.inside
(comment)+ @comment.around
(doc_comment) @comment.inside
(doc_comment)+ @comment.around
(test_item) @test.around
