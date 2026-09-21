/**
 * @file Tree-sitter grammar for Nexium
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const PREC = {
  orelse: 1,
  or: 2,
  and: 3,
  compare: 4,
  bitor: 5,
  bitxor: 6,
  bitand: 7,
  shift: 8,
  add: 9,
  mul: 10,
  cast: 11,
  unary: 12,
  postfix: 13,
  pipe: 0,
};

const sep = (rule) => optional(seq(rule, repeat(seq(',', rule)), optional(',')));

module.exports = grammar({
  name: 'nexium',

  extras: ($) => [/\s/, $.comment, $.doc_comment],

  word: ($) => $.identifier,

  conflicts: ($) => [
    [$.label, $._primary_expression],
    [$.array_literal, $.slice_type],
    [$.primary_type, $.error_union_type],
    [$.type, $.error_union_type],
    [$.struct_literal, $._primary_expression],
  ],

  supertypes: ($) => [$.item, $.statement, $.expression, $.type, $.pattern],

  rules: {
    source_file: ($) => repeat($.item),

    comment: ($) => token(seq('//', /[^/\n][^\n]*|/)),
    doc_comment: ($) => token(seq('///', /[^\n]*/)),

    // ------------------------------------------------------------ items
    item: ($) =>
      choice(
        $.function_item,
        $.extern_function,
        $.struct_item,
        $.enum_item,
        $.trait_item,
        $.impl_item,
        $.type_alias,
        $.error_set,
        $.const_item,
        $.global_item,
        $.import_item,
        $.test_item,
        $.artifact_item,
      ),

    visibility: ($) => 'pub',

    function_item: ($) =>
      seq(
        optional($.visibility),
        'fn',
        field('name', $.identifier),
        field('parameters', $.parameters),
        optional(seq('->', field('return_type', $.type))),
        repeat($.effect),
        optional($.export_clause),
        field('body', $.block),
      ),

    extern_function: ($) =>
      seq(optional($.visibility), 'extern', 'fn', field('name', $.identifier), field('parameters', $.parameters), optional(seq('->', field('return_type', $.type)))),

    parameters: ($) => seq('(', sep($.parameter), ')'),

    parameter: ($) =>
      choice(
        seq('comptime', field('name', $.identifier), ':', field('type', $.type), optional($.where_clause)),
        seq(optional('own'), field('name', $.identifier), ':', field('type', $.type), optional(seq('=', $.expression))),
      ),

    where_clause: ($) => seq('where', $.identifier, ':', $.type),

    effect: ($) => prec.dynamic(1, seq('!', $.identifier)),

    export_clause: ($) => seq('export', '(', $.identifier, ')'),

    struct_item: ($) =>
      seq(
        optional($.visibility),
        choice('struct', 'record', seq('ref', 'class')),
        field('name', $.identifier),
        optional($.type_parameters),
        repeat($.attribute),
        field('body', $.field_list),
      ),

    type_parameters: ($) => prec(2, seq('(', sep($.identifier), ')')),

    attribute: ($) => seq(field('name', $.identifier), '(', sep($.identifier), ')'),

    field_list: ($) => seq('{', repeat(seq($.field_declaration, optional(','))), '}'),

    field_declaration: ($) =>
      seq(
        optional($.visibility),
        field('name', $.identifier),
        ':',
        field('type', $.type),
        optional(seq('where', $.expression)),
        optional(seq('=', $.expression)),
      ),

    enum_item: ($) =>
      seq(optional($.visibility), 'enum', field('name', $.identifier), optional($.type_parameters), '{', repeat(seq($.variant, optional(','))), '}'),

    variant: ($) =>
      seq(field('name', $.identifier), optional(choice(seq('(', sep($.type), ')'), $.field_list))),

    trait_item: ($) =>
      seq(optional($.visibility), 'trait', field('name', $.identifier), '{', repeat(choice($.function_item, $.function_signature)), '}'),

    function_signature: ($) =>
      seq('fn', field('name', $.identifier), field('parameters', $.parameters), optional(seq('->', field('return_type', $.type))), repeat($.effect)),

    impl_item: ($) =>
      seq(
        'impl',
        optional($.type_parameters),
        field('type', $.type),
        optional(seq('for', field('target', $.type))),
        '{',
        repeat($.function_item),
        '}',
      ),

    type_alias: ($) => seq(optional($.visibility), 'type', field('name', $.identifier), '=', field('type', $.type)),

    error_set: ($) => seq(optional($.visibility), 'error', field('name', $.identifier), '{', sep($.identifier), '}'),

    const_item: ($) =>
      seq(optional($.visibility), 'const', field('name', $.identifier), optional(seq(':', field('type', $.type))), '=', field('value', $.expression)),

    global_item: ($) =>
      seq(optional($.visibility), 'var', field('name', $.identifier), optional(seq(':', field('type', $.type))), '=', field('value', $.expression)),

    import_item: ($) =>
      seq(
        'import',
        field('path', $.module_path),
        optional(choice(seq('as', field('alias', $.identifier)), seq('{', sep($.identifier), '}'))),
      ),

    module_path: ($) => seq($.identifier, repeat(seq('.', $.identifier))),

    test_item: ($) => seq(optional('comptime'), 'test', field('name', $.string_literal), field('body', $.block)),

    artifact_item: ($) => seq('artifact', field('kind', $.identifier), '{', repeat(seq($.identifier, '=', $.expression, optional(','))), '}'),

    // ------------------------------------------------------------ statements
    block: ($) => seq('{', repeat(seq($.statement, optional(';'))), '}'),

    statement: ($) =>
      choice(
        $.let_statement,
        $.assignment_statement,
        $.while_statement,
        $.for_statement,
        $.return_statement,
        $.break_statement,
        $.continue_statement,
        $.defer_statement,
        $.using_statement,
        $.unsafe_block,
        $.expression_statement,
        $.function_item,
        $.struct_item,
        $.enum_item,
        $.const_item,
      ),

    let_statement: ($) =>
      prec.right(seq(
        choice('let', 'var'),
        field('pattern', choice($.identifier, $.tuple_binding)),
        optional(seq(':', field('type', $.type))),
        optional(seq('=', field('value', $.expression))),
      )),

    tuple_binding: ($) => seq('(', sep(choice($.identifier, '_')), ')'),

    assignment_statement: ($) =>
      prec.right(seq(field('left', choice($.expression, '_')), field('operator', choice('=', '+=', '-=', '*=', '/=', '%=', '&=', '|=', '^=', '<<=', '>>=', '+%=', '-%=', '*%=', '+|=', '-|=', '*|=')), field('right', $.expression))),

    capture: ($) => seq('|', sep(choice($.identifier, '_')), '|'),

    while_statement: ($) =>
      seq(optional($.label), 'while', field('condition', $.expression), field('body', $.block), optional(seq('else', $.block))),

    // `for x, i in items { }`, `for i in lo..hi step s { }`, `for parallel x in items { }`
    for_statement: ($) =>
      seq(
        optional($.label),
        'for',
        optional('parallel'),
        field('bindings', $.for_bindings),
        'in',
        choice(seq($.expression, '..', $.expression, optional(seq('step', $.expression))), seq($.expression, repeat(seq(',', $.expression)))),
        field('body', $.block),
      ),

    for_bindings: ($) => seq(choice($.identifier, '_', $.tuple_binding), repeat(seq(',', choice($.identifier, '_', $.tuple_binding)))),

    // `outer: for (...)` declares a label; `break :outer` refers to it
    label: ($) => seq($.identifier, ':'),
    label_ref: ($) => seq(':', $.identifier),

    return_statement: ($) => prec.right(seq('return', optional($.expression))),
    break_statement: ($) => prec.right(seq('break', optional($.label_ref), optional($.expression))),
    continue_statement: ($) => prec.right(seq('continue', optional($.label_ref))),
    defer_statement: ($) => seq(choice('defer', 'errdefer'), $.statement),
    using_statement: ($) => seq('using', $.expression, $.block),
    unsafe_block: ($) => seq('unsafe', $.block),
    labeled_block: ($) => seq($.label, $.block),
    expression_statement: ($) => prec.right(-2, $.expression),

    // ------------------------------------------------------------ expressions
    expression: ($) =>
      choice(
        $.binary_expression,
        $.unary_expression,
        $.cast_expression,
        $.try_expression,
        $.catch_expression,
        $.orelse_expression,
        $.pipe_expression,
        $._postfix_expression,
        $._primary_expression,
      ),

    _primary_expression: ($) =>
      choice(
        $.identifier,
        $.integer_literal,
        $.float_literal,
        $.string_literal,
        $.bytes_literal,
        $.char_literal,
        $.boolean_literal,
        $.null_literal,
        $.undefined_literal,
        $.error_value,
        $.enum_literal,
        $.struct_literal,
        $.anonymous_literal,
        $.array_literal,
        $.tuple_expression,
        $.parenthesized_expression,
        $.builtin_call,
        $.closure_expression,
        $.if_expression,
        $.match_expression,
        $.block,
        $.labeled_block,
        $.comptime_expression,
        $.unreachable_expression,
        $.address_of,
        $.binary_build,
      ),

    comptime_expression: ($) => prec.right(-1, seq('comptime', $.expression)),
    unreachable_expression: ($) => 'unreachable',

    address_of: ($) => prec(PREC.unary, seq('&', optional('mut'), $.expression)),

    _postfix_expression: ($) => choice($.call_expression, $.field_expression, $.index_expression, $.slice_expression, $.unwrap_expression, $.deref_expression, $.optional_chain_expression),

    call_expression: ($) => prec(PREC.postfix, seq(field('function', $.expression), field('arguments', $.arguments))),
    arguments: ($) => seq('(', sep(choice($.expression, $.type_argument)), ')'),
    // types that cannot be mistaken for expressions may be passed to generic constructors: `List([]u8)`
    type_argument: ($) => choice($.slice_type, $.pointer_type, $.optional_type, $.dyn_type, $.function_type),

    field_expression: ($) => prec(PREC.postfix, seq(field('value', $.expression), '.', field('field', choice($.identifier, $.integer_literal)))),
    index_expression: ($) => prec(PREC.postfix, seq(field('value', $.expression), '[', field('index', $.expression), ']')),
    slice_expression: ($) =>
      prec(PREC.postfix, seq(field('value', $.expression), '[', optional(field('from', $.expression)), '..', optional(field('to', $.expression)), ']')),
    unwrap_expression: ($) => prec(PREC.postfix, seq($.expression, '.?')),
    // `a?.b`: null when `a` is, otherwise the member of the payload, as an optional
    optional_chain_expression: ($) => prec(PREC.postfix, seq(field('value', $.expression), '?.', field('field', choice($.identifier, $.integer_literal)))),
    deref_expression: ($) => prec(PREC.postfix, seq($.expression, '.*')),

    unary_expression: ($) => prec(PREC.unary, seq(field('operator', choice('-', '!', '~')), field('operand', $.expression))),

    cast_expression: ($) => prec.left(PREC.cast, seq($.expression, 'as', $.type)),

    try_expression: ($) => prec.right(PREC.unary, seq('try', $.expression)),

    // the right side of `catch` and `orelse` may leave the function or loop:
    // `x orelse return null`, `ch.recv() orelse break`. A value follows the
    // keyword only on the same line (the keyword token swallows the blanks
    // after it), so a bare `return` at a line end does not take the next
    // statement as its value.
    catch_expression: ($) => prec.left(PREC.orelse, seq($.expression, 'catch', optional($.capture), choice($.expression, $._jump_expression))),
    orelse_expression: ($) => prec.left(PREC.orelse, seq($.expression, 'orelse', choice($.expression, $._jump_expression))),
    _jump_expression: ($) => choice($.return_expression, $.break_expression, $.continue_expression),
    return_expression: ($) => prec.right(choice(seq(alias(token(seq('return', /[ 	]+/)), 'return'), $.expression), 'return')),
    break_expression: ($) =>
      prec.right(choice(seq(alias(token(seq('break', /[ 	]+/)), 'break'), optional($.label_ref), optional($.expression)), 'break')),
    continue_expression: ($) => prec.right(choice(seq(alias(token(seq('continue', /[ 	]+/)), 'continue'), optional($.label_ref)), 'continue')),
    pipe_expression: ($) => prec.left(PREC.pipe, seq($.expression, '|>', $.expression)),

    binary_expression: ($) => {
      const table = [
        [PREC.or, 'or'],
        [PREC.and, 'and'],
        [PREC.compare, choice('==', '!=', '<', '<=', '>', '>=')],
        [PREC.bitor, '|'],
        [PREC.bitxor, '^'],
        [PREC.bitand, '&'],
        [PREC.shift, choice('<<', '>>')],
        [PREC.add, choice('+', '-', '+%', '-%', '+|', '-|')],
        [PREC.mul, choice('*', '/', '%', '*%', '*|')],
      ];
      return choice(
        ...table.map(([p, op]) =>
          prec.left(p, seq(field('left', $.expression), field('operator', op), field('right', $.expression))),
        ),
        prec.left(PREC.compare, seq(field('left', $.expression), field('operator', '..'), field('right', $.expression))),
        prec.left(PREC.compare, seq(field('left', $.expression), field('operator', '..='), field('right', $.expression))),
      );
    },

    // `if c { } else if d { } else { }`; `if let v = opt { }` unwraps an optional
    if_expression: ($) =>
      prec.right(
        seq(
          'if',
          optional(seq('let', field('binding', $.identifier), '=')),
          field('condition', $.expression),
          field('then', $.block),
          optional(seq('else', field('else', choice($.block, $.if_expression)))),
        ),
      ),

    // a match arm body is an expression or one jump statement: `_ => return v`
    branch: ($) => prec.right(-2, choice($.expression, $.assignment_statement, $.return_statement, $.break_statement, $.continue_statement)),

    match_expression: ($) => seq('match', field('value', $.expression), '{', repeat(seq($.match_arm, optional(','))), '}'),

    match_arm: ($) => seq(field('pattern', $.pattern), optional(seq('if', $.expression)), '=>', field('body', $.branch)),

    closure_expression: ($) =>
      prec.right(-1, seq(
        '|',
        optional($.captures),
        sep($.parameter),
        '|',
        optional(seq('->', $.type)),
        field('body', $.expression),
      )),

    captures: ($) => seq('[', sep(seq(optional('&'), optional('mut'), $.identifier)), ']'),

    builtin_call: ($) => seq('@', field('name', $.identifier), $.arguments),

    // `<<4:4, 1500:16/big, "ab">> into buf[..]` builds bytes into a buffer
    binary_build: ($) => prec.dynamic(3, prec.right(seq('<<', sep($.build_segment), '>>', 'into', $.expression))),
    build_segment: ($) => prec.right(PREC.shift + 1, seq($.expression, optional(seq(':', $.segment_size, repeat(seq('/', $.segment_modifier)))))),

    error_value: ($) => seq('error', '.', $.identifier),

    enum_literal: ($) => prec.right(PREC.postfix, seq('.', field('variant', $.identifier), optional($.arguments))),

    struct_literal: ($) =>
      prec.dynamic(2, seq(field('type', choice($.identifier, seq($.identifier, '.', $.identifier), seq($.identifier, $.type_arguments))), '{', sep($.field_initializer), '}')),

    anonymous_literal: ($) => seq('.{', sep(choice($.field_initializer, $.expression)), '}'),

    field_initializer: ($) => seq('.', field('name', $.identifier), '=', field('value', $.expression)),

    array_literal: ($) => seq('[', sep($.expression), ']'),
    tuple_expression: ($) => seq('(', $.expression, ',', sep($.expression), ')'),
    parenthesized_expression: ($) => seq('(', $.expression, ')'),

    // ------------------------------------------------------------ patterns
    pattern: ($) =>
      choice(
        $.wildcard_pattern,
        $.identifier,
        $.literal_pattern,
        $.range_pattern,
        $.enum_pattern,
        $.error_pattern,
        $.null_literal,
        $.tuple_pattern,
        $.slice_pattern,
        $.at_pattern,
        $.or_pattern,
        $.binary_pattern,
        $.else_pattern,
      ),

    wildcard_pattern: ($) => '_',
    else_pattern: ($) => 'else',
    literal_pattern: ($) => choice($.integer_literal, seq('-', $.integer_literal), $.float_literal, $.string_literal, $.char_literal, $.boolean_literal),
    range_pattern: ($) => prec.left(seq(choice($.integer_literal, $.char_literal), choice('..', '..='), choice($.integer_literal, $.char_literal))),
    enum_pattern: ($) =>
      seq('.', field('variant', $.identifier), optional(choice(seq('(', sep($.pattern), ')'), seq('{', sep(seq($.identifier, ':', $.pattern)), '}')))),
    error_pattern: ($) => seq('error', '.', $.identifier),
    tuple_pattern: ($) => seq('(', sep($.pattern), ')'),
    // `[a, b]`, `[first, rest..]`, `[.., last]`
    slice_pattern: ($) => seq('[', sep(choice($.pattern, $.rest_pattern)), ']'),
    rest_pattern: ($) => seq(optional($.identifier), '..'),
    // `whole @ pattern`
    at_pattern: ($) => prec.right(seq(field('name', $.identifier), '@', field('pattern', $.pattern))),
    or_pattern: ($) => prec.left(seq($.pattern, '|', $.pattern)),
    binary_pattern: ($) => seq('<<', sep($.binary_segment), '>>'),
    binary_segment: ($) => seq(choice($.identifier, $.integer_literal, $.char_literal, $.string_literal, '_'), optional(seq(':', $.segment_size)), repeat(seq('/', $.segment_modifier))),
    // `/little-signed`: modifiers joined with `-`
    segment_modifier: ($) => /[A-Za-z_][A-Za-z0-9_]*(-[A-Za-z_][A-Za-z0-9_]*)*/,
    segment_size: ($) => seq(choice($.identifier, $.integer_literal), optional(seq('*', choice($.identifier, $.integer_literal)))),

    // ------------------------------------------------------------ types
    type: ($) =>
      choice(
        $.primary_type,
        $.path_type,
        $.generic_type,
        $.optional_type,
        $.error_union_type,
        $.pointer_type,
        $.slice_type,
        $.array_type,
        $.function_type,
        $.tuple_type,
        $.dyn_type,
        $.weak_type,
        $.distinct_type,
      ),

    primary_type: ($) => $.identifier,
    path_type: ($) => prec.left(1, seq($.identifier, repeat1(seq('.', $.identifier)))),
    generic_type: ($) => prec(1, seq(choice($.identifier, $.path_type), $.type_arguments)),
    type_arguments: ($) => seq('(', sep($.type), ')'),
    optional_type: ($) => prec.right(seq('?', $.type)),
    error_union_type: ($) => prec.right(seq(optional(choice($.identifier, $.path_type)), '!', $.type)),
    pointer_type: ($) => prec.right(seq('*', optional('mut'), $.type)),
    slice_type: ($) => prec.right(seq('[', ']', optional('mut'), $.type)),
    array_type: ($) => prec.right(seq('[', choice($.integer_literal, $.module_path), ']', $.type)),
    function_type: ($) => prec.right(seq('fn', '(', sep($.type), ')', optional(seq('->', $.type)), repeat($.effect))),
    tuple_type: ($) => seq('(', $.type, ',', sep($.type), ')'),
    dyn_type: ($) => prec.right(seq('dyn', $.type)),
    weak_type: ($) => prec.right(seq('weak', $.type)),
    distinct_type: ($) => prec.right(seq('distinct', $.type)),

    // ------------------------------------------------------------ tokens
    identifier: ($) => /[A-Za-z_][A-Za-z0-9_]*/,
    integer_literal: ($) => token(choice(/0[xX][0-9a-fA-F_]+/, /0[oO][0-7_]+/, /0[bB][01_]+/, /[0-9][0-9_]*/)),
    float_literal: ($) => token(choice(/[0-9][0-9_]*\.[0-9][0-9_]*([eE][+-]?[0-9]+)?/, /[0-9][0-9_]*[eE][+-]?[0-9]+/, /0[xX][0-9a-fA-F]+\.[0-9a-fA-F]*[pP][+-]?[0-9]+/)),
    string_literal: ($) => token(seq('"', repeat(choice(/[^"\\\n]/, /\\./)), '"')),
    bytes_literal: ($) => token(seq('b"', repeat(choice(/[^"\\\n]/, /\\./)), '"')),
    char_literal: ($) => token(seq("'", choice(/[^'\\\n]/, /\\./, /\\x[0-9a-fA-F]{2}/, /\\u\{[0-9a-fA-F]+\}/), "'")),
    boolean_literal: ($) => choice('true', 'false'),
    null_literal: ($) => 'null',
    undefined_literal: ($) => 'undefined',
  },
});
