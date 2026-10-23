//! Private palette execution state and exact byte-lane codec.
#![forbid(unsafe_code)]
use super::transport::Codec;
#[derive(Clone, Copy)]
#[repr(usize)]
pub(super) enum Column {
    Id = 0x1e7d,
    Color = 0x1e8d,
    PreAction = 0x1ead,
    Program = 0x1ebd,
    Delay = 0x1ecd,
    Counter = 0x1edd,
}
impl Column {
    pub(super) fn address(self, index: u16) -> usize {
        self as usize + usize::from(index & !1)
    }
}
#[derive(Clone, Copy, Default)]
struct Slot {
    id: u16,
    color: u16,
    pre_action: u16,
    program: u16,
    delay: u16,
    counter: u16,
}
impl Slot {
    fn word(&mut self, column: usize) -> &mut u16 {
        match column {
            0 => &mut self.id,
            1 => &mut self.color,
            2 => &mut self.pre_action,
            3 => &mut self.program,
            4 => &mut self.delay,
            _ => &mut self.counter,
        }
    }
}
#[derive(Default)]
pub(super) struct PaletteEffects {
    slots: [Slot; 8],
    flags: u16,
    selected: u16,
    heat: u16,
    previous_heat: u16,
}
const REGIONS: [(usize, usize); 2] = [(0x1e79, 18), (0x1ead, 34)];
fn location(address: usize) -> Option<(usize, usize, usize)> {
    for (column, base) in [0x1e7d, 0x1e8d, 0x1ead, 0x1ebd, 0x1ecd, 0x1edd]
        .into_iter()
        .enumerate()
    {
        if (base..base + 16).contains(&address) {
            return Some((column, (address - base) / 2, (address - base) % 2));
        }
    }
    for (index, base) in [0x1e79, 0x1e7b, 0x1eed, 0x1eef].into_iter().enumerate() {
        if (base..base + 2).contains(&address) {
            return Some((6, index, address - base));
        }
    }
    None
}
impl PaletteEffects {
    fn word(&mut self, column: usize, index: usize) -> &mut u16 {
        if column < 6 {
            self.slots[index].word(column)
        } else {
            match index {
                0 => &mut self.flags,
                1 => &mut self.selected,
                2 => &mut self.heat,
                _ => &mut self.previous_heat,
            }
        }
    }
    fn value(&self, column: usize, index: usize) -> u16 {
        if column < 6 {
            let mut slot = self.slots[index];
            *slot.word(column)
        } else {
            match index {
                0 => self.flags,
                1 => self.selected,
                2 => self.heat,
                _ => self.previous_heat,
            }
        }
    }
}
impl Codec for PaletteEffects {
    fn decode(ram: &[u8]) -> Self {
        let mut state = Self::default();
        for (base, count) in REGIONS {
            for i in 0..count {
                let at = base + 2 * i;
                let (c, s, _) = location(at).unwrap();
                *state.word(c, s) = u16::from_le_bytes([ram[at], ram[at + 1]]);
            }
        }
        state
    }
    fn encode(&self, ram: &mut [u8]) {
        for (base, count) in REGIONS {
            for i in 0..count {
                let at = base + 2 * i;
                let (c, s, _) = location(at).unwrap();
                ram[at..at + 2].copy_from_slice(&self.value(c, s).to_le_bytes());
            }
        }
    }
    fn byte(&self, address: usize) -> Option<u8> {
        let (c, s, lane) = location(address)?;
        Some(self.value(c, s).to_le_bytes()[lane])
    }
    fn put_byte(&mut self, address: usize, value: u8) -> bool {
        let Some((c, s, lane)) = location(address) else {
            return false;
        };
        let word = self.word(c, s);
        let mut bytes = word.to_le_bytes();
        bytes[lane] = value;
        *word = u16::from_le_bytes(bytes);
        true
    }
}
