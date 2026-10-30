//! The compatibility runtime as the `sm_game::menus` host.
use crate::game::room_entry::{ArrivalRequest, StartSite};
use crate::game::session_runtime::Runtime;
use crate::game::visual_runtime;
use crate::game_state::Game;
use crate::save_ram::SaveRam;
use sm_game::menus::host::{Host, MenuRuntime};

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

impl sm_game::sprite_output::SpriteHost for Game {
    fn warn_invalid_rom(&mut self, address: u32) {
        // SAFETY: a C-string literal and one integer argument, as the format expects.
        unsafe {
            crate::c_library::printf(
                c"RomPtr - Invalid access 0x%x!
"
                .as_ptr(),
                address,
            );
        }
    }
    fn set_failure(&mut self) {
        self.g_fail = true;
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
