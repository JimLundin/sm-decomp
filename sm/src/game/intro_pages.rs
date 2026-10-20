//! Intro narrative pages, scene entry, crossfades and Japanese text continuations.
//! Numeric callbacks select these operations; Game retains the cinematic state.

use super::constants::bank_8d::addr_kPalfx_E1BC;
use super::constants::bank_91::{addr_stru_91877E, addr_stru_918784};
use super::constants::samus_poses::kPose_02_FaceL_Normal;
use super::sm_80::{EnableNMI, HandleFadeOut, QueueMusic_Delayed8};
use super::sm_88::SpawnIntroCutsceneCrossfadeHdma;
use super::sm_8b::{
    addr_kCinematicBgObjectDef_8BCF5D, addr_kCinematicSpriteObjectDef_8BCE55,
    addr_kCinematicSpriteObjectDef_8BCE5B, addr_kCinematicSpriteObjectDef_8BCE61,
    addr_kCinematicSpriteObjectDef_8BCE67, addr_kCinematicSpriteObjectDef_8BCE73,
    addr_kCinematicSpriteObjectDef_8BCE79, addr_kCinematicSpriteObjectDef_8BCF27,
    CinematicBgInstr_SetSomeStuffForSpriteObject_14, CinematicFunction_Intro_Func20,
    ClearEnglishTextTilemap, ClearJapaneseTextTiles, ClearYColorsFromIndexX, ComposeFadingPalettes,
    DecomposePaletteDataForFading, DisableCinematicBgTilemapUpdates,
    EnableCinematicBgTilemapUpdates, FadeInYColorsFromIndexX, FadeOutYColorsFromIndexX,
    RevertButtonConfig, SetSomeStuffForSpriteObject_15, SetSomeStuffForSpriteObject_16,
    SpawnCinematicBgObject, SpawnCinematicSpriteObject, SpawnCinematicSpriteObjectAtSlot,
    SpawnMode7Object, TransferJapaneseTextTilesToVram,
};
use super::sm_8d::SpawnPalfxObject;
use super::sm_91::{
    EnableDemoInput, LoadDemoInputObject, ResetDemoData, SamusFunc_F433,
    Samus_SetAnimationFrameIfPoseChanged,
};
use crate::game_state::Game;
use crate::ram_map::{ram_ptr, word};

// Narrative callbacks retain their ROM addresses; these functions own the shared
// page/scene lifecycle, including the Japanese continuation and input delay.
pub(super) unsafe fn create_japanese_text(object: u16, j: u16, game: &mut Game) -> u16 {
    word!(game.cinematicbg_var1 = 0);
    if word!(game.japanese_text_flag) != 0 {
        SpawnMode7Object(object, word!(game.japanese_text_flag), game);
    }
    j
}

pub(super) unsafe fn spawn_marker(
    next: u16,
    continuation: Option<u16>,
    k: u16,
    j: u16,
    game: &mut Game,
) -> u16 {
    let j = CinematicBgInstr_SetSomeStuffForSpriteObject_14(k, j, game);
    if word!(game.japanese_text_flag) != 0 {
        let failed = SpawnCinematicSpriteObject(
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
            SpawnMode7Object(object, parameter, game);
            return j;
        }
    }
    word!(game.cinematic_function = next);
    j
}

unsafe fn page_input_ready(game: &mut Game) -> bool {
    if word!(game.cinematic_var16) != 0 {
        word!(game.cinematic_var16, wrapping_sub(1));
        false
    } else {
        word!(game.joypad1_newkeys) != 0
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
    if !page_input_ready(game) {
        return;
    }
    let palette = match scene {
        Scene::MotherBrain | Scene::BabyMetroid => {
            let mother_brain = matches!(scene, Scene::MotherBrain);
            *ram_ptr!(game; 0x58) = if mother_brain { 80 } else { 84 };
            if !mother_brain {
                word!(game.room_width_in_blocks = 32);
                word!(game.room_height_in_blocks = 16);
            }
            word!(game.samus_pose = kPose_02_FaceL_Normal as u16);
            SamusFunc_F433(game);
            Samus_SetAnimationFrameIfPoseChanged(game);
            word!(game.samus_last_different_pose = word!(game.samus_prev_pose));
            *(ram_ptr!(game, u16; 0xa26)) = *(ram_ptr!(game, u16; 0xa22));
            word!(game.samus_prev_pose = word!(game.samus_pose));
            *(ram_ptr!(game, u16; 0xa22)) = *(ram_ptr!(game, u16; 0xa1e));
            let (x, y) = if mother_brain { (155, 115) } else { (376, 147) };
            word!(game.samus_x_pos = x);
            word!(game.samus_prev_x_pos = x);
            word!(game.samus_y_pos = y);
            word!(game.samus_prev_y_pos = y);
            word!(game.cinematic_var8 = 0);
            word!(game.cinematic_var13 = 127);
            if mother_brain {
                // The preceding $1A4B store leaves $007F in the ROM accumulator.
                SpawnCinematicSpriteObjectAtSlot(
                    addr_kCinematicSpriteObjectDef_8BCE55 as u16,
                    0,
                    0x7f,
                    game,
                );
                // Palette left in A by the first sprite initializer.
                SpawnCinematicSpriteObject(
                    addr_kCinematicSpriteObjectDef_8BCF27 as u16,
                    0x0e00,
                    game,
                );
                word_transfers!(game, ascending, 224; le(level_data, 0) <- rom(0x8cbec3, 0));
                word_transfers!(game, ascending, 256; native(0x16402) <- value(0));
                word!(game.hud_item_index = 1);
            } else {
                SpawnCinematicSpriteObject(
                    addr_kCinematicSpriteObjectDef_8BCE5B as u16,
                    0x7f,
                    game,
                );
                SpawnCinematicSpriteObject(
                    addr_kCinematicSpriteObjectDef_8BCE79 as u16,
                    0x0e00,
                    game,
                );
                word!(game.cinematic_var11 = 0);
                word_transfers!(game, ascending, 384; le(level_data, 0) <- rom(0x8cc083, 0));
                word!(game.hud_item_index = 0);
            }
            word!(game.frame_handler_alfa = ((0x90e6c9i32) as u16));
            word!(game.frame_handler_beta = ((0x90e833i32) as u16));
            ResetDemoData(game);
            EnableDemoInput(game);
            let input = if mother_brain {
                addr_stru_918784
            } else {
                addr_stru_91877E
            };
            // EnableDemoInput leaves $8000 in the ROM accumulator for the loader.
            LoadDemoInputObject(0x8000, input as u16, game);
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
            SpawnCinematicSpriteObject(object as u16, 0x7f, game);
            Palette::Scientist
        }
    };
    *ram_ptr!(game; 0x69) = 6;
    *ram_ptr!(game; 0x6b) = 17;
    *ram_ptr!(game; 0x6f) = 2;
    *ram_ptr!(game; 0x72) = 0;
    SpawnIntroCutsceneCrossfadeHdma(game);
    SetSomeStuffForSpriteObject_15(game);
    word!(
        game.cinematic_function = match palette {
            Palette::Gameplay => 0xb250,
            Palette::Scientist => 0xb2d2,
        }
    );
    word_transfers!(game, ascending, 256; native(0x2200) <- rom(0x8ce3e9, 0));
    DecomposePaletteDataForFading(game);
    for &(index, count) in palette.colors() {
        ClearYColorsFromIndexX(index, count, game);
    }
    ComposeFadingPalettes(game);
}

pub(super) unsafe fn fade_to_scene(palette: Palette, game: &mut Game) {
    if (word!(game.cinematic_var4) & 3) == 0 {
        FadeOutYColorsFromIndexX(0, 0x14, game);
        FadeOutYColorsFromIndexX(0x60, 0x10, game);
        FadeOutYColorsFromIndexX(0x1d2, 6, game);
        for &(index, count) in palette.colors() {
            FadeInYColorsFromIndexX(index, count, game);
        }
        ComposeFadingPalettes(game);
    }
    if (word!(game.cinematic_var4, wrapping_sub(1), reread) & 0x8000) != 0 {
        *ram_ptr!(game; 0x69) = 21;
        *ram_ptr!(game; 0x6b) = 0;
        *ram_ptr!(game; 0x6f) = 0;
        word_transfers!(game, ascending, 640; pause_map(128) <- value(47));
        ClearJapaneseTextTiles(game);
        word!(game.cinematic_function = ((0x8ba391i32) as u16));
        if matches!(palette, Palette::Gameplay) && (word!(game.cinematic_var15) & 0x8000) != 0 {
            SpawnPalfxObject(addr_kPalfx_E1BC as u16, game);
        }
    }
}

pub(super) unsafe fn start_page(next: u16, j: u16, game: &mut Game) -> u16 {
    word!(game.cinematic_function = next);
    DisableCinematicBgTilemapUpdates(game);
    ClearJapaneseTextTiles(game);
    TransferJapaneseTextTilesToVram(game);
    j
}

pub(super) unsafe fn show_page(object: u16, next: u16, game: &mut Game) {
    SpawnCinematicBgObject(object, 0x4c00, game);
    word!(game.cinematic_function = next);
    setup_page_palettes(game);
}

pub(super) unsafe fn fade_to_page(palette: Palette, game: &mut Game) {
    if (word!(game.cinematic_var13) & 3) == 0 {
        FadeInYColorsFromIndexX(0, 0x10, game);
        FadeInYColorsFromIndexX(0x60, 0x10, game);
        FadeInYColorsFromIndexX(0x1d2, 6, game);
        for &(index, count) in palette.colors() {
            FadeOutYColorsFromIndexX(index, count, game);
        }
        ComposeFadingPalettes(game);
    }
    if (word!(game.cinematic_var13, wrapping_sub(1), reread) & 0x8000) != 0 {
        *ram_ptr!(game; 0x69) = 22;
        *ram_ptr!(game; 0x6b) = 0;
        *ram_ptr!(game; 0x6f) = 0;
        *ram_ptr!(game; 0x72) = 0;
        if matches!(palette, Palette::Scientist) {
            word_transfers!(game, ascending, 7; native(0xc1c0) <- rom(0x8ce3e9, 224));
        }
        word!(game.cinematic_function = ((0x8ba391i32) as u16));
    }
}

pub(super) unsafe fn draw_japanese_text(
    lines: &[u16],
    restore_palette: bool,
    j: u16,
    game: &mut Game,
) -> u16 {
    DisableCinematicBgTilemapUpdates(game);
    if restore_palette {
        word!(game.palette_buffer[17] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 17));
        word!(game.palette_buffer[18] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 18));
        word!(game.palette_buffer[19] = crate::rtl::rom_table_word!(game, 0x8ce3e9, 19));
    }
    ClearJapaneseTextTiles(game);
    for &line in lines {
        CinematicFunction_Intro_Func20(line, game);
    }
    TransferJapaneseTextTilesToVram(game);
    j
}

pub(super) unsafe fn continue_japanese_text(next: u16, lines: &[u16], k: u16, game: &mut Game) {
    if word!(game.joypad1_newkeys) != 0 {
        let slot = k as usize >> 1;
        word!(game.mode7_obj_instr_ptr[slot] = next);
        word!(game.mode7_obj_instr_timer[slot] = 1);
        word!(game.mode7_obj_preinstr_func[slot] = ((0x8b93d9i32) as u16));
        draw_japanese_text(lines, false, 0, game);
        word!(game.cinematic_var16 = 60);
    }
}

pub(super) unsafe fn wait_after_japanese_text(next: u16, j: u16, game: &mut Game) -> u16 {
    EnableCinematicBgTilemapUpdates(game);
    word!(game.cinematic_function = next);
    j
}

pub(super) unsafe fn show_final_page(game: &mut Game) {
    EnableCinematicBgTilemapUpdates(game);
    ClearEnglishTextTilemap(game);
    SpawnCinematicBgObject(addr_kCinematicBgObjectDef_8BCF5D as u16, 0x4c00, game);
    word!(game.cinematic_function = ((0x8ba390i32) as u16));
    SetSomeStuffForSpriteObject_16(game);
    QueueMusic_Delayed8(0, game);
}

pub(super) unsafe fn wait_for_final_page(game: &mut Game) {
    if page_input_ready(game) {
        word!(game.cinematic_var13 = 127);
        if word!(game.japanese_text_flag) != 0 {
            word!(game.cinematic_function = ((0x8bb207i32) as u16));
            DisableCinematicBgTilemapUpdates(game);
            ClearJapaneseTextTiles(game);
            TransferJapaneseTextTilesToVram(game);
        } else {
            show_final_page(game);
        }
    }
}

unsafe fn setup_page_palettes(game: &mut Game) {
    *ram_ptr!(game; 0x69) = 6;
    *ram_ptr!(game; 0x6b) = 17;
    *ram_ptr!(game; 0x6f) = 2;
    *ram_ptr!(game; 0x72) = 0;
    word_transfers!(game, ascending, 256; native(0x2200) <- rom(0x8ce3e9, 0));
    DecomposePaletteDataForFading(game);
    ClearYColorsFromIndexX(0, 0x10, game);
    ClearYColorsFromIndexX(0x60, 0x10, game);
    ClearYColorsFromIndexX(0x1d2, 6, game);
    ComposeFadingPalettes(game);
    word!(game.cinematic_var4 = 127);
    EnableCinematicBgTilemapUpdates(game);
    SetSomeStuffForSpriteObject_16(game);
}

pub(super) unsafe fn enable_text(j: u16, game: &mut Game) -> u16 {
    EnableCinematicBgTilemapUpdates(game);
    j
}

pub(super) unsafe fn fade_out_to_ceres(game: &mut Game) {
    HandleFadeOut(game);
    if *ram_ptr!(game; 0x51) == 0x80 {
        EnableNMI(game);
        word! { game, 0;
            screen_fade_delay, screen_fade_counter, reg_BG2HOFS, reg_BG3HOFS, reg_BG2VOFS,
            reg_BG3VOFS, cinematic_var5,
        }
        word_transfers!(game, descending, 329; native(0x1c1d) <- value(0));
        word!(game.cinematic_function = ((0x8bbca0i32) as u16));
        RevertButtonConfig(game);
        word! { game, 0; samus_max_missiles, samus_missiles }
    }
}

pub(super) unsafe fn finish_pages(j: u16, game: &mut Game) -> u16 {
    word!(game.cinematic_function = ((0x8bb72fi32) as u16));
    word! { game, 1; screen_fade_delay, screen_fade_counter }
    j
}
