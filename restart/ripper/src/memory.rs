pub(crate) const SIZE: usize = 0x20000;
pub(crate) const CURRENT: usize = 0xe54;
pub(crate) const ENEMIES: usize = 0xf78;
pub(crate) const TIMER: usize = 0x18;
pub(crate) const INSTRUCTION: usize = 0x1a;
pub(crate) const INSTRUCTION_TIMER: usize = 0x1c;
pub(crate) const C: usize = 0x34;
pub(crate) const D: usize = 0x36;
pub(crate) const E: usize = 0x38;
pub(crate) const PARAMETER1: usize = 0x3c;
pub(crate) const PARAMETER2: usize = 0x3e;
pub(crate) const TABLE: usize = 0x110187;
pub(crate) const TABLE_SIZE: usize = 65 * 8;

pub(crate) fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
pub(crate) fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
pub(crate) fn valid_slot(slot: u16) -> bool {
    slot < 0x800 && slot.is_multiple_of(0x40)
}
pub(crate) fn valid_row(offset: u16) -> bool {
    offset <= 0x200 && offset.is_multiple_of(8)
}
pub(crate) fn speed(rom: &[u8], row: u16, right: bool) -> (u16, u16) {
    let at = TABLE + usize::from(row) + if right { 0 } else { 4 };
    (word(rom, at), word(rom, at + 2))
}
