//! The free-run localiser: where a row's free run first leaves the oracle, per field.
//!
//! Lockstep ([`crate::lockstep`]) re-seeds from the oracle every Tick, so its first difference
//! says where one Tick is wrong, not where the free run breaks; the free-run gate
//! ([`crate::gate`]) folds digests over 1000-frame spans, so it names a span, not a frame.
//! The localiser seeds once from the row's seed record and steps freely as the gate does
//! ([`gate::step_freely`]), and after every Tick compares the engine's State with the oracle's
//! State decoded from the row's record cache at the same frame, with lockstep's comparators.
//!
//! The result is a lockstep [`Report`], one step per frame whose record decodes, so its field
//! summaries ([`Report::fields`]) give each field's first differing frame and values, and the
//! ratchet's [`Summary`] the first differing frame per group.

use std::fmt;
use std::path::Path;
use std::sync::Arc;

use data::span::Cut;
use game::Content;

use crate::bundle::{RowInputs, content, rooms};
use crate::cache::{Cache, CacheError, CutKind};
use crate::decode::{RoomMap, decode};
use crate::gate::{Stop, step_freely};
use crate::lock::{Frame, FrameCount, Lock, Row, RowId};
use crate::lockstep::{ErrorKind, Report, Skip, Step, auto_repeat, differences};
use crate::ratchet::Summary;

/// The first frame whose cut the engine ended differently from the oracle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CutDifference {
    /// The frame.
    pub frame: Frame,
    /// The engine's cut.
    pub engine: Cut,
    /// The oracle's cut.
    pub oracle: Cut,
}

/// A row's free run localised against the oracle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Localised {
    /// The row's id.
    pub row: RowId,
    /// The frames the engine stepped.
    pub frames: FrameCount,
    /// Why the run stopped before the row's last frame, if it did.
    pub stop: Option<Stop>,
    /// The comparison: a step per frame whose record decodes, the others counted as skipped.
    pub report: Report,
    /// The first frame the cuts differ at, if any.
    pub cut: Option<CutDifference>,
}

/// The cut the recorder's span fold names for a cached cut kind.
const fn span_cut(kind: CutKind) -> Cut {
    match kind {
        CutKind::Requested => Cut::Requested,
        CutKind::Overrun => Cut::Overrun,
        CutKind::VblankWithoutNmi => Cut::VblankWithoutNmi,
        CutKind::NmiStillRunning => Cut::NmiStillRunning,
    }
}

/// Runs `row` freely from `inputs` and compares every frame with `cache`'s.
///
/// # Errors
///
/// A [`CacheError`] when a record doesn't read or isn't the frame the run is at.
pub fn localise(
    row: &Row,
    inputs: RowInputs,
    cache: &Cache,
    rooms: &RoomMap,
    content: &Arc<Content>,
) -> Result<Localised, CacheError> {
    let delays = inputs.seed.rules.auto_repeat;
    let mut report = Report {
        auto_repeat: delays,
        auto_repeat_changes: 0,
        steps: Vec::new(),
        skipped: Vec::new(),
        undecodable: std::collections::BTreeMap::new(),
    };
    let mut frames = 0_u64;
    let mut first_cut = None;
    let mut failure = None;
    let stop = step_freely(
        row,
        inputs.seed,
        &inputs.pads,
        content,
        |frame, state, cut| {
            frames = frames.saturating_add(1);
            if failure.is_some() {
                return;
            }
            let entry = usize::try_from(frame.0)
                .ok()
                .and_then(|index| cache.entries().get(index))
                .filter(|entry| entry.frame == frame);
            let record = match entry
                .ok_or(CacheError::OutOfRange(frame))
                .and_then(|entry| cache.record(entry))
            {
                Ok(record) => record,
                Err(error) => {
                    failure = Some(error);
                    return;
                }
            };
            let oracle_cut = span_cut(CutKind::of(record.cut));
            if first_cut.is_none() && oracle_cut != cut {
                first_cut = Some(CutDifference {
                    frame,
                    engine: cut,
                    oracle: oracle_cut,
                });
            }
            match decode(&record, rooms) {
                Ok(oracle) => {
                    if auto_repeat(&record).ok() != Some(delays) {
                        report.auto_repeat_changes = report.auto_repeat_changes.saturating_add(1);
                    }
                    report.steps.push(Step {
                        frame,
                        differences: differences(state, &oracle.state, oracle.quiescence),
                    });
                }
                Err(error) => {
                    let kind = ErrorKind::of(&error);
                    let count = report.undecodable.entry(kind).or_default();
                    *count = count.saturating_add(1);
                    let skip = Skip::Undecodable(kind);
                    match report.skipped.iter_mut().find(|(seen, _)| *seen == skip) {
                        Some((_, count)) => *count = count.saturating_add(1),
                        None => report.skipped.push((skip, 1)),
                    }
                }
            }
        },
    );
    if let Some(error) = failure {
        return Err(error);
    }
    Ok(Localised {
        row: row.id.clone(),
        frames: FrameCount(frames),
        stop,
        report,
        cut: first_cut,
    })
}

/// Localises row `row` of the lock at `lock`, seeded from the relock bundle directory
/// `bundle`, against the record cache under `cache`, on the content read from `rom`.
///
/// # Errors
///
/// The unreadable lock, bundle, cache or ROM, as text.
pub fn run_row(
    lock: &Path,
    bundle: &Path,
    cache: &Path,
    rom: &Path,
    row: &str,
) -> Result<Localised, String> {
    let shown = |path: &Path| path.display().to_string();
    let text =
        std::fs::read_to_string(lock).map_err(|error| format!("{}: {error}", shown(lock)))?;
    let lock = Lock::parse(&text).map_err(|error| format!("{}: {error:?}", shown(lock)))?;
    let row = lock
        .row(&RowId(row.to_owned()))
        .ok_or_else(|| format!("{row}: not in the lock"))?;
    let cache = Cache::open(cache, row).map_err(|error| error.to_string())?;
    let image = std::fs::read(rom).map_err(|error| format!("{}: {error}", shown(rom)))?;
    let content =
        Arc::new(content(image.clone()).map_err(|error| format!("{}: {error}", shown(rom)))?);
    let rooms = rooms(&content, &image).map_err(|error| format!("{}: {error:?}", shown(rom)))?;
    let inputs = RowInputs::read(bundle, row, &rooms).map_err(|error| error.to_string())?;
    localise(row, inputs, &cache, &rooms, &content).map_err(|error| error.to_string())
}

impl Localised {
    /// Each field that differs, with its first differing frame and difference, earliest
    /// first; fields first differing at the same frame keep the coverage table's order.
    #[must_use]
    pub fn first_frames(&self) -> Vec<(Frame, &'static str, canonical::Difference)> {
        let mut fields: Vec<_> = self
            .report
            .fields()
            .into_iter()
            .filter_map(|field| {
                field
                    .first
                    .map(|(frame, difference)| (frame, field.path, difference))
            })
            .collect();
        fields.sort_by_key(|(frame, _, _)| frame.0);
        fields
    }

    /// The ratchet's numbers for the free run: per group, the frames it differs on and the
    /// first of them, the frames until its first difference.
    #[must_use]
    pub fn summary(&self) -> Summary {
        Summary::of(&self.row.0, &self.report)
    }
}

impl fmt::Display for Localised {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let summary = self.summary();
        writeln!(
            f,
            "{}: {} frames stepped, {} compared, {} equal",
            self.row, self.frames.0, summary.ticks, summary.equal
        )?;
        if let Some(stop) = &self.stop {
            writeln!(f, "  stopped: {stop}")?;
        }
        writeln!(f, "  undecodable frames {:?}", self.report.undecodable)?;
        match &self.cut {
            Some(cut) => writeln!(
                f,
                "  cut first differs at frame {}: engine {:?}, oracle {:?}",
                cut.frame.0, cut.engine, cut.oracle
            )?,
            None => writeln!(f, "  cut equal on every frame")?,
        }
        writeln!(f, "  groups (first differing frame, frames differing):")?;
        let mut groups: Vec<_> = summary.groups.iter().collect();
        groups.sort_by_key(|(_, group)| group.first);
        for (name, group) in groups {
            writeln!(f, "    {name} {} {}", group.first, group.differed)?;
        }
        writeln!(
            f,
            "  fields (first differing frame, first difference: engine != oracle):"
        )?;
        for (frame, _, difference) in self.first_frames() {
            writeln!(f, "    {} {difference}", frame.0)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{CutKind, run_row, span_cut};
    use std::path::{Path, PathBuf};

    #[test]
    fn every_cached_cut_has_its_span_cut() {
        for kind in [
            CutKind::Requested,
            CutKind::Overrun,
            CutKind::VblankWithoutNmi,
            CutKind::NmiStillRunning,
        ] {
            assert_eq!(span_cut(kind).quiescent(), kind.quiescent());
        }
    }

    /// The localiser runs both slice-0 rows to their last frame against the record cache in
    /// `SM_CACHE`, seeded from the relock bundle directory in `SM_BUNDLE`, on the ROM in
    /// `SM_ROM`. Skipped only where `SM_BUNDLE` is unset; with it set, the other two must
    /// be too. Run with `--nocapture` for the tables.
    #[test]
    fn the_free_run_localises_against_the_cache() {
        use std::io::Write as _;
        let Some(bundle) = std::env::var_os("SM_BUNDLE").map(PathBuf::from) else {
            let _ = writeln!(std::io::stdout(), "skipped: SM_BUNDLE unset");
            return;
        };
        let path = |name: &str| {
            std::env::var_os(name)
                .map(PathBuf::from)
                .ok_or_else(|| format!("SM_BUNDLE is set, so {name} must be"))
        };
        let (cache, rom) = match (path("SM_CACHE"), path("SM_ROM")) {
            (Ok(cache), Ok(rom)) => (cache, rom),
            (Err(error), _) | (_, Err(error)) => panic!("{error}"),
        };
        let lock = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/locks/lsnes.toml");
        for row in ["area-brinstar", "item-morph-ball"] {
            match run_row(&lock, &bundle, &cache, &rom, row) {
                Ok(localised) => {
                    let _ = write!(std::io::stdout(), "{localised}");
                    assert_eq!(localised.stop, None, "{row} stopped early");
                    let skipped: u64 = localised.report.skipped.iter().map(|(_, n)| n).sum();
                    let compared = u64::try_from(localised.report.steps.len()).unwrap_or(0);
                    assert_eq!(compared.saturating_add(skipped), localised.frames.0);
                }
                Err(error) => panic!("{row}: {error}"),
            }
        }
    }
}
