//! The lights, one piece of the shaft as she arrives, which the shaft's overlay shows: the
//! beacon pulsing through its colour sets and the platform below turning. They remember
//! nothing: each follows from the count of finished frames on the frame it is shown.
//!
//! The caller steps them ([`Lights::step`]) on each frame the shaft's overlay runs. On a frame
//! it doesn't run, and in every other room, the caller doesn't step them, and the screen keeps
//! what it shows.

use crate::look::AlternatingLook;
use crate::rules::Rules;
use crate::station::Stage;

/// `BeaconCycles`: the beacon's cycle has eight colour sets.
const BEACON_STEPS: u16 = 8;
/// `PlatformTurns`: the platform shows each of its looks for two frames.
const PLATFORM_FRAMES: u16 = 2;
/// `PlatformTurns`: the platform has two looks.
const PLATFORM_LOOKS: u16 = 2;

/// The fade's steps taken: how many steps of a door's fade the passage has taken when the
/// overlay runs, 0 to 14. It is 0 outside a door's fade, the screen's brightness coming up at
/// the arrival included.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FadeSteps(u8);

impl FadeSteps {
    /// The most steps a door's fade takes.
    pub const MOST: u8 = 14;
    /// No step of a door's fade taken.
    pub const NONE: Self = Self(0);

    /// This many steps, or none past the most a fade takes.
    #[must_use]
    pub const fn new(steps: u8) -> Option<Self> {
        if steps <= Self::MOST {
            Some(Self(steps))
        } else {
            None
        }
    }

    /// How many steps it is.
    #[must_use]
    pub const fn steps(self) -> u8 {
        self.0
    }
}

/// The beacon's step: its place in its cycle of eight colour sets, 0 to 7. The colours of each
/// set are the shaft's content; showing them is the renderer's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeaconStep(u16);

impl BeaconStep {
    /// The step at this place in the cycle, or none past its last.
    #[must_use]
    pub const fn new(place: u16) -> Option<Self> {
        if place < BEACON_STEPS {
            Some(Self(place))
        } else {
            None
        }
    }

    /// Its place in the cycle.
    #[must_use]
    pub const fn place(self) -> u16 {
        self.0
    }
}

/// What a step of the lights reads, on a frame the shaft's overlay runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightsInputs {
    /// The count of finished frames: the frames the game has finished, as the last frame left
    /// it, wrapping at 16 bits. It never starts again, unlike the HUD's frame count.
    pub finished_frames: u16,
    /// The fade's steps taken.
    pub fade_steps_taken: FadeSteps,
    /// The station's stage.
    pub stage: Stage,
}

/// What a step of the lights shows. Each none leaves the screen as it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lights {
    /// `BeaconCycles`: the beacon's step, whose colour set the screen's colours take.
    pub beacon: Option<BeaconStep>,
    /// `PlatformTurns`: the platform's look, which of its two pictures the planes show.
    pub platform: Option<AlternatingLook>,
}

impl Lights {
    /// One step of the lights, on a frame the shaft's overlay runs. In the escape they show no
    /// beacon step and no platform look: a later piece owns the shaft in the escape.
    #[must_use]
    pub fn step(inputs: LightsInputs, rules: &Rules) -> Self {
        match inputs.stage {
            Stage::Escape => Self {
                beacon: None,
                platform: None,
            },
            Stage::Calm | Stage::Getaway => Self {
                beacon: beacon(inputs, rules),
                platform: Some(platform(inputs.finished_frames)),
            },
        }
    }
}

/// `BeaconCycles`: the beacon shows step `(count / r) mod 8` of its cycle, one step every `r`
/// frames; none once a door's fade has taken a step, which leaves the colours as the fade
/// makes them.
fn beacon(inputs: LightsInputs, rules: &Rules) -> Option<BeaconStep> {
    (inputs.fade_steps_taken == FadeSteps::NONE)
        .then(|| BeaconStep(inputs.finished_frames / rules.rhythm % BEACON_STEPS))
}

/// `PlatformTurns`: the platform shows its two looks in turn, two frames each, by
/// `(count / 2) mod 2`, while a door's fade is under way too.
const fn platform(finished_frames: u16) -> AlternatingLook {
    if (finished_frames / PLATFORM_FRAMES).is_multiple_of(PLATFORM_LOOKS) {
        AlternatingLook::First
    } else {
        AlternatingLook::Second
    }
}
