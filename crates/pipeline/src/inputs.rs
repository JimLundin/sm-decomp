//! Compiler inputs and conservative module discovery for legacy family wrappers.
#[path = "bounded_inputs.rs"]
pub(crate) mod bounded;
use anyhow::{Context, Result, ensure};
use bounded::digest;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

// This original captured Ridley log is 71,595,494 bytes. Its exact identity,
// not an arbitrary family filename, selects the bounded streaming exception.
pub(crate) const CAPTURED_RIDLEY_EVENT: &str = "/home/jim/sm-decomp/out/port/ridley-subsystem/recovery-inventory/1791858734336387661-2/ridley-subsystem-events.jsonl";
const CAPTURED_RIDLEY_EVENT_SHA256: &str = "f1f286cae8090e5f23f8e1e9b0a20fcc4de41d91766f349c9a0be7518eec1f35";
const CAPTURED_RIDLEY_EVENT_BYTES: u64 = 128 * 1024 * 1024;
const CAPTURED_RIDLEY_SNAPSHOT: &str = "dependency-ridley-subsystem-events.jsonl";

pub(crate) fn captured_ridley_manifest(manifest: &crate::family_model::Manifest) -> bool {
    manifest.family == "ridley-subsystem-captured"
        && manifest.callback_adapter.as_deref() == Some("ridley-subsystem")
        && manifest.rust_dependencies.iter().any(|path| path == Path::new(CAPTURED_RIDLEY_EVENT))
}

pub(crate) fn captured_ridley_event_digest(path: &Path) -> Result<String> {
    let hash = bounded::digest_limit(path, CAPTURED_RIDLEY_EVENT_BYTES)?;
    ensure!(hash == CAPTURED_RIDLEY_EVENT_SHA256, "original captured Ridley event input changed");
    Ok(hash)
}

/// A family run may retain this one larger snapshot only for the exact original
/// captured declaration, and its bytes must equal the reviewed event asset.
pub(crate) fn artifact_digest(run: &Path, path: &Path) -> Result<String> {
    if path == run.join(CAPTURED_RIDLEY_SNAPSHOT) {
        let manifest = crate::family::object(&run.join("manifest.toml"))?;
        ensure!(captured_ridley_manifest(&manifest), "large Ridley snapshot has no reviewed declaration");
        captured_ridley_event_digest(path)
    } else {
        digest(path)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WrapperRequest {
    pub checkout: PathBuf,
    pub run: PathBuf,
    pub modules: Vec<PathBuf>,
    pub imports: Vec<String>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DependenciesRequest {
    pub checkout: PathBuf,
    pub run: PathBuf,
    pub extra: Vec<PathBuf>,
}
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct WrapperResult {
    pub path: PathBuf,
}

/// The installed confined family adapter places every gameplay module at the
/// crate root. A selected candidate must never be declared both under a memory
/// wrapper and as a separate `family` module: those create nominally distinct
/// copies of types in its dependency closure.
pub(crate) fn canonical_sources(checkout: &Path, modules: &[PathBuf], imports: &[String]) -> Result<Vec<PathBuf>> {
    bounded::limited(|| canonical_sources_bounded(checkout, modules, imports))
}

pub(crate) fn canonical_sources_until(
    checkout: &Path,
    modules: &[PathBuf],
    imports: &[String],
    deadline: u64,
    max_bytes: u64,
) -> Result<Vec<PathBuf>> {
    bounded::until(deadline, max_bytes, || canonical_sources_bounded(checkout, modules, imports))
}

fn canonical_sources_bounded(checkout: &Path, modules: &[PathBuf], imports: &[String]) -> Result<Vec<PathBuf>> {
    bounded::safe(checkout)?;
    let checkout = fs::canonicalize(checkout)?;
    let sources = checkout.join("crates/port/src");
    let mut available = BTreeMap::new();
    ensure!(modules.len() <= 4096 && imports.len() <= 4096, "canonical module seed count exceeded");
    for path in bounded::entries(&sources)? {
        ensure!(!path.is_symlink(), "gameplay source alias");
        if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            ensure!(available.len() < 4096, "canonical module inventory exceeded");
            let name = path.file_stem().context("gameplay source stem missing")?.to_string_lossy().into_owned();
            if name != "lib" && name != "memory" {
                available.insert(name, path);
            }
        }
    }
    let mut pending = modules.to_vec();
    for name in imports {
        pending.push(available.get(name).context("unknown canonical gameplay import")?.clone());
    }
    let mut selected = BTreeSet::new();
    while let Some(path) = pending.pop() {
        bounded::check()?;
        bounded::safe(&path)?;
        let path = fs::canonicalize(path)?;
        ensure!(
            path.parent() == Some(sources.as_path()) && path.extension().is_some_and(|value| value == "rs"),
            "canonical family modules must be direct gameplay source files"
        );
        if !selected.insert(path.clone()) {
            continue;
        }
        for word in bounded::text(&path)?.split(|value: char| !(value.is_ascii_alphanumeric() || value == '_')) {
            if let Some(dependency) = available.get(word)
                && !selected.contains(dependency)
                && !pending.contains(dependency)
            {
                pending.push(dependency.clone());
            }
        }
    }
    selected.insert(sources.join("memory.rs"));
    bounded::safe(&sources.join("memory.rs"))?;
    ensure!(sources.join("memory.rs").symlink_metadata()?.is_file(), "canonical memory source missing or special");
    Ok(selected.into_iter().collect())
}

pub(crate) fn canonical_wrapper(checkout: &Path, run: &Path, sources: &[PathBuf]) -> Result<WrapperResult> {
    bounded::limited(|| canonical_wrapper_bounded(checkout, run, sources))
}

fn canonical_wrapper_bounded(checkout: &Path, run: &Path, sources: &[PathBuf]) -> Result<WrapperResult> {
    ensure!(sources.len() <= 4096, "canonical wrapper source inventory exceeded");
    let mut output = String::new();
    let mut names = BTreeSet::new();
    for path in sources {
        bounded::check()?;
        let relative = path.strip_prefix(checkout)?;
        let name = path.file_stem().context("canonical module stem missing")?.to_str().context("module name UTF-8")?;
        ensure!(crate::family::identifier(name) && names.insert(name), "duplicate canonical module placement");
        output.push_str(&format!("#[path={:?}] pub mod {name};\n", format!("/inputs/rust/{}", relative.display())));
    }
    ensure!(names.contains("memory"), "canonical memory source missing");
    output.push_str("pub use memory::{WRAM_SIZE, Wram};\n");
    let path = run.join("memory.rs");
    immutable_bytes(&path, output.as_bytes())?;
    Ok(WrapperResult { path })
}

/// Controller-owned immutable publication. A partial write has only the
/// identified pending name; response recovery preserves final file metadata.
pub(crate) fn immutable_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    bounded::limited(|| immutable_bytes_bounded(path, bytes))
}

fn immutable_bytes_bounded(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    ensure!(path.is_absolute(), "immutable input requires absolute path");
    bounded::safe(path)?;
    ensure!(bytes.len() as u64 <= bounded::MAX_FILE_BYTES, "immutable native file byte extent exceeded");
    if path.exists() {
        ensure!(
            !path.is_symlink() && bounded::read(path)? == bytes,
            "immutable native file changed: {}",
            path.display()
        );
        return Ok(());
    }
    let pending = path.with_file_name(format!(
        "{}.pending",
        path.file_name().context("immutable file name missing")?.to_string_lossy()
    ));
    if pending.exists() {
        ensure!(
            !pending.is_symlink() && pending.is_file() && pending.metadata()?.len() <= bytes.len() as u64,
            "immutable pending extent changed"
        );
        fs::remove_file(&pending)?;
    }
    use std::os::unix::fs::OpenOptionsExt;
    bounded::account(bytes.len() as u64)?;
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(&pending)?;
    for block in bytes.chunks(64 * 1024) {
        bounded::check()?;
        file.write_all(block)?;
    }
    file.sync_all()?;
    bounded::check()?;
    fs::rename(&pending, path)?;
    fs::File::open(path.parent().context("immutable parent missing")?)?.sync_all()?;
    Ok(())
}

pub fn wrapper(request: WrapperRequest) -> Result<WrapperResult> {
    bounded::limited(|| wrapper_bounded(request))
}

fn wrapper_bounded(request: WrapperRequest) -> Result<WrapperResult> {
    ensure!(request.modules.len() <= 4096 && request.imports.len() <= 4096, "wrapper source seed inventory exceeded");
    let checkout = fs::canonicalize(request.checkout)?;
    let run = fs::canonicalize(request.run)?;
    let sources = checkout.join("crates/port/src");
    let mut available = BTreeMap::new();
    for p in bounded::entries(&sources)? {
        if p.extension().and_then(|v| v.to_str()) != Some("rs") {
            continue;
        }
        let name = p.file_stem().context("source stem missing")?.to_string_lossy().into_owned();
        if name != "lib" && name != "memory" {
            available.insert(name, p);
        }
    }
    let mut pending: Vec<PathBuf> = request.modules.iter().map(fs::canonicalize).collect::<std::io::Result<_>>()?;
    let mut selected = BTreeSet::new();
    for name in request.imports {
        pending.push(available.get(&name).context("unknown gameplay import")?.clone());
        selected.insert(name);
    }
    let mut visited = BTreeSet::new();
    while let Some(p) = pending.pop() {
        bounded::check()?;
        if !visited.insert(p.clone()) {
            continue;
        }
        let source = bounded::text(&p)?;
        for word in source.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
            if let Some(module) = available.get(word)
                && selected.insert(word.to_owned())
            {
                pending.push(module.clone());
            }
        }
    }
    let memory = sources.join("memory.rs");
    let mut output = format!(
        "#[path={:?}] mod canonical;\npub use canonical::*;\n",
        memory.to_str().context("non-UTF8 source path")?
    );
    for name in selected {
        output.push_str(&format!(
            "#[path={:?}] pub mod {name};\n",
            available[&name].to_str().context("non-UTF8 source path")?
        ));
    }
    let destination = run.join("memory.rs");
    bounded::write(&destination, output.as_bytes())?;
    Ok(WrapperResult { path: destination })
}

pub(crate) fn dep_paths(text: &str) -> Result<Vec<String>> {
    bounded::check()?;
    ensure!(text.len() as u64 <= bounded::MAX_DOCUMENT_BYTES, "compiler dependency rule extent exceeded");
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
                ensure!(result.len() <= 65536, "compiler dependency path count exceeded");
                bounded::check()?;
            }
        } else {
            token.push(c);
        }
    }
    ensure!(!escaped, "truncated compiler dependency escape");
    if !token.is_empty() {
        result.push(token);
    }
    ensure!(result.len() <= 65536, "compiler dependency path count exceeded");
    bounded::check()?;
    Ok(result)
}

pub fn dependencies(request: DependenciesRequest) -> Result<BTreeMap<String, String>> {
    dependency_inventory(request, false, false)
}

pub(crate) fn confined_dependencies(request: DependenciesRequest) -> Result<BTreeMap<String, String>> {
    dependency_inventory(request, true, false)
}

pub(crate) fn captured_ridley_dependencies(
    request: DependenciesRequest,
    manifest: &crate::family_model::Manifest,
    confined: bool,
) -> Result<BTreeMap<String, String>> {
    ensure!(
        captured_ridley_manifest(manifest) && request.extra == manifest.rust_dependencies,
        "captured Ridley dependency profile differs from its original declaration"
    );
    dependency_inventory(request, confined, true)
}

fn dependency_inventory(
    request: DependenciesRequest,
    confined: bool,
    captured_ridley: bool,
) -> Result<BTreeMap<String, String>> {
    bounded::limited(|| dependency_inventory_bounded(request, confined, captured_ridley))
}

fn dependency_inventory_bounded(
    request: DependenciesRequest,
    confined: bool,
    captured_ridley: bool,
) -> Result<BTreeMap<String, String>> {
    ensure!(
        request.extra.len() <= if captured_ridley { 8192 } else { 4096 },
        "compiler extra input inventory exceeded"
    );
    let checkout = fs::canonicalize(request.checkout)?;
    let run = fs::canonicalize(request.run)?;
    let mut paths = BTreeSet::new();
    for name in dep_paths(&bounded::text(run.join("compiler.d"))?)? {
        bounded::check()?;
        let p = Path::new(&name);
        let path = if confined {
            if let Ok(relative) = p.strip_prefix("/inputs/rust") {
                ensure!(
                    relative.components().all(|part| matches!(part, std::path::Component::Normal(_))),
                    "confined compiler dependency alias"
                );
                checkout.join(relative)
            } else if let Ok(relative) = p.strip_prefix("/inputs/generated") {
                ensure!(
                    relative.components().all(|part| matches!(part, std::path::Component::Normal(_))),
                    "generated compiler dependency alias"
                );
                run.join(relative)
            } else {
                anyhow::bail!("undeclared confined compiler dependency: {name}");
            }
        } else if p.is_absolute() {
            p.to_path_buf()
        } else {
            run.join(p)
        };
        bounded::safe(&path)?;
        paths.insert(fs::canonicalize(path)?);
    }
    for p in &request.extra {
        bounded::check()?;
        paths.insert(fs::canonicalize(if p.is_absolute() { p.to_path_buf() } else { checkout.join(p) })?);
    }
    let mut result = BTreeMap::new();
    for (index, p) in paths.into_iter().enumerate() {
        bounded::check()?;
        if p.starts_with(&run) {
            continue;
        }
        let key = p.strip_prefix(&checkout).unwrap_or(&p).to_string_lossy().into_owned();
        if captured_ridley && p == Path::new(CAPTURED_RIDLEY_EVENT) {
            let hash = captured_ridley_event_digest(&p)?;
            bounded::immutable_copy(&p, &run.join(CAPTURED_RIDLEY_SNAPSHOT), &hash, CAPTURED_RIDLEY_EVENT_BYTES)?;
            result.insert(key, hash);
        } else {
            result.insert(key, digest(&p)?);
            snapshot(&p, &run.join(format!("dependency-{index:03}.rs")), confined)?;
        }
    }
    // Keep the tool identity alongside the source inventory, so changing the
    // discovery algorithm cannot silently admit an older dependency contract.
    let mut tools = BTreeMap::new();
    for (name, compiled) in [
        ("main.rs", include_str!("main.rs")),
        ("inputs.rs", include_str!("inputs.rs")),
        ("bounded_inputs.rs", include_str!("bounded_inputs.rs")),
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
        snapshot(
            &checkout.join(format!("crates/pipeline/src/{name}")),
            &run.join(format!("pipeline-{name}")),
            confined,
        )?;
    }
    #[derive(Serialize)]
    struct PipelineInputs<'a> {
        binary_sha256: &'a str,
        sources: BTreeMap<String, String>,
    }
    let path = run.join("pipeline-inputs.toml");
    let bytes = toml::to_string(&PipelineInputs { binary_sha256: crate::binary_digest()?, sources: tools })?;
    if confined {
        immutable_bytes(&path, bytes.as_bytes())?;
    } else {
        bounded::write(&path, bytes.as_bytes())?;
    }
    Ok(result)
}

fn snapshot(source: &Path, destination: &Path, immutable: bool) -> Result<()> {
    if immutable {
        immutable_bytes(destination, &bounded::read(source)?)?;
    } else {
        bounded::write(destination, &bounded::read(source)?)?;
    }
    Ok(())
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

    #[test]
    fn canonical_discovery_deduplicates_cycles_and_repeated_tokens_with_one_budget() -> Result<()> {
        let directory = bounded::tests::Directory::new()?;
        let sources = directory.0.join("crates/port/src");
        fs::create_dir_all(&sources)?;
        fs::write(sources.join("memory.rs"), "pub struct Wram;")?;
        let candidate = "dependency ".repeat(8192);
        let dependency = "candidate ".repeat(8192);
        fs::write(sources.join("candidate.rs"), &candidate)?;
        fs::write(sources.join("dependency.rs"), &dependency)?;
        let expected = vec![sources.join("candidate.rs"), sources.join("dependency.rs"), sources.join("memory.rs")];
        let cap = candidate.len() as u64 + dependency.len() as u64;
        let result = canonical_sources_until(
            &directory.0,
            &[sources.join("candidate.rs")],
            &[],
            bounded::now_unix_ms()? + 1000,
            cap,
        )?;
        assert_eq!(result, expected);
        assert!(
            canonical_sources_until(
                &directory.0,
                &[sources.join("candidate.rs")],
                &[],
                bounded::now_unix_ms()? + 1000,
                cap - 1
            )
            .is_err()
        );
        assert!(
            canonical_sources_until(&directory.0.join("absent"), &[], &[], 0, 1024)
                .unwrap_err()
                .to_string()
                .contains("deadline")
        );
        Ok(())
    }

    #[test]
    fn canonical_discovery_rejects_module_symlink() -> Result<()> {
        let directory = bounded::tests::Directory::new()?;
        let sources = directory.0.join("crates/port/src");
        fs::create_dir_all(&sources)?;
        fs::write(sources.join("candidate.rs"), "pub fn Step() {}")?;
        fs::write(sources.join("memory.rs"), "pub struct Wram;")?;
        std::os::unix::fs::symlink(sources.join("candidate.rs"), sources.join("alias.rs"))?;
        assert!(canonical_sources(&directory.0, &[sources.join("alias.rs")], &[]).is_err());
        Ok(())
    }

    #[test]
    fn canonical_wrapper_declares_the_candidate_and_dependencies_once() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "sm-canonical-inputs-{}-{}",
            std::process::id(),
            crate::family::unique_id()
        ));
        let sources = root.join("crates/port/src");
        let run = root.join("run");
        fs::create_dir_all(&sources)?;
        fs::create_dir(&run)?;
        fs::write(sources.join("memory.rs"), "pub struct Wram; pub const WRAM_SIZE:usize=0x20000;")?;
        fs::write(sources.join("candidate.rs"), "use crate::{dependency,Wram};")?;
        fs::write(sources.join("dependency.rs"), "use crate::Wram;")?;
        let paths = canonical_sources(&root, &[sources.join("candidate.rs")], &[])?;
        let wrapper = canonical_wrapper(&root, &run, &paths)?;
        let text = bounded::text(&wrapper.path)?;
        assert_eq!(text.matches("pub mod candidate;").count(), 1);
        assert_eq!(text.matches("pub mod dependency;").count(), 1);
        assert_eq!(text.matches("pub mod memory;").count(), 1);
        let modified = wrapper.path.metadata()?.modified()?;
        canonical_wrapper(&root, &run, &paths)?;
        assert_eq!(wrapper.path.metadata()?.modified()?, modified);
        assert!(canonical_wrapper(&root, &run, &[sources.join("memory.rs")]).is_err());
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn immutable_input_recovers_partial_publication_and_rejects_changed_final_bytes() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "sm-immutable-inputs-{}-{}",
            std::process::id(),
            crate::family::unique_id()
        ));
        fs::create_dir(&root)?;
        fs::write(root.join("source.pending"), "par")?;
        let file = root.join("source");
        immutable_bytes(&file, b"complete")?;
        let modified = file.metadata()?.modified()?;
        immutable_bytes(&file, b"complete")?;
        assert_eq!(file.metadata()?.modified()?, modified);
        assert!(immutable_bytes(&file, b"different").is_err());
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
