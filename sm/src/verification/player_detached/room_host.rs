//! Prepared room host. Every unprepared backend effect fails explicitly.
#![allow(unused_variables)]
use super::{Composed, Event, RomImage};
pub(crate) struct RoomRecord {
    image: RomImage,
    base: usize,
}
use super::game::room_mechanisms::host::*;

impl ImportedAliases for Composed {
    fn saved_graphics_word(&self, index: usize) -> u16 {
        panic!("unprepared saved graphics alias")
    }
    fn set_saved_graphics_word(&mut self, index: usize, value: u16, order: StoreOrder) {
        panic!("unprepared saved graphics alias")
    }

    fn mechanism_word(&self, member: SlotMember, signed_slot: isize) -> u16 {
        panic!("unprepared room operation: mechanism_word")
    }
    fn mechanism_byte(&self, member: SlotMember, signed_slot: isize, lane: usize) -> u8 {
        panic!("unprepared room operation: mechanism_byte")
    }
    fn set_mechanism_word(
        &mut self,
        member: SlotMember,
        signed_slot: isize,
        value: u16,
        order: StoreOrder,
    ) {
        panic!("unprepared room operation: set_mechanism_word")
    }
    fn set_mechanism_byte(
        &mut self,
        member: SlotMember,
        signed_slot: isize,
        lane: usize,
        value: u8,
    ) {
        panic!("unprepared room operation: set_mechanism_byte")
    }
    fn projectile_kind(&self, signed_slot: isize, order: ByteOrder) -> u16 {
        panic!("unprepared room operation: projectile_kind")
    }
    fn terrain_word(&self, byte_block: usize, order: ByteOrder) -> u16 {
        panic!("unprepared room operation: terrain_word")
    }
    fn set_terrain_word(&mut self, byte_block: usize, value: u16, order: ByteOrder) {
        panic!("unprepared room operation: set_terrain_word")
    }
    fn terrain_high(&self, unrounded_byte_block: u16) -> u8 {
        panic!("unprepared room operation: terrain_high")
    }
    fn set_terrain_high(&mut self, unrounded_byte_block: u16, value: u8) {
        panic!("unprepared room operation: set_terrain_high")
    }
    fn behavior_byte(&self, byte_index: usize) -> u8 {
        panic!("unprepared room operation: behavior_byte")
    }
    fn set_behavior_byte(&mut self, byte_index: usize, value: u8) {
        panic!("unprepared room operation: set_behavior_byte")
    }
    fn set_behavior_pair(&mut self, byte_index: usize, value: u16) {
        panic!("unprepared room operation: set_behavior_pair")
    }
    fn contact_displacement(&self, token: &ContactIdentity) -> i32 {
        panic!("unprepared room operation: contact_displacement")
    }
    fn set_contact_displacement(&mut self, token: &ContactIdentity, value: i32) {
        panic!("unprepared room operation: set_contact_displacement")
    }
    fn scroll_pair(&mut self, byte_index: usize, value: u16) {
        panic!("unprepared room operation: scroll_pair")
    }
    fn set_scroll(&mut self, byte_index: usize, value: u8) {
        panic!("unprepared room operation: set_scroll")
    }
    fn signal_turret(&mut self, association: u16) {
        panic!("unprepared room operation: signal_turret")
    }
}

impl Content for Composed {
    type Record = RoomRecord;
    type TerrainProgram = ();
    fn loaded_record(&mut self, address: ContentAddress) -> Self::Record {
        assert_eq!(address.bank, 0x84);
        RoomRecord {
            image: self.image.clone(),
            base: usize::from(address.bank & 0x7f) * 0x8000 + usize::from(address.offset & 0x7fff),
        }
    }
    fn native_record(&mut self, address: ContentAddress) -> Self::Record {
        <Self as Content>::loaded_record(self, address)
    }
    fn definition_word(&mut self, address: ContentAddress, byte_offset: usize) -> u16 {
        panic!("unprepared room operation: definition_word")
    }
    fn table_word(&self, table: ContentTable, index: isize) -> u16 {
        panic!("unprepared room operation: table_word")
    }
    fn capture_terrain_program(&mut self, id: DrawProgramId) -> Self::TerrainProgram {
        panic!("unprepared room operation: capture_terrain_program")
    }
    fn terrain_program_word(&self, program: &Self::TerrainProgram, byte_offset: usize) -> u16 {
        panic!("unprepared room operation: terrain_program_word")
    }
    fn drawing_word(&mut self, cursor: DrawCursor) -> u16 {
        panic!("unprepared room operation: drawing_word")
    }
    fn drawing_byte(&mut self, cursor: DrawCursor) -> u8 {
        panic!("unprepared room operation: drawing_byte")
    }
}

impl Graphics for Composed {
    fn queue_graphics_transfer<R: InstructionRecord>(
        &mut self,
        record: &R,
        kind: GraphicsTransferKind,
    ) {
        panic!("unprepared room operation: queue_graphics_transfer")
    }
    fn queue_draw_transfers(
        &mut self,
        destination: TilemapDestination,
        count: u16,
        layout: DrawLayout,
    ) -> Option<DrawTargets> {
        panic!("unprepared room operation: queue_draw_transfers")
    }
    fn write_draw_tile(&mut self, target: &DrawTarget, offset: u16, source: TileDefinitionCorner) {
        panic!("unprepared room operation: write_draw_tile")
    }
    fn set_item_tile_definition(&mut self, word_index: usize, value: u16) {
        panic!("unprepared room operation: set_item_tile_definition")
    }
    fn reveal_block(&mut self, block: u16, x: u16, y: u16) {
        panic!("unprepared room operation: reveal_block")
    }
}

impl Effects for Composed {
    fn set_has_area_map(&mut self, value: u16) {
        panic!("unprepared area map effect")
    }

    fn sound(&mut self, channel: SoundChannel, maximum: u8, cue: u16) {
        let channel = match channel {
            SoundChannel::One => 1,
            SoundChannel::Two => 2,
            SoundChannel::Three => 3,
        };
        self.events.push(Event::RoomSound(channel, maximum, cue));
        if let Some(callback) = self.on_room_sound {
            callback(self);
        }
    }
    fn music(&mut self, track: u16) {
        panic!("unprepared room operation: music")
    }
    fn clear_music_queue(&mut self) {
        panic!("unprepared room operation: clear_music_queue")
    }
    fn delayed_room_music(&mut self, frames: u16) {
        assert_eq!(frames, 0x168);
        self.events.push(Event::RoomMusic(frames));
        if let Some(message) = self.late_message {
            self.byte(0x84, 0x9802, message);
        }
    }
    fn display_message(&mut self, request: u16) -> u8 {
        self.events.push(Event::RoomMessage(
            request,
            self.world.player.inventory.ammunition[0].current,
        ));
        self.message_yields
            .pop_front()
            .expect("unprepared message display")
    }
    fn poll_message(&mut self, request: u16) -> i32 {
        panic!("unprepared room operation: poll_message")
    }
    fn player_control(&mut self, operation: u16) {
        self.events.push(Event::RoomControl(
            operation,
            self.world.player.messages.queued,
            self.world.player.inventory.ammunition[0].current,
        ));
        assert_eq!(operation, 1);
        assert_eq!(self.world.player.messages.queued, 0x16);
        assert_eq!(
            self.world.player.inventory.ammunition[0].current,
            self.world.player.inventory.ammunition[0].maximum
        );
    }
    fn face_forward(&mut self) {
        panic!("unprepared room operation: face_forward")
    }
    fn suit_pickup(&mut self, pickup: SuitPickup) {
        panic!("unprepared room operation: suit_pickup")
    }
    fn pickup_visual(&mut self, visual: VisualUpdate) {
        let id = match visual {
            VisualUpdate::Missiles => {
                assert_eq!(self.world.player.inventory.ammunition[0].current, 5);
                2
            }
            VisualUpdate::Beams => {
                assert_eq!(self.world.player.inventory.beams.active, 1);
                1
            }
            _ => panic!("unprepared pickup visual"),
        };
        self.events.push(Event::RoomVisual(id));
    }
    fn save(&mut self, slot: u16) {
        panic!("unprepared room operation: save")
    }
    fn enemy_projectile(&mut self, definition: u16, parameter: u16) {
        super::game::enemy_projectiles::spawn_room(self, definition, parameter);
    }
    fn wake_projectile_for_block(&mut self, block: u16) {
        panic!("unprepared room operation: wake_projectile_for_block")
    }
    fn map_used(&self) -> bool {
        panic!("unprepared room operation: map_used")
    }
    fn mark_map_used(&mut self) {
        panic!("unprepared room operation: mark_map_used")
    }
    fn mark_save_station(&mut self, station: u16) {
        panic!("unprepared room operation: mark_save_station")
    }
    fn selected_save_slot(&self) -> u16 {
        panic!("unprepared room operation: selected_save_slot")
    }
    fn statue_signal(&mut self) {
        panic!("unprepared room operation: statue_signal")
    }
    fn statue_wait_actor(&mut self, actor: u16, waiting: bool) {
        panic!("unprepared room operation: statue_wait_actor")
    }
    fn diagnostic(&mut self, diagnostic: Diagnostic) -> u8 {
        panic!("unprepared room operation: diagnostic")
    }
}

impl Host for Composed {}

impl InstructionRecord for RoomRecord {
    fn byte(&self, offset: isize) -> u8 {
        self.image.read(
            self.base
                .checked_add_signed(offset)
                .expect("room content offset"),
        )
    }
    fn word(&self, offset: isize) -> u16 {
        u16::from_ne_bytes([self.byte(offset), self.byte(offset.checked_add(1).unwrap())])
    }
}
