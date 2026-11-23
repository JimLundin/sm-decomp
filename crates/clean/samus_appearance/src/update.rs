//! A frame of her appearance: the frame's inputs go in, her appearance State and the frame's
//! outputs come out, each rule applied once, where it happens in the frame.

use types::AnimationSound;

use crate::ahead::Ahead;
use crate::cannon::CannonEvent;
use crate::colours::{DryColourReset, PaletteOpportunity, ordinary_dry_colours};
use crate::inputs::{Content, Inputs};
use crate::moves::{fast_fall, rising_hold, standing_restart, wall_jump_ready};
use crate::opportunity::{
    GraphicalContact, NormalDrawing, PassageEntry, Selection, SoundRequest, Stride,
};
use crate::play::play;
use crate::rules::Rules;
use crate::run::{dash_began, dash_lost};
use crate::starting::start;
use crate::state::{AppearanceState, State};

/// What one frame brings to her appearance. Each part is one of the frame's opportunities, with
/// the inputs its owners give it; a part the frame doesn't have is left out.
///
/// A frame may have any of them: one whose animation is frozen, at a passage's door or while
/// paused, has no animation update and may still draw, run her item controls or meet a palette
/// opportunity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    /// What the frame does to her cannon's delay: its item controls, or her initialization.
    pub cannon: CannonEvent,
    /// Her stride, after her running movement and before the animation update (`FootContact`).
    pub stride: Option<Stride>,
    /// The animation update, with samus-movement's results and the pose she takes. A frozen
    /// frame has none.
    pub animation: Option<Inputs>,
    /// Her pose selection (`SpinStarts`).
    pub selection: Option<Selection>,
    /// The palette opportunity, after movement and the suit selection (`OrdinaryDryColours`).
    pub palette: PaletteOpportunity,
    /// Her first entry into a passage, before it holds its sounds (`AtTheDoor`).
    pub passage: Option<PassageEntry>,
    /// The frame's normal drawing (`SpinEnds`).
    pub drawing: Option<NormalDrawing>,
}

/// A sound the frame asks for, for the sound domain to admit or not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameSound {
    /// Her animation update's (`WallJumpBranch`, `BoostStages`).
    Animation(AnimationSound),
    /// One of the frame's sound opportunities' (`FootContact`, `SpinStarts`, `AtTheDoor`,
    /// `SpinEnds`).
    Opportunity(SoundRequest),
}

/// What the frame tells, as the whole frame leaves it, beside the appearance State it leaves.
///
/// Her step, the boost stage and her colour sequences are outputs too: each is read from that
/// State, where it lives, as the frame leaves it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameOutputs {
    /// The end her clip reaches and whether her spin has turned far, from the frame's start
    /// alone: the same reading as [`Ahead::of`], which samus-movement makes before its step.
    /// A frame without an animation update has none.
    pub ahead: Option<Ahead>,
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
    pub sounds: Vec<FrameSound>,
}

/// Plays one frame of her appearance from `state`, as the frame's parts happen.
///
/// Spec: `CannonItemDelay`, `CannonReadinessReset`, `FootContact`, `SpinStarts`, `AtTheDoor`,
/// `SpinEnds`, `OrdinaryDryColours`, and the animation's entries in their order:
///
/// 1. Her item controls, or her initialization, set the cannon's delay, before her gun's
///    picture reads it.
/// 2. The end and turned far, from the State and her clip as the frame starts.
/// 3. Her stride, from the frame's start.
/// 4. The animation update's first half: her moves' changes, the rising hold, then the clip of
///    her pose at the frame's start plays.
/// 5. Her pose selection's spin start.
/// 6. The animation update's second half: the clip of the pose she takes starts, then a dash's
///    loss.
/// 7. The palette opportunity readies her colour sequences.
/// 8. A first passage entry quiets her, then normal drawing ends a spin's sound.
#[must_use]
pub fn update(
    state: AppearanceState,
    frame: &Frame,
    content: &impl Content,
    rules: &Rules,
) -> (AppearanceState, FrameOutputs) {
    let cannon = state.cannon.after(frame.cannon);
    let at_start = state.animation;
    let ahead = frame
        .animation
        .as_ref()
        .map(|inputs| Ahead::of(at_start.playhead, content.clip(&inputs.figure)));
    let mut sounds = Vec::new();
    let contact = frame.stride.and_then(|stride| {
        let (contact, footstep) = stride.contact(at_start, content.clip(&stride.figure));
        sounds.extend(footstep.map(FrameSound::Opportunity));
        contact
    });
    let played = frame.animation.as_ref().map_or(at_start, |inputs| {
        let (played, sound) = play_her_clip(at_start, inputs, content, rules);
        sounds.extend(sound.map(FrameSound::Animation));
        played
    });
    let spin_start = frame.selection.and_then(Selection::spin_start);
    sounds.extend(spin_start.map(FrameSound::Opportunity));
    let animation = frame
        .animation
        .as_ref()
        .map_or(played, |inputs| take_her_pose(played, inputs, content));
    let (colours, dry_colours) = ordinary_dry_colours(state.colours, frame.palette);
    let quieting = frame.passage.and_then(PassageEntry::quieting);
    let spin_end = frame.drawing.and_then(NormalDrawing::spin_end);
    sounds.extend(
        [quieting, spin_end]
            .into_iter()
            .flatten()
            .map(FrameSound::Opportunity),
    );
    let after = AppearanceState {
        animation,
        cannon,
        colours,
    };
    let outputs = FrameOutputs {
        ahead,
        contact,
        cannon_may_follow: cannon.may_follow(),
        dry_colours,
        sounds,
    };
    (after, outputs)
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
) -> (State, Option<AnimationSound>) {
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
