//! Draygon runs in `sm_game::actors::draygon_*`; its sprites, VRAM queue
//! writes and runtime hooks come from here.
#![forbid(unsafe_code)]
use crate::game::session_runtime::{Runtime, actor_cosine};
use crate::game::sprite_effect_runtime as sprites;
use crate::game_state::Game;
use crate::types::Point16U;
pub(crate) use sm_game::actors::draygon_flight::*;
use sm_game::actors::draygon_host::Host;

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

    fn unreachable(&mut self) {
        crate::game::animated_tiles::native::unreachable(self);
    }
}
