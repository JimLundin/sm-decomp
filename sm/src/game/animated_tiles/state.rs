//! Ordinary animation data and its graphics request/acknowledgement protocol.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Default)]
pub(super) struct Slot {
    pub(super) id: u16,
    pub(super) counter: u16,
    pub(super) program: u16,
    pub(super) delay: u16,
    pub(super) source: u16,
    pub(super) size: u16,
    pub(super) vram: u16,
}
#[derive(Default)]
pub(super) struct AnimatedTiles {
    pub(super) slots: [Slot; 6],
    pub(super) flags: u16,
    pub(super) selected: u16,
    pub(super) last_instruction: u16,
}
/// One bank-87 graphics request. Completion acknowledges the captured slot.
pub(super) struct Upload {
    pub(super) source: u16,
    pub(super) size: u16,
    pub(super) vram: u16,
}
impl AnimatedTiles {
    pub(super) fn pending_upload(&self, slot: usize) -> Option<Upload> {
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
    pub(super) fn acknowledge_upload(&mut self, slot: usize) {
        self.slots[slot].source = 0;
    }
}
