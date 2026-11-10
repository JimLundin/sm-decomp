//! §5 Her run: its pace while she dashes, and the speed booster's stages up to boosting.

use clean_types::{Action, AnimationSound, BoostStage, Item, Set};

use crate::clip::{Pace, Paces};
use crate::inputs::Inputs;
use crate::rules::Rules;
use crate::state::{Boost, Laps};

/// Spec: `RunPace`. The pace her stride takes as the clip moves on, if it takes one: running
/// and dashing, the plain pace without the speed booster, her stage's boosted pace with it.
/// Dashing in any other action, her clip's own holds show.
#[must_use]
pub fn stride<'a>(inputs: &Inputs, boost: Boost, paces: &'a Paces) -> Option<&'a Pace> {
    if !inputs.dashing || inputs.pose.motion.action != Action::Running {
        return None;
    }
    Some(if inputs.items[Item::SpeedBooster] {
        &paces.boosted[boost.stage]
    } else {
        &paces.plain
    })
}

/// How a lap of her run ends: her clip, running and dashing with run held, moves on to a
/// control.
pub enum LapEnd<'a> {
    /// The laps left are counted down, and the control does as the clip says.
    AsClipSays(Boost),
    /// The lap starts over: step 0, on the pace's first hold.
    StartsOver {
        /// Her boost for the new lap.
        boost: Boost,
        /// The pace whose first hold the lap starts on.
        pace: &'a Pace,
        /// `Boosting`, when the stage goes up to 4.
        sound: Option<AnimationSound>,
    },
}

/// Spec: `RunPace` and `BoostStages`. A lap ends with run held: without the speed booster it
/// starts over on the plain pace; with it, the laps left count down, and once the stage has
/// lasted its laps the stage goes up (or stays boosting) and the lap starts over on its pace.
#[must_use]
pub fn lap_ends<'a>(boost: Boost, inputs: &Inputs, paces: &'a Paces, rules: &Rules) -> LapEnd<'a> {
    if !inputs.items[Item::SpeedBooster] {
        return LapEnd::StartsOver {
            boost,
            pace: &paces.plain,
            sound: None,
        };
    }
    let counted = boost.laps.0.checked_sub(1).filter(|&laps| laps > 0);
    counted.map_or_else(
        || stage_up(boost.stage, inputs.sounds_waiting, paces, rules),
        |laps| {
            LapEnd::AsClipSays(Boost {
                laps: Laps(laps),
                ..boost
            })
        },
    )
}

/// Spec: `BoostStages`. The stage goes up by 1, to 4 at most, and the new lap takes its laps
/// and its pace; going up to 4 asks for `Boosting`, and takes `BoostingLap`'s stage instead.
fn stage_up<'a>(from: BoostStage, waiting: u8, paces: &'a Paces, rules: &Rules) -> LapEnd<'a> {
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
            laps: rules.laps_per_stage[lap_stage],
        },
        pace: &paces.boosted[lap_stage],
        sound: starts_boosting.then_some(AnimationSound::Boosting),
    }
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

/// `BoostingLap`: with 6 or more sounds waiting, the first boosting lap is stage 3's.
const MANY_WAITING: BoostStage = BoostStage::Stage3;

/// Spec: `BoostingLap` (a quirk). The lap on which she starts boosting runs at the pace, and
/// lasts the laps, of the stage numbered by the sounds waiting: stage `w` for `w` at most 4,
/// stage 3 for 6 or more. Five waiting is outside the spec (§1.5) and reads as 6 or more.
fn boosting_lap(waiting: u8) -> BoostStage {
    BoostStage::ALL
        .get(usize::from(waiting))
        .copied()
        .unwrap_or(MANY_WAITING)
}

/// Spec: `BoostStages`. A dash began: stage 0, with stage 0's laps with the speed booster and 0
/// without it.
#[must_use]
pub fn dash_began(items: &Set<Item>, rules: &Rules) -> Boost {
    let stage = BoostStage::Stage0;
    let laps = if items[Item::SpeedBooster] {
        rules.laps_per_stage[stage]
    } else {
        Laps::default()
    };
    Boost { stage, laps }
}
