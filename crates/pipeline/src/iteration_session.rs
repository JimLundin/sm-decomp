use crate::readiness_metadata::toml_value_decode;
use crate::{
    digest, family, iteration_increment as increment, iteration_selection, lock, readiness,
    readiness_contract as contract, readiness_origins as origins, readiness_reuse as reuse,
    readiness_run::{Run, source_inventory},
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use toml::Value;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Running,
    Complete,
    RepairNeeded,
    DecisionRequired,
    Stopped,
    Error,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Baseline {
    pub run_id: String,
    pub run_path: PathBuf,
    pub source_sha256: String,
    pub generated: u64,
    pub interpreted: u64,
    pub validated_keys: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Selection {
    pub run_path: PathBuf,
    pub outcome: pipeline_model::SelectionOutcome,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Candidate {
    pub published: bool,
    pub status: Status,
    pub reason: String,
    pub run_path: Option<PathBuf>,
    pub baseline: Option<Baseline>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    pub sequence: u64,
    pub operation: String,
    pub evidence: String,
    pub sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct State {
    pub schema_version: u32,
    pub session_id: String,
    pub status: Status,
    pub reason: String,
    pub count: Option<u64>,
    pub target_generated: Option<u64>,
    pub accepted_count: u64,
    pub increment_id: Option<String>,
    pub baseline: Option<Baseline>,
    pub initial_baseline: Option<Baseline>,
    pub selection: Option<Selection>,
    pub preparation: Option<PathBuf>,
    pub candidate: Option<Candidate>,
    pub pending_run: Option<PathBuf>,
    pub events: Vec<Event>,
}
#[derive(Serialize, Deserialize)]
struct EventRecord {
    sequence: u64,
    operation: String,
    previous_sha256: Option<String>,
    result: toml::Value,
}
#[derive(Serialize, Deserialize)]
struct RequestRecord {
    count: Option<u64>,
    target_generated: Option<u64>,
}
#[derive(Serialize, Deserialize)]
struct FinishRecord {
    status: Status,
    reason: String,
}
fn transition(state: &mut State, name: &str, result: &toml::Value) -> Result<()> {
    ensure!(state.events.is_empty() == (name == "request"), "session must begin with one request");
    match name {
        "request" => {
            let request: RequestRecord = result.clone().try_into()?;
            ensure!(
                request.count == state.count
                    && request.target_generated == state.target_generated
                    && request.count.is_some() != request.target_generated.is_some()
                    && request.count.is_none_or(|n| n > 0)
                    && request.target_generated.is_none_or(|n| n > 0),
                "session request differs"
            );
        }
        "resume" => {
            state.status = Status::Running;
            state.reason.clear();
        }
        "baseline" => {
            ensure!(state.increment_id.is_none(), "cannot replace unresolved predecessor");
            let baseline: Baseline = result.clone().try_into()?;
            if state.initial_baseline.is_none() {
                state.initial_baseline = Some(baseline.clone());
            }
            state.baseline = Some(baseline);
        }
        "increment" => {
            ensure!(
                state.baseline.is_some() && state.increment_id.is_none(),
                "increment lacks baseline or is already unresolved"
            );
            let id: String = result.clone().try_into()?;
            safe_id(&id)?;
            state.increment_id = Some(id);
        }
        "select" => {
            ensure!(
                state.increment_id.is_some() && state.selection.is_none(),
                "selection outside unresolved increment"
            );
            state.selection = Some(result.clone().try_into()?);
        }
        "prepare" => {
            ensure!(state.selection.is_some() && state.preparation.is_none(), "preparation outside selected increment");
            state.preparation = Some(result.clone().try_into()?);
        }
        "validate" | "retry" => {
            ensure!(
                state.preparation.is_some() && (name != "retry" || state.candidate.is_some()),
                "validation lacks preparation/candidate"
            );
            state.candidate = Some(result.clone().try_into()?);
            state.pending_run = None;
        }
        "validation-intent" => {
            ensure!(state.preparation.is_some() && state.pending_run.is_none(), "validation intent out of order");
            state.pending_run = Some(result.clone().try_into()?);
        }
        "accept" => {
            let candidate: Candidate = result.clone().try_into()?;
            ensure!(
                state.candidate.as_ref() == Some(&candidate)
                    && candidate.published
                    && candidate.status == Status::Complete
                    && candidate.run_path.is_some()
                    && candidate.baseline.is_some()
                    && candidate.run_path.as_ref() == candidate.baseline.as_ref().map(|b| &b.run_path),
                "acceptance differs from candidate"
            );
            state.accepted_count = state.accepted_count.checked_add(1).context("session count overflow")?;
            state.baseline = candidate.baseline;
            state.increment_id = None;
            state.selection = None;
            state.preparation = None;
            state.candidate = None;
            state.pending_run = None;
        }
        "finish" => {
            let finish: FinishRecord = result.clone().try_into()?;
            ensure!(finish.status != Status::Running, "terminal event cannot be running");
            if finish.status == Status::Complete {
                ensure!(
                    state.increment_id.is_none()
                        && (state.count.is_some_and(|n| state.accepted_count >= n)
                            || state
                                .target_generated
                                .is_some_and(|n| state.baseline.as_ref().is_some_and(|b| b.generated >= n))),
                    "completion lacks measured target"
                );
            }
            state.status = finish.status;
            state.reason = finish.reason;
        }
        _ => anyhow::bail!("unknown session event {name}"),
    }
    Ok(())
}
fn safe_id(id: &str) -> Result<()> {
    ensure!(
        !id.is_empty() && id.len() <= 96 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
        "unsafe session/increment ID"
    );
    Ok(())
}
fn write<T: Serialize>(path: &Path, record: &T, replace: bool) -> Result<()> {
    let temporary = path.with_extension(format!("{}-pending", std::process::id()));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
        file.write_all(toml::to_string_pretty(record)?.as_bytes())?;
        file.sync_all()?;
        if !replace {
            ensure!(!path.exists(), "immutable session event already exists");
        }
        fs::rename(&temporary, path)?;
        File::open(path.parent().unwrap())?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}
trait Operations {
    fn baseline(&mut self) -> Result<Baseline>;
    fn select(&mut self, baseline: &Baseline) -> Result<Selection>;
    fn prepare(&mut self, selection: &Selection) -> Result<PathBuf>;
    fn validate(&mut self, state: &State, retry: bool) -> Result<Candidate>;
    fn authenticate(&mut self, state: &State, candidate: &Candidate) -> Result<()>;
    fn allocate(&mut self, _: &State) -> Result<Option<PathBuf>> {
        Ok(None)
    }
    fn recover(&mut self, _: &State) -> Result<Candidate> {
        anyhow::bail!("unexpected pending production run")
    }
}
struct Session {
    path: PathBuf,
    state: State,
}
impl Session {
    fn create(root: &Path, count: Option<u64>, target: Option<u64>) -> Result<Self> {
        ensure!(
            count.is_some() != target.is_some() && count.is_none_or(|n| n > 0) && target.is_none_or(|n| n > 0),
            "specify one positive count or generated target"
        );
        let id = format!("session-{}-{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(), std::process::id());
        let path = root.join("out/iteration-sessions").join(&id);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::create_dir(&path)?;
        let state = State {
            schema_version: 2,
            session_id: id,
            status: Status::Running,
            reason: String::new(),
            count,
            target_generated: target,
            accepted_count: 0,
            increment_id: None,
            baseline: None,
            initial_baseline: None,
            selection: None,
            preparation: None,
            candidate: None,
            pending_run: None,
            events: vec![],
        };
        let mut session = Self { path, state };
        session.record("request", &RequestRecord { count, target_generated: target })?;
        Ok(session)
    }
    fn resume(root: &Path, id: &str) -> Result<Self> {
        safe_id(id)?;
        ensure!(id.starts_with("session-"), "invalid session ID");
        let path = root.join("out/iteration-sessions").join(id);
        let state: State = pipeline_model::read(&path.join("state.toml"))?;
        ensure!(state.schema_version == 2 && state.session_id == id, "session identity differs");
        let mut session = Self { path, state };
        session.recover_events()?;
        Ok(session)
    }
    fn save(&self) -> Result<()> {
        write(&self.path.join("state.toml"), &self.state, true)
    }
    fn recover_events(&mut self) -> Result<()> {
        let mut replay = State {
            schema_version: 2,
            session_id: self.state.session_id.clone(),
            status: Status::Running,
            reason: String::new(),
            count: self.state.count,
            target_generated: self.state.target_generated,
            accepted_count: 0,
            increment_id: None,
            baseline: None,
            initial_baseline: None,
            selection: None,
            preparation: None,
            candidate: None,
            pending_run: None,
            events: vec![],
        };
        let mut previous = None;
        let mut paths = fs::read_dir(&self.path)?.map(|e| Ok(e?.path())).collect::<Result<Vec<_>>>()?;
        paths.retain(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.as_bytes().first().is_some_and(u8::is_ascii_digit) && n.ends_with(".toml"))
        });
        paths.sort();
        ensure!(paths.len() >= self.state.events.len(), "session event evidence missing");
        for (i, path) in paths.iter().enumerate() {
            let record: EventRecord = pipeline_model::read(path)?;
            safe_id(&record.operation)?;
            let event = Event {
                sequence: i as u64 + 1,
                operation: record.operation.clone(),
                evidence: format!("{:04}-{}.toml", i + 1, record.operation),
                sha256: digest(path)?,
            };
            ensure!(
                path.file_name().and_then(|n| n.to_str()) == Some(&event.evidence),
                "session event ordering differs"
            );
            if let Some(indexed) = self.state.events.get(i) {
                ensure!(*indexed == event, "session event checksum/index differs");
            }
            ensure!(
                record.sequence == event.sequence && record.previous_sha256 == previous,
                "session event chain differs"
            );
            transition(&mut replay, &record.operation, &record.result)?;
            previous = Some(event.sha256.clone());
            replay.events.push(event);
            if i + 1 == self.state.events.len() {
                ensure!(replay == self.state, "session state differs from independently replayed events");
            }
        }
        ensure!(!replay.events.is_empty(), "session request evidence missing");
        let changed = self.state != replay;
        self.state = replay;
        if changed {
            self.save()?;
        }
        Ok(())
    }
    fn record<T: Serialize>(&mut self, name: &str, result: &T) -> Result<()> {
        let sequence = self.state.events.len() as u64 + 1;
        let evidence = format!("{sequence:04}-{name}.toml");
        let record = EventRecord {
            sequence,
            operation: name.into(),
            previous_sha256: self.state.events.last().map(|e| e.sha256.clone()),
            result: toml::Value::try_from(result)?,
        };
        let mut next = self.state.clone();
        transition(&mut next, name, &record.result)?;
        write(&self.path.join(&evidence), &record, false)?;
        self.state = next;
        self.state.events.push(Event {
            sequence,
            operation: name.into(),
            evidence: evidence.clone(),
            sha256: digest(&self.path.join(evidence))?,
        });
        self.save()
    }
    fn finish(&mut self, status: Status, reason: String) -> Result<State> {
        self.record("finish", &FinishRecord { status, reason })?;
        fs::write(
            self.path.join("brief.md"),
            format!(
                "{:?}: {} accepted increment(s).\n\n{}\n\nUnresolved increment: {}.\n\n[State and ordered evidence](state.toml)\n",
                self.state.status,
                self.state.accepted_count,
                self.state.reason,
                self.state.increment_id.as_deref().unwrap_or("none")
            ),
        )?;
        Ok(self.state.clone())
    }
    fn accept(&mut self, operations: &mut impl Operations, candidate: Candidate) -> Result<bool> {
        if !candidate.published {
            return Ok(false);
        }
        ensure!(
            candidate.status == Status::Complete && candidate.baseline.is_some(),
            "published result lacks complete accepted baseline"
        );
        operations.authenticate(&self.state, &candidate)?;
        self.record("accept", &candidate)?;
        Ok(true)
    }
    fn run(&mut self, operations: &mut impl Operations) -> Result<State> {
        let _guard = lock(&self.path.join("session.lock"))?;
        self.state = pipeline_model::read(&self.path.join("state.toml"))?;
        self.recover_events()?;
        let mut replay = self.state.clone();
        replay.accepted_count = 0;
        replay.increment_id = None;
        replay.baseline = None;
        replay.initial_baseline = None;
        replay.selection = None;
        replay.preparation = None;
        replay.candidate = None;
        replay.pending_run = None;
        replay.events.clear();
        for event in &self.state.events {
            let record: EventRecord = pipeline_model::read(&self.path.join(&event.evidence))?;
            if record.operation == "accept" {
                let candidate: Candidate = record.result.clone().try_into()?;
                operations.authenticate(&replay, &candidate)?;
            }
            transition(&mut replay, &record.operation, &record.result)?;
            replay.events.push(event.clone());
        }
        if self.state.status == Status::Complete {
            return Ok(self.state.clone());
        }
        self.record("resume", &true)?;
        let outcome = (|| -> Result<State> {
            loop {
                if self.state.pending_run.is_some() {
                    let candidate = operations.recover(&self.state)?;
                    self.record(if self.state.candidate.is_some() { "retry" } else { "validate" }, &candidate)?;
                }
                if self.state.candidate.as_ref().is_some_and(|c| c.published) {
                    let candidate = self.state.candidate.clone().unwrap();
                    self.accept(operations, candidate)?;
                } else if self.state.candidate.is_some() {
                    if let Some(owner) = operations.allocate(&self.state)? {
                        self.record("validation-intent", &owner)?;
                    }
                    let candidate = operations.validate(&self.state, true)?;
                    self.record("retry", &candidate)?;
                    if !self.accept(operations, candidate.clone())? {
                        return self.finish(candidate.status, candidate.reason);
                    }
                }
                let baseline = if self.state.increment_id.is_some() {
                    self.state.baseline.clone().context("unresolved baseline missing")?
                } else {
                    let baseline = operations.baseline()?;
                    self.record("baseline", &baseline)?;
                    baseline
                };
                if self.state.count.is_some_and(|n| self.state.accepted_count >= n)
                    || self.state.target_generated.is_some_and(|n| baseline.generated >= n)
                {
                    return self.finish(Status::Complete, "requested measured scope completed".into());
                }
                if self.state.increment_id.is_none() {
                    self.record(
                        "increment",
                        &format!("increment-{}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()),
                    )?;
                }
                let selection = if let Some(selected) = &self.state.selection {
                    selected.clone()
                } else {
                    let selected = operations.select(&baseline)?;
                    self.record("select", &selected)?;
                    selected
                };
                if selection.outcome != pipeline_model::SelectionOutcome::SupportedRoutine {
                    return self.finish(
                        Status::DecisionRequired,
                        format!(
                            "selection requires {:?}; retained evidence identifies the next route",
                            selection.outcome
                        ),
                    );
                }
                if self.state.preparation.is_none() {
                    let preparation = operations.prepare(&selection)?;
                    self.record("prepare", &preparation)?;
                }
                if let Some(owner) = operations.allocate(&self.state)? {
                    self.record("validation-intent", &owner)?;
                }
                let candidate = operations.validate(&self.state, false)?;
                self.record("validate", &candidate)?;
                if !self.accept(operations, candidate.clone())? {
                    return self.finish(candidate.status, candidate.reason);
                }
            }
        })();
        match outcome {
            Ok(state) => Ok(state),
            Err(error) => self.finish(
                if crate::readiness_run::was_interrupted() { Status::Stopped } else { Status::Error },
                format!("{error:#}"),
            ),
        }
    }
}
struct Production {
    root: PathBuf,
    timeout: u64,
    original: Option<PathBuf>,
}
impl Production {
    fn authenticate_candidate(&self, state: &State, candidate: &Candidate) -> Result<()> {
        let owner = candidate.run_path.as_ref().context("published candidate path missing")?;
        let verified = reuse::verify_accepted(owner, true)?;
        readiness::verify_origins(owner, &verified)?;
        let manifest = &verified["manifest"];
        let baseline = candidate.baseline.as_ref().context("published baseline missing")?;
        let coverage = &verified["verdict"]["checks"]["changed-entry-coverage"];
        ensure!(
            candidate.published
                && candidate.status == Status::Complete
                && manifest["run_id"]
                    == crate::readiness_metadata::value(&baseline.run_id).expect("validated native metadata")
                && manifest["identities"]["source"]["sha256"]
                    == crate::readiness_metadata::value(&baseline.source_sha256).expect("validated native metadata")
                && coverage["generated_instructions"].as_integer().and_then(|n| u64::try_from(n).ok())
                    == Some(baseline.generated)
                && coverage["interpreted_instructions"].as_integer().and_then(|n| u64::try_from(n).ok())
                    == Some(baseline.interpreted)
                && coverage["validated_distinct_keys"].as_integer().and_then(|n| u64::try_from(n).ok())
                    == Some(baseline.validated_keys)
                && manifest["increment"]["session_id"]
                    == crate::readiness_metadata::value(&state.session_id).expect("validated native metadata")
                && manifest["increment"]["increment_id"]
                    == crate::readiness_metadata::value(&toml::Value::try_from(&state.increment_id)?)
                        .expect("validated native metadata")
                && manifest["accepted_parent"]
                    == crate::readiness_metadata::value(
                        &state.baseline.as_ref().context("candidate predecessor missing")?.run_id
                    )
                    .expect("validated native metadata")
                && origins::document(&owner.join("acceptance-result.toml"))?["accepted"]
                    == crate::readiness_metadata::value(&true).expect("validated native metadata"),
            "published session candidate metrics or lineage differs"
        );
        Ok(())
    }
    fn accepted(&self) -> Result<Baseline> {
        let pointer = origins::document(&self.root.join("out/accepted.toml"))?;
        let owner = Path::new(family::field(&pointer, "run_path")?);
        let verified = reuse::verify_accepted(owner, true)?;
        readiness::verify_origins(owner, &verified)?;
        ensure!(
            reuse::pointer(owner, &verified["manifest"])? == pointer,
            "current pointer differs from accepted origin"
        );
        let coverage = &verified["verdict"]["checks"]["changed-entry-coverage"];
        let inputs = &verified["manifest"]["identities"]["inputs"];
        for (file, key) in [("rom/sm.sfc", "rom_sha256"), ("movies/super-metroid-any-3653.lsmv", "movie_sha256")] {
            ensure!(digest(&self.root.join(file))? == family::field(inputs, key)?, "accepted material differs");
        }
        Ok(Baseline {
            run_id: family::field(&pointer, "run_id")?.into(),
            run_path: owner.into(),
            source_sha256: family::field(&verified["manifest"]["identities"]["source"], "sha256")?.into(),
            generated: coverage["generated_instructions"]
                .as_integer()
                .and_then(|n| u64::try_from(n).ok())
                .context("accepted generated count missing")?,
            interpreted: coverage["interpreted_instructions"]
                .as_integer()
                .and_then(|n| u64::try_from(n).ok())
                .context("accepted interpreted count missing")?,
            validated_keys: coverage["validated_distinct_keys"]
                .as_integer()
                .and_then(|n| u64::try_from(n).ok())
                .context("accepted validated count missing")?,
        })
    }
    fn readiness(&self, previous: Option<&Path>, expected: &str) -> Result<Value> {
        let mut request = crate :: report ! ({ "checkout" : self . root , "timeout" : self . timeout , "original_run" : self . original , "predecessor" : previous , });
        {
            let value: toml::Value = expected.into();
            request.as_table_mut().context("native metadata table missing")?.insert(("expected_pointer").into(), value);
        };
        readiness::operate(request)
    }
}
impl Operations for Production {
    fn allocate(&mut self, state: &State) -> Result<Option<PathBuf>> {
        let baseline = state.baseline.as_ref().context("validation baseline missing")?;
        let mut run = Run::create(&self.root, "iteration", contract::domain(), Some(&baseline.run_id))?;
        let mut increment: increment::Increment =
            pipeline_model::read(&state.preparation.as_ref().context("preparation missing")?.join("increment.toml"))?;
        increment.session_id = Some(state.session_id.clone());
        increment.increment_id = state.increment_id.clone();
        run.manifest.increment =
            Some(toml::Value::try_from(increment)?.as_table().context("increment table missing")?.clone());
        run.save()?;
        Ok(Some(run.path))
    }
    fn recover(&mut self, state: &State) -> Result<Candidate> {
        let owner = state.pending_run.as_ref().context("pending run missing")?;
        if Run::allocated(owner, &self.root).is_ok() {
            return self.validate(state, false);
        }
        if owner.join("verdict.toml").exists() {
            let verified = reuse::verify_accepted(owner, true);
            if let Ok(verified) = verified {
                let coverage = &verified["verdict"]["checks"]["changed-entry-coverage"];
                let baseline = Baseline {
                    run_id: family::field(&verified["manifest"], "run_id")?.into(),
                    run_path: owner.clone(),
                    source_sha256: family::field(&verified["manifest"]["identities"]["source"], "sha256")?.into(),
                    generated: coverage["generated_instructions"]
                        .as_integer()
                        .and_then(|n| u64::try_from(n).ok())
                        .context("generated count missing")?,
                    interpreted: coverage["interpreted_instructions"]
                        .as_integer()
                        .and_then(|n| u64::try_from(n).ok())
                        .context("interpreted count missing")?,
                    validated_keys: coverage["validated_distinct_keys"]
                        .as_integer()
                        .and_then(|n| u64::try_from(n).ok())
                        .context("validated count missing")?,
                };
                let candidate = Candidate {
                    published: true,
                    status: Status::Complete,
                    reason: "recovered independently published validation intent".into(),
                    run_path: Some(owner.clone()),
                    baseline: Some(baseline),
                };
                self.authenticate_candidate(state, &candidate)?;
                return Ok(candidate);
            }
        }
        Ok(Candidate {
            published: false,
            status: Status::Stopped,
            reason: "interrupted validation retained; repair or rerun this exact increment".into(),
            run_path: Some(owner.clone()),
            baseline: None,
        })
    }
    fn authenticate(&mut self, state: &State, candidate: &Candidate) -> Result<()> {
        self.authenticate_candidate(state, candidate)
    }
    fn baseline(&mut self) -> Result<Baseline> {
        let expected_pointer = crate::readiness_reuse::pointer_token(&self.root)?;
        let previous = match self.accepted() {
            Ok(b) => Some(b),
            Err(error) => {
                let pointer = self.root.join("out/accepted.toml");
                if pointer.exists() {
                    let mut run = Run::create(&self.root, "accepted-pointer-recovery", contract::domain(), None)?;
                    run.capture_source()?;
                    run.record_artifact("rejected-pointer", &pointer, Some("diagnostics/rejected-pointer.bin"))?;
                    run . record_check ("rejected-pointer" , crate :: report ! ({ "status" : "pass" , "complete" : true , "reason" : format ! ("{error:#}") , }) ,) ? ;
                    run.finalize(
                        "pass",
                        None,
                        Some("invalid pointer preserved; complete readiness required"),
                        &["rejected-pointer".into()],
                    )?;
                    ensure!(
                        crate::readiness_reuse::pointer_token(&self.root)? == expected_pointer,
                        "accepted pointer changed during rejection capture"
                    );
                }
                None
            }
        };
        let changed = if let Some(b) = &previous {
            crate::readiness_metadata::value(&source_inventory(&self.root)?).expect("validated native metadata")
                != origins::document(&b.run_path.join("source/inventory.toml"))?["files"]
        } else {
            true
        };
        if changed {
            ensure!(
                crate::readiness_reuse::pointer_token(&self.root)? == expected_pointer,
                "accepted pointer changed before readiness refresh"
            );
            let verdict = self.readiness(previous.as_ref().map(|b| b.run_path.as_path()), &expected_pointer)?;
            ensure!(
                verdict["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
                "required readiness refresh failed; inspect retained run {}",
                verdict["run_id"]
            );
        }
        self.accepted()
    }
    fn select(&mut self, baseline: &Baseline) -> Result<Selection> {
        let verdict = iteration_selection::operate(
            crate :: report ! ({ "checkout" : self . root , "timeout" : self . timeout , "predecessor" : baseline . run_path , }),
        )?;
        ensure!(
            verdict["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
            "measured selection failed"
        );
        let owner = self.root.join("out/runs").join(family::field(&verdict, "run_id")?);
        let selected: pipeline_model::BoundSelection = pipeline_model::read(&owner.join("selection.toml"))?;
        Ok(Selection { run_path: owner, outcome: selected.ranking.outcome })
    }
    fn prepare(&mut self, selection: &Selection) -> Result<PathBuf> {
        increment::prepare(&self.root, &selection.run_path, self.timeout)
    }
    fn validate(&mut self, state: &State, retry: bool) -> Result<Candidate> {
        let baseline = state.baseline.as_ref().context("session baseline missing")?;
        ensure!(self.accepted()?.run_id == baseline.run_id, "competing acceptance changed unresolved predecessor");
        let mut increment = if retry {
            let candidate =
                state.candidate.as_ref().and_then(|c| c.run_path.as_ref()).context("retry candidate missing")?;
            if !candidate.join("source/inventory.toml").exists() {
                increment::recover_application(
                    &self.root,
                    state.preparation.as_ref().context("session preparation missing")?,
                )?
            } else {
                let terminal = if candidate.join("verdict.toml").exists() {
                    Some(origins::document(&candidate.join("verdict.toml"))?)
                } else {
                    None
                };
                let (manifest, inventory) = origins::snapshot(candidate)?;
                origins::owned(candidate, &manifest)?;
                if crate::readiness_metadata::value(&source_inventory(&self.root)?).expect("validated native metadata")
                    == inventory["files"]
                    && terminal.as_ref().is_some_and(|v| {
                        v["status"]
                            != crate::readiness_metadata::value(&"incomplete").expect("validated native metadata")
                    })
                {
                    return Ok(Candidate {
                        published: false,
                        status: Status::RepairNeeded,
                        reason: "apply a bounded repair or new discriminating experiment before resume".into(),
                        run_path: Some(candidate.clone()),
                        baseline: None,
                    });
                }
                toml_value_decode(manifest["increment"].clone())?
            }
        } else {
            increment::recover_application(
                &self.root,
                state.preparation.as_ref().context("session preparation missing")?,
            )?
        };
        increment.session_id = Some(state.session_id.clone());
        increment.increment_id = state.increment_id.clone();
        let run = if let Some(owner) = &state.pending_run {
            Run::allocated(owner, &self.root)?
        } else {
            Run::create(&self.root, "iteration", contract::domain(), Some(&baseline.run_id))?
        };
        let verdict = readiness::operate_run(
            crate :: report ! ({ "checkout" : self . root , "timeout" : self . timeout , "original_run" : self . original , "predecessor" : baseline . run_path , "increment" : increment , }),
            run,
        )?;
        let owner = self.root.join("out/runs").join(family::field(&verdict, "run_id")?);
        if verdict["status"] != crate::readiness_metadata::value(&"pass").expect("validated native metadata") {
            return Ok(Candidate {
                published: false,
                status: if verdict["status"]
                    == crate::readiness_metadata::value(&"incomplete").expect("validated native metadata")
                {
                    Status::Stopped
                } else {
                    Status::RepairNeeded
                },
                reason: format!(
                    "candidate {} failed; retained verdict and source identify the repair",
                    verdict["run_id"]
                ),
                run_path: Some(owner),
                baseline: None,
            });
        }
        let accepted = self.accepted()?;
        let (manifest, _) = origins::snapshot(&owner)?;
        ensure!(
            crate::readiness_metadata::value(&accepted.run_id).expect("validated native metadata") == verdict["run_id"]
                && manifest["increment"]["session_id"]
                    == crate::readiness_metadata::value(&state.session_id).expect("validated native metadata")
                && manifest["increment"]["increment_id"]
                    == crate::readiness_metadata::value(&toml::Value::try_from(&state.increment_id)?)
                        .expect("validated native metadata")
                && origins::document(&owner.join("acceptance-result.toml"))?["accepted"]
                    == crate::readiness_metadata::value(&true).expect("validated native metadata"),
            "published result is not this session candidate"
        );
        Ok(Candidate {
            published: true,
            status: Status::Complete,
            reason: "independently accepted session candidate".into(),
            run_path: Some(owner),
            baseline: Some(accepted),
        })
    }
}
pub fn operate(request: Value) -> Result<State> {
    let root = fs::canonicalize(
        request.get("checkout").and_then(Value::as_str).map(PathBuf::from).unwrap_or(std::env::current_dir()?),
    )?;
    let count = request.get("count").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok());
    let target = request.get("target_generated").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok());
    let timeout =
        request.get("timeout").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).unwrap_or(2400);
    ensure!(timeout > 0, "timeout must be positive");
    let mut session = if let Some(id) = request.get("resume").and_then(Value::as_str) {
        ensure!(count.is_none() && target.is_none(), "resume preserves original request");
        Session::resume(&root, id)?
    } else {
        Session::create(&root, if count.is_none() && target.is_none() { Some(1) } else { count }, target)?
    };
    eprintln!("Session: {}\nBrief: {}", session.path.display(), session.path.join("brief.md").display());
    session.run(&mut Production {
        root,
        timeout,
        original: request.get("original_run").and_then(Value::as_str).map(PathBuf::from),
    })
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyReport {
    pub schema_version: u32,
    pub scope: String,
    pub production_accepted_count: u64,
    pub cases: std::collections::BTreeMap<String, State>,
}
#[derive(Default)]
struct Simulation {
    count: u64,
    fail: bool,
}
impl Simulation {
    fn value(&self) -> Baseline {
        Baseline {
            run_id: format!("symbolic-{}", self.count),
            run_path: format!("symbolic-{}", self.count).into(),
            source_sha256: "symbolic".into(),
            generated: 10 + self.count * 10,
            interpreted: 90 - self.count * 10,
            validated_keys: 1 + self.count,
        }
    }
}
impl Operations for Simulation {
    fn baseline(&mut self) -> Result<Baseline> {
        Ok(self.value())
    }
    fn select(&mut self, _: &Baseline) -> Result<Selection> {
        Ok(Selection {
            run_path: "symbolic-selection".into(),
            outcome: pipeline_model::SelectionOutcome::SupportedRoutine,
        })
    }
    fn prepare(&mut self, _: &Selection) -> Result<PathBuf> {
        Ok("symbolic-preparation".into())
    }
    fn authenticate(&mut self, _: &State, candidate: &Candidate) -> Result<()> {
        let baseline = candidate.baseline.as_ref().context("symbolic baseline missing")?;
        let count = baseline.run_id.strip_prefix("symbolic-").context("invalid symbolic ID")?.parse::<u64>()?;
        let expected = Simulation { count, fail: false }.value();
        ensure!(baseline == &expected && count <= self.count, "symbolic acceptance differs");
        Ok(())
    }
    fn validate(&mut self, _: &State, _: bool) -> Result<Candidate> {
        if self.fail {
            return Ok(Candidate {
                published: false,
                status: Status::RepairNeeded,
                reason: "simulated publication refusal".into(),
                run_path: Some("symbolic-rejected".into()),
                baseline: None,
            });
        }
        self.count += 1;
        Ok(Candidate {
            published: true,
            status: Status::Complete,
            reason: "symbolic policy result; no production acceptance".into(),
            run_path: Some(format!("symbolic-{}", self.count).into()),
            baseline: Some(self.value()),
        })
    }
}
pub(crate) fn policy(output: &Path) -> Result<PolicyReport> {
    fs::create_dir(output)?;
    let mut cases = std::collections::BTreeMap::new();
    for (name, count, target, fail) in [
        ("count1", Some(1), None, false),
        ("count2", Some(2), None, false),
        ("target", None, Some(20), false),
        ("already-satisfied", None, Some(10), false),
        ("failed-publication", Some(1), None, true),
        ("user-stop", Some(1), None, false),
    ] {
        let root = output.join(name);
        fs::create_dir(&root)?;
        let mut session = Session::create(&root, count, target)?;
        let state = if name == "user-stop" {
            session.finish(Status::Stopped, "simulated user stop".into())?
        } else {
            session.run(&mut Simulation { count: 0, fail })?
        };
        cases.insert(name.into(), state);
    }
    let report = PolicyReport {
        schema_version: 1,
        scope: "session policy simulation; acceptance not certified".into(),
        production_accepted_count: 0,
        cases,
    };
    pipeline_model::write(&output.join("report.toml"), &report)?;
    verify_policy(output, &report)?;
    Ok(report)
}
pub(crate) fn verify_policy(output: &Path, report: &PolicyReport) -> Result<()> {
    ensure!(
        report.schema_version == 1
            && report.production_accepted_count == 0
            && report.scope == "session policy simulation; acceptance not certified"
            && report.cases.len() == 6,
        "policy simulation scope differs"
    );
    for (name, count, status) in [
        ("count1", 1, Status::Complete),
        ("count2", 2, Status::Complete),
        ("target", 1, Status::Complete),
        ("already-satisfied", 0, Status::Complete),
        ("failed-publication", 0, Status::RepairNeeded),
        ("user-stop", 0, Status::Stopped),
    ] {
        let state = report.cases.get(name).context("policy case missing")?;
        let root = output.join(name);
        let mut session = Session::resume(&root, &state.session_id)?;
        session.recover_events()?;
        ensure!(
            &session.state == state
                && state.accepted_count == count
                && state.status == status
                && !root.join("out/accepted.toml").exists(),
            "policy simulation outcome differs"
        );
        if name == "failed-publication" {
            ensure!(
                state.increment_id.is_some() && state.candidate.as_ref().is_some_and(|c| !c.published),
                "failed proposal was discarded"
            );
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Temp(PathBuf);
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn root() -> Temp {
        let path = std::env::temp_dir()
            .join(format!("sm-native-session-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&path).unwrap();
        Temp(path)
    }
    #[derive(Default)]
    struct Fixture {
        counter: u64,
        selects: u64,
        validates: u64,
        retries: u64,
        fail: bool,
    }
    impl Fixture {
        fn baseline_value(&self) -> Baseline {
            Baseline {
                run_id: format!("symbolic-{}", self.counter),
                run_path: format!("symbolic-{}", self.counter).into(),
                source_sha256: "symbolic".into(),
                generated: 10 + self.counter * 10,
                interpreted: 90 - self.counter * 10,
                validated_keys: 1 + self.counter,
            }
        }
    }
    impl Operations for Fixture {
        fn authenticate(&mut self, _: &State, candidate: &Candidate) -> Result<()> {
            let baseline = candidate.baseline.as_ref().context("symbolic baseline missing")?;
            let ordinal = baseline.run_id.strip_prefix("symbolic-").context("symbolic ID missing")?.parse::<u64>()?;
            ensure!(
                ordinal <= self.counter
                    && baseline.generated == 10 + ordinal * 10
                    && baseline.interpreted == 90 - ordinal * 10
                    && baseline.validated_keys == 1 + ordinal,
                "symbolic acceptance differs"
            );
            Ok(())
        }
        fn baseline(&mut self) -> Result<Baseline> {
            Ok(self.baseline_value())
        }
        fn select(&mut self, _: &Baseline) -> Result<Selection> {
            self.selects += 1;
            Ok(Selection {
                run_path: "symbolic-selection".into(),
                outcome: pipeline_model::SelectionOutcome::SupportedRoutine,
            })
        }
        fn prepare(&mut self, _: &Selection) -> Result<PathBuf> {
            Ok("symbolic-preparation".into())
        }
        fn validate(&mut self, state: &State, retry: bool) -> Result<Candidate> {
            assert!(state.increment_id.is_some());
            if retry {
                self.retries += 1;
            } else {
                self.validates += 1;
            }
            if self.fail {
                self.fail = false;
                return Ok(Candidate {
                    published: false,
                    status: Status::RepairNeeded,
                    reason: "symbolic failed publication".into(),
                    run_path: Some("symbolic-failed-candidate".into()),
                    baseline: None,
                });
            }
            self.counter += 1;
            Ok(Candidate {
                published: true,
                status: Status::Complete,
                reason: "symbolic policy result; no production acceptance".into(),
                run_path: Some(format!("symbolic-{}", self.counter).into()),
                baseline: Some(self.baseline_value()),
            })
        }
    }
    #[test]
    fn bounded_count_target_and_failed_publication_preserve_exact_unresolved_increment() {
        for (count, target, expected) in
            [(Some(1), None, 1), (Some(2), None, 2), (None, Some(10), 0), (None, Some(15), 1)]
        {
            let root = root();
            let mut session = Session::create(&root.0, count, target).unwrap();
            let mut operations = Fixture::default();
            let state = session.run(&mut operations).unwrap();
            assert_eq!(state.status, Status::Complete);
            assert_eq!(state.accepted_count, expected);
            assert_eq!(operations.validates, expected);
            assert_eq!(operations.selects, expected);
            let state = Session::resume(&root.0, &state.session_id).unwrap().run(&mut operations).unwrap();
            assert_eq!(state.accepted_count, expected);
            assert_eq!(operations.validates, expected);
        }
        let root = root();
        let mut session = Session::create(&root.0, Some(1), None).unwrap();
        let mut operations = Fixture { fail: true, ..Default::default() };
        let failed = session.run(&mut operations).unwrap();
        assert_eq!(failed.accepted_count, 0);
        assert_eq!(failed.status, Status::RepairNeeded);
        let increment = failed.increment_id.clone();
        let resumed = Session::resume(&root.0, &failed.session_id).unwrap().run(&mut operations).unwrap();
        assert_eq!(resumed.status, Status::Complete);
        assert_eq!(resumed.accepted_count, 1);
        assert_eq!(operations.selects, 1);
        assert_eq!(operations.retries, 1);
        let retry = resumed.events.iter().find(|e| e.operation == "retry").unwrap();
        let retry: EventRecord = pipeline_model::read(&session.path.join(&retry.evidence)).unwrap();
        assert!(retry.result.get("published").unwrap().as_bool().unwrap());
        assert!(increment.is_some());
    }
    #[test]
    fn invalid_requests_and_corrupt_ordered_events_reject_before_candidate_work() {
        let root = root();
        for (count, target) in [(Some(0), None), (Some(1), Some(100)), (None, Some(0)), (None, None)] {
            assert!(Session::create(&root.0, count, target).is_err());
        }
        assert!(!root.0.join("out").exists());
        let mut session = Session::create(&root.0, Some(1), None).unwrap();
        let mut operations = Fixture { fail: true, ..Default::default() };
        let state = session.run(&mut operations).unwrap();
        let event = session.path.join(&state.events[0].evidence);
        fs::write(event, "forged").unwrap();
        assert!(Session::resume(&root.0, &state.session_id).is_err());
        assert_eq!(operations.validates, 1);
        assert!(Session::resume(&root.0, "../escape").is_err());
    }
    #[test]
    fn replay_rejects_forged_count_and_recovers_committed_event_before_state_save() {
        let root = root();
        let mut session = Session::create(&root.0, Some(2), None).unwrap();
        let before = fs::read(session.path.join("state.toml")).unwrap();
        let baseline = Fixture::default().baseline_value();
        session.record("baseline", &baseline).unwrap();
        fs::write(session.path.join("state.toml"), before).unwrap();
        let recovered = Session::resume(&root.0, &session.state.session_id).unwrap();
        assert_eq!(recovered.state.baseline, Some(baseline));
        assert_eq!(recovered.state.events.len(), 2);
        let mut forged = recovered.state.clone();
        forged.accepted_count = 2;
        write(&recovered.path.join("state.toml"), &forged, true).unwrap();
        assert!(Session::resume(&root.0, &forged.session_id).is_err());
    }
    #[test]
    fn published_event_recovery_counts_once_and_reauthenticates_before_counting() {
        let root = root();
        let mut session = Session::create(&root.0, Some(1), None).unwrap();
        let mut operations = Fixture::default();
        session.record("baseline", &operations.baseline_value()).unwrap();
        session.record("increment", &"increment-fixture").unwrap();
        session.record("select", &operations.select(&session.state.baseline.clone().unwrap()).unwrap()).unwrap();
        session.record("prepare", &PathBuf::from("symbolic-preparation")).unwrap();
        let candidate = operations.validate(&session.state, false).unwrap();
        session.record("validate", &candidate).unwrap();
        let state = Session::resume(&root.0, &session.state.session_id).unwrap().run(&mut operations).unwrap();
        assert_eq!(state.accepted_count, 1);
        assert_eq!(operations.validates, 1);
        assert_eq!(
            Session::resume(&root.0, &state.session_id).unwrap().run(&mut operations).unwrap().accepted_count,
            1
        );
        operations.counter = 0;
        assert!(Session::resume(&root.0, &state.session_id).unwrap().run(&mut operations).is_err());
    }
}
