//! The Ceres station's State as the seed holds it. Its actors' compared fields are their own
//! modules': `ceres_doors`, `ridley` and `shaft`.

use crate::replay::result::Outcome;
use crate::replay::{ceres, ridley, shaft};
use game::production::played;

/// The station as the seed holds it, with the seed room's Ceres `doors` and the held `stage`.
pub(super) fn seed(
    wram: &[u8],
    doors: Vec<boss_ceres_ridley::CeresDoor>,
    stage: boss_ceres_ridley::Stage,
) -> Result<played::station::State, Outcome> {
    Ok(played::station::State {
        doors,
        vents: super::steam::decode(wram)?,
        pad_and_cover: shaft::pad_and_cover(wram)?,
        shaft_overlay: ceres::shaft_overlay(wram)?,
        ridley: ridley::seed(wram)?,
        stage,
    })
}
