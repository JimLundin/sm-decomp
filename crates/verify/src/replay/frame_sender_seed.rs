//! Independently qualified current CPU sender owners at a completed frame return.
//! Outgoing SMP APURAM ports supply replies; CPU output ports have the opposite direction.

use audio_transport::{CpuMasterTicks, CueId, Request, SenderProgress, SenderState, SoundGroup};
use data::assets::audio::CueAlias;
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
    decode_origin(wram, apuram, None)
}

/// Import the checked empty Contact FIFO while the native reply still holds Footstep.
/// The current origin must already establish that no sender service is active.
///
/// # Errors
/// Rejects malformed source, any other progress, and missing or ambiguous cue aliases.
pub fn completed_contact(
    wram: &[u8],
    apuram: &[u8],
    aliases: &[CueAlias],
) -> Result<Senders, SeedFault> {
    decode_origin(wram, apuram, Some(aliases))
}

fn decode_origin(
    wram: &[u8],
    apuram: &[u8],
    held_contact: Option<&[CueAlias]>,
) -> Result<Senders, SeedFault> {
    if wram.len() != 131_072 || apuram.len() != 65_536 {
        return Err(SeedFault::Length);
    }
    let [action, ambient, contact] = [0_usize, 1, 2]
        .map(|group| sender(wram, apuram, group, held_contact.filter(|_| group == 2)));
    Senders::new(action?, ambient?, contact?).map_err(|_| SeedFault::Owner)
}

fn sender(
    wram: &[u8],
    apuram: &[u8],
    group: usize,
    held_contact: Option<&[CueAlias]>,
) -> Result<SenderState, SeedFault> {
    let byte = |at| wram.get(at).copied().ok_or(SeedFault::Length);
    let read = byte(0x0643_usize.saturating_add(group))?;
    let write = byte(0x0646_usize.saturating_add(group))?;
    if read >= 16 || write >= 16 {
        return Err(SeedFault::Cursor);
    }
    let phase = byte(0x0649_usize.saturating_add(group))?;
    if phase != if held_contact.is_some() { 3 } else { 0 } {
        return Err(SeedFault::Progress);
    }
    if read != write {
        return Err(SeedFault::Queue);
    }
    let reply_byte = apuram
        .get(0x00f5_usize.saturating_add(group))
        .copied()
        .ok_or(SeedFault::Length)?;
    let (progress, reply) = if let Some(aliases) = held_contact {
        let mut matches = aliases
            .iter()
            .filter(|alias| alias.group == SoundGroup::Contact && alias.byte == reply_byte);
        if reply_byte == 0
            || matches
                .next()
                .is_none_or(|alias| alias.cue != CueId::Footstep)
            || matches.next().is_some()
        {
            return Err(SeedFault::Reply);
        }
        (SenderProgress::AwaitClear, Request::Cue(CueId::Footstep))
    } else {
        if reply_byte != 0 {
            return Err(SeedFault::Reply);
        }
        (SenderProgress::Ready, Request::Absent)
    };
    Ok(SenderState {
        queue: std::collections::VecDeque::new(),
        progress,
        reply,
        now: CpuMasterTicks(0),
        pending: None,
    })
}
