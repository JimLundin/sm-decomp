//! Kraid and boss transfer bindings on Runtime's existing Session authority.
#![forbid(unsafe_code)]
use super::Runtime;
use sm_data::cartridge::RomRecord;
use sm_game::actors::content_records::ContentSource;
use sm_game::actors::{
    boss_decompression_owned, boss_part_hitboxes_owned, boss_vram_entries_owned, kraid_owned,
    kraid_parts_owned, kraid_sinking_owned,
};
use sm_game::session_access::SessionAccess;
fn native(host: &mut Runtime<'_>, bank: u8, address: u16) -> RomRecord {
    sm_game::player_projectiles::host::Host::native_record(host, bank, address)
}
impl kraid_owned::Host for Runtime<'_> {
    fn observe_kraid_flags(&mut self, step: u8) {
        self.foreign(|game| crate::game::hosts::observe_kraid_flags(step, game));
    }
    fn disable_boss_minimap(&mut self) {
        sm_game::player_update::minimap::disable_for_boss(self);
    }
    fn native_actor_record(&mut self, bank: u8, address: u16) -> RomRecord {
        native(self, bank, address)
    }
}
impl kraid_parts_owned::Host for Runtime<'_> {
    fn native_kraid_part(&mut self, bank: u8, address: u16) -> RomRecord {
        native(self, bank, address)
    }
}
impl sm_game::actors::kraid_lint_owned::Host for Runtime<'_> {
    fn actor_record(&mut self, bank: u8, address: u16) -> RomRecord {
        native(self, bank, address)
    }
    fn react_to_contact(&mut self, death: sm_game::actors::host::Death) {
        sm_game::actors::owned_common_lifecycle::BossState::react_to_contact(self, death);
    }
}
impl boss_part_hitboxes_owned::Host for Runtime<'_> {
    fn native_actor_record(&mut self, bank: u8, address: u16) -> RomRecord {
        native(self, bank, address)
    }
}
impl boss_vram_entries_owned::Host for Runtime<'_> {
    fn native_actor_record(&mut self, bank: u8, address: u16) -> RomRecord {
        native(self, bank, address)
    }
}
impl boss_decompression_owned::Host for Runtime<'_> {
    fn loaded_decompression_image(&self) -> sm_data::cartridge::RomImage {
        self.content_image()
    }
    fn invalid_decompression_address(&mut self, address: u32) {
        self.foreign(|game| crate::rtl::report_invalid_rom_address(address, game));
    }
}
impl kraid_sinking_owned::Host for Runtime<'_> {
    fn native_sinking_table(&mut self) -> RomRecord {
        native(self, 0xa7, 0xc5e7)
    }
    fn kraid_y(&self) -> u16 {
        self.session().actors.slots[0].y.whole
    }
    fn bg2_tilemap_base(&self) -> u8 {
        sm_game::display::preparation::Access::display_control_byte(
            self,
            sm_game::display::preparation::Field::Screens1,
        )
    }
}

impl sm_game::actors::kraid_pause_owned::Host for Runtime<'_> {
    fn secondary_continuation(&self) -> u8 {
        *self.session().control.secondary()
    }
    fn set_secondary_continuation(&mut self, value: u8) {
        *self.session_mut().control.secondary_mut() = value;
        self.refresh_inputs();
    }
    fn bg12_tile_base(&self) -> u8 {
        sm_game::display::preparation::Access::display_control_byte(
            self,
            sm_game::display::preparation::Field::Characters0,
        )
    }
    fn set_blank(&mut self, blank: bool) {
        use sm_game::display::preparation::{Access, Field, Store};
        let brightness =
            (self.display_control_byte(Field::Brightness) & 0x7f) | if blank { 0x80 } else { 0 };
        self.store_display_control_byte(Field::Brightness, brightness, Store::Owned);
    }
    fn wait_for_nmi(&mut self) -> u8 {
        use sm_game::display::preparation::{Access, Field, Store};
        if self.display_control_byte(Field::NmiAcknowledgment) != 0 {
            self.store_display_control_byte(Field::NmiAcknowledgment, 0, Store::Owned);
            return 0;
        }
        self.store_display_control_byte(Field::NmiPending, 1, Store::Owned);
        self.store_display_control_byte(Field::NmiAcknowledgment, 1, Store::Owned);
        1
    }
    fn load_graphics(
        &mut self,
        graphics: sm_game::actors::kraid_pause_owned::Graphics,
        destination: u16,
        words: u16,
    ) {
        use sm_game::actors::kraid_pause_owned::Graphics;
        let source = match graphics {
            Graphics::RoomBackground => 0xa7a716,
            Graphics::StandardBg3Tiles => 0x9ab200,
            Graphics::SavedKraidTiles => 0x7e5000,
            Graphics::KraidTopHalfTilemap => 0x7e2000,
        };
        self.foreign(|game| {
            sm_game::display_refresh::GraphicsAsset::VramWords(source, destination, words)
                .load(game)
        });
    }
    fn sinking_counter(&self) -> u16 {
        self.session().messages.delay
    }
    fn set_sinking_counter(&mut self, value: u16) {
        self.session_mut().messages.delay = value;
        self.refresh_inputs();
    }
    fn saved_queue_cursor(&self) -> u16 {
        self.session().suspended_work.queue_cursor
    }
    fn set_saved_queue_cursor(&mut self, value: u16) {
        self.session_mut().suspended_work.queue_cursor = value;
        self.refresh_inputs();
    }
    fn readback_kraid_tiles(&mut self) {
        if !self.session().presentation.is_owned() {
            self.foreign(|game| {
                sm_game::display_refresh::enqueue_readback(0x5d, 15872, 0x5000, 1024, false, game)
            });
            return;
        }
        let tail = self
            .session()
            .presentation
            .owner()
            .expect("Kraid readback")
            .state
            .readback_tail;
        let entry = 0x340 + usize::from(tail);
        let target =
            (u16::from(sm_game::actors::kraid_pause_owned::Host::bg12_tile_base(self) & 0xfc) << 8)
                .wrapping_add(15872);
        self.write_slice(entry, &target.to_ne_bytes());
        self.write_slice(entry + 2, &129u16.to_ne_bytes());
        self.write_slice(entry + 3, &57u16.to_ne_bytes());
        self.write_slice(entry + 4, &0x5000u16.to_ne_bytes());
        self.write_slice(entry + 6, &126u16.to_ne_bytes());
        self.write_slice(entry + 7, &1024u16.to_ne_bytes());
        self.write_slice(0x360, &tail.wrapping_add(9).to_le_bytes());
    }
}
