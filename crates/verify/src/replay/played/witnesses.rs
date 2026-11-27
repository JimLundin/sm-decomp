//! Native writer attribution shared by ordinary play and the indexed providers.

use crate::replay::{data_failure, result::Outcome};
use game::state::video::OAM_SPRITES;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::replay) struct Witnesses {
    samus_body: Vec<BodyTrace>,
    pub(super) ribs: Vec<RowWindow>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RowWindow {
    row: String,
    first: u64,
    last: u64,
}

impl RowWindow {
    pub(super) fn covers(&self, row: &str, first: u64, last: u64) -> bool {
        self.row == row && self.first <= first && first <= last && last <= self.last
    }

    pub(super) fn contains(&self, (row, cut): (&str, u64)) -> bool {
        self.covers(row, cut, cut)
    }

    const fn valid(&self) -> bool {
        !self.row.is_empty() && self.first <= self.last
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BodyTrace {
    row: String,
    first: u64,
    slot: u8,
    pieces: Vec<u8>,
}

impl BodyTrace {
    fn last(&self) -> Option<u64> {
        self.first
            .checked_add(u64::try_from(self.pieces.len().checked_sub(1)?).ok()?)
    }

    fn counts(&self, first: u64, last: u64) -> Option<&[u8]> {
        let offset = |cut: u64| usize::try_from(cut.checked_sub(self.first)?).ok();
        self.pieces.get(offset(first)?..=offset(last)?)
    }

    pub(super) fn slots(&self, cut: u64) -> Option<core::ops::Range<usize>> {
        let count = usize::from(*self.counts(cut, cut)?.first()?);
        let slot = usize::from(self.slot);
        Some(slot..slot.checked_add(count)?)
    }

    pub(super) fn has_draw(&self, first: u64, last: u64) -> bool {
        self.counts(first, last)
            .is_some_and(|counts| counts.iter().any(|&count| count > 0))
    }
}

impl Witnesses {
    pub(in crate::replay) fn valid(&self) -> bool {
        self.ribs.iter().all(RowWindow::valid)
            && self.samus_body.iter().enumerate().all(|(index, trace)| {
                !trace.row.is_empty()
                    && trace.last().is_some()
                    && usize::from(trace.slot) < OAM_SPRITES
                    && trace.pieces.iter().all(|&count| {
                        usize::from(trace.slot).saturating_add(usize::from(count)) <= OAM_SPRITES
                    })
                    && self.samus_body.iter().take(index).all(|other| {
                        other.row != trace.row
                            || other.last().is_some_and(|cut| cut < trace.first)
                            || trace.last().is_some_and(|cut| cut < other.first)
                    })
            })
    }

    pub(super) fn body(&self, row: &str, first: u64, last: u64) -> Option<&BodyTrace> {
        self.samus_body
            .iter()
            .find(|trace| trace.row == row && trace.counts(first, last).is_some())
    }
}

#[derive(Deserialize)]
struct Published {
    played_witnesses: Option<Witnesses>,
}

pub(super) fn read(root: &std::path::Path) -> Result<Witnesses, Outcome> {
    let bad = || data_failure(None, "played witness catalogue");
    let path = root.join("scenarios/replay/index.toml");
    match std::fs::read_to_string(path) {
        Ok(text) => match toml::from_str::<Published>(&text)
            .map_err(|_| bad())?
            .played_witnesses
        {
            Some(data) if data.valid() => Ok(data),
            Some(_) => Err(bad()),
            None => published().cloned(),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => published().cloned(),
        Err(_) => Err(bad()),
    }
}

/// Format-1 indices and standalone lock trees predating the declarations use the reviewed catalogue.
pub(super) fn published() -> Result<&'static Witnesses, Outcome> {
    crate::replay::legacy_packets::played_witnesses()
        .ok_or_else(|| data_failure(None, "historical played witnesses"))
}

pub(in crate::replay) fn selected(
    case: &crate::replay::index::Case,
) -> Result<&Witnesses, Outcome> {
    case.played_witnesses
        .as_ref()
        .or_else(|| crate::replay::legacy_packets::played_witnesses())
        .ok_or_else(|| data_failure(None, "historical played witnesses"))
}
