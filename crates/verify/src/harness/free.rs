//! The domain free run (MODEL §4, "The primary proof is a free run"): one domain's model fed
//! its own State across a row, everything else from the records.
//!
//! The per-frame harness ([`super::run`]) decodes the domain's State from every record N, so
//! an error never outlives its frame: it finds where a model goes wrong, but a model can look
//! right on nearly every frame and still not be the game. The free run decodes the State once,
//! at a seed, and from then on carries the model's own: each frame reads only its inputs and
//! its neighbours' answers from the records, steps the carried State, and compares the
//! declared fields with record N+1's.
//!
//! A **segment** is one such run from a seed. It ends at:
//! - the first frame on which an unmasked field differs (**diverged**): the measure;
//! - a frame with a mask (**masked**): another domain's writer moved a declared field, which
//!   the carried State can't follow, so the frame counts as held on its unmasked fields and
//!   the next frame seeds afresh;
//! - a frame the domain doesn't run in or can't decode (**skipped**);
//! - the row's end.
//!
//! The next segment seeds from the next frame the domain runs in. A segment's **held** frames
//! are those it ran before it ended, all matching. The report gives, per row, the frames held,
//! the segments by how they ended, the longest, and each field's first divergence. A model
//! that holds a row in one segment plays that row alone, as far as its fields go; a row of
//! many short segments ending in masks says the domain's boundary, not its model, limits the
//! proof.

use core::fmt;
use std::collections::BTreeMap;
use std::path::Path;

use oracle_record::FrameRecord;

use super::{Domain, FieldSet, HarnessError, Interval, Mismatch, differing, frames, open_cached};
use crate::lock::Frame;

/// The longest segments a report keeps.
pub const LONGEST: usize = 5;

/// How a segment ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum End {
    /// A declared field differed at this frame: the fields, with both values.
    Diverged(Frame, Vec<(&'static str, Mismatch)>),
    /// A mask hid a field at this frame, which the carried State can't follow.
    Masked(Frame, Vec<&'static str>),
    /// The domain doesn't run in, or couldn't decode, the frame ending here.
    Skipped(Frame),
    /// The row ended.
    RowEnd,
}

/// One run from a seed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    /// The record its State was decoded from.
    pub seed: Frame,
    /// The frames it ran, all matching on every unmasked field.
    pub held: u64,
    /// How it ended.
    pub end: End,
}

/// A domain's free run over one or more rows.
#[derive(Clone, Debug, Default)]
pub struct FreeReport {
    /// The domain.
    pub domain: &'static str,
    /// The frames in the run: every pair of consecutive records.
    pub frames: u64,
    /// The frames held: run from a seed and matching on every unmasked field.
    pub held: u64,
    /// The segments that ended in a divergence.
    pub diverged: u64,
    /// The segments that ended at a mask.
    pub masked: u64,
    /// The segments that ended at a frame the domain doesn't run in or couldn't decode.
    pub skipped: u64,
    /// The segments that ran to the row's end.
    pub row_ends: u64,
    /// The [`LONGEST`] longest segments, longest first.
    pub longest: Vec<Segment>,
    /// Each declared field's first divergence, by its name, in the domain's order; `None`
    /// where it never diverged.
    pub first: Vec<(&'static str, Option<Mismatch>)>,
    /// How often each field ended a segment, by its name.
    pub ended_by: BTreeMap<&'static str, u64>,
}

impl FreeReport {
    /// An empty report for `D`.
    #[must_use]
    pub fn new<D: Domain>() -> Self {
        Self {
            domain: D::NAME,
            first: D::FIELDS.iter().map(|field| (field.name, None)).collect(),
            ..Self::default()
        }
    }

    /// The segments, of every kind.
    #[must_use]
    pub const fn segments(&self) -> u64 {
        self.diverged
            .saturating_add(self.masked)
            .saturating_add(self.skipped)
            .saturating_add(self.row_ends)
    }

    fn close(&mut self, segment: Segment) {
        self.held = self.held.saturating_add(segment.held);
        let count = match &segment.end {
            End::Diverged(_, fields) => {
                for (name, mismatch) in fields {
                    let ended = self.ended_by.entry(name).or_default();
                    *ended = ended.saturating_add(1);
                    if let Some((_, first @ None)) =
                        self.first.iter_mut().find(|(field, _)| field == name)
                    {
                        *first = Some(mismatch.clone());
                    }
                }
                &mut self.diverged
            }
            End::Masked(..) => &mut self.masked,
            End::Skipped(_) => &mut self.skipped,
            End::RowEnd => &mut self.row_ends,
        };
        *count = count.saturating_add(1);
        self.keep_longest(segment);
    }

    fn keep_longest(&mut self, segment: Segment) {
        let place = self
            .longest
            .iter()
            .position(|kept| kept.held < segment.held)
            .unwrap_or(self.longest.len());
        if place < LONGEST {
            self.longest.insert(place, segment);
            self.longest.truncate(LONGEST);
        }
    }

    /// `other`'s counts added to `self`'s, a total over rows. The longest segments and first
    /// divergences are a row's, so the total keeps none.
    pub fn add(&mut self, other: &Self) {
        self.domain = other.domain;
        self.frames = self.frames.saturating_add(other.frames);
        self.held = self.held.saturating_add(other.held);
        self.diverged = self.diverged.saturating_add(other.diverged);
        self.masked = self.masked.saturating_add(other.masked);
        self.skipped = self.skipped.saturating_add(other.skipped);
        self.row_ends = self.row_ends.saturating_add(other.row_ends);
        for (name, count) in &other.ended_by {
            let total = self.ended_by.entry(name).or_default();
            *total = total.saturating_add(*count);
        }
    }
}

impl fmt::Display for FreeReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} free run: {} frames, {} held; {} segments: {} diverged, {} masked, {} skipped, \
             {} to the row's end",
            self.domain,
            self.frames,
            self.held,
            self.segments(),
            self.diverged,
            self.masked,
            self.skipped,
            self.row_ends,
        )?;
        for segment in &self.longest {
            write!(f, "  from {}: {} held, ", segment.seed.0, segment.held)?;
            match &segment.end {
                End::Diverged(frame, fields) => {
                    let names: Vec<&str> = fields.iter().map(|(name, _)| *name).collect();
                    writeln!(f, "diverged at {} on {}", frame.0, names.join(", "))
                }
                End::Masked(frame, fields) => {
                    writeln!(f, "masked at {} ({})", frame.0, fields.join(", "))
                }
                End::Skipped(frame) => writeln!(f, "skipped at {}", frame.0),
                End::RowEnd => writeln!(f, "to the row's end"),
            }?;
        }
        for (name, first) in &self.first {
            if let Some(mismatch) = first {
                writeln!(
                    f,
                    "  {name} first diverged at {}: model {} oracle {}",
                    mismatch.frame.0, mismatch.model, mismatch.oracle
                )?;
            }
        }
        for (name, count) in &self.ended_by {
            writeln!(f, "  {name} ended {count} segments")?;
        }
        Ok(())
    }
}

/// The segment in progress: its seed, its frames so far and the State it carries.
struct Open<S> {
    seed: Frame,
    held: u64,
    state: S,
}

/// What one frame of a free run gave.
enum Step<S> {
    /// The frame matched and the segment runs on with this State.
    Held(Open<S>),
    /// The frame ended the segment.
    Ended(Segment),
}

/// The names of the fields in `set`, in the domain's order.
fn names<D: Domain>(set: FieldSet) -> Vec<&'static str> {
    D::FIELDS
        .iter()
        .enumerate()
        .filter(|(index, _)| set.contains(*index))
        .map(|(_, field)| field.name)
        .collect()
}

/// One frame from `from` to `next`, stepping the segment in progress or seeding one; `None`
/// when the domain doesn't run in the frame or can't decode it.
fn step<D: Domain>(
    domain: &D,
    open: Option<Open<D::State>>,
    from: &FrameRecord,
    next: &FrameRecord,
) -> Option<Step<D::State>> {
    let (seeded, inputs) = domain.decode(from, Interval::ending_at(next)).ok()?;
    let mut world = domain.world(from, next).ok()?;
    let masks = domain.masks(from, next, &inputs).ok()?;
    let oracle = domain.expected(next).ok()?;
    let Open {
        seed,
        held,
        mut state,
    } = open.unwrap_or(Open {
        seed: Frame(from.index),
        held: 0,
        state: seeded,
    });
    domain.step(&mut state, &inputs, &mut world);
    let model = domain.fields(&state);
    let masked = masks
        .iter()
        .fold(FieldSet::EMPTY, |set, mask| set.union(mask.fields));
    let differs = differing::<D>(&model, &oracle).without(masked);
    let frame = Frame(next.index);
    if !differs.is_empty() {
        let fields = D::FIELDS
            .iter()
            .enumerate()
            .filter(|(index, _)| differs.contains(*index))
            .map(|(_, field)| {
                (
                    field.name,
                    Mismatch {
                        frame,
                        model: (field.show)(&model),
                        oracle: (field.show)(&oracle),
                    },
                )
            })
            .collect();
        return Some(Step::Ended(Segment {
            seed,
            held,
            end: End::Diverged(frame, fields),
        }));
    }
    let held = held.saturating_add(1);
    if masked.is_empty() {
        Some(Step::Held(Open { seed, held, state }))
    } else {
        Some(Step::Ended(Segment {
            seed,
            held,
            end: End::Masked(frame, names::<D>(masked)),
        }))
    }
}

/// `domain` run free over `records`, a scenario's records in frame order.
///
/// # Errors
///
/// The first record that couldn't be read.
pub fn run<D: Domain, E>(
    domain: &D,
    records: impl IntoIterator<Item = Result<FrameRecord, E>>,
) -> Result<FreeReport, E> {
    let mut report = FreeReport::new::<D>();
    let mut open: Option<Open<D::State>> = None;
    frames(domain, records, |from, next| {
        report.frames = report.frames.saturating_add(1);
        let carried = open.take();
        let (seed, held) = carried
            .as_ref()
            .map_or((Frame(from.index), 0), |open| (open.seed, open.held));
        match step(domain, carried, from, next) {
            Some(Step::Held(running)) => open = Some(running),
            Some(Step::Ended(segment)) => report.close(segment),
            None if held > 0 => report.close(Segment {
                seed,
                held,
                end: End::Skipped(Frame(next.index)),
            }),
            None => {}
        }
    })?;
    if let Some(Open { seed, held, .. }) = open {
        report.close(Segment {
            seed,
            held,
            end: End::RowEnd,
        });
    }
    Ok(report)
}

/// `domain` run free over row `row`'s record cache under `cache`, checked against the lock at
/// `lock`.
///
/// # Errors
///
/// A [`HarnessError`] when the lock, the row or its cache can't be read.
pub fn run_cached<D: Domain>(
    domain: &D,
    lock: &Path,
    cache: &Path,
    row: &str,
) -> Result<FreeReport, HarnessError> {
    let cache = open_cached(lock, cache, row)?;
    let records = cache
        .entries()
        .iter()
        .map(|entry| cache.record(entry).map_err(HarnessError::from));
    run(domain, records)
}

#[cfg(test)]
mod tests;
