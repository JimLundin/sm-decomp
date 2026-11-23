//! Spec: `FootContact`, `SpinStarts`, `SpinEnds`, `AtTheDoor`. The domain's sounds, and the
//! frame's sound rules at her stride, the pose she takes, a first passage entry and normal
//! drawing. Each reads the frame's one set of inputs, applies at most once a frame, and neither
//! advances her animation nor changes its quantities.

use types::{Action, BoostStage, Item};

use crate::clip::{Clip, Frames};
use crate::inputs::{Activity, Charge, Figure, Inputs};
use crate::state::{Playhead, State};

/// A sound the domain asks for: a semantic request for the sound domain to admit or not, the
/// same request whichever rule asks for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SoundRequest {
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
}

/// The sound domain's queue a request joins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundQueue {
    /// The footstep sounds.
    Footsteps,
    /// Samus sounds.
    Samus,
}

impl SoundRequest {
    /// The queue it joins. Admission to it, sending, retries and replies are the sound domain's.
    #[must_use]
    pub const fn queue(self) -> SoundQueue {
        match self {
            Self::Footstep | Self::Boosting => SoundQueue::Footsteps,
            Self::PlainSpin
            | Self::SpaceJump
            | Self::ScrewAttack
            | Self::SpinEnded
            | Self::SamusSilence => SoundQueue::Samus,
        }
    }

    /// The most sounds that may already wait for it to join its queue: 6 for the footstep,
    /// boosting and spin starts, 15 for a spin's end and the silence. The sound domain's
    /// admission, not a rule of her look.
    #[must_use]
    pub const fn max_waiting(self) -> u8 {
        match self {
            Self::Footstep
            | Self::Boosting
            | Self::PlainSpin
            | Self::SpaceJump
            | Self::ScrewAttack => 6,
            Self::SpinEnded | Self::SamusSilence => 15,
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

/// Spin and wall jump share one spin sound's lifetime (`SpinStarts`, `SpinEnds`, `AtTheDoor`).
const fn spinning(action: Action) -> bool {
    matches!(action, Action::Spinning | Action::WallJumping)
}

/// Spec: `FootContact`. Her stride, from the frame's start, before the animation update: running,
/// on the last frame (exactly 1 left) of a shown step that `clip`, her figure's as the frame
/// starts, marks as a contact, she touches the ground. That is a graphical contact for part 2,
/// and a `Footstep` unless she is boosting or a cinematic, a boss or special colour time is
/// active. Nothing that quiets the footstep takes the graphical contact away.
#[must_use]
pub fn foot_contact(
    at_start: State,
    inputs: &Inputs,
    clip: &Clip,
) -> (Option<GraphicalContact>, Option<SoundRequest>) {
    let (figure, playhead) = (inputs.figure, at_start.playhead);
    let contact = (figure.pose.action == Action::Running
        && playhead.time_left == Frames::LAST
        && clip.contacts.contains(&playhead.step)
        && clip.hold(playhead.step).is_some())
    .then_some(GraphicalContact { figure, playhead });
    let quiet =
        at_start.stage == BoostStage::Boosting || inputs.activities.values().any(|&active| active);
    let sound = (contact.is_some() && !quiet).then_some(SoundRequest::Footstep);
    (contact, sound)
}

/// Spec: `SpinStarts`. Taking a different figure (its sprite included) that spins, not
/// submerged, starts its spin's sound: `SpaceJump` with the space jump and not the screw attack,
/// from any action and during a cinematic too; otherwise nothing from a spin or a wall jump;
/// otherwise `ScrewAttack` with the screw attack, during a cinematic too; otherwise `PlainSpin`
/// unless a cinematic is active.
#[must_use]
pub fn spin_start(inputs: &Inputs) -> Option<SoundRequest> {
    let (from, takes, items) = (inputs.figure, inputs.figure_after, inputs.items);
    if from == takes || takes.pose.action != Action::Spinning || inputs.liquid.submerged {
        return None;
    }
    if !items[Item::ScrewAttack] && items[Item::SpaceJump] {
        return Some(SoundRequest::SpaceJump);
    }
    if spinning(from.pose.action) {
        return None;
    }
    if items[Item::ScrewAttack] {
        return Some(SoundRequest::ScrewAttack);
    }
    (!inputs.activities[Activity::Cinematic]).then_some(SoundRequest::PlainSpin)
}

/// Spec: `AtTheDoor`. On her first entry into a passage, before it holds its sounds: ending the
/// frame spinning or wall jumping, her spin's sound ends (`SpinEnded`), whatever shoot and the
/// charge; otherwise, with shoot released and the charge below 16 frames, she is quieted
/// (`SamusSilence`).
#[must_use]
pub fn at_the_door(inputs: &Inputs) -> Option<SoundRequest> {
    if spinning(inputs.figure_after.pose.action) {
        return Some(SoundRequest::SpinEnded);
    }
    (!inputs.shoot_held && inputs.charge.frames() < Charge::QUIET_BELOW)
        .then_some(SoundRequest::SamusSilence)
}

/// Spec: `SpinEnds`. Normal drawing, with her animation frozen too: with `kept`, the action her
/// last prepared frame kept, a spin or a wall jump, and her figure as the frame ends doing
/// neither, the spin's sound ends (`SpinEnded`). No other drawing owner's admission is read.
#[must_use]
pub fn spin_end(kept: Action, inputs: &Inputs) -> Option<SoundRequest> {
    (spinning(kept) && !spinning(inputs.figure_after.pose.action))
        .then_some(SoundRequest::SpinEnded)
}
