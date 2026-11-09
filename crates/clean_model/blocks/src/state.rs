//! What the model remembers from frame to frame (spec §1.2): the cells, the places and the
//! screens. A caller builds each part at any value the spec allows and reads each part back.

use clean_types::{Cell, ScreenKind};

use crate::cells::{Grid, Tilemap};
use crate::places::{Place, Places, Touched};

/// The blocks' state: everything a later frame needs, and nothing else.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// The room's cells: their kinds, marks and looks.
    pub tiles: Tilemap,
    /// The places, each free or with what it holds.
    pub places: Places,
    /// The room's screens, which the camera reads.
    pub screens: Grid<ScreenKind>,
}

impl State {
    /// `BlocksMarkedNothing`, `ScrollCells`: a request takes the highest free place until the next
    /// run that isn't paused; with every place taken it takes none. Returns whether it took one.
    pub fn hold_until_run(&mut self) -> bool {
        self.places.take(Place::HeldUntilRun)
    }

    /// `ScrollCells`: a scroll cell touched takes the highest free place until the run, and its
    /// reaction is touched; with every place taken it isn't touched.
    pub fn touch_scroll(&mut self, cell: Cell) {
        if !self.hold_until_run() {
            return;
        }
        if let Some(scroll) = self.places.scroll_at(cell) {
            scroll.touched = Touched::Yes;
        }
    }
}
