//! The numbers the spec's entries give, as a game's tuning: `Rules::VANILLA` holds the spec's.

use clean_types::BoostStage;
use enum_map::EnumMap;

use crate::clip::Frames;
use crate::state::Laps;

/// The tuning the model reads its numbers from.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Rules {
    /// `Breathing`: the energy at which she still breathes in a loop.
    pub breathing_threshold: u16,
    /// `StandingRestart`: the restart's hold, which no liquid slows.
    pub restart_hold: Frames,
    /// `FastFall`: the downward speed, in pixels a frame, at which her fall jumps on.
    pub fast_fall_speed: u16,
    /// `FastFall`: the fast fall step's hold, which no liquid slows.
    pub fast_fall_hold: Frames,
    /// `RisingHold`: the rising step's hold while she rises, which no liquid slows.
    pub rising_hold: Frames,
    /// `BoostStages` (a parameter): the laps each stage lasts.
    pub laps_per_stage: EnumMap<BoostStage, Laps>,
}

impl Rules {
    /// The spec's values.
    pub const VANILLA: Self = Self {
        breathing_threshold: 30,
        restart_hold: Frames(16),
        fast_fall_speed: 5,
        fast_fall_hold: Frames(8),
        rising_hold: Frames(4),
        laps_per_stage: EnumMap::from_array([Laps(1), Laps(1), Laps(1), Laps(1), Laps(2)]),
    };
}
