//! Mother Brain's tables in banks $A9 and $AD: the fake death's explosions,
//! the brain's attack instruction lists and palette sets, the body's walking
//! instruction lists, the falling tubes' functions and the rainbow beam palettes,
//! plus the health-based body and back-leg palettes and the rainbow beam's colour
//! cycle in bank $88.
pub use crate::table::LengthError;
use crate::table::Rows;

/// $A9:8929: the fake death's eight explosion positions.
pub const FAKE_DEATH_EXPLOSION_ADDRESS: u32 = 0xa9_8929;
pub const FAKE_DEATH_EXPLOSION_ROWS: usize = 8;
pub const FAKE_DEATH_EXPLOSION_BYTES: usize = FAKE_DEATH_EXPLOSION_ROWS * 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExplosionPosition {
    pub x: u16,
    pub y: u16,
}

pub type FakeDeathExplosions = Rows<ExplosionPosition, FAKE_DEATH_EXPLOSION_ROWS>;

pub fn import_fake_death_explosions(bytes: &[u8]) -> Result<FakeDeathExplosions, LengthError> {
    Rows::import(bytes, |[x, y]| ExplosionPosition { x, y })
}

/// $A9:B6D4: the brain's instruction lists for neutral, blue rings, laser and
/// bomb, chosen by the attack RNG thresholds.
pub const BRAIN_ATTACK_ADDRESS: u32 = 0xa9_b6d4;
pub const BRAIN_ATTACK_ROWS: usize = 4;
pub const BRAIN_ATTACK_BYTES: usize = BRAIN_ATTACK_ROWS * 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrainAttack {
    /// Bank $A9 pointer to the brain's instruction list.
    pub instruction_list: u16,
}

pub type BrainAttacks = Rows<BrainAttack, BRAIN_ATTACK_ROWS>;

pub fn import_brain_attacks(bytes: &[u8]) -> Result<BrainAttacks, LengthError> {
    Rows::import(bytes, |[instruction_list]| BrainAttack { instruction_list })
}

/// $A9:D260: the brain's palette sets, normal then charging her laser.
pub const BRAIN_PALETTE_SET_ADDRESS: u32 = 0xa9_d260;
pub const BRAIN_PALETTE_SET_ROWS: usize = 2;
pub const BRAIN_PALETTE_SET_BYTES: usize = BRAIN_PALETTE_SET_ROWS * 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BrainPaletteSet {
    /// Bank $A9 pointer to eight rows of four brain colours.
    pub colors: u16,
}

pub type BrainPaletteSets = Rows<BrainPaletteSet, BRAIN_PALETTE_SET_ROWS>;

pub fn import_brain_palette_sets(bytes: &[u8]) -> Result<BrainPaletteSets, LengthError> {
    Rows::import(bytes, |[colors]| BrainPaletteSet { colors })
}

/// $A9:C61E and $A9:C664: the body's walking instruction lists, indexed by
/// animation delay / 2 from really fast (1) to really slow (5). Row 0 is unused.
pub const WALK_FORWARDS_ADDRESS: u32 = 0xa9_c61e;
pub const WALK_BACKWARDS_ADDRESS: u32 = 0xa9_c664;
pub const WALK_ROWS: usize = 6;
pub const WALK_BYTES: usize = WALK_ROWS * 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Walk {
    /// Bank $A9 pointer to the body's instruction list.
    pub instruction_list: u16,
}

pub type Walks = Rows<Walk, WALK_ROWS>;

pub fn import_walks(bytes: &[u8]) -> Result<Walks, LengthError> {
    Rows::import(bytes, |[instruction_list]| Walk { instruction_list })
}

/// $A9:8B7B: the falling tubes' functions, indexed by enemy parameter 1 / 2.
pub const FALLING_TUBE_ADDRESS: u32 = 0xa9_8b7b;
pub const FALLING_TUBE_ROWS: usize = 5;
pub const FALLING_TUBE_BYTES: usize = FALLING_TUBE_ROWS * 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FallingTube {
    /// Bank $A9 pointer to the tube's function.
    pub function: u16,
}

pub type FallingTubes = Rows<FallingTube, FALLING_TUBE_ROWS>;

pub fn import_falling_tubes(bytes: &[u8]) -> Result<FallingTubes, LengthError> {
    Rows::import(bytes, |[function]| FallingTube { function })
}

/// $AD:E434: the body's rainbow beam palettes, ten then a zero terminator.
pub const RAINBOW_BEAM_PALETTE_ADDRESS: u32 = 0xad_e434;
pub const RAINBOW_BEAM_PALETTE_ROWS: usize = 11;
pub const RAINBOW_BEAM_PALETTE_BYTES: usize = RAINBOW_BEAM_PALETTE_ROWS * 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RainbowBeamPalette {
    /// Bank $AD pointer to two lines of fifteen colours; zero ends the table.
    pub colors: u16,
}

pub type RainbowBeamPalettes = Rows<RainbowBeamPalette, RAINBOW_BEAM_PALETTE_ROWS>;

pub fn import_rainbow_beam_palettes(bytes: &[u8]) -> Result<RainbowBeamPalettes, LengthError> {
    Rows::import(bytes, |[colors]| RainbowBeamPalette { colors })
}

/// $AD:E6A2: brain/body palette pointers by health level.
pub const BODY_PALETTES_ADDRESS: u32 = 0xad_e6a2;
/// $AD:E742: back leg palette pointers by health level.
pub const BACK_LEG_PALETTES_ADDRESS: u32 = 0xad_e742;
pub const HEALTH_LEVELS: usize = 5;
pub const PALETTES_BYTES: usize = 2 * HEALTH_LEVELS;

/// Bank $AD addresses of fifteen-colour palettes, one per health level.
pub type HealthPalettes = Rows<u16, HEALTH_LEVELS>;

pub fn import_health_palettes(bytes: &[u8]) -> Result<HealthPalettes, LengthError> {
    Rows::import(bytes, |[palette]| palette)
}

/// $88:E833: 75 BGR555 colours, then a run of $FFFF words; the cycle restarts
/// at the first word with bit 15 set.
pub const RAINBOW_ADDRESS: u32 = 0x88_e833;
pub const RAINBOW_WORDS: usize = 83;
pub const RAINBOW_BYTES: usize = 2 * RAINBOW_WORDS;

pub type RainbowColors = Rows<u16, RAINBOW_WORDS>;

pub fn import_rainbow(bytes: &[u8]) -> Result<RainbowColors, LengthError> {
    Rows::import(bytes, |[color]| color)
}
