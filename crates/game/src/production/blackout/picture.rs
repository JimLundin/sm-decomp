//! Assemble the HUD owner's timer output and keep its queued room picture until NMI.
use super::Fault;
use crate::intent::SpriteDraw;
use crate::state::video::{BgMode, Brightness, Layer, NmiBatch, Registers, Video, VramWord};
use canonical::Canonical;
use data::assets::countdown::Figures;
use interface_hud::CountdownPicture;
use types::Pair;
use types::ceres_blackout::{BlackoutLight, PreparedPicture, TimerSprite};

/// The rotating overlay's queued low-byte writes. E2 preserves the tile graphics in the high lane.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Platform {
    /// First tilemap word.
    pub start: VramWord,
    /// Four tilemap entries.
    pub tiles: [u8; 4],
}

impl Platform {
    pub(super) fn publish(self, video: &mut Video) -> Result<(), Fault> {
        let words = video
            .memory
            .vram
            .get_mut(usize::from(self.start.0)..)
            .and_then(<[_]>::first_chunk_mut::<4>)
            .ok_or(Fault::Picture)?;
        for (word, tile) in words.iter_mut().zip(self.tiles) {
            *word = (*word & 0xff00) | u16::from(tile);
        }
        Ok(())
    }
}

pub(super) fn draw(
    batch: &mut NmiBatch,
    figures: &Figures,
    picture: CountdownPicture,
) -> Result<PreparedPicture, Fault> {
    let CountdownPicture::Shown { place, time } = picture else {
        return Err(Fault::Timer);
    };
    let origin = Pair {
        x: i32::from(place.x),
        y: i32::from(place.y),
    };
    let mut draw = SpriteDraw::default();
    draw.place(&figures.frame, origin);
    for (pair, offset) in time.shown().into_iter().zip([-28_i32, -4, 20]) {
        for (digit, x) in [pair / 10, pair % 10]
            .into_iter()
            .zip([offset, offset.saturating_add(8)])
        {
            let map = usize::try_from(digit)
                .ok()
                .and_then(|n| figures.digits.get(n))
                .ok_or(Fault::Picture)?;
            draw.place(
                map,
                Pair {
                    x: origin.x.saturating_add(x),
                    y: origin.y,
                },
            );
        }
    }
    if draw.pieces.len() > crate::state::video::OAM_SPRITES {
        return Err(Fault::Picture);
    }
    for (i, piece) in draw.pieces.iter_mut().enumerate() {
        piece.look.palette |= 5;
        if let Some(high) = batch.oam.high.get_mut(i / 4) {
            *high &= !(3 << ((i % 4).wrapping_mul(2)));
        }
    }
    batch.restart_sprites();
    batch.draw_sprites(&draw);
    Ok(PreparedPicture {
        timer_sprites: draw
            .pieces
            .iter()
            .map(|piece| TimerSprite(piece.look.word()))
            .collect(),
    })
}

pub(super) fn light(registers: &mut Registers, light: BlackoutLight) {
    registers.forced_blank = light == BlackoutLight::ForcedBlank;
    registers.brightness = Brightness(match light {
        BlackoutLight::ForcedBlank => 0,
        BlackoutLight::Lit(level) => level.get(),
    });
}

pub(super) fn sprites_only(registers: &mut Registers) {
    registers.mode = BgMode(9);
    registers.main_screen = crate::state::video::Layers::default();
    registers.main_screen.insert(Layer::Objects);
    registers.sub_screen = crate::state::video::Layers::default();
    registers.main_window_mask = crate::state::video::Layers::default();
    registers.sub_window_mask = crate::state::video::Layers::default();
    registers.colour_math.layers = crate::state::video::ColourMathLayers::default();
    registers.colour_math.select = crate::state::video::ColourMathSelect::default();
}
