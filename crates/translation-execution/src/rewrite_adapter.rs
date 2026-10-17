//! The actual rewrite process receives only an immutable capsule on stdin.
//! Installed programs are sealed, run without host inputs, and return source JSON.
use crate::{
    confinement::{Program, RunLimits, helper_runtime},
    model::{
        ExecutionPhase, OperationContext, RewriteCapsule, RewriteLimits, SourceReplacements, digest_valid, sealed,
    },
    operation, rewrite,
};
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Component, Path, PathBuf},
};

const MAX_INSTALLED_FILE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_RECORDED_RESPONSE_BYTES: usize = 64 * 1024;

/// A trusted supervisor supplies the operation identity; the worker cannot
/// select commands, resource roots, credentials, stages, or evidence records.
pub trait RewriteAdapter {
    fn identity(&self) -> String;
    fn rewrite(
        &mut self,
        context: &OperationContext,
        capsule: &RewriteCapsule,
        limits: RewriteLimits,
    ) -> Result<SourceReplacements>;
}

fn unaliased(path: &Path) -> Result<()> {
    ensure!(path.is_absolute(), "rewrite installation paths must be absolute");
    let normalized = path.components().collect::<PathBuf>();
    ensure!(normalized.as_os_str() == path.as_os_str(), "rewrite installation path is noncanonical");
    let mut part = PathBuf::new();
    for component in path.components() {
        ensure!(matches!(component, Component::RootDir | Component::Normal(_)), "rewrite installation path is aliased");
        part.push(component);
        match part.symlink_metadata() {
            Ok(metadata) => ensure!(!metadata.file_type().is_symlink(), "rewrite installation path crosses a symlink"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn elf_seal(path: &Path, shared_library: bool) -> Result<String> {
    unaliased(path)?;
    let mut file = fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(path)?;
    let before = file.metadata()?;
    ensure!(before.is_file() && before.nlink() == 1, "rewrite installation must use regular unaliased files");
    ensure!(before.len() <= MAX_INSTALLED_FILE_BYTES, "rewrite installation file exceeds bounded seal extent");
    let mut header = [0u8; 20];
    file.read_exact(&mut header).context("rewrite program/runtime must be an ELF binary")?;
    ensure!(
        &header[..4] == b"\x7fELF"
            && header[4] == 2
            && header[5] == 1
            && u16::from_le_bytes([header[18], header[19]]) == 62,
        "rewrite installation must use Linux x86_64 ELF files"
    );
    let kind = u16::from_le_bytes([header[16], header[17]]);
    ensure!(if shared_library { kind == 3 } else { matches!(kind, 2 | 3) }, "invalid rewrite ELF file type");
    if !shared_library {
        ensure!(before.mode() & 0o111 != 0, "rewrite installed program is not executable");
    }
    let mut hash = Sha256::new();
    hash.update(header);
    let mut count = header.len() as u64;
    let mut buffer = [0u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        count = count.checked_add(read as u64).context("rewrite installation size overflow")?;
        ensure!(count <= MAX_INSTALLED_FILE_BYTES, "rewrite installation changed beyond seal extent");
        hash.update(&buffer[..read]);
    }
    let after = file.metadata()?;
    ensure!(
        count == before.len()
            && before.len() == after.len()
            && before.ino() == after.ino()
            && before.dev() == after.dev()
            && after.nlink() == 1
            && before.mtime() == after.mtime()
            && before.mtime_nsec() == after.mtime_nsec(),
        "rewrite installation changed during sealing"
    );
    Ok(format!("{:x}", hash.finalize()))
}

fn run_limits_valid(limits: &RunLimits) -> Result<()> {
    ensure!(
        !limits.timeout.is_zero()
            && limits.timeout.as_secs() <= 3600
            && limits.writable_bytes >= 4096
            && limits.writable_bytes.is_multiple_of(4096)
            && limits.writable_bytes <= rewrite::MAX_PROTOCOL_BYTES
            && limits.stdout_bytes > 0
            && limits.stderr_bytes > 0
            && limits.stdin_bytes > 0
            && limits.stdout_bytes as u64 <= rewrite::MAX_PROTOCOL_BYTES
            && limits.stdin_bytes as u64 <= rewrite::MAX_PROTOCOL_BYTES
            && limits.stderr_bytes <= 65536,
        "rewrite invocation lacks bounded time/input/output limits"
    );
    ensure!(
        limits.stdout_bytes.checked_add(limits.stderr_bytes).is_some_and(|bytes| bytes as u64 <= limits.writable_bytes),
        "rewrite output extents exceed aggregate writable quota"
    );
    Ok(())
}

fn recorded_response(arguments: &[String]) -> Result<Option<SourceReplacements>> {
    if arguments.is_empty() {
        return Ok(None);
    }
    ensure!(
        arguments.len() == 1 && arguments[0].len() <= MAX_RECORDED_RESPONSE_BYTES,
        "rewrite command arguments refused"
    );
    let response: SourceReplacements =
        serde_json::from_str(&arguments[0]).context("rewrite arguments must be a recorded source response")?;
    ensure!(serde_json::to_string(&response)? == arguments[0], "recorded source response must be canonical JSON");
    Ok(Some(response))
}

/// Seal a reviewed program, helper, minimal library runtime, and hard limits.
/// No input directory, tool directory, command argv, or output path is accepted.
pub fn installed_identity(program: &Program, limits: &RunLimits) -> Result<String> {
    run_limits_valid(limits)?;
    ensure!(
        program.inputs.is_empty() && program.outputs.is_empty(),
        "rewrite programs cannot bind host inputs or publish files"
    );
    let _ = recorded_response(&program.arguments)?;
    let executable = elf_seal(&program.executable, false)?;
    let helper = elf_seal(&program.helper, false)?;
    let mut libraries = Vec::new();
    let mut destinations = std::collections::BTreeSet::new();
    for bind in &program.runtime {
        let destination = &bind.destination;
        let normalized = destination.components().collect::<PathBuf>();
        ensure!(
            destination.is_absolute()
                && normalized.as_os_str() == destination.as_os_str()
                && destination.components().all(|part| matches!(part, Component::RootDir | Component::Normal(_)))
                && ["/lib", "/lib64", "/usr/lib"].iter().any(|root| destination.starts_with(root))
                && destination.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.contains(".so")),
            "rewrite runtime must contain only individually installed shared libraries"
        );
        ensure!(
            ["/lib", "/lib64", "/usr/lib"].iter().any(|root| bind.source.starts_with(root)),
            "rewrite libraries must come from installed runtime locations"
        );
        ensure!(destinations.insert(destination.clone()), "duplicate rewrite library destination");
        libraries.push((&bind.source, destination, elf_seal(&bind.source, true)?));
    }
    libraries.sort_by(|a, b| a.1.cmp(b.1));
    sealed(&(
        "translation-rewrite-installation-v1",
        &program.executable,
        executable,
        &program.helper,
        helper,
        &program.arguments,
        libraries,
        limits.timeout.as_secs(),
        limits.timeout.subsec_nanos(),
        limits.writable_bytes,
        limits.stdout_bytes,
        limits.stderr_bytes,
        limits.stdin_bytes,
    ))
}

/// An installed protocol program. Its only input is capsule JSON on stdin and
/// its only accepted response is SourceReplacements JSON on stdout.
pub struct InstalledRewriteAdapter {
    operation_root: PathBuf,
    program: Program,
    run_limits: RunLimits,
    identity: String,
}

impl InstalledRewriteAdapter {
    pub fn new(
        operation_root: PathBuf,
        program: Program,
        run_limits: RunLimits,
        expected_identity: String,
    ) -> Result<Self> {
        ensure!(program.arguments.is_empty(), "installed rewrite programs cannot receive host command arguments");
        Self::checked(operation_root, program, run_limits, expected_identity)
    }

    fn checked(
        operation_root: PathBuf,
        program: Program,
        run_limits: RunLimits,
        expected_identity: String,
    ) -> Result<Self> {
        unaliased(&operation_root)?;
        ensure!(digest_valid(&expected_identity), "rewrite installation requires a reviewed identity seal");
        ensure!(
            installed_identity(&program, &run_limits)? == expected_identity,
            "installed rewrite program differs from its reviewed seal"
        );
        Ok(Self { operation_root, program, run_limits, identity: expected_identity })
    }

    fn invoke(
        &self,
        context: &OperationContext,
        capsule: &RewriteCapsule,
        limits: RewriteLimits,
    ) -> Result<SourceReplacements> {
        ensure!(
            context.run_id == capsule.run_id
                && context.attempt == capsule.attempt
                && context.policy_sha256 == capsule.policy_sha256
                && context.phase == ExecutionPhase::Candidate
                && digest_valid(&context.operation_key),
            "rewrite invocation has stale or foreign supervisor identity"
        );
        let input = rewrite::encode(capsule, limits)?;
        ensure!(
            self.run_limits.stdout_bytes as u64 <= limits.max_response_bytes
                && input.len() <= self.run_limits.stdin_bytes,
            "rewrite invocation exceeds frozen capsule budgets"
        );
        ensure!(
            installed_identity(&self.program, &self.run_limits)? == self.identity,
            "installed rewrite program changed before invocation"
        );
        // Durable run owns process identity and wraps confinement::run. A retry
        // authenticates its original bounded result and cannot launch again.
        let result = operation::run(&self.operation_root, context, &self.program, &self.run_limits, &input)?;
        ensure!(result.passed && result.status_raw == Some(0), "restricted rewrite stopped: {}", result.diagnostic);
        ensure!(result.outputs.is_empty(), "rewrite invocation returned a foreign output path");
        rewrite::decode(capsule, &result.stdout, limits)
    }
}

impl RewriteAdapter for InstalledRewriteAdapter {
    fn identity(&self) -> String {
        self.identity.clone()
    }
    fn rewrite(
        &mut self,
        context: &OperationContext,
        capsule: &RewriteCapsule,
        limits: RewriteLimits,
    ) -> Result<SourceReplacements> {
        self.invoke(context, capsule, limits)
    }
}

/// Deterministic replay goes through the same actual restricted process boundary.
pub struct RecordedRewriteAdapter {
    installed: InstalledRewriteAdapter,
    capsule_id: String,
}

impl RecordedRewriteAdapter {
    pub fn program(executable: PathBuf, helper: PathBuf, response: &SourceReplacements) -> Result<Program> {
        let argument = serde_json::to_string(response)?;
        ensure!(argument.len() <= MAX_RECORDED_RESPONSE_BYTES, "recorded source response exceeds bounded argv extent");
        Ok(Program {
            executable,
            helper,
            arguments: vec![argument],
            inputs: vec![],
            outputs: vec![],
            runtime: helper_runtime()?,
        })
    }

    pub fn new(
        operation_root: PathBuf,
        program: Program,
        run_limits: RunLimits,
        expected_identity: String,
        capsule: &RewriteCapsule,
        replacements: &SourceReplacements,
        limits: RewriteLimits,
    ) -> Result<Self> {
        let encoded = serde_json::to_vec(replacements)?;
        ensure!(
            rewrite::decode(capsule, &encoded, limits)? == *replacements,
            "recorded response changed during validation"
        );
        ensure!(
            recorded_response(&program.arguments)?.as_ref() == Some(replacements),
            "recorded program response differs from approved source replacements"
        );
        Ok(Self {
            installed: InstalledRewriteAdapter::checked(operation_root, program, run_limits, expected_identity)?,
            capsule_id: capsule.id.clone(),
        })
    }
}

impl RewriteAdapter for RecordedRewriteAdapter {
    fn identity(&self) -> String {
        self.installed.identity.clone()
    }
    fn rewrite(
        &mut self,
        context: &OperationContext,
        capsule: &RewriteCapsule,
        limits: RewriteLimits,
    ) -> Result<SourceReplacements> {
        ensure!(self.capsule_id == capsule.id, "recorded adapter received a stale or foreign capsule");
        self.installed.invoke(context, capsule, limits)
    }
}

/// Entrypoint of the installed recorded protocol program. stdout contains only
/// validated SourceReplacements; diagnostics are written to bounded stderr.
pub fn recorded_main() -> Result<()> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let response = recorded_response(&arguments)?.context("recorded adapter requires one sealed source response")?;
    let mut input = Vec::new();
    std::io::stdin().take(rewrite::MAX_PROTOCOL_BYTES + 1).read_to_end(&mut input)?;
    ensure!(input.len() as u64 <= rewrite::MAX_PROTOCOL_BYTES, "recorded capsule exceeds protocol ceiling");
    let limits = RewriteLimits {
        max_context_bytes: rewrite::MAX_PROTOCOL_BYTES,
        max_response_bytes: rewrite::MAX_PROTOCOL_BYTES,
        max_file_bytes: rewrite::MAX_PROTOCOL_BYTES,
        max_total_bytes: rewrite::MAX_PROTOCOL_BYTES,
    };
    let capsule = rewrite::decode_capsule(&input, limits)?;
    let encoded = serde_json::to_vec(&response)?;
    let _ = rewrite::decode(&capsule, &encoded, limits)?;
    let mut output = std::io::stdout().lock();
    output.write_all(&encoded)?;
    output.flush()?;
    Ok(())
}
