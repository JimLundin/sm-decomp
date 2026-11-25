//! Spec: `FootContact`, `SpinStarts`, `SpinEnds`, `AtTheDoor`. The domain's sounds, her spin's
//! sound by her items, and the
//! frame's sound rules at her stride, the pose she takes, a first passage entry and normal
//! drawing. Each reads the frame's one set of inputs, applies at most once a frame, and leaves
//! her animation as it finds it.

use types::{Action, BoostStage, Item, QueueAllowance, Set};

use crate::clip::{Clip, Frames};
use crate::inputs::{Activity, Charge, Figure, Inputs};
use crate::state::{Animation, Playhead};

/// A sound the domain asks for, the same whichever rule asks for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Sound {
    /// A running stride touches the ground (`FootContact`).
    Footstep,
    /// She starts boosting (`BoostStages`).
    Boosting,
    /// A plain spin starts (`WallJumpBranch`, `SpinStarts`).
    PlainSpin,
    /// A spin starts as the space jump (`WallJumpBranch`, `SpinStarts`).
    SpaceJump,
    /// A spin starts as the screw attack (`WallJumpBranch`, `SpinStarts`).
    ScrewAttack,
    /// Her spin's sound ends (`SpinEnds`, `AtTheDoor`).
    SpinEnded,
    /// Her quiet uncharged sound stops at a passage (`AtTheDoor`).
    SamusSilence,
    /// She is hurt: the second frame of her hurt pulse (`HurtColours`).
    Hurt,
}

/// The sound domain's queue a sound is offered to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundQueue {
    /// The footstep sounds.
    Footsteps,
    /// Samus sounds.
    Samus,
}

/// A sound request as the frame reports it: the sound, the queue it is offered to, and its
/// allowance in that queue. The sound domain decides from these whether it is admitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoundOffer {
    /// The sound asked for.
    pub sound: Sound,
    /// The queue it is offered to.
    pub queue: SoundQueue,
    /// Its allowance there, in sounds.
    pub allowance: QueueAllowance,
}

impl Sound {
    /// The request for this sound: the footstep sounds with an allowance of 6 for the footstep
    /// and boosting; Samus sounds with 6 for a spin's start and the hurt sound, and with 15 for a
    /// spin's end and the silence.
    #[must_use]
    pub const fn offer(self) -> SoundOffer {
        SoundOffer {
            sound: self,
            queue: self.queue(),
            allowance: self.allowance(),
        }
    }

    /// The queue it is offered to.
    const fn queue(self) -> SoundQueue {
        match self {
            Self::Footstep | Self::Boosting => SoundQueue::Footsteps,
            Self::PlainSpin
            | Self::SpaceJump
            | Self::ScrewAttack
            | Self::SpinEnded
            | Self::SamusSilence
            | Self::Hurt => SoundQueue::Samus,
        }
    }

    /// Its allowance in that queue.
    const fn allowance(self) -> QueueAllowance {
        match self {
            Self::Footstep
            | Self::Boosting
            | Self::PlainSpin
            | Self::SpaceJump
            | Self::ScrewAttack
            | Self::Hurt => QueueAllowance(6),
            Self::SpinEnded | Self::SamusSilence => QueueAllowance(15),
        }
    }
}

/// The spin she spins, by her items, each with its own sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spin {
    /// A plain spin.
    Plain,
    /// The space jump's spin.
    SpaceJump,
    /// The screw attack's spin.
    ScrewAttack,
}

impl Spin {
    /// Spec: `WallJumpBranch`, `SpinStarts`, `HurtColours`. Her spin by her items: the screw
    /// attack's with it, else the space jump's with it, else a plain one.
    #[must_use]
    pub fn of(items: &Set<Item>) -> Self {
        if items[Item::ScrewAttack] {
            Self::ScrewAttack
        } else if items[Item::SpaceJump] {
            Self::SpaceJump
        } else {
            Self::Plain
        }
    }

    /// The sound its start asks for.
    #[must_use]
    pub const fn sound(self) -> Sound {
        match self {
            Self::Plain => Sound::PlainSpin,
            Self::SpaceJump => Sound::SpaceJump,
            Self::ScrewAttack => Sound::ScrewAttack,
        }
    }
}

/// Spec: `FootContact`. Her stride's contact with the ground, for part 2's splashes and dust,
/// whether or not its footstep is heard.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GraphicalContact {
    /// The figure she runs in, as the frame starts.
    pub figure: Figure,
    /// The contact picture's step and time left, before her clip plays this frame.
    pub playhead: Playhead,
}

/// Spec: `FootContact`. Her stride, from the frame's start, before the animation update: running,
/// on the last frame (exactly 1 left) of a shown step that `clip`, her figure's as the frame
/// starts, marks as a contact, she touches the ground. That is a graphical contact for part 2,
/// and a `Footstep` unless she is boosting or a cinematic, a boss or special colour time is
/// active. The graphical contact stays whatever quiets the footstep.
#[must_use]
pub fn foot_contact(
    at_start: Animation,
    inputs: &Inputs,
    clip: &Clip,
) -> (Option<GraphicalContact>, Option<Sound>) {
    let (figure, playhead) = (inputs.figure, at_start.playhead);
    let contact = (figure.pose.action == Action::Running
        && playhead.time_left == Frames::LAST
        && clip.contacts.contains(&playhead.step)
        && clip.hold(playhead.step).is_some())
    .then_some(GraphicalContact { figure, playhead });
    let quiet =
        at_start.stage == BoostStage::Boosting || inputs.activities.values().any(|&active| active);
    let sound = (contact.is_some() && !quiet).then_some(Sound::Footstep);
    (contact, sound)
}

/// Spec: `SpinStarts`. Taking a different figure (its sprite included) that spins, unless she is
/// submerged, starts the noise her kind of spin makes (`Spin::of`): the space jump's from any
/// action and during a cinematic too; the screw attack's unless she already spins or wall jumps,
/// during a cinematic too; a plain spin's unless she already spins or wall jumps, or a cinematic
/// is active.
#[must_use]
pub fn spin_start(inputs: &Inputs) -> Option<Sound> {
    let (from, takes) = (inputs.figure, inputs.figure_after);
    if from == takes || takes.pose.action != Action::Spinning || inputs.liquid.submerged {
        return None;
    }
    let already_spinning = from.pose.action.spins();
    let spin = Spin::of(&inputs.items);
    let quiet = match spin {
        Spin::SpaceJump => false,
        Spin::ScrewAttack => already_spinning,
        Spin::Plain => already_spinning || inputs.activities[Activity::Cinematic],
    };
    (!quiet).then_some(spin.sound())
}

/// Spec: `AtTheDoor`. On her first entry into a passage, before it holds its sounds: ending the
/// frame spinning or wall jumping, her spin's sound ends (`SpinEnded`), whatever shoot and the
/// charge; otherwise, with shoot released and the charge below 16 frames, she is quieted
/// (`SamusSilence`).
#[must_use]
pub fn at_the_door(inputs: &Inputs) -> Option<Sound> {
    if inputs.figure_after.pose.action.spins() {
        return Some(Sound::SpinEnded);
    }
    (!inputs.shoot_held && inputs.charge.frames() < Charge::QUIET_BELOW)
        .then_some(Sound::SamusSilence)
}

/// Spec: `SpinEnds`. Normal drawing, with her animation frozen too: with `kept`, the action her
/// last prepared frame kept, a spin or a wall jump, and her figure as the frame ends doing
/// neither, the spin's sound ends (`SpinEnded`).
#[must_use]
pub fn spin_end(kept: Action, inputs: &Inputs) -> Option<Sound> {
    (kept.spins() && !inputs.figure_after.pose.action.spins()).then_some(Sound::SpinEnded)
}
