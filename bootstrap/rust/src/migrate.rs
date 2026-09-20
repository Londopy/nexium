//! Migration from the 0.5 surface syntax to the 0.6 one, applied by `nx fmt`
//! before formatting, so upgrading a file is running the formatter on it.
//!
//! What changed (decision 87):
//! - `if (c) body` and `while (c) { }` lose the parentheses, and an `if`
//!   body is always a block: `if (c) return v` becomes `if c { return v }`,
//!   `if (c) a else b` becomes `if c { a } else { b }`;
//! - `for (items) |x, i| { }` becomes `for x, i in items { }`, and
//!   `for parallel (items) |x| { }` becomes `for parallel x in items { }`;
//! - `if (opt) |v| { }` becomes `if let v = opt { }`.
//!
//! The pass works on the token stream with byte spans and produces text
//! edits, so comments, blank lines and the author's layout survive; the
//! formatter then normalizes spacing. It is idempotent on new syntax: a
//! parenthesized condition is only unwrapped when the token after `)` is a
//! block, a capture, or the start of an unbraced body, never when the
//! parentheses are part of a larger expression.

use crate::lexer::{Lexer, Tok, Token};

/// A text edit: replace `[start, end)` with `text` (insert when start == end).
struct Edit {
    start: usize,
    end: usize,
    text: String,
}

pub fn migrate(src: &str) -> String {
    let (toks, diags) = Lexer::new(src, 0).with_comments().lex();
    if !diags.is_empty() {
        return src.to_string();
    }
    let mut edits: Vec<Edit> = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        if is_kw(&toks[i], "if") || is_kw(&toks[i], "while") {
            let is_if = is_kw(&toks[i], "if");
            if matches!(toks.get(i + 1).map(|t| &t.tok), Some(Tok::LParen)) {
                if let Some(close) = matching(&toks, i + 1) {
                    let after = toks.get(close + 1).map(|t| &t.tok);
                    let lp = &toks[i + 1];
                    let rp = &toks[close];
                    let inner = src[lp.span.end as usize..rp.span.start as usize].trim();
                    match after {
                        // `if (opt) |v| {`: a capture becomes `if let v = opt`
                        Some(Tok::Pipe) if is_if => {
                            if let (Some(Tok::Ident(v)), Some(Tok::Pipe)) = (toks.get(close + 2).map(|t| &t.tok), toks.get(close + 3).map(|t| &t.tok)) {
                                let end = toks[close + 3].span.end as usize;
                                edits.push(Edit { start: lp.span.start as usize, end, text: format!("let {} = {}", v, inner) });
                                let body_start = close + 4;
                                if !matches!(toks.get(body_start).map(|t| &t.tok), Some(Tok::LBrace)) {
                                    brace_body(&toks, body_start, &mut edits);
                                }
                                i = body_start;
                                continue;
                            }
                        }
                        // a token that can both continue the condition and start a
                        // body (`if (c) -v else { v }` vs `if (a) - b {`): it is a
                        // body when the line ends or reaches `else` before any `{`
                        Some(t) if prefix_capable(t) && !unbraced_body_follows(&toks, close + 1) => {}
                        // the parentheses are part of a longer condition: leave them
                        Some(t) if continues_expression(t) && !prefix_capable(t) => {}
                        // `if (c) {`: just unwrap
                        Some(Tok::LBrace) => {
                            edits.push(Edit { start: lp.span.start as usize, end: rp.span.end as usize, text: inner.to_string() });
                            i = close + 1;
                            continue;
                        }
                        // `if (c) stmt`: unwrap and brace the body
                        Some(_) if is_if => {
                            edits.push(Edit { start: lp.span.start as usize, end: rp.span.end as usize, text: inner.to_string() });
                            let body_start = close + 1;
                            brace_body(&toks, body_start, &mut edits);
                            // an unbraced `else` after the body is braced when the
                            // scan reaches it
                            i = body_start;
                            continue;
                        }
                        _ => {}
                    }
                }
            }
            i += 1;
            continue;
        }
        if is_kw(&toks[i], "else") {
            // `} else stmt` where the if was already migrated or braced by hand
            // (`else => ...` is a match arm's catch-all pattern, not a branch)
            let next = toks.get(i + 1).map(|t| &t.tok);
            if !matches!(next, Some(Tok::LBrace) | Some(Tok::Newline) | Some(Tok::Eof) | Some(Tok::FatArrow)) && !matches!(next, Some(Tok::Ident(s)) if s == "if") {
                brace_else(&toks, i, &mut edits);
            }
            i += 1;
            continue;
        }
        if is_kw(&toks[i], "for") {
            let mut j = i + 1;
            let parallel = matches!(toks.get(j).map(|t| &t.tok), Some(Tok::Ident(s)) if s == "parallel");
            if parallel {
                j += 1;
            }
            if matches!(toks.get(j).map(|t| &t.tok), Some(Tok::LParen)) {
                if let Some(close) = matching(&toks, j) {
                    if matches!(toks.get(close + 1).map(|t| &t.tok), Some(Tok::Pipe)) {
                        // find the closing pipe of the bindings
                        let mut k = close + 2;
                        while k < toks.len() && !matches!(toks[k].tok, Tok::Pipe) {
                            k += 1;
                        }
                        if k < toks.len() {
                            let inner = src[toks[j].span.end as usize..toks[close].span.start as usize].trim();
                            let names = src[toks[close + 1].span.end as usize..toks[k].span.start as usize].trim();
                            edits.push(Edit { start: toks[j].span.start as usize, end: toks[k].span.end as usize, text: format!("{} in {}", names, inner) });
                            i = k + 1;
                            continue;
                        }
                    }
                }
            }
            i += 1;
            continue;
        }
        i += 1;
    }
    apply(src, edits)
}

fn is_kw(t: &Token, k: &str) -> bool {
    matches!(&t.tok, Tok::Ident(s) if s == k)
}

/// Does a token after `)` mean the parentheses were part of a larger
/// condition rather than the old wrapper?
fn continues_expression(t: &Tok) -> bool {
    match t {
        Tok::Dot | Tok::DotStar | Tok::DotQuestion | Tok::DotDot | Tok::LBracket | Tok::LParen | Tok::PipeGt => true,
        Tok::Plus | Tok::Minus | Tok::Star | Tok::Slash | Tok::Percent | Tok::PlusPct | Tok::MinusPct | Tok::StarPct | Tok::PlusPipe | Tok::MinusPipe | Tok::StarPipe => true,
        Tok::Amp | Tok::Caret | Tok::Shl | Tok::Shr | Tok::EqEq | Tok::Ne | Tok::Lt | Tok::Le | Tok::Gt | Tok::Ge | Tok::Question | Tok::Bang => true,
        Tok::Ident(s) => matches!(s.as_str(), "and" | "or" | "as" | "catch" | "orelse"),
        _ => false,
    }
}

/// Tokens that are binary operators but also start an expression.
fn prefix_capable(t: &Tok) -> bool {
    matches!(t, Tok::Minus | Tok::Star | Tok::Amp | Tok::LParen | Tok::LBracket)
}

/// From `at`, does the line end (or reach `else`) at depth 0 before any `{`?
/// True means the tokens are an old unbraced body, not a continuation.
fn unbraced_body_follows(toks: &[Token], at: usize) -> bool {
    let mut depth = 0i32;
    let mut k = at;
    while k < toks.len() {
        match &toks[k].tok {
            Tok::LParen | Tok::LBracket | Tok::DotLBrace => depth += 1,
            Tok::LBrace if depth == 0 => return false,
            Tok::LBrace => depth += 1,
            Tok::RParen | Tok::RBracket | Tok::RBrace => {
                if depth == 0 {
                    return true;
                }
                depth -= 1;
            }
            Tok::Newline | Tok::Eof | Tok::Comment(_) | Tok::Doc(_) if depth == 0 => return true,
            Tok::Ident(s) if s == "else" && depth == 0 => return true,
            _ => {}
        }
        k += 1;
    }
    true
}

/// The index of the `)` matching the `(` at `open`.
fn matching(toks: &[Token], open: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (k, t) in toks.iter().enumerate().skip(open) {
        match t.tok {
            Tok::LParen | Tok::LBracket | Tok::LBrace | Tok::DotLBrace => depth += 1,
            Tok::RParen | Tok::RBracket | Tok::RBrace => {
                depth -= 1;
                if depth == 0 {
                    return if matches!(t.tok, Tok::RParen) { Some(k) } else { None };
                }
            }
            Tok::Eof => return None,
            _ => {}
        }
    }
    None
}

/// Wrap the unbraced body starting at `start` in `{ ... }`.
fn brace_body(toks: &[Token], start: usize, edits: &mut Vec<Edit>) {
    let end = body_end(toks, start);
    edits.push(Edit { start: toks[start].span.start as usize, end: toks[start].span.start as usize, text: "{ ".into() });
    let end_off = toks[end - 1].span.end as usize;
    edits.push(Edit { start: end_off, end: end_off, text: " }".into() });
}

/// One past the last token of an unbraced body starting at `start`: the body
/// runs to the end of the line, an `else` of its own, a comment, or a closing
/// brace at its own depth. An `if` inside the body owns the next `else`, so
/// `return if (c) a else b` is one body.
fn body_end(toks: &[Token], start: usize) -> usize {
    let mut depth = 0i32;
    let mut inner_ifs = 0usize;
    let mut k = start;
    while k < toks.len() {
        match &toks[k].tok {
            Tok::LParen | Tok::LBracket | Tok::LBrace | Tok::DotLBrace => depth += 1,
            Tok::RParen | Tok::RBracket | Tok::RBrace => {
                if depth == 0 {
                    return k;
                }
                depth -= 1;
            }
            Tok::Newline | Tok::Eof | Tok::Comment(_) | Tok::Doc(_) if depth == 0 => return k,
            // `.{if (c) a else b, sp}`: the body is one element of the literal
            Tok::Comma if depth == 0 => return k,
            Tok::Ident(s) if s == "if" && depth == 0 => inner_ifs += 1,
            Tok::Ident(s) if s == "else" && depth == 0 => {
                if inner_ifs == 0 {
                    return k;
                }
                inner_ifs -= 1;
            }
            _ => {}
        }
        k += 1;
    }
    k
}

/// Brace the unbraced statement after the `else` at `at`.
fn brace_else(toks: &[Token], at: usize, edits: &mut Vec<Edit>) {
    let next = at + 1;
    match toks.get(next).map(|t| &t.tok) {
        Some(Tok::LBrace) | Some(Tok::Newline) | Some(Tok::Eof) | Some(Tok::FatArrow) | None => {}
        Some(Tok::Ident(s)) if s == "if" => {}
        _ => {
            let end = body_end(toks, next);
            edits.push(Edit { start: toks[next].span.start as usize, end: toks[next].span.start as usize, text: "{ ".into() });
            let end_off = toks[end - 1].span.end as usize;
            edits.push(Edit { start: end_off, end: end_off, text: " }".into() });
        }
    }
}

fn apply(src: &str, mut edits: Vec<Edit>) -> String {
    // later edits first, so earlier offsets stay valid; at equal offsets keep
    // insertion order (an inner `}` before an outer one is the same text)
    edits.sort_by(|a, b| b.start.cmp(&a.start));
    let mut out = src.to_string();
    for e in edits {
        out.replace_range(e.start..e.end, &e.text);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::migrate;

    #[test]
    fn conditions_and_bodies() {
        assert_eq!(migrate("if (x > 3) { f() }"), "if x > 3 { f() }");
        assert_eq!(migrate("if (c) return v\n"), "if c { return v }\n");
        assert_eq!(migrate("let x = if (c) 1 else 2\n"), "let x = if c { 1 } else { 2 }\n");
        assert_eq!(migrate("if (a) x = 1 else if (b) y = 2 else z = 3\n"), "if a { x = 1 } else if b { y = 2 } else { z = 3 }\n");
        assert_eq!(migrate("if (c) return // note\n"), "if c { return } // note\n");
        assert_eq!(migrate("while (i < n) { i += 1 }"), "while i < n { i += 1 }");
        assert_eq!(migrate("if (a) and (b) { f() }"), "if (a) and (b) { f() }");
        assert_eq!(migrate("if (f(x)).ok { g() }"), "if (f(x)).ok { g() }");
        assert_eq!(migrate("if (opt) |v| { use(v) }"), "if let v = opt { use(v) }");
        assert_eq!(migrate("{ if (c) x }"), "{ if c { x } }");
        assert_eq!(
            migrate(
                "let n = if (v < 0) - v else { v }
"
            ),
            "let n = if v < 0 { - v } else { v }
"
        );
        assert_eq!(
            migrate(
                "let m = if (c) (a as i64) - 3 else { 9 }
"
            ),
            "let m = if c { (a as i64) - 3 } else { 9 }
"
        );
        assert_eq!(migrate("if (a) - b > 0 { f() }"), "if (a) - b > 0 { f() }");
        assert_eq!(
            migrate(
                "if (opt) |v| return v
"
            ),
            "if let v = opt { return v }
"
        );
        assert_eq!(
            migrate(
                "if (a) return if (b) x else y
"
            ),
            "if a { return if b { x } else { y } }
"
        );
        assert_eq!(
            migrate(
                "if (a) x else if (b) y
"
            ),
            "if a { x } else if b { y }
"
        );
        assert_eq!(migrate(".{if (c) a else b, sp}"), ".{if c { a } else { b }, sp}");
        assert_eq!(
            migrate(
                "match x {
    1 => a,
    else => b,
}
"
            ),
            "match x {
    1 => a,
    else => b,
}
"
        );
    }

    #[test]
    fn loops() {
        assert_eq!(migrate("for (items) |x, i| { f(x, i) }"), "for x, i in items { f(x, i) }");
        assert_eq!(migrate("for (0..n) |i| { }"), "for i in 0..n { }");
        assert_eq!(migrate("for parallel (items) |x| { }"), "for parallel x in items { }");
        assert_eq!(migrate("for (a, b) |x, y| { }"), "for x, y in a, b { }");
        assert_eq!(migrate("for (0..10 step 2) |i| { }"), "for i in 0..10 step 2 { }");
    }

    #[test]
    fn already_new() {
        let s = "fn f(c: bool) -> i32 {\n    if c { return 1 }\n    for x in xs { g(x) }\n    if let v = opt { h(v) }\n    while c { break }\n    return 2\n}\n";
        assert_eq!(migrate(s), s);
    }
}
