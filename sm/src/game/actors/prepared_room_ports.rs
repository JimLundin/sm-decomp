//! Finite Windows room resource edges for the current exclusive typed owner.
//! Camera, entry cursors, terrain, music queues and waits run in sm_game.
//! These retained allocations, device writes and packed transfers are not
//! backend-independence proof. No callback opens another Session runtime.
#![forbid(unsafe_code)]
use super::prepared_execution::PreparedActorPorts;
use sm_data::cartridge::{RomImage, RomRecord, lorom_offset};
use sm_game::actors::owned_execution::room_bindings::{RoomPorts, RoomRecordSource};
use sm_game::actors::owned_execution::{ActorAllocations, ImportedActorAllocation};
use sm_game::cartridge::LoadedImage;
use sm_game::display::sequence::Registers;
use sm_game::persistent_session::PersistentSession;
use sm_game::room_entry::{content::*, effects::*};

fn external_address(field: ExternalField) -> usize {
    use crate::ram_map::words as W;
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

fn clear_presentation(clear: BufferClear, game: &mut crate::game_state::Game) {
    use sm_game::display::registers::*;
    let (count, first, second, value, destination, bytes) = match clear {
        BufferClear::DoorFx => (960, 0x477e, 0x4efe, 6222, Some(0x5880), 0xf00),
        BufferClear::DoorBg2 => (1024, 0x47fe, 0x4ffe, 824, Some(0x4800), 0x1000),
        BufferClear::DoorDualBg => (1024, 0x47fe, 0x4ffe, 824, None, 0),
        BufferClear::TransitionXray => (256, 0x9dfe, 0x9ffe, 0, None, 0),
        BufferClear::InitialFx | BufferClear::InitialBg2 => {
            let (destination, low, high, size) = if matches!(clear, BufferClear::InitialFx) {
                (0x5880, 0xa2f7, 0xa2f8, 0x780)
            } else {
                (0x4800, 0xa29a, 0xa29a, 0x800)
            };
            let assets = game.world().prepared_room_assets();
            // Low plane and then high plane; every register callback completes
            // before evaluating the next write, retaining DMA failure prefixes.
            for (mode, source, increment) in [(0x1808, low, 0), (0x1908, high, 0x80)] {
                game.write_register_word(
                    u16::try_from(VMADDL).expect("VRAM register"),
                    destination,
                );
                game.write_register_word(u16::try_from(DMAP1).expect("DMA register"), mode);
                game.write_register_word(u16::try_from(A1T1L).expect("DMA source"), source);
                game.write_register_word(u16::try_from(A1B1).expect("DMA bank"), 0x80);
                game.write_register_word(u16::try_from(DAS1L).expect("DMA size"), size);
                game.write_register(u16::try_from(VMAIN).expect("VRAM increment"), increment);
                sm_game::display::transfer::RoomDma::start_room_dma(game, 2, assets.as_deref());
            }
            return;
        }
        BufferClear::InitialLevel | BufferClear::DoorLevel => {
            unreachable!("level clear runs on the typed terrain owner")
        }
    };
    let encoded = if matches!(clear, BufferClear::DoorDualBg) {
        u16::to_ne_bytes(value)
    } else {
        u16::to_le_bytes(value)
    };
    for index in 0..count {
        for at in [first - 2 * index, second - 2 * index] {
            game.ram().write_byte(at, encoded[0]);
            game.ram().write_byte(at + 1, encoded[1]);
        }
    }
    if let Some(destination) = destination {
        sm_game::display::transfer::copy_to_vram_now(destination, 0x7e4000, bytes, None, game);
    }
}

fn residual_room_setup(_code: u16, game: &mut crate::game_state::Game) {
    crate::rtl::report_unreachable(game);
}

fn residual_door_setup(code: u16, game: &mut crate::game_state::Game) {
    match code {
        0xe4e0 => {
            game.ram().write_byte(0x56, 7);
            game.write_register(
                u16::try_from(sm_game::display::registers::BGMODE).expect("mode register"),
                7,
            );
            sm_game::frame_display_word!(game.AffineWords0Pair = 256);
            sm_game::frame_display_word!(game.AffineWords3Pair = 256);
            sm_game::frame_display_word!(game.AffineWords1Pair = 0);
            sm_game::frame_display_word!(game.AffineWords2Pair = 0);
            sm_game::frame_display_word!(game.AffineWords4Pair = 128);
            sm_game::frame_display_word!(game.AffineWords5Pair = 1008);
            game.ram()
                .copy_in(crate::ram_map::words::IRQ_ENABLE_MODE7, &1u16.to_le_bytes());
            game.ram().copy_in(0x7e1, &34u16.to_le_bytes());
            game.ram().copy_in(0x7e3, &60u16.to_le_bytes());
        }
        0xe513 => {
            game.ram().write_byte(0x56, 9);
            game.ram()
                .copy_in(crate::ram_map::words::IRQ_ENABLE_MODE7, &0u16.to_le_bytes());
        }
        _ => {
            crate::rtl::report_unreachable(game);
        }
    }
}

impl RoomPorts for PreparedActorPorts<'_> {
    fn room_scroll_alias_byte(&self, owner: &PersistentSession, offset: usize) -> u8 {
        let at = 0xcd20usize
            .checked_add(offset)
            .expect("scroll alias extent");
        sm_game::session_encoding::projected_byte(&owner.gameplay, at)
            .unwrap_or_else(|| self.actor_alias_byte(&owner.gameplay, at))
    }
    fn store_room_scroll_alias_byte(
        &mut self,
        owner: &mut PersistentSession,
        offset: usize,
        value: u8,
    ) {
        let at = 0xcd20usize
            .checked_add(offset)
            .expect("scroll alias extent");
        if !sm_game::session_encoding::write_projected_byte(&mut owner.gameplay, at, value) {
            self.set_actor_alias_byte(&mut owner.gameplay, at, value);
        }
    }
    fn store_room_scroll_alias_pair(
        &mut self,
        owner: &mut PersistentSession,
        offset: usize,
        value: [u8; 2],
    ) {
        let at = 0xcd20usize
            .checked_add(offset)
            .expect("scroll alias extent");
        let end = at.checked_add(2).expect("scroll pair extent");
        assert!(end <= 0x20000);
        // Admit the whole selected physical extent before publishing either
        // canonical owner, including the last saved-map byte and its neighbor.
        self.store_room_scroll_alias_byte(owner, offset, value[0]);
        self.store_room_scroll_alias_byte(owner, offset + 1, value[1]);
    }
    fn room_low_record(
        &mut self,
        owner: &mut PersistentSession,
        allocations: &mut ActorAllocations,
        source: RoomRecordSource,
        bank: u8,
        address: u16,
    ) -> RomRecord {
        let record = self.physical_effect(owner, |game| match source {
            RoomRecordSource::Room => {
                sm_game::room_state::EnemyGraphicsHost::rom_record(game, bank, address)
            }
            RoomRecordSource::NativePointer => {
                let absolute = (u32::from(bank) << 16) | u32::from(address);
                crate::game::session_runtime::native_rom_diagnostic(absolute, game);
                let _ = game.legacy_native_rom_ptr();
                RomRecord {
                    image: game.capture_native_rom(),
                    base: usize::try_from(lorom_offset(absolute)).expect("native room record"),
                }
            }
            RoomRecordSource::Closing | RoomRecordSource::BackgroundInitial => {
                let absolute = (u32::from(bank) << 16) | u32::from(address);
                crate::game::session_runtime::invalid_window(game, absolute);
                #[cfg(sm_verify_dll)]
                if matches!(source, RoomRecordSource::BackgroundInitial) {
                    crate::room_background_observer::diagnostic(absolute, game);
                }
                RomRecord {
                    image: game.loaded_image().clone(),
                    base: usize::try_from(lorom_offset(absolute)).expect("room record"),
                }
            }
        });
        self.refresh_allocations(owner, allocations);
        record
    }
    fn room_pointer_diagnostic(
        &mut self,
        owner: &mut PersistentSession,
        allocations: &mut ActorAllocations,
        bank: u8,
        address: u16,
    ) {
        self.physical_effect(owner, |game| {
            crate::game::session_runtime::native_rom_diagnostic(
                (u32::from(bank) << 16) | u32::from(address),
                game,
            );
            let _ = game.legacy_native_rom_ptr();
        });
        self.refresh_allocations(owner, allocations);
    }
    fn room_native_diagnostic(
        &mut self,
        owner: &mut PersistentSession,
        allocations: &mut ActorAllocations,
        address: u32,
    ) {
        self.physical_effect(owner, |game| {
            crate::game::session_runtime::native_rom_diagnostic(address, game)
        });
        self.refresh_allocations(owner, allocations);
    }
    fn room_invalid_selector(&mut self, owner: &mut PersistentSession, _address: u32) -> u8 {
        self.physical_effect(owner, |game| u8::from(crate::rtl::report_unreachable(game)))
    }
    fn physical_slope_alias_byte(
        &mut self,
        owner: &mut PersistentSession,
        block: u16,
        lane: u8,
    ) -> u8 {
        self.physical_slope_byte(owner, block, lane)
    }
    fn room_physical_scroll_byte(
        &self,
        _owner: &PersistentSession,
        word: sm_game::display_scroll::Word,
        high: bool,
    ) -> u8 {
        sm_game::display_scroll::Access::physical_scroll_byte(&*self.game, word, high)
    }
    fn store_room_physical_scroll_byte(
        &mut self,
        owner: &mut PersistentSession,
        word: sm_game::display_scroll::Word,
        high: bool,
        value: u8,
    ) {
        self.physical_effect(owner, |game| {
            sm_game::display_scroll::Access::write_physical_scroll_byte(game, word, high, value)
        });
    }
    fn room_external(&mut self, owner: &mut PersistentSession, field: ExternalField) -> u16 {
        self.physical_effect(owner, |game| {
            let at = external_address(field);
            let low = game.ram().read_byte(at);
            if matches!(
                field,
                ExternalField::BackgroundCopyByte | ExternalField::LoadingLowByte
            ) {
                u16::from(low)
            } else {
                u16::from_le_bytes([low, game.ram().read_byte(at + 1)])
            }
        })
    }
    fn store_room_external(
        &mut self,
        owner: &mut PersistentSession,
        field: ExternalField,
        value: u16,
    ) {
        self.physical_effect(owner, |game| {
            let at = external_address(field);
            let [low, high] = value.to_le_bytes();
            game.ram().write_byte(at, low);
            if !matches!(
                field,
                ExternalField::BackgroundCopyByte | ExternalField::LoadingLowByte
            ) {
                game.ram().write_byte(at + 1, high);
            }
        });
    }
    fn room_queue_scrolling_tiles(
        &mut self,
        owner: &mut PersistentSession,
        offset: u16,
        layout: sm_game::display::transfer::ScrollingLayout,
    ) {
        self.physical_effect(owner, |game| {
            sm_game::display_refresh::queue_scrolling_tiles(offset, layout, game)
        });
    }
    fn room_flush_tilemaps(&mut self, owner: &mut PersistentSession) {
        self.physical_effect(owner, sm_game::display_refresh::Host::flush_tilemaps);
    }
    fn room_store_background_size(&mut self, owner: &mut PersistentSession, value: u16) {
        self.physical_effect(owner, |game| {
            game.ram()
                .copy_in(crate::ram_map::words::SIZE_OF_BG2, &value.to_le_bytes())
        });
    }
    fn room_music_effect(
        &mut self,
        owner: &mut PersistentSession,
        effect: sm_game::music_requests::MusicEffect,
        meaning: Option<&sm_game::session_owners::QueuedMusicMeaning>,
    ) -> Result<(), sm_game::scene::state::DisplayError> {
        self.physical_music_effect(owner, effect, meaning)
    }
    fn room_leaf(&mut self, owner: &mut PersistentSession, effect: LeafEffect) {
        self.physical_effect(owner, |game| match effect {
            LeafEffect::LoadEnemyGraphics => sm_game::room_state::load_enemy_graphics(game),
            LeafEffect::LoadSpritePalettes => crate::game::room_state::load_sprite_palettes(game),
            LeafEffect::EnableNmi => sm_game::display_refresh::enable(true, game),
            LeafEffect::DisableNmi => sm_game::display_refresh::enable(false, game),
            LeafEffect::EnableIrq => sm_game::display_refresh::enable_scanline_irq(false, game),
            LeafEffect::WaitVblankEnableIrq => {
                if game.ram().read_byte(0x84) & 0x30 != 0x30 {
                    sm_game::display_refresh::enable_scanline_irq(true, game);
                }
            }
            LeafEffect::DisableIrq => sm_game::display_refresh::disable_scanline_irq(game),
            // These arms must move at the actual same-open host, never through
            // a Game getter facade or an invented success callback.
            LeafEffect::DrawTimer => sm_game::escape_timer::draw(game),
            LeafEffect::ResetSoundQueues
            | LeafEffect::ClearTransitionSounds
            | LeafEffect::QueueDoorSound
            | LeafEffect::SelectEnemies
            | LeafEffect::RunEnemies
            | LeafEffect::DrawScene
            | LeafEffect::EnsureSamusDrawn
            | LeafEffect::QueueEnemyTiles
            | LeafEffect::SpinJumpSound
            | LeafEffect::SetLiquidPhysics
            | LeafEffect::InitializeEnemies
            | LeafEffect::LoadRoomMusic
            | LeafEffect::UpdateMusicTrack
            | LeafEffect::LoadChangedMusic
            | LeafEffect::UpdateBeamTilesAndPalette
            | LeafEffect::LoadRoomAssets
            | LeafEffect::LoadSuitTargetPalette
            | LeafEffect::LoadFxHeader
            | LeafEffect::SpawnBg3ScrollHdma
            | LeafEffect::StopHdmaEffects
            | LeafEffect::WaitVblankClearHdma
            | LeafEffect::InitializeRoomSpecialEffects
            | LeafEffect::LoadExploredMap
            | LeafEffect::SaveExploredMap
            | LeafEffect::MarkElevatorsUsed => {
                unreachable!("room leaf runs on the current typed host")
            }
        });
    }
    fn room_setup_presentation(
        &mut self,
        owner: &mut PersistentSession,
        kind: SetupKind,
        code: u16,
    ) {
        self.physical_effect(owner, |game| match kind {
            SetupKind::Room => residual_room_setup(code, game),
            SetupKind::Door => residual_door_setup(code, game),
        });
    }
    fn room_initialization_register(
        &mut self,
        owner: &mut PersistentSession,
        register: u16,
        value: u8,
    ) {
        match register {
            0x4320..=0x437f => self.physical_effect(owner, |game| {
                sm_game::menus::host::MenuRuntime::write_dma_register(game, register, value)
            }),
            0x2181..=0x2183 => self.physical_effect(owner, |game| {
                crate::game::session_runtime::prepared_wram_pointer_register(register, value, game)
            }),
            0x420b..=0x420c => {
                self.physical_effect(owner, |game| game.write_register(register, value))
            }
            _ => panic!("room initialization register outside finite device family"),
        }
    }
    fn room_device_word(&mut self, owner: &mut PersistentSession, word: DeviceWord, value: u16) {
        use sm_game::display::registers::*;
        let register = match word {
            DeviceWord::VramIncrement => VMAIN,
            DeviceWord::Dma1Mode => DMAP1,
            DeviceWord::Dma1SourceLow => A1T1L,
            DeviceWord::Dma1SourceBank => A1B1,
            DeviceWord::Dma1Count => DAS1L,
            DeviceWord::DmaEnable => MDMAEN,
        };
        self.physical_effect(owner, |game| {
            game.write_register_word(u16::try_from(register).expect("room register"), value)
        });
    }
    fn room_device_byte(&mut self, owner: &mut PersistentSession, byte: DeviceByte, value: u8) {
        use sm_game::display::registers::*;
        let register = match byte {
            DeviceByte::VramIncrement => VMAIN,
            DeviceByte::DmaEnable => MDMAEN,
        };
        self.physical_effect(owner, |game| {
            game.write_register(u16::try_from(register).expect("room register"), value)
        });
    }
    fn room_vram_address(&mut self, owner: &mut PersistentSession, destination: VramDestination) {
        let word = destination.compatibility_word();
        self.physical_effect(owner, |game| {
            game.write_register_word(
                u16::try_from(sm_game::display::registers::VMADDL).expect("VRAM register"),
                word,
            )
        });
    }
    fn room_decompress(
        &mut self,
        owner: &mut PersistentSession,
        source: AssetSource,
        destination: InstallBuffer,
    ) {
        let palette = matches!(destination, InstallBuffer::Palette);
        let resource = match &source {
            AssetSource::Bound(AssetRef::Prepared(resource)) => Some(*resource),
            _ => None,
        };
        let assets = owner.gameplay.world.prepared_room_assets();
        let palette_image: Option<RomImage> = if palette && assets.is_none() {
            Some(self.game.loaded_image().clone())
        } else {
            None
        };
        if palette {
            owner.gameplay.actor_identity.clear_palette_residency();
        }
        let stream = if matches!(
            destination,
            InstallBuffer::TileTablePrimary | InstallBuffer::TileTableAfterCre
        ) {
            owner.gameplay.world.prepared_table(&source)
        } else {
            assets.as_ref().map(|assets| {
                assets
                    .stream(&source)
                    .expect("selected room stream unavailable")
            })
        };
        if assets.is_some() {
            assert!(stream.is_some());
        }
        let legacy = stream
            .is_none()
            .then(|| source.compatibility_address(&owner.gameplay.world));
        let at = match destination {
            InstallBuffer::Tiles => 0x2000,
            InstallBuffer::CreTiles => 0x7000,
            InstallBuffer::Palette => 0xc200,
            InstallBuffer::TileTablePrimary => 0xa000,
            InstallBuffer::TileTableAfterCre => 0xa800,
            InstallBuffer::Background(target) => target.compatibility_offset(),
        };
        let produced = self.physical_effect(owner, |game| match stream {
            Some(stream) => sm_game::decompression::decompress_prepared_table(&stream, at, game),
            None => sm_game::decompression::decompress_to_ram(
                legacy.expect("legacy room source"),
                at,
                game,
            ),
        });
        if palette {
            for row in 0..(produced / 32).min(8) {
                let row = u16::try_from(row).expect("palette row");
                if let (Some(assets), Some(resource)) = (&assets, resource) {
                    owner
                        .gameplay
                        .actor_identity
                        .record_prepared_palette_residency(row, resource, assets.fingerprint());
                }
                if let Some(image) = &palette_image {
                    owner.gameplay.actor_identity.record_palette_residency(
                        row,
                        resource,
                        image.clone(),
                    );
                }
            }
        }
    }
    fn room_decompress_vram(
        &mut self,
        owner: &mut PersistentSession,
        source: AssetSource,
        destination: VramDecodeTarget,
    ) {
        let assets = owner.gameplay.world.prepared_room_assets();
        let stream = assets.as_ref().map(|assets| {
            assets
                .stream(&source)
                .expect("selected VRAM room stream unavailable")
        });
        let legacy = stream
            .is_none()
            .then(|| source.compatibility_address(&owner.gameplay.world));
        let target = match destination {
            VramDecodeTarget::MainTiles => 0,
            VramDecodeTarget::Cre => 0x5000,
        };
        self.physical_effect(owner, |game| match stream {
            Some(stream) => {
                sm_game::display_refresh::decompress_prepared_vram(&stream, target, game)
            }
            None => sm_game::display_refresh::decompress_to_vram(
                legacy.expect("legacy VRAM source"),
                target,
                game,
            ),
        });
    }
    fn room_copy_vram(
        &mut self,
        owner: &mut PersistentSession,
        destination: VramDestination,
        source: AssetSource,
        bytes: u16,
    ) {
        let destination = destination.compatibility_word();
        let assets = if matches!(source, AssetSource::Staging(_)) {
            None
        } else {
            owner.gameplay.world.prepared_room_assets()
        };
        let source = source.compatibility_address(&owner.gameplay.world);
        self.physical_effect(owner, |game| {
            sm_game::display::transfer::copy_to_vram_now(
                destination,
                source,
                bytes,
                assets.as_deref(),
                game,
            )
        });
    }
    fn room_clear_presentation(&mut self, owner: &mut PersistentSession, clear: BufferClear) {
        // Implemented by the fixed clear/device sequence below, without opening
        // a compatibility Runtime around the leased PersistentSession.
        self.physical_effect(owner, |game| clear_presentation(clear, game));
    }
    fn room_decode_imported(
        &mut self,
        owner: &mut PersistentSession,
        allocations: &mut ActorAllocations,
        source: AssetSource,
    ) {
        let source = source.compatibility_address(&owner.gameplay.world);
        self.physical_effect(owner, |game| {
            sm_game::decompression::decompress_to_ram(source, 0x10000, game);
        });
        self.refresh_allocations(owner, allocations);
    }
    fn room_invalid_stage(&mut self, owner: &mut PersistentSession, _address: u32) -> u8 {
        self.physical_effect(owner, |game| u8::from(crate::rtl::report_unreachable(game)))
    }
}
