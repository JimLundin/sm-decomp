//! The camera as the engine meets it: the camera model (`clean_model_camera`).
//!
//! The model is stepped over the engine's room and Samus, and written back to the engine's
//! camera and her previous position (`docs/behaviours/camera.decode.md`, "State" and "Inputs").
//! The engine keeps the camera as the game keeps it: layer 1's position and the point it last
//! saw her, which is her previous position. The model's camera is rebuilt from those each frame
//! and written back after its step.

use clean_model_camera::{
    Camera as Model, Frame, Heights, LeadSetting, Room, Rules, Samus as Her, ScreenKind,
};
use clean_types::{Facing as Side, Fixed, Pair};
use core::num::Wrapping;

use crate::model::num::Fixed as Exact;
use crate::state::room::{Camera, CameraDistance, ScrollArea, Scrollers, StaleScroll};
use crate::state::samus::{Facing, MovementType, Point, Samus, SpeedMode, VerticalMotion};

/// The model's quantity from the engine's.
fn quantity(value: crate::state::samus::Fixed) -> Fixed {
    Wrapping(Exact::from(value).0)
}

/// The engine's quantity from the model's.
fn exact(value: Fixed) -> crate::state::samus::Fixed {
    Exact(value.0).into()
}

/// The model's point from the engine's.
fn pair(point: Point) -> Pair<Fixed> {
    Pair {
        x: quantity(point.x),
        y: quantity(point.y),
    }
}

/// The engine's point from the model's.
fn point(pair: Pair<Fixed>) -> Point {
    Point {
        x: exact(pair.x),
        y: exact(pair.y),
    }
}

/// A screen's kind by its scroll area: red is closed, blue short, green open.
#[must_use]
pub const fn screen(area: ScrollArea) -> ScreenKind {
    match area {
        ScrollArea::Red => ScreenKind::Closed,
        ScrollArea::Blue => ScreenKind::Short,
        ScrollArea::Green => ScreenKind::Open,
    }
}

/// The room as the camera reads it: its scroll areas, `width` screens to a row, then the rows
/// the scroll buffer holds after them (`BelowTheRoom`), and its two aim heights.
///
/// The areas and the buffer run on as one table, row after row, as the game indexes them: a
/// row the areas leave short is the first of the rows below.
#[must_use]
pub fn room(
    areas: &[ScrollArea],
    stale: &[StaleScroll],
    width: usize,
    scrollers: Scrollers,
) -> Room {
    let kinds: Vec<ScreenKind> = areas
        .iter()
        .copied()
        .chain(stale.iter().map(|stale| stale.area()))
        .map(screen)
        .collect();
    let rows: Vec<Vec<ScreenKind>> = if width == 0 {
        Vec::new()
    } else {
        kinds.chunks(width).map(<[ScreenKind]>::to_vec).collect()
    };
    let height = areas.len().checked_div(width).unwrap_or(0);
    let (screens, below) = rows.split_at(height.min(rows.len()));
    Room {
        screens: screens.to_vec(),
        below: below.to_vec(),
        heights: Heights {
            level: Wrapping(scrollers.up),
            rising: Wrapping(scrollers.down),
        },
    }
}

/// The lead setting by the camera distance.
#[must_use]
pub const fn lead(distance: CameraDistance) -> LeadSetting {
    match distance {
        CameraDistance::Leading => LeadSetting::Ahead,
        CameraDistance::Near => LeadSetting::Near,
        CameraDistance::LeftEdge => LeadSetting::KeepLeft,
        CameraDistance::RightEdge => LeadSetting::KeepRight,
    }
}

/// What the camera reads of her, from the engine's State at the scroll.
///
/// - `camera_from` is her previous y: where her motion's step placed her (pose fit, a change of
///   shape) it has already written the placement there (`samus_world::step`), and otherwise it
///   is the y the camera last saw her at, so the model's placement changes nothing.
/// - Going backwards is a skid (turning round, her speed mode) or a moonwalk (her movement
///   type). The engine has no knockback yet (the damage domain), so none turns the lead.
/// - Rising is her rise state, the vertical direction up, which holds from the rise's start
///   through the frame that spends it.
/// - Her facing is her pose's; facing the screen, or a facing the pose table doesn't name,
///   leads as facing the screen does, since only facing left changes the lead.
#[must_use]
pub fn her(samus: &Samus) -> Her {
    let motion = &samus.motion;
    Her {
        position: Pair {
            x: Wrapping(motion.position.x.0),
            y: Wrapping(motion.position.y.0),
        },
        camera_from: quantity(samus.previous_position.y),
        facing: match samus.facing {
            Facing::Left => Side::Left,
            Facing::Right => Side::Right,
            Facing::Forward | Facing::Unknown(_) => Side::Screen,
        },
        backwards: motion.mode == SpeedMode::TurningAround
            || samus.movement == MovementType::Moonwalking,
        rising: motion.vertical == VerticalMotion::Up,
    }
}

/// The camera's frame through the model over `room`.
///
/// Layer 1 follows her, and the point it saw her at becomes her position. Time is never
/// stopped: the engine has none of the states that stop it yet (the x-ray scope, the reserve
/// tanks' refill).
pub fn follow(camera: &mut Camera, samus: &mut Samus, room: &Room) {
    let model = Model {
        position: Pair {
            x: quantity(camera.x),
            y: quantity(camera.y),
        },
        seen: pair(samus.previous_position),
    };
    let frame = Frame {
        samus: her(samus),
        room,
        lead: lead(camera.distance),
        time_stopped: false,
    };
    let after = model.step(&Rules::REFERENCE, &frame);
    let Point { x, y } = point(after.position);
    (camera.x, camera.y) = (x, y);
    samus.previous_position = point(after.seen);
}

#[cfg(test)]
mod tests;
