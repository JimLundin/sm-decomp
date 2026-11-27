//! Recorded successors of an intro seed, in the movie's continuous coordinates.

use super::{
    Sources, data_failure,
    index::{Case, InputProvider, Start},
    played::rows::Rows,
    result::Outcome,
};
use crate::lock::Lock;
use game::production::played::intro_rooms::Start as Scene;
use oracle_record::FrameRecord;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Segment {
    row: String,
    first: u64,
    last: u64,
}

impl Segment {
    fn len(&self) -> Option<u64> {
        self.last.checked_sub(self.first)?.checked_add(1)
    }

    pub(super) fn qualify(&self, rows: &mut Rows<'_>, first: u64) -> Result<u64, Outcome> {
        let last = self
            .last
            .checked_sub(self.first)
            .and_then(|span| first.checked_add(span))
            .ok_or_else(|| data_failure(Some(first), "intro segment extent"))?;
        for (cut, local) in [(first, self.first), (last, self.last)] {
            let (row, frame) = rows.place(cut)?;
            if row != self.row || frame != local {
                return Err(data_failure(Some(cut), "intro segment movie coordinates"));
            }
        }
        Ok(last)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(remote = "Scene", rename_all = "kebab-case")]
pub(super) enum Room {
    Fight,
    Discovery,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(super) enum Join {
    Room {
        #[serde(with = "Room")]
        scene: Scene,
        records: Segment,
    },
    Playback {
        segments: Vec<Segment>,
    },
    Discovery {
        segments: Vec<Segment>,
    },
    Flashback {
        handoff: RecordCut,
        segments: Vec<Segment>,
    },
}

impl Join {
    pub(super) fn valid(&self, case: &Case) -> bool {
        let Some(first) = case.first_cut() else {
            return false;
        };
        let (provider, segments) = match self {
            Self::Room { records, .. } => {
                if records.len() != Some(2) {
                    return false;
                }
                (
                    InputProvider::IntroRoomJoinInputs,
                    std::slice::from_ref(records),
                )
            }
            Self::Playback { segments } => (
                InputProvider::CinematicActorOpportunities,
                segments.as_slice(),
            ),
            Self::Discovery { segments } => (InputProvider::DiscoveryInputs, segments.as_slice()),
            Self::Flashback { handoff, segments } => {
                if handoff.row.is_empty() || handoff.cut == u64::MAX {
                    return false;
                }
                (InputProvider::FlashbackInputs, segments.as_slice())
            }
        };
        let Some(length) = case
            .end
            .cut_inclusive
            .checked_sub(first)
            .and_then(|span| span.checked_add(1))
        else {
            return false;
        };
        matches!(case.start, Start::CacheCut { .. })
            && case.inputs == provider
            && segments.iter().all(|s| !s.row.is_empty())
            && segments
                .iter()
                .try_fold(0u64, |count, s| count.checked_add(s.len()?))
                == Some(length)
    }
}

pub(super) fn origin<'a>(
    case: &Case,
    sources: &'a Sources,
    lock: &'a Lock,
) -> Result<(Rows<'a>, FrameRecord), Outcome> {
    let Start::CacheCut { frame } = case.start else {
        return Err(data_failure(None, "intro cache origin"));
    };
    source(&case.row, frame, sources, lock)
}

fn source<'a>(
    id: &str,
    frame: u64,
    sources: &'a Sources,
    lock: &'a Lock,
) -> Result<(Rows<'a>, FrameRecord), Outcome> {
    let mut rows = super::played::open_rows(sources, lock, id)?;
    let (row, local) = rows.place(frame)?;
    if row != id || local != frame {
        return Err(data_failure(Some(frame), "intro seed row coordinate"));
    }
    let seed = rows.record(frame)?;
    Ok((rows, seed))
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RecordCut {
    row: String,
    cut: u64,
}

impl RecordCut {
    pub(super) fn before<'a>(
        &self,
        case: &Case,
        first: u64,
        sources: &'a Sources,
        lock: &'a Lock,
    ) -> Result<(Rows<'a>, FrameRecord), Outcome> {
        let (mut rows, handoff) = source(&self.row, self.cut, sources, lock)?;
        let successor = self
            .cut
            .checked_add(1)
            .ok_or_else(|| data_failure(Some(first), "intro handoff successor"))?;
        let (row, frame) = rows.place(successor)?;
        if row != case.row || frame != first {
            return Err(data_failure(Some(first), "intro handoff predecessor"));
        }
        Ok((rows, handoff))
    }
}

/// The case's comparison coordinates may start in the row after its handoff record.
pub(super) fn visit(
    mut rows: Rows<'_>,
    mut previous: FrameRecord,
    segments: &[Segment],
    mut cuts: RangeInclusive<u64>,
    mut compare: impl FnMut(u64, &FrameRecord, &FrameRecord) -> Result<(), Outcome>,
) -> Result<(), Outcome> {
    let mut before = previous.index;
    for segment in segments {
        let start = before
            .checked_add(1)
            .ok_or_else(|| data_failure(Some(before), "intro record successor"))?;
        let last = segment.qualify(&mut rows, start)?;
        for record in start..=last {
            let cut = cuts
                .next()
                .ok_or_else(|| data_failure(None, "intro segment past horizon"))?;
            let current = rows.record(record)?;
            if current.wram.len() != 0x2_0000
                || previous.video_frame.checked_add(1) != Some(current.video_frame)
            {
                return Err(data_failure(Some(cut), "intro consecutive movie records"));
            }
            compare(cut, &previous, &current)?;
            previous = current;
        }
        before = last;
    }
    if let Some(cut) = cuts.next() {
        return Err(data_failure(Some(cut), "incomplete intro horizon"));
    }
    Ok(())
}
