//! Encode the countdown's part of each prepared and published sprite buffer.

use canonical::Canonical;
use data::assets::countdown::Figures;
use interface_hud::{Countdown, CountdownPicture};

use crate::state::video::NmiBatch;

/// Timer sprites in OAM order. Each piece carries four low bytes and its two high bits.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct ClockPicture {
    /// Timer sprites prepared by room or passage work.
    pub prepared: Vec<[u8; 5]>,
    /// Timer sprites copied to physical OAM at publication.
    pub published: Vec<[u8; 5]>,
}

impl ClockPicture {
    pub(super) fn play(
        &self,
        figures: Option<&Figures>,
        picture: CountdownPicture,
        publishes: bool,
    ) -> Option<Self> {
        let mut drawn = self.clone();
        drawn.prepare(figures, picture)?;
        if publishes {
            drawn.publish();
        }
        Some(drawn)
    }

    pub(super) fn prepare(
        &mut self,
        figures: Option<&Figures>,
        picture: CountdownPicture,
    ) -> Option<()> {
        self.prepared = match (figures, picture) {
            (Some(figures), CountdownPicture::Shown { place, time }) => {
                let draw = crate::exact::countdown::sprites(figures, place, time)?;
                let mut batch = NmiBatch::default();
                batch.draw_sprites(&draw);
                batch
                    .oam
                    .low
                    .iter()
                    .take(draw.pieces.len())
                    .enumerate()
                    .map(|(i, entry)| {
                        let high = (batch.oam.high.get(i / 4).copied().unwrap_or_default()
                            >> (i % 4).wrapping_mul(2))
                            & 3;
                        [entry.x, entry.y, entry.tile, entry.attributes, high]
                    })
                    .collect()
            }
            _ => Vec::new(),
        };
        Some(())
    }

    pub(super) fn passage(
        &self,
        figures: Option<&Figures>,
        countdown: Countdown,
        draws: bool,
        timing: &super::Passing,
    ) -> Option<Self> {
        let mut picture = self.clone();
        if timing.work_done || draws {
            picture.prepare(figures, held_picture(countdown, draws))?;
        }
        if timing.publishes {
            picture.publish();
        }
        Some(picture)
    }

    pub(super) fn publish(&mut self) {
        self.published.clone_from(&self.prepared);
    }
}

pub(super) const fn held_picture(countdown: Countdown, draws: bool) -> CountdownPicture {
    match countdown {
        Countdown::Begun(clock) if draws => CountdownPicture::Shown {
            place: clock.place.whole_pixels(),
            time: clock.time_left,
        },
        Countdown::Dormant | Countdown::Begun(_) => CountdownPicture::Absent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_saved_ready_picture_publishes_without_drawing_it_again() {
        use data::spritemap::{Spritemap, SpritemapEntry};
        use interface_hud::TimeLeft;
        use types::Pair;

        let digit = |tile| Spritemap {
            entries: vec![SpritemapEntry {
                attributes: data::spritemap::SpriteAttributes {
                    tile,
                    ..Default::default()
                },
                ..Default::default()
            }],
        };
        let figures = Figures {
            frame: digit(11),
            digits: core::array::from_fn(|i| digit(u16::try_from(i).unwrap())),
        };
        let picture = |seconds| CountdownPicture::Shown {
            place: Pair { x: 128, y: 32 },
            time: TimeLeft::new(0, seconds, 0).unwrap(),
        };
        let mut direct = ClockPicture::default();
        direct.prepare(Some(&figures), picture(59)).unwrap();
        direct.publish();
        direct.prepare(Some(&figures), picture(58)).unwrap();
        assert_ne!(direct.prepared, direct.published);
        let saved = canonical::to_bytes(&direct);
        let mut restored: ClockPicture = canonical::from_bytes(&saved).unwrap();
        direct.publish();
        restored.publish();
        assert_eq!(restored, direct);
        assert_eq!(restored.prepared, restored.published);
        restored
            .prepare(Some(&figures), CountdownPicture::Absent)
            .unwrap();
        assert_eq!(restored.prepared, Vec::<[u8; 5]>::new());
        assert_ne!(restored.published, Vec::<[u8; 5]>::new());
        restored.publish();
        assert_eq!(restored.published, Vec::<[u8; 5]>::new());
    }
}
