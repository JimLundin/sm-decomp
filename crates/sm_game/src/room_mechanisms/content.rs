//! Captured instruction positions independent of content storage and backend.
#![forbid(unsafe_code)]
use super::host::InstructionRecord;

pub struct Script<R> {
    record: R,
    origin: u16,
    cursor: isize,
    jump_target: Option<u16>,
}
impl<R: InstructionRecord> Script<R> {
    pub fn new(record: R, origin: u16) -> Self {
        Self {
            record,
            origin,
            cursor: 0,
            jump_target: None,
        }
    }
    pub fn byte(&self, offset: isize) -> u8 {
        self.record.byte(
            self.cursor
                .checked_add(offset)
                .expect("mechanism script offset overflow"),
        )
    }
    pub fn word(&self, offset: isize) -> u16 {
        // Keep the provider's native word semantics and sequential operand reads.
        self.record.word(
            self.cursor
                .checked_add(offset)
                .expect("mechanism script offset overflow"),
        )
    }
    pub fn load24(&self, offset: isize) -> u32 {
        u32::from_ne_bytes(core::array::from_fn(|i| self.byte(offset + i as isize))) & 0xffffff
    }
    pub fn advance(mut self, bytes: isize) -> Self {
        self.jump_target = None;
        self.cursor = self
            .cursor
            .checked_add(bytes)
            .expect("mechanism script offset overflow");
        self
    }
    pub fn seek(mut self, position: u16) -> Self {
        self.jump_target = None;
        self.cursor = position as isize - self.origin as isize;
        self
    }
    pub fn jump(self, address: u16) -> Option<Self> {
        if address == 0 {
            None
        } else {
            let mut result = self.seek(address);
            result.jump_target = Some(address);
            Some(result)
        }
    }
    pub fn jump_word(self, offset: isize) -> Option<Self> {
        let address = self.word(offset);
        self.jump(address)
    }
    pub fn jump_target(&self) -> Option<u16> {
        self.jump_target
    }
    pub fn into_parts(self) -> (R, isize) {
        (self.record, self.cursor)
    }
    pub fn position(&self) -> u16 {
        (self.origin as isize + self.cursor) as u16
    }
}
impl<R: InstructionRecord> InstructionRecord for Script<R> {
    fn byte(&self, offset: isize) -> u8 {
        self.byte(offset)
    }
    fn word(&self, offset: isize) -> u16 {
        self.word(offset)
    }
}
