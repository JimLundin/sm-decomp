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
