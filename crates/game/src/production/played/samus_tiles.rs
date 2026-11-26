//! Resolve the ordinary drawing's pose and playhead to the NMI's sprite strips.

use data::assets::samus::{
    key::{Dress, SpinSprite, Suit},
    map,
};
use data::pose::{animation::AnimationFrame, tiles::SamusTiles};
use types::Sprite;

use super::Fault;
use crate::state::video::SamusTileUploads;
use samus_appearance::{Figure, Step};

pub(super) fn uploads(
    tiles: &SamusTiles,
    figure: Figure,
    step: Step,
) -> Result<SamusTileUploads, Fault> {
    let sprite = figure.sprite;
    let dress = Dress {
        spin: match sprite {
            Sprite::Plain | Sprite::OtherSuit => SpinSprite::Plain,
            Sprite::SpaceJump => SpinSprite::SpaceJump,
            Sprite::ScrewAttack => SpinSprite::ScrewAttack,
        },
        suit: if sprite == Sprite::OtherSuit {
            Suit::VariaOrGravity
        } else {
            Suit::First
        },
    };
    let step = AnimationFrame(u16::from(step.0));
    let frame = map::id_of(figure.pose, dress)
        .and_then(|pose| tiles.frame(pose, step))
        .ok_or(Fault::SamusTiles(figure))?;
    Ok(SamusTileUploads {
        top: Some(frame.top),
        bottom: frame.bottom,
    })
}
