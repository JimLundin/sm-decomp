//! The Ceres doors: their seed and compared fields.

use crate::replay::compare::Observation;
use crate::replay::result::Outcome;
use crate::replay::{ceres, movement};
use game::production::played;
use types::Hitbox;

/// The seed room's Ceres doors, decoded once from the seed record. Her motion meets each one
/// as the loop carries it, so none may stand among the actor boxes held from the seed too.
///
/// Her motion meets the boxes in the original's slot order, and the first that blocks her
/// decides the move. Production puts the doors' boxes after the held ones, since the held
/// geometry keeps no slots, so a held box in a slot after a door's would be met out of order.
pub(super) fn seed(seed: &[u8]) -> Result<Vec<boss_ceres_ridley::CeresDoor>, Outcome> {
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

/// The seed room's Ceres doors and whether they own the entire observed collision roster.
///
/// Her motion meets the boxes in the original's slot order, and the first that blocks her
/// decides the move. Production puts the doors' boxes after the held ones, since the held
/// geometry keeps no slots, so a held box in a slot after a door's would be met out of order.
pub(super) fn seed_with_roster(
    seed: &[u8],
    native_boxes: &[Hitbox],
    supported_passage: bool,
) -> Result<(Vec<boss_ceres_ridley::CeresDoor>, bool), Outcome> {
    let slots = ceres::doors(seed)?;
    let roster = movement::roster_slots(seed)?;
    let refuse = |dependency: &str| Outcome::Unsupported {
        cut: None,
        dependency: dependency.into(),
    };
    let owns_roster = slots.iter().any(|slot| roster.contains(&slot.index()));
    let first_door = slots.iter().map(|slot| slot.index()).min();
    if owns_roster {
        if (!supported_passage && super::passage::passing(seed)?)
            || roster
                .iter()
                .any(|&index| !slots.iter().any(|slot| slot.index() == index))
            || roster.array_windows().any(|[first, next]| first >= next)
        {
            return Err(refuse(
                "played seed holds a Ceres door among the held actor boxes",
            ));
        }
    } else if first_door.is_some_and(|door| roster.iter().any(|&held| held > door)) {
        return Err(refuse(
            "played seed holds an actor box in a slot after a Ceres door's",
        ));
    }
    let doors = slots
        .into_iter()
        .map(|slot| ceres::seed(seed, slot))
        .collect::<Result<Vec<_>, _>>()?;
    // A passage has no Samus movement query. Its interactive roster can still describe
    // the departing frame while the room's new door owners have already been placed.
    if owns_roster
        && !supported_passage
        && !doors
            .iter()
            .filter(|door| door.phase.solid())
            .map(|door| door.hitbox())
            .eq(native_boxes.iter().cloned())
    {
        return Err(refuse(
            "played seed Ceres door boxes differ from the roster",
        ));
    }
    Ok((doors, owns_roster))
}

/// The doors as the record holds them, compared at a cut whose frame of work finished.
pub(super) fn expected(wram: &[u8]) -> Result<Vec<Observation>, Outcome> {
    Ok(observations(
        ceres::doors(wram)?
            .into_iter()
            .map(|slot| {
                let mut parts = ceres::expected(wram, slot)?;
                parts.push(ceres::drawn_with_ridley(wram, slot)?);
                Ok(parts)
            })
            .collect::<Result<_, _>>()?,
    ))
}

/// The doors as the played State holds them.
pub(super) fn actual(station: &played::station::State) -> Vec<Observation> {
    observations(
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
    )
}

/// The room's Ceres doors by part, each named by its place in the room's order of its enemies,
/// after how many there are, so a difference names the door and the part that differ.
fn observations(doors: Vec<Vec<Observation>>) -> Vec<Observation> {
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
