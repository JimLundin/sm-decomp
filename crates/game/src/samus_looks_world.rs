//! Her animation's timing as the engine meets it: the samus-looks model
//! (`clean_model_samus_looks`).
//!
//! The engine keeps the model's State in `Samus::animation`: her step, the time left, the boost
//! stage and the laps left. Each frame the model is read twice. Before samus-motion's step, the
//! end her clip reaches and whether her spin has turned far enough to kick off a wall, both from
//! the State the frame starts with. After the pose change, the update, from samus-motion's
//! outputs (`Outcome`) and the pose she took. The liquids, the sound queues and the boost's
//! readers aren't the engine's yet (LEGACY, "Samus's animation timing in the engine"): she is in
//! no liquid, no sound waits and the sounds asked for go nowhere.

pub mod import;

use clean_model_samus_looks::{
    Boost, Frames, Inputs, Laps, Liquid, Move, Playhead, Rules, State, Step, update,
};
use clean_model_samus_motion::{Dashing, Outcome, StandingAnimation, WallTest};
use clean_types::{BoostStage, ClipEnd, Set};
use data::pose::animation::AnimationFrame;
use data::pose::transition::Control;

pub use import::{Clips, ImportError, Imported, pose_of};

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

/// The model's State from the engine's animation.
const fn state(animation: Animation) -> State {
    State {
        playhead: playhead(animation),
        boost: Boost {
            stage: model_stage(animation.boost.stage),
            laps_left: Laps(animation.boost.laps_left),
        },
    }
}

/// Where her clip is, from the engine's animation.
const fn playhead(animation: Animation) -> Playhead {
    Playhead {
        step: Step(animation.step.0),
        frames_left: Frames(animation.frames_left),
    }
}

/// Her pose's clip, with its step map; `None` for a pose outside her domain.
fn imported<'a>(samus: &Samus, clips: &'a Clips) -> Option<&'a Imported> {
    clips.of(samus.pose)
}

/// `ClipEnds`: the end her clip reaches this frame, read before samus-motion's step.
#[must_use]
pub fn end_ahead(samus: &Samus, clips: &Clips) -> Option<ClipEnd> {
    let imported = imported(samus, clips)?;
    playhead(samus.animation).end_ahead(&imported.clip)
}

/// `SpinTurnedFar`: her spin has turned far enough to kick off a wall, as the frame starts.
///
/// Her clip has a kick step and her step is at it or past it. The model reports it at the end
/// of the frame before (`Report::turned_far`); a frame that starts from a seed has no frame
/// before, so it is read again from the State, which holds all it depends on.
#[must_use]
pub fn turned_far(samus: &Samus, clips: &Clips) -> bool {
    imported(samus, clips)
        .and_then(|imported| imported.clip.kick)
        .is_some_and(|kick| Step(samus.animation.step.0) >= kick)
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

/// What her moves did this frame that her animation reads, from samus-motion's outputs.
#[must_use]
pub fn moves(outcome: &Outcome) -> Set<Move> {
    let mut moves = Set::<Move>::default();
    moves[Move::DashBegan] = outcome.dash_began;
    moves[Move::StandingRestart] = outcome.standing_animation == StandingAnimation::Restarts;
    moves[Move::WallJumpReadied] = outcome.wall_test == WallTest::Readied;
    moves[Move::HandedOver] = outcome.handed_over;
    moves[Move::DashLost] = outcome.dash_lost;
    moves
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
    let (Some(outcome), Some(pose), Some(pose_after)) =
        (outcome, pose_of(start.pose), pose_of(samus.pose))
    else {
        return;
    };
    let inputs = Inputs {
        pose,
        pose_after,
        moves: moves(outcome),
        dashing: outcome.dashing_after_moves == Dashing::On,
        run_held: controls(controller.held).contains(Control::Run),
        heading_down: outcome.heading_down,
        vertical_speed: outcome.vertical_speed_after_moves,
        energy: start.energy,
        items: progress_world::switched_on(&samus.equipment),
        liquid: Liquid::default(),
        sounds_waiting: 0,
    };
    let (after, _report) = update(state(samus.animation), &inputs, clips, &Rules::VANILLA);
    samus.animation = Animation {
        step: ClipStep(after.playhead.step.0),
        frames_left: after.playhead.frames_left.0,
        boost: samus::Boost {
            stage: engine_stage(after.boost.stage),
            laps_left: after.boost.laps_left.0,
        },
    };
}
