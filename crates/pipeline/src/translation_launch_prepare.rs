//! Install a new model-backed ordered launch from exact approved scope templates.
//! Source baselines, original claim lineage and verification contracts are copied
//! unchanged. Production migration remains subject to its managed admission.
use crate::translation_execution as execution;
use crate::translation_execution::Admission;
use crate::translation_execution::Installation;
use crate::translation_execution::InstalledFile;
use crate::translation_launch::Plan;
use crate::translation_launch::Scope;
use ::translation_execution::{
    model::{digest_valid, sealed},
    model_rewrite::Provider,
    retention::{self, Reference},
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::{
        fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
        io::AsRawFd,
    },
    path::{Path, PathBuf},
};

const MAX_RECORD: u64 = 4 * 1024 * 1024;
const MAX_TOOL: u64 = 512 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ScopeRequest {
    pub id: String,
    pub after: Vec<String>,
    pub template: Reference,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Request {
    pub schema: String,
    pub id: String,
    pub output_root: PathBuf,
    pub helper: Reference,
    pub provider: Provider,
    pub scopes: Vec<ScopeRequest>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Intent {
    schema: String,
    request: Reference,
    request_sha256: String,
    controller: Reference,
    helper: Reference,
    templates: Vec<Reference>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema: String,
    request_sha256: String,
    plan: Reference,
    controller: Reference,
    helper: Reference,
    installations: Vec<Reference>,
}

fn name(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.bytes().all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
}
fn read<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T> {
    ensure!(path.is_absolute(), "launch request path must be absolute");
    serde_json::from_slice(&tool_execution::bounded::read(path)?).context("invalid launch preparation JSON")
}
fn write<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    ensure!(bytes.len() as u64 <= MAX_RECORD, "launch preparation metadata exceeds bound");
    tool_execution::bounded::safe(path)?;
    fs::create_dir_all(path.parent().context("launch preparation record parent missing")?)?;
    tool_execution::bounded::immutable_bytes(path, &bytes)
}
fn protect(path: &Path) -> Result<()> {
    tool_execution::bounded::safe(path)?;
    let meta = path.symlink_metadata()?;
    ensure!(
        meta.uid() == unsafe { libc::geteuid() } && meta.mode() & 0o022 == 0,
        "launch installation has a foreign or mutable owner"
    );
    Ok(())
}
fn lock(root: &Path) -> Result<File> {
    tool_execution::bounded::safe(root)?;
    fs::create_dir_all(root)?;
    protect(root)?;
    let path = root.join("preparation.lock");
    tool_execution::bounded::safe(&path)?;
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&path)?;
    ensure!(file.metadata()?.is_file() && file.metadata()?.nlink() == 1, "launch preparation lock is aliased");
    ensure!(
        unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0,
        "launch preparation has another owner"
    );
    Ok(file)
}

fn install(reference: &Reference, path: &Path) -> Result<Reference> {
    retention::authenticate_reference(reference, MAX_TOOL)?;
    let source = reference.path.symlink_metadata()?;
    ensure!(source.is_file() && source.mode() & 0o111 != 0, "installed launcher tool is not executable");
    tool_execution::bounded::safe(path)?;
    if path.try_exists()? {
        protect(path)?;
        let saved = retention::seal_file(path, MAX_TOOL)?;
        ensure!(saved.sha256 == reference.sha256 && saved.bytes == reference.bytes, "installed launcher tool changed");
        return Ok(saved);
    }
    let parent = path.parent().context("installed launcher tool parent missing")?;
    fs::create_dir_all(parent)?;
    protect(parent)?;
    let pending = path.with_extension("pending");
    if pending.try_exists()? {
        tool_execution::bounded::safe(&pending)?;
        ensure!(
            pending.symlink_metadata()?.is_file()
                && pending.metadata()?.nlink() == 1
                && pending.metadata()?.len() <= reference.bytes,
            "unrecognized launcher tool pending extent"
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
    let mut buffer = [0u8; 64 * 1024];
    let mut total = 0u64;
    loop {
        tool_execution::bounded::check()?;
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total = total.checked_add(count as u64).context("launcher tool copy overflow")?;
        ensure!(total <= reference.bytes, "launcher tool changed while copying");
        output.write_all(&buffer[..count])?;
    }
    output.sync_all()?;
    output.set_permissions(fs::Permissions::from_mode(0o500))?;
    let staged = retention::seal_file(&pending, MAX_TOOL)?;
    ensure!(total == reference.bytes && staged.sha256 == reference.sha256, "launcher tool copy identity differs");
    fs::rename(pending, path)?;
    File::open(parent)?.sync_all()?;
    retention::seal_file(path, MAX_TOOL)
}

fn authenticate(request: &Request, request_reference: &Reference, receipt: &Receipt) -> Result<Reference> {
    ensure!(
        receipt.schema == "translation-launch-installation-receipt-v1"
            && receipt.request_sha256 == sealed(request)?
            && receipt.plan.path == request.output_root.join("plan.json")
            && receipt.controller.path == request.output_root.join("installed/controller")
            && receipt.helper.path == request.output_root.join("installed/helper")
            && receipt.helper.sha256 == request.helper.sha256
            && receipt.installations.len() == request.scopes.len(),
        "launch installation receipt changed"
    );
    let intent: Intent = read(&request.output_root.join("intent.json"))?;
    ensure!(
        intent.schema == "translation-launch-installation-intent-v1"
            && intent.request == *request_reference
            && intent.request_sha256 == receipt.request_sha256
            && intent.helper == request.helper
            && intent.templates == request.scopes.iter().map(|s| s.template.clone()).collect::<Vec<_>>()
            && intent.controller.sha256 == receipt.controller.sha256,
        "launch installation intent changed"
    );
    for reference in [request_reference, &receipt.plan, &receipt.controller, &receipt.helper] {
        retention::authenticate_reference(reference, MAX_TOOL)?;
    }
    let plan: Plan = read(&receipt.plan.path)?;
    ensure!(
        plan.schema == "translation-launch-plan-v1"
            && plan.id == request.id
            && plan.state_root == request.output_root.join("launch-state")
            && plan.scopes.len() == request.scopes.len(),
        "installed launch plan changed"
    );
    for ((scope, installed), original) in plan.scopes.iter().zip(&receipt.installations).zip(&request.scopes) {
        ensure!(
            scope.id == original.id && scope.after == original.after && scope.installation == *installed,
            "installed launch order changed"
        );
        retention::authenticate_reference(installed, MAX_RECORD)?;
        retention::authenticate_reference(&original.template, MAX_RECORD)?;
        let installation: Installation = read(&installed.path)?;
        let template: Installation = read(&original.template.path)?;
        let expected = configure(template, request, original, &receipt.controller, &receipt.helper)?;
        ensure!(
            sealed(&installation)? == sealed(&expected)?,
            "installed scope differs from original approved template"
        );
    }
    Ok(receipt.plan.clone())
}

fn configure(
    mut template: Installation,
    request: &Request,
    scope: &ScopeRequest,
    controller: &Reference,
    helper: &Reference,
) -> Result<Installation> {
    ensure!(
        matches!(template.admission, Admission::IsolatedProof { .. }),
        "managed scope installation requires the separate reviewed migration; supply its final --plan instead"
    );
    template.policy.validate()?;
    ensure!(
        template.policy.adapter_sha256 == execution::installation_identity(&template)?,
        "approved launch template identity changed"
    );
    ensure!(
        !request.output_root.starts_with(&template.source_repository)
            && !template.source_repository.starts_with(&request.output_root)
            && !request.output_root.starts_with(&template.integration_repository)
            && !template.integration_repository.starts_with(&request.output_root),
        "launch installation overlaps source repositories"
    );
    template.id = scope.id.clone();
    template.controller = InstalledFile { path: controller.path.clone(), sha256: controller.sha256.clone() };
    template.helper = InstalledFile { path: helper.path.clone(), sha256: helper.sha256.clone() };
    template.rewrite.executable = template.helper.clone();
    template.rewrite.recorded = false;
    template.rewrite.provider = Some(request.provider.clone());
    template.state_root = request.output_root.join("scopes").join(&scope.id).join("state");
    template.policy.adapter_sha256 = execution::installation_identity(&template)?;
    template.validate()?;
    Ok(template)
}

pub(crate) fn prepare(request_path: &Path, read_only: bool) -> Result<Reference> {
    let entered = tool_execution::bounded::now_unix_ms()?;
    decomp_source::context::until(entered + 120_000, 4 * tool_execution::bounded::DEFAULT_BYTES, || {
        prepare_bounded(request_path, read_only)
    })
}

fn prepare_bounded(request_path: &Path, read_only: bool) -> Result<Reference> {
    let request: Request = read(request_path)?;
    ensure!(
        request.schema == "translation-launch-request-v1"
            && name(&request.id)
            && !request.scopes.is_empty()
            && request.scopes.len() <= 32,
        "unsupported launch installation request"
    );
    tool_execution::bounded::safe(&request.output_root)?;
    ensure!(
        request.output_root.is_absolute() && !request_path.starts_with(&request.output_root),
        "launch installation overlaps original request"
    );
    request.provider.validate()?;
    ensure!(
        !request.helper.path.starts_with(&request.output_root)
            && request.helper.bytes <= MAX_TOOL
            && digest_valid(&request.helper.sha256),
        "launch helper input overlaps output"
    );
    let mut seen = BTreeSet::new();
    let mut templates = Vec::new();
    for scope in &request.scopes {
        ensure!(
            name(&scope.id)
                && seen.insert(scope.id.clone())
                && scope.after.iter().all(|id| id != &scope.id && seen.contains(id)),
            "scope order or dependencies invalid"
        );
        ensure!(!scope.template.path.starts_with(&request.output_root), "launch template overlaps output");
        retention::authenticate_reference(&scope.template, MAX_RECORD)?;
        templates.push(read::<Installation>(&scope.template.path)?);
    }
    let request_reference = retention::seal_file(request_path, MAX_RECORD)?;
    let receipt_path = request.output_root.join("receipt.json");
    if read_only {
        let receipt: Receipt = read(&receipt_path)?;
        return authenticate(&request, &request_reference, &receipt);
    }
    let _lease = lock(&request.output_root)?;
    if receipt_path.try_exists()? {
        let receipt: Receipt = read(&receipt_path)?;
        return authenticate(&request, &request_reference, &receipt);
    }
    let controller = retention::seal_file(&std::env::current_exe()?, MAX_TOOL)?;
    let intent = Intent {
        schema: "translation-launch-installation-intent-v1".into(),
        request: request_reference.clone(),
        request_sha256: sealed(&request)?,
        controller: controller.clone(),
        helper: request.helper.clone(),
        templates: request.scopes.iter().map(|s| s.template.clone()).collect(),
    };
    let destination = request.output_root.join("intent.json");
    // Exact intent publication refuses an interrupted installation resumed
    // with a different binary, plan, credential name or model configuration.
    write(&destination, &intent)?;
    let controller = install(&controller, &request.output_root.join("installed/controller"))?;
    let helper = install(&request.helper, &request.output_root.join("installed/helper"))?;
    let mut installations = Vec::new();
    let mut scopes = Vec::new();
    for (scope, template) in request.scopes.iter().zip(templates) {
        let installation = configure(template, &request, scope, &controller, &helper)?;
        let path = request.output_root.join("scopes").join(&scope.id).join("installation.json");
        write(&path, &installation)?;
        let reference = retention::seal_file(&path, MAX_RECORD)?;
        scopes.push(Scope { id: scope.id.clone(), after: scope.after.clone(), installation: reference.clone() });
        installations.push(reference);
    }
    let plan = Plan {
        schema: "translation-launch-plan-v1".into(),
        id: request.id.clone(),
        state_root: request.output_root.join("launch-state"),
        scopes,
    };
    write(&request.output_root.join("plan.json"), &plan)?;
    let receipt = Receipt {
        schema: "translation-launch-installation-receipt-v1".into(),
        request_sha256: intent.request_sha256,
        plan: retention::seal_file(&request.output_root.join("plan.json"), MAX_RECORD)?,
        controller,
        helper,
        installations,
    };
    write(&receipt_path, &receipt)?;
    authenticate(&request, &request_reference, &receipt)
}
