//! Packages: the `nexium.toml` manifest, the `nexium.lock` file, and fetching
//! dependencies into `nexium_modules/`.
//!
//! A package is a directory with a manifest and a `src/` folder. `import
//! foo` loads `src/lib.nx` of the dependency `foo`; `import foo.bar` loads
//! `src/bar.nx`. Dependencies come from a git tag or a local path:
//!
//! ```toml
//! [package]
//! name = "app"
//! version = "0.1.0"
//!
//! [dependencies]
//! greet = { git = "https://github.com/someone/greet", tag = "v1.2.0" }
//! local = { path = "../local" }
//! ```
//!
//! The TOML reader covers this subset: tables, string, number and boolean
//! values, inline tables, arrays of strings, comments.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const MANIFEST: &str = "nexium.toml";
pub const LOCK: &str = "nexium.lock";
pub const MODULES_DIR: &str = "nexium_modules";

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Str(String),
    Table(Vec<(String, Value)>),
    List(Vec<Value>),
}

impl Value {
    fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Table(kv) => kv.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
}

/// Parse the TOML subset into `table path -> entries`.
pub fn parse_toml(text: &str) -> Result<HashMap<String, Vec<(String, Value)>>, String> {
    let mut tables: HashMap<String, Vec<(String, Value)>> = HashMap::new();
    let mut current = String::new();
    tables.entry(current.clone()).or_default();
    for (i, raw) in text.lines().enumerate() {
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        let at = |m: &str| format!("{}:{}: {}", MANIFEST, i + 1, m);
        if let Some(rest) = line.strip_prefix('[') {
            let name = rest.strip_suffix(']').ok_or_else(|| at("unterminated table header"))?;
            current = name.trim().to_string();
            tables.entry(current.clone()).or_default();
            continue;
        }
        let (k, v) = line.split_once('=').ok_or_else(|| at("expected `key = value`"))?;
        let key = k.trim().trim_matches('"').to_string();
        let value = parse_value(v.trim()).map_err(|m| at(&m))?;
        tables.get_mut(&current).unwrap().push((key, value));
    }
    Ok(tables)
}

fn strip_comment(line: &str) -> &str {
    let mut in_str = false;
    for (i, ch) in line.char_indices() {
        match ch {
            '"' => in_str = !in_str,
            '#' if !in_str => return &line[..i],
            _ => {}
        }
    }
    line
}

fn parse_value(v: &str) -> Result<Value, String> {
    if let Some(inner) = v.strip_prefix('"') {
        let end = inner.find('"').ok_or("unterminated string")?;
        return Ok(Value::Str(inner[..end].to_string()));
    }
    if let Some(inner) = v.strip_prefix('{') {
        let inner = inner.strip_suffix('}').ok_or("unterminated inline table")?;
        let mut kv = Vec::new();
        for part in split_top(inner) {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let (k, x) = part.split_once('=').ok_or("expected `key = value` in the inline table")?;
            kv.push((k.trim().to_string(), parse_value(x.trim())?));
        }
        return Ok(Value::Table(kv));
    }
    if let Some(inner) = v.strip_prefix('[') {
        let inner = inner.strip_suffix(']').ok_or("unterminated array")?;
        let mut items = Vec::new();
        for part in split_top(inner) {
            let part = part.trim();
            if !part.is_empty() {
                items.push(parse_value(part)?);
            }
        }
        return Ok(Value::List(items));
    }
    // numbers and booleans are kept as their text
    Ok(Value::Str(v.to_string()))
}

/// Split on commas outside strings and brackets.
fn split_top(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0;
    let mut in_str = false;
    let mut cur = String::new();
    for ch in s.chars() {
        match ch {
            '"' => {
                in_str = !in_str;
                cur.push(ch);
            }
            '{' | '[' if !in_str => {
                depth += 1;
                cur.push(ch);
            }
            '}' | ']' if !in_str => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if !in_str && depth == 0 => {
                out.push(std::mem::take(&mut cur));
            }
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur);
    }
    out
}

#[derive(Clone, Debug)]
pub struct Dep {
    pub name: String,
    pub git: Option<String>,
    pub tag: Option<String>,
    pub path: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub deps: Vec<Dep>,
    /// the directory holding the manifest
    pub dir: PathBuf,
}

impl Manifest {
    pub fn read(path: &Path) -> Result<Manifest, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
        let tables = parse_toml(&text)?;
        let pkg = tables.get("package").ok_or_else(|| format!("{}: missing a [package] table", path.display()))?;
        let field = |k: &str| pkg.iter().find(|(n, _)| n == k).and_then(|(_, v)| v.as_str().map(|s| s.to_string()));
        let name = field("name").ok_or_else(|| format!("{}: [package] needs a name", path.display()))?;
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') || name.is_empty() {
            return Err(format!("{}: package name `{}` must be an identifier (letters, digits, underscores)", path.display(), name));
        }
        let version = field("version").unwrap_or_else(|| "0.0.0".into());
        let mut deps = Vec::new();
        if let Some(d) = tables.get("dependencies") {
            for (dname, v) in d {
                let dep = match v {
                    Value::Str(s) => Dep { name: dname.clone(), git: None, tag: Some(s.clone()), path: None },
                    Value::Table(_) => Dep {
                        name: dname.clone(),
                        git: v.get("git").and_then(|x| x.as_str()).map(|s| s.to_string()),
                        tag: v.get("tag").and_then(|x| x.as_str()).map(|s| s.to_string()),
                        path: v.get("path").and_then(|x| x.as_str()).map(|s| s.to_string()),
                    },
                    Value::List(_) => return Err(format!("{}: dependency `{}` cannot be an array", path.display(), dname)),
                };
                if dep.git.is_none() && dep.path.is_none() {
                    return Err(format!("{}: dependency `{}` needs `git = \"url\"` (with `tag`) or `path = \"dir\"`", path.display(), dname));
                }
                deps.push(dep);
            }
        }
        Ok(Manifest { name, version, deps, dir: path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from(".")) })
    }

    /// Where a dependency's checkout lives.
    pub fn dep_dir(&self, dep: &Dep) -> PathBuf {
        match &dep.path {
            Some(p) => self.dir.join(p),
            None => self.dir.join(MODULES_DIR).join(&dep.name),
        }
    }
}

/// The manifest governing a directory: the nearest `nexium.toml` at or above it.
pub fn find(start: &Path) -> Option<Manifest> {
    let mut dir = if start.is_dir() { start.to_path_buf() } else { start.parent()?.to_path_buf() };
    let dir_abs = std::fs::canonicalize(&dir).unwrap_or_else(|_| dir.clone());
    // Windows canonical paths carry a `\\?\` prefix that only confuses messages
    let text = dir_abs.to_string_lossy().to_string();
    dir = PathBuf::from(text.strip_prefix("\\\\?\\").unwrap_or(&text));
    loop {
        let m = dir.join(MANIFEST);
        if m.exists() {
            return Manifest::read(&m).map_err(|e| eprintln!("error: {}", e)).ok();
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Every package reachable from the manifest, name and `src` directory,
/// the root's direct dependencies first. Transitive dependencies are read
/// from each dependency's own manifest; the first package to claim a name
/// wins.
pub fn packages(root: &Manifest) -> Result<Vec<(String, PathBuf)>, String> {
    let mut out: Vec<(String, PathBuf)> = Vec::new();
    let mut queue: Vec<Manifest> = vec![root.clone()];
    let mut qi = 0;
    while qi < queue.len() {
        let m = queue[qi].clone();
        qi += 1;
        for d in &m.deps {
            if out.iter().any(|(n, _)| *n == d.name) {
                continue;
            }
            let dir = if d.path.is_some() { m.dep_dir(d) } else { root.dir.join(MODULES_DIR).join(&d.name) };
            if !dir.exists() {
                return Err(format!("dependency `{}` is not fetched; run `nx fetch` in {}", d.name, root.dir.display()));
            }
            out.push((d.name.clone(), dir.join("src")));
            let sub = dir.join(MANIFEST);
            if sub.exists() {
                queue.push(Manifest::read(&sub)?);
            }
        }
    }
    Ok(out)
}

fn git(args: &[&str], cwd: Option<&Path>) -> Result<String, String> {
    let mut c = Command::new("git");
    c.args(args);
    if let Some(d) = cwd {
        c.current_dir(d);
    }
    let out = c.output().map_err(|e| format!("cannot run git: {}", e))?;
    if !out.status.success() {
        return Err(format!("git {} failed: {}", args.join(" "), String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Clone every git dependency (transitively) into `nexium_modules/` and
/// write `nexium.lock` with the commit each one resolved to.
pub fn fetch(root: &Manifest) -> Result<Vec<String>, String> {
    let modules = root.dir.join(MODULES_DIR);
    let mut lock: Vec<String> = Vec::new();
    let mut done: Vec<String> = Vec::new();
    let mut queue: Vec<Manifest> = vec![root.clone()];
    let mut qi = 0;
    while qi < queue.len() {
        let m = queue[qi].clone();
        qi += 1;
        for d in &m.deps {
            if done.contains(&d.name) {
                continue;
            }
            done.push(d.name.clone());
            let dir = if d.path.is_some() { m.dep_dir(d) } else { modules.join(&d.name) };
            if let Some(url) = &d.git {
                if !dir.exists() {
                    std::fs::create_dir_all(&modules).map_err(|e| format!("cannot create {}: {}", modules.display(), e))?;
                    let mut args = vec!["clone", "--depth", "1", "--quiet"];
                    if let Some(t) = &d.tag {
                        args.extend(["--branch", t.as_str()]);
                    }
                    args.push(url);
                    let dir_s = dir.to_string_lossy().to_string();
                    args.push(&dir_s);
                    git(&args, None)?;
                }
                let commit = git(&["rev-parse", "HEAD"], Some(&dir))?;
                lock.push(format!("{}\t{}\t{}\t{}", d.name, url, d.tag.clone().unwrap_or_default(), commit));
            } else {
                if !dir.exists() {
                    return Err(format!("dependency `{}`: path {} does not exist", d.name, dir.display()));
                }
                lock.push(format!("{}\tpath\t{}\t", d.name, d.path.clone().unwrap_or_default()));
            }
            let sub = dir.join(MANIFEST);
            if sub.exists() {
                queue.push(Manifest::read(&sub)?);
            }
        }
    }
    let mut text = String::from("# nexium.lock: dependency\tsource\ttag\tcommit (written by `nx fetch`)\n");
    for l in &lock {
        text.push_str(l);
        text.push('\n');
    }
    std::fs::write(root.dir.join(LOCK), text).map_err(|e| format!("cannot write {}: {}", LOCK, e))?;
    Ok(lock)
}

/// Create a manifest (and a `main.nx` when the directory has no source yet).
pub fn init(dir: &Path, name: &str) -> Result<Vec<PathBuf>, String> {
    let mut written = Vec::new();
    let m = dir.join(MANIFEST);
    if m.exists() {
        return Err(format!("{} already exists", m.display()));
    }
    let text = format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\n\n[dependencies]\n", name);
    std::fs::write(&m, text).map_err(|e| format!("cannot write {}: {}", m.display(), e))?;
    written.push(m);
    let main = dir.join("main.nx");
    let has_source = std::fs::read_dir(dir).map(|rd| rd.filter_map(|e| e.ok()).any(|e| e.path().extension().map(|x| x == "nx").unwrap_or(false))).unwrap_or(false);
    if !has_source {
        std::fs::write(&main, "fn main() {\n    println(\"hello from a Nexium package\", .{})\n}\n").map_err(|e| format!("cannot write {}: {}", main.display(), e))?;
        written.push(main);
    }
    Ok(written)
}

/// Append a dependency to the manifest text.
pub fn add(manifest_path: &Path, dep: &Dep) -> Result<(), String> {
    let mut text = std::fs::read_to_string(manifest_path).map_err(|e| format!("cannot read {}: {}", manifest_path.display(), e))?;
    let line = match (&dep.git, &dep.path) {
        (Some(url), _) => match &dep.tag {
            Some(t) => format!("{} = {{ git = \"{}\", tag = \"{}\" }}\n", dep.name, url, t),
            None => format!("{} = {{ git = \"{}\" }}\n", dep.name, url),
        },
        (None, Some(p)) => format!("{} = {{ path = \"{}\" }}\n", dep.name, p.replace('\\', "/")),
        _ => return Err("a dependency needs --git URL or --path DIR".into()),
    };
    if let Some(i) = text.find("[dependencies]") {
        let after = i + "[dependencies]".len();
        let nl = text[after..].find('\n').map(|k| after + k + 1).unwrap_or(text.len());
        text.insert_str(nl, &line);
    } else {
        if !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str("\n[dependencies]\n");
        text.push_str(&line);
    }
    std::fs::write(manifest_path, text).map_err(|e| format!("cannot write {}: {}", manifest_path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_subset() {
        let t = parse_toml(
            "# c\n[package]\nname = \"app\" # trailing\nversion = \"0.1.0\"\n\n[dependencies]\ngreet = { git = \"https://x/y\", tag = \"v1\" }\nloc = { path = \"../loc\" }\nlist = [\"a\", \"b\"]\n",
        )
        .unwrap();
        assert_eq!(t["package"][0], ("name".into(), Value::Str("app".into())));
        let deps = &t["dependencies"];
        assert_eq!(deps[0].1.get("tag").unwrap().as_str(), Some("v1"));
        assert_eq!(deps[1].1.get("path").unwrap().as_str(), Some("../loc"));
        assert!(matches!(&deps[2].1, Value::List(v) if v.len() == 2));
    }
}
