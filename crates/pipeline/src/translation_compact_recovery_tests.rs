//! Supplementary crash recovery controls at the compact adapter API.
//!
//! These small already-prepared fixtures exercise durable finalization only.
//! They do not replace preparation or the original ROM/movie experiment.
use super::{CompactContract, CompactReceipt, PreparedCompact, StrictRuntime, authenticate, references, run};
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use translation_execution::{
    OperationContext,
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
