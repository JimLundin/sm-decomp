//! `sm_game::actors::boss_lease` for Game, over the RAM image and its effects,
//! and for Runtime, on the leased Session.
#![forbid(unsafe_code)]
use super::{api, combat};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::actors::boss_lease::{BossLease, BossState, LeaseBoss};
use sm_game::actors::host::{ActorRuntime, BossEffects, Death, Shot};
use sm_game::geometry::Point;

fn death(death: Death) -> api::DeathHandling {
    match death {
        Death::Finalize => api::DeathHandling::Finalize,
        Death::LeavePending => api::DeathHandling::LeavePending,
    }
}

fn shot(shot: Shot) -> api::ShotHandling {
    match shot {
        Shot::ReactionOnly => api::ShotHandling::ReactionOnly,
        Shot::Flash => api::ShotHandling::Flash,
        Shot::Finalize => api::ShotHandling::Finalize,
    }
}

impl BossState for Game {
    fn react_to_shot(&mut self, handling: Shot) {
        ActorRuntime::shot_reaction(self, handling);
    }
    fn react_to_contact(&mut self, handling: Death) {
        ActorRuntime::contact_reaction(self, handling);
    }
    fn react_to_power_bomb(&mut self, handling: Death) {
        ActorRuntime::power_bomb_reaction(self, handling);
    }
    fn place_projectile_spawn(&mut self, x: u16, y: u16) {
        BossEffects::set_projectile_spawn_point(self, x, y);
    }
    fn spawn_room_eproj(&mut self, definition: u16, parameter: u16) -> u16 {
        ActorRuntime::spawn_room_projectile(self, definition, parameter)
    }
    fn spawn_actor_eproj(&mut self, parameter: u16, k: u16, definition: u16) -> u16 {
        ActorRuntime::spawn_projectile(self, parameter, k, definition)
    }
    fn roll_random(&mut self) -> u16 {
        Game::next_random(self)
    }
    fn decompress_into_ram(&mut self, source: u32, destination: usize) {
        sm_game::decompression::decompress_to_ram(source, destination, self);
    }
    fn player_direction(&mut self) -> api::PlayerDirection {
        api::direction_to_player(&mut Runtime::new(self))
    }
    fn camera(&mut self) -> sm_game::camera::Camera {
        sm_game::camera::current(self)
    }
}

impl BossLease for Game {
    type Game = Game;
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Game) -> T) -> T {
        action(self)
    }
}

impl LeaseBoss for Game {
    type Lease<'a> = Runtime<'a>;
    fn lease_boss(&mut self) -> Runtime<'_> {
        Runtime::new(self)
    }
}

// Each effect is the one the game ran on a lease of its own, here on this one.
impl BossState for Runtime<'_> {
    fn react_to_shot(&mut self, handling: Shot) {
        combat::shot_reaction(self, shot(handling));
    }
    fn react_to_contact(&mut self, handling: Death) {
        combat::contact_reaction(self, death(handling));
    }
    fn react_to_power_bomb(&mut self, handling: Death) {
        combat::power_bomb_reaction(self, death(handling));
    }
    fn place_projectile_spawn(&mut self, x: u16, y: u16) {
        self.session_mut().enemy_projectiles.spawn.point = Point { x, y };
    }
    fn spawn_room_eproj(&mut self, definition: u16, parameter: u16) -> u16 {
        sm_game::enemy_projectiles::lifecycle::spawn_room(self, definition, parameter)
    }
    fn spawn_actor_eproj(&mut self, parameter: u16, k: u16, definition: u16) -> u16 {
        sm_game::enemy_projectiles::lifecycle::spawn_from_enemy(self, parameter, k, definition)
    }
    fn roll_random(&mut self) -> u16 {
        self.foreign(Game::next_random)
    }
    fn decompress_into_ram(&mut self, source: u32, destination: usize) {
        self.foreign(|game| sm_game::decompression::decompress_to_ram(source, destination, game));
    }
    fn player_direction(&mut self) -> api::PlayerDirection {
        api::direction_to_player(self)
    }
    fn camera(&mut self) -> sm_game::camera::Camera {
        self.session().room.camera
    }
}

impl BossLease for Runtime<'_> {
    type Game = Game;
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Game) -> T) -> T {
        Runtime::foreign(self, action)
    }
}

impl sm_game::cartridge::NativeDiagnostics for Runtime<'_> {
    fn native_rom_diagnostic(&mut self, address: u32) {
        self.foreign(|game| crate::game::session_runtime::native_rom_diagnostic(address, game));
    }
}
