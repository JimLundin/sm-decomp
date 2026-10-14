use super::*;
use std::{
    fs,
    sync::atomic::{AtomicU64, Ordering},
};
use translation_execution::model::sha256;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "sm-cargo-adapter-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn host_contract(root: &Path) -> CargoContract {
    let files = [
        ("Cargo.toml", "[workspace]\nmembers = [\"crates/port\"]\nresolver = \"3\"\n"),
        ("Cargo.lock", "version = 4\n[[package]]\nname = \"port\"\nversion = \"0.1.0\"\n"),
        ("crates/port/Cargo.toml", "[package]\nname = \"port\"\nversion = \"0.1.0\"\nedition = \"2024\"\n"),
        ("crates/port/src/lib.rs", "pub fn preserved_candidate() {}\n"),
        ("crates/port/tests/game_host.rs", include_str!("../../port/tests/game_host.rs")),
        ("crates/port/tests/room_scroll.rs", include_str!("../../port/tests/room_scroll.rs")),
        ("crates/port/tests/samus_projectile_frame.rs", include_str!("../../port/tests/samus_projectile_frame.rs")),
    ];
    let mut source_files = BTreeMap::new();
    for (name, source) in files {
        let path = root.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, source).unwrap();
        source_files.insert(name.to_owned(), sha256(source));
    }
    let vendor = root.join("vendor");
    fs::create_dir(&vendor).unwrap();
    CargoContract {
        schema: "translation-cargo-contract-v1".into(),
        kind: CargoKind::SharedGameHost,
        source_files,
        approved_files: vec!["crates/port/src/lib.rs".into()],
        vendor: CargoClosure { root: vendor, files: BTreeMap::new() },
        dependency_cache: None,
        limits: CargoLimits {
            timeout_seconds: 60,
            writable_bytes: 256 * 1024 * 1024,
            stdout_bytes: 65536,
            stderr_bytes: 65536,
        },
    }
}

#[test]
fn preparation_prescribes_all_original_controls_before_accepting_a_source_closure() {
    let scratch = Scratch::new();
    let mut contract = host_contract(&scratch.0);
    prepare(&scratch.0, &contract).expect("the complete frozen original control inventory is admissible");
    let name = "crates/port/tests/game_host.rs";
    let reduced = fs::read_to_string(scratch.0.join(name))
        .unwrap()
        .replace("fn first_failing_request_retains_rust_ownership()", "fn removed_original_control()");
    fs::write(scratch.0.join(name), &reduced).unwrap();
    contract.source_files.insert(name.into(), sha256(&reduced));
    let error = prepare(&scratch.0, &contract).unwrap_err();
    assert!(error.to_string().contains("original Cargo control is missing"), "{error:#}");
}

fn collect_fixture(source: &Path, destination: Option<&Path>, names: &mut BTreeMap<String, String>, prefix: &Path) {
    let mut entries = fs::read_dir(source).unwrap().map(|entry| entry.unwrap().path()).collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        let leaf = path.file_name().unwrap();
        if [".git", ".scratch"].iter().any(|name| leaf == *name) {
            continue;
        }
        let name = prefix.join(leaf);
        let metadata = path.symlink_metadata().unwrap();
        if metadata.is_dir() {
            collect_fixture(&path, destination, names, &name);
        } else {
            assert!(metadata.is_file(), "fixture input is not regular: {}", path.display());
            let bytes = fs::read(&path).unwrap();
            if let Some(destination) = destination {
                let target = destination.join(&name);
                fs::create_dir_all(target.parent().unwrap()).unwrap();
                fs::write(target, &bytes).unwrap();
            }
            names.insert(name.to_str().unwrap().to_owned(), sha256(bytes));
        }
    }
}

#[test]
#[ignore = "real confined original Cargo proof; requires the installed producer and private offline vendor closure"]
fn real_original_cargo_controls_retain_authenticatable_success_and_unchanged_replay() {
    let root = PathBuf::from(std::env::var_os("SM_CARGO_PROOF_ROOT").expect("SM_CARGO_PROOF_ROOT"));
    let current = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let checkout = root.join("source");
    fs::create_dir_all(&checkout).unwrap();
    let mut sources = BTreeMap::new();
    collect_fixture(&current.join("crates"), Some(&checkout), &mut sources, Path::new("crates"));
    for name in ["Cargo.toml", "Cargo.lock"] {
        let bytes = fs::read(current.join(name)).unwrap();
        fs::write(checkout.join(name), &bytes).unwrap();
        sources.insert(name.to_owned(), sha256(bytes));
    }
    let deps = fs::canonicalize(current.join("deps")).unwrap();
    collect_fixture(&deps.join("rustysnes/crates"), Some(&checkout), &mut sources, Path::new("deps/rustysnes/crates"));
    let name = "deps/rustysnes/Cargo.toml";
    let bytes = fs::read(deps.join("rustysnes/Cargo.toml")).unwrap();
    fs::write(checkout.join(name), &bytes).unwrap();
    sources.insert(name.into(), sha256(bytes));
    let vendor = PathBuf::from(std::env::var_os("SM_CARGO_VENDOR").expect("SM_CARGO_VENDOR"));
    let mut vendor_files = BTreeMap::new();
    collect_fixture(&vendor, None, &mut vendor_files, Path::new(""));
    let contract = CargoContract {
        schema: "translation-cargo-contract-v1".into(),
        kind: CargoKind::SharedGameHost,
        source_files: sources,
        approved_files: vec!["crates/port/src/game_host.rs".into(), "crates/port/src/lib.rs".into()],
        vendor: CargoClosure { root: vendor, files: vendor_files },
        dependency_cache: None,
        limits: CargoLimits {
            timeout_seconds: 60,
            writable_bytes: 256 * 1024 * 1024,
            stdout_bytes: 256 * 1024,
            stderr_bytes: 65536,
        },
    };
    let prepared = prepare(&checkout, &contract).unwrap();
    fs::write(root.join("prepared.json"), serde_json::to_vec_pretty(&prepared).unwrap()).unwrap();
    let runtime = StrictRuntime {
        helper: std::env::var_os("SM_CARGO_HELPER")
            .map(PathBuf::from)
            .unwrap_or_else(|| "/home/jim/sm-decomp-production-execution/bin/translation-sandbox".into()),
        rustc: "/home/jim/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/rustc".into(),
        cargo: "/home/jim/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/cargo".into(),
        producer: std::env::var_os("SM_CARGO_PRODUCER")
            .map(PathBuf::from)
            .unwrap_or_else(|| "/tmp/sm-ticket10-cargo-target/debug/sm-pipeline".into()),
        operation_root: root.join("operations"),
        pipeline_root: "/home/jim/sm-decomp/out/pipeline".into(),
        workers: 4,
        gate_timeout_seconds: 60,
        max_output_bytes: 1024 * 1024 * 1024,
        deadline_unix_ms: None,
    };
    let context = OperationContext {
        run_id: sha256("ticket10-original-cargo-success"),
        policy_sha256: sha256("ticket10-original-cargo-policy"),
        operation_key: sha256("ticket10-original-cargo-candidate"),
        attempt: 1,
        phase: translation_execution::model::ExecutionPhase::Candidate,
    };
    let output = root.join("candidate");
    let mut environment_context = context.clone();
    environment_context.operation_key = sha256("ticket10-original-cargo-runtime-preparation");
    environment_context.phase = translation_execution::model::ExecutionPhase::Preparation;
    let environment = prepare_environment(
        &prepared,
        &checkout,
        &root.join("environment"),
        &runtime,
        &environment_context,
        180,
        false,
    )
    .unwrap();
    assert!(environment.passed, "{}", environment.diagnostic);
    authenticate_environment(&prepared, &checkout, &environment).unwrap();
    let receipt = run(&prepared, &checkout, &output, &runtime, &context).unwrap();
    assert!(receipt.passed, "{}", receipt.diagnostic);
    let verdict = authenticate(&prepared, &checkout, &receipt).unwrap();
    assert_eq!(verdict.test_count, 31);
    assert_eq!(verdict.operations.len(), 1);
    let replay = run(&prepared, &checkout, &output, &runtime, &context).unwrap();
    assert_eq!(replay, receipt);
    assert_eq!(authenticate(&prepared, &checkout, &replay).unwrap(), verdict);
}
