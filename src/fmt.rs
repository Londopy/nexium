//! `nx fmt`: canonical formatting, no options (archived section 18).
//!
//! The formatter is token based and line preserving: it never joins or splits
//! lines, so a program's shape stays the author's. It normalizes indentation
//! (four spaces per open brace, one extra level for a continuation line),
//! spacing around operators, after commas and colons, trailing whitespace,
//! and runs of blank lines (at most one). Comments stay where they are.

use crate::lexer::{Lexer, Tok, Token};
use crate::parser::EFFECT_NAMES;

pub fn format_source(src: &str) -> Result<String, String> {
    let (toks, diags) = Lexer::new(src, 0).with_comments().lex();
    if !diags.is_empty() {
        return Err("the file has lexical errors; fix them before formatting".into());
    }
    let line_of = |t: &Token| src[..t.span.start as usize].matches('\n').count();
    let n_lines = src.lines().count() + 1;
    let mut lines: Vec<Vec<&Token>> = vec![Vec::new(); n_lines];
    for t in &toks {
        if matches!(t.tok, Tok::Newline | Tok::Eof) {
            continue;
        }
        let l = line_of(t);
        if l < lines.len() {
            lines[l].push(t);
        }
    }
    let mut out = String::new();
    let mut depth: i32 = 0;
    let mut paren: i32 = 0;
    let mut blank_run = 0;
    let mut prev_line_last: Option<Tok> = None;
    // depth at which an `enum` body opened (variants with `{` payloads live there)
    let mut enum_body_depth: Option<i32> = None;
    let mut pending_enum = false;
    let mut had_content = false;
    let mut pattern_depth: i32 = 0;
    for toks in &lines {
        if toks.is_empty() {
            if had_content {
                blank_run += 1;
                if blank_run == 1 {
                    out.push('\n');
                }
            }
            continue;
        }
        had_content = true;
        blank_run = 0;
        let mut line_depth = depth;
        if matches!(toks[0].tok, Tok::RBrace) {
            line_depth -= 1;
        }
        let leading_dot_continues = matches!(toks[0].tok, Tok::Dot) && !matches!(prev_line_last, Some(Tok::LBrace) | Some(Tok::Comma) | Some(Tok::FatArrow) | Some(Tok::RBrace) | None);
        let continuation = paren > 0
            || leading_dot_continues
            || matches!(toks[0].tok, Tok::PipeGt)
            || matches!(&toks[0].tok, Tok::Ident(s) if s == "catch" || s == "orelse" || s == "and" || s == "or")
            || matches!(prev_line_last, Some(ref t) if is_binary_op(t) && !matches!(t, Tok::FatArrow | Tok::Eq));
        let extra = if continuation && !matches!(toks[0].tok, Tok::RBrace | Tok::RParen | Tok::RBracket) { 4 } else { 0 };
        out.push_str(&" ".repeat((line_depth.max(0) as usize) * 4 + extra));
        let in_enum_body = enum_body_depth == Some(depth);
        let mut ctx = LineCtx { bracket_prev: Vec::new(), in_enum_body, pattern_depth_at_start: pattern_depth };
        for (i, t) in toks.iter().enumerate() {
            if i > 0 && needs_space(toks, i, &ctx) {
                out.push(' ');
            }
            out.push_str(&src[t.span.start as usize..t.span.end as usize]);
            match &t.tok {
                Tok::LBrace | Tok::DotLBrace => {
                    depth += 1;
                    if pending_enum {
                        enum_body_depth = Some(depth);
                        pending_enum = false;
                    }
                }
                Tok::RBrace => {
                    if enum_body_depth == Some(depth) {
                        enum_body_depth = None;
                    }
                    depth -= 1;
                }
                Tok::LParen | Tok::LBracket => {
                    paren += 1;
                    ctx.bracket_prev.push(if i > 0 { Some(toks[i - 1].tok.clone()) } else { None });
                }
                Tok::RParen | Tok::RBracket => {
                    paren -= 1;
                    ctx.bracket_prev.pop();
                }
                Tok::Ident(s) if s == "enum" => pending_enum = true,
                Tok::LtLt if i == 0 || !ends_operand(&toks[i - 1].tok) => {
                    pattern_depth += 1;
                    paren += 1;
                }
                Tok::GtGt if pattern_depth > 0 => {
                    pattern_depth -= 1;
                    paren -= 1;
                }
                _ => {}
            }
        }
        while out.ends_with(' ') {
            out.pop();
        }
        out.push('\n');
        prev_line_last = toks.last().map(|t| t.tok.clone());
    }
    while out.ends_with("\n\n") {
        out.pop();
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

struct LineCtx {
    /// for each open bracket on this line, the token before it
    bracket_prev: Vec<Option<Tok>>,
    in_enum_body: bool,
    /// binary patterns open at the start of this line
    pattern_depth_at_start: i32,
}

/// Is position `i` inside a `<< ... >>` binary pattern on this line?
fn in_pattern(toks: &[&Token], i: usize, ctx: &LineCtx) -> bool {
    let mut d = ctx.pattern_depth_at_start;
    for (j, t) in toks.iter().enumerate().take(i) {
        match &t.tok {
            Tok::LtLt if j == 0 || !ends_operand(&toks[j - 1].tok) => d += 1,
            Tok::GtGt if d > 0 => d -= 1,
            _ => {}
        }
    }
    d > 0
}

fn is_binary_op(t: &Tok) -> bool {
    matches!(
        t,
        Tok::Plus
            | Tok::Minus
            | Tok::Star
            | Tok::Slash
            | Tok::Percent
            | Tok::PlusPct
            | Tok::MinusPct
            | Tok::StarPct
            | Tok::PlusPipe
            | Tok::MinusPipe
            | Tok::StarPipe
            | Tok::Amp
            | Tok::Caret
            | Tok::LtLt
            | Tok::GtGt
            | Tok::Eq
            | Tok::EqEq
            | Tok::Ne
            | Tok::Lt
            | Tok::Le
            | Tok::Gt
            | Tok::Ge
            | Tok::PlusEq
            | Tok::MinusEq
            | Tok::StarEq
            | Tok::SlashEq
            | Tok::PercentEq
            | Tok::AmpEq
            | Tok::PipeEq
            | Tok::CaretEq
            | Tok::ShlEq
            | Tok::ShrEq
            | Tok::PlusPctEq
            | Tok::MinusPctEq
            | Tok::StarPctEq
            | Tok::PipeGt
            | Tok::Arrow
            | Tok::FatArrow
    )
}

fn is_word(t: &Tok) -> bool {
    matches!(t, Tok::Ident(_) | Tok::Int(_) | Tok::Float(_) | Tok::Str(_) | Tok::Bytes(_) | Tok::Char(_))
}

fn is_kw(t: &Tok, k: &str) -> bool {
    matches!(t, Tok::Ident(s) if s == k)
}

fn is_keyword_tok(t: &Tok) -> bool {
    matches!(t, Tok::Ident(s) if crate::lexer::is_keyword(s))
}

fn is_effect_name(t: &Tok) -> bool {
    matches!(t, Tok::Ident(s) if EFFECT_NAMES.contains(&s.as_str()))
}

/// Ends an operand: a word or a closer, so the next `-`/`&`/`*` is binary.
fn ends_operand(t: &Tok) -> bool {
    (is_word(t) && !is_keyword_tok(t)) || matches!(t, Tok::RParen | Tok::RBracket | Tok::RBrace | Tok::DotQuestion | Tok::DotStar) || is_kw(t, "true") || is_kw(t, "false") || is_kw(t, "null")
}

/// Is the token a type-position introducer (`:` `->` `?` `*` `[]`)?
fn type_position_before(t: &Tok) -> bool {
    matches!(t, Tok::Colon | Tok::Arrow | Tok::Question | Tok::Bang | Tok::Comma | Tok::LParen | Tok::LBracket | Tok::RBracket | Tok::Eq | Tok::FatArrow)
        || is_kw(t, "mut")
        || is_kw(t, "as")
        || is_kw(t, "distinct")
        || is_kw(t, "weak")
        || is_kw(t, "type")
}

/// Should a space separate `toks[i-1]` and `toks[i]`?
/// Does the `)` at `at` close the condition of an `if`, `while`, or `for`?
fn closes_control_head(toks: &[&Token], at: usize) -> bool {
    let mut depth = 0;
    let mut j = at;
    loop {
        match &toks[j].tok {
            Tok::RParen => depth += 1,
            Tok::LParen => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        if j == 0 {
            return false;
        }
        j -= 1;
    }
    j > 0 && matches!(&toks[j - 1].tok, Tok::Ident(k) if k == "if" || k == "while" || k == "for")
}

fn needs_space(toks: &[&Token], i: usize, ctx: &LineCtx) -> bool {
    use Tok::*;
    let a = &toks[i - 1].tok;
    let b = &toks[i].tok;
    let pp = if i >= 2 { Some(&toks[i - 2].tok) } else { None };
    let next = toks.get(i + 1).map(|t| &t.tok);
    if matches!(b, Comment(_) | Doc(_)) {
        return true;
    }
    // ranges and member access are tight: `0..n`, `1..=9`
    if matches!(a, DotDot) || matches!(b, DotDot) || (matches!(a, Eq) && matches!(pp, Some(DotDot))) {
        return false;
    }
    // or-patterns in match arms: `.A | .B => ...`
    let arm_line = toks.iter().skip(i).any(|t| matches!(t.tok, FatArrow));
    let first_bar_prev = toks.iter().position(|t| matches!(t.tok, Pipe)).and_then(|k| if k > 0 { Some(&toks[k - 1].tok) } else { None });
    let closure_bar = matches!(first_bar_prev, Some(t) if matches!(t, Eq | LParen | RParen | Comma | FatArrow) || is_keyword_tok(t));
    if (matches!(a, Pipe) || matches!(b, Pipe)) && arm_line && !closure_bar {
        return true;
    }
    // labels: `break :outer`, `outer: {`
    if matches!(b, Colon) && (is_kw(a, "break") || is_kw(a, "continue")) {
        return true;
    }
    if matches!(a, Colon) && matches!(pp, Some(t) if is_kw(t, "break") || is_kw(t, "continue")) {
        return false;
    }
    // binary patterns: `<<a:4, rest:bytes>>` is written tight
    let inside_pattern = in_pattern(toks, i, ctx);
    if inside_pattern {
        if matches!(a, LtLt)
            || matches!(b, GtGt)
            || matches!(a, Colon)
            || matches!(b, Colon)
            || matches!(a, Slash)
            || matches!(b, Slash)
            || matches!(a, Minus)
            || matches!(b, Minus)
            || matches!(a, Star)
            || matches!(b, Star)
        {
            return false;
        }
    } else if matches!(b, LtLt) && !ends_operand(a) {
        return !matches!(a, LParen | LBracket | Comma) || matches!(a, Comma);
    }
    // openers and member access never have a space after them
    if matches!(a, LParen | LBracket | Dot | DotStar | DotQuestion | At) {
        return false;
    }
    if matches!(a, Pipe) && matches!(b, LBracket) {
        return false;
    }
    // closers and separators never have a space before them
    if matches!(b, RParen | RBracket | Comma | Semi | DotStar | DotQuestion) {
        return false;
    }
    // `.field` after a comma or an opening brace is a field initializer; otherwise member access
    if matches!(b, Dot) {
        return matches!(a, Comma | LBrace | DotLBrace);
    }
    if matches!(a, Colon | Comma) {
        return true;
    }
    // inside braces: `{ .x = 1 }` and blocks get spaces, `.{a, b}` tuples do not
    if matches!(a, LBrace | DotLBrace) {
        return matches!(a, LBrace) || matches!(b, Dot);
    }
    if matches!(b, RBrace) {
        let mut depth = 0;
        let mut j = i - 1;
        loop {
            match &toks[j].tok {
                RBrace => depth += 1,
                LBrace | DotLBrace => {
                    if depth == 0 {
                        return matches!(toks[j].tok, LBrace) || matches!(toks.get(j + 1).map(|t| &t.tok), Some(Dot));
                    }
                    depth -= 1;
                }
                _ => {}
            }
            if j == 0 {
                break;
            }
            j -= 1;
        }
        return true;
    }
    // `name:` in fields/params/labels/segments, but `error.X => ...` and `a ? b`
    if matches!(b, Colon) {
        return false;
    }
    // calls, indexing, generic instantiation: `f(`, `xs[`, `List(`
    if matches!(b, LParen) {
        if is_kw(a, "impl") || is_kw(a, "fn") || is_kw(a, "export") || is_kw(a, "derive") || is_kw(a, "layout") {
            return false;
        }
        if matches!(a, RParen) && closes_control_head(toks, i - 1) {
            return true;
        }
        return !(ends_operand(a) || matches!(a, Question | Star))
            || is_kw(a, "return")
            || is_kw(a, "if")
            || is_kw(a, "while")
            || is_kw(a, "for")
            || is_kw(a, "match")
            || is_kw(a, "and")
            || is_kw(a, "or")
            || is_kw(a, "in");
    }
    if matches!(b, LBracket) {
        // `xs[i]` vs `: []u8`, `-> [N]T`, `= [1, 2]`, `?[]u8`
        if matches!(a, Question | Star | Bang) {
            return false;
        }
        return !ends_operand(a) || is_kw(a, "return") || is_kw(a, "as");
    }
    // `[]u8`, `[N]T`, `[]dyn T`: no space after `]` in type position
    if matches!(a, RBracket) && is_word(b) && (!is_keyword_tok(b) || is_kw(b, "dyn") || is_kw(b, "weak") || is_kw(b, "fn") || is_kw(b, "mut")) {
        let opener_prev = ctx.bracket_prev.last().cloned().flatten();
        let _ = opener_prev;
        // find the matching bracket's predecessor: scan back on this line
        let mut depth = 0;
        let mut j = i - 1;
        loop {
            match &toks[j].tok {
                RBracket => depth += 1,
                LBracket => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
            if j == 0 {
                break;
            }
            j -= 1;
        }
        let before = if j > 0 { Some(&toks[j - 1].tok) } else { None };
        return match before {
            Some(t) => !type_position_before(t) && !is_kw(t, "mut"),
            None => false,
        };
    }
    // struct literals `Point{`, declarations `struct Point {`, blocks `) {`
    if matches!(b, LBrace) {
        if matches!(&toks[0].tok, Ident(k) if matches!(k.as_str(), "struct" | "enum" | "record" | "ref" | "impl" | "trait" | "error" | "artifact" | "using" | "test")) {
            return true;
        }
        if let Ident(s) = a {
            let declared = matches!(pp, Some(t) if is_kw(t, "struct") || is_kw(t, "enum") || is_kw(t, "record") || is_kw(t, "class") || is_kw(t, "trait") || is_kw(t, "impl") || is_kw(t, "error") || is_kw(t, "for") || is_kw(t, "using") || is_kw(t, "artifact"));
            let uppercase = s.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false);
            let type_pos = matches!(pp, Some(Arrow) | Some(Colon) | Some(Bang) | Some(Question) | Some(Star));
            if uppercase && !declared && !type_pos && !ctx.in_enum_body && !crate::lexer::is_keyword(s) {
                return false;
            }
        }
        if matches!(a, RParen) && matches!(pp, Some(t) if is_word(t)) {
            // `Pair(i32){` generic instantiation literal vs `if (c) {`
            let mut depth = 0;
            let mut j = i - 1;
            loop {
                match &toks[j].tok {
                    RParen => depth += 1,
                    LParen => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                if j == 0 {
                    break;
                }
                j -= 1;
            }
            let head = if j > 0 { Some(&toks[j - 1].tok) } else { None };
            let before_head = if j > 1 { Some(&toks[j - 2].tok) } else { None };
            if let Some(Ident(h)) = head {
                let in_return_type = matches!(before_head, Some(Arrow));
                if h.chars().next().map(|c| c.is_ascii_uppercase()).unwrap_or(false) && !in_return_type {
                    return false;
                }
            }
        }
        return true;
    }

    // error unions and effect bounds: `ParseError!i64`, `!void`, `-> i32 !allocates`
    if matches!(b, Bang) {
        return !(is_word(a) && !is_keyword_tok(a) && !matches!(next, Some(t) if is_effect_name(t)));
    }
    if matches!(a, Bang) {
        return false;
    }
    if matches!(a, Question) || matches!(a, Tilde) {
        return false;
    }
    if matches!(b, Question) {
        return matches!(a, Arrow | Eq | FatArrow | RParen) || is_kw(a, "mut") || is_kw(a, "as") || is_kw(a, "return");
    }
    // unary minus / address-of / pointer types
    if matches!(a, Minus | Amp | Star) {
        let binary = matches!(pp, Some(t) if ends_operand(t));
        if !binary {
            return false;
        }
    }
    if matches!(b, Minus | Amp | Star) && !ends_operand(a) {
        // unary after an operator or opener: `x = -1`, `f(&x)`, `: *mut T`
        return !matches!(a, LParen | LBracket | DotLBrace | Colon | Arrow) || matches!(a, Colon | Arrow);
    }
    // closure parameter bars: `|[a] x: i32|`, `|x|`
    if matches!(a, Pipe) {
        let bars_so_far = toks[..i].iter().filter(|t| matches!(t.tok, Pipe)).count();
        let closing = bars_so_far % 2 == 0;
        if closing && (is_word(b) || matches!(b, LParen | Minus | Bang | At)) {
            return true;
        }
        return matches!(b, LBrace) || (is_binary_op(b) && !matches!(b, Pipe)) || false;
    }
    if matches!(b, Pipe) {
        // `for (xs) |x|`, `catch |e|`, `let f = |x| ...` open a bar with a space; a closing bar follows its parameter directly
        return matches!(a, RParen | Eq | Comma | LParen | FatArrow) || is_keyword_tok(a);
    }
    if is_binary_op(a) || is_binary_op(b) {
        return true;
    }
    if matches!(a, Colon | Comma) {
        return true;
    }
    true
}
