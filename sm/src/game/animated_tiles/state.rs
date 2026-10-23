//! Ordinary animation state and synchronous graphics request/acknowledgement.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Slot {
    pub id: u16,
    pub counter: u16,
    pub program: u16,
    pub delay: u16,
    pub source: u16,
    pub size: u16,
    pub vram: u16,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Field {
    Id,
    Counter,
    Program,
    Delay,
    Source,
}
impl Slot {
    pub(crate) fn field(&self, field: Field) -> u16 {
        match field {
            Field::Id => self.id,
            Field::Counter => self.counter,
            Field::Program => self.program,
            Field::Delay => self.delay,
            Field::Source => self.source,
        }
    }
    pub(crate) fn field_mut(&mut self, field: Field) -> &mut u16 {
        match field {
            Field::Id => &mut self.id,
            Field::Counter => &mut self.counter,
            Field::Program => &mut self.program,
            Field::Delay => &mut self.delay,
            Field::Source => &mut self.source,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct AnimatedTiles {
    pub slots: [Slot; 6],
    pub flags: u16,
    pub selected: u16,
    pub last_instruction: u16,
}
/// Size zero retains the graphics backend's full-bank transfer meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Upload {
    pub source: u16,
    pub size: u16,
    pub vram: u16,
}
impl AnimatedTiles {
    pub(crate) fn set_enabled(&mut self, enabled: bool) {
        self.flags = if enabled {
            self.flags | 0x8000
        } else {
            self.flags & !0x8000
        };
    }
    pub(crate) fn clear(&mut self) {
        for slot in self.slots.iter_mut().rev() {
            slot.id = 0;
        }
    }
    pub(crate) fn pending_upload(&self, slot: usize) -> Option<Upload> {
        let slot = &self.slots[slot];
        if slot.id == 0 || slot.source == 0 {
            return None;
        }
        Some(Upload {
            source: slot.source,
            size: slot.size,
            vram: slot.vram,
        })
    }
    pub(crate) fn acknowledge_upload(&mut self, slot: usize) {
        self.slots[slot].source = 0;
    }
}
