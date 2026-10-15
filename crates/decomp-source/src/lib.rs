//! Pinned Recovered C source identity, definitions and authenticated inventories.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};
use tool_execution::bounded::{self, digest, object};
mod archive;
pub mod context;
mod definitions;
mod inventory;
pub use context::SourceContext;
pub use definitions::{Definition, DefinitionSource, recovered_definition};
pub use inventory::{Bank, Inventory, Routine, inventory, inventory_until};
#[cfg(any(test, feature = "test-support"))]
pub mod testing;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub revision: String,
    pub archive_sha256: String,
    pub files: BTreeMap<String, String>,
    pub freshness: String,
}

#[derive(Deserialize)]
struct DependencyPins {
    dependencies: Vec<DependencyPin>,
}
#[derive(Deserialize)]
struct DependencyPin {
    name: String,
    archive: String,
    revision: String,
    sha256: String,
}

pub fn verify_source(checkout: &Path, source: &Path) -> Result<Source> {
    context::limited(|| verify_source_bounded(checkout, source))
}

pub fn verify_source_until(checkout: &Path, source: &Path, deadline: u64, max_bytes: u64) -> Result<Source> {
    context::until(deadline, max_bytes, || verify_source_bounded(checkout, source))
}

fn verify_source_bounded(checkout: &Path, source: &Path) -> Result<Source> {
    bounded::safe(checkout)?;
    bounded::safe(source)?;
    let checkout = fs::canonicalize(checkout)?;
    let source = fs::canonicalize(source)?;
    bounded::safe(&checkout)?;
    bounded::safe(&source)?;
    let pins: DependencyPins = object(&checkout.join("tools/dependencies.toml"))?;
    ensure!(pins.dependencies.len() <= 1024, "dependency pin count exceeded");
    let pin = pins.dependencies.iter().find(|v| v.name == "snesrev-sm").context("snesrev pin missing")?;
    ensure!(
        safe_relative(&pin.archive) && !pin.archive.contains('/') && pin.sha256.len() == 64,
        "pinned archive name or digest invalid"
    );
    let archive = source.parent().context("source parent missing")?.join("downloads").join(&pin.archive);
    let key = (
        checkout,
        source.clone(),
        format!("{:x}", Sha256::digest(serde_json::to_vec(&(&pin.archive, &pin.revision, &pin.sha256))?)),
    );
    if let Some(cached) = context::cached_source(&key) {
        ensure!(digest(&archive)? == pin.sha256, "pinned archive identity changed");
        for (name, expected) in &cached.files {
            ensure!(digest(&source.join(name))? == *expected, "pinned source changed: {name}");
        }
        return Ok(cached);
    }
    let files = archive::authenticate_tar_source(&archive, &source, &pin.sha256)?;
    let result = Source {
        revision: pin.revision.clone(),
        archive_sha256: pin.sha256.clone(),
        files,
        freshness: "exact pinned archive; no latest-upstream assertion".into(),
    };
    context::cache_source(key, result.clone());
    Ok(result)
}

pub fn c_units(source: &Path) -> Result<Vec<PathBuf>> {
    bounded::limited(|| c_units_bounded(source))
}

fn c_units_bounded(source: &Path) -> Result<Vec<PathBuf>> {
    let mut units = bounded::entries(&source.join("src"))?;
    units.retain(|path| {
        path.extension().is_some_and(|v| v == "c")
            && path.file_stem().is_some_and(|v| {
                v.to_string_lossy().starts_with("sm_")
                    || ["spc_player", "tracing", "util"].contains(&v.to_string_lossy().as_ref())
            })
    });
    for path in bounded::entries(&source.join("src/snes"))? {
        bounded::check()?;
        if path.extension().is_some_and(|v| v == "c") {
            units.push(path);
        }
    }
    ensure!(units.len() <= 4096, "pinned C unit inventory exceeded");
    for path in &units {
        bounded::safe(path)?;
        ensure!(path.symlink_metadata()?.is_file(), "pinned C unit is not a regular file");
    }
    units.sort();
    Ok(units)
}

pub fn authenticate_inventory(checkout: &Path, source: &Path, path: &Path) -> Result<Source> {
    context::limited(|| authenticate_inventory_bounded(checkout, source, path))
}

pub fn authenticate_inventory_until(
    checkout: &Path,
    source: &Path,
    path: &Path,
    deadline: u64,
    max_bytes: u64,
) -> Result<Source> {
    context::until(deadline, max_bytes, || authenticate_inventory_bounded(checkout, source, path))
}

fn authenticate_inventory_bounded(checkout: &Path, source: &Path, path: &Path) -> Result<Source> {
    #[derive(Deserialize)]
    struct DeclaredInventory {
        schema_version: u32,
        source_identity: Source,
        source_identity_sha256: String,
        routines: Vec<DeclaredRoutine>,
    }
    #[derive(Deserialize)]
    struct DeclaredRoutine {
        symbol: String,
        source: String,
        line: usize,
        definition_sha256: String,
        #[serde(default)]
        defined_dependencies: Vec<String>,
        #[serde(default)]
        external_or_macro_calls: Vec<String>,
        #[serde(default)]
        indirect_indexed_calls: Vec<String>,
    }
    let inventory: DeclaredInventory = object(path)?;
    ensure!(
        inventory.schema_version == 1 && !inventory.routines.is_empty() && inventory.routines.len() <= 65536,
        "unsupported native source inventory"
    );
    let identity = verify_source(checkout, source)?;
    ensure!(
        inventory.source_identity == identity
            && inventory.source_identity_sha256 == digest(&checkout.join("tools/dependencies.toml"))?,
        "native source inventory identity differs"
    );
    let symbols = inventory.routines.iter().map(|routine| routine.symbol.as_str()).collect::<BTreeSet<_>>();
    ensure!(symbols.len() == inventory.routines.len(), "duplicate inventory symbol");
    let mut files = BTreeMap::new();
    for routine in &inventory.routines {
        bounded::check()?;
        ensure!(identifier(&routine.symbol) && safe_relative(&routine.source), "unsafe inventory routine");
        ensure!(
            identity.files.contains_key(&routine.source) && routine.line > 0,
            "inventory definition provenance missing"
        );
        if !files.contains_key(&routine.source) {
            let text = bounded::text(source.join(&routine.source))?;
            let code = inventory::clean_bounded(&text)?;
            files.insert(routine.source.clone(), (text, code));
        }
        let (text, code) = &files[&routine.source];
        let offset = text.split_inclusive('\n').take(routine.line - 1).map(str::len).sum::<usize>();
        let original = text.get(offset..).context("inventory line exceeds source")?;
        let code = code.get(offset..).context("inventory code exceeds source")?;
        let open = code.iter().position(|value| *value == b'{').context("inventory definition body missing")?;
        let signature = &code[..open];
        ensure!(!signature.contains(&b';'), "inventory definition absorbed prototype");
        let paren = signature.iter().position(|value| *value == b'(').context("inventory signature missing")?;
        let prefix = std::str::from_utf8(&signature[..paren])?.trim();
        let name = prefix
            .split(|value: char| !(value.is_ascii_alphanumeric() || value == '_'))
            .rfind(|value| !value.is_empty());
        ensure!(name == Some(routine.symbol.as_str()), "inventory symbol disagrees with pinned definition");
        let mut end = open + 1;
        let mut depth = 1;
        while end < code.len() && depth > 0 {
            if end.is_multiple_of(64 * 1024) {
                bounded::check()?;
            }
            match code[end] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            end += 1;
        }
        ensure!(
            depth == 0 && format!("{:x}", Sha256::digest(&original.as_bytes()[..end])) == routine.definition_sha256,
            "inventory definition hash differs"
        );
        let (calls, indexed) = inventory::names_bounded(&code[open..end])?;
        let defined = calls.iter().filter(|name| symbols.contains(name.as_str())).cloned().collect::<Vec<_>>();
        let external = calls.iter().filter(|name| !symbols.contains(name.as_str())).cloned().collect::<Vec<_>>();
        ensure!(
            defined == routine.defined_dependencies
                && external == routine.external_or_macro_calls
                && indexed == routine.indirect_indexed_calls,
            "inventory lexical dependency agreement missing"
        );
    }
    Ok(identity)
}

fn identifier(text: &str) -> bool {
    let mut chars = text.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
fn safe_relative(text: &str) -> bool {
    !text.is_empty() && Path::new(text).components().all(|c| matches!(c, Component::Normal(_)))
}

#[cfg(test)]
mod source_tests {
    use super::*;
    use crate::testing::{append_tar, gzip_archive};
    use tool_execution::testing::Directory;
    #[test]
    fn pinned_source_cache_rechecks_files_and_binds_changed_revision() -> Result<()> {
        let directory = Directory::new()?;
        let source = directory.0.join("deps/snesrev-sm");
        let downloads = directory.0.join("deps/downloads");
        fs::create_dir_all(source.join("src"))?;
        fs::create_dir_all(&downloads)?;
        fs::create_dir(directory.0.join("tools"))?;
        let material = source.join("src/sm_80.c");
        fs::write(&material, b"void Step(void) {}\n")?;
        fs::write(source.join("LICENSE.txt"), b"test source\n")?;
        let mut tar = Vec::new();
        append_tar(&mut tar, "root/src/sm_80.c", b'0', b"void Step(void) {}\n");
        append_tar(&mut tar, "root/LICENSE.txt", b'0', b"test source\n");
        tar.extend_from_slice(&[0u8; 1024]);
        let hash = gzip_archive(&downloads.join("source.tar.gz"), &tar)?;
        let pins = directory.0.join("tools/dependencies.toml");
        let pin = |revision: &str| {
            format!(
                "[[dependencies]]\nname=\"snesrev-sm\"\narchive=\"source.tar.gz\"\nrevision={revision:?}\nsha256={hash:?}\n"
            )
        };
        fs::write(&pins, pin("first"))?;
        let packed = fs::read(downloads.join("source.tar.gz"))?;
        let recheck_bytes = pins.metadata()?.len()
            + packed.len() as u64
            + material.metadata()?.len()
            + source.join("LICENSE.txt").metadata()?.len();
        context::until(bounded::now_unix_ms()? + 10_000, 1024 * 1024, || {
            let first = verify_source(&directory.0, &source)?;
            assert_eq!(first.revision, "first");
            let before = bounded::charged_bytes();
            assert_eq!(first, verify_source(&directory.0, &source)?);
            assert_eq!(
                bounded::charged_bytes() - before,
                recheck_bytes,
                "source reuse rehashes every input without expanding the archive again"
            );
            let shared = context::shared_scope()?;
            let before = bounded::charged_bytes();
            let worker = std::thread::scope(|scope| {
                scope.spawn(|| shared.enter(|| verify_source(&directory.0, &source))).join().unwrap()
            })?;
            assert_eq!(first, worker);
            assert_eq!(
                bounded::charged_bytes() - before,
                recheck_bytes,
                "the source worker must inherit both the cache and the input counters"
            );
            fs::write(&material, b"void Evil(void) {}\n")?;
            assert!(verify_source(&directory.0, &source).is_err());
            fs::write(&material, b"void Step(void) {}\n")?;
            fs::write(&pins, pin("second"))?;
            let second = verify_source(&directory.0, &source)?;
            assert_eq!(second.revision, "second");
            assert_eq!(first.files, second.files);
            fs::write(downloads.join("source.tar.gz"), b"changed archive")?;
            assert!(verify_source(&directory.0, &source).is_err());
            fs::write(downloads.join("source.tar.gz"), &packed)?;
            Ok(())
        })?;
        context::until(bounded::now_unix_ms()? + 10_000, 1024 * 1024, || {
            verify_source(&directory.0, &source)?;
            assert!(
                bounded::charged_bytes() > recheck_bytes,
                "a new operation must expand and authenticate the archive again"
            );
            Ok(())
        })?;
        Ok(())
    }

    #[test]
    fn rust_archive_reader_authenticates_actual_pinned_source() -> Result<()> {
        let checkout = Path::new("/home/jim/sm-decomp");
        if !checkout.join("deps/downloads/snesrev-sm.tar.gz").is_file() {
            return Ok(());
        }
        let source = verify_source_until(
            checkout,
            &checkout.join("deps/snesrev-sm"),
            bounded::now_unix_ms()? + 10_000,
            64 * 1024 * 1024,
        )?;
        assert_eq!(source.revision, "578f90b3cc49557bb70060ad033bb90b8cf8ac50");
        assert_eq!(source.archive_sha256, "69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193");
        assert!(source.files.contains_key("src/sm_80.c"));
        assert!(source.files.contains_key("LICENSE.txt"));
        assert_eq!(source.files["src/sm_80.c"], digest(&checkout.join("deps/snesrev-sm/src/sm_80.c"))?);
        Ok(())
    }
}

/// Exact compiled source inputs to retain in higher-level evidence contracts.
pub const IMPLEMENTATION_SOURCES: &[(&str, &str)] = &[
    ("crates/decomp-source/Cargo.toml", include_str!("../Cargo.toml")),
    ("crates/decomp-source/src/lib.rs", include_str!("lib.rs")),
    ("crates/decomp-source/src/archive.rs", include_str!("archive.rs")),
    ("crates/decomp-source/src/context.rs", include_str!("context.rs")),
    ("crates/decomp-source/src/definitions.rs", include_str!("definitions.rs")),
    ("crates/decomp-source/src/inventory.rs", include_str!("inventory.rs")),
];
