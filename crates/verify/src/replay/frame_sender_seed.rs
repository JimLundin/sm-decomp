//! Independently qualified current CPU sender owners at a completed frame return.
//! Outgoing SMP APURAM ports supply replies; CPU output ports have the opposite direction.

use audio_transport::{CpuMasterTicks, Request, SenderProgress, SenderState};
use game::production::frame::Senders;

/// Malformed context or a current origin arm outside the independently qualified seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeedFault {
    /// The complete source memory member is absent.
    Length,
    /// A ring cursor does not describe a sixteen-slot queue.
    Cursor,
    /// Current progress requires an unqualified cue or delay interpretation.
    Progress,
    /// A current queued cue requires an unavailable checked native cue mapping.
    Queue,
    /// A current nonempty reply requires an unavailable checked native cue mapping.
    Reply,
    /// The decoded semantic owners fail production admission.
    Owner,
}

/// Import the reached Ready/Empty/Absent origin arms, with no active sender service.
/// The source admission proves the completed caller return, zero earlier SMP port writers,
/// and the reply backing relation. No service, queue offer or reply observation runs here.
///
/// # Errors
/// Incomplete members, malformed queues or other current semantic arms reject admission.
pub fn decode(wram: &[u8], apuram: &[u8]) -> Result<Senders, SeedFault> {
    if wram.len() != 131_072 || apuram.len() != 65_536 {
        return Err(SeedFault::Length);
    }
    let [action, ambient, contact] = [0_usize, 1, 2].map(|group| sender(wram, apuram, group));
    Senders::new(action?, ambient?, contact?).map_err(|_| SeedFault::Owner)
}

fn sender(wram: &[u8], apuram: &[u8], group: usize) -> Result<SenderState, SeedFault> {
    let byte = |at| wram.get(at).copied().ok_or(SeedFault::Length);
    let read = byte(0x0643_usize.saturating_add(group))?;
    let write = byte(0x0646_usize.saturating_add(group))?;
    if read >= 16 || write >= 16 {
        return Err(SeedFault::Cursor);
    }
    if byte(0x0649_usize.saturating_add(group))? != 0 {
        return Err(SeedFault::Progress);
    }
    if read != write {
        return Err(SeedFault::Queue);
    }
    if apuram
        .get(0x00f5_usize.saturating_add(group))
        .copied()
        .ok_or(SeedFault::Length)?
        != 0
    {
        return Err(SeedFault::Reply);
    }
    Ok(SenderState {
        queue: std::collections::VecDeque::new(),
        progress: SenderProgress::Ready,
        reply: Request::Absent,
        now: CpuMasterTicks(0),
        pending: None,
    })
}
