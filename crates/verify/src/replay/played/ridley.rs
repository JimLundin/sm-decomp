//! Ceres Ridley and the baby: his content and their compared fields.

use crate::replay::compare::Observation;
use crate::replay::result::Outcome;
use crate::replay::{data_failure, ridley};
use boss_ceres_ridley::RidleyContent;
use data::assets::actors::CeresRidleyColours;
use game::production::{played, ridley as production};

/// His content as the model reads it, from the imported colours.
pub(super) fn content(colours: &CeresRidleyColours) -> Result<RidleyContent, Outcome> {
    production::content(colours)
        .map_err(|production::ColourFault| data_failure(None, "Ceres Ridley content"))
}

/// Ridley as the record holds him, compared at a cut whose frame of work finished.
pub(super) fn expected(wram: &[u8]) -> Result<Vec<Observation>, Outcome> {
    let Some((ridley, seen)) = ridley::observed(wram)? else {
        return Ok(ridley::observations(None));
    };
    let tail = ridley::tail_segments(wram, ridley.phase)?;
    Ok(ridley::observations(Some((&ridley, seen, &tail))))
}

/// Ridley as the played State holds him.
pub(super) fn actual(station: &played::station::State) -> Vec<Observation> {
    let Some(ridley) = station.ridley.as_ref() else {
        return ridley::observations(None);
    };
    let tail = ridley::modelled_tail_segments(ridley);
    ridley::observations(Some((ridley, ridley::Shows::of(ridley.phase), &tail)))
}
