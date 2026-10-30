//! Shared room coordinates and direct mechanism observations for enemy producers.
#![forbid(unsafe_code)]
use super::host::{Host, MechanismMember};
pub fn block_index(handle: u16, h: &impl Host) -> u16 {
    let slot = usize::from(handle >> 1);
    if let Some(mechanism) = h.session().mechanisms.slots.get(slot) {
        mechanism.block
    } else {
        h.mechanism_word(MechanismMember::Block, slot as isize)
    }
}
pub fn projectile_argument(handle: u16, h: &impl Host) -> u16 {
    let slot = usize::from(handle >> 1);
    let value = if let Some(mechanism) = h.session().mechanisms.slots.get(slot) {
        mechanism.room_argument
    } else {
        h.mechanism_word(MechanismMember::RoomArgument, slot as isize)
    };
    u16::from_ne_bytes(value.to_le_bytes())
}
pub fn calculate_coordinates(handle: u16, h: &mut impl Host) {
    let block = block_index(handle, h);
    crate::room_mechanisms::creation::coordinates_from_block(h.session_mut(), block);
}
