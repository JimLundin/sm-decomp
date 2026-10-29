//! Ceres Ridley's tables in bank $A6: the room fade-in, attack choices,
//! baby metroid and wing frames, the getaway path and the Ridley fade-in colours.
use crate::table::{LengthError, Rows};

/// $A6:A4EB: colour data pointers for the Ridley room background fade-in,
/// one per transition frame; the zero sixteenth ends the fade.
pub const BACKGROUND_FADE_ADDRESS: u32 = 0xa6_a4eb;
pub const BACKGROUND_FADE_FRAMES: usize = 16;

/// $A6:A743: hovering's next function, chosen by the random number's low
/// nibble (six fireballs, five lunges, five swoops).
pub const ATTACK_ADDRESS: u32 = 0xa6_a743;
pub const ATTACKS: usize = 16;

/// $A6:ACDA: tile transfer pointers for the baby metroid's four animation frames.
pub const BABY_METROID_FRAME_ADDRESS: u32 = 0xa6_acda;
pub const BABY_METROID_FRAMES: usize = 4;

/// $A6:AD45: tile transfer pointers for the wings' two animation frames.
pub const WING_FRAME_ADDRESS: u32 = 0xa6_ad45;
pub const WING_FRAMES: usize = 2;

/// $A6:AE4D: the getaway's zoom levels, Y velocities ($AF2F) and X velocities
/// ($B00F), one list after another. The zoom list has one more word, the
/// $FFFF that ends the getaway.
pub const GETAWAY_ADDRESS: u32 = 0xa6_ae4d;
pub const GETAWAY_STEPS: usize = 112;
pub const GETAWAY_BYTES: usize = (GETAWAY_STEPS + 1) * 2 + GETAWAY_STEPS * 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GetawayVelocity {
    pub y: u16,
    pub x: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Getaway {
    zoom: Rows<u16, { GETAWAY_STEPS + 1 }>,
    velocity: Rows<GetawayVelocity, GETAWAY_STEPS>,
}

impl Getaway {
    pub fn import(bytes: &[u8]) -> Result<Self, LengthError> {
        if bytes.len() != GETAWAY_BYTES {
            return Err(LengthError(bytes.len()));
        }
        let (zoom, velocity) = bytes.split_at((GETAWAY_STEPS + 1) * 2);
        Ok(Self {
            zoom: crate::table::words(zoom)?,
            velocity: Rows::import_columns(velocity, |[y, x]| GetawayVelocity { y, x })?,
        })
    }

    /// Step `index`'s zoom level, or the terminator one past the last step.
    pub fn zoom(&self, index: usize) -> Option<u16> {
        self.zoom.row(index)
    }

    pub fn velocity(&self, index: usize) -> Option<GetawayVelocity> {
        self.velocity.row(index)
    }
}

/// $A6:E2AA: sixteen fade-in steps of Ridley's eye colours Ch..Eh.
pub const EYE_FADE_ADDRESS: u32 = 0xa6_e2aa;
pub const EYE_FADE_COLOURS: usize = 3;
/// $A6:E30A: sixteen fade-in steps of Ridley's body colours 1..Bh.
pub const BODY_FADE_ADDRESS: u32 = 0xa6_e30a;
pub const BODY_FADE_COLOURS: usize = 11;
pub const FADE_STEPS: usize = 16;
