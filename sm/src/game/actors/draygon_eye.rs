//! Draygon's eye runs in `sm_game::actors::draygon_eye`; the actor lease for
//! Game is the compat Runtime.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
pub(crate) use sm_game::actors::draygon_eye::*;
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
}

impl LeaseActors for Game {
    type Lease<'a> = Runtime<'a>;

    fn lease(&mut self) -> Runtime<'_> {
        Runtime::new(self)
    }
}
