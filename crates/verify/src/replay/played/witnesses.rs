//! Native writer attribution shared by ordinary play and the indexed providers.

pub(super) mod shaft;

use crate::replay::{data_failure, result::Outcome};
use game::state::video::OAM_SPRITES;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::replay) struct Witnesses {
    room_video: Option<super::room_video::RoomVideoWitnesses>,
    shaft: Option<shaft::Shaft>,
    front_end: Option<FrontEnd>,
    samus_body: Vec<BodyTrace>,
    pub(super) ribs: Vec<RowWindow>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::replay) struct RowWindow {
    pub(in crate::replay::played) row: String,
    pub(in crate::replay::played) first: u64,
    pub(in crate::replay::played) last: u64,
}

impl RowWindow {
    pub(in crate::replay) fn covers(&self, row: &str, first: u64, last: u64) -> bool {
        self.row == row && self.first <= first && first <= last && last <= self.last
    }

    pub(super) fn contains(&self, (row, cut): (&str, u64)) -> bool {
        self.covers(row, cut, cut)
    }

    fn overlaps(&self, other: &Self) -> bool {
        self.row == other.row && self.first.max(other.first) <= self.last.min(other.last)
    }

    pub(super) const fn valid(&self) -> bool {
        !self.row.is_empty() && self.first <= self.last
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FrontEnd {
    opening_hud: RowWindow,
    extra_finished: std::collections::BTreeMap<String, Vec<u64>>,
}

impl FrontEnd {
    fn valid(&self) -> bool {
        self.opening_hud.valid()
            && self.extra_finished.iter().all(|(row, cuts)| {
                !row.is_empty()
                    && cuts.first().is_some_and(|cut| *cut > 0)
                    && cuts.is_sorted_by(|first, second| first < second)
            })
    }

    pub(super) fn extra_finished(&self, row: &str) -> &[u64] {
        self.extra_finished.get(row).map_or(&[], Vec::as_slice)
    }

    pub(super) fn hud_publication(&self, (row, cut): (&str, u64)) -> Option<bool> {
        (row == self.opening_hud.row).then_some(cut == self.opening_hud.first)
    }

    pub(super) fn hud_visible(&self, place: (&str, u64)) -> bool {
        self.opening_hud.contains(place)
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
    pub(in crate::replay) fn resolve(declared: Option<Self>) -> Result<Self, &'static str> {
        declared
            .filter(Self::valid)
            .ok_or("missing or invalid played witnesses")
    }

    pub(in crate::replay) fn valid(&self) -> bool {
        self.room_video
            .as_ref()
            .is_some_and(super::room_video::RoomVideoWitnesses::valid)
            && self.shaft.as_ref().is_some_and(shaft::Shaft::valid)
            && self.front_end.as_ref().is_some_and(FrontEnd::valid)
            && self.ribs.iter().all(RowWindow::valid)
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

    pub(super) fn room_video(&self) -> Result<&super::room_video::RoomVideoWitnesses, Outcome> {
        self.room_video
            .as_ref()
            .ok_or_else(|| data_failure(None, "room video witnesses"))
    }

    pub(in crate::replay) fn shaft(&self) -> Result<&shaft::Shaft, Outcome> {
        self.shaft
            .as_ref()
            .ok_or_else(|| data_failure(None, "shaft witnesses"))
    }

    pub(super) fn front_end(&self) -> Result<&FrontEnd, Outcome> {
        self.front_end
            .as_ref()
            .ok_or_else(|| data_failure(None, "front end witnesses"))
    }

    pub(super) fn body(&self, row: &str, first: u64, last: u64) -> Option<&BodyTrace> {
        self.samus_body
            .iter()
            .find(|trace| trace.row == row && trace.counts(first, last).is_some())
    }
}

#[derive(Deserialize)]
struct Catalogue {
    format: u16,
    played_witnesses: Option<Witnesses>,
}

fn parse(text: &str) -> Result<Witnesses, Outcome> {
    let bad = || data_failure(None, "played witness catalogue");
    let index: Catalogue = toml::from_str(text).map_err(|_| bad())?;
    if index.format != crate::replay::index::CURRENT_FORMAT {
        return Err(bad());
    }
    Witnesses::resolve(index.played_witnesses).map_err(|_| bad())
}

pub(super) fn read(root: &std::path::Path) -> Result<Witnesses, Outcome> {
    let text = std::fs::read_to_string(root.join("scenarios/replay/index.toml"))
        .map_err(|_| data_failure(None, "played witness catalogue"))?;
    parse(&text)
}

#[cfg(test)]
pub(super) fn indexed() -> Result<&'static Witnesses, Outcome> {
    static INDEXED: std::sync::LazyLock<Result<Witnesses, Outcome>> =
        std::sync::LazyLock::new(|| {
            parse(include_str!("../../../../../scenarios/replay/index.toml"))
        });
    INDEXED.as_ref().map_err(Clone::clone)
}

pub(in crate::replay) fn selected(
    case: &crate::replay::index::Case,
) -> Result<&Witnesses, Outcome> {
    case.played_witnesses
        .as_ref()
        .ok_or_else(|| data_failure(None, "case played witnesses"))
}

#[cfg(test)]
mod tests {
    use crate::replay::index::Index;

    const INDEX: &str = include_str!("../../../../../scenarios/replay/index.toml");

    #[test]
    fn indexes_declare_every_played_witness_table() {
        let mut document: toml::Table = toml::from_str(INDEX).unwrap();
        document["case"]
            .as_array_mut()
            .unwrap()
            .retain(|case| case["id"].as_str() == Some("ridley/ribs-from-opening"));
        for omitted in ["played_witnesses", "room_video", "shaft", "front_end"] {
            let mut partial = document.clone();
            if omitted == "played_witnesses" {
                partial.remove(omitted).unwrap();
            } else {
                partial["played_witnesses"]
                    .as_table_mut()
                    .unwrap()
                    .remove(omitted)
                    .unwrap();
            }
            let text = toml::to_string(&partial).unwrap();
            assert!(Index::parse(&text).is_err(), "{omitted}");
            assert!(super::parse(&text).is_err(), "{omitted}");
        }

        let mut local = document["played_witnesses"].clone();
        local["room_video"]["overlay_pieces"] = 10.into();
        document["case"][0]
            .as_table_mut()
            .unwrap()
            .insert("played_witnesses".into(), local);
        let index = Index::parse(&toml::to_string(&document).unwrap()).unwrap();
        let resolved = super::selected(&index.cases[0]).unwrap();
        assert_eq!(resolved.room_video().unwrap().overlay_pieces, 10);

        document["format"] = 1.into();
        assert!(Index::parse(&toml::to_string(&document).unwrap()).is_err());
        assert!(super::parse(&toml::to_string(&document).unwrap()).is_err());
    }

    #[test]
    fn unrelated_current_recipes_can_omit_the_shared_played_witnesses() {
        let mut index = Index::parse(INDEX).unwrap();
        index
            .cases
            .retain(|case| case.id.0 == "audio/native-control");
        assert!(Index::parse(&toml::to_string(&index).unwrap()).is_ok());
    }
}
