//! Supplementary crash recovery controls at the compact adapter API.
//!
//! These small already-prepared fixtures exercise durable finalization only.
//! They do not replace preparation or the original ROM/movie experiment.
use super::{
    CompactContract, CompactInvocation, CompactReceipt, CompactVerdict, PreparedCompact, StrictRuntime, authenticate,
    references, run,
};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use translation_execution::{
    OperationContext,
    confinement::{Program, ReadOnlyBind, RunLimits},
    model::{ExecutionPhase, sealed, sha256},
    retention,
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const TEMPLATE: &str = "crates/pipeline/src/prefix_compact_observer.lua.in";

struct Fixture {
    root: PathBuf,
    checkout: PathBuf,
    output: PathBuf,
    prepared: PreparedCompact,
    runtime: StrictRuntime,
    context: OperationContext,
    retained: bool,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let name = format!("{label}-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed));
        let retained = std::env::var_os("SM_COMPACT_RECOVERY_PROOF_ROOT");
        let root = retained
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("sm-compact-recovery-tests"))
            .join(name);
        let checkout = root.join("checkout");
        let output = root.join("gate");
        fs::create_dir_all(checkout.join(TEMPLATE).parent().unwrap()).unwrap();
        fs::create_dir_all(&output).unwrap();
        fs::write(checkout.join(TEMPLATE), "-- compact recovery fixture\n").unwrap();
        let native = root.join("unused-native");
        fs::write(&native, "not executable; the expired attempt must not launch\n").unwrap();
        let contract = CompactContract {
            schema: "translation-compact-contract-v1".into(),
            frames: 2240,
            timeout_seconds: 60,
            source_files: BTreeMap::from([(TEMPLATE.into(), sha256(fs::read(checkout.join(TEMPLATE)).unwrap()))]),
            private_inputs: BTreeMap::from([("lsnes".into(), retention::seal_file(&native, 1024).unwrap())]),
            c_units: vec![],
            runtime: vec![],
            historical: vec![],
        };
        let prepared = PreparedCompact {
            dependencies_sha256: sealed(&(
                "original-compact-dependencies-v1",
                &contract.source_files,
                &contract.private_inputs,
                &contract.runtime,
            ))
            .unwrap(),
            verification_sha256: sealed(&("original-compact-verification-v1", &contract)).unwrap(),
            contract,
        };
        let runtime = StrictRuntime {
            helper: root.join("unused-helper"),
            producer: root.join("unused-producer"),
            operation_root: root.join("operations"),
            pipeline_root: "/home/jim/sm-decomp/out/pipeline".into(),
            workers: 4,
            gate_timeout_seconds: 60,
            deadline_unix_ms: Some(60_001),
            max_output_bytes: 128 * 1024 * 1024,
        };
        let context = OperationContext {
            run_id: sha256("compact-recovery-fixture"),
            policy_sha256: sealed(&prepared).unwrap(),
            operation_key: sha256(label),
            attempt: 1,
            phase: ExecutionPhase::Candidate,
        };
        fs::write(output.join("experiment-deadline.toml"), "started_unix_ms = 1\ndeadline_unix_ms = 60001\n").unwrap();
        for (name, value) in [
            ("prepared.toml", toml::to_string_pretty(&prepared).unwrap()),
            ("runtime.toml", toml::to_string_pretty(&runtime).unwrap()),
            ("context.toml", toml::to_string_pretty(&context).unwrap()),
        ] {
            fs::write(root.join(name), value).unwrap();
        }
        Self { root, checkout, output, prepared, runtime, context, retained: retained.is_some() }
    }

    fn run(&self) -> anyhow::Result<CompactReceipt> {
        run(&self.prepared, &self.checkout, &self.output, &self.runtime, &self.context)
    }

    fn stopped(&self) -> CompactReceipt {
        let receipt = self.run().unwrap();
        let verdict = authenticate(&self.prepared, &self.checkout, &receipt).unwrap();
        assert!(!receipt.passed);
        assert!(receipt.diagnostic.contains("deadline exhausted before native"), "{}", receipt.diagnostic);
        assert_eq!(verdict.experiment_started_unix_ms, Some(1));
        assert_eq!(verdict.experiment_deadline_unix_ms, Some(60_001));
        assert!(verdict.operations.is_empty());
        assert!(!self.runtime.operation_root.exists());
        receipt
    }

    fn refuse(&self, result: anyhow::Result<CompactReceipt>, expected: &str) {
        let error = result.unwrap_err();
        assert!(format!("{error:#}").contains(expected), "expected {expected:?}: {error:#}");
        assert!(!self.output.join("compact-receipt.toml").exists());
        assert!(!self.runtime.operation_root.exists());
    }

    fn replace(&self, name: &str, bytes: &[u8]) {
        let path = self.output.join(name);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, bytes).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o400)).unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if !self.retained {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
}

#[test]
fn sealed_verdict_recovers_an_expired_stopped_experiment_without_relaunch() {
    let fixture = Fixture::new("expired-verdict");
    let original = fixture.stopped();
    let verdict = authenticate(&fixture.prepared, &fixture.checkout, &original).unwrap();
    let bytes = fs::read(&original.verdict.file).unwrap();
    let retained = references(&original).unwrap();
    assert_eq!(fixture.run().unwrap(), original);
    assert_eq!(fs::read(&original.verdict.file).unwrap(), bytes);
    fs::remove_file(fixture.output.join("compact-receipt.toml")).unwrap();
    // Cross the recorded millisecond deterministically so a retry that rebuilds
    // the verdict cannot accidentally republish identical timing by coincidence.
    let waited = Instant::now();
    while SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()
        <= u128::from(verdict.experiment_finished_unix_ms.unwrap())
    {
        assert!(waited.elapsed() < Duration::from_secs(1));
        std::thread::sleep(Duration::from_millis(1));
    }
    let recovered = fixture.run().expect("a sealed verdict must finish receipt publication after its deadline");
    assert_eq!(recovered, original);
    assert_eq!(fs::read(&original.verdict.file).unwrap(), bytes);
    assert_eq!(references(&recovered).unwrap(), retained);
    assert_eq!(authenticate(&fixture.prepared, &fixture.checkout, &recovered).unwrap(), verdict);
    assert!(!fixture.runtime.operation_root.exists());
    assert!(!fixture.output.join("native.zst").exists());
    assert_eq!(fixture.run().unwrap(), recovered);
}

#[test]
fn verdict_only_recovery_refuses_changed_caller_identity() {
    let fixture = Fixture::new("changed-identity");
    let original = fixture.stopped();
    fs::remove_file(fixture.output.join("compact-receipt.toml")).unwrap();

    let mut context = fixture.context.clone();
    context.attempt += 1;
    fixture.refuse(
        run(&fixture.prepared, &fixture.checkout, &fixture.output, &fixture.runtime, &context),
        "compact finalization recovery changed identity",
    );
    let mut runtime = fixture.runtime.clone();
    runtime.workers = 3;
    fixture.refuse(
        run(&fixture.prepared, &fixture.checkout, &fixture.output, &runtime, &fixture.context),
        "compact finalization recovery changed identity",
    );
    let mut prepared = fixture.prepared.clone();
    prepared.verification_sha256 = "0".repeat(64);
    fixture.refuse(
        run(&prepared, &fixture.checkout, &fixture.output, &fixture.runtime, &fixture.context),
        "compact receipt belongs to different prepared/original source",
    );
    let source = fs::read(fixture.checkout.join(TEMPLATE)).unwrap();
    fs::write(fixture.checkout.join(TEMPLATE), "changed source\n").unwrap();
    fixture.refuse(fixture.run(), "original compact candidate/source dependency changed");
    fs::write(fixture.checkout.join(TEMPLATE), source).unwrap();
    assert_eq!(fixture.run().unwrap(), original);
}

#[test]
fn verdict_only_recovery_authenticates_retained_timing_source_and_budgets() {
    let fixture = Fixture::new("changed-evidence");
    let original = fixture.stopped();
    let verdict = authenticate(&fixture.prepared, &fixture.checkout, &original).unwrap();
    let bytes = fs::read(&original.verdict.file).unwrap();
    fs::remove_file(fixture.output.join("compact-receipt.toml")).unwrap();
    type Mutation = (&'static str, fn(&mut CompactVerdict));
    let mutations: [Mutation; 7] = [
        ("compact receipt belongs to different prepared/original source", |value| {
            value.candidate_sha256 = "0".repeat(64);
        }),
        ("compact acceptance, target, limitations or aggregate budget changed", |value| {
            value.requested_frames = 320;
        }),
        ("compact retained experiment interval changed", |value| value.experiment_finished_unix_ms = Some(0)),
        ("compact retained experiment interval changed", |value| value.experiment_deadline_unix_ms = Some(60_000)),
        ("compact retained original source differs", |value| {
            value.source_snapshots.get_mut(TEMPLATE).unwrap().sha256 = "0".repeat(64);
        }),
        ("compact evidence exceeded aggregate pre-reservation", |value| value.reserved_bytes = 0),
        ("compact accepted native capture missing", |value| {
            value.passed = true;
            value.diagnostic.clear();
        }),
    ];
    for (expected, mutate) in mutations {
        let mut changed = verdict.clone();
        mutate(&mut changed);
        fixture.replace("compact-verdict.toml", toml::to_string_pretty(&changed).unwrap().as_bytes());
        fixture.refuse(fixture.run(), expected);
    }
    fixture.replace("compact-verdict.toml", &bytes);

    let intent_bytes = fs::read(fixture.output.join("compact-intent.toml")).unwrap();
    let mut intent: toml::Value = toml::from_str(std::str::from_utf8(&intent_bytes).unwrap()).unwrap();
    intent["candidate_sha256"] = toml::Value::String("0".repeat(64));
    fixture.replace("compact-intent.toml", toml::to_string_pretty(&intent).unwrap().as_bytes());
    fixture.refuse(fixture.run(), "compact durable intent differs");
    fixture.replace("compact-intent.toml", &intent_bytes);
    assert_eq!(fixture.run().unwrap(), original);
}

#[test]
fn verdict_only_recovery_refuses_unbound_operation_declarations() {
    let fixture = Fixture::new("changed-operations");
    let original = fixture.stopped();
    let verdict = authenticate(&fixture.prepared, &fixture.checkout, &original).unwrap();
    let bytes = fs::read(&original.verdict.file).unwrap();
    fs::remove_file(fixture.output.join("compact-receipt.toml")).unwrap();
    // A declaration cannot replace the operation engine's durable evidence.
    // Keep its metadata file present so these controls reach binding checks.
    fs::write(fixture.output.join("native-invocation.toml"), "").unwrap();
    let mut context = fixture.context.clone();
    context.operation_key = sealed(&(&fixture.context.operation_key, &fixture.output, "native")).unwrap();
    let invocation = CompactInvocation {
        label: "native".into(),
        context,
        program: Program {
            executable: fixture.runtime.producer.clone(),
            helper: fixture.runtime.helper.clone(),
            runtime: vec![],
            inputs: [
                (fixture.output.join("private/rom.sfc"), "/inputs/rom.sfc"),
                (fixture.output.join("private/movie.lsmv"), "/inputs/movie.lsmv"),
                (fixture.prepared.contract.private_inputs["lsnes"].path.clone(), "/inputs/lsnes"),
                (fixture.output.join("observe.lua"), "/inputs/observe.lua"),
            ]
            .map(|(source, destination)| ReadOnlyBind { source, destination: destination.into() })
            .into(),
            arguments: vec!["translation-compact-producer".into(), "native".into()],
            outputs: vec!["capture.zst".into()],
        },
        limits: RunLimits {
            timeout: Duration::from_secs(60),
            writable_bytes: 256 * 1024 * 1024,
            stdout_bytes: 65536,
            stderr_bytes: 65536,
            stdin_bytes: 0,
        },
        reserved_bytes: 0,
        result_sha256: "0".repeat(64),
        references: vec![],
    };
    type Mutation = (&'static str, fn(&mut CompactInvocation));
    let mutations: [Mutation; 4] = [
        ("compact producer operation lineage changed", |value| value.context.operation_key = "0".repeat(64)),
        ("compact producer operation lineage changed", |value| value.context.phase = ExecutionPhase::Integration),
        ("compact fixed producer recipe or bounds changed", |value| value.program.arguments.push("changed".into())),
        ("compact fixed producer recipe or bounds changed", |value| value.limits.writable_bytes += 4096),
    ];
    for (expected, mutate) in mutations {
        let mut declaration = invocation.clone();
        mutate(&mut declaration);
        let mut changed = verdict.clone();
        changed.operations.push(declaration);
        fixture.replace("compact-verdict.toml", toml::to_string_pretty(&changed).unwrap().as_bytes());
        fixture.refuse(fixture.run(), expected);
    }
    fixture.replace("compact-verdict.toml", &bytes);
    fs::remove_file(fixture.output.join("native-invocation.toml")).unwrap();
    assert_eq!(fixture.run().unwrap(), original);
}

#[test]
#[ignore = "read-only authentication of retained compact recovery fixtures after cleanup"]
fn retained_recovery_fixtures_authenticate_without_execution() {
    let root = PathBuf::from(std::env::var_os("SM_COMPACT_RECOVERY_PROOF_ROOT").unwrap());
    let mut count = 0;
    for entry in fs::read_dir(root).unwrap() {
        let root = entry.unwrap().path();
        let prepared: PreparedCompact =
            toml::from_str(&fs::read_to_string(root.join("prepared.toml")).unwrap()).unwrap();
        let runtime: StrictRuntime = toml::from_str(&fs::read_to_string(root.join("runtime.toml")).unwrap()).unwrap();
        let receipt_path = root.join("gate/compact-receipt.toml");
        let receipt_bytes = fs::read(&receipt_path).unwrap();
        let receipt: CompactReceipt = toml::from_str(std::str::from_utf8(&receipt_bytes).unwrap()).unwrap();
        let before = references(&receipt).unwrap();
        let verdict = authenticate(&prepared, &root.join("checkout"), &receipt).unwrap();
        assert!(!verdict.passed);
        assert!(verdict.operations.is_empty());
        assert_eq!(references(&receipt).unwrap(), before);
        assert_eq!(fs::read(receipt_path).unwrap(), receipt_bytes);
        assert!(!runtime.operation_root.exists());
        count += 1;
    }
    assert!(count > 0);
    println!("Authenticated {count} retained stopped recovery fixtures without execution");
}
