use crate::game::actors;
use crate::game::actors::api as actor_api;
use crate::game::music::QueueMusic_Delayed8;
use crate::game::palette_copies::{copy_rom_colors_to_palette, copy_rom_colors_to_target_palette};
pub use crate::game::prelude::*;
use crate::game::rom_vram_entries::queue_mother_brain_sprite_tiles;
use crate::game::session_runtime::Runtime;
use crate::game::shitroid_tables;
use sm_game::actors::corpses;
use sm_game::actors::mother_brain::{MotherBrainBody, MotherBrainBrain};
use sm_game::enemy_script::EnemyScript;

pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a6::*;
pub use crate::game::constants::bank_a9::*;

use crate::game::enemy_projectiles::{
    Math_MultByCos, Math_MultBySin, SpawnEprojWithGfx, SpawnEprojWithRoomGfx,
};
use crate::game::mother_brain_samus_vitals as vitals;
use crate::game::mother_brain_visuals;
use crate::game::music::{QueueSfx1_Max6, QueueSfx2_Max6, QueueSfx3_Max3, QueueSfx3_Max6};
use crate::game::room_mechanisms::spawn_hardcoded;
use crate::game::room_state::load_fx_entry as LoadFxEntry;
use crate::game::session_runtime::player_command as CallSomeSamusCode;
use crate::game::session_runtime::player_mother_brain;
use sm_game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;

use crate::game::sm_a6::{HomingReversal, homing_velocity};

use crate::ram_map::slot;

pub use crate::types::Consts_B7;

crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct MotherBrainInstrExecState {
        pub ip, timer: u16;
    }
}

crate::type_aliases! {
    pub Enemy_Shitroid, Enemy_DeadTorizo, Enemy_DeadSidehopper =
        crate::game::enemy_layouts::EnemyLayout40;
}

pub type Enemy_DeadMonsters = crate::game::enemy_layouts::EnemyLayout72Late;

pub type Enemy_ShitroidInCutscene = crate::game::enemy_layouts::EnemyLayout72;

pub type Enemy_DeadEnemy = crate::game::enemy_layouts::EnemyLayout40Late;

crate::game::constants::typed_constants! { i32;
    kCoroutineNone = 0,
    fnEproj_Init_0x8bde_SkreeDownLeft = 0x868b15, fnMotherBrainsBody_UnpauseHook = 0xa98763,
    fnMotherBrainsBody_FirstPhase_DoubleRet = 0xa987e1,
}
crate::game::constants::typed_constants! { u32;
    fnMotherBrainBody_FakeDeath_Descent_0_Pause = 11110429,
}
static g_byte_A9F56A: [i8; 16] = [
    0x10, 0xf, 0xe, 0xd, 0xc, 0xb, 0xa, 9, 8, 7, 6, 5, 4, 3, 2, 1,
];
/// Mother Brain's fight runs in `sm_game::actors::mother_brain_fight`.
pub use sm_game::actors::mother_brain_fight::*;
/// Kept unsafe for the player collision ownership checks, which wrap it in an unsafe block.
pub unsafe fn MotherBrain_MoveSamusVerticallyTowardsCeilingFloor(a: u16, game: &mut Game) -> u8 {
    move_samus_vertically_towards_ceiling_floor(a, game)
}
pub(crate) fn move_samus_vertically_towards_ceiling_floor(mut a: u16, game: &mut Game) -> u8 {
    {
        let carry: i32 = game.ram().read_byte(0xafc + 1) as i32 + a.to_ne_bytes()[0] as i32;
        {
            let value: u8 = carry as u8;
            let at: usize = 0xafc + 1;
            game.ram().write_byte(at, value);
        };
        {
            let value: u8 = carry as u8;
            let at: usize = 0xb16 + 1;
            game.ram().write_byte(at, value);
        };
        let v4: u16 =
            (word!(game.samus_y_pos) as i32 + (a >> 8) as i8 as i32 + (carry >> 8)) as u16;
        let mut v6: u16 = if v4 as i32 - 48i32 & 0x8000 != 0 {
            48
        } else {
            if v4 as i32 - 192i32 & 0x8000 != 0 {
                {
                    let at: usize = 0xafa;
                    let bytes: [u8; 2] = (v4).to_le_bytes();
                    game.ram().write_byte(at, bytes[0]);
                    game.ram().write_byte(at + 1, bytes[1]);
                }
                word!(
                    game.samus_prev_y_pos = u16::from_le_bytes([
                        game.ram().read_byte(0xafa),
                        game.ram().read_byte(0xafb)
                    ])
                );
                return 0;
            }
            192
        };
        {
            let at: usize = 0xafa;
            let bytes: [u8; 2] = (v6).to_le_bytes();
            game.ram().write_byte(at, bytes[0]);
            game.ram().write_byte(at + 1, bytes[1]);
        }
        word!(
            game.samus_prev_y_pos =
                u16::from_le_bytes([game.ram().read_byte(0xafa), game.ram().read_byte(0xafb)])
        );
        word! { game, 0; samus_y_subpos, samus_prev_y_subpos }
        1
    }
}
/// Kept unsafe for the player collision ownership checks, which wrap it in an unsafe block.
pub unsafe fn MotherBrain_MoveSamusHorizTowardsWall(a: u16, game: &mut Game) -> u8 {
    move_samus_horizontally_towards_wall(a, game)
}
pub(crate) fn move_samus_horizontally_towards_wall(mut a: u16, game: &mut Game) -> u8 {
    {
        let carry: i32 = game.ram().read_byte(0xaf8 + 1) as i32 + a.to_ne_bytes()[0] as i32;
        {
            let value: u8 = carry as u8;
            let at: usize = 0xaf8 + 1;
            game.ram().write_byte(at, value);
        };
        {
            let value: u8 = carry as u8;
            let at: usize = 0xb12 + 1;
            game.ram().write_byte(at, value);
        };
        let v4: u16 =
            (word!(game.samus_x_pos) as i32 + (a >> 8) as i8 as i32 + (carry >> 8)) as u16;
        if v4 as i32 - 235i32 & 0x8000 != 0 {
            word! {game; samus_x_pos = v4; samus_prev_x_pos = v4; }
            0
        } else {
            word! { game, 235; samus_x_pos, samus_prev_x_pos }
            word! { game, 0; samus_x_subpos, samus_prev_x_subpos }
            1
        }
    }
}
crate::forward_calls! { pub fn (a: u16, r18: u16, game: &mut Game) -> u16;
    ComputeSinMult => sub_A9C46C(r18, a, game);
    ComputeCosMult => sub_A9C46C((r18).wrapping_add(64), a, game);
}
pub fn sub_A9C46C(a: u16, j: u16, game: &mut Game) -> u16 {
    {
        let mut v2: u16 = kSinCosTable8bit_Sext[((a as i32 & 0xff) + 64) as usize] as u16;
        // Hardware multiply through the PPU's Mode 7 registers.
        unsafe {
            crate::rtl::register_writes! { game;
                byte(M7A as u16, v2 as u8); byte(M7A as u16, v2.to_ne_bytes()[1]);
                byte(M7B as u16, j as u8);
            }
            ReadRegWord(MPYM as u16, game)
        }
    }
}
crate::game::callback_dispatch! {
    pub fn CallShitroidCutsceneFuncA(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub fn (k: u16, game: &mut Game)];
    [11126764] ShitroidInCutscene_CurveTowardsBrain [] {
        ShitroidInCutscene_UpdateSpeedAndAngle(k, -384i32 as u16, -20480i32 as u16, 2560, game);
        let E: i32 = k as i32;
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = slot!(game, Enemy_ShitroidInCutscene[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_ShitroidInCutscene[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9c811i32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = 9);
        }
    };
    [11126801] ShitroidInCutscene_GetIntoFace [] {
        ShitroidInCutscene_UpdateSpeedAndAngle(k, -1536i32 as u16, -32256i32 as u16, 3584, game);
        let mut E: i32 = 0x40;
        let rect: Rect16U = Rect16U {
            x: slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos),
            y: slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos),
            w: 4,
            h: 4,
        };
        if actors::shitroid::separated_from_rect(k, rect, game) == 0 || {
            E = k as i32;
            let mut v3: bool = ((slot!(game, Enemy_ShitroidInCutscene[E].var_F)) as i16).wrapping_sub(1) < 0;
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = slot!(game, Enemy_ShitroidInCutscene[E].var_F).wrapping_sub(1));
            v3 as i32 != 0
        } {
            slot!(game, Enemy_ShitroidInCutscene[k as i32].var_A = (0xa9c851i32) as u16);
            player_mother_brain(1, game);
        }
    };
    [11126865] ShitroidInCutscene_LatchOntoBrain [] {
        let E: i32 = 0x40;
        let rect: Rect16U = Rect16U {
            x: slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos),
            y: ((slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos)).wrapping_sub(24)),
            w: 8,
            h: 8,
        };
        Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0, rect.x, rect.y, game);
        if actors::shitroid::separated_from_rect(k, rect, game) == 0 {
            slot!(game, Enemy_ShitroidInCutscene[k as i32].var_A = (0xa9c879i32) as u16);
        }
    };
    [11126905] ShitroidInCutscene_SetMotherBrainToStumbleBack [] {
        let E: i32 = 0;
        MotherBrain_MakeWalkBackwards((slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos)).wrapping_sub(1), 2, game);
        slot!(game, Enemy_ShitroidInCutscene[k as i32].var_A = (0xa9c889i32) as u16);
        ShitroidInCutscene_ActivateRainbowBeam(k, game);
    };
    [11126921] ShitroidInCutscene_ActivateRainbowBeam [] {
        let E1: i32 = 0x40;
        if Shitroid_AccelerateTowardsPoint(
            k,
            0x200,
            slot!(game, Enemy_ShitroidInCutscene[E1].base.x_pos),
            (slot!(game, Enemy_ShitroidInCutscene[E1].base.y_pos)).wrapping_sub(24),
            game,
        ) & 1
            != 0
        {
            let E: i32 = k as i32;
            slot! { game, Enemy_ShitroidInCutscene[E]; var_B = 0, var_C = 0 }
            slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos = slot!(game, Enemy_ShitroidInCutscene[E1].base.x_pos));
            slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos = (slot!(game, Enemy_ShitroidInCutscene[E1].base.y_pos)).wrapping_sub(24));
            Enemy_SetInstrList(k, addr_kShitroid_Ilist_CFB8 as u16, game);
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9c8e2i32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_E = 1);
            let E0: i32 = 0;
            slot!(game, Enemy_ShitroidInCutscene[E0].var_A = (0xa9be38i32) as u16);
            QueueSfx1_Max6(0x40, game);
            slot!(game, Enemy_ShitroidInCutscene[E0].var_16 = 1);
        }
    };
    [11127061] ShitroidInCutscene_StopDraining [] {
        let E: i32 = k as i32;
        let E1: i32 = 0x40;
        slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos = slot!(game, Enemy_ShitroidInCutscene[E1].base.x_pos));
        slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos = (slot!(game, Enemy_ShitroidInCutscene[E1].base.y_pos)).wrapping_sub(24));
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = slot!(game, Enemy_ShitroidInCutscene[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_ShitroidInCutscene[E].var_F) & 0x8000) != 0 {
            Enemy_SetInstrList(k, addr_kShitroid_Ilist_CFA2 as u16, game);
            slot!(game, Enemy_ShitroidInCutscene[E].var_E = 10);
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9c94bi32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = 32);
            slot! { game, Enemy_ShitroidInCutscene[E]; var_B = 0, var_C = 0 }
        }
    };
    [11127129] ShitroidInCutscene_MoveUpToCeiling [] {
        let rect: Rect16U = Rect16U {
            x: slot!(game, Enemy_ShitroidInCutscene[0x40i32].base.x_pos),
            y: 0,
            w: 4,
            h: 4,
        };
        Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0, rect.x, rect.y, game);
        if actors::shitroid::separated_from_rect(k, rect, game) == 0 {
            player_mother_brain(4, game);
            let E: i32 = k as i32;
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9c9c3i32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_0E = addr_stru_A9CA24 as u16);
        }
    };
    [11127398] ShitroidInCutscene_LatchOntoSamus [] {
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            k,
            0,
            word!(game.samus_x_pos),
            (word!(game.samus_y_pos)).wrapping_sub(20),
            game,
        );
    };
    [11127485] ShitroidInCutscene_IdleUntilToNoHealth [] {
        ShitroidInCutscene_HandleCry(game);
        let E: i32 = k as i32;
        let sice_var_06: u16 = slot!(game, Enemy_ShitroidInCutscene[E].var_06);
        if sice_var_06 != 0 {
            let v3: i32 = (sice_var_06 as i32 & 6) >> 1;
            slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos = (word!(game.samus_x_pos) as i32 + 2 * g_word_A993BB[v3 as usize] as i32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos = (word!(game.samus_y_pos) as i32
                + 2 * g_word_A993C3[v3 as usize] as i32
                - 20i32) as u16);
        }
        if slot!(game, Enemy_ShitroidInCutscene[E].base.health) == 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].base.health = 320);
            slot! { game, Enemy_ShitroidInCutscene[E]; var_A = (0xa9cb13i32) as u16, var_E = 10, var_D = 0, var_0F = (0xa9f68fi32) as u16, var_04 = 1, var_09 = 0 };
        }
    };
    [11127571] ShitroidInCutscene_ReleaseSamus [] {
        QueueSfx2_Max6(0x72, game);
        slot!(game, Enemy_ShitroidInCutscene[0x40i32].var_0B = 1);
        slot!(game, Enemy_ShitroidInCutscene[k as i32].var_A = (0xa9cb2di32) as u16);
        slot!(game, Enemy_ShitroidInCutscene[0i32].var_A = (0xa9c18ei32) as u16);
        ShitroidInCutscene_StareDownMotherBrain(k, game);
    };
    [11127597] ShitroidInCutscene_StareDownMotherBrain [] {
        let rect: Rect16U = Rect16U {
            x: ((word!(game.samus_x_pos)).wrapping_sub(4)),
            y: 96,
            w: 4,
            h: 4,
        };
        Shitroid_GraduallyAccelerateTowards0x10(k, 0, rect.x, rect.y, game);
        if actors::shitroid::separated_from_rect(k, rect, game) == 0 {
            slot!(game, Enemy_ShitroidInCutscene[k as i32].var_A = (0xa9cb56i32) as u16);
        }
    };
    [11127638] ShitroidInCutscene_FlyOffScreen [] {
        let rect: Rect16U = Rect16U {
            x: 272,
            y: 64,
            w: 4,
            h: 4,
        };
        Shitroid_GraduallyAccelerateTowards0x10(k, 0, rect.x, rect.y, game);
        if actors::shitroid::separated_from_rect(k, rect, game) == 0 {
            slot!(game, Enemy_ShitroidInCutscene[k as i32].var_A = (0xa9cb7bi32) as u16);
        }
    };
    [11127675] ShitroidInCutscene_MoveToFinalChargeStart [] {
        let rect: Rect16U = Rect16U {
            x: 305,
            y: 160,
            w: 4,
            h: 4,
        };
        Shitroid_GraduallyAccelerateTowards0x10(k, 0, rect.x, rect.y, game);
        if actors::shitroid::separated_from_rect(k, rect, game) == 0 {
            let E: i32 = k as i32;
            slot!(game, Enemy_ShitroidInCutscene[E].base.health = 79);
            slot!(game, Enemy_ShitroidInCutscene[0x40i32].var_0B = 0);
            slot!(game, Enemy_ShitroidInCutscene[0i32].var_A = (0xa9c19ai32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9cbb3i32) as u16);
        }
    };
    [11127731] ShitroidInCutscene_InitiateFinalCharge [] {
        let rect: Rect16U = Rect16U {
            x: 290,
            y: 128,
            w: 4,
            h: 4,
        };
        Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0xa, rect.x, rect.y, game);
        if actors::shitroid::separated_from_rect(k, rect, game) == 0 {
            slot!(game, Enemy_ShitroidInCutscene[k as i32].var_A = (0xa9cbd8i32) as u16);
        }
    };
    [11127768] ShitroidInCutscene_FinalCharge [] {
        let E1: i32 = 0x40;
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            k,
            0xc,
            slot!(game, Enemy_ShitroidInCutscene[E1].base.x_pos),
            (slot!(game, Enemy_ShitroidInCutscene[E1].base.y_pos)).wrapping_sub(32),
            game,
        );
        let E: i32 = k as i32;
        if slot!(game, Enemy_ShitroidInCutscene[E].base.health) == 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].var_0F = (0xa9cc98i32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].base.vram_tiles_index = 4256);
            QueueSfx3_Max6(0x19, game);
            unsafe { mother_brain_visuals::lights_off(game) };
            Enemy_SetInstrList(k, addr_kShitroid_Ilist_CFCE as u16, game);
            slot! { game, Enemy_ShitroidInCutscene[E]; var_B = 0, var_C = 0 }
            slot!(game, Enemy_ShitroidInCutscene[0i32].var_A = addr_locret_A9C18D as u16);
            QueueMusic_Delayed8(0, game);
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9cc3ei32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = 16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_10 = slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos));
            slot!(game, Enemy_ShitroidInCutscene[E].var_11 = slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos));
            ShitroidInCutscene_ShitroidFinalBelow(k, game);
        }
    };
    [11127870] ShitroidInCutscene_ShitroidFinalBelow [] {
        ShitroidInCutscene_Shake(k, game);
        let E: i32 = k as i32;
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = slot!(game, Enemy_ShitroidInCutscene[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_ShitroidInCutscene[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos = slot!(game, Enemy_ShitroidInCutscene[E].var_10));
            slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos = slot!(game, Enemy_ShitroidInCutscene[E].var_11));
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9cc60i32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = 56);
            ShitroidInCutscene_PlaySamusTheme(k, game);
        }
    };
    [11127904] ShitroidInCutscene_PlaySamusTheme [] {
        let E: i32 = k as i32;
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = slot!(game, Enemy_ShitroidInCutscene[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_ShitroidInCutscene[E].var_F) & 0x8000) != 0 {
            QueueMusic_Delayed8(0xff48, game);
            QueueMusic_Delayed8(5, game);
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9cc7fi32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = 12);
            ShitroidInCutscene_PrepareSamusHyperbeam(k, game);
        }
    };
    [11127961] ShitroidInCutscene_DeathSequence [] {
        HandleSamusRainbowPaletteAnimation(k, game);
        ShitroidInCutscene_AccelerateDownwards(k, game);
        if ShitroidInCutscene_FadeShitroidToBlack(k, game) & 1 != 0 {
            let E: i32 = k as i32;
            slot!(game, Enemy_ShitroidInCutscene[E].base.properties = (slot!(game, Enemy_ShitroidInCutscene[E].base.properties)) | ((kEnemyProps_Invisible) as u16));
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9ccc0i32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = 128);
        } else {
            ShitroidInCutscene_HandleShitroidDeathExplosions(k, game);
            ShitroidInCutscene_HandleEnemyBlinking(k, game);
        };
    };
    [11128000] ShitroidInCutscene_UnloadShitroid [] {
        HandleSamusRainbowPaletteAnimation(k, game);
        let E: i32 = k as i32;
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = slot!(game, Enemy_ShitroidInCutscene[E].var_F).wrapping_sub(1));
        let v2: bool = (slot!(game, Enemy_ShitroidInCutscene[E].var_F) & 0x8000) != 0;
        if v2 && queue_mother_brain_sprite_tiles(0xa9, addr_stru_A98FC7 as u16, game) != 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9ccdei32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = 176);
            ShitroidInCutscene_LetSamusRainbowMore(k, game);
        }
    };
    [11128048] ShitroidInCutscene_FinishCutscene [] {
        let E: i32 = 0;
        let v2: u16 = (slot!(game, Enemy_ShitroidInCutscene[E].var_37)).wrapping_add(1);
        slot!(game, Enemy_ShitroidInCutscene[E].var_37 = v2);
        if (unsafe { mother_brain_visuals::fade(mother_brain_visuals::Fade::LightsOn, (v2).wrapping_sub(1), game) }) & 1 != 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9c1cfi32) as u16);
            CallSomeSamusCode(0x17, game);
            player_mother_brain(3, game);
            let fresh12: i32 = k as i32;
            slot!(game, Enemy_ShitroidInCutscene[fresh12].base.properties, |old| old | (kEnemyProps_Deleted as u16));
            slot!(game, Enemy_ShitroidInCutscene[0x40i32].var_0A = 0);
        }
    };
    @functions [pub fn (k: u16, game: &mut Game)];
    @prefix { let E: i32 = k as i32; };
    [11127010] ShitroidInCutscene_BrainTurnsToCorpse [] {
        let E1: i32 = 0x40;
        let v2: i32 = (slot!(game, Enemy_ShitroidInCutscene[E].base.frame_counter) as i32 & 6) >> 1;
        slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos = (slot!(game, Enemy_ShitroidInCutscene[E1].base.x_pos)).wrapping_add((g_word_A993BB[v2 as usize]) as u16));
        slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos = (slot!(game, Enemy_ShitroidInCutscene[E1].base.y_pos) as i32 + g_word_A993C3[v2 as usize] as i32 - 24i32) as u16);
        if slot!(game, Enemy_ShitroidInCutscene[0i32].var_1F) != 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9c915i32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_F = 64);
        }
    };
    [11127115] ShitroidInCutscene_LetGoAndSpawnDust [] {
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = slot!(game, Enemy_ShitroidInCutscene[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_ShitroidInCutscene[E].var_F) & 0x8000) != 0 {
            ShitroidInCutscene_SpawnThreeDustClouds(game);
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = addr_loc_A9C959 as u16);
        }
        ShitroidInCutscene_MoveUpToCeiling(k, game);
    };
    [11127235] ShitroidInCutscene_MoveToSamus [] {
        slot!(game, Enemy_ShitroidInCutscene[E].var_04 = 0);
        slot!(game, Enemy_ShitroidInCutscene[E].var_09 = 1);
        if (word!(game.random_number) & 0xfff) >= 0xfa0 {
            QueueSfx2_Max6(0x52, game);
        }
        let sice_var_0E: u16 = slot!(game, Enemy_ShitroidInCutscene[E].var_0E);
        let v3 = crate::game::session_runtime::native_record(0xa9, sice_var_0E, game);
        let rect: Rect16U = Rect16U {
            x: v3.word(0),
            y: v3.word(2),
            w: 4,
            h: 4,
        };
        CallShitroidMoveFunc(
            (v3.word(6) as u32) | (0xa90000u32),
            k,
            v3.word(4),
            rect.x,
            rect.y,
            game,
        );
        if actors::shitroid::separated_from_rect(k, rect, game) == 0 {
            let mut v4: i16 = crate::game::session_runtime::native_record(0xa9, sice_var_0E, game).word(8) as i16;
            if v4 < 0 {
                slot!(game, Enemy_ShitroidInCutscene[E].var_A = v4 as u16);
            } else {
                slot!(game, Enemy_ShitroidInCutscene[E].var_0E = (slot!(game, Enemy_ShitroidInCutscene[E].var_0E)).wrapping_add(8));
            }
        }
    };
    [11127418] ShitroidInCutscene_HealSamusToFullHealth [] {
        slot!(game, Enemy_ShitroidInCutscene[E].var_04 = 0);
        ShitroidInCutscene_HandleCry(game);
        let v2: i32 = (slot!(game, Enemy_ShitroidInCutscene[E].base.frame_counter) as i32 & 6) >> 1;
        slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos = (word!(game.samus_x_pos)).wrapping_add((g_word_A993BB[v2 as usize]) as u16));
        slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos = (word!(game.samus_y_pos) as i32 + g_word_A993C3[v2 as usize] as i32 - 20i32) as u16);
        if !vitals::heal_by_baby_metroid(game) {
            word!(game.samus_reserve_health = word!(game.samus_max_reserve_health));
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9cabdi32) as u16);
            slot!(game, Enemy_ShitroidInCutscene[E].var_0F = (0xa9cabci32) as u16);
        }
    };
    ShitroidInCutscene_HandleShitroidDeathExplosions [] {
        let mut v2: i16 = ((slot!(game, Enemy_ShitroidInCutscene[E].var_08)) as i16).wrapping_sub(1);
        if v2 < 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].var_08 = 4);
            let mut v3: u16 = (slot!(game, Enemy_ShitroidInCutscene[E].var_07)).wrapping_add(1);
            if slot!(game, Enemy_ShitroidInCutscene[E].var_07) as i32 - 9i32 & 0x8000 == 0 {
                v3 = 0;
            }
            slot!(game, Enemy_ShitroidInCutscene[E].var_07 = v3);
            let v4: i32 = ((4u16).wrapping_mul(v3)) as i32 >> 1;
            crate::game::sm_a6::set_explosion_spawn_offset!(game, E, 0xa9cdfc, v4);
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 3, game);
            QueueSfx3_Max3(0x13, game);
        } else {
            slot!(game, Enemy_ShitroidInCutscene[E].var_08 = v2 as u16);
        };
    };
    ShitroidInCutscene_HandleHealthBasedPalette [] {
        if slot!(game, Enemy_ShitroidInCutscene[E].var_09) != 0 {
            let health: u16 = slot!(game, Enemy_ShitroidInCutscene[E].base.health);
            if health as i32 - 2560i32 & 0x8000 != 0 {
                let mut v3: u16 = 2;
                if health as i32 - 2240i32 & 0x8000 != 0 {
                    v3 = 4;
                    if health as i32 - 1920i32 & 0x8000 != 0 {
                        v3 = 6;
                        if health as i32 - 1600i32 & 0x8000 != 0 {
                            v3 = 8;
                            if health as i32 - 1280i32 & 0x8000 != 0 {
                                v3 = 10;
                                if health as i32 - 960i32 & 0x8000 != 0 {
                                    v3 = 12;
                                    if health as i32 - 640i32 & 0x8000 != 0 {
                                        v3 = 14;
                                    }
                                }
                            }
                        }
                    }
                }
                let v5: u16 = v3;
                let v4: u16 = shitroid_tables::shell_palette(game, (v3 >> 1) as isize).colors;
                copy_rom_colors_to_palette(0x1e2, 0xad, v4, 4, game);
                copy_rom_colors_to_palette(0x1f4, 0xad, (v4).wrapping_add(8), 5, game);
                copy_rom_colors_to_palette(
                    0x1ea,
                    0xad,
                    shitroid_tables::innards_palette(game, (v5 >> 1) as isize).colors,
                    5,
                    game,
                );
            }
        }
    };
    ShitroidInCutscene_Shake [] {
        slot!(game, Enemy_ShitroidInCutscene[E].var_C = slot!(game, Enemy_ShitroidInCutscene[E].var_C).wrapping_sub(1));
        let v2: i32 = (slot!(game, Enemy_ShitroidInCutscene[E].base.frame_counter) as i32 & 6) >> 1;
        slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos = (slot!(game, Enemy_ShitroidInCutscene[E].var_10)).wrapping_add((g_word_A993BB[v2 as usize]) as u16));
        slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos = (slot!(game, Enemy_ShitroidInCutscene[E].var_11)).wrapping_add((g_word_A993C3[v2 as usize]) as u16));
    };
    @countdown [pub fn (k: u16, game: &mut Game)];
    before {
        let E: i32 = k as i32;
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = slot!(game, Enemy_ShitroidInCutscene[E].var_F).wrapping_sub(1));
    }
    when (slot!(game, Enemy_ShitroidInCutscene[E].var_F) & 0x8000) != 0;
    [11126732] ShitroidInCutscene_DashOntoScreen {
        slot!(game, Enemy_ShitroidInCutscene[E].var_0A = -10240i32 as u16);
        slot!(game, Enemy_ShitroidInCutscene[E].var_0B = 2560);
        slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9c7eci32) as u16);
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = 10);
        ShitroidInCutscene_CurveTowardsBrain(k, game);
    };
    [11127935] ShitroidInCutscene_PrepareSamusHyperbeam {
        CallSomeSamusCode(0x19, game);
        slot!(game, Enemy_ShitroidInCutscene[0x40i32].var_16 = (0xa9cd30i32) as u16);
        slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9cc99i32) as u16);
    };
    [11128030] ShitroidInCutscene_LetSamusRainbowMore {
        slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9ccf0i32) as u16);
        slot!(game, Enemy_ShitroidInCutscene[0i32].var_37 = 0);
        ShitroidInCutscene_FinishCutscene(k, game);
    };
    @external {
        11125133 => [()];
    }
}
crate::game::callback_dispatch! {
    pub fn CallShitroidCutsceneFunc0F(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @forward [pub fn (game: &mut Game)];
    Shitroid_HandleNormalPalette => Shitroid_HandleCutscenePalette_Common(addr_word_A9F6D1 as u16, 330, game);
    [11138691]
    Shitroid_HandleCutscenePalette => Shitroid_HandleCutscenePalette_Common(addr_word_A9F6D1 as u16, 490, game);
    [11138703]
    Shitroid_HandleCutscenePalette_LowHealth => Shitroid_HandleCutscenePalette_Common(addr_word_A9F711 as u16, 490, game);
    @external {
        11127484 => [()]; 11127960 => [()];
    }
}
pub fn ShitroidInCutscene_Flashing(k: u16, game: &mut Game) {
    let mut v1: i16 = 3584;
    let E: i32 = k as i32;
    let sice_var_06: u16 = slot!(game, Enemy_ShitroidInCutscene[E].var_06);
    if sice_var_06 != 0 {
        let v4: u16 = (sice_var_06).wrapping_sub(1);
        slot!(game, Enemy_ShitroidInCutscene[E].var_06 = v4);
        if (v4 & 2) != 0 {
            v1 = 0;
        }
    }
    slot!(
        game,
        Enemy_ShitroidInCutscene[E].base.palette_index = v1 as u16
    );
}
pub fn ShitroidInCutscene_HandleCry(game: &mut Game) {
    let E: i32 = 0;
    if slot!(game, Enemy_ShitroidInCutscene[E].var_14) != 0 {
        slot!(game, Enemy_ShitroidInCutscene[E].var_14 = 0);
        QueueSfx2_Max6(0x72, game);
    }
}
pub fn ShitroidInCutscene_SpawnOneDustCloudAt(xd: u16, yd: u16, game: &mut Game) {
    let E: i32 = 0x40;
    game.eproj_spawn_pt = Point16U {
        x: ((slot!(game, Enemy_ShitroidInCutscene[E].base.x_pos)).wrapping_add(xd)),
        y: ((slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos)).wrapping_add(yd)),
    };
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, game);
}
pub fn ShitroidInCutscene_SpawnThreeDustClouds(game: &mut Game) {
    {
        ShitroidInCutscene_SpawnOneDustCloudAt(-16i32 as u16, -8i32 as u16, game);
        ShitroidInCutscene_SpawnOneDustCloudAt(0, -16i32 as u16, game);
        ShitroidInCutscene_SpawnOneDustCloudAt(16, -8i32 as u16, game);
    }
}
crate::game::callback_dispatch! {
    pub fn CallShitroidMoveFunc(ea: u32, k: u16, j: u16, r18: u16, r20: u16, game: &mut Game) => ea;
    call (k, j, r18, r20, game); default { Unreachable(game); };
    @forward [pub fn ( k: u16, j: u16, r18: u16, r20: u16, game: &mut Game, )];
    Shitroid_Func_GraduallyAccelerateTowards0x400 => Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 1024, game);
    Shitroid_GraduallyAccelerateTowards0x4 => Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 4, game);
    [11138143]
    Shitroid_GraduallyAccelerateTowards0x8 => Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 8, game);
    [11138150]
    Shitroid_GraduallyAccelerateTowards0x10 => Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 16, game);
}
crate::game::callback_dispatch! {
    pub fn CallSamusRainbowPaletteFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub fn (k: u16, game: &mut Game)];
    [11128112] SamusRainbowPaletteFunc_ActivateWhenEnemyLow [] {
        if ((slot!(game, Enemy_ShitroidInCutscene[k as i32].base.y_pos) as i32 + 16
            - word!(game.samus_y_pos) as i32) as i16)
            >= 0
        {
            CallSomeSamusCode(0x16, game);
            slot!(game, Enemy_ShitroidInCutscene[0x40i32].var_16 = (0xa9cd4bi32) as u16);
        }
    };
    @external {
        11128139 => [SamusRainbowPaletteFunc_GraduallySlowDown(game)];
    }
}
pub fn HandleSamusRainbowPaletteAnimation(k: u16, game: &mut Game) {
    {
        let r18: u16 = slot!(game, Enemy_ShitroidInCutscene[0x40i32].var_16);
        CallSamusRainbowPaletteFunc(((r18) as u32) | (0xa90000u32), k, game);
    }
}
pub fn SamusRainbowPaletteFunc_GraduallySlowDown(game: &mut Game) {
    let E: i32 = 0x40;
    let sice_var_1D: u16 = slot!(game, Enemy_ShitroidInCutscene[E].var_1D);
    slot!(
        game,
        Enemy_ShitroidInCutscene[E].var_1D = (sice_var_1D).wrapping_add(768)
    );
    if sice_var_1D >= 0xfd00 {
        let mut v2: u16 = (word!(game.special_samus_palette_frame)).wrapping_add(1);
        if word!(game.special_samus_palette_frame) as i32 - 9i32 & 0x8000 == 0 {
            v2 = 10;
        }
        word!(game.special_samus_palette_frame = v2);
    }
}
pub fn ShitroidInCutscene_FadeShitroidToBlack(k: u16, game: &mut Game) -> u8 {
    {
        let E: i32 = k as i32;
        if slot!(game, Enemy_ShitroidInCutscene[E].base.y_pos) as i32 - 128i32 & 0x8000 != 0 {
            return 0;
        }
        let mut v2: i16 =
            ((slot!(game, Enemy_ShitroidInCutscene[E].var_0C)) as i16).wrapping_sub(1);
        if v2 >= 0 {
            slot!(game, Enemy_ShitroidInCutscene[E].var_0C = v2 as u16);
            return 0;
        }
        slot!(game, Enemy_ShitroidInCutscene[E].var_0C = 8);
        let v4: u16 = (slot!(game, Enemy_ShitroidInCutscene[E].var_0D)).wrapping_add(1);
        if slot!(game, Enemy_ShitroidInCutscene[E].var_0D) as i32 - 6i32 & 0x8000 == 0 {
            return 1;
        }
        slot!(game, Enemy_ShitroidInCutscene[E].var_0D = v4);
        copy_rom_colors_to_palette(
            0x1e2,
            0xad,
            shitroid_tables::fade_palette(game, v4 as isize).colors,
            0xe,
            game,
        );
        0
    }
}
pub fn ShitroidInCutscene_HandleEnemyBlinking(k: u16, game: &mut Game) {
    let E: i32 = k as i32;
    if (slot!(game, Enemy_ShitroidInCutscene[E].base.frame_counter) & 1) != 0 {
        slot!(
            game,
            Enemy_ShitroidInCutscene[E].base.properties =
                (slot!(game, Enemy_ShitroidInCutscene[E].base.properties) as i32
                    & !(kEnemyProps_Invisible as i32)) as u16
        );
    } else {
        slot!(
            game,
            Enemy_ShitroidInCutscene[E].base.properties =
                (slot!(game, Enemy_ShitroidInCutscene[E].base.properties))
                    | ((kEnemyProps_Invisible) as u16)
        );
    };
}
pub fn ShitroidInCutscene_AccelerateDownwards(k: u16, game: &mut Game) {
    let E: i32 = k as i32;
    let mut v2: i16 =
        ((abs16(slot!(game, Enemy_ShitroidInCutscene[E].var_B))) as i16).wrapping_sub(32);
    if v2 < 0 {
        v2 = 0;
    }
    slot!(
        game,
        Enemy_ShitroidInCutscene[E].var_B =
            (if slot!(game, Enemy_ShitroidInCutscene[E].var_B) & 0x8000 != 0 {
                -(v2 as i32)
            } else {
                v2 as i32
            }) as u16
    );
    slot!(
        game,
        Enemy_ShitroidInCutscene[E].var_C =
            (slot!(game, Enemy_ShitroidInCutscene[E].var_C)).wrapping_add(2)
    );
}
pub fn ShitroidInCutscene_UpdateSpeedAndAngle(
    k: u16,
    mut r18: u16,
    r20: u16,
    r22: u16,
    game: &mut Game,
) {
    {
        let E: i32 = k as i32;
        crate::game::sm_a6::steer_enemy_speed_and_angle!(
            game,
            Enemy_ShitroidInCutscene,
            r22,
            E,
            r18,
            r20
        );
    }
}
crate::instruction_callbacks! { pub fn (_k: u16, _jp: EnemyScript) -> EnemyScript;
    Shitroid_Instr_1 [EnemyScript::jump(addr_kShitroid_Ilist_CFA2 as u16)] { };
    Shitroid_Instr_2 [EnemyScript::jump(addr_kShitroid_Ilist_CFB8 as u16)] { };
}

crate::game::callback_dispatch! {
    pub fn CallShitroidFunc(ea: u32, k: u16, game: &mut Game) => ea;
    call (k, game); default { Unreachable(game); };
    @functions [pub fn (k: u16, game: &mut Game)];
    [11137097] Shitroid_Func_7 [] {
        let rect: Rect16U = Rect16U {
            x: 584,
            y: 74,
            w: 1,
            h: 1,
        };
        Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0xf, rect.x, rect.y, game);
        if actors::shitroid::separated_from_rect(k, rect, game) == 0 {
            slot!(game, Enemy_Shitroid[k as i32].var_A = (0xa9f06di32) as u16);
        }
    };
    [11137133] Shitroid_Func_8 [] {
        let E: i32 = k as i32 + 64;
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            k,
            0xf,
            slot!(game, Enemy_Shitroid[E].base.x_pos),
            (slot!(game, Enemy_Shitroid[E].base.y_pos)).wrapping_sub(32),
            game,
        );
        if actors::shitroid::separated_from_enemy(k, (k).wrapping_add(64), game) & 1 == 0 {
            slot!(game, Enemy_Shitroid[k as i32].var_A = (0xa9f094i32) as u16);
        }
    };
    [11137172] Shitroid_Func_9 [] {
        let E1: i32 = k as i32 + 64;
        if Shitroid_AccelerateTowardsPoint(
            k,
            0x200,
            slot!(game, Enemy_Shitroid[E1].base.x_pos),
            (slot!(game, Enemy_Shitroid[E1].base.y_pos)).wrapping_sub(32),
            game,
        ) & 1
            != 0
        {
            let E: i32 = k as i32;
            slot! { game, Enemy_Shitroid[E]; var_B = 0, var_C = 0 }
            slot!(game, Enemy_Shitroid[E].base.x_pos = slot!(game, Enemy_Shitroid[E1].base.x_pos));
            slot!(game, Enemy_Shitroid[E].base.y_pos = (slot!(game, Enemy_Shitroid[E1].base.y_pos)).wrapping_sub(32));
            slot! { game, Enemy_Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F924 as u16, instruction_timer = 1 }
            slot!(game, Enemy_Shitroid[E].base.timer = 0);
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9f0e6i32) as u16);
            slot!(game, Enemy_Shitroid[E].var_E = 1);
            slot!(game, Enemy_Shitroid[E].parameter_2 = 0);
            slot!(game, Enemy_Shitroid[E].var_F = 320);
        }
    };
    [11137317] Shitroid_Func_11 [] {
        slot!(game, Enemy_Shitroid[0x40i32].var_08 = 1);
        let E: i32 = k as i32;
        slot!(game, Enemy_Shitroid[E].var_A = (0xa9f138i32) as u16);
        slot!(game, Enemy_Shitroid[E].var_F = 192);
        Shitroid_Func_12(k, game);
    };
    [11137530] Shitroid_Func_14 [] {
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            k,
            0xf,
            word!(game.samus_x_pos),
            (word!(game.samus_y_pos)).wrapping_sub(32),
            game,
        );
    };
    [11137550] Shitroid_Func_15 [] {
        CallSomeSamusCode(0x12, game);
        slot!(game, Enemy_Shitroid[0i32].var_A = (0xa9f21bi32) as u16);
        Shitroid_Func_16(k, game);
    };
    @functions [pub fn (_k: u16, game: &mut Game)];
    [11137728] Shitroid_Func_19 [] {
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            word!(game.cur_enemy_index),
            0,
            word!(game.samus_x_pos),
            104,
            game,
        );
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_Shitroid[E].var_F = slot!(game, Enemy_Shitroid[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_Shitroid[E].var_F) & 0x8000) != 0 {
            QueueSfx2_Max6(0x7d, game);
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9f2fbi32) as u16);
            slot!(game, Enemy_Shitroid[E].var_F = 88);
            slot! { game, Enemy_Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F924 as u16, instruction_timer = 1 }
            slot!(game, Enemy_Shitroid[E].base.timer = 0);
            Shitroid_Func_20(game);
        }
    };
    @functions [pub fn (k: u16, game: &mut Game)];
    [11137888] Shitroid_Func_22 [] {
        QueueSfx2_Max6(0x52, game);
        slot!(game, Enemy_Shitroid[k as i32].var_A = (0xa9f36di32) as u16);
        Shitroid_Func_23(game);
    };
    [11137982] Shitroid_Func_25 [] {
        if Shitroid_Func_27(k, game) & 1 != 0 {
            Shitroid_F3C4(k, game);
        }
    };
    @functions [pub fn (k: u16, game: &mut Game)];
    @prefix { let E: i32 = k as i32; };
    [11136991] Shitroid_Func_3 [] {
        slot! { game, Enemy_Shitroid[E]; var_B = 0, var_C = 0 }
    };
    [11137067] Shitroid_Func_5 [] {
        slot!(game, Enemy_Shitroid[E].var_A = (0xa9f037i32) as u16);
        slot!(game, Enemy_Shitroid[E].var_F = 464);
        Shitroid_Func_6(k, game);
    };
    [11137254] Shitroid_Func_10 [] {
        let v2: i32 = (slot!(game, Enemy_Shitroid[E].base.frame_counter) as i32 & 6) >> 1;
        let E1: i32 = k as i32 + 64;
        slot!(game, Enemy_Shitroid[E].base.x_pos = (slot!(game, Enemy_Shitroid[E1].base.x_pos)).wrapping_add((g_word_A993BB[v2 as usize]) as u16));
        slot!(game, Enemy_Shitroid[E].base.y_pos = (slot!(game, Enemy_Shitroid[E1].base.y_pos) as i32 + g_word_A993C3[v2 as usize] as i32 - 32i32) as u16);
        let fresh13: u16 = slot!(game, Enemy_Shitroid[E].var_F);
        slot!(game, Enemy_Shitroid[E].var_F = slot!(game, Enemy_Shitroid[E].var_F).wrapping_sub(1));
        if fresh13 == 1 {
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9f125i32) as u16);
            slot! { game, Enemy_Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F906 as u16, instruction_timer = 1 }
            slot!(game, Enemy_Shitroid[E].base.timer = 0);
            slot!(game, Enemy_Shitroid[E].var_E = 10);
        }
    };
    [11137336] Shitroid_Func_12 [] {
        Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0, slot!(game, Enemy_Shitroid[E].base.x_pos), 104, game);
        slot!(game, Enemy_Shitroid[E].var_F = slot!(game, Enemy_Shitroid[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_Shitroid[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9f180i32) as u16);
            slot!(game, Enemy_Shitroid[E].parameter_2 = 1);
            { let at: usize = 0xcd20; let bytes: [u8; 2] = ((u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])) | (0x100u16)).to_le_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }
            { let at: usize = 0xcd20 + (2 ) as usize; let bytes: [u8; 2] = ((u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])) | (0x100u16)).to_le_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x30, 0x3, 0xb763 as VoidP), game);
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x1f, 0x3, 0xb763 as VoidP), game);
        }
    };
    [11137408] Shitroid_Func_13 [] {
        let v2: u16 = abs16((slot!(game, Enemy_Shitroid[E].base.x_pos)).wrapping_sub(word!(game.samus_x_pos)));
        let mut shitro_var_02: u16 = 0;
        if v2 >= 8 {
            shitro_var_02 = slot!(game, Enemy_Shitroid[E].var_02);
            if shitro_var_02 != 0 {
                shitro_var_02 = shitro_var_02.wrapping_sub(1);
                if (shitro_var_02 & 0x8000) != 0 {
                    shitro_var_02 = 0;
                }
            }
        } else {
            shitro_var_02 = ((v2 >= 8) as i32 + slot!(game, Enemy_Shitroid[E].var_02) as i32 + 2) as u16;
        }
        slot!(game, Enemy_Shitroid[E].var_02 = shitro_var_02);
        if shitro_var_02 >= 0x100 || word!(game.samus_x_pos) as i32 - 512i32 & 0x8000 != 0 {
            slot!(game, Enemy_Shitroid[word!(game.cur_enemy_index) as i32].var_A = (0xa9f1fai32) as u16);
        } else {
            let shitro_var_01: u16 = slot!(game, Enemy_Shitroid[E].var_01);
            let mut v5: u16 = 0;
            if shitro_var_01 != 0 {
                slot!(game, Enemy_Shitroid[E].var_01 = (shitro_var_01).wrapping_sub(1));
                v5 = word!(game.samus_y_pos);
            } else {
                v5 = 80;
                if (word!(game.random_number) & 0xfff) >= 0xfe0 {
                    slot!(game, Enemy_Shitroid[E].var_01 = 32);
                }
            }
            Shitroid_Func_GraduallyAccelerateTowards0x400(
                word!(game.cur_enemy_index),
                0xa,
                word!(game.samus_x_pos),
                v5,
                game,
            );
        };
    };
    [11137563] Shitroid_Func_16 [] {
        if word!(game.samus_health) < 2 {
            word! { game, 0; samus_x_speed_divisor, bomb_counter }
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9f2a2i32) as u16);
            slot! { game, Enemy_Shitroid[E]; var_B = 0, var_C = 0 }
            Enemy_SetInstrList(k, addr_kShitroid_Ilist_F906 as u16, game);
            slot!(game, Enemy_Shitroid[E].var_E = 10);
            CallSomeSamusCode(0x13, game);
            player_mother_brain(0, game);
            slot!(game, Enemy_Shitroid[E].var_04 = 0);
            QueueMusic_Delayed8(7, game);
        } else {
            word! {game; cooldown_timer = 8; bomb_counter = 5; samus_x_speed_divisor = 2; }
            if word!(game.samus_y_speed) as i32 - 4i32 & 0x8000 == 0 {
                word!(game.samus_y_speed = 2);
            }
            let v2: i32 = (slot!(game, Enemy_Shitroid[E].base.frame_counter) as i32 & 6) >> 1;
            slot!(game, Enemy_Shitroid[E].base.x_pos = (word!(game.samus_x_pos)).wrapping_add((g_word_A993BB[v2 as usize]) as u16));
            slot!(game, Enemy_Shitroid[E].base.y_pos = (word!(game.samus_y_pos) as i32 + g_word_A993C3[v2 as usize] as i32 - 20i32) as u16);
            vitals::drain_health_by_baby_metroid(game);
        };
    };
    [11137698] Shitroid_Func_17 [] {
        slot!(game, Enemy_Shitroid[E].var_A = (0xa9f2aei32) as u16);
        slot!(game, Enemy_Shitroid[E].var_F = 120);
        Shitroid_Func_18(k, game);
    };
    [11137955] Shitroid_Func_24 [] {
        slot!(game, Enemy_Shitroid[E].var_F = slot!(game, Enemy_Shitroid[E].var_F).wrapping_sub(1));
        if (slot!(game, Enemy_Shitroid[E].var_F) & 0x8000) != 0 {
            player_mother_brain(2, game);
            slot!(game, Enemy_Shitroid[E].parameter_2 = 1);
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9f3bei32) as u16);
            Shitroid_Func_25(k, game);
        } else {
            Shitroid_Func_27(k, game);
        };
    };
    @countdown [pub fn (k: u16, game: &mut Game)];
    before {
        let E: i32 = k as i32;
        slot!(game, Enemy_Shitroid[E].var_F = slot!(game, Enemy_Shitroid[E].var_F).wrapping_sub(1));
    }
    when (slot!(game, Enemy_Shitroid[E].var_F) & 0x8000) != 0;
    [11137079] Shitroid_Func_6 {
        QueueMusic_Delayed8(5, game);
        slot!(game, Enemy_Shitroid[E].var_A = (0xa9f049i32) as u16);
        Shitroid_Func_7(k, game);
    };
    [11137710] Shitroid_Func_18 {
        slot!(game, Enemy_Shitroid[E].var_A = (0xa9f2c0i32) as u16);
        slot!(game, Enemy_Shitroid[E].var_F = 192);
        Shitroid_Func_19(k, game);
    };
    @external {
        11136998 => [actors::shitroid::wait_for_camera(game)];
        11137787 => [Shitroid_Func_20(game)]; 11137828 => [Shitroid_Func_21(game)];
        11137901 => [Shitroid_Func_23(game)];
    }
}
pub fn Shitroid_Func_20(game: &mut Game) {
    {
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            word!(game.cur_enemy_index),
            0,
            (word!(game.samus_x_pos)).wrapping_sub(64),
            100,
            game,
        );
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(
            game,
            Enemy_Shitroid[E].var_F = slot!(game, Enemy_Shitroid[E].var_F).wrapping_sub(1)
        );
        if (slot!(game, Enemy_Shitroid[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9f324i32) as u16);
            slot!(game, Enemy_Shitroid[E].var_F = 88);
            Shitroid_Func_21(game);
        }
    }
}
pub fn Shitroid_Func_21(game: &mut Game) {
    {
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            word!(game.cur_enemy_index),
            0,
            (word!(game.samus_x_pos)).wrapping_add(96),
            104,
            game,
        );
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(
            game,
            Enemy_Shitroid[E].var_F = slot!(game, Enemy_Shitroid[E].var_F).wrapping_sub(1)
        );
        if (slot!(game, Enemy_Shitroid[E].var_F) & 0x8000) != 0 {
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9f3a3i32) as u16);
            slot!(game, Enemy_Shitroid[E].var_F = 256);
            slot! { game, Enemy_Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F93A as u16, instruction_timer = 1 }
            slot!(game, Enemy_Shitroid[E].base.timer = 0);
        }
    }
}
pub fn Shitroid_Func_23(game: &mut Game) {
    {
        let rect: Rect16U = Rect16U {
            x: -128i32 as u16,
            y: 64,
            w: 8,
            h: 8,
        };
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            word!(game.cur_enemy_index),
            0,
            rect.x,
            rect.y,
            game,
        );
        if actors::shitroid::separated_from_rect(word!(game.cur_enemy_index), rect, game) == 0 {
            let E: i32 = word!(game.cur_enemy_index) as i32;
            slot! { game, Enemy_Shitroid[E]; var_B = 0, var_C = 0 }
            slot!(
                game,
                Enemy_Shitroid[E].base.properties = (slot!(game, Enemy_Shitroid[E].base.properties)
                    as i32
                    & !(kEnemyProps_DisableSamusColl as i32 | kEnemyProps_Invisible as i32))
                    as u16
            );
            slot!(game, Enemy_Shitroid[E].var_A = (0xa9efdfi32) as u16);
        }
    }
}
pub fn Shitroid_F3C4(k: u16, game: &mut Game) {
    slot!(game, Enemy_Shitroid[k as i32].var_A = (0xa9f360i32) as u16);
}
pub fn Shitroid_Func_26(k: u16, game: &mut Game) {
    {
        if slot!(game, Enemy_Shitroid[k as i32].var_A) as i32 == 0xa9f3be & 0xffff {
            Shitroid_F3C4(k, game);
        }
    }
}
pub fn Shitroid_Func_27(k: u16, game: &mut Game) -> u8 {
    {
        let E: i32 = k as i32;
        let v2: u16 = abs16(
            (slot!(game, Enemy_Shitroid[E].base.x_pos)).wrapping_sub(word!(game.samus_x_pos)),
        );
        let mut shitro_var_02: u16 = 0;
        if v2 >= 2 {
            shitro_var_02 = slot!(game, Enemy_Shitroid[E].var_02);
            if shitro_var_02 != 0 {
                shitro_var_02 = shitro_var_02.wrapping_sub(1);
                if (shitro_var_02 & 0x8000) != 0 {
                    shitro_var_02 = 0;
                }
            }
        } else {
            shitro_var_02 =
                ((v2 >= 2) as i32 + slot!(game, Enemy_Shitroid[E].var_02) as i32 + 2) as u16;
        }
        slot!(game, Enemy_Shitroid[E].var_02 = shitro_var_02);
        let shitro_var_01: u16 = slot!(game, Enemy_Shitroid[E].var_01);
        let mut v5: u16 = 0;
        if shitro_var_01 != 0 {
            slot!(
                game,
                Enemy_Shitroid[E].var_01 = (shitro_var_01).wrapping_sub(1)
            );
            v5 = (word!(game.samus_y_pos)).wrapping_sub(18);
        } else {
            v5 = 80;
            if (word!(game.random_number) & 0xfff) >= 0xfe0 {
                slot!(game, Enemy_Shitroid[E].var_01 = 32);
            }
        }
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            word!(game.cur_enemy_index),
            8,
            word!(game.samus_x_pos),
            v5,
            game,
        );
        (slot!(
            game,
            Enemy_Shitroid[word!(game.cur_enemy_index) as i32].var_02
        ) >= 0x400
            || word!(game.samus_x_pos) as i32 - 128i32 & 0x8000 != 0) as u8
    }
}
pub fn Shitroid_GraduallyAccelerateTowardsPt(
    k: u16,
    j: u16,
    r18: u16,
    r20: u16,
    r26: u16,
    game: &mut Game,
) {
    {
        crate::verify_probe::hit("homing.metroid");
        let divisor = g_byte_A9F56A[j as usize];
        Shitroid_GraduallyAccelerateHoriz(k, r18, divisor as u16, r26, game);
        let E: i32 = k as i32;
        let distance = (slot!(game, Enemy_Shitroid[E].base.y_pos) as i16).wrapping_sub(r20 as i16);
        if distance != 0 {
            slot!(
                game,
                Enemy_Shitroid[E].var_C = homing_velocity(
                    distance,
                    slot!(game, Enemy_Shitroid[E].var_C),
                    divisor as u8,
                    HomingReversal::ExtraForce(8),
                    1280,
                )
            );
        }
    }
}
pub fn Shitroid_GraduallyAccelerateHoriz(k: u16, r18: u16, r24: u16, r26: u16, game: &mut Game) {
    {
        let E: i32 = k as i32;
        let distance = (slot!(game, Enemy_Shitroid[E].base.x_pos) as i16).wrapping_sub(r18 as i16);
        if distance != 0 {
            // Capture velocity before the screen check, as in both original branches.
            let velocity = slot!(game, Enemy_Shitroid[E].var_B);
            let reversing = (velocity as i16 >= 0) == (distance > 0);
            let force = if reversing && actors::shitroid::is_off_screen(k, game) & 1 != 0 {
                r26.wrapping_add(8)
            } else {
                8
            };
            slot!(
                game,
                Enemy_Shitroid[E].var_B = homing_velocity(
                    distance,
                    velocity,
                    r24 as u8,
                    HomingReversal::ExtraForce(force),
                    2048,
                )
            );
        }
    }
}
pub fn Shitroid_AccelerateTowardsPoint(k: u16, a: u16, r18: u16, r20: u16, game: &mut Game) -> u8 {
    {
        let mut R28: u16 = 0;
        R28 = (R28).wrapping_add(actors::shitroid::accelerate_towards_x(k, r18, a, game));
        R28 = (R28).wrapping_add(actors::shitroid::accelerate_towards_y(k, r20, a, game));
        R28 = R28 >> 1;
        ((R28) as u8) & (1u8)
    }
}
pub fn Shitroid_HandleCutscenePalette_Common(r22: u16, r18: u16, game: &mut Game) {
    {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        let mut shitro_var_D_high: i8 = slot!(byte game, Enemy_Shitroid[E].var_D[1]) as i8;
        if shitro_var_D_high != 0 {
            slot!(byte game, Enemy_Shitroid[E].var_D[1] = ((shitro_var_D_high) as u8).wrapping_sub(1));
        } else {
            slot!(byte game, Enemy_Shitroid[E].var_D[1] = slot!(game, Enemy_Shitroid[E].var_E) as u8);
            let v3: u8 = (slot!(byte game, Enemy_Shitroid[E].var_D[0]) as i32 + 1 & 7) as u8;
            slot!(byte game, Enemy_Shitroid[E].var_D[0] = v3);
            let v4: u16 =
                actors::shitroid::handle_cry(word!(game.cur_enemy_index), v3 as u16, game);
            copy_rom_colors_to_palette(r18, 0xa9, (r22 as i32 + 8 * v4 as i32) as u16, 4, game);
        };
    }
}
crate::instruction_callbacks! { pub fn (_k: u16, _jp: EnemyScript) -> EnemyScript;
    Shitroid_Instr_3 [EnemyScript::jump(addr_kShitroid_Ilist_F90E as u16)] { };
    Shitroid_Instr_4 [EnemyScript::jump(addr_kShitroid_Ilist_F924 as u16)] { };
    Shitroid_Instr_6 [EnemyScript::jump(addr_kShitroid_Ilist_F93A as u16)] { };
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub fn (game: &mut Game)];
    [11110023] MotherBrainsBody_Init [] {
        word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(824));
        MotherBrain_SetBodyInstrs(addr_kMotherBrain_Ilist_9C13 as u16, game);
        let E: i32 = 0;
        slot!(game, MotherBrainBody[E].base.vram_tiles_index = 0);
        slot!(game, MotherBrainBody[E].base.properties = (slot!(game, MotherBrainBody[E].base.properties) as i32
            | (kEnemyProps_BlockPlasmaBeam as i32
                | kEnemyProps_Tangible as i32
                | kEnemyProps_Invisible as i32)) as u16);
        slot!(game, MotherBrainBody[E].base.palette_index = 0);
        copy_rom_colors_to_target_palette(
            0xa9,
            0x162,
            (addr_kMotherBrainPalette_4 as i32 + 2) as u16,
            0xf,
            game,
        );
        copy_rom_colors_to_target_palette(
            0xa9,
            0x1e2,
            (addr_kMotherBrainPalette_3 as i32 + 2) as u16,
            0xf,
            game,
        );
        slot!(game, MotherBrainBody[E].form = 0);
        let E1: i32 = 0x40;
        slot!(game, MotherBrainBrain[E1].unpause_hook = 0);
        slot!(game, MotherBrainBody[E].hitboxes = 2);
        slot!(game, MotherBrainBrain[E1].function = (0xa987d0i32) as u16);
        slot!(game, MotherBrainBody[E].function = (0xa987e1i32) as u16);
        LoadFxEntry(1, game);
        for v3 in 0u16..12 {
            SpawnEprojWithRoomGfx(addr_kEproj_MotherBrainRoomTurrets as u16, v3, game);
        }
    };
    [11110149] MotherBrainsBrain_Init [] {
        corpses::InitializeEnemyCorpseRotting(0x40, addr_stru_A9DE08 as u16, game);
        let E1: i32 = 0x40;
        slot!(game, MotherBrainBrain[E1].base.health = 3000);
        MotherBrain_SetBrainUnusedInstrs(addr_kMotherBrain_Ilist_9C13 as u16, game);
        slot! { game, MotherBrainBrain[E1].base; vram_tiles_index = 0, properties = (slot!(game, MotherBrainBrain[E1].base.properties)) | (0x1100u16), palette_index = 512 };
        let E: i32 = 0;
        slot! { game, MotherBrainBody[E]; neck_palette = 512, brain_palette = 512 }
        MotherBrain_SetBrainInstrs(addr_stru_A99C21 as u16, game);
        MotherBrain_SetupBrainNormalPal(game);
    };
    [11110206] MotherBrainsBody_Hurt [] {
        MotherBrain_Pal_HandleRoomPal(game);
        let E: i32 = 0;
        if slot!(game, MotherBrainBody[E].function) == (fnMotherBrainsBody_FirstPhase_DoubleRet as u16) {
            MotherBrainsBody_FirstPhase_DoubleRet(game);
            return;
        }
        CallMotherBrainFunc(((slot!(game, MotherBrainBody[E].function)) as u32) | (0xa90000u32), game);
        MotherBrain_HandlePalette(game);
        MotherBrain_SamusCollDetect(game);
        MotherBrain_Pal_ProcessInvincibility(game);
        if slot!(game, MotherBrainBrain[0x40i32].unpause_hook) != 0 {
            { let value: u32 = fnMotherBrainsBody_UnpauseHook as u32; { let value: u16 = value as u16; let at: usize = 0x604; let bytes = value.to_ne_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }; game.ram().write_byte(0x604 + 2, (value >> 16) as u8); };
        }
    };
    [11110279] MotherBrainsBody_Powerbomb [] {
        actors::combat::power_bomb_reaction(&mut Runtime::new(game), actors::api::DeathHandling::LeavePending);
        MotherBrainsBrain_Hurt(game);
    };
    [11110283] MotherBrainsBrain_Hurt [] {
        { let value: u32 = 0xa98786; { let value: u16 = value as u16; let at: usize = 0x178c; let bytes = value.to_ne_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }; game.ram().write_byte(0x178c + 2, (value >> 16) as u8); };
        let v0: i32 = 0;
        if (slot!(game, EnemyData[v0 + core::mem::size_of::<EnemyData>() as i32].properties) & 0x100) != 0 {
            CallMotherBrainFunc(((slot!(game, EnemyData[v0 + core::mem::size_of::<EnemyData>() as i32].ai_var_A)) as u32) | (0xa90000u32), game);
        }
    };
    [11111221] MotherBrainsTubesFalling_Init [] { sm_game::actors::mother_brain_tubes::initialize(game); };
    [11121923] MotherBrainsBody_Shot [] {
        actors::combat::create_dud(&mut Runtime::new(game));
    };
    [11122118] MotherBrainsBrain_Touch [] {
        if game.ram().read_byte(0xa1f) == 3 {
            let mut v0: i16 = 13;
            let v1: i32 = 0;
            let flash_timer: u16 = slot!(game, EnemyData[v1 + core::mem::size_of::<EnemyData>() as i32].flash_timer);
            if flash_timer != 0 && flash_timer & 1 != 0 {
                v0 = 14;
            }
            slot!(game, EnemyData[v1 + core::mem::size_of::<EnemyData>() as i32].flash_timer = v0 as u16);
        }
    };
    [11126544] ShitroidInCutscene_Init [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_ShitroidInCutscene[E].base.properties = (slot!(game, Enemy_ShitroidInCutscene[E].base.properties) as i32
            | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_BlockPlasmaBeam as i32))
            as u16);
        slot!(game, Enemy_ShitroidInCutscene[E].base.palette_index = 3584);
        slot! { game, Enemy_ShitroidInCutscene[E].base; current_instruction = addr_kShitroid_Ilist_CFA2 as u16, instruction_timer = 1 }
        slot!(game, Enemy_ShitroidInCutscene[E].var_04 = 1);
        slot!(game, Enemy_ShitroidInCutscene[E].base.timer = 0);
        slot!(game, Enemy_ShitroidInCutscene[E].var_E = 10);
        slot! { game, Enemy_ShitroidInCutscene[E].base; vram_tiles_index = 160, x_pos = 320, y_pos = 96 };
        slot! { game, Enemy_ShitroidInCutscene[E]; var_B = 0, var_C = 0, var_09 = 0 }
        slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9c7cci32) as u16);
        slot!(game, Enemy_ShitroidInCutscene[E].var_F = 248);
        slot!(game, Enemy_ShitroidInCutscene[E].var_0F = (0xa9f683i32) as u16);
        copy_rom_colors_to_palette(
            0x1e2,
            0xa9,
            (addr_kMotherBrainPalette_2 as i32 + 2) as u16,
            0xf,
            game,
        );
    };
    [11126649] ShitroidInCutscene_Main [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_ShitroidInCutscene[E].base.shake_timer = 0);
        CallShitroidCutsceneFuncA(
            ((slot!(game, Enemy_ShitroidInCutscene[E].var_A)) as u32) | (0xa90000u32),
            word!(game.cur_enemy_index),
            game,
        );
        MoveEnemyWithVelocity(game);
        ShitroidInCutscene_Flashing(word!(game.cur_enemy_index), game);
        ShitroidInCutscene_HandleHealthBasedPalette(word!(game.cur_enemy_index), game);
        let r18: u16 = slot!(game, Enemy_ShitroidInCutscene[word!(game.cur_enemy_index) as i32].var_0F);
        CallShitroidCutsceneFunc0F(((r18) as u32) | (0xa90000u32), game);
    };
    [11128579] ShitroidInCutscene_Touch [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        if slot!(game, Enemy_ShitroidInCutscene[E].var_A) as i32 == 0xa9ca66 & 0xffff
            && Shitroid_AccelerateTowardsPoint(
                word!(game.cur_enemy_index),
                0x10,
                word!(game.samus_x_pos),
                (word!(game.samus_y_pos)).wrapping_sub(20),
                game,
            ) & 1
                != 0
        {
            slot! { game, Enemy_ShitroidInCutscene[E]; var_B = 0, var_C = 0 }
            slot!(game, Enemy_ShitroidInCutscene[E].var_A = (0xa9ca7ai32) as u16);
        }
    };
    [11136823] Shitroid_Init [] {
        word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(0));
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_Shitroid[E].base.properties = (slot!(game, Enemy_Shitroid[E].base.properties) as i32
            | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_BlockPlasmaBeam as i32))
            as u16);
        slot!(game, Enemy_Shitroid[E].base.palette_index = 1024);
        slot! { game, Enemy_Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F90E as u16, instruction_timer = 1 }
        slot!(game, Enemy_Shitroid[E].base.timer = 0);
        let mut v2: u16 = (0xa9efe6i32) as u16;
        if (word!(game.layer1_x_pos) & 0x8000) != 0 {
            slot!(game, Enemy_Shitroid[E].base.properties = (slot!(game, Enemy_Shitroid[E].base.properties) as i32
                | (kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
                as u16);
            v2 = (0xa9efdfi32) as u16;
        }
        slot!(game, Enemy_Shitroid[E].var_A = v2);
        slot! { game, Enemy_Shitroid[E]; var_B = 0, var_C = 0 }
        slot!(game, Enemy_Shitroid[E].var_E = 10);
        slot!(game, Enemy_Shitroid[E].parameter_2 = 0);
        copy_rom_colors_to_target_palette(0xa9, 0x120, addr_word_A9F8C6 as u16, 0x10, game);
        copy_rom_colors_to_target_palette(
            0xa9,
            0x140,
            addr_kShitroidInCutscene_Palette as u16,
            0x10,
            game,
        );
        copy_rom_colors_to_target_palette(
            0xa9,
            0x1e0,
            addr_kDeadSidehopper_Palette_0 as u16,
            0x10,
            game,
        );
    };
    [11136954] Shitroid_Powerbomb [] {
        if slot!(game, Enemy_Shitroid[word!(game.cur_enemy_index) as i32].parameter_2) != 0 {
            Shitroid_Func_26(word!(game.cur_enemy_index), game);
        }
        Shitroid_Main(game);
    };
    [11136965] Shitroid_Main [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Enemy_Shitroid[E].base.health = 0x7fff);
        CallShitroidFunc(
            ((slot!(game, Enemy_Shitroid[E].var_A)) as u32) | (0xa90000u32),
            word!(game.cur_enemy_index),
            game,
        );
        MoveEnemyWithVelocity(game);
        if word!(game.palette_change_num) == 0 {
            Shitroid_HandleNormalPalette(game);
        }
    };
    [11138953] Shitroid_Touch [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        if slot!(game, Enemy_Shitroid[E].parameter_2) != 0 {
            Shitroid_Func_26(word!(game.cur_enemy_index), game);
            if game.ram().read_byte(0xa1f) == 3 && word!(game.samus_x_pos) as i32 - 512i32 & 0x8000 == 0
            {
                let r18: u16 = (0x80
                    - actor_api::angle_from_delta((word!(game.samus_x_pos)).wrapping_sub(slot!(game, Enemy_Shitroid[E].base.x_pos)), (word!(game.samus_y_pos)).wrapping_sub(slot!(game, Enemy_Shitroid[E].base.y_pos))) as i32
                    + 0x80) as u8 as u16;
                slot!(game, Enemy_Shitroid[E].var_B = (slot!(game, Enemy_Shitroid[E].var_B)).wrapping_add(Math_MultBySin(0x40, r18)));
                slot!(game, Enemy_Shitroid[E].var_C = (slot!(game, Enemy_Shitroid[E].var_C)).wrapping_add(Math_MultByCos(0x40, r18)));
            } else if slot!(game, Enemy_Shitroid[E].var_A) as i32 == 0xa9f1fa & 0xffff {
                if Shitroid_AccelerateTowardsPoint(
                    word!(game.cur_enemy_index),
                    0x200,
                    word!(game.samus_x_pos),
                    (word!(game.samus_y_pos)).wrapping_sub(32),
                    game,
                ) & 1
                    != 0
                {
                    slot! { game, Enemy_Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F924 as u16, instruction_timer = 1 }
                    slot!(game, Enemy_Shitroid[E].base.timer = 0);
                    slot!(game, Enemy_Shitroid[E].var_E = 1);
                    slot! { game, Enemy_Shitroid[E]; parameter_2 = 0, var_B = 0, var_C = 0 }
                    slot!(game, Enemy_Shitroid[E].var_A = (0xa9f20ei32) as u16);
                }
            } else if slot!(game, Enemy_Shitroid[E].var_A) as i32 == 0xa9f180 & 0xffff {
                slot!(game, Enemy_Shitroid[E].var_A = (0xa9f1fai32) as u16);
            }
        }
    };
    [11139138] Shitroid_Shot [] {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        if slot!(game, Enemy_Shitroid[E].parameter_2) != 0 {
            Shitroid_Func_26(word!(game.cur_enemy_index), game);
            let r18: u16 = -(actor_api::angle_from_delta((word!(game.projectile_x_pos[0])).wrapping_sub(slot!(game, Enemy_Shitroid[E].base.x_pos)), (word!(game.projectile_y_pos[0])).wrapping_sub(slot!(game, Enemy_Shitroid[E].base.y_pos))) as i32) as u8 as u16;
            let mut v1: u16 = (8u16).wrapping_mul(word!(
                game.projectile_damage[word!(game.collision_detection_index)]
            ));
            if v1 >= 0xf0 {
                v1 = 240;
            }
            slot!(game, Enemy_Shitroid[E].var_B = (slot!(game, Enemy_Shitroid[E].var_B)).wrapping_add(Math_MultBySin(v1, r18)));
            slot!(game, Enemy_Shitroid[E].var_C = (slot!(game, Enemy_Shitroid[E].var_C)).wrapping_add(Math_MultByCos(v1, r18)));
        }
    };
    @forward [pub fn (game: &mut Game)];
    [11108367] Enemy_GrappleReact_CancelBeam_A9 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [11108417] Enemy_NormalFrozenAI_A9 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
    @external {
        11129608 => corpses::DeadTorizo_Init;
        11129704 => corpses::DeadTorizo_Main;
        11129898 => corpses::DeadTorizo_Powerbomb;
        11129907 => corpses::DeadTorizo_Shot;
        11130806 => corpses::DeadSidehopper_Init;
        11130953 => corpses::DeadZoomer_Init;
        11130998 => corpses::DeadRipper_Init;
        11131039 => corpses::DeadSkree_Init;
        11131084 => corpses::DeadSidehopper_Powerbomb;
        11131099 => corpses::DeadSidehopper_Main;
        11132141 => corpses::DeadZoomer_Powerbomb;
        11132152 => corpses::DeadZoomer_Shot;
        11132157 => corpses::DeadRipper_Powerbomb;
        11132168 => corpses::DeadRipper_Shot;
        11132173 => corpses::DeadSkree_Powerbomb;
        11132184 => corpses::DeadSkree_Shot;
        11132189 => corpses::DeadSidehopper_Shot;
        11132228 => corpses::DeadSidehopper_Touch;
    }
}

crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyInstr(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { match crate::game::mother_brain_body::dispatch_instruction(ea, j, game) { Some(script) => script, None => { crate::rtl::Unreachable(game); EnemyScript::Stop } } };
    @functions [pub fn (k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11139476] Shitroid_Instr_5 [] {
        if (word!(game.random_number) & 0x8000) == 0 {
            return EnemyScript::jump(jp.word(0));
        }
        QueueSfx2_Max6(0x52, game);
        jp.offset(1)
    };
    @functions [pub fn (_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript];
    [11115208] MotherBrain_Instr_SpawnEprojToOffset [jp.offset(3)] {
        let E: i32 = 0;
        game.eproj_spawn_pt = Point16U {
            x: ((slot!(game, MotherBrainBody[E].base.x_pos)).wrapping_add(jp.word(0))),
            y: ((slot!(game, MotherBrainBody[E].base.y_pos)).wrapping_add(jp.word(2))),
        };
        SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, jp.word(4), game);
    };
    [11115247] MotherBrain_Instr_SpawnDeathBeamEproj [jp] {
        let spawn_parameter = crate::game::music::queue_sfx(1, 0x63, 6, game);
        SpawnEprojWithGfx(
            spawn_parameter,
            0x40,
            addr_kEproj_MotherBrainDeathBeamCharging as u16,
            game,
        );
    };
    [11115269] MotherBrain_Instr_IncrBeamAttackPhase [jp] {
        let E: i32 = 0;
        slot!(game, MotherBrainBody[E].hand_beam_phase = slot!(game, MotherBrainBody[E].hand_beam_phase).wrapping_add(1));
    };
    @external {
        11136208 => corpses::sub_A9ECD0;
    }
}

impl corpses::Host for Game {
    fn dead_torizo_contact(&mut self) -> bool {
        actors::part_hitboxes::dead_torizo_contact(self)
    }
}

impl sm_game::actors::mother_brain_fight::Host for Game {
    fn bank_unreachable(&mut self) -> bool {
        crate::rtl::Unreachable(self)
    }

    fn spawn_legacy_population(&mut self, bank: u8, population: u16) -> u16 {
        crate::game::actors::population::spawn_legacy(
            &mut crate::game::session_runtime::Runtime::new(self),
            bank,
            population,
        )
    }

    fn set_projectile_spawn_r22(&mut self, value: u16) {
        self.eproj_spawn_r22 = value;
    }

    fn create_dud(&mut self) {
        crate::game::actors::combat::create_dud(&mut crate::game::session_runtime::Runtime::new(
            self,
        ));
    }

    fn fade(&mut self, kind: sm_game::actors::mother_brain_fight::Fade, frame: u16) -> u8 {
        use crate::game::mother_brain_visuals::Fade as Visual;
        use sm_game::actors::mother_brain_fight::Fade;
        let kind = match kind {
            Fade::ToBlack => Visual::ToBlack,
            Fade::FakeDeathToGray => Visual::FakeDeathToGray,
            Fade::FakeDeathFromGray => Visual::FakeDeathFromGray,
            Fade::DrainedFromGray => Visual::DrainedFromGray,
            Fade::DrainedToGray => Visual::DrainedToGray,
            Fade::RealDeathToGray => Visual::RealDeathToGray,
            Fade::LightsOn => Visual::LightsOn,
        };
        unsafe { crate::game::mother_brain_visuals::fade(kind, frame, self) }
    }

    fn update_health_palette(&mut self) {
        unsafe { crate::game::mother_brain_visuals::update_health_palette(self) };
    }

    fn seal_wall(&mut self) {
        unsafe { crate::game::mother_brain_visuals::seal_wall(self) };
    }

    fn calculate_rainbow_window(&mut self) {
        unsafe { crate::game::beam_windows::calculate_rainbow_window(self) };
    }

    fn flicker_body(&mut self) {
        crate::game::mother_brain_visuals::flicker_body(self);
    }

    fn drain_health_by_rainbow_beam(&mut self) {
        crate::game::mother_brain_samus_vitals::drain_health_by_rainbow_beam(self);
    }

    fn drain_ammo_by_rainbow_beam(&mut self) {
        crate::game::mother_brain_samus_vitals::drain_ammo_by_rainbow_beam(self);
    }

    fn add_glass_hits(&mut self, slot: usize, amount: u16) {
        crate::game::room_mechanisms::add_glass_hits(slot, amount, self);
    }

    fn spawn_palette_effect(&mut self, definition: u16) {
        crate::game::palette_effects::spawn(definition, self);
    }

    fn queue_mother_brain_sprite_tiles(&mut self, bank: u8, start: u16) -> u8 {
        crate::game::rom_vram_entries::queue_mother_brain_sprite_tiles(bank, start, self)
    }

    fn process_corpse_rotting(&mut self, k: u16) -> u8 {
        corpses::ProcessCorpseRotting(k, self)
    }

    fn process_corpse_rotting_vram_transfers(&mut self, k: u16) {
        corpses::ProcessCorpseRottingVramTransfers(k, self);
    }

    fn start_zebes_escape_typewriter(&mut self) {
        crate::game::escape_typewriter::start_zebes_escape_typewriter(self);
    }

    fn type_escape_text(&mut self, tiles: u16) -> u8 {
        crate::game::escape_typewriter::type_escape_text(tiles, self)
    }

    fn set_boss_bit_for_current_area(&mut self, mask: u16) {
        crate::game::gameplay_flag_runtime::SetBossBitForCurArea(mask, self);
    }

    fn set_event_happened(&mut self, event: u16) {
        crate::game::gameplay_flag_runtime::SetEventHappened(event, self);
    }

    fn check_event_happened(&mut self, event: u16) -> u16 {
        crate::game::gameplay_flag_runtime::CheckEventHappened(event, self)
    }

    fn spawn_rainbow_hdma(&mut self) -> u16 {
        crate::game::mother_brain_visuals::spawn_rainbow_hdma(self)
    }

    fn spawn_rising_hdma(&mut self) -> u16 {
        crate::game::mother_brain_visuals::spawn_rising_hdma(self)
    }

    fn player_mother_brain(&mut self, command: u16) {
        crate::game::session_runtime::player_mother_brain(command, self);
    }

    fn load_fx_entry(&mut self, index: u16) {
        crate::game::room_state::load_fx_entry(index, self);
    }

    fn native_word(&mut self, offset: usize) -> u16 {
        crate::game::display_refresh::native_word(offset, self)
    }

    fn set_native_word(&mut self, offset: usize, value: u16) {
        crate::game::display_refresh::set_native_word(offset, value, self);
    }

    fn samus_contact(
        &mut self,
        boxes: u16,
        x: u16,
        y: u16,
    ) -> sm_game::actors::mother_brain_fight::Contact {
        use crate::game::actors::part_hitboxes::Contact as Part;
        use sm_game::actors::mother_brain_fight::Contact;
        match crate::game::actors::part_hitboxes::samus_contact(boxes, x, y, self) {
            Part::None => Contact::None,
            Part::Pushed => Contact::Pushed,
            Part::Hurt => Contact::Hurt,
        }
    }

    fn ridley_func_98(&mut self) {
        crate::game::sm_a6::Ridley_Func_98(self);
    }

    fn move_samus_horizontally_towards_wall(&mut self, amount: u16) -> u8 {
        crate::game::sm_a9::move_samus_horizontally_towards_wall(amount, self)
    }

    fn move_samus_vertically_towards_ceiling_floor(&mut self, amount: u16) -> u8 {
        crate::game::sm_a9::move_samus_vertically_towards_ceiling_floor(amount, self)
    }
}
