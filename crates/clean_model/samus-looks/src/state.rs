//! What the domain remembers from one frame to the next: her step, the time left on it, the boost
//! stage and the laps left. A caller may build each at any value the spec allows and read it
//! back, and keeps them between frames as `update` left them: nothing else carries over.

use clean_types::BoostStage;

use crate::clip::{Clip, Frames, Hold, Instruction, Slowdown, Step};

/// Spec: `ClipPlays`, `BoostStages`. How the time left and the laps left run out: the count less
/// 1, while that is above 0; at 0 or below the count is over. So 1 left and none left are over
/// alike.
fn less_one(count: u8) -> Option<u8> {
    count.checked_sub(1).filter(|&left| left > 0)
}

/// Where her clip is: her step, and the time left on it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Playhead {
    /// Her step: the picture she shows (part 2).
    pub step: Step,
    /// The time left: how long her step still shows, 0 to 255 frames.
    pub time_left: Frames,
}

/// What her clip does this frame as the time runs down (`ClipPlays`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RunDown {
    /// Time is left, or her clip has no step after hers (outside the spec's clips): she stays on
    /// her step, as this playhead says.
    Stays(Playhead),
    /// The time has run out: the clip moves on to this step, which holds this instruction.
    MovesOn(Step, Instruction),
}

impl Playhead {
    /// Her clip on `step`, with `hold` lengthened by each of `slowdowns` as the time left. A
    /// control has no hold, so she rests on it with no time left, however slow the liquid
    /// (`ClipControls`).
    #[must_use]
    pub fn on(step: Step, hold: Option<Hold>, slowdowns: &[Slowdown]) -> Self {
        let time_left = hold.map_or(Frames::NONE, |hold| {
            slowdowns
                .iter()
                .fold(hold.frames(), |left, &slowdown| left.slowed(slowdown))
        });
        Self { step, time_left }
    }

    /// Spec: `ClipPlays`. The time left runs down by a frame: while some is left she stays on her
    /// step, and once it is over the clip moves on to the step after it.
    #[must_use]
    pub fn run_down(self, clip: &Clip) -> RunDown {
        if let Some(left) = less_one(self.time_left.0) {
            return RunDown::Stays(Self {
                time_left: Frames(left),
                ..self
            });
        }
        let next = self.step.next().and_then(|to| Some((to, clip.at(to)?)));
        next.map_or(RunDown::Stays(self), |(to, instruction)| {
            RunDown::MovesOn(to, instruction)
        })
    }
}

/// A number of laps of her run, 1 or more: the laps a stage lasts (`BoostStages`), which an
/// adjusted game may set to any number from 1, or the laps a stage still lasts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Laps(pub(crate) u8);

impl Laps {
    /// The laps of `count`, if it is 1 or more.
    #[must_use]
    pub const fn new(count: u8) -> Option<Self> {
        if count > 0 { Some(Self(count)) } else { None }
    }

    /// How many laps.
    #[must_use]
    pub const fn count(self) -> u8 {
        self.0
    }
}

/// The laps of her run the boost stage still lasts, the lap she is running included, so at 1
/// that lap is the stage's last (`BoostStages`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LapsLeft {
    /// No laps, 0: her run without the speed booster. A dash begun without it leaves none, and so
    /// does a dash lost, until the next dash begins.
    WithoutBooster,
    /// Her run with the speed booster: 1 lap or more, up to the laps the stage lasts.
    WithBooster(Laps),
}

impl LapsLeft {
    /// Spec: `BoostStages`. The laps left once she has run a lap: less 1, while that is above 0;
    /// none once the stage has lasted its laps.
    #[must_use]
    pub fn after_lap(self) -> Option<Laps> {
        match self {
            Self::WithBooster(laps) => less_one(laps.0).map(Laps),
            Self::WithoutBooster => None,
        }
    }
}

/// The State: everything the domain remembers from one frame to the next.
///
/// The end, whether her spin has turned far, where the clip of the pose she takes starts, the
/// slowdowns and what her moves did are each worked out within the frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct State {
    /// Her step and the time left on it.
    pub playhead: Playhead,
    /// The boost stage, 0 to 4; 4 is boosting.
    pub stage: BoostStage,
    /// The laps the stage still lasts.
    pub laps_left: LapsLeft,
}
