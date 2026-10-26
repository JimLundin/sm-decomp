//! One captured, linear enemy instruction cursor shared by all hosts.
#![forbid(unsafe_code)]
use super::host::{ScriptPosition, ScriptRecord};
const BANK_BASE: usize = 0x28000;
/// A captured allocation supporting checked physical byte reads.
pub(crate) trait ScriptBytes {
    fn read_byte(&self, position: usize) -> u8;
}
pub(crate) struct EprojScript<I> {
    image: I,
    position: usize,
}
impl<I: ScriptBytes> EprojScript<I> {
    pub(crate) fn new(image: I, address: u16) -> Self {
        Self {
            image,
            position: BANK_BASE + usize::from(address),
        }
    }
}
impl<I: ScriptBytes> ScriptRecord for EprojScript<I> {
    fn byte(&self, offset: isize) -> u8 {
        self.image.read_byte(
            self.position
                .checked_add_signed(offset)
                .expect("instruction offset overflow"),
        )
    }
    fn native_word(&self, offset: isize) -> u16 {
        u16::from_ne_bytes([self.byte(offset), self.byte(offset + 1)])
    }
    fn long24(&self) -> u32 {
        u32::from(self.native_word(0)) | (u32::from(self.byte(2)) << 16)
    }
    fn bank_address(&self) -> u16 {
        self.position.wrapping_sub(BANK_BASE) as u16
    }
    fn offset(&self, bytes: isize) -> Option<ScriptPosition> {
        Some(ScriptPosition(
            self.position
                .checked_add_signed(bytes)
                .expect("instruction offset overflow"),
        ))
    }
    fn jump(&self, address: u16) -> Option<ScriptPosition> {
        (address != 0).then_some(ScriptPosition(BANK_BASE + usize::from(address)))
    }
    fn seek(&mut self, position: ScriptPosition) {
        self.position = position.0;
    }
}
