//! Intro narrative pages, scene entry, crossfades and Japanese text continuations.
//! Numeric callbacks select these operations; Game retains the cinematic state.
use crate::game::player_transitions as pt;
use crate::game::session_runtime::Runtime;
use crate::game::visual_state::CinematicObjects::{
    Background as CinematicBackground, Mode7 as CinematicMode7, Sprites as CinematicSprites,
};

use super::constants::bank_8d::addr_kPalfx_E1BC;
use super::constants::bank_91::{addr_stru_91877E, addr_stru_918784};
use super::constants::samus_poses::kPose_02_FaceL_Normal;
use super::palette_effects;
use super::sm_8b::{
    CinematicBgInstr_SetSomeStuffForSpriteObject_14, RevertButtonConfig,
    SetSomeStuffForSpriteObject_15, SetSomeStuffForSpriteObject_16,
    addr_kCinematicBgObjectDef_8BCF5D, addr_kCinematicSpriteObjectDef_8BCE5B,
    addr_kCinematicSpriteObjectDef_8BCE55, addr_kCinematicSpriteObjectDef_8BCE61,
    addr_kCinematicSpriteObjectDef_8BCE67, addr_kCinematicSpriteObjectDef_8BCE73,
    addr_kCinematicSpriteObjectDef_8BCE79, addr_kCinematicSpriteObjectDef_8BCF27,
};
use super::visual_runtime::{
    clear_cinematic_colors, compose_cinematic_palette, fade_cinematic_colors,
    initialize_intro_crossfade, prepare_cinematic_palette_fade, queue_cinematic_tilemap,
    set_cinematic_tilemap_updates, spawn_cinematic_sprite_at,
};
use super::visual_state::CinematicTilemap;
use crate::game::music::QueueMusic_Delayed8;
use crate::game_state::Game;
use crate::ram_map::{checked_word, ram_ptr, word};

unsafe fn load_japanese_line(j: u16, game: &mut Game) {
    unsafe {
        let mut tt = crate::rtl::rom_bank_ptr::<0x8b>(j, game) as *mut u16;
        let mut v2: u16 = *tt;
        let mut r18: i32 = *tt.offset(1) as i32;
        let dst = &raw mut (*(ram_ptr!(game, crate::types::Ram4000; 0x4000)))
            .intro_japanese_text_tiles as *mut u16;
        do_until! { r18 == 0 ;
            crate::c_library::memcpy(
                dst.offset((v2 >> 1) as isize) as *mut core::ffi::c_void,
                ram_ptr!(game, u16; (*tt.offset(2) as i32 + 0x1a000) as isize) as *const core::ffi::c_void,
                16 as usize,
            );
            crate::c_library::memcpy(
                dst.offset((v2 as i32 + 768 >> 1) as isize) as *mut core::ffi::c_void,
                ram_ptr!(game, u16; (*tt.offset(3) as i32 + 0x1a000) as isize) as *const core::ffi::c_void,
                16 as usize,
            );
            v2 = (v2).wrapping_add(16);
            tt = tt.offset(2);
            r18 -= 1;
        }
    }
}

pub(super) fn clear_english_text(game: &mut Game) {
    word_transfers!(game, ascending, 640; pause_map(128) <- value(47));
}
pub(super) fn clear_japanese_tiles(game: &mut Game) {
    for start in (0..768).step_by(8) {
        word_transfers!(game, ascending, 8; xray(start) <- native(0x18290));
    }
}

// Narrative callbacks retain their ROM addresses; these functions own the shared
// page/scene lifecycle, including the Japanese continuation and input delay.
pub(super) unsafe fn create_japanese_text(object: u16, j: u16, game: &mut Game) -> u16 {
    unsafe {
        word!(game.cinematicbg_var1 = 0);
        if word!(game.japanese_text_flag) != 0 {
            CinematicMode7.spawn(object, word!(game.japanese_text_flag), game);
        }
        j
    }
}

pub(super) unsafe fn spawn_marker(
    next: u16,
    continuation: Option<u16>,
    k: u16,
    j: u16,
    game: &mut Game,
) -> u16 {
    unsafe {
        let j = CinematicBgInstr_SetSomeStuffForSpriteObject_14(k, j, game);
        if word!(game.japanese_text_flag) != 0 {
            let failed = CinematicSprites.spawn(
                addr_kCinematicSpriteObjectDef_8BCE73 as u16,
                word!(game.japanese_text_flag),
                game,
            ) != 0;
            if let Some(object) = continuation {
                // The arrow leaves its palette in A; a full allocator leaves slot zero's instruction.
                let parameter = if failed {
                    word!(game.cinematicspr_instr_ptr[0])
                } else {
                    0x0c00
                };
                CinematicMode7.spawn(object, parameter, game);
                return j;
            }
        }
        word!(game.cinematic_function = next);
        j
    }
}

fn page_input_ready(game: &mut Game) -> bool {
    {
        if word!(game.cinematic_var16) != 0 {
            word!(game.cinematic_var16 -= 1);
            false
        } else {
            word!(game.joypad1_newkeys) != 0
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum Scene {
    MotherBrain,
    BabyMetroid,
    ScientistPage3,
    ScientistPage4,
}

#[derive(Clone, Copy)]
pub(super) enum Palette {
    Gameplay,
    Scientist,
}

impl Palette {
    fn colors(self) -> &'static [(u16, u16)] {
        match self {
            Self::Gameplay => &[(0x28, 3), (0xe0, 0x10), (0x180, 0x20), (0x1e0, 0x10)],
            Self::Scientist => &[(0x40, 0x10), (0x1c0, 9)],
        }
    }
}

pub(super) unsafe fn wait_for_scene(scene: Scene, game: &mut Game) {
    unsafe {
        if !page_input_ready(game) {
            return;
        }
        let palette = match scene {
            Scene::MotherBrain | Scene::BabyMetroid => {
                let mother_brain = matches!(scene, Scene::MotherBrain);
                *ram_ptr!(game; 0x58) = if mother_brain { 80 } else { 84 };
                if !mother_brain {
                    word! {game; room_width_in_blocks = 32; room_height_in_blocks = 16; }
                }
                word!(game.samus_pose = kPose_02_FaceL_Normal as u16);
                pt::finish_pose_change(&mut Runtime::new(game));
                let (x, y) = if mother_brain { (155, 115) } else { (376, 147) };
                word! {game; samus_x_pos = x; samus_prev_x_pos = x; samus_y_pos = y;
                samus_prev_y_pos = y; cinematic_var8 = 0; cinematic_var13 = 127; }
                if mother_brain {
                    // The preceding $1A4B store leaves $007F in the ROM accumulator.
                    spawn_cinematic_sprite_at(
                        addr_kCinematicSpriteObjectDef_8BCE55 as u16,
                        0,
                        0x7f,
                        game,
                    );
                    // Palette left in A by the first sprite initializer.
                    CinematicSprites.spawn(
                        addr_kCinematicSpriteObjectDef_8BCF27 as u16,
                        0x0e00,
                        game,
                    );
                    word_transfers!(game, ascending, 224; le(level_data, 0) <- rom(0x8cbec3, 0));
                    word_transfers!(game, ascending, 256; native(0x16402) <- value(0));
                    word!(game.hud_item_index = 1);
                } else {
                    CinematicSprites.spawn(
                        addr_kCinematicSpriteObjectDef_8BCE5B as u16,
                        0x7f,
                        game,
                    );
                    CinematicSprites.spawn(
                        addr_kCinematicSpriteObjectDef_8BCE79 as u16,
                        0x0e00,
                        game,
                    );
                    word!(game.cinematic_var11 = 0);
                    word_transfers!(game, ascending, 384; le(level_data, 0) <- rom(0x8cc083, 0));
                    word!(game.hud_item_index = 0);
                }
                word!(game.frame_handler_alfa = (0x90e6c9i32) as u16);
                word!(game.frame_handler_beta = (0x90e833i32) as u16);
                pt::demo::start(Runtime::new(game).session_mut());
                let input = if mother_brain {
                    addr_stru_918784
                } else {
                    addr_stru_91877E
                };
                // EnableDemoInput leaves $8000 in the ROM accumulator for the loader.
                pt::demo::load(&mut Runtime::new(game), 0x8000, input as u16);
                word!(game.cinematic_var15 = if mother_brain { u16::MAX } else { 1 });
                Palette::Gameplay
            }
            Scene::ScientistPage3 | Scene::ScientistPage4 => {
                let scientist = matches!(scene, Scene::ScientistPage3);
                *ram_ptr!(game; 0x58) = if scientist { 88 } else { 92 };
                word!(game.cinematic_var8 = if scientist { 32 } else { 0 });
                word!(game.cinematic_var10 = if scientist { 8 } else { -24i16 as u16 });
                word!(game.cinematic_var13 = 127);
                let object = if scientist {
                    addr_kCinematicSpriteObjectDef_8BCE61
                } else {
                    addr_kCinematicSpriteObjectDef_8BCE67
                };
                CinematicSprites.spawn(object as u16, 0x7f, game);
                Palette::Scientist
            }
        };
        *ram_ptr!(game; 0x69) = 6;
        *ram_ptr!(game; 0x6b) = 17;
        *ram_ptr!(game; 0x6f) = 2;
        *ram_ptr!(game; 0x72) = 0;
        initialize_intro_crossfade(game);
        SetSomeStuffForSpriteObject_15(game);
        word!(
            game.cinematic_function = match palette {
                Palette::Gameplay => 0xb250,
                Palette::Scientist => 0xb2d2,
            }
        );
        word_transfers!(game, ascending, 256; native(0x2200) <- rom(0x8ce3e9, 0));
        prepare_cinematic_palette_fade(game);
        for &(index, count) in palette.colors() {
            clear_cinematic_colors(index, count, game);
        }
        compose_cinematic_palette(game);
    }
}

pub(super) unsafe fn fade_to_scene(palette: Palette, game: &mut Game) {
    unsafe {
        if (word!(game.cinematic_var4) & 3) == 0 {
            fade_cinematic_colors(0, 0x14, false, game);
            fade_cinematic_colors(0x60, 0x10, false, game);
            fade_cinematic_colors(0x1d2, 6, false, game);
            for &(index, count) in palette.colors() {
                fade_cinematic_colors(index, count, true, game);
            }
            compose_cinematic_palette(game);
        }
        if (word!(game.cinematic_var4 -= 1, reread) & 0x8000) != 0 {
            *ram_ptr!(game; 0x69) = 21;
            *ram_ptr!(game; 0x6b) = 0;
            *ram_ptr!(game; 0x6f) = 0;
            word_transfers!(game, ascending, 640; pause_map(128) <- value(47));
            clear_japanese_tiles(game);
            word!(game.cinematic_function = (0x8ba391i32) as u16);
            if matches!(palette, Palette::Gameplay) && (word!(game.cinematic_var15) & 0x8000) != 0 {
                palette_effects::spawn(addr_kPalfx_E1BC as u16, game);
            }
        }
    }
}

pub(super) fn start_page(next: u16, j: u16, game: &mut Game) -> u16 {
    word!(game.cinematic_function = next);
    set_cinematic_tilemap_updates(false, game);
    clear_japanese_tiles(game);
    queue_cinematic_tilemap(CinematicTilemap::Japanese, game);
    j
}

pub(super) unsafe fn show_page(object: u16, next: u16, game: &mut Game) {
    unsafe {
        CinematicBackground.spawn(object, 0x4c00, game);
        word!(game.cinematic_function = next);
        setup_page_palettes(game);
    }
}

pub(super) unsafe fn fade_to_page(palette: Palette, game: &mut Game) {
    unsafe {
        if (word!(game.cinematic_var13) & 3) == 0 {
            fade_cinematic_colors(0, 0x10, true, game);
            fade_cinematic_colors(0x60, 0x10, true, game);
            fade_cinematic_colors(0x1d2, 6, true, game);
            for &(index, count) in palette.colors() {
                fade_cinematic_colors(index, count, false, game);
            }
            compose_cinematic_palette(game);
        }
        if (word!(game.cinematic_var13 -= 1, reread) & 0x8000) != 0 {
            *ram_ptr!(game; 0x69) = 22;
            *ram_ptr!(game; 0x6b) = 0;
            *ram_ptr!(game; 0x6f) = 0;
            *ram_ptr!(game; 0x72) = 0;
            if matches!(palette, Palette::Scientist) {
                word_transfers!(game, ascending, 7; native(0xc1c0) <- rom(0x8ce3e9, 224));
            }
            word!(game.cinematic_function = (0x8ba391i32) as u16);
        }
    }
}

pub(super) unsafe fn draw_japanese_text(
    lines: &[u16],
    restore_palette: bool,
    j: u16,
    game: &mut Game,
) -> u16 {
    unsafe {
        set_cinematic_tilemap_updates(false, game);
        if restore_palette {
            word!(game.palette_buffer[17] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 17));
            word!(game.palette_buffer[18] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 18));
            word!(game.palette_buffer[19] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 19));
        }
        clear_japanese_tiles(game);
        for &line in lines {
            load_japanese_line(line, game);
        }
        queue_cinematic_tilemap(CinematicTilemap::Japanese, game);
        j
    }
}

pub(super) unsafe fn continue_japanese_text(next: u16, lines: &[u16], k: u16, game: &mut Game) {
    unsafe {
        if word!(game.joypad1_newkeys) != 0 {
            let slot = k as usize >> 1;
            word! {game; mode7_obj_instr_ptr[slot] = next; mode7_obj_instr_timer[slot] = 1;
            mode7_obj_preinstr_func[slot] = ((0x8b93d9i32) as u16); }
            draw_japanese_text(lines, false, 0, game);
            word!(game.cinematic_var16 = 60);
        }
    }
}

pub(super) fn wait_after_japanese_text(next: u16, j: u16, game: &mut Game) -> u16 {
    set_cinematic_tilemap_updates(true, game);
    word!(game.cinematic_function = next);
    j
}

pub(super) unsafe fn show_final_page(game: &mut Game) {
    unsafe {
        set_cinematic_tilemap_updates(true, game);
        clear_english_text(game);
        CinematicBackground.spawn(addr_kCinematicBgObjectDef_8BCF5D as u16, 0x4c00, game);
        word!(game.cinematic_function = (0x8ba390i32) as u16);
        SetSomeStuffForSpriteObject_16(game);
        QueueMusic_Delayed8(0, game);
    }
}

pub(super) unsafe fn wait_for_final_page(game: &mut Game) {
    unsafe {
        if page_input_ready(game) {
            word!(game.cinematic_var13 = 127);
            if word!(game.japanese_text_flag) != 0 {
                word!(game.cinematic_function = (0x8bb207i32) as u16);
                set_cinematic_tilemap_updates(false, game);
                clear_japanese_tiles(game);
                queue_cinematic_tilemap(CinematicTilemap::Japanese, game);
            } else {
                show_final_page(game);
            }
        }
    }
}

unsafe fn setup_page_palettes(game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x69) = 6;
        *ram_ptr!(game; 0x6b) = 17;
        *ram_ptr!(game; 0x6f) = 2;
        *ram_ptr!(game; 0x72) = 0;
        word_transfers!(game, ascending, 256; native(0x2200) <- rom(0x8ce3e9, 0));
        prepare_cinematic_palette_fade(game);
        clear_cinematic_colors(0, 0x10, game);
        clear_cinematic_colors(0x60, 0x10, game);
        clear_cinematic_colors(0x1d2, 6, game);
        compose_cinematic_palette(game);
        word!(game.cinematic_var4 = 127);
        set_cinematic_tilemap_updates(true, game);
        SetSomeStuffForSpriteObject_16(game);
    }
}

pub(super) fn enable_text(j: u16, game: &mut Game) -> u16 {
    set_cinematic_tilemap_updates(true, game);
    j
}

pub(super) unsafe fn fade_out_to_ceres(game: &mut Game) {
    unsafe {
        crate::game::display_refresh::fade(false, game);
        if *ram_ptr!(game; 0x51) == 0x80 {
            crate::game::display_refresh::enable(true, game);
            word! { game, 0;
                screen_fade_delay, screen_fade_counter, reg_BG2HOFS, reg_BG3HOFS, reg_BG2VOFS,
                reg_BG3VOFS, cinematic_var5,
            }
            word_transfers!(game, descending, 329; native(0x1c1d) <- value(0));
            word!(game.cinematic_function = (0x8bbca0i32) as u16);
            RevertButtonConfig(game);
            word! { game, 0; samus_max_missiles, samus_missiles }
        }
    }
}

pub(super) fn finish_pages(j: u16, game: &mut Game) -> u16 {
    {
        word!(game.cinematic_function = (0x8bb72fi32) as u16);
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        j
    }
}

pub(super) fn spawn_text_glow(definition: u16, position: u16, game: &mut Game) {
    for slot in (0..8usize).rev() {
        let offset = 2 * slot;
        if checked_word!(game, 0x19f7 + offset;) == 0 {
            word_transfers!(game, ascending, 1; native(0x19f7 + offset) <- value(definition));
            word! {game; eproj_pre_instr[slot + 2] = 1;
            eproj_pre_instr[slot + 10] = position as u8 as u16; }
            word_transfers!(game, ascending, 1;
                native(0x1a27 + offset) <- value(u16::from(position.to_ne_bytes()[1])),
                native(0x1a37 + offset) <- value(0),
            );
            return;
        }
    }
}

pub(super) unsafe fn advance_text_glow(game: &mut Game) {
    let mut object = 14i32;
    while object >= 0 {
        word!(game.cinematic_var19 = object as u16);
        if checked_word!(game, 0x19f7 + object as usize;) != 0 {
            unsafe {
                advance_text_glow_object(game);
            }
            object = i32::from(word!(game.cinematic_var19));
        }
        object -= 2;
    }
}

unsafe fn advance_text_glow_object(game: &mut Game) {
    let object = usize::from(word!(game.cinematic_var19));
    let timer_at = 0x1a07 + (object & !1);
    let timer = checked_word!(game, timer_at;);
    checked_word!(game, timer_at; = timer.wrapping_sub(1));
    if timer != 1 {
        return;
    }
    let palette = u16::from_ne_bytes([game.g_ram[0x1a37 + object], game.g_ram[0x1a38 + object]]);
    let definition = u16::from_ne_bytes([game.g_ram[0x19f7 + object], game.g_ram[0x19f8 + object]]);
    let mut row = 2 * u16::from(game.g_ram[0x1a17 + (object & !1)])
        + 64 * u16::from(game.g_ram[0x1a27 + object]);
    let (width, mut rows) = unsafe {
        let definition = crate::rtl::rom_bank_ptr::<0x8c>(definition, game);
        (i32::from(*definition.add(2)), i32::from(*definition.add(3)))
    };
    loop {
        let mut column = row;
        let mut remaining = width;
        loop {
            let at = 0x3000 + usize::from(column);
            let tile = u16::from_ne_bytes([game.g_ram[at], game.g_ram[at + 1]]);
            word_transfers!(game, ascending, 1; native(at) <- value(palette | (tile & 0xe3ff)));
            column = column.wrapping_add(2);
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }
        rows -= 1;
        if rows == 0 {
            break;
        }
        row = row.wrapping_add(64);
    }
    // Retirement rereads the live selector after the ordered tile writes.
    let selected = usize::from(word!(game.cinematic_var19));
    let phase_at = 0x1a37 + selected;
    if u16::from_ne_bytes([game.g_ram[phase_at], game.g_ram[phase_at + 1]]) == 3072 {
        word_transfers!(game, ascending, 1; native(0x19f7 + selected) <- value(0));
    } else {
        checked_word!(game, phase_at; = checked_word!(game, phase_at;).wrapping_add(1024));
        word_transfers!(game, ascending, 1; native(0x1a07 + (selected & !1)) <- value(5));
    }
}
