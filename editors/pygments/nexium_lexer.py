"""A Pygments lexer for Nexium.

Shipped inside the `nexium-lang` wheel as `nexium_lang.pygments` with a
`pygments.lexers` entry point, so `pip install nexium-lang` makes ``nexium``
(aliases ``nx``) a language Pygments knows: Sphinx, MkDocs, Jupyter, the
``pygmentize`` command and every tool built on Pygments highlight ``.nx``
files and ``nexium`` code blocks. The token rules follow the lexer of the
compiler (self/lexer.nx); the keyword list is its KEYWORDS.

    pygmentize -l nexium examples/hello.nx
"""

from pygments.lexer import RegexLexer, words, bygroups
from pygments.token import (Comment, Keyword, Name, Number, Operator, Punctuation, String, Text, Whitespace)

__all__ = ["NexiumLexer"]

KEYWORDS = (
    "fn", "let", "var", "const", "struct", "enum", "record", "ref", "class", "trait", "impl", "pub", "import",
    "return", "if", "else", "for", "while", "match", "break", "continue", "try", "catch", "defer", "errdefer",
    "comptime", "unsafe", "error", "type", "distinct", "where", "into", "artifact", "test", "bench", "export",
    "unreachable", "as", "orelse", "in", "dyn", "weak", "parallel", "extern", "using", "own", "step", "derive",
    "layout",
)
CONSTANTS = ("true", "false", "null", "undefined")
TYPES = (
    "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "isize", "usize", "f32", "f64",
    "bool", "char", "void", "never", "String", "List", "Map", "Self",
)
EFFECTS = ("allocates", "refcounts", "blocks", "shared_mutable", "nondeterministic", "panics", "ffi", "unbounded_stack")


class NexiumLexer(RegexLexer):
    name = "Nexium"
    url = "https://londopy.github.io/nexium/"
    aliases = ["nexium", "nx"]
    filenames = ["*.nx"]
    mimetypes = ["text/x-nexium"]
    version_added = "1.2"

    tokens = {
        "root": [
            (r"\s+", Whitespace),
            (r"//[!/].*?$", Comment.Special),
            (r"//.*?$", Comment.Single),
            # binary patterns: `<<len:16/big, rest:bytes>>`
            (r"<<|>>", Punctuation),
            # strings, byte strings, raw strings, chars
            (r'b"', String, "bytestring"),
            (r'r"[^"]*"', String),
            (r'"', String, "string"),
            (r"'(\\.|[^\\'])'", String.Char),
            # numbers: hex, binary, octal, floats, integers with underscores
            (r"0[xX][0-9a-fA-F_]+", Number.Hex),
            (r"0[bB][01_]+", Number.Bin),
            (r"0[oO][0-7_]+", Number.Oct),
            (r"\d[\d_]*\.\d[\d_]*([eE][+-]?\d+)?", Number.Float),
            (r"\d[\d_]*[eE][+-]?\d+", Number.Float),
            (r"\d[\d_]*", Number.Integer),
            # builtins and effect bounds: `@cImport`, `!allocates`
            (r"@[A-Za-z_]\w*", Name.Builtin),
            (r"@", Operator),
            (r"(!)(" + "|".join(EFFECTS) + r")\b", bygroups(Operator, Name.Decorator)),
            # declarations
            (r"(fn)(\s+)([A-Za-z_]\w*)", bygroups(Keyword, Whitespace, Name.Function)),
            (r"(struct|enum|record|trait|error|class)(\s+)([A-Za-z_]\w*)", bygroups(Keyword, Whitespace, Name.Class)),
            (r"(import)(\s+)([\w.]+)", bygroups(Keyword.Namespace, Whitespace, Name.Namespace)),
            (words(KEYWORDS, prefix=r"\b", suffix=r"\b"), Keyword),
            (words(CONSTANTS, prefix=r"\b", suffix=r"\b"), Keyword.Constant),
            (words(TYPES, prefix=r"\b", suffix=r"\b"), Keyword.Type),
            # a call or a type: `name(` and `Name`
            (r"[A-Za-z_]\w*(?=\s*\()", Name.Function),
            (r"[A-Z][A-Za-z0-9_]*", Name.Class),
            (r"[a-z_]\w*", Name),
            # operators and punctuation
            (r"\.\.=|\.\.|\?\.|\.\?|\.\*|->|=>|\|>|[+\-*]%|[+\-*]\||[-+*/%&|^~<>=!]=?|\.{|[?!]", Operator),
            (r"[()\[\]{},;:.$#]", Punctuation),
        ],
        "string": [
            (r'[^"\\{}]+', String),
            (r"\\.", String.Escape),
            (r"\{\{|\}\}", String.Escape),
            (r"\{[^}\"]*\}", String.Interpol),
            (r"[{}]", String),
            (r'"', String, "#pop"),
        ],
        "bytestring": [
            (r'[^"\\]+', String),
            (r"\\.", String.Escape),
            (r'"', String, "#pop"),
        ],
    }
