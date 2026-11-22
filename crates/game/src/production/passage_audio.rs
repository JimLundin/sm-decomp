//! The touched doorway's sound calls through the first hold.

use super::{
    AdmissionFault, ProductionFault, Unsupported,
    gameplay_sounds::{GameplaySounds, Offer},
};
use audio_transport::{CueId, QueueAllowance};
use canonical::Canonical;
use data::assets::world::Doorway;
use samus_appearance::{Charge, Opportunity, SoundQueue, SoundRequest};
use types::{Action, Pair};
use world_doors::{Arrival, Doors, HerSound, Inputs, Passage, Phase, RoomSound, Rules, Scene};

/// The admitted doorway, its retained scene, and the scoped first-work dependencies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Content {
    /// Scene retained at the previous native return.
    pub scene: Scene,
    /// Imported arrival for the touched door.
    pub doorway: Doorway,
    /// Independently admitted first-entry work inputs.
    pub inputs: Inputs,
    /// Retained movement consumed by Appearance.
    pub movement: Action,
    /// Weapon charge owned by the previous interval.
    pub charge: Charge,
}

/// Scoped advance boundary for the first passage calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Advance {
    /// One caller operation, allowing a save between calls.
    Next(ShootHeld),
    /// The first entry through Hold, inclusive.
    ThroughHold(ShootHeld),
}

/// Sampled shoot action when the passage calls Appearance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShootHeld(pub bool);

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Touched,
    Ambient([u16; world_doors::COLOURS]),
    Hold([u16; world_doors::COLOURS]),
    Quieting([u16; world_doors::COLOURS]),
}

/// The actual completed caller operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Output {
    /// Appearance's first passage opportunity, possibly silent.
    Action(Option<Box<Offer>>),
    /// The doorway's background quieting offer.
    Ambient(Box<Offer>),
    /// The doorway set the sound hold after both opportunities.
    Held,
}

/// Complete sender owners and pending passage sound operations.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    stage: Stage,
    audio: GameplaySounds,
}

impl Content {
    fn arrival(self) -> Arrival {
        let given = self.doorway.arrival;
        Arrival {
            way: given.way,
            view: Pair {
                x: given.view.x.pixels(),
                y: given.view.y.pixels(),
            },
            speed: given.speed,
        }
    }
}

impl State {
    /// Admit at the touched doorway, before the first entry work.
    #[must_use]
    pub const fn touched(audio: GameplaySounds) -> Self {
        Self {
            stage: Stage::Touched,
            audio,
        }
    }

    /// The independently admitted sender owners after completed operations.
    #[must_use]
    pub const fn audio(&self) -> &GameplaySounds {
        &self.audio
    }

    pub(super) fn validate(&self, content: &Content) -> Result<(), AdmissionFault> {
        if content.inputs.doorway != Some(content.arrival())
            || match self.stage {
                Stage::Touched => false,
                Stage::Ambient(goal) | Stage::Hold(goal) | Stage::Quieting(goal) => goal
                    .into_iter()
                    .any(|bits| types::Colour::new(bits).is_none()),
            }
        {
            return Err(AdmissionFault::WrongScope);
        }
        self.audio.validate()
    }

    pub(super) fn advance(
        &mut self,
        content: &Content,
        rules: &Rules,
        input: Advance,
    ) -> Result<Vec<Output>, ProductionFault> {
        match input {
            Advance::Next(shoot) => self.next(content, rules, shoot).map(|output| vec![output]),
            Advance::ThroughHold(shoot) if self.stage == Stage::Touched => Ok(vec![
                self.next(content, rules, shoot)?,
                self.next(content, rules, shoot)?,
                self.next(content, rules, shoot)?,
            ]),
            Advance::ThroughHold(_) => Err(ProductionFault::Unsupported(Unsupported::PassageAudio)),
        }
    }

    fn next(
        &mut self,
        content: &Content,
        rules: &Rules,
        shoot: ShootHeld,
    ) -> Result<Output, ProductionFault> {
        match self.stage {
            Stage::Touched => self.first_entry(content, rules, shoot),
            Stage::Ambient(goal) => {
                let offer = self
                    .audio
                    .offer(CueId::BackgroundSilence, QueueAllowance::Fifteen)?;
                self.stage = Stage::Hold(goal);
                Ok(Output::Ambient(Box::new(offer)))
            }
            Stage::Hold(goal) => {
                self.audio.hold();
                self.stage = Stage::Quieting(goal);
                Ok(Output::Held)
            }
            Stage::Quieting(_) => Err(ProductionFault::Unsupported(Unsupported::SenderService)),
        }
    }

    fn first_entry(
        &mut self,
        content: &Content,
        rules: &Rules,
        shoot: ShootHeld,
    ) -> Result<Output, ProductionFault> {
        let output = Doors::Passing(Passage::START).step(content.scene, &content.inputs, rules);
        let asked = Passage {
            phase: Phase::Quieting { asked: true },
            ..Passage::START
        };
        if output.doors != Doors::Passing(asked)
            || output.scene.her != content.scene.her
            || output.scene.seen != content.scene.seen
            || output.scene.camera != content.scene.camera
            || output.scene.palette.screen != content.scene.palette.screen
            || output.report.her_sound != Some(HerSound::AtTheDoor)
            || output.report.room_sounds != types::set_of([RoomSound::Silence, RoomSound::Held])
        {
            return Err(ProductionFault::Unsupported(Unsupported::PassageAudio));
        }
        let opportunity = Opportunity::PassageEntry {
            current: content.movement,
            shoot_held: shoot.0,
            charge: content.charge,
        }
        .read();
        let offer = if let Some(sound) = opportunity.sound {
            if sound != SoundRequest::SamusSilence
                || sound.queue() != SoundQueue::Samus
                || sound.max_waiting() != 15
            {
                return Err(ProductionFault::Unsupported(Unsupported::PassageAudio));
            }
            Some(Box::new(
                self.audio
                    .offer(CueId::SamusSilence, QueueAllowance::Fifteen)?,
            ))
        } else {
            None
        };
        self.stage = Stage::Ambient(output.scene.palette.goal.map(u16::from));
        Ok(Output::Action(offer))
    }
}
