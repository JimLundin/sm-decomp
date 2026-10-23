//! Transitional byte codec and out-of-range legacy slot aliases.
#![forbid(unsafe_code)]
use super::{
    Owner,
    state::{AnimatedTiles, Slot},
    transport::Codec,
};
impl Slot {
    fn word(&mut self, column: usize) -> &mut u16 {
        match column {
            0 => &mut self.id,
            1 => &mut self.counter,
            2 => &mut self.program,
            3 => &mut self.delay,
            4 => &mut self.source,
            5 => &mut self.size,
            _ => &mut self.vram,
        }
    }
}
impl AnimatedTiles {
    fn word(&mut self, offset: usize) -> &mut u16 {
        match offset {
            0 => &mut self.flags,
            1 => &mut self.selected,
            44 => &mut self.last_instruction,
            _ => self.slots[(offset - 2) % 6].word((offset - 2) / 6),
        }
    }
    fn value(&self, offset: usize) -> u16 {
        match offset {
            0 => self.flags,
            1 => self.selected,
            44 => self.last_instruction,
            _ => {
                let mut slot = self.slots[(offset - 2) % 6];
                *slot.word((offset - 2) / 6)
            }
        }
    }
}
impl Codec for AnimatedTiles {
    fn decode(ram: &[u8]) -> Self {
        let mut state = Self::default();
        for offset in 0..45 {
            let at = 0x1ef1 + offset * 2;
            *state.word(offset) = u16::from_le_bytes([ram[at], ram[at + 1]]);
        }
        state
    }
    fn encode(&self, ram: &mut [u8]) {
        for offset in 0..45 {
            let at = 0x1ef1 + offset * 2;
            ram[at..at + 2].copy_from_slice(&self.value(offset).to_le_bytes());
        }
    }
    fn byte(&self, address: usize) -> Option<u8> {
        if !(0x1ef1..0x1f4b).contains(&address) {
            return None;
        }
        let offset = address - 0x1ef1;
        Some(self.value(offset / 2).to_le_bytes()[offset % 2])
    }
    fn put_byte(&mut self, address: usize, value: u8) -> bool {
        if !(0x1ef1..0x1f4b).contains(&address) {
            return false;
        }
        let offset = address - 0x1ef1;
        let word = self.word(offset / 2);
        let mut bytes = word.to_le_bytes();
        bytes[offset % 2] = value;
        *word = u16::from_le_bytes(bytes);
        true
    }
}

#[derive(Clone, Copy)]
pub(super) enum SlotWord {
    Id,
    Counter,
    Program,
    Delay,
    Source,
}
impl SlotWord {
    fn field(self, slot: &mut Slot) -> &mut u16 {
        match self {
            Self::Id => &mut slot.id,
            Self::Counter => &mut slot.counter,
            Self::Program => &mut slot.program,
            Self::Delay => &mut slot.delay,
            Self::Source => &mut slot.source,
        }
    }
    fn address(self, index: u16) -> usize {
        let base = match self {
            Self::Id => 0x1ef5,
            Self::Counter => 0x1f01,
            Self::Program => 0x1f0d,
            Self::Delay => 0x1f19,
            Self::Source => 0x1f25,
        };
        base + usize::from(index & !1)
    }
}
impl Owner<'_> {
    pub(super) fn read_slot(&self, field: SlotWord, index: u16) -> u16 {
        if let Some(mut slot) = self.state.slots.get(usize::from(index >> 1)).copied() {
            *field.field(&mut slot)
        } else {
            self.read_le(field.address(index))
        }
    }
    pub(super) fn write_slot(&mut self, field: SlotWord, index: u16, value: u16) {
        if let Some(slot) = self.state.slots.get_mut(usize::from(index >> 1)) {
            *field.field(slot) = value;
        } else {
            self.write_word_le(value, field.address(index));
        }
    }
    pub(super) fn write_counter_byte(&mut self, index: u16, value: u8) {
        if let Some(slot) = self.state.slots.get_mut(usize::from(index >> 1)) {
            let mut bytes = slot.counter.to_le_bytes();
            bytes[usize::from(index & 1)] = value;
            slot.counter = u16::from_le_bytes(bytes);
        } else {
            self.set_byte(0x1f01 + usize::from(index), value);
        }
    }
}
