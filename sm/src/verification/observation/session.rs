//! The Session-space comparison's two Sessions (docs/handoff/session-compare.md).
//!
//! Verification owns the translation of RAM into a Session (REVIEW, D6). This
//! first slice decodes with the engine's own codec, `session_encoding::decode`,
//! which moves here once the engine no longer imports RAM. The engine's
//! Session is read through the same codec (encoded onto its own frame result,
//! then decoded), so both sides carry exactly the fields the codec owns, and
//! the comparison's ranges are the RAM comparison's: each normalized range
//! takes the oracle's bytes on both sides.

use crate::cpu_infra::NORMALIZED_RAM;
use sm_game::room_entry::world::WorldBinding;
use sm_game::session::Session;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

/// Work RAM's size, as the snapshots hold it.
const RAM_BYTES: usize = 0x20000;

/// Run `decode` without printing a panic: a RAM image the codec rejects is a
/// counted result of this report-only comparison, not a failure.
fn quietly<T>(decode: impl FnOnce() -> T) -> Option<T> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = catch_unwind(AssertUnwindSafe(decode)).ok();
    std::panic::set_hook(hook);
    result
}

fn decode(ram: &[u8], world: &Arc<WorldBinding>) -> Session {
    sm_game::session_encoding::decode(&|at| ram[at], ram.len(), Arc::clone(world))
}

/// The oracle's frame result as a Session, or `None` when the codec rejects it.
pub(super) fn oracle(ram: &[u8; RAM_BYTES], world: &Arc<WorldBinding>) -> Option<Session> {
    quietly(|| decode(ram, world))
}

/// The engine's Session as the codec carries it: encoded onto the engine's
/// frame result (which supplies the bytes the Session does not own), with the
/// normalized ranges taken from the oracle, then decoded.
pub(super) fn engine(
    session: &Session,
    mine: &[u8; RAM_BYTES],
    theirs: &[u8; RAM_BYTES],
    world: &Arc<WorldBinding>,
) -> Option<Session> {
    quietly(|| {
        let mut image = mine.to_vec();
        sm_game::session_encoding::encode(session, |at, span| {
            image[at..at + span.len()].copy_from_slice(span);
        });
        for (_, range) in NORMALIZED_RAM {
            image[range.clone()].copy_from_slice(&theirs[range]);
        }
        decode(&image, world)
    })
}
