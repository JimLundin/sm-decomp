pub(crate) const SIZE: usize = 0x20000;
pub(crate) const X: usize = 0x0af6;
pub(crate) const XF: usize = 0x0af8;
pub(crate) const Y: usize = 0x0afa;
pub(crate) const YF: usize = 0x0afc;
pub(crate) const XR: usize = 0x0afe;
pub(crate) const YR: usize = 0x0b00;
pub(crate) const DIRECTION: usize = 0x0b02;
pub(crate) const MOVEMENT: usize = 0x0a1f;
pub(crate) const COLLISION: usize = 0x0dd0;
pub(crate) const FEEDBACK: usize = 0x0dc6;
pub(crate) const UP: usize = 0x0db2;
pub(crate) const DOWN: usize = 0x0db6;
pub(crate) const LIST_GATE: usize = 0x17a6;
pub(crate) const LIST: usize = 0x17ec;
pub(crate) const COLLIDING: usize = 0x182c;
pub(crate) const INDEX: usize = 0x18a6;
pub(crate) const ENEMIES: usize = 0x0f78;

pub(crate) fn word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([ram[at], ram[at + 1]])
}

pub(crate) fn put(ram: &mut [u8], at: usize, value: u16) {
    ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}

pub(crate) fn set_hilo(ram: &mut [u8], hi: usize, amount: i32) {
    put(ram, hi, (amount >> 16) as u16);
    put(ram, hi + 2, amount as u16);
}

pub(crate) fn add_y(ram: &mut [u8], amount: i32) {
    let position = (u32::from(word(ram, Y)) << 16) | u32::from(word(ram, YF));
    let next = position.wrapping_add(amount as u32);
    put(ram, Y, (next >> 16) as u16);
    put(ram, YF, next as u16);
}
