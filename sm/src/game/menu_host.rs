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
        visual_runtime::clear_hdma(self)
    }
    fn clear_hdma_flags(&mut self) {
        sm_game::hdma_objects::clear_hdma_flags(self)
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
        crate::game::display_refresh::GraphicsAsset::VramWords(source, destination, bytes)
            .load(self)
    }
    fn file_select_fade(&mut self) -> u8 {
        sm_game::palette_effects::fading::file_select(self)
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
        crate::game::music::queue_music_delayed_8(track, self)
    }
    fn spawn_hdma(&mut self, bank: u8, args: &crate::types::HdmaObjectDefinition) -> u16 {
        sm_game::hdma_objects::spawn_hdma_owned(bank, args, self)
    }
    fn rtl_unreachable(&mut self) -> bool {
        crate::rtl::Unreachable(self)
    }
    fn spawn_bg3_scroll(&mut self) {
        sm_game::hdma_objects::spawn_bg3_scroll_owned(self)
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
        sm_game::cinematic_dispatch::run_sprite_pre_instruction(pre, object, self)
    }
    fn cinematic_sprite_instruction(&mut self, command: u32, object: u16, operand: u16) -> u16 {
        sm_game::cinematic_states::run_sprite_instruction(command, object, operand, self)
    }
    fn enqueue_mode7_rom(&mut self, bank: u8, cursor: u16) {
        crate::game::mode7_transfers::enqueue_rom(bank, cursor, self)
    }
    fn draw_player_frame(&mut self) {
        visual_runtime::draw_player_frame(self)
    }
    fn draw_projectiles(&mut self) {
        crate::game::player_projectiles::draw_projectiles(self)
    }
    fn event_happened(&mut self, event: u16) -> u16 {
        crate::game::gameplay_flag_runtime::event_has_happened(event, self)
    }
    fn set_event_happened(&mut self, event: u16) {
        crate::game::gameplay_flag_runtime::mark_event(event, self)
    }
    fn boss_bit_for_current_area(&mut self, bit: u16) -> u8 {
        crate::game::gameplay_flag_runtime::CheckBossBitForCurArea(bit, self)
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
    fn update_layer_blending(&mut self) {
        sm_game::layer_blending::update(self)
    }
    fn handle_music_queue(&mut self) {
        // SAFETY: HDMA updates run from the frame loop with the backend installed.
        unsafe { crate::game::music::HandleMusicQueue(self) }
    }
    fn hdma_effect(&mut self, effect: sm_game::hdma_objects::HdmaEffect, slot: u16) {
        use sm_game::hdma_effects::HdmaEffects;
        use sm_game::hdma_objects::HdmaEffect::*;
        match effect {
            // SAFETY: HDMA runs from the frame loop with the backend installed.
            InitializeRainbowBeam => {
                return unsafe { crate::game::mother_brain_visuals::initialize_rainbow_beam(self) };
            }
            AdvanceRainbowBeam => {
                return unsafe { crate::game::mother_brain_visuals::advance_rainbow_beam(self) };
            }
            _ => {}
        }
        let mut runtime = Runtime::new(self);
        match effect {
            AdvanceProgram => runtime.advance_hdma_program(),
            InitializePhantoonWave => runtime.initialize_phantoon_wave(slot),
            InitializeEyeBeam => runtime.initialize_eye_beam(slot),
            ResetWave => runtime.reset_hdma_wave(slot),
            InitializeWavePair { base, primary } => runtime.initialize_wave_pair(base, primary),
            SetPhantoonMotion { direction, speed } => runtime.set_phantoon_motion(direction, speed),
            ScrollWeather(weather) => runtime.scroll_weather_layers(slot, weather),
            BeginHaze { color } => runtime.begin_haze(slot, color),
            AdvanceHaze(phase) => runtime.advance_haze(slot, phase),
            SelectDraygonLayer => runtime.select_draygon_layer(slot),
            SelectPhantoonLayer => runtime.select_phantoon_layer(),
            AdvanceWave(wave) => runtime.advance_hdma_wave(slot, wave),
            AdvanceEyeBeam(phase) => runtime.advance_eye_beam(slot, phase),
            AdvanceXrayBackground => runtime.advance_xray_background(),
            AdvanceXrayBeam(phase) => runtime.advance_xray_beam(phase),
            InitializeSuitPickup => runtime.initialize_suit_pickup_hdma(),
            AdvanceSuitPickup { gravity } => runtime.advance_suit_pickup(gravity),
            InitializeRainbowBeam | AdvanceRainbowBeam => unreachable!(),
        }
    }
    fn retire_projectile(&mut self, slot: u16) {
        crate::game::player_projectiles::retire(slot, self)
    }
    fn spawn_palette_effect(&mut self, definition: u16) {
        crate::game::palette_effects::spawn(definition, self)
    }
    fn enable_display_nmi(&mut self, enabled: bool) {
        crate::game::display_refresh::enable(enabled, self)
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
    fn beam_gradients(&self) -> &sm_game::beam_windows::ImportedTables {
        &self.beam_gradients
    }
    fn read_rom_word(&self, address: u32, offset: usize) -> u16 {
        Game::read_rom_word(self, address, offset)
    }
    fn print_carry_message(&mut self) {
        // SAFETY: a NUL-terminated literal through the C library's printf.
        unsafe { crate::c_library::printf(c"Carry crap!\n".as_ptr()) };
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
    use crate::game::constants::bank_8b as bank;
    use sm_game::intro_pages as intro;
    assert!(
        intro::ADDR_CINEMATIC_BG_OBJECT_DEF_8BCF3F
            == bank::ADDR_K_CINEMATIC_BG_OBJECT_DEF_8BCF3F as u32
    );
    assert!(
        intro::ADDR_CINEMATIC_BG_OBJECT_DEF_8BCF5D
            == bank::ADDR_K_CINEMATIC_BG_OBJECT_DEF_8BCF5D as u32
    );
    assert!(
        intro::ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE5B
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE5B as u32
    );
    assert!(
        intro::ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE55
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE55 as u32
    );
    assert!(
        intro::ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE61
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE61 as u32
    );
    assert!(
        intro::ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE67
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE67 as u32
    );
    assert!(
        intro::ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE73
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE73 as u32
    );
    assert!(
        intro::ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE79
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE79 as u32
    );
    assert!(
        intro::ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCF27
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCF27 as u32
    );
};

// The cinematic engine's named bank $8B definitions must match.
const _: () = {
    use crate::game::constants::bank_8b as bank;
    use sm_game::ending_screens::samus as s;
    use sm_game::ending_sequence::object as e;
    use sm_game::intro_sequence::{self as i, mode7 as m};
    use sm_game::title_sequence::object as t;
    assert!(t::COPYRIGHT_FADE_IN == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BA113 as u16);
    assert!(
        i::object::PLANET_ZEBES_JAPANESE_TEXT
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCEB5 as u16
    );
    assert!(i::ZEBES_TEXT_FADE_OUT == bank::ADDR_LOC_8BE1B4 as u16);
    assert!(m::JAPANESE_TEXT_PAGE_1 == bank::ADDR_K_MODE7_OBJECT_DEF_8BD401 as u16);
    assert!(m::JAPANESE_TEXT_PAGE_2 == bank::ADDR_K_MODE7_OBJECT_DEF_8BD407 as u16);
    assert!(m::JAPANESE_TEXT_PAGE_3 == bank::ADDR_K_MODE7_OBJECT_DEF_8BD413 as u16);
    assert!(m::JAPANESE_TEXT_PAGE_4 == bank::ADDR_K_MODE7_OBJECT_DEF_8BD41F as u16);
    assert!(m::JAPANESE_TEXT_PAGE_5 == bank::ADDR_K_MODE7_OBJECT_DEF_8BD42B as u16);
    assert!(m::JAPANESE_TEXT_PAGE_6 == bank::ADDR_K_MODE7_OBJECT_DEF_8BD437 as u16);
    assert!(e::ZEBES_SILHOUETTE == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEB5 as u16);
    assert!(e::ZEBES_STARS_RIGHT == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEBB as u16);
    assert!(e::ZEBES_AFTERGLOW == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEC1 as u16);
    assert!(e::COMPLETED_SUCCESSFULLY == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEECD as u16);
    assert!(e::ZEBES_STARS_LEFT == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEF7 as u16);
    assert!(e::CLEAR_TIME == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEFD as u16);
    assert!(e::CLEAR_TIME_DIGITS == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF03 as u16);
    assert!(s::SUITLESS_IDLE == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF27 as u16);
    assert!(s::SUITLESS_IDLE_LEGS == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF2D as u16);
    assert!(
        s::SUITLESS_LETTING_HAIR_DOWN == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF33 as u16
    );
    assert!(
        s::SUITLESS_LETTING_HAIR_DOWN_LEGS
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF39 as u16
    );
    assert!(s::SUITLESS_JUMP == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF3F as u16);
    assert!(s::SUITED_IDLE_BODY == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF45 as u16);
    assert!(
        s::SUITED_IDLE_HEAD_WITH_HELMET == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF4B as u16
    );
    assert!(
        s::SUITED_IDLE_HEAD_WITHOUT_HELMET
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF51 as u16
    );
    assert!(s::SUITED_THUMBS_UP_BODY == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF57 as u16);
    assert!(s::SUITED_THUMBS_UP_ARM == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF5D as u16);
    assert!(
        s::SUITED_THUMBS_UP_HEAD_WITH_HELMET
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF63 as u16
    );
    assert!(
        s::SUITED_THUMBS_UP_HEAD_WITHOUT_HELMET
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF69 as u16
    );
    assert!(s::SUITED_JUMP_BODY == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF6F as u16);
    assert!(
        s::SUITED_JUMP_HEAD_WITH_HELMET == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF75 as u16
    );
    assert!(
        s::SUITED_JUMP_HEAD_WITHOUT_HELMET
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF7B as u16
    );
};

// The beam window's local scratch layout must match the decompilation's.
const _: () = assert!(
    core::mem::offset_of!(sm_game::beam_windows::BeamScratch, row)
        == core::mem::offset_of!(crate::types::Ram8000_Default, var_3f)
);

impl sm_game::cinematic_host::Host for Game {
    fn clear_palette_effects(&mut self) {
        crate::game::palette_effects::clear(self);
    }

    fn set_palette_effects_enabled(&mut self, enabled: bool) {
        crate::game::palette_effects::set_enabled(enabled, self);
    }

    fn queue_music_delayed(&mut self, track: u16, delay: u16) {
        crate::game::music::QueueMusic_DelayedY(track, delay, self);
    }

    fn spawn_power_bomb(&mut self) {
        sm_game::explosion_window::spawn_power_bomb(self);
    }

    fn initialize_player(&mut self) {
        crate::game::session_runtime::initialize_player(self);
    }

    fn use_cinematic_bindings(&mut self) {
        sm_game::menus::controller_input::use_cinematic_bindings(self);
    }

    fn disable_scanline_irq(&mut self) {
        crate::game::scanline_irq::disable(self);
    }

    fn advance_ending_tile_upload(&mut self) {
        crate::game::mode7_transfers::advance_ending_tile_upload(self);
    }

    fn advance_palette_effects(&mut self) {
        crate::game::palette_effects::advance(self);
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

// The named bank $8B functions, objects and lists are the decompilation's.
const _: () = {
    use crate::game::constants::bank_8b::*;
    use sm_game::intro_sequence as i;
    use sm_game::title_sequence as t;
    const fn at(word: u16) -> u32 {
        0x8b_0000 | word as u32
    }
    assert!(at(t::function::NOTHING) == 0x8b9a47);
    assert!(at(t::function::SETUP) == 0x8b9b68);
    assert!(at(t::function::ZOOMING_OUT) == 9150091 && at(t::function::TO_FILE_SELECT) == 9150290);
    assert!(at(t::function::TO_DEMOS) == 9150382 && at(t::function::WAITING) == 9150248);
    assert!(at(t::function::TITLE_SCREEN) == 9150249 && at(t::NO_PRE_INSTRUCTION) == 9147353);
    assert!(t::object::SCROLLING_1994 == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BA0EF as u16);
    assert!(t::object::TITLE_LOGO == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BA107 as u16);
    assert!(t::object::COPYRIGHT == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BA119 as u16);
    assert!(t::object::DEMO_LOGO == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BA125 as u16);
    assert!(t::object::BABY_METROID == ADDR_K_MODE7_OBJ_BABY_METROID_IN_TITLE as u16);
    assert!(
        at(i::function::WAIT_FOR_MUSIC_AND_FADE_IN) == 0x8ba5a7
            && at(i::function::PAGE_6_DRAWN) == 0x8ba38f
    );
    assert!(
        at(i::function::GALAXY_IS_AT_PEACE) == 9152019
            && at(i::function::WAIT_240_FRAMES) == 9152057
    );
    assert!(
        at(i::function::WAIT_FOR_FADE_OUT) == 9152099
            && at(i::function::SPAWN_TEXT_PAGE_1) == 9152555
    );
    assert!(
        at(i::function::CERES_WAIT_FOR_MUSIC) == 9158116
            && at(i::function::CERES_FLYING_INTO_CAMERA) == 9158137
    );
    assert!(
        at(i::function::CERES_FLYING_INTO_CERES) == 9158618
            && at(i::function::BOOM_WAIT_FOR_MUSIC) == 9159396
    );
    assert!(
        at(i::function::BOOM_SMALL_EXPLOSIONS) == 9159409
            && at(i::function::BOOM_EXPLOSIONS) == 9159493
    );
    assert!(
        at(i::function::BOOM_GUNSHIP_FLYING_AWAY) == 9160138
            && at(i::function::BOOM_WAIT_C0H_FRAMES) == 9160208
    );
    assert!(at(i::function::BOOM_FADE_OUT) == 9160231 && at(i::function::ZEBES_INITIAL) == 9160345);
    assert!(
        at(i::function::ZEBES_FADING_IN) == 9160604
            && at(i::function::ZEBES_SLIDE_SCENE_AWAY) == 9161438
    );
    assert!(
        at(i::pre_instruction::BABY_HATCHED) == 9157235
            && at(i::pre_instruction::BABY_IDLING) == 9157389
    );
    assert!(
        at(i::pre_instruction::BABY_DANCING) == 9157412
            && at(i::pre_instruction::NOTHING) == 9147353
    );
    assert!(i::object::TEXT_CARET == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE6D as u16);
    assert!(i::object::CERES_GUNSHIP == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE7F as u16);
    assert!(i::object::CERES_UNDER_ATTACK == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE85 as u16);
    assert!(i::object::CERES_SMALL_ASTEROIDS == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE8B as u16);
    assert!(i::object::CERES_VORTEX == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE91 as u16);
    assert!(i::object::CERES_EXPLOSION_2 == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCEC1 as u16);
    assert!(i::object::SLIME_DROPS == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCEF1 as u16);
    assert!(i::object::CERES_STARS == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCF0F as u16);
    assert!(i::object::CERES_EXPLOSION_GUNSHIP == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCF2D as u16);
    assert!(i::object::CERES_EXPLOSION_SPAWNER == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCF33 as u16);
    assert!(i::object::CERES_LARGE_ASTEROIDS == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCF39 as u16);
    assert!(i::object::SAMUS_BLINKING == ADDR_K_CINEMATIC_BG_OBJECT_DEF_8BCF63 as u16);
    assert!(i::object::CERES_BG == ADDR_K_CINEMATIC_BG_OBJECT_DEF_8BCF69 as u16);
    assert!(i::mode7::FLYING_INTO_CERES == ADDR_K_CINEMATIC_FUNCTION_INTRO_FUNC56_M7 as u16);
    assert!(i::mode7::BOOM == ADDR_K_CINEMATIC_FUNCTION_INTRO_FUNC76_M7_0 as u16);
    assert!(i::mode7::BOOM_GUNSHIP == ADDR_K_CINEMATIC_FUNCTION_INTRO_FUNC76_M7_1 as u16);
    assert!(i::mode7::BOOM_WITH_SAMUS == ADDR_K_CINEMATIC_FUNCTION_INTRO_FUNC76_M7_2 as u16);
    assert!(
        i::NEXT_PAGE_ARROW
            == 0x3000 + core::mem::offset_of!(crate::types::Ram3000, menu.menu_tilemap) + 286
    );
};

const _: () = {
    use crate::game::constants::bank_8b::*;
    use sm_game::ending_sequence::{function as f, object as o};
    const fn at(word: u16) -> u32 {
        0x8b_0000 | word as u32
    }
    assert!(at(f::WAIT_FOR_MUSIC) == 9164503 && at(f::SCENE_0_FADING_IN) == 9164530);
    assert!(at(f::SCENE_1_FADING_IN) == 9164792 && at(f::SCENE_2_FADING_IN) == 9165314);
    assert!(at(f::SCENE_2) == 9165338 && at(f::LOAD_GUNSHIP_BG) == 9165523);
    assert!(at(f::CREDITS) == 9165725 && at(f::GUNSHIP_SETUP) == 9165764);
    assert!(at(f::GUNSHIP_SPINNING_FAST) == 9165989 && at(f::GUNSHIP_FREEZE_FRAME) == 9166435);
    assert!(at(f::CREDITS_SETUP) == 9166464 && at(f::FADE_IN_SHOOTING_STARS) == 9167120);
    assert!(at(f::FADE_IN_SAMUS) == 9167192 && at(f::WAVY_SAMUS) == 9167248);
    assert!(at(f::DEER_FORCE) == 9167314 && at(f::IDLE_SAMUS_2_CROSS_FADE) == 9167636);
    assert!(at(f::IDLE_SAMUS_2) == 9167682 && at(f::FINISHED) == 9168826);
    assert!(o::EXPLODING_ZEBES == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEE9D as u16);
    assert!(o::PURPLE_GLOW == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEA3 as u16);
    assert!(o::STARS == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEA9 as u16);
    assert!(o::LAVA == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEAF as u16);
    assert!(o::GUNSHIP_TEXT == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEC7 as u16);
    assert!(o::ITEM_PERCENTAGE == ADDR_K_CINEMATIC_BG_OBJECT_DEF_8BF748 as u16);
    assert!(
        o::ICON == ADDR_K_CINEMATIC_BG_OBJECT_DEF_8BF74E as u16
            && o::CREDITS == ADDR_STRU_8BF6F8 as u16
    );
};
