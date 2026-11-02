//! Packed Mode7 records reach the actual canonical channel and byte consumers.
#![forbid(unsafe_code)]
use super::state::{DisplayError, TransferSource};
use crate::display::transfer_types::{Mode7Target, Mode7Upload};
use crate::session::Session;

struct Packed<'a>(&'a mut Session);
impl crate::display::mode7_owned::Drain for Packed<'_> {
    type Error = DisplayError;
    fn byte(&mut self, at: usize) -> Result<u8, DisplayError> {
        crate::session_encoding::projected_byte(self.0, at)
            .ok_or(DisplayError::Alias { offset: at })
    }
    fn word(&mut self, at: usize) -> Result<u16, DisplayError> {
        Ok(u16::from_le_bytes([self.byte(at)?, self.byte(at + 1)?]))
    }
    fn upload(&mut self, upload: Mode7Upload) -> Result<(), DisplayError> {
        deliver(self.0, upload)
    }
}
pub(crate) fn drain(session: &mut Session) -> Result<(), DisplayError> {
    if session.presentation.owner()?.state.mode7_tail == 0 {
        return Ok(());
    }
    crate::display::mode7_owned::drain(&mut Packed(session))?;
    let state = &mut session.presentation.owner_mut()?.state;
    state.mode7_queue[..2].fill(0);
    state.mode7_tail = 0;
    // The imported semantic list described the same packed queue.
    state.mode7_uploads.clear();
    Ok(())
}

/// Register publications precede the first symbolic source lookup, in native order.
fn deliver(session: &mut Session, upload: Mode7Upload) -> Result<(), DisplayError> {
    let state = &mut session.presentation.owner_mut()?.state;
    super::hardware::write_dma_channel(state, 1, super::hardware::DmaField::Control, upload.mode)?;
    state.hdma.channels[1].source_address = upload.source.address;
    state.hdma.channels[1].source_bank = upload.source.bank;
    state.hdma.channels[1].indirect_cursor = upload.bytes;
    match upload.target {
        Mode7Target::Vram {
            high_byte,
            address,
            increment,
        } => {
            state.hdma.channels[1].destination = if high_byte { 0x19 } else { 0x18 };
            // Address writes prime the original VRAM read latch before VMAIN.
            let [low, high] = address.to_le_bytes();
            session
                .presentation
                .owner_mut()?
                .mode7_pointer_byte(false, low)?;
            session
                .presentation
                .owner_mut()?
                .mode7_pointer_byte(true, high)?;
            let state = &mut session.presentation.owner_mut()?.state;
            state.latches.tile_increment = match increment & 3 {
                0 => 1,
                1 => 32,
                _ => 128,
            };
            state.latches.tile_remap = (increment >> 2) & 3;
            state.latches.tile_increment_on_high = increment & 0x80 != 0;
        }
        Mode7Target::Cgram { index } => {
            state.hdma.channels[1].destination = 0x22;
            state.latches.palette_pointer = index;
            state.latches.palette_second = false;
        }
    }
    let state = &mut session.presentation.owner_mut()?.state;
    for (index, channel) in state.hdma.channels.iter_mut().enumerate() {
        channel.dma_active = index == 1;
    }
    state.hdma.dma_busy = true;
    state.hdma.dma_timer = state.hdma.dma_timer.wrapping_add(16);
    // MDMAEN's synchronous drain settles the timer rather than advancing a new clock.
    loop {
        let state = &mut session.presentation.owner_mut()?.state;
        if state.hdma.dma_timer != 0 {
            state.hdma.dma_timer = state.hdma.dma_timer.wrapping_sub(2);
            continue;
        }
        if !state.hdma.channels[1].dma_active {
            state.hdma.dma_busy = false;
            return Ok(());
        }
        const OFFSETS: [[u8; 4]; 8] = [
            [0, 0, 0, 0],
            [0, 1, 0, 1],
            [0, 0, 0, 0],
            [0, 0, 1, 1],
            [0, 1, 2, 3],
            [0, 1, 0, 1],
            [0, 0, 0, 0],
            [0, 0, 1, 1],
        ];
        let channel = &mut state.hdma.channels[1];
        let cursor = channel.transfer_cursor;
        channel.transfer_cursor = channel.transfer_cursor.wrapping_add(1);
        let destination = channel
            .destination
            .wrapping_add(OFFSETS[usize::from(channel.mode)][usize::from(cursor)]);
        let address = channel.source_address;
        let bank = channel.source_bank;
        let [low, high] = upload.source.address.to_le_bytes();
        let source = session
            .presentation
            .owner()?
            .state
            .queued_sources
            .get(&[low, high, bank])
            .cloned()
            .ok_or(DisplayError::Source)?;
        let value = source_byte(session, source, upload.source.address, address)?;
        session
            .presentation
            .owner_mut()?
            .mode7_destination(destination, value)?;
        let state = &mut session.presentation.owner_mut()?.state;
        let channel = &mut state.hdma.channels[1];
        channel.transfer_cursor &= 3;
        state.hdma.dma_timer = state.hdma.dma_timer.wrapping_add(6);
        if !channel.fixed {
            channel.source_address = if channel.decrement {
                channel.source_address.wrapping_sub(1)
            } else {
                channel.source_address.wrapping_add(1)
            };
        }
        channel.indirect_cursor = channel.indirect_cursor.wrapping_sub(1);
        if channel.indirect_cursor == 0 {
            channel.transfer_cursor = 0;
            channel.dma_active = false;
            state.hdma.dma_timer = state.hdma.dma_timer.wrapping_add(8);
        }
    }
}
fn source_byte(
    session: &Session,
    mut source: TransferSource,
    initial: u16,
    current: u16,
) -> Result<u8, DisplayError> {
    match &mut source {
        TransferSource::ActorState { address } => {
            let actual = address.wrapping_add(current.wrapping_sub(initial));
            crate::session_encoding::projected_byte(session, usize::from(actual)).ok_or(
                DisplayError::Alias {
                    offset: usize::from(actual),
                },
            )
        }
        TransferSource::Asset(span) => {
            let delta = i32::from(current) - i32::from(initial);
            let offset = span
                .offset
                .checked_add_signed(isize::try_from(delta).map_err(|_| DisplayError::Source)?)
                .ok_or(DisplayError::Source)?;
            span.bytes.get(offset).copied().ok_or(DisplayError::Source)
        }
        other => session
            .presentation
            .owner()?
            .state
            .source_byte(other, usize::from(current.wrapping_sub(initial))),
    }
}
