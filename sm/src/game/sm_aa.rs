use crate::game::actors;
use crate::game::actors::access::ActorKey as Key;
use crate::game::actors::api as actor_api;
pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_84::*;
pub use crate::game::constants::bank_86::*;
use crate::game::enemy_projectiles::{DropSource, scatter_drop as scatter};
use crate::game::enemy_script::{EnemyScript, bank_base};
use crate::game::gameplay_flag_host::BossWordOrder;
use crate::game::gameplay_flag_runtime;
use crate::game::music::QueueMusic_Delayed8;
pub use crate::game::prelude::*;
use crate::game::session_runtime::{Runtime, actor_species_pre_instruction};
use crate::game::shaktool_tables;
use crate::game::statue_tables;
use crate::game::torizo_tables;
use crate::ram_map::slot;

use crate::game::gameplay_flag_runtime::{CheckBossBitForCurArea, SetBossBitForCurArea};
use crate::game::music::QueueSfx2_Max6;
use crate::game::palette_effects::fading::selected;
use crate::game::room_mechanisms::{spawn_collision, spawn_hardcoded};

use crate::game::enemy_projectiles::{SpawnEprojWithGfx, SpawnEprojWithRoomGfx};
use crate::game::palette_effects::{self, torizo};
use crate::game::session_runtime::player_command as CallSomeSamusCode;
use crate::game::visual_runtime::initialize_bomb_torizo_haze;

pub use crate::types::Consts_AA;

crate::game::constants::typed_constants! { Consts_AA;
    addr_kShaktool_Ilist_E461 = 58465, addr_kShaktool_Ilist_E3A7 = 58279,
    addr_kTorizo_Ilist_D2BF = 53951, addr_kTorizo_Ilist_D2AD = 53933,
    addr_kTorizo_Ilist_D203 = 53763, addr_kTorizo_Ilist_D1F1 = 53745,
    addr_kTorizo_Ilist_CEFF = 52991, addr_kTorizo_Ilist_CEA5 = 52901,
    addr_kTorizo_Ilist_CE43 = 52803, addr_kTorizo_Ilist_CDE1 = 52705,
    addr_locret_AAC95E = 51550, addr_kTorizo_Ilist_C188 = 49544,
    addr_off_AAC0F2 = 49394, addr_kTorizo_Ilist_BDD8 = 48600,
    addr_kTorizo_Ilist_BD0E = 48398, addr_off_AABC78 = 48248,
    addr_kTorizo_Ilist_B962 = 47458, addr_kTorizo_Ilist_B1C8 = 45512,
    addr_kTorizo_Ilist_B155 = 45397, addr_kTorizo_Ilist_B0E5 = 45285,
    addr_kTorizo_ExtSprmap_87D0 = 34768, addr_kSpritemap_Nothing_AA = 32845,
}

pub type Enemy_Shaktool = crate::game::enemy_layouts::EnemyLayout8;

pub type Enemy_Torizo = crate::game::enemy_layouts::EnemyLayout40;

crate::game::constants::typed_constants! { i32;
    fnnullsub_171_AA = 0xaa807b,
    fnnullsub_274 = 0xaadcaa, fnnullsub_276 = 0xaae7a6,
}
static g_word_AAB098: u16 = 5;
static g_word_AAB09A: u16 = 3;
static g_word_AAC3EE: [i16; 16] = [-9, -6, -7, 5, -16, -7, 0, 0, 9, 6, 7, -5, 16, 7, 0, 0];
static g_word_AAC40E: [i16; 8] = [0, -6, -6, -7, 0, 0, 0, 0];
static g_word_AAC440: [i16; 16] = [-9, -6, -7, 5, -16, -7, 0, 0, 9, 6, 7, -5, 16, 7, 0, 0];
static g_word_AAC460: [i16; 8] = [0, -6, -6, -7, 0, 0, 0, 0];
static g_word_AAC95F: [i16; 2] = [0xdb, 0x1a8];
static g_word_AAC963: [i16; 2] = [0xb3, 0x90];
static g_word_AAC96B: [i16; 2] = [0x2800, 0x2800];
static g_word_AAC96F: [i16; 2] = [0x12, 0x12];
static g_word_AAC973: [i16; 2] = [0x30, 0x29];
static g_word_AADEE9: [u16; 7] = [0, 0x20, 0x60, 0xc0, 0x140, 0x1a0, 0x1e0];
static kChozoStatue_Palette: [u16; 16] = [
    0x3800, 0x633f, 0x4a9f, 0x2ddf, 0x6739, 0x4e73, 0x318c, 0x18c6, 0x27ff, 0x1af7, 0xdce, 0xc6,
    0x3fff, 0x2b39, 0x7fff, 0,
];
static kChozoStatue_Palettes: [u16; 16] = [
    0x3800, 0x633f, 0x4a9f, 0x2ddf, 0x4210, 0x318c, 0x2108, 0x1084, 0x27ff, 0x1af7, 0xdce, 0xc6,
    0x3fff, 0x2b39, 0x5294, 0,
];
static kChozoStatue_Palettes2: [u16; 16] = [
    0x3800, 0x633f, 0x4a9f, 0x2ddf, 0x2f7c, 0x2295, 0x118d, 0x8e8, 0x27ff, 0x1af7, 0xdce, 0xc6,
    0x3fff, 0x2b39, 0x73df, 0x43,
];
static kChozoStatue_Palettes3: [u16; 16] = [
    0x3800, 0x633f, 0x4a9f, 0x2ddf, 0x2295, 0x118d, 0x8e8, 0x85, 0x27ff, 0x1af7, 0xdce, 0xc6,
    0x3fff, 0x2b39, 0x5294, 1,
];
pub fn Torizo_Instr_7(_k: u16, jp: EnemyScript) -> EnemyScript {
    jp
}
macro_rules! torizo_fall_until_timer {
    (
        $enemy:ident, $enemy_index:ident, $game:ident, $facing_left_instruction:ident,
        $facing_right_instruction:ident $(,)?
    ) => {{
        let torizo_var_03: u16 = slot!($game, Enemy_Torizo[$enemy].var_03);
        if torizo_var_03 == 0 || {
            let mut v8: u16 = (torizo_var_03).wrapping_sub(1);
            slot!($game, Enemy_Torizo[$enemy].var_03 = v8);
            slot!($game, Enemy_Torizo[$enemy].var_03) != 0
        } {
            let mut v10: u16 = (abs16(slot!($game, Enemy_Torizo[$enemy].var_A))).wrapping_add(1);
            if v10 >= 0x10 {
                v10 = 15;
            }
            if {
                let (actor, amount) = ($enemy_index, (v10 as i16 as i32) << 16);
                (actor_api::move_vertical(
                    &mut Runtime::new($game),
                    Key::from_legacy(actor),
                    amount,
                ) == actor_api::MovementResult::Blocked) as u8
            } == 0
            {
                let mut v11: u16 = if (slot!($game, Enemy_Torizo[$enemy].parameter_1) & 0x8000) != 0
                {
                    addr_off_AAC0F2 as u16
                } else {
                    addr_off_AABC78 as u16
                };
                slot!($game, Enemy_Torizo[$enemy].base.current_instruction = v11);
                slot!($game, Enemy_Torizo[$enemy].base.instruction_timer = 1);
                slot!($game, Enemy_Torizo[$enemy].var_B = 256);
                slot!($game, Enemy_Torizo[$enemy].var_A = 0);
            }
        } else {
            slot!($game, Enemy_Torizo[$enemy].var_03 = 0);
            let mut v9: u16 = if (slot!($game, Enemy_Torizo[$enemy].parameter_1) & 0x8000) != 0 {
                $facing_left_instruction as u16
            } else {
                $facing_right_instruction as u16
            };
            slot!($game, Enemy_Torizo[$enemy].base.current_instruction = v9);
            slot!($game, Enemy_Torizo[$enemy].base.instruction_timer = 1);
        }
    }};
}

crate::game::callback_dispatch! {
    pub fn CallTorizoFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub fn (k: u16, game: &mut Game)];
    [11191999] Torizo_Func_3 [] {
        actors::torizo::drool_at_low_health(k, game);
        Torizo_C643(k, game);
    };
    [11192063] Torizo_Func_1 [] {
        actors::torizo::drool_at_low_health(k, game);
        let E: i32 = k as i32;
        if (slot!(game, Enemy_Torizo[E].parameter_2) & 0x8000) != 0 || slot!(game, Enemy_Torizo[E].base.health) >= 0x15e {
            if (slot!(game, Enemy_Torizo[E].parameter_2) & 0x4000) != 0 || slot!(game, Enemy_Torizo[E].base.health) >= 0x64 {
                unsafe { actor_species_pre_instruction(((slot!(game, Enemy_Torizo[E].var_F)) as u32) | (0xaa0000u32), game) };
            } else {
                let mut v2: u16 = if (slot!(game, Enemy_Torizo[E].parameter_1) & 0x8000) != 0 {
                    addr_kTorizo_Ilist_BD0E as u16
                } else {
                    addr_kTorizo_Ilist_C188 as u16
                };
                slot!(game, Enemy_Torizo[E].var_00 = v2);
                slot!(game, Enemy_Torizo[E].base.current_instruction = addr_kTorizo_Ilist_B155 as u16);
                slot!(game, Enemy_Torizo[E].base.instruction_timer = 1);
            }
        } else {
            slot!(game, Enemy_Torizo[E].var_01 = slot!(game, Enemy_Torizo[E].base.current_instruction));
            slot!(game, Enemy_Torizo[E].base.current_instruction = addr_kTorizo_Ilist_B0E5 as u16);
            slot!(game, Enemy_Torizo[E].base.instruction_timer = 1);
        };
    };
    @functions [pub fn (_k: u16, game: &mut Game)];
    [11195842] Torizo_Func_9 [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        if word!(game.samus_y_pos) > 0x140 && word!(game.samus_x_pos) > 0x170 {
            slot!(game, Enemy_Torizo[E].base.current_instruction = slot!(game, Enemy_Torizo[E].base.current_instruction).wrapping_add(1));
            slot!(game, Enemy_Torizo[E].base.current_instruction = slot!(game, Enemy_Torizo[E].base.current_instruction).wrapping_add(1));
            slot!(game, Enemy_Torizo[E].base.instruction_timer = 1);
        }
    };
    @functions [pub fn (k: u16, game: &mut Game)];
    [11195878] Torizo_D5E6 [] {
        actors::torizo::drool_at_low_health(k, game);
        let E: i32 = k as i32;
        unsafe { actor_species_pre_instruction(((slot!(game, Enemy_Torizo[E].var_F)) as u32) | (0xaa0000u32), game) };
    };
    @functions [pub fn (k: u16, game: &mut Game)];
    @prefix { let E: i32 = k as i32; };
    Torizo_C20A [] {
        let mut v2: i16 = if (slot!(game, Enemy_Torizo[E].parameter_1) & 0x8000) != 0 {
            512
        } else {
            -512i16
        };
        slot!(game, Enemy_Torizo[E].var_A = v2 as u16);
        slot!(game, Enemy_Torizo[E].var_B = -1472i32 as u16);
        slot!(game, Enemy_Torizo[E].var_C = 40);
        slot!(game, Enemy_Torizo[E].base.instruction_timer = 1);
    };
    Torizo_C22D [] {
        let mut v2: i16 = if (slot!(game, Enemy_Torizo[E].parameter_1) & 0x8000) != 0 {
            -768i16
        } else {
            768
        };
        slot!(game, Enemy_Torizo[E].var_A = v2 as u16);
        slot!(game, Enemy_Torizo[E].var_B = -1152i32 as u16);
        slot!(game, Enemy_Torizo[E].var_C = 40);
        slot!(game, Enemy_Torizo[E].base.instruction_timer = 1);
    };
    Torizo_C643 [] {
        if ({ let (actor, amount) = (k, (slot!(game, Enemy_Torizo[E].var_B) as i16 as i32) << 8); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
            let v3: i16 = slot!(game, Enemy_Torizo[E].var_B) as i16;
            if v3 >= 0 && v3 != 256 {
                word! {game; earthquake_type = 4; earthquake_timer = 32; }
                slot!(game, Enemy_Torizo[E].var_B = 256);
            }
        } else {
            slot!(game, Enemy_Torizo[E].var_B = (slot!(game, Enemy_Torizo[E].var_B)).wrapping_add(40));
        };
    };
    Torizo_Func_5 [] {
        if (slot!(game, Enemy_Torizo[E].parameter_2) & 0x4000) != 0 {
            torizo_fall_until_timer!(E, k, game, addr_kTorizo_Ilist_BD0E, addr_kTorizo_Ilist_C188);
        } else {
            torizo_fall_until_timer!(E, k, game, addr_kTorizo_Ilist_B962, addr_kTorizo_Ilist_BDD8);
        };
    };
    Torizo_Func_7 [] {
        ({ let (actor, amount) = (k, (slot!(game, Enemy_Torizo[E].var_A) as i16 as i32) << 8); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 });
        ({ let actor = k; actor_api::snap_to_slopes(&mut Runtime::new(game), Key::from_legacy(actor)) as u8 });
        if ({ let (actor, amount) = (k, (slot!(game, Enemy_Torizo[E].var_B) as i16 as i32) << 8); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0 {
            slot!(game, Enemy_Torizo[E].base.current_instruction = slot!(game, Enemy_Torizo[E].var_00));
            slot!(game, Enemy_Torizo[E].base.instruction_timer = 1);
            slot!(game, Enemy_Torizo[E].var_B = 256);
            word! {game; earthquake_type = 4; earthquake_timer = 32; }
        } else {
            slot!(game, Enemy_Torizo[E].var_B = (slot!(game, Enemy_Torizo[E].var_B)).wrapping_add(slot!(game, Enemy_Torizo[E].var_C)));
        };
    };
    Torizo_Func_11 [] {
        torizo::health_current(slot!(game, Enemy_Torizo[E].base.health), game);
    };
    Torizo_D5F1 [] {
        torizo_fall_until_timer!(E, k, game, addr_kTorizo_Ilist_D203, addr_kTorizo_Ilist_D2BF);
    };
    @external {
        11192006 => [crate::game::room_mechanisms::wait_statue(k, game)];
        11191979 => [()];
        11195871 => Torizo_Func_10;
    }
}
pub fn Torizo_Func_8(game: &mut Game) {
    {
        if word!(game.area_index) != 0 {
            Torizo_D658(game);
        }
    }
}
pub use self::Torizo_Func_3 as Torizo_Func_10;
pub use self::Torizo_Instr_7 as Shaktool_Instr_6;
pub fn sub_AAE784(game: &mut Game) {
    {
        word_transfers!(game, descending, 16;
            le(target_palettes, 175) <- table(kChozoStatue_Palettes3, 15),
            le(target_palettes, 159) <- table(kChozoStatue_Palettes2, 15),
        );
        spawn_hardcoded(SpawnHardcodedPlmArgs::new(0xc, 0x1d, 0xd6d6 as VoidP), game);
    }
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    [11191934] Torizo_Hurt [] {
        actors::torizo::drool_at_low_health(word!(game.cur_enemy_index), game);
        if slot!(game, Enemy_Torizo[word!(game.cur_enemy_index) as i32].base.flash_timer)
            & 1
            != 0
        {
            torizo::flash_current(game);
        } else {
            torizo::restore_current(game);
        };
    };
    [11192695] GoldTorizo_Touch [] {
        actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
    };
    [11192700] Torizo_Shot [] {
        if word!(game.area_index) != 0 {
            GoldTorizo_Shot(game);
        } else {
            let E: i32 = word!(game.cur_enemy_index) as i32;
            if slot!(game, Enemy_Torizo[E].var_04) | slot!(game, Enemy_Torizo[E].base.flash_timer) == 0 {
                actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::ReactionOnly);
                if slot!(game, Enemy_Torizo[E].base.health) == 0 {
                    slot!(game, Enemy_Torizo[E].var_E = (0xaac6abi32) as u16);
                    slot!(game, Enemy_Torizo[E].base.current_instruction = addr_kTorizo_Ilist_B1C8 as u16);
                    slot!(game, Enemy_Torizo[E].base.instruction_timer = 1);
                    slot!(game, Enemy_Torizo[E].parameter_2 = (slot!(game, Enemy_Torizo[E].parameter_2)) | (0xc000u16));
                    slot!(game, Enemy_Torizo[E].base.properties = (slot!(game, Enemy_Torizo[E].base.properties)) | ((kEnemyProps_Tangible) as u16));
                }
            }
        };
    };
    [11195322] GoldTorizo_Hurt [] {
        actors::torizo::drool_at_low_health(word!(game.cur_enemy_index), game);
        if slot!(game, EnemyData[word!(game.cur_enemy_index) as i32].flash_timer) & 1 != 0 {
            torizo::flash_current(game);
        } else {
            Torizo_Func_11(word!(game.cur_enemy_index), game);
        };
    };
    [11196360] TourianEntranceStatue_Init [] {
        let v0: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, EnemyData[v0].palette_index = 0);
        slot!(game, EnemyData[v0].instruction_timer = 1);
        slot!(game, EnemyData[v0].timer = 0);
        let v1: u16 = statue_tables::tourian_statue(
            game,
            (slot!(game, EnemyData[v0].parameter_1) >> 1) as isize,
        )
        .instruction_list;
        slot!(game, EnemyData[v0].current_instruction = v1);
        if slot!(game, EnemyData[v0].parameter_1) == 0 {
            SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueBaseDecoration as u16, 0, game);
            SpawnEprojWithRoomGfx(addr_kEproj_TourianStatueRidley as u16, 0, game);
            SpawnEprojWithRoomGfx(addr_kEproj_TourianStatuePhantoon as u16, 0, game);
        }
        word_transfers!(game, descending, 16;
            le(target_palettes, 255) <- rom(0xaad785, 15),
            le(target_palettes, 175) <- rom(0xaad765, 15),
        );
    };
    [11197603] Shaktool_Hurt [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        unsafe { actor_species_pre_instruction(((slot!(game, Enemy_Shaktool[E].var_F)) as u32) | (0xaa0000u32), game) };
    };
    [11198019] Shaktool_Init [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_Shaktool[E].base.instruction_timer = 1);
        slot!(game, Enemy_Shaktool[E].base.timer = 0);
        slot!(game, Enemy_Shaktool[E].var_A = 0);
        slot!(game, Enemy_Shaktool[E].var_D = 0);
        let shakto_parameter_2: u16 = slot!(game, Enemy_Shaktool[E].parameter_2);
        let v3: i32 = shakto_parameter_2 as i32 >> 1;
        let piece = shaktool_tables::piece(game, v3 as isize);
        slot!(game, Enemy_Shaktool[E].base.properties = (slot!(game, Enemy_Shaktool[E].base.properties)) | (piece.properties));
        slot!(game, Enemy_Shaktool[E].var_E = (word!(game.cur_enemy_index)).wrapping_sub(piece.ram_offset));
        slot!(game, Enemy_Shaktool[E].var_F = crate::rom::RomRecord {
            image: game.capture_native_rom(),
            base: crate::rtl::lorom_offset(0xaadedb) as usize,
        }.word(shakto_parameter_2 as usize));
        slot!(game, Enemy_Shaktool[E].var_C = {
            let at = shakto_parameter_2 as usize;
            u16::from_ne_bytes([
                g_word_AADEE9[at / 2].to_ne_bytes()[at % 2],
                g_word_AADEE9[(at + 1) / 2].to_ne_bytes()[(at + 1) % 2],
            ])
        }.wrapping_sub(piece.zero));
        slot!(game, Enemy_Shaktool[E].var_B = piece.neighbour_angle);
        slot!(game, Enemy_Shaktool[E].base.current_instruction = piece.instruction_list);
        slot!(game, Enemy_Shaktool[E].base.layer = piece.layer);
        if shakto_parameter_2 != 0 {
            actors::shaktool::position_relative_to_previous(word!(game.cur_enemy_index), game);
        }
    };
    [11198255] Shaktool_Touch [] {
        actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    };
    [11198260] Shaktool_Shot [] {
        actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
        let E: i32 = word!(game.cur_enemy_index) as i32;
        if slot!(game, Enemy_Shaktool[E].base.health) == 0 {
            let shakto_var_E: u16 = slot!(game, Enemy_Shaktool[E].var_E);
            for segment in actors::shaktool::segments(shakto_var_E) {
                slot!(game, Enemy_Shaktool[segment].base.properties = 512);
            }
        }
    };
    [11200278] N00bTubeCracks_Init [] {
        word_transfers!(game, descending, 32; le(target_palettes, 175) <- rom(0xaae2dd, 31));
    };
    [11200293] ChozoStatue_Init [] {
        let v0: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, EnemyData[v0].properties = (slot!(game, EnemyData[v0].properties) as i32
            | (kEnemyProps_DisableSamusColl as i32
                | kEnemyProps_ProcessedOffscreen as i32
                | 0x8000)) as u16);
        slot!(game, EnemyData[v0].spritemap_pointer = addr_kSpritemap_Nothing_AA as u16);
        slot!(game, EnemyData[v0].instruction_timer = 1);
        slot!(game, EnemyData[v0].timer = 0);
        slot!(game, EnemyData[v0].ai_preinstr = (0xaae7a6i32) as u16);
        slot!(game, EnemyData[v0].parameter_1 = 0);
        slot!(game, EnemyData[v0].palette_index = 0);
        slot!(game, EnemyData[0i32].layer = 0);
        let parameter_2: u16 = slot!(game, EnemyData[v0].parameter_2);
        slot!(game, EnemyData[v0].current_instruction = statue_tables::chozo_statue(game, (parameter_2 >> 1) as isize).instruction_list);
        if parameter_2 != 0 {
            sub_AAE784(game);
        } else {
            word_transfers!(game, descending, 16;
                le(target_palettes, 175) <- table(kChozoStatue_Palettes, 15),
                le(target_palettes, 159) <- table(kChozoStatue_Palette, 15),
            );
            spawn_hardcoded(
                SpawnHardcodedPlmArgs::new(0x4a, 0x17, 0xd6ee as VoidP),
                game,
            );
            spawn_hardcoded(
                SpawnHardcodedPlmArgs::new(0x17, 0x1d, 0xd6fc as VoidP),
                game,
            );
        };
    };
    [11200423] ChozoStatue_Main [] {
        let v0: i32 = word!(game.cur_enemy_index) as i32;
        unsafe { actor_species_pre_instruction(((slot!(game, EnemyData[v0].ai_preinstr)) as u32) | (0xaa0000u32), game) };
    };
    @functions [pub fn (game: &mut Game)];
    @prefix { let E: i32 = word!(game.cur_enemy_index) as i32; };
    [11191972] Torizo_Main [] {
        CallTorizoFunc(
            ((slot!(game, Enemy_Torizo[E].var_E)) as u32) | (0xaa0000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    [11192447] Torizo_Init [] {
        if CheckBossBitForCurArea(4, game) & 1 != 0 {
            slot!(game, Enemy_Torizo[E].base.properties = (slot!(game, Enemy_Torizo[E].base.properties)) | ((kEnemyProps_Deleted) as u16));
        } else {
            let v2: i32 = word!(game.area_index) as i32 >> 1;
            slot!(game, Enemy_Torizo[E].base.properties = (slot!(game, Enemy_Torizo[E].base.properties)) | ((g_word_AAC96B[v2 as usize]) as u16));
            slot!(game, Enemy_Torizo[E].base.extra_properties = (slot!(game, Enemy_Torizo[E].base.extra_properties)) | (4u16));
            slot!(game, Enemy_Torizo[E].base.x_width = g_word_AAC96F[v2 as usize] as u16);
            slot!(game, Enemy_Torizo[E].base.y_height = g_word_AAC973[v2 as usize] as u16);
            slot!(game, Enemy_Torizo[E].var_E = (0xaac6bfi32) as u16);
            slot!(game, Enemy_Torizo[E].base.instruction_timer = 1);
            slot!(game, Enemy_Torizo[E].base.timer = 0);
            slot!(game, Enemy_Torizo[E].base.palette_index = 0);
            slot!(game, Enemy_Torizo[E].var_F = addr_locret_AAC95E as u16);
            slot!(game, Enemy_Torizo[E].base.current_instruction = torizo_tables::torizo(game, v2 as isize).instruction_list);
            slot!(game, Enemy_Torizo[E].base.spritemap_pointer = addr_kTorizo_ExtSprmap_87D0 as u16);
            slot!(game, Enemy_Torizo[E].base.x_pos = g_word_AAC95F[v2 as usize] as u16);
            slot!(game, Enemy_Torizo[E].base.y_pos = g_word_AAC963[v2 as usize] as u16);
            slot!(game, Enemy_Torizo[E].var_A = 0);
            slot!(game, Enemy_Torizo[E].var_B = 256);
            torizo::initialize_targets(game);
            if word!(game.area_index) != 0 {
                torizo::set_targets(torizo::GOLDEN_START, game);
                if word!(game.joypad1_lastkeys) == 0xc0c0 {
                    word! { game, 700; samus_health, samus_max_health }
                    word! { game, 300; samus_max_reserve_health, samus_reserve_health }
                    word! { game, 100; samus_missiles, samus_max_missiles }
                    word! { game, 20;
                        samus_super_missiles, samus_max_super_missiles, samus_power_bombs,
                        samus_max_power_bombs,
                    }
                    word!(game.equipped_items = -3273i32 as u16);
                    word!(game.collected_items = -3273i32 as u16);
                    word! { game, 4111; equipped_beams, collected_beams }
                }
            } else {
                torizo::set_targets(torizo::BOMB_START, game);
                initialize_bomb_torizo_haze(game);
            }
        };
    };
    [11195241] GoldTorizo_Main [] {
        if word!(game.samus_pose) as i32 == kPose_1B_FaceR_SpaceJump as i32
            || word!(game.samus_pose) as i32 == kPose_1C_FaceL_SpaceJump as i32
        {
            slot!(game, Enemy_Torizo[E].var_07 = slot!(game, Enemy_Torizo[E].var_07).wrapping_add(1));
        } else {
            slot!(game, Enemy_Torizo[E].var_07 = 0);
        }
        CallTorizoFunc(
            ((slot!(game, Enemy_Torizo[E].var_E)) as u32) | (0xaa0000u32),
            word!(game.cur_enemy_index),
            game,
        );
    };
    Torizo_D658 [] {
        if slot!(game, Enemy_Torizo[E].base.flash_timer) == 0 && slot!(game, Enemy_Torizo[E].var_04) == 0 {
            actors::torizo::golden_damaged_shot_reaction(game);
        }
    };
    [11196007] GoldTorizo_Shot [] {
        if slot!(game, Enemy_Torizo[E].base.flash_timer) == 0 {
            if slot!(game, Enemy_Torizo[E].var_04) != 0 {
                actors::torizo::golden_damaged_shot_reaction(game);
                return;
            }
            let react_to_shot: bool = ((slot!(game, Enemy_Torizo[E].parameter_2) & 0x1000) != 0)
                || ({
                    let mut v2: u16 = (2u16).wrapping_mul(word!(game.collision_detection_index));
                    let mut v3: u16 =
                        (word!(game.projectile_type[word!(game.collision_detection_index)]))
                            & (0xf00u16);
                    slot!(game, Enemy_Torizo[E].var_05 = v3);
                    if v3 == 256 {
                        actors::torizo::golden_missile_reaction(word!(game.cur_enemy_index), v2, game);
                        return;
                    }
                    (v3 != 512)
                        || ({
                            actors::torizo::golden_super_missile_reaction(word!(game.cur_enemy_index), v2, game);
                            false
                        })
                });
            if react_to_shot {
                slot!(game, Enemy_Torizo[E].parameter_2 = (slot!(game, Enemy_Torizo[E].parameter_2)) | (0x2000u16));
                actors::torizo::golden_damaged_shot_reaction(game);
            }
        }
    };
    @forward [pub fn (game: &mut Game)];
    [11173903] Enemy_GrappleReact_CancelBeam_AA => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [11173953] Enemy_NormalFrozenAI_AA => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11174004] Enemy_ClearAiPreInstr_AA [jp] {
        slot!(game, EnemyData[k as i32].ai_preinstr = (0xaa807bi32) as u16);
    };
    [11173995] Enemy_SetAiPreInstr_AA [jp.offset(1)] {
        slot!(game, EnemyData[k as i32].ai_preinstr = jp.word(0));
    };
    [11186332] Torizo_Instr_3 [jp.offset(1)] { slot!(game, Enemy_Torizo[k as i32].var_E = jp.word(0)); };
    [11186461] Torizo_Instr_31 [jp] {
        let E: i32 = k as i32;
        slot!(game, Enemy_Torizo[E].parameter_2 = (slot!(game, Enemy_Torizo[E].parameter_2)) | (0x8000u16));
        let mut parameter = SpawnEprojWithGfx(slot!(game, Enemy_Torizo[E].parameter_2), k, addr_stru_86A95B as u16, game);
        parameter = SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as u16, game);
        parameter = SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as u16, game);
        parameter = SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as u16, game);
        parameter = SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as u16, game);
        SpawnEprojWithGfx(parameter, k, addr_stru_86A95B as u16, game);
    };
    [11186622] Torizo_Instr_33 [jp] {
        let E: i32 = k as i32;
        slot!(game, Enemy_Torizo[E].parameter_2 = (slot!(game, Enemy_Torizo[E].parameter_2)) | (0x4000u16));
    };
    [11186724] Torizo_Instr_36 [jp] {
        let E: i32 = k as i32;
        slot!(game, Enemy_Torizo[E].base.properties = (slot!(game, Enemy_Torizo[E].base.properties) as i32 & !(kEnemyProps_Invisible as i32)) as u16);
    };
    [11186734] Torizo_Instr_37 [jp] {
        let E: i32 = k as i32;
        slot!(game, Enemy_Torizo[E].base.properties = (slot!(game, Enemy_Torizo[E].base.properties)) | ((kEnemyProps_Invisible) as u16));
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11186744] Torizo_Instr_35 [jp] {
        word_transfers!(game, descending, 16;
            le(target_palettes, 175) <- value(0),
            le(target_palettes, 159) <- value(0),
        );
    };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11186765] Torizo_Instr_38 [jp] {
        SetBossBitForCurArea(4, game);
        QueueMusic_Delayed8(g_word_AAB09A, game);
        if word!(game.area_index) != 0 {
            scatter(DropSource::GoldenTorizo, k, game);
        } else {
            scatter(DropSource::BombTorizo, k, game);
        }
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11186801] Torizo_Instr_6 [jp] { selected(0x600, game); };
    [11188557] Torizo_Instr_5 [jp] { torizo::set_targets(torizo::BOMB_ACTIVE, game); };
    [11188561] Torizo_Instr_9 [jp] {
        QueueMusic_Delayed8(g_word_AAB098, game);
        palette_effects::spawn(addr_stru_8DF759 as u16, game);
    };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11190985] Torizo_Instr_2 [jp] { slot!(game, Enemy_Torizo[k as i32].var_04 = 30583); };
    [11190993] Torizo_Instr_8 [jp] { slot!(game, Enemy_Torizo[k as i32].var_04 = 0); };
    [11191001] Torizo_Instr_25 [] {
        if (slot!(game, Enemy_Torizo[k as i32].parameter_2) & 0x4000) != 0 {
            return EnemyScript::jump(jp.word(0));
        }
        if word!(game.area_index) != 0 {
            EnemyScript::jump(jp.word(2))
        } else {
            jp.offset(2)
        }
    };
    [11191021] Torizo_Instr_22 [jp.offset(1)] { slot!(game, Enemy_Torizo[k as i32].var_00 = jp.word(0)); };
    @functions [pub fn (k: u16, _jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11191031] Torizo_Instr_19 [EnemyScript::jump(slot!(game, Enemy_Torizo[k as i32].var_00))] { };
    [11191037] Torizo_Instr_32 [EnemyScript::jump(slot!(game, Enemy_Torizo[k as i32].var_01))] { };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11191043] Torizo_Instr_30 [EnemyScript::Stop] {
        let a: u16 = jp.word(0);
        for _ in 0..=5 {
            SpawnEprojWithRoomGfx(addr_kEproj_BombTorizoLowHealthExplode as u16, a, game);
        }
        let E: i32 = k as i32;
        slot!(game, Enemy_Torizo[E].base.current_instruction = jp.offset(1)
            .offset_from(bank_base(slot!(byte game, EnemyData[k as i32].bank[0])))
            as c_long as u16);
        slot!(game, Enemy_Torizo[E].base.flash_timer = 40);
        slot!(game, Enemy_Torizo[E].base.instruction_timer = 40);
    };
    [11191087] Torizo_Instr_34 [EnemyScript::Stop] {
        // The instruction dispatcher leaves this handler address in A.
        SpawnEprojWithRoomGfx(addr_kEproj_BombTorizoDeathExplosion as u16, 0xc32f, game);
        let E: i32 = k as i32;
        slot!(game, Enemy_Torizo[E].base.current_instruction = jp
            .offset_from(bank_base(slot!(byte game, EnemyData[k as i32].bank[0])))
            as c_long as u16);
        slot!(game, Enemy_Torizo[E].base.flash_timer = 1);
        slot!(game, Enemy_Torizo[E].base.instruction_timer = 1);
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11191114] Torizo_Instr_24 [jp] {
        let left_foot_parameter = SpawnEprojWithRoomGfx(
            addr_kEproj_TourianLandingDustCloudsRightFoot as u16,
            0xc34a,
            game,
        );
        SpawnEprojWithRoomGfx(addr_stru_86AFF3 as u16, left_foot_parameter, game);
    };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11191131] Torizo_Instr_12 [jp] {
        let health: u16 = slot!(game, Enemy_Torizo[k as i32].base.health);
        if health < 0x15e {
            SpawnEprojWithGfx(
                health,
                k,
                addr_kEproj_BombTorizoLowHealthInitialDrool as u16,
                game,
            );
        }
    };
    [11191149] Torizo_Instr_10 [jp] {
        let E: i32 = k as i32;
        slot!(game, Enemy_Torizo[E].parameter_1 = (slot!(game, Enemy_Torizo[E].parameter_1)) | (0x4000u16));
    };
    [11191159] Torizo_Instr_11 [jp] {
        let E: i32 = k as i32;
        slot!(game, Enemy_Torizo[E].parameter_1 = (slot!(game, Enemy_Torizo[E].parameter_1)) & (0x1fffu16));
        slot!(game, Enemy_Torizo[E].var_09 = slot!(game, Enemy_Torizo[E].var_09).wrapping_add(1));
    };
    [11191178] Torizo_Instr_29 [jp] {
        let E: i32 = k as i32;
        slot!(game, Enemy_Torizo[E].parameter_1 = slot!(game, Enemy_Torizo[E].parameter_1) & 0x1fff | 0x8000);
        slot!(game, Enemy_Torizo[E].var_09 = slot!(game, Enemy_Torizo[E].var_09).wrapping_add(1));
    };
    [11191200] Torizo_Instr_1 [jp] {
        let E: i32 = k as i32;
        slot!(game, Enemy_Torizo[E].parameter_1 = slot!(game, Enemy_Torizo[E].parameter_1) & 0x1fff | 0x2000);
        slot!(game, Enemy_Torizo[E].var_09 = slot!(game, Enemy_Torizo[E].var_09).wrapping_add(1));
    };
    [11191222] Torizo_Instr_28 [jp] {
        let E: i32 = k as i32;
        slot!(game, Enemy_Torizo[E].parameter_1 = slot!(game, Enemy_Torizo[E].parameter_1) & 0x1fff | 0xa000);
        slot!(game, Enemy_Torizo[E].var_09 = slot!(game, Enemy_Torizo[E].var_09).wrapping_add(1));
    };
    [11191244] Torizo_Instr_4 [jp.offset(1)] {
        let v2: u16 = jp.word(0);
        let E: i32 = k as i32;
        slot!(game, Enemy_Torizo[E].base.x_pos = (slot!(game, Enemy_Torizo[E].base.x_pos)).wrapping_add((g_word_AAC3EE[(v2 >> 1) as usize]) as u16));
        slot!(game, Enemy_Torizo[E].base.y_pos = (slot!(game, Enemy_Torizo[E].base.y_pos))
            .wrapping_add((g_word_AAC40E[((v2 as i32 & 0xf) >> 1) as usize]) as u16));
    };
    [11191326] Torizo_Instr_40 [jp.offset(1)] {
        let v2: u16 = jp.word(0);
        let E: i32 = k as i32;
        slot!(game, Enemy_Torizo[E].base.x_pos = (slot!(game, Enemy_Torizo[E].base.x_pos)).wrapping_sub((g_word_AAC440[(v2 >> 1) as usize]) as u16));
        slot!(game, Enemy_Torizo[E].base.y_pos = (slot!(game, Enemy_Torizo[E].base.y_pos))
            .wrapping_sub((g_word_AAC460[((v2 as i32 & 0xf) >> 1) as usize]) as u16));
    };
    [11191408] Torizo_Instr_16 [] {
        actors::torizo::walk(
            k,
            jp,
            addr_kTorizo_Ilist_B962 as u16,
            addr_kTorizo_Ilist_BDD8 as u16,
            72,
            game,
        )
    };
    [11191525] Torizo_Instr_27 [] {
        actors::torizo::walk(
            k,
            jp,
            addr_kTorizo_Ilist_BD0E as u16,
            addr_kTorizo_Ilist_C188 as u16,
            72,
            game,
        )
    };
    [11191642] Torizo_Instr_23 [] {
        if (slot!(game, Enemy_Torizo[k as i32].var_B) & 0x8000) != 0 {
            EnemyScript::jump(jp.word(0))
        } else {
            jp.offset(1)
        }
    };
    [11191655] Torizo_Instr_14 [EnemyScript::jump(jp.word(0))] {
        if ({ let (actor, threshold) = (k, 0x38); let delta = actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X); u8::from((delta as i16).wrapping_abs() as u16 >= threshold) }) & 1 != 0 {
            return jp.offset(1);
        }
        let E: i32 = k as i32;
        if ((slot!(game, Enemy_Torizo[E].parameter_1) as i32 ^ word!(game.samus_x_pos) as i32 - slot!(game, Enemy_Torizo[E].base.x_pos) as i32)
            & 0x8000)
            == 0
        {
            return jp.offset(1);
        }
        slot!(game, Enemy_Torizo[E].var_00 = jp.offset(1)
            .offset_from(bank_base(slot!(byte game, EnemyData[k as i32].bank[0])))
            as c_long as u16);
    };
    [11191691] Torizo_Instr_15 [EnemyScript::jump(jp.word(0))] {
        if ({ let (actor, threshold) = (k, 0x20); let delta = actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X); u8::from((delta as i16).wrapping_abs() as u16 >= threshold) }) & 1 != 0 || (actors::torizo::facing_against_samus(k, game) & 0x8000) != 0 {
            return jp.offset(1);
        }
        Torizo_C22D(k, game);
    };
    [11191716] Torizo_Instr_26 [] {
        let E: i32 = k as i32;
        slot!(game, Enemy_Torizo[E].var_00 = jp.offset(2)
            .offset_from(bank_base(slot!(byte game, EnemyData[k as i32].bank[0])))
            as c_long as u16);
        if word!(game.samus_missiles) < 5
            || (word!(game.nmi_frame_counter_word) as i32
                + (word!(game.samus_x_pos) as i32 & 1)
                + (word!(game.samus_x_pos) as i32 >> 1)
                & 8)
                != 0
        {
            EnemyScript::jump(jp.word(0))
        } else {
            EnemyScript::jump(jp.word(2))
        }
    };
    [11191755] Torizo_Instr_18 [jp] {
        // The instruction dispatcher leaves this handler address in A.
        let mut parameter = SpawnEprojWithGfx(0xc5cb, k, addr_kEproj_BombTorizosChozoOrbs as u16, game);
        parameter = SpawnEprojWithGfx(parameter, k, addr_kEproj_BombTorizosChozoOrbs as u16, game);
        SpawnEprojWithGfx(parameter, k, addr_kEproj_BombTorizosChozoOrbs as u16, game);
    };
    [11191779] Torizo_Instr_20 [jp.offset(1)] {
        SpawnEprojWithGfx(
            jp.word(0),
            k,
            addr_kEproj_BombTorizoSonicBoom as u16,
            game,
        );
    };
    [11191794] Torizo_Instr_44 [jp.offset(1)] {
        SpawnEprojWithGfx(
            jp.word(0),
            k,
            addr_kEproj_GoldenTorizoSonicBoom as u16,
            game,
        );
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11191809] Torizo_Instr_21 [jp.offset(1)] {
        SpawnEprojWithRoomGfx(
            addr_kEproj_BombTorizoExplosiveSwipe as u16,
            jp.word(0),
            game,
        );
    };
    [11191824] Torizo_Instr_17 [jp] { QueueSfx2_Max6(0x27, game); };
    [11191832] Torizo_Instr_13 [jp] { QueueSfx2_Max6(0x4b, game); };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11193038] Torizo_Instr_39 [] {
        if slot!(game, Enemy_Torizo[k as i32].base.y_pos) == 375 {
            jp.offset(1)
        } else {
            EnemyScript::jump(jp.word(0))
        }
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11193054] Torizo_Instr_41 [jp] { torizo::set_targets(torizo::GOLDEN_ACTIVE, game); };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11193058] Torizo_Instr_42 [jp] {
        QueueMusic_Delayed8(5, game);
        let E: i32 = k as i32;
        slot!(game, Enemy_Torizo[E].base.x_width = 18);
        slot!(game, Enemy_Torizo[E].base.y_height = 48);
        palette_effects::spawn(addr_stru_8DF75D as u16, game);
    };
    [11193815] Torizo_Instr_48 [jp] {
        let E: i32 = k as i32;
        slot!(game, Enemy_Torizo[E].parameter_2 = slot!(game, Enemy_Torizo[E].parameter_2) & !(0x1000));
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11194601] Torizo_Instr_57 [jp] { SpawnEprojWithRoomGfx(addr_stru_86B1C0 as u16, 0, game); };
    [11194611] Torizo_Instr_58 [EnemyScript::jump(jp.word(0))] {
        let mut v2: i16 = 34;
        while {
            let at = usize::try_from(0x1997isize + v2 as u16 as i32 as isize)
                .expect("enemy projectile word outside RAM");
            u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
        } != 0xb1c0
        {
            v2 = (v2).wrapping_sub(2);
            if v2 < 0 {
                return jp.offset(1);
            }
        }
    };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    @prefix { let E: i32 = k as i32; };
    [11194747] Torizo_Instr_59 [jp] {  slot!(game, Enemy_Torizo[E].var_06 = slot!(game, Enemy_Torizo[E].var_06) & !(0x8000)); };
    [11194759] Torizo_Instr_62 [jp] {  slot!(game, Enemy_Torizo[E].var_06 = (slot!(game, Enemy_Torizo[E].var_06)) | (0x8000u16)); };
    [11194855] Torizo_Instr_63 [jp] {  slot!(game, Enemy_Torizo[E].parameter_2 = slot!(game, Enemy_Torizo[E].parameter_2) & !(0x2000)); };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11195279] Torizo_Instr_56 [jp] { QueueSfx2_Max6(0x34, game); };
    [11195287] Torizo_Instr_60 [jp] { QueueSfx2_Max6(0x67, game); };
    [11195295] Torizo_Instr_46 [jp] { QueueSfx2_Max6(0x48, game); };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11195360] Torizo_Instr_47 [jp] {
        SpawnEprojWithRoomGfx(addr_kEproj_GoldenTorizoSuperMissile as u16, 0, game);
    };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11195370] Torizo_Instr_49 [EnemyScript::jump(jp.word(0))] {
        if (actors::torizo::facing_against_samus(k, game) & 0x8000) == 0
            || ({ let (actor, threshold) = (k, 4); let delta = actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X); u8::from((delta as i16).wrapping_abs() as u16 >= threshold) }) & 1 == 0
            || ({ let (actor, threshold) = (k, 0x28); let delta = actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X); u8::from((delta as i16).wrapping_abs() as u16 >= threshold) }) & 1 != 0
            || word!(game.samus_pose) as i32 != kPose_1D_FaceR_Morphball_Ground as i32
                && word!(game.samus_pose) as i32 != kPose_1E_MoveR_Morphball_Ground as i32
                && word!(game.samus_pose) as i32 != kPose_1F_MoveL_Morphball_Ground as i32
                && word!(game.samus_pose) as i32 != kPose_79_FaceR_Springball_Ground as i32
                && word!(game.samus_pose) as i32 != kPose_7A_FaceL_Springball_Ground as i32
                && word!(game.samus_pose) as i32 != kPose_7B_MoveR_Springball_Ground as i32
                && word!(game.samus_pose) as i32 != kPose_7C_MoveL_Springball_Ground as i32
        {
            return jp.offset(1);
        }
        slot!(game, Enemy_Torizo[k as i32].var_09 = 0);
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11195446] Torizo_Instr_61 [jp.offset(1)] {
        SpawnEprojWithRoomGfx(addr_kEproj_GoldenTorizoEyeBeam as u16, jp.word(0), game);
    };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11195461] Torizo_Instr_53 [EnemyScript::jump(jp.word(0))] {
        if (actors::torizo::facing_against_samus(k, game) & 0x8000) != 0
            || ({ let (actor, threshold) = (k, 0x20); let delta = actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X); u8::from((delta as i16).wrapping_abs() as u16 >= threshold) }) & 1 == 0
            || ({ let (actor, threshold) = (k, 0x60); let delta = actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X); u8::from((delta as i16).wrapping_abs() as u16 >= threshold) }) & 1 != 0
            || (game.next_random() & 0x110) != 0
        {
            return jp.offset(1);
        }
        slot!(game, Enemy_Torizo[k as i32].var_00 = jp.offset(1)
            .offset_from(bank_base(slot!(byte game, EnemyData[k as i32].bank[0])))
            as c_long as u16);
    };
    [11195508] Torizo_Instr_55 [EnemyScript::jump(jp.word(0))] {
        let E: i32 = k as i32;
        if slot!(game, Enemy_Torizo[E].base.health) > 0x788 || (game.next_random() & 0x102) != 0 {
            return jp.offset(1);
        }
        slot!(game, Enemy_Torizo[E].var_09 = 0);
        slot!(game, Enemy_Torizo[E].var_00 = jp.offset(1)
            .offset_from(bank_base(slot!(byte game, EnemyData[k as i32].bank[0])))
            as c_long as u16);
    };
    [11195547] Torizo_Instr_52 [EnemyScript::jump(jp.word(0))] {
        let E: i32 = k as i32;
        if slot!(game, Enemy_Torizo[E].base.health) <= 0x2a30 || (slot!(game, Enemy_Torizo[E].parameter_2) & 0x2000) == 0 {
            return jp.offset(1);
        }
        slot!(game, Enemy_Torizo[E].var_00 = jp.offset(1)
            .offset_from(bank_base(slot!(byte game, EnemyData[k as i32].bank[0])))
            as c_long as u16);
    };
    [11195578] Torizo_Instr_50 [EnemyScript::jump(jp.word(0))] {
        if ({ let (actor, threshold) = (k, 0x70); let delta = actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X); u8::from((delta as i16).wrapping_abs() as u16 >= threshold) }) & 1 == 0 {
            return jp.offset(1);
        }
        if (actors::torizo::facing_against_samus(k, game) & 0x8000) != 0 {
            return jp.offset(1);
        }
        let E: i32 = k as i32;
        if slot!(game, Enemy_Torizo[E].var_07) <= 0x168
            && ((word!(game.joypad1_lastkeys) & 0x300) == 0 || (game.next_random() & 0x101) == 0)
        {
            return jp.offset(1);
        }
        slot!(game, Enemy_Torizo[E].var_09 = 0);
        Torizo_C20A(k, game);
    };
    [11195635] Torizo_Instr_43 [jp] {
        // The instruction dispatcher leaves this handler address in A.
        SpawnEprojWithGfx(0xd4f3, k, addr_kEproj_GoldenTorizosChozoOrbs as u16, game);
    };
    [11195645] Torizo_Instr_51 [EnemyScript::jump(jp.word(0))] {
        let E: i32 = k as i32;
        if slot!(game, Enemy_Torizo[E].var_09) < 8
            && (({ let (actor, threshold) = (k, 0x20); let delta = actor_api::player_delta(&Runtime::new(game), Key::from_legacy(actor), actor_api::Axis::X); u8::from((delta as i16).wrapping_abs() as u16 >= threshold) }) & 1 != 0
                || (actors::torizo::facing_against_samus(k, game) & 0x8000) != 0)
        {
            return jp.offset(1);
        }
        slot!(game, Enemy_Torizo[E].var_09 = 0);
        Torizo_C22D(k, game);
    };
    [11195686] Torizo_Instr_45 [] {
        slot!(game, Enemy_Torizo[k as i32].var_00 = jp.offset(2)
            .offset_from(bank_base(slot!(byte game, EnemyData[k as i32].bank[0])))
            as c_long as u16);
        if word!(game.samus_missiles) < 0x20
            || (word!(game.nmi_frame_counter_word) as i32
                + (word!(game.samus_x_pos) as i32 & 1)
                + (word!(game.samus_x_pos) as i32 >> 1)
                & 8)
                != 0
        {
            EnemyScript::jump(jp.word(0))
        } else {
            EnemyScript::jump(jp.word(2))
        }
    };
    [11195725] Torizo_Instr_54 [] {
        actors::torizo::walk(
            k,
            jp,
            addr_kTorizo_Ilist_D203 as u16,
            addr_kTorizo_Ilist_D2BF as u16,
            16,
            game,
        )
    };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    @prefix { let E: i32 = k as i32; };
    [11196721] Shaktool_Instr_2 [actors::shaktool::move_piece_one_pixel(k, jp, (slot!(game, Enemy_Shaktool[EX].var_D)) ^ (0x80u16), game)] {

        let EX: i32 = slot!(game, Enemy_Shaktool[E].var_E) as i32 + 192;
    };
    [11196735] Shaktool_Instr_3 [actors::shaktool::move_piece_one_pixel(k, jp, slot!(game, Enemy_Shaktool[EX].var_D), game)] {

        let EX: i32 = slot!(game, Enemy_Shaktool[E].var_E) as i32 + 192;
    };
    [11196746] Shaktool_Instr_4 [actors::shaktool::move_piece_one_pixel(
            k,
            jp,
            u16::from_ne_bytes([
                slot!(byte game, Enemy_Shaktool[E].var_A[1]),
                slot!(byte game, Enemy_Shaktool[E].var_A[2]),
            ]) ^ (0x80u16),
            game,
        )] {
    };
    [11196755] Shaktool_Instr_5 [actors::shaktool::move_piece_one_pixel(
            k,
            jp,
            u16::from_ne_bytes([
                slot!(byte game, Enemy_Shaktool[E].var_A[1]),
                slot!(byte game, Enemy_Shaktool[E].var_A[2]),
            ]),
            game,
        )] {
    };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11196858] Shaktool_Instr_1 [] {
        let shakt_var_E: u16 = slot!(game, Enemy_Shaktool[k as i32].var_E);
        for (index, segment) in actors::shaktool::segments(shakt_var_E).enumerate() {
            let pre_instruction = shaktool_tables::piece(game, index as isize).function;
            slot!(game, Enemy_Shaktool[segment].var_F = pre_instruction);
        }
        jp
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11199529] Shaktool_Instr_9 [jp] {
        word! {game; fx_timer = 32; fx_y_vel = 64; }
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11199542] Shaktool_Instr_11 [jp] { crate::game::room_mechanisms::lower_acid(game); };
    [11199549] Shaktool_Instr_10 [jp] { CallSomeSamusCode(1, game); };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11199871] Shaktool_Instr_8 [jp] { QueueSfx2_Max6(0x1c, game); };
    [11199879] Shaktool_Instr_13 [jp] { QueueSfx2_Max6(0x4b, game); };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11199887] Shaktool_Instr_12 [jp.offset(1)] {
        let e: i32 = k as i32;
        let arg: u16 = jp.word(0);
        ({ let (x, y) = ((slot!(game, Enemy_Shaktool[e].base.x_pos)).wrapping_add(arg), (slot!(game, Enemy_Shaktool[e].base.y_pos)).wrapping_add(28)); actor_api::select_block_at(&mut Runtime::new(game), sm_game::geometry::Point { x, y }) });
        if ({
            let index = word!(game.room_width_in_blocks) as i32 + word!(game.cur_block_index) as i32;
            let at = 0x10002usize + 2 * index as usize;
            u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
        } & 0xf000)
            == 0xa000
        {
            unsafe { spawn_collision(addr_kPlmHeader_D113 as u16, game) };
            SpawnEprojWithRoomGfx(
                addr_kEproj_WreckedShipChozoSpikeFootsteps_1 as u16,
                arg,
                game,
            );
        }
    };
    [11199960] Shaktool_Instr_7 [jp.offset(1)] {
        let E: i32 = k as i32;
        slot!(game, Enemy_Shaktool[E].var_C = jp.word(0));
        ({ let (actor, amount) = (k, (statue_tables::movement(game, (slot!(game, Enemy_Shaktool[E].var_C) >> 1) as isize).x_velocity as i16 as i32)
                << 8); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 });
        ({ let (actor, amount) = (k, (abs16(statue_tables::movement(
                game,
                (slot!(game, Enemy_Shaktool[E].var_C) >> 1) as isize
            ).x_velocity) as i16 as i32)
                << 8); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 });
        ({ let actor = k; actor_api::snap_to_slopes(&mut Runtime::new(game), Key::from_legacy(actor)) as u8 });
        let v6: i32 = slot!(game, Enemy_Shaktool[E].var_C) as i32 >> 1;
        let movement = statue_tables::movement(game, v6 as isize);
        word! { game;
            samus_x_pos = (movement.samus_x_offset)
                .wrapping_add(slot!(game, Enemy_Shaktool[E].base.x_pos));
            samus_y_pos = (movement.samus_y_offset)
                .wrapping_add(slot!(game, Enemy_Shaktool[E].base.y_pos));
        }
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11200240] Shaktool_Instr_14 [jp] {
        CallSomeSamusCode(1, game);
        game.ram().copy_in(0xcd26, &0u16.to_ne_bytes());
        game.ram().copy_in(0xcd28, &0u16.to_ne_bytes());
        game.ram().copy_in(0xcd29, &0u16.to_ne_bytes());
        game.ram().copy_in(0xcd2d, &1u16.to_ne_bytes());
        spawn_hardcoded(
            SpawnHardcodedPlmArgs::new(0x17, 0x1d, 0xd6fc as VoidP),
            game,
        );
    };
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyPreInstr(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { crate::rtl::Unreachable(game); };
    @functions [pub fn (k: u16, game: &mut Game)];
    [11192360 | 11195885] Torizo_Func_6 [] {
        Torizo_C643(k, game);
    };
    [11199557] sub_AAE445 [] {
        let v1: i32 = k as i32;
        if slot!(game, EnemyData[v1].parameter_1) != 0 {
            slot!(game, EnemyData[v1].current_instruction = addr_kShaktool_Ilist_E3A7 as u16);
            slot!(game, EnemyData[v1].instruction_timer = 1);
        }
    };
    [11200430] Shaktool_PreInstr_0 [] {
        if (gameplay_flag_runtime::boss_word_at(word!(game.area_index), BossWordOrder::Native, game) & 1) != 0 {
            let E: i32 = k as i32;
            if slot!(game, Enemy_Shaktool[E].parameter_1) != 0 {
                slot!(game, Enemy_Shaktool[E].base.current_instruction = addr_kShaktool_Ilist_E461 as u16);
                slot!(game, Enemy_Shaktool[E].base.instruction_timer = 1);
                slot!(game, Enemy_Shaktool[E].var_A = -256i32 as u16);
                slot!(game, Enemy_Shaktool[E].var_B = 256);
            }
        }
    };
    [11197612] Shaktool_DCAC [] {
        actors::shaktool::position_relative_to_previous(k, game);
        let E: i32 = k as i32;
        let mut shakto_var_C: u16 = if (slot!(game, Enemy_Shaktool[E].parameter_1) & 0x4000) != 0 {
            slot!(game, Enemy_Shaktool[E].var_A = (slot!(game, Enemy_Shaktool[E].var_A)).wrapping_add(256));
            256
        } else {
            slot!(game, Enemy_Shaktool[E].var_C)
        };
        slot!(game, Enemy_Shaktool[E].var_B = (slot!(game, Enemy_Shaktool[E].var_B) as i32
            + if slot!(game, Enemy_Shaktool[E].parameter_1) & 0x8000 != 0 {
                -(shakto_var_C as i32)
            } else {
                shakto_var_C as i32
            }) as u16);
    };
    [11197655] Shaktool_DCD7 [] {
        Shaktool_DCAC(k, game);
        let E: i32 = k as i32;
        let r18: u16 = (slot!(game, Enemy_Shaktool[E].var_B)) ^ (0x8000u16);
        let mut v2: i16 = (r18 as i32
            + (((slot!(game, Enemy_Shaktool[k as i32 + 64i32].var_B)).wrapping_sub(r18)) as i32 >> 1))
            as i16;
        if (slot!(game, Enemy_Shaktool[E].var_D) & 0x8000) != 0 {
            v2 = {
                let mut bytes = v2.to_ne_bytes();
                bytes[1] ^= 0x80;
                i16::from_ne_bytes(bytes)
            };
        }
        let v3: u16 = (v2.to_ne_bytes()[1] as i32 + 8 & 0xe0) as u16;
        slot!(byte game, Enemy_Shaktool[E].var_D[0] = v3 as u8);
        slot!(game, Enemy_Shaktool[E].base.current_instruction = shaktool_tables::head_direction(game, ((v3 >> 4) >> 1) as isize).instruction_list);
        slot!(game, Enemy_Shaktool[E].base.instruction_timer = 1);
    };
    @functions [pub fn (mut k: u16, game: &mut Game)];
    [11197733] Shaktool_DD25 [] {
        let E: i32 = k as i32;
        let y_pos: u16 = slot!(game, Enemy_Shaktool[E].base.y_pos);
        let x_pos: u16 = slot!(game, Enemy_Shaktool[E].base.x_pos);
        Shaktool_DCAC(k, game);
        let v12: u16 = slot!(game, Enemy_Shaktool[E].base.y_pos);
        let v11: u16 = slot!(game, Enemy_Shaktool[E].base.x_pos);
        slot!(game, Enemy_Shaktool[E].base.x_pos = x_pos);
        slot!(game, Enemy_Shaktool[E].base.y_pos = y_pos);
        if ({ let (actor, amount) = (k, ((((v11) as i16).wrapping_sub((x_pos) as i16)) as i32) << 16); (actor_api::move_horizontal(&mut Runtime::new(game), Key::from_legacy(actor), amount, actor_api::SlopePolicy::Ignore) == actor_api::MovementResult::Blocked) as u8 }) != 0
            || {
                slot!(game, Enemy_Shaktool[E].base.y_pos = y_pos);
                ({ let (actor, amount) = (k, ((((v12) as i16).wrapping_sub((y_pos) as i16)) as i32) << 16); (actor_api::move_vertical(&mut Runtime::new(game), Key::from_legacy(actor), amount) == actor_api::MovementResult::Blocked) as u8 }) != 0
            }
        {
            if (slot!(game, Enemy_Shaktool[E].parameter_1) & 0x2000) != 0 {
                actors::shaktool::set_movement_options(k, (slot!(game, Enemy_Shaktool[E].parameter_1) ^ 0x8000) & 0x8fff, game);
            } else {
                slot!(game, Enemy_Shaktool[E].base.x_pos = x_pos);
                slot!(game, Enemy_Shaktool[E].base.y_pos = y_pos);
                actors::shaktool::flip(k, game);
                actors::shaktool::set_movement_options(
                    word!(game.cur_enemy_index),
                    (slot!(game, Enemy_Shaktool[E].parameter_1)) | (0x2000u16),
                    game,
                );
                actors::shaktool::set_movement_options(
                    word!(game.cur_enemy_index),
                    (slot!(game, Enemy_Shaktool[E].parameter_1)) & (0xbfffu16),
                    game,
                );
            }
            slot!(game, Enemy_Shaktool[E].var_D = 0);
            let v5: u16 = { let (actor, target) = (k, slot!(game, Enemy_Shaktool[E].var_E)); actor_api::angle_between(&Runtime::new(game), Key::from_legacy(actor), Key::from_legacy(target)) };
            let mut v7: i16 = ((v5 as i32) << 8) as i16;
            let mut v8: u16 = if (slot!(game, Enemy_Shaktool[E].parameter_1) & 0x8000) != 0 {
                ((v7) as u16).wrapping_sub(0x4000)
            } else {
                ((v7) as u16).wrapping_add(0x4000)
            };
            actors::shaktool::set_facing_angles(k, v8, game);
            for i in (0i32..=12).rev().step_by(2) {
                actors::shaktool::set_collision_angle_delta(k, game);
                let EK: i32 = k as i32;
                slot!(game, Enemy_Shaktool[EK].var_F = (0xaadcaai32) as u16);
                slot!(game, Enemy_Shaktool[EK].base.current_instruction = shaktool_tables::piece(game, (i >> 1) as isize).head_bob);
                slot!(game, Enemy_Shaktool[EK].base.instruction_timer = 1);
                k = (k).wrapping_sub(64);
            }
        } else {
            slot!(game, Enemy_Shaktool[E].base.x_pos = v11);
            slot!(game, Enemy_Shaktool[E].base.y_pos = v12);
            if (slot!(game, Enemy_Shaktool[E].parameter_1) & 0x4000) != 0 {
                slot!(game, Enemy_Shaktool[E].var_A = (slot!(game, Enemy_Shaktool[E].var_A)).wrapping_add(256));
            } else {
                if ((slot!(game, Enemy_Shaktool[E].var_A) ^ slot!(game, Enemy_Shaktool[E].var_B)) & 0xff00) == 0 {
                    actors::shaktool::set_curling_angles(k, game);
                    slot!(game, Enemy_Shaktool[E].var_D = 30720);
                    actors::shaktool::set_movement_options(k, (slot!(game, Enemy_Shaktool[E].parameter_1)) & (0xdfffu16), game);
                    slot!(game, Enemy_Shaktool[E].var_D = ((slot!(byte game, Enemy_Shaktool[E].var_D[1]) as i32) << 8) as u16);
                }
                let v2: u16 = (slot!(game, Enemy_Shaktool[E].var_D)).wrapping_add(slot!(game, Enemy_Shaktool[E].var_C));
                slot!(game, Enemy_Shaktool[E].var_D = v2);
                if v2 >= 0xf000 {
                    actors::shaktool::set_movement_options(k, (slot!(game, Enemy_Shaktool[E].parameter_1)) | (0x4000u16), game);
                }
            }
        };
    };
}
