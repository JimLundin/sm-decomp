//! Botwoon's hit, tail and body landing handshake words held by the Session.
//! Unknown words are retained; a nonzero signal has the original meaning.
use super::access::{AccessMode::Native, ActorAccess};
use crate::enemy_projectiles::host::{BotwoonField as B, Host as EprojHost};

#[derive(Clone, Default, Debug, PartialEq)]
pub struct DeathProtocol {
    pub pending_hit: u16,
    pub active: u16,
    pub tail_ready: u16,
    pub previous_health: u16,
    pub body_landed: u16,
}

#[derive(Clone, Copy)]
pub enum Signal {
    PendingHit,
    Active,
    TailReady,
    PreviousHealth,
    BodyLanded,
}

/// Native actor accesses retain their full selector and ordered byte stores.
/// Projectile capabilities retain their separate little-endian publication.
pub trait Host: ActorAccess {
    fn native_death_signal(&self, enemy: u16, signal: Signal) -> u16;
    fn set_native_death_signal(&mut self, enemy: u16, signal: Signal, value: u16);
    fn set_collapse_delay(&mut self, enemy: u16, value: u16);
}

pub fn pending(h: &impl Host, enemy: u16) -> bool {
    h.native_death_signal(enemy, Signal::PendingHit) != 0
}
pub fn landed(h: &impl Host, enemy: u16) -> bool {
    h.native_death_signal(enemy, Signal::BodyLanded) != 0
}
pub fn arm(h: &mut impl Host) {
    let actor = h.selected_actor();
    let enemy = actor.byte_index();
    if pending(h, enemy) && h.native_death_signal(enemy, Signal::TailReady) != 0 {
        h.set_native_death_signal(enemy, Signal::Active, 1);
        h.edit_actor(actor, Native, |actor| actor.variables[3] = 0x9a46);
        h.set_collapse_delay(enemy, 240);
        h.set_native_death_signal(enemy, Signal::PendingHit, 0);
        h.set_native_death_signal(enemy, Signal::TailReady, 0);
    }
}
/// Keep the projectile visibility/pose stores before this tail confirmation.
pub fn confirm_tail(h: &mut impl EprojHost, enemy: usize) {
    h.set_botwoon_field(enemy, B::TransitionPending, 0);
    h.set_botwoon_field(enemy, B::TailState, 0);
    h.set_botwoon_field(enemy, B::TransitionCursor, 0xffff);
    if h.botwoon_field(enemy, B::Pattern) == 0 {
        h.set_botwoon_field(enemy, B::TailState, 1);
    }
}
