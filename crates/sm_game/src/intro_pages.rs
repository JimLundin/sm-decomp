//! Intro narrative pages, scene entry, crossfades and Japanese text continuations.
//! Numeric callbacks select these operations; Game retains the cinematic state.
use crate::cinematic_objects::CinematicObjects;
use crate::cinematic_state::{Array, Cinematic};
use crate::imported::Imported;
use crate::ram::RawRam as _;

use crate::cinematic::{
    advance_cinematic_fade, clear_cinematic_colors, compose_cinematic_palette,
    fade_cinematic_colors, prepare_cinematic_palette_fade, queue_cinematic_tilemap,
    set_cinematic_blend, set_cinematic_tilemap_updates,
};
use crate::constants::bank_8d::ADDR_K_PALFX_E1BC;
use crate::constants::bank_91::{ADDR_STRU_91877E, ADDR_STRU_918784};
use crate::constants::samus_poses::K_POSE_02_FACE_L_NORMAL;
use crate::menus::prelude::{HdmaObjectDefinition, display_scroll, do_until, owned};
use crate::ram::word;
use crate::visual_state::{CinematicFade, CinematicTilemap};
use sm_data::intro_palette::{
    ADDRESS as INTRO_PALETTE, BYTES as INTRO_PALETTE_BYTES, IntroPalette,
};

// Bank $8B cinematic object definitions.
pub const ADDR_CINEMATIC_BG_OBJECT_DEF_8BCF3F: u32 = 0xcf3f;
pub const ADDR_CINEMATIC_BG_OBJECT_DEF_8BCF5D: u32 = 0xcf5d;
pub const ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE5B: u32 = 0xce5b;
pub const ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE55: u32 = 0xce55;
pub const ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE61: u32 = 0xce61;
pub const ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE67: u32 = 0xce67;
pub const ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE73: u32 = 0xce73;
pub const ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE79: u32 = 0xce79;
pub const ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCF27: u32 = 0xcf27;

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

pub fn advance_scene_sprite<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    scene: u8,
    cine: &mut Cinematic,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    if scene == 0 {
        if (game.player_x() as i32 - 169) & 0x8000 != 0 {
            crate::cinematic::resume_cinematic_sprite(slot, 0xcb3b, cine);
            *cine.spr_pre_instruction(slot) = 0x93d9;
        }
    } else if scene == 1 {
        if *cine.intro_japanese_text_timer() == 59 {
            crate::cinematic::resume_cinematic_sprite(slot, 0xcc23, cine);
        }
        if *cine.var13() == 127 {
            crate::cinematic::resume_cinematic_sprite(slot, 0xce53, cine);
        }
    } else if *cine.spr_instruction_pointer(15) == 0 {
        crate::cinematic::resume_cinematic_sprite(slot, 0xee9b, cine);
    }
}

pub fn wait_for_scene_end(object: u16, stop_samus: bool, cine: &mut Cinematic) {
    if *cine.var13() == 0 {
        crate::cinematic::resume_cinematic_sprite(usize::from(object >> 1), 0xce53, cine);
        if stop_samus {
            *cine.intro_samus_display() = 0;
        }
    }
}

pub fn advance_laboratory_baby(object: u16, examined: bool, cine: &mut Cinematic) {
    let slot = usize::from(object >> 1);
    if *cine.var13() == 0 {
        crate::cinematic::resume_cinematic_sprite(slot, 0xce53, cine);
    } else if *cine.function_timer() & 3 == 0 {
        if examined {
            if (*cine.y() as i32 - 8) & 0x8000 != 0 {
                *cine.y() = cine.y().wrapping_add(1);
                *cine.bg_arr8(slot) = cine.bg_arr8(slot).wrapping_sub(1);
            }
        } else if *cine.x() != 0 {
            *cine.x() = cine.x().wrapping_sub(1);
            *cine.bg_arr7(slot) = cine.bg_arr7(slot).wrapping_add(1);
        }
    }
}

fn hide_caret(cine: &mut Cinematic) {
    *cine.bg_arr7(15) = 8;
    *cine.bg_arr8(15) = 248;
}

pub fn reset_caret(cine: &mut Cinematic) {
    *cine.bg_arr7(15) = 8;
    *cine.bg_arr8(15) = 24;
    *cine.spr_instruction_pointer(15) = 0xcbfb;
    *cine.spr_instruction_timer(15) = 1;
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
    cine: &mut Cinematic,
    game: &mut H,
) -> u16 {
    *cine.intro_text_click_flag() = 0;
    if crate::session_words::current(game).japanese_text_flag != 0 {
        CinematicObjects::Mode7.spawn(
            object,
            crate::session_words::current(game).japanese_text_flag,
            cine,
            game,
        );
    }
    j
}

pub fn reset_text_marker(cine: &mut Cinematic) {
    *cine.spr_instruction_pointer(15) = 0xcc03;
    *cine.spr_instruction_timer(15) = 1;
}

pub fn spawn_marker<H: crate::menus::host::Host + ?Sized>(
    next: u16,
    continuation: Option<u16>,
    j: u16,
    cine: &mut Cinematic,
    game: &mut H,
) -> u16 {
    reset_text_marker(cine);
    if crate::session_words::current(game).japanese_text_flag != 0 {
        let failed = CinematicObjects::Sprites.spawn(
            ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE73 as u16,
            crate::session_words::current(game).japanese_text_flag,
            cine,
            game,
        ) != 0;
        if let Some(object) = continuation {
            // The arrow leaves its palette in A; a full allocator leaves slot zero's instruction.
            let parameter = if failed {
                *cine.spr_instruction_pointer(0)
            } else {
                0x0c00
            };
            CinematicObjects::Mode7.spawn(object, parameter, cine, game);
            return j;
        }
    }
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = next
    });
    j
}

fn page_input_ready<H: crate::menus::host::Host + ?Sized>(
    cine: &mut Cinematic,
    game: &mut H,
) -> bool {
    if *cine.intro_japanese_text_timer() != 0 {
        *cine.intro_japanese_text_timer() = cine.intro_japanese_text_timer().wrapping_sub(1);
        false
    } else {
        word!(game.JOYPAD1_NEWKEYS) != 0
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

pub fn wait_for_scene<H: crate::menus::host::Host + ?Sized>(
    scene: Scene,
    cine: &mut Cinematic,
    game: &mut H,
) {
    if !page_input_ready(cine, game) {
        return;
    }
    let palette = match scene {
        Scene::MotherBrain | Scene::BabyMetroid => {
            let mother_brain = matches!(scene, Scene::MotherBrain);
            game.ram()
                .write_byte(0x58, if mother_brain { 80 } else { 84 });
            if !mother_brain {
                crate::room_view::set(game, crate::room_view::Word::Width, 32);
                crate::room_view::set(game, crate::room_view::Word::Height, 16);
            }
            crate::session_cohort::with(
                game,
                |_, words: &mut crate::session_words::SessionWords| {
                    words.samus_pose = K_POSE_02_FACE_L_NORMAL as u16
                },
            );
            game.finish_pose_change();
            let (x, y) = if mother_brain { (155, 115) } else { (376, 147) };
            game.set_player_x(x);
            game.set_previous_player_x(x);
            game.set_player_y(y);
            game.set_previous_player_y(y);
            *cine.x() = 0;
            *cine.var13() = 127;
            if mother_brain {
                // The preceding $1A4B store leaves $007F in the ROM accumulator.
                crate::cinematic_objects::spawn_cinematic_sprite_at(
                    ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE55 as u16,
                    0,
                    0x7f,
                    cine,
                    game,
                );
                // Palette left in A by the first sprite initializer.
                CinematicObjects::Sprites.spawn(
                    ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCF27 as u16,
                    0x0e00,
                    cine,
                    game,
                );
                crate::word_transfers!(game, ascending, 224; le(LEVEL_DATA, 0) <- rom(0x8cbec3, 0));
                crate::word_transfers!(game, ascending, 256; native(0x16402) <- value(0));
                crate::session_cohort::with(
                    game,
                    |_, words: &mut crate::session_words::SessionWords| words.hud_item_index = 1,
                );
            } else {
                CinematicObjects::Sprites.spawn(
                    ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE5B as u16,
                    0x7f,
                    cine,
                    game,
                );
                CinematicObjects::Sprites.spawn(
                    ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE79 as u16,
                    0x0e00,
                    cine,
                    game,
                );
                *cine.intro_frame_counter() = 0;
                crate::word_transfers!(game, ascending, 384; le(LEVEL_DATA, 0) <- rom(0x8cc083, 0));
                crate::session_cohort::with(
                    game,
                    |_, words: &mut crate::session_words::SessionWords| words.hud_item_index = 0,
                );
            }
            word!(game.FRAME_HANDLER_ALFA = (0x90e6c9i32) as u16);
            word!(game.FRAME_HANDLER_BETA = (0x90e833i32) as u16);
            game.demo_start();
            let input = if mother_brain {
                ADDR_STRU_918784
            } else {
                ADDR_STRU_91877E
            };
            // EnableDemoInput leaves $8000 in the ROM accumulator for the loader.
            game.demo_load(0x8000, input as u16);
            *cine.intro_samus_display() = if mother_brain { u16::MAX } else { 1 };
            Palette::Gameplay
        }
        Scene::ScientistPage3 | Scene::ScientistPage4 => {
            let scientist = matches!(scene, Scene::ScientistPage3);
            game.ram().write_byte(0x58, if scientist { 88 } else { 92 });
            *cine.x() = if scientist { 32 } else { 0 };
            *cine.y() = if scientist { 8 } else { -24i16 as u16 };
            *cine.var13() = 127;
            let object = if scientist {
                ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE61
            } else {
                ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE67
            };
            CinematicObjects::Sprites.spawn(object as u16, 0x7f, cine, game);
            Palette::Scientist
        }
    };
    set_cinematic_blend(6, 17, 2, 0, game);
    game.spawn_hdma(0x88, &HdmaObjectDefinition::new(0, 0x31, 0xec03));
    hide_caret(cine);
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = match palette {
            Palette::Gameplay => 0xb250,
            Palette::Scientist => 0xb2d2,
        }
    });
    crate::word_transfers!(game, ascending, 256; native(0x2200) <- rom(0x8ce3e9, 0));
    prepare_cinematic_palette_fade(game);
    for &(index, count) in palette.colors() {
        clear_cinematic_colors(index, count, game);
    }
    compose_cinematic_palette(game);
}

pub fn fade_to_scene<H: crate::menus::host::Host + ?Sized>(
    palette: Palette,
    cine: &mut Cinematic,
    game: &mut H,
) {
    if (*cine.function_timer() & 3) == 0 {
        palette.crossfade(false, game);
    }
    if ({
        *cine.function_timer() = cine.function_timer().wrapping_sub(1);
        *cine.function_timer()
    } & 0x8000)
        != 0
    {
        game.ram().write_byte(0x69, 21);
        game.ram().write_byte(0x6b, 0);
        game.ram().write_byte(0x6f, 0);
        crate::word_transfers!(game, ascending, 640; pause_map(128) <- value(47));
        clear_japanese_tiles(game);
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = 0xa391
        });
        if matches!(palette, Palette::Gameplay) && (*cine.intro_samus_display() & 0x8000) != 0 {
            crate::palette_effects::execution::leased::spawn(ADDR_K_PALFX_E1BC as u16, game);
        }
    }
}

pub fn wait_for_music<H: crate::menus::host::Host + ?Sized>(
    next: u16,
    cine: &mut Cinematic,
    game: &mut H,
) {
    if crate::music_requests::has_queued_music(game) & 1 == 0 {
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = next
        });
        if next == 0xa64c {
            *cine.function_timer() = 240;
        } else {
            crate::screen_fade::store(game, 2, 2);
            if next == 0xa84a {
                CinematicObjects::Background.spawn(
                    ADDR_CINEMATIC_BG_OBJECT_DEF_8BCF3F as u16,
                    0x4c00,
                    cine,
                    game,
                );
            }
        }
    }
}

pub fn wait_for_fade<H: crate::menus::host::Host + ?Sized>(
    fade: CinematicFade,
    next: u16,
    cine: &mut Cinematic,
    game: &mut H,
) {
    if advance_cinematic_fade(fade, game) {
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = next
        });
        if next == 0xa5f8 {
            *cine.function_timer() = 60;
            if crate::session_words::current(game).japanese_text_flag != 0 {
                queue_cinematic_tilemap(CinematicTilemap::JapaneseIntro, cine, game);
            }
        }
    }
}

pub fn start_page<H: crate::menus::host::Host + ?Sized>(
    next: u16,
    j: u16,
    cine: &mut Cinematic,
    game: &mut H,
) -> u16 {
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = next
    });
    set_cinematic_tilemap_updates(false, cine);
    clear_japanese_tiles(game);
    queue_cinematic_tilemap(CinematicTilemap::Japanese, cine, game);
    j
}

pub fn show_page<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    next: u16,
    cine: &mut Cinematic,
    game: &mut H,
) {
    CinematicObjects::Background.spawn(object, 0x4c00, cine, game);
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = next
    });
    setup_page_palettes(cine, game);
}

pub fn fade_to_page<H: crate::menus::host::Host + ?Sized>(
    palette: Palette,
    cine: &mut Cinematic,
    game: &mut H,
) {
    if (*cine.var13() & 3) == 0 {
        palette.crossfade(true, game);
    }
    if ({
        *cine.var13() = cine.var13().wrapping_sub(1);
        *cine.var13()
    } & 0x8000)
        != 0
    {
        game.ram().write_byte(0x69, 22);
        game.ram().write_byte(0x6b, 0);
        game.ram().write_byte(0x6f, 0);
        game.ram().write_byte(0x72, 0);
        if matches!(palette, Palette::Scientist) {
            crate::word_transfers!(game, ascending, 7; native(0xc1c0) <- rom(0x8ce3e9, 224));
        }
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = 0xa391
        });
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
    cine: &mut Cinematic,
    game: &mut H,
) -> u16 {
    set_cinematic_tilemap_updates(false, cine);
    if restore_palette {
        crate::session_cohort::with(
            game,
            |game, palette: &mut crate::cinematic::PaletteBuffer| {
                for color in 17..20 {
                    palette.0[color] = intro_color(game, color);
                }
            },
        );
    }
    clear_japanese_tiles(game);
    for &line in lines {
        load_japanese_line(line, game);
    }
    queue_cinematic_tilemap(CinematicTilemap::Japanese, cine, game);
    j
}

pub fn continue_japanese_text<H: crate::menus::host::Host + ?Sized>(
    next: u16,
    lines: &[u16],
    k: u16,
    cine: &mut Cinematic,
    game: &mut H,
) {
    if word!(game.JOYPAD1_NEWKEYS) != 0 {
        let slot = k as usize >> 1;
        *cine.mode7_instruction_pointer(slot) = next;
        *cine.mode7_instruction_timer(slot) = 1;
        *cine.mode7_pre_instruction(slot) = 0x93d9;
        draw_japanese_text(lines, false, 0, cine, game);
        *cine.intro_japanese_text_timer() = 60;
    }
}

pub fn wait_after_japanese_text<H: crate::menus::host::Host + ?Sized>(
    next: u16,
    j: u16,
    cine: &mut Cinematic,
    game: &mut H,
) -> u16 {
    set_cinematic_tilemap_updates(true, cine);
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = next
    });
    j
}

pub fn show_final_page<H: crate::menus::host::Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    set_cinematic_tilemap_updates(true, cine);
    clear_english_text(game);
    CinematicObjects::Background.spawn(
        ADDR_CINEMATIC_BG_OBJECT_DEF_8BCF5D as u16,
        0x4c00,
        cine,
        game,
    );
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = 0xa390
    });
    reset_caret(cine);
    crate::music_requests::queue_delayed_8(0, game);
}

pub fn wait_for_final_page<H: crate::menus::host::Host + ?Sized>(
    cine: &mut Cinematic,
    game: &mut H,
) {
    if page_input_ready(cine, game) {
        *cine.var13() = 127;
        if crate::session_words::current(game).japanese_text_flag != 0 {
            crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
                flow.cinematic_function = 0xb207
            });
            set_cinematic_tilemap_updates(false, cine);
            clear_japanese_tiles(game);
            queue_cinematic_tilemap(CinematicTilemap::Japanese, cine, game);
        } else {
            show_final_page(cine, game);
        }
    }
}

fn setup_page_palettes<H: crate::menus::host::Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    set_cinematic_blend(6, 17, 2, 0, game);
    crate::word_transfers!(game, ascending, 256; native(0x2200) <- rom(0x8ce3e9, 0));
    prepare_cinematic_palette_fade(game);
    clear_cinematic_colors(0, 0x10, game);
    clear_cinematic_colors(0x60, 0x10, game);
    clear_cinematic_colors(0x1d2, 6, game);
    compose_cinematic_palette(game);
    *cine.function_timer() = 127;
    set_cinematic_tilemap_updates(true, cine);
    reset_caret(cine);
}

pub fn enable_text(j: u16, cine: &mut Cinematic) -> u16 {
    set_cinematic_tilemap_updates(true, cine);
    j
}

pub fn fade_out_to_ceres<H: crate::menus::host::Host + ?Sized>(
    restore_samus: bool,
    cine: &mut Cinematic,
    game: &mut H,
) {
    game.fade(false);
    if game.ram().read_byte(0x51) == 0x80 {
        crate::display_refresh::enable(true, game);
        crate::screen_fade::store(game, 0, 0);
        display_scroll!(game, 0; Bg2H, Bg3H, Bg2V, Bg3V);
        *cine.mode7_angle() = 0;
        cine.clear();
        crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = if restore_samus { 0xbca0 } else { 0xc100 }
        });
        if restore_samus {
            crate::menus::controller_input::restore_cinematic_bindings(game);
            crate::session_cohort::with(game, |_, inventory: &mut crate::inventory::Inventory| {
                inventory.ammunition[0] = crate::inventory::Supply::default();
            });
        }
    }
}

pub fn finish_pages<H: crate::menus::host::Host + ?Sized>(j: u16, game: &mut H) -> u16 {
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
        flow.cinematic_function = 0xb72f
    });
    crate::screen_fade::store(game, 1, 1);
    j
}

// The text glow objects' tables, by byte offset (RAM map $19F7..$1A46).
const GLOW_POINTERS: Array = Array::TextGlowPointers;
const GLOW_TIMERS: Array = Array::TextGlowTimers;
const GLOW_X: Array = Array::TextGlowXPositions;
const GLOW_Y: Array = Array::TextGlowYPositions;
const GLOW_PALETTES: Array = Array::TextGlowPalettes;

pub fn spawn_text_glow(definition: u16, position: u16, cine: &mut Cinematic) {
    for slot in (0..8usize).rev() {
        if *cine.text_glow_pointers(slot) == 0 {
            *cine.text_glow_pointers(slot) = definition;
            *cine.text_glow_timers(slot) = 1;
            *cine.text_glow_x_positions(slot) = position & 0xff;
            *cine.text_glow_y_positions(slot) = position >> 8;
            *cine.text_glow_palettes(slot) = 0;
            return;
        }
    }
}

pub fn advance_text_glow<H: crate::menus::host::Host + ?Sized>(cine: &mut Cinematic, game: &mut H) {
    let mut object = 14i32;
    while object >= 0 {
        *cine.text_glow_object_index() = object as u16;
        if *cine.offset(GLOW_POINTERS, object as usize) != 0 {
            advance_text_glow_object(cine, game);
            object = i32::from(*cine.text_glow_object_index());
        }
        object -= 2;
    }
}

fn advance_text_glow_object<H: crate::menus::host::Host + ?Sized>(
    cine: &mut Cinematic,
    game: &mut H,
) {
    let object = usize::from(*cine.text_glow_object_index());
    let timer = *cine.offset(GLOW_TIMERS, object & !1);
    *cine.offset(GLOW_TIMERS, object & !1) = timer.wrapping_sub(1);
    if timer != 1 {
        return;
    }
    let palette = *cine.offset(GLOW_PALETTES, object);
    let definition = *cine.offset(GLOW_POINTERS, object);
    let [x, _] = (*cine.offset(GLOW_X, object & !1)).to_le_bytes();
    let [y, _] = (*cine.offset(GLOW_Y, object)).to_le_bytes();
    let mut row = 2 * u16::from(x) + 64 * u16::from(y);
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
    let selected = usize::from(*cine.text_glow_object_index());
    let phase = *cine.offset(GLOW_PALETTES, selected);
    if phase == 3072 {
        *cine.offset(GLOW_POINTERS, selected) = 0;
    } else {
        *cine.offset(GLOW_PALETTES, selected) = phase.wrapping_add(1024);
        *cine.offset(GLOW_TIMERS, selected & !1) = 5;
    }
}
