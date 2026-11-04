//! The Session-space comparison's two Sessions.
//!
//! Verification owns the translation of RAM into a Session (REVIEW, D6). The
//! oracle's Session is decoded from its RAM with the engine's own codec,
//! `session_encoding::decode`, which moves here once the engine no longer
//! imports RAM. The engine's Session is compared directly: no encode of it
//! reaches the comparison. Two kinds of field are not compared, and the
//! engine's Session takes the oracle's value of each: the fields RAM cannot
//! carry (`sm_tools::session_compare::Uncarried`), and the fields whose bytes
//! a `NORMALIZED_RAM` row still normalizes, until the row retires
//! (`coverage::SHARED_RAM`, `Shared::UntilRowRetires`).
//!
//! Both Sessions are taken at the frame's end. The oracle's RAM is the
//! frame-end state. The engine's frame-end Session is the one its next lease
//! takes up: the retained Session while it is current, the kept frame Session
//! with the RAM written since its export (settlement's Game-side stores), or,
//! when no Session is current with RAM, a decode of its RAM, which is then the
//! only place the engine's frame-end state lives.

use super::coverage::{SHARED_RAM, Shared};
use sm_game::room_entry::world::WorldBinding;
use sm_game::session::Session;
use sm_game::session_encoding::{projected_byte, write_projected_byte};
use sm_tools::session_compare::{Coverage, SessionSource, Uncarried};
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

    /// The Session the frame names, if any: the one coverage marks.
    pub(super) fn session(&self) -> Option<&Session> {
        match self {
            Self::Retained(session) | Self::Kept(session, _) => Some(session),
            Self::Imported(session) => *session,
        }
    }
}

/// The engine's Sessions, ready to compare with the oracle's.
pub(super) struct Engine {
    /// The frame-end Session.
    pub(super) frame_end: Session,
    /// The Tick's last lease Session, when RAM was written after it.
    pub(super) last_lease: Option<Session>,
}

/// Bring `session` up to the bytes `written` holds in `ram`, as the next
/// lease takes them: each byte the Session projects and RAM holds
/// differently goes in as a store of that byte would put it.
fn take_written(session: &mut Session, ram: &[u8], written: &[Range<usize>]) {
    for at in written.iter().flat_map(Clone::clone) {
        let value = ram[at];
        if projected_byte(session, at).is_some_and(|byte| byte != value) {
            write_projected_byte(session, at, value);
        }
    }
}

/// Give `engine` the oracle's value of every field the comparison leaves
/// out: the fields RAM cannot carry, and the bytes of each field a
/// normalized range still covers.
fn leave_out(engine: &mut Session, oracle: &Session) {
    Uncarried::take_all(engine, oracle);
    let normalized = SHARED_RAM
        .iter()
        .filter(|(_, _, shared)| *shared == Shared::UntilRowRetires)
        .flat_map(|(range, _, _)| range.clone());
    for at in normalized {
        if let Some(value) = projected_byte(oracle, at)
            && projected_byte(engine, at) != Some(value)
        {
            write_projected_byte(engine, at, value);
        }
    }
}

/// Mark the encoder's spans of `session` in `coverage` (slice 2's map). The
/// spans do not depend on the Session's state (#223's rows and a decode of
/// zeroed RAM give the same map), so one Session per replay marks them; the
/// map measures the codec and is no part of the comparison.
pub(super) fn mark_coverage(session: &Session, coverage: &mut Coverage) {
    sm_game::session_encoding::encode(session, |at, span| coverage.mark(at..at + span.len()));
}

/// The engine's Sessions at the frame's end, compared directly: the
/// retained or kept Session itself (the kept one with the RAM written since
/// its export), or a decode of the engine's RAM when no Session is current
/// with it; each with the fields the comparison leaves out taken from the
/// oracle's Session. `None` when the codec rejects an image or a byte.
pub(super) fn engine(
    frame: &Frame<'_>,
    mine: &[u8; RAM_BYTES],
    oracle: &Session,
    world: &Arc<WorldBinding>,
) -> Option<Engine> {
    quietly(|| {
        let (mut frame_end, mut last_lease) = match frame {
            Frame::Retained(session) => ((*session).clone(), None),
            Frame::Kept(session, written) => {
                let mut frame_end = (*session).clone();
                take_written(&mut frame_end, mine, written);
                let last_lease = (!written.is_empty()).then(|| (*session).clone());
                (frame_end, last_lease)
            }
            Frame::Imported(session) => (decode(mine, world), session.cloned()),
        };
        leave_out(&mut frame_end, oracle);
        if let Some(last_lease) = &mut last_lease {
            leave_out(last_lease, oracle);
        }
        Engine {
            frame_end,
            last_lease,
        }
    })
}
