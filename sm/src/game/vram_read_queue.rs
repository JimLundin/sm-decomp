//! Packed VRAM readback requests shared by X-ray backgrounds and Kraid pause.
use crate::game::display_commands::byte;
use crate::game::display_transfers::{DrainHost, Queue};
use crate::snes::registers::VMAIN;

/// The queue's records ($7E:0340).
const QUEUE: usize = 0x340;
/// The queue's tail ($7E:0360).
const TAIL: usize = crate::ram_map::words::VRAM_READ_QUEUE_TAIL;

/// The readback queue's drain, on the frame's lease, through its transfer
/// port, as the NMI runs it: its DMA starts return the work RAM bytes they
/// write to the Session.
pub(super) fn process(host: &mut impl DrainHost) {
    let tail = host.tail(TAIL) as u8;
    if tail != 0 {
        // Only the low sentinel byte is cleared; the high byte remains live.
        host.store_queue_byte(QUEUE + usize::from(tail), 0);
        byte(VMAIN, 0x80, host);
        // DMA can overwrite later records, so each is decoded after the last ran.
        sm_game::display::transfer::drain_readbacks(&mut Queue(host));
        host.store_queue_byte(TAIL, 0);
    }
}
