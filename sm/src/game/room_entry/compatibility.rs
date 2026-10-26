//! Exact retained room-entry effects on the one Session runtime boundary.
use super::{api::EntryWait, content::*, effects::*};
use crate::game::actors;
use crate::{
    game::{self, room_mechanisms, session_runtime::Runtime},
    ram_map::words as W,
    snes::registers::*,
};
fn external_address(field: ExternalField) -> usize {
    match field {
        ExternalField::GameMode => W::game_state,
        ExternalField::DebugMinimap => W::debug_disable_minimap,
        ExternalField::TimerStatus => W::timer_status,
        ExternalField::PreviousPlayerX => W::samus_prev_x_pos,
        ExternalField::PreviousPlayerY => W::samus_prev_y_pos,
        ExternalField::MusicDataIndex => W::music_data_index,
        ExternalField::MusicTrackIndex => W::music_track_index,
        ExternalField::HdmaEnabled => W::hdma_objects_enable_flag,
        ExternalField::NextIrqHandler => W::irqhandler_next_handler,
        ExternalField::FxTilemap => W::fx_tilemap_ptr,
        ExternalField::BackgroundCopyByte => 0x5e,
        ExternalField::DisplayAndObjectSelect => 0x51,
        ExternalField::LoadingLowByte => 0x5f7,
    }
}
fn camera_address(field: CameraField) -> usize {
    match field {
        CameraField::Layer1X => W::layer1_x_pos,
        CameraField::Layer1Y => W::layer1_y_pos,
        CameraField::Layer2X => W::layer2_x_pos,
        CameraField::Layer2Y => W::layer2_y_pos,
        CameraField::Bg1XOffset => W::bg1_x_offset,
        CameraField::Bg1YOffset => W::bg1_y_offset,
        CameraField::Bg2XScroll => W::bg2_x_scroll,
        CameraField::Bg2YScroll => W::bg2_y_scroll,
        CameraField::Bg1Hofs => W::reg_BG1HOFS,
        CameraField::Bg1Vofs => W::reg_BG1VOFS,
        CameraField::Bg2Hofs => W::reg_BG2HOFS,
        CameraField::Bg2Vofs => W::reg_BG2VOFS,
    }
}
impl EntryHost for Runtime<'_> {}
impl EntryEffects for Runtime<'_> {
    fn read_external(&mut self, field: ExternalField) -> u16 {
        let at = external_address(field);
        match field {
            ExternalField::BackgroundCopyByte | ExternalField::LoadingLowByte => {
                u16::from(self.byte(at))
            }
            _ => self.word(at),
        }
    }
    fn write_external(&mut self, field: ExternalField, value: u16) {
        let at = external_address(field);
        match field {
            ExternalField::BackgroundCopyByte | ExternalField::LoadingLowByte => {
                self.set_byte(at, value as u8)
            }
            _ => self.write_slice_le(at, value),
        }
    }
    fn read_camera(&mut self, field: CameraField) -> u16 {
        self.word(camera_address(field))
    }
    fn write_camera(&mut self, field: CameraField, value: u16) {
        self.write_slice_le(camera_address(field), value);
    }
    fn call(&mut self, effect: LeafEffect) {
        self.foreign(|game| unsafe {
            match effect {
                LeafEffect::QueueDoorSound => {
                    game::music::QueueSfx2_Max15(0x71, game);
                }
                LeafEffect::ResetSoundQueues => {
                    game::music::ResetSoundQueues(game);
                }
                LeafEffect::ClearTransitionSounds => {
                    game::session_runtime::player_command(0x1d, game);
                }
                LeafEffect::SelectEnemies => {
                    actors::scheduling::select(&mut Runtime::new(game));
                }
                LeafEffect::RunEnemies => {
                    actors::scheduling::advance_selected(&mut Runtime::new(game));
                }
                LeafEffect::DrawScene => {
                    actors::drawing::draw_scene(&mut Runtime::new(game));
                }
                LeafEffect::EnsureSamusDrawn => {
                    game::sm_82::EnsureSamusDrawnEachFrame(game);
                }
                LeafEffect::QueueEnemyTiles => {
                    actors::staging::queue_actor_tiles(&mut Runtime::new(game));
                }
                LeafEffect::DrawTimer => {
                    game::escape_timer::draw(game);
                }
                LeafEffect::SpinJumpSound => {
                    game::session_runtime::player_command(0x1c, game);
                }
                LeafEffect::SetLiquidPhysics => {
                    game::player_movement::speed::set_liquid_physics(&mut Runtime::new(game));
                }
                LeafEffect::StopHdmaEffects => {
                    game::visual_runtime::clear_hdma_channels(game);
                }
                LeafEffect::InitializeRoomSpecialEffects => {
                    game::visual_runtime::initialize_room_effects(game);
                }
                LeafEffect::LoadFxHeader => {
                    game::room_state::load_fx_header(game);
                }
                LeafEffect::InitializeEnemies => {
                    actors::population::install_population(&mut Runtime::new(game));
                }
                LeafEffect::LoadEnemyGraphics => {
                    game::room_state::load_enemy_graphics(game);
                }
                LeafEffect::LoadRoomMusic => {
                    game::music::LoadRoomMusic(game);
                }
                LeafEffect::UpdateMusicTrack => {
                    game::music::UpdateMusicTrackIndex(game);
                }
                LeafEffect::LoadChangedMusic => {
                    game::music::LoadNewMusicTrackIfChanged(game);
                }
                LeafEffect::UpdateBeamTilesAndPalette => {
                    game::visual_runtime::update_beam_tiles_and_palette(game);
                }
                LeafEffect::LoadSpritePalettes => {
                    game::room_state::load_sprite_palettes(game);
                }
                LeafEffect::LoadRoomAssets => {
                    actors::staging::prepare_room_assets(&mut Runtime::new(game));
                }
                LeafEffect::LoadSuitTargetPalette => {
                    game::samus_palette::load_suit(
                        &mut Runtime::new(game),
                        game::visual_state::PaletteBuffer::Target,
                    );
                }
                LeafEffect::SpawnBg3ScrollHdma => {
                    game::visual_runtime::spawn_bg3_scroll(game);
                }
                LeafEffect::SaveExploredMap => {
                    game::save_slots::save_area_map(game);
                }
                LeafEffect::LoadExploredMap => {
                    game::save_slots::load_area_map(game);
                }
                LeafEffect::MarkElevatorsUsed => {
                    game::save_slots::mark_elevator(game);
                }
                LeafEffect::EnableNmi => {
                    crate::game::display_refresh::enable(true, game);
                }
                LeafEffect::DisableNmi => {
                    crate::game::display_refresh::enable(false, game);
                }
                LeafEffect::EnableIrq => {
                    game::scanline_irq::enable(false, game);
                }
                LeafEffect::DisableIrq => {
                    game::scanline_irq::disable(game);
                }
                LeafEffect::WaitVblankClearHdma => {
                    game::visual_runtime::clear_hdma(game);
                }
                LeafEffect::WaitVblankEnableIrq => {
                    game::sm_82::WaitUntilEndOfVblankAndEnableIrq(game);
                }
            }
        });
    }
    fn camera_call(&mut self, effect: CameraEffect) -> u8 {
        self.foreign(|game| unsafe {
            match effect {
                CameraEffect::CalculateLayer2X => {
                    game::room_scrolls::layer2_position(Axis::X, game)
                }
                CameraEffect::CalculateLayer2Y => {
                    game::room_scrolls::layer2_position(Axis::Y, game)
                }
                CameraEffect::CalculateLayer2AndScrolls => {
                    game::room_scrolls::update_scrolls(game);
                    0
                }
                CameraEffect::CalculateBgScrolls => {
                    game::room_scrolls::calculate_scrolls(game);
                    0
                }
                CameraEffect::DisplayVisibleRoom => {
                    game::room_scrolls::fill_viewport(game, |game| {
                        game::display_refresh::flush_tilemaps(game)
                    });
                    0
                }
                CameraEffect::FixDoorMovingUp => {
                    game::room_scrolls::door::fix_up(game);
                    0
                }
                CameraEffect::SetupDoorScroll => {
                    game::room_scrolls::door::begin(game);
                    0
                }
                CameraEffect::StepDoorScrollRight => {
                    game::room_scrolls::door::horizontal(false, game)
                }
                CameraEffect::StepDoorScrollLeft => {
                    game::room_scrolls::door::horizontal(true, game)
                }
                CameraEffect::StepDoorScrollDown => game::room_scrolls::door::down(game),
                CameraEffect::StepDoorScrollUp => game::room_scrolls::door::up(game),
            }
        })
    }
    fn palette_fade(&mut self) -> u8 {
        self.foreign(|game| game::palette_effects::fading::all(12, game))
    }
    fn wait(&mut self, cause: EntryWait) -> u8 {
        self.foreign(|game| unsafe {
            match cause {
                EntryWait::Nmi => crate::game::display_refresh::wait(true, game),
                EntryWait::NoUpdate => crate::game::display_refresh::wait(false, game),
                EntryWait::Music => {
                    game::music::wait_for_music(game, |effect, game| match effect {
                        game::music::MusicWaitEffect::EnableNmi(enabled) => {
                            game::display_refresh::enable(enabled, game)
                        }
                        game::music::MusicWaitEffect::AdvanceQueue => {
                            game::music::HandleMusicQueue(game)
                        }
                    })
                }
                EntryWait::Mechanisms => room_mechanisms::run_door(game),
            }
        })
    }
    fn sfx_empty(&mut self, queue: SfxQueue) -> bool {
        let index = match queue {
            SfxQueue::One => 0,
            SfxQueue::Two => 1,
            SfxQueue::Three => 2,
        };
        self.foreign(|game| game.sfx_queue_is_empty(index))
    }
    fn queued_music(&mut self) -> u8 {
        self.foreign(game::music::HasQueuedMusic)
    }
    fn samus_command(&mut self, command: u16) {
        self.foreign(|game| game::session_runtime::player_command(command, game));
    }
    fn spawn_initial_hardcoded_object(&mut self) {
        room_mechanisms::creation::spawn(
            self,
            room_mechanisms::host::SpawnSource::Hardcoded {
                x: 8,
                y: 8,
                header: 0xb7eb,
            },
            &mut room_mechanisms::host::Contact::Absent,
        );
    }
    fn close_door(&mut self) {
        room_mechanisms::creation::close_door(self);
    }
    fn write_scroll_byte(&mut self, cell: u8, value: u8) {
        self.set_byte(0xcd20 + usize::from(cell), value);
    }
    fn write_scroll_pair(&mut self, byte_offset: u16, value: [u8; 2]) {
        self.write_slice(0xcd20 + usize::from(byte_offset), &value);
    }
    fn device_word(&mut self, register: DeviceWord, value: u16) {
        let reg = match register {
            DeviceWord::VramIncrement => VMAIN,
            DeviceWord::Dma1Mode => DMAP1,
            DeviceWord::Dma1SourceLow => A1T1L,
            DeviceWord::Dma1SourceBank => A1B1,
            DeviceWord::Dma1Count => DAS1L,
            DeviceWord::DmaEnable => MDMAEN,
        };
        self.foreign(|game| unsafe { crate::rtl::WriteRegWord(reg as u16, value, game) });
    }
    fn device_byte(&mut self, register: DeviceByte, value: u8) {
        let reg = match register {
            DeviceByte::VramIncrement => VMAIN,
            DeviceByte::DmaEnable => MDMAEN,
        };
        self.foreign(|game| unsafe { crate::rtl::WriteReg(reg as u16, value, game) });
    }
    fn clear_buffer(&mut self, operation: BufferClear) {
        self.foreign(|game| unsafe {
            match operation {
                BufferClear::InitialLevel => {
                    word_transfers!(game, descending, 12800;
                    le(level_data, 12799) <- value(0x8000));
                }
                BufferClear::DoorLevel => {
                    word_transfers!(game, descending, 3200;
                    le(level_data, 3199) <- value(0x8000),
                    native(0x13200) <- value(0x8000),
                    native(0x14b00) <- value(0x8000),
                    native(0x16400) <- value(0x8000));
                }
                BufferClear::InitialFx => game::room_backgrounds::clear_fx(LoadMode::Initial, game),
                BufferClear::InitialBg2 => {
                    game::room_backgrounds::clear_bg2(LoadMode::Initial, game)
                }
                BufferClear::DoorFx => game::room_backgrounds::clear_fx(LoadMode::Door, game),
                BufferClear::DoorBg2 => game::room_backgrounds::clear_bg2(LoadMode::Door, game),
                BufferClear::DoorDualBg => game::room_backgrounds::clear_dual_bg(game),
                BufferClear::TransitionXray => {
                    word_transfers!(game, descending, 256;
                    little(0x9dfe) <- value(0), little(0x9ffe) <- value(0));
                }
            }
        });
    }
    fn invalid_stage(&mut self, _full_address: u32) -> u8 {
        self.foreign(|game| unsafe { crate::rtl::Unreachable(game) as u8 })
    }
    fn device_vram_address(&mut self, destination: VramDestination) {
        let value = destination.compatibility_word();
        self.foreign(|game| unsafe { crate::rtl::WriteRegWord(VMADDL as u16, value, game) });
    }
    fn decompress(&mut self, source: AssetSource, destination: InstallBuffer) {
        let palette = matches!(destination, InstallBuffer::Palette);
        let palette_resource = match &source {
            AssetSource::Bound(AssetRef::Prepared(resource)) => Some(*resource),
            _ => None,
        };
        let palette_image = if palette { self.loaded_image() } else { None };
        if palette {
            self.session_mut().actor_identity.clear_palette_residency();
        }
        let source = source.compatibility_address(&self.session().world);
        let at = match destination {
            InstallBuffer::Tiles => 0x2000,
            InstallBuffer::CreTiles => 0x7000,
            InstallBuffer::Palette => 0xc200,
            InstallBuffer::TileTablePrimary => 0xa000,
            InstallBuffer::TileTableAfterCre => 0xa800,
            InstallBuffer::Background(target) => target.compatibility_offset(),
        };
        let produced =
            self.foreign(|game| game::decompression::decompress_to_ram(source, at, game));
        if let Some(image) = palette_image {
            // Only complete BG rows emitted by this actual decode establish residency.
            // Actor staging later establishes OBJ rows without erasing these receipts.
            for row in 0..(produced / 32).min(8) {
                self.session_mut().actor_identity.record_palette_residency(
                    row as u16,
                    palette_resource,
                    image.clone(),
                );
            }
        }
        #[cfg(sm_verify_dll)]
        if palette {
            crate::game::actors::composed_checks::palette_phase(self);
            crate::game::actors::composed_checks::reject_unstaged_spawn(self);
        }
    }
    fn decompress_vram(&mut self, source: AssetSource, destination: VramDecodeTarget) {
        let source = source.compatibility_address(&self.session().world);
        let target = match destination {
            VramDecodeTarget::MainTiles => 0,
            VramDecodeTarget::Cre => 0x5000,
        };
        self.foreign(|game| unsafe {
            game::display_refresh::DecompressToVRAM(source, target, game)
        });
    }
    fn copy_vram(&mut self, destination: VramDestination, source: AssetSource, bytes: u16) {
        let destination = destination.compatibility_word();
        let source = source.compatibility_address(&self.session().world);
        self.foreign(|game| unsafe {
            game::display_refresh::CopyToVramNow(destination, source, bytes, game)
        });
    }
    fn decode_level_and_copy_planes(&mut self, source: AssetSource) {
        let source = source.compatibility_address(&self.session().world);
        self.foreign(|game| game::room_state::decompress_level(source, game));
    }
    fn setup(&mut self, action: SetupAction) {
        let (kind, code) = action.compatibility_code(&self.session().world);
        self.foreign(|game| unsafe {
            match kind {
                SetupKind::Room => game::room_state::setup_script(0x8f0000 | u32::from(code), game),
                SetupKind::Door => game::room_state::door_script(0x8f0000 | u32::from(code), game),
            }
        });
    }
    fn spawn_room_object(&mut self, cursor: &ObjectCursor) {
        match cursor.spawn_object(&self.session().world) {
            SpawnObject::Legacy(address) => {
                room_mechanisms::creation::spawn(
                    self,
                    room_mechanisms::host::SpawnSource::Room {
                        entry: room_mechanisms::host::ProgramCursor(address),
                    },
                    &mut room_mechanisms::host::Contact::Absent,
                );
            }
            SpawnObject::Prepared {
                x,
                y,
                definition,
                argument,
            } => {
                self.session_mut().spawn_scratch.header =
                    u16::from_le_bytes(definition.to_ne_bytes());
                self.session_mut().spawn_scratch.coordinates = u16::from_le_bytes([x, y]);
                self.session_mut().spawn_scratch.argument =
                    u16::from_le_bytes(argument.to_ne_bytes());
                room_mechanisms::creation::spawn(
                    self,
                    room_mechanisms::host::SpawnSource::Room {
                        entry: room_mechanisms::host::ProgramCursor(0x12),
                    },
                    &mut room_mechanisms::host::Contact::Absent,
                );
            }
        }
    }
}
