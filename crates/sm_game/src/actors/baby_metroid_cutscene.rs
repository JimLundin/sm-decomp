//! The baby Metroid in Mother Brain's cutscene: it dashes in, drains Mother Brain,
//! heals Samus, takes the last rainbow beam for her and dies, and Samus's
//! rainbow palette fades as the lights come back on.
//!
//! Was `BabyMetroidInCutscene_*` and `SamusRainbowPaletteFunc_*` in the original
//! decompilation (bank $A9:C710-$A9:CF00). Its own slot words are named after
//! the bank log's notes; slots 0 and $40 are Mother Brain's body and brain,
//! through `mother_brain`'s views. The bank's dispatch ids are unchanged; the
//! functions and their reads and writes keep the original order.
#![forbid(unsafe_code)]
#![allow(non_snake_case, non_upper_case_globals, unused_parens)]
#![allow(unused_mut, unused_assignments, clippy::all)]
use crate::actors::baby_metroid_encounter::{
    BabyMetroid_AccelerateTowardsPoint, BabyMetroid_Func_GraduallyAccelerateTowards0x400,
    BabyMetroid_GraduallyAccelerateTowards0x10, CallBabyMetroidCutsceneFunc0F,
    CallBabyMetroidMoveFunc,
};
use crate::actors::host::abs16;
use crate::actors::layouts::part_view;
use crate::actors::mother_brain::{MotherBrainBody, MotherBrainBrain};
use crate::actors::mother_brain_fight::{
    CallSomeSamusCode, Enemy_SetInstrList, Fade, MotherBrain_MakeWalkBackwards,
    MoveEnemyWithVelocity, QueueMusic_Delayed8, QueueSfx1_Max6, QueueSfx2_Max6, QueueSfx3_Max3,
    QueueSfx3_Max6, SpawnEprojWithRoomGfx, Unreachable, g_word_A993BB, g_word_A993C3,
    player_mother_brain, queue_mother_brain_sprite_tiles,
};
use crate::actors::mother_brain_fight::{Lease, State};
use crate::actors::session_slots::session_slot;
use crate::baby_metroid_tables;
use crate::constants::bank_86::*;
use crate::constants::bank_a9::*;
use crate::constants::enemy_properties::*;
use crate::enemy_projectiles::shared_pre::{Math_MultByCos, Math_MultBySin};
use crate::enemy_script::EnemyScript;
use crate::menus::menu_tilemap::native_record;
use crate::palette_copies::copy_rom_colors_to_palette;
use crate::ram::CompatibilityRam as _;
use crate::ram::word;

/// What the cutscene reaches beyond the baby Metroid's encounter.
pub trait Host: crate::actors::baby_metroid_encounter::Host {
    /// Turn Mother Brain's room lights off for the baby Metroid's last charge.
    fn lights_off(&mut self);
    /// Heal Samus by the baby Metroid; false once she is full. Was
    /// `heal_by_baby_metroid`.
    fn heal_by_baby_metroid(&mut self) -> bool;
}

/// The bank $A9 functions these slots run, named after the bank log's headings.
pub mod function {
    /// $A9:BE38: Mother Brain body function - drained by Shitroid - taken aback by Shitroid.
    pub const MOTHER_BRAIN_DRAINED_BY_BABY_METROID_TAKEN_ABACK: u16 = 0xbe38;
    /// $A9:C18E: Mother Brain body function - second phase - prepare for final Shitroid attack.
    pub const MOTHER_BRAIN_PREPARE_FOR_FINAL_BABY_METROID_ATTACK: u16 = 0xc18e;
    /// $A9:C19A: Mother Brain body function - second phase - execute final Shitroid attack.
    pub const MOTHER_BRAIN_EXECUTE_FINAL_BABY_METROID_ATTACK: u16 = 0xc19a;
    /// $A9:C1CF: Mother Brain body function - third phase - recover from cutscene - make some distance.
    pub const MOTHER_BRAIN_PHASE_3_RECOVER_FROM_CUTSCENE_MAKE_SOME_DISTANCE: u16 = 0xc1cf;
    /// $A9:C7CC: Shitroid function - dash onto screen.
    pub const DASH_ONTO_SCREEN: u16 = 0xc7cc;
    /// $A9:C7EC: Shitroid function - curve towards Mother Brain brain.
    pub const CURVE_TOWARDS_MOTHER_BRAIN_BRAIN: u16 = 0xc7ec;
    /// $A9:C811: Shitroid function - get right up in Mother Brain's face.
    pub const GET_RIGHT_UP_IN_MOTHER_BRAINS_FACE: u16 = 0xc811;
    /// $A9:C851: Shitroid function - latch onto Mother Brain.
    pub const LATCH_ONTO_MOTHER_BRAIN: u16 = 0xc851;
    /// $A9:C879: Shitroid function - set Mother Brain to stumble back.
    pub const SET_MOTHER_BRAIN_TO_STUMBLE_BACK: u16 = 0xc879;
    /// $A9:C889: Shitroid function - activate rainbow beam and Mother Brain body.
    pub const ACTIVATE_RAINBOW_BEAM_AND_MOTHER_BRAIN_BODY: u16 = 0xc889;
    /// $A9:C8E2: Shitroid function - wait for Mother Brain to turn to corpse.
    pub const WAIT_FOR_MOTHER_BRAIN_TO_TURN_TO_CORPSE: u16 = 0xc8e2;
    /// $A9:C915: Shitroid function - stop draining.
    pub const STOP_DRAINING: u16 = 0xc915;
    /// $A9:C94B: Shitroid function - let go and spawn dust clouds.
    pub const LET_GO_AND_SPAWN_DUST_CLOUDS: u16 = 0xc94b;
    /// $A9:C9C3: Shitroid function - move to Samus.
    pub const MOVE_TO_SAMUS: u16 = 0xc9c3;
    /// $A9:CA66: Shitroid function - latch onto Samus.
    pub const LATCH_ONTO_SAMUS: u16 = 0xca66;
    /// $A9:CA7A: Shitroid function - heal Samus up to full health.
    pub const HEAL_SAMUS_UP_TO_FULL_HEALTH: u16 = 0xca7a;
    pub const PALETTE_NONE: u16 = 0xcabc;
    /// $A9:CABD: Shitroid function - idle until run out of health.
    pub const IDLE_UNTIL_RUN_OUT_OF_HEALTH: u16 = 0xcabd;
    /// $A9:CB13: Shitroid function - release Samus.
    pub const RELEASE_SAMUS: u16 = 0xcb13;
    /// $A9:CB2D: Shitroid function - stare down Mother Brain.
    pub const STARE_DOWN_MOTHER_BRAIN: u16 = 0xcb2d;
    /// $A9:CB56: Shitroid function - fly off-screen.
    pub const FLY_OFF_SCREEN: u16 = 0xcb56;
    /// $A9:CB7B: Shitroid function - move to final charge start position.
    pub const MOVE_TO_FINAL_CHARGE_START_POSITION: u16 = 0xcb7b;
    /// $A9:CBB3: Shitroid function - initiate final charge.
    pub const INITIATE_FINAL_CHARGE: u16 = 0xcbb3;
    /// $A9:CBD8: Shitroid function - final charge.
    pub const FINAL_CHARGE: u16 = 0xcbd8;
    /// $A9:CC3E: Shitroid function - Shitroid takes fatal blow.
    pub const BABY_METROID_TAKES_FATAL_BLOW: u16 = 0xcc3e;
    /// $A9:CC60: Shitroid function - play Samus theme.
    pub const PLAY_SAMUS_THEME: u16 = 0xcc60;
    /// $A9:CC7F: Shitroid function - prepare Samus for hyper beam acquisition.
    pub const PREPARE_SAMUS_FOR_HYPER_BEAM_ACQUISITION: u16 = 0xcc7f;
    pub const DEATH_SEQUENCE_PENDING: u16 = 0xcc98;
    /// $A9:CC99: Shitroid function - death sequence.
    pub const DEATH_SEQUENCE: u16 = 0xcc99;
    /// $A9:CCC0: Shitroid function - unload Shitroid tiles.
    pub const UNLOAD_BABY_METROID_TILES: u16 = 0xccc0;
    /// $A9:CCDE: Shitroid function - let Samus rainbow some more.
    pub const LET_SAMUS_RAINBOW_SOME_MORE: u16 = 0xccde;
    /// $A9:CCF0: Shitroid function - finish cutscene.
    pub const FINISH_CUTSCENE: u16 = 0xccf0;
    /// $A9:CD30: Samus rainbow palette function - activate rainbow when enemy is low enough.
    pub const RAINBOW_ACTIVATE_RAINBOW_WHEN_ENEMY_IS_LOW_ENOUGH: u16 = 0xcd30;
    /// $A9:CD4B: Samus rainbow palette function - gradually slow animation down.
    pub const RAINBOW_GRADUALLY_SLOW_ANIMATION_DOWN: u16 = 0xcd4b;
    /// $A9:F683: Handle Shitroid in cutscene palette - normal.
    pub const HANDLE_BABY_METROID_IN_CUTSCENE_PALETTE_NORMAL: u16 = 0xf683;
    /// $A9:F68F: Handle Shitroid in cutscene palette - low health.
    pub const HANDLE_BABY_METROID_IN_CUTSCENE_PALETTE_LOW_HEALTH: u16 = 0xf68f;
}

// Each named function is the one its dispatch entry runs.
const _: () = {
    use function as f;
    const fn at(word: u16) -> u32 {
        0xa9_0000 | word as u32
    }
    assert!(at(f::MOTHER_BRAIN_DRAINED_BY_BABY_METROID_TAKEN_ABACK) == 11124280);
    assert!(at(f::MOTHER_BRAIN_PREPARE_FOR_FINAL_BABY_METROID_ATTACK) == 11125134);
    assert!(at(f::MOTHER_BRAIN_EXECUTE_FINAL_BABY_METROID_ATTACK) == 11125146);
    assert!(at(f::MOTHER_BRAIN_PHASE_3_RECOVER_FROM_CUTSCENE_MAKE_SOME_DISTANCE) == 11125199);
    assert!(at(f::DASH_ONTO_SCREEN) == 11126732);
    assert!(at(f::CURVE_TOWARDS_MOTHER_BRAIN_BRAIN) == 11126764);
    assert!(at(f::GET_RIGHT_UP_IN_MOTHER_BRAINS_FACE) == 11126801);
    assert!(at(f::LATCH_ONTO_MOTHER_BRAIN) == 11126865);
    assert!(at(f::SET_MOTHER_BRAIN_TO_STUMBLE_BACK) == 11126905);
    assert!(at(f::ACTIVATE_RAINBOW_BEAM_AND_MOTHER_BRAIN_BODY) == 11126921);
    assert!(at(f::WAIT_FOR_MOTHER_BRAIN_TO_TURN_TO_CORPSE) == 11127010);
    assert!(at(f::STOP_DRAINING) == 11127061);
    assert!(at(f::LET_GO_AND_SPAWN_DUST_CLOUDS) == 11127115);
    assert!(at(f::MOVE_TO_SAMUS) == 11127235);
    assert!(at(f::LATCH_ONTO_SAMUS) == 11127398);
    assert!(at(f::HEAL_SAMUS_UP_TO_FULL_HEALTH) == 11127418);
    assert!(at(f::PALETTE_NONE) == 11127484);
    assert!(at(f::IDLE_UNTIL_RUN_OUT_OF_HEALTH) == 11127485);
    assert!(at(f::RELEASE_SAMUS) == 11127571);
    assert!(at(f::STARE_DOWN_MOTHER_BRAIN) == 11127597);
    assert!(at(f::FLY_OFF_SCREEN) == 11127638);
    assert!(at(f::MOVE_TO_FINAL_CHARGE_START_POSITION) == 11127675);
    assert!(at(f::INITIATE_FINAL_CHARGE) == 11127731);
    assert!(at(f::FINAL_CHARGE) == 11127768);
    assert!(at(f::BABY_METROID_TAKES_FATAL_BLOW) == 11127870);
    assert!(at(f::PLAY_SAMUS_THEME) == 11127904);
    assert!(at(f::PREPARE_SAMUS_FOR_HYPER_BEAM_ACQUISITION) == 11127935);
    assert!(at(f::DEATH_SEQUENCE_PENDING) == 11127960);
    assert!(at(f::DEATH_SEQUENCE) == 11127961);
    assert!(at(f::UNLOAD_BABY_METROID_TILES) == 11128000);
    assert!(at(f::LET_SAMUS_RAINBOW_SOME_MORE) == 11128030);
    assert!(at(f::FINISH_CUTSCENE) == 11128048);
    assert!(at(f::RAINBOW_ACTIVATE_RAINBOW_WHEN_ENEMY_IS_LOW_ENOUGH) == 11128112);
    assert!(at(f::RAINBOW_GRADUALLY_SLOW_ANIMATION_DOWN) == 11128139);
    assert!(at(f::HANDLE_BABY_METROID_IN_CUTSCENE_PALETTE_NORMAL) == 11138691);
    assert!(at(f::HANDLE_BABY_METROID_IN_CUTSCENE_PALETTE_LOW_HEALTH) == 11138703);
};

part_view! {
    /// The baby Metroid in the cutscene's view of its slot.
    BabyMetroidInCutscene: EnemyLayout72 {
        words: [
            function, x_velocity, y_velocity,
            // $0FAE: the palette animation frame (low byte) and timer (high byte).
            palette_animation,
            palette_handler_delay, function_timer, parameter_1, parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            var_00, var_01, var_02, var_03,
            // $7E:7808: the baby Metroid's cry.
            cry_enabled, var_05,
            // $7E:780C: frames of flashing left.
            flash_timer,
            // $7E:780E and $7E:7810: the death explosions' index and timer.
            death_explosion_index, death_explosion_timer,
            // $7E:7812: the health-based palette.
            health_palette_enabled,
            // $7E:7814 and $7E:7816: the enemy angle and speed it steers by.
            angle, speed,
            // $7E:7818 and $7E:781A: fading to black, timer and index.
            fade_timer, fade_index,
            // $7E:781C: the movement table pointer; $7E:781E: the palette function.
            movement, palette_function,
            // $7E:7820 and $7E:7822: where the final shake is centred.
            shake_origin_x, shake_origin_y,
            var_12, var_13, var_14, var_15, var_16, var_17,
            var_18, var_19, var_1A, var_1B, var_1C, var_1D,
            var_1E, var_1F,
        ],
        pad2: [u8; 1984],
        words: [
            var_20, var_21, var_22, var_23, var_24, var_25,
            var_26, var_27, var_28, var_29, var_2A, var_2B,
            var_2C, var_2D, var_2E, var_2F, var_30, var_31,
            var_32, var_33, var_34, var_35, var_36, var_37,
            var_38, var_39, var_3A, var_3B, var_3C, var_3D,
            var_3E, var_3F,
        ],
    }
}

/// $A9:CDFC: the death explosions' offsets from the baby Metroid, (x, y) pairs.
const DEATH_EXPLOSION_OFFSETS: u32 = 0xa9_cdfc;

crate::dispatch::callback_dispatch! {
    pub fn CallBabyMetroidCutsceneFuncA(ea: u32, k: u16, game: &mut impl Host) => ea;
    call (k, game); default { Unreachable(game); };
    @body checked;
    @functions [pub fn (k: u16, game: &mut (impl State + ?Sized))];
    [11126764] BabyMetroidInCutscene_CurveTowardsBrain [] {
        BabyMetroidInCutscene_UpdateSpeedAndAngle(k, -384i32 as u16, -20480i32 as u16, 2560, game);
        let E: i32 = k as i32;
        session_slot!(game, BabyMetroidInCutscene[E].function_timer = session_slot!(game, BabyMetroidInCutscene[E].function_timer).wrapping_sub(1));
        if (session_slot!(game, BabyMetroidInCutscene[E].function_timer) & 0x8000) != 0 {
            session_slot!(game, BabyMetroidInCutscene[E].function = function::GET_RIGHT_UP_IN_MOTHER_BRAINS_FACE);
            session_slot!(game, BabyMetroidInCutscene[E].function_timer = 9);
        }
    };
    @functions [pub fn (k: u16, game: &mut impl Host)];
    [11126801] BabyMetroidInCutscene_GetIntoFace [] {
        BabyMetroidInCutscene_UpdateSpeedAndAngle(k, -1536i32 as u16, -32256i32 as u16, 3584, game);
        let mut E: i32 = 0x40;
        let rect = (session_slot!(game, MotherBrainBrain[E].base.x_pos), session_slot!(game, MotherBrainBrain[E].base.y_pos), 4, 4);
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 || {
            E = k as i32;
            let mut v3: bool = ((session_slot!(game, BabyMetroidInCutscene[E].function_timer)) as i16).wrapping_sub(1) < 0;
            session_slot!(game, BabyMetroidInCutscene[E].function_timer = session_slot!(game, BabyMetroidInCutscene[E].function_timer).wrapping_sub(1));
            v3 as i32 != 0
        } {
            session_slot!(game, BabyMetroidInCutscene[k as i32].function = function::LATCH_ONTO_MOTHER_BRAIN);
            player_mother_brain(1, game);
        }
    };
    [11126865] BabyMetroidInCutscene_LatchOntoBrain [] {
        let E: i32 = 0x40;
        let rect = (session_slot!(game, MotherBrainBrain[E].base.x_pos), ((session_slot!(game, MotherBrainBrain[E].base.y_pos)).wrapping_sub(24)), 8, 8);
        BabyMetroid_Func_GraduallyAccelerateTowards0x400(k, 0, rect.0, rect.1, game);
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 {
            session_slot!(game, BabyMetroidInCutscene[k as i32].function = function::SET_MOTHER_BRAIN_TO_STUMBLE_BACK);
        }
    };
    [11126905] BabyMetroidInCutscene_SetMotherBrainToStumbleBack [] {
        let E: i32 = 0;
        MotherBrain_MakeWalkBackwards((session_slot!(game, MotherBrainBody[E].base.x_pos)).wrapping_sub(1), 2, game);
        session_slot!(game, BabyMetroidInCutscene[k as i32].function = function::ACTIVATE_RAINBOW_BEAM_AND_MOTHER_BRAIN_BODY);
        BabyMetroidInCutscene_ActivateRainbowBeam(k, game);
    };
    [11126921] BabyMetroidInCutscene_ActivateRainbowBeam [] {
        let E1: i32 = 0x40;
        if BabyMetroid_AccelerateTowardsPoint(
            k,
            0x200,
            session_slot!(game, MotherBrainBrain[E1].base.x_pos),
            (session_slot!(game, MotherBrainBrain[E1].base.y_pos)).wrapping_sub(24),
            game,
        ) & 1
            != 0
        {
            let E: i32 = k as i32;
            session_slot! { game, BabyMetroidInCutscene[E]; x_velocity = 0, y_velocity = 0 }
            session_slot!(game, BabyMetroidInCutscene[E].base.x_pos = session_slot!(game, MotherBrainBrain[E1].base.x_pos));
            session_slot!(game, BabyMetroidInCutscene[E].base.y_pos = (session_slot!(game, MotherBrainBrain[E1].base.y_pos)).wrapping_sub(24));
            Enemy_SetInstrList(k, addr_kBabyMetroid_Ilist_CFB8 as u16, game);
            session_slot!(game, BabyMetroidInCutscene[E].function = function::WAIT_FOR_MOTHER_BRAIN_TO_TURN_TO_CORPSE);
            session_slot!(game, BabyMetroidInCutscene[E].palette_handler_delay = 1);
            let E0: i32 = 0;
            session_slot!(game, MotherBrainBody[E0].function = function::MOTHER_BRAIN_DRAINED_BY_BABY_METROID_TAKEN_ABACK);
            QueueSfx1_Max6(0x40, game);
            session_slot!(game, MotherBrainBody[E0].beam_sound_playing = 1);
        }
    };
    @functions [pub fn (k: u16, game: &mut (impl State + ?Sized))];
    [11127061] BabyMetroidInCutscene_StopDraining [] {
        let E: i32 = k as i32;
        let E1: i32 = 0x40;
        session_slot!(game, BabyMetroidInCutscene[E].base.x_pos = session_slot!(game, MotherBrainBrain[E1].base.x_pos));
        session_slot!(game, BabyMetroidInCutscene[E].base.y_pos = (session_slot!(game, MotherBrainBrain[E1].base.y_pos)).wrapping_sub(24));
        session_slot!(game, BabyMetroidInCutscene[E].function_timer = session_slot!(game, BabyMetroidInCutscene[E].function_timer).wrapping_sub(1));
        if (session_slot!(game, BabyMetroidInCutscene[E].function_timer) & 0x8000) != 0 {
            Enemy_SetInstrList(k, addr_kBabyMetroid_Ilist_CFA2 as u16, game);
            session_slot!(game, BabyMetroidInCutscene[E].palette_handler_delay = 10);
            session_slot!(game, BabyMetroidInCutscene[E].function = function::LET_GO_AND_SPAWN_DUST_CLOUDS);
            session_slot!(game, BabyMetroidInCutscene[E].function_timer = 32);
            session_slot! { game, BabyMetroidInCutscene[E]; x_velocity = 0, y_velocity = 0 }
        }
    };
    @functions [pub fn (k: u16, game: &mut impl Host)];
    [11127129] BabyMetroidInCutscene_MoveUpToCeiling [] {
        let rect = (session_slot!(game, MotherBrainBrain[0x40i32].base.x_pos), 0, 4, 4);
        BabyMetroid_Func_GraduallyAccelerateTowards0x400(k, 0, rect.0, rect.1, game);
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 {
            player_mother_brain(4, game);
            let E: i32 = k as i32;
            session_slot!(game, BabyMetroidInCutscene[E].function = function::MOVE_TO_SAMUS);
            session_slot!(game, BabyMetroidInCutscene[E].movement = addr_stru_A9CA24 as u16);
        }
    };
    [11127398] BabyMetroidInCutscene_LatchOntoSamus [] {
        BabyMetroid_Func_GraduallyAccelerateTowards0x400(
            k,
            0,
            word!(game.samus_x_pos),
            (word!(game.samus_y_pos)).wrapping_sub(20),
            game,
        );
    };
    [11127485] BabyMetroidInCutscene_IdleUntilToNoHealth [] {
        BabyMetroidInCutscene_HandleCry(game);
        let E: i32 = k as i32;
        let flash_timer: u16 = session_slot!(game, BabyMetroidInCutscene[E].flash_timer);
        if flash_timer != 0 {
            let v3: i32 = (flash_timer as i32 & 6) >> 1;
            session_slot!(game, BabyMetroidInCutscene[E].base.x_pos = (word!(game.samus_x_pos) as i32 + 2 * g_word_A993BB[v3 as usize] as i32) as u16);
            session_slot!(game, BabyMetroidInCutscene[E].base.y_pos = (word!(game.samus_y_pos) as i32
                + 2 * g_word_A993C3[v3 as usize] as i32
                - 20i32) as u16);
        }
        if session_slot!(game, BabyMetroidInCutscene[E].base.health) == 0 {
            session_slot!(game, BabyMetroidInCutscene[E].base.health = 320);
            session_slot! { game, BabyMetroidInCutscene[E]; function = function::RELEASE_SAMUS, palette_handler_delay = 10, palette_animation = 0, palette_function = function::HANDLE_BABY_METROID_IN_CUTSCENE_PALETTE_LOW_HEALTH, cry_enabled = 1, health_palette_enabled = 0 };
        }
    };
    [11127571] BabyMetroidInCutscene_ReleaseSamus [] {
        QueueSfx2_Max6(0x72, game);
        session_slot!(game, MotherBrainBrain[0x40i32].baby_metroid_released = 1);
        session_slot!(game, BabyMetroidInCutscene[k as i32].function = function::STARE_DOWN_MOTHER_BRAIN);
        session_slot!(game, MotherBrainBody[0i32].function = function::MOTHER_BRAIN_PREPARE_FOR_FINAL_BABY_METROID_ATTACK);
        BabyMetroidInCutscene_StareDownMotherBrain(k, game);
    };
    [11127597] BabyMetroidInCutscene_StareDownMotherBrain [] {
        let rect = (((word!(game.samus_x_pos)).wrapping_sub(4)), 96, 4, 4);
        BabyMetroid_GraduallyAccelerateTowards0x10(k, 0, rect.0, rect.1, game);
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 {
            session_slot!(game, BabyMetroidInCutscene[k as i32].function = function::FLY_OFF_SCREEN);
        }
    };
    [11127638] BabyMetroidInCutscene_FlyOffScreen [] {
        let rect = (272, 64, 4, 4);
        BabyMetroid_GraduallyAccelerateTowards0x10(k, 0, rect.0, rect.1, game);
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 {
            session_slot!(game, BabyMetroidInCutscene[k as i32].function = function::MOVE_TO_FINAL_CHARGE_START_POSITION);
        }
    };
    [11127675] BabyMetroidInCutscene_MoveToFinalChargeStart [] {
        let rect = (305, 160, 4, 4);
        BabyMetroid_GraduallyAccelerateTowards0x10(k, 0, rect.0, rect.1, game);
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 {
            let E: i32 = k as i32;
            session_slot!(game, BabyMetroidInCutscene[E].base.health = 79);
            session_slot!(game, MotherBrainBrain[0x40i32].baby_metroid_released = 0);
            session_slot!(game, MotherBrainBody[0i32].function = function::MOTHER_BRAIN_EXECUTE_FINAL_BABY_METROID_ATTACK);
            session_slot!(game, BabyMetroidInCutscene[E].function = function::INITIATE_FINAL_CHARGE);
        }
    };
    [11127731] BabyMetroidInCutscene_InitiateFinalCharge [] {
        let rect = (290, 128, 4, 4);
        BabyMetroid_Func_GraduallyAccelerateTowards0x400(k, 0xa, rect.0, rect.1, game);
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 {
            session_slot!(game, BabyMetroidInCutscene[k as i32].function = function::FINAL_CHARGE);
        }
    };
    [11127768] BabyMetroidInCutscene_FinalCharge [] {
        let E1: i32 = 0x40;
        BabyMetroid_Func_GraduallyAccelerateTowards0x400(
            k,
            0xc,
            session_slot!(game, MotherBrainBrain[E1].base.x_pos),
            (session_slot!(game, MotherBrainBrain[E1].base.y_pos)).wrapping_sub(32),
            game,
        );
        let E: i32 = k as i32;
        if session_slot!(game, BabyMetroidInCutscene[E].base.health) == 0 {
            session_slot!(game, BabyMetroidInCutscene[E].palette_function = function::DEATH_SEQUENCE_PENDING);
            session_slot!(game, BabyMetroidInCutscene[E].base.vram_tiles_index = 4256);
            QueueSfx3_Max6(0x19, game);
            game.lights_off();
            Enemy_SetInstrList(k, addr_kBabyMetroid_Ilist_CFCE as u16, game);
            session_slot! { game, BabyMetroidInCutscene[E]; x_velocity = 0, y_velocity = 0 }
            session_slot!(game, MotherBrainBody[0i32].function = addr_locret_A9C18D as u16);
            QueueMusic_Delayed8(0, game);
            session_slot!(game, BabyMetroidInCutscene[E].function = function::BABY_METROID_TAKES_FATAL_BLOW);
            session_slot!(game, BabyMetroidInCutscene[E].function_timer = 16);
            session_slot!(game, BabyMetroidInCutscene[E].shake_origin_x = session_slot!(game, BabyMetroidInCutscene[E].base.x_pos));
            session_slot!(game, BabyMetroidInCutscene[E].shake_origin_y = session_slot!(game, BabyMetroidInCutscene[E].base.y_pos));
            BabyMetroidInCutscene_BabyMetroidFinalBelow(k, game);
        }
    };
    [11127870] BabyMetroidInCutscene_BabyMetroidFinalBelow [] {
        BabyMetroidInCutscene_Shake(k, game);
        let E: i32 = k as i32;
        session_slot!(game, BabyMetroidInCutscene[E].function_timer = session_slot!(game, BabyMetroidInCutscene[E].function_timer).wrapping_sub(1));
        if (session_slot!(game, BabyMetroidInCutscene[E].function_timer) & 0x8000) != 0 {
            session_slot!(game, BabyMetroidInCutscene[E].base.x_pos = session_slot!(game, BabyMetroidInCutscene[E].shake_origin_x));
            session_slot!(game, BabyMetroidInCutscene[E].base.y_pos = session_slot!(game, BabyMetroidInCutscene[E].shake_origin_y));
            session_slot!(game, BabyMetroidInCutscene[E].function = function::PLAY_SAMUS_THEME);
            session_slot!(game, BabyMetroidInCutscene[E].function_timer = 56);
            BabyMetroidInCutscene_PlaySamusTheme(k, game);
        }
    };
    [11127904] BabyMetroidInCutscene_PlaySamusTheme [] {
        let E: i32 = k as i32;
        session_slot!(game, BabyMetroidInCutscene[E].function_timer = session_slot!(game, BabyMetroidInCutscene[E].function_timer).wrapping_sub(1));
        if (session_slot!(game, BabyMetroidInCutscene[E].function_timer) & 0x8000) != 0 {
            QueueMusic_Delayed8(0xff48, game);
            QueueMusic_Delayed8(5, game);
            session_slot!(game, BabyMetroidInCutscene[E].function = function::PREPARE_SAMUS_FOR_HYPER_BEAM_ACQUISITION);
            session_slot!(game, BabyMetroidInCutscene[E].function_timer = 12);
            BabyMetroidInCutscene_PrepareSamusHyperbeam(k, game);
        }
    };
    [11127961] BabyMetroidInCutscene_DeathSequence [] {
        HandleSamusRainbowPaletteAnimation(k, game);
        BabyMetroidInCutscene_AccelerateDownwards(k, game);
        if BabyMetroidInCutscene_FadeBabyMetroidToBlack(k, game) & 1 != 0 {
            let E: i32 = k as i32;
            session_slot!(game, BabyMetroidInCutscene[E].base.properties = (session_slot!(game, BabyMetroidInCutscene[E].base.properties)) | ((kEnemyProps_Invisible) as u16));
            session_slot!(game, BabyMetroidInCutscene[E].function = function::UNLOAD_BABY_METROID_TILES);
            session_slot!(game, BabyMetroidInCutscene[E].function_timer = 128);
        } else {
            BabyMetroidInCutscene_HandleBabyMetroidDeathExplosions(k, game);
            BabyMetroidInCutscene_HandleEnemyBlinking(k, game);
        };
    };
    [11128000] BabyMetroidInCutscene_UnloadBabyMetroid [] {
        HandleSamusRainbowPaletteAnimation(k, game);
        let E: i32 = k as i32;
        session_slot!(game, BabyMetroidInCutscene[E].function_timer = session_slot!(game, BabyMetroidInCutscene[E].function_timer).wrapping_sub(1));
        let v2: bool = (session_slot!(game, BabyMetroidInCutscene[E].function_timer) & 0x8000) != 0;
        if v2 && queue_mother_brain_sprite_tiles(0xa9, addr_stru_A98FC7 as u16, game) != 0 {
            session_slot!(game, BabyMetroidInCutscene[E].function = function::LET_SAMUS_RAINBOW_SOME_MORE);
            session_slot!(game, BabyMetroidInCutscene[E].function_timer = 176);
            BabyMetroidInCutscene_LetSamusRainbowMore(k, game);
        }
    };
    [11128048] BabyMetroidInCutscene_FinishCutscene [] {
        let E: i32 = 0;
        let v2: u16 = (session_slot!(game, MotherBrainBody[E].grey_transition)).wrapping_add(1);
        session_slot!(game, MotherBrainBody[E].grey_transition = v2);
        if game.fade(Fade::LightsOn, (v2).wrapping_sub(1)) & 1 != 0 {
            session_slot!(game, MotherBrainBody[E].function = function::MOTHER_BRAIN_PHASE_3_RECOVER_FROM_CUTSCENE_MAKE_SOME_DISTANCE);
            CallSomeSamusCode(0x17, game);
            player_mother_brain(3, game);
            let fresh12: i32 = k as i32;
            session_slot!(game, BabyMetroidInCutscene[fresh12].base.properties, |old| old | (kEnemyProps_Deleted as u16));
            session_slot!(game, MotherBrainBrain[0x40i32].baby_metroid = 0);
        }
    };
    @functions [pub fn (k: u16, game: &mut (impl State + ?Sized))];
    @prefix { let E: i32 = k as i32; };
    [11127010] BabyMetroidInCutscene_BrainTurnsToCorpse [] {
        let E1: i32 = 0x40;
        let v2: i32 = (session_slot!(game, BabyMetroidInCutscene[E].base.frame_counter) as i32 & 6) >> 1;
        session_slot!(game, BabyMetroidInCutscene[E].base.x_pos = (session_slot!(game, MotherBrainBrain[E1].base.x_pos)).wrapping_add((g_word_A993BB[v2 as usize]) as u16));
        session_slot!(game, BabyMetroidInCutscene[E].base.y_pos = (session_slot!(game, MotherBrainBrain[E1].base.y_pos) as i32 + g_word_A993C3[v2 as usize] as i32 - 24i32) as u16);
        if session_slot!(game, MotherBrainBody[0i32].corpse_state) != 0 {
            session_slot!(game, BabyMetroidInCutscene[E].function = function::STOP_DRAINING);
            session_slot!(game, BabyMetroidInCutscene[E].function_timer = 64);
        }
    };
    @functions [pub fn (k: u16, game: &mut impl Host)];
    @prefix { let E: i32 = k as i32; };
    [11127115] BabyMetroidInCutscene_LetGoAndSpawnDust [] {
        session_slot!(game, BabyMetroidInCutscene[E].function_timer = session_slot!(game, BabyMetroidInCutscene[E].function_timer).wrapping_sub(1));
        if (session_slot!(game, BabyMetroidInCutscene[E].function_timer) & 0x8000) != 0 {
            BabyMetroidInCutscene_SpawnThreeDustClouds(game);
            session_slot!(game, BabyMetroidInCutscene[E].function = addr_loc_A9C959 as u16);
        }
        BabyMetroidInCutscene_MoveUpToCeiling(k, game);
    };
    [11127235] BabyMetroidInCutscene_MoveToSamus [] {
        session_slot!(game, BabyMetroidInCutscene[E].cry_enabled = 0);
        session_slot!(game, BabyMetroidInCutscene[E].health_palette_enabled = 1);
        if (word!(game.random_number) & 0xfff) >= 0xfa0 {
            QueueSfx2_Max6(0x52, game);
        }
        let movement: u16 = session_slot!(game, BabyMetroidInCutscene[E].movement);
        let v3 = crate::cartridge::native_record(0xa9, movement, game);
        let rect = (v3.word(0), v3.word(2), 4, 4);
        CallBabyMetroidMoveFunc(
            (v3.word(6) as u32) | (0xa90000u32),
            k,
            v3.word(4),
            rect.0,
            rect.1,
            game,
        );
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 {
            let mut v4: i16 = crate::cartridge::native_record(0xa9, movement, game).word(8) as i16;
            if v4 < 0 {
                session_slot!(game, BabyMetroidInCutscene[E].function = v4 as u16);
            } else {
                session_slot!(game, BabyMetroidInCutscene[E].movement = (session_slot!(game, BabyMetroidInCutscene[E].movement)).wrapping_add(8));
            }
        }
    };
    [11127418] BabyMetroidInCutscene_HealSamusToFullHealth [] {
        session_slot!(game, BabyMetroidInCutscene[E].cry_enabled = 0);
        BabyMetroidInCutscene_HandleCry(game);
        let v2: i32 = (session_slot!(game, BabyMetroidInCutscene[E].base.frame_counter) as i32 & 6) >> 1;
        session_slot!(game, BabyMetroidInCutscene[E].base.x_pos = (word!(game.samus_x_pos)).wrapping_add((g_word_A993BB[v2 as usize]) as u16));
        session_slot!(game, BabyMetroidInCutscene[E].base.y_pos = (word!(game.samus_y_pos) as i32 + g_word_A993C3[v2 as usize] as i32 - 20i32) as u16);
        if !game.heal_by_baby_metroid() {
            word!(game.samus_reserve_health = word!(game.samus_max_reserve_health));
            session_slot!(game, BabyMetroidInCutscene[E].function = function::IDLE_UNTIL_RUN_OUT_OF_HEALTH);
            session_slot!(game, BabyMetroidInCutscene[E].palette_function = function::PALETTE_NONE);
        }
    };
    BabyMetroidInCutscene_HandleBabyMetroidDeathExplosions [] {
        let mut v2: i16 = ((session_slot!(game, BabyMetroidInCutscene[E].death_explosion_timer)) as i16).wrapping_sub(1);
        if v2 < 0 {
            session_slot!(game, BabyMetroidInCutscene[E].death_explosion_timer = 4);
            let mut v3: u16 = (session_slot!(game, BabyMetroidInCutscene[E].death_explosion_index)).wrapping_add(1);
            if session_slot!(game, BabyMetroidInCutscene[E].death_explosion_index) as i32 - 9i32 & 0x8000 == 0 {
                v3 = 0;
            }
            session_slot!(game, BabyMetroidInCutscene[E].death_explosion_index = v3);
            let v4: i32 = ((4u16).wrapping_mul(v3)) as i32 >> 1;
            {
                let offsets = native_record(DEATH_EXPLOSION_OFFSETS, game);
                let x = (session_slot!(game, BabyMetroidInCutscene[E].base.x_pos))
                    .wrapping_add(offsets.table_word(v4 as isize));
                let y = (session_slot!(game, BabyMetroidInCutscene[E].base.y_pos))
                    .wrapping_add(offsets.table_word((v4 + 1) as isize));
                crate::actors::host::BossEffects::set_projectile_spawn_point(game, x, y);
            }
            SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 3, game);
            QueueSfx3_Max3(0x13, game);
        } else {
            session_slot!(game, BabyMetroidInCutscene[E].death_explosion_timer = v2 as u16);
        };
    };
    BabyMetroidInCutscene_HandleHealthBasedPalette [] {
        if session_slot!(game, BabyMetroidInCutscene[E].health_palette_enabled) != 0 {
            let health: u16 = session_slot!(game, BabyMetroidInCutscene[E].base.health);
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
                let v4: u16 = baby_metroid_tables::shell_palette(game, (v3 >> 1) as isize).colors;
                copy_rom_colors_to_palette(0x1e2, 0xad, v4, 4, game);
                copy_rom_colors_to_palette(0x1f4, 0xad, (v4).wrapping_add(8), 5, game);
                copy_rom_colors_to_palette(
                    0x1ea,
                    0xad,
                    baby_metroid_tables::innards_palette(game, (v5 >> 1) as isize).colors,
                    5,
                    game,
                );
            }
        }
    };
    @functions [pub fn (k: u16, game: &mut (impl State + ?Sized))];
    @prefix { let E: i32 = k as i32; };
    BabyMetroidInCutscene_Shake [] {
        session_slot!(game, BabyMetroidInCutscene[E].y_velocity = session_slot!(game, BabyMetroidInCutscene[E].y_velocity).wrapping_sub(1));
        let v2: i32 = (session_slot!(game, BabyMetroidInCutscene[E].base.frame_counter) as i32 & 6) >> 1;
        session_slot!(game, BabyMetroidInCutscene[E].base.x_pos = (session_slot!(game, BabyMetroidInCutscene[E].shake_origin_x)).wrapping_add((g_word_A993BB[v2 as usize]) as u16));
        session_slot!(game, BabyMetroidInCutscene[E].base.y_pos = (session_slot!(game, BabyMetroidInCutscene[E].shake_origin_y)).wrapping_add((g_word_A993C3[v2 as usize]) as u16));
    };
    @countdown [pub fn (k: u16, game: &mut impl Host)];
    before {
        let E: i32 = k as i32;
        session_slot!(game, BabyMetroidInCutscene[E].function_timer = session_slot!(game, BabyMetroidInCutscene[E].function_timer).wrapping_sub(1));
    }
    when (session_slot!(game, BabyMetroidInCutscene[E].function_timer) & 0x8000) != 0;
    [11126732] BabyMetroidInCutscene_DashOntoScreen {
        session_slot!(game, BabyMetroidInCutscene[E].angle = -10240i32 as u16);
        session_slot!(game, BabyMetroidInCutscene[E].speed = 2560);
        session_slot!(game, BabyMetroidInCutscene[E].function = function::CURVE_TOWARDS_MOTHER_BRAIN_BRAIN);
        session_slot!(game, BabyMetroidInCutscene[E].function_timer = 10);
        BabyMetroidInCutscene_CurveTowardsBrain(k, game);
    };
    [11127935] BabyMetroidInCutscene_PrepareSamusHyperbeam {
        CallSomeSamusCode(0x19, game);
        session_slot!(game, MotherBrainBrain[0x40i32].samus_rainbow_function = function::RAINBOW_ACTIVATE_RAINBOW_WHEN_ENEMY_IS_LOW_ENOUGH);
        session_slot!(game, BabyMetroidInCutscene[E].function = function::DEATH_SEQUENCE);
    };
    [11128030] BabyMetroidInCutscene_LetSamusRainbowMore {
        session_slot!(game, BabyMetroidInCutscene[E].function = function::FINISH_CUTSCENE);
        session_slot!(game, MotherBrainBody[0i32].grey_transition = 0);
        BabyMetroidInCutscene_FinishCutscene(k, game);
    };
    @external {
        11125133 => [()];
    }
}
/// Run the cutscene baby Metroid's function `ea` on the lease when it needs no more than the
/// state, and on the game otherwise.
fn run_cutscene_function<H: Host, L: Lease<Game = H> + ?Sized>(ea: u32, k: u16, lease: &mut L) {
    match ea {
        11126764 => BabyMetroidInCutscene_CurveTowardsBrain(k, lease),
        11127061 => BabyMetroidInCutscene_StopDraining(k, lease),
        11127010 => BabyMetroidInCutscene_BrainTurnsToCorpse(k, lease),
        11125133 => {}
        _ => lease.foreign(|game| CallBabyMetroidCutsceneFuncA(ea, k, game)),
    }
}

pub fn BabyMetroidInCutscene_Flashing(k: u16, game: &mut (impl State + ?Sized)) {
    let mut v1: i16 = 3584;
    let E: i32 = k as i32;
    let flash_timer: u16 = session_slot!(game, BabyMetroidInCutscene[E].flash_timer);
    if flash_timer != 0 {
        let v4: u16 = (flash_timer).wrapping_sub(1);
        session_slot!(game, BabyMetroidInCutscene[E].flash_timer = v4);
        if (v4 & 2) != 0 {
            v1 = 0;
        }
    }
    session_slot!(
        game,
        BabyMetroidInCutscene[E].base.palette_index = v1 as u16
    );
}
pub fn BabyMetroidInCutscene_HandleCry(game: &mut impl Host) {
    let E: i32 = 0;
    if session_slot!(game, MotherBrainBody[E].baby_metroid_cry) != 0 {
        session_slot!(game, MotherBrainBody[E].baby_metroid_cry = 0);
        QueueSfx2_Max6(0x72, game);
    }
}
pub fn BabyMetroidInCutscene_SpawnOneDustCloudAt(xd: u16, yd: u16, game: &mut impl Host) {
    let E: i32 = 0x40;
    {
        let (x, y) = (
            ((session_slot!(game, MotherBrainBrain[E].base.x_pos)).wrapping_add(xd)),
            ((session_slot!(game, MotherBrainBrain[E].base.y_pos)).wrapping_add(yd)),
        );
        crate::actors::host::BossEffects::set_projectile_spawn_point(game, x, y);
    }
    SpawnEprojWithRoomGfx(addr_kEproj_DustCloudExplosion as u16, 9, game);
}
pub fn BabyMetroidInCutscene_SpawnThreeDustClouds(game: &mut impl Host) {
    {
        BabyMetroidInCutscene_SpawnOneDustCloudAt(-16i32 as u16, -8i32 as u16, game);
        BabyMetroidInCutscene_SpawnOneDustCloudAt(0, -16i32 as u16, game);
        BabyMetroidInCutscene_SpawnOneDustCloudAt(16, -8i32 as u16, game);
    }
}
crate::dispatch::callback_dispatch! {
    pub fn CallSamusRainbowPaletteFunc(ea: u32, k: u16, game: &mut impl Host) => ea;
    call (k, game); default { Unreachable(game); };
    @body checked;
    @functions [pub fn (k: u16, game: &mut impl Host)];
    [11128112] SamusRainbowPaletteFunc_ActivateWhenEnemyLow [] {
        if ((session_slot!(game, BabyMetroidInCutscene[k as i32].base.y_pos) as i32 + 16
            - word!(game.samus_y_pos) as i32) as i16)
            >= 0
        {
            CallSomeSamusCode(0x16, game);
            session_slot!(game, MotherBrainBrain[0x40i32].samus_rainbow_function = function::RAINBOW_GRADUALLY_SLOW_ANIMATION_DOWN);
        }
    };
    @external {
        11128139 => [SamusRainbowPaletteFunc_GraduallySlowDown(game)];
    }
}
pub fn HandleSamusRainbowPaletteAnimation(k: u16, game: &mut impl Host) {
    {
        let r18: u16 = session_slot!(game, MotherBrainBrain[0x40i32].samus_rainbow_function);
        CallSamusRainbowPaletteFunc(((r18) as u32) | (0xa90000u32), k, game);
    }
}
pub fn SamusRainbowPaletteFunc_GraduallySlowDown(game: &mut (impl State + ?Sized)) {
    let E: i32 = 0x40;
    let rainbow_counter: u16 = session_slot!(game, MotherBrainBrain[E].samus_rainbow_counter);
    session_slot!(
        game,
        MotherBrainBrain[E].samus_rainbow_counter = (rainbow_counter).wrapping_add(768)
    );
    if rainbow_counter >= 0xfd00 {
        let mut v2: u16 = (word!(game.special_samus_palette_frame)).wrapping_add(1);
        if word!(game.special_samus_palette_frame) as i32 - 9i32 & 0x8000 == 0 {
            v2 = 10;
        }
        word!(game.special_samus_palette_frame = v2);
    }
}
pub fn BabyMetroidInCutscene_FadeBabyMetroidToBlack(k: u16, game: &mut impl Host) -> u8 {
    {
        let E: i32 = k as i32;
        if session_slot!(game, BabyMetroidInCutscene[E].base.y_pos) as i32 - 128i32 & 0x8000 != 0 {
            return 0;
        }
        let mut v2: i16 =
            ((session_slot!(game, BabyMetroidInCutscene[E].fade_timer)) as i16).wrapping_sub(1);
        if v2 >= 0 {
            session_slot!(game, BabyMetroidInCutscene[E].fade_timer = v2 as u16);
            return 0;
        }
        session_slot!(game, BabyMetroidInCutscene[E].fade_timer = 8);
        let v4: u16 = (session_slot!(game, BabyMetroidInCutscene[E].fade_index)).wrapping_add(1);
        if session_slot!(game, BabyMetroidInCutscene[E].fade_index) as i32 - 6i32 & 0x8000 == 0 {
            return 1;
        }
        session_slot!(game, BabyMetroidInCutscene[E].fade_index = v4);
        copy_rom_colors_to_palette(
            0x1e2,
            0xad,
            baby_metroid_tables::fade_palette(game, v4 as isize).colors,
            0xe,
            game,
        );
        0
    }
}
pub fn BabyMetroidInCutscene_HandleEnemyBlinking(k: u16, game: &mut (impl State + ?Sized)) {
    let E: i32 = k as i32;
    if (session_slot!(game, BabyMetroidInCutscene[E].base.frame_counter) & 1) != 0 {
        session_slot!(
            game,
            BabyMetroidInCutscene[E].base.properties =
                (session_slot!(game, BabyMetroidInCutscene[E].base.properties) as i32
                    & !(kEnemyProps_Invisible as i32)) as u16
        );
    } else {
        session_slot!(
            game,
            BabyMetroidInCutscene[E].base.properties =
                (session_slot!(game, BabyMetroidInCutscene[E].base.properties))
                    | ((kEnemyProps_Invisible) as u16)
        );
    };
}
pub fn BabyMetroidInCutscene_AccelerateDownwards(k: u16, game: &mut (impl State + ?Sized)) {
    let E: i32 = k as i32;
    let mut v2: i16 =
        ((abs16(session_slot!(game, BabyMetroidInCutscene[E].x_velocity))) as i16).wrapping_sub(32);
    if v2 < 0 {
        v2 = 0;
    }
    session_slot!(
        game,
        BabyMetroidInCutscene[E].x_velocity =
            (if session_slot!(game, BabyMetroidInCutscene[E].x_velocity) & 0x8000 != 0 {
                -(v2 as i32)
            } else {
                v2 as i32
            }) as u16
    );
    session_slot!(
        game,
        BabyMetroidInCutscene[E].y_velocity =
            (session_slot!(game, BabyMetroidInCutscene[E].y_velocity)).wrapping_add(2)
    );
}
pub fn BabyMetroidInCutscene_UpdateSpeedAndAngle(
    k: u16,
    mut r18: u16,
    r20: u16,
    r22: u16,
    game: &mut (impl State + ?Sized),
) {
    {
        let E: i32 = k as i32;
        // Steer the speed 20h a frame towards `r22` and the angle by `r18`
        // up to `r20`, then set the velocities along the angle.
        if r22 != session_slot!(game, BabyMetroidInCutscene[E].speed) {
            if ((r22) as i16)
                .wrapping_sub((session_slot!(game, BabyMetroidInCutscene[E].speed)) as i16)
                >= 0
            {
                let mut v2: u16 =
                    (session_slot!(game, BabyMetroidInCutscene[E].speed)).wrapping_add(32);
                if v2 as i32 - r22 as i32 & 0x8000 == 0 {
                    v2 = r22;
                }
                session_slot!(game, BabyMetroidInCutscene[E].speed = v2);
            } else {
                let mut v1: u16 =
                    (session_slot!(game, BabyMetroidInCutscene[E].speed)).wrapping_sub(32);
                if v1 as i32 - r22 as i32 & 0x8000 != 0 {
                    v1 = r22;
                }
                session_slot!(game, BabyMetroidInCutscene[E].speed = v1);
            }
        }
        let mut v3: u16 = 0;
        let use_target_angle: bool = if (r18 & 0x8000) != 0 {
            v3 = (session_slot!(game, BabyMetroidInCutscene[E].angle)).wrapping_add(r18);
            (v3 as i32 - r20 as i32 & 0x8000 != 0)
        } else {
            v3 = (session_slot!(game, BabyMetroidInCutscene[E].angle)).wrapping_add(r18);
            (v3 as i32 - r20 as i32 & 0x8000 == 0)
        };
        if use_target_angle {
            v3 = r20;
        }
        session_slot!(game, BabyMetroidInCutscene[E].angle = v3);
        r18 = v3.to_ne_bytes()[1] as u16;
        session_slot!(
            game,
            BabyMetroidInCutscene[E].x_velocity =
                Math_MultBySin(session_slot!(game, BabyMetroidInCutscene[E].speed), r18)
        );
        session_slot!(
            game,
            BabyMetroidInCutscene[E].y_velocity =
                Math_MultByCos(session_slot!(game, BabyMetroidInCutscene[E].speed), r18)
        );
    }
}
crate::dispatch::instruction_callbacks! { pub fn (_k: u16, _jp: EnemyScript) -> EnemyScript;
    BabyMetroid_Instr_1 [EnemyScript::jump(addr_kBabyMetroid_Ilist_CFA2 as u16)] { };
    BabyMetroid_Instr_2 [EnemyScript::jump(addr_kBabyMetroid_Ilist_CFB8 as u16)] { };
}
pub fn BabyMetroidInCutscene_Init(game: &mut impl Host) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    session_slot!(
        game,
        BabyMetroidInCutscene[E].base.properties =
            (session_slot!(game, BabyMetroidInCutscene[E].base.properties) as i32
                | (kEnemyProps_DisableSamusColl as i32 | kEnemyProps_BlockPlasmaBeam as i32))
                as u16
    );
    session_slot!(game, BabyMetroidInCutscene[E].base.palette_index = 3584);
    session_slot! { game, BabyMetroidInCutscene[E].base; current_instruction = addr_kBabyMetroid_Ilist_CFA2 as u16, instruction_timer = 1 }
    session_slot!(game, BabyMetroidInCutscene[E].cry_enabled = 1);
    session_slot!(game, BabyMetroidInCutscene[E].base.timer = 0);
    session_slot!(game, BabyMetroidInCutscene[E].palette_handler_delay = 10);
    session_slot! { game, BabyMetroidInCutscene[E].base; vram_tiles_index = 160, x_pos = 320, y_pos = 96 };
    session_slot! { game, BabyMetroidInCutscene[E]; x_velocity = 0, y_velocity = 0, health_palette_enabled = 0 }
    session_slot!(
        game,
        BabyMetroidInCutscene[E].function = function::DASH_ONTO_SCREEN
    );
    session_slot!(game, BabyMetroidInCutscene[E].function_timer = 248);
    session_slot!(
        game,
        BabyMetroidInCutscene[E].palette_function =
            function::HANDLE_BABY_METROID_IN_CUTSCENE_PALETTE_NORMAL
    );
    copy_rom_colors_to_palette(
        0x1e2,
        0xa9,
        (addr_kMotherBrainPalette_2 as i32 + 2) as u16,
        0xf,
        game,
    );
}
/// Main AI - baby Metroid in the cutscene: one lease for the frame; the
/// function, the health-based palette and the palette function run on the
/// game.
pub fn BabyMetroidInCutscene_Main(game: &mut impl Host) {
    let mut lease = game.lease();
    let lease = &mut lease;
    let E: i32 = word!(lease.cur_enemy_index) as i32;
    session_slot!(lease, BabyMetroidInCutscene[E].base.shake_timer = 0);
    let baby_function = session_slot!(lease, BabyMetroidInCutscene[E].function);
    let k = word!(lease.cur_enemy_index);
    run_cutscene_function((baby_function as u32) | (0xa90000u32), k, lease);
    MoveEnemyWithVelocity(lease);
    BabyMetroidInCutscene_Flashing(word!(lease.cur_enemy_index), lease);
    let k = word!(lease.cur_enemy_index);
    lease.foreign(|game| {
        BabyMetroidInCutscene_HandleHealthBasedPalette(k, game);
        let r18: u16 = session_slot!(
            game,
            BabyMetroidInCutscene[word!(game.cur_enemy_index) as i32].palette_function
        );
        CallBabyMetroidCutsceneFunc0F(((r18) as u32) | (0xa90000u32), game);
    });
}
pub fn BabyMetroidInCutscene_Touch(game: &mut impl Host) {
    let E: i32 = word!(game.cur_enemy_index) as i32;
    if session_slot!(game, BabyMetroidInCutscene[E].function) == function::LATCH_ONTO_SAMUS
        && BabyMetroid_AccelerateTowardsPoint(
            word!(game.cur_enemy_index),
            0x10,
            word!(game.samus_x_pos),
            (word!(game.samus_y_pos)).wrapping_sub(20),
            game,
        ) & 1
            != 0
    {
        session_slot! { game, BabyMetroidInCutscene[E]; x_velocity = 0, y_velocity = 0 }
        session_slot!(
            game,
            BabyMetroidInCutscene[E].function = function::HEAL_SAMUS_UP_TO_FULL_HEALTH
        );
    }
}
