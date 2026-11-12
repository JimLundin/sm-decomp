//! The spec's parameters, as a game's tuning: `Rules::VANILLA` holds the game's values, and any
//! other `Rules` is an adjusted game.

use core::num::Wrapping;

use clean_types::{BoostStage, Fixed};
use enum_map::EnumMap;

use crate::clip::Hold;
use crate::state::Laps;

/// The tuning the model reads its parameters from, one field per parameter the spec names.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Rules {
    /// `Breathing`: the threshold, the energy at which she still breathes in a loop.
    pub breathing_threshold: u16,
    /// `StandingRestart`: the restart's hold, which no liquid slows.
    pub restart_hold: Hold,
    /// `FastFall`: the size of her vertical speed, rising or falling, at which her fall jumps on.
    pub fast_fall_speed: Fixed,
    /// `FastFall`: the fast fall step's hold, which no liquid slows.
    pub fast_fall_hold: Hold,
    /// `RisingHold`: the rising step's hold while she rises, which no liquid slows.
    pub rising_hold: Hold,
    /// `BoostStages`: the laps each stage lasts.
    pub laps_per_stage: EnumMap<BoostStage, Laps>,
}

impl Rules {
    /// The game's values.
    pub const VANILLA: Self = Self {
        breathing_threshold: 30,
        restart_hold: Hold(16),
        // 5.0000 pixels a frame.
        fast_fall_speed: Wrapping(0x0005_0000),
        fast_fall_hold: Hold(8),
        rising_hold: Hold(4),
        // 1 lap at stages 0 to 3, 2 at stage 4.
        laps_per_stage: EnumMap::from_array([Laps(1), Laps(1), Laps(1), Laps(1), Laps(2)]),
    };
}
