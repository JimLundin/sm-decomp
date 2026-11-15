//! Her animation's timing as the engine meets it: the samus-appearance model
//! (`samus_appearance`).
//!
//! The engine keeps the model's State in `Samus::animation`: her step, the time left, the boost
//! stage and the laps left, directly in its pinned canonical model. Each frame the model is read twice. Before samus-movement's step,
//! `Ahead`: the end her clip reaches and whether her spin has turned far enough to kick off a
//! wall, both from the State the frame starts with. After the pose change, the update, from
//! samus-movement's outputs (`Outcome`) and the pose she took. The liquids, the sound queues and
//! the boost's readers aren't the engine's yet (LEGACY, "Samus's animation timing in the
//! engine"): she is in no liquid and no sound waits. Ordered producer requests leave through
//! `session::TickOutput`; U1 still owns their sending and acknowledgements.

pub mod import;
pub mod producer;

use data::pose::animation::AnimationFrame;
use data::pose::transition::Control;
use samus_appearance::{Ahead, Dash, Inputs, Liquid, MoveEvent, Rules, SoundsWaiting, update};
use samus_movement::{Dashing, Outcome, StandingAnimation, WallTest};
use types::{Fixed, Set};

pub use import::{Clips, ImportError, Imported, figure_of};

use crate::pose::controls;
use crate::run_progress_bridge;
use crate::state::Controller;
use crate::state::samus::{PoseId, Samus};

/// Her pose's clip, with its step map; `None` for a pose outside her domain.
fn imported<'a>(samus: &Samus, clips: &'a Clips) -> Option<&'a Imported> {
    clips.of(samus.pose)
}

/// What her clip tells of the frame from its start, read before samus-movement's step.
///
/// The end it reaches (`ClipEnds`) and whether her spin has turned far (`SpinTurnedFar`); `None`
/// for a pose outside her domain, which reaches no end and has no spin.
#[must_use]
pub fn ahead(samus: &Samus, clips: &Clips) -> Option<Ahead> {
    let imported = imported(samus, clips)?;
    Some(Ahead::of(samus.animation.0.0.playhead, &imported.clip))
}

/// The original's frame of her step, which her sprite is drawn by; `None` for a pose outside her
/// domain or a step past her clip.
#[must_use]
pub fn frame(samus: &Samus, clips: &Clips) -> Option<AnimationFrame> {
    imported(samus, clips)?.frame_of(samus.animation.0.0.playhead.step)
}

/// What her animation reads as the frame starts, before her moves and the pose change.
#[derive(Clone, Copy, Debug)]
pub struct Start {
    /// Her pose.
    pose: PoseId,
    /// Her energy: the frame's damage comes after her animation.
    energy: u16,
}

impl Start {
    /// The frame's start, from Samus as it starts.
    #[must_use]
    pub const fn of(samus: &Samus) -> Self {
        Self {
            pose: samus.pose,
            energy: samus.equipment.energy.current,
        }
    }
}

/// The model's inputs that samus-movement's step gives: what her moves did, and how they leave her.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Moved {
    /// A wall jump readied, the standing restart, a dash lost, the pose an end handed over.
    pub moves: Set<MoveEvent>,
    /// Whether she is dashing as her moves leave her, a dash begun this frame included.
    pub dash: Dash,
    /// She is heading down as her moves leave her.
    pub heading_down: bool,
    /// Her vertical speed as her moves leave her, downward positive.
    pub vertical_speed: Fixed,
}

impl From<&Outcome> for Moved {
    fn from(outcome: &Outcome) -> Self {
        let mut moves = Set::<MoveEvent>::default();
        moves[MoveEvent::StandingRestart] =
            outcome.standing_animation == StandingAnimation::Restarts;
        moves[MoveEvent::WallJumpReadied] = outcome.wall_test == WallTest::Readied;
        moves[MoveEvent::HandedOver] = outcome.handed_over;
        moves[MoveEvent::DashLost] = outcome.dash_lost;
        let dash = match (outcome.dash_began, outcome.dashing_after_moves) {
            (true, _) => Dash::Began,
            (false, Dashing::On) => Dash::On,
            (false, Dashing::Off) => Dash::Off,
        };
        Self {
            moves,
            dash,
            heading_down: outcome.heading_down,
            vertical_speed: outcome.vertical_speed_after_moves,
        }
    }
}

/// The model's inputs for the frame Samus ends as `samus`, from her `start` and samus-movement's
/// `outcome`; `None` when the pose she starts or ends in is outside her domain.
fn inputs(
    samus: &Samus,
    start: Start,
    outcome: &Outcome,
    controller: &Controller,
) -> Option<Inputs> {
    let moved = Moved::from(outcome);
    Some(Inputs {
        figure: figure_of(start.pose)?,
        figure_after: figure_of(samus.pose)?,
        moves: moved.moves,
        dash: moved.dash,
        run_held: controls(controller.held).contains(Control::Run),
        heading_down: moved.heading_down,
        vertical_speed: moved.vertical_speed,
        energy: start.energy,
        items: run_progress_bridge::switched_on(&samus.equipment),
        liquid: Liquid::NONE,
        sounds_waiting: SoundsWaiting::NONE,
    })
}

/// The frame's update of her animation, after the pose change.
///
/// It reads samus-movement's `outcome` where its model ran the frame. A frame it didn't run leaves
/// her animation as it was.
pub fn animate(
    samus: &mut Samus,
    start: Start,
    outcome: Option<&Outcome>,
    controller: &Controller,
    clips: &Clips,
) -> Option<types::AnimationSound> {
    let inputs = outcome.and_then(|outcome| inputs(samus, start, outcome, controller))?;
    let (after, sound) = update(samus.animation.0.0, &inputs, clips, &Rules::VANILLA);
    samus.animation.0.0 = after;
    sound
}
