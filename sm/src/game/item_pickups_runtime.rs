//! Legacy callers enter the same owned inventory used by gameplay cores.
#![forbid(unsafe_code)]
use crate::{
    game::{item_pickups::Resource, session_runtime::Runtime},
    game_state::Game,
};

pub(crate) fn restore(resource: Resource, amount: u16, game: &mut Game) {
    #[cfg(sm_verify_dll)]
    let tag = match resource {
        Resource::Health => 0,
        Resource::Missiles => 1,
        Resource::SuperMissiles => 2,
        Resource::PowerBombs => 3,
    };
    #[cfg(sm_verify_dll)]
    crate::game::enemy_projectiles::checks::supplemental_cases::restore_cases::observe(
        tag, 0, amount, game,
    );
    {
        let mut runtime = Runtime::new(game);
        runtime.session_mut().inventory.restore(resource, amount);
    }
    #[cfg(sm_verify_dll)]
    crate::game::enemy_projectiles::checks::supplemental_cases::restore_cases::observe(
        tag, 1, amount, game,
    );
}
