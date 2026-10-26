use crate::game::actors;
use crate::game::actors::access::ActorKey as Key;
use crate::game::actors::api as actor_api;
use crate::game::decompression::decompress_to_ram;
use crate::game::enemy_projectiles::{DropSource, scatter_drop as scatter};
use crate::game::enemy_script::EnemyScript;
use crate::game::gameplay_flag_host::BossWordOrder;
use crate::game::gameplay_flag_runtime;
use crate::game::music::QueueMusic_Delayed8;
pub use crate::game::prelude::*;
use crate::game::session_runtime::{Runtime, actor_species_pre_instruction};
use crate::rtl::rom_record;
use core::mem::offset_of;

pub use crate::game::constants::bank_86::*;

use crate::game::display_refresh::GraphicsAsset;
use crate::game::enemy_layouts::enemy_layout;
use crate::game::enemy_projectiles::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::music::{QueueSfx1_Max6, QueueSfx2_Max6, QueueSfx2_Max15, QueueSfx3_Max6};
use crate::game::palette_effects::fading::background6;
use crate::game::player_update::minimap;
use crate::game::room_mechanisms::spawn_hardcoded;
use crate::game::sprite_effect_runtime::create_sprite;
use crate::game::visual_runtime::initialize_phantoon_motion;
use crate::game::visual_runtime::spawn_hdma;

crate::typed_struct! {
    [
        #[derive(Copy, Clone)]
        #[repr(C, packed)]
    ];
    pub struct SpriteDrawInstr {
        pub field_0: u16;
        pub field_2: VoidP;
    }

    pub struct KraidInstrList {
        pub timer: u16;
        pub tilemap, vuln_mouth_hitbox, invuln_mouth_hitbox: VoidP;
    }

    pub struct KraidSinkTable {
        pub field_0, field_2: u16;
        pub field_4: VoidP;
    }
}

pub use crate::types::Consts_A7;

crate::game::constants::typed_constants! { Consts_A7;
    addr_kDachora_Ilist_F4C1 = 62657, addr_kDachora_Ilist_F4B9 = 62649,
    addr_kDachora_Ilist_F4B3 = 62643, addr_kDachora_Ilist_F48B = 62603,
    addr_kDachora_Ilist_F45B = 62555, addr_kDachora_Ilist_F407 = 62471,
    addr_kDachora_Ilist_F3FF = 62463, addr_kDachora_Ilist_F3F7 = 62455,
    addr_kDachora_Ilist_F3C9 = 62409, addr_kDachora_Ilist_F399 = 62361,
    addr_kDachora_Ilist_F345 = 62277, addr_kDachora_Palette = 61989,
    addr_kEtecoon_Ilist_E8D6 = 59606, addr_kEtecoon_Ilist_E8CE = 59598,
    addr_kEtecoon_Ilist_E8C8 = 59592, addr_kEtecoon_Ilist_E8AC = 59564,
    addr_stru_A7E898 = 59544, addr_kEtecoon_Ilist_E894 = 59540,
    addr_stru_A7E880 = 59520, addr_kEtecoon_Ilist_E876 = 59510,
    addr_kEtecoon_Ilist_E870 = 59504, addr_stru_A7E862 = 59490,
    addr_kEtecoon_Ilist_E854 = 59476, addr_kEtecoon_Ilist_E83C = 59452,
    addr_stru_A7E828 = 59432, addr_kEtecoon_Ilist_E81E = 59422,
    addr_kPhantoonMoveData = 58322, addr_kKraid_Ilist_CCEB = 52459,
    addr_kKraid_Ilist_CC9D = 52381, addr_kKraid_Ilist_CC91 = 52369,
    addr_kKraid_Ilist_CC81 = 52353, addr_kKraid_Ilist_CC53 = 52307,
    addr_kKraid_Ilist_CC4D = 52301, addr_kKraid_Ilist_CC47 = 52295,
    addr_kKraid_Ilist_CC41 = 52289, addr_locret_A7B831 = 47153,
    addr_kKraidsRoomBg = 42774, addr_kKraid_Sprmap_A5DF = 42463,
    addr_kKraidTilemaps_3 = 41160, addr_kKraidTilemaps_2 = 40392,
    addr_kKraidTilemaps_1 = 39624, addr_kKraidTilemaps_0 = 38856,
    addr_stru_A79764 = 38756, addr_stru_A7974A = 38730,
    addr_stru_A796DA = 38618, addr_kKraid_Sprmap_8C6C = 35948,
    addr_kKraid_Ilist_8B0A = 35594, addr_kKraid_Ilist_8B04 = 35588,
    addr_kKraid_Ilist_8AFE = 35582, addr_kKraid_Ilist_8AF0 = 35568,
    addr_kKraid_Ilist_8AA4 = 35492, addr_kKraid_Ilist_8A41 = 35393,
    addr_kKraid_Ilist_89F3 = 35315, addr_kKraid_Ilist_8887 = 34951,
    addr_off_A78885 = 34949, addr_kKraid_Ilist_87BD = 34749,
    addr_off_A787BB = 34747, addr_kKraid_Ilist_86F3 = 34547,
    addr_kKraid_Ilist_86ED = 34541, addr_kKraid_Ilist_86E7 = 34535,
    addr_kSpritemap_Nothing_A7 = 32845,
}

crate::type_aliases! {
    pub Enemy_Phantoon, Enemy_Etecoon, Enemy_Dachora = crate::game::enemy_layouts::EnemyLayout8;
}

enemy_layout! {
    Enemy_Kraid {
        base: EnemyBase,
        words: [
            kraid_var_A, kraid_var_B, kraid_var_C, kraid_var_D, kraid_var_E, kraid_var_F,
            kraid_parameter_1, kraid_parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            kraid_next, field_2, field_4, kraid_thinking, kraid_min_y_pos_eject,
            kraid_mouth_flags,
        ],
        kraid_healths_8ths: [u16; 8],
        words: [field_1C, kraid_target_x],
        kraid_healths_4ths: [u16; 4],
        words: [field_28, kraid_hurt_frame, kraid_hurt_frame_timer],
        field_2E: [u16; 9],
        words: [],
    }
}

crate::game::constants::typed_constants! { i32;
    kCoroutineNone = 0,
    fnnullsub_170_A7 = 0xa7804c,
    fnUnpauseHook_Kraid_IsDead = 0xa7c1fb, fnUnpauseHook_Kraid_IsAlive = 0xa7c24e,
    fnKraid_UnpauseHook_IsSinking = 0xa7c2a0, fnPauseHook_Kraid = 0xa7c325,
    fnnullsub_237 = 0xa7d4a8, fnPhantoon_Spawn8FireballsInCircleAtStart = 0xa7d4a9,
    fnPhantoon_WaitBetweenSpawningAndSpinningFireballs = 0xa7d4ee, fnPhantoon_SpawnFireballsBeforeFight = 0xa7d508,
    fnPhantoon_WavyFadeIn = 0xa7d54a, fnPhantoon_PickPatternForRound1 = 0xa7d596,
    fnPhantoon_MovePhantoonInFigure8ThenOpenEye = 0xa7d5e7, fnPhantoon_EyeFollowsSamusUntilTimerRunsOut = 0xa7d60d,
    fnPhantoon_BecomesSolidAndBodyVuln = 0xa7d65c, fnPhantoon_IsSwooping = 0xa7d678,
    fnPhantoon_FadeoutWithSwoop = 0xa7d6b9, fnPhantoon_WaitAfterFadeOut = 0xa7d6d4,
    fnPhantoon_MoveLeftOrRightAndPickEyeOpenPatt = 0xa7d6e2, fnPhantoon_FadeInBeforeFigure8 = 0xa7d72d,
    fnPhantoon_BecomeSolidAfterRainingFireballs = 0xa7d73f, fnPhantoon_FadeInDuringFireballRain = 0xa7d767,
    fnPhantoon_FollowSamusWithEyeDuringFireballRain = 0xa7d788, fnPhantoon_FadeOutDuringFireballRain = 0xa7d7d5,
    fnPhantoon_SpawnRainingFireballs = 0xa7d7f7, fnPhantoon_FadeOutBeforeFirstFireballRain = 0xa7d82a,
    fnPhantoon_FadeOutBeforeEnrage = 0xa7d85c, fnPhantoon_MoveEnragedPhantoonToTopCenter = 0xa7d874,
    fnPhantoon_FadeInEnragedPhantoon = 0xa7d891, fnPhantoon_Enraged = 0xa7d8ac,
    fnPhantoon_FadeoutAfterEnrage = 0xa7d916, fnPhantoon_CompleteSwoopAfterFatalShot = 0xa7d92e,
    fnPhantoon_DyingPhantoonFadeInOut = 0xa7d948, fnPhantoon_DyingPhantoonExplosions = 0xa7d98b,
    fnPhantoon_WavyDyingPhantoonAndCry = 0xa7da51, fnPhantoon_DyingFadeOut = 0xa7da86,
    fnPhantoon_AlmostDead = 0xa7dad7, fnPhantoon_Dead = 0xa7db3d,
    fnEtecoon_Func_4 = 0xa7e9af, fnEtecoon_Func_5 = 0xa7ea00,
    fnEtecoon_Func_6 = 0xa7ea37, fnEtecoon_Func_7 = 0xa7eab5,
    fnEtecoon_Func_8 = 0xa7eb02, fnEtecoon_Func_9 = 0xa7eb2c,
    fnEtecoon_Func_10 = 0xa7eb50, fnEtecoon_Func_11 = 0xa7ebcd,
    fnEtecoon_Func_12 = 0xa7ec1b, fnEtecoon_Func_16 = 0xa7ec97,
    fnEtecoon_Func_17 = 0xa7ecbb, fnEtecoon_Func_18 = 0xa7ecdf,
    fnEtecoon_Func_19 = 0xa7ed09, fnEtecoon_Func_20 = 0xa7ed2a,
    fnEtecoon_Func_21 = 0xa7ed54, fnEtecoon_Func_22 = 0xa7ed75,
    fnEtecoon_Func_23 = 0xa7edc7, fnEtecoon_Func_24 = 0xa7ee3e,
    fnEtecoon_Func_25 = 0xa7ee9a, fnEtecoon_Func_26 = 0xa7eeb8,
    fnDachora_Func_2 = 0xa7f570, fnDachora_Func_3 = 0xa7f5bc,
    fnDachora_Func_4 = 0xa7f5ed, fnDachora_Func_5 = 0xa7f65e,
    fnDachora_Func_7 = 0xa7f78f, fnDachora_Func_8 = 0xa7f806,
    fnDachora_Func_11 = 0xa7f935, fnDachora_Func_12 = 0xa7f98c,
}
const RomPtr_A7: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0xa7>;
crate::ram_map::ram_pointer_functions! {
    gVramWriteEntry(a: i32) -> VramWriteEntry = [(0xd0 + a) as isize];
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
    Get_Dachora(o: i32) -> Enemy_Dachora = [0xf78, o as isize];
    Get_Etecoon(o: i32) -> Enemy_Etecoon = [0xf78, o as isize];
    Get_Kraid(o: i32) -> Enemy_Kraid = [0xf78, o as isize];
    Get_Phantoon(o: i32) -> Enemy_Phantoon = [0xf78, o as isize];
}
static g_word_A7A916: u16 = 0x120;
static g_word_A7A918: u16 = 0xa0;
static g_word_A7A91A: u16 = 0x40;
static g_word_A7A91C: u16 = 3;
static g_word_A7A920: u16 = 3;
static g_word_A7A922: u16 = 4;
static g_word_A7A926: u16 = 0x8000;
static g_word_A7A928: u16 = 3;
static g_word_A7CD73: u16 = 0x600;
static g_word_A7CD75: u16 = 0;
static g_word_A7CD77: u16 = 0x1000;
static g_word_A7CD79: u16 = 0;
static g_word_A7CD7B: u16 = 2;
static g_word_A7CD7D: u16 = 7;
static g_word_A7CD7F: u16 = 0;
static g_word_A7CD81: u16 = 0x600;
static g_word_A7CD83: u16 = 0;
static g_word_A7CD85: u16 = 0x1000;
static g_word_A7CD87: u16 = 0;
static g_word_A7CD89: u16 = 0xfffe;
static g_word_A7CD8B: u16 = 0xfff9;
static g_word_A7CD8D: u16 = 0;
static g_word_A7CD9B: u16 = 0x40;
static g_word_A7CD9D: u16 = 0xc00;
static g_word_A7CD9F: u16 = 0x100;
static g_word_A7CDA1: u16 = 0xf000;
static g_word_A7CDA3: u16 = 8;
static g_byte_A7CDA5: [u8; 8] = [6, 6, 8, 8, 6, 8, 6, 8];
static g_word_A7E900: u16 = 0xfffd;
static g_word_A7E902: u16 = 0;
static g_word_A7E906: u16 = 0;
static g_word_A7E904: u16 = 0xfffc;
static g_word_A7E908: u16 = 2;
static g_word_A7E90A: u16 = 0;
static g_word_A7E90C: u16 = 0xfffe;
static g_word_A7E90E: u16 = 0;
static g_word_A7E910: u16 = 0x40;
static g_word_A7F4C9: u16 = 0x60;
static g_word_A7F4CD: u16 = 0x78;
static g_word_A7F4CF: u16 = 0x3c;
static g_word_A7F4D1: u16 = 1;
static g_word_A7F4D3: u16 = 8;
static g_word_A7F4D5: u16 = 8;
static g_word_A7F4D7: u16 = 0;
static g_word_A7F4D9: u16 = 0;
static g_word_A7F4DB: u16 = 0x1000;
crate::game::callback_dispatch! {
    pub unsafe fn CallEnemyInstrExtFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10997485] Phantoon_Func_1 [] {
        QueueSfx2_Max6(
            crate::rtl::rom_table_word!(game, 0xa7cded, word!(game.g_word_7E9032) as isize),
            game,
        );
        let mut v0: u16 = (word!(game.g_word_7E9032)).wrapping_add(1);
        if word!(game.g_word_7E9032) as i32 - 2i32 & 0x8000 == 0 {
            v0 = 0;
        }
        word!(game.g_word_7E9032 = v0);
    };
    [10997823] Phantoon_StartTrackingSamusAndInitEyeTimer [] {
        (*Get_Phantoon(0x80i32, game)).var_A = 0;
        let E = Get_Phantoon(0, game);
        enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kKraid_Ilist_CC4D as u16}
        (*E).base.properties =
            ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
        (*E).var_E =
            crate::rtl::rom_table_word!(game, 0xa7cd41, (game.next_random() as i32 & 7) as isize);
        (*E).var_F = (0xa7d60di32) as u16;
        let E1 = Get_Phantoon(0x40, game);
        enemy_instruction! {[(*E1).base]; timer = 1, instruction = addr_kKraid_Ilist_CC9D as u16}
    };
    [10997878] Phantoon_PickPatternForRound2 [] {
        let E = Get_Phantoon(0, game);
        (*E).var_E = 60;
        let v1: u16 =
            crate::rtl::rom_table_word!(game, 0xa7cd53, (game.next_random() as i32 & 7) as isize);
        let E1 = Get_Phantoon(0x40, game);
        (*E1).var_A = v1;
        if (word!(game.nmi_frame_counter_word) & 1) != 0 {
            if (*E1).var_C == 0 {
                let mut v4: i16 = (((*E).var_A) as i16).wrapping_sub(1);
                (*E).var_A = v4 as u16;
                if v4 < 0 {
                    (*E).var_A = 533;
                }
            }
            set_fields! {[(*E)], 0; var_C, var_B, var_D}
            (*E1).var_C = 1;
        } else {
            if (*E1).var_C != 0 {
                let v3: u16 = ((*E).var_A).wrapping_add(1);
                (*E).var_A = v3;
                if v3 as i32 - 534i32 & 0x8000 == 0 {
                    (*E).var_A = 0;
                }
            }
            (*E).var_C = 1;
            set_fields! {[(*E)], 0; var_B, var_D}
            (*E1).var_C = 0;
        }
        if (*E).parameter_2 != 0 {
            (*E1).var_F = 0;
            (*E).var_F = (0xa7d82ai32) as u16;
        } else {
            (*E).var_F = (0xa7d5e7i32) as u16;
        };
    };
    @external {
        10997598 => [Phantoon_Func_4(k, game)];
    }
}
pub unsafe fn Kraid_Touch_ArmFoot(game: &mut Game) {
    {
        actors::combat::contact_reaction(
            &mut Runtime::new(game),
            actors::api::DeathHandling::Finalize,
        );
    }
}
pub fn Kraid_Func_1(game: &mut Game) {
    {
        word! {game; extra_samus_x_displacement = 4; extra_samus_y_displacement = -8i32 as u16; }
    }
}
pub unsafe fn Kraid_Arm_Shot(j: u16, game: &mut Game) {
    unsafe {
        Kraid_SpawnExplosionEproj(j, game);
        word!(game.projectile_dir[(j >> 1)] |= (0x10u16));
    }
}
pub unsafe fn Kraid_CheckIfDead(game: &mut Game) -> u16 {
    {
        ((gameplay_flag_runtime::boss_word_at(word!(game.area_index), BossWordOrder::Native, game)
            & 1)
            != 0) as u16
    }
}
pub unsafe fn Kraid_SetEnemyPropsToDead(game: &mut Game) {
    unsafe {
        let E = Get_Kraid(word!(game.cur_enemy_index) as i32, game);
        (*E).base.properties = (*E).base.properties & 0x50ff | 0x700;
    }
}
pub unsafe fn Kraid_SetupGfxWithTilePrioClear(r18: u16, game: &mut Game) {
    unsafe {
        decompress_to_ram(0xb9fa38, 0x4000, game);
        decompress_to_ram(0xb9fe3e, 0x2000, game);
        let E = Get_Kraid(0, game);
        set_fields! {[(*E)], 0; kraid_hurt_frame, kraid_hurt_frame_timer}
        for v1 in (0u16..1536).step_by(2) {
            *(ram_ptr!(game, u16; 0x2000)).offset(((v1 as i32 >> 1) + 1024) as isize) =
                (word!(game.tilemap_stuff[(v1 >> 1)])) & (0xdfffu16);
        }
        for v2 in 0u16..1024 {
            word!(
                game.tilemap_stuff[v2] =
                    (r18) & ((*(ram_ptr!(game, Ram4000; 0x4000))).xray_tilemaps[v2 as usize])
            );
        }
    }
}
pub unsafe fn Kraid_DrawRoomBg(game: &mut Game) {
    unsafe {
        word_transfers!(game, ascending, 16; le(target_palettes, 96) <- rom(0xa786c7, 0));
        word!(game.palette_change_num = 0);
        let v2: u16 = word!(game.vram_write_queue_tail);
        let mut v3 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v3).size = 512;
        (*v3).src.addr = addr_kKraidsRoomBg as i32 as VoidP;
        (*v3).src.bank = -89i32 as u8;
        (*v3).vram_dst = (((*ram_ptr!(game; 0x5d) as i32 & 0xf) << 8) + 16128) as u16;
        word!(game.vram_write_queue_tail = (v2).wrapping_add(7));
    }
}
pub unsafe fn Kraid_GetsBig_Thinking_Setup(game: &mut Game) {
    unsafe {
        (*Get_Kraid(0i32, game)).kraid_var_A = (0xa7aec4i32) as u16;
        Kraid_SetLintYAndRandomThinkTimer(game);
    }
}
pub unsafe fn Kraid_Mainloop_Thinking_Setup(game: &mut Game) {
    unsafe {
        (*Get_Kraid(0i32, game)).kraid_var_A = (0xa7aea4i32) as u16;
        Kraid_SetLintYAndRandomThinkTimer(game);
    }
}
pub unsafe fn Kraid_Lints_Enable(k: u16, a: u16, game: &mut Game) {
    unsafe {
        let E = Get_Kraid(k as i32, game);
        (*E).kraid_var_F = a;
        (*E).kraid_var_A = (0xa7b923i32) as u16;
        (*E).kraid_next = (0xa7b832i32) as u16;
        (*E).kraid_var_B = 0;
    }
}
macro_rules! advance_kraid_thinking_timer {
    ($enemy:ident, $next_state:literal, $game:ident) => {
        let kraid_thinking: u16 = (*$enemy).kraid_thinking;
        if kraid_thinking != 0 {
            let v2: u16 = (kraid_thinking).wrapping_sub(1);
            (*$enemy).kraid_thinking = v2;
            if v2 == 0 {
                (*$enemy).kraid_var_A = (($next_state) as u16);
                (*$enemy).kraid_var_B = addr_stru_A796DA as u16;
                (*$enemy).kraid_var_C = (*($crate::rtl::rom_ptr!($game, KraidInstrList; 0xa796d2)))
                    .timer;
            }
        }
    };
}

macro_rules! advance_kraid_mouth_hit_response {
    ($enemy:ident) => {
        (*$enemy).kraid_var_C = 90;
        let kraid_mouth_flags: u16 = (*$enemy).kraid_mouth_flags;
        if (kraid_mouth_flags & 4) != 0 && {
            let mut v2: u16 = (kraid_mouth_flags).wrapping_sub(256);
            (*$enemy).kraid_mouth_flags = v2;
            (v2 & 0xff00) != 0
        } {
            (*$enemy).kraid_var_A = ((0xa7b92di32) as u16);
            (*$enemy).kraid_var_F = 64;
            (*$enemy).kraid_next = ((0xa7b6bfi32) as u16);
            (*$enemy).field_2 = 2;
        } else {
            (*$enemy).kraid_mouth_flags = 0;
        }
    };
}

pub unsafe fn Kraid_ProcessKraidInstr(game: &mut Game) -> u16 {
    unsafe {
        let E = Get_Kraid(0, game);
        let result: u16 = (*E).kraid_var_C;
        if result != 0 {
            crate::update_place! { previous fresh1 = (*E).kraid_var_C => (*E).kraid_var_C.wrapping_sub(1); }
            if fresh1 == 1 {
                return Kraid_ExecuteInstr(game);
            }
        }
        result
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallKraidInstr(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @forward [pub unsafe fn (game: &mut Game)];
    [10989460] KraidInstr_PlayRoarSfx => QueueSfx2_Max6(0x2d, game);
    [10989471] KraidInstr_PlayDyingSfx => QueueSfx2_Max15(0x2e, game);
}
pub unsafe fn Kraid_ExecuteInstr(game: &mut Game) -> u16 {
    unsafe {
        let E = Get_Kraid(0, game);
        loop {
            let kraid_var_B: u16 = (*E).kraid_var_B;
            let v2 = RomPtr_A7(kraid_var_B, game) as *const u16;
            let result: u16 = *v2;
            if *v2 != 0xffff {
                if ((*v2) as i16).wrapping_add(1) < 0 {
                    CallKraidInstr(((result) as u32) | (0xa70000u32), game);
                    (*E).kraid_var_B = ((*E).kraid_var_B).wrapping_add(2);
                } else {
                    (*E).kraid_var_C = result;
                    (*E).kraid_var_B = (kraid_var_B).wrapping_add(8);
                    let v4: u16 = *v2.offset(1);
                    let mut v5: u16 = word!(game.vram_write_queue_tail);
                    (*gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game)).size = 704;
                    v5 = (v5).wrapping_add(2);
                    (*gVramWriteEntry(v5 as i32, game)).size = v4;
                    v5 = (v5).wrapping_add(2);
                    crate::update_place! { previous fresh0 = v5 => v5.wrapping_add(1); }
                    *(&raw mut (*(gVramWriteEntry
                        as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
                        fresh0 as i32, game
                    ))
                    .size as *mut u8) = -89i32 as u8;
                    (*gVramWriteEntry(v5 as i32, game)).size =
                        ((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8) as u16;
                    word!(game.vram_write_queue_tail = (v5).wrapping_add(2));
                    return 1;
                }
            } else {
                return result;
            }
        }
    }
}
pub unsafe fn Kraid_Shot_Mouth(game: &mut Game) {
    unsafe {
        let mut v3: i16 = 0;
        let E = Get_Kraid(0, game);
        if (*E).kraid_var_A as i32 + 0x3ac9 & 0x8000 == 0 {
            return;
        }
        let v2 = RomPtr_A7(((*E).kraid_var_B).wrapping_sub(8), game);
        if *(v2.offset(6) as *mut u16) == 0xffff {
            v3 = 0;
        } else {
            let mut v4: u16 = *(v2.offset(6) as *mut u16);
            (*E).kraid_var_E = 1;
            v3 = 0;
            let mut v5: *const u8 = RomPtr_A7(v4, game);
            let mut r22: u16 = ((*E).base.x_pos).wrapping_add(*(v5 as *mut u16));
            let mut r20: u16 = ((*E).base.y_pos).wrapping_add(*(v5.offset(2) as *mut u16));
            let mut r18: u16 = ((*E).base.y_pos).wrapping_add(*(v5.offset(6) as *mut u16));
            if word!(game.projectile_counter) != 0 {
                let mut v6: u16 = (2u16).wrapping_mul(word!(game.projectile_counter));
                do_until! { (v6 & 0x8000) != 0;
                    let mut v7: i32 = v6 as i32 >> 1;
                    if !(word!(game.projectile_y_pos[v7]) as i32
                        - word!(game.projectile_y_radius[v7]) as i32
                        - 1i32
                        - r18 as i32
                        & 0x8000
                        == 0
                        || word!(game.projectile_y_radius[v7]) as i32
                            + word!(game.projectile_y_pos[v7]) as i32
                            - r20 as i32
                            & 0x8000
                            != 0
                        || word!(game.projectile_x_radius[v7]) as i32
                            + word!(game.projectile_x_pos[v7]) as i32
                            - r22 as i32
                            & 0x8000
                            != 0)
                    {
                        let mut v8: u16 = word!(game.projectile_type[v7]);
                        let mut skip_mouth_hit: bool = !((v8 & 0xf00) != 0)
                            && (!((v8 & 0x10) != 0)
                                || ({
                                    (*E).kraid_mouth_flags = ((*E).kraid_mouth_flags) | (1u16);
                                    false
                                }));
                        if skip_mouth_hit {
                        } else {
                            word!(game.collision_detection_index = v6 >> 1);
                            actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::ReactionOnly);
                            word!(game.projectile_dir[(v6 >> 1)] |= (0x10u16));
                            v3 = 1;
                            v6 = 0;
                        }
                    }
                    v6 = (v6).wrapping_sub(2);
                }
            }
        }
        if v3 != 0 {
            (*E).kraid_hurt_frame = 6;
            (*E).kraid_hurt_frame_timer = 2;
            let kraid_mouth_flags: u16 = (*E).kraid_mouth_flags;
            if (kraid_mouth_flags & 2) != 0 {
                (*E).kraid_mouth_flags = (kraid_mouth_flags) | (4u16);
            }
            if (*E).base.health as i32 - 1i32 & 0x8000 != 0
                && (*E).kraid_var_A as i32 + 0x3ca0 & 0x8000 != 0
            {
                (*E).kraid_var_A = (0xa7c360i32) as u16;
                (*E).kraid_mouth_flags = 0;
                (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
                Kraid_SetupGfxWithTilePrioClear(!(0x2000i32) as u16, game);
                for v12 in (0u16..384).step_by(64) {
                    let EL = Get_Kraid(v12 as i32, game);
                    (*EL).base.properties =
                        ((*EL).base.properties) | ((kEnemyProps_Tangible) as u16);
                }
                if (*E).kraid_var_B as i32 + 0x68f2 & 0x8000 != 0 {
                    (*E).kraid_var_B = ((*E).kraid_var_B).wrapping_add(60);
                }
            }
        }
    }
}
pub unsafe fn Kraid_SpawnExplosionEproj(k: u16, game: &mut Game) {
    {
        let v1: i32 = k as i32 >> 1;
        game.eproj_spawn_pt = Point16U {
            x: word!(game.projectile_x_pos[v1]),
            y: word!(game.projectile_y_pos[v1]),
        };
        let v2: u16 = (if (word!(game.projectile_type[v1]) & 0x200) == 0 {
            6
        } else {
            29
        }) as u16;
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, v2, game);
        QueueSfx1_Max6(0x3d, game);
    }
}
pub unsafe fn Kraid_HurtFlash_Handling(game: &mut Game) {
    unsafe {
        let mut v0: u16 = 0;
        if ((*Get_Kraid(0i32, game)).kraid_hurt_frame & 1) == 0 {
            v0 = 32;
        }
        let mut v1: u16 = 0;
        do_until! { ((v1) as i16).wrapping_sub(32) >= 0 ;
            *(ram_ptr!(game, u16; 0xc000)).offset(((v1 as i32 >> 1) + 240) as isize) =
                crate::rtl::rom_table_word!(game, 0xa7b513, (v0 >> 1) as isize);
            v1 = (v1).wrapping_add(2);
            v0 = (v0).wrapping_add(2);
        }
    }
}
pub unsafe fn Kraid_HealthBasedPaletteHandling(game: &mut Game) {
    unsafe {
        let mut v0: u16 = 0;
        let E = Get_Kraid(0, game);
        if ((*E).kraid_hurt_frame & 1) == 0 {
            let mut v2: u16 = 14;
            let health: u16 = (*E).base.health;
            while ((health) as i16).wrapping_sub(
                ((*Get_Kraid(v2 as i32, game)).kraid_healths_8ths[0i32 as usize]) as i16,
            ) < 0
            {
                v2 = (v2).wrapping_sub(2);
                if v2 == 0 {
                    break;
                }
            }
            v0 = (16 * (v2 as i32 + 2)) as u16;
        }
        for v4 in (0u16..32).step_by(2) {
            let v5: i32 = v0 as i32 >> 1;
            let v6: i32 = v4 as i32 >> 1;
            word! { game;
                palette_buffer[(v6 + 112)] = crate::rtl::rom_table_word!(game, 0xa7b3d3, v5 as isize);
                palette_buffer[(v6 + 240)] = crate::rtl::rom_table_word!(game, 0xa7b513, v5 as isize);
            }
            v0 = (v0).wrapping_add(2);
        }
    }
}
pub fn Kraid_Instr_1(k: u16, jp: EnemyScript) -> EnemyScript {
    jp
}
pub unsafe fn KraidLintCommon_Main(k: u16, game: &mut Game) {
    unsafe {
        Kraid_EnemyTouch_Lint(k, game);
        let v1 = gEnemyData(k as i32, game);
        CallKraidFunc((((*v1).ai_var_A) as u32) | (0xa70000u32), game);
    }
}
pub unsafe fn KraidLint_ProduceLint(k: u16, game: &mut Game) {
    unsafe {
        let v1 = gEnemyData(k as i32, game);
        (*v1).properties = ((*v1).properties) & (0xfaffu16);
        (*v1).x_pos = ((*v1).ai_var_C as i32 + (*gEnemyData(0i32, game)).x_pos as i32
            - (*v1).ai_var_B as i32) as u16;
        let v2: u16 = ((*v1).ai_var_B).wrapping_add(1);
        (*v1).ai_var_B = v2;
        if v2 as i32 - 32i32 & 0x8000 == 0 {
            (*v1).ai_var_A = (0xa7b868i32) as u16;
            (*v1).ai_preinstr = 30;
        }
    }
}
pub unsafe fn KraidLint_ChargeLint(k: u16, game: &mut Game) {
    unsafe {
        let E = gEnemyData(k as i32, game);
        let mut v1: i16 = 0;
        if ((*E).ai_preinstr & 1) != 0 {
            v1 = 3584;
        }
        (*E).palette_index = v1 as u16;
        (*E).x_pos = ((*E).ai_var_C as i32 + (*gEnemyData(0i32, game)).x_pos as i32
            - (*E).ai_var_B as i32) as u16;
        crate::update_place! { previous fresh5 = (*E).ai_preinstr => (*E).ai_preinstr.wrapping_sub(1); }
        if fresh5 == 1 {
            (*E).ai_var_A = (0xa7b89bi32) as u16;
            QueueSfx3_Max6(0x1f, game);
        }
    }
}
pub unsafe fn KraidLint_FireLint(k: u16, game: &mut Game) {
    unsafe {
        let E = Get_Kraid(k as i32, game);
        words!(
            (*E).base,
            x_pos,
            x_subpos += -(words!(i32, (g_word_A7A928, g_word_A7A926))) as u32
        );
        if (*E).base.x_pos as i32 - 56i32 & 0x8000 != 0 {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
        }
        if (*E).base.x_pos as i32 - 32i32 & 0x8000 != 0 {
            (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
            (*E).kraid_var_A = (0xa7b923i32) as u16;
            (*E).kraid_var_F = 300;
            (*E).kraid_next = (0xa7b832i32) as u16;
            (*E).kraid_var_B = 0;
        }
        if {
            let actor = k;
            0u16.wrapping_sub(u16::from(actor_api::touches_player_from_above(
                &Runtime::new(game),
                Key::from_legacy(actor),
            )))
        } != 0
        {
            words!(
                game.extra_samus_x_displacement,
                extra_samus_x_subdisplacement +=
                    -(words!(i32, (g_word_A7A928, g_word_A7A926))) as u32
            );
            if word!(game.extra_samus_x_displacement) as i32 + 16 & 0x8000 != 0 {
                word!(game.extra_samus_x_displacement = -16i32 as u16);
            }
        }
    }
}
pub unsafe fn KraidFingernail_WaitForLintXpos(k: u16, game: &mut Game) {
    unsafe {
        let E2 = Get_Kraid(0x80, game);
        if (*E2).base.x_pos as i32 - 256i32 & 0x8000 == 0 {
            let E = Get_Kraid(k as i32, game);
            (*E).kraid_var_A = (*E).kraid_next;
            (*E).base.properties = ((*E).base.properties) & (0xfaffu16);
        }
    }
}
pub unsafe fn KraidEnemy_HandleFunctionTimer(k: u16, game: &mut Game) {
    unsafe {
        let E = Get_Kraid(k as i32, game);
        if (*E).kraid_var_F != 0
            && {
                crate::update_place! { previous fresh6 = (*E).kraid_var_F => (*E).kraid_var_F.wrapping_sub(1); }
                fresh6 == 1
            }
        {
            (*E).kraid_var_A = (*E).kraid_next;
        }
    }
}
pub unsafe fn Kraid_AlignEnemyToKraid(k: u16, game: &mut Game) {
    unsafe {
        let x_pos: u16 = (*Get_Kraid(0i32, game)).base.x_pos;
        let E = Get_Kraid(k as i32, game);
        (*E).base.x_pos = (x_pos).wrapping_sub((*E).base.x_width);
        KraidEnemy_HandleFunctionTimer(k, game);
    }
}
pub unsafe fn KraidFoot_FirstPhase_Thinking(k: u16, game: &mut Game) {
    unsafe {
        Kraid_HandleFirstPhase(game);
        KraidEnemy_HandleFunctionTimer(k, game);
    }
}
pub unsafe fn KraidEnemy_ProcessInstrEnemyTimer(k: u16, game: &mut Game) {
    unsafe {
        Kraid_ProcessKraidInstr(game);
        KraidEnemy_HandleFunctionTimer(k, game);
    }
}
pub unsafe fn Kraid_EnemyTouch_Lint(k: u16, game: &mut Game) {
    unsafe {
        let E = Get_Kraid(k as i32, game);
        if ((*E).base.properties as i32 & kEnemyProps_Tangible as i32) == 0
            && word!(game.samus_invincibility_timer) == 0
        {
            let r18: u16 = ((*(rom_ptr!(game, Hitbox; 0xa792b7))).left as i32
                + (*E).base.x_pos as i32
                - 2i32) as u16;
            if word!(game.samus_x_radius) as i32 + word!(game.samus_x_pos) as i32 - r18 as i32
                & 0x8000
                == 0
                && word!(game.samus_x_pos) as i32 - word!(game.samus_x_radius) as i32 - r18 as i32
                    & 0x8000
                    != 0
            {
                let r22: u16 = ((*(rom_ptr!(game, Hitbox; 0xa792b7))).top as i32
                    + (*E).base.y_pos as i32
                    + 2) as u16;
                if word!(game.samus_y_radius) as i32 + word!(game.samus_y_pos) as i32 - r22 as i32
                    & 0x8000
                    == 0
                {
                    let r24: u16 = ((*(rom_ptr!(game, Hitbox; 0xa792b7))).bottom as i32
                        + (*E).base.y_pos as i32
                        - 2i32) as u16;
                    if word!(game.samus_y_pos) as i32
                        - word!(game.samus_y_radius) as i32
                        - r24 as i32
                        & 0x8000
                        != 0
                    {
                        let mut v2: u16 = (word!(game.extra_samus_x_displacement) as i32
                            + !(word!(game.samus_x_radius) as i32 + 16))
                            as u16;
                        if v2 as i32 - 16i32 & 0x8000 == 0 {
                            v2 = 16;
                        }
                        word!(game.extra_samus_x_displacement = v2);
                        actors::combat::contact_reaction(
                            &mut Runtime::new(game),
                            actors::api::DeathHandling::Finalize,
                        );
                        (*E).base.properties =
                            ((*E).base.properties) | ((kEnemyProps_Tangible) as u16);
                    }
                }
            }
        }
    }
}
pub unsafe fn Kraid_SetWalkingBackwards(j: u16, a: u16, game: &mut Game) {
    unsafe {
        (*Get_Kraid(0i32, game)).kraid_target_x = a;
        let E = Get_Kraid(0x140, game);
        (*E).kraid_next = j;
        (*E).kraid_var_A = (0xa7bb45i32) as u16;
        enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kKraid_Ilist_8887 as u16}
    }
}
pub unsafe fn Kraid_SetWalkingForwards(j: u16, a: u16, game: &mut Game) {
    unsafe {
        (*Get_Kraid(0i32, game)).kraid_target_x = a;
        let E = Get_Kraid(0x140, game);
        (*E).kraid_next = j;
        (*E).kraid_var_A = (0xa7bbaei32) as u16;
        enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kKraid_Ilist_86F3 as u16}
    }
}
pub unsafe fn KraidFingernailInit(k: u16, game: &mut Game) {
    unsafe {
        let palette_index: u16 = (*Get_Kraid(0i32, game)).base.palette_index;
        let E = Get_Kraid(k as i32, game);
        (*E).base.palette_index = palette_index;
        (*E).kraid_var_B = 40;
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
        enemy_instruction! {[(*E).base]; timer = 0x7fff, instruction = addr_kKraid_Ilist_8B0A as u16}
        (*E).base.spritemap_pointer = (*(rom_ptr!(game, SpriteDrawInstr; 0xa78b0a))).field_2;
        (*E).kraid_next = (0xa7bd60i32) as u16;
        (*E).kraid_var_A = (0xa7b92di32) as u16;
        (*E).kraid_var_F = 64;
    }
}
pub unsafe fn KraidsFingernail_Fire(k: u16, game: &mut Game) {
    unsafe {
        let v1 = gEnemyData(k as i32, game);
        if {
            let (actor, amount) = (k, (words!((*v1), ai_var_C, ai_var_B)) as i32);
            (actor_api::move_horizontal(
                &mut Runtime::new(game),
                Key::from_legacy(actor),
                amount,
                actor_api::SlopePolicy::Ignore,
            ) == actor_api::MovementResult::Blocked) as u8
        } != 0
        {
            (*v1).ai_var_B = -((*v1).ai_var_B as i32) as u16;
            (*v1).ai_var_C = -((*v1).ai_var_C as i32) as u16;
        } else {
            let mut v3 = null_mut::<EnemyData>();
            let mut v4 = null_mut::<EnemyData>();
            let mut i: i32 = 0;
            loop {
                v3 = gEnemyData(0, game);
                v4 = gEnemyData(k as i32, game);
                if ((crate::rtl::rom_table_word!(game, 0xa7bf1d, ((i >> 1) + 1) as isize) as i32
                    + (*v3).y_pos as i32
                    - (*v4).y_pos as i32) as i16)
                    < 0
                {
                    break;
                }
                i += 4;
            }
            let r18: u16 = (crate::rtl::rom_table_word!(game, 0xa7bf1d, (i >> 1) as isize))
                .wrapping_add((*v3).x_pos);
            if (*v4).x_width as i32 + (*v4).x_pos as i32 - r18 as i32 & 0x8000 == 0
                && ((*v4).ai_var_C & 0x8000) == 0
            {
                (*v4).ai_var_B = -((*v4).ai_var_B as i32) as u16;
                (*v4).ai_var_C = -((*v4).ai_var_C as i32) as u16;
            }
        }
        let v5 = gEnemyData(k as i32, game);
        if {
            let (actor, amount) = (k, (words!((*v5), ai_var_E, ai_var_D)) as i32);
            (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount)
                == actor_api::MovementResult::Blocked) as u8
        } != 0
        {
            words!(
                (*v5),
                ai_var_E,
                ai_var_D = -(words!(i32, (*v5), ai_var_E, ai_var_D)) as u32
            );
        }
    }
}
crate::forward_calls! { pub unsafe fn (game: &mut Game);
    Kraid_SpawnPlmToClearCeiling => spawn_hardcoded(
        SpawnHardcodedPlmArgs::new(0x2, 0x12, 0xb7b7 as VoidP),
        game,
    );
    Kraid_ClearSomeSpikes => spawn_hardcoded(
        SpawnHardcodedPlmArgs::new(0x5, 0x1b, 0xb7bb as VoidP),
        game,
    );
}
pub unsafe fn UnpauseHook_Kraid_IsDead(game: &mut Game) -> CoroutineRet {
    unsafe {
        crate::game::display_refresh::set_blank(true, game);
        GraphicsAsset::VramWords(
            0xa7a716,
            ((16 * (*ram_ptr!(game; 0x5d) as i32 & 0xf) + 63) as u8 as u16) << 8,
            0x200,
        )
        .load(game);
        GraphicsAsset::VramWords(0x9ab200, 0x4000, 0x800).load(game);
        Kraid_TransferTopHalfToVram(game);
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn UnpauseHook_Kraid_IsAlive(game: &mut Game) -> CoroutineRet {
    unsafe {
        crate::game::display_refresh::set_blank(true, game);
        GraphicsAsset::VramWords(
            0x7e5000,
            ((*ram_ptr!(game; 0x5d)).wrapping_add(62) as u16) << 8,
            0x400,
        )
        .load(game);
        Kraid_TransferTopHalfToVram(game);
        kCoroutineNone as CoroutineRet
    }
}
pub unsafe fn Kraid_TransferTopHalfToVram(game: &mut Game) {
    unsafe {
        GraphicsAsset::VramWords(
            0x7e2000,
            ((*ram_ptr!(game; 0x59) & 0xfc) as u16) << 8,
            0x800,
        )
        .load(game);
        crate::game::display_refresh::set_blank(false, game);
    }
}
pub unsafe fn Kraid_UnpauseHook_IsSinking(game: &mut Game) -> CoroutineRet {
    unsafe {
        let mut _state_variable_: *mut u8 = ram_ptr!(game; 0x613);
        's_100: {
            let mut queue_transfer = match *_state_variable_ as i32 {
                0 => {
                    crate::game::display_refresh::set_blank(true, game);
                    GraphicsAsset::VramWords(
                        0x7e5000,
                        ((*ram_ptr!(game; 0x5d)).wrapping_add(62) as u16) << 8,
                        0x400,
                    )
                    .load(game);
                    if (((*Get_Kraid(0i32, game)).base.y_pos) as i16).wrapping_sub(
                        ((*(rom_ptr!(game, KraidSinkTable; 0xa7c5e7))).field_0) as i16,
                    ) >= 0
                    {
                        word! {game; my_counter = 0; my_counter2 = word!(game.vram_write_queue_tail); }
                        true
                    } else {
                        break 's_100;
                    }
                }
                1 => false,
                _ => break 's_100,
            };
            loop {
                if queue_transfer {
                    if !((*(rom_ptr!(game, KraidSinkTable; 0xa7c5e7))
                        .offset(word!(game.my_counter) as isize))
                    .field_0
                        != 0xffff
                        && (((*Get_Kraid(0i32, game)).base.y_pos) as i16).wrapping_sub(
                            ((*(rom_ptr!(game, KraidSinkTable; 0xa7c5e7))
                                .offset(word!(game.my_counter) as isize))
                            .field_0) as i16,
                        ) >= 0)
                    {
                        break 's_100;
                    }
                    let mut v2: *mut VramWriteEntry =
                        gVramWriteEntry(word!(game.my_counter2) as i32, game);
                    (*v2).size = 64;
                    (*v2).src.addr = 0x2fc0 as VoidP;
                    (*v2).src.bank = 0x7e;
                    (*v2).vram_dst = ((*(rom_ptr!(game, KraidSinkTable; 0xa7c5e7))
                        .offset(word!(game.my_counter) as isize))
                    .field_2 as i32
                        + ((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8))
                        as u16;
                    word!(game.vram_write_queue_tail = (word!(game.my_counter2)).wrapping_add(7));
                }
                let mut _coret_: u8 = crate::game::display_refresh::wait(true, game);
                if _coret_ != 0 {
                    *_state_variable_ = 1;
                    return _coret_;
                }
                word!(game.my_counter += 1);
                queue_transfer = true;
            }
        }
        *_state_variable_ = 0;
        0 as CoroutineRet
    }
}
#[forbid(unsafe_code)]
pub fn PauseHook_Kraid(game: &mut Game) {
    crate::game::vram_read_queue::enqueue(0x5d, 15872, 0x5000, 1024, false, game);
}
pub unsafe fn Kraid_PlaySoundEveryHalfSecond(game: &mut Game) {
    {
        if word!(game.kraid_unk9000 -= 1, reread) == 0 {
            QueueSfx3_Max6(0x1e, game);
            word!(game.kraid_unk9000 = 30);
        }
    }
}
pub unsafe fn Kraid_HandleSinking(game: &mut Game) {
    unsafe {
        let mut i: i32 = 0;
        loop {
            let mut v1: i16 =
                (*(rom_ptr!(game, KraidSinkTable; 0xa7c5e7)).offset(i as isize)).field_0 as i16;
            if v1 < 0 {
                break;
            }
            if v1 as i32 == (*Get_Kraid(0i32, game)).base.y_pos as i32 {
                if ((*(rom_ptr!(game, KraidSinkTable; 0xa7c5e7)).offset(i as isize)).field_2
                    & 0x8000)
                    == 0
                {
                    let v2: u16 = word!(game.vram_write_queue_tail);
                    let mut v3 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
                    (*v3).size = 64;
                    (*v3).src.addr = 12224 as VoidP;
                    (*v3).src.bank = 126;
                    (*v3).vram_dst = ((*(rom_ptr!(game, KraidSinkTable; 0xa7c5e7))
                        .offset(i as isize))
                    .field_2 as i32
                        + ((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8))
                        as u16;
                    word!(game.vram_write_queue_tail = (v2).wrapping_add(7));
                }
                CallKraidSinkTableFunc(
                    (((*(rom_ptr!(game, KraidSinkTable; 0xa7c5e7)).offset(i as isize)).field_4)
                        as u32)
                        | (0xa70000u32),
                    game,
                );
                return;
            }
            i += 1;
        }
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallKraidSinkTableFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10995345] Kraid_CrumbleLeftPlatform_Left [] {
        SpawnEprojWithGfx(
            0x70,
            word!(game.cur_enemy_index),
            addr_kEproj_RocksFallingKraidCeiling as u16,
            game,
        );
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x7, 0x12, 0xb7a7 as VoidP), game);
    };
    [10995367] Kraid_CrumbleRightPlatform_Middle [] {
        SpawnEprojWithGfx(
            0xf0,
            word!(game.cur_enemy_index),
            addr_kEproj_RocksFallingKraidCeiling as u16,
            game,
        );
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xf, 0x12, 0xb7a7 as VoidP), game);
    };
    [10995389] Kraid_CrumbleRightPlatform_Left [] {
        SpawnEprojWithGfx(
            0xe0,
            word!(game.cur_enemy_index),
            addr_kEproj_RocksFallingKraidCeiling as u16,
            game,
        );
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xe, 0x12, 0xb7af as VoidP), game);
    };
    [10995411] Kraid_CrumbleLeftPlatform_Right [] {
        SpawnEprojWithGfx(
            0x90,
            word!(game.cur_enemy_index),
            addr_kEproj_RocksFallingKraidCeiling as u16,
            game,
        );
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x9, 0x12, 0xb7a7 as VoidP), game);
    };
    [10995433] Kraid_CrumbleLeftPlatform_Middle [] {
        SpawnEprojWithGfx(
            0x80,
            word!(game.cur_enemy_index),
            addr_kEproj_RocksFallingKraidCeiling as u16,
            game,
        );
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x8, 0x12, 0xb7af as VoidP), game);
    };
    [10995455] Kraid_CrumbleRightPlatform_Right [] {
        SpawnEprojWithGfx(
            0x100,
            word!(game.cur_enemy_index),
            addr_kEproj_RocksFallingKraidCeiling as u16,
            game,
        );
        spawn_hardcoded(
            SpawnHardcodedPlmArgs::new(0x10, 0x12, 0xb7af as VoidP),
            game,
        );
    };
    @external {
        10995366 => [()];
    }
}
pub unsafe fn Kraid_RestrictSamusXtoFirstScreen(k: u16, game: &mut Game) {
    unsafe {
        Kraid_RestrictSamusXtoFirstScreen_2(game);
        KraidEnemy_HandleFunctionTimer(k, game);
    }
}
pub unsafe fn Kraid_UpdateBg2TilemapTopHalf(game: &mut Game) {
    unsafe {
        let v0: u16 = word!(game.vram_write_queue_tail);
        let mut v1 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v1).size = 2048;
        (*v1).src.addr = 0x2000 as VoidP;
        (*v1).src.bank = 126;
        (*v1).vram_dst = ((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8) as u16;
        word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
    }
}
pub unsafe fn Kraid_UpdateBG2TilemapBottomHalf(game: &mut Game) {
    unsafe {
        let v0: u16 = word!(game.vram_write_queue_tail);
        let mut v1 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v1).size = 2048;
        (*v1).src.addr = 10240 as VoidP;
        (*v1).src.bank = 126;
        (*v1).vram_dst = (((*ram_ptr!(game; 0x59) as i32 & 0xfc) << 8) + 2048) as u16;
        word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
    }
}
pub unsafe fn Kraid_RaiseKraidThroughFloor(k: u16, game: &mut Game) {
    unsafe {
        Kraid_RestrictSamusXtoFirstScreen_2(game);
        (*Get_Kraid(0i32, game)).kraid_var_A = (0xa7c89ai32) as u16;
        Kraid_UpdateBg2TilemapTopHalf(game);
    }
}
macro_rules! spawn_kraid_quake_projectiles_until_timer {
    ($game:ident, $interval_mask:literal, $next_state:literal, $next_timer:literal) => {
        Kraid_RestrictSamusXtoFirstScreen_2($game);
        let E = Get_Kraid(0, $game);
        let v1: u16 = ((*E).kraid_var_F).wrapping_sub(1);
        (*E).kraid_var_F = v1;
        if v1 != 0 {
            if (v1 & $interval_mask) == 0 {
                Kraid_SpawnRandomQuakeProjs($game);
            }
        } else {
            (*E).kraid_var_A = (($next_state) as u16);
            (*E).kraid_var_F = $next_timer;
        }
    };
}

pub unsafe fn Kraid_SpawnRandomQuakeProjs(game: &mut Game) {
    unsafe {
        let mut v0: u16 = (word!(game.random_number)) & (0x3fu16);
        if (word!(game.random_number) & 2) == 0 {
            v0 = !(v0 as i32) as u16;
        }
        let x: u16 = ((*Get_Kraid(0i32, game)).base.x_pos).wrapping_add(v0);
        let y: u16 = (448 - (((word!(game.random_number)) & (0x3f00u16)) as i32 >> 8)) as u16;
        create_sprite(x, y, 21, 0, game);
        let v1: u16 = (if (word!(game.random_number) & 0x10) != 0 {
            addr_kEproj_RocksWhenKraidRisesRight as i32
        } else {
            addr_kEproj_RocksWhenKraidRisesLeft as i32
        }) as u16;
        SpawnEprojWithGfx(
            (word!(game.random_number)) & (0x3f0u16),
            word!(game.cur_enemy_index),
            v1,
            game,
        );
    }
}
pub fn Kraid_RestrictSamusXtoFirstScreen_2(game: &mut Game) {
    {
        if ((word!(game.samus_x_pos)) as i16).wrapping_sub(256) >= 0 {
            word! { game, 256; samus_x_pos, samus_prev_x_pos }
        }
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallKraidFunc(ea: u32, game: &mut Game) => ea;
    setup { let k: u16 = word!(game.cur_enemy_index); }
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10988621] Kraid_GetsBig_BreakCeilingPlatforms [] {
        let mut kraid_var_F: u16 = 0;
        if (word!(game.nmi_frame_counter_word) & 7) == 0 {
            Kraid_SpawnRandomQuakeProjs(game);
        }
        let mut v0: u16 = 1;
        let E = Get_Kraid(0, game);
        if ((*E).base.y_pos & 2) != 0 {
            v0 = -1i32 as u16;
        }
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(v0);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
        if !(((*E).base.y_pos & 3) != 0 || {
            kraid_var_F = (*E).kraid_var_F;
            ((kraid_var_F) as i16).wrapping_sub(18) >= 0
        }) {
            SpawnEprojWithGfx(
                crate::rtl::rom_table_word!(game, 0xa7acb3, (kraid_var_F >> 1) as isize),
                word!(game.cur_enemy_index),
                addr_kEproj_RocksFallingKraidCeiling as u16,
                game,
            );
            match (*E).kraid_var_F as i32 >> 1 {
                0 => {
                    spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x6, 0x12, 0xb7b3 as VoidP), game)
                }
                1 => {
                    spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xd, 0x12, 0xb7ab as VoidP), game)
                }
                2 => {
                    spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x2, 0x12, 0xb7a3 as VoidP), game)
                }
                3 => {
                    spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xa, 0x12, 0xb7b3 as VoidP), game)
                }
                4 => {
                    spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x5, 0x12, 0xb7ab as VoidP), game)
                }
                5 => {
                    spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xc, 0x12, 0xb7b3 as VoidP), game)
                }
                6 => {
                    spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x3, 0x12, 0xb7ab as VoidP), game)
                }
                7 => {
                    spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xb, 0x12, 0xb7ab as VoidP), game)
                }
                8 => {
                    spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x4, 0x12, 0xb7b3 as VoidP), game)
                }
                _ => {}
            }
            (*E).kraid_var_F = ((*E).kraid_var_F).wrapping_add(2);
        }
        if (*E).base.y_pos as i32 - 296i32 & 0x8000 != 0 {
            (*E).kraid_var_A = (0xa7ad3ai32) as u16;
        }
    };
    [10988858] Kraid_GetsBig_SetBG2TilemapPrioBits [] {
        for v0 in (0u16..4096).step_by(2) {
            word!(game.tilemap_stuff[(v0 >> 1)] |= (0x2000u16));
        }
        let E = Get_Kraid(0x40, game);
        (*E).base.properties = (*E).base.properties & !(0x400);
        (*Get_Kraid(0i32, game)).kraid_var_A = (0xa7ad61i32) as u16;
        Kraid_UpdateBg2TilemapTopHalf(game);
    };
    [10988897] Kraid_GetsBig_FinishUpdateBg2Tilemap [] {
        (*Get_Kraid(0i32, game)).kraid_var_A = (0xa7ad8ei32) as u16;
        let E5 = Get_Kraid(0x140, game);
        enemy_instruction! {[(*E5).base]; timer = 1, instruction = addr_kKraid_Ilist_86ED as u16}
        let E2 = Get_Kraid(0x80, game);
        (*E2).base.current_instruction = addr_kKraid_Ilist_8B04 as u16;
        let E3 = Get_Kraid(0xc0, game);
        (*E3).base.current_instruction = addr_kKraid_Ilist_8B04 as u16;
        let E = Get_Kraid(0x100, game);
        (*E).base.current_instruction = addr_kKraid_Ilist_8B04 as u16;
        (*E2).base.spritemap_pointer = addr_kKraid_Sprmap_8C6C as u16;
        (*E3).base.spritemap_pointer = addr_kKraid_Sprmap_8C6C as u16;
        (*E).base.spritemap_pointer = addr_kKraid_Sprmap_8C6C as u16;
        Kraid_UpdateBG2TilemapBottomHalf(game);
    };
    [10989091] Kraid_GetsBig_FadeInRoomBg [] {
        if background6(game) & 1 != 0 {
            Kraid_GetsBig_Thinking_Setup(game);
            Kraid_Lints_Enable(0x80, g_word_A7A916, game);
            Kraid_Lints_Enable(0xc0, g_word_A7A918, game);
            Kraid_Lints_Enable(0x100, g_word_A7A91A, game);
            let E6 = Get_Kraid(0x180, game);
            (*E6).kraid_next = (0xa7bd60i32) as u16;
            let E7 = Get_Kraid(0x1c0, game);
            (*E7).kraid_next = (0xa7bd60i32) as u16;
            (*E6).kraid_var_A = (0xa7b92di32) as u16;
            (*E7).kraid_var_A = (0xa7b92di32) as u16;
            (*E6).kraid_var_F = 64;
            (*E7).kraid_var_F = 128;
            (*Get_Kraid(0x40i32, game)).kraid_var_C = 1;
            let E0 = Get_Kraid(0, game);
            (*E0).kraid_var_B = addr_stru_A796DA as u16;
            (*E0).kraid_target_x = 288;
            let E5 = Get_Kraid(0x140, game);
            (*E5).kraid_var_A = (0xa7bb6ei32) as u16;
            enemy_instruction! {[(*E5).base]; timer = 1, instruction = addr_kKraid_Ilist_8887 as u16}
        }
    };
    [10991319] Kraid_Shot_GlowHisEye [] {
        Kraid_ProcessKraidInstr(game);
        let mut v1: i16 = 0;
        for v0 in (226u16..232).step_by(2) {
            let v2: i32 = v0 as i32 >> 1;
            let mut v3: u16 = ((word!(game.palette_buffer[v2]) as i32 & 0x1f) + 1) as u16;
            if (word!(game.palette_buffer[v2]) as i32 & 0x1f) - 30i32 & 0x8000 == 0 {
                v1 += 1;
                v3 = 31;
            }
            let mut v4: u16 = ((word!(game.palette_buffer[v2]) as i32 & 0x3e0) + 32) as u16;
            if (word!(game.palette_buffer[v2]) as i32 & 0x3e0) - 960i32 & 0x8000 == 0 {
                v1 += 1;
                v4 = 992;
            }
            word!(game.palette_buffer[v2] = v4 | v3 | word!(game.palette_buffer[v2]) & 0xfc00);
        }
        if (v1).wrapping_sub(6) >= 0 {
            (*Get_Kraid(0i32, game)).kraid_var_A = (0xa7b73di32) as u16;
        }
    };
    [10991935] KraidEnemy_DecrementEnemyFunctionTimer [] {
        let E = Get_Kraid(word!(game.cur_enemy_index) as i32, game);
        if (*E).kraid_var_F != 0 {
            crate::update_place! { previous fresh8 = (*E).kraid_var_F => (*E).kraid_var_F.wrapping_sub(1); }
            if fresh8 == 1 {
                (*E).kraid_var_A = (*E).kraid_next;
                enemy_instruction! {[(*E).base]; instruction = addr_kKraid_Ilist_8887 as u16, timer = 1}
            }
        }
    };
    [10992174] KraidsFoot_SecondPhase_Thinking [] {
        let E5 = Get_Kraid(0x140, game);
        let v1: u16 = ((*E5).kraid_next).wrapping_sub(1);
        (*E5).kraid_next = v1;
        if v1 == 0 {
            let mut v2: u16 = 0;
            let mut E0 = null_mut::<Enemy_Kraid>();
            loop {
                E0 = Get_Kraid(0, game);
                if (*E0).base.x_pos
                    == crate::rtl::rom_table_word!(game, 0xa7ba7d, (v2 >> 1) as isize)
                {
                    break;
                }
                v2 = (v2).wrapping_add(4);
                if ((v2) as i16).wrapping_sub(24) < 0 {
                    continue;
                }
                v2 = 4;
                break;
            }
            let mut v4: i16 = ((word!(game.random_number)) as i16) & (0x1ci16);
            if v4 as i32 - 16i32 & 0x8000 == 0 {
                v4 = 16;
            }
            let v5 = RomPtr_A7(
                (crate::rtl::rom_table_word!(game, 0xa7ba7d, ((v2 as i32 >> 1) + 1) as isize))
                    .wrapping_add((v4) as u16),
                game,
            ) as *const u16;
            let v6: u16 = *v5.offset(1);
            if ((*v5) as i16).wrapping_sub(((*E0).base.x_pos) as i16) >= 0 {
                Kraid_SetWalkingBackwards(v6, *v5, game);
            } else {
                Kraid_SetWalkingForwards(v6, *v5, game);
            }
        }
    };
    [10992548] KraidsFoot_SecondPhase_Init [] {
        Kraid_SetWalkingBackwards(0xb4, 0x160, game);
    };
    [10992992] KraidsFingernail_Init [] {
        let mut kraid_var_E: u16 = (*Get_Kraid(0x180i32, game)).kraid_var_E;
        if word!(game.cur_enemy_index) == 384 {
            kraid_var_E = (*Get_Kraid(0x1c0i32, game)).kraid_var_E;
        }
        let mut v2: u16 = if kraid_var_E & 0x8000 != 0 {
            crate::rtl::rom_table_word!(
                game,
                0xa7be3e,
                ((word!(game.random_number) as i32 & 6) >> 1) as isize
            )
        } else {
            crate::rtl::rom_table_word!(
                game,
                0xa7be46,
                ((word!(game.random_number) as i32 & 6) >> 1) as isize
            )
        };
        let v3 = RomPtr_A7(v2, game);
        let E = Get_Kraid(word!(game.cur_enemy_index) as i32, game);
        set_fields! {[(*E)]; kraid_var_B = *(v3 as *mut u16),
            kraid_var_C = *(v3.offset(2) as *mut u16), kraid_var_D = *(v3.offset(4) as *mut u16),
            kraid_var_E = *(v3.offset(6) as *mut u16), kraid_parameter_1 = 1, };
        (*E).base.properties = ((*E).base.properties as i32
            & !(kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
            as u16;
        enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kKraid_Ilist_8B0A as u16}
        (*E).kraid_var_A = (0xa7be8ei32) as u16;
        let position_at_kraid: bool = ((word!(game.random_number) & 1) == 0)
            || ({
                let mut v7: u16 = (*Get_Kraid(0x180i32, game)).kraid_healths_8ths[1i32 as usize];
                if word!(game.cur_enemy_index) != 448 {
                    v7 = (*Get_Kraid(0x1c0i32, game)).kraid_healths_8ths[1i32 as usize];
                }
                (v7 == 1)
                    || ({
                        (*E).kraid_healths_8ths[1i32 as usize] = 1;
                        (*E).base.x_pos = 50;
                        (*E).base.y_pos = 240;
                        (*E).kraid_var_B = 0;
                        (*E).kraid_var_C = 1;
                        set_fields! {[(*E)], 0; kraid_var_D, kraid_var_E}
                        (*E).kraid_var_A = (0xa7b907i32) as u16;
                        (*E).kraid_next = (0xa7be8ei32) as u16;
                        (*E).base.properties = ((*E).base.properties as i32
                            | (kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
                            as u16;
                        false
                    })
            });
        if position_at_kraid {
            (*E).kraid_healths_8ths[1i32 as usize] = 0;
            let E0 = Get_Kraid(0, game);
            (*E).base.x_pos =
                ((*E0).base.x_pos as i32 - (*E0).base.x_width as i32 - (*E).base.x_width as i32
                    & 0xfff0) as u16;
            (*E).base.y_pos = ((*Get_Kraid(0x40i32, game)).base.y_pos).wrapping_add(128);
        };
    };
    [10993453] KraidsFoot_PrepareToLungeForward [] {
        Kraid_HandleFirstPhase(game);
        let v0 = gEnemyData(0, game);
        if (*v0.offset(1)).current_instruction as i32 + 0x75c9 & 0x8000 == 0 {
            enemy_instruction! {[(*v0.offset(1))]; instruction = addr_kKraid_Ilist_8AF0 as u16, timer = 1}
            enemy_instruction! {[(*v0.offset(5))]; timer = 1, instruction = addr_kKraid_Ilist_87BD as u16}
            (*v0.offset(5)).ai_var_A = (0xa7bf5di32) as u16;
            (*v0.offset(5)).ai_preinstr = 0;
        }
    };
    [10993501] KraidsFoot_FirstPhase_LungeForward [] {
        Kraid_HandleFirstPhase(game);
        let E = Get_Kraid(0, game);
        if (*E).base.x_pos as i32 - 92i32 & 0x8000 != 0 {
            (*E).base.x_pos = 92;
        }
        let E5 = Get_Kraid(0x140, game);
        if (*E5).base.current_instruction == (addr_off_A78885 as u16) {
            if (*E).base.x_pos == 92 {
                (*E5).kraid_next = (0xa7bfabi32) as u16;
                (*E5).kraid_var_A = (0xa7b93fi32) as u16;
                (*E5).kraid_var_F = 1;
                enemy_instruction! {[(*E5).base]; timer = 1, instruction = addr_kKraid_Ilist_86ED as u16}
            } else {
                enemy_instruction! {[(*E5).base]; timer = 1, instruction = addr_kKraid_Ilist_87BD as u16}
            }
        }
    };
    [10993579] KraidsFoot_FirstPhase_RetreatFromLunge [] {
        Kraid_HandleFirstPhase(game);
        let E0 = Get_Kraid(0, game);
        if (*E0).base.x_pos as i32 - 176i32 & 0x8000 == 0 {
            (*E0).base.x_pos = 176;
        }
        let E5 = Get_Kraid(0x140, game);
        if (*E5).base.current_instruction as i32 + 0x76c7 & 0x8000 == 0 {
            if (*E0).base.x_pos == 176 {
                let E1 = Get_Kraid(0x40, game);
                enemy_instruction! {[(*E1).base]; instruction = addr_kKraid_Ilist_89F3 as u16, timer = 1}
                enemy_instruction! {[(*E5).base]; timer = 1, instruction = addr_kKraid_Ilist_86ED as u16}
                (*E5).kraid_var_A = (0xa7b960i32) as u16;
                (*E5).kraid_var_F = 300;
                (*E5).kraid_next = (0xa7bf2di32) as u16;
            } else {
                printf(b"X undefined\n\0" as *const u8 as *const c_char);
                let E = Get_Kraid(word!(game.cur_enemy_index) as i32, game);
                enemy_instruction! {[(*E).base]; instruction = addr_kKraid_Ilist_8887 as u16, timer = 1}
            }
        }
    };
    [10994681] Kraid_Death_Fadeout [] {
        Kraid_ProcessKraidInstr(game);
        if background6(game) & 1 != 0 {
            let E = Get_Kraid(0, game);
            (*E).kraid_var_A = (0xa7c4a4i32) as u16;
            set_fields! {[(*E)], 1; kraid_hurt_frame_timer, kraid_hurt_frame}
            Kraid_Palette_Handling(game);
            word_transfers!(game, (4), 128; xray(0) <- value(0), xray(1) <- value(0));
            let v2: u16 = word!(game.vram_write_queue_tail);
            let mut v3 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
            (*v3).size = 512;
            (*v3).src.addr = 0x4000 as VoidP;
            (*v3).src.bank = 126;
            (*v3).vram_dst = (((*ram_ptr!(game; 0x5d) as i32 & 0xf) << 8) + 16128) as u16;
            word!(game.vram_write_queue_tail = (v2).wrapping_add(7));
        }
    };
    [10994852] Kraid_Death_UpdateBG2TilemapTopHalf [] {
        Kraid_ProcessKraidInstr(game);
        (*Get_Kraid(0i32, game)).kraid_var_A = (0xa7c4c8i32) as u16;
        let E2 = Get_Kraid(0x80, game);
        (*E2).kraid_var_A = (0xa7b923i32) as u16;
        (*E2).kraid_var_F = 0x7fff;
        let E3 = Get_Kraid(0xc0, game);
        (*E3).kraid_var_A = (0xa7b923i32) as u16;
        (*E3).kraid_var_F = 0x7fff;
        let E4 = Get_Kraid(0x100, game);
        (*E4).kraid_var_A = (0xa7b923i32) as u16;
        (*E4).kraid_var_F = 0x7fff;
        Kraid_UpdateBg2TilemapTopHalf(game);
    };
    [10994888] Kraid_Death_UpdateBG2TilemapBottomHalf [] {
        Kraid_ProcessKraidInstr(game);
        (*(ram_ptr!(game, LongPtr; 0x604))).bank = -89i32 as u8;
        (*(ram_ptr!(game, LongPtr; 0x604))).addr = (0xa7c2a0i32) as u16;
        let E = Get_Kraid(0, game);
        (*E).kraid_var_A = (0xa7c537i32) as u16;
        word!(game.kraid_unk9000 = 43);
        (*E).base.properties = ((*E).base.properties) | (0x8000u16);
        word! {game; earthquake_type = 1; earthquake_timer = 256; }
        let E1 = Get_Kraid(0x40, game);
        enemy_instruction! {[(*E1).base]; instruction = addr_kKraid_Ilist_8AA4 as u16, timer = 1}
        let E5 = Get_Kraid(0x140, game);
        enemy_instruction! {[(*E5).base]; instruction = addr_kKraid_Ilist_86E7 as u16, timer = 1}
        (*E5).kraid_var_A = (0xa7ba2di32) as u16;
        Kraid_UpdateBG2TilemapBottomHalf(game);
    };
    [10994999] Kraid_Death_SinkThroughFloor [] {
        Kraid_ProcessKraidInstr(game);
        Kraid_PlaySoundEveryHalfSecond(game);
        Kraid_HandleSinking(game);
        let E0 = Get_Kraid(0, game);
        (*E0).base.y_pos = (*E0).base.y_pos.wrapping_add(1);
        if (*E0).base.y_pos as i32 - 608i32 & 0x8000 == 0 {
            (*E0).base.properties =
                ((*E0).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
            word!(game.enemy_bg2_tilemap_size = 2);
            let enemy_ptr: u16 = (*Get_Kraid(word!(game.cur_enemy_index) as i32, game))
                .base
                .enemy_ptr;
            rom_record(0xa0, enemy_ptr, game).set_word(offset_of!(EnemyDef, shot_ai), (0xa7804ci32) as u16);
            #[cfg(sm_verify_dll)]
            crate::verify_probe::hit("actors.kraid.shot_ai.disabled");
            let E1 = Get_Kraid(0x40, game);
            let v3: u16 = ((*E1).base.properties as i32
                | kEnemyProps_Tangible as i32
                | kEnemyProps_Deleted as i32) as u16;
            (*E1).base.properties = v3;
            let v4: u16 = v3 & 0x51ff | 0x600;
            (*Get_Kraid(0x80i32, game)).base.properties = v4;
            (*Get_Kraid(0xc0i32, game)).base.properties = v4;
            (*Get_Kraid(0x100i32, game)).base.properties = v4;
            (*Get_Kraid(0x140i32, game)).base.properties = v4;
            (*Get_Kraid(0i32, game)).kraid_var_A = (0xa7c715i32) as u16;
            word!(game.camera_distance_index = 0);
            scatter(DropSource::Kraid, enemy_ptr, game);
            Kraid_DrawRoomBg(game);
        }
    };
    [10995477] Kraid_FadeInBg_ClearBg2TilemapTopHalf [] {
        *ram_ptr!(game; 0x59) = 72;
        word_transfers!(game, descending, 1024; le(tilemap_stuff, 1023) <- value(824));
        let v1: u16 = word!(game.vram_write_queue_tail);
        let mut v2 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v2).size = 1024;
        (*v2).src.addr = 0x2000 as VoidP;
        *(&raw mut (*v2).src.bank as *mut u16) = 126;
        (*v2).vram_dst = 18432;
        word!(game.vram_write_queue_tail = (v1).wrapping_add(7));
        (*Get_Kraid(0i32, game)).kraid_var_A = (0xa7c751i32) as u16;
    };
    [10995537] Kraid_FadeInBg_ClearBg2TilemapBottomHalf [] {
        let v0: u16 = word!(game.vram_write_queue_tail);
        let mut v1 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v1).size = 1024;
        (*v1).src.addr = 0x2000 as VoidP;
        *(&raw mut (*v1).src.bank as *mut u16) = 126;
        (*v1).vram_dst = 18944;
        word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
        (*Get_Kraid(0i32, game)).kraid_var_A = (0xa7c777i32) as u16;
    };
    [10995575] Kraid_FadeInBg_LoadBg3Tiles1of4 [] {
        (*(ram_ptr!(game, LongPtr; 0x604))).addr = (0xa7c1fbi32) as u16;
        (*Get_Kraid(0i32, game)).kraid_var_A = (0xa7c7a3i32) as u16;
        let v0: u16 = word!(game.vram_write_queue_tail);
        let mut v1 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v1).size = 1024;
        (*v1).src.addr = -19968i32 as VoidP;
        *(&raw mut (*v1).src.bank as *mut u16) = 154;
        (*v1).vram_dst = 0x4000;
        word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
    };
    [10995619] Kraid_FadeInBg_LoadBg3Tiles2of4 [] {
        (*Get_Kraid(0i32, game)).kraid_var_A = (0xa7c7c9i32) as u16;
        let v0: u16 = word!(game.vram_write_queue_tail);
        let mut v1 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v1).size = 1024;
        (*v1).src.addr = -18944i32 as VoidP;
        *(&raw mut (*v1).src.bank as *mut u16) = 154;
        (*v1).vram_dst = 16896;
        word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
    };
    [10995657] Kraid_FadeInBg_LoadBg3Tiles3of4 [] {
        (*Get_Kraid(0i32, game)).kraid_var_A = (0xa7c7efi32) as u16;
        let v0: u16 = word!(game.vram_write_queue_tail);
        let mut v1 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v1).size = 1024;
        (*v1).src.addr = -17920i32 as VoidP;
        *(&raw mut (*v1).src.bank as *mut u16) = 154;
        (*v1).vram_dst = 17408;
        word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
    };
    [10995695] Kraid_FadeInBg_LoadBg3Tiles4of4 [] {
        (*Get_Kraid(0i32, game)).kraid_var_A = (0xa7c815i32) as u16;
        let v0: u16 = word!(game.vram_write_queue_tail);
        let mut v1 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
        (*v1).size = 1024;
        (*v1).src.addr = -16896i32 as VoidP;
        *(&raw mut (*v1).src.bank as *mut u16) = 154;
        (*v1).vram_dst = 17920;
        word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
    };
    [10995733] Kraid_FadeInBg_FadeInBp6 [] {
        #[cfg(sm_verify_dll)]
        crate::kraid_flags_observer::observe(0, game);
        if background6(game) & 1 != 0 {
            #[cfg(sm_verify_dll)]
            crate::kraid_flags_observer::observe(1, game);
            QueueMusic_Delayed8(3, game);
            #[cfg(sm_verify_dll)]
            crate::kraid_flags_observer::observe(2, game);
            let mut v0: i16 =
                gameplay_flag_runtime::boss_word_at(word!(game.area_index), BossWordOrder::Native, game) as i16;
            if (v0 as u16 & 1) != 0 {
                (*Get_Kraid(0i32, game)).kraid_var_A = (0xa7c851i32) as u16;
            } else {
                gameplay_flag_runtime::set_boss_word_native(
                    word!(game.area_index), ((v0) as u16) | (1u16), game,
                );
                (*Get_Kraid(0i32, game)).kraid_var_A = (0xa7c843i32) as u16;
            }
        }
        #[cfg(sm_verify_dll)]
        crate::kraid_flags_observer::observe(3, game);
    };
    [10995779] Kraid_FadeInBg_SetEnemyDead_KraidWasAlive [] {
        if Kraid_CheckIfDead(game) == 0 || word!(game.layer1_x_pos) != 0 {
            Kraid_SetEnemyPropsToDead(game);
        }
    };
    [10995793] Kraid_FadeInBg_SetEnemyDead_KraidWasDead [] {
        if Kraid_CheckIfDead(game) == 0 || word!(game.layer1_x_pos) != 0 {
            Kraid_SetEnemyPropsToDead(game);
        }
        word!(game.previous_layer1_x_block = -1i32 as u16);
    };
    [10995866] Kraid_Raise_LoadTilemapBottomAndShake [] {
        Kraid_RestrictSamusXtoFirstScreen_2(game);
        let E = Get_Kraid(0, game);
        (*E).kraid_var_A = (0xa7c8e0i32) as u16;
        (*E).kraid_var_F = 120;
        word!(game.earthquake_timer = 496);
        QueueMusic_Delayed8(5, game);
        Kraid_UpdateBG2TilemapBottomHalf(game);
    };
    [10995936] Kraid_Raise_SpawnRandomEarthquakeProjs16 [] {
        spawn_kraid_quake_projectiles_until_timer!(game, 0xf, 0xa7c902i32, 96);
    };
    [10995970] Kraid_Raise_SpawnRandomEarthquakeProjs8 [] {
        spawn_kraid_quake_projectiles_until_timer!(game, 7, 0xa7c924i32, 288);
    };
    [10996004] Kraid_Raise_Handler [] {
        Kraid_RestrictSamusXtoFirstScreen_2(game);
        if (word!(game.earthquake_timer) & 5) == 0 {
            Kraid_SpawnRandomQuakeProjs(game);
        }
        let E0 = Get_Kraid(0, game);
        (*E0).base.x_pos = ((*E0).base.x_pos as i32
            + if ((*E0).base.y_pos & 2) == 0 {
                -1i32
            } else {
                1
            }) as u16;
        words!((*E0).base, y_pos, y_subpos += -0x8000i32 as u32);
        if (*E0).base.y_pos as i32 - 457i32 & 0x8000 != 0 {
            (*E0).base.x_pos = 176;
            let E5 = Get_Kraid(0x140, game);
            (*E5).kraid_var_A = (0xa7b960i32) as u16;
            (*E5).kraid_var_F = 300;
            (*E5).kraid_next = (0xa7bf2di32) as u16;
            (*E0).kraid_var_B = addr_stru_A796DA as u16;
            Kraid_Mainloop_Thinking_Setup(game);
            let E1 = Get_Kraid(0x40, game);
            enemy_instruction! {[(*E1).base]; instruction = addr_kKraid_Ilist_89F3 as u16, timer = 1}
        }
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = Get_Kraid(0, game); };
    [10988942] Kraid_GetsBig_DrawRoomBg [] {
        (*E).kraid_var_A = (0xa7ae23i32) as u16;
        set_fields! {[(*E)], 0; kraid_var_E, kraid_var_F}
        Kraid_DrawRoomBg(game);
    };
    Kraid_SetLintYAndRandomThinkTimer [] {
        (*Get_Kraid(0x80i32, game)).base.y_pos = ((*E).base.y_pos).wrapping_sub(20);
        (*Get_Kraid(0xc0i32, game)).base.y_pos = ((*E).base.y_pos).wrapping_add(46);
        (*Get_Kraid(0x100i32, game)).base.y_pos = ((*E).base.y_pos).wrapping_add(112);
        let mut v1: i16 = ((word!(game.random_number)) as i16) & (7i16);
        if (word!(game.random_number) & 7) == 0 {
            v1 = 2;
        }
        (*E).kraid_thinking = ((v1 as i32) << 6) as u16;
    };
    [10989220] Kraid_Mainloop_Thinking [] {
        advance_kraid_thinking_timer!(E, 0xa7bbeai32, game);
    };
    [10989252] Kraid_GetsBig_Thinking [] {
        advance_kraid_thinking_timer!(E, 0xa7aee4i32, game);
    };
    [10989284] Kraid_Shot_MouthIsOpen [] {
        if Kraid_ProcessKraidInstr(game) == 0xffff {
            (*E).kraid_var_A = (0xa7aea4i32) as u16;
            advance_kraid_mouth_hit_response!(E);
        }
    };
    Kraid_Enemy_Touch [] {
        if (*E).kraid_var_A as i32 + 0x3ca0 & 0x8000 != 0 {
            let r18: u16 = (word!(game.samus_x_radius)).wrapping_add(word!(game.samus_x_pos));
            let mut v1: i16 = ((word!(game.samus_y_pos)) as i16).wrapping_sub(((*E).base.y_pos) as i16);
            let mut i: i32 = 0;
            loop {
                let v3: i32 = i >> 1;
                if (v1).wrapping_sub((crate::rtl::rom_table_word!(game, 0xa7b161, v3 as isize)) as i16)
                    >= 0
                    || (v1).wrapping_sub(
                        (crate::rtl::rom_table_word!(game, 0xa7b161, (v3 + 2) as isize)) as i16,
                    ) >= 0
                {
                    break;
                }
                i += 4;
            }
            if (((*E).base.x_pos as i32
                + crate::rtl::rom_table_word!(game, 0xa7b161, ((i >> 1) + 1) as isize) as i32
                - r18 as i32) as i16)
                < 0
            {
                if word!(game.samus_x_pos) as i32 - 40i32 & 0x8000 == 0 {
                    word!(game.samus_x_pos -= 8);
                    word!(game.samus_prev_x_pos = word!(game.samus_x_pos));
                }
                let mut kraid_min_y_pos_eject: u16 = (word!(game.samus_y_pos)).wrapping_sub(8);
                if ((word!(game.samus_y_pos) as i32 - 8i32 - (*E).kraid_min_y_pos_eject as i32) as i16)
                    < 0
                {
                    kraid_min_y_pos_eject = (*E).kraid_min_y_pos_eject;
                }
                word!(game.samus_y_pos = kraid_min_y_pos_eject);
                word!(game.samus_prev_y_pos = kraid_min_y_pos_eject);
                Kraid_Func_1(game);
                if word!(game.samus_invincibility_timer) == 0 {
                    actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
                }
            }
        }
    };
    Kraid_Shot_Body [] {
        if (*E).kraid_var_A as i32 + 0x3ac9 & 0x8000 != 0 {
            (*E).kraid_var_E = 0;
            (*E).kraid_mouth_flags = (*E).kraid_mouth_flags & !(1);
            let mut R48: u16 = 0;
            let v2 = RomPtr_A7(((*E).kraid_var_B).wrapping_sub(8), game);
            let v3 = RomPtr_A7(*(v2.offset(4) as *mut u16), game);
            let r22: u16 = ((*E).base.x_pos).wrapping_add(*(v3 as *mut u16));
            let r20: u16 = ((*E).base.y_pos).wrapping_add(*(v3.offset(2) as *mut u16));
            let mut r18: u16 = ((*E).base.y_pos).wrapping_add(*(v3.offset(6) as *mut u16));
            if word!(game.projectile_counter) != 0 {
                let mut i: i32 = 2 * word!(game.projectile_counter) as i32;
                while i >= 0 {
                    let v5: i32 = i >> 1;

                    let hit_body: bool = if word!(game.projectile_y_pos[v5]) as i32
                        - word!(game.projectile_y_radius[v5]) as i32
                        - 1i32
                        - r18 as i32
                        & 0x8000
                        != 0
                    {
                        word!(game.projectile_y_radius[v5]) as i32
                            + word!(game.projectile_y_pos[v5]) as i32
                            - r20 as i32
                            & 0x8000
                            == 0
                            && word!(game.projectile_x_radius[v5]) as i32
                                + word!(game.projectile_x_pos[v5]) as i32
                                - r22 as i32
                                & 0x8000
                                == 0
                    } else {
                        let mut v10: i32 = i >> 1;
                        r18 = (word!(game.projectile_x_radius[v10]))
                            .wrapping_add(word!(game.projectile_x_pos[v10]));
                        let mut v11: i16 = ((word!(game.projectile_y_pos[v10])) as i16)
                            .wrapping_sub(((*E).base.y_pos) as i16);
                        let mut j: u16 = 0;
                        loop {
                            let v13: i32 = j as i32 >> 1;
                            if (v11).wrapping_sub(
                                (crate::rtl::rom_table_word!(game, 0xa7b161, v13 as isize)) as i16,
                            ) >= 0
                                || (v11).wrapping_sub(
                                    (crate::rtl::rom_table_word!(game, 0xa7b161, (v13 + 2) as isize))
                                        as i16,
                                ) >= 0
                            {
                                break;
                            }
                            j = (j).wrapping_add(4);
                        }
                        (((*E).base.x_pos as i32
                            + crate::rtl::rom_table_word!(
                                game,
                                0xa7b161,
                                ((j as i32 >> 1) + 1) as isize
                            ) as i32
                            - r18 as i32) as i16)
                            < 0
                    };
                    if hit_body {
                        Kraid_SpawnExplosionEproj(i as u16, game);
                        let v6: i32 = i >> 1;
                        word!(game.projectile_dir[v6] |= (0x10u16));
                        if (word!(game.projectile_type[v6]) & 0x10) != 0 {
                            (*E).kraid_mouth_flags = ((*E).kraid_mouth_flags) | (1u16);
                        }
                        R48 = R48.wrapping_add(1);
                    }
                    i -= 2;
                }
            }
            if R48 != 0 && (*E).kraid_var_A as i32 == 0xa7aea4 & 0xffff {
                (*E).kraid_var_A = (0xa7b6bfi32) as u16;
                let kraid_mouth_flags: u16 = (*E).kraid_mouth_flags;
                if (kraid_mouth_flags & 1) != 0 {
                    (*E).kraid_mouth_flags = (kraid_mouth_flags) | (0x302u16);
                }
            }
        }
    };
    Kraid_Palette_Handling [] {
        if (*E).base.health as i32 - 1i32 & 0x8000 != 0 {
            (*E).kraid_hurt_frame = (*E).base.health;
        } else {
            let continue_palette_update: bool = ((*E).kraid_hurt_frame != 0)
                && ({
                    let v1: u16 = ((*E).kraid_hurt_frame_timer).wrapping_sub(1);
                    (*E).kraid_hurt_frame_timer = v1;
                    (v1 == 0)
                        && ({
                            (*E).kraid_hurt_frame_timer = 2;
                            (*E).kraid_hurt_frame = (*E).kraid_hurt_frame.wrapping_sub(1);
                            true
                        })
                });
            if !(continue_palette_update) {
                return;
            }
        }
        Kraid_HurtFlash_Handling(game);
        Kraid_HealthBasedPaletteHandling(game);
    };
    [10991295] Kraid_InitEyeGlowing [] {
        (*E).kraid_var_A = (0xa7aee4i32) as u16;
        (*E).kraid_var_A = (0xa7b6d7i32) as u16;
        (*E).kraid_var_B = (addr_stru_A7974A as i32 + 8) as u16;
        (*E).kraid_var_C = (*(rom_ptr!(game, KraidInstrList; 0xa7974a))).timer;
        Kraid_Shot_GlowHisEye(game);
    };
    [10991421] Kraid_Shot_UnglowEye [] {
        let mut v0: u16 = 14;
        let health: u16 = (*E).base.health;
        while ((health) as i16)
            .wrapping_sub(((*Get_Kraid(v0 as i32, game)).kraid_healths_8ths[0i32 as usize]) as i16)
            < 0
        {
            v0 = (v0).wrapping_sub(2);
            if v0 == 0 {
                break;
            }
        }
        let mut v2: u16 = (16 * (v0 as i32 + 2)) as u16;
        let mut v3: u16 = 226;
        let mut r20: u16 = 0;
        do_until! { ((v3) as i16).wrapping_sub(232) >= 0 ;
    let v4: i32 = v3 as i32 >> 1;
    let mut r18: u16 = (word!(game.palette_buffer[v4])) & (0x1fu16);
    let v5: i32 = v2 as i32 >> 1;
    if crate::rtl::rom_table_word!(game, 0xa7b3d3, (v5 + 1) as isize) & 0x1f != r18 {
        r20 = r20.wrapping_add(1);
        word!(game.palette_buffer[v4] -= 1);
    }
    r18 = (word!(game.palette_buffer[v4])) & (0x3e0u16);
    if crate::rtl::rom_table_word!(game, 0xa7b3d3, (v5 + 1) as isize) & 0x3e0 != r18 {
        r20 = r20.wrapping_add(1);
        word!(game.palette_buffer[v4] -= 32);
    }
    v3 = (v3).wrapping_add(2);
    v2 = (v2).wrapping_add(2);
}
        if r20 == 0 {
            (*E).kraid_var_A = (0xa7aee4i32) as u16;
            (*E).kraid_var_B = addr_stru_A796DA as u16;
            (*E).kraid_var_C = (*(rom_ptr!(game, KraidInstrList; 0xa796d2))).timer;
        }
    };
    [10992618] Kraid_Main_AttackWithMouthOpen [] {
        if Kraid_ProcessKraidInstr(game) == 0xffff {
            Kraid_Mainloop_Thinking_Setup(game);
            advance_kraid_mouth_hit_response!(E);
        } else if *(RomPtr_A7(((*E).kraid_var_B).wrapping_sub(8u16), game) as *mut u16).offset(1) as i32
            == addr_kKraidTilemaps_3 as i32
            && ((*E).kraid_var_C & 0xf) == 0
        {
            SpawnEprojWithGfx(
                crate::rtl::rom_table_word!(
                    game,
                    0xa7bc65,
                    ((word!(game.random_number) as i32 & 0xe) >> 1) as isize
                ),
                (word!(game.random_number)) & (0xeu16),
                addr_kEproj_RocksKraidSpits as u16,
                game,
            );
            QueueSfx3_Max6(0x1e, game);
        }
    };
    [10993825] Kraid_GetsBig_ReleaseCamera [] {
        (*E).kraid_var_A = (0xa7ac4di32) as u16;
        *(ram_ptr!(game, u16; 0xcd20)) = 514;
        *(ram_ptr!(game, u16; 0xcd20, 2)) = 257;
        (*E).kraid_min_y_pos_eject = 164;
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E0 = Get_Kraid(0, game); };
    [10992453] KraidsFoot_SecondPhase_WalkingBackwards [] {
        let kraid_target_x: u16 = (*E0).kraid_target_x;
        if kraid_target_x != (*E0).base.x_pos {
            if ((kraid_target_x) as i16).wrapping_sub(((*E0).base.x_pos) as i16) >= 0 {
                return;
            }
            (*E0).base.x_pos = kraid_target_x;
        }
        let E5 = Get_Kraid(0x140, game);
        if (*E5).base.current_instruction as i32 + 0x76c7 & 0x8000 == 0 {
            (*E5).kraid_var_A = (0xa7ba2ei32) as u16;
            enemy_instruction! {[(*E5).base]; timer = 1, instruction = addr_kKraid_Ilist_86ED as u16}
        }
    };
    [10992494] KraidsFoot_SecondPhaseSetup_WalkToStartPt [] {
        let kraid_target_x: u16 = (*E0).kraid_target_x;
        if kraid_target_x != (*E0).base.x_pos {
            if ((kraid_target_x) as i16).wrapping_sub(((*E0).base.x_pos) as i16) >= 0 {
                return;
            }
            (*E0).base.x_pos = kraid_target_x;
        }
        let E5 = Get_Kraid(0x140, game);
        if (*E5).base.current_instruction as i32 + 0x76c7 & 0x8000 == 0 {
            (*E5).kraid_var_A = (0xa7b92di32) as u16;
            (*E5).kraid_var_F = 180;
            (*E5).kraid_next = (0xa7bba4i32) as u16;
            enemy_instruction! {[(*E5).base]; timer = 1, instruction = addr_kKraid_Ilist_86ED as u16}
        }
    };
    [10992558] KraidsFoot_SecondPhase_WalkForward [] {
        let kraid_target_x: u16 = (*E0).kraid_target_x;
        if ((kraid_target_x) as i16).wrapping_sub(((*E0).base.x_pos) as i16) < 0 {
            let E5 = Get_Kraid(0x140, game);
            if (*E5).base.current_instruction == (addr_off_A787BB as u16) {
                enemy_instruction! {[(*E5).base]; instruction = addr_kKraid_Ilist_86F3 as u16, timer = 1}
            }
        } else {
            (*E0).base.x_pos = kraid_target_x;
            let E5_0 = Get_Kraid(0x140, game);
            if (*E5_0).base.current_instruction == (addr_off_A787BB as u16) {
                (*E5_0).kraid_var_A = (0xa7ba2ei32) as u16;
                enemy_instruction! {[(*E5_0).base]; timer = 1, instruction = addr_kKraid_Ilist_86ED as u16}
            }
        };
    };
    Kraid_HandleFirstPhase [] {
        if (((*E0).base.health) as i16).wrapping_sub(((*E0).kraid_healths_8ths[6i32 as usize]) as i16)
            < 0
        {
            (*E0).kraid_var_A = (0xa7b965i32) as u16;
            (*E0).kraid_var_F = 180;
            (*E0).kraid_next = (0xa7c0a1i32) as u16;
            let v2: u16 = *(RomPtr_A7((*E0).kraid_var_B, game) as *mut u16).offset(1);
            let mut v3: u16 = 50;
            if v2 as i32 != addr_kKraidTilemaps_0 as i32 {
                v3 = 42;
                if v2 as i32 != addr_kKraidTilemaps_1 as i32 {
                    v3 = 34;
                    if v2 as i32 != addr_kKraidTilemaps_2 as i32 {
                        v3 = 26;
                    }
                }
            }
            (*E0).kraid_var_B = (v3).wrapping_sub(26918);
            (*E0).kraid_var_C = *((&raw mut (*(rom_ptr!(game, KraidInstrList; 0xa796d2))).timer
                as *mut u8)
                .offset(v3 as i32 as isize) as *mut u16);
            word! {game; earthquake_type = 4; earthquake_timer = 340; }
            let E5 = Get_Kraid(0x140, game);
            enemy_instruction! {[(*E5).base]; instruction = addr_kKraid_Ilist_86E7 as u16, timer = 1}
            (*E5).kraid_var_A = (0xa7ba2di32) as u16;
            let E1 = Get_Kraid(0x40, game);
            enemy_instruction! {[(*E1).base]; instruction = addr_kKraid_Ilist_89F3 as u16, timer = 1}
            let E2 = Get_Kraid(0x80, game);
            (*E2).base.properties = ((*E2).base.properties) | (0x100u16);
            let E3 = Get_Kraid(0xc0, game);
            (*E3).base.properties = ((*E3).base.properties) | (0x100u16);
            let E4 = Get_Kraid(0x100, game);
            (*E4).base.properties = ((*E4).base.properties) | (0x100u16);
            (*E1).base.properties = ((*E1).base.properties) | (0x400u16);
        }
    };
    [10994528] Kraid_Death_Init [] {
        if (*E0).kraid_hurt_frame == 0 {
            word_transfers!(game, ascending, 16; le(target_palettes, 96) <- value(0));
            word_transfers!(game, descending, 16; le(palette_buffer, 127) <- rom(0xa7b4f3, 15));
            let E1 = Get_Kraid(0x40, game);
            enemy_instruction! {[(*E1).base]; instruction = addr_kKraid_Ilist_8AF0 as u16, timer = 1}
            (*E0).kraid_var_A = (0xa7c3f9i32) as u16;
            (*E0).kraid_var_B = (addr_stru_A79764 as i32 + 8) as u16;
            (*E0).kraid_var_C = (*(rom_ptr!(game, KraidInstrList; 0xa79764))).timer;
            let v4: u16 = word!(game.cur_enemy_index);
            let v7: u16 = word!(game.cur_enemy_index);
            let E6 = Get_Kraid(0x180, game);
            (*E6).base.properties = (*E6).base.properties & !(0x4000);
            word!(game.cur_enemy_index = 384);
            ({ let (_ignored_key, animation) = (v4, 0x180); actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Normal { animation }) });
            let E7 = Get_Kraid(0x1c0, game);
            (*E7).base.properties = (*E7).base.properties & !(0x4000);
            word!(game.cur_enemy_index = 448);
            ({ let (_ignored_key, animation) = (v4, 0x1c0); actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Normal { animation }) });
            word!(game.cur_enemy_index = 128);
            ({ let (_ignored_key, animation) = (v4, 0x80); actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Normal { animation }) });
            word!(game.cur_enemy_index = 192);
            ({ let (_ignored_key, animation) = (v4, 0xc0); actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Normal { animation }) });
            word!(game.cur_enemy_index = 256);
            ({ let (_ignored_key, animation) = (v4, 0x100); actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Normal { animation }) });
            word!(game.cur_enemy_index = v7);
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x5, 0x1b, 0xb7bf as VoidP), game);
        }
    };
    @external {
        10991666 => [KraidLint_ProduceLint(k, game)];
        10991771 => [KraidLint_FireLint(k, game)];
        10991879 => [KraidFingernail_WaitForLintXpos(k, game)];
        10991907 => [Kraid_AlignEnemyToKraid(k, game)];
        10991917 => [KraidEnemy_HandleFunctionTimer(k, game)];
        10991968 => [KraidFoot_FirstPhase_Thinking(k, game)];
        10991973 => [KraidEnemy_ProcessInstrEnemyTimer(k, game)];
        10993294 => [KraidsFingernail_Fire(k, game)];
        10995813 => [Kraid_RestrictSamusXtoFirstScreen(k, game)];
        10995819 => [Kraid_RaiseKraidThroughFloor(k, game)];
        10991720 => [KraidLint_ChargeLint(k, game)]; 10992173 => [()];
        10991665 => [()];
    }
}
pub unsafe fn Phantoon_Func_2(k: u16, game: &mut Game) {
    unsafe {
        if k == 0 && (word!(game.joypad1_newkeys) & 0x4000) != 0 {
            *((&raw mut (*(rom_ptr!(game, ExtendedSpriteMap; 0xa7902d))).ypos as *mut u8).offset(1)
                as *mut u16) = (*((&raw mut (*(rom_ptr!(game, ExtendedSpriteMap; 0xa7902d))).ypos
                as *mut u8)
                .offset(1) as *mut u16)
                == 0) as u16;
        }
    }
}
pub unsafe fn Phantoon_Func_3(amt: i32, game: &mut Game) -> u8 {
    unsafe {
        let E = Get_Phantoon(0xc0, game);
        let r18: u16 = amt as u16;
        let r20: u16 = (amt >> 16) as u16;
        if (*E).var_E != 0 {
            let phant_var_D: u16 = (*E).var_D;
            let v4: bool = phant_var_D < r18;
            (*E).var_D = (phant_var_D).wrapping_sub(r18);
            if v4 {
                (*E).var_D = 0;
                return 1;
            }
        } else {
            let r22: u16 = ((r20 as i32 & 0xff00) >> 8) as u16;
            let mut v1: u16 = (r18).wrapping_add((*E).var_D);
            (*E).var_D = v1;
            v1 = ((v1 as i32 & 0xff00) >> 8) as u16;
            if v1 as i32 - r22 as i32 & 0x8000 == 0 {
                (*E).var_D = r20;
            }
        }
        0
    }
}
pub unsafe fn Phantoon_Func_4(k: u16, game: &mut Game) {
    {
        SpawnEprojWithGfx(0, k, addr_kEproj_DestroyableFireballs as u16, game);
        QueueSfx3_Max6(0x1d, game);
    }
}
pub unsafe fn Phantoon_Func_5(k: u16, game: &mut Game) {
    {
        for i in (0i32..=7).rev() {
            SpawnEprojWithGfx(
                (i | 0x600) as u16,
                k,
                addr_kEproj_DestroyableFireballs as u16,
                game,
            );
        }
        QueueSfx3_Max6(0x28, game);
    }
}
pub unsafe fn Phantoon_Func_6(k: u16, a: u16, game: &mut Game) {
    unsafe {
        let mut v2: i16 = byte!(@pointer rom_ptr!(game; 0xa7cfc2); a as isize) as i16;
        let mut r20: i32 = 16;
        for _ in 0..=7 {
            let mut v3: i16 = v2;
            crate::update_place! { previous fresh25 = v2 => v2 + 1; }
            SpawnEprojWithGfx(
                (r20 | fresh25 as i32 | 0x400) as u16,
                k,
                addr_kEproj_DestroyableFireballs as u16,
                game,
            );
            if (v3).wrapping_sub(8) >= 0 {
                v2 = 0;
            }
            r20 += 16;
        }
    }
}
pub unsafe fn Phantoon_Func_7(k: u16, game: &mut Game) {
    unsafe {
        let E = Get_Phantoon(k as i32 + 192, game);
        let mut v3: bool = (*E).var_B == 1;
        (*E).var_B = (*E).var_B.wrapping_sub(1);
        let v4: bool = ((*E).var_B & 0x8000) != 0;
        if v3 || v4 {
            if ((*E).var_C & 0x8000) == 0 {
                v3 = (*E).var_C == 1;
                (*E).var_C = (*E).var_C.wrapping_sub(1);
                let v8: bool = ((*E).var_C & 0x8000) != 0;
                if v3 || v8 {
                    (*E).var_C = -1i32 as u16;
                    (*E).var_B = *(RomPtr_A7(
                        crate::rtl::rom_table_word!(game, 0xa7ccfd, (*E).var_A as isize),
                        game,
                    ) as *mut u16)
                        .offset(1);
                } else {
                    (*E).var_B = *(RomPtr_A7(
                        (2i32 * (*E).var_C as i32
                            + crate::rtl::rom_table_word!(game, 0xa7ccfd, (*E).var_A as isize)
                                as i32) as u16,
                        game,
                    ) as *mut u16)
                        .offset(1);
                }
                enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kKraid_Ilist_CCEB as u16}
            } else {
                let v5: u16 = (game.next_random()) & (3u16);
                (*E).var_A = v5;
                let v6: u16 = crate::rtl::rom_table_word!(game, 0xa7ccfd, v5 as isize);
                let v7: u16 = *(RomPtr_A7(v6, game) as *mut u16);
                (*E).var_C = v7;
                (*E).var_B =
                    *(RomPtr_A7((2i32 * v7 as i32 + v6 as i32) as u16, game) as *mut u16).offset(1);
            }
        }
    }
}
pub unsafe fn Phantoon_AdjustSpeedAndMoveInFigure8(game: &mut Game) {
    unsafe {
        if (*Get_Phantoon(0x40i32, game)).var_C != 0 {
            Phantoon_AdjustSpeedRightSideClockwise(game);
            Phantoon_MoveInFigure8_RightSideClockwise(addr_kPhantoonMoveData as u16, 533, game);
        } else {
            Phantoon_AdjustSpeedLeftSideClockwise(game);
            Phantoon_MoveInFigure8_LeftSideClockwise(addr_kPhantoonMoveData as u16, 534, game);
        };
    }
}
pub unsafe fn Phantoon_AdjustSpeedLeftSideClockwise(game: &mut Game) {
    unsafe {
        let E = Get_Phantoon(0, game);
        let phant_var_D: u16 = (*E).var_D;
        if phant_var_D != 0 {
            if (phant_var_D & 1) != 0 {
                words!((*E), var_C, var_B += words!((g_word_A7CD79, g_word_A7CD77)));
                if (((*E).var_C) as i16).wrapping_sub((g_word_A7CD7D) as i16) >= 0 {
                    (*E).var_C = g_word_A7CD7D;
                    (*E).var_B = 0;
                    (*E).var_D = (*E).var_D.wrapping_add(1);
                }
            } else {
                words!(
                    (*E),
                    var_C,
                    var_B += -(words!(i32, (g_word_A7CD79, g_word_A7CD77))) as u32
                );
                if (*E).var_C == g_word_A7CD7F
                    || (((*E).var_C) as i16).wrapping_sub((g_word_A7CD7F) as i16) < 0
                {
                    (*E).var_C = (g_word_A7CD7F).wrapping_add(1);
                    set_fields! {[(*E)], 0; var_B, var_D}
                }
            }
        } else {
            words!((*E), var_C, var_B += words!((g_word_A7CD75, g_word_A7CD73)));
            if (((*E).var_C) as i16).wrapping_sub((g_word_A7CD7B) as i16) >= 0 {
                (*E).var_C = (g_word_A7CD7B).wrapping_sub(1);
                (*E).var_B = 0;
                (*E).var_D = (*E).var_D.wrapping_add(1);
            }
        };
    }
}
pub unsafe fn Phantoon_AdjustSpeedRightSideClockwise(game: &mut Game) {
    unsafe {
        let E = Get_Phantoon(0, game);
        let phant_var_D: u16 = (*E).var_D;
        if phant_var_D != 0 {
            if (phant_var_D & 1) != 0 {
                words!(
                    (*E),
                    var_C,
                    var_B += -(words!(i32, (g_word_A7CD87, g_word_A7CD85))) as u32
                );
                if (*E).var_C == g_word_A7CD8B
                    || (((*E).var_C) as i16).wrapping_sub((g_word_A7CD8B) as i16) < 0
                {
                    (*E).var_C = (g_word_A7CD8B).wrapping_add(1);
                    (*E).var_B = 0;
                    (*E).var_D = (*E).var_D.wrapping_add(1);
                }
            } else {
                words!((*E), var_C, var_B += words!((g_word_A7CD87, g_word_A7CD85)));
                if (((*E).var_C) as i16).wrapping_sub((g_word_A7CD8D) as i16) >= 0 {
                    (*E).var_C = g_word_A7CD8D;
                    set_fields! {[(*E)], 0; var_B, var_D}
                }
            }
        } else {
            words!(
                (*E),
                var_C,
                var_B += -(words!(i32, (g_word_A7CD83, g_word_A7CD81))) as u32
            );
            if (*E).var_C == g_word_A7CD89
                || (((*E).var_C) as i16).wrapping_sub((g_word_A7CD89) as i16) < 0
            {
                (*E).var_C = (g_word_A7CD89).wrapping_add(2);
                (*E).var_B = 0;
                (*E).var_D = (*E).var_D.wrapping_add(1);
            }
        };
    }
}
pub unsafe fn Phantoon_MoveInFigure8_LeftSideClockwise(j: u16, r20: u16, game: &mut Game) {
    unsafe {
        let E = Get_Phantoon(0, game);
        let mut n: i32 = (*E).var_C as i32;
        while n != 0 {
            let v2 = RomPtr_A7((2 * (*E).var_A as i32 + j as i32) as u16, game);
            (*E).base.x_pos = ((*E).base.x_pos).wrapping_add((*v2 as i8) as u16);
            (*E).base.y_pos = ((*E).base.y_pos).wrapping_add((*v2.offset(1) as i8) as u16);
            (*E).var_A = (*E).var_A.wrapping_add(1);
            if (*E).var_A as i32 - r20 as i32 & 0x8000 == 0 {
                (*E).var_A = 0;
            }
            n -= 1;
        }
    }
}
pub unsafe fn Phantoon_MoveInFigure8_RightSideClockwise(j: u16, r20: u16, game: &mut Game) {
    unsafe {
        let E = Get_Phantoon(0, game);
        let mut n: i32 = -((*E).var_C as i32) as u16 as i32;
        while n != 0 {
            let v4 = RomPtr_A7((2 * (*E).var_A as i32 + j as i32) as u16, game);
            (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub((*v4 as i8) as u16);
            (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub((*v4.offset(1) as i8) as u16);
            (*E).var_A = (*E).var_A.wrapping_sub(1);
            if (*E).var_A & 0x8000 != 0 {
                (*E).var_A = r20;
            }
            n -= 1;
        }
    }
}
pub unsafe fn Phantoon_MoveInSwoopingPattern(k: u16, game: &mut Game) {
    unsafe {
        let mut v4: u16 = 0;
        let E2 = Get_Phantoon(0x80, game);
        let mut phant_var_E: i16 = (*E2).var_E as i16;
        if phant_var_E < 0 {
            let mut v6: u16 = ((phant_var_E) as u16).wrapping_sub(2);
            (*E2).var_E = v6;
            v4 = (v6) & (0x7fffu16);
            if v4 == 0 {
                v4 = 0;
                (*E2).var_E = 0;
            }
        } else {
            v4 = ((phant_var_E) as u16).wrapping_add(2);
            (*E2).var_E = v4;
            if v4 as i32 - 256i32 & 0x8000 == 0 {
                let mut v5: u16 = (v4) | (0x8000u16);
                (*E2).var_E = v5;
                v4 = (v5) & (0x7fffu16);
            }
        }
        let E0 = Get_Phantoon(0, game);
        if ((v4) as i16).wrapping_sub(((*E0).base.x_pos) as i16) < 0 {
            if (*E2).var_C as i32 + 2047 & 0x8000 == 0 {
                (*E2).var_C = ((*E2).var_C).wrapping_sub(32);
            }
        } else if (*E2).var_C as i32 - 2048i32 & 0x8000 != 0 {
            (*E2).var_C = ((*E2).var_C).wrapping_add(32);
        }
        words!(
            (*E0).base,
            x_pos,
            x_subpos += (((*E2).var_C as i16 as i32) << 8) as u32
        );
        if (*E0).base.x_pos as i32 + 64 & 0x8000 != 0 {
            (*E0).base.x_pos = -64i32 as u16;
        } else if (*E0).base.x_pos as i32 - 448i32 & 0x8000 == 0 {
            (*E0).base.x_pos = 448;
        }
        let mut v13: i16 = if (*Get_Phantoon(k as i32, game)).var_F as i32 == 0xa7d92e & 0xffff {
            112
        } else {
            ((word!(game.samus_y_pos)) as i16).wrapping_sub(48)
        };
        if (v13).wrapping_sub(((*E0).base.y_pos) as i16) < 0 {
            if (*E2).var_D as i32 + 1535 & 0x8000 == 0 {
                (*E2).var_D = ((*E2).var_D).wrapping_sub(64);
            }
        } else if (*E2).var_D as i32 - 1536i32 & 0x8000 != 0 {
            (*E2).var_D = ((*E2).var_D).wrapping_add(64);
        }
        words!(
            (*E0).base,
            y_pos,
            y_subpos += (((*E2).var_D as i16 as i32) << 8) as u32
        );
        if (*E0).base.y_pos as i32 - 64i32 & 0x8000 != 0 {
            (*E0).base.y_pos = 64;
        } else if (*E0).base.y_pos as i32 - 216i32 & 0x8000 == 0 {
            (*E0).base.y_pos = 216;
        }
    }
}
pub unsafe fn Phantoon_BeginSwoopingPattern(k: u16, game: &mut Game) {
    unsafe {
        let E2 = Get_Phantoon(0x80, game);
        set_fields! {[(*E2)], 1024; var_C, var_D}
        (*E2).var_E = 0;
        let E = Get_Phantoon(k as i32, game);
        (*E).var_F = (0xa7d678i32) as u16;
        (*E).var_E = 360;
    }
}
pub unsafe fn Phantoon_ChangeEyeSpriteBasedOnSamusDist(game: &mut Game) {
    unsafe {
        let v0: u16 = (2u16).wrapping_mul({
            match actor_api::direction_to_player(&mut Runtime::new(game)) {
                actor_api::PlayerDirection::Up => 0u16,
                actor_api::PlayerDirection::UpRight => 1u16,
                actor_api::PlayerDirection::Right => 2u16,
                actor_api::PlayerDirection::DownRight => 3u16,
                actor_api::PlayerDirection::Down => 4u16,
                actor_api::PlayerDirection::DownLeft => 6u16,
                actor_api::PlayerDirection::Left => 7u16,
                actor_api::PlayerDirection::UpLeft => 8u16,
            }
        });
        let E = Get_Phantoon(0x40, game);
        enemy_instruction! {[(*E).base]; timer = 1, instruction = crate::rtl::rom_table_word!(game, 0xa7d40d, (v0 >> 1) as isize)}
    }
}
pub unsafe fn Phantoon_StartDeathSequence(k: u16, game: &mut Game) {
    unsafe {
        let E = Get_Phantoon(k as i32, game);
        if (*E).var_F as i32 == 0xa7d678 & 0xffff || (*E).var_F as i32 == 0xa7d6b9 & 0xffff {
            (*E).var_F = (0xa7d92ei32) as u16;
        } else {
            (*E).var_F = (0xa7d948i32) as u16;
        }
        let E1 = Get_Phantoon(0x40, game);
        set_fields! {[(*E1)], 0; var_C, var_F}
        Phantoon_ChangeEyeSpriteBasedOnSamusDist(game);
        let v4: u16 = word!(game.reg_BG2HOFS);
        for v3 in (0i16..=510).rev().step_by(2) {
            *(ram_ptr!(game, u16; 0x9100, v3 as u16 as i32 as isize)) = v4;
        }
        word!(game.phantom_related_layer_flag |= (0x4000u16));
        (*Get_Phantoon(0xc0i32, game)).parameter_2 = 1;
    }
}
pub unsafe fn Phantoon_FadeOut(a: u16, game: &mut Game) {
    unsafe {
        if (word!(game.nmi_frame_counter_word) & 1) == 0 {
            let E = Get_Phantoon(0x40, game);
            if (*E).var_F == 0 {
                (*E).var_D = a;
                if Phantoon_Func_8(game) & 1 != 0 {
                    (*E).var_F = 1;
                }
            }
        }
    }
}
pub unsafe fn Phantoon_FadeIn(a: u16, game: &mut Game) {
    unsafe {
        if (word!(game.nmi_frame_counter_word) & 1) == 0 {
            let E = Get_Phantoon(0x40, game);
            if (*E).var_F == 0 {
                (*E).var_D = a;
                if Phantoon_SetColorBasedOnHp(game) & 1 != 0 {
                    (*E).var_F = 1;
                }
            }
        }
    }
}
pub unsafe fn Phantoon_Func_8(game: &mut Game) -> u8 {
    unsafe {
        let E = Get_Phantoon(0x40, game);
        if ((*E).var_D).wrapping_add(1) >= (*E).var_E {
            for i in (0i32..32).step_by(2) {
                word!(
                    game.palette_buffer[((i >> 1) + 112)] =
                        Phantoon_Func_10_CalculateNthTransitionColorFromXtoY(
                            (*E).var_E,
                            *(ram_ptr!(game, u16; 0xc000)).offset(((i >> 1) + 112) as isize,),
                            crate::rtl::rom_table_word!(game, 0xa7ca41, (i >> 1) as isize),
                            game,
                        )
                );
            }
            (*E).var_E = (*E).var_E.wrapping_add(1);
            0
        } else {
            (*E).var_E = 0;
            1
        }
    }
}
pub unsafe fn Phantoon_SetColorBasedOnHp(game: &mut Game) -> u8 {
    unsafe {
        let mut Entry: PairU16 = PairU16 { k: 0, j: 0 };
        let E = Get_Phantoon(0x40, game);
        if ((*E).var_D).wrapping_add(1) >= (*E).var_E {
            for v2 in (0u16..32).step_by(2) {
                Entry = Phantoon_SetColorBasedOnHp_FindEntry(v2, game);
                let j: u16 = Entry.j;
                let v5: u16 =
                    *(ram_ptr!(game, u16; 0xc000)).offset(((Entry.k as i32 >> 1) + 112) as isize);
                *(ram_ptr!(game, u16; 0xc000)).offset(((v2 as i32 >> 1) + 112) as isize) =
                    Phantoon_Func_10_CalculateNthTransitionColorFromXtoY((*E).var_E, v5, j, game);
            }
            (*E).var_E = (*E).var_E.wrapping_add(1);
            0
        } else {
            (*E).var_E = 0;
            1
        }
    }
}
pub unsafe fn Phantoon_SetColorBasedOnHp_FindEntry(k: u16, game: &mut Game) -> PairU16 {
    unsafe {
        let r24: u16 = k;
        let r18: u16 = 312;
        let mut r20: u16 = 312;
        let mut r22: u16 = 0;
        while ((r20) as i16).wrapping_sub(
            ((*Get_Phantoon(word!(game.cur_enemy_index) as i32, game))
                .base
                .health) as i16,
        ) < 0
        {
            r20 = (r20).wrapping_add(r18);
            r22 = r22.wrapping_add(1);
            if r22 as i32 - 7i32 & 0x8000 == 0 {
                break;
            }
        }
        let v1 = RomPtr_A7(
            (r24).wrapping_add(crate::rtl::rom_table_word!(game, 0xa7dc4a, r22 as isize)),
            game,
        ) as *const u16;
        MakePairU16(r24, *v1)
    }
}
pub unsafe fn Phantoon_Func_9(game: &mut Game) -> u8 {
    unsafe {
        let E = Get_Phantoon(0x40, game);
        if ((*E).var_D).wrapping_add(1) >= (*E).var_E {
            for i in (0i32..224).step_by(2) {
                word!(
                    game.palette_buffer[(i >> 1)] =
                        Phantoon_Func_10_CalculateNthTransitionColorFromXtoY(
                            (*E).var_E,
                            word!(game.palette_buffer[(i >> 1)]),
                            crate::rtl::rom_table_word!(game, 0xa7ca61, (i >> 1) as isize),
                            game,
                        )
                );
            }
            (*E).var_E = (*E).var_E.wrapping_add(1);
            0
        } else {
            (*E).var_E = 0;
            1
        }
    }
}
pub unsafe fn Phantoon_Func_10_CalculateNthTransitionColorFromXtoY(
    a: u16,
    k: u16,
    j: u16,
    game: &mut Game,
) -> u16 {
    unsafe {
        (Phantoon_CalculateNthTransitionColorComponentFromXtoY(
            a,
            (k) & (0x1fu16),
            (j) & (0x1fu16),
            game,
        ) as i32
            + ((Phantoon_CalculateNthTransitionColorComponentFromXtoY(
                a,
                (k as i32 >> 5 & 0x1f) as u16,
                (j as i32 >> 5 & 0x1f) as u16,
                game,
            ) as i32)
                << 5)
            + ((Phantoon_CalculateNthTransitionColorComponentFromXtoY(
                a,
                (k as i32 >> 10 & 0x1f) as u16,
                (j as i32 >> 10 & 0x1f) as u16,
                game,
            ) as i32)
                << 10)) as u16
    }
}
pub unsafe fn Phantoon_CalculateNthTransitionColorComponentFromXtoY(
    a: u16,
    k: u16,
    j: u16,
    game: &mut Game,
) -> u16 {
    unsafe {
        if a == 0 {
            return k;
        }
        let v4: u16 = (a).wrapping_sub(1);
        let E = Get_Phantoon(0x40, game);
        if v4 == (*E).var_D {
            return j;
        }
        let r20: u16 = (v4).wrapping_add(1);
        let mut r18: u16 = (j).wrapping_sub(k);
        let v6: u8 = abs16(r18) as u8;
        let RegWord: u16 = SnesDivide(
            ((v6 as i32) << 8) as u16,
            (byte!((*E).var_D; 0) as i32 - r20 as i32 + 1) as u8,
        );
        r18 = (if r18 & 0x8000 != 0 {
            -(RegWord as i32)
        } else {
            RegWord as i32
        }) as u16;
        ((r18).wrapping_add(u16::swap_bytes(k))) >> 8
    }
}
pub unsafe fn Etecoon_Func_1(k: u16, game: &mut Game) {
    unsafe {
        if word!(game.earthquake_timer) != 0 {
            let E = Get_Etecoon(k as i32, game);
            (*E).parameter_2 = ((byte!((*E).parameter_2; 0)) as u16) | (0x8000u16);
            (*E).base.instruction_timer = ((*E).base.instruction_timer).wrapping_add(128);
        }
    }
}
pub unsafe fn Etecoon_Func_2(k: u16, game: &mut Game) -> u8 {
    unsafe {
        let E = Get_Etecoon(k as i32, game);
        {
            let (actor, amount) = (k, (words!((*E), var_C, var_D)) as i32);
            (actor_api::move_horizontal(
                &mut Runtime::new(game),
                Key::from_legacy(actor),
                amount,
                actor_api::SlopePolicy::Ignore,
            ) == actor_api::MovementResult::Blocked) as u8
        }
    }
}
pub unsafe fn Etecoon_Func_3(k: u16, game: &mut Game) -> u8 {
    unsafe {
        let E = Get_Etecoon(k as i32, game);
        let amt: i32 = (words!((*E), var_A, var_B)) as i32;
        if (*E).var_A as i32 - 5i32 & 0x8000 != 0 {
            words!(
                (*E),
                var_A,
                var_B += words!(game.samus_y_accel, samus_y_subaccel)
            );
        }
        {
            let (actor, amount) = (k, amt);
            (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount)
                == actor_api::MovementResult::Blocked) as u8
        }
    }
}
static funcs_A2A79: [Func_Y_V; 3] = callback_table! { nonnull Func_Y_V;
    Etecoon_Func_13, Etecoon_Func_14, Etecoon_Func_15,
};
pub unsafe fn Etecoon_Func_13(j: u16, game: &mut Game) {
    unsafe {
        let E = Get_Etecoon(j as i32, game);
        (*E).var_F = (0xa7ec97i32) as u16;
        (*E).base.current_instruction = addr_stru_A7E828 as u16;
        (*E).var_C = g_word_A7E90C;
        (*E).var_D = g_word_A7E90E;
    }
}
pub unsafe fn Etecoon_Func_14(j: u16, game: &mut Game) {
    unsafe {
        let E = Get_Etecoon(j as i32, game);
        (*E).var_F = (0xa7ecbbi32) as u16;
        (*E).base.current_instruction = addr_stru_A7E880 as u16;
        (*E).var_C = g_word_A7E908;
        (*E).var_D = g_word_A7E90A;
    }
}
pub unsafe fn Etecoon_Func_15(j: u16, game: &mut Game) {
    unsafe {
        let E = Get_Etecoon(j as i32, game);
        (*E).var_F = (0xa7ed75i32) as u16;
        (*E).base.current_instruction = ((*E).base.current_instruction).wrapping_add(2);
        (*E).var_C = g_word_A7E908;
        (*E).var_D = g_word_A7E90A;
    }
}
pub unsafe fn Dachora_Func_1(j: u16, k: u16, game: &mut Game) {
    unsafe {
        let E = Get_Dachora(k as i32, game);
        let v3: u16 = u16::swap_bytes((*E).base.palette_index);
        memcpy(
            ram_ptr!(game, c_void; crate::rtl::rom_table_word!(
            game,
            0xa7f55f,
            (v3 >> 1) as isize
        ) as i32 as isize),
            RomPtr_A7(j, game) as *const c_void,
            32 as size_t,
        );
    }
}
pub unsafe fn Dachora_Func_6(k: u16, game: &mut Game) -> i32 {
    unsafe {
        let E = Get_Dachora(k as i32, game);
        if (((*E).var_A) as i16).wrapping_sub((g_word_A7F4D5) as i16) >= 0 {
            if (*E).var_E == 1 {
                QueueSfx2_Max6(0x39, game);
            }
            let v2: u16 = ((*E).var_E).wrapping_sub(1);
            (*E).var_E = v2;
            if v2 as u8 == 0 {
                Dachora_Func_1(
                    crate::rtl::rom_table_word!(
                        game,
                        0xa7f787,
                        *(&raw mut (*E).var_E as *mut u8).offset(1) as isize
                    ),
                    k,
                    game,
                );
                let v3: u16 = ((*E).var_E).wrapping_add(272);
                (*E).var_E = v3;
                if v3 as i32 - 1040i32 & 0x8000 == 0 {
                    (*E).var_E = 784;
                }
            }
        }
        {
            let (actor, amount) = (k, (1i16 as i32) << 16);
            (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount)
                == actor_api::MovementResult::Blocked) as u8
        };
        if (((*E).var_A) as i16).wrapping_sub((g_word_A7F4D5) as i16) >= 0
            && (((*E).var_B) as i16).wrapping_sub((g_word_A7F4D7) as i16) >= 0
        {
            (*E).var_A = g_word_A7F4D5;
            (*E).var_B = g_word_A7F4D7;
            return (words!((g_word_A7F4D5, g_word_A7F4D7))) as i32;
        }
        words!((*E), var_A, var_B += words!((g_word_A7F4D9, g_word_A7F4DB)));
        if ((*E).var_A == 4 || (*E).var_A == 8) && (*E).var_B == 0 {
            (*E).base.current_instruction = ((*E).base.current_instruction).wrapping_add(28);
        }
        (words!((*E), var_A, var_B)) as i32
    }
}
pub unsafe fn Dachora_Func_9(k: u16, game: &mut Game) {
    unsafe {
        let EK = Get_Dachora(k as i32, game);
        let E1 = Get_Dachora(k as i32 + 64, game);
        let dachor_var_A: u16 = (*E1).var_A;
        if dachor_var_A != 0 {
            (*E1).var_A = (dachor_var_A).wrapping_sub(1);
        } else {
            (*E1).var_A = g_word_A7F4D1;
            if (*E1).var_D != 0 {
                let E2 = Get_Dachora(k as i32 + 128, game);
                if (*E2).var_D != 0 {
                    let E3 = Get_Dachora(k as i32 + 192, game);
                    if (*E3).var_D != 0 {
                        let E4 = Get_Dachora(k as i32 + 256, game);
                        if (*E4).var_D == 0 {
                            (*E4).base.x_pos = (*EK).base.x_pos;
                            (*E4).base.y_pos = (*EK).base.y_pos;
                            (*E4).var_D = g_word_A7F4D3;
                        }
                    } else {
                        (*E3).base.x_pos = (*EK).base.x_pos;
                        (*E3).base.y_pos = (*EK).base.y_pos;
                        (*E3).var_D = g_word_A7F4D3;
                    }
                } else {
                    (*E2).base.x_pos = (*EK).base.x_pos;
                    (*E2).base.y_pos = (*EK).base.y_pos;
                    (*E2).var_D = g_word_A7F4D3;
                }
            } else {
                (*E1).base.x_pos = (*EK).base.x_pos;
                (*E1).base.y_pos = (*EK).base.y_pos;
                (*E1).var_D = g_word_A7F4D3;
            }
        };
    }
}
pub unsafe fn Dachora_Func_10(k: u16, game: &mut Game) {
    unsafe {
        let E = Get_Dachora(k as i32, game);
        Dachora_Func_1(
            crate::rtl::rom_table_word!(
                game,
                0xa7f92d,
                *(&raw mut (*E).var_E as *mut u8).offset(1) as isize
            ),
            k,
            game,
        );
        let v2: u16 = ((*E).var_E).wrapping_add(256);
        (*E).var_E = v2;
        if v2 as i32 - 1024i32 & 0x8000 == 0 {
            (*E).var_E = 0;
        }
    }
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10982544] KraidsArm_Touch [] {
        if word!(game.samus_invincibility_timer) == 0 {
            Kraid_Func_1(game);
            (*gEnemyData(0i32, game).offset(4)).ai_var_A = (0xa7b89bi32) as u16;
            Kraid_Touch(game);
        }
    };
    [10987865] Kraid_Init [] {
        (*(ram_ptr!(game, LongPtr; 0x604))).bank = -89i32 as u8;
        (*(ram_ptr!(game, LongPtr; 0x601))).bank = -89i32 as u8;
        (*(ram_ptr!(game, LongPtr; 0x604))).addr = (0xa7c24ei32) as u16;
        (*(ram_ptr!(game, LongPtr; 0x601))).addr = (0xa7c325i32) as u16;
        if Kraid_CheckIfDead(game) != 0 {
            word_transfers!(game, ascending, 16; le(target_palettes, 96) <- rom(0xa786c7, 0));
            word_transfers!(game, descending, 1024; le(tilemap_stuff, 1023) <- value(824));
            let E = Get_Kraid(0, game);
            (*E).field_4 = 0;
            let v4: u16 = word!(game.vram_write_queue_tail);
            let mut v5 = gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game);
            (*v5).size = 512;
            (*v5).src.addr = addr_kKraidsRoomBg as i32 as VoidP;
            (*v5).src.bank = -89i32 as u8;
            (*v5).vram_dst = (((*ram_ptr!(game; 0x5d) as i32 & 0xf) << 8) + 16128) as u16;
            word!(game.vram_write_queue_tail = (v4).wrapping_add(7));
            Kraid_SpawnPlmToClearCeiling(game);
            Kraid_ClearSomeSpikes(game);
            (*E).kraid_var_A = (0xa7c715i32) as u16;
        } else {
            *ram_ptr!(game; 0x59) = 67;
            word!(game.camera_distance_index = 2);
            *(ram_ptr!(game, u16; 0xcd20)) = 0;
            *(ram_ptr!(game, u16; 0xcd20, 2)) = 1;
            let E_0 = Get_Kraid(0, game);
            (*E_0).kraid_min_y_pos_eject = 324;
            let mut v8: u16 = (*E_0).base.health >> 3;
            let r18: u16 = v8;
            for v7 in (0u16..16).step_by(2) {
                (*Get_Kraid(v7 as i32, game)).kraid_healths_8ths[0i32 as usize] = v8;
                v8 = (v8).wrapping_add(r18);
            }
            Kraid_SetupGfxWithTilePrioClear(0xdfff, game);
            let v10: u16 = (*E_0).base.health >> 2;
            (*E_0).kraid_healths_4ths[0i32 as usize] = v10;
            let v11: u16 = ((*E_0).kraid_healths_4ths[0i32 as usize]).wrapping_add(v10);
            (*E_0).kraid_healths_4ths[1i32 as usize] = v11;
            let v12: u16 = ((*E_0).kraid_healths_4ths[0i32 as usize]).wrapping_add(v11);
            (*E_0).kraid_healths_4ths[2i32 as usize] = v12;
            (*E_0).kraid_healths_4ths[3i32 as usize] =
                ((*E_0).kraid_healths_4ths[0i32 as usize]).wrapping_add(v12);
            (*E_0).field_2E[8i32 as usize] = 0;
            set_fields! {[(*E_0).base]; x_pos = 176, y_pos = 592,
            properties = ((*E_0).base.properties) | ((kEnemyProps_Tangible) as u16), };
            set_fields! {[(*E_0)]; kraid_var_A = (0xa7c865i32) as u16, kraid_var_F = 300,
            kraid_next = (0xa7c86bi32) as u16, kraid_var_C = 64, };
            minimap::disable_for_boss(&mut crate::game::session_runtime::Runtime::new(game));
            word_transfers!(game, descending, 32; native(0x2ffe) <- value(824));
            word!(game.earthquake_type = 5);
            word_transfers!(game, ascending, 16; native(0xc360) <- rom(0xa7aaa6, 0));
        };
    };
    [10988355] KraidsArm_Init [] {
        if Kraid_CheckIfDead(game) != 0 {
            Kraid_SetEnemyPropsToDead(game);
        } else {
            let v0 = gEnemyData(0, game);
            (*v0.offset(1)).palette_index = (*v0).palette_index;
            (*v0.offset(1)).ai_var_A = (0xa7ba2di32) as u16;
            enemy_instruction! {[(*v0.offset(1))]; instruction = addr_kKraid_Ilist_8AA4 as u16, timer = 1}
            (*v0.offset(1)).ai_var_B = 0;
        };
    };
    [10988392] KraidsTopLint_Init [] {
        if Kraid_CheckIfDead(game) != 0 {
            Kraid_SetEnemyPropsToDead(game);
        } else {
            let v0 = gEnemyData(0, game);
            (*v0.offset(2)).palette_index = (*v0).palette_index;
            enemy_instruction! {[(*v0.offset(2))]; timer = 0x7fff, instruction = addr_kKraid_Ilist_8AFE as u16}
            (*v0.offset(2)).spritemap_pointer = addr_kKraid_Sprmap_A5DF as u16;
            set_fields! {[(*v0.offset(2))]; ai_var_A = addr_locret_A7B831 as u16,
            ai_preinstr = 0x7fff, ai_var_C = 0, };
        };
    };
    [10988444] KraidsMiddleLint_Init [] {
        if Kraid_CheckIfDead(game) != 0 {
            Kraid_SetEnemyPropsToDead(game);
        } else {
            let v0 = gEnemyData(0, game);
            (*v0.offset(3)).palette_index = (*v0).palette_index;
            enemy_instruction! {[(*v0.offset(3))]; timer = 0x7fff, instruction = addr_kKraid_Ilist_8AFE as u16}
            (*v0.offset(3)).spritemap_pointer = addr_kKraid_Sprmap_A5DF as u16;
            (*v0.offset(3)).ai_var_A = addr_locret_A7B831 as u16;
            (*v0.offset(3)).ai_var_C = -16i32 as u16;
        };
    };
    [10988490] KraidsBottomLint_Init [] {
        if Kraid_CheckIfDead(game) != 0 {
            Kraid_SetEnemyPropsToDead(game);
        } else {
            let v0 = gEnemyData(0, game);
            (*v0.offset(4)).palette_index = (*v0).palette_index;
            enemy_instruction! {[(*v0.offset(4))]; timer = 0x7fff, instruction = addr_kKraid_Ilist_8AFE as u16}
            (*v0.offset(4)).spritemap_pointer = addr_kKraid_Sprmap_A5DF as u16;
            (*v0.offset(4)).ai_var_A = addr_locret_A7B831 as u16;
            (*v0.offset(4)).ai_var_C = -16i32 as u16;
        };
    };
    [10988536] KraidsFoot_Init [] {
        if Kraid_CheckIfDead(game) != 0 {
            Kraid_SetEnemyPropsToDead(game);
        } else {
            let palette_index: u16 = (*Get_Kraid(0i32, game)).base.palette_index;
            let E = Get_Kraid(0x140, game);
            (*E).base.palette_index = palette_index;
            enemy_instruction! {[(*E).base]; instruction = addr_kKraid_Ilist_86E7 as u16, timer = 1}
            (*E).kraid_var_A = (0xa7ba2di32) as u16;
            (*E).kraid_next = 0;
        };
    };
    [10988577] Kraid_Main [] {
        Kraid_Shot_Mouth(game);
        Kraid_Palette_Handling(game);
        Kraid_Shot_Body(game);
        Kraid_Enemy_Touch(game);
        let E = Get_Kraid(0, game);
        word! { game;
            reg_BG2HOFS = ((*E).base.x_width as i32 + word!(game.reg_BG1HOFS) as i32
                - word!(game.bg1_x_offset) as i32
                - (*E).base.x_pos as i32) as u16;
            reg_BG2VOFS = (word!(game.layer1_y_pos) as i32 - (*E).base.y_pos as i32 + 152) as u16;
        }
        CallKraidFunc((((*E).kraid_var_A) as u32) | (0xa70000u32), game);
    };
    [10991549] KraidsArm_Main [] {
        let r18: u16 = (word!(game.layer1_y_pos)).wrapping_add(224);
        let E = Get_Kraid(0, game);
        let v1: u16 = ((*E).base.y_pos).wrapping_sub(44);
        let E1 = Get_Kraid(0x40, game);
        (*E1).base.y_pos = v1;
        let v3: u16 = v1;
        let mut v4: u16 = ((*E1).base.properties) | (0x100u16);
        if ((v3) as i16).wrapping_sub((word!(game.layer1_y_pos)) as i16) >= 0
            && ((v3) as i16).wrapping_sub((r18) as i16) < 0
        {
            v4 = ((*E1).base.properties) & (0xfeffu16);
        }
        (*E1).base.properties = v4;
        (*E1).base.x_pos = (*E).base.x_pos;
        if byte!((*E).kraid_mouth_flags; 1) != 0 {
            (*E1).base.instruction_timer = (*E1).base.instruction_timer.wrapping_add(1);
        }
    };
    [10991617] KraidsTopLint_Main [] {
        (*gEnemyData(0x80i32, game)).instruction_timer = 0x7fff;
        KraidLintCommon_Main(0x80, game);
    };
    [10991629] KraidsMiddleLint_Main [] {
        (*gEnemyData(0xc0i32, game)).instruction_timer = 0x7fff;
        KraidLintCommon_Main(0xc0, game);
    };
    [10991641] KraidsBottomLint_Main [] {
        (*gEnemyData(0x100i32, game)).instruction_timer = 0x7fff;
        KraidLintCommon_Main(0x100, game);
    };
    [10992118] KraidsFoot_Main [] {
        let v0 = gEnemyData(0, game);
        (*v0.offset(5)).x_pos = (*v0).x_pos;
        let v1: u16 = ((*v0).y_pos).wrapping_add(100);
        (*v0.offset(5)).y_pos = v1;
        let v2: u16 = v1;
        let mut v3: i16 = ((v1) as i16).wrapping_sub(224);
        let mut v4: u16 = ((*v0.offset(5)).properties) & (0xfeffu16);
        if ((v2) as i16).wrapping_sub((word!(game.layer1_y_pos)) as i16) >= 0 {
            if (v3).wrapping_sub((word!(game.layer1_y_pos)) as i16) >= 0 {
                v4 = (v4) | (0x100u16);
            }
        } else {
            v4 = (v4) | (0x100u16);
        }
        (*v0.offset(5)).properties = v4;
        CallKraidFunc((((*v0.offset(5)).ai_var_A) as u32) | (0xa70000u32), game);
    };
    [10992847 | 10992862] KraidsGoodFingernail_Touch [] {
        let v0: u16 = 0;
        printf(b"X undefined\n\0" as *const u8 as *const c_char);
        actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
        ({ let (_ignored_key, animation) = (word!(game.cur_enemy_index), v0); actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Normal { animation }) });
    };
    [10992946] KraidsGoodFingernail_Main [] {
        let v0 = gEnemyData(0, game);
        if (*v0).health as i32 - 1i32 & 0x8000 != 0 {
            let v1 = gEnemyData(0, game);
            let fresh12 = &raw mut (*v1.offset(6)).properties;
            fresh12.write_unaligned(
                (fresh12.read_unaligned() as i32
                    | (kEnemyProps_Deleted as i32 | kEnemyProps_Invisible as i32))
                    as u16,
            );
        } else {
            CallKraidFunc((((*v0.offset(6)).ai_var_A) as u32) | (0xa70000u32), game);
        };
    };
    [10992969] KraidsBadFingernail_Main [] {
        let v0 = gEnemyData(0, game);
        if (*v0).health as i32 - 1i32 & 0x8000 != 0 {
            let v1 = gEnemyData(0, game);
            let fresh4 = &raw mut (*v1.offset(7)).properties;
            fresh4.write_unaligned(
                (fresh4.read_unaligned() as i32
                    | (kEnemyProps_Deleted as i32 | kEnemyProps_Invisible as i32))
                    as u16,
            );
        } else {
            CallKraidFunc((((*v0.offset(7)).ai_var_A) as u32) | (0xa70000u32), game);
        };
    };
    [10997235] Phantoon_Init [] {
        word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(824));
        word_transfers!(game, descending, 1024; native(0x97fe) <- value(0));
        word_transfers!(game, descending, 16; native(0xc2fe) <- value(0));
        word!(game.enemy_bg2_tilemap_size = 864);
        minimap::disable_for_boss(&mut crate::game::session_runtime::Runtime::new(game));
        let E = Get_Phantoon(word!(game.cur_enemy_index) as i32, game);
        (*E).var_E = 120;
        set_fields! {[(*E)], 0; var_A, var_B}
        word!(game.g_word_7E9032 = 0);
        let E1 = Get_Phantoon(0x40, game);
        set_fields! {[(*E1)], 0; parameter_1, parameter_2}
        let E0 = Get_Phantoon(0, game);
        (*E0).base.properties = ((*E0).base.properties) | ((kEnemyProps_Tangible) as u16);
        static unk_A7CE51: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x1, 0x14, 0xce96 as VoidP);
        spawn_hdma(0xa7, &unk_A7CE51, game);
        Phantoon2_Init(game);
    };
    [10997333] Phantoon2_Init [] {
        let v6 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
        (*v6).spritemap_pointer = addr_kSpritemap_Nothing_A7 as u16;
        (*v6).instruction_timer = 1;
        (*v6).timer = 0;
        let v7 = gEnemyData(0, game);
        (*v6).palette_index = (*v7).palette_index;
        (*v6).vram_tiles_index = (*v7).vram_tiles_index;
        (*v6).current_instruction =
            crate::rtl::rom_table_word!(game, 0xa7ce8e, (*v6).parameter_2 as isize);
        (*v6).ai_preinstr = (0xa7d4a9i32) as u16;
        (*v7.offset(3)).parameter_1 = 0;
        (*v7.offset(3)).ai_var_C = -1i32 as u16;
    };
    [10997414] Phantoon_Main [] {
        Phantoon_Func_2(word!(game.cur_enemy_index), game);
        let EK = Get_Phantoon(word!(game.cur_enemy_index) as i32, game);
        actor_species_pre_instruction((((*EK).var_F) as u32) | (0xa70000u32), game);
        if word!(game.cur_enemy_index) == 0 {
            let E0 = Get_Phantoon(0, game);
            let E1 = Get_Phantoon(0x40, game);
            let E2 = Get_Phantoon(0x80, game);
            let E3 = Get_Phantoon(0xc0, game);
            let x_pos: u16 = (*E0).base.x_pos;
            (*E1).base.x_pos = x_pos;
            (*E2).base.x_pos = x_pos;
            (*E3).base.x_pos = x_pos;
            let y_pos: u16 = (*E0).base.y_pos;
            (*E1).base.y_pos = y_pos;
            (*E2).base.y_pos = y_pos;
            (*E3).base.y_pos = y_pos;
            if (*E1).parameter_1 == 0 {
                word! { game;
                    reg_BG2HOFS =
                        (word!(game.layer1_x_pos) as i32 - (*E0).base.x_pos as i32 + 40) as u16;
                    reg_BG2VOFS =
                        (word!(game.layer1_y_pos) as i32 - (*E0).base.y_pos as i32 + 40) as u16;
                }
            }
        }
    };
    [11001151] Phantoon_Hurt [] {
        let mut Entry: PairU16 = PairU16 { k: 0, j: 0 };
        let E0 = Get_Phantoon(0, game);
        if (*E0).base.flash_timer != 8 {
            if ((*E0).base.frame_counter & 2) == 0 {
                if byte!((*(Get_Phantoon as unsafe fn(i32, &mut Game) -> *mut Enemy_Phantoon)(
                0x80i32, game,
            ))
            .parameter_2; 1)
                    == 0
                {
                    return;
                }
            } else {
                if byte!((*(Get_Phantoon as unsafe fn(i32, &mut Game) -> *mut Enemy_Phantoon)(
                0x80i32, game,
            ))
            .parameter_2; 1)
                    == 0
                {
                    word_transfers!(game, descending, 16; native(0xc0fe) <- value(0x7fff));
                    let E3_0 = Get_Phantoon(0x80, game);
                    (*E3_0).parameter_2 = ((*E3_0).parameter_2) | (0x100u16);
                }
                return;
            }
        }
        let mut v1: u16 = 30;
        do_until! { ((Entry.k) as i16).wrapping_sub(2) < 0 ;
            Entry = Phantoon_SetColorBasedOnHp_FindEntry(v1, game);
            *(ram_ptr!(game, u16; 0xc000)).offset(((Entry.k as i32 >> 1) + 112) as isize) = Entry.j;
            v1 = (Entry.k).wrapping_sub(2);
        }
        let E3 = Get_Phantoon(0x80, game);
        (*E3).parameter_2 = byte!((*E3).parameter_2; 0) as u16;
    };
    [11001237] Phantoon_Touch [] {
        actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
    };
    [11001243] Phantoon_Shot [] {
        let E0 = Get_Phantoon(0, game);
        if (*E0).var_F as i32 + 0x26b8 & 0x8000 == 0 {
            return;
        }
        let v1: u16 = word!(game.cur_enemy_index);
        let EK = Get_Phantoon(word!(game.cur_enemy_index) as i32, game);
        let mut health: u16 = (*EK).base.health;
        actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::ReactionOnly);
        if (*EK).base.health == 0 {
            QueueSfx2_Max6(0x73, game);
            (*Get_Phantoon(0x80i32, game)).parameter_2 = 1;
            (*E0).base.properties = ((*E0).base.properties) | ((kEnemyProps_Tangible) as u16);
            Phantoon_StartDeathSequence(v1, game);
            return;
        }
        if ((*EK).base.ai_handler_bits & 2) != 0 {
            QueueSfx2_Max6(0x73, game);
            let phanto_var_F: u16 = (*EK).var_F;
            'react_to_hit: {
                let initial_hit = phanto_var_F as i32 == (0xa7d60d & 0xffff)
                    || phanto_var_F as i32 == (0xa7d788 & 0xffff);
                if !initial_hit && phanto_var_F as i32 != (0xa7d678 & 0xffff) {
                    break 'react_to_hit;
                }
                health = health.wrapping_sub((*EK).base.health);
                if health as i32 - 300i32 & 0x8000 != 0
                    || (word!(game.projectile_type[word!(game.collision_detection_index)]) & 0xf00)
                        != 512
                {
                    let E2 = Get_Phantoon(v1 as i32 + 0x80, game);
                    let damage = health.wrapping_add((*E2).var_B);
                    (*E2).var_B = damage;
                    if !initial_hit {
                        if damage as i32 - 300i32 & 0x8000 == 0 {
                            (*EK).var_E = 1;
                        }
                        break 'react_to_hit;
                    }
                    if damage as i32 - 300i32 & 0x8000 == 0 {
                        (*EK).var_F = (0xa7d6b9i32) as u16;
                    } else {
                        let v10: u16 = (game.next_random()) & (7u16);
                        (*Get_Phantoon(0x40i32, game)).var_B =
                            *(&raw const g_byte_A7CDA5 as *const u8).offset(v10 as i32 as isize)
                                as u16;
                        (*Get_Phantoon(0xc0i32, game)).parameter_2 = v10;
                        let E2x = Get_Phantoon(0x80, game);
                        (*E2x).parameter_2 = 1;
                        if (*E2x).var_A == 0 {
                            (*E2x).var_A = 1;
                            if (*EK).var_E as i32 - 16i32 & 0x8000 == 0 {
                                (*EK).var_E = 16;
                            }
                        }
                        return;
                    }
                } else {
                    (*EK).var_F = (0xa7d85ci32) as u16;
                }
                (*EK).var_E = 0;
                (*Get_Phantoon(0x80i32, game)).var_A = 0;
                (*Get_Phantoon(v1 as i32 + 0x80i32, game)).var_B = 0;
                word!(game.phantom_related_layer_flag |= (0x4000u16));
                (*E0).base.instruction_timer = 1;
                let E1 = Get_Phantoon(0x40, game);
                (*E1).base.instruction_timer = 1;
                (*E0).base.current_instruction = addr_kKraid_Ilist_CC41 as u16;
                (*E1).base.current_instruction = addr_kKraid_Ilist_CC91 as u16;
                (*E0).base.properties = ((*E0).base.properties) | ((kEnemyProps_Tangible) as u16);
                (*E1).var_F = 0;
            }
            (*Get_Phantoon(0x80i32, game)).parameter_2 = 2;
        }
    };
    [11004178] Etecoon_Init [] {
        let E = Get_Etecoon(word!(game.cur_enemy_index) as i32, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
        (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A7 as u16;
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        (*E).base.current_instruction = addr_kEtecoon_Ilist_E8CE as u16;
        (*E).var_F = (0xa7e9afi32) as u16;
        (*E).var_E = -1i32 as u16;
    };
    [11004224] Etecoon_Main [] {
        let E = Get_Etecoon(word!(game.cur_enemy_index) as i32, game);
        if byte!((*E).parameter_2; 1) != 0 {
            (*E).parameter_2 = ((*E).parameter_2).wrapping_sub(256);
        } else {
            actor_species_pre_instruction((((*E).var_F) as u32) | (0xa70000u32), game);
        };
    };
    [11007197] Dachora_Init [] {
        let E = Get_Dachora(word!(game.cur_enemy_index) as i32, game);
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_DisableSamusColl) as u16);
        (*E).base.spritemap_pointer = addr_kSpritemap_Nothing_A7 as u16;
        (*E).base.instruction_timer = 1;
        (*E).base.timer = 0;
        let mut dachor_parameter_1: i16 = (*E).parameter_1 as i16;
        if dachor_parameter_1 < 0 {
            if (dachor_parameter_1 as u16 & 1) != 0 {
                (*E).base.current_instruction = addr_kDachora_Ilist_F4B9 as u16;
            } else {
                (*E).base.current_instruction = addr_kDachora_Ilist_F3F7 as u16;
            }
            (*E).var_F = (0xa7f98ci32) as u16;
        } else {
            if dachor_parameter_1 != 0 {
                (*E).base.current_instruction = addr_kDachora_Ilist_F45B as u16;
            } else {
                (*E).base.current_instruction = addr_kDachora_Ilist_F399 as u16;
            }
            (*E).var_F = (0xa7f570i32) as u16;
        };
    };
    [11007278] Dachora_Main [] {
        let E = Get_Dachora(word!(game.cur_enemy_index) as i32, game);
        actor_species_pre_instruction((((*E).var_F) as u32) | (0xa70000u32), game);
    };
    @forward [pub unsafe fn (game: &mut Game)];
    [10977295] Enemy_GrappleReact_CancelBeam_A7 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [10977325] Enemy_NormalShotAI_A7 => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
    [10977340]
    Enemy_NormalPowerBombAI_SkipDeathAnim_A7 => actors::combat::power_bomb_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
    [10977345] Enemy_NormalFrozenAI_A7 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
    @forward [pub unsafe fn (game: &mut Game)];
    [10982559] Kraid_Touch => actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
    Kraid_Shot => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
    @forward [pub unsafe fn (game: &mut Game)];
    [10992879] KraidsGoodFingernail_Init => KraidFingernailInit(word!(game.cur_enemy_index), game);
    [10992941] KraidsBadFingernail_Init => KraidFingernailInit(word!(game.cur_enemy_index), game);
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10977418] EnemyInstr_Call_A7 [] {
        CallEnemyInstrExtFunc(((jp.word(0)) as u32) | (0xa70000u32), k, game);
        jp.offset(1)
    };
    [10979983] Kraid_Instr_9 [] {
        let E = Get_Kraid(0, game);
        if (((*E).base.health) as i16).wrapping_sub(((*E).kraid_healths_8ths[3i32 as usize]) as i16)
            < 0
        {
            let E1 = Get_Kraid(0x40, game);
            if (*E1).base.current_instruction as i32 + 0x75bf & 0x8000 != 0 {
                return EnemyScript::jump(addr_kKraid_Ilist_8A41 as u16);
            }
        }
        jp
    };
    @functions [pub unsafe fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10991158] Kraid_Instr_DecYpos [jp] {
        let E = Get_Kraid(0, game);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_sub(1);
    };
    [10991164] Kraid_Instr_IncrYpos_Shake [jp] {
        let E = Get_Kraid(0, game);
        (*E).base.y_pos = (*E).base.y_pos.wrapping_add(1);
        word! {game; earthquake_type = 1; earthquake_timer = 10; }
    };
    [10991182] Kraid_Instr_PlaySound_0x76 [jp] { QueueSfx2_Max6(0x76, game); };
    [10991194] Kraid_Instr_XposMinus3 [jp] {
        let E = Get_Kraid(0, game);
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(g_word_A7A91C);
    };
    [10991207] Kraid_Instr_XposMinus3b [jp] {
        let E = Get_Kraid(0, game);
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_sub(g_word_A7A91C);
    };
    [10991220] Kraid_Instr_XposPlus3 [jp] {
        let E = Get_Kraid(0, game);
        (*E).base.x_pos = ((*E).base.x_pos).wrapping_add(g_word_A7A920);
    };
    [10991235] Kraid_Instr_MoveHimRight [jp] {
        let E = Get_Kraid(0, game);
        if ((*E).base.x_pos as i32 - 320i32 & 0x8000 != 0 || {
            let mut v3: u16 = ((*E).kraid_target_x).wrapping_sub(1);
            (*E).kraid_target_x = v3;
            (*E).kraid_target_x == 0
        }) && ({ let (actor, amount) = (0, (g_word_A7A922 as i16 as i32) << 16); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) != 0
        {
            word! {game; earthquake_type = 0; earthquake_timer = 7; }
            let x_pos: u16 = (*Get_Kraid(0, game)).base.x_pos;
            (*Get_Kraid(0x140, game)).base.x_pos = x_pos;
        }
    };
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [10999114] Phantoon_WavyFadeIn [] {
        Phantoon_FadeIn(0xc, game);
        if Phantoon_Func_3((words!((g_word_A7CD9D, g_word_A7CD9B))) as i32, game) != 0 {
            let E = Get_Phantoon(k as i32, game);
            (*E).var_F = (0xa7d596i32) as u16;
            (*Get_Phantoon(0xc0i32, game)).parameter_1 = 1;
            (*E).var_E = 30;
        } else {
            let E_0 = Get_Phantoon(k as i32, game);
            let v2: bool = (*E_0).var_E == 1;
            (*E_0).var_E = (*E_0).var_E.wrapping_sub(1);
            let v3: bool = ((*E_0).var_E & 0x8000) != 0;
            if v2 || v3 {
                (*Get_Phantoon(0xc0i32, game)).var_E = 1;
            }
        };
    };
    [10999271] Phantoon_MovePhantoonInFigure8ThenOpenEye [] {
        Phantoon_AdjustSpeedAndMoveInFigure8(game);
        Phantoon_Func_7(k, game);
        let E = Get_Phantoon(0x40, game);
        let v2: bool = (*E).var_A == 1;
        (*E).var_A = (*E).var_A.wrapping_sub(1);
        let v3: bool = ((*E).var_A & 0x8000) != 0;
        if v2 || v3 {
            (*Get_Phantoon(k as i32, game)).var_F = (0xa7d4a8i32) as u16;
            enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kKraid_Ilist_CC53 as u16}
            (*Get_Phantoon(0i32, game)).parameter_2 = 0;
            Phantoon_Func_5(k, game);
        }
    };
    [10999309] Phantoon_EyeFollowsSamusUntilTimerRunsOut [] {
        let EK = Get_Phantoon(k as i32, game);
        let v2: bool = (*EK).var_E == 1;
        (*EK).var_E = (*EK).var_E.wrapping_sub(1);
        let v3: bool = ((*EK).var_E & 0x8000) != 0;
        if v2 || v3 {
            (*Get_Phantoon(k as i32 + 128i32, game)).var_B = 0;
            let E2 = Get_Phantoon(0x80, game);
            if (*E2).var_A == 0 {
                (*EK).var_F = (0xa7d4a8i32) as u16;
                let E0 = Get_Phantoon(0, game);
                (*E0).base.instruction_timer = 1;
                let E1 = Get_Phantoon(0x40, game);
                (*E1).base.instruction_timer = 1;
                (*E0).base.current_instruction = addr_kKraid_Ilist_CC41 as u16;
                (*E1).base.current_instruction = addr_kKraid_Ilist_CC81 as u16;
                (*E0).base.properties = ((*E0).base.properties) | ((kEnemyProps_Tangible) as u16);
                (*E0).parameter_2 = 1;
                return;
            }
            (*E2).var_A = 0;
            (*EK).var_E = 60;
            (*EK).var_F = (0xa7d65ci32) as u16;
        }
        Phantoon_ChangeEyeSpriteBasedOnSamusDist(game);
    };
    [10999416] Phantoon_IsSwooping [] {
        Phantoon_ChangeEyeSpriteBasedOnSamusDist(game);
        Phantoon_MoveInSwoopingPattern(k, game);
        let EK = Get_Phantoon(k as i32, game);
        let v2: bool = (*EK).var_E == 1;
        (*EK).var_E = (*EK).var_E.wrapping_sub(1);
        let v3: bool = ((*EK).var_E & 0x8000) != 0;
        if v2 || v3 {
            (*EK).var_F = (0xa7d6b9i32) as u16;
            word!(game.phantom_related_layer_flag |= (0x4000u16));
            let E0 = Get_Phantoon(0, game);
            (*E0).base.instruction_timer = 1;
            let E1 = Get_Phantoon(0x40, game);
            (*E1).base.instruction_timer = 1;
            (*E0).base.current_instruction = addr_kKraid_Ilist_CC41 as u16;
            (*E1).base.current_instruction = addr_kKraid_Ilist_CC91 as u16;
            (*E0).base.properties = ((*E0).base.properties) | ((kEnemyProps_Tangible) as u16);
            (*E1).var_F = 0;
            (*Get_Phantoon(k as i32 + 128i32, game)).var_B = 0;
        }
    };
    [10999481] Phantoon_FadeoutWithSwoop [] {
        Phantoon_MoveInSwoopingPattern(k, game);
        Phantoon_FadeOut(0xc, game);
        if (*Get_Phantoon(0x40i32, game)).var_F != 0 {
            let E = Get_Phantoon(k as i32, game);
            (*E).var_F = (0xa7d6d4i32) as u16;
            (*E).var_E = 120;
        }
    };
    [10999522] Phantoon_MoveLeftOrRightAndPickEyeOpenPatt [] {
        let E = Get_Phantoon(0, game);
        if (game.next_random() & 1) != 0 {
            (*E).var_A = 136;
            (*E).base.x_pos = 208;
        } else {
            (*E).var_A = 399;
            (*E).base.x_pos = 48;
        }
        (*E).base.y_pos = 96;
        let E1 = Get_Phantoon(0x40, game);
        (*E1).var_C = 0;
        (*E).var_C = 1;
        set_fields! {[(*E)], 0; var_B, parameter_2}
        Phantoon_PickPatternForRound2(game);
        (*Get_Phantoon(k as i32, game)).var_F = (0xa7d72di32) as u16;
        (*E1).var_F = 0;
    };
    [10999597] Phantoon_FadeInBeforeFigure8 [] {
        Phantoon_FadeIn(0xc, game);
        if (*Get_Phantoon(0x40i32, game)).var_F != 0 {
            (*Get_Phantoon(0i32, game)).var_F = (0xa7d5e7i32) as u16;
        }
    };
    [10999615] Phantoon_BecomeSolidAfterRainingFireballs [] {
        let E1 = Get_Phantoon(0x40, game);
        (*E1).var_F = 0;
        word!(game.phantom_related_layer_flag &= !(0x4000));
        let E = Get_Phantoon(0, game);
        (*E).base.instruction_timer = 1;
        (*E1).base.instruction_timer = 1;
        (*E).base.current_instruction = addr_kKraid_Ilist_CC47 as u16;
        (*E1).base.current_instruction = addr_kKraid_Ilist_CC9D as u16;
        (*Get_Phantoon(k as i32, game)).var_F = (0xa7d767i32) as u16;
    };
    [10999655] Phantoon_FadeInDuringFireballRain [] {
        Phantoon_FadeIn(1, game);
        if (*Get_Phantoon(0x40i32, game)).var_F != 0 {
            let E = Get_Phantoon(0, game);
            (*E).base.properties =
                ((*E).base.properties as i32 & !(kEnemyProps_Tangible as i32)) as u16;
            (*Get_Phantoon(k as i32, game)).var_F = (0xa7d788i32) as u16;
            (*E).var_E = 90;
        }
    };
    [10999765] Phantoon_FadeOutDuringFireballRain [] {
        Phantoon_FadeOut(0xc, game);
        if (*Get_Phantoon(0x40i32, game)).var_F != 0 {
            let E = Get_Phantoon(k as i32, game);
            (*E).var_F = (0xa7d7f7i32) as u16;
            (*E).var_E =
                crate::rtl::rom_table_word!(game, 0xa7cd63, (game.next_random() as i32 & 7) as isize);
        }
    };
    [10999850] Phantoon_FadeOutBeforeFirstFireballRain [] {
        Phantoon_FadeOut(0xc, game);
        Phantoon_AdjustSpeedAndMoveInFigure8(game);
        Phantoon_Func_7(k, game);
        let E = Get_Phantoon(0x40, game);
        let v2: bool = (*E).var_A == 1;
        (*E).var_A = (*E).var_A.wrapping_sub(1);
        let v3: bool = ((*E).var_A & 0x8000) != 0;
        if v2 || v3 {
            (*Get_Phantoon(0x80i32, game)).var_A = 0;
            (*Get_Phantoon(k as i32, game)).var_F = (0xa7d73fi32) as u16;
            if (*Get_Phantoon(0i32, game)).base.x_pos as i32 - 128i32 & 0x8000 != 0 {
                Phantoon_Func_6(k, 0, game);
            } else {
                Phantoon_Func_6(k, 2, game);
            }
        }
    };
    [10999900] Phantoon_FadeOutBeforeEnrage [] {
        Phantoon_FadeOut(0xc, game);
        if (*Get_Phantoon(0x40i32, game)).var_F != 0 {
            let E = Get_Phantoon(k as i32, game);
            (*E).var_F = (0xa7d874i32) as u16;
            (*E).var_E = 120;
        }
    };
    [10999953] Phantoon_FadeInEnragedPhantoon [] {
        Phantoon_FadeIn(0xc, game);
        let E = Get_Phantoon(0x40, game);
        if (*E).var_F != 0 {
            (*Get_Phantoon(0i32, game)).var_F = (0xa7d8aci32) as u16;
            (*Get_Phantoon(k as i32, game)).var_E = 4;
            (*E).var_F = 0;
        }
    };
    [10999980] Phantoon_Enraged [] {
        let E = Get_Phantoon(k as i32, game);
        let v2: bool = (*E).var_E == 1;
        (*E).var_E = (*E).var_E.wrapping_sub(1);
        let v3: bool = ((*E).var_E & 0x8000) != 0;
        if v2 || v3 {
            if ((*Get_Phantoon(0x40i32, game)).var_F & 1) != 0 {
                let mut v5: i16 = 15;
                do_until! { (v8).wrapping_sub(9) < 0 ;
                    let mut v8: i16 = v5;
                    crate::update_place! { previous fresh24 = v5 => v5 - 1; }
                    SpawnEprojWithGfx(
                        ((fresh24) as u16) | (0x200u16),
                        k,
                        addr_kEproj_DestroyableFireballs as u16,
                        game,
                    );
                }
            } else {
                for i in (0i32..=6).rev() {
                    SpawnEprojWithGfx(
                        (i | 0x200) as u16,
                        k,
                        addr_kEproj_DestroyableFireballs as u16,
                        game,
                    );
                }
            }
            QueueSfx3_Max6(0x29, game);
            let E1 = Get_Phantoon(0x40, game);
            let v7: u16 = ((*E1).var_F).wrapping_add(1);
            (*E1).var_F = v7;
            if v7 as i32 - 8i32 & 0x8000 != 0 {
                (*Get_Phantoon(k as i32, game)).var_E = 128;
            } else {
                enemy_instruction! {[(*E1).base]; timer = 1, instruction = addr_kKraid_Ilist_CC91 as u16}
                (*E1).var_F = 0;
                (*Get_Phantoon(k as i32, game)).var_F = (0xa7d916i32) as u16;
            }
        }
    };
    [11000086] Phantoon_FadeoutAfterEnrage [] {
        Phantoon_FadeOut(0xc, game);
        if (*Get_Phantoon(0x40i32, game)).var_F != 0 {
            let E = Get_Phantoon(k as i32, game);
            (*E).var_F = (0xa7d6d4i32) as u16;
            (*E).var_E = 120;
        }
    };
    [11000110] Phantoon_CompleteSwoopAfterFatalShot [] {
        Phantoon_ChangeEyeSpriteBasedOnSamusDist(game);
        Phantoon_MoveInSwoopingPattern(k, game);
        let x_pos: u16 = (*Get_Phantoon(0i32, game)).base.x_pos;
        if x_pos as i32 - 96i32 & 0x8000 == 0 && x_pos as i32 - 160i32 & 0x8000 != 0 {
            (*Get_Phantoon(k as i32, game)).var_F = (0xa7d948i32) as u16;
        }
    };
    [11000136] Phantoon_DyingPhantoonFadeInOut [] {
        let E1 = Get_Phantoon(0x40, game);
        if ((*E1).var_C & 1) != 0 {
            Phantoon_FadeIn(0xc, game);
            if (*E1).var_F == 0 {
                return;
            }
        } else {
            Phantoon_FadeOut(0xc, game);
            if (*E1).var_F == 0 {
                return;
            }
        }
        (*E1).var_F = 0;
        let v2: u16 = ((*E1).var_C).wrapping_add(1);
        (*E1).var_C = v2;
        if v2 as i32 - 10i32 & 0x8000 == 0 {
            let E = Get_Phantoon(k as i32, game);
            (*E).var_F = (0xa7d98bi32) as u16;
            (*E).var_E = 15;
            (*Get_Phantoon(0x80i32, game)).var_F = 0;
            (*Get_Phantoon(0i32, game)).var_A = 0;
        }
    };
    [11000401] Phantoon_WavyDyingPhantoonAndCry [] {
        initialize_phantoon_motion(1, g_word_A7CDA3, game);
        (*Get_Phantoon(0xc0i32, game)).var_D = 0;
        (*Get_Phantoon(k as i32, game)).var_F = (0xa7da86i32) as u16;
        let mut Phantoon = Get_Phantoon(0x40, game);
        (*Phantoon).var_C = 2;
        let v2: u16 = ((*Get_Phantoon(0i32, game)).base.properties as i32
            & !(kEnemyProps_DisableSamusColl as i32
                | kEnemyProps_Tangible as i32
                | kEnemyProps_Invisible as i32)
            | kEnemyProps_Tangible as i32
            | kEnemyProps_Invisible as i32) as u16;
        (*Phantoon).base.properties = v2;
        (*Get_Phantoon(0x80i32, game)).base.properties = v2;
        (*Get_Phantoon(0xc0i32, game)).base.properties = v2;
        QueueSfx2_Max6(0x7e, game);
    };
    [11000454] Phantoon_DyingFadeOut [] {
        Phantoon_Func_3((words!((g_word_A7CDA1, g_word_A7CD9F))) as i32, game);
        let E = Get_Phantoon(0x40, game);
        if (*E).var_C == 0xffff {
            Phantoon_FadeOut(0xc, game);
            if (*E).var_F != 0 {
                (*Get_Phantoon(k as i32, game)).var_F = (0xa7dad7i32) as u16;
            }
        } else if (word!(game.nmi_frame_counter_word) & 0xf) == 0 {
            if byte!((*E).var_C; 0) == 0xf2 {
                (*E).var_C = -1i32 as u16;
                (*E).var_F = 0;
            } else {
                let v2: u8 = (byte!((*E).var_C; 0)).wrapping_add(16);
                byte!((*E).var_C; 0) = v2;
                *ram_ptr!(game; 0x57) = v2;
            }
        }
    };
    [11000535] Phantoon_AlmostDead [] {
        *ram_ptr!(game; 0x57) = 0;
        let E1 = Get_Phantoon(0x40, game);
        (*E1).parameter_1 = 0;
        word!(game.phantom_related_layer_flag &= !(0x4000));
        (*Get_Phantoon(0xc0i32, game)).parameter_1 = -1i32 as u16;
        let EK = Get_Phantoon(k as i32, game);
        (*EK).var_F = (0xa7db3di32) as u16;
        (*EK).var_E = 60;
        (*E1).var_F = 0;
        let E0 = Get_Phantoon(0, game);
        (*E0).base.x_pos = 384;
        (*E0).base.y_pos = 128;
        word_transfers!(game, descending, 512; le(tilemap_stuff, 511) <- value(824));
        let mut v5: u16 = word!(game.vram_write_queue_tail);
        (*gVramWriteEntry(word!(game.vram_write_queue_tail) as i32, game)).size = 1024;
        v5 = (v5).wrapping_add(2);
        (*gVramWriteEntry(v5 as i32, game)).size = 0x2000;
        v5 = (v5).wrapping_add(2);
        crate::update_place! { previous fresh19 = v5 => v5.wrapping_add(1); }
        *(&raw mut (*(gVramWriteEntry as unsafe fn(i32, &mut Game) -> *mut VramWriteEntry)(
            fresh19 as i32,
            game,
        ))
        .size as *mut u8) = 126;
        (*gVramWriteEntry(v5 as i32, game)).size = 18432;
        word!(game.vram_write_queue_tail = (v5).wrapping_add(2));
    };
    [11000637] Phantoon_Dead [] {
        let EK = Get_Phantoon(k as i32, game);
        if (*EK).var_E != 0 {
            (*EK).var_E = (*EK).var_E.wrapping_sub(1);
        } else if (word!(game.nmi_frame_counter_word) & 3) == 0 {
            let E1 = Get_Phantoon(0x40, game);
            (*E1).var_D = 12;
            if Phantoon_Func_9(game) & 1 != 0 {
                crate::update_place! { reference fresh22 = *ram_ptr!(game; 0x69) => (*fresh22) | (2u8); }
                scatter(DropSource::Phantoon, k, game);
                let E0 = Get_Phantoon(0, game);
                let v4: u16 = ((*E0).base.properties) | ((kEnemyProps_Deleted) as u16);
                (*E0).base.properties = v4;
                (*E1).base.properties = v4;
                (*Get_Phantoon(0x80i32, game)).base.properties = v4;
                (*Get_Phantoon(0xc0i32, game)).base.properties = v4;
                let area = word!(game.area_index);
                let bits = gameplay_flag_runtime::boss_word_at(area, BossWordOrder::Little, game);
                gameplay_flag_runtime::set_boss_word_le(area, bits | 1, game);
                spawn_hardcoded(SpawnHardcodedPlmArgs::new(0, 0x6, 0xb78b as VoidP), game);
                QueueMusic_Delayed8(3, game);
            }
        }
    };
    [11004335] Etecoon_Func_4 [] {
        if word!(game.door_transition_flag_enemies) == 0 {
            let E = Get_Etecoon(k as i32, game);
            if ((*E).var_E & 0x8000) == 0 {
                let v2: bool = (*E).var_E == 1;
                (*E).var_E = (*E).var_E.wrapping_sub(1);
                let v3: bool = ((*E).var_E & 0x8000) != 0;
                if v2 || v3 {
                    (*E).base.current_instruction = addr_kEtecoon_Ilist_E854 as u16;
                    (*E).var_F = (0xa7ea00i32) as u16;
                    (*E).var_E = 11;
                }
            } else if ({ let (actor, distance) = (k, 0x80); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y, distance)) }) != 0 {
                if ((*E).parameter_2 & 3) == 0 {
                    QueueSfx2_Max15(0x35, game);
                }
                enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kEtecoon_Ilist_E8D6 as u16}
                (*E).var_E = 256;
            }
        }
    };
    [11004471] Etecoon_Func_6 [] {
        if Etecoon_Func_3(k, game) & 1 != 0 {
            let E = Get_Etecoon(k as i32, game);
            if ((*E).var_A & 0x8000) == 0 {
                if ({ let (actor, distance) = (k, 0x40); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y, distance)) }) != 0
                    && ({ let (actor, distance) = (k, g_word_A7E910); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0
                {
                    let v2: u16 = { match actor_api::direction_to_player(&mut Runtime::new(game)) { actor_api::PlayerDirection::Up=>0u16,actor_api::PlayerDirection::UpRight=>1u16,actor_api::PlayerDirection::Right=>2u16,actor_api::PlayerDirection::DownRight=>3u16,actor_api::PlayerDirection::Down=>4u16,actor_api::PlayerDirection::DownLeft=>6u16,actor_api::PlayerDirection::Left=>7u16,actor_api::PlayerDirection::UpLeft=>8u16 } };
                    if v2 as i32 - 5i32 & 0x8000 != 0 {
                        (*E).base.current_instruction = addr_kEtecoon_Ilist_E81E as u16;
                        (*E).parameter_1 = 0;
                    } else {
                        (*E).base.current_instruction = addr_kEtecoon_Ilist_E876 as u16;
                        (*E).parameter_1 = 1;
                    }
                    (*E).var_E = 32;
                    (*E).base.instruction_timer = 1;
                    (*E).var_F = (0xa7eab5i32) as u16;
                } else {
                    (*E).var_E = 11;
                    (*E).var_F = (0xa7ea00i32) as u16;
                    enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kEtecoon_Ilist_E854 as u16}
                }
            } else {
                set_fields! {[(*E)], 0; var_A, var_B}
                enemy_instruction! {[(*E).base]; timer = 3, instruction = addr_stru_A7E862 as u16}
            }
        }
    };
    [11004674] Etecoon_Func_8 [] {
        if Etecoon_Func_2(k, game) & 1 != 0 {
            let E = Get_Etecoon(k as i32, game);
            (*E).var_C = g_word_A7E908;
            (*E).var_D = g_word_A7E90A;
            (*E).var_F = (0xa7eb2ci32) as u16;
            enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_stru_A7E880 as u16}
            (*E).parameter_1 = 1;
        }
    };
    [11004716] Etecoon_Func_9 [] {
        if ({ let (actor, amount) = (k, ((32u32) << 16 | 0u16 as u32) as i32); (actor_api::probe_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
            let E = Get_Etecoon(k as i32, game);
            enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_stru_A7E898 as u16}
            (*E).var_F = (0xa7eb50i32) as u16;
        } else {
            Etecoon_Func_2(k, game);
        };
    };
    [11004752] Etecoon_Func_10 [] {
        Etecoon_Func_1(k, game);
        if Etecoon_Func_2(k, game) & 1 != 0 {
            let E = Get_Etecoon(k as i32, game);
            if (*E).parameter_1 != 0 {
                (*E).base.current_instruction = addr_kEtecoon_Ilist_E870 as u16;
                (*E).parameter_1 = 0;
            } else {
                (*E).base.current_instruction = addr_kEtecoon_Ilist_E8C8 as u16;
                (*E).parameter_1 = 1;
            }
            (*E).base.instruction_timer = 1;
            (*E).var_F = (0xa7ebcdi32) as u16;
            (*E).var_E = 8;
            if word!(game.samus_x_pos) as i32 - 256i32 & 0x8000 == 0 {
                QueueSfx2_Max6(0x32, game);
            }
        } else if Etecoon_Func_3(k, game) & 1 != 0 {
            let E_0 = Get_Etecoon(k as i32, game);
            if (*E_0).parameter_1 != 0 {
                (*E_0).base.current_instruction = addr_kEtecoon_Ilist_E854 as u16;
            } else {
                (*E_0).base.current_instruction = addr_kEtecoon_Ilist_E8AC as u16;
            }
            (*E_0).base.instruction_timer = 1;
            set_fields! {[(*E_0)]; var_E = 11, var_F = (0xa7ec1bi32) as u16, var_A = g_word_A7E900,
            var_B = g_word_A7E902, };
        }
    };
    [11004877] Etecoon_Func_11 [] {
        Etecoon_Func_1(k, game);
        let E = Get_Etecoon(k as i32, game);
        let v2: bool = (*E).var_E == 1;
        (*E).var_E = (*E).var_E.wrapping_sub(1);
        let v3: bool = ((*E).var_E & 0x8000) != 0;
        if v2 || v3 {
            if (*E).parameter_1 != 0 {
                (*E).base.current_instruction = addr_kEtecoon_Ilist_E894 as u16;
                (*E).var_C = g_word_A7E908;
                (*E).var_D = g_word_A7E90A;
            } else {
                (*E).base.current_instruction = addr_kEtecoon_Ilist_E83C as u16;
                (*E).var_C = g_word_A7E90C;
                (*E).var_D = g_word_A7E90E;
            }
            (*E).base.instruction_timer = 1;
            (*E).var_F = (0xa7eb50i32) as u16;
            (*E).var_A = g_word_A7E900;
            (*E).var_B = g_word_A7E902;
        }
    };
    [11005079] Etecoon_Func_16 [] {
        Etecoon_Func_2(k, game);
        let E = Get_Etecoon(k as i32, game);
        if (*E).base.x_pos as i32 - 537i32 & 0x8000 != 0 {
            (*E).var_E = 11;
            (*E).var_F = (0xa7edc7i32) as u16;
            enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kEtecoon_Ilist_E854 as u16}
        }
    };
    [11005115] Etecoon_Func_17 [] {
        Etecoon_Func_2(k, game);
        let E = Get_Etecoon(k as i32, game);
        if (*E).base.x_pos as i32 - 600i32 & 0x8000 == 0 {
            (*E).var_E = 11;
            (*E).var_F = (0xa7edc7i32) as u16;
            enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kEtecoon_Ilist_E854 as u16}
        }
    };
    [11005151] Etecoon_Func_18 [] {
        Etecoon_Func_2(k, game);
        let E = Get_Etecoon(k as i32, game);
        if (*E).base.x_pos as i32 - 600i32 & 0x8000 == 0 {
            (*E).var_F = (0xa7ed09i32) as u16;
            (*E).var_A = g_word_A7E904;
            (*E).var_B = g_word_A7E906;
            enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_stru_A7E898 as u16}
        }
    };
    [11005193] Etecoon_Func_19 [] {
        Etecoon_Func_2(k, game);
        Etecoon_Func_3(k, game);
        let E = Get_Etecoon(k as i32, game);
        if (*E).base.x_pos as i32 - 680i32 & 0x8000 == 0 {
            enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_stru_A7E880 as u16}
            (*E).var_F = (0xa7ed2ai32) as u16;
        }
    };
    [11005226] Etecoon_Func_20 [] {
        Etecoon_Func_2(k, game);
        let E = Get_Etecoon(k as i32, game);
        if (*E).base.x_pos as i32 - 840i32 & 0x8000 == 0 {
            enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_stru_A7E898 as u16}
            (*E).var_F = (0xa7ed54i32) as u16;
            (*E).var_A = -1i32 as u16;
            (*E).var_B = g_word_A7E906;
        }
    };
    [11005268] Etecoon_Func_21 [] {
        Etecoon_Func_2(k, game);
        if Etecoon_Func_3(k, game) & 1 != 0 {
            let E = Get_Etecoon(k as i32, game);
            (*E).var_E = 11;
            enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kEtecoon_Ilist_E854 as u16}
            (*E).var_F = (0xa7edc7i32) as u16;
        }
    };
    [11005301] Etecoon_Func_22 [] {
        Etecoon_Func_1(k, game);
        if Etecoon_Func_3(k, game) & 1 != 0 {
            let E = Get_Etecoon(k as i32, game);
            if ((*E).var_A & 0x8000) == 0 {
                (*E).var_E = 11;
                enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kEtecoon_Ilist_E854 as u16}
                if ((*E).parameter_2 & 2) != 0 && (*E).base.x_pos as i32 - 832i32 & 0x8000 != 0 {
                    (*E).var_F = (0xa7ee3ei32) as u16;
                } else {
                    (*E).var_F = (0xa7edc7i32) as u16;
                }
            } else {
                set_fields! {[(*E)], 0; var_A, var_B}
                enemy_instruction! {[(*E).base]; timer = 3, instruction = addr_stru_A7E862 as u16}
            }
        }
    };
    [11005383] Etecoon_Func_23 [] {
        Etecoon_Func_1(k, game);
        let E = Get_Etecoon(k as i32, game);
        let v2: bool = (*E).var_E == 1;
        (*E).var_E = (*E).var_E.wrapping_sub(1);
        let v3: bool = ((*E).var_E & 0x8000) != 0;
        if v2 || v3 {
            let v4: u16 = ((*E).parameter_1).wrapping_add(256);
            (*E).parameter_1 = v4;
            if (v4 as i32 & 0xff00) - 1024i32 & 0x8000 != 0 || {
                (*E).parameter_1 = byte!((*E).parameter_1; 0) as u16;
                ((*E).parameter_2 & 2) != 0
            } {
                (*E).var_F = (0xa7ed75i32) as u16;
                (*E).base.current_instruction = ((*E).base.current_instruction).wrapping_add(2);
            } else {
                (*E).var_F = (0xa7ee9ai32) as u16;
                (*E).base.current_instruction = addr_stru_A7E880 as u16;
                (*E).var_C = g_word_A7E908;
                (*E).var_D = g_word_A7E90A;
            }
            (*E).var_A = g_word_A7E900;
            (*E).var_B = g_word_A7E902;
            (*E).base.instruction_timer = 1;
            if word!(game.samus_x_pos) as i32 - 256i32 & 0x8000 == 0 {
                QueueSfx2_Max6(0x33, game);
            }
        }
    };
    [11005502] Etecoon_Func_24 [] {
        Etecoon_Func_1(k, game);
        let E = Get_Etecoon(k as i32, game);
        let v2: bool = (*E).var_E == 1;
        (*E).var_E = (*E).var_E.wrapping_sub(1);
        let v3: bool = ((*E).var_E & 0x8000) != 0;
        if v2 || v3 {
            if ({ let (actor, distance) = (k, 0x40); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y, distance)) }) != 0 && ({ let (actor, distance) = (k, 0x30); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0
            {
                (*E).base.current_instruction = addr_stru_A7E880 as u16;
                (*E).var_F = (0xa7ecdfi32) as u16;
            } else {
                (*E).var_A = g_word_A7E900;
                (*E).var_B = g_word_A7E902;
                (*E).base.current_instruction = ((*E).base.current_instruction).wrapping_add(2);
                (*E).var_F = (0xa7ed75i32) as u16;
                if word!(game.samus_x_pos) as i32 - 256i32 & 0x8000 == 0 {
                    QueueSfx2_Max6(0x33, game);
                }
            }
            (*E).base.instruction_timer = 1;
        }
    };
    [11005594] Etecoon_Func_25 [] {
        Etecoon_Func_2(k, game);
        let E = Get_Etecoon(k as i32, game);
        if (*E).base.x_pos as i32 - 600i32 & 0x8000 == 0 {
            (*E).var_F = (0xa7eeb8i32) as u16;
            enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_stru_A7E898 as u16}
        }
    };
    [11005624] Etecoon_Func_26 [] {
        Etecoon_Func_2(k, game);
        if Etecoon_Func_3(k, game) & 1 != 0 {
            let E = Get_Etecoon(k as i32, game);
            set_fields! {[(*E)]; var_C = g_word_A7E90C, var_D = g_word_A7E90E,
            var_F = (0xa7eb02i32) as u16, var_A = g_word_A7E900, var_B = g_word_A7E902, };
            enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_stru_A7E828 as u16}
        }
    };
    [11007344] Dachora_Func_2 [] {
        ({ let (actor, amount) = (k, ((1u32) << 16 | 0u16 as u32) as i32); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 });
        if ({ let (actor, distance) = (k, 0x40); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y, distance)) }) != 0
            && ({ let (actor, distance) = (k, g_word_A7F4C9); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0
        {
            let E = Get_Dachora(k as i32, game);
            if (*E).parameter_1 != 0 {
                (*E).base.current_instruction = addr_kDachora_Ilist_F48B as u16;
            } else {
                (*E).base.current_instruction = addr_kDachora_Ilist_F3C9 as u16;
            }
            (*E).base.instruction_timer = 1;
            (*E).var_F = (0xa7f5bci32) as u16;
            (*E).var_A = g_word_A7F4CD;
            QueueSfx2_Max15(0x1d, game);
        }
    };
    [11007420] Dachora_Func_3 [] {
        let E = Get_Dachora(k as i32, game);
        crate::update_place! { previous fresh2 = (*E).var_A => (*E).var_A.wrapping_sub(1); }
        if fresh2 == 1 {
            if (*E).parameter_1 != 0 {
                (*E).base.current_instruction = addr_kDachora_Ilist_F407 as u16;
                (*E).var_F = (0xa7f65ei32) as u16;
            } else {
                (*E).base.current_instruction = addr_kDachora_Ilist_F345 as u16;
                (*E).var_F = (0xa7f5edi32) as u16;
            }
            (*E).base.instruction_timer = 1;
            (*E).var_E = 1;
        }
    };
    [11007469] Dachora_Func_4 [] {
        let amt: i32 = -Dachora_Func_6(k, game);
        let E = Get_Dachora(k as i32, game);
        if ({ let (actor, amount) = (k, amt); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) != 0 || {
            ({ let actor = k; actor_api::snap_to_slopes(&mut Runtime::new(game), Key::from_legacy(actor)) as u8 });
            (*E).base.x_pos as i32 - 96i32 & 0x8000 != 0
        } {
            (*E).base.current_instruction = addr_kDachora_Ilist_F407 as u16;
            (*E).var_F = (0xa7f65ei32) as u16;
            set_fields! {[(*E)], 1; var_E, parameter_1}
            (*E).base.instruction_timer = 1;
            set_fields! {[(*E)], 0; var_A, var_B}
            Dachora_Func_1(addr_kDachora_Palette as u16, k, game);
        }
    };
    [11007582] Dachora_Func_5 [] {
        let amt: i32 = Dachora_Func_6(k, game);
        let E = Get_Dachora(k as i32, game);
        if ({ let (actor, amount) = (k, amt); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
            QueueSfx2_Max15(0x71, game);
            (*E).base.current_instruction = addr_kDachora_Ilist_F345 as u16;
            (*E).var_F = (0xa7f5edi32) as u16;
            set_fields! {[(*E)], 0; parameter_1, var_A}
            Dachora_Func_1(addr_kDachora_Palette as u16, k, game);
        } else {
            ({ let actor = k; actor_api::snap_to_slopes(&mut Runtime::new(game), Key::from_legacy(actor)) as u8 });
            if (*E).base.x_pos as i32 - 1152i32 & 0x8000 == 0 {
                (*E).base.current_instruction = addr_kDachora_Ilist_F4B3 as u16;
                (*E).var_F = (0xa7f78fi32) as u16;
                (*E).var_A = g_word_A7F4CF;
                (*E).base.y_pos = ((*E).base.y_pos).wrapping_add(8);
                QueueSfx2_Max6(0x3d, game);
            } else {
                return;
            }
        }
        set_fields! {[(*E)], 0; var_E, var_B}
        (*E).base.instruction_timer = 1;
    };
    [11007887] Dachora_Func_7 [] {
        Dachora_Func_10(k, game);
        let E = Get_Dachora(k as i32, game);
        crate::update_place! { previous fresh3 = (*E).var_A => (*E).var_A.wrapping_sub(1); }
        if fresh3 == 1 {
            enemy_instruction! {[(*E).base]; instruction = ((*E).base.current_instruction).wrapping_add(2), timer = 1}
            (*E).var_F = (0xa7f806i32) as u16;
            let E1 = Get_Dachora(k as i32 + 64, game);
            set_fields! {[(*E1)], 0; var_A, var_D}
            (*Get_Dachora(k as i32 + 128i32, game)).var_D = 0;
            (*Get_Dachora(k as i32 + 192i32, game)).var_D = 0;
            (*Get_Dachora(k as i32 + 256i32, game)).var_D = 0;
            set_fields! {[(*E)], 0; var_C, var_D}
            (*E).base.y_pos = ((*E).base.y_pos).wrapping_sub(8);
            QueueSfx2_Max6(0x3b, game);
            if (*E).parameter_1 != 0 {
                (*Get_Dachora(k as i32 + 64i32, game))
                    .base
                    .current_instruction = addr_kDachora_Ilist_F4B9 as u16;
                (*Get_Dachora(k as i32 + 128i32, game))
                    .base
                    .current_instruction = addr_kDachora_Ilist_F4B9 as u16;
                (*Get_Dachora(k as i32 + 192i32, game))
                    .base
                    .current_instruction = addr_kDachora_Ilist_F4B9 as u16;
                (*Get_Dachora(k as i32 + 256i32, game))
                    .base
                    .current_instruction = addr_kDachora_Ilist_F4B9 as u16;
            } else {
                (*Get_Dachora(k as i32 + 64i32, game))
                    .base
                    .current_instruction = addr_kDachora_Ilist_F3F7 as u16;
                (*Get_Dachora(k as i32 + 128i32, game))
                    .base
                    .current_instruction = addr_kDachora_Ilist_F3F7 as u16;
                (*Get_Dachora(k as i32 + 192i32, game))
                    .base
                    .current_instruction = addr_kDachora_Ilist_F3F7 as u16;
                (*Get_Dachora(k as i32 + 256i32, game))
                    .base
                    .current_instruction = addr_kDachora_Ilist_F3F7 as u16;
            }
            (*Get_Dachora(k as i32 + 64i32, game))
                .base
                .instruction_timer = 1;
            (*Get_Dachora(k as i32 + 128i32, game))
                .base
                .instruction_timer = 1;
            (*Get_Dachora(k as i32 + 192i32, game))
                .base
                .instruction_timer = 1;
            (*Get_Dachora(k as i32 + 256i32, game))
                .base
                .instruction_timer = 1;
        }
    };
    [11008006] Dachora_Func_8 [] {
        Dachora_Func_10(k, game);
        Dachora_Func_9(k, game);
        let E = Get_Dachora(k as i32, game);
        words!(
            (*E),
            var_C,
            var_D += words!(game.samus_y_accel, samus_y_subaccel)
        );
        words!((*E), var_A, var_B += words!((*E), var_C, var_D));
        let mut v6: u16 = (*E).var_A;
        if v6 as i32 - 15i32 & 0x8000 == 0 {
            v6 = 15;
        }
        let amt: i32 = (words!((v6, (*E).var_B))) as i32;
        if ({ let (actor, amount) = (k, -amt); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
            if (*E).parameter_1 != 0 {
                (*E).base.current_instruction = addr_kDachora_Ilist_F3FF as u16;
                (*E).parameter_1 = 0;
            } else {
                (*E).base.current_instruction = addr_kDachora_Ilist_F4C1 as u16;
                (*E).parameter_1 = 1;
            }
            (*E).var_F = (0xa7f935i32) as u16;
            (*E).base.instruction_timer = 1;
            set_fields! {[(*E)], 0; var_A, var_B, var_E}
            Dachora_Func_1(addr_kDachora_Palette as u16, k, game);
            QueueSfx2_Max6(0x3c, game);
        }
    };
    [11008309] Dachora_Func_11 [] {
        let E = Get_Dachora(k as i32, game);
        words!(
            (*E),
            var_A,
            var_B += words!(game.samus_y_accel, samus_y_subaccel)
        );
        let mut amt: i32 = (words!((*E), var_A, var_B)) as i32;
        if (amt >> 16) - 10i32 & 0x8000 == 0 {
            amt = (10i16 as i32) << 16;
        }
        if ({ let (actor, amount) = (k, amt); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
            if (*E).parameter_1 != 0 {
                (*E).base.current_instruction = addr_kDachora_Ilist_F407 as u16;
                (*E).var_F = (0xa7f65ei32) as u16;
            } else {
                (*E).base.current_instruction = addr_kDachora_Ilist_F345 as u16;
                (*E).var_F = (0xa7f5edi32) as u16;
            }
            (*E).base.instruction_timer = 1;
            set_fields! {[(*E)], 0; var_A, var_B}
        }
    };
    [11008396] Dachora_Func_12 [] {
        let E = Get_Dachora(k as i32, game);
        let dachor_var_D: u16 = (*E).var_D;
        if dachor_var_D != 0 {
            (*E).var_D = (dachor_var_D).wrapping_sub(1);
            let keep_hidden: bool = if (k & 0x40) != 0 {
                (word!(game.nmi_frame_counter_word) & 1) == 0
            } else {
                (word!(game.nmi_frame_counter_word) & 1) != 0
            };
            if keep_hidden {
            } else {
                (*E).base.properties =
                    ((*E).base.properties as i32 & !(kEnemyProps_Invisible as i32)) as u16;
                return;
            }
        }
        (*E).base.properties = ((*E).base.properties) | ((kEnemyProps_Invisible) as u16);
    };
    @functions [pub unsafe fn (v0: u16, game: &mut Game)];
    [10999388] Phantoon_BecomesSolidAndBodyVuln [] {
        Phantoon_ChangeEyeSpriteBasedOnSamusDist(game);
        word!(game.phantom_related_layer_flag &= !(0x4000));
        Phantoon_BeginSwoopingPattern(v0, game);
        let E = Get_Phantoon(0, game);
        enemy_instruction! {[(*E).base]; timer = 1, instruction = addr_kKraid_Ilist_CC47 as u16}
    };
    @countdown [pub unsafe fn (k: u16, game: &mut Game)];
    before {
        let EK = Get_Phantoon(k as i32, game);
        let v2: bool = (*EK).var_E == 1;
        (*EK).var_E = (*EK).var_E.wrapping_sub(1);
        let v3: bool = ((*EK).var_E & 0x8000) != 0;
    }
    when v2 || v3;
    [10998953] Phantoon_Spawn8FireballsInCircleAtStart {
        let E0 = Get_Phantoon(0, game);
        SpawnEprojWithGfx((*E0).var_A, k, addr_kEproj_StartingFireballs as u16, game);
        QueueSfx3_Max6(0x1d, game);
        (*EK).var_E = 30;
        let v5: u16 = ((*E0).var_A).wrapping_add(1);
        (*E0).var_A = v5;
        if v5 as i32 - 8i32 & 0x8000 == 0 {
            (*E0).var_A = 0;
            (*Get_Phantoon(k as i32 + 128i32, game)).var_B = 0;
            (*EK).var_F = (0xa7d4eei32) as u16;
            (*EK).var_E = 30;
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0, 0x6, 0xb781 as VoidP), game);
        }
    };
    [10999022] Phantoon_WaitBetweenSpawningAndSpinningFireballs {
        (*EK).var_E = 240;
        (*EK).var_B = 1;
        (*EK).var_F = (0xa7d508i32) as u16;
    };
    [10999048] Phantoon_SpawnFireballsBeforeFight {
        (*EK).var_B = 0;
        word!(game.phantom_related_layer_flag |= (0x4000u16));
        (*EK).var_F = (0xa7d54ai32) as u16;
        let E3 = Get_Phantoon(0xc0, game);
        (*E3).parameter_1 = -32767i32 as u16;
        (*EK).var_E = 120;
        initialize_phantoon_motion(2, g_word_A7CDA3, game);
        (*E3).var_D = g_word_A7CD9D;
        (*Get_Phantoon(0x40i32, game)).var_F = 0;
        QueueMusic_Delayed8(5, game);
    };
    [10999190] Phantoon_PickPatternForRound1 {
        let E1 = Get_Phantoon(0x40, game);
        (*E1).parameter_1 = 0;
        (*EK).var_F = (0xa7d5e7i32) as u16;
        (*E1).var_A = crate::rtl::rom_table_word!(
            game,
            0xa7cd53,
            (word!(game.nmi_frame_counter_word) as i32 >> 1 & 3) as isize
        );
        if (game.next_random() & 1) != 0 {
            let E = Get_Phantoon(0, game);
            set_fields! {[(*E)], 0; var_C, var_B, var_D}
            (*E1).var_C = 1;
            (*E).var_A = 533;
        } else {
            let E_0 = Get_Phantoon(0, game);
            (*E_0).var_C = 1;
            set_fields! {[(*E_0)], 0; var_B, var_D}
            (*E1).var_C = 0;
            (*E_0).var_A = 0;
        }
    };
    [10999508] Phantoon_WaitAfterFadeOut {
        (*EK).var_F = (0xa7d6e2i32) as u16;
    };
    [10999688] Phantoon_FollowSamusWithEyeDuringFireballRain {
        (*Get_Phantoon(k as i32 + 128i32, game)).var_B = 0;
        let E2 = Get_Phantoon(0x80, game);
        if (*E2).var_A != 0 {
            (*E2).var_A = 0;
            (*Get_Phantoon(0i32, game)).parameter_2 = 1;
            Phantoon_BeginSwoopingPattern(k, game);
        } else {
            (*EK).var_F = (0xa7d7d5i32) as u16;
            let E1 = Get_Phantoon(0x40, game);
            (*E1).var_F = 0;
            let E0 = Get_Phantoon(0, game);
            (*E0).base.instruction_timer = 1;
            (*E1).base.instruction_timer = 1;
            (*E0).base.current_instruction = addr_kKraid_Ilist_CC41 as u16;
            (*E1).base.current_instruction = addr_kKraid_Ilist_CC91 as u16;
            (*E0).base.properties = ((*E0).base.properties) | ((kEnemyProps_Tangible) as u16);
            word!(game.phantom_related_layer_flag |= (0x4000u16));
        }
    };
    [10999799] Phantoon_SpawnRainingFireballs {
        let a: u16 = (game.next_random()) & (7u16);
        let v4: i32 = 8 * a as i32 >> 1;
        let E0 = Get_Phantoon(0, game);
        (*E0).var_A = crate::rtl::rom_table_word!(game, 0xa7cdad, v4 as isize);
        (*E0).base.x_pos = crate::rtl::rom_table_word!(game, 0xa7cdad, (v4 + 1) as isize);
        (*E0).base.y_pos = crate::rtl::rom_table_word!(game, 0xa7cdad, (v4 + 2) as isize);
        (*Get_Phantoon(0x40i32, game)).var_C = 0;
        (*EK).var_F = (0xa7d73fi32) as u16;
        Phantoon_Func_6(k, a, game);
    };
    [10999924] Phantoon_MoveEnragedPhantoonToTopCenter {
        (*EK).var_F = (0xa7d891i32) as u16;
        let E_0 = Get_Phantoon(0, game);
        (*E_0).base.x_pos = 128;
        (*E_0).base.y_pos = 32;
        (*Get_Phantoon(0x40i32, game)).var_F = 0;
    };
    [11000203] Phantoon_DyingPhantoonExplosions {
        let E2 = Get_Phantoon(0x80, game);
        let v5: u16 = (4u16).wrapping_mul((*E2).var_F);
        let E0 = Get_Phantoon(0, game);
        game.eproj_spawn_pt = Point16U {
            x: (((byte!(@pointer rom_ptr!(game; 0xa7da1d); v5 as isize) as i8) as u16)
                .wrapping_add((*E0).base.x_pos)),
            y: (((byte!(@pointer rom_ptr!(game; 0xa7da1d); (v5 as i32 + 1i32) as isize) as i8) as u16)
                .wrapping_add((*E0).base.y_pos)),
        };
        let v11: u16 = byte!(@pointer rom_ptr!(game; 0xa7da1d); (v5 as i32 + 2) as isize) as u16;
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, v11, game);
        if v11 == 29 {
            QueueSfx2_Max6(0x24, game);
        } else {
            QueueSfx2_Max6(0x2b, game);
        }
        (*EK).var_E = byte!(@pointer rom_ptr!(game; 0xa7da1d); (v5 as i32 + 3) as isize) as u16;
        let v9: u16 = ((*E2).var_F).wrapping_add(1);
        (*E2).var_F = v9;
        if v9 as i32 - 13i32 & 0x8000 == 0 {
            (*E2).var_F = 5;
            let v10: u16 = ((*E0).var_A).wrapping_add(1);
            (*E0).var_A = v10;
            if v10 as i32 - 3i32 & 0x8000 == 0 {
                (*EK).var_F = (0xa7da51i32) as u16;
            }
        }
    };
    @countdown [pub unsafe fn (k: u16, game: &mut Game)];
    before {
        let E = Get_Etecoon(k as i32, game);
        let v2: bool = (*E).var_E == 1;
        (*E).var_E = (*E).var_E.wrapping_sub(1);
        let v3: bool = ((*E).var_E & 0x8000) != 0;
    }
    when v2 || v3;
    [11004416] Etecoon_Func_5 {
        (*E).var_A = g_word_A7E900;
        (*E).var_B = g_word_A7E902;
        enemy_instruction! {[(*E).base]; instruction = ((*E).base.current_instruction).wrapping_add(2), timer = 1}
        (*E).var_F = (0xa7ea37i32) as u16;
        if word!(game.samus_x_pos) as i32 - 256i32 & 0x8000 == 0 {
            QueueSfx2_Max6(0x33, game);
        }
    };
    [11004597] Etecoon_Func_7 {
        enemy_instruction! {[(*E).base]; instruction = ((*E).base.current_instruction).wrapping_add(2), timer = 1}
        if (*E).parameter_1 != 0 {
            (*E).var_C = g_word_A7E908;
            (*E).var_D = g_word_A7E90A;
            (*E).var_F = (0xa7eb2ci32) as u16;
        } else {
            (*E).var_C = g_word_A7E90C;
            (*E).var_D = g_word_A7E90E;
            (*E).var_F = (0xa7eb02i32) as u16;
        }
        (*E).var_A = g_word_A7E900;
        (*E).var_B = g_word_A7E902;
    };
    [11004955] Etecoon_Func_12 {
        (*E).var_A = g_word_A7E900;
        (*E).var_B = g_word_A7E902;
        funcs_A2A79[byte!((*E).parameter_2; 0) as usize](k, game);
        (*Get_Etecoon(k as i32, game)).base.instruction_timer = 1;
    };
}
