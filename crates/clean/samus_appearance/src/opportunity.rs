//! Spec: `FootContact`, `SpinStarts`, `SpinEnds`, `AtTheDoor`.
//! Independent reads at movement, selection, drawing and first passage entry. No retained
//! animation or sound-driver state is changed; the caller retains outputs in opportunity order.

use enum_map::Enum;
use types::{Action, BoostStage, Item, Set};

use crate::clip::{Clip, Frames};
use crate::inputs::Figure;
use crate::state::Playhead;

/// Spec: `FootContact`. Owners that suppress contact sound, never its graphics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum ContactSuppressor {
    /// A cinematic is active.
    Cinematic,
    /// A boss is active.
    Boss,
    /// Special colour time remains.
    SpecialColours,
}

/// Spec: `SpinStarts`. Cinematic activity supplied by its owner, independent of selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CinematicActivity {
    /// No cinematic is active.
    Inactive,
    /// A cinematic is active.
    Active,
}

/// Spec: `AtTheDoor`. Weapon charge in frames, 0 to 32767.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Charge(u16);

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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphicalContact {
    /// Figure at the running movement opportunity.
    pub figure: Figure,
    /// Contact picture's playhead, before animation advances.
    pub playhead: Playhead,
}

/// Spec: `FootContact`, `SpinStarts`, `SpinEnds`, `AtTheDoor`. Outputs of one read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpportunityOutput {
    /// Graphical contact retained independently of sound suppression.
    pub contact: Option<GraphicalContact>,
    /// Semantic request, if the named opportunity asks for one.
    pub sound: Option<SoundRequest>,
}

/// Spec: `FootContact`, `SpinStarts`, `SpinEnds`, `AtTheDoor`. The caller invokes each actual
/// opportunity once.
///
/// Every supplied action/guard combination has an answer, including drawing
/// with frozen animation. Engine capability predicates for other drawing owners are not inputs.
#[derive(Clone, Copy, Debug)]
pub enum Opportunity<'a> {
    /// Read after running position changes and before the animation update.
    Contact {
        /// Stride figure, including sprite.
        figure: Figure,
        /// Playhead at the stride opportunity, not a later animation result.
        playhead: Playhead,
        /// Retained boost stage at that opportunity.
        stage: BoostStage,
        /// Importer-supplied instructions and contact marks for the stride figure.
        clip: &'a Clip,
        /// Active sound suppressors; all combinations are valid.
        suppressors: Set<ContactSuppressor>,
    },
    /// Read each actual pose selection, without another animation or movement step.
    PoseSelected {
        /// Figure immediately before selection.
        from: Figure,
        /// Selected figure, with sprite; equality includes every part.
        selected: Figure,
        /// Switched-on items at selection.
        items: Set<Item>,
        /// Liquid owner's selection-time predicate.
        submerged: bool,
        /// Cinematic owner's selection-time predicate.
        cinematic: CinematicActivity,
    },
    /// Read normal drawing, including passage quieting while motion and animation are frozen.
    NormalDraw {
        /// Remembered prior movement from samus-movement, not inferred from clip changes.
        previous: Action,
        /// Current movement after selection.
        current: Action,
    },
    /// Read once on first passage entry, before the sound hold, separately from normal drawing.
    PassageEntry {
        /// Current movement.
        current: Action,
        /// Pad's shoot action.
        shoot_held: bool,
        /// Weapon owner's charge, with no engine-admission restriction to zero.
        charge: Charge,
    },
}

impl Opportunity<'_> {
    /// Evaluate only this named opportunity. No animation state is taken mutably or advanced.
    #[must_use]
    pub fn read(self) -> OpportunityOutput {
        let sound = match self {
            Self::Contact {
                figure,
                playhead,
                stage,
                clip,
                suppressors,
            } => {
                return foot_contact(figure, playhead, stage, clip, suppressors);
            }
            Self::PoseSelected {
                from,
                selected,
                items,
                submerged,
                cinematic,
            } => spin_starts(from, selected, &items, submerged, cinematic),
            Self::NormalDraw { previous, current } => spin_ends(previous, current),
            Self::PassageEntry {
                current,
                shoot_held,
                charge,
            } => at_the_door(current, shoot_held, charge),
        };
        OpportunityOutput {
            contact: None,
            sound,
        }
    }
}

/// Spec: `FootContact`. Last frame of a marked shown running picture yields graphics;
/// cinematic, boss, colours and boosting suppress only the footstep request.
fn foot_contact(
    figure: Figure,
    playhead: Playhead,
    stage: BoostStage,
    clip: &Clip,
    suppressors: Set<ContactSuppressor>,
) -> OpportunityOutput {
    let contact = (figure.pose.action == Action::Running
        && playhead.time_left == Frames::LAST
        && clip.contacts.contains(&playhead.step)
        && clip.hold(playhead.step).is_some())
    .then_some(GraphicalContact { figure, playhead });
    let sound = (contact.is_some()
        && stage != BoostStage::Boosting
        && !suppressors.values().any(|&active| active))
    .then_some(SoundRequest::Footstep);
    OpportunityOutput { contact, sound }
}

/// Spec: `SpinStarts`. Different, unsubmerged spinning figure: space without screw always
/// sounds; screw only on entry; plain only on noncinematic entry. Continuing is movement-based.
fn spin_starts(
    from: Figure,
    selected: Figure,
    items: &Set<Item>,
    submerged: bool,
    cinematic: CinematicActivity,
) -> Option<SoundRequest> {
    if from == selected || selected.pose.action != Action::Spinning || submerged {
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
    (cinematic == CinematicActivity::Inactive).then_some(SoundRequest::PlainSpin)
}

/// Spin and wall jump share the sound's lifetime (`SpinStarts`, `SpinEnds`, `AtTheDoor`).
const fn spinning(action: Action) -> bool {
    matches!(action, Action::Spinning | Action::WallJumping)
}

/// Spec: `SpinEnds`. Normal drawing ends sound when remembered spin/wall becomes neither.
fn spin_ends(previous: Action, current: Action) -> Option<SoundRequest> {
    (spinning(previous) && !spinning(current)).then_some(SoundRequest::SpinEnded)
}

/// Spec: `AtTheDoor`. Spin takes priority over shoot/charge; otherwise silence only when
/// shoot is released and charge is below 16, at the first passage opportunity.
fn at_the_door(current: Action, shoot_held: bool, charge: Charge) -> Option<SoundRequest> {
    if spinning(current) {
        return Some(SoundRequest::SpinEnded);
    }
    (!shoot_held && charge.frames() < Charge::QUIET_CUTOFF).then_some(SoundRequest::SamusSilence)
}
