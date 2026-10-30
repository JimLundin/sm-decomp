//! The compatibility runtime as the `sm_game::menus` host.
use crate::game::session_runtime::Runtime;
use crate::game::visual_runtime;
use crate::game_state::Game;
use crate::save_ram::SaveRam;
use sm_game::menus::host::{Host, MenuRuntime};
use {sm_game::room_entry::api::ArrivalRequest, sm_game::room_entry::api::StartSite};

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
    fn file_select_fade(&mut self) -> u8 {
        crate::game::palette_effects::fading::file_select(self)
    }
    fn boss_byte(&mut self, area: u16) -> u8 {
        crate::game::gameplay_flag_runtime::boss_byte_at(area, self)
    }
    fn enter_load_station(&mut self) {
        sm_game::room_entry::selection::begin(
            &mut Runtime::new(self),
            ArrivalRequest::Start(StartSite::CurrentLoadStation),
        );
    }
    fn queue_fx_clear(&mut self) {
        crate::game::display_refresh::queue_fx_clear(self)
    }
    fn player_command(&mut self, command: u16) -> u16 {
        crate::game::session_runtime::player_command(command, self)
    }
    fn update_beam_tiles_and_palette(&mut self) {
        crate::game::visual_runtime::update_beam_tiles_and_palette(self)
    }
    fn has_queued_music(&mut self) -> u8 {
        crate::game::music::HasQueuedMusic(self)
    }
    fn queue_music_delayed8(&mut self, track: u16) {
        crate::game::music::QueueMusic_Delayed8(track, self)
    }
    fn spawn_hdma(&mut self, bank: u8, args: &crate::types::SpawnHdmaObject_Args) -> u16 {
        visual_runtime::spawn_hdma(bank, args, self)
    }
    fn rtl_unreachable(&mut self) -> bool {
        crate::rtl::Unreachable(self)
    }
    fn spawn_bg3_scroll(&mut self) {
        visual_runtime::spawn_bg3_scroll(self)
    }
    fn spawn_animated_tiles(&mut self, definition: u16) {
        crate::game::animated_tiles::spawn(definition, self)
    }
    fn shake_room_with_sound(&mut self, kind: u16) {
        visual_runtime::shake_room_with_sound(kind, self)
    }
    fn display_wait(&mut self, update: bool) -> u8 {
        crate::game::display_refresh::wait(update, self)
    }
    fn set_boss_byte(&mut self, area: u16, value: u8) {
        crate::game::gameplay_flag_runtime::set_boss_byte_at(area, value, self)
    }
    fn native_image_binding(&self) -> Option<crate::rom::RomImage> {
        self.capture_native_rom_binding()
    }
    fn advance_dialogue_audio(&mut self) {
        // SAFETY: the dialogue runs from the frame loop with the backend installed.
        unsafe { crate::game::music::HandleMusicQueue(self) };
        crate::game::music::advance_sound_effects(self, |channel, value, meaning, game| {
            // SAFETY: as above; the Cue sink writes the installed APU queue.
            unsafe {
                crate::native_audio::emit(
                    crate::native_audio::sound_cue(
                        channel,
                        sm_data::OriginalSoundEffect(value),
                        meaning,
                    ),
                    sm_data::CuePhase::DialogueSound,
                    game,
                )
            }
        });
    }
    fn restore_movement_sound(&mut self) {
        crate::game::player_projectiles::restore_sound(self)
    }
    fn message_box_das0l_value(&self) -> u16 {
        self.message_box_das0l_value
    }
    fn set_message_box_das0l_value(&mut self, value: u16) {
        self.message_box_das0l_value = value;
    }
    fn assert_ram_pointer(&mut self, address: u16) {
        // SAFETY: the failure path reports through the C assertion binding and does not return.
        unsafe {
            crate::native_assertion! {
                '_c2rust_label: address < 0x2000;
                b"addr < 0x2000\0", b"src/sm_rtl.h\0", 51, b"uint8 *RomPtr_RAM(uint16_t)\0"
            };
        }
    }
    fn cinematic_sprite_pre_instruction(&mut self, pre: u32, object: u16) {
        crate::game::sm_8b::CallCinematicSprPreInstr(pre, object, self)
    }
    fn cinematic_sprite_instruction(&mut self, command: u32, object: u16, operand: u16) -> u16 {
        crate::game::sm_8b::CallCinematicSprInstr(command, object, operand, self)
    }
    fn enqueue_mode7_rom(&mut self, bank: u8, cursor: u16) {
        crate::game::mode7_transfers::enqueue_rom(bank, cursor, self)
    }
    fn retire_projectile(&mut self, slot: u16) {
        crate::game::player_projectiles::retire(slot, self)
    }
    fn spawn_palette_effect(&mut self, definition: u16) {
        crate::game::palette_effects::spawn(definition, self)
    }
    fn enable_display_nmi(&mut self, enabled: bool) {
        // SAFETY: the intro runs from the frame loop with the backend installed.
        unsafe { crate::game::display_refresh::enable(enabled, self) }
    }
    fn finish_pose_change(&mut self) {
        crate::game::player_transitions::finish_pose_change(&mut Runtime::new(self));
    }
    fn demo_start(&mut self) {
        crate::game::player_transitions::demo::start(Runtime::new(self).session_mut());
    }
    fn demo_load(&mut self, mode: u16, input: u16) {
        crate::game::player_transitions::demo::load(&mut Runtime::new(self), mode, input);
    }
    fn request_reset(&mut self) {
        Game::request_reset(self)
    }
    fn flush_sram(&mut self) {
        unsafe { crate::rtl::RtlWriteSram(self) }
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
    #[cfg(sm_verify_dll)]
    fn observe_selected_continue(&mut self, load_result: u8) {
        crate::flags_save_observer::selected_continue(load_result)
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
    fn demo_sets(&self) -> &Option<sm_game::demo_rooms::ImportedSets> {
        &self.demo_sets
    }
    fn demo_scenes(&self) -> &Option<sm_game::demo_rooms::ImportedScenes> {
        &self.demo_scenes
    }
    fn intro_palette(&self) -> &Option<sm_game::intro_pages::ImportedIntroPalette> {
        &self.intro_palette
    }
    fn visual_tables(&self) -> &sm_game::visual_tables::ImportedTables {
        &self.visual_tables
    }
    fn area_select_palettes(&self) -> &Option<sm_game::menus::menu_tilemap::ImportedAreaPalettes> {
        &self.area_select_palettes
    }
}

impl sm_game::cartridge::NativeDiagnostics for Game {
    fn native_rom_diagnostic(&mut self, address: u32) {
        crate::game::session_runtime::native_rom_diagnostic(address, self)
    }
}

impl sm_game::decompression::DecompressionHost for Game {
    fn decompress_source(&self) -> u32 {
        self.decompress_src
    }
    fn set_decompress_source(&mut self, source: u32) {
        self.decompress_src = source;
    }
    fn report_invalid_rom_address(&mut self, address: u32) {
        crate::rtl::report_invalid_rom_address(address, self)
    }
    fn read_rom_byte(&self, address: u32, offset: usize) -> u8 {
        Game::read_rom_byte(self, address, offset)
    }
}

// The moved menus and screens address these records by layout.
const _: () = assert!(core::mem::size_of::<crate::types::VramWriteEntry>() == 7);
const _: () = assert!(core::mem::offset_of!(crate::types::PairU16, j) == 2);
const _: () = assert!(
    core::mem::offset_of!(crate::types::EnemyData, ai_preinstr)
        == core::mem::offset_of!(sm_game::menus::prelude::EnemyData, ai_preinstr)
);

// The intro's local copies of these bank $8B definitions must match.
const _: () = {
    use crate::game::sm_8b as bank;
    use sm_game::intro_pages as intro;
    assert!(
        intro::addr_kCinematicBgObjectDef_8BCF3F == bank::addr_kCinematicBgObjectDef_8BCF3F as u32
    );
    assert!(
        intro::addr_kCinematicBgObjectDef_8BCF5D == bank::addr_kCinematicBgObjectDef_8BCF5D as u32
    );
    assert!(
        intro::addr_kCinematicSpriteObjectDef_8BCE5B
            == bank::addr_kCinematicSpriteObjectDef_8BCE5B as u32
    );
    assert!(
        intro::addr_kCinematicSpriteObjectDef_8BCE55
            == bank::addr_kCinematicSpriteObjectDef_8BCE55 as u32
    );
    assert!(
        intro::addr_kCinematicSpriteObjectDef_8BCE61
            == bank::addr_kCinematicSpriteObjectDef_8BCE61 as u32
    );
    assert!(
        intro::addr_kCinematicSpriteObjectDef_8BCE67
            == bank::addr_kCinematicSpriteObjectDef_8BCE67 as u32
    );
    assert!(
        intro::addr_kCinematicSpriteObjectDef_8BCE73
            == bank::addr_kCinematicSpriteObjectDef_8BCE73 as u32
    );
    assert!(
        intro::addr_kCinematicSpriteObjectDef_8BCE79
            == bank::addr_kCinematicSpriteObjectDef_8BCE79 as u32
    );
    assert!(
        intro::addr_kCinematicSpriteObjectDef_8BCF27
            == bank::addr_kCinematicSpriteObjectDef_8BCF27 as u32
    );
};

// The cinematic engine's local copies of these bank $8B definitions must match.
const _: () = {
    use crate::game::sm_8b as bank;
    use sm_game::cinematic_objects as engine;
    assert!(
        engine::addr_kCinematicSpriteObjectDef_8BA113
            == bank::addr_kCinematicSpriteObjectDef_8BA113 as u32
    );
    assert!(
        engine::addr_kCinematicSpriteObjectDef_8BCEB5
            == bank::addr_kCinematicSpriteObjectDef_8BCEB5 as u32
    );
    assert!(
        engine::addr_kCinematicSpriteObjectDef_8BEEB5
            == bank::addr_kCinematicSpriteObjectDef_8BEEB5 as u32
    );
    assert!(
        engine::addr_kCinematicSpriteObjectDef_8BEEBB
            == bank::addr_kCinematicSpriteObjectDef_8BEEBB as u32
    );
    assert!(
        engine::addr_kCinematicSpriteObjectDef_8BEEC1
            == bank::addr_kCinematicSpriteObjectDef_8BEEC1 as u32
    );
    assert!(
        engine::addr_kCinematicSpriteObjectDef_8BEECD
            == bank::addr_kCinematicSpriteObjectDef_8BEECD as u32
    );
    assert!(
        engine::addr_kCinematicSpriteObjectDef_8BEEF7
            == bank::addr_kCinematicSpriteObjectDef_8BEEF7 as u32
    );
    assert!(
        engine::addr_kCinematicSpriteObjectDef_8BEEFD
            == bank::addr_kCinematicSpriteObjectDef_8BEEFD as u32
    );
    assert!(
        engine::addr_kCinematicSpriteObjectDef_8BEF3F
            == bank::addr_kCinematicSpriteObjectDef_8BEF3F as u32
    );
    assert!(
        engine::addr_kCinematicSpriteObjectDef_8BEF6F
            == bank::addr_kCinematicSpriteObjectDef_8BEF6F as u32
    );
    assert!(
        engine::addr_kCinematicSpriteObjectDef_8BEF7B
            == bank::addr_kCinematicSpriteObjectDef_8BEF7B as u32
    );
    assert!(
        engine::addr_kCinematicSpriteObjectDef_8BEF75
            == bank::addr_kCinematicSpriteObjectDef_8BEF75 as u32
    );
    assert!(engine::addr_loc_8BE1B4 == bank::addr_loc_8BE1B4 as u32);
    assert!(engine::addr_kMode7ObjectDef_8BD401 == bank::addr_kMode7ObjectDef_8BD401 as u32);
    assert!(engine::addr_kMode7ObjectDef_8BD407 == bank::addr_kMode7ObjectDef_8BD407 as u32);
    assert!(engine::addr_kMode7ObjectDef_8BD413 == bank::addr_kMode7ObjectDef_8BD413 as u32);
    assert!(engine::addr_kMode7ObjectDef_8BD41F == bank::addr_kMode7ObjectDef_8BD41F as u32);
    assert!(engine::addr_kMode7ObjectDef_8BD42B == bank::addr_kMode7ObjectDef_8BD42B as u32);
    assert!(engine::addr_kMode7ObjectDef_8BD437 == bank::addr_kMode7ObjectDef_8BD437 as u32);
};
