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

#[test]
fn dependency_cache_accepts_installed_layouts_without_admitting_project_artifacts() {
    let scratch = Scratch::new();
    let mut contract = host_contract(&scratch.0);
    let root = scratch.0.join("dependencies");
    let external = "debug/deps/libunicode_ident-0123456789abcdef.rlib";
    let path = root.join(external);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "external dependency").unwrap();
    contract.dependency_cache = Some(CargoClosure {
        root: root.clone(),
        files: BTreeMap::from([(external.into(), sha256("external dependency"))]),
    });
    prepare(&scratch.0, &contract).expect("installed Cargo's external library layout is admissible");
    for project in [
        "debug/deps/libport-0123456789abcdef.rlib",
        "debug/.fingerprint/port-0123456789abcdef/lib-port",
        "debug/build/port-0123456789abcdef/out/generated.rs",
        "debug/build/port/0123456789abcdef/out/libport.rlib",
    ] {
        let path = root.join(project);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "project artifact").unwrap();
        contract.dependency_cache.as_mut().unwrap().files.insert(project.into(), sha256("project artifact"));
        let error = prepare(&scratch.0, &contract).unwrap_err();
        assert!(error.to_string().contains("Cargo dependency cache contains"), "{error:#}");
        contract.dependency_cache.as_mut().unwrap().files.remove(project);
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn original_cargo_refuses_compiler_jobs_outside_the_shared_capacity_before_launch() {
    let scratch = Scratch::new();
    let contract = host_contract(&scratch.0);
    let prepared = prepare(&scratch.0, &contract).unwrap();
    let mut runtime = StrictRuntime {
        helper: scratch.0.join("unused-helper"),
        rustc: scratch.0.join("unused-rustc"),
        cargo: scratch.0.join("unused-cargo"),
        producer: scratch.0.join("unused-producer"),
        operation_root: scratch.0.join("operations"),
        pipeline_root: scratch.0.join("pipeline"),
        workers: 4,
        jobs: 5,
        gate_timeout_seconds: 60,
        max_output_bytes: 1024 * 1024 * 1024,
        deadline_unix_ms: None,
    };
    let context = OperationContext {
        run_id: sha256("cargo-capacity-proof"),
        policy_sha256: sha256("cargo-capacity-policy"),
        operation_key: sha256("cargo-capacity-operation"),
        attempt: 1,
        phase: translation_execution::model::ExecutionPhase::Candidate,
    };
    for jobs in [0, 5] {
        runtime.jobs = jobs;
        let error = run(&prepared, &scratch.0, &scratch.0.join("candidate"), &runtime, &context).unwrap_err();
        assert!(error.to_string().contains("Cargo compiler jobs exceed the shared worker capacity"), "{error:#}");
        let error =
            prepare_environment(&prepared, &scratch.0, &scratch.0.join("environment"), &runtime, &context, 180, true)
                .unwrap_err();
        assert!(error.to_string().contains("Cargo compiler jobs exceed the shared worker capacity"), "{error:#}");
    }
    assert!(!runtime.operation_root.exists());
    assert!(!runtime.pipeline_root.exists());
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
    real_controls(CargoKind::SharedGameHost);
}

#[test]
#[ignore = "real confined original pause controls; requires the installed producer and private offline vendor closure"]
fn real_original_pause_controls_run_on_candidate_and_integrated_source() {
    real_controls(CargoKind::PauseEquipment);
}

#[test]
#[ignore = "real confined original compact controls and dependency preparation; requires installed tools and original source"]
fn real_original_compact_controls_run_on_candidate_and_integrated_source() {
    real_controls(CargoKind::CompactController);
}

#[test]
#[ignore = "read-only authentication of retained real Cargo proof receipts after cleanup"]
fn retained_original_cargo_proofs_authenticate_after_cleanup() {
    let root = PathBuf::from(std::env::var_os("SM_CARGO_PROOF_ROOT").expect("SM_CARGO_PROOF_ROOT"));
    let prepared =
        |name: &str| -> PreparedCargo { serde_json::from_slice(&fs::read(root.join(name)).unwrap()).unwrap() };
    let mut authenticated = 0usize;
    let environment_path = root.join("environment/cargo-environment-receipt.toml");
    if environment_path.is_file() {
        let prerequisite =
            prepared(if root.join("prerequisite.json").is_file() { "prerequisite.json" } else { "prepared.json" });
        let receipt: CargoEnvironmentReceipt = family::object(&environment_path).unwrap();
        authenticate_environment(&prerequisite, &root.join("environment/source"), &receipt).unwrap();
        authenticated += 1;
    }
    for phase in ["candidate", "integrated", "protected-input-refusal"] {
        let receipt_path = root.join(phase).join("cargo-receipt.toml");
        if receipt_path.is_file() {
            let receipt: CargoReceipt = family::object(&receipt_path).unwrap();
            authenticate(&prepared("prepared.json"), &root.join(phase).join("source"), &receipt).unwrap();
            authenticated += 1;
        }
    }
    assert!(authenticated > 0, "no retained Cargo proof receipt was authenticated");
}

fn real_controls(kind: CargoKind) {
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
    if kind == CargoKind::CompactController {
        for name in [
            "crates/pipeline/src/decomp_prefix.rs",
            "crates/pipeline/src/prefix_compact_observer.lua.in",
            "crates/pipeline/src/prefix_observer.lua.in",
            "crates/pipeline/tests/prefix.rs",
            "docs/DECOMP_PREFIX.md",
        ] {
            let original = std::process::Command::new("git")
                .current_dir(current)
                .args(["show", &format!("223f8a1b0c7382817b1290d13c7a41f4e722d5e1:{name}")])
                .output()
                .unwrap();
            assert!(original.status.success(), "original compact source missing: {name}");
            let path = checkout.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, &original.stdout).unwrap();
            sources.insert(name.into(), sha256(original.stdout));
        }
        for name in [
            "deps/snesrev-sm/src/sm_a0.c",
            "deps/snesrev-sm/src/sm_a6.c",
            "deps/snesrev-sm/src/sm_86.c",
            "deps/snesrev-sm/src/sm_80.c",
            "deps/snesrev-sm/src/sm_90.c",
            "deps/snesrev-sm/src/snes/snes.c",
            "tools/decomp/headless.c",
        ] {
            let bytes = fs::read(current.join(name)).unwrap();
            let path = checkout.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, &bytes).unwrap();
            sources.insert(name.into(), sha256(bytes));
        }
    }
    let vendor = PathBuf::from(std::env::var_os("SM_CARGO_VENDOR").expect("SM_CARGO_VENDOR"));
    let mut vendor_files = BTreeMap::new();
    collect_fixture(&vendor, None, &mut vendor_files, Path::new(""));
    let mut contract = CargoContract {
        schema: "translation-cargo-contract-v1".into(),
        kind,
        source_files: sources,
        approved_files: match kind {
            CargoKind::CompactController => vec!["crates/pipeline/src/decomp_prefix.rs".into()],
            CargoKind::SharedGameHost => vec!["crates/port/src/game_host.rs".into(), "crates/port/src/lib.rs".into()],
            CargoKind::PauseEquipment => vec!["crates/port/src/pause_equipment_render.rs".into()],
        },
        vendor: CargoClosure { root: vendor, files: vendor_files },
        dependency_cache: None,
        limits: CargoLimits {
            timeout_seconds: 60,
            writable_bytes: if kind == CargoKind::CompactController { 768 * 1024 * 1024 } else { 256 * 1024 * 1024 },
            stdout_bytes: 256 * 1024,
            stderr_bytes: 65536,
        },
    };
    let prerequisite = prepare(&checkout, &contract).unwrap();
    fs::write(root.join("prerequisite.json"), serde_json::to_vec_pretty(&prerequisite).unwrap()).unwrap();
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
        jobs: 4,
        gate_timeout_seconds: if kind == CargoKind::CompactController { 120 } else { 60 },
        max_output_bytes: if kind == CargoKind::CompactController {
            4 * 1024 * 1024 * 1024
        } else {
            1024 * 1024 * 1024
        },
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
        &prerequisite,
        &checkout,
        &root.join("environment"),
        &runtime,
        &environment_context,
        if kind == CargoKind::CompactController { 600 } else { 180 },
        kind == CargoKind::CompactController,
    )
    .unwrap();
    authenticate_environment(&prerequisite, &checkout, &environment).unwrap();
    assert_eq!(
        prepare_environment(
            &prerequisite,
            &checkout,
            &root.join("environment"),
            &runtime,
            &environment_context,
            if kind == CargoKind::CompactController { 600 } else { 180 },
            kind == CargoKind::CompactController,
        )
        .unwrap(),
        environment
    );
    assert!(environment.passed, "{}", environment.diagnostic);
    contract.dependency_cache = environment.cache.clone();
    let prepared = prepare(&checkout, &contract).unwrap();
    fs::write(root.join("prepared.json"), serde_json::to_vec_pretty(&prepared).unwrap()).unwrap();
    let receipt = run(&prepared, &checkout, &output, &runtime, &context).unwrap();
    let verdict = authenticate(&prepared, &checkout, &receipt).unwrap();
    let replay = run(&prepared, &checkout, &output, &runtime, &context).unwrap();
    assert_eq!(replay, receipt);
    assert_eq!(authenticate(&prepared, &checkout, &replay).unwrap(), verdict);
    assert!(receipt.passed, "{}", receipt.diagnostic);
    let expected = if kind == CargoKind::SharedGameHost { 31 } else { 10 };
    assert_eq!(verdict.test_count, expected);
    assert_eq!(verdict.operations.len(), if kind == CargoKind::CompactController { 2 } else { 1 });
    let protected = root.join("undeclared-input.txt");
    fs::write(&protected, "outside-the-cargo-input-closure").unwrap();
    let approved = match kind {
        CargoKind::CompactController => "crates/pipeline/src/decomp_prefix.rs",
        CargoKind::SharedGameHost => "crates/port/src/game_host.rs",
        CargoKind::PauseEquipment => "crates/port/src/pause_equipment_render.rs",
    };
    let original = fs::read_to_string(checkout.join(approved)).unwrap();
    let integrated_source = if kind == CargoKind::SharedGameHost {
        let marker = "    fn request(&mut self, ram: &mut Wram<'_>, request: Request) -> Result<Response> {";
        assert!(original.contains(marker));
        original.replacen(marker, &format!("{marker}\n        assert!(std::fs::read({protected:?}).is_err());"), 1)
    } else {
        format!("// Integrated source identity control.\n{original}")
    };
    fs::write(checkout.join(approved), integrated_source).unwrap();
    let mut integrated_context = context.clone();
    integrated_context.operation_key = sha256("ticket10-original-cargo-integrated");
    integrated_context.phase = translation_execution::model::ExecutionPhase::Integration;
    let integrated = run(&prepared, &checkout, &root.join("integrated"), &runtime, &integrated_context).unwrap();
    assert_eq!(run(&prepared, &checkout, &root.join("integrated"), &runtime, &integrated_context).unwrap(), integrated);
    let integrated_verdict = authenticate(&prepared, &checkout, &integrated).unwrap();
    assert!(integrated.passed, "{}", integrated.diagnostic);
    assert_ne!(integrated.candidate_sha256, receipt.candidate_sha256);
    assert_eq!(integrated_verdict.test_count, expected);
    authenticate(&prepared, &output.join("source"), &receipt).unwrap();

    if kind == CargoKind::SharedGameHost {
        let library = checkout.join("crates/port/src/lib.rs");
        let mut source = fs::read_to_string(&library).unwrap();
        source.push_str(&format!("\nconst UNDECLARED_INPUT: &str = include_str!({protected:?});\n"));
        fs::write(library, source).unwrap();
        let mut refused_context = context.clone();
        refused_context.operation_key = sha256("ticket10-original-cargo-protected-input-refusal");
        let refused_output = root.join("protected-input-refusal");
        let refused = run(&prepared, &checkout, &refused_output, &runtime, &refused_context).unwrap();
        assert!(!refused.passed);
        let refused_verdict = authenticate(&prepared, &checkout, &refused).unwrap();
        assert_eq!(refused_verdict.test_count, 0);
        assert_eq!(refused_verdict.operations.len(), 1);
        assert!(refused.diagnostic.contains("couldn't read"), "{}", refused.diagnostic);
        assert!(!refused.diagnostic.contains("outside-the-cargo-input-closure"));
        assert_eq!(run(&prepared, &checkout, &refused_output, &runtime, &refused_context).unwrap(), refused);
        authenticate(&prepared, &root.join("integrated/source"), &integrated).unwrap();
    }
}
