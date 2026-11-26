//! The byte pipeline of an increasing direct transfer to the two VRAM ports.

use canonical::Canonical;
use data::dma::BusAddress;

use crate::state::video::{BBusRegister, DmaChannel, DmaControl, VramWord};

/// A write in the bus pipeline. The offset fixes both the word and its byte lane.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingWrite {
    /// Byte offset within the transfer destination.
    pub offset: u16,
    /// Accepted byte waiting to be written.
    pub value: u8,
}

/// A direct transfer suspended at a display cut.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transfer {
    /// First address on the source bus.
    pub source: BusAddress,
    /// First word on the destination bus.
    pub destination: VramWord,
    /// Full transfer length in bytes.
    pub length: u16,
    /// Number of source reads accepted.
    pub accepted: u16,
    /// Number of VRAM writes completed.
    pub committed: u16,
    /// Most recently accepted write, if it still waits in the pipeline.
    pub pending: Option<PendingWrite>,
    /// Source selected for the read in progress.
    pub selected: Option<BusAddress>,
}

/// Why a direct transfer cannot make the requested progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Incoherent progress or VRAM storage.
    Position,
    /// The next read needs a live source outside the available bytes.
    Source,
}

impl Transfer {
    /// The transfer before its first read selection.
    #[must_use]
    pub const fn new(source: BusAddress, destination: VramWord, length: u16) -> Self {
        Self {
            source,
            destination,
            length,
            accepted: 0,
            committed: 0,
            pending: None,
            selected: None,
        }
    }

    /// Check a receipt against this transfer and the available immutable source span.
    ///
    /// # Errors
    /// Progress cannot go back, lose an outstanding read, or reach an unavailable source.
    pub fn validate(
        &self,
        memory: &[u16],
        source: &[u8],
        accepted: u16,
        selected: bool,
    ) -> Result<(), Fault> {
        if accepted < self.accepted
            || accepted > self.length
            || (selected && accepted == self.length)
            || (self.selected.is_some() && accepted == self.accepted && !selected)
            || memory.len() != crate::state::video::VRAM_WORDS
        {
            return Err(Fault::Position);
        }
        if usize::from(accepted.saturating_add(u16::from(selected))) > source.len() {
            return Err(Fault::Source);
        }
        Ok(())
    }

    /// Accept reads up to the supplied count and leave the next selected read where indicated.
    ///
    /// # Errors
    /// A receipt outside the supported source span or inconsistent with carried work is refused.
    pub fn advance(
        &mut self,
        memory: &mut [u16],
        source: &[u8],
        accepted: u16,
        selected: bool,
    ) -> Result<(), Fault> {
        self.validate(memory, source, accepted, selected)?;
        for value in source
            .iter()
            .copied()
            .take(usize::from(accepted))
            .skip(usize::from(self.accepted))
        {
            self.write_pending(memory);
            self.pending = Some(PendingWrite {
                offset: self.accepted,
                value,
            });
            self.accepted = self.accepted.saturating_add(1);
        }
        self.selected = selected.then(|| self.source.advanced(self.accepted));
        Ok(())
    }

    /// Retire the last pending write after the final byte has been accepted.
    ///
    /// # Errors
    /// Draining requires the complete transfer and a valid VRAM buffer.
    pub fn drain(&mut self, memory: &mut [u16]) -> Result<(), Fault> {
        if self.accepted != self.length
            || self.selected.is_some()
            || self.pending.is_none()
            || memory.len() != crate::state::video::VRAM_WORDS
        {
            return Err(Fault::Position);
        }
        self.write_pending(memory);
        Ok(())
    }

    fn write_pending(&mut self, memory: &mut [u16]) {
        if let Some(write) = self.pending.take() {
            let word = usize::from(self.destination.0.wrapping_add(write.offset / 2) & 0x7fff);
            if let Some(slot) = memory.get_mut(word) {
                let [lo, hi] = slot.to_le_bytes();
                *slot = u16::from_le_bytes(if write.offset & 1 == 0 {
                    [write.value, hi]
                } else {
                    [lo, write.value]
                });
            }
            self.committed = self.committed.saturating_add(1);
        }
    }

    /// The live VRAM word address, which advances after the high byte writes.
    #[must_use]
    pub const fn vram_cursor(&self) -> VramWord {
        VramWord(self.destination.0.wrapping_add(self.committed / 2) & 0x7fff)
    }

    /// Update the DMA channel's changing registers and retain unrelated HDMA fields.
    pub fn project_channel(&self, channel: &mut DmaChannel) {
        channel.control = DmaControl::TO_TWO_REGISTERS;
        channel.destination = BBusRegister::VramData;
        let next = self.selected.map_or_else(
            || self.source.advanced(self.accepted),
            |read| read.advanced(1),
        );
        channel.source = next.offset;
        channel.source_bank = next.bank;
        channel.count = self.length.saturating_sub(self.accepted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_read_at_the_end_of_the_available_span_is_refused_atomically() {
        let mut memory = vec![0xabcd; crate::state::video::VRAM_WORDS];
        let mut transfer = Transfer::new(
            BusAddress {
                bank: 0,
                offset: u16::MAX,
            },
            VramWord(0),
            2,
        );
        transfer.advance(&mut memory, &[0x12], 1, false).unwrap();
        let held = (transfer, memory.clone());
        assert_eq!(
            transfer.advance(&mut memory, &[0x12], 1, true),
            Err(Fault::Source)
        );
        assert_eq!((transfer, memory), held);
    }

    #[test]
    fn the_last_high_byte_writes_at_drain() {
        let mut memory = vec![0xabcd; crate::state::video::VRAM_WORDS];
        let mut transfer = Transfer::new(BusAddress::default(), VramWord(0), 2);
        transfer
            .advance(&mut memory, &[0x12, 0x34], 2, false)
            .unwrap();
        assert_eq!(memory[0], 0xab12);
        assert_eq!(transfer.vram_cursor(), VramWord(0));
        transfer.drain(&mut memory).unwrap();
        assert_eq!(memory[0], 0x3412);
        assert_eq!(transfer.vram_cursor(), VramWord(1));
    }
}
