//! Small gameplay tables across banks: palette bands, death palettes, area
//! label pointers, sines and instruction lists, each indexed by a stage,
//! suit, frame, area or angle.

/// $82:965F: the pause map's area label pointers, by area.
pub const AREA_LABELS: u32 = 0x82_965f;
pub const AREAS: usize = 8;

/// $84:8032: the Torizo statue's low palette band per health stage.
pub const TORIZO_LOW_PALETTE_BANDS: u32 = 0x84_8032;
/// $84:8132: the Torizo statue's high palette band per health stage.
pub const TORIZO_HIGH_PALETTE_BANDS: u32 = 0x84_8132;
/// Eight health stages of sixteen colours.
pub const TORIZO_PALETTE_WORDS: usize = 8 * 16;

/// $9B:B5C8: Samus's death palette pointers before the flash, by suit.
pub const DEATH_PALETTES: u32 = 0x9b_b5c8;
/// $9B:B6D2: Samus's death palette pointers during the flash, by suit.
pub const DEATH_FLASH_PALETTES: u32 = 0x9b_b6d2;
pub const SUITS: usize = 3;
/// $9B:B80F: the death sequence's second palette, by frame.
pub const DEATH_FRAME_PALETTES: u32 = 0x9b_b80f;
pub const DEATH_FRAMES: usize = 10;

/// $A0:B1C3: the enemy sine table, by 8-bit angle.
pub const ENEMY_SINE: u32 = 0xa0_b1c3;
pub const ANGLES: usize = 256;

/// $A2:EA56: the timed shutter's two timer words per setting.
pub const SHUTTER_TIMERS: u32 = 0xa2_ea56;
pub const SHUTTER_TIMER_WORDS: usize = 2 * 8;

/// $A5:E379: Spore Spawn's health palettes, four stages of sixteen colours.
pub const SPORE_SPAWN_PALETTES: u32 = 0xa5_e379;
pub const SPORE_SPAWN_PALETTE_WORDS: usize = 4 * 16;

/// $B2:F959: space pirate wall-jump instruction lists, by choice.
pub const PIRATE_INSTRUCTION_LISTS: u32 = 0xb2_f959;
pub const PIRATE_CHOICES: usize = 8;

/// $88:A938: the scrolling sky's wave offsets, by row.
pub const SKY_SCROLL_WAVE: u32 = 0x88_a938;
pub const SKY_SCROLL_WAVE_WORDS: usize = 32;
/// $88:B589: the lava's horizontal wave offsets, by sample.
pub const LAVA_HORIZONTAL_WAVE: u32 = 0x88_b589;
/// $88:B60A: the lava's vertical wave offsets, by sample.
pub const LAVA_VERTICAL_WAVE: u32 = 0x88_b60a;
pub const LAVA_WAVE_SAMPLES: usize = 16;

/// $9B:C346: the grapple point's tile source, by beam angle.
pub const GRAPPLE_POINT_TILES: u32 = 0x9b_c346;
pub const GRAPPLE_POINT_ANGLES: usize = 64;
