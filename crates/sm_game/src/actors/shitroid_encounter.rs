//! The Shitroid's encounter in Tourian: it drains the sidehopper, latches onto
//! Samus and drains her until she is nearly dead, then lets go and leaves.
//!
//! Was `Shitroid_*` in the original decompilation (bank $A9:EF37-$A9:F8A5).
//! Slot words are named through `shitroid`'s view after the bank log's
//! Shitroid notes. The bank's dispatch ids are unchanged; the functions and
//! their reads and writes keep the original order. Mother Brain's shared
//! helpers come from `mother_brain_fight`.
#![forbid(unsafe_code)]
#![allow(non_snake_case, non_upper_case_globals, unused_parens)]
#![allow(unused_mut, unused_assignments, clippy::all)]
use crate::actors::corpses::Corpse;
use crate::actors::host::abs16;
use crate::actors::mother_brain_fight::{
    CallSomeSamusCode, Enemy_SetInstrList, MoveEnemyWithVelocity, QueueMusic_Delayed8,
    QueueSfx2_Max6, SpawnHardcodedPlmArgs, Unreachable, VoidP, g_word_A993BB, g_word_A993C3,
    player_mother_brain, spawn_hardcoded,
};
use crate::actors::ridley::{HomingReversal, homing_velocity};
use crate::actors::shitroid::Shitroid;
use crate::constants::bank_a9::*;
use crate::constants::enemy_properties::*;
use crate::enemy_projectiles::shared_pre::{Math_MultByCos, Math_MultBySin};
use crate::enemy_script::EnemyScript;
use crate::palette_copies::{copy_rom_colors_to_palette, copy_rom_colors_to_target_palette};
use crate::ram::RawRam as _;
use crate::ram::{slot, word};
use crate::word_transfers;

/// What the Shitroid reaches beyond Mother Brain's fight and its own helpers.
/// The bank $A9 functions these slots run, by their dispatch names.
pub mod function {
    /// $A9:EFDF: Shitroid function - disappeared.
    pub const DISAPPEARED: u16 = 0xefdf;
    /// $A9:EFE6: Shitroid function - wait for camera.
    pub const WAIT_FOR_CAMERA: u16 = 0xefe6;
    /// $A9:F037: Shitroid function - queue battle music.
    pub const QUEUE_BATTLE_MUSIC: u16 = 0xf037;
    /// $A9:F049: Shitroid function - rush to middle of room.
    pub const RUSH_TO_MIDDLE_OF_ROOM: u16 = 0xf049;
    /// $A9:F06D: Shitroid function - rush to sidehopper.
    pub const RUSH_TO_SIDEHOPPER: u16 = 0xf06d;
    /// $A9:F094: Shitroid function - latch onto sidehopper.
    pub const LATCH_ONTO_SIDEHOPPER: u16 = 0xf094;
    /// $A9:F0E6: Shitroid function - draining sidehopper.
    pub const DRAINING_SIDEHOPPER: u16 = 0xf0e6;
    /// $A9:F125: Shitroid function - make sidehopper rottable.
    pub const MAKE_SIDEHOPPER_ROTTABLE: u16 = 0xf125;
    /// $A9:F138: Shitroid function - move up and unlock camera.
    pub const MOVE_UP_AND_UNLOCK_CAMERA: u16 = 0xf138;
    /// $A9:F180: Shitroid function - stare down Samus.
    pub const STARE_DOWN_SAMUS: u16 = 0xf180;
    /// $A9:F1FA: Shitroid function - latch onto Samus.
    pub const LATCH_ONTO_SAMUS: u16 = 0xf1fa;
    /// $A9:F20E: Shitroid function - start draining Samus.
    pub const START_DRAINING_SAMUS: u16 = 0xf20e;
    /// $A9:F21B: Shitroid function - draining Samus.
    pub const DRAINING_SAMUS: u16 = 0xf21b;
    /// $A9:F2A2: Shitroid function - start heel realisation.
    pub const START_HEEL_REALISATION: u16 = 0xf2a2;
    /// $A9:F2AE: Shitroid function - heel realisation.
    pub const HEEL_REALISATION: u16 = 0xf2ae;
    /// $A9:F2C0: Shitroid function - back off guiltily.
    pub const BACK_OFF_GUILTILY: u16 = 0xf2c0;
    /// $A9:F2FB: Shitroid function - go left guiltily.
    pub const GO_LEFT_GUILTILY: u16 = 0xf2fb;
    /// $A9:F324: Shitroid function - go right guiltily.
    pub const GO_RIGHT_GUILTILY: u16 = 0xf324;
    /// $A9:F360: Shitroid function - flee remorsefully.
    pub const FLEE_REMORSEFULLY: u16 = 0xf360;
    /// $A9:F36D: Shitroid function - fleeing.
    pub const FLEEING: u16 = 0xf36d;
    /// $A9:F3A3: Shitroid function - Samus recovering.
    pub const SAMUS_RECOVERING: u16 = 0xf3a3;
    /// $A9:F3BE: Shitroid function - remorse.
    pub const REMORSE: u16 = 0xf3be;
}

// Each named function is the one its dispatch entry runs.
const _: () = {
    use function as f;
    const fn at(word: u16) -> u32 {
        0xa9_0000 | word as u32
    }
    assert!(at(f::DISAPPEARED) == 11136991);
    assert!(at(f::WAIT_FOR_CAMERA) == 11136998);
    assert!(at(f::QUEUE_BATTLE_MUSIC) == 11137079);
    assert!(at(f::RUSH_TO_MIDDLE_OF_ROOM) == 11137097);
    assert!(at(f::RUSH_TO_SIDEHOPPER) == 11137133);
    assert!(at(f::LATCH_ONTO_SIDEHOPPER) == 11137172);
    assert!(at(f::DRAINING_SIDEHOPPER) == 11137254);
    assert!(at(f::MAKE_SIDEHOPPER_ROTTABLE) == 11137317);
    assert!(at(f::MOVE_UP_AND_UNLOCK_CAMERA) == 11137336);
    assert!(at(f::STARE_DOWN_SAMUS) == 11137408);
    assert!(at(f::LATCH_ONTO_SAMUS) == 11137530);
    assert!(at(f::START_DRAINING_SAMUS) == 11137550);
    assert!(at(f::DRAINING_SAMUS) == 11137563);
    assert!(at(f::START_HEEL_REALISATION) == 11137698);
    assert!(at(f::HEEL_REALISATION) == 11137710);
    assert!(at(f::BACK_OFF_GUILTILY) == 11137728);
    assert!(at(f::GO_LEFT_GUILTILY) == 11137787);
    assert!(at(f::GO_RIGHT_GUILTILY) == 11137828);
    assert!(at(f::FLEE_REMORSEFULLY) == 11137888);
    assert!(at(f::FLEEING) == 11137901);
    assert!(at(f::SAMUS_RECOVERING) == 11137955);
    assert!(at(f::REMORSE) == 11137982);
};

pub trait Host: crate::actors::mother_brain_fight::Host + crate::actors::shitroid::Host {
    /// Drain Samus's health while the Shitroid is latched on. Was
    /// `drain_health_by_baby_metroid`.
    fn drain_health_by_shitroid(&mut self);
    /// Mark a homing step for the verifier's probes.
    fn homing_probe(&mut self);
}

pub static g_byte_A9F56A: [i8; 16] = [
    0x10, 0xf, 0xe, 0xd, 0xc, 0xb, 0xa, 9, 8, 7, 6, 5, 4, 3, 2, 1,
];
crate::dispatch::callback_dispatch! {
    pub fn CallShitroidCutsceneFunc0F(ea: u32, game: &mut impl Host) => ea;
    call (game); default { Unreachable(game); };
    @body checked;
    @forward [pub fn (game: &mut impl Host)];
    Shitroid_HandleNormalPalette => Shitroid_HandleCutscenePalette_Common(addr_word_A9F6D1 as u16, 330, game);
    [11138691]
    Shitroid_HandleCutscenePalette => Shitroid_HandleCutscenePalette_Common(addr_word_A9F6D1 as u16, 490, game);
    [11138703]
    Shitroid_HandleCutscenePalette_LowHealth => Shitroid_HandleCutscenePalette_Common(addr_word_A9F711 as u16, 490, game);
    @external {
        11127484 => [()]; 11127960 => [()];
    }
}
crate::dispatch::callback_dispatch! {
    pub fn CallShitroidMoveFunc(ea: u32, k: u16, j: u16, r18: u16, r20: u16, game: &mut impl Host) => ea;
    call (k, j, r18, r20, game); default { Unreachable(game); };
    @body checked;
    @forward [pub fn ( k: u16, j: u16, r18: u16, r20: u16, game: &mut impl Host, )];
    Shitroid_Func_GraduallyAccelerateTowards0x400 => Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 1024, game);
    Shitroid_GraduallyAccelerateTowards0x4 => Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 4, game);
    [11138143]
    Shitroid_GraduallyAccelerateTowards0x8 => Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 8, game);
    [11138150]
    Shitroid_GraduallyAccelerateTowards0x10 => Shitroid_GraduallyAccelerateTowardsPt(k, j, r18, r20, 16, game);
}
crate::dispatch::callback_dispatch! {
    pub fn CallShitroidFunc(ea: u32, k: u16, game: &mut impl Host) => ea;
    call (k, game); default { Unreachable(game); };
    @body checked;
    @functions [pub fn (k: u16, game: &mut impl Host)];
    [11137097] Shitroid_Func_7 [] {
        let rect = (584, 74, 1, 1);
        Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0xf, rect.0, rect.1, game);
        if crate::actors::shitroid::separated_from_rect(k, rect, game) == 0 {
            slot!(game, Shitroid[k as i32].function = function::RUSH_TO_SIDEHOPPER);
        }
    };
    [11137133] Shitroid_Func_8 [] {
        let E: i32 = k as i32 + 64;
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            k,
            0xf,
            slot!(game, Shitroid[E].base.x_pos),
            (slot!(game, Shitroid[E].base.y_pos)).wrapping_sub(32),
            game,
        );
        if crate::actors::shitroid::separated_from_enemy(k, (k).wrapping_add(64), game) & 1 == 0 {
            slot!(game, Shitroid[k as i32].function = function::LATCH_ONTO_SIDEHOPPER);
        }
    };
    [11137172] Shitroid_Func_9 [] {
        let E1: i32 = k as i32 + 64;
        if Shitroid_AccelerateTowardsPoint(
            k,
            0x200,
            slot!(game, Shitroid[E1].base.x_pos),
            (slot!(game, Shitroid[E1].base.y_pos)).wrapping_sub(32),
            game,
        ) & 1
            != 0
        {
            let E: i32 = k as i32;
            slot! { game, Shitroid[E]; x_velocity = 0, y_velocity = 0 }
            slot!(game, Shitroid[E].base.x_pos = slot!(game, Shitroid[E1].base.x_pos));
            slot!(game, Shitroid[E].base.y_pos = (slot!(game, Shitroid[E1].base.y_pos)).wrapping_sub(32));
            slot! { game, Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F924 as u16, instruction_timer = 1 }
            slot!(game, Shitroid[E].base.timer = 0);
            slot!(game, Shitroid[E].function = function::DRAINING_SIDEHOPPER);
            slot!(game, Shitroid[E].palette_handler_delay = 1);
            slot!(game, Shitroid[E].contact_enabled = 0);
            slot!(game, Shitroid[E].function_timer = 320);
        }
    };
    [11137317] Shitroid_Func_11 [] {
        slot!(game, Corpse[0x40i32].drained_palette_index = 1);
        let E: i32 = k as i32;
        slot!(game, Shitroid[E].function = function::MOVE_UP_AND_UNLOCK_CAMERA);
        slot!(game, Shitroid[E].function_timer = 192);
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
        slot!(game, Shitroid[0i32].function = function::DRAINING_SAMUS);
        Shitroid_Func_16(k, game);
    };
    @functions [pub fn (_k: u16, game: &mut impl Host)];
    [11137728] Shitroid_Func_19 [] {
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            word!(game.cur_enemy_index),
            0,
            word!(game.samus_x_pos),
            104,
            game,
        );
        let E: i32 = word!(game.cur_enemy_index) as i32;
        slot!(game, Shitroid[E].function_timer = slot!(game, Shitroid[E].function_timer).wrapping_sub(1));
        if (slot!(game, Shitroid[E].function_timer) & 0x8000) != 0 {
            QueueSfx2_Max6(0x7d, game);
            slot!(game, Shitroid[E].function = function::GO_LEFT_GUILTILY);
            slot!(game, Shitroid[E].function_timer = 88);
            slot! { game, Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F924 as u16, instruction_timer = 1 }
            slot!(game, Shitroid[E].base.timer = 0);
            Shitroid_Func_20(game);
        }
    };
    @functions [pub fn (k: u16, game: &mut impl Host)];
    [11137888] Shitroid_Func_22 [] {
        QueueSfx2_Max6(0x52, game);
        slot!(game, Shitroid[k as i32].function = function::FLEEING);
        Shitroid_Func_23(game);
    };
    [11137982] Shitroid_Func_25 [] {
        if Shitroid_Func_27(k, game) & 1 != 0 {
            Shitroid_F3C4(k, game);
        }
    };
    @functions [pub fn (k: u16, game: &mut impl Host)];
    @prefix { let E: i32 = k as i32; };
    [11136991] Shitroid_Func_3 [] {
        slot! { game, Shitroid[E]; x_velocity = 0, y_velocity = 0 }
    };
    [11137067] Shitroid_Func_5 [] {
        slot!(game, Shitroid[E].function = function::QUEUE_BATTLE_MUSIC);
        slot!(game, Shitroid[E].function_timer = 464);
        Shitroid_Func_6(k, game);
    };
    [11137254] Shitroid_Func_10 [] {
        let v2: i32 = (slot!(game, Shitroid[E].base.frame_counter) as i32 & 6) >> 1;
        let E1: i32 = k as i32 + 64;
        slot!(game, Shitroid[E].base.x_pos = (slot!(game, Shitroid[E1].base.x_pos)).wrapping_add((g_word_A993BB[v2 as usize]) as u16));
        slot!(game, Shitroid[E].base.y_pos = (slot!(game, Shitroid[E1].base.y_pos) as i32 + g_word_A993C3[v2 as usize] as i32 - 32i32) as u16);
        let fresh13: u16 = slot!(game, Shitroid[E].function_timer);
        slot!(game, Shitroid[E].function_timer = slot!(game, Shitroid[E].function_timer).wrapping_sub(1));
        if fresh13 == 1 {
            slot!(game, Shitroid[E].function = function::MAKE_SIDEHOPPER_ROTTABLE);
            slot! { game, Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F906 as u16, instruction_timer = 1 }
            slot!(game, Shitroid[E].base.timer = 0);
            slot!(game, Shitroid[E].palette_handler_delay = 10);
        }
    };
    [11137336] Shitroid_Func_12 [] {
        Shitroid_Func_GraduallyAccelerateTowards0x400(k, 0, slot!(game, Shitroid[E].base.x_pos), 104, game);
        slot!(game, Shitroid[E].function_timer = slot!(game, Shitroid[E].function_timer).wrapping_sub(1));
        if (slot!(game, Shitroid[E].function_timer) & 0x8000) != 0 {
            slot!(game, Shitroid[E].function = function::STARE_DOWN_SAMUS);
            slot!(game, Shitroid[E].contact_enabled = 1);
            { let at: usize = 0xcd20; let bytes: [u8; 2] = ((u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])) | (0x100u16)).to_le_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }
            { let at: usize = 0xcd20 + (2 ) as usize; let bytes: [u8; 2] = ((u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])) | (0x100u16)).to_le_bytes(); game.ram().write_byte(at, bytes[0]); game.ram().write_byte(at + 1, bytes[1]); }
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x30, 0x3, 0xb763 as VoidP), game);
            spawn_hardcoded(SpawnHardcodedPlmArgs::new(0x1f, 0x3, 0xb763 as VoidP), game);
        }
    };
    [11137408] Shitroid_Func_13 [] {
        let v2: u16 = abs16((slot!(game, Shitroid[E].base.x_pos)).wrapping_sub(word!(game.samus_x_pos)));
        let mut proximity: u16 = 0;
        if v2 >= 8 {
            proximity = slot!(game, Shitroid[E].proximity_counter);
            if proximity != 0 {
                proximity = proximity.wrapping_sub(1);
                if (proximity & 0x8000) != 0 {
                    proximity = 0;
                }
            }
        } else {
            proximity = ((v2 >= 8) as i32 + slot!(game, Shitroid[E].proximity_counter) as i32 + 2) as u16;
        }
        slot!(game, Shitroid[E].proximity_counter = proximity);
        if proximity >= 0x100 || word!(game.samus_x_pos) as i32 - 512i32 & 0x8000 != 0 {
            slot!(game, Shitroid[word!(game.cur_enemy_index) as i32].function = function::LATCH_ONTO_SAMUS);
        } else {
            let aggro_timer: u16 = slot!(game, Shitroid[E].aggro_timer);
            let mut v5: u16 = 0;
            if aggro_timer != 0 {
                slot!(game, Shitroid[E].aggro_timer = (aggro_timer).wrapping_sub(1));
                v5 = word!(game.samus_y_pos);
            } else {
                v5 = 80;
                if (word!(game.random_number) & 0xfff) >= 0xfe0 {
                    slot!(game, Shitroid[E].aggro_timer = 32);
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
            slot!(game, Shitroid[E].function = function::START_HEEL_REALISATION);
            slot! { game, Shitroid[E]; x_velocity = 0, y_velocity = 0 }
            Enemy_SetInstrList(k, addr_kShitroid_Ilist_F906 as u16, game);
            slot!(game, Shitroid[E].palette_handler_delay = 10);
            CallSomeSamusCode(0x13, game);
            player_mother_brain(0, game);
            slot!(game, Shitroid[E].cry_enabled = 0);
            QueueMusic_Delayed8(7, game);
        } else {
            word! {game; cooldown_timer = 8; bomb_counter = 5; samus_x_speed_divisor = 2; }
            if word!(game.samus_y_speed) as i32 - 4i32 & 0x8000 == 0 {
                word!(game.samus_y_speed = 2);
            }
            let v2: i32 = (slot!(game, Shitroid[E].base.frame_counter) as i32 & 6) >> 1;
            slot!(game, Shitroid[E].base.x_pos = (word!(game.samus_x_pos)).wrapping_add((g_word_A993BB[v2 as usize]) as u16));
            slot!(game, Shitroid[E].base.y_pos = (word!(game.samus_y_pos) as i32 + g_word_A993C3[v2 as usize] as i32 - 20i32) as u16);
            game.drain_health_by_shitroid();
        };
    };
    [11137698] Shitroid_Func_17 [] {
        slot!(game, Shitroid[E].function = function::HEEL_REALISATION);
        slot!(game, Shitroid[E].function_timer = 120);
        Shitroid_Func_18(k, game);
    };
    [11137955] Shitroid_Func_24 [] {
        slot!(game, Shitroid[E].function_timer = slot!(game, Shitroid[E].function_timer).wrapping_sub(1));
        if (slot!(game, Shitroid[E].function_timer) & 0x8000) != 0 {
            player_mother_brain(2, game);
            slot!(game, Shitroid[E].contact_enabled = 1);
            slot!(game, Shitroid[E].function = function::REMORSE);
            Shitroid_Func_25(k, game);
        } else {
            Shitroid_Func_27(k, game);
        };
    };
    @countdown [pub fn (k: u16, game: &mut impl Host)];
    before {
        let E: i32 = k as i32;
        slot!(game, Shitroid[E].function_timer = slot!(game, Shitroid[E].function_timer).wrapping_sub(1));
    }
    when (slot!(game, Shitroid[E].function_timer) & 0x8000) != 0;
    [11137079] Shitroid_Func_6 {
        QueueMusic_Delayed8(5, game);
        slot!(game, Shitroid[E].function = function::RUSH_TO_MIDDLE_OF_ROOM);
        Shitroid_Func_7(k, game);
    };
    [11137710] Shitroid_Func_18 {
        slot!(game, Shitroid[E].function = function::BACK_OFF_GUILTILY);
        slot!(game, Shitroid[E].function_timer = 192);
        Shitroid_Func_19(k, game);
    };
    @external {
        11136998 => [crate::actors::shitroid::wait_for_camera(game)];
        11137787 => [Shitroid_Func_20(game)]; 11137828 => [Shitroid_Func_21(game)];
        11137901 => [Shitroid_Func_23(game)];
    }
}
pub fn Shitroid_Func_20(game: &mut impl Host) {
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
            Shitroid[E].function_timer = slot!(game, Shitroid[E].function_timer).wrapping_sub(1)
        );
        if (slot!(game, Shitroid[E].function_timer) & 0x8000) != 0 {
            slot!(game, Shitroid[E].function = function::GO_RIGHT_GUILTILY);
            slot!(game, Shitroid[E].function_timer = 88);
            Shitroid_Func_21(game);
        }
    }
}
pub fn Shitroid_Func_21(game: &mut impl Host) {
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
            Shitroid[E].function_timer = slot!(game, Shitroid[E].function_timer).wrapping_sub(1)
        );
        if (slot!(game, Shitroid[E].function_timer) & 0x8000) != 0 {
            slot!(game, Shitroid[E].function = function::SAMUS_RECOVERING);
            slot!(game, Shitroid[E].function_timer = 256);
            slot! { game, Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F93A as u16, instruction_timer = 1 }
            slot!(game, Shitroid[E].base.timer = 0);
        }
    }
}
pub fn Shitroid_Func_23(game: &mut impl Host) {
    {
        let rect = (-128i32 as u16, 64, 8, 8);
        Shitroid_Func_GraduallyAccelerateTowards0x400(
            word!(game.cur_enemy_index),
            0,
            rect.0,
            rect.1,
            game,
        );
        if crate::actors::shitroid::separated_from_rect(word!(game.cur_enemy_index), rect, game)
            == 0
        {
            let E: i32 = word!(game.cur_enemy_index) as i32;
            slot! { game, Shitroid[E]; x_velocity = 0, y_velocity = 0 }
            slot!(
                game,
                Shitroid[E].base.properties = (slot!(game, Shitroid[E].base.properties) as i32
                    & !(kEnemyProps_DisableSamusColl as i32 | kEnemyProps_Invisible as i32))
                    as u16
            );
            slot!(game, Shitroid[E].function = function::DISAPPEARED);
        }
    }
}
pub fn Shitroid_F3C4(k: u16, game: &mut impl Host) {
    slot!(
        game,
        Shitroid[k as i32].function = function::FLEE_REMORSEFULLY
    );
}
pub fn Shitroid_Func_26(k: u16, game: &mut impl Host) {
    {
        if slot!(game, Shitroid[k as i32].function) == function::REMORSE {
            Shitroid_F3C4(k, game);
        }
    }
}
pub fn Shitroid_Func_27(k: u16, game: &mut impl Host) -> u8 {
    {
        let E: i32 = k as i32;
        let v2: u16 =
            abs16((slot!(game, Shitroid[E].base.x_pos)).wrapping_sub(word!(game.samus_x_pos)));
        let mut proximity: u16 = 0;
        if v2 >= 2 {
            proximity = slot!(game, Shitroid[E].proximity_counter);
            if proximity != 0 {
                proximity = proximity.wrapping_sub(1);
                if (proximity & 0x8000) != 0 {
                    proximity = 0;
                }
            }
        } else {
            proximity =
                ((v2 >= 2) as i32 + slot!(game, Shitroid[E].proximity_counter) as i32 + 2) as u16;
        }
        slot!(game, Shitroid[E].proximity_counter = proximity);
        let aggro_timer: u16 = slot!(game, Shitroid[E].aggro_timer);
        let mut v5: u16 = 0;
        if aggro_timer != 0 {
            slot!(
                game,
                Shitroid[E].aggro_timer = (aggro_timer).wrapping_sub(1)
            );
            v5 = (word!(game.samus_y_pos)).wrapping_sub(18);
        } else {
            v5 = 80;
            if (word!(game.random_number) & 0xfff) >= 0xfe0 {
                slot!(game, Shitroid[E].aggro_timer = 32);
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
            Shitroid[word!(game.cur_enemy_index) as i32].proximity_counter
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
    game: &mut impl Host,
) {
    {
        game.homing_probe();
        let divisor = g_byte_A9F56A[j as usize];
        Shitroid_GraduallyAccelerateHoriz(k, r18, divisor as u16, r26, game);
        let E: i32 = k as i32;
        let distance = (slot!(game, Shitroid[E].base.y_pos) as i16).wrapping_sub(r20 as i16);
        if distance != 0 {
            slot!(
                game,
                Shitroid[E].y_velocity = homing_velocity(
                    distance,
                    slot!(game, Shitroid[E].y_velocity),
                    divisor as u8,
                    HomingReversal::ExtraForce(8),
                    1280,
                )
            );
        }
    }
}
pub fn Shitroid_GraduallyAccelerateHoriz(
    k: u16,
    r18: u16,
    r24: u16,
    r26: u16,
    game: &mut impl Host,
) {
    {
        let E: i32 = k as i32;
        let distance = (slot!(game, Shitroid[E].base.x_pos) as i16).wrapping_sub(r18 as i16);
        if distance != 0 {
            // Capture velocity before the screen check, as in both original branches.
            let velocity = slot!(game, Shitroid[E].x_velocity);
            let reversing = (velocity as i16 >= 0) == (distance > 0);
            let force = if reversing && crate::actors::shitroid::is_off_screen(k, game) & 1 != 0 {
                r26.wrapping_add(8)
            } else {
                8
            };
            slot!(
                game,
                Shitroid[E].x_velocity = homing_velocity(
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
pub fn Shitroid_AccelerateTowardsPoint(
    k: u16,
    a: u16,
    r18: u16,
    r20: u16,
    game: &mut impl Host,
) -> u8 {
    {
        let mut R28: u16 = 0;
        R28 = (R28).wrapping_add(crate::actors::shitroid::accelerate_towards_x(
            k, r18, a, game,
        ));
        R28 = (R28).wrapping_add(crate::actors::shitroid::accelerate_towards_y(
            k, r20, a, game,
        ));
        R28 = R28 >> 1;
        ((R28) as u8) & (1u8)
    }
}
pub fn Shitroid_HandleCutscenePalette_Common(r22: u16, r18: u16, game: &mut impl Host) {
    {
        let E: i32 = word!(game.cur_enemy_index) as i32;
        let mut palette_timer: i8 = slot!(byte game, Shitroid[E].palette_animation[1]) as i8;
        if palette_timer != 0 {
            slot!(byte game, Shitroid[E].palette_animation[1] = ((palette_timer) as u8).wrapping_sub(1));
        } else {
            slot!(byte game, Shitroid[E].palette_animation[1] = slot!(game, Shitroid[E].palette_handler_delay) as u8);
            let v3: u8 = (slot!(byte game, Shitroid[E].palette_animation[0]) as i32 + 1 & 7) as u8;
            slot!(byte game, Shitroid[E].palette_animation[0] = v3);
            let v4: u16 =
                crate::actors::shitroid::handle_cry(word!(game.cur_enemy_index), v3 as u16, game);
            copy_rom_colors_to_palette(r18, 0xa9, (r22 as i32 + 8 * v4 as i32) as u16, 4, game);
        };
    }
}
crate::dispatch::instruction_callbacks! { pub fn (_k: u16, _jp: EnemyScript) -> EnemyScript;
    Shitroid_Instr_3 [EnemyScript::jump(addr_kShitroid_Ilist_F90E as u16)] { };
    Shitroid_Instr_4 [EnemyScript::jump(addr_kShitroid_Ilist_F924 as u16)] { };
    Shitroid_Instr_6 [EnemyScript::jump(addr_kShitroid_Ilist_F93A as u16)] { };
}
pub fn Shitroid_Init(game: &mut impl Host) {
    word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(0));
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(
        game,
        Shitroid[E].base.properties = (slot!(game, Shitroid[E].base.properties) as i32
            | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_BlockPlasmaBeam as i32))
            as u16
    );
    slot!(game, Shitroid[E].base.palette_index = 1024);
    slot! { game, Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F90E as u16, instruction_timer = 1 }
    slot!(game, Shitroid[E].base.timer = 0);
    let mut v2: u16 = function::WAIT_FOR_CAMERA;
    if (word!(game.layer1_x_pos) & 0x8000) != 0 {
        slot!(
            game,
            Shitroid[E].base.properties = (slot!(game, Shitroid[E].base.properties) as i32
                | (kEnemyProps_Tangible as i32 | kEnemyProps_Invisible as i32))
                as u16
        );
        v2 = function::DISAPPEARED;
    }
    slot!(game, Shitroid[E].function = v2);
    slot! { game, Shitroid[E]; x_velocity = 0, y_velocity = 0 }
    slot!(game, Shitroid[E].palette_handler_delay = 10);
    slot!(game, Shitroid[E].contact_enabled = 0);
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
}
pub fn Shitroid_Powerbomb(game: &mut impl Host) {
    if slot!(
        game,
        Shitroid[word!(game.cur_enemy_index) as i32].contact_enabled
    ) != 0
    {
        Shitroid_Func_26(word!(game.cur_enemy_index), game);
    }
    Shitroid_Main(game);
}
pub fn Shitroid_Main(game: &mut impl Host) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    slot!(game, Shitroid[E].base.health = 0x7fff);
    CallShitroidFunc(
        ((slot!(game, Shitroid[E].function)) as u32) | (0xa90000u32),
        word!(game.cur_enemy_index),
        game,
    );
    MoveEnemyWithVelocity(game);
    if word!(game.palette_change_num) == 0 {
        Shitroid_HandleNormalPalette(game);
    }
}
pub fn Shitroid_Touch(game: &mut impl Host) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    if slot!(game, Shitroid[E].contact_enabled) != 0 {
        Shitroid_Func_26(word!(game.cur_enemy_index), game);
        if game.ram().read_byte(0xa1f) == 3 && word!(game.samus_x_pos) as i32 - 512i32 & 0x8000 == 0
        {
            let r18: u16 = (0x80
                - crate::actors::queries::angle_from_delta(
                    (word!(game.samus_x_pos)).wrapping_sub(slot!(game, Shitroid[E].base.x_pos)),
                    (word!(game.samus_y_pos)).wrapping_sub(slot!(game, Shitroid[E].base.y_pos)),
                ) as i32
                + 0x80) as u8 as u16;
            slot!(
                game,
                Shitroid[E].x_velocity =
                    (slot!(game, Shitroid[E].x_velocity)).wrapping_add(Math_MultBySin(0x40, r18))
            );
            slot!(
                game,
                Shitroid[E].y_velocity =
                    (slot!(game, Shitroid[E].y_velocity)).wrapping_add(Math_MultByCos(0x40, r18))
            );
        } else if slot!(game, Shitroid[E].function) == function::LATCH_ONTO_SAMUS {
            if Shitroid_AccelerateTowardsPoint(
                word!(game.cur_enemy_index),
                0x200,
                word!(game.samus_x_pos),
                (word!(game.samus_y_pos)).wrapping_sub(32),
                game,
            ) & 1
                != 0
            {
                slot! { game, Shitroid[E].base; current_instruction = addr_kShitroid_Ilist_F924 as u16, instruction_timer = 1 }
                slot!(game, Shitroid[E].base.timer = 0);
                slot!(game, Shitroid[E].palette_handler_delay = 1);
                slot! { game, Shitroid[E]; contact_enabled = 0, x_velocity = 0, y_velocity = 0 }
                slot!(game, Shitroid[E].function = function::START_DRAINING_SAMUS);
            }
        } else if slot!(game, Shitroid[E].function) == function::STARE_DOWN_SAMUS {
            slot!(game, Shitroid[E].function = function::LATCH_ONTO_SAMUS);
        }
    }
}
pub fn Shitroid_Shot(game: &mut impl Host) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    if slot!(game, Shitroid[E].contact_enabled) != 0 {
        Shitroid_Func_26(word!(game.cur_enemy_index), game);
        let r18: u16 = -(crate::actors::queries::angle_from_delta(
            (word!(game.projectile_x_pos[0])).wrapping_sub(slot!(game, Shitroid[E].base.x_pos)),
            (word!(game.projectile_y_pos[0])).wrapping_sub(slot!(game, Shitroid[E].base.y_pos)),
        ) as i32) as u8 as u16;
        let mut v1: u16 = (8u16).wrapping_mul(word!(
            game.projectile_damage[word!(game.collision_detection_index)]
        ));
        if v1 >= 0xf0 {
            v1 = 240;
        }
        slot!(
            game,
            Shitroid[E].x_velocity =
                (slot!(game, Shitroid[E].x_velocity)).wrapping_add(Math_MultBySin(v1, r18))
        );
        slot!(
            game,
            Shitroid[E].y_velocity =
                (slot!(game, Shitroid[E].y_velocity)).wrapping_add(Math_MultByCos(v1, r18))
        );
    }
}
pub fn Shitroid_Instr_5(_k: u16, jp: EnemyScript, game: &mut impl Host) -> EnemyScript {
    if (word!(game.random_number) & 0x8000) == 0 {
        return EnemyScript::jump(jp.word(0));
    }
    QueueSfx2_Max6(0x52, game);
    jp.offset(1)
}
