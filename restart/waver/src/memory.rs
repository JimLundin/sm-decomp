pub(crate) const SIZE: usize = 0x20000;
pub(crate) const CURRENT: usize = 0x0e54;
pub(crate) const ENEMIES: usize = 0x0f78;
pub(crate) const TIMER: usize = 0x18;
pub(crate) const INSTRUCTION: usize = 0x1a;
pub(crate) const INSTRUCTION_TIMER: usize = 0x1c;
pub(crate) const A: usize = 0x30;
pub(crate) const B: usize = 0x32;
pub(crate) const C: usize = 0x34;
pub(crate) const D: usize = 0x36;
pub(crate) const E: usize = 0x38;
pub(crate) const F: usize = 0x3a;
pub(crate) const PARAMETER: usize = 0x3c;
pub(crate) const TABLE: usize = 0x1186db;
pub(crate) fn word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([ram[at], ram[at + 1]])
}
pub(crate) fn put(ram: &mut [u8], at: usize, value: u16) {
    ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
pub(crate) fn valid_slot(slot: u16) -> bool {
    slot < 0x800 && slot & 63 == 0
}
pub(crate) fn current(ram: &[u8]) -> Result<(u16, usize), String> {
    let slot = word(ram, CURRENT);
    if !valid_slot(slot) {
        return Err(format!("invalid Waver current slot {slot:#06x}"));
    }
    Ok((slot, ENEMIES + usize::from(slot)))
}
pub(crate) fn table_available(rom: &[u8]) -> Result<(), String> {
    if rom.get(TABLE..TABLE + 8).is_none() {
        return Err("Waver runtime ROM animation table is incomplete".into());
    }
    Ok(())
}
