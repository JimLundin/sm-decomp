//! Her animation's timing as the engine meets it: the samus-looks model
//! (`clean_model_samus_looks`).
//!
//! The engine keeps the model's State in `Samus::animation`: her step, the time left, the boost
//! stage and the laps left. Each frame the model is read twice. Before samus-motion's step,
//! `Ahead`: the end her clip reaches and whether her spin has turned far enough to kick off a
//! wall, both from the State the frame starts with. After the pose change, the update, from
//! samus-motion's outputs (`Outcome`) and the pose she took. The liquids, the sound queues and
//! the boost's readers aren't the engine's yet (LEGACY, "Samus's animation timing in the
//! engine"): she is in no liquid, no sound waits and the sounds asked for go nowhere.

pub mod import;

use clean_model_samus_looks::{
    Ahead, Dash, Frames, Inputs, Laps, LapsLeft, Liquid, MoveEvent, Playhead, Rules, SoundsWaiting,
    State, Step, update,
};
use clean_model_samus_motion::{Dashing, Outcome, StandingAnimation, WallTest};
use clean_types::{BoostStage, Fixed, Set};
use data::pose::animation::AnimationFrame;
use data::pose::transition::Control;

pub use import::{Clips, ImportError, Imported, figure_of};

use crate::pose::controls;
use crate::progress_world;
use crate::state::Controller;
use crate::state::samus::{self, Animation, ClipStep, PoseId, Samus};

/// The model's boost stage for the engine's.
const fn model_stage(stage: samus::BoostStage) -> BoostStage {
    match stage {
        samus::BoostStage::Stage0 => BoostStage::Stage0,
        samus::BoostStage::Stage1 => BoostStage::Stage1,
        samus::BoostStage::Stage2 => BoostStage::Stage2,
        samus::BoostStage::Stage3 => BoostStage::Stage3,
        samus::BoostStage::Boosting => BoostStage::Boosting,
    }
}

/// The engine's boost stage for the model's.
const fn engine_stage(stage: BoostStage) -> samus::BoostStage {
    match stage {
        BoostStage::Stage0 => samus::BoostStage::Stage0,
        BoostStage::Stage1 => samus::BoostStage::Stage1,
        BoostStage::Stage2 => samus::BoostStage::Stage2,
        BoostStage::Stage3 => samus::BoostStage::Stage3,
        BoostStage::Boosting => samus::BoostStage::Boosting,
    }
}

/// The engine's count of the model's laps left.
const fn laps_count(laps_left: LapsLeft) -> u8 {
    match laps_left {
        LapsLeft::WithoutBooster => 0,
        LapsLeft::WithBooster(laps) => laps.count(),
    }
}

/// The model's State from the engine's animation: the engine's 0 laps left is her run without
/// the speed booster.
fn state(animation: Animation) -> State {
    State {
        playhead: playhead(animation),
        stage: model_stage(animation.boost.stage),
        laps_left: Laps::new(animation.boost.laps_left)
            .map_or(LapsLeft::WithoutBooster, LapsLeft::WithBooster),
    }
}

/// Where her clip is, from the engine's animation.
const fn playhead(animation: Animation) -> Playhead {
    Playhead {
        step: Step(animation.step.0),
        time_left: Frames(animation.frames_left),
    }
}

/// Her pose's clip, with its step map; `None` for a pose outside her domain.
fn imported<'a>(samus: &Samus, clips: &'a Clips) -> Option<&'a Imported> {
    clips.of(samus.pose)
}

/// What her clip tells of the frame from its start, read before samus-motion's step.
///
/// The end it reaches (`ClipEnds`) and whether her spin has turned far (`SpinTurnedFar`); `None`
/// for a pose outside her domain, which reaches no end and has no spin.
#[must_use]
pub fn ahead(samus: &Samus, clips: &Clips) -> Option<Ahead> {
    let imported = imported(samus, clips)?;
    Some(Ahead::of(playhead(samus.animation), &imported.clip))
}

/// The original's frame of her step, which her sprite is drawn by; `None` for a pose outside her
/// domain or a step past her clip.
#[must_use]
pub fn frame(samus: &Samus, clips: &Clips) -> Option<AnimationFrame> {
    imported(samus, clips)?.frame_of(Step(samus.animation.step.0))
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

/// The model's inputs that samus-motion's step gives: what her moves did, and how they leave her.
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

/// The model's inputs for the frame Samus ends as `samus`, from her `start` and samus-motion's
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
        items: progress_world::switched_on(&samus.equipment),
        liquid: Liquid::NONE,
        sounds_waiting: SoundsWaiting::NONE,
    })
}

/// The frame's update of her animation, after the pose change.
///
/// It reads samus-motion's `outcome` where its model ran the frame. A frame it didn't run leaves
/// her animation as it was.
pub fn animate(
    samus: &mut Samus,
    start: Start,
    outcome: Option<&Outcome>,
    controller: &Controller,
    clips: &Clips,
) {
    let Some(inputs) = outcome.and_then(|outcome| inputs(samus, start, outcome, controller)) else {
        return;
    };
    let (after, _sound) = update(state(samus.animation), &inputs, clips, &Rules::VANILLA);
    samus.animation = Animation {
        step: ClipStep(after.playhead.step.0),
        frames_left: after.playhead.time_left.0,
        boost: samus::Boost {
            stage: engine_stage(after.stage),
            laps_left: laps_count(after.laps_left),
        },
    };
}
