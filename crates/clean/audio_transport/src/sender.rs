//! Spec: OfferAcceptance, RequestDelivery and RequestRelease. The game's side of the transport:
//! its sound queues and the sender service that presents their cues on the request lines.
use std::num::NonZeroU8;
use types::audio::{
    CueAllowance as QueueAllowance, CueId, CueQueue, OfferPermission, Request, SenderProgress,
    SenderState, count_down,
};
/// Spec: RequestDelivery. Services an acknowledged cue waits before its line is cleared.
const CLEAR_DELAY_TURNS: u8 = 2;
/// Spec: OfferAcceptance. A producer's offer of one cue to its group's queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Offer {
    /// The cue offered; its group names the queue.
    pub cue: CueId,
    /// How long the queue may already be for the offer to be admitted.
    pub allowance: QueueAllowance,
    /// Whether the producer permits sound now.
    pub permission: OfferPermission,
}
/// Spec: OfferAcceptance. Admit an offer only while sound is permitted and the queue is shorter
/// than the offer's allowance. An offer never removes or reorders accepted cues.
///
/// The caller selects the carried queue using the offered cue's group.
pub fn offer(queue: &mut CueQueue, offer: Offer) {
    let allowed = match offer.allowance {
        QueueAllowance::One => 1,
        QueueAllowance::Three => 3,
        QueueAllowance::Six => 6,
        QueueAllowance::Nine => 9,
        QueueAllowance::Fifteen => 15,
    };
    if offer.permission == OfferPermission::Enabled && queue.len() < allowed {
        queue.push(offer.cue);
    }
}
/// Spec: RequestDelivery and RequestRelease. Serve one group's queue once: read the group's last
/// published reply and, where the rule presents a value, put it on the group's request line.
///
/// Pass the sender, line and reply for the same group. Keep the updated sender and line to resume
/// at the next service; this operation does not advance effects or music.
pub fn serve(sender: &mut SenderState, line: &mut Request, reply: Request) {
    match sender.progress {
        SenderProgress::Ready => deliver(sender, line),
        SenderProgress::AwaitCue(cue) => {
            if reply == Request::Cue(cue) {
                // The new delay is not consumed in the service that starts it.
                sender.progress = SenderProgress::ClearDelay(
                    NonZeroU8::new(CLEAR_DELAY_TURNS).unwrap_or(NonZeroU8::MIN),
                );
            } else {
                *line = Request::Cue(cue);
            }
        }
        SenderProgress::ClearDelay(turns) => {
            if let Some(left) = count_down(turns) {
                sender.progress = SenderProgress::ClearDelay(left);
            } else {
                *line = Request::Absent;
                sender.progress = SenderProgress::AwaitClear;
            }
        }
        SenderProgress::AwaitClear => {
            if reply == Request::Absent {
                // Clear acceptance and the next cue's presentation share this service.
                deliver(sender, line);
            } else {
                *line = Request::Absent;
            }
        }
        SenderProgress::ResetDelay(turns) => {
            // Once the delay expires, queued cues wait for the next service.
            sender.progress =
                count_down(turns).map_or(SenderProgress::Ready, SenderProgress::ResetDelay);
        }
    }
}
/// Spec: RequestDelivery. Present the oldest queued cue and wait for it, or stay ready.
fn deliver(sender: &mut SenderState, line: &mut Request) {
    sender.progress = sender
        .queue
        .pop_oldest()
        .map_or(SenderProgress::Ready, |cue| {
            *line = Request::Cue(cue);
            SenderProgress::AwaitCue(cue)
        });
}
