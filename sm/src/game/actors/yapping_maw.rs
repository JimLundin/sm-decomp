//! Yapping Maw runs in `sm_game::actors::yapping_maw`; sprites, attachments
//! and Samus control come from here.
#![forbid(unsafe_code)]
use super::{api, scheduling};
use crate::game::enemy_projectiles;
use crate::game::session_runtime::{Runtime, correct_previous_position, player_command};
use crate::game::sprite_effect_runtime as sprites;
use crate::game_state::Game;
pub(crate) use sm_game::actors::yapping_maw::*;

// The named bank $A8 addresses are the ones `session_runtime` dispatches on.
const _: () = {
    use sm_game::actors::yapping_maw::instruction as i;
    assert!(i::OFFSET_SAMUS_UP_RIGHT == 11051207 && i::OFFSET_SAMUS_UP_LEFT == 11051225);
    assert!(i::OFFSET_SAMUS_DOWN_RIGHT == 11051243 && i::OFFSET_SAMUS_DOWN_LEFT == 11051261);
    assert!(i::OFFSET_SAMUS_UP == 11051279 && i::OFFSET_SAMUS_DOWN == 11051297);
    assert!(i::QUEUE_SOUND_IF_ON_SCREEN == 11051315);
};
use sm_game::geometry::Point;

impl Host for Game {
    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) -> u16 {
        sprites::create_sprite(x, y, kind, appearance, self)
    }

    fn maw_appearance(&mut self, handle: u16, appearance: u16, enemy: u16) {
        sprites::maw_appearance(handle, appearance, enemy, self);
    }

    fn retire_sprite(&mut self, handle: u16) {
        sprites::retire(handle, self);
    }

    fn distance_and_direction(&mut self, origin: Point, target: Point) -> (u16, u16) {
        let pair: api::DistanceDirection =
            api::distance_and_direction(&mut Runtime::new(self), origin, target);
        (pair.distance, pair.direction)
    }

    fn command_player(&mut self, command: u16) {
        player_command(command, self);
    }

    fn correct_previous_position(&mut self) {
        correct_previous_position(self);
    }

    fn update_maw_visibility(&mut self, actor: i32) {
        Runtime::new(self).update_maw_visibility(actor);
    }

    fn update_maw_segment(&mut self, part: u8) {
        enemy_projectiles::update_maw_segment(part, self);
    }

    fn update_maw_palette(&mut self, handle: u16) {
        enemy_projectiles::update_maw_palette(handle, self);
    }

    fn update_frozen_enemy(&mut self) {
        scheduling::update_frozen_enemy(&mut Runtime::new(self));
    }
}
