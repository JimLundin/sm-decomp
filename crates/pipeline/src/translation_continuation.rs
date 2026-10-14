//! Reviewed original-contract installation and append-only successor preparation.
//! Each successor is frozen only after its predecessor's source/queue publication
//! has authenticated. The original production registry and stopped run remain
//! immutable dependencies of this separately reviewed installation.
use crate::{
    assignment,
    inputs::bounded,
    translation_execution::{self as execution, Admission, Contract, Installation},
    translation_migration::{self as migration, Migration, MigrationExecution},
    translation_original as original, translation_production as production,
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::{
        fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
        io::AsRawFd,
    },
    path::{Path, PathBuf},
};
use translation_execution::{
    checkout::ControllerProjection,
    model::{State, View, sealed, sha256},
    retention::{self, Reference},
};

pub(crate) const MARKER: &str = ".scratch/translation-execution/original-production.json";
const MAX_RECORD: u64 = 4 * 1024 * 1024;
const MAX_PRESERVED: u64 = 1024 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Scope {
    pub id: String,
    pub migration: Reference,
    pub original_claim: Reference,
    pub contract: Reference,
    pub prepared: Reference,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    pub schema: String,
    pub checkout: PathBuf,
    pub registry: Reference,
    pub output_root: PathBuf,
    pub template: Reference,
    pub reviewed_head: String,
    pub preserved: Vec<Reference>,
    /// Authentication of this stopped result is a prerequisite for installing
    /// changed acceptance. It is never retried or rewritten by this command.
    pub stopped_installation: Reference,
    pub stopped_view: View,
    pub scopes: Vec<Scope>,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Activation {
    schema: String,
    registry: Reference,
    config: Reference,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Boundary {
    pub head: String,
    /// Copies at immutable paths, keyed by the actual original effort.
    pub ledgers: BTreeMap<String, Reference>,
    /// Exact tracked modifications and untracked retained files. The key is
    /// their original host path; the reference is an immutable content copy.
    pub files: BTreeMap<PathBuf, Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Stage {
    pub schema: String,
    pub config: Reference,
    pub index: usize,
    pub predecessor: Option<Reference>,
    pub before: Boundary,
    pub migration: Reference,
    pub installation: Reference,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Terminal {
    pub schema: String,
    pub stage: Reference,
    pub view: View,
    pub accepted: Reference,
    pub after: Boundary,
    pub controller: ControllerProjection,
}

#[derive(Serialize)]
pub(crate) struct Report {
    pub schema: String,
    pub state: String,
    pub completed: Vec<String>,
    pub current: Option<String>,
    pub view: Option<View>,
    pub config: Reference,
}

fn name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
}

fn read<T: for<'a> Deserialize<'a>>(reference: &Reference) -> Result<T> {
    original::read(reference)
}
fn object<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T> {
    read(&retention::seal_file(path, MAX_RECORD)?)
}
fn write<T: Serialize>(path: &Path, value: &T) -> Result<Reference> {
    original::write(path, value)
}

fn stage_root(config: &Config, index: usize) -> PathBuf {
    config.output_root.join("scopes").join(&config.scopes[index].id)
}
fn stage_path(config: &Config, index: usize) -> PathBuf {
    stage_root(config, index).join("transition.json")
}
fn terminal_path(config: &Config, index: usize) -> PathBuf {
    stage_root(config, index).join("terminal.json")
}

pub(crate) fn lease(root: &Path) -> Result<File> {
    bounded::safe(root)?;
    fs::create_dir_all(root)?;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(root.join("continuation.lock"))?;
    let metadata = file.metadata()?;
    ensure!(
        metadata.is_file()
            && metadata.nlink() == 1
            && metadata.uid() == unsafe { libc::geteuid() }
            && metadata.mode() & 0o022 == 0,
        "original continuation lease is aliased or writable by others"
    );
    ensure!(
        unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0,
        "original continuation already has an active controller"
    );
    Ok(file)
}

fn activated(checkout: &Path) -> Result<Option<(Activation, Config)>> {
    let path = checkout.join(MARKER);
    if !path.try_exists()? {
        return Ok(None);
    }
    let activation: Activation = object(&path)?;
    ensure!(activation.schema == "translation-original-production-v1", "unsupported original production activation");
    retention::authenticate_reference(&activation.registry, MAX_RECORD)?;
    ensure!(
        activation.registry.path == checkout.join(production::REGISTRY),
        "original continuation replaced its ownership registry"
    );
    let config: Config = read(&activation.config)?;
    ensure!(
        config.checkout == checkout && config.registry == activation.registry,
        "original continuation activation changed"
    );
    validate_config(&config)?;
    Ok(Some((activation, config)))
}

fn validate_config(config: &Config) -> Result<()> {
    bounded::safe(&config.checkout)?;
    bounded::safe(&config.output_root)?;
    ensure!(
        config.schema == "translation-original-production-config-v1"
            && config.checkout.is_absolute()
            && config.output_root.is_absolute()
            && !config.output_root.starts_with(&config.checkout)
            && !config.checkout.starts_with(&config.output_root)
            && config.scopes.len() == 3
            && !config.reason.trim().is_empty()
            && config.reason.len() <= 4096
            && config.reviewed_head.len() == 40
            && config.reviewed_head.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "original production review, order or resource roots are invalid"
    );
    let registry: production::Registry = read(&config.registry)?;
    ensure!(
        registry.checkout == config.checkout && config.registry.path == config.checkout.join(production::REGISTRY),
        "original production registry identity changed"
    );
    let template: Installation = read(&config.template)?;
    ensure!(
        template.source_repository == config.checkout && template.integration_repository == config.checkout,
        "original production template names another controller or source authority"
    );
    let mut names = BTreeSet::new();
    for (index, scope) in config.scopes.iter().enumerate() {
        ensure!(name(&scope.id) && names.insert(&scope.id), "original production repeats a scope");
        ensure!(
            registry.migrations.contains(&scope.migration),
            "original production scope lost original migration membership"
        );
        let domain: Migration = read(&scope.migration)?;
        let contract: original::OriginalContract = read(&scope.contract)?;
        let prepared: original::PreparedOriginal = read(&scope.prepared)?;
        ensure!(
            contract.kind
                == [
                    original::OriginalKind::SharedGameHost,
                    original::OriginalKind::CompactController,
                    original::OriginalKind::PauseEquipment
                ][index]
                && scope.original_claim == contract.original_claim
                && sealed(&prepared.contract)? == sealed(&contract)?
                && domain.claims.iter().any(|claim| claim.original_claim == scope.original_claim),
            "original production changed required ordering or original claim lineage"
        );
    }
    ensure!(config.preserved.len() <= 4096, "reviewed preserved metadata inventory exceeded");
    ensure!(
        matches!(config.stopped_view.state, State::Stopped { .. }),
        "changed contracts require a reviewed terminal stop"
    );
    Ok(())
}

pub(crate) fn paths(checkout: &Path) -> Result<BTreeSet<PathBuf>> {
    let mut paths = BTreeSet::new();
    for argv in [
        vec!["diff", "--name-only", "--no-renames", "-z", "--no-ext-diff", "HEAD", "--"],
        vec!["ls-files", "--others", "--exclude-standard", "-z"],
    ] {
        let bytes = translation_execution::checkout::read_only_paths_until(
            checkout,
            &argv,
            MAX_RECORD,
            bounded::now_unix_ms()?.saturating_add(120_000),
        )?;
        for name in bytes.split(|byte| *byte == 0).filter(|name| !name.is_empty()) {
            let name = std::str::from_utf8(name)?;
            ensure!(
                Path::new(name).starts_with(".scratch")
                    && !Path::new(name).is_absolute()
                    && name.split('/').all(|part| !part.is_empty() && part != "." && part != ".."),
                "continuation found an unreviewed source modification: {name}"
            );
            let path = checkout.join(name);
            bounded::safe(&path)?;
            ensure!(path.is_file(), "continuation cannot preserve a deleted or nonregular input");
            paths.insert(path);
        }
    }
    ensure!(paths.len() <= 4096, "continuation host metadata inventory exceeded");
    ensure!(
        migration::git(checkout, &["diff", "--cached", "--name-only"])?.is_empty(),
        "continuation cannot consume a user's staged changes"
    );
    Ok(paths)
}

pub(crate) fn copy(path: &Path, output: &Path, maximum: u64) -> Result<Reference> {
    let source = retention::seal_file(path, maximum)?;
    bounded::safe(output)?;
    fs::create_dir_all(output.parent().context("boundary copy parent missing")?)?;
    let expected = Reference { path: output.into(), ..source.clone() };
    if output.exists() {
        retention::authenticate_reference(&expected, maximum)?;
        return Ok(expected);
    }
    let pending = output.with_extension("pending");
    if pending.exists() {
        let metadata = pending.symlink_metadata()?;
        ensure!(
            metadata.is_file() && metadata.nlink() == 1 && metadata.len() <= source.bytes,
            "boundary copy has an unrecognized partial file"
        );
        fs::remove_file(&pending)?;
    }
    let mut input = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path)?;
    let mut destination =
        OpenOptions::new().write(true).create_new(true).mode(0o600).custom_flags(libc::O_NOFOLLOW).open(&pending)?;
    let mut buffer = [0u8; 64 * 1024];
    let mut count = 0u64;
    loop {
        bounded::check()?;
        let n = input.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        count = count.checked_add(n as u64).context("boundary copy extent overflow")?;
        ensure!(count <= source.bytes, "boundary input grew while copying");
        bounded::account(n as u64)?;
        destination.write_all(&buffer[..n])?;
    }
    destination.sync_all()?;
    ensure!(count == source.bytes, "boundary input shrank while copying");
    retention::authenticate_reference(&Reference { path: pending.clone(), ..source.clone() }, maximum)?;
    retention::authenticate_reference(&source, maximum)?;
    destination.set_permissions(fs::Permissions::from_mode(0o400))?;
    fs::rename(&pending, output)?;
    File::open(output.parent().unwrap())?.sync_all()?;
    Ok(expected)
}

fn capture(config: &Config, output: &Path) -> Result<Boundary> {
    let head = migration::git(&config.checkout, &["rev-parse", "HEAD"])?;
    let mut ledgers = BTreeMap::new();
    let mut files = BTreeMap::new();
    let mut bytes = 0u64;
    for scope in &config.scopes {
        let domain: Migration = read(&scope.migration)?;
        if ledgers.contains_key(&domain.effort) {
            continue;
        }
        let reference = copy(
            &config.checkout.join(format!(".scratch/{}/ledger.toml", domain.effort)),
            &output.join("ledgers").join(format!("{}.toml", domain.effort)),
            MAX_RECORD,
        )?;
        ledgers.insert(domain.effort, reference);
    }
    for path in paths(&config.checkout)? {
        let reference = copy(&path, &output.join("files").join(path.strip_prefix(&config.checkout)?), MAX_PRESERVED)?;
        bytes = bytes.checked_add(reference.bytes).context("boundary extent overflow")?;
        ensure!(bytes <= MAX_PRESERVED, "boundary retained metadata exceeds its byte budget");
        files.insert(path, reference);
    }
    let boundary = Boundary { head, ledgers, files };
    authenticate_live(config, &boundary)?;
    Ok(boundary)
}

fn authenticate_boundary(boundary: &Boundary) -> Result<()> {
    ensure!(
        boundary.head.len() == 40 && boundary.files.len() <= 4096 && boundary.ledgers.len() <= 32,
        "continuation boundary inventory changed"
    );
    let mut total = 0u64;
    for reference in boundary.files.values().chain(boundary.ledgers.values()) {
        total = total.checked_add(reference.bytes).context("boundary extent overflow")?;
        ensure!(total <= MAX_PRESERVED + 32 * MAX_RECORD, "boundary extent exceeded");
        retention::authenticate_reference(reference, MAX_PRESERVED)?;
    }
    Ok(())
}

fn authenticate_live(config: &Config, boundary: &Boundary) -> Result<()> {
    authenticate_boundary(boundary)?;
    ensure!(
        migration::git(&config.checkout, &["rev-parse", "HEAD"])? == boundary.head
            && paths(&config.checkout)? == boundary.files.keys().cloned().collect(),
        "continuation host source or complete metadata inventory changed"
    );
    for (path, reference) in &boundary.files {
        retention::authenticate_reference(&Reference { path: path.clone(), ..reference.clone() }, MAX_PRESERVED)?;
    }
    for (effort, reference) in &boundary.ledgers {
        retention::authenticate_reference(
            &Reference { path: config.checkout.join(format!(".scratch/{effort}/ledger.toml")), ..reference.clone() },
            MAX_RECORD,
        )?;
    }
    Ok(())
}

fn authenticate_initial(config: &Config) -> Result<()> {
    ensure!(
        migration::git(&config.checkout, &["rev-parse", "HEAD"])? == config.reviewed_head,
        "first original contract no longer has the reviewed source baseline"
    );
    let mut expected = config
        .preserved
        .iter()
        .map(|reference| (reference.path.clone(), reference.clone()))
        .collect::<BTreeMap<_, _>>();
    let marker = config.checkout.join(MARKER);
    expected.insert(marker.clone(), retention::seal_file(&marker, MAX_RECORD)?);
    ensure!(
        paths(&config.checkout)? == expected.keys().cloned().collect(),
        "first original contract has unreviewed host metadata"
    );
    for reference in expected.values() {
        retention::authenticate_reference(reference, MAX_PRESERVED)?;
    }
    Ok(())
}

pub(crate) fn install(path: &Path) -> Result<Reference> {
    let reference = retention::seal_file(path, MAX_RECORD)?;
    let config: Config = read(&reference)?;
    validate_config(&config)?;
    let _lease = lease(&config.output_root)?;
    let expected = Activation {
        schema: "translation-original-production-v1".into(),
        registry: config.registry.clone(),
        config: reference,
    };
    let marker = config.checkout.join(MARKER);
    if marker.exists() {
        ensure!(object::<Activation>(&marker)? == expected, "another reviewed original installation is already active");
        return retention::seal_file(&marker, MAX_RECORD);
    }
    // The old executable authenticates its own identity and stopped journal.
    // A new controller never pretends to be the reader of the old stop.
    let stopped: Installation = read(&config.stopped_installation)?;
    let observed = inspect_installed(&config.stopped_installation, &stopped)?;
    ensure!(observed == config.stopped_view, "reviewed original prerequisite stop changed");
    let _lock = assignment::mutation_lock(&config.checkout)?;
    retention::authenticate_reference(&config.registry, MAX_RECORD)?;
    ensure!(
        migration::git(&config.checkout, &["rev-parse", "HEAD"])? == config.reviewed_head,
        "reviewed original installation host baseline changed"
    );
    let preserved = config.preserved.iter().map(|reference| reference.path.clone()).collect::<BTreeSet<_>>();
    ensure!(
        preserved.len() == config.preserved.len() && paths(&config.checkout)? == preserved,
        "reviewed original installation preserved file inventory changed"
    );
    for retained in &config.preserved {
        retention::authenticate_reference(retained, MAX_PRESERVED)?;
    }
    write(&marker, &expected)
}

fn inspect_installed(reference: &Reference, installed: &Installation) -> Result<View> {
    // The controller is an exact retained trusted reader, not a candidate. Its
    // inspect route creates no attempt, gate, model request or queue update.
    retention::authenticate_reference(
        &Reference {
            path: installed.controller.path.clone(),
            sha256: installed.controller.sha256.clone(),
            bytes: fs::metadata(&installed.controller.path)?.len(),
        },
        512 * 1024 * 1024,
    )?;
    let value =
        crate::translation_migration::controller_inspect(&installed.controller.path, &reference.path, installed)?;
    Ok(value)
}

fn prepare_stage(
    config_reference: &Reference,
    config: &Config,
    index: usize,
    predecessor: Option<&Reference>,
) -> Result<Stage> {
    let path = stage_path(config, index);
    if path.exists() {
        let stage: Stage = object(&path)?;
        ensure!(
            stage.config == *config_reference && stage.index == index && stage.predecessor.as_ref() == predecessor,
            "successor retry changed its exact predecessor"
        );
        authenticate_stage(config, &stage)?;
        return Ok(stage);
    }
    let _lock = assignment::mutation_lock(&config.checkout)?;
    let before = if let Some(reference) = predecessor {
        let terminal: Terminal = read(reference)?;
        authenticate_terminal(config, &terminal)?;
        authenticate_live(config, &terminal.after)?;
        terminal.after
    } else {
        ensure!(index == 0, "original successor has no completed predecessor");
        authenticate_initial(config)?;
        capture(config, &stage_root(config, index).join("before"))?
    };
    let root = stage_root(config, index);
    let scope = &config.scopes[index];
    let mut domain: Migration = read(&scope.migration)?;
    let contract: original::OriginalContract = read(&scope.contract)?;
    let prepared: original::PreparedOriginal = read(&scope.prepared)?;
    ensure!(sealed(&prepared.contract)? == sealed(&contract)?, "prepared successor contract changed");
    let current = domain
        .claims
        .iter()
        .find(|claim| claim.original_claim == scope.original_claim)
        .context("original claim not in migration")?
        .claim
        .clone();
    domain.claims.retain(|claim| claim.claim == current || matches!(&claim.execution, Some(MigrationExecution::HeldIntegration { submission }) if *submission == current));
    domain.ledger = Reference {
        path: config.checkout.join(format!(".scratch/{}/ledger.toml", domain.effort)),
        ..before.ledgers[&domain.effort].clone()
    };
    let mut installed: Installation = read(&config.template)?;
    installed.id = scope.id.clone();
    installed.state_root = root.join("state");
    installed.pipeline_root = root.join("pipeline");
    installed.integration_baseline = before.head.clone();
    installed.policy.scope_id = format!("original-acceptance-{}", scope.id);
    installed.policy.claim = current.clone();
    installed.policy.baseline_commit = prepared.original.source_commit.clone();
    installed.policy.approved_files = prepared.original.source_files.clone();
    installed.policy.dependencies_sha256 = prepared.dependencies_sha256.clone();
    installed.policy.verification_sha256 = prepared.verification_sha256.clone();
    installed.policy.pinned_c_sha256 = prepared.pinned_c_sha256.clone();
    installed.policy.verification_stages = contract.kind.stages();
    installed.contract = Contract::Original { contract };
    installed.admission = Admission::Managed {
        checkout: config.checkout.clone(),
        effort: domain.effort.clone(),
        coordinator: domain.coordinator.clone(),
        session_id: domain.session_id.clone(),
        generation: domain.generation,
        migration: crate::family_model::Asset { file: root.join("migration.json"), sha256: "0".repeat(64) },
    };
    let installation_path = root.join("installation.json");
    for claim in &mut domain.claims {
        if claim.claim == current {
            claim.scope_id = installed.policy.scope_id.clone();
            claim.installation = Some(installation_path.clone());
            claim.execution = Some(MigrationExecution::ImportedSubmission);
        }
    }
    installed.policy.adapter_sha256 = execution::installation_identity_in_domain(&installed, &domain)?;
    let policy_hash = sealed(&installed.policy)?;
    for claim in &mut domain.claims {
        if claim.claim == current {
            claim.policy_sha256 = policy_hash.clone();
        }
    }
    let migration = write(&root.join("migration.json"), &domain)?;
    let Admission::Managed { migration: asset, .. } = &mut installed.admission else { unreachable!() };
    asset.sha256 = migration.sha256.clone();
    installed.validate()?;
    let installation = write(&installation_path, &installed)?;
    let stage = Stage {
        schema: "translation-original-successor-v1".into(),
        config: config_reference.clone(),
        index,
        predecessor: predecessor.cloned(),
        before,
        migration,
        installation,
    };
    authenticate_stage(config, &stage)?;
    write(&path, &stage)?;
    Ok(stage)
}

fn authenticate_stage(config: &Config, stage: &Stage) -> Result<()> {
    ensure!(
        stage.schema == "translation-original-successor-v1" && stage.index < config.scopes.len(),
        "original successor identity invalid"
    );
    authenticate_boundary(&stage.before)?;
    let scope = &config.scopes[stage.index];
    let installed: Installation = read(&stage.installation)?;
    let domain: Migration = read(&stage.migration)?;
    let original: Migration = read(&scope.migration)?;
    let template: Installation = read(&config.template)?;
    let before_ledger = stage.before.ledgers.get(&domain.effort).context("successor omitted its original ledger")?;
    ensure!(
        stage.migration.path == stage_root(config, stage.index).join("migration.json")
            && stage.installation.path == stage_root(config, stage.index).join("installation.json")
            && stage.config.path.is_absolute()
            && sealed(&read::<Config>(&stage.config)?)? == sealed(config)?
            && installed.id == scope.id
            && installed.integration_baseline == stage.before.head
            && installed.source_repository == config.checkout
            && installed.integration_repository == config.checkout
            && installed.controller.path == template.controller.path
            && installed.controller.sha256 == template.controller.sha256
            && installed.state_root == stage_root(config, stage.index).join("state")
            && installed.pipeline_root == stage_root(config, stage.index).join("pipeline")
            && domain.checkout == config.checkout
            && domain.authority == original.authority
            && domain.session_id == original.session_id
            && domain.generation == original.generation
            && domain.coordinator == original.coordinator
            && domain.effort == original.effort
            && domain.ledger.sha256 == before_ledger.sha256
            && domain.ledger.bytes == before_ledger.bytes
            && domain.historical == original.historical
            && domain.stopped_owners == original.stopped_owners
            && domain.proof == original.proof
            && domain.limitations == original.limitations,
        "successor changed original authority, source boundary or retained evidence"
    );
    let Contract::Original { contract } = &installed.contract else {
        anyhow::bail!("successor lost original composite acceptance")
    };
    ensure!(
        sealed(contract)? == sealed(&read::<original::OriginalContract>(&scope.contract)?)?
            && contract.original_claim == scope.original_claim,
        "successor changed its reviewed original contract"
    );
    let retained = original
        .claims
        .iter()
        .find(|claim| claim.original_claim == scope.original_claim)
        .context("successor original lineage missing")?;
    ensure!(
        installed.policy.claim == retained.claim
            && installed.policy.baseline_commit == retained.source_commit
            && installed.policy.approved_files.keys().map(PathBuf::from).collect::<BTreeSet<_>>()
                == retained.writable_files.iter().cloned().collect(),
        "successor changed original submitted source or ownership"
    );
    let expected = original.claims.iter().filter(|claim| claim.claim == retained.claim || matches!(&claim.execution, Some(MigrationExecution::HeldIntegration { submission }) if *submission == retained.claim)).collect::<Vec<_>>();
    ensure!(domain.claims.len() == expected.len(), "successor replaced original held ownership");
    for (claim, prior) in domain.claims.iter().zip(expected) {
        ensure!(
            claim.claim == prior.claim
                && claim.original_claim == prior.original_claim
                && claim.source_commit == prior.source_commit
                && claim.writable_files == prior.writable_files
                && (claim.claim == installed.policy.claim || sealed(claim)? == sealed(prior)?),
            "successor changed an original claim, result or held integration"
        );
    }
    let prepared: original::PreparedOriginal = read(&scope.prepared)?;
    let mut expected = template;
    expected.id = scope.id.clone();
    expected.state_root = stage_root(config, stage.index).join("state");
    expected.pipeline_root = stage_root(config, stage.index).join("pipeline");
    expected.integration_baseline = stage.before.head.clone();
    expected.policy.scope_id = format!("original-acceptance-{}", scope.id);
    expected.policy.claim = retained.claim.clone();
    expected.policy.baseline_commit = prepared.original.source_commit.clone();
    expected.policy.approved_files = prepared.original.source_files.clone();
    expected.policy.dependencies_sha256 = prepared.dependencies_sha256;
    expected.policy.verification_sha256 = prepared.verification_sha256;
    expected.policy.pinned_c_sha256 = prepared.pinned_c_sha256;
    expected.policy.verification_stages = contract.kind.stages();
    expected.contract = Contract::Original { contract: contract.clone() };
    expected.admission = Admission::Managed {
        checkout: config.checkout.clone(),
        effort: domain.effort.clone(),
        coordinator: domain.coordinator.clone(),
        session_id: domain.session_id.clone(),
        generation: domain.generation,
        migration: crate::family_model::Asset {
            file: stage.migration.path.clone(),
            sha256: stage.migration.sha256.clone(),
        },
    };
    expected.policy.adapter_sha256 = execution::installation_identity_in_domain(&expected, &domain)?;
    ensure!(
        sealed(&expected)? == sealed(&installed)?,
        "successor changed its reviewed runtime, limits or frozen policy"
    );
    ensure!(
        installed.policy.adapter_sha256 == execution::installation_identity(&installed)?,
        "successor installation was rebased or changed"
    );
    if stage.index == 0 {
        ensure!(
            stage.predecessor.is_none() && stage.before.head == config.reviewed_head,
            "first original source boundary changed"
        );
        let mut expected = config
            .preserved
            .iter()
            .map(|reference| (reference.path.clone(), reference.clone()))
            .collect::<BTreeMap<_, _>>();
        let marker = config.checkout.join(MARKER);
        expected.insert(marker.clone(), retention::seal_file(&marker, MAX_RECORD)?);
        ensure!(
            stage.before.files.keys().eq(expected.keys())
                && stage.before.files.iter().all(|(path, reference)| {
                    reference.sha256 == expected[path].sha256 && reference.bytes == expected[path].bytes
                }),
            "first successor changed the reviewed preserved metadata boundary"
        );
        let mut ledgers = BTreeMap::new();
        for scope in &config.scopes {
            let original: Migration = read(&scope.migration)?;
            ledgers.insert(original.effort, original.ledger);
        }
        ensure!(
            stage.before.ledgers.keys().eq(ledgers.keys())
                && stage.before.ledgers.iter().all(|(effort, reference)| {
                    reference.sha256 == ledgers[effort].sha256 && reference.bytes == ledgers[effort].bytes
                }),
            "first successor changed an original frozen ledger"
        );
    } else {
        let previous = stage.predecessor.as_ref().context("successor was prepared without a completed predecessor")?;
        ensure!(previous.path == terminal_path(config, stage.index - 1), "successor skipped original production order");
        let terminal: Terminal = read(previous)?;
        ensure!(terminal.after == stage.before, "successor silently rebased its frozen ledger or source policy");
    }
    Ok(())
}

fn authenticate_terminal(config: &Config, terminal: &Terminal) -> Result<()> {
    ensure!(terminal.schema == "translation-original-terminal-v1", "original terminal schema changed");
    let stage: Stage = read(&terminal.stage)?;
    authenticate_stage(config, &stage)?;
    let State::Complete { completion, .. } = &terminal.view.state else {
        anyhow::bail!("original predecessor did not complete")
    };
    ensure!(
        terminal.after.head == completion.integrated.commit,
        "terminal original source differs from integrated evidence"
    );
    authenticate_boundary(&terminal.after)?;
    let installed: Installation = read(&stage.installation)?;
    let domain: Migration = read(&stage.migration)?;
    ensure!(
        stage.before.ledgers.keys().eq(terminal.after.ledgers.keys()),
        "terminal changed the original effort ledger inventory"
    );
    for (effort, reference) in &stage.before.ledgers {
        if effort != &domain.effort {
            let after = terminal.after.ledgers.get(effort).context("terminal dropped an original effort ledger")?;
            ensure!(
                reference.sha256 == after.sha256 && reference.bytes == after.bytes,
                "predecessor changed another original effort"
            );
        }
    }
    let controller = migration::authenticate_frozen_publication(
        &installed,
        &terminal.accepted,
        &terminal.after.ledgers[&domain.effort],
        &terminal.after.head,
    )?;
    ensure!(terminal.controller == controller, "terminal changed its sealed completed controller projection");
    let mutable = controller.paths.iter().collect::<BTreeSet<_>>();
    let projected =
        controller.references.iter().map(|reference| (&reference.path, reference)).collect::<BTreeMap<_, _>>();
    let ledger_path = config.checkout.join(format!(".scratch/{}/ledger.toml", domain.effort));
    let ledger = projected.get(&ledger_path).context("completed original ledger projection is missing")?;
    let after_ledger = &terminal.after.ledgers[&domain.effort];
    ensure!(
        ledger.sha256 == after_ledger.sha256 && ledger.bytes == after_ledger.bytes,
        "terminal ledger bytes differ from the accepted completed projection"
    );
    for (path, before) in &stage.before.files {
        if !mutable.contains(path) {
            let after = terminal.after.files.get(path).context("terminal dropped preserved host metadata")?;
            ensure!(
                before.sha256 == after.sha256 && before.bytes == after.bytes,
                "terminal changed preserved host metadata"
            );
        }
    }
    for (path, after) in &terminal.after.files {
        if mutable.contains(path) {
            let expected = projected.get(path).context("terminal added an unsealed controller projection")?;
            ensure!(
                expected.sha256 == after.sha256 && expected.bytes == after.bytes,
                "terminal controller output differs from its accepted publication"
            );
        } else {
            ensure!(stage.before.files.contains_key(path), "terminal admitted unrelated new host metadata");
        }
    }
    for reference in &controller.references {
        if terminal.after.files.contains_key(&reference.path) {
            continue;
        }
        let name = reference.path.strip_prefix(&config.checkout)?.to_str().context("controller path is not UTF8")?;
        let bytes = migration::git_bytes(&config.checkout, &["show", &format!("{}:{name}", terminal.after.head)])?;
        ensure!(
            reference.sha256 == sha256(&bytes) && reference.bytes == bytes.len() as u64,
            "terminal omitted changed controller metadata from its frozen boundary"
        );
    }
    execution::authenticate_managed_terminal(&installed, &terminal.view, &terminal.accepted)?;
    Ok(())
}

pub(crate) fn registered(installed: &Installation, path: &Path, sha: &str) -> Result<bool> {
    let Admission::Managed { checkout, .. } = &installed.admission else { return Ok(false) };
    let Some((activation, config)) = activated(checkout)? else { return Ok(false) };
    for index in 0..config.scopes.len() {
        let record = stage_path(&config, index);
        if !record.exists() {
            continue;
        }
        let stage: Stage = object(&record)?;
        if stage.migration.path != path || stage.migration.sha256 != sha {
            continue;
        }
        ensure!(stage.config == activation.config, "successor belongs to another reviewed installation");
        authenticate_stage(&config, &stage)?;
        ensure!(
            sealed(installed)? == sealed(&read::<Installation>(&stage.installation)?)?,
            "installed successor differs from its transition"
        );
        if let Some(previous) = &stage.predecessor {
            authenticate_terminal(&config, &read(previous)?)?;
        }
        return Ok(true);
    }
    Ok(false)
}

/// The static terminal snapshot authenticates history after later original jobs
/// advance the shared source and queue. It grants no fresh producer admission.
pub(crate) fn historical(installed: &Installation) -> Result<Option<Terminal>> {
    let Admission::Managed { checkout, .. } = &installed.admission else { return Ok(None) };
    let Some((_, config)) = activated(checkout)? else { return Ok(None) };
    let Some(index) = config.scopes.iter().position(|scope| scope.id == installed.id) else { return Ok(None) };
    let path = terminal_path(&config, index);
    if !path.exists() {
        return Ok(None);
    }
    let terminal: Terminal = object(&path)?;
    let stage: Stage = read(&terminal.stage)?;
    ensure!(
        sealed(installed)? == sealed(&read::<Installation>(&stage.installation)?)?,
        "historical original installation changed"
    );
    authenticate_terminal(&config, &terminal)?;
    Ok(Some(terminal))
}

pub(crate) fn preserved(installed: &Installation) -> Result<Vec<Reference>> {
    let Admission::Managed { checkout, .. } = &installed.admission else { return Ok(vec![]) };
    let Some((_, config)) = activated(checkout)? else { return Ok(vec![]) };
    let Some(index) = config.scopes.iter().position(|scope| scope.id == installed.id) else { return Ok(vec![]) };
    let path = stage_path(&config, index);
    if !path.exists() {
        return Ok(vec![]);
    }
    let stage: Stage = object(&path)?;
    ensure!(
        sealed(installed)? == sealed(&read::<Installation>(&stage.installation)?)?,
        "preserved original source belongs to another installation"
    );
    authenticate_boundary(&stage.before)?;
    Ok(stage.before.files.into_iter().map(|(path, reference)| Reference { path, ..reference }).collect())
}

pub(crate) fn run(checkout: &Path, action: &str) -> Result<Option<Report>> {
    let Some((activation, config)) = activated(checkout)? else { return Ok(None) };
    ensure!(matches!(action, "run" | "status" | "recover"), "unsupported original continuation action");
    let _lease = (action != "status").then(|| lease(&config.output_root)).transpose()?;
    let mut report = Report {
        schema: "translation-original-production-report-v1".into(),
        state: "ready".into(),
        completed: vec![],
        current: None,
        view: None,
        config: activation.config.clone(),
    };
    let mut predecessor = None;
    for index in 0..config.scopes.len() {
        let terminal = terminal_path(&config, index);
        if terminal.exists() {
            authenticate_terminal(&config, &object(&terminal)?)?;
            predecessor = Some(retention::seal_file(&terminal, MAX_RECORD)?);
            report.completed.push(config.scopes[index].id.clone());
            continue;
        }
        report.current = Some(config.scopes[index].id.clone());
        if action == "status" && !stage_path(&config, index).exists() {
            if let Some(reference) = &predecessor {
                let terminal: Terminal = read(reference)?;
                authenticate_live(&config, &terminal.after)?;
            } else {
                authenticate_initial(&config)?;
            }
            return Ok(Some(report));
        }
        let stage = prepare_stage(&activation.config, &config, index, predecessor.as_ref())?;
        let installed: Installation = read(&stage.installation)?;
        let run_id = sha256(format!("translation-execution-run-v1:{}", sealed(&installed.policy)?));
        if action == "status" && !installed.state_root.join(run_id).join("journal").exists() {
            authenticate_live(&config, &stage.before)?;
            return Ok(Some(report));
        }
        let view = execution::execute(installed.clone(), if action == "status" { "inspect" } else { "run" }, None)?;
        report.state = match view.state {
            State::Complete { .. } => "complete",
            State::Stopped { .. } => "stopped",
            _ => "pending",
        }
        .into();
        report.view = Some(view.clone());
        if action == "status" || !matches!(view.state, State::Complete { .. }) {
            return Ok(Some(report));
        }
        let State::Complete { completion, .. } = &view.state else { unreachable!() };
        let (accepted, controller) = migration::capture_publication(&installed, completion)?;
        let _lock = assignment::mutation_lock(checkout)?;
        let after = capture(&config, &stage_root(&config, index).join("after"))?;
        let terminal = Terminal {
            schema: "translation-original-terminal-v1".into(),
            stage: retention::seal_file(&stage_path(&config, index), MAX_RECORD)?,
            view,
            accepted,
            after,
            controller,
        };
        authenticate_terminal(&config, &terminal)?;
        predecessor = Some(write(&terminal_path(&config, index), &terminal)?);
        report.completed.push(config.scopes[index].id.clone());
    }
    if let Some(reference) = predecessor {
        let terminal: Terminal = read(&reference)?;
        authenticate_live(&config, &terminal.after)?;
    }
    report.state = "complete".into();
    report.current = None;
    Ok(Some(report))
}

/// The human-facing installed command delegates to the separately sealed new
/// controller. The original executable remains available to read its old stop.
pub(crate) fn launch(checkout: &Path, action: &str) -> Result<bool> {
    let Some((_, config)) = activated(checkout)? else { return Ok(false) };
    ensure!(matches!(action, "run" | "status" | "recover"), "unknown original production action");
    let template: Installation = read(&config.template)?;
    let domain: Migration = read(&config.scopes[0].migration)?;
    let reference = retention::seal_file(&template.controller.path, 512 * 1024 * 1024)?;
    ensure!(reference.sha256 == template.controller.sha256, "installed original controller changed");
    let status = std::process::Command::new(&template.controller.path)
        .args(["translation-original", "--checkout"])
        .arg(checkout)
        .arg(action)
        .env("CODEX_THREAD_ID", &domain.session_id)
        .current_dir(checkout)
        .status()?;
    ensure!(status.success(), "installed original controller exited with {status}");
    Ok(true)
}

#[cfg(test)]
#[path = "translation_continuation_tests.rs"]
mod tests;
