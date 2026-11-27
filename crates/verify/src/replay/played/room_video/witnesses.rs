//! Native room display qualifications and attributed OAM spans.

use super::super::witnesses::RowWindow;
use game::state::video::OAM_SPRITES;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::replay::played) struct RoomVideoWitnesses {
    pub(super) ridley_dma: Vec<RowWindow>,
    pub(super) ridley_reset: RowWindow,
    pub(super) departure_layout: RowWindow,
    pub(super) shaft_transform: RowWindow,
    pub(super) overlay_placement: RowWindow,
    pub(super) shaft_sheet: RowWindow,
    pub(super) bg2_shadow: RowWindow,
    pub(super) platform: Vec<RowWindow>,
    pub(in crate::replay::played) overlay_pieces: u8,
    pictures: Vec<Point>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Point {
    row: String,
    cut: u64,
    picture: Picture,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(in crate::replay::played) enum Picture {
    Prepare,
    Publish,
    FirstEffect { lane: Lane, slot: u8 },
    Overlay { slot: u8 },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::replay::played) enum Lane {
    Prepared,
    Published,
}

impl RoomVideoWitnesses {
    pub(in crate::replay::played) fn picture(&self, (row, cut): (&str, u64)) -> Option<&Picture> {
        self.pictures
            .iter()
            .find(|point| point.row == row && point.cut == cut)
            .map(|point| &point.picture)
    }

    pub(in crate::replay::played) fn valid(&self) -> bool {
        let count = usize::from(self.overlay_pieces);
        let mut positions = std::collections::BTreeSet::new();
        (1..=OAM_SPRITES).contains(&count)
            && !self.ridley_dma.is_empty()
            && !self.platform.is_empty()
            && !self.pictures.is_empty()
            && self
                .ridley_dma
                .iter()
                .chain(&self.platform)
                .chain([
                    &self.ridley_reset,
                    &self.departure_layout,
                    &self.shaft_transform,
                    &self.overlay_placement,
                    &self.shaft_sheet,
                    &self.bg2_shadow,
                ])
                .all(RowWindow::valid)
            && self.pictures.iter().all(|point| {
                !point.row.is_empty()
                    && positions.insert((&point.row, point.cut))
                    && match point.picture {
                        Picture::Prepare | Picture::Publish => true,
                        Picture::FirstEffect { slot, .. } => usize::from(slot) < OAM_SPRITES,
                        Picture::Overlay { slot } => {
                            usize::from(slot).saturating_add(count) <= OAM_SPRITES
                        }
                    }
            })
    }
}
