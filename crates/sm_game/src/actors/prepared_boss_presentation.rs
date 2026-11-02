//! Shared ordered boss uploads and Phantoon's state/content bindings.
#![forbid(unsafe_code)]
use crate::actors::owned_execution::PreparedActorState;
use crate::session_access::SessionAccess;

impl crate::progress::exploration::ExplorationHost for PreparedActorState<'_> {
    fn minimap(&self) -> &crate::player_update::state::State {
        &self.session().minimap
    }
    fn minimap_mut(&mut self) -> &mut crate::player_update::state::State {
        &mut self.session_mut().minimap
    }
    fn imported_exploration_byte(&self, offset: isize) -> u8 {
        self.current_byte(
            0x7f7usize
                .checked_add_signed(offset)
                .expect("exploration offset overflow"),
        )
    }
    fn set_imported_exploration_byte(&mut self, offset: isize, value: u8) {
        self.set_current_byte(
            0x7f7usize
                .checked_add_signed(offset)
                .expect("exploration offset overflow"),
            value,
        );
    }
    fn map_origin(&self) -> (u16, u16) {
        (
            self.session().room.entry.map_x,
            self.session().room.entry.map_y,
        )
    }
}
impl crate::player_update::minimap::BossMap for PreparedActorState<'_> {
    type Coordinates = u16;
    type CoordinateRecord = sm_data::cartridge::RomRecord;
    fn selected_boss(&self) -> u16 {
        self.session().actors.schedule.boss_id
    }
    fn descriptor_boss(&self, index: usize) -> u16 {
        crate::visual_host::Content::table(self, crate::visual_host::Source::Native, 0x90a83a)
            .word(index * 4)
    }
    fn descriptor_coordinates(&self, index: usize) -> u16 {
        crate::visual_host::Content::table(self, crate::visual_host::Source::Native, 0x90a83a)
            .word(index * 4 + 2)
    }
    fn coordinate_record(&mut self, coordinates: u16, index: usize) -> Self::CoordinateRecord {
        let displacement = u16::from_le_bytes(
            (index * 4).to_le_bytes()[..2]
                .try_into()
                .expect("coordinate displacement"),
        );
        self.native_actor_record(0x90, coordinates.wrapping_add(displacement))
    }
}

impl crate::presentation_workspace::Access for PreparedActorState<'_> {
    fn screen_apply<T>(
        &mut self,
        action: impl FnOnce(
            &mut dyn crate::presentation_workspace::Canvas,
        ) -> Result<T, crate::scene::state::DisplayError>,
    ) -> T {
        self.guarded_actor_call(|host| action(host.session_mut()).expect("screen operation"))
    }
    fn screen_byte(&self, point: crate::presentation_workspace::Point) -> u8 {
        let workspace = self.session().presentation.workspace();
        match workspace.byte(point) {
            Ok(byte) => byte,
            Err(error) => match workspace.alias_address(point).expect("screen binding") {
                Some(at) => self.current_byte(at),
                None => panic!("screen allocation {error:?}"),
            },
        }
    }
    fn screen_write_byte(&mut self, point: crate::presentation_workspace::Point, value: u8) {
        self.guarded_actor_call(|host| {
            let alias = host
                .session()
                .presentation
                .workspace()
                .alias_address(point)
                .expect("screen binding");
            if let Err(error) = host.session_mut().publish_workspace_byte(point, value) {
                match alias {
                    Some(at) => host.set_current_byte(at, value),
                    None => panic!("screen allocation {error:?}"),
                }
            }
        });
    }
    fn screen_write_slice(&mut self, point: crate::presentation_workspace::Point, bytes: &[u8]) {
        self.guarded_actor_call(|host| {
            let alias = host
                .session()
                .presentation
                .workspace()
                .alias_address(point)
                .expect("screen binding");
            if let Err(error) = host.session_mut().publish_workspace_slice(point, bytes) {
                match alias {
                    Some(at) => host.write_current_slice(at, bytes),
                    None => panic!("screen allocation {error:?}"),
                }
            }
        });
    }
}

impl crate::presentation_workspace::QueueAccess for PreparedActorState<'_> {
    fn queue_byte_at(&mut self, offset: usize, value: u8) {
        self.guarded_actor_call(|host| {
            host.presentation
                .queue_byte(
                    &mut host.owner.gameplay,
                    offset,
                    value,
                    host.aliases.presentation_diagnostics(),
                )
                .expect("screen queue byte publication");
            host.refresh_inputs();
        });
    }
    fn queue_native_slice_at(&mut self, offset: usize, value: u16) {
        let at = 0xd0usize.checked_add(offset).expect("queue offset");
        let end = at.checked_add(2).expect("queue slice extent");
        assert!(end <= 0x20000);
        for (lane, byte) in value.to_ne_bytes().into_iter().enumerate() {
            crate::presentation_workspace::QueueAccess::queue_byte_at(self, offset + lane, byte);
        }
    }
}

impl crate::display::queued_tilemaps::QueueAccess for PreparedActorState<'_> {
    fn queue_tail(&self) -> u16 {
        self.session().queued_tilemaps.tail
    }
    fn store_queue_tail(&mut self, value: u16) {
        crate::enemy_projectiles::host::Graphics::set_vram_write_queue_tail(self, value);
    }
    fn queue_native_word(&mut self, cursor: u16, value: u16) {
        for (lane, byte) in value.to_ne_bytes().into_iter().enumerate() {
            crate::presentation_workspace::QueueAccess::queue_byte_at(
                self,
                usize::from(cursor) + lane,
                byte,
            );
        }
    }
    fn queue_live_source(
        &mut self,
        address: u16,
        bank: u8,
        source: crate::scene::state::TransferSource,
    ) {
        let bytes = self.session().queued_tilemaps.native_order.encode(address);
        if u16::from_le_bytes(bytes) != address {
            return;
        }
        let token = [bytes[0], bytes[1], bank];
        if let Ok(owner) = self.session_mut().presentation.owner_mut() {
            owner.state.queued_sources.insert(token, source);
        }
    }
}

impl crate::actors::mother_brain_state_owned::State for PreparedActorState<'_> {
    fn exceptional_byte(&self, at: usize) -> u8 {
        self.current_byte(at)
    }
    fn set_exceptional_byte(&mut self, at: usize, value: u8) {
        self.set_current_byte(at, value);
    }
}

impl crate::display::transfer::TilemapQueue for PreparedActorState<'_> {
    fn tilemap_queue_tail(&self) -> u16 {
        self.owner.gameplay.queued_tilemaps.tail
    }
    fn set_tilemap_queue_tail(&mut self, tail: u16) {
        crate::enemy_projectiles::host::Graphics::set_vram_write_queue_tail(self, tail);
    }
    fn set_tilemap_queue_byte(&mut self, at: usize, value: u8) {
        self.guarded_actor_call(|host| {
            let offset = at
                .checked_sub(crate::display::transfer::TILEMAP_QUEUE)
                .expect("tilemap queue prefix");
            host.presentation
                .queue_byte(
                    &mut host.owner.gameplay,
                    offset,
                    value,
                    host.aliases.presentation_diagnostics(),
                )
                .expect("boss queue byte publication");
            host.refresh_inputs();
        });
    }
}

impl crate::actors::phantoon_owned::State for PreparedActorState<'_> {
    fn phantoon_random(&mut self) -> u16 {
        self.ordinary_random()
    }
    fn exceptional_byte(&self, at: usize) -> u8 {
        self.current_byte(at)
    }
    fn set_exceptional_byte(&mut self, at: usize, value: u8) {
        self.set_current_byte(at, value);
    }
    fn native_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord {
        self.native_actor_record(bank, address)
    }
    fn loaded_word(&self, address: u32, index: isize) -> u16 {
        self.loaded_table_word(address, index)
    }
    fn set_live_color_native(&mut self, index: usize, value: u16) {
        crate::enemy_projectiles::host::Graphics::set_palette_buffer(
            self,
            isize::try_from(index).expect("live palette index"),
            u16::from_le_bytes(value.to_ne_bytes()),
        );
    }
}

impl crate::actors::kraid_sinking_owned::Host for PreparedActorState<'_> {
    fn native_sinking_table(&mut self) -> sm_data::cartridge::RomRecord {
        self.native_actor_record(0xa7, 0xc5e7)
    }
    fn kraid_y(&self) -> u16 {
        self.owner.gameplay.actors.slots[0].y.whole
    }
    fn bg2_tilemap_base(&self) -> u8 {
        self.owner
            .gameplay
            .presentation
            .owner()
            .expect("prepared boss display")
            .state
            .controls
            .screens[1]
    }
}

impl crate::actors::kraid_pause_owned::Host for PreparedActorState<'_> {
    fn secondary_continuation(&self) -> u8 {
        self.owner
            .gameplay
            .control
            .metadata()
            .expect("Kraid continuation")
            .continuation
            .secondary
    }
    fn set_secondary_continuation(&mut self, value: u8) {
        self.owner
            .gameplay
            .control
            .metadata_mut()
            .expect("Kraid continuation")
            .continuation
            .secondary = value;
        self.refresh_inputs();
    }
    fn bg12_tile_base(&self) -> u8 {
        self.owner
            .gameplay
            .presentation
            .owner()
            .expect("Kraid display")
            .state
            .controls
            .characters[0]
    }
    fn set_blank(&mut self, blank: bool) {
        let brightness = &mut self
            .owner
            .gameplay
            .presentation
            .owner_mut()
            .expect("Kraid display")
            .state
            .controls
            .brightness;
        *brightness = (*brightness & 0x7f) | if blank { 0x80 } else { 0 };
        self.refresh_inputs();
    }
    fn wait_for_nmi(&mut self) -> u8 {
        let yielded = self
            .owner
            .gameplay
            .presentation
            .owner_mut()
            .expect("Kraid display")
            .state
            .nmi
            .wait(true);
        self.refresh_inputs();
        yielded
    }
    fn load_graphics(
        &mut self,
        graphics: crate::actors::kraid_pause_owned::Graphics,
        destination: u16,
        bytes: u16,
    ) {
        use crate::actors::kraid_pause_owned::Graphics;
        let token = match graphics {
            Graphics::RoomBackground => [0x16, 0xa7, 0xa7],
            Graphics::StandardBg3Tiles => [0, 0xb2, 0x9a],
            Graphics::SavedKraidTiles => [0, 0x50, 0x7e],
            Graphics::KraidTopHalfTilemap => [0, 0x20, 0x7e],
        };
        self.guarded_actor_call(|host| {
            host.presentation
                .boss_graphics(&mut host.owner.gameplay, token, destination, bytes)
                .expect("Kraid graphics source");
            host.refresh_inputs();
        });
    }
    fn sinking_counter(&self) -> u16 {
        self.owner.gameplay.messages.delay
    }
    fn set_sinking_counter(&mut self, value: u16) {
        self.owner.gameplay.messages.delay = value;
        self.refresh_inputs();
    }
    fn saved_queue_cursor(&self) -> u16 {
        self.owner.gameplay.suspended_work.queue_cursor
    }
    fn set_saved_queue_cursor(&mut self, value: u16) {
        self.owner.gameplay.suspended_work.queue_cursor = value;
        self.refresh_inputs();
    }
    fn readback_kraid_tiles(&mut self) {
        let tail = self.current_word(0x360);
        let entry = 0x340 + usize::from(tail);
        let target =
            (u16::from(crate::actors::kraid_pause_owned::Host::bg12_tile_base(self) & 0xfc) << 8)
                .wrapping_add(15872);
        // Original overlapping slice stores, each preflighted before either byte.
        self.write_current_slice(entry, &target.to_ne_bytes());
        self.write_current_slice(entry + 2, &129u16.to_ne_bytes());
        self.write_current_slice(entry + 3, &57u16.to_ne_bytes());
        self.write_current_slice(entry + 4, &0x5000u16.to_ne_bytes());
        self.write_current_slice(entry + 6, &126u16.to_ne_bytes());
        self.write_current_slice(entry + 7, &1024u16.to_ne_bytes());
        self.write_current_slice(0x360, &tail.wrapping_add(9).to_le_bytes());
    }
}
