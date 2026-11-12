//! §5 Her run: its pace while she dashes, and the speed booster's stages up to boosting.

use clean_types::{Action, AnimationSound, BoostStage, Item};

use crate::clip::{Pace, Paces};
use crate::inputs::{Inputs, Move, SoundsWaiting};
use crate::rules::Rules;
use crate::state::{Boost, Laps};

/// How a lap of her run ends, as the clip moves on to a control with run held.
#[derive(Clone, Copy)]
pub enum LapEnd<'p> {
    /// The laps left counted down to these, and the control does as the clip says: the step it
    /// leads to shows the clip's own hold, not the pace's.
    CountedDown(Laps),
    /// The lap starts over in the control's place: step 0 on `pace`'s first hold, with `boost`
    /// as her boost after it, asking for `sound`.
    StartsOver {
        /// Her boost for the lap that starts.
        boost: Boost,
        /// The pace whose first hold the lap starts on.
        pace: &'p Pace,
        /// The sound asked for: `Boosting` as the stage goes up to 4.
        sound: Option<AnimationSound>,
    },
}

/// Spec: `RunPace`. The pace her stride takes as the clip moves on, if it takes one: running and
/// dashing, the plain pace without the speed booster, the boosted pace of `stage` with it.
/// Dashing in any other action, her clip's own holds show.
#[must_use]
pub fn stride<'p>(inputs: &Inputs, paces: &'p Paces, stage: BoostStage) -> Option<&'p Pace> {
    if !inputs.dashing || inputs.pose.motion.action != Action::Running {
        return None;
    }
    Some(if inputs.items[Item::SpeedBooster] {
        &paces.boosted[stage]
    } else {
        &paces.plain
    })
}

/// Spec: `RunPace`, `BoostStages`. The clip moving on to a control ends a lap of her run when her
/// stride takes a pace and run is held; none otherwise, and the control does as the clip says.
/// Without the speed booster the lap starts over on the plain pace. With it, the laps left count
/// down; once the stage has lasted its laps, the stage goes up (or stays boosting) and the lap
/// starts over on its pace.
#[must_use]
pub fn lap_end<'p>(
    boost: Boost,
    inputs: &Inputs,
    paces: &'p Paces,
    rules: &Rules,
) -> Option<LapEnd<'p>> {
    let pace = stride(inputs, paces, boost.stage).filter(|_| inputs.run_held)?;
    if !inputs.items[Item::SpeedBooster] {
        return Some(LapEnd::StartsOver {
            boost,
            pace,
            sound: None,
        });
    }
    Some(boost.laps_left.and_then(Laps::counted_down).map_or_else(
        || stage_up(boost.stage, inputs.sounds_waiting, paces, rules),
        LapEnd::CountedDown,
    ))
}

/// Spec: `BoostStages`. The stage goes up by 1, to 4 at most, and the next lap takes its laps and
/// starts on its pace's first hold. Going up to 4 asks for `Boosting`, and that lap takes its laps
/// and its first hold alone from `BoostingLap`'s stage; her stage is 4, so its other steps show
/// the boosting pace (`RunPace`).
fn stage_up<'p>(
    from: BoostStage,
    waiting: SoundsWaiting,
    paces: &'p Paces,
    rules: &Rules,
) -> LapEnd<'p> {
    let stage = next_stage(from);
    let starts_boosting = stage == BoostStage::Boosting && from != BoostStage::Boosting;
    let lap_stage = if starts_boosting {
        boosting_lap(waiting)
    } else {
        stage
    };
    LapEnd::StartsOver {
        boost: Boost {
            stage,
            laps_left: Some(rules.laps_per_stage[lap_stage]),
        },
        pace: &paces.boosted[lap_stage],
        sound: starts_boosting.then_some(AnimationSound::Boosting),
    }
}

/// Spec: `BoostStages`. When a dash began, before the clip moves on: stage 0, with stage 0's laps
/// with the speed booster and 0 without it.
#[must_use]
pub fn dash_began(inputs: &Inputs, rules: &Rules) -> Option<Boost> {
    let stage = BoostStage::Stage0;
    let laps_left = inputs.items[Item::SpeedBooster].then_some(rules.laps_per_stage[stage]);
    inputs.moves[Move::DashBegan].then_some(Boost { stage, laps_left })
}

/// Spec: `BoostStages`. When a dash was lost, after the clip moves on: stage 0, with 0 laps left.
#[must_use]
pub fn dash_lost(inputs: &Inputs) -> Option<Boost> {
    inputs.moves[Move::DashLost].then_some(Boost::LOST)
}

/// `BoostStages`: the stage after `stage`; boosting stays boosting.
const fn next_stage(stage: BoostStage) -> BoostStage {
    match stage {
        BoostStage::Stage0 => BoostStage::Stage1,
        BoostStage::Stage1 => BoostStage::Stage2,
        BoostStage::Stage2 => BoostStage::Stage3,
        BoostStage::Stage3 | BoostStage::Boosting => BoostStage::Boosting,
    }
}

/// `BoostingLap`: with 6 or more sounds waiting, the first boosting lap's first hold and its laps
/// are stage 3's.
const MANY_WAITING: BoostStage = BoostStage::Stage3;

/// Spec: `BoostingLap` (a quirk). The stage the sounds waiting name for the lap on which she
/// starts boosting: stage `w` for `w` at most 4, stage 3 for 6 or more. Only that lap's first
/// hold and its laps come from this stage; the rest of the lap runs at the boosting pace. Five
/// waiting is outside the spec (§1.5) and reads as 6 or more.
fn boosting_lap(waiting: SoundsWaiting) -> BoostStage {
    BoostStage::ALL
        .get(usize::from(waiting.count()))
        .copied()
        .unwrap_or(MANY_WAITING)
}
