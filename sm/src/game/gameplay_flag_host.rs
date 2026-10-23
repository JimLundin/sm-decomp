//! Finite compatibility seam only; normal domain calls use owned collections.
#![forbid(unsafe_code)]
use crate::game::session_access::SessionAccess;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LegacyFlagColumn {
    Events,
    Bosses,
    RoomObjects,
    Items,
    Doors,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BossWordOrder {
    Little,
    Native,
}
pub(crate) trait FlagAliases {
    /// Widened signed displacement from a finite collection base. Never clamp.
    fn flag_byte(&self, column: LegacyFlagColumn, index: isize) -> u8;
    fn set_flag_byte(&mut self, column: LegacyFlagColumn, index: isize, value: u8);
    /// Includes the byte after the owned door collection when index is 63.
    fn door_adjacent_word_le(&self, byte_index: isize) -> u16;
    /// Source-specific reads: native typed dereference vs defined unaligned LE.
    fn boss_word(&self, area: u16, order: BossWordOrder) -> u16;
    /// update_place!(unaligned_word) uses LE reads AND LE writes.
    fn set_boss_word_le(&mut self, area: u16, value: u16);
    /// Kraid's actual typed-pointer store remains native.
    fn set_boss_word_native(&mut self, area: u16, value: u16);
}
pub(crate) trait Host: SessionAccess + FlagAliases {}
