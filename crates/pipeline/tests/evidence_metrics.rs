//! Evidence CLI metrics preserve stored identities and diagnose failures separately.
use evidence::{ImportReceipt, Inventory, Pointer, Store, TraceReceipt, hex, parse_id, read_stream};
use pipeline_model::metrics::{Clock, ContextReceipt, Measurement, Receipt};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

const TRACE: &[u8] =
    b"plm 65535 -2 0 255\nppu 7 1 255 0\napu 0 4294967295 255\nppu 0 0 0 255\nunknown 3 exact bytes\ntrailing literal";
const CAPTURE_STAGES: &[&str] = &[
    "capture-store-open",
    "collector-ready-flush",
    "trace-ingest",
    "trace-commit",
    "trace-receipt-fsync",
    "staged-store-open",
    "staged-tree-ingest",
    "final-commit",
    "reopened-store-open",
    "reopened-verification",
    "candidate-pointer-fsync",
];
const IMPORT_STAGES: &[&str] = &[
    "import-source-open",
    "import-source-verification",
    "import-destination-open",
    "shared-import",
    "import-commit",
    "import-destination-reopen",
    "import-destination-verification",
];
const INVENTORY_STAGES: &[&str] = &["inventory-store-open", "inventory-verification"];

struct Run(PathBuf);
impl Run {
    fn new() -> Self {
        let root =
            std::env::var_os("SM_EVIDENCE_METRICS_TEST_ROOT").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        let path = root.join(format!(
            "evidence-metrics-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn context(&self, mode: &str, enabled: bool, path: PathBuf) -> PathBuf {
        let file = self.0.join(format!("{mode}-context.toml"));
        let context = ContextReceipt {
            run_id: "fixed-run".into(),
            scope_id: mode.into(),
            parent_scope: Some("parent".into()),
            clock: Clock::new().unwrap(),
            enabled,
            path,
        };
        fs::write(&file, toml::to_string(&context).unwrap()).unwrap();
        file
    }
    fn capture_root(&self, name: &str) -> PathBuf {
        let root = self.0.join(name);
        fs::create_dir_all(root.join("stage/fixtures")).unwrap();
        fs::write(root.join("stage/terminal.bin"), b"exact state").unwrap();
        fs::write(root.join("stage/fixtures/state.wram"), b"\0\xff\x01\x80").unwrap();
        fs::write(root.join("provenance.toml"), "schema_version=2\n").unwrap();
        fs::write(root.join("input.txt"), TRACE).unwrap();
        root
    }
}
impl Drop for Run {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn reader() -> PathBuf {
    std::env::var_os("SM_EVIDENCE_TEST_BINARY").map(PathBuf::from).unwrap_or_else(|| {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().join("target/debug/sm-evidence")
    })
}
fn command(context: Option<&Path>) -> Command {
    let mut command = Command::new(reader());
    command.env_remove("SM_PIPELINE_METRICS_CONTEXT").stdin(Stdio::null());
    if let Some(context) = context {
        command.env("SM_PIPELINE_METRICS_CONTEXT", context);
    }
    command
}
fn capture(root: &Path, context: Option<&Path>) -> Output {
    command(context)
        .arg("capture")
        .arg(root.join("store"))
        .arg(root.join("trace.toml"))
        .arg(root.join("stage"))
        .arg(root.join("provenance.toml"))
        .arg(root.join("pointer.toml"))
        .stdin(fs::File::open(root.join("input.txt")).unwrap())
        .output()
        .unwrap()
}
fn read<T: DeserializeOwned>(path: &Path) -> T {
    toml::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}
fn success(output: &Output) {
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}
fn receipt(path: &Path, role: &str, enabled: bool) -> Receipt {
    let receipt: Receipt = read(path);
    receipt.validate().unwrap();
    assert_eq!(receipt.run_id, "fixed-run");
    assert_eq!(receipt.parent_scope.as_deref(), Some("parent"));
    assert_eq!(receipt.role, role);
    assert_eq!(receipt.enabled, enabled);
    assert!(receipt.processes.is_empty());
    assert!(
        matches!(receipt.resources.aggregate_sampled_rss_bytes, Measurement::Unavailable { ref reason } if !reason.is_empty())
    );
    if !enabled {
        assert!(receipt.stages.is_empty());
        assert_eq!(receipt.configuration.get("telemetry").map(String::as_str), Some("disabled"));
    }
    receipt
}
fn successful_stages(receipt: &Receipt, expected: &[&str]) {
    assert_eq!(receipt.outcome, "success");
    assert!(receipt.error.is_none());
    if receipt.enabled {
        let names: Vec<_> = receipt.stages.iter().map(|stage| stage.name.as_str()).collect();
        assert_eq!(names, expected);
        assert_eq!(receipt.last_completed_stage.as_deref(), expected.last().copied());
        for stage in &receipt.stages {
            assert_eq!(stage.parent.as_deref(), Some(receipt.scope_id.as_str()));
            assert_eq!(stage.outcome, "success");
        }
    }
}
fn failed_receipt(path: &Path, role: &str, completed: &str, failed_stage: Option<&str>) {
    let receipt = receipt(path, role, true);
    assert_eq!(receipt.outcome, "failed");
    assert!(receipt.error.as_deref().is_some_and(|error| !error.is_empty()));
    assert_eq!(receipt.last_completed_stage.as_deref(), Some(completed));
    let last = receipt.stages.last().unwrap();
    assert_eq!(last.name, failed_stage.unwrap_or(completed));
    assert_eq!(last.outcome, if failed_stage.is_some() { "failed" } else { "success" });
}
fn member(store: &Path, pointer: &Pointer, name: &str) -> Vec<u8> {
    let output = command(None).arg("cat").arg(store).arg(&pointer.run).arg(name).output().unwrap();
    success(&output);
    output.stdout
}

#[test]
fn disabled_capture_keeps_exact_evidence_and_marks_optional_telemetry_unavailable() {
    let run = Run::new();
    let mut identities = Vec::new();
    let mut streams = Vec::new();
    for (mode, enabled) in [("enabled", Some(true)), ("disabled", Some(false)), ("absent", None)] {
        let root = run.capture_root(mode);
        let context = enabled.map(|enabled| run.context(mode, enabled, root.join("metrics.toml")));
        let output = capture(&root, context.as_deref());
        success(&output);
        assert_eq!(output.stdout, b"ready\n");
        let pointer: Pointer = read(&root.join("pointer.toml"));
        assert_eq!(pointer.files, 3);
        assert_eq!(
            pointer.artifacts.keys().map(String::as_str).collect::<Vec<_>>(),
            ["fixtures/state.wram", "hardware.log", "terminal.bin"]
        );
        assert_eq!(member(&root.join("store"), &pointer, "hardware.log"), TRACE);
        assert_eq!(pointer.artifacts["hardware.log"], hex(&Sha256::digest(TRACE).into()));
        assert_eq!(pointer.sizes["hardware.log"], TRACE.len() as u64);
        let trace: TraceReceipt = read(&root.join("trace.toml"));
        assert_eq!(trace.bytes, TRACE.len() as u64);
        assert_eq!(trace.sha256, pointer.artifacts["hardware.log"]);
        let store = Store::open(&root.join("store")).unwrap();
        let info = read_stream(&store, &parse_id(&trace.stream).unwrap()).unwrap();
        assert_eq!(info.bytes, trace.bytes);
        assert_eq!(hex(&info.legacy_sha256), trace.sha256);
        assert_eq!(info.lines, 5);
        let frame = command(None)
            .arg("frame")
            .arg(root.join("store"))
            .arg(&pointer.run)
            .arg("hardware.log")
            .arg("7")
            .output()
            .unwrap();
        success(&frame);
        assert_eq!(frame.stdout, b"ppu 7 1 255 0\n");
        streams.push((trace.stream, info.bytes, info.lines, info.events, info.segments));
        identities.push(pointer);
        if let Some(enabled) = enabled {
            successful_stages(&receipt(&root.join("metrics.toml"), "capture", enabled), CAPTURE_STAGES);
        } else {
            assert!(!root.join("metrics.toml").exists());
        }
    }
    assert_eq!(identities[0], identities[1]);
    assert_eq!(identities[0], identities[2]);
    assert_eq!(streams[0], streams[1]);
    assert_eq!(streams[0], streams[2]);
}

#[test]
fn import_and_inventory_sidecars_preserve_authenticated_members_with_existing_destination_objects() {
    let run = Run::new();
    let source = run.capture_root("source");
    success(&capture(&source, None));
    let pointer: Pointer = read(&source.join("pointer.toml"));
    let mut inventories = Vec::new();
    for (mode, enabled) in [("enabled", true), ("disabled", false)] {
        let root = run.0.join(mode);
        fs::create_dir_all(root.join("seed")).unwrap();
        fs::write(root.join("seed/unrelated.txt"), b"distinct preexisting objects force ordinal remapping").unwrap();
        success(&command(None).arg("ingest").arg(root.join("store")).arg(root.join("seed")).output().unwrap());
        let context = run.context(&format!("import-{mode}"), enabled, root.join("import.metrics.toml"));
        let output = command(Some(&context))
            .arg("import")
            .arg(source.join("store"))
            .arg(root.join("store"))
            .arg(&pointer.run)
            .output()
            .unwrap();
        success(&output);
        let imported: ImportReceipt = toml::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
        assert_eq!(imported.run, pointer.run);
        assert_eq!(imported.observations, pointer.observations);
        assert_eq!(imported.artifacts, pointer.artifacts);
        assert_eq!(imported.provenance, pointer.provenance);
        assert_eq!((imported.files, imported.bytes), (pointer.files, pointer.bytes));
        successful_stages(&receipt(&root.join("import.metrics.toml"), "import", enabled), IMPORT_STAGES);
        let context = run.context(&format!("inventory-{mode}"), enabled, root.join("inventory.metrics.toml"));
        let output =
            command(Some(&context)).arg("inventory").arg(root.join("store")).arg(&pointer.run).output().unwrap();
        success(&output);
        let inventory: Inventory = toml::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
        assert_eq!(inventory.observations, pointer.observations);
        assert_eq!(inventory.artifacts, pointer.artifacts);
        assert_eq!(inventory.provenance, pointer.provenance);
        assert_eq!((inventory.files, inventory.bytes), (pointer.files, pointer.bytes));
        successful_stages(&receipt(&root.join("inventory.metrics.toml"), "inventory", enabled), INVENTORY_STAGES);
        assert_eq!(member(&root.join("store"), &pointer, "hardware.log"), TRACE);
        assert_eq!(member(&root.join("store"), &pointer, "fixtures/state.wram"), b"\0\xff\x01\x80");
        assert_eq!(member(&root.join("store"), &pointer, "terminal.bin"), b"exact state");
        inventories.push(output.stdout);
    }
    assert_eq!(inventories[0], inventories[1]);
}

#[test]
fn failed_capture_preserves_stage_diagnostics_and_withholds_new_candidate_pointer() {
    let run = Run::new();
    for (mode, completed, failed) in [
        ("empty", "trace-ingest", None),
        ("trace-receipt-exists", "trace-commit", Some("trace-receipt-fsync")),
        ("staged-tree-invalid", "staged-store-open", Some("staged-tree-ingest")),
        ("pointer-exists", "reopened-verification", Some("candidate-pointer-fsync")),
    ] {
        let root = run.capture_root(mode);
        match mode {
            "empty" => fs::write(root.join("input.txt"), b"").unwrap(),
            "trace-receipt-exists" => fs::write(root.join("trace.toml"), b"preserved trace receipt").unwrap(),
            "staged-tree-invalid" => {
                fs::rename(root.join("stage"), root.join("retained-stage")).unwrap();
                fs::write(root.join("stage"), b"not a directory").unwrap();
            }
            "pointer-exists" => fs::write(root.join("pointer.toml"), b"preserved pointer").unwrap(),
            _ => unreachable!(),
        };
        let context = run.context(mode, true, root.join("metrics.toml"));
        let output = capture(&root, Some(&context));
        assert!(!output.status.success());
        assert_eq!(output.stdout, b"ready\n");
        assert!(!output.stderr.is_empty());
        failed_receipt(&root.join("metrics.toml"), "capture", completed, failed);
        if mode == "pointer-exists" {
            assert_eq!(fs::read(root.join("pointer.toml")).unwrap(), b"preserved pointer");
        } else {
            assert!(!root.join("pointer.toml").exists());
        }
        if mode == "empty" {
            assert!(!root.join("trace.toml").exists());
        } else if mode == "trace-receipt-exists" {
            assert_eq!(fs::read(root.join("trace.toml")).unwrap(), b"preserved trace receipt");
        } else {
            let trace: TraceReceipt = read(&root.join("trace.toml"));
            let store = Store::open(&root.join("store")).unwrap();
            let mut retained = Vec::new();
            evidence::write_stream(&store, &parse_id(&trace.stream).unwrap(), &mut retained).unwrap();
            assert_eq!(retained, TRACE);
        }
    }
}

#[test]
fn import_and_inventory_failures_record_last_authenticated_boundary() {
    let run = Run::new();
    let source = run.capture_root("source");
    success(&capture(&source, None));
    let pointer: Pointer = read(&source.join("pointer.toml"));
    let unknown_run = "00".repeat(32);
    let metrics = run.0.join("inventory.metrics.toml");
    let context = run.context("inventory", true, metrics.clone());
    let output = command(Some(&context)).arg("inventory").arg(source.join("store")).arg(&unknown_run).output().unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    failed_receipt(&metrics, "inventory", "inventory-store-open", Some("inventory-verification"));
    let metrics = run.0.join("import-source.metrics.toml");
    let context = run.context("import-source", true, metrics.clone());
    let output = command(Some(&context))
        .arg("import")
        .arg(source.join("store"))
        .arg(run.0.join("unpublished-store"))
        .arg(&unknown_run)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!run.0.join("unpublished-store").exists());
    failed_receipt(&metrics, "import", "import-source-open", Some("import-source-verification"));
    let destination = run.0.join("destination-file");
    fs::write(&destination, b"preserved destination").unwrap();
    let metrics = run.0.join("import-destination.metrics.toml");
    let context = run.context("import-destination", true, metrics.clone());
    let output = command(Some(&context))
        .arg("import")
        .arg(source.join("store"))
        .arg(&destination)
        .arg(&pointer.run)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(fs::read(&destination).unwrap(), b"preserved destination");
    failed_receipt(&metrics, "import", "import-source-verification", Some("import-destination-open"));
}

#[test]
fn invalid_metrics_clocks_are_rejected_before_capture_readiness_or_store_creation() {
    let run = Run::new();
    for (mode, expected) in [
        ("name", "unrelated metrics clock"),
        ("domain", "unrelated metrics clock domain"),
        ("future", "clock origin is in the future"),
    ] {
        let root = run.capture_root(mode);
        let context = run.context(mode, true, root.join("metrics.toml"));
        let mut value: ContextReceipt = read(&context);
        match mode {
            "name" => value.clock.name = "process-local-instant".into(),
            "domain" => value.clock.domain = "unrelated-host-clock".into(),
            "future" => value.clock.origin_ns = i64::MAX as u64,
            _ => unreachable!(),
        }
        fs::write(&context, toml::to_string(&value).unwrap()).unwrap();
        let output = capture(&root, Some(&context));
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected));
        for name in ["store", "trace.toml", "pointer.toml", "metrics.toml", "metrics.metrics.pending"] {
            assert!(!root.join(name).exists(), "unexpected {name} after rejecting {mode} clock");
        }
    }
}
