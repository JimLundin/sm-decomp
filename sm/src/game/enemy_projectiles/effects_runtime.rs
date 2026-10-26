//! Live foreign adapters over the shared Session projection.
#![forbid(unsafe_code)]
use super::host::Effects;
use crate::game::session_runtime::Runtime;
impl Effects for Runtime<'_> {
    fn next_random(&mut self) -> u16 {
        self.foreign(crate::game_state::Game::next_random)
    }
    fn grapple_pickup_query(&mut self) -> u16 {
        self.foreign(|game| super::legacy::call_player_code(0xd, game))
    }
    fn resume_player_after_elevator(&mut self) -> u16 {
        self.foreign(|game| super::legacy::call_player_code(0xe, game))
    }
    fn calculate_angle_from_xy(&mut self, x: u16, y: u16) -> u16 {
        self.foreign(|game| super::legacy::calculate_angle_from_xy(x, y, game))
    }
    fn damage_player(&mut self, damage: u16) -> () {
        self.owned_call(|runtime| runtime.apply_contact_damage(damage))
    }
    fn ridley_offscreen(&mut self, x: u16, y: u16) -> u8 {
        self.foreign(|game| super::legacy::ridley_offscreen(x, y, game))
    }
    fn queue_music(&mut self, track: u16) -> () {
        self.foreign(|game| crate::game::music::QueueMusic_Delayed8(track, game))
    }
    fn convert_angle_to_xy(
        &mut self,
        angle: u16,
        amplitude: u16,
    ) -> crate::game::session::Vector32 {
        self.foreign(|game| {
            crate::game::session_runtime::actor_direction_vector(angle, amplitude, game)
        })
    }
    fn queue_sound(&mut self, channel: u8, max: u8, sound: u16) {
        self.foreign(|game| {
            if !(1..=3).contains(&channel) || !matches!(max, 1 | 3 | 6 | 9 | 15) {
                unreachable!("invalid projectile sound channel/limit");
            }
            crate::game::music::queue_sfx(usize::from(channel - 1), sound as u8, max, game);
        });
    }
    fn unreachable(&mut self) {
        self.foreign(super::legacy::unreachable);
    }
    fn draw_spritemap(
        &mut self,
        offscreen: bool,
        bank: u8,
        map: u16,
        x: u16,
        y: u16,
        tile: u16,
        graphics: u16,
    ) {
        self.foreign(|game| {
            super::legacy::draw_spritemap(offscreen, bank, map, x, y, tile, graphics, game)
        });
        crate::verify_probe::hit("eproj.drawn");
    }
    fn respawn_enemy(&mut self, index: u16) {
        self.foreign(|game| super::legacy::respawn_enemy(index, game));
    }
    fn reset_room_enemy_workspace(&mut self) {
        self.foreign(|game| game.g_ram[0x7000..0x9800].fill(0));
    }
}
