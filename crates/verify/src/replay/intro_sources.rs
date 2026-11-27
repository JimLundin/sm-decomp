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
enum Room {
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
}

impl Join {
    pub(super) fn valid(&self, case: &Case) -> bool {
        let Start::CacheCut { frame } = case.start else {
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
        };
        case.inputs == provider
            && !segments.is_empty()
            && segments.iter().all(|s| !s.row.is_empty())
            && segments
                .iter()
                .try_fold(frame, |cut, s| cut.checked_add(s.len()?))
                == Some(case.end.cut_inclusive)
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
    let root = sources
        .lock
        .ancestors()
        .nth(3)
        .ok_or_else(|| data_failure(None, "intro lock path"))?;
    let mut rows = Rows::open(lock, root, &sources.cache, &case.row)?;
    let (row, local) = rows.place(frame)?;
    if row != case.row || local != frame {
        return Err(data_failure(Some(frame), "intro seed row coordinate"));
    }
    let seed = rows.record(frame)?;
    Ok((rows, seed))
}
