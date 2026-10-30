//! Pause and file-select menu word tables in banks $81 and $82: area select
//! colours, controller binding tilemaps, equipment icons and the Samus
//! wireframe by suit.

/// $81:A40E: area select foreground colours, read five at a time.
pub const AREA_SELECT_COLOURS: u32 = 0x81_a40e;
/// Up to the area select palette runs at $81:A4E6.
pub const AREA_SELECT_COLOUR_WORDS: usize = (0xa4e6 - 0xa40e) / 2;

/// $82:F639: controller binding tilemap destinations, by row.
pub const BINDING_DESTINATIONS: u32 = 0x82_f639;
/// $82:F647: controller binding button tilemaps, by binding.
pub const BINDING_BUTTON_TILEMAPS: u32 = 0x82_f647;
pub const BINDING_ROWS: usize = 7;

/// $82:C04C: equipment item bits; weapons start here, suits and misc items
/// at $C056 and boots at $C062.
pub const EQUIPMENT_BITS: u32 = 0x82_c04c;
pub const EQUIPMENT_BIT_WORDS: usize = (0xc06c - 0xc04c) / 2;
/// $82:C068: reserve tank tilemap destinations.
pub const RESERVE_TANK_DESTINATIONS: u32 = 0x82_c068;
/// $82:C088: reserve tank tilemaps.
pub const RESERVE_TANK_TILEMAPS: u32 = 0x82_c088;
pub const RESERVE_TANKS: usize = 2;
/// $82:C08C: equipment icon tilemaps; weapons start here, suits and misc
/// items at $C096 and boots at $C0A2.
pub const EQUIPMENT_TILEMAPS: u32 = 0x82_c08c;
pub const EQUIPMENT_TILEMAP_WORDS: usize = (0xc0a8 - 0xc08c) / 2;
/// $82:C0A8: the weapon icon tilemaps shown with the hyper beam.
pub const HYPER_BEAM_TILEMAPS: u32 = 0x82_c0a8;
pub const HYPER_BEAM_WEAPONS: usize = 5;

/// $82:B257: the equipped-suit bits each wireframe matches.
pub const WIREFRAME_SUITS: u32 = 0x82_b257;
/// $82:B25F: the Samus wireframe tilemaps, by suit.
pub const WIREFRAME_TILEMAPS: u32 = 0x82_b25f;
pub const WIREFRAMES: usize = 4;
