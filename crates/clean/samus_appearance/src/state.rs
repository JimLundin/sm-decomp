//! What her appearance remembers from one frame to the next: her animation's four quantities
//! (her step, the time left on it, the boost stage and the laps left), her kept action, the
//! cannon selection delay, and her colour sequences. A caller may build each at any value its
//! range allows, read it back, save and restore it, and gives it back to the next frame as
//! `update` left it.

use core::num::NonZeroU8;

use serde::{Deserialize, Serialize};
use types::{Action, BoostStage};

use crate::cannon::CannonDelay;
use crate::clip::{Clip, Frames, Hold, Instruction, Slowdown, Step};
use crate::colours::{ColourSequences, ORDINARY_READY};

/// Spec: `ClipPlays`, `BoostStages`. How the time left and the laps left run out: the count less
/// 1, while that is above 0; at 0 or below the count is over. So 1 left and none left are over
/// alike.
fn less_one(count: u8) -> Option<u8> {
    count.checked_sub(1).filter(|&left| left > 0)
}

/// Where her clip is: her step, and the time left on it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Playhead {
    /// Her step: the picture she shows (part 2).
    pub step: Step,
    /// The time left: how long her step still shows, 0 to 255 frames.
    pub time_left: Frames,
}

/// What her clip does this frame as the time runs down (`ClipPlays`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RunDown {
    /// Time is left, or her clip has no step after hers: she stays on her step, as this
    /// playhead says.
    Stays(Playhead),
    /// The time has run out: the clip moves on to this step, which holds this instruction.
    MovesOn(Step, Instruction),
}

impl Playhead {
    /// Her clip on `step`, with `hold` lengthened by each of `slowdowns` as the time left. A
    /// control has no hold, so she rests on it with no time left, however slow the liquid
    /// (`ClipControls`).
    pub(crate) fn on(step: Step, hold: Option<Hold>, slowdowns: &[Slowdown]) -> Self {
        let time_left = hold.map_or(Frames::NONE, |hold| {
            slowdowns
                .iter()
                .fold(hold.frames(), |left, &slowdown| left.slowed(slowdown))
        });
        Self { step, time_left }
    }

    /// Spec: `ClipPlays`. The time left runs down by a frame: while some is left she stays on her
    /// step, and once it is over the clip moves on to the step after it.
    pub(crate) fn run_down(self, clip: &Clip) -> RunDown {
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Laps(NonZeroU8);

impl Laps {
    /// Spec: `BoostStages`. Vanilla stages before boosting last one lap.
    pub const ONE: Self = Self(NonZeroU8::MIN);
    /// Spec: `BoostStages`. Vanilla boosting lasts two laps.
    pub const TWO: Self = Self(NonZeroU8::MIN.saturating_add(1));
    /// The laps of `count`, if it is 1 or more.
    #[must_use]
    pub const fn new(count: u8) -> Option<Self> {
        match NonZeroU8::new(count) {
            Some(count) => Some(Self(count)),
            None => None,
        }
    }

    /// How many laps.
    #[must_use]
    pub const fn count(self) -> u8 {
        self.0.get()
    }
}

/// The laps of her run the boost stage still lasts, the lap she is running included, so at 1
/// that lap is the stage's last (`BoostStages`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub enum LapsLeft {
    /// No laps, 0: her run without the speed booster. A dash begun without it leaves none, as
    /// does a dash lost, until the next dash begins.
    WithoutBooster,
    /// Her run with the speed booster: 1 lap or more, up to the laps the stage lasts.
    WithBooster(Laps),
}

impl LapsLeft {
    /// Spec: `BoostStages`. The laps left once she has run a lap: less 1, while that is above 0;
    /// none once the stage has lasted its laps.
    pub(crate) fn after_lap(self) -> Option<Laps> {
        match self {
            Self::WithBooster(laps) => less_one(laps.count()).and_then(Laps::new),
            Self::WithoutBooster => None,
        }
    }
}

/// Spec: `ClipPlays`, `BoostStages`. Her animation: the part of her appearance State that plays
/// her clip, its four quantities her step, the time left, the boost stage and the laps left.
///
/// The end, whether her spin has turned far, where the clip of the pose she takes starts, the
/// slowdowns and what her moves did are each worked out within the frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Animation {
    /// Her step and the time left on it.
    pub playhead: Playhead,
    /// The boost stage, 0 to 4; 4 is boosting.
    pub stage: BoostStage,
    /// The laps the stage still lasts.
    pub laps_left: LapsLeft,
}

impl Animation {
    /// Her clip on `playhead`, with the boost stage and the laps left as they are.
    #[must_use]
    pub const fn with_playhead(self, playhead: Playhead) -> Self {
        Self { playhead, ..self }
    }
}

/// Her appearance State: all her appearance remembers from one frame to the next.
///
/// Spec: `SpinEnds`, `CannonItemDelay`, `OrdinaryDryColours`. Her kept action, the cannon
/// selection delay and the colour sequences are memories of their own, each kept apart from her
/// animation, her pose, the selected item and the palettes. The current and target palettes
/// live with their shared owner, beside this State.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct AppearanceState {
    /// Her animation's four quantities.
    pub animation: Animation,
    /// Her kept action: the action of her pose as her last prepared frame started, which normal
    /// drawing reads (`SpinEnds`).
    pub kept_action: Action,
    /// The cannon selection delay.
    pub cannon: CannonDelay,
    /// The visor delay, the visor's entry and the charge glow's position.
    pub colours: ColourSequences,
}

impl AppearanceState {
    /// Her appearance as Samus is initialized (her arrival at Ceres or Zebes, or a demo's
    /// loading), whatever came before: step 0 with no time left, stage 0 with no laps left
    /// whatever her items, her kept action `Still` (no spin, so drawing ends none until a
    /// preparation keeps one), the cannon waiting 2 frames (`CannonReadinessReset`), and the
    /// colour sequences as the ordinary reset leaves them (`OrdinaryDryColours`). A frame with
    /// her initialization sets every quantity to this first, and plays on from it.
    pub const INITIAL: Self = Self {
        animation: Animation {
            playhead: Playhead {
                step: Step::FIRST,
                time_left: Frames::NONE,
            },
            stage: BoostStage::Stage0,
            laps_left: LapsLeft::WithoutBooster,
        },
        kept_action: Action::Still,
        cannon: CannonDelay::INITIAL,
        colours: ORDINARY_READY,
    };
}
