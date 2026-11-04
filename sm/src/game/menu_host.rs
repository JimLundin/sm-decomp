//! The compatibility runtime as the `sm_game::menus` host.
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use sm_game::menu_state::SaveSelection;
use sm_game::menus::host::{Host, MenuRuntime};
use sm_game::save_file::SaveRam;
use sm_game::session_access::SessionAccess as _;

impl sm_game::sprite_output::SpriteHost for Game {}

/// The RAM address of a sprite output word.
pub(crate) fn sprite_word_address(word: sm_game::sprite_output::SpriteWord) -> usize {
    use sm_game::sprite_output::SpriteWord::*;
    match word {
        EchoX(i) => 0xab0usize.checked_add(2 * i).expect("echo offset"),
        EchoY(i) => 0xab8usize.checked_add(2 * i).expect("echo offset"),
        EchoSpeed(i) => 0xac0usize.checked_add(2 * i).expect("echo offset"),
        TopHalf => 0xac8,
        BottomHalf => 0xaca,
        GrappleEndX => 0xd08,
        GrappleEndY => 0xd0c,
    }
}
/// The layer 1 camera words sprite output reads.
pub(crate) const SPRITE_CAMERA: [usize; 2] = [0x911, 0x915];
/// The RAM address of shot `slot`'s spritemap pointer.
pub(crate) fn projectile_sprite_address(slot: u16) -> usize {
    0xcb8 + 2 * usize::from(slot >> 1)
}

impl sm_game::sprite_output::SpriteWords for Game {
    fn sprite_word(&self, word: sm_game::sprite_output::SpriteWord) -> u16 {
        let at = sprite_word_address(word);
        u16::from_le_bytes([self.compatibility_byte(at), self.compatibility_byte(at + 1)])
    }
    fn sprite_camera(&self) -> [u16; 2] {
        SPRITE_CAMERA.map(|at| {
            u16::from_le_bytes([self.compatibility_byte(at), self.compatibility_byte(at + 1)])
        })
    }
    fn projectile_sprite_cursor(&self, slot: u16) -> u16 {
        let at = projectile_sprite_address(slot);
        u16::from_ne_bytes([self.compatibility_byte(at), self.compatibility_byte(at + 1)])
    }
    fn echo_frame(&self) -> u16 {
        let game = self;
        sm_game::frame_display_word!(game.NmiFrameWordPair)
    }
    fn samus_pose(&self) -> u16 {
        let game = self;
        sm_game::sequence_word!(game.SAMUS_POSE)
    }
}

impl sm_game::sprite_output::MapHost for Game {
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
    fn update_beam_tiles_and_palette(&mut self) {
        crate::game::visual_runtime::update_beam_tiles_and_palette(self)
    }
    fn spawn_hdma(&mut self, bank: u8, args: &crate::types::HdmaObjectDefinition) -> u16 {
        sm_game::hdma_objects::spawn_hdma_owned(bank, args, self)
    }
    fn rtl_unreachable(&mut self) -> bool {
        crate::rtl::report_unreachable(self)
    }
    fn run_hdma_objects(&mut self) {
        // Closing the box ($85:8589) ends on a borrow, and nothing up to the handler sets C.
        crate::game::visual_runtime::update_hdma(false, self);
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
    fn cinematic_sprite_pre_instruction(
        &mut self,
        pre: u32,
        object: u16,
        cine: &mut sm_game::cinematic_state::Cinematic,
    ) {
        sm_game::cinematic_dispatch::run_sprite_pre_instruction(pre, object, cine, self)
    }
    fn cinematic_sprite_instruction(
        &mut self,
        command: u32,
        object: u16,
        operand: u16,
        cine: &mut sm_game::cinematic_state::Cinematic,
    ) -> u16 {
        sm_game::cinematic_states::run_sprite_instruction(command, object, operand, cine, self)
    }
    fn write_dma_register(&mut self, register: u16, value: u8) {
        crate::rtl::write_dma_register(register, value, self)
    }
    fn handle_music_queue(&mut self) {
        // SAFETY: HDMA updates run from the frame loop with the backend installed.
        unsafe { crate::game::music::handle_music_queue(self) }
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

impl sm_game::menus::host::ResetRequest for Game {
    fn request_reset(&mut self) {
        Game::request_reset(self)
    }
}

/// A save on the lease: the projected words and the save selection are its
/// Session's, and SRAM and its slot offsets the game's, which the lease does
/// not hold.
impl sm_game::menus::save_slots::SlotHost for Runtime<'_> {
    type SlotRam = SaveRam;
    fn slot_ram(&self) -> &SaveRam {
        &self.samus_game().g_sram
    }
    fn slot_offsets(&self) -> &Option<sm_game::menus::save_slots::ImportedSlots> {
        &self.samus_game().save_slots
    }
    fn flush_slots(&mut self) {
        // It reads only SRAM.
        unsafe { crate::rtl::rtl_write_sram(self.samus_game()) }
    }
    fn save_selection(&self) -> SaveSelection {
        self.session().menu_screens.save_selection
    }
    /// The store the game's persistent menu commit makes on its lease.
    fn commit_save_selection(&mut self, _before: &SaveSelection, after: SaveSelection) {
        self.session_mut().menu_screens.save_selection = after;
        self.refresh_inputs();
    }
    fn observe_save(&mut self, _stage: u8) {
        #[cfg(sm_verify_dll)]
        {
            let length = self.samus_game().ram().len();
            crate::flags_save_observer::observe_image(
                _stage,
                || (0..length).map(|at| self.byte(at)).collect(),
                &self.samus_game().g_sram,
            );
        }
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
            .unwrap_or_else(|| self.read_register_word(crate::rtl::JOY1L as u16))
    }
    fn controller_word(&mut self, controller: sm_game::menus::host::Controller) -> u16 {
        use sm_game::menus::host::Controller;
        self.read_register_word(match controller {
            Controller::One => crate::rtl::JOY1L,
            Controller::Two => crate::rtl::JOY2L,
        } as u16)
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

    fn advance_ending_tile_upload(&mut self, page: u16) {
        crate::game::mode7_transfers::advance_ending_tile_upload(page, self);
    }

    fn advance_intro_player(&mut self) {
        // Samus's projectiles and her frame run on one lease.
        let mut runtime = crate::game::session_runtime::Runtime::new(self);
        crate::game::player_projectiles::prepare(&mut runtime);
        crate::game::session_runtime::player_update_runtime::advance_frame(&mut runtime);
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
