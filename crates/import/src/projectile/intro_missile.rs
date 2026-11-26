//! Read the ordinary missile clips and the one-sided exhaust used in the intro.

use super::{ImportError, Pools, advance, word};
use data::assets::projectile::{
    ClipEnd, Damage, ExhaustFrame, IntroMissile, Offset, TrailAppearanceId,
};
use data::rom::Lorom;
use data::spritemap::SpriteAttributes;
use std::num::NonZeroU16;

const SHOTS: u8 = 0x93;
const TRAIL_CODE: u8 = 0x90;
const MISSILE_DESCRIPTOR: u16 = 0x8641;
const LEFT_FLIGHT: u16 = 0x9f03;
const IMPACT: u16 = 0xa039;
const EXHAUST: u16 = 0xb5a1;
const EMPTY_EXHAUST: u16 = 0xb4c9;
const EXHAUST_FRAME_LIMIT: usize = 32;

/// Import the cinematic missile's art, animation holds and damage.
///
/// # Errors
/// Invalid sprite, clip, damage or exhaust records.
pub fn import_intro_missile(rom: Lorom<'_>) -> Result<IntroMissile, ImportError> {
    let damage =
        Damage::try_from(word(rom, SHOTS, MISSILE_DESCRIPTOR)?).map_err(|_| ImportError::Damage)?;
    let mut pool = Pools::default();
    let (flight, _, _) = pool.read_clip(rom, LEFT_FLIGHT, |_, _| Ok(TrailAppearanceId(0)))?;
    let (impact, _, _) = pool.read_clip(rom, IMPACT, |_, _| Ok(TrailAppearanceId(0)))?;
    if !matches!(flight.end, ClipEnd::RepeatFrom(_)) || impact.end != ClipEnd::Remove {
        return Err(ImportError::Control);
    }
    if word(rom, TRAIL_CODE, EMPTY_EXHAUST)? != 0 {
        return Err(ImportError::Control);
    }
    Ok(IntroMissile {
        sheet: pool.sheet,
        flight,
        impact,
        damage,
        exhaust_offset: Offset { x: -4, y: -4 },
        exhaust: exhaust(rom)?,
    })
}

fn exhaust(rom: Lorom<'_>) -> Result<Vec<ExhaustFrame>, ImportError> {
    let mut frames = Vec::new();
    let mut pointer = EXHAUST;
    for _ in 0..EXHAUST_FRAME_LIMIT {
        let Some(hold) = NonZeroU16::new(word(rom, TRAIL_CODE, pointer)?) else {
            return if frames.is_empty() {
                Err(ImportError::EmptyClip)
            } else {
                Ok(frames)
            };
        };
        if hold.get() > i16::MAX.unsigned_abs() {
            return Err(ImportError::Control);
        }
        let look = SpriteAttributes::from_word(word(rom, TRAIL_CODE, advance(pointer, 2)?)?);
        frames.push(ExhaustFrame { hold, look });
        pointer = advance(pointer, 4)?;
    }
    Err(ImportError::Control)
}
