//! The Ceres room's physical display writes.

use super::Fault;
use crate::intent::{SpriteDraw, on_screen};
use crate::production::blackout::Platform;
use crate::state::video::{NmiBatch, VRAM_WORDS, Video, VramWord};
use boss_ceres_ridley::{AlternatingLook, OverlayTurn};
use data::assets::shaft::{OVERLAY_BYTES, Shaft};
use types::{Pair, Px};

const STANDARD_BG3: core::ops::Range<usize> = 0x4000..0x4800;

const OVERLAY_START: usize = 0x7000;
const OVERLAY_END: usize = 0x7200;
const PLATFORM_START: VramWord = VramWord(0x060e);
const OVERLAY_PALETTE: u8 = 7;

/// The sprite graphics installed when the escape shaft places its overlay.
pub struct ShaftOverlay;

impl ShaftOverlay {
    /// Publish the complete sprite strip at actor placement, without touching the NMI batch.
    ///
    /// Returns `None` for an incomplete VRAM image, leaving it intact.
    #[must_use]
    pub fn publish(video: &mut Video, content: &Shaft) -> Option<()> {
        if video.memory.vram.len() != VRAM_WORDS {
            return None;
        }
        let words = video.memory.vram.get_mut(OVERLAY_START..OVERLAY_END)?;
        let bytes: &[u8; OVERLAY_BYTES] = &content.overlay;
        for (word, &source) in words.iter_mut().zip(bytes.as_chunks::<2>().0) {
            *word = u16::from_le_bytes(source);
        }
        Some(())
    }

    fn draw(batch: &mut NmiBatch, content: &Shaft, anchor: Pair<Px>, camera: Pair<Px>) {
        let mut draw = SpriteDraw::default();
        draw.place(
            &content.figure,
            Pair {
                x: on_screen(anchor.x.0, camera.x.0),
                y: on_screen(anchor.y.0, camera.y.0),
            },
        );
        for piece in &mut draw.pieces {
            piece.look.palette = OVERLAY_PALETTE;
        }
        batch.draw_sprites(&draw);
    }
}

/// Prepared shaft writes awaiting the same NMI as the room's sprite batch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ShaftPicture {
    platform: Option<Platform>,
    oam: bool,
}

/// The parts of physical room graphics produced by the room's loaders.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RoomVideo {
    /// Physical memories and the pending NMI batch carried by the room.
    pub video: Box<Video>,
    /// The first 2,048 words of standard background tiles from Ceres setup.
    pub standard_bg3: Option<Vec<u16>>,
    shaft_picture: Option<ShaftPicture>,
    standard_bg3_published: bool,
}

impl RoomVideo {
    /// The platform transfer currently waiting for publication.
    #[must_use]
    pub fn pending_platform(&self) -> Option<Platform> {
        self.shaft_picture.and_then(|picture| picture.platform)
    }

    pub(super) fn shaft_picture(
        &mut self,
        turn: Option<OverlayTurn>,
        camera: Pair<Px>,
        publishes: bool,
        art: Option<&Shaft>,
    ) -> Result<(), Fault> {
        if let Some(turn) = turn {
            let art = art.ok_or(Fault::Escape)?;
            if self.video.memory.vram.len() != VRAM_WORDS {
                return Err(Fault::Escape);
            }
            let platform = turn.lights.and_then(|lights| lights.platform).map(|look| {
                let [first, second] = art.platform;
                Platform {
                    start: PLATFORM_START,
                    tiles: match look {
                        AlternatingLook::First => first,
                        AlternatingLook::Second => second,
                    },
                }
            });
            let oam = turn.figure.is_some() || turn.start_rotation.is_some();
            if oam && self.video.batch.sprites.0 == 0 {
                self.video.batch.clear_high_oam();
            }
            if let Some(anchor) = turn.figure {
                ShaftOverlay::draw(&mut self.video.batch, art, anchor, camera);
            }
            self.shaft_picture = Some(ShaftPicture { platform, oam });
        }
        if publishes && let Some(picture) = self.shaft_picture {
            if let Some(platform) = picture.platform {
                platform
                    .publish(&mut self.video)
                    .map_err(|_| Fault::Escape)?;
            }
            if picture.oam {
                self.video.batch.finalise_oam();
                self.video.upload_sprite_oam();
            }
            self.shaft_picture = None;
        }
        Ok(())
    }

    /// Publish the standard text tile prefix when the room's direct upload has completed.
    ///
    /// Returns `None` when the required source prefix is unavailable.
    #[must_use]
    pub fn standard_background(&mut self, background: &[u16]) -> Option<()> {
        let words = self.video.memory.vram.get_mut(STANDARD_BG3)?;
        words.copy_from_slice(background.get(..words.len())?);
        self.standard_bg3_published = true;
        Some(())
    }

    /// The published BG3 prefix as it now stands in physical VRAM.
    #[must_use]
    pub fn standard_bg3(&self) -> Option<&[u16]> {
        if self.standard_bg3_published {
            self.video.memory.vram.get(STANDARD_BG3)
        } else {
            None
        }
    }
}
