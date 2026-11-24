//! Her run: its pace while she dashes, and the speed booster's stages up to boosting.

use types::{Action, BoostStage, Item};

use crate::clip::{Clip, Hold, Pace, Paces, Step};
use crate::inputs::{Inputs, SoundsWaiting};
use crate::rules::Rules;
use crate::state::{Animation, LapsLeft};

/// Spec: `RunPace`. The pace her stride takes as the clip moves on, if it takes one: running and
/// dashing as the clip moves on, the plain pace without the speed booster, the boosted pace of
/// `stage` with it. In any other action, or with no dash as the clip moves on, her clip's own
/// holds show.
#[must_use]
pub fn stride<'p>(inputs: &Inputs, paces: &'p Paces, stage: BoostStage) -> Option<&'p Pace> {
    let running = inputs.figure.pose.action == Action::Running;
    if !running || !inputs.dash.dashing_as_the_clip_moves_on() {
        return None;
    }
    Some(if inputs.items[Item::SpeedBooster] {
        &paces.boosted[stage]
    } else {
        &paces.plain
    })
}

/// Spec: `RunPace`. The hold `step` shows: `pace`'s for it, where her stride takes a pace that
/// gives one, else the clip's own.
#[must_use]
pub fn hold_on(step: Step, pace: Option<&Pace>, clip: &Clip) -> Option<Hold> {
    pace.and_then(|pace| pace.hold(step))
        .or_else(|| clip.hold(step))
}

/// Spec: `BoostStages`. The stage after `stage` as a stage lasts out its laps; boosting stays
/// boosting.
#[must_use]
pub const fn next_stage(stage: BoostStage) -> BoostStage {
    match stage {
        BoostStage::Stage0 => BoostStage::Stage1,
        BoostStage::Stage1 => BoostStage::Stage2,
        BoostStage::Stage2 => BoostStage::Stage3,
        BoostStage::Stage3 | BoostStage::Boosting => BoostStage::Boosting,
    }
}

/// Spec: `BoostingLap`. With 6 or more sounds waiting, the lap's stage is stage 3; 5 waiting
/// reads as 6.
const BOOSTING_LAP_STAGE_MANY_WAITING: BoostStage = BoostStage::Stage3;

/// Spec: `BoostingLap` (a quirk). The stage the sounds waiting, `w`, name for the lap on which
/// she starts boosting: that lap starts on this stage's pace's first hold and lasts this stage's
/// laps. Stage `w` for each `w` at most 4, by the stages' own numbering; stage 3 for more. The
/// rest of the lap runs at the boosting pace, her stage being 4.
#[must_use]
pub fn boosting_lap_stage(waiting: SoundsWaiting) -> BoostStage {
    BoostStage::ALL
        .get(usize::from(waiting.count()))
        .copied()
        .unwrap_or(BOOSTING_LAP_STAGE_MANY_WAITING)
}

/// Spec: `BoostStages`. A dash that began this frame, before the clip moves on, starts her run
/// at stage 0: with stage 0's laps left with the speed booster, none without it.
#[must_use]
pub fn dash_began(animation: Animation, inputs: &Inputs, rules: &Rules) -> Animation {
    if !inputs.dash.began() {
        return animation;
    }
    let laps_left = if inputs.items[Item::SpeedBooster] {
        LapsLeft::WithBooster(rules.laps_per_stage[BoostStage::Stage0])
    } else {
        LapsLeft::WithoutBooster
    };
    Animation {
        stage: BoostStage::Stage0,
        laps_left,
        ..animation
    }
}

/// Spec: `BoostStages`. A dash lost by the frame's end leaves stage 0 with no laps left, after
/// the clip has moved on and the clip of the pose she takes has started.
#[must_use]
pub const fn dash_lost(animation: Animation, inputs: &Inputs) -> Animation {
    if !inputs.dash.lost() {
        return animation;
    }
    Animation {
        stage: BoostStage::Stage0,
        laps_left: LapsLeft::WithoutBooster,
        ..animation
    }
}
