//! §5 Her run: its pace while she dashes, and the speed booster's stages up to boosting.

use clean_types::{Action, AnimationSound, BoostStage, Item};

use crate::clip::{Pace, Step};
use crate::frame::Frame;
use crate::inputs::Move;
use crate::state::{Boost, Laps, Playhead};

/// How a lap of her run ends, in one answer: her boost after it, and whether and how the next
/// lap starts over.
#[derive(Clone, Copy)]
pub struct LapEnd {
    /// Her boost after the lap.
    pub boost: Boost,
    /// How the next lap starts, if it starts over; none where the laps left only count down, and
    /// the control does as the clip says, its step showing the clip's own hold, not the pace's.
    pub starts_over: Option<LapStart>,
}

/// A lap of her run starting over: step 0 on a pace's first hold.
#[derive(Clone, Copy)]
pub struct LapStart {
    /// Step 0, with the pace's first hold plus the slowdown as the time left.
    pub playhead: Playhead,
    /// `Boosting`, when the stage goes up to 4.
    pub sound: Option<AnimationSound>,
}

impl Frame<'_> {
    /// Spec: `RunPace`. The pace her stride takes as the clip moves on, if it takes one: running
    /// and dashing, the plain pace without the speed booster, her stage's boosted pace with it.
    /// Dashing in any other action, her clip's own holds show.
    #[must_use]
    pub fn stride(&self, boost: Boost) -> Option<&Pace> {
        let inputs = self.inputs;
        if !inputs.dashing || inputs.pose.motion.action != Action::Running {
            return None;
        }
        Some(if inputs.items[Item::SpeedBooster] {
            &self.paces.boosted[boost.stage]
        } else {
            &self.paces.plain
        })
    }

    /// Spec: `RunPace`, `BoostStages`. The clip moving on to a control ends a lap of her run when
    /// her stride takes a pace and run is held; none otherwise, and the control does as the clip
    /// says. Without the speed booster the lap starts over on the plain pace. With it, the laps
    /// left count down; once the stage has lasted its laps, the stage goes up (or stays
    /// boosting) and the lap starts over on its pace.
    #[must_use]
    pub fn lap_end(&self, boost: Boost) -> Option<LapEnd> {
        let pace = self.stride(boost).filter(|_| self.inputs.run_held)?;
        if !self.inputs.items[Item::SpeedBooster] {
            return Some(LapEnd {
                boost,
                starts_over: Some(self.lap_start(pace, None)),
            });
        }
        if let Some(laps_left) = boost.laps_left.counted_down() {
            return Some(LapEnd {
                boost: Boost { laps_left, ..boost },
                starts_over: None,
            });
        }
        Some(self.stage_up(boost.stage))
    }

    /// Spec: `BoostStages`. The stage goes up by 1, to 4 at most, and the next lap takes its laps
    /// and its pace; going up to 4 asks for `Boosting`, and the lap takes `BoostingLap`'s stage
    /// instead.
    fn stage_up(&self, from: BoostStage) -> LapEnd {
        let stage = next_stage(from);
        let starts_boosting = stage == BoostStage::Boosting && from != BoostStage::Boosting;
        let lap_stage = if starts_boosting {
            boosting_lap(self.inputs.sounds_waiting)
        } else {
            stage
        };
        let sound = starts_boosting.then_some(AnimationSound::Boosting);
        LapEnd {
            boost: Boost {
                stage,
                laps_left: self.rules.laps_per_stage[lap_stage],
            },
            starts_over: Some(self.lap_start(&self.paces.boosted[lap_stage], sound)),
        }
    }

    /// `RunPace`, `BoostStages`: the next lap starts at step 0 on `pace`'s first hold.
    fn lap_start(&self, pace: &Pace, sound: Option<AnimationSound>) -> LapStart {
        LapStart {
            playhead: self.goes_on(Step::FIRST, Some(pace)),
            sound,
        }
    }

    /// Spec: `BoostStages`. A dash began, before the clip moves on: stage 0, with stage 0's laps
    /// with the speed booster and 0 without it.
    #[must_use]
    pub fn dash_began(&self, boost: Boost) -> Boost {
        if !self.inputs.moves[Move::DashBegan] {
            return boost;
        }
        let stage = BoostStage::Stage0;
        let laps_left = if self.inputs.items[Item::SpeedBooster] {
            self.rules.laps_per_stage[stage]
        } else {
            Laps::default()
        };
        Boost { stage, laps_left }
    }

    /// Spec: `BoostStages`. A dash lost, after the clip moves on: stage 0, with 0 laps left.
    #[must_use]
    pub fn dash_lost(&self, boost: Boost) -> Boost {
        if self.inputs.moves[Move::DashLost] {
            Boost::default()
        } else {
            boost
        }
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
