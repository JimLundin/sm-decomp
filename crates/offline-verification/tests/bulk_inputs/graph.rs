//! Batch scheduling controls cross the same closed interface as the pipeline.
use super::*;

fn batch_request<'a>(project: &Project, catalog: &'a offline_verification::CatalogSnapshot) -> BatchVerification<'a> {
    BatchVerification {
        catalog,
        source: project.source(),
        root: pipeline_root(),
        workers: 4,
        family_jobs: 1,
        case_jobs: 2,
        families: ["a", "b"]
            .into_iter()
            .map(|family| FamilyJob { family: family.into(), output: project.root.join(family), previous: None })
            .collect(),
        inputs: None,
    }
}

fn native_environment(runtime: &ConfinedEnvironment) -> ConfinedEnvironment {
    ConfinedEnvironment {
        helper: runtime.helper.clone(),
        rustc: runtime.rustc.clone(),
        c_compiler: runtime.c_compiler.clone(),
        operation_root: runtime.operation_root.clone(),
        pipeline_root: runtime.pipeline_root.clone(),
        workers: runtime.workers,
        gate_timeout_seconds: runtime.gate_timeout_seconds,
        deadline_unix_ms: runtime.deadline_unix_ms,
        max_output_bytes: runtime.max_output_bytes,
    }
}

pub(super) fn native_recovery_controls(
    project: &Project,
    offline: &OfflineFamilies,
    prepared: &PreparedNative,
    runtime: &ConfinedEnvironment,
    context: &OperationContext,
    output: &Path,
    receipt: &NativeReceipt,
) -> Result<()> {
    let projection = output.join("work-graph.toml");
    let intent_path = output.join("native-intent.toml");
    let intent_bytes = fs::read(&intent_path)?;
    let intent: toml::Value = toml::from_str(std::str::from_utf8(&intent_bytes)?)?;
    ensure!(intent["graph_metadata_bytes"].as_integer() == Some(65536));
    ensure!(receipt.advisory_output_bytes()? == 65536);
    ensure!(
        receipt.evidence_bytes + 65536 <= receipt.reserved_bytes && receipt.reserved_bytes <= receipt.max_output_bytes
    );
    let saved_deadline = intent["deadline_unix_ms"].as_integer().context("saved Native deadline")?;
    let parent_bytes =
        intent["parent_reservation_bytes"].as_integer().context("saved Native parent reservation")? as u64;
    let producers = receipt
        .retained_references()?
        .into_iter()
        .filter(|reference| reference.path.starts_with(&runtime.operation_root))
        .map(|reference| {
            Ok((reference.path.clone(), digest(&reference.path)?, fs::metadata(&reference.path)?.modified()?))
        })
        .collect::<Result<Vec<_>>>()?;
    ensure!(!producers.is_empty(), "Native recovery control did not retain actual producers");
    let mut absent = native_environment(runtime);
    absent.helper = project.root.join("missing-helper");
    absent.rustc = project.root.join("missing-rustc");
    absent.c_compiler = project.root.join("missing-cc");
    let retry = |environment: &ConfinedEnvironment, context: &OperationContext| {
        prepared.verify(
            NativeTarget { checkout: &project.checkout(), output, context, fixture_jobs: Some(2) },
            environment,
            offline.host_inputs(),
        )
    };

    // A missing projection is powerless to create a new attempt.
    fs::remove_file(&projection)?;
    let mut wrong_context = context.clone();
    wrong_context.attempt += 1;
    ensure!(retry(&absent, &wrong_context).is_err(), "wrong Native attempt admitted");
    ensure!(!projection.exists(), "wrong attempt reconstructed advisory progress before authentication");
    for corrupt in [false, true] {
        if corrupt {
            fs::write(&projection, b"corrupt advisory bytes\n")?;
        }
        let replay = retry(&absent, context)?;
        ensure!(sealed(&replay)? == sealed(receipt)?, "advisory loss changed the sealed Native result");
        let restored: toml::Value = bounded::object(&projection)?;
        ensure!(restored["recovered_from_authenticated_receipt"].as_bool() == Some(true));
        ensure!(restored["deadline_unix_ms"].as_integer() == Some(saved_deadline), "replay extended the deadline");
        ensure!(restored["required_producers"].as_integer() == Some(8));
        ensure!(restored["compared_cases"].as_integer() == Some(3));
        ensure!(restored["operation_context_sha256"].as_str() == Some(sealed(context)?.as_str()));
    }
    for allowance in [None, Some(0), Some(65535), Some(65537), Some(131072)] {
        let mut changed = intent.clone();
        if let Some(allowance) = allowance {
            changed["graph_metadata_bytes"] = toml::Value::Integer(allowance);
        } else {
            changed.as_table_mut().unwrap().remove("graph_metadata_bytes");
        }
        fs::write(&intent_path, toml::to_string(&changed)?)?;
        let rejected = retry(&absent, context);
        fs::write(&intent_path, &intent_bytes)?;
        ensure!(rejected.is_err(), "tampered Native advisory allowance admitted: {allowance:?}");
    }

    // Reconcile settled producers when publication alone was interrupted.
    let receipt_path = output.join("native-receipt.toml");
    let saved_receipt = fs::read(&receipt_path)?;
    fs::remove_file(&receipt_path)?;
    fs::remove_file(&projection)?;
    let recovered = retry(runtime, context);
    if recovered.is_err() {
        fs::write(&receipt_path, &saved_receipt)?;
    }
    let recovered = recovered.context("Native pre-publication recovery")?;
    ensure!(
        sealed(&recovered)? == sealed(receipt)?,
        "Native publication recovery changed the original producer result"
    );
    ensure!(recovered.operation_count()? == 8, "publication recovery changed producer coverage");
    ensure!(fs::read(&intent_path)? == intent_bytes, "publication recovery changed intent or deadline");
    for (path, hash, modified) in producers {
        ensure!(
            digest(&path)? == hash && fs::metadata(&path)?.modified()? == modified,
            "Native graph recovery changed producer evidence: {}",
            path.display()
        );
    }

    // Keep the output path length equal so the captured path inventory has
    // the identical reservation extent; refuse before intent/producers exist.
    let tight_output = project.root.join("low");
    ensure!(tight_output.as_os_str().len() == output.as_os_str().len());
    let mut tight = native_environment(runtime);
    tight.max_output_bytes = parent_bytes - 1;
    let error = prepared
        .verify(
            NativeTarget { checkout: &project.checkout(), output: &tight_output, context, fixture_jobs: Some(2) },
            &tight,
            offline.host_inputs(),
        )
        .err()
        .context("unreserved Native advisory output admitted")?;
    ensure!(
        format!("{error:#}").contains("native parent snapshots exceed aggregate output budget"),
        "wrong reservation refusal: {error:#}"
    );
    ensure!(!tight_output.join("native-intent.toml").exists(), "tight budget published an admitted Native intent");
    atomic_toml(
        &project.root.join("graph-recovery-controls.toml"),
        &BTreeMap::from([
            ("producer_count", 8u64),
            ("advisory_reserved_bytes", 65536),
            ("reserved_bytes_before_and_after", receipt.reserved_bytes),
            ("saved_deadline_unix_ms", saved_deadline as u64),
            ("parent_reservation_bytes", parent_bytes),
        ]),
    )
}

#[test]
#[ignore = "authenticate a retained actual-C Native result from before graph metadata existed"]
fn legacy_native_intent_replays_without_unreserved_projection() -> Result<()> {
    let replay: Replay = bounded::object(&PathBuf::from(std::env::var("SM_GRAPH_LEGACY_REPLAY")?))?;
    let (prepared_path, receipt_path) = replay.native.context("legacy Native replay pair missing")?;
    let prepared: PreparedNative = bounded::object(&prepared_path)?;
    let receipt: NativeReceipt = bounded::object(&receipt_path)?;
    let output = receipt_path.parent().context("legacy Native output")?;
    let intent: toml::Value = bounded::object(&output.join("native-intent.toml"))?;
    ensure!(intent.get("graph_metadata_bytes").is_none(), "control is not an absent-field historical intent");
    ensure!(!output.join("work-graph.toml").exists(), "legacy control already has a new projection");
    let before = receipt
        .retained_references()?
        .into_iter()
        .map(|reference| {
            Ok((reference.path.clone(), digest(&reference.path)?, fs::metadata(&reference.path)?.modified()?))
        })
        .collect::<Result<Vec<_>>>()?;
    let runtime = ConfinedEnvironment {
        helper: output.join("absent-helper"),
        rustc: output.join("absent-rustc"),
        c_compiler: output.join("absent-cc"),
        operation_root: receipt.operation_root.clone(),
        pipeline_root: pipeline_root(),
        workers: 4,
        gate_timeout_seconds: 1,
        deadline_unix_ms: intent.get("outer_deadline_unix_ms").and_then(toml::Value::as_integer).map(|n| n as u64),
        max_output_bytes: receipt.max_output_bytes,
    };
    // Retained evidence authenticates against its original trusted host path
    // contract. Fresh producer controls above still use the complete HOST.
    const LEGACY_HOST: &[(&str, &str)] =
        &[("crates/offline-verification/tests/bulk_inputs.rs", include_str!("../bulk_inputs.rs"))];
    let offline = OfflineFamilies::new(HostInputs::new(LEGACY_HOST, digest(&std::env::current_exe()?)?)?);
    let replayed = prepared.verify(
        NativeTarget {
            checkout: &replay.checkout,
            output,
            context: &receipt.context,
            fixture_jobs: Some(intent["fixture_jobs"].as_integer().context("legacy Fixture jobs")? as usize),
        },
        &runtime,
        offline.host_inputs(),
    )?;
    ensure!(sealed(&replayed)? == sealed(&receipt)? && replayed.operation_count()? == receipt.operation_count()?);
    ensure!(
        replayed.advisory_output_bytes()? == 0 && !output.join("work-graph.toml").exists(),
        "old receipt acquired unreserved output or changed reservation"
    );
    for (path, hash, modified) in before {
        ensure!(
            digest(&path)? == hash && fs::metadata(&path)?.modified()? == modified,
            "old Native replay changed retained evidence: {}",
            path.display()
        );
    }
    println!(
        "Authenticated historical Native intent with {} unchanged producers and zero advisory bytes",
        receipt.operation_count()?
    );
    Ok(())
}

#[test]
fn native_sequence_settles_failed_wave_and_retains_exact_family_prefix_on_retry() -> Result<()> {
    let mut project = Project::new("graph-native-prefix")?;
    let offline = verifier()?;
    for family in ["a", "b"] {
        let path = project.checkout().join(format!("tools/port/{family}.smf"));
        let mut manifest = fixtures::read(&path)?;
        for case in &mut manifest.routines[0].cases {
            case.expected_after_file = None;
            case.expected_after_sha256 = None;
        }
        if family == "a" {
            manifest.routines[0].cases[0].expected_return = Some(500);
        }
        fixtures::write(&path, &manifest)?;
    }
    let inventory = project.root.join("inventory.toml");
    atomic_toml(&inventory, &decomp_source::inventory(&project.checkout(), &project.source())?)?;
    let catalog = project.root.join("catalog.toml");
    atomic_toml(&catalog, offline.capture_catalog(CatalogRequest::new(&project.checkout())?)?.catalog())?;
    let requirements = project.root.join("requirements.md");
    fs::write(
        &requirements,
        "Complete failed first-family wave; no later-family admission; original receipt retry.\n",
    )?;
    let mut prepared = Vec::new();
    for family in ["a", "b"] {
        let contract = offline.declare_native(NativeDeclaration {
            checkout: &project.checkout(),
            kind: NativeDeclarationKind::Family(Path::new(&format!("tools/port/{family}.smf"))),
            source_inventory: Asset { file: inventory.clone(), sha256: digest(&inventory)? },
            provenance: NativeProvenance::ReviewedCatalog {
                catalog: Asset { file: catalog.clone(), sha256: digest(&catalog)? },
                requirements: Asset { file: requirements.clone(), sha256: digest(&requirements)? },
                checkout: project.checkout(),
                rationale: "Complete Native wave and family-prefix controls".into(),
            },
            compiler_limits: NativeLimits {
                timeout_seconds: 180,
                writable_bytes: 128 * 1024 * 1024,
                stdout_bytes: 4096,
                stderr_bytes: 256 * 1024,
                stdin_bytes: 0,
            },
            fixture_limits: NativeLimits {
                timeout_seconds: 5,
                writable_bytes: 512 * 1024,
                stdout_bytes: 4096,
                stderr_bytes: 4096,
                stdin_bytes: 0,
            },
            deadline_unix_ms: None,
            max_input_bytes: bounded::DEFAULT_BYTES,
        })?;
        prepared.push(offline.prepare_native(
            NativePreparation::new(&project.checkout(), &project.source(), &contract),
            &NoScopeAdmission,
        )?);
    }
    let runtime = ConfinedEnvironment {
        helper: PathBuf::from(std::env::var("SM_BULK_NATIVE_HELPER")?),
        rustc: PathBuf::from(std::env::var("SM_BULK_NATIVE_RUSTC")?),
        c_compiler: "/usr/bin/x86_64-linux-gnu-gcc-9".into(),
        operation_root: project.root.join("operations"),
        pipeline_root: pipeline_root(),
        workers: 4,
        gate_timeout_seconds: 600,
        deadline_unix_ms: None,
        max_output_bytes: bounded::DEFAULT_BYTES,
    };
    let output = project.root.join("sequence");
    let jobs = || {
        ["a", "b"]
            .into_iter()
            .zip(&prepared)
            .map(|(family, prepared)| {
                Ok(offline_verification::NativeSequenceJob {
                    family: family.into(),
                    required_prepared_sha256: sealed(prepared)?,
                    prepared: Ok(prepared.clone()),
                    context: OperationContext {
                        run_id: sha256(project.root.display().to_string()),
                        attempt: 1,
                        policy_sha256: sha256("native-prefix-control"),
                        operation_key: sha256(family),
                        phase: ExecutionPhase::Candidate,
                    },
                })
            })
            .collect::<Result<Vec<_>>>()
    };
    let run = |runtime: &ConfinedEnvironment| {
        offline.verify_native_sequence(offline_verification::NativeSequence {
            checkout: &project.checkout(),
            output: &output,
            environment: runtime,
            retained_bytes: 0,
            families: jobs()?,
        })
    };
    let outcome = run(&runtime)?;
    ensure!(
        !outcome.passed && outcome.families.keys().map(String::as_str).collect::<Vec<_>>() == ["a"],
        "failed Native family did not preserve the exact required prefix"
    );
    ensure!(
        !output.join("native/b").exists() && !output.join("families/b.json").exists(),
        "later Native family launched after rejection"
    );
    let record = &outcome.families["a"];
    let family: offline_verification::NativeFamilyReceipt = serde_json::from_slice(&fs::read(&record.path)?)?;
    ensure!(
        !family.receipt.passed && family.receipt.operation_count()? == 6,
        "failed wave lost complete admitted producer attribution"
    );
    let family_path = output.join("native/a/family");
    for case in ["root", "captured"] {
        ensure!(family_path.join(case).join("c-after.wram").is_file(), "admitted Native wave did not settle actual C");
    }
    ensure!(!family_path.join("continued/c-after.wram").exists(), "Native failed predecessor admitted a continuation");
    let before = family
        .receipt
        .retained_references()?
        .into_iter()
        .map(|reference| {
            Ok((reference.path.clone(), digest(&reference.path)?, fs::metadata(&reference.path)?.modified()?))
        })
        .collect::<Result<Vec<_>>>()?;
    let record_before = (digest(&record.path)?, fs::metadata(&record.path)?.modified()?);
    let mut absent = native_environment(&runtime);
    absent.helper = project.root.join("absent-helper");
    absent.rustc = project.root.join("absent-rustc");
    absent.c_compiler = project.root.join("absent-cc");
    fs::remove_file(output.join("native/a/work-graph.toml"))?;
    let replayed = run(&absent)?;
    ensure!(
        !replayed.passed && replayed.families == outcome.families && replayed.retained_bytes == outcome.retained_bytes,
        "Native sequence retry changed immutable family receipt or aggregate accounting"
    );
    ensure!(record_before == (digest(&record.path)?, fs::metadata(&record.path)?.modified()?));
    for (path, hash, modified) in before {
        ensure!(
            digest(&path)? == hash && fs::metadata(&path)?.modified()? == modified,
            "Native sequence retry rewrote retained producer evidence"
        );
    }
    let replay = project.root.join("sequence-replay.toml");
    atomic_toml(&replay, &SequenceReplay { checkout: project.checkout(), record: record.path.clone(), operations: 6 })?;
    project.cleanup()?;
    let result = Command::new(std::env::current_exe()?)
        .args(["--exact", "graph::retained_native_sequence_reader", "--ignored", "--nocapture"])
        .env("SM_GRAPH_SEQUENCE_REPLAY", &replay)
        .env("PATH", "/unavailable-sequence-compilers")
        .output()?;
    fs::write(replay.with_extension("reader.stdout"), &result.stdout)?;
    fs::write(replay.with_extension("reader.stderr"), &result.stderr)?;
    ensure!(
        result.status.success(),
        "independent Native sequence reader failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct SequenceReplay {
    checkout: PathBuf,
    record: PathBuf,
    operations: usize,
}

#[test]
#[ignore = "independent Native prefix reader after cleanup"]
fn retained_native_sequence_reader() -> Result<()> {
    let replay: SequenceReplay = bounded::object(&PathBuf::from(std::env::var("SM_GRAPH_SEQUENCE_REPLAY")?))?;
    let family: offline_verification::NativeFamilyReceipt = serde_json::from_slice(&fs::read(&replay.record)?)?;
    let admitted =
        family.prepared.authenticate(NativeEvidence { checkout: &replay.checkout, receipt: &family.receipt })?;
    ensure!(
        admitted.report().status == "rejected"
            && !family.receipt.passed
            && family.receipt.operation_count()? == replay.operations
    );
    Ok(())
}

#[test]
fn batch_stops_new_family_admission_after_terminal_rejection() -> Result<()> {
    let mut project = Project::new("graph-rejection")?;
    let manifest_path = project.checkout().join("tools/port/a.smf");
    let mut manifest = fixtures::read(&manifest_path)?;
    manifest.routines[0].cases[0].expected_return = Some(500);
    fixtures::write(&manifest_path, &manifest)?;
    let offline = verifier()?;
    let catalog = offline.capture_catalog(CatalogRequest::new(&project.checkout())?)?;
    let completed = std::sync::Mutex::new(Vec::new());
    let batch = offline.verify_batch(batch_request(&project, &catalog), |outcome| {
        completed.lock().unwrap().push(outcome.family().to_owned());
        Ok(())
    })?;
    let families = batch.families();
    ensure!(families.iter().map(|outcome| outcome.family()).collect::<Vec<_>>() == ["a", "b"]);
    let rejected = families[0].result().as_ref().map_err(|e| anyhow::anyhow!("{e:#}"))?;
    ensure!(rejected.run.report().status == "rejected", "terminal rejection control did not reach the adapter");
    let report = rejected.run.report();
    ensure!(
        report.error.as_deref().is_some_and(|error| error.contains("independent expected return mismatch: root")),
        "exact comparison rejection was lost"
    );
    ensure!(report.run.join("root/c-after.wram").is_file(), "control did not execute actual C");
    ensure!(report.run.join("continued/blocked.toml").is_file(), "blocked continuation diagnostic was lost");
    ensure!(!project.root.join("b").exists(), "batch admitted family b after terminal rejection");
    ensure!(families[1].result().is_err(), "unadmitted family disappeared instead of retaining canonical outcome");
    ensure!(*completed.lock().unwrap() == ["a", "b"], "completion projection omitted a blocked family");
    project.cleanup()
}

#[test]
fn canonical_rejection_keeps_long_diagnostic_tail_and_complete_record() -> Result<()> {
    let mut project = Project::new("graph-long-diagnostic")?;
    let symbol = format!("Touch{}DISTINGUISHING_DIAGNOSTIC_TAIL", "A".repeat(8192));
    let rust = project.checkout().join("crates/port/src/fixture.rs");
    fs::write(&rust, fs::read_to_string(&rust)?.replace("Touch", &symbol))?;
    let c = project.source().join("src/sm_80.c");
    fs::write(
        &c,
        fs::read_to_string(&c)?
            .replace("Touch", &symbol)
            .replace("g_ram[17] = (uint8_t)a;", "g_ram[17] = (uint8_t)(a + 1);"),
    )?;
    project.pin()?;
    let manifest_path = project.checkout().join("tools/port/a.smf");
    let mut manifest = fixtures::read(&manifest_path)?;
    manifest.routines[0].symbol = symbol.clone();
    manifest.routines[0].cases.truncate(1);
    fixtures::write(&manifest_path, &manifest)?;
    let offline = verifier()?;
    let catalog = offline.capture_catalog(CatalogRequest::new(&project.checkout())?)?;
    let mut request = batch_request(&project, &catalog);
    request.families.truncate(1);
    let batch = offline.verify_batch(request, |_| Ok(()))?;
    let run = &batch.families()[0].result().as_ref().map_err(|error| anyhow::anyhow!("{error:#}"))?.run;
    let report = run.report();
    let expected = format!("WRAM mismatch at 0x11: {symbol}/root");
    ensure!(report.status == "rejected" && expected.len() > 8192);
    ensure!(report.error.as_deref() == Some(expected.as_str()), "canonical diagnostic lost its distinguishing tail");
    let retained: offline_verification::Verdict = bounded::object(&run.evidence())?;
    ensure!(retained.error == report.error, "published canonical diagnostic changed");
    let record: CaseRecord = bounded::object(&report.run.join("root/rejection.toml"))?;
    ensure!(report.cases.len() == 1 && record.routine == symbol && report.cases[0] == record);
    ensure!(record.returns.len() == 2 && record.after_sha256.len() == 2 && record.first_mismatch.is_some());
    ensure!(fs::metadata(report.run.join("root/c-after.wram"))?.len() == 0x20000);
    project.cleanup()
}

fn chain_workload(project: &Project) -> Result<()> {
    let source_path = project.source().join("src/sm_80.c");
    let source = fs::read_to_string(&source_path)?
        .replace("#include <stdint.h>", "#include <stdint.h>\n#include <unistd.h>")
        .replace("  const char *path", "  usleep(50000);\n  const char *path");
    fs::write(source_path, source)?;
    project.pin()?;
    let mut manifest = fixtures::read(&project.checkout().join("tools/port/a.smf"))?;
    let root = manifest.routines[0].cases[0].clone();
    manifest.routines[0].cases = (0..24)
        .map(|index| Case {
            name: format!("case-{index:02}"),
            continue_from: (index >= 4).then(|| format!("case-{:02}", if index == 4 { 0 } else { index - 1 })),
            fill: (index < 4).then_some(7),
            ..root.clone()
        })
        .collect();
    for family in ["a", "b"] {
        fixtures::write(&project.checkout().join(format!("tools/port/{family}.smf")), &manifest)?;
    }
    Ok(())
}

fn peak_processes(receipt: &pipeline_model::metrics::Receipt, selected: impl Fn(&str) -> bool) -> usize {
    let mut events = Vec::new();
    for process in receipt.processes.iter().filter(|process| selected(&process.role)) {
        events.push((process.start_ns, 1i32));
        events.push((process.end_ns, -1i32));
    }
    events.sort_unstable();
    let mut active = 0;
    let mut peak = 0;
    for (_, delta) in events {
        active += delta;
        peak = peak.max(active);
    }
    peak as usize
}

fn family_interval(receipt: &pipeline_model::metrics::Receipt, root: &Path, family: &str) -> Result<(u64, u64)> {
    let prefix = root.join(family).display().to_string();
    let processes = receipt
        .processes
        .iter()
        .filter(|process| process.role == "c" && process.argv.first().is_some_and(|arg| arg.starts_with(&prefix)))
        .collect::<Vec<_>>();
    Ok((
        processes.iter().map(|process| process.start_ns).min().context("family C producers missing")?,
        processes.iter().map(|process| process.end_ns).max().context("family C producers missing")?,
    ))
}

#[test]
fn shared_batch_bounds_helpers_and_preserves_complete_chain_results() -> Result<()> {
    let mut project = Project::new("graph-helpers")?;
    chain_workload(&project)?;
    let offline = verifier()?;
    let catalog = offline.capture_catalog(CatalogRequest::new(&project.checkout())?)?;
    let mut request = batch_request(&project, &catalog);
    request.family_jobs = 2;
    request.case_jobs = 4;
    let metrics_path = project.root.join("batch.metrics.toml");
    let mut metrics = tool_execution::metrics::Run::start(metrics_path.clone(), "graph-helper-control")?;
    let batch = offline.verify_batch(request, |_| Ok(()))?;
    metrics.finish("success", None)?;
    let receipt: pipeline_model::metrics::Receipt = bounded::object(&metrics_path)?;
    receipt.validate()?;
    let mut observed = Vec::new();
    let mut verdicts = Vec::new();
    for family in batch.into_families() {
        let run = family.into_result()?.run;
        ensure!(run.report().status == "offline-verified", "{:?}", run.report().error);
        ensure!(run.report().verified_cases == 24 && run.report().cases.len() == 24, "Fixture coverage changed");
        observed.push(observations(run.report())?);
        verdicts.push(run.evidence());
    }
    ensure!(observed[0] == observed[1], "complete chain states/returns differ between aliases");
    let helper_peak = peak_processes(&receipt, |role| role.starts_with("fixture-helper-"));
    let producer_peak = peak_processes(&receipt, |role| matches!(role, "rust" | "c"));
    let a = family_interval(&receipt, &project.root, "a")?;
    let b = family_interval(&receipt, &project.root, "b")?;
    ensure!(a.0 < b.1 && b.0 < a.1, "one family waited for the other's complete long chain");
    let summary = BTreeMap::from([
        ("helper_peak", helper_peak as f64),
        ("producer_peak", producer_peak as f64),
        ("elapsed_seconds", receipt.elapsed_ns as f64 / 1e9),
    ]);
    atomic_toml(&project.root.join("observed-bounds.toml"), &summary)?;
    println!("Actual process bounds: {summary:?}");
    ensure!(producer_peak <= 4 && producer_peak >= 2, "shared producer capacity not exercised: {producer_peak}");
    ensure!(
        helper_peak <= 8,
        "resident Fixture helpers multiplied across families: {helper_peak} exceeds two helpers per shared worker"
    );
    ensure!(receipt.processes.iter().all(|process| process.reap_count == 1), "an owned process was not reaped");
    for verdict in verdicts {
        let admitted = offline.authenticate(AuthenticateRequest { checkout: project.checkout(), verdict })?;
        ensure!(admitted.report().verified_cases == 24, "independent reader lost chain coverage");
    }
    project.cleanup()
}

#[test]
fn maximum_family_window_measures_helper_reuse_and_complete_fixture_work() -> Result<()> {
    let mut project = Project::new("graph-window")?;
    chain_workload(&project)?;
    // Keep each complete long chain active while the remaining families build,
    // so the eight-family window also stresses scheduling after preparation.
    let c = project.source().join("src/sm_80.c");
    fs::write(&c, fs::read_to_string(&c)?.replace("usleep(50000)", "usleep(1000000)"))?;
    project.pin()?;
    let families = ["a", "b", "c", "d", "e", "f", "g", "h"];
    project.write(
        "tools/port/batch.toml",
        format!(
            "[families]\n{}\n[profiles]\nall={families:?}\n",
            families.map(|name| format!("{name}='family'")).join("\n")
        ),
    )?;
    let manifest = fixtures::read(&project.checkout().join("tools/port/a.smf"))?;
    for family in families {
        fixtures::write(&project.checkout().join(format!("tools/port/{family}.smf")), &manifest)?;
    }
    let offline = verifier()?;
    let catalog = offline.capture_catalog(CatalogRequest::new(&project.checkout())?)?;
    let mut request = batch_request(&project, &catalog);
    request.family_jobs = 8;
    request.case_jobs = 4;
    request.families = families
        .into_iter()
        .map(|family| FamilyJob { family: family.into(), output: project.root.join(family), previous: None })
        .collect();
    let metrics_path = project.root.join("batch.metrics.toml");
    let mut metrics = tool_execution::metrics::Run::start(metrics_path.clone(), "maximum-family-window")?;
    let batch = offline.verify_batch(request, |_| Ok(()))?;
    metrics.finish("success", None)?;
    let receipt: pipeline_model::metrics::Receipt = bounded::object(&metrics_path)?;
    receipt.validate()?;
    let mut expected = None;
    let mut verdicts = Vec::new();
    ensure!(
        batch.families().iter().map(|family| family.family()).collect::<Vec<_>>() == families,
        "maximum window changed canonical family reporting"
    );
    for family in batch.into_families() {
        let run = family.into_result()?.run;
        ensure!(run.report().status == "offline-verified", "{:?}", run.report().error);
        ensure!(run.report().verified_cases == 24 && run.report().cases.len() == 24, "maximum window lost Fixtures");
        let actual = observations(run.report())?;
        if let Some(expected) = &expected {
            ensure!(expected == &actual, "maximum window changed full WRAM or returns");
        } else {
            expected = Some(actual);
        }
        verdicts.push(run.evidence());
    }
    let summary = window_measurement(&receipt)?;
    let helper_peak = peak_processes(&receipt, |role| role.starts_with("fixture-helper-"));
    let producer_peak = peak_processes(&receipt, |role| matches!(role, "rust" | "c"));
    atomic_toml(&project.root.join("observed-window.toml"), &summary)?;
    println!("Maximum family window: {summary:?}");
    ensure!(helper_peak <= 8 && (2..=4).contains(&producer_peak), "maximum window exceeded capacity or lost overlap");
    ensure!(receipt.processes.iter().all(|process| process.reap_count == 1), "maximum window lost owned children");
    let replay = project.root.join("replay.toml");
    atomic_toml(&replay, &GraphReplay { checkout: project.checkout(), verdicts, cases: 24 })?;
    project.cleanup()?;
    graph_reader(&replay)
}

fn window_measurement(receipt: &pipeline_model::metrics::Receipt) -> Result<BTreeMap<&'static str, f64>> {
    // Trace spans can be omitted at the receipt bound. Fixed per-scope
    // summaries retain all intervals independently of trace sampling.
    let stages = receipt
        .scopes
        .iter()
        .filter_map(|scope| {
            let key = "stage-summary:fixture-execution-and-comparison:";
            scope.configuration.get(&format!("{key}count")).map(|count| -> Result<(u64, u64, u64)> {
                ensure!(count == "1", "unexpected repeated Fixture stage");
                let number = |field: &str| -> Result<u64> {
                    Ok(scope.configuration.get(&format!("{key}{field}")).context("Fixture summary field")?.parse()?)
                };
                Ok((number("first-start-ns")?, number("last-end-ns")?, number("summed-elapsed-ns")?))
            })
        })
        .collect::<Result<Vec<_>>>()?;
    ensure!(stages.len() == 8, "complete Fixture scope summaries missing");
    let first = stages.iter().map(|stage| stage.0).min().unwrap();
    let last = stages.iter().map(|stage| stage.1).max().unwrap();
    let mut events = stages.iter().flat_map(|stage| [(stage.0, 1i32), (stage.1, -1i32)]).collect::<Vec<_>>();
    events.sort_unstable();
    let mut active = 0;
    let mut family_peak = 0;
    for (_, delta) in events {
        active += delta;
        family_peak = family_peak.max(active);
    }
    Ok(BTreeMap::from([
        ("fixture_pairs", 192.0),
        (
            "helper_starts",
            receipt.processes.iter().filter(|process| process.role.starts_with("fixture-helper-")).count() as f64,
        ),
        ("helper_peak", peak_processes(receipt, |role| role.starts_with("fixture-helper-")) as f64),
        ("producer_peak", peak_processes(receipt, |role| matches!(role, "rust" | "c")) as f64),
        ("fixture_family_peak", family_peak as f64),
        ("fixture_stage_wall_seconds", (last - first) as f64 / 1e9),
        ("fixture_stage_sum_seconds", stages.iter().map(|stage| stage.2).sum::<u64>() as f64 / 1e9),
        ("elapsed_seconds", receipt.elapsed_ns as f64 / 1e9),
    ]))
}

#[test]
#[ignore = "read-only summary recovery for an already completed stress control"]
fn retained_window_measurement() -> Result<()> {
    let path = PathBuf::from(std::env::var("SM_GRAPH_WINDOW")?);
    let receipt: pipeline_model::metrics::Receipt = bounded::object(&path.join("batch.metrics.toml"))?;
    receipt.validate()?;
    let summary = window_measurement(&receipt)?;
    atomic_toml(&path.join("observed-window.toml"), &summary)?;
    println!("Retained maximum window measurement: {summary:?}");
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Competing {
    project: PathBuf,
    label: String,
}

#[test]
#[ignore = "isolated competing batch, invoked by the public parent control"]
fn competing_batch_child() -> Result<()> {
    let spec: Competing = bounded::object(&PathBuf::from(std::env::var("SM_GRAPH_COMPETING")?))?;
    // The parent waits for both owners before cleaning the shared proof.
    let project = Project { root: spec.project, cleaned: true };
    let offline = verifier()?;
    let catalog = offline.capture_catalog(CatalogRequest::new(&project.checkout())?)?;
    let required = vec!["a".into(), "b".into()];
    let inputs = offline.capture_batch_inputs(BatchInputRequest {
        catalog: &catalog,
        source: &project.source(),
        required: &required,
    })?;
    let output = project.root.join(&spec.label);
    fs::create_dir_all(&output)?;
    fs::write(output.join("ready"), b"captured\n")?;
    let deadline = Instant::now() + std::time::Duration::from_secs(60);
    while !project.root.join("start").exists() {
        ensure!(Instant::now() < deadline, "competing start rendezvous expired");
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let mut request = batch_request(&project, &catalog);
    request.family_jobs = 2;
    request.case_jobs = 4;
    request.inputs = Some(&inputs);
    for job in &mut request.families {
        job.output = output.join(&job.family);
    }
    let mut metrics = tool_execution::metrics::Run::start(output.join("batch.metrics.toml"), "competing-batch")?;
    let result = offline.verify_batch(request, |_| Ok(()));
    let error = result.as_ref().err().map(|error| format!("{error:#}"));
    metrics.finish(tool_execution::metrics::outcome(error.as_deref()), error)?;
    let batch = result?;
    let mut verdicts = Vec::new();
    for completion in batch.into_families() {
        let run = completion.into_result()?.run;
        ensure!(run.report().status == "offline-verified", "{:?}", run.report().error);
        verdicts.push(run.evidence());
    }
    atomic_toml(&output.join("replay.toml"), &GraphReplay { checkout: project.checkout(), verdicts, cases: 24 })
}

#[derive(Serialize, Deserialize)]
struct GraphReplay {
    checkout: PathBuf,
    verdicts: Vec<PathBuf>,
    cases: usize,
}

#[test]
#[ignore = "retained independent graph reader, invoked with an exact replay manifest"]
fn retained_graph_reader() -> Result<()> {
    let replay: GraphReplay = bounded::object(&PathBuf::from(std::env::var("SM_GRAPH_REPLAY")?))?;
    let offline = verifier()?;
    for verdict in replay.verdicts {
        let admitted = offline.authenticate(AuthenticateRequest { checkout: replay.checkout.clone(), verdict })?;
        ensure!(admitted.report().verified_cases == replay.cases, "retained graph coverage changed");
        let (cases, _) = observations(admitted.report())?;
        ensure!(cases.len() == replay.cases, "retained complete canonical states missing");
    }
    Ok(())
}

fn graph_reader(path: &Path) -> Result<()> {
    let output = Command::new(std::env::current_exe()?)
        .args(["--exact", "graph::retained_graph_reader", "--ignored", "--nocapture"])
        .env("SM_GRAPH_REPLAY", path)
        .env("PATH", "/unavailable-graph-compilers")
        .output()?;
    fs::write(path.with_extension("reader.stdout"), &output.stdout)?;
    fs::write(path.with_extension("reader.stderr"), &output.stderr)?;
    ensure!(output.status.success(), "retained graph reader failed: {}", String::from_utf8_lossy(&output.stderr));
    Ok(())
}

struct Children(Vec<std::process::Child>);
impl Drop for Children {
    fn drop(&mut self) {
        for child in &mut self.0 {
            if child.try_wait().ok().flatten().is_none() {
                // SAFETY: this is an owned, unreaped child. Let its control
                // settle helpers and descendants before scoped cleanup.
                unsafe {
                    libc::kill(child.id() as i32, libc::SIGTERM);
                }
            }
        }
        let deadline = Instant::now() + std::time::Duration::from_secs(10);
        for child in &mut self.0 {
            while child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            if child.try_wait().ok().flatten().is_none() {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
    }
}

#[test]
#[ignore = "isolated signal recipient for the cancellation control"]
fn cancelled_batch_child() -> Result<()> {
    let project = Project { root: PathBuf::from(std::env::var("SM_GRAPH_CANCEL_PROJECT")?), cleaned: true };
    let offline = verifier()?;
    let catalog = offline.capture_catalog(CatalogRequest::new(&project.checkout())?)?;
    let mut metrics = tool_execution::metrics::Run::start(project.root.join("batch.metrics.toml"), "cancelled-batch")?;
    let mut request = batch_request(&project, &catalog);
    request.case_jobs = 4;
    let batch = offline.verify_batch(request, |_| Ok(()))?;
    let first = batch.families()[0].result().as_ref().map_err(|error| anyhow::anyhow!("{error:#}"))?;
    ensure!(
        first.run.report().status == "rejected"
            && first.run.report().error.as_deref().is_some_and(|error| error.contains("interrupted by signal")),
        "interrupted Fixture diagnostic missing"
    );
    ensure!(
        batch.families()[1].family() == "b" && batch.families()[1].result().is_err(),
        "cancellation lost canonical unadmitted family"
    );
    metrics.finish("interrupted", first.run.report().error.clone())?;
    Ok(())
}

#[test]
fn cancelling_batch_reaps_helpers_fixture_descendant_and_blocks_new_admission() -> Result<()> {
    let mut project = Project::new("graph-cancellation")?;
    let rust = project.checkout().join("crates/port/src/fixture.rs");
    // Keep Rust within the captured source contract: the trusted controller
    // observes its PID through /proc; no unmodeled filesystem/foreign imports
    // are introduced solely to instrument a gameplay candidate.
    let code = fs::read_to_string(&rust)?
        .replace("    if name", "    if a == 3 { loop { std::thread::yield_now(); } }\n    if name");
    fs::write(&rust, code)?;
    let c = project.source().join("src/sm_80.c");
    fs::write(
        &c,
        fs::read_to_string(&c)?
            .replace("#include <stdint.h>", "#include <stdint.h>\n#include <stdio.h>\n#include <unistd.h>")
            .replace(
                "  const char *path",
                r#"  if (a == 9) {
    FILE *out = fopen("helper.pid", "w"); fprintf(out, "%d", getppid()); fclose(out);
    out = fopen("fixture.pid", "w"); fprintf(out, "%d", getpid()); fclose(out);
    if (fork() == 0) {
      out = fopen("descendant.pid", "w"); fprintf(out, "%d", getpid()); fclose(out);
    }
    sleep(30);
  }
  const char *path"#,
            ),
    )?;
    project.pin()?;
    let manifest_path = project.checkout().join("tools/port/a.smf");
    let mut manifest = fixtures::read(&manifest_path)?;
    let mut completed = manifest.routines[0].cases[0].clone();
    completed.name = "completed".into();
    completed.a = 4;
    completed.expected_return = Some(5);
    completed.expected_after_file = None;
    completed.expected_after_sha256 = None;
    manifest.routines[0].cases.push(completed);
    fixtures::write(&manifest_path, &manifest)?;
    let mut children = Children(vec![
        Command::new(std::env::current_exe()?)
            .args(["--exact", "graph::cancelled_batch_child", "--ignored", "--nocapture"])
            .env("SM_GRAPH_CANCEL_PROJECT", &project.root)
            .stdout(fs::File::create(project.root.join("child.stdout"))?)
            .stderr(fs::File::create(project.root.join("child.stderr"))?)
            .spawn()?,
    ]);
    let deadline = Instant::now() + std::time::Duration::from_secs(40);
    let directory = loop {
        if let Ok(entries) = fs::read_dir(project.root.join("a/evidence"))
            && let Some(directory) =
                entries.filter_map(Result::ok).map(|entry| entry.path().join("root")).find(|path| {
                    path.parent().unwrap().join("captured/descendant.pid").is_file()
                        && path.parent().unwrap().join("completed/c-after.wram").is_file()
                })
        {
            break directory;
        }
        ensure!(Instant::now() < deadline, "cancellation control did not reach its complete admitted work");
        ensure!(children.0[0].try_wait()?.is_none(), "cancellation child stopped before signal; see retained logs");
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    let rust_process = fs::read_dir("/proc")?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|process| fs::read_link(process.join("cwd")).is_ok_and(|cwd| cwd == directory))
        .context("admitted Rust Fixture process missing")?;
    let rust_pid: i32 = rust_process.file_name().context("Rust Fixture PID")?.to_str().context("PID UTF-8")?.parse()?;
    let rust_status = fs::read_to_string(rust_process.join("status"))?;
    let rust_helper: i32 = rust_status
        .lines()
        .find_map(|line| line.strip_prefix("PPid:"))
        .context("Rust Fixture helper parent missing")?
        .trim()
        .parse()?;
    let mut pids = vec![rust_pid, rust_helper];
    let captured = directory.parent().unwrap().join("captured");
    for name in ["helper.pid", "fixture.pid", "descendant.pid"] {
        pids.push(fs::read_to_string(captured.join(name))?.parse()?);
    }
    let mut identities = Vec::new();
    for pid in pids {
        let path = PathBuf::from(format!("/proc/{pid}/stat"));
        let stat = fs::read_to_string(&path)?;
        let started = stat
            .rsplit_once(')')
            .context("process identity delimiter")?
            .1
            .split_whitespace()
            .nth(19)
            .context("process start identity")?
            .to_owned();
        identities.push((path, started));
    }
    // SAFETY: the child is still owned and unreaped; it installs the existing
    // cooperative cancellation handler before starting its graph workers.
    ensure!(unsafe { libc::kill(children.0[0].id() as i32, libc::SIGTERM) } == 0);
    let deadline = Instant::now() + std::time::Duration::from_secs(10);
    let status = loop {
        if let Some(status) = children.0[0].try_wait()? {
            break status;
        }
        ensure!(Instant::now() < deadline, "batch did not settle cancellation");
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    ensure!(status.success(), "isolated cancellation assertions failed; see retained child logs");
    for (path, started) in identities {
        ensure!(
            !fs::read_to_string(path).is_ok_and(|stat| {
                stat.rsplit_once(')').is_some_and(|(_, rest)| rest.split_whitespace().nth(19) == Some(started.as_str()))
            }),
            "owned helper, Fixture or descendant survived cancellation"
        );
    }
    ensure!(!directory.join("c.stdout").exists(), "cancelled Rust Fixture admitted C");
    ensure!(
        directory.parent().unwrap().join("continued/blocked.toml").is_file(),
        "cancelled predecessor lost blocked diagnostic"
    );
    ensure!(!project.root.join("b").exists(), "cancellation admitted a later family");
    let receipt: pipeline_model::metrics::Receipt = bounded::object(&project.root.join("batch.metrics.toml"))?;
    receipt.validate()?;
    ensure!(receipt.outcome == "interrupted" && receipt.processes.iter().all(|process| process.reap_count == 1));
    ensure!(peak_processes(&receipt, |role| role.starts_with("fixture-helper-")) <= 8);
    drop(children);
    project.cleanup()
}

#[test]
fn two_competing_batches_share_four_slots_and_bound_resident_helpers() -> Result<()> {
    let mut project = Project::new("graph-competing")?;
    chain_workload(&project)?;
    let mut children = Children(Vec::new());
    for label in ["one", "two"] {
        let spec = project.root.join(format!("{label}.toml"));
        atomic_toml(&spec, &Competing { project: project.root.clone(), label: label.into() })?;
        let child = Command::new(std::env::current_exe()?)
            .args(["--exact", "graph::competing_batch_child", "--ignored", "--nocapture"])
            .env("SM_GRAPH_COMPETING", spec)
            .stdout(fs::File::create(project.root.join(format!("{label}.stdout")))?)
            .stderr(fs::File::create(project.root.join(format!("{label}.stderr")))?)
            .spawn()?;
        children.0.push(child);
    }
    let deadline = Instant::now() + std::time::Duration::from_secs(60);
    while !["one", "two"].iter().all(|label| project.root.join(label).join("ready").exists()) {
        ensure!(Instant::now() < deadline, "competing batches failed to capture inputs");
        ensure!(
            children.0.iter_mut().all(|child| child.try_wait().ok().flatten().is_none()),
            "competing child failed before start"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    fs::write(project.root.join("start"), b"both captured\n")?;
    let mut statuses = Vec::new();
    for child in &mut children.0 {
        statuses.push(child.wait()?);
    }
    ensure!(statuses.iter().all(|status| status.success()), "competing child failed; see retained child logs");
    let mut receipts = ["one", "two"]
        .into_iter()
        .map(|label| {
            bounded::object::<pipeline_model::metrics::Receipt>(&project.root.join(label).join("batch.metrics.toml"))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut replay_paths = Vec::new();
    let mut all_verdicts = Vec::new();
    for (label, receipt) in ["one", "two"].into_iter().zip(&receipts) {
        receipt.validate()?;
        ensure!(
            peak_processes(receipt, |role| role.starts_with("fixture-helper-")) <= 8,
            "one batch exceeded its helper bound"
        );
        ensure!(
            receipt.processes.iter().all(|process| process.reap_count == 1),
            "competing child lost process accounting"
        );
        let replay_path = project.root.join(label).join("replay.toml");
        let replay: GraphReplay = bounded::object(&replay_path)?;
        all_verdicts.extend(replay.verdicts);
        replay_paths.push(replay_path);
    }
    // Each child has its own monotonic origin in the same OS clock domain.
    // Translate observed intervals to that common clock before comparing them.
    for receipt in &mut receipts {
        for process in &mut receipt.processes {
            process.start_ns += receipt.clock.origin_ns;
            process.end_ns += receipt.clock.origin_ns;
        }
    }
    let one = (
        receipts[0].processes.iter().map(|p| p.start_ns).min().unwrap(),
        receipts[0].processes.iter().map(|p| p.end_ns).max().unwrap(),
    );
    let two = (
        receipts[1].processes.iter().map(|p| p.start_ns).min().unwrap(),
        receipts[1].processes.iter().map(|p| p.end_ns).max().unwrap(),
    );
    ensure!(one.0 < two.1 && two.0 < one.1, "the two batches did not compete");
    let other = receipts.pop().unwrap();
    receipts[0].processes.extend(other.processes);
    let producer_peak = peak_processes(&receipts[0], |role| matches!(role, "rust" | "c"));
    let helper_peak = peak_processes(&receipts[0], |role| role.starts_with("fixture-helper-"));
    ensure!(producer_peak == 4, "two batches did not respect/exercise the four shared producer slots: {producer_peak}");
    ensure!(helper_peak <= 16, "two batches exceeded their aggregate resident helper bounds: {helper_peak}");
    atomic_toml(
        &project.root.join("observed-bounds.toml"),
        &BTreeMap::from([("producer_peak", producer_peak), ("helper_peak", helper_peak)]),
    )?;
    let offline = verifier()?;
    let mut previous = None;
    for verdict in all_verdicts {
        let admitted = offline.authenticate(AuthenticateRequest { checkout: project.checkout(), verdict })?;
        let current = observations(admitted.report())?;
        if let Some(previous) = &previous {
            ensure!(*previous == current, "competing batch full states/returns changed");
        }
        previous = Some(current);
    }
    drop(children);
    project.cleanup()?;
    for replay in replay_paths {
        graph_reader(&replay)?;
    }
    Ok(())
}

#[test]
fn batch_stops_new_family_admission_after_publication_error() -> Result<()> {
    let mut project = Project::new("graph-publication")?;
    let offline = verifier()?;
    let catalog = offline.capture_catalog(CatalogRequest::new(&project.checkout())?)?;
    let published = std::sync::Mutex::new(Vec::new());
    let result = offline.verify_batch(batch_request(&project, &catalog), |outcome| {
        published.lock().unwrap().push(outcome.family().to_owned());
        if outcome.family() == "a" {
            let completed = outcome.result().as_ref().map_err(|e| anyhow::anyhow!("{e:#}"))?;
            ensure!(completed.run.report().status == "offline-verified", "publication control did not finish a");
            anyhow::bail!("injected completion publication failure");
        }
        Ok(())
    });
    let error = result.err().context("publication failure was hidden")?;
    ensure!(format!("{error:#}").contains("injected completion publication failure"), "{error:#}");
    ensure!(!project.root.join("b").exists(), "batch admitted family b after publication failure");
    ensure!(*published.lock().unwrap() == ["a", "b"], "completion projection omitted a blocked family");
    project.cleanup()
}

#[test]
fn batch_observer_panic_settles_owned_helpers_and_stops_admission() -> Result<()> {
    let mut project = Project::new("graph-observer-panic")?;
    let offline = verifier()?;
    let catalog = offline.capture_catalog(CatalogRequest::new(&project.checkout())?)?;
    let metrics_path = project.root.join("batch.metrics.toml");
    let mut metrics = tool_execution::metrics::Run::start(metrics_path.clone(), "graph-observer-panic")?;
    let result = offline.verify_batch(batch_request(&project, &catalog), |outcome| {
        assert!(outcome.family() != "a", "injected observer panic after successful actual C");
        Ok(())
    });
    metrics.finish("failed", Some("injected observer panic".into()))?;
    ensure!(format!("{:#}", result.err().context("observer panic was hidden")?).contains("observer panicked"));
    ensure!(!project.root.join("b").exists(), "observer panic admitted the next family");
    let receipt: pipeline_model::metrics::Receipt = bounded::object(&metrics_path)?;
    receipt.validate()?;
    ensure!(receipt.processes.iter().filter(|process| process.role == "c").count() == 3, "actual C coverage missing");
    for process in &receipt.processes {
        ensure!(process.reap_count == 1, "observer panic lost owned child accounting");
        ensure!(unsafe { libc::kill(process.pid as i32, 0) } == -1, "owned process survived observer panic");
    }
    project.cleanup()
}
