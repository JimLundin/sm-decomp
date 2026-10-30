//! The Torizos run in `sm_game::actors::torizo`; movement, projectiles and the
//! shot reaction come from here.
#![forbid(unsafe_code)]
use crate::game::actors::access::ActorKey as Key;
use crate::game::actors::{self, api as actor_api};
use crate::game::constants::bank_86::addr_kEproj_BombTorizoLowHealthInitialDrool;
use crate::game::enemy_projectiles::SpawnEprojWithGfx;
use crate::game::session_runtime::Runtime;
use crate::game::sm_aa::{
    addr_kTorizo_Ilist_B1C8, addr_kTorizo_Ilist_CDE1, addr_kTorizo_Ilist_CE43,
    addr_kTorizo_Ilist_CEA5, addr_kTorizo_Ilist_CEFF, addr_kTorizo_Ilist_D1F1,
    addr_kTorizo_Ilist_D2AD,
};
use crate::game_state::Game;
pub(crate) use sm_game::actors::torizo::*;

const _: () = {
    use sm_game::actors::torizo::instruction_list as list;
    assert!(
        list::DEATH_SEQUENCE == addr_kTorizo_Ilist_B1C8 as u16
            && list::CAUGHT_SUPER_MISSILE_LEFT_LEFT_LEG == addr_kTorizo_Ilist_CDE1 as u16
            && list::CAUGHT_SUPER_MISSILE_LEFT_RIGHT_LEG == addr_kTorizo_Ilist_CE43 as u16
            && list::CAUGHT_SUPER_MISSILE_RIGHT_RIGHT_LEG == addr_kTorizo_Ilist_CEA5 as u16
            && list::CAUGHT_SUPER_MISSILE_RIGHT_LEFT_LEG == addr_kTorizo_Ilist_CEFF as u16
            && list::DODGE_TURNING_LEFT == addr_kTorizo_Ilist_D1F1 as u16
            && list::DODGE_TURNING_RIGHT == addr_kTorizo_Ilist_D2AD as u16
            && LOW_HEALTH_DROOL == addr_kEproj_BombTorizoLowHealthInitialDrool as u16
    );
};

impl Host for Game {
    fn move_horizontal_ignoring_slopes(&mut self, k: u16, amount: i32) -> bool {
        actor_api::move_horizontal(
            &mut Runtime::new(self),
            Key::from_legacy(k),
            amount,
            actor_api::SlopePolicy::Ignore,
        ) == actor_api::MovementResult::Blocked
    }

    fn snap_to_slopes(&mut self, k: u16) {
        actor_api::snap_to_slopes(&mut Runtime::new(self), Key::from_legacy(k));
    }

    fn spawn_projectile(&mut self, parameter: u16, k: u16, definition: u16) {
        SpawnEprojWithGfx(parameter, k, definition, self);
    }

    fn shot_reaction_only(&mut self) {
        actors::combat::shot_reaction(
            &mut Runtime::new(self),
            actors::api::ShotHandling::ReactionOnly,
        );
    }
}
