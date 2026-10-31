//! Live legacy/prepared content acquisition through the shared Session interval.
//! Captured values retain allocations; diagnostics precede replacement captures.
use super::{
    StartSite,
    content::*,
    effects::{AssetRead, AssetSlot},
    world::{ExitId, Identity, RoomId, StateCondition, StateId},
};
use crate::{
    game::session_runtime::{Runtime, invalid_window, native_rom_diagnostic},
    rom::RomRecord,
};
use core::mem::offset_of;

impl Runtime<'_> {
    /// Normal room_record policy: loaded image, native words, room observer.
    fn room_capture<P>(&mut self, bank: u8, address: u16) -> Capture<P> {
        let record = if address & 0x8000 == 0 {
            // Retain the real rom_record boundary, including its cfg observers.
            self.foreign(|game| unsafe { crate::rtl::rom_record(bank, address, game) })
        } else {
            RomRecord {
                image: self.loaded_image().expect("ROM must be loaded"),
                base: crate::rtl::lorom_offset((u32::from(bank) << 16) | u32::from(address))
                    as usize,
            }
        };
        Capture::Legacy {
            image: record.image,
            base: record.base,
            access: RecordAccess::Native,
            world: self.session().world.clone(),
        }
    }
    /// RomPtr policy used by load stations and their final header acquisition.
    fn native_room_capture<P>(&mut self, bank: u8, address: u16) -> Capture<P> {
        self.room_pointer_diagnostic(bank, address);
        Capture::Legacy {
            image: self.native_image(),
            base: crate::rtl::lorom_offset((u32::from(bank) << 16) | u32::from(address)) as usize,
            access: RecordAccess::Native,
            world: self.session().world.clone(),
        }
    }
    fn room_pointer_diagnostic(&mut self, bank: u8, address: u16) {
        if address & 0x8000 == 0 {
            self.foreign(|game| {
                // Keep the actual RomPtr observer and native binding acquisition.
                native_rom_diagnostic((u32::from(bank) << 16) | u32::from(address), game);
                let _ = game.legacy_native_rom_ptr();
            });
        }
    }
    /// PLM loaded_record policy has no room_record or RomPtr observer hook.
    fn closing_capture<P>(&mut self, bank: u8, address: u16) -> Capture<P> {
        let absolute = (u32::from(bank) << 16) | u32::from(address);
        if address & 0x8000 == 0 {
            self.foreign(|game| invalid_window(game, absolute));
        }
        Capture::Legacy {
            image: self.loaded_image().expect("ROM must be loaded"),
            base: crate::rtl::lorom_offset(absolute) as usize,
            access: RecordAccess::Native,
            world: self.session().world.clone(),
        }
    }
    fn background_capture<P>(&mut self, address: u16, mode: LoadMode) -> Capture<P> {
        let absolute = 0x8f0000 | u32::from(address);
        if address & 0x8000 == 0 {
            match mode {
                LoadMode::Initial => self.foreign(|game| {
                    invalid_window(game, absolute);
                    #[cfg(sm_verify_dll)]
                    crate::room_background_observer::diagnostic(absolute, game);
                }),
                LoadMode::Door => panic!("ROM address must select a LoROM window"),
            }
        }
        Capture::Legacy {
            image: self.loaded_image().expect("ROM must be loaded"),
            base: crate::rtl::lorom_offset(absolute) as usize,
            access: match mode {
                LoadMode::Initial => RecordAccess::Native,
                LoadMode::Door => RecordAccess::LittleEndian,
            },
            world: self.session().world.clone(),
        }
    }
    fn contact_word(&self, bank: u8, address: u16) -> u16 {
        let image = self.loaded_image().expect("ROM must be loaded");
        let at = crate::rtl::lorom_offset((u32::from(bank) << 16) | 0x8000 | u32::from(address))
            as usize;
        assert!(at + 2 <= image.len(), "ROM read outside loaded allocation");
        u16::from_le_bytes([image.read(at), image.read(at + 1)])
    }
}
impl EntryContent for Runtime<'_> {
    fn contact_bts(&mut self) -> u8 {
        let block = self.session().room.collision_cursor;
        self.byte(0x16402 + usize::from(block))
    }
    fn contact_exit(&mut self, list: &DoorList, bts: u8) -> Identity<ExitId> {
        match list {
            DoorList::Prepared(id) => Identity::Known(
                self.session()
                    .world
                    .definition()
                    .expect("prepared door list")
                    .door_lists[id][usize::from(bts & 0x7f)],
            ),
            DoorList::Legacy(list) => {
                let address = list.wrapping_add(u16::from(bts & 0x7f) * 2);
                self.room_pointer_diagnostic(0x8f, address);
                let key = self.contact_word(0x8f, address);
                self.session().world.exit_identity(key)
            }
        }
    }
    fn contact_has_destination(&mut self, exit: &Identity<ExitId>) -> bool {
        match exit {
            Identity::Known(id) => {
                let world = self.session().world.definition().expect("prepared exit");
                let _ = &world.rooms[&world.exits[id].destination];
                true
            }
            Identity::LegacyUnresolved(key) => {
                self.room_pointer_diagnostic(0x83, *key);
                self.contact_word(0x83, *key) & 0x8000 != 0
            }
        }
    }
    fn capture_station(&mut self, site: StartSite) -> StationCapture {
        let world = self.session().world.clone();
        match site {
            StartSite::Authored(position) => Capture::Prepared { world, position },
            StartSite::CurrentLoadStation if !world.is_canonical_legacy() => {
                let position = world.start_at(self.session().room.area, self.word(0x78b));
                Capture::Prepared { world, position }
            }
            StartSite::CurrentLoadStation => {
                // Table allocation is captured before evaluating its area index.
                let image = self.loaded_image().expect("ROM must be loaded");
                let base = image.table_word(
                    crate::rtl::lorom_offset(0x80c4b5) as usize,
                    self.session().room.area as isize,
                );
                let address = (i32::from(base) + 14 * i32::from(self.word(0x78b))) as u16;
                self.native_room_capture(0x80, address)
            }
        }
    }
    fn capture_header(&mut self, room: &Identity<RoomId>) -> HeaderCapture {
        match room {
            Identity::Known(position) => Capture::Prepared {
                world: self.session().world.clone(),
                position: *position,
            },
            Identity::LegacyUnresolved(key) => self.room_capture(0x8f, *key),
        }
    }
    fn capture_start_header(&mut self, room: &Identity<RoomId>) -> HeaderCapture {
        match room {
            Identity::Known(position) => Capture::Prepared {
                world: self.session().world.clone(),
                position: *position,
            },
            Identity::LegacyUnresolved(key) => self.native_room_capture(0x8f, *key),
        }
    }
    fn selector(&mut self, cursor: &SelectorCursor) -> (SelectorKind, SelectorCursor) {
        match cursor {
            SelectorCursor::Legacy(address) => {
                let token: OperandCapture = self.room_capture(0x8f, *address);
                let opcode = 0x8f0000 | u32::from(token.raw_word(0));
                let kind = SelectorKind::from_legacy(opcode);
                (kind, SelectorCursor::Legacy(address.wrapping_add(2)))
            }
            SelectorCursor::Prepared(position) => {
                let world = self
                    .session()
                    .world
                    .definition()
                    .expect("prepared selector");
                let kind = match world.rooms[&position.room].selectors[position.index].condition {
                    StateCondition::Default => SelectorKind::Default,
                    StateCondition::Event(_) => SelectorKind::Event,
                    StateCondition::Boss(_) => SelectorKind::Boss,
                    StateCondition::BossOne => SelectorKind::BossOne,
                    StateCondition::MorphAndMissiles => SelectorKind::MorphAndMissiles,
                    StateCondition::PowerBombs => SelectorKind::PowerBombs,
                };
                (kind, cursor.clone())
            }
        }
    }
    fn capture_operand(&mut self, cursor: &SelectorCursor) -> OperandCapture {
        match cursor {
            SelectorCursor::Legacy(address) => self.room_capture(0x8f, *address),
            SelectorCursor::Prepared(position) => Capture::Prepared {
                world: self.session().world.clone(),
                position: *position,
            },
        }
    }
    fn direct_target(&mut self, cursor: &SelectorCursor) -> Identity<StateId> {
        match cursor {
            SelectorCursor::Legacy(address) => {
                let token: OperandCapture = self.room_capture(0x8f, *address);
                self.session().world.state_identity(token.raw_word(0))
            }
            SelectorCursor::Prepared(position) => Identity::Known(
                self.session()
                    .world
                    .definition()
                    .expect("prepared selector")
                    .rooms[&position.room]
                    .selectors[position.index]
                    .state,
            ),
        }
    }
    fn invalid_selector(&mut self, _address: u32) -> Option<SelectorCursor> {
        let value = self.foreign(|game| crate::rtl::report_unreachable(game)) as u16;
        (value != 0).then_some(SelectorCursor::Legacy(value))
    }
    fn capture_state(&mut self, state: &Identity<StateId>) -> StateCapture {
        match state {
            Identity::Known(position) => Capture::Prepared {
                world: self.session().world.clone(),
                position: *position,
            },
            Identity::LegacyUnresolved(key) => self.room_capture(0x8f, *key),
        }
    }
    fn capture_tileset(&mut self, selection: TilesetSelection) -> TilesetCapture {
        match selection {
            TilesetSelection::Prepared(position) => Capture::Prepared {
                world: self.session().world.clone(),
                position,
            },
            TilesetSelection::Legacy(index) => {
                let image = self.loaded_image().expect("ROM must be loaded");
                let at = crate::rtl::lorom_offset(0x8fe7a7) as usize + usize::from(index) * 2;
                assert!(at + 2 <= image.len(), "ROM read outside loaded allocation");
                let address = u16::from_le_bytes([image.read(at), image.read(at + 1)]);
                self.room_capture(0x8f, address)
            }
        }
    }
    fn capture_door(&mut self, exit: &Identity<ExitId>) -> DoorCapture {
        match exit {
            Identity::Known(position) => Capture::Prepared {
                world: self.session().world.clone(),
                position: *position,
            },
            Identity::LegacyUnresolved(key) => self.room_capture(0x83, *key),
        }
    }
    fn scroll_pair(&mut self, cursor: &ScrollCursor) -> [u8; 2] {
        match cursor {
            ScrollCursor::Legacy(address) => {
                let record: StateCapture = self.room_capture(0x8f, *address);
                record.raw_word(0).to_ne_bytes()
            }
            ScrollCursor::Prepared { state, index } => {
                let world = self.session().world.definition().expect("prepared scrolls");
                match &world.states[state].scrolls {
                    super::world::ScrollDefinition::Cells(bytes) => {
                        [bytes[*index], bytes[*index + 1]]
                    }
                    _ => panic!("record cursor for generated scrolls"),
                }
            }
        }
    }
    fn object_exists(&mut self, cursor: &ObjectCursor) -> bool {
        match cursor {
            ObjectCursor::LegacyInitial(address) => {
                let record: StateCapture = self.room_capture(0x8f, *address);
                record.raw_word(0) != 0
            }
            ObjectCursor::LegacyDoor(address) => {
                let record: StateCapture = self.room_capture(0x8f, *address as u16);
                record.raw_word(0) != 0
            }
            ObjectCursor::Prepared { state, index } => {
                *index
                    < self
                        .session()
                        .world
                        .definition()
                        .expect("prepared objects")
                        .states[state]
                        .objects
                        .len()
            }
        }
    }
    fn selected_background(
        &mut self,
        state: &Identity<StateId>,
        mode: LoadMode,
    ) -> Option<BackgroundCursor> {
        match state {
            Identity::Known(state) => {
                let world = self
                    .session()
                    .world
                    .definition()
                    .expect("prepared background");
                (!world.states[state].background.is_empty()).then_some(BackgroundCursor::Prepared(
                    BackgroundPosition {
                        state: *state,
                        index: 0,
                        part: BackgroundPart::Opcode,
                    },
                ))
            }
            Identity::LegacyUnresolved(address) => {
                let record: StateCapture = self.background_capture(*address, mode);
                let cursor =
                    record.background_word(offset_of!(crate::types::RoomDefRoomstate, bg_data_ptr));
                (cursor & 0x8000 != 0).then_some(BackgroundCursor::Legacy(cursor))
            }
        }
    }
    fn capture_background(
        &mut self,
        cursor: &BackgroundCursor,
        mode: LoadMode,
    ) -> BackgroundCapture {
        match cursor {
            BackgroundCursor::Legacy(address) => self.background_capture(*address, mode),
            BackgroundCursor::Prepared(position) => Capture::Prepared {
                world: self.session().world.clone(),
                position: *position,
            },
        }
    }
    fn live_asset(&self, slot: AssetSlot, read: AssetRead) -> AssetSource {
        let base = match slot {
            AssetSlot::Level => 0x7bd,
            AssetSlot::TileTable => 0x7c0,
            AssetSlot::Tiles => 0x7c3,
            AssetSlot::Palette => 0x7c6,
        };
        let bytes = match read {
            AssetRead::ThreeLittleEndian => {
                [self.byte(base), self.byte(base + 1), self.byte(base + 2)]
            }
            AssetRead::FourNativeMasked => {
                let value =
                    u32::from_ne_bytes(std::array::from_fn(|i| self.byte(base + i))) & 0xffffff;
                let bytes = value.to_le_bytes();
                [bytes[0], bytes[1], bytes[2]]
            }
        };
        AssetSource::Bound(self.session().world.asset(slot, bytes))
    }
    fn capture_closing_door(&mut self, exit: &Identity<ExitId>) -> ClosingDoorCapture {
        match exit {
            Identity::Known(position) => Capture::Prepared {
                world: self.session().world.clone(),
                position: *position,
            },
            Identity::LegacyUnresolved(key) => self.closing_capture(0x83, *key),
        }
    }
    fn capture_closing_header(&mut self, direction: u16) -> ClosingHeaderCapture {
        if self.session().world.is_canonical_legacy() {
            let address = direction.wrapping_mul(2).wrapping_add(0xe68a);
            self.closing_capture(0x8f, address)
        } else {
            Capture::Prepared {
                world: self.session().world.clone(),
                position: ClosingHeaderPosition { direction },
            }
        }
    }
}
