//! Resolve the ordinary drawing's pose and playhead to the NMI's sprite strips.

use data::assets::samus::map;
use data::pose::{animation::AnimationFrame, tiles::SamusTiles};

use super::Fault;
use crate::state::video::SamusTileUploads;
use samus_appearance::{Figure, Step};

pub(super) fn uploads(
    tiles: &SamusTiles,
    figure: Figure,
    step: Step,
) -> Result<SamusTileUploads, Fault> {
    let step = AnimationFrame(u16::from(step.0));
    let frame = map::id_of(figure.pose, figure.sprite.into())
        .and_then(|pose| tiles.frame(pose, step))
        .ok_or(Fault::SamusTiles(figure))?;
    Ok(SamusTileUploads {
        top: Some(frame.top),
        bottom: frame.bottom,
    })
}
