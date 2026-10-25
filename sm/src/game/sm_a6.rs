use crate::game::actors;
use crate::game::actors::access::ActorKey as Key;
use crate::game::actors::api as actor_api;
use crate::game::enemy_projectiles::{DropSource, scatter_drop as scatter};
use crate::game::enemy_script::EnemyScript;
use crate::game::gameplay_flag_host::BossWordOrder;
use crate::game::gameplay_flag_runtime;
use crate::game::music::QueueMusic_Delayed8;
use crate::game::player_update::minimap;
pub use crate::game::prelude::*;
use crate::game::projectile_presentation_ram as sprite_output;
use crate::game::session_runtime::Runtime;
use crate::game::sprite_drawing::Clip;

pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a6::*;
pub use crate::game::constants::bank_a9::*;

use crate::game::enemy_projectiles::{
    Math_MultByCos, Math_MultBySin, SpawnEprojWithGfx, SpawnEprojWithRoomGfx,
};
use crate::game::gameplay_flag_runtime::{
    CheckEventHappened, ClearEventHappened, SetBossBitForCurArea, SetEventHappened,
};
use crate::game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;
use crate::game::session_runtime::damage_player;
use crate::game::session_runtime::player_command as CallSomeSamusCode;
use crate::game::sm_8b::CalcCeresSteamPos_Mode7;
use crate::game::sm_80::{
    NextRandom, QueueMode7Transfers, QueueSfx1_Max6, QueueSfx2_Max3, QueueSfx2_Max6,
    QueueSfx3_Max3, QueueSfx3_Max6,
};
use crate::game::sm_a0::{CosineMult8bit, SineMult8bit};
use crate::game::sm_a9::{
    ComputeCosMult, ComputeSinMult, MoveEnemyWithVelocity, Shitroid_Func_2, WriteColorsToPalette,
    WriteColorsToTargetPalette,
};
use crate::game::sprite_effect_runtime::create_sprite;

pub type Enemy_Ridley = crate::game::enemy_layouts::EnemyLayout104;

crate::type_aliases! {
    pub Enemy_CeresDoor, Enemy_CeresSteam, Enemy_FireGeyser, Enemy_RidleysExplosion, Enemy_Zebetites
        = crate::game::enemy_layouts::EnemyLayout8;
}

crate::type_aliases! {
    pub Enemy_FakeKraid, Enemy_SpikeyPlatform, Enemy_Boulder =
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
const RomPtr_A0: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa0>;
const RomPtr_A6: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa6>;
crate::ram_map::ram_pointer_functions! {
    gOamEnt(v: i32) -> OamEnt = [(0x370 + v) as isize];
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    gEnemySpawnData(o: i32) -> EnemySpawnData = [0x7000, o as isize];
    gRam8800_Default(o: i32) -> Ram8800_Default = [0x8800, o as isize];
    Get_Boulder(o: i32) -> Enemy_Boulder = [0xf78, o as isize];
    Get_CeresDoor(o: i32) -> Enemy_CeresDoor = [0xf78, o as isize];
    Get_CeresRidley(o: i32) -> Enemy_CeresRidley = [0xf78, o as isize];
    Get_CeresSteam(o: i32) -> Enemy_CeresSteam = [0xf78, o as isize];
    Get_FakeKraid(o: i32) -> Enemy_FakeKraid = [0xf78, o as isize];
    Get_FireGeyser(o: i32) -> Enemy_FireGeyser = [0xf78, o as isize];
    Get_MotherBrain(o: i32) -> Enemy_MotherBrain = [0xf78, o as isize];
    Get_NuclearWaffle(o: i32) -> Enemy_NuclearWaffle = [0xf78, o as isize];
    Get_Ridley(o: i32) -> Enemy_Ridley = [0xf78, o as isize];
    Get_RidleysExplosion(o: i32) -> Enemy_RidleysExplosion = [0xf78, o as isize];
    Get_SpikeyPlatform(o: i32) -> Enemy_SpikeyPlatform = [0xf78, o as isize];
    Get_Zebetites(o: i32) -> Enemy_Zebetites = [0xf78, o as isize];
}
static g_word_A686F1: [u16; 2] = [0x1000, 0x1800];
static g_word_A68DBB: u16 = 5;
static g_word_A68DBD: u16 = 0xa;
static g_word_A68DBF: u16 = 0xf;
static g_word_A68DC1: u16 = 0x14;
static g_word_A68DC3: u16 = 0x19;
static g_word_A68DC5: u16 = 0x1e;
static g_word_A68DC7: u16 = 0x23;
static g_word_A68DC9: u16 = 0x28;
static g_word_A68DCB: u16 = 0x2d;
static g_word_A68DCD: u16 = 0x32;
static g_word_A68DCF: u16 = 0x37;
static g_word_A68DD1: u16 = 0x3c;
static g_word_A68DD3: u16 = 0x41;
static g_word_A68DD5: u16 = 0x46;
static g_word_A68DD7: u16 = 0x4b;
static g_word_A68DD9: u16 = 0x50;
static g_word_A68DDB: u16 = 0x55;
static g_word_A68DDD: u16 = 0x5a;
static g_word_A68DDF: u16 = 0x5f;
static g_word_A68DE1: u16 = 0x64;
static g_word_A68DE3: u16 = 0x69;
static g_word_A68DE5: u16 = 0x6e;
static g_word_A68DE7: u16 = 0x18;
static g_word_A68DE9: u16 = 0x18;
static g_word_A68DEB: u16 = 0x18;
static g_word_A68DED: u16 = 0x18;
static g_word_A68DEF: u16 = 0x18;
static g_word_A68DF1: u16 = 0x18;
static g_word_A68DF3: u16 = 0x18;
static g_word_A68DF5: u16 = 0x18;
static g_word_A68DF7: u16 = 0x18;
static g_word_A68DF9: u16 = 0x18;
static g_word_A68DFB: u16 = 0x18;
static g_word_A68DFD: u16 = 0x18;
static g_word_A68DFF: u16 = 0x18;
static g_word_A68E01: u16 = 0x18;
static g_word_A68E03: u16 = 0x18;
static g_word_A68E05: u16 = 0x18;
static g_word_A68E07: u16 = 0x18;
static g_word_A68E09: u16 = 0x18;
static g_word_A68E0B: u16 = 0x14;
static g_word_A68E0D: u16 = 0x10;
static g_word_A68E0F: u16 = 0xc;
static g_word_A68E11: u16 = 8;
static g_word_A695F6: [u16; 4] = [0x190, 0xf0, 0xf0, 0x190];
static g_word_A695FE: [u16; 4] = [0xffe8, 0xfff4, 0x18, 0xc];
static g_word_A69606: [u16; 4] = [0x180, 0x100, 0x100, 0x180];
static g_word_A69A48: [u16; 8] = [0xfe00, 0xfb00, 0xfc00, 0xfb00, 0x200, 0xfb00, 0x400, 0xfb00];
static g_word_A6B288: u16 = 8;
static g_word_A6BBEB: [u16; 3] = [0x40, 0, 0xd0];
static g_word_A6BC62: [u16; 3] = [0xb0, 0, 0x50];
static g_word_A6C66E: [i16; 20] = [
    -24, -24, -20, 20, 16, -30, 30, -3, 14, -13, -2, 18, -2, -32, -31, 8, -4, -10, 19, 19,
];
static g_word_A6CF54: [i16; 3] = [0x20, 0, -0x20];
static g_byte_A6D61F: [u8; 16] = [
    0x10, 0xf, 0xe, 0xd, 0xc, 0xb, 0xa, 9, 8, 7, 6, 5, 4, 3, 2, 1,
];
static g_byte_A6D712: [u8; 16] = [
    0x10, 0xf, 0xe, 0xd, 0xc, 0xb, 0xa, 9, 8, 7, 6, 5, 4, 3, 2, 1,
];
static g_word_A6D9ED: [u16; 8] = [0xc, 0xe, 0x10, 0x12, 0x1c, 0x20, 0x28, 0x30];
crate::game::callback_dispatch! {
    pub unsafe fn CallBoulderFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10913690] Boulder_Func_1 [] {
        let mut SamusEnemyDelta_Y: i16 =
            ({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y) }) as i16;
        if SamusEnemyDelta_Y >= 0 {
            let E = Get_Boulder(word!(game.cur_enemy_index) as i32, game);
            if (SamusEnemyDelta_Y).wrapping_sub(((*E).var_06) as i16) < 0 {
                if (*E).var_E != 0 {
                    let mut SamusEnemyDelta_X: i16 =
                        ({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) as i16;
                    if SamusEnemyDelta_X < 0
                        && (SamusEnemyDelta_X).wrapping_sub(((*E).var_00) as i16) >= 0
                    {
                        (*E).var_A = (0xa687edi32) as u16;
                        if (*E).var_07 != 0 {
                            (*E).var_A = (0xa68942i32) as u16;
                        }
                    }
                } else {
                    let mut v3: i16 =
                        ({ let actor = word!(game.cur_enemy_index); actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X) }) as i16;
                    if v3 >= 0 && (v3).wrapping_sub(((*E).var_00) as i16) < 0 {
                        (*E).var_A = (0xa687edi32) as u16;
                        if (*E).var_07 != 0 {
                            (*E).var_A = (0xa68942i32) as u16;
                        }
                    }
                }
            }
        }
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_Boulder(word!(game.cur_enemy_index) as i32, game); };
    Boulder_Init [] {
        set_fields! {[(*E)], 0; var_C, var_B}
        (*E).var_D = 2;
        (*E).var_A = (0xa6879ai32) as u16;
        (*E).var_04 = (*E).base.x_pos;
        let y_pos: u16 = (*E).base.y_pos;
        (*E).var_05 = y_pos;
        (*E).var_03 = y_pos;
        (*E).var_02 = (*E).base.y_subpos;
        let mut boulder_parameter_2_high: i16 = byte!((*E).parameter_2; 1) as i16;
        if byte!((*E).parameter_2; 1) == 0 {
            boulder_parameter_2_high = 1;
            (*E).var_07 = 1;
        }
        (*E).base.y_pos = ((*E).var_05).wrapping_sub((boulder_parameter_2_high) as u16);
        (*E).var_00 = -(byte!((*E).parameter_2; 0) as i32) as u16;
        (*E).var_06 = byte!((*E).base.current_instruction; 0) as u16;
        (*E).base.current_instruction = addr_kBoulder_Ilist_86A7 as u16;
        let boulder_parameter_1_high: u16 = byte!((*E).parameter_1; 1) as u16;
        (*E).var_E = boulder_parameter_1_high;
        if boulder_parameter_1_high == 0 {
            (*E).var_00 = -((*E).var_00 as i32) as u16;
            (*E).base.current_instruction = addr_kBoulder_Ilist_86CB as u16;
        }
        (*E).var_01 = 2;
        if byte!((*E).parameter_1; 0) != 0 {
            (*E).var_01 = 5;
        }
    };
    Boulder_Main [] {
        CallBoulderFunc((((*E).var_A) as u32) | (0xa60000u32), game);
    };
    [10913773] Boulder_Func_2 [] {
        Boulder_Func_9((8u16).wrapping_mul((byte!((*E).var_C; 1)) as u16), game);
        if (((*E).base.y_pos) as i16).wrapping_sub(((*E).var_05) as i16) < 0 {
            let v1: u16 = ((*E).var_C).wrapping_add(256);
            (*E).var_C = v1;
            if v1 as i32 - 20480i32 & 0x8000 == 0 {
                (*E).var_C = 20480;
            }
        } else {
            (*E).base.y_pos = (*E).var_05;
            (*E).var_A = (0xa68832i32) as u16;
            (*E).var_C = 0x2000;
        };
    };
    [10913842] Boulder_Func_3 [] {
        Boulder_Func_9((8 * byte!((*E).var_C; 1) as i32 + 4) as u16, game);
        let mut v1: i16 = (((*E).var_C) as i16).wrapping_sub(256);
        (*E).var_C = v1 as u16;
        if v1 >= 0 {
            let mut v2: u16 = (8u16).wrapping_mul((byte!((*E).var_B; 1)) as u16);
            if (*E).var_E != 0 {
                v2 = (v2).wrapping_add(4);
            }
            Boulder_Func_7(v2, game);
            let v3: u16 = ((*E).var_B).wrapping_add(32);
            (*E).var_B = v3;
            if v3 as i32 - 20480i32 & 0x8000 == 0 {
                (*E).var_B = 20480;
            }
        } else {
            (*E).var_C = 0;
            (*E).var_A = (0xa6888bi32) as u16;
        };
    };
    [10913931] Boulder_Func_4 [] {
        let v1: i32 = 8 * byte!((*E).var_C; 1) as i32 >> 1;
        if ({ let (actor, amount) = (word!(game.cur_enemy_index), *(rom_ptr!(game, u32; 0xa0cbc7)).offset((v1 >> 1) as isize) as i32); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0
        {
            QueueSfx2_Max6(0x42, game);
            if (*E).var_E == 2 {
                (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
                game.eproj_spawn_pt = Point16U {
                    x: (*E).base.x_pos,
                    y: (*E).base.y_pos,
                };
                SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x11, game);
                QueueSfx2_Max6(0x43, game);
            } else {
                (*E).var_A = (0xa68832i32) as u16;
                (*E).var_C = g_word_A686F1[((*E).var_D as i32 - 1i32) as usize];
                (*E).var_D = (*E).var_D.wrapping_sub(1);
                if ((*E).var_D & 0x8000) != 0 {
                    (*E).var_03 = (*E).base.y_pos;
                    (*E).var_02 = (*E).base.y_subpos;
                    (*E).var_A = (0xa68942i32) as u16;
                }
            }
        } else {
            (*E).var_C = ((*E).var_C).wrapping_add(256);
            let mut v3: u16 = (8u16).wrapping_mul((byte!((*E).var_B; 1)) as u16);
            if (*E).var_E != 0 {
                v3 = (v3).wrapping_add(4);
            }
            Boulder_Func_7(v3, game);
            let v4: u16 = ((*E).var_B).wrapping_add(32);
            (*E).var_B = v4;
            if v4 as i32 - 20480i32 & 0x8000 == 0 {
                (*E).var_B = 20480;
            }
        };
    };
    [10914114] Boulder_Func_5 [] {
        let mut v2: u16 = (8u16).wrapping_mul((byte!((*E).var_B; 1)) as u16);
        let v3: i32 = v2 as i32 >> 1;
        ({ let (actor, amount) = (word!(game.cur_enemy_index), (*(rom_ptr!(game, u32; 0xa0cbc7)).offset((v3 >> 1i32) as isize))
                .wrapping_add((((*E).var_01 as i32) << 16) as u32) as i32); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 });
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub((*E).var_01);
        if (*E).var_E != 0 {
            v2 = (v2).wrapping_add(4);
        }
        let v4: i32 = v2 as i32 >> 1;
        if ({ let (actor, amount) = (word!(game.cur_enemy_index), *(rom_ptr!(game, u32; 0xa0cbc7)).offset((v4 >> 1) as isize) as i32); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) != 0
        {
            (*E).base.properties = ((*E).base.properties) | (0x300u16);
            (*E).var_A = (0xa689fci32) as u16;
            QueueSfx2_Max6(0x42, game);
            game.eproj_spawn_pt = Point16U {
                x: (*E).base.x_pos,
                y: (*E).base.y_pos,
            };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 0x11, game);
            QueueSfx2_Max6(0x43, game);
        } else {
            let v5: u16 = ((*E).var_B).wrapping_add(64);
            (*E).var_B = v5;
            if v5 as i32 - 0x4000i32 & 0x8000 == 0 {
                (*E).var_B = 0x4000;
            }
            if (*E).base.y_pos == (*E).var_03 && (*E).base.y_subpos == (*E).var_02 {
                (*E).var_01 = 0;
            }
        }
        (*E).var_03 = (*E).base.y_pos;
        (*E).var_02 = (*E).base.y_subpos;
    };
    @external {
        10914300 => [Boulder_Func_6()];
    }
}
pub fn Boulder_Func_6() {}
pub unsafe fn Boulder_Func_7(j: u16, game: &mut Game) {
    unsafe {
        let E = Get_Boulder(word!(game.cur_enemy_index) as i32, game);
        let v3: i32 = j as i32 >> 1;
        words!(
            (*E).base,
            x_pos,
            x_subpos += words!((
                crate::rtl::rom_table_word!(game, 0xa2838f, (v3 + 1) as isize),
                crate::rtl::rom_table_word!(game, 0xa2838f, v3 as isize)
            ))
        );
    }
}
pub unsafe fn Boulder_Func_9(j: u16, game: &mut Game) {
    unsafe {
        let E = Get_Boulder(word!(game.cur_enemy_index) as i32, game);
        let v3: i32 = j as i32 >> 1;
        words!(
            (*E).base,
            y_pos,
            y_subpos += words!((
                crate::rtl::rom_table_word!(game, 0xa2838f, (v3 + 1) as isize),
                crate::rtl::rom_table_word!(game, 0xa2838f, v3 as isize)
            ))
        );
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallSpikeyPlatformFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_SpikeyPlatform(word!(game.cur_enemy_index) as i32, game); };
    SpikeyPlatform_Init [] {
        (*E).base.current_instruction = addr_kSpikeyPlatform_Ilist_8B29 as u16;
        (*E).var_A = (0xa68bb4i32) as u16;
        let v1: u16 = (8u16).wrapping_mul((byte!((*E).parameter_1; 0)) as u16);
        (*E).var_02 = v1;
        let v2: i32 = v1 as i32 >> 1;
        (*E).var_E = crate::rtl::rom_table_word!(game, 0xa28187, (v2 + 1) as isize);
        (*E).var_F = crate::rtl::rom_table_word!(game, 0xa28187, v2 as isize);
        (*E).var_D = ((*E).base.y_pos).wrapping_add((byte!((*E).parameter_2; 1)) as u16);
        (*E).var_C = (*E).base.y_pos;
        let spm_parameter_2_low: u16 = byte!((*E).parameter_2; 0) as u16;
        (*E).var_00 = spm_parameter_2_low;
        (*E).var_B = spm_parameter_2_low;
    };
    SpikeyPlatform_Main [] {
        CallSpikeyPlatformFunc(
            (((*E).var_A) as u32) | (0xa60000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    [10914740] SpikeyPlatform_Func_1 [] {
        (*E).var_B = (*E).var_B.wrapping_sub(1);
        if (*E).var_B == 0 {
            (*E).var_B = (*E).var_00;
            (*E).var_02 = (8u16).wrapping_mul((byte!((*E).parameter_1; 0)) as u16);
            (*E).var_A = (0xa68bdci32) as u16;
        }
    };
    [10914780] SpikeyPlatform_Func_2 [] {
        (*E).var_01 = (*E).base.y_pos;
        let v1: i32 = (*E).var_02 as i32 >> 1;
        words!(
            (*E).base,
            y_pos,
            y_subpos += words!((
                crate::rtl::rom_table_word!(game, 0xa28187, v1 as isize),
                crate::rtl::rom_table_word!(game, 0xa28187, (v1 + 1) as isize)
            ))
        );
        if (((*E).base.y_pos) as i16).wrapping_sub(((*E).var_D) as i16) >= 0 {
            (*E).var_03 = 64;
            (*E).var_A = (0xa68c4ai32) as u16;
            (*E).base.y_pos = (*E).var_D;
            QueueSfx2_Max6(0x1b, game);
        }
        if SpikeyPlatform_Func_5(word!(game.cur_enemy_index), game) != 0 {
            word!(checked_i32, game.extra_samus_y_displacement += ((*E).base.y_pos as i32 - (*E).var_01 as i32));
        }
        let mut v4: i16 = (((*E).var_02) as i16).wrapping_add(8);
        if (*E).var_02 as i32 - 504i32 & 0x8000 == 0 {
            v4 = 512;
        }
        (*E).var_02 = v4 as u16;
    };
    [10914909] SpikeyPlatform_Func_4 [] {
        (*E).var_01 = (*E).base.y_pos;
        words!((*E).base, y_pos, y_subpos += -0x8000i32 as u32);
        if (((*E).base.y_pos) as i16).wrapping_sub(((*E).var_C) as i16) < 0 {
            (*E).var_A = (0xa68bb4i32) as u16;
            (*E).base.y_pos = (*E).var_C;
        }
        if SpikeyPlatform_Func_5(word!(game.cur_enemy_index), game) != 0 {
            word!(checked_i32, game.extra_samus_y_displacement += ((*E).base.y_pos as i32 - (*E).var_01 as i32));
        }
    };
    @external {
        10914890 => [SpikeyPlatform_Func_3(k, game)];
    }
}
pub unsafe fn SpikeyPlatform_Func_3(k: u16, game: &mut Game) {
    unsafe {
        let E = Get_SpikeyPlatform(k as i32, game);
        let v2: u16 = ((*E).var_03).wrapping_sub(1);
        (*E).var_03 = v2;
        if v2 == 0 {
            (*E).var_A = (0xa68c5di32) as u16;
        }
    }
}
pub unsafe fn SpikeyPlatform_Func_5(k: u16, game: &mut Game) -> u16 {
    unsafe {
        let E = Get_SpikeyPlatform(k as i32, game);
        let v2: u16 = abs16((word!(game.samus_x_pos)).wrapping_sub((*E).base.x_pos));
        let mut v3: bool = v2 < word!(game.samus_x_radius);
        let v4: u16 = (v2).wrapping_sub(word!(game.samus_x_radius));
        if !v3 && v4 >= (*E).base.x_width {
            return 0;
        }
        if ((word!(game.samus_y_pos) as i32 + 5 - (*E).base.y_pos as i32) as i16) < 0 {
            let v6: u16 = ((*E).base.y_pos as i32 - (word!(game.samus_y_pos) as i32 + 5)) as u16;
            v3 = v6 < word!(game.samus_y_radius);
            let v7: u16 = (v6).wrapping_sub(word!(game.samus_y_radius));
            if v3 || v7 == (*E).base.y_height || v7 < (*E).base.y_height {
                return -1i32 as u16;
            }
        }
        0
    }
}
// Each animation step reads the live parent, then selects the adjacent child.
unsafe fn set_fire_geyser_shape(offset: u16, height: u16, begin: bool, game: &mut Game) {
    unsafe {
        let y = (*Get_FireGeyser(word!(game.cur_enemy_index) as i32, game))
            .var_D
            .wrapping_sub(offset);
        let child = Get_FireGeyser(word!(game.cur_enemy_index) as i32 + 64, game);
        (*child).base.y_pos = y;
        (*child).base.y_height = height;
        if begin {
            (*child).base.x_width = 8;
        }
    }
}

crate::game::callback_dispatch! {
    pub unsafe fn CallFireGeyserFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10915887] FireGeyser_Func_1 [] {
        let E = Get_FireGeyser(word!(game.cur_enemy_index) as i32, game);
        (*E).var_B = (*E).var_B.wrapping_sub(1);
        if ((*E).var_B & 0x8000) != 0 {
            (*E).var_A = (0xa69062i32) as u16;
            (*E).var_C = 0;
            (*E).base.instruction_timer = 1;
            (*E).base.timer = 0;
            (*E).base.current_instruction = addr_kFireGeyser_Ilist_8D1B as u16;
            (*E).base.properties =
                ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
            let E1 = Get_FireGeyser(word!(game.cur_enemy_index) as i32 + 64, game);
            (*E1).base.properties = (*E1).base.properties & !(0x400);
        }
    };
    [10915938] FireGeyser_Func_2 [] {
        let E = Get_FireGeyser(word!(game.cur_enemy_index) as i32, game);
        if (*E).var_C != 0 {
            (*E).var_B = (*E).parameter_1;
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
            (*E).var_A = (0xa6902fi32) as u16;
        }
    };
}
crate::game::callback_dispatch! {
    pub unsafe fn CallNuclearWaffleFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10917397] NuclearWaffle_Func_1 [] {
        let E = Get_NuclearWaffle(word!(game.cur_enemy_index) as i32, game);
        (*E).var_B = (*E).var_B.wrapping_sub(1);
        if ((*E).var_B & 0x8000) != 0 {
            (*E).var_B = (*E).var_F;
            (*E).var_21 = (*E).var_23;
            (*E).var_A = (0xa69682i32) as u16;
            set_fields! {[(*E)], 0; var_33, var_08, var_09, var_0A, var_0B, var_0C, var_0D, var_0E, var_0F, var_18, var_19, var_1A, var_1B, var_1C, var_1D, var_1E, var_1F}
            (*E).base.properties =
                ((*E).base.properties) | ((kEnemyProps_ProcessedOffscreen) as u16);
        }
    };
    [10917506] NuclearWaffle_Func_2 [] {
        crate::game::enemy_projectiles::advance_waffle_orbit(game);
    };
}
pub unsafe fn FakeKraid_Func_1(k: u16, j: u16, game: &mut Game) {
    unsafe {
        let ET = Get_FakeKraid(k as i32 + j as i32, game);
        let fkd_var_03: u16 = (*ET).var_03;
        if fkd_var_03 != 0 {
            (*ET).var_03 = (fkd_var_03).wrapping_sub(1);
        } else {
            (*ET).var_03 = ((word!(game.random_number) as i32 & 0x3f) + 16) as u16;
            let E = Get_FakeKraid(k as i32, game);
            (*E).var_06 = j;
            let mut v6: u16 = addr_kEproj_MiniKraidSpikesLeft as u16;
            let fkd_var_C: i16 = (*E).var_C as i16;
            if fkd_var_C >= 0 {
                v6 = addr_kEproj_MiniKraidSpikesRight as u16;
            }
            SpawnEprojWithGfx(fkd_var_C as u16, k, v6, game);
            if u16::from(actor_api::current_outside_screen(&Runtime::new(game))) == 0 {
                QueueSfx2_Max6(0x3f, game);
            }
        };
    }
}
pub unsafe fn FakeKraid_InstrHelper_45(k: u16, j: u16, a: u16, game: &mut Game) {
    unsafe {
        let ka: u16 = word!(game.cur_enemy_index);
        let E = Get_FakeKraid(word!(game.cur_enemy_index) as i32, game);
        (*E).var_02 = a;
        let v5: i32 = j as i32 >> 1;
        (*E).var_00 = g_word_A69A48[v5 as usize];
        let v6: u16 = g_word_A69A48[(v5 + 1) as usize];
        (*E).var_01 = v6;
        SpawnEprojWithGfx(
            v6,
            word!(game.cur_enemy_index),
            addr_kEproj_MiniKraidSpit as u16,
            game,
        );
        (*E).var_00 = g_word_A69A48[(v5 + 2) as usize];
        let v8: u16 = g_word_A69A48[(v5 + 3) as usize];
        (*E).var_01 = v8;
        SpawnEprojWithGfx(v8, ka, addr_kEproj_MiniKraidSpit as u16, game);
    }
}
pub unsafe fn FakeKraid_9C50(game: &mut Game) {
    unsafe {
        if (*Get_FakeKraid(word!(game.cur_enemy_index) as i32, game))
            .base
            .health
            == 0
        {
            // The death animation remains in the ROM spawn parameter for all four drops.
            let death_animation = 3;
            ({
                let (_ignored_key, animation) = (word!(game.cur_enemy_index), death_animation);
                actors::combat::die(
                    &mut Runtime::new(game),
                    actors::api::DeathKind::Normal { animation },
                )
            });
            scatter(DropSource::MiniKraid, word!(game.cur_enemy_index), game);
        }
    }
}

pub fn CeresRidley_Func_1(game: &mut Game) {
    {
        if word!(game.random_number) >= 0xff00 {
            word!(game.tilemap_stuff[15] = ((word!(game.random_number) as i32 & 0xf) + 8) as u16);
        }
    }
}
pub unsafe fn Ridley_A2DC(game: &mut Game) {
    unsafe {
        Ridley_Func_49(game);
        if (*Get_CeresRidley(0i32, game)).var_02 != 0 {
            mov24(ram_ptr!(game, LongPtr; 0x178c), 0xa6a2f2);
        } else {
            Ridley_A2F2(game);
        };
    }
}
pub unsafe fn Ridley_A2F2(game: &mut Game) {
    unsafe {
        if word!(game.ceres_status) == 0 {
            DrawBabyMetroid_0(game);
        }
        let E = Get_CeresRidley(0x40, game);
        if (*E).var_B != 0 {
            static g_word_A6A321: [i16; 4] = [0, -1024, -4, -1];
            sprite_output::paletted(
                0xa6,
                addr_kCeresRidley_Sprmap_A329 as u16,
                (
                    ((*E).base.x_pos).wrapping_add(
                        (g_word_A6A321[(word!(game.earthquake_timer) as i32 & 3) as usize]) as u16,
                    ),
                    (*E).base.y_pos,
                ),
                1024,
                Clip::Bounded,
                game,
            );
        }
    }
}
pub unsafe fn CeresRidley_Func_8(a: u16, game: &mut Game) -> u8 {
    unsafe {
        let v1: u16 = crate::rtl::rom_table_word!(game, 0xa6a4eb, a as isize);
        if v1 == 0 {
            return 1;
        }
        WriteColorsToPalette(0xe2, 0xa6, v1, 0xe, game);
        0
    }
}
pub unsafe fn CeresRidley_Func_12(game: &mut Game) -> u8 {
    unsafe {
        let rect: Rect16U = Rect16U {
            x: 192,
            y: 100,
            w: 8,
            h: 8,
        };
        Ridley_Func_106(0, 0, rect.x, rect.y, game);
        Shitroid_Func_2(0, rect, game)
    }
}
pub unsafe fn CeresRidley_Func_25(game: &mut Game) {
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
pub unsafe fn CeresRidley_Func_27(game: &mut Game) {
    unsafe {
        *ram_ptr!(game; 0x56) = 7;
        word!(game.irq_enable_mode7 = 1);
        *ram_ptr!(game; 0x5f) = 0x80;
        word! { game, 256; reg_M7A, reg_M7B, reg_M7C, reg_M7D }
        word! { game, 64; reg_M7X, reg_M7Y }
        let E = Get_CeresRidley(0, game);
        set_fields! {[(*E)], 0; var_32, var_33}
        (*E).var_0A = 1;
        (*E).var_34 = -128i32 as u16;
        word!(game.reg_BG1HOFS = -128i32 as u16);
        (*E).var_35 = 32;
        word!(game.reg_BG1VOFS = 32);
        (*E).var_30 = 2048;
        (*E).var_31 = 1024;
    }
}
pub unsafe fn HandleCeresRidleyGetawayCutscene(game: &mut Game) {
    unsafe {
        if (*Get_CeresRidley(0i32, game)).var_32 == 0 {
            CeresRidley_Func_28(game);
        }
    }
}
unsafe fn CeresMult(a: u16, b: u16) -> u16 {
    let r: u16 = (abs16(a) as i32 * b as i32 >> 8) as u16;
    (if a & 0x8000 != 0 {
        -(r as i32)
    } else {
        r as i32
    }) as u16
}
pub unsafe fn CeresRidley_AC30(a: u16, r20: i16) -> i16 {
    (kSinCosTable8bit_Sext[((a as i32 & 0xff) + 64) as usize] as i32 * r20 as i32 >> 8) as i16
}
pub unsafe fn CeresRidley_Func_33(game: &mut Game) {
    unsafe {
        if (word!(game.nmi_frame_counter_word) & 3) == 0 {
            let E = Get_CeresRidley(0, game);
            let v1: u16 = ((*E).var_36 as i32 + 1 & 3) as u16;
            (*E).var_36 = v1;
            QueueMode7Transfers(
                0xa6,
                crate::rtl::rom_table_word!(game, 0xa6acda, v1 as isize),
                game,
            );
        }
    }
}
pub unsafe fn CeresRidley_Func_34(game: &mut Game) {
    unsafe {
        if (word!(game.nmi_frame_counter_word) & 7) == 0 {
            let E = Get_CeresRidley(0, game);
            let v1: u16 = ((*E).var_37 as i32 + 1 & 1) as u16;
            (*E).var_37 = v1;
            QueueMode7Transfers(
                0xa6,
                crate::rtl::rom_table_word!(game, 0xa6ad45, v1 as isize),
                game,
            );
        }
    }
}
pub unsafe fn CeresRidley_Func_35(mut a: u16, game: &mut Game) {
    unsafe {
        WriteColorsToPalette(
            0xa2,
            0xa6,
            (32 * byte!(a; 1) as i32 - 0x4ef9i32) as u16,
            0xf,
            game,
        );
    }
}
pub unsafe fn Ridley_Func_4(game: &mut Game) -> u16 {
    unsafe {
        let mut r18: u16 = 0;
        if *ram_ptr!(game; 0xa1f) == 3 {
            r18 = addr_off_A6B3CC as u16;
        } else {
            let E = Get_Ridley(0, game);
            let health: u16 = (*E).base.health;
            if health != 0 {
                if health as i32 - 14400i32 & 0x8000 != 0 {
                    r18 = addr_off_A6B38C as u16;
                } else if Ridley_Func_41(game) & 1 != 0 {
                    r18 = addr_off_A6B3BC as u16;
                } else if (Ridley_Func_40_Carry(game) & 1) != 0 {
                    r18 = addr_off_A6B3AC as u16;
                } else {
                    r18 = (if (*E).base.health as i32 - 9000i32 & 0x8000 != 0 {
                        addr_off_A6B39C as i32
                    } else {
                        addr_off_A6B38C as i32
                    }) as u16;
                }
            } else {
                r18 = addr_off_A6B3DC as u16;
                (*E).var_25 = (*E).var_25.wrapping_add(1);
            }
        }
        r18
    }
}
pub unsafe fn Ridley_Func_7(game: &mut Game) -> u16 {
    unsafe {
        static g_word_A6B439: [u16; 4] = [4, 8, 0xa, 0xc];
        g_word_A6B439[(*Get_Ridley(0i32, game)).var_12 as usize]
    }
}
macro_rules! advance_ridley_instruction_with_timeout {
    ($game:ident, $instruction_table:literal, $timeout_state:literal) => {
        let E = Get_Ridley(0, $game);
        if Ridley_Func_17(
            crate::rtl::rom_table_word!($game, $instruction_table, (*E).var_10 as isize),
            $game,
        ) & 1
            != 0
        {
            Ridley_Func_18($game);
        } else {
            (*E).var_F = (*E).var_F.wrapping_sub(1);
            if ((*E).var_F & 0x8000) != 0 {
                (*E).var_A = (($timeout_state) as u16);
                (*E).var_F = 128;
                Ridley_Func_114($game);
            }
        }
    };
}

pub unsafe fn Ridley_Func_17(r18: u16, game: &mut Game) -> u8 {
    unsafe {
        let mut v0: u16 = word!(game.samus_y_pos);
        if word!(game.samus_y_pos) as i32 - 352i32 & 0x8000 == 0 {
            v0 = 352;
        }
        Ridley_Func_104_0(0, Ridley_Func_7(game), r18, v0, game);
        word!(game.tilemap_stuff[2] = 1);
        if *ram_ptr!(game; 0xa1f) != 3 {
            return 1;
        }
        if (word!(game.random_number) as u8) >= 0x80 {
            let E = Get_Ridley(0, game);
            if (*E).var_0F == 0 && (*E).var_10 != 1 {
                Ridley_Func_99(addr_kRidley_Ilist_E73A as u16, game);
            }
        }
        0
    }
}
pub unsafe fn Ridley_Func_18(game: &mut Game) {
    unsafe {
        word! {game; tilemap_stuff[9] = 240; tilemap_stuff[15] = 16; tilemap_stuff[0] = 1; }
        (*Get_Ridley(0i32, game)).var_A = (0xa6b6a7i32) as u16;
    }
}
pub unsafe fn Ridley_Func_23(game: &mut Game) -> u8 {
    unsafe {
        let mut v0: u8 = (Ridley_Func_103(
            word!(game.tilemap_stuff[82]),
            (word!(game.tilemap_stuff[83])).wrapping_add(16),
            game,
        )) & (1u8);
        if v0 == 0 {
            v0 = (Ridley_Func_103(
                word!(game.tilemap_stuff[72]),
                (word!(game.tilemap_stuff[73])).wrapping_add(18),
                game,
            )) & (1u8);
            if v0 == 0 {
                v0 = (Ridley_Func_103(
                    word!(game.tilemap_stuff[62]),
                    (word!(game.tilemap_stuff[63])).wrapping_add(18),
                    game,
                )) & (1u8);
                if v0 == 0 {
                    v0 = (Ridley_Func_103(
                        word!(game.tilemap_stuff[52]),
                        (word!(game.tilemap_stuff[53])).wrapping_add(18),
                        game,
                    )) & (1u8);
                    if v0 == 0 {
                        return (Ridley_Func_103(
                            word!(game.tilemap_stuff[42]),
                            (word!(game.tilemap_stuff[43])).wrapping_add(18),
                            game,
                        )) & (1u8);
                    }
                }
            }
        }
        v0
    }
}
pub fn Ridley_Func_24(game: &mut Game) {
    {
        word! { game, 1; tilemap_stuff[0], tilemap_stuff[10] }
    }
}
pub unsafe fn Ridley_Func_25(game: &mut Game) -> u8 {
    unsafe {
        if (Ridley_Func_40_Carry(game) & 1) != 0 {
            (Ridley_Func_26(4, 4, game)) & (1u8)
        } else {
            0
        }
    }
}
pub unsafe fn Ridley_Func_26(k: u16, j: u16, game: &mut Game) -> u8 {
    unsafe {
        let E = Get_Ridley(0, game);
        let rect: Rect16U = Rect16U {
            x: (((*E).base.x_pos).wrapping_add(crate::rtl::rom_table_word!(
                game,
                0xa6b9d5,
                (*E).var_10 as isize
            ))),
            y: (((*E).base.y_pos).wrapping_add(crate::rtl::rom_table_word!(
                game,
                0xa6b9db,
                ((*E).var_1D >> 1) as isize
            ))),
            w: k,
            h: j,
        };
        Ridley_Func_124(rect, game)
    }
}
pub unsafe fn Ridley_Func_29(game: &mut Game) {
    unsafe {
        let v0: i32 = word!(game.random_number) as i32 & 3;
        let r18: u16 = crate::rtl::rom_table_word!(game, 0xa6b965, v0 as isize);
        let r20: u16 = crate::rtl::rom_table_word!(game, 0xa6b96d, v0 as isize);
        let E = Get_Ridley(0, game);
        let v2: u16 = (2 * ((*E).var_12 as i32 + 2)) as u16;
        let v3: i32 = v2 as i32 >> 1;
        word!(game.tilemap_stuff[7] = crate::rtl::rom_table_word!(game, 0xa6b94d, v3 as isize));
        word!(game.tilemap_stuff[6] = crate::rtl::rom_table_word!(game, 0xa6b959, v3 as isize));
        (*E).var_C = *((RomPtr_A6 as unsafe fn(u16, &mut Game) -> *const u8)(r20, game)
            .offset(v2 as isize) as *mut u16);
        let v4 = RomPtr_A6(r18, game);
        let t: u16 = *(v4.offset(v2 as isize) as *mut u16);
        (*E).var_B = (if (*E).var_B & 0x8000 != 0 {
            -(t as i32)
        } else {
            t as i32
        }) as u16;
    }
}

macro_rules! reduce_ridley_axis_speed {
    ($velocity:ident, $reduced_velocity:ident, $enemy:ident, $velocity_field:ident) => {
        if $velocity != 0 {
            let v2: i16 = ((abs16($velocity)) as i16).wrapping_sub(4);
            if v2 >= 0 {
                $reduced_velocity = (if $velocity & 0x8000 != 0 {
                    -(v2 as i32)
                } else {
                    v2 as i32
                }) as u16;
            } else {
                $reduced_velocity = 0;
            }
            (*$enemy).$velocity_field = $reduced_velocity;
        }
    };
}

crate::game::callback_dispatch! {
    pub unsafe fn CallRidleyFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10920795] CeresRidley_Func_3 [] {
        if word!(game.door_transition_flag_enemies) == 0 {
            let E = Get_CeresRidley(0, game);
            (*E).var_A = (0xa6a377i32) as u16;
            let mut v1: i16 = 512;
            if word!(game.area_index) == 2 {
                v1 = 170;
            }
            (*E).var_F = v1 as u16;
            CeresRidley_A377(game);
        }
    };
    [10921977] CeresRidley_A7F9 [] {
        let r22: u16 = (if word!(game.random_number) & 0x8000 != 0 {
            -(word!(game.random_number) as i32 & 7i32)
        } else {
            word!(game.random_number) as i32 & 7
        }) as u16;
        let E = Get_CeresRidley(0, game);
        let r18: u16 = ((*E).var_17).wrapping_add(r22);
        let r20: u16 = ((*E).var_18).wrapping_add(r22);
        Ridley_Func_106(0, 0, r18, r20, game);
        let v4: i16 = (((*E).var_00) as i16).wrapping_sub(1);
        (*E).var_00 = v4 as u16;
        if v4 < 0 {
            (*E).var_00 = 0;
            (*E).var_A = (0xa6a6e8i32) as u16;
        }
    };
    [10922044] CeresRidley_Func_14 [] {
        Ridley_Func_99(addr_kRidley_Ilist_E548 as u16, game);
        let E = Get_CeresRidley(0, game);
        (*E).var_A = (0xa6a84ei32) as u16;
        (*E).var_F = 64;
        CeresRidley_Func_15(game);
    };
    [10922148] CeresRidley_Func_17 [] {
        Ridley_Func_106(0, 1, 192, 80, game);
        let E = Get_CeresRidley(0, game);
        if (*E).base.y_pos as i32 - 96i32 & 0x8000 != 0 {
            (*E).var_A = (0xa6a8d4i32) as u16;
            (*E).var_F = 10;
            (*E).var_0A = 0;
        }
    };
    [10922196] CeresRidley_Func_18 [] {
        Ridley_Func_111(-32i32 as u16, -1024i32 as u16, 768, game);
        let E = Get_CeresRidley(0, game);
        (*E).var_F = (*E).var_F.wrapping_sub(1);
        if ((*E).var_F & 0x8000) != 0 {
            (*E).var_A = (0xa6a8f8i32) as u16;
            (*E).var_F = 36;
        }
    };
    [10922232] CeresRidley_Func_19 [] {
        Ridley_Func_111(-512i32 as u16, -16384i32 as u16, 768, game);
        let E = Get_CeresRidley(0, game);
        (*E).var_F = (*E).var_F.wrapping_sub(1);
        if ((*E).var_F & 0x8000) != 0 {
            (*E).var_A = (0xa6a923i32) as u16;
            (*E).var_F = 28;
            word!(game.tilemap_stuff[2] = 1);
        }
    };
    [10922275] CeresRidley_Func_20 [] {
        Ridley_Func_111(-512i32 as u16, -30720i32 as u16, 768, game);
        let E = Get_CeresRidley(0, game);
        (*E).var_F = (*E).var_F.wrapping_sub(1);
        if ((*E).var_F & 0x8000) != 0 {
            (*E).var_A = (0xa6a947i32) as u16;
            (*E).var_F = 1;
        }
    };
    [10922311] CeresRidley_Func_21 [] {
        Ridley_Func_111(-768i32 as u16, -30720i32 as u16, 768, game);
        let E = Get_CeresRidley(0, game);
        (*E).var_F = (*E).var_F.wrapping_sub(1);
        if ((*E).var_F & 0x8000) != 0 {
            (*E).var_A = (0xa6a6e8i32) as u16;
            (*E).var_00 = 0;
            word!(game.tilemap_stuff[1] = 1);
        }
    };
    [10922513] CeresRidley_Func_24 [] {
        (*Get_CeresRidley(0i32, game)).var_A = (0xa6aa4fi32) as u16;
        word!(game.ceres_status = 1);
        CeresRidley_Func_27(game);
    };
    [10922576] CeresRidley_Func_26 [] {
        Ridley_Func_61(game);
    };
    [10924787] Ridley_Func_3 [] {
        let rect: Rect16U = Rect16U {
            x: 64,
            y: 256,
            w: 8,
            h: 8,
        };
        Ridley_Func_104_0(0, 14, rect.x, rect.y, game);
        if Shitroid_Func_2(0, rect, game) == 0 {
            let E = Get_Ridley(0, game);
            (*E).var_01 = 1;
            (*E).var_A = (0xa6b321i32) as u16;
            Ridley_Func_3b(game);
        }
    };
    [10924833] Ridley_Func_3b [] {
        let r18: u16 = Ridley_Func_4(game);
        let v0: i32 = 2 * (NextRandom(game) as i32 & 7);
        let v1: u16 = *((RomPtr_A6 as unsafe fn(u16, &mut Game) -> *const u8)(r18, game)
            .offset(v0 as isize) as *mut u16);
        let E = Get_Ridley(0, game);
        (*E).var_A = v1;
        CallRidleyFunc((((*E).var_A) as u32) | (0xa60000u32), game);
    };
    [10925048] Ridley_Func_6 [] {
        let mut rect: Rect16U = Rect16U {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        };

        let E = Get_Ridley(0, game);
        (*E).var_F = (*E).var_F.wrapping_sub(1);
        let change_ridley_state: bool = (((*E).var_F & 0x8000) != 0)
            || ({
                let mut v2: u16 = 192;
                if (*E).var_10 != 0 {
                    v2 = 96;
                }
                rect = Rect16U {
                    x: v2,
                    y: 256,
                    w: 8,
                    h: 8,
                };
                Ridley_Func_104_0(0, Ridley_Func_7(game), rect.x, rect.y, game);
                Shitroid_Func_2(0, rect, game) == 0
            });
        if change_ridley_state {
            (*E).var_A = (0xa6b321i32) as u16;
        };
    };
    [10925460] Ridley_Func_13 [] {
        Ridley_Func_111(0, 0x8000, 448, game);
        let E = Get_Ridley(0, game);
        let ridley_var_00: u16 = (*E).var_00;
        if ridley_var_00 != 0 {
            (*E).var_00 = (ridley_var_00).wrapping_sub(1);
        } else {
            (*E).var_A = (if Ridley_Func_40_Carry(game) != 0 {
                0xa6bab7 & 0xffff
            } else {
                0xa6b321 & 0xffff
            }) as u16;
        };
    };
    [10925508] Ridley_Func_14 [] {
        word! {game; tilemap_stuff[15] = 11; tilemap_stuff[9] = 384; }
        let E = Get_Ridley(0, game);
        (*E).var_A = (0xa6b5e5i32) as u16;
        (*E).var_F = ((word!(game.random_number) as i32 & 0x1f) + 32) as u16;
        Ridley_Func_15(game);
    };
    [10925541] Ridley_Func_15 [] {
        advance_ridley_instruction_with_timeout!(game, 0xa6b60d, 0xa6b613i32);
    };
    [10925587] Ridley_Func_16 [] {
        advance_ridley_instruction_with_timeout!(game, 0xa6b63b, 0xa6b5e5i32);
    };
    [10925838] Ridley_Func_21 [] {
        if Ridley_Func_25(game) & 1 != 0 {
            Ridley_Func_24(game);
            Ridley_Func_27(game);
        } else {
            let E = Get_Ridley(0, game);
            let mut v1: i16 =
                ((word!(game.tilemap_stuff[6])) as i16).wrapping_add(((*E).var_C) as i16);
            if v1 as i32 - 1536i32 & 0x8000 == 0 {
                v1 = 1536;
            }
            (*E).var_C = v1 as u16;
            if Ridley_Func_23(game) & 1 != 0 {
                game.eproj_spawn_pt = Point16U {
                    x: word!(game.tilemap_stuff[82]),
                    y: ((word!(game.tilemap_stuff[83])).wrapping_add(12)),
                };
                SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, game);
                QueueSfx2_Max6(0x76, game);
                word! {game; earthquake_type = 13; earthquake_timer = 4; }
                Ridley_Func_28(game);
                Ridley_Func_29(game);
                Ridley_Func_93(0x2026, addr_word_A6D37C as u16, game);
                word! { game, 3072;
                    tilemap_stuff[20], tilemap_stuff[30], tilemap_stuff[40], tilemap_stuff[50],
                    tilemap_stuff[60], tilemap_stuff[70], tilemap_stuff[80],
                }
                word!(game.tilemap_stuff[0] = 4);
                let mut v2: u16 = ((*E).var_26).wrapping_add(1);
                if (*E).var_26 as i32 - 1i32 & 0x8000 == 0 {
                    if (*E).var_10 != 1 {
                        Ridley_Func_99(addr_kRidley_Ilist_E73A as u16, game);
                    }
                    v2 = 0;
                }
                (*E).var_26 = v2;
                (*E).var_A = (0xa6b7b9i32) as u16;
            }
        };
    };
    [10927438] Ridley_Func_43 [] {
        if word!(game.power_bomb_flag) != 0 {
            (*Get_Ridley(0i32, game)).var_01 = 2;
            let r18: u16 = (if word!(game.power_bomb_explosion_x_pos) as i32 - 128i32 & 0x8000 != 0
            {
                192
            } else {
                80
            }) as u16;
            let r20: u16 = (if word!(game.power_bomb_explosion_y_pos) as i32 - 256i32 & 0x8000 != 0
            {
                384
            } else {
                192
            }) as u16;
            Ridley_Func_104_0(0, Ridley_Func_7(game), r18, r20, game);
        } else {
            let E = Get_Ridley(0, game);
            (*E).var_01 = 1;
            (*E).var_A = (if (*E).var_1B != 0 {
                0xa6bb8f & 0xffff
            } else {
                0xa6b321 & 0xffff
            }) as u16;
        };
    };
    [10927514] Ridley_Func_44 [] {
        Ridley_Func_106(0, 1, 192, 128, game);
        let E = Get_Ridley(0, game);
        if (*E).base.x_pos as i32 - 192i32 & 0x8000 == 0 {
            (*E).var_A = (0xa6bdbci32) as u16;
        }
    };
    [10928206] Ridley_Func_54 [] {
        let ridley_var_F: u16 = (*Get_Ridley(0i32, game)).var_F;
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
    [10929464] Ridley_Func_63 [] {
        if Ridley_Func_68(game) & 1 == 0 {
            Ridley_C53E(game);
        }
    };
    [10929470] Ridley_C53E [] {
        Ridley_Func_99(addr_kRidley_Ilist_E6C8 as u16, game);
        let E = Get_Ridley(0, game);
        (*E).var_A = (0xa6c551i32) as u16;
        (*E).var_F = 32;
    };
    [10929489] Ridley_C551 [] {
        Ridley_Func_68(game);
        let E = Get_Ridley(0, game);
        (*E).var_F = (*E).var_F.wrapping_sub(1);
        if ((*E).var_F & 0x8000) != 0 {
            set_fields! {[(*E)], 0; var_B, var_C}
            word! {game; fx_target_y_pos = 528; fx_y_vel = 64; fx_timer = 1; }
            set_fields! {[(*E)], 0; var_27, var_28}
            (*E).var_A = (0xa6c588i32) as u16;
            (*E).var_F = 160;
            Ridley_Func_64(game);
        }
    };
    [10929544] Ridley_Func_64 [] {
        Ridley_Func_69(game);
        let E = Get_Ridley(0, game);
        (*E).var_F = (*E).var_F.wrapping_sub(1);
        if ((*E).var_F & 0x8000) != 0 {
            if (*E).var_1B != 0 {
                Ridley_Func_38(game);
            }
            (*E).var_A = (0xa6c5a8i32) as u16;
            set_fields! {[(*E)], 0; var_B, var_C}
            RidleysExplosion_Func_2(game);
        }
    };
    [10929576] Ridley_Func_65 [] {
        Ridley_Func_69(game);
        let E = Get_Ridley(0, game);
        (*E).var_02 = 0;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        (*E).var_A = (0xa6c5c8i32) as u16;
        (*E).var_F = 32;
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_CeresRidley(0, game); };
    CeresRidley_Init [] {
        if (gameplay_flag_runtime::boss_byte_at(word!(game.area_index), game) & 1) != 0 {
            (*E).base.properties = ((*E).base.properties as i32
                | (kEnemyProps_Tangible as i32
                    | kEnemyProps_Deleted as i32
                    | kEnemyProps_Invisible as i32)) as u16;
        } else {
            word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(0));
            minimap::disable_for_boss(&mut crate::game::session_runtime::Runtime::new(game));
            set_fields! {[(*E)], 0; parameter_1, parameter_2}
            Ridley_Func_99(addr_kRidley_Ilist_E538 as u16, game);
            (*E).base.palette_index = 3584;
            (*E).var_0C = 3584;
            (*E).base.extra_properties = ((*E).base.extra_properties) | (4u16);
            set_fields! {[(*E)], 0; var_01, var_0D}
            word! { game, 0; earthquake_type, earthquake_timer }
            Ridley_Func_92(game);
            Ridley_Func_117(game);
            if word!(game.area_index) == 2 {
                (*E).base.properties = ((*E).base.properties as i32
                    | (kEnemyProps_BlockPlasmaBeam as i32 | kEnemyProps_Tangible as i32))
                    as u16;
                (*E).base.x_pos = 96;
                (*E).base.y_pos = 394;
                (*E).var_A = (0xa6a35bi32) as u16;
                set_fields! {[(*E)], 0; var_B, var_C}
                (*E).base.layer = 5;
                set_fields! {[(*E)]; var_01 = 0, var_02 = 1, var_20 = 64, var_21 = 416,
                    var_22 = 64, var_23 = 224, var_10 = 2, var_1C = 120, };
                WriteColorsToTargetPalette(0xa6, 0x140, addr_word_A6E1CF as u16, 0x20, game);
                word_transfers!(game, ascending, 15;
                    le(target_palettes, 113) <- value(0),
                    le(target_palettes, 241) <- value(0),
                );
            } else {
                (*E).base.properties = ((*E).base.properties as i32
                    | (kEnemyProps_BlockPlasmaBeam as i32 | kEnemyProps_Tangible as i32))
                    as u16;
                (*E).base.x_pos = 186;
                (*E).base.y_pos = 169;
                word!(game.ceres_status = 0);
                set_fields! {[(*E)], 0; var_10, var_02}
                word!(game.tilemap_stuff[1] = 1);
                (*E).var_1C = 15;
                (*E).var_A = (0xa6a35bi32) as u16;
                set_fields! {[(*E)], 0; var_B, var_C}
                (*E).var_20 = -32i32 as u16;
                (*E).var_21 = 176;
                (*E).var_22 = 40;
                (*E).var_23 = 224;
                WriteColorsToTargetPalette(0xa6, 0x140, addr_word_A6E16F as u16, 0x20, game);
                word_transfers!(game, ascending, 15; le(target_palettes, 241) <- value(0));
                (*E).var_03 = addr_kBabyMetroid_Ilist_BF31 as u16;
                (*E).var_04 = 1;
                (*gRam8800_Default(0i32, game)).var_40 = (0xa6be9ci32) as u16;
                (*E).var_07 = 5;
                QueueMusic_Delayed8(0, game);
            }
        };
    };
    CeresRidley_Main [] {
        (*E).base.health = 0x7fff;
        CallRidleyFunc((((*E).var_A) as u32) | (0xa60000u32), game);
        if word!(game.ceres_status) == 0 {
            if (*Get_CeresRidley(0i32, game)).var_02 != 0 {
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
        set_fields! {[(*E)], 0; var_B, var_C}
    };
    [10920841] CeresRidley_Func_4 [] {
        if ((*E).var_E & 0x8000) == 0 {
            let cry_var_E: u16 = (*E).var_E;
            (*E).var_F = (*E).var_F.wrapping_add(1);
            if (*E).var_F != 0 {
                (*E).var_F = 0;
                let mut v2: i16 = byte!(@pointer rom_ptr!(game; 0xa6e269); cry_var_E as isize) as i16;
                if v2 == 255 {
                    (*E).var_E = 0;
                    (*E).var_A = (0xa6a3dfi32) as u16;
                    (*E).var_02 = 1;
                } else {
                    (*E).var_E = (cry_var_E).wrapping_add(1);
                    let v3: i32 = ((6u16).wrapping_mul((v2) as u16)) as i32 >> 1;
                    word! { game;
                        palette_buffer[252] = crate::rtl::rom_table_word!(game, 0xa6e2aa, v3 as isize);
                        palette_buffer[253] =
                            crate::rtl::rom_table_word!(game, 0xa6e2aa, (v3 + 1) as isize);
                        palette_buffer[254] =
                            crate::rtl::rom_table_word!(game, 0xa6e2aa, (v3 + 2) as isize);
                    }
                }
            }
        }
    };
    [10920927] CeresRidley_Func_5 [] {
        (*E).var_F = (*E).var_F.wrapping_add(1);
        if (*E).var_F >= 2 {
            (*E).var_F = 0;
            let mut cry_var_E: u16 = (*E).var_E;
            let mut r18: u16 = 290;
            let mut r20: u16 = 482;
            let mut v2: i16 = 11;
            do_until! { v5 == 1 ;
    let mut v5: i16 = v2;
    let v3: u16 = crate::rtl::rom_table_word!(game, 0xa6e30a, (cry_var_E >> 1) as isize);
    word! {game; palette_buffer[(r18 >> 1)] = v3; palette_buffer[(r20 >> 1)] = v3; }
    cry_var_E = (cry_var_E).wrapping_add(2);
    r18 = (r18).wrapping_add(2);
    r20 = (r20).wrapping_add(2);
    v2 = (v5).wrapping_sub(1);
}
            if cry_var_E >= 0x160 {
                if word!(game.area_index) == 2 {
                    (*E).base.layer = 2;
                }
                (*E).base.properties =
                    ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
                (*E).var_E = 0;
                (*E).var_A = (0xa6a455i32) as u16;
                (*E).var_F = 4;
                QueueMusic_Delayed8(5, game);
            } else {
                (*E).var_E = cry_var_E;
            }
        }
    };
    [10921080] CeresRidley_Func_7 [] {
        if word!(game.area_index) == 2 {
            (*E).var_F = (*E).var_F.wrapping_sub(1);
            let v1: bool = ((*E).var_F & 0x8000) != 0;
            if !v1 {
                return;
            }
            (*E).var_F = 2;
            let v2: u16 = ((*E).var_E).wrapping_add(1);
            (*E).var_E = v2;
            if CeresRidley_Func_8((v2).wrapping_sub(1), game) & 1 == 0 {
                return;
            }
            word! {game; fx_target_y_pos = 440; fx_y_vel = -96i32 as u16; fx_timer = 32; }
        } else {
            (*E).var_F = (*E).var_F.wrapping_sub(1);
            let v1_0: bool = ((*E).var_F & 0x8000) != 0;
            if !v1_0 {
                return;
            }
        }
        (*E).var_E = 0;
        Ridley_Func_99(addr_kRidley_Ilist_E91D as u16, game);
        set_fields! {[(*E)], 8; var_08, var_09}
        Ridley_Func_95(game);
        word!(game.tilemap_stuff[0] = 1);
        (*E).var_A = (0xa6a354i32) as u16;
    };
    [10921647] CeresRidley_Func_9 [] {
        (*E).var_C = ((*E).var_C).wrapping_sub(16);
        if (*E).base.y_pos as i32 - 112i32 & 0x8000 != 0 {
            (*E).var_A = (0xa6a6c8i32) as u16;
            CeresRidley_Func_10(game);
        }
    };
    [10921672] CeresRidley_Func_10 [] {
        (*E).var_C = ((*E).var_C).wrapping_add(20);
        if (*E).base.y_pos as i32 - 80i32 & 0x8000 != 0 {
            (*E).var_A = (0xa6a6e8i32) as u16;
            (*E).var_01 = 1;
        }
    };
    [10921704] CeresRidley_Func_11 [] {
        if (*E).var_0D >= 0x64 {
            (*E).var_01 = 0;
            (*E).var_A = (0xa6bd9ai32) as u16;
            Ridley_Func_44(game);
        } else if word!(game.samus_health) as i32 - 30i32 & 0x8000 != 0 {
            (*E).var_01 = 0;
            (*E).var_A = (0xa6a971i32) as u16;
            CeresRidley_Func_22(game);
        } else if CeresRidley_Func_12(game) & 1 == 0 || {
            let mut v1: u16 = ((*E).var_00).wrapping_add(1);
            (*E).var_00 = v1;
            v1 >= 0x7c
        } {
            (*E).var_A = crate::rtl::rom_table_word!(
                game,
                0xa6a743,
                (word!(game.random_number) as i32 & 0xf) as isize
            );
            (*E).var_00 = 0;
        }
    };
    [10921858] CeresRidley_Func_13 [] {
        let mut v1: u16 = abs16((*E).var_C);
        if v1 < 0x80 {
            v1 = 128;
        }
        (*E).var_C = (if (*E).var_C & 0x8000 != 0 {
            -(v1 as i32)
        } else {
            v1 as i32
        }) as u16;
        Ridley_Func_106(0, 0, (*E).base.x_pos, 88, game);
        if (*E).base.y_pos as i32 - 80i32 & 0x8000 != 0 {
            let v3: u16 = ((*E).var_00).wrapping_add(1);
            (*E).var_00 = v3;
            if v3 >= 0x30 {
                (*E).var_A = (0xa6a88di32) as u16;
            }
        } else if (*E).base.y_pos as i32 - 128i32 & 0x8000 != 0 {
            (*E).var_17 = (*E).base.x_pos;
            (*E).var_18 = (*E).base.y_pos;
            Ridley_Func_99(addr_kRidley_Ilist_E73A as u16, game);
            (*E).var_A = addr_loc_A6A7F9 as u16;
            (*E).var_00 = 224;
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
        if Shitroid_Func_2(0, rect, game) == 0 || {
            let mut v2: bool = (((*E).var_F) as i16).wrapping_sub(1) < 0;
            (*E).var_F = (*E).var_F.wrapping_sub(1);
            v2 as i32 != 0
        } {
            (*E).var_00 = 0;
            (*E).var_A = (0xa6a6e8i32) as u16;
        }
    };
    [10922125] CeresRidley_Func_16 [] {
        (*E).var_A = (0xa6a8a4i32) as u16;
        (*E).var_F = 10;
        (*E).var_0A = 0;
        word!(game.tilemap_stuff[1] = 0);
        CeresRidley_Func_17(game);
    };
    [10922353] CeresRidley_Func_22 [] {
        (*E).var_20 = -192i32 as u16;
        Ridley_Func_106(0, 1, 192, -128i32 as u16, game);
        if (*E).base.y_pos as i32 + 128 & 0x8000 != 0 {
            CeresRidley_A994(game);
        }
    };
    CeresRidley_A994 [] {
        (*E).var_A = (0xa6a9a0i32) as u16;
        (*E).var_F = 64;
        CeresRidley_Func_23(game);
    };
    CeresRidley_Func_28 [] {
        crate::update_place! { previous cry_var_33: u16 = (*E).var_33 => (cry_var_33).wrapping_add(2); }
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
        if crate::rtl::rom_table_word!(game, 0xa6ae4d, v2 as isize) == 0xffff {
            (*E).var_32 = (*E).var_32.wrapping_add(1);
            *ram_ptr!(game; 0x56) = 9;
            *ram_ptr!(game; 0x5f) = 0;
            word! { game, 0;
                reg_M7A, reg_M7B, reg_M7C, reg_M7D, reg_M7X, reg_M7Y, reg_BG1HOFS, reg_BG1VOFS,
            }
            (*E).var_A = (0xa6c04ei32) as u16;
            (*E).var_F = 0;
        } else {
            (*E).var_30 = crate::rtl::rom_table_word!(game, 0xa6ae4d, v2 as isize);
            let v3: u16 =
                (crate::rtl::rom_table_word!(game, 0xa6af2f, v2 as isize)).wrapping_add((*E).var_35);
            (*E).var_35 = v3;
            word!(game.reg_BG1VOFS = v3);
            let v4: u16 =
                ((*E).var_34).wrapping_sub(crate::rtl::rom_table_word!(game, 0xa6b00f, v2 as isize));
            (*E).var_34 = v4;
            word!(game.reg_BG1HOFS = v4);
            CeresRidley_Func_35((*E).var_30, game);
            (*E).var_0A = ((*E).var_0A).wrapping_add(48);
            CeresRidley_Func_29(game);
            CeresRidley_Func_33(game);
            CeresRidley_Func_34(game);
        };
    };
    CeresRidley_Func_29 [] {
        let r18: u16 = byte!((*E).var_0A; 1) as u16;
        word! { game;
            reg_M7A = CeresMult(
                CeresRidley_AC30((r18).wrapping_add(64), 0x100) as u16,
                (*E).var_30,
            );
            reg_M7B = CeresMult(CeresRidley_AC30(r18, 0x100) as u16, (*E).var_30);
            reg_M7C = CeresMult(-(CeresRidley_AC30(r18, 0x100) as i32) as u16, (*E).var_30);
            reg_M7D = CeresMult(
                CeresRidley_AC30((r18).wrapping_add(64), 0x100) as u16,
                (*E).var_30,
            );
        }
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_Ridley(0, game); };
    Ridley_Main [] {
        let mut v1: i16 = (((*E).var_24) as i16).wrapping_sub(4);
        if v1 < 0 {
            v1 = 0;
        }
        (*E).var_24 = v1 as u16;
        Ridley_Func_39(game);
        Ridley_Func_42(game);
        CallRidleyFunc((((*E).var_A) as u32) | (0xa60000u32), game);
        if (*E).var_02 != 0 {
            Ridley_Func_102(game);
            Ridley_Func_112(game);
            Ridley_Func_115(game);
            Ridley_Func_70(game);
            Ridley_Func_129(game);
            sub_A6DB2A(game);
            Ridley_Func_120(game);
            Ridley_Func_118(game);
            if (*E).var_1B != 0 {
                Ridley_Func_30(game);
            }
        }
        Ridley_Func_100(game);
    };
    Ridley_Func_2 [] {
        (*E).var_24 = 0;
        (*E).base.frame_counter = 1;
        Ridley_Hurt(game);
    };
    Ridley_Hurt [] {
        let update_hurt_animation: bool = !(((*E).base.frame_counter & 1) == 0)
            || ({
                Ridley_Func_39(game);
                Ridley_Func_42(game);
                CallRidleyFunc((((*E).var_A) as u32) | (0xa60000u32), game);
                !((*Get_Ridley(0i32, game)).var_02 == 0)
                    && ({
                        Ridley_Func_112(game);
                        Ridley_Func_115(game);
                        Ridley_Func_70(game);
                        true
                    })
            });
        if update_hurt_animation {
            Ridley_Func_129(game);
            Ridley_Func_102(game);
            sub_A6DB2A(game);
            Ridley_Func_120(game);
            Ridley_Func_100(game);
            Ridley_Func_118(game);
            let mut v2: u16 = ((*E).var_24).wrapping_add(1);
            if ((v2) as i16).wrapping_sub((g_word_A6B288) as i16) >= 0 {
                v2 = g_word_A6B288;
            }
            (*E).var_24 = v2;
            if (((*E).var_24) as i16).wrapping_sub((g_word_A6B288) as i16) < 0 {
                return;
            }
        }
        if (*Get_Ridley(0i32, game)).var_1B != 0 {
            Ridley_Func_30(game);
        }
    };
    [10925036] Ridley_Func_5 [] {
        (*E).var_A = (0xa6b3f8i32) as u16;
        (*E).var_F = 128;
        Ridley_Func_6(game);
    };
    [10925121] Ridley_Func_8 [] {
        (*E).var_A = addr_loc_A6B455 as u16;
        (*E).var_00 = 10;
        (*E).var_0A = 0;
        Ridley_B455(game);
    };
    [10925141] Ridley_B455 [] {
        let mut v1: u16 = 192;
        if (*E).var_10 != 0 {
            v1 = 64;
        }
        let rect: Rect16U = Rect16U {
            x: v1,
            y: 128,
            w: 8,
            h: 8,
        };
        Ridley_Func_104_0(0, 1, rect.x, rect.y, game);
        if Shitroid_Func_2(0, rect, game) == 0 {
            (*E).var_A = (0xa6b493i32) as u16;
            (*E).var_00 = 32;
            (*E).var_0A = 0;
        }
    };
    [10925203] Ridley_Func_9 [] {
        if (*E).var_10 != 0 {
            Ridley_Func_111(32, 512, 1152, game);
        } else {
            Ridley_Func_111(-32i32 as u16, -512i32 as u16, 1152, game);
        }
        let ridley_var_00: u16 = (*E).var_00;
        if ridley_var_00 != 0 {
            (*E).var_00 = (ridley_var_00).wrapping_sub(1);
        } else {
            (*E).var_A = (0xa6b4d1i32) as u16;
            (*E).var_00 = 20;
        };
    };
    [10925265] Ridley_Func_10 [] {
        if (*E).var_10 != 0 {
            Ridley_Func_111(320, 0x4000, 1280, game);
        } else {
            Ridley_Func_111(-320i32 as u16, -16384i32 as u16, 1280, game);
        }
        let ridley_var_00: u16 = (*E).var_00;
        if ridley_var_00 != 0 {
            (*E).var_00 = (ridley_var_00).wrapping_sub(1);
        } else {
            (*E).var_A = (0xa6b516i32) as u16;
            (*E).var_00 = 16;
            word!(game.tilemap_stuff[2] = 1);
        };
    };
    [10925334] Ridley_Func_11 [] {
        if (*E).var_10 != 0 {
            Ridley_Func_111(512, 30720, 768, game);
        } else {
            Ridley_Func_111(-512i32 as u16, -30720i32 as u16, 768, game);
        }
        let ridley_var_00: u16 = (*E).var_00;
        if ridley_var_00 != 0 {
            (*E).var_00 = (ridley_var_00).wrapping_sub(1);
        } else {
            (*E).var_A = (0xa6b554i32) as u16;
            (*E).var_00 = 32;
        };
    };
    [10925396] Ridley_Func_12 [] {
        if (*E).var_10 != 0 {
            Ridley_Func_111(1024, 30720, 768, game);
        } else {
            Ridley_Func_111(-1024i32 as u16, -30720i32 as u16, 768, game);
        }
        let ridley_var_00: u16 = (*E).var_00;
        if ridley_var_00 != 0 {
            (*E).var_00 = (ridley_var_00).wrapping_sub(1);
        } else {
            (*E).var_A = (0xa6b594i32) as u16;
            (*E).var_00 = 32;
            Ridley_Func_114(game);
        };
    };
    [10925735] Ridley_Func_19 [] {
        if (*E).base.y_pos as i32 - 288i32 & 0x8000 != 0 {
            Ridley_Func_114(game);
            (*E).var_A = (0xa6b6ddi32) as u16;
            (*E).var_F = 32;
            Ridley_Func_20(game);
        } else {
            Ridley_Func_104_0(
                0,
                0,
                crate::rtl::rom_table_word!(game, 0xa6b6c8, (*E).var_10 as isize),
                288,
                game,
            );
        };
    };
    [10925789] Ridley_Func_20 [] {
        Ridley_Func_104_0(0, 0, (*E).base.x_pos, 288, game);
        (*E).var_F = (*E).var_F.wrapping_sub(1);
        if ((*E).var_F & 0x8000) != 0 {
            Ridley_Func_71(game);
            Ridley_Func_29(game);
            (*E).var_A = (0xa6b70ei32) as u16;
            (*E).var_F = ((word!(game.random_number) as i32 & 0x3f) + 128) as u16;
        }
    };
    [10926009] Ridley_Func_22 [] {
        let mut v1: bool = false;
        if Ridley_Func_41(game) & 1 == 0 || {
            v1 = (((*E).var_F) as i16).wrapping_sub(1) < 0;
            (*E).var_F = (*E).var_F.wrapping_sub(1);
            v1 as i32 != 0
        } {
            Ridley_Func_24(game);
            (*E).var_A = (0xa6b321i32) as u16;
        } else {
            let ridley_var_C: u16 = (*E).var_C;
            v1 = ((word!(game.tilemap_stuff[7])) as i16).wrapping_add((ridley_var_C) as i16) < 0;
            (*E).var_C = (word!(game.tilemap_stuff[7])).wrapping_add(ridley_var_C);
            if !v1 {
                (*E).var_C = 0;
                (*E).var_A = (0xa6b70ei32) as u16;
            }
        };
    };
    Ridley_Func_27 [] {
        let mut ridley_var_C: i16 = (*E).var_C as i16;
        if ridley_var_C >= 0 {
            ridley_var_C = -(ridley_var_C as i32) as i16;
        }
        if ridley_var_C as i32 + 512 & 0x8000 == 0 {
            ridley_var_C = -512i16;
        }
        (*E).var_C = ridley_var_C as u16;
        Ridley_Func_24(game);
        (*E).var_A = (0xa6bb8fi32) as u16;
        Ridley_Func_33b(game);
    };
    Ridley_Func_28 [] {
        if (*E).var_B == 0 {
            (*E).var_B = (if (*E).base.x_pos & 0x80 != 0 {
                -0xc0i32
            } else {
                0xc0
            }) as u16;
        }
        let facing_away =
            (((*E).var_B as i32 ^ (*E).base.x_pos as i32 - word!(game.samus_x_pos) as i32) & 0x8000)
                == 0;
        let x_pos = (*E).base.x_pos;
        if (x_pos as i16).wrapping_sub((*E).var_22 as i16) < 0 {
            if ((*E).var_B & 0x8000) == 0 {
                return;
            }
        } else if (x_pos as i16).wrapping_sub((*E).var_23 as i16) >= 0 {
            if ((*E).var_B & 0x8000) != 0 {
                return;
            }
        } else if facing_away == (word!(game.random_number) < 0x555) {
            return;
        }
        (*E).var_B = -((*E).var_B as i32) as u16;
    };
    Ridley_Func_30 [] {
        let ridley_var_14: u16 = (*E).var_14;
        let mut v3: u16 = 0;
        let mut v6: u16 = 0;
        reduce_ridley_axis_speed!(ridley_var_14, v3, E, var_14);
        let ridley_var_15: u16 = (*E).var_15;
        reduce_ridley_axis_speed!(ridley_var_15, v6, E, var_15);
        word! { game;
            samus_x_pos = ((*E).var_14 as i32
                + (*E).base.x_pos as i32
                + crate::rtl::rom_table_word!(game, 0xa6b9d5, (*E).var_10 as isize) as i32)
                as u16;
            samus_y_pos = ((*E).var_15 as i32
                + (*E).base.y_pos as i32
                + crate::rtl::rom_table_word!(game, 0xa6b9db, ((*E).var_1D >> 1) as isize) as i32)
                as u16;
        }
    };
    Ridley_Func_31 [] {
        (*E).var_14 = (word!(game.samus_x_pos) as i32
            - ((*E).base.x_pos as i32
                + crate::rtl::rom_table_word!(game, 0xa6b9d5, (*E).var_10 as isize) as i32))
            as u16;
        (*E).var_15 = (word!(game.samus_y_pos) as i32
            - ((*E).base.y_pos as i32
                + crate::rtl::rom_table_word!(game, 0xa6b9db, ((*E).var_1D >> 1) as isize) as i32))
            as u16;
    };
    Ridley_Func_32 [] {
        if (*E).var_25 as i32 - 10i32 & 0x8000 != 0 {
            if word!(game.power_bomb_flag) != 0 {
                (*E).var_A = (0xa6bd4ei32) as u16;
                Ridley_Func_43(game);
            } else {
                (*E).var_A = (0xa6b3eci32) as u16;
                word!(game.tilemap_stuff[2] = 1);
            }
        } else {
            Ridley_Func_126(game);
            (*E).var_A = (0xa6c53ei32) as u16;
            Ridley_C53E(game);
        };
    };
    [10926775] Ridley_Func_33 [] {
        if (Ridley_Func_40_Carry(game) & 1) != 0
            && (*E).var_1F == 0
            && {
                let mut r18: u16 = crate::rtl::rom_table_word!(game, 0xa6bb48, (*E).var_10 as isize);
                let mut r20: u16 = ((*E).base.x_pos).wrapping_sub(word!(game.samus_x_pos));
                ((r18 ^ r20) & 0x8000) == 0 || {
                    let mut v1: u16 = abs16(r20);
                    v1 as i32 - 32i32 & 0x8000 != 0
                }
            }
            && (((*E).base.y_pos as i32 + 35 - word!(game.samus_y_pos) as i32) as i16) < 0
        {
            let mut v2: i16 = -16i16;
            if (*E).var_10 != 0 {
                v2 = 16;
            }
            Ridley_Func_104_0(
                0,
                crate::rtl::rom_table_word!(game, 0xa6bb4e, (*E).var_12 as isize),
                (word!(game.samus_x_pos)).wrapping_add((v2) as u16),
                (word!(game.samus_y_pos)).wrapping_sub(4),
                game,
            );
            let rect: Rect16U = Rect16U {
                x: (((*E).base.x_pos).wrapping_add(crate::rtl::rom_table_word!(
                    game,
                    0xa6b9d5,
                    (*E).var_10 as isize
                ))),
                y: (((*E).base.y_pos).wrapping_add(35)),
                w: 8,
                h: 12,
            };
            if Ridley_Func_124(rect, game) != 0 {
                (*E).var_C = -((*E).var_C as i32) as u16;
                if (*E).base.health != 0 {
                    if word!(game.power_bomb_flag) != 0 {
                        Ridley_Func_37(game);
                        (*E).var_A = (0xa6bd4ei32) as u16;
                        Ridley_Func_43(game);
                    } else {
                        Ridley_Func_33b(game);
                    }
                } else {
                    if (*E).var_1B == 0 {
                        Ridley_Func_37(game);
                    }
                    Ridley_Func_126(game);
                    (*E).var_A = (0xa6c538i32) as u16;
                    Ridley_Func_63(game);
                }
            }
        } else {
            Ridley_Func_32(game);
        };
    };
    [10926991] Ridley_Func_33b [] {
        (*E).var_17 = g_word_A6BBEB[(*E).var_10 as usize];
        let mut v1: i16 = (((*E).base.y_pos) as i16).wrapping_sub(64);
        if (*E).base.y_pos as i32 - 320i32 & 0x8000 != 0 {
            v1 = 256;
        }
        (*E).var_18 = v1 as u16;
        if (*E).var_1B == 0 {
            Ridley_Func_37(game);
        }
        (*E).var_A = (0xa6bbc4i32) as u16;
        (*E).var_F = 32;
        Ridley_Func_34(game);
    };
    [10927044] Ridley_Func_34 [] {
        Ridley_Func_104_0(0, 0, (*E).var_17, (*E).var_18, game);
        (*E).var_F = (*E).var_F.wrapping_sub(1);
        if ((*E).var_F & 0x8000) != 0 {
            (*E).var_A = (0xa6bbf1i32) as u16;
            (*E).var_F = 32;
        }
    };
    [10927089] Ridley_Func_35 [] {
        (*E).var_F = (*E).var_F.wrapping_sub(1);
        if ((*E).var_F & 0x8000) != 0 {
            word! {game; tilemap_stuff[15] = 8; tilemap_stuff[9] = 240; }
            Ridley_Func_38(game);
            (*E).var_A = (0xa6bc2ei32) as u16;
            (*E).var_F = 64;
        } else {
            Ridley_Func_104_0(0, 0, (*E).var_17, 256, game);
        };
    };
    [10927150] Ridley_Func_36 [] {
        (*E).var_F = (*E).var_F.wrapping_sub(1);
        if ((*E).var_F & 0x8000) != 0 {
            word! {game; tilemap_stuff[15] = 16; tilemap_stuff[9] = 240; }
            (*E).var_A = (0xa6b321i32) as u16;
        } else {
            Ridley_Func_104_0(0, 0, g_word_A6BC62[(*E).var_10 as usize], 224, game);
        };
    };
    Ridley_Func_39 [] {
        let mut ridley_var_01: i16 = (*E).var_01 as i16;
        if ridley_var_01 >= 0 {
            if ridley_var_01 != 0 {
                if Ridley_Func_121(game) & 1 != 0 {
                    (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
                    return;
                }
                (*E).base.properties =
                    ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
            }
            let ridley_var_1E: u16 = (*E).var_1E;
            if ridley_var_1E != 0 {
                let v3: u16 = (ridley_var_1E).wrapping_sub(1);
                (*E).var_1E = v3;
                if v3 == 0 && ((*E).var_01 & 0x8000) == 0 {
                    (*E).base.properties =
                        ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
                }
            }
        }
    };
    Ridley_Func_42 [] {
        let mut ridley_var_01: i16 = (*E).var_01 as i16;
        if ridley_var_01 > 0
            && ridley_var_01 != 2
            && word!(game.power_bomb_flag) != 0
            && (*E).var_1B == 0
        {
            Ridley_Func_24(game);
            (*E).var_A = (0xa6bab7i32) as u16;
        }
    };
    [10927548] Ridley_Func_45 [] {
        (*E).var_20 = -192i32 as u16;
        Ridley_Func_106(0, 1, 192, -128i32 as u16, game);
        if (*E).base.y_pos as i32 - 32i32 & 0x8000 != 0 {
            (*E).var_40 = (0xa6becai32) as u16;
            (*E).var_A = (0xa6bdf2i32) as u16;
            (*E).var_F = 21;
        }
    };
    [10927619] Ridley_Func_47 [] {
        Ridley_Func_104_0(
            0,
            12,
            ((*E).var_42).wrapping_sub(10),
            ((*E).var_44).wrapping_sub(56),
            game,
        );
        let rect: Rect16U = Rect16U {
            x: (((*E).base.x_pos).wrapping_add(14)),
            y: (((*E).base.y_pos).wrapping_add(66)),
            w: 4,
            h: 4,
        };
        if Ridley_Func_48(rect, game) & 1 != 0 {
            (*E).var_1B = 1;
            Ridley_Func_119(1, game);
            (*E).var_C = -512i32 as u16;
            (*E).var_40 = (0xa6beb3i32) as u16;
            (*E).var_A = (0xa6a971i32) as u16;
        }
    };
    Ridley_Func_50 [] {
        (*E).var_42 = ((*E).base.x_pos).wrapping_sub(16);
        (*E).var_44 = ((*E).base.y_pos).wrapping_add(22);
    };
    Ridley_Func_51 [] {
        (*E).var_42 = ((*E).base.x_pos).wrapping_add(14);
        (*E).var_44 = ((*E).base.y_pos).wrapping_add(66);
    };
    Ridley_Func_52 [] {
        set_fields! {[(*E)], 0; var_45, var_46}
        (*E).var_40 = (0xa6bedci32) as u16;
        Ridley_Func_53(game);
    };
    Ridley_Func_53 [] {
        let v1: u16 = ((*E).var_46).wrapping_add(8);
        (*E).var_46 = v1;
        let t: i32 = byte!((*E).var_43; 1) as i32 + (v1 as i32 & 0xff);
        byte!((*E).var_43; 1) = t as u8;
        (*E).var_44 = ((*E).var_44 as i32 + ((v1 >> 8) as i8 as i32 + (t >> 8))) as u16;
        if (*E).var_44 as i32 - 192i32 & 0x8000 == 0 {
            (*E).var_44 = 192;
            (*E).var_40 = addr_locret_A6BF19 as u16;
        }
    };
    Ridley_Func_57 [] {
        crate::update_place! { previous fresh15 = (*E).var_E => (*E).var_E.wrapping_sub(1); }
        if fresh15 == 1 {
            (*E).var_F = (*E).var_F.wrapping_add(1);
            (*E).var_F = (*E).var_F.wrapping_add(1);
            Ridley_Func_62(game);
        }
        Ridley_C104(game);
    };
    Ridley_Func_69 [] {
        let mut v1: i16 = (((*E).var_27) as i16).wrapping_sub(1);
        if v1 < 0 {
            (*E).var_27 = 4;
            let mut v2: u16 = ((*E).var_28).wrapping_add(1);
            if (*E).var_28 as i32 - 9i32 & 0x8000 == 0 {
                v2 = 0;
            }
            (*E).var_28 = v2;
            let v3: i32 = ((4u16).wrapping_mul(v2)) as i32 >> 1;
            game.eproj_spawn_pt = Point16U {
                x: (((*E).base.x_pos).wrapping_add((g_word_A6C66E[v3 as usize]) as u16)),
                y: (((*E).base.y_pos).wrapping_add((g_word_A6C66E[(v3 + 1) as usize]) as u16)),
            };
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 3, game);
            QueueSfx2_Max3(0x24, game);
        } else {
            (*E).var_27 = v1 as u16;
        };
    };
    Ridley_Func_83 [] {
        Ridley_Func_77(game);
        if Ridley_Func_78(game) & 1 != 0
            && ((word!(game.random_number) as u8) >= 0xf0
                || abs16((word!(game.samus_x_pos)).wrapping_sub((*E).base.x_pos)) < 0x80)
            && (word!(game.tilemap_stuff[13]) & word!(game.tilemap_stuff[14]) & 0x8000) != 0
        {
            word! {game; tilemap_stuff[13] = 16128; tilemap_stuff[10] = 8; }
        } else if word!(game.tilemap_stuff[76])
            | (word!(game.tilemap_stuff[66])
                | word!(game.tilemap_stuff[56])
                | word!(game.tilemap_stuff[46])
                | word!(game.tilemap_stuff[36])
                | word!(game.tilemap_stuff[26])
                | word!(game.tilemap_stuff[16]))
            == 0
        {
            if ((*Get_Ridley(0i32, game)).var_C & 0x8000) == 0 {
                word!(game.tilemap_stuff[0] = 5);
            }
            word! {game; tilemap_stuff[13] = -1i32 as u16; tilemap_stuff[14] = -1i32 as u16; }
            let mut v1: u16 = word!(game.tilemap_stuff[8]);
            if word!(game.tilemap_stuff[8]) == 0
                || {
                    v1 = (word!(game.tilemap_stuff[8])).wrapping_sub(1);
                    crate::update_place! { unaligned_word fresh14 = &raw mut (*(ram_ptr!(game, u16; 0x2000)).offset(8)) => v1; }
                    u16::from_le(fresh14.read_unaligned()) != 0
                }
            {
                word!(game.tilemap_stuff[10] = v1);
            } else {
                word!(game.tilemap_stuff[16] = 0x8000);
                Ridley_Func_85(game);
                word!(game.tilemap_stuff[10] = 8);
            }
        }
    };
    Ridley_Func_84 [] {
        Ridley_Func_77(game);
        if Ridley_Func_78(game) & 1 != 0
            && ((word!(game.random_number) as u8) >= 0xf0
                || abs16((word!(game.samus_x_pos)).wrapping_sub((*E).base.x_pos)) < 0x80)
            && (word!(game.tilemap_stuff[13]) & word!(game.tilemap_stuff[14]) & 0x8000) != 0
        {
            word! {game; tilemap_stuff[13] = 16128; tilemap_stuff[10] = 8; }
        } else if word!(game.tilemap_stuff[76])
            | (word!(game.tilemap_stuff[66])
                | word!(game.tilemap_stuff[56])
                | word!(game.tilemap_stuff[46])
                | word!(game.tilemap_stuff[36])
                | word!(game.tilemap_stuff[26])
                | word!(game.tilemap_stuff[16]))
            == 0
        {
            if ((*E).var_C & 0x8000) == 0 {
                word!(game.tilemap_stuff[0] = 6);
                word! { game, 2560;
                    tilemap_stuff[20], tilemap_stuff[30], tilemap_stuff[40], tilemap_stuff[50],
                    tilemap_stuff[60], tilemap_stuff[70], tilemap_stuff[80],
                }
                word! { game, 0x4000;
                    tilemap_stuff[21], tilemap_stuff[31], tilemap_stuff[41], tilemap_stuff[51],
                    tilemap_stuff[61], tilemap_stuff[71], tilemap_stuff[81],
                }
                word!(game.tilemap_stuff[16] = 0x8000);
            }
            word! {game; tilemap_stuff[13] = -1i32 as u16; tilemap_stuff[14] = -1i32 as u16; }
            let mut v1: u16 = word!(game.tilemap_stuff[8]);
            if word!(game.tilemap_stuff[8]) != 0 {
                v1 = (word!(game.tilemap_stuff[8])).wrapping_sub(1);
                word!(game.tilemap_stuff[8] = v1);
                if v1 == 0 {
                    word!(game.tilemap_stuff[16] = 0x8000);
                    Ridley_Func_85(game);
                    v1 = 8;
                }
            }
            word!(game.tilemap_stuff[10] = v1);
        }
    };
    Ridley_Func_86 [] {
        word!(
            game.tilemap_stuff[23] =
                (word!(game.tilemap_stuff[25]) as i32 + (*E).base.y_pos as i32 + 16) as u16
        );
        let ram = (&raw mut game.g_ram).cast::<u8>();
        ridley_tail_chain(ram, 23);
        let mut v1: i16 = (((*E).var_10) as i16).wrapping_sub(1);
        if (*E).var_10 != 1 {
            v1 = ((word!(game.tilemap_stuff[24])) as i16)
                .wrapping_add(g_word_A6CF54[(*E).var_10 as usize]);
        }
        word!(game.tilemap_stuff[22] = ((*E).base.x_pos).wrapping_add((v1) as u16));
        if (*E).var_10 == 1 {
            word!(game.tilemap_stuff[32] = (*E).base.x_pos);
            for index in [42, 52, 62, 72, 82] {
                let value = ridley_tail_word(ram, 32);
                ridley_tail_store(ram, index, value);
            }
        } else {
            ridley_tail_chain(ram, 22);
        };
    };
    Ridley_Func_91 [] {
        if word!(game.tilemap_stuff[10]) as i32 - 8i32 & 0x8000 == 0 {
            let v1: u16 = ((*E).var_0E).wrapping_add(1);
            (*E).var_0E = v1;
            if v1 as i32 - 16i32 & 0x8000 != 0 {
                return;
            }
            if (*E).var_0F == 0 {
                QueueSfx3_Max6(0x21, game);
            }
        }
        (*E).var_0E = 0;
    };
    Ridley_Func_101 [] {
        if (*E).var_01 != 0 && (*E).var_0D >= 0x32 {
            Ridley_D495((if (*E).var_0D >= 0x46 { 2 } else { 0 }) as u16, game);
        }
    };
    Ridley_Func_114 [] {
        let ridley_var_10: u16 = (*E).var_10;
        let mut v2: u16 = 0;
        let change_instruction: bool = if ridley_var_10 == 0 {
            if *((&raw mut (*E).base.enemy_ptr as *mut u8).offset(1) as *mut i16) < 0 {
                return;
            }
            v2 = addr_kRidley_Ilist_E6F0 as u16;
            true
        } else {
            (ridley_var_10 != 1
                && *((&raw mut (*E).base.enemy_ptr as *mut u8).offset(1) as *mut i16) < 0)
                && ({
                    v2 = addr_kRidley_Ilist_E706 as u16;
                    true
                })
        };
        if change_instruction {
            enemy_instruction! {[(*E).base]; instruction = v2, timer = 2}
            (*E).base.timer = 0;
        };
    };
    Ridley_Func_116 [] {
        let r18: u16 = abs16((*E).var_B);
        let v1: u16 = (r18).wrapping_add(abs16((*E).var_C));
        if v1 != 0 {
            let mut v2: u16 = (v1).wrapping_sub(r18);
            if v2 as i32 - r18 as i32 & 0x8000 != 0 {
                v2 = r18;
            }
            let mut v5: i32 = (4 * v2 as i32 & 0xf00) >> 8;
            if v5 >= 7 {
                v5 = 7;
            }
            let mut v6: u16 = g_word_A6D9ED[v5 as usize];
            if ((*E).var_C & 0x8000) == 0 {
                v6 = v6 >> 1;
            }
            (*E).var_08 = v6;
        } else {
            (*E).var_08 = 0;
        };
    };
    Ridley_Func_117 [] {
        (*E).var_05 = addr_word_A6DA71 as u16;
        (*E).var_06 = 1;
    };
    Ridley_Func_118 [] {
        let v1: u16 = ((*E).var_06).wrapping_sub(1);
        (*E).var_06 = v1;
        if v1 == 0 {
            let mut v3: u16 = 0;
            let mut v4 = null_mut::<u16>();
            let mut i: i32 = (*E).var_05 as i32;
            loop {
                v3 = i as u16;
                v4 = RomPtr_A6(i as u16, game) as *mut u16;
                i = *v4 as i32;
                if (*v4 & 0x8000) == 0 {
                    break;
                }
                (*E).var_05 = i as u16;
            }
            (*E).var_06 = i as u16;
            let v6: u16 = word!(game.vram_write_queue_tail);
            let mut v7: *mut VramWriteEntry =
                gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
            *((&raw mut (*v7).src.addr as *mut u8).offset(1) as *mut VoidP) = -20480i32 as VoidP;
            *((&raw mut (*v7.offset(1)).src.addr as *mut u8).offset(1) as *mut VoidP) =
                -20480i32 as VoidP;
            (*v7).src.addr = *v4.offset(1);
            (*v7.offset(1)).src.addr = *v4.offset(2);
            (*v7).vram_dst = 29216;
            (*v7.offset(1)).vram_dst = 29472;
            (*v7).size = 64;
            (*v7.offset(1)).size = 64;
            word!(game.vram_write_queue_tail = (v6).wrapping_add(14));
            (*gVramWriteEntry(v6 as i32 + 14i32, game)).size = 0;
            (*E).var_05 = (v3).wrapping_add(6);
        }
    };
    Ridley_Func_120 [] {
        let mut ridley_var_10: i16 = (*E).var_10 as i16;
        if ridley_var_10 != 0 {
            if ridley_var_10 == 1 {
                return;
            }
            ridley_var_10 = 10;
        }
        let v2: u16 = crate::rtl::rom_table_word!(
            game,
            0xa6db02,
            ((*E).var_07 as i32 + ridley_var_10 as i32) as isize
        );
        sub_A6DC13(v2, (*E).base.x_pos, (*E).base.y_pos, (*E).var_0C, game);
    };
    Ridley_Func_126 [] {
        if (*E).base.health == 0 && ((*E).var_01 & 0x8000) == 0 {
            (*E).var_01 = -1i32 as u16;
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
            (*E).var_A = (0xa6c538i32) as u16;
        }
    };
    @countdown [pub unsafe fn (game: &mut Game)];
    before {
        let E = Get_CeresRidley(0, game);
        (*E).var_F = (*E).var_F.wrapping_sub(1);
    }
    when ((*E).var_F & 0x8000) != 0;
    [10920823] CeresRidley_A377 {
        (*E).var_A = (0xa6a389i32) as u16;
        set_fields! {[(*E)], 0; var_E, var_F}
    };
    [10921045] CeresRidley_Func_6 {
        Ridley_Func_99(addr_kRidley_Ilist_E690 as u16, game);
        (*E).var_F = 0;
        (*E).var_A = (0xa6a478i32) as u16;
        if word!(game.area_index) != 2 {
            (*E).var_F = 252;
        }
    };
    [10922400] CeresRidley_Func_23 {
        CeresRidley_Func_25(game);
        set_fields! {[(*E)], 0; var_B, var_C}
        word!(game.tilemap_stuff[0] = 0);
        (*E).var_A = (0xa6aa11i32) as u16;
        WriteColorsToPalette(0xa2, 0xa6, addr_word_A6A9E3 as u16, 0xf, game);
        WriteColorsToPalette(0x42, 0xa6, addr_word_A6AA01 as u16, 8, game);
        WriteColorsToPalette(0x1e2, 0xa6, addr_word_A6AA01 as u16, 8, game);
    };
    @countdown [pub unsafe fn (game: &mut Game)];
    before {
        let E = Get_Ridley(0, game);
        (*E).var_F = (*E).var_F.wrapping_sub(1);
    }
    when ((*E).var_F & 0x8000) != 0;
    [10927602] Ridley_Func_46 {
        Ridley_Func_99(addr_kRidley_Ilist_E658 as u16, game);
        (*E).var_A = (0xa6be03i32) as u16;
        Ridley_Func_47(game);
    };
    [10929608] Ridley_Func_66 {
        (*E).var_A = (0xa6c5dai32) as u16;
        (*E).var_F = 256;
    };
    [10929626] Ridley_Func_67 {
        let k: u16 = 0;
        printf(b"Warning: X undefined\n\0" as *const u8 as *const c_char);
        SetBossBitForCurArea(1, game);
        scatter(DropSource::Ridley, k, game);
        QueueMusic_Delayed8(3, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        (*E).var_A = addr_locret_A6C600 as u16;
    };
    @external {
        10922575 => [()];
        10929664 => [()];
    }
}
pub unsafe fn Ridley_Func_37(game: &mut Game) {
    unsafe {
        Ridley_Func_31(game);
        let E = Get_Ridley(0, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        (*E).var_1B = 1;
        CallSomeSamusCode(0, game);
        Ridley_Func_119(1, game);
    }
}
pub unsafe fn Ridley_Func_38(game: &mut Game) {
    unsafe {
        word! { game, 1; tilemap_stuff[2], tilemap_stuff[0] }
        let E = Get_Ridley(0, game);
        if ((*E).var_01 & 0x8000) == 0 {
            (*E).var_1E = (if Ridley_Func_40_Sign(game) != 0 {
                6
            } else {
                10
            }) as u16;
        }
        (*E).var_1B = 0;
        CallSomeSamusCode(1, game);
        Ridley_Func_119(0, game);
    }
}
static byte_A6BD04: [u8; 28] = [
    0x80, 0x80, 0x80, 0, 0xff, 0x80, 0x80, 0xff, 0xff, 0xff, 0x80, 0, 0, 0x80, 0x80, 0x80, 0x80,
    0xff, 0xff, 0xff, 0x80, 0x80, 0, 0x80, 0x80, 0, 0, 0x80,
];
unsafe fn Ridley_Func_40_Carry(game: &mut Game) -> u8 {
    unsafe { ((byte_A6BD04[*ram_ptr!(game; 0xa1f) as usize] & 0x80) != 0) as u8 }
}
unsafe fn Ridley_Func_40_Sign(game: &mut Game) -> u8 {
    unsafe { ((byte_A6BD04[*ram_ptr!(game; 0xa1f) as usize] & 0x40) != 0) as u8 }
}
pub fn Ridley_Func_41(game: &mut Game) -> u8 {
    ((word!(game.samus_y_pos) as i32 - 352i32 & 0x8000) == 0) as u8
}
pub unsafe fn Ridley_Func_48(rect: Rect16U, game: &mut Game) -> u8 {
    unsafe {
        let E = Get_Ridley(0, game);
        let v1: u16 = abs16(((*E).var_42).wrapping_sub(rect.x));
        let mut v2: bool = v1 < 4;
        let v3: u16 = (v1).wrapping_sub(4);
        let result: u8 = 0;
        if v2 || v3 < rect.w {
            let v4: u16 = abs16(((*E).var_44).wrapping_sub(rect.y));
            v2 = v4 < 4;
            let v5: u16 = (v4).wrapping_sub(4);
            if v2 || v5 < rect.h {
                return 1;
            }
        }
        result
    }
}
crate::game::address_dispatch! {
    pub unsafe fn CallRidleyFunc_var40(ea: u32, game: &mut Game) => ea;
        call (game); default { Unreachable(game); };
    {
        10927772 => Ridley_Func_50; 10927795 => Ridley_Func_51; 10927818 => Ridley_Func_52;
        10927836 => Ridley_Func_53; 10927897 => [()];
    }
}
pub unsafe fn Ridley_Func_49(game: &mut Game) {
    unsafe {
        CallRidleyFunc_var40(
            (((*Get_Ridley(0i32, game)).var_40) as u32) | (0xa60000u32),
            game,
        );
    }
}
pub unsafe fn DrawBabyMetroid_0(game: &mut Game) {
    unsafe {
        let v0: i32 = BabyMetroid_DBCB_DoubleRetEx(
            ((ram_ptr!(game, ExtraEnemyRam7800; 0x7800) as *mut u8)
                .offset_from(&raw mut game.g_ram as *mut u8) as c_long
                + 6 as c_long) as u16,
            game,
        );
        if v0 < 0 {
            return;
        }
        let E = Get_Ridley(0, game);
        sub_A6DC13(v0 as u16, (*E).var_42, (*E).var_44, 0, game);
    }
}
pub unsafe fn Ridley_Func_55(game: &mut Game) {
    unsafe {
        word!(game.palette_buffer[97] = word!(game.palette_buffer[1]));
        word!(game.palette_buffer[99] = word!(game.palette_buffer[3]));
        word!(game.palette_buffer[81] = word!(game.palette_buffer[17]));
        word!(game.palette_buffer[83] = word!(game.palette_buffer[19]));
        let E = Get_Ridley(0, game);
        (*E).var_E = -15157i32 as u16;
        (*E).var_F = (*E).var_F.wrapping_add(1);
        (*E).var_F = (*E).var_F.wrapping_add(1);
        Ridley_C08E(game);
    }
}
pub unsafe fn Ridley_C08E(game: &mut Game) {
    unsafe {
        if ProcessEscapeTimerTileTransfers(game) & 1 != 0 {
            let E = Get_Ridley(0, game);
            (*E).var_E = -15106i32 as u16;
            (*E).var_F = (*E).var_F.wrapping_add(1);
            (*E).var_F = (*E).var_F.wrapping_add(1);
            Ridley_C09F(game);
        }
    }
}
pub unsafe fn Ridley_C09F(game: &mut Game) {
    unsafe {
        if ProcessEscapeTimerTileTransfers(game) & 1 != 0 {
            let E = Get_Ridley(0, game);
            (*E).var_F = (*E).var_F.wrapping_add(1);
            (*E).var_F = (*E).var_F.wrapping_add(1);
            Ridley_Func_59(game);
            (*E).var_E = 128;
            QueueMusic_Delayed8(7, game);
        }
    }
}
pub unsafe fn Ridley_Func_56(game: &mut Game) {
    unsafe {
        Ridley_Func_61(game);
        let E = Get_Ridley(0, game);
        crate::update_place! { previous fresh16 = (*E).var_E => (*E).var_E.wrapping_sub(1); }
        if fresh16 == 1 {
            (*E).var_3B = -15280i32 as u16;
            set_fields! {[(*E)], 0; var_3C, var_3D, var_3E, var_3F}
            (*E).var_E = 32;
            if word!(game.japanese_text_flag) == 0 {
                (*E).var_F = (*E).var_F.wrapping_add(1);
                (*E).var_F = (*E).var_F.wrapping_add(1);
            }
            (*E).var_F = (*E).var_F.wrapping_add(1);
            (*E).var_F = (*E).var_F.wrapping_add(1);
        }
    }
}
pub unsafe fn Ridley_C104(game: &mut Game) {
    unsafe {
        Ridley_Func_61(game);
        if HandleTypewriterText_Ext(0x3582, game) & 1 != 0 {
            let E = Get_Ridley(0, game);
            (*E).var_F = (*E).var_F.wrapping_add(1);
            (*E).var_F = (*E).var_F.wrapping_add(1);
        }
    }
}
pub unsafe fn Ridley_Func_58(game: &mut Game) {
    unsafe {
        Ridley_Func_61(game);
        CeresRidley_Func_2(game);
        let E = Get_Ridley(0, game);
        (*E).var_F = 0;
        (*E).var_A = (0xa6aa50i32) as u16;
        word! {game; timer_status = 1; ceres_status = 2; }
        SetBossBitForCurArea(1, game);
    }
}
pub unsafe fn Ridley_Func_59(game: &mut Game) {
    unsafe {
        let v0: u16 = word!(game.vram_write_queue_tail);
        let v1 = RomPtr_A6(addr_byte_A6C15D as u16, game);
        let mut v2 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v2).size = *(v1 as *mut u16);
        *((&raw mut (*v2).src.addr as *mut u8).offset(1) as *mut VoidP) =
            *(v1.offset(3) as *mut u16);
        (*v2).src.addr = *(v1.offset(2) as *mut u16);
        (*v2).vram_dst = *(v1.offset(5) as *mut u16);
        word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
    }
}
pub unsafe fn Ridley_Func_61(game: &mut Game) {
    unsafe {
        if word!(game.door_transition_flag_enemies) | word!(game.palette_change_num) == 0
            && (word!(game.nmi_frame_counter_word) & 3) == 0
        {
            let E = Get_Ridley(0, game);
            let mut v1: u16 = ((*E).var_38).wrapping_add(1);
            if v1 >= 0x10 {
                v1 = 0;
            }
            (*E).var_38 = v1;
            let v2: i32 = (*E).var_38 as i32 + 2 * v1 as i32;
            word! { game;
                palette_buffer[97] = crate::rtl::rom_table_word!(game, 0xa6c1df, v2 as isize);
                palette_buffer[98] = crate::rtl::rom_table_word!(game, 0xa6c1df, (v2 + 1) as isize);
                palette_buffer[99] = crate::rtl::rom_table_word!(game, 0xa6c1df, (v2 + 2) as isize);
            }
        }
    }
}
pub unsafe fn SetupZebesEscapeTypewriter(game: &mut Game) {
    unsafe {
        word!(game.palette_buffer[157] = word!(game.palette_buffer[125]));
        word!(game.palette_buffer[158] = word!(game.palette_buffer[126]));
        let E = Get_MotherBrain(0, game);
        (*E).var_3B = addr_loc_A9C49C as u16;
        set_fields! {[(*E)], 0; var_3C, var_3D, var_3E, var_3F}
        word! { game, 0; reg_BG2HOFS, reg_BG2VOFS }
    }
}
pub unsafe fn ProcessEscapeTimerTileTransfers(game: &mut Game) -> u8 {
    unsafe {
        let v0 = gEnemyData(0, game);
        let ai_var_E: u16 = (*v0).ai_var_E;
        let v2: u16 = word!(game.vram_write_queue_tail);
        let v3 = RomPtr_A6(ai_var_E, game);
        let result: u8 = 1;
        if *(v3 as *mut u16) != 0 {
            let mut v4: *mut VramWriteEntry =
                gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
            (*v4).size = *(v3 as *mut u16);
            *((&raw mut (*v4).src.addr as *mut u8).offset(1) as *mut VoidP) =
                *(v3.offset(3) as *mut u16);
            (*v4).src.addr = *(v3.offset(2) as *mut u16);
            (*v4).vram_dst = *(v3.offset(5) as *mut u16);
            word!(game.vram_write_queue_tail = (v2).wrapping_add(7));
            (*v0).ai_var_E = (ai_var_E).wrapping_add(7);
            if *(RomPtr_A6((*v0).ai_var_E, game) as *mut u16) != 0 {
                return 0;
            }
        }
        result
    }
}
pub unsafe fn HandleTypewriterText_Ext(a: u16, game: &mut Game) -> u8 {
    unsafe {
        let r18: u16 = a;
        let mut v4: i16 = 0;
        let E = Get_MotherBrain(0, game);
        let mbn_var_3D: u16 = (*E).var_3D;
        if mbn_var_3D != 0 {
            (*E).var_3D = (mbn_var_3D).wrapping_sub(1);
            0
        } else {
            (*E).var_3D = (*E).var_3E;
            let mut i: i32 = (*E).var_3B as i32;
            loop {
                loop {
                    v4 = *(RomPtr_A6(i as u16, game) as *mut u16) as i16;
                    if v4 == 0 {
                        return 1;
                    }
                    if v4 != 1 {
                        break;
                    }
                    let v5: u16 = (i + 2) as u16;
                    let v6: u16 = *(RomPtr_A6(v5, game) as *mut u16);
                    (*E).var_3E = v6;
                    i = v5 as i32 + 2;
                }
                if v4 != 13 {
                    break;
                }
                let mut v7: i32 = i + 2;
                let v8: u16 = *(RomPtr_A6(v7 as u16, game) as *mut u16);
                (*E).var_3C = v8;
                i = v7 + 2;
            }
            v4 = v4 as u8 as i16;
            if (v4 as u8) == 32 {
                (*E).var_3C = (*E).var_3C.wrapping_add(1);
                (*E).var_3B = (i + 1) as u16;
                0
            } else {
                if (v4 as u8) == 33 {
                    v4 = 91;
                }
                (*E).var_3B = (i + 1) as u16;
                let v11: u16 = word!(game.vram_write_queue_tail);
                let mut v12: *mut VramWriteEntry =
                    gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
                (*v12).size = 2;
                *((&raw mut (*v12).src.addr as *mut u8).offset(1) as *mut VoidP) = 32256 as VoidP;
                (*E).var_3A = (r18 as i32 + v4 as i32 - 65i32) as u16;
                (*v12).src.addr = ((ram_ptr!(game, ExtraEnemyRam8000; 0x8000) as *mut u8)
                    .offset_from(&raw mut game.g_ram as *mut u8)
                    as c_long
                    + 52 as c_long) as VoidP;
                let mbn_var_3C: u16 = (*E).var_3C;
                (*v12).vram_dst = mbn_var_3C;
                (*E).var_3C = (mbn_var_3C).wrapping_add(1);
                word!(game.vram_write_queue_tail = (v11).wrapping_add(7));
                let v14: u16 = ((*E).var_3F).wrapping_add(1);
                (*E).var_3F = v14;
                if v14 as i32 - 2i32 & 0x8000 == 0 {
                    (*E).var_3F = 0;
                    if word!(game.area_index) == 6 {
                        QueueSfx2_Max3(0x45, game);
                    } else {
                        QueueSfx3_Max3(0xd, game);
                    }
                }
                0
            }
        }
    }
}
pub unsafe fn Ridley_Func_62(game: &mut Game) {
    unsafe {
        let mut v0: u16 = addr_stru_A6C3B8 as u16;
        let mut i: i32 = word!(game.vram_write_queue_tail) as i32;
        loop {
            let v2 = RomPtr_A6(v0, game);
            if *(v2 as *mut u16) == 0 {
                break;
            }
            let mut v3: *mut VramWriteEntry = gVramWriteEntry(i, game);
            (*v3).size = *(v2 as *mut u16);
            *((&raw mut (*v3).src.addr as *mut u8).offset(1) as *mut VoidP) =
                *(v2.offset(3) as *mut u16);
            (*v3).src.addr = *(v2.offset(2) as *mut u16);
            (*v3).vram_dst = *(v2.offset(5) as *mut u16);
            v0 = (v0).wrapping_add(7);
            i += 7;
        }
        word!(game.vram_write_queue_tail = i as u16);
    }
}
pub unsafe fn Ridley_Func_68(game: &mut Game) -> u8 {
    unsafe {
        let rect: Rect16U = Rect16U {
            x: 128,
            y: 328,
            w: 4,
            h: 4,
        };
        Ridley_Func_104(0, 0, 0x10, rect.x, rect.y, game);
        Shitroid_Func_2(0, rect, game)
    }
}

macro_rules! place_ridley_explosion_x {
    (
        $ridley:ident, $game:ident, $orientation_index:ident, $x_offset_table:literal,
        $explosion:ident $(,)?
    ) => {
        let mut v10: u16 = 0;
        let $ridley = Get_Ridley(0, $game);
        if (*$ridley).var_10 != 0 {
            v10 = 2;
        }
        let $orientation_index: i32 = v10 as i32 >> 1;
        let v14: u16 = ((*$ridley).base.x_pos).wrapping_add(crate::rtl::rom_table_word!(
            $game,
            $x_offset_table,
            $orientation_index as isize
        ));
        (*$explosion).base.x_pos = v14;
    };
}

pub unsafe fn RidleysExplosion_Func_1(k: u16, game: &mut Game) {
    unsafe {
        let E = Get_RidleysExplosion(k as i32, game);
        let v2: u16 = ((*E).base.frame_counter) & (1u16);
        (*E).base.frame_counter = v2;
        let mut v3: u16 = if v2 != 0 {
            ((*E).base.properties) | ((kEnemyProps_Invisible) as u16)
        } else {
            ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16
        };
        (*E).base.properties = v3;
    }
}
pub unsafe fn RidleysExplosion_Func_2(game: &mut Game) {
    {
        ({
            let (bank, population) = (0xa6, addr_stru_A6C9E7 as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        });
        ({
            let (bank, population) = (0xa6, addr_stru_A6C9D7 as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        });
        ({
            let (bank, population) = (0xa6, addr_stru_A6C9C7 as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        });
        ({
            let (bank, population) = (0xa6, addr_stru_A6C9B7 as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        });
        ({
            let (bank, population) = (0xa6, addr_stru_A6C9A7 as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        });
        ({
            let (bank, population) = (0xa6, addr_stru_A6C997 as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        });
        ({
            let (bank, population) = (0xa6, addr_stru_A6C987 as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        });
        ({
            let (bank, population) = (0xa6, addr_stru_A6C9F7 as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        });
        ({
            let (bank, population) = (0xa6, addr_stru_A6CA07 as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        });
        ({
            let (bank, population) = (0xa6, addr_stru_A6CA27 as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        });
        ({
            let (bank, population) = (0xa6, addr_stru_A6CA17 as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        });
        ({
            let (bank, population) = (0xa6, addr_stru_A6CA37 as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        });
    }
}
pub fn locret_A6CB20(game: &mut Game) {}
static off_A6CB21: [Func_V; 9] = callback_table! { nonnull Func_V;
    locret_A6CB20, Ridley_CBC0, Ridley_Func_71, Ridley_Func_72, Ridley_CBC7, Ridley_CBCE,
    Ridley_Func_73, Ridley_Func_74, Ridley_CBD5,
};
pub unsafe fn Ridley_Func_70(game: &mut Game) {
    unsafe {
        Ridley_Func_91(game);
        off_A6CB21[word!(game.tilemap_stuff[0]) as usize](game);
        Ridley_Func_86(game);
        Ridley_Func_87(game);
        let E = Get_Ridley(0, game);
        if (*E).var_1B
            | (word!(game.samus_invincibility_timer) as i32
                | (*E).base.properties as i32 & kEnemyProps_Tangible as i32) as u16
            == 0
        {
            Ridley_Func_127(game);
        }
    }
}
pub unsafe fn Ridley_Func_71(game: &mut Game) {
    unsafe {
        word!(game.tilemap_stuff[10] = 8);
        Ridley_Func_75(game);
        word!(game.tilemap_stuff[0] = 3);
    }
}
pub unsafe fn Ridley_Func_72(game: &mut Game) {
    unsafe {
        word!(game.tilemap_stuff[10] = 8);
        Ridley_Func_75(game);
    }
}
pub unsafe fn Ridley_Func_73(game: &mut Game) {
    unsafe {
        word!(game.tilemap_stuff[10] = 3);
        Ridley_Func_75(game);
        word!(game.tilemap_stuff[0] = 6);
    }
}
pub unsafe fn Ridley_Func_74(game: &mut Game) {
    unsafe {
        word!(game.tilemap_stuff[10] = 2);
        Ridley_Func_75(game);
        word!(game.tilemap_stuff[0] = 7);
    }
}
pub unsafe fn Ridley_Func_75(game: &mut Game) {
    unsafe {
        if (*Get_Ridley(0i32, game)).var_10 != 0 {
            word! {game; tilemap_stuff[13] = 0x4000; tilemap_stuff[14] = -1i32 as u16; }
        } else {
            word! {game; tilemap_stuff[14] = 0x4000; tilemap_stuff[13] = -1i32 as u16; }
        }
        Ridley_CBC7(game);
        if word!(game.tilemap_stuff[76])
            | (word!(game.tilemap_stuff[66])
                | word!(game.tilemap_stuff[56])
                | word!(game.tilemap_stuff[46])
                | word!(game.tilemap_stuff[36])
                | word!(game.tilemap_stuff[26])
                | word!(game.tilemap_stuff[16]))
            == 0
        {
            word!(game.tilemap_stuff[0] = 4);
        }
    }
}
pub unsafe fn Ridley_CBC0(game: &mut Game) {
    unsafe {
        Ridley_Func_79(game);
        Ridley_CBDC(game);
    }
}
pub unsafe fn Ridley_CBC7(game: &mut Game) {
    unsafe {
        Ridley_Func_83(game);
        Ridley_CBDC(game);
    }
}
pub unsafe fn Ridley_CBCE(game: &mut Game) {
    unsafe {
        Ridley_Func_84(game);
        Ridley_CBDC(game);
    }
}
pub unsafe fn Ridley_CBD5(game: &mut Game) {
    unsafe {
        Ridley_Func_81(game);
        Ridley_CBDC(game);
    }
}
pub unsafe fn Ridley_CBDC(game: &mut Game) {
    unsafe {
        let mut v0: u16 = 7;
        let mut v1: u16 = 0;
        do_until! { fresh11 == 1 ;
            Ridley_Func_88(v1, game);
            v1 = (v1).wrapping_add(20);
            crate::update_place! { previous fresh11 = v0 => v0.wrapping_sub(1); }
        }
    }
}
pub unsafe fn Ridley_Func_77(game: &mut Game) {
    unsafe {
        let v0: i32 = (*Get_Ridley(0i32, game)).var_10 as i32;
        word!(game.tilemap_stuff[11] = crate::rtl::rom_table_word!(game, 0xa6cc12, v0 as isize));
        word!(game.tilemap_stuff[12] = crate::rtl::rom_table_word!(game, 0xa6cc18, v0 as isize));
    }
}
pub fn Ridley_Func_78(game: &mut Game) -> u8 {
    {
        ((word!(game.tilemap_stuff[76])
            & (word!(game.tilemap_stuff[66])
                & word!(game.tilemap_stuff[56])
                & word!(game.tilemap_stuff[46])
                & word!(game.tilemap_stuff[36])
                & word!(game.tilemap_stuff[26])
                & word!(game.tilemap_stuff[16])))
            != 0) as u8
    }
}

macro_rules! start_ridley_tile_effect_when_idle {
    ($game:ident) => {
        if word!($game.tilemap_stuff[76])
            | (word!($game.tilemap_stuff[66])
                | word!($game.tilemap_stuff[56])
                | word!($game.tilemap_stuff[46])
                | word!($game.tilemap_stuff[36])
                | word!($game.tilemap_stuff[26])
                | word!($game.tilemap_stuff[16]))
            == 0
        {
            word!($game.tilemap_stuff[16] = 0x8000);
            word!($game.tilemap_stuff[13] = -1i32 as u16);
            word!($game.tilemap_stuff[14] = -1i32 as u16);
            let mut v0: u16 = 2;
            if word!($game.samus_x_pos) < 0x70 {
                v0 = 1;
            }
            word!($game.tilemap_stuff[10] = v0);
        }
    };
}

pub unsafe fn Ridley_Func_79(game: &mut Game) {
    unsafe {
        Ridley_Func_77(game);
        if Ridley_Func_78(game) != 0 && Ridley_Func_80_DoubleRet(game) != 0 {
            return;
        }
        start_ridley_tile_effect_when_idle!(game);
    }
}
pub unsafe fn Ridley_Func_80_DoubleRet(game: &mut Game) -> u8 {
    unsafe {
        let E = Get_Ridley(0, game);
        let mut retval: bool = false_0 != 0;
        let attack = if word!(game.tilemap_stuff[2]) != 0
            && (word!(game.tilemap_stuff[13]) & word!(game.tilemap_stuff[14]) & 0x8000) != 0
        {
            Some((word!(game.tilemap_stuff[2])).wrapping_sub(1))
        } else if word!(game.tilemap_stuff[1]) == 0 {
            None
        } else if (word!(game.random_number) as u8) >= 0xf0 {
            Some((word!(game.tilemap_stuff[2])).wrapping_sub(1))
        } else if abs16((word!(game.samus_x_pos)).wrapping_sub((*E).base.x_pos)) < 0x80 {
            Some(0)
        } else {
            None
        };
        if let Some(attack) = attack {
            Ridley_Func_89(attack, game);
            retval = true_0 != 0;
        }
        word!(game.tilemap_stuff[2] = 0);
        retval as u8
    }
}
pub unsafe fn Ridley_Func_81(game: &mut Game) {
    unsafe {
        Ridley_Func_77(game);
        if Ridley_Func_78(game) & 1 != 0 && Ridley_Func_82_DoubleRet(game) {
            return;
        }
        start_ridley_tile_effect_when_idle!(game);
    }
}
pub unsafe fn Ridley_Func_82_DoubleRet(game: &mut Game) -> bool {
    unsafe {
        if word!(game.tilemap_stuff[2]) != 0
            && (word!(game.tilemap_stuff[13]) & word!(game.tilemap_stuff[14]) & 0x8000) != 0
        {
            Ridley_Func_89(1, game);
            word!(game.tilemap_stuff[2] = 0);
            return true_0 != 0;
        }
        false_0 != 0
    }
}
// Tail words alias the shared tilemap workspace; keep each load/store ordered.
unsafe fn ridley_tail_word(ram: *mut u8, index: usize) -> u16 {
    unsafe { u16::from_le(ram.add(0x2000 + index * 2).cast::<u16>().read_unaligned()) }
}
unsafe fn ridley_tail_store(ram: *mut u8, index: usize, value: u16) {
    unsafe {
        ram.add(0x2000 + index * 2)
            .cast::<u16>()
            .write_unaligned(value.to_le());
    }
}
unsafe fn ridley_tail_chain(ram: *mut u8, first: usize) {
    unsafe {
        for index in (first + 10..=first + 50).step_by(20) {
            let value =
                ridley_tail_word(ram, index + 2).wrapping_add(ridley_tail_word(ram, index - 10));
            ridley_tail_store(ram, index, value);
            let value = ridley_tail_word(ram, index + 12)
                .wrapping_add(ridley_tail_word(ram, index + 2))
                .wrapping_add(ridley_tail_word(ram, index - 10));
            ridley_tail_store(ram, index + 10, value);
        }
    }
}
unsafe fn ridley_tail_length(ram: *mut u8, index: usize, maximum: u16, resting: u16) {
    unsafe {
        if ridley_tail_word(ram, index + 1) != 0 {
            if (ridley_tail_word(ram, index + 1) as i16)
                .wrapping_sub(ridley_tail_word(ram, index) as i16)
                < 0
            {
                ridley_tail_store(ram, index + 1, 0);
            }
            let mut value = ridley_tail_word(ram, 9).wrapping_add(ridley_tail_word(ram, index));
            if (ridley_tail_word(ram, 9) as i32 + ridley_tail_word(ram, index) as i32
                - maximum as i32)
                & 0x8000
                == 0
            {
                value = maximum;
            }
            ridley_tail_store(ram, index, value);
        } else if (resting as i16).wrapping_sub(ridley_tail_word(ram, index) as i16) < 0 {
            let value = ridley_tail_word(ram, index).wrapping_sub(128);
            ridley_tail_store(ram, index, value);
        }
    }
}
pub unsafe fn Ridley_Func_85(game: &mut Game) {
    unsafe {
        let ram = (&raw mut game.g_ram).cast::<u8>();
        for index in [21, 31, 41, 51] {
            let value = ram.add(0x2000 + index * 2).read() as u16 | 0x4000;
            ridley_tail_store(ram, index, value);
        }
        // Segment 61 deliberately uses word 21, including its bit 10.
        let value = ridley_tail_word(ram, 21) & 0x4ff | 0x4000;
        ridley_tail_store(ram, 61, value);
        for index in [71, 81] {
            let value = ram.add(0x2000 + index * 2).read() as u16 | 0x4000;
            ridley_tail_store(ram, index, value);
        }
    }
}
pub unsafe fn Ridley_Func_87(game: &mut Game) {
    unsafe {
        let ram = (&raw mut game.g_ram).cast::<u8>();
        ridley_tail_length(ram, 29, 6144, 2048);
        ridley_tail_length(ram, 39, 6144, 2048);
        ridley_tail_length(ram, 49, 5632, 2048);
        ridley_tail_length(ram, 59, 5632, 2048);
        ridley_tail_length(ram, 69, 4608, 2048);
        ridley_tail_length(ram, 79, 1280, 1280);
    }
}
pub unsafe fn Ridley_Func_88(k: u16, game: &mut Game) {
    unsafe {
        let mut v6: i16 = 0;
        let mut v2: u16 = 0;
        let v1: i32 = k as i32 >> 1;
        if (word!(game.tilemap_stuff[(v1 + 16)]) & 0x8000) != 0 {
            let v4: i32 = k as i32 >> 1;
            let v5: u16 = word!(game.tilemap_stuff[(v4 + 17)]);
            if v5 < word!(game.tilemap_stuff[15]) {
                word!(game.tilemap_stuff[((k as i32 >> 1) + 17)] += word!(game.tilemap_stuff[10]));
                return;
            }
            if v5 != 0xffff {
                word!(game.tilemap_stuff[(v4 + 17)] = -1i32 as u16);
                word!(game.tilemap_stuff[(v4 + 26)] = 0x8000);
                word!(game.tilemap_stuff[(v4 + 28)] = word!(game.tilemap_stuff[(v4 + 18)]));
            }
            let stopped = 'advance: {
                if (word!(game.tilemap_stuff[(v4 + 18)]) & 0x8000) == 0 {
                    let turn = if (word!(game.tilemap_stuff[14]) & 0x8000) != 0 {
                        v2 = (word!(game.tilemap_stuff[10]))
                            .wrapping_add(word!(game.tilemap_stuff[(v4 + 21)]));
                        (v2 as i16).wrapping_sub(word!(game.tilemap_stuff[12]) as i16) >= 0
                    } else {
                        word!(game.tilemap_stuff[(v4 + 20)] = 3072);
                        v2 = (word!(game.tilemap_stuff[10]))
                            .wrapping_add(word!(game.tilemap_stuff[(v4 + 21)]));
                        if (v2 as i16).wrapping_sub(word!(game.tilemap_stuff[14]) as i16) >= 0 {
                            if k == 0 || word!(game.tilemap_stuff[(v4 + 6)]) == 0 {
                                word!(
                                    game.tilemap_stuff[(v4 + 21)] = word!(game.tilemap_stuff[14])
                                );
                                break 'advance true;
                            }
                            true
                        } else {
                            false
                        }
                    };
                    if turn {
                        word!(game.tilemap_stuff[(v4 + 18)] = 0x8000);
                        v2 = word!(game.tilemap_stuff[12]);
                    }
                } else {
                    let turn = if (word!(game.tilemap_stuff[13]) & 0x8000) != 0 {
                        v6 = (word!(game.tilemap_stuff[(v4 + 21)]) as i32
                            - word!(game.tilemap_stuff[10]) as i32
                            - 1i32) as i16;
                        v6.wrapping_sub(word!(game.tilemap_stuff[11]) as i16) < 0
                    } else {
                        word!(game.tilemap_stuff[(v4 + 20)] = 3072);
                        v6 = (word!(game.tilemap_stuff[(v4 + 21)]) as i32
                            - word!(game.tilemap_stuff[10]) as i32
                            - 1i32) as i16;
                        if v6.wrapping_sub(word!(game.tilemap_stuff[13]) as i16) < 0 {
                            if k == 0 || word!(game.tilemap_stuff[(v4 + 6)]) == 0 {
                                word!(
                                    game.tilemap_stuff[(v4 + 21)] = word!(game.tilemap_stuff[13])
                                );
                                break 'advance true;
                            }
                            true
                        } else {
                            false
                        }
                    };
                    if turn {
                        word!(game.tilemap_stuff[(v4 + 18)] = 0);
                        v2 = word!(game.tilemap_stuff[11]);
                    } else {
                        v2 = (v6 as u16).wrapping_add(1);
                    }
                }
                word!(game.tilemap_stuff[(v4 + 21)] = v2);
                false
            };
            if stopped {
                let v3: i32 = k as i32 >> 1;
                word! { game, 0; tilemap_stuff[(v3 + 16)], tilemap_stuff[(v3 + 17)], }
                word!(game.tilemap_stuff[(v3 + 18)] ^= (0x8000u16));
                return;
            }
        } else {
            v2 = word!(game.tilemap_stuff[(v1 + 21)]);
        }
        let mut r18: u16 = v2 as u8 as u16;
        if k != 0 {
            r18 = ((byte!(@pointer (ram_ptr!(game, u16; 0x2000)).offset(((k as i32 >> 1i32) + 11i32) as isize); 0))
        .wrapping_add((r18) as u8)) as u16;
        }
        let v7: i32 = k as i32 >> 1;
        word! { game;
            tilemap_stuff[(v7 + 24)] = ComputeSinMult(
                *((ram_ptr!(game, u16; 0x2000)).offset((v7 + 19) as isize) as *mut u8).offset(1) as u16,
                r18,
                game,
            );
            tilemap_stuff[(v7 + 25)] = ComputeCosMult(
                *((ram_ptr!(game, u16; 0x2000)).offset((v7 + 19) as isize) as *mut u8).offset(1) as u16,
                r18,
                game,
            );
        }
    }
}
pub unsafe fn Ridley_Func_89(a: u16, game: &mut Game) {
    unsafe {
        let mut v1: u16 = 0;
        let mut r18: u16 = 0;
        let mut r20: u16 = 0;
        let mut v2: i8 = a as i8;
        byte!(v1; 0) = 0;
        byte!(v1; 1) = v2 as u8;
        word!(game.tilemap_stuff[5] = v1);
        let E = Get_Ridley(0, game);
        if (*E).var_10 != 1 && (*E).var_01 != 0 {
            let mut pt: Point16U = Point16U { x: 0, y: 0 };
            if Ridley_Func_90(&raw mut pt, game) != 0 {
                r18 = (pt.x).wrapping_sub(word!(game.tilemap_stuff[22]));
                r20 = (pt.y).wrapping_sub(word!(game.tilemap_stuff[23]));
            } else {
                r18 = (word!(game.samus_x_pos)).wrapping_sub(word!(game.tilemap_stuff[22]));
                r20 = (word!(game.samus_y_pos) as i32 + 24 - word!(game.tilemap_stuff[23]) as i32)
                    as u16;
            }
            r18 = -(actor_api::angle_from_delta(r18, r20) as i32 + 0x80i32) as u8 as u16;
            if (*E).var_10 != 0 {
                let mut v7: u16 = r18;
                if r18 >= 0x18 && r18 < 0xe8 {
                    v7 = 24;
                }
                let v8: u16 = (word!(game.tilemap_stuff[5]) as i32 + v7 as i32 + 0x4000) as u16;
                if v8 >= word!(game.tilemap_stuff[21]) {
                    word! {game; tilemap_stuff[14] = v8; tilemap_stuff[10] = 8; }
                }
            } else {
                let mut v5: u16 = r18;
                if r18 >= 0x18 && r18 < 0xe8 {
                    v5 = 232;
                }
                let v6: u16 = (v5 as i32 + 16128 - word!(game.tilemap_stuff[5]) as i32) as u16;
                if v6 < word!(game.tilemap_stuff[21]) {
                    word! {game; tilemap_stuff[13] = v6; tilemap_stuff[10] = 8; }
                }
            }
        }
    }
}
pub unsafe fn Ridley_Func_90(out: *mut Point16U, game: &mut Game) -> u8 {
    unsafe {
        let rect: Rect16U = Rect16U {
            x: word!(game.tilemap_stuff[82]),
            y: word!(game.tilemap_stuff[83]),
            w: 64,
            h: 64,
        };
        if word!(game.projectile_counter) == 0 {
            return 0;
        }
        let mut v1: u16 = 0;
        loop {
            let v2: i32 = v1 as i32 >> 1;
            if (byte!(@pointer (ram_ptr!(game, u16; 0xc18)).offset(v2 as isize); 1) & 0xf) == 1
                || (byte!(@pointer (ram_ptr!(game, u16; 0xc18)).offset(v2 as isize); 1) & 0xf) == 2
            {
                let v3: u16 = abs16((word!(game.projectile_x_pos[v2])).wrapping_sub(rect.x));
                let mut v4: bool = v3 < word!(game.projectile_x_radius[v2]);
                let v5: u16 = (v3).wrapping_sub(word!(game.projectile_x_radius[v2]));
                if v4 || v5 < rect.w {
                    let v6: u16 = abs16((word!(game.projectile_y_pos[v2])).wrapping_sub(rect.y));
                    v4 = v6 < word!(game.projectile_y_radius[v2]);
                    let v7: u16 = (v6).wrapping_sub(word!(game.projectile_y_radius[v2]));
                    if v4 || v7 < rect.h {
                        break;
                    }
                }
            }
            v1 = (v1).wrapping_add(2);
            if ((v1) as i16).wrapping_sub(10) >= 0 {
                return 0;
            }
        }
        let v8: i32 = v1 as i32 >> 1;
        (*out).x = word!(game.projectile_x_pos[v8]);
        (*out).y = word!(game.projectile_y_pos[v8]);
        1
    }
}
pub unsafe fn Ridley_Func_92(game: &mut Game) {
    unsafe {
        word! {game; tilemap_stuff[10] = 1; tilemap_stuff[11] = 16368; tilemap_stuff[12] = 16448;
        tilemap_stuff[13] = -1i32 as u16; tilemap_stuff[14] = -1i32 as u16;
        tilemap_stuff[9] = 240; tilemap_stuff[15] = 16; }
        word! { game, 17;
            tilemap_stuff[17], tilemap_stuff[27], tilemap_stuff[37], tilemap_stuff[47],
            tilemap_stuff[57], tilemap_stuff[67], tilemap_stuff[77],
        }
        Ridley_Func_93(0x2024, addr_word_A6D36E as u16, game);
        Ridley_Func_93(0x2026, addr_word_A6D37C as u16, game);
        Ridley_Func_93(0x202a, addr_word_A6D38A as u16, game);
        Ridley_Func_93(0x202c, addr_word_A6D398 as u16, game);
        Ridley_Func_93(0x202e, addr_word_A6D3A6 as u16, game);
        word! { game, 0;
            tilemap_stuff[16], tilemap_stuff[26], tilemap_stuff[36], tilemap_stuff[46],
            tilemap_stuff[56], tilemap_stuff[66], tilemap_stuff[76],
        }
    }
}
pub unsafe fn Ridley_Func_93(j: u16, k: u16, game: &mut Game) {
    unsafe {
        let mut dst: *mut u16 = ram_ptr!(game, u16; j as i32 as isize);
        let mut src = RomPtr_A6(k, game) as *const u16;
        for _ in 0..7 {
            *dst = *src;
            dst = dst.offset(10);
            src = src.offset(1);
        }
    }
}
crate::forward_calls! { pub fn (game: &mut Game);
    Ridley_Func_95 => Ridley_Func_D3DC(0x8000, game);
}
pub fn Ridley_Func_D3DC(a: u16, game: &mut Game) {
    {
        word! {game; tilemap_stuff[16] = a; tilemap_stuff[26] = a; tilemap_stuff[36] = a;
        tilemap_stuff[46] = a; tilemap_stuff[56] = a; tilemap_stuff[66] = a;
        tilemap_stuff[76] = a; }
    }
}
pub unsafe fn Ridley_Func_96(game: &mut Game) {
    unsafe {
        Ridley_Func_93(0x2026, addr_word_A6D37C as u16, game);
        let mut v1: u16 = 0;
        for _ in 0i16..7i16 {
            let v2: i32 = v1 as i32 >> 1;
            word!(
                game.tilemap_stuff[(v2 + 21)] =
                    (0x8000u16).wrapping_sub(word!(game.tilemap_stuff[(v2 + 21)]),)
            );
            word!(game.tilemap_stuff[(v2 + 18)] |= (0x8000u16));
            v1 = (v1).wrapping_add(20);
        }
    }
}
pub unsafe fn Ridley_Func_98(game: &mut Game) {
    unsafe {
        let enemy_ptr: u16 = (*Get_Ridley(word!(game.cur_enemy_index) as i32, game))
            .base
            .enemy_ptr;
        let v1 = RomPtr_A0(enemy_ptr, game);
        damage_player(
            {
                let amount = *(v1.offset(6) as *mut u16);
                actors::combat::suit_damage(&mut Runtime::new(game), amount)
            },
            game,
        );
    }
}
pub unsafe fn Ridley_Func_99(a: u16, game: &mut Game) {
    unsafe {
        let E = Get_Ridley(0, game);
        enemy_instruction! {[(*E).base]; instruction = a, timer = 1}
        (*E).base.timer = 0;
    }
}
pub unsafe fn Ridley_Func_100(game: &mut Game) {
    unsafe {
        let mut v0: i16 = 0;
        let E = Get_Ridley(0, game);
        let health: u16 = (*E).base.health;
        if health < 0x2328 {
            v0 = 1;
            if health < 0x1518 {
                v0 = 2;
                if health < 0x708 {
                    v0 = 3;
                }
            }
        }
        (*E).var_12 = v0 as u16;
        if (v0).wrapping_sub(1) >= 0 {
            Ridley_D495(((v0) as u16).wrapping_sub(1), game);
        }
    }
}
pub unsafe fn Ridley_D495(r18: u16, game: &mut Game) {
    unsafe {
        if word!(game.palette_change_num) == 0 {
            WriteColorsToPalette(0x1e2, 0xa6, (28 * r18 as i32 - 7062i32) as u16, 0xe, game);
        }
    }
}
pub unsafe fn Ridley_Func_102(game: &mut Game) {
    unsafe {
        let mut v0: i16 = 3584;
        let E = Get_Ridley(0, game);
        if (*E).base.flash_timer >= 2 && (word!(game.random_enemy_counter) as i32 + 1 & 2) != 0 {
            v0 = 0;
        }
        (*E).var_0C = v0 as u16;
    }
}
pub unsafe fn Ridley_Func_103(k: u16, j: u16, game: &mut Game) -> u8 {
    unsafe {
        let prod: u16 = Mult8x8((j >> 4) as u8, word!(game.room_width_in_blocks) as u8);
        ((*(ram_ptr!(game, u16; 0x10002)).offset((prod as i32 + (k as i32 >> 4)) as isize)
            & 0xf000)
            != 0) as u8
    }
}
pub unsafe fn Ridley_Func_104_0(k: u16, j: u16, r18: u16, r20: u16, game: &mut Game) {
    unsafe {
        Ridley_Func_104(k, j, 0, r18, r20, game);
    }
}

pub(super) enum HomingReversal {
    ExtraForce(u16),
    DoubleStep,
}

// Distance, acceleration and the limit tests wrap at sixteen bits. In
// particular, the limit is a signed-difference test, not an ordinary clamp.
pub(super) unsafe fn homing_velocity(
    distance: i16,
    mut velocity: u16,
    divisor: u8,
    reversal: HomingReversal,
    limit: u16,
) -> u16 {
    unsafe {
        let step = SnesDivide(distance.unsigned_abs(), divisor).max(1);
        let reversing = (velocity as i16 >= 0) == (distance > 0);
        let extra = if reversing {
            match reversal {
                HomingReversal::ExtraForce(force) => step.wrapping_add(force),
                HomingReversal::DoubleStep => step.wrapping_mul(2),
            }
        } else {
            0
        };
        if distance > 0 {
            velocity = velocity.wrapping_sub(extra).wrapping_sub(step);
            if velocity.wrapping_add(limit) & 0x8000 != 0 {
                velocity = 0u16.wrapping_sub(limit);
            }
        } else {
            velocity = velocity.wrapping_add(extra).wrapping_add(step);
            if velocity.wrapping_sub(limit) & 0x8000 == 0 {
                velocity = limit;
            }
        }
        velocity
    }
}

pub unsafe fn Ridley_Func_104(k: u16, j: u16, a: u16, r18: u16, r20: u16, game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("homing.ridley.extra");
        let divisor = g_byte_A6D61F[j as usize];
        Ridley_Func_105(k, r18, u16::from(divisor), a, game);
        let E = Get_Ridley(k as i32, game);
        let distance = ((*E).base.y_pos as i16).wrapping_sub(r20 as i16);
        if distance != 0 {
            (*E).var_C = homing_velocity(
                distance,
                (*E).var_C,
                divisor as u8,
                HomingReversal::ExtraForce(a.wrapping_add(8)),
                1280,
            );
        }
    }
}
pub unsafe fn Ridley_Func_105(k: u16, r18: u16, r24: u16, r26: u16, game: &mut Game) {
    unsafe {
        let E = Get_Ridley(k as i32, game);
        let distance = ((*E).base.x_pos as i16).wrapping_sub(r18 as i16);
        if distance != 0 {
            (*E).var_B = homing_velocity(
                distance,
                (*E).var_B,
                r24 as u8,
                HomingReversal::ExtraForce(r26.wrapping_add(8)),
                1280,
            );
        }
    }
}
pub unsafe fn Ridley_Func_106(k: u16, j: u16, r18: u16, r20: u16, game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("homing.ridley.double");
        let divisor = g_byte_A6D712[j as usize];
        Ridley_Func_107(k, r18, u16::from(divisor), game);
        let E = Get_Ridley(k as i32, game);
        let distance = ((*E).base.y_pos as i16).wrapping_sub(r20 as i16);
        if distance != 0 {
            (*E).var_C = homing_velocity(
                distance,
                (*E).var_C,
                divisor as u8,
                HomingReversal::DoubleStep,
                1280,
            );
        }
    }
}
pub unsafe fn Ridley_Func_107(k: u16, r18: u16, r24: u16, game: &mut Game) {
    unsafe {
        let E = Get_Ridley(k as i32, game);
        let distance = ((*E).base.x_pos as i16).wrapping_sub(r18 as i16);
        if distance != 0 {
            (*E).var_B = homing_velocity(
                distance,
                (*E).var_B,
                r24 as u8,
                HomingReversal::DoubleStep,
                1280,
            );
        }
    }
}

macro_rules! steer_enemy_speed_and_angle {
    ($target_speed:ident, $enemy:ident, $angle_step:ident, $target_angle:ident) => {
        if $target_speed != (*$enemy).var_0B {
            if (($target_speed) as i16).wrapping_sub(((*$enemy).var_0B) as i16) >= 0 {
                let mut v2: u16 = ((*$enemy).var_0B).wrapping_add(32);
                if v2 as i32 - $target_speed as i32 & 0x8000 == 0 {
                    v2 = $target_speed;
                }
                (*$enemy).var_0B = v2;
            } else {
                let mut v1: u16 = ((*$enemy).var_0B).wrapping_sub(32);
                if v1 as i32 - $target_speed as i32 & 0x8000 != 0 {
                    v1 = $target_speed;
                }
                (*$enemy).var_0B = v1;
            }
        }
        let mut v3: u16 = 0;

        let use_target_angle: bool = if ($angle_step & 0x8000) != 0 {
            v3 = ((*$enemy).var_0A).wrapping_add($angle_step);
            (v3 as i32 - $target_angle as i32 & 0x8000 != 0)
        } else {
            v3 = ((*$enemy).var_0A).wrapping_add($angle_step);
            (v3 as i32 - $target_angle as i32 & 0x8000 == 0)
        };
        if use_target_angle {
            v3 = $target_angle;
        }
        (*$enemy).var_0A = v3;
        $angle_step = byte!(v3; 1) as u16;
        (*$enemy).var_B = Math_MultBySin((*$enemy).var_0B, $angle_step);
        (*$enemy).var_C = Math_MultByCos((*$enemy).var_0B, $angle_step);
    };
}
pub(crate) use steer_enemy_speed_and_angle;

pub unsafe fn Ridley_Func_111(mut r18: u16, r20: u16, r22: u16, game: &mut Game) {
    unsafe {
        let E = Get_Ridley(0, game);
        steer_enemy_speed_and_angle!(r22, E, r18, r20);
    }
}
pub unsafe fn Ridley_Func_112(game: &mut Game) {
    unsafe {
        let mut Ridley = Get_Ridley(0, game);
        (*Ridley).var_1F = 0;
        let mut v1: u16 = word!(game.cur_enemy_index);
        let mut v2 = Get_Ridley(word!(game.cur_enemy_index) as i32, game);
        let mut with_carry: i32 =
            byte!((*v2).base.x_subpos; 1) as i32 + byte!((*v2).var_B; 0) as i32;
        byte!((*v2).base.x_subpos; 1) = with_carry as u8;
        let mut ridley_var_23: u16 =
            ((*v2).base.x_pos as i32 + ((*v2).var_B >> 8) as i8 as i32 + (with_carry >> 8)) as u16;
        if ((ridley_var_23) as i16).wrapping_sub(((*Ridley).var_22) as i16) >= 0 {
            if ((ridley_var_23) as i16).wrapping_sub(((*Ridley).var_23) as i16) >= 0 {
                (*v2).var_B = 0;
                (*Ridley).var_1F = 2;
                ridley_var_23 = (*Ridley).var_23;
            }
            (*v2).base.x_pos = ridley_var_23;
        } else {
            Ridley_Func_113(v1, game);
            (*v2).base.x_pos = (*Ridley).var_22;
            (*v2).var_B = 0;
            (*Ridley).var_1F = 1;
        }
        with_carry = byte!((*v2).base.y_subpos; 1) as i32 + byte!((*v2).var_C; 0) as i32;
        byte!((*v2).base.y_subpos; 1) = with_carry as u8;
        let mut ridley_var_21: u16 =
            ((*v2).base.y_pos as i32 + ((*v2).var_C >> 8) as i8 as i32 + (with_carry >> 8)) as u16;
        if ((ridley_var_21) as i16).wrapping_sub(((*Ridley).var_20) as i16) >= 0 {
            if ((ridley_var_21) as i16).wrapping_sub(((*Ridley).var_21) as i16) >= 0 {
                (*v2).var_C = 0;
                (*Ridley).var_1F = 8;
                ridley_var_21 = (*Ridley).var_21;
            }
            (*v2).base.y_pos = ridley_var_21;
        } else {
            (*v2).base.y_pos = (*Ridley).var_20;
            (*v2).var_C = 0;
            (*Ridley).var_1F = 4;
        };
    }
}
pub unsafe fn Ridley_Func_113(k: u16, game: &mut Game) {
    unsafe {
        if word!(game.area_index) != 2 {
            let E = Get_Ridley(k as i32, game);
            let r18: u16 = abs16((*E).var_B);
            let mut v2: u16 = abs16((*E).var_C);
            if v2 < r18 {
                v2 = r18;
            }
            if v2 >= 0x280 {
                let mut v3: u16 = if word!(game.area_index) == 2 { 24 } else { 33 };
                word! {game; earthquake_type = v3; earthquake_timer = 12; }
            }
        }
    }
}
pub unsafe fn Ridley_Func_115(game: &mut Game) {
    unsafe {
        Ridley_Func_116(game);
        let E = Get_Ridley(0, game);
        let mut v1: i16 = (((*E).var_09) as i16).wrapping_sub(((*E).var_08) as i16);
        (*E).var_09 = v1 as u16;
        if v1 < 0 {
            (*E).var_09 = 32;
            let mut v2: u16 = ((*E).var_07).wrapping_add(1);
            if v2 >= 0xa {
                v2 = 0;
            }
            (*E).var_07 = v2;
        }
    }
}
pub unsafe fn Ridley_Func_119(carry: u8, game: &mut Game) {
    unsafe {
        let mut v1: u16 = addr_off_A6DAD0 as u16;
        if carry != 0 {
            v1 = addr_off_A6DAD4 as u16;
        }
        let v2: u16 = word!(game.vram_write_queue_tail);
        let mut v3 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        *((&raw mut (*v3).src.addr as *mut u8).offset(1) as *mut VoidP) = -20480i32 as VoidP;
        *((&raw mut (*v3.offset(1)).src.addr as *mut u8).offset(1) as *mut VoidP) =
            -20480i32 as VoidP;
        let v4 = RomPtr_A6(v1, game);
        (*v3).src.addr = *(v4 as *mut u16);
        (*v3.offset(1)).src.addr = *(v4.offset(2) as *mut u16);
        (*v3).vram_dst = 31424;
        (*v3.offset(1)).vram_dst = 31680;
        (*v3).size = 128;
        (*v3.offset(1)).size = 128;
        word!(game.vram_write_queue_tail = (v2).wrapping_add(14));
        (*gVramWriteEntry(v2 as i32 + 14i32, game)).size = 0;
    }
}
pub unsafe fn sub_A6DB2A(game: &mut Game) {
    unsafe {
        if ((*gEnemyData(0i32, game)).properties as i32 & kEnemyProps_Invisible as i32) == 0 {
            sub_A6DBC2(
                crate::rtl::rom_table_word!(
                    game,
                    0xa6dcba,
                    ((*((ram_ptr!(game, u16; 0x2000)).offset(71) as *mut u8) as i32
                        + *((ram_ptr!(game, u16; 0x2000)).offset(81) as *mut u8) as i32
                        + 8
                        & 0xf0) as u8
                        >> 4) as isize,
                ),
                word!(game.tilemap_stuff[82]),
                word!(game.tilemap_stuff[83]),
                game,
            );
            sub_A6DBC2(
                0xdc9e,
                word!(game.tilemap_stuff[72]),
                word!(game.tilemap_stuff[73]),
                game,
            );
            sub_A6DBC2(
                0xdc9e,
                word!(game.tilemap_stuff[62]),
                word!(game.tilemap_stuff[63]),
                game,
            );
            sub_A6DBC2(
                0xdc97,
                word!(game.tilemap_stuff[52]),
                word!(game.tilemap_stuff[53]),
                game,
            );
            sub_A6DBC2(
                0xdc97,
                word!(game.tilemap_stuff[42]),
                word!(game.tilemap_stuff[43]),
                game,
            );
            sub_A6DBC2(
                0xdc90,
                word!(game.tilemap_stuff[32]),
                word!(game.tilemap_stuff[33]),
                game,
            );
            sub_A6DBC2(
                0xdc90,
                word!(game.tilemap_stuff[22]),
                word!(game.tilemap_stuff[23]),
                game,
            );
        }
    }
}
pub unsafe fn sub_A6DBC2(j: u16, x: u16, y: u16, game: &mut Game) {
    unsafe {
        sub_A6DC13(j, x, y, (*Get_Ridley(0i32, game)).var_0C, game);
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallBabyMetroidInstr(ea: u32, k: u16, game: &mut Game) -> u16 => ea;
    call (k, game); default Unreachable(game) as u16;
    @functions [pub unsafe fn (k: u16, game: &mut Game) -> u16];
    [10928073] BabyMetroid_Instr_2 [(k).wrapping_add(2)] {
        if (*Get_Ridley(0, game)).var_46 == 0 && (word!(game.random_number) & 1) != 0 {
            return BabyMetroid_Goto(k, game);
        }
        QueueSfx3_Max6(0x24, game);
    };
    [10928097] BabyMetroid_Instr_3 [(k).wrapping_add(2)] {
        let v1: u16 = *(RomPtr_A6(k, game) as *mut u16);
        WriteColorsToPalette(0x162, 0xa6, v1, 0xf, game);
    };
    [10928114] BabyMetroid_Instr_1 [] {
        if (*Get_Ridley(0, game)).var_46 != 0 {
            BabyMetroid_Goto(k, game)
        } else {
            (k).wrapping_add(2)
        }
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game) -> u16];
    [10928120] BabyMetroid_Goto [] {
        *(RomPtr_A6(k, game) as *mut u16)
    };
}
unsafe fn BabyMetroid_DBCB_DoubleRetEx(a: u16, game: &mut Game) -> i32 {
    unsafe {
        let st: *mut BabyMetroidExecState = ram_ptr!(game, BabyMetroidExecState; a as isize);
        if ((*st).ip & 0x8000) == 0 {
            return -1i32;
        }
        let mut v2: u16 = (*st).ip;
        let mut v3: *const u16 = RomPtr_A6(v2, game) as *mut u16;
        let mut execute_instruction: bool = (*v3 & 0x8000 != 0)
            || ({
                if (*st).timer != *v3 {
                    (*st).timer = (*st).timer.wrapping_add(1);
                    return *v3.offset(1) as i32;
                }
                v2 = (v2).wrapping_add(4);
                false
            });
        loop {
            if execute_instruction {
                v2 = CallBabyMetroidInstr(
                    ((*v3) as u32) | (0xa60000u32),
                    (v2).wrapping_add(2),
                    game,
                );
                execute_instruction = false;
            } else {
                v3 = RomPtr_A6(v2, game) as *mut u16;
                if *v3 & 0x8000 == 0 {
                    break;
                } else {
                    execute_instruction = true;
                }
            }
        }
        (*st).timer = 1;
        (*st).ip = v2;
        *v3.offset(1) as i32
    }
}
pub unsafe fn sub_A6DC13(j: u16, r18: u16, r20: u16, r22: u16, game: &mut Game) {
    unsafe {
        let mut p = RomPtr_A6(j, game);
        let mut n: i32 = *(p as *mut u16) as i32;
        p = p.offset(2);
        let mut idx: i32 = word!(game.oam_next_ptr) as i32;
        do_until! { n == 0 ;
            let v9: i16 =
                (r20 as i32 + *p.offset(2) as i8 as i32 - word!(game.layer1_y_pos) as i32) as i16;
            if v9 >= 0 && v9 as i32 - 224i32 & 0x8000 != 0 {
                let x: u16 =
                    (r18 as i32 + *(p as *mut u16) as i32 - word!(game.layer1_x_pos) as i32) as u16;
                let v11 = gOamEnt(idx, game);
                (*v11).xcoord = x as u8;
                (*v11).ycoord = v9 as u8;
                *(&raw mut (*v11).charnum as *mut u16) = (r22) | (*(p.offset(3) as *mut u16));
                word!(bits_i32, game.oam_ext[(idx >> 5)] |= ((x as i32 & 0x100) >> 8 | (*(p as *mut i16) < 0) as i32 * 2)
                        << 2 * (idx >> 2 & 7));
                idx = idx + 4 & 0x1ff;
            }
            p = p.offset(5);
            n -= 1;
        }
        word!(game.oam_next_ptr = idx as u16);
    }
}
pub unsafe fn Ridley_Func_121(game: &mut Game) -> u8 {
    unsafe {
        let E = Get_Ridley(0, game);
        let mut y_pos: i16 = (*E).base.y_pos as i16;
        let result: u8 = 1;
        if y_pos >= 0 {
            let mut v2: i16 = (y_pos as i32 + 32 - word!(game.layer1_y_pos) as i32) as i16;
            if v2 >= 0 && v2 as i32 - 288i32 & 0x8000 != 0 {
                let mut x_pos: i16 = (*E).base.x_pos as i16;
                if x_pos >= 0 {
                    let mut v4: i16 = (x_pos as i32 + 32 - word!(game.layer1_x_pos) as i32) as i16;
                    if v4 >= 0 && v4 as i32 - 320i32 & 0x8000 != 0 {
                        return 0;
                    }
                }
            }
        }
        result
    }
}
pub unsafe fn Ridley_Func_122(rect: Rect16U, game: &mut Game) -> u16 {
    unsafe {
        if word!(game.projectile_counter) == 0 {
            return 0xffff;
        }
        let mut result: u16 = 0;
        loop {
            let v1: i32 = result as i32 >> 1;
            let mut v2: i16 = word!(game.projectile_type[v1]) as i16;
            if v2 < 0 && (byte!(v2; 1) as i32 & 0xf) - 3i32 & 0x8000 != 0 {
                let v3: u16 = abs16((word!(game.projectile_x_pos[v1])).wrapping_sub(rect.x));
                let mut v4: bool = v3 < word!(game.projectile_x_radius[v1]);
                let v5: u16 = (v3).wrapping_sub(word!(game.projectile_x_radius[v1]));
                if v4 || v5 < rect.w {
                    let v6: u16 = abs16((word!(game.projectile_y_pos[v1])).wrapping_sub(rect.y));
                    v4 = v6 < word!(game.projectile_y_radius[v1]);
                    let v7: u16 = (v6).wrapping_sub(word!(game.projectile_y_radius[v1]));
                    if v4 || v7 < rect.h {
                        break;
                    }
                }
            }
            result = (result).wrapping_add(2);
            if ((result) as i16).wrapping_sub(10) >= 0 {
                return 0xffff;
            }
        }
        let v8: i32 = result as i32 >> 1;
        word! {game; projectile_x_pos[v8] = rect.x; projectile_y_pos[v8] = rect.y; }
        word!(game.projectile_dir[v8] |= (0x10u16));
        result
    }
}
pub fn Ridley_Func_124(rect: Rect16U, game: &mut Game) -> u8 {
    {
        let v0: u16 = abs16((word!(game.samus_x_pos)).wrapping_sub(rect.x));
        let mut v1: bool = v0 < word!(game.samus_x_radius);
        let v2: u16 = (v0).wrapping_sub(word!(game.samus_x_radius));
        if v1 || v2 < rect.w {
            let v3: u16 = abs16((word!(game.samus_y_pos)).wrapping_sub(rect.y));
            v1 = v3 < word!(game.samus_y_radius);
            let v4: u16 = (v3).wrapping_sub(word!(game.samus_y_radius));
            if v1 || v4 < rect.h {
                return 1;
            }
        }
        0
    }
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    sub_A6DF59 => actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
}
pub unsafe fn Ridley_Func_127(game: &mut Game) {
    unsafe {
        let rect: Rect16U = Rect16U {
            x: word!(game.tilemap_stuff[82]),
            y: word!(game.tilemap_stuff[83]),
            w: 14,
            h: 14,
        };
        if Ridley_Func_124(rect, game) != 0 {
            let E = Get_Ridley(0, game);
            damage_player(
                {
                    let amount = (*E).var_1C;
                    actors::combat::suit_damage(&mut Runtime::new(game), amount)
                },
                game,
            );
            word!(game.samus_invincibility_timer = 96);
            word!(game.samus_knockback_timer = 5);
            word!(
                game.knockback_x_dir = (((word!(game.samus_x_pos)) as i16)
                    .wrapping_sub((word!(game.tilemap_stuff[82])) as i16,)
                    >= 0) as u16
            );
        }
    }
}
pub unsafe fn Ridley_Func_129(game: &mut Game) {
    unsafe {
        if ((*Get_Ridley(0i32, game)).base.properties as i32 & kEnemyProps_Tangible as i32) == 0 {
            let mut v0: u16 = Ridley_Func_122(
                Rect16U {
                    x: word!(game.tilemap_stuff[82]),
                    y: word!(game.tilemap_stuff[83]),
                    w: 14,
                    h: 14,
                },
                game,
            );
            if v0 & 0x8000 != 0 {
                v0 = Ridley_Func_122(
                    Rect16U {
                        x: word!(game.tilemap_stuff[72]),
                        y: word!(game.tilemap_stuff[73]),
                        w: 10,
                        h: 10,
                    },
                    game,
                );
                if v0 & 0x8000 != 0 {
                    return;
                }
            }
            let v1: i32 = v0 as i32 >> 1;
            game.eproj_spawn_pt = Point16U {
                x: word!(game.projectile_x_pos[v1]),
                y: word!(game.projectile_y_pos[v1]),
            };
            let mut v2: u16 = 12;
            if (byte!(@pointer (ram_ptr!(game, u16; 0xc18)).offset(v1 as isize); 1) & 0xf) == 1 {
                QueueSfx1_Max6(0x3d, game);
                v2 = 6;
            }
            SpawnEprojWithRoomGfx(0xe509, v2, game);
        }
    }
}
pub unsafe fn Ridley_E90C(k: u16, jp: EnemyScript, a: u16, game: &mut Game) -> EnemyScript {
    unsafe {
        word!(game.cinematic_var9 = a);
        let E = Get_Ridley(0, game);
        SpawnEprojWithRoomGfx(0x9642, (*E).var_10, game);
        jp
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallCeresSteamFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [10940441] CeresSteam_Func_1 [] {
        let E = Get_CeresSteam(k as i32, game);
        let pt: Point16U = CalcCeresSteamPos_Mode7(
            Point16U {
                x: (*E).base.x_pos,
                y: (*E).base.y_pos,
            },
            game,
        );
        let v3 = gEnemySpawnData(word!(game.cur_enemy_index) as i32, game);
        (*v3).xpos2 = (pt.x).wrapping_sub((*E).base.x_pos);
        (*v3).ypos2 = (pt.y).wrapping_sub((*E).base.y_pos);
    };
    @external {
        10940404 => [()];
    }
}
pub fn CeresDoor_Func_6b(game: &mut Game) {
    {
        if word!(game.ceres_status) != 0 {
            word!(game.ceres_status = 0x8000);
        }
    }
}
pub unsafe fn CeresDoor_Func_1(k: u16, game: &mut Game) {
    unsafe {
        if (*Get_CeresDoor(k as i32, game)).parameter_1 == 2 {
            let v1: u16 = word!(game.vram_write_queue_tail);
            let mut v2: *mut VramWriteEntry =
                gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
            (*v2).size = 1024;
            *((&raw mut (*v2).src.addr as *mut u8).offset(1) as *mut VoidP) = -20480i32 as VoidP;
            (*v2).src.addr = -15360i32 as VoidP;
            (*v2).vram_dst = 28672;
            word!(game.vram_write_queue_tail = (v1).wrapping_add(7));
        }
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallCeresDoor(ea: u32, game: &mut Game) => ea;
    setup { let k: u16 = word!(game.cur_enemy_index); }
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10942373] CeresDoor_Func_4 [] {
        CeresSteam_Instr_5(word!(game.cur_enemy_index), EnemyScript::Stop, game);
        if word!(game.ceres_status) & 1 != 0 {
            (*Get_CeresDoor(word!(game.cur_enemy_index) as i32, game))
                .base
                .palette_index = 3584;
            CeresDoor_Instr_2(word!(game.cur_enemy_index), EnemyScript::Stop, game);
        }
    };
    [10942544] CeresDoor_Func_7 [] {
        CeresDoor_Func_8(game);
        if word!(game.palette_change_num) == 0 {
            WriteColorsToPalette(
                0x52,
                0xa6,
                (2 * (word!(game.nmi_frame_counter_word) as i32 & 0x38) - 0x78fi32) as u16,
                6,
                game,
            );
        }
    };
    @forward [pub fn (game: &mut Game)];
    [10942315] CeresDoor_Func_2 => CeresDoor_F773(0x14, game);
    [10942320] CeresDoor_Func_3 => CeresDoor_F773(0x1d, game);
    @external {
        10942397 => [CeresDoor_Func_5(k, game)]; 10942428 => [CeresDoor_Func_6(k, game)];
    }
}
pub fn CeresDoor_F773(j: u16, game: &mut Game) {
    {
        if word!(game.ceres_status) >= 2 && word!(game.earthquake_timer) == 0 {
            if (word!(game.random_number) & 0xfff) < 0x80 {
                word! {game; earthquake_timer = 4; earthquake_type = (j).wrapping_add(6); }
            } else {
                word! {game; earthquake_timer = 2; earthquake_type = j; }
            }
        }
    }
}
pub unsafe fn CeresDoor_Func_5(k: u16, game: &mut Game) {
    unsafe {
        CeresDoor_Func_7(game);
        if word!(game.ceres_status) >= 2 {
            let E = Get_CeresDoor(k as i32, game);
            (*E).var_A = (0xa6f7dci32) as u16;
            (*E).var_D = 48;
            let E0 = Get_CeresDoor(0, game);
            set_fields! {[(*E0)], 0; var_E, var_F}
        }
    }
}

macro_rules! set_explosion_spawn_offset {
    ($game:ident, $enemy:ident, $offset_table:literal, $offset_index:ident) => {
        $game.eproj_spawn_pt = Point16U {
            x: (((*$enemy).base.x_pos).wrapping_add(crate::rtl::rom_table_word!(
                $game,
                $offset_table,
                $offset_index as isize
            ))),
            y: (((*$enemy).base.y_pos).wrapping_add(crate::rtl::rom_table_word!(
                $game,
                $offset_table,
                ($offset_index + 1) as isize
            ))),
        };
    };
}
pub(crate) use set_explosion_spawn_offset;

pub unsafe fn CeresDoor_Func_6(k: u16, game: &mut Game) {
    unsafe {
        let E = Get_CeresDoor(k as i32, game);
        (*E).var_D = (*E).var_D.wrapping_sub(1);
        let mut v3: bool = ((*E).var_D & 0x8000) != 0;
        if v3 {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
            (*E).var_A = (0xa6f850i32) as u16;
            CeresDoor_Func_6b(game);
        } else {
            let E0 = Get_CeresDoor(0, game);
            (*E0).var_E = (*E0).var_E.wrapping_sub(1);
            v3 = ((*E0).var_E & 0x8000) != 0;
            if v3 {
                (*E0).var_E = 4;
                (*E0).var_F = (*E0).var_F.wrapping_sub(1);
                v3 = ((*E0).var_F & 0x8000) != 0;
                if v3 {
                    (*E0).var_F = 3;
                }
                let v5: i32 = ((4u16).wrapping_mul((*E0).var_F)) as i32 >> 1;
                set_explosion_spawn_offset!(game, E, 0xa6f840, v5);
                let mut v6: u16 = 3;
                if NextRandom(game) < 0x4000 {
                    v6 = 12;
                }
                SpawnEprojWithRoomGfx(0xe509, v6, game);
                QueueSfx2_Max6(0x25, game);
            }
        };
    }
}
pub unsafe fn CeresDoor_Func_8(game: &mut Game) {
    unsafe {
        QueueMode7Transfers(
            0xa6,
            crate::rtl::rom_table_word!(
                game,
                0xa6f900,
                ((word!(game.nmi_frame_counter_word) as i32 & 2) >> 1) as isize
            ),
            game,
        );
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallZebetitesFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [10943553] Zebetites_Func_1 [] {
        let E = Get_Zebetites(k as i32, game);
        if ((*E).var_F & 0x8000) != 0 {
            let new_k: u16 = Zebetites_Func_9(game);
            (*Get_Zebetites(new_k as i32, game)).parameter_2 = k;
            (*Get_Zebetites(k as i32, game)).parameter_2 = new_k;
        }
        (*Get_Zebetites(k as i32, game)).var_A = (0xa6fc5bi32) as u16;
        Zebetites_Func_2(k, game);
    };
    [10943579] Zebetites_Func_2 [] {
        if word!(game.door_transition_flag_elevator_zebetites) == 0 {
            (*Get_Zebetites(k as i32, game)).var_A = (0xa6fc67i32) as u16;
            Zebetites_Func_3(game);
        }
    };
    @external {
        10943591 => [Zebetites_Func_3(game)];
    }
}
pub unsafe fn Zebetites_Func_3(game: &mut Game) {
    unsafe {
        Zebetites_Func_4(game);
        Zebetites_Func_5(game);
        let E = Get_Zebetites(word!(game.cur_enemy_index) as i32, game);
        let health: u16 = (*E).base.health;
        if health != 0 {
            let mut v3: i16 = ((health) as i16).wrapping_add(1);
            if v3 as i32 - 1000i32 & 0x8000 == 0 {
                v3 = 1000;
            }
            (*E).base.health = v3 as u16;
        } else if (*E).parameter_1 != 0 || {
            Zebetites_Func_6(game);
            (*E).var_D as i32 - 4i32 & 0x8000 == 0
        } {
            ({
                let (_ignored_key, animation) = (word!(game.cur_enemy_index), 0);
                actors::combat::die(
                    &mut Runtime::new(game),
                    actors::api::DeathKind::Normal { animation },
                )
            });
        } else {
            ({
                let (_ignored_key, animation) = (word!(game.cur_enemy_index), 0);
                actors::combat::die(
                    &mut Runtime::new(game),
                    actors::api::DeathKind::Normal { animation },
                )
            });
            Zebetites_Func_8(game);
        };
    }
}
pub unsafe fn Zebetites_Func_6(game: &mut Game) {
    unsafe {
        let E = Get_Zebetites(word!(game.cur_enemy_index) as i32, game);
        let mut v1: i16 = (((*E).var_D) as i16).wrapping_add(1);
        (*E).var_D = v1 as u16;
        let r18: u16 = v1 as u16;
        Zebetites_Func_7(3, (r18 as i32 >> 0 & 1) as u8, game);
        Zebetites_Func_7(4, (r18 as i32 >> 1 & 1) as u8, game);
        Zebetites_Func_7(5, (r18 as i32 >> 2 & 1) as u8, game);
    }
}
pub unsafe fn Zebetites_Func_7(j: u16, a: u8, game: &mut Game) {
    if a != 0 {
        SetEventHappened(j, game);
    } else {
        ClearEventHappened(j, game);
    };
}
pub unsafe fn Zebetites_Func_8(game: &mut Game) {
    {
        ({
            let (bank, population) = (0xa6, addr_stru_A6FCE1 as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        });
    }
}
pub unsafe fn Zebetites_Func_9(game: &mut Game) -> u16 {
    {
        {
            let (bank, population) = (0xa6, addr_stru_A6FCF9 as u16);
            actors::population::spawn_legacy(&mut Runtime::new(game), bank, population)
        }
    }
}
pub unsafe fn Zebetites_Func_5(game: &mut Game) {
    unsafe {
        let mut v0: u16 = 0;
        let E = Get_Zebetites(word!(game.cur_enemy_index) as i32, game);
        let health: u16 = (*E).base.health;
        if health as i32 - 800i32 & 0x8000 != 0 {
            v0 = 2;
            if health as i32 - 600i32 & 0x8000 != 0 {
                v0 = 4;
                if health as i32 - 400i32 & 0x8000 != 0 {
                    v0 = 6;
                    if health as i32 - 200i32 & 0x8000 != 0 {
                        v0 = 8;
                    }
                }
            }
        }
        let v3: i32 = v0 as i32 >> 1;
        let mut v4: u16 = crate::rtl::rom_table_word!(game, 0xa6fd4a, v3 as isize);
        if ((*E).var_F & 0x8000) != 0 {
            v4 = crate::rtl::rom_table_word!(game, 0xa6fd54, v3 as isize);
        }
        enemy_instruction! {[(*E).base]; instruction = v4, timer = 1}
        (*E).base.timer = 0;
    }
}
pub unsafe fn Zebetites_Func_4(game: &mut Game) {
    unsafe {
        if word!(game.palette_change_num) == 0
            && (*Get_Zebetites(word!(game.cur_enemy_index) as i32, game)).parameter_1 == 0
        {
            let E = Get_Zebetites(0, game);
            let v1: u16 = ((*E).var_C as i32 + 1 & 7) as u16;
            (*E).var_C = v1;
            WriteColorsToPalette(0x158, 0xa6, (4 * v1 as i32 - 0x279i32) as u16, 2, game);
        }
    }
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10914693 | 10914713] SpikeyPlatform2ndEnemy_Init [] {
        let v0: i32 = word!(game.cur_enemy_index) as i32 >> 1;
        let v1 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
        (*v1).x_pos = word!(game.enemy_drawing_queue[(v0 + 91)]);
        (*v1).y_pos = (word!(game.enemy_drawing_queue[(v0 + 93)])).wrapping_add(12);
    };
    [10915836] FireGeyser_Init [] {
        let E = Get_FireGeyser(word!(game.cur_enemy_index) as i32, game);
        (*E).base.current_instruction = addr_kFireGeyser_Ilist_8DA9 as u16;
        if (*E).parameter_2 == 0 {
            (*E).base.current_instruction = addr_kFireGeyser_Ilist_8D1B as u16;
            (*E).var_A = (0xa6902fi32) as u16;
            (*E).var_D = (*E).base.y_pos;
            (*E).base.x_width = 0;
        }
    };
    [10915875] FireGeyser_Main [] {
        let E = Get_FireGeyser(word!(game.cur_enemy_index) as i32, game);
        if (*E).parameter_2 == 0 {
            CallFireGeyserFunc((((*E).var_A) as u32) | (0xa60000u32), game);
        }
    };
    [10917060] NuclearWaffle_Init [] {
        let E = Get_NuclearWaffle(word!(game.cur_enemy_index) as i32, game);
        (*E).base.current_instruction = addr_kNuclearWaffle_Ilist_9490 as u16;
        (*E).var_C = byte!((*E).parameter_1; 0) as u16;
        (*E).var_D = byte!((*E).parameter_1; 1) as u16;
        (*E).var_E = byte!((*E).parameter_2; 0) as u16;
        let nwe_parameter_2_high: u16 = byte!((*E).parameter_2; 1) as u16;
        (*E).var_F = nwe_parameter_2_high;
        (*E).var_B = nwe_parameter_2_high;
        (*E).var_A = (0xa69615i32) as u16;
        let v2: i32 = ((4u16).wrapping_mul((*E).var_E)) as i32 >> 1;
        let v3: u16 = g_word_A695F6[v2 as usize];
        set_fields! {[(*E)]; var_23 = v3, var_21 = v3, var_2E = g_word_A695F6[(v2 + 1) as usize],
        var_2B = g_word_A695FE[v2 as usize], var_2C = g_word_A695FE[(v2 + 1) as usize],
        var_30 = g_word_A69606[v2 as usize], var_2F = g_word_A69606[(v2 + 1) as usize], };
        let mut v4: u16 = (8u16).wrapping_mul((*E).var_C);
        if (*E).var_E == 0 {
            v4 = (v4).wrapping_add(4);
        }
        let v5: i32 = v4 as i32 >> 1;
        (*E).var_25 = crate::rtl::rom_table_word!(game, 0xa28187, v5 as isize);
        (*E).var_24 = crate::rtl::rom_table_word!(game, 0xa28187, (v5 + 1) as isize);
        (*E).var_26 = (*E).base.x_pos;
        (*E).var_27 = (*E).base.y_pos;
        let v6: u16 = ((*E).var_26).wrapping_add(CosineMult8bit((*E).var_23, (*E).var_D, game));
        (*E).var_28 = v6;
        (*E).base.x_pos = v6;
        let v7: u16 = ((*E).var_27).wrapping_add(SineMult8bit((*E).var_23, (*E).var_D, game));
        (*E).var_29 = v7;
        (*E).base.y_pos = v7;
        let mut v8: u16 = 8;
        (*E).var_2A = 8;
        do_until! { v8 == 0 ;
            SpawnEprojWithGfx(
                v8,
                word!(game.cur_enemy_index),
                addr_kEproj_NuclearWaffleBody as u16,
                game,
            );
            v8 = ((*E).var_2A).wrapping_sub(2);
            (*E).var_2A = v8;
        }
        (*E).var_2A = 6;
        do_until! { v15 == 0 ;
            let v13: u16 = ((*E).base.vram_tiles_index) | ((*E).base.palette_index);
            (*E).var_34 = v13;
            let r18: u16 = create_sprite((*E).base.x_pos, (*E).base.y_pos, 43, v13, game);
            (*Get_NuclearWaffle(
                word!(game.cur_enemy_index) as i32 + (*E).var_2A as i32,
                game,
            ))
            .var_10 = r18;
            let mut v15: u16 = ((*E).var_2A).wrapping_sub(2);
            (*E).var_2A = v15;
        }
    };
    [10917390] NuclearWaffle_Main [] {
        let E = Get_NuclearWaffle(word!(game.cur_enemy_index) as i32, game);
        CallNuclearWaffleFunc((((*E).var_A) as u32) | (0xa60000u32), game);
    };
    [10918488] FakeKraid_Init [] {
        let mut v0: u16 = ((word!(game.random_number) as i32 & 3) + 2) as u16;
        let E = Get_FakeKraid(word!(game.cur_enemy_index) as i32, game);
        (*E).var_D = v0;
        (*E).var_E = v0;
        v0 = (v0).wrapping_add(64);
        (*E).var_03 = v0;
        v0 = (v0).wrapping_add(32);
        (*E).var_04 = v0;
        (*E).var_05 = (v0).wrapping_sub(48);
        (*E).var_07 = 0;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).var_B = -4i32 as u16;
        (*E).var_C = -4i32 as u16;
        let mut v2: u16 = addr_stru_A699AE as u16;
        if (((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) < 0 {
            set_fields! {[(*E)], 4; var_B, var_C}
            v2 = addr_stru_A699FC as u16;
        }
        (*E).base.current_instruction = v2;
    };
    [10918594] FakeKraid_Main [] {
        let E = Get_FakeKraid(word!(game.cur_enemy_index) as i32, game);
        let fkd_var_07: u16 = (*E).var_07;
        let mut v3: u16 = (fkd_var_07).wrapping_add(2);
        if fkd_var_07 as i32 - 4i32 & 0x8000 == 0 {
            v3 = 0;
        }
        (*E).var_07 = v3;
        FakeKraid_Func_1(word!(game.cur_enemy_index), fkd_var_07, game);
    };
    [10918946] FakeKraid_Touch [] {
        let E = Get_FakeKraid(word!(game.cur_enemy_index) as i32, game);
        word!(game.special_death_item_drop_x_origin_pos = (*E).base.x_pos);
        word!(game.special_death_item_drop_y_origin_pos = (*E).base.y_pos);
        actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
        FakeKraid_9C50(game);
    };
    [10918969] FakeKraid_Shot [] {
        let E = Get_FakeKraid(word!(game.cur_enemy_index) as i32, game);
        word!(game.special_death_item_drop_x_origin_pos = (*E).base.x_pos);
        word!(game.special_death_item_drop_y_origin_pos = (*E).base.y_pos);
        actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::ReactionOnly);
        FakeKraid_9C50(game);
    };
    [10920659] CeresRidley_Hurt [] {
        Ridley_Func_102(game);
        sub_A6DB2A(game);
        Ridley_Func_120(game);
        Ridley_A2DC(game);
    };
    [10929814] RidleysExplosion_Init [] {
        let E = Get_Ridley(word!(game.cur_enemy_index) as i32, game);
        (*E).base.instruction_timer = 1;
        set_fields! {[(*E).base], 0; timer, vram_tiles_index}
        (*E).base.palette_index = 3584;
        let ridley_parameter_1: u16 = (*E).parameter_1;
        (*E).var_F =
            crate::rtl::rom_table_word!(game, 0xa6c6ce, (ridley_parameter_1 >> 1) as isize);
        let Random: u16 = NextRandom(game);
        (*E).var_B = (if Random & 0x8000 != 0 {
            -(Random as i32 & 0x130i32)
        } else {
            Random as i32 & 0x130
        }) as u16;
        (*E).var_C = 0;
        if ridley_parameter_1 != 0 {
            match ridley_parameter_1 as i32 {
                2 => {
                    (*E).base.x_pos = word!(game.tilemap_stuff[32]);
                    (*E).base.y_pos = word!(game.tilemap_stuff[33]);
                    (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA47 as u16;
                }
                4 => {
                    (*E).base.x_pos = word!(game.tilemap_stuff[42]);
                    (*E).base.y_pos = word!(game.tilemap_stuff[43]);
                    (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA4D as u16;
                }
                6 => {
                    (*E).base.x_pos = word!(game.tilemap_stuff[52]);
                    (*E).base.y_pos = word!(game.tilemap_stuff[53]);
                    (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA4D as u16;
                }
                8 => {
                    (*E).base.x_pos = word!(game.tilemap_stuff[62]);
                    (*E).base.y_pos = word!(game.tilemap_stuff[63]);
                    (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA53 as u16;
                }
                10 => {
                    (*E).base.x_pos = word!(game.tilemap_stuff[72]);
                    (*E).base.y_pos = word!(game.tilemap_stuff[73]);
                    (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA53 as u16;
                }
                12 => {
                    (*E).base.x_pos = word!(game.tilemap_stuff[82]);
                    (*E).base.y_pos = word!(game.tilemap_stuff[83]);
                    (*E).base.current_instruction = *(rom_ptr!(game, u16; 0xa6c7ba)).offset(
                        ((byte!(@pointer (ram_ptr!(game, u16; 0x2000)).offset(71); 0) as i32
                            + byte!(@pointer (ram_ptr!(game, u16; 0x2000)).offset(81); 0) as i32
                            + 8
                            & 0xf0) as u8
                            >> 4) as isize,
                    );
                }
                14 => {
                    place_ridley_explosion_x!(E0, game, v13, 0xa6c804, E);
                    (*E).base.y_pos = (*E0).base.y_pos;
                    (*E).base.current_instruction =
                        crate::rtl::rom_table_word!(game, 0xa6c808, v13 as isize);
                }
                16 => {
                    place_ridley_explosion_x!(E0_0, game, v19, 0xa6c836, E);
                    (*E).base.y_pos = ((*E0_0).base.y_pos).wrapping_add(22);
                    (*E).base.current_instruction =
                        crate::rtl::rom_table_word!(game, 0xa6c83a, v19 as isize);
                }
                18 => {
                    place_ridley_explosion_x!(E0_1, game, v25, 0xa6c868, E);
                    (*E).base.y_pos = ((*E0_1).base.y_pos).wrapping_sub(24);
                    (*E).base.current_instruction =
                        crate::rtl::rom_table_word!(game, 0xa6c86c, v25 as isize);
                }
                20 => {
                    place_ridley_explosion_x!(E0_2, game, v31, 0xa6c89a, E);
                    (*E).base.y_pos = (*E0_2).base.y_pos;
                    (*E).base.current_instruction =
                        crate::rtl::rom_table_word!(game, 0xa6c89e, v31 as isize);
                }
                22 => {
                    place_ridley_explosion_x!(E0_3, game, v37, 0xa6c8cc, E);
                    (*E).base.y_pos = ((*E0_3).base.y_pos).wrapping_add(7);
                    (*E).base.current_instruction =
                        crate::rtl::rom_table_word!(game, 0xa6c8d0, v37 as isize);
                }
                _ => {
                    Unreachable(game);
                    loop {}
                }
            }
        } else {
            (*E).base.x_pos = word!(game.tilemap_stuff[22]);
            (*E).base.y_pos = word!(game.tilemap_stuff[23]);
            (*E).base.current_instruction = addr_kRidleysExplosion_Ilist_CA47 as u16;
        };
    };
    [10930388] RidleysExplosion_Main [] {
        RidleysExplosion_Func_1(word!(game.cur_enemy_index), game);
        let E = Get_RidleysExplosion(word!(game.cur_enemy_index) as i32, game);
        let mut v2: i16 = ((abs16((*E).var_B)) as i16).wrapping_sub(4);
        if v2 < 0 {
            v2 = 0;
        }
        (*E).var_B = (if (*E).var_B & 0x8000 != 0 {
            -(v2 as i32)
        } else {
            v2 as i32
        }) as u16;
        (*E).var_C = ((*E).var_C).wrapping_add(4);
        MoveEnemyWithVelocity(game);
        (*E).var_F = (*E).var_F.wrapping_sub(1);
        if ((*E).var_F & 0x8000) != 0 {
            ({ let (_ignored_key, animation) = (word!(game.cur_enemy_index), 0); actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Normal { animation }) });
        }
    };
    [10936202] Ridley_Shot [] {
        if word!(game.area_index) == 2 {
            actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::ReactionOnly);
        } else {
            let mut v0: i16 = 13;
            let E = Get_Ridley(0, game);
            let flash_timer: u16 = (*E).base.flash_timer;
            if flash_timer != 0 && flash_timer & 1 != 0 {
                v0 = 14;
            }
            (*E).base.flash_timer = v0 as u16;
            (*E).var_0D = (*E).var_0D.wrapping_add(1);
        };
    };
    [10936242] Ridley_Powerbomb [] {
        actors::combat::power_bomb_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
    };
    [10940337] CeresSteam_Init [] {
        let E = Get_CeresSteam(word!(game.cur_enemy_index) as i32, game);
        set_fields! {[(*E).base]; vram_tiles_index = 0,
        properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16),
        extra_properties = ((*E).base.extra_properties) | (4u16), instruction_timer = 1,
        timer = 0, palette_index = 2560, };
        (*E).var_D = ((NextRandom(game) as i32 & 0x1f) + 1) as u16;
        let v1: i32 = (*E).parameter_1 as i32;
        (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa6eff5, v1 as isize);
        (*E).var_A = crate::rtl::rom_table_word!(game, 0xa6f001, v1 as isize);
    };
    [10940429] CeresSteam_Main [] {
        let E = Get_CeresSteam(word!(game.cur_enemy_index) as i32, game);
        (*E).base.health = 0x7fff;
        CallCeresSteamFunc(
            (((*E).var_A) as u32) | (0xa60000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    [10940479] CeresSteam_Touch [] {
        (*Get_CeresSteam(word!(game.cur_enemy_index) as i32, game))
            .base
            .health = 0x7fff;
        actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    };
    [10942149] CeresDoor_Init [] {
        let E = Get_CeresDoor(word!(game.cur_enemy_index) as i32, game);
        (*E).base.spritemap_pointer = addr_kCeresDoor_Sprmap_FAC7 as u16;
        (*E).base.instruction_timer = 1;
        set_fields! {[(*E).base], 0; timer, vram_tiles_index}
        (*E).base.palette_index = 1024;
        let v2: i32 = (*E).parameter_1 as i32;
        (*E).var_A = crate::rtl::rom_table_word!(game, 0xa6f72b, v2 as isize);
        (*E).base.current_instruction = crate::rtl::rom_table_word!(game, 0xa6f52c, v2 as isize);
        (*E).var_B = 0;
        CeresDoor_Func_1(word!(game.cur_enemy_index), game);
        let mut v3: u16 = if word!(game.ceres_status) != 0 {
            (addr_word_A6F50C as i32 + 2) as u16
        } else {
            if (*E).parameter_1 == 3 {
                WriteColorsToTargetPalette(
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
        (*Get_CeresDoor(word!(game.cur_enemy_index) as i32, game))
            .base
            .palette_index = 3584;
        WriteColorsToTargetPalette(0xa6, 0x1e2, v3, 0xf, game);
    };
    [10942309] CeresDoor_Main [] {
        let E = Get_CeresDoor(word!(game.cur_enemy_index) as i32, game);
        CallCeresDoor((((*E).var_A) as u32) | (0xa60000u32), game);
    };
    [10943346] Zebetites_Init [] {
        let E = Get_Zebetites(word!(game.cur_enemy_index) as i32, game);
        set_fields! {[(*E).base]; properties = ((*E).base.properties) | (0xa000u16),
        instruction_timer = 1, timer = 0, palette_index = 1024, vram_tiles_index = 128, };
        (*E).var_C = 0;
        let mut v1: u16 = (0xa6fc41i32) as u16;
        if (*E).parameter_1 != 0 {
            v1 = (0xa6fc5bi32) as u16;
        }
        (*E).var_A = v1;
        let mut r18: u16 = 0;
        r18 = ((r18 as i32) << 1 | CheckEventHappened(5, game) as i32) as u16;
        r18 = ((r18 as i32) << 1 | CheckEventHappened(4, game) as i32) as u16;
        r18 = ((r18 as i32) << 1 | CheckEventHappened(3, game) as i32) as u16;
        let v2: u16 = r18;
        (*E).var_D = r18;
        if v2 as i32 - 4i32 & 0x8000 != 0 {
            let v3: i32 = v2 as i32;
            (*E).var_F = crate::rtl::rom_table_word!(game, 0xa6fc03, v3 as isize);
            (*E).base.y_height = crate::rtl::rom_table_word!(game, 0xa6fc0b, v3 as isize);
            (*E).base.current_instruction =
                crate::rtl::rom_table_word!(game, 0xa6fc13, v3 as isize);
            (*E).base.x_pos = crate::rtl::rom_table_word!(game, 0xa6fc1b, v3 as isize);
            let mut v4: u16 = if (*E).parameter_1 != 0 {
                crate::rtl::rom_table_word!(game, 0xa6fc2b, v3 as isize)
            } else {
                crate::rtl::rom_table_word!(game, 0xa6fc23, v3 as isize)
            };
            (*E).base.y_pos = v4;
        } else {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Deleted) as u16);
        };
    };
    [10943539] Zebetites_Main [] {
        if word!(game.earthquake_timer) == 0 {
            (*Get_Zebetites(word!(game.cur_enemy_index) as i32, game))
                .base
                .shake_timer = 0;
        }
        let E = Get_Zebetites(word!(game.cur_enemy_index) as i32, game);
        CallZebetitesFunc(
            (((*E).var_A) as u32) | (0xa60000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    [10943911] Zebetites_Touch [] {
        actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
    };
    [10943916] Zebetites_Shot [] {
        QueueSfx3_Max6(9, game);
        actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::ReactionOnly);
        let E = Get_Zebetites(word!(game.cur_enemy_index) as i32, game);
        let G = Get_Zebetites((*E).parameter_2 as i32, game);
        (*G).base.health = (*E).base.health;
        (*G).base.flash_timer = (*E).base.flash_timer;
    };
    @forward [pub unsafe fn (game: &mut Game)];
    [10911759] Enemy_GrappleReact_CancelBeam_A6 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [10911779] Enemy_NormalTouchAI_A6 => actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    [10911789] Enemy_NormalShotAI_A6 => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
    [10911809] Enemy_NormalFrozenAI_A6 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10915247] FireGeyser_Instr_1 [jp] { QueueSfx2_Max6(0x61, game); };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10915347]
    FireGeyser_Instr_2 [jp] { set_fire_geyser_shape(g_word_A68DBB, g_word_A68DE7, true, game); };
    [10915373]
    FireGeyser_Instr_3 [jp] { set_fire_geyser_shape(g_word_A68DBD, g_word_A68DE9, false, game); };
    [10915393]
    FireGeyser_Instr_4 [jp] { set_fire_geyser_shape(g_word_A68DBF, g_word_A68DEB, false, game); };
    [10915413]
    FireGeyser_Instr_5 [jp] { set_fire_geyser_shape(g_word_A68DC1, g_word_A68DED, false, game); };
    [10915433]
    FireGeyser_Instr_6 [jp] { set_fire_geyser_shape(g_word_A68DC3, g_word_A68DEF, false, game); };
    [10915453]
    FireGeyser_Instr_7 [jp] { set_fire_geyser_shape(g_word_A68DC5, g_word_A68DF1, false, game); };
    [10915473]
    FireGeyser_Instr_8 [jp] { set_fire_geyser_shape(g_word_A68DC7, g_word_A68DF3, false, game); };
    [10915493]
    FireGeyser_Instr_9 [jp] { set_fire_geyser_shape(g_word_A68DC9, g_word_A68DF5, false, game); };
    [10915513]
    FireGeyser_Instr_10 [jp] { set_fire_geyser_shape(g_word_A68DCB, g_word_A68DF7, false, game); };
    [10915533]
    FireGeyser_Instr_11 [jp] { set_fire_geyser_shape(g_word_A68DCD, g_word_A68DF9, false, game); };
    [10915553]
    FireGeyser_Instr_12 [jp] { set_fire_geyser_shape(g_word_A68DCF, g_word_A68DFB, false, game); };
    [10915573]
    FireGeyser_Instr_13 [jp] { set_fire_geyser_shape(g_word_A68DD1, g_word_A68DFD, false, game); };
    [10915593]
    FireGeyser_Instr_14 [jp] { set_fire_geyser_shape(g_word_A68DD3, g_word_A68DFF, false, game); };
    [10915613]
    FireGeyser_Instr_15 [jp] { set_fire_geyser_shape(g_word_A68DD5, g_word_A68E01, false, game); };
    [10915633]
    FireGeyser_Instr_16 [jp] { set_fire_geyser_shape(g_word_A68DD7, g_word_A68E03, false, game); };
    [10915653]
    FireGeyser_Instr_17 [jp] { set_fire_geyser_shape(g_word_A68DD9, g_word_A68E05, false, game); };
    [10915673]
    FireGeyser_Instr_18 [jp] { set_fire_geyser_shape(g_word_A68DDB, g_word_A68E07, false, game); };
    [10915693]
    FireGeyser_Instr_19 [jp] { set_fire_geyser_shape(g_word_A68DDD, g_word_A68E09, false, game); };
    [10915713]
    FireGeyser_Instr_20 [jp] { set_fire_geyser_shape(g_word_A68DDF, g_word_A68E0B, false, game); };
    [10915733]
    FireGeyser_Instr_21 [jp] { set_fire_geyser_shape(g_word_A68DE1, g_word_A68E0D, false, game); };
    [10915753]
    FireGeyser_Instr_22 [jp] { set_fire_geyser_shape(g_word_A68DE3, g_word_A68E0F, false, game); };
    [10915773]
    FireGeyser_Instr_23 [jp] { set_fire_geyser_shape(g_word_A68DE5, g_word_A68E11, false, game); };
    [10915793] FireGeyser_Instr_24 [jp] {
        let E = Get_FireGeyser(word!(game.cur_enemy_index) as i32, game);
        (*E).var_C = 1;
        let E1 = Get_FireGeyser(word!(game.cur_enemy_index) as i32 + 64, game);
        set_fields! {[(*E1).base], 0; x_width, y_height}
        (*E).base.y_pos = (*E).var_D;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        (*E1).base.properties = ((*E1).base.properties) | (0x400u16);
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10918694] FakeKraid_Instr_2 [jp] {
        let E = Get_FakeKraid(word!(game.cur_enemy_index) as i32, game);
        if (*E).var_E != 0 {
            (*E).var_E = (*E).var_E.wrapping_sub(1);
        }
        crate::update_place! { previous fresh1 = (*E).var_D => (*E).var_D.wrapping_sub(1); }
        let reverse_direction: bool = if fresh1 == 1 {
            (*E).var_D = ((word!(game.random_number) as i32 & 3) + 7) as u16;
            true
        } else {
            !(({ let (actor, amount) = (word!(game.cur_enemy_index), ((*E).var_B as i16 as i32) << 16); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) == 0)
        };
        if reverse_direction {
            (*E).var_B = -((*E).var_B as i32) as u16;
        }
        (*E).var_C = -4i32 as u16;
        if (((*E).base.x_pos) as i16).wrapping_sub((word!(game.samus_x_pos)) as i16) < 0 {
            (*E).var_C = 4;
        }
    };
    [10918772] FakeKraid_Instr_1 [] {
        let E = Get_FakeKraid(word!(game.cur_enemy_index) as i32, game);
        if (*E).var_E != 0 {
            if ((*E).var_C & 0x8000) != 0 {
                if ((*Get_FakeKraid(word!(game.cur_enemy_index) as i32, game)).var_B & 0x8000) == 0
                {
                    return EnemyScript::jump(addr_stru_A699C6 as u16);
                }
                EnemyScript::jump(addr_stru_A699AE as u16)
            } else {
                if ((*E).var_B & 0x8000) != 0 {
                    return EnemyScript::jump(addr_stru_A69A14 as u16);
                }
                EnemyScript::jump(addr_stru_A699FC as u16)
            }
        } else {
            (*E).var_E = ((word!(game.random_number) as i32 & 3) + 3) as u16;
            if ((*E).var_C & 0x8000) != 0 {
                return EnemyScript::jump(addr_kFakeKraid_Ilist_99DC as u16);
            }
            EnemyScript::jump(addr_kFakeKraid_Ilist_9A2A as u16)
        }
    };
    [10918834] FakeKraid_Instr_3 [jp] {
        if u16::from(actor_api::current_outside_screen(&Runtime::new(game))) == 0 {
            QueueSfx2_Max6(0x16, game);
        }
    };
    [10918852] FakeKraid_Instr_4 [jp] { FakeKraid_InstrHelper_45(k, 0, 0xfffc, game); };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10918914] FakeKraid_Instr_5 [jp] { FakeKraid_InstrHelper_45(k, 8, 4, game); };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10937534] Ridley_Instr_5 [jp] {
        (*Get_Ridley(0, game)).var_0F = 89;
        QueueSfx2_Max6(0x59, game);
    };
    [10937546] Ridley_Instr_6 [jp] { (*Get_Ridley(0, game)).var_0F = 0; };
    [10937554] Ridley_Instr_10 [EnemyScript::jump(jp.word(0))] {
        if word!(game.area_index) == 2
            || word!(game.samus_health) as i32 - 30i32 & 0x8000 == 0
        {
            return jp.offset(1);
        }
        (*Get_Ridley(0, game)).var_00 = 8;
    };
    [10937582] Ridley_Instr_4 [] {
        if (*Get_Ridley(0, game)).var_1B != 0 {
            EnemyScript::jump(jp.word(0))
        } else {
            EnemyScript::jump(jp.word(2))
        }
    };
    [10937592] Ridley_Instr_3 [] {
        if (*Get_Ridley(0, game)).var_1B != 0 {
            jp.offset(1)
        } else {
            EnemyScript::jump(jp.word(0))
        }
    };
    [10937601] Ridley_Instr_2 [jp.offset(1)] {
        word!(game.samus_prev_x_pos = word!(game.samus_x_pos));
        word!(game.samus_prev_y_pos = word!(game.samus_y_pos));
        (*Get_Ridley(0, game)).var_1D = jp.word(0);
    };
    [10937623] Ridley_Instr_1 [] {
        if (*Get_Ridley(0, game)).var_10 != 0 {
            EnemyScript::jump(jp.word(0))
        } else {
            jp.offset(1)
        }
    };
    [10937631] Ridley_Instr_14 [jp.offset(2)] {
        let E = Get_Ridley(0, game);
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(jp.word(0));
        (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(jp.word(2));
    };
    [10938140] Ridley_Instr_9 [jp] { (*Get_Ridley(0, game)).var_10 = 0; Ridley_Func_96(game); };
    [10938151] Ridley_Instr_7 [jp] { (*Get_Ridley(0, game)).var_10 = 1; };
    [10938159] Ridley_Instr_8 [jp] { (*Get_Ridley(0, game)).var_10 = 2; Ridley_Func_96(game); };
    [10938445] Ridley_Instr_11 [jp] {
        let E = Get_Ridley(0, game);
        let mut r18: u16 = if (*E).var_10 != 0 {
            let mut v4: u16 = -(actor_api::angle_from_delta((word!(game.samus_x_pos) as i32 - ((*E).base.x_pos as i32 + 25)) as u16, (word!(game.samus_y_pos) as i32 - ((*E).base.y_pos as i32 - 43i32)) as u16) as i32
                + 0x80) as u8 as u16;
            if v4 < 0x15 || v4 >= 0xc0 {
                v4 = 0x15;
            } else if v4 >= 0x50 {
                v4 = 0x50;
            }
            v4
        } else {
            let mut v3: u16 = -(actor_api::angle_from_delta((word!(game.samus_x_pos) as i32 - ((*E).base.x_pos as i32 - 25i32)) as u16, (word!(game.samus_y_pos) as i32 - ((*E).base.y_pos as i32 - 43i32)) as u16) as i32
                + 0x80) as u8 as u16;
            if v3 < 0x40 || v3 >= 0xeb {
                v3 = 0xeb;
            } else if v3 < 0xb0 {
                v3 = 0xb0;
            }
            v3
        };
        (*E).var_19 = Math_MultBySin(0x500, r18);
        (*E).var_1A = Math_MultByCos(0x500, r18);
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    @prefix { let E = Get_Ridley(0, game); };
    [10938729]
    Ridley_Instr_15 [jp] {  (*E).var_A = (0xa6a6afi32) as u16; (*E).var_C = -352i32 as u16; };
    [10938742]
    Ridley_Instr_16 [jp] {  (*E).var_A = (0xa6b2f3i32) as u16; (*E).var_C = -352i32 as u16; };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10940701] CeresSteam_Instr_1 [jp] {
        let E = Get_CeresSteam(k as i32, game);
        (*E).base.properties = ((*E).base.properties as i32
            | (kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
            as u16;
    };
    [10940711]
    CeresSteam_Instr_2 [CeresSteam_Instr_3(k, { let record = crate::rtl::rom_record(0xa6, jp.word(2), game); EnemyScript::at(record.image, record.base) }, game)] {
        let E = Get_CeresSteam(k as i32, game);
        crate::update_place! { previous fresh0 = (*E).var_D => (*E).var_D.wrapping_sub(1); }
        if fresh0 != 1 {
            return EnemyScript::jump(jp.word(0));
        }
    };
    [10940725] CeresSteam_Instr_3 [jp] {
        let E = Get_CeresSteam(k as i32, game);
        (*E).base.properties = ((*E).base.properties) & (0xfaffu16);
    };
    [10942014] CeresDoor_Instr_6 [] {
        let E = Get_CeresDoor(k as i32, game);
        if abs16(((*E).base.x_pos).wrapping_sub(word!(game.samus_x_pos))) >= 0x30
            || abs16(((*E).base.y_pos).wrapping_sub(word!(game.samus_y_pos))) >= 0x30
        {
            EnemyScript::jump(jp.word(0))
        } else {
            jp.offset(1)
        }
    };
    [10942058] CeresDoor_Instr_4 [] {
        if gameplay_flag_runtime::boss_word_at(word!(game.area_index), BossWordOrder::Native, game) & 1 != 0 {
            jp.offset(1)
        } else {
            EnemyScript::jump(jp.word(0))
        }
    };
    [10942072] CeresDoor_Instr_8 [] {
        if word!(game.ceres_status) != 0 {
            jp.offset(1)
        } else {
            EnemyScript::jump(jp.word(0))
        }
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10942091] CeresSteam_Instr_4 [jp] {
        let E = Get_CeresSteam(k as i32, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
    };
    [10942101] CeresDoor_Instr_1 [jp] {
        let E = Get_CeresDoor(k as i32, game);
        (*E).base.properties = ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
    };
    [10942111] CeresDoor_Instr_3 [jp] { (*Get_CeresDoor(k as i32, game)).var_B = 1; };
    [10942118] CeresSteam_Instr_5 [jp] {
        let E = Get_CeresSteam(k as i32, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    };
    [10942128] CeresDoor_Instr_5 [CeresDoor_Instr_2(k, jp, game)] {
        (*Get_CeresDoor(k as i32, game)).var_B = 0;
    };
    [10942131] CeresDoor_Instr_2 [jp] {
        let E = Get_CeresDoor(k as i32, game);
        (*E).base.properties = ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
    };
    [10942141] CeresDoor_Instr_7 [jp] { QueueSfx3_Max6(0x2c, game); };
    @forward [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10938628] Ridley_Instr_12 => Ridley_E90C(k, jp, 0, game);
    [10938633] Ridley_Instr_13 => Ridley_E90C(k, jp, 0xe, game);
}
