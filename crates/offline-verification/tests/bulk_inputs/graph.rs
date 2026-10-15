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
                let _ = child.kill();
            }
            let _ = child.wait();
        }
    }
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
