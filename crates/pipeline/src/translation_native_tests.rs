//! Retained real Native proofs use the public Offline verification interface.
use super::*;
fn current_checkout() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}

#[test]
#[ignore = "real separately confined actual-C controls; retains source, replay and cleanup evidence"]
fn real_reviewed_recipes_replay_and_authenticate() -> Result<()> {
    let _ = crate::BINARY_SHA256.set(crate::digest(&std::env::current_exe()?)?);
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
    for directory in ["crates/pipeline/src", "crates/port/src"] {
        for entry in fs::read_dir(current.join(directory))? {
            let path = entry?.path();
            if path.is_file() {
                paths.insert(path.strip_prefix(&current)?.display().to_string());
            }
        }
    }
    for name in &names {
        if *name == "pause-direct-animation" {
            paths.extend([
                "tools/port/pause-equipment-render.smf".into(),
                ".scratch/decomp-coverage/renderer/direct-animation.toml".into(),
            ]);
        } else {
            paths.insert(format!("tools/port/{name}.smf"));
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
        prepare_generated_plm(&checkout, &checkout.join(".scratch/translation-native/plm-instructions"))?;
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
    let inputs = prepare_catalog_inputs_until(
        &checkout,
        pinned,
        &inventory,
        now_unix_ms()? + 600_000,
        8 * bounded::DEFAULT_BYTES,
    )?;
    for name in names {
        let output = root.join(name);
        fs::create_dir_all(&output)?;
        let contract = if name == "pause-direct-animation" {
            contract_for_direct_animation(
                &checkout,
                Path::new(".scratch/decomp-coverage/renderer/direct-animation.toml"),
                inventory.clone(),
                provenance.clone(),
                compiler.clone(),
                fixture.clone(),
            )?
        } else {
            contract_for_family(
                &checkout,
                Path::new(&format!("tools/port/{name}.smf")),
                inventory.clone(),
                provenance.clone(),
                compiler.clone(),
                fixture.clone(),
            )?
        };
        let declared = recipes::manifest(&checkout, &checkout.join(&contract.manifest.file), contract.recipe.as_ref())?;
        let prepared = prepare_scope_in_catalog_until(
            &checkout,
            pinned,
            &contract,
            &[declared.rust_module],
            &inputs,
            now_unix_ms()? + 600_000,
            8 * bounded::DEFAULT_BYTES,
        )?;
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
            policy_sha256: prepared.verification_sha256.clone(),
            operation_key: sha256(name),
            phase: translation_execution::model::ExecutionPhase::Candidate,
        };
        let fixture_jobs = std::env::var("SM_NATIVE_FIXTURE_JOBS")
            .ok()
            .map(|value| value.parse::<usize>())
            .transpose()?
            .unwrap_or(runtime.workers);
        let receipt = run_bounded(&prepared, &checkout, &output.join("run"), &runtime, &context, fixture_jobs)?;
        let verdict = authenticate(&prepared, &checkout, &receipt)?;
        let mut identities = Vec::new();
        receipt.visit_operations(|operation| {
            identities.push((operation.result.path.clone(), operation.result.path.metadata()?.modified()?));
            Ok(())
        })?;
        ensure!(
            sealed(&run(&prepared, &checkout, &output.join("run"), &runtime, &context)?)? == sealed(&receipt)?,
            "recipe replay changed receipt"
        );
        for (path, modified) in identities {
            ensure!(path.metadata()?.modified()? == modified, "recipe replay changed producer");
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
    ensure!(failures.is_empty(), "{}", failures.join("; "));
    Ok(())
}

#[test]
#[ignore = "freezes every current catalog declaration with real pinned source; no producer is launched"]
fn complete_current_catalog_has_a_frozen_native_recipe() -> Result<()> {
    let root = PathBuf::from(std::env::var("SM_NATIVE_CATALOG_PROOF")?);
    fs::create_dir_all(&root)?;
    let checkout = std::env::var_os("SM_NATIVE_CATALOG_SOURCE").map(PathBuf::from).unwrap_or_else(current_checkout);
    prepare_generated_plm(&checkout, &checkout.join(".scratch/translation-native/plm-instructions"))?;
    let catalog = batch::catalog(&checkout)?;
    let catalog_path = root.join("catalog.toml");
    immutable(&catalog_path, toml::to_string(&catalog)?.as_bytes())?;
    let requirements = checkout.join(".scratch/translation-scope-execution/issues/10-original-acceptance-adapters.md");
    let provenance = NativeProvenance::ReviewedCatalog { catalog: Asset { file: catalog_path.clone(), sha256: digest(&catalog_path)? },
            requirements: Asset { file: requirements.clone(), sha256: digest(&requirements)? }, checkout: checkout.clone(),
            rationale: "Current conservative catalog: every registered declaration, all generated cases, exact original requirements".into() };
    let inventory = Asset {
        file: "tools/port/source-inventory.toml".into(),
        sha256: digest(&checkout.join("tools/port/source-inventory.toml"))?,
    };
    let pinned = Path::new("/home/jim/sm-decomp/deps/snesrev-sm");
    let deadline = now_unix_ms()? + 600_000;
    let inputs = prepare_catalog_inputs_until(&checkout, pinned, &inventory, deadline, 8 * bounded::DEFAULT_BYTES)?;
    let limits = NativeLimits {
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
    let mut failures = Vec::new();
    for name in catalog.families.keys() {
        let result = (|| -> Result<usize> {
            let entry = batch::entry(&catalog, name)?;
            let contract = contract_for_family_until(
                &checkout,
                &entry.manifest,
                inventory.clone(),
                provenance.clone(),
                limits.clone(),
                fixture.clone(),
                deadline,
                8 * bounded::DEFAULT_BYTES,
            )?;
            let prepared = bounded::until(deadline, 8 * bounded::DEFAULT_BYTES, || {
                prepare_bounded_with_catalog(&checkout, pinned, &contract, Some(&inputs))
            })?;
            let output = root.join(format!("{name}.toml"));
            immutable(&output, toml::to_string(&prepared)?.as_bytes())?;
            Ok(contract.coverage.values().map(Vec::len).sum())
        })();
        match result {
            Ok(cases) => {
                summary.insert(name.clone(), format!("prepared {cases} fixture pairs"));
            }
            Err(error) => {
                summary.insert(name.clone(), format!("rejected: {error:#}"));
                failures.push(format!("{name}: {error:#}"));
            }
        }
        atomic_toml(&root.join("summary.toml"), &summary)?;
    }
    ensure!(summary.len() == catalog.families.len(), "catalog entry omitted");
    let direct = (|| -> Result<usize> {
        let contract = bounded::until(deadline, 8 * bounded::DEFAULT_BYTES, || {
            contract_for_direct_animation(
                &checkout,
                Path::new(".scratch/decomp-coverage/renderer/direct-animation.toml"),
                inventory,
                provenance,
                limits,
                fixture,
            )
        })?;
        let prepared = bounded::until(deadline, 8 * bounded::DEFAULT_BYTES, || {
            prepare_bounded_with_catalog(&checkout, pinned, &contract, Some(&inputs))
        })?;
        let cases = contract.coverage.values().map(Vec::len).sum();
        ensure!(contract.family == "pause-direct-animation" && cases == 360, "auxiliary animation domain changed");
        immutable(&root.join("pause-direct-animation.toml"), toml::to_string(&prepared)?.as_bytes())?;
        Ok(cases)
    })();
    let result = match direct {
        Ok(cases) => format!("prepared {cases} fixture pairs"),
        Err(error) => {
            failures.push(format!("pause-direct-animation: {error:#}"));
            format!("rejected: {error:#}")
        }
    };
    atomic_toml(&root.join("auxiliary-summary.toml"), &BTreeMap::from([("pause-direct-animation", result)]))?;
    ensure!(failures.is_empty(), "{}", failures.join("; "));
    Ok(())
}

fn compiled_sources() -> Vec<(&'static str, &'static str)> {
    vec![
        ("main.rs", include_str!("main.rs")),
        ("inputs.rs", include_str!("inputs.rs")),
        ("bounded_inputs.rs", include_str!("bounded_inputs.rs")),
        ("builds.rs", include_str!("builds.rs")),
        ("build_cache.rs", include_str!("build_cache.rs")),
        ("family.rs", include_str!("family.rs")),
        ("family_model.rs", include_str!("family_model.rs")),
        ("fixture_format.rs", include_str!("fixture_format.rs")),
        ("fixture_writes.rs", include_str!("fixture_writes.rs")),
        ("fixture_cli.rs", include_str!("fixture_cli.rs")),
        ("translation_native.rs", include_str!("translation_native.rs")),
        ("source_inventory.rs", include_str!("source_inventory.rs")),
        ("batch.rs", include_str!("batch.rs")),
        ("family_header.c", include_str!("family_header.c")),
        ("family_pointer.c", include_str!("family_pointer.c")),
        ("dispatcher_pointer.c", include_str!("dispatcher_pointer.c")),
        ("dispatcher_plm.c", include_str!("dispatcher_plm.c")),
        ("dispatcher_eproj.c", include_str!("dispatcher_eproj.c")),
        ("dispatcher_host.c", include_str!("dispatcher_host.c")),
        ("plm_fixtures.rs", include_str!("plm_fixtures.rs")),
        ("cleanup.rs", include_str!("cleanup.rs")),
        ("fixture_execution.rs", include_str!("fixture_execution.rs")),
        ("fixture_worker.rs", include_str!("fixture_worker.rs")),
        ("metrics.rs", include_str!("metrics.rs")),
        ("resource_sampler.rs", include_str!("resource_sampler.rs")),
    ]
}

fn variant_source(prepared: &PreparedNative, original: &Path, destination: &Path) -> Result<PathBuf> {
    fs::create_dir_all(destination)?;
    for name in prepared.contract.frozen_inputs.keys() {
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
    let _ = crate::BINARY_SHA256.set(crate::digest(&std::env::current_exe()?)?);
    let proof = PathBuf::from(std::env::var("SM_NATIVE_PROOF_ROOT")?);
    let root = proof.join(std::env::var("SM_NATIVE_PROOF_NAME").unwrap_or_else(|_| "actual-c".into()));
    fs::create_dir_all(&root)?;
    let current = current_checkout();
    let checkout = root.join("source");
    fs::create_dir_all(checkout.join("crates/pipeline/src"))?;
    fs::create_dir_all(checkout.join("crates/port/src"))?;
    fs::create_dir_all(checkout.join("tools/port"))?;
    let manifest_path = Path::new("tools/port/enemy-jump-instructions.smf");
    let manifest = crate::fixture_format::read(&current.join(manifest_path))?;
    immutable(&checkout.join(manifest_path), &bounded::read(current.join(manifest_path))?)?;
    for path in inputs::canonical_sources(&current, &[current.join(&manifest.rust_module)], &manifest.rust_imports)? {
        let relative = path.strip_prefix(&current)?;
        immutable(&checkout.join(relative), &bounded::read(&path)?)?;
    }
    for (name, text) in compiled_sources() {
        immutable(&checkout.join("crates/pipeline/src").join(name), text.as_bytes())?;
    }
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
    let contract = contract_for_family(
        &checkout,
        manifest_path,
        Asset { file: inventory_path.clone(), sha256: digest(&inventory_path)? },
        provenance,
        limits,
        fixture_limits,
    )?;
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
        policy_sha256: prepared.verification_sha256.clone(),
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
    ensure!(receipt.passed, "{}", receipt.diagnostic);
    let authentication_started = std::time::Instant::now();
    let verdict = authenticate(&prepared, &checkout, &receipt)?;
    let authentication_seconds = authentication_started.elapsed().as_secs_f64();
    ensure!(
        verdict.verified_cases == 16 && verdict.verified_routines == 2 && receipt.operation_count()? == 34,
        "real family coverage differs"
    );
    let mut before = Vec::new();
    let mut operations = Vec::new();
    let mut invocation_times = BTreeMap::new();
    receipt.visit_operations(|operation| {
        before.push((operation.result.path.clone(), fs::metadata(&operation.result.path)?.modified()?));
        invocation_times.insert(operation.invocation.file.clone(), operation.invocation.file.metadata()?.modified()?);
        operations.push(operation.clone());
        Ok(())
    })?;
    let replay_started = std::time::Instant::now();
    let retry = run(&prepared, &checkout, &root.join("run"), &runtime, &context)?;
    let replay_seconds = replay_started.elapsed().as_secs_f64();
    ensure!(sealed(&retry)? == sealed(&receipt)?, "lost response retry allocated another result");
    for (path, modified) in before {
        ensure!(fs::metadata(path)?.modified()? == modified, "retry rewrote a producer result");
    }
    // Hash-only family tampering cannot bypass the independent durable
    // producer authenticators, even if a forged receipt copies a new hash.
    let operation = &operations[0];
    let invocation_bytes = bounded::read(&operation.invocation.file)?;
    let original: builds::StrictInvocation = family::object(&operation.invocation.file)?;
    let mut changed = original.clone();
    changed.program.arguments.push("--undeclared".into());
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
        references(&receipt)?;
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
            let timing: builds::ExecutionRecord = family::object(&timing_path)?;
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
        return Ok(());
    }
    let compile_source = variant_source(&prepared, &checkout, &root.join("compile-denial-source"))?;
    let candidate = compile_source.join(&prepared.candidate_path);
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
    ensure!(retry.is_some(), "failed native evaluation was not durably reused");
    ensure!(
        !denied.passed && denied.diagnostic.contains("compiler-secret"),
        "compile-time undeclared inclusion was not denied: {}",
        denied.diagnostic
    );
    authenticate(&prepared, &compile_source, &denied)?;

    let runtime_source = variant_source(&prepared, &checkout, &root.join("runtime-denial-source"))?;
    let candidate = runtime_source.join(&prepared.candidate_path);
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
    ensure!(
        denied.passed && bounded::text(&runtime_secret)? == "protected parent verdict",
        "runtime denial escaped namespace or changed real family semantics: {}",
        denied.diagnostic
    );
    authenticate(&prepared, &runtime_source, &denied)?;
    let mut deny_context = context.clone();
    deny_context.operation_key = sha256("native-output-reservation-denial");
    let narrow = StrictRuntime { max_output_bytes: parent_reservation(&prepared, &checkout)? + 4096, ..runtime };
    let denied = run(&prepared, &checkout, &root.join("reservation-denial"), &narrow, &deny_context)?;
    ensure!(
        !denied.passed && denied.operation_count()? == 0 && denied.diagnostic.contains("reservation exhausted"),
        "aggregate reservation did not stop before compiler launch"
    );
    println!(
        "Preparation gaps, compiler undeclared inclusion, runtime parent/sibling read/write, tampered operation and aggregate prelaunch reservation checks passed"
    );
    Ok(())
}
