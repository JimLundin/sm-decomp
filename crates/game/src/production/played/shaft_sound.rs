//! The CPU side of the shaft burst's Ambient sender. The platform supplies its reply reads.

use super::super::sender::Sounds;
use audio_transport::{CueId, Offer, QueueAllowance};
use boss_ceres_ridley::OverlayTurn;
use types::audio::{CueQueue, OfferPermission, Request, SenderProgress, SenderState};
use world_doors::{Report, RoomSound};

/// The room's sound permission and the shaft cue group's carried CPU state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// Permission from the passage's hold and release.
    pub permission: Sounds,
    /// The current Ambient FIFO and sender progress.
    pub sender: SenderState,
    /// The request currently presented to audio.
    pub request: Request,
}

impl Default for State {
    fn default() -> Self {
        Self {
            permission: Sounds::Enabled,
            sender: SenderState {
                queue: CueQueue::default(),
                progress: SenderProgress::Ready,
            },
            request: Request::Absent,
        }
    }
}

impl State {
    pub(super) fn next(
        &self,
        turn: Option<OverlayTurn>,
        doors: &Report,
        reply: Option<Request>,
    ) -> Result<(Self, Option<Offer>), super::Fault> {
        let mut next = self.clone();
        if doors.room_sounds[RoomSound::Held] {
            next.permission = Sounds::Disabled;
        }
        let offer = next.offer(turn)?;
        // Brightening can run the actor on the same cut that releases the hold.
        if doors.room_sounds[RoomSound::LetThrough] {
            next.permission = Sounds::Enabled;
        }
        if let Some(reply) = reply {
            audio_transport::serve(&mut next.sender, &mut next.request, reply);
        }
        Ok((next, offer))
    }

    fn offer(&mut self, turn: Option<OverlayTurn>) -> Result<Option<Offer>, super::Fault> {
        let Some(burst) = turn.and_then(|turn| turn.burst) else {
            return Ok(None);
        };
        if burst.sound_limit != 6 {
            return Err(super::Fault::ShaftSoundAllowance);
        }
        let permission = match self.permission {
            Sounds::Enabled => OfferPermission::Enabled,
            Sounds::Disabled => OfferPermission::Disabled,
        };
        let offer = Offer {
            cue: CueId::ShaftExplosion,
            allowance: QueueAllowance::Six,
            permission,
        };
        audio_transport::offer(&mut self.sender.queue, offer);
        Ok(Some(offer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boss_ceres_ridley::{RumbleBurst, RumbleParticle};

    #[test]
    fn a_burst_on_release_still_observes_the_hold() {
        let mut passage = Report::play();
        passage.room_sounds[RoomSound::Held] = true;
        let (held, _) = State::default().next(None, &passage, None).unwrap();
        let turn = Some(OverlayTurn {
            figure: None,
            lights: None,
            burst: Some(RumbleBurst {
                particle: RumbleParticle::Smoke,
                centre: types::Pair {
                    x: core::num::Wrapping(0),
                    y: core::num::Wrapping(0),
                },
                sound_limit: 6,
            }),
            start_rotation: None,
        });
        let mut release = Report::play();
        release.room_sounds[RoomSound::LetThrough] = true;

        let (released, offer) = held.next(turn, &release, Some(Request::Absent)).unwrap();
        assert_eq!(offer.unwrap().permission, OfferPermission::Disabled);
        assert_eq!(released, State::default());

        let (served, offer) = released
            .next(turn, &Report::play(), Some(Request::Absent))
            .unwrap();
        assert_eq!(offer.unwrap().permission, OfferPermission::Enabled);
        assert_eq!(
            served.sender.progress,
            SenderProgress::AwaitCue(CueId::ShaftExplosion)
        );
        assert!(served.sender.queue.cues().is_empty());
        assert_eq!(served.request, Request::Cue(CueId::ShaftExplosion));
    }
}
