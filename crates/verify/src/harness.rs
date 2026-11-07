//! The domain harness (MODEL §4, "What proves it"): one domain's model checked against the
//! oracle directly, frame by frame, over every recorded frame of a scenario.
//!
//! For each pair of consecutive records (N, N+1) the harness decodes the domain's State and
//! inputs from record N, steps the domain's model one frame, and compares the domain's
//! declared fields with the ones record N+1 holds. The [`Report`] says how many of the M
//! frames matched on every field, and per field how many matched and the first mismatches with
//! both values. A domain plugs in by implementing [`Domain`]; [`run`] is the one runner and
//! [`run_cached`] runs it over a lock row's record cache.
//!
//! The harness never consults today's engine: the oracle is the yardstick (MODEL §4).

pub mod game_time;
pub mod samus_motion;

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use oracle_record::{FrameRecord, Pad};

use crate::cache::{Cache, CacheError, CutKind};
use crate::lock::{Frame, Lock, RowId};

/// The mismatches a field's report keeps, the earliest ones.
pub const FIRST_MISMATCHES: usize = 5;

/// What the world gives the frame from N to N+1, from record N+1.
///
/// That is the pad the frame reads and how its interval ended. A domain sees nothing else of
/// record N+1 before it steps, so its model can't read the answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Interval {
    /// The pad auto-joypad read at the frame's vblank.
    pub pad: Pad,
    /// How the frame's interval ended.
    pub cut: CutKind,
}

impl Interval {
    /// The interval that ends at `next`.
    #[must_use]
    pub const fn ending_at(next: &FrameRecord) -> Self {
        Self {
            pad: next.pad,
            cut: CutKind::of(next.cut),
        }
    }
}

/// One declared field of a domain's [`Domain::Fields`]: its name, whether two projections
/// agree on it, and how a mismatch shows its value. [`crate::domain_field!`] builds one from a
/// field path.
pub struct Field<F> {
    /// The field's name in the spec.
    pub name: &'static str,
    /// Whether the two projections agree on the field.
    pub same: fn(&F, &F) -> bool,
    /// The field's value, as a mismatch reports it.
    pub show: fn(&F) -> String,
}

/// A [`Field`] for the field at a path of a domain's projection type:
/// `domain_field!("speed.x", speed.x)`.
#[macro_export]
macro_rules! domain_field {
    ($name:literal, $($path:tt)+) => {
        $crate::harness::Field {
            name: $name,
            same: |model, oracle| model.$($path)+ == oracle.$($path)+,
            show: |fields| format!("{:?}", fields.$($path)+),
        }
    };
}

/// A domain of the engine as the harness checks it.
///
/// `decode`, `world` and `expected` are the observer's (MODEL §5, step 4); `step` is the model
/// under test; `fields` projects the model's State onto the declared fields, the spec's
/// quantities whose "Read by" isn't "none".
pub trait Domain {
    /// The domain's name in the report.
    const NAME: &'static str;
    /// The domain's State.
    type State;
    /// What the domain reads each frame from outside its State.
    type Inputs;
    /// The harness's stand-in for the neighbouring domains the model asks during its frame
    /// (collision, pose selection), which aren't under test. Unlike the inputs it may answer
    /// from record N+1.
    type World;
    /// The declared fields, as both the model and the oracle give them.
    type Fields: 'static;
    /// Why a record isn't decoded for this domain: a frame the domain doesn't cover.
    type Skip: fmt::Display;
    /// The declared fields, in the spec's order.
    const FIELDS: &'static [Field<Self::Fields>];

    /// The State and inputs of the frame from record `from` to the end of `interval`.
    ///
    /// # Errors
    ///
    /// The reason the frame isn't one the domain covers.
    fn decode(
        &self,
        from: &FrameRecord,
        interval: Interval,
    ) -> Result<(Self::State, Self::Inputs), Self::Skip>;

    /// The neighbours' answers for the frame from record `from` to record `next`.
    ///
    /// # Errors
    ///
    /// The reason the frame isn't one the domain covers.
    fn world(&self, from: &FrameRecord, next: &FrameRecord) -> Result<Self::World, Self::Skip>;

    /// The model: one frame of `state` under `inputs`, asking `world`.
    fn step(&self, state: &mut Self::State, inputs: &Self::Inputs, world: &mut Self::World);

    /// The declared fields of the model's `state`.
    fn fields(&self, state: &Self::State) -> Self::Fields;

    /// The declared fields the oracle's record `next` holds.
    ///
    /// # Errors
    ///
    /// The reason the record isn't one the domain covers.
    fn expected(&self, next: &FrameRecord) -> Result<Self::Fields, Self::Skip>;
}

/// A field's value at a frame where the model and the oracle disagree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mismatch {
    /// Frame N+1, the record compared with.
    pub frame: Frame,
    /// The model's value.
    pub model: String,
    /// The oracle's value.
    pub oracle: String,
}

/// One declared field over a run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldReport {
    /// The field's name.
    pub name: &'static str,
    /// The compared frames on which the field matched.
    pub matched: u64,
    /// The compared frames on which it didn't.
    pub mismatched: u64,
    /// The first [`FIRST_MISMATCHES`] mismatches, in frame order.
    pub first: Vec<Mismatch>,
    /// The first [`FIRST_MISMATCHES`] runs of consecutive mismatched frames.
    pub runs: Vec<Run>,
}

/// Consecutive frames on which a field mismatched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    /// The run's first frame.
    pub first: Frame,
    /// Its last frame.
    pub last: Frame,
}

/// A domain's run over one scenario's records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// The domain.
    pub domain: &'static str,
    /// The frames in the run, M: every pair of consecutive records.
    pub frames: u64,
    /// The frames compared: M less the skipped ones.
    pub compared: u64,
    /// The compared frames on which every declared field matched, N.
    pub matched: u64,
    /// The skipped frames, by reason.
    pub skipped: BTreeMap<String, u64>,
    /// Each declared field, in the domain's order.
    pub fields: Vec<FieldReport>,
}

impl Report {
    fn new<D: Domain>() -> Self {
        Self {
            domain: D::NAME,
            frames: 0,
            compared: 0,
            matched: 0,
            skipped: BTreeMap::new(),
            fields: D::FIELDS
                .iter()
                .map(|field| FieldReport {
                    name: field.name,
                    matched: 0,
                    mismatched: 0,
                    first: Vec::new(),
                    runs: Vec::new(),
                })
                .collect(),
        }
    }

    fn skip(&mut self, reason: impl fmt::Display) {
        let count = self.skipped.entry(reason.to_string()).or_default();
        *count = count.saturating_add(1);
    }

    fn compare<D: Domain>(&mut self, frame: Frame, model: &D::Fields, oracle: &D::Fields) {
        self.compared = self.compared.saturating_add(1);
        let mut all = true;
        for (field, report) in D::FIELDS.iter().zip(&mut self.fields) {
            if (field.same)(model, oracle) {
                report.matched = report.matched.saturating_add(1);
                continue;
            }
            all = false;
            report.mismatched = report.mismatched.saturating_add(1);
            if report.first.len() < FIRST_MISMATCHES {
                report.first.push(Mismatch {
                    frame,
                    model: (field.show)(model),
                    oracle: (field.show)(oracle),
                });
            }
            let room = report.runs.len() < FIRST_MISMATCHES;
            match report.runs.last_mut() {
                Some(run) if run.last.0.checked_add(1) == Some(frame.0) => run.last = frame,
                Some(_) | None if room => report.runs.push(Run {
                    first: frame,
                    last: frame,
                }),
                Some(_) | None => {}
            }
        }
        if all {
            self.matched = self.matched.saturating_add(1);
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}: {} of {} frames matched ({} compared)",
            self.domain, self.matched, self.frames, self.compared
        )?;
        for (reason, count) in &self.skipped {
            writeln!(f, "  skipped {count}: {reason}")?;
        }
        for field in &self.fields {
            writeln!(
                f,
                "  {} {} of {}",
                field.name,
                field.matched,
                field.matched.saturating_add(field.mismatched)
            )?;
            for run in &field.runs {
                writeln!(f, "    frames {}..={}", run.first.0, run.last.0)?;
            }
            for mismatch in &field.first {
                writeln!(
                    f,
                    "    frame {}: model {} oracle {}",
                    mismatch.frame.0, mismatch.model, mismatch.oracle
                )?;
            }
        }
        Ok(())
    }
}

/// `domain` over `records`, a scenario's records in frame order.
///
/// # Errors
///
/// The first record that couldn't be read.
pub fn run<D: Domain, E>(
    domain: &D,
    records: impl IntoIterator<Item = Result<FrameRecord, E>>,
) -> Result<Report, E> {
    let mut report = Report::new::<D>();
    let mut records = records.into_iter();
    let Some(first) = records.next() else {
        return Ok(report);
    };
    let mut from = first?;
    for record in records {
        let next = record?;
        report.frames = report.frames.saturating_add(1);
        let compared = domain
            .decode(&from, Interval::ending_at(&next))
            .and_then(|decoded| Ok((decoded, domain.world(&from, &next)?)))
            .and_then(|decoded| Ok((decoded, domain.expected(&next)?)));
        match compared {
            Ok((((mut state, inputs), mut world), oracle)) => {
                domain.step(&mut state, &inputs, &mut world);
                let model = domain.fields(&state);
                report.compare::<D>(Frame(next.index), &model, &oracle);
            }
            Err(reason) => report.skip(reason),
        }
        from = next;
    }
    Ok(report)
}

/// Why a cached run couldn't start or finish.
#[derive(Debug)]
pub enum HarnessError {
    /// The lock file couldn't be read or parsed.
    Lock(String),
    /// The lock has no such row.
    NoRow(String),
    /// The row's cache couldn't be opened or a record read.
    Cache(CacheError),
}

impl fmt::Display for HarnessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lock(error) => write!(f, "{error}"),
            Self::NoRow(row) => write!(f, "{row}: not in the lock"),
            Self::Cache(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for HarnessError {}

impl From<CacheError> for HarnessError {
    fn from(error: CacheError) -> Self {
        Self::Cache(error)
    }
}

/// `domain` over row `row`'s record cache under `cache`, checked against the lock at `lock`.
///
/// # Errors
///
/// A [`HarnessError`] when the lock, the row or its cache can't be read.
pub fn run_cached<D: Domain>(
    domain: &D,
    lock: &Path,
    cache: &Path,
    row: &str,
) -> Result<Report, HarnessError> {
    let text = std::fs::read_to_string(lock)
        .map_err(|error| HarnessError::Lock(format!("{}: {error}", lock.display())))?;
    let parsed = Lock::parse(&text)
        .map_err(|error| HarnessError::Lock(format!("{}: {error:?}", lock.display())))?;
    let row = parsed
        .row(&RowId(row.to_owned()))
        .ok_or_else(|| HarnessError::NoRow(row.to_owned()))?;
    let cache = Cache::open(cache, row)?;
    let records = cache
        .entries()
        .iter()
        .map(|entry| cache.record(entry).map_err(HarnessError::from));
    run(domain, records)
}

#[cfg(test)]
mod tests;
