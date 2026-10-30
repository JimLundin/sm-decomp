//! Kraid, Phantoon and Dachora word tables in bank $A7 that the bank code
//! reads directly: rock positions, spit records, speeds, sound effects,
//! instruction lists and palettes.

/// $A7:ACB3: X positions of the rocks that fall from Kraid's ceiling.
pub const KRAID_ROCK_X_POSITIONS: u32 = 0xa7_acb3;
pub const KRAID_ROCKS: usize = 8;

/// $A7:BC65: Kraid's rock spit X velocities in 1/100h px/frame, by random choice.
pub const KRAID_ROCK_SPIT_X_VELOCITIES: u32 = 0xa7_bc65;
pub const KRAID_ROCK_SPITS: usize = 8;

/// $A7:BE3E: Kraid's fingernail upward velocity records, by random choice.
pub const KRAID_NAIL_UPWARD_VELOCITIES: u32 = 0xa7_be3e;
/// $A7:BE46: Kraid's fingernail downward velocity records, by random choice.
pub const KRAID_NAIL_DOWNWARD_VELOCITIES: u32 = 0xa7_be46;
pub const KRAID_NAIL_CHOICES: usize = 4;

/// $A7:CA41: the fade-out target palette, sixteen colours.
pub const FADE_OUT_TARGET_PALETTE: u32 = 0xa7_ca41;
pub const FADE_OUT_TARGET_COLOURS: usize = 16;
/// $A7:CA61: the Wrecked Ship powered-on BG1/2 target palettes 0..6.
pub const WRECKED_SHIP_POWERED_ON_PALETTES: u32 = 0xa7_ca61;
pub const WRECKED_SHIP_POWERED_ON_COLOURS: usize = 7 * 16;

/// $A7:CDED: Phantoon's materialisation sound effects, by step.
pub const PHANTOON_MATERIALISATION_SOUNDS: u32 = 0xa7_cded;
pub const PHANTOON_MATERIALISATION_STEPS: usize = 3;

/// $A7:CE8E: Phantoon's flame instruction lists, by flame kind.
pub const PHANTOON_FLAME_INSTRUCTION_LISTS: u32 = 0xa7_ce8e;
pub const PHANTOON_FLAME_KINDS: usize = 4;

/// $A7:D40D: Phantoon's body instruction lists, by pattern.
pub const PHANTOON_INSTRUCTION_LISTS: u32 = 0xa7_d40d;
pub const PHANTOON_PATTERNS: usize = 8;

/// $A7:F787: Dachora's palettes while charging its shine, by step.
pub const DACHORA_CHARGE_PALETTES: u32 = 0xa7_f787;
/// $A7:F92D: Dachora's palettes while shining, by step.
pub const DACHORA_SHINE_PALETTES: u32 = 0xa7_f92d;
pub const DACHORA_PALETTE_STEPS: usize = 4;
