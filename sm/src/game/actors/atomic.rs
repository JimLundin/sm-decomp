//! Atomic runs in `sm_game::actors::atomic`; its unreachable diagnostic runs
//! in a Runtime lease, as before.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
pub(crate) use sm_game::actors::atomic::*;
use sm_game::enemy_projectiles::host::Effects;

impl Host for Game {
    fn unreachable(&mut self) {
        Effects::unreachable(&mut Runtime::new(self));
    }
}
