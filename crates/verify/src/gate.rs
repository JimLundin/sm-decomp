//! The free-run gate (ARCHITECTURE D22, docs/ORACLE.md, RELOCK §8).
//!
//! A row's free run from its start State under the `Recorded` timing source is folded as the
//! recorder folds the oracle's records and compared with the row's lock; green (row, stream)
//! pairs are kept in a ratchet.
//!
//! The start State is the seed's record decoded: the oracle's cut before the row's first
//! frame (`seeds/<capture>/NNNN.record`, [`crate::bundle`]). Every frame of the row is one
//! Tick, frame 0's included, with that frame's pad. The lock's `overruns` and `nmi_off` frames
//! are counted from the row's first, so the Tick that ends at frame f takes the schedule's
//! entry for f.
//!
//! The `sram` stream is the State's save RAM encoded back to its battery image each frame.
//! What the engine doesn't give yet: the shown picture (the scan-out has no line program
//! until B6) is left out, so the comparison reports it `Absent(Side::Engine)`. The CPU's APU port accesses are none yet (no sound domain until
//! C11), so `apu` folds an empty log on every busy span.

use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;

use data::span::Cut;
use game::nmi::NmiKind;
use game::state::sram::SAVE_RAM_BYTES;
use game::state::{Buttons, NmiTiming, State, Tick};
use game::{
    Content, OpenError, Rules, Schedule, Session, SessionFault, Switches, TickInput, TimingSource,
};
use oracle_record::{FrameRecord, Pad};

use crate::decode::{DecodeError, RoomMap};
use crate::fold::{EngineFold, EngineFrame};
use crate::lock::{Frame, FrameCount, Row, RowId, RowReport, Stream};
use crate::lockstep::auto_repeat;

/// Why a free run stopped before the row's last frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stop {
    /// The Session refused the start State.
    Open(OpenError),
    /// The Tick ending at this frame faulted.
    Fault {
        /// The frame the Tick would have ended at.
        frame: Frame,
        /// The fault.
        fault: SessionFault,
    },
    /// The pads end before this frame.
    NoPad(Frame),
}

impl fmt::Display for Stop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open(error) => write!(f, "the start State doesn't open: {error:?}"),
            Self::Fault { frame, fault } => write!(f, "frame {}: {fault:?}", frame.0),
            Self::NoPad(frame) => write!(f, "frame {}: no pad", frame.0),
        }
    }
}

/// A row's free run against its lock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreeRun {
    /// The comparison, per stream and fact.
    pub report: RowReport,
    /// The frames folded.
    pub frames: FrameCount,
    /// Why the run stopped early, if it did; the streams then differ at the span the run
    /// didn't finish.
    pub stop: Option<Stop>,
}

/// The engine's buttons for a pad file's JOY1 word.
#[must_use]
pub fn buttons(pad: Pad) -> Buttons {
    crate::decode::buttons(pad.0)
}

/// The cut a frame ended at, as the recorder names it.
///
/// The record's cut says where the NMI found the main thread, which is the timing source's
/// side; the handler's path (commit or lag) doesn't change it. A message box's lag NMI finds
/// the main thread at a wait site, so the oracle records it as requested.
const fn cut(timing: NmiTiming, nmi: NmiKind) -> Cut {
    match (timing, nmi) {
        (NmiTiming::Disabled, _) | (_, NmiKind::Disabled) => Cut::VblankWithoutNmi,
        (NmiTiming::Overrun, NmiKind::Committed | NmiKind::Lag) => Cut::Overrun,
        (NmiTiming::OnTime, NmiKind::Committed | NmiKind::Lag) => Cut::Requested,
    }
}

/// The engine frame a State at a cut gives, with its save RAM's image.
const fn engine_frame<'a>(
    state: &'a State,
    cut: Cut,
    ppu: u64,
    sram: &'a [u8; SAVE_RAM_BYTES],
) -> EngineFrame<'a> {
    EngineFrame {
        cut,
        video: &state.video.memory,
        sram: Some(sram),
        ppu: Some(ppu),
        ports: &[],
        shown: None,
    }
}

fn fold_state(fold: &mut EngineFold, state: &State, cut: Cut) {
    let ppu = crate::ppu::digest(&state.video);
    let sram = state.sram.to_bytes();
    fold.fold(&engine_frame(state, cut, ppu, &sram));
}

/// The row's `Recorded` timing source: its overrun and NMI-off frames as Ticks keyed by the
/// frame each Tick ends at.
fn recorded(row: &Row) -> TimingSource {
    let ticks = |frames: &[Frame]| frames.iter().map(|frame| Tick(frame.0)).collect();
    TimingSource::Recorded(Schedule::new(ticks(&row.overruns), ticks(&row.nmi_off)))
}

/// Where a free run starts: the cut before the row's first frame as a State, and the rules it
/// runs under.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seed {
    /// The decoded State.
    pub state: State,
    /// The Session's rules; the auto-repeat delays are the record's.
    pub rules: Rules,
}

impl Seed {
    /// The seed a seed's record gives: the record of the cut its state was saved after, one
    /// frame before the row's first.
    ///
    /// # Errors
    ///
    /// A [`DecodeError`] when the record doesn't decode or lacks the auto-repeat words.
    pub fn decode(record: &FrameRecord, rooms: &RoomMap) -> Result<Self, DecodeError> {
        let state = crate::decode(record, rooms)?.state;
        let rules = Rules {
            switches: Switches::REFERENCE,
            auto_repeat: auto_repeat(record)?,
        };
        Ok(Self { state, rules })
    }
}

/// Steps `row` freely from `seed`, giving `visit` each frame's State and cut.
///
/// The run has the row's frames, `pads` (one per frame, from frame 0) and the row's
/// `Recorded` timing; `visit` sees the State the Tick ending at a frame left and the cut it
/// ended at. The result is why the run stopped early, if it did.
///
/// The free-run gate folds what it visits; the localiser ([`crate::localise`]) compares it
/// with the oracle's State at the same frame.
pub fn step_freely(
    row: &Row,
    seed: Seed,
    pads: &[Buttons],
    content: &Arc<Content>,
    mut visit: impl FnMut(Frame, &State, Cut),
) -> Option<Stop> {
    let Seed {
        state: start,
        rules,
    } = seed;
    let timing = recorded(row);
    match Session::open(start, rules, Arc::clone(content)) {
        Ok(mut session) => (0..row.frames.0).find_map(|index| {
            let frame = Frame(index);
            let pad = usize::try_from(index)
                .ok()
                .and_then(|index| pads.get(index))
                .copied();
            let Some(pad) = pad else {
                return Some(Stop::NoPad(frame));
            };
            let nmi_timing = timing.at(Tick(index));
            match session.step(TickInput {
                pad,
                timing: nmi_timing,
            }) {
                Ok(output) => {
                    visit(frame, session.state(), cut(nmi_timing, output.nmi));
                    None
                }
                Err(fault) => Some(Stop::Fault { frame, fault }),
            }
        }),
        Err(error) => Some(Stop::Open(error)),
    }
}

/// Runs `row` from `seed` for its frames with `pads` (one per frame, from frame 0) and
/// the row's `Recorded` timing, folds the engine's outputs and compares them with the lock.
#[must_use]
pub fn free_run(row: &Row, seed: Seed, pads: &[Buttons], content: &Arc<Content>) -> FreeRun {
    let mut fold = EngineFold::default();
    let stop = step_freely(row, seed, pads, content, |_, state, cut| {
        fold_state(&mut fold, state, cut);
    });
    let frames = FrameCount(fold.frames());
    FreeRun {
        report: row.compare(&fold.finish()),
        frames,
        stop,
    }
}

/// The ratchet of green (row, stream) pairs (D22): once a pair is green it stays green.
///
/// Its text is one `ROW STREAM` line per pair, sorted, with `#` comments; the file sits
/// beside the lock it was measured against (`scenarios/locks/lsnes.green.txt`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ratchet {
    /// The green pairs.
    pub pairs: BTreeSet<(RowId, Stream)>,
}

/// A ratchet line that doesn't read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RatchetError {
    /// The 1-based line.
    pub line: usize,
}

impl fmt::Display for RatchetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: not `ROW STREAM`", self.line)
    }
}

impl std::error::Error for RatchetError {}

/// The ratchet file's head comment.
const RATCHET_HEAD: &str = "\
# The free-run gate's green (row, stream) pairs (ARCHITECTURE D22), against the lock beside
# this file. `tools gate` fails when a listed pair isn't green and adds new ones with
# `--update`; never remove a line by hand.
";

impl Ratchet {
    /// Reads a ratchet file's text.
    ///
    /// # Errors
    ///
    /// A [`RatchetError`] for a line that isn't a row id and a stream name.
    pub fn parse(text: &str) -> Result<Self, RatchetError> {
        let pairs = text
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty() && !line.starts_with('#'))
            .map(|(number, line)| {
                let error = RatchetError {
                    line: number.saturating_add(1),
                };
                let mut words = line.split_whitespace();
                let (Some(row), Some(stream), None) = (words.next(), words.next(), words.next())
                else {
                    return Err(error);
                };
                let stream = Stream::try_from(stream).map_err(|_| error)?;
                Ok((RowId(row.to_owned()), stream))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { pairs })
    }

    /// The listed pairs of the reported rows that aren't green: the gate's failures.
    #[must_use]
    pub fn regressions(&self, reports: &[RowReport]) -> Vec<(RowId, Stream)> {
        reports
            .iter()
            .flat_map(|report| {
                self.pairs
                    .iter()
                    .filter(move |(row, _)| *row == report.id)
                    .filter(move |(_, stream)| {
                        !report
                            .outcome(*stream)
                            .is_some_and(|outcome| outcome == crate::lock::Outcome::Equal)
                    })
                    .cloned()
            })
            .collect()
    }

    /// The reported green pairs this ratchet doesn't list yet.
    #[must_use]
    pub fn new_pairs(&self, reports: &[RowReport]) -> Vec<(RowId, Stream)> {
        reports
            .iter()
            .flat_map(|report| {
                report
                    .green_streams()
                    .map(|stream| (report.id.clone(), stream))
            })
            .filter(|pair| !self.pairs.contains(pair))
            .collect()
    }

    /// This ratchet grown by every green pair of `reports`.
    #[must_use]
    pub fn grown(&self, reports: &[RowReport]) -> Self {
        let mut pairs = self.pairs.clone();
        pairs.extend(self.new_pairs(reports));
        Self { pairs }
    }
}

impl fmt::Display for Ratchet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(RATCHET_HEAD)?;
        self.pairs
            .iter()
            .try_for_each(|(row, stream)| writeln!(f, "{row} {stream}"))
    }
}

#[cfg(test)]
mod tests {
    use super::{Ratchet, cut};
    use crate::lock::{Outcome, RowId, RowReport, SpanIndex, Stream, StreamReport};
    use data::span::Cut;
    use game::nmi::NmiKind;
    use game::state::NmiTiming;

    fn report(row: &str, outcomes: &[(Stream, Outcome)]) -> RowReport {
        RowReport {
            id: RowId(row.to_owned()),
            streams: outcomes
                .iter()
                .map(|&(stream, outcome)| StreamReport { stream, outcome })
                .collect(),
            facts: Vec::new(),
        }
    }

    #[test]
    fn a_ratchet_reads_what_it_writes() {
        let text = "# head\n\nrow-b timing\nrow-a apu\n";
        let ratchet = Ratchet::parse(text).unwrap_or_default();
        assert_eq!(ratchet.pairs.len(), 2);
        let written = ratchet.to_string();
        assert!(written.ends_with("row-a apu\nrow-b timing\n"));
        assert_eq!(Ratchet::parse(&written), Ok(ratchet));
        assert_eq!(
            Ratchet::parse("row-a apu extra\n").map_err(|error| error.line),
            Err(1)
        );
        assert!(Ratchet::parse("row-a nonsense\n").is_err());
    }

    #[test]
    fn a_green_pair_turning_red_is_a_regression() {
        let ratchet = Ratchet::parse("row-a apu\nrow-a timing\nrow-b apu\n").unwrap_or_default();
        let reports = [report(
            "row-a",
            &[
                (Stream::Apu, Outcome::Equal),
                (Stream::Timing, Outcome::Differs { span: SpanIndex(0) }),
                (Stream::Vram, Outcome::Equal),
            ],
        )];
        // row-b wasn't run, so its pair isn't checked.
        assert_eq!(
            ratchet.regressions(&reports),
            [(RowId("row-a".to_owned()), Stream::Timing)]
        );
        assert_eq!(
            ratchet.new_pairs(&reports),
            [(RowId("row-a".to_owned()), Stream::Vram)]
        );
        assert_eq!(ratchet.grown(&reports).pairs.len(), 4);
    }

    #[test]
    fn the_cut_is_the_timing_sources_not_the_handlers_path() {
        assert_eq!(cut(NmiTiming::OnTime, NmiKind::Lag), Cut::Requested);
        assert_eq!(cut(NmiTiming::OnTime, NmiKind::Committed), Cut::Requested);
        assert_eq!(cut(NmiTiming::Overrun, NmiKind::Lag), Cut::Overrun);
        assert_eq!(
            cut(NmiTiming::Disabled, NmiKind::Disabled),
            Cut::VblankWithoutNmi
        );
    }
}
