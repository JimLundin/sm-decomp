//! The compatibility runtime as the `sm_game::menus` host.
use crate::game::session_runtime::Runtime;
use crate::game::visual_runtime;
use crate::game_state::Game;
use sm_game::menus::host::{Host, MenuRuntime};
use sm_game::save_file::SaveRam;
use {sm_game::room_entry::api::ArrivalRequest, sm_game::room_entry::api::StartSite};

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
    fn load_vram_words(&mut self, source: u32, destination: u16, bytes: u16) {
        crate::game::display_refresh::GraphicsAsset::VramWords(source, destination, bytes)
            .load(self)
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
    fn update_beam_tiles_and_palette(&mut self) {
        crate::game::visual_runtime::update_beam_tiles_and_palette(self)
    }
    fn spawn_hdma(&mut self, bank: u8, args: &crate::types::HdmaObjectDefinition) -> u16 {
        sm_game::hdma_objects::spawn_hdma_owned(bank, args, self)
    }
    fn rtl_unreachable(&mut self) -> bool {
        crate::rtl::report_unreachable(self)
    }
    fn shake_room_with_sound(&mut self, kind: u16) {
        visual_runtime::shake_room_with_sound(kind, self)
    }
    fn set_boss_byte(&mut self, area: u16, value: u8) {
        crate::game::gameplay_flag_runtime::set_boss_byte_at(area, value, self)
    }
    fn native_image_binding(&self) -> Option<crate::rom::RomImage> {
        self.capture_native_rom_binding()
    }
    fn run_hdma_objects(&mut self) {
        // Closing the box ($85:8589) ends on a borrow, and nothing up to the handler sets C.
        self.message_box_hdma = true;
        crate::game::visual_runtime::update_hdma(self);
        self.message_box_hdma = false;
    }
    fn hdma_handler_carry(&self) -> bool {
        !self.message_box_hdma
    }
    fn advance_dialogue_sound_effects(&mut self) {
        crate::game::music::advance_sound_effects(self, |channel, value, meaning, game| {
            // SAFETY: the dialogue runs from the frame loop; the Cue sink writes the installed APU queue.
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
        sm_game::cinematic_dispatch::run_sprite_pre_instruction(pre, object, self)
    }
    fn cinematic_sprite_instruction(&mut self, command: u32, object: u16, operand: u16) -> u16 {
        sm_game::cinematic_states::run_sprite_instruction(command, object, operand, self)
    }
    fn enqueue_mode7_rom(&mut self, bank: u8, cursor: u16) {
        crate::game::mode7_transfers::enqueue_rom(bank, cursor, self)
    }
    fn event_happened(&mut self, event: u16) -> u16 {
        crate::game::gameplay_flag_runtime::event_has_happened(event, self)
    }
    fn set_event_happened(&mut self, event: u16) {
        crate::game::gameplay_flag_runtime::mark_event(event, self)
    }
    fn boss_bit_for_current_area(&mut self, bit: u16) -> u8 {
        crate::game::gameplay_flag_runtime::check_boss_bit_for_cur_area(bit, self)
    }
    fn spawn_hardcoded_plm(&mut self, x: u8, y: u8, id: u16) {
        crate::game::room_mechanisms::spawn_hardcoded(
            crate::types::SpawnHardcodedPlmArgs::new(x, y, id),
            self,
        )
    }
    fn spawn_room_enemy_projectile(&mut self, definition: u16, parameter: u16) -> u16 {
        crate::game::enemy_projectiles::spawn_with_room_gfx(definition, parameter, self)
    }
    fn write_dma_register(&mut self, register: u16, value: u8) {
        crate::rtl::write_dma_register(register, value, self)
    }
    fn handle_music_queue(&mut self) {
        // SAFETY: HDMA updates run from the frame loop with the backend installed.
        unsafe { crate::game::music::handle_music_queue(self) }
    }
    fn hdma_effect(&mut self, effect: sm_game::hdma_objects::HdmaEffect, slot: u16) {
        sm_game::hdma_effects::apply(effect, slot, &mut Runtime::new(self));
    }
    fn request_reset(&mut self) {
        Game::request_reset(self)
    }
    fn flush_sram(&mut self) {
        unsafe { crate::rtl::rtl_write_sram(self) }
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
    fn read_rom_word(&self, address: u32, offset: usize) -> u16 {
        Game::read_rom_word(self, address, offset)
    }
    fn visual_tables(&self) -> &sm_game::visual_tables::ImportedTables {
        &self.visual_tables
    }
    fn area_select_palettes(&self) -> &Option<sm_game::menus::menu_tilemap::ImportedAreaPalettes> {
        &self.area_select_palettes
    }
    fn nmi_controller_word(&mut self) -> u16 {
        crate::oracle_mode::nmi_input()
            .map(u16::reverse_bits)
            .unwrap_or_else(|| {
                sm_game::display::sequence::Registers::read_register_word(self, 0x4218)
            })
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

impl sm_game::cinematic_host::Host for Game {
    fn spawn_power_bomb(&mut self) {
        sm_game::explosion_window::spawn_power_bomb(self);
    }

    fn initialize_player(&mut self) {
        crate::game::session_runtime::initialize_player(self);
    }

    fn disable_scanline_irq(&mut self) {
        sm_game::display_refresh::disable_scanline_irq(self);
    }

    fn advance_ending_tile_upload(&mut self) {
        crate::game::mode7_transfers::advance_ending_tile_upload(self);
    }

    fn advance_intro_player(&mut self) {
        crate::game::player_projectiles::prepare(self);
        crate::game::session_runtime::player_update_runtime::advance_frame(self);
    }

    fn native_assertion(
        &mut self,
        expression: &'static [u8],
        line: u32,
        function: &'static [u8],
    ) -> ! {
        crate::c_library::assertion_failed(expression, b"src/sm_8b.c\0", line, function)
    }
}

// The menus address these enemy record fields by offset.
const _: () = assert!(core::mem::offset_of!(crate::types::EnemyData, x_pos) == 2);
const _: () = assert!(core::mem::offset_of!(crate::types::EnemyData, palette_index) == 30);

// The intro's next-page arrow is a word of the menu tilemap at $7E:3000.
const _: () = assert!(
    sm_game::intro_sequence::NEXT_PAGE_ARROW
        == 0x3000 + core::mem::offset_of!(crate::types::Ram3000, menu.menu_tilemap) + 286
);
