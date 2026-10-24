//! Room-entry state; compatibility projection owns 82 bytes, not a packed image.
#![forbid(unsafe_code)]
use super::content::{AssetRef, DoorList};
use super::world::{ExitId, Identity, RoomId, StateId};

/// Installed under the existing room owner. Area and geometry stay there.
/// Identity keys and descriptor bytes are projected through the pinned world;
/// no mutable key mirror or retained content capture belongs in this state.
/// The *_key resource words are compatibility-only, never duplicate ResourceIds.
/// Retained effects resolve each CURRENT word by kind through the pinned binding;
/// they must not reconstruct a value from the selected State and ignore alias writes.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RoomEntry {
    pub(crate) exit: Identity<ExitId>,   // 078D, two projected bytes.
    pub(crate) direction: u16,           // 0791
    pub(crate) elevator_properties: u16, // 0793
    pub(crate) elevator_interlock: u16,  // 0795
    pub(crate) enemy_interlock: u16,     // 0797
    pub(crate) elevator_direction: u16,  // 0799
    pub(crate) room: Identity<RoomId>,   // 079B, two projected bytes.
    pub(crate) room_index: u16,          // 079D
    pub(crate) map_x: u16,               // 07A1; low-byte writes retain the high byte.
    pub(crate) map_y: u16,               // 07A3; low-byte writes retain the high byte.
    pub(crate) scroll_up: u16,           // 07AD
    pub(crate) scroll_down: u16,         // 07AF
    pub(crate) previous_cre: u16,        // 07B1
    pub(crate) cre: u16,                 // 07B3
    pub(crate) doors: DoorList,          // 07B5, two projected bytes.
    pub(crate) state: Identity<StateId>, // 07BB, two projected bytes.
    // Each asset projects exactly three bytes. A native four-byte observation
    // must obtain its fourth byte from the live neighboring owner.
    pub(crate) level: AssetRef,           // 07BD..07BF
    pub(crate) tile_table: AssetRef,      // 07C0..07C2
    pub(crate) tiles: AssetRef,           // 07C3..07C5
    pub(crate) palette: AssetRef,         // 07C6..07C8
    pub(crate) music_track: u16,          // 07C9
    pub(crate) music_data: u16,           // 07CB
    pub(crate) fx_key: u16,               // 07CD
    pub(crate) enemy_population_key: u16, // 07CF
    pub(crate) enemy_tilesets_key: u16,   // 07D1
    pub(crate) main_code_key: u16,        // 07DF
    pub(crate) scrolling_hook: u16,       // 07E9
    pub(crate) layer2_x: u8,              // 091B; independently writable policy byte.
    pub(crate) layer2_y: u8,              // 091C; independently writable policy byte.
    pub(crate) frame: u16,                // 0925
    pub(crate) arrival_x: u16,            // 0927
    pub(crate) arrival_y: u16,            // 0929
    pub(crate) speed_fraction: u16,       // 092B
    pub(crate) speed_whole: u16,          // 092D
    pub(crate) elevator_delay: u16,       // 092F
    pub(crate) completion: u16,           // 0931
    pub(crate) transition_selector: u16,  // 099C; unknown selectors remain lossless.
    pub(crate) elevator_flags: u16,       // 0E16; shared with retained actor/player calls.
    pub(crate) elevator_status: u16,      // 0E18; shared with retained actor/player calls.
    pub(crate) irq_policy: u16,           // 00A9; selected room policy, not IRQ scheduler state.
}

/// Aggregate Session compatibility state, outside RoomEntry.
/// Callers capture local control views at the original read point. Import must
/// retain all 256 values; publication after effects follows each caller's order.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SharedContinuations {
    pub(crate) primary: u8,   // 0612
    pub(crate) secondary: u8, // 0613
}
