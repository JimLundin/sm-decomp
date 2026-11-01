//! Shaktool's pieces run in `sm_game::actors::shaktool` on the scheduler's
//! lease; the bank $AA functions they do not run themselves run on the game.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
use sm_game::actors::access::ActorKey as Key;
pub(crate) use sm_game::actors::shaktool::*;

impl Lease for Runtime<'_> {
    fn run_function(&mut self, address: u32) {
        if sm_game::actors::species::is_torizo_pre_instruction(address) {
            self.owned_call(|runtime| {
                let k = crate::game::session_runtime::Runtime::word(runtime, 0xe54);
                sm_game::actors::species::torizo_pre_instruction(address, k, runtime);
            });
        } else {
            self.foreign(|game| crate::game::session_runtime::run_aa_function(address, game));
        }
    }

    fn angle_between(&mut self, from: u16, to: u16) -> u16 {
        crate::game::actors::api::angle_between(self, Key::from_legacy(from), Key::from_legacy(to))
    }
}

// The named bank $AA functions are the ones `DispatchEnemyPreInstr` dispatches on.
const _: () = {
    use sm_game::constants::bank_aa::FNNULLSUB_274;
    const fn at(word: u16) -> u32 {
        0xaa_0000 | word as u32
    }
    assert!(at(function::NOTHING) == FNNULLSUB_274 as u32 && at(function::ARM_PIECE) == 11197612);
    assert!(at(function::HEAD) == 11197655 && at(function::FINAL_PIECE) == 11197733);
};
