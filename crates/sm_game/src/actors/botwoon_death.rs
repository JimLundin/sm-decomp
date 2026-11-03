//! Botwoon's death check ($B3:96C6): the hit, tail and body landing handshake
//! words held by the Session.
//! Unknown words are retained; a nonzero signal has the original meaning.
use super::access::{AccessMode::Native, ActorAccess};
use super::slot_view::{SlotMut, SlotRef, SlotState};
use crate::actors::botwoon::function;
use crate::enemy_projectiles::host::{BotwoonField as B, Host as EprojHost};

/// The pre-death counter starts at F0h and the delay ends at 100h.
const PRE_DEATH_START: u16 = 0xf0;

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

impl Signal {
    /// The extended word of this signal in an actor's slot ($7E:801E, $8020,
    /// $803C, $8818 and $883E for slot 0).
    fn var(self) -> usize {
        match self {
            Signal::PendingHit => 0x2f,
            Signal::Active => 0x30,
            Signal::TailReady => 0x3e,
            Signal::PreviousHealth => 0x4c,
            Signal::BodyLanded => 0x5f,
        }
    }
}

/// The collapse delay's extended word ($7E:8004 for slot 0).
const COLLAPSE_DELAY: usize = 0x22;

/// The death signals and collapse delay through the slot view: slot 0's
/// signals are its death protocol (whose stores refresh the observations, as
/// the projected stores did), any other slot's (and the delay) that slot's
/// extended words.
pub trait Host: ActorAccess + SlotState {
    fn death_signal(&self, enemy: u16, signal: Signal) -> u16 {
        SlotRef::of(self, enemy).extended(signal.var())
    }
    fn set_death_signal(&mut self, enemy: u16, signal: Signal, value: u16) {
        *SlotMut::of(self, enemy).extended(signal.var()) = value;
    }
    fn set_collapse_delay(&mut self, enemy: u16, value: u16) {
        *SlotMut::of(self, enemy).extended(COLLAPSE_DELAY) = value;
    }
}
impl<T: ActorAccess + SlotState + ?Sized> Host for T {}

pub fn pending(h: &impl Host, enemy: u16) -> bool {
    h.death_signal(enemy, Signal::PendingHit) != 0
}
pub fn landed(h: &impl Host, enemy: u16) -> bool {
    h.death_signal(enemy, Signal::BodyLanded) != 0
}
pub fn arm(h: &mut impl Host) {
    let actor = h.selected_actor();
    let enemy = actor.byte_index();
    if pending(h, enemy) && h.death_signal(enemy, Signal::TailReady) != 0 {
        h.set_death_signal(enemy, Signal::Active, 1);
        h.edit_actor(actor, Native, |actor| {
            actor.variables[3] = function::DEATH_PRE_DEATH_DELAY
        });
        h.set_collapse_delay(enemy, PRE_DEATH_START);
        h.set_death_signal(enemy, Signal::PendingHit, 0);
        h.set_death_signal(enemy, Signal::TailReady, 0);
    }
}
/// Keep the projectile visibility/pose stores before this tail confirmation.
pub fn confirm_tail(h: &mut impl EprojHost, enemy: usize) {
    h.set_botwoon_field(enemy, B::TransitionPending, 0);
    h.set_botwoon_field(enemy, B::TailState, 0);
    h.set_botwoon_field(enemy, B::TransitionCursor, u16::MAX);
    if h.botwoon_field(enemy, B::Pattern) == 0 {
        h.set_botwoon_field(enemy, B::TailState, 1);
    }
}
