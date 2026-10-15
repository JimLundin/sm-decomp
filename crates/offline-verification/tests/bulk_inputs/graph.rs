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
