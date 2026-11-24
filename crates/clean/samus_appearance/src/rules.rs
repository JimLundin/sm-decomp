//! The spec's parameters, as a game's tuning: `Rules::VANILLA` holds the game's values, and any
//! other `Rules` is an adjusted game.

use core::num::Wrapping;

use enum_map::EnumMap;
use types::{BoostStage, Fixed};

use crate::clip::ShownFrames;
use crate::state::Laps;

/// The tuning the model reads its parameters from, one field per parameter the spec names.
#[derive(Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Rules {
    /// `Breathing`: the threshold, the energy at which she still breathes in a loop.
    pub breathing_threshold: u16,
    /// `StandingRestart`: the frames step 0 shows after a restart, the restart's own frame the
    /// first of them.
    pub restart_frames: ShownFrames,
    /// `FastFall`: the size of her vertical speed, rising or falling, at which her fall jumps on.
    pub fast_fall_speed: Fixed,
    /// `FastFall`: the frames the fast fall step shows, the frame she reaches it the first.
    pub fast_fall_frames: ShownFrames,
    /// `RisingHold`: the frames the rising step shows again each time she still rises on its
    /// last frame, that frame the first.
    pub rising_frames: ShownFrames,
    /// `BoostStages`: the laps each stage lasts.
    pub laps_per_stage: EnumMap<BoostStage, Laps>,
}

impl Rules {
    /// The game's values.
    pub const VANILLA: Self = Self {
        breathing_threshold: 30,
        restart_frames: ShownFrames(15),
        // 5.0000 pixels a frame.
        fast_fall_speed: Wrapping(0x0005_0000),
        fast_fall_frames: ShownFrames(7),
        rising_frames: ShownFrames(3),
        // 1 lap at stages 0 to 3, 2 at stage 4.
        laps_per_stage: EnumMap::from_array([
            Laps::ONE,
            Laps::ONE,
            Laps::ONE,
            Laps::ONE,
            Laps::TWO,
        ]),
    };
}
