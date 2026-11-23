//! Spec: `FootContact`, `SpinStarts`, `SpinEnds`, `AtTheDoor`. The frame's sound opportunities
//! at her stride, her pose selection, a first passage entry and normal drawing. Each has inputs
//! of its own, applies at most once a frame, and neither advances her animation nor changes its
//! State.

use enum_map::Enum;
use types::{Action, BoostStage, Item, Set};

use crate::clip::{Clip, Frames};
use crate::inputs::Figure;
use crate::state::{Playhead, State};

/// Spec: `FootContact`. Owners that suppress contact sound, never its graphics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, serde::Serialize, serde::Deserialize)]
pub enum ContactSuppressor {
    /// A cinematic is active.
    Cinematic,
    /// A boss is active.
    Boss,
    /// Special colour time remains.
    SpecialColours,
}

/// Spec: `SpinStarts`. Cinematic activity supplied by its owner, independent of selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CinematicActivity {
    /// No cinematic is active.
    Inactive,
    /// A cinematic is active.
    Active,
}

/// Spec: `AtTheDoor`. Weapon charge in frames, 0 to 32767.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "u16")]
pub struct Charge(u16);

impl TryFrom<u16> for Charge {
    type Error = &'static str;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value).ok_or("charge must be from 0 to 32767 frames")
    }
}

impl Charge {
    /// Largest charge supplied by weapons.
    pub const MOST: u16 = 32767;
    /// The quiet-sound cutoff, 16 frames.
    pub const QUIET_CUTOFF: u16 = 16;

    /// Accept only the specified charge range.
    #[must_use]
    pub const fn new(frames: u16) -> Option<Self> {
        if frames <= Self::MOST {
            Some(Self(frames))
        } else {
            None
        }
    }

    /// Frames supplied by the weapon owner.
    #[must_use]
    pub const fn frames(self) -> u16 {
        self.0
    }
}

/// Spec: `FootContact`, `SpinStarts`, `SpinEnds`, `AtTheDoor`. Meaning, not driver commands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SoundRequest {
    /// Ground contact of a running stride.
    Footstep,
    /// Start a plain spin.
    PlainSpin,
    /// Start a space jump spin.
    SpaceJump,
    /// Start a screw attack spin.
    ScrewAttack,
    /// End the spin sound.
    SpinEnded,
    /// Quiet Samus's uncharged sound at passage entry.
    SamusSilence,
}

/// Spec: `FootContact`, `SpinStarts`, `SpinEnds`, `AtTheDoor`. Sound-domain destination.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundQueue {
    /// Footstep sounds.
    Footsteps,
    /// Samus sounds.
    Samus,
}

impl SoundRequest {
    /// The sound domain owns admission to this queue, sending, retries and replies.
    #[must_use]
    pub const fn queue(self) -> SoundQueue {
        match self {
            Self::Footstep => SoundQueue::Footsteps,
            Self::PlainSpin
            | Self::SpaceJump
            | Self::ScrewAttack
            | Self::SpinEnded
            | Self::SamusSilence => SoundQueue::Samus,
        }
    }

    /// Spec: `FootContact`, `SpinStarts`, `SpinEnds`, `AtTheDoor`. Maximum already waiting
    /// for admission: 6 for contact/start, 15 for end/silence. This is not an appearance guard.
    #[must_use]
    pub const fn max_waiting(self) -> u8 {
        match self {
            Self::Footstep | Self::PlainSpin | Self::SpaceJump | Self::ScrewAttack => 6,
            Self::SpinEnded | Self::SamusSilence => 15,
        }
    }
}

/// Spec: `FootContact`. Available to part 2 for dust/splash decisions, even with no sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GraphicalContact {
    /// Figure at the running movement opportunity.
    pub figure: Figure,
    /// Contact picture's playhead, before animation advances.
    pub playhead: Playhead,
}

/// Spin and wall jump share one spin sound's lifetime (`SpinStarts`, `SpinEnds`, `AtTheDoor`).
const fn spinning(action: Action) -> bool {
    matches!(action, Action::Spinning | Action::WallJumping)
}

/// Spec: `FootContact`. The frame's stride, before the animation update.
///
/// It holds her figure as the frame starts, after her running movement, and the owners active
/// then. Her step, the time left and the boost stage are her State's as the frame starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stride {
    /// Her figure, with its sprite; its clip marks the contact steps.
    pub figure: Figure,
    /// The active owners that quiet the footstep; every combination is valid.
    pub suppressors: Set<ContactSuppressor>,
}

impl Stride {
    /// Spec: `FootContact`. Running, on the last frame (exactly 1 left) of a shown step her clip
    /// marks as a contact, she touches the ground: a graphical contact for part 2, and a
    /// `Footstep` unless a suppressor is active or she is boosting. A suppressor never takes
    /// the graphical contact away.
    pub(crate) fn contact(
        self,
        state: State,
        clip: &Clip,
    ) -> (Option<GraphicalContact>, Option<SoundRequest>) {
        let playhead = state.playhead;
        let contact = (self.figure.pose.action == Action::Running
            && playhead.time_left == Frames::LAST
            && clip.contacts.contains(&playhead.step)
            && clip.hold(playhead.step).is_some())
        .then_some(GraphicalContact {
            figure: self.figure,
            playhead,
        });
        let quiet =
            state.stage == BoostStage::Boosting || self.suppressors.values().any(|&active| active);
        let sound = (contact.is_some() && !quiet).then_some(SoundRequest::Footstep);
        (contact, sound)
    }
}

/// Spec: `SpinStarts`. The frame's pose selection: the figure she starts the frame in and the
/// one she ends it in, each with its sprite, and the facts at selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    /// Her figure as the frame starts.
    pub from: Figure,
    /// The figure selected, the one she ends the frame in.
    pub selected: Figure,
    /// The items switched on.
    pub items: Set<Item>,
    /// Whether she is submerged (the liquids).
    pub submerged: bool,
    /// The cinematic owner's activity.
    pub cinematic: CinematicActivity,
}

impl Selection {
    /// Spec: `SpinStarts`. A different figure selected, spinning, not submerged, starts its
    /// spin's sound: `SpaceJump` with the space jump and not the screw attack, from any action and
    /// during a cinematic too; otherwise nothing from a spin or a wall jump; otherwise
    /// `ScrewAttack` with the screw attack, during a cinematic too; otherwise `PlainSpin` unless
    /// a cinematic is active.
    pub(crate) fn spin_start(self) -> Option<SoundRequest> {
        let (from, selected, items) = (self.from, self.selected, self.items);
        if from == selected || selected.pose.action != Action::Spinning || self.submerged {
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
        (self.cinematic == CinematicActivity::Inactive).then_some(SoundRequest::PlainSpin)
    }
}

/// Spec: `AtTheDoor`. The frame in which she first enters a passage, before the passage holds
/// its sounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PassageEntry {
    /// Her action now (samus-movement's).
    pub current: Action,
    /// Whether the pad holds shoot.
    pub shoot_held: bool,
    /// The weapons' charge.
    pub charge: Charge,
}

impl PassageEntry {
    /// Spec: `AtTheDoor`. Spinning or wall jumping, her spin sound ends (`SpinEnded`), whatever
    /// shoot and the charge; otherwise, with shoot released and the charge below 16 frames,
    /// Samus is quieted (`SamusSilence`).
    pub(crate) fn quieting(self) -> Option<SoundRequest> {
        if spinning(self.current) {
            return Some(SoundRequest::SpinEnded);
        }
        (!self.shoot_held && self.charge.frames() < Charge::QUIET_CUTOFF)
            .then_some(SoundRequest::SamusSilence)
    }
}

/// Spec: `SpinEnds`. The frame's normal drawing, which happens with her animation frozen too.
///
/// No engine-admission predicate of another drawing owner is an input here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NormalDrawing {
    /// The action the caller kept when the frame's work was last prepared, whatever the pose
    /// changes since; not her figure as the frame starts.
    pub previous: Action,
    /// Her action as the frame ends (samus-movement's).
    pub current: Action,
}

impl NormalDrawing {
    /// Spec: `SpinEnds`. Having left a spin or a wall jump for any other action ends its sound
    /// (`SpinEnded`).
    pub(crate) fn spin_end(self) -> Option<SoundRequest> {
        (spinning(self.previous) && !spinning(self.current)).then_some(SoundRequest::SpinEnded)
    }
}
