//! Composite acceptance for the original imported production submissions.
//! The original candidate remains a Git identity. Reviewed verification source
//! is a separate, immutable input; each tested source is overlaid on that input.
use crate::{
    assignment, batch, family, inputs::bounded, translation_cargo as cargo, translation_compact as compact,
    translation_legacy as legacy, translation_migration::OriginalClaim, translation_native as native,
};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use translation_execution::{
    model::{OperationContext, SourceSnapshot, VerificationStage, digest_valid, sealed, sha256},
    retention::{self, Reference},
};

pub(crate) const MAX_RECORD: u64 = 4 * 1024 * 1024;
pub(crate) const MAX_SOURCE: u64 = 512 * 1024 * 1024;
const MAX_FAMILY_RECORD: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum OriginalKind {
    SharedGameHost,
    CompactController,
    PauseEquipment,
}

impl OriginalKind {
    pub(crate) fn stages(self) -> Vec<VerificationStage> {
        match self {
            Self::CompactController => vec![VerificationStage::Focused],
            Self::SharedGameHost | Self::PauseEquipment => {
                vec![VerificationStage::Focused, VerificationStage::Offline]
            }
        }
    }

    pub(crate) fn limitations(self) -> Vec<String> {
        match self {
            Self::SharedGameHost => vec![
                "Original shared-host infrastructure controls and the complete frozen conservative Offline catalog only.".into(),
                "No Live replay, Ceres completion or original-ROM parity acceptance.".into(),
            ],
            Self::CompactController => vec![
                "Original compact/full controller controls and bounded causal diagnostic only.".into(),
                "Diagnostic success grants no gameplay Offline, Live replay or original-ROM parity acceptance.".into(),
            ],
            Self::PauseEquipment => vec![
                "All eight original pause equipment routines: 247 declared family pairs, 360 direct animation pairs and ten Rust controls.".into(),
                "Live pause scheduling and original-ROM parity remain untested.".into(),
            ],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct VerificationSource {
    pub root: PathBuf,
    pub files: BTreeMap<String, String>,
}

/// All executable recipes are separately sealed inputs. They are operator
/// configuration, never additions to a Rewrite capsule or Source replacement.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OriginalContract {
    pub schema: String,
    pub kind: OriginalKind,
    pub original_claim: Reference,
    pub legacy: Option<legacy::LegacyContract>,
    pub verification_source: VerificationSource,
    pub cargo: Reference,
    pub cargo_executable: Reference,
    pub cargo_rustc: Reference,
    pub families: BTreeMap<String, Reference>,
    pub compact: Option<Reference>,
    pub catalog_sha256: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PreparedOriginal {
    pub schema: String,
    pub contract: OriginalContract,
    pub original: OriginalClaim,
    pub dependencies_sha256: String,
    pub verification_sha256: String,
    pub pinned_c_sha256: String,
    pub native: BTreeMap<String, String>,
    pub cargo: cargo::PreparedCargo,
    pub compact: Option<compact::PreparedCompact>,
    pub references: Vec<Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceOverlay {
    pub source: SourceSnapshot,
    pub root: PathBuf,
    pub files: BTreeMap<String, Reference>,
}

pub(crate) struct StrictRuntime {
    pub native: native::StrictRuntime,
    pub cargo: cargo::StrictRuntime,
    pub compact: compact::StrictRuntime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OriginalReceipt {
    pub prepared_sha256: String,
    pub context: OperationContext,
    pub source: SourceSnapshot,
    pub stage: VerificationStage,
    pub record: Reference,
    pub passed: bool,
    pub diagnostic: String,
    pub evidence_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FamilyReceipt {
    pub prepared: native::PreparedNative,
    pub receipt: native::NativeReceipt,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OriginalVerdict {
    pub schema: String,
    pub prepared_sha256: String,
    pub context: OperationContext,
    pub source: SourceSnapshot,
    pub stage: VerificationStage,
    pub overlay: Reference,
    pub cargo: Option<cargo::CargoReceipt>,
    pub compact: Option<compact::CompactReceipt>,
    pub families: BTreeMap<String, Reference>,
    pub passed: bool,
    pub diagnostic: String,
    pub limitations: Vec<String>,
    pub output_limit: u64,
}

pub(crate) fn read<T: DeserializeOwned>(reference: &Reference) -> Result<T> {
    read_with_limit(reference, MAX_RECORD)
}

pub(crate) fn read_family(reference: &Reference) -> Result<FamilyReceipt> {
    read_with_limit(reference, MAX_FAMILY_RECORD)
}

fn read_with_limit<T: DeserializeOwned>(reference: &Reference, maximum: u64) -> Result<T> {
    ensure!(digest_valid(&reference.sha256) && reference.bytes <= maximum, "original acceptance input extent invalid");
    retention::authenticate_reference(reference, maximum)?;
    serde_json::from_slice(&bounded::read_limit(&reference.path, maximum)?)
        .with_context(|| format!("invalid original acceptance input {}", reference.path.display()))
}

pub(crate) fn write<T: Serialize>(path: &Path, value: &T) -> Result<Reference> {
    write_with_limit(path, value, MAX_RECORD)
}

fn write_with_limit<T: Serialize>(path: &Path, value: &T, maximum: u64) -> Result<Reference> {
    let bytes = serde_json::to_vec(value)?;
    ensure!(bytes.len() as u64 <= maximum, "original acceptance record extent exceeded");
    bounded::safe(path)?;
    fs::create_dir_all(path.parent().context("original acceptance record parent missing")?)?;
    crate::inputs::immutable_bytes(path, &bytes)?;
    retention::seal_file(path, maximum)
}

fn relative(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', '\0'])
        && path.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
        && !path.split('/').any(|part| matches!(part, ".git" | "out" | "target"))
        && (!path.starts_with(".scratch/") || path.starts_with(".scratch/translation-native/"))
}

fn lineage(contract: &OriginalContract) -> Result<OriginalClaim> {
    let original: OriginalClaim = read(&contract.original_claim)?;
    ensure!(
        original.schema == "translation-original-claim-v1"
            && original.state == assignment::State::Submitted
            && original.job.kind == assignment::Kind::Implement
            && original.claim.id == original.job.id
            && original.claim.revision == original.job.revision
            && original.claim.attempt == 1
            && original.source_commit.len() == 40
            && original.source_commit.bytes().all(|byte| byte.is_ascii_hexdigit())
            && matches!(&original.result, Some(assignment::WorkResult::Submitted { commit, .. })
                if commit == &original.source_commit),
        "original composite requires the unchanged original Submitted source and claim"
    );
    ensure!(
        original.source_files.keys().map(PathBuf::from).collect::<BTreeSet<_>>()
            == original.job.writable_paths.iter().cloned().collect(),
        "original composite changed writable ownership"
    );
    match contract.kind {
        OriginalKind::SharedGameHost | OriginalKind::CompactController => {
            let preserved = contract.legacy.as_ref().context("original Ceres acceptance contract missing")?;
            let expected = if contract.kind == OriginalKind::SharedGameHost {
                legacy::LegacyKind::SharedGameHost
            } else {
                legacy::LegacyKind::CompactController
            };
            ensure!(
                preserved.kind == expected && preserved.original_claim == contract.original_claim,
                "original Ceres contract lineage changed"
            );
            legacy::validate_original(preserved, &original)?;
        }
        OriginalKind::PauseEquipment => {
            ensure!(
                contract.legacy.is_none()
                    && original.effort == "decomp-parallel"
                    && original.job.id == "pause-equipment-render-offline"
                    && original.claim.coordinator == "decomp-run-20261013"
                    && original.job.inputs.get("admission-domain").map(String::as_str) == Some("offline-coverage-v1")
                    && original.source_files.keys().map(String::as_str).collect::<BTreeSet<_>>()
                        == BTreeSet::from([
                            "crates/port/src/pause_equipment_render.rs",
                            "crates/port/tests/pause_equipment_render.rs",
                            "tools/port/pause-equipment-render.toml",
                        ]),
                "pause composite differs from the original eight-routine scope"
            );
        }
    }
    for (name, expected) in &original.source_files {
        ensure!(relative(name) && digest_valid(expected), "unsafe original source file");
        ensure!(
            bounded::digest(&original.source_checkout.join(name))? == *expected,
            "original retained submitted source changed: {name}"
        );
    }
    for artifact in original.result.iter().flat_map(assignment::result_artifacts).chain(original.observations.iter()) {
        ensure!(bounded::digest(&artifact.path)? == artifact.sha256, "original historical evidence changed");
    }
    Ok(original)
}

fn source_references(source: &VerificationSource) -> Result<Vec<Reference>> {
    bounded::safe(&source.root)?;
    ensure!(
        source.root.is_absolute() && !source.files.is_empty() && source.files.len() <= 8192,
        "original verification source closure is empty, excessive or relative"
    );
    let mut total = 0u64;
    let mut references = Vec::new();
    for (name, expected) in &source.files {
        ensure!(relative(name) && digest_valid(expected), "original verification source alias");
        let reference = retention::seal_file(&source.root.join(name), 16 * MAX_RECORD)?;
        ensure!(reference.sha256 == *expected, "original frozen verification input changed: {name}");
        total = total.checked_add(reference.bytes).context("original source extent overflow")?;
        ensure!(total <= MAX_SOURCE, "original source closure exceeds installed extent");
        references.push(reference);
    }
    Ok(references)
}

fn required_families(
    contract: &OriginalContract,
    declarations: &BTreeMap<String, native::NativeContract>,
) -> Result<()> {
    match contract.kind {
        OriginalKind::SharedGameHost => {
            let catalog = batch::catalog(&contract.verification_source.root)?;
            ensure!(
                contract.catalog_sha256.as_deref() == Some(sealed(&catalog)?.as_str()),
                "original shared-host conservative catalog changed"
            );
            ensure!(
                declarations.keys().collect::<BTreeSet<_>>() == catalog.families.keys().collect(),
                "original shared-host acceptance omits part of the current conservative Offline catalog"
            );
            for (key, declaration) in declarations {
                ensure!(
                    batch::entry(&catalog, key)?.manifest == declaration.manifest.file,
                    "original catalog key was redirected to another Offline declaration: {key}"
                );
            }
            let mut pairs = 0;
            for name in ["game-host-projectile", "game-host-room-scroll"] {
                let declared = declarations.get(name).context("original material shared-host matrix missing")?;
                let manifest: crate::family_model::Manifest =
                    family::object(&contract.verification_source.root.join(&declared.manifest.file))?;
                ensure!(
                    manifest.callback_adapter.as_deref() == Some("game-host"),
                    "material shared-host recipe changed"
                );
                pairs += declared.coverage.values().map(Vec::len).sum::<usize>();
            }
            ensure!(pairs == 10, "material shared-host acceptance requires all ten original Fixture pairs");
            ensure!(contract.compact.is_none(), "shared-host contract contains a foreign diagnostic");
        }
        OriginalKind::CompactController => {
            ensure!(
                declarations.is_empty() && contract.catalog_sha256.is_none() && contract.compact.is_some(),
                "original compact acceptance requires its complete causal diagnostic and historical readers"
            );
        }
        OriginalKind::PauseEquipment => {
            ensure!(
                contract.compact.is_none() && contract.catalog_sha256.is_none(),
                "pause contract contains foreign gates"
            );
            let primary =
                declarations.get("pause-equipment-render").context("pause equipment primary family missing")?;
            let direct = declarations.get("pause-direct-animation").context(
                "pause equipment requires auxiliary direct animation; plain Native-family acceptance is insufficient",
            )?;
            ensure!(
                declarations.len() == 2
                    && primary.coverage.values().map(Vec::len).sum::<usize>() == 247
                    && direct.coverage.len() == 1
                    && direct.coverage.get("DrawPauseScreenSpriteAnim").is_some_and(|cases| cases.len() == 360),
                "pause equipment requires all 607 original actual-C Fixture pairs"
            );
            let routines =
                primary.coverage.keys().chain(direct.coverage.keys()).map(String::as_str).collect::<BTreeSet<_>>();
            ensure!(
                routines
                    == BTreeSet::from([
                        "DrawMenuSpritemap",
                        "DrawPauseScreenSpriteAnim",
                        "EquipmentScreenDrawItemSelector",
                        "EquipmentScreenDisplayReserveTankAmount",
                        "EquipmentScreenDisplayReserveTankAmount_",
                        "EquipmentScreenDisplayReserves_PaletteSetup",
                        "WriteSamusWireframeTilemap",
                        "WriteSamusWireframeTilemapAndQueue",
                    ]),
                "pause composite changed the eight original routines"
            );
        }
    }
    Ok(())
}

/// Read-only prerequisite authentication. A new policy must be frozen from this
/// complete result; historical readiness-only policies are not upgraded in place.
pub(crate) fn prepare(contract: &OriginalContract, pinned_source: &Path) -> Result<PreparedOriginal> {
    let deadline = bounded::now_unix_ms()?.checked_add(600_000).context("original preparation deadline overflow")?;
    bounded::until(deadline, 8 * bounded::DEFAULT_BYTES, || {
        ensure!(contract.schema == "translation-original-contract-v1", "unsupported original acceptance contract");
        let original = lineage(contract)?;
        let mut references = source_references(&contract.verification_source)?;
        if let Some(legacy) = &contract.legacy {
            references.extend(legacy::validate_original(legacy, &original)?.references);
        } else {
            for name in original.source_files.keys() {
                references.push(retention::seal_file(&original.source_checkout.join(name), 16 * MAX_RECORD)?);
            }
            for artifact in
                original.result.iter().flat_map(assignment::result_artifacts).chain(original.observations.iter())
            {
                let reference = retention::seal_file(&artifact.path, 16 * MAX_RECORD)?;
                ensure!(reference.sha256 == artifact.sha256, "original historical evidence changed");
                references.push(reference);
            }
        }
        references.push(contract.original_claim.clone());
        references.push(contract.cargo.clone());
        for executable in [&contract.cargo_executable, &contract.cargo_rustc] {
            retention::authenticate_reference(executable, MAX_SOURCE)?;
            references.push(executable.clone());
        }
        ensure!(
            contract.cargo_executable.path.parent() == contract.cargo_rustc.path.parent(),
            "original Cargo and rustc must be the reviewed matching toolchain"
        );
        let cargo_contract: cargo::CargoContract = read(&contract.cargo)?;
        let cargo_kind = match contract.kind {
            OriginalKind::SharedGameHost => cargo::CargoKind::SharedGameHost,
            OriginalKind::CompactController => cargo::CargoKind::CompactController,
            OriginalKind::PauseEquipment => cargo::CargoKind::PauseEquipment,
        };
        ensure!(
            cargo_contract.kind == cargo_kind
                && cargo_contract.approved_files.iter().collect::<BTreeSet<_>>()
                    == original.source_files.keys().collect()
                && cargo_contract
                    .source_files
                    .iter()
                    .all(|(path, hash)| contract.verification_source.files.get(path) == Some(hash)),
            "original Cargo control kind, ownership or frozen source differs"
        );
        let cargo = cargo::prepare(&contract.verification_source.root, &cargo_contract)?;
        let mut declarations = BTreeMap::new();
        for (name, reference) in &contract.families {
            let declared: native::NativeContract = read(reference)?;
            ensure!(!declared.family.is_empty(), "original Offline family identity is missing");
            if let native::NativeProvenance::ReviewedCatalog { catalog, requirements, .. } = &declared.provenance {
                for asset in [catalog, requirements] {
                    let path = if asset.file.is_absolute() {
                        asset.file.clone()
                    } else {
                        contract.verification_source.root.join(&asset.file)
                    };
                    let retained = retention::seal_file(&path, MAX_RECORD)?;
                    ensure!(retained.sha256 == asset.sha256, "original catalog provenance changed");
                    references.push(retained);
                }
            }
            declarations.insert(name.clone(), declared);
            references.push(reference.clone());
        }
        required_families(contract, &declarations)?;
        let paths = native_paths(&original);
        let mut native = BTreeMap::new();
        let mut source = BTreeSet::new();
        let catalog = declarations
            .values()
            .next()
            .map(|first| {
                native::prepare_catalog_inputs_until(
                    &contract.verification_source.root,
                    pinned_source,
                    &first.source_inventory,
                    deadline,
                    8 * bounded::DEFAULT_BYTES,
                )
            })
            .transpose()?;
        for (name, declared) in declarations {
            let prepared = native::prepare_scope_in_catalog_until(
                &contract.verification_source.root,
                pinned_source,
                &declared,
                &paths,
                catalog.as_ref().context("original catalog inputs missing")?,
                deadline,
                8 * bounded::DEFAULT_BYTES,
            )
            .with_context(|| format!("preparing original Offline family {name}"))?;
            source.insert(sealed(&prepared.source)?);
            native.insert(name, sealed(&prepared)?);
        }
        let compact = if let Some(reference) = &contract.compact {
            let declaration: compact::CompactContract = read(reference)?;
            ensure!(
                declaration
                    .source_files
                    .iter()
                    .all(|(path, hash)| contract.verification_source.files.get(path) == Some(hash)),
                "original compact verification source differs"
            );
            references.push(reference.clone());
            Some(compact::prepare(&contract.verification_source.root, &declaration)?)
        } else {
            None
        };
        Ok(PreparedOriginal {
            schema: "translation-original-prepared-v1".into(),
            contract: contract.clone(),
            original,
            dependencies_sha256: sealed(&("original-acceptance-dependencies-v1", contract, &references))?,
            verification_sha256: sealed(&(
                "original-acceptance-verification-v1",
                contract.kind,
                &native,
                &cargo,
                &compact,
            ))?,
            pinned_c_sha256: sealed(&("original-acceptance-pinned-c-v1", source))?,
            native,
            cargo,
            compact,
            references,
        })
    })
}

/// Materialize a complete immutable verification input view. Nothing in this
/// tree is writable by a compiler or Fixture invocation.
pub(crate) fn overlay(
    prepared: &PreparedOriginal,
    checkout: &Path,
    source: &SourceSnapshot,
    root: &Path,
) -> Result<SourceOverlay> {
    let record = root.join("overlay.json");
    if record.exists() {
        let saved: SourceOverlay = read(&retention::seal_file(&record, MAX_RECORD)?)?;
        ensure!(
            saved.source == *source && saved.root == root.join("source"),
            "original overlay retried with changed source"
        );
        authenticate_overlay(prepared, &saved)?;
        return Ok(saved);
    }
    ensure!(
        source.files_sha256.keys().collect::<BTreeSet<_>>() == prepared.original.source_files.keys().collect(),
        "original tested source ownership changed"
    );
    let output = root.join("source");
    let mut files = BTreeMap::new();
    let mut total = 0u64;
    for (name, baseline) in &prepared.contract.verification_source.files {
        let (input, expected) = if let Some(expected) = source.files_sha256.get(name) {
            (checkout.join(name), expected)
        } else {
            (prepared.contract.verification_source.root.join(name), baseline)
        };
        let bytes = bounded::read_limit(&input, 16 * MAX_RECORD)?;
        ensure!(sha256(&bytes) == *expected, "original overlay input changed: {name}");
        total = total.checked_add(bytes.len() as u64).context("original overlay extent overflow")?;
        ensure!(total <= MAX_SOURCE, "original overlay source quota exceeded");
        let path = output.join(name);
        bounded::safe(&path)?;
        fs::create_dir_all(path.parent().context("overlay input parent missing")?)?;
        crate::inputs::immutable_bytes(&path, &bytes)?;
        files.insert(name.clone(), retention::seal_file(&path, 16 * MAX_RECORD)?);
    }
    let result = SourceOverlay { source: source.clone(), root: output, files };
    authenticate_overlay(prepared, &result)?;
    write(&record, &result)?;
    Ok(result)
}

pub(crate) fn authenticate_overlay(prepared: &PreparedOriginal, overlay: &SourceOverlay) -> Result<()> {
    ensure!(
        overlay.files.keys().collect::<BTreeSet<_>>() == prepared.contract.verification_source.files.keys().collect(),
        "original overlay source closure changed"
    );
    ensure!(
        prepared.original.source_files.keys().all(|name| overlay.files.contains_key(name)),
        "original verification closure omitted approved source"
    );
    for (name, reference) in &overlay.files {
        let expected =
            overlay.source.files_sha256.get(name).unwrap_or(&prepared.contract.verification_source.files[name]);
        ensure!(
            reference.path == overlay.root.join(name) && reference.sha256 == *expected,
            "original overlay changed a frozen dependency"
        );
        retention::authenticate_reference(reference, 16 * MAX_RECORD)?;
    }
    Ok(())
}

fn native_paths(original: &OriginalClaim) -> Vec<String> {
    original
        .source_files
        .keys()
        .filter(|name| {
            Path::new(name).parent() == Some(Path::new("crates/port/src"))
                && name.ends_with(".rs")
                && !name.ends_with("/lib.rs")
                && !name.ends_with("/memory.rs")
        })
        .cloned()
        .collect()
}

fn child(context: &OperationContext, label: &str) -> Result<OperationContext> {
    let mut child = context.clone();
    child.operation_key = sealed(&("original-acceptance-operation-v1", &context.operation_key, label))?;
    Ok(child)
}

/// The supervisor reserves each original component before a producer starts.
/// Source overlays and verdicts have a separate bounded metadata allowance.
pub(crate) fn gate_count(prepared: &PreparedOriginal, stage: VerificationStage) -> Result<usize> {
    ensure!(prepared.contract.kind.stages().contains(&stage), "original acceptance stage was not prescribed");
    Ok(match stage {
        VerificationStage::Focused => 1 + usize::from(prepared.compact.is_some()),
        VerificationStage::Offline => prepared.native.len(),
        VerificationStage::Live => bail!("original diagnostic/Offline contract cannot establish Live acceptance"),
    })
}

pub(crate) const CARGO_CAPACITY: u64 = 4 * 1024 * 1024 * 1024;
const COMPACT_CAPACITY: u64 = 8 * 1024 * 1024 * 1024;
pub(crate) const ENVIRONMENT_CAPACITY: u64 = CARGO_CAPACITY + MAX_SOURCE + MAX_RECORD;

pub(crate) fn gate_capacity(prepared: &PreparedOriginal, stage: VerificationStage, installed: u64) -> Result<u64> {
    gate_count(prepared, stage)?;
    let requested = match stage {
        VerificationStage::Focused => {
            MAX_SOURCE + 4 * MAX_RECORD + CARGO_CAPACITY + if prepared.compact.is_some() { COMPACT_CAPACITY } else { 0 }
        }
        VerificationStage::Offline => installed,
        VerificationStage::Live => unreachable!(),
    };
    ensure!(requested <= installed, "installed original gate cannot reserve its prescribed components");
    Ok(requested)
}

fn family_capacity(contract: &native::NativeContract) -> Result<u64> {
    let cases = contract.coverage.values().map(Vec::len).sum::<usize>() as u64;
    // Native reserves writable output three times (namespace, imported output,
    // and retained material), plus bounded runtime/reference metadata. The
    // source allowance also covers generated fixture state and receipt records.
    let compiler = contract
        .compiler_limits
        .writable_bytes
        .checked_mul(6)
        .and_then(|bytes| bytes.checked_add(16 * MAX_RECORD))
        .context("original compiler capacity overflow")?;
    let fixture = contract
        .fixture_limits
        .writable_bytes
        .checked_mul(6)
        .and_then(|bytes| bytes.checked_add(2 * MAX_RECORD))
        .context("original fixture capacity overflow")?;
    (2 * MAX_SOURCE)
        .checked_add(compiler)
        .and_then(|bytes| fixture.checked_mul(cases + 1).and_then(|fixtures| bytes.checked_add(fixtures)))
        .context("original family capacity overflow")
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run(
    prepared: &PreparedOriginal,
    pinned_source: &Path,
    checkout: &Path,
    source: &SourceSnapshot,
    output: &Path,
    stage: VerificationStage,
    runtime: &StrictRuntime,
    context: &OperationContext,
) -> Result<OriginalReceipt> {
    let receipt_path = output.join("receipt.json");
    if receipt_path.exists() {
        let receipt: OriginalReceipt = read(&retention::seal_file(&receipt_path, MAX_RECORD)?)?;
        ensure!(
            receipt.context == *context && receipt.source == *source && receipt.stage == stage,
            "original acceptance retry changed its source or operation"
        );
        authenticate(prepared, &receipt)?;
        return Ok(receipt);
    }
    gate_count(prepared, stage)?;
    let output_limit = gate_capacity(prepared, stage, runtime.native.max_output_bytes)?;
    let overlay = overlay(prepared, checkout, source, &output.join("input"))?;
    let overlay_reference = retention::seal_file(&output.join("input/overlay.json"), MAX_RECORD)?;
    let mut cargo_receipt = None;
    let mut compact_receipt = None;
    let mut families = BTreeMap::new();
    let mut passed = true;
    let mut diagnostic = String::new();
    let mut retained =
        prepared.references.iter().chain(overlay.files.values()).try_fold(4 * MAX_RECORD, |sum, reference| {
            sum.checked_add(reference.bytes).context("original initial extent overflow")
        })?;
    match stage {
        VerificationStage::Focused => {
            let mut cargo_runtime = runtime.cargo.clone();
            cargo_runtime.max_output_bytes = CARGO_CAPACITY;
            let receipt = cargo::run(
                &prepared.cargo,
                &overlay.root,
                &output.join("cargo"),
                &cargo_runtime,
                &child(context, "cargo")?,
            )?;
            cargo::authenticate(&prepared.cargo, &overlay.root, &receipt)?;
            passed = receipt.passed;
            diagnostic = receipt.diagnostic.clone();
            cargo_receipt = Some(receipt);
            if passed && let Some(compact) = &prepared.compact {
                let mut compact_runtime = runtime.compact.clone();
                compact_runtime.max_output_bytes = COMPACT_CAPACITY;
                let receipt = compact::run(
                    compact,
                    &overlay.root,
                    &output.join("compact"),
                    &compact_runtime,
                    &child(context, "compact")?,
                )?;
                compact::authenticate(compact, &overlay.root, &receipt)?;
                passed = receipt.passed;
                diagnostic = receipt.diagnostic.clone();
                compact_receipt = Some(receipt);
            }
        }
        VerificationStage::Offline => {
            let paths = native_paths(&prepared.original);
            let first: native::NativeContract =
                read(prepared.contract.families.values().next().context("original Offline catalog empty")?)?;
            let deadline = runtime.native.deadline_unix_ms.context("original Offline stage deadline missing")?;
            let catalog = native::prepare_catalog_inputs_until(
                &prepared.contract.verification_source.root,
                pinned_source,
                &first.source_inventory,
                deadline,
                8 * bounded::DEFAULT_BYTES,
            )?;
            for (name, fingerprint) in &prepared.native {
                let declaration: native::NativeContract = read(&prepared.contract.families[name])?;
                // Preparation always reads the reviewed baseline. Candidate
                // variability is limited by the separately bound approved set.
                let family = native::prepare_scope_in_catalog_until(
                    &prepared.contract.verification_source.root,
                    pinned_source,
                    &declaration,
                    &paths,
                    &catalog,
                    deadline,
                    8 * bounded::DEFAULT_BYTES,
                )?;
                ensure!(sealed(&family)? == *fingerprint, "original native preparation changed: {name}");
                let capacity = family_capacity(&declaration)?.min(output_limit.saturating_sub(retained));
                ensure!(
                    capacity >= MAX_FAMILY_RECORD,
                    "original catalog evidence budget exhausted before family launch"
                );
                let family_runtime = native::StrictRuntime {
                    helper: runtime.native.helper.clone(),
                    rustc: runtime.native.rustc.clone(),
                    c_compiler: runtime.native.c_compiler.clone(),
                    operation_root: runtime.native.operation_root.clone(),
                    pipeline_root: runtime.native.pipeline_root.clone(),
                    workers: runtime.native.workers,
                    gate_timeout_seconds: runtime.native.gate_timeout_seconds,
                    deadline_unix_ms: runtime.native.deadline_unix_ms,
                    max_output_bytes: capacity,
                };
                let receipt = native::run(
                    &family,
                    &overlay.root,
                    &output.join("native").join(name),
                    &family_runtime,
                    &child(context, name)?,
                )?;
                native::authenticate(&family, &overlay.root, &receipt)?;
                retained = retained.checked_add(receipt.evidence_bytes).context("original catalog extent overflow")?;
                passed = receipt.passed;
                diagnostic = receipt.diagnostic.clone();
                let reference = write_with_limit(
                    &output.join("families").join(format!("{name}.json")),
                    &FamilyReceipt { prepared: family, receipt },
                    MAX_FAMILY_RECORD,
                )?;
                retained = retained.checked_add(reference.bytes).context("original catalog record extent overflow")?;
                ensure!(retained <= output_limit, "original catalog exceeded its reserved capacity");
                families.insert(name.clone(), reference);
                if !passed {
                    break;
                }
            }
        }
        VerificationStage::Live => unreachable!("stage checked above"),
    }
    let verdict = OriginalVerdict {
        schema: "translation-original-verdict-v1".into(),
        prepared_sha256: sealed(prepared)?,
        context: context.clone(),
        source: source.clone(),
        stage,
        overlay: overlay_reference,
        cargo: cargo_receipt,
        compact: compact_receipt,
        families,
        passed,
        diagnostic,
        limitations: prepared.contract.kind.limitations(),
        output_limit,
    };
    let record = write(&output.join("verdict.json"), &verdict)?;
    let mut receipt = OriginalReceipt {
        prepared_sha256: verdict.prepared_sha256.clone(),
        context: context.clone(),
        source: source.clone(),
        stage,
        record,
        passed: verdict.passed,
        diagnostic: verdict.diagnostic.clone(),
        evidence_bytes: 0,
    };
    receipt.evidence_bytes = references(prepared, &receipt)?.iter().try_fold(0u64, |total, reference| {
        total.checked_add(reference.bytes).context("original acceptance evidence extent overflow")
    })?;
    ensure!(receipt.evidence_bytes <= output_limit, "original composite exceeded reserved evidence capacity");
    authenticate(prepared, &receipt)?;
    write(&receipt_path, &receipt)?;
    Ok(receipt)
}

/// Recompute required coverage and each subordinate result. Stored counts and
/// aggregate pass booleans cannot substitute for independent evidence readers.
pub(crate) fn authenticate(prepared: &PreparedOriginal, receipt: &OriginalReceipt) -> Result<OriginalVerdict> {
    let verdict: OriginalVerdict = read(&receipt.record)?;
    ensure!(
        verdict.schema == "translation-original-verdict-v1"
            && verdict.prepared_sha256 == sealed(prepared)?
            && receipt.prepared_sha256 == verdict.prepared_sha256
            && verdict.context == receipt.context
            && verdict.source == receipt.source
            && verdict.stage == receipt.stage
            && verdict.passed == receipt.passed
            && verdict.diagnostic == receipt.diagnostic
            && verdict.limitations == prepared.contract.kind.limitations(),
        "original composite receipt differs from its frozen source and requirements"
    );
    gate_count(prepared, receipt.stage)?;
    let overlay: SourceOverlay = read(&verdict.overlay)?;
    ensure!(overlay.source == receipt.source, "original evidence overlay identifies a different Git source");
    authenticate_overlay(prepared, &overlay)?;
    let mut passed = true;
    let mut diagnosis = String::new();
    match verdict.stage {
        VerificationStage::Focused => {
            ensure!(verdict.families.is_empty(), "original controls contain an unprescribed Offline result");
            let receipt = verdict.cargo.as_ref().context("original Rust controls are missing")?;
            ensure!(receipt.context == child(&verdict.context, "cargo")?, "original Cargo operation changed");
            let result = cargo::authenticate(&prepared.cargo, &overlay.root, receipt)?;
            passed = result.passed;
            diagnosis = receipt.diagnostic.clone();
            if let Some(compact) = &prepared.compact {
                if passed {
                    let receipt = verdict.compact.as_ref().context("original whole-experiment diagnostic missing")?;
                    ensure!(
                        receipt.context == child(&verdict.context, "compact")?,
                        "original compact operation changed"
                    );
                    compact::authenticate(compact, &overlay.root, receipt)?;
                    passed = receipt.passed;
                    diagnosis = receipt.diagnostic.clone();
                } else {
                    ensure!(verdict.compact.is_none(), "compact capture ran after a stopped Rust control");
                }
            } else {
                ensure!(verdict.compact.is_none(), "foreign compact diagnostic in original controls");
            }
        }
        VerificationStage::Offline => {
            ensure!(
                verdict.cargo.is_none() && verdict.compact.is_none() && !verdict.families.is_empty(),
                "original Offline result components differ"
            );
            let expected = prepared.native.keys().take(verdict.families.len()).collect::<Vec<_>>();
            ensure!(
                expected == verdict.families.keys().collect::<Vec<_>>(),
                "original Offline result is not the prescribed complete prefix"
            );
            for (name, reference) in &verdict.families {
                ensure!(passed, "original Offline producer continued after a stopped family");
                let family = read_family(reference)?;
                ensure!(
                    prepared.native.get(name) == Some(&sealed(&family.prepared)?)
                        && family.receipt.context == child(&verdict.context, name)?,
                    "original native recipe, source or operation changed: {name}"
                );
                native::authenticate(&family.prepared, &overlay.root, &family.receipt)?;
                passed = family.receipt.passed;
                diagnosis = family.receipt.diagnostic.clone();
            }
            ensure!(
                !passed || verdict.families.len() == prepared.native.len(),
                "passing original Offline result omitted a required family"
            );
        }
        VerificationStage::Live => unreachable!("stage checked above"),
    }
    ensure!(
        passed == verdict.passed && diagnosis == verdict.diagnostic,
        "original composite result was not derived from its evidence"
    );
    let bytes = references(prepared, receipt)?.iter().try_fold(0u64, |sum, reference| {
        sum.checked_add(reference.bytes).context("original evidence extent overflow")
    })?;
    ensure!(
        bytes == receipt.evidence_bytes && bytes <= verdict.output_limit,
        "original composite evidence extent changed"
    );
    Ok(verdict)
}

pub(crate) fn references(prepared: &PreparedOriginal, receipt: &OriginalReceipt) -> Result<Vec<Reference>> {
    let verdict: OriginalVerdict = read(&receipt.record)?;
    let overlay: SourceOverlay = read(&verdict.overlay)?;
    let mut references = prepared.references.clone();
    references.extend([receipt.record.clone(), verdict.overlay.clone()]);
    references.extend(overlay.files.into_values());
    if let Some(receipt) = &verdict.cargo {
        references.extend(cargo::references(receipt)?);
    }
    if let Some(receipt) = &verdict.compact {
        references.extend(compact::references(receipt)?);
    }
    for reference in verdict.families.values() {
        let family = read_family(reference)?;
        references.push(reference.clone());
        references.extend(native::references(&family.receipt)?);
    }
    let mut unique = BTreeMap::new();
    for reference in references {
        if let Some(prior) = unique.insert(reference.path.clone(), reference.clone()) {
            ensure!(prior == reference, "original evidence contains conflicting references");
        }
    }
    Ok(unique.into_values().collect())
}

#[cfg(test)]
#[path = "translation_original_tests.rs"]
mod tests;
