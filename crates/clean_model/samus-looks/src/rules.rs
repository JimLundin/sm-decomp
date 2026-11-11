//! The numbers the spec's entries give, as a game's tuning: `Rules::VANILLA` holds the spec's.

use core::num::Wrapping;

use clean_types::{BoostStage, Fixed};
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
    /// `FastFall`: the size of her vertical speed, rising or falling, at which her fall jumps on.
    pub fast_fall_speed: Fixed,
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
        // 5.0000 pixels a frame.
        fast_fall_speed: Wrapping(0x0005_0000),
        fast_fall_hold: Frames(8),
        rising_hold: Frames(4),
        laps_per_stage: EnumMap::from_array([Laps(1), Laps(1), Laps(1), Laps(1), Laps(2)]),
    };
}
