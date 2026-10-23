//! Transitional native ROM reads retain the caller's allocation contract.
use super::port::Execution;
use crate::c_library::__assert_fail;
use std::ffi::c_char;
pub(super) struct NativeSequence(*const u8);
impl NativeSequence {
    /// The captured native allocation must remain live and contain this offset.
    pub(super) unsafe fn byte(&self, offset: isize) -> u8 {
        unsafe { *self.0.offset(offset) }
    }
    /// The allocation must contain both bytes; words use native byte order.
    pub(super) unsafe fn word(&self, offset: isize) -> u16 {
        unsafe { u16::from_ne_bytes([self.byte(offset), self.byte(offset + 1)]) }
    }
}
/// Captures g_rom, which is intentionally distinct from loaded_rom.
/// The caller supplies RomPtr's valid allocation and address preconditions.
pub(super) unsafe fn sequence(
    execution: &mut Execution<'_>,
    bank: u8,
    cursor: u16,
) -> NativeSequence {
    execution.foreign(|game| {
        NativeSequence(unsafe {
            crate::rtl::RomPtr((u32::from(bank) << 16) | u32::from(cursor), game)
        })
    })
}
pub(super) fn validate_ram_draw_address(address: u16) {
    unsafe {
        crate::native_assertion! {
            '_c2rust_label: address < 0x2000;
            b"addr < 0x2000\0", b"src/sm_84.c\0", 265, b"uint8 *RomPtr_84orRAM(uint16_t)\0"
        };
    }
}
impl Execution<'_> {
    pub(super) unsafe fn draw_word(&mut self, address: u16) -> u16 {
        if address & 0x8000 != 0 {
            unsafe { sequence(self, 0x84, address).word(0) }
        } else {
            if address >= 0x2000 {
                self.foreign(|_| validate_ram_draw_address(address));
            }
            self.port().native_word(usize::from(address))
        }
    }
    pub(super) unsafe fn draw_byte(&mut self, address: u16) -> u8 {
        if address & 0x8000 != 0 {
            unsafe { sequence(self, 0x84, address).byte(0) }
        } else {
            if address >= 0x2000 {
                self.foreign(|_| validate_ram_draw_address(address));
            }
            self.port().byte(usize::from(address))
        }
    }
}
