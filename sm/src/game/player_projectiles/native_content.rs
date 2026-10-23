//! Retained native-ROM contract for the two flare sequences.
//!
//! Unlike checked record consumers, these native callers select g_rom, which
//! can differ from loaded_rom. No allocation length or ownership is invented.
//! This is explicit transitional unsafe content debt, not a safe domain module.
use super::Transaction;
pub(super) struct FlareSequence(*const u8);
impl FlareSequence {
    pub(super) unsafe fn byte(&self, offset: usize) -> u8 {
        // The original sequence's valid native allocation/index contract remains.
        unsafe { *self.0.add(offset) }
    }
}
pub(super) unsafe fn flare_sequence(
    owner: Transaction<'_>,
    cursor: u16,
) -> (Transaction<'_>, FlareSequence) {
    owner.foreign(|game| {
        FlareSequence(unsafe { crate::rtl::RomPtr(0x900000 | u32::from(cursor), game) })
    })
}
