#![cfg(not(doctest))]
//! The HUD model, built from its behaviour spec alone: the status bar at the top of the screen,
//! the selected item and the mini-map.
//!
//! The HUD is a view of what the game holds, drawn into a grid of 32 × 3 cells. Its state,
//! `HudState`, is changed in place by each step of the running game; its grid is the picture
//! drawn. A new file's State comes from room setup; then which step a frame is, the main loop
//! chooses, calling one of four entries:
//!
//! - `HudState::new_file`, room setup for a new file at Ceres, before her first frame of play:
//!   from the importer's `OpeningFrame` and the prepared room's `OpeningPreview`, with the frame
//!   count at setup and her fresh energy, it gives the `NewFileHud`: the upper row for the video
//!   encoder and the State the fade and play carry on from.
//! - `HudState::frame_of_play`, one whole frame of play: with the rules, the HUD's pictures
//!   (content the importer supplies) and the frame's `FrameInputs` (her stock as the frame
//!   begins and the buttons, her position after her moves, the values as the frame ends, the
//!   drops, and the escape countdown's `CountdownInputs`: whether the scene runs it or only
//!   shows it, its start, the count of finished frames), it leaves the State as the frame ends
//!   and returns the `FrameOutputs`: the selection after the controls, the `HudEvent`s in the
//!   order raised (`HudEvent::Switched { item }` for the weapons and her look, `HudEvent::Click`
//!   for the sound domain) and the countdown's answer, its expiry and its picture.
//! - `HudState::out_of_play`: a step between frames of play, where only the mini-map's turn-on,
//!   a new area's explored cells and a drop of the selection change anything.
//! - `HudState::boss_blanks_the_map`: a boss met on entering its room, which blanks the
//!   mini-map and turns it off.
//!
//! A caller that owns the escape countdown on its own steps it with `Countdown::play`, the same
//! turn `HudState::frame_of_play` gives it: from the frame's `CountdownInputs` it returns the
//! countdown's answer, its expiry and its picture, apart from the rest of the HUD.
//!
//! The HUD's cells, her map's pictures and the sets of her map's cells are each a `Grid`, read
//! and written at a `Place`, column first; a place outside a grid has no cell. A picture with
//! its flips is the shared `Look`; a HUD cell, a `Tile`, is a look in one of its colour sets.
//! The content a caller lends, `HudPictures` and `HerMap`, is always supplied by the importer;
//! each one's `Default` is an empty stand-in for a caller with nothing imported yet. The values
//! last shown and the cells are independent: neither is worked out from the other. `Click` is a
//! sound request, not a promise that a sound is heard.
//!
//! The values the rules read, every Parameter the spec lists, are in `Rules`: `Rules::reference`,
//! its default, holds the spec's. `CATALOGUE` lists one behaviour per spec entry.
#![expect(
    clippy::doc_markdown,
    reason = "docs name a spec entry as the spec writes it, Spec: Name, keeping backticks for Rust items"
)]

mod catalogue;
mod countdown;
mod events;
mod grid;
mod inputs;
mod item;
mod layout;
mod map;
mod mini_map;
mod new_file;
mod pictures;
mod rules;
mod selection;
mod state;
mod status_bar;
mod step;
mod tile;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use countdown::{
    BegunCountdown, Countdown, CountdownAge, CountdownAnswer, CountdownExpiry, CountdownInputs,
    CountdownPicture, CountdownPlace, CountdownRuns, FinishedFrames, StartTime, TimeLeft,
    UnsupportedFrame,
};
pub use events::HudEvent;
pub use grid::{Grid, Place};
pub use inputs::{
    Ammunition, CancelButton, Controls, Dropped, Energy, Equipped, FrameCount, FrameInputs,
    MiniMapInputs, Mute, NewArea, OutOfPlayInputs, Reserve, ReserveMode, Room, SelectButton,
    Status, Stock, StockAtStart,
};
pub use item::{Ammo, SelectableItem};
pub use map::{HerMap, MapCellSet, MapPictures};
pub use mini_map::{OpeningPreview, PreviewCell};
pub use new_file::{NewFileHud, OpeningFrame, UpperRow};
pub use pictures::{HudPictures, Icons};
pub use rules::{Alternation, Boss, Rules};
pub use state::{HudState, MiniMapPower, Selection, Shown};
pub use step::FrameOutputs;
pub use tile::{ColourSet, HudGrid, Tile};
pub use types::{Checked, Flip, Kind, Look, Pair, Picture, Screen, Switch};

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
