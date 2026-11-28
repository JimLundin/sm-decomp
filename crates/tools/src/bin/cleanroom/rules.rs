mod doc;
mod lex;
mod manifest;
mod shape;
mod source;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use manifest::{Approval, Checked, WorkspaceFile, normal};
use source::Include;

pub struct Finding {
    pub path: String,
    pub tag: &'static str,
    pub detail: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}: {}", self.path, self.tag, self.detail)
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Level {
    Plain(String),
    Table { level: String },
}

#[derive(Deserialize, Default)]
struct LintTables {
    #[serde(default)]
    rust: BTreeMap<String, Level>,
    #[serde(default)]
    clippy: BTreeMap<String, Level>,
}

#[derive(Deserialize)]
struct TrustedWorkspace {
    #[serde(default)]
    lints: LintTables,
}

#[derive(Deserialize)]
struct Trusted {
    workspace: TrustedWorkspace,
}

pub struct Policy {
    denied: BTreeSet<String>,
    approved: BTreeMap<String, Approval>,
}

impl Policy {
    pub fn load(trusted: &Path) -> Result<Self, String> {
        let manifest = read(&trusted.join("Cargo.toml"))?;
        let lints = toml::from_str::<Trusted>(&manifest)
            .map_err(|e| format!("{}: {e}", trusted.join("Cargo.toml").display()))?
            .workspace
            .lints;
        let pinned = toml::from_str::<WorkspaceFile>(&manifest)
            .map_err(|e| e.to_string())?
            .workspace
            .dependencies;
        let mut denied = BTreeSet::from(["warnings".to_owned()]);
        for (prefix, table) in [("", &lints.rust), ("clippy::", &lints.clippy)] {
            for (name, level) in table {
                let level = match level {
                    Level::Plain(level) | Level::Table { level } => level,
                };
                if matches!(level.as_str(), "deny" | "forbid") {
                    denied.insert(format!("{prefix}{name}"));
                }
            }
        }
        let mut approved = BTreeMap::new();
        for line in read(&trusted.join("tools/cleanroom/allowed_deps.txt"))?.lines() {
            let fields: Vec<&str> = line
                .split('#')
                .next()
                .unwrap_or_default()
                .split_whitespace()
                .collect();
            let [name, kinds, features] = fields.as_slice() else {
                if fields.is_empty() {
                    continue;
                }
                return Err(format!("allowed_deps.txt: `{line}` is not three columns"));
            };
            let version = pinned
                .get(*name)
                .and_then(manifest::Dep::version)
                .filter(|v| v.starts_with('='))
                .ok_or_else(|| format!("{name} needs an exact workspace version"))?;
            approved.insert(
                (*name).to_owned(),
                Approval {
                    version,
                    kinds: kinds.split(',').map(str::to_owned).collect(),
                    features: features
                        .split(',')
                        .filter(|f| *f != "-")
                        .map(str::to_owned)
                        .collect(),
                },
            );
        }
        Ok(Self { denied, approved })
    }
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn walk(dir: &Path, rel: &str, out: &mut Vec<(String, bool)>) -> Result<(), String> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .and_then(Iterator::collect::<Result<Vec<_>, _>>)
        .map_err(|e| format!("{}: {e}", dir.display()))?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = if rel.is_empty() {
            name
        } else {
            format!("{rel}/{name}")
        };
        let kind = entry
            .file_type()
            .map_err(|e| format!("{}: {e}", entry.path().display()))?;
        if kind.is_dir() {
            walk(&entry.path(), &path, out)?;
        } else {
            out.push((path, kind.is_symlink()));
        }
    }
    Ok(())
}

fn is_test_file(path: &str) -> bool {
    path.starts_with("tests/") || path.contains("/tests/") || path.ends_with("/tests.rs")
}

fn include_allowed(target: &Path, text: bool, domain: Option<&str>) -> bool {
    let Some(target) = target.to_str() else {
        return false;
    };
    if let Some(domain) = domain {
        if target == format!("scenarios/vectors/{domain}.toml") {
            return true;
        }
        if text && target == format!("docs/behaviours/{domain}.md") {
            return true;
        }
    }
    text && Path::new(target).extension().is_some_and(|e| e == "rs")
        && target
            .strip_prefix("crates/clean/")
            .is_some_and(|rest| rest.contains("/src/"))
}

pub fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }
    let mut files = Vec::new();
    walk(dir, "", &mut files)?;
    out.extend(
        files
            .into_iter()
            .filter(|(p, link)| !link && Path::new(p).extension().is_some_and(|e| e == "rs"))
            .map(|(p, _)| dir.join(p)),
    );
    Ok(())
}

pub fn reads_docs(text: &str) -> bool {
    let toks = lex::tokens(text, 0);
    let toks = lex::Toks(&toks);
    (0..toks.len()).any(|i| {
        matches!(toks.text(i), "include" | "include_str" | "include_bytes")
            && toks.punct(i.saturating_add(1), "!")
            && toks.is(i.saturating_add(2), lex::Kind::Open)
            && toks
                .get(i.saturating_add(3))
                .is_some_and(|t| t.kind == lex::Kind::Lit && t.text.contains("docs/"))
    })
}

pub fn check(root: &Path, policy: &Policy) -> Result<Vec<Finding>, String> {
    let clean = root.join("crates/clean");
    let mut out = Vec::new();
    if !clean.is_dir() {
        return Ok(out);
    }
    let mut crates: Vec<_> = fs::read_dir(&clean)
        .and_then(Iterator::collect::<Result<Vec<_>, _>>)
        .map_err(|e| format!("{}: {e}", clean.display()))?;
    crates.sort_by_key(fs::DirEntry::file_name);
    let workspace = fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default();
    for entry in crates {
        if entry.path().is_dir() {
            let name = entry.file_name().to_string_lossy().into_owned();
            check_crate(root, &name, &workspace, policy, &mut out)?;
        }
    }
    Ok(out)
}

fn check_crate(
    root: &Path,
    name: &str,
    workspace: &str,
    policy: &Policy,
    out: &mut Vec<Finding>,
) -> Result<(), String> {
    let rel = format!("crates/clean/{name}");
    let dir = root.join(&rel);
    let domain = (name != "types").then(|| name.replace('_', "-"));
    let mut push = |path: String, tag: &'static str, detail: String| {
        out.push(Finding { path, tag, detail });
    };
    let mut files = Vec::new();
    walk(&dir, "", &mut files)?;
    if files.iter().any(|(p, link)| p == "Cargo.toml" && !link) {
        let mut allowed_paths = BTreeSet::from([PathBuf::from("crates/clean/types")]);
        if domain.is_some() {
            allowed_paths.insert(PathBuf::from(&rel));
        }
        let checked = Checked {
            crate_dir: Path::new(&rel),
            root: Path::new(""),
            allowed_paths: &allowed_paths,
            approved: &policy.approved,
        };
        for (tag, detail) in checked.check(&read(&dir.join("Cargo.toml"))?, workspace) {
            push(format!("{rel}/Cargo.toml"), tag, detail);
        }
    } else {
        push(rel.clone(), "extra-file", "no Cargo.toml".to_owned());
    }
    let mut library: Vec<(String, String)> = Vec::new();
    let mut sources: Vec<(String, Vec<lex::Token>)> = Vec::new();
    for (path, link) in &files {
        let source_dir = path.starts_with("src/") || path.starts_with("tests/");
        if path != "Cargo.toml" && !source_dir {
            push(rel.clone(), "extra-file", path.clone());
        }
        if *link {
            push(rel.clone(), "extra-file", format!("{path} is a symlink"));
            continue;
        }
        if !source_dir || Path::new(path).extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let text = read(&dir.join(path))?;
        let (toks, comments) = lex::lex(&text, 0);
        let mut found = source::check(lex::Toks(&toks), is_test_file(path), &policy.denied);
        doc::check(lex::Toks(&toks), &comments, &mut found.hits);
        let file = format!("{rel}/{path}");
        for hit in found.hits {
            push(file.clone(), hit.tag, hit.detail);
        }
        for include in found.includes {
            match include {
                Include::Literal { path: lit, text } => {
                    let parent = Path::new(&file).parent().unwrap_or_else(|| Path::new(""));
                    let allowed = normal(&parent.join(&lit))
                        .is_some_and(|target| include_allowed(&target, text, domain.as_deref()));
                    if !allowed {
                        push(
                            file.clone(),
                            "include-path",
                            format!(
                                "\"{lit}\" is not \"{}\"'s vector file, its spec or a clean crate's source",
                                domain.as_deref().unwrap_or_default()
                            ),
                        );
                    }
                }
                Include::Other(call) => {
                    push(
                        file.clone(),
                        "include-path",
                        format!("{call} is not a literal path"),
                    );
                }
            }
        }
        sources.push((file, toks));
        if let Some(lib) = path
            .strip_prefix("src/")
            .filter(|p| !is_test_file(&format!("/{p}")))
        {
            library.push((lib.to_owned(), text));
        }
    }
    for (file, hit) in doc::macros(&sources) {
        push(file, hit.tag, hit.detail);
    }
    let modules: Vec<String> = library.iter().map(|(p, _)| shape::module_of(p)).collect();
    let toks: Vec<lex::Token> = library
        .iter()
        .enumerate()
        .flat_map(|(i, (_, text))| lex::tokens(text, i))
        .collect();
    for found in shape::check(lex::Toks(&toks), &modules) {
        let path = library.get(found.file).map_or("", |(p, _)| p.as_str());
        push(format!("{rel}/src/{path}"), found.tag, found.detail);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
