//! Connect the shaft's ordinary movement and view to Appearance and its OAM consumer.

use data::assets::samus::WallJumpDustArt;
use enum_map::{Enum, enum_map};
use samus_appearance::{DustArrival, DustClip, DustDrawing, DustLook};
use types::{Action, Pair, Parts, Px, Side};

pub(in crate::production::played) use samus_appearance::DustPlacement as Placement;

use super::super::{Content, Fault, samus::Her};
use crate::{intent::SpriteDraw, state::video::NmiBatch};

pub(super) fn advance(
    her: Option<&mut Her>,
    view: Pair<Px>,
    content: &Content,
) -> Result<Placement, Fault> {
    let Some(her) = her else {
        return Ok(Placement::None);
    };
    let drawing = if let Some(outcome) = &her.outcome {
        let art = content
            .wall_jump_dust_art
            .as_ref()
            .ok_or(Fault::WallJumpDustArt)?;
        let [first, second, third, fourth] = art.visits;
        let clip = DustClip(enum_map! {
            DustLook::First => first,
            DustLook::Second => second,
            DustLook::Third => third,
            DustLook::Fourth => fourth,
        });
        let arrival = if let Some(pose) = outcome
            .pose_change
            .filter(|pose| pose.action == Action::WallJumping)
        {
            let radius = content.movement.radius(pose).ok_or(Fault::Body(pose))?;
            DustArrival::WallJump {
                facing: Side::of(pose.facing),
                centre: Pair {
                    x: her.samus.position.x.pixel(),
                    y: her.samus.position.y.pixel(),
                },
                half_height: radius.y.0,
            }
        } else if outcome.touchdown_graphics {
            // The shaft is dry, so this graphics opportunity clears the landing dust.
            DustArrival::DryTouchdown
        } else {
            DustArrival::None
        };
        DustDrawing::Fresh {
            arrival,
            view,
            clip,
        }
    } else {
        DustDrawing::Skipped
    };
    Ok(her.draw_wall_jump_dust(drawing))
}

pub(in crate::production::played) fn append(
    batch: &mut NmiBatch,
    placement: Placement,
    art: Option<&WallJumpDustArt>,
) -> Result<(), Fault> {
    match placement {
        Placement::None => (),
        Placement::HorizontalOnly { left } => batch.write_clipped_puff_x(left),
        Placement::SmallSprite { left, top, look } => {
            let attributes = art
                .and_then(|art| art.looks.get(look.into_usize()))
                .copied()
                .ok_or(Fault::WallJumpDustArt)?;
            let mut draw = SpriteDraw::default();
            draw.lone(
                Pair {
                    x: i32::from(left),
                    y: i32::from(top),
                },
                attributes,
            );
            batch.draw_sprites(&draw);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dust_state_has_the_pinned_canonical_shape() {
        assert_eq!(
            canonical::model::pin_holds::<crate::state::owned::AppearancePin>(),
            Ok(())
        );
    }

    #[test]
    fn clipped_and_held_dust_preserve_the_next_slot_until_a_whole_sprite_uses_it() {
        use core::num::NonZeroU16;
        use data::spritemap::SpriteAttributes;

        let mut batch = NmiBatch::default();
        let held = batch.clone();
        append(&mut batch, Placement::None, None).unwrap();
        assert_eq!(batch, held);

        append(&mut batch, Placement::HorizontalOnly { left: 17 }, None).unwrap();
        assert_eq!(batch.oam.low[0].x, 17);
        assert_eq!(batch.sprites, held.sprites);

        let art = WallJumpDustArt {
            looks: [
                SpriteAttributes::default(),
                SpriteAttributes::default(),
                SpriteAttributes::default(),
                SpriteAttributes::from_word(0x24ad),
            ],
            visits: [NonZeroU16::MIN; 4],
        };
        append(
            &mut batch,
            Placement::SmallSprite {
                left: 19,
                top: 23,
                look: DustLook::Fourth,
            },
            Some(&art),
        )
        .unwrap();
        let sprite = batch.oam.low[0];
        assert_eq!(
            [sprite.x, sprite.y, sprite.tile, sprite.attributes],
            [19, 23, 0xad, 0x24]
        );
        assert_eq!(batch.sprites.0, 1);
    }
}
