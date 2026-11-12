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
//! - a frame on which another domain's writer moved a declared field (**masked**, a mask that
//!   hides [`Hides::AnotherWriter`]): the carried State can't follow it, so the frame counts
//!   as held on its unmasked fields and the next frame seeds afresh. A mask no writer stands
//!   behind ([`Hides::NoWriter`]: a field reported and not counted, or one the frame doesn't
//!   produce) leaves the field uncompared and the segment running;
//! - a frame the domain doesn't run in or can't decode (**skipped**);
//! - the row's end.
//!
//! The next segment seeds from the next frame the domain runs in. A segment's **held** frames
//! are those it ran before it ended, all matching. Each of a row's records but its first, which
//! only seeds, is one of: **folded**, a record the domain folds into the frame before it, so it
//! ends no frame of its own; **skipped**, ending a frame the domain doesn't run in or can't
//! decode; or **compared**, ending a frame held or diverged. A compared frame on which another
//! writer moved a declared field is compared on the rest (**masked**). The report gives, per
//! row, the frames held and how its records are accounted for, the segments by how they ended,
//! the longest, and each field's first divergence. A model that holds a row in one segment
//! plays that row alone, as far as its fields go; a row of many short segments ending in masks
//! says the domain's boundary, not its model, limits the proof.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use oracle_record::FrameRecord;

use super::{
    Domain, FieldSet, HarnessError, Hides, Interval, Mismatch, differing, frames, open_cached,
};
use crate::lock::Frame;

/// The longest segments a report keeps.
pub const LONGEST: usize = 5;

/// How a segment ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum End {
    /// A declared field differed at this frame: the fields, with both values.
    Diverged(Frame, Vec<(&'static str, Mismatch)>),
    /// Another domain's writer moved these fields at this frame, which the carried State can't
    /// follow.
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
    /// The frames in the run: every pair of records the domain doesn't fold together.
    pub frames: u64,
    /// The records folded into the frame before them, ending no frame of their own.
    pub folded: u64,
    /// The frames the domain doesn't run in or couldn't decode, so never compared.
    pub skipped_frames: u64,
    /// The compared frames on which another domain's writer moved a declared field, compared on
    /// the others.
    pub masked_frames: u64,
    /// The records that end a compared frame, by index: a row's, as the longest segments are.
    pub compared_at: BTreeSet<u64>,
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

    /// The frames compared: run from a seed, held or diverged.
    #[must_use]
    pub const fn compared(&self) -> u64 {
        self.frames.saturating_sub(self.skipped_frames)
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
        self.folded = self.folded.saturating_add(other.folded);
        self.skipped_frames = self.skipped_frames.saturating_add(other.skipped_frames);
        self.masked_frames = self.masked_frames.saturating_add(other.masked_frames);
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
            "{} free run: {} frames compared, {} held, {} masked; {} skipped, {} folded; \
             {} segments: {} diverged, {} masked, {} skipped, {} to the row's end",
            self.domain,
            self.compared(),
            self.held,
            self.masked_frames,
            self.skipped_frames,
            self.folded,
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

/// One frame from `from` to `next`, stepping the segment in progress or seeding one, and
/// whether another writer moved a declared field in it; `None` when the domain doesn't run in
/// the frame or can't decode it.
fn step<D: Domain>(
    domain: &D,
    open: Option<Open<D::State>>,
    from: &FrameRecord,
    next: &FrameRecord,
) -> Option<(Step<D::State>, bool)> {
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
    let hidden = |hides| {
        masks
            .iter()
            .filter(|mask| mask.hides == hides)
            .fold(FieldSet::EMPTY, |set, mask| set.union(mask.fields))
    };
    let lost = hidden(Hides::AnotherWriter);
    let masked = lost.union(hidden(Hides::NoWriter));
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
        let ended = Step::Ended(Segment {
            seed,
            held,
            end: End::Diverged(frame, fields),
        });
        return Some((ended, !lost.is_empty()));
    }
    let held = held.saturating_add(1);
    if lost.is_empty() {
        Some((Step::Held(Open { seed, held, state }), false))
    } else {
        let ended = Step::Ended(Segment {
            seed,
            held,
            end: End::Masked(frame, names::<D>(lost)),
        });
        Some((ended, true))
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
    let mut read = 0_u64;
    let records = records
        .into_iter()
        .inspect(|_| read = read.saturating_add(1));
    frames(domain, records, |from, next| {
        report.frames = report.frames.saturating_add(1);
        let carried = open.take();
        let (seed, held) = carried
            .as_ref()
            .map_or((Frame(from.index), 0), |open| (open.seed, open.held));
        let Some((stepped, masked)) = step(domain, carried, from, next) else {
            report.skipped_frames = report.skipped_frames.saturating_add(1);
            if held > 0 {
                report.close(Segment {
                    seed,
                    held,
                    end: End::Skipped(Frame(next.index)),
                });
            }
            return;
        };
        report.compared_at.insert(next.index);
        if masked {
            report.masked_frames = report.masked_frames.saturating_add(1);
        }
        match stepped {
            Step::Held(running) => open = Some(running),
            Step::Ended(segment) => report.close(segment),
        }
    })?;
    // The first record only seeds; every other record ends a frame or is folded into one.
    report.folded = read.saturating_sub(1).saturating_sub(report.frames);
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

/// A domain's free run over rows as `free_floors.txt` holds it: a floor under the frames it
/// holds, and how it accounts for the rows' records after their seeds, which holds exactly.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Accounting {
    /// The frames held, a floor.
    pub held: u64,
    /// The frames compared, held or diverged.
    pub compared: u64,
    /// Of those, the frames another writer moved a declared field in, compared on the rest.
    pub masked: u64,
    /// The frames the domain doesn't run in or can't decode.
    pub skipped: u64,
    /// The records folded into the frame before them.
    pub folded: u64,
}

impl Accounting {
    /// A row's report as its line would hold it.
    #[must_use]
    pub const fn of(report: &FreeReport) -> Self {
        Self {
            held: report.held,
            compared: report.compared(),
            masked: report.masked_frames,
            skipped: report.skipped_frames,
            folded: report.folded,
        }
    }

    /// Whether `self` accounts for the records as `other` does, whatever each holds.
    #[must_use]
    pub const fn same_records(&self, other: &Self) -> bool {
        self.compared == other.compared
            && self.masked == other.masked
            && self.skipped == other.skipped
            && self.folded == other.folded
    }

    /// The records after the rows' seeds: compared, skipped or folded.
    #[must_use]
    pub const fn frames(&self) -> u64 {
        self.compared
            .saturating_add(self.skipped)
            .saturating_add(self.folded)
    }

    /// `other`'s figures added to `self`'s, a total over rows.
    pub const fn add(&mut self, other: &Self) {
        self.held = self.held.saturating_add(other.held);
        self.compared = self.compared.saturating_add(other.compared);
        self.masked = self.masked.saturating_add(other.masked);
        self.skipped = self.skipped.saturating_add(other.skipped);
        self.folded = self.folded.saturating_add(other.folded);
    }

    /// Its line in `free_floors.txt` for `domain` on `row`.
    #[must_use]
    pub fn line(&self, domain: &str, row: &str) -> String {
        format!(
            "{domain} {row} {} {} {} {} {}",
            self.held, self.compared, self.masked, self.skipped, self.folded
        )
    }
}

/// The word that starts a row's line of the frames no model domain (all but the clock) compares in `free_floors.txt`.
pub const UNCOMPARED: &str = "uncompared";

/// The committed free-run floors, `free_floors.txt`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Floors {
    /// Each domain's run over each row, by (domain, row).
    pub domains: BTreeMap<(String, String), Accounting>,
    /// Per row, its frames after the seed that no model domain compares.
    pub uncompared: BTreeMap<String, u64>,
}

impl Floors {
    /// The floors in `text`: `<domain> <row> <held> <compared> <masked> <skipped> <folded>` and
    /// `uncompared <row> <frames>` lines, `#` comments skipped.
    ///
    /// # Errors
    ///
    /// The first line that is neither, with why.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut floors = Self::default();
        for line in text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
        {
            let number = |word: &str| word.parse::<u64>().map_err(|e| format!("{line}: {e}"));
            match line.split_whitespace().collect::<Vec<_>>().as_slice() {
                [UNCOMPARED, row, frames] => {
                    floors.uncompared.insert((*row).to_owned(), number(frames)?);
                }
                [domain, row, held, compared, masked, skipped, folded] => {
                    let accounting = Accounting {
                        held: number(held)?,
                        compared: number(compared)?,
                        masked: number(masked)?,
                        skipped: number(skipped)?,
                        folded: number(folded)?,
                    };
                    floors
                        .domains
                        .insert(((*domain).to_owned(), (*row).to_owned()), accounting);
                }
                _ => {
                    return Err(format!(
                        "{line}: not `domain row held compared masked skipped folded` nor \
                         `{UNCOMPARED} row frames`"
                    ));
                }
            }
        }
        Ok(floors)
    }
}

#[cfg(test)]
mod tests;
