//! The Shitroid runs in `sm_game::actors::shitroid`; PLM spawns come from here.
#![forbid(unsafe_code)]
use crate::game::room_mechanisms::spawn_hardcoded;
use crate::game_state::Game;
use crate::types::{Rect16U, SpawnHardcodedPlmArgs, VoidP};
use sm_game::actors::shitroid;
pub(crate) use sm_game::actors::shitroid::{
    Host, accelerate_towards_x, accelerate_towards_y, cry_sound_effect, is_off_screen,
    pin_camera_at_512, separated_from_enemy,
};

/// 1 when actor `k`'s hitbox does not overlap `rect`. Was `Shitroid_Func_2`.
pub(crate) fn separated_from_rect(k: u16, rect: Rect16U, game: &mut Game) -> u8 {
    shitroid::separated_from_rect(k, (rect.x, rect.y, rect.w, rect.h), game)
}

impl Host for Game {
    fn spawn_hardcoded_plm(&mut self, block_x: u8, block_y: u8, header: u16) {
        spawn_hardcoded(
            SpawnHardcodedPlmArgs::new(block_x, block_y, header as VoidP),
            self,
        );
    }
}
