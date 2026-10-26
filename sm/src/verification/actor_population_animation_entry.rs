//! Verification-only actor child: actual candidate operations, no copied gameplay.
#![forbid(unsafe_code)]
use super::access::ActorKey;
use crate::{game::session_runtime::Runtime, game_state::Game};

pub(crate) fn prepare(g: &mut Game) {
    super::api::prepare_room_assets(&mut Runtime::new(g));
}
pub(crate) fn transfer(g: &mut Game) {
    super::api::transfer_actor_tiles_and_initialize(&mut Runtime::new(g));
}
pub(crate) fn spawn(bank: u8, address: u16, g: &mut Game) -> u16 {
    super::population::spawn_legacy(&mut Runtime::new(g), bank, address)
}
pub(crate) fn record(index: u16, g: &mut Game) {
    super::population::record_spawn(&mut Runtime::new(g), ActorKey::from_legacy(index));
}
pub(crate) fn respawn(index: u16, g: &mut Game) {
    super::api::respawn(&mut Runtime::new(g), ActorKey::from_legacy(index));
}
pub(crate) fn animation(g: &mut Game) {
    super::animation::advance_animation(&mut Runtime::new(g));
}
pub(crate) fn advance(g: &mut Game) {
    super::api::advance_selected(&mut Runtime::new(g));
}
// Remains INSIDE the actual A2804C dispatch, after outer Runtime ownership export.
// Observer releases its RefCell borrow before this actual nested spawn enters Runtime.
pub(crate) fn dispatch_noop(g: &mut Game) {
    super::initializer_checks::dispatch_noop(g);
    if crate::actor_population_animation_observer::dispatch_noop(g) {
        spawn(0xa1, 0x8040, g);
    }
}
