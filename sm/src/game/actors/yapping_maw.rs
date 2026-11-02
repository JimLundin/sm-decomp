//! Yapping Maw runs in `sm_game::actors::yapping_maw`; sprites, attachments
//! and Samus control come from here.
#![forbid(unsafe_code)]
use super::{api, scheduling};
use crate::game::enemy_projectiles;
use crate::game::session_runtime::{Runtime, player_command, write_multiplicand};
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

/// Main AI - Yapping Maw ($A8:A211) on the scheduler's lease:
/// its function, then its body segments; the multiplicand writes and an
/// unknown function's failure run on the game.
pub(crate) fn run_main_ai(runtime: &mut Runtime<'_>) {
    let known = owned::main_ai(runtime);
    if !known {
        runtime.foreign(|game| crate::rtl::report_unreachable(game));
    }
    owned::update_segments(runtime);
}

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

impl owned::Host for Runtime<'_> {
    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) -> u16 {
        self.foreign(|game| sprites::create_sprite(x, y, kind, appearance, game))
    }
    fn spawn_projectile(&mut self, parameter: u16, key: u16, definition: u16) -> u16 {
        sm_game::enemy_projectiles::lifecycle::spawn_from_enemy(self, parameter, key, definition)
    }
    fn distance_and_direction(&mut self, origin: Point, target: Point) -> (u16, u16) {
        let pair = sm_game::actors::queries::distance_and_direction(self, origin, target);
        (pair.distance, pair.direction)
    }
    fn command_player(&mut self, command: u16) {
        sm_game::player_transitions::command(
            self,
            command,
            sm_game::player_transitions::commands::finish_ceres_arrival,
        );
    }
    fn correct_previous_position(&mut self) {
        sm_game::player_position::correct_previous_position(self);
    }
    fn write_maw_operand(&mut self, operand: u8) {
        self.foreign(|game| write_multiplicand(operand, game));
    }
    fn enemy_sine(&mut self, index: isize) -> u16 {
        sm_game::gameplay_tables::enemy_sine(self, index)
    }
    fn attack_list(&mut self, index: isize) -> u16 {
        sm_game::actor_tables::word(self, |tables| &tables.maw_attack_lists, index)
    }
    fn attachment_offset(&mut self, y: bool, native: bool, index: isize) -> u16 {
        let address = if y { 0xa8a0a9 } else { 0xa8a0a7 };
        let image = if native {
            self.native_image()
        } else {
            self.loaded_image().expect("Maw loaded source")
        };
        image.table_word(
            usize::try_from(sm_data::cartridge::lorom_offset(address)).expect("Maw offset table"),
            index,
        )
    }
    fn enemy_speed(&self, table: sm_data::enemy_speeds::SpeedTable, index: isize) -> u16 {
        Runtime::enemy_speed(self, table, index)
    }
    fn update_maw_visibility(&mut self, key: i32) {
        Runtime::update_maw_visibility(self, key);
    }
    fn update_maw_segment(&mut self, part: u8) {
        sm_game::enemy_projectiles::attachments_spore_maw::update_maw_segment(part, self);
    }
    fn maw_appearance(&mut self, handle: u16, appearance: u16, key: u16) {
        self.foreign(|game| sprites::maw_appearance(handle, appearance, key, game));
    }
    fn shot_reaction(&mut self, shot: sm_game::actors::host::Shot) {
        sm_game::actors::host::ActorRuntime::shot_reaction(self, shot);
    }
    fn free_projectile(&mut self, key: u16) {
        self.write_slice_native(0x1997 + usize::from(key), 0);
    }
    fn retire_sprite(&mut self, handle: u16) {
        self.foreign(|game| sprites::retire(handle, game));
    }
    fn update_frozen_enemy(&mut self) {
        scheduling::update_frozen_enemy(self);
    }
    fn update_maw_palette(&mut self, handle: u16) {
        sm_game::enemy_projectiles::attachments_spore_maw::update_maw_palette(handle, self);
    }
}
