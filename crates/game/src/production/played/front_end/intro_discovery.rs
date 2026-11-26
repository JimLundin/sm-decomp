//! Live discovery playback, room motion and encounter publication.

use super::super::{discovery, intro::Fault, intro_rooms as rooms};
use super::ActorsContent;
use crate::production::playback::{self, Playback};
use screens_intro::DiscoveryReply;
use types::{Pair, Parity, Parts};

/// The recorded controller and live encounter selected by the page 2 press.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveryRoom {
    playback: Playback,
    encounter: discovery::DiscoveryState,
    /// The encounter's most recent completed publication.
    pub publication: Option<discovery::Publication>,
}

/// Production publications from a discovery room turn.
pub(super) struct DiscoveryTurn {
    pub(super) reply: DiscoveryReply,
    pub(super) sounds: Vec<super::Sound>,
}

impl DiscoveryRoom {
    pub(super) fn start(room: &mut rooms::State, content: &ActorsContent) -> Result<Self, Fault> {
        let art = content.discovery.as_ref().ok_or(Fault::DiscoveryRoom)?;
        room.start(
            rooms::Start::Discovery,
            &content.room.rooms,
            &content.room.appearance,
        )
        .map_err(Fault::IntroRoom)?;
        let mut playback = Playback::new(content.pad.clone());
        if playback.start(playback::SceneStart::Discovery)
            != Some(playback::Handoff::StartDiscovery)
        {
            return Err(Fault::IntroControls);
        }
        let mut encounter = discovery::DiscoveryState::dormant();
        let publication = art.turn(
            &mut encounter,
            discovery::Cut::Unfinished,
            discovery::IntroRequest::StartDiscovery,
            || Err(Fault::IntroRoomTurn),
        )?;
        Ok(Self {
            playback,
            encounter,
            publication: Some(publication),
        })
    }

    /// The encounter's carried state.
    #[must_use]
    pub const fn encounter(&self) -> &discovery::DiscoveryState {
        &self.encounter
    }

    pub(super) fn turn(
        &mut self,
        room: &mut rooms::State,
        dissolve: u16,
        closes: bool,
        parity: Parity,
        motion: &rooms::Motion<'_>,
        content: &ActorsContent,
    ) -> Result<DiscoveryTurn, Fault> {
        let art = content.discovery.as_ref().ok_or(Fault::DiscoveryRoom)?;
        let turn = self.playback.turn(
            playback::WholeX(room.actor.samus.movement.0.position.x.pixel().0),
            playback::CrossfadeCount(dissolve),
        );
        let request = if closes {
            discovery::IntroRequest::FinishDiscovery
        } else {
            discovery::IntroRequest::None
        };
        let mut sounds = Vec::new();
        let publication = art.turn(
            &mut self.encounter,
            discovery::Cut::Finished,
            request,
            || {
                match (turn.authority, turn.pad, turn.handoff) {
                    (playback::Authority::Controlled, Some(pad), None) => {
                        room.actor
                            .sample_body(motion.movement)
                            .map_err(Fault::IntroRoom)?;
                        sounds = room
                            .actor_turn(
                                rooms::DemoPad {
                                    held: playback::buttons(pad.current.held),
                                    pressed: playback::buttons(pad.current.pressed),
                                },
                                parity,
                                &content.room.rooms,
                                motion,
                                true,
                            )
                            .map_err(Fault::IntroRoom)?
                            .sounds;
                    }
                    (playback::Authority::Releasing, Some(_), Some(playback::Handoff::Release))
                        if closes =>
                    {
                        room.actor.carry.0.demo = rooms::DemoPad::default();
                    }
                    _ => return Err(Fault::IntroControls),
                }
                let position = room.actor.samus.movement.0.position;
                Ok(discovery::RoomView {
                    centre: Pair {
                        x: i32::from(position.x.pixel().0),
                        y: i32::from(position.y.pixel().0),
                    },
                    scroll: Pair { x: 0, y: 0 },
                })
            },
        )?;
        let reply = if publication.cue.is_some() {
            DiscoveryReply::ReadyForPageThree
        } else {
            DiscoveryReply::Playing
        };
        let sounds = sounds
            .into_iter()
            .map(super::Sound::Appearance)
            .chain(
                publication
                    .sounds
                    .iter()
                    .copied()
                    .map(super::Sound::Discovery),
            )
            .collect();
        self.publication = Some(publication);
        Ok(DiscoveryTurn { reply, sounds })
    }
}
