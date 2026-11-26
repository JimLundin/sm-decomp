//! Place Samus after the countdown and prepare her separate NMI graphics transfer.

use data::spritemap::{SpriteAttributes, SpriteSize};
use samus_appearance::{
    BodyRow, ElevatorDrawing, ElevatorPictureError, ElevatorProjection, ObjectSize, Step,
    elevator_standing_picture,
};
use types::{Pair, Pose};

use crate::intent::{Placing, SpriteDraw, SpritePiece};
use crate::samus_appearance_bridge::elevator::Art;
use crate::state::video::{NmiBatch, OAM_SPRITES, StandingStrips};

use super::Fault;

/// Standing pose and pre-rotation geometry at a locked room drawing opportunity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StandingFrame {
    /// The forward standing pose in the ordinary Power suit, with no flicker.
    pub pose: Pose,
    /// The displayed clip step.
    pub step: Step,
    /// Geometry at drawing, with the shaft view before its late rotation.
    pub projection: ElevatorProjection,
}

pub(super) fn prepare(batch: &mut NmiBatch, frame: StandingFrame, art: &Art) -> Result<(), Fault> {
    let body = elevator_standing_picture(
        ElevatorDrawing::Locked {
            pose: frame.pose,
            step: frame.step,
            projection: frame.projection,
        },
        &art.pictures,
    )
    .map_err(|error| match error {
        ElevatorPictureError::UnsupportedPose => Fault::StandingPose,
        ElevatorPictureError::MissingArt(_) => Fault::StandingArt,
    })?
    .ok_or(Fault::StandingPicture)?;
    let start = usize::from(batch.sprites.0);
    let end = start.saturating_add(body.placements.len());
    if end > OAM_SPRITES {
        return Err(Fault::StandingPicture);
    }

    let draw = SpriteDraw {
        pieces: body
            .placements
            .iter()
            .map(|piece| {
                let at = piece.position.0;
                let style = piece.style;
                let low = |pixels: i64| {
                    let [a, b, c, d, ..] = pixels.to_le_bytes();
                    i32::from_le_bytes([a, b, c, d])
                };
                SpritePiece {
                    at: Pair {
                        x: low(at.x),
                        y: low(at.y),
                    },
                    look: SpriteAttributes {
                        tile: style.tile,
                        palette: style.palette,
                        priority: style.priority,
                        flip_x: style.flips.x,
                        flip_y: style.flips.y,
                    },
                    placing: Placing::Mapped(match style.size {
                        ObjectSize::Small => SpriteSize::Small,
                        ObjectSize::Large => SpriteSize::Large,
                    }),
                }
            })
            .collect(),
    };
    // The writer only sets bits in the shared high table. Clear our slots before drawing.
    for index in start..end {
        if let Some(high) = batch.oam.high.get_mut(index / 4) {
            *high &= !(3 << ((index % 4).wrapping_mul(2)));
        }
    }
    batch.draw_sprites(&draw);
    batch.standing_strips = StandingStrips {
        upper_first: body.strips[BodyRow::UpperFirst].copied(),
        upper_second: body.strips[BodyRow::UpperSecond].copied(),
        lower_first: body.strips[BodyRow::LowerFirst].copied(),
        lower_second: body.strips[BodyRow::LowerSecond].copied(),
    };
    Ok(())
}
