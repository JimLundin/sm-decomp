//! Sprite drawing for the flight to Ceres.

use crate::intent::SpriteDraw;
use crate::state::video::Video;
use data::assets::flight::SpritePictures;
use screens_flight::FrameReport;
use types::Pair;
use types::flight::{Drifter, FlightSprite};

/// The vortex alternates pictures while placed, independently of the flight's brightness.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Picture {
    alternate_vortex: bool,
}

const fn draw_order(kind: FlightSprite) -> u8 {
    match kind {
        FlightSprite::RushingStars | FlightSprite::Drifting(Drifter::LargeAsteroids) => 0,
        FlightSprite::Drifting(Drifter::Station) => 1,
        FlightSprite::Drifting(Drifter::SmallAsteroids) => 2,
        FlightSprite::Drifting(Drifter::Vortex) => 3,
        FlightSprite::Drifting(Drifter::DistantStars) => 4,
    }
}

impl Picture {
    /// Prepare the sprites the flight placed this frame and publish them through NMI.
    /// An empty final picture clears the high table and hides the unused low entries by Y.
    pub fn publish(&mut self, video: &mut Video, report: &FrameReport, pictures: &SpritePictures) {
        let mut sprites = report.sprites.clone();
        sprites.sort_by_key(|sprite| draw_order(sprite.kind));
        let batch = &mut video.batch;
        batch.clear_high_oam();
        batch.restart_sprites();
        for sprite in sprites {
            let use_alternate =
                sprite.kind == FlightSprite::Drifting(Drifter::Vortex) && self.alternate_vortex;
            let art = match sprite.kind {
                FlightSprite::RushingStars => &pictures.rushing,
                FlightSprite::Drifting(Drifter::LargeAsteroids) => &pictures.large_asteroids,
                FlightSprite::Drifting(Drifter::Station) => &pictures.station,
                FlightSprite::Drifting(Drifter::SmallAsteroids) => &pictures.small_asteroids,
                FlightSprite::Drifting(Drifter::Vortex) => {
                    self.alternate_vortex = !self.alternate_vortex;
                    &pictures.vortex
                }
                FlightSprite::Drifting(Drifter::DistantStars) => &pictures.distant_stars,
            };
            if !(-128..383).contains(&sprite.place.y) {
                continue;
            }
            let map = if use_alternate {
                &pictures.alternate_vortex
            } else {
                &art.map
            };
            let mut draw = SpriteDraw::default();
            draw.place(
                map,
                Pair {
                    x: sprite.place.x.into(),
                    y: sprite.place.y.into(),
                },
            );
            for piece in &mut draw.pieces {
                piece.look.palette |= art.palette;
                if !(-32..224).contains(&piece.at.y) {
                    piece.at = Pair { x: 384, y: 224 };
                }
            }
            batch.draw_capped_sprites(&draw);
        }
        batch.finalise_oam();
        video.upload_oam();
    }
}
