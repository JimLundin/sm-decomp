//! Typed observations in the lockstep verifier (docs/SCENARIO_HARNESS.md,
//! migration step 1): the oracle's projection and the paired frame's hooks.
//!
//! The model, the engine's typed projection, the comparison and the report
//! are `sm_tools`; this module decodes the oracle's RAM and hands both sides
//! to `sm_tools::lockstep::Lockstep` beside the unchanged RAM comparison.

pub(crate) mod oracle;

use crate::game_state::Game;
use sm_tools::lockstep::EngineSource;
pub use sm_tools::lockstep::Lockstep;

/// The engine's Tick starts: capture the frame leases it releases.
pub(crate) fn begin_engine_tick(game: &mut Game) {
    if let Some(lockstep) = game.observation.as_mut() {
        lockstep.begin_engine_tick();
    }
}

/// Project both results of the paired frame just run and compare them. The
/// oracle's is its result snapshot, before comparison normalization; the
/// engine's is the retained Session when it is current, otherwise the Session
/// of the Tick's last frame lease. `None` when observation is off.
pub(crate) fn observe(game: &mut Game) -> Option<bool> {
    let mut lockstep = game.observation.take()?;
    let start = std::time::Instant::now();
    let lease = lockstep.end_engine_tick();
    let engine = match game.observed_session() {
        Some(session) => Some((sm_tools::engine::project(session), EngineSource::Persistent)),
        None => lease.map(|projection| (projection, EngineSource::Lease)),
    };
    let oracle = oracle::project(&game.g_snapshot_theirs.ram);
    let diverged = lockstep.compare(
        game.snes_frame_counter,
        &oracle,
        engine
            .as_ref()
            .map(|(projection, source)| (projection, *source)),
    );
    lockstep.elapsed += start.elapsed();
    game.observation = Some(lockstep);
    Some(diverged)
}

/// Count the paired frame's agreement between the two comparisons.
pub(crate) fn agree(game: &mut Game, observation_failed: Option<bool>, ram_failed: bool) {
    let tick = game.snes_frame_counter;
    if let (Some(lockstep), Some(failed)) = (game.observation.as_mut(), observation_failed) {
        lockstep.agree(tick, failed, ram_failed);
    }
}
