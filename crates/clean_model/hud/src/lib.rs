//! The HUD model, built from its behaviour spec alone: the status bar at the top of the screen,
//! the selected item and the mini-map.
//!
//! The HUD is a view of what the game holds, drawn into a grid of 32 × 3 tiles. Each frame,
//! `HudState::frame` takes what other domains report (her energy, ammunition and reserve, the
//! items she has on, the buttons, where she is on her map) and the HUD's state, and returns the
//! state after the frame (its grid is the picture drawn) with the events it reports: `Switched`
//! for the weapons and her look, `Click` for the sound domain. Outside a frame,
//! `HudState::boss_turns_off_mini_map` is a boss turning the mini-map off on entering its room.
//!
//! The HUD's tiles, her map's pictures and the sets of her map's cells are each a `Grid`, its
//! positions `Pos`, across then down.
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
mod map;
mod mini_map;
mod pictures;
mod rules;
mod selection;
mod state;
mod status_bar;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use clean_types::{Checked, Kind, Pair, Switch};
pub use events::Event;
pub use frame::FrameOutcome;
pub use grid::{Grid, Pos};
pub use inputs::{
    AMMO_KINDS, AmmoKind, Ammunition, CancelButton, Controls, Energy, Equipped, FrameCount,
    FrameInputs, MiniMapInputs, Mute, Reserve, ReserveMode, Room, SelectButton, Status, Stock,
};
pub use item::SelectableItem;
pub use map::{HerMap, MapCell, MapCellSet, MapPicture, PicturePos};
pub use pictures::{ColourSet, Flip, HudGrid, HudPos, Picture, Tile};
pub use rules::{Alternation, Boss, Rules, Screen};
pub use state::{HudState, Selection, Shown};
