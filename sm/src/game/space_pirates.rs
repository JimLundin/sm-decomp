//! Space pirates run in `sm_game::actors::space_pirates`; the game side of
//! their hosts lives here.
#![forbid(unsafe_code)]
use crate::game::enemy_projectiles::{scatter_drop, spawn_with_enemy_gfx, spawn_with_room_gfx};
use crate::game::session_runtime::{Runtime, actor_cosine, actor_sine};
use crate::game::sprite_effect_runtime::create_sprite;
use crate::game_state::Game;
use crate::ram_map::word;
use crate::rom::RomRecord;
use crate::types::{EnemyDef, Point16U, Rect16U, Vulnerability};
use core::mem::offset_of;
pub(crate) use sm_game::actors::space_pirates::*;
use sm_game::enemy_projectiles::drops::DropSource;

const _: () = {
    assert!(offset_of!(EnemyDef, vulnerability_ptr) == DEFINITION_VULNERABILITY);
    assert!(offset_of!(Vulnerability, power) == VULNERABILITY_POWER);
    assert!(offset_of!(Vulnerability, plasma_ice_wave) == VULNERABILITY_PLASMA_ICE_WAVE);
};

impl Pirates for Game {
    type Lease<'a> = Runtime<'a>;

    fn lease(&mut self) -> Runtime<'_> {
        Runtime::new(self)
    }

    fn actor_sine(&mut self, phase: u16, amplitude: u16) -> u16 {
        actor_sine(phase, amplitude, self)
    }

    fn actor_cosine(&mut self, phase: u16, amplitude: u16) -> u16 {
        actor_cosine(phase, amplitude, self)
    }

    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) -> u16 {
        create_sprite(x, y, kind, appearance, self)
    }

    fn fire_laser(&mut self, x: u16, y: u16, right: bool) {
        self.eproj_spawn_pt = Point16U { x, y };
        self.eproj_spawn_r22 = u16::from(right);
        spawn_with_room_gfx(0xa17b, u16::from(right), self);
    }

    fn spawn_in_rect(&mut self, x: u16, y: u16, width: u16, height: u16, parameter: u16) {
        self.eproj_spawn_rect = Rect16U {
            x,
            y,
            w: width,
            h: height,
        };
        spawn_with_enemy_gfx(parameter, word!(self.CUR_ENEMY_INDEX), 0xa189, self);
    }

    fn scatter_pirate_drop(&mut self, index: u16) {
        scatter_drop(DropSource::LowerNorfairPirate, index, self);
    }

    fn reflect_player_projectile(&mut self, index: u16) {
        crate::game::player_projectiles::reflect(index, self);
    }

    fn load_ninja_palette(&mut self) {
        let game = self;
        word_transfers!(game, ascending, 16; native(0xc3e0) <- rom(0xb28727, 0));
    }
}

impl PirateLease for Runtime<'_> {
    type Game = Game;

    fn word(&self, at: usize) -> u16 {
        Runtime::word(self, at)
    }

    fn native_word(&self, at: usize) -> u16 {
        Runtime::native_word(self, at)
    }

    fn write_slice_native(&mut self, at: usize, value: u16) {
        Runtime::write_slice_native(self, at, value);
    }

    fn write_word_le(&mut self, value: u16, at: usize) {
        Runtime::write_word_le(self, value, at);
    }

    fn foreign<T>(&mut self, action: impl FnOnce(&mut Game) -> T) -> T {
        Runtime::foreign(self, action)
    }

    fn actor_legacy_record(&mut self, bank: u8, address: u16) -> RomRecord {
        Runtime::actor_legacy_record(self, bank, address)
    }

    fn legacy_native_record(&mut self, bank: u8, address: u16) -> RomRecord {
        Runtime::legacy_native_record(self, bank, address)
    }
}
