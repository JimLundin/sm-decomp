//! Append the mounted vent's current puff to the shared room picture.

use crate::exact::oam::HIDDEN_Y;
use crate::intent::{ScreenPoint, SpriteDraw};
use crate::state::video::NmiBatch;
use boss_ceres_ridley::{PuffFigure, VentFacing};
use data::assets::{actors::SpriteRow, shaft::VentPictures};

/// Append a puff figure at its projected, shaken screen origin.
pub fn append(
    batch: &mut NmiBatch,
    pictures: &VentPictures,
    facing: VentFacing,
    figure: PuffFigure,
    position: ScreenPoint,
) {
    let figures = match facing {
        VentFacing::Up => &pictures.up,
        VentFacing::Left => &pictures.left,
        VentFacing::Down => &pictures.down,
        VentFacing::Right => &pictures.right,
    };
    let Some(map) = figures.get(usize::from(figure.get().saturating_sub(1))) else {
        return;
    };
    let mut draw = SpriteDraw::default();
    draw.place(map, position);
    for piece in &mut draw.pieces {
        piece.look.palette |= SpriteRow::COMMON.get();
        if !(0..256).contains(&piece.at.y) {
            piece.at.y = i32::from(HIDDEN_Y);
        }
    }
    batch.draw_sprites(&draw);
}
