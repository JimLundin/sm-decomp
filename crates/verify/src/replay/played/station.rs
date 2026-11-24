//! The Ceres station's doors, the shaft's pad and cover, and Ridley: their seed and compared
//! fields.

use crate::replay::compare::Observation;
use crate::replay::result::Outcome;
use crate::replay::{ceres, movement, ridley, shaft};
use game::production::played;

/// The station as the seed holds it, with the seed room's Ceres `doors` and the held `stage`.
pub(super) fn seed(
    wram: &[u8],
    doors: Vec<boss_ceres_ridley::CeresDoor>,
    stage: boss_ceres_ridley::Stage,
) -> Result<played::station::State, Outcome> {
    Ok(played::station::State {
        doors,
        pad_and_cover: shaft::pad_and_cover(wram)?,
        shaft_overlay: ceres::shaft_overlay(wram)?,
        ridley: ridley::seed(wram)?,
        stage,
    })
}

/// The seed room's Ceres doors, decoded once from the seed record. Her motion meets each one
/// as the loop carries it, so none may stand among the actor boxes held from the seed too.
///
/// Her motion meets the boxes in the original's slot order, and the first that blocks her
/// decides the move. Production puts the doors' boxes after the held ones, since the held
/// geometry keeps no slots, so a held box in a slot after a door's would be met out of order.
pub(super) fn seed_doors(seed: &[u8]) -> Result<Vec<boss_ceres_ridley::CeresDoor>, Outcome> {
    let slots = ceres::doors(seed)?;
    let roster = movement::roster_slots(seed)?;
    let refuse = |dependency: &str| Outcome::Unsupported {
        cut: None,
        dependency: dependency.into(),
    };
    if slots.iter().any(|slot| roster.contains(&slot.index())) {
        return Err(refuse(
            "played seed holds a Ceres door among the held actor boxes",
        ));
    }
    let first_door = slots.iter().map(|slot| slot.index()).min();
    if first_door.is_some_and(|door| roster.iter().any(|&held| held > door)) {
        return Err(refuse(
            "played seed holds an actor box in a slot after a Ceres door's",
        ));
    }
    slots
        .into_iter()
        .map(|slot| ceres::seed(seed, slot))
        .collect()
}

/// The doors and Ridley as the record holds them, compared at a cut whose frame of work
/// finished.
pub(super) fn expected(wram: &[u8]) -> Result<Vec<Observation>, Outcome> {
    let mut fields = door_observations(
        ceres::doors(wram)?
            .into_iter()
            .map(|slot| {
                let mut parts = ceres::expected(wram, slot)?;
                parts.push(ceres::drawn_with_ridley(wram, slot)?);
                Ok(parts)
            })
            .collect::<Result<_, _>>()?,
    );
    fields.extend(ridley::observations(ridley::seed(wram)?.as_ref()));
    Ok(fields)
}

/// The doors and Ridley as the played State holds them.
pub(super) fn actual(station: &played::station::State) -> Vec<Observation> {
    let mut fields = door_observations(
        station
            .doors
            .iter()
            .map(|door| {
                let mut parts = ceres::initial(door);
                parts.push(ceres::drawn_with_ridley_observation(
                    ceres::drawn_with_ridley_of(door),
                ));
                parts
            })
            .collect(),
    );
    fields.extend(ridley::observations(station.ridley.as_ref()));
    fields
}

/// The shaft's pad and cover as the record holds them, compared at every cut.
pub(super) fn pad_expected(wram: &[u8]) -> Result<[Observation; 2], Outcome> {
    Ok(shaft::observations(&shaft::pad_and_cover(wram)?))
}

/// The shaft's pad and cover as the played State holds them.
pub(super) fn pad_actual(station: &played::station::State) -> [Observation; 2] {
    shaft::observations(&station.pad_and_cover)
}

/// The room's Ceres doors by part, each named by its place in the room's order of its enemies,
/// after how many there are, so a difference names the door and the part that differ.
fn door_observations(doors: Vec<Vec<Observation>>) -> Vec<Observation> {
    let count = i64::try_from(doors.len()).unwrap_or(i64::MAX);
    let mut fields = vec![Observation::quantity("ceres.doors", count)];
    for (place, parts) in doors.into_iter().enumerate() {
        fields.extend(parts.into_iter().map(|part| Observation {
            field: part.field.replacen("ceres.", &format!("ceres.{place}."), 1),
            bytes: part.bytes,
        }));
    }
    fields
}
