//! Complete ordinary batches. Policy, durable batch records and Live replay
//! belong to the application; this module owns complete family execution.
use crate::{
    AuthenticateRequest, AuthenticatedFamily, CatalogSnapshot, FamilyFacts, OfflineFamilies, PreviousFamily,
    RegisteredRequest, RegisteredRun, declarations::Declarations, family,
};
use anyhow::{Context, Result, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};

pub struct FamilyJob {
    pub family: String,
    pub output: PathBuf,
    pub previous: Option<PreviousFamily>,
}

pub struct BatchVerification<'a> {
    pub catalog: &'a CatalogSnapshot,
    pub source: PathBuf,
    pub root: PathBuf,
    pub workers: usize,
    pub family_jobs: usize,
    pub case_jobs: usize,
    pub families: Vec<FamilyJob>,
    /// Capture before an application lock wait when execution follows planning.
    /// The convenience path captures the same inputs before starting workers.
    pub inputs: Option<&'a BatchInputs>,
}

pub struct BatchInputRequest<'a> {
    pub catalog: &'a CatalogSnapshot,
    pub source: &'a Path,
    pub required: &'a [String],
}

pub struct BatchInputs {
    catalog: CatalogSnapshot,
    required: BTreeSet<String>,
    view: Arc<crate::input_view::InputView>,
}

impl BatchInputs {
    pub fn validate_current(&self) -> Result<()> {
        self.catalog.validate_current()?;
        decomp_source::context::limited(|| self.view.validate_current())
    }
}

/// A terminal family outcome, including a pre-execution refusal with no run.
/// Completion observers receive only an immutable projection of the result.
pub struct FamilyCompletion {
    family: String,
    seconds: f64,
    result: Result<RegisteredRun>,
}

impl FamilyCompletion {
    pub fn family(&self) -> &str {
        &self.family
    }
    pub fn seconds(&self) -> f64 {
        self.seconds
    }
    pub fn result(&self) -> &Result<RegisteredRun> {
        &self.result
    }
    pub fn into_result(self) -> Result<RegisteredRun> {
        self.result
    }
}

pub struct BatchRun {
    families: Vec<FamilyCompletion>,
}

fn publish_completion(
    completed: &(impl Fn(&FamilyCompletion) -> Result<()> + Sync),
    outcome: &FamilyCompletion,
) -> Result<()> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| completed(outcome)))
        .unwrap_or_else(|_| Err(anyhow::anyhow!("batch completion observer panicked")))
}
impl BatchRun {
    /// Canonical request order, independent of completion/publication order.
    pub fn families(&self) -> &[FamilyCompletion] {
        &self.families
    }
    pub fn into_families(self) -> Vec<FamilyCompletion> {
        self.families
    }
}

pub struct FamilyEvidence {
    pub family: String,
    pub verdict: PathBuf,
    pub manifest_sha256: String,
    pub kind: String,
}

pub struct BatchAdmission<'a> {
    pub catalog: &'a CatalogSnapshot,
    pub required: &'a [String],
    pub source: PathBuf,
    pub evidence: Vec<FamilyEvidence>,
}

/// Complete independently admitted membership. No report deserializer can
/// manufacture this value, and later admissions start with a fresh byte cache.
pub struct AuthenticatedBatch {
    families: BTreeMap<String, AuthenticatedFamily>,
    statistics: crate::CatalogStatistics,
}
impl AuthenticatedBatch {
    pub fn families(&self) -> &BTreeMap<String, AuthenticatedFamily> {
        &self.families
    }
    pub fn statistics(&self) -> crate::CatalogStatistics {
        self.statistics
    }
}

pub(crate) fn check_admitted_binding(
    facts: &FamilyFacts,
    admitted: &family::AdmittedFamily,
    checkout: &Path,
    declarations: &Declarations,
) -> Result<()> {
    crate::identities::check_registered_computation(&facts.entry().kind, admitted)?;
    if facts.entry().kind == "plm-instructions" {
        let rom = admitted.original.rom.as_ref().context("generated PLM ROM missing")?;
        let (expected, _) = crate::plm_fixtures::generated_contents(checkout, &rom.file)?;
        let bytes = crate::fixture_format::encode(&expected)?;
        let hash = translation_execution::model::sha256(&bytes);
        ensure!(
            declarations.decode(bytes, hash)?.manifest == admitted.original,
            "registered generated PLM evidence differs from complete generator"
        );
    } else {
        ensure!(
            admitted.original.as_ref() == facts.declaration(),
            "admitted declaration differs from required registered family"
        );
        ensure!(
            admitted
                .report
                .artifacts
                .get(crate::fixture_format::RUN_MANIFEST)
                .is_some_and(|hash| hash == facts.manifest_sha256()),
            "admitted manifest identity differs from captured registration"
        );
    }
    Ok(())
}

impl OfflineFamilies {
    pub fn capture_batch_inputs(&self, request: BatchInputRequest<'_>) -> Result<BatchInputs> {
        let required = request.required.iter().cloned().collect::<BTreeSet<_>>();
        ensure!(required.len() == request.required.len(), "duplicate required family");
        request.catalog.validate_current()?;
        let facts = request.required.iter().map(|name| request.catalog.family(name)).collect::<Result<Vec<_>>>()?;
        let manifests = facts.iter().map(FamilyFacts::declaration).collect::<Vec<_>>();
        let view = crate::input_view::InputView::capture(
            self.host_inputs(),
            request.catalog.checkout(),
            request.source,
            &manifests,
            facts.iter().any(|facts| facts.entry().kind == "plm-instructions"),
            request.catalog.declarations(),
        )?;
        request.catalog.validate_current()?;
        Ok(BatchInputs { catalog: request.catalog.clone(), required, view })
    }
    /// Publish each completion while sibling families are still running. The
    /// observer cannot alter required work; its errors are returned after all
    /// admitted workers have joined so callers can safely perform cleanup.
    pub fn verify_batch(
        &self,
        request: BatchVerification<'_>,
        completed: impl Fn(&FamilyCompletion) -> Result<()> + Sync,
    ) -> Result<BatchRun> {
        ensure!((1..=8).contains(&request.family_jobs), "batch jobs must be 1..8");
        ensure!(
            (1..=64).contains(&request.case_jobs) && (1..=64).contains(&request.workers),
            "invalid batch worker extent"
        );
        let mut names = BTreeSet::new();
        let mut outputs = BTreeSet::new();
        for job in &request.families {
            ensure!(names.insert(&job.family), "duplicate required family");
            ensure!(outputs.insert(&job.output), "duplicate family output directory");
            crate::catalog::entry(request.catalog.catalog(), &job.family)?;
        }
        request.catalog.validate_current()?;
        if request.families.is_empty() {
            return Ok(BatchRun { families: Vec::new() });
        }
        let owned_inputs;
        let captured = if let Some(inputs) = request.inputs {
            ensure!(
                inputs.catalog.same_capture(request.catalog)
                    && inputs.required == names.into_iter().cloned().collect()
                    && std::fs::canonicalize(&request.source)? == inputs.view.source(),
                "batch captured input requirement differs"
            );
            inputs.validate_current()?;
            inputs
        } else {
            let required = request.families.iter().map(|job| job.family.clone()).collect::<Vec<_>>();
            owned_inputs = self.capture_batch_inputs(BatchInputRequest {
                catalog: request.catalog,
                source: &request.source,
                required: &required,
            })?;
            &owned_inputs
        };
        let graph = crate::work_graph::Graph::new(
            request.catalog,
            &captured.view,
            &request.families,
            request.root.clone(),
            request.workers,
            request.family_jobs,
        )?;
        let family_names = request.families.iter().map(|job| job.family.clone()).collect::<Vec<_>>();
        let jobs: Vec<_> = request.families.into_iter().map(|job| Mutex::new(Some(job))).collect();
        let outcomes = Mutex::new(BTreeMap::new());
        let publication_errors = Mutex::new(Vec::new());
        let metrics = tool_execution::metrics::active();
        std::thread::scope(|scope| {
            let executors = (0..request.workers).map(|_| scope.spawn(|| graph.serve())).collect::<Vec<_>>();
            let families = (0..request.family_jobs.min(jobs.len()))
                .map(|_| {
                    scope.spawn(|| {
                        let _active = metrics.as_ref().map(|context| context.enter());
                        while let Some(index) = graph.next_family() {
                            let node = graph.node(index);
                            let job = &jobs[index];
                            let job = job.lock().unwrap().take().expect("one family worker per index");
                            let started = Instant::now();
                            let result = (|| -> Result<RegisteredRun> {
                                let metrics_directory = job.output.join("metrics");
                                std::fs::create_dir_all(&metrics_directory)?;
                                let mut metrics_run = tool_execution::metrics::Run::start(
                                    tool_execution::metrics::sidecar(&metrics_directory, &job.family),
                                    "batch-family",
                                )?;
                                metrics_run.context.configuration("family", &job.family);
                                metrics_run.context.configuration("case-jobs", request.case_jobs);
                                let result = self.verify_registered_captured(
                                    RegisteredRequest {
                                        checkout: request.catalog.checkout().into(),
                                        source: request.source.clone(),
                                        root: request.root.clone(),
                                        workers: request.workers,
                                        case_jobs: request.case_jobs,
                                        family: job.family.clone(),
                                        output: job.output,
                                        previous: job.previous,
                                    },
                                    request.catalog,
                                    Some(&captured.view),
                                    Some(node),
                                );
                                let error = result
                                    .as_ref()
                                    .err()
                                    .map(|error| format!("{error:#}"))
                                    .or_else(|| result.as_ref().ok()?.run.report().error.clone());
                                let finished =
                                    metrics_run.finish(tool_execution::metrics::outcome(error.as_deref()), error);
                                if result.is_ok() {
                                    finished?;
                                }
                                result
                            })();
                            let outcome = FamilyCompletion {
                                family: job.family,
                                seconds: started.elapsed().as_secs_f64(),
                                result,
                            };
                            if outcome
                                .result
                                .as_ref()
                                .is_ok_and(|result| result.run.report().status == "offline-verified")
                            {
                                node.phase(crate::work_graph::Phase::Complete);
                            } else {
                                let reason = match &outcome.result {
                                    Err(error) => format!("{error:#}"),
                                    Ok(result) => {
                                        result.run.report().error.clone().unwrap_or_else(|| "family rejected".into())
                                    }
                                };
                                graph.stop(&reason);
                                node.phase(crate::work_graph::Phase::Rejected);
                            }
                            if let Err(error) = crate::work_graph::FamilyNode::metadata(Some(node), || {
                                publish_completion(&completed, &outcome)
                            }) {
                                graph.stop(&format!("completion publication failed: {error:#}"));
                                publication_errors.lock().unwrap().push(format!("{error:#}"));
                            }
                            outcomes.lock().unwrap().insert(index, outcome);
                        }
                    })
                })
                .collect::<Vec<_>>();
            for family in families {
                if family.join().is_err() {
                    graph.stop("batch family worker panicked");
                    publication_errors.lock().unwrap().push("batch family worker panicked".into());
                }
            }
            graph.close();
            for executor in executors {
                if executor.join().is_err() {
                    publication_errors.lock().unwrap().push("batch Fixture executor panicked".into());
                }
            }
        });
        let mut outcomes = outcomes.into_inner().unwrap();
        for (index, family) in family_names.into_iter().enumerate() {
            if outcomes.contains_key(&index) {
                continue;
            }
            let outcome = FamilyCompletion {
                family,
                seconds: 0.,
                result: Err(graph
                    .check_admission()
                    .err()
                    .unwrap_or_else(|| anyhow::anyhow!("batch family did not complete"))),
            };
            if let Err(error) = publish_completion(&completed, &outcome) {
                publication_errors.lock().unwrap().push(format!("{error:#}"));
            }
            outcomes.insert(index, outcome);
        }
        let errors = publication_errors.into_inner().unwrap();
        ensure!(errors.is_empty(), "batch completion publication failed: {}", errors.join("; "));
        request.catalog.validate_current()?;
        captured.validate_current()?;
        Ok(BatchRun { families: outcomes.into_values().collect() })
    }

    pub fn authenticate_batch(&self, request: BatchAdmission<'_>) -> Result<AuthenticatedBatch> {
        let required = request.required.iter().collect::<BTreeSet<_>>();
        ensure!(required.len() == request.required.len(), "duplicate required family");
        let mut evidence = BTreeMap::new();
        for item in request.evidence {
            ensure!(required.contains(&item.family), "extra family evidence");
            ensure!(evidence.insert(item.family.clone(), item).is_none(), "duplicate family evidence");
        }
        ensure!(evidence.len() == required.len(), "incomplete batch evidence membership");
        request.catalog.validate_current()?;
        let declarations = Declarations::standard();
        let mut families = BTreeMap::new();
        crate::inputs::with_host(self.host_inputs(), || {
            for name in request.required {
                let item = evidence.remove(name).context("required family evidence missing")?;
                let facts = request.catalog.family(name)?;
                ensure!(
                    item.kind == facts.entry().kind && item.manifest_sha256 == facts.manifest_sha256(),
                    "batch evidence registration or manifest identity differs"
                );
                // A distinct family admission scope preserves existing per-family
                // byte limits instead of sharing a 1 GiB whole-batch output cap.
                let admitted = family::admit_with_source(
                    AuthenticateRequest { checkout: request.catalog.checkout().into(), verdict: item.verdict },
                    &request.source,
                    &declarations,
                )?;
                check_admitted_binding(&facts, &admitted, request.catalog.checkout(), &declarations)?;
                families.insert(name.clone(), AuthenticatedFamily::from_admitted(admitted));
            }
            Ok(())
        })?;
        request.catalog.validate_current()?;
        Ok(AuthenticatedBatch { families, statistics: declarations.counters.statistics() })
    }
}
