//! A frame of her appearance: the frame's inputs and parts go in, her appearance State and the
//! frame's outputs come out, each rule applied once, where it happens in the frame.

use enum_map::Enum;
use types::Set;

use crate::cannon::CannonEvent;
use crate::colours::{DryColourReset, PaletteOpportunity, ordinary_dry_colours};
use crate::inputs::{Content, Inputs};
use crate::moves::{fast_fall, rising_hold, standing_restart, wall_jump_ready};
use crate::play::play;
use crate::rules::Rules;
use crate::run::{dash_began, dash_lost};
use crate::sounds::{
    GraphicalContact, SoundRequest, at_the_door, foot_contact, spin_end, spin_start,
};
use crate::starting::start;
use crate::state::{AppearanceState, State};

/// A part a frame may have, each at most once, that carries nothing beyond the frame's inputs.
/// Listed in the frame's order.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Enum)]
pub enum FramePart {
    /// The preparation of her work, before her movement (`SpinEnds`). A frame whose work is
    /// locked is still prepared; a paused frame, or a passage's frame that bypasses her work, is
    /// not.
    Preparation,
    /// Her movement: samus-movement's step, her stride in it, and the animation update on its
    /// results (every animation entry, `FootContact`, `SpinStarts`). A frozen frame, at a
    /// passage's door or while paused, has none.
    Movement,
    /// Her first entry into a passage, before the passage holds its sounds (`AtTheDoor`).
    PassageEntry,
    /// Normal drawing, the frame's last (`SpinEnds`).
    Drawing,
}

/// What one frame brings to her appearance: its one set of inputs, and the parts it has. A
/// frame may have any of its parts, each at most once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Each fact of the frame, once, read by every rule that needs it.
    pub inputs: Inputs,
    /// The parts the frame has of those that carry nothing more.
    pub parts: Set<FramePart>,
    /// Her item controls, with their answer (`CannonItemDelay`), or her initialization, which
    /// comes first in its frame and sets her whole appearance State (`CannonReadinessReset`), or
    /// neither.
    pub cannon: CannonEvent,
    /// The palette opportunity, with its own observations, or none (`OrdinaryDryColours`).
    pub palette: PaletteOpportunity,
}

/// What the frame tells, as the whole frame leaves it, beside the appearance State it leaves.
///
/// Her step, the boost stage and her colour sequences are outputs too: each is read from that
/// State, where it lives, as the frame leaves it. The end and turned far are not: they are the
/// read-ahead, `Ahead::of`, asked of her State before the frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameOutputs {
    /// Her stride's graphical contact, for part 2's splashes and dust (`FootContact`).
    pub contact: Option<GraphicalContact>,
    /// Whether her cannon may follow the selected item, as the frame's item controls leave its
    /// delay, for the picture of her gun.
    pub cannon_may_follow: bool,
    /// What the palette opportunity did with the ordinary reset (`OrdinaryDryColours`).
    pub dry_colours: DryColourReset,
    /// The frame's sounds, in the order asked: the stride's, the animation update's, the spin
    /// start's, the passage entry's, then normal drawing's. Several may be asked for in one
    /// frame; none is merged or dropped.
    pub sounds: Vec<SoundRequest>,
}

/// Plays one frame of her appearance from `state`, as the frame's parts happen.
///
/// Spec: `CannonItemDelay`, `CannonReadinessReset`, `FootContact`, `SpinStarts`, `AtTheDoor`,
/// `SpinEnds`, `OrdinaryDryColours`, and the animation's entries in their order:
///
/// 1. Her initialization, first in its frame, sets her whole appearance State as
///    [`AppearanceState::INITIAL`] gives it, whatever the frame starts with; the rest of the
///    frame plays on from it. Or her item controls set the cannon's delay, before her gun's
///    picture reads it.
/// 2. The preparation of her work keeps the action of her pose as the frame starts.
/// 3. Her movement: her stride; her clip plays; the spin start of the pose she takes; that
///    pose's clip starts.
/// 4. The palette opportunity readies her colour sequences.
/// 5. A first passage entry quiets her, then normal drawing ends a kept spin's sound.
#[must_use]
pub fn update(
    state: AppearanceState,
    frame: &Frame,
    content: &impl Content,
    rules: &Rules,
) -> (AppearanceState, FrameOutputs) {
    let (inputs, parts) = (&frame.inputs, &frame.parts);
    let state = initialized(state, frame.cannon);
    let cannon = state.cannon.after(frame.cannon);
    let kept_action = if parts[FramePart::Preparation] {
        inputs.figure.pose.action
    } else {
        state.kept_action
    };
    let mut sounds = Vec::new();
    let (animation, contact) = if parts[FramePart::Movement] {
        move_her(state.animation, inputs, content, rules, &mut sounds)
    } else {
        (state.animation, None)
    };
    let (colours, dry_colours) = ordinary_dry_colours(state.colours, frame.palette, inputs);
    if parts[FramePart::PassageEntry] {
        sounds.extend(at_the_door(inputs));
    }
    if parts[FramePart::Drawing] {
        sounds.extend(spin_end(kept_action, inputs));
    }
    let after = AppearanceState {
        animation,
        kept_action,
        cannon,
        colours,
    };
    let outputs = FrameOutputs {
        contact,
        cannon_may_follow: cannon.may_follow(),
        dry_colours,
        sounds,
    };
    (after, outputs)
}

/// Spec: `CannonReadinessReset`, and her appearance at initialization. A frame with her
/// initialization starts from her State at initialization, whatever State it was given: every
/// quantity, the cannon's delay of 2 among them. Any other frame starts from `state`.
const fn initialized(state: AppearanceState, event: CannonEvent) -> AppearanceState {
    match event {
        CannonEvent::SamusInitialized => AppearanceState::INITIAL,
        CannonEvent::ControlsSkipped | CannonEvent::ControlsRan(_) => state,
    }
}

/// Her movement's part of the frame, in its order, from `at_start`, her animation as the frame
/// starts: her stride (`FootContact`); her clip plays; the spin start of the pose she takes
/// (`SpinStarts`); that pose's clip starts. The sounds asked for join `sounds` in that order.
/// Gives her animation as it leaves it and her stride's graphical contact.
fn move_her(
    at_start: State,
    inputs: &Inputs,
    content: &impl Content,
    rules: &Rules,
    sounds: &mut Vec<SoundRequest>,
) -> (State, Option<GraphicalContact>) {
    let clip = content.clip(&inputs.figure);
    let (contact, footstep) = foot_contact(at_start, inputs, clip);
    sounds.extend(footstep);
    let (played, sound) = play_her_clip(at_start, inputs, content, rules);
    sounds.extend(sound);
    sounds.extend(spin_start(inputs));
    (take_her_pose(played, inputs, content), contact)
}

/// The animation update's first half, in its order: her moves' changes (a dash's beginning, a
/// standing restart, a readied wall jump, a fast fall), the rising hold, then the time runs down
/// and the clip of her pose at the frame's start moves on. Gives the State and the sound asked
/// for, at most one.
fn play_her_clip(
    state: State,
    inputs: &Inputs,
    content: &impl Content,
    rules: &Rules,
) -> (State, Option<SoundRequest>) {
    let clip = content.clip(&inputs.figure);
    let begun = dash_began(state, inputs, rules);
    let playhead = standing_restart(begun.playhead, inputs, rules);
    let playhead = wall_jump_ready(playhead, inputs, clip);
    let playhead = fast_fall(playhead, inputs, clip, rules);
    let playhead = rising_hold(playhead, inputs, clip, rules);
    play(State { playhead, ..begun }, inputs, content, rules)
}

/// The animation update's second half, in its order: the clip of the pose she takes starts
/// (`StartingClip`, `HandOverSlowdown`), then a dash's loss.
fn take_her_pose(played: State, inputs: &Inputs, content: &impl Content) -> State {
    let clip_after = content.clip(&inputs.figure_after);
    let started = State {
        playhead: start(played.playhead, inputs, clip_after),
        ..played
    };
    dash_lost(started, inputs)
}
