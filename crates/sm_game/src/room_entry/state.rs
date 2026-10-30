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
pub struct RoomEntry {
    pub exit: Identity<ExitId>,   // 078D, two projected bytes.
    pub direction: u16,           // 0791
    pub elevator_properties: u16, // 0793
    pub elevator_interlock: u16,  // 0795
    pub enemy_interlock: u16,     // 0797
    pub elevator_direction: u16,  // 0799
    pub room: Identity<RoomId>,   // 079B, two projected bytes.
    pub room_index: u16,          // 079D
    pub map_x: u16,               // 07A1; low-byte writes retain the high byte.
    pub map_y: u16,               // 07A3; low-byte writes retain the high byte.
    pub scroll_up: u16,           // 07AD
    pub scroll_down: u16,         // 07AF
    pub previous_cre: u16,        // 07B1
    pub cre: u16,                 // 07B3
    pub doors: DoorList,          // 07B5, two projected bytes.
    pub state: Identity<StateId>, // 07BB, two projected bytes.
    // Each asset projects exactly three bytes. A native four-byte observation
    // must obtain its fourth byte from the live neighboring owner.
    pub level: AssetRef,           // 07BD..07BF
    pub tile_table: AssetRef,      // 07C0..07C2
    pub tiles: AssetRef,           // 07C3..07C5
    pub palette: AssetRef,         // 07C6..07C8
    pub music_track: u16,          // 07C9
    pub music_data: u16,           // 07CB
    pub fx_key: u16,               // 07CD
    pub enemy_population_key: u16, // 07CF
    pub enemy_tilesets_key: u16,   // 07D1
    pub main_code_key: u16,        // 07DF
    pub scrolling_hook: u16,       // 07E9
    pub layer2_x: u8,              // 091B; independently writable policy byte.
    pub layer2_y: u8,              // 091C; independently writable policy byte.
    pub frame: u16,                // 0925
    pub arrival_x: u16,            // 0927
    pub arrival_y: u16,            // 0929
    pub speed_fraction: u16,       // 092B
    pub speed_whole: u16,          // 092D
    pub elevator_delay: u16,       // 092F
    pub completion: u16,           // 0931
    pub transition_selector: u16,  // 099C; unknown selectors remain lossless.
    pub elevator_flags: u16,       // 0E16; shared with retained actor/player calls.
    pub elevator_status: u16,      // 0E18; shared with retained actor/player calls.
    pub irq_policy: u16,           // 00A9; selected room policy, not IRQ scheduler state.
}

/// Aggregate Session compatibility state, outside RoomEntry.
/// Callers capture local control views at the original read point. Import must
/// retain all 256 values; publication after effects follows each caller's order.
#[derive(Clone, Debug, PartialEq)]
pub struct SharedContinuations {
    pub primary: u8,   // 0612
    pub secondary: u8, // 0613
}
