//! Import the intro's three pad timelines as named holds.

use core::num::NonZeroU16;
use data::assets::playback::{PadAction, PadHold, PadSample, ScenePad};
use data::rom::{AddressError, Lorom, RomAddress};

/// Why a pad recording could not be imported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    /// A recording extends beyond the image.
    Address(AddressError),
    /// A hold is empty, uses unsupported buttons or presses a button it does not hold.
    Hold(RomAddress),
    /// The last hold of a discovery timeline does not hold only Left.
    DiscoveryTail(RomAddress),
}

/// Import the finite battle, approach and watch holds.
///
/// # Errors
/// The source spans must be readable and contain valid holds with Left as each discovery tail.
pub fn import(rom: Lorom<'_>) -> Result<ScenePad, ImportError> {
    Ok(ScenePad {
        battle: timeline(rom, 0x8694, 17, false)?,
        approach: timeline(rom, 0x860d, 2, true)?,
        watch: timeline(rom, 0x8623, 5, true)?,
    })
}

fn timeline(
    rom: Lorom<'_>,
    start: u16,
    entries: u16,
    discovery: bool,
) -> Result<Vec<PadHold>, ImportError> {
    let mut holds = Vec::with_capacity(usize::from(entries));
    for entry in 0..entries {
        let at = RomAddress {
            bank: 0x91,
            offset: start.wrapping_add(entry.wrapping_mul(6)),
        };
        let bytes: [u8; 6] = rom
            .read(at, 6)
            .map_err(ImportError::Address)?
            .try_into()
            .map_err(|_| ImportError::Hold(at))?;
        let [d0, d1, h0, h1, p0, p1] = bytes;
        let turns = NonZeroU16::new(u16::from_le_bytes([d0, d1]))
            .filter(|turns| turns.get() < 0x8000)
            .ok_or(ImportError::Hold(at))?;
        let sample = sample(u16::from_le_bytes([h0, h1]), u16::from_le_bytes([p0, p1]))
            .ok_or(ImportError::Hold(at))?;
        if discovery
            && entry == entries.saturating_sub(1)
            && sample.held != PadSample::walking_left().held
        {
            return Err(ImportError::DiscoveryTail(at));
        }
        holds.push(PadHold { turns, sample });
    }
    Ok(holds)
}

fn sample(held: u16, pressed: u16) -> Option<PadSample> {
    let bindings = [
        (0x0200, PadAction::Left),
        (0x0080, PadAction::Jump),
        (0x0040, PadAction::Fire),
        (0x0010, PadAction::AimUp),
    ];
    if (held & !bindings.iter().fold(0, |mask, (bit, _)| mask | bit)) != 0 || (pressed & !held) != 0
    {
        return None;
    }
    let mut sample = PadSample::neutral();
    for (bit, action) in bindings {
        sample.held[action] = held & bit != 0;
        sample.pressed[action] = pressed & bit != 0;
    }
    Some(sample)
}
