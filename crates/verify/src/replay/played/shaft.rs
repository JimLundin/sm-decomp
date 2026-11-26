//! The Ceres shaft's pad and the cover below it: their compared fields.

use crate::replay::compare::Observation;
use crate::replay::result::Outcome;
use crate::replay::shaft;
use game::production::played;

/// The pad and cover as the record holds them, compared at every cut.
pub(super) fn expected(wram: &[u8], elevator: bool) -> Result<[Observation; 2], Outcome> {
    Ok(shaft::observations(&shaft::pad_cover_with_elevator(
        wram, elevator,
    )?))
}

/// The pad and cover as the played State holds them.
pub(super) fn actual(station: &played::station::State) -> [Observation; 2] {
    shaft::observations(&station.pad_and_cover)
}

pub(super) fn burst_art(
    rom: data::rom::Lorom<'_>,
) -> Result<game::production::played::shaft::BurstArt, super::Outcome> {
    import::room_effects::shaft_burst(rom)
        .map(game::production::played::shaft::BurstArt::new)
        .map_err(|_| super::data_failure(None, "shaft burst clip content"))
}
