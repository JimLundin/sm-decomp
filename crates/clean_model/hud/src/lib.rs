//! The HUD model, built from its behaviour spec alone: the status bar at the top of the screen,
//! the selected item and the mini-map.
//!
//! The HUD is a view of what the game holds, drawn into a grid of 32 × 3 cells. Each frame,
//! `HudState::frame` takes the rules, the HUD's pictures (content the importer supplies), what
//! other domains report (her energy, ammunition and reserve, the items she has on, the buttons,
//! where she is on her map) and the HUD's state, and returns the state after the frame (its grid
//! is the picture drawn) with the `HudEvent`s it reports: `Switched` for the weapons and her
//! look, `Click` for the sound domain. Outside a frame, `HudState::boss_encountered` is a boss
//! met on entering its room, which blanks the mini-map and turns it off.
//!
//! The HUD's cells, her map's pictures and the sets of her map's cells are each a `Grid`, read
//! and written at a `Place`, column first; a place outside a grid has no cell. A picture with
//! its flips is the shared `Look`; a HUD cell, a `Tile`, is a look in one of its colour sets.
//! The content a caller lends, `HudPictures` and `HerMap`, is always supplied by the importer;
//! each one's `Default` is an empty stand-in for a caller with nothing imported yet, which no
//! rule reads.
//!
//! The values the rules read are in `Rules`; `Rules::reference` holds the spec's. `CATALOGUE`
//! lists one behaviour per spec entry.
#![expect(
    clippy::doc_markdown,
    reason = "docs name a spec entry as the spec writes it, Spec: Name, keeping backticks for Rust items"
)]

mod catalogue;
mod events;
mod frame;
mod grid;
mod inputs;
mod item;
mod layout;
mod map;
mod mini_map;
mod pictures;
mod rules;
mod selection;
mod state;
mod status_bar;
mod tile;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use clean_types::{Checked, Flip, Kind, Look, Pair, Picture, Screen, Switch};
pub use events::HudEvent;
pub use frame::FrameOutcome;
pub use grid::{Grid, Place};
pub use inputs::{
    Ammunition, CancelButton, Controls, Energy, Equipped, FrameCount, FrameInputs, MiniMapInputs,
    Mute, Reserve, ReserveMode, Room, SelectButton, Status, Stock,
};
pub use item::{Ammo, SelectableItem};
pub use map::{HerMap, MapCellSet, MapPictures};
pub use pictures::{HudPictures, Icons};
pub use rules::{Alternation, Boss, RoomBoss, Rules};
pub use state::{HudState, MiniMapPower, Selection, Shown};
pub use tile::{ColourSet, HudGrid, Tile};

#[cfg(test)]
mod tests {
    use enum_map::{Enum, EnumMap};

    /// Whether `rows`, a table keyed by the enum `K`, names each member of `K` exactly once.
    pub fn names_each_member_once<K: Enum, R>(rows: &[R], member: fn(&R) -> K) -> bool {
        let mut named: EnumMap<K, u8> = EnumMap::default();
        for row in rows {
            named[member(row)] = named[member(row)].saturating_add(1);
        }
        named.values().all(|&times| times == 1)
    }

    #[test]
    fn a_table_missing_a_member_is_refused() {
        #[derive(Clone, Copy, Enum)]
        enum Two {
            A,
            B,
        }
        assert!(names_each_member_once::<Two, Two>(
            &[Two::A, Two::B],
            |&k| k
        ));
        assert!(!names_each_member_once::<Two, Two>(
            &[Two::A, Two::A],
            |&k| k
        ));
    }
}
