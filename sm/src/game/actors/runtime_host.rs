//! `sm_game::actors::host::ActorRuntime` for Game, over the actor API.
#![forbid(unsafe_code)]
use super::access::ActorKey as Key;
use super::{api, combat, queries};
use crate::game::enemy_projectiles::{spawn_with_enemy_gfx, spawn_with_room_gfx};
use crate::game::enemy_speeds;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::actors::host::{ActorRuntime, Axis, Death, Shot, Slopes};

fn axis(axis: Axis) -> api::Axis {
    match axis {
        Axis::X => api::Axis::X,
        Axis::Y => api::Axis::Y,
    }
}

fn death(death: Death) -> api::DeathHandling {
    match death {
        Death::Finalize => api::DeathHandling::Finalize,
        Death::LeavePending => api::DeathHandling::LeavePending,
    }
}

impl ActorRuntime for Game {
    fn player_delta(&mut self, k: u16, on: Axis) -> u16 {
        queries::player_delta(&Runtime::new(self), Key::from_legacy(k), axis(on))
    }

    fn player_within(&mut self, k: u16, on: Axis, distance: u16) -> bool {
        queries::player_within(
            &mut Runtime::new(self),
            Key::from_legacy(k),
            axis(on),
            distance,
        )
    }

    fn current_outside_screen(&mut self) -> bool {
        queries::current_outside_screen(&Runtime::new(self))
    }

    fn move_horizontal(&mut self, k: u16, amount: i32, slopes: Slopes) -> bool {
        let slopes = match slopes {
            Slopes::Ignore => api::SlopePolicy::Ignore,
            Slopes::Solid => api::SlopePolicy::Solid,
            Slopes::Follow => api::SlopePolicy::Follow,
        };
        api::move_horizontal(&mut Runtime::new(self), Key::from_legacy(k), amount, slopes)
            == api::MovementResult::Blocked
    }

    fn move_vertical(&mut self, k: u16, amount: i32) -> bool {
        api::move_vertical(&mut Runtime::new(self), Key::from_legacy(k), amount)
            == api::MovementResult::Blocked
    }

    fn snap_to_slopes(&mut self, k: u16) {
        api::snap_to_slopes(&mut Runtime::new(self), Key::from_legacy(k));
    }

    fn contact_reaction(&mut self, handling: Death) {
        combat::contact_reaction(&mut Runtime::new(self), death(handling));
    }

    fn shot_reaction(&mut self, shot: Shot) {
        let shot = match shot {
            Shot::ReactionOnly => api::ShotHandling::ReactionOnly,
            Shot::Flash => api::ShotHandling::Flash,
            Shot::Finalize => api::ShotHandling::Finalize,
        };
        combat::shot_reaction(&mut Runtime::new(self), shot);
    }

    fn power_bomb_reaction(&mut self, handling: Death) {
        combat::power_bomb_reaction(&mut Runtime::new(self), death(handling));
    }

    fn spawn_projectile(&mut self, parameter: u16, k: u16, definition: u16) -> u16 {
        spawn_with_enemy_gfx(parameter, k, definition, self)
    }

    fn spawn_room_projectile(&mut self, definition: u16, parameter: u16) -> u16 {
        spawn_with_room_gfx(definition, parameter, self)
    }

    fn enemy_speed(&self, table: sm_data::enemy_speeds::SpeedTable, index: isize) -> u16 {
        enemy_speeds::word(&self.enemy_speeds, &self.loaded_rom, table, index)
    }
}

// The same operations on a lease, over its Session.
impl ActorRuntime for Runtime<'_> {
    fn player_delta(&mut self, k: u16, on: Axis) -> u16 {
        queries::player_delta(&*self, Key::from_legacy(k), axis(on))
    }

    fn player_within(&mut self, k: u16, on: Axis, distance: u16) -> bool {
        queries::player_within(self, Key::from_legacy(k), axis(on), distance)
    }

    fn current_outside_screen(&mut self) -> bool {
        queries::current_outside_screen(&*self)
    }

    fn move_horizontal(&mut self, k: u16, amount: i32, slopes: Slopes) -> bool {
        let slopes = match slopes {
            Slopes::Ignore => api::SlopePolicy::Ignore,
            Slopes::Solid => api::SlopePolicy::Solid,
            Slopes::Follow => api::SlopePolicy::Follow,
        };
        api::move_horizontal(self, Key::from_legacy(k), amount, slopes)
            == api::MovementResult::Blocked
    }

    fn move_vertical(&mut self, k: u16, amount: i32) -> bool {
        api::move_vertical(self, Key::from_legacy(k), amount) == api::MovementResult::Blocked
    }

    fn snap_to_slopes(&mut self, k: u16) {
        api::snap_to_slopes(self, Key::from_legacy(k));
    }

    fn contact_reaction(&mut self, handling: Death) {
        combat::contact_reaction(self, death(handling));
    }

    fn shot_reaction(&mut self, shot: Shot) {
        let shot = match shot {
            Shot::ReactionOnly => api::ShotHandling::ReactionOnly,
            Shot::Flash => api::ShotHandling::Flash,
            Shot::Finalize => api::ShotHandling::Finalize,
        };
        combat::shot_reaction(self, shot);
    }

    fn power_bomb_reaction(&mut self, handling: Death) {
        combat::power_bomb_reaction(self, death(handling));
    }

    fn spawn_projectile(&mut self, parameter: u16, k: u16, definition: u16) -> u16 {
        sm_game::enemy_projectiles::lifecycle::spawn_from_enemy(self, parameter, k, definition)
    }

    fn spawn_room_projectile(&mut self, definition: u16, parameter: u16) -> u16 {
        sm_game::enemy_projectiles::lifecycle::spawn_room(self, definition, parameter)
    }

    fn enemy_speed(&self, table: sm_data::enemy_speeds::SpeedTable, index: isize) -> u16 {
        Runtime::enemy_speed(self, table, index)
    }
}

impl sm_game::actors::rippers::owned::Host for Runtime<'_> {
    fn ripper_speed(&self, table: sm_data::enemy_speeds::SpeedTable, index: isize) -> u16 {
        Runtime::enemy_speed(self, table, index)
    }
    fn finalized_shot_reaction(&mut self) {
        combat::shot_reaction(self, api::ShotHandling::Finalize);
    }
}

impl sm_game::actors::choot::owned::Host for Runtime<'_> {
    fn choot_speed(&self, table: sm_data::enemy_speeds::SpeedTable, index: isize) -> u16 {
        Runtime::enemy_speed(self, table, index)
    }
    fn choot_table(&self, table: sm_game::actors::choot::owned::Table, index: isize) -> u16 {
        let address = match table {
            sm_game::actors::choot::owned::Table::Patterns => 0xa2_df5e,
            sm_game::actors::choot::owned::Table::PatternYDistances => 0xa2_df6a,
        };
        sm_game::player_projectiles::content::table(self, address, index)
    }
    fn choot_record(&mut self, address: u16) -> sm_data::cartridge::RomRecord {
        self.legacy_native_record(0xa2, address)
    }
    fn unknown_choot_function(&mut self) {
        sm_game::enemy_projectiles::host::Effects::unreachable(self);
    }
}
impl sm_game::actors::zoa::owned::Host for Runtime<'_> {
    fn zoa_table(&self, table: sm_game::actors::zoa::owned::Table, index: isize) -> u16 {
        let address = match table {
            sm_game::actors::zoa::owned::Table::InstructionLists => 0xa3_b40d,
            sm_game::actors::zoa::owned::Table::XSpeeds => 0xa3_b415,
        };
        sm_game::player_projectiles::content::table(self, address, index)
    }
    fn unknown_zoa_function(&mut self) {
        sm_game::enemy_projectiles::host::Effects::unreachable(self);
    }
}

impl sm_game::actors::roach::owned::Host for Runtime<'_> {
    fn roach_table(&self, table: sm_game::actors::roach::owned::RoachTable, index: isize) -> u16 {
        use sm_game::actors::roach::owned::RoachTable::*;
        let address = match table {
            InstructionLists => 0xa3_a111,
            ActivateFunctions => 0xa3_a121,
        };
        sm_game::player_projectiles::content::table(self, address, index)
    }
    fn roach_direction_vector(
        &mut self,
        angle: u16,
        amplitude: u16,
    ) -> sm_game::geometry::Vector32 {
        sm_game::enemy_projectiles::host::Effects::convert_angle_to_xy(self, angle, amplitude)
    }
    fn roach_cosine_full(&mut self, angle: u16, amplitude: u16) -> u32 {
        self.foreign(|game| {
            sm_game::actors::host::BossEffects::actor_cosine_full(game, angle, amplitude)
        })
    }
    fn roach_sine_full(&mut self, angle: u16, amplitude: u16) -> u32 {
        self.foreign(|game| {
            sm_game::actors::host::BossEffects::actor_sine_full(game, angle, amplitude)
        })
    }
    fn roach_random(&mut self) -> u16 {
        sm_game::enemy_projectiles::host::Effects::next_random(self)
    }
    fn unknown_roach_function(&mut self) {
        sm_game::enemy_projectiles::host::Effects::unreachable(self)
    }
}
impl sm_game::actors::tripper::owned::Host for Runtime<'_> {
    fn tripper_speed(&self, table: sm_data::enemy_speeds::SpeedTable, index: isize) -> u16 {
        Runtime::enemy_speed(self, table, index)
    }
    fn finalized_tripper_shot_reaction(&mut self) {
        sm_game::actors::combat::shot_reaction(self, sm_game::actors::api::ShotHandling::Finalize)
    }
}
impl sm_game::actors::sidehopper::owned::Host for Runtime<'_> {
    fn sidehopper_table(
        &self,
        table: sm_game::actors::sidehopper::owned::SidehopperTable,
        index: isize,
    ) -> u16 {
        use sm_game::actors::sidehopper::owned::SidehopperTable::*;
        let address = match table {
            LandedUp => 0xa3_aac2,
            LandedDown => 0xa3_aaca,
            HoppingUp => 0xa3_aad2,
            HoppingDown => 0xa3_aada,
            HopFunctions => 0xa3_aae2,
            SmallDelta => 0xa3_aae6,
            SmallSpeed => 0xa3_aaea,
            SmallHeight => 0xa3_aaee,
            BigDelta => 0xa3_aaf2,
            BigSpeed => 0xa3_aaf6,
            BigHeight => 0xa3_aafa,
        };
        sm_game::player_projectiles::content::table(self, address, index)
    }
    fn sidehopper_speed(&self, table: sm_data::enemy_speeds::SpeedTable, index: isize) -> u16 {
        Runtime::enemy_speed(self, table, index)
    }
    fn sidehopper_variant(&mut self, definition: u16) -> u16 {
        sm_game::player_projectiles::host::Host::record(self, 0xa0, definition).word(0x2a)
    }
    fn capture_hop_speeds(&self) -> sm_data::cartridge::RomImage {
        sm_game::cartridge::NativeImage::native_image(self)
    }
    fn sidehopper_random(&mut self) -> u16 {
        sm_game::enemy_projectiles::host::Effects::next_random(self)
    }
    fn sidehopper_sound(&mut self, sound: u16) {
        sm_game::enemy_projectiles::host::Effects::queue_sound(self, 2, 3, sound)
    }
    fn unknown_sidehopper_function(&mut self) {
        sm_game::enemy_projectiles::host::Effects::unreachable(self)
    }
}
