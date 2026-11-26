//! Independent observations of the Ceres shaft as she arrives: the elevator's pad and the cover
//! below it, read from the enemy projectile slots, and whether the game holds her.

use super::{compare::Observation, data_failure, result::Outcome};
use crate::source::WramWord;
use boss_ceres_ridley::{AlternatingLook, Cover, Pad, PadAndCover, PadWait};
use core::num::Wrapping;
use types::Pair;

/// The projectile slots, a word apart.
const SLOTS: core::ops::RangeInclusive<u16> = 0..=0x22;
/// Each slot's word: its header, timer, position and spritemap.
const ID: u16 = 0x1997;
const TIMER: u16 = 0x1aff;
const X: u16 = 0x1a4b;
const Y: u16 = 0x1a93;
const SPRITEMAP: u16 = 0x1b6b;
/// The headers of the elevator's pad and the cover below it.
const PAD: u16 = 0xa387;
const COVER: u16 = 0xa395;
/// The pad's spritemaps: none before its first run, then its two looks.
const NO_SPRITEMAP: u16 = 0x8000;
const FIRST_LOOK: u16 = 0xb1ba;
const SECOND_LOOK: u16 = 0xb1d0;

/// Samus's new-state handler, a bank $90 routine pointer: `$E8DC` while the pad holds her.
const SAMUS_NEW_STATE_HANDLER: u16 = 0x0a44;
const HELD_HANDLER: u16 = 0xe8dc;

fn word(wram: &[u8], offset: u16) -> Result<u16, Outcome> {
    WramWord {
        name: "Ceres shaft observation",
        offset,
    }
    .read(wram)
    .map_err(|_| data_failure(None, "Ceres shaft observation"))
}

fn unsupported(dependency: &str) -> Outcome {
    Outcome::Unsupported {
        cut: None,
        dependency: dependency.to_owned(),
    }
}

/// The projectile slot holding `header`, if any.
fn slot(wram: &[u8], header: u16) -> Result<Option<u16>, Outcome> {
    for slot in SLOTS.step_by(2) {
        if word(wram, ID.wrapping_add(slot))? == header {
            return Ok(Some(slot));
        }
    }
    Ok(None)
}

fn position(wram: &[u8], slot: u16) -> Result<Pair<types::Px>, Outcome> {
    Ok(Pair {
        x: Wrapping(word(wram, X.wrapping_add(slot))?),
        y: Wrapping(word(wram, Y.wrapping_add(slot))?),
    })
}

fn pad(wram: &[u8], slot: u16) -> Result<Pad, Outcome> {
    let wait = PadWait::new(word(wram, TIMER.wrapping_add(slot))?)
        .ok_or_else(|| unsupported("Ceres pad's wait past the longest"))?;
    let look = match word(wram, SPRITEMAP.wrapping_add(slot))? {
        NO_SPRITEMAP => None,
        FIRST_LOOK => Some(AlternatingLook::First),
        SECOND_LOOK => Some(AlternatingLook::Second),
        _ => return Err(unsupported("Ceres pad's look")),
    };
    Ok(Pad {
        wait,
        position: position(wram, slot)?,
        look,
    })
}

/// The pad and the cover as the record holds them. The game holds her exactly while the pad is
/// present; a record where the two disagree is outside the shaft's model.
pub(super) fn pad_and_cover(wram: &[u8]) -> Result<PadAndCover, Outcome> {
    pad_cover_with_elevator(wram, false)
}

pub(super) fn pad_cover_with_elevator(wram: &[u8], elevator: bool) -> Result<PadAndCover, Outcome> {
    let pad = slot(wram, PAD)?.map(|slot| pad(wram, slot)).transpose()?;
    let cover = slot(wram, COVER)?
        .map(|slot| position(wram, slot).map(|position| Cover { position }))
        .transpose()?;
    let held = word(wram, SAMUS_NEW_STATE_HANDLER)? == HELD_HANDLER;
    if held != (pad.is_some() || elevator) {
        return Err(unsupported(
            "her lock without the Ceres pad, or the pad without her lock",
        ));
    }
    Ok(PadAndCover { pad, cover })
}

/// The pad and the cover by part, so a difference names the one that differs.
pub(super) fn observations(pad_and_cover: &PadAndCover) -> [Observation; 2] {
    let part = |field: &str, value: String| Observation {
        field: field.into(),
        bytes: value.into_bytes(),
    };
    [
        part("shaft.pad", format!("{:?}", pad_and_cover.pad)),
        part("shaft.cover", format!("{:?}", pad_and_cover.cover)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(wram: &mut [u8], at: u16, value: u16) {
        let at = usize::from(at);
        wram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn the_pad_and_the_cover_are_read_from_their_slots_and_hold_her() {
        let mut wram = vec![0; 0x2000];
        write(&mut wram, ID + 0x22, PAD);
        write(&mut wram, TIMER + 0x22, 29);
        write(&mut wram, X + 0x22, 128);
        write(&mut wram, Y + 0x22, 28);
        write(&mut wram, SPRITEMAP + 0x22, SECOND_LOOK);
        write(&mut wram, ID + 0x20, COVER);
        write(&mut wram, X + 0x20, 128);
        write(&mut wram, Y + 0x20, 97);
        assert!(pad_and_cover(&wram).is_err(), "the pad without her lock");
        write(&mut wram, SAMUS_NEW_STATE_HANDLER, HELD_HANDLER);
        let read = pad_and_cover(&wram).unwrap();
        let pad = read.pad.unwrap();
        assert_eq!(pad.wait, PadWait::new(29).unwrap());
        assert_eq!(pad.position.y, Wrapping(28));
        assert_eq!(pad.look, Some(AlternatingLook::Second));
        assert_eq!(read.cover.unwrap().position.y, Wrapping(97));
        write(&mut wram, ID + 0x22, 0);
        assert!(pad_and_cover(&wram).is_err(), "her lock without the pad");
    }
}
