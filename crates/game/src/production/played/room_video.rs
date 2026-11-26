//! The Ceres room's physical display writes.

use super::Fault;
use crate::commit::VRAM_CHANNEL;
use crate::exact::art::planar_4bpp;
use crate::exact::direct_dma::Transfer;
use crate::exact::tilemap::{TileQueue, View};
use crate::intent::{OverlayRows, SpriteDraw, TileWrite, on_screen};
use crate::production::blackout::Platform;
use crate::production::hud;
use crate::state::video::{
    DmaChannel, NmiBatch, TilemapEntry, VRAM_WORDS, Video, VramSource, VramStep, VramTransfer,
    VramWord,
};
use boss_ceres_ridley::{AlternatingLook, OverlayTurn};
use data::assets::actors::EnemyTilePatch;
use data::assets::art::TileSheet;
use data::assets::landing_site::Graphics as StandardGraphics;
use data::assets::shaft::{OVERLAY_BYTES, Shaft};
use data::dma::BusAddress;
use data::pose::tiles::Graphics;
use types::{Pair, Px};

const SHAFT_SHEET: core::ops::Range<usize> = 0..0x4000;
const STANDARD_BG3: core::ops::Range<usize> = 0x4000..0x4800;

const OPENING_HUD: core::ops::Range<usize> = 0x5800..0x5880;

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

const ENEMY_SHEET_SOURCE: u16 = 0x7000;
const ENEMY_SHEET_DESTINATION: usize = 0x6c00;
const ENEMY_SHEET_BYTES: usize = 0x2800;
const ENEMY_UPLOAD_BYTES: usize = 0x0800;

/// The enemy tile staging buffer after cold boot has cleared it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EnemyStaging {
    /// The startup RAM clear finished.
    Cleared,
}

impl EnemyStaging {
    /// The source range reserved for the starting room's enemy tiles.
    pub const WRAM: core::ops::Range<usize> = 0x7000..0x9800;

    /// Admit the staging buffer at the origin of a scoped startup.
    #[must_use]
    pub fn admit(wram: &[u8]) -> Option<Self> {
        wram.get(Self::WRAM)?
            .iter()
            .all(|&byte| byte == 0)
            .then_some(Self::Cleared)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum EnemyUploads {
    Waiting(Vec<EnemyTilePatch>),
    Staged { source: Box<[u8]>, queued: usize },
    Published,
}

impl EnemyUploads {
    fn stage(tiles: &[EnemyTilePatch]) -> Option<Self> {
        let mut source = vec![0; ENEMY_SHEET_BYTES];
        for patch in tiles {
            let end = patch.offset.checked_add(patch.bytes.len())?;
            source
                .get_mut(patch.offset..end)?
                .copy_from_slice(&patch.bytes);
        }
        Some(Self::Staged {
            source: source.into_boxed_slice(),
            queued: 0,
        })
    }

    const fn queued_bytes(&self) -> usize {
        match self {
            Self::Waiting(_) => 0,
            Self::Staged { queued, .. } => *queued,
            Self::Published => ENEMY_SHEET_BYTES,
        }
    }
}

/// The parts of physical room graphics produced by the room's loaders.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomVideo {
    /// Physical memories and the pending NMI batch carried by the room.
    pub video: Box<Video>,
    standard_objects: Option<Transfer>,
    shaft_picture: Option<ShaftPicture>,
    shaft_raster_finished: bool,
    enemy_uploads: Option<EnemyUploads>,
    effects_initialized: bool,
}

impl Default for RoomVideo {
    fn default() -> Self {
        let mut video = Box::<Video>::default();
        // These channels keep their reset state until Ceres room setup.
        video.dma[5] = DmaChannel::AT_POWER_ON;
        video.dma[6] = DmaChannel::AT_POWER_ON;
        Self {
            video,
            standard_objects: None,
            shaft_picture: None,
            shaft_raster_finished: false,
            enemy_uploads: None,
            effects_initialized: false,
        }
    }
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
                self.video.upload_oam();
            }
            self.shaft_picture = None;
        }
        Ok(())
    }

    pub(super) fn finish_shaft_hdma(
        &mut self,
        programs: &[data::assets::countdown::IndirectHdmaProgram; 3],
    ) {
        self.video.finish_shaft_hdma(programs);
        self.shaft_raster_finished = true;
    }

    /// The physical channels after the return shaft's raster setup.
    #[must_use]
    pub const fn shaft_raster_finished(&self) -> bool {
        self.shaft_raster_finished
    }

    /// Initialize the effects before the room's second music service.
    pub(super) fn initialize_effects(&mut self) {
        self.video.reset_room_hdma();
        self.effects_initialized = true;
    }

    /// The channels left idle by the Ceres effects since initialization.
    #[must_use]
    pub const fn idle_hdma(&self) -> Option<[DmaChannel; 2]> {
        if self.effects_initialized {
            Some([self.video.dma[5], self.video.dma[6]])
        } else {
            None
        }
    }

    /// Carry the initial room's asset patches to its staging event after the cold boot.
    pub(super) fn offer_enemy_tiles(
        &mut self,
        staging: Option<EnemyStaging>,
        tiles: Vec<EnemyTilePatch>,
    ) {
        if staging == Some(EnemyStaging::Cleared) && !tiles.is_empty() {
            self.enemy_uploads = Some(EnemyUploads::Waiting(tiles));
        }
    }

    /// Fill the startup staging buffer using the selected room's tile patches.
    pub(super) fn prepare_enemy_tiles(&mut self) -> Option<()> {
        let Some(EnemyUploads::Waiting(tiles)) = self.enemy_uploads.as_ref() else {
            return None;
        };
        self.enemy_uploads = Some(EnemyUploads::stage(tiles)?);
        Some(())
    }

    /// The startup enemy tiles prepared in WRAM.
    #[must_use]
    pub fn staged_enemy_tiles(&self) -> Option<&[u8]> {
        match self.enemy_uploads.as_ref()? {
            EnemyUploads::Staged { source, .. } => Some(source),
            EnemyUploads::Waiting(_) | EnemyUploads::Published => None,
        }
    }

    /// Queue the next enemy tile chunk for the preparation's requested NMI.
    pub(super) fn queue_enemy_tiles(&mut self) -> Option<()> {
        let Some(EnemyUploads::Staged { source, queued }) = self.enemy_uploads.as_mut() else {
            return None;
        };
        let start = *queued;
        let end = start.checked_add(ENEMY_UPLOAD_BYTES)?;
        let to = u16::try_from(ENEMY_SHEET_DESTINATION.checked_add(start / 2)?).ok()?;
        let offset = ENEMY_SHEET_SOURCE.checked_add(u16::try_from(start).ok()?)?;
        let source = source.get(start..end)?;
        self.video.batch.vram.push(VramTransfer {
            to: VramWord(to),
            step: VramStep::Row,
            source: VramSource::Words {
                from: BusAddress { bank: 0x7e, offset },
                words: source
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .copied()
                    .map(u16::from_le_bytes)
                    .collect(),
            },
        });
        *queued = end;
        if end == ENEMY_SHEET_BYTES {
            self.enemy_uploads = Some(EnemyUploads::Published);
        }
        Some(())
    }

    /// The words committed by the starting room's enemy tile uploads.
    #[must_use]
    pub fn uploaded_enemy_tiles(&self) -> Option<&[u16]> {
        let end =
            ENEMY_SHEET_DESTINATION.checked_add(self.enemy_uploads.as_ref()?.queued_bytes() / 2)?;
        self.video.memory.vram.get(ENEMY_SHEET_DESTINATION..end)
    }

    /// Publish standard text tiles and advance the interrupted object upload.
    ///
    /// Returns `None` for incomplete graphics or inconsistent transfer progress.
    #[must_use]
    pub fn standard_graphics(&mut self, graphics: &StandardGraphics, accepted: u16) -> Option<()> {
        if self.standard_objects.is_some() {
            return None;
        }
        let background = graphics.background.get(..STANDARD_BG3.len())?;
        let mut transfer = Transfer::new(
            graphics.sprite_source,
            VramWord(0x6000),
            graphics.sprite_length,
        );
        transfer
            .advance(
                &mut self.video.memory.vram,
                &graphics.sprites,
                accepted,
                true,
            )
            .ok()?;
        self.video
            .memory
            .vram
            .get_mut(STANDARD_BG3)?
            .copy_from_slice(background);
        transfer.project_channel(self.video.dma.get_mut(VRAM_CHANNEL)?);
        self.standard_objects = Some(transfer);
        Some(())
    }

    /// Publish the complete Mode 7 sheet at the return passage's settled tile fetch.
    pub(super) fn shaft_sheet(&mut self, sheet: &TileSheet) -> Option<()> {
        let encoded = planar_4bpp(sheet);
        let (source, trailing) = encoded.as_chunks::<2>();
        let vram = self.video.memory.vram.get_mut(SHAFT_SHEET)?;
        if !trailing.is_empty() || source.len() != vram.len() {
            return None;
        }
        for (word, &pair) in vram.iter_mut().zip(source) {
            *word = u16::from_le_bytes(pair);
        }
        Some(())
    }

    /// Install the opening HUD, including the fixed upper row.
    pub(super) fn opening_hud(&mut self, opening: &interface_hud::NewFileHud) -> Option<()> {
        let lower = hud::tilemap(&opening.state).ok()?;
        let words = self.video.memory.vram.get_mut(OPENING_HUD)?;
        let upper = opening
            .upper_row
            .iter()
            .map(|&cell| u16::from_le_bytes(hud::cell(cell)));
        let lower = lower
            .as_chunks::<2>()
            .0
            .iter()
            .copied()
            .map(u16::from_le_bytes);
        for (slot, word) in words.iter_mut().zip(upper.chain(lower)) {
            *slot = word;
        }
        Some(())
    }

    /// Prepare the status bar's three changing rows for the next NMI.
    pub(super) fn prepare_hud(&mut self, state: &interface_hud::HudState) -> Option<()> {
        let bytes = hud::tilemap(state).ok()?;
        let cells = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&cell| TilemapEntry::from_word(u16::from_le_bytes(cell)))
            .collect();
        TileQueue::new(&mut self.video.batch).write(
            View::default(),
            &TileWrite::Overlay(OverlayRows { row: 1, cells }),
        );
        Some(())
    }

    /// Commit the room's pending VRAM writes in NMI order.
    pub(super) fn publish(&mut self) {
        self.video.process_vram_writes(&Graphics::default());
    }

    /// Advance through ROM and, when available, the mapped low WRAM at the source wrap.
    ///
    /// With no WRAM source, retain the transfer at the ROM boundary.
    /// Returns `None` for a missing or incoherent transfer or a short supplied WRAM source.
    #[must_use]
    pub fn finish_standard_objects(
        &mut self,
        graphics: &StandardGraphics,
        low_wram: Option<&[u8]>,
    ) -> Option<()> {
        let mut transfer = self.standard_objects?;
        let rom_bytes = u16::try_from(graphics.sprites.len()).ok()?;
        if let Some(low_wram) = low_wram {
            let wrapped = usize::from(transfer.length.checked_sub(rom_bytes)?);
            let mut source = graphics.sprites.clone();
            source.extend_from_slice(low_wram.get(..wrapped)?);
            transfer
                .advance(&mut self.video.memory.vram, &source, transfer.length, false)
                .ok()?;
            transfer.drain(&mut self.video.memory.vram).ok()?;
        } else {
            transfer
                .advance(
                    &mut self.video.memory.vram,
                    &graphics.sprites,
                    rom_bytes,
                    false,
                )
                .ok()?;
            transfer.retire_pending(&mut self.video.memory.vram).ok()?;
        }
        self.standard_objects = Some(transfer);
        Some(())
    }

    /// The unfinished or completed standard object upload.
    #[must_use]
    pub const fn standard_objects(&self) -> Option<&Transfer> {
        self.standard_objects.as_ref()
    }

    /// The published BG3 prefix as it now stands in physical VRAM.
    #[must_use]
    pub fn standard_bg3(&self) -> Option<&[u16]> {
        self.standard_objects?;
        self.video.memory.vram.get(STANDARD_BG3)
    }
}
