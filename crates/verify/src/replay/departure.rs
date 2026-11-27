//! The departure fields the played run compares as the Landing Site loader starts receiving.
mod observations;
pub(super) use observations::actual;

use super::played::rows::Rows;
use super::result::Outcome;

pub(super) fn check_receiving(
    production: &game::production::Production,
    previous: &oracle_record::FrameRecord,
    current: &oracle_record::FrameRecord,
    graphics: &data::assets::landing_site::Graphics,
    rows: &mut Rows<'_>,
    cut: u64,
) -> Result<(), Outcome> {
    super::compared(
        cut,
        &observations::native(previous, current, graphics, rows, cut)?.fields,
        &actual(production, cut)?,
    )
}
