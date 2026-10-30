//! Room liquid (FX) and HDMA objects as one Session-owned domain: design.
//!
//! Step 1 (liquid) is implemented below; step 2 (HDMA objects) is planned.
//! Constraints come from docs/ENGINE_STATE_MIGRATION.md.
//!
//! # Scope and owner types
//!
//! Step 1, liquid: `Room::liquid: Liquid` joins the five existing `room.liquid_*`
//! words (kept as they are) with the rest of the $195C–$1980 block (not $1966–$196A, which
//! is unnamed; not the layer blending words at $1982–$1986, which belong to
//! display):
//!
//! | RAM | field | before step 1 |
//! |---|---|---|
//! | $195C/$195E | `surface: Position` (fx_y_subpos/pos) | RAM only |
//! | $1960/$1962 | `lava_acid: Position` | RAM only |
//! | $1964 | `tilemap: u16` | RAM only |
//! | $196C | `rising: u16` (bank $88 handler) | RAM only |
//! | $196E | `kind: u16` (fx_type) | RAM only |
//! | $1970/$1972 | `offset: Position` | RAM only |
//! | $1974 | `tide_phase: u16` | RAM only |
//! | $1976 | `base_fraction: u16` (whole stays `room.liquid_height`) | RAM only |
//! | $1978–$1980 | `room.liquid_{height,target,velocity,options,delay}` | owned |
//!
//! Step 2, HDMA objects: `Presentation::hdma: HdmaObjects` takes $18B0–$1958:
//! the enable flag, selected index, and six slots of channel mask, bank,
//! instruction list, table, instruction timer, pre-instruction (+bank),
//! timer and the six behaviour words A–F. The liquid routines are HDMA
//! pre-instructions over these slots, so step 2 completes step 1's callers.
//!
//! Every word keeps its full stored `u16` (no enums yet): behaviours write
//! unknown values and off-table indices alias neighbouring words.
//!
//! # Encoder and checkpoints
//!
//! - `session_encoding`: one `$f(addr, field)` entry per word above, so import
//!   decodes and export encodes exactly the same bytes (little-endian words).
//! - Corpus checkpoints (`.smvchk`) serialise WRAM plus actors, not Session,
//!   so their layout fingerprint and seed assets do not change.
//! - The persistent `SessionCheckpoint` codec (`checkpoint_fields!` for Room
//!   and Presentation) gains the new fields deliberately, with a roundtrip check.
//!
//! # Writers: closed or bridged
//!
//! One authority per interval: a Runtime lease owns the domain while open;
//! compat writes outside a lease write RAM, bump the epoch and are re-imported
//! at the next lease (the existing bridge), so none is lost.
//!
//! Verifier-observed lease and transfer counts must not change, so no writer
//! gains a lease it did not already have:
//!
//! - `room_state` (FX kind and tilemap) and `player_transitions_runtime`
//!   (surface reset to $FFFF): closed. They already hold a Runtime lease, so
//!   they write the typed fields.
//! - `liquid_effects` (sm_game; HDMA pre-instructions), `sm_a6` (Ridley/Ceres
//!   rising lava), `sm_aa` (Torizo acid) and `visual_runtime`: bridged. They run
//!   on `Game` without a lease; their RAM writes suspend the owner, which is
//!   re-imported at the next lease. They close when their callers run on one.
//! - `visual_runtime` HDMA interpreter: bridged in step 1; closed in step 2,
//!   when it runs the slot table from `presentation.hdma`.
//! - Readers (grapple, sky, x-ray, Crocomire, escape critters, explosion
//!   windows, Mother Brain, menus, display refresh): already read through the
//!   projection while the owner is current; converted to typed reads as their
//!   own modules migrate.
//!
//! # Risks
//!
//! - Step 1 is a complete closure only for the liquid words; it lands as one
//!   commit, step 2 as another, each behind a full corpus.
use crate::geometry::Position;
use crate::session::Room;

/// The room liquid's motion and surface words not already in `Room`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Liquid {
    /// $195C/$195E: the drawn liquid surface.
    pub surface: Position,
    /// $1960/$1962: the lava or acid surface used for damage.
    pub lava_acid: Position,
    /// $1964: FX tilemap pointer.
    pub tilemap: u16,
    /// $196C: the bank $88 rising handler.
    pub rising: u16,
    /// $196E: FX type.
    pub kind: u16,
    /// $1970/$1972: the surface's wave offset.
    pub offset: Position,
    /// $1974.
    pub tide_phase: u16,
    /// $1976: the fraction of the base height, whose whole is `Room::liquid_height`.
    pub base_fraction: u16,
}

/// The liquid words one frame reads and writes: `Room`'s five plus `Liquid`.
/// Taken from the owned Room under a lease, or read from compatibility RAM
/// when a lease is not admissible; the same frame logic runs on either.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LiquidWords {
    /// $1978.
    pub height: u16,
    /// $197A.
    pub target: u16,
    /// $197C.
    pub velocity: u16,
    /// $197E.
    pub options: u16,
    /// $1980.
    pub delay: u16,
    pub liquid: Liquid,
}

impl LiquidWords {
    pub fn of(room: &Room) -> Self {
        Self {
            height: room.liquid_height,
            target: room.liquid_target,
            velocity: room.liquid_velocity,
            options: room.liquid_options,
            delay: room.liquid_delay,
            liquid: room.liquid,
        }
    }
    pub fn store(&self, room: &mut Room) {
        room.liquid_height = self.height;
        room.liquid_target = self.target;
        room.liquid_velocity = self.velocity;
        room.liquid_options = self.options;
        room.liquid_delay = self.delay;
        room.liquid = self.liquid;
    }
    /// The words at their compatibility addresses, little-endian as the
    /// encoder stores them.
    fn slots(&mut self) -> [(usize, &mut u16); 16] {
        let l = &mut self.liquid;
        [
            (0x195c, &mut l.surface.fraction),
            (0x195e, &mut l.surface.whole),
            (0x1960, &mut l.lava_acid.fraction),
            (0x1962, &mut l.lava_acid.whole),
            (0x1964, &mut l.tilemap),
            (0x196c, &mut l.rising),
            (0x196e, &mut l.kind),
            (0x1970, &mut l.offset.fraction),
            (0x1972, &mut l.offset.whole),
            (0x1974, &mut l.tide_phase),
            (0x1976, &mut l.base_fraction),
            (0x1978, &mut self.height),
            (0x197a, &mut self.target),
            (0x197c, &mut self.velocity),
            (0x197e, &mut self.options),
            (0x1980, &mut self.delay),
        ]
    }
    pub fn read(ram: &(impl crate::ram::CompatibilityRam + ?Sized)) -> Self {
        let mut words = Self::default();
        for (at, word) in words.slots() {
            *word = crate::projection::word(ram, at);
        }
        words
    }
    /// Store `value` to the one word `field` selects, low byte then high,
    /// as a named word store does.
    pub fn set(
        ram: &mut (impl crate::ram::CompatibilityRam + ?Sized),
        field: impl FnOnce(&mut Self) -> &mut u16,
        value: u16,
    ) {
        let mut words = Self::default();
        let target: *const u16 = field(&mut words);
        let at = words
            .slots()
            .into_iter()
            .find(|(_, word)| core::ptr::eq(&**word, target))
            .map(|(at, _)| at)
            .expect("liquid word");
        crate::projection::set_word(ram, at, value);
    }
    pub fn write(mut self, ram: &mut (impl crate::ram::CompatibilityRam + ?Sized)) {
        for (at, word) in self.slots() {
            crate::projection::set_word(ram, at, *word);
        }
    }
}

/// The bank $88 handlers in `Liquid::rising`: lava and acid ("hot") liquids and
/// water each idle, count down `Room::liquid_delay`, then move to the target.
pub mod rising {
    pub const HOT_IDLE: u16 = 0xb343;
    pub const HOT_DELAY: u16 = 0xb367;
    pub const HOT_MOVING: u16 = 0xb382;
    pub const WATER_IDLE: u16 = 0xc428;
    pub const WATER_DELAY: u16 = 0xc44c;
    pub const WATER_MOVING: u16 = 0xc458;
}

/// What a motion step asks for once its state change is published.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MotionEffects {
    /// A hot liquid waiting or moving shakes the room with its sound.
    pub shake: bool,
    /// `rising` held no known handler; the original reports it and continues.
    pub unknown_handler: bool,
}

/// Advance the base height toward its target; true once there, or when the
/// target is disabled (bit 15 set).
fn move_toward_target(words: &mut LiquidWords) -> bool {
    if words.target & 0x8000 != 0 {
        return true;
    }
    let rising = words.velocity & 0x8000 != 0;
    let velocity = ((words.velocity as i16 as i32) << 8) as u32;
    let base = (u32::from(words.height) << 16) | u32::from(words.liquid.base_fraction);
    let mut position = (velocity.wrapping_add(base) >> 16) as u16;
    // Only the fraction word takes the 32-bit sum; the whole is set below.
    words.liquid.base_fraction =
        u32::from(words.liquid.base_fraction).wrapping_add(velocity) as u16;
    if position & 0x8000 != 0 {
        position = if rising { 0 } else { 0xffff };
    }
    words.height = position;
    let reached = if rising {
        words.target >= position
    } else {
        words.target < position
    };
    if reached {
        words.height = words.target;
        words.liquid.base_fraction = 0;
    }
    reached
}

/// One frame of the rising state machine.
pub fn advance_motion(words: &mut LiquidWords) -> MotionEffects {
    use rising::*;
    let state = words.liquid.rising;
    let mut effects = MotionEffects {
        shake: matches!(state, HOT_DELAY | HOT_MOVING),
        unknown_handler: false,
    };
    match state {
        HOT_IDLE | WATER_IDLE => {
            let velocity = words.velocity;
            let target = words.target;
            let base = words.height;
            if velocity != 0
                && if velocity & 0x8000 == 0 {
                    target > base
                } else {
                    target < base
                }
            {
                words.liquid.rising = if state == HOT_IDLE {
                    HOT_DELAY
                } else {
                    WATER_DELAY
                };
            }
        }
        HOT_DELAY | WATER_DELAY => {
            words.delay = words.delay.wrapping_sub(1);
            if words.delay == 0 {
                words.liquid.rising = if state == HOT_DELAY {
                    HOT_MOVING
                } else {
                    WATER_MOVING
                };
            }
        }
        HOT_MOVING | WATER_MOVING => {
            if move_toward_target(words) {
                words.velocity = 0;
                words.liquid.rising = if state == HOT_MOVING {
                    HOT_IDLE
                } else {
                    WATER_IDLE
                };
            }
        }
        _ => effects.unknown_handler = true,
    }
    effects
}

/// One frame of the tide: a sine offset in amplitude 8 (options bit 15 read
/// across the velocity/options words) or 32 (bit 14), else none.
pub fn advance_tide(words: &mut LiquidWords) {
    // The control word straddles the velocity's high byte and the options' low byte.
    let control = u16::from_ne_bytes([
        words.velocity.to_le_bytes()[1],
        words.options.to_le_bytes()[0],
    ]);
    let (amplitude, sign_mask, positive_step, negative_step) = if control & 0x8000 != 0 {
        (8u16, 0x1000, 288, 192)
    } else if control & 0x4000 != 0 {
        (32u16, 0x400, 224, 128)
    } else {
        return;
    };
    words.liquid.offset = Position::default();
    let sine =
        crate::projectile_geometry::SINE_COSINE[usize::from(words.liquid.tide_phase >> 8)] as u16;
    let product = amplitude.wrapping_mul(sine);
    if sine & sign_mask != 0 {
        words.liquid.offset.whole = words.liquid.offset.whole.wrapping_sub(1);
    }
    // The product is stored as a native word at $1971: the fraction's high
    // byte and the whole's low byte.
    let [low, high] = product.to_ne_bytes();
    let offset = &mut words.liquid.offset;
    offset.fraction = u16::from_le_bytes([offset.fraction.to_le_bytes()[0], low]);
    offset.whole = u16::from_le_bytes([high, offset.whole.to_le_bytes()[1]]);
    words.liquid.tide_phase = words.liquid.tide_phase.wrapping_add(if sine & 0x8000 == 0 {
        positive_step
    } else {
        negative_step
    });
}

/// Which surface a liquid's frame writes: lava and acid damage at
/// `Liquid::lava_acid`; water is drawn at `Liquid::surface`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Surface {
    LavaAcid,
    Water,
}

/// Store the base height plus the tide offset as the surface, as the original
/// native word stores did.
pub fn place_surface(words: &mut LiquidWords, surface: Surface) {
    let offset =
        (u32::from(words.liquid.offset.whole) << 16) | u32::from(words.liquid.offset.fraction);
    let base = (u32::from(words.height) << 16) | u32::from(words.liquid.base_fraction);
    let position = offset.wrapping_add(base);
    let native = |value: u16| u16::from_le_bytes(value.to_ne_bytes());
    let stored = Position {
        whole: native((position >> 16) as u16),
        fraction: native(position as u16),
    };
    match surface {
        Surface::LavaAcid => words.liquid.lava_acid = stored,
        Surface::Water => words.liquid.surface = stored,
    }
}

/// One frame of motion, tide and surface. Stops before the tide when the
/// handler is unknown, so the caller can report it with the original's prefix.
pub fn step(words: &mut LiquidWords, surface: Surface) -> MotionEffects {
    let effects = advance_motion(words);
    if !effects.unknown_handler {
        advance_tide(words);
        place_surface(words, surface);
    }
    effects
}
