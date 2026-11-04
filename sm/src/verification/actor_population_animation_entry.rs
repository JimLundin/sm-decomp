//! Verification-only actor child: actual candidate operations, no copied gameplay.
#![forbid(unsafe_code)]
use crate::{game::session_runtime::Runtime, game_state::Game};
use sm_game::actors::access::ActorKey;

pub(crate) fn prepare(g: &mut Game) {
    crate::game::actors::api::prepare_room_assets(&mut Runtime::new(g));
}
pub(crate) fn transfer(g: &mut Game) {
    crate::game::actors::api::transfer_actor_tiles_and_initialize(&mut Runtime::new(g));
}
pub(crate) fn spawn(bank: u8, address: u16, g: &mut Game) -> u16 {
    crate::game::actors::population::spawn_legacy(&mut Runtime::new(g), bank, address)
}
pub(crate) fn record(index: u16, g: &mut Game) {
    crate::game::actors::population::record_spawn(
        &mut Runtime::new(g),
        ActorKey::from_legacy(index),
    );
}
pub(crate) fn respawn(index: u16, g: &mut Game) {
    crate::game::actors::api::respawn(&mut Runtime::new(g), ActorKey::from_legacy(index));
}
pub(crate) fn animation(g: &mut Game) {
    crate::game::actors::animation::advance_animation(&mut Runtime::new(g));
}
pub(crate) fn advance(g: &mut Game) {
    crate::game::actors::api::advance_selected(&mut Runtime::new(g));
}
// Remains INSIDE the actual A2804C dispatch, after outer Runtime ownership export.
// Observer releases its RefCell borrow before this actual nested spawn enters Runtime.
pub(crate) fn dispatch_noop(g: &mut Game) {
    crate::game::actors::initializer_checks::dispatch_noop(g);
    if crate::actor_population_animation_observer::dispatch_noop(g) {
        spawn(0xa1, 0x8040, g);
    }
}
/// The $A2:804C AI on the scheduler's lease. Only an armed initializer or population case
/// observes it, and those check the payload the game owns inside the dispatch, so they
/// run on the game exported from the lease, as the engine's former foreign fallback ran
/// them; otherwise the RTL does nothing and the lease stays held.
pub(crate) fn dispatch_noop_on_lease(runtime: &mut Runtime) {
    if crate::game::actors::initializer_checks::armed()
        || crate::actor_population_animation_observer::noop_armed()
    {
        runtime.foreign(dispatch_noop);
    }
}
