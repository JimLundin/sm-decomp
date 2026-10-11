//! Compiler inputs and conservative module discovery for legacy family wrappers.
use crate::{atomic_json, digest};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

fn path(request: &Value, key: &str) -> Result<PathBuf> {
    Ok(fs::canonicalize(request[key].as_str().context(format!("missing {key}"))?)?)
}

pub fn wrapper(request: Value) -> Result<Value> {
    let checkout = path(&request, "checkout")?;
    let run = path(&request, "run")?;
    let sources = checkout.join("crates/port/src");
    let mut available = BTreeMap::new();
    for entry in fs::read_dir(&sources)? {
        let p = entry?.path();
        if p.extension().and_then(|v| v.to_str()) != Some("rs") {
            continue;
        }
        let name = p.file_stem().context("source stem missing")?.to_string_lossy().into_owned();
        if name != "lib" && name != "memory" {
            available.insert(name, p);
        }
    }
    let mut pending: Vec<PathBuf> = request["modules"]
        .as_array()
        .context("modules missing")?
        .iter()
        .map(|v| Ok(fs::canonicalize(v.as_str().context("invalid module")?)?))
        .collect::<Result<_>>()?;
    let mut selected = BTreeSet::new();
    for name in request["imports"].as_array().context("imports missing")? {
        let name = name.as_str().context("invalid import")?;
        pending.push(available.get(name).context("unknown gameplay import")?.clone());
        selected.insert(name.to_owned());
    }
    let mut visited = BTreeSet::new();
    while let Some(p) = pending.pop() {
        if !visited.insert(p.clone()) {
            continue;
        }
        let source = fs::read_to_string(&p)?;
        for word in source.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
            if let Some(module) = available.get(word)
                && selected.insert(word.to_owned())
            {
                pending.push(module.clone());
            }
        }
    }
    let memory = sources.join("memory.rs");
    let mut output = format!("#[path={}] mod canonical;\npub use canonical::*;\n", json!(memory));
    for name in selected {
        output.push_str(&format!("#[path={}] pub mod {name};\n", json!(available[&name])));
    }
    let destination = run.join("memory.rs");
    fs::write(&destination, output)?;
    Ok(json!({"path":destination}))
}

pub(crate) fn dep_paths(text: &str) -> Result<Vec<String>> {
    let joined = text.replace("\\\n", "");
    let rule =
        joined.split_once(": ").context("invalid compiler dependency rule")?.1.lines().next().unwrap_or_default();
    let mut result = Vec::new();
    let mut token = String::new();
    let mut escaped = false;
    for c in rule.chars() {
        if escaped {
            token.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c.is_whitespace() {
            if !token.is_empty() {
                result.push(std::mem::take(&mut token));
            }
        } else {
            token.push(c);
        }
    }
    ensure!(!escaped, "truncated compiler dependency escape");
    if !token.is_empty() {
        result.push(token);
    }
    Ok(result)
}

pub fn dependencies(request: Value) -> Result<Value> {
    let checkout = path(&request, "checkout")?;
    let run = path(&request, "run")?;
    let mut paths = BTreeSet::new();
    for name in dep_paths(&fs::read_to_string(run.join("compiler.d"))?)? {
        let p = Path::new(&name);
        paths.insert(fs::canonicalize(if p.is_absolute() { p.to_path_buf() } else { run.join(p) })?);
    }
    for name in request["extra"].as_array().context("extra dependencies missing")? {
        let p = Path::new(name.as_str().context("invalid dependency")?);
        paths.insert(fs::canonicalize(if p.is_absolute() { p.to_path_buf() } else { checkout.join(p) })?);
    }
    let mut result = BTreeMap::new();
    for (index, p) in paths.into_iter().enumerate() {
        if p.starts_with(&run) {
            continue;
        }
        let key = p.strip_prefix(&checkout).unwrap_or(&p).to_string_lossy().into_owned();
        result.insert(key, digest(&p)?);
        fs::copy(&p, run.join(format!("dependency-{index:03}.rs")))?;
    }
    // Keep the tool identity alongside the source inventory, so changing the
    // discovery algorithm cannot silently admit an older dependency contract.
    let mut tools = BTreeMap::new();
    for (name, compiled) in [
        ("main.rs", include_str!("main.rs")),
        ("inputs.rs", include_str!("inputs.rs")),
        ("builds.rs", include_str!("builds.rs")),
    ] {
        let key = format!("crates/pipeline/src/{name}");
        let hash = digest(&checkout.join(&key))?;
        ensure!(
            hash == format!("{:x}", <sha2::Sha256 as sha2::Digest>::digest(compiled.as_bytes())),
            "pipeline executable is stale: {name}"
        );
        result.insert(key.clone(), hash.clone());
        tools.insert(key, hash);
        fs::copy(checkout.join(format!("crates/pipeline/src/{name}")), run.join(format!("pipeline-{name}")))?;
    }
    atomic_json(
        &run.join("pipeline-inputs.json"),
        &json!({"binary_sha256":digest(&std::env::current_exe()?)?, "sources":tools}),
    )?;
    Ok(json!(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_compiler_rules_with_spaces_and_continuations() {
        assert_eq!(
            dep_paths("out: /a/source.rs /a/with\\ space.rs \\\n /last.rs\nother: /ignored.rs\n").unwrap(),
            vec!["/a/source.rs", "/a/with space.rs", "/last.rs"]
        );
        assert!(dep_paths("missing rule").is_err());
    }
}
