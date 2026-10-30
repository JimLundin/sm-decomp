//! Draygon runs in `sm_game::actors::draygon_*`; its sprites, VRAM queue
//! writes and runtime hooks come from here.
#![forbid(unsafe_code)]
use crate::game::actors::{api, combat};
use crate::game::enemy_projectiles::scatter_drop;
use crate::game::gameplay_flag_runtime;
use crate::game::music::QueueMusic_Delayed8;
use crate::game::player_movement::forced;
use crate::game::session_runtime::{
    Runtime, actor_cosine, actor_direction_vector, actor_sine, correct_previous_position,
    damage_player, player_command,
};
use crate::game::sprite_effect_runtime as sprites;
use crate::game_state::Game;
use crate::types::{EnemyDef, Point16U};
use core::mem::offset_of;
pub(crate) use sm_game::actors::draygon_flight::*;
use sm_game::actors::draygon_grab::{ENEMY_DEF_DAMAGE, ENEMY_DEF_HURT_AI_TIME};
use sm_game::actors::draygon_host::Host;
use sm_game::enemy_projectiles::drops::DropSource;
use sm_game::gameplay_flag_host::BossWordOrder;
use sm_game::geometry::Vector32;

const _: () = assert!(
    offset_of!(EnemyDef, hurt_ai_time) == ENEMY_DEF_HURT_AI_TIME
        && offset_of!(EnemyDef, damage) == ENEMY_DEF_DAMAGE
);

impl Host for Game {
    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) -> u16 {
        sprites::create_sprite(x, y, kind, appearance, self)
    }

    fn draygon_body_bubble(&mut self) {
        sprites::draygon_body_bubble(self);
    }

    fn draygon_spawn_trails(&mut self) {
        sprites::draygon_spawn_trails(self);
    }

    fn draygon_trajectory(&mut self) {
        sprites::draygon_trajectory(self);
    }

    fn set_projectile_spawn_point(&mut self, x: u16, y: u16) {
        self.eproj_spawn_pt = Point16U { x, y };
    }

    fn write_native_words(&mut self, writes: &[(usize, u16)]) {
        let mut runtime = Runtime::new(self);
        for &(at, value) in writes {
            runtime.write_slice_native(at, value);
        }
    }

    fn disable_minimap_for_boss(&mut self) {
        sm_game::player_update::minimap::disable_for_boss(&mut Runtime::new(self));
    }

    fn actor_cosine(&mut self, phase: u16, amplitude: u16) -> u16 {
        actor_cosine(phase, amplitude, self)
    }

    fn actor_sine(&mut self, phase: u16, amplitude: u16) -> u16 {
        actor_sine(phase, amplitude, self)
    }

    fn actor_direction_vector(&mut self, phase: u16, amplitude: u16) -> Vector32 {
        actor_direction_vector(phase, amplitude, self)
    }

    fn apply_vector(&mut self, phase: u16, vector: Vector32) {
        api::apply_vector(&mut Runtime::new(self), phase, vector);
    }

    fn set_enemy_scratch(&mut self, value: u16) {
        Runtime::new(self).session_mut().compatibility_enemy_scratch = value;
    }

    fn actor_legacy_record(&mut self, bank: u8, address: u16) -> crate::rom::RomRecord {
        Runtime::new(self).actor_legacy_record(bank, address)
    }

    fn legacy_native_record(&mut self, bank: u8, address: u16) -> crate::rom::RomRecord {
        Runtime::new(self).legacy_native_record(bank, address)
    }

    fn command_player(&mut self, command: u16) -> u16 {
        player_command(command, self)
    }

    fn correct_previous_position(&mut self) {
        correct_previous_position(self);
    }

    fn grab_samus(&mut self, direction: u16) {
        forced::grab_by_draygon(direction, &mut Runtime::new(self));
    }

    fn release_samus(&mut self) {
        forced::release_from_draygon(&mut Runtime::new(self));
    }

    fn suit_damage(&mut self, amount: u16) -> u16 {
        combat::suit_damage(&mut Runtime::new(self), amount)
    }

    fn damage_player(&mut self, amount: u16) {
        damage_player(amount, self);
    }

    fn draygon_spawn_ring(&mut self) {
        sprites::draygon_spawn_ring(self);
    }

    fn draygon_death_bubble(&mut self) {
        sprites::draygon_death_bubble(self);
    }

    fn draygon_motion(&mut self, horizontal_bias: u16) {
        sprites::draygon_motion(horizontal_bias, self);
    }

    fn retire_all_sprites(&mut self) {
        sprites::retire_all(self);
    }

    fn queue_music_delayed(&mut self, track: u16) {
        QueueMusic_Delayed8(track, self);
    }

    fn boss_word(&mut self, area: u16) -> u16 {
        gameplay_flag_runtime::boss_word_at(area, BossWordOrder::Little, self)
    }

    fn set_boss_word(&mut self, area: u16, value: u16) {
        gameplay_flag_runtime::set_boss_word_le(area, value, self);
    }

    fn scatter_draygon_drop(&mut self, area: u16) {
        scatter_drop(DropSource::Draygon, area, self);
    }

    fn unreachable(&mut self) {
        crate::game::animated_tiles::native::unreachable(self);
    }
}
