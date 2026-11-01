//! The cinematic object engine: sprite, Mode 7 and background objects, their scripts and drawing.
use crate::cinematic_state::{Array, Cinematic, Word, cinematic};
use crate::ending_graphics::draw_cinematic_item_percentage;
use crate::ram::RawRam as _;
use crate::ram::word;
use crate::visual_state::CeresExplosion;
use crate::{cinematic::*, ending_screens::*, intro_objects::*, title_screen::*};

use crate::ending_sequence::object as ending;
use crate::intro_sequence::{mode7 as intro_mode7, object as intro};
use crate::title_sequence::object as title;
const _: () = {
    use crate::constants::bank_8d as palfx;
    use crate::ending_sequence::palette_effect as e;
    assert!(crate::intro_sequence::ZEBES_TEXT_FADE_IN == palfx::ADDR_K_PALFX_E1B0 as u16);
    assert!(e::ZEBES_FADE_OUT == palfx::ADDR_K_PALFX_E1C4 as u16);
    assert!(e::EXPLOSION_WIDE_FOREGROUND == palfx::ADDR_K_PALFX_E1C8 as u16);
    assert!(e::EXPLOSION_FINALE == palfx::ADDR_K_PALFX_E1CC as u16);
    assert!(e::EXPLOSION_SPACE_WHITE_OUT == palfx::ADDR_K_PALFX_E1D0 as u16);
    assert!(e::EXPLOSION_WIDE_BACKGROUND == palfx::ADDR_K_PALFX_E1E8 as u16);
};

#[derive(Clone, Copy)]
pub enum CinematicObjects {
    Sprites,
    Mode7,
    Background,
}

/// A cinematic object table's words in the cinematic state: its slot count,
/// selected object and spawn parameter, the first of its four planes
/// (instruction pointers, pre-instructions, timers, then the goto counters)
/// and the tables a spawn clears.
struct CinematicObjectLayout {
    slots: u16,
    current: Word,
    instructions: Array,
    parameter: Word,
    cleared: &'static [Array],
}

impl CinematicObjectLayout {
    /// Object `object`'s word in `plane`, as an index from the instruction
    /// pointers.
    fn index(&self, plane: u16, object: u16) -> u16 {
        (plane * self.slots) + (object >> 1)
    }
    fn get(&self, cine: &mut Cinematic, plane: u16, object: u16) -> u16 {
        *cine.array(self.instructions, self.index(plane, object))
    }
    fn set(&self, cine: &mut Cinematic, plane: u16, object: u16, value: u16) {
        *cine.array(self.instructions, self.index(plane, object)) = value;
    }
}

/// The planes after the instruction pointers.
const PRE_INSTRUCTION: u16 = 1;
const TIMER: u16 = 2;
const COUNTER: u16 = 3;

impl CinematicObjects {
    fn layout(self) -> CinematicObjectLayout {
        match self {
            Self::Sprites => CinematicObjectLayout {
                slots: 16,
                current: Word::ObjectIndex,
                instructions: Array::SprInstructionPointer,
                parameter: Word::SpawnParameter,
                cleared: &[
                    Array::SprWhatToDraw,
                    Array::SprGotoTimer,
                    Array::SprArr6,
                    Array::SprArr7,
                ],
            },
            Self::Mode7 => CinematicObjectLayout {
                slots: 2,
                current: Word::Mode7ObjectIndex,
                instructions: Array::Mode7InstructionPointer,
                parameter: Word::Mode7SpawnParameter,
                cleared: &[Array::Mode7GotoTimer],
            },
            Self::Background => CinematicObjectLayout {
                slots: 4,
                current: Word::BgObjectIndex,
                instructions: Array::BgInstructionPointer,
                parameter: Word::BgVramAddress,
                cleared: &[Array::BgArr1, Array::BgArr6],
            },
        }
    }

    pub fn spawn<H: crate::menus::host::Host + ?Sized>(
        self,
        definition: u16,
        parameter: u16,
        cine: &mut Cinematic,
        game: &mut H,
    ) -> u8 {
        let layout = self.layout();
        *cine.word(layout.parameter) = parameter;
        for slot in (0..layout.slots).rev() {
            if layout.get(cine, 0, slot * 2) == 0 {
                return initialize_cinematic_object(self, definition, slot * 2, cine, game);
            }
        }
        1
    }

    pub fn advance<H: crate::menus::host::Host + ?Sized>(self, cine: &mut Cinematic, game: &mut H) {
        use crate::visual_state::CinematicTilemap;
        use CinematicObjects::Background;
        if matches!(self, Background) && *cine.enable_objects() & 0x8000 == 0 {
            return;
        }
        let layout = self.layout();
        let mut object = i32::from(2 * (layout.slots - 1));
        while object >= 0 {
            *cine.word(layout.current) = object as u16;
            if layout.get(cine, 0, object as u16) != 0 {
                match self {
                    Background => advance_cinematic_object::<0x8c>(self, object as u16, cine, game),
                    _ => advance_cinematic_object::<0x8b>(self, object as u16, cine, game),
                }
                // A callback may redirect the scan outside its original slot range.
                object = i32::from(*cine.word(layout.current));
            }
            object -= 2;
        }
        if matches!(self, Background) {
            if *cine.enable_bg_tilemap() & 0x8000 != 0 {
                queue_cinematic_tilemap(CinematicTilemap::Text, cine, game);
            }
            queue_cinematic_tilemap(CinematicTilemap::Eyes, cine, game);
        }
    }
}

pub fn spawn_cinematic_burst<H: crate::menus::host::Host + ?Sized>(
    mut definition: u16,
    parameters: impl IntoIterator<Item = u16>,
    stride: u16,
    cine: &mut Cinematic,
    game: &mut H,
) {
    for parameter in parameters {
        CinematicObjects::Sprites.spawn(definition, parameter, cine, game);
        definition = definition.wrapping_add(stride);
    }
}

pub fn advance_intro_mother_brain<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    stage: u8,
    cine: &mut Cinematic,
    game: &mut H,
) {
    let slot = usize::from(object >> 1);
    if stage != 2 {
        let timer = *cine.spr_arr6(slot);
        if timer != 0 {
            if timer & 1 != 0 {
                crate::word_transfers!(game, descending, 16; native(0xc1fe) <- rom(0x8ce3e9, 255));
            } else {
                crate::word_transfers!(game, descending, 16; native(0xc1fe) <- value(0x7fff));
            }
            *cine.spr_arr6(slot) = cine.spr_arr6(slot).wrapping_sub(1);
        }
    }
    if stage == 0 {
        let Some(projectile) = (0..5).rfind(|&i| cinematic!(game.ProjectileType[i]) & 0xfff == 256)
        else {
            return;
        };
        if (cinematic!(game.ProjectileX[projectile]) as i32 - 84) & 0x8000 == 0 {
            return;
        }
        game.retire_projectile(2 * projectile as u16);
        *cine.spr_arr6(slot) = 8;
    } else {
        if *cine.frame_counter() & 1 != 0 {
            *cine.y() = cine.y().wrapping_sub(4);
        } else {
            *cine.y() = cine.y().wrapping_add(4);
        }
        if stage == 2 {
            crate::intro_pages::wait_for_scene_end(object, true, cine);
            return;
        }
    }
    let count = (*cine.spr_goto_timer(slot)).wrapping_add(1);
    *cine.spr_goto_timer(slot) = count;
    if stage == 0 && count == 4 {
        *cine.spr_goto_timer(slot) = 0;
        *cine.spr_pre_instruction(slot) = 0xb80f;
        spawn_cinematic_burst(0xcf1b, 0..3, 0, cine, game);
        spawn_cinematic_burst(0xcf15, 0..5, 0, cine, game);
    }
    if stage == 1 && (count as i32 - 128) & 0x8000 == 0 {
        resume_cinematic_sprite(slot, 0xcb19, cine);
    }
}

pub fn execute_cinematic_sprite_instruction<H: crate::menus::host::Host + ?Sized>(
    address: u32,
    k: u16,
    mut j: u16,
    cine: &mut Cinematic,
    game: &mut H,
) -> Option<u16> {
    use crate::ending_sequence::palette_effect as ending_effect;
    use CinematicObjects::Sprites as CinematicSprites;
    crate::game_callback_dispatch!(@body [checked] {
        match address {
            9150157 => {
            let v0: u16 = 0x9ecd;
            CinematicSprites.spawn(title::COPYRIGHT_FADE_IN, v0, cine, game);
            },
            9150192 => {
            *cine.bg_arr9(k >> 1) = 0;
            CinematicSprites.spawn(title::COPYRIGHT_FADE_IN, 0, cine, game);
            },
            9150233 => {
            crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| flow.demo_timer = 900);
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| flow.cinematic_function = 0x9f29);
            },
            9158818 => {
            crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| flow.cinematic_function = 0xc0c5);
            crate::screen_fade::set_both(game, 1, 1);
            },
            // The instruction's own address is in A at $F51D.
            9172253 => {
            spawn_cinematic_sprite_at(samus::SUITLESS_JUMP, 0, 0xf51d, cine, game);
            },
            // A is the slot for the head, then the palette its initializer loaded.
            9172308 => {
            let head = if (word!(game.GAME_TIME_HOURS) as i32 - 10i32) & 0x8000 != 0 {
                samus::SUITED_JUMP_HEAD_WITHOUT_HELMET
            } else {
                samus::SUITED_JUMP_HEAD_WITH_HELMET
            };
            spawn_cinematic_sprite_at(head, 2, 2, cine, game);
            let parameter = crate::ending_screens::initializer_palette(head);
            spawn_cinematic_sprite_at(samus::SUITED_JUMP_BODY, 0, parameter, cine, game);
            },
            9158806 => {
            if cinematic!(game.JapaneseText) == 0 {
                j = (j).wrapping_add(6);
            }
            },
            9161125 => { crate::palette_effects::execution::leased::spawn(crate::intro_sequence::ZEBES_TEXT_FADE_IN, game);         },
            9161135 => {
            if cinematic!(game.JapaneseText) != 0 {
                CinematicSprites.spawn(intro::PLANET_ZEBES_JAPANESE_TEXT,
                    cinematic!(game.JapaneseText),
                    cine,
                    game,
                );
            }
                },
            9161149 => { crate::palette_effects::execution::leased::spawn(crate::intro_sequence::ZEBES_TEXT_FADE_OUT, game);         },
            9171550 => {
            set_cinematic_blend(3, 16, 2, 34, game);
            game.ram().write_byte(0x5d, 100);
            game.ram().write_byte(0x59, 84);
            *cine.function_timer() = 0;
    crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| flow.cinematic_function = 0xe58a);
                },
            9171588 => {
            crate::palette_effects::execution::leased::spawn(ending_effect::ZEBES_FADE_OUT, game);
            cinematic!(game.PaletteBuffer[254] = 1);
                },
            9171605 => {
            CinematicSprites.spawn(ending::ZEBES_SILHOUETTE, 0xf295, cine, game);
            cinematic!(game.PaletteBuffer[0] = 0x7fff);
                },
            9171639 => {
            set_cinematic_blend(17, 2, 2, 17, game);
            game.ram().write_byte(0x59, 124);
            spawn_cinematic_sprite_at(ending::ZEBES_STARS_RIGHT, 0, 0xf27c, cine, game);
            spawn_cinematic_sprite_at(ending::ZEBES_STARS_LEFT, 2, 2, cine, game);
            crate::palette_effects::execution::leased::spawn(ending_effect::EXPLOSION_WIDE_FOREGROUND, game);
            crate::palette_effects::execution::leased::spawn(ending_effect::EXPLOSION_SPACE_WHITE_OUT, game);
            crate::palette_effects::execution::leased::spawn(ending_effect::EXPLOSION_WIDE_BACKGROUND, game);
                },
            9171706 => {
            set_cinematic_blend(3, 18, 2, 51, game);
            game.ram().write_byte(0x58, 116);
            game.ram().write_byte(0x59, 120);
            crate::palette_effects::execution::leased::spawn(ending_effect::EXPLOSION_FINALE, game);
            spawn_cinematic_sprite_at(ending::ZEBES_AFTERGLOW, 6, 6, cine, game);
                },
            9171755 => {
            game.ram().write_byte(0x69, 0);
            game.ram().write_byte(0x6b, 0);
            cinematic! { game, 0x7fff; PaletteBuffer[0], PaletteBuffer[128] }
            crate::word_transfers!(game, descending, 16; le(PALETTE_BUFFER, 31) <- value(0x7fff));
            crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| flow.cinematic_function = 0xdb9e);
    *cine.function_timer() = 120;
                },
            9172040 => {
            crate::screen_fade::set_both(game, 0, 0);
            crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| flow.cinematic_function = 0xde64);
                },
            9172484 => {
            *cine.bg_arr9(k >> 1) = 3584;
            crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| flow.cinematic_function = 0xe3ae);
            set_cinematic_blend(17, 16, 2, -127i32 as u8, game);
            *cine.x() = -4i32 as u16;
            *cine.y() = -8i32 as u16;
            *cine.function_timer() = 16;
            *cine.var14() = 32;
            *cine.var17() = 32;
            capture_cinematic_palette(game);
            prepare_cinematic_palette_fade(game);
            game.queue_sound(0, 0x22_u8, 15);
                },
            9172561 => {
            cinematic! {game; SamusYSubspeed = 0; SamusYSpeed = -16i32 as u16; }
                },
            9152792 => { spawn_cinematic_burst(0xcecd, 0..6, 6, cine, game); game.queue_sound(1, 0xb_u8, 6); },
            9159684 => { spawn_cinematic_burst(0xcebb, 0..5, 0, cine, game); },
            9159948 => { spawn_cinematic_burst(0xcec7, 0..4, 0, cine, game); },
            9157153 => { spawn_cinematic_burst(0xcf21, 0..2, 0, cine, game); },
            9157174 => { spawn_cinematic_burst(0xcf21, 2..4, 0, cine, game); },
            9171888 => { CinematicSprites.spawn(ending::COMPLETED_SUCCESSFULLY, 0xf3b0, cine, game); },
            9171918 => { CinematicSprites.spawn(ending::CLEAR_TIME, 0xf3ce, cine, game); prepare_ending_time(game); },
            9171995 | 9172004 | 9172013 | 9172022 | 9172031 => {
                let digit = ((address - 9171995) / 9) as u16;
                CinematicSprites.spawn(ending::CLEAR_TIME_DIGITS + 6 * digit, 0xf41b + 9 * digit, cine, game);
            },
            9161159 => { crate::intro_objects::advance_zebes_flight(0, cine, game); },
            9172375 => j = crate::ending_screens::position_ending_samus_head(k, false, j, cine, game),
            9172410 => j = crate::ending_screens::position_ending_samus_head(k, true, j, cine, game),
            9149665 => j = crate::title_screen::start_title_pan(0, j, cine, game),
            9149789 => j = crate::title_screen::start_title_pan(1, j, cine, game),
            9149910 => j = crate::title_screen::start_title_pan(2, j, cine, game),
            9156805 => j = start_intro_rinka(k, j, cine),
            9155382 => j = crate::intro_pages::start_page(0xb35f, j, cine, game),
            9155390 => j = crate::intro_pages::start_page(0xb370, j, cine, game),
            9155398 => j = crate::intro_pages::start_page(0xb381, j, cine, game),
            9155406 => j = crate::intro_pages::start_page(0xb392, j, cine, game),
            _ => return None,
        }
        Some(j)
        })
}

pub fn spawn_cinematic_sprite_at<H: crate::menus::host::Host + ?Sized>(
    definition: u16,
    slot: u16,
    parameter: u16,
    cine: &mut Cinematic,
    game: &mut H,
) -> u8 {
    *cine.spawn_parameter() = parameter;
    initialize_cinematic_object(CinematicObjects::Sprites, definition, slot, cine, game)
}

fn initialize_cinematic_object<H: crate::menus::host::Host + ?Sized>(
    objects: CinematicObjects,
    definition: u16,
    object: u16,
    cine: &mut Cinematic,
    game: &mut H,
) -> u8 {
    use CinematicObjects::*;
    let layout = objects.layout();
    let record = cinematic_native_record::<0x8b>(definition, game);
    layout.set(cine, PRE_INSTRUCTION, object, record.word(2));
    layout.set(cine, 0, object, record.word(4));
    layout.set(cine, TIMER, object, 1);
    for &table in layout.cleared {
        *cine.array(table, object >> 1) = 0;
    }
    // Setup is read after publication, and Mode7 alone receives a word index.
    let setup = u32::from(record.word(0)) | 0x8b0000;
    drop(record);
    match objects {
        Sprites => initialize_cinematic_sprite(setup, object, false, cine, game),
        Mode7 => initialize_cinematic_sprite(setup, object >> 1, true, cine, game),
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
    cine: &mut Cinematic,
    game: &mut H,
) {
    const EGG_POSITIONS: [i16; 12] = [
        0x5c, 0x58, 0x63, 0x58, 0x59, 0x5d, 0x60, 0x5b, 0x66, 0x5e, 0x63, 0x60,
    ];
    let slot = usize::from(object >> 1);
    match setup {
        9149210 => place_cinematic_sprite(object, (128, 48), 1024, cine),
        9149229 => place_cinematic_sprite(object, (128, 81), 0, cine),
        9149248 => place_cinematic_sprite(object, (128, 196), 2048, cine),
        9149628 | 9149770 | 9149891 | 9150021 => {
            place_cinematic_sprite(object, (129, 112), 512, cine)
        }
        9150131 => initialize_title_sprite(object, 0, cine, game),
        9150166 => initialize_title_sprite(object, 1, cine, game),
        9150207 => initialize_title_sprite(object, 2, cine, game),
        9147353 => {}
        _ if mode7 => {
            game.rtl_unreachable();
        }
        9152856 => {
            let parameter = *cine.spawn_parameter();
            *cine.spr_goto_timer(slot) = *cine.spawn_parameter();
            let position = usize::from(4u16.wrapping_mul(parameter) >> 1);
            *cine.bg_arr7(slot) = (EGG_POSITIONS[position] as u16).wrapping_add(16);
            *cine.bg_arr8(slot) = (EGG_POSITIONS[position + 1] as u16).wrapping_add(59);
            *cine.bg_arr9(slot) = 3584;
        }
        9153178 => {
            *cine.spr_goto_timer(slot) = *cine.spawn_parameter();
            *cine.bg_arr7(slot) = *cine.bg_arr7(13);
            *cine.bg_arr8(slot) = *cine.bg_arr8(13);
            *cine.bg_arr9(slot) = 3584;
        }
        9158270 => {
            if *cine.spawn_parameter() != 0 {
                *cine.spr_pre_instruction(slot) = 0xbfc6;
                *cine.bg_arr7(slot) = -32i32 as u16;
            } else {
                *cine.spr_goto_timer(slot) = -1024i32 as u16;
                *cine.bg_arr7(slot) = 112;
            }
            *cine.bg_arr8(slot) = 87;
            *cine.bg_arr9(slot) = 2048;
        }
        9158560 => {
            if *cine.spawn_parameter() != 0 {
                *cine.bg_arr7(slot) = 224;
            } else {
                *cine.bg_arr7(slot) = 112;
                *cine.spr_pre_instruction(slot) = 0xbfd9;
            }
            *cine.bg_arr8(slot) = 87;
            *cine.bg_arr9(slot) = 2048;
        }
        9171343 => initialize_ending_sprite_pair(object, true, cine),
        9171368 => initialize_ending_sprite_pair(object, false, cine),
        9170841 => update_ending_critters(object, true, cine),
        9157005 => initialize_intro_brain_particle(object, true, cine),
        9157076 => initialize_intro_brain_particle(object, false, cine),
        9159732 => initialize_ceres_explosion(object, CeresExplosion::First, cine, game),
        9159865 => initialize_ceres_explosion(object, CeresExplosion::Second, cine, game),
        9159987 => initialize_ceres_explosion(object, CeresExplosion::Third, cine, game),
        9160105 => initialize_ceres_explosion(object, CeresExplosion::Final, cine, game),
        9156758 => initialize_intro_rinka(object, cine),
        9171038 => initialize_ending_time_glyph(object, 0, cine, game),
        9171053 => initialize_ending_time_glyph(object, 1, cine, game),
        9171068 => initialize_ending_time_glyph(object, 2, cine, game),
        9171077 => initialize_ending_time_glyph(object, 3, cine, game),
        9171092 => initialize_ending_time_glyph(object, 4, cine, game),
        9171122 => initialize_ending_cloud_band(object, -96i32 as u16, cine),
        9171169 => initialize_ending_cloud_band(object, -32i32 as u16, cine),
        9171177 => initialize_ending_cloud_band(object, 288, cine),
        9171185 => initialize_ending_cloud_band(object, 352, cine),
        9171193 => initialize_ending_cloud_side(object, true, cine),
        9171230 => initialize_ending_cloud_side(object, false, cine),
        9152705 => place_cinematic_sprite(object, (8, 24), 3072, cine),
        9152725 | 9157195 => place_cinematic_sprite(object, (112, 155), 3584, cine),
        9153877 => place_cinematic_sprite(object, (84, 139), 3072, cine),
        9153939 => place_cinematic_sprite(object, (112, 111), 3072, cine),
        9154055 => place_cinematic_sprite(object, (128, 216), 3072, cine),
        9156467 => place_cinematic_sprite(object, (56, 111), 3584, cine),
        9158434 => place_cinematic_sprite(object, (80, 159), 2048, cine),
        9158476 => place_cinematic_sprite(object, (116, 160), 3072, cine),
        9158518 => place_cinematic_sprite(object, (128, 96), 2048, cine),
        9158787 => place_cinematic_sprite(object, (124, 186), 0, cine),
        9158834 => place_cinematic_sprite(object, (124, 204), 0, cine),
        9160763 => place_cinematic_sprite(object, (136, 111), 3584, cine),
        9160855 => place_cinematic_sprite(object, (128, 127), 2048, cine),
        9161026 => place_cinematic_sprite(object, (48, 47), 2048, cine),
        9161046 => place_cinematic_sprite(object, (208, 47), 2048, cine),
        9161066 => place_cinematic_sprite(object, (48, 207), 2048, cine),
        9161086 => place_cinematic_sprite(object, (208, 207), 2048, cine),
        9161106 => place_cinematic_sprite(object, (128, 186), 0, cine),
        9161190 => place_cinematic_sprite(object, (128, 204), 0, cine),
        9170922 => place_cinematic_sprite(object, (128, 128), 3584, cine),
        9170941 => place_cinematic_sprite(object, (-128i32 as u16, 128), 3584, cine),
        9170949 => place_cinematic_sprite(object, (128, 128), 2560, cine),
        9170968 => place_cinematic_sprite(object, (128, 128), 3072, cine),
        9170987 => place_cinematic_sprite(object, (128, 96), 1024, cine),
        9171006 => place_cinematic_sprite(object, (128, 160), 512, cine),
        9171267 => place_cinematic_sprite(object, (120, 136), 2560, cine),
        9171286 => place_cinematic_sprite(object, (120, 152), 3072, cine),
        9171305 => place_cinematic_sprite(object, (124, 108), 3072, cine),
        9171324 => place_cinematic_sprite(object, (121, 107), 2560, cine),
        9171393 => place_cinematic_sprite(object, (385, 366), 3584, cine),
        9171412 => place_cinematic_sprite(object, (391, 384), 3584, cine),
        _ => {
            game.rtl_unreachable();
        }
    }
}

pub fn cinematic_mode7_command<H: crate::menus::host::Host + ?Sized>(
    address: u32,
    operand: u16,
    cine: &mut Cinematic,
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
        9155870 => intro_pages::draw_japanese_text(&[0xcf75, 0xcfbd], true, operand, cine, game),
        9155919 => intro_pages::enable_text(operand, cine), 9155924 => intro_pages::draw_japanese_text(&[0xcffd, 0xd055], true, operand, cine, game),
        9156024 => intro_pages::wait_after_japanese_text(0xaf6c, operand, cine, game),
        9156035 => intro_pages::draw_japanese_text(&[0xd0f9], true, operand, cine, game), 9156123 => intro_pages::wait_after_japanese_text(0xb0f2, operand, cine, game),
        9156134 => intro_pages::draw_japanese_text(&[0xd1b9, 0xd215], true, operand, cine, game), 9156234 => intro_pages::wait_after_japanese_text(0xb123, operand, cine, game),
        9156245 => intro_pages::draw_japanese_text(&[0xd2d5], true, operand, cine, game), 9156333 => intro_pages::wait_after_japanese_text(0xb1da, operand, cine, game),
        9156344 => intro_pages::draw_japanese_text(&[0xd371], false, operand, cine, game), 9156363 => intro_pages::enable_text(operand, cine),
        _ => game.rtl_unreachable() as u16,
    }
    })
}

pub fn cinematic_mode7_pre<H: crate::menus::host::Host + ?Sized>(
    address: u32,
    object: u16,
    cine: &mut Cinematic,
    game: &mut H,
) {
    use crate::intro_pages;
    crate::game_callback_dispatch!(@body [checked] {
    match address {
        9147353 => {}, 9155973 => intro_pages::continue_japanese_text(54181, &[0xd085, 0xd0e1], object, cine, game),
        9156078 => intro_pages::continue_japanese_text(54203, &[0xd15d], object, cine, game), 9156183 => intro_pages::continue_japanese_text(54225, &[0xd259, 0xd2a5], object, cine, game),
        9156288 => intro_pages::continue_japanese_text(0xd3e7, &[0xd30d], object, cine, game),
        _ => { game.rtl_unreachable(); },
    }
    })
}

fn advance_cinematic_object<const BANK: u8>(
    objects: CinematicObjects,
    object: u16,
    cine: &mut Cinematic,
    game: &mut (impl crate::menus::host::Host + ?Sized),
) {
    use CinematicObjects::*;
    let layout = objects.layout();
    let pre = u32::from(layout.get(cine, PRE_INSTRUCTION, object)) | 0x8b0000;
    match objects {
        Sprites => game.cinematic_sprite_pre_instruction(pre, object, cine),
        Mode7 => cinematic_mode7_pre(pre, object, cine, game),
        Background => advance_cinematic_background_pre(pre, object, cine, game),
    }
    let selected = *cine.word(layout.current);
    let timer = layout.get(cine, TIMER, selected);
    layout.set(cine, TIMER, selected, timer.wrapping_sub(1));
    if timer != 1 {
        return;
    }
    let mut address = layout.get(cine, 0, selected);
    let duration = loop {
        let instruction = cinematic_native_record::<BANK>(address, game).word(0);
        if instruction & 0x8000 == 0 {
            break instruction;
        }
        let operand = address.wrapping_add(2);
        address = match (objects, instruction) {
            (Sprites, 0x9438) | (Mode7, 0x956b) | (Background, 0x9698) => {
                if !matches!(objects, Mode7) {
                    *cine.array(layout.cleared[0], selected >> 1) = 0;
                }
                layout.set(cine, 0, selected, 0);
                0
            }
            (Sprites, 0x9442) => {
                layout.set(cine, 0, selected, operand.wrapping_sub(2));
                0
            }
            (Sprites, 0x944c | 0x94d6) | (Mode7, 0x9572 | 0x9597) => {
                let plane = if matches!(instruction, 0x94d6 | 0x9597) {
                    COUNTER
                } else {
                    PRE_INSTRUCTION
                };
                let value = cinematic_native_record::<0x8b>(operand, game).word(0);
                layout.set(cine, plane, selected, value);
                operand.wrapping_add(2)
            }
            (Sprites, 0x9457) | (Mode7, 0x957d) => {
                let next = if matches!(objects, Sprites) {
                    0x945f
                } else {
                    0x9585
                };
                layout.set(cine, PRE_INSTRUCTION, selected, next);
                operand
            }
            (Sprites, 0x94a2 | 0x94bc | 0x94c3 | 0x94cd)
            | (Mode7, 0x9586 | 0x958d)
            | (Background, 0x971e) => {
                let relative = matches!(instruction, 0x94a2 | 0x94cd);
                let skip = matches!(instruction, 0x94c3 | 0x94cd | 0x958d) && {
                    let counter = layout.get(cine, COUNTER, selected);
                    layout.set(cine, COUNTER, selected, counter.wrapping_sub(1));
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
                    Sprites => game.cinematic_sprite_instruction(command, selected, operand, cine),
                    Mode7 => cinematic_mode7_command(command, operand, cine, game),
                    Background => {
                        cinematic_background_command(command, selected, operand, cine, game)
                    }
                }
            }
        };
        if address == 0 {
            return;
        }
    };
    layout.set(cine, TIMER, selected, duration);
    let payload = cinematic_native_record::<BANK>(address, game);
    if matches!(objects, Background) {
        *cine.bg_arr1(selected >> 1) = payload.word(4);
    }
    let value = payload.word(2);
    drop(payload);
    match objects {
        Sprites => *cine.spr_what_to_draw(selected >> 1) = value,
        Mode7 => crate::display::transfer::enqueue_mode7_rom(0x8b, value, game),
        Background => draw_cinematic_background(selected, value, cine, game),
    }
    let size = if matches!(objects, Background) { 6 } else { 4 };
    layout.set(cine, 0, selected, address.wrapping_add(size));
}

pub fn advance_cinematic_background_pre<H: crate::menus::host::Host + ?Sized>(
    command: u32,
    object: u16,
    cine: &mut Cinematic,
    game: &mut H,
) {
    match command {
        0x8b93d9 => {}
        0x8bb4bc => {
            if crate::session_cohort::load::<crate::flow::Flow, _>(game).cinematic_function
                == 0xb207
                || crate::session_cohort::load::<crate::flow::Flow, _>(game).cinematic_function
                    == 0xa390
            {
                *cine.bg_instruction_pointer(object >> 1) = 0xd613;
                *cine.bg_instruction_timer(object >> 1) = 1;
                *cine.bg_pre_instruction(object >> 1) = 0x93d9;
            }
        }
        _ => {
            game.rtl_unreachable();
        }
    }
}

fn draw_cinematic_background<H: crate::menus::host::Host + ?Sized>(
    object: u16,
    position: u16,
    cine: &mut Cinematic,
    game: &mut H,
) {
    let definition = *cine.bg_arr1(object >> 1);
    let command = cinematic_native_record::<0x8c>(definition, game).word(0);
    match command {
        0x8849 => {}
        0x884d | 0x88b7 | 0x88fd => {
            if command == 0x884d {
                *cine.intro_text_click_flag() = u16::from(*cine.intro_text_click_flag() == 0);
                crate::intro_pages::spawn_text_glow(definition, position, cine);
                let record = cinematic_native_record::<0x8c>(
                    *cine.bg_instruction_pointer(object >> 1),
                    game,
                );
                let bottom = if (record.word(6) as i16) < 0 {
                    *cine.bg_arr7(15) = 8;
                    8 * (u16::from(record.byte(3)) + 2)
                } else {
                    *cine.bg_arr7(15) = 8 * u16::from(record.byte(8));
                    8 * u16::from(record.byte(9))
                };
                *cine.bg_arr8(15) = bottom.wrapping_sub(8);
                if record.word(4) != 0xd67d && *cine.intro_text_click_flag() != 0 {
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
            let mut tail = word!(game.MODE7_VRAM_WRITE_QUEUE_TAIL);
            loop {
                for (field, value) in [
                    (0, 128),
                    (1, source),
                    (3, 139),
                    (4, length),
                    (6, destination),
                    (8, if columns { 2 } else { 0 }),
                ] {
                    let at = crate::display::transfer::MODE7_QUEUE + usize::from(tail) + field;
                    game.write_slice_native(at, value);
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
            word!(game.MODE7_VRAM_WRITE_QUEUE_TAIL = tail);
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
    cine: &mut Cinematic,
    game: &mut H,
) -> u16 {
    use crate::intro_pages;
    match command {
        9154004 => intro_pages::reset_text_marker(cine),
        9168423 => draw_cinematic_item_percentage(game),
        9168745 => {
            if cinematic!(game.JapaneseText) != 0 {
                crate::word_transfers!(game, descending, 64; pause_map(799) <- rom(0x8cdf5b, 63));
            }
        }
        9168768 => {
            crate::word_transfers!(game, descending, 64; native(0x363e) <- value(127));
            crate::session_cohort::with(game, |_, flow: &mut crate::flow::Flow| {
                flow.cinematic_function = 0xe797
            });
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
            return intro_pages::create_japanese_text(definition, operand, cine, game);
        }
        9154139 | 9154193 | 9154700 | 9154763 | 9154995 => {
            let (next, continuation) = match command {
                9154139 => (0xaeb8, None),
                9154193 => (0xaf6c, Some(0xd40d)),
                9154700 => (0xb0f2, Some(0xd419)),
                9154763 => (0xb123, Some(0xd425)),
                _ => (0xb1da, Some(0xd431)),
            };
            return intro_pages::spawn_marker(next, continuation, operand, cine, game);
        }
        9155136 => return intro_pages::finish_pages(operand, game),
        9158806 | 9158818 => {
            return execute_cinematic_sprite_instruction(command, object, operand, cine, game)
                .expect("known cinematic instruction");
        }
        _ => return game.rtl_unreachable() as u16,
    }
    operand
}

pub fn draw_cinematic_sprites<H: crate::menus::host::Host + ?Sized>(
    ending: bool,
    cine: &mut Cinematic,
    game: &mut H,
) {
    use crate::sprite_drawing::Clip;
    let layer1 = crate::camera::current(game).layer1;
    for slot in (0..16usize).rev() {
        let sprite = *cine.spr_what_to_draw(slot);
        if sprite == 0 {
            continue;
        }
        let palette = *cine.bg_arr9(slot);
        let x = (*cine.bg_arr7(slot)).wrapping_sub(layer1[0]);
        let y = (*cine.bg_arr8(slot)).wrapping_sub(layer1[1]);
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
        && crate::session_cohort::load::<crate::flow::Flow, _>(game)
            .cinematic_function
            .wrapping_sub(0xda02)
            & 0x8000
            != 0
    {
        crate::sprite_drawing::hide_unused_large(game);
    }
}
