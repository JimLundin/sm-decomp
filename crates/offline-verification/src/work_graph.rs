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
    idle: Vec<(usize, Worker)>,
    residents: usize,
    families: Vec<Family>,
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

enum Action {
    Pair { family: usize, task: usize, work: Arc<Fixtures>, worker: Worker },
    Blocked { family: usize, task: usize, work: Arc<Fixtures>, reason: String },
    Retire { family: usize, worker: Worker },
    Done,
}

fn framed(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
}

fn diagnostic(message: &str) -> String {
    let mut end = message.len().min(4096);
    while !message.is_char_boundary(end) {
        end -= 1;
    }
    message[..end].to_owned()
}

impl Graph {
    pub(crate) fn new(
        catalog: &CatalogSnapshot,
        inputs: &InputView,
        jobs: &[FamilyJob],
        root: PathBuf,
        capacity: usize,
        family_jobs: usize,
    ) -> Result<Self> {
        let declarations = catalog.declarations().clone();
        let bytes = jobs
            .iter()
            .try_fold(PROJECTION_BYTES * capacity.min(family_jobs) as u64, |total, job| {
                total.checked_add((job.family.len() + job.output.as_os_str().len()) as u64 * 2 + 1024)
            })
            .context("batch graph metadata extent overflow")?;
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
            context
                .configuration("batch-graph-projection", "advisory only; never read for producer admission or reuse");
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
            self.stop(&format!("{error:#}"));
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
            self.stop(&format!("{error:#}"));
        }
        let state = self.state.lock().unwrap();
        if let Some(reason) = &state.stopped {
            bail!("batch stopped new admission: {reason}");
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
                state.stopped.get_or_insert_with(|| diagnostic(&format!("{error:#}")));
            }
            let selected = state
                .groups
                .range(state.cursor..)
                .chain(state.groups.range(..state.cursor))
                .find(|(_, group)| !group.ready.is_empty() && group.running < group.work.jobs)
                .map(|(id, _)| *id);
            if let Some(family) = selected {
                let group = &state.groups[&family];
                let (&task, &blocked) = group.ready.first_key_value().unwrap();
                let work = group.work.clone();
                let reason = state.stopped.clone().or_else(|| blocked.then(|| "predecessor did not verify".into()));
                let action = if let Some(reason) = reason {
                    Action::Blocked { family, task, work, reason }
                } else {
                    let worker = if let Some(index) = state.idle.iter().position(|(id, _)| *id == family) {
                        state.idle.swap_remove(index).1
                    } else if state.residents < self.capacity {
                        state.residents += 1;
                        state.groups.get_mut(&family).unwrap().residents += 1;
                        Worker::new(&work.run, work.fork, &work.context)
                    } else if !state.idle.is_empty() {
                        // Retire an idle helper pair before creating its replacement.
                        // Waiting for a predecessor or cache never retains a lease.
                        let index =
                            state.idle.iter().position(|(id, _)| state.groups[id].ready.is_empty()).unwrap_or(0);
                        let (family, worker) = state.idle.swap_remove(index);
                        return Action::Retire { family, worker };
                    } else {
                        state = self.changed.wait_timeout(state, Duration::from_millis(20)).unwrap().0;
                        continue;
                    };
                    Action::Pair { family, task, work, worker }
                };
                let group = state.groups.get_mut(&family).unwrap();
                group.ready.remove(&task);
                group.running += 1;
                state.cursor = family + 1;
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
        if let Err(error) = &result {
            state.stopped.get_or_insert_with(|| diagnostic(&format!("{error:#}")));
        }
        let group = state.groups.get_mut(&family).expect("resident helper owns its group");
        if let Err(error) = result {
            group.outcomes.push((group.work.indices.len(), Err(error)));
        }
        group.residents -= 1;
        state.residents -= 1;
        self.changed.notify_all();
    }

    fn settled(&self, family: usize, task: usize, result: Result<CaseRecord>, worker: Option<Worker>, blocked: bool) {
        let failed = result.is_err();
        let mut state = self.state.lock().unwrap();
        if let Err(error) = &result {
            state.stopped.get_or_insert_with(|| diagnostic(&format!("{error:#}")));
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
                    retired.push(state.idle.swap_remove(index).1);
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
                        let (_lease, wait) =
                            crate::acquire_lease(&self.root, self.capacity).context("shared batch Fixture startup")?;
                        // A stop while this task waited for another batch's
                        // capacity must not admit a fresh Rust/C invocation.
                        self.check_admission()?;
                        worker.set_wait(wait);
                        invoked = true;
                        work.execute(task, &mut worker, &self.declarations)
                    }))
                    .unwrap_or_else(|_| Err(anyhow::anyhow!("batch Fixture worker panicked")));
                    let result = if !invoked {
                        work.blocked(
                            task,
                            &diagnostic(&format!("{:#}", result.err().expect("unadmitted pair has an error"))),
                        )
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
            ensure!(toml::to_string(&projection)?.len() as u64 <= PROJECTION_BYTES, "graph projection extent exceeded");
            fs::create_dir_all(path.parent().context("graph projection parent missing")?)?;
            crate::atomic_toml(&path, &projection)
        })();
        // Losing advisory progress cannot change producer, deadline or reuse
        // decisions. Report the unavailable projection through bounded metrics.
        if let Err(error) = result
            && let Some(context) = metrics::active()
        {
            context.configuration("batch-graph-projection-error", diagnostic(&format!("{error:#}")));
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
            state.groups.insert(
                self.index,
                Group {
                    work: Arc::new(work),
                    ready,
                    dependents,
                    remaining: count,
                    running: 0,
                    residents: 0,
                    outcomes: Vec::with_capacity(count),
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
