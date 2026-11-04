//! Draygon runs in `sm_game::actors::draygon_*`; its sprites, VRAM queue
//! writes and runtime hooks come from here.
#![forbid(unsafe_code)]
use crate::game::actors::{api, combat};
use crate::game::gameplay_flag_runtime;
use crate::game::music::queue_music_delayed_8;
use crate::game::session_runtime::{
    Runtime, actor_cosine, actor_direction_vector, actor_sine, correct_previous_position,
    damage_player, player_command,
};
use crate::game::sprite_effect_runtime as sprites;
use crate::game_state::Game;
use crate::types::{EnemyDef, Point16U};
use core::mem::offset_of;
use sm_game::actors::draygon_grab::{ENEMY_DEF_DAMAGE, ENEMY_DEF_HURT_AI_TIME};
use sm_game::actors::host::BossEffects;
use sm_game::geometry::Vector32;

const _: () = assert!(
    offset_of!(EnemyDef, hurt_ai_time) == ENEMY_DEF_HURT_AI_TIME
        && offset_of!(EnemyDef, damage) == ENEMY_DEF_DAMAGE
);

// The named bank $A5 functions are the ones the species table dispatches on.
const _: () = {
    use sm_game::actors::draygon_host::function as f;
    const fn at(word: u16) -> u32 {
        0xa5_0000 | word as u32
    }
    assert!(at(f::FIGHT_INTRO_INITIAL_DELAY) == 10848027 && at(f::FIGHT_INTRO_DANCE) == 10848139);
    assert!(at(f::SWOOP_RIGHT_SETUP) == 10848244 && at(f::SWOOP_RIGHT_DESCENDING) == 10848433);
    assert!(at(f::SWOOP_RIGHT_APEX) == 10848546 && at(f::SWOOP_RIGHT_ASCENDING) == 10848593);
    assert!(at(f::SWOOP_LEFT_SETUP) == 10848691 && at(f::SWOOP_LEFT_DESCENDING) == 10848768);
    assert!(at(f::SWOOP_LEFT_APEX) == 10848848 && at(f::SWOOP_LEFT_ASCENDING) == 10848912);
    assert!(at(f::GOOP_RIGHT_SETUP) == 10849034 && at(f::GOOP_LEFT_SETUP) == 10849422);
    assert!(at(f::GOOP_RIGHT_MOVE_UNTIL_IN_RANGE) == 10849106);
    assert!(at(f::GOOP_LEFT_MOVE_UNTIL_IN_RANGE) == 10849492);
    assert!(at(f::GOOP_RIGHT_FIRING) == 10849198 && at(f::GOOP_LEFT_FIRING) == 10849584);
    assert!(at(f::GOOP_RIGHT_MOVE_UNTIL_OFF_SCREEN) == 10849331);
    assert!(at(f::GOOP_LEFT_MOVE_UNTIL_OFF_SCREEN) == 10849714);
    assert!(at(f::CHASE_SAMUS) == 10849817 && at(f::REPELLED_BY_GRAPPLE) == 10850064);
    assert!(
        at(f::GRABBED_MOVING_TO_TARGET) == 10850078 && at(f::GRABBED_RISING_SPIRAL) == 10850262
    );
    assert!(at(f::GRABBED_TAIL_WHIP) == 10850516);
    assert!(at(f::GRABBED_FINAL_TAIL_WHIPS_START) == 10850565);
    assert!(at(f::GRABBED_FINAL_TAIL_WHIPS_ONGOING) == 10850596);
    assert!(
        at(f::FLAIL_TAIL_AND_FLY_STRAIGHT_UP) == 10850600 && at(f::FLY_STRAIGHT_UP) == 10850644
    );
    assert!(
        at(f::DEATH_DRIFT_TO_DEATH_SPOT) == 10850693 && at(f::DEATH_WAIT_FOR_EVIRS) == 10850964
    );
    assert!(at(f::DEATH_BURIED_BY_EVIRS) == 10850987 && at(f::NONE) == 10846283);
    assert!(at(f::EYE_FACING_LEFT) == 10863757 && at(f::EYE_FACING_RIGHT) == 10863891);
};

impl BossEffects for Game {
    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) -> u16 {
        sprites::create_sprite(x, y, kind, appearance, self)
    }

    fn set_projectile_spawn_point(&mut self, x: u16, y: u16) {
        self.eproj_spawn_pt = Point16U { x, y };
    }

    fn actor_cosine(&mut self, phase: u16, amplitude: u16) -> u16 {
        actor_cosine(phase, amplitude, self)
    }

    fn actor_sine(&mut self, phase: u16, amplitude: u16) -> u16 {
        actor_sine(phase, amplitude, self)
    }

    fn actor_cosine_full(&mut self, phase: u16, amplitude: u16) -> u32 {
        crate::game::session_runtime::actor_cosine_full(phase, amplitude, self)
    }

    fn actor_sine_full(&mut self, phase: u16, amplitude: u16) -> u32 {
        crate::game::session_runtime::actor_sine_full(phase, amplitude, self)
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

    fn suit_damage(&mut self, amount: u16) -> u16 {
        combat::suit_damage(&mut Runtime::new(self), amount)
    }

    fn damage_player(&mut self, amount: u16) {
        damage_player(amount, self);
    }

    fn queue_music_delayed(&mut self, track: u16) {
        queue_music_delayed_8(track, self);
    }

    fn set_boss_word(&mut self, area: u16, value: u16) {
        gameplay_flag_runtime::set_boss_word_le(area, value, self);
    }

    fn unreachable(&mut self) -> ! {
        crate::rtl::unreachable()
    }

    fn spawn_plm(&mut self, block_x: u8, block_y: u8, header: u16) {
        crate::game::room_mechanisms::spawn_hardcoded(
            crate::types::SpawnHardcodedPlmArgs::new(
                block_x,
                block_y,
                header as crate::types::VoidP,
            ),
            self,
        );
    }
}
