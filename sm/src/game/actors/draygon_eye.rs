//! Draygon's eye runs in `sm_game::actors::draygon_eye`; the actor lease for
//! Game is the compat Runtime.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::actors::host::{ActorLease, LeaseActors};

impl ActorLease for Runtime<'_> {
    type Game = Game;

    fn word(&self, at: usize) -> u16 {
        Runtime::word(self, at)
    }

    fn foreign<T>(&mut self, action: impl FnOnce(&mut Game) -> T) -> T {
        Runtime::foreign(self, action)
    }

    fn write_slice_native(&mut self, at: usize, value: u16) {
        Runtime::write_slice_native(self, at, value);
    }

    fn write_slice_le(&mut self, at: usize, value: u16) {
        Runtime::write_slice_le(self, at, value);
    }

    fn leased_image(&self) -> Option<crate::rom::RomImage> {
        Runtime::loaded_image(self)
    }

    fn lease_native_word(&self, at: usize) -> u16 {
        Runtime::native_word(self, at)
    }

    fn lease_native_record(&mut self, bank: u8, address: u16) -> crate::rom::RomRecord {
        Runtime::legacy_native_record(self, bank, address)
    }

    fn lease_enemy_speed(&self, table: sm_data::enemy_speeds::SpeedTable, index: isize) -> u16 {
        Runtime::enemy_speed(self, table, index)
    }

    fn lease_enemy_speed32(&self, offset: usize) -> u32 {
        Runtime::native_speed32(self, offset)
    }

    fn lease_byte(&self, at: usize) -> u8 {
        Runtime::byte(self, at)
    }

    fn owned_lease_call<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
        Runtime::owned_call(self, action)
    }
}

impl LeaseActors for Game {
    type Lease<'a> = Runtime<'a>;

    fn lease(&mut self) -> Runtime<'_> {
        Runtime::new(self)
    }
}

impl sm_game::actors::etecoon::owned::Host for Runtime<'_> {
    fn probe_horizontal(&mut self, key: u16, amount: i32) -> bool {
        sm_game::actors::movement::probe_horizontal(
            self,
            sm_game::actors::access::ActorKey::from_legacy(key),
            amount,
        ) == sm_game::actors::api::MovementResult::Blocked
    }
    fn invalid_function(&mut self, _address: u16) -> ! {
        sm_game::actors::remaining_core::Core::remaining_unknown(self)
    }
}
impl sm_game::actors::dachora_run::owned::Host for Runtime<'_> {
    fn frame(&self) -> u16 {
        sm_game::actors::boss_lease::BossState::frame(self)
    }
    fn load_palette(&mut self, source: u16, key: u16) {
        sm_game::actors::dachora::load_palette(source, key, self);
    }
    fn load_shine_palette(&mut self, key: u16) {
        sm_game::actors::dachora_run::load_shine_palette(self, key);
    }
    fn dachora_palette(&mut self, charging: bool, index: isize) -> u16 {
        sm_game::boss_lists::dachora_palette(self, charging, index)
    }
    fn invalid_function(&mut self, _address: u16) -> ! {
        sm_game::actors::remaining_core::Core::remaining_unknown(self)
    }
}
impl sm_game::actors::mini_kraid::owned::Host for Runtime<'_> {
    fn spawn_actor_eproj(&mut self, parameter: u16, key: u16, definition: u16) -> u16 {
        sm_game::enemy_projectiles::lifecycle::spawn_from_enemy(self, parameter, key, definition)
    }
    fn normal_death(&mut self, animation: u16) {
        sm_game::actors::deaths::Deaths::normal_death(self, animation);
    }
    fn scatter_enemy_drop(
        &mut self,
        source: sm_game::enemy_projectiles::drops::DropSource,
        key: u16,
    ) {
        sm_game::actors::deaths::Deaths::scatter_enemy_drop(self, source, key);
    }
}
impl sm_game::actors::zebetites::owned::Host for Runtime<'_> {
    fn placement(&self, destroyed: isize) -> sm_data::zebetites::Placement {
        sm_game::zebetite_data::placement(self, destroyed)
    }
    fn health_lists(&self, stage: isize) -> sm_data::zebetites::HealthLists {
        sm_game::zebetite_data::health_lists(self, stage)
    }
    fn event_happened(&mut self, event: u16) -> u16 {
        u16::from(sm_game::progress::event_happened(self, event))
    }
    fn set_event(&mut self, event: u16) {
        sm_game::progress::set_event(self, event);
    }
    fn clear_event(&mut self, event: u16) {
        sm_game::progress::clear_event(self, event);
    }
    fn spawn_population(&mut self, bank: u8, population: u16) -> u16 {
        sm_game::actors::population::spawn_legacy(self, bank, population)
    }
    fn glow(&mut self) {
        sm_game::actors::zebetites::glow(self);
    }
    fn normal_death(&mut self, animation: u16) {
        sm_game::actors::deaths::Deaths::normal_death(self, animation);
    }
    fn invalid_function(&mut self, _state: u16) -> ! {
        sm_game::actors::remaining_core::Core::remaining_unknown(self)
    }
    fn react_to_contact(&mut self, death: sm_game::actors::host::Death) {
        sm_game::actors::host::ActorRuntime::contact_reaction(self, death);
    }
    fn react_to_shot(&mut self, shot: sm_game::actors::host::Shot) {
        sm_game::actors::host::ActorRuntime::shot_reaction(self, shot);
    }
}
