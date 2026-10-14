//! Installed actual-C adapter for frozen, registered plain Offline families.
//! Candidate compilation and each fixture invocation are separate confined
//! producers. Only the trusted parent writes the family verdict and inventory.
use crate::{
    atomic_toml, batch, builds, family,
    family_model::{Asset, Manifest, Source, Verdict, VerifyRequest},
    inputs,
};
use anyhow::{Context, Result, ensure};
use inputs::bounded::{self, digest};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use translation_execution::{
    OperationContext,
    confinement::RunLimits,
    model::{sealed, sha256},
    retention::Reference,
};

#[path = "translation_native_recipe.rs"]
mod recipes;
pub(crate) use recipes::{NativeRecipe, contract_for_direct_animation, prepare_generated_plm};
#[path = "translation_native_operations.rs"]
mod operation_storage;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeLimits {
    pub timeout_seconds: u64,
    pub writable_bytes: u64,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub stdin_bytes: usize,
}
impl NativeLimits {
    fn run_limits(&self) -> Result<RunLimits> {
        ensure!(self.timeout_seconds > 0 && self.writable_bytes >= 4096, "native producer budget missing");
        ensure!(
            self.stdout_bytes <= 1024 * 1024 && self.stderr_bytes <= 1024 * 1024,
            "native logs exceed installed bound"
        );
        Ok(RunLimits {
            timeout: Duration::from_secs(self.timeout_seconds),
            writable_bytes: self.writable_bytes,
            stdout_bytes: self.stdout_bytes,
            stderr_bytes: self.stderr_bytes,
            stdin_bytes: self.stdin_bytes,
        })
    }
}

/// A prior Offline family proof establishes only its declared fixture domain.
/// NativeScope uses the existing directly exercised native scope validator.
/// An installed migration policy can require NativeScope; rewrite responses
/// cannot choose either mode or supply acceptance/provenance artifacts.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum NativeProvenance {
    NativeScope { scope: Asset, checkout: PathBuf },
    ReviewedOfflineFamily { verdict: Asset, checkout: PathBuf },
    ReviewedCatalog { catalog: Asset, requirements: Asset, checkout: PathBuf, rationale: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeContract {
    pub family: String,
    /// Registered manifest relative to the prepared checkout.
    pub manifest: Asset,
    pub source_inventory: Asset,
    /// Complete registered routine/case coverage, never a worker-selected subset.
    pub coverage: BTreeMap<String, Vec<String>>,
    pub provenance: NativeProvenance,
    /// Complete selected Rust closure, harness and pin at preparation.
    pub frozen_inputs: BTreeMap<String, String>,
    pub compiler_limits: NativeLimits,
    pub fixture_limits: NativeLimits,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe: Option<NativeRecipe>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PreparedNative {
    pub contract: NativeContract,
    pub manifest: Manifest,
    pub source: Source,
    pub pinned_source: PathBuf,
    pub candidate_path: String,
    /// Writable members of this family's Rust closure approved by the whole
    /// translation scope. Omitted for historical single-module installations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_candidate_paths: Option<BTreeSet<String>>,
    pub rust_inputs: Vec<String>,
    pub c_inputs: Vec<String>,
    pub dependencies_sha256: String,
    pub verification_sha256: String,
}

/// Trusted one-catalog preparation context. Construction authenticates actual
/// source and the complete inventory once. It is not a deserializable artifact;
/// every produced PreparedNative still seals that exact Source and inventory.
pub(crate) struct NativeCatalogInputs {
    checkout: PathBuf,
    pinned_source: PathBuf,
    inventory_path: PathBuf,
    inventory_sha256: String,
    source: Source,
    symbols: BTreeSet<String>,
}

pub(crate) fn prepare_catalog_inputs_until(
    checkout: &Path,
    pinned_source: &Path,
    inventory: &Asset,
    deadline: u64,
    max_bytes: u64,
) -> Result<NativeCatalogInputs> {
    bounded::until(deadline, max_bytes, || catalog_inputs(checkout, pinned_source, inventory))
}

fn catalog_inputs(checkout: &Path, pinned_source: &Path, inventory: &Asset) -> Result<NativeCatalogInputs> {
    bounded::safe(checkout)?;
    bounded::safe(pinned_source)?;
    let checkout = fs::canonicalize(checkout)?;
    let pinned_source = fs::canonicalize(pinned_source)?;
    let inventory_path = check_asset(&checkout, inventory)?;
    let source = family::authenticate_inventory(&checkout, &pinned_source, &inventory_path)?;
    let declared: toml::Value = family::object(&inventory_path)?;
    let routines = declared["routines"].as_array().context("native routine inventory missing")?;
    ensure!(routines.len() <= 65536, "native routine source inventory exceeded");
    let symbols = routines
        .iter()
        .map(|r| Ok(r["symbol"].as_str().context("native inventory symbol missing")?.to_owned()))
        .collect::<Result<_>>()?;
    Ok(NativeCatalogInputs {
        checkout,
        pinned_source,
        inventory_path,
        inventory_sha256: inventory.sha256.clone(),
        source,
        symbols,
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_scope_in_catalog_until(
    checkout: &Path,
    pinned_source: &Path,
    contract: &NativeContract,
    approved_paths: &[String],
    catalog: &NativeCatalogInputs,
    deadline: u64,
    max_bytes: u64,
) -> Result<PreparedNative> {
    bounded::until(deadline, max_bytes, || {
        ensure!(
            matches!(contract.provenance, NativeProvenance::ReviewedCatalog { .. }),
            "catalog input context requires reviewed catalog provenance"
        );
        let mut prepared = prepare_bounded_with_catalog(checkout, pinned_source, contract, Some(catalog))?;
        bind_scope_candidates(&mut prepared, approved_paths)?;
        Ok(prepared)
    })
}

/// Installed paths/resources provided by the trusted supervisor configuration.
pub(crate) struct StrictRuntime {
    pub helper: PathBuf,
    pub rustc: PathBuf,
    pub c_compiler: PathBuf,
    pub operation_root: PathBuf,
    pub pipeline_root: PathBuf,
    pub workers: usize,
    pub gate_timeout_seconds: u64,
    /// Protected whole-stage deadline, shared by every affected family.
    pub deadline_unix_ms: Option<u64>,
    pub max_output_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeOperation {
    pub directory: PathBuf,
    pub label: String,
    pub invocation: Asset,
    pub context: OperationContext,
    pub result: Reference,
    pub result_sha256: String,
    pub references: Vec<Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeReceipt {
    pub prepared_sha256: String,
    pub candidate_sha256: String,
    pub verdict: Asset,
    pub passed: bool,
    pub diagnostic: String,
    pub evidence_bytes: u64,
    pub context: OperationContext,
    pub operation_root: PathBuf,
    pub operations: Vec<NativeOperation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_index: Option<Reference>,
    pub max_output_bytes: u64,
    pub reserved_bytes: u64,
}

impl NativeReceipt {
    fn store_operations(&mut self, output: &Path) -> Result<()> {
        let index = operation_storage::store(self, output)?;
        self.operation_index = Some(index);
        self.operations.clear();
        Ok(())
    }

    pub(crate) fn visit_operations(&self, mut visit: impl FnMut(&NativeOperation) -> Result<()>) -> Result<()> {
        if self.operation_index.is_some() {
            return operation_storage::visit(self, visit);
        }
        ensure!(self.operations.len() <= 32770, "native operation receipt inventory exceeded");
        for operation in &self.operations {
            bounded::check()?;
            visit(operation)?;
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn operation_count(&self) -> Result<usize> {
        let mut count = 0;
        self.visit_operations(|_| {
            count += 1;
            Ok(())
        })?;
        Ok(count)
    }
}

#[derive(Serialize, Deserialize)]
struct NativeIntent {
    context: OperationContext,
    prepared_sha256: String,
    candidate_sha256: String,
    deadline_unix_ms: u64,
    #[serde(default)]
    outer_deadline_unix_ms: Option<u64>,
    max_output_bytes: u64,
    operation_root: PathBuf,
    parent_reservation_bytes: u64,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    reconcile_producer_reservations: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    operation_metadata_bytes: Option<u64>,
    helper: PathBuf,
    rustc: PathBuf,
    c_compiler: PathBuf,
    compiler_runtime_sha256: String,
}

fn now_unix_ms() -> Result<u64> {
    Ok(u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?)
}

fn new_deadline(entered: u64, runtime: &StrictRuntime) -> Result<u64> {
    ensure!(
        runtime.gate_timeout_seconds > 0 && runtime.gate_timeout_seconds <= 3600,
        "native aggregate gate budget missing"
    );
    let local = entered
        .checked_add(runtime.gate_timeout_seconds.checked_mul(1000).context("native deadline overflow")?)
        .context("native deadline overflow")?;
    Ok(runtime.deadline_unix_ms.map_or(local, |outer| local.min(outer)))
}

fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty() && path.components().all(|part| matches!(part, Component::Normal(_)))
}
fn file(path: &Path) -> Result<PathBuf> {
    bounded::safe(path)?;
    ensure!(path.symlink_metadata()?.is_file(), "native input is not a regular file");
    Ok(path.into())
}
fn immutable(path: &Path, bytes: &[u8]) -> Result<()> {
    inputs::immutable_bytes(path, bytes)
}
fn check_asset(checkout: &Path, asset: &Asset) -> Result<PathBuf> {
    let path = if asset.file.is_absolute() {
        asset.file.clone()
    } else {
        ensure!(relative(&asset.file), "native asset relative path alias");
        checkout.join(&asset.file)
    };
    let path = file(&path)?;
    ensure!(digest(&path)? == asset.sha256, "native asset changed: {}", path.display());
    Ok(path)
}
fn coverage(manifest: &Manifest) -> Result<BTreeMap<String, Vec<String>>> {
    ensure!(
        manifest.routines.len() <= 1024
            && manifest.routines.iter().map(|routine| routine.cases.len()).sum::<usize>() <= 16384,
        "native routine or fixture inventory exceeded"
    );
    bounded::count(manifest.routines.len())?;
    family::ordered_tasks(manifest)?;
    let mut result = BTreeMap::new();
    for routine in &manifest.routines {
        bounded::check()?;
        ensure!(family::identifier(&routine.symbol), "native routine signature missing");
        let cases = routine.cases.iter().map(|case| case.name.clone()).collect::<Vec<_>>();
        ensure!(result.insert(routine.symbol.clone(), cases).is_none(), "native routine duplicated");
    }
    ensure!(!result.is_empty(), "native fixture coverage missing");
    Ok(result)
}
fn plain(manifest: &Manifest) -> Result<()> {
    ensure!(
        !family::dispatcher(manifest)
            && !manifest.projectile_host
            && !manifest.projectile_rendering
            && manifest.callback_adapter.is_none()
            && !manifest.infrastructure_probe,
        "native recipe does not support this registered harness"
    );
    ensure!(manifest.cases.is_empty(), "plain family has undeclared top-level cases");
    Ok(())
}

fn c_inputs(source: &Path, manifest: &Manifest, full: bool) -> Result<Vec<PathBuf>> {
    ensure!(manifest.c_support.len() <= 4096, "native C support inventory exceeded");
    fn headers(path: &Path, result: &mut BTreeSet<PathBuf>, depth: usize) -> Result<()> {
        ensure!(depth <= 32, "pinned C header tree depth exceeded");
        for path in bounded::entries(path)? {
            bounded::check()?;
            ensure!(!path.is_symlink(), "pinned C input symlink");
            if path.is_dir() {
                headers(&path, result, depth + 1)?;
            } else if path.extension().is_some_and(|extension| extension == "h") {
                result.insert(path);
                ensure!(result.len() <= 4096, "pinned C header inventory exceeded");
            }
        }
        Ok(())
    }
    let mut result = BTreeSet::new();
    headers(&source.join("src"), &mut result, 0)?;
    for name in std::iter::once(&manifest.bank).chain(&manifest.c_support) {
        bounded::check()?;
        if full && name.is_empty() {
            continue;
        }
        ensure!(
            relative(Path::new(name)) && !name.contains('/') && name.starts_with("sm_") && name.ends_with(".c"),
            "unsafe native C unit"
        );
        result.insert(file(&source.join("src").join(name))?);
    }
    ensure!(result.len() <= 4096, "pinned C input inventory exceeded");
    Ok(result.into_iter().collect())
}

const HARNESS: &[&str] = &[
    "main.rs",
    "inputs.rs",
    "bounded_inputs.rs",
    "builds.rs",
    "family.rs",
    "family_model.rs",
    "translation_native.rs",
    "source_inventory.rs",
    "batch.rs",
    "family_header.c",
    "family_pointer.c",
    "dispatcher_pointer.c",
    "dispatcher_plm.c",
    "dispatcher_eproj.c",
    "dispatcher_host.c",
    "plm_fixtures.rs",
    "cleanup.rs",
    "fixture_execution.rs",
    "fixture_worker.rs",
    "metrics.rs",
    "resource_sampler.rs",
];

/// Freeze a reviewed catalog entry. Preparation still authenticates every seal
/// and rejects missing provenance/coverage before the supervisor emits a capsule.
pub(crate) fn contract_for_family(
    checkout: &Path,
    manifest: &Path,
    source_inventory: Asset,
    provenance: NativeProvenance,
    compiler_limits: NativeLimits,
    fixture_limits: NativeLimits,
) -> Result<NativeContract> {
    bounded::limited(|| {
        contract_for_family_bounded(checkout, manifest, source_inventory, provenance, compiler_limits, fixture_limits)
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn contract_for_family_until(
    checkout: &Path,
    manifest: &Path,
    source_inventory: Asset,
    provenance: NativeProvenance,
    compiler_limits: NativeLimits,
    fixture_limits: NativeLimits,
    deadline: u64,
    max_bytes: u64,
) -> Result<NativeContract> {
    bounded::until(deadline, max_bytes, || {
        contract_for_family_bounded(checkout, manifest, source_inventory, provenance, compiler_limits, fixture_limits)
    })
}

fn contract_for_family_bounded(
    checkout: &Path,
    manifest: &Path,
    source_inventory: Asset,
    provenance: NativeProvenance,
    compiler_limits: NativeLimits,
    fixture_limits: NativeLimits,
) -> Result<NativeContract> {
    bounded::safe(checkout)?;
    let checkout = fs::canonicalize(checkout)?;
    ensure!(relative(manifest), "registered native manifest must be relative");
    let path = file(&checkout.join(manifest))?;
    let declared: Manifest = family::object(&path)?;
    let recipe = recipes::for_family(&checkout, manifest, &declared)?;
    contract_for_declared(
        &checkout,
        manifest,
        source_inventory,
        provenance,
        compiler_limits,
        fixture_limits,
        recipe,
        declared,
    )
}

#[allow(clippy::too_many_arguments)]
fn contract_for_declared(
    checkout: &Path,
    manifest: &Path,
    source_inventory: Asset,
    provenance: NativeProvenance,
    compiler_limits: NativeLimits,
    fixture_limits: NativeLimits,
    recipe: Option<NativeRecipe>,
    original: Manifest,
) -> Result<NativeContract> {
    let path = file(&checkout.join(manifest))?;
    let declared = recipes::normalize_manifest(checkout, original, recipe.as_ref())?;
    let candidate = file(&checkout.join(&declared.rust_module))?;
    ensure!(
        declared.rust_dependencies.len() <= recipes::input_inventory_limit(recipe.as_ref()),
        "native declared dependency inventory exceeded"
    );
    let sources = inputs::canonical_sources(&checkout, &[candidate], &declared.rust_imports)?;
    let mut frozen_inputs = BTreeMap::new();
    for path in sources {
        bounded::check()?;
        frozen_inputs.insert(path.strip_prefix(&checkout)?.display().to_string(), digest(&path)?);
    }
    for name in HARNESS {
        let name = format!("crates/pipeline/src/{name}");
        frozen_inputs.insert(name.clone(), digest(&file(&checkout.join(&name))?)?);
    }
    for name in recipes::harness(recipe.as_ref()) {
        frozen_inputs.insert(name.into(), digest(&file(&checkout.join(name))?)?);
    }
    for path in &declared.rust_dependencies {
        bounded::check()?;
        ensure!(
            relative(path)
                || (recipe.is_some() || matches!(provenance, NativeProvenance::ReviewedCatalog { .. }))
                    && path.is_absolute(),
            "native declared dependency must be relative"
        );
        frozen_inputs.insert(
            path.to_str().context("native dependency path UTF-8")?.into(),
            recipes::dependency_digest(recipe.as_ref(), &file(&checkout.join(path))?)
                .with_context(|| format!("hash frozen native dependency {}", path.display()))?,
        );
    }
    frozen_inputs.insert("tools/dependencies.toml".into(), digest(&file(&checkout.join("tools/dependencies.toml"))?)?);
    frozen_inputs.insert(manifest.display().to_string(), digest(&path)?);
    if matches!(provenance, NativeProvenance::ReviewedCatalog { .. }) {
        frozen_inputs.insert("tools/port/batch.toml".into(), digest(&checkout.join("tools/port/batch.toml"))?);
    }
    recipes::freeze_assets(checkout, recipe.as_ref(), &mut frozen_inputs)?;
    for fixture in declared.routines.iter().flat_map(|routine| &routine.cases) {
        for (path, expected) in [
            (fixture.before_file.as_ref(), fixture.before_sha256.as_ref()),
            (fixture.expected_after_file.as_ref(), fixture.expected_after_sha256.as_ref()),
        ] {
            if let Some(path) = path {
                let expected = expected.context("native captured fixture digest missing")?;
                ensure!(digest(&file(&checkout.join(path))?)? == *expected, "native captured fixture changed");
                frozen_inputs.insert(path.to_str().context("native captured path UTF-8")?.into(), expected.clone());
            }
        }
    }
    Ok(NativeContract {
        family: declared.family.clone(),
        manifest: Asset { file: manifest.into(), sha256: digest(&path)? },
        source_inventory,
        coverage: coverage(&declared)?,
        provenance,
        frozen_inputs,
        compiler_limits,
        fixture_limits,
        recipe,
    })
}

fn provenance(
    checkout: &Path,
    pinned_source: &Path,
    contract: &NativeContract,
    manifest: &Manifest,
    source: &Source,
    registration: &recipes::Registration,
) -> Result<()> {
    match &contract.provenance {
        NativeProvenance::ReviewedCatalog { catalog, requirements, checkout: owner, rationale } => {
            ensure!(!rationale.trim().is_empty() && rationale.len() <= 4096, "catalog review rationale missing");
            check_asset(owner, requirements)?;
            let reviewed = check_asset(owner, catalog)?;
            let inventory: batch::Catalog = family::object(&reviewed)?;
            ensure!(
                sealed(&batch::entry(&inventory, &registration.catalog_family)?)? == sealed(&registration.entry)?,
                "reviewed catalog entry differs"
            );
        }
        NativeProvenance::NativeScope { scope, checkout: owner } => {
            let path = check_asset(owner, scope)?;
            crate::translation::validate_scope(owner, &path)?;
            let scope: crate::translation::Scope = family::object(&path)?;
            ensure!(
                manifest.routines.iter().all(|routine| scope.routines.contains(&routine.symbol)),
                "native scope coverage disagrees with family"
            );
        }
        NativeProvenance::ReviewedOfflineFamily { verdict, checkout: owner } => {
            let path = check_asset(owner, verdict)?;
            let prior = family::authenticate_with_source(
                crate::family_model::AuthenticateRequest { checkout: owner.clone(), verdict: path },
                pinned_source,
            )?;
            ensure!(
                prior.family == contract.family && prior.source.as_ref() == Some(source),
                "offline provenance family/source differs"
            );
            let prior_manifest: Manifest = family::object(&prior.run.join("manifest.toml"))?;
            ensure!(
                coverage(&prior_manifest)? == contract.coverage && prior_manifest == *manifest,
                "offline provenance verification declarations differ"
            );
            ensure!(
                prior.implementation.as_ref().context("offline provenance implementation missing")?.module_sha256
                    == digest(&checkout.join(&manifest.rust_module))?,
                "offline provenance does not authenticate the prepared candidate"
            );
        }
    }
    Ok(())
}

pub(crate) fn prepare(checkout: &Path, pinned_source: &Path, contract: &NativeContract) -> Result<PreparedNative> {
    bounded::limited(|| prepare_bounded(checkout, pinned_source, contract))
}

pub(crate) fn prepare_until(
    checkout: &Path,
    pinned_source: &Path,
    contract: &NativeContract,
    deadline: u64,
    max_bytes: u64,
) -> Result<PreparedNative> {
    bounded::until(deadline, max_bytes, || prepare_bounded(checkout, pinned_source, contract))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn prepare_scope_until(
    checkout: &Path,
    pinned_source: &Path,
    contract: &NativeContract,
    approved_paths: &[String],
    deadline: u64,
    max_bytes: u64,
) -> Result<PreparedNative> {
    bounded::until(deadline, max_bytes, || {
        let mut prepared = prepare_bounded(checkout, pinned_source, contract)?;
        bind_scope_candidates(&mut prepared, approved_paths)?;
        Ok(prepared)
    })
}

fn bind_scope_candidates(prepared: &mut PreparedNative, approved_paths: &[String]) -> Result<()> {
    ensure!(!approved_paths.is_empty() && approved_paths.len() <= 4096, "native scope ownership is empty or excessive");
    let mut approved = BTreeSet::new();
    for name in approved_paths {
        bounded::check()?;
        let path = Path::new(name);
        ensure!(
            relative(path)
                && path.parent() == Some(Path::new("crates/port/src"))
                && path.extension().is_some_and(|extension| extension == "rs")
                && !["memory.rs", "lib.rs"].iter().any(|name| path.file_name().is_some_and(|file| file == *name)),
            "native scope ownership is not an approved gameplay module: {name}"
        );
        ensure!(approved.insert(name.clone()), "duplicate native scope ownership");
    }
    // An affected family may have a read-only root and a writable dependency.
    // Inputs outside its compilation closure remain irrelevant to its receipt.
    let candidates = approved.into_iter().filter(|path| prepared.rust_inputs.contains(path)).collect::<BTreeSet<_>>();
    // Preserve the exact historical identity for a single-root scope. Every
    // other scope binds the complete permitted closure into the frozen policy.
    if candidates == BTreeSet::from([prepared.candidate_path.clone()]) {
        prepared.approved_candidate_paths = None;
    } else {
        prepared.approved_candidate_paths = Some(candidates);
        prepared.dependencies_sha256 =
            sealed(&("scope-native-inputs-v1", &prepared.dependencies_sha256, &prepared.approved_candidate_paths))?;
    }
    Ok(())
}

fn prepare_bounded(checkout: &Path, pinned_source: &Path, contract: &NativeContract) -> Result<PreparedNative> {
    prepare_bounded_with_catalog(checkout, pinned_source, contract, None)
}

fn prepare_bounded_with_catalog(
    checkout: &Path,
    pinned_source: &Path,
    contract: &NativeContract,
    catalog: Option<&NativeCatalogInputs>,
) -> Result<PreparedNative> {
    bounded::safe(checkout)?;
    bounded::safe(pinned_source)?;
    let checkout = fs::canonicalize(checkout)?;
    let pinned_source = fs::canonicalize(pinned_source)?;
    ensure!(relative(&contract.manifest.file), "native manifest is not registered relative input");
    let manifest_path = check_asset(&checkout, &contract.manifest)?;
    let original = family::object(&manifest_path)?;
    let registration = recipes::registration(&checkout, contract, &original)?;
    let manifest = recipes::normalize_manifest(&checkout, original, contract.recipe.as_ref())?;
    ensure!(
        contract.frozen_inputs.len() <= recipes::input_inventory_limit(contract.recipe.as_ref())
            && contract.coverage.len() <= 1024,
        "native frozen input or coverage inventory exceeded"
    );
    bounded::count(contract.frozen_inputs.len())?;
    if contract.recipe.is_none() {
        plain(&manifest)?;
    }
    ensure!(
        manifest.family == contract.family && coverage(&manifest)? == contract.coverage,
        "prescribed native fixture coverage missing or changed"
    );
    contract.compiler_limits.run_limits()?;
    let fixture_limits = contract.fixture_limits.run_limits()?;
    ensure!(fixture_limits.writable_bytes > 0x20000, "native fixture quota cannot retain full WRAM");
    let owned;
    let catalog = if let Some(catalog) = catalog {
        ensure!(
            catalog.checkout == checkout
                && catalog.pinned_source == pinned_source
                && checkout.join(&contract.source_inventory.file) == catalog.inventory_path
                && contract.source_inventory.sha256 == catalog.inventory_sha256,
            "reused native source/inventory context differs"
        );
        catalog
    } else {
        owned = catalog_inputs(&checkout, &pinned_source, &contract.source_inventory)?;
        &owned
    };
    let source = catalog.source.clone();
    ensure!(
        recipes::source_symbols(&manifest, contract.recipe.as_ref())
            .iter()
            .all(|symbol| catalog.symbols.contains(*symbol)),
        "required native routine is absent from agreed source inventory"
    );
    provenance(&checkout, &pinned_source, contract, &manifest, &source, &registration)?;
    for (name, expected) in &contract.frozen_inputs {
        bounded::check()?;
        ensure!(
            (relative(Path::new(name))
                || (contract.recipe.is_some()
                    || matches!(contract.provenance, NativeProvenance::ReviewedCatalog { .. }))
                    && Path::new(name).is_absolute())
                && recipes::dependency_digest(contract.recipe.as_ref(), &file(&checkout.join(name))?)? == *expected,
            "native frozen input changed: {name}"
        );
    }
    let candidate = file(&checkout.join(&manifest.rust_module))?;
    let rust = inputs::canonical_sources(&checkout, &[candidate], &manifest.rust_imports)?;
    for path in &rust {
        let name = path.strip_prefix(&checkout)?.display().to_string();
        ensure!(
            contract.frozen_inputs.get(&name) == Some(&digest(path)?),
            "native Rust harness dependency missing: {name}"
        );
    }
    for name in HARNESS {
        ensure!(
            contract.frozen_inputs.contains_key(&format!("crates/pipeline/src/{name}")),
            "installed native harness dependency missing: {name}"
        );
    }
    for name in recipes::harness(contract.recipe.as_ref()) {
        ensure!(
            contract.frozen_inputs.get(name) == Some(&digest(&file(&checkout.join(name))?)?),
            "native recipe input missing: {name}"
        );
    }
    for dependency in &manifest.rust_dependencies {
        ensure!(
            relative(dependency)
                || (contract.recipe.is_some()
                    || matches!(contract.provenance, NativeProvenance::ReviewedCatalog { .. }))
                    && dependency.is_absolute(),
            "native declared dependency must be relative"
        );
        let name = dependency.to_str().context("native dependency path UTF-8")?;
        ensure!(
            contract.frozen_inputs.get(name)
                == Some(&recipes::dependency_digest(contract.recipe.as_ref(), &file(&checkout.join(dependency))?)?),
            "explicit native harness dependency missing: {name}"
        );
    }
    if let Some(rom) = &manifest.rom {
        check_asset(&checkout, rom)?;
    }
    let mut c = c_inputs(&pinned_source, &manifest, recipes::full_c(contract.recipe.as_ref()))?;
    if recipes::full_c(contract.recipe.as_ref()) {
        c.extend(family::c_units(&pinned_source)?);
        c.sort();
        c.dedup();
    }
    let c_names = c
        .iter()
        .map(|path| Ok(path.strip_prefix(&pinned_source)?.display().to_string()))
        .collect::<Result<Vec<_>>>()?;
    let rust_names =
        rust.iter().map(|path| Ok(path.strip_prefix(&checkout)?.display().to_string())).collect::<Result<Vec<_>>>()?;
    let candidate_path = manifest.rust_module.clone();
    ensure!(relative(Path::new(&candidate_path)), "native candidate ownership path alias");
    let dependencies_sha256 = sealed(&(&source, &contract.frozen_inputs, &c_names, &rust_names))?;
    let verification_sha256 = sealed(&(contract, &manifest))?;
    Ok(PreparedNative {
        contract: contract.clone(),
        manifest,
        source,
        pinned_source,
        candidate_path,
        approved_candidate_paths: None,
        rust_inputs: rust_names,
        c_inputs: c_names,
        dependencies_sha256,
        verification_sha256,
    })
}

fn check_candidate(prepared: &PreparedNative, checkout: &Path) -> Result<()> {
    ensure!(
        prepared.contract.frozen_inputs.len() <= recipes::input_inventory_limit(prepared.contract.recipe.as_ref())
            && prepared.source.files.len() <= 4096,
        "native candidate authentication inventory exceeded"
    );
    for (path, expected) in &prepared.contract.frozen_inputs {
        bounded::check()?;
        if !mutable_candidate(prepared, path) {
            ensure!(
                recipes::dependency_digest(prepared.contract.recipe.as_ref(), &file(&checkout.join(path))?)?
                    == *expected,
                "frozen native dependency changed: {path}"
            );
        }
    }
    check_asset(checkout, &prepared.contract.manifest)?;
    for (name, expected) in &prepared.source.files {
        bounded::check()?;
        ensure!(relative(Path::new(name)), "pinned native source path alias");
        ensure!(
            digest(&file(&prepared.pinned_source.join(name))?)? == *expected,
            "pinned native source changed: {name}"
        );
    }
    file(&checkout.join(&prepared.candidate_path))?;
    Ok(())
}

fn mutable_candidate(prepared: &PreparedNative, path: &str) -> bool {
    prepared.approved_candidate_paths.as_ref().map_or(path == prepared.candidate_path, |paths| paths.contains(path))
}

fn candidate_digest(prepared: &PreparedNative, checkout: &Path) -> Result<String> {
    let Some(paths) = &prepared.approved_candidate_paths else {
        return digest(&file(&checkout.join(&prepared.candidate_path))?);
    };
    let mut inputs = BTreeMap::new();
    for path in paths {
        bounded::check()?;
        inputs.insert(path, digest(&file(&checkout.join(path))?)?);
    }
    sealed(&inputs)
}

/// Before rewrite dispatch, the supervisor runs this same prescribed path on
/// baseline source to establish actual bounded harness readiness.
pub(crate) fn check_ready(
    prepared: &PreparedNative,
    checkout: &Path,
    output: &Path,
    runtime: &StrictRuntime,
    context: &OperationContext,
) -> Result<NativeReceipt> {
    run(prepared, checkout, output, runtime, context)
}

pub(crate) fn run(
    prepared: &PreparedNative,
    checkout: &Path,
    output: &Path,
    runtime: &StrictRuntime,
    context: &OperationContext,
) -> Result<NativeReceipt> {
    run_bounded(prepared, checkout, output, runtime, context)
}

fn run_bounded(
    prepared: &PreparedNative,
    checkout: &Path,
    output: &Path,
    runtime: &StrictRuntime,
    context: &OperationContext,
) -> Result<NativeReceipt> {
    let entered = now_unix_ms()?;
    let intent_exists = output.join("native-intent.toml").is_file();
    if !intent_exists {
        ensure!(
            entered < new_deadline(entered, runtime)?,
            "native whole-stage time budget exhausted before preparation"
        );
    }
    let receipt_path = output.join("native-receipt.toml");
    if receipt_path.is_file() {
        return bounded::reconcile(8 * bounded::DEFAULT_BYTES, || {
            let checkout = fs::canonicalize(checkout)?;
            let receipt: NativeReceipt = family::object(&receipt_path)?;
            ensure!(
                receipt.context == *context
                    && receipt.operation_root == runtime.operation_root
                    && receipt.max_output_bytes == runtime.max_output_bytes,
                "native gate receipt retried with another installed context"
            );
            let intent: NativeIntent = family::object(&output.join("native-intent.toml"))?;
            ensure!(
                intent.outer_deadline_unix_ms == runtime.deadline_unix_ms,
                "native whole-stage deadline changed on retry"
            );
            authenticate(prepared, &checkout, &receipt)?;
            Ok(receipt)
        });
    }
    let saved_intent =
        if intent_exists { Some(family::object::<NativeIntent>(&output.join("native-intent.toml"))?) } else { None };
    let deadline = saved_intent.as_ref().map_or(new_deadline(entered, runtime)?, |intent| intent.deadline_unix_ms);
    let setup = || -> Result<_> {
        bounded::safe(checkout)?;
        bounded::safe(output)?;
        let checkout = fs::canonicalize(checkout)?;
        check_candidate(prepared, &checkout)?;
        fs::create_dir_all(output)?;
        let output = fs::canonicalize(output)?;
        let run = output.join("family");
        fs::create_dir_all(&run)?;
        let prepared_sha256 = sealed(prepared)?;
        let candidate_sha256 = candidate_digest(prepared, &checkout)?;
        let helper = file(&fs::canonicalize(&runtime.helper)?)?;
        let rustc = file(&fs::canonicalize(&runtime.rustc)?)?;
        let c_compiler = file(&fs::canonicalize(&runtime.c_compiler)?)?;
        if !intent_exists {
            ensure!(
                now_unix_ms()? < new_deadline(entered, runtime)?,
                "native whole-stage time budget exhausted before runtime preparation"
            );
        }
        let compiler_runtime = if entered >= deadline {
            builds::strict_compiler_runtime(&rustc)?
        } else {
            builds::strict_compiler_runtime_until(&rustc, deadline, 8 * bounded::DEFAULT_BYTES)?
        };
        let compiler_runtime_sha256 = sealed(&compiler_runtime)?;
        let intent_path = output.join("native-intent.toml");
        let intent = if let Some(intent) = saved_intent {
            ensure!(
                intent.context == *context
                    && intent.prepared_sha256 == prepared_sha256
                    && intent.candidate_sha256 == candidate_sha256
                    && intent.max_output_bytes == runtime.max_output_bytes
                    && intent.operation_root == runtime.operation_root
                    && intent.outer_deadline_unix_ms == runtime.deadline_unix_ms,
                "native gate intent retried with changed source or context"
            );
            ensure!(
                intent.helper == helper
                    && intent.rustc == rustc
                    && intent.c_compiler == c_compiler
                    && intent.compiler_runtime_sha256 == compiler_runtime_sha256,
                "native installed runner configuration changed"
            );
            intent
        } else {
            let deadline_unix_ms = deadline;
            ensure!(
                now_unix_ms()? < deadline_unix_ms,
                "native whole-stage time budget exhausted during runtime preparation"
            );
            let parent_reservation_bytes = parent_reservation(prepared, &checkout)?;
            ensure!(
                runtime.max_output_bytes >= parent_reservation_bytes,
                "native parent snapshots exceed aggregate output budget"
            );
            let intent = NativeIntent {
                context: context.clone(),
                prepared_sha256: prepared_sha256.clone(),
                candidate_sha256: candidate_sha256.clone(),
                deadline_unix_ms,
                outer_deadline_unix_ms: runtime.deadline_unix_ms,
                max_output_bytes: runtime.max_output_bytes,
                operation_root: runtime.operation_root.clone(),
                parent_reservation_bytes,
                reconcile_producer_reservations: true,
                operation_metadata_bytes: Some(operation_storage::capacity(operation_slots(prepared, &run)?.len())?),
                helper: helper.clone(),
                rustc: rustc.clone(),
                c_compiler: c_compiler.clone(),
                compiler_runtime_sha256,
            };
            atomic_toml(&intent_path, &intent)?;
            intent
        };
        let execution = builds::StrictExecution {
            helper,
            rustc,
            c_compiler,
            compiler_runtime,
            operation_root: runtime.operation_root.clone(),
            context: context.clone(),
            root: runtime.pipeline_root.clone(),
            workers: runtime.workers,
            compiler_limits: prepared.contract.compiler_limits.run_limits()?,
            fixture_limits: prepared.contract.fixture_limits.run_limits()?,
            rust_inputs: prepared.rust_inputs.iter().map(|path| checkout.join(path)).collect(),
            c_inputs: prepared.c_inputs.iter().map(|path| prepared.pinned_source.join(path)).collect(),
            deadline_unix_ms: intent.deadline_unix_ms,
            max_output_bytes: intent.max_output_bytes,
            reserved_bytes: RefCell::new(intent.parent_reservation_bytes),
            reconcile_reservations: intent.reconcile_producer_reservations,
        };
        Ok((checkout, output, run, intent, execution))
    };
    let (checkout, output, run, intent, execution) = if intent_exists && entered >= deadline {
        bounded::reconcile(8 * bounded::DEFAULT_BYTES, setup)?
    } else {
        bounded::until(deadline, 8 * bounded::DEFAULT_BYTES, setup)?
    };
    let prepared_sha256 = intent.prepared_sha256.clone();
    let candidate_sha256 = intent.candidate_sha256.clone();
    let request = VerifyRequest {
        execution: "fresh".into(),
        checkout: checkout.clone(),
        source: prepared.pinned_source.clone(),
        output: output.clone(),
        manifest: prepared.contract.manifest.file.clone(),
        root: runtime.pipeline_root.clone(),
        workers: runtime.workers,
        case_jobs: 1,
        adapter: recipes::adapter(prepared.contract.recipe.as_ref()).into(),
    };
    let mut verdict = Verdict {
        status: "rejected".into(),
        run: run.clone(),
        family: prepared.contract.family.clone(),
        original_rom_parity: "not-tested".into(),
        source: Some(prepared.source.clone()),
        ..Default::default()
    };
    let produce = || -> Result<()> {
        let manifest_bytes = recipes::manifest_bytes(prepared, &checkout)?;
        immutable(&run.join("manifest.toml"), &manifest_bytes)?;
        immutable(&run.join("LICENSE.txt"), &bounded::read(prepared.pinned_source.join("LICENSE.txt"))?)?;
        if let Some(rom) = &prepared.manifest.rom {
            let bytes = bounded::read(check_asset(&checkout, rom)?)?;
            ensure!(!bytes.is_empty() && bytes.len() <= 0x400000, "native ROM extent differs");
            immutable(&run.join("rom.bin"), &bytes)?;
            verdict.rom = Some(crate::family_model::RomIdentity { sha256: sha256(&bytes), bytes: bytes.len() });
        }
        let mut implementation = recipes::build(&request, prepared, &checkout, &run, &execution)?;
        implementation.manifest = Some(Asset {
            file: if recipes::generated(prepared.contract.recipe.as_ref()) {
                run.join("manifest.toml")
            } else {
                checkout.join(&prepared.contract.manifest.file)
            },
            sha256: sha256(&manifest_bytes),
        });
        immutable(
            &run.join("manifest-input.toml"),
            toml::to_string(implementation.manifest.as_ref().unwrap())?.as_bytes(),
        )?;
        verdict.implementation = Some(implementation);
        for (routine, fixture, index) in family::ordered_tasks(&prepared.manifest)? {
            verdict.cases.push(recipes::fixture(prepared, routine, fixture, index, &run, &execution)?);
        }
        recipes::verdict_fields(prepared, &mut verdict)?;
        check_candidate(prepared, &checkout)?;
        ensure!(now_unix_ms()? < intent.deadline_unix_ms, "native full family gate time budget exhausted");
        verdict.status = "offline-verified".into();
        verdict.verified_cases = verdict.cases.len();
        verdict.verified_routines = prepared.manifest.routines.len();
        verdict.scope = "declared valid fixture domain only; full canonical WRAM and return values; separately confined actual-C/Rust invocations".into();
        Ok(())
    };
    let result = execute_or_reconcile(intent_exists, deadline, produce);
    if let Err(error) = result {
        if error.downcast_ref::<translation_execution::ExecutionPending>().is_some() {
            return Err(error);
        }
        let diagnostic = format!("{error:#}");
        let mut end = diagnostic.len().min(4096);
        while !diagnostic.is_char_boundary(end) {
            end -= 1;
        }
        verdict.status = "rejected".into();
        verdict.verified_cases = 0;
        verdict.verified_routines = 0;
        verdict.error = Some(diagnostic[..end].into());
    }
    bounded::reconcile(8 * bounded::DEFAULT_BYTES, || {
        bounded::discovery(|| family::inventory(&run, &run, &mut verdict.artifacts))?;
        atomic_toml(&run.join("verdict.toml"), &verdict)?;
        let operations = collect_operations(prepared, &run, &runtime.operation_root, context)?;
        let mut receipt = NativeReceipt {
            prepared_sha256,
            candidate_sha256,
            verdict: Asset { file: run.join("verdict.toml"), sha256: digest(&run.join("verdict.toml"))? },
            passed: verdict.status == "offline-verified",
            diagnostic: verdict.error.clone().unwrap_or_default(),
            evidence_bytes: 0,
            context: context.clone(),
            operation_root: runtime.operation_root.clone(),
            operations,
            operation_index: None,
            max_output_bytes: intent.max_output_bytes,
            reserved_bytes: *execution.reserved_bytes.borrow(),
        };
        if intent.operation_metadata_bytes.is_some() {
            receipt.store_operations(&output)?;
        }
        receipt.evidence_bytes = evidence_bytes(&run, &receipt)?;
        ensure!(
            receipt.evidence_bytes <= intent.max_output_bytes,
            "native evidence exceeded prelaunch aggregate reservation"
        );
        if intent.reconcile_producer_reservations {
            ensure!(
                receipt.evidence_bytes <= receipt.reserved_bytes,
                "native evidence exceeded reconciled reservation"
            );
        }
        if receipt.passed {
            authenticate(prepared, &checkout, &receipt)?;
            if now_unix_ms()? >= intent.deadline_unix_ms {
                verdict.status = "rejected".into();
                verdict.error = Some("native whole-stage time budget exhausted before acceptance".into());
                verdict.verified_cases = 0;
                verdict.verified_routines = 0;
                atomic_toml(&run.join("verdict.toml"), &verdict)?;
                receipt.passed = false;
                receipt.diagnostic = verdict.error.clone().unwrap();
                receipt.verdict.sha256 = digest(&run.join("verdict.toml"))?;
                receipt.evidence_bytes = evidence_bytes(&run, &receipt)?;
                ensure!(
                    receipt.evidence_bytes <= intent.max_output_bytes,
                    "native failed evidence exceeded aggregate reservation"
                );
            }
        }
        atomic_toml(&receipt_path, &receipt)?;
        Ok(receipt)
    })
}

fn execute_or_reconcile<T>(resumed: bool, deadline: u64, action: impl FnOnce() -> Result<T>) -> Result<T> {
    if resumed && now_unix_ms()? >= deadline {
        bounded::reconcile(8 * bounded::DEFAULT_BYTES, action)
    } else {
        bounded::until(deadline, 8 * bounded::DEFAULT_BYTES, action)
    }
}

pub(crate) fn authenticate(prepared: &PreparedNative, checkout: &Path, receipt: &NativeReceipt) -> Result<Verdict> {
    bounded::until(now_unix_ms()?.saturating_add(60_000), 8 * bounded::DEFAULT_BYTES, || {
        authenticate_bounded(prepared, checkout, receipt)
    })
}

fn authenticate_bounded(prepared: &PreparedNative, checkout: &Path, receipt: &NativeReceipt) -> Result<Verdict> {
    bounded::safe(checkout)?;
    let checkout = fs::canonicalize(checkout)?;
    ensure!(receipt.prepared_sha256 == sealed(prepared)?, "native prepared identity differs");
    check_candidate(prepared, &checkout)?;
    ensure!(receipt.candidate_sha256 == candidate_digest(prepared, &checkout)?, "native candidate changed");
    let path = check_asset(&checkout, &receipt.verdict)?;
    let run = path.parent().context("native verdict has no run parent")?;
    let output = run.parent().context("native run has no output parent")?;
    let intent: NativeIntent = family::object(&output.join("native-intent.toml"))?;
    ensure!(
        intent.context == receipt.context
            && intent.prepared_sha256 == receipt.prepared_sha256
            && intent.candidate_sha256 == receipt.candidate_sha256
            && intent.operation_root == receipt.operation_root
            && intent.max_output_bytes == receipt.max_output_bytes
            && intent.outer_deadline_unix_ms.is_none_or(|outer| intent.deadline_unix_ms <= outer)
            && receipt.reserved_bytes <= receipt.max_output_bytes,
        "native receipt intent or aggregate budget differs"
    );
    let expected = operation_slots(prepared, run)?.into_iter().collect::<BTreeSet<_>>();
    ensure!(
        intent.operation_metadata_bytes.is_some() == receipt.operation_index.is_some(),
        "native operation storage mode changed"
    );
    if let Some(maximum) = intent.operation_metadata_bytes {
        ensure!(
            maximum == operation_storage::capacity(expected.len())?,
            "native operation metadata reservation changed"
        );
    }
    let mut observed = BTreeSet::new();
    let mut reservations = Vec::new();
    receipt.visit_operations(|operation| {
        bounded::check()?;
        ensure!(
            expected.contains(&(operation.directory.clone(), operation.label.clone()))
                && observed.insert((operation.directory.clone(), operation.label.clone())),
            "native producer receipt is unexpected or duplicated"
        );
        let (passed, reservation) =
            authenticate_operation(prepared, &receipt.operation_root, &receipt.context, operation)?;
        ensure!(!receipt.passed || passed, "accepted native family contains a rejected producer");
        reservations.extend(reservation);
        Ok(())
    })?;
    if intent.reconcile_producer_reservations {
        ensure!(reservations.len() == observed.len(), "native producer reservation record missing");
        let retained = builds::authenticate_reservations(
            intent.parent_reservation_bytes,
            &mut reservations,
            intent.max_output_bytes,
        )?;
        ensure!(
            receipt.reserved_bytes >= retained && (!receipt.passed || receipt.reserved_bytes == retained),
            "native completed reservation extent differs"
        );
        ensure!(receipt.evidence_bytes <= receipt.reserved_bytes, "native evidence exceeds retained reservation");
    } else {
        ensure!(reservations.is_empty(), "historical native reservation mode changed");
    }
    ensure!(
        !receipt.passed || observed.len() == expected.len(),
        "native fixture pair compiler/execution receipts incomplete"
    );
    ensure!(
        receipt.evidence_bytes == evidence_bytes(run, receipt)? && receipt.evidence_bytes <= receipt.max_output_bytes,
        "native retained evidence extent differs"
    );
    if !receipt.passed {
        let verdict: Verdict = family::object(&path)?;
        ensure!(
            verdict.status == "rejected" && verdict.error.as_deref() == Some(&receipt.diagnostic),
            "native failure receipt differs"
        );
        let mut actual = BTreeMap::new();
        bounded::discovery(|| family::inventory(&verdict.run, &verdict.run, &mut actual))?;
        ensure!(actual == verdict.artifacts, "native failed artifact inventory differs");
        return Ok(verdict);
    }
    let verdict = bounded::discovery(|| {
        family::authenticate_with_source(
            crate::family_model::AuthenticateRequest { checkout, verdict: path },
            &prepared.pinned_source,
        )
    })?;
    ensure!(
        verdict.family == prepared.contract.family && verdict.source.as_ref() == Some(&prepared.source),
        "native family source identity differs"
    );
    let declared: Manifest = family::object(&verdict.run.join("manifest.toml"))?;
    let declared = if matches!(prepared.contract.recipe, Some(NativeRecipe::Plm)) {
        crate::plm_spawn::normalize(&declared)?
    } else {
        declared
    };
    ensure!(coverage(&declared)? == prepared.contract.coverage, "native frozen coverage differs");
    Ok(verdict)
}

fn operation_slots(prepared: &PreparedNative, run: &Path) -> Result<Vec<(PathBuf, String)>> {
    let mut result = vec![(run.to_path_buf(), "c-build".into()), (run.to_path_buf(), "rust-build".into())];
    if recipes::abi(prepared.contract.recipe.as_ref()) {
        result.push((run.to_path_buf(), "abi-controls".into()));
    }
    for (_, fixture, _) in family::ordered_tasks(&prepared.manifest)? {
        bounded::check()?;
        result.push((run.join(&fixture.name), "rust".into()));
        result.push((run.join(&fixture.name), "c".into()));
    }
    Ok(result)
}

fn collect_operations(
    prepared: &PreparedNative,
    run: &Path,
    root: &Path,
    context: &OperationContext,
) -> Result<Vec<NativeOperation>> {
    let mut operations = Vec::new();
    for (directory, label) in operation_slots(prepared, run)? {
        bounded::check()?;
        let invocation_path = directory.join(format!("{label}.invocation.toml"));
        if !invocation_path.is_file() {
            continue;
        }
        let invocation: builds::StrictInvocation = family::object(&invocation_path)?;
        if !translation_execution::operation::directory(root, &invocation.context)?.join("result.json").exists() {
            continue;
        }
        let result = translation_execution::operation::authenticate_request(
            root,
            &invocation.context,
            &invocation.program,
            &invocation.limits,
            &[],
        )?;
        let operation = NativeOperation {
            directory,
            label,
            invocation: Asset { file: invocation_path.clone(), sha256: digest(&invocation_path)? },
            context: invocation.context.clone(),
            result: translation_execution::operation::reference(root, &invocation.context)?,
            result_sha256: result.result_sha256,
            references: translation_execution::operation::references(root, &invocation.context)?,
        };
        authenticate_operation(prepared, root, context, &operation)?;
        operations.push(operation);
    }
    Ok(operations)
}

fn authenticate_operation(
    prepared: &PreparedNative,
    root: &Path,
    context: &OperationContext,
    operation: &NativeOperation,
) -> Result<(bool, Option<builds::ProducerReservation>)> {
    let invocation_path = file(&operation.invocation.file)?;
    ensure!(
        invocation_path == operation.directory.join(format!("{}.invocation.toml", operation.label))
            && digest(&invocation_path)? == operation.invocation.sha256,
        "native invocation reference changed"
    );
    let invocation: builds::StrictInvocation = family::object(&invocation_path)?;
    let mut expected_context = context.clone();
    expected_context.operation_key = sealed(&(&context.operation_key, &operation.directory, &operation.label))?;
    ensure!(
        operation.context == expected_context
            && invocation.context == expected_context
            && invocation.request_sha256 == sealed(&(&invocation.program, &invocation.requested_limits))?,
        "native producer context/request changed"
    );
    let configured = if operation.label.ends_with("-build") {
        prepared.contract.compiler_limits.run_limits()?
    } else {
        prepared.contract.fixture_limits.run_limits()?
    };
    ensure!(
        configured == invocation.requested_limits
            && invocation.limits.timeout <= configured.timeout
            && invocation.limits.writable_bytes == configured.writable_bytes
            && invocation.limits.stdout_bytes == configured.stdout_bytes
            && invocation.limits.stderr_bytes == configured.stderr_bytes
            && invocation.limits.stdin_bytes == configured.stdin_bytes,
        "native installed producer limits changed"
    );
    let compiler = operation.label.ends_with("-build");
    let rust = operation.label == "rust-build";
    let family_run = if compiler || operation.label == "abi-controls" {
        operation.directory.clone()
    } else {
        operation.directory.parent().context("native fixture family parent missing")?.to_path_buf()
    };
    let intent: NativeIntent =
        family::object(&family_run.parent().context("native output parent missing")?.join("native-intent.toml"))?;
    ensure!(
        invocation.reservation_before_bytes.is_some() == intent.reconcile_producer_reservations,
        "native authenticated reservation mode changed"
    );
    ensure!(invocation.program.helper == intent.helper, "native helper configuration changed");
    let candidate_root = if compiler && rust {
        let candidate = invocation
            .program
            .inputs
            .iter()
            .find(|input| input.destination == Path::new("/inputs/rust").join(&prepared.candidate_path))
            .context("confined candidate compiler input missing")?;
        let mut checkout = candidate.source.clone();
        for _ in Path::new(&prepared.candidate_path).components() {
            checkout.pop();
        }
        Some(checkout)
    } else {
        None
    };
    let mut expected_inputs = BTreeSet::new();
    if compiler {
        ensure!(
            invocation.program.executable == if rust { intent.rustc } else { intent.c_compiler }
                && sealed(&invocation.program.runtime)? == intent.compiler_runtime_sha256,
            "native compiler installed capabilities changed"
        );
        for name in if rust { &prepared.rust_inputs } else { &prepared.c_inputs } {
            let source = if rust {
                // Compiler inputs use the exact frozen candidate checkout path
                // already retained by its family implementation identity.
                candidate_root.as_ref().context("confined candidate checkout missing")?.join(name)
            } else {
                prepared.pinned_source.join(name)
            };
            expected_inputs.insert((source, Path::new(if rust { "/inputs/rust" } else { "/inputs/c" }).join(name)));
        }
        let (generated, outputs, headless) = recipes::compiler_files(prepared.contract.recipe.as_ref(), rust);
        for name in generated {
            expected_inputs.insert((operation.directory.join(name), Path::new("/inputs/generated").join(name)));
        }
        if headless {
            for name in ["headless.c", "headless_frame.h"] {
                // The first retained recipe stop predates the complete header
                // closure. It remains a valid failed receipt, never readiness.
                if name == "headless_frame.h"
                    && !prepared.contract.frozen_inputs.contains_key("tools/decomp/headless_frame.h")
                {
                    continue;
                }
                let destination = Path::new("/inputs/rust/tools/decomp").join(name);
                let input = invocation
                    .program
                    .inputs
                    .iter()
                    .find(|i| i.destination == destination)
                    .context("native headless source missing")?;
                ensure!(
                    prepared.contract.frozen_inputs.get(&format!("tools/decomp/{name}"))
                        == Some(&digest(&input.source)?),
                    "native headless source changed"
                );
                expected_inputs.insert((input.source.clone(), destination));
            }
        }
        let outputs = outputs.into_iter().map(PathBuf::from).collect::<BTreeSet<_>>();
        ensure!(
            invocation.program.outputs.iter().cloned().collect::<BTreeSet<_>>() == outputs,
            "native compiler outputs changed"
        );
    } else {
        let expected =
            recipes::fixture_program(prepared, &family_run, &operation.directory, &operation.label, &intent.helper)?;
        ensure!(invocation.program == expected, "native fixture arguments, outputs or capabilities changed");
        expected_inputs.extend(expected.inputs.into_iter().map(|i| (i.source, i.destination)));
    }
    ensure!(
        invocation
            .program
            .inputs
            .iter()
            .map(|input| (input.source.clone(), input.destination.clone()))
            .collect::<BTreeSet<_>>()
            == expected_inputs,
        "native compiler/fixture read capabilities changed"
    );
    let result = translation_execution::operation::authenticate_request(
        root,
        &expected_context,
        &invocation.program,
        &invocation.limits,
        &[],
    )?;
    ensure!(
        result.result_sha256 == operation.result_sha256
            && translation_execution::operation::reference(root, &expected_context)? == operation.result
            && translation_execution::operation::references(root, &expected_context)? == operation.references,
        "native retained producer evidence changed"
    );
    for reference in &operation.references {
        translation_execution::retention::authenticate_reference(reference, invocation.reserved_bytes)?;
    }
    ensure!(
        bounded::read(operation.directory.join(format!("{}.stdout", operation.label)))? == result.stdout
            && bounded::read(operation.directory.join(format!("{}.stderr", operation.label)))? == result.stderr,
        "native parent logs differ from confined producer"
    );
    if !result.diagnostic.is_empty() {
        ensure!(
            bounded::text(operation.directory.join(format!("{}.diagnostic.txt", operation.label)))?
                == result.diagnostic,
            "native producer failure diagnostic changed"
        );
    }
    for (name, bytes) in &result.outputs {
        let destination = builds::strict_output_path(&operation.directory, &operation.label, name);
        ensure!(bounded::read(destination)? == *bytes, "native parent output differs from confined producer");
    }
    if result.passed
        && !compiler
        && matches!(
            prepared.contract.recipe,
            Some(NativeRecipe::Callback | NativeRecipe::PowerBomb | NativeRecipe::GameHost)
        )
    {
        crate::callback::authenticate_packed_snapshots(&operation.directory, &operation.label)?;
    }
    let reservation = invocation
        .reservation_before_bytes
        .map(|before_bytes| -> Result<builds::ProducerReservation> {
            Ok(builds::ProducerReservation {
                before_bytes,
                reserved_bytes: invocation.reserved_bytes,
                retained_bytes: builds::retained_producer_bytes(
                    &invocation,
                    &result,
                    &operation.references,
                    &operation.directory,
                    &operation.label,
                )?,
            })
        })
        .transpose()?;
    Ok((result.passed, reservation))
}

/// Parent artifacts and all durable runner dependencies, including historical
/// installed input manifests, belong to the retention graph.
pub(crate) fn references(receipt: &NativeReceipt) -> Result<Vec<Reference>> {
    bounded::until(now_unix_ms()?.saturating_add(60_000), 8 * bounded::DEFAULT_BYTES, || references_bounded(receipt))
}

fn references_bounded(receipt: &NativeReceipt) -> Result<Vec<Reference>> {
    let run = receipt.verdict.file.parent().context("native verdict parent missing")?;
    let output = run.parent().context("native output parent missing")?;
    let mut result = BTreeMap::new();
    fn files(run: &Path, path: &Path, result: &mut BTreeMap<PathBuf, Reference>, depth: usize) -> Result<()> {
        ensure!(depth <= 32, "native retention tree depth exceeded");
        for path in bounded::entries(path)? {
            bounded::safe(&path)?;
            let metadata = path.symlink_metadata()?;
            if metadata.is_dir() {
                files(run, &path, result, depth + 1)?;
            } else {
                ensure!(metadata.is_file(), "native retention contains special file");
                result.insert(
                    path.clone(),
                    Reference {
                        path: path.clone(),
                        sha256: inputs::artifact_digest(run, &path)?,
                        bytes: path.metadata()?.len(),
                    },
                );
            }
        }
        Ok(())
    }
    bounded::discovery(|| files(run, run, &mut result, 0))?;
    for name in ["native-intent.toml", "native-receipt.toml"] {
        let path = output.join(name);
        if path.exists() {
            result.insert(
                path.clone(),
                Reference { path: path.clone(), sha256: digest(&path)?, bytes: path.metadata()?.len() },
            );
        }
    }
    if receipt.operation_index.is_some() {
        for reference in operation_storage::references(receipt)? {
            result.insert(reference.path.clone(), reference);
        }
    }
    receipt.visit_operations(|operation| {
        bounded::check()?;
        for reference in &operation.references {
            bounded::check()?;
            result.insert(reference.path.clone(), reference.clone());
            ensure!(result.len() <= 65536, "native reference inventory exceeded");
        }
        Ok(())
    })?;
    Ok(result.into_values().collect())
}

fn evidence_bytes(run: &Path, receipt: &NativeReceipt) -> Result<u64> {
    let mut seen = BTreeSet::new();
    let mut bytes = tree_bytes(run)?;
    if receipt.operation_index.is_some() {
        for reference in operation_storage::references(receipt)? {
            bytes = bytes.checked_add(reference.bytes).context("native operation metadata extent overflow")?;
            seen.insert(reference.path);
        }
    }
    receipt.visit_operations(|operation| {
        bounded::check()?;
        for reference in &operation.references {
            bounded::check()?;
            if !reference.path.starts_with(run) && seen.insert(reference.path.clone()) {
                bytes = bytes.checked_add(reference.bytes).context("native retained byte overflow")?;
            }
        }
        Ok(())
    })?;
    Ok(bytes)
}

fn parent_reservation(prepared: &PreparedNative, checkout: &Path) -> Result<u64> {
    let mut bytes = 2 * 1024 * 1024u64; // bounded verdict, native receipts and generated declarations
    let operations = operation_slots(prepared, checkout)?.len();
    bytes = bytes
        .checked_add(operation_storage::capacity(operations)?)
        .context("native operation metadata reservation overflow")?;
    for name in prepared.contract.frozen_inputs.keys() {
        bounded::check()?;
        let path = file(&checkout.join(name))?;
        bytes = bytes
            .checked_add(path.metadata()?.len().checked_mul(2).context("native source extent overflow")?)
            .context("native source snapshot extent overflow")?;
    }
    // Preserve the historical conservative reservation; authentication now
    // compares the archive in bounded Rust buffers without temporary extraction.
    for name in prepared.source.files.keys() {
        bounded::check()?;
        let path = file(&prepared.pinned_source.join(name))?;
        bytes = bytes
            .checked_add(path.metadata()?.len().checked_mul(2).context("native pinned extent overflow")?)
            .context("native pinned-source authentication extent overflow")?;
    }
    bytes = bytes
        .checked_add(prepared.pinned_source.join("LICENSE.txt").metadata()?.len())
        .context("native license extent overflow")?;
    if let Some(rom) = &prepared.manifest.rom {
        bytes =
            bytes.checked_add(check_asset(checkout, rom)?.metadata()?.len()).context("native ROM extent overflow")?;
    }
    for (_, case, _) in family::ordered_tasks(&prepared.manifest)? {
        bounded::check()?;
        bytes = bytes
            .checked_add(0x20000 + 16 * 1024 + if case.expected_after_file.is_some() { 0x20000 } else { 0 })
            .context("native fixture extent overflow")?;
    }
    Ok(bytes)
}

fn tree_bytes(path: &Path) -> Result<u64> {
    bounded::discovery(|| Ok(bounded::tree(path, 65536, 8 * bounded::DEFAULT_BYTES)?.1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_receipt_storage_preserves_large_operation_sets_within_controller_record_bound() -> Result<()> {
        let directory = bounded::tests::Directory::new()?;
        let mut retained = directory.0.join("retained");
        for level in 0..16 {
            retained.push(format!("{level}-{}", "source".repeat(16)));
        }
        fs::create_dir_all(&retained)?;
        let retained = retained.join("result.json");
        fs::write(&retained, b"retained producer storage control")?;
        let reference = translation_execution::retention::seal_file(&retained, 4096)?;
        let context = OperationContext {
            run_id: sha256("receipt-storage"),
            attempt: 1,
            policy_sha256: sha256("frozen-policy"),
            operation_key: sha256("native-family"),
            phase: translation_execution::model::ExecutionPhase::Candidate,
        };
        let operations = (0..2048)
            .map(|index| {
                let mut operation_context = context.clone();
                operation_context.operation_key = sha256(format!("producer-{index}"));
                NativeOperation {
                    directory: directory.0.join(format!("family/pair-{index}")),
                    label: "rust".into(),
                    invocation: Asset { file: reference.path.clone(), sha256: reference.sha256.clone() },
                    context: operation_context,
                    result: reference.clone(),
                    result_sha256: reference.sha256.clone(),
                    references: vec![reference.clone()],
                }
            })
            .collect::<Vec<_>>();
        // This exercises storage only; these declarations grant no producer or
        // Offline acceptance. Actual invocation authentication remains separate.
        let expected = operations.iter().map(|operation| operation.context.operation_key.clone()).collect::<Vec<_>>();
        let mut receipt = NativeReceipt {
            prepared_sha256: sha256("prepared"),
            candidate_sha256: sha256("candidate"),
            verdict: Asset { file: directory.0.join("family/verdict.toml"), sha256: sha256("verdict") },
            passed: false,
            diagnostic: "storage control only".into(),
            evidence_bytes: 0,
            context,
            operation_root: directory.0.join("operations"),
            operations,
            operation_index: None,
            max_output_bytes: 64 * 1024 * 1024,
            reserved_bytes: 64 * 1024 * 1024,
        };
        receipt.store_operations(&directory.0)?;
        let encoded = serde_json::to_vec(&receipt)?;
        ensure!(
            encoded.len() <= crate::translation_original::MAX_RECORD as usize,
            "complete native producer attribution exceeds the controller record bound: {} bytes",
            encoded.len()
        );
        let restored: NativeReceipt = serde_json::from_slice(&encoded)?;
        let mut actual = Vec::new();
        restored.visit_operations(|operation| {
            actual.push(operation.context.operation_key.clone());
            Ok(())
        })?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn frozen_shared_host_contracts_preserve_both_original_matrices() -> Result<()> {
        let checkout = current_checkout();
        let limits = NativeLimits {
            timeout_seconds: 60,
            writable_bytes: 128 * 1024 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 256 * 1024,
            stdin_bytes: 0,
        };
        let mut pairs = 0;
        for name in ["game-host-projectile", "game-host-room-scroll"] {
            let contract = contract_for_family(
                &checkout,
                &PathBuf::from(format!("tools/port/{name}.toml")),
                Asset::default(),
                NativeProvenance::NativeScope { scope: Asset::default(), checkout: checkout.clone() },
                limits.clone(),
                limits.clone(),
            )?;
            pairs += contract.coverage.values().map(Vec::len).sum::<usize>();
        }
        assert_eq!(pairs, 10);
        Ok(())
    }

    #[test]
    fn discovered_recipe_rejects_a_declaration_from_an_alias_path() -> Result<()> {
        let directory = bounded::tests::Directory::new()?;
        let checkout = &directory.0;
        fs::create_dir_all(checkout.join("tools/port"))?;
        fs::write(checkout.join("tools/port/batch.toml"), "[families]\n[profiles]\n")?;
        let manifest = Manifest { family: "registered".into(), ..Default::default() };
        fs::write(checkout.join("tools/port/alias.toml"), toml::to_string(&manifest)?)?;
        let limits = NativeLimits {
            timeout_seconds: 1,
            writable_bytes: 1024 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 4096,
            stdin_bytes: 0,
        };
        let error = contract_for_family(
            checkout,
            Path::new("tools/port/alias.toml"),
            Asset::default(),
            NativeProvenance::NativeScope { scope: Asset::default(), checkout: checkout.clone() },
            limits.clone(),
            limits,
        )
        .unwrap_err();
        ensure!(error.to_string().contains("registration path differs"), "unexpected rejection: {error:#}");
        Ok(())
    }

    #[test]
    fn registered_catalog_alias_preserves_declared_family_and_coverage() -> Result<()> {
        let checkout = current_checkout();
        let path = Path::new("tools/port/instructions.toml");
        let declared: Manifest = family::object(&checkout.join(path))?;
        let limits = NativeLimits {
            timeout_seconds: 60,
            writable_bytes: 128 * 1024 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 4096,
            stdin_bytes: 0,
        };
        let contract = contract_for_family(
            &checkout,
            path,
            Asset::default(),
            NativeProvenance::NativeScope { scope: Asset::default(), checkout: checkout.clone() },
            limits.clone(),
            limits,
        )?;
        assert_eq!(contract.family, "enemy-instructions");
        assert_eq!(contract.coverage, coverage(&declared)?);
        let registration = recipes::registration(&checkout, &contract, &declared)?;
        assert_eq!(registration.catalog_family, "instructions");
        assert_eq!(registration.entry.manifest, path);
        Ok(())
    }

    #[test]
    fn discovered_captured_ridley_keeps_its_original_callback_recipe() -> Result<()> {
        let checkout = current_checkout();
        let path = Path::new("tools/port/ridley-subsystem-captured.toml");
        let original: Manifest = family::object(&checkout.join(path))?;
        let recipe = recipes::for_family(&checkout, path, &original)?;
        assert_eq!(recipe, Some(NativeRecipe::Ridley));
        let normalized = recipes::normalize_manifest(&checkout, original.clone(), recipe.as_ref())?;
        assert_eq!(normalized, original);
        assert!(normalized.routines.iter().flat_map(|routine| &routine.cases).any(|case| case.before_file.is_some()));
        let limits = NativeLimits {
            timeout_seconds: 1,
            writable_bytes: 1024 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 4096,
            stdin_bytes: 0,
        };
        let contract = contract_for_family(
            &checkout,
            path,
            Asset::default(),
            NativeProvenance::NativeScope { scope: Asset::default(), checkout: checkout.clone() },
            limits.clone(),
            limits,
        )?;
        assert_eq!(contract.coverage, coverage(&original)?);
        assert!(original.rust_dependencies.len() > 4096);
        assert!(contract.frozen_inputs.len() > 4096 && contract.frozen_inputs.len() <= 8192);
        let event = Path::new(inputs::CAPTURED_RIDLEY_EVENT);
        assert!(event.metadata()?.len() > bounded::MAX_FILE_BYTES);
        assert!(recipes::dependency_digest(None, event).is_err());
        assert!(recipes::dependency_digest(Some(&NativeRecipe::Callback), event).is_err());
        assert_eq!(
            contract.frozen_inputs[inputs::CAPTURED_RIDLEY_EVENT],
            recipes::dependency_digest(Some(&NativeRecipe::Ridley), event)?
        );
        let mut unknown = original;
        unknown.callback_adapter = Some("unknown-captured-adapter".into());
        assert!(recipes::for_family(&checkout, path, &unknown).is_err());
        Ok(())
    }

    #[test]
    #[ignore = "real separately confined actual-C controls; retains source, replay and cleanup evidence"]
    fn real_reviewed_recipes_replay_and_authenticate() -> Result<()> {
        let _ = crate::BINARY_SHA256.set(crate::digest(&std::env::current_exe()?)?);
        let root = PathBuf::from(std::env::var("SM_NATIVE_RECIPE_PROOF")?);
        fs::create_dir_all(&root)?;
        let current = current_checkout();
        let checkout = root.join("source");
        let selection = std::env::var("SM_NATIVE_RECIPE_FAMILIES")
            .unwrap_or_else(|_| "game-host-projectile,game-host-room-scroll".into());
        let names = selection.split(',').collect::<Vec<_>>();
        let mut paths = BTreeSet::from([
            "tools/port/batch.toml".to_owned(),
            "tools/dependencies.toml".into(),
            "tools/decomp/headless.c".into(),
            "tools/decomp/headless_frame.h".into(),
            "tools/port/source-inventory.toml".into(),
        ]);
        for directory in ["crates/pipeline/src", "crates/port/src"] {
            for entry in fs::read_dir(current.join(directory))? {
                let path = entry?.path();
                if path.is_file() {
                    paths.insert(path.strip_prefix(&current)?.display().to_string());
                }
            }
        }
        for name in &names {
            if *name == "pause-direct-animation" {
                paths.extend([
                    "tools/port/pause-equipment-render.toml".into(),
                    ".scratch/decomp-coverage/renderer/direct-animation.toml".into(),
                ]);
            } else {
                paths.insert(format!("tools/port/{name}.toml"));
            }
        }
        if names.contains(&"plm-instructions") {
            paths.insert("rom/sm.sfc".into());
        }
        for path in paths {
            fs::create_dir_all(checkout.join(&path).parent().unwrap())?;
            immutable(&checkout.join(&path), &bounded::read(current.join(&path))?)?;
        }
        if names.contains(&"plm-instructions") {
            prepare_generated_plm(&checkout, &checkout.join(".scratch/translation-native/plm-instructions"))?;
        }
        let catalog = root.join("catalog.toml");
        immutable(&catalog, toml::to_string(&batch::catalog(&checkout)?)?.as_bytes())?;
        let requirements = root.join("requirements.md");
        immutable(
            &requirements,
            &bounded::read(
                current.join(".scratch/translation-scope-execution/issues/10-original-acceptance-adapters.md"),
            )?,
        )?;
        let inventory = Asset {
            file: "tools/port/source-inventory.toml".into(),
            sha256: digest(&checkout.join("tools/port/source-inventory.toml"))?,
        };
        let provenance = NativeProvenance::ReviewedCatalog {
            catalog: Asset { file: catalog.clone(), sha256: digest(&catalog)? },
            requirements: Asset { file: requirements.clone(), sha256: digest(&requirements)? },
            checkout: checkout.clone(),
            rationale: "Ticket 10 public original recipe test with complete selected declarations and actual pinned C"
                .into(),
        };
        let compiler = NativeLimits {
            timeout_seconds: std::env::var("SM_NATIVE_RECIPE_COMPILER_SECONDS")
                .ok()
                .map(|s| s.parse())
                .transpose()?
                .unwrap_or(180),
            writable_bytes: 128 * 1024 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 256 * 1024,
            stdin_bytes: 0,
        };
        let fixture = NativeLimits {
            timeout_seconds: 5,
            writable_bytes: 16 * 1024 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 4096,
            stdin_bytes: 0,
        };
        let mut completed = Vec::new();
        let mut failures = Vec::new();
        let pinned = Path::new("/home/jim/sm-decomp/deps/snesrev-sm");
        let inputs = prepare_catalog_inputs_until(
            &checkout,
            pinned,
            &inventory,
            now_unix_ms()? + 600_000,
            8 * bounded::DEFAULT_BYTES,
        )?;
        for name in names {
            let output = root.join(name);
            fs::create_dir_all(&output)?;
            let contract = if name == "pause-direct-animation" {
                contract_for_direct_animation(
                    &checkout,
                    Path::new(".scratch/decomp-coverage/renderer/direct-animation.toml"),
                    inventory.clone(),
                    provenance.clone(),
                    compiler.clone(),
                    fixture.clone(),
                )?
            } else {
                contract_for_family(
                    &checkout,
                    Path::new(&format!("tools/port/{name}.toml")),
                    inventory.clone(),
                    provenance.clone(),
                    compiler.clone(),
                    fixture.clone(),
                )?
            };
            let declared =
                recipes::manifest(&checkout, &checkout.join(&contract.manifest.file), contract.recipe.as_ref())?;
            let prepared = prepare_scope_in_catalog_until(
                &checkout,
                pinned,
                &contract,
                &[declared.rust_module],
                &inputs,
                now_unix_ms()? + 600_000,
                8 * bounded::DEFAULT_BYTES,
            )?;
            immutable(&output.join("prepared.toml"), toml::to_string(&prepared)?.as_bytes())?;
            let runtime = StrictRuntime {
                helper: std::env::var("SM_NATIVE_RECIPE_HELPER")
                    .unwrap_or_else(|_| "/home/jim/sm-decomp/out/translation-production/sm-translation-sandbox".into())
                    .into(),
                rustc: "/home/jim/.rustup/toolchains/nightly-2026-09-29-x86_64-unknown-linux-gnu/bin/rustc".into(),
                c_compiler: "/usr/bin/x86_64-linux-gnu-gcc-9".into(),
                operation_root: output.join("operations"),
                pipeline_root: "/home/jim/sm-decomp/out/pipeline".into(),
                workers: 4,
                gate_timeout_seconds: 600,
                deadline_unix_ms: None,
                max_output_bytes: 64 * 1024 * 1024 * 1024,
            };
            let context = OperationContext {
                run_id: sha256(root.display().to_string()),
                attempt: 1,
                policy_sha256: prepared.verification_sha256.clone(),
                operation_key: sha256(name),
                phase: translation_execution::model::ExecutionPhase::Candidate,
            };
            let receipt = run(&prepared, &checkout, &output.join("run"), &runtime, &context)?;
            let verdict = authenticate(&prepared, &checkout, &receipt)?;
            let mut identities = Vec::new();
            receipt.visit_operations(|operation| {
                identities.push((operation.result.path.clone(), operation.result.path.metadata()?.modified()?));
                Ok(())
            })?;
            ensure!(
                sealed(&run(&prepared, &checkout, &output.join("run"), &runtime, &context)?)? == sealed(&receipt)?,
                "recipe replay changed receipt"
            );
            for (path, modified) in identities {
                ensure!(path.metadata()?.modified()? == modified, "recipe replay changed producer");
            }
            for deduplicate in [false, true] {
                let cleanup = crate::cleanup::cleanup(crate::cleanup::CleanupRequest {
                    root: output.join("run"),
                    apply: true,
                    deduplicate,
                })?;
                atomic_toml(
                    &output.join(if deduplicate { "cleanup-deduplicated.toml" } else { "cleanup.toml" }),
                    &cleanup,
                )?;
            }
            authenticate(&prepared, &checkout, &receipt)?;
            if receipt.passed {
                if name == "game-host-room-scroll" {
                    ensure!(
                        verdict.cases.len() == 3 && verdict.cases.iter().all(|case| case.callbacks == 17),
                        "original VRAM flush effects missing"
                    );
                }
                if name == "game-host-projectile" {
                    ensure!(
                        verdict.cases.len() == 7
                            && verdict.cases[0].callbacks == 2
                            && verdict.cases[1].callbacks == 2
                            && verdict.cases[2].callbacks >= 1,
                        "original HDMA/PLM material controls missing"
                    );
                }
                if name == "pause-direct-animation" {
                    ensure!(verdict.verified_cases == 360, "original direct animation extent changed");
                }
            } else {
                failures.push(format!("{name}: {}", receipt.diagnostic));
            }
            completed.push((name.to_string(), receipt.passed, verdict.verified_cases, receipt.diagnostic));
        }
        atomic_toml(&root.join("summary.toml"), &BTreeMap::from([("families", completed)]))?;
        ensure!(failures.is_empty(), "{}", failures.join("; "));
        Ok(())
    }

    #[test]
    #[ignore = "freezes every current catalog declaration with real pinned source; no producer is launched"]
    fn complete_current_catalog_has_a_frozen_native_recipe() -> Result<()> {
        let root = PathBuf::from(std::env::var("SM_NATIVE_CATALOG_PROOF")?);
        fs::create_dir_all(&root)?;
        let checkout = std::env::var_os("SM_NATIVE_CATALOG_SOURCE").map(PathBuf::from).unwrap_or_else(current_checkout);
        prepare_generated_plm(&checkout, &checkout.join(".scratch/translation-native/plm-instructions"))?;
        let catalog = batch::catalog(&checkout)?;
        let catalog_path = root.join("catalog.toml");
        immutable(&catalog_path, toml::to_string(&catalog)?.as_bytes())?;
        let requirements =
            checkout.join(".scratch/translation-scope-execution/issues/10-original-acceptance-adapters.md");
        let provenance = NativeProvenance::ReviewedCatalog { catalog: Asset { file: catalog_path.clone(), sha256: digest(&catalog_path)? },
            requirements: Asset { file: requirements.clone(), sha256: digest(&requirements)? }, checkout: checkout.clone(),
            rationale: "Current conservative catalog: every registered declaration, all generated cases, exact original requirements".into() };
        let inventory = Asset {
            file: "tools/port/source-inventory.toml".into(),
            sha256: digest(&checkout.join("tools/port/source-inventory.toml"))?,
        };
        let pinned = Path::new("/home/jim/sm-decomp/deps/snesrev-sm");
        let deadline = now_unix_ms()? + 600_000;
        let inputs = prepare_catalog_inputs_until(&checkout, pinned, &inventory, deadline, 8 * bounded::DEFAULT_BYTES)?;
        let limits = NativeLimits {
            timeout_seconds: 180,
            writable_bytes: 128 * 1024 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 256 * 1024,
            stdin_bytes: 0,
        };
        let fixture = NativeLimits {
            timeout_seconds: 5,
            writable_bytes: 16 * 1024 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 4096,
            stdin_bytes: 0,
        };
        let mut summary = BTreeMap::new();
        let mut failures = Vec::new();
        for name in catalog.families.keys() {
            let result = (|| -> Result<usize> {
                let entry = batch::entry(&catalog, name)?;
                let contract = contract_for_family_until(
                    &checkout,
                    &entry.manifest,
                    inventory.clone(),
                    provenance.clone(),
                    limits.clone(),
                    fixture.clone(),
                    deadline,
                    8 * bounded::DEFAULT_BYTES,
                )?;
                let prepared = bounded::until(deadline, 8 * bounded::DEFAULT_BYTES, || {
                    prepare_bounded_with_catalog(&checkout, pinned, &contract, Some(&inputs))
                })?;
                let output = root.join(format!("{name}.toml"));
                immutable(&output, toml::to_string(&prepared)?.as_bytes())?;
                Ok(contract.coverage.values().map(Vec::len).sum())
            })();
            match result {
                Ok(cases) => {
                    summary.insert(name.clone(), format!("prepared {cases} fixture pairs"));
                }
                Err(error) => {
                    summary.insert(name.clone(), format!("rejected: {error:#}"));
                    failures.push(format!("{name}: {error:#}"));
                }
            }
            atomic_toml(&root.join("summary.toml"), &summary)?;
        }
        ensure!(summary.len() == catalog.families.len(), "catalog entry omitted");
        let direct = (|| -> Result<usize> {
            let contract = bounded::until(deadline, 8 * bounded::DEFAULT_BYTES, || {
                contract_for_direct_animation(
                    &checkout,
                    Path::new(".scratch/decomp-coverage/renderer/direct-animation.toml"),
                    inventory,
                    provenance,
                    limits,
                    fixture,
                )
            })?;
            let prepared = bounded::until(deadline, 8 * bounded::DEFAULT_BYTES, || {
                prepare_bounded_with_catalog(&checkout, pinned, &contract, Some(&inputs))
            })?;
            let cases = contract.coverage.values().map(Vec::len).sum();
            ensure!(contract.family == "pause-direct-animation" && cases == 360, "auxiliary animation domain changed");
            immutable(&root.join("pause-direct-animation.toml"), toml::to_string(&prepared)?.as_bytes())?;
            Ok(cases)
        })();
        let result = match direct {
            Ok(cases) => format!("prepared {cases} fixture pairs"),
            Err(error) => {
                failures.push(format!("pause-direct-animation: {error:#}"));
                format!("rejected: {error:#}")
            }
        };
        atomic_toml(&root.join("auxiliary-summary.toml"), &BTreeMap::from([("pause-direct-animation", result)]))?;
        ensure!(failures.is_empty(), "{}", failures.join("; "));
        Ok(())
    }

    #[test]
    fn scope_candidate_dependencies_are_mutable_and_bound_to_receipt_identity() -> Result<()> {
        let directory = bounded::tests::Directory::new()?;
        let checkout = &directory.0;
        fs::create_dir_all(checkout.join("crates/port/src"))?;
        let names = ["root", "helper", "fixed"].map(|name| format!("crates/port/src/{name}.rs"));
        let mut frozen_inputs = BTreeMap::new();
        for name in &names {
            fs::write(checkout.join(name), format!("// original {name}"))?;
            frozen_inputs.insert(name.clone(), digest(&checkout.join(name))?);
        }
        fs::write(checkout.join("manifest.toml"), b"family='test'")?;
        let limits =
            NativeLimits { timeout_seconds: 1, writable_bytes: 4096, stdout_bytes: 0, stderr_bytes: 0, stdin_bytes: 0 };
        let baseline = PreparedNative {
            contract: NativeContract {
                family: "test".into(),
                manifest: Asset { file: "manifest.toml".into(), sha256: digest(&checkout.join("manifest.toml"))? },
                source_inventory: Asset::default(),
                coverage: BTreeMap::new(),
                provenance: NativeProvenance::NativeScope { scope: Asset::default(), checkout: checkout.clone() },
                frozen_inputs,
                compiler_limits: limits.clone(),
                fixture_limits: limits,
                recipe: None,
            },
            manifest: Manifest::default(),
            source: Source::default(),
            pinned_source: checkout.clone(),
            candidate_path: names[0].clone(),
            approved_candidate_paths: None,
            rust_inputs: names.to_vec(),
            c_inputs: Vec::new(),
            dependencies_sha256: sha256("dependencies"),
            verification_sha256: sha256("verification"),
        };
        let mut single = baseline.clone();
        bind_scope_candidates(&mut single, &[names[0].clone()])?;
        assert_eq!(sealed(&single)?, sealed(&baseline)?);
        assert!(!toml::to_string(&single)?.contains("approved_candidate_paths"));
        let mut scoped = baseline.clone();
        bind_scope_candidates(&mut scoped, &names[..2])?;
        assert_ne!(scoped.dependencies_sha256, baseline.dependencies_sha256);
        let original = candidate_digest(&scoped, checkout)?;
        fs::write(checkout.join(&names[1]), b"// changed approved helper")?;
        check_candidate(&scoped, checkout)?;
        assert_ne!(candidate_digest(&scoped, checkout)?, original);
        assert!(check_candidate(&baseline, checkout).is_err());
        fs::write(checkout.join(&names[2]), b"// changed frozen helper")?;
        assert!(check_candidate(&scoped, checkout).is_err());
        for name in &names {
            fs::write(checkout.join(name), format!("// original {name}"))?;
        }
        let mut dependency_only = baseline.clone();
        bind_scope_candidates(&mut dependency_only, &[names[1].clone()])?;
        fs::write(checkout.join(&names[0]), b"// unauthorized root change")?;
        assert!(check_candidate(&dependency_only, checkout).is_err());
        let mut uncertain = baseline.clone();
        bind_scope_candidates(&mut uncertain, &["crates/port/src/unrelated.rs".into()])?;
        assert_eq!(uncertain.approved_candidate_paths, Some(BTreeSet::new()));
        assert!(check_candidate(&uncertain, checkout).is_err());
        for invalid in ["crates/port/src/memory.rs", "crates/port/src/../helper.rs", "tools/port/family.toml"] {
            assert!(bind_scope_candidates(&mut baseline.clone(), &[invalid.into()]).is_err());
        }
        Ok(())
    }

    #[test]
    fn coverage_rejects_empty_duplicate_and_incomplete_registered_domain() {
        let fixture = crate::family_model::Case { name: "step".into(), ..Default::default() };
        let routine =
            crate::family_model::Routine { symbol: "Step".into(), cases: vec![fixture], ..Default::default() };
        let mut manifest = Manifest { routines: vec![routine.clone()], ..Default::default() };
        assert_eq!(coverage(&manifest).unwrap()["Step"], vec!["step"]);
        manifest.routines.push(routine);
        assert!(coverage(&manifest).is_err());
        manifest.routines.clear();
        assert!(coverage(&manifest).is_err());
        assert!(
            NativeLimits { timeout_seconds: 0, writable_bytes: 4096, stdout_bytes: 0, stderr_bytes: 0, stdin_bytes: 0 }
                .run_limits()
                .is_err()
        );
    }

    #[test]
    fn expired_outer_deadline_refuses_before_accessing_or_creating_any_source() -> Result<()> {
        let absent =
            std::env::temp_dir().join(format!("sm-expired-native-{}-{}", std::process::id(), family::unique_id()));
        let limits =
            NativeLimits { timeout_seconds: 1, writable_bytes: 4096, stdout_bytes: 0, stderr_bytes: 0, stdin_bytes: 0 };
        let prepared = PreparedNative {
            contract: NativeContract {
                family: "expired".into(),
                manifest: Asset { file: "manifest.toml".into(), sha256: sha256("manifest") },
                source_inventory: Asset { file: "inventory.toml".into(), sha256: sha256("inventory") },
                coverage: BTreeMap::new(),
                provenance: NativeProvenance::NativeScope {
                    scope: Asset { file: "scope.toml".into(), sha256: sha256("scope") },
                    checkout: absent.clone(),
                },
                frozen_inputs: BTreeMap::new(),
                compiler_limits: limits.clone(),
                fixture_limits: limits,
                recipe: None,
            },
            manifest: Manifest::default(),
            source: Source::default(),
            pinned_source: absent.clone(),
            candidate_path: "candidate.rs".into(),
            approved_candidate_paths: None,
            rust_inputs: Vec::new(),
            c_inputs: Vec::new(),
            dependencies_sha256: sha256("dependencies"),
            verification_sha256: sha256("verification"),
        };
        let mut runtime = StrictRuntime {
            helper: absent.join("helper"),
            rustc: absent.join("rustc"),
            c_compiler: absent.join("cc"),
            operation_root: absent.join("operations"),
            pipeline_root: absent.join("pipeline"),
            workers: 4,
            gate_timeout_seconds: 60,
            deadline_unix_ms: Some(0),
            max_output_bytes: 4096,
        };
        let context = OperationContext {
            run_id: sha256("expired"),
            attempt: 1,
            policy_sha256: sha256("policy"),
            operation_key: sha256("key"),
            phase: translation_execution::model::ExecutionPhase::Candidate,
        };
        let error = run(&prepared, &absent.join("source"), &absent.join("output"), &runtime, &context).unwrap_err();
        ensure!(
            error.to_string().contains("whole-stage time budget exhausted"),
            "expired outer stage accessed undeclared inputs: {error}"
        );
        ensure!(!absent.exists(), "expired outer stage created resource directories");
        runtime.deadline_unix_ms = Some(1001);
        ensure!(new_deadline(1000, &runtime)? == 1001, "outer stage did not cap family deadline");
        runtime.deadline_unix_ms = Some(u64::MAX);
        ensure!(new_deadline(1000, &runtime)? == 61000, "outer stage expanded local deadline");
        Ok(())
    }

    #[test]
    fn expired_saved_failure_replays_authentication_without_runtime_or_dispatch() -> Result<()> {
        let directory = bounded::tests::Directory::new()?;
        let checkout = directory.0.join("checkout");
        fs::create_dir(&checkout)?;
        let output = directory.0.join("output");
        let run_path = output.join("family");
        fs::create_dir_all(&run_path)?;
        fs::write(checkout.join("candidate.rs"), b"pub fn Step() {}")?;
        let manifest = Manifest { family: "failure".into(), rust_module: "candidate.rs".into(), ..Default::default() };
        fs::write(checkout.join("manifest.toml"), toml::to_string(&manifest)?)?;
        let limits =
            NativeLimits { timeout_seconds: 1, writable_bytes: 4096, stdout_bytes: 0, stderr_bytes: 0, stdin_bytes: 0 };
        let prepared = PreparedNative {
            contract: NativeContract {
                family: "failure".into(),
                manifest: Asset { file: "manifest.toml".into(), sha256: digest(&checkout.join("manifest.toml"))? },
                source_inventory: Asset::default(),
                coverage: BTreeMap::new(),
                provenance: NativeProvenance::NativeScope { scope: Asset::default(), checkout: checkout.clone() },
                frozen_inputs: BTreeMap::new(),
                compiler_limits: limits.clone(),
                fixture_limits: limits,
                recipe: None,
            },
            manifest,
            source: Source::default(),
            pinned_source: directory.0.join("unavailable-source"),
            candidate_path: "candidate.rs".into(),
            approved_candidate_paths: None,
            rust_inputs: Vec::new(),
            c_inputs: Vec::new(),
            dependencies_sha256: sha256("deps"),
            verification_sha256: sha256("verification"),
        };
        let context = OperationContext {
            run_id: sha256("failed-replay"),
            attempt: 1,
            policy_sha256: sha256("policy"),
            operation_key: sha256("key"),
            phase: translation_execution::model::ExecutionPhase::Candidate,
        };
        let runtime = StrictRuntime {
            helper: directory.0.join("absent-helper"),
            rustc: directory.0.join("absent-rustc"),
            c_compiler: directory.0.join("absent-cc"),
            operation_root: directory.0.join("operations"),
            pipeline_root: directory.0.join("pipeline"),
            workers: 4,
            gate_timeout_seconds: 1,
            deadline_unix_ms: Some(0),
            max_output_bytes: 1024 * 1024,
        };
        let intent = NativeIntent {
            context: context.clone(),
            prepared_sha256: sealed(&prepared)?,
            candidate_sha256: digest(&checkout.join("candidate.rs"))?,
            deadline_unix_ms: 0,
            outer_deadline_unix_ms: Some(0),
            max_output_bytes: runtime.max_output_bytes,
            operation_root: runtime.operation_root.clone(),
            parent_reservation_bytes: 4096,
            reconcile_producer_reservations: false,
            operation_metadata_bytes: None,
            helper: runtime.helper.clone(),
            rustc: runtime.rustc.clone(),
            c_compiler: runtime.c_compiler.clone(),
            compiler_runtime_sha256: sha256("none"),
        };
        atomic_toml(&output.join("native-intent.toml"), &intent)?;
        let verdict = Verdict {
            status: "rejected".into(),
            run: run_path.clone(),
            family: "failure".into(),
            error: Some("original bounded stop".into()),
            ..Default::default()
        };
        atomic_toml(&run_path.join("verdict.toml"), &verdict)?;
        let receipt = NativeReceipt {
            prepared_sha256: intent.prepared_sha256,
            candidate_sha256: intent.candidate_sha256,
            verdict: Asset { file: run_path.join("verdict.toml"), sha256: digest(&run_path.join("verdict.toml"))? },
            passed: false,
            diagnostic: "original bounded stop".into(),
            evidence_bytes: tree_bytes(&run_path)?,
            context: context.clone(),
            operation_root: runtime.operation_root.clone(),
            operations: Vec::new(),
            operation_index: None,
            max_output_bytes: runtime.max_output_bytes,
            reserved_bytes: 4096,
        };
        atomic_toml(&output.join("native-receipt.toml"), &receipt)?;
        let metadata = output.join("native-receipt.toml").metadata()?.modified()?;
        let replay = run(&prepared, &checkout, &output, &runtime, &context)?;
        assert!(!replay.passed);
        assert_eq!(replay.diagnostic, "original bounded stop");
        assert_eq!(output.join("native-receipt.toml").metadata()?.modified()?, metadata);
        assert!(!runtime.operation_root.exists() && !runtime.pipeline_root.exists());
        fs::write(checkout.join("candidate.rs"), b"pub fn Changed() {}")?;
        assert!(run(&prepared, &checkout, &output, &runtime, &context).is_err());
        Ok(())
    }

    fn current_checkout() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
    }

    fn compiled_sources() -> Vec<(&'static str, &'static str)> {
        vec![
            ("main.rs", include_str!("main.rs")),
            ("inputs.rs", include_str!("inputs.rs")),
            ("bounded_inputs.rs", include_str!("bounded_inputs.rs")),
            ("builds.rs", include_str!("builds.rs")),
            ("family.rs", include_str!("family.rs")),
            ("family_model.rs", include_str!("family_model.rs")),
            ("translation_native.rs", include_str!("translation_native.rs")),
            ("source_inventory.rs", include_str!("source_inventory.rs")),
            ("batch.rs", include_str!("batch.rs")),
            ("family_header.c", include_str!("family_header.c")),
            ("family_pointer.c", include_str!("family_pointer.c")),
            ("dispatcher_pointer.c", include_str!("dispatcher_pointer.c")),
            ("dispatcher_plm.c", include_str!("dispatcher_plm.c")),
            ("dispatcher_eproj.c", include_str!("dispatcher_eproj.c")),
            ("dispatcher_host.c", include_str!("dispatcher_host.c")),
            ("plm_fixtures.rs", include_str!("plm_fixtures.rs")),
            ("cleanup.rs", include_str!("cleanup.rs")),
            ("fixture_execution.rs", include_str!("fixture_execution.rs")),
            ("fixture_worker.rs", include_str!("fixture_worker.rs")),
            ("metrics.rs", include_str!("metrics.rs")),
            ("resource_sampler.rs", include_str!("resource_sampler.rs")),
        ]
    }

    fn variant_source(prepared: &PreparedNative, original: &Path, destination: &Path) -> Result<PathBuf> {
        fs::create_dir_all(destination)?;
        for name in prepared.contract.frozen_inputs.keys() {
            let path = destination.join(name);
            fs::create_dir_all(path.parent().unwrap())?;
            immutable(&path, &bounded::read(original.join(name))?)?;
        }
        immutable(&destination.join("tools/port/batch.toml"), &bounded::read(original.join("tools/port/batch.toml"))?)?;
        Ok(destination.to_path_buf())
    }

    #[test]
    #[ignore = "bounded real compiler/actual-C proof; requires installed confinement and private ROM"]
    fn full_real_family_retains_authenticates_and_reuses_every_producer() -> Result<()> {
        let _ = crate::BINARY_SHA256.set(crate::digest(&std::env::current_exe()?)?);
        let proof = PathBuf::from(std::env::var("SM_NATIVE_PROOF_ROOT")?);
        let root = proof.join(std::env::var("SM_NATIVE_PROOF_NAME").unwrap_or_else(|_| "actual-c".into()));
        fs::create_dir_all(&root)?;
        let current = current_checkout();
        let checkout = root.join("source");
        fs::create_dir_all(checkout.join("crates/pipeline/src"))?;
        fs::create_dir_all(checkout.join("crates/port/src"))?;
        fs::create_dir_all(checkout.join("tools/port"))?;
        let manifest_path = Path::new("tools/port/enemy-jump-instructions.toml");
        let manifest: Manifest = family::object(&current.join(manifest_path))?;
        immutable(&checkout.join(manifest_path), &bounded::read(current.join(manifest_path))?)?;
        for path in inputs::canonical_sources(&current, &[current.join(&manifest.rust_module)], &manifest.rust_imports)?
        {
            let relative = path.strip_prefix(&current)?;
            immutable(&checkout.join(relative), &bounded::read(&path)?)?;
        }
        for (name, text) in compiled_sources() {
            immutable(&checkout.join("crates/pipeline/src").join(name), text.as_bytes())?;
        }
        immutable(&checkout.join("tools/dependencies.toml"), &bounded::read(current.join("tools/dependencies.toml"))?)?;
        immutable(
            &checkout.join("tools/port/batch.toml"),
            b"[families]\nenemy-jump-instructions = \"family\"\n[profiles]\n",
        )?;
        let pinned = Path::new("/home/jim/sm-decomp/deps/snesrev-sm");
        let inventory_path = root.join("source-inventory.toml");
        if !inventory_path.exists() {
            crate::source_inventory::run(&checkout, pinned, &inventory_path)?;
        }
        let provenance: NativeProvenance = family::object(&proof.join("provenance/provenance.toml"))?;
        let limits = NativeLimits {
            timeout_seconds: 60,
            writable_bytes: 128 * 1024 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 256 * 1024,
            stdin_bytes: 0,
        };
        let fixture_limits = NativeLimits {
            timeout_seconds: 5,
            writable_bytes: 512 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 4096,
            stdin_bytes: 0,
        };
        let contract = contract_for_family(
            &checkout,
            manifest_path,
            Asset { file: inventory_path.clone(), sha256: digest(&inventory_path)? },
            provenance,
            limits,
            fixture_limits,
        )?;
        let prepared = prepare(&checkout, pinned, &contract)?;
        let mut missing = contract.clone();
        missing.coverage.get_mut("Sidehopper_Func_1").unwrap().pop();
        ensure!(prepare(&checkout, pinned, &missing).is_err(), "incomplete prescribed coverage admitted");
        let mut missing = contract.clone();
        missing.frozen_inputs.remove("crates/port/src/memory.rs");
        ensure!(prepare(&checkout, pinned, &missing).is_err(), "missing canonical harness dependency admitted");
        let mut missing = contract.clone();
        if let NativeProvenance::ReviewedOfflineFamily { verdict, .. } = &mut missing.provenance {
            verdict.sha256 = "0".repeat(64);
        }
        ensure!(prepare(&checkout, pinned, &missing).is_err(), "changed exercise provenance admitted");
        atomic_toml(&root.join("prepared.toml"), &prepared)?;
        let runtime = StrictRuntime {
            helper: PathBuf::from(std::env::var("SM_NATIVE_SANDBOX")?),
            rustc: PathBuf::from(std::env::var("SM_NATIVE_RUSTC")?),
            c_compiler: "/usr/bin/x86_64-linux-gnu-gcc-9".into(),
            operation_root: root.join("operations"),
            pipeline_root: "/home/jim/sm-decomp/out/pipeline".into(),
            workers: 4,
            gate_timeout_seconds: 180,
            deadline_unix_ms: None,
            max_output_bytes: 1024 * 1024 * 1024,
        };
        let context = OperationContext {
            run_id: sha256("native-adapter-real-enemy-jump-family"),
            attempt: 1,
            policy_sha256: prepared.verification_sha256.clone(),
            operation_key: sha256("native-real-family-first"),
            phase: translation_execution::model::ExecutionPhase::Candidate,
        };
        let started = std::time::Instant::now();
        let receipt = run(&prepared, &checkout, &root.join("run"), &runtime, &context)?;
        ensure!(receipt.passed, "{}", receipt.diagnostic);
        let verdict = authenticate(&prepared, &checkout, &receipt)?;
        ensure!(
            verdict.verified_cases == 16 && verdict.verified_routines == 2 && receipt.operation_count()? == 34,
            "real family coverage differs"
        );
        let mut before = Vec::new();
        let mut operations = Vec::new();
        receipt.visit_operations(|operation| {
            before.push((operation.result.path.clone(), fs::metadata(&operation.result.path)?.modified()?));
            operations.push(operation.clone());
            Ok(())
        })?;
        let retry = run(&prepared, &checkout, &root.join("run"), &runtime, &context)?;
        ensure!(sealed(&retry)? == sealed(&receipt)?, "lost response retry allocated another result");
        for (path, modified) in before {
            ensure!(fs::metadata(path)?.modified()? == modified, "retry rewrote a producer result");
        }
        // Hash-only family tampering cannot bypass the independent durable
        // producer authenticators, even if a forged receipt copies a new hash.
        let operation = &operations[0];
        let invocation_bytes = bounded::read(&operation.invocation.file)?;
        let original: builds::StrictInvocation = family::object(&operation.invocation.file)?;
        let mut changed = original.clone();
        changed.program.arguments.push("--undeclared".into());
        atomic_toml(&operation.invocation.file, &changed)?;
        let rejected = authenticate(&prepared, &checkout, &receipt).is_err();
        fs::write(&operation.invocation.file, &invocation_bytes)?;
        ensure!(rejected, "tampered producer request admitted");
        authenticate(&prepared, &checkout, &receipt)?;
        println!(
            "Real family proof: 16 cases, 2 routines, 34 durable producers, {:.3}s, {} evidence bytes, {} reserved bytes; identical retry reused all results",
            started.elapsed().as_secs_f64(),
            receipt.evidence_bytes,
            receipt.reserved_bytes
        );
        let compile_source = variant_source(&prepared, &checkout, &root.join("compile-denial-source"))?;
        let candidate = compile_source.join(&prepared.candidate_path);
        let candidate_bytes = bounded::read(&candidate)?;
        let undeclared = root.join("compiler-secret.txt");
        fs::write(&undeclared, "protected input")?;
        let compile_read = format!("\nconst UNDECLARED:&str=include_str!({:?});\n", undeclared);
        let mut mutated = candidate_bytes.clone();
        mutated.extend(compile_read.as_bytes());
        fs::write(&candidate, &mutated)?;
        let mut deny_context = context.clone();
        deny_context.operation_key = sha256("native-compile-time-denial");
        let denied = run(&prepared, &compile_source, &root.join("compile-denial"), &runtime, &deny_context);
        let retry = match &denied {
            Ok(denied) => {
                let retry = run(&prepared, &compile_source, &root.join("compile-denial"), &runtime, &deny_context)?;
                ensure!(sealed(&retry)? == sealed(denied)?, "unchanged rejected candidate reran compiler");
                Some(retry)
            }
            Err(_) => None,
        };
        let denied = denied?;
        ensure!(retry.is_some(), "failed native evaluation was not durably reused");
        ensure!(
            !denied.passed && denied.diagnostic.contains("compiler-secret"),
            "compile-time undeclared inclusion was not denied: {}",
            denied.diagnostic
        );
        authenticate(&prepared, &compile_source, &denied)?;

        let runtime_source = variant_source(&prepared, &checkout, &root.join("runtime-denial-source"))?;
        let candidate = runtime_source.join(&prepared.candidate_path);
        let runtime_secret = root.join("parent-verdict.toml");
        fs::write(&runtime_secret, "protected parent verdict")?;
        let candidate_text = String::from_utf8(candidate_bytes.clone())?;
        let signature = "pub fn dispatch_rom(r: &mut Wram<'_>, rom: &[u8], symbol: &str, a: i32, b: i32) -> Result<u32, &'static str> {";
        ensure!(candidate_text.matches(signature).count() == 1, "real candidate denial seam missing");
        let checks = format!(
            "{signature}\n assert!(std::fs::read({runtime_secret:?}).is_err());\n assert!(std::fs::write({runtime_secret:?},b\"forged\").is_err());\n assert!(std::fs::read(\"../Sidehopper_Func_1-0/c-after.wram\").is_err());\n assert!(std::fs::read(\"/program-c\").is_err());\n"
        );
        fs::write(&candidate, candidate_text.replacen(signature, &checks, 1))?;
        let mut deny_context = context.clone();
        deny_context.operation_key = sha256("native-runtime-protected-denial");
        let denied = run(&prepared, &runtime_source, &root.join("runtime-denial"), &runtime, &deny_context);
        let denied = denied?;
        ensure!(
            denied.passed && bounded::text(&runtime_secret)? == "protected parent verdict",
            "runtime denial escaped namespace or changed real family semantics: {}",
            denied.diagnostic
        );
        authenticate(&prepared, &runtime_source, &denied)?;
        let mut deny_context = context.clone();
        deny_context.operation_key = sha256("native-output-reservation-denial");
        let narrow = StrictRuntime { max_output_bytes: parent_reservation(&prepared, &checkout)? + 4096, ..runtime };
        let denied = run(&prepared, &checkout, &root.join("reservation-denial"), &narrow, &deny_context)?;
        ensure!(
            !denied.passed && denied.operation_count()? == 0 && denied.diagnostic.contains("reservation exhausted"),
            "aggregate reservation did not stop before compiler launch"
        );
        println!(
            "Preparation gaps, compiler undeclared inclusion, runtime parent/sibling read/write, tampered operation and aggregate prelaunch reservation checks passed"
        );
        Ok(())
    }

    /// Read-only authentication of completed producers needs neither their
    /// retired compiler helper nor today's source. Preserve the original source
    /// snapshots and every transitive durable reference through run cleanup.
    #[test]
    #[ignore = "requires retained real native proof; executes no compiler or fixture"]
    fn retained_real_family_authenticates_without_executing_producers() -> Result<()> {
        let proof = PathBuf::from(std::env::var("SM_NATIVE_PROOF_ROOT")?);
        let root = proof.join(std::env::var("SM_NATIVE_PROOF_NAME")?);
        let prepared: PreparedNative = family::object(&root.join("prepared.toml"))?;
        let mut retained = BTreeMap::new();
        for (output, source) in [
            ("run", "source"),
            ("compile-denial", "compile-denial-source"),
            ("runtime-denial", "runtime-denial-source"),
            ("reservation-denial", "source"),
        ] {
            let path = root.join(output).join("native-receipt.toml");
            if !path.is_file() {
                continue;
            }
            let checkout =
                std::env::var_os("SM_NATIVE_PROOF_SOURCE").map(PathBuf::from).unwrap_or_else(|| root.join(source));
            let receipt: NativeReceipt = family::object(&path)?;
            authenticate(&prepared, &checkout, &receipt)?;
            let intent: NativeIntent = family::object(&root.join(output).join("native-intent.toml"))?;
            let mut runtime = StrictRuntime {
                helper: root.join("retired-helper"),
                rustc: root.join("retired-rustc"),
                c_compiler: root.join("retired-cc"),
                operation_root: receipt.operation_root.clone(),
                pipeline_root: root.join("retired-pipeline"),
                workers: 4,
                gate_timeout_seconds: 180,
                deadline_unix_ms: intent.outer_deadline_unix_ms,
                max_output_bytes: receipt.max_output_bytes,
            };
            let retry = run(&prepared, &checkout, &root.join(output), &runtime, &receipt.context)?;
            ensure!(sealed(&retry)? == sealed(&receipt)?, "expired deadline changed a completed receipt");
            runtime.deadline_unix_ms = Some(intent.outer_deadline_unix_ms.unwrap_or(0).saturating_add(1));
            ensure!(
                run(&prepared, &checkout, &root.join(output), &runtime, &receipt.context).is_err(),
                "completed receipt admitted a changed outer deadline"
            );
            let mut operations = Vec::new();
            receipt.visit_operations(|operation| {
                operations.push((operation.result.path.clone(), fs::metadata(&operation.result.path)?.modified()?));
                Ok(())
            })?;
            let references = references(&receipt)?;
            for reference in &references {
                translation_execution::retention::authenticate_reference(reference, receipt.max_output_bytes)?;
            }
            authenticate(&prepared, &checkout, &receipt)?;
            for (path, modified) in operations {
                ensure!(fs::metadata(path)?.modified()? == modified, "authentication rewrote a producer result");
            }
            println!(
                "Authenticated retained {output}: {} producers, {} references",
                receipt.operation_count()?,
                references.len()
            );
            retained.insert(output, references);
        }
        ensure!(!retained.is_empty(), "native proof has no completed receipt");
        immutable(&root.join("retained-references.toml"), toml::to_string(&retained)?.as_bytes())?;
        Ok(())
    }

    #[test]
    #[ignore = "audits a retained finalization failure; never publishes native acceptance or launches a producer"]
    fn retained_complete_family_finalization_authenticates_without_acceptance() -> Result<()> {
        let proof = PathBuf::from(std::env::var("SM_NATIVE_PROOF_ROOT")?);
        let root = proof.join(std::env::var("SM_NATIVE_PROOF_NAME")?);
        let checkout = PathBuf::from(std::env::var("SM_NATIVE_PROOF_SOURCE")?);
        let prepared: PreparedNative = family::object(&root.join("prepared.toml"))?;
        let output = root.join("run");
        ensure!(!output.join("native-receipt.toml").exists(), "audit requires the retained finalization failure");
        let intent: NativeIntent = family::object(&output.join("native-intent.toml"))?;
        ensure!(now_unix_ms()? >= intent.deadline_unix_ms, "original failed family deadline is not expired");
        let run = output.join("family");
        let (pairs, producers, bytes) = bounded::reconcile(8 * bounded::DEFAULT_BYTES, || {
            check_candidate(&prepared, &checkout)?;
            ensure!(candidate_digest(&prepared, &checkout)? == intent.candidate_sha256, "retained candidate differs");
            let verdict: Verdict = family::object(&run.join("verdict.toml"))?;
            let mut inventory = BTreeMap::new();
            bounded::discovery(|| family::inventory(&run, &run, &mut inventory))?;
            ensure!(inventory == verdict.artifacts, "retained artifact inventory differs");
            let operations = collect_operations(&prepared, &run, &intent.operation_root, &intent.context)?;
            ensure!(operations.len() == operation_slots(&prepared, &run)?.len(), "retained producer is missing");
            let identities = operations
                .iter()
                .map(|operation| Ok((operation.result.path.clone(), fs::metadata(&operation.result.path)?.modified()?)))
                .collect::<Result<Vec<_>>>()?;
            let bytes = evidence_bytes(&run, &operations)?;
            ensure!(evidence_bytes(&run, &operations)? == bytes, "retained evidence extent changed");
            let checked = bounded::discovery(|| {
                family::authenticate_with_source(
                    crate::family_model::AuthenticateRequest { checkout, verdict: run.join("verdict.toml") },
                    &prepared.pinned_source,
                )
            })?;
            ensure!(
                checked.verified_cases == prepared.contract.coverage.values().map(Vec::len).sum::<usize>(),
                "retained complete family extent differs"
            );
            for (path, modified) in identities {
                ensure!(fs::metadata(path)?.modified()? == modified, "audit changed a producer result");
            }
            Ok((checked.verified_cases, operations.len(), bytes))
        })?;
        ensure!(!output.join("native-receipt.toml").exists(), "audit published native acceptance");
        let report = BTreeMap::from([
            ("verified_family_pairs", pairs as u64),
            ("authenticated_producers", producers as u64),
            ("retained_evidence_bytes", bytes),
            ("native_acceptance_published", 0),
        ]);
        immutable(&root.join("retained-finalization-audit.toml"), toml::to_string(&report)?.as_bytes())?;
        println!(
            "Authenticated {pairs} retained Fixture pairs and {producers} producers; no native acceptance published"
        );
        Ok(())
    }

    /// A real retained proof is never authenticated against today's edited
    /// harness. Preserve the exact small sources already sealed by the verdict.
    #[test]
    #[ignore = "requires the retained actual-C run and private pinned source"]
    fn historical_provenance_authenticates_retained_source() -> Result<()> {
        let proof = PathBuf::from(std::env::var("SM_NATIVE_PROOF_ROOT")?);
        let current = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf();
        let verdict_path = PathBuf::from(
            "/home/jim/sm-decomp/out/port/decomp-run-20261013/host-integrated/families/enemy-jump-instructions/evidence/1791902651665897-546014-290/verdict.toml",
        );
        let prior: Verdict = family::object(&verdict_path)?;
        let implementation = prior.implementation.as_ref().context("prior implementation missing")?;
        let snapshot = proof.join("provenance/source");
        fs::create_dir_all(&snapshot)?;
        let mut retained = BTreeMap::new();
        for entry in fs::read_dir(&prior.run)? {
            let path = entry?.path();
            if path.is_file() && !path.is_symlink() {
                retained.insert(digest(&path)?, path);
            }
        }
        for map in [&implementation.rust_dependencies, &implementation.harness_sources] {
            for (relative_name, hash) in map {
                ensure!(relative(Path::new(relative_name)), "historical source ownership is not relative");
                let source = retained
                    .get(hash)
                    .with_context(|| format!("sealed historical source unavailable: {relative_name}"))?;
                let destination = snapshot.join(relative_name);
                fs::create_dir_all(destination.parent().unwrap())?;
                immutable(&destination, &bounded::read(source)?)?;
                ensure!(digest(&destination)? == *hash, "historical source snapshot differs");
            }
        }
        let declared = implementation.manifest.as_ref().context("historical manifest missing")?;
        let relative_manifest = declared.file.strip_prefix(&implementation.source_root)?;
        let destination = snapshot.join(relative_manifest);
        fs::create_dir_all(destination.parent().unwrap())?;
        immutable(&destination, &bounded::read(prior.run.join("manifest.toml"))?)?;
        ensure!(digest(&destination)? == declared.sha256, "historical manifest snapshot differs");
        fs::create_dir_all(snapshot.join("tools"))?;
        immutable(&snapshot.join("tools/dependencies.toml"), &bounded::read(current.join("tools/dependencies.toml"))?)?;
        let authenticated = family::authenticate_with_source(
            crate::family_model::AuthenticateRequest { checkout: snapshot.clone(), verdict: verdict_path.clone() },
            Path::new("/home/jim/sm-decomp/deps/snesrev-sm"),
        )?;
        ensure!(authenticated == prior, "historical authentication changed verdict");
        let provenance = NativeProvenance::ReviewedOfflineFamily {
            verdict: Asset { file: verdict_path, sha256: digest(&prior.run.join("verdict.toml"))? },
            checkout: snapshot,
        };
        let asset_path = proof.join("provenance/provenance.toml");
        if asset_path.exists() {
            ensure!(
                sealed(&family::object::<NativeProvenance>(&asset_path)?)? == sealed(&provenance)?,
                "historical provenance changed"
            );
        } else {
            atomic_toml(&asset_path, &provenance)?;
        }
        println!(
            "Authenticated historical provenance: {} {} (16 cases, 2 routines)",
            asset_path.display(),
            digest(&asset_path)?
        );
        Ok(())
    }
}
