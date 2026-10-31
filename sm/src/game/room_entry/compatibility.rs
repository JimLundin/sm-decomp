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
        ExternalField::GameMode => W::GAME_STATE,
        ExternalField::DebugMinimap => W::DEBUG_DISABLE_MINIMAP,
        ExternalField::TimerStatus => W::TIMER_STATUS,
        ExternalField::PreviousPlayerX => W::SAMUS_PREV_X_POS,
        ExternalField::PreviousPlayerY => W::SAMUS_PREV_Y_POS,
        ExternalField::MusicDataIndex => W::MUSIC_DATA_INDEX,
        ExternalField::MusicTrackIndex => W::MUSIC_TRACK_INDEX,
        ExternalField::HdmaEnabled => W::HDMA_OBJECTS_ENABLE_FLAG,
        ExternalField::NextIrqHandler => W::IRQHANDLER_NEXT_HANDLER,
        ExternalField::FxTilemap => W::FX_TILEMAP_PTR,
        ExternalField::BackgroundCopyByte => 0x5e,
        ExternalField::DisplayAndObjectSelect => 0x51,
        ExternalField::LoadingLowByte => 0x5f7,
    }
}
/// The owned camera word a room-entry camera field names, or the BG scroll
/// register mirror it names.
enum CameraWord {
    Layer1(usize),
    Layer2(usize),
    Bg1Offset(usize),
    Bg2Scroll(usize),
    Mirror(usize),
}
fn camera_word(field: CameraField) -> CameraWord {
    match field {
        CameraField::Layer1X => CameraWord::Layer1(0),
        CameraField::Layer1Y => CameraWord::Layer1(1),
        CameraField::Layer2X => CameraWord::Layer2(0),
        CameraField::Layer2Y => CameraWord::Layer2(1),
        CameraField::Bg1XOffset => CameraWord::Bg1Offset(0),
        CameraField::Bg1YOffset => CameraWord::Bg1Offset(1),
        CameraField::Bg2XScroll => CameraWord::Bg2Scroll(0),
        CameraField::Bg2YScroll => CameraWord::Bg2Scroll(1),
        CameraField::Bg1Hofs => CameraWord::Mirror(W::REG_BG1HOFS),
        CameraField::Bg1Vofs => CameraWord::Mirror(W::REG_BG1VOFS),
        CameraField::Bg2Hofs => CameraWord::Mirror(W::REG_BG2HOFS),
        CameraField::Bg2Vofs => CameraWord::Mirror(W::REG_BG2VOFS),
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
        let camera = &self.session().room.camera;
        match camera_word(field) {
            CameraWord::Layer1(axis) => camera.layer1[axis],
            CameraWord::Layer2(axis) => camera.layer2[axis],
            CameraWord::Bg1Offset(axis) => camera.bg1_offset[axis],
            CameraWord::Bg2Scroll(axis) => camera.bg2_scroll[axis],
            CameraWord::Mirror(at) => self.word(at),
        }
    }
    fn write_camera(&mut self, field: CameraField, value: u16) {
        // Layer 1 and BG1's offsets publish through the bytes: they are
        // also the lease's camera observations.
        match camera_word(field) {
            CameraWord::Layer1(axis) => sm_game::camera::set_layer1(self, axis, value),
            CameraWord::Bg1Offset(axis) => sm_game::camera::set_bg1_offset(self, axis, value),
            CameraWord::Layer2(axis) => self.session_mut().room.camera.layer2[axis] = value,
            CameraWord::Bg2Scroll(axis) => self.session_mut().room.camera.bg2_scroll[axis] = value,
            CameraWord::Mirror(at) => self.write_slice_le(at, value),
        }
    }
    fn call(&mut self, effect: LeafEffect) {
        self.foreign(|game| unsafe {
            match effect {
                LeafEffect::QueueDoorSound => {
                    game::music::queue_sfx2_max15(0x71, game);
                }
                LeafEffect::ResetSoundQueues => {
                    game::music::clear_sound_queues(game);
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
                    if crate::ram_map::word!(game.ELEVATOR_FLAGS) == 0 {
                        game::samus_animation::samus_draw_when_not_animating_or_dying(game);
                    }
                }
                LeafEffect::QueueEnemyTiles => {
                    actors::staging::queue_actor_tiles(&mut Runtime::new(game));
                }
                LeafEffect::DrawTimer => {
                    sm_game::escape_timer::draw(game);
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
                    game::music::load_room_music(game);
                }
                LeafEffect::UpdateMusicTrack => {
                    game::music::update_music_track_index(game);
                }
                LeafEffect::LoadChangedMusic => {
                    game::music::load_new_music_track_if_changed(game);
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
                    sm_game::menus::save_slots::save_area_map(game);
                }
                LeafEffect::LoadExploredMap => {
                    sm_game::menus::save_slots::load_area_map(game);
                }
                LeafEffect::MarkElevatorsUsed => {
                    sm_game::menus::save_slots::mark_elevator(game);
                }
                LeafEffect::EnableNmi => {
                    crate::game::display_refresh::enable(true, game);
                }
                LeafEffect::DisableNmi => {
                    crate::game::display_refresh::enable(false, game);
                }
                LeafEffect::EnableIrq | LeafEffect::WaitVblankEnableIrq => {
                    let immediate = matches!(effect, LeafEffect::WaitVblankEnableIrq);
                    if !immediate || game.ram().read_byte(0x84) & 0x30 != 0x30 {
                        game::scanline_irq::enable(immediate, game);
                    }
                }
                LeafEffect::DisableIrq => {
                    game::scanline_irq::disable(game);
                }
                LeafEffect::WaitVblankClearHdma => {
                    game::visual_runtime::clear_hdma(game);
                }
            }
        });
    }
    fn camera_call(&mut self, effect: CameraEffect) -> u8 {
        use game::room_scrolls::{self, door};
        match effect {
            CameraEffect::CalculateLayer2X => room_scrolls::layer2_position(Axis::X, self),
            CameraEffect::CalculateLayer2Y => room_scrolls::layer2_position(Axis::Y, self),
            CameraEffect::CalculateLayer2AndScrolls => {
                room_scrolls::update_scrolls(self);
                0
            }
            CameraEffect::CalculateBgScrolls => {
                room_scrolls::calculate_scrolls(self);
                0
            }
            CameraEffect::DisplayVisibleRoom => {
                room_scrolls::fill_viewport(self);
                0
            }
            CameraEffect::FixDoorMovingUp => {
                door::fix_up(self);
                0
            }
            CameraEffect::SetupDoorScroll => {
                door::begin(self);
                0
            }
            CameraEffect::StepDoorScrollRight => door::horizontal(false, self),
            CameraEffect::StepDoorScrollLeft => door::horizontal(true, self),
            CameraEffect::StepDoorScrollDown => door::down(self),
            CameraEffect::StepDoorScrollUp => door::up(self),
        }
    }
    fn palette_fade(&mut self) -> u8 {
        self.foreign(|game| sm_game::palette_effects::fading::all(12, game))
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
                            game::music::handle_music_queue(game)
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
        self.foreign(game::music::has_queued_music)
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
        self.foreign(|game| unsafe { crate::rtl::write_reg_word(reg as u16, value, game) });
    }
    fn device_byte(&mut self, register: DeviceByte, value: u8) {
        let reg = match register {
            DeviceByte::VramIncrement => VMAIN,
            DeviceByte::DmaEnable => MDMAEN,
        };
        self.foreign(|game| unsafe { crate::rtl::write_reg(reg as u16, value, game) });
    }
    fn clear_buffer(&mut self, operation: BufferClear) {
        self.foreign(|game| unsafe {
            match operation {
                BufferClear::InitialLevel => {
                    word_transfers!(game, descending, 12800;
                    le(LEVEL_DATA, 12799) <- value(0x8000));
                }
                BufferClear::DoorLevel => {
                    word_transfers!(game, descending, 3200;
                    le(LEVEL_DATA, 3199) <- value(0x8000),
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
        self.foreign(|game| crate::rtl::report_unreachable(game) as u8)
    }
    fn device_vram_address(&mut self, destination: VramDestination) {
        let value = destination.compatibility_word();
        self.foreign(|game| unsafe { crate::rtl::write_reg_word(VMADDL as u16, value, game) });
    }
    fn decompress(&mut self, source: AssetSource, destination: InstallBuffer) {
        let palette = matches!(destination, InstallBuffer::Palette);
        let palette_resource = match &source {
            AssetSource::Bound(AssetRef::Prepared(resource)) => Some(*resource),
            _ => None,
        };
        let assets = self.session().world.prepared_room_assets();
        let palette_image = if palette && assets.is_none() {
            self.loaded_image()
        } else {
            None
        };
        if palette {
            self.session_mut().actor_identity.clear_palette_residency();
        }
        let prepared = if matches!(
            destination,
            InstallBuffer::TileTablePrimary | InstallBuffer::TileTableAfterCre
        ) {
            self.session().world.prepared_table(&source)
        } else {
            assets.as_ref().map(|assets| {
                assets
                    .stream(&source)
                    .expect("selected room stream is unavailable")
            })
        };
        if assets.is_some() {
            assert!(prepared.is_some(), "selected room stream is unavailable");
        }
        let legacy = if prepared.is_none() {
            Some(source.compatibility_address(&self.session().world))
        } else {
            None
        };
        let at = match destination {
            InstallBuffer::Tiles => 0x2000,
            InstallBuffer::CreTiles => 0x7000,
            InstallBuffer::Palette => 0xc200,
            InstallBuffer::TileTablePrimary => 0xa000,
            InstallBuffer::TileTableAfterCre => 0xa800,
            InstallBuffer::Background(target) => target.compatibility_offset(),
        };
        let produced = self.foreign(|game| match prepared {
            Some(stream) => sm_game::decompression::decompress_prepared_table(&stream, at, game),
            None => sm_game::decompression::decompress_to_ram(legacy.unwrap(), at, game),
        });
        if palette {
            if let (Some(assets), Some(resource)) = (&assets, palette_resource) {
                for row in 0..(produced / 32).min(8) {
                    self.session_mut()
                        .actor_identity
                        .record_prepared_palette_residency(
                            row as u16,
                            resource,
                            assets.fingerprint(),
                        );
                }
            }
        }
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
        let assets = self.session().world.prepared_room_assets();
        let prepared = assets.as_ref().map(|assets| {
            assets
                .stream(&source)
                .expect("selected VRAM room stream is unavailable")
        });
        let legacy = if prepared.is_none() {
            Some(source.compatibility_address(&self.session().world))
        } else {
            None
        };
        let target = match destination {
            VramDecodeTarget::MainTiles => 0,
            VramDecodeTarget::Cre => 0x5000,
        };
        self.foreign(|game| match prepared {
            Some(stream) => game::display_refresh::decompress_prepared_vram(&stream, target, game),
            None => game::display_refresh::decompress_to_vram(legacy.unwrap(), target, game),
        });
    }
    fn copy_vram(&mut self, destination: VramDestination, source: AssetSource, bytes: u16) {
        let destination = destination.compatibility_word();
        let assets = if matches!(source, AssetSource::Staging(_)) {
            None
        } else {
            self.session().world.prepared_room_assets()
        };
        let source = source.compatibility_address(&self.session().world);
        self.foreign(|game| unsafe {
            game::display_refresh::copy_to_vram_with_source(
                destination,
                source,
                bytes,
                assets.as_deref(),
                game,
            )
        });
    }
    fn decode_level_and_copy_planes(&mut self, source: AssetSource) {
        let assets = self.session().world.prepared_room_assets();
        if let Some(assets) = assets {
            let stream = assets
                .stream(&source)
                .expect("selected Level stream is unavailable");
            self.foreign(|game| {
                sm_game::decompression::decompress_prepared_table(&stream, 0x10000, game);
                game::room_state::copy_level_planes(game);
            });
        } else {
            let source = source.compatibility_address(&self.session().world);
            self.foreign(|game| game::room_state::decompress_level(source, game));
        }
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
