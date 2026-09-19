//! Lexical analysis for Nexium (.nx).
//!
//! Rules from the specification, section 4:
//!   - `//` comments to end of line, `///` doc comments attach to the next declaration
//!   - identifiers are ASCII letters, digits, underscore; unicode identifiers are rejected
//!   - integer literals: 42, 0xFF, 0o755, 0b1010_1100, underscores anywhere
//!   - floats: 3.14, 1e-9, 0x1.8p3
//!   - strings "..." with escapes, raw strings r"...", byte strings b"...", chars 'a'
//!   - newlines terminate statements; they are suppressed inside ( ) and [ ]

use crate::diag::{Diag, Span};

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Ident(String),
    Int(u128),
    Float(f64),
    Str(Vec<u8>),   // UTF-8 text literal, already unescaped
    Bytes(Vec<u8>), // b"..." not validated
    Char(u32),
    Doc(String),     // /// doc comment line
    Comment(String), // // comment (only when the lexer keeps comments)
    Newline,
    Eof,
    // punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semi,
    Dot,
    DotStar,
    DotQuestion,
    DotDot,
    DotLBrace,
    Arrow,
    FatArrow,
    Question,
    Bang,
    Tilde,
    At,
    Pipe,
    PipeGt,
    Hash,
    Dollar,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    PlusPct,
    MinusPct,
    StarPct, // wrapping  +% -% *%
    PlusPipe,
    MinusPipe,
    StarPipe, // saturating +| -| *|
    Amp,
    Caret,
    Shl,
    Shr,
    Eq,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    AmpEq,
    PipeEq,
    CaretEq,
    ShlEq,
    ShrEq,
    PlusPctEq,
    MinusPctEq,
    StarPctEq,
    LtLt,
    GtGt, // binary pattern delimiters (same chars as Shl/Shr; parser decides)
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
}

pub const KEYWORDS: &[&str] = &[
    "fn",
    "let",
    "var",
    "const",
    "struct",
    "enum",
    "record",
    "ref",
    "class",
    "trait",
    "impl",
    "pub",
    "import",
    "return",
    "if",
    "else",
    "for",
    "while",
    "match",
    "break",
    "continue",
    "try",
    "catch",
    "defer",
    "errdefer",
    "comptime",
    "unsafe",
    "error",
    "true",
    "false",
    "null",
    "undefined",
    "and",
    "or",
    "type",
    "distinct",
    "where",
    "into",
    "artifact",
    "test",
    "export",
    "unreachable",
    "as",
    "orelse",
    "in",
    "dyn",
    "weak",
    "parallel",
    "extern",
    "using",
];

pub fn is_keyword(s: &str) -> bool {
    KEYWORDS.contains(&s)
}

pub struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
    file: u32,
    depth: Vec<u8>, // bracket stack: b'(' b'[' b'{'
    toks: Vec<Token>,
    pub diags: Vec<Diag>,
    keep_comments: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str, file: u32) -> Self {
        Lexer { src: src.as_bytes(), pos: 0, file, depth: Vec::new(), toks: Vec::new(), diags: Vec::new(), keep_comments: false }
    }

    /// Keep `//` comments as tokens (used by the formatter).
    pub fn with_comments(mut self) -> Self {
        self.keep_comments = true;
        self
    }

    fn peek(&self) -> u8 {
        *self.src.get(self.pos).unwrap_or(&0)
    }
    fn peek_at(&self, n: usize) -> u8 {
        *self.src.get(self.pos + n).unwrap_or(&0)
    }
    fn span(&self, start: usize) -> Span {
        Span { file: self.file, start: start as u32, end: self.pos as u32 }
    }
    fn push(&mut self, tok: Tok, start: usize) {
        let span = self.span(start);
        self.toks.push(Token { tok, span });
    }
    fn err(&mut self, start: usize, msg: impl Into<String>) {
        let span = self.span(start);
        self.diags.push(Diag::error(span, msg));
    }

    fn newlines_suppressed(&self) -> bool {
        matches!(self.depth.last(), Some(b'(') | Some(b'['))
    }

    pub fn lex(mut self) -> (Vec<Token>, Vec<Diag>) {
        loop {
            let c = self.peek();
            if c == 0 {
                break;
            }
            let start = self.pos;
            match c {
                b' ' | b'\t' | b'\r' => {
                    self.pos += 1;
                }
                b'\n' => {
                    self.pos += 1;
                    if !self.newlines_suppressed() {
                        // collapse runs of newlines
                        if !matches!(self.toks.last().map(|t| &t.tok), Some(Tok::Newline) | None) {
                            self.push(Tok::Newline, start);
                        }
                    }
                }
                b'/' if self.peek_at(1) == b'/' => {
                    let is_doc = self.peek_at(2) == b'/' && self.peek_at(3) != b'/';
                    self.pos += if is_doc { 3 } else { 2 };
                    let s = self.pos;
                    while self.peek() != b'\n' && self.peek() != 0 {
                        self.pos += 1;
                    }
                    if is_doc {
                        let text = String::from_utf8_lossy(&self.src[s..self.pos]).trim().to_string();
                        self.push(Tok::Doc(text), start);
                    } else if self.keep_comments {
                        let text = String::from_utf8_lossy(&self.src[s..self.pos]).trim().to_string();
                        self.push(Tok::Comment(text), start);
                    }
                }
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                    // raw / byte string prefixes
                    if (c == b'r' || c == b'b') && self.peek_at(1) == b'"' {
                        self.pos += 1;
                        self.string(start, c == b'r', c == b'b');
                        continue;
                    }
                    while self.peek().is_ascii_alphanumeric() || self.peek() == b'_' {
                        self.pos += 1;
                    }
                    let s = String::from_utf8_lossy(&self.src[start..self.pos]).to_string();
                    self.push(Tok::Ident(s), start);
                }
                b'0'..=b'9' => self.number(start),
                b'"' => self.string(start, false, false),
                b'\'' => self.char_lit(start),
                _ if c >= 0x80 => {
                    // reject unicode outside strings/comments
                    while self.peek() >= 0x80 {
                        self.pos += 1;
                    }
                    self.err(start, "non-ASCII characters are not permitted outside string literals and comments");
                }
                _ => self.punct(start),
            }
        }
        let start = self.pos;
        if !matches!(self.toks.last().map(|t| &t.tok), Some(Tok::Newline) | None) {
            self.push(Tok::Newline, start);
        }
        self.push(Tok::Eof, start);
        (self.toks, self.diags)
    }

    fn number(&mut self, start: usize) {
        let mut radix = 10u32;
        if self.peek() == b'0' {
            match self.peek_at(1) {
                b'x' | b'X' => {
                    radix = 16;
                    self.pos += 2;
                }
                b'o' | b'O' => {
                    radix = 8;
                    self.pos += 2;
                }
                b'b' | b'B' => {
                    radix = 2;
                    self.pos += 2;
                }
                _ => {}
            }
        }
        let digits_start = self.pos;
        let mut is_float = false;
        loop {
            let c = self.peek();
            if c == b'_' || c.is_ascii_digit() || (radix == 16 && c.is_ascii_hexdigit()) {
                self.pos += 1;
            } else if c == b'.' && radix == 10 && self.peek_at(1).is_ascii_digit() {
                is_float = true;
                self.pos += 1;
            } else if c == b'.' && radix == 16 && self.peek_at(1).is_ascii_hexdigit() {
                is_float = true;
                self.pos += 1;
            } else if (c == b'e' || c == b'E') && radix == 10 {
                is_float = true;
                self.pos += 1;
                if self.peek() == b'+' || self.peek() == b'-' {
                    self.pos += 1;
                }
            } else if (c == b'p' || c == b'P') && radix == 16 {
                is_float = true;
                self.pos += 1;
                if self.peek() == b'+' || self.peek() == b'-' {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
        let text: String = self.src[digits_start..self.pos].iter().filter(|&&b| b != b'_').map(|&b| b as char).collect();
        if is_float {
            let v = if radix == 16 { parse_hex_float(&text) } else { text.parse::<f64>().ok() };
            match v {
                Some(f) => self.push(Tok::Float(f), start),
                None => self.err(start, format!("malformed float literal `{}`", text)),
            }
        } else {
            match u128::from_str_radix(&text, radix) {
                Ok(v) => self.push(Tok::Int(v), start),
                Err(_) => self.err(start, format!("malformed integer literal `{}`", text)),
            }
        }
        if self.peek().is_ascii_alphabetic() {
            let s = self.pos;
            while self.peek().is_ascii_alphanumeric() {
                self.pos += 1;
            }
            self.err(s, "unexpected characters after numeric literal");
        }
    }

    fn escape(&mut self, out: &mut Vec<u8>, start: usize) {
        // called after the backslash was consumed
        let c = self.peek();
        self.pos += 1;
        match c {
            b'n' => out.push(b'\n'),
            b'r' => out.push(b'\r'),
            b't' => out.push(b'\t'),
            b'0' => out.push(0),
            b'\\' => out.push(b'\\'),
            b'"' => out.push(b'"'),
            b'\'' => out.push(b'\''),
            b'x' => {
                let h = &self.src[self.pos..(self.pos + 2).min(self.src.len())];
                if h.len() == 2 && h.iter().all(|b| b.is_ascii_hexdigit()) {
                    let v = u8::from_str_radix(std::str::from_utf8(h).unwrap(), 16).unwrap();
                    out.push(v);
                    self.pos += 2;
                } else {
                    self.err(start, "\\x escape needs two hex digits");
                }
            }
            b'u' => {
                if self.peek() == b'{' {
                    self.pos += 1;
                    let s = self.pos;
                    while self.peek().is_ascii_hexdigit() {
                        self.pos += 1;
                    }
                    let hex = std::str::from_utf8(&self.src[s..self.pos]).unwrap().to_string();
                    if self.peek() == b'}' {
                        self.pos += 1;
                    }
                    match u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                        Some(ch) => {
                            let mut b = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut b).as_bytes());
                        }
                        None => self.err(start, "invalid \\u{...} escape"),
                    }
                } else {
                    self.err(start, "\\u escape needs braces: \\u{1F600}");
                }
            }
            _ => self.err(start, format!("unknown escape `\\{}`", c as char)),
        }
    }

    fn string(&mut self, start: usize, raw: bool, bytes: bool) {
        // self.pos at opening quote
        self.pos += 1;
        let mut out = Vec::new();
        loop {
            let c = self.peek();
            if c == 0 {
                self.err(start, "unterminated string literal");
                break;
            }
            if c == b'"' {
                self.pos += 1;
                break;
            }
            if c == b'\\' && !raw {
                self.pos += 1;
                self.escape(&mut out, start);
                continue;
            }
            if c == b'\n' && !raw {
                self.err(start, "newline in string literal (use \\n or a raw string)");
            }
            out.push(c);
            self.pos += 1;
        }
        if bytes {
            self.push(Tok::Bytes(out), start);
        } else {
            if std::str::from_utf8(&out).is_err() {
                self.err(start, "string literal is not valid UTF-8 (use b\"...\" for bytes)");
            }
            self.push(Tok::Str(out), start);
        }
    }

    fn char_lit(&mut self, start: usize) {
        self.pos += 1;
        let mut out = Vec::new();
        let c = self.peek();
        if c == b'\\' {
            self.pos += 1;
            self.escape(&mut out, start);
        } else if c == 0 || c == b'\n' {
            self.err(start, "unterminated character literal");
            return;
        } else {
            // may be multi-byte UTF-8
            let len = utf8_len(c);
            out.extend_from_slice(&self.src[self.pos..(self.pos + len).min(self.src.len())]);
            self.pos += len;
        }
        if self.peek() != b'\'' {
            self.err(start, "character literal must contain exactly one character");
            while self.peek() != b'\'' && self.peek() != b'\n' && self.peek() != 0 {
                self.pos += 1;
            }
        }
        if self.peek() == b'\'' {
            self.pos += 1;
        }
        let v = match std::str::from_utf8(&out).ok().and_then(|s| s.chars().next()) {
            Some(ch) => ch as u32,
            None => out.first().copied().unwrap_or(0) as u32,
        };
        self.push(Tok::Char(v), start);
    }

    fn punct(&mut self, start: usize) {
        let c = self.peek();
        let c1 = self.peek_at(1);
        let c2 = self.peek_at(2);
        let (tok, len) = match (c, c1, c2) {
            (b'(', _, _) => {
                self.depth.push(b'(');
                (Tok::LParen, 1)
            }
            (b'[', _, _) => {
                self.depth.push(b'[');
                (Tok::LBracket, 1)
            }
            (b'{', _, _) => {
                self.depth.push(b'{');
                (Tok::LBrace, 1)
            }
            (b')', _, _) => {
                self.depth.pop();
                (Tok::RParen, 1)
            }
            (b']', _, _) => {
                self.depth.pop();
                (Tok::RBracket, 1)
            }
            (b'}', _, _) => {
                self.depth.pop();
                (Tok::RBrace, 1)
            }
            (b',', _, _) => (Tok::Comma, 1),
            (b';', _, _) => (Tok::Semi, 1),
            (b':', _, _) => (Tok::Colon, 1),
            (b'.', b'*', _) => (Tok::DotStar, 2),
            (b'.', b'?', _) => (Tok::DotQuestion, 2),
            (b'.', b'.', _) => (Tok::DotDot, 2),
            (b'.', b'{', _) => {
                self.depth.push(b'{');
                (Tok::DotLBrace, 2)
            }
            (b'.', _, _) => (Tok::Dot, 1),
            (b'-', b'>', _) => (Tok::Arrow, 2),
            (b'=', b'>', _) => (Tok::FatArrow, 2),
            (b'=', b'=', _) => (Tok::EqEq, 2),
            (b'=', _, _) => (Tok::Eq, 1),
            (b'!', b'=', _) => (Tok::Ne, 2),
            (b'!', _, _) => (Tok::Bang, 1),
            (b'<', b'<', b'=') => (Tok::ShlEq, 3),
            (b'<', b'<', _) => (Tok::LtLt, 2),
            (b'<', b'=', _) => (Tok::Le, 2),
            (b'<', _, _) => (Tok::Lt, 1),
            (b'>', b'>', b'=') => (Tok::ShrEq, 3),
            (b'>', b'>', _) => (Tok::GtGt, 2),
            (b'>', b'=', _) => (Tok::Ge, 2),
            (b'>', _, _) => (Tok::Gt, 1),
            (b'+', b'%', b'=') => (Tok::PlusPctEq, 3),
            (b'+', b'%', _) => (Tok::PlusPct, 2),
            (b'+', b'|', _) => (Tok::PlusPipe, 2),
            (b'+', b'=', _) => (Tok::PlusEq, 2),
            (b'+', _, _) => (Tok::Plus, 1),
            (b'-', b'%', b'=') => (Tok::MinusPctEq, 3),
            (b'-', b'%', _) => (Tok::MinusPct, 2),
            (b'-', b'|', _) => (Tok::MinusPipe, 2),
            (b'-', b'=', _) => (Tok::MinusEq, 2),
            (b'-', _, _) => (Tok::Minus, 1),
            (b'*', b'%', b'=') => (Tok::StarPctEq, 3),
            (b'*', b'%', _) => (Tok::StarPct, 2),
            (b'*', b'|', _) => (Tok::StarPipe, 2),
            (b'*', b'=', _) => (Tok::StarEq, 2),
            (b'*', _, _) => (Tok::Star, 1),
            (b'/', b'=', _) => (Tok::SlashEq, 2),
            (b'/', _, _) => (Tok::Slash, 1),
            (b'%', b'=', _) => (Tok::PercentEq, 2),
            (b'%', _, _) => (Tok::Percent, 1),
            (b'&', b'=', _) => (Tok::AmpEq, 2),
            (b'&', _, _) => (Tok::Amp, 1),
            (b'|', b'>', _) => (Tok::PipeGt, 2),
            (b'|', b'=', _) => (Tok::PipeEq, 2),
            (b'|', _, _) => (Tok::Pipe, 1),
            (b'^', b'=', _) => (Tok::CaretEq, 2),
            (b'^', _, _) => (Tok::Caret, 1),
            (b'?', _, _) => (Tok::Question, 1),
            (b'~', _, _) => (Tok::Tilde, 1),
            (b'@', _, _) => (Tok::At, 1),
            (b'#', _, _) => (Tok::Hash, 1),
            (b'$', _, _) => (Tok::Dollar, 1),
            _ => {
                self.pos += 1;
                self.err(start, format!("unexpected character `{}`", c as char));
                return;
            }
        };
        self.pos += len;
        self.push(tok, start);
    }
}

fn utf8_len(first: u8) -> usize {
    if first < 0x80 {
        1
    } else if first >> 5 == 0b110 {
        2
    } else if first >> 4 == 0b1110 {
        3
    } else {
        4
    }
}

fn parse_hex_float(text: &str) -> Option<f64> {
    // 1.8p3  => (0x1.8) * 2^3
    let (mant, exp) = match text.find(|c| c == 'p' || c == 'P') {
        Some(i) => (&text[..i], text[i + 1..].parse::<i32>().ok()?),
        None => (text, 0),
    };
    let (ip, fp) = match mant.find('.') {
        Some(i) => (&mant[..i], &mant[i + 1..]),
        None => (mant, ""),
    };
    let mut v = if ip.is_empty() { 0.0 } else { u64::from_str_radix(ip, 16).ok()? as f64 };
    let mut scale = 1.0 / 16.0;
    for ch in fp.chars() {
        v += ch.to_digit(16)? as f64 * scale;
        scale /= 16.0;
    }
    Some(v * 2f64.powi(exp))
}

impl Tok {
    pub fn describe(&self) -> String {
        match self {
            Tok::Ident(s) => format!("`{}`", s),
            Tok::Int(v) => format!("integer `{}`", v),
            Tok::Float(v) => format!("float `{}`", v),
            Tok::Str(_) => "string literal".into(),
            Tok::Bytes(_) => "byte string literal".into(),
            Tok::Char(_) => "character literal".into(),
            Tok::Doc(_) => "doc comment".into(),
            Tok::Comment(_) => "comment".into(),
            Tok::Newline => "newline".into(),
            Tok::Eof => "end of file".into(),
            other => format!("`{}`", other.text()),
        }
    }
    pub fn text(&self) -> &'static str {
        match self {
            Tok::LParen => "(",
            Tok::RParen => ")",
            Tok::LBrace => "{",
            Tok::RBrace => "}",
            Tok::LBracket => "[",
            Tok::RBracket => "]",
            Tok::Comma => ",",
            Tok::Colon => ":",
            Tok::Semi => ";",
            Tok::Dot => ".",
            Tok::DotStar => ".*",
            Tok::DotQuestion => ".?",
            Tok::DotDot => "..",
            Tok::DotLBrace => ".{",
            Tok::Arrow => "->",
            Tok::FatArrow => "=>",
            Tok::Question => "?",
            Tok::Bang => "!",
            Tok::Tilde => "~",
            Tok::At => "@",
            Tok::Pipe => "|",
            Tok::PipeGt => "|>",
            Tok::Hash => "#",
            Tok::Dollar => "$",
            Tok::Plus => "+",
            Tok::Minus => "-",
            Tok::Star => "*",
            Tok::Slash => "/",
            Tok::Percent => "%",
            Tok::PlusPct => "+%",
            Tok::MinusPct => "-%",
            Tok::StarPct => "*%",
            Tok::PlusPipe => "+|",
            Tok::MinusPipe => "-|",
            Tok::StarPipe => "*|",
            Tok::Amp => "&",
            Tok::Caret => "^",
            Tok::Shl => "<<",
            Tok::Shr => ">>",
            Tok::Eq => "=",
            Tok::EqEq => "==",
            Tok::Ne => "!=",
            Tok::Lt => "<",
            Tok::Le => "<=",
            Tok::Gt => ">",
            Tok::Ge => ">=",
            Tok::PlusEq => "+=",
            Tok::MinusEq => "-=",
            Tok::StarEq => "*=",
            Tok::SlashEq => "/=",
            Tok::PercentEq => "%=",
            Tok::AmpEq => "&=",
            Tok::PipeEq => "|=",
            Tok::CaretEq => "^=",
            Tok::ShlEq => "<<=",
            Tok::ShrEq => ">>=",
            Tok::PlusPctEq => "+%=",
            Tok::MinusPctEq => "-%=",
            Tok::StarPctEq => "*%=",
            Tok::LtLt => "<<",
            Tok::GtGt => ">>",
            _ => "?",
        }
    }
}
