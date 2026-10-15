//! Install an immutable production switch request from preserved migration
//! packages. Preparation writes only the declared external output root; the
//! separate production command owns activation and all queue mutations.
use crate::assignment;
use crate::translation_execution as execution;
use crate::translation_execution::Admission;
use crate::translation_execution::Contract;
use crate::translation_execution::Installation;
use crate::translation_execution::InstalledFile;
use crate::translation_launch::Plan;
use crate::translation_launch::Scope;
use crate::translation_legacy as legacy;
use crate::translation_legacy::LegacyContract;
use crate::translation_legacy::LegacyKind;
use crate::translation_migration as migration;
use crate::translation_migration::Migration;
use crate::translation_migration::MigrationExecution;
use crate::translation_migration::OriginalClaim;
use crate::translation_production::BlockedClaim;
use crate::translation_production::Deployment;
use crate::translation_production::Request;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::{
        fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt},
        io::AsRawFd,
    },
    path::{Component, Path, PathBuf},
};
use translation_execution::{
    model::{digest_valid, sealed, sha256},
    retention::{self, Reference},
};

const MAX_RECORD: u64 = 4 * 1024 * 1024;
const MAX_TOOL: u64 = 512 * 1024 * 1024;
const AUTHORITY: &str = ".scratch/decomp-parallel/authority.toml";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Executable {
    pub effort: String,
    pub claim_id: String,
    pub kind: LegacyKind,
    pub submission: Reference,
    pub acceptance_inputs: Vec<Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Blocked {
    pub effort: String,
    pub claim_id: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    pub schema: String,
    pub output_root: PathBuf,
    pub packages: Vec<Reference>,
    pub helper: Reference,
    pub template: Reference,
    pub pinned_source: PathBuf,
    pub integration_baseline: String,
    pub executable: Vec<Executable>,
    pub blocked: Vec<Blocked>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Intent {
    schema: String,
    config: Reference,
    config_sha256: String,
    controller_sha256: String,
    controller_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema: String,
    intent: Reference,
    request: Reference,
    outputs: Vec<Reference>,
}

fn read<T: DeserializeOwned>(path: &Path) -> Result<T> {
    ensure!(path.is_absolute(), "production preparation input must be absolute");
    tool_execution::bounded::safe(path)?;
    serde_json::from_slice(&tool_execution::bounded::read_limit(path, MAX_RECORD)?)
        .context("invalid production preparation JSON")
}

fn authenticate(reference: &Reference, maximum: u64) -> Result<()> {
    ensure!(
        digest_valid(&reference.sha256) && reference.bytes <= maximum,
        "production preparation reference exceeds its bound"
    );
    retention::authenticate_reference(reference, maximum)
}

fn name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
}

fn exact_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn protect(path: &Path) -> Result<()> {
    tool_execution::bounded::safe(path)?;
    ensure!(
        path.components().collect::<PathBuf>().as_os_str() == path.as_os_str(),
        "production preparation path alias"
    );
    let metadata = path.symlink_metadata()?;
    ensure!(
        metadata.uid() == unsafe { libc::geteuid() } && metadata.mode() & 0o022 == 0,
        "production preparation path is foreign or writable by others"
    );
    Ok(())
}

fn directory(path: &Path) -> Result<()> {
    tool_execution::bounded::safe(path)?;
    fs::DirBuilder::new().recursive(true).mode(0o700).create(path)?;
    protect(path)
}

fn lock(root: &Path) -> Result<File> {
    directory(root)?;
    let path = root.join("preparation.lock");
    tool_execution::bounded::safe(&path)?;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    ensure!(file.metadata()?.is_file() && file.metadata()?.nlink() == 1, "production preparation lease is aliased");
    ensure!(
        unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0,
        "production preparation already has an owner"
    );
    Ok(file)
}

fn write<T: Serialize>(path: &Path, value: &T) -> Result<Reference> {
    let bytes = serde_json::to_vec(value)?;
    ensure!(bytes.len() as u64 <= MAX_RECORD, "production preparation metadata extent exceeded");
    directory(path.parent().context("production preparation parent missing")?)?;
    tool_execution::bounded::immutable_bytes(path, &bytes)?;
    protect(path)?;
    retention::seal_file(path, MAX_RECORD)
}

pub(crate) fn install(reference: &Reference, path: &Path) -> Result<Reference> {
    authenticate(reference, MAX_TOOL)?;
    let metadata = reference.path.symlink_metadata()?;
    ensure!(metadata.is_file() && metadata.mode() & 0o111 != 0, "production tool is not executable");
    tool_execution::bounded::safe(path)?;
    if path.try_exists()? {
        protect(path)?;
        let saved = retention::seal_file(path, MAX_TOOL)?;
        ensure!(
            saved.sha256 == reference.sha256 && saved.bytes == reference.bytes,
            "installed production tool changed"
        );
        return Ok(saved);
    }
    let parent = path.parent().context("production tool parent missing")?;
    directory(parent)?;
    let pending = path.with_extension("pending");
    if pending.try_exists()? {
        protect(&pending)?;
        let metadata = pending.symlink_metadata()?;
        ensure!(
            metadata.is_file() && metadata.nlink() == 1 && metadata.len() <= reference.bytes,
            "unrecognized production tool intermediate"
        );
        fs::remove_file(&pending)?;
    }
    let mut input = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(&reference.path)?;
    let mut output = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o700)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&pending)?;
    let mut buffer = [0u8; 65536];
    let mut total = 0u64;
    loop {
        tool_execution::bounded::check()?;
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total = total.checked_add(count as u64).context("production tool extent overflow")?;
        ensure!(total <= reference.bytes, "production tool changed during installation");
        output.write_all(&buffer[..count])?;
    }
    output.sync_all()?;
    output.set_permissions(fs::Permissions::from_mode(0o500))?;
    let staged = retention::seal_file(&pending, MAX_TOOL)?;
    ensure!(total == reference.bytes && staged.sha256 == reference.sha256, "production tool installation seal differs");
    fs::rename(&pending, path)?;
    File::open(parent)?.sync_all()?;
    retention::seal_file(path, MAX_TOOL)
}

/// Read the main repository's loose/packed HEAD without starting a process or
/// acquiring any Git writer. Unsupported ref layouts refuse preparation.
fn current_head(checkout: &Path) -> Result<String> {
    let metadata = checkout.join(".git");
    protect(&metadata)?;
    ensure!(metadata.is_dir(), "production preparation requires the main repository Git directory");
    let head = String::from_utf8(tool_execution::bounded::read_limit(&metadata.join("HEAD"), 4096)?)?;
    let head = head.trim();
    if exact_commit(head) {
        return Ok(head.into());
    }
    let branch = head.strip_prefix("ref: ").context("unsupported production HEAD layout")?;
    ensure!(
        branch.starts_with("refs/heads/")
            && branch.len() <= 256
            && Path::new(branch).components().all(|part| matches!(part, Component::Normal(_))),
        "production HEAD branch alias"
    );
    let loose = metadata.join(branch);
    let value = if loose.try_exists()? {
        String::from_utf8(tool_execution::bounded::read_limit(&loose, 4096)?)?.trim().to_owned()
    } else {
        let packed =
            String::from_utf8(tool_execution::bounded::read_limit(&metadata.join("packed-refs"), MAX_RECORD)?)?;
        let matches = packed
            .lines()
            .filter_map(|line| line.split_once(' '))
            .filter(|(_, name)| *name == branch)
            .map(|(hash, _)| hash.to_owned())
            .collect::<Vec<_>>();
        ensure!(matches.len() == 1, "production branch lacks one exact packed identity");
        matches[0].clone()
    };
    ensure!(exact_commit(&value), "production HEAD is not an exact commit");
    Ok(value)
}

fn canonical_identity(installed: &Installation, domain: &Migration) -> Result<String> {
    let mut canonical = installed.clone();
    canonical.policy.adapter_sha256 = "0".repeat(64);
    let Admission::Managed { migration, .. } = &mut canonical.admission else {
        anyhow::bail!("production installation lacks managed admission")
    };
    migration.sha256 = migration::domain_identity(domain)?;
    // The final asset path participates in the identity, while its circular
    // content seal is replaced by exactly the normalization used by the reader.
    sealed(&("installed-translation-supervisor-v1", canonical))
}

fn authenticate_receipt(config: &Config, intent: &Intent, receipt: &Receipt) -> Result<Reference> {
    ensure!(
        receipt.schema == "translation-production-preparation-receipt-v1"
            && receipt.request.path == config.output_root.join("production-request.json")
            && receipt.intent.path == config.output_root.join("intent.json"),
        "production preparation receipt differs"
    );
    authenticate(&receipt.intent, MAX_RECORD)?;
    ensure!(read::<Intent>(&receipt.intent.path)? == *intent, "production preparation intent changed");
    for reference in &receipt.outputs {
        ensure!(reference.path.starts_with(&config.output_root), "production preparation output escaped declared root");
        authenticate(
            reference,
            if reference.path.starts_with(config.output_root.join("bin")) { MAX_TOOL } else { MAX_RECORD },
        )?;
    }
    authenticate(&receipt.request, MAX_RECORD)?;
    let request: Request = read(&receipt.request.path)?;
    for reference in &request.installations {
        authenticate(reference, MAX_RECORD)?;
        read::<Installation>(&reference.path)?.validate()?;
    }
    let plan: Plan = read(&request.launch_plan.path)?;
    plan.validate(&request.launch_plan.path)?;
    Ok(receipt.request.clone())
}

pub(crate) fn prepare(path: &Path) -> Result<Reference> {
    let now = tool_execution::bounded::now_unix_ms()?;
    decomp_source::context::until(now + 120_000, 4 * tool_execution::bounded::DEFAULT_BYTES, || prepare_inner(path))
}

fn prepare_inner(path: &Path) -> Result<Reference> {
    let config: Config = read(path)?;
    ensure!(
        config.schema == "translation-production-prepare-v1"
            && !config.packages.is_empty()
            && config.packages.len() <= 32
            && !config.executable.is_empty()
            && config.executable.len() <= 32
            && config.blocked.len() <= 4096
            && exact_commit(&config.integration_baseline),
        "unsupported production preparation configuration"
    );
    tool_execution::bounded::safe(&config.output_root)?;
    protect(&config.pinned_source)?;
    ensure!(config.pinned_source.is_dir(), "production pinned source must be a real external directory");
    ensure!(
        config.output_root.is_absolute() && !path.starts_with(&config.output_root),
        "production preparation overlaps its configuration"
    );
    for reference in config.packages.iter().chain([&config.template, &config.helper]) {
        ensure!(!reference.path.starts_with(&config.output_root), "production preparation input overlaps output");
    }
    authenticate(&config.template, MAX_RECORD)?;
    let template: Installation = read(&config.template.path)?;
    template.policy.validate()?;
    ensure!(
        template.policy.adapter_sha256 == execution::installation_identity(&template)?
            && template.rewrite.provider.is_some()
            && !template.rewrite.recorded,
        "production template must preserve its authenticated model provider"
    );
    template.rewrite.provider.as_ref().unwrap().validate()?;
    let mut domains = Vec::new();
    let mut efforts = BTreeSet::new();
    for reference in &config.packages {
        authenticate(reference, MAX_RECORD)?;
        let domain = crate::translation_migration_guard::read(&reference.path)?;
        ensure!(
            domain.state == "prepared" && efforts.insert(domain.effort.clone()),
            "production requires distinct prepared package efforts"
        );
        ensure!(
            !config.output_root.starts_with(&domain.checkout) && !domain.checkout.starts_with(&config.output_root),
            "production output overlaps main source"
        );
        domains.push(domain);
    }
    let checkout = domains[0].checkout.clone();
    ensure!(
        !config.pinned_source.starts_with(&checkout)
            && !checkout.starts_with(&config.pinned_source)
            && !config.pinned_source.starts_with(&config.output_root)
            && !config.output_root.starts_with(&config.pinned_source),
        "production pinned source overlaps source or writable outputs"
    );
    let authority = retention::seal_file(&checkout.join(AUTHORITY), MAX_RECORD)?;
    ensure!(
        current_head(&checkout)? == config.integration_baseline,
        "production integration baseline differs from current main HEAD"
    );
    for domain in &domains {
        ensure!(
            domain.checkout == checkout && domain.authority == authority,
            "production package does not name the current shared authority"
        );
        authenticate(&domain.ledger, MAX_RECORD)?;
    }
    let mut selections = BTreeSet::new();
    for selected in &config.executable {
        ensure!(
            name(&selected.effort)
                && name(&selected.claim_id)
                && name(&format!("{}-{}", selected.effort, selected.claim_id))
                && selections.insert((selected.effort.clone(), selected.claim_id.clone())),
            "production executable selection is invalid or repeated"
        );
    }
    let mut blocked_selections = BTreeSet::new();
    for blocked in &config.blocked {
        ensure!(
            name(&blocked.effort)
                && name(&blocked.claim_id)
                && !blocked.reason.trim().is_empty()
                && blocked.reason.len() <= 4096
                && !selections.contains(&(blocked.effort.clone(), blocked.claim_id.clone()))
                && blocked_selections.insert((blocked.effort.clone(), blocked.claim_id.clone())),
            "production blocked selection is invalid or repeated"
        );
    }
    let controller = retention::seal_file(&std::env::current_exe()?, MAX_TOOL)?;
    let intent = Intent {
        schema: "translation-production-preparation-intent-v1".into(),
        config: retention::seal_file(path, MAX_RECORD)?,
        config_sha256: sealed(&config)?,
        controller_sha256: controller.sha256.clone(),
        controller_bytes: controller.bytes,
    };
    let _lease = lock(&config.output_root)?;
    let receipt_path = config.output_root.join("receipt.json");
    // Reconstruct every expected record on retry. A modified receipt cannot
    // authorize different policies merely by updating its own content hashes.
    let intent_reference = write(&config.output_root.join("intent.json"), &intent)?;
    let controller = install(&controller, &config.output_root.join("bin/sm-pipeline"))?;
    let helper = install(&config.helper, &config.output_root.join("bin/translation-sandbox"))?;
    let mut outputs = vec![controller.clone(), helper.clone()];
    let mut originals = BTreeMap::new();
    for domain in &mut domains {
        for migrated in &mut domain.claims {
            authenticate(&migrated.original_claim, MAX_RECORD)?;
            let original: OriginalClaim = read(&migrated.original_claim.path)?;
            let key = (domain.effort.clone(), migrated.claim.id.clone());
            ensure!(
                original.effort == domain.effort
                    && original.checkout == checkout
                    && original.source_commit == migrated.source_commit
                    && original.claim.id == migrated.claim.id,
                "production package changed original source or claim"
            );
            if selections.contains(&key) {
                ensure!(
                    original.state == assignment::State::Submitted && original.job.kind == assignment::Kind::Implement,
                    "production imported executor is not an original submission"
                );
                migrated.execution = Some(MigrationExecution::ImportedSubmission);
                migrated.installation = Some(
                    config.output_root.join("scopes").join(format!("{}-{}", key.0, key.1)).join("installation.json"),
                );
            }
            ensure!(originals.insert(key, original).is_none(), "production package repeats an original claim");
        }
        let selected = domain
            .claims
            .iter()
            .filter(|claim| claim.execution == Some(MigrationExecution::ImportedSubmission))
            .cloned()
            .collect::<Vec<_>>();
        for migrated in &mut domain.claims {
            let original = &originals[&(domain.effort.clone(), migrated.claim.id.clone())];
            if original.job.kind == assignment::Kind::Integrate {
                if let Some(owner) = selected.iter().find(|owner| {
                    original.job.inputs.get("submission-id") == Some(&owner.claim.id)
                        && original.job.inputs.get("submission-revision") == Some(&owner.claim.revision)
                        && original.job.inputs.get("submission-attempt") == Some(&owner.claim.attempt.to_string())
                }) {
                    ensure!(
                        !blocked_selections.contains(&(domain.effort.clone(), migrated.claim.id.clone())),
                        "held integration cannot also be independently blocked"
                    );
                    migrated.execution = Some(MigrationExecution::HeldIntegration { submission: owner.claim.clone() });
                    migrated.installation = None;
                }
            }
            ensure!(
                migrated.execution.is_some()
                    || blocked_selections.contains(&(domain.effort.clone(), migrated.claim.id.clone())),
                "production claim lacks an explicit executor, held lineage or blocked diagnosis"
            );
        }
    }
    ensure!(
        selections.iter().chain(&blocked_selections).all(|key| originals.contains_key(key)),
        "production selection is absent from original packages"
    );
    let mut executable = config.executable.clone();
    executable.sort_by_key(|item| {
        (if item.kind == LegacyKind::SharedGameHost { 0 } else { 1 }, item.effort.clone(), item.claim_id.clone())
    });
    let mut installed = Vec::new();
    for selected in &executable {
        let domain = domains.iter().find(|domain| domain.effort == selected.effort).unwrap();
        let migrated = domain.claims.iter().find(|claim| claim.claim.id == selected.claim_id).unwrap();
        let original = &originals[&(selected.effort.clone(), selected.claim_id.clone())];
        let id = format!("{}-{}", selected.effort, selected.claim_id);
        let scope_root = config.output_root.join("scopes").join(&id);
        let contract = LegacyContract {
            schema: "translation-legacy-contract-v1".into(),
            kind: selected.kind,
            original_claim: migrated.original_claim.clone(),
            submission: selected.submission.clone(),
            integration_baseline: config.integration_baseline.clone(),
            source_files: original.source_files.clone(),
            acceptance_inputs: selected.acceptance_inputs.clone(),
        };
        let prepared = legacy::prepare(&contract)?;
        outputs.push(write(&scope_root.join("contract.json"), &contract)?);
        outputs.push(write(&scope_root.join("prepared.json"), &prepared)?);
        outputs.push(write(&scope_root.join("readiness.json"), &legacy::readiness(&prepared))?);
        let mut installation = template.clone();
        installation.id = id;
        installation.controller = InstalledFile { path: controller.path.clone(), sha256: controller.sha256.clone() };
        installation.helper = InstalledFile { path: helper.path.clone(), sha256: helper.sha256.clone() };
        installation.rewrite.executable = installation.helper.clone();
        installation.source_repository = checkout.clone();
        installation.integration_repository = checkout.clone();
        installation.integration_baseline = config.integration_baseline.clone();
        installation.pinned_source = config.pinned_source.clone();
        installation.state_root = scope_root.join("state");
        installation.pipeline_root = scope_root.join("pipeline");
        installation.policy.claim = migrated.claim.clone();
        installation.policy.scope_id = migrated.scope_id.clone();
        installation.policy.baseline_commit = original.source_commit.clone();
        installation.policy.approved_files = original.source_files.clone();
        installation.policy.dependencies_sha256 = prepared.dependencies_sha256;
        installation.policy.verification_sha256 = prepared.verification_sha256;
        installation.policy.pinned_c_sha256 = sha256("legacy-no-actual-c-v1");
        installation.contract = Contract::Legacy { contract };
        installation.admission = Admission::Managed {
            checkout: checkout.clone(),
            effort: domain.effort.clone(),
            coordinator: domain.coordinator.clone(),
            session_id: domain.session_id.clone(),
            generation: domain.generation,
            migration: offline_verification::fixtures::Asset {
                file: config.output_root.join("migrations").join(&domain.effort).join("active.json"),
                sha256: "0".repeat(64),
            },
        };
        installation.policy.adapter_sha256 = canonical_identity(&installation, domain)?;
        installed.push((domain.effort.clone(), selected.claim_id.clone(), installation));
    }
    for (effort, claim_id, installation) in &installed {
        let domain = domains.iter_mut().find(|domain| &domain.effort == effort).unwrap();
        domain.claims.iter_mut().find(|claim| &claim.claim.id == claim_id).unwrap().policy_sha256 =
            sealed(&installation.policy)?;
    }
    let mut deployments = Vec::new();
    let mut active_references = BTreeMap::new();
    for domain in &mut domains {
        let root = config.output_root.join("migrations").join(&domain.effort);
        domain.state = "prepared".into();
        let prepared = write(&root.join("prepared.json"), domain)?;
        domain.state = "active".into();
        let active = write(&root.join("active.json"), domain)?;
        outputs.extend([prepared.clone(), active.clone()]);
        active_references.insert(domain.effort.clone(), active.clone());
        deployments.push(Deployment { prepared, active });
    }
    let mut scopes: Vec<Scope> = Vec::new();
    let mut installation_references = Vec::new();
    for (effort, _, mut installation) in installed {
        let Admission::Managed { migration, .. } = &mut installation.admission else { unreachable!() };
        migration.sha256 = active_references[&effort].sha256.clone();
        ensure!(
            execution::installation_identity(&installation)? == installation.policy.adapter_sha256,
            "production installation identity changed after final migration sealing"
        );
        installation.validate()?;
        let reference =
            write(&config.output_root.join("scopes").join(&installation.id).join("installation.json"), &installation)?;
        let after = scopes.last().map(|scope| vec![scope.id.clone()]).unwrap_or_default();
        scopes.push(Scope { id: installation.id, after, installation: reference.clone() });
        installation_references.push(reference.clone());
        outputs.push(reference);
    }
    let plan = Plan {
        schema: "translation-launch-plan-v1".into(),
        id: "deterministic-production".into(),
        state_root: config.output_root.join("launch-state"),
        scopes,
    };
    let launch_plan = write(&config.output_root.join("plan.json"), &plan)?;
    plan.validate(&launch_plan.path)?;
    outputs.push(launch_plan.clone());
    let blocked = config
        .blocked
        .iter()
        .map(|blocked| {
            let domain = domains.iter().find(|domain| domain.effort == blocked.effort).unwrap();
            let claim = domain.claims.iter().find(|claim| claim.claim.id == blocked.claim_id).unwrap();
            BlockedClaim {
                effort: blocked.effort.clone(),
                claim: claim.claim.clone(),
                original_claim: claim.original_claim.clone(),
                reason: blocked.reason.clone(),
            }
        })
        .collect();
    ensure!(
        current_head(&checkout)? == config.integration_baseline
            && retention::seal_file(&checkout.join(AUTHORITY), MAX_RECORD)? == authority,
        "production source or authority changed during preparation"
    );
    let request = Request {
        schema: "translation-production-request-v1".into(),
        checkout,
        authority,
        migrations: deployments,
        installations: installation_references,
        launch_plan,
        blocked,
    };
    let request = write(&config.output_root.join("production-request.json"), &request)?;
    outputs.push(request.clone());
    let receipt = Receipt {
        schema: "translation-production-preparation-receipt-v1".into(),
        intent: intent_reference,
        request,
        outputs,
    };
    write(&receipt_path, &receipt)?;
    authenticate_receipt(&config, &intent, &receipt)
}
