//! Actual room acquisition, installation and continuations on the prepared owner.
#![forbid(unsafe_code)]
use crate::actors::owned_execution::{ActorAllocations, PreparedActorState};
use crate::room_entry::{api::*, content::*, effects::*};
use crate::session_access::SessionAccess;
use sm_data::cartridge::{RomRecord, lorom_offset};
#[path = "prepared_initializers.rs"]
mod prepared_initializers;

#[derive(Clone, Copy, Debug)]
pub enum RoomRecordSource {
    Room,
    NativePointer,
    Closing,
    BackgroundInitial,
}
/// Finite interactions whose physical/display/audio owner is external to room rules.
/// No default success, synthetic content or independent room state is supplied.
pub trait RoomPorts {
    fn room_queue_scrolling_tiles(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        offset: u16,
        layout: crate::display::transfer::ScrollingLayout,
    );
    fn room_flush_tilemaps(&mut self, owner: &mut crate::persistent_session::PersistentSession);
    fn room_store_background_size(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        value: u16,
    );
    fn room_music_effect(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        effect: crate::music_requests::MusicEffect,
        meaning: Option<&crate::session_owners::QueuedMusicMeaning>,
    ) -> Result<(), crate::scene::state::DisplayError>;
    fn room_initialization_register(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        register: u16,
        value: u8,
    );
    fn room_native_diagnostic(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        allocations: &mut ActorAllocations,
        address: u32,
    );
    /// Full relative scroll offset beyond the adjacent saved area-map owner.
    fn room_scroll_alias_byte(
        &self,
        owner: &crate::persistent_session::PersistentSession,
        byte_offset: usize,
    ) -> u8;
    fn store_room_scroll_alias_byte(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        byte_offset: usize,
        value: u8,
    );
    /// Whole selected extent is checked before either byte is published. Resolve
    /// both canonical owners, including a final area-map byte crossing this port.
    fn store_room_scroll_alias_pair(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        byte_offset: usize,
        value: [u8; 2],
    );
    fn room_low_record(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        allocations: &mut ActorAllocations,
        source: RoomRecordSource,
        bank: u8,
        address: u16,
    ) -> RomRecord;
    fn room_pointer_diagnostic(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        allocations: &mut ActorAllocations,
        bank: u8,
        address: u16,
    );
    fn room_invalid_selector(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        address: u32,
    ) -> u8;
    fn physical_slope_alias_byte(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        block: u16,
        lane: u8,
    ) -> u8;
    fn room_physical_scroll_byte(
        &self,
        owner: &crate::persistent_session::PersistentSession,
        word: crate::display_scroll::Word,
        high: bool,
    ) -> u8;
    fn store_room_physical_scroll_byte(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        word: crate::display_scroll::Word,
        high: bool,
        value: u8,
    );
    fn room_external(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        field: ExternalField,
    ) -> u16;
    fn store_room_external(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        field: ExternalField,
        value: u16,
    );
    fn room_leaf(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        effect: LeafEffect,
    );
    fn room_setup_presentation(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        kind: SetupKind,
        code: u16,
    );
    fn room_device_word(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        word: DeviceWord,
        value: u16,
    );
    fn room_device_byte(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        byte: DeviceByte,
        value: u8,
    );
    fn room_vram_address(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        destination: VramDestination,
    );
    fn room_decompress(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        source: AssetSource,
        destination: InstallBuffer,
    );
    fn room_decompress_vram(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        source: AssetSource,
        destination: VramDecodeTarget,
    );
    fn room_copy_vram(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        destination: VramDestination,
        source: AssetSource,
        bytes: u16,
    );
    fn room_clear_presentation(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        clear: BufferClear,
    );
    fn room_decode_imported(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        allocations: &mut ActorAllocations,
        source: AssetSource,
    );
    fn room_invalid_stage(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        address: u32,
    ) -> u8;
}

impl PreparedActorState<'_> {
    fn capture_room<P>(&mut self, source: RoomRecordSource, bank: u8, address: u16) -> Capture<P> {
        let record = if address & 0x8000 == 0 {
            self.aliases
                .room_low_record(self.owner, self.allocations, source, bank, address)
        } else {
            let image = if matches!(source, RoomRecordSource::NativePointer) {
                self.allocations.native().clone()
            } else {
                self.allocations.loaded().clone()
            };
            RomRecord {
                image,
                base: usize::try_from(lorom_offset((u32::from(bank) << 16) | u32::from(address)))
                    .expect("room record offset"),
            }
        };
        self.refresh_inputs();
        Capture::Legacy {
            image: record.image,
            base: record.base,
            access: RecordAccess::Native,
            world: self.session().world.clone(),
        }
    }
    /// Real door admission; collision locals are discarded before the next frame.
    pub fn room_begin(&mut self, arrival: ArrivalRequest) -> EntryPoll {
        let result = crate::room_entry::selection::begin(self, arrival);
        self.refresh_inputs();
        result
    }
    /// The caller captures signal at its actual outer dispatch. Existing room
    /// primary/secondary and transition_selector retain all suspension cursors.
    pub fn room_resume(&mut self, signal: EntrySignal) -> EntryPoll {
        // IRQ is a separate synchronous child and never replaces a suspended
        // outer room call. Capture outer dispatch before any effect can change mode.
        let captured = if signal == EntrySignal::ScrollIrq {
            signal
        } else {
            *self
                .session_mut()
                .room
                .entry
                .execution
                .signal
                .get_or_insert(signal)
        };
        let result = crate::room_entry::protocol::resume(self, captured);
        if signal != EntrySignal::ScrollIrq && result.returned_byte() == 0 {
            self.session_mut().room.entry.execution.signal = None;
        }
        self.refresh_inputs();
        result
    }
    pub fn room_slope_pair(&mut self, block: u16) -> u16 {
        // Bank7F is owned physical WRAM; a carried bank80 access may be I/O or
        // open bus and must invoke its distinct physical owner in source order.
        let mut values = [0; 2];
        for (lane, value) in values.iter_mut().enumerate() {
            let address = 0x7f6402u32 + u32::from(block) + u32::try_from(lane).expect("slope lane");
            *value = if address < 0x800000 {
                self.current_byte(usize::try_from(address - 0x7e0000).expect("slope allocation"))
            } else {
                let value = self.aliases.physical_slope_alias_byte(
                    self.owner,
                    block,
                    u8::try_from(lane).expect("slope lane"),
                );
                self.refresh_inputs();
                value
            };
        }
        u16::from_le_bytes(values)
    }
    fn room_door_setup(&mut self, code: u16) -> bool {
        match code {
            0xb971 | 0xe1d8 => {
                use crate::constants::bank_87::*;
                let (animation, definition) = if code == 0xb971 {
                    (ADDR_K_ANIMTILES_WRECKED_SHIP_TRADMILL_RIGHT, 0xb64b)
                } else {
                    (ADDR_K_ANIMTILES_WRECKED_SHIP_TRADMILL_LEFT, 0xb64f)
                };
                crate::animated_tiles::execution::spawn(
                    self,
                    u16::try_from(animation).expect("treadmill animation"),
                );
                self.refresh_inputs();
                crate::room_state::setup::Host::setup_mechanism(self, 4, 9, definition);
            }
            0xe26c | 0xe291 => {
                let room = &mut self.session_mut().room;
                if code == 0xe26c {
                    room.set_main_native(2, 0xff00);
                    room.set_main_native(1, 2496);
                    room.set_main_native(3, 0xffe0);
                } else {
                    room.set_main_native(2, 256);
                    room.main_variables[1] = 64;
                    room.main_variables[3] = 32;
                }
                crate::player_effects::command(self, 0);
                crate::room_state::setup::Host::setup_mechanism(self, 1, 0, 0xb8f9);
            }
            0xe301 | 0xe309 => {
                if code == 0xe309 {
                    crate::room_scrolls::state::store_pair(self, 0, 514u16.to_ne_bytes());
                }
                crate::player_effects::command(self, 1);
            }
            _ => return crate::room_scrolls::apply(0x8f0000 | u32::from(code), self),
        }
        true
    }
}
impl EntryContent for PreparedActorState<'_> {
    fn contact_bts(&mut self) -> u8 {
        crate::room_terrain::bts(self, usize::from(self.session().room.collision_cursor))
    }
    fn legacy_word(&mut self, bank: u8, address: u16) -> u16 {
        if address & 0x8000 == 0 {
            self.aliases
                .room_pointer_diagnostic(self.owner, self.allocations, bank, address);
            self.refresh_inputs();
        }
        // Capture AFTER the actual diagnostic, including replacement allocation.
        let image = self.allocations.loaded().clone();
        let base = usize::try_from(lorom_offset(
            (u32::from(bank) << 16) | 0x8000 | u32::from(address),
        ))
        .expect("contact record");
        assert!(base + 2 <= image.len());
        u16::from_le_bytes([image.read(base), image.read(base + 1)])
    }
    fn load_station_index(&self) -> u16 {
        self.session().menu_screens.save_selection.load_station
    }
    fn legacy_station(&mut self) -> StationCapture {
        // Loaded table allocation is captured before the live area/index reads.
        let image = self.allocations.loaded().clone();
        let base = image.table_word(
            usize::try_from(lorom_offset(0x80c4b5)).expect("station table"),
            isize::try_from(self.session().room.area).expect("station area"),
        );
        let offset = i32::from(base) + 14 * i32::from(self.load_station_index());
        let address = u16::from_le_bytes(
            offset.to_le_bytes()[..2]
                .try_into()
                .expect("station address"),
        );
        self.native_room_capture(0x80, address)
    }
    fn room_capture<P>(&mut self, bank: u8, address: u16) -> Capture<P> {
        self.capture_room(RoomRecordSource::Room, bank, address)
    }
    fn native_room_capture<P>(&mut self, bank: u8, address: u16) -> Capture<P> {
        self.capture_room(RoomRecordSource::NativePointer, bank, address)
    }
    fn closing_capture<P>(&mut self, bank: u8, address: u16) -> Capture<P> {
        self.capture_room(RoomRecordSource::Closing, bank, address)
    }
    fn background_capture<P>(&mut self, address: u16, mode: LoadMode) -> Capture<P> {
        if matches!(mode, LoadMode::Door) && address & 0x8000 == 0 {
            self.owner.poison();
            std::panic::panic_any(RoomFault::BackgroundWindow);
        }
        let mut captured = self.capture_room(RoomRecordSource::BackgroundInitial, 0x8f, address);
        if let Capture::Legacy { access, .. } = &mut captured {
            *access = if matches!(mode, LoadMode::Initial) {
                RecordAccess::Native
            } else {
                RecordAccess::LittleEndian
            };
        }
        captured
    }
    fn invalid_selector(&mut self, address: u32) -> Option<SelectorCursor> {
        let value = self.aliases.room_invalid_selector(self.owner, address);
        self.refresh_inputs();
        (value != 0).then_some(SelectorCursor::Legacy(u16::from(value)))
    }
    fn legacy_tileset(&mut self, index: u8) -> TilesetCapture {
        let image = self.allocations.loaded().clone();
        let base = usize::try_from(lorom_offset(0x8fe7a7)).expect("tileset table")
            + usize::from(index) * 2;
        assert!(base + 2 <= image.len());
        let address = u16::from_le_bytes([image.read(base), image.read(base + 1)]);
        self.room_capture(0x8f, address)
    }
    fn asset_bytes(&self, slot: AssetSlot, read: AssetRead) -> [u8; 3] {
        let entry = &self.session().room.entry;
        let (asset, neighbor) = match slot {
            AssetSlot::Level => (
                &entry.level,
                self.session().world.asset_bytes(&entry.tile_table)[0],
            ),
            AssetSlot::TileTable => (
                &entry.tile_table,
                self.session().world.asset_bytes(&entry.tiles)[0],
            ),
            AssetSlot::Tiles => (
                &entry.tiles,
                self.session().world.asset_bytes(&entry.palette)[0],
            ),
            AssetSlot::Palette => (&entry.palette, entry.music_track.to_le_bytes()[0]),
        };
        let bytes = self.session().world.asset_bytes(asset);
        match read {
            AssetRead::ThreeLittleEndian => bytes,
            AssetRead::FourNativeMasked => {
                let value = u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], neighbor]) & 0xffffff;
                value.to_le_bytes()[..3].try_into().expect("room asset")
            }
        }
    }
}

impl EntryHost for PreparedActorState<'_> {}
impl EntryEffects for PreparedActorState<'_> {
    fn read_external(&mut self, field: ExternalField) -> u16 {
        match field {
            ExternalField::GameMode => self.session().game_loop.mode,
            ExternalField::PreviousPlayerX => self.session().player.previous_position.x.whole,
            ExternalField::PreviousPlayerY => self.session().player.previous_position.y.whole,
            ExternalField::TimerStatus => self.session().timing.timer_status,
            ExternalField::LoadingLowByte => {
                u16::from(self.session().minimap.disabled.to_le_bytes()[0])
            }
            ExternalField::MusicDataIndex => {
                self.session()
                    .control
                    .requests()
                    .expect("owned room audio")
                    .music
                    .data_index
            }
            ExternalField::MusicTrackIndex => {
                self.session()
                    .control
                    .requests()
                    .expect("owned room audio")
                    .music
                    .track_index
            }
            ExternalField::DisplayAndObjectSelect => {
                let controls = &self
                    .session()
                    .presentation
                    .owner()
                    .expect("owned room display")
                    .state
                    .controls;
                u16::from_le_bytes([controls.brightness, controls.object_layout])
            }
            _ => {
                let value = self.aliases.room_external(self.owner, field);
                self.refresh_inputs();
                value
            }
        }
    }
    fn write_external(&mut self, field: ExternalField, value: u16) {
        match field {
            ExternalField::GameMode => self.session_mut().game_loop.mode = value,
            ExternalField::PreviousPlayerX => {
                self.session_mut().player.previous_position.x.whole = value
            }
            ExternalField::PreviousPlayerY => {
                self.session_mut().player.previous_position.y.whole = value
            }
            ExternalField::TimerStatus => self.session_mut().timing.timer_status = value,
            ExternalField::LoadingLowByte => {
                let old = self.session().minimap.disabled;
                self.session_mut().minimap.disabled = (old & 0xff00) | (value & 0xff);
            }
            ExternalField::DisplayAndObjectSelect => {
                let [brightness, layout] = value.to_le_bytes();
                let controls = &mut self
                    .session_mut()
                    .presentation
                    .owner_mut()
                    .expect("owned room display")
                    .state
                    .controls;
                controls.brightness = brightness;
                controls.object_layout = layout;
            }
            _ => self.aliases.store_room_external(self.owner, field, value),
        }
        self.refresh_inputs();
    }
    fn call(&mut self, effect: LeafEffect) {
        match effect {
            LeafEffect::ResetSoundQueues => {
                crate::music_requests::reset_sound_queues(self).expect("room sound reset")
            }
            LeafEffect::QueueDoorSound => {
                crate::music_requests::queue_sfx(1, 0x71, 15, self).expect("room door sound");
            }
            LeafEffect::SetLiquidPhysics => crate::player_movement::speed::set_liquid_physics(self),
            LeafEffect::LoadFxHeader => crate::room_state::fx::load_header(self),
            LeafEffect::LoadSpritePalettes => crate::room_state::load_sprite_palettes(self),
            LeafEffect::SpawnBg3ScrollHdma => {
                crate::actors::boss_hdma_owned::spawn_bg3_scroll(self)
            }
            LeafEffect::StopHdmaEffects => self.room_clear_hdma(false),
            LeafEffect::WaitVblankClearHdma => self.room_clear_hdma(true),
            LeafEffect::InitializeRoomSpecialEffects => self.room_initialize_special_effects(),
            LeafEffect::LoadExploredMap => self.room_load_explored_map(),
            LeafEffect::SaveExploredMap => self.room_save_explored_map(),
            LeafEffect::MarkElevatorsUsed => self.room_mark_elevators_used(),
            LeafEffect::QueueEnemyTiles => crate::actors::staging::queue_actor_tiles(self),
            LeafEffect::LoadRoomAssets => crate::actors::staging::prepare_room_assets(self),
            LeafEffect::LoadSuitTargetPalette => {
                crate::samus_palette::load_suit(self, crate::visual_state::PaletteBuffer::Target)
            }
            LeafEffect::UpdateBeamTilesAndPalette => self.prepared_beam_tiles_and_palette(),
            LeafEffect::EnsureSamusDrawn => {
                if self.session().room.entry.elevator_flags == 0 {
                    crate::player_drawing::FrameHost::draw_body(
                        self,
                        crate::scene::samus_body::Body::Frozen,
                    );
                }
            }
            LeafEffect::LoadRoomMusic => self.room_load_music(),
            LeafEffect::UpdateMusicTrack => self.room_update_music_track(),
            LeafEffect::LoadChangedMusic => self.room_load_changed_music(),
            LeafEffect::SelectEnemies => crate::actors::scheduling::select(self),
            LeafEffect::RunEnemies => crate::actors::scheduling::advance_selected(self),
            LeafEffect::DrawScene => crate::actors::drawing::draw_scene(self),
            LeafEffect::InitializeEnemies => crate::actors::population::install_population(self),
            LeafEffect::ClearTransitionSounds => {
                crate::player_effects::command(self, 0x1d);
            }
            LeafEffect::SpinJumpSound => {
                crate::player_effects::command(self, 0x1c);
            }
            _ => self.aliases.room_leaf(self.owner, effect),
        }
        self.refresh_inputs();
    }
    fn camera_call(&mut self, effect: CameraEffect) -> u8 {
        let value = match effect {
            CameraEffect::CalculateLayer2X => crate::room_scrolls::layer2_position(Axis::X, self),
            CameraEffect::CalculateLayer2Y => crate::room_scrolls::layer2_position(Axis::Y, self),
            CameraEffect::CalculateLayer2AndScrolls => {
                crate::room_scrolls::update_scrolls(self);
                0
            }
            CameraEffect::CalculateBgScrolls => {
                crate::room_scrolls::calculate_scrolls(self);
                0
            }
            CameraEffect::DisplayVisibleRoom => {
                crate::room_scrolls::fill_viewport(self);
                0
            }
            CameraEffect::FixDoorMovingUp => {
                crate::room_scrolls::door::fix_up(self);
                0
            }
            CameraEffect::SetupDoorScroll => {
                crate::room_scrolls::door::begin(self);
                0
            }
            CameraEffect::StepDoorScrollRight => crate::room_scrolls::door::horizontal(false, self),
            CameraEffect::StepDoorScrollLeft => crate::room_scrolls::door::horizontal(true, self),
            CameraEffect::StepDoorScrollDown => crate::room_scrolls::door::down(self),
            CameraEffect::StepDoorScrollUp => crate::room_scrolls::door::up(self),
        };
        self.refresh_inputs();
        value
    }
    fn palette_fade(&mut self) -> u8 {
        let value = crate::actors::boss_palette_fade_owned::all(12, self);
        self.refresh_inputs();
        value
    }
    fn wait(&mut self, cause: EntryWait) -> u8 {
        let value = match cause {
            EntryWait::Nmi => self.room_nmi_wait(true),
            EntryWait::NoUpdate => self.room_nmi_wait(false),
            EntryWait::Mechanisms => {
                self.run_mechanisms(crate::room_mechanisms::host::CallerContext::DoorTransition)
            }
            EntryWait::Music => self.room_music_wait(),
        };
        self.refresh_inputs();
        value
    }
    fn sfx_empty(&mut self, queue: SfxQueue) -> bool {
        let index = match queue {
            SfxQueue::One => 0,
            SfxQueue::Two => 1,
            SfxQueue::Three => 2,
        };
        let sound = &self
            .session()
            .control
            .requests()
            .expect("room audio requests")
            .sounds[index];
        (sound.write.wrapping_sub(sound.read) & 0xf) == 0
    }
    fn queued_music(&mut self) -> u8 {
        crate::music_requests::has_queued_music(self)
    }
    fn samus_command(&mut self, command: u16) {
        crate::player_effects::command(self, command);
    }
    fn setup(&mut self, action: SetupAction) {
        let (kind, code) = action.compatibility_code(&self.session().world);
        if kind == SetupKind::Room {
            if crate::room_state::setup::execute(code, self) {
                return;
            }
            if code == 0x91d7 {
                use crate::constants::bank_87::*;
                for animation in [
                    ADDR_K_ANIMTILES_TOURIAN_STATUE_KRAID,
                    ADDR_K_ANIMTILES_TOURIAN_STATUE_PHANTOON,
                    ADDR_K_ANIMTILES_TOURIAN_STATUE_DRAYGON,
                    ADDR_K_ANIMTILES_TOURIAN_STATUE_RIDLEY,
                ] {
                    crate::animated_tiles::execution::spawn(
                        self,
                        u16::try_from(animation).expect("statue animation"),
                    );
                }
                return;
            }
            if code == 0x91bd {
                self.session_mut().feedback.earthquake_kind = 6;
                self.session_mut().feedback.earthquake_timer = u16::MAX;
            }
            if code == 0xc96e {
                crate::room_state::setup::Host::setup_mechanism(self, 0xf, 0x26, 0xba48);
            }
            match code {
                0x91bd | 0x91c9 => {
                    self.room_initialize_land();
                    return;
                }
                0x91ce => {
                    self.room_initialize_ocean();
                    return;
                }
                0xc96e | 0xc976 => {
                    self.room_initialize_haze();
                    return;
                }
                0xc97b => {
                    let display = &mut self
                        .session_mut()
                        .presentation
                        .owner_mut()
                        .expect("Ceres room display")
                        .state;
                    crate::display::preparation::set_display_byte(
                        display,
                        crate::display::preparation::Field::Characters0,
                        102,
                    );
                    self.room_initialize_haze();
                    self.write_current_word_le(0x7eb, 9);
                    return;
                }
                0xc8dd => {
                    for (at, hook) in [(0x601, 0xc8f6u16), (0x604, 0xc8fc)] {
                        self.write_current_slice(at, &hook.to_ne_bytes());
                        self.set_current_byte(at + 2, 0x8f);
                    }
                    return;
                }
                0xc90a => {
                    let at = 0xd908 + usize::from(self.session().room.area);
                    let value = self.current_native_word(at) | 1;
                    self.write_current_slice(at, &value.to_ne_bytes());
                    self.session_mut().menu_screens.save_selection.has_area_map = value;
                    self.refresh_inputs();
                    return;
                }
                _ => {}
            }
        }
        if kind == SetupKind::Door && self.room_door_setup(code) {
            return;
        }
        self.aliases.room_setup_presentation(self.owner, kind, code);
        self.refresh_inputs();
    }
    fn spawn_room_object(&mut self, cursor: &ObjectCursor) {
        use crate::room_mechanisms::host::{Contact, ProgramCursor, SpawnSource};
        let address = match cursor.spawn_object(&self.session().world) {
            SpawnObject::Legacy(address) => address,
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
                0x12
            }
        };
        crate::room_mechanisms::creation::spawn(
            self,
            SpawnSource::Room {
                entry: ProgramCursor(address),
            },
            &mut Contact::Absent,
        );
    }
    fn spawn_initial_hardcoded_object(&mut self) {
        crate::room_mechanisms::creation::spawn(
            self,
            crate::room_mechanisms::host::SpawnSource::Hardcoded {
                x: 8,
                y: 8,
                header: 0xb7eb,
            },
            &mut crate::room_mechanisms::host::Contact::Absent,
        );
    }
    fn close_door(&mut self) {
        crate::room_mechanisms::creation::close_door(self);
    }
    fn write_scroll_byte(&mut self, cell: u8, value: u8) {
        crate::room_scrolls::CameraHost::set_camera_scroll_byte(self, usize::from(cell), value);
    }
    fn write_scroll_pair(&mut self, offset: u16, value: [u8; 2]) {
        crate::room_scrolls::state::store_pair(self, usize::from(offset), value);
    }
    fn device_word(&mut self, word: DeviceWord, value: u16) {
        self.aliases.room_device_word(self.owner, word, value);
        self.refresh_inputs();
    }
    fn device_byte(&mut self, byte: DeviceByte, value: u8) {
        self.aliases.room_device_byte(self.owner, byte, value);
        self.refresh_inputs();
    }
    fn device_vram_address(&mut self, destination: VramDestination) {
        self.aliases.room_vram_address(self.owner, destination);
        self.refresh_inputs();
    }
    fn decompress(&mut self, source: AssetSource, destination: InstallBuffer) {
        self.aliases
            .room_decompress(self.owner, source, destination);
        self.refresh_inputs();
    }
    fn decompress_vram(&mut self, source: AssetSource, destination: VramDecodeTarget) {
        self.aliases
            .room_decompress_vram(self.owner, source, destination);
        self.refresh_inputs();
    }
    fn copy_vram(&mut self, destination: VramDestination, source: AssetSource, bytes: u16) {
        self.aliases
            .room_copy_vram(self.owner, destination, source, bytes);
        self.refresh_inputs();
    }
    fn clear_buffer(&mut self, clear: BufferClear) {
        match clear {
            BufferClear::InitialLevel => {
                for cell in self.session_mut().room.terrain.cells_mut().iter_mut().rev() {
                    cell.level = 0x8000;
                }
            }
            BufferClear::DoorLevel => {
                for index in (0..3200usize).rev() {
                    self.session_mut().room.terrain.cells_mut()[index].level = 0x8000;
                    for offset in [0x1902 + index * 2, 0x3202 + index * 2, 0x4b02 + index * 2] {
                        // Original three native descending stores alias the same live level/BTS planes.
                        let bytes = 0x8000u16.to_ne_bytes();
                        crate::room_terrain::install_byte(self, offset, bytes[0]);
                        crate::room_terrain::install_byte(self, offset + 1, bytes[1]);
                    }
                }
            }
            _ => self.aliases.room_clear_presentation(self.owner, clear),
        }
        self.refresh_inputs();
    }
    fn decode_level_and_copy_planes(&mut self, source: AssetSource) {
        let assets = self.session().world.prepared_room_assets();
        if let Some(assets) = assets {
            let stream = assets
                .stream(&source)
                .expect("selected Level stream unavailable");
            crate::room_terrain::load_prepared(&stream, self);
        } else {
            self.aliases
                .room_decode_imported(self.owner, self.allocations, source);
            self.refresh_inputs();
            crate::room_terrain::copy_level_planes(self);
        }
        self.refresh_inputs();
    }
    fn invalid_stage(&mut self, address: u32) -> u8 {
        let value = self.aliases.room_invalid_stage(self.owner, address);
        self.refresh_inputs();
        value
    }
}
impl crate::room_terrain::PlaneAliases for PreparedActorState<'_> {
    fn imported_level_byte(&self, offset: isize) -> u8 {
        let index = 0x10000usize
            .checked_add_signed(offset)
            .expect("imported level offset");
        self.current_byte(index)
    }
    fn set_imported_level_byte(&mut self, offset: isize, value: u8) {
        let index = 0x10000usize
            .checked_add_signed(offset)
            .expect("imported level offset");
        self.set_current_byte(index, value);
    }
}
impl crate::room_scrolls::CameraHost for PreparedActorState<'_> {
    fn camera_unreachable(&mut self) -> bool {
        self.unknown_function()
    }
    fn publish_camera_position(&mut self, _axis: usize, _value: u16) {
        self.refresh_inputs();
    }
}
impl crate::display_scroll::Access for PreparedActorState<'_> {
    fn scroll_word(&self, word: crate::display_scroll::Word) -> u16 {
        self.session()
            .presentation
            .owner()
            .expect("owned room scroll")
            .state
            .controls
            .scroll_words[usize::from(word)]
    }
    fn set_scroll_word(&mut self, word: crate::display_scroll::Word, value: u16) {
        self.session_mut()
            .presentation
            .owner_mut()
            .expect("owned room scroll")
            .state
            .controls
            .scroll_words[usize::from(word)] = value;
        self.refresh_inputs();
    }
    fn physical_scroll_byte(&self, word: crate::display_scroll::Word, high: bool) -> u8 {
        self.aliases
            .room_physical_scroll_byte(self.owner, word, high)
    }
    fn write_physical_scroll_byte(
        &mut self,
        word: crate::display_scroll::Word,
        high: bool,
        value: u8,
    ) {
        self.aliases
            .store_room_physical_scroll_byte(self.owner, word, high, value);
        self.refresh_inputs();
    }
}
impl crate::room_state::setup::Host for PreparedActorState<'_> {
    fn setup_mechanism(&mut self, x: u8, y: u8, definition: u16) {
        crate::room_mechanisms::creation::spawn(
            self,
            crate::room_mechanisms::host::SpawnSource::Hardcoded {
                x,
                y,
                header: definition,
            },
            &mut crate::room_mechanisms::host::Contact::Absent,
        );
    }
    fn setup_room_projectile(&mut self, definition: u16, parameter: u16) {
        crate::enemy_projectiles::lifecycle::spawn_room(self, definition, parameter);
    }
    fn setup_event(&mut self, event: u16) {
        crate::progress::set_event(self, event);
    }
}
#[derive(Debug)]
pub enum RoomFault {
    BackgroundWindow,
}

impl crate::room_scrolls::state::Aliases for PreparedActorState<'_> {
    fn scroll_alias_byte(&self, index: usize) -> u8 {
        if let Some(byte) = index
            .checked_sub(50)
            .and_then(|at| self.session().save_state.area_maps.get(at))
        {
            *byte
        } else {
            self.aliases.room_scroll_alias_byte(self.owner, index)
        }
    }
    fn store_scroll_alias_byte(&mut self, index: usize, value: u8) {
        if let Some(byte) = index
            .checked_sub(50)
            .and_then(|at| self.session_mut().save_state.area_maps.get_mut(at))
        {
            *byte = value;
        } else {
            self.aliases
                .store_room_scroll_alias_byte(self.owner, index, value);
            self.refresh_inputs();
        }
    }
    fn store_scroll_alias_pair(&mut self, index: usize, value: [u8; 2]) {
        let end = index.checked_add(2).expect("scroll extent");
        // Both bytes are within the scroll and saved area-map owners, so the
        // whole extent is valid before these two publications.
        if end <= 50 + self.session().save_state.area_maps.len() {
            crate::room_scrolls::state::store_byte(self, index, value[0]);
            crate::room_scrolls::state::store_byte(self, index + 1, value[1]);
        } else {
            self.aliases
                .store_room_scroll_alias_pair(self.owner, index, value);
            self.refresh_inputs();
        }
    }
}

impl crate::cartridge::NativeDiagnostics for PreparedActorState<'_> {
    fn native_rom_diagnostic(&mut self, address: u32) {
        self.aliases
            .room_native_diagnostic(self.owner, self.allocations, address);
        self.refresh_inputs();
    }
}

impl crate::room_state::fx::FxHost for PreparedActorState<'_> {
    fn initialize_fx(&mut self, initializer: crate::room_state::fx::Initializer) {
        self.room_initialize_fx(initializer);
        self.refresh_inputs();
    }
    fn spawn_fx_palette(&mut self, definition: u16) {
        crate::palette_effects::execution::spawn(self, definition);
        self.refresh_inputs();
    }
    fn spawn_fx_animated_tiles(&mut self, definition: u16) {
        crate::animated_tiles::execution::spawn(self, definition);
        self.refresh_inputs();
    }
}

impl crate::room_scrolls::Host for PreparedActorState<'_> {
    fn unreachable(&mut self) -> bool {
        self.unknown_function()
    }
    fn camera_screen_base(&self, second: bool) -> u8 {
        self.session()
            .presentation
            .owner()
            .expect("room screen base")
            .state
            .controls
            .screens[usize::from(second)]
    }
    fn store_camera_background_size(&mut self, value: u16) {
        self.aliases.room_store_background_size(self.owner, value);
        self.refresh_inputs();
    }
    fn queue_scrolling_tiles(
        &mut self,
        offset: u16,
        layout: crate::display::transfer::ScrollingLayout,
    ) {
        self.aliases
            .room_queue_scrolling_tiles(self.owner, offset, layout);
        self.refresh_inputs();
    }
    fn flush_tilemaps(&mut self) {
        self.aliases.room_flush_tilemaps(self.owner);
        self.refresh_inputs();
    }
}

impl PreparedActorState<'_> {
    fn room_nmi_wait(&mut self, update: bool) -> u8 {
        let value = self
            .session_mut()
            .presentation
            .owner_mut()
            .expect("room NMI wait")
            .state
            .nmi
            .wait(update);
        self.refresh_inputs();
        value
    }
    fn room_music_wait(&mut self) -> u8 {
        let resume = self
            .session()
            .control
            .metadata()
            .expect("room music continuation")
            .continuation
            .music;
        if resume == 0 {
            self.aliases.room_leaf(self.owner, LeafEffect::EnableNmi);
            self.refresh_inputs();
            self.session_mut().messages.delay = 0;
        }
        if resume <= 1 {
            let mut waiting = resume == 1;
            loop {
                if !waiting {
                    if self.session().messages.delay == 20 {
                        self.aliases.room_leaf(self.owner, LeafEffect::DisableNmi);
                        self.refresh_inputs();
                        break;
                    }
                    crate::music_requests::advance_music(self, |effect, meaning, host| {
                        let result = host.aliases.room_music_effect(host.owner, effect, meaning);
                        host.refresh_inputs();
                        result
                    })
                    .expect("room music advance");
                }
                let yielded = self.room_nmi_wait(true);
                if yielded != 0 {
                    self.session_mut()
                        .control
                        .metadata_mut()
                        .expect("room music continuation")
                        .continuation
                        .music = 1;
                    return yielded;
                }
                let delay = &mut self.session_mut().messages.delay;
                *delay = delay.wrapping_add(1);
                waiting = false;
            }
        }
        self.session_mut()
            .control
            .metadata_mut()
            .expect("room music continuation")
            .continuation
            .music = 0;
        0
    }
    fn room_track_changed(&self) -> bool {
        let state = self.session();
        let room = &state.room.entry;
        let music = &state.control.requests().expect("room music requests").music;
        state.game_loop.mode < 40
            && room.music_track != 0
            && ((room.music_data << 8) | room.music_track)
                != ((music.data_index << 8) | music.track_index)
    }
    fn room_load_music(&mut self) {
        let state = self.session();
        let data = state.room.entry.music_data;
        if state.game_loop.mode < 40
            && data != 0
            && data
                != state
                    .control
                    .requests()
                    .expect("room music requests")
                    .music
                    .data_index
        {
            crate::music_requests::queue_delayed_8(0, self);
            // Queue effects are ordered; take the second room data selection freshly.
            let data = self.session().room.entry.music_data;
            crate::music_requests::queue_delayed_8(data | 0xff00, self);
        }
    }
    fn room_update_music_track(&mut self) {
        if self.room_track_changed() {
            let track = self.session().room.entry.music_track;
            crate::session_owners::AudioHost::audio_store_word(self, 0x7f5, track);
        }
    }
    fn room_load_changed_music(&mut self) {
        if self.room_track_changed() {
            crate::music_requests::queue_delayed(self.session().room.entry.music_track, 6, self);
        }
    }
}
