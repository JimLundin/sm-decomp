//! Live foreign adapters over the shared Session projection.
#![forbid(unsafe_code)]
use super::host::Effects;
use crate::game::session_runtime::Runtime;
impl Effects for Runtime<'_> {
    fn next_random(&mut self) -> u16 {
        self.next_random_number()
    }
    fn grapple_pickup_query(&mut self) -> u16 {
        // The command's former foreign entry ran it on a fresh nested lease.
        self.owned_player_call(|runtime| {
            crate::game::session_runtime::player_command_in(runtime, 0xd)
        })
    }
    fn resume_player_after_elevator(&mut self) -> u16 {
        self.owned_player_call(|runtime| {
            crate::game::session_runtime::player_command_in(runtime, 0xe)
        })
    }
    fn calculate_angle_from_xy(&mut self, x: u16, y: u16) -> u16 {
        self.owned_player_call(|_| crate::game::actors::api::angle_from_delta(x, y))
    }
    fn damage_player(&mut self, damage: u16) -> () {
        self.owned_call(|runtime| sm_game::samus_periodic_damage::contact(runtime, damage))
    }
    fn ridley_offscreen(&mut self, x: u16, y: u16) -> u8 {
        // The former foreign entry read the game's level data; the lease reads
        // its Session terrain in an owned call, keeping the observation refresh.
        self.owned_player_call(|runtime| {
            u8::from(sm_game::actors::ridley::solid_block_at(x, y, runtime))
        })
    }
    fn queue_music(&mut self, track: u16) -> () {
        self.queue_music_request(track);
    }
    fn convert_angle_to_xy(&mut self, angle: u16, amplitude: u16) -> sm_game::geometry::Vector32 {
        // The game read the $A0:B7EE sine table inside `foreign`; the
        // lease reads it in an owned call, keeping the observation refresh.
        self.owned_player_call(|runtime| {
            sm_game::actors::boss_lease::BossState::direction_step(runtime, angle, amplitude)
        })
    }
    fn queue_sound(&mut self, channel: u8, max: u8, sound: u16) {
        if !(1..=3).contains(&channel) || !matches!(max, 1 | 3 | 6 | 9 | 15) {
            unreachable!("invalid projectile sound channel/limit");
        }
        self.queue_sound_request(usize::from(channel - 1), sound as u8, max);
    }
    fn draw_spritemap(
        &mut self,
        offscreen: bool,
        bank: u8,
        map: u16,
        position: sm_game::geometry::Point,
        tile: u16,
        graphics: u16,
    ) {
        self.draw_objects(|drawing| {
            super::legacy::draw_spritemap(
                offscreen, bank, map, position.x, position.y, tile, graphics, drawing,
            )
        });
        crate::verify_probe::hit("eproj.drawn");
    }
    fn respawn_enemy(&mut self, index: u16) {
        self.owned_player_call(|runtime| {
            crate::game::actors::population::respawn(
                runtime,
                crate::game::actors::access::ActorKey::from_legacy(index),
            );
        });
    }
    fn reset_room_enemy_workspace(&mut self) {
        self.foreign(|game| game.ram().fill(0x7000..0x9800, 0));
    }
}
