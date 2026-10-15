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
    path::PathBuf,
    time::SystemTime,
};
use tool_execution::{
    atomic_toml,
    bounded::{digest, immutable_bytes as immutable, now_unix_ms},
};
use translation_execution::{
    OperationContext,
    model::{sealed, sha256},
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
}

#[derive(Deserialize, Serialize)]
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

fn retain_case(root: &Path, name: &str, mut case: RetainedCase) -> Result<()> {
    case.prepared = fs::canonicalize(case.prepared)?;
    case.checkout = fs::canonicalize(case.checkout)?;
    case.receipt = fs::canonicalize(case.receipt)?;
    let path = root.join("retained-authentication.toml");
    let mut proof = if path.exists() {
        bounded::object::<RetainedProof>(&path)?
    } else {
        RetainedProof { schema: "retained-native-proof-v1".into(), cases: BTreeMap::new() }
    };
    ensure!(proof.schema == "retained-native-proof-v1", "retained Native proof schema changed");
    proof.cases.insert(name.into(), case);
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
        proof.schema == "retained-native-proof-v1" && !proof.cases.is_empty(),
        "retained Native proof is incomplete"
    );
    for (name, case) in proof.cases {
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
        if let Some(diagnostic) = case.diagnostic_contains {
            ensure!(receipt.diagnostic.contains(&diagnostic), "{name}: rejection diagnostic changed");
        }
        println!("Authenticated retained Native proof {name}: {}", admitted.report().status);
    }
    Ok(())
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
    let compile_source = variant_source(&prepared, &checkout, &root.join("compile-denial-source"))?;
    let candidate = compile_source.join(prepared.candidate_path());
    let candidate_bytes = bounded::read(&candidate)?;
    let undeclared = root.join("compiler-secret.txt");
    fs::write(&undeclared, "protected input")?;
    let compile_read = format!("\nconst UNDECLARED:&str=include_str!({:?});\n", undeclared);
    let mut mutated = candidate_bytes.clone();
    mutated.extend(compile_read.as_bytes());
    fs::write(&candidate, &mutated)?;
    let mut deny_context = context.clone();
    deny_context.operation_key = sha256("native-compile-time-denial");
    let denied = run(&prepared, &compile_source, &root.join("compile-denial"), &runtime, &deny_context);
    let retry = match &denied {
        Ok(denied) => {
            let retry = run(&prepared, &compile_source, &root.join("compile-denial"), &runtime, &deny_context)?;
            ensure!(sealed(&retry)? == sealed(denied)?, "unchanged rejected candidate reran compiler");
            Some(retry)
        }
        Err(_) => None,
    };
    let denied = denied?;
    retain_case(
        &root,
        "compile-denial",
        RetainedCase {
            prepared: root.join("prepared.toml"),
            checkout: compile_source.clone(),
            receipt: root.join("compile-denial/native-receipt.toml"),
            passed: false,
            fixture_pairs: None,
            operation_count: None,
            diagnostic_contains: Some("compiler-secret".into()),
        },
    )?;
    ensure!(retry.is_some(), "failed native evaluation was not durably reused");
    ensure!(
        !denied.passed && denied.diagnostic.contains("compiler-secret"),
        "compile-time undeclared inclusion was not denied: {}",
        denied.diagnostic
    );
    authenticate(&prepared, &compile_source, &denied)?;

    let runtime_source = variant_source(&prepared, &checkout, &root.join("runtime-denial-source"))?;
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
    deny_context.operation_key = sha256("native-runtime-protected-denial");
    let denied = run(&prepared, &runtime_source, &root.join("runtime-denial"), &runtime, &deny_context);
    let denied = denied?;
    retain_case(
        &root,
        "runtime-denial",
        RetainedCase {
            prepared: root.join("prepared.toml"),
            checkout: runtime_source.clone(),
            receipt: root.join("runtime-denial/native-receipt.toml"),
            passed: true,
            fixture_pairs: Some(16),
            operation_count: Some(34),
            diagnostic_contains: None,
        },
    )?;
    ensure!(
        denied.passed && bounded::text(&runtime_secret)? == "protected parent verdict",
        "runtime denial escaped namespace or changed real family semantics: {}",
        denied.diagnostic
    );
    authenticate(&prepared, &runtime_source, &denied)?;
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
        "Preparation gaps, compiler undeclared inclusion, runtime parent/sibling read/write, tampered operation and aggregate prelaunch reservation checks passed"
    );
    Ok(())
}
