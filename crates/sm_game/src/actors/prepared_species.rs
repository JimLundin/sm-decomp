//! Concrete ordinary-family bindings to the current actor interval.
//! All motion/combat/allocations are the actual shared cores, not callbacks to Game.
#![forbid(unsafe_code)]
use super::PreparedActorState;
use crate::actors::access::ActorKey;
use crate::actors::{api, combat, host, movement};
use crate::session_access::SessionAccess;
use sm_data::enemy_speeds::SpeedTable;

fn death(value: host::Death) -> api::DeathHandling {
    match value {
        host::Death::Finalize => api::DeathHandling::Finalize,
        host::Death::LeavePending => api::DeathHandling::LeavePending,
    }
}
impl host::ActorRuntime for PreparedActorState<'_> {
    fn player_delta(&mut self, key: u16, axis: host::Axis) -> u16 {
        host::ActorQueries::player_delta(self, key, axis)
    }
    fn player_within(&mut self, key: u16, axis: host::Axis, distance: u16) -> bool {
        host::ActorQueries::player_within(self, key, axis, distance)
    }
    fn current_outside_screen(&mut self) -> bool {
        host::ActorQueries::current_outside_screen(self)
    }
    fn move_horizontal(&mut self, key: u16, amount: i32, slopes: host::Slopes) -> bool {
        movement::move_horizontal(
            self,
            ActorKey::from_legacy(key),
            amount,
            match slopes {
                host::Slopes::Ignore => api::SlopePolicy::Ignore,
                host::Slopes::Solid => api::SlopePolicy::Solid,
                host::Slopes::Follow => api::SlopePolicy::Follow,
            },
        ) == api::MovementResult::Blocked
    }
    fn move_vertical(&mut self, key: u16, amount: i32) -> bool {
        movement::move_vertical(self, ActorKey::from_legacy(key), amount)
            == api::MovementResult::Blocked
    }
    fn snap_to_slopes(&mut self, key: u16) {
        movement::snap_to_slopes(self, ActorKey::from_legacy(key));
    }
    fn contact_reaction(&mut self, handling: host::Death) {
        combat::contact_reaction(self, death(handling));
    }
    fn shot_reaction(&mut self, handling: host::Shot) {
        combat::shot_reaction(
            self,
            match handling {
                host::Shot::ReactionOnly => api::ShotHandling::ReactionOnly,
                host::Shot::Flash => api::ShotHandling::Flash,
                host::Shot::Finalize => api::ShotHandling::Finalize,
            },
        );
    }
    fn power_bomb_reaction(&mut self, handling: host::Death) {
        combat::power_bomb_reaction(self, death(handling));
    }
    fn spawn_projectile(&mut self, parameter: u16, key: u16, definition: u16) -> u16 {
        crate::enemy_projectiles::lifecycle::spawn_from_enemy(self, parameter, key, definition)
    }
    fn spawn_room_projectile(&mut self, definition: u16, parameter: u16) -> u16 {
        crate::enemy_projectiles::lifecycle::spawn_room(self, definition, parameter)
    }
    fn enemy_speed(&self, table: SpeedTable, index: isize) -> u16 {
        self.speed_word(table, index)
    }
}
impl crate::actors::rippers::owned::Host for PreparedActorState<'_> {
    fn ripper_speed(&self, table: SpeedTable, index: isize) -> u16 {
        self.speed_word(table, index)
    }
    fn finalized_shot_reaction(&mut self) {
        combat::shot_reaction(self, api::ShotHandling::Finalize);
    }
}
impl crate::actors::roach::owned::Host for PreparedActorState<'_> {
    fn roach_table(&self, table: crate::actors::roach::owned::RoachTable, index: isize) -> u16 {
        self.roach_table_word(table, index)
    }
    fn roach_direction_vector(&mut self, angle: u16, amplitude: u16) -> crate::geometry::Vector32 {
        self.ordinary_direction_vector(angle, amplitude)
    }
    fn roach_cosine_full(&mut self, angle: u16, amplitude: u16) -> u32 {
        let phase = u16::from(angle.wrapping_add(64).to_le_bytes()[0]);
        crate::actors::queries::sine_product(self, amplitude, phase)
    }
    fn roach_sine_full(&mut self, angle: u16, amplitude: u16) -> u32 {
        let phase = u16::from(angle.wrapping_add(128).to_le_bytes()[0]);
        crate::actors::queries::sine_product(self, amplitude, phase)
    }
    fn roach_random(&mut self) -> u16 {
        self.ordinary_random()
    }
    fn unknown_roach_function(&mut self) {
        self.unknown_function();
    }
}
impl crate::actors::tripper::owned::Host for PreparedActorState<'_> {
    fn tripper_speed(&self, table: SpeedTable, index: isize) -> u16 {
        self.speed_word(table, index)
    }
    fn finalized_tripper_shot_reaction(&mut self) {
        combat::shot_reaction(self, api::ShotHandling::Finalize);
    }
}
impl crate::actors::sidehopper::owned::Host for PreparedActorState<'_> {
    fn sidehopper_table(
        &self,
        table: crate::actors::sidehopper::owned::SidehopperTable,
        index: isize,
    ) -> u16 {
        self.sidehopper_table_word(table, index)
    }
    fn sidehopper_speed(&self, table: SpeedTable, index: isize) -> u16 {
        self.speed_word(table, index)
    }
    fn sidehopper_variant(&mut self, definition: u16) -> u16 {
        self.sidehopper_variant_word(definition)
    }
    fn capture_hop_speeds(&self) -> sm_data::cartridge::RomImage {
        self.capture_sidehopper_hop_speeds()
    }
    fn sidehopper_random(&mut self) -> u16 {
        self.ordinary_random()
    }
    fn sidehopper_sound(&mut self, sound: u16) {
        crate::enemy_projectiles::host::Effects::queue_sound(self, 2, 3, sound);
    }
    fn unknown_sidehopper_function(&mut self) {
        self.unknown_function();
    }
}
impl crate::actors::draygon_eye::owned::Host for PreparedActorState<'_> {
    fn place_projectile_spawn(&mut self, x: u16, y: u16) {
        self.session_mut().enemy_projectiles.spawn.point = crate::geometry::Point { x, y };
    }
    fn spawn_room_projectile(&mut self, definition: u16, parameter: u16) -> u16 {
        crate::enemy_projectiles::lifecycle::spawn_room(self, definition, parameter)
    }
}
