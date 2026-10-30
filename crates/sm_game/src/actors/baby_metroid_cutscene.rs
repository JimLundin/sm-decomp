//! The baby Metroid in Mother Brain's cutscene: it dashes in, drains Mother Brain,
//! heals Samus, takes the last rainbow beam for her and dies, and Samus's
//! rainbow palette fades as the lights come back on.
//!
//! Was `BabyMetroidInCutscene_*` and `SamusRainbowPaletteFunc_*` in the original
//! decompilation (bank $A9:C710-$A9:CF00). Its own slot words are named after
//! the bank log's notes; slots 0 and $40 are Mother Brain's body and brain,
//! through `mother_brain`'s views. The bank's dispatch ids are unchanged; the
//! functions and their reads and writes keep the original order.
use crate::actors::baby_metroid_encounter::{
    accelerate_towards_point, gradually_accelerate_towards_point_10h,
    gradually_accelerate_towards_point_400h, run_acceleration_function, run_palette_function,
};
use crate::actors::host::abs16;
use crate::actors::layouts::part_view;
use crate::actors::mother_brain::{BRAIN, MotherBrainBody, MotherBrainBrain};
use crate::actors::mother_brain_fight::{
    BANK, BANK_BASE, Lease as FightLease, State, cgram, music, samus_command, sound,
};
use crate::actors::mother_brain_fight::{
    Fade, SHAKE_X_OFFSETS, SHAKE_Y_OFFSETS, bank_unreachable, make_walk_backwards,
    move_enemy_with_velocity, player_mother_brain, queue_mother_brain_sprite_tiles, queue_music,
    queue_sfx1_max6, queue_sfx2_max6, queue_sfx3_max3, queue_sfx3_max6, run_samus_command,
    set_enemy_instruction_list, spawn_room_projectile,
};
use crate::actors::session_slots::session_slot;
// The game's traits, for the calls made on it through `foreign`.
use crate::actors::mother_brain_fight::Host as _;

use crate::baby_metroid_tables;
use crate::constants::bank_86::*;
use crate::constants::bank_a9::*;
use crate::constants::enemy_properties::*;
use crate::enemy_projectiles::shared_pre::{x_velocity_at_angle, y_velocity_at_angle};
use crate::enemy_script::EnemyScript;
use crate::menus::menu_tilemap::native_record;
use crate::palette_copies::copy_rom_colors_to_palette;
use crate::ram::word;

/// What the cutscene reaches beyond the baby Metroid's encounter.
pub trait Host: crate::actors::baby_metroid_encounter::Host {
    /// Turn Mother Brain's room lights off for the baby Metroid's last charge.
    fn lights_off(&mut self);
    /// Heal Samus by the baby Metroid; false once she is full. Was
    /// `heal_by_baby_metroid`.
    fn heal_by_baby_metroid(&mut self) -> bool;
}

/// A Mother Brain lease whose game is this module's host.
pub trait Lease: FightLease<Game: Host> {}
impl<T: FightLease<Game: Host> + ?Sized> Lease for T {}

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
            var_18, var_19, var_1a, var_1b, var_1c, var_1d,
            var_1e, var_1f,
        ],
        pad2: [u8; 1984],
        words: [
            var_20, var_21, var_22, var_23, var_24, var_25,
            var_26, var_27, var_28, var_29, var_2a, var_2b,
            var_2c, var_2d, var_2e, var_2f, var_30, var_31,
            var_32, var_33, var_34, var_35, var_36, var_37,
            var_38, var_39, var_3a, var_3b, var_3c, var_3d,
            var_3e, var_3f,
        ],
    }
}

/// $A9:CDFC: the death explosions' offsets from the baby Metroid, (x, y) pairs.
const DEATH_EXPLOSION_OFFSETS: u32 = 0xa9_cdfc;

crate::dispatch::callback_dispatch! {
    pub fn run_cutscene_function(ea: u32, k: u16, game: &mut impl Lease) => ea;
    call (k, game); default { bank_unreachable(game); };
    @body checked;
    @functions [pub fn (k: u16, game: &mut impl Lease)];
    [11126764] curve_towards_brain [] {
        update_speed_and_angle(k, -384i32 as u16, -20480i32 as u16, 2560, game);
        let e: i32 = k as i32;
        session_slot!(game, BabyMetroidInCutscene[e].function_timer = session_slot!(game, BabyMetroidInCutscene[e].function_timer).wrapping_sub(1));
        if (session_slot!(game, BabyMetroidInCutscene[e].function_timer) & 0x8000) != 0 {
            session_slot!(game, BabyMetroidInCutscene[e].function = function::GET_RIGHT_UP_IN_MOTHER_BRAINS_FACE);
            session_slot!(game, BabyMetroidInCutscene[e].function_timer = 9);
        }
    };
    [11126801] get_in_mother_brains_face [] {
        update_speed_and_angle(k, -1536i32 as u16, -32256i32 as u16, 3584, game);
        let mut e: i32 = BRAIN;
        let rect = (session_slot!(game, MotherBrainBrain[e].base.x_pos), session_slot!(game, MotherBrainBrain[e].base.y_pos), 4, 4);
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 || {
            e = k as i32;
            let v3: bool = ((session_slot!(game, BabyMetroidInCutscene[e].function_timer)) as i16).wrapping_sub(1) < 0;
            session_slot!(game, BabyMetroidInCutscene[e].function_timer = session_slot!(game, BabyMetroidInCutscene[e].function_timer).wrapping_sub(1));
            v3 as i32 != 0
        } {
            session_slot!(game, BabyMetroidInCutscene[k as i32].function = function::LATCH_ONTO_MOTHER_BRAIN);
            player_mother_brain(1, game);
        }
    };
    [11126865] latch_onto_mother_brain [] {
        let e: i32 = BRAIN;
        let rect = (session_slot!(game, MotherBrainBrain[e].base.x_pos), ((session_slot!(game, MotherBrainBrain[e].base.y_pos)).wrapping_sub(24)), 8, 8);
        gradually_accelerate_towards_point_400h(k, 0, rect.0, rect.1, game);
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 {
            session_slot!(game, BabyMetroidInCutscene[k as i32].function = function::SET_MOTHER_BRAIN_TO_STUMBLE_BACK);
        }
    };
    [11126905] set_mother_brain_to_stumble_back [] {
        let e: i32 = 0;
        make_walk_backwards((session_slot!(game, MotherBrainBody[e].base.x_pos)).wrapping_sub(1), 2, game);
        session_slot!(game, BabyMetroidInCutscene[k as i32].function = function::ACTIVATE_RAINBOW_BEAM_AND_MOTHER_BRAIN_BODY);
        activate_rainbow_beam_and_body(k, game);
    };
    [11126921] activate_rainbow_beam_and_body [] {
        let e1: i32 = BRAIN;
        if accelerate_towards_point(
            k,
            0x200,
            session_slot!(game, MotherBrainBrain[e1].base.x_pos),
            (session_slot!(game, MotherBrainBrain[e1].base.y_pos)).wrapping_sub(24),
            game,
        ) & 1
            != 0
        {
            let e: i32 = k as i32;
            session_slot! { game, BabyMetroidInCutscene[e]; x_velocity = 0, y_velocity = 0 }
            session_slot!(game, BabyMetroidInCutscene[e].base.x_pos = session_slot!(game, MotherBrainBrain[e1].base.x_pos));
            session_slot!(game, BabyMetroidInCutscene[e].base.y_pos = (session_slot!(game, MotherBrainBrain[e1].base.y_pos)).wrapping_sub(24));
            set_enemy_instruction_list(k, ADDR_K_BABY_METROID_ILIST_CFB8 as u16, game);
            session_slot!(game, BabyMetroidInCutscene[e].function = function::WAIT_FOR_MOTHER_BRAIN_TO_TURN_TO_CORPSE);
            session_slot!(game, BabyMetroidInCutscene[e].palette_handler_delay = 1);
            let e0: i32 = 0;
            session_slot!(game, MotherBrainBody[e0].function = function::MOTHER_BRAIN_DRAINED_BY_BABY_METROID_TAKEN_ABACK);
            queue_sfx1_max6(sound::RAINBOW_BEAM, game);
            session_slot!(game, MotherBrainBody[e0].beam_sound_playing = 1);
        }
    };
    [11127061] stop_draining [] {
        let e: i32 = k as i32;
        let e1: i32 = BRAIN;
        session_slot!(game, BabyMetroidInCutscene[e].base.x_pos = session_slot!(game, MotherBrainBrain[e1].base.x_pos));
        session_slot!(game, BabyMetroidInCutscene[e].base.y_pos = (session_slot!(game, MotherBrainBrain[e1].base.y_pos)).wrapping_sub(24));
        session_slot!(game, BabyMetroidInCutscene[e].function_timer = session_slot!(game, BabyMetroidInCutscene[e].function_timer).wrapping_sub(1));
        if (session_slot!(game, BabyMetroidInCutscene[e].function_timer) & 0x8000) != 0 {
            set_enemy_instruction_list(k, ADDR_K_BABY_METROID_ILIST_CFA2 as u16, game);
            session_slot!(game, BabyMetroidInCutscene[e].palette_handler_delay = 10);
            session_slot!(game, BabyMetroidInCutscene[e].function = function::LET_GO_AND_SPAWN_DUST_CLOUDS);
            session_slot!(game, BabyMetroidInCutscene[e].function_timer = 32);
            session_slot! { game, BabyMetroidInCutscene[e]; x_velocity = 0, y_velocity = 0 }
        }
    };
    [11127129] move_up_to_ceiling [] {
        let rect = (session_slot!(game, MotherBrainBrain[BRAIN].base.x_pos), 0, 4, 4);
        gradually_accelerate_towards_point_400h(k, 0, rect.0, rect.1, game);
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 {
            player_mother_brain(4, game);
            let e: i32 = k as i32;
            session_slot!(game, BabyMetroidInCutscene[e].function = function::MOVE_TO_SAMUS);
            session_slot!(game, BabyMetroidInCutscene[e].movement = ADDR_STRU_A9CA24 as u16);
        }
    };
    [11127398] latch_onto_samus [] {
        gradually_accelerate_towards_point_400h(
            k,
            0,
            game.player_x(),
            (game.player_y()).wrapping_sub(20),
            game,
        );
    };
    [11127485] idle_until_out_of_health [] {
        handle_cry(game);
        let e: i32 = k as i32;
        let flash_timer: u16 = session_slot!(game, BabyMetroidInCutscene[e].flash_timer);
        if flash_timer != 0 {
            let v3: i32 = (flash_timer as i32 & 6) >> 1;
            session_slot!(game, BabyMetroidInCutscene[e].base.x_pos = (game.player_x() as i32 + 2 * SHAKE_X_OFFSETS[v3 as usize] as i32) as u16);
            session_slot!(game, BabyMetroidInCutscene[e].base.y_pos = (game.player_y() as i32
                + 2 * SHAKE_Y_OFFSETS[v3 as usize] as i32
                - 20i32) as u16);
        }
        if session_slot!(game, BabyMetroidInCutscene[e].base.health) == 0 {
            session_slot!(game, BabyMetroidInCutscene[e].base.health = 320);
            session_slot! { game, BabyMetroidInCutscene[e]; function = function::RELEASE_SAMUS, palette_handler_delay = 10, palette_animation = 0, palette_function = function::HANDLE_BABY_METROID_IN_CUTSCENE_PALETTE_LOW_HEALTH, cry_enabled = 1, health_palette_enabled = 0 };
        }
    };
    [11127571] release_samus [] {
        queue_sfx2_max6(sound::BABY_METROID_CRY, game);
        session_slot!(game, MotherBrainBrain[BRAIN].baby_metroid_released = 1);
        session_slot!(game, BabyMetroidInCutscene[k as i32].function = function::STARE_DOWN_MOTHER_BRAIN);
        session_slot!(game, MotherBrainBody[0i32].function = function::MOTHER_BRAIN_PREPARE_FOR_FINAL_BABY_METROID_ATTACK);
        stare_down_mother_brain(k, game);
    };
    [11127597] stare_down_mother_brain [] {
        let rect = (((game.player_x()).wrapping_sub(4)), 96, 4, 4);
        gradually_accelerate_towards_point_10h(k, 0, rect.0, rect.1, game);
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 {
            session_slot!(game, BabyMetroidInCutscene[k as i32].function = function::FLY_OFF_SCREEN);
        }
    };
    [11127638] fly_off_screen [] {
        let rect = (272, 64, 4, 4);
        gradually_accelerate_towards_point_10h(k, 0, rect.0, rect.1, game);
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 {
            session_slot!(game, BabyMetroidInCutscene[k as i32].function = function::MOVE_TO_FINAL_CHARGE_START_POSITION);
        }
    };
    [11127675] move_to_final_charge_start [] {
        let rect = (305, 160, 4, 4);
        gradually_accelerate_towards_point_10h(k, 0, rect.0, rect.1, game);
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 {
            let e: i32 = k as i32;
            session_slot!(game, BabyMetroidInCutscene[e].base.health = 79);
            session_slot!(game, MotherBrainBrain[BRAIN].baby_metroid_released = 0);
            session_slot!(game, MotherBrainBody[0i32].function = function::MOTHER_BRAIN_EXECUTE_FINAL_BABY_METROID_ATTACK);
            session_slot!(game, BabyMetroidInCutscene[e].function = function::INITIATE_FINAL_CHARGE);
        }
    };
    [11127731] initiate_final_charge [] {
        let rect = (290, 128, 4, 4);
        gradually_accelerate_towards_point_400h(k, 0xa, rect.0, rect.1, game);
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 {
            session_slot!(game, BabyMetroidInCutscene[k as i32].function = function::FINAL_CHARGE);
        }
    };
    [11127768] final_charge [] {
        let e1: i32 = BRAIN;
        gradually_accelerate_towards_point_400h(
            k,
            0xc,
            session_slot!(game, MotherBrainBrain[e1].base.x_pos),
            (session_slot!(game, MotherBrainBrain[e1].base.y_pos)).wrapping_sub(32),
            game,
        );
        let e: i32 = k as i32;
        if session_slot!(game, BabyMetroidInCutscene[e].base.health) == 0 {
            session_slot!(game, BabyMetroidInCutscene[e].palette_function = function::DEATH_SEQUENCE_PENDING);
            session_slot!(game, BabyMetroidInCutscene[e].base.vram_tiles_index = 4256);
            queue_sfx3_max6(sound::BABY_METROID_DIES, game);
            crate::probe::hit("baby-metroid.fatal-blow");
            game.foreign(|game| game.lights_off());
            set_enemy_instruction_list(k, ADDR_K_BABY_METROID_ILIST_CFCE as u16, game);
            session_slot! { game, BabyMetroidInCutscene[e]; x_velocity = 0, y_velocity = 0 }
            session_slot!(game, MotherBrainBody[0i32].function = ADDR_LOCRET_A9C18D as u16);
            queue_music(music::STOP, game);
            session_slot!(game, BabyMetroidInCutscene[e].function = function::BABY_METROID_TAKES_FATAL_BLOW);
            session_slot!(game, BabyMetroidInCutscene[e].function_timer = 16);
            session_slot!(game, BabyMetroidInCutscene[e].shake_origin_x = session_slot!(game, BabyMetroidInCutscene[e].base.x_pos));
            session_slot!(game, BabyMetroidInCutscene[e].shake_origin_y = session_slot!(game, BabyMetroidInCutscene[e].base.y_pos));
            take_fatal_blow(k, game);
        }
    };
    [11127870] take_fatal_blow [] {
        shake(k, game);
        let e: i32 = k as i32;
        session_slot!(game, BabyMetroidInCutscene[e].function_timer = session_slot!(game, BabyMetroidInCutscene[e].function_timer).wrapping_sub(1));
        if (session_slot!(game, BabyMetroidInCutscene[e].function_timer) & 0x8000) != 0 {
            session_slot!(game, BabyMetroidInCutscene[e].base.x_pos = session_slot!(game, BabyMetroidInCutscene[e].shake_origin_x));
            session_slot!(game, BabyMetroidInCutscene[e].base.y_pos = session_slot!(game, BabyMetroidInCutscene[e].shake_origin_y));
            session_slot!(game, BabyMetroidInCutscene[e].function = function::PLAY_SAMUS_THEME);
            session_slot!(game, BabyMetroidInCutscene[e].function_timer = 56);
            play_samus_theme(k, game);
        }
    };
    [11127904] play_samus_theme [] {
        let e: i32 = k as i32;
        session_slot!(game, BabyMetroidInCutscene[e].function_timer = session_slot!(game, BabyMetroidInCutscene[e].function_timer).wrapping_sub(1));
        if (session_slot!(game, BabyMetroidInCutscene[e].function_timer) & 0x8000) != 0 {
            queue_music(music::SAMUS_THEME_DATA, game);
            queue_music(music::SONG_0, game);
            session_slot!(game, BabyMetroidInCutscene[e].function = function::PREPARE_SAMUS_FOR_HYPER_BEAM_ACQUISITION);
            session_slot!(game, BabyMetroidInCutscene[e].function_timer = 12);
            prepare_samus_for_hyper_beam_acquisition(k, game);
        }
    };
    [11127961] death_sequence [] {
        handle_samus_rainbow_palette_animation(k, game);
        accelerate_downwards(k, game);
        if fade_to_black(k, game) & 1 != 0 {
            let e: i32 = k as i32;
            session_slot!(game, BabyMetroidInCutscene[e].base.properties = (session_slot!(game, BabyMetroidInCutscene[e].base.properties)) | ((K_ENEMY_PROPS_INVISIBLE) as u16));
            session_slot!(game, BabyMetroidInCutscene[e].function = function::UNLOAD_BABY_METROID_TILES);
            session_slot!(game, BabyMetroidInCutscene[e].function_timer = 128);
        } else {
            handle_death_explosions(k, game);
            handle_enemy_blinking(k, game);
        };
    };
    [11128000] unload_tiles [] {
        handle_samus_rainbow_palette_animation(k, game);
        let e: i32 = k as i32;
        session_slot!(game, BabyMetroidInCutscene[e].function_timer = session_slot!(game, BabyMetroidInCutscene[e].function_timer).wrapping_sub(1));
        let v2: bool = (session_slot!(game, BabyMetroidInCutscene[e].function_timer) & 0x8000) != 0;
        if v2 && queue_mother_brain_sprite_tiles(BANK, ADDR_STRU_A98FC7 as u16, game) != 0 {
            session_slot!(game, BabyMetroidInCutscene[e].function = function::LET_SAMUS_RAINBOW_SOME_MORE);
            session_slot!(game, BabyMetroidInCutscene[e].function_timer = 176);
            let_samus_rainbow_some_more(k, game);
        }
    };
    [11128048] finish_cutscene [] {
        let e: i32 = 0;
        let v2: u16 = (session_slot!(game, MotherBrainBody[e].grey_transition)).wrapping_add(1);
        session_slot!(game, MotherBrainBody[e].grey_transition = v2);
        if game.foreign(|game| game.fade(Fade::LightsOn, (v2).wrapping_sub(1))) & 1 != 0 {
            session_slot!(game, MotherBrainBody[e].function = function::MOTHER_BRAIN_PHASE_3_RECOVER_FROM_CUTSCENE_MAKE_SOME_DISTANCE);
            run_samus_command(samus_command::DISABLE_RAINBOW_AND_STAND_UP, game);
            player_mother_brain(3, game);
            let fresh12: i32 = k as i32;
            session_slot!(game, BabyMetroidInCutscene[fresh12].base.properties, |old| old | (K_ENEMY_PROPS_DELETED as u16));
            session_slot!(game, MotherBrainBrain[BRAIN].baby_metroid = 0);
        }
    };
    @functions [pub fn (k: u16, game: &mut impl Lease)];
    @prefix { let e: i32 = k as i32; };
    [11127010] wait_for_brain_to_turn_to_corpse [] {
        let e1: i32 = BRAIN;
        let v2: i32 = (session_slot!(game, BabyMetroidInCutscene[e].base.frame_counter) as i32 & 6) >> 1;
        session_slot!(game, BabyMetroidInCutscene[e].base.x_pos = (session_slot!(game, MotherBrainBrain[e1].base.x_pos)).wrapping_add((SHAKE_X_OFFSETS[v2 as usize]) as u16));
        session_slot!(game, BabyMetroidInCutscene[e].base.y_pos = (session_slot!(game, MotherBrainBrain[e1].base.y_pos) as i32 + SHAKE_Y_OFFSETS[v2 as usize] as i32 - 24i32) as u16);
        if session_slot!(game, MotherBrainBody[0i32].corpse_state) != 0 {
            session_slot!(game, BabyMetroidInCutscene[e].function = function::STOP_DRAINING);
            session_slot!(game, BabyMetroidInCutscene[e].function_timer = 64);
        }
    };
    [11127115] let_go_and_spawn_dust_clouds [] {
        session_slot!(game, BabyMetroidInCutscene[e].function_timer = session_slot!(game, BabyMetroidInCutscene[e].function_timer).wrapping_sub(1));
        if (session_slot!(game, BabyMetroidInCutscene[e].function_timer) & 0x8000) != 0 {
            spawn_three_dust_clouds_on_brain(game);
            session_slot!(game, BabyMetroidInCutscene[e].function = ADDR_LOC_A9C959 as u16);
        }
        move_up_to_ceiling(k, game);
    };
    [11127235] move_to_samus [] {
        session_slot!(game, BabyMetroidInCutscene[e].cry_enabled = 0);
        session_slot!(game, BabyMetroidInCutscene[e].health_palette_enabled = 1);
        if (word!(game.RANDOM_NUMBER) & 0xfff) >= 0xfa0 {
            queue_sfx2_max6(sound::BABY_METROID_REMORSE, game);
        }
        let movement: u16 = session_slot!(game, BabyMetroidInCutscene[e].movement);
        let v3 = crate::cartridge::native_record(BANK, movement, game);
        let rect = (v3.word(0), v3.word(2), 4, 4);
        run_acceleration_function(
            (v3.word(6) as u32) | (BANK_BASE),
            k,
            v3.word(4),
            rect.0,
            rect.1,
            game,
        );
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 {
            let v4: i16 = crate::cartridge::native_record(BANK, movement, game).word(8) as i16;
            if v4 < 0 {
                session_slot!(game, BabyMetroidInCutscene[e].function = v4 as u16);
            } else {
                session_slot!(game, BabyMetroidInCutscene[e].movement = (session_slot!(game, BabyMetroidInCutscene[e].movement)).wrapping_add(8));
            }
        }
    };
    [11127418] heal_samus_to_full_health [] {
        session_slot!(game, BabyMetroidInCutscene[e].cry_enabled = 0);
        handle_cry(game);
        let v2: i32 = (session_slot!(game, BabyMetroidInCutscene[e].base.frame_counter) as i32 & 6) >> 1;
        session_slot!(game, BabyMetroidInCutscene[e].base.x_pos = (game.player_x()).wrapping_add((SHAKE_X_OFFSETS[v2 as usize]) as u16));
        session_slot!(game, BabyMetroidInCutscene[e].base.y_pos = (game.player_y() as i32 + SHAKE_Y_OFFSETS[v2 as usize] as i32 - 20i32) as u16);
        if !game.foreign(|game| game.heal_by_baby_metroid()) {
            word!(game.SAMUS_RESERVE_HEALTH = word!(game.SAMUS_MAX_RESERVE_HEALTH));
            session_slot!(game, BabyMetroidInCutscene[e].function = function::IDLE_UNTIL_RUN_OUT_OF_HEALTH);
            session_slot!(game, BabyMetroidInCutscene[e].palette_function = function::PALETTE_NONE);
        }
    };
    handle_death_explosions [] {
        let v2: i16 = ((session_slot!(game, BabyMetroidInCutscene[e].death_explosion_timer)) as i16).wrapping_sub(1);
        if v2 < 0 {
            session_slot!(game, BabyMetroidInCutscene[e].death_explosion_timer = 4);
            let mut v3: u16 = (session_slot!(game, BabyMetroidInCutscene[e].death_explosion_index)).wrapping_add(1);
            if (session_slot!(game, BabyMetroidInCutscene[e].death_explosion_index) as i32 - 9i32) & 0x8000 == 0 {
                v3 = 0;
            }
            session_slot!(game, BabyMetroidInCutscene[e].death_explosion_index = v3);
            let v4: i32 = ((4u16).wrapping_mul(v3)) as i32 >> 1;
            let offsets = native_record(DEATH_EXPLOSION_OFFSETS, game);
            let x = (session_slot!(game, BabyMetroidInCutscene[e].base.x_pos))
                .wrapping_add(offsets.table_word(v4 as isize));
            let y = (session_slot!(game, BabyMetroidInCutscene[e].base.y_pos))
                .wrapping_add(offsets.table_word((v4 + 1) as isize));
            game.foreign(|game| crate::actors::host::BossEffects::set_projectile_spawn_point(game, x, y));
            spawn_room_projectile(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 3, game);
            queue_sfx3_max3(sound::EXPLODING, game);
        } else {
            session_slot!(game, BabyMetroidInCutscene[e].death_explosion_timer = v2 as u16);
        };
    };
    handle_health_based_palette [] {
        if session_slot!(game, BabyMetroidInCutscene[e].health_palette_enabled) != 0 {
            let health: u16 = session_slot!(game, BabyMetroidInCutscene[e].base.health);
            if (health as i32 - 2560i32) & 0x8000 != 0 {
                let mut v3: u16 = 2;
                if (health as i32 - 2240i32) & 0x8000 != 0 {
                    v3 = 4;
                    if (health as i32 - 1920i32) & 0x8000 != 0 {
                        v3 = 6;
                        if (health as i32 - 1600i32) & 0x8000 != 0 {
                            v3 = 8;
                            if (health as i32 - 1280i32) & 0x8000 != 0 {
                                v3 = 10;
                                if (health as i32 - 960i32) & 0x8000 != 0 {
                                    v3 = 12;
                                    if (health as i32 - 640i32) & 0x8000 != 0 {
                                        v3 = 14;
                                    }
                                }
                            }
                        }
                    }
                }
                let v5: u16 = v3;
                let v4: u16 = baby_metroid_tables::shell_palette(game, (v3 >> 1) as isize).colors;
                copy_rom_colors_to_palette(cgram::SPRITE_PALETTE_7, 0xad, v4, 4, game);
                copy_rom_colors_to_palette(cgram::SPRITE_PALETTE_7_COLOUR_10, 0xad, (v4).wrapping_add(8), 5, game);
                copy_rom_colors_to_palette(
                    cgram::SPRITE_PALETTE_7_COLOUR_5,
                    0xad,
                    baby_metroid_tables::innards_palette(game, (v5 >> 1) as isize).colors,
                    5,
                    game,
                );
            }
        }
    };
    shake [] {
        session_slot!(game, BabyMetroidInCutscene[e].y_velocity = session_slot!(game, BabyMetroidInCutscene[e].y_velocity).wrapping_sub(1));
        let v2: i32 = (session_slot!(game, BabyMetroidInCutscene[e].base.frame_counter) as i32 & 6) >> 1;
        session_slot!(game, BabyMetroidInCutscene[e].base.x_pos = (session_slot!(game, BabyMetroidInCutscene[e].shake_origin_x)).wrapping_add((SHAKE_X_OFFSETS[v2 as usize]) as u16));
        session_slot!(game, BabyMetroidInCutscene[e].base.y_pos = (session_slot!(game, BabyMetroidInCutscene[e].shake_origin_y)).wrapping_add((SHAKE_Y_OFFSETS[v2 as usize]) as u16));
    };
    @countdown [pub fn (k: u16, game: &mut impl Lease)];
    before {
        let e: i32 = k as i32;
        session_slot!(game, BabyMetroidInCutscene[e].function_timer = session_slot!(game, BabyMetroidInCutscene[e].function_timer).wrapping_sub(1));
    }
    when (session_slot!(game, BabyMetroidInCutscene[e].function_timer) & 0x8000) != 0;
    [11126732] dash_onto_screen {
        session_slot!(game, BabyMetroidInCutscene[e].angle = -10240i32 as u16);
        session_slot!(game, BabyMetroidInCutscene[e].speed = 2560);
        session_slot!(game, BabyMetroidInCutscene[e].function = function::CURVE_TOWARDS_MOTHER_BRAIN_BRAIN);
        session_slot!(game, BabyMetroidInCutscene[e].function_timer = 10);
        curve_towards_brain(k, game);
    };
    [11127935] prepare_samus_for_hyper_beam_acquisition {
        run_samus_command(samus_command::FREEZE_DRAINED_ANIMATION, game);
        session_slot!(game, MotherBrainBrain[BRAIN].samus_rainbow_function = function::RAINBOW_ACTIVATE_RAINBOW_WHEN_ENEMY_IS_LOW_ENOUGH);
        session_slot!(game, BabyMetroidInCutscene[e].function = function::DEATH_SEQUENCE);
    };
    [11128030] let_samus_rainbow_some_more {
        session_slot!(game, BabyMetroidInCutscene[e].function = function::FINISH_CUTSCENE);
        session_slot!(game, MotherBrainBody[0i32].grey_transition = 0);
        finish_cutscene(k, game);
    };
    @external {
        11125133 => [()];
    }
}
pub fn flashing(k: u16, game: &mut impl State) {
    let mut v1: i16 = 3584;
    let e: i32 = k as i32;
    let flash_timer: u16 = session_slot!(game, BabyMetroidInCutscene[e].flash_timer);
    if flash_timer != 0 {
        let v4: u16 = (flash_timer).wrapping_sub(1);
        session_slot!(game, BabyMetroidInCutscene[e].flash_timer = v4);
        if (v4 & 2) != 0 {
            v1 = 0;
        }
    }
    session_slot!(
        game,
        BabyMetroidInCutscene[e].base.palette_index = v1 as u16
    );
}
pub fn handle_cry(game: &mut impl State) {
    let e: i32 = 0;
    if session_slot!(game, MotherBrainBody[e].baby_metroid_cry) != 0 {
        session_slot!(game, MotherBrainBody[e].baby_metroid_cry = 0);
        queue_sfx2_max6(sound::BABY_METROID_CRY, game);
    }
}
pub fn spawn_dust_cloud_at_brain_offset(xd: u16, yd: u16, game: &mut impl Lease) {
    let e: i32 = BRAIN;
    {
        let (x, y) = (
            ((session_slot!(game, MotherBrainBrain[e].base.x_pos)).wrapping_add(xd)),
            ((session_slot!(game, MotherBrainBrain[e].base.y_pos)).wrapping_add(yd)),
        );
        game.foreign(|game| {
            crate::actors::host::BossEffects::set_projectile_spawn_point(game, x, y)
        });
    }
    spawn_room_projectile(ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16, 9, game);
}
pub fn spawn_three_dust_clouds_on_brain(game: &mut impl Lease) {
    spawn_dust_cloud_at_brain_offset(-16i32 as u16, -8i32 as u16, game);
    spawn_dust_cloud_at_brain_offset(0, -16i32 as u16, game);
    spawn_dust_cloud_at_brain_offset(16, -8i32 as u16, game);
}
crate::dispatch::callback_dispatch! {
    pub fn run_rainbow_palette_function(ea: u32, k: u16, game: &mut impl Lease) => ea;
    call (k, game); default { bank_unreachable(game); };
    @body checked;
    @functions [pub fn (k: u16, game: &mut impl Lease)];
    [11128112] rainbow_palette_activate_when_enemy_low [] {
        if ((session_slot!(game, BabyMetroidInCutscene[k as i32].base.y_pos) as i32 + 16
            - game.player_y() as i32) as i16)
            >= 0
        {
            run_samus_command(samus_command::ENABLE_RAINBOW, game);
            session_slot!(game, MotherBrainBrain[BRAIN].samus_rainbow_function = function::RAINBOW_GRADUALLY_SLOW_ANIMATION_DOWN);
        }
    };
    @external {
        11128139 => [rainbow_palette_gradually_slow_down(game)];
    }
}
pub fn handle_samus_rainbow_palette_animation(k: u16, game: &mut impl Lease) {
    let r18: u16 = session_slot!(game, MotherBrainBrain[BRAIN].samus_rainbow_function);
    run_rainbow_palette_function(((r18) as u32) | (BANK_BASE), k, game);
}
pub fn rainbow_palette_gradually_slow_down(game: &mut impl State) {
    let e: i32 = BRAIN;
    let rainbow_counter: u16 = session_slot!(game, MotherBrainBrain[e].samus_rainbow_counter);
    session_slot!(
        game,
        MotherBrainBrain[e].samus_rainbow_counter = (rainbow_counter).wrapping_add(768)
    );
    if rainbow_counter >= 0xfd00 {
        let mut v2: u16 = (word!(game.SPECIAL_SAMUS_PALETTE_FRAME)).wrapping_add(1);
        if (word!(game.SPECIAL_SAMUS_PALETTE_FRAME) as i32 - 9i32) & 0x8000 == 0 {
            v2 = 10;
        }
        word!(game.SPECIAL_SAMUS_PALETTE_FRAME = v2);
    }
}
pub fn fade_to_black(k: u16, game: &mut impl State) -> u8 {
    {
        let e: i32 = k as i32;
        if (session_slot!(game, BabyMetroidInCutscene[e].base.y_pos) as i32 - 128i32) & 0x8000 != 0
        {
            return 0;
        }
        let v2: i16 =
            ((session_slot!(game, BabyMetroidInCutscene[e].fade_timer)) as i16).wrapping_sub(1);
        if v2 >= 0 {
            session_slot!(game, BabyMetroidInCutscene[e].fade_timer = v2 as u16);
            return 0;
        }
        session_slot!(game, BabyMetroidInCutscene[e].fade_timer = 8);
        let v4: u16 = (session_slot!(game, BabyMetroidInCutscene[e].fade_index)).wrapping_add(1);
        if (session_slot!(game, BabyMetroidInCutscene[e].fade_index) as i32 - 6i32) & 0x8000 == 0 {
            return 1;
        }
        session_slot!(game, BabyMetroidInCutscene[e].fade_index = v4);
        copy_rom_colors_to_palette(
            cgram::SPRITE_PALETTE_7,
            0xad,
            baby_metroid_tables::fade_palette(game, v4 as isize).colors,
            0xe,
            game,
        );
        0
    }
}
pub fn handle_enemy_blinking(k: u16, game: &mut impl State) {
    let e: i32 = k as i32;
    if (session_slot!(game, BabyMetroidInCutscene[e].base.frame_counter) & 1) != 0 {
        session_slot!(
            game,
            BabyMetroidInCutscene[e].base.properties =
                (session_slot!(game, BabyMetroidInCutscene[e].base.properties) as i32
                    & !(K_ENEMY_PROPS_INVISIBLE as i32)) as u16
        );
    } else {
        session_slot!(
            game,
            BabyMetroidInCutscene[e].base.properties =
                (session_slot!(game, BabyMetroidInCutscene[e].base.properties))
                    | ((K_ENEMY_PROPS_INVISIBLE) as u16)
        );
    };
}
pub fn accelerate_downwards(k: u16, game: &mut impl State) {
    let e: i32 = k as i32;
    let mut v2: i16 =
        ((abs16(session_slot!(game, BabyMetroidInCutscene[e].x_velocity))) as i16).wrapping_sub(32);
    if v2 < 0 {
        v2 = 0;
    }
    session_slot!(
        game,
        BabyMetroidInCutscene[e].x_velocity =
            (if session_slot!(game, BabyMetroidInCutscene[e].x_velocity) & 0x8000 != 0 {
                -(v2 as i32)
            } else {
                v2 as i32
            }) as u16
    );
    session_slot!(
        game,
        BabyMetroidInCutscene[e].y_velocity =
            (session_slot!(game, BabyMetroidInCutscene[e].y_velocity)).wrapping_add(2)
    );
}
pub fn update_speed_and_angle(k: u16, mut r18: u16, r20: u16, r22: u16, game: &mut impl State) {
    {
        let e: i32 = k as i32;
        // Steer the speed 20h a frame towards `r22` and the angle by `r18`
        // up to `r20`, then set the velocities along the angle.
        if r22 != session_slot!(game, BabyMetroidInCutscene[e].speed) {
            if ((r22) as i16)
                .wrapping_sub((session_slot!(game, BabyMetroidInCutscene[e].speed)) as i16)
                >= 0
            {
                let mut v2: u16 =
                    (session_slot!(game, BabyMetroidInCutscene[e].speed)).wrapping_add(32);
                if (v2 as i32 - r22 as i32) & 0x8000 == 0 {
                    v2 = r22;
                }
                session_slot!(game, BabyMetroidInCutscene[e].speed = v2);
            } else {
                let mut v1: u16 =
                    (session_slot!(game, BabyMetroidInCutscene[e].speed)).wrapping_sub(32);
                if (v1 as i32 - r22 as i32) & 0x8000 != 0 {
                    v1 = r22;
                }
                session_slot!(game, BabyMetroidInCutscene[e].speed = v1);
            }
        }
        let mut v3: u16;
        let use_target_angle: bool = if (r18 & 0x8000) != 0 {
            v3 = (session_slot!(game, BabyMetroidInCutscene[e].angle)).wrapping_add(r18);
            (v3 as i32 - r20 as i32) & 0x8000 != 0
        } else {
            v3 = (session_slot!(game, BabyMetroidInCutscene[e].angle)).wrapping_add(r18);
            (v3 as i32 - r20 as i32) & 0x8000 == 0
        };
        if use_target_angle {
            v3 = r20;
        }
        session_slot!(game, BabyMetroidInCutscene[e].angle = v3);
        r18 = v3.to_ne_bytes()[1] as u16;
        session_slot!(
            game,
            BabyMetroidInCutscene[e].x_velocity =
                x_velocity_at_angle(session_slot!(game, BabyMetroidInCutscene[e].speed), r18)
        );
        session_slot!(
            game,
            BabyMetroidInCutscene[e].y_velocity =
                y_velocity_at_angle(session_slot!(game, BabyMetroidInCutscene[e].speed), r18)
        );
    }
}
crate::dispatch::instruction_callbacks! { pub fn (_k: u16, _jp: EnemyScript) -> EnemyScript;
    go_to_initial [EnemyScript::jump(ADDR_K_BABY_METROID_ILIST_CFA2 as u16)] { };
    go_to_draining_mother_brain [EnemyScript::jump(ADDR_K_BABY_METROID_ILIST_CFB8 as u16)] { };
}
pub fn initialize(game: &mut impl State) {
    let e: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
    session_slot!(
        game,
        BabyMetroidInCutscene[e].base.properties =
            (session_slot!(game, BabyMetroidInCutscene[e].base.properties) as i32
                | (K_ENEMY_PROPS_DISABLE_SAMUS_COLL as i32
                    | K_ENEMY_PROPS_BLOCK_PLASMA_BEAM as i32)) as u16
    );
    session_slot!(game, BabyMetroidInCutscene[e].base.palette_index = 3584);
    session_slot! { game, BabyMetroidInCutscene[e].base; current_instruction = ADDR_K_BABY_METROID_ILIST_CFA2 as u16, instruction_timer = 1 }
    session_slot!(game, BabyMetroidInCutscene[e].cry_enabled = 1);
    session_slot!(game, BabyMetroidInCutscene[e].base.timer = 0);
    session_slot!(game, BabyMetroidInCutscene[e].palette_handler_delay = 10);
    session_slot! { game, BabyMetroidInCutscene[e].base; vram_tiles_index = 160, x_pos = 320, y_pos = 96 };
    session_slot! { game, BabyMetroidInCutscene[e]; x_velocity = 0, y_velocity = 0, health_palette_enabled = 0 }
    session_slot!(
        game,
        BabyMetroidInCutscene[e].function = function::DASH_ONTO_SCREEN
    );
    session_slot!(game, BabyMetroidInCutscene[e].function_timer = 248);
    session_slot!(
        game,
        BabyMetroidInCutscene[e].palette_function =
            function::HANDLE_BABY_METROID_IN_CUTSCENE_PALETTE_NORMAL
    );
    copy_rom_colors_to_palette(
        cgram::SPRITE_PALETTE_7,
        BANK,
        (ADDR_K_MOTHER_BRAIN_PALETTE_2 as i32 + 2) as u16,
        0xf,
        game,
    );
}
/// Main AI - baby Metroid in the cutscene: the frame runs on one lease.
pub fn main_ai(game: &mut impl Host) {
    cutscene_frame(&mut game.lease());
}
fn cutscene_frame(game: &mut impl Lease) {
    let e: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
    session_slot!(game, BabyMetroidInCutscene[e].base.shake_timer = 0);
    run_cutscene_function(
        ((session_slot!(game, BabyMetroidInCutscene[e].function)) as u32) | (BANK_BASE),
        word!(game.CUR_ENEMY_INDEX),
        game,
    );
    move_enemy_with_velocity(game);
    flashing(word!(game.CUR_ENEMY_INDEX), game);
    handle_health_based_palette(word!(game.CUR_ENEMY_INDEX), game);
    let r18: u16 = session_slot!(
        game,
        BabyMetroidInCutscene[word!(game.CUR_ENEMY_INDEX) as i32].palette_function
    );
    run_palette_function(((r18) as u32) | (BANK_BASE), game);
}
pub fn touch(game: &mut impl State) {
    let e: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
    if session_slot!(game, BabyMetroidInCutscene[e].function) == function::LATCH_ONTO_SAMUS
        && accelerate_towards_point(
            word!(game.CUR_ENEMY_INDEX),
            0x10,
            game.player_x(),
            (game.player_y()).wrapping_sub(20),
            game,
        ) & 1
            != 0
    {
        session_slot! { game, BabyMetroidInCutscene[e]; x_velocity = 0, y_velocity = 0 }
        session_slot!(
            game,
            BabyMetroidInCutscene[e].function = function::HEAL_SAMUS_UP_TO_FULL_HEALTH
        );
    }
}
