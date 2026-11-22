//! One-way import of actual initial prelude owner facts.
//! Later bands, blend results, lists and producer readiness come from separate receipts.
// Source identities are imported only at this one-way verification edge.
const RAMP_PRESTEP: u32 = 8_969_773;
const HOLD_PRESTEP: u32 = 8_946_872;
const BUILDER_CURSOR: usize = 1_424;
const PREPARED_FLAGS: usize = 1_821;
const PREPARED_TOP: usize = 1_823;
const PREPARED_BOTTOM: usize = 1_825;

use data::timing::ChannelSlot;
use game::production::frame::borrowed::{
    ActorEligibilityOrigin, ActorSlot, ColourComponent, DefaultLayerBlend, HazeOrigin, HazeSlot,
    PreludeSeed, SpriteBuilderOrigin,
};

/// Invalid origin context or unqualified semantic identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreludeSeedFault {
    /// Not a complete physical WRAM origin.
    Length,
    /// Unsupported pointer, channel, component or owner policy.
    Unsupported,
}
struct Wram<'a>(&'a [u8]);
impl Wram<'_> {
    fn byte(&self, at: usize) -> Result<u8, PreludeSeedFault> {
        self.0.get(at).copied().ok_or(PreludeSeedFault::Length)
    }
    fn word(&self, at: usize) -> Result<u16, PreludeSeedFault> {
        Ok(u16::from_le_bytes([
            self.byte(at)?,
            self.byte(at.wrapping_add(1))?,
        ]))
    }
    fn haze(&self) -> Result<HazeOrigin, PreludeSeedFault> {
        if self.word(2680)? != 0 || self.word(1426)? != 0 || self.word(6320)? & 0x8000 == 0 {
            return Err(PreludeSeedFault::Unsupported);
        }
        let mut slots = [HazeSlot::Empty; 6];
        for (slot, destination) in slots.iter_mut().enumerate() {
            let stride = slot.wrapping_mul(2);
            let mask = self.byte(6324_usize.wrapping_add(stride))?;
            if mask == 0 {
                continue;
            }
            let channel = match mask {
                4 => ChannelSlot::Three,
                128 => ChannelSlot::Eight,
                _ => return Err(PreludeSeedFault::Unsupported),
            };
            let pointer = (u32::from(self.word(6396_usize.wrapping_add(stride))?) << 16)
                | u32::from(self.word(6384_usize.wrapping_add(stride))?);
            let countdown = self.word(6372_usize.wrapping_add(stride))?;
            *destination = if pointer == RAMP_PRESTEP {
                let component = match self.byte(6432_usize.wrapping_add(stride))? {
                    32 => ColourComponent::Red,
                    64 => ColourComponent::Green,
                    128 => ColourComponent::Blue,
                    _ => return Err(PreludeSeedFault::Unsupported),
                };
                HazeSlot::BackdropRamp {
                    channel,
                    countdown,
                    maximum: self.word(6420_usize.wrapping_add(stride))?,
                    component,
                }
            } else if pointer == HOLD_PRESTEP {
                HazeSlot::KeepTransfer { channel, countdown }
            } else {
                return Err(PreludeSeedFault::Unsupported);
            };
        }
        Ok(HazeOrigin {
            running: true,
            power_bomb_inactive: true,
            enabled: true,
            default_blend: DefaultLayerBlend(self.word(6530)?),
            slots,
        })
    }
    fn builder(&self) -> Result<SpriteBuilderOrigin, PreludeSeedFault> {
        let backing = self
            .0
            .get(880..1424)
            .ok_or(PreludeSeedFault::Length)?
            .try_into()
            .map_err(|_| PreludeSeedFault::Length)?;
        Ok(SpriteBuilderOrigin {
            backing,
            cursor: self.word(BUILDER_CURSOR)?,
            prepared_flags: self.word(PREPARED_FLAGS)?,
            prepared_top: self.word(PREPARED_TOP)?,
            prepared_bottom: self.word(PREPARED_BOTTOM)?,
        })
    }
    fn actors(&self) -> Result<ActorEligibilityOrigin, PreludeSeedFault> {
        if self.word(6204)? != 0 {
            return Err(PreludeSeedFault::Unsupported);
        }
        let mut slots = [ActorSlot::Empty; 32];
        for (slot, destination) in slots.iter_mut().enumerate() {
            let at = 3960_usize.wrapping_add(slot.wrapping_mul(64));
            *destination = match self.word(at)? {
                0 => ActorSlot::Empty,
                0xdaff => ActorSlot::Placeholder,
                _ => {
                    let properties = self.word(at.wrapping_add(14))?;
                    ActorSlot::Occupied {
                        deleted: properties & 0x0200 != 0,
                        process_offscreen: properties & 0x0800 != 0,
                        intangible: properties & 0x0400 != 0,
                    }
                }
            };
        }
        Ok(ActorEligibilityOrigin {
            permit_marked_offscreen: true,
            slots,
        })
    }
}
/// Import one qualified completed-return origin, never a later producer snapshot.
///
/// Source identity/integrity and bounded invariance are separate admission obligations.
/// No bands, prepared blend, random result, membership list or readiness is manufactured.
/// # Errors
/// Rejects incomplete origin memory and unsupported current semantic identities/policies.
pub fn decode(bytes: &[u8]) -> Result<PreludeSeed, PreludeSeedFault> {
    if bytes.len() != 131_072 {
        return Err(PreludeSeedFault::Length);
    }
    let wram = Wram(bytes);
    Ok(PreludeSeed {
        haze: wram.haze()?,
        builder: wram.builder()?,
        actors: actors(bytes)?,
    })
}

pub fn actors(bytes: &[u8]) -> Result<ActorEligibilityOrigin, PreludeSeedFault> {
    if bytes.len() != 131_072 {
        return Err(PreludeSeedFault::Length);
    }
    Wram(bytes).actors()
}
