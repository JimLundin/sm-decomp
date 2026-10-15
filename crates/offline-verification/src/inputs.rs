//! Compiler inputs and conservative module discovery for Offline family wrappers.
use anyhow::{Context, Result, ensure};
use bounded::digest;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use tool_execution::bounded;

thread_local! {
    static HOST: RefCell<Option<crate::HostInputs>> = const { RefCell::new(None) };
}

/// Install trusted producer configuration for this operation, restoring a
/// containing operation on every return path, including unwinding.
pub(crate) fn with_host<T>(host: &crate::HostInputs, action: impl FnOnce() -> Result<T>) -> Result<T> {
    struct Restore(Option<crate::HostInputs>);
    impl Drop for Restore {
        fn drop(&mut self) {
            HOST.with(|current| *current.borrow_mut() = self.0.take());
        }
    }
    let _restore = Restore(HOST.with(|current| current.borrow_mut().replace(host.clone())));
    action()
}

pub(crate) fn host_inputs() -> Result<crate::HostInputs> {
    HOST.with(|current| current.borrow().clone().context("verification host inputs were not supplied"))
}

pub(crate) fn required_sources(host: &crate::HostInputs) -> Vec<(&'static str, &'static str)> {
    host.sources()
        .iter()
        .chain(crate::IMPLEMENTATION_SOURCES)
        .chain(decomp_source::IMPLEMENTATION_SOURCES)
        .chain(tool_execution::IMPLEMENTATION_SOURCES)
        .chain(translation_execution::compiler_runtime::IMPLEMENTATION_SOURCES)
        .copied()
        .collect()
}

fn source_snapshot(name: &str) -> String {
    format!("sources/{name}")
}

/// One conservative implementation contract is used by every recipe. The
/// application cannot omit the verifier, source or execution owner's inputs.
pub(crate) fn implementation_sources(checkout: &Path, run: &Path, immutable: bool) -> Result<BTreeMap<String, String>> {
    use sha2::{Digest, Sha256};
    let host = host_inputs()?;
    let mut sources = BTreeMap::new();
    for (name, compiled) in required_sources(&host) {
        let path = checkout.join(name);
        let hash = digest(&path)?;
        ensure!(hash == format!("{:x}", Sha256::digest(compiled.as_bytes())), "stale verifier executable: {name}");
        let destination = run.join(source_snapshot(name));
        fs::create_dir_all(destination.parent().context("source snapshot parent missing")?)?;
        snapshot(&path, &destination, immutable)?;
        sources.insert(name.into(), hash);
    }
    Ok(sources)
}

/// Authenticate retained producer provenance and every required source stamp.
/// The producer hash belongs to the retained run, not the current executable.
pub(crate) fn authenticate_implementation(
    run: &Path,
    implementation: &crate::family_model::Implementation,
    artifacts: &BTreeMap<String, String>,
    reader: &mut crate::admission_reader::Window<'_, '_>,
) -> Result<()> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ProducerInputs {
        binary_sha256: String,
        sources: BTreeMap<String, String>,
    }
    let retained: ProducerInputs = reader.object(run.join("pipeline-inputs.toml"))?;
    ensure!(
        retained.binary_sha256.len() == 64 && retained.binary_sha256.bytes().all(|b| b.is_ascii_hexdigit()),
        "retained producer identity is invalid"
    );
    let mut required: Vec<_> = crate::IMPLEMENTATION_SOURCES
        .iter()
        .chain(decomp_source::IMPLEMENTATION_SOURCES)
        .chain(tool_execution::IMPLEMENTATION_SOURCES)
        .chain(translation_execution::compiler_runtime::IMPLEMENTATION_SOURCES)
        .map(|(name, _)| *name)
        .collect();
    if let Some(host) = HOST.with(|current| current.borrow().clone()) {
        required.extend(host.sources().iter().map(|(name, _)| *name));
    }
    for name in required {
        ensure!(retained.sources.contains_key(name), "required verifier input missing: {name}");
    }
    for (name, hash) in &retained.sources {
        ensure!(
            implementation.rust_dependencies.get(name) == Some(hash)
                && implementation.harness_sources.get(name) == Some(hash)
                && artifacts.get(&source_snapshot(name)) == Some(hash),
            "retained verifier source provenance differs: {name}"
        );
    }
    Ok(())
}

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
        let manifest = crate::fixture_format::read(&run.join("manifest.smf"))?;
        ensure!(captured_ridley_manifest(&manifest), "large Ridley snapshot has no reviewed declaration");
        captured_ridley_event_digest(path)
    } else {
        digest(path)
    }
}

use crate::authoring::{DependenciesRequest, WrapperRequest, WrapperResult};

/// The installed confined family adapter places every gameplay module at the
/// crate root. A selected candidate must never be declared both under a memory
/// wrapper and as a separate `family` module: those create nominally distinct
/// copies of types in its dependency closure.
pub(crate) fn canonical_sources(checkout: &Path, modules: &[PathBuf], imports: &[String]) -> Result<Vec<PathBuf>> {
    decomp_source::context::limited(|| canonical_sources_bounded(checkout, modules, imports))
}

#[cfg(test)]
fn canonical_sources_until(
    checkout: &Path,
    modules: &[PathBuf],
    imports: &[String],
    deadline: u64,
    max_bytes: u64,
) -> Result<Vec<PathBuf>> {
    decomp_source::context::until(deadline, max_bytes, || canonical_sources_bounded(checkout, modules, imports))
}

fn canonical_sources_bounded(checkout: &Path, modules: &[PathBuf], imports: &[String]) -> Result<Vec<PathBuf>> {
    discover_sources_bounded(checkout, modules, imports, true)
}

fn discover_sources_bounded(
    checkout: &Path,
    modules: &[PathBuf],
    imports: &[String],
    direct_only: bool,
) -> Result<Vec<PathBuf>> {
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
    pending.push(sources.join("memory.rs"));
    for name in imports {
        pending.push(available.get(name).context("unknown canonical gameplay import")?.clone());
    }
    let mut selected = BTreeSet::new();
    while let Some(path) = pending.pop() {
        bounded::check()?;
        bounded::safe(&path)?;
        let path = fs::canonicalize(path)?;
        ensure!(
            (!direct_only || path.parent() == Some(sources.as_path()))
                && path.extension().is_some_and(|value| value == "rs"),
            "canonical family modules must be direct gameplay source files"
        );
        if !selected.insert(path.clone()) {
            continue;
        }
        let facts = crate::source_facts::scan(&path, &bounded::read(&path)?)?;
        facts.require_complete()?;
        for word in facts.identifiers() {
            if let Some(dependency) = available.get(word)
                && !selected.contains(dependency)
                && !pending.contains(dependency)
            {
                pending.push(dependency.clone());
            }
        }
        pending.extend(facts.file_dependencies().iter().filter(|path| !selected.contains(*path)).cloned());
    }
    selected.insert(sources.join("memory.rs"));
    bounded::safe(&sources.join("memory.rs"))?;
    ensure!(sources.join("memory.rs").symlink_metadata()?.is_file(), "canonical memory source missing or special");
    Ok(selected.into_iter().collect())
}

pub(crate) fn canonical_wrapper(checkout: &Path, run: &Path, sources: &[PathBuf]) -> Result<WrapperResult> {
    decomp_source::context::limited(|| canonical_wrapper_bounded(checkout, run, sources))
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

use tool_execution::bounded::immutable_bytes;

pub fn wrapper(request: WrapperRequest) -> Result<WrapperResult> {
    decomp_source::context::limited(|| wrapper_bounded(request))
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
    pending.push(sources.join("memory.rs"));
    let mut selected = BTreeSet::new();
    for name in request.imports {
        pending.push(available.get(&name).context("unknown gameplay import")?.clone());
        selected.insert(name);
    }
    let mut visited = BTreeSet::new();
    let captured = crate::input_view::current();
    while let Some(p) = pending.pop() {
        bounded::check()?;
        if !visited.insert(p.clone()) {
            continue;
        }
        let retained;
        let fresh;
        let facts = if let Some(view) = &captured {
            retained = view.source_facts(&p)?;
            &retained.facts
        } else {
            fresh = crate::source_facts::scan(&p, &bounded::read(&p)?)?;
            &fresh
        };
        facts.require_complete()?;
        for word in facts.identifiers() {
            if let Some(module) = available.get(word)
                && selected.insert(word.to_owned())
            {
                pending.push(module.clone());
            }
        }
        // A module path is an actual compiler input, not a crate-root import.
        // Traverse its dependencies without declaring another nominal copy.
        for path in facts.file_dependencies() {
            let path = if let Some(view) = &captured { view.physical(path)? } else { path.clone() };
            if !visited.contains(&path) {
                pending.push(path);
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

use tool_execution::builds::dep_paths;

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
    decomp_source::context::limited(|| dependency_inventory_bounded(request, confined, captured_ridley))
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
    let compiler_dependencies = bounded::text(run.join("compiler.d"))?;
    ensure!(
        !compiler_dependencies.lines().any(|line| line.starts_with("# env-dep:")),
        "unsupported compiler environment dependency"
    );
    for name in dep_paths(&compiler_dependencies)? {
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
        let path = if let Some(view) = crate::input_view::current() {
            // Retained Native captures live below run, but remain source
            // dependencies. Only the recipe's known generated Rust files gain
            // the run-prefix exemption.
            if view.logical(&path).is_ok() {
                view.physical(&path)?
            } else if ["wrapper.rs", "memory.rs", "fixture-worker.rs"].iter().any(|name| path == run.join(name)) {
                path
            } else {
                anyhow::bail!("unbound compiler dependency: {}", path.display());
            }
        } else {
            path
        };
        bounded::safe(&path)?;
        paths.insert(fs::canonicalize(path)?);
    }
    for p in &request.extra {
        bounded::check()?;
        let path = if p.is_absolute() { p.to_path_buf() } else { checkout.join(p) };
        let path = if let Some(view) = crate::input_view::current() { view.physical(&path)? } else { path };
        paths.insert(fs::canonicalize(path)?);
    }
    let mut result = BTreeMap::new();
    for (index, p) in paths.into_iter().enumerate() {
        bounded::check()?;
        let logical = if let Some(view) = crate::input_view::current() {
            if let Ok(logical) = view.logical(&p) {
                logical
            } else {
                ensure!(
                    ["wrapper.rs", "memory.rs", "fixture-worker.rs"].iter().any(|name| p == run.join(name)),
                    "unbound retained compiler dependency"
                );
                continue;
            }
        } else {
            if p.starts_with(&run) {
                continue;
            }
            p.clone()
        };
        let logical_checkout =
            crate::input_view::current().map(|view| view.checkout().to_path_buf()).unwrap_or_else(|| checkout.clone());
        let key = logical.strip_prefix(&logical_checkout).unwrap_or(&logical).to_string_lossy().into_owned();
        if captured_ridley && logical == Path::new(CAPTURED_RIDLEY_EVENT) {
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
    let tools = implementation_sources(&checkout, &run, confined)?;
    result.extend(tools.clone());
    #[derive(Serialize)]
    struct PipelineInputs<'a> {
        binary_sha256: &'a str,
        sources: BTreeMap<String, String>,
    }
    let path = run.join("pipeline-inputs.toml");
    let bytes = toml::to_string(&PipelineInputs { binary_sha256: host_inputs()?.producer_sha256(), sources: tools })?;
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
        let directory = tool_execution::testing::Directory::new()?;
        let sources = directory.0.join("crates/port/src");
        fs::create_dir_all(&sources)?;
        fs::write(sources.join("memory.rs"), "pub struct Wram;")?;
        let candidate = format!("pub fn step() {{ {} }}", "crate::dependency::step(); ".repeat(8192));
        let dependency = format!("pub fn step() {{ {} }}", "crate::candidate::step(); ".repeat(8192));
        fs::write(sources.join("candidate.rs"), &candidate)?;
        fs::write(sources.join("dependency.rs"), &dependency)?;
        let expected = vec![sources.join("candidate.rs"), sources.join("dependency.rs"), sources.join("memory.rs")];
        let cap = candidate.len() as u64 + dependency.len() as u64 + b"pub struct Wram;".len() as u64;
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
    fn canonical_discovery_proves_parent_glob_builtins_from_literal_module_edges() -> Result<()> {
        let directory = tool_execution::testing::Directory::new()?;
        let sources = directory.0.join("crates/port/src");
        fs::create_dir_all(&sources)?;
        fs::write(sources.join("memory.rs"), "pub struct Wram;")?;
        let root = sources.join("candidate.rs");
        let child = sources.join("child_impl.rs");
        fs::write(&root, "#[path=\"child_impl.rs\"] mod owned; pub fn step() { owned::step(); }")?;
        fs::write(&child, "use super::*; #[derive(Clone)] pub struct State; pub fn step() { let _=vec![1]; }")?;
        let paths = canonical_sources(&directory.0, std::slice::from_ref(&root), &[])?;
        assert!(paths.contains(&root) && paths.contains(&child));
        fs::write(&root, "use crate::macros::vec; #[path=\"child_impl.rs\"] mod owned;")?;
        assert!(canonical_sources(&directory.0, &[root], &[]).is_err());
        Ok(())
    }

    #[test]
    fn canonical_discovery_rejects_module_symlink() -> Result<()> {
        let directory = tool_execution::testing::Directory::new()?;
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
            tool_execution::unique_id()
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
            tool_execution::unique_id()
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
