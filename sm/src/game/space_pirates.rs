//! Space pirates run in `sm_game::actors::space_pirates`; the game side of
//! their hosts lives here.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
use crate::rom::RomRecord;
use crate::types::{EnemyDef, Vulnerability};
use core::mem::offset_of;
use sm_game::actors::host::ActorRuntime;
pub(crate) use sm_game::actors::space_pirates::*;

const _: () = {
    assert!(offset_of!(EnemyDef, vulnerability_ptr) == DEFINITION_VULNERABILITY);
    assert!(offset_of!(Vulnerability, power) == VULNERABILITY_POWER);
    assert!(offset_of!(Vulnerability, plasma_ice_wave) == VULNERABILITY_PLASMA_ICE_WAVE);
};

/// The pirates' spawns on their lease: the spawn parameters are the Session's, which the
/// game's native spawn fields were imported into.
impl PirateHost for Runtime<'_> {
    fn fire_laser_projectile(&mut self, x: u16, y: u16, right: bool) {
        let spawn = &mut self.session_mut().enemy_projectiles.spawn;
        (spawn.point.x, spawn.point.y) = (x, y);
        spawn.variant = u16::from(right);
        ActorRuntime::spawn_room_projectile(self, 0xa17b, u16::from(right));
    }

    fn spawn_in_rect(&mut self, x: u16, y: u16, width: u16, height: u16, parameter: u16) {
        let rectangle = &mut self.session_mut().enemy_projectiles.spawn.rectangle;
        (rectangle.origin.x, rectangle.origin.y) = (x, y);
        (rectangle.extent.x, rectangle.extent.y) = (width, height);
        let enemy = self.session().actors.controls.selected;
        ActorRuntime::spawn_projectile(self, parameter, enemy, 0xa189);
    }
}

impl PirateLease for Runtime<'_> {
    fn word(&self, at: usize) -> u16 {
        Runtime::word(self, at)
    }

    fn write_word_le(&mut self, value: u16, at: usize) {
        Runtime::write_word_le(self, value, at);
    }

    fn owned_step<T>(&mut self, action: impl FnOnce(&mut Self) -> T) -> T {
        self.owned_player_call(action)
    }

    fn actor_legacy_record(&mut self, bank: u8, address: u16) -> RomRecord {
        Runtime::actor_legacy_record(self, bank, address)
    }

    fn legacy_native_record(&mut self, bank: u8, address: u16) -> RomRecord {
        Runtime::legacy_native_record(self, bank, address)
    }
}
