//! Ceres Ridley and the baby: his content and their compared fields.

pub(super) mod retreat;
use crate::replay::compare::Observation;
use crate::replay::result::Outcome;
use crate::replay::{data_failure, ridley};
use boss_ceres_ridley::{RidleyContent, TAIL_SEGMENTS};
use data::assets::actors::CeresRidleyColours;
use game::production::{played, ridley as production};
use oracle_record::FrameRecord;
use types::{Pair, Px};

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
    let mut fields = ridley::observations(Some((&ridley, seen, &tail)));
    if let Some(flight) = ridley.flight_tail {
        fields.extend(places(&flight.places()));
    }
    Ok(fields)
}

/// Ridley as the played State holds him.
pub(super) fn actual(station: &played::station::State) -> Vec<Observation> {
    let Some(ridley) = station.ridley.as_ref() else {
        return ridley::observations(None);
    };
    let tail = ridley::modelled_tail_segments(ridley);
    ridley::observations(Some((ridley, ridley::Shows::of(ridley.phase), &tail)))
}

/// Read the appended cue in Ridley's sound library, independently of the tail's timer.
pub(super) fn expected_swish(
    previous: &FrameRecord,
    current: &FrameRecord,
) -> Result<Observation, Outcome> {
    let added = super::screens::Queues::read(previous)?.added(current)?;
    // The native cue is 21h in Library 3. Decode that tag independently of the model.
    Ok(swish(added.contains(&[3, 0x21, 0])))
}

/// The tail's request at the audio boundary.
pub(super) fn swish(requested: bool) -> Observation {
    Observation::quantity("ridley.tail.swish-request", i64::from(requested))
}

/// Seven positions published to drawing, compared apart from the chain that produced them.
pub(super) fn places(
    published: &[Pair<Px>; TAIL_SEGMENTS],
) -> impl Iterator<Item = Observation> + '_ {
    published.iter().enumerate().map(|(index, Pair { x, y })| {
        Observation::quantity(
            format!("ridley.tail.place.{index}"),
            (i64::from(x.0) << 16) | i64::from(y.0),
        )
    })
}
