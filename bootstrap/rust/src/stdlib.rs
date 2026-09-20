//! The standard library written in Nexium. Each module is embedded in the
//! compiler and loaded by `import std.<name>`; no files to install.
//!
//! Names must not collide with the builtin namespaces (`math`, `io`, `os`,
//! `time`, `random`, `mem`, `process`, ...), which are implemented in the
//! checker and the C backend.

pub const MODULES: &[(&str, &str)] = &[
    ("strings", include_str!("../../../std/strings.nx")),
    ("lists", include_str!("../../../std/lists.nx")),
    ("bytes", include_str!("../../../std/bytes.nx")),
    ("num", include_str!("../../../std/num.nx")),
    ("json", include_str!("../../../std/json.nx")),
    ("args", include_str!("../../../std/args.nx")),
    ("fs", include_str!("../../../std/fs.nx")),
    ("time", include_str!("../../../std/time.nx")),
    ("regex", include_str!("../../../std/regex.nx")),
    ("text", include_str!("../../../std/text.nx")),
    ("testing", include_str!("../../../std/testing.nx")),
    ("stream", include_str!("../../../std/stream.nx")),
    ("net", include_str!("../../../std/net.nx")),
    ("http", include_str!("../../../std/http.nx")),
    ("thread", include_str!("../../../std/thread.nx")),
    ("process", include_str!("../../../std/process.nx")),
];

pub fn source(name: &str) -> Option<&'static str> {
    MODULES.iter().find(|(n, _)| *n == name).map(|(_, s)| *s)
}
