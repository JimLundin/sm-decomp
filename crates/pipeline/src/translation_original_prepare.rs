//! Read-only source preparation for the original production contracts. The
//! result is a separately sealed installation; it never edits an original
//! migration, claim, submission, policy or stopped attempt.
use crate::{
    translation_cargo as cargo, translation_compact as compact, translation_continuation as continuation,
    translation_execution::{Contract, Installation, InstalledFile},
    translation_migration::{self as migration, Migration, OriginalClaim},
    translation_original as original, translation_production as production,
    translation_production_prepare as deployment,
};
use anyhow::{Context, Result, ensure};
use offline_verification::{self as native, catalog as batch, fixtures::Asset};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use tool_execution::bounded;
use translation_execution::{
    model::sealed,
    retention::{self, Reference},
};

const MAX_RECORD: u64 = 4 * 1024 * 1024;
const MAX_DEPENDENCIES: u64 = 512 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    pub schema: String,
    pub checkout: PathBuf,
    pub output_root: PathBuf,
    pub registry: Reference,
    pub template: Reference,
    pub helper: Reference,
    pub rustc: Reference,
    pub cargo: Reference,
    pub cargo_rustc: Reference,
    pub c_compiler: Reference,
    pub vendor: cargo::CargoClosure,
    pub compact_dependency_cache: Option<cargo::CargoClosure>,
    pub requirements: Reference,
    pub direct_animation: Reference,
    pub reviewed_head: String,
    pub gate_timeout_seconds: u64,
    pub max_gate_bytes: u64,
    pub max_checkout_bytes: u64,
    pub max_evidence_bytes: u64,
    pub reason: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Intent {
    schema: String,
    config: Reference,
    controller: Reference,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Report {
    pub schema: String,
    pub config: Reference,
    pub installation: Reference,
    pub controller: Reference,
    pub scopes: Vec<ScopeReport>,
    pub limitation: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScopeReport {
    pub id: String,
    pub native_families: usize,
}

fn installed(reference: &Reference) -> Result<InstalledFile> {
    retention::authenticate_reference(reference, MAX_DEPENDENCIES)?;
    Ok(InstalledFile { path: reference.path.clone(), sha256: reference.sha256.clone() })
}

fn collect(root: &Path, directory: &Path, files: &mut BTreeSet<String>) -> Result<()> {
    bounded::safe(directory)?;
    for path in bounded::entries(directory)? {
        let leaf = path.file_name().and_then(|name| name.to_str()).context("source name is not UTF8")?;
        if matches!(leaf, ".git" | "target" | "out") {
            continue;
        }
        let metadata = path.symlink_metadata()?;
        ensure!(!metadata.file_type().is_symlink(), "original source snapshot crosses a symlink");
        if metadata.is_dir() {
            collect(root, &path, files)?;
        } else {
            ensure!(metadata.is_file(), "original source snapshot contains a special file");
            files.insert(path.strip_prefix(root)?.to_str().context("source name is not UTF8")?.into());
            ensure!(files.len() <= 8192, "original source inventory exceeded");
        }
    }
    Ok(())
}

fn snapshot(config: &Config, claim: &OriginalClaim, output: &Path) -> Result<original::VerificationSource> {
    let mut selected = BTreeSet::new();
    for directory in ["crates", "tools/port", "tools/decomp"] {
        // Source is checked against the reviewed HEAD before and after this
        // copy. Only declared original ownership uses the Submitted snapshot.
        collect(&config.checkout, &config.checkout.join(directory), &mut selected)?;
    }
    selected.extend(["Cargo.toml", "Cargo.lock", "tools/dependencies.toml", "docs/DECOMP_PREFIX.md"].map(String::from));
    selected.extend(claim.source_files.keys().cloned());
    let mut files = BTreeMap::new();
    let mut extent = 0u64;
    for name in selected {
        let path = if claim.source_files.contains_key(&name) {
            claim.source_checkout.join(&name)
        } else {
            config.checkout.join(&name)
        };
        let reference = continuation::copy(&path, &output.join(&name), 64 * 1024 * 1024)?;
        if let Some(expected) = claim.source_files.get(&name) {
            ensure!(reference.sha256 == *expected, "original Submitted source changed during preparation");
        }
        extent = extent.checked_add(reference.bytes).context("original source extent overflow")?;
        ensure!(extent <= original::MAX_SOURCE, "original verifier source capacity exceeded");
        files.insert(name, reference.sha256);
    }
    let rustysnes = fs::canonicalize(config.checkout.join("deps/rustysnes"))?;
    let mut dependencies = BTreeSet::from(["Cargo.toml".to_owned()]);
    collect(&rustysnes, &rustysnes.join("crates"), &mut dependencies)?;
    for name in dependencies {
        let target = format!("deps/rustysnes/{name}");
        let reference = continuation::copy(&rustysnes.join(name), &output.join(&target), 64 * 1024 * 1024)?;
        extent = extent.checked_add(reference.bytes).context("Rust dependency extent overflow")?;
        files.insert(target, reference.sha256);
    }
    let c_source = fs::canonicalize(config.checkout.join("deps/snesrev-sm"))?;
    let mut c_files = BTreeSet::new();
    collect(&c_source, &c_source.join("src"), &mut c_files)?;
    for name in c_files {
        let target = format!("deps/snesrev-sm/{name}");
        let reference = continuation::copy(&c_source.join(name), &output.join(&target), 64 * 1024 * 1024)?;
        extent = extent.checked_add(reference.bytes).context("C dependency extent overflow")?;
        files.insert(target, reference.sha256);
    }
    let rom = continuation::copy(&config.checkout.join("rom/sm.sfc"), &output.join("rom/sm.sfc"), 4 * 1024 * 1024)?;
    files.insert("rom/sm.sfc".into(), rom.sha256);
    let plan = ".scratch/translation-native/pause-direct-animation.toml";
    let reference = continuation::copy(&config.direct_animation.path, &output.join(plan), MAX_RECORD)?;
    ensure!(reference.sha256 == config.direct_animation.sha256, "direct animation review changed");
    files.insert(plan.into(), reference.sha256);
    if !output.join(".scratch/translation-native/plm-instructions/manifest.smf").exists() {
        crate::offline::families()?
            .prepare_generated_plm(output, &output.join(".scratch/translation-native/plm-instructions"))?;
    }
    let mut generated = BTreeSet::new();
    collect(output, &output.join(".scratch/translation-native/plm-instructions"), &mut generated)?;
    for name in generated {
        let reference = retention::seal_file(&output.join(&name), 64 * 1024 * 1024)?;
        fs::set_permissions(&reference.path, fs::Permissions::from_mode(0o400))?;
        extent = extent.checked_add(reference.bytes).context("generated source extent overflow")?;
        files.insert(name, reference.sha256);
    }
    ensure!(
        extent <= original::MAX_SOURCE && files.len() <= 8192,
        "complete original source exceeds reviewed capacity"
    );
    Ok(original::VerificationSource { root: output.into(), files })
}

fn controls(
    config: &Config,
    kind: original::OriginalKind,
    source: &original::VerificationSource,
    claim: &OriginalClaim,
) -> cargo::CargoContract {
    let cargo_kind = match kind {
        original::OriginalKind::SharedGameHost => cargo::CargoKind::SharedGameHost,
        original::OriginalKind::CompactController => cargo::CargoKind::CompactController,
        original::OriginalKind::PauseEquipment => cargo::CargoKind::PauseEquipment,
    };
    let files = source
        .files
        .iter()
        .filter(|(name, _)| {
            name.starts_with("crates/")
                || name.starts_with("deps/rustysnes/")
                || name.starts_with("deps/snesrev-sm/src/")
                || matches!(name.as_str(), "Cargo.toml" | "Cargo.lock" | "tools/decomp/headless.c")
                || claim.source_files.contains_key(*name)
        })
        .map(|(name, sha)| (name.clone(), sha.clone()))
        .collect();
    cargo::CargoContract {
        schema: "translation-cargo-contract-v1".into(),
        kind: cargo_kind,
        source_files: files,
        approved_files: claim.source_files.keys().cloned().collect(),
        vendor: config.vendor.clone(),
        dependency_cache: if kind == original::OriginalKind::CompactController {
            config.compact_dependency_cache.clone()
        } else {
            None
        },
        limits: cargo::CargoLimits {
            timeout_seconds: 60,
            writable_bytes: if kind == original::OriginalKind::CompactController {
                768 * 1024 * 1024
            } else {
                256 * 1024 * 1024
            },
            stdout_bytes: 256 * 1024,
            stderr_bytes: 64 * 1024,
        },
    }
}

pub(crate) fn prepare(path: &Path) -> Result<Report> {
    let reference = retention::seal_file(path, MAX_RECORD)?;
    let config: Config = original::read(&reference)?;
    bounded::safe(&config.checkout)?;
    bounded::safe(&config.output_root)?;
    ensure!(
        config.schema == "translation-original-preparation-v1"
            && !config.output_root.starts_with(&config.checkout)
            && !config.checkout.starts_with(&config.output_root)
            && (600..=3600).contains(&config.gate_timeout_seconds)
            && config.max_gate_bytes >= 16 * 1024 * 1024 * 1024
            && config.max_checkout_bytes >= 2 * 1024 * 1024 * 1024
            && config.max_gate_bytes.checked_mul(2).is_some_and(|bytes| config.max_evidence_bytes > bytes)
            && !config.reason.trim().is_empty(),
        "original preparation roots, limits or review are invalid"
    );
    let _lease = continuation::lease(&config.output_root)?;
    let registry: production::Registry = original::read(&config.registry)?;
    ensure!(
        registry.checkout == config.checkout && config.registry.path == config.checkout.join(production::REGISTRY),
        "original preparation changed ownership registry"
    );
    let request: production::Request = original::read(&registry.request)?;
    ensure!(request.installations.contains(&config.template), "original template lost registry membership");
    let mut template: Installation = original::read(&config.template)?;
    let current = std::env::current_exe()?;
    let current = retention::seal_file(&current, MAX_DEPENDENCIES)?;
    let intent = Intent {
        schema: "translation-original-preparation-intent-v1".into(),
        config: reference.clone(),
        controller: current.clone(),
    };
    original::write(&config.output_root.join("preparation-intent.json"), &intent)?;
    ensure!(
        migration::git(&config.checkout, &["rev-parse", "HEAD"])? == config.reviewed_head,
        "original preparation requires its reviewed source HEAD"
    );
    // This rejects dirty source while preserving the exact initial .scratch
    // inventory for explicit installation review.
    continuation::paths(&config.checkout)?;
    retention::authenticate_reference(&config.requirements, MAX_RECORD)?;
    retention::authenticate_reference(&config.direct_animation, MAX_RECORD)?;
    let requirements =
        continuation::copy(&config.requirements.path, &config.output_root.join("inputs/requirements.md"), MAX_RECORD)?;
    let controller = deployment::install(&current, &config.output_root.join("bin/sm-pipeline"))?;
    let helper = deployment::install(&config.helper, &config.output_root.join("bin/translation-sandbox"))?;
    template.controller = installed(&controller)?;
    template.helper = installed(&helper)?;
    template.rewrite.executable = installed(&helper)?;
    template.rustc = installed(&config.rustc)?;
    template.c_compiler = installed(&config.c_compiler)?;
    installed(&config.cargo)?;
    installed(&config.cargo_rustc)?;
    template.policy.budgets.max_attempts = 1;
    template.policy.budgets.gate_timeout_seconds = config.gate_timeout_seconds;
    template.policy.budgets.max_evidence_bytes = config.max_evidence_bytes;
    template.max_gate_bytes = config.max_gate_bytes;
    template.max_checkout_bytes = config.max_checkout_bytes;
    let template_reference = original::write(&config.output_root.join("template.json"), &template)?;
    let mut scopes = Vec::new();
    let mut report = Vec::new();
    for (kind, id) in [
        (original::OriginalKind::SharedGameHost, "shared-game-host"),
        (original::OriginalKind::CompactController, "compact-controller"),
        (original::OriginalKind::PauseEquipment, "pause-equipment-render-offline"),
    ] {
        let mut selected = None;
        for domain_reference in &registry.migrations {
            let domain: Migration = original::read(domain_reference)?;
            if let Some(claim) = domain.claims.iter().find(|claim| claim.claim.id == id) {
                ensure!(selected.is_none(), "original production repeats a claim");
                selected = Some((domain_reference.clone(), claim.original_claim.clone()));
            }
        }
        let (domain, claim_reference) = selected.context("required original production claim is missing")?;
        let claim: OriginalClaim = original::read(&claim_reference)?;
        let scope_root = config.output_root.join("contracts").join(id);
        let source = snapshot(&config, &claim, &scope_root.join("source"))?;
        let cargo = controls(&config, kind, &source, &claim);
        let cargo = original::write(&scope_root.join("cargo.json"), &cargo)?;
        let catalog = batch::catalog(&source.root)?;
        let catalog_path = scope_root.join("catalog.toml");
        bounded::immutable_bytes(&catalog_path, toml::to_string(&catalog)?.as_bytes())?;
        let catalog_reference = retention::seal_file(&catalog_path, MAX_RECORD)?;
        let inventory = Asset {
            file: "tools/port/source-inventory.toml".into(),
            sha256: source.files["tools/port/source-inventory.toml"].clone(),
        };
        let provenance = native::NativeProvenance::ReviewedCatalog {
            catalog: Asset { file: catalog_path, sha256: catalog_reference.sha256 },
            requirements: Asset { file: requirements.path.clone(), sha256: requirements.sha256.clone() },
            checkout: source.root.clone(),
            rationale: config.reason.clone(),
        };
        let compiler = native::NativeLimits {
            timeout_seconds: 180,
            writable_bytes: 128 * 1024 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 256 * 1024,
            stdin_bytes: 0,
        };
        let fixture = |callback: bool| native::NativeLimits {
            timeout_seconds: 5,
            writable_bytes: if callback { 16 * 1024 * 1024 } else { 512 * 1024 },
            stdout_bytes: 4096,
            stderr_bytes: 4096,
            stdin_bytes: 0,
        };
        let names = match kind {
            original::OriginalKind::SharedGameHost => catalog.families.keys().cloned().collect::<Vec<_>>(),
            original::OriginalKind::CompactController => vec![],
            original::OriginalKind::PauseEquipment => vec!["pause-equipment-render".into()],
        };
        let mut families = BTreeMap::new();
        for name in names {
            let entry = batch::entry(&catalog, &name)?;
            let callback = matches!(entry.kind.as_str(), "callback" | "power-bomb" | "ridley-subsystem");
            let contract = crate::offline::families()?
                .declare_native(native::NativeDeclaration {
                    checkout: &source.root,
                    kind: native::NativeDeclarationKind::Family(&entry.manifest),
                    source_inventory: inventory.clone(),
                    provenance: provenance.clone(),
                    compiler_limits: compiler.clone(),
                    fixture_limits: fixture(callback),
                    deadline_unix_ms: None,
                    max_input_bytes: bounded::DEFAULT_BYTES,
                })
                .with_context(|| format!("preparing original {id} Offline family {name}"))?;
            families.insert(
                name.clone(),
                original::write(&scope_root.join("families").join(format!("{name}.json")), &contract)?,
            );
        }
        if kind == original::OriginalKind::PauseEquipment {
            let direct = crate::offline::families()?.declare_native(native::NativeDeclaration {
                checkout: &source.root,
                kind: native::NativeDeclarationKind::DirectAnimation(Path::new(
                    ".scratch/translation-native/pause-direct-animation.toml",
                )),
                source_inventory: inventory,
                provenance,
                compiler_limits: compiler,
                fixture_limits: fixture(false),
                deadline_unix_ms: None,
                max_input_bytes: bounded::DEFAULT_BYTES,
            })?;
            families.insert(
                direct.family.clone(),
                original::write(&scope_root.join("families/pause-direct-animation.json"), &direct)?,
            );
        }
        let legacy = if kind != original::OriginalKind::PauseEquipment {
            let prior = request
                .installations
                .iter()
                .map(|reference| original::read::<Installation>(reference))
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .find(|installed| installed.policy.claim.id == id)
                .context("original Ceres installation missing")?;
            let Contract::Legacy { contract } = prior.contract else {
                anyhow::bail!("original Ceres policy changed");
            };
            Some(contract)
        } else {
            None
        };
        let compact = if kind == original::OriginalKind::CompactController {
            let contract = compact::contract_for_original(
                &source.root,
                &config.checkout,
                &template.rustc.path,
                compact::historical_for_original(&config.checkout)?,
            )?;
            Some(original::write(&scope_root.join("compact.json"), &contract)?)
        } else {
            None
        };
        let contract = original::OriginalContract {
            schema: "translation-original-contract-v1".into(),
            kind,
            original_claim: claim_reference.clone(),
            legacy,
            verification_source: source,
            cargo,
            cargo_executable: config.cargo.clone(),
            cargo_rustc: config.cargo_rustc.clone(),
            families,
            compact,
            catalog_sha256: (kind == original::OriginalKind::SharedGameHost).then(|| sealed(&catalog)).transpose()?,
        };
        let prepared = original::prepare(&contract, &template.pinned_source)
            .with_context(|| format!("authenticating complete original {id} preparation"))?;
        let contract = original::write(&scope_root.join("contract.json"), &contract)?;
        report.push(ScopeReport { id: id.into(), native_families: prepared.native.len() });
        scopes.push(continuation::Scope {
            id: id.into(),
            migration: domain,
            original_claim: claim_reference,
            contract,
            prepared: original::write(&scope_root.join("prepared.json"), &prepared)?,
        });
    }
    ensure!(
        migration::git(&config.checkout, &["rev-parse", "HEAD"])? == config.reviewed_head,
        "original preparation source advanced before review"
    );
    let preserved = continuation::paths(&config.checkout)?
        .into_iter()
        .map(|path| retention::seal_file(&path, 1024 * 1024 * 1024))
        .collect::<Result<Vec<_>>>()?;
    let stopped: Installation = original::read(&config.template)?;
    let stopped_view = migration::controller_inspect(&stopped.controller.path, &config.template.path, &stopped)?;
    let installation = continuation::Config {
        schema: "translation-original-production-config-v1".into(),
        checkout: config.checkout.clone(),
        registry: config.registry.clone(),
        output_root: config.output_root.clone(),
        template: template_reference,
        reviewed_head: config.reviewed_head.clone(),
        preserved,
        stopped_installation: config.template.clone(),
        stopped_view,
        scopes,
        reason: config.reason,
    };
    let report = Report { schema: "translation-original-preparation-report-v1".into(), config: reference,
        installation: original::write(&config.output_root.join("original-production.json"), &installation)?, controller,
        scopes: report, limitation: "Preparation only; every original acceptance gate remains to be executed on exact candidate and integrated source.".into() };
    original::write(&config.output_root.join("preparation-report.json"), &report)?;
    Ok(report)
}
