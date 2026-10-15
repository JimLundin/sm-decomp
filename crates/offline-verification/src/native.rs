//! Installed actual-C execution for complete frozen, registered Offline families.
//! Candidate compilation and each fixture invocation are separate confined
//! producers. Only the trusted parent writes the family verdict and inventory.
use crate::{
    HostInputs, OfflineFamilies, builds, catalog as batch, family,
    family_model::{Asset, Manifest, Source, Verdict, VerifyRequest},
    inputs, recipe as recipes,
};
use anyhow::{Context, Result, ensure};
use decomp_source::context as source_context;
use recipes::Recipe as NativeRecipe;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tool_execution::{
    atomic_toml,
    bounded::{self, digest},
};
use translation_execution::{
    OperationContext,
    confinement::RunLimits,
    model::{sealed, sha256},
    retention::Reference,
};

#[path = "native_operations.rs"]
mod operation_storage;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeLimits {
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
pub enum NativeProvenance {
    NativeScope { scope: Asset, checkout: PathBuf },
    ReviewedOfflineFamily { verdict: Asset, checkout: PathBuf },
    ReviewedCatalog { catalog: Asset, requirements: Asset, checkout: PathBuf, rationale: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeContract {
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
    pub(crate) recipe: Option<NativeRecipe>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedNative {
    contract: NativeContract,
    /// Reopened from the contract's authenticated binary input for each operation.
    /// Persisting this cache would expand shared fixture payloads back into text.
    #[serde(skip)]
    manifest: Manifest,
    source: Source,
    pinned_source: PathBuf,
    candidate_path: String,
    /// Writable members of this family's Rust closure approved by the whole
    /// translation scope, including the root when it is writable.
    approved_candidate_paths: BTreeSet<String>,
    rust_inputs: Vec<String>,
    c_inputs: Vec<String>,
    dependencies_sha256: String,
    verification_sha256: String,
}

/// Existing registered declarations available to the trusted application.
pub enum NativeDeclarationKind<'a> {
    Family(&'a Path),
    DirectAnimation(&'a Path),
}

/// Freeze a complete declaration; a candidate cannot select a recipe or argv.
pub struct NativeDeclaration<'a> {
    pub checkout: &'a Path,
    pub kind: NativeDeclarationKind<'a>,
    pub source_inventory: Asset,
    pub provenance: NativeProvenance,
    pub compiler_limits: NativeLimits,
    pub fixture_limits: NativeLimits,
    pub deadline_unix_ms: Option<u64>,
    pub max_input_bytes: u64,
}

/// Source preparation has no execution runtime and performs no early IO.
#[derive(Clone, Copy)]
pub struct NativePreparation<'a> {
    pub checkout: &'a Path,
    pub pinned_source: &'a Path,
    pub contract: &'a NativeContract,
    pub approved_candidate_paths: Option<&'a [String]>,
    pub deadline_unix_ms: Option<u64>,
    pub max_input_bytes: u64,
}

impl<'a> NativePreparation<'a> {
    pub fn new(checkout: &'a Path, pinned_source: &'a Path, contract: &'a NativeContract) -> Self {
        Self {
            checkout,
            pinned_source,
            contract,
            approved_candidate_paths: None,
            deadline_unix_ms: None,
            max_input_bytes: bounded::DEFAULT_BYTES,
        }
    }
}

/// Facts returned only by the application's existing scope approval adapter.
pub struct AdmittedScope {
    pub checkout: PathBuf,
    pub scope: Asset,
    pub routines: BTreeSet<String>,
    pub writable_paths: BTreeSet<String>,
    pub inputs: BTreeMap<String, String>,
}

pub trait ScopeAdmission {
    fn admit(&self, checkout: &Path, scope: &Asset) -> Result<AdmittedScope>;
}

/// Reviewed catalog/Offline provenance needs no Translation assignment policy.
pub struct NoScopeAdmission;

impl ScopeAdmission for NoScopeAdmission {
    fn admit(&self, _: &Path, _: &Asset) -> Result<AdmittedScope> {
        anyhow::bail!("native scope provenance requires the application's scope admission adapter")
    }
}

pub struct NativeTarget<'a> {
    pub checkout: &'a Path,
    pub output: &'a Path,
    pub context: &'a OperationContext,
    pub fixture_jobs: Option<usize>,
}

pub struct NativeEvidence<'a> {
    pub checkout: &'a Path,
    pub receipt: &'a NativeReceipt,
}

/// Admission authenticates the outcome, including a durably rejected run.
pub struct AuthenticatedNative {
    report: Verdict,
    references: Vec<Reference>,
}

impl AuthenticatedNative {
    pub fn report(&self) -> &Verdict {
        &self.report
    }

    pub fn references(&self) -> &[Reference] {
        &self.references
    }
}

impl OfflineFamilies {
    pub fn declare_native(&self, request: NativeDeclaration<'_>) -> Result<NativeContract> {
        let action = || match request.kind {
            NativeDeclarationKind::Family(manifest) => contract_for_family_bounded(
                self.host_inputs(),
                request.checkout,
                manifest,
                request.source_inventory,
                request.provenance,
                request.compiler_limits,
                request.fixture_limits,
            ),
            NativeDeclarationKind::DirectAnimation(plan) => contract_for_direct_animation_bounded(
                self.host_inputs(),
                request.checkout,
                plan,
                request.source_inventory,
                request.provenance,
                request.compiler_limits,
                request.fixture_limits,
            ),
        };
        match request.deadline_unix_ms {
            Some(deadline) => source_context::until(deadline, request.max_input_bytes, action),
            None => source_context::limited(action),
        }
    }

    pub fn prepare_native(
        &self,
        request: NativePreparation<'_>,
        scope_admission: &dyn ScopeAdmission,
    ) -> Result<PreparedNative> {
        prepare_request(self.host_inputs(), &request, scope_admission, None)
    }

    /// Reuse the existing authenticated inventory within one bounded catalog
    /// preparation, without exposing its mutable implementation context.
    /// Preparation stays lazy so a caller can retain each complete result and
    /// stop at its first rejected family without inspecting later candidates.
    pub fn prepare_native_catalog<'a>(
        &'a self,
        requests: &'a [NativePreparation<'a>],
        scope_admission: &'a dyn ScopeAdmission,
    ) -> Result<impl Iterator<Item = Result<PreparedNative>> + 'a> {
        let catalog = requests
            .first()
            .map(|first| match first.deadline_unix_ms {
                Some(deadline) => source_context::until(deadline, first.max_input_bytes, || {
                    catalog_inputs(first.checkout, first.pinned_source, &first.contract.source_inventory)
                }),
                None => source_context::limited(|| {
                    catalog_inputs(first.checkout, first.pinned_source, &first.contract.source_inventory)
                }),
            })
            .transpose()?;
        Ok(requests.iter().map(move |request| {
            ensure!(
                matches!(request.contract.provenance, NativeProvenance::ReviewedCatalog { .. }),
                "catalog input context requires reviewed catalog provenance"
            );
            prepare_request(self.host_inputs(), request, scope_admission, catalog.as_ref())
        }))
    }

    pub fn prepare_generated_plm(&self, checkout: &Path, output: &Path) -> Result<crate::family_model::PlmGeneration> {
        recipes::prepare_generated_plm(checkout, output)
    }
}

/// Trusted one-catalog preparation context. Construction authenticates actual
/// source and the complete inventory once. It is not a deserializable artifact;
/// every produced PreparedNative still seals that exact Source and inventory.
struct NativeCatalogInputs {
    checkout: PathBuf,
    pinned_source: PathBuf,
    inventory_path: PathBuf,
    inventory_sha256: String,
    source: Source,
    symbols: BTreeSet<String>,
}

fn catalog_inputs(checkout: &Path, pinned_source: &Path, inventory: &Asset) -> Result<NativeCatalogInputs> {
    bounded::safe(checkout)?;
    bounded::safe(pinned_source)?;
    let checkout = fs::canonicalize(checkout)?;
    let pinned_source = fs::canonicalize(pinned_source)?;
    let inventory_path = check_asset(&checkout, inventory)?;
    let source = decomp_source::authenticate_inventory(&checkout, &pinned_source, &inventory_path)?;
    let declared: toml::Value = bounded::object(&inventory_path)?;
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

/// Installed paths/resources provided by the trusted supervisor configuration.
pub struct ConfinedEnvironment {
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
pub struct NativeReceipt {
    pub prepared_sha256: String,
    pub candidate_sha256: String,
    pub verdict: Asset,
    pub passed: bool,
    pub diagnostic: String,
    pub evidence_bytes: u64,
    pub context: OperationContext,
    pub operation_root: PathBuf,
    pub(crate) operation_index: Reference,
    pub max_output_bytes: u64,
    pub reserved_bytes: u64,
}

impl NativeReceipt {
    pub(crate) fn visit_operations(&self, visit: impl FnMut(&NativeOperation) -> Result<()>) -> Result<()> {
        operation_storage::visit(self, visit).map(|_| ())
    }

    /// Read the sealed attribution count without exposing operation records.
    pub fn operation_count(&self) -> Result<usize> {
        let mut count = 0;
        self.visit_operations(|_| {
            count += 1;
            Ok(())
        })?;
        Ok(count)
    }

    /// Validate and collect the retained reference bundle. This is retention
    /// data; only PreparedNative::authenticate admits the complete outcome.
    pub fn retained_references(&self) -> Result<Vec<Reference>> {
        references(self)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeIntent {
    context: OperationContext,
    prepared_sha256: String,
    candidate_sha256: String,
    deadline_unix_ms: u64,
    outer_deadline_unix_ms: Option<u64>,
    max_output_bytes: u64,
    operation_root: PathBuf,
    parent_reservation_bytes: u64,
    operation_metadata_bytes: u64,
    fixture_jobs: usize,
    helper: PathBuf,
    rustc: PathBuf,
    c_compiler: PathBuf,
    compiler_runtime_sha256: String,
}

fn now_unix_ms() -> Result<u64> {
    Ok(u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?)
}

fn new_deadline(entered: u64, runtime: &ConfinedEnvironment) -> Result<u64> {
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
    tool_execution::bounded::immutable_bytes(path, bytes)
}
fn asset_path(checkout: &Path, asset: &Asset) -> Result<PathBuf> {
    let path = if asset.file.is_absolute() {
        asset.file.clone()
    } else {
        ensure!(relative(&asset.file), "native asset relative path alias");
        checkout.join(&asset.file)
    };
    file(&path)
}
fn check_asset(checkout: &Path, asset: &Asset) -> Result<PathBuf> {
    let path = asset_path(checkout, asset)?;
    ensure!(digest(&path)? == asset.sha256, "native asset changed: {}", path.display());
    Ok(path)
}
fn read_fixture(checkout: &Path, asset: &Asset) -> Result<Manifest> {
    crate::fixture_format::read_authenticated(&asset_path(checkout, asset)?, &asset.sha256)
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

fn contract_for_family_bounded(
    host: &HostInputs,
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
    let declared = crate::fixture_format::read(&path)?;
    let recipe = recipes::for_family(&checkout, manifest, &declared)?;
    contract_for_declared(
        host,
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
fn contract_for_direct_animation_bounded(
    host: &HostInputs,
    checkout: &Path,
    plan: &Path,
    source_inventory: Asset,
    provenance: NativeProvenance,
    compiler_limits: NativeLimits,
    fixture_limits: NativeLimits,
) -> Result<NativeContract> {
    ensure!(relative(plan), "direct animation plan must be a frozen relative input");
    let recipe = NativeRecipe::DirectAnimation {
        plan: Asset { file: plan.into(), sha256: digest(&file(&checkout.join(plan))?)? },
    };
    let manifest = Path::new("tools/port/pause-equipment-render.smf");
    let declared = crate::fixture_format::read(&file(&checkout.join(manifest))?)?;
    contract_for_declared(
        host,
        checkout,
        manifest,
        source_inventory,
        provenance,
        compiler_limits,
        fixture_limits,
        Some(recipe),
        declared,
    )
}

#[allow(clippy::too_many_arguments)]
fn contract_for_declared(
    host: &HostInputs,
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
    for (name, compiled) in inputs::required_sources(host) {
        let actual = digest(&file(&checkout.join(name))?)?;
        ensure!(actual == sha256(compiled.as_bytes()), "native executable source is stale: {name}");
        frozen_inputs.insert(name.into(), actual);
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
    scope_admission: &dyn ScopeAdmission,
    approved_paths: Option<&[String]>,
) -> Result<()> {
    match &contract.provenance {
        NativeProvenance::ReviewedCatalog { catalog, requirements, checkout: owner, rationale } => {
            ensure!(!rationale.trim().is_empty() && rationale.len() <= 4096, "catalog review rationale missing");
            check_asset(owner, requirements)?;
            let reviewed = check_asset(owner, catalog)?;
            let inventory: batch::Catalog = bounded::object(&reviewed)?;
            ensure!(
                sealed(&batch::entry(&inventory, &registration.catalog_family)?)? == sealed(&registration.entry)?,
                "reviewed catalog entry differs"
            );
        }
        NativeProvenance::NativeScope { scope, checkout: owner } => {
            check_asset(owner, scope)?;
            let admitted = scope_admission.admit(owner, scope)?;
            ensure!(
                admitted.checkout == fs::canonicalize(owner)? && admitted.scope == *scope,
                "native scope admission identity differs"
            );
            ensure!(
                manifest.routines.iter().all(|routine| admitted.routines.contains(&routine.symbol)),
                "native scope coverage disagrees with family"
            );
            let default_candidate = [manifest.rust_module.clone()];
            ensure!(
                approved_paths.unwrap_or(&default_candidate).iter().all(|path| admitted.writable_paths.contains(path)),
                "native candidate ownership disagrees with admitted scope"
            );
            ensure!(
                contract
                    .frozen_inputs
                    .iter()
                    .all(|(name, expected)| { admitted.inputs.get(name).is_none_or(|actual| actual == expected) }),
                "native frozen inputs disagree with admitted scope"
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
            let prior_manifest = crate::fixture_format::read(&prior.run.join("manifest.smf"))?;
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

fn prepare_request(
    host: &HostInputs,
    request: &NativePreparation<'_>,
    scope_admission: &dyn ScopeAdmission,
    catalog: Option<&NativeCatalogInputs>,
) -> Result<PreparedNative> {
    let action = || {
        let mut prepared = prepare_bounded_with_catalog(host, request, scope_admission, catalog)?;
        if let Some(approved_paths) = request.approved_candidate_paths {
            bind_scope_candidates(&mut prepared, approved_paths)?;
        }
        Ok(prepared)
    };
    match request.deadline_unix_ms {
        Some(deadline) => source_context::until(deadline, request.max_input_bytes, action),
        None => source_context::limited(action),
    }
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
    prepared.approved_candidate_paths = candidates;
    prepared.dependencies_sha256 =
        sealed(&("scope-native-inputs-v1", &prepared.dependencies_sha256, &prepared.approved_candidate_paths))?;
    Ok(())
}

fn prepare_bounded_with_catalog(
    host: &HostInputs,
    request: &NativePreparation<'_>,
    scope_admission: &dyn ScopeAdmission,
    catalog: Option<&NativeCatalogInputs>,
) -> Result<PreparedNative> {
    let NativePreparation { checkout, pinned_source, contract, .. } = *request;
    bounded::safe(checkout)?;
    bounded::safe(pinned_source)?;
    let checkout = fs::canonicalize(checkout)?;
    let pinned_source = fs::canonicalize(pinned_source)?;
    ensure!(relative(&contract.manifest.file), "native manifest is not registered relative input");
    let original = read_fixture(&checkout, &contract.manifest)?;
    let registration =
        recipes::registration(&checkout, &contract.manifest, &contract.family, contract.recipe.as_ref(), &original)?;
    let manifest = recipes::normalize_manifest(&checkout, original, contract.recipe.as_ref())?;
    ensure!(
        contract.frozen_inputs.len() <= recipes::input_inventory_limit(contract.recipe.as_ref())
            && contract.coverage.len() <= 1024,
        "native frozen input or coverage inventory exceeded"
    );
    bounded::count(contract.frozen_inputs.len())?;
    if contract.recipe.is_none() {
        recipes::plain(&manifest)?;
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
    provenance(
        &checkout,
        &pinned_source,
        contract,
        &manifest,
        &source,
        &registration,
        scope_admission,
        request.approved_candidate_paths,
    )?;
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
    for (name, compiled) in inputs::required_sources(host) {
        ensure!(
            contract.frozen_inputs.get(name) == Some(&sha256(compiled.as_bytes())),
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
        c.extend(decomp_source::c_units(&pinned_source)?);
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
    let verification_sha256 = sealed(&(contract, crate::fixture_format::identity(&manifest)?))?;
    Ok(PreparedNative {
        contract: contract.clone(),
        manifest,
        source,
        pinned_source,
        approved_candidate_paths: BTreeSet::from([candidate_path.clone()]),
        candidate_path,
        rust_inputs: rust_names,
        c_inputs: c_names,
        dependencies_sha256,
        verification_sha256,
    })
}

impl PreparedNative {
    pub fn contract(&self) -> &NativeContract {
        &self.contract
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    pub fn pinned_source(&self) -> &Path {
        &self.pinned_source
    }

    pub fn candidate_path(&self) -> &str {
        &self.candidate_path
    }

    pub fn dependencies_sha256(&self) -> &str {
        &self.dependencies_sha256
    }

    pub fn verification_sha256(&self) -> &str {
        &self.verification_sha256
    }

    pub fn covers_candidate(&self, path: &str) -> bool {
        self.rust_inputs.iter().any(|input| input == path)
    }

    /// Conservative capacity used by composite Original acceptance. Keep the
    /// existing source, compiler, fixture and runtime-metadata allowances in
    /// this owner instead of reconstructing reservations in callers.
    pub fn required_output_bytes(&self) -> Result<u64> {
        const SOURCE_ALLOWANCE: u64 = 512 * 1024 * 1024;
        const RECORD_ALLOWANCE: u64 = 4 * 1024 * 1024;
        let cases = self.contract.coverage.values().map(Vec::len).sum::<usize>() as u64;
        let compiler = self
            .contract
            .compiler_limits
            .writable_bytes
            .checked_mul(6)
            .and_then(|bytes| bytes.checked_add(16 * RECORD_ALLOWANCE))
            .context("native compiler capacity overflow")?;
        let fixture = self
            .contract
            .fixture_limits
            .writable_bytes
            .checked_mul(6)
            .and_then(|bytes| bytes.checked_add(2 * RECORD_ALLOWANCE))
            .context("native fixture capacity overflow")?;
        (2 * SOURCE_ALLOWANCE)
            .checked_add(compiler)
            .and_then(|bytes| fixture.checked_mul(cases + 1).and_then(|fixtures| bytes.checked_add(fixtures)))
            .context("native family capacity overflow")
    }

    pub fn declaration(&self, checkout: &Path) -> Result<Manifest> {
        let original = read_fixture(checkout, &self.contract.manifest)?;
        let manifest = recipes::normalize_manifest(checkout, original, self.contract.recipe.as_ref())?;
        ensure!(
            self.verification_sha256 == sealed(&(&self.contract, crate::fixture_format::identity(&manifest)?))?,
            "prepared binary fixture identity differs"
        );
        Ok(manifest)
    }

    /// Run the complete prescribed family, or authenticate its unchanged
    /// retained result on retry before touching any compiler/runtime path.
    pub fn verify(
        &self,
        target: NativeTarget<'_>,
        environment: &ConfinedEnvironment,
        host: &HostInputs,
    ) -> Result<NativeReceipt> {
        inputs::with_host(host, || {
            run_bounded(
                self,
                target.checkout,
                target.output,
                environment,
                target.context,
                target.fixture_jobs.unwrap_or(environment.workers),
            )
        })
    }

    pub fn authenticate(&self, evidence: NativeEvidence<'_>) -> Result<AuthenticatedNative> {
        let report = authenticate(self, evidence.checkout, evidence.receipt)?;
        let references = references(evidence.receipt)?;
        Ok(AuthenticatedNative { report, references })
    }
}

fn open_prepared(prepared: &PreparedNative, checkout: &Path) -> Result<PreparedNative> {
    let manifest = prepared.declaration(checkout)?;
    let mut opened = prepared.clone();
    opened.manifest = manifest;
    Ok(opened)
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
    prepared.approved_candidate_paths.contains(path)
}

fn candidate_digest(prepared: &PreparedNative, checkout: &Path) -> Result<String> {
    let mut inputs = BTreeMap::new();
    for path in &prepared.approved_candidate_paths {
        bounded::check()?;
        inputs.insert(path, digest(&file(&checkout.join(path))?)?);
    }
    sealed(&inputs)
}

#[cfg(test)]
fn run(
    prepared: &PreparedNative,
    checkout: &Path,
    output: &Path,
    runtime: &ConfinedEnvironment,
    context: &OperationContext,
) -> Result<NativeReceipt> {
    run_bounded(prepared, checkout, output, runtime, context, runtime.workers)
}

fn run_bounded(
    prepared: &PreparedNative,
    checkout: &Path,
    output: &Path,
    runtime: &ConfinedEnvironment,
    context: &OperationContext,
    fixture_jobs: usize,
) -> Result<NativeReceipt> {
    let entered = now_unix_ms()?;
    let intent_exists = output.join("native-intent.toml").is_file();
    if !intent_exists {
        ensure!(
            (1..=64).contains(&fixture_jobs) && fixture_jobs <= runtime.workers,
            "native fixture worker count exceeds shared capacity"
        );
        ensure!(
            entered < new_deadline(entered, runtime)?,
            "native whole-stage time budget exhausted before preparation"
        );
    }
    let receipt_path = output.join("native-receipt.toml");
    if receipt_path.is_file() {
        return source_context::reconcile(8 * bounded::DEFAULT_BYTES, || {
            let checkout = fs::canonicalize(checkout)?;
            let receipt: NativeReceipt = bounded::object(&receipt_path)?;
            ensure!(
                receipt.context == *context
                    && receipt.operation_root == runtime.operation_root
                    && receipt.max_output_bytes == runtime.max_output_bytes,
                "native gate receipt retried with another installed context"
            );
            let intent: NativeIntent = bounded::object(&output.join("native-intent.toml"))?;
            ensure!(
                intent.outer_deadline_unix_ms == runtime.deadline_unix_ms,
                "native whole-stage deadline changed on retry"
            );
            authenticate(prepared, &checkout, &receipt)?;
            Ok(receipt)
        });
    }
    let saved_intent =
        if intent_exists { Some(bounded::object::<NativeIntent>(&output.join("native-intent.toml"))?) } else { None };
    let deadline = saved_intent.as_ref().map_or(new_deadline(entered, runtime)?, |intent| intent.deadline_unix_ms);
    let opened = execute_or_reconcile(intent_exists, deadline, || open_prepared(prepared, checkout))?;
    let prepared = &opened;
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
            translation_execution::compiler_runtime::discover(&rustc)?
        } else {
            translation_execution::compiler_runtime::discover_until(&rustc, deadline, 8 * bounded::DEFAULT_BYTES)?
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
                operation_metadata_bytes: operation_storage::capacity(operation_slots(prepared, &run)?.len())?,
                fixture_jobs,
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
            reserved_bytes: Mutex::new(intent.parent_reservation_bytes),
            reservation_group: Mutex::new(None),
        };
        Ok((checkout, output, run, intent, execution))
    };
    let (checkout, output, run, intent, execution) = if intent_exists && entered >= deadline {
        source_context::reconcile(8 * bounded::DEFAULT_BYTES, setup)?
    } else {
        source_context::until(deadline, 8 * bounded::DEFAULT_BYTES, setup)?
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
        case_jobs: intent.fixture_jobs,
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
        let manifest_bytes = recipes::manifest_bytes(
            prepared.contract.recipe.as_ref(),
            &prepared.manifest,
            &prepared.contract.manifest,
            &checkout,
        )?;
        immutable(&run.join("manifest.smf"), &manifest_bytes)?;
        immutable(&run.join("LICENSE.txt"), &bounded::read(prepared.pinned_source.join("LICENSE.txt"))?)?;
        if let Some(rom) = &prepared.manifest.rom {
            let bytes = bounded::read(check_asset(&checkout, rom)?)?;
            ensure!(!bytes.is_empty() && bytes.len() <= 0x400000, "native ROM extent differs");
            immutable(&run.join("rom.bin"), &bytes)?;
            verdict.rom = Some(crate::family_model::RomIdentity { sha256: sha256(&bytes), bytes: bytes.len() });
        }
        let mut implementation = recipes::build(
            prepared.contract.recipe.as_ref(),
            &prepared.manifest,
            &request,
            &checkout,
            &prepared.pinned_source,
            &run,
            Some(&execution),
        )?;
        implementation.manifest = Some(Asset {
            file: if recipes::generated(prepared.contract.recipe.as_ref()) {
                run.join("manifest.smf")
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
        let tasks = family::ordered_tasks(&prepared.manifest)?;
        if intent.fixture_jobs == 1 {
            // A single worker executes directly without per-fixture thread startup.
            for (routine, fixture, index) in tasks {
                verdict.cases.push(recipes::fixture(
                    prepared.contract.recipe.as_ref(),
                    &prepared.manifest,
                    routine,
                    fixture,
                    index,
                    &run,
                    recipes::FixtureExecution::Confined(&execution),
                )?);
            }
        } else {
            let outcomes = crate::fixture_execution::run_waves(
                &tasks,
                intent.fixture_jobs,
                || execution.begin_fixture_group(),
                || execution.finish_fixture_group(),
                |(routine, fixture, index)| {
                    recipes::fixture(
                        prepared.contract.recipe.as_ref(),
                        &prepared.manifest,
                        routine,
                        fixture,
                        index,
                        &run,
                        recipes::FixtureExecution::Confined(&execution),
                    )
                },
            )?;
            let mut failure = None;
            for (_, outcome) in outcomes {
                match outcome {
                    Ok(case) => verdict.cases.push(case),
                    Err(error) => {
                        if failure.is_none()
                            || error.downcast_ref::<translation_execution::ExecutionPending>().is_some()
                        {
                            failure = Some(error);
                        }
                    }
                }
            }
            if let Some(error) = failure {
                return Err(error);
            }
        }
        recipes::verdict_fields(prepared.contract.recipe.as_ref(), &prepared.manifest, &mut verdict)?;
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
    source_context::reconcile(8 * bounded::DEFAULT_BYTES, || {
        family::inventory_with_limits(&run, &run, &mut verdict.artifacts, retained_tree_limits(&run)?)?;
        atomic_toml(&run.join("verdict.toml"), &verdict)?;
        let operations = collect_operations(prepared, &run, &intent)?;
        let operation_index =
            operation_storage::store(&output, &prepared_sha256, &candidate_sha256, context, &operations)?;
        let mut receipt = NativeReceipt {
            prepared_sha256,
            candidate_sha256,
            verdict: Asset { file: run.join("verdict.toml"), sha256: digest(&run.join("verdict.toml"))? },
            passed: verdict.status == "offline-verified",
            diagnostic: verdict.error.clone().unwrap_or_default(),
            evidence_bytes: 0,
            context: context.clone(),
            operation_root: runtime.operation_root.clone(),
            operation_index,
            max_output_bytes: intent.max_output_bytes,
            reserved_bytes: *execution.reserved_bytes.lock().unwrap(),
        };
        receipt.evidence_bytes = evidence_bytes(&run, &receipt)?;
        ensure!(
            receipt.evidence_bytes <= intent.max_output_bytes,
            "native evidence exceeded prelaunch aggregate reservation"
        );
        ensure!(receipt.evidence_bytes <= receipt.reserved_bytes, "native evidence exceeded reconciled reservation");
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
        source_context::reconcile(8 * bounded::DEFAULT_BYTES, action)
    } else {
        source_context::until(deadline, 8 * bounded::DEFAULT_BYTES, action)
    }
}

pub(crate) fn authenticate(prepared: &PreparedNative, checkout: &Path, receipt: &NativeReceipt) -> Result<Verdict> {
    source_context::until(now_unix_ms()?.saturating_add(60_000), 8 * bounded::DEFAULT_BYTES, || {
        authenticate_bounded(prepared, checkout, receipt)
    })
}

fn authenticate_bounded(prepared: &PreparedNative, checkout: &Path, receipt: &NativeReceipt) -> Result<Verdict> {
    bounded::safe(checkout)?;
    let checkout = fs::canonicalize(checkout)?;
    let opened = open_prepared(prepared, &checkout)?;
    let prepared = &opened;
    ensure!(receipt.prepared_sha256 == sealed(prepared)?, "native prepared identity differs");
    check_candidate(prepared, &checkout)?;
    ensure!(receipt.candidate_sha256 == candidate_digest(prepared, &checkout)?, "native candidate changed");
    let path = check_asset(&checkout, &receipt.verdict)?;
    let run = path.parent().context("native verdict has no run parent")?;
    let output = run.parent().context("native run has no output parent")?;
    let intent: NativeIntent = bounded::object(&output.join("native-intent.toml"))?;
    ensure!(
        intent.context == receipt.context
            && intent.prepared_sha256 == receipt.prepared_sha256
            && intent.candidate_sha256 == receipt.candidate_sha256
            && intent.operation_root == receipt.operation_root
            && intent.max_output_bytes == receipt.max_output_bytes
            && (1..=64).contains(&intent.fixture_jobs)
            && intent.outer_deadline_unix_ms.is_none_or(|outer| intent.deadline_unix_ms <= outer)
            && receipt.reserved_bytes <= receipt.max_output_bytes,
        "native receipt intent or aggregate budget differs"
    );
    let expected = operation_slots(prepared, run)?.into_iter().collect::<BTreeSet<_>>();
    ensure!(
        intent.operation_metadata_bytes == operation_storage::capacity(expected.len())?,
        "native operation metadata reservation changed"
    );
    let mut observed = BTreeSet::new();
    let mut reservations = Vec::new();
    let tasks = task_lookup(&prepared.manifest)?;
    receipt.visit_operations(|operation| {
        bounded::check()?;
        ensure!(
            expected.contains(&(operation.directory.clone(), operation.label.clone()))
                && observed.insert((operation.directory.clone(), operation.label.clone())),
            "native producer receipt is unexpected or duplicated"
        );
        let (passed, reservation) = authenticate_operation(prepared, &tasks, &intent, operation)?;
        ensure!(!receipt.passed || passed, "accepted native family contains a rejected producer");
        reservations.push(reservation);
        Ok(())
    })?;
    let retained =
        builds::authenticate_reservations(intent.parent_reservation_bytes, &mut reservations, intent.max_output_bytes)?;
    ensure!(
        receipt.reserved_bytes >= retained && (!receipt.passed || receipt.reserved_bytes == retained),
        "native completed reservation extent differs"
    );
    ensure!(receipt.evidence_bytes <= receipt.reserved_bytes, "native evidence exceeds retained reservation");
    ensure!(
        !receipt.passed || observed.len() == expected.len(),
        "native fixture pair compiler/execution receipts incomplete"
    );
    ensure!(
        receipt.evidence_bytes == evidence_bytes(run, receipt)? && receipt.evidence_bytes <= receipt.max_output_bytes,
        "native retained evidence extent differs"
    );
    if !receipt.passed {
        let verdict: Verdict = bounded::object(&path)?;
        ensure!(
            verdict.status == "rejected" && verdict.error.as_deref() == Some(&receipt.diagnostic),
            "native failure receipt differs"
        );
        let mut actual = BTreeMap::new();
        family::inventory_with_limits(&verdict.run, &verdict.run, &mut actual, retained_tree_limits(&verdict.run)?)?;
        ensure!(actual == verdict.artifacts, "native failed artifact inventory differs");
        return Ok(verdict);
    }
    let limits = retained_tree_limits(run)?;
    let verdict = family::authenticate_with_source_limits(
        crate::family_model::AuthenticateRequest { checkout, verdict: path },
        &prepared.pinned_source,
        limits,
    )?;
    ensure!(
        verdict.family == prepared.contract.family && verdict.source.as_ref() == Some(&prepared.source),
        "native family source identity differs"
    );
    let declared = crate::fixture_format::read(&verdict.run.join("manifest.smf"))?;
    let declared = recipes::retained_manifest(declared, prepared.contract.recipe.as_ref())?;
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

fn collect_operations(prepared: &PreparedNative, run: &Path, intent: &NativeIntent) -> Result<Vec<NativeOperation>> {
    let root = &intent.operation_root;
    let mut operations = Vec::new();
    let tasks = task_lookup(&prepared.manifest)?;
    for (directory, label) in operation_slots(prepared, run)? {
        bounded::check()?;
        let invocation_path = directory.join(format!("{label}.invocation.toml"));
        if !invocation_path.is_file() {
            continue;
        }
        let invocation: builds::StrictInvocation = bounded::object(&invocation_path)?;
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
        authenticate_operation(prepared, &tasks, intent, &operation)?;
        operations.push(operation);
    }
    Ok(operations)
}

type TaskLookup<'a> = BTreeMap<&'a str, (&'a crate::family_model::Routine, &'a crate::family_model::Case, usize)>;

fn task_lookup(manifest: &Manifest) -> Result<TaskLookup<'_>> {
    Ok(family::ordered_tasks(manifest)?.into_iter().map(|task| (task.1.name.as_str(), task)).collect())
}

fn authenticate_operation(
    prepared: &PreparedNative,
    tasks: &TaskLookup<'_>,
    intent: &NativeIntent,
    operation: &NativeOperation,
) -> Result<(bool, builds::ProducerReservation)> {
    let root = &intent.operation_root;
    let context = &intent.context;
    let invocation_path = file(&operation.invocation.file)?;
    ensure!(
        invocation_path == operation.directory.join(format!("{}.invocation.toml", operation.label))
            && digest(&invocation_path)? == operation.invocation.sha256,
        "native invocation reference changed"
    );
    let invocation: builds::StrictInvocation = bounded::object(&invocation_path)?;
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
    ensure!(invocation.program.helper == intent.helper, "native helper configuration changed");
    if compiler {
        ensure!(
            &invocation.program.executable == if rust { &intent.rustc } else { &intent.c_compiler }
                && sealed(&invocation.program.runtime)? == intent.compiler_runtime_sha256,
            "native compiler installed capabilities changed"
        );
        let recipe = prepared.contract.recipe.as_ref();
        let c_inputs = prepared.c_inputs.iter().map(|name| prepared.pinned_source.join(name)).collect::<Vec<_>>();
        let c_units =
            c_inputs.iter().filter(|path| path.extension().is_some_and(|ext| ext == "c")).cloned().collect::<Vec<_>>();
        let description = if rust {
            recipes::rust_compiler_command(recipe, &family_run)
        } else {
            recipes::c_compiler_command(
                recipe,
                &prepared.manifest,
                &prepared.pinned_source,
                &family_run,
                &c_units,
                false,
            )?
        };
        // A completed receipt may be reused from an equivalent retained
        // checkout. Recover its original binding rather than remapping it to
        // the checkout currently being authenticated.
        let root_input = if rust {
            Some(PathBuf::from(&prepared.candidate_path))
        } else if description.full_c {
            Some(PathBuf::from("tools/decomp/headless.c"))
        } else {
            None
        };
        let compiler_checkout = if let Some(relative) = root_input {
            let input = invocation
                .program
                .inputs
                .iter()
                .find(|input| input.destination == Path::new("/inputs/rust").join(&relative))
                .context("confined compiler checkout input missing")?;
            let mut checkout = input.source.clone();
            for _ in relative.components() {
                ensure!(checkout.pop(), "confined compiler checkout path truncated");
            }
            checkout
        } else {
            // Plain C compilation has no checkout capability or argument.
            family_run.clone()
        };
        if description.full_c {
            for name in ["headless.c", "headless_frame.h"] {
                ensure!(
                    prepared.contract.frozen_inputs.get(&format!("tools/decomp/{name}"))
                        == Some(&digest(&compiler_checkout.join("tools/decomp").join(name))?),
                    "native headless source changed"
                );
            }
        }
        let rust_inputs = prepared.rust_inputs.iter().map(|name| compiler_checkout.join(name)).collect::<Vec<_>>();
        let expected = builds::compiler_program(
            &description,
            &family_run,
            builds::CompilerContext {
                helper: &intent.helper,
                rustc: &intent.rustc,
                c_compiler: &intent.c_compiler,
                runtime: &invocation.program.runtime,
                rust_inputs: &rust_inputs,
                c_inputs: &c_inputs,
                checkout: &compiler_checkout,
                source: &prepared.pinned_source,
            },
        )?;
        ensure!(invocation.program == expected, "native compiler arguments, outputs or capabilities changed");
    } else {
        let selection = if operation.label == "abi-controls" {
            recipes::FixtureSelection::AbiControls
        } else {
            let name = operation
                .directory
                .file_name()
                .and_then(|name| name.to_str())
                .context("native fixture name missing")?;
            let &(routine, fixture, index) = tasks.get(name).context("undeclared native fixture")?;
            ensure!(family_run.join(&fixture.name) == operation.directory, "native fixture directory differs");
            recipes::FixtureSelection::Case { routine, fixture, index }
        };
        let expected = recipes::fixture_program(
            prepared.contract.recipe.as_ref(),
            &prepared.manifest,
            selection,
            &family_run,
            &operation.label,
            &intent.helper,
        )?;
        ensure!(invocation.program == expected, "native fixture arguments, outputs or capabilities changed");
    }
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
        let destination = recipes::fixture_output_path(&operation.directory, &operation.label, name);
        ensure!(bounded::read(destination)? == *bytes, "native parent output differs from confined producer");
    }
    if result.passed && !compiler {
        recipes::authenticate_fixture_artifacts(
            prepared.contract.recipe.as_ref(),
            &operation.directory,
            &operation.label,
        )?;
    }
    let reservation = builds::ProducerReservation {
        before_bytes: invocation.reservation_before_bytes,
        reserved_bytes: invocation.reserved_bytes,
        retained_bytes: builds::retained_producer_bytes(
            &invocation,
            &result,
            &operation.references,
            &operation.directory,
            &operation.label,
        )?,
    };
    Ok((result.passed, reservation))
}

/// Parent artifacts and durable runner dependencies required to authenticate
/// the current verification result.
pub(crate) fn references(receipt: &NativeReceipt) -> Result<Vec<Reference>> {
    source_context::until(now_unix_ms()?.saturating_add(60_000), 8 * bounded::DEFAULT_BYTES, || {
        references_bounded(receipt)
    })
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
    source_context::discovery_with_limit(retained_tree_limits(run)?.paths, || files(run, run, &mut result, 0))?;
    for name in ["native-intent.toml", "native-receipt.toml"] {
        let path = output.join(name);
        if path.exists() {
            result.insert(
                path.clone(),
                Reference { path: path.clone(), sha256: digest(&path)?, bytes: path.metadata()?.len() },
            );
        }
    }
    let storage = operation_storage::visit(receipt, |operation| {
        bounded::check()?;
        for reference in &operation.references {
            bounded::check()?;
            result.insert(reference.path.clone(), reference.clone());
            ensure!(result.len() <= 65536, "native reference inventory exceeded");
        }
        Ok(())
    })?;
    for reference in storage {
        result.insert(reference.path.clone(), reference);
        ensure!(result.len() <= 65536, "native reference inventory exceeded");
    }
    Ok(result.into_values().collect())
}

fn evidence_bytes(run: &Path, receipt: &NativeReceipt) -> Result<u64> {
    let mut seen = BTreeSet::new();
    let mut bytes = tree_bytes(run)?;
    let storage = operation_storage::visit(receipt, |operation| {
        bounded::check()?;
        for reference in &operation.references {
            bounded::check()?;
            if !reference.path.starts_with(run) && seen.insert(reference.path.clone()) {
                bytes = bytes.checked_add(reference.bytes).context("native retained byte overflow")?;
            }
        }
        Ok(())
    })?;
    for reference in storage {
        if seen.insert(reference.path) {
            bytes = bytes.checked_add(reference.bytes).context("native operation metadata extent overflow")?;
        }
    }
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
    // Reserve source archive bytes and framing. Authentication compares the
    // archive in bounded Rust buffers without temporary extraction.
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
    let limits = retained_tree_limits(path)?;
    source_context::discovery_with_limit(limits.paths, || {
        Ok(bounded::tree(path, limits.files, 8 * bounded::DEFAULT_BYTES)?.1)
    })
}

fn declared_tree_limits(manifest: &Manifest, max_output_bytes: u64) -> Result<family::ArtifactLimits> {
    let coverage = coverage(manifest)?;
    let fixtures = coverage.values().map(Vec::len).sum::<usize>();
    // Preserve the ordinary root/compiler allowance. Each declared fixture
    // needs fewer than 64 fixed parent/producer records across all recipes;
    // extra unpacked callback images each consume one full WRAM from the
    // original aggregate output budget. Do not count deduplication as relief.
    let snapshots = usize::try_from(max_output_bytes / 0x20000)?;
    let files = 65536usize
        .checked_add(fixtures.checked_mul(64).context("native fixture artifact ceiling overflow")?)
        .and_then(|files| files.checked_add(snapshots))
        .context("native retained artifact ceiling overflow")?;
    let limits =
        family::ArtifactLimits { files, paths: files.checked_mul(2).context("native retained path ceiling overflow")? };
    limits.validate()?;
    Ok(limits)
}

pub(crate) fn retained_tree_limits(run: &Path) -> Result<family::ArtifactLimits> {
    let manifest_path = run.join("manifest.smf");
    // An expired producer can fail before publishing its declaration. Such a
    // partial run gains no declared-workload allowance.
    if !manifest_path.exists() {
        return Ok(family::ArtifactLimits::default());
    }
    let manifest = crate::fixture_format::read(&manifest_path)?;
    let intent: NativeIntent =
        bounded::object(&run.parent().context("native artifact parent missing")?.join("native-intent.toml"))?;
    declared_tree_limits(&manifest, intent.max_output_bytes)
}

#[cfg(test)]
#[path = "native_tests.rs"]
mod tests;
