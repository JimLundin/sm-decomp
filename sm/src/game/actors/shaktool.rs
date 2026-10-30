//! Shaktool's pieces run in `sm_game::actors::shaktool`; its tables come from here.
#![forbid(unsafe_code)]
use crate::game::shaktool_tables;
use crate::game_state::Game;
pub(crate) use sm_game::actors::shaktool::*;

impl Host for Game {
    fn shaktool_sine(&self, angle: isize) -> u16 {
        shaktool_tables::sine(self, angle)
    }

    fn shaktool_negative_cosine(&self, angle: isize) -> u16 {
        shaktool_tables::negative_cosine(self, angle)
    }
}
