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
    Ok(ridley::observations(
        ridley::observed(wram)?
            .as_ref()
            .map(|(ridley, seen)| (ridley, *seen)),
    ))
}

/// Ridley as the played State holds him.
pub(super) fn actual(station: &played::station::State) -> Vec<Observation> {
    ridley::observations(
        station
            .ridley
            .as_ref()
            .map(|ridley| (ridley, ridley::Shows::of(ridley.phase))),
    )
}
