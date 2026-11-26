//! Qualify the elapsed clock at gameplay work opportunities.

use game_time::{Clock, PlayFrames, PlayHours, PlayMinutes, PlaySeconds, PlayTime};

/// The time before a fresh file earns a gameplay frame.
pub const ZERO: PlayTime = PlayTime {
    hours: PlayHours::ZERO,
    minutes: PlayMinutes::ZERO,
    seconds: PlaySeconds::ZERO,
    frames: PlayFrames::ZERO,
};

/// Advance after a fresh ordinary or locked room turn with positive whole Health.
/// Publication and passage work do not call this entry.
#[must_use]
pub fn after_work(time: PlayTime, health: samus_health::State, fresh: bool) -> PlayTime {
    if fresh && (1..=i16::MAX.cast_unsigned()).contains(&health.energy.whole.get()) {
        Clock::restore(time).advance(game_time::QualifiedOpportunity)
    } else {
        time
    }
}
