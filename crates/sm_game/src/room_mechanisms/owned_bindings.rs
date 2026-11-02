//! Mechanisms resolve the current prepared aggregate, including exceptional aliases.
#![forbid(unsafe_code)]
use super::PreparedActorState;
use crate::actors::content_records::ContentSource;
use crate::room_mechanisms::host::*;
use crate::session_access::SessionAccess;

fn slot_address(member: SlotMember, slot: isize) -> usize {
    let base: usize = match member {
        SlotMember::Header => 0x1c37,
        SlotMember::Block => 0x1c87,
        SlotMember::PreAction => 0x1cd7,
        SlotMember::Program => 0x1d27,
        SlotMember::Counter => 0x1d77,
        SlotMember::RoomArgument => 0x1dc7,
        SlotMember::Work0 => 0x1e17,
        SlotMember::InstructionDelay => 0xde1c,
        SlotMember::DrawProgram => 0xde6c,
        SlotMember::ReturnProgram => 0xdebc,
        SlotMember::Work1 => 0xdf0c,
    };
    base.checked_add_signed(slot.checked_mul(2).expect("mechanism slot extent"))
        .expect("mechanism slot address")
}
impl PreparedActorState<'_> {
    fn mechanism_native_word(&self, at: usize) -> u16 {
        u16::from_ne_bytes([self.current_byte(at), self.current_byte(at + 1)])
    }
    fn mechanism_store(&mut self, at: usize, value: u16, order: StoreOrder) {
        match order {
            StoreOrder::PrefixLittle => {
                let [low, high] = value.to_le_bytes();
                self.set_current_byte(at, low);
                self.set_current_byte(at + 1, high);
            }
            StoreOrder::SliceLittle => self.write_current_slice(at, &value.to_le_bytes()),
            StoreOrder::SliceNative => self.write_current_slice(at, &value.to_ne_bytes()),
        }
    }
    fn mechanism_fault(&mut self, diagnostic: Diagnostic) -> u8 {
        self.aliases.mechanism_diagnostic(self.owner, diagnostic)
    }
}
impl ImportedAliases for PreparedActorState<'_> {
    fn mechanism_word(&self, member: SlotMember, slot: isize) -> u16 {
        self.current_word(slot_address(member, slot))
    }
    fn mechanism_byte(&self, member: SlotMember, slot: isize, lane: usize) -> u8 {
        self.current_byte(
            slot_address(member, slot)
                .checked_add(lane)
                .expect("mechanism byte extent"),
        )
    }
    fn set_mechanism_word(
        &mut self,
        member: SlotMember,
        slot: isize,
        value: u16,
        order: StoreOrder,
    ) {
        self.mechanism_store(slot_address(member, slot), value, order);
    }
    fn set_mechanism_byte(&mut self, member: SlotMember, slot: isize, lane: usize, value: u8) {
        self.set_current_byte(
            slot_address(member, slot)
                .checked_add(lane)
                .expect("mechanism byte extent"),
            value,
        );
    }
    fn projectile_kind(&self, slot: isize, order: ByteOrder) -> u16 {
        let at = 0xc18usize
            .checked_add_signed(slot.checked_mul(2).expect("projectile kind extent"))
            .expect("projectile kind address");
        match order {
            ByteOrder::Little => self.current_word(at),
            ByteOrder::Native => self.mechanism_native_word(at),
        }
    }
    fn set_saved_graphics_word(&mut self, index: usize, value: u16, order: StoreOrder) {
        self.mechanism_store(0x1c2f + index * 2, value, order);
    }
    fn terrain_word(&self, block: usize, order: ByteOrder) -> u16 {
        let at = 0x10002usize.checked_add(block).expect("terrain extent");
        match order {
            ByteOrder::Little => self.current_word(at),
            ByteOrder::Native => self.mechanism_native_word(at),
        }
    }
    fn set_terrain_word(&mut self, block: usize, value: u16, order: ByteOrder) {
        let at = 0x10002usize.checked_add(block).expect("terrain extent");
        self.mechanism_store(
            at,
            value,
            match order {
                ByteOrder::Little => StoreOrder::SliceLittle,
                ByteOrder::Native => StoreOrder::SliceNative,
            },
        );
    }
    fn terrain_high(&self, block: u16) -> u8 {
        self.current_byte(0x10003 + usize::from(block))
    }
    fn set_terrain_high(&mut self, block: u16, value: u8) {
        self.set_current_byte(0x10003 + usize::from(block), value);
    }
    fn behavior_byte(&self, index: usize) -> u8 {
        self.current_byte(0x16402usize.checked_add(index).expect("behavior extent"))
    }
    fn set_behavior_byte(&mut self, index: usize, value: u8) {
        self.set_current_byte(
            0x16402usize.checked_add(index).expect("behavior extent"),
            value,
        );
    }
    fn set_behavior_pair(&mut self, index: usize, value: u16) {
        self.mechanism_store(
            0x16402usize.checked_add(index).expect("behavior extent"),
            value,
            StoreOrder::SliceNative,
        );
    }
    fn contact_displacement(&self, token: &ContactIdentity) -> i32 {
        i32::from_ne_bytes(core::array::from_fn(|lane| {
            self.current_byte(token.0 + lane)
        }))
    }
    fn set_contact_displacement(&mut self, token: &ContactIdentity, value: i32) {
        self.write_current_slice(token.0, &value.to_ne_bytes());
    }
    fn scroll_pair(&mut self, index: usize, value: u16) {
        crate::room_scrolls::state::store_pair(self, index, value.to_ne_bytes());
    }
    fn set_scroll(&mut self, index: usize, value: u8) {
        crate::room_scrolls::state::store_byte(self, index, value);
    }
    fn signal_turret(&mut self, association: u16) {
        self.mechanism_store(usize::from(association), 1, StoreOrder::SliceNative);
    }
}

pub enum TerrainProgram {
    Loaded(sm_data::cartridge::RomRecord),
    Dynamic(usize),
}
impl Content for PreparedActorState<'_> {
    type Record = sm_data::cartridge::RomRecord;
    type TerrainProgram = TerrainProgram;
    fn loaded_record(&mut self, at: ContentAddress) -> Self::Record {
        self.content_record(at.bank, at.offset)
    }
    fn native_record(&mut self, at: ContentAddress) -> Self::Record {
        self.native_actor_record(at.bank, at.offset)
    }
    fn definition_word(&mut self, at: ContentAddress, offset: usize) -> u16 {
        // This path uses the loaded allocation and checks the complete extent
        // before reading either byte. It does not run a low-window callback.
        assert!(
            at.offset & 0x8000 != 0,
            "ROM address must select a LoROM window"
        );
        let image = self.content_image();
        let base = usize::try_from(sm_data::cartridge::lorom_offset(
            (u32::from(at.bank) << 16) | u32::from(at.offset),
        ))
        .expect("definition base");
        let start = base.checked_add(offset).expect("ROM offset overflow");
        let end = start.checked_add(2).expect("ROM offset overflow");
        assert!(end <= image.len(), "ROM read outside loaded allocation");
        u16::from_le_bytes([image.read(start), image.read(start + 1)])
    }
    fn table_word(&self, table: ContentTable, index: isize) -> u16 {
        let (address, offset, little) = match table {
            ContentTable::Gate { up, byte_offset } => {
                (if up { 0x84c764 } else { 0x84c70a }, byte_offset, false)
            }
            ContentTable::MetroidWatcher => (0x84db28, 0, false),
            ContentTable::ItemGraphicDestination => (0x8487cd, 0, true),
            ContentTable::ItemTileOffset => (0x8487d5, 0, true),
            ContentTable::ItemTileNumber => (0x8487dd, 0, true),
            ContentTable::ItemReveal => (0x84839d, 0, false),
        };
        let base = usize::try_from(sm_data::cartridge::lorom_offset(address))
            .expect("mechanism table")
            + offset;
        let image = self.content_image();
        if !little {
            return image.table_word(base, index);
        }
        let at = base
            .checked_add_signed(index.checked_mul(2).expect("ROM table offset overflow"))
            .expect("ROM table offset overflow");
        u16::from_le_bytes([image.read(at), image.read(at + 1)])
    }
    fn capture_terrain_program(&mut self, id: DrawProgramId) -> Self::TerrainProgram {
        if id.0 & 0x8000 != 0 {
            TerrainProgram::Loaded(sm_data::cartridge::RomRecord {
                image: self.content_image(),
                base: usize::try_from(sm_data::cartridge::lorom_offset(0x840000 | u32::from(id.0)))
                    .expect("terrain program offset"),
            })
        } else {
            if id.0 >= 0x2000 {
                self.mechanism_fault(Diagnostic::InvalidRamDraw(id.0));
            }
            TerrainProgram::Dynamic(usize::from(id.0))
        }
    }
    fn terrain_program_word(&self, program: &Self::TerrainProgram, offset: usize) -> u16 {
        match program {
            TerrainProgram::Loaded(record) => InstructionRecord::word(
                record,
                isize::try_from(offset).expect("terrain program offset"),
            ),
            TerrainProgram::Dynamic(base) => self.mechanism_native_word(
                base.checked_add(offset)
                    .expect("mechanism terrain cursor overflow"),
            ),
        }
    }
    fn drawing_word(&mut self, cursor: DrawCursor) -> u16 {
        if cursor.0 & 0x8000 != 0 {
            InstructionRecord::word(&self.native_actor_record(0x84, cursor.0), 0)
        } else {
            if cursor.0 >= 0x2000 {
                self.mechanism_fault(Diagnostic::InvalidRamDraw(cursor.0));
            }
            self.mechanism_native_word(usize::from(cursor.0))
        }
    }
    fn drawing_byte(&mut self, cursor: DrawCursor) -> u8 {
        if cursor.0 & 0x8000 != 0 {
            InstructionRecord::byte(&self.native_actor_record(0x84, cursor.0), 0)
        } else {
            if cursor.0 >= 0x2000 {
                self.mechanism_fault(Diagnostic::InvalidRamDraw(cursor.0));
            }
            self.current_byte(usize::from(cursor.0))
        }
    }
}

fn flag_address(column: crate::progress::FlagColumn, index: isize) -> usize {
    use crate::progress::FlagColumn;
    let base: usize = match column {
        FlagColumn::Events => 0xd820,
        FlagColumn::Bosses => 0xd828,
        FlagColumn::RoomObjects => 0xd830,
        FlagColumn::Items => 0xd870,
        FlagColumn::Doors => 0xd8b0,
    };
    base.checked_add_signed(index).expect("flag alias extent")
}
impl crate::progress::FlagAliases for PreparedActorState<'_> {
    fn flag_byte(&self, column: crate::progress::FlagColumn, index: isize) -> u8 {
        self.current_byte(flag_address(column, index))
    }
    fn set_flag_byte(&mut self, column: crate::progress::FlagColumn, index: isize, value: u8) {
        self.set_current_byte(flag_address(column, index), value);
    }
    fn door_adjacent_word_le(&self, index: isize) -> u16 {
        self.current_word(flag_address(crate::progress::FlagColumn::Doors, index))
    }
    fn boss_word(&self, area: u16, order: crate::progress::BossWordOrder) -> u16 {
        let at = 0xd828 + usize::from(area);
        match order {
            crate::progress::BossWordOrder::Little => self.current_word(at),
            crate::progress::BossWordOrder::Native => self.mechanism_native_word(at),
        }
    }
    fn set_boss_word_le(&mut self, area: u16, value: u16) {
        self.write_current_slice(0xd828 + usize::from(area), &value.to_le_bytes());
    }
    fn set_boss_word_native(&mut self, area: u16, value: u16) {
        // The native original checks the full slice before either store.
        self.mechanism_store(0xd828 + usize::from(area), value, StoreOrder::SliceNative);
    }
}
impl crate::progress::FlagHost for PreparedActorState<'_> {
    fn flags(&self) -> &crate::gameplay_flag_state::GameplayFlags {
        &self.session().gameplay_flags
    }
    fn flags_mut(&mut self) -> &mut crate::gameplay_flag_state::GameplayFlags {
        &mut self.session_mut().gameplay_flags
    }
    fn compatibility(&self) -> &crate::gameplay_flag_state::FlagCompatibility {
        &self.session().flag_compatibility
    }
    fn compatibility_mut(&mut self) -> &mut crate::gameplay_flag_state::FlagCompatibility {
        &mut self.session_mut().flag_compatibility
    }
    fn area(&self) -> u16 {
        self.session().room.area
    }
}
impl crate::gameplay_flag_host::Host for PreparedActorState<'_> {}

impl PreparedActorState<'_> {
    pub(crate) fn mechanism_sound(&mut self, channel: SoundChannel, maximum: u8, cue: u16) {
        assert!(
            matches!(maximum, 1 | 3 | 6 | 9 | 15),
            "mechanism sound limit"
        );
        let channel = match channel {
            SoundChannel::One => 0,
            SoundChannel::Two => 1,
            SoundChannel::Three => 2,
        };
        crate::music_requests::queue_sfx(channel, cue.to_le_bytes()[0], maximum, self)
            .expect("audio sound enqueue");
    }
    pub(crate) fn mechanism_music(&mut self, track: u16) {
        crate::music_requests::queue_delayed_8(track, self);
    }
    pub(crate) fn mechanism_clear_music_queue(&mut self) {
        crate::music_requests::clear_queue(self).expect("audio queue clear");
    }
    pub(crate) fn mechanism_delayed_room_music(&mut self, frames: u16) {
        if self.current_word(0x998) < 40 {
            crate::music_requests::queue_delayed(0, frames, self);
            // Fresh selection occurs after the first enqueue and publication.
            let track = self.current_word(0x7f5);
            crate::music_requests::queue_delayed_8(track, self);
        }
    }
}

impl PreparedActorState<'_> {
    pub(crate) fn mechanism_map_used(&self) -> bool {
        self.current_byte(0xd908 + usize::from(self.session().room.area)) != 0
    }
    pub(crate) fn mechanism_mark_map_used(&mut self) {
        self.set_current_byte(0xd908 + usize::from(self.session().room.area), 255);
    }
    pub(crate) fn mechanism_set_has_area_map(&mut self, value: u16) {
        self.session_mut().menu_screens.save_selection.has_area_map = value;
    }
    pub(crate) fn mechanism_mark_save_station(&mut self, station: u16) {
        self.write_current_slice(0x78b, &station.to_le_bytes());
        let station = self.current_word(0x78b);
        crate::progress::prepare_flag_mask(&mut self.session_mut().flag_compatibility, station);
        let at = 0xd8f8 + usize::from(self.session().room.area) * 2;
        let value =
            self.current_byte(at) | self.session().flag_compatibility.bitmask.to_le_bytes()[0];
        self.set_current_byte(at, value);
    }
    pub(crate) fn mechanism_selected_save_slot(&self) -> u16 {
        self.session().menu_screens.save_selection.selected_slot
    }
    pub(crate) fn mechanism_statue_signal(&mut self) {
        self.mechanism_store(0xfb4, 1, StoreOrder::SliceNative);
    }
    pub(crate) fn mechanism_statue_wait_actor(&mut self, actor: u16, waiting: bool) {
        let actor = 0xf78 + usize::from(actor);
        let old = self.mechanism_native_word(actor + 0x0e);
        self.mechanism_store(
            actor + 0x0e,
            if waiting { old | 0x400 } else { old & !0x400 },
            StoreOrder::SliceNative,
        );
        if !waiting {
            let instruction = self.mechanism_native_word(actor + 0x1a).wrapping_add(2);
            self.mechanism_store(actor + 0x1a, instruction, StoreOrder::SliceNative);
            self.mechanism_store(actor + 0x1c, 1, StoreOrder::SliceNative);
        }
    }
    pub(crate) fn mechanism_wake_projectile_for_block(&mut self, block: u16) {
        if let Some(slot) = self
            .session_mut()
            .enemy_projectiles
            .slots
            .iter_mut()
            .rev()
            .find(|slot| slot.behavior_a == block)
        {
            slot.delay = 1;
            slot.program = slot.program.wrapping_add(2);
            return;
        }
        // Search failure retains the original signed -1 indexed aliases.
        self.write_current_slice(0x1b8d, &1u16.to_le_bytes());
        let value = self.current_word(0x1b45).wrapping_add(2);
        self.write_current_slice(0x1b45, &value.to_le_bytes());
    }
}

impl PreparedActorState<'_> {
    fn mechanism_queue_word(&mut self, at: usize, value: u16, order: ByteOrder) {
        let bytes = match order {
            ByteOrder::Little => value.to_le_bytes(),
            ByteOrder::Native => value.to_ne_bytes(),
        };
        self.mechanism_queue_slice(at, &bytes);
    }
    fn mechanism_queue_pair(&mut self, destination: u16, tail: u16, count: u16, mode: u16) {
        let entry = 0xd0 + usize::from(tail);
        self.mechanism_queue_word(entry + 5, mode | destination, ByteOrder::Native);
        let size = 4u16.wrapping_mul(count);
        let source = self.session().mechanisms.draw_cursor.wrapping_add(0xc6c8);
        self.mechanism_queue_word(entry, size, ByteOrder::Native);
        self.mechanism_queue_word(entry + 7, size, ByteOrder::Native);
        self.mechanism_queue_word(entry + 2, source, ByteOrder::Native);
        self.mechanism_queue_slice(entry + 4, &[126]);
        self.mechanism_queue_word(entry + 9, source.wrapping_add(size), ByteOrder::Native);
        self.mechanism_queue_slice(entry + 11, &[126]);
    }
}
impl Graphics for PreparedActorState<'_> {
    fn queue_graphics_transfer<R: InstructionRecord>(
        &mut self,
        record: &R,
        kind: GraphicsTransferKind,
    ) {
        let tail = self.current_word(0x330);
        let entry = 0xd0 + usize::from(self.current_word(0x330));
        match kind {
            GraphicsTransferKind::Item { destination } => {
                self.mechanism_queue_word(entry, 256, ByteOrder::Little);
                let source = record.word(0);
                self.mechanism_queue_word(entry + 2, source, ByteOrder::Little);
                self.mechanism_queue_slice(entry + 4, &[0x89]);
                self.mechanism_queue_word(entry + 5, destination.0, ByteOrder::Little);
            }
            GraphicsTransferKind::CopyScript => {
                let count = record.word(0);
                self.mechanism_queue_word(entry, count, ByteOrder::Little);
                let source = record.word(2);
                self.mechanism_queue_word(entry + 2, source, ByteOrder::Little);
                let bank = record.byte(4);
                self.mechanism_queue_slice(entry + 4, &[bank]);
                let destination = record.word(5);
                self.mechanism_queue_word(entry + 5, destination, ByteOrder::Little);
            }
        }
        self.mechanism_store(0x330, tail.wrapping_add(7), StoreOrder::SliceLittle);
    }
    fn queue_draw_transfers(
        &mut self,
        destination: TilemapDestination,
        count: u16,
        layout: DrawLayout,
    ) -> Option<DrawTargets> {
        let tail = self.current_word(0x330);
        let limit = if matches!(layout, DrawLayout::VerticalPair) {
            240
        } else {
            480
        };
        if !(i16::from_le_bytes(self.current_word(0x330).to_le_bytes()).wrapping_sub(limit) < 0
            && (((i32::from(512u16.wrapping_sub(self.session().mechanisms.draw_cursor)) >> 3)
                - i32::from(count))
                & 0x8000)
                == 0)
        {
            return None;
        }
        let destination = destination.0;
        let entry = 0xd0 + usize::from(tail);
        let (first, second);
        match layout {
            DrawLayout::VerticalPair => {
                let live_tail = self.current_word(0x330);
                self.mechanism_queue_pair(destination, live_tail, count, 0x8000);
                let next = self.mechanism_native_word(entry + 5).wrapping_add(1);
                self.mechanism_queue_word(entry + 12, next, ByteOrder::Native);
                first = usize::from(self.mechanism_native_word(entry + 2));
                second = usize::from(self.mechanism_native_word(entry + 9));
                self.mechanism_store(0x330, tail.wrapping_add(14), StoreOrder::SliceLittle);
            }
            DrawLayout::HorizontalPair => {
                self.mechanism_queue_pair(destination, tail, count, 0);
                let next = self.mechanism_native_word(entry + 5).wrapping_add(32);
                self.mechanism_queue_word(entry + 12, next, ByteOrder::Native);
                first = usize::from(self.mechanism_native_word(entry + 2));
                second = usize::from(self.mechanism_native_word(entry + 9));
                self.mechanism_store(0x330, tail.wrapping_add(14), StoreOrder::SliceLittle);
            }
            DrawLayout::HorizontalSplit => {
                let column = destination & 31;
                if i16::from_le_bytes(tail.to_le_bytes()).wrapping_sub(228) >= 0
                    || 32i16.wrapping_sub(i16::from_le_bytes(column.to_le_bytes())) < 0
                {
                    return None;
                }
                let size = 2 * (32 - column);
                self.mechanism_queue_word(entry, size, ByteOrder::Native);
                self.mechanism_queue_word(entry + 14, size, ByteOrder::Native);
                self.mechanism_queue_word(entry + 5, destination, ByteOrder::Native);
                self.mechanism_queue_word(
                    entry + 12,
                    (destination & 0xffe0) ^ 0x400,
                    ByteOrder::Native,
                );
                let next = self.mechanism_native_word(entry + 12).wrapping_add(32);
                self.mechanism_queue_word(entry + 26, next, ByteOrder::Native);
                let next = self.mechanism_native_word(entry + 5).wrapping_add(32);
                self.mechanism_queue_word(entry + 19, next, ByteOrder::Native);
                let remaining =
                    (4u16.wrapping_mul(count)).wrapping_sub(self.mechanism_native_word(entry));
                self.mechanism_queue_word(entry + 7, remaining, ByteOrder::Native);
                self.mechanism_queue_word(entry + 21, remaining, ByteOrder::Native);
                let source = self.session().mechanisms.draw_cursor.wrapping_sub(14648);
                let live_source = self.session().mechanisms.draw_cursor.wrapping_sub(14648);
                self.mechanism_queue_word(entry + 2, live_source, ByteOrder::Native);
                first = usize::from(source);
                let next = self.mechanism_native_word(entry).wrapping_add(source);
                self.mechanism_queue_word(entry + 9, next, ByteOrder::Native);
                let next = self.mechanism_native_word(entry + 7).wrapping_add(next);
                self.mechanism_queue_word(entry + 16, next, ByteOrder::Native);
                second = usize::from(next);
                let next = self.mechanism_native_word(entry + 14).wrapping_add(next);
                self.mechanism_queue_word(entry + 23, next, ByteOrder::Native);
                for offset in [4, 11, 18, 25] {
                    self.mechanism_queue_slice(entry + offset, &[126]);
                }
                self.mechanism_store(0x330, tail.wrapping_add(28), StoreOrder::SliceLittle);
            }
        }
        Some(DrawTargets {
            first: DrawTarget(first),
            second: DrawTarget(second),
        })
    }
    fn write_draw_tile(&mut self, target: &DrawTarget, offset: u16, source: TileDefinitionCorner) {
        let flip = (source.block >> 10) & 3;
        let index = usize::from((source.block & 0x3ff) * 4 + (source.corner ^ flip));
        let tile = self
            .session()
            .room
            .terrain
            .tiles()
            .get(index / 4)
            .map(|definition| definition[index % 4])
            .unwrap_or_else(|| self.current_word(0xa000 + index * 2));
        let tile = tile ^ ((source.block & 0xc00) << 4);
        self.mechanism_store(
            target.0 + usize::from(offset >> 1) * 2,
            tile,
            StoreOrder::SliceLittle,
        );
    }
    fn set_item_tile_definition(&mut self, offset: usize, value: u16) {
        if offset & 1 == 0
            && let Some(definition) = self
                .session_mut()
                .room
                .terrain
                .tiles_mut()
                .get_mut(offset / 8)
        {
            definition[(offset / 2) % 4] = value;
            return;
        }
        self.mechanism_store(0xa000 + offset, value, StoreOrder::SliceLittle);
    }
    fn reveal_block(&mut self, tile: u16, x: u16, y: u16) {
        crate::owned_hdma_update::reveal_room_block(tile, x, y, self);
    }
}

impl Effects for PreparedActorState<'_> {
    fn sound(&mut self, channel: SoundChannel, maximum: u8, cue: u16) {
        self.mechanism_sound(channel, maximum, cue);
    }
    fn music(&mut self, track: u16) {
        self.mechanism_music(track);
    }
    fn clear_music_queue(&mut self) {
        self.mechanism_clear_music_queue();
    }
    fn delayed_room_music(&mut self, frames: u16) {
        self.mechanism_delayed_room_music(frames);
    }
    fn display_message(&mut self, request: u16) -> u8 {
        self.mechanism_display_message(request)
    }
    fn poll_message(&mut self, request: u16) -> i32 {
        self.mechanism_poll_message(request)
    }
    fn player_control(&mut self, operation: u16) {
        self.mechanism_player_control(operation);
    }
    fn face_forward(&mut self) {
        self.mechanism_face_forward();
    }
    fn suit_pickup(&mut self, pickup: SuitPickup) {
        self.mechanism_suit_pickup(pickup);
    }
    fn pickup_visual(&mut self, visual: VisualUpdate) {
        self.mechanism_pickup_visual(visual);
    }
    fn save(&mut self, slot: u16) {
        self.mechanism_save(slot);
    }
    fn enemy_projectile(&mut self, definition: u16, parameter: u16) {
        crate::enemy_projectiles::lifecycle::spawn_room(self, definition, parameter);
    }
    fn wake_projectile_for_block(&mut self, block: u16) {
        self.mechanism_wake_projectile_for_block(block);
    }
    fn map_used(&self) -> bool {
        self.mechanism_map_used()
    }
    fn mark_map_used(&mut self) {
        self.mechanism_mark_map_used();
    }
    fn set_has_area_map(&mut self, value: u16) {
        self.mechanism_set_has_area_map(value);
    }
    fn mark_save_station(&mut self, station: u16) {
        self.mechanism_mark_save_station(station);
    }
    fn selected_save_slot(&self) -> u16 {
        self.mechanism_selected_save_slot()
    }
    fn statue_signal(&mut self) {
        self.mechanism_statue_signal();
    }
    fn statue_wait_actor(&mut self, actor: u16, waiting: bool) {
        self.mechanism_statue_wait_actor(actor, waiting);
    }
    fn diagnostic(&mut self, diagnostic: Diagnostic) -> u8 {
        self.mechanism_fault(diagnostic)
    }
}
impl Host for PreparedActorState<'_> {}

impl ClosingContent for PreparedActorState<'_> {
    fn capture_closing_door(
        &mut self,
        exit: &crate::room_entry::world::Identity<crate::room_entry::world::ExitId>,
    ) -> crate::room_entry::content::ClosingDoorCapture {
        use crate::room_entry::{
            content::{Capture, RecordAccess},
            world::Identity,
        };
        match exit {
            Identity::Known(position) => Capture::Prepared {
                world: self.session().world.clone(),
                position: *position,
            },
            Identity::LegacyUnresolved(key) => {
                let record = Content::loaded_record(
                    self,
                    ContentAddress {
                        bank: 0x83,
                        offset: *key,
                    },
                );
                Capture::Legacy {
                    image: record.image,
                    base: record.base,
                    access: RecordAccess::Native,
                    world: self.session().world.clone(),
                }
            }
        }
    }
    fn capture_closing_header(
        &mut self,
        direction: u16,
    ) -> crate::room_entry::content::ClosingHeaderCapture {
        use crate::room_entry::content::{Capture, ClosingHeaderPosition};
        // PreparedActorState admission requires a real prepared WorldBinding.
        // Unknown direction remains a lazy query in Capture::word, not an
        // opening-time check or a truncated direction.
        Capture::Prepared {
            world: self.session().world.clone(),
            position: ClosingHeaderPosition { direction },
        }
    }
}

impl PreparedActorState<'_> {
    /// These calls execute bodies now, before any actor/shot caller resumes.
    pub fn spawn_collision_mechanism(&mut self, header: u16, displacement: &mut Contact<'_>) -> u8 {
        crate::room_mechanisms::creation::spawn(
            self,
            SpawnSource::Collision { header },
            displacement,
        )
    }
    pub fn spawn_hardcoded_mechanism(&mut self, x: u8, y: u8, header: u16) -> u8 {
        crate::room_mechanisms::creation::spawn(
            self,
            SpawnSource::Hardcoded { x, y, header },
            &mut Contact::Absent,
        )
    }
    pub fn spawn_room_mechanism(&mut self, entry: ProgramCursor) -> u8 {
        crate::room_mechanisms::creation::spawn(
            self,
            SpawnSource::Room { entry },
            &mut Contact::Absent,
        )
    }
    pub fn clear_mechanisms(&mut self) {
        crate::room_mechanisms::creation::clear_session(self.session_mut());
    }
    pub fn close_mechanism_door(&mut self) {
        crate::room_mechanisms::creation::close_door(self);
    }
    /// Pass belongs to the active caller. It is never reconstructed from mode
    /// or reset at callback entry; message suspension returns immediately.
    pub fn advance_mechanisms(&mut self, pass: &mut Pass) -> RunResult {
        crate::room_mechanisms::scheduler::advance(self, pass)
    }
    pub fn run_mechanisms(&mut self, context: CallerContext) -> u8 {
        // Capture once, before preparation or any synchronous callback. The
        // existing unknown continuation values complete rather than starting.
        let mut pass = match *self.session().control.secondary() {
            0 => Pass::Instructions,
            1 => Pass::DisplayMessage,
            value => Pass::Invalid(value),
        };
        let result = self.advance_mechanisms(&mut pass);
        match context {
            CallerContext::Gameplay | CallerContext::DoorTransition => match result {
                RunResult::Complete => {
                    self.set_current_byte(0x613, 0);
                    0
                }
                RunResult::Yield(value) => {
                    self.set_current_byte(0x613, 1);
                    value.get()
                }
            },
        }
    }
}
