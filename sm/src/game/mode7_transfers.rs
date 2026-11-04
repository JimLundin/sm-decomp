//! Mode 7 VRAM and palette transfers, packed in the original RAM queue.
use crate::game::display_transfers::{DrainHost, Queue};

const QUEUE: usize = sm_game::display::transfer::MODE7_QUEUE;
/// The queue's tail ($7E:0334).
const TAIL: usize = crate::ram_map::words::MODE7_VRAM_WRITE_QUEUE_TAIL;

/// The Mode 7 queue's drain, as the NMI runs it: on Game, or on the frame's
/// lease through its transfer port.
pub(super) fn process(host: &mut impl DrainHost) {
    if host.tail(TAIL) == 0 {
        return;
    }
    sm_game::display::transfer::drain_mode7(&mut Queue(host));
    host.store_queue_byte(QUEUE, 0);
    host.store_queue_byte(QUEUE + 1, 0);
    host.set_tail(TAIL, 0);
}
