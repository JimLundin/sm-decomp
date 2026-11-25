//! The numbers a caller may adjust: an adjusted `Rules` is an adjusted game.

use types::file_select::{HelmetTurn, MenuPace};

/// File select's adjustable numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Spec: `MenuBrightens`, `MenuDims`. Finished frames per level of the menu's fades.
    pub menu_pace: MenuPace,
    /// Spec: `HelmetTurns`. Frames of work the chosen file's helmet turns.
    pub helmet_turn: HelmetTurn,
}

impl Rules {
    /// The reference game: a level every 2 finished frames, a helmet turn of 57 frames of work.
    pub const REFERENCE: Self = Self {
        menu_pace: MenuPace::REFERENCE,
        helmet_turn: HelmetTurn::REFERENCE,
    };
}
