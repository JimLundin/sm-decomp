//! The Ceres room's physical display writes.

use super::{Content, Fault};
use crate::commit::VRAM_CHANNEL;
use crate::exact::art::planar_4bpp;
use crate::exact::countdown;
use crate::exact::direct_dma::Transfer;
use crate::exact::tilemap::{TileQueue, View};
use crate::intent::{OverlayRows, SpriteDraw, TileWrite};
use crate::production::blackout::Platform;
use crate::production::{blackout, hud};
use crate::state::video::{
    BBusRegister, DmaChannel, DmaControl, NmiBatch, TilemapEntry, VRAM_WORDS, Video, VramSource,
    VramStep, VramTransfer, VramWord,
};
use boss_ceres_ridley::{AlternatingLook, OverlayTurn};
use data::assets::actors::{DoorEnemyTiles, EnemyTilePatch};
use data::assets::art::TileSheet;
use data::assets::landing_site::Graphics as StandardGraphics;
use data::assets::shaft::{OVERLAY_BYTES, Shaft};
use data::dma::BusAddress;
use data::pose::tiles::Graphics;
use interface_hud::{CountdownPicture, HudState};
use types::{Pair, Parts, Position};

const SHADOW_WORDS: usize = 1024;
const SHAFT_BG2_STAGING: core::ops::Range<usize> = 0x0800..0x0c00;

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

    fn draw(batch: &mut NmiBatch, content: &Shaft, at: crate::intent::ScreenPoint) {
        let mut draw = SpriteDraw::default();
        draw.place(&content.figure, at);
        for piece in &mut draw.pieces {
            piece.look.palette = OVERLAY_PALETTE;
        }
        batch.draw_sprites(&draw);
    }
}

/// Prepared shaft writes awaiting the same NMI as the room's sprite batch.
pub(super) struct ShaftSprites {
    pub(super) actors: super::shaft::actor_picture::ActorPictures,
    pub(super) effects: Vec<enemy_interactions::picture::FigureRequest>,
    pub(super) standing: Option<blackout::StandingFrame>,
    pub(super) dust: super::shaft::dust::Placement,
    pub(super) body: Option<SpriteDraw>,
    pub(super) held_body: Option<SpriteDraw>,
    pub(super) timer: CountdownPicture,
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
    pending_platform: Option<Platform>,
    sprite_pending: bool,
    /// The background's pixel displacement from the room camera across a passage.
    pub bg1_offset: crate::state::video::Scroll,
    shaft_bg2_offset: Option<crate::state::video::Scroll>,
    shaft_raster_finished: bool,
    shaft_bg2: Option<Box<[u16]>>,
    enemy_uploads: Option<EnemyUploads>,
    effects_initialized: bool,
}

/// Physical video and the room publication still waiting for NMI.
#[derive(canonical::Canonical)]
struct SavedRoomPicture {
    video: Box<Video>,
    pending_platform: Option<Platform>,
    sprite_pending: bool,
}

impl Default for RoomVideo {
    fn default() -> Self {
        let mut video = Box::<Video>::default();
        // These channels keep their reset state until Ceres room setup.
        video.dma[4..=6].fill(DmaChannel::AT_POWER_ON);
        Self {
            video,
            bg1_offset: crate::state::video::Scroll::default(),
            shaft_bg2_offset: None,
            standard_objects: None,
            pending_platform: None,
            sprite_pending: false,
            shaft_raster_finished: false,
            shaft_bg2: None,
            enemy_uploads: None,
            effects_initialized: false,
        }
    }
}

impl RoomVideo {
    pub(super) fn start_shaft_bg2_scroll(&mut self, aligned_view: Position) {
        // The passage clears BG2 scroll before placing the incoming camera.
        self.shaft_bg2_offset = Some(crate::state::video::Scroll {
            x: aligned_view.x.pixel().0.wrapping_neg(),
            y: aligned_view.y.pixel().0.wrapping_neg(),
        });
    }

    pub(super) fn prepare_shaft_bg2_scroll(&mut self, view: Position, shake: world_quake::Offset) {
        if let Some(offset) = self.shaft_bg2_offset {
            // The shaft's back layer follows the camera on both axes.
            self.video.batch.registers.scroll[1] = crate::state::video::Scroll {
                x: view
                    .x
                    .pixel()
                    .0
                    .wrapping_add(offset.x)
                    .wrapping_add_signed(shake.x),
                y: view
                    .y
                    .pixel()
                    .0
                    .wrapping_add(offset.y)
                    .wrapping_add_signed(shake.y),
            };
        }
    }

    pub(super) fn reset_ridley_main_screen(&mut self) {
        self.video.dma[4].clear_room_effect();
    }

    pub(super) fn finish_ridley_main_screen(&mut self, raster: data::dma::DirectHdmaProgram) {
        let channel = &mut self.video.dma[4];
        channel.control = DmaControl::from_byte(raster.control);
        channel.destination = BBusRegister::from_byte(raster.destination);
        channel.source = raster.source.offset;
        channel.source_bank = raster.source.bank;
        channel.table = raster.terminal_cursor;
        channel.line_counter = 0;
    }

    /// The platform transfer currently waiting for publication.
    #[must_use]
    pub const fn pending_platform(&self) -> Option<Platform> {
        self.pending_platform
    }

    pub(super) fn prepare_shaft_picture(
        &mut self,
        turn: OverlayTurn,
        sprites: ShaftSprites,
        content: &Content,
    ) -> Result<(), Fault> {
        let art = content.shaft_overlay_art.as_ref().ok_or(Fault::Escape)?;
        let vent_art = content.shaft_vent_art.as_ref().ok_or(Fault::Escape)?;
        let effects = content.shaft_burst_art.as_ref().ok_or(Fault::Escape)?;
        if self.video.memory.vram.len() != VRAM_WORDS {
            return Err(Fault::Escape);
        }
        let mut batch = self.video.batch.clone();
        batch.restart_sprites();
        batch.clear_high_oam();
        if let CountdownPicture::Shown { place, time } = sprites.timer {
            let figures = content.clock_figures.as_ref().ok_or(Fault::Hud)?;
            let timer = countdown::sprites(figures, place, time).ok_or(Fault::Hud)?;
            batch.draw_sprites(&timer);
        }
        for figure in sprites.effects {
            effects.draw(&mut batch, figure).ok_or(Fault::Escape)?;
        }
        let point = |origin: enemy_interactions::picture::ScreenPoint| Pair {
            x: i32::from(origin.x),
            y: i32::from(origin.y),
        };
        if let Some(overlay) = sprites.actors.overlay {
            ShaftOverlay::draw(&mut batch, art, point(overlay.origin));
        }
        super::shaft::dust::append(
            &mut batch,
            sprites.dust,
            content.wall_jump_dust_art.as_ref(),
        )?;
        if let Some(standing) = sprites.standing {
            let body = content
                .standing
                .as_ref()
                .ok_or(Fault::StandingPicture(blackout::Fault::StandingArt))?;
            blackout::body::prepare(&mut batch, standing, body).map_err(Fault::StandingPicture)?;
        } else if let Some(body) = sprites.body {
            batch.draw_sprites(&body);
        }
        for (facing, figure, request) in sprites.actors.vents {
            super::shaft::vent::append(&mut batch, vent_art, facing, figure, point(request.origin));
        }
        if let Some(held) = sprites.held_body {
            batch.draw_sprites(&held);
        }
        self.pending_platform = turn.lights.and_then(|lights| lights.platform).map(|look| {
            let [first, second] = art.platform;
            Platform {
                start: PLATFORM_START,
                tiles: match look {
                    AlternatingLook::First => first,
                    AlternatingLook::Second => second,
                },
            }
        });
        self.video.batch = batch;
        self.sprite_pending = true;
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

    /// Queue the new file's beam tile strip for the requested NMI.
    pub(super) fn queue_beam_tiles(&mut self, from: BusAddress, words: &[u16]) {
        self.video.batch.vram.push(VramTransfer {
            to: VramWord(0x6300),
            step: VramStep::Row,
            source: VramSource::Words {
                from,
                words: words.to_vec(),
            },
        });
    }

    /// Queue the overlay requested by the initial Ceres door actor.
    pub(super) fn queue_overlay_tiles(&mut self, bytes: &[u8; OVERLAY_BYTES]) {
        self.video.batch.vram.push(VramTransfer {
            to: VramWord(0x7000),
            step: VramStep::Row,
            source: VramSource::Words {
                from: BusAddress {
                    bank: 0xb0,
                    offset: 0xc400,
                },
                words: bytes
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .copied()
                    .map(u16::from_le_bytes)
                    .collect(),
            },
        });
    }

    /// Apply the doorway's ordered object writes to carried physical VRAM.
    pub(super) fn install_door_enemy_tiles(&mut self, tiles: &[DoorEnemyTiles]) -> Option<()> {
        let vram = &mut self.video.memory.vram;
        if tiles.is_empty()
            || !tiles.iter().all(|tile| {
                tile.to
                    .checked_add(tile.words.len())
                    .is_some_and(|end| end <= vram.len())
            })
        {
            return None;
        }
        for tile in tiles {
            let end = tile.to.checked_add(tile.words.len())?;
            vram.get_mut(tile.to..end)?.copy_from_slice(&tile.words);
        }
        Some(())
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
        self.video.committed.set_gameplay_tile_layout();
        self.video.batch.registers.set_gameplay_tile_layout();
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
        let shadow = source
            .get(SHAFT_BG2_STAGING)?
            .iter()
            .copied()
            .map(u16::from_le_bytes)
            .collect();
        for (word, &pair) in vram.iter_mut().zip(source) {
            *word = u16::from_le_bytes(pair);
        }
        self.shaft_bg2 = Some(shadow);
        Some(())
    }

    /// The complete BG2 shadow left in work RAM by the shaft tileset fetch.
    #[must_use]
    pub fn shaft_bg2_shadow(&self) -> Option<&[u16]> {
        self.shaft_bg2.as_deref()
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

    /// Prepare the room HUD rows for the next NMI.
    pub(super) fn prepare_play_hud(&mut self, hud: &HudState) -> Result<(), Fault> {
        let bytes = hud::tilemap(hud).map_err(|_| Fault::Hud)?;
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
        Ok(())
    }

    pub(super) fn begin_ordinary_sprites(&mut self) {
        self.video.batch.restart_sprites();
        self.video.batch.clear_high_oam();
        self.sprite_pending = true;
    }

    pub(super) fn append_ordinary_body(
        &mut self,
        body: &crate::intent::SpriteDraw,
    ) -> core::ops::Range<usize> {
        let start = usize::from(self.video.batch.sprites.0);
        self.video.batch.draw_sprites(body);
        start..usize::from(self.video.batch.sprites.0)
    }

    pub(super) fn publish_sprites(&mut self) {
        if std::mem::take(&mut self.sprite_pending) {
            self.video.batch.finalise_oam();
            self.video.upload_oam();
        }
    }

    /// Encode physical video and its pending room publication together.
    #[must_use]
    pub fn picture_checkpoint(&self) -> Vec<u8> {
        canonical::to_bytes(&SavedRoomPicture {
            video: self.video.clone(),
            pending_platform: self.pending_platform,
            sprite_pending: self.sprite_pending,
        })
    }

    /// Restore the pending picture and physical video into the carried room loaders.
    ///
    /// # Errors
    /// Invalid canonical bytes leave the receiver intact.
    pub fn restore_picture(&mut self, bytes: &[u8]) -> Result<(), canonical::DecodeError> {
        let saved: SavedRoomPicture = canonical::from_bytes(bytes)?;
        self.video = saved.video;
        self.pending_platform = saved.pending_platform;
        self.sprite_pending = saved.sprite_pending;
        Ok(())
    }

    /// Commit the room's pending sprites and return the regular transfers in NMI order.
    pub(super) fn publish(
        &mut self,
        standing: Option<&crate::samus_appearance_bridge::elevator::Art>,
    ) -> Result<Vec<VramTransfer>, Fault> {
        if self.shaft_bg2_offset.is_some() {
            let layer = self.video.batch.registers.scroll[1];
            self.video.committed.scroll[1] = layer.latched();
        }
        if let Some(art) = standing.filter(|art| art.accepts(self.video.batch.standing_strips)) {
            self.video.publish_standing_strips(&art.graphics);
        }
        if let Some(platform) = self.pending_platform.take() {
            platform
                .publish(&mut self.video)
                .map_err(|_| Fault::Escape)?;
        }
        let serviced = self.video.process_vram_writes(&Graphics::default());
        crate::exact::tilemap::commit_exposures(&mut self.video);
        Ok(serviced)
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

impl super::State {
    /// The two work RAM pages retained after fetching the escape shaft.
    #[must_use]
    pub fn departure_shadows(&self) -> Option<super::landing_site::ShadowTilemaps> {
        use super::room_tiles::ShaftBackground;
        if self.room_tiles != ShaftBackground::BackgroundTransferred {
            return None;
        }
        Some(super::landing_site::ShadowTilemaps {
            bg1: vec![self.room_tiles.shadow_word()?; SHADOW_WORDS],
            bg2: self.room_video.shaft_bg2_shadow()?.to_vec(),
        })
    }
}
