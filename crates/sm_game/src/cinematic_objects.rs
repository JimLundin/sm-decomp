//! The cinematic object engine: sprite, Mode 7 and background objects, their scripts and drawing.
// The original's names and assignment shapes are kept.
#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_assignments)]
use crate::ending_graphics::draw_cinematic_item_percentage;
#[allow(unused_imports)]
use crate::ram::RawRam as _;
use crate::ram::{checked_word, word};
use crate::visual_state::CeresExplosion;
use crate::{cinematic::*, ending_screens::*, intro_objects::*, title_screen::*};

use crate::ending_sequence::object as ending;
use crate::intro_sequence::{mode7 as intro_mode7, object as intro};
use crate::title_sequence::object as title;
const _: () = {
    use crate::constants::bank_8d as palfx;
    use crate::ending_sequence::palette_effect as e;
    assert!(crate::intro_sequence::ZEBES_TEXT_FADE_IN == palfx::addr_kPalfx_E1B0 as u16);
    assert!(e::ZEBES_FADE_OUT == palfx::addr_kPalfx_E1C4 as u16);
    assert!(e::EXPLOSION_WIDE_FOREGROUND == palfx::addr_kPalfx_E1C8 as u16);
    assert!(e::EXPLOSION_FINALE == palfx::addr_kPalfx_E1CC as u16);
    assert!(e::EXPLOSION_SPACE_WHITE_OUT == palfx::addr_kPalfx_E1D0 as u16);
    assert!(e::EXPLOSION_WIDE_BACKGROUND == palfx::addr_kPalfx_E1E8 as u16);
};

#[derive(Clone, Copy)]
pub enum CinematicObjects {
    Sprites,
    Mode7,
    Background,
}

/// Cinematic object tables addressed through the RAM-compatible layouts below;
/// stays with them until those layouts have a typed owner.
struct CinematicObjectLayout {
    slots: u16,
    current: usize,
    instructions: usize,
    parameter: usize,
    cleared: &'static [usize],
}

impl CinematicObjects {
    fn layout(self) -> CinematicObjectLayout {
        use crate::ram::{word_arrays as a, words as w};
        match self {
            Self::Sprites => CinematicObjectLayout {
                slots: 16,
                current: w::cinematic_obj_index,
                instructions: a::cinematicspr_instr_ptr,
                parameter: w::cinematic_spawn_param,
                cleared: &[
                    a::cinematicspr_whattodraw,
                    a::cinematicspr_goto_timer,
                    a::cinematicspr_arr6,
                    a::cinematicspr_arr7,
                ],
            },
            Self::Mode7 => CinematicObjectLayout {
                slots: 2,
                current: w::mode7_cur_index,
                instructions: a::mode7_obj_instr_ptr,
                parameter: w::mode7_spawn_param,
                cleared: &[a::mode7_obj_goto_timer],
            },
            Self::Background => CinematicObjectLayout {
                slots: 4,
                current: w::cinematicbg_var2,
                instructions: a::cinematicbg_instr_ptr,
                parameter: w::cinematicbg_var3,
                cleared: &[a::cinematicbg_arr1, a::cinematicbg_arr6],
            },
        }
    }

    pub fn spawn<H: crate::menus::host::Host + ?Sized>(
        self,
        definition: u16,
        parameter: u16,
        game: &mut H,
    ) -> u8 {
        let layout = self.layout();
        checked_word!(game, layout.parameter; = parameter);
        for slot in (0..layout.slots).rev() {
            if checked_word!(game, layout.instructions + 2 * usize::from(slot);) == 0 {
                return initialize_cinematic_object(self, definition, slot * 2, game);
            }
        }
        1
    }

    pub fn advance<H: crate::menus::host::Host + ?Sized>(self, game: &mut H) {
        use crate::visual_state::CinematicTilemap;
        use CinematicObjects::Background;
        if matches!(self, Background) && word!(game.cinematic_enable_objs) & 0x8000 == 0 {
            return;
        }
        let layout = self.layout();
        let mut object = i32::from(2 * (layout.slots - 1));
        while object >= 0 {
            checked_word!(game, layout.current; = object as u16);
            if checked_word!(game, layout.instructions + 2 * (object as usize >> 1);) != 0 {
                {
                    match self {
                        Background => advance_cinematic_object::<0x8c>(self, object as u16, game),
                        _ => advance_cinematic_object::<0x8b>(self, object as u16, game),
                    }
                }
                // A callback may redirect the scan outside its original slot range.
                object = i32::from(checked_word!(game, layout.current;));
            }
            object -= 2;
        }
        if matches!(self, Background) {
            if word!(game.cinematic_enable_bg_tilemap) & 0x8000 != 0 {
                queue_cinematic_tilemap(CinematicTilemap::Text, game);
            }
            queue_cinematic_tilemap(CinematicTilemap::Eyes, game);
        }
    }
}

pub fn spawn_cinematic_burst<H: crate::menus::host::Host + ?Sized>(
    mut definition: u16,
    parameters: impl IntoIterator<Item = u16>,
    stride: u16,
    game: &mut H,
) {
    for parameter in parameters {
        CinematicObjects::Sprites.spawn(definition, parameter, game);
        definition = definition.wrapping_add(stride);
    }
}

pub fn advance_intro_mother_brain<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    stage: u8,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    if stage != 2 {
        let timer = word!(game.cinematicspr_arr6[slot]);
        if timer != 0 {
            if timer & 1 != 0 {
                crate::word_transfers!(game, descending, 16; native(0xc1fe) <- rom(0x8ce3e9, 255));
            } else {
                crate::word_transfers!(game, descending, 16; native(0xc1fe) <- value(0x7fff));
            }
            word!(game.cinematicspr_arr6[slot] -= 1);
        }
    }
    if stage == 0 {
        let Some(projectile) = (0..5).rfind(|&i| word!(game.projectile_type[i]) & 0xfff == 256)
        else {
            return;
        };
        if word!(game.projectile_x_pos[projectile]) as i32 - 84 & 0x8000 == 0 {
            return;
        }
        game.retire_projectile(2 * projectile as u16);
        word!(game.cinematicspr_arr6[slot] = 8);
    } else {
        if word!(game.cinematic_var12) & 1 != 0 {
            word!(game.cinematic_var10 -= 4);
        } else {
            word!(game.cinematic_var10 += 4);
        }
        if stage == 2 {
            crate::intro_pages::wait_for_scene_end(object, true, game);
            return;
        }
    }
    let count = word!(game.cinematicspr_goto_timer[slot]).wrapping_add(1);
    word!(game.cinematicspr_goto_timer[slot] = count);
    if stage == 0 && count == 4 {
        word! {game; cinematicspr_goto_timer[slot] = 0; cinematicspr_preinstr_func[slot] = 0xb80f; }
        spawn_cinematic_burst(0xcf1b, 0..3, 0, game);
        spawn_cinematic_burst(0xcf15, 0..5, 0, game);
    }
    if stage == 1 && count as i32 - 128 & 0x8000 == 0 {
        resume_cinematic_sprite(slot, 0xcb19, game);
    }
}

#[forbid(unsafe_code)]
pub fn execute_cinematic_sprite_instruction<H: crate::menus::host::Host + ?Sized>(
    address: u32,
    k: u16,
    mut j: u16,
    game: &mut H,
) -> Option<u16> {
    use crate::ending_sequence::palette_effect as ending_effect;
    use CinematicObjects::Sprites as CinematicSprites;
    crate::game_callback_dispatch!(@body [checked] {
        match address {
            9150157 => {
            let v0: u16 = 0x9ecd;
            CinematicSprites.spawn(title::COPYRIGHT_FADE_IN, v0, game);
            },
            9150192 => {
            word!(game.cinematicbg_arr9[(k >> 1)] = 0);
            CinematicSprites.spawn(title::COPYRIGHT_FADE_IN, 0, game);
            },
            9150233 => {
            crate::flow::set(game, crate::flow::Word::DemoTimer, 900);
    crate::flow::set(game, crate::flow::Word::CinematicFunction, (0x8b9f29i32) as u16);
            },
            9158818 => {
            crate::flow::set(game, crate::flow::Word::CinematicFunction, (0x8bc0c5i32) as u16);
            word! { game, 1; screen_fade_delay, screen_fade_counter }
            },
            9172253 => {
            spawn_cinematic_sprite_at(samus::SUITLESS_JUMP, 0, 0, game);
            },
            9172308 => {
            if word!(game.game_time_hours) as i32 - 10i32 & 0x8000 != 0 {
                spawn_cinematic_sprite_at(samus::SUITED_JUMP_HEAD_WITHOUT_HELMET, 2, 0, game);
            } else {
                spawn_cinematic_sprite_at(samus::SUITED_JUMP_HEAD_WITH_HELMET, 2, 0, game);
            }
            spawn_cinematic_sprite_at(samus::SUITED_JUMP_BODY, 0, 0, game);
            },
            9158806 => {
            if word!(game.japanese_text_flag) == 0 {
                j = (j).wrapping_add(6);
            }
            },
            9161125 => { game.spawn_palette_effect(crate::intro_sequence::ZEBES_TEXT_FADE_IN);         },
            9161135 => {
            if word!(game.japanese_text_flag) != 0 {
                CinematicSprites.spawn(intro::PLANET_ZEBES_JAPANESE_TEXT,
                    word!(game.japanese_text_flag),
                    game,
                );
            }
                },
            9161149 => { game.spawn_palette_effect(crate::intro_sequence::ZEBES_TEXT_FADE_OUT);         },
            9171550 => {
            set_cinematic_blend(3, 16, 2, 34, game);
            game.ram().write_byte(0x5d, 100);
            game.ram().write_byte(0x59, 84);
            word!(game.cinematic_var4 = 0);
    crate::flow::set(game, crate::flow::Word::CinematicFunction, (0x8be58ai32) as u16);
                },
            9171588 => {
            game.spawn_palette_effect(ending_effect::ZEBES_FADE_OUT);
            word!(game.palette_buffer[254] = 1);
                },
            9171605 => {
            CinematicSprites.spawn(ending::ZEBES_SILHOUETTE, 0xf295, game);
            word!(game.palette_buffer[0] = 0x7fff);
                },
            9171639 => {
            set_cinematic_blend(17, 2, 2, 17, game);
            game.ram().write_byte(0x59, 124);
            spawn_cinematic_sprite_at(ending::ZEBES_STARS_RIGHT, 0, 0xf27c, game);
            spawn_cinematic_sprite_at(ending::ZEBES_STARS_LEFT, 2, 2, game);
            game.spawn_palette_effect(ending_effect::EXPLOSION_WIDE_FOREGROUND);
            game.spawn_palette_effect(ending_effect::EXPLOSION_SPACE_WHITE_OUT);
            game.spawn_palette_effect(ending_effect::EXPLOSION_WIDE_BACKGROUND);
                },
            9171706 => {
            set_cinematic_blend(3, 18, 2, 51, game);
            game.ram().write_byte(0x58, 116);
            game.ram().write_byte(0x59, 120);
            game.spawn_palette_effect(ending_effect::EXPLOSION_FINALE);
            spawn_cinematic_sprite_at(ending::ZEBES_AFTERGLOW, 6, 6, game);
                },
            9171755 => {
            game.ram().write_byte(0x69, 0);
            game.ram().write_byte(0x6b, 0);
            word! { game, 0x7fff; palette_buffer[0], palette_buffer[128] }
            crate::word_transfers!(game, descending, 16; le(palette_buffer, 31) <- value(0x7fff));
            crate::flow::set(game, crate::flow::Word::CinematicFunction, (0x8bdb9ei32) as u16);
    word!(game.cinematic_var4 = 120);
                },
            9172040 => {
            word! { game, 0; screen_fade_delay, screen_fade_counter }
            crate::flow::set(game, crate::flow::Word::CinematicFunction, (0x8bde64i32) as u16);
                },
            9172484 => {
            word!(game.cinematicbg_arr9[(k >> 1)] = 3584);
            crate::flow::set(game, crate::flow::Word::CinematicFunction, (0x8be3aei32) as u16);
            set_cinematic_blend(17, 16, 2, -127i32 as u8, game);
            word! {game; cinematic_var8 = -4i32 as u16; cinematic_var10 = -8i32 as u16;
                cinematic_var4 = 16; }
            word! { game, 32; cinematic_var14, cinematic_var17 }
            capture_cinematic_palette(game);
            prepare_cinematic_palette_fade(game);
            game.queue_sound(0, 0x22_u8, 15);
                },
            9172561 => {
            word! {game; samus_y_subspeed = 0; samus_y_speed = -16i32 as u16; }
                },
            9152792 => { spawn_cinematic_burst(0xcecd, 0..6, 6, game); game.queue_sound(1, 0xb_u8, 6); },
            9159684 => { spawn_cinematic_burst(0xcebb, 0..5, 0, game); },
            9159948 => { spawn_cinematic_burst(0xcec7, 0..4, 0, game); },
            9157153 => { spawn_cinematic_burst(0xcf21, 0..2, 0, game); },
            9157174 => { spawn_cinematic_burst(0xcf21, 2..4, 0, game); },
            9171888 => { CinematicSprites.spawn(ending::COMPLETED_SUCCESSFULLY, 0xf3b0, game); },
            9171918 => { CinematicSprites.spawn(ending::CLEAR_TIME, 0xf3ce, game); prepare_ending_time(game); },
            9171995 | 9172004 | 9172013 | 9172022 | 9172031 => {
                let digit = ((address - 9171995) / 9) as u16;
                CinematicSprites.spawn(ending::CLEAR_TIME_DIGITS + 6 * digit, 0xf41b + 9 * digit, game);
            },
            9161159 => { crate::intro_objects::advance_zebes_flight(0, game); },
            9172375 => j = crate::ending_screens::position_ending_samus_head(k, false, j, game),
            9172410 => j = crate::ending_screens::position_ending_samus_head(k, true, j, game),
            9149665 => j = crate::title_screen::start_title_pan(0, j, game),
            9149789 => j = crate::title_screen::start_title_pan(1, j, game),
            9149910 => j = crate::title_screen::start_title_pan(2, j, game),
            9156805 => j = start_intro_rinka(k, j, game),
            9155382 => j = crate::intro_pages::start_page(0xb35f, j, game),
            9155390 => j = crate::intro_pages::start_page(0xb370, j, game),
            9155398 => j = crate::intro_pages::start_page(0xb381, j, game),
            9155406 => j = crate::intro_pages::start_page(0xb392, j, game),
            _ => return None,
        }
        Some(j)
        })
}

pub fn spawn_cinematic_sprite_at<H: crate::menus::host::Host + ?Sized>(
    definition: u16,
    slot: u16,
    parameter: u16,
    game: &mut H,
) -> u8 {
    word!(game.cinematic_spawn_param = parameter);
    initialize_cinematic_object(CinematicObjects::Sprites, definition, slot, game)
}

fn initialize_cinematic_object<H: crate::menus::host::Host + ?Sized>(
    objects: CinematicObjects,
    definition: u16,
    object: u16,
    game: &mut H,
) -> u8 {
    use CinematicObjects::*;
    let layout = objects.layout();
    let offset = usize::from(object & !1);
    let stride = 2 * usize::from(layout.slots);
    let record = cinematic_native_record::<0x8b>(definition, game);
    checked_word!(game, layout.instructions + stride + offset; = record.word(2));
    checked_word!(game, layout.instructions + offset; = record.word(4));
    checked_word!(game, layout.instructions + 2 * stride + offset; = 1);
    for field in layout.cleared {
        checked_word!(game, field + offset; = 0);
    }
    // Setup is read after publication, and Mode7 alone receives a word index.
    let setup = u32::from(record.word(0)) | 0x8b0000;
    drop(record);
    match objects {
        Sprites => initialize_cinematic_sprite(setup, object, false, game),
        Mode7 => initialize_cinematic_sprite(setup, object >> 1, true, game),
        Background => {
            if setup != 0x8b93d9 {
                game.rtl_unreachable();
            }
        }
    }
    0
}

fn initialize_cinematic_sprite<H: crate::menus::host::Host + ?Sized>(
    setup: u32,
    object: u16,
    mode7: bool,
    game: &mut H,
) {
    const EGG_POSITIONS: [i16; 12] = [
        0x5c, 0x58, 0x63, 0x58, 0x59, 0x5d, 0x60, 0x5b, 0x66, 0x5e, 0x63, 0x60,
    ];
    let slot = usize::from(object >> 1);
    match setup {
        9149210 => place_cinematic_sprite(object, (128, 48), 1024, game),
        9149229 => place_cinematic_sprite(object, (128, 81), 0, game),
        9149248 => place_cinematic_sprite(object, (128, 196), 2048, game),
        9149628 | 9149770 | 9149891 | 9150021 => {
            place_cinematic_sprite(object, (129, 112), 512, game)
        }
        9150131 => initialize_title_sprite(object, 0, game),
        9150166 => initialize_title_sprite(object, 1, game),
        9150207 => initialize_title_sprite(object, 2, game),
        9147353 => {}
        _ if mode7 => {
            game.rtl_unreachable();
        }
        9152856 => {
            let parameter = word!(game.cinematic_spawn_param);
            word!(game.cinematicspr_goto_timer[slot] = word!(game.cinematic_spawn_param));
            let position = usize::from(4u16.wrapping_mul(parameter) >> 1);
            word! { game;
                cinematicbg_arr7[slot] = (EGG_POSITIONS[position] as u16).wrapping_add(16);
                cinematicbg_arr8[slot] = (EGG_POSITIONS[position + 1] as u16).wrapping_add(59);
                cinematicbg_arr9[slot] = 3584;
            }
        }
        9153178 => {
            word! { game;
                cinematicspr_goto_timer[slot] = word!(game.cinematic_spawn_param);
                cinematicbg_arr7[slot] = word!(game.cinematicbg_arr7[13]);
                cinematicbg_arr8[slot] = word!(game.cinematicbg_arr8[13]);
                cinematicbg_arr9[slot] = 3584;
            }
        }
        9158270 => {
            if word!(game.cinematic_spawn_param) != 0 {
                word!(game.cinematicspr_preinstr_func[slot] = 0xbfc6);
                word!(game.cinematicbg_arr7[slot] = -32i32 as u16);
            } else {
                word!(game.cinematicspr_goto_timer[slot] = -1024i32 as u16);
                word!(game.cinematicbg_arr7[slot] = 112);
            }
            word! {game; cinematicbg_arr8[slot] = 87; cinematicbg_arr9[slot] = 2048; }
        }
        9158560 => {
            if word!(game.cinematic_spawn_param) != 0 {
                word!(game.cinematicbg_arr7[slot] = 224);
            } else {
                word!(game.cinematicbg_arr7[slot] = 112);
                word!(game.cinematicspr_preinstr_func[slot] = 0xbfd9);
            }
            word! {game; cinematicbg_arr8[slot] = 87; cinematicbg_arr9[slot] = 2048; }
        }
        9171343 => initialize_ending_sprite_pair(object, true, game),
        9171368 => initialize_ending_sprite_pair(object, false, game),
        9170841 => update_ending_critters(object, true, game),
        9157005 => initialize_intro_brain_particle(object, true, game),
        9157076 => initialize_intro_brain_particle(object, false, game),
        9159732 => initialize_ceres_explosion(object, CeresExplosion::First, game),
        9159865 => initialize_ceres_explosion(object, CeresExplosion::Second, game),
        9159987 => initialize_ceres_explosion(object, CeresExplosion::Third, game),
        9160105 => initialize_ceres_explosion(object, CeresExplosion::Final, game),
        9156758 => initialize_intro_rinka(object, game),
        9171038 => initialize_ending_time_glyph(object, 0, game),
        9171053 => initialize_ending_time_glyph(object, 1, game),
        9171068 => initialize_ending_time_glyph(object, 2, game),
        9171077 => initialize_ending_time_glyph(object, 3, game),
        9171092 => initialize_ending_time_glyph(object, 4, game),
        9171122 => initialize_ending_cloud_band(object, -96i32 as u16, game),
        9171169 => initialize_ending_cloud_band(object, -32i32 as u16, game),
        9171177 => initialize_ending_cloud_band(object, 288, game),
        9171185 => initialize_ending_cloud_band(object, 352, game),
        9171193 => initialize_ending_cloud_side(object, true, game),
        9171230 => initialize_ending_cloud_side(object, false, game),
        9152705 => place_cinematic_sprite(object, (8, 24), 3072, game),
        9152725 | 9157195 => place_cinematic_sprite(object, (112, 155), 3584, game),
        9153877 => place_cinematic_sprite(object, (84, 139), 3072, game),
        9153939 => place_cinematic_sprite(object, (112, 111), 3072, game),
        9154055 => place_cinematic_sprite(object, (128, 216), 3072, game),
        9156467 => place_cinematic_sprite(object, (56, 111), 3584, game),
        9158434 => place_cinematic_sprite(object, (80, 159), 2048, game),
        9158476 => place_cinematic_sprite(object, (116, 160), 3072, game),
        9158518 => place_cinematic_sprite(object, (128, 96), 2048, game),
        9158787 => place_cinematic_sprite(object, (124, 186), 0, game),
        9158834 => place_cinematic_sprite(object, (124, 204), 0, game),
        9160763 => place_cinematic_sprite(object, (136, 111), 3584, game),
        9160855 => place_cinematic_sprite(object, (128, 127), 2048, game),
        9161026 => place_cinematic_sprite(object, (48, 47), 2048, game),
        9161046 => place_cinematic_sprite(object, (208, 47), 2048, game),
        9161066 => place_cinematic_sprite(object, (48, 207), 2048, game),
        9161086 => place_cinematic_sprite(object, (208, 207), 2048, game),
        9161106 => place_cinematic_sprite(object, (128, 186), 0, game),
        9161190 => place_cinematic_sprite(object, (128, 204), 0, game),
        9170922 => place_cinematic_sprite(object, (128, 128), 3584, game),
        9170941 => place_cinematic_sprite(object, (-128i32 as u16, 128), 3584, game),
        9170949 => place_cinematic_sprite(object, (128, 128), 2560, game),
        9170968 => place_cinematic_sprite(object, (128, 128), 3072, game),
        9170987 => place_cinematic_sprite(object, (128, 96), 1024, game),
        9171006 => place_cinematic_sprite(object, (128, 160), 512, game),
        9171267 => place_cinematic_sprite(object, (120, 136), 2560, game),
        9171286 => place_cinematic_sprite(object, (120, 152), 3072, game),
        9171305 => place_cinematic_sprite(object, (124, 108), 3072, game),
        9171324 => place_cinematic_sprite(object, (121, 107), 2560, game),
        9171393 => place_cinematic_sprite(object, (385, 366), 3584, game),
        9171412 => place_cinematic_sprite(object, (391, 384), 3584, game),
        _ => {
            game.rtl_unreachable();
        }
    }
}

#[forbid(unsafe_code)]
pub fn cinematic_mode7_command<H: crate::menus::host::Host + ?Sized>(
    address: u32,
    operand: u16,
    game: &mut H,
) -> u16 {
    use crate::intro_pages;
    crate::game_callback_dispatch!(@body [checked] {
    match address {
        9151067 => title_metroid_cry(0x23, operand, game),
        9151075 => title_metroid_cry(0x26, operand, game),
        9151083 => title_metroid_cry(0x27, operand, game),
        9151091 => title_metroid_palette(METROID_PAL_1, operand, game),
        9151108 => title_metroid_palette(METROID_PAL_2, operand, game),
        9151125 => title_metroid_palette(METROID_PAL_3, operand, game),
        9151142 => title_metroid_palette(METROID_PAL_4, operand, game),
        9155870 => intro_pages::draw_japanese_text(&[0xcf75, 0xcfbd], true, operand, game),
        9155919 => intro_pages::enable_text(operand, game), 9155924 => intro_pages::draw_japanese_text(&[0xcffd, 0xd055], true, operand, game),
        9156024 => intro_pages::wait_after_japanese_text(0xaf6c, operand, game),
        9156035 => intro_pages::draw_japanese_text(&[0xd0f9], true, operand, game), 9156123 => intro_pages::wait_after_japanese_text(0xb0f2, operand, game),
        9156134 => intro_pages::draw_japanese_text(&[0xd1b9, 0xd215], true, operand, game), 9156234 => intro_pages::wait_after_japanese_text(0xb123, operand, game),
        9156245 => intro_pages::draw_japanese_text(&[0xd2d5], true, operand, game), 9156333 => intro_pages::wait_after_japanese_text(0xb1da, operand, game),
        9156344 => intro_pages::draw_japanese_text(&[0xd371], false, operand, game), 9156363 => intro_pages::enable_text(operand, game),
        _ => game.rtl_unreachable() as u16,
    }
    })
}

#[forbid(unsafe_code)]
pub fn cinematic_mode7_pre<H: crate::menus::host::Host + ?Sized>(
    address: u32,
    object: u16,
    game: &mut H,
) {
    use crate::intro_pages;
    crate::game_callback_dispatch!(@body [checked] {
    match address {
        9147353 => {}, 9155973 => intro_pages::continue_japanese_text(54181, &[0xd085, 0xd0e1], object, game),
        9156078 => intro_pages::continue_japanese_text(54203, &[0xd15d], object, game), 9156183 => intro_pages::continue_japanese_text(54225, &[0xd259, 0xd2a5], object, game),
        9156288 => intro_pages::continue_japanese_text(0xd3e7, &[0xd30d], object, game),
        _ => { game.rtl_unreachable(); },
    }
    })
}

fn advance_cinematic_object<const BANK: u8>(
    objects: CinematicObjects,
    object: u16,
    game: &mut (impl crate::menus::host::Host + ?Sized),
) {
    use CinematicObjects::*;
    let layout = objects.layout();
    let stride = 2 * usize::from(layout.slots);
    {
        let pre = u32::from(checked_word!(game,
            layout.instructions + stride + usize::from(object & !1);))
            | 0x8b0000;
        match objects {
            Sprites => game.cinematic_sprite_pre_instruction(pre, object),
            Mode7 => cinematic_mode7_pre(pre, object, game),
            Background => advance_cinematic_background_pre(pre, object, game),
        }
        let selected = checked_word!(game, layout.current;);
        let offset = usize::from(selected & !1);
        let timer_at = layout.instructions + 2 * stride + offset;
        let timer = checked_word!(game, timer_at;);
        checked_word!(game, timer_at; = timer.wrapping_sub(1));
        if timer != 1 {
            return;
        }
        let mut address = checked_word!(game, layout.instructions + offset;);
        let duration = loop {
            let instruction = cinematic_native_record::<BANK>(address, game).word(0);
            if instruction & 0x8000 == 0 {
                break instruction;
            }
            let operand = address.wrapping_add(2);
            address = match (objects, instruction) {
                (Sprites, 0x9438) | (Mode7, 0x956b) | (Background, 0x9698) => {
                    if !matches!(objects, Mode7) {
                        checked_word!(game, layout.cleared[0] + offset; = 0);
                    }
                    checked_word!(game, layout.instructions + offset; = 0);
                    0
                }
                (Sprites, 0x9442) => {
                    checked_word!(game, layout.instructions + offset; = operand.wrapping_sub(2));
                    0
                }
                (Sprites, 0x944c | 0x94d6) | (Mode7, 0x9572 | 0x9597) => {
                    let plane = if matches!(instruction, 0x94d6 | 0x9597) {
                        3
                    } else {
                        1
                    };
                    checked_word!(game, layout.instructions + plane * stride + offset;
                        = cinematic_native_record::<0x8b>(operand, game).word(0));
                    operand.wrapping_add(2)
                }
                (Sprites, 0x9457) | (Mode7, 0x957d) => {
                    checked_word!(game, layout.instructions + stride + offset;
                        = if matches!(objects, Sprites) { 0x945f } else { 0x9585 });
                    operand
                }
                (Sprites, 0x94a2 | 0x94bc | 0x94c3 | 0x94cd)
                | (Mode7, 0x9586 | 0x958d)
                | (Background, 0x971e) => {
                    let relative = matches!(instruction, 0x94a2 | 0x94cd);
                    let skip = matches!(instruction, 0x94c3 | 0x94cd | 0x958d) && {
                        let counter_at = layout.instructions + 3 * stride + offset;
                        let counter = checked_word!(game, counter_at;);
                        checked_word!(game, counter_at; = counter.wrapping_sub(1));
                        counter == 1
                    };
                    if skip {
                        // Skipping a jump must not read its ROM operand.
                        operand.wrapping_add(if relative { 1 } else { 2 })
                    } else {
                        let target = cinematic_native_record::<BANK>(operand, game);
                        if relative {
                            operand.wrapping_add((target.byte(0) as i8) as u16)
                        } else {
                            target.word(0)
                        }
                    }
                }
                _ => {
                    let command = u32::from(instruction) | 0x8b0000;
                    match objects {
                        Sprites => game.cinematic_sprite_instruction(command, selected, operand),
                        Mode7 => cinematic_mode7_command(command, operand, game),
                        Background => {
                            cinematic_background_command(command, selected, operand, game)
                        }
                    }
                }
            };
            if address == 0 {
                return;
            }
        };
        checked_word!(game, timer_at; = duration);
        let payload = cinematic_native_record::<BANK>(address, game);
        if matches!(objects, Background) {
            word!(game.cinematicbg_arr1[(selected >> 1)] = payload.word(4));
        }
        let value = payload.word(2);
        drop(payload);
        match objects {
            Sprites => word!(game.cinematicspr_whattodraw[(selected >> 1)] = value),
            Mode7 => game.enqueue_mode7_rom(0x8b, value),
            Background => draw_cinematic_background(selected, value, game),
        }
        let size = if matches!(objects, Background) { 6 } else { 4 };
        checked_word!(game, layout.instructions + offset; = address.wrapping_add(size));
    }
}

#[forbid(unsafe_code)]
pub fn advance_cinematic_background_pre<H: crate::menus::host::Host + ?Sized>(
    command: u32,
    object: u16,
    game: &mut H,
) {
    match command {
        0x8b93d9 => {}
        0x8bb4bc => {
            if crate::flow::current(game).cinematic_function == 0xb207
                || crate::flow::current(game).cinematic_function == 0xa390
            {
                word!(game.cinematicbg_instr_ptr[(object >> 1)] = 0xd613);
                word!(game.cinematicbg_instr_timer[(object >> 1)] = 1);
                word!(game.cinematicbg_preinstr[(object >> 1)] = 0x93d9);
            }
        }
        _ => {
            game.rtl_unreachable();
        }
    }
}

#[forbid(unsafe_code)]
fn draw_cinematic_background<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    position: u16,
    game: &mut H,
) {
    let definition = word!(game.cinematicbg_arr1[(object >> 1)]);
    let command = cinematic_native_record::<0x8c>(definition, game).word(0);
    match command {
        0x8849 => {}
        0x884d | 0x88b7 | 0x88fd => {
            if command == 0x884d {
                word!(game.cinematicbg_var1 = u16::from(word!(game.cinematicbg_var1) == 0));
                crate::intro_pages::spawn_text_glow(definition, position, game);
                let record = cinematic_native_record::<0x8c>(
                    word!(game.cinematicbg_instr_ptr[(object >> 1)]),
                    game,
                );
                let bottom = if (record.word(6) as i16) < 0 {
                    word!(game.cinematicbg_arr7[15] = 8);
                    8 * (u16::from(record.byte(3)) + 2)
                } else {
                    word!(game.cinematicbg_arr7[15] = 8 * u16::from(record.byte(8)));
                    8 * u16::from(record.byte(9))
                };
                word!(game.cinematicbg_arr8[15] = bottom.wrapping_sub(8));
                if record.word(4) != 0xd67d && word!(game.cinematicbg_var1) != 0 {
                    game.queue_sound(2, 0xd_u8, 6);
                }
            }
            draw_cinematic_rectangle(command != 0x88fd, definition, position, game);
        }
        0x896b | 0x89cf => {
            let mut destination =
                u16::from(position as u8) + 128 * u16::from(position.to_ne_bytes()[1]);
            let record = cinematic_native_record::<0x8c>(definition, game);
            let width = u16::from(record.byte(2));
            let height = u16::from(record.byte(3));
            let columns = command == 0x89cf;
            let (length, mut remaining) = if columns {
                (height, i32::from(width))
            } else {
                (width, i32::from(height))
            };
            let mut source = definition.wrapping_add(4);
            let mut tail = word!(game.mode7_vram_write_queue_tail);
            loop {
                for (field, value) in [
                    (0, 128),
                    (1, source),
                    (3, 139),
                    (4, length),
                    (6, destination),
                    (8, if columns { 2 } else { 0 }),
                ] {
                    crate::cinematic::write_mode7_word(tail, field, value, game);
                }
                tail = tail.wrapping_add(9);
                source = source.wrapping_add(length);
                destination = destination.wrapping_add(if columns { 1 } else { 128 });
                // Row counts wrap at u16; column counts retain their signed decrement.
                remaining = if columns {
                    remaining - 1
                } else {
                    i32::from((remaining as u16).wrapping_sub(1))
                };
                if remaining == 0 {
                    break;
                }
            }
            word!(game.mode7_vram_write_queue_tail = tail);
        }
        _ => {
            game.rtl_unreachable();
        }
    }
}

fn cinematic_background_command<H: crate::menus::host::Host + ?Sized>(
    command: u32,
    object: u16,
    operand: u16,
    game: &mut H,
) -> u16 {
    use crate::intro_pages;
    {
        match command {
            9154004 => intro_pages::reset_text_marker(game),
            9168423 => draw_cinematic_item_percentage(game),
            9168745 => {
                if word!(game.japanese_text_flag) != 0 {
                    crate::word_transfers!(game, descending, 64; pause_map(799) <- rom(0x8cdf5b, 63));
                }
            }
            9168768 => {
                crate::word_transfers!(game, descending, 64; native(0x363e) <- value(127));
                crate::flow::set(game, crate::flow::Word::CinematicFunction, 0xe797);
            }
            9154115 | 9154169 | 9154676 | 9154739 | 9154971 | 9155112 => {
                let definition = match command {
                    9154115 => intro_mode7::JAPANESE_TEXT_PAGE_1,
                    9154169 => intro_mode7::JAPANESE_TEXT_PAGE_2,
                    9154676 => intro_mode7::JAPANESE_TEXT_PAGE_3,
                    9154739 => intro_mode7::JAPANESE_TEXT_PAGE_4,
                    9154971 => intro_mode7::JAPANESE_TEXT_PAGE_5,
                    _ => intro_mode7::JAPANESE_TEXT_PAGE_6,
                };
                return intro_pages::create_japanese_text(definition, operand, game);
            }
            9154139 | 9154193 | 9154700 | 9154763 | 9154995 => {
                let (next, continuation) = match command {
                    9154139 => (0xaeb8, None),
                    9154193 => (0xaf6c, Some(0xd40d)),
                    9154700 => (0xb0f2, Some(0xd419)),
                    9154763 => (0xb123, Some(0xd425)),
                    _ => (0xb1da, Some(0xd431)),
                };
                return intro_pages::spawn_marker(next, continuation, operand, game);
            }
            9155136 => return intro_pages::finish_pages(operand, game),
            9158806 | 9158818 => {
                return execute_cinematic_sprite_instruction(command, object, operand, game)
                    .expect("known cinematic instruction");
            }
            _ => return game.rtl_unreachable() as u16,
        }
    }
    operand
}

pub fn draw_cinematic_sprites<H: crate::menus::host::Host + ?Sized>(ending: bool, game: &mut H) {
    use crate::sprite_drawing::Clip;
    let layer1 = crate::camera::current(game).layer1;
    for slot in (0..16usize).rev() {
        let sprite = word!(game.cinematicspr_whattodraw[slot]);
        if sprite == 0 {
            continue;
        }
        let palette = word!(game.cinematicbg_arr9[slot]);
        let x = word!(game.cinematicbg_arr7[slot]).wrapping_sub(layer1[0]);
        let y = word!(game.cinematicbg_arr8[slot]).wrapping_sub(layer1[1]);
        if y.wrapping_add(128) < 0x1ff {
            let clip = if y & 0xff00 != 0 {
                Clip::BoundedOutside
            } else {
                Clip::Bounded
            };
            crate::sprite_output::paletted(0x8c, sprite, (x, y), palette, clip, game);
        }
    }
    if ending
        && crate::flow::current(game)
            .cinematic_function
            .wrapping_sub(0xda02)
            & 0x8000
            != 0
        && word!(game.oam_next_ptr) < 0x200
    {
        let first = usize::from(word!(game.oam_next_ptr) >> 2);
        let group = usize::from(word!(game.oam_next_ptr) >> 5);
        word!(game.oam_ext[group] |= 0x5555u16 << (2 * (first & 7)));
        for group in group + 1..16 {
            word!(game.oam_ext[group] = 0x5555);
        }
        // The extension writes precede rereading the first unused OAM entry.
        let first = usize::from(word!(game.oam_next_ptr) >> 2);
        crate::word_transfers!(game, 4, 128 - first; native(0x370 + 4 * first) <- value(0x80));
    }
}
