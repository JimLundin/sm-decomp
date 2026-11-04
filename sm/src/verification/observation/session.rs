//! The Session-space comparison's two Sessions (docs/handoff/session-compare.md).
//!
//! Verification owns the translation of RAM into a Session (REVIEW, D6). This
//! first slice decodes with the engine's own codec, `session_encoding::decode`,
//! which moves here once the engine no longer imports RAM. The engine's
//! Session is read through the same codec (encoded onto its own frame result,
//! then decoded), so both sides carry exactly the fields the codec owns, and
//! the comparison's ranges are the RAM comparison's: each normalized range
//! takes the oracle's bytes on both sides.
//!
//! Both Sessions are taken at the frame's end. The oracle's RAM is the
//! frame-end state; the engine's last lease ends before Game-side code
//! finishes the frame (the NMI's tilemap flush and the controller latch,
//! `gameplay_frame::frame`), so the engine's frame-end Session is the one its
//! next lease takes up: the retained Session while it is current, the kept
//! frame Session with the RAM written since its export, or a decode of its
//! RAM.

use crate::cpu_infra::NORMALIZED_RAM;
use sm_game::room_entry::world::WorldBinding;
use sm_game::session::Session;
use sm_tools::session_compare::SessionSource;
use std::ops::Range;
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

/// The Session the engine's next lease takes up, from which the frame-end
/// Session is read.
pub(super) enum Frame<'a> {
    /// The retained Session, current with RAM.
    Retained(&'a Session),
    /// The Tick's last lease Session, kept for the next frame, and the RAM
    /// ranges written since its export, which the next lease takes from RAM.
    Kept(&'a Session, Vec<Range<usize>>),
    /// The next lease decodes RAM: no Session is current with it. The Tick's
    /// last lease Session, if it released one, is earlier than the frame end.
    Imported(Option<&'a Session>),
}

impl Frame<'_> {
    pub(super) fn source(&self) -> SessionSource {
        match self {
            Self::Retained(_) => SessionSource::Retained,
            Self::Kept(..) => SessionSource::Kept,
            Self::Imported(_) => SessionSource::Imported,
        }
    }
}

/// The engine's Sessions as the codec carries them.
pub(super) struct Engine {
    /// The frame-end Session.
    pub(super) frame_end: Session,
    /// The Tick's last lease Session, when RAM was written after it.
    pub(super) last_lease: Option<Session>,
}

/// Copy the normalized ranges from the oracle's result, as the RAM comparison does.
fn normalize(image: &mut [u8], theirs: &[u8; RAM_BYTES]) {
    for (_, range) in NORMALIZED_RAM {
        image[range.clone()].copy_from_slice(&theirs[range]);
    }
}

/// The engine's Sessions, read through the codec: a lease Session encoded
/// onto the engine's frame result (which supplies the bytes the Session does
/// not own), the RAM written after it taken from that result, with the
/// normalized ranges taken from the oracle, then decoded. `None` when the
/// codec rejects an image.
pub(super) fn engine(
    frame: &Frame<'_>,
    mine: &[u8; RAM_BYTES],
    theirs: &[u8; RAM_BYTES],
    world: &Arc<WorldBinding>,
) -> Option<Engine> {
    quietly(|| {
        let lease = match frame {
            Frame::Retained(session) | Frame::Kept(session, _) => Some(*session),
            Frame::Imported(session) => *session,
        };
        // The lease Session over the engine's frame result.
        let encoded = lease.map(|session| {
            let mut image = mine.to_vec();
            sm_game::session_encoding::encode(session, |at, span| {
                image[at..at + span.len()].copy_from_slice(span);
            });
            image
        });
        let mut frame_end = match (frame, &encoded) {
            (Frame::Retained(_), Some(image)) => image.clone(),
            (Frame::Kept(_, written), Some(image)) => {
                let mut image = image.clone();
                for range in written {
                    image[range.clone()].copy_from_slice(&mine[range.clone()]);
                }
                image
            }
            _ => mine.to_vec(),
        };
        normalize(&mut frame_end, theirs);
        let written_after = match frame {
            Frame::Retained(_) => false,
            Frame::Kept(_, written) => !written.is_empty(),
            Frame::Imported(_) => true,
        };
        let last_lease = encoded.filter(|_| written_after).map(|mut image| {
            normalize(&mut image, theirs);
            decode(&image, world)
        });
        Engine {
            frame_end: decode(&frame_end, world),
            last_lease,
        }
    })
}
