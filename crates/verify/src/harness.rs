//! The domain harness (MODEL §4, "What proves it"): one domain's model checked against the
//! oracle directly, frame by frame, over every recorded frame of a scenario.
//!
//! For each pair of consecutive records (N, N+1) the harness decodes the domain's State and
//! inputs from record N, steps the domain's model one frame, and compares the domain's
//! declared fields with the ones record N+1 holds. A domain plugs in by implementing
//! [`Domain`]; [`run`] is the one runner and [`run_cached`] runs it over a lock row's record
//! cache.
//!
//! The measure is honest (MODEL §4, "The measure must be honest"). The [`Report`] counts every
//! recorded frame pair, M, and splits it into frames the domain doesn't run in (skipped, by
//! reason), frames the decode failed on (harness defects) and compared frames. On a compared
//! frame a field out of the domain's scope is masked, by reason, and the rest are compared. A
//! field whose value depends on what the domain's stand-in for its neighbours copied from
//! record N+1 is unverified, not matched: the harness steps the frame a second time against
//! [`Domain::neutral`]'s stand-in, which copies nothing, and a field whose value differs
//! depended on the copy. Beside the model the report gives the null model's score on the same
//! frames, a step that does nothing, and per behaviour the frames that exercised it.
//!
//! The harness never consults today's engine: the oracle is the yardstick (MODEL §4).

pub mod camera;
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

/// A set of a domain's declared fields, by their position in [`Domain::FIELDS`] (the first 64).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FieldSet(u64);

impl FieldSet {
    /// No field.
    pub const EMPTY: Self = Self(0);
    /// Every field.
    pub const ALL: Self = Self(u64::MAX);

    /// The field at position `index` alone.
    #[must_use]
    pub const fn single(index: usize) -> Self {
        if index < 64 {
            Self(1_u64 << index)
        } else {
            Self::EMPTY
        }
    }

    /// The fields in `self` or `other`.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Whether the field at `index` is in the set.
    #[must_use]
    pub const fn contains(self, index: usize) -> bool {
        self.0 & Self::single(index).0 != 0
    }
}

/// Declared fields out of a domain's scope on one frame, and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mask {
    /// Why the fields aren't the domain's on this frame.
    pub reason: &'static str,
    /// The fields.
    pub fields: FieldSet,
}

/// Why a frame pair isn't compared: the domain doesn't run in it, or its decode failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Uncompared {
    /// The domain doesn't run in the frame, for this named reason.
    Skip(String),
    /// The harness couldn't decode the frame: a defect of the harness, never a scope call.
    Defect(String),
}

/// A domain of the engine as the harness checks it.
///
/// `decode`, `world`, `masks` and `expected` are the observer's (MODEL §5, step 4); `step` is
/// the model under test; `fields` projects the model's State onto the declared fields, the
/// spec's quantities whose "Read by" isn't "none".
pub trait Domain {
    /// The domain's name in the report.
    const NAME: &'static str;
    /// The domain's State.
    type State: Clone;
    /// What the domain reads each frame from outside its State.
    type Inputs;
    /// The harness's stand-in for the neighbouring domains the model asks during its frame
    /// (collision, pose selection), which aren't under test. Unlike the inputs it may answer
    /// from record N+1; every field such an answer feeds is unverified on the frame.
    type World;
    /// The declared fields, as both the model and the oracle give them.
    type Fields: 'static;
    /// Why a record isn't compared for this domain: a frame the domain doesn't run in, or one
    /// the decode failed on ([`Domain::uncompared`] tells them apart).
    type Skip: fmt::Display;
    /// The declared fields, in the spec's order.
    const FIELDS: &'static [Field<Self::Fields>];

    /// The names of the domain's catalogue entries, in the catalogue's order, which the report
    /// attributes frames to.
    fn behaviours(&self) -> Vec<String> {
        Vec::new()
    }

    /// The State and inputs of the frame from record `from` to the end of `interval`.
    ///
    /// # Errors
    ///
    /// The reason the frame isn't one the domain runs in, or the decode's failure.
    fn decode(
        &self,
        from: &FrameRecord,
        interval: Interval,
    ) -> Result<(Self::State, Self::Inputs), Self::Skip>;

    /// The neighbours' answers for the frame from record `from` to record `next`.
    ///
    /// # Errors
    ///
    /// The reason the frame isn't one the domain runs in, or the decode's failure.
    fn world(&self, from: &FrameRecord, next: &FrameRecord) -> Result<Self::World, Self::Skip>;

    /// The fields out of the domain's scope on the frame from `from` to `next`. A mask may read
    /// record N and the neighbours' answers, never the declared fields of record N+1.
    ///
    /// # Errors
    ///
    /// The decode's failure.
    fn masks(
        &self,
        _from: &FrameRecord,
        _next: &FrameRecord,
        _inputs: &Self::Inputs,
    ) -> Result<Vec<Mask>, Self::Skip> {
        Ok(Vec::new())
    }

    /// `world` with every answer copied from record N+1 replaced by the answer that copies
    /// nothing, or `None` when `world` copies nothing.
    fn neutral(&self, _world: &Self::World) -> Option<Self::World> {
        None
    }

    /// The model: one frame of `state` under `inputs`, asking `world`.
    fn step(&self, state: &mut Self::State, inputs: &Self::Inputs, world: &mut Self::World);

    /// The catalogue entries the frame exercised, by their places in [`Domain::behaviours`]:
    /// `state` and `inputs` as decoded, `world` as the step left it.
    fn exercised(
        &self,
        _state: &Self::State,
        _inputs: &Self::Inputs,
        _world: &Self::World,
    ) -> Vec<usize> {
        Vec::new()
    }

    /// The declared fields of the model's `state`.
    fn fields(&self, state: &Self::State) -> Self::Fields;

    /// The declared fields the oracle's record `next` holds.
    ///
    /// # Errors
    ///
    /// The reason the record isn't one the domain covers, or the decode's failure.
    fn expected(&self, next: &FrameRecord) -> Result<Self::Fields, Self::Skip>;

    /// Whether `skip` is a frame the domain doesn't run in, or a decode failure.
    fn uncompared(&self, skip: &Self::Skip) -> Uncompared {
        Uncompared::Skip(skip.to_string())
    }
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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FieldReport {
    /// The field's name.
    pub name: &'static str,
    /// The compared frames on which the field matched and was verified.
    pub matched: u64,
    /// The compared frames on which it matched, but its value came through an answer the
    /// stand-in copied from record N+1.
    pub unverified: u64,
    /// The compared frames on which it didn't match.
    pub mismatched: u64,
    /// The compared frames on which it was masked.
    pub masked: u64,
    /// The unmasked frames on which the null model matched it.
    pub null_matched: u64,
    /// The first [`FIRST_MISMATCHES`] mismatches, in frame order.
    pub first: Vec<Mismatch>,
    /// The first [`FIRST_MISMATCHES`] runs of consecutive mismatched frames.
    pub runs: Vec<Run>,
}

impl FieldReport {
    /// The frames the field was compared on, unmasked: matched, unverified or not.
    #[must_use]
    pub const fn compared(&self) -> u64 {
        self.matched
            .saturating_add(self.unverified)
            .saturating_add(self.mismatched)
    }

    const fn add(&mut self, other: &Self) {
        self.matched = self.matched.saturating_add(other.matched);
        self.unverified = self.unverified.saturating_add(other.unverified);
        self.mismatched = self.mismatched.saturating_add(other.mismatched);
        self.masked = self.masked.saturating_add(other.masked);
        self.null_matched = self.null_matched.saturating_add(other.null_matched);
    }
}

/// Consecutive frames on which a field mismatched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    /// The run's first frame.
    pub first: Frame,
    /// Its last frame.
    pub last: Frame,
}

/// One catalogue entry over a run.
#[derive(Clone, Debug)]
pub struct BehaviourReport {
    /// The entry's name.
    pub name: String,
    /// The compared frames that exercised it.
    pub exercised: u64,
    /// Those of them that matched and were verified on every unmasked field.
    pub matched: u64,
}

/// A domain's run over one scenario's records.
#[derive(Clone, Debug, Default)]
pub struct Report {
    /// The domain.
    pub domain: &'static str,
    /// The frames in the run, M: every pair of consecutive records.
    pub frames: u64,
    /// The frames compared: M less the skipped frames and the defects.
    pub compared: u64,
    /// The compared frames on which every unmasked field matched and was verified.
    pub matched: u64,
    /// The compared frames on which every unmasked field matched, but at least one only
    /// through an answer copied from record N+1.
    pub unverified: u64,
    /// The compared frames on which every field was masked.
    pub masked_whole: u64,
    /// The compared frames on which the null model matched every unmasked field.
    pub null_matched: u64,
    /// The frames the domain doesn't run in, by reason.
    pub skipped: BTreeMap<String, u64>,
    /// The frames the harness couldn't decode, by failure: harness defects.
    pub defects: BTreeMap<String, u64>,
    /// The compared frames with a mask, by the mask's reason.
    pub masked: BTreeMap<&'static str, u64>,
    /// Each declared field, in the domain's order.
    pub fields: Vec<FieldReport>,
    /// Each catalogue entry, in the catalogue's order.
    pub behaviours: Vec<BehaviourReport>,
}

/// What one compared frame gave.
struct Compared<'a, F> {
    frame: Frame,
    model: &'a F,
    oracle: &'a F,
    null: &'a F,
    masked: FieldSet,
    unverified: FieldSet,
}

impl Report {
    /// An empty report for `domain`.
    #[must_use]
    pub fn new<D: Domain>(domain: &D) -> Self {
        Self {
            domain: D::NAME,
            fields: D::FIELDS
                .iter()
                .map(|field| FieldReport {
                    name: field.name,
                    ..FieldReport::default()
                })
                .collect(),
            behaviours: domain
                .behaviours()
                .into_iter()
                .map(|name| BehaviourReport {
                    name,
                    exercised: 0,
                    matched: 0,
                })
                .collect(),
            ..Self::default()
        }
    }

    fn uncompared(&mut self, reason: Uncompared) {
        let (counts, reason) = match reason {
            Uncompared::Skip(reason) => (&mut self.skipped, reason),
            Uncompared::Defect(reason) => (&mut self.defects, reason),
        };
        let count = counts.entry(reason).or_default();
        *count = count.saturating_add(1);
    }

    /// The frames the domain doesn't run in.
    #[must_use]
    pub fn skipped_frames(&self) -> u64 {
        self.skipped.values().sum()
    }

    /// The frames the harness couldn't decode.
    #[must_use]
    pub fn defect_frames(&self) -> u64 {
        self.defects.values().sum()
    }

    fn compare<D: Domain>(&mut self, frame: &Compared<'_, D::Fields>, exercised: &[usize]) {
        self.compared = self.compared.saturating_add(1);
        let (mut all, mut verified, mut null_all, mut any) = (true, true, true, false);
        for (index, (field, report)) in D::FIELDS.iter().zip(&mut self.fields).enumerate() {
            if frame.masked.contains(index) {
                report.masked = report.masked.saturating_add(1);
                continue;
            }
            any = true;
            if (field.same)(frame.null, frame.oracle) {
                report.null_matched = report.null_matched.saturating_add(1);
            } else {
                null_all = false;
            }
            if (field.same)(frame.model, frame.oracle) {
                if frame.unverified.contains(index) {
                    verified = false;
                    report.unverified = report.unverified.saturating_add(1);
                } else {
                    report.matched = report.matched.saturating_add(1);
                }
                continue;
            }
            all = false;
            report.mismatched = report.mismatched.saturating_add(1);
            if report.first.len() < FIRST_MISMATCHES {
                report.first.push(Mismatch {
                    frame: frame.frame,
                    model: (field.show)(frame.model),
                    oracle: (field.show)(frame.oracle),
                });
            }
            let room = report.runs.len() < FIRST_MISMATCHES;
            match report.runs.last_mut() {
                Some(run) if run.last.0.checked_add(1) == Some(frame.frame.0) => {
                    run.last = frame.frame;
                }
                Some(_) | None if room => report.runs.push(Run {
                    first: frame.frame,
                    last: frame.frame,
                }),
                Some(_) | None => {}
            }
        }
        if !any {
            self.masked_whole = self.masked_whole.saturating_add(1);
            return;
        }
        match (all, verified) {
            (true, true) => self.matched = self.matched.saturating_add(1),
            (true, false) => self.unverified = self.unverified.saturating_add(1),
            (false, _) => {}
        }
        if null_all {
            self.null_matched = self.null_matched.saturating_add(1);
        }
        for &place in exercised {
            if let Some(report) = self.behaviours.get_mut(place) {
                report.exercised = report.exercised.saturating_add(1);
                if all && verified {
                    report.matched = report.matched.saturating_add(1);
                }
            }
        }
    }

    /// `other`'s counts added to `self`'s, a total over rows (mismatch lists aren't kept).
    pub fn add(&mut self, other: &Self) {
        self.domain = other.domain;
        self.frames = self.frames.saturating_add(other.frames);
        self.compared = self.compared.saturating_add(other.compared);
        self.matched = self.matched.saturating_add(other.matched);
        self.unverified = self.unverified.saturating_add(other.unverified);
        self.masked_whole = self.masked_whole.saturating_add(other.masked_whole);
        self.null_matched = self.null_matched.saturating_add(other.null_matched);
        for (counts, more) in [
            (&mut self.skipped, &other.skipped),
            (&mut self.defects, &other.defects),
        ] {
            for (reason, count) in more {
                let total = counts.entry(reason.clone()).or_default();
                *total = total.saturating_add(*count);
            }
        }
        for (reason, count) in &other.masked {
            let total = self.masked.entry(reason).or_default();
            *total = total.saturating_add(*count);
        }
        for field in &other.fields {
            match self.fields.iter_mut().find(|mine| mine.name == field.name) {
                Some(mine) => mine.add(field),
                None => self.fields.push(FieldReport {
                    first: Vec::new(),
                    runs: Vec::new(),
                    ..field.clone()
                }),
            }
        }
        for entry in &other.behaviours {
            match self
                .behaviours
                .iter_mut()
                .find(|mine| mine.name == entry.name)
            {
                Some(mine) => {
                    mine.exercised = mine.exercised.saturating_add(entry.exercised);
                    mine.matched = mine.matched.saturating_add(entry.matched);
                }
                None => self.behaviours.push(entry.clone()),
            }
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}: {} frames: {} compared, {} skipped, {} defects; {} matched, {} unverified, \
             {} masked whole; null model {} matched",
            self.domain,
            self.frames,
            self.compared,
            self.skipped_frames(),
            self.defect_frames(),
            self.matched,
            self.unverified,
            self.masked_whole,
            self.null_matched,
        )?;
        for (reason, count) in &self.skipped {
            writeln!(f, "  skipped {count}: {reason}")?;
        }
        for (reason, count) in &self.defects {
            writeln!(f, "  defect {count}: {reason}")?;
        }
        for (reason, count) in &self.masked {
            writeln!(f, "  masked {count}: {reason}")?;
        }
        for field in &self.fields {
            writeln!(
                f,
                "  {} {} of {} matched, {} unverified, {} masked; null {}",
                field.name,
                field.matched,
                field.compared(),
                field.unverified,
                field.masked,
                field.null_matched,
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

/// The per-behaviour table of a report: each entry's exercised and matched frames, then the
/// entries no compared frame exercised, "vectors only".
pub struct Behaviours<'a>(pub &'a Report);

impl fmt::Display for Behaviours<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (exercised, unexercised): (Vec<&BehaviourReport>, Vec<&BehaviourReport>) = self
            .0
            .behaviours
            .iter()
            .partition(|entry| entry.exercised > 0);
        for entry in exercised {
            writeln!(
                f,
                "  {}: {} matched of {} exercised",
                entry.name, entry.matched, entry.exercised
            )?;
        }
        let names: Vec<String> = unexercised.iter().map(|entry| entry.name.clone()).collect();
        writeln!(f, "  vectors only: {}", names.join(", "))
    }
}

/// The fields on which two projections differ.
fn differing<D: Domain>(left: &D::Fields, right: &D::Fields) -> FieldSet {
    D::FIELDS
        .iter()
        .enumerate()
        .filter(|(_, field)| !(field.same)(left, right))
        .fold(FieldSet::EMPTY, |set, (index, _)| {
            set.union(FieldSet::single(index))
        })
}

/// One frame from `from` to `next`, compared into `report`.
fn frame<D: Domain>(
    domain: &D,
    report: &mut Report,
    from: &FrameRecord,
    next: &FrameRecord,
) -> Result<(), D::Skip> {
    let (mut state, inputs) = domain.decode(from, Interval::ending_at(next))?;
    let mut world = domain.world(from, next)?;
    let masks = domain.masks(from, next, &inputs)?;
    let oracle = domain.expected(next)?;
    let decoded = state.clone();
    let null = domain.fields(&decoded);
    let neutral = domain.neutral(&world);
    domain.step(&mut state, &inputs, &mut world);
    let model = domain.fields(&state);
    let unverified = neutral.map_or(FieldSet::EMPTY, |mut neutral| {
        let mut alone = decoded.clone();
        domain.step(&mut alone, &inputs, &mut neutral);
        differing::<D>(&domain.fields(&alone), &model)
    });
    let masked = masks
        .iter()
        .fold(FieldSet::EMPTY, |set, mask| set.union(mask.fields));
    for mask in &masks {
        let count = report.masked.entry(mask.reason).or_default();
        *count = count.saturating_add(1);
    }
    let exercised = domain.exercised(&decoded, &inputs, &world);
    report.compare::<D>(
        &Compared {
            frame: Frame(next.index),
            model: &model,
            oracle: &oracle,
            null: &null,
            masked,
            unverified,
        },
        &exercised,
    );
    Ok(())
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
    let mut report = Report::new(domain);
    let mut records = records.into_iter();
    let Some(first) = records.next() else {
        return Ok(report);
    };
    let mut from = first?;
    for record in records {
        let next = record?;
        report.frames = report.frames.saturating_add(1);
        if let Err(reason) = frame(domain, &mut report, &from, &next) {
            report.uncompared(domain.uncompared(&reason));
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
