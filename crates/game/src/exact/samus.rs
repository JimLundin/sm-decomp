//! Encode Samus's ordinary body from her imported pose and animation step.

use data::assets::samus::SamusAssets;
use samus_appearance::{ArtDisplacement, ElevatorProjection, Figure, ShaftView, Step};
use types::{Pair, Parts, Position};

use crate::intent::{ScreenPoint, SpriteDraw};
use crate::state::video::Mode7;

/// Project the room centre through its carried view at drawing.
#[must_use]
pub fn project(centre: Position, camera: Position, rotation: Option<Mode7>) -> ScreenPoint {
    let pixels = |at: Position| Pair {
        x: i32::from(at.x.pixel().0),
        y: i32::from(at.y.pixel().0),
    };
    let centre = pixels(centre);
    let camera = pixels(camera);
    let Some(view) = rotation else {
        return Pair {
            x: centre.x.wrapping_sub(camera.x),
            y: centre.y.wrapping_sub(camera.y),
        };
    };
    let [diagonal, cross, ..] = view.matrix;
    let [pivot_x, pivot_y] = view.centre;
    let point = ElevatorProjection {
        centre,
        pivot: Pair {
            x: i32::from(pivot_x),
            y: i32::from(pivot_y),
        },
        camera_origin: camera,
        carried_view: ShaftView { diagonal, cross },
    }
    .origin(ArtDisplacement(0));
    let narrow = |value: i64| {
        let [a, b, c, d, ..] = value.to_le_bytes();
        i32::from_le_bytes([a, b, c, d])
    };
    Pair {
        x: narrow(point.0.x),
        y: narrow(point.0.y),
    }
}

/// Prepare the body at its projected centre. The caller supplies the draw opportunity.
///
/// The art selects the top map, optional chest sprite and bottom map in drawing order.
/// Returns `None` when the selected figure, step or any named map is missing.
#[must_use]
pub fn draw(
    art: &SamusAssets,
    figure: Figure,
    step: Step,
    centre: ScreenPoint,
) -> Option<SpriteDraw> {
    let clip = art.clip(figure.pose, figure.sprite.into())?;
    let frame = clip.frames.get(usize::from(step.0))?;
    let origin = Pair {
        x: centre.x,
        y: centre.y.wrapping_add(i32::from(frame.origin_y)),
    };
    let mut draw = SpriteDraw::default();
    if let Some(top) = frame.top {
        draw.place(art.sheet.sprite(top)?, origin);
    }
    if frame.chest {
        draw.lone(
            Pair {
                x: centre.x.wrapping_sub(i32::from(art.chest.left)),
                y: centre.y.wrapping_sub(i32::from(art.chest.up)),
            },
            art.chest.attributes,
        );
    }
    if let Some(bottom) = frame.bottom {
        draw.place(art.sheet.sprite(bottom)?, origin);
    }
    Some(draw)
}
