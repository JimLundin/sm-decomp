//! The Ceres shaft's pad and the cover below it: their compared fields.

use crate::replay::compare::Observation;
use crate::replay::result::Outcome;
use crate::replay::shaft;
use game::production::played;

/// The pad and cover as the record holds them, compared at every cut.
pub(super) fn expected(wram: &[u8]) -> Result<[Observation; 2], Outcome> {
    Ok(shaft::observations(&shaft::pad_and_cover(wram)?))
}

/// The pad and cover as the played State holds them.
pub(super) fn actual(station: &played::station::State) -> [Observation; 2] {
    shaft::observations(&station.pad_and_cover)
}
