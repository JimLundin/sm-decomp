//! The compatibility runtime as the `sm_game::menus` host.
use crate::game::room_entry::{ArrivalRequest, StartSite};
use crate::game::session_runtime::Runtime;
use crate::game::{projectile_presentation_ram as sprites, sprite_drawing::Clip, visual_runtime};
use crate::game_state::Game;
use crate::save_ram::SaveRam;
use sm_game::menus::host::{Host, MenuRuntime, MenuSprites};

impl sm_game::menus::host::SaveRam for SaveRam {
    fn word(&self, at: usize) -> u16 {
        SaveRam::word(self, at)
    }
    fn set_word(&self, at: usize, value: u16) {
        SaveRam::set_word(self, at, value)
    }
    fn bytes(&self, range: core::ops::Range<usize>) -> Vec<u8> {
        SaveRam::bytes(self, range)
    }
    fn write_bytes(&self, at: usize, input: &[u8]) {
        SaveRam::write_bytes(self, at, input)
    }
    fn copy_within(&self, source: core::ops::Range<usize>, destination: usize) {
        SaveRam::copy_within(self, source, destination)
    }
    fn fill(&self, range: core::ops::Range<usize>, value: u8) {
        SaveRam::fill(self, range, value)
    }
}

impl MenuSprites for Game {
    fn menu_sprite(&mut self, index: u16, x: u16, y: u16, palette: u16) {
        sprites::menu(index, x, y, palette, self)
    }
    fn menu_selection_sprite(&mut self) {
        sprites::menu_selection(self)
    }
    fn file_slot_cursor(&mut self, exit_y: u16) {
        sprites::file_slot_cursor(exit_y, self)
    }
    fn file_helmets(&mut self, animate: bool) {
        sprites::file_helmets(animate, self)
    }
    fn file_menu_cursor(&mut self) {
        sprites::file_menu_cursor(self)
    }
    fn file_copy_arrow(&mut self) {
        sprites::file_copy_arrow(self)
    }
    fn paletted_sprite(
        &mut self,
        bank: u8,
        address: u16,
        position: (u16, u16),
        palette: u16,
        bounded: bool,
    ) {
        assert!(bounded, "menus draw only bounded sprites");
        sprites::paletted(bank, address, position, palette, Clip::Bounded, self)
    }
}

// SAFETY (all `unsafe` calls below): the menus run from the frame loop with
// the SNES backend installed, and `&mut self` excludes any other access.
impl MenuRuntime for Game {
    fn set_hdma_enabled(&mut self, enabled: bool) {
        visual_runtime::set_hdma_enabled(enabled, self)
    }
    fn clear_hdma(&mut self) {
        unsafe { visual_runtime::clear_hdma(self) }
    }
    fn queue_native_transfer(
        &mut self,
        cursor: u16,
        size: u16,
        source: &dyn Fn(&Self) -> u16,
        bank: u8,
        destination: &dyn Fn(&Self) -> u16,
    ) -> u16 {
        crate::game::display_refresh::queue_native_transfer(
            cursor,
            size,
            source,
            bank,
            destination,
            self,
        )
    }
    fn load_vram_words(&mut self, source: u32, destination: u16, bytes: u16) {
        unsafe {
            crate::game::display_refresh::GraphicsAsset::VramWords(source, destination, bytes)
                .load(self)
        }
    }
    fn decompress_to_ram(&mut self, source: u32, destination: usize) -> usize {
        crate::game::decompression::decompress_to_ram(source, destination, self)
    }
    fn file_select_fade(&mut self) -> u8 {
        crate::game::palette_effects::fading::file_select(self)
    }
    fn boss_byte(&mut self, area: u16) -> u8 {
        crate::game::gameplay_flag_runtime::boss_byte_at(area, self)
    }
    fn enter_load_station(&mut self) {
        crate::game::room_entry::begin(
            &mut Runtime::new(self),
            ArrivalRequest::Start(StartSite::CurrentLoadStation),
        );
    }
    fn hud_reserve_mode(&mut self, automatic: bool) {
        crate::game::hud::menu_reserve_mode(automatic, self)
    }
    fn request_reset(&mut self) {
        Game::request_reset(self)
    }
    fn flush_sram(&mut self) {
        unsafe { crate::rtl::RtlWriteSram(self) }
    }
    fn native_rom_diagnostic(&mut self, address: u32) {
        crate::game::session_runtime::native_rom_diagnostic(address, self)
    }
    fn invalid_window(&mut self, address: u32) {
        crate::game::session_runtime::invalid_window(self, address)
    }
    fn legacy_assertion(&self, drawing: bool) -> ! {
        crate::game::session_runtime::legacy_assertion(drawing)
    }
    fn unreachable_native(&mut self) -> u16 {
        crate::game::animated_tiles::native::unreachable(self)
    }
    #[cfg(sm_verify_dll)]
    fn observe_map_write(&self, byte: usize, index: usize, stage: u8) {
        crate::game::session_runtime::player_legacy_edges::map_write(self, byte, index, stage)
    }
    #[cfg(sm_verify_dll)]
    fn observe_flags_save(&mut self, stage: u8) {
        crate::flags_save_observer::observe(stage, self)
    }
}

impl Host for Game {
    type Sram = SaveRam;
    fn sram(&self) -> &SaveRam {
        &self.g_sram
    }
    fn pause_menu_tables(&self) -> &sm_game::menus::map_menu::ImportedTables {
        &self.pause_menu
    }
    fn equipment_screen_tables(&self) -> &sm_game::menus::equipment_menu::ImportedTables {
        &self.equipment_screen
    }
    fn save_slot_offsets(&self) -> &Option<sm_game::menus::save_slots::ImportedSlots> {
        &self.save_slots
    }
    fn area_select_palettes(&self) -> &Option<sm_game::menus::menu_tilemap::ImportedAreaPalettes> {
        &self.area_select_palettes
    }
}
