//! The one rule by which her animation's counts run down: the time left on her step, a frame at
//! a time (`ClipPlays`), and the laps left of a boost stage, a lap at a time (`BoostStages`).

/// A count run down by one: less 1, above 0, that is what is left; at 0 or below, the count is
/// over at its last. So a count of 1 and a count of 0 are over alike.
#[must_use]
pub const fn counted_down(count: u8) -> Option<u8> {
    match count.checked_sub(1) {
        Some(left) if left > 0 => Some(left),
        Some(_) | None => None,
    }
}
