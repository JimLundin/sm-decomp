//! Intro narrative pages, scene entry, crossfades and Japanese text continuations.
//! Numeric callbacks select these operations; Game retains the cinematic state.
// The original's names and assignment shapes are kept.
#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_assignments)]
use crate::cinematic::CinematicLayer;
use crate::imported::Imported;
#[allow(unused_imports)]
use crate::ram::RawRam as _;

use crate::cinematic::{
    advance_cinematic_fade, clear_cinematic_colors, compose_cinematic_palette,
    fade_cinematic_colors, prepare_cinematic_palette_fade, queue_cinematic_tilemap,
    set_cinematic_blend, set_cinematic_tilemap_updates,
};
use crate::constants::bank_8d::addr_kPalfx_E1BC;
use crate::constants::bank_91::{addr_stru_91877E, addr_stru_918784};
use crate::constants::samus_poses::kPose_02_FaceL_Normal;
use crate::menus::prelude::{SpawnHdmaObject_Args, do_until};
use crate::ram::{checked_word, word};
use crate::visual_state::{CinematicFade, CinematicTilemap};
use sm_data::intro_palette::{
    ADDRESS as INTRO_PALETTE, BYTES as INTRO_PALETTE_BYTES, IntroPalette,
};

// Bank $8B cinematic object definitions (sm checks them against sm_8b).
pub const addr_kCinematicBgObjectDef_8BCF3F: u32 = 0xcf3f;
pub const addr_kCinematicBgObjectDef_8BCF5D: u32 = 0xcf5d;
pub const addr_kCinematicSpriteObjectDef_8BCE5B: u32 = 0xce5b;
pub const addr_kCinematicSpriteObjectDef_8BCE55: u32 = 0xce55;
pub const addr_kCinematicSpriteObjectDef_8BCE61: u32 = 0xce61;
pub const addr_kCinematicSpriteObjectDef_8BCE67: u32 = 0xce67;
pub const addr_kCinematicSpriteObjectDef_8BCE73: u32 = 0xce73;
pub const addr_kCinematicSpriteObjectDef_8BCE79: u32 = 0xce79;
pub const addr_kCinematicSpriteObjectDef_8BCF27: u32 = 0xcf27;

fn load_japanese_line<H: crate::menus::host::Host + ?Sized>(address: u16, game: &mut H) {
    let record =
        crate::menus::menu_tilemap::native_record((0x8bu32 << 16) | u32::from(address), game);
    let mut destination = record.word(0);
    let mut remaining = i32::from(record.word(2));
    let mut cursor = 4;
    do_until! { remaining == 0;
        // Finish each glyph copy before reading the next native source word.
        for (word_offset, row_offset) in [(0, 0), (2, 768)] {
            let target = 0x4000 + ((usize::from(destination) + row_offset) >> 1) * 2;
            let source = 0x1a000 + usize::from(record.word(cursor + word_offset));
            for byte in 0..16 {
                let value = game.ram().read_byte(source + byte);
                game.ram().write_byte(target + byte, value);
            }
        }
        destination = destination.wrapping_add(16);
        cursor += 4;
        remaining -= 1;
    }
}

#[forbid(unsafe_code)]
pub fn advance_scene_sprite<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    scene: u8,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    if scene == 0 {
        if word!(game.samus_x_pos) as i32 - 169 & 0x8000 != 0 {
            crate::cinematic::resume_cinematic_sprite(slot, 0xcb3b, game);
            word!(game.cinematicspr_preinstr_func[slot] = 0x93d9);
        }
    } else if scene == 1 {
        if word!(game.cinematic_var16) == 59 {
            crate::cinematic::resume_cinematic_sprite(slot, 0xcc23, game);
        }
        if word!(game.cinematic_var13) == 127 {
            crate::cinematic::resume_cinematic_sprite(slot, 0xce53, game);
        }
    } else if word!(game.cinematicspr_instr_ptr[15]) == 0 {
        crate::cinematic::resume_cinematic_sprite(slot, 0xee9b, game);
    }
}

pub fn wait_for_scene_end<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    stop_samus: bool,
    game: &mut H,
) {
    if word!(game.cinematic_var13) == 0 {
        crate::cinematic::resume_cinematic_sprite(usize::from(object >> 1), 0xce53, game);
        if stop_samus {
            word!(game.cinematic_var15 = 0);
        }
    }
}

pub fn advance_laboratory_baby<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    examined: bool,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    if word!(game.cinematic_var13) == 0 {
        crate::cinematic::resume_cinematic_sprite(slot, 0xce53, game);
    } else if word!(game.cinematic_var4) & 3 == 0 {
        if examined {
            if word!(game.cinematic_var10) as i32 - 8 & 0x8000 != 0 {
                word!(game.cinematic_var10 += 1);
                word!(game.cinematicbg_arr8[slot] -= 1);
            }
        } else if word!(game.cinematic_var8) != 0 {
            word!(game.cinematic_var8 -= 1);
            word!(game.cinematicbg_arr7[slot] += 1);
        }
    }
}

fn hide_caret<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    word!(game.cinematicbg_arr7[15] = 8);
    word!(game.cinematicbg_arr8[15] = 248);
}

pub fn reset_caret<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    word!(game.cinematicbg_arr7[15] = 8);
    word!(game.cinematicbg_arr8[15] = 24);
    word!(game.cinematicspr_instr_ptr[15] = 0xcbfb);
    word!(game.cinematicspr_instr_timer[15] = 1);
}

pub fn clear_english_text<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    crate::word_transfers!(game, ascending, 640; pause_map(128) <- value(47));
}
pub fn clear_japanese_tiles<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    for start in (0..768).step_by(8) {
        crate::word_transfers!(game, ascending, 8; xray(start) <- native(0x18290));
    }
}

// Narrative callbacks retain their ROM addresses; these functions own the shared
// page/scene lifecycle, including the Japanese continuation and input delay.
pub fn create_japanese_text<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    j: u16,
    game: &mut H,
) -> u16 {
    word!(game.cinematicbg_var1 = 0);
    if word!(game.japanese_text_flag) != 0 {
        game.spawn_cinematic_object(
            CinematicLayer::Mode7,
            object,
            word!(game.japanese_text_flag),
        );
    }
    j
}

pub fn reset_text_marker<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    word!(game.cinematicspr_instr_ptr[15] = 0xcc03);
    word!(game.cinematicspr_instr_timer[15] = 1);
}

pub fn spawn_marker<H: crate::menus::host::Host + ?Sized>(
    next: u16,
    continuation: Option<u16>,
    j: u16,
    game: &mut H,
) -> u16 {
    reset_text_marker(game);
    if word!(game.japanese_text_flag) != 0 {
        let failed = game.spawn_cinematic_object(
            CinematicLayer::Sprites,
            addr_kCinematicSpriteObjectDef_8BCE73 as u16,
            word!(game.japanese_text_flag),
        ) != 0;
        if let Some(object) = continuation {
            // The arrow leaves its palette in A; a full allocator leaves slot zero's instruction.
            let parameter = if failed {
                word!(game.cinematicspr_instr_ptr[0])
            } else {
                0x0c00
            };
            game.spawn_cinematic_object(CinematicLayer::Mode7, object, parameter);
            return j;
        }
    }
    word!(game.cinematic_function = next);
    j
}

fn page_input_ready<H: crate::menus::host::Host + ?Sized>(game: &mut H) -> bool {
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
pub enum Scene {
    MotherBrain,
    BabyMetroid,
    ScientistPage3,
    ScientistPage4,
}

#[derive(Clone, Copy)]
pub enum Palette {
    Gameplay,
    Scientist,
}

impl Palette {
    fn crossfade<H: crate::menus::host::Host + ?Sized>(self, to_page: bool, game: &mut H) {
        fade_cinematic_colors(0, if to_page { 0x10 } else { 0x14 }, to_page, game);
        fade_cinematic_colors(0x60, 0x10, to_page, game);
        fade_cinematic_colors(0x1d2, 6, to_page, game);
        for &(index, count) in self.colors() {
            fade_cinematic_colors(index, count, !to_page, game);
        }
        compose_cinematic_palette(game);
    }

    fn colors(self) -> &'static [(u16, u16)] {
        match self {
            Self::Gameplay => &[(0x28, 3), (0xe0, 0x10), (0x180, 0x20), (0x1e0, 0x10)],
            Self::Scientist => &[(0x40, 0x10), (0x1c0, 9)],
        }
    }
}

pub fn wait_for_scene<H: crate::menus::host::Host + ?Sized>(scene: Scene, game: &mut H) {
    if !page_input_ready(game) {
        return;
    }
    let palette = match scene {
        Scene::MotherBrain | Scene::BabyMetroid => {
            let mother_brain = matches!(scene, Scene::MotherBrain);
            game.ram()
                .write_byte(0x58, if mother_brain { 80 } else { 84 });
            if !mother_brain {
                word! {game; room_width_in_blocks = 32; room_height_in_blocks = 16; }
            }
            word!(game.samus_pose = kPose_02_FaceL_Normal as u16);
            game.finish_pose_change();
            let (x, y) = if mother_brain { (155, 115) } else { (376, 147) };
            word! {game; samus_x_pos = x; samus_prev_x_pos = x; samus_y_pos = y;
            samus_prev_y_pos = y; cinematic_var8 = 0; cinematic_var13 = 127; }
            if mother_brain {
                // The preceding $1A4B store leaves $007F in the ROM accumulator.
                game.spawn_cinematic_sprite_at(
                    addr_kCinematicSpriteObjectDef_8BCE55 as u16,
                    0,
                    0x7f,
                );
                // Palette left in A by the first sprite initializer.
                game.spawn_cinematic_object(
                    CinematicLayer::Sprites,
                    addr_kCinematicSpriteObjectDef_8BCF27 as u16,
                    0x0e00,
                );
                crate::word_transfers!(game, ascending, 224; le(level_data, 0) <- rom(0x8cbec3, 0));
                crate::word_transfers!(game, ascending, 256; native(0x16402) <- value(0));
                word!(game.hud_item_index = 1);
            } else {
                game.spawn_cinematic_object(
                    CinematicLayer::Sprites,
                    addr_kCinematicSpriteObjectDef_8BCE5B as u16,
                    0x7f,
                );
                game.spawn_cinematic_object(
                    CinematicLayer::Sprites,
                    addr_kCinematicSpriteObjectDef_8BCE79 as u16,
                    0x0e00,
                );
                word!(game.cinematic_var11 = 0);
                crate::word_transfers!(game, ascending, 384; le(level_data, 0) <- rom(0x8cc083, 0));
                word!(game.hud_item_index = 0);
            }
            word!(game.frame_handler_alfa = (0x90e6c9i32) as u16);
            word!(game.frame_handler_beta = (0x90e833i32) as u16);
            game.demo_start();
            let input = if mother_brain {
                addr_stru_918784
            } else {
                addr_stru_91877E
            };
            // EnableDemoInput leaves $8000 in the ROM accumulator for the loader.
            game.demo_load(0x8000, input as u16);
            word!(game.cinematic_var15 = if mother_brain { u16::MAX } else { 1 });
            Palette::Gameplay
        }
        Scene::ScientistPage3 | Scene::ScientistPage4 => {
            let scientist = matches!(scene, Scene::ScientistPage3);
            game.ram().write_byte(0x58, if scientist { 88 } else { 92 });
            word!(game.cinematic_var8 = if scientist { 32 } else { 0 });
            word!(game.cinematic_var10 = if scientist { 8 } else { -24i16 as u16 });
            word!(game.cinematic_var13 = 127);
            let object = if scientist {
                addr_kCinematicSpriteObjectDef_8BCE61
            } else {
                addr_kCinematicSpriteObjectDef_8BCE67
            };
            game.spawn_cinematic_object(CinematicLayer::Sprites, object as u16, 0x7f);
            Palette::Scientist
        }
    };
    set_cinematic_blend(6, 17, 2, 0, game);
    game.spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0, 0x31, 0xec03));
    hide_caret(game);
    word!(
        game.cinematic_function = match palette {
            Palette::Gameplay => 0xb250,
            Palette::Scientist => 0xb2d2,
        }
    );
    crate::word_transfers!(game, ascending, 256; native(0x2200) <- rom(0x8ce3e9, 0));
    prepare_cinematic_palette_fade(game);
    for &(index, count) in palette.colors() {
        clear_cinematic_colors(index, count, game);
    }
    compose_cinematic_palette(game);
}

pub fn fade_to_scene<H: crate::menus::host::Host + ?Sized>(palette: Palette, game: &mut H) {
    if (word!(game.cinematic_var4) & 3) == 0 {
        palette.crossfade(false, game);
    }
    if (word!(game.cinematic_var4 -= 1, reread) & 0x8000) != 0 {
        game.ram().write_byte(0x69, 21);
        game.ram().write_byte(0x6b, 0);
        game.ram().write_byte(0x6f, 0);
        crate::word_transfers!(game, ascending, 640; pause_map(128) <- value(47));
        clear_japanese_tiles(game);
        word!(game.cinematic_function = (0x8ba391i32) as u16);
        if matches!(palette, Palette::Gameplay) && (word!(game.cinematic_var15) & 0x8000) != 0 {
            game.spawn_palette_effect(addr_kPalfx_E1BC as u16);
        }
    }
}

pub fn wait_for_music<H: crate::menus::host::Host + ?Sized>(next: u16, game: &mut H) {
    if game.has_queued_music() & 1 == 0 {
        word!(game.cinematic_function = next);
        if next == 0xa64c {
            word!(game.cinematic_var4 = 240);
        } else {
            word! { game, 2; screen_fade_delay, screen_fade_counter }
            if next == 0xa84a {
                game.spawn_cinematic_object(
                    CinematicLayer::Background,
                    addr_kCinematicBgObjectDef_8BCF3F as u16,
                    0x4c00,
                );
            }
        }
    }
}

pub fn wait_for_fade<H: crate::menus::host::Host + ?Sized>(
    fade: CinematicFade,
    next: u16,
    game: &mut H,
) {
    if advance_cinematic_fade(fade, game) {
        word!(game.cinematic_function = next);
        if next == 0xa5f8 {
            word!(game.cinematic_var4 = 60);
            if word!(game.japanese_text_flag) != 0 {
                queue_cinematic_tilemap(CinematicTilemap::JapaneseIntro, game);
            }
        }
    }
}

pub fn start_page<H: crate::menus::host::Host + ?Sized>(next: u16, j: u16, game: &mut H) -> u16 {
    word!(game.cinematic_function = next);
    set_cinematic_tilemap_updates(false, game);
    clear_japanese_tiles(game);
    queue_cinematic_tilemap(CinematicTilemap::Japanese, game);
    j
}

pub fn show_page<H: crate::menus::host::Host + ?Sized>(object: u16, next: u16, game: &mut H) {
    game.spawn_cinematic_object(CinematicLayer::Background, object, 0x4c00);
    word!(game.cinematic_function = next);
    setup_page_palettes(game);
}

pub fn fade_to_page<H: crate::menus::host::Host + ?Sized>(palette: Palette, game: &mut H) {
    if (word!(game.cinematic_var13) & 3) == 0 {
        palette.crossfade(true, game);
    }
    if (word!(game.cinematic_var13 -= 1, reread) & 0x8000) != 0 {
        game.ram().write_byte(0x69, 22);
        game.ram().write_byte(0x6b, 0);
        game.ram().write_byte(0x6f, 0);
        game.ram().write_byte(0x72, 0);
        if matches!(palette, Palette::Scientist) {
            crate::word_transfers!(game, ascending, 7; native(0xc1c0) <- rom(0x8ce3e9, 224));
        }
        word!(game.cinematic_function = (0x8ba391i32) as u16);
    }
}

pub type ImportedIntroPalette = Imported<IntroPalette>;

pub fn import_intro_palette(image: &sm_data::cartridge::RomImage) -> Option<ImportedIntroPalette> {
    Imported::import(
        image,
        INTRO_PALETTE,
        INTRO_PALETTE_BYTES,
        IntroPalette::import,
    )
}

/// Intro colour `index` in the loaded image. The record serves it while that
/// image is the imported allocation; otherwise, and past the 256 colours, the
/// original indexed load runs with its failures.
fn intro_color<H: crate::menus::host::Host + ?Sized>(game: &H, index: usize) -> u16 {
    let rom = game.loaded_image();
    Imported::from(game.intro_palette(), rom)
        .and_then(|palette| palette.color(index))
        .unwrap_or_else(|| {
            rom.table_word(
                sm_data::cartridge::lorom_offset(INTRO_PALETTE) as usize,
                index as isize,
            )
        })
}

pub fn draw_japanese_text<H: crate::menus::host::Host + ?Sized>(
    lines: &[u16],
    restore_palette: bool,
    j: u16,
    game: &mut H,
) -> u16 {
    set_cinematic_tilemap_updates(false, game);
    if restore_palette {
        for color in 17..20 {
            word!(game.palette_buffer[color] = intro_color(game, color));
        }
    }
    clear_japanese_tiles(game);
    for &line in lines {
        load_japanese_line(line, game);
    }
    queue_cinematic_tilemap(CinematicTilemap::Japanese, game);
    j
}

pub fn continue_japanese_text<H: crate::menus::host::Host + ?Sized>(
    next: u16,
    lines: &[u16],
    k: u16,
    game: &mut H,
) {
    if word!(game.joypad1_newkeys) != 0 {
        let slot = k as usize >> 1;
        word! {game; mode7_obj_instr_ptr[slot] = next; mode7_obj_instr_timer[slot] = 1;
        mode7_obj_preinstr_func[slot] = ((0x8b93d9i32) as u16); }
        draw_japanese_text(lines, false, 0, game);
        word!(game.cinematic_var16 = 60);
    }
}

pub fn wait_after_japanese_text<H: crate::menus::host::Host + ?Sized>(
    next: u16,
    j: u16,
    game: &mut H,
) -> u16 {
    set_cinematic_tilemap_updates(true, game);
    word!(game.cinematic_function = next);
    j
}

pub fn show_final_page<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    set_cinematic_tilemap_updates(true, game);
    clear_english_text(game);
    game.spawn_cinematic_object(
        CinematicLayer::Background,
        addr_kCinematicBgObjectDef_8BCF5D as u16,
        0x4c00,
    );
    word!(game.cinematic_function = (0x8ba390i32) as u16);
    reset_caret(game);
    game.queue_music_delayed8(0);
}

pub fn wait_for_final_page<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
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

fn setup_page_palettes<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    set_cinematic_blend(6, 17, 2, 0, game);
    crate::word_transfers!(game, ascending, 256; native(0x2200) <- rom(0x8ce3e9, 0));
    prepare_cinematic_palette_fade(game);
    clear_cinematic_colors(0, 0x10, game);
    clear_cinematic_colors(0x60, 0x10, game);
    clear_cinematic_colors(0x1d2, 6, game);
    compose_cinematic_palette(game);
    word!(game.cinematic_var4 = 127);
    set_cinematic_tilemap_updates(true, game);
    reset_caret(game);
}

pub fn enable_text<H: crate::menus::host::Host + ?Sized>(j: u16, game: &mut H) -> u16 {
    set_cinematic_tilemap_updates(true, game);
    j
}

pub fn fade_out_to_ceres<H: crate::menus::host::Host + ?Sized>(restore_samus: bool, game: &mut H) {
    game.fade(false);
    if game.ram().read_byte(0x51) == 0x80 {
        game.enable_display_nmi(true);
        word! { game, 0;
            screen_fade_delay, screen_fade_counter, reg_BG2HOFS, reg_BG3HOFS, reg_BG2VOFS,
            reg_BG3VOFS, cinematic_var5,
        }
        crate::word_transfers!(game, descending, 329; native(0x1c1d) <- value(0));
        word!(game.cinematic_function = if restore_samus { 0xbca0 } else { 0xc100 });
        if restore_samus {
            crate::menus::controller_input::restore_cinematic_bindings(game);
            word! { game, 0; samus_max_missiles, samus_missiles }
        }
    }
}

pub fn finish_pages<H: crate::menus::host::Host + ?Sized>(j: u16, game: &mut H) -> u16 {
    {
        word!(game.cinematic_function = (0x8bb72fi32) as u16);
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        j
    }
}

pub fn spawn_text_glow<H: crate::menus::host::Host + ?Sized>(
    definition: u16,
    position: u16,
    game: &mut H,
) {
    for slot in (0..8usize).rev() {
        let offset = 2 * slot;
        if checked_word!(game, 0x19f7 + offset;) == 0 {
            crate::word_transfers!(game, ascending, 1; native(0x19f7 + offset) <- value(definition));
            word! {game; eproj_pre_instr[slot + 2] = 1;
            eproj_pre_instr[slot + 10] = position as u8 as u16; }
            crate::word_transfers!(game, ascending, 1;
                native(0x1a27 + offset) <- value(u16::from(position.to_ne_bytes()[1])),
                native(0x1a37 + offset) <- value(0),
            );
            return;
        }
    }
}

#[forbid(unsafe_code)]
pub fn advance_text_glow<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let mut object = 14i32;
    while object >= 0 {
        word!(game.cinematic_var19 = object as u16);
        if checked_word!(game, 0x19f7 + object as usize;) != 0 {
            advance_text_glow_object(game);
            object = i32::from(word!(game.cinematic_var19));
        }
        object -= 2;
    }
}

#[forbid(unsafe_code)]
fn advance_text_glow_object<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    let object = usize::from(word!(game.cinematic_var19));
    let timer_at = 0x1a07 + (object & !1);
    let timer = checked_word!(game, timer_at;);
    checked_word!(game, timer_at; = timer.wrapping_sub(1));
    if timer != 1 {
        return;
    }
    let palette = u16::from_ne_bytes([
        game.ram().read_byte(0x1a37 + object),
        game.ram().read_byte(0x1a38 + object),
    ]);
    let definition = u16::from_ne_bytes([
        game.ram().read_byte(0x19f7 + object),
        game.ram().read_byte(0x19f8 + object),
    ]);
    let mut row = 2 * u16::from(game.ram().read_byte(0x1a17 + (object & !1)))
        + 64 * u16::from(game.ram().read_byte(0x1a27 + object));
    let (width, mut rows) = {
        let definition = crate::menus::menu_tilemap::native_record(
            (0x8cu32 << 16) | u32::from(definition),
            game,
        );
        (i32::from(definition.byte(2)), i32::from(definition.byte(3)))
    };
    loop {
        let mut column = row;
        let mut remaining = width;
        loop {
            let at = 0x3000 + usize::from(column);
            let tile = u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]);
            crate::word_transfers!(game, ascending, 1; native(at) <- value(palette | (tile & 0xe3ff)));
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
    if u16::from_ne_bytes([
        game.ram().read_byte(phase_at),
        game.ram().read_byte(phase_at + 1),
    ]) == 3072
    {
        crate::word_transfers!(game, ascending, 1; native(0x19f7 + selected) <- value(0));
    } else {
        checked_word!(game, phase_at; = checked_word!(game, phase_at;).wrapping_add(1024));
        crate::word_transfers!(game, ascending, 1; native(0x1a07 + (selected & !1)) <- value(5));
    }
}
