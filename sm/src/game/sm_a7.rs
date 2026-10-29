use crate::game::actors;
use crate::game::actors::access::ActorKey as Key;
use crate::game::actors::api as actor_api;
use crate::game::actors::dachora::Dachora_Func_1;
use crate::game::actors::kraid_fingernails::KraidFingernailInit;
use crate::game::actors::phantoon_actions::Phantoon_SetColorBasedOnHp_FindEntry;
use crate::game::decompression::decompress_to_ram;
use crate::game::enemy_projectiles::{DropSource, scatter_drop as scatter};
use crate::game::enemy_script::EnemyScript;
use crate::game::gameplay_flag_host::BossWordOrder;
use crate::game::gameplay_flag_runtime;
use crate::game::kraid_tables;
use crate::game::music::QueueMusic_Delayed8;
pub use crate::game::prelude::*;
use crate::game::session_runtime::{Runtime, actor_species_pre_instruction, native_rom_diagnostic};
use crate::ram_map::slot;
use crate::rom::{RomRecord, lorom_offset};
use crate::rtl::rom_record;
use core::mem::offset_of;

pub use crate::game::constants::bank_86::*;

use crate::game::display_refresh::{GraphicsAsset, queue_native_transfer};
use crate::game::enemy_layouts::KraidLayout as Enemy_Kraid;
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
static g_word_A7A916: u16 = 0x120;
static g_word_A7A918: u16 = 0xa0;
static g_word_A7A91A: u16 = 0x40;
static g_word_A7A91C: u16 = 3;
static g_word_A7A920: u16 = 3;
static g_word_A7A922: u16 = 4;
static g_word_A7A926: u16 = 0x8000;
static g_word_A7A928: u16 = 3;
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
        slot!(game, Enemy_Phantoon[0x80i32].var_A = 0);
        let E = (0) as i32;
        slot!(game, Enemy_Phantoon[E].base; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_CC4D as u16);
        slot!(game, Enemy_Phantoon[E].base.properties = (slot!(game, Enemy_Phantoon[E].base.properties) as i32 & !(kEnemyProps_Tangible as i32)) as u16);
        slot!(game, Enemy_Phantoon[E].var_E = crate::rtl::rom_table_word!(game, 0xa7cd41, (game.next_random() as i32 & 7) as isize));
        slot!(game, Enemy_Phantoon[E].var_F = (0xa7d60di32) as u16);
        let E1 = (0x40) as i32;
        slot!(game, Enemy_Phantoon[E1].base; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_CC9D as u16);
    };
    [10997878] Phantoon_PickPatternForRound2 [] {
        let E = (0) as i32;
        slot!(game, Enemy_Phantoon[E].var_E = 60);
        let v1: u16 =
            crate::rtl::rom_table_word!(game, 0xa7cd53, (game.next_random() as i32 & 7) as isize);
        let E1 = (0x40) as i32;
        slot!(game, Enemy_Phantoon[E1].var_A = v1);
        if (word!(game.nmi_frame_counter_word) & 1) != 0 {
            if slot!(game, Enemy_Phantoon[E1].var_C) == 0 {
                let mut v4: i16 = ((slot!(game, Enemy_Phantoon[E].var_A)) as i16).wrapping_sub(1);
                slot!(game, Enemy_Phantoon[E].var_A = v4 as u16);
                if v4 < 0 {
                    slot!(game, Enemy_Phantoon[E].var_A = 533);
                }
            }
            slot!(game, Enemy_Phantoon[E]; var_C = 0, var_B = 0, var_D = 0);
            slot!(game, Enemy_Phantoon[E1].var_C = 1);
        } else {
            if slot!(game, Enemy_Phantoon[E1].var_C) != 0 {
                let v3: u16 = (slot!(game, Enemy_Phantoon[E].var_A)).wrapping_add(1);
                slot!(game, Enemy_Phantoon[E].var_A = v3);
                if v3 as i32 - 534i32 & 0x8000 == 0 {
                    slot!(game, Enemy_Phantoon[E].var_A = 0);
                }
            }
            slot!(game, Enemy_Phantoon[E].var_C = 1);
            slot!(game, Enemy_Phantoon[E]; var_B = 0, var_D = 0);
            slot!(game, Enemy_Phantoon[E1].var_C = 0);
        }
        if slot!(game, Enemy_Phantoon[E].parameter_2) != 0 {
            slot!(game, Enemy_Phantoon[E1].var_F = 0);
            slot!(game, Enemy_Phantoon[E].var_F = (0xa7d82ai32) as u16);
        } else {
            slot!(game, Enemy_Phantoon[E].var_F = (0xa7d5e7i32) as u16);
        };
    };
    @external {
        10997598 => [actors::phantoon_actions::spawn_fireball(k, game)];
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
    let E = (word!(game.cur_enemy_index) as i32) as i32;
    slot!(
        game,
        Enemy_Kraid[E].base.properties =
            slot!(game, Enemy_Kraid[E].base.properties) & 0x50ff | 0x700
    );
}
pub unsafe fn Kraid_SetupGfxWithTilePrioClear(r18: u16, game: &mut Game) {
    decompress_to_ram(0xb9fa38, 0x4000, game);
    decompress_to_ram(0xb9fe3e, 0x2000, game);
    let E = (0) as i32;
    slot!(game, Enemy_Kraid[E]; kraid_hurt_frame = 0, kraid_hurt_frame_timer = 0);
    for v1 in (0u16..1536).step_by(2) {
        match (
            (word!(game.tilemap_stuff[(v1 >> 1)])) & (0xdfffu16),
            usize::try_from((0x2000) + (((v1 as i32 >> 1) + 1024) as isize) * 2)
                .expect("native word outside RAM"),
        ) {
            (value, at) => {
                let bytes = u16::to_ne_bytes(value);
                game.ram().write_byte(at, bytes[0]);
                game.ram().write_byte(at + 1, bytes[1]);
            }
        };
    }
    for v2 in 0u16..1024 {
        word!(
            game.tilemap_stuff[v2] = (r18)
                & ({
                    let at = 0x4000 + offset_of!(Ram4000, xray_tilemaps) + v2 as usize * 2;
                    u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
                })
        );
    }
}
pub unsafe fn Kraid_DrawRoomBg(game: &mut Game) {
    word_transfers!(game, ascending, 16; le(target_palettes, 96) <- rom(0xa786c7, 0));
    word!(game.palette_change_num = 0);
    let v2: u16 = word!(game.vram_write_queue_tail);
    let mut v3 = (0xd0 + (word!(game.vram_write_queue_tail) as i32)) as i32;
    match (
        512,
        usize::try_from((v3) as isize + offset_of!(VramWriteEntry, size) as isize)
            .expect("VRAM entry outside RAM"),
    ) {
        (value, at) => {
            let bytes = u16::to_ne_bytes(value);
            game.ram().write_byte(at, bytes[0]);
            game.ram().write_byte(at + 1, bytes[1]);
        }
    };
    match (
        addr_kKraidsRoomBg as i32 as VoidP,
        usize::try_from((v3) as isize + offset_of!(VramWriteEntry, src.addr) as isize)
            .expect("VRAM entry outside RAM"),
    ) {
        (value, at) => {
            let bytes = u16::to_ne_bytes(value);
            game.ram().write_byte(at, bytes[0]);
            game.ram().write_byte(at + 1, bytes[1]);
        }
    };
    match (
        -89i32 as u8,
        usize::try_from((v3) as isize + offset_of!(VramWriteEntry, src.bank) as isize)
            .expect("VRAM entry outside RAM"),
    ) {
        (value, at) => game.ram().write_byte(at, value),
    };
    match (
        (((game.ram().read_byte(0x5d) as i32 & 0xf) << 8) + 16128) as u16,
        usize::try_from((v3) as isize + offset_of!(VramWriteEntry, vram_dst) as isize)
            .expect("VRAM entry outside RAM"),
    ) {
        (value, at) => {
            let bytes = u16::to_ne_bytes(value);
            game.ram().write_byte(at, bytes[0]);
            game.ram().write_byte(at + 1, bytes[1]);
        }
    };
    word!(game.vram_write_queue_tail = (v2).wrapping_add(7));
}
pub unsafe fn Kraid_GetsBig_Thinking_Setup(game: &mut Game) {
    unsafe {
        slot!(game, Enemy_Kraid[0i32].kraid_var_A = (0xa7aec4i32) as u16);
        Kraid_SetLintYAndRandomThinkTimer(game);
    }
}
pub unsafe fn Kraid_Mainloop_Thinking_Setup(game: &mut Game) {
    unsafe {
        slot!(game, Enemy_Kraid[0i32].kraid_var_A = (0xa7aea4i32) as u16);
        Kraid_SetLintYAndRandomThinkTimer(game);
    }
}
pub unsafe fn Kraid_Lints_Enable(k: u16, a: u16, game: &mut Game) {
    let E = (k as i32) as i32;
    slot!(game, Enemy_Kraid[E].kraid_var_F = a);
    slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7b923i32) as u16);
    slot!(game, Enemy_Kraid[E].kraid_next = (0xa7b832i32) as u16);
    slot!(game, Enemy_Kraid[E].kraid_var_B = 0);
}
#[forbid(unsafe_code)]
fn kraid_native_record(address: u16, game: &mut Game) -> RomRecord {
    let address = 0xa70000 | u32::from(address);
    if address & 0x8000 == 0 {
        native_rom_diagnostic(address, game);
    }
    RomRecord {
        image: game.capture_native_rom(),
        base: lorom_offset(address) as usize,
    }
}

macro_rules! advance_kraid_thinking_timer {
    ($enemy:ident, $next_state:literal, $game:ident) => {
        let kraid_thinking: u16 = $crate::ram_map::slot!($game, Enemy_Kraid[$enemy].kraid_thinking);
        if kraid_thinking != 0 {
            let v2: u16 = (kraid_thinking).wrapping_sub(1);
            $crate::ram_map::slot!($game, Enemy_Kraid[$enemy].kraid_thinking = v2);
            if v2 == 0 {
                $crate::ram_map::slot!(
                    $game,
                    Enemy_Kraid[$enemy].kraid_var_A = (($next_state) as u16)
                );
                $crate::ram_map::slot!(
                    $game,
                    Enemy_Kraid[$enemy].kraid_var_B = addr_stru_A796DA as u16
                );
                $crate::ram_map::slot!(
                    $game,
                    Enemy_Kraid[$enemy].kraid_var_C = kraid_native_record(0x96d2, $game).word(0)
                );
            }
        }
    };
}

macro_rules! advance_kraid_mouth_hit_response {
    ($enemy:ident, $game:ident) => {
        $crate::ram_map::slot!($game, Enemy_Kraid[$enemy].kraid_var_C = 90);
        let kraid_mouth_flags: u16 =
            $crate::ram_map::slot!($game, Enemy_Kraid[$enemy].kraid_mouth_flags);
        if (kraid_mouth_flags & 4) != 0 && {
            let mut v2: u16 = (kraid_mouth_flags).wrapping_sub(256);
            $crate::ram_map::slot!($game, Enemy_Kraid[$enemy].kraid_mouth_flags = v2);
            (v2 & 0xff00) != 0
        } {
            $crate::ram_map::slot!(
                $game,
                Enemy_Kraid[$enemy].kraid_var_A = ((0xa7b92di32) as u16)
            );
            $crate::ram_map::slot!($game, Enemy_Kraid[$enemy].kraid_var_F = 64);
            $crate::ram_map::slot!(
                $game,
                Enemy_Kraid[$enemy].kraid_next = ((0xa7b6bfi32) as u16)
            );
            $crate::ram_map::slot!($game, Enemy_Kraid[$enemy].field_2 = 2);
        } else {
            $crate::ram_map::slot!($game, Enemy_Kraid[$enemy].kraid_mouth_flags = 0);
        }
    };
}

pub unsafe fn Kraid_ProcessKraidInstr(game: &mut Game) -> u16 {
    unsafe {
        let E = (0) as i32;
        let result: u16 = slot!(game, Enemy_Kraid[E].kraid_var_C);
        if result != 0 {
            let fresh1 = slot!(game, Enemy_Kraid[E].kraid_var_C);
            slot!(
                game,
                Enemy_Kraid[E].kraid_var_C =
                    slot!(game, Enemy_Kraid[E].kraid_var_C).wrapping_sub(1)
            );
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
        let E = (0) as i32;
        loop {
            let kraid_var_B: u16 = slot!(game, Enemy_Kraid[E].kraid_var_B);
            let v2 = kraid_native_record(kraid_var_B, game);
            let result: u16 = v2.word(0);
            if v2.word(0) != 0xffff {
                if (v2.word(0) as i16).wrapping_add(1) < 0 {
                    CallKraidInstr(((result) as u32) | (0xa70000u32), game);
                    slot!(
                        game,
                        Enemy_Kraid[E].kraid_var_B =
                            (slot!(game, Enemy_Kraid[E].kraid_var_B)).wrapping_add(2)
                    );
                } else {
                    slot!(game, Enemy_Kraid[E].kraid_var_C = result);
                    slot!(
                        game,
                        Enemy_Kraid[E].kraid_var_B = (kraid_var_B).wrapping_add(8)
                    );
                    let v4: u16 = v2.word(2);
                    let v5: u16 = word!(game.vram_write_queue_tail);
                    let _size_tail: u16 = word!(game.vram_write_queue_tail);
                    // The two fixed tail reads have no intervening effects and select the same entry.
                    queue_native_transfer(
                        v5,
                        704,
                        |_| v4,
                        -89i32 as u8,
                        |game| ((game.ram().read_byte(0x59) as i32 & 0xfc) << 8) as u16,
                        game,
                    );
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
        let E = (0) as i32;
        if slot!(game, Enemy_Kraid[E].kraid_var_A) as i32 + 0x3ac9 & 0x8000 == 0 {
            return;
        }
        let v2 = kraid_native_record(
            (slot!(game, Enemy_Kraid[E].kraid_var_B)).wrapping_sub(8),
            game,
        );
        if v2.word(6) == 0xffff {
            v3 = 0;
        } else {
            let mut v4: u16 = v2.word(6);
            slot!(game, Enemy_Kraid[E].kraid_var_E = 1);
            v3 = 0;
            let mut v5 = kraid_native_record(v4, game);
            let mut r22: u16 = (slot!(game, Enemy_Kraid[E].base.x_pos)).wrapping_add(v5.word(0));
            let mut r20: u16 = (slot!(game, Enemy_Kraid[E].base.y_pos)).wrapping_add(v5.word(2));
            let mut r18: u16 = (slot!(game, Enemy_Kraid[E].base.y_pos)).wrapping_add(v5.word(6));
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
                                    slot!(game, Enemy_Kraid[E].kraid_mouth_flags = (slot!(game, Enemy_Kraid[E].kraid_mouth_flags)) | (1u16));
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
            slot!(game, Enemy_Kraid[E].kraid_hurt_frame = 6);
            slot!(game, Enemy_Kraid[E].kraid_hurt_frame_timer = 2);
            let kraid_mouth_flags: u16 = slot!(game, Enemy_Kraid[E].kraid_mouth_flags);
            if (kraid_mouth_flags & 2) != 0 {
                slot!(
                    game,
                    Enemy_Kraid[E].kraid_mouth_flags = (kraid_mouth_flags) | (4u16)
                );
            }
            if slot!(game, Enemy_Kraid[E].base.health) as i32 - 1i32 & 0x8000 != 0
                && slot!(game, Enemy_Kraid[E].kraid_var_A) as i32 + 0x3ca0 & 0x8000 != 0
            {
                slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7c360i32) as u16);
                slot!(game, Enemy_Kraid[E].kraid_mouth_flags = 0);
                slot!(
                    game,
                    Enemy_Kraid[E].base.properties = (slot!(game, Enemy_Kraid[E].base.properties))
                        | ((kEnemyProps_Tangible) as u16)
                );
                Kraid_SetupGfxWithTilePrioClear(!(0x2000i32) as u16, game);
                for v12 in (0u16..384).step_by(64) {
                    let EL = (v12 as i32) as i32;
                    slot!(
                        game,
                        Enemy_Kraid[EL].base.properties =
                            (slot!(game, Enemy_Kraid[EL].base.properties))
                                | ((kEnemyProps_Tangible) as u16)
                    );
                }
                if slot!(game, Enemy_Kraid[E].kraid_var_B) as i32 + 0x68f2 & 0x8000 != 0 {
                    slot!(
                        game,
                        Enemy_Kraid[E].kraid_var_B =
                            (slot!(game, Enemy_Kraid[E].kraid_var_B)).wrapping_add(60)
                    );
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
    let mut v0: u16 = 0;
    if (slot!(game, Enemy_Kraid[0i32].kraid_hurt_frame) & 1) == 0 {
        v0 = 32;
    }
    let mut v1: u16 = 0;
    do_until! { ((v1) as i16).wrapping_sub(32) >= 0 ;
        match (kraid_tables::sprite_color(game, (v0 >> 1) as usize), usize::try_from((0xc000) + (((v1 as i32 >> 1) + 240) as isize) * 2).expect("native word outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        v1 = (v1).wrapping_add(2);
        v0 = (v0).wrapping_add(2);
    }
}
pub unsafe fn Kraid_HealthBasedPaletteHandling(game: &mut Game) {
    let mut v0: u16 = 0;
    let E = (0) as i32;
    if (slot!(game, Enemy_Kraid[E].kraid_hurt_frame) & 1) == 0 {
        let mut v2: u16 = 14;
        let health: u16 = slot!(game, Enemy_Kraid[E].base.health);
        while ((health) as i16).wrapping_sub(
            ({
                let at =
                    slot!(@at Enemy_Kraid, v2 as i32, kraid_healths_8ths) + (0i32 as usize) * 2;
                u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
            }) as i16,
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
            palette_buffer[(v6 + 112)] = kraid_tables::bg_color(game, v5 as usize);
            palette_buffer[(v6 + 240)] = kraid_tables::sprite_color(game, v5 as usize);
        }
        v0 = (v0).wrapping_add(2);
    }
}
pub fn Kraid_Instr_1(_k: u16, jp: EnemyScript) -> EnemyScript {
    jp
}
pub unsafe fn KraidLintCommon_Main(k: u16, game: &mut Game) {
    unsafe {
        actors::kraid_lint::lint_touch_samus(k, game);
        let v1 = (k as i32) as i32;
        CallKraidFunc(
            ((slot!(game, EnemyData[v1].ai_var_A)) as u32) | (0xa70000u32),
            game,
        );
    }
}
pub unsafe fn KraidLint_ProduceLint(k: u16, game: &mut Game) {
    let v1 = (k as i32) as i32;
    slot!(
        game,
        EnemyData[v1].properties = (slot!(game, EnemyData[v1].properties)) & (0xfaffu16)
    );
    slot!(
        game,
        EnemyData[v1].x_pos = (slot!(game, EnemyData[v1].ai_var_C) as i32
            + slot!(game, EnemyData[0i32].x_pos) as i32
            - slot!(game, EnemyData[v1].ai_var_B) as i32) as u16
    );
    let v2: u16 = (slot!(game, EnemyData[v1].ai_var_B)).wrapping_add(1);
    slot!(game, EnemyData[v1].ai_var_B = v2);
    if v2 as i32 - 32i32 & 0x8000 == 0 {
        slot!(game, EnemyData[v1].ai_var_A = (0xa7b868i32) as u16);
        slot!(game, EnemyData[v1].ai_preinstr = 30);
    }
}
pub unsafe fn KraidLint_ChargeLint(k: u16, game: &mut Game) {
    let E = (k as i32) as i32;
    let mut v1: i16 = 0;
    if (slot!(game, EnemyData[E].ai_preinstr) & 1) != 0 {
        v1 = 3584;
    }
    slot!(game, EnemyData[E].palette_index = v1 as u16);
    slot!(
        game,
        EnemyData[E].x_pos = (slot!(game, EnemyData[E].ai_var_C) as i32
            + slot!(game, EnemyData[0i32].x_pos) as i32
            - slot!(game, EnemyData[E].ai_var_B) as i32) as u16
    );
    let fresh5 = slot!(game, EnemyData[E].ai_preinstr);
    slot!(
        game,
        EnemyData[E].ai_preinstr = slot!(game, EnemyData[E].ai_preinstr).wrapping_sub(1)
    );
    if fresh5 == 1 {
        slot!(game, EnemyData[E].ai_var_A = (0xa7b89bi32) as u16);
        QueueSfx3_Max6(0x1f, game);
    }
}
pub unsafe fn KraidLint_FireLint(k: u16, game: &mut Game) {
    let E = (k as i32) as i32;
    slot!(
        game,
        Enemy_Kraid[E].base(x_pos, x_subpos) +=
            -(words!(i32, (g_word_A7A928, g_word_A7A926))) as u32
    );
    if slot!(game, Enemy_Kraid[E].base.x_pos) as i32 - 56i32 & 0x8000 != 0 {
        slot!(
            game,
            Enemy_Kraid[E].base.properties =
                (slot!(game, Enemy_Kraid[E].base.properties)) | ((kEnemyProps_Tangible) as u16)
        );
    }
    if slot!(game, Enemy_Kraid[E].base.x_pos) as i32 - 32i32 & 0x8000 != 0 {
        slot!(
            game,
            Enemy_Kraid[E].base.properties =
                (slot!(game, Enemy_Kraid[E].base.properties)) | ((kEnemyProps_Invisible) as u16)
        );
        slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7b923i32) as u16);
        slot!(game, Enemy_Kraid[E].kraid_var_F = 300);
        slot!(game, Enemy_Kraid[E].kraid_next = (0xa7b832i32) as u16);
        slot!(game, Enemy_Kraid[E].kraid_var_B = 0);
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
            extra_samus_x_subdisplacement += -(words!(i32, (g_word_A7A928, g_word_A7A926))) as u32
        );
        if word!(game.extra_samus_x_displacement) as i32 + 16 & 0x8000 != 0 {
            word!(game.extra_samus_x_displacement = -16i32 as u16);
        }
    }
}
pub unsafe fn KraidFingernail_WaitForLintXpos(k: u16, game: &mut Game) {
    let E2 = (0x80) as i32;
    if slot!(game, Enemy_Kraid[E2].base.x_pos) as i32 - 256i32 & 0x8000 == 0 {
        let E = (k as i32) as i32;
        slot!(
            game,
            Enemy_Kraid[E].kraid_var_A = slot!(game, Enemy_Kraid[E].kraid_next)
        );
        slot!(
            game,
            Enemy_Kraid[E].base.properties =
                (slot!(game, Enemy_Kraid[E].base.properties)) & (0xfaffu16)
        );
    }
}
pub unsafe fn KraidEnemy_HandleFunctionTimer(k: u16, game: &mut Game) {
    let E = (k as i32) as i32;
    if slot!(game, Enemy_Kraid[E].kraid_var_F) != 0 && {
        let fresh6 = slot!(game, Enemy_Kraid[E].kraid_var_F);
        slot!(
            game,
            Enemy_Kraid[E].kraid_var_F = slot!(game, Enemy_Kraid[E].kraid_var_F).wrapping_sub(1)
        );
        fresh6 == 1
    } {
        slot!(
            game,
            Enemy_Kraid[E].kraid_var_A = slot!(game, Enemy_Kraid[E].kraid_next)
        );
    }
}
pub unsafe fn Kraid_AlignEnemyToKraid(k: u16, game: &mut Game) {
    unsafe {
        let x_pos: u16 = slot!(game, Enemy_Kraid[0i32].base.x_pos);
        let E = (k as i32) as i32;
        slot!(
            game,
            Enemy_Kraid[E].base.x_pos =
                (x_pos).wrapping_sub(slot!(game, Enemy_Kraid[E].base.x_width))
        );
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
pub unsafe fn Kraid_SetWalkingBackwards(j: u16, a: u16, game: &mut Game) {
    slot!(game, Enemy_Kraid[0i32].kraid_target_x = a);
    let E = (0x140) as i32;
    slot!(game, Enemy_Kraid[E].kraid_next = j);
    slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7bb45i32) as u16);
    slot!(game, Enemy_Kraid[E].base; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_8887 as u16);
}
pub unsafe fn Kraid_SetWalkingForwards(j: u16, a: u16, game: &mut Game) {
    slot!(game, Enemy_Kraid[0i32].kraid_target_x = a);
    let E = (0x140) as i32;
    slot!(game, Enemy_Kraid[E].kraid_next = j);
    slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7bbaei32) as u16);
    slot!(game, Enemy_Kraid[E].base; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_86F3 as u16);
}
pub unsafe fn KraidsFingernail_Fire(k: u16, game: &mut Game) {
    let v1 = (k as i32) as i32;
    if {
        let (actor, amount) = (
            k,
            (words!((
                slot!(game, EnemyData[v1].ai_var_C),
                slot!(game, EnemyData[v1].ai_var_B)
            ))) as i32,
        );
        (actor_api::move_horizontal(
            &mut Runtime::new(game),
            Key::from_legacy(actor),
            amount,
            actor_api::SlopePolicy::Ignore,
        ) == actor_api::MovementResult::Blocked) as u8
    } != 0
    {
        slot!(
            game,
            EnemyData[v1].ai_var_B = -(slot!(game, EnemyData[v1].ai_var_B) as i32) as u16
        );
        slot!(
            game,
            EnemyData[v1].ai_var_C = -(slot!(game, EnemyData[v1].ai_var_C) as i32) as u16
        );
    } else {
        let mut v3 = (0) as i32;
        let mut v4 = (0) as i32;
        let mut i: i32 = 0;
        loop {
            v3 = (0) as i32;
            v4 = (k as i32) as i32;
            if ((kraid_tables::fingernail_boundary(game, (i >> 2) as usize).top as i32
                + slot!(game, EnemyData[v3].y_pos) as i32
                - slot!(game, EnemyData[v4].y_pos) as i32) as i16)
                < 0
            {
                break;
            }
            i += 4;
        }
        let r18: u16 = (kraid_tables::fingernail_boundary(game, (i >> 2) as usize).left)
            .wrapping_add(slot!(game, EnemyData[v3].x_pos));
        if slot!(game, EnemyData[v4].x_width) as i32 + slot!(game, EnemyData[v4].x_pos) as i32
            - r18 as i32
            & 0x8000
            == 0
            && (slot!(game, EnemyData[v4].ai_var_C) & 0x8000) == 0
        {
            slot!(
                game,
                EnemyData[v4].ai_var_B = -(slot!(game, EnemyData[v4].ai_var_B) as i32) as u16
            );
            slot!(
                game,
                EnemyData[v4].ai_var_C = -(slot!(game, EnemyData[v4].ai_var_C) as i32) as u16
            );
        }
    }
    let v5 = (k as i32) as i32;
    if {
        let (actor, amount) = (
            k,
            (words!((
                slot!(game, EnemyData[v5].ai_var_E),
                slot!(game, EnemyData[v5].ai_var_D)
            ))) as i32,
        );
        (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount)
            == actor_api::MovementResult::Blocked) as u8
    } != 0
    {
        slot!(
            game,
            EnemyData[v5](ai_var_E, ai_var_D) = -(words!(
                i32,
                (
                    slot!(game, EnemyData[v5].ai_var_E),
                    slot!(game, EnemyData[v5].ai_var_D)
                )
            )) as u32
        );
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
            ((16 * (game.ram().read_byte(0x5d) as i32 & 0xf) + 63) as u8 as u16) << 8,
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
            ((game.ram().read_byte(0x5d)).wrapping_add(62) as u16) << 8,
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
            ((game.ram().read_byte(0x59) & 0xfc) as u16) << 8,
            0x800,
        )
        .load(game);
        crate::game::display_refresh::set_blank(false, game);
    }
}
pub unsafe fn Kraid_UnpauseHook_IsSinking(game: &mut Game) -> CoroutineRet {
    unsafe {
        's_100: {
            let mut queue_transfer = match game.ram().read_byte(0x613) as i32 {
                0 => {
                    crate::game::display_refresh::set_blank(true, game);
                    GraphicsAsset::VramWords(
                        0x7e5000,
                        ((game.ram().read_byte(0x5d)).wrapping_add(62) as u16) << 8,
                        0x400,
                    )
                    .load(game);
                    if ((slot!(game, Enemy_Kraid[0i32].base.y_pos)) as i16)
                        .wrapping_sub(actors::kraid_sinking::row(0, game).word(0) as i16)
                        >= 0
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
                    let entry =
                        actors::kraid_sinking::row(usize::from(word!(game.my_counter)), game);
                    if !(entry.word(0) != 0xffff
                        && ((slot!(game, Enemy_Kraid[0i32].base.y_pos)) as i16)
                            .wrapping_sub(entry.word(0) as i16)
                            >= 0)
                    {
                        break 's_100;
                    }
                    actors::kraid_sinking::queue_tiles(
                        word!(game.my_counter2),
                        entry.word(2),
                        game,
                    );
                }
                let mut _coret_: u8 = crate::game::display_refresh::wait(true, game);
                if _coret_ != 0 {
                    game.ram().write_byte(0x613, 1);
                    return _coret_;
                }
                word!(game.my_counter += 1);
                queue_transfer = true;
            }
        }
        game.ram().write_byte(0x613, 0);
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
    if let Some(callback) = actors::kraid_sinking::sink_row(game) {
        unsafe { CallKraidSinkTableFunc(callback, game) };
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
    let v0: u16 = word!(game.vram_write_queue_tail);
    let mut v1 = (0xd0 + (word!(game.vram_write_queue_tail) as i32)) as i32;
    match (
        2048,
        usize::try_from((v1) as isize + offset_of!(VramWriteEntry, size) as isize)
            .expect("VRAM entry outside RAM"),
    ) {
        (value, at) => {
            let bytes = u16::to_ne_bytes(value);
            game.ram().write_byte(at, bytes[0]);
            game.ram().write_byte(at + 1, bytes[1]);
        }
    };
    match (
        0x2000 as VoidP,
        usize::try_from((v1) as isize + offset_of!(VramWriteEntry, src.addr) as isize)
            .expect("VRAM entry outside RAM"),
    ) {
        (value, at) => {
            let bytes = u16::to_ne_bytes(value);
            game.ram().write_byte(at, bytes[0]);
            game.ram().write_byte(at + 1, bytes[1]);
        }
    };
    match (
        126,
        usize::try_from((v1) as isize + offset_of!(VramWriteEntry, src.bank) as isize)
            .expect("VRAM entry outside RAM"),
    ) {
        (value, at) => game.ram().write_byte(at, value),
    };
    match (
        ((game.ram().read_byte(0x59) as i32 & 0xfc) << 8) as u16,
        usize::try_from((v1) as isize + offset_of!(VramWriteEntry, vram_dst) as isize)
            .expect("VRAM entry outside RAM"),
    ) {
        (value, at) => {
            let bytes = u16::to_ne_bytes(value);
            game.ram().write_byte(at, bytes[0]);
            game.ram().write_byte(at + 1, bytes[1]);
        }
    };
    word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
}
pub unsafe fn Kraid_UpdateBG2TilemapBottomHalf(game: &mut Game) {
    let v0: u16 = word!(game.vram_write_queue_tail);
    let mut v1 = (0xd0 + (word!(game.vram_write_queue_tail) as i32)) as i32;
    match (
        2048,
        usize::try_from((v1) as isize + offset_of!(VramWriteEntry, size) as isize)
            .expect("VRAM entry outside RAM"),
    ) {
        (value, at) => {
            let bytes = u16::to_ne_bytes(value);
            game.ram().write_byte(at, bytes[0]);
            game.ram().write_byte(at + 1, bytes[1]);
        }
    };
    match (
        10240 as VoidP,
        usize::try_from((v1) as isize + offset_of!(VramWriteEntry, src.addr) as isize)
            .expect("VRAM entry outside RAM"),
    ) {
        (value, at) => {
            let bytes = u16::to_ne_bytes(value);
            game.ram().write_byte(at, bytes[0]);
            game.ram().write_byte(at + 1, bytes[1]);
        }
    };
    match (
        126,
        usize::try_from((v1) as isize + offset_of!(VramWriteEntry, src.bank) as isize)
            .expect("VRAM entry outside RAM"),
    ) {
        (value, at) => game.ram().write_byte(at, value),
    };
    match (
        (((game.ram().read_byte(0x59) as i32 & 0xfc) << 8) + 2048) as u16,
        usize::try_from((v1) as isize + offset_of!(VramWriteEntry, vram_dst) as isize)
            .expect("VRAM entry outside RAM"),
    ) {
        (value, at) => {
            let bytes = u16::to_ne_bytes(value);
            game.ram().write_byte(at, bytes[0]);
            game.ram().write_byte(at + 1, bytes[1]);
        }
    };
    word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
}
pub unsafe fn Kraid_RaiseKraidThroughFloor(_k: u16, game: &mut Game) {
    unsafe {
        Kraid_RestrictSamusXtoFirstScreen_2(game);
        slot!(game, Enemy_Kraid[0i32].kraid_var_A = (0xa7c89ai32) as u16);
        Kraid_UpdateBg2TilemapTopHalf(game);
    }
}
macro_rules! spawn_kraid_quake_projectiles_until_timer {
    ($game:ident, $interval_mask:literal, $next_state:literal, $next_timer:literal) => {
        Kraid_RestrictSamusXtoFirstScreen_2($game);
        let E = (0) as i32;
        let v1: u16 = (slot!($game, Enemy_Kraid[E].kraid_var_F)).wrapping_sub(1);
        slot!($game, Enemy_Kraid[E].kraid_var_F = v1);
        if v1 != 0 {
            if (v1 & $interval_mask) == 0 {
                Kraid_SpawnRandomQuakeProjs($game);
            }
        } else {
            slot!($game, Enemy_Kraid[E].kraid_var_A = (($next_state) as u16));
            slot!($game, Enemy_Kraid[E].kraid_var_F = $next_timer);
        }
    };
}

pub unsafe fn Kraid_SpawnRandomQuakeProjs(game: &mut Game) {
    let mut v0: u16 = (word!(game.random_number)) & (0x3fu16);
    if (word!(game.random_number) & 2) == 0 {
        v0 = !(v0 as i32) as u16;
    }
    let x: u16 = (slot!(game, Enemy_Kraid[0i32].base.x_pos)).wrapping_add(v0);
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
        let E = (0) as i32;
        if (slot!(game, Enemy_Kraid[E].base.y_pos) & 2) != 0 {
            v0 = -1i32 as u16;
        }
        slot!(game, Enemy_Kraid[E].base.x_pos = (slot!(game, Enemy_Kraid[E].base.x_pos)).wrapping_add(v0));
        slot!(game, Enemy_Kraid[E].base.y_pos = slot!(game, Enemy_Kraid[E].base.y_pos).wrapping_sub(1));
        if !((slot!(game, Enemy_Kraid[E].base.y_pos) & 3) != 0 || {
            kraid_var_F = slot!(game, Enemy_Kraid[E].kraid_var_F);
            ((kraid_var_F) as i16).wrapping_sub(18) >= 0
        }) {
            SpawnEprojWithGfx(
                crate::rtl::rom_table_word!(game, 0xa7acb3, (kraid_var_F >> 1) as isize),
                word!(game.cur_enemy_index),
                addr_kEproj_RocksFallingKraidCeiling as u16,
                game,
            );
            match slot!(game, Enemy_Kraid[E].kraid_var_F) as i32 >> 1 {
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
            slot!(game, Enemy_Kraid[E].kraid_var_F = (slot!(game, Enemy_Kraid[E].kraid_var_F)).wrapping_add(2));
        }
        if slot!(game, Enemy_Kraid[E].base.y_pos) as i32 - 296i32 & 0x8000 != 0 {
            slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7ad3ai32) as u16);
        }
    };
    [10988858] Kraid_GetsBig_SetBG2TilemapPrioBits [] {
        for v0 in (0u16..4096).step_by(2) {
            word!(game.tilemap_stuff[(v0 >> 1)] |= (0x2000u16));
        }
        let E = (0x40) as i32;
        slot!(game, Enemy_Kraid[E].base.properties = slot!(game, Enemy_Kraid[E].base.properties) & !(0x400));
        slot!(game, Enemy_Kraid[0i32].kraid_var_A = (0xa7ad61i32) as u16);
        Kraid_UpdateBg2TilemapTopHalf(game);
    };
    [10988897] Kraid_GetsBig_FinishUpdateBg2Tilemap [] {
        slot!(game, Enemy_Kraid[0i32].kraid_var_A = (0xa7ad8ei32) as u16);
        let E5 = (0x140) as i32;
        slot!(game, Enemy_Kraid[E5].base; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_86ED as u16);
        let E2 = (0x80) as i32;
        slot!(game, Enemy_Kraid[E2].base.current_instruction = addr_kKraid_Ilist_8B04 as u16);
        let E3 = (0xc0) as i32;
        slot!(game, Enemy_Kraid[E3].base.current_instruction = addr_kKraid_Ilist_8B04 as u16);
        let E = (0x100) as i32;
        slot!(game, Enemy_Kraid[E].base.current_instruction = addr_kKraid_Ilist_8B04 as u16);
        slot!(game, Enemy_Kraid[E2].base.spritemap_pointer = addr_kKraid_Sprmap_8C6C as u16);
        slot!(game, Enemy_Kraid[E3].base.spritemap_pointer = addr_kKraid_Sprmap_8C6C as u16);
        slot!(game, Enemy_Kraid[E].base.spritemap_pointer = addr_kKraid_Sprmap_8C6C as u16);
        Kraid_UpdateBG2TilemapBottomHalf(game);
    };
    [10989091] Kraid_GetsBig_FadeInRoomBg [] {
        if background6(game) & 1 != 0 {
            Kraid_GetsBig_Thinking_Setup(game);
            Kraid_Lints_Enable(0x80, g_word_A7A916, game);
            Kraid_Lints_Enable(0xc0, g_word_A7A918, game);
            Kraid_Lints_Enable(0x100, g_word_A7A91A, game);
            let E6 = (0x180) as i32;
            slot!(game, Enemy_Kraid[E6].kraid_next = (0xa7bd60i32) as u16);
            let E7 = (0x1c0) as i32;
            slot!(game, Enemy_Kraid[E7].kraid_next = (0xa7bd60i32) as u16);
            slot!(game, Enemy_Kraid[E6].kraid_var_A = (0xa7b92di32) as u16);
            slot!(game, Enemy_Kraid[E7].kraid_var_A = (0xa7b92di32) as u16);
            slot!(game, Enemy_Kraid[E6].kraid_var_F = 64);
            slot!(game, Enemy_Kraid[E7].kraid_var_F = 128);
            slot!(game, Enemy_Kraid[0x40i32].kraid_var_C = 1);
            let E0 = (0) as i32;
            slot!(game, Enemy_Kraid[E0].kraid_var_B = addr_stru_A796DA as u16);
            slot!(game, Enemy_Kraid[E0].kraid_target_x = 288);
            let E5 = (0x140) as i32;
            slot!(game, Enemy_Kraid[E5].kraid_var_A = (0xa7bb6ei32) as u16);
            slot!(game, Enemy_Kraid[E5].base; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_8887 as u16);
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
            slot!(game, Enemy_Kraid[0i32].kraid_var_A = (0xa7b73di32) as u16);
        }
    };
    [10991935] KraidEnemy_DecrementEnemyFunctionTimer [] {
        let E = (word!(game.cur_enemy_index) as i32) as i32;
        if slot!(game, Enemy_Kraid[E].kraid_var_F) != 0 {
            let fresh8 = slot!(game, Enemy_Kraid[E].kraid_var_F); slot!(game, Enemy_Kraid[E].kraid_var_F = slot!(game, Enemy_Kraid[E].kraid_var_F).wrapping_sub(1));
            if fresh8 == 1 {
                slot!(game, Enemy_Kraid[E].kraid_var_A = slot!(game, Enemy_Kraid[E].kraid_next));
                slot!(game, Enemy_Kraid[E].base; current_instruction = addr_kKraid_Ilist_8887 as u16, instruction_timer = 1);
            }
        }
    };
    [10992174] KraidsFoot_SecondPhase_Thinking [] {
        let E5 = (0x140) as i32;
        let v1: u16 = (slot!(game, Enemy_Kraid[E5].kraid_next)).wrapping_sub(1);
        slot!(game, Enemy_Kraid[E5].kraid_next = v1);
        if v1 == 0 {
            let mut v2: u16 = 0;
            let mut E0 = (0) as i32;
            loop {
                E0 = (0) as i32;
                if slot!(game, Enemy_Kraid[E0].base.x_pos)
                    == kraid_tables::foot_target(game, (v2 >> 2) as usize).x_position
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
                (kraid_tables::foot_target(game, (v2 >> 2) as usize).movement)
                    .wrapping_add((v4) as u16),
                game,
            ) as *const u16;
            let v6: u16 = *v5.offset(1);
            if ((*v5) as i16).wrapping_sub((slot!(game, Enemy_Kraid[E0].base.x_pos)) as i16) >= 0 {
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
        let mut kraid_var_E: u16 = slot!(game, Enemy_Kraid[0x180i32].kraid_var_E);
        if word!(game.cur_enemy_index) == 384 {
            kraid_var_E = slot!(game, Enemy_Kraid[0x1c0i32].kraid_var_E);
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
        let E = (word!(game.cur_enemy_index) as i32) as i32;
        slot!(game, Enemy_Kraid[E]; kraid_var_B = *(v3 as *mut u16), kraid_var_C = *(v3.offset(2) as *mut u16), kraid_var_D = *(v3.offset(4) as *mut u16), kraid_var_E = *(v3.offset(6) as *mut u16), kraid_parameter_1 = 1);
        slot!(game, Enemy_Kraid[E].base.properties = (slot!(game, Enemy_Kraid[E].base.properties) as i32
            & !(kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
            as u16);
        slot!(game, Enemy_Kraid[E].base; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_8B0A as u16);
        slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7be8ei32) as u16);
        let position_at_kraid: bool = ((word!(game.random_number) & 1) == 0)
            || ({
                let mut v7: u16 = { let at = slot!(@at Enemy_Kraid, 0x180i32, kraid_healths_8ths) + (1i32 as usize) * 2; u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]) };
                if word!(game.cur_enemy_index) != 448 {
                    v7 = { let at = slot!(@at Enemy_Kraid, 0x1c0i32, kraid_healths_8ths) + (1i32 as usize) * 2; u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]) };
                }
                (v7 == 1)
                    || ({
                        match (1, (slot!(@at Enemy_Kraid, E, kraid_healths_8ths) + (1i32 as usize) * 2)) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
                        slot!(game, Enemy_Kraid[E].base.x_pos = 50);
                        slot!(game, Enemy_Kraid[E].base.y_pos = 240);
                        slot!(game, Enemy_Kraid[E].kraid_var_B = 0);
                        slot!(game, Enemy_Kraid[E].kraid_var_C = 1);
                        slot!(game, Enemy_Kraid[E]; kraid_var_D = 0, kraid_var_E = 0);
                        slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7b907i32) as u16);
                        slot!(game, Enemy_Kraid[E].kraid_next = (0xa7be8ei32) as u16);
                        slot!(game, Enemy_Kraid[E].base.properties = (slot!(game, Enemy_Kraid[E].base.properties) as i32
                            | (kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
                            as u16);
                        false
                    })
            });
        if position_at_kraid {
            match (0, (slot!(@at Enemy_Kraid, E, kraid_healths_8ths) + (1i32 as usize) * 2)) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
            let E0 = (0) as i32;
            slot!(game, Enemy_Kraid[E].base.x_pos = (slot!(game, Enemy_Kraid[E0].base.x_pos) as i32 - slot!(game, Enemy_Kraid[E0].base.x_width) as i32 - slot!(game, Enemy_Kraid[E].base.x_width) as i32
                    & 0xfff0) as u16);
            slot!(game, Enemy_Kraid[E].base.y_pos = (slot!(game, Enemy_Kraid[0x40i32].base.y_pos)).wrapping_add(128));
        };
    };
    [10993453] KraidsFoot_PrepareToLungeForward [] {
        Kraid_HandleFirstPhase(game);
        let v0 = (0) as i32;
        if slot!(game, EnemyData[v0 + (1) * 64].current_instruction) as i32 + 0x75c9 & 0x8000 == 0 {
            slot!(game, EnemyData[v0 + (1) * 64]; current_instruction = addr_kKraid_Ilist_8AF0 as u16, instruction_timer = 1);
            slot!(game, EnemyData[v0 + (5) * 64]; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_87BD as u16);
            slot!(game, EnemyData[v0 + (5) * 64].ai_var_A = (0xa7bf5di32) as u16);
            slot!(game, EnemyData[v0 + (5) * 64].ai_preinstr = 0);
        }
    };
    [10993501] KraidsFoot_FirstPhase_LungeForward [] {
        Kraid_HandleFirstPhase(game);
        let E = (0) as i32;
        if slot!(game, Enemy_Kraid[E].base.x_pos) as i32 - 92i32 & 0x8000 != 0 {
            slot!(game, Enemy_Kraid[E].base.x_pos = 92);
        }
        let E5 = (0x140) as i32;
        if slot!(game, Enemy_Kraid[E5].base.current_instruction) == (addr_off_A78885 as u16) {
            if slot!(game, Enemy_Kraid[E].base.x_pos) == 92 {
                slot!(game, Enemy_Kraid[E5].kraid_next = (0xa7bfabi32) as u16);
                slot!(game, Enemy_Kraid[E5].kraid_var_A = (0xa7b93fi32) as u16);
                slot!(game, Enemy_Kraid[E5].kraid_var_F = 1);
                slot!(game, Enemy_Kraid[E5].base; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_86ED as u16);
            } else {
                slot!(game, Enemy_Kraid[E5].base; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_87BD as u16);
            }
        }
    };
    [10993579] KraidsFoot_FirstPhase_RetreatFromLunge [] {
        Kraid_HandleFirstPhase(game);
        let E0 = (0) as i32;
        if slot!(game, Enemy_Kraid[E0].base.x_pos) as i32 - 176i32 & 0x8000 == 0 {
            slot!(game, Enemy_Kraid[E0].base.x_pos = 176);
        }
        let E5 = (0x140) as i32;
        if slot!(game, Enemy_Kraid[E5].base.current_instruction) as i32 + 0x76c7 & 0x8000 == 0 {
            if slot!(game, Enemy_Kraid[E0].base.x_pos) == 176 {
                let E1 = (0x40) as i32;
                slot!(game, Enemy_Kraid[E1].base; current_instruction = addr_kKraid_Ilist_89F3 as u16, instruction_timer = 1);
                slot!(game, Enemy_Kraid[E5].base; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_86ED as u16);
                slot!(game, Enemy_Kraid[E5].kraid_var_A = (0xa7b960i32) as u16);
                slot!(game, Enemy_Kraid[E5].kraid_var_F = 300);
                slot!(game, Enemy_Kraid[E5].kraid_next = (0xa7bf2di32) as u16);
            } else {
                printf(b"X undefined\n\0" as *const u8 as *const c_char);
                let E = (word!(game.cur_enemy_index) as i32) as i32;
                slot!(game, Enemy_Kraid[E].base; current_instruction = addr_kKraid_Ilist_8887 as u16, instruction_timer = 1);
            }
        }
    };
    [10994681] Kraid_Death_Fadeout [] {
        Kraid_ProcessKraidInstr(game);
        if background6(game) & 1 != 0 {
            let E = (0) as i32;
            slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7c4a4i32) as u16);
            slot!(game, Enemy_Kraid[E]; kraid_hurt_frame_timer = 1, kraid_hurt_frame = 1);
            Kraid_Palette_Handling(game);
            word_transfers!(game, (4), 128; xray(0) <- value(0), xray(1) <- value(0));
            let v2: u16 = word!(game.vram_write_queue_tail);
            let mut v3 = (0xd0 + (word!(game.vram_write_queue_tail) as i32)) as i32;
            match (512, usize::try_from((v3) as isize + offset_of!(VramWriteEntry, size) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
            match (0x4000 as VoidP, usize::try_from((v3) as isize + offset_of!(VramWriteEntry, src.addr) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
            match (126, usize::try_from((v3) as isize + offset_of!(VramWriteEntry, src.bank) as isize).expect("VRAM entry outside RAM")) { (value, at) => game.ram().write_byte(at, value) };
            match ((((game.ram().read_byte(0x5d) as i32 & 0xf) << 8) + 16128) as u16, usize::try_from((v3) as isize + offset_of!(VramWriteEntry, vram_dst) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
            word!(game.vram_write_queue_tail = (v2).wrapping_add(7));
        }
    };
    [10994852] Kraid_Death_UpdateBG2TilemapTopHalf [] {
        Kraid_ProcessKraidInstr(game);
        slot!(game, Enemy_Kraid[0i32].kraid_var_A = (0xa7c4c8i32) as u16);
        let E2 = (0x80) as i32;
        slot!(game, Enemy_Kraid[E2].kraid_var_A = (0xa7b923i32) as u16);
        slot!(game, Enemy_Kraid[E2].kraid_var_F = 0x7fff);
        let E3 = (0xc0) as i32;
        slot!(game, Enemy_Kraid[E3].kraid_var_A = (0xa7b923i32) as u16);
        slot!(game, Enemy_Kraid[E3].kraid_var_F = 0x7fff);
        let E4 = (0x100) as i32;
        slot!(game, Enemy_Kraid[E4].kraid_var_A = (0xa7b923i32) as u16);
        slot!(game, Enemy_Kraid[E4].kraid_var_F = 0x7fff);
        Kraid_UpdateBg2TilemapTopHalf(game);
    };
    [10994888] Kraid_Death_UpdateBG2TilemapBottomHalf [] {
        Kraid_ProcessKraidInstr(game);
        match (-89i32 as u8, 0x604 + offset_of!(LongPtr, bank)) { (value, at) => game.ram().write_byte(at, value) };
        match ((0xa7c2a0i32) as u16, 0x604 + offset_of!(LongPtr, addr)) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        let E = (0) as i32;
        slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7c537i32) as u16);
        word!(game.kraid_unk9000 = 43);
        slot!(game, Enemy_Kraid[E].base.properties = (slot!(game, Enemy_Kraid[E].base.properties)) | (0x8000u16));
        word! {game; earthquake_type = 1; earthquake_timer = 256; }
        let E1 = (0x40) as i32;
        slot!(game, Enemy_Kraid[E1].base; current_instruction = addr_kKraid_Ilist_8AA4 as u16, instruction_timer = 1);
        let E5 = (0x140) as i32;
        slot!(game, Enemy_Kraid[E5].base; current_instruction = addr_kKraid_Ilist_86E7 as u16, instruction_timer = 1);
        slot!(game, Enemy_Kraid[E5].kraid_var_A = (0xa7ba2di32) as u16);
        Kraid_UpdateBG2TilemapBottomHalf(game);
    };
    [10994999] Kraid_Death_SinkThroughFloor [] {
        Kraid_ProcessKraidInstr(game);
        Kraid_PlaySoundEveryHalfSecond(game);
        Kraid_HandleSinking(game);
        let E0 = (0) as i32;
        slot!(game, Enemy_Kraid[E0].base.y_pos = slot!(game, Enemy_Kraid[E0].base.y_pos).wrapping_add(1));
        if slot!(game, Enemy_Kraid[E0].base.y_pos) as i32 - 608i32 & 0x8000 == 0 {
            slot!(game, Enemy_Kraid[E0].base.properties = (slot!(game, Enemy_Kraid[E0].base.properties) as i32 & !(kEnemyProps_Tangible as i32)) as u16);
            word!(game.enemy_bg2_tilemap_size = 2);
            let enemy_ptr: u16 = slot!(game, Enemy_Kraid[word!(game.cur_enemy_index) as i32].base.enemy_ptr);
            rom_record(0xa0, enemy_ptr, game).set_word(offset_of!(EnemyDef, shot_ai), (0xa7804ci32) as u16);
            #[cfg(sm_verify_dll)]
            crate::verify_probe::hit("actors.kraid.shot_ai.disabled");
            let E1 = (0x40) as i32;
            let v3: u16 = (slot!(game, Enemy_Kraid[E1].base.properties) as i32
                | kEnemyProps_Tangible as i32
                | kEnemyProps_Deleted as i32) as u16;
            slot!(game, Enemy_Kraid[E1].base.properties = v3);
            let v4: u16 = v3 & 0x51ff | 0x600;
            slot!(game, Enemy_Kraid[0x80i32].base.properties = v4);
            slot!(game, Enemy_Kraid[0xc0i32].base.properties = v4);
            slot!(game, Enemy_Kraid[0x100i32].base.properties = v4);
            slot!(game, Enemy_Kraid[0x140i32].base.properties = v4);
            slot!(game, Enemy_Kraid[0i32].kraid_var_A = (0xa7c715i32) as u16);
            word!(game.camera_distance_index = 0);
            scatter(DropSource::Kraid, enemy_ptr, game);
            Kraid_DrawRoomBg(game);
        }
    };
    [10995477] Kraid_FadeInBg_ClearBg2TilemapTopHalf [] {
        match (72, 0x59) { (value, at) => game.ram().write_byte(at, value) };
        word_transfers!(game, descending, 1024; le(tilemap_stuff, 1023) <- value(824));
        let v1: u16 = word!(game.vram_write_queue_tail);
        let mut v2 = (0xd0 + (word!(game.vram_write_queue_tail) as i32)) as i32;
        match (1024, usize::try_from((v2) as isize + offset_of!(VramWriteEntry, size) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (0x2000 as VoidP, usize::try_from((v2) as isize + offset_of!(VramWriteEntry, src.addr) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (126, usize::try_from((v2) as isize + offset_of!(VramWriteEntry, src.bank) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (18432, usize::try_from((v2) as isize + offset_of!(VramWriteEntry, vram_dst) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        word!(game.vram_write_queue_tail = (v1).wrapping_add(7));
        slot!(game, Enemy_Kraid[0i32].kraid_var_A = (0xa7c751i32) as u16);
    };
    [10995537] Kraid_FadeInBg_ClearBg2TilemapBottomHalf [] {
        let v0: u16 = word!(game.vram_write_queue_tail);
        let mut v1 = (0xd0 + (word!(game.vram_write_queue_tail) as i32)) as i32;
        match (1024, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, size) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (0x2000 as VoidP, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, src.addr) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (126, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, src.bank) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (18944, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, vram_dst) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
        slot!(game, Enemy_Kraid[0i32].kraid_var_A = (0xa7c777i32) as u16);
    };
    [10995575] Kraid_FadeInBg_LoadBg3Tiles1of4 [] {
        match ((0xa7c1fbi32) as u16, 0x604 + offset_of!(LongPtr, addr)) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        slot!(game, Enemy_Kraid[0i32].kraid_var_A = (0xa7c7a3i32) as u16);
        let v0: u16 = word!(game.vram_write_queue_tail);
        let mut v1 = (0xd0 + (word!(game.vram_write_queue_tail) as i32)) as i32;
        match (1024, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, size) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (-19968i32 as VoidP, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, src.addr) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (154, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, src.bank) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (0x4000, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, vram_dst) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
    };
    [10995619] Kraid_FadeInBg_LoadBg3Tiles2of4 [] {
        slot!(game, Enemy_Kraid[0i32].kraid_var_A = (0xa7c7c9i32) as u16);
        let v0: u16 = word!(game.vram_write_queue_tail);
        let mut v1 = (0xd0 + (word!(game.vram_write_queue_tail) as i32)) as i32;
        match (1024, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, size) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (-18944i32 as VoidP, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, src.addr) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (154, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, src.bank) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (16896, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, vram_dst) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
    };
    [10995657] Kraid_FadeInBg_LoadBg3Tiles3of4 [] {
        slot!(game, Enemy_Kraid[0i32].kraid_var_A = (0xa7c7efi32) as u16);
        let v0: u16 = word!(game.vram_write_queue_tail);
        let mut v1 = (0xd0 + (word!(game.vram_write_queue_tail) as i32)) as i32;
        match (1024, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, size) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (-17920i32 as VoidP, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, src.addr) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (154, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, src.bank) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (17408, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, vram_dst) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        word!(game.vram_write_queue_tail = (v0).wrapping_add(7));
    };
    [10995695] Kraid_FadeInBg_LoadBg3Tiles4of4 [] {
        slot!(game, Enemy_Kraid[0i32].kraid_var_A = (0xa7c815i32) as u16);
        let v0: u16 = word!(game.vram_write_queue_tail);
        let mut v1 = (0xd0 + (word!(game.vram_write_queue_tail) as i32)) as i32;
        match (1024, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, size) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (-16896i32 as VoidP, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, src.addr) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (154, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, src.bank) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (17920, usize::try_from((v1) as isize + offset_of!(VramWriteEntry, vram_dst) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
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
                slot!(game, Enemy_Kraid[0i32].kraid_var_A = (0xa7c851i32) as u16);
            } else {
                gameplay_flag_runtime::set_boss_word_native(
                    word!(game.area_index), ((v0) as u16) | (1u16), game,
                );
                slot!(game, Enemy_Kraid[0i32].kraid_var_A = (0xa7c843i32) as u16);
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
        let E = (0) as i32;
        slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7c8e0i32) as u16);
        slot!(game, Enemy_Kraid[E].kraid_var_F = 120);
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
        let E0 = (0) as i32;
        slot!(game, Enemy_Kraid[E0].base.x_pos = (slot!(game, Enemy_Kraid[E0].base.x_pos) as i32
            + if (slot!(game, Enemy_Kraid[E0].base.y_pos) & 2) == 0 {
                -1i32
            } else {
                1
            }) as u16);
        slot!(game, Enemy_Kraid[E0].base(y_pos, y_subpos) += -0x8000i32 as u32);
        if slot!(game, Enemy_Kraid[E0].base.y_pos) as i32 - 457i32 & 0x8000 != 0 {
            slot!(game, Enemy_Kraid[E0].base.x_pos = 176);
            let E5 = (0x140) as i32;
            slot!(game, Enemy_Kraid[E5].kraid_var_A = (0xa7b960i32) as u16);
            slot!(game, Enemy_Kraid[E5].kraid_var_F = 300);
            slot!(game, Enemy_Kraid[E5].kraid_next = (0xa7bf2di32) as u16);
            slot!(game, Enemy_Kraid[E0].kraid_var_B = addr_stru_A796DA as u16);
            Kraid_Mainloop_Thinking_Setup(game);
            let E1 = (0x40) as i32;
            slot!(game, Enemy_Kraid[E1].base; current_instruction = addr_kKraid_Ilist_89F3 as u16, instruction_timer = 1);
        }
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E = (0) as i32; };
    [10988942] Kraid_GetsBig_DrawRoomBg [] {
        slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7ae23i32) as u16);
        slot!(game, Enemy_Kraid[E]; kraid_var_E = 0, kraid_var_F = 0);
        Kraid_DrawRoomBg(game);
    };
    Kraid_SetLintYAndRandomThinkTimer [] {
        slot!(game, Enemy_Kraid[0x80i32].base.y_pos = (slot!(game, Enemy_Kraid[E].base.y_pos)).wrapping_sub(20));
        slot!(game, Enemy_Kraid[0xc0i32].base.y_pos = (slot!(game, Enemy_Kraid[E].base.y_pos)).wrapping_add(46));
        slot!(game, Enemy_Kraid[0x100i32].base.y_pos = (slot!(game, Enemy_Kraid[E].base.y_pos)).wrapping_add(112));
        let mut v1: i16 = ((word!(game.random_number)) as i16) & (7i16);
        if (word!(game.random_number) & 7) == 0 {
            v1 = 2;
        }
        slot!(game, Enemy_Kraid[E].kraid_thinking = ((v1 as i32) << 6) as u16);
    };
    [10989220] Kraid_Mainloop_Thinking [] {
        advance_kraid_thinking_timer!(E, 0xa7bbeai32, game);
    };
    [10989252] Kraid_GetsBig_Thinking [] {
        advance_kraid_thinking_timer!(E, 0xa7aee4i32, game);
    };
    [10989284] Kraid_Shot_MouthIsOpen [] {
        if Kraid_ProcessKraidInstr(game) == 0xffff {
            slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7aea4i32) as u16);
            advance_kraid_mouth_hit_response!(E, game);
        }
    };
    Kraid_Enemy_Touch [] {
        if slot!(game, Enemy_Kraid[E].kraid_var_A) as i32 + 0x3ca0 & 0x8000 != 0 {
            let r18: u16 = (word!(game.samus_x_radius)).wrapping_add(word!(game.samus_x_pos));
            let mut v1: i16 = ((word!(game.samus_y_pos)) as i16).wrapping_sub((slot!(game, Enemy_Kraid[E].base.y_pos)) as i16);
            let mut i: i32 = 0;
            loop {
                let band = (i >> 2) as usize;
                if (v1).wrapping_sub(kraid_tables::body_hitbox(game, band).bottom as i16) >= 0
                    || (v1).wrapping_sub(kraid_tables::body_hitbox(game, band + 1).bottom as i16)
                        >= 0
                {
                    break;
                }
                i += 4;
            }
            if ((slot!(game, Enemy_Kraid[E].base.x_pos) as i32
                + kraid_tables::body_hitbox(game, (i >> 2) as usize).left as i32
                - r18 as i32) as i16)
                < 0
            {
                if word!(game.samus_x_pos) as i32 - 40i32 & 0x8000 == 0 {
                    word!(game.samus_x_pos -= 8);
                    word!(game.samus_prev_x_pos = word!(game.samus_x_pos));
                }
                let mut kraid_min_y_pos_eject: u16 = (word!(game.samus_y_pos)).wrapping_sub(8);
                if ((word!(game.samus_y_pos) as i32 - 8i32 - slot!(game, Enemy_Kraid[E].kraid_min_y_pos_eject) as i32) as i16)
                    < 0
                {
                    kraid_min_y_pos_eject = slot!(game, Enemy_Kraid[E].kraid_min_y_pos_eject);
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
        if slot!(game, Enemy_Kraid[E].kraid_var_A) as i32 + 0x3ac9 & 0x8000 != 0 {
            slot!(game, Enemy_Kraid[E].kraid_var_E = 0);
            slot!(game, Enemy_Kraid[E].kraid_mouth_flags = slot!(game, Enemy_Kraid[E].kraid_mouth_flags) & !(1));
            let mut R48: u16 = 0;
            let v2 = kraid_native_record((slot!(game, Enemy_Kraid[E].kraid_var_B)).wrapping_sub(8), game);
            let v3 = kraid_native_record(v2.word(4), game);
            let r22: u16 = (slot!(game, Enemy_Kraid[E].base.x_pos)).wrapping_add(v3.word(0));
            let r20: u16 = (slot!(game, Enemy_Kraid[E].base.y_pos)).wrapping_add(v3.word(2));
            let mut r18: u16 = (slot!(game, Enemy_Kraid[E].base.y_pos)).wrapping_add(v3.word(6));
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
                            .wrapping_sub((slot!(game, Enemy_Kraid[E].base.y_pos)) as i16);
                        let mut j: u16 = 0;
                        loop {
                            let band = (j >> 2) as usize;
                            if (v11).wrapping_sub(kraid_tables::body_hitbox(game, band).bottom as i16)
                                >= 0
                                || (v11).wrapping_sub(
                                    kraid_tables::body_hitbox(game, band + 1).bottom as i16,
                                ) >= 0
                            {
                                break;
                            }
                            j = (j).wrapping_add(4);
                        }
                        ((slot!(game, Enemy_Kraid[E].base.x_pos) as i32
                            + kraid_tables::body_hitbox(game, (j >> 2) as usize).left as i32
                            - r18 as i32) as i16)
                            < 0
                    };
                    if hit_body {
                        Kraid_SpawnExplosionEproj(i as u16, game);
                        let v6: i32 = i >> 1;
                        word!(game.projectile_dir[v6] |= (0x10u16));
                        if (word!(game.projectile_type[v6]) & 0x10) != 0 {
                            slot!(game, Enemy_Kraid[E].kraid_mouth_flags = (slot!(game, Enemy_Kraid[E].kraid_mouth_flags)) | (1u16));
                        }
                        R48 = R48.wrapping_add(1);
                    }
                    i -= 2;
                }
            }
            if R48 != 0 && slot!(game, Enemy_Kraid[E].kraid_var_A) as i32 == 0xa7aea4 & 0xffff {
                slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7b6bfi32) as u16);
                let kraid_mouth_flags: u16 = slot!(game, Enemy_Kraid[E].kraid_mouth_flags);
                if (kraid_mouth_flags & 1) != 0 {
                    slot!(game, Enemy_Kraid[E].kraid_mouth_flags = (kraid_mouth_flags) | (0x302u16));
                }
            }
        }
    };
    Kraid_Palette_Handling [] {
        if slot!(game, Enemy_Kraid[E].base.health) as i32 - 1i32 & 0x8000 != 0 {
            slot!(game, Enemy_Kraid[E].kraid_hurt_frame = slot!(game, Enemy_Kraid[E].base.health));
        } else {
            let continue_palette_update: bool = (slot!(game, Enemy_Kraid[E].kraid_hurt_frame) != 0)
                && ({
                    let v1: u16 = (slot!(game, Enemy_Kraid[E].kraid_hurt_frame_timer)).wrapping_sub(1);
                    slot!(game, Enemy_Kraid[E].kraid_hurt_frame_timer = v1);
                    (v1 == 0)
                        && ({
                            slot!(game, Enemy_Kraid[E].kraid_hurt_frame_timer = 2);
                            slot!(game, Enemy_Kraid[E].kraid_hurt_frame = slot!(game, Enemy_Kraid[E].kraid_hurt_frame).wrapping_sub(1));
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
        slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7aee4i32) as u16);
        slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7b6d7i32) as u16);
        slot!(game, Enemy_Kraid[E].kraid_var_B = (addr_stru_A7974A as i32 + 8) as u16);
        slot!(game, Enemy_Kraid[E].kraid_var_C = kraid_native_record(0x974a, game).word(0));
        Kraid_Shot_GlowHisEye(game);
    };
    [10991421] Kraid_Shot_UnglowEye [] {
        let mut v0: u16 = 14;
        let health: u16 = slot!(game, Enemy_Kraid[E].base.health);
        while ((health) as i16)
            .wrapping_sub(({ let at = slot!(@at Enemy_Kraid, v0 as i32, kraid_healths_8ths) + (0i32 as usize) * 2; u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]) }) as i16)
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
    if kraid_tables::bg_color(game, (v5 + 1) as usize) & 0x1f != r18 {
        r20 = r20.wrapping_add(1);
        word!(game.palette_buffer[v4] -= 1);
    }
    r18 = (word!(game.palette_buffer[v4])) & (0x3e0u16);
    if kraid_tables::bg_color(game, (v5 + 1) as usize) & 0x3e0 != r18 {
        r20 = r20.wrapping_add(1);
        word!(game.palette_buffer[v4] -= 32);
    }
    v3 = (v3).wrapping_add(2);
    v2 = (v2).wrapping_add(2);
}
        if r20 == 0 {
            slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7aee4i32) as u16);
            slot!(game, Enemy_Kraid[E].kraid_var_B = addr_stru_A796DA as u16);
            slot!(game, Enemy_Kraid[E].kraid_var_C = kraid_native_record(0x96d2, game).word(0));
        }
    };
    [10992618] Kraid_Main_AttackWithMouthOpen [] {
        if Kraid_ProcessKraidInstr(game) == 0xffff {
            Kraid_Mainloop_Thinking_Setup(game);
            advance_kraid_mouth_hit_response!(E, game);
        } else if kraid_native_record((slot!(game, Enemy_Kraid[E].kraid_var_B)).wrapping_sub(8u16), game).word(2) as i32
            == addr_kKraidTilemaps_3 as i32
            && (slot!(game, Enemy_Kraid[E].kraid_var_C) & 0xf) == 0
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
        slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7ac4di32) as u16);
        match (514, usize::try_from(0xcd20).expect("native word outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match (257, usize::try_from((0xcd20) + (2)).expect("native word outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        slot!(game, Enemy_Kraid[E].kraid_min_y_pos_eject = 164);
    };
    @functions [pub unsafe fn (game: &mut Game)];
    @prefix { let E0 = (0) as i32; };
    [10992453] KraidsFoot_SecondPhase_WalkingBackwards [] {
        let kraid_target_x: u16 = slot!(game, Enemy_Kraid[E0].kraid_target_x);
        if kraid_target_x != slot!(game, Enemy_Kraid[E0].base.x_pos) {
            if ((kraid_target_x) as i16).wrapping_sub((slot!(game, Enemy_Kraid[E0].base.x_pos)) as i16) >= 0 {
                return;
            }
            slot!(game, Enemy_Kraid[E0].base.x_pos = kraid_target_x);
        }
        let E5 = (0x140) as i32;
        if slot!(game, Enemy_Kraid[E5].base.current_instruction) as i32 + 0x76c7 & 0x8000 == 0 {
            slot!(game, Enemy_Kraid[E5].kraid_var_A = (0xa7ba2ei32) as u16);
            slot!(game, Enemy_Kraid[E5].base; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_86ED as u16);
        }
    };
    [10992494] KraidsFoot_SecondPhaseSetup_WalkToStartPt [] {
        let kraid_target_x: u16 = slot!(game, Enemy_Kraid[E0].kraid_target_x);
        if kraid_target_x != slot!(game, Enemy_Kraid[E0].base.x_pos) {
            if ((kraid_target_x) as i16).wrapping_sub((slot!(game, Enemy_Kraid[E0].base.x_pos)) as i16) >= 0 {
                return;
            }
            slot!(game, Enemy_Kraid[E0].base.x_pos = kraid_target_x);
        }
        let E5 = (0x140) as i32;
        if slot!(game, Enemy_Kraid[E5].base.current_instruction) as i32 + 0x76c7 & 0x8000 == 0 {
            slot!(game, Enemy_Kraid[E5].kraid_var_A = (0xa7b92di32) as u16);
            slot!(game, Enemy_Kraid[E5].kraid_var_F = 180);
            slot!(game, Enemy_Kraid[E5].kraid_next = (0xa7bba4i32) as u16);
            slot!(game, Enemy_Kraid[E5].base; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_86ED as u16);
        }
    };
    [10992558] KraidsFoot_SecondPhase_WalkForward [] {
        let kraid_target_x: u16 = slot!(game, Enemy_Kraid[E0].kraid_target_x);
        if ((kraid_target_x) as i16).wrapping_sub((slot!(game, Enemy_Kraid[E0].base.x_pos)) as i16) < 0 {
            let E5 = (0x140) as i32;
            if slot!(game, Enemy_Kraid[E5].base.current_instruction) == (addr_off_A787BB as u16) {
                slot!(game, Enemy_Kraid[E5].base; current_instruction = addr_kKraid_Ilist_86F3 as u16, instruction_timer = 1);
            }
        } else {
            slot!(game, Enemy_Kraid[E0].base.x_pos = kraid_target_x);
            let E5_0 = (0x140) as i32;
            if slot!(game, Enemy_Kraid[E5_0].base.current_instruction) == (addr_off_A787BB as u16) {
                slot!(game, Enemy_Kraid[E5_0].kraid_var_A = (0xa7ba2ei32) as u16);
                slot!(game, Enemy_Kraid[E5_0].base; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_86ED as u16);
            }
        };
    };
    Kraid_HandleFirstPhase [] {
        if ((slot!(game, Enemy_Kraid[E0].base.health)) as i16).wrapping_sub(({ let at = slot!(@at Enemy_Kraid, E0, kraid_healths_8ths) + (6i32 as usize) * 2; u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]) }) as i16)
            < 0
        {
            slot!(game, Enemy_Kraid[E0].kraid_var_A = (0xa7b965i32) as u16);
            slot!(game, Enemy_Kraid[E0].kraid_var_F = 180);
            slot!(game, Enemy_Kraid[E0].kraid_next = (0xa7c0a1i32) as u16);
            let v2: u16 = kraid_native_record(slot!(game, Enemy_Kraid[E0].kraid_var_B), game).word(2);
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
            slot!(game, Enemy_Kraid[E0].kraid_var_B = (v3).wrapping_sub(26918));
            slot!(game, Enemy_Kraid[E0].kraid_var_C = kraid_native_record(0x96d2, game).word(usize::from(v3)));
            word! {game; earthquake_type = 4; earthquake_timer = 340; }
            let E5 = (0x140) as i32;
            slot!(game, Enemy_Kraid[E5].base; current_instruction = addr_kKraid_Ilist_86E7 as u16, instruction_timer = 1);
            slot!(game, Enemy_Kraid[E5].kraid_var_A = (0xa7ba2di32) as u16);
            let E1 = (0x40) as i32;
            slot!(game, Enemy_Kraid[E1].base; current_instruction = addr_kKraid_Ilist_89F3 as u16, instruction_timer = 1);
            let E2 = (0x80) as i32;
            slot!(game, Enemy_Kraid[E2].base.properties = (slot!(game, Enemy_Kraid[E2].base.properties)) | (0x100u16));
            let E3 = (0xc0) as i32;
            slot!(game, Enemy_Kraid[E3].base.properties = (slot!(game, Enemy_Kraid[E3].base.properties)) | (0x100u16));
            let E4 = (0x100) as i32;
            slot!(game, Enemy_Kraid[E4].base.properties = (slot!(game, Enemy_Kraid[E4].base.properties)) | (0x100u16));
            slot!(game, Enemy_Kraid[E1].base.properties = (slot!(game, Enemy_Kraid[E1].base.properties)) | (0x400u16));
        }
    };
    [10994528] Kraid_Death_Init [] {
        if slot!(game, Enemy_Kraid[E0].kraid_hurt_frame) == 0 {
            word_transfers!(game, ascending, 16; le(target_palettes, 96) <- value(0));
            word_transfers!(game, descending, 16; le(palette_buffer, 127) <- rom(0xa7b4f3, 15));
            let E1 = (0x40) as i32;
            slot!(game, Enemy_Kraid[E1].base; current_instruction = addr_kKraid_Ilist_8AF0 as u16, instruction_timer = 1);
            slot!(game, Enemy_Kraid[E0].kraid_var_A = (0xa7c3f9i32) as u16);
            slot!(game, Enemy_Kraid[E0].kraid_var_B = (addr_stru_A79764 as i32 + 8) as u16);
            slot!(game, Enemy_Kraid[E0].kraid_var_C = kraid_native_record(0x9764, game).word(0));
            let v4: u16 = word!(game.cur_enemy_index);
            let v7: u16 = word!(game.cur_enemy_index);
            let E6 = (0x180) as i32;
            slot!(game, Enemy_Kraid[E6].base.properties = slot!(game, Enemy_Kraid[E6].base.properties) & !(0x4000));
            word!(game.cur_enemy_index = 384);
            ({ let (_ignored_key, animation) = (v4, 0x180); actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Normal { animation }) });
            let E7 = (0x1c0) as i32;
            slot!(game, Enemy_Kraid[E7].base.properties = slot!(game, Enemy_Kraid[E7].base.properties) & !(0x4000));
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
pub unsafe fn Phantoon_MoveInSwoopingPattern(k: u16, game: &mut Game) {
    let mut v4: u16 = 0;
    let E2 = (0x80) as i32;
    let mut phant_var_E: i16 = slot!(game, Enemy_Phantoon[E2].var_E) as i16;
    if phant_var_E < 0 {
        let mut v6: u16 = ((phant_var_E) as u16).wrapping_sub(2);
        slot!(game, Enemy_Phantoon[E2].var_E = v6);
        v4 = (v6) & (0x7fffu16);
        if v4 == 0 {
            v4 = 0;
            slot!(game, Enemy_Phantoon[E2].var_E = 0);
        }
    } else {
        v4 = ((phant_var_E) as u16).wrapping_add(2);
        slot!(game, Enemy_Phantoon[E2].var_E = v4);
        if v4 as i32 - 256i32 & 0x8000 == 0 {
            let mut v5: u16 = (v4) | (0x8000u16);
            slot!(game, Enemy_Phantoon[E2].var_E = v5);
            v4 = (v5) & (0x7fffu16);
        }
    }
    let E0 = (0) as i32;
    if ((v4) as i16).wrapping_sub((slot!(game, Enemy_Phantoon[E0].base.x_pos)) as i16) < 0 {
        if slot!(game, Enemy_Phantoon[E2].var_C) as i32 + 2047 & 0x8000 == 0 {
            slot!(
                game,
                Enemy_Phantoon[E2].var_C = (slot!(game, Enemy_Phantoon[E2].var_C)).wrapping_sub(32)
            );
        }
    } else if slot!(game, Enemy_Phantoon[E2].var_C) as i32 - 2048i32 & 0x8000 != 0 {
        slot!(
            game,
            Enemy_Phantoon[E2].var_C = (slot!(game, Enemy_Phantoon[E2].var_C)).wrapping_add(32)
        );
    }
    slot!(
        game,
        Enemy_Phantoon[E0].base(x_pos, x_subpos) +=
            ((slot!(game, Enemy_Phantoon[E2].var_C) as i16 as i32) << 8) as u32
    );
    if slot!(game, Enemy_Phantoon[E0].base.x_pos) as i32 + 64 & 0x8000 != 0 {
        slot!(game, Enemy_Phantoon[E0].base.x_pos = -64i32 as u16);
    } else if slot!(game, Enemy_Phantoon[E0].base.x_pos) as i32 - 448i32 & 0x8000 == 0 {
        slot!(game, Enemy_Phantoon[E0].base.x_pos = 448);
    }
    let mut v13: i16 = if slot!(game, Enemy_Phantoon[k as i32].var_F) as i32 == 0xa7d92e & 0xffff {
        112
    } else {
        ((word!(game.samus_y_pos)) as i16).wrapping_sub(48)
    };
    if (v13).wrapping_sub((slot!(game, Enemy_Phantoon[E0].base.y_pos)) as i16) < 0 {
        if slot!(game, Enemy_Phantoon[E2].var_D) as i32 + 1535 & 0x8000 == 0 {
            slot!(
                game,
                Enemy_Phantoon[E2].var_D = (slot!(game, Enemy_Phantoon[E2].var_D)).wrapping_sub(64)
            );
        }
    } else if slot!(game, Enemy_Phantoon[E2].var_D) as i32 - 1536i32 & 0x8000 != 0 {
        slot!(
            game,
            Enemy_Phantoon[E2].var_D = (slot!(game, Enemy_Phantoon[E2].var_D)).wrapping_add(64)
        );
    }
    slot!(
        game,
        Enemy_Phantoon[E0].base(y_pos, y_subpos) +=
            ((slot!(game, Enemy_Phantoon[E2].var_D) as i16 as i32) << 8) as u32
    );
    if slot!(game, Enemy_Phantoon[E0].base.y_pos) as i32 - 64i32 & 0x8000 != 0 {
        slot!(game, Enemy_Phantoon[E0].base.y_pos = 64);
    } else if slot!(game, Enemy_Phantoon[E0].base.y_pos) as i32 - 216i32 & 0x8000 == 0 {
        slot!(game, Enemy_Phantoon[E0].base.y_pos = 216);
    }
}
pub unsafe fn Phantoon_BeginSwoopingPattern(k: u16, game: &mut Game) {
    let E2 = (0x80) as i32;
    slot!(game, Enemy_Phantoon[E2]; var_C = 1024, var_D = 1024);
    slot!(game, Enemy_Phantoon[E2].var_E = 0);
    let E = (k as i32) as i32;
    slot!(game, Enemy_Phantoon[E].var_F = (0xa7d678i32) as u16);
    slot!(game, Enemy_Phantoon[E].var_E = 360);
}
pub unsafe fn Phantoon_ChangeEyeSpriteBasedOnSamusDist(game: &mut Game) {
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
    let E = (0x40) as i32;
    slot!(game, Enemy_Phantoon[E].base; instruction_timer = 1, current_instruction = crate::rtl::rom_table_word!(game, 0xa7d40d, (v0 >> 1) as isize));
}
pub unsafe fn Phantoon_StartDeathSequence(k: u16, game: &mut Game) {
    unsafe {
        let E = (k as i32) as i32;
        if slot!(game, Enemy_Phantoon[E].var_F) as i32 == 0xa7d678 & 0xffff
            || slot!(game, Enemy_Phantoon[E].var_F) as i32 == 0xa7d6b9 & 0xffff
        {
            slot!(game, Enemy_Phantoon[E].var_F = (0xa7d92ei32) as u16);
        } else {
            slot!(game, Enemy_Phantoon[E].var_F = (0xa7d948i32) as u16);
        }
        let E1 = (0x40) as i32;
        slot!(game, Enemy_Phantoon[E1]; var_C = 0, var_F = 0);
        Phantoon_ChangeEyeSpriteBasedOnSamusDist(game);
        let v4: u16 = word!(game.reg_BG2HOFS);
        for v3 in (0i16..=510).rev().step_by(2) {
            match (
                v4,
                usize::try_from((0x9100) + (v3 as u16 as i32 as isize))
                    .expect("native word outside RAM"),
            ) {
                (value, at) => {
                    let bytes = u16::to_ne_bytes(value);
                    game.ram().write_byte(at, bytes[0]);
                    game.ram().write_byte(at + 1, bytes[1]);
                }
            };
        }
        word!(game.phantom_related_layer_flag |= (0x4000u16));
        slot!(game, Enemy_Phantoon[0xc0i32].parameter_2 = 1);
    }
}
pub unsafe fn Phantoon_FadeOut(a: u16, game: &mut Game) {
    unsafe {
        if (word!(game.nmi_frame_counter_word) & 1) == 0 {
            let E = (0x40) as i32;
            if slot!(game, Enemy_Phantoon[E].var_F) == 0 {
                slot!(game, Enemy_Phantoon[E].var_D = a);
                if Phantoon_Func_8(game) & 1 != 0 {
                    slot!(game, Enemy_Phantoon[E].var_F = 1);
                }
            }
        }
    }
}
pub unsafe fn Phantoon_FadeIn(a: u16, game: &mut Game) {
    unsafe {
        if (word!(game.nmi_frame_counter_word) & 1) == 0 {
            let E = (0x40) as i32;
            if slot!(game, Enemy_Phantoon[E].var_F) == 0 {
                slot!(game, Enemy_Phantoon[E].var_D = a);
                if Phantoon_SetColorBasedOnHp(game) & 1 != 0 {
                    slot!(game, Enemy_Phantoon[E].var_F = 1);
                }
            }
        }
    }
}
pub unsafe fn Phantoon_Func_8(game: &mut Game) -> u8 {
    unsafe {
        let E = (0x40) as i32;
        if (slot!(game, Enemy_Phantoon[E].var_D)).wrapping_add(1)
            >= slot!(game, Enemy_Phantoon[E].var_E)
        {
            for i in (0i32..32).step_by(2) {
                word!(
                    game.palette_buffer[((i >> 1) + 112)] =
                        Phantoon_Func_10_CalculateNthTransitionColorFromXtoY(
                            slot!(game, Enemy_Phantoon[E].var_E),
                            {
                                let at =
                                    usize::try_from((0xc000) + (((i >> 1) + 112) as isize) * 2)
                                        .expect("native word outside RAM");
                                u16::from_ne_bytes([
                                    game.ram().read_byte(at),
                                    game.ram().read_byte(at + 1),
                                ])
                            },
                            crate::rtl::rom_table_word!(game, 0xa7ca41, (i >> 1) as isize),
                            game,
                        )
                );
            }
            slot!(
                game,
                Enemy_Phantoon[E].var_E = slot!(game, Enemy_Phantoon[E].var_E).wrapping_add(1)
            );
            0
        } else {
            slot!(game, Enemy_Phantoon[E].var_E = 0);
            1
        }
    }
}
pub unsafe fn Phantoon_SetColorBasedOnHp(game: &mut Game) -> u8 {
    unsafe {
        let mut Entry: PairU16 = PairU16 { k: 0, j: 0 };
        let E = (0x40) as i32;
        if (slot!(game, Enemy_Phantoon[E].var_D)).wrapping_add(1)
            >= slot!(game, Enemy_Phantoon[E].var_E)
        {
            for v2 in (0u16..32).step_by(2) {
                Entry = Phantoon_SetColorBasedOnHp_FindEntry(v2, game);
                let j: u16 = Entry.j;
                let v5: u16 = {
                    let at =
                        usize::try_from((0xc000) + (((Entry.k as i32 >> 1) + 112) as isize) * 2)
                            .expect("native word outside RAM");
                    u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
                };
                match (
                    Phantoon_Func_10_CalculateNthTransitionColorFromXtoY(
                        slot!(game, Enemy_Phantoon[E].var_E),
                        v5,
                        j,
                        game,
                    ),
                    usize::try_from((0xc000) + (((v2 as i32 >> 1) + 112) as isize) * 2)
                        .expect("native word outside RAM"),
                ) {
                    (value, at) => {
                        let bytes = u16::to_ne_bytes(value);
                        game.ram().write_byte(at, bytes[0]);
                        game.ram().write_byte(at + 1, bytes[1]);
                    }
                };
            }
            slot!(
                game,
                Enemy_Phantoon[E].var_E = slot!(game, Enemy_Phantoon[E].var_E).wrapping_add(1)
            );
            0
        } else {
            slot!(game, Enemy_Phantoon[E].var_E = 0);
            1
        }
    }
}
pub unsafe fn Phantoon_Func_9(game: &mut Game) -> u8 {
    unsafe {
        let E = (0x40) as i32;
        if (slot!(game, Enemy_Phantoon[E].var_D)).wrapping_add(1)
            >= slot!(game, Enemy_Phantoon[E].var_E)
        {
            for i in (0i32..224).step_by(2) {
                word!(
                    game.palette_buffer[(i >> 1)] =
                        Phantoon_Func_10_CalculateNthTransitionColorFromXtoY(
                            slot!(game, Enemy_Phantoon[E].var_E),
                            word!(game.palette_buffer[(i >> 1)]),
                            crate::rtl::rom_table_word!(game, 0xa7ca61, (i >> 1) as isize),
                            game,
                        )
                );
            }
            slot!(
                game,
                Enemy_Phantoon[E].var_E = slot!(game, Enemy_Phantoon[E].var_E).wrapping_add(1)
            );
            0
        } else {
            slot!(game, Enemy_Phantoon[E].var_E = 0);
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
    if a == 0 {
        return k;
    }
    let v4: u16 = (a).wrapping_sub(1);
    let E = (0x40) as i32;
    if v4 == slot!(game, Enemy_Phantoon[E].var_D) {
        return j;
    }
    let r20: u16 = (v4).wrapping_add(1);
    let mut r18: u16 = (j).wrapping_sub(k);
    let v6: u8 = abs16(r18) as u8;
    let RegWord: u16 = SnesDivide(
        ((v6 as i32) << 8) as u16,
        (slot!(byte game, Enemy_Phantoon[E].var_D[0]) as i32 - r20 as i32 + 1) as u8,
    );
    r18 = (if r18 & 0x8000 != 0 {
        -(RegWord as i32)
    } else {
        RegWord as i32
    }) as u16;
    ((r18).wrapping_add(u16::swap_bytes(k))) >> 8
}
pub unsafe fn Etecoon_Func_1(k: u16, game: &mut Game) {
    if word!(game.earthquake_timer) != 0 {
        let E = (k as i32) as i32;
        slot!(
            game,
            Enemy_Etecoon[E].parameter_2 =
                ((slot!(byte game, Enemy_Etecoon[E].parameter_2[0])) as u16) | (0x8000u16)
        );
        slot!(
            game,
            Enemy_Etecoon[E].base.instruction_timer =
                (slot!(game, Enemy_Etecoon[E].base.instruction_timer)).wrapping_add(128)
        );
    }
}
pub unsafe fn Etecoon_Func_2(k: u16, game: &mut Game) -> u8 {
    let E = (k as i32) as i32;
    {
        let (actor, amount) = (
            k,
            (words!((
                slot!(game, Enemy_Etecoon[E].var_C),
                slot!(game, Enemy_Etecoon[E].var_D)
            ))) as i32,
        );
        (actor_api::move_horizontal(
            &mut Runtime::new(game),
            Key::from_legacy(actor),
            amount,
            actor_api::SlopePolicy::Ignore,
        ) == actor_api::MovementResult::Blocked) as u8
    }
}
pub unsafe fn Etecoon_Func_3(k: u16, game: &mut Game) -> u8 {
    let E = (k as i32) as i32;
    let amt: i32 = (words!((
        slot!(game, Enemy_Etecoon[E].var_A),
        slot!(game, Enemy_Etecoon[E].var_B)
    ))) as i32;
    if slot!(game, Enemy_Etecoon[E].var_A) as i32 - 5i32 & 0x8000 != 0 {
        slot!(
            game,
            Enemy_Etecoon[E](var_A, var_B) += words!(game.samus_y_accel, samus_y_subaccel)
        );
    }
    {
        let (actor, amount) = (k, amt);
        (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount)
            == actor_api::MovementResult::Blocked) as u8
    }
}
static funcs_A2A79: [Func_Y_V; 3] = callback_table! { nonnull Func_Y_V;
    Etecoon_Func_13, Etecoon_Func_14, Etecoon_Func_15,
};
pub unsafe fn Etecoon_Func_13(j: u16, game: &mut Game) {
    let E = (j as i32) as i32;
    slot!(game, Enemy_Etecoon[E].var_F = (0xa7ec97i32) as u16);
    slot!(
        game,
        Enemy_Etecoon[E].base.current_instruction = addr_stru_A7E828 as u16
    );
    slot!(game, Enemy_Etecoon[E].var_C = g_word_A7E90C);
    slot!(game, Enemy_Etecoon[E].var_D = g_word_A7E90E);
}
pub unsafe fn Etecoon_Func_14(j: u16, game: &mut Game) {
    let E = (j as i32) as i32;
    slot!(game, Enemy_Etecoon[E].var_F = (0xa7ecbbi32) as u16);
    slot!(
        game,
        Enemy_Etecoon[E].base.current_instruction = addr_stru_A7E880 as u16
    );
    slot!(game, Enemy_Etecoon[E].var_C = g_word_A7E908);
    slot!(game, Enemy_Etecoon[E].var_D = g_word_A7E90A);
}
pub unsafe fn Etecoon_Func_15(j: u16, game: &mut Game) {
    let E = (j as i32) as i32;
    slot!(game, Enemy_Etecoon[E].var_F = (0xa7ed75i32) as u16);
    slot!(
        game,
        Enemy_Etecoon[E].base.current_instruction =
            (slot!(game, Enemy_Etecoon[E].base.current_instruction)).wrapping_add(2)
    );
    slot!(game, Enemy_Etecoon[E].var_C = g_word_A7E908);
    slot!(game, Enemy_Etecoon[E].var_D = g_word_A7E90A);
}
pub unsafe fn Dachora_Func_6(k: u16, game: &mut Game) -> i32 {
    let E = (k as i32) as i32;
    if ((slot!(game, Enemy_Dachora[E].var_A)) as i16).wrapping_sub((g_word_A7F4D5) as i16) >= 0 {
        if slot!(game, Enemy_Dachora[E].var_E) == 1 {
            QueueSfx2_Max6(0x39, game);
        }
        let v2: u16 = (slot!(game, Enemy_Dachora[E].var_E)).wrapping_sub(1);
        slot!(game, Enemy_Dachora[E].var_E = v2);
        if v2 as u8 == 0 {
            Dachora_Func_1(
                crate::rtl::rom_table_word!(
                    game,
                    0xa7f787,
                    slot!(byte game, Enemy_Dachora[E].var_E[1]) as isize
                ),
                k,
                game,
            );
            let v3: u16 = (slot!(game, Enemy_Dachora[E].var_E)).wrapping_add(272);
            slot!(game, Enemy_Dachora[E].var_E = v3);
            if v3 as i32 - 1040i32 & 0x8000 == 0 {
                slot!(game, Enemy_Dachora[E].var_E = 784);
            }
        }
    }
    {
        let (actor, amount) = (k, (1i16 as i32) << 16);
        (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount)
            == actor_api::MovementResult::Blocked) as u8
    };
    if ((slot!(game, Enemy_Dachora[E].var_A)) as i16).wrapping_sub((g_word_A7F4D5) as i16) >= 0
        && ((slot!(game, Enemy_Dachora[E].var_B)) as i16).wrapping_sub((g_word_A7F4D7) as i16) >= 0
    {
        slot!(game, Enemy_Dachora[E].var_A = g_word_A7F4D5);
        slot!(game, Enemy_Dachora[E].var_B = g_word_A7F4D7);
        return (words!((g_word_A7F4D5, g_word_A7F4D7))) as i32;
    }
    slot!(
        game,
        Enemy_Dachora[E](var_A, var_B) += words!((g_word_A7F4D9, g_word_A7F4DB))
    );
    if (slot!(game, Enemy_Dachora[E].var_A) == 4 || slot!(game, Enemy_Dachora[E].var_A) == 8)
        && slot!(game, Enemy_Dachora[E].var_B) == 0
    {
        slot!(
            game,
            Enemy_Dachora[E].base.current_instruction =
                (slot!(game, Enemy_Dachora[E].base.current_instruction)).wrapping_add(28)
        );
    }
    (words!((
        slot!(game, Enemy_Dachora[E].var_A),
        slot!(game, Enemy_Dachora[E].var_B)
    ))) as i32
}
pub unsafe fn Dachora_Func_9(k: u16, game: &mut Game) {
    let EK = (k as i32) as i32;
    let E1 = (k as i32 + 64) as i32;
    let dachor_var_A: u16 = slot!(game, Enemy_Dachora[E1].var_A);
    if dachor_var_A != 0 {
        slot!(
            game,
            Enemy_Dachora[E1].var_A = (dachor_var_A).wrapping_sub(1)
        );
    } else {
        slot!(game, Enemy_Dachora[E1].var_A = g_word_A7F4D1);
        if slot!(game, Enemy_Dachora[E1].var_D) != 0 {
            let E2 = (k as i32 + 128) as i32;
            if slot!(game, Enemy_Dachora[E2].var_D) != 0 {
                let E3 = (k as i32 + 192) as i32;
                if slot!(game, Enemy_Dachora[E3].var_D) != 0 {
                    let E4 = (k as i32 + 256) as i32;
                    if slot!(game, Enemy_Dachora[E4].var_D) == 0 {
                        slot!(
                            game,
                            Enemy_Dachora[E4].base.x_pos =
                                slot!(game, Enemy_Dachora[EK].base.x_pos)
                        );
                        slot!(
                            game,
                            Enemy_Dachora[E4].base.y_pos =
                                slot!(game, Enemy_Dachora[EK].base.y_pos)
                        );
                        slot!(game, Enemy_Dachora[E4].var_D = g_word_A7F4D3);
                    }
                } else {
                    slot!(
                        game,
                        Enemy_Dachora[E3].base.x_pos = slot!(game, Enemy_Dachora[EK].base.x_pos)
                    );
                    slot!(
                        game,
                        Enemy_Dachora[E3].base.y_pos = slot!(game, Enemy_Dachora[EK].base.y_pos)
                    );
                    slot!(game, Enemy_Dachora[E3].var_D = g_word_A7F4D3);
                }
            } else {
                slot!(
                    game,
                    Enemy_Dachora[E2].base.x_pos = slot!(game, Enemy_Dachora[EK].base.x_pos)
                );
                slot!(
                    game,
                    Enemy_Dachora[E2].base.y_pos = slot!(game, Enemy_Dachora[EK].base.y_pos)
                );
                slot!(game, Enemy_Dachora[E2].var_D = g_word_A7F4D3);
            }
        } else {
            slot!(
                game,
                Enemy_Dachora[E1].base.x_pos = slot!(game, Enemy_Dachora[EK].base.x_pos)
            );
            slot!(
                game,
                Enemy_Dachora[E1].base.y_pos = slot!(game, Enemy_Dachora[EK].base.y_pos)
            );
            slot!(game, Enemy_Dachora[E1].var_D = g_word_A7F4D3);
        }
    };
}
pub unsafe fn Dachora_Func_10(k: u16, game: &mut Game) {
    let E = (k as i32) as i32;
    Dachora_Func_1(
        crate::rtl::rom_table_word!(
            game,
            0xa7f92d,
            slot!(byte game, Enemy_Dachora[E].var_E[1]) as isize
        ),
        k,
        game,
    );
    let v2: u16 = (slot!(game, Enemy_Dachora[E].var_E)).wrapping_add(256);
    slot!(game, Enemy_Dachora[E].var_E = v2);
    if v2 as i32 - 1024i32 & 0x8000 == 0 {
        slot!(game, Enemy_Dachora[E].var_E = 0);
    }
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [10982544] KraidsArm_Touch [] {
        if word!(game.samus_invincibility_timer) == 0 {
            Kraid_Func_1(game);
            slot!(game, EnemyData[0i32 + (4) * 64].ai_var_A = (0xa7b89bi32) as u16);
            Kraid_Touch(game);
        }
    };
    [10987865] Kraid_Init [] {
        match (-89i32 as u8, 0x604 + offset_of!(LongPtr, bank)) { (value, at) => game.ram().write_byte(at, value) };
        match (-89i32 as u8, 0x601 + offset_of!(LongPtr, bank)) { (value, at) => game.ram().write_byte(at, value) };
        match ((0xa7c24ei32) as u16, 0x604 + offset_of!(LongPtr, addr)) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        match ((0xa7c325i32) as u16, 0x601 + offset_of!(LongPtr, addr)) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        if Kraid_CheckIfDead(game) != 0 {
            word_transfers!(game, ascending, 16; le(target_palettes, 96) <- rom(0xa786c7, 0));
            word_transfers!(game, descending, 1024; le(tilemap_stuff, 1023) <- value(824));
            let E = (0) as i32;
            slot!(game, Enemy_Kraid[E].field_4 = 0);
            let v4: u16 = word!(game.vram_write_queue_tail);
            let mut v5 = (0xd0 + (word!(game.vram_write_queue_tail) as i32)) as i32;
            match (512, usize::try_from((v5) as isize + offset_of!(VramWriteEntry, size) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
            match (addr_kKraidsRoomBg as i32 as VoidP, usize::try_from((v5) as isize + offset_of!(VramWriteEntry, src.addr) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
            match (-89i32 as u8, usize::try_from((v5) as isize + offset_of!(VramWriteEntry, src.bank) as isize).expect("VRAM entry outside RAM")) { (value, at) => game.ram().write_byte(at, value) };
            match ((((game.ram().read_byte(0x5d) as i32 & 0xf) << 8) + 16128) as u16, usize::try_from((v5) as isize + offset_of!(VramWriteEntry, vram_dst) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
            word!(game.vram_write_queue_tail = (v4).wrapping_add(7));
            Kraid_SpawnPlmToClearCeiling(game);
            Kraid_ClearSomeSpikes(game);
            slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7c715i32) as u16);
        } else {
            match (67, 0x59) { (value, at) => game.ram().write_byte(at, value) };
            word!(game.camera_distance_index = 2);
            match (0, usize::try_from(0xcd20).expect("native word outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
            match (1, usize::try_from((0xcd20) + (2)).expect("native word outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
            let E_0 = (0) as i32;
            slot!(game, Enemy_Kraid[E_0].kraid_min_y_pos_eject = 324);
            let mut v8: u16 = slot!(game, Enemy_Kraid[E_0].base.health) >> 3;
            let r18: u16 = v8;
            for v7 in (0u16..16).step_by(2) {
                match (v8, (slot!(@at Enemy_Kraid, v7 as i32, kraid_healths_8ths) + (0i32 as usize) * 2)) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
                v8 = (v8).wrapping_add(r18);
            }
            Kraid_SetupGfxWithTilePrioClear(0xdfff, game);
            let v10: u16 = slot!(game, Enemy_Kraid[E_0].base.health) >> 2;
            match (v10, (slot!(@at Enemy_Kraid, E_0, kraid_healths_4ths) + (0i32 as usize) * 2)) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
            let v11: u16 = ({ let at = slot!(@at Enemy_Kraid, E_0, kraid_healths_4ths) + (0i32 as usize) * 2; u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]) }).wrapping_add(v10);
            match (v11, (slot!(@at Enemy_Kraid, E_0, kraid_healths_4ths) + (1i32 as usize) * 2)) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
            let v12: u16 = ({ let at = slot!(@at Enemy_Kraid, E_0, kraid_healths_4ths) + (0i32 as usize) * 2; u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]) }).wrapping_add(v11);
            match (v12, (slot!(@at Enemy_Kraid, E_0, kraid_healths_4ths) + (2i32 as usize) * 2)) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
            match (({ let at = slot!(@at Enemy_Kraid, E_0, kraid_healths_4ths) + (0i32 as usize) * 2; u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]) }).wrapping_add(v12), (slot!(@at Enemy_Kraid, E_0, kraid_healths_4ths) + (3i32 as usize) * 2)) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
            match (0, (slot!(@at Enemy_Kraid, E_0, field_2E) + (8i32 as usize) * 2)) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
            slot!(game, Enemy_Kraid[E_0].base; x_pos = 176, y_pos = 592, properties = (slot!(game, Enemy_Kraid[E_0].base.properties)) | ((kEnemyProps_Tangible) as u16));
            slot!(game, Enemy_Kraid[E_0]; kraid_var_A = (0xa7c865i32) as u16, kraid_var_F = 300, kraid_next = (0xa7c86bi32) as u16, kraid_var_C = 64);
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
            let v0 = (0) as i32;
            slot!(game, EnemyData[v0 + (1) * 64].palette_index = slot!(game, EnemyData[v0].palette_index));
            slot!(game, EnemyData[v0 + (1) * 64].ai_var_A = (0xa7ba2di32) as u16);
            slot!(game, EnemyData[v0 + (1) * 64]; current_instruction = addr_kKraid_Ilist_8AA4 as u16, instruction_timer = 1);
            slot!(game, EnemyData[v0 + (1) * 64].ai_var_B = 0);
        };
    };
    [10988392] KraidsTopLint_Init [] {
        if Kraid_CheckIfDead(game) != 0 {
            Kraid_SetEnemyPropsToDead(game);
        } else {
            let v0 = (0) as i32;
            slot!(game, EnemyData[v0 + (2) * 64].palette_index = slot!(game, EnemyData[v0].palette_index));
            slot!(game, EnemyData[v0 + (2) * 64]; instruction_timer = 0x7fff, current_instruction = addr_kKraid_Ilist_8AFE as u16);
            slot!(game, EnemyData[v0 + (2) * 64].spritemap_pointer = addr_kKraid_Sprmap_A5DF as u16);
            slot!(game, EnemyData[v0 + (2) * 64]; ai_var_A = addr_locret_A7B831 as u16, ai_preinstr = 0x7fff, ai_var_C = 0);
        };
    };
    [10988444] KraidsMiddleLint_Init [] {
        if Kraid_CheckIfDead(game) != 0 {
            Kraid_SetEnemyPropsToDead(game);
        } else {
            let v0 = (0) as i32;
            slot!(game, EnemyData[v0 + (3) * 64].palette_index = slot!(game, EnemyData[v0].palette_index));
            slot!(game, EnemyData[v0 + (3) * 64]; instruction_timer = 0x7fff, current_instruction = addr_kKraid_Ilist_8AFE as u16);
            slot!(game, EnemyData[v0 + (3) * 64].spritemap_pointer = addr_kKraid_Sprmap_A5DF as u16);
            slot!(game, EnemyData[v0 + (3) * 64].ai_var_A = addr_locret_A7B831 as u16);
            slot!(game, EnemyData[v0 + (3) * 64].ai_var_C = -16i32 as u16);
        };
    };
    [10988490] KraidsBottomLint_Init [] {
        if Kraid_CheckIfDead(game) != 0 {
            Kraid_SetEnemyPropsToDead(game);
        } else {
            let v0 = (0) as i32;
            slot!(game, EnemyData[v0 + (4) * 64].palette_index = slot!(game, EnemyData[v0].palette_index));
            slot!(game, EnemyData[v0 + (4) * 64]; instruction_timer = 0x7fff, current_instruction = addr_kKraid_Ilist_8AFE as u16);
            slot!(game, EnemyData[v0 + (4) * 64].spritemap_pointer = addr_kKraid_Sprmap_A5DF as u16);
            slot!(game, EnemyData[v0 + (4) * 64].ai_var_A = addr_locret_A7B831 as u16);
            slot!(game, EnemyData[v0 + (4) * 64].ai_var_C = -16i32 as u16);
        };
    };
    [10988536] KraidsFoot_Init [] {
        if Kraid_CheckIfDead(game) != 0 {
            Kraid_SetEnemyPropsToDead(game);
        } else {
            let palette_index: u16 = slot!(game, Enemy_Kraid[0i32].base.palette_index);
            let E = (0x140) as i32;
            slot!(game, Enemy_Kraid[E].base.palette_index = palette_index);
            slot!(game, Enemy_Kraid[E].base; current_instruction = addr_kKraid_Ilist_86E7 as u16, instruction_timer = 1);
            slot!(game, Enemy_Kraid[E].kraid_var_A = (0xa7ba2di32) as u16);
            slot!(game, Enemy_Kraid[E].kraid_next = 0);
        };
    };
    [10988577] Kraid_Main [] {
        Kraid_Shot_Mouth(game);
        Kraid_Palette_Handling(game);
        Kraid_Shot_Body(game);
        Kraid_Enemy_Touch(game);
        let E = (0) as i32;
        word! { game;
            reg_BG2HOFS = (slot!(game, Enemy_Kraid[E].base.x_width) as i32 + word!(game.reg_BG1HOFS) as i32
                - word!(game.bg1_x_offset) as i32
                - slot!(game, Enemy_Kraid[E].base.x_pos) as i32) as u16;
            reg_BG2VOFS = (word!(game.layer1_y_pos) as i32 - slot!(game, Enemy_Kraid[E].base.y_pos) as i32 + 152) as u16;
        }
        CallKraidFunc(((slot!(game, Enemy_Kraid[E].kraid_var_A)) as u32) | (0xa70000u32), game);
    };
    [10991549] KraidsArm_Main [] {
        let r18: u16 = (word!(game.layer1_y_pos)).wrapping_add(224);
        let E = (0) as i32;
        let v1: u16 = (slot!(game, Enemy_Kraid[E].base.y_pos)).wrapping_sub(44);
        let E1 = (0x40) as i32;
        slot!(game, Enemy_Kraid[E1].base.y_pos = v1);
        let v3: u16 = v1;
        let mut v4: u16 = (slot!(game, Enemy_Kraid[E1].base.properties)) | (0x100u16);
        if ((v3) as i16).wrapping_sub((word!(game.layer1_y_pos)) as i16) >= 0
            && ((v3) as i16).wrapping_sub((r18) as i16) < 0
        {
            v4 = (slot!(game, Enemy_Kraid[E1].base.properties)) & (0xfeffu16);
        }
        slot!(game, Enemy_Kraid[E1].base.properties = v4);
        slot!(game, Enemy_Kraid[E1].base.x_pos = slot!(game, Enemy_Kraid[E].base.x_pos));
        if slot!(byte game, Enemy_Kraid[E].kraid_mouth_flags[1]) != 0 {
            slot!(game, Enemy_Kraid[E1].base.instruction_timer = slot!(game, Enemy_Kraid[E1].base.instruction_timer).wrapping_add(1));
        }
    };
    [10991617] KraidsTopLint_Main [] {
        slot!(game, EnemyData[0x80i32].instruction_timer = 0x7fff);
        KraidLintCommon_Main(0x80, game);
    };
    [10991629] KraidsMiddleLint_Main [] {
        slot!(game, EnemyData[0xc0i32].instruction_timer = 0x7fff);
        KraidLintCommon_Main(0xc0, game);
    };
    [10991641] KraidsBottomLint_Main [] {
        slot!(game, EnemyData[0x100i32].instruction_timer = 0x7fff);
        KraidLintCommon_Main(0x100, game);
    };
    [10992118] KraidsFoot_Main [] {
        let v0 = (0) as i32;
        slot!(game, EnemyData[v0 + (5) * 64].x_pos = slot!(game, EnemyData[v0].x_pos));
        let v1: u16 = (slot!(game, EnemyData[v0].y_pos)).wrapping_add(100);
        slot!(game, EnemyData[v0 + (5) * 64].y_pos = v1);
        let v2: u16 = v1;
        let mut v3: i16 = ((v1) as i16).wrapping_sub(224);
        let mut v4: u16 = (slot!(game, EnemyData[v0 + (5) * 64].properties)) & (0xfeffu16);
        if ((v2) as i16).wrapping_sub((word!(game.layer1_y_pos)) as i16) >= 0 {
            if (v3).wrapping_sub((word!(game.layer1_y_pos)) as i16) >= 0 {
                v4 = (v4) | (0x100u16);
            }
        } else {
            v4 = (v4) | (0x100u16);
        }
        slot!(game, EnemyData[v0 + (5) * 64].properties = v4);
        CallKraidFunc(((slot!(game, EnemyData[v0 + (5) * 64].ai_var_A)) as u32) | (0xa70000u32), game);
    };
    [10992847 | 10992862] KraidsGoodFingernail_Touch [] {
        let v0: u16 = 0;
        printf(b"X undefined\n\0" as *const u8 as *const c_char);
        actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
        ({ let (_ignored_key, animation) = (word!(game.cur_enemy_index), v0); actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Normal { animation }) });
    };
    [10992946] KraidsGoodFingernail_Main [] {
        let v0 = (0) as i32;
        if slot!(game, EnemyData[v0].health) as i32 - 1i32 & 0x8000 != 0 {
            let v1 = (0) as i32;
            let fresh12: i32 = v1 + (6) * 64;
slot!(game, EnemyData[fresh12].properties = (slot!(game, EnemyData[fresh12].properties) as i32
                    | (kEnemyProps_Deleted as i32 | kEnemyProps_Invisible as i32))
                    as u16);
        } else {
            CallKraidFunc(((slot!(game, EnemyData[v0 + (6) * 64].ai_var_A)) as u32) | (0xa70000u32), game);
        };
    };
    [10992969] KraidsBadFingernail_Main [] {
        let v0 = (0) as i32;
        if slot!(game, EnemyData[v0].health) as i32 - 1i32 & 0x8000 != 0 {
            let v1 = (0) as i32;
            let fresh4: i32 = v1 + (7) * 64;
slot!(game, EnemyData[fresh4].properties = (slot!(game, EnemyData[fresh4].properties) as i32
                    | (kEnemyProps_Deleted as i32 | kEnemyProps_Invisible as i32))
                    as u16);
        } else {
            CallKraidFunc(((slot!(game, EnemyData[v0 + (7) * 64].ai_var_A)) as u32) | (0xa70000u32), game);
        };
    };
    [10997235] Phantoon_Init [] {
        word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(824));
        word_transfers!(game, descending, 1024; native(0x97fe) <- value(0));
        word_transfers!(game, descending, 16; native(0xc2fe) <- value(0));
        word!(game.enemy_bg2_tilemap_size = 864);
        minimap::disable_for_boss(&mut crate::game::session_runtime::Runtime::new(game));
        let E = (word!(game.cur_enemy_index) as i32) as i32;
        slot!(game, Enemy_Phantoon[E].var_E = 120);
        slot!(game, Enemy_Phantoon[E]; var_A = 0, var_B = 0);
        word!(game.g_word_7E9032 = 0);
        let E1 = (0x40) as i32;
        slot!(game, Enemy_Phantoon[E1]; parameter_1 = 0, parameter_2 = 0);
        let E0 = (0) as i32;
        slot!(game, Enemy_Phantoon[E0].base.properties = (slot!(game, Enemy_Phantoon[E0].base.properties)) | ((kEnemyProps_Tangible) as u16));
        static unk_A7CE51: SpawnHdmaObject_Args =
            SpawnHdmaObject_Args::new(0x1, 0x14, 0xce96 as VoidP);
        spawn_hdma(0xa7, &unk_A7CE51, game);
        Phantoon2_Init(game);
    };
    [10997333] Phantoon2_Init [] {
        let v6 = (word!(game.cur_enemy_index) as i32) as i32;
        slot!(game, EnemyData[v6].spritemap_pointer = addr_kSpritemap_Nothing_A7 as u16);
        slot!(game, EnemyData[v6].instruction_timer = 1);
        slot!(game, EnemyData[v6].timer = 0);
        let v7 = (0) as i32;
        slot!(game, EnemyData[v6].palette_index = slot!(game, EnemyData[v7].palette_index));
        slot!(game, EnemyData[v6].vram_tiles_index = slot!(game, EnemyData[v7].vram_tiles_index));
        slot!(game, EnemyData[v6].current_instruction = crate::rtl::rom_table_word!(game, 0xa7ce8e, slot!(game, EnemyData[v6].parameter_2) as isize));
        slot!(game, EnemyData[v6].ai_preinstr = (0xa7d4a9i32) as u16);
        slot!(game, EnemyData[v7 + (3) * 64].parameter_1 = 0);
        slot!(game, EnemyData[v7 + (3) * 64].ai_var_C = -1i32 as u16);
    };
    [10997414] Phantoon_Main [] {
        actors::phantoon_actions::debug_toggle_sprite_map(word!(game.cur_enemy_index), game);
        let EK = (word!(game.cur_enemy_index) as i32) as i32;
        actor_species_pre_instruction(((slot!(game, Enemy_Phantoon[EK].var_F)) as u32) | (0xa70000u32), game);
        if word!(game.cur_enemy_index) == 0 {
            let E0 = (0) as i32;
            let E1 = (0x40) as i32;
            let E2 = (0x80) as i32;
            let E3 = (0xc0) as i32;
            let x_pos: u16 = slot!(game, Enemy_Phantoon[E0].base.x_pos);
            slot!(game, Enemy_Phantoon[E1].base.x_pos = x_pos);
            slot!(game, Enemy_Phantoon[E2].base.x_pos = x_pos);
            slot!(game, Enemy_Phantoon[E3].base.x_pos = x_pos);
            let y_pos: u16 = slot!(game, Enemy_Phantoon[E0].base.y_pos);
            slot!(game, Enemy_Phantoon[E1].base.y_pos = y_pos);
            slot!(game, Enemy_Phantoon[E2].base.y_pos = y_pos);
            slot!(game, Enemy_Phantoon[E3].base.y_pos = y_pos);
            if slot!(game, Enemy_Phantoon[E1].parameter_1) == 0 {
                word! { game;
                    reg_BG2HOFS =
                        (word!(game.layer1_x_pos) as i32 - slot!(game, Enemy_Phantoon[E0].base.x_pos) as i32 + 40) as u16;
                    reg_BG2VOFS =
                        (word!(game.layer1_y_pos) as i32 - slot!(game, Enemy_Phantoon[E0].base.y_pos) as i32 + 40) as u16;
                }
            }
        }
    };
    [11001151] Phantoon_Hurt [] {
        let mut Entry: PairU16 = PairU16 { k: 0, j: 0 };
        let E0 = (0) as i32;
        if slot!(game, Enemy_Phantoon[E0].base.flash_timer) != 8 {
            if (slot!(game, Enemy_Phantoon[E0].base.frame_counter) & 2) == 0 {
                if slot!(byte game, Enemy_Phantoon[0x80i32].parameter_2[1])
                    == 0
                {
                    return;
                }
            } else {
                if slot!(byte game, Enemy_Phantoon[0x80i32].parameter_2[1])
                    == 0
                {
                    word_transfers!(game, descending, 16; native(0xc0fe) <- value(0x7fff));
                    let E3_0 = (0x80) as i32;
                    slot!(game, Enemy_Phantoon[E3_0].parameter_2 = (slot!(game, Enemy_Phantoon[E3_0].parameter_2)) | (0x100u16));
                }
                return;
            }
        }
        let mut v1: u16 = 30;
        do_until! { ((Entry.k) as i16).wrapping_sub(2) < 0 ;
            Entry = Phantoon_SetColorBasedOnHp_FindEntry(v1, game);
            match (Entry.j, usize::try_from((0xc000) + (((Entry.k as i32 >> 1) + 112) as isize) * 2).expect("native word outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
            v1 = (Entry.k).wrapping_sub(2);
        }
        let E3 = (0x80) as i32;
        slot!(game, Enemy_Phantoon[E3].parameter_2 = slot!(byte game, Enemy_Phantoon[E3].parameter_2[0]) as u16);
    };
    [11001237] Phantoon_Touch [] {
        actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
    };
    [11001243] Phantoon_Shot [] {
        let E0 = (0) as i32;
        if slot!(game, Enemy_Phantoon[E0].var_F) as i32 + 0x26b8 & 0x8000 == 0 {
            return;
        }
        let v1: u16 = word!(game.cur_enemy_index);
        let EK = (word!(game.cur_enemy_index) as i32) as i32;
        let mut health: u16 = slot!(game, Enemy_Phantoon[EK].base.health);
        actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::ReactionOnly);
        if slot!(game, Enemy_Phantoon[EK].base.health) == 0 {
            QueueSfx2_Max6(0x73, game);
            slot!(game, Enemy_Phantoon[0x80i32].parameter_2 = 1);
            slot!(game, Enemy_Phantoon[E0].base.properties = (slot!(game, Enemy_Phantoon[E0].base.properties)) | ((kEnemyProps_Tangible) as u16));
            Phantoon_StartDeathSequence(v1, game);
            return;
        }
        if (slot!(game, Enemy_Phantoon[EK].base.ai_handler_bits) & 2) != 0 {
            QueueSfx2_Max6(0x73, game);
            let phanto_var_F: u16 = slot!(game, Enemy_Phantoon[EK].var_F);
            'react_to_hit: {
                let initial_hit = phanto_var_F as i32 == (0xa7d60d & 0xffff)
                    || phanto_var_F as i32 == (0xa7d788 & 0xffff);
                if !initial_hit && phanto_var_F as i32 != (0xa7d678 & 0xffff) {
                    break 'react_to_hit;
                }
                health = health.wrapping_sub(slot!(game, Enemy_Phantoon[EK].base.health));
                if health as i32 - 300i32 & 0x8000 != 0
                    || (word!(game.projectile_type[word!(game.collision_detection_index)]) & 0xf00)
                        != 512
                {
                    let E2 = (v1 as i32 + 0x80) as i32;
                    let damage = health.wrapping_add(slot!(game, Enemy_Phantoon[E2].var_B));
                    slot!(game, Enemy_Phantoon[E2].var_B = damage);
                    if !initial_hit {
                        if damage as i32 - 300i32 & 0x8000 == 0 {
                            slot!(game, Enemy_Phantoon[EK].var_E = 1);
                        }
                        break 'react_to_hit;
                    }
                    if damage as i32 - 300i32 & 0x8000 == 0 {
                        slot!(game, Enemy_Phantoon[EK].var_F = (0xa7d6b9i32) as u16);
                    } else {
                        let v10: u16 = (game.next_random()) & (7u16);
                        slot!(game, Enemy_Phantoon[0x40i32].var_B = *(&raw const g_byte_A7CDA5 as *const u8).offset(v10 as i32 as isize)
                                as u16);
                        slot!(game, Enemy_Phantoon[0xc0i32].parameter_2 = v10);
                        let E2x = (0x80) as i32;
                        slot!(game, Enemy_Phantoon[E2x].parameter_2 = 1);
                        if slot!(game, Enemy_Phantoon[E2x].var_A) == 0 {
                            slot!(game, Enemy_Phantoon[E2x].var_A = 1);
                            if slot!(game, Enemy_Phantoon[EK].var_E) as i32 - 16i32 & 0x8000 == 0 {
                                slot!(game, Enemy_Phantoon[EK].var_E = 16);
                            }
                        }
                        return;
                    }
                } else {
                    slot!(game, Enemy_Phantoon[EK].var_F = (0xa7d85ci32) as u16);
                }
                slot!(game, Enemy_Phantoon[EK].var_E = 0);
                slot!(game, Enemy_Phantoon[0x80i32].var_A = 0);
                slot!(game, Enemy_Phantoon[v1 as i32 + 0x80i32].var_B = 0);
                word!(game.phantom_related_layer_flag |= (0x4000u16));
                slot!(game, Enemy_Phantoon[E0].base.instruction_timer = 1);
                let E1 = (0x40) as i32;
                slot!(game, Enemy_Phantoon[E1].base.instruction_timer = 1);
                slot!(game, Enemy_Phantoon[E0].base.current_instruction = addr_kKraid_Ilist_CC41 as u16);
                slot!(game, Enemy_Phantoon[E1].base.current_instruction = addr_kKraid_Ilist_CC91 as u16);
                slot!(game, Enemy_Phantoon[E0].base.properties = (slot!(game, Enemy_Phantoon[E0].base.properties)) | ((kEnemyProps_Tangible) as u16));
                slot!(game, Enemy_Phantoon[E1].var_F = 0);
            }
            slot!(game, Enemy_Phantoon[0x80i32].parameter_2 = 2);
        }
    };
    [11004178] Etecoon_Init [] {
        let E = (word!(game.cur_enemy_index) as i32) as i32;
        slot!(game, Enemy_Etecoon[E].base.properties = (slot!(game, Enemy_Etecoon[E].base.properties)) | ((kEnemyProps_DisableSamusColl) as u16));
        slot!(game, Enemy_Etecoon[E].base.spritemap_pointer = addr_kSpritemap_Nothing_A7 as u16);
        slot!(game, Enemy_Etecoon[E].base.instruction_timer = 1);
        slot!(game, Enemy_Etecoon[E].base.timer = 0);
        slot!(game, Enemy_Etecoon[E].base.current_instruction = addr_kEtecoon_Ilist_E8CE as u16);
        slot!(game, Enemy_Etecoon[E].var_F = (0xa7e9afi32) as u16);
        slot!(game, Enemy_Etecoon[E].var_E = -1i32 as u16);
    };
    [11004224] Etecoon_Main [] {
        let E = (word!(game.cur_enemy_index) as i32) as i32;
        if slot!(byte game, Enemy_Etecoon[E].parameter_2[1]) != 0 {
            slot!(game, Enemy_Etecoon[E].parameter_2 = (slot!(game, Enemy_Etecoon[E].parameter_2)).wrapping_sub(256));
        } else {
            actor_species_pre_instruction(((slot!(game, Enemy_Etecoon[E].var_F)) as u32) | (0xa70000u32), game);
        };
    };
    [11007197] Dachora_Init [] {
        let E = (word!(game.cur_enemy_index) as i32) as i32;
        slot!(game, Enemy_Dachora[E].base.properties = (slot!(game, Enemy_Dachora[E].base.properties)) | ((kEnemyProps_DisableSamusColl) as u16));
        slot!(game, Enemy_Dachora[E].base.spritemap_pointer = addr_kSpritemap_Nothing_A7 as u16);
        slot!(game, Enemy_Dachora[E].base.instruction_timer = 1);
        slot!(game, Enemy_Dachora[E].base.timer = 0);
        let mut dachor_parameter_1: i16 = slot!(game, Enemy_Dachora[E].parameter_1) as i16;
        if dachor_parameter_1 < 0 {
            if (dachor_parameter_1 as u16 & 1) != 0 {
                slot!(game, Enemy_Dachora[E].base.current_instruction = addr_kDachora_Ilist_F4B9 as u16);
            } else {
                slot!(game, Enemy_Dachora[E].base.current_instruction = addr_kDachora_Ilist_F3F7 as u16);
            }
            slot!(game, Enemy_Dachora[E].var_F = (0xa7f98ci32) as u16);
        } else {
            if dachor_parameter_1 != 0 {
                slot!(game, Enemy_Dachora[E].base.current_instruction = addr_kDachora_Ilist_F45B as u16);
            } else {
                slot!(game, Enemy_Dachora[E].base.current_instruction = addr_kDachora_Ilist_F399 as u16);
            }
            slot!(game, Enemy_Dachora[E].var_F = (0xa7f570i32) as u16);
        };
    };
    [11007278] Dachora_Main [] {
        let E = (word!(game.cur_enemy_index) as i32) as i32;
        actor_species_pre_instruction(((slot!(game, Enemy_Dachora[E].var_F)) as u32) | (0xa70000u32), game);
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
    @functions [pub unsafe fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10979983] Kraid_Instr_9 [] {
        let E = (0) as i32;
        if ((slot!(game, Enemy_Kraid[E].base.health)) as i16).wrapping_sub(({ let at = slot!(@at Enemy_Kraid, E, kraid_healths_8ths) + (3i32 as usize) * 2; u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]) }) as i16)
            < 0
        {
            let E1 = (0x40) as i32;
            if slot!(game, Enemy_Kraid[E1].base.current_instruction) as i32 + 0x75bf & 0x8000 != 0 {
                return EnemyScript::jump(addr_kKraid_Ilist_8A41 as u16);
            }
        }
        jp
    };
    @functions [pub unsafe fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [10991158] Kraid_Instr_DecYpos [jp] {
        let E = (0) as i32;
        slot!(game, Enemy_Kraid[E].base.y_pos = slot!(game, Enemy_Kraid[E].base.y_pos).wrapping_sub(1));
    };
    [10991164] Kraid_Instr_IncrYpos_Shake [jp] {
        let E = (0) as i32;
        slot!(game, Enemy_Kraid[E].base.y_pos = slot!(game, Enemy_Kraid[E].base.y_pos).wrapping_add(1));
        word! {game; earthquake_type = 1; earthquake_timer = 10; }
    };
    [10991182] Kraid_Instr_PlaySound_0x76 [jp] { QueueSfx2_Max6(0x76, game); };
    [10991194] Kraid_Instr_XposMinus3 [jp] {
        let E = (0) as i32;
        slot!(game, Enemy_Kraid[E].base.x_pos = (slot!(game, Enemy_Kraid[E].base.x_pos)).wrapping_sub(g_word_A7A91C));
    };
    [10991207] Kraid_Instr_XposMinus3b [jp] {
        let E = (0) as i32;
        slot!(game, Enemy_Kraid[E].base.x_pos = (slot!(game, Enemy_Kraid[E].base.x_pos)).wrapping_sub(g_word_A7A91C));
    };
    [10991220] Kraid_Instr_XposPlus3 [jp] {
        let E = (0) as i32;
        slot!(game, Enemy_Kraid[E].base.x_pos = (slot!(game, Enemy_Kraid[E].base.x_pos)).wrapping_add(g_word_A7A920));
    };
    [10991235] Kraid_Instr_MoveHimRight [jp] {
        let E = (0) as i32;
        if (slot!(game, Enemy_Kraid[E].base.x_pos) as i32 - 320i32 & 0x8000 != 0 || {
            let mut v3: u16 = (slot!(game, Enemy_Kraid[E].kraid_target_x)).wrapping_sub(1);
            slot!(game, Enemy_Kraid[E].kraid_target_x = v3);
            slot!(game, Enemy_Kraid[E].kraid_target_x) == 0
        }) && ({ let (actor, amount) = (0, (g_word_A7A922 as i16 as i32) << 16); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) != 0
        {
            word! {game; earthquake_type = 0; earthquake_timer = 7; }
            let x_pos: u16 = slot!(game, Enemy_Kraid[0].base.x_pos);
            slot!(game, Enemy_Kraid[0x140].base.x_pos = x_pos);
        }
    };
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [10999114] Phantoon_WavyFadeIn [] {
        Phantoon_FadeIn(0xc, game);
        if actors::phantoon_actions::approach_speed((words!((g_word_A7CD9D, g_word_A7CD9B))) as i32, game) != 0 {
            let E = (k as i32) as i32;
            slot!(game, Enemy_Phantoon[E].var_F = (0xa7d596i32) as u16);
            slot!(game, Enemy_Phantoon[0xc0i32].parameter_1 = 1);
            slot!(game, Enemy_Phantoon[E].var_E = 30);
        } else {
            let E_0 = (k as i32) as i32;
            let v2: bool = slot!(game, Enemy_Phantoon[E_0].var_E) == 1;
            slot!(game, Enemy_Phantoon[E_0].var_E = slot!(game, Enemy_Phantoon[E_0].var_E).wrapping_sub(1));
            let v3: bool = (slot!(game, Enemy_Phantoon[E_0].var_E) & 0x8000) != 0;
            if v2 || v3 {
                slot!(game, Enemy_Phantoon[0xc0i32].var_E = 1);
            }
        };
    };
    [10999271] Phantoon_MovePhantoonInFigure8ThenOpenEye [] {
        actors::phantoon_motion::advance_figure_eight(k, game);
        let E = (0x40) as i32;
        let v2: bool = slot!(game, Enemy_Phantoon[E].var_A) == 1;
        slot!(game, Enemy_Phantoon[E].var_A = slot!(game, Enemy_Phantoon[E].var_A).wrapping_sub(1));
        let v3: bool = (slot!(game, Enemy_Phantoon[E].var_A) & 0x8000) != 0;
        if v2 || v3 {
            slot!(game, Enemy_Phantoon[k as i32].var_F = (0xa7d4a8i32) as u16);
            slot!(game, Enemy_Phantoon[E].base; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_CC53 as u16);
            slot!(game, Enemy_Phantoon[0i32].parameter_2 = 0);
            actors::phantoon_actions::spawn_fireball_circle(k, game);
        }
    };
    [10999309] Phantoon_EyeFollowsSamusUntilTimerRunsOut [] {
        let EK = (k as i32) as i32;
        let v2: bool = slot!(game, Enemy_Phantoon[EK].var_E) == 1;
        slot!(game, Enemy_Phantoon[EK].var_E = slot!(game, Enemy_Phantoon[EK].var_E).wrapping_sub(1));
        let v3: bool = (slot!(game, Enemy_Phantoon[EK].var_E) & 0x8000) != 0;
        if v2 || v3 {
            slot!(game, Enemy_Phantoon[k as i32 + 128i32].var_B = 0);
            let E2 = (0x80) as i32;
            if slot!(game, Enemy_Phantoon[E2].var_A) == 0 {
                slot!(game, Enemy_Phantoon[EK].var_F = (0xa7d4a8i32) as u16);
                let E0 = (0) as i32;
                slot!(game, Enemy_Phantoon[E0].base.instruction_timer = 1);
                let E1 = (0x40) as i32;
                slot!(game, Enemy_Phantoon[E1].base.instruction_timer = 1);
                slot!(game, Enemy_Phantoon[E0].base.current_instruction = addr_kKraid_Ilist_CC41 as u16);
                slot!(game, Enemy_Phantoon[E1].base.current_instruction = addr_kKraid_Ilist_CC81 as u16);
                slot!(game, Enemy_Phantoon[E0].base.properties = (slot!(game, Enemy_Phantoon[E0].base.properties)) | ((kEnemyProps_Tangible) as u16));
                slot!(game, Enemy_Phantoon[E0].parameter_2 = 1);
                return;
            }
            slot!(game, Enemy_Phantoon[E2].var_A = 0);
            slot!(game, Enemy_Phantoon[EK].var_E = 60);
            slot!(game, Enemy_Phantoon[EK].var_F = (0xa7d65ci32) as u16);
        }
        Phantoon_ChangeEyeSpriteBasedOnSamusDist(game);
    };
    [10999416] Phantoon_IsSwooping [] {
        Phantoon_ChangeEyeSpriteBasedOnSamusDist(game);
        Phantoon_MoveInSwoopingPattern(k, game);
        let EK = (k as i32) as i32;
        let v2: bool = slot!(game, Enemy_Phantoon[EK].var_E) == 1;
        slot!(game, Enemy_Phantoon[EK].var_E = slot!(game, Enemy_Phantoon[EK].var_E).wrapping_sub(1));
        let v3: bool = (slot!(game, Enemy_Phantoon[EK].var_E) & 0x8000) != 0;
        if v2 || v3 {
            slot!(game, Enemy_Phantoon[EK].var_F = (0xa7d6b9i32) as u16);
            word!(game.phantom_related_layer_flag |= (0x4000u16));
            let E0 = (0) as i32;
            slot!(game, Enemy_Phantoon[E0].base.instruction_timer = 1);
            let E1 = (0x40) as i32;
            slot!(game, Enemy_Phantoon[E1].base.instruction_timer = 1);
            slot!(game, Enemy_Phantoon[E0].base.current_instruction = addr_kKraid_Ilist_CC41 as u16);
            slot!(game, Enemy_Phantoon[E1].base.current_instruction = addr_kKraid_Ilist_CC91 as u16);
            slot!(game, Enemy_Phantoon[E0].base.properties = (slot!(game, Enemy_Phantoon[E0].base.properties)) | ((kEnemyProps_Tangible) as u16));
            slot!(game, Enemy_Phantoon[E1].var_F = 0);
            slot!(game, Enemy_Phantoon[k as i32 + 128i32].var_B = 0);
        }
    };
    [10999481] Phantoon_FadeoutWithSwoop [] {
        Phantoon_MoveInSwoopingPattern(k, game);
        Phantoon_FadeOut(0xc, game);
        if slot!(game, Enemy_Phantoon[0x40i32].var_F) != 0 {
            let E = (k as i32) as i32;
            slot!(game, Enemy_Phantoon[E].var_F = (0xa7d6d4i32) as u16);
            slot!(game, Enemy_Phantoon[E].var_E = 120);
        }
    };
    [10999522] Phantoon_MoveLeftOrRightAndPickEyeOpenPatt [] {
        let E = (0) as i32;
        if (game.next_random() & 1) != 0 {
            slot!(game, Enemy_Phantoon[E].var_A = 136);
            slot!(game, Enemy_Phantoon[E].base.x_pos = 208);
        } else {
            slot!(game, Enemy_Phantoon[E].var_A = 399);
            slot!(game, Enemy_Phantoon[E].base.x_pos = 48);
        }
        slot!(game, Enemy_Phantoon[E].base.y_pos = 96);
        let E1 = (0x40) as i32;
        slot!(game, Enemy_Phantoon[E1].var_C = 0);
        slot!(game, Enemy_Phantoon[E].var_C = 1);
        slot!(game, Enemy_Phantoon[E]; var_B = 0, parameter_2 = 0);
        Phantoon_PickPatternForRound2(game);
        slot!(game, Enemy_Phantoon[k as i32].var_F = (0xa7d72di32) as u16);
        slot!(game, Enemy_Phantoon[E1].var_F = 0);
    };
    @functions [pub unsafe fn (_k: u16, game: &mut Game)];
    [10999597] Phantoon_FadeInBeforeFigure8 [] {
        Phantoon_FadeIn(0xc, game);
        if slot!(game, Enemy_Phantoon[0x40i32].var_F) != 0 {
            slot!(game, Enemy_Phantoon[0i32].var_F = (0xa7d5e7i32) as u16);
        }
    };
    @functions [pub unsafe fn (k: u16, game: &mut Game)];
    [10999615] Phantoon_BecomeSolidAfterRainingFireballs [] {
        let E1 = (0x40) as i32;
        slot!(game, Enemy_Phantoon[E1].var_F = 0);
        word!(game.phantom_related_layer_flag &= !(0x4000));
        let E = (0) as i32;
        slot!(game, Enemy_Phantoon[E].base.instruction_timer = 1);
        slot!(game, Enemy_Phantoon[E1].base.instruction_timer = 1);
        slot!(game, Enemy_Phantoon[E].base.current_instruction = addr_kKraid_Ilist_CC47 as u16);
        slot!(game, Enemy_Phantoon[E1].base.current_instruction = addr_kKraid_Ilist_CC9D as u16);
        slot!(game, Enemy_Phantoon[k as i32].var_F = (0xa7d767i32) as u16);
    };
    [10999655] Phantoon_FadeInDuringFireballRain [] {
        Phantoon_FadeIn(1, game);
        if slot!(game, Enemy_Phantoon[0x40i32].var_F) != 0 {
            let E = (0) as i32;
            slot!(game, Enemy_Phantoon[E].base.properties = (slot!(game, Enemy_Phantoon[E].base.properties) as i32 & !(kEnemyProps_Tangible as i32)) as u16);
            slot!(game, Enemy_Phantoon[k as i32].var_F = (0xa7d788i32) as u16);
            slot!(game, Enemy_Phantoon[E].var_E = 90);
        }
    };
    [10999765] Phantoon_FadeOutDuringFireballRain [] {
        Phantoon_FadeOut(0xc, game);
        if slot!(game, Enemy_Phantoon[0x40i32].var_F) != 0 {
            let E = (k as i32) as i32;
            slot!(game, Enemy_Phantoon[E].var_F = (0xa7d7f7i32) as u16);
            slot!(game, Enemy_Phantoon[E].var_E = crate::rtl::rom_table_word!(game, 0xa7cd63, (game.next_random() as i32 & 7) as isize));
        }
    };
    [10999850] Phantoon_FadeOutBeforeFirstFireballRain [] {
        Phantoon_FadeOut(0xc, game);
        actors::phantoon_motion::advance_figure_eight(k, game);
        let E = (0x40) as i32;
        let v2: bool = slot!(game, Enemy_Phantoon[E].var_A) == 1;
        slot!(game, Enemy_Phantoon[E].var_A = slot!(game, Enemy_Phantoon[E].var_A).wrapping_sub(1));
        let v3: bool = (slot!(game, Enemy_Phantoon[E].var_A) & 0x8000) != 0;
        if v2 || v3 {
            slot!(game, Enemy_Phantoon[0x80i32].var_A = 0);
            slot!(game, Enemy_Phantoon[k as i32].var_F = (0xa7d73fi32) as u16);
            if slot!(game, Enemy_Phantoon[0i32].base.x_pos) as i32 - 128i32 & 0x8000 != 0 {
                actors::phantoon_actions::spawn_fireball_pattern(k, 0, game);
            } else {
                actors::phantoon_actions::spawn_fireball_pattern(k, 2, game);
            }
        }
    };
    [10999900] Phantoon_FadeOutBeforeEnrage [] {
        Phantoon_FadeOut(0xc, game);
        if slot!(game, Enemy_Phantoon[0x40i32].var_F) != 0 {
            let E = (k as i32) as i32;
            slot!(game, Enemy_Phantoon[E].var_F = (0xa7d874i32) as u16);
            slot!(game, Enemy_Phantoon[E].var_E = 120);
        }
    };
    [10999953] Phantoon_FadeInEnragedPhantoon [] {
        Phantoon_FadeIn(0xc, game);
        let E = (0x40) as i32;
        if slot!(game, Enemy_Phantoon[E].var_F) != 0 {
            slot!(game, Enemy_Phantoon[0i32].var_F = (0xa7d8aci32) as u16);
            slot!(game, Enemy_Phantoon[k as i32].var_E = 4);
            slot!(game, Enemy_Phantoon[E].var_F = 0);
        }
    };
    [10999980] Phantoon_Enraged [] {
        let E = (k as i32) as i32;
        let v2: bool = slot!(game, Enemy_Phantoon[E].var_E) == 1;
        slot!(game, Enemy_Phantoon[E].var_E = slot!(game, Enemy_Phantoon[E].var_E).wrapping_sub(1));
        let v3: bool = (slot!(game, Enemy_Phantoon[E].var_E) & 0x8000) != 0;
        if v2 || v3 {
            if (slot!(game, Enemy_Phantoon[0x40i32].var_F) & 1) != 0 {
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
            let E1 = (0x40) as i32;
            let v7: u16 = (slot!(game, Enemy_Phantoon[E1].var_F)).wrapping_add(1);
            slot!(game, Enemy_Phantoon[E1].var_F = v7);
            if v7 as i32 - 8i32 & 0x8000 != 0 {
                slot!(game, Enemy_Phantoon[k as i32].var_E = 128);
            } else {
                slot!(game, Enemy_Phantoon[E1].base; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_CC91 as u16);
                slot!(game, Enemy_Phantoon[E1].var_F = 0);
                slot!(game, Enemy_Phantoon[k as i32].var_F = (0xa7d916i32) as u16);
            }
        }
    };
    [11000086] Phantoon_FadeoutAfterEnrage [] {
        Phantoon_FadeOut(0xc, game);
        if slot!(game, Enemy_Phantoon[0x40i32].var_F) != 0 {
            let E = (k as i32) as i32;
            slot!(game, Enemy_Phantoon[E].var_F = (0xa7d6d4i32) as u16);
            slot!(game, Enemy_Phantoon[E].var_E = 120);
        }
    };
    [11000110] Phantoon_CompleteSwoopAfterFatalShot [] {
        Phantoon_ChangeEyeSpriteBasedOnSamusDist(game);
        Phantoon_MoveInSwoopingPattern(k, game);
        let x_pos: u16 = slot!(game, Enemy_Phantoon[0i32].base.x_pos);
        if x_pos as i32 - 96i32 & 0x8000 == 0 && x_pos as i32 - 160i32 & 0x8000 != 0 {
            slot!(game, Enemy_Phantoon[k as i32].var_F = (0xa7d948i32) as u16);
        }
    };
    [11000136] Phantoon_DyingPhantoonFadeInOut [] {
        let E1 = (0x40) as i32;
        if (slot!(game, Enemy_Phantoon[E1].var_C) & 1) != 0 {
            Phantoon_FadeIn(0xc, game);
            if slot!(game, Enemy_Phantoon[E1].var_F) == 0 {
                return;
            }
        } else {
            Phantoon_FadeOut(0xc, game);
            if slot!(game, Enemy_Phantoon[E1].var_F) == 0 {
                return;
            }
        }
        slot!(game, Enemy_Phantoon[E1].var_F = 0);
        let v2: u16 = (slot!(game, Enemy_Phantoon[E1].var_C)).wrapping_add(1);
        slot!(game, Enemy_Phantoon[E1].var_C = v2);
        if v2 as i32 - 10i32 & 0x8000 == 0 {
            let E = (k as i32) as i32;
            slot!(game, Enemy_Phantoon[E].var_F = (0xa7d98bi32) as u16);
            slot!(game, Enemy_Phantoon[E].var_E = 15);
            slot!(game, Enemy_Phantoon[0x80i32].var_F = 0);
            slot!(game, Enemy_Phantoon[0i32].var_A = 0);
        }
    };
    [11000401] Phantoon_WavyDyingPhantoonAndCry [] {
        initialize_phantoon_motion(1, g_word_A7CDA3, game);
        slot!(game, Enemy_Phantoon[0xc0i32].var_D = 0);
        slot!(game, Enemy_Phantoon[k as i32].var_F = (0xa7da86i32) as u16);
        let mut Phantoon = (0x40) as i32;
        slot!(game, Enemy_Phantoon[Phantoon].var_C = 2);
        let v2: u16 = (slot!(game, Enemy_Phantoon[0i32].base.properties) as i32
            & !(kEnemyProps_DisableSamusColl as i32
                | kEnemyProps_Tangible as i32
                | kEnemyProps_Invisible as i32)
            | kEnemyProps_Tangible as i32
            | kEnemyProps_Invisible as i32) as u16;
        slot!(game, Enemy_Phantoon[Phantoon].base.properties = v2);
        slot!(game, Enemy_Phantoon[0x80i32].base.properties = v2);
        slot!(game, Enemy_Phantoon[0xc0i32].base.properties = v2);
        QueueSfx2_Max6(0x7e, game);
    };
    [11000454] Phantoon_DyingFadeOut [] {
        actors::phantoon_actions::approach_speed((words!((g_word_A7CDA1, g_word_A7CD9F))) as i32, game);
        let E = (0x40) as i32;
        if slot!(game, Enemy_Phantoon[E].var_C) == 0xffff {
            Phantoon_FadeOut(0xc, game);
            if slot!(game, Enemy_Phantoon[E].var_F) != 0 {
                slot!(game, Enemy_Phantoon[k as i32].var_F = (0xa7dad7i32) as u16);
            }
        } else if (word!(game.nmi_frame_counter_word) & 0xf) == 0 {
            if slot!(byte game, Enemy_Phantoon[E].var_C[0]) == 0xf2 {
                slot!(game, Enemy_Phantoon[E].var_C = -1i32 as u16);
                slot!(game, Enemy_Phantoon[E].var_F = 0);
            } else {
                let v2: u8 = (slot!(byte game, Enemy_Phantoon[E].var_C[0])).wrapping_add(16);
                slot!(byte game, Enemy_Phantoon[E].var_C[0] = v2);
                match (v2, 0x57) { (value, at) => game.ram().write_byte(at, value) };
            }
        }
    };
    [11000535] Phantoon_AlmostDead [] {
        match (0, 0x57) { (value, at) => game.ram().write_byte(at, value) };
        let E1 = (0x40) as i32;
        slot!(game, Enemy_Phantoon[E1].parameter_1 = 0);
        word!(game.phantom_related_layer_flag &= !(0x4000));
        slot!(game, Enemy_Phantoon[0xc0i32].parameter_1 = -1i32 as u16);
        let EK = (k as i32) as i32;
        slot!(game, Enemy_Phantoon[EK].var_F = (0xa7db3di32) as u16);
        slot!(game, Enemy_Phantoon[EK].var_E = 60);
        slot!(game, Enemy_Phantoon[E1].var_F = 0);
        let E0 = (0) as i32;
        slot!(game, Enemy_Phantoon[E0].base.x_pos = 384);
        slot!(game, Enemy_Phantoon[E0].base.y_pos = 128);
        word_transfers!(game, descending, 512; le(tilemap_stuff, 511) <- value(824));
        let mut v5: u16 = word!(game.vram_write_queue_tail);
        match (1024, usize::try_from(((0xd0 + (word!(game.vram_write_queue_tail) as i32)) as i32) as isize + offset_of!(VramWriteEntry, size) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        v5 = (v5).wrapping_add(2);
        match (0x2000, usize::try_from(((0xd0 + (v5 as i32)) as i32) as isize + offset_of!(VramWriteEntry, size) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        v5 = (v5).wrapping_add(2);
        crate::update_place! { previous fresh19 = v5 => v5.wrapping_add(1); }
        match (126, usize::try_from(((0xd0 + (fresh19 as i32)) as i32) as isize + offset_of!(VramWriteEntry, size) as isize).expect("VRAM entry outside RAM")) { (value, at) => game.ram().write_byte(at, value) };
        match (18432, usize::try_from(((0xd0 + (v5 as i32)) as i32) as isize + offset_of!(VramWriteEntry, size) as isize).expect("VRAM entry outside RAM")) { (value, at) => { let bytes = u16::to_ne_bytes(value); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); } };
        word!(game.vram_write_queue_tail = (v5).wrapping_add(2));
    };
    [11000637] Phantoon_Dead [] {
        let EK = (k as i32) as i32;
        if slot!(game, Enemy_Phantoon[EK].var_E) != 0 {
            slot!(game, Enemy_Phantoon[EK].var_E = slot!(game, Enemy_Phantoon[EK].var_E).wrapping_sub(1));
        } else if (word!(game.nmi_frame_counter_word) & 3) == 0 {
            let E1 = (0x40) as i32;
            slot!(game, Enemy_Phantoon[E1].var_D = 12);
            if Phantoon_Func_9(game) & 1 != 0 {
                let value = game.ram().read_byte(0x69) | 2;
                game.ram().write_byte(0x69, value);
                scatter(DropSource::Phantoon, k, game);
                let E0 = (0) as i32;
                let v4: u16 = (slot!(game, Enemy_Phantoon[E0].base.properties)) | ((kEnemyProps_Deleted) as u16);
                slot!(game, Enemy_Phantoon[E0].base.properties = v4);
                slot!(game, Enemy_Phantoon[E1].base.properties = v4);
                slot!(game, Enemy_Phantoon[0x80i32].base.properties = v4);
                slot!(game, Enemy_Phantoon[0xc0i32].base.properties = v4);
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
            let E = (k as i32) as i32;
            if (slot!(game, Enemy_Etecoon[E].var_E) & 0x8000) == 0 {
                let v2: bool = slot!(game, Enemy_Etecoon[E].var_E) == 1;
                slot!(game, Enemy_Etecoon[E].var_E = slot!(game, Enemy_Etecoon[E].var_E).wrapping_sub(1));
                let v3: bool = (slot!(game, Enemy_Etecoon[E].var_E) & 0x8000) != 0;
                if v2 || v3 {
                    slot!(game, Enemy_Etecoon[E].base.current_instruction = addr_kEtecoon_Ilist_E854 as u16);
                    slot!(game, Enemy_Etecoon[E].var_F = (0xa7ea00i32) as u16);
                    slot!(game, Enemy_Etecoon[E].var_E = 11);
                }
            } else if ({ let (actor, distance) = (k, 0x80); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y, distance)) }) != 0 {
                if (slot!(game, Enemy_Etecoon[E].parameter_2) & 3) == 0 {
                    QueueSfx2_Max15(0x35, game);
                }
                slot!(game, Enemy_Etecoon[E].base; instruction_timer = 1, current_instruction = addr_kEtecoon_Ilist_E8D6 as u16);
                slot!(game, Enemy_Etecoon[E].var_E = 256);
            }
        }
    };
    [11004471] Etecoon_Func_6 [] {
        if Etecoon_Func_3(k, game) & 1 != 0 {
            let E = (k as i32) as i32;
            if (slot!(game, Enemy_Etecoon[E].var_A) & 0x8000) == 0 {
                if ({ let (actor, distance) = (k, 0x40); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y, distance)) }) != 0
                    && ({ let (actor, distance) = (k, g_word_A7E910); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0
                {
                    let v2: u16 = { match actor_api::direction_to_player(&mut Runtime::new(game)) { actor_api::PlayerDirection::Up=>0u16,actor_api::PlayerDirection::UpRight=>1u16,actor_api::PlayerDirection::Right=>2u16,actor_api::PlayerDirection::DownRight=>3u16,actor_api::PlayerDirection::Down=>4u16,actor_api::PlayerDirection::DownLeft=>6u16,actor_api::PlayerDirection::Left=>7u16,actor_api::PlayerDirection::UpLeft=>8u16 } };
                    if v2 as i32 - 5i32 & 0x8000 != 0 {
                        slot!(game, Enemy_Etecoon[E].base.current_instruction = addr_kEtecoon_Ilist_E81E as u16);
                        slot!(game, Enemy_Etecoon[E].parameter_1 = 0);
                    } else {
                        slot!(game, Enemy_Etecoon[E].base.current_instruction = addr_kEtecoon_Ilist_E876 as u16);
                        slot!(game, Enemy_Etecoon[E].parameter_1 = 1);
                    }
                    slot!(game, Enemy_Etecoon[E].var_E = 32);
                    slot!(game, Enemy_Etecoon[E].base.instruction_timer = 1);
                    slot!(game, Enemy_Etecoon[E].var_F = (0xa7eab5i32) as u16);
                } else {
                    slot!(game, Enemy_Etecoon[E].var_E = 11);
                    slot!(game, Enemy_Etecoon[E].var_F = (0xa7ea00i32) as u16);
                    slot!(game, Enemy_Etecoon[E].base; instruction_timer = 1, current_instruction = addr_kEtecoon_Ilist_E854 as u16);
                }
            } else {
                slot!(game, Enemy_Etecoon[E]; var_A = 0, var_B = 0);
                slot!(game, Enemy_Etecoon[E].base; instruction_timer = 3, current_instruction = addr_stru_A7E862 as u16);
            }
        }
    };
    [11004674] Etecoon_Func_8 [] {
        if Etecoon_Func_2(k, game) & 1 != 0 {
            let E = (k as i32) as i32;
            slot!(game, Enemy_Etecoon[E].var_C = g_word_A7E908);
            slot!(game, Enemy_Etecoon[E].var_D = g_word_A7E90A);
            slot!(game, Enemy_Etecoon[E].var_F = (0xa7eb2ci32) as u16);
            slot!(game, Enemy_Etecoon[E].base; instruction_timer = 1, current_instruction = addr_stru_A7E880 as u16);
            slot!(game, Enemy_Etecoon[E].parameter_1 = 1);
        }
    };
    [11004716] Etecoon_Func_9 [] {
        if ({ let (actor, amount) = (k, ((32u32) << 16 | 0u16 as u32) as i32); (actor_api::probe_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
            let E = (k as i32) as i32;
            slot!(game, Enemy_Etecoon[E].base; instruction_timer = 1, current_instruction = addr_stru_A7E898 as u16);
            slot!(game, Enemy_Etecoon[E].var_F = (0xa7eb50i32) as u16);
        } else {
            Etecoon_Func_2(k, game);
        };
    };
    [11004752] Etecoon_Func_10 [] {
        Etecoon_Func_1(k, game);
        if Etecoon_Func_2(k, game) & 1 != 0 {
            let E = (k as i32) as i32;
            if slot!(game, Enemy_Etecoon[E].parameter_1) != 0 {
                slot!(game, Enemy_Etecoon[E].base.current_instruction = addr_kEtecoon_Ilist_E870 as u16);
                slot!(game, Enemy_Etecoon[E].parameter_1 = 0);
            } else {
                slot!(game, Enemy_Etecoon[E].base.current_instruction = addr_kEtecoon_Ilist_E8C8 as u16);
                slot!(game, Enemy_Etecoon[E].parameter_1 = 1);
            }
            slot!(game, Enemy_Etecoon[E].base.instruction_timer = 1);
            slot!(game, Enemy_Etecoon[E].var_F = (0xa7ebcdi32) as u16);
            slot!(game, Enemy_Etecoon[E].var_E = 8);
            if word!(game.samus_x_pos) as i32 - 256i32 & 0x8000 == 0 {
                QueueSfx2_Max6(0x32, game);
            }
        } else if Etecoon_Func_3(k, game) & 1 != 0 {
            let E_0 = (k as i32) as i32;
            if slot!(game, Enemy_Etecoon[E_0].parameter_1) != 0 {
                slot!(game, Enemy_Etecoon[E_0].base.current_instruction = addr_kEtecoon_Ilist_E854 as u16);
            } else {
                slot!(game, Enemy_Etecoon[E_0].base.current_instruction = addr_kEtecoon_Ilist_E8AC as u16);
            }
            slot!(game, Enemy_Etecoon[E_0].base.instruction_timer = 1);
            slot!(game, Enemy_Etecoon[E_0]; var_E = 11, var_F = (0xa7ec1bi32) as u16, var_A = g_word_A7E900, var_B = g_word_A7E902);
        }
    };
    [11004877] Etecoon_Func_11 [] {
        Etecoon_Func_1(k, game);
        let E = (k as i32) as i32;
        let v2: bool = slot!(game, Enemy_Etecoon[E].var_E) == 1;
        slot!(game, Enemy_Etecoon[E].var_E = slot!(game, Enemy_Etecoon[E].var_E).wrapping_sub(1));
        let v3: bool = (slot!(game, Enemy_Etecoon[E].var_E) & 0x8000) != 0;
        if v2 || v3 {
            if slot!(game, Enemy_Etecoon[E].parameter_1) != 0 {
                slot!(game, Enemy_Etecoon[E].base.current_instruction = addr_kEtecoon_Ilist_E894 as u16);
                slot!(game, Enemy_Etecoon[E].var_C = g_word_A7E908);
                slot!(game, Enemy_Etecoon[E].var_D = g_word_A7E90A);
            } else {
                slot!(game, Enemy_Etecoon[E].base.current_instruction = addr_kEtecoon_Ilist_E83C as u16);
                slot!(game, Enemy_Etecoon[E].var_C = g_word_A7E90C);
                slot!(game, Enemy_Etecoon[E].var_D = g_word_A7E90E);
            }
            slot!(game, Enemy_Etecoon[E].base.instruction_timer = 1);
            slot!(game, Enemy_Etecoon[E].var_F = (0xa7eb50i32) as u16);
            slot!(game, Enemy_Etecoon[E].var_A = g_word_A7E900);
            slot!(game, Enemy_Etecoon[E].var_B = g_word_A7E902);
        }
    };
    [11005079] Etecoon_Func_16 [] {
        Etecoon_Func_2(k, game);
        let E = (k as i32) as i32;
        if slot!(game, Enemy_Etecoon[E].base.x_pos) as i32 - 537i32 & 0x8000 != 0 {
            slot!(game, Enemy_Etecoon[E].var_E = 11);
            slot!(game, Enemy_Etecoon[E].var_F = (0xa7edc7i32) as u16);
            slot!(game, Enemy_Etecoon[E].base; instruction_timer = 1, current_instruction = addr_kEtecoon_Ilist_E854 as u16);
        }
    };
    [11005115] Etecoon_Func_17 [] {
        Etecoon_Func_2(k, game);
        let E = (k as i32) as i32;
        if slot!(game, Enemy_Etecoon[E].base.x_pos) as i32 - 600i32 & 0x8000 == 0 {
            slot!(game, Enemy_Etecoon[E].var_E = 11);
            slot!(game, Enemy_Etecoon[E].var_F = (0xa7edc7i32) as u16);
            slot!(game, Enemy_Etecoon[E].base; instruction_timer = 1, current_instruction = addr_kEtecoon_Ilist_E854 as u16);
        }
    };
    [11005151] Etecoon_Func_18 [] {
        Etecoon_Func_2(k, game);
        let E = (k as i32) as i32;
        if slot!(game, Enemy_Etecoon[E].base.x_pos) as i32 - 600i32 & 0x8000 == 0 {
            slot!(game, Enemy_Etecoon[E].var_F = (0xa7ed09i32) as u16);
            slot!(game, Enemy_Etecoon[E].var_A = g_word_A7E904);
            slot!(game, Enemy_Etecoon[E].var_B = g_word_A7E906);
            slot!(game, Enemy_Etecoon[E].base; instruction_timer = 1, current_instruction = addr_stru_A7E898 as u16);
        }
    };
    [11005193] Etecoon_Func_19 [] {
        Etecoon_Func_2(k, game);
        Etecoon_Func_3(k, game);
        let E = (k as i32) as i32;
        if slot!(game, Enemy_Etecoon[E].base.x_pos) as i32 - 680i32 & 0x8000 == 0 {
            slot!(game, Enemy_Etecoon[E].base; instruction_timer = 1, current_instruction = addr_stru_A7E880 as u16);
            slot!(game, Enemy_Etecoon[E].var_F = (0xa7ed2ai32) as u16);
        }
    };
    [11005226] Etecoon_Func_20 [] {
        Etecoon_Func_2(k, game);
        let E = (k as i32) as i32;
        if slot!(game, Enemy_Etecoon[E].base.x_pos) as i32 - 840i32 & 0x8000 == 0 {
            slot!(game, Enemy_Etecoon[E].base; instruction_timer = 1, current_instruction = addr_stru_A7E898 as u16);
            slot!(game, Enemy_Etecoon[E].var_F = (0xa7ed54i32) as u16);
            slot!(game, Enemy_Etecoon[E].var_A = -1i32 as u16);
            slot!(game, Enemy_Etecoon[E].var_B = g_word_A7E906);
        }
    };
    [11005268] Etecoon_Func_21 [] {
        Etecoon_Func_2(k, game);
        if Etecoon_Func_3(k, game) & 1 != 0 {
            let E = (k as i32) as i32;
            slot!(game, Enemy_Etecoon[E].var_E = 11);
            slot!(game, Enemy_Etecoon[E].base; instruction_timer = 1, current_instruction = addr_kEtecoon_Ilist_E854 as u16);
            slot!(game, Enemy_Etecoon[E].var_F = (0xa7edc7i32) as u16);
        }
    };
    [11005301] Etecoon_Func_22 [] {
        Etecoon_Func_1(k, game);
        if Etecoon_Func_3(k, game) & 1 != 0 {
            let E = (k as i32) as i32;
            if (slot!(game, Enemy_Etecoon[E].var_A) & 0x8000) == 0 {
                slot!(game, Enemy_Etecoon[E].var_E = 11);
                slot!(game, Enemy_Etecoon[E].base; instruction_timer = 1, current_instruction = addr_kEtecoon_Ilist_E854 as u16);
                if (slot!(game, Enemy_Etecoon[E].parameter_2) & 2) != 0 && slot!(game, Enemy_Etecoon[E].base.x_pos) as i32 - 832i32 & 0x8000 != 0 {
                    slot!(game, Enemy_Etecoon[E].var_F = (0xa7ee3ei32) as u16);
                } else {
                    slot!(game, Enemy_Etecoon[E].var_F = (0xa7edc7i32) as u16);
                }
            } else {
                slot!(game, Enemy_Etecoon[E]; var_A = 0, var_B = 0);
                slot!(game, Enemy_Etecoon[E].base; instruction_timer = 3, current_instruction = addr_stru_A7E862 as u16);
            }
        }
    };
    [11005383] Etecoon_Func_23 [] {
        Etecoon_Func_1(k, game);
        let E = (k as i32) as i32;
        let v2: bool = slot!(game, Enemy_Etecoon[E].var_E) == 1;
        slot!(game, Enemy_Etecoon[E].var_E = slot!(game, Enemy_Etecoon[E].var_E).wrapping_sub(1));
        let v3: bool = (slot!(game, Enemy_Etecoon[E].var_E) & 0x8000) != 0;
        if v2 || v3 {
            let v4: u16 = (slot!(game, Enemy_Etecoon[E].parameter_1)).wrapping_add(256);
            slot!(game, Enemy_Etecoon[E].parameter_1 = v4);
            if (v4 as i32 & 0xff00) - 1024i32 & 0x8000 != 0 || {
                slot!(game, Enemy_Etecoon[E].parameter_1 = slot!(byte game, Enemy_Etecoon[E].parameter_1[0]) as u16);
                (slot!(game, Enemy_Etecoon[E].parameter_2) & 2) != 0
            } {
                slot!(game, Enemy_Etecoon[E].var_F = (0xa7ed75i32) as u16);
                slot!(game, Enemy_Etecoon[E].base.current_instruction = (slot!(game, Enemy_Etecoon[E].base.current_instruction)).wrapping_add(2));
            } else {
                slot!(game, Enemy_Etecoon[E].var_F = (0xa7ee9ai32) as u16);
                slot!(game, Enemy_Etecoon[E].base.current_instruction = addr_stru_A7E880 as u16);
                slot!(game, Enemy_Etecoon[E].var_C = g_word_A7E908);
                slot!(game, Enemy_Etecoon[E].var_D = g_word_A7E90A);
            }
            slot!(game, Enemy_Etecoon[E].var_A = g_word_A7E900);
            slot!(game, Enemy_Etecoon[E].var_B = g_word_A7E902);
            slot!(game, Enemy_Etecoon[E].base.instruction_timer = 1);
            if word!(game.samus_x_pos) as i32 - 256i32 & 0x8000 == 0 {
                QueueSfx2_Max6(0x33, game);
            }
        }
    };
    [11005502] Etecoon_Func_24 [] {
        Etecoon_Func_1(k, game);
        let E = (k as i32) as i32;
        let v2: bool = slot!(game, Enemy_Etecoon[E].var_E) == 1;
        slot!(game, Enemy_Etecoon[E].var_E = slot!(game, Enemy_Etecoon[E].var_E).wrapping_sub(1));
        let v3: bool = (slot!(game, Enemy_Etecoon[E].var_E) & 0x8000) != 0;
        if v2 || v3 {
            if ({ let (actor, distance) = (k, 0x40); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y, distance)) }) != 0 && ({ let (actor, distance) = (k, 0x30); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0
            {
                slot!(game, Enemy_Etecoon[E].base.current_instruction = addr_stru_A7E880 as u16);
                slot!(game, Enemy_Etecoon[E].var_F = (0xa7ecdfi32) as u16);
            } else {
                slot!(game, Enemy_Etecoon[E].var_A = g_word_A7E900);
                slot!(game, Enemy_Etecoon[E].var_B = g_word_A7E902);
                slot!(game, Enemy_Etecoon[E].base.current_instruction = (slot!(game, Enemy_Etecoon[E].base.current_instruction)).wrapping_add(2));
                slot!(game, Enemy_Etecoon[E].var_F = (0xa7ed75i32) as u16);
                if word!(game.samus_x_pos) as i32 - 256i32 & 0x8000 == 0 {
                    QueueSfx2_Max6(0x33, game);
                }
            }
            slot!(game, Enemy_Etecoon[E].base.instruction_timer = 1);
        }
    };
    [11005594] Etecoon_Func_25 [] {
        Etecoon_Func_2(k, game);
        let E = (k as i32) as i32;
        if slot!(game, Enemy_Etecoon[E].base.x_pos) as i32 - 600i32 & 0x8000 == 0 {
            slot!(game, Enemy_Etecoon[E].var_F = (0xa7eeb8i32) as u16);
            slot!(game, Enemy_Etecoon[E].base; instruction_timer = 1, current_instruction = addr_stru_A7E898 as u16);
        }
    };
    [11005624] Etecoon_Func_26 [] {
        Etecoon_Func_2(k, game);
        if Etecoon_Func_3(k, game) & 1 != 0 {
            let E = (k as i32) as i32;
            slot!(game, Enemy_Etecoon[E]; var_C = g_word_A7E90C, var_D = g_word_A7E90E, var_F = (0xa7eb02i32) as u16, var_A = g_word_A7E900, var_B = g_word_A7E902);
            slot!(game, Enemy_Etecoon[E].base; instruction_timer = 1, current_instruction = addr_stru_A7E828 as u16);
        }
    };
    [11007344] Dachora_Func_2 [] {
        ({ let (actor, amount) = (k, ((1u32) << 16 | 0u16 as u32) as i32); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 });
        if ({ let (actor, distance) = (k, 0x40); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::Y, distance)) }) != 0
            && ({ let (actor, distance) = (k, g_word_A7F4C9); u16::from(actor_api::player_within(&mut Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X, distance)) }) != 0
        {
            let E = (k as i32) as i32;
            if slot!(game, Enemy_Dachora[E].parameter_1) != 0 {
                slot!(game, Enemy_Dachora[E].base.current_instruction = addr_kDachora_Ilist_F48B as u16);
            } else {
                slot!(game, Enemy_Dachora[E].base.current_instruction = addr_kDachora_Ilist_F3C9 as u16);
            }
            slot!(game, Enemy_Dachora[E].base.instruction_timer = 1);
            slot!(game, Enemy_Dachora[E].var_F = (0xa7f5bci32) as u16);
            slot!(game, Enemy_Dachora[E].var_A = g_word_A7F4CD);
            QueueSfx2_Max15(0x1d, game);
        }
    };
    [11007420] Dachora_Func_3 [] {
        let E = (k as i32) as i32;
        let fresh2 = slot!(game, Enemy_Dachora[E].var_A); slot!(game, Enemy_Dachora[E].var_A = slot!(game, Enemy_Dachora[E].var_A).wrapping_sub(1));
        if fresh2 == 1 {
            if slot!(game, Enemy_Dachora[E].parameter_1) != 0 {
                slot!(game, Enemy_Dachora[E].base.current_instruction = addr_kDachora_Ilist_F407 as u16);
                slot!(game, Enemy_Dachora[E].var_F = (0xa7f65ei32) as u16);
            } else {
                slot!(game, Enemy_Dachora[E].base.current_instruction = addr_kDachora_Ilist_F345 as u16);
                slot!(game, Enemy_Dachora[E].var_F = (0xa7f5edi32) as u16);
            }
            slot!(game, Enemy_Dachora[E].base.instruction_timer = 1);
            slot!(game, Enemy_Dachora[E].var_E = 1);
        }
    };
    [11007469] Dachora_Func_4 [] {
        let amt: i32 = -Dachora_Func_6(k, game);
        let E = (k as i32) as i32;
        if ({ let (actor, amount) = (k, amt); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) != 0 || {
            ({ let actor = k; actor_api::snap_to_slopes(&mut Runtime::new(game), Key::from_legacy(actor)) as u8 });
            slot!(game, Enemy_Dachora[E].base.x_pos) as i32 - 96i32 & 0x8000 != 0
        } {
            slot!(game, Enemy_Dachora[E].base.current_instruction = addr_kDachora_Ilist_F407 as u16);
            slot!(game, Enemy_Dachora[E].var_F = (0xa7f65ei32) as u16);
            slot!(game, Enemy_Dachora[E]; var_E = 1, parameter_1 = 1);
            slot!(game, Enemy_Dachora[E].base.instruction_timer = 1);
            slot!(game, Enemy_Dachora[E]; var_A = 0, var_B = 0);
            Dachora_Func_1(addr_kDachora_Palette as u16, k, game);
        }
    };
    [11007582] Dachora_Func_5 [] {
        let amt: i32 = Dachora_Func_6(k, game);
        let E = (k as i32) as i32;
        if ({ let (actor, amount) = (k, amt); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
            QueueSfx2_Max15(0x71, game);
            slot!(game, Enemy_Dachora[E].base.current_instruction = addr_kDachora_Ilist_F345 as u16);
            slot!(game, Enemy_Dachora[E].var_F = (0xa7f5edi32) as u16);
            slot!(game, Enemy_Dachora[E]; parameter_1 = 0, var_A = 0);
            Dachora_Func_1(addr_kDachora_Palette as u16, k, game);
        } else {
            ({ let actor = k; actor_api::snap_to_slopes(&mut Runtime::new(game), Key::from_legacy(actor)) as u8 });
            if slot!(game, Enemy_Dachora[E].base.x_pos) as i32 - 1152i32 & 0x8000 == 0 {
                slot!(game, Enemy_Dachora[E].base.current_instruction = addr_kDachora_Ilist_F4B3 as u16);
                slot!(game, Enemy_Dachora[E].var_F = (0xa7f78fi32) as u16);
                slot!(game, Enemy_Dachora[E].var_A = g_word_A7F4CF);
                slot!(game, Enemy_Dachora[E].base.y_pos = (slot!(game, Enemy_Dachora[E].base.y_pos)).wrapping_add(8));
                QueueSfx2_Max6(0x3d, game);
            } else {
                return;
            }
        }
        slot!(game, Enemy_Dachora[E]; var_E = 0, var_B = 0);
        slot!(game, Enemy_Dachora[E].base.instruction_timer = 1);
    };
    [11007887] Dachora_Func_7 [] {
        Dachora_Func_10(k, game);
        let E = (k as i32) as i32;
        let fresh3 = slot!(game, Enemy_Dachora[E].var_A); slot!(game, Enemy_Dachora[E].var_A = slot!(game, Enemy_Dachora[E].var_A).wrapping_sub(1));
        if fresh3 == 1 {
            slot!(game, Enemy_Dachora[E].base; current_instruction = (slot!(game, Enemy_Dachora[E].base.current_instruction)).wrapping_add(2), instruction_timer = 1);
            slot!(game, Enemy_Dachora[E].var_F = (0xa7f806i32) as u16);
            let E1 = (k as i32 + 64) as i32;
            slot!(game, Enemy_Dachora[E1]; var_A = 0, var_D = 0);
            slot!(game, Enemy_Dachora[k as i32 + 128i32].var_D = 0);
            slot!(game, Enemy_Dachora[k as i32 + 192i32].var_D = 0);
            slot!(game, Enemy_Dachora[k as i32 + 256i32].var_D = 0);
            slot!(game, Enemy_Dachora[E]; var_C = 0, var_D = 0);
            slot!(game, Enemy_Dachora[E].base.y_pos = (slot!(game, Enemy_Dachora[E].base.y_pos)).wrapping_sub(8));
            QueueSfx2_Max6(0x3b, game);
            if slot!(game, Enemy_Dachora[E].parameter_1) != 0 {
                slot!(game, Enemy_Dachora[k as i32 + 64i32].base.current_instruction = addr_kDachora_Ilist_F4B9 as u16);
                slot!(game, Enemy_Dachora[k as i32 + 128i32].base.current_instruction = addr_kDachora_Ilist_F4B9 as u16);
                slot!(game, Enemy_Dachora[k as i32 + 192i32].base.current_instruction = addr_kDachora_Ilist_F4B9 as u16);
                slot!(game, Enemy_Dachora[k as i32 + 256i32].base.current_instruction = addr_kDachora_Ilist_F4B9 as u16);
            } else {
                slot!(game, Enemy_Dachora[k as i32 + 64i32].base.current_instruction = addr_kDachora_Ilist_F3F7 as u16);
                slot!(game, Enemy_Dachora[k as i32 + 128i32].base.current_instruction = addr_kDachora_Ilist_F3F7 as u16);
                slot!(game, Enemy_Dachora[k as i32 + 192i32].base.current_instruction = addr_kDachora_Ilist_F3F7 as u16);
                slot!(game, Enemy_Dachora[k as i32 + 256i32].base.current_instruction = addr_kDachora_Ilist_F3F7 as u16);
            }
            slot!(game, Enemy_Dachora[k as i32 + 64i32].base.instruction_timer = 1);
            slot!(game, Enemy_Dachora[k as i32 + 128i32].base.instruction_timer = 1);
            slot!(game, Enemy_Dachora[k as i32 + 192i32].base.instruction_timer = 1);
            slot!(game, Enemy_Dachora[k as i32 + 256i32].base.instruction_timer = 1);
        }
    };
    [11008006] Dachora_Func_8 [] {
        Dachora_Func_10(k, game);
        Dachora_Func_9(k, game);
        let E = (k as i32) as i32;
        slot!(game, Enemy_Dachora[E](var_C, var_D) += words!(game.samus_y_accel, samus_y_subaccel));
        slot!(game, Enemy_Dachora[E](var_A, var_B) += words!((slot!(game, Enemy_Dachora[E].var_C), slot!(game, Enemy_Dachora[E].var_D))));
        let mut v6: u16 = slot!(game, Enemy_Dachora[E].var_A);
        if v6 as i32 - 15i32 & 0x8000 == 0 {
            v6 = 15;
        }
        let amt: i32 = (words!((v6, slot!(game, Enemy_Dachora[E].var_B)))) as i32;
        if ({ let (actor, amount) = (k, -amt); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
            if slot!(game, Enemy_Dachora[E].parameter_1) != 0 {
                slot!(game, Enemy_Dachora[E].base.current_instruction = addr_kDachora_Ilist_F3FF as u16);
                slot!(game, Enemy_Dachora[E].parameter_1 = 0);
            } else {
                slot!(game, Enemy_Dachora[E].base.current_instruction = addr_kDachora_Ilist_F4C1 as u16);
                slot!(game, Enemy_Dachora[E].parameter_1 = 1);
            }
            slot!(game, Enemy_Dachora[E].var_F = (0xa7f935i32) as u16);
            slot!(game, Enemy_Dachora[E].base.instruction_timer = 1);
            slot!(game, Enemy_Dachora[E]; var_A = 0, var_B = 0, var_E = 0);
            Dachora_Func_1(addr_kDachora_Palette as u16, k, game);
            QueueSfx2_Max6(0x3c, game);
        }
    };
    [11008309] Dachora_Func_11 [] {
        let E = (k as i32) as i32;
        slot!(game, Enemy_Dachora[E](var_A, var_B) += words!(game.samus_y_accel, samus_y_subaccel));
        let mut amt: i32 = (words!((slot!(game, Enemy_Dachora[E].var_A), slot!(game, Enemy_Dachora[E].var_B)))) as i32;
        if (amt >> 16) - 10i32 & 0x8000 == 0 {
            amt = (10i16 as i32) << 16;
        }
        if ({ let (actor, amount) = (k, amt); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
            if slot!(game, Enemy_Dachora[E].parameter_1) != 0 {
                slot!(game, Enemy_Dachora[E].base.current_instruction = addr_kDachora_Ilist_F407 as u16);
                slot!(game, Enemy_Dachora[E].var_F = (0xa7f65ei32) as u16);
            } else {
                slot!(game, Enemy_Dachora[E].base.current_instruction = addr_kDachora_Ilist_F345 as u16);
                slot!(game, Enemy_Dachora[E].var_F = (0xa7f5edi32) as u16);
            }
            slot!(game, Enemy_Dachora[E].base.instruction_timer = 1);
            slot!(game, Enemy_Dachora[E]; var_A = 0, var_B = 0);
        }
    };
    [11008396] Dachora_Func_12 [] {
        let E = (k as i32) as i32;
        let dachor_var_D: u16 = slot!(game, Enemy_Dachora[E].var_D);
        if dachor_var_D != 0 {
            slot!(game, Enemy_Dachora[E].var_D = (dachor_var_D).wrapping_sub(1));
            let keep_hidden: bool = if (k & 0x40) != 0 {
                (word!(game.nmi_frame_counter_word) & 1) == 0
            } else {
                (word!(game.nmi_frame_counter_word) & 1) != 0
            };
            if keep_hidden {
            } else {
                slot!(game, Enemy_Dachora[E].base.properties = (slot!(game, Enemy_Dachora[E].base.properties) as i32 & !(kEnemyProps_Invisible as i32)) as u16);
                return;
            }
        }
        slot!(game, Enemy_Dachora[E].base.properties = (slot!(game, Enemy_Dachora[E].base.properties)) | ((kEnemyProps_Invisible) as u16));
    };
    @functions [pub unsafe fn (v0: u16, game: &mut Game)];
    [10999388] Phantoon_BecomesSolidAndBodyVuln [] {
        Phantoon_ChangeEyeSpriteBasedOnSamusDist(game);
        word!(game.phantom_related_layer_flag &= !(0x4000));
        Phantoon_BeginSwoopingPattern(v0, game);
        let E = (0) as i32;
        slot!(game, Enemy_Phantoon[E].base; instruction_timer = 1, current_instruction = addr_kKraid_Ilist_CC47 as u16);
    };
    @countdown [pub unsafe fn (k: u16, game: &mut Game)];
    before {
        let EK = (k as i32) as i32;
        let v2: bool = slot!(game, Enemy_Phantoon[EK].var_E) == 1;
        slot!(game, Enemy_Phantoon[EK].var_E = slot!(game, Enemy_Phantoon[EK].var_E).wrapping_sub(1));
        let v3: bool = (slot!(game, Enemy_Phantoon[EK].var_E) & 0x8000) != 0;
    }
    when v2 || v3;
    [10998953] Phantoon_Spawn8FireballsInCircleAtStart {
        let E0 = (0) as i32;
        SpawnEprojWithGfx(slot!(game, Enemy_Phantoon[E0].var_A), k, addr_kEproj_StartingFireballs as u16, game);
        QueueSfx3_Max6(0x1d, game);
        slot!(game, Enemy_Phantoon[EK].var_E = 30);
        let v5: u16 = (slot!(game, Enemy_Phantoon[E0].var_A)).wrapping_add(1);
        slot!(game, Enemy_Phantoon[E0].var_A = v5);
        if v5 as i32 - 8i32 & 0x8000 == 0 {
            slot!(game, Enemy_Phantoon[E0].var_A = 0);
            slot!(game, Enemy_Phantoon[k as i32 + 128i32].var_B = 0);
            slot!(game, Enemy_Phantoon[EK].var_F = (0xa7d4eei32) as u16);
            slot!(game, Enemy_Phantoon[EK].var_E = 30);
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0, 0x6, 0xb781 as VoidP), game);
        }
    };
    [10999022] Phantoon_WaitBetweenSpawningAndSpinningFireballs {
        slot!(game, Enemy_Phantoon[EK].var_E = 240);
        slot!(game, Enemy_Phantoon[EK].var_B = 1);
        slot!(game, Enemy_Phantoon[EK].var_F = (0xa7d508i32) as u16);
    };
    [10999048] Phantoon_SpawnFireballsBeforeFight {
        slot!(game, Enemy_Phantoon[EK].var_B = 0);
        word!(game.phantom_related_layer_flag |= (0x4000u16));
        slot!(game, Enemy_Phantoon[EK].var_F = (0xa7d54ai32) as u16);
        let E3 = (0xc0) as i32;
        slot!(game, Enemy_Phantoon[E3].parameter_1 = -32767i32 as u16);
        slot!(game, Enemy_Phantoon[EK].var_E = 120);
        initialize_phantoon_motion(2, g_word_A7CDA3, game);
        slot!(game, Enemy_Phantoon[E3].var_D = g_word_A7CD9D);
        slot!(game, Enemy_Phantoon[0x40i32].var_F = 0);
        QueueMusic_Delayed8(5, game);
    };
    [10999190] Phantoon_PickPatternForRound1 {
        let E1 = (0x40) as i32;
        slot!(game, Enemy_Phantoon[E1].parameter_1 = 0);
        slot!(game, Enemy_Phantoon[EK].var_F = (0xa7d5e7i32) as u16);
        slot!(game, Enemy_Phantoon[E1].var_A = crate::rtl::rom_table_word!(
            game,
            0xa7cd53,
            (word!(game.nmi_frame_counter_word) as i32 >> 1 & 3) as isize
        ));
        if (game.next_random() & 1) != 0 {
            let E = (0) as i32;
            slot!(game, Enemy_Phantoon[E]; var_C = 0, var_B = 0, var_D = 0);
            slot!(game, Enemy_Phantoon[E1].var_C = 1);
            slot!(game, Enemy_Phantoon[E].var_A = 533);
        } else {
            let E_0 = (0) as i32;
            slot!(game, Enemy_Phantoon[E_0].var_C = 1);
            slot!(game, Enemy_Phantoon[E_0]; var_B = 0, var_D = 0);
            slot!(game, Enemy_Phantoon[E1].var_C = 0);
            slot!(game, Enemy_Phantoon[E_0].var_A = 0);
        }
    };
    [10999508] Phantoon_WaitAfterFadeOut {
        slot!(game, Enemy_Phantoon[EK].var_F = (0xa7d6e2i32) as u16);
    };
    [10999688] Phantoon_FollowSamusWithEyeDuringFireballRain {
        slot!(game, Enemy_Phantoon[k as i32 + 128i32].var_B = 0);
        let E2 = (0x80) as i32;
        if slot!(game, Enemy_Phantoon[E2].var_A) != 0 {
            slot!(game, Enemy_Phantoon[E2].var_A = 0);
            slot!(game, Enemy_Phantoon[0i32].parameter_2 = 1);
            Phantoon_BeginSwoopingPattern(k, game);
        } else {
            slot!(game, Enemy_Phantoon[EK].var_F = (0xa7d7d5i32) as u16);
            let E1 = (0x40) as i32;
            slot!(game, Enemy_Phantoon[E1].var_F = 0);
            let E0 = (0) as i32;
            slot!(game, Enemy_Phantoon[E0].base.instruction_timer = 1);
            slot!(game, Enemy_Phantoon[E1].base.instruction_timer = 1);
            slot!(game, Enemy_Phantoon[E0].base.current_instruction = addr_kKraid_Ilist_CC41 as u16);
            slot!(game, Enemy_Phantoon[E1].base.current_instruction = addr_kKraid_Ilist_CC91 as u16);
            slot!(game, Enemy_Phantoon[E0].base.properties = (slot!(game, Enemy_Phantoon[E0].base.properties)) | ((kEnemyProps_Tangible) as u16));
            word!(game.phantom_related_layer_flag |= (0x4000u16));
        }
    };
    [10999799] Phantoon_SpawnRainingFireballs {
        let a: u16 = (game.next_random()) & (7u16);
        let v4: i32 = 8 * a as i32 >> 1;
        let E0 = (0) as i32;
        slot!(game, Enemy_Phantoon[E0].var_A = crate::rtl::rom_table_word!(game, 0xa7cdad, v4 as isize));
        slot!(game, Enemy_Phantoon[E0].base.x_pos = crate::rtl::rom_table_word!(game, 0xa7cdad, (v4 + 1) as isize));
        slot!(game, Enemy_Phantoon[E0].base.y_pos = crate::rtl::rom_table_word!(game, 0xa7cdad, (v4 + 2) as isize));
        slot!(game, Enemy_Phantoon[0x40i32].var_C = 0);
        slot!(game, Enemy_Phantoon[EK].var_F = (0xa7d73fi32) as u16);
        actors::phantoon_actions::spawn_fireball_pattern(k, a, game);
    };
    [10999924] Phantoon_MoveEnragedPhantoonToTopCenter {
        slot!(game, Enemy_Phantoon[EK].var_F = (0xa7d891i32) as u16);
        let E_0 = (0) as i32;
        slot!(game, Enemy_Phantoon[E_0].base.x_pos = 128);
        slot!(game, Enemy_Phantoon[E_0].base.y_pos = 32);
        slot!(game, Enemy_Phantoon[0x40i32].var_F = 0);
    };
    [11000203] Phantoon_DyingPhantoonExplosions {
        let E2 = (0x80) as i32;
        let v5: u16 = (4u16).wrapping_mul(slot!(game, Enemy_Phantoon[E2].var_F));
        let E0 = (0) as i32;
        game.eproj_spawn_pt = Point16U {
            x: (((byte!(@pointer rom_ptr!(game; 0xa7da1d); v5 as isize) as i8) as u16)
                .wrapping_add(slot!(game, Enemy_Phantoon[E0].base.x_pos))),
            y: (((byte!(@pointer rom_ptr!(game; 0xa7da1d); (v5 as i32 + 1i32) as isize) as i8) as u16)
                .wrapping_add(slot!(game, Enemy_Phantoon[E0].base.y_pos))),
        };
        let v11: u16 = byte!(@pointer rom_ptr!(game; 0xa7da1d); (v5 as i32 + 2) as isize) as u16;
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, v11, game);
        if v11 == 29 {
            QueueSfx2_Max6(0x24, game);
        } else {
            QueueSfx2_Max6(0x2b, game);
        }
        slot!(game, Enemy_Phantoon[EK].var_E = byte!(@pointer rom_ptr!(game; 0xa7da1d); (v5 as i32 + 3) as isize) as u16);
        let v9: u16 = (slot!(game, Enemy_Phantoon[E2].var_F)).wrapping_add(1);
        slot!(game, Enemy_Phantoon[E2].var_F = v9);
        if v9 as i32 - 13i32 & 0x8000 == 0 {
            slot!(game, Enemy_Phantoon[E2].var_F = 5);
            let v10: u16 = (slot!(game, Enemy_Phantoon[E0].var_A)).wrapping_add(1);
            slot!(game, Enemy_Phantoon[E0].var_A = v10);
            if v10 as i32 - 3i32 & 0x8000 == 0 {
                slot!(game, Enemy_Phantoon[EK].var_F = (0xa7da51i32) as u16);
            }
        }
    };
    @countdown [pub unsafe fn (k: u16, game: &mut Game)];
    before {
        let E = (k as i32) as i32;
        let v2: bool = slot!(game, Enemy_Etecoon[E].var_E) == 1;
        slot!(game, Enemy_Etecoon[E].var_E = slot!(game, Enemy_Etecoon[E].var_E).wrapping_sub(1));
        let v3: bool = (slot!(game, Enemy_Etecoon[E].var_E) & 0x8000) != 0;
    }
    when v2 || v3;
    [11004416] Etecoon_Func_5 {
        slot!(game, Enemy_Etecoon[E].var_A = g_word_A7E900);
        slot!(game, Enemy_Etecoon[E].var_B = g_word_A7E902);
        slot!(game, Enemy_Etecoon[E].base; current_instruction = (slot!(game, Enemy_Etecoon[E].base.current_instruction)).wrapping_add(2), instruction_timer = 1);
        slot!(game, Enemy_Etecoon[E].var_F = (0xa7ea37i32) as u16);
        if word!(game.samus_x_pos) as i32 - 256i32 & 0x8000 == 0 {
            QueueSfx2_Max6(0x33, game);
        }
    };
    [11004597] Etecoon_Func_7 {
        slot!(game, Enemy_Etecoon[E].base; current_instruction = (slot!(game, Enemy_Etecoon[E].base.current_instruction)).wrapping_add(2), instruction_timer = 1);
        if slot!(game, Enemy_Etecoon[E].parameter_1) != 0 {
            slot!(game, Enemy_Etecoon[E].var_C = g_word_A7E908);
            slot!(game, Enemy_Etecoon[E].var_D = g_word_A7E90A);
            slot!(game, Enemy_Etecoon[E].var_F = (0xa7eb2ci32) as u16);
        } else {
            slot!(game, Enemy_Etecoon[E].var_C = g_word_A7E90C);
            slot!(game, Enemy_Etecoon[E].var_D = g_word_A7E90E);
            slot!(game, Enemy_Etecoon[E].var_F = (0xa7eb02i32) as u16);
        }
        slot!(game, Enemy_Etecoon[E].var_A = g_word_A7E900);
        slot!(game, Enemy_Etecoon[E].var_B = g_word_A7E902);
    };
    [11004955] Etecoon_Func_12 {
        slot!(game, Enemy_Etecoon[E].var_A = g_word_A7E900);
        slot!(game, Enemy_Etecoon[E].var_B = g_word_A7E902);
        funcs_A2A79[slot!(byte game, Enemy_Etecoon[E].parameter_2[0]) as usize](k, game);
        slot!(game, Enemy_Etecoon[k as i32].base.instruction_timer = 1);
    };
}
