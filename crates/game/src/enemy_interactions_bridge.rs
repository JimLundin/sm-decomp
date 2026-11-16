//! Ordered Ceres contact against the production weapons owner's current shots.
//! Each supplied opportunity carries its current geometry and interaction permissions.

use enemy_interactions as contact;
use weapons_projectiles as beam;

/// Apply one supported door opportunity immediately, retaining the body for
/// later actors. Geometry and permissions must be read anew for each call.
#[must_use]
pub fn contact_door(
    state: &mut beam::State,
    conditions: contact::ContactConditions,
    door: contact::DoorContact,
) -> Option<beam::ShotSlot> {
    let shots = contact::Shots::from_fn(|slot| match &state.slots[beam_slot(slot)] {
        beam::ShotState::Free { .. } => contact::ShotBody::Free,
        beam::ShotState::Impact { .. } => contact::ShotBody::Impact,
        beam::ShotState::Flying { body, consumed } => contact::ShotBody::Power {
            figure: contact::ContactBox {
                centre: contact::Centre {
                    x: body.position.x.integer(),
                    y: body.position.y.integer(),
                },
                radii: contact::Radii {
                    x: u16::from(body.radius.x.0),
                    y: u16::from(body.radius.y.0),
                },
            },
            consumed: *consumed,
        },
    });
    let slot = contact::consumption(conditions, door, contact::ShotCount(state.count.0), &shots)
        .map(beam_slot)?;
    state.consume(slot);
    Some(slot)
}

const fn beam_slot(slot: contact::ShotSlot) -> beam::ShotSlot {
    match slot {
        contact::ShotSlot::Slot0 => beam::ShotSlot::Slot0,
        contact::ShotSlot::Slot1 => beam::ShotSlot::Slot1,
        contact::ShotSlot::Slot2 => beam::ShotSlot::Slot2,
        contact::ShotSlot::Slot3 => beam::ShotSlot::Slot3,
        contact::ShotSlot::Slot4 => beam::ShotSlot::Slot4,
    }
}
