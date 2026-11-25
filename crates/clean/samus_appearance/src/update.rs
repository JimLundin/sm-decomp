//! A frame of her appearance: the frame's inputs and parts go in, her appearance State and the
//! frame's outputs come out, each rule applied once, where it happens in the frame.

use enum_map::Enum;
use types::{RowColours, Set};

use crate::cannon::ItemControls;
use crate::colours::{DryColourOutcome, PaletteOpportunity, ordinary_dry_colours};
use crate::hurt::{HurtOpportunity, hurt_colours};
use crate::inputs::{AppearanceContent, Inputs, MissingClip, clip_of};
use crate::moves::{fast_fall, rising_hold, standing_restart, wall_jump_ready};
use crate::play::ClipPlay;
use crate::rules::Rules;
use crate::run::{dash_began, dash_lost};
use crate::sounds::{
    GraphicalContact, Sound, SoundOffer, at_the_door, foot_contact, spin_end, spin_start,
};
use crate::starting::start;
use crate::state::{Animation, AppearanceState};

/// A part a frame may have, each at most once, that carries nothing beyond the frame's inputs.
/// Listed in the frame's order.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Enum)]
pub enum FramePart {
    /// Her initialization, first in its frame: it sets her whole appearance State to
    /// [`AppearanceState::INITIAL`], whatever State the frame starts with, and the frame's other
    /// parts play on from it (`CannonReadinessReset`). The game's frame of her initialization
    /// has none of her other parts.
    Initialization,
    /// The preparation of her work, before her movement (`SpinEnds`). A frame whose work is
    /// locked is still prepared; a paused frame, or a passage's frame that bypasses her work,
    /// goes unprepared.
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
    /// Her item controls, with their answer when they run (`CannonItemDelay`).
    pub item_controls: ItemControls,
    /// The palette opportunity, with its own observations, or none (`OrdinaryDryColours`).
    pub palette: PaletteOpportunity,
    /// Her hurt colours, after Health, with their observations, or none (`HurtColours`).
    pub hurt: HurtOpportunity,
}

/// What the frame tells, as the whole frame leaves it, beside the appearance State it leaves.
///
/// Her step, the boost stage and her colour sequences are outputs too: each is read from that
/// State, where it lives, as the frame leaves it. The end and turned far are the read-ahead,
/// `Ahead::of`, asked of her State before the frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameOutputs {
    /// Her stride's graphical contact, for part 2's splashes and dust (`FootContact`).
    pub contact: Option<GraphicalContact>,
    /// Whether her cannon may follow the selected item, as the frame's item controls leave its
    /// delay, for the picture of her gun.
    pub cannon_may_follow: bool,
    /// What the palette opportunity ended in, where the frame has one (`OrdinaryDryColours`).
    pub dry_colours: Option<DryColourOutcome>,
    /// The 16 colours her row of the current colours takes this frame, for the palette's owner
    /// to write there; none leaves her row as it is. The target colours are left as they are
    /// (`HurtColours`).
    pub her_colours: Option<RowColours>,
    /// The frame's sound requests, each with its queue and allowance, in the order asked: the
    /// stride's, the animation update's, the spin start's, her hurt colours', the passage
    /// entry's, then normal drawing's. Each request asked for is here, in its place.
    pub sounds: Vec<SoundOffer>,
}

/// Plays one frame of her appearance from `state`, as the frame's parts happen.
///
/// Spec: `CannonItemDelay`, `CannonReadinessReset`, `FootContact`, `SpinStarts`, `AtTheDoor`,
/// `SpinEnds`, `OrdinaryDryColours`, `HurtColours`, and the animation's entries in their order:
///
/// 1. Her initialization, first in its frame, sets her whole appearance State as
///    [`AppearanceState::INITIAL`] gives it, whatever the frame starts with; the rest of the
///    frame plays on from it.
/// 2. Her item controls set the cannon's delay, before her gun's picture reads it.
/// 3. The preparation of her work keeps the action of her pose as the frame starts.
/// 4. Her movement: her stride; her clip plays; the spin start of the pose she takes; that
///    pose's clip starts.
/// 5. The palette opportunity readies her colour sequences, reading the hurt count as the frame
///    starts.
/// 6. After Health, her hurt colours: the pulse, and the hurt count.
/// 7. A first passage entry quiets her, then normal drawing ends a kept spin's sound.
///
/// # Errors
///
/// `MissingClip` with the figure, where the frame has her movement and `content` holds no clip
/// for the figure she starts or ends the frame in.
pub fn update(
    state: AppearanceState,
    frame: &Frame,
    content: &impl AppearanceContent,
    rules: &Rules,
) -> Result<(AppearanceState, FrameOutputs), MissingClip> {
    let (inputs, parts) = (&frame.inputs, &frame.parts);
    let state = if parts[FramePart::Initialization] {
        AppearanceState::INITIAL
    } else {
        state
    };
    let cannon = state.cannon.after(frame.item_controls);
    let kept_action = if parts[FramePart::Preparation] {
        inputs.figure.pose.action
    } else {
        state.kept_action
    };
    let mut sounds = Vec::new();
    let (animation, contact) = if parts[FramePart::Movement] {
        move_her(state.animation, inputs, content, rules, &mut sounds)?
    } else {
        (state.animation, None)
    };
    let (colours, dry_colours) =
        ordinary_dry_colours(state.colours, frame.palette, inputs, state.hurt_count);
    let (hurt_count, her_colours, hurt_sound) = hurt_colours(
        state.hurt_count,
        frame.hurt,
        inputs,
        content.pulse_colours(),
    );
    sounds.extend(hurt_sound);
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
        hurt_count,
    };
    let outputs = FrameOutputs {
        contact,
        cannon_may_follow: cannon.may_follow(),
        dry_colours,
        her_colours,
        sounds: sounds.into_iter().map(Sound::offer).collect(),
    };
    Ok((after, outputs))
}

/// Her movement's part of the frame, in its order, from `at_start`, her animation as the frame
/// starts: her stride (`FootContact`); her clip plays; the spin start of the pose she takes
/// (`SpinStarts`); that pose's clip starts, then a dash's loss. The sounds asked for join
/// `sounds` in that order. Gives her animation as it leaves it and her stride's graphical
/// contact, or the figure whose clip `content` lacks.
fn move_her(
    at_start: Animation,
    inputs: &Inputs,
    content: &impl AppearanceContent,
    rules: &Rules,
    sounds: &mut Vec<Sound>,
) -> Result<(Animation, Option<GraphicalContact>), MissingClip> {
    let clip = clip_of(content, inputs.figure)?;
    let clip_after = clip_of(content, inputs.figure_after)?;
    let (contact, footstep) = foot_contact(at_start, inputs, clip);
    sounds.extend(footstep);
    let playing = ClipPlay {
        inputs,
        clip,
        paces: content.paces(),
        rules,
    };
    let (played, sound) = play_her_clip(at_start, &playing);
    sounds.extend(sound);
    sounds.extend(spin_start(inputs));
    let started = played.with_playhead(start(played.playhead, inputs, clip_after));
    Ok((dash_lost(started, inputs), contact))
}

/// The animation update's first half, in its order: her moves' changes (a dash's beginning, a
/// standing restart, a readied wall jump, a fast fall), the rising hold, then the time runs down
/// and the clip of her figure at the frame's start moves on. Gives her animation and the sound
/// asked for, at most one.
fn play_her_clip(animation: Animation, playing: &ClipPlay<'_>) -> (Animation, Option<Sound>) {
    let (inputs, clip, rules) = (playing.inputs, playing.clip, playing.rules);
    let begun = dash_began(animation, inputs, rules);
    let playhead = standing_restart(begun.playhead, inputs, rules);
    let playhead = wall_jump_ready(playhead, inputs, clip);
    let playhead = fast_fall(playhead, inputs, clip, rules);
    let playhead = rising_hold(playhead, inputs, clip, rules);
    playing.play(begun.with_playhead(playhead))
}
