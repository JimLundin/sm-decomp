use crate::game::actors;
use crate::game::ceres_door_data;
use crate::game::ceres_doors;
use crate::game::ceres_ridley_tables;
use crate::game::fire_geysers;
use crate::game::gameplay_flag_runtime;
use crate::game::music::QueueMusic_Delayed8;
use crate::game::player_update::minimap;
pub use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;
use crate::game::session_runtime::{actor_cosine, actor_sine};
use crate::ram_map::slot;
pub use sm_game::actors::ridley::{
    HomingReversal, activate_tail_segments as Ridley_Func_95,
    calculate_swoop_velocities as Ridley_Func_111, ceres_accelerate as Ridley_Func_106,
    deal_contact_damage as Ridley_Func_98, draw_baby_metroid_and_door as Ridley_A2F2,
    draw_tail as sub_A6DB2A, emergency_text_glow as Ridley_Func_61,
    handle_baby_metroid as Ridley_A2DC, handle_tail as Ridley_Func_70, homing_velocity,
    hurt_flash as Ridley_Func_102, initialise_tail as Ridley_Func_92,
    move_ridley as Ridley_Func_112, set_instruction_list as Ridley_Func_99, touch as sub_A6DF59,
    update_wings as Ridley_Func_115,
};
use sm_game::enemy_script::EnemyScript;

pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a6::*;
pub use crate::game::constants::bank_a9::*;

use crate::game::actors::shitroid::separated_from_rect;
use crate::game::enemy_projectiles::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::escape_typewriter::type_escape_text;
use crate::game::gameplay_flag_runtime::SetBossBitForCurArea;
use crate::game::mode7_transfers::enqueue_rom;
use crate::game::music::QueueSfx2_Max6;
use crate::game::palette_copies::{copy_rom_colors_to_palette, copy_rom_colors_to_target_palette};
use crate::game::rom_vram_entries::{
    queue_escape_tile_table, queue_escape_timer_tiles, queue_getaway_tiles,
};
use crate::game::sprite_effect_runtime::create_sprite;
use sm_game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;

const VANISH: u16 = ceres_doors::TANGIBLE | ceres_doors::INVISIBLE;
use crate::game::visual_runtime::project_cinematic_point;

pub type Enemy_Ridley = crate::game::enemy_layouts::EnemyLayout104;

crate::type_aliases! {
    pub Enemy_CeresDoor, Enemy_CeresSteam, Enemy_RidleysExplosion, Enemy_Zebetites
        = crate::game::enemy_layouts::EnemyLayout8;
}

crate::type_aliases! {
    pub Enemy_FakeKraid, Enemy_Boulder =
        crate::game::enemy_layouts::EnemyLayout40;
}

crate::type_aliases! {
    pub Enemy_NuclearWaffle, Enemy_CeresRidley = crate::game::enemy_layouts::EnemyLayout72;
}

crate::typed_struct! {
    [
        #[derive(Copy, Clone)]
        #[repr(C)]
    ];
    pub struct BabyMetroidExecState {
        pub ip, timer: u16;
    }

    pub struct Ram8800_Default {
        pub var_40, var_41, var_42, var_43, var_44, var_45, var_46, var_47, var_48, var_49, var_4A,
            var_4B, var_4C, var_4D, var_4E, var_4F, var_50, var_51, var_52, var_53, var_54, var_55,
            var_56, var_57, var_58, var_59, var_5A, var_5B, var_5C, var_5D, var_5E, var_5F: u16;
    }
}

crate::game::constants::typed_constants! { i32;
    true_0 = 1, false_0 = 0,
}

/// Ridley's "Warning: X undefined" line, for `still_falling_apart`.
pub(crate) fn warn_x_undefined() {
    unsafe { printf(b"Warning: X undefined\n\0" as *const u8 as *const c_char) };
}
static g_word_A695F6: [u16; 4] = [0x190, 0xf0, 0xf0, 0x190];
static g_word_A695FE: [u16; 4] = [0xffe8, 0xfff4, 0x18, 0xc];
static g_word_A69606: [u16; 4] = [0x180, 0x100, 0x100, 0x180];
/// The boulder runs in `sm_game::actors::boulder`.
pub fn Boulder_Init(game: &mut Game) {
    sm_game::actors::boulder::initialize(game);
}
pub fn Boulder_Main(game: &mut Game) {
    sm_game::actors::boulder::main_ai(game);
}
crate::game::callback_dispatch! {
    pub fn CallNuclearWaffleFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    [10917397] NuclearWaffle_Func_1 [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_NuclearWaffle[E].var_B = slot!(game, Enemy_NuclearWaffle[E].var_B).wrapping_sub(1));
        if (slot!(game, Enemy_NuclearWaffle[E].var_B) & 0x8000) != 0 {
            slot!(game, Enemy_NuclearWaffle[E].var_B = slot!(game, Enemy_NuclearWaffle[E].var_F));
            slot!(game, Enemy_NuclearWaffle[E].var_21 = slot!(game, Enemy_NuclearWaffle[E].var_23));
            slot!(game, Enemy_NuclearWaffle[E].var_A = (0xa69682i32) as u16);
            slot!(game, Enemy_NuclearWaffle[E]; var_33 = 0, var_08 = 0, var_09 = 0, var_0A = 0, var_0B = 0, var_0C = 0, var_0D = 0, var_0E = 0, var_0F = 0, var_18 = 0, var_19 = 0, var_1A = 0, var_1B = 0, var_1C = 0, var_1D = 0, var_1E = 0, var_1F = 0);
            slot!(game, Enemy_NuclearWaffle[E].base.properties = (slot!(game, Enemy_NuclearWaffle[E].base.properties)) | ((kEnemyProps_ProcessedOffscreen) as u16));
        }
    };
    [10917506] NuclearWaffle_Func_2 [] {
        crate::game::enemy_projectiles::advance_waffle_orbit(game);
    };
}

pub fn CeresRidley_Func_1(game: &mut Game) {
    {
        if word!(game.random_number) >= 0xff00 {
            word!(game.tilemap_stuff[15] = ((word!(game.random_number) as i32 & 0xf) + 8) as u16);
        }
    }
}
pub fn CeresRidley_Func_8(a: u16, game: &mut Game) -> u8 {
    {
        let v1: u16 = ceres_ridley_tables::background_fade(game, a as isize);
        if v1 == 0 {
            return 1;
        }
        copy_rom_colors_to_palette(0xe2, 0xa6, v1, 0xe, game);
        0
    }
}
pub fn CeresRidley_Func_12(game: &mut Game) -> u8 {
    {
        let rect: Rect16U = Rect16U {
            x: 192,
            y: 100,
            w: 8,
            h: 8,
        };
        Ridley_Func_106(0, 0, rect.x, rect.y, game);
        separated_from_rect(0, rect, game)
    }
}
pub fn CeresRidley_Func_25(game: &mut Game) {
    {
        ({
            let (bank, population) = (0xa6, addr_stru_A6AA2F as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        });
        ({
            let (bank, population) = (0xa6, addr_stru_A6AA3F as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        });
    }
}
pub fn CeresRidley_Func_27(game: &mut Game) {
    game.ram().write_byte(0x56, 7);
    word!(game.irq_enable_mode7 = 1);
    game.ram().write_byte(0x5f, 0x80);
    word! { game, 256; reg_M7A, reg_M7B, reg_M7C, reg_M7D }
    word! { game, 64; reg_M7X, reg_M7Y }
    let E: i32 = 0;
    slot!(game, Enemy_CeresRidley[E]; var_32 = 0, var_33 = 0);
    slot!(game, Enemy_CeresRidley[E].var_0A = 1);
    slot!(game, Enemy_CeresRidley[E].var_34 = -128i32 as u16);
    word!(game.reg_BG1HOFS = -128i32 as u16);
    slot!(game, Enemy_CeresRidley[E].var_35 = 32);
    word!(game.reg_BG1VOFS = 32);
    slot!(game, Enemy_CeresRidley[E].var_30 = 2048);
    slot!(game, Enemy_CeresRidley[E].var_31 = 1024);
}
pub fn HandleCeresRidleyGetawayCutscene(game: &mut Game) {
    {
        if slot!(game, Enemy_CeresRidley[0i32].var_32) == 0 {
            CeresRidley_Func_28(game);
        }
    }
}
fn CeresMult(a: u16, b: u16) -> u16 {
    let r: u16 = (abs16(a) as i32 * b as i32 >> 8) as u16;
    (if a & 0x8000 != 0 {
        -(r as i32)
    } else {
        r as i32
    }) as u16
}
pub fn CeresRidley_AC30(a: u16, r20: i16) -> i16 {
    (kSinCosTable8bit_Sext[((a as i32 & 0xff) + 64) as usize] as i32 * r20 as i32 >> 8) as i16
}
pub fn CeresRidley_Func_33(game: &mut Game) {
    if (word!(game.nmi_frame_counter_word) & 3) == 0 {
        let E: i32 = 0;
        let v1: u16 = (slot!(game, Enemy_CeresRidley[E].var_36) as i32 + 1 & 3) as u16;
        slot!(game, Enemy_CeresRidley[E].var_36 = v1);
        enqueue_rom(
            0xa6,
            ceres_ridley_tables::baby_metroid_frame(game, v1 as isize),
            game,
        );
    }
}
pub fn CeresRidley_Func_34(game: &mut Game) {
    if (word!(game.nmi_frame_counter_word) & 7) == 0 {
        let E: i32 = 0;
        let v1: u16 = (slot!(game, Enemy_CeresRidley[E].var_37) as i32 + 1 & 1) as u16;
        slot!(game, Enemy_CeresRidley[E].var_37 = v1);
        enqueue_rom(
            0xa6,
            ceres_ridley_tables::wing_frame(game, v1 as isize),
            game,
        );
    }
}
pub fn CeresRidley_Func_35(mut a: u16, game: &mut Game) {
    {
        copy_rom_colors_to_palette(
            0xa2,
            0xa6,
            (32 * a.to_ne_bytes()[1] as i32 - 0x4ef9i32) as u16,
            0xf,
            game,
        );
    }
}

crate::game::callback_dispatch! {
    pub fn CallRidleyFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    [10920795] CeresRidley_Func_3 [] {
        if word!(game.door_transition_flag_enemies) == 0 {
            let E: i32 = 0;
            slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a377i32) as u16);
            let mut v1: i16 = 512;
            if word!(game.area_index) == 2 {
                v1 = 170;
            }
            slot!(game, Enemy_CeresRidley[E].var_F = v1 as u16);
            CeresRidley_A377(game);
        }
    };
    [10921977] CeresRidley_A7F9 [] {
        let r22: u16 = (if word!(game.random_number) & 0x8000 != 0 {
            -(word!(game.random_number) as i32 & 7i32)
        } else {
            word!(game.random_number) as i32 & 7
        }) as u16;
        let E: i32 = 0;
        let r18: u16 = (slot!(game, Enemy_CeresRidley[E].var_17)).wrapping_add(r22);
        let r20: u16 = (slot!(game, Enemy_CeresRidley[E].var_18)).wrapping_add(r22);
        Ridley_Func_106(0, 0, r18, r20, game);
        let v4: i16 = ((slot!(game, Enemy_CeresRidley[E].var_00)) as i16).wrapping_sub(1);
        slot!(game, Enemy_CeresRidley[E].var_00 = v4 as u16);
        if v4 < 0 {
            slot!(game, Enemy_CeresRidley[E].var_00 = 0);
            slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a6e8i32) as u16);
        }
    };
    [10922044] CeresRidley_Func_14 [] {
        Ridley_Func_99(addr_kRidley_Ilist_E548 as u16, game);
        let E: i32 = 0;
        slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a84ei32) as u16);
        slot!(game, Enemy_CeresRidley[E].var_F = 64);
        CeresRidley_Func_15(game);
    };
    [10922148] CeresRidley_Func_17 [] {
        Ridley_Func_106(0, 1, 192, 80, game);
        let E: i32 = 0;
        if slot!(game, Enemy_CeresRidley[E].base.y_pos) as i32 - 96i32 & 0x8000 != 0 {
            slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a8d4i32) as u16);
            slot!(game, Enemy_CeresRidley[E].var_F = 10);
            slot!(game, Enemy_CeresRidley[E].var_0A = 0);
        }
    };
    [10922196] CeresRidley_Func_18 [] {
        Ridley_Func_111(-32i32 as u16, -1024i32 as u16, 768, game);
        let E: i32 = 0;
        slot!(game, Enemy_CeresRidley[E].var_F = slot!(game, Enemy_CeresRidley[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_CeresRidley[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a8f8i32) as u16);
            slot!(game, Enemy_CeresRidley[E].var_F = 36);
        }
    };
    [10922232] CeresRidley_Func_19 [] {
        Ridley_Func_111(-512i32 as u16, -16384i32 as u16, 768, game);
        let E: i32 = 0;
        slot!(game, Enemy_CeresRidley[E].var_F = slot!(game, Enemy_CeresRidley[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_CeresRidley[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a923i32) as u16);
            slot!(game, Enemy_CeresRidley[E].var_F = 28);
            word!(game.tilemap_stuff[2] = 1);
        }
    };
    [10922275] CeresRidley_Func_20 [] {
        Ridley_Func_111(-512i32 as u16, -30720i32 as u16, 768, game);
        let E: i32 = 0;
        slot!(game, Enemy_CeresRidley[E].var_F = slot!(game, Enemy_CeresRidley[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_CeresRidley[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a947i32) as u16);
            slot!(game, Enemy_CeresRidley[E].var_F = 1);
        }
    };
    [10922311] CeresRidley_Func_21 [] {
        Ridley_Func_111(-768i32 as u16, -30720i32 as u16, 768, game);
        let E: i32 = 0;
        slot!(game, Enemy_CeresRidley[E].var_F = slot!(game, Enemy_CeresRidley[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_CeresRidley[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a6e8i32) as u16);
            slot!(game, Enemy_CeresRidley[E].var_00 = 0);
            word!(game.tilemap_stuff[1] = 1);
        }
    };
    [10922513] CeresRidley_Func_24 [] {
        slot!(game, Enemy_CeresRidley[0i32].var_A = (0xa6aa4fi32) as u16);
        word!(game.ceres_status = 1);
        CeresRidley_Func_27(game);
    };
    [10922576] CeresRidley_Func_26 [] {
        Ridley_Func_61(game);
    };










    [10928206] Ridley_Func_54 [] {
        let ridley_var_F: u16 = slot!(game, Enemy_Ridley[0i32].var_F);
        if ridley_var_F != 0 {
            match ridley_var_F as i32 {
                2 => Ridley_C08E(game),
                4 => Ridley_C09F(game),
                6 => Ridley_Func_56(game),
                8 => Ridley_Func_57(game),
                10 => Ridley_C104(game),
                12 => Ridley_Func_58(game),
                _ => {
                    Unreachable(game);
                    loop {}
                }
            }
        } else {
            Ridley_Func_55(game);
        };
    };





    @functions [pub fn (game: &mut Game)];
    @prefix { let E: i32 = 0; };
    CeresRidley_Init [] {
        if (gameplay_flag_runtime::boss_byte_at(word!(game.area_index), game) & 1) != 0 {
            slot!(game, Enemy_CeresRidley[E].base.properties = (slot!(game, Enemy_CeresRidley[E].base.properties) as i32
                | (kEnemyProps_Tangible as i32
                    | kEnemyProps_Deleted as i32
                    | kEnemyProps_Invisible as i32)) as u16);
        } else {
            word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(0));
            minimap::disable_for_boss(&mut crate::game::session_runtime::Runtime::new(game));
            slot!(game, Enemy_CeresRidley[E]; parameter_1 = 0, parameter_2 = 0);
            Ridley_Func_99(addr_kRidley_Ilist_E538 as u16, game);
            slot!(game, Enemy_CeresRidley[E].base.palette_index = 3584);
            slot!(game, Enemy_CeresRidley[E].var_0C = 3584);
            slot!(game, Enemy_CeresRidley[E].base.extra_properties = (slot!(game, Enemy_CeresRidley[E].base.extra_properties)) | (4u16));
            slot!(game, Enemy_CeresRidley[E]; var_01 = 0, var_0D = 0);
            word! { game, 0; earthquake_type, earthquake_timer }
            Ridley_Func_92(game);
            Ridley_Func_117(game);
            if word!(game.area_index) == 2 {
                slot!(game, Enemy_CeresRidley[E].base.properties = (slot!(game, Enemy_CeresRidley[E].base.properties) as i32
                    | (kEnemyProps_BlockPlasmaBeam as i32 | kEnemyProps_Tangible as i32))
                    as u16);
                slot!(game, Enemy_CeresRidley[E].base.x_pos = 96);
                slot!(game, Enemy_CeresRidley[E].base.y_pos = 394);
                slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a35bi32) as u16);
                slot!(game, Enemy_CeresRidley[E]; var_B = 0, var_C = 0);
                slot!(game, Enemy_CeresRidley[E].base.layer = 5);
                slot!(game, Enemy_CeresRidley[E]; var_01 = 0, var_02 = 1, var_20 = 64, var_21 = 416,
                    var_22 = 64, var_23 = 224, var_10 = 2, var_1C = 120, );
                copy_rom_colors_to_target_palette(0xa6, 0x140, addr_word_A6E1CF as u16, 0x20, game);
                word_transfers!(game, ascending, 15;
                    le(target_palettes, 113) <- value(0),
                    le(target_palettes, 241) <- value(0),
                );
            } else {
                slot!(game, Enemy_CeresRidley[E].base.properties = (slot!(game, Enemy_CeresRidley[E].base.properties) as i32
                    | (kEnemyProps_BlockPlasmaBeam as i32 | kEnemyProps_Tangible as i32))
                    as u16);
                slot!(game, Enemy_CeresRidley[E].base.x_pos = 186);
                slot!(game, Enemy_CeresRidley[E].base.y_pos = 169);
                word!(game.ceres_status = 0);
                slot!(game, Enemy_CeresRidley[E]; var_10 = 0, var_02 = 0);
                word!(game.tilemap_stuff[1] = 1);
                slot!(game, Enemy_CeresRidley[E].var_1C = 15);
                slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a35bi32) as u16);
                slot!(game, Enemy_CeresRidley[E]; var_B = 0, var_C = 0);
                slot!(game, Enemy_CeresRidley[E].var_20 = -32i32 as u16);
                slot!(game, Enemy_CeresRidley[E].var_21 = 176);
                slot!(game, Enemy_CeresRidley[E].var_22 = 40);
                slot!(game, Enemy_CeresRidley[E].var_23 = 224);
                copy_rom_colors_to_target_palette(0xa6, 0x140, addr_word_A6E16F as u16, 0x20, game);
                word_transfers!(game, ascending, 15; le(target_palettes, 241) <- value(0));
                slot!(game, Enemy_CeresRidley[E].var_03 = addr_kBabyMetroid_Ilist_BF31 as u16);
                slot!(game, Enemy_CeresRidley[E].var_04 = 1);
                slot!(game, Ram8800_Default[(0x8800 + (0i32)) - 0xf78].var_40 = (0xa6be9ci32) as u16);
                slot!(game, Enemy_CeresRidley[E].var_07 = 5);
                QueueMusic_Delayed8(0, game);
            }
        };
    };
    CeresRidley_Main [] {
        slot!(game, Enemy_CeresRidley[E].base.health = 0x7fff);
        CallRidleyFunc(((slot!(game, Enemy_CeresRidley[E].var_A)) as u32) | (0xa60000u32), game);
        if word!(game.ceres_status) == 0 {
            if slot!(game, Enemy_CeresRidley[0i32].var_02) != 0 {
                Ridley_Func_102(game);
                Ridley_Func_112(game);
                Ridley_Func_115(game);
                CeresRidley_Func_1(game);
                Ridley_Func_70(game);
                Ridley_Func_101(game);
                Ridley_Func_118(game);
                sub_A6DB2A(game);
                Ridley_Func_120(game);
            }
            Ridley_A2DC(game);
        }
    };
    [10920788] CeresRidley_Func_2 [] {
        slot!(game, Enemy_CeresRidley[E]; var_B = 0, var_C = 0);
    };
    [10920841] CeresRidley_Func_4 [] {
        if (slot!(game, Enemy_CeresRidley[E].var_E) & 0x8000) == 0 {
            let cry_var_E: u16 = slot!(game, Enemy_CeresRidley[E].var_E);
            slot!(game, Enemy_CeresRidley[E].var_F = slot!(game, Enemy_CeresRidley[E].var_F).wrapping_add(1));
            if slot!(game, Enemy_CeresRidley[E].var_F) != 0 {
                slot!(game, Enemy_CeresRidley[E].var_F = 0);
                let mut v2: i16 = game
                    .capture_native_rom()
                    .read(crate::rtl::lorom_offset(0xa6e269) as usize + usize::from(cry_var_E))
                    as i16;
                if v2 == 255 {
                    slot!(game, Enemy_CeresRidley[E].var_E = 0);
                    slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a3dfi32) as u16);
                    slot!(game, Enemy_CeresRidley[E].var_02 = 1);
                } else {
                    slot!(game, Enemy_CeresRidley[E].var_E = (cry_var_E).wrapping_add(1));
                    let v3: i32 = ((6u16).wrapping_mul((v2) as u16)) as i32 >> 1;
                    word! { game;
                        palette_buffer[252] = ceres_ridley_tables::eye_fade(game, v3 as isize);
                        palette_buffer[253] = ceres_ridley_tables::eye_fade(game, (v3 + 1) as isize);
                        palette_buffer[254] = ceres_ridley_tables::eye_fade(game, (v3 + 2) as isize);
                    }
                }
            }
        }
    };
    [10920927] CeresRidley_Func_5 [] {
        slot!(game, Enemy_CeresRidley[E].var_F = slot!(game, Enemy_CeresRidley[E].var_F).wrapping_add(1));
        if slot!(game, Enemy_CeresRidley[E].var_F) >= 2 {
            slot!(game, Enemy_CeresRidley[E].var_F = 0);
            let mut cry_var_E: u16 = slot!(game, Enemy_CeresRidley[E].var_E);
            let mut r18: u16 = 290;
            let mut r20: u16 = 482;
            let mut v2: i16 = 11;
            do_until! { v5 == 1 ;
    let mut v5: i16 = v2;
    let v3: u16 = ceres_ridley_tables::body_fade(game, (cry_var_E >> 1) as isize);
    word! {game; palette_buffer[(r18 >> 1)] = v3; palette_buffer[(r20 >> 1)] = v3; }
    cry_var_E = (cry_var_E).wrapping_add(2);
    r18 = (r18).wrapping_add(2);
    r20 = (r20).wrapping_add(2);
    v2 = (v5).wrapping_sub(1);
}
            if cry_var_E >= 0x160 {
                if word!(game.area_index) == 2 {
                    slot!(game, Enemy_CeresRidley[E].base.layer = 2);
                }
                slot!(game, Enemy_CeresRidley[E].base.properties = (slot!(game, Enemy_CeresRidley[E].base.properties) as i32 & !(kEnemyProps_Tangible as i32)) as u16);
                slot!(game, Enemy_CeresRidley[E].var_E = 0);
                slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a455i32) as u16);
                slot!(game, Enemy_CeresRidley[E].var_F = 4);
                QueueMusic_Delayed8(5, game);
            } else {
                slot!(game, Enemy_CeresRidley[E].var_E = cry_var_E);
            }
        }
    };
    [10921080] CeresRidley_Func_7 [] {
        if word!(game.area_index) == 2 {
            slot!(game, Enemy_CeresRidley[E].var_F = slot!(game, Enemy_CeresRidley[E].var_F).wrapping_sub(1));
            let v1: bool = (slot!(game, Enemy_CeresRidley[E].var_F) & 0x8000) != 0;
            if !v1 {
                return;
            }
            slot!(game, Enemy_CeresRidley[E].var_F = 2);
            let v2: u16 = (slot!(game, Enemy_CeresRidley[E].var_E)).wrapping_add(1);
            slot!(game, Enemy_CeresRidley[E].var_E = v2);
            if CeresRidley_Func_8((v2).wrapping_sub(1), game) & 1 == 0 {
                return;
            }
            {
                let mut runtime = Runtime::new(game);
                let room = &mut runtime.session_mut().room;
                room.liquid_target = 440;
                room.liquid_velocity = -96i32 as u16;
                room.liquid_delay = 32;
            }
        } else {
            slot!(game, Enemy_CeresRidley[E].var_F = slot!(game, Enemy_CeresRidley[E].var_F).wrapping_sub(1));
            let v1_0: bool = (slot!(game, Enemy_CeresRidley[E].var_F) & 0x8000) != 0;
            if !v1_0 {
                return;
            }
        }
        slot!(game, Enemy_CeresRidley[E].var_E = 0);
        Ridley_Func_99(addr_kRidley_Ilist_E91D as u16, game);
        slot!(game, Enemy_CeresRidley[E]; var_08 = 8, var_09 = 8);
        Ridley_Func_95(game);
        word!(game.tilemap_stuff[0] = 1);
        slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a354i32) as u16);
    };
    [10921647] CeresRidley_Func_9 [] {
        slot!(game, Enemy_CeresRidley[E].var_C = (slot!(game, Enemy_CeresRidley[E].var_C)).wrapping_sub(16));
        if slot!(game, Enemy_CeresRidley[E].base.y_pos) as i32 - 112i32 & 0x8000 != 0 {
            slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a6c8i32) as u16);
            CeresRidley_Func_10(game);
        }
    };
    [10921672] CeresRidley_Func_10 [] {
        slot!(game, Enemy_CeresRidley[E].var_C = (slot!(game, Enemy_CeresRidley[E].var_C)).wrapping_add(20));
        if slot!(game, Enemy_CeresRidley[E].base.y_pos) as i32 - 80i32 & 0x8000 != 0 {
            slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a6e8i32) as u16);
            slot!(game, Enemy_CeresRidley[E].var_01 = 1);
        }
    };
    [10921704] CeresRidley_Func_11 [] {
        if slot!(game, Enemy_CeresRidley[E].var_0D) >= 0x64 {
            slot!(game, Enemy_CeresRidley[E].var_01 = 0);
            slot!(game, Enemy_CeresRidley[E].var_A = (0xa6bd9ai32) as u16);
            Ridley_Func_44(game);
        } else if word!(game.samus_health) as i32 - 30i32 & 0x8000 != 0 {
            slot!(game, Enemy_CeresRidley[E].var_01 = 0);
            slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a971i32) as u16);
            CeresRidley_Func_22(game);
        } else if CeresRidley_Func_12(game) & 1 == 0 || {
            let mut v1: u16 = (slot!(game, Enemy_CeresRidley[E].var_00)).wrapping_add(1);
            slot!(game, Enemy_CeresRidley[E].var_00 = v1);
            v1 >= 0x7c
        } {
            slot!(game, Enemy_CeresRidley[E].var_A = ceres_ridley_tables::attack(
                game,
                (word!(game.random_number) as i32 & 0xf) as isize
            ));
            slot!(game, Enemy_CeresRidley[E].var_00 = 0);
        }
    };
    [10921858] CeresRidley_Func_13 [] {
        let mut v1: u16 = abs16(slot!(game, Enemy_CeresRidley[E].var_C));
        if v1 < 0x80 {
            v1 = 128;
        }
        slot!(game, Enemy_CeresRidley[E].var_C = (if slot!(game, Enemy_CeresRidley[E].var_C) & 0x8000 != 0 {
            -(v1 as i32)
        } else {
            v1 as i32
        }) as u16);
        Ridley_Func_106(0, 0, slot!(game, Enemy_CeresRidley[E].base.x_pos), 88, game);
        if slot!(game, Enemy_CeresRidley[E].base.y_pos) as i32 - 80i32 & 0x8000 != 0 {
            let v3: u16 = (slot!(game, Enemy_CeresRidley[E].var_00)).wrapping_add(1);
            slot!(game, Enemy_CeresRidley[E].var_00 = v3);
            if v3 >= 0x30 {
                slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a88di32) as u16);
            }
        } else if slot!(game, Enemy_CeresRidley[E].base.y_pos) as i32 - 128i32 & 0x8000 != 0 {
            slot!(game, Enemy_CeresRidley[E].var_17 = slot!(game, Enemy_CeresRidley[E].base.x_pos));
            slot!(game, Enemy_CeresRidley[E].var_18 = slot!(game, Enemy_CeresRidley[E].base.y_pos));
            Ridley_Func_99(addr_kRidley_Ilist_E73A as u16, game);
            slot!(game, Enemy_CeresRidley[E].var_A = addr_loc_A6A7F9 as u16);
            slot!(game, Enemy_CeresRidley[E].var_00 = 224);
            CeresRidley_A7F9(game);
        }
    };
    [10922062] CeresRidley_Func_15 [] {
        let r18: u16 = word!(game.samus_x_pos);
        let mut v0: u16 = (word!(game.samus_y_pos)).wrapping_sub(68);
        if word!(game.samus_y_pos) as i32 - 132i32 & 0x8000 != 0 {
            v0 = 64;
        }
        let r20: u16 = v0;
        Ridley_Func_106(0, 0xd, r18, r20, game);
        let rect: Rect16U = Rect16U {
            x: r18,
            y: r20,
            w: 2,
            h: 2,
        };
        if separated_from_rect(0, rect, game) == 0 || {
            let mut v2: bool = ((slot!(game, Enemy_CeresRidley[E].var_F)) as i16).wrapping_sub(1) < 0;
            slot!(game, Enemy_CeresRidley[E].var_F = slot!(game, Enemy_CeresRidley[E].var_F).wrapping_sub(1));
            v2 as i32 != 0
        } {
            slot!(game, Enemy_CeresRidley[E].var_00 = 0);
            slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a6e8i32) as u16);
        }
    };
    [10922125] CeresRidley_Func_16 [] {
        slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a8a4i32) as u16);
        slot!(game, Enemy_CeresRidley[E].var_F = 10);
        slot!(game, Enemy_CeresRidley[E].var_0A = 0);
        word!(game.tilemap_stuff[1] = 0);
        CeresRidley_Func_17(game);
    };
    [10922353] CeresRidley_Func_22 [] {
        slot!(game, Enemy_CeresRidley[E].var_20 = -192i32 as u16);
        Ridley_Func_106(0, 1, 192, -128i32 as u16, game);
        if slot!(game, Enemy_CeresRidley[E].base.y_pos) as i32 + 128 & 0x8000 != 0 {
            CeresRidley_A994(game);
        }
    };
    CeresRidley_A994 [] {
        slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a9a0i32) as u16);
        slot!(game, Enemy_CeresRidley[E].var_F = 64);
        CeresRidley_Func_23(game);
    };
    CeresRidley_Func_28 [] {
        let cry_var_33 = slot!(game, Enemy_CeresRidley[E].var_33);
        slot!(game, Enemy_CeresRidley[E].var_33 = (cry_var_33).wrapping_add(2));
        if cry_var_33 == 0 {
            QueueSfx2_Max6(0x4e, game);
        }
        if cry_var_33 == 208 {
            crate::game::player_movement::forced::begin_ceres_push(
                crate::game::session_runtime::Runtime::new(game).session_mut(),
            );
            word! {game; earthquake_type = 35; earthquake_timer = 64; }
        }
        let v2: i32 = cry_var_33 as i32 >> 1;
        if ceres_ridley_tables::getaway_zoom(game, v2 as isize) == 0xffff {
            slot!(game, Enemy_CeresRidley[E].var_32 = slot!(game, Enemy_CeresRidley[E].var_32).wrapping_add(1));
            game.ram().write_byte(0x56, 9);
            game.ram().write_byte(0x5f, 0);
            word! { game, 0;
                reg_M7A, reg_M7B, reg_M7C, reg_M7D, reg_M7X, reg_M7Y, reg_BG1HOFS, reg_BG1VOFS,
            }
            slot!(game, Enemy_CeresRidley[E].var_A = (0xa6c04ei32) as u16);
            slot!(game, Enemy_CeresRidley[E].var_F = 0);
        } else {
            slot!(game, Enemy_CeresRidley[E].var_30 = ceres_ridley_tables::getaway_zoom(game, v2 as isize));
            let velocity = ceres_ridley_tables::getaway_velocity(game, v2 as isize);
            let v3: u16 = velocity.y.wrapping_add(slot!(game, Enemy_CeresRidley[E].var_35));
            slot!(game, Enemy_CeresRidley[E].var_35 = v3);
            word!(game.reg_BG1VOFS = v3);
            let v4: u16 =
                (slot!(game, Enemy_CeresRidley[E].var_34)).wrapping_sub(velocity.x);
            slot!(game, Enemy_CeresRidley[E].var_34 = v4);
            word!(game.reg_BG1HOFS = v4);
            CeresRidley_Func_35(slot!(game, Enemy_CeresRidley[E].var_30), game);
            slot!(game, Enemy_CeresRidley[E].var_0A = (slot!(game, Enemy_CeresRidley[E].var_0A)).wrapping_add(48));
            CeresRidley_Func_29(game);
            CeresRidley_Func_33(game);
            CeresRidley_Func_34(game);
        };
    };
    CeresRidley_Func_29 [] {
        let r18: u16 = slot!(byte game, Enemy_CeresRidley[E].var_0A[1]) as u16;
        word! { game;
            reg_M7A = CeresMult(
                CeresRidley_AC30((r18).wrapping_add(64), 0x100) as u16,
                slot!(game, Enemy_CeresRidley[E].var_30),
            );
            reg_M7B = CeresMult(CeresRidley_AC30(r18, 0x100) as u16, slot!(game, Enemy_CeresRidley[E].var_30));
            reg_M7C = CeresMult(-(CeresRidley_AC30(r18, 0x100) as i32) as u16, slot!(game, Enemy_CeresRidley[E].var_30));
            reg_M7D = CeresMult(
                CeresRidley_AC30((r18).wrapping_add(64), 0x100) as u16,
                slot!(game, Enemy_CeresRidley[E].var_30),
            );
        }
    };
    @functions [pub fn (game: &mut Game)];
    @prefix { let E: i32 = 0; };































    Ridley_Func_57 [] {
        let fresh15 = slot!(game, Enemy_Ridley[E].var_E);
        slot!(game, Enemy_Ridley[E].var_E = slot!(game, Enemy_Ridley[E].var_E).wrapping_sub(1));
        if fresh15 == 1 {
            slot!(game, Enemy_Ridley[E].var_F = slot!(game, Enemy_Ridley[E].var_F).wrapping_add(1));
            slot!(game, Enemy_Ridley[E].var_F = slot!(game, Enemy_Ridley[E].var_F).wrapping_add(1));
            queue_escape_tile_table(game);
        }
        Ridley_C104(game);
    };












    @functions [pub fn (game: &mut Game)];
    [10924787] Ridley_Func_3 [] { sm_game::actors::ridley::liftoff(game) };
    [10924833] Ridley_Func_3b [] { sm_game::actors::ridley::decide_action(game) };
    [10925048] Ridley_Func_6 [] { sm_game::actors::ridley::lunge_missed_move_to_position(game) };
    [10925460] Ridley_Func_13 [] { sm_game::actors::ridley::swoop_decelerate(game) };
    [10925508] Ridley_Func_14 [] { sm_game::actors::ridley::hover(game) };
    [10925541] Ridley_Func_15 [] { sm_game::actors::ridley::hover_to_wall_behind(game) };
    [10925587] Ridley_Func_16 [] { sm_game::actors::ridley::hover_to_wall_in_front(game) };
    [10925838] Ridley_Func_21 [] { sm_game::actors::ridley::pogo_descending(game) };
    [10927438] Ridley_Func_43 [] { sm_game::actors::ridley::dodge_power_bomb(game) };
    [10927514] Ridley_Func_44 [] { sm_game::actors::ridley::fake_retreat_move_to_position(game) };
    [10929464] Ridley_Func_63 [] { sm_game::actors::ridley::dying_grabbed_samus(game) };
    [10929470] Ridley_C53E [] { sm_game::actors::ridley::death_roar(game) };
    [10929489] Ridley_C551 [] { sm_game::actors::ridley::death_move_to_death_spot(game) };
    [10929544] Ridley_Func_64 [] { sm_game::actors::ridley::lower_acid_and_release_samus(game) };
    [10929576] Ridley_Func_65 [] { sm_game::actors::ridley::start_falling_apart(game) };
    Ridley_Main [] { sm_game::actors::ridley::main(game) };
    Ridley_Func_2 [] { sm_game::actors::ridley::time_frozen(game) };
    Ridley_Hurt [] { sm_game::actors::ridley::hurt(game) };
    [10925036] Ridley_Func_5 [] { sm_game::actors::ridley::lunge_missed(game) };
    [10925121] Ridley_Func_8 [] { sm_game::actors::ridley::swoop(game) };
    [10925141] Ridley_B455 [] { sm_game::actors::ridley::swoop_move_to_position(game) };
    [10925203] Ridley_Func_9 [] { sm_game::actors::ridley::swoop_aiming_down(game) };
    [10925265] Ridley_Func_10 [] { sm_game::actors::ridley::swoop_aiming_horizontal(game) };
    [10925334] Ridley_Func_11 [] { sm_game::actors::ridley::swoop_aiming_up(game) };
    [10925396] Ridley_Func_12 [] { sm_game::actors::ridley::swoop_aiming_up_faster(game) };
    [10925735] Ridley_Func_19 [] { sm_game::actors::ridley::pogo_fly_to_position(game) };
    [10925789] Ridley_Func_20 [] { sm_game::actors::ridley::pogo_wait_at_position(game) };
    [10926009] Ridley_Func_22 [] { sm_game::actors::ridley::pogo_ascending(game) };
    Ridley_Func_27 [] { sm_game::actors::ridley::grab_samus_from_pogo(game) };
    Ridley_Func_28 [] { sm_game::actors::ridley::set_pogo_x_direction(game) };
    Ridley_Func_30 [] { sm_game::actors::ridley::move_samus_with_feet(game) };
    Ridley_Func_31 [] { sm_game::actors::ridley::set_grabbed_samus_offset(game) };
    Ridley_Func_32 [] { sm_game::actors::ridley::lunge_missed_samus(game) };
    [10926775] Ridley_Func_33 [] { sm_game::actors::ridley::lunge(game) };
    [10926991] Ridley_Func_33b [] { sm_game::actors::ridley::grabbed_samus(game) };
    [10927044] Ridley_Func_34 [] { sm_game::actors::ridley::grabbed_samus_rising(game) };
    [10927089] Ridley_Func_35 [] { sm_game::actors::ridley::grabbed_samus_rising_to_middle(game) };
    [10927150] Ridley_Func_36 [] { sm_game::actors::ridley::grabbed_samus_release(game) };
    Ridley_Func_39 [] { sm_game::actors::ridley::set_tangibility(game) };
    Ridley_Func_42 [] { sm_game::actors::ridley::lunge_if_power_bomb(game) };
    [10927548] Ridley_Func_45 [] { sm_game::actors::ridley::fake_retreat_rising(game) };
    [10927619] Ridley_Func_47 [] { sm_game::actors::ridley::retrieve_baby_metroid(game) };
    Ridley_Func_50 [] { sm_game::actors::ridley::baby_metroid_held(game) };
    Ridley_Func_51 [] { sm_game::actors::ridley::baby_metroid_held_by_feet(game) };
    Ridley_Func_52 [] { sm_game::actors::ridley::baby_metroid_start_falling(game) };
    Ridley_Func_53 [] { sm_game::actors::ridley::baby_metroid_falling(game) };
    Ridley_Func_69 [] { sm_game::actors::ridley::spawn_small_explosion(game) };
    Ridley_Func_83 [] { sm_game::actors::ridley::handle_normal_pogo_tail(game) };
    Ridley_Func_84 [] { sm_game::actors::ridley::handle_pogo_tail_stab(game) };
    Ridley_Func_86 [] { sm_game::actors::ridley::calculate_tail_segment_positions(game) };
    Ridley_Func_91 [] { sm_game::actors::ridley::tail_whip_sound(game) };
    Ridley_Func_101 [] { sm_game::actors::ridley::ceres_health_palette(game) };
    Ridley_Func_114 [] { sm_game::actors::ridley::face_room_middle(game) };
    Ridley_Func_116 [] { sm_game::actors::ridley::wings_animation_timer(game) };
    Ridley_Func_117 [] { sm_game::actors::ridley::initialise_ribs(game) };
    Ridley_Func_118 [] { sm_game::actors::ridley::handle_ribs(game) };
    Ridley_Func_120 [] { sm_game::actors::ridley::draw_wings(game) };
    Ridley_Func_126 [] { sm_game::actors::ridley::start_death_sequence(game) };
    [10927602] Ridley_Func_46 [] { sm_game::actors::ridley::drop_baby_metroid(game) };
    [10929608] Ridley_Func_66 [] { sm_game::actors::ridley::falling_apart(game) };
    [10929626] Ridley_Func_67 [] { sm_game::actors::ridley::still_falling_apart(game) };
    @countdown [pub fn (game: &mut Game)];
    before {
        let E: i32 = 0;
        slot!(game, Enemy_CeresRidley[E].var_F = slot!(game, Enemy_CeresRidley[E].var_F).wrapping_sub(1));
    }
    when (slot!(game, Enemy_CeresRidley[E].var_F) & 0x8000) != 0;
    [10920823] CeresRidley_A377 {
        slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a389i32) as u16);
        slot!(game, Enemy_CeresRidley[E]; var_E = 0, var_F = 0);
    };
    [10921045] CeresRidley_Func_6 {
        Ridley_Func_99(addr_kRidley_Ilist_E690 as u16, game);
        slot!(game, Enemy_CeresRidley[E].var_F = 0);
        slot!(game, Enemy_CeresRidley[E].var_A = (0xa6a478i32) as u16);
        if word!(game.area_index) != 2 {
            slot!(game, Enemy_CeresRidley[E].var_F = 252);
        }
    };
    [10922400] CeresRidley_Func_23 {
        CeresRidley_Func_25(game);
        slot!(game, Enemy_CeresRidley[E]; var_B = 0, var_C = 0);
        word!(game.tilemap_stuff[0] = 0);
        slot!(game, Enemy_CeresRidley[E].var_A = (0xa6aa11i32) as u16);
        copy_rom_colors_to_palette(0xa2, 0xa6, addr_word_A6A9E3 as u16, 0xf, game);
        copy_rom_colors_to_palette(0x42, 0xa6, addr_word_A6AA01 as u16, 8, game);
        copy_rom_colors_to_palette(0x1e2, 0xa6, addr_word_A6AA01 as u16, 8, game);
    };



    @external {
        10922575 => [()];
        10929664 => [()];
    }
}
crate::game::address_dispatch! {
    pub fn CallRidleyFunc_var40(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10927772 => Ridley_Func_50; 10927795 => Ridley_Func_51; 10927818 => Ridley_Func_52;
        10927836 => Ridley_Func_53; 10927897 => [()];
    }
}
pub fn Ridley_Func_55(game: &mut Game) {
    {
        word!(game.palette_buffer[97] = word!(game.palette_buffer[1]));
        word!(game.palette_buffer[99] = word!(game.palette_buffer[3]));
        word!(game.palette_buffer[81] = word!(game.palette_buffer[17]));
        word!(game.palette_buffer[83] = word!(game.palette_buffer[19]));
        let E: i32 = 0;
        slot!(game, Enemy_Ridley[E].var_E = -15157i32 as u16);
        slot!(
            game,
            Enemy_Ridley[E].var_F = slot!(game, Enemy_Ridley[E].var_F).wrapping_add(1)
        );
        slot!(
            game,
            Enemy_Ridley[E].var_F = slot!(game, Enemy_Ridley[E].var_F).wrapping_add(1)
        );
        Ridley_C08E(game);
    }
}
pub fn Ridley_C08E(game: &mut Game) {
    {
        if queue_escape_timer_tiles(game) & 1 != 0 {
            let E: i32 = 0;
            slot!(game, Enemy_Ridley[E].var_E = -15106i32 as u16);
            slot!(
                game,
                Enemy_Ridley[E].var_F = slot!(game, Enemy_Ridley[E].var_F).wrapping_add(1)
            );
            slot!(
                game,
                Enemy_Ridley[E].var_F = slot!(game, Enemy_Ridley[E].var_F).wrapping_add(1)
            );
            Ridley_C09F(game);
        }
    }
}
pub fn Ridley_C09F(game: &mut Game) {
    {
        if queue_escape_timer_tiles(game) & 1 != 0 {
            let E: i32 = 0;
            slot!(
                game,
                Enemy_Ridley[E].var_F = slot!(game, Enemy_Ridley[E].var_F).wrapping_add(1)
            );
            slot!(
                game,
                Enemy_Ridley[E].var_F = slot!(game, Enemy_Ridley[E].var_F).wrapping_add(1)
            );
            queue_getaway_tiles(game);
            slot!(game, Enemy_Ridley[E].var_E = 128);
            QueueMusic_Delayed8(7, game);
        }
    }
}
pub fn Ridley_Func_56(game: &mut Game) {
    {
        Ridley_Func_61(game);
        let E: i32 = 0;
        let fresh16 = slot!(game, Enemy_Ridley[E].var_E);
        slot!(
            game,
            Enemy_Ridley[E].var_E = slot!(game, Enemy_Ridley[E].var_E).wrapping_sub(1)
        );
        if fresh16 == 1 {
            slot!(game, Enemy_Ridley[E].var_3B = -15280i32 as u16);
            slot!(game, Enemy_Ridley[E]; var_3C = 0, var_3D = 0, var_3E = 0, var_3F = 0);
            slot!(game, Enemy_Ridley[E].var_E = 32);
            if word!(game.japanese_text_flag) == 0 {
                slot!(
                    game,
                    Enemy_Ridley[E].var_F = slot!(game, Enemy_Ridley[E].var_F).wrapping_add(1)
                );
                slot!(
                    game,
                    Enemy_Ridley[E].var_F = slot!(game, Enemy_Ridley[E].var_F).wrapping_add(1)
                );
            }
            slot!(
                game,
                Enemy_Ridley[E].var_F = slot!(game, Enemy_Ridley[E].var_F).wrapping_add(1)
            );
            slot!(
                game,
                Enemy_Ridley[E].var_F = slot!(game, Enemy_Ridley[E].var_F).wrapping_add(1)
            );
        }
    }
}
pub fn Ridley_C104(game: &mut Game) {
    {
        Ridley_Func_61(game);
        if type_escape_text(0x3582, game) & 1 != 0 {
            let E: i32 = 0;
            slot!(
                game,
                Enemy_Ridley[E].var_F = slot!(game, Enemy_Ridley[E].var_F).wrapping_add(1)
            );
            slot!(
                game,
                Enemy_Ridley[E].var_F = slot!(game, Enemy_Ridley[E].var_F).wrapping_add(1)
            );
        }
    }
}
pub fn Ridley_Func_58(game: &mut Game) {
    {
        Ridley_Func_61(game);
        CeresRidley_Func_2(game);
        let E: i32 = 0;
        slot!(game, Enemy_Ridley[E].var_F = 0);
        slot!(game, Enemy_Ridley[E].var_A = (0xa6aa50i32) as u16);
        word! {game; timer_status = 1; ceres_status = 2; }
        SetBossBitForCurArea(1, game);
    }
}

// Tail words alias the shared tilemap workspace; keep each load/store ordered.

macro_rules! steer_enemy_speed_and_angle {
    ($game:ident, $layout:ident, $target_speed:ident, $enemy:ident, $angle_step:ident, $target_angle:ident) => {{
        if $target_speed != slot!($game, $layout[$enemy].var_0B) {
            if (($target_speed) as i16).wrapping_sub((slot!($game, $layout[$enemy].var_0B)) as i16)
                >= 0
            {
                let mut v2: u16 = (slot!($game, $layout[$enemy].var_0B)).wrapping_add(32);
                if v2 as i32 - $target_speed as i32 & 0x8000 == 0 {
                    v2 = $target_speed;
                }
                slot!($game, $layout[$enemy].var_0B = v2);
            } else {
                let mut v1: u16 = (slot!($game, $layout[$enemy].var_0B)).wrapping_sub(32);
                if v1 as i32 - $target_speed as i32 & 0x8000 != 0 {
                    v1 = $target_speed;
                }
                slot!($game, $layout[$enemy].var_0B = v1);
            }
        }
        let mut v3: u16 = 0;

        let use_target_angle: bool = if ($angle_step & 0x8000) != 0 {
            v3 = (slot!($game, $layout[$enemy].var_0A)).wrapping_add($angle_step);
            (v3 as i32 - $target_angle as i32 & 0x8000 != 0)
        } else {
            v3 = (slot!($game, $layout[$enemy].var_0A)).wrapping_add($angle_step);
            (v3 as i32 - $target_angle as i32 & 0x8000 == 0)
        };
        if use_target_angle {
            v3 = $target_angle;
        }
        slot!($game, $layout[$enemy].var_0A = v3);
        $angle_step = v3.to_ne_bytes()[1] as u16;
        slot!(
            $game,
            $layout[$enemy].var_B =
                Math_MultBySin(slot!($game, $layout[$enemy].var_0B), $angle_step)
        );
        slot!(
            $game,
            $layout[$enemy].var_C =
                Math_MultByCos(slot!($game, $layout[$enemy].var_0B), $angle_step)
        );
    }};
}
pub(crate) use steer_enemy_speed_and_angle;

crate::game::callback_dispatch! {
    pub fn CallBabyMetroidInstr(ea: u32, k: u16, game: &mut Game) -> u16 => ea;
    call (k, game); default Unreachable(game) as u16;




    @functions [pub fn (k: u16, game: &mut Game) -> u16];
    [10928073] BabyMetroid_Instr_2 [] { sm_game::actors::ridley::baby_metroid_goto_if_still(k, game) };
    [10928097] BabyMetroid_Instr_3 [] { sm_game::actors::ridley::baby_metroid_load_palette(k, game) };
    [10928114] BabyMetroid_Instr_1 [] { sm_game::actors::ridley::baby_metroid_goto_if_moving(k, game) };
    [10928120] BabyMetroid_Goto [] { sm_game::actors::ridley::baby_metroid_goto(k, game) };
}
crate::game::callback_dispatch! {
    pub fn CallCeresSteamFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub fn (k: u16, game: &mut Game)];
    [10940441] CeresSteam_Func_1 [] {
        let E: i32 = k as i32;
        let pt: Point16U = project_cinematic_point(
            Point16U {
                x: slot!(game, Enemy_CeresSteam[E].base.x_pos),
                y: slot!(game, Enemy_CeresSteam[E].base.y_pos),
            },
            game,
        );
        let v3: i32 = (0x7000 + (word!(game.cur_enemy_index) as i32)) - 0xf78;
        slot!(game, EnemySpawnData[v3].xpos2 = (pt.x).wrapping_sub(slot!(game, Enemy_CeresSteam[E].base.x_pos)));
        slot!(game, EnemySpawnData[v3].ypos2 = (pt.y).wrapping_sub(slot!(game, Enemy_CeresSteam[E].base.y_pos)));
    };
    @external {
        10940404 => [()];
    }
}
pub fn CeresDoor_Func_1(k: u16, game: &mut Game) {
    if slot!(game, Enemy_CeresDoor[k as i32].parameter_1) == 2 {
        let v1: u16 = word!(game.vram_write_queue_tail);
        let mut v2: i32 = (0xd0 + (word!(game.vram_write_queue_tail) as i32)) - 0xf78;
        slot!(game, VramWriteEntry[v2].size = 1024);
        match (
            -20480i32 as VoidP,
            slot!(@at VramWriteEntry, v2, src.addr) + 1,
        ) {
            (value, at) => {
                let bytes = u16::to_ne_bytes(value);
                game.ram().write_byte(at, bytes[0]);
                game.ram().write_byte(at + 1, bytes[1]);
            }
        };
        slot!(game, VramWriteEntry[v2].src.addr = -15360i32 as VoidP);
        slot!(game, VramWriteEntry[v2].vram_dst = 28672);
        word!(game.vram_write_queue_tail = (v1).wrapping_add(7));
    }
}
crate::game::callback_dispatch! {
    pub fn CallCeresDoor(ea: u32, game: &mut Game) => ea;
    setup { let k: u16 = word!(game.cur_enemy_index); }
    call (game); default { Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    [10942373] CeresDoor_Func_4 [] {
        ceres_doors::change_properties(word!(game.cur_enemy_index), ceres_doors::INVISIBLE, 0, EnemyScript::Stop, game);
        if word!(game.ceres_status) & 1 != 0 {
            slot!(game, Enemy_CeresDoor[word!(game.cur_enemy_index) as i32].base.palette_index = 3584);
            ceres_doors::change_properties(word!(game.cur_enemy_index), 0, ceres_doors::INVISIBLE, EnemyScript::Stop, game);
        }
    };
    [10942544] CeresDoor_Func_7 [] {
        CeresDoor_Func_8(game);
        if word!(game.palette_change_num) == 0 {
            copy_rom_colors_to_palette(
                0x52,
                0xa6,
                (2 * (word!(game.nmi_frame_counter_word) as i32 & 0x38) - 0x78fi32) as u16,
                6,
                game,
            );
        }
    };
    @forward [pub fn (game: &mut Game)];
    [10942315] CeresDoor_Func_2 => ceres_doors::escape_rumble(0x14, game);
    [10942320] CeresDoor_Func_3 => ceres_doors::escape_rumble(0x1d, game);
    @external {
        10942397 => [CeresDoor_Func_5(k, game)]; 10942428 => [CeresDoor_Func_6(k, game)];
    }
}
pub fn CeresDoor_Func_5(k: u16, game: &mut Game) {
    {
        CeresDoor_Func_7(game);
        if word!(game.ceres_status) >= 2 {
            let E: i32 = k as i32;
            slot!(game, Enemy_CeresDoor[E].var_A = (0xa6f7dci32) as u16);
            slot!(game, Enemy_CeresDoor[E].var_D = 48);
            let E0: i32 = 0;
            slot!(game, Enemy_CeresDoor[E0]; var_E = 0, var_F = 0);
        }
    }
}

macro_rules! set_explosion_spawn_offset {
    ($game:ident, $enemy:ident, $offset_table:literal, $offset_index:ident) => {
        $game.eproj_spawn_pt =
            Point16U {
                x: ((slot!($game, EnemyData[$enemy].x_pos)).wrapping_add(
                    crate::rtl::rom_table_word!($game, $offset_table, $offset_index as isize),
                )),
                y: ((slot!($game, EnemyData[$enemy].y_pos)).wrapping_add(
                    crate::rtl::rom_table_word!($game, $offset_table, ($offset_index + 1) as isize),
                )),
            };
    };
    ($game:ident, $enemy:ident, $offset:path, $offset_index:ident) => {
        $game.eproj_spawn_pt = Point16U {
            x: ((slot!($game, EnemyData[$enemy].x_pos))
                .wrapping_add($offset($game, $offset_index as isize))),
            y: ((slot!($game, EnemyData[$enemy].y_pos))
                .wrapping_add($offset($game, ($offset_index + 1) as isize))),
        };
    };
}
pub(crate) use set_explosion_spawn_offset;

pub fn CeresDoor_Func_6(k: u16, game: &mut Game) {
    let E: i32 = k as i32;
    slot!(
        game,
        Enemy_CeresDoor[E].var_D = slot!(game, Enemy_CeresDoor[E].var_D).wrapping_sub(1)
    );
    let mut v3: bool = (slot!(game, Enemy_CeresDoor[E].var_D) & 0x8000) != 0;
    if v3 {
        slot!(
            game,
            Enemy_CeresDoor[E].base.properties = (slot!(game, Enemy_CeresDoor[E].base.properties))
                | ((kEnemyProps_Invisible) as u16)
        );
        slot!(game, Enemy_CeresDoor[E].var_A = (0xa6f850i32) as u16);
        ceres_doors::finish_door_explosion(game);
    } else {
        let E0: i32 = 0;
        slot!(
            game,
            Enemy_CeresDoor[E0].var_E = slot!(game, Enemy_CeresDoor[E0].var_E).wrapping_sub(1)
        );
        v3 = (slot!(game, Enemy_CeresDoor[E0].var_E) & 0x8000) != 0;
        if v3 {
            slot!(game, Enemy_CeresDoor[E0].var_E = 4);
            slot!(
                game,
                Enemy_CeresDoor[E0].var_F = slot!(game, Enemy_CeresDoor[E0].var_F).wrapping_sub(1)
            );
            v3 = (slot!(game, Enemy_CeresDoor[E0].var_F) & 0x8000) != 0;
            if v3 {
                slot!(game, Enemy_CeresDoor[E0].var_F = 3);
            }
            let v5: i32 = ((4u16).wrapping_mul(slot!(game, Enemy_CeresDoor[E0].var_F))) as i32 >> 1;
            set_explosion_spawn_offset!(game, E, ceres_door_data::explosion_offset, v5);
            let mut v6: u16 = 3;
            if game.next_random() < 0x4000 {
                v6 = 12;
            }
            SpawnEprojWithRoomGfx(0xe509, v6, game);
            QueueSfx2_Max6(0x25, game);
        }
    };
}
pub fn CeresDoor_Func_8(game: &mut Game) {
    {
        enqueue_rom(
            0xa6,
            ceres_door_data::platform_frame(
                game,
                ((word!(game.nmi_frame_counter_word) as i32 & 2) >> 1) as isize,
            ),
            game,
        );
    }
}
crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    [10914693 | 10914713] SpikeyPlatform2ndEnemy_Init [] {
        let v0: i32 = word!(game.cur_enemy_index) as i32 >> 1;
        let v1: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, EnemyData[v1].x_pos = word!(game.enemy_drawing_queue[(v0 + 91)]));
        slot!(game, EnemyData[v1].y_pos = (word!(game.enemy_drawing_queue[(v0 + 93)])).wrapping_add(12));
    };
    [10917060] NuclearWaffle_Init [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_NuclearWaffle[E].base.current_instruction = addr_kNuclearWaffle_Ilist_9490 as u16);
        slot!(game, Enemy_NuclearWaffle[E].var_C = slot!(byte game, Enemy_NuclearWaffle[E].parameter_1[0]) as u16);
        slot!(game, Enemy_NuclearWaffle[E].var_D = slot!(byte game, Enemy_NuclearWaffle[E].parameter_1[1]) as u16);
        slot!(game, Enemy_NuclearWaffle[E].var_E = slot!(byte game, Enemy_NuclearWaffle[E].parameter_2[0]) as u16);
        let nwe_parameter_2_high: u16 = slot!(byte game, Enemy_NuclearWaffle[E].parameter_2[1]) as u16;
        slot!(game, Enemy_NuclearWaffle[E].var_F = nwe_parameter_2_high);
        slot!(game, Enemy_NuclearWaffle[E].var_B = nwe_parameter_2_high);
        slot!(game, Enemy_NuclearWaffle[E].var_A = (0xa69615i32) as u16);
        let v2: i32 = ((4u16).wrapping_mul(slot!(game, Enemy_NuclearWaffle[E].var_E))) as i32 >> 1;
        let v3: u16 = g_word_A695F6[v2 as usize];
        slot!(game, Enemy_NuclearWaffle[E]; var_23 = v3, var_21 = v3, var_2E = g_word_A695F6[(v2 + 1) as usize],
        var_2B = g_word_A695FE[v2 as usize], var_2C = g_word_A695FE[(v2 + 1) as usize],
        var_30 = g_word_A69606[v2 as usize], var_2F = g_word_A69606[(v2 + 1) as usize], );
        let mut v4: u16 = (8u16).wrapping_mul(slot!(game, Enemy_NuclearWaffle[E].var_C));
        if slot!(game, Enemy_NuclearWaffle[E].var_E) == 0 {
            v4 = (v4).wrapping_add(4);
        }
        let v5: i32 = v4 as i32 >> 1;
        slot!(game, Enemy_NuclearWaffle[E].var_25 = crate::game::enemy_speeds::enemy_speed!(game, Linear, v5 as isize));
        slot!(game, Enemy_NuclearWaffle[E].var_24 = crate::game::enemy_speeds::enemy_speed!(game, Linear, (v5 + 1) as isize));
        slot!(game, Enemy_NuclearWaffle[E].var_26 = slot!(game, Enemy_NuclearWaffle[E].base.x_pos));
        slot!(game, Enemy_NuclearWaffle[E].var_27 = slot!(game, Enemy_NuclearWaffle[E].base.y_pos));
        let v6: u16 = (slot!(game, Enemy_NuclearWaffle[E].var_26)).wrapping_add(actor_cosine(slot!(game, Enemy_NuclearWaffle[E].var_23), slot!(game, Enemy_NuclearWaffle[E].var_D), game));
        slot!(game, Enemy_NuclearWaffle[E].var_28 = v6);
        slot!(game, Enemy_NuclearWaffle[E].base.x_pos = v6);
        let v7: u16 = (slot!(game, Enemy_NuclearWaffle[E].var_27)).wrapping_add(actor_sine(slot!(game, Enemy_NuclearWaffle[E].var_23), slot!(game, Enemy_NuclearWaffle[E].var_D), game));
        slot!(game, Enemy_NuclearWaffle[E].var_29 = v7);
        slot!(game, Enemy_NuclearWaffle[E].base.y_pos = v7);
        let mut v8: u16 = 8;
        slot!(game, Enemy_NuclearWaffle[E].var_2A = 8);
        do_until! { v8 == 0 ;
            SpawnEprojWithGfx(
                v8,
                word!(game.cur_enemy_index),
                addr_kEproj_NuclearWaffleBody as u16,
                game,
            );
            v8 = (slot!(game, Enemy_NuclearWaffle[E].var_2A)).wrapping_sub(2);
            slot!(game, Enemy_NuclearWaffle[E].var_2A = v8);
        }
        slot!(game, Enemy_NuclearWaffle[E].var_2A = 6);
        do_until! { v15 == 0 ;
            let v13: u16 = (slot!(game, Enemy_NuclearWaffle[E].base.vram_tiles_index)) | (slot!(game, Enemy_NuclearWaffle[E].base.palette_index));
            slot!(game, Enemy_NuclearWaffle[E].var_34 = v13);
            let r18: u16 = create_sprite(slot!(game, Enemy_NuclearWaffle[E].base.x_pos), slot!(game, Enemy_NuclearWaffle[E].base.y_pos), 43, v13, game);
            slot!(game, Enemy_NuclearWaffle[word!(game.cur_enemy_index) as i32 + slot!(game, Enemy_NuclearWaffle[E].var_2A) as i32].var_10 = r18);
            let mut v15: u16 = (slot!(game, Enemy_NuclearWaffle[E].var_2A)).wrapping_sub(2);
            slot!(game, Enemy_NuclearWaffle[E].var_2A = v15);
        }
    };
    [10917390] NuclearWaffle_Main [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        CallNuclearWaffleFunc(((slot!(game, Enemy_NuclearWaffle[E].var_A)) as u32) | (0xa60000u32), game);
    };
    [10918488] FakeKraid_Init [] { sm_game::actors::mini_kraid::initialize(game); };
    [10918594] FakeKraid_Main [] { sm_game::actors::mini_kraid::main_ai(game); };
    [10918946] FakeKraid_Touch [] { sm_game::actors::mini_kraid::touch(game); };
    [10918969] FakeKraid_Shot [] { sm_game::actors::mini_kraid::shot(game); };
    [10920659] CeresRidley_Hurt [] {
        Ridley_Func_102(game);
        sub_A6DB2A(game);
        Ridley_Func_120(game);
        Ridley_A2DC(game);
    };




    [10940337] CeresSteam_Init [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_CeresSteam[E].base; vram_tiles_index = 0,
        properties = (slot!(game, Enemy_CeresSteam[E].base.properties)) | ((kEnemyProps_DisableSamusColl) as u16),
        extra_properties = (slot!(game, Enemy_CeresSteam[E].base.extra_properties)) | (4u16), instruction_timer = 1,
        timer = 0, palette_index = 2560, );
        slot!(game, Enemy_CeresSteam[E].var_D = ((game.next_random() as i32 & 0x1f) + 1) as u16);
        let v1: i32 = slot!(game, Enemy_CeresSteam[E].parameter_1) as i32;
        let kind = ceres_door_data::steam(game, v1 as isize);
        slot!(game, Enemy_CeresSteam[E].base.current_instruction = kind.instruction_list);
        slot!(game, Enemy_CeresSteam[E].var_A = kind.function);
    };
    [10940429] CeresSteam_Main [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_CeresSteam[E].base.health = 0x7fff);
        CallCeresSteamFunc(
            ((slot!(game, Enemy_CeresSteam[E].var_A)) as u32) | (0xa60000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    [10942149] CeresDoor_Init [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_CeresDoor[E].base.spritemap_pointer = addr_kCeresDoor_Sprmap_FAC7 as u16);
        slot!(game, Enemy_CeresDoor[E].base.instruction_timer = 1);
        slot!(game, Enemy_CeresDoor[E].base; timer = 0, vram_tiles_index = 0);
        slot!(game, Enemy_CeresDoor[E].base.palette_index = 1024);
        let v2: i32 = slot!(game, Enemy_CeresDoor[E].parameter_1) as i32;
        slot!(game, Enemy_CeresDoor[E].var_A = ceres_door_data::door_function(game, v2 as isize));
        slot!(game, Enemy_CeresDoor[E].base.current_instruction = ceres_door_data::door_instructions(game, v2 as isize));
        slot!(game, Enemy_CeresDoor[E].var_B = 0);
        CeresDoor_Func_1(word!(game.cur_enemy_index), game);
        let mut v3: u16 = if word!(game.ceres_status) != 0 {
            (addr_word_A6F50C as i32 + 2) as u16
        } else {
            if slot!(game, Enemy_CeresDoor[E].parameter_1) == 3 {
                copy_rom_colors_to_target_palette(
                    0xa6,
                    0x142,
                    (addr_kCeresDoor_Palette as i32 + 2) as u16,
                    0xf,
                    game,
                );
                return;
            }
            (addr_kCeresDoor_Palette as i32 + 2) as u16
        };
        slot!(game, Enemy_CeresDoor[word!(game.cur_enemy_index) as i32].base.palette_index = 3584);
        copy_rom_colors_to_target_palette(0xa6, 0x1e2, v3, 0xf, game);
    };
    [10942309] CeresDoor_Main [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        CallCeresDoor(((slot!(game, Enemy_CeresDoor[E].var_A)) as u32) | (0xa60000u32), game);
    };
    [10943346] Zebetites_Init [] { sm_game::actors::zebetites::initialize(game); };
    [10943539] Zebetites_Main [] { sm_game::actors::zebetites::main_ai(game); };
    [10943911] Zebetites_Touch [] { sm_game::actors::zebetites::touch(game); };
    [10943916] Zebetites_Shot [] { sm_game::actors::zebetites::shot(game); };
    @functions [pub fn (game: &mut Game)];
    [10929814] RidleysExplosion_Init [] { sm_game::actors::ridley::explosion_init(game) };
    [10930388] RidleysExplosion_Main [] { sm_game::actors::ridley::explosion_main(game) };
    [10936202] Ridley_Shot [] { sm_game::actors::ridley::shot(game) };
    [10936242] Ridley_Powerbomb [] { sm_game::actors::ridley::power_bomb(game) };
    @forward [pub fn (game: &mut Game)];
    [10911759] Enemy_GrappleReact_CancelBeam_A6 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [10911779] Enemy_NormalTouchAI_A6 => actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    [10911789] Enemy_NormalShotAI_A6 => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
    [10911809] Enemy_NormalFrozenAI_A6 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
    @external {
        10915836 => [fire_geysers::initialize(game)];
        10915875 => [fire_geysers::run(game)];
    }
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10918694] FakeKraid_Instr_2 [jp] { sm_game::actors::mini_kraid::step(game); };
    @functions [pub fn (_k: u16, _jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10918772] FakeKraid_Instr_1 [] { sm_game::actors::mini_kraid::choose_animation(game) };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10918834] FakeKraid_Instr_3 [jp] { sm_game::actors::mini_kraid::roar(game); };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10918852] FakeKraid_Instr_4 [jp] { sm_game::actors::mini_kraid::spit_left(game); };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10918914] FakeKraid_Instr_5 [jp] { sm_game::actors::mini_kraid::spit_right(game); };














    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10940711]
    CeresSteam_Instr_2 [ceres_doors::change_properties(k, 0, VANISH, { let record = unsafe { crate::rtl::rom_record(0xa6, jp.word(2), game) }; EnemyScript::at(record.image, record.base) }, game)] {
        let E: i32 = k as i32;
        let fresh0 = slot!(game, Enemy_CeresSteam[E].var_D);
        slot!(game, Enemy_CeresSteam[E].var_D = slot!(game, Enemy_CeresSteam[E].var_D).wrapping_sub(1));
        if fresh0 != 1 {
            return EnemyScript::jump(jp.word(0));
        }
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10937534] Ridley_Instr_5 [] { sm_game::actors::ridley::roar(jp, game) };
    [10937546] Ridley_Instr_6 [] { sm_game::actors::ridley::stop_roaring(jp, game) };
    [10937554] Ridley_Instr_10 [] { sm_game::actors::ridley::goto_if_samus_health_low(jp, game) };
    [10937582] Ridley_Instr_4 [] { sm_game::actors::ridley::goto_by_holding_samus(jp, game) };
    [10937592] Ridley_Instr_3 [] { sm_game::actors::ridley::goto_if_not_holding_samus(jp, game) };
    [10937601] Ridley_Instr_2 [] { sm_game::actors::ridley::set_feet_distance(jp, game) };
    [10937623] Ridley_Instr_1 [] { sm_game::actors::ridley::goto_if_not_facing_left(jp, game) };
    [10937631] Ridley_Instr_14 [] { sm_game::actors::ridley::move_by(jp, game) };
    [10938140] Ridley_Instr_9 [] { sm_game::actors::ridley::flip_left(jp, game) };
    [10938151] Ridley_Instr_7 [] { sm_game::actors::ridley::face_forward(jp, game) };
    [10938159] Ridley_Instr_8 [] { sm_game::actors::ridley::flip_right(jp, game) };
    [10938445] Ridley_Instr_11 [] { sm_game::actors::ridley::calculate_fireball_velocities(jp, game) };
    [10938729] Ridley_Instr_15 [] { sm_game::actors::ridley::start_ceres_liftoff(jp, game) };
    [10938742] Ridley_Instr_16 [] { sm_game::actors::ridley::start_norfair_liftoff(jp, game) };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10938628] Ridley_Instr_12 [] { sm_game::actors::ridley::fireball_with_afterburn(jp, game) };
    [10938633] Ridley_Instr_13 [] { sm_game::actors::ridley::fireball_without_afterburn(jp, game) };


    @external {
        10915247 | 10915347..=10915793 => [fire_geysers::instruction(ea, j, game)];
        10940701 => [ceres_doors::change_properties(k, VANISH, 0, j, game)];
        10940725 => [ceres_doors::change_properties(k, 0, VANISH, j, game)];
        10942014 => ceres_doors::branch_unless_samus_near;
        10942058 => [ceres_doors::branch_unless_boss_defeated(j, game)];
        10942072 => [ceres_doors::branch_unless_escape(j, game)];
        10942091 => [ceres_doors::change_properties(k, ceres_doors::TANGIBLE, 0, j, game)];
        10942101 => [ceres_doors::change_properties(k, 0, ceres_doors::TANGIBLE, j, game)];
        10942111 => [ceres_doors::mark_open(k, true, j, game)];
        10942118 => [ceres_doors::change_properties(k, ceres_doors::INVISIBLE, 0, j, game)];
        10942128 => [ceres_doors::mark_open(k, false, j, game)];
        10942131 => [ceres_doors::change_properties(k, 0, ceres_doors::INVISIBLE, j, game)];
        10942141 => [ceres_doors::play_open_sound(j, game)];
    }
}
