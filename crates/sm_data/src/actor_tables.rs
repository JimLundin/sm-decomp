//! Small word tables of the bosses and enemies, each indexed by a health
//! stage, angle step or random choice.

/// $B3:946B: Botwoon's head instruction lists, by eighth of a turn.
pub const BOTWOON_HEAD_INSTRUCTION_LISTS: u32 = 0xb3_946b;
/// $B3:948B: Botwoon's spitting instruction lists, by eighth of a turn.
pub const BOTWOON_SPIT_INSTRUCTION_LISTS: u32 = 0xb3_948b;
pub const EIGHTHS: usize = 8;

/// $A5:96AF: Draygon's health palettes, eight stages of four colours.
pub const DRAYGON_HEALTH_PALETTES: u32 = 0xa5_96af;
pub const DRAYGON_HEALTH_PALETTE_WORDS: usize = 8 * 4;
/// $A5:96EF: the health below which each Draygon palette stage starts.
pub const DRAYGON_HEALTH_THRESHOLDS: u32 = 0xa5_96ef;
pub const DRAYGON_HEALTH_STAGES: usize = 9;
/// $A5:87DC: the wall turrets' (x, y) positions, by turret slot.
pub const DRAYGON_TURRET_POSITIONS: u32 = 0xa5_87dc;
pub const DRAYGON_TURRET_POSITION_WORDS: usize = 2 * 6;

/// $A6:B965 and $A6:B96D: pointers to Ridley's pogo X and Y velocity rows,
/// by random pick.
pub const RIDLEY_X_VELOCITY_ROWS: u32 = 0xa6_b965;
pub const RIDLEY_Y_VELOCITY_ROWS: u32 = 0xa6_b96d;
pub const RIDLEY_VELOCITY_ROWS: usize = 4;
/// $A6:B94D and $A6:B959: Ridley's pogo accelerations, by acceleration step.
pub const RIDLEY_UPWARDS_ACCELERATIONS: u32 = 0xa6_b94d;
pub const RIDLEY_DOWNWARDS_ACCELERATIONS: u32 = 0xa6_b959;
pub const RIDLEY_ACCELERATION_STEPS: usize = 6;

/// $A8:A097: Yapping Maw's attack instruction lists, by eighth of a turn.
pub const MAW_ATTACK_INSTRUCTION_LISTS: u32 = 0xa8_a097;
