//! Diagnostics: spans, source map, and rendering.
//!
//! Section 18 of the archived specification: every diagnostic names the
//! constraint violated, the location violating it, and where the constraint
//! was declared. `Diag::note` carries the "where it was declared" half.

use std::fmt::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub file: u32,
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn to(self, other: Span) -> Span {
        Span { file: self.file, start: self.start.min(other.start), end: self.end.max(other.end) }
    }
    pub fn dummy() -> Span {
        Span { file: u32::MAX, start: 0, end: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Error,
    Warning,
    Note,
}

#[derive(Clone, Debug)]
pub struct Diag {
    pub level: Level,
    pub span: Span,
    pub msg: String,
    pub notes: Vec<(Option<Span>, String)>,
}

impl Diag {
    pub fn error(span: Span, msg: impl Into<String>) -> Diag {
        Diag { level: Level::Error, span, msg: msg.into(), notes: Vec::new() }
    }
    pub fn warning(span: Span, msg: impl Into<String>) -> Diag {
        Diag { level: Level::Warning, span, msg: msg.into(), notes: Vec::new() }
    }
    pub fn note(mut self, span: Option<Span>, msg: impl Into<String>) -> Diag {
        self.notes.push((span, msg.into()));
        self
    }
}

pub struct SourceFile {
    pub name: String,
    pub text: String,
    line_starts: Vec<u32>,
}

#[derive(Default)]
pub struct SourceMap {
    pub files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn add(&mut self, name: String, text: String) -> u32 {
        let mut line_starts = vec![0u32];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i as u32 + 1);
            }
        }
        self.files.push(SourceFile { name, text, line_starts });
        (self.files.len() - 1) as u32
    }

    pub fn file(&self, id: u32) -> Option<&SourceFile> {
        self.files.get(id as usize)
    }

    /// 1-based line and column
    pub fn line_col(&self, span: Span) -> Option<(usize, usize)> {
        let f = self.file(span.file)?;
        let line = match f.line_starts.binary_search(&span.start) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let col = span.start - f.line_starts[line];
        Some((line + 1, col as usize + 1))
    }

    pub fn line_text(&self, file: u32, line1: usize) -> Option<&str> {
        let f = self.file(file)?;
        let s = *f.line_starts.get(line1 - 1)? as usize;
        let e = f.line_starts.get(line1).map(|&e| e as usize).unwrap_or(f.text.len());
        Some(f.text[s..e].trim_end_matches(['\n', '\r']))
    }

    pub fn location(&self, span: Span) -> String {
        match (self.file(span.file), self.line_col(span)) {
            (Some(f), Some((l, c))) => format!("{}:{}:{}", f.name, l, c),
            _ => "<unknown>".into(),
        }
    }

    pub fn render(&self, d: &Diag) -> String {
        let mut out = String::new();
        let level = match d.level {
            Level::Error => "error",
            Level::Warning => "warning",
            Level::Note => "note",
        };
        let _ = writeln!(out, "{}: {}", level, d.msg);
        self.render_span(&mut out, d.span);
        for (span, msg) in &d.notes {
            let _ = writeln!(out, "  note: {}", msg);
            if let Some(s) = span {
                self.render_span(&mut out, *s);
            }
        }
        out
    }

    fn render_span(&self, out: &mut String, span: Span) {
        if let (Some(f), Some((l, c))) = (self.file(span.file), self.line_col(span)) {
            let _ = writeln!(out, "  --> {}:{}:{}", f.name, l, c);
            if let Some(text) = self.line_text(span.file, l) {
                let _ = writeln!(out, "   |");
                let _ = writeln!(out, "{:>3}| {}", l, text);
                let width = ((span.end - span.start) as usize).max(1).min(text.len().saturating_sub(c - 1).max(1));
                let _ = writeln!(out, "   | {}{}", " ".repeat(c - 1), "^".repeat(width));
            }
        }
    }
}
