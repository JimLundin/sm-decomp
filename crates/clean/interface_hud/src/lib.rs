//! The HUD model, built from its behaviour spec alone: the status bar at the top of the screen,
//! the selected item and the mini-map.
//!
//! The HUD is a view of what the game holds, drawn into a grid of 32 × 3 cells. Its state,
//! `HudState`, is changed in place by each step of the running game; its grid is the picture
//! drawn. Which step a frame is, the main loop chooses, calling one of three entries:
//!
//! - `HudState::frame_of_play`, with the rules, the HUD's pictures (content the importer
//!   supplies) and what other domains report (her energy, ammunition and reserve, the items she
//!   has on, the buttons, the selection dropped before the status bar and after it, where she is
//!   on her map). It returns the `HudEvent`s it reports: `HudEvent::Switched { item }` for
//!   the weapons and her look (`item: None` still reports a switch), `HudEvent::Click` for
//!   the sound domain. Events use ordinary externally tagged derived Serde; the named optional
//!   payload keeps a switch to no item distinct from a click through TOML restoration.
//! - `HudState::out_of_play`: a step between frames of play, where only the mini-map's turn-on,
//!   a new area's explored cells and a drop of the selection change anything.
//! - `HudState::boss_blanks_the_map`: a boss met on entering its room, which blanks the
//!   mini-map and turns it off.
//!
//! The HUD's cells, her map's pictures and the sets of her map's cells are each a `Grid`, read
//! and written at a `Place`, column first; a place outside a grid has no cell. A picture with
//! its flips is the shared `Look`; a HUD cell, a `Tile`, is a look in one of its colour sets.
//! The content a caller lends, `HudPictures` and `HerMap`, is always supplied by the importer;
//! each one's `Default` is an empty stand-in for a caller with nothing imported yet. No
//! meaningful drawing is defined for unimported content. Caches and visible cells are
//! independent: neither is reconstructed from the other. `Click` is a sound request, not a
//! promise that a sound is heard.
//!
//! For staged frames, call `HudWork::begin` with the full State and frame-start count. Choose
//! and apply controls, movement mini-map and late status in that order. Inspect each chosen
//! answer before applying it. After movement application the map is already visible on the
//! carried HUD; only after status does `publication` give the complete HUD and event history.
//! Save the entire `HudWork` and any caller-owned `EventCursor` values to restore and continue.
//!
//! Spec: EscapeCountdown, CountdownContinuation. `HudState::countdown` owns the admitted
//! `Countdown::Dormant` value. A distinct `CountdownWork` receives the producer's qualified
//! `CountdownOpportunity`, exposes its chosen expiry/picture answer, and completes it once.
//! Ordinary HUD refresh never schedules countdown maintenance. Whole-work serde restoration
//! retains the full HUD and chosen/completed progress; scene qualification and active escape
//! countdowns remain unsupported neighboring responsibilities.
//!
//! The values the rules read, every Parameter the spec lists, are in `Rules`: `Rules::reference`,
//! its default, holds the spec's. `CATALOGUE` lists one behaviour per spec entry.
#![expect(
    clippy::doc_markdown,
    reason = "docs name a spec entry as the spec writes it, Spec: Name, keeping backticks for Rust items"
)]

mod catalogue;
mod countdown;
mod countdown_work;
mod events;
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
mod step;
mod tile;
mod work;

pub use work::{
    ControlAnswer, DisplayAnswer, DisplayInputs, EventCursor, HudOpportunity, HudWork, MapAnswer,
    MovementInputs, Observation, StatusAnswer,
};

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use countdown::{
    Countdown, CountdownAnswer, CountdownExpiry, CountdownOpportunity, CountdownPicture,
};
pub use countdown_work::CountdownWork;
pub use events::HudEvent;
pub use grid::{Grid, Place};
pub use inputs::{
    Ammunition, CancelButton, Controls, Dropped, Energy, Equipped, FrameCount, FrameInputs,
    ItemControlInputs, MiniMapInputs, Mute, NewArea, OutOfPlayInputs, Reserve, ReserveMode, Room,
    SelectButton, Status, StatusInputs, Stock,
};
pub use item::{Ammo, SelectableItem};
pub use map::{HerMap, MapCellSet, MapPictures};
pub use pictures::{HudPictures, Icons};
pub use rules::{Alternation, Boss, Rules};
pub use state::{HudState, MiniMapPower, Selection, Shown};
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
