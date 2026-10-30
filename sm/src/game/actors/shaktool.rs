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

impl Ai for Game {
    fn run_function(&mut self, address: u32) {
        crate::game::session_runtime::run_aa_function(address, self);
    }

    fn angle_between(&mut self, from: u16, to: u16) -> u16 {
        crate::game::actors::api::angle_between(
            &crate::game::session_runtime::Runtime::new(self),
            sm_game::actors::access::ActorKey::from_legacy(from),
            sm_game::actors::access::ActorKey::from_legacy(to),
        )
    }
}

// The named bank $AA functions are the ones `DispatchEnemyPreInstr` dispatches on.
const _: () = {
    use sm_game::constants::bank_aa::fnnullsub_274;
    const fn at(word: u16) -> u32 {
        0xaa_0000 | word as u32
    }
    assert!(at(function::NOTHING) == fnnullsub_274 as u32 && at(function::ARM_PIECE) == 11197612);
    assert!(at(function::HEAD) == 11197655 && at(function::FINAL_PIECE) == 11197733);
};
