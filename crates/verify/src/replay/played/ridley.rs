//! Ceres Ridley and the baby: their compared fields.

use crate::replay::compare::Observation;
use crate::replay::result::Outcome;
use crate::replay::ridley;
use game::production::played;

/// Ridley as the record holds him, compared at a cut whose frame of work finished.
pub(super) fn expected(wram: &[u8]) -> Result<Vec<Observation>, Outcome> {
    Ok(ridley::observations(ridley::seed(wram)?.as_ref()))
}

/// Ridley as the played State holds him.
pub(super) fn actual(station: &played::station::State) -> Vec<Observation> {
    ridley::observations(station.ridley.as_ref())
}
