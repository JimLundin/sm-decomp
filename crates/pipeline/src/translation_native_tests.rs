//! Retained real Native proofs use the public Offline verification interface.
use super::*;
use offline_verification::{
    AuthenticatedNative, ConfinedEnvironment as StrictRuntime, NativeContract, NativeDeclaration,
    NativeDeclarationKind, NativeEvidence, NativeLimits, NativePreparation, NativeProvenance, NativeReceipt,
    NativeTarget, NoScopeAdmission, PreparedNative, Verdict, catalog as batch,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    os::unix::fs::PermissionsExt,
    os::unix::process::ExitStatusExt,
    path::PathBuf,
    time::{Duration, SystemTime},
};
use tool_execution::{
    atomic_toml,
    bounded::{digest, immutable_bytes as immutable, now_unix_ms},
};
use translation_execution::{
    OperationContext,
    confinement::{Program, ReadOnlyBind, RunLimits},
    model::{sealed, sha256},
    operation,
    retention::Reference,
};

fn current_checkout() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}

fn prepare(checkout: &Path, pinned: &Path, contract: &NativeContract) -> Result<PreparedNative> {
    crate::offline::families()?.prepare_native(NativePreparation::new(checkout, pinned, contract), &ScopeApproval)
}

fn run(
    prepared: &PreparedNative,
    checkout: &Path,
    output: &Path,
    runtime: &StrictRuntime,
    context: &OperationContext,
) -> Result<NativeReceipt> {
    run_bounded(prepared, checkout, output, runtime, context, runtime.workers)
}

fn run_bounded(
    prepared: &PreparedNative,
    checkout: &Path,
    output: &Path,
    runtime: &StrictRuntime,
    context: &OperationContext,
    fixture_jobs: usize,
) -> Result<NativeReceipt> {
    prepared.verify(
        NativeTarget { checkout, output, context, fixture_jobs: Some(fixture_jobs) },
        runtime,
        &crate::offline::host_inputs()?,
    )
}

fn authenticate(prepared: &PreparedNative, checkout: &Path, receipt: &NativeReceipt) -> Result<Verdict> {
    Ok(prepared.authenticate(NativeEvidence { checkout, receipt })?.report().clone())
}

struct ProofProducer {
    directory: PathBuf,
    label: String,
    invocation: Asset,
    result: Reference,
}

fn proof_producers(admitted: &AuthenticatedNative, receipt: &NativeReceipt) -> Result<Vec<ProofProducer>> {
    let mut producers = Vec::new();
    for reference in admitted.references() {
        let Some(label) = reference
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(".invocation.toml"))
        else {
            continue;
        };
        if !reference.path.starts_with(&admitted.report().run) {
            continue;
        }
        let record: toml::Value = bounded::object(&reference.path)?;
        let context: OperationContext =
            record.get("context").context("retained producer context missing")?.clone().try_into()?;
        let result = translation_execution::operation::reference(&receipt.operation_root, &context)?;
        ensure!(admitted.references().contains(&result), "producer result absent from admitted retention");
        producers.push(ProofProducer {
            directory: reference.path.parent().context("producer invocation parent missing")?.to_path_buf(),
            label: label.to_owned(),
            invocation: Asset { file: reference.path.clone(), sha256: reference.sha256.clone() },
            result,
        });
    }
    ensure!(producers.len() == receipt.operation_count()?, "retained producer attribution incomplete");
    Ok(producers)
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RetainedProof {
    schema: String,
    cases: BTreeMap<String, RetainedCase>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    capture_refusals: BTreeMap<String, RetainedCaptureRefusal>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    compiler_denials: BTreeMap<String, RetainedOperation>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    runtime_probes: BTreeMap<String, RetainedRuntimeProbes>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RetainedCase {
    prepared: PathBuf,
    checkout: PathBuf,
    receipt: PathBuf,
    passed: bool,
    fixture_pairs: Option<usize>,
    operation_count: Option<usize>,
    diagnostic_contains: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RetainedCaptureRefusal {
    candidate: Asset,
    output: PathBuf,
    diagnostic: String,
    #[serde(default)]
    kind: CaptureRefusalKind,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum CaptureRefusalKind {
    #[default]
    FileMacro,
    RuntimeCapability,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CompilerDenialRequest {
    operation_root: PathBuf,
    context: OperationContext,
    program: Program,
    limits: RunLimits,
    deadline_unix_ms: u64,
    source: Asset,
    protected: Asset,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RetainedOperation {
    request: Asset,
    result: Reference,
    references: Vec<Reference>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProbeRequest {
    operation_root: PathBuf,
    context: OperationContext,
    program: Program,
    limits: RunLimits,
    deadline_unix_ms: u64,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RetainedRuntimeProbe {
    original_invocation: Asset,
    operation: RetainedOperation,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RetainedRuntimeProbes {
    native_case: String,
    operation_root: PathBuf,
    deadline_unix_ms: u64,
    protected_parent: Asset,
    protected_sibling: Asset,
    protected_c_program: Asset,
    source: Asset,
    executable: Asset,
    compiler_invocation: Asset,
    build: RetainedOperation,
    fixtures: Vec<RetainedRuntimeProbe>,
}

// Read only after the entire original Native receipt admitted this exact
// invocation reference. Other fields remain authenticated by that admission.
#[derive(Deserialize)]
struct AdmittedInvocation {
    context: OperationContext,
    program: Program,
    requested_limits: RunLimits,
    limits: RunLimits,
}

fn retain_case(root: &Path, name: &str, mut case: RetainedCase) -> Result<()> {
    case.prepared = fs::canonicalize(case.prepared)?;
    case.checkout = fs::canonicalize(case.checkout)?;
    case.receipt = fs::canonicalize(case.receipt)?;
    retain_proof(root, |proof| {
        proof.cases.insert(name.into(), case);
    })
}

fn retain_proof(root: &Path, update: impl FnOnce(&mut RetainedProof)) -> Result<()> {
    let path = root.join("retained-authentication.toml");
    let mut proof = if path.exists() {
        bounded::object::<RetainedProof>(&path)?
    } else {
        RetainedProof {
            schema: "retained-native-proof-v1".into(),
            cases: BTreeMap::new(),
            capture_refusals: BTreeMap::new(),
            compiler_denials: BTreeMap::new(),
            runtime_probes: BTreeMap::new(),
        }
    };
    ensure!(proof.schema == "retained-native-proof-v1", "retained Native proof schema changed");
    update(&mut proof);
    atomic_toml(&path, &proof)
}

fn cleanup_proof(root: &Path) -> Result<()> {
    for deduplicate in [false, true] {
        let result =
            crate::cleanup::cleanup(crate::cleanup::CleanupRequest { root: root.into(), apply: true, deduplicate })?;
        atomic_toml(&root.join(if deduplicate { "cleanup-deduplicated.toml" } else { "cleanup.toml" }), &result)?;
    }
    Ok(())
}

fn authenticate_retained(path: &Path) -> Result<()> {
    let proof: RetainedProof = bounded::object(path)?;
    ensure!(
        proof.schema == "retained-native-proof-v1"
            && !(proof.cases.is_empty()
                && proof.capture_refusals.is_empty()
                && proof.compiler_denials.is_empty()
                && proof.runtime_probes.is_empty()),
        "retained Native proof is incomplete"
    );
    for (name, case) in &proof.cases {
        let prepared: PreparedNative = bounded::object(&case.prepared)?;
        let receipt: NativeReceipt = bounded::object(&case.receipt)?;
        let admitted = prepared.authenticate(NativeEvidence { checkout: &case.checkout, receipt: &receipt })?;
        ensure!(receipt.passed == case.passed, "{name}: retained Native outcome changed");
        if let Some(pairs) = case.fixture_pairs {
            ensure!(admitted.report().verified_cases == pairs, "{name}: complete Fixture domain changed");
        }
        if let Some(operations) = case.operation_count {
            ensure!(receipt.operation_count()? == operations, "{name}: producer attribution changed");
        }
        if let Some(diagnostic) = &case.diagnostic_contains {
            ensure!(receipt.diagnostic.contains(diagnostic.as_str()), "{name}: rejection diagnostic changed");
        }
        println!("Authenticated retained Native proof {name}: {}", admitted.report().status);
    }
    for (name, refusal) in proof.capture_refusals {
        authenticate_capture_refusal(&refusal)?;
        println!("Authenticated retained Native capture refusal {name}");
    }
    for (name, denial) in proof.compiler_denials {
        authenticate_compiler_denial(&denial)?;
        println!("Authenticated retained confined compiler denial {name}");
    }
    for (name, probes) in proof.runtime_probes {
        let case = proof.cases.get(&probes.native_case).context("runtime probes have no retained Native case")?;
        authenticate_runtime_probes(&probes, case)?;
        println!("Authenticated retained runtime proof {name}: 16 original Rust fixture namespaces");
    }
    Ok(())
}

fn authenticate_capture_refusal(refusal: &RetainedCaptureRefusal) -> Result<()> {
    let (bytes, hash) = bounded::read_with_digest(&refusal.candidate.file)?;
    ensure!(hash == refusal.candidate.sha256, "capture-refusal candidate changed");
    let error = offline_verification::source_facts::scan(&refusal.candidate.file, &bytes)?
        .require_complete()
        .err()
        .context("uncaptured file macro became a complete source closure")?;
    let markers: &[&str] = match refusal.kind {
        CaptureRefusalKind::FileMacro => &["uncaptured file macro include_str!"],
        CaptureRefusalKind::RuntimeCapability => &[
            "unmodeled runtime input or capability: std::fs",
            "unmodeled runtime input or capability: std::fs::read",
            "unmodeled runtime input or capability: std::fs::write",
        ],
    };
    ensure!(
        format!("{error:#}") == refusal.diagnostic
            && refusal.diagnostic.contains("Rust source has unresolved inputs")
            && markers.iter().all(|marker| refusal.diagnostic.contains(marker)),
        "Native capture-refusal diagnostic changed"
    );
    ensure!(
        !refusal.output.join("native-intent.toml").exists()
            && !refusal.output.join("native-receipt.toml").exists()
            && bounded::entries(&refusal.output.join("family"))?.is_empty(),
        "refused source capture published Native intent or producer evidence"
    );
    Ok(())
}

fn compiler_denial_source(protected: &Path) -> String {
    format!("const SECRET: &str = include_str!({protected:?});\nfn main() {{ let _ = SECRET; }}\n")
}

fn compiler_denial_arguments() -> Vec<String> {
    [
        "--edition=2024",
        "--crate-name=native_compile_denial",
        "--sysroot",
        "/toolchain",
        "--emit=metadata",
        "/inputs/denial.rs",
        "-o",
        "/work/tmp/denial.rmeta",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn compiler_denial_limits() -> RunLimits {
    RunLimits {
        timeout: Duration::from_secs(60),
        writable_bytes: 128 * 1024 * 1024,
        stdout_bytes: 4096,
        stderr_bytes: 256 * 1024,
        stdin_bytes: 0,
    }
}

fn producer_stamps(directory: &Path) -> Result<BTreeMap<PathBuf, (String, SystemTime)>> {
    ["intent.json", "runner.json", "sandbox.json", "result.json"]
        .into_iter()
        .map(|name| {
            let path = directory.join(name);
            Ok((path.clone(), (digest(&path)?, fs::metadata(&path)?.modified()?)))
        })
        .collect()
}

fn authenticate_compiler_denial(denial: &RetainedOperation) -> Result<()> {
    ensure!(digest(&denial.request.file)? == denial.request.sha256, "compiler-denial request changed");
    let request: CompilerDenialRequest = bounded::object(&denial.request.file)?;
    let (source, hash) = bounded::read_with_digest(&request.source.file)?;
    ensure!(
        hash == request.source.sha256 && source == compiler_denial_source(&request.protected.file).as_bytes(),
        "compiler-denial probe source changed"
    );
    let (protected, hash) = bounded::read_with_digest(&request.protected.file)?;
    ensure!(
        hash == request.protected.sha256 && protected == b"protected compiler input\n",
        "compiler-denial protected input changed"
    );
    ensure!(
        request.program.arguments == compiler_denial_arguments()
            && request.program.outputs.is_empty()
            && request.program.inputs
                == vec![ReadOnlyBind { source: request.source.file.clone(), destination: "/inputs/denial.rs".into() }]
            && request.program.executable.file_name().is_some_and(|name| name == "rustc")
            && request.limits == compiler_denial_limits(),
        "compiler-denial request does not prescribe the real Rust compiler probe"
    );
    ensure!(
        request.program.inputs.iter().chain(&request.program.runtime).all(|binding| {
            !request.protected.file.starts_with(&binding.source)
                && !request.protected.file.starts_with(&binding.destination)
        }),
        "compiler-denial request exposes the protected host file"
    );
    let directory = operation::directory(&request.operation_root, &request.context)?;
    let before = producer_stamps(&directory)?;
    let admitted = operation::authenticate_request_with_references(
        &request.operation_root,
        &request.context,
        &request.program,
        &request.limits,
        b"",
    )?;
    let result = admitted.result();
    ensure!(
        admitted.result_reference() == &denial.result && admitted.references() == denial.references,
        "compiler-denial retained attribution changed"
    );
    ensure!(
        !result.passed
            && result.status_raw.is_some_and(|raw| !std::process::ExitStatus::from_raw(raw).success())
            && result.diagnostic.contains("couldn't read")
            && result.diagnostic.contains(request.protected.file.to_str().context("protected path is not UTF-8")?),
        "the actual confined compiler did not deny the protected file: {}",
        result.diagnostic
    );
    for name in ["runner.json", "sandbox.json"] {
        ensure!(
            admitted.references().iter().any(|reference| reference.path == directory.join(name)),
            "compiler-denial result lacks actual runner attribution"
        );
    }
    ensure!(producer_stamps(&directory)? == before, "compiler-denial admission rewrote producer evidence");
    Ok(())
}

fn prove_compiler_denial(root: &Path, runtime: &StrictRuntime, context: &OperationContext) -> Result<()> {
    let root = fs::canonicalize(root)?;
    let directory = root.join("compiler-denial");
    fs::create_dir_all(&directory)?;
    let protected = directory.join("compiler-secret.txt");
    immutable(&protected, b"protected compiler input\n")?;
    let source = directory.join("denial.rs");
    immutable(&source, compiler_denial_source(&protected).as_bytes())?;
    let now = now_unix_ms()?;
    let deadline_unix_ms = now
        .checked_add(180_000)
        .context("compiler-denial proof deadline overflow")?
        .min(runtime.deadline_unix_ms.unwrap_or(u64::MAX));
    let rustc = fs::canonicalize(&runtime.rustc)?;
    let program = Program {
        executable: rustc.clone(),
        arguments: compiler_denial_arguments(),
        inputs: vec![ReadOnlyBind { source: source.clone(), destination: "/inputs/denial.rs".into() }],
        runtime: translation_execution::compiler_runtime::discover_until(
            &rustc,
            deadline_unix_ms,
            8 * bounded::DEFAULT_BYTES,
        )?,
        helper: fs::canonicalize(&runtime.helper)?,
        outputs: Vec::new(),
    };
    let mut context = context.clone();
    context.operation_key = sha256("native-actual-compiler-file-denial");
    let request = CompilerDenialRequest {
        operation_root: runtime.operation_root.clone(),
        context,
        program,
        limits: compiler_denial_limits(),
        deadline_unix_ms,
        source: Asset { file: source.clone(), sha256: digest(&source)? },
        protected: Asset { file: protected.clone(), sha256: digest(&protected)? },
    };
    let request_path = directory.join("request.toml");
    immutable(&request_path, toml::to_string(&request)?.as_bytes())?;
    let (lease, _) =
        tool_execution::acquire_lease_until(&runtime.pipeline_root, runtime.workers, request.deadline_unix_ms)?;
    let result = operation::run_until(
        &request.operation_root,
        &request.context,
        &request.program,
        &request.limits,
        b"",
        request.deadline_unix_ms,
    )?;
    drop(lease);
    let admitted = operation::authenticate_request_with_references(
        &request.operation_root,
        &request.context,
        &request.program,
        &request.limits,
        b"",
    )?;
    let denial = RetainedOperation {
        request: Asset { file: request_path.clone(), sha256: digest(&request_path)? },
        result: admitted.result_reference().clone(),
        references: admitted.references().to_vec(),
    };
    authenticate_compiler_denial(&denial)?;
    let operation_directory = operation::directory(&request.operation_root, &request.context)?;
    let before = producer_stamps(&operation_directory)?;
    let retry = operation::run_until(
        &request.operation_root,
        &request.context,
        &request.program,
        &request.limits,
        b"",
        request.deadline_unix_ms,
    )?;
    ensure!(retry == result, "compiler-denial retry returned another result");
    ensure!(producer_stamps(&operation_directory)? == before, "compiler-denial retry reran or rewrote the producer");
    authenticate_compiler_denial(&denial)?;
    retain_proof(&root, |proof| {
        proof.compiler_denials.insert("undeclared-host-file".into(), denial);
    })?;
    println!(
        "Actual Rust compiler denied the undeclared host file; durable retry preserved result and runner bytes/mtimes"
    );
    Ok(())
}

fn retained_asset(path: &Path) -> Result<Asset> {
    bounded::safe(path)?;
    let file = fs::canonicalize(path)?;
    ensure!(file.symlink_metadata()?.is_file(), "proof asset is not a regular file");
    Ok(Asset { sha256: digest(&file)?, file })
}

fn read_proof_asset(asset: &Asset) -> Result<Vec<u8>> {
    let (bytes, hash) = bounded::read_with_digest(&asset.file)?;
    ensure!(hash == asset.sha256, "retained proof asset changed: {}", asset.file.display());
    Ok(bytes)
}

fn admitted_invocation(admitted: &AuthenticatedNative, asset: &Asset) -> Result<AdmittedInvocation> {
    ensure!(
        admitted.references().iter().any(|reference| reference.path == asset.file && reference.sha256 == asset.sha256),
        "probe source invocation is not independently admitted Native evidence"
    );
    Ok(toml::from_str(std::str::from_utf8(&read_proof_asset(asset)?)?)?)
}

fn admit_runtime_case(case: &RetainedCase) -> Result<(NativeReceipt, AuthenticatedNative)> {
    ensure!(
        case.passed && case.fixture_pairs == Some(16) && case.operation_count == Some(34),
        "runtime probes require the complete passing Native family"
    );
    let prepared: PreparedNative = bounded::object(&case.prepared)?;
    let receipt: NativeReceipt = bounded::object(&case.receipt)?;
    let admitted = prepared.authenticate(NativeEvidence { checkout: &case.checkout, receipt: &receipt })?;
    ensure!(
        receipt.passed && receipt.operation_count()? == 34 && admitted.report().verified_cases == 16,
        "runtime probe Native source domain changed"
    );
    Ok((receipt, admitted))
}

fn runtime_probe_source(parent: &Path) -> String {
    format!(
        r#"fn main() {{
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 7);
    assert_eq!(args[4], "/inputs/before.wram");
    assert_eq!(args[5], "/work/after.wram");
    assert_eq!(args[6], "/inputs/rom.bin");
    assert!(std::fs::read({parent:?}).is_err(), "parent read escaped");
    assert!(std::fs::write({parent:?}, b"forged").is_err(), "parent write escaped");
    assert!(std::fs::read("../Sidehopper_Func_1-0/c-after.wram").is_err(), "sibling read escaped");
    assert!(std::fs::read("/program-c").is_err(), "C program read escaped");
    let before = std::fs::read(&args[4]).unwrap();
    assert_eq!(before.len(), 131072);
    std::fs::write(&args[5], &before).unwrap();
    print!("native-runtime-denials-passed\n");
}}
"#
    )
}

fn runtime_fixture_limits() -> RunLimits {
    RunLimits {
        timeout: Duration::from_secs(5),
        writable_bytes: 512 * 1024,
        stdout_bytes: 4096,
        stderr_bytes: 4096,
        stdin_bytes: 0,
    }
}

fn probe_context(original: &OperationContext, role: &str, subject: &Asset) -> Result<OperationContext> {
    let mut context = original.clone();
    context.operation_key = sealed(&(role, original, subject))?;
    Ok(context)
}

fn runtime_probe_build_program(original: &Program, source: &Asset) -> Result<Program> {
    ensure!(
        original.executable.file_name().is_some_and(|name| name == "rustc")
            && original.arguments.iter().any(|argument| argument == "/inputs/generated/wrapper.rs")
            && original.outputs == vec![PathBuf::from("rust"), PathBuf::from("compiler.d")],
        "admitted Native compiler has no supported standalone probe source seam"
    );
    let mut program = original.clone();
    program.inputs =
        vec![ReadOnlyBind { source: source.file.clone(), destination: "/inputs/generated/wrapper.rs".into() }];
    Ok(program)
}

fn run_passing_probe(request: &ProbeRequest, path: &Path, runtime: &StrictRuntime) -> Result<RetainedOperation> {
    immutable(path, toml::to_string(request)?.as_bytes())?;
    let (lease, _) =
        tool_execution::acquire_lease_until(&runtime.pipeline_root, runtime.workers, request.deadline_unix_ms)?;
    let result = operation::run_until(
        &request.operation_root,
        &request.context,
        &request.program,
        &request.limits,
        b"",
        request.deadline_unix_ms,
    )?;
    drop(lease);
    ensure!(result.passed, "runtime diagnostic operation failed: {}", result.diagnostic);
    let directory = operation::directory(&request.operation_root, &request.context)?;
    let before = producer_stamps(&directory)?;
    let retry = operation::run_until(
        &request.operation_root,
        &request.context,
        &request.program,
        &request.limits,
        b"",
        request.deadline_unix_ms,
    )?;
    ensure!(retry == result, "runtime diagnostic retry returned another result");
    ensure!(producer_stamps(&directory)? == before, "runtime diagnostic retry reran or rewrote its producer");
    let admitted = operation::authenticate_request_with_references(
        &request.operation_root,
        &request.context,
        &request.program,
        &request.limits,
        b"",
    )?;
    Ok(RetainedOperation {
        request: retained_asset(path)?,
        result: admitted.result_reference().clone(),
        references: admitted.references().to_vec(),
    })
}

fn authenticate_probe_operation(
    record: &RetainedOperation,
) -> Result<(ProbeRequest, operation::AuthenticatedOperation)> {
    let request: ProbeRequest = toml::from_str(std::str::from_utf8(&read_proof_asset(&record.request)?)?)?;
    let directory = operation::directory(&request.operation_root, &request.context)?;
    let before = producer_stamps(&directory)?;
    let admitted = operation::authenticate_request_with_references(
        &request.operation_root,
        &request.context,
        &request.program,
        &request.limits,
        b"",
    )?;
    ensure!(
        admitted.result_reference() == &record.result && admitted.references() == record.references,
        "runtime diagnostic operation attribution changed"
    );
    ensure!(
        admitted.result().passed
            && admitted.result().status_raw.is_some_and(|raw| std::process::ExitStatus::from_raw(raw).success()),
        "runtime diagnostic operation was not executed successfully"
    );
    for name in ["runner.json", "sandbox.json"] {
        ensure!(
            admitted.references().iter().any(|reference| reference.path == directory.join(name)),
            "runtime diagnostic operation lacks actual process attribution"
        );
    }
    ensure!(producer_stamps(&directory)? == before, "runtime diagnostic admission rewrote producer evidence");
    Ok((request, admitted))
}

fn check_runtime_protected(proof: &RetainedRuntimeProbes, admitted: &AuthenticatedNative) -> Result<()> {
    ensure!(
        read_proof_asset(&proof.protected_parent)? == b"protected parent verdict",
        "runtime probe changed the real protected parent"
    );
    ensure!(
        proof.protected_sibling.file == admitted.report().run.join("Sidehopper_Func_1-0/c-after.wram")
            && proof.protected_c_program.file == admitted.report().run.join("c"),
        "runtime probe protected host paths do not name the real Native siblings"
    );
    for asset in [&proof.protected_sibling, &proof.protected_c_program] {
        ensure!(
            admitted
                .references()
                .iter()
                .any(|reference| reference.path == asset.file && reference.sha256 == asset.sha256),
            "runtime probe protected sibling is not independently admitted Native evidence"
        );
        ensure!(!read_proof_asset(asset)?.is_empty(), "runtime probe protected host file is absent or empty");
    }
    Ok(())
}

fn authenticate_runtime_probes(proof: &RetainedRuntimeProbes, case: &RetainedCase) -> Result<()> {
    let (receipt, admitted) = admit_runtime_case(case)?;
    let producers = proof_producers(&admitted, &receipt)?;
    let compiler =
        producers.iter().find(|producer| producer.label == "rust-build").context("Native compiler missing")?;
    ensure!(compiler.invocation == proof.compiler_invocation, "runtime probe compiler attribution changed");
    let original = admitted_invocation(&admitted, &proof.compiler_invocation)?;
    ensure!(
        original.requested_limits == compiler_denial_limits() && original.limits == compiler_denial_limits(),
        "runtime probe compiler limits differ from the original 60-second Native compiler"
    );
    ensure!(
        read_proof_asset(&proof.source)? == runtime_probe_source(&proof.protected_parent.file).as_bytes(),
        "runtime probe source no longer performs the prescribed denials and declared output copy"
    );
    check_runtime_protected(proof, &admitted)?;
    let (build, compiled) = authenticate_probe_operation(&proof.build)?;
    ensure!(
        build.operation_root == proof.operation_root
            && build.deadline_unix_ms == proof.deadline_unix_ms
            && build.context == probe_context(&original.context, "native-runtime-probe-build-v1", &proof.source)?
            && build.program == runtime_probe_build_program(&original.program, &proof.source)?
            && build.limits == original.limits,
        "runtime probe build is not bound to the admitted Native compiler and trusted probe source"
    );
    ensure!(
        compiled.result().outputs.get(Path::new("rust")).context("runtime probe compiler output missing")?
            == &read_proof_asset(&proof.executable)?,
        "runtime probe executable differs from its durable compiler output"
    );
    let expected = producers
        .iter()
        .filter(|producer| producer.label == "rust")
        .map(|producer| (producer.invocation.file.clone(), producer.invocation.sha256.clone()))
        .collect::<BTreeMap<_, _>>();
    ensure!(
        expected.len() == 16 && proof.fixtures.len() == 16,
        "runtime proof does not cover every Native Rust fixture"
    );
    let mut observed = BTreeSet::new();
    for fixture in &proof.fixtures {
        ensure!(
            expected.get(&fixture.original_invocation.file) == Some(&fixture.original_invocation.sha256)
                && observed.insert(fixture.original_invocation.file.clone()),
            "runtime proof repeats or substitutes a Native Rust fixture"
        );
        let original = admitted_invocation(&admitted, &fixture.original_invocation)?;
        ensure!(
            original.requested_limits == runtime_fixture_limits() && original.limits == runtime_fixture_limits(),
            "runtime probe changed the original five-second/512-KiB Fixture limits"
        );
        let mut program = original.program.clone();
        program.executable = proof.executable.file.clone();
        let (request, result) = authenticate_probe_operation(&fixture.operation)?;
        ensure!(
            request.operation_root == proof.operation_root
                && request.deadline_unix_ms == proof.deadline_unix_ms
                && request.context
                    == probe_context(&original.context, "native-runtime-probe-fixture-v1", &proof.executable)?
                && request.program == program
                && request.limits == original.limits,
            "runtime probe changed the admitted Fixture namespace, argv, runtime, helper or output obligations"
        );
        let input = program
            .inputs
            .iter()
            .find(|input| input.destination == Path::new("/inputs/before.wram"))
            .context("original Native Fixture lacks its declared WRAM input")?;
        ensure!(
            result.result().stdout == b"native-runtime-denials-passed\n"
                && result.result().stderr.is_empty()
                && result.result().outputs
                    == BTreeMap::from([(PathBuf::from("after.wram"), bounded::read(&input.source)?)]),
            "runtime probe failed its exact success marker or declared before-to-after output contract"
        );
    }
    ensure!(observed.len() == expected.len(), "runtime probe Native fixture membership is incomplete");
    check_runtime_protected(proof, &admitted)?;
    Ok(())
}

fn prove_runtime_probes(root: &Path, native_case: &str, case: &RetainedCase, runtime: &StrictRuntime) -> Result<()> {
    let (receipt, admitted) = admit_runtime_case(case)?;
    let producers = proof_producers(&admitted, &receipt)?;
    ensure!(
        producers.iter().filter(|producer| producer.label == "rust").count() == 16,
        "runtime probes require every original Rust Fixture invocation"
    );
    let root = fs::canonicalize(root)?;
    let directory = root.join("runtime-probes");
    fs::create_dir_all(&directory)?;
    let parent = root.join("parent-verdict.toml");
    immutable(&parent, b"protected parent verdict")?;
    let source_path = directory.join("probe.rs");
    immutable(&source_path, runtime_probe_source(&parent).as_bytes())?;
    let source = retained_asset(&source_path)?;
    let protected_parent = retained_asset(&parent)?;
    let protected_sibling = retained_asset(&admitted.report().run.join("Sidehopper_Func_1-0/c-after.wram"))?;
    let protected_c_program = retained_asset(&admitted.report().run.join("c"))?;
    let compiler =
        producers.iter().find(|producer| producer.label == "rust-build").context("Native compiler missing")?;
    let original = admitted_invocation(&admitted, &compiler.invocation)?;
    ensure!(
        original.requested_limits == compiler_denial_limits() && original.limits == compiler_denial_limits(),
        "runtime probe compiler limits differ from the original 60-second Native compiler"
    );
    let deadline_unix_ms = now_unix_ms()?
        .checked_add(180_000)
        .context("runtime probe proof deadline overflow")?
        .min(runtime.deadline_unix_ms.unwrap_or(u64::MAX));
    let request = ProbeRequest {
        operation_root: runtime.operation_root.clone(),
        context: probe_context(&original.context, "native-runtime-probe-build-v1", &source)?,
        program: runtime_probe_build_program(&original.program, &source)?,
        limits: original.limits,
        deadline_unix_ms,
    };
    let build = run_passing_probe(&request, &directory.join("build.toml"), runtime)?;
    let (_, compiled) = authenticate_probe_operation(&build)?;
    let executable_path = directory.join("probe");
    immutable(&executable_path, compiled.result().outputs.get(Path::new("rust")).context("probe binary missing")?)?;
    fs::set_permissions(&executable_path, fs::Permissions::from_mode(0o500))?;
    let executable = retained_asset(&executable_path)?;
    let mut proof = RetainedRuntimeProbes {
        native_case: native_case.into(),
        operation_root: runtime.operation_root.clone(),
        deadline_unix_ms,
        protected_parent,
        protected_sibling,
        protected_c_program,
        source,
        executable,
        compiler_invocation: compiler.invocation.clone(),
        build,
        fixtures: Vec::new(),
    };
    check_runtime_protected(&proof, &admitted)?;
    for (index, producer) in producers.iter().filter(|producer| producer.label == "rust").enumerate() {
        let original = admitted_invocation(&admitted, &producer.invocation)?;
        ensure!(
            original.requested_limits == runtime_fixture_limits() && original.limits == runtime_fixture_limits(),
            "runtime probe changed the original five-second/512-KiB Fixture limits"
        );
        let mut program = original.program;
        program.executable = proof.executable.file.clone();
        let request = ProbeRequest {
            operation_root: runtime.operation_root.clone(),
            context: probe_context(&original.context, "native-runtime-probe-fixture-v1", &proof.executable)?,
            program,
            limits: original.limits,
            deadline_unix_ms,
        };
        let operation = run_passing_probe(&request, &directory.join(format!("fixture-{index:02}.toml")), runtime)?;
        proof.fixtures.push(RetainedRuntimeProbe { original_invocation: producer.invocation.clone(), operation });
        check_runtime_protected(&proof, &admitted)?;
    }
    authenticate_runtime_probes(&proof, case)?;
    retain_proof(&root, |retained| {
        retained.runtime_probes.insert("all-native-rust-fixture-namespaces".into(), proof);
    })?;
    println!(
        "Runtime denials passed in all 16 admitted Rust Fixture namespaces; 17 separate diagnostic producers replayed unchanged"
    );
    Ok(())
}

#[test]
#[ignore = "real runtime probes in all 16 admitted Native Fixture namespaces; retains build, replay and readback"]
fn confined_runtime_probes_replay_and_authenticate() -> Result<()> {
    let root = PathBuf::from(std::env::var("SM_NATIVE_RUNTIME_PROBE_PROOF")?);
    fs::create_dir_all(&root)?;
    let seed: RetainedProof = bounded::object(Path::new(&std::env::var("SM_NATIVE_RUNTIME_SEED_MANIFEST")?))?;
    let case = seed.cases.get("run").context("runtime probe seed has no full Native run")?.clone();
    retain_case(&root, "runtime-family", case.clone())?;
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
    prove_runtime_probes(&root, "runtime-family", &case, &runtime)?;
    cleanup_proof(&root)?;
    authenticate_retained(&root.join("retained-authentication.toml"))
}

#[test]
#[ignore = "bounded real compiler denial; retains exact operation, retry and independent authentication evidence"]
fn confined_compiler_denial_replays_and_authenticates() -> Result<()> {
    let root = PathBuf::from(std::env::var("SM_NATIVE_COMPILER_DENIAL_PROOF")?);
    fs::create_dir_all(&root)?;
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
        run_id: sha256("native-compiler-denial-focused-proof"),
        attempt: 1,
        policy_sha256: sha256("actual compiler must deny an undeclared host input"),
        operation_key: sha256("native-compiler-denial-focused-proof"),
        phase: translation_execution::model::ExecutionPhase::Candidate,
    };
    prove_compiler_denial(&root, &runtime, &context)?;
    cleanup_proof(&root)?;
    authenticate_retained(&root.join("retained-authentication.toml"))
}

#[test]
#[ignore = "read-only retained Native authentication; never prepares or executes a producer"]
fn retained_native_proofs_authenticate_without_execution() -> Result<()> {
    authenticate_retained(Path::new(&std::env::var("SM_NATIVE_AUTHENTICATION_MANIFEST")?))
}

#[test]
#[ignore = "real separately confined actual-C controls; retains source, replay and cleanup evidence"]
fn real_reviewed_recipes_replay_and_authenticate() -> Result<()> {
    crate::initialize_test_binary_identity()?;
    let offline = crate::offline::families()?;
    let root = PathBuf::from(std::env::var("SM_NATIVE_RECIPE_PROOF")?);
    fs::create_dir_all(&root)?;
    let current = std::env::var_os("SM_NATIVE_RECIPE_SOURCE").map(PathBuf::from).unwrap_or_else(current_checkout);
    let current = fs::canonicalize(current)?;
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
    crate::translation_execution::tests::copy_native_source_snapshot(&current, &checkout)?;
    for name in &names {
        let manifest = if *name == "pause-direct-animation" {
            paths.insert(".scratch/decomp-coverage/renderer/direct-animation.toml".into());
            "tools/port/pause-equipment-render.smf".to_owned()
        } else {
            format!("tools/port/{name}.smf")
        };
        let declaration = offline_verification::fixtures::read(&current.join(&manifest))?;
        paths.insert(manifest);
        for dependency in declaration.rust_dependencies {
            // Captured declarations can name authoring inputs outside the
            // compiled harness. Absolute capture inputs retain their paths.
            if dependency.is_relative() {
                paths.insert(dependency.to_str().context("native proof dependency path is not UTF-8")?.into());
            }
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
        offline.prepare_generated_plm(&checkout, &checkout.join(".scratch/translation-native/plm-instructions"))?;
    }
    let catalog = root.join("catalog.toml");
    immutable(&catalog, toml::to_string(&batch::catalog(&checkout)?)?.as_bytes())?;
    let requirements = root.join("requirements.md");
    let requirements_source =
        std::env::var_os("SM_NATIVE_RECIPE_REQUIREMENTS").map(PathBuf::from).unwrap_or_else(|| {
            current.join(".scratch/translation-scope-execution/issues/10-original-acceptance-adapters.md")
        });
    immutable(&requirements, &bounded::read(requirements_source)?)?;
    let inventory = Asset {
        file: "tools/port/source-inventory.toml".into(),
        sha256: digest(&checkout.join("tools/port/source-inventory.toml"))?,
    };
    let provenance = NativeProvenance::ReviewedCatalog {
        catalog: Asset { file: catalog.clone(), sha256: digest(&catalog)? },
        requirements: Asset { file: requirements.clone(), sha256: digest(&requirements)? },
        checkout: checkout.clone(),
        rationale:
            "Complete selected native declarations against actual pinned C, bound to the retained requirements artifact"
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
    let contracts = names
        .iter()
        .map(|name| {
            let path = if *name == "pause-direct-animation" {
                PathBuf::from(".scratch/decomp-coverage/renderer/direct-animation.toml")
            } else {
                PathBuf::from(format!("tools/port/{name}.smf"))
            };
            offline.declare_native(NativeDeclaration {
                checkout: &checkout,
                kind: if *name == "pause-direct-animation" {
                    NativeDeclarationKind::DirectAnimation(&path)
                } else {
                    NativeDeclarationKind::Family(&path)
                },
                source_inventory: inventory.clone(),
                provenance: provenance.clone(),
                compiler_limits: compiler.clone(),
                fixture_limits: fixture.clone(),
                deadline_unix_ms: None,
                max_input_bytes: 8 * bounded::DEFAULT_BYTES,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let requests = contracts
        .iter()
        .map(|contract| NativePreparation {
            deadline_unix_ms: None,
            max_input_bytes: 8 * bounded::DEFAULT_BYTES,
            ..NativePreparation::new(&checkout, pinned, contract)
        })
        .collect::<Vec<_>>();
    let mut preparations = decomp_source::context::until(now_unix_ms()? + 600_000, 8 * bounded::DEFAULT_BYTES, || {
        offline.prepare_native_catalog(&requests, &NoScopeAdmission)
    })?;
    for name in names {
        let output = root.join(name);
        fs::create_dir_all(&output)?;
        let prepared = decomp_source::context::until(now_unix_ms()? + 600_000, 8 * bounded::DEFAULT_BYTES, || {
            preparations.next().context("prepared recipe declaration missing")?
        })?;
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
            policy_sha256: prepared.verification_sha256().to_owned(),
            operation_key: sha256(name),
            phase: translation_execution::model::ExecutionPhase::Candidate,
        };
        let fixture_jobs = std::env::var("SM_NATIVE_FIXTURE_JOBS")
            .ok()
            .map(|value| value.parse::<usize>())
            .transpose()?
            .unwrap_or(runtime.workers);
        let receipt = run_bounded(&prepared, &checkout, &output.join("run"), &runtime, &context, fixture_jobs)?;
        retain_case(
            &root,
            name,
            RetainedCase {
                prepared: output.join("prepared.toml"),
                checkout: checkout.clone(),
                receipt: output.join("run/native-receipt.toml"),
                passed: true,
                fixture_pairs: Some(prepared.contract().coverage.values().map(Vec::len).sum()),
                operation_count: None,
                diagnostic_contains: None,
            },
        )?;
        let admitted = prepared.authenticate(NativeEvidence { checkout: &checkout, receipt: &receipt })?;
        let verdict = admitted.report();
        let identities = proof_producers(&admitted, &receipt)?
            .into_iter()
            .map(|operation| {
                Ok((
                    operation.result.path.clone(),
                    operation.result.sha256,
                    operation.result.path.metadata()?.modified()?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        ensure!(
            sealed(&run(&prepared, &checkout, &output.join("run"), &runtime, &context)?)? == sealed(&receipt)?,
            "recipe replay changed receipt"
        );
        for (path, hash, modified) in identities {
            ensure!(
                path.metadata()?.modified()? == modified && digest(&path)? == hash,
                "recipe replay changed producer"
            );
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
    cleanup_proof(&root)?;
    ensure!(failures.is_empty(), "{}", failures.join("; "));
    authenticate_retained(&root.join("retained-authentication.toml"))?;
    Ok(())
}

#[test]
#[ignore = "freezes every current catalog declaration with real pinned source; no producer is launched"]
fn complete_current_catalog_has_a_frozen_native_recipe() -> Result<()> {
    crate::initialize_test_binary_identity()?;
    let offline = crate::offline::families()?;
    let root = PathBuf::from(std::env::var("SM_NATIVE_CATALOG_PROOF")?);
    fs::create_dir_all(&root)?;
    // The catalog's generated PLM declarations belong in an explicit retained
    // source snapshot, because this proof authors those inputs.
    let checkout = fs::canonicalize(PathBuf::from(std::env::var("SM_NATIVE_CATALOG_SOURCE")?))?;
    ensure!(checkout != fs::canonicalize(current_checkout())?, "catalog proof requires a separate source snapshot");
    offline.prepare_generated_plm(&checkout, &checkout.join(".scratch/translation-native/plm-instructions"))?;
    let catalog = batch::catalog(&checkout)?;
    let catalog_path = root.join("catalog.toml");
    immutable(&catalog_path, toml::to_string(&catalog)?.as_bytes())?;
    let requirements = checkout.join(".scratch/translation-scope-execution/issues/10-original-acceptance-adapters.md");
    let provenance = NativeProvenance::ReviewedCatalog {
        catalog: Asset { file: catalog_path.clone(), sha256: digest(&catalog_path)? },
        requirements: Asset { file: requirements.clone(), sha256: digest(&requirements)? },
        checkout: checkout.clone(),
        rationale: "Current conservative catalog: every registered declaration, all generated cases, exact original requirements".into(),
    };
    let inventory = Asset {
        file: "tools/port/source-inventory.toml".into(),
        sha256: digest(&checkout.join("tools/port/source-inventory.toml"))?,
    };
    let pinned = Path::new("/home/jim/sm-decomp/deps/snesrev-sm");
    let deadline = now_unix_ms()? + 600_000;
    let compiler = NativeLimits {
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
    let mut auxiliary = BTreeMap::new();
    let mut failures = Vec::new();
    let mut declarations = Vec::new();
    for name in catalog.families.keys().map(String::as_str).chain(["pause-direct-animation"]) {
        let result = (|| -> Result<NativeContract> {
            let path = if name == "pause-direct-animation" {
                PathBuf::from(".scratch/decomp-coverage/renderer/direct-animation.toml")
            } else {
                batch::entry(&catalog, name)?.manifest
            };
            let contract = offline.declare_native(NativeDeclaration {
                checkout: &checkout,
                kind: if name == "pause-direct-animation" {
                    NativeDeclarationKind::DirectAnimation(&path)
                } else {
                    NativeDeclarationKind::Family(&path)
                },
                source_inventory: inventory.clone(),
                provenance: provenance.clone(),
                compiler_limits: compiler.clone(),
                fixture_limits: fixture.clone(),
                deadline_unix_ms: Some(deadline),
                max_input_bytes: 8 * bounded::DEFAULT_BYTES,
            })?;
            if name == "pause-direct-animation" {
                ensure!(
                    contract.family == name && contract.coverage.values().map(Vec::len).sum::<usize>() == 360,
                    "auxiliary animation domain changed"
                );
            }
            Ok(contract)
        })();
        match result {
            Ok(contract) => declarations.push((name.to_owned(), contract)),
            Err(error) => {
                let message = format!("rejected: {error:#}");
                if name == "pause-direct-animation" {
                    auxiliary.insert(name.to_owned(), message);
                } else {
                    summary.insert(name.to_owned(), message);
                }
                failures.push(format!("{name}: {error:#}"));
            }
        }
    }
    let requests = declarations
        .iter()
        .map(|(_, contract)| NativePreparation {
            deadline_unix_ms: Some(deadline),
            max_input_bytes: 8 * bounded::DEFAULT_BYTES,
            ..NativePreparation::new(&checkout, pinned, contract)
        })
        .collect::<Vec<_>>();
    for ((name, contract), prepared) in
        declarations.iter().zip(offline.prepare_native_catalog(&requests, &NoScopeAdmission)?)
    {
        let result = prepared.and_then(|prepared| {
            immutable(&root.join(format!("{name}.toml")), toml::to_string(&prepared)?.as_bytes())?;
            Ok(contract.coverage.values().map(Vec::len).sum::<usize>())
        });
        let message = match result {
            Ok(cases) => format!("prepared {cases} fixture pairs"),
            Err(error) => {
                failures.push(format!("{name}: {error:#}"));
                format!("rejected: {error:#}")
            }
        };
        if name == "pause-direct-animation" {
            auxiliary.insert(name.clone(), message);
        } else {
            summary.insert(name.clone(), message);
        }
        atomic_toml(&root.join("summary.toml"), &summary)?;
        atomic_toml(&root.join("auxiliary-summary.toml"), &auxiliary)?;
    }
    ensure!(summary.len() == catalog.families.len() && auxiliary.len() == 1, "catalog entry omitted");
    cleanup_proof(&root)?;
    ensure!(failures.is_empty(), "{}", failures.join("; "));
    Ok(())
}

fn variant_source(prepared: &PreparedNative, original: &Path, destination: &Path) -> Result<PathBuf> {
    fs::create_dir_all(destination)?;
    for name in prepared.contract().frozen_inputs.keys() {
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
    let preparation_started = std::time::Instant::now();
    crate::initialize_test_binary_identity()?;
    let offline = crate::offline::families()?;
    let proof = PathBuf::from(std::env::var("SM_NATIVE_PROOF_ROOT")?);
    let root = proof.join(std::env::var("SM_NATIVE_PROOF_NAME").unwrap_or_else(|_| "actual-c".into()));
    fs::create_dir_all(&root)?;
    let current = std::env::var_os("SM_NATIVE_FULL_SOURCE").map(PathBuf::from).unwrap_or_else(current_checkout);
    let checkout = root.join("source");
    fs::create_dir_all(checkout.join("crates/pipeline/src"))?;
    fs::create_dir_all(checkout.join("crates/port/src"))?;
    fs::create_dir_all(checkout.join("tools/port"))?;
    let manifest_path = Path::new("tools/port/enemy-jump-instructions.smf");
    immutable(&checkout.join(manifest_path), &bounded::read(current.join(manifest_path))?)?;
    crate::translation_execution::tests::copy_native_source_snapshot(&current, &checkout)?;
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
    let catalog = root.join("catalog.toml");
    immutable(&catalog, toml::to_string(&batch::catalog(&checkout)?)?.as_bytes())?;
    let requirements = root.join("requirements.md");
    immutable(
            &requirements,
            b"Verify every enemy-jump-instructions Fixture pair against pinned C, with current producer attribution, bounded execution and retry controls.\n",
        )?;
    let provenance = NativeProvenance::ReviewedCatalog {
        catalog: Asset { file: catalog.clone(), sha256: digest(&catalog)? },
        requirements: Asset { file: requirements.clone(), sha256: digest(&requirements)? },
        checkout: checkout.clone(),
        rationale: "Complete current native fixture domain and execution controls".into(),
    };
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
    let contract = offline.declare_native(NativeDeclaration {
        checkout: &checkout,
        kind: NativeDeclarationKind::Family(manifest_path),
        source_inventory: Asset { file: inventory_path.clone(), sha256: digest(&inventory_path)? },
        provenance,
        compiler_limits: limits,
        fixture_limits,
        deadline_unix_ms: None,
        max_input_bytes: bounded::DEFAULT_BYTES,
    })?;
    let prepared = prepare(&checkout, pinned, &contract)?;
    let mut missing = contract.clone();
    missing.coverage.get_mut("Sidehopper_Func_1").unwrap().pop();
    ensure!(prepare(&checkout, pinned, &missing).is_err(), "incomplete prescribed coverage admitted");
    let mut missing = contract.clone();
    missing.frozen_inputs.remove("crates/port/src/memory.rs");
    ensure!(prepare(&checkout, pinned, &missing).is_err(), "missing canonical harness dependency admitted");
    let mut missing = contract.clone();
    let NativeProvenance::ReviewedCatalog { catalog, .. } = &mut missing.provenance else { unreachable!() };
    catalog.sha256 = "0".repeat(64);
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
        policy_sha256: prepared.verification_sha256().to_owned(),
        operation_key: sha256("native-real-family-first"),
        phase: translation_execution::model::ExecutionPhase::Candidate,
    };
    let preparation_seconds = preparation_started.elapsed().as_secs_f64();
    let fixture_jobs = std::env::var("SM_NATIVE_FIXTURE_JOBS")
        .ok()
        .map(|value| value.parse::<usize>())
        .transpose()?
        .unwrap_or(runtime.workers);
    let started = std::time::Instant::now();
    let receipt = run_bounded(&prepared, &checkout, &root.join("run"), &runtime, &context, fixture_jobs)?;
    let run_seconds = started.elapsed().as_secs_f64();
    retain_case(
        &root,
        "run",
        RetainedCase {
            prepared: root.join("prepared.toml"),
            checkout: checkout.clone(),
            receipt: root.join("run/native-receipt.toml"),
            passed: true,
            fixture_pairs: Some(16),
            operation_count: Some(34),
            diagnostic_contains: None,
        },
    )?;
    ensure!(receipt.passed, "{}", receipt.diagnostic);
    let authentication_started = std::time::Instant::now();
    let admitted = prepared.authenticate(NativeEvidence { checkout: &checkout, receipt: &receipt })?;
    let verdict = admitted.report();
    let authentication_seconds = authentication_started.elapsed().as_secs_f64();
    ensure!(
        verdict.verified_cases == 16 && verdict.verified_routines == 2 && receipt.operation_count()? == 34,
        "real family coverage differs"
    );
    let mut before = Vec::new();
    let operations = proof_producers(&admitted, &receipt)?;
    let mut invocation_times = BTreeMap::new();
    for operation in &operations {
        before.push((
            operation.result.path.clone(),
            operation.result.sha256.clone(),
            fs::metadata(&operation.result.path)?.modified()?,
        ));
        invocation_times.insert(operation.invocation.file.clone(), operation.invocation.file.metadata()?.modified()?);
    }
    let replay_started = std::time::Instant::now();
    let retry = run(&prepared, &checkout, &root.join("run"), &runtime, &context)?;
    let replay_seconds = replay_started.elapsed().as_secs_f64();
    ensure!(sealed(&retry)? == sealed(&receipt)?, "lost response retry allocated another result");
    for (path, hash, modified) in before {
        ensure!(
            fs::metadata(&path)?.modified()? == modified && digest(&path)? == hash,
            "retry rewrote a producer result"
        );
    }
    // Hash-only family tampering cannot bypass the independent durable
    // producer authenticators, even if a forged receipt copies a new hash.
    let operation = operations
        .iter()
        .find(|operation| operation.label == "rust-build")
        .context("retained Rust compiler operation missing")?;
    let invocation_bytes = bounded::read(&operation.invocation.file)?;
    let mut changed: toml::Value = bounded::object(&operation.invocation.file)?;
    changed
        .get_mut("program")
        .and_then(|program| program.get_mut("arguments"))
        .and_then(toml::Value::as_array_mut)
        .context("retained compiler arguments missing")?
        .push(toml::Value::String("--undeclared".into()));
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
    if std::env::var_os("SM_NATIVE_PERFORMANCE_ONLY").is_some() {
        #[derive(Serialize)]
        struct ProducerTiming {
            label: String,
            directory: PathBuf,
            seconds: f64,
            identity_import_seconds: f64,
            runner_startup_seconds: f64,
            sandbox_and_finalization_seconds: f64,
            parent_publication_seconds: f64,
        }
        #[derive(Serialize)]
        struct Performance {
            fixture_pairs: usize,
            routines: usize,
            operation_count: usize,
            fixture_jobs: usize,
            evidence_bytes: u64,
            preparation_seconds: f64,
            run_seconds: f64,
            authentication_seconds: f64,
            replay_seconds: f64,
            references_seconds: f64,
            producer_seconds: f64,
            compile_seconds: f64,
            fixture_seconds: f64,
            fixture_span_seconds: f64,
            producers: Vec<ProducerTiming>,
        }
        let references_started = std::time::Instant::now();
        receipt.retained_references()?;
        let references_seconds = references_started.elapsed().as_secs_f64();
        let mut producers = Vec::new();
        let mut compile_seconds = 0.0;
        let mut fixture_seconds = 0.0;
        let mut fixture_first = None;
        let mut fixture_last = None;
        let elapsed = |first: &Path, last: &Path| -> Result<f64> {
            let first = invocation_times.get(first).copied().map_or_else(|| first.metadata()?.modified(), Ok)?;
            Ok(last.metadata()?.modified()?.duration_since(first)?.as_secs_f64())
        };
        for operation in &operations {
            let timing_path = operation.directory.join(format!("{}.timing.toml", operation.label));
            let timing: tool_execution::builds::ExecutionRecord = bounded::object(&timing_path)?;
            let directory = operation.result.path.parent().context("operation result parent missing")?;
            if operation.directory == verdict.run {
                compile_seconds += timing.seconds;
            } else {
                fixture_seconds += timing.seconds;
                let first = invocation_times[&operation.invocation.file];
                let last = timing_path.metadata()?.modified()?;
                fixture_first = Some(fixture_first.map_or(first, |previous: SystemTime| previous.min(first)));
                fixture_last = Some(fixture_last.map_or(last, |previous: SystemTime| previous.max(last)));
            }
            producers.push(ProducerTiming {
                label: operation.label.clone(),
                directory: operation.directory.clone(),
                seconds: timing.seconds,
                identity_import_seconds: elapsed(&operation.invocation.file, &directory.join("intent.json"))?,
                runner_startup_seconds: elapsed(&directory.join("intent.json"), &directory.join("sandbox.json"))?,
                sandbox_and_finalization_seconds: elapsed(&directory.join("sandbox.json"), &operation.result.path)?,
                parent_publication_seconds: elapsed(&operation.result.path, &timing_path)?,
            });
        }
        atomic_toml(
            &root.join("performance.toml"),
            &Performance {
                fixture_pairs: verdict.verified_cases,
                routines: verdict.verified_routines,
                operation_count: operations.len(),
                fixture_jobs,
                evidence_bytes: receipt.evidence_bytes,
                preparation_seconds,
                run_seconds,
                authentication_seconds,
                replay_seconds,
                references_seconds,
                producer_seconds: compile_seconds + fixture_seconds,
                compile_seconds,
                fixture_seconds,
                fixture_span_seconds: fixture_last
                    .context("missing last fixture")?
                    .duration_since(fixture_first.context("missing first fixture")?)?
                    .as_secs_f64(),
                producers,
            },
        )?;
        cleanup_proof(&root)?;
        authenticate_retained(&root.join("retained-authentication.toml"))?;
        return Ok(());
    }
    let capture_source = variant_source(&prepared, &checkout, &root.join("capture-refusal-source"))?;
    let candidate = capture_source.join(prepared.candidate_path());
    let candidate_bytes = bounded::read(&candidate)?;
    let undeclared = root.join("compiler-secret.txt");
    fs::write(&undeclared, "protected input")?;
    let compile_read = format!("\nconst UNDECLARED:&str=include_str!({:?});\n", undeclared);
    let mut mutated = candidate_bytes.clone();
    mutated.extend(compile_read.as_bytes());
    fs::write(&candidate, &mutated)?;
    let mut deny_context = context.clone();
    deny_context.operation_key = sha256("native-uncaptured-input-refusal");
    let capture_output = root.join("capture-refusal");
    let producers_before = bounded::entries(&runtime.operation_root)?;
    let error = run(&prepared, &capture_source, &capture_output, &runtime, &deny_context)
        .err()
        .context("uncaptured file macro reached Native producer execution")?;
    ensure!(
        bounded::entries(&runtime.operation_root)? == producers_before,
        "refused source capture created a producer operation"
    );
    let refusal = RetainedCaptureRefusal {
        candidate: Asset { file: candidate.clone(), sha256: digest(&candidate)? },
        output: capture_output,
        diagnostic: format!("{error:#}"),
        kind: CaptureRefusalKind::FileMacro,
    };
    authenticate_capture_refusal(&refusal)?;
    ensure!(
        refusal.diagnostic.contains(undeclared.to_str().context("uncaptured input path is not UTF-8")?),
        "capture refusal did not identify the undeclared input"
    );
    retain_proof(&root, |proof| {
        proof.capture_refusals.insert("uncaptured-file-macro".into(), refusal);
    })?;
    println!("Uncaptured include_str! refused before Native intent or producer publication");

    // Source capture refuses unknown inputs before compilation. Exercise the
    // actual confinement boundary separately with an explicitly bound probe.
    prove_compiler_denial(&root, &runtime, &context)?;

    // A complete source closure can still fail in rustc. This independently
    // preserves rejected Native receipts and their exact producer replay.
    let compile_source = variant_source(&prepared, &checkout, &root.join("compile-failure-source"))?;
    let candidate = compile_source.join(prepared.candidate_path());
    let mut mutated = candidate_bytes.clone();
    mutated.extend(b"\ncompile_error!(\"native compiler failure control\");\n");
    fs::write(&candidate, &mutated)?;
    let mut deny_context = context.clone();
    deny_context.operation_key = sha256("native-compiler-failure-replay");
    let compile_output = root.join("compile-failure");
    let denied = run(&prepared, &compile_source, &compile_output, &runtime, &deny_context)?;
    ensure!(
        !denied.passed
            && denied.diagnostic.contains("native compiler failure control")
            && denied.operation_count()? == 2,
        "native compiler failure was not retained: {}",
        denied.diagnostic
    );
    let admitted = prepared.authenticate(NativeEvidence { checkout: &compile_source, receipt: &denied })?;
    let operations = proof_producers(&admitted, &denied)?;
    let producer_stamps_before = operations
        .iter()
        .map(|producer| producer_stamps(producer.result.path.parent().context("producer result parent missing")?))
        .collect::<Result<Vec<_>>>()?;
    let receipt_path = compile_output.join("native-receipt.toml");
    let receipt_before = (digest(&receipt_path)?, fs::metadata(&receipt_path)?.modified()?);
    let retry = run(&prepared, &compile_source, &compile_output, &runtime, &deny_context)?;
    ensure!(sealed(&retry)? == sealed(&denied)?, "unchanged rejected candidate returned another Native receipt");
    ensure!(
        (digest(&receipt_path)?, fs::metadata(&receipt_path)?.modified()?) == receipt_before,
        "rejected Native retry rewrote its receipt"
    );
    for (producer, before) in operations.iter().zip(producer_stamps_before) {
        ensure!(
            producer_stamps(producer.result.path.parent().context("producer result parent missing")?)? == before,
            "rejected Native retry reran or rewrote a compiler producer"
        );
    }
    retain_case(
        &root,
        "compile-failure",
        RetainedCase {
            prepared: root.join("prepared.toml"),
            checkout: compile_source.clone(),
            receipt: receipt_path,
            passed: false,
            fixture_pairs: None,
            operation_count: Some(2),
            diagnostic_contains: Some("native compiler failure control".into()),
        },
    )?;
    authenticate(&prepared, &compile_source, &denied)?;

    let runtime_source = variant_source(&prepared, &checkout, &root.join("runtime-refusal-source"))?;
    let candidate = runtime_source.join(prepared.candidate_path());
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
    deny_context.operation_key = sha256("native-runtime-capability-refusal");
    let runtime_output = root.join("runtime-refusal");
    let producers_before = bounded::entries(&runtime.operation_root)?;
    let error = run(&prepared, &runtime_source, &runtime_output, &runtime, &deny_context)
        .err()
        .context("unmodeled runtime capability reached Native producer execution")?;
    ensure!(
        bounded::entries(&runtime.operation_root)? == producers_before
            && bounded::text(&runtime_secret)? == "protected parent verdict",
        "refused runtime capability created a producer or changed the protected parent"
    );
    let refusal = RetainedCaptureRefusal {
        candidate: retained_asset(&candidate)?,
        output: runtime_output,
        diagnostic: format!("{error:#}"),
        kind: CaptureRefusalKind::RuntimeCapability,
    };
    authenticate_capture_refusal(&refusal)?;
    retain_proof(&root, |proof| {
        proof.capture_refusals.insert("unmodeled-runtime-capability".into(), refusal);
    })?;
    println!("Injected std::fs capabilities refused before Native intent or producer publication");

    // Preserve the second complete actual-C family evaluation. Its admitted
    // Rust invocations prescribe each diagnostic probe's exact namespace.
    let runtime_source = variant_source(&prepared, &checkout, &root.join("runtime-family-source"))?;
    let mut runtime_context = context.clone();
    runtime_context.operation_key = sha256("native-runtime-fixture-namespaces");
    let runtime_receipt = run(&prepared, &runtime_source, &root.join("runtime-family"), &runtime, &runtime_context)?;
    ensure!(
        runtime_receipt.passed && runtime_receipt.operation_count()? == 34,
        "second complete Native runtime family failed: {}",
        runtime_receipt.diagnostic
    );
    let runtime_case = RetainedCase {
        prepared: root.join("prepared.toml"),
        checkout: runtime_source,
        receipt: root.join("runtime-family/native-receipt.toml"),
        passed: true,
        fixture_pairs: Some(16),
        operation_count: Some(34),
        diagnostic_contains: None,
    };
    retain_case(&root, "runtime-family", runtime_case.clone())?;
    prove_runtime_probes(&root, "runtime-family", &runtime_case, &runtime)?;
    let mut deny_context = context.clone();
    deny_context.operation_key = sha256("native-output-reservation-denial");
    let intent: toml::Value = bounded::object(&root.join("run/native-intent.toml"))?;
    let parent_bytes = intent
        .get("parent_reservation_bytes")
        .and_then(toml::Value::as_integer)
        .context("recorded parent reservation missing")?;
    let narrow = StrictRuntime {
        max_output_bytes: u64::try_from(parent_bytes)?.checked_add(4096).context("proof reservation overflow")?,
        ..runtime
    };
    let denied = run(&prepared, &checkout, &root.join("reservation-denial"), &narrow, &deny_context)?;
    retain_case(
        &root,
        "reservation-denial",
        RetainedCase {
            prepared: root.join("prepared.toml"),
            checkout: checkout.clone(),
            receipt: root.join("reservation-denial/native-receipt.toml"),
            passed: false,
            fixture_pairs: None,
            operation_count: Some(0),
            diagnostic_contains: Some("reservation exhausted".into()),
        },
    )?;
    ensure!(
        !denied.passed && denied.operation_count()? == 0 && denied.diagnostic.contains("reservation exhausted"),
        "aggregate reservation did not stop before compiler launch"
    );
    authenticate(&prepared, &checkout, &denied)?;
    cleanup_proof(&root)?;
    authenticate_retained(&root.join("retained-authentication.toml"))?;
    println!(
        "Preparation gaps, input/capability capture refusals, actual compiler file denial, rejected Native replay, all 16 runtime namespace probes, tampered operation and aggregate prelaunch reservation checks passed"
    );
    Ok(())
}
