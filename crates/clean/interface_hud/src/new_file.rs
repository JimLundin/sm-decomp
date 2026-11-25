//! A new file's HUD (§2): room setup for a new file at Ceres puts up the status bar and the
//! opening map preview while the screen is still black, before her first frame of play, and hands
//! the HUD's State on to the fade and to ordinary play.

use serde::{Deserialize, Serialize};

use crate::countdown::Countdown;
use crate::inputs::{Energy, FrameCount};
use crate::map::MapCellSet;
use crate::mini_map::OpeningPreview;
use crate::pictures::HudPictures;
use crate::rules::Rules;
use crate::state::{HudState, MiniMapPower, Selection, Shown};
use crate::tile::{HudGrid, Tile};

/// The status bar's upper row: one row of 32 cells above the HUD's three, a fixed part of the
/// status bar that the video encoder receives when the HUD goes up (§1.4).
pub type UpperRow = [Tile; 32];

/// The opening frame of the status bar (§1.3, §1.4): content from the importer, each cell with
/// its colour set and flips.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct OpeningFrame {
    /// The upper row, published as it is.
    pub upper_row: UpperRow,
    /// The three rows below it: the HUD's cells before setup draws the gauge and the preview.
    pub lower_rows: HudGrid,
}

/// What a new file's setup publishes: the upper row for the video encoder, and the HUD's State
/// for the fade and the frames of play that follow.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NewFileHud {
    /// The opening frame's upper row.
    pub upper_row: UpperRow,
    /// The HUD's State as setup leaves it.
    pub state: HudState,
}

impl HudState {
    /// Room setup puts up the HUD for a new file at Ceres, from the opening frame, the opening
    /// map preview, the frame count at setup and her fresh energy.
    ///
    /// - Spec: NewFileStatusBar. The upper row is published and the three lower rows become the
    ///   HUD's cells. The countdown is dormant; the selection, the one-shot item and the item
    ///   last shown are none; the ammunition last shown is zero, its cells and the reserve sign's
    ///   keeping their opening pictures. The energy last shown starts at zero and `EnergyGauge`
    ///   draws her fresh energy, so an energy of zero draws nothing.
    /// - Spec: NewFileMiniMap. The window shows the preview, her cell blinking by the frame
    ///   count (`HerBlink`); nothing is explored and the mini-map is on, so each frame of play
    ///   from the next uses the ordinary mini-map rules.
    #[must_use]
    pub fn new_file(
        rules: &Rules,
        pictures: &HudPictures,
        opening: &OpeningFrame,
        preview: &OpeningPreview,
        frame: FrameCount,
        fresh_energy: Energy,
    ) -> NewFileHud {
        let mut state = Self {
            grid: opening.lower_rows,
            shown: Shown::default(),
            selection: Selection::None,
            explored: MapCellSet::default(),
            mini_map: MiniMapPower::On,
            countdown: Countdown::Dormant,
        };
        state.energy_gauge(pictures, fresh_energy);
        state.draw_opening_preview(rules, pictures, preview, frame);
        NewFileHud {
            upper_row: opening.upper_row,
            state,
        }
    }
}
