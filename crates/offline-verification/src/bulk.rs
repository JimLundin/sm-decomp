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
        let manifests = facts
            .iter()
            .filter(|facts| facts.entry().kind != "plm-instructions")
            .map(FamilyFacts::declaration)
            .collect::<Vec<_>>();
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
            let families =
                (0..request.family_jobs.min(jobs.len()))
                    .map(|_| {
                        scope.spawn(|| {
                            let _active = metrics.as_ref().map(|context| context.enter());
                            while let Some(index) = graph.next_family() {
                                let node = graph.node(index);
                                let job = &jobs[index];
                                let job = job.lock().unwrap().take().expect("one family worker per index");
                                let started = Instant::now();
                                let mut result = (|| -> Result<RegisteredRun> {
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
                                    let error =
                                        result.as_ref().err().map(crate::work_graph::error_diagnostic).or_else(|| {
                                            result
                                                .as_ref()
                                                .ok()?
                                                .run
                                                .report()
                                                .error
                                                .as_deref()
                                                .map(crate::work_graph::diagnostic)
                                        });
                                    let finished =
                                        metrics_run.finish(tool_execution::metrics::outcome(error.as_deref()), error);
                                    if result.is_ok() {
                                        finished?;
                                    }
                                    result
                                })();
                                if let Err(error) = node.retain_completed(&result) {
                                    graph.stop(&crate::work_graph::error_diagnostic(&error));
                                    result = Err(error);
                                }
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
                                    match &outcome.result {
                                        Err(error) => graph.stop(&crate::work_graph::error_diagnostic(error)),
                                        Ok(result) => graph
                                            .stop(result.run.report().error.as_deref().unwrap_or("family rejected")),
                                    }
                                    node.phase(crate::work_graph::Phase::Rejected);
                                }
                                if let Err(error) = crate::work_graph::FamilyNode::metadata(Some(node), || {
                                    publish_completion(&completed, &outcome)
                                }) {
                                    graph.stop(&crate::work_graph::error_diagnostic(&error));
                                    publication_errors.lock().unwrap().push(
                                        node.publication_error(error).unwrap_or_else(|error| format!("{error:#}")),
                                    );
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
                publication_errors
                    .lock()
                    .unwrap()
                    .push(graph.node(index).publication_error(error).unwrap_or_else(|error| format!("{error:#}")));
            }
            outcomes.insert(index, outcome);
        }
        let errors = publication_errors.into_inner().unwrap();
        if !errors.is_empty() {
            let prefix = "batch completion publication failed: ";
            let bytes = errors.iter().try_fold(prefix.len(), |bytes, error| {
                bytes
                    .checked_add(error.len())
                    .and_then(|n| n.checked_add(2))
                    .context("batch publication diagnostic extent overflow")
            })?;
            let mut joined = String::with_capacity(bytes);
            joined.push_str(prefix);
            for (index, error) in errors.into_iter().enumerate() {
                if index > 0 {
                    joined.push_str("; ");
                }
                joined.push_str(&error);
            }
            return Err(anyhow::Error::msg(joined));
        }
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

pub(crate) mod work_graph {
    //! Invocation-owned ordinary work graph. No caller-supplied executable tasks,
    //! durable attempts, or producer ownership live here. Its sidecars are bounded
    //! advisory projections; execution never reads them to decide what may run.
    use crate::{
        CatalogSnapshot, FamilyJob,
        declarations::Declarations,
        family_model::{CaseRecord, Manifest},
        fixture_execution::{self, Worker},
        input_view::InputView,
        recipe::Recipe,
    };
    use anyhow::{Context as _, Result, bail, ensure};
    use serde::Serialize;
    use sha2::{Digest, Sha256};
    use std::{
        collections::BTreeMap,
        fs,
        os::unix::ffi::OsStrExt,
        path::PathBuf,
        sync::{Arc, Condvar, Mutex},
        time::Duration,
    };
    use tool_execution::metrics::{self, Context};

    const PROJECTION_BYTES: u64 = 64 * 1024;
    // Keep pristine helpers warm for a bounded number of ready pairs. A pending
    // round-robin turn may be bypassed only by these finite remaining credits.
    const HELPER_QUANTUM: usize = 8;

    #[derive(Clone, Copy, Debug, Serialize)]
    #[serde(rename_all = "kebab-case")]
    pub(crate) enum Phase {
        Resolving,
        ReuseAdmission,
        Preparing,
        Building,
        Fixtures,
        Publishing,
        Admission,
        Complete,
        Rejected,
    }

    #[derive(Default, Clone, Serialize)]
    struct FixtureProgress {
        declared: usize,
        order_sha256: String,
        passed: usize,
        failed: usize,
        blocked: usize,
    }

    struct Family {
        name: String,
        manifest_sha256: String,
        projection: PathBuf,
        fixtures: FixtureProgress,
    }

    struct State {
        next: usize,
        stopped: Option<String>,
        closed: bool,
        cursor: usize,
        groups: BTreeMap<usize, Group>,
        idle: Vec<(usize, Reusable)>,
        residents: usize,
        families: Vec<Family>,
        // Completed records move into the batch's canonical results. Keep their
        // reservation through the whole invocation, after their group is removed.
        result_reservations: BTreeMap<usize, Arc<Mutex<Reservation>>>,
    }

    /// Graph scheduling metadata shares the invocation's existing resident budget.
    /// The reservation is released only when its last group/result owner is gone.
    struct Reservation {
        declarations: Arc<Declarations>,
        bytes: u64,
    }
    impl Reservation {
        fn new(declarations: &Arc<Declarations>, bytes: u64) -> Result<Self> {
            declarations.reserve_metadata(bytes)?;
            Ok(Self { declarations: declarations.clone(), bytes })
        }
        fn resize(&mut self, bytes: u64) -> Result<()> {
            if bytes > self.bytes {
                self.declarations.reserve_metadata(bytes - self.bytes)?;
            } else {
                self.declarations.release_metadata(self.bytes - bytes);
            }
            self.bytes = bytes;
            Ok(())
        }
        fn grow(&mut self, bytes: u64) -> Result<()> {
            self.resize(self.bytes.checked_add(bytes).context("Fixture result reservation overflow")?)
        }
    }
    impl Drop for Reservation {
        fn drop(&mut self) {
            self.declarations.release_metadata(self.bytes);
        }
    }

    pub(crate) struct Graph {
        state: Mutex<State>,
        changed: Condvar,
        metadata: Mutex<usize>,
        metadata_changed: Condvar,
        metadata_limit: usize,
        capacity: usize,
        root: PathBuf,
        identity: String,
        declarations: Arc<Declarations>,
        _reservation: Reservation,
    }

    #[derive(Clone, Copy)]
    pub(crate) struct FamilyNode<'a> {
        graph: &'a Graph,
        index: usize,
    }

    pub(crate) struct Metadata<'a>(&'a Graph);
    impl Drop for Metadata<'_> {
        fn drop(&mut self) {
            *self.0.metadata.lock().unwrap() -= 1;
            self.0.metadata_changed.notify_one();
        }
    }

    /// Ready work retains indices into one owned normalized declaration. Large
    /// packed Fixture payloads are never copied into queue entries.
    pub(crate) struct Fixtures {
        manifest: Arc<Manifest>,
        recipe: Option<Recipe>,
        view: Arc<InputView>,
        run: PathBuf,
        context: Context,
        host: crate::HostInputs,
        indices: Vec<(usize, usize)>,
        jobs: usize,
        fork: bool,
        _reservation: Reservation,
        result_reservation: Arc<Mutex<Reservation>>,
        result_bounds: Mutex<Vec<u64>>,
    }

    impl Fixtures {
        pub(crate) fn new(
            manifest: Arc<Manifest>,
            recipe: Option<Recipe>,
            view: Arc<InputView>,
            run: PathBuf,
            jobs: usize,
            fork: bool,
        ) -> Result<Self> {
            let count = manifest
                .routines
                .iter()
                .try_fold(0u64, |count, routine| count.checked_add(routine.cases.len() as u64))
                .context("batch Fixture count overflow")?;
            let declarations = crate::declarations::current().context("missing batch declarations")?;
            let reservation = Reservation::new(
                &declarations,
                count
                    .checked_mul(1024)
                    .and_then(|bytes| bytes.checked_add(4096))
                    .context("batch Fixture metadata extent overflow")?,
            )?;
            let indices = crate::family::ordered_task_indices(&manifest)?;
            let result_bounds = indices
                .iter()
                .map(|&(routine, case)| {
                    let routine = &manifest.routines[routine];
                    // Closed recipes retain two subject entries per map, short hash
                    // strings and declaration names. Check the actual retained extent
                    // before queueing a record, including future recipe extensions.
                    result_bound(routine, &routine.cases[case])
                })
                .collect::<Vec<_>>();
            let result_bytes = result_bounds
                .iter()
                .try_fold(0u64, |sum, bytes| sum.checked_add(*bytes))
                .context("batch Fixture result reservation overflow")?;
            let result_reservation = Arc::new(Mutex::new(Reservation::new(&declarations, result_bytes)?));
            Ok(Self {
                manifest,
                recipe,
                view,
                run,
                context: metrics::active().context("missing batch Fixture metrics scope")?,
                host: crate::inputs::host_inputs()?,
                indices,
                jobs,
                fork,
                _reservation: reservation,
                result_reservation,
                result_bounds: Mutex::new(result_bounds),
            })
        }

        fn task(&self, index: usize) -> fixture_execution::Task<'_> {
            let (routine, case) = self.indices[index];
            (&self.manifest.routines[routine], &self.manifest.routines[routine].cases[case], routine)
        }

        fn execute(&self, index: usize, worker: &mut Worker, declarations: &Arc<Declarations>) -> Result<CaseRecord> {
            let (routine, case, routine_index) = self.task(index);
            declarations.enter(|| {
                crate::inputs::with_host(&self.host, || {
                    self.view.enter(|| {
                        crate::recipe::fixture(
                            self.recipe.as_ref(),
                            &self.manifest,
                            routine,
                            case,
                            routine_index,
                            &self.run,
                            crate::recipe::FixtureExecution::Ordinary { worker },
                        )
                    })
                })
            })
        }

        fn blocked(&self, index: usize, reason: &str) -> Result<CaseRecord> {
            let case = self.task(index).1;
            let directory = self.run.join(&case.name);
            fs::create_dir_all(&directory)?;
            #[derive(Serialize)]
            struct Blocked<'a> {
                blocked_by: &'a str,
                reason: &'a str,
            }
            let previous = case.continue_from.as_deref().unwrap_or("batch");
            crate::atomic_toml(&directory.join("blocked.toml"), &Blocked { blocked_by: previous, reason })?;
            bail!("fixture {} blocked by {previous}: {reason}", case.name)
        }

        fn reserve_result(&self, index: usize, bytes: u64) -> Result<()> {
            let mut bounds = self.result_bounds.lock().unwrap();
            if bytes > bounds[index] {
                self.result_reservation.lock().unwrap().grow(bytes - bounds[index])?;
                bounds[index] = bytes;
            }
            Ok(())
        }

        fn bounded_result(&self, index: usize, result: Result<CaseRecord>) -> Result<CaseRecord> {
            match result {
                Err(error) => {
                    let diagnostic = own_error(error, |bytes| self.reserve_result(index, bytes))?;
                    let path = self.run.join(&self.task(index).1.name).join("rejection.toml");
                    let rejection = if path.is_file() {
                        // The rejection record is a second owned result, alongside
                        // its complete error and the eventual joined diagnostic.
                        // Reserve bounded TOML read/parse working storage before
                        // reopening it, then charge its actual retained shape.
                        let extent = fs::metadata(&path)?.len();
                        let declarations = self.result_reservation.lock().unwrap().declarations.clone();
                        let _parsing = Reservation::new(
                            &declarations,
                            extent
                                .checked_mul(64)
                                .and_then(|n| n.checked_add(4096))
                                .context("Fixture rejection parse extent overflow")?,
                        )?;
                        let record: CaseRecord = tool_execution::bounded::object(&path)?;
                        self.reserve_result(
                            index,
                            diagnostic_extent(diagnostic.capacity())?
                                .checked_add(retained_case_bytes(&record)?)
                                .context("Fixture rejection result extent overflow")?,
                        )?;
                        Some(record)
                    } else {
                        None
                    };
                    Err(anyhow::Error::msg(FixtureError { diagnostic, rejection }))
                }
                Ok(record) => {
                    self.reserve_result(index, retained_case_bytes(&record)?)?;
                    Ok(record)
                }
            }
        }
    }

    /// Canonical errors carry their retained rejection record across the graph
    /// seam. The family must not reopen another unaccounted copy during publication.
    #[derive(Debug)]
    pub(crate) struct FixtureError {
        diagnostic: String,
        rejection: Option<CaseRecord>,
    }
    impl FixtureError {
        pub(crate) fn take_rejection(&mut self) -> Option<CaseRecord> {
            self.rejection.take()
        }
    }
    impl std::fmt::Display for FixtureError {
        fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            out.write_str(&self.diagnostic)
        }
    }

    pub(crate) fn formatted_error_bytes(error: &anyhow::Error) -> Result<usize> {
        struct Counter(usize);
        impl std::fmt::Write for Counter {
            fn write_str(&mut self, text: &str) -> std::fmt::Result {
                self.0 = self.0.checked_add(text.len()).ok_or(std::fmt::Error)?;
                Ok(())
            }
        }
        use std::fmt::Write as _;
        let mut counter = Counter(0);
        write!(&mut counter, "{error:#}").context("canonical diagnostic extent overflow")?;
        Ok(counter.0)
    }

    fn diagnostic_extent(capacity: usize) -> Result<u64> {
        // The complete message and exact-sized aggregate coexist. Container,
        // separator and allocator overhead are separate from String capacity.
        (capacity as u64)
            .checked_mul(2)
            .and_then(|n| n.checked_add(512))
            .context("canonical diagnostic reservation overflow")
    }

    fn own_error(error: anyhow::Error, reserve: impl FnOnce(u64) -> Result<()>) -> Result<String> {
        match error.downcast::<String>() {
            Ok(text) => {
                reserve(diagnostic_extent(text.capacity())?)?;
                Ok(text)
            }
            Err(error) => {
                // Count through fmt::Write without constructing a temporary full
                // diagnostic. Reserve before allocating its canonical owned copy.
                let bytes = formatted_error_bytes(&error)?;
                reserve(
                    diagnostic_extent(bytes)?
                        .checked_add(bytes as u64)
                        .context("diagnostic conversion extent overflow")?,
                )?;
                let mut text = String::with_capacity(bytes);
                use std::fmt::Write as _;
                write!(&mut text, "{error:#}")?;
                Ok(text)
            }
        }
    }

    fn retained_case_bytes(record: &CaseRecord) -> Result<u64> {
        // Closed subject maps have zero or two entries. A nonempty map has one
        // leaf; 1 KiB covers its eleven key/value slots and allocator overhead for
        // all four concrete value types. Larger future shapes reserve one such
        // node per entry until their representation is reviewed.
        let mut bytes = std::mem::size_of::<CaseRecord>() as u64 + 128;
        for count in [record.returns.len(), record.after_sha256.len(), record.results.len(), record.spawn_results.len()]
        {
            let nodes = if count == 0 {
                0
            } else if count <= 2 {
                1
            } else {
                count
            };
            bytes = bytes.checked_add(nodes as u64 * 1024).context("Fixture result map extent overflow")?;
        }
        for value in [&record.routine, &record.name, &record.before_sha256]
            .into_iter()
            .chain(record.capture_after_sha256.iter())
            .chain(record.returns.keys())
            .chain(record.after_sha256.keys())
            .chain(record.after_sha256.values())
            .chain(record.results.keys())
            .chain(record.spawn_results.keys())
        {
            bytes = bytes.checked_add(value.capacity() as u64).context("Fixture result extent overflow")?;
        }
        Ok(bytes)
    }

    fn result_bound(routine: &crate::family_model::Routine, case: &crate::family_model::Case) -> u64 {
        // Ordinary records fit this provisional allowance. Full diagnostics and
        // rejection records grow it by their measured extent when needed; they
        // are never shortened to fit. Completed families shrink to owned capacity.
        8 * 1024 + 512 + 2 * (routine.symbol.len() + case.name.len()) as u64
    }

    struct Group {
        work: Arc<Fixtures>,
        ready: BTreeMap<usize, bool>,
        dependents: Vec<Vec<usize>>,
        remaining: usize,
        running: usize,
        residents: usize,
        outcomes: Vec<(usize, Result<CaseRecord>)>,
    }

    struct Reusable {
        worker: Worker,
        remaining: usize,
    }

    enum Action {
        Pair { family: usize, task: usize, work: Arc<Fixtures>, worker: Reusable },
        Blocked { family: usize, task: usize, work: Arc<Fixtures>, reason: String },
        Retire { family: usize, worker: Worker },
        Done,
    }

    fn framed(hash: &mut Sha256, bytes: &[u8]) {
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }

    pub(crate) fn diagnostic(message: &str) -> String {
        let mut end = message.len().min(4096);
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        message[..end].to_owned()
    }

    pub(crate) fn error_diagnostic(error: &anyhow::Error) -> String {
        struct Advisory(String);
        impl std::fmt::Write for Advisory {
            fn write_str(&mut self, text: &str) -> std::fmt::Result {
                let mut count = text.len().min(4096 - self.0.len());
                while !text.is_char_boundary(count) {
                    count -= 1;
                }
                self.0.push_str(&text[..count]);
                Ok(())
            }
        }
        use std::fmt::Write as _;
        let mut out = Advisory(String::with_capacity(4096));
        write!(&mut out, "{error:#}").expect("bounded diagnostic writer cannot fail");
        out.0
    }

    impl Graph {
        fn metadata_bytes(jobs: &[FamilyJob], capacity: usize, family_jobs: usize) -> Result<u64> {
            jobs.iter()
                .try_fold(PROJECTION_BYTES * capacity.min(family_jobs) as u64 + capacity as u64 * 8192, |total, job| {
                    total.checked_add((job.family.len() + job.output.as_os_str().len()) as u64 * 2 + 1024)
                })
                .context("batch graph metadata extent overflow")
        }

        pub(crate) fn new(
            catalog: &CatalogSnapshot,
            inputs: &InputView,
            jobs: &[FamilyJob],
            root: PathBuf,
            capacity: usize,
            family_jobs: usize,
        ) -> Result<Self> {
            let declarations = catalog.declarations().clone();
            let bytes = Self::metadata_bytes(jobs, capacity, family_jobs)?;
            let reservation = Reservation::new(&declarations, bytes)?;
            let mut hash = Sha256::new();
            framed(&mut hash, b"offline-batch-graph-v1");
            for (path, digest) in inputs.identities() {
                framed(&mut hash, path.as_os_str().as_bytes());
                framed(&mut hash, digest.as_bytes());
            }
            let families = jobs
                .iter()
                .map(|job| {
                    let facts = catalog.family(&job.family)?;
                    framed(&mut hash, job.family.as_bytes());
                    framed(&mut hash, facts.manifest_sha256().as_bytes());
                    framed(&mut hash, facts.entry().kind.as_bytes());
                    Ok(Family {
                        name: job.family.clone(),
                        manifest_sha256: facts.manifest_sha256().into(),
                        projection: job.output.join("work-graph.toml"),
                        fixtures: FixtureProgress::default(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            if let Some(context) = metrics::active() {
                context.configuration("batch-graph", "ready-families-v1");
                context.configuration("batch-fixture-workers", capacity);
                context.configuration("batch-resident-helper-limit", 2 * capacity);
                context.configuration("batch-metadata-workers", capacity.min(family_jobs));
                context.configuration("batch-family-window", family_jobs);
                context.configuration("batch-helper-reuse-quantum", HELPER_QUANTUM);
                context.configuration(
                    "batch-graph-projection",
                    "advisory only; never read for producer admission or reuse",
                );
            }
            Ok(Self {
                state: Mutex::new(State {
                    next: 0,
                    stopped: None,
                    closed: false,
                    cursor: 0,
                    groups: BTreeMap::new(),
                    idle: Vec::new(),
                    residents: 0,
                    families,
                    result_reservations: BTreeMap::new(),
                }),
                changed: Condvar::new(),
                metadata: Mutex::new(0),
                metadata_changed: Condvar::new(),
                metadata_limit: capacity.min(family_jobs),
                capacity,
                root,
                identity: format!("{:x}", hash.finalize()),
                declarations,
                _reservation: reservation,
            })
        }

        pub(crate) fn node(&self, index: usize) -> FamilyNode<'_> {
            FamilyNode { graph: self, index }
        }

        pub(crate) fn next_family(&self) -> Option<usize> {
            if let Err(error) = metrics::check_cancelled() {
                self.stop(&error_diagnostic(&error));
            }
            let mut state = self.state.lock().unwrap();
            if state.stopped.is_some() || state.next == state.families.len() {
                return None;
            }
            let index = state.next;
            state.next += 1;
            Some(index)
        }

        pub(crate) fn stop(&self, reason: &str) {
            self.state.lock().unwrap().stopped.get_or_insert_with(|| diagnostic(reason));
            self.changed.notify_all();
        }

        pub(crate) fn check_admission(&self) -> Result<()> {
            if let Err(error) = metrics::check_cancelled() {
                self.stop(&error_diagnostic(&error));
                return Err(error);
            }
            let state = self.state.lock().unwrap();
            if state.stopped.is_some() {
                // The primary full diagnostic belongs to its canonical outcome.
                // Do not turn the bounded advisory projection into a truncated
                // diagnostic for every subsequently blocked family or Fixture.
                bail!("batch stopped new admission after a prior terminal outcome");
            }
            Ok(())
        }

        pub(crate) fn close(&self) {
            self.state.lock().unwrap().closed = true;
            self.changed.notify_all();
        }

        fn metadata(&self) -> Metadata<'_> {
            let mut active = self.metadata.lock().unwrap();
            while *active >= self.metadata_limit {
                active = self.metadata_changed.wait(active).unwrap();
            }
            *active += 1;
            Metadata(self)
        }

        fn select(&self) -> Action {
            let mut state = self.state.lock().unwrap();
            loop {
                if let Err(error) = metrics::check_cancelled() {
                    state.stopped.get_or_insert_with(|| error_diagnostic(&error));
                }
                let selected = state
                    .groups
                    .range(state.cursor..)
                    .chain(state.groups.range(..state.cursor))
                    .find(|(_, group)| !group.ready.is_empty() && group.running < group.work.jobs)
                    .map(|(id, _)| *id);
                if let Some(turn) = selected {
                    let mut family = turn;
                    if state.stopped.is_none()
                        && state.residents == self.capacity
                        && !state.idle.iter().any(|(id, _)| *id == turn)
                    {
                        // Preserve the pending fair turn while amortizing helper
                        // startup. Every bypass consumes a finite resident credit;
                        // it never renews a helper or moves the fair cursor.
                        if let Some((&id, _)) =
                            state.groups.range(state.cursor..).chain(state.groups.range(..state.cursor)).find(
                                |(id, group)| {
                                    !group.ready.is_empty()
                                        && group.running < group.work.jobs
                                        && state.idle.iter().any(|(owner, worker)| owner == *id && worker.remaining > 0)
                                },
                            )
                        {
                            family = id;
                        }
                    }
                    let fair_turn = family == turn;
                    let group = &state.groups[&family];
                    let (&task, &blocked) = group.ready.first_key_value().unwrap();
                    let work = group.work.clone();
                    let reason = state
                        .stopped
                        .as_ref()
                        .map(|_| "batch stopped new admission after a prior terminal outcome".to_owned())
                        .or_else(|| blocked.then(|| "predecessor did not verify".into()));
                    let action = if let Some(reason) = reason {
                        Action::Blocked { family, task, work, reason }
                    } else {
                        let mut worker = if let Some(index) = state
                            .idle
                            .iter()
                            .position(|(id, worker)| *id == family && (fair_turn || worker.remaining > 0))
                        {
                            let mut worker = state.idle.swap_remove(index).1;
                            if fair_turn {
                                worker.remaining = HELPER_QUANTUM;
                            }
                            worker
                        } else if state.residents < self.capacity {
                            state.residents += 1;
                            state.groups.get_mut(&family).unwrap().residents += 1;
                            Reusable {
                                worker: Worker::new(&work.run, work.fork, &work.context),
                                remaining: HELPER_QUANTUM,
                            }
                        } else if !state.idle.is_empty() {
                            // Retire an idle helper pair before creating its replacement.
                            // Waiting for a predecessor or cache never retains a lease.
                            let index =
                                state.idle.iter().position(|(id, _)| state.groups[id].ready.is_empty()).unwrap_or(0);
                            let (family, worker) = state.idle.swap_remove(index);
                            return Action::Retire { family, worker: worker.worker };
                        } else {
                            state = self.changed.wait_timeout(state, Duration::from_millis(20)).unwrap().0;
                            continue;
                        };
                        worker.remaining -= 1;
                        Action::Pair { family, task, work, worker }
                    };
                    let group = state.groups.get_mut(&family).unwrap();
                    group.ready.remove(&task);
                    group.running += 1;
                    if fair_turn {
                        state.cursor = family + 1;
                    }
                    return action;
                }
                if state.closed {
                    return Action::Done;
                }
                state = self.changed.wait_timeout(state, Duration::from_millis(20)).unwrap().0;
            }
        }

        fn retire(&self, family: usize, mut worker: Worker) {
            // Drop reaps all remaining helpers even if an explicit stop fails.
            // Only then may this family's metrics and evidence be finalized.
            let result = worker.finish();
            drop(worker);
            let mut state = self.state.lock().unwrap();
            if let Err(error) = result {
                let budget = state.result_reservations[&family].clone();
                let error = match own_error(error, |bytes| budget.lock().unwrap().grow(bytes)) {
                    Ok(text) => anyhow::Error::msg(text),
                    Err(error) => error,
                };
                state.stopped.get_or_insert_with(|| error_diagnostic(&error));
                let group = state.groups.get_mut(&family).expect("resident helper owns its group");
                group.outcomes.push((group.work.indices.len(), Err(error)));
            }
            let group = state.groups.get_mut(&family).expect("resident helper owns its group");
            group.residents -= 1;
            state.residents -= 1;
            self.changed.notify_all();
        }

        fn settled(
            &self,
            family: usize,
            task: usize,
            result: Result<CaseRecord>,
            worker: Option<Reusable>,
            blocked: bool,
        ) {
            let mut state = self.state.lock().unwrap();
            let result = state.groups[&family].work.bounded_result(task, result);
            let failed = result.is_err();
            if let Err(error) = &result {
                state.stopped.get_or_insert_with(|| error_diagnostic(error));
            }
            let group = state.groups.get_mut(&family).unwrap();
            for &dependent in &group.dependents[task] {
                group.ready.insert(dependent, failed);
            }
            group.remaining -= 1;
            group.running -= 1;
            group.outcomes.push((task, result));
            let done = group.remaining == 0;
            let progress = &mut state.families[family].fixtures;
            if blocked {
                progress.blocked += 1;
            } else if failed {
                progress.failed += 1;
            } else {
                progress.passed += 1;
            }
            if let Some(worker) = worker {
                state.idle.push((family, worker));
            }
            let mut retired = Vec::new();
            if done {
                let mut index = 0;
                while index < state.idle.len() {
                    if state.idle[index].0 == family {
                        retired.push(state.idle.swap_remove(index).1.worker);
                    } else {
                        index += 1;
                    }
                }
            }
            self.changed.notify_all();
            drop(state);
            for worker in retired {
                self.retire(family, worker);
            }
        }

        pub(crate) fn serve(&self) {
            loop {
                match self.select() {
                    Action::Done => return,
                    Action::Retire { family, worker } => self.retire(family, worker),
                    Action::Blocked { family, task, work, reason } => {
                        let _entered = work.context.enter();
                        self.settled(family, task, work.blocked(task, &reason), None, true);
                    }
                    Action::Pair { family, task, work, mut worker } => {
                        let _entered = work.context.enter();
                        let mut invoked = false;
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            self.check_admission()?;
                            let (_lease, wait) = crate::acquire_lease(&self.root, self.capacity)
                                .context("shared batch Fixture startup")?;
                            // A stop while this task waited for another batch's
                            // capacity must not admit a fresh Rust/C invocation.
                            self.check_admission()?;
                            worker.worker.set_wait(wait);
                            invoked = true;
                            work.execute(task, &mut worker.worker, &self.declarations)
                        }))
                        .unwrap_or_else(|_| Err(anyhow::anyhow!("batch Fixture worker panicked")));
                        let result = if !invoked {
                            match own_error(result.err().expect("unadmitted pair has an error"), |bytes| {
                                work.reserve_result(task, bytes)
                            }) {
                                Ok(message) => work.blocked(task, &message),
                                Err(error) => Err(error),
                            }
                        } else {
                            result
                        };
                        self.settled(family, task, result, Some(worker), !invoked);
                    }
                }
            }
        }
    }

    impl<'a> FamilyNode<'a> {
        pub(crate) fn retain_completed(self, result: &Result<crate::RegisteredRun>) -> Result<()> {
            let bytes = match result {
                Ok(result) => {
                    let report = result.run.report();
                    let initial = report.error.as_ref().map_or(0, |error| error.capacity() as u64)
                        + (report.cases.capacity() - report.cases.len()) as u64
                            * std::mem::size_of::<CaseRecord>() as u64;
                    report.cases.iter().try_fold(initial, |bytes, record| {
                        bytes
                            .checked_add(retained_case_bytes(record)?)
                            .context("completed family result extent overflow")
                    })?
                }
                Err(error) => {
                    error
                        .downcast_ref::<String>()
                        .map(String::capacity)
                        .map(Ok)
                        .unwrap_or_else(|| formatted_error_bytes(error))? as u64
                        + 128
                }
            };
            let mut state = self.graph.state.lock().unwrap();
            if let Some(reservation) = state.result_reservations.get_mut(&self.index) {
                reservation.lock().unwrap().resize(bytes)?;
            } else {
                // Reused evidence submits no new Fixtures but returns records that
                // the batch retains until its invocation is complete.
                state
                    .result_reservations
                    .insert(self.index, Arc::new(Mutex::new(Reservation::new(&self.graph.declarations, bytes)?)));
            }
            Ok(())
        }

        pub(crate) fn report_error(self, error: anyhow::Error, report: &crate::Verdict) -> Result<String> {
            let case_bytes = report.cases.iter().try_fold(
                (report.cases.capacity() - report.cases.len()) as u64 * std::mem::size_of::<CaseRecord>() as u64,
                |bytes, record| {
                    bytes.checked_add(retained_case_bytes(record)?).context("rejected family result extent overflow")
                },
            )?;
            own_error(error, |bytes| {
                let bytes = case_bytes.checked_add(bytes).context("rejected family diagnostic extent overflow")?;
                let mut state = self.graph.state.lock().unwrap();
                if let Some(reservation) = state.result_reservations.get(&self.index) {
                    let mut reservation = reservation.lock().unwrap();
                    let required = reservation.bytes.max(bytes);
                    reservation.resize(required)
                } else {
                    state
                        .result_reservations
                        .insert(self.index, Arc::new(Mutex::new(Reservation::new(&self.graph.declarations, bytes)?)));
                    Ok(())
                }
            })
        }

        pub(crate) fn publication_error(self, error: anyhow::Error) -> Result<String> {
            own_error(error, |bytes| {
                let mut state = self.graph.state.lock().unwrap();
                if let Some(reservation) = state.result_reservations.get(&self.index) {
                    reservation.lock().unwrap().grow(bytes)
                } else {
                    state
                        .result_reservations
                        .insert(self.index, Arc::new(Mutex::new(Reservation::new(&self.graph.declarations, bytes)?)));
                    Ok(())
                }
            })
        }
        pub(crate) fn check_admission(self) -> Result<()> {
            self.graph.check_admission()
        }

        pub(crate) fn stop(self, reason: &str) {
            self.graph.stop(reason);
        }

        pub(crate) fn metadata_permit(self) -> Metadata<'a> {
            self.graph.metadata()
        }

        pub(crate) fn phase(self, phase: Phase) {
            #[derive(Serialize)]
            struct Projection<'a> {
                schema: &'static str,
                authority: &'static str,
                batch_sha256: &'a str,
                family_index: usize,
                family: String,
                manifest_sha256: String,
                phase: Phase,
                fixtures: FixtureProgress,
                stopped: Option<String>,
            }
            let _metadata = self.graph.metadata();
            let state = self.graph.state.lock().unwrap();
            let family = &state.families[self.index];
            let projection = Projection {
                schema: "offline-work-graph-v1",
                authority: "advisory; authenticate original requirements and evidence",
                batch_sha256: &self.graph.identity,
                family_index: self.index,
                family: family.name.clone(),
                manifest_sha256: family.manifest_sha256.clone(),
                phase,
                fixtures: family.fixtures.clone(),
                stopped: state.stopped.clone(),
            };
            let path = family.projection.clone();
            drop(state);
            let result = (|| -> Result<()> {
                ensure!(
                    toml::to_string(&projection)?.len() as u64 <= PROJECTION_BYTES,
                    "graph projection extent exceeded"
                );
                fs::create_dir_all(path.parent().context("graph projection parent missing")?)?;
                crate::atomic_toml(&path, &projection)
            })();
            // Losing advisory progress cannot change producer, deadline or reuse
            // decisions. Report the unavailable projection through bounded metrics.
            if let Err(error) = result
                && let Some(context) = metrics::active()
            {
                context.configuration("batch-graph-projection-error", error_diagnostic(&error));
            }
        }

        pub(crate) fn metadata<T>(node: Option<Self>, action: impl FnOnce() -> Result<T>) -> Result<T> {
            let _permit = node.map(|node| node.graph.metadata());
            action()
        }

        pub(crate) fn fixtures(self, work: Fixtures) -> Result<Vec<(usize, Result<CaseRecord>)>> {
            let tasks = (0..work.indices.len()).map(|index| work.task(index)).collect::<Vec<_>>();
            let count = tasks.len();
            ensure!(count > 0 && (1..=64).contains(&work.jobs), "invalid batch Fixture group");
            // Fixtures reserved queue, dependency and result-shell metadata before
            // allocating indices. Its manifest already owns the shared payloads.
            let predecessors = fixture_execution::predecessors(&tasks)?;
            let mut order = Sha256::new();
            framed(&mut order, b"offline-fixture-order-v1");
            let mut ready = BTreeMap::new();
            let mut dependents = vec![Vec::new(); count];
            for (index, predecessor) in predecessors.into_iter().enumerate() {
                framed(&mut order, tasks[index].1.name.as_bytes());
                framed(&mut order, &predecessor.map(|p| p as u64).unwrap_or(u64::MAX).to_le_bytes());
                if let Some(predecessor) = predecessor {
                    dependents[predecessor].push(index);
                } else {
                    ready.insert(index, false);
                }
            }
            work.context.configuration("fixture-execution", "shared-batch-dependency-v1");
            work.context.configuration("fixture-process", if work.fork { "fork" } else { "fresh" });
            drop(tasks);
            {
                let mut state = self.graph.state.lock().unwrap();
                ensure!(!state.groups.contains_key(&self.index), "batch family submitted duplicate Fixture work");
                state.families[self.index].fixtures = FixtureProgress {
                    declared: count,
                    order_sha256: format!("{:x}", order.finalize()),
                    ..Default::default()
                };
                state.result_reservations.insert(self.index, work.result_reservation.clone());
                state.groups.insert(
                    self.index,
                    Group {
                        work: Arc::new(work),
                        ready,
                        dependents,
                        remaining: count,
                        running: 0,
                        residents: 0,
                        // A terminal retirement error can add at most one row per
                        // resident executor. Avoid doubling this large vector.
                        outcomes: Vec::with_capacity(count + self.graph.capacity),
                    },
                );
                self.graph.changed.notify_all();
            }
            self.phase(Phase::Fixtures);
            let mut state = self.graph.state.lock().unwrap();
            loop {
                let group = &state.groups[&self.index];
                if group.remaining == 0 && group.residents == 0 {
                    let mut group = state.groups.remove(&self.index).unwrap();
                    group.outcomes.sort_by_key(|(index, _)| *index);
                    return Ok(group.outcomes);
                }
                state = self.graph.changed.wait(state).unwrap();
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::family_model::{CallbackMetrics, SpawnMetrics};

        // Check the standard-library representation independently of the retained
        // shape formula. Only this thread's explicit measurement window is counted;
        // other unit tests and the production allocator are unaffected.
        struct ShapeAllocator;
        thread_local! {
            static SHAPE_ALLOCATIONS: std::cell::Cell<Option<(usize, usize)>> = const { std::cell::Cell::new(None) };
        }
        #[global_allocator]
        static SHAPE_ALLOCATOR: ShapeAllocator = ShapeAllocator;
        fn allocated(bytes: usize) {
            let _ = SHAPE_ALLOCATIONS.try_with(|counter| {
                if let Some((count, extent)) = counter.get() {
                    counter.set(Some((count + 1, extent + bytes)));
                }
            });
        }
        unsafe impl std::alloc::GlobalAlloc for ShapeAllocator {
            unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
                let pointer = unsafe { std::alloc::System.alloc(layout) };
                if !pointer.is_null() {
                    allocated(layout.size());
                }
                pointer
            }
            unsafe fn dealloc(&self, pointer: *mut u8, layout: std::alloc::Layout) {
                unsafe { std::alloc::System.dealloc(pointer, layout) }
            }
            unsafe fn realloc(&self, pointer: *mut u8, layout: std::alloc::Layout, bytes: usize) -> *mut u8 {
                let pointer = unsafe { std::alloc::System.realloc(pointer, layout, bytes) };
                if !pointer.is_null() {
                    allocated(bytes);
                }
                pointer
            }
        }

        #[test]
        fn actual_closed_subject_maps_fit_retained_leaf_allowance() {
            fn check<V: Default>() {
                let entries = [(String::from("rust"), V::default()), (String::from("c"), V::default())];
                SHAPE_ALLOCATIONS.with(|counter| counter.set(Some((0, 0))));
                let map = BTreeMap::from(entries);
                let (count, bytes) = SHAPE_ALLOCATIONS.with(|counter| counter.replace(None).unwrap());
                std::hint::black_box(&map);
                assert_eq!(count, 1, "closed two-subject map no longer has one leaf allocation");
                assert!(bytes + 32 <= 1024, "actual map leaf plus allocator overhead exceeded allowance: {bytes}");
            }
            check::<u64>();
            check::<String>();
            check::<CallbackMetrics>();
            check::<SpawnMetrics>();
        }

        fn captured_catalog() -> Result<(PathBuf, CatalogSnapshot)> {
            let checkout = std::fs::canonicalize(
                std::env::var_os("SM_GRAPH_MEMORY_CHECKOUT")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")),
            )?;
            let catalog = crate::catalog::capture(crate::CatalogRequest::new(&checkout)?)?;
            Ok((checkout, catalog))
        }

        #[derive(Debug)]
        struct Extent {
            family: String,
            cases: usize,
            retained: u64,
            provisional: u64,
            queue: u64,
        }

        fn extents(checkout: &std::path::Path, catalog: &CatalogSnapshot) -> Result<Vec<Extent>> {
            let facts =
                catalog.catalog().families.keys().map(|name| catalog.family(name)).collect::<Result<Vec<_>>>()?;
            let mut counts = Vec::new();
            for facts in &facts {
                let declared = facts.declaration();
                let recipe = crate::recipe::for_family(checkout, &facts.entry().manifest, declared)
                    .with_context(|| format!("catalog result shape recipe {}", facts.name()))?;
                let manifest = crate::recipe::normalize_manifest(&checkout, declared.clone(), recipe.as_ref())?;
                let mut retained = 0;
                let mut provisional = 0;
                let indices = crate::family::ordered_task_indices(&manifest)?;
                for &(routine, case) in &indices {
                    let routine = &manifest.routines[routine];
                    let case = &routine.cases[case];
                    let mut record =
                        CaseRecord { routine: routine.symbol.clone(), name: case.name.clone(), ..Default::default() };
                    match recipe.as_ref() {
                        Some(Recipe::Plm) => {
                            record.spawn_results = BTreeMap::from([
                                ("rust".into(), SpawnMetrics::default()),
                                ("c".into(), SpawnMetrics::default()),
                            ])
                        }
                        Some(Recipe::Callback | Recipe::PowerBomb | Recipe::GameHost | Recipe::Ridley) => {
                            record.results = BTreeMap::from([
                                ("rust".into(), CallbackMetrics::default()),
                                ("c".into(), CallbackMetrics::default()),
                            ])
                        }
                        _ => {
                            record.returns = BTreeMap::from([("rust".into(), 0), ("c".into(), 0)]);
                            record.before_sha256 = "0".repeat(64);
                            record.after_sha256 =
                                BTreeMap::from([("rust".into(), "0".repeat(64)), ("c".into(), "0".repeat(64))]);
                            record.capture_after_sha256 = case.expected_after_file.as_ref().map(|_| "0".repeat(64));
                        }
                    }
                    retained += retained_case_bytes(&record)?;
                    provisional += result_bound(routine, case);
                }
                // Vec growth can leave unused CaseRecord slots in the returned
                // report. Account its next power-of-two capacity conservatively.
                let unused = indices.len().max(4).next_power_of_two() - indices.len();
                retained += unused as u64 * std::mem::size_of::<CaseRecord>() as u64;
                counts.push(Extent {
                    family: facts.name().to_owned(),
                    cases: indices.len(),
                    retained,
                    provisional,
                    queue: indices.len() as u64 * 1024 + 4096,
                });
            }
            counts.sort_by_key(|extent| std::cmp::Reverse(extent.provisional - extent.retained + extent.queue));
            Ok(counts)
        }

        fn totals(extents: &[Extent]) -> Result<(u64, u64)> {
            let case_count = extents.iter().map(|extent| extent.cases).sum::<usize>();
            let completed_bytes = extents.iter().map(|extent| extent.retained).sum::<u64>();
            // Any eight active families, even when all others have completed. This
            // is a stricter window than canonical admission normally produces.
            let active_extra =
                extents.iter().take(8).map(|extent| extent.provisional - extent.retained + extent.queue).sum::<u64>();
            println!(
                "Catalog graph result shapes: families={}, cases={case_count}, completed_case_bytes={completed_bytes}, maximum_eight_active_extra_bytes={active_extra}",
                extents.len()
            );
            for extent in extents.iter().take(8) {
                println!(
                    "{}: cases={}, retained={}, provisional={}, queues={}",
                    extent.family, extent.cases, extent.retained, extent.provisional, extent.queue
                );
            }
            ensure!(case_count >= 150_204 && extents.len() >= 177, "memory control omitted the complete catalog");
            Ok((completed_bytes, active_extra))
        }

        #[test]
        #[ignore = "read-only declaration shape measurement; no input capture or producers"]
        fn complete_catalog_result_shapes() -> Result<()> {
            let (checkout, catalog) = captured_catalog()?;
            let extents = extents(&checkout, &catalog)?;
            let (completed, active) = totals(&extents)?;
            let declarations = catalog.statistics().retained_bytes;
            println!(
                "Declaration bytes={declarations}, declarations_and_results={}",
                declarations + completed + active
            );
            Ok(())
        }

        #[test]
        #[ignore = "complete production input capture and graph memory control; requires retained project inputs"]
        fn full_catalog_graph_reservations_fit_standard_budget() -> Result<()> {
            let (checkout, catalog) = captured_catalog()?;
            let extents = extents(&checkout, &catalog)?;
            let (completed_bytes, active_extra) = totals(&extents)?;
            let host = crate::HostInputs::new(&[("Cargo.toml", include_str!("../../../Cargo.toml"))], "1".repeat(64))?;
            let offline = crate::OfflineFamilies::new(host);
            let required = catalog.catalog().families.keys().cloned().collect::<Vec<_>>();
            let source = std::fs::canonicalize(checkout.join("deps/snesrev-sm"))?;
            let inputs = offline.capture_batch_inputs(crate::BatchInputRequest {
                catalog: &catalog,
                source: &source,
                required: &required,
            })?;
            let jobs = required
                .into_iter()
                .map(|family| FamilyJob {
                    output: checkout.join("unused-memory-control").join(&family),
                    family,
                    previous: None,
                })
                .collect::<Vec<_>>();
            let graph = Reservation::new(catalog.declarations(), Graph::metadata_bytes(&jobs, 4, 8)?)?;
            let base_bytes = catalog.statistics().retained_bytes;
            let combined = base_bytes + completed_bytes + active_extra;
            println!(
                "Complete captured graph reservation: base_bytes={base_bytes}, completed_bytes={completed_bytes}, active_extra_bytes={active_extra}, combined_bytes={combined}, limit={}",
                tool_execution::bounded::DEFAULT_BYTES
            );
            let completed = Reservation::new(catalog.declarations(), completed_bytes)?;
            let active = Reservation::new(catalog.declarations(), active_extra)?;
            ensure!(
                catalog.statistics().retained_bytes == combined,
                "structural reservation did not cover the complete active window"
            );
            drop((active, completed, graph, inputs));
            Ok(())
        }
    }
}
