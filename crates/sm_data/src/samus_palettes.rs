//! Samus's palette pointer tables: by suit, by animation frame and by beam.
//! Each entry points at a palette in bank $9B (or $90 for the beams).

/// $91:D727: the suit palettes, by suit (power, varia, gravity).
pub const SUIT_PALETTES: u32 = 0x91_d727;
/// $91:D998: the charge-glow sequences, by suit.
pub const CHARGE_SEQUENCES: u32 = 0x91_d998;
/// $91:DA4A: the screw attack frame tables, by suit.
pub const SCREW_ATTACK_FRAMES: u32 = 0x91_da4a;
/// $91:DAA9: the speed booster frame tables, by suit.
pub const SPEED_BOOST_FRAMES: u32 = 0x91_daa9;
/// $91:DB10: the speed booster shine frame tables, by suit.
pub const SHINE_FRAMES: u32 = 0x91_db10;
/// $91:DB75: the shinespark frame tables, by suit.
pub const SHINESPARK_FRAMES: u32 = 0x91_db75;
pub const SUITS: usize = 3;

/// $9B:A3C0: the visor colours of the x-ray and scanning visor, by frame.
pub const VISOR_COLOURS: u32 = 0x9b_a3c0;
pub const VISOR_FRAMES: usize = 6;

/// $91:D99E: the charged-shot palettes, by charge step.
pub const CHARGE_PALETTES: u32 = 0x91_d99e;
pub const CHARGE_STEPS: usize = 10;

/// $91:DC28: the crystal flash's last six colours, by frame.
pub const CRYSTAL_LAST_COLOURS: u32 = 0x91_dc28;
pub const CRYSTAL_FRAMES: usize = 6;

/// $91:D829: the Samus glow palettes, by phase; phase 0 is unused.
pub const GLOW_PALETTES: u32 = 0x91_d829;
pub const GLOW_PHASES: usize = 11;

/// $90:C3C9: the beam palettes in bank $90, by beam combination.
pub const BEAM_PALETTES: u32 = 0x90_c3c9;
pub const BEAM_COMBINATIONS: usize = 12;
