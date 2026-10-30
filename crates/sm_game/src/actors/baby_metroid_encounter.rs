//! The baby Metroid's encounter in Tourian: it drains the sidehopper, latches onto
//! Samus and drains her until she is nearly dead, then lets go and leaves.
//!
//! Was `BabyMetroid_*` in the original decompilation (bank $A9:EF37-$A9:F8A5).
//! Slot words are named through `baby_metroid`'s view after the bank log's
//! baby Metroid notes. The bank's dispatch ids are unchanged; the functions and
//! their reads and writes keep the original order. Mother Brain's shared
//! helpers come from `mother_brain_fight`.
use crate::actors::baby_metroid::BabyMetroid;
use crate::actors::corpses::Corpse;
use crate::actors::host::abs16;
use crate::actors::mother_brain_fight::{
    BANK, BANK_BASE, Lease as FightLease, State, cgram, music, samus_command, sound,
};
use crate::actors::mother_brain_fight::{
    SHAKE_X_OFFSETS, SHAKE_Y_OFFSETS, bank_unreachable, move_enemy_with_velocity,
    player_mother_brain, queue_music, queue_sfx2_max6, run_samus_command,
    set_enemy_instruction_list, spawn_plm,
};
use crate::actors::ridley::{HomingReversal, homing_velocity};
use crate::actors::session_slots::session_slot;
use crate::runtime_access::SessionRam;
// The game's traits, for the calls made on it through `foreign`.

use crate::constants::bank_a9::*;
use crate::constants::enemy_properties::*;
use crate::enemy_projectiles::shared_pre::{x_velocity_at_angle, y_velocity_at_angle};
use crate::enemy_script::EnemyScript;
use crate::palette_copies::{copy_rom_colors_to_palette, copy_rom_colors_to_target_palette};
use crate::ram::word;
use crate::word_transfers;

/// What the baby Metroid reaches beyond Mother Brain's fight and its own helpers.
/// The bank $A9 functions these slots run, named after the bank log's headings.
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

pub trait Host:
    crate::actors::mother_brain_fight::Host + crate::actors::baby_metroid::Host
{
    /// Drain Samus's health while the baby Metroid is latched on. Was
    /// `drain_health_by_baby_metroid`.
    fn drain_health_by_baby_metroid(&mut self);
    /// Mark a homing step for the verifier's probes.
    fn homing_probe(&mut self);
}

/// A Mother Brain lease whose game is this module's host.
pub trait Lease: FightLease<Game: Host> {}
impl<T: FightLease<Game: Host> + ?Sized> Lease for T {}

pub static GRADUAL_ACCELERATION_DIVISORS: [i8; 16] = [
    0x10, 0xf, 0xe, 0xd, 0xc, 0xb, 0xa, 9, 8, 7, 6, 5, 4, 3, 2, 1,
];
crate::dispatch::callback_dispatch! {
    pub fn run_palette_function(ea: u32, game: &mut impl Lease) => ea;
    call (game); default { bank_unreachable(game); };
    @body checked;
    @forward [pub fn (game: &mut impl State)];
    handle_normal_palette => handle_palette(ADDR_WORD_A9F6D1 as u16, 330, game);
    [11138691]
    handle_cutscene_palette => handle_palette(ADDR_WORD_A9F6D1 as u16, 490, game);
    [11138703]
    handle_cutscene_palette_low_health => handle_palette(ADDR_WORD_A9F711 as u16, 490, game);
    @external {
        11127484 => [()]; 11127960 => [()];
    }
}
crate::dispatch::callback_dispatch! {
    pub fn run_acceleration_function(ea: u32, k: u16, j: u16, r18: u16, r20: u16, game: &mut impl Lease) => ea;
    call (k, j, r18, r20, game); default { bank_unreachable(game); };
    @body checked;
    @forward [pub fn ( k: u16, j: u16, r18: u16, r20: u16, game: &mut impl Lease, )];
    gradually_accelerate_towards_point_400h => gradually_accelerate_towards_point(k, j, r18, r20, 1024, game);
    gradually_accelerate_towards_point_4h => gradually_accelerate_towards_point(k, j, r18, r20, 4, game);
    [11138143]
    gradually_accelerate_towards_point_8h => gradually_accelerate_towards_point(k, j, r18, r20, 8, game);
    [11138150]
    gradually_accelerate_towards_point_10h => gradually_accelerate_towards_point(k, j, r18, r20, 16, game);
}
crate::dispatch::callback_dispatch! {
    pub fn run_baby_metroid_function(ea: u32, k: u16, game: &mut impl Lease) => ea;
    call (k, game); default { bank_unreachable(game); };
    @body checked;
    @functions [pub fn (k: u16, game: &mut impl Lease)];
    [11137097] rush_to_middle_of_room [] {
        let rect = (584, 74, 1, 1);
        gradually_accelerate_towards_point_400h(k, 0xf, rect.0, rect.1, game);
        if crate::actors::baby_metroid::separated_from_rect(k, rect, game) == 0 {
            session_slot!(game, BabyMetroid[k as i32].function = function::RUSH_TO_SIDEHOPPER);
        }
    };
    [11137133] rush_to_sidehopper [] {
        let e: i32 = k as i32 + 64;
        gradually_accelerate_towards_point_400h(
            k,
            0xf,
            session_slot!(game, BabyMetroid[e].base.x_pos),
            (session_slot!(game, BabyMetroid[e].base.y_pos)).wrapping_sub(32),
            game,
        );
        if crate::actors::baby_metroid::separated_from_enemy(k, (k).wrapping_add(64), game) & 1 == 0 {
            session_slot!(game, BabyMetroid[k as i32].function = function::LATCH_ONTO_SIDEHOPPER);
        }
    };
    [11137172] latch_onto_sidehopper [] {
        let e1: i32 = k as i32 + 64;
        if accelerate_towards_point(
            k,
            0x200,
            session_slot!(game, BabyMetroid[e1].base.x_pos),
            (session_slot!(game, BabyMetroid[e1].base.y_pos)).wrapping_sub(32),
            game,
        ) & 1
            != 0
        {
            let e: i32 = k as i32;
            session_slot! { game, BabyMetroid[e]; x_velocity = 0, y_velocity = 0 }
            session_slot!(game, BabyMetroid[e].base.x_pos = session_slot!(game, BabyMetroid[e1].base.x_pos));
            session_slot!(game, BabyMetroid[e].base.y_pos = (session_slot!(game, BabyMetroid[e1].base.y_pos)).wrapping_sub(32));
            session_slot! { game, BabyMetroid[e].base; current_instruction = ADDR_K_BABY_METROID_ILIST_F924 as u16, instruction_timer = 1 }
            session_slot!(game, BabyMetroid[e].base.timer = 0);
            session_slot! { game, BabyMetroid[e]; function = function::DRAINING_SIDEHOPPER, palette_handler_delay = 1 }
            session_slot! { game, BabyMetroid[e]; contact_enabled = 0, function_timer = 320 }
        }
    };
    [11137317] make_sidehopper_rottable [] {
        session_slot!(game, Corpse[0x40i32].drained_palette_index = 1);
        let e: i32 = k as i32;
        session_slot! { game, BabyMetroid[e]; function = function::MOVE_UP_AND_UNLOCK_CAMERA, function_timer = 192 }
        move_up_and_unlock_camera(k, game);
    };
    [11137530] latch_onto_samus [] {
        gradually_accelerate_towards_point_400h(
            k,
            0xf,
            game.player_x(),
            (game.player_y()).wrapping_sub(32),
            game,
        );
    };
    [11137550] start_draining_samus [] {
        run_samus_command(samus_command::ENABLE_BLUE_FLASHING, game);
        session_slot!(game, BabyMetroid[0i32].function = function::DRAINING_SAMUS);
        draining_samus(k, game);
    };
    @functions [pub fn (_k: u16, game: &mut impl Lease)];
    [11137728] back_off_guiltily [] {
        gradually_accelerate_towards_point_400h(
            word!(game.CUR_ENEMY_INDEX),
            0,
            game.player_x(),
            104,
            game,
        );
        let e: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
        session_slot!(game, BabyMetroid[e].function_timer = session_slot!(game, BabyMetroid[e].function_timer).wrapping_sub(1));
        if (session_slot!(game, BabyMetroid[e].function_timer) & 0x8000) != 0 {
            queue_sfx2_max6(sound::BABY_METROID_GUILTY, game);
            session_slot! { game, BabyMetroid[e]; function = function::GO_LEFT_GUILTILY, function_timer = 88 }
            session_slot! { game, BabyMetroid[e].base; current_instruction = ADDR_K_BABY_METROID_ILIST_F924 as u16, instruction_timer = 1 }
            session_slot!(game, BabyMetroid[e].base.timer = 0);
            go_left_guiltily(game);
        }
    };
    @functions [pub fn (k: u16, game: &mut impl Lease)];
    [11137888] flee_remorsefully [] {
        queue_sfx2_max6(sound::BABY_METROID_REMORSE, game);
        session_slot!(game, BabyMetroid[k as i32].function = function::FLEEING);
        fleeing(game);
    };
    [11137982] remorse [] {
        if gradually_accelerate_towards_samus(k, game) & 1 != 0 {
            make_flee(k, game);
        }
    };
    @functions [pub fn (k: u16, game: &mut impl Lease)];
    @prefix { let e: i32 = k as i32; };
    [11136991] disappeared [] {
        session_slot! { game, BabyMetroid[e]; x_velocity = 0, y_velocity = 0 }
    };
    [11137067] let_sidehopper_live_for_a_bit [] {
        session_slot! { game, BabyMetroid[e]; function = function::QUEUE_BATTLE_MUSIC, function_timer = 464 }
        queue_battle_music(k, game);
    };
    [11137254] draining_sidehopper [] {
        let v2: i32 = (session_slot!(game, BabyMetroid[e].base.frame_counter) as i32 & 6) >> 1;
        let e1: i32 = k as i32 + 64;
        session_slot!(game, BabyMetroid[e].base.x_pos = (session_slot!(game, BabyMetroid[e1].base.x_pos)).wrapping_add((SHAKE_X_OFFSETS[v2 as usize]) as u16));
        session_slot!(game, BabyMetroid[e].base.y_pos = (session_slot!(game, BabyMetroid[e1].base.y_pos) as i32 + SHAKE_Y_OFFSETS[v2 as usize] as i32 - 32i32) as u16);
        let fresh13: u16 = session_slot!(game, BabyMetroid[e].function_timer);
        session_slot!(game, BabyMetroid[e].function_timer = session_slot!(game, BabyMetroid[e].function_timer).wrapping_sub(1));
        if fresh13 == 1 {
            session_slot!(game, BabyMetroid[e].function = function::MAKE_SIDEHOPPER_ROTTABLE);
            session_slot! { game, BabyMetroid[e].base; current_instruction = ADDR_K_BABY_METROID_ILIST_F906 as u16, instruction_timer = 1 }
            session_slot!(game, BabyMetroid[e].base.timer = 0);
            session_slot!(game, BabyMetroid[e].palette_handler_delay = 10);
        }
    };
    [11137336] move_up_and_unlock_camera [] {
        gradually_accelerate_towards_point_400h(k, 0, session_slot!(game, BabyMetroid[e].base.x_pos), 104, game);
        session_slot!(game, BabyMetroid[e].function_timer = session_slot!(game, BabyMetroid[e].function_timer).wrapping_sub(1));
        if (session_slot!(game, BabyMetroid[e].function_timer) & 0x8000) != 0 {
            session_slot! { game, BabyMetroid[e]; function = function::STARE_DOWN_SAMUS, contact_enabled = 1 }
            { let at: usize = crate::ram::bytes::SCROLLS; let bytes: [u8; 2] = ((u16::from_le_bytes([SessionRam::byte(game, at), SessionRam::byte(game, at + 1)])) | (0x100u16)).to_le_bytes(); SessionRam::set_byte(game, at, bytes[0]); SessionRam::set_byte(game, at + 1, bytes[1]); }
            { let at: usize = crate::ram::bytes::SCROLLS + 2; let bytes: [u8; 2] = ((u16::from_le_bytes([SessionRam::byte(game, at), SessionRam::byte(game, at + 1)])) | (0x100u16)).to_le_bytes(); SessionRam::set_byte(game, at, bytes[0]); SessionRam::set_byte(game, at + 1, bytes[1]); }
            spawn_plm(0x30, 0x3, 0xb763, game);
            spawn_plm(0x1f, 0x3, 0xb763, game);
        }
    };
    [11137408] stare_down_samus [] {
        let v2: u16 = abs16((session_slot!(game, BabyMetroid[e].base.x_pos)).wrapping_sub(game.player_x()));
        let mut proximity: u16;
        if v2 >= 8 {
            proximity = session_slot!(game, BabyMetroid[e].proximity_counter);
            if proximity != 0 {
                proximity = proximity.wrapping_sub(1);
                if (proximity & 0x8000) != 0 {
                    proximity = 0;
                }
            }
        } else {
            proximity = ((v2 >= 8) as i32 + session_slot!(game, BabyMetroid[e].proximity_counter) as i32 + 2) as u16;
        }
        session_slot!(game, BabyMetroid[e].proximity_counter = proximity);
        if proximity >= 0x100 || (game.player_x() as i32 - 512i32) & 0x8000 != 0 {
            session_slot!(game, BabyMetroid[word!(game.CUR_ENEMY_INDEX) as i32].function = function::LATCH_ONTO_SAMUS);
        } else {
            let aggro_timer: u16 = session_slot!(game, BabyMetroid[e].aggro_timer);
            let v5: u16;
            if aggro_timer != 0 {
                session_slot!(game, BabyMetroid[e].aggro_timer = (aggro_timer).wrapping_sub(1));
                v5 = game.player_y();
            } else {
                v5 = 80;
                if (word!(game.RANDOM_NUMBER) & 0xfff) >= 0xfe0 {
                    session_slot!(game, BabyMetroid[e].aggro_timer = 32);
                }
            }
            gradually_accelerate_towards_point_400h(
                word!(game.CUR_ENEMY_INDEX),
                0xa,
                game.player_x(),
                v5,
                game,
            );
        };
    };
    [11137563] draining_samus [] {
        if word!(game.SAMUS_HEALTH) < 2 {
            word! { game, 0; SAMUS_X_SPEED_DIVISOR, BOMB_COUNTER }
            session_slot!(game, BabyMetroid[e].function = function::START_HEEL_REALISATION);
            session_slot! { game, BabyMetroid[e]; x_velocity = 0, y_velocity = 0 }
            set_enemy_instruction_list(k, ADDR_K_BABY_METROID_ILIST_F906 as u16, game);
            session_slot!(game, BabyMetroid[e].palette_handler_delay = 10);
            run_samus_command(samus_command::DISABLE_BLUE_FLASHING, game);
            player_mother_brain(0, game);
            session_slot!(game, BabyMetroid[e].cry_enabled = 0);
            queue_music(music::SONG_2, game);
        } else {
            word! {game; COOLDOWN_TIMER = 8; BOMB_COUNTER = 5; SAMUS_X_SPEED_DIVISOR = 2; }
            if (word!(game.SAMUS_Y_SPEED) as i32 - 4i32) & 0x8000 == 0 {
                word!(game.SAMUS_Y_SPEED = 2);
            }
            let v2: i32 = (session_slot!(game, BabyMetroid[e].base.frame_counter) as i32 & 6) >> 1;
            session_slot!(game, BabyMetroid[e].base.x_pos = (game.player_x()).wrapping_add((SHAKE_X_OFFSETS[v2 as usize]) as u16));
            session_slot!(game, BabyMetroid[e].base.y_pos = (game.player_y() as i32 + SHAKE_Y_OFFSETS[v2 as usize] as i32 - 20i32) as u16);
            game.foreign(|game| game.drain_health_by_baby_metroid());
        };
    };
    [11137698] start_heel_realisation [] {
        session_slot! { game, BabyMetroid[e]; function = function::HEEL_REALISATION, function_timer = 120 }
        heel_realisation(k, game);
    };
    [11137955] samus_recovering [] {
        session_slot!(game, BabyMetroid[e].function_timer = session_slot!(game, BabyMetroid[e].function_timer).wrapping_sub(1));
        if (session_slot!(game, BabyMetroid[e].function_timer) & 0x8000) != 0 {
            player_mother_brain(2, game);
            session_slot! { game, BabyMetroid[e]; contact_enabled = 1, function = function::REMORSE }
            remorse(k, game);
        } else {
            gradually_accelerate_towards_samus(k, game);
        };
    };
    @countdown [pub fn (k: u16, game: &mut impl Lease)];
    before {
        let e: i32 = k as i32;
        session_slot!(game, BabyMetroid[e].function_timer = session_slot!(game, BabyMetroid[e].function_timer).wrapping_sub(1));
    }
    when (session_slot!(game, BabyMetroid[e].function_timer) & 0x8000) != 0;
    [11137079] queue_battle_music {
        queue_music(music::SONG_0, game);
        session_slot!(game, BabyMetroid[e].function = function::RUSH_TO_MIDDLE_OF_ROOM);
        rush_to_middle_of_room(k, game);
    };
    [11137710] heel_realisation {
        session_slot! { game, BabyMetroid[e]; function = function::BACK_OFF_GUILTILY, function_timer = 192 }
        back_off_guiltily(k, game);
    };
    @external {
        11136998 => [game.foreign(crate::actors::baby_metroid::wait_for_camera)];
        11137787 => [go_left_guiltily(game)]; 11137828 => [go_right_guiltily(game)];
        11137901 => [fleeing(game)];
    }
}
pub fn go_left_guiltily(game: &mut impl Lease) {
    gradually_accelerate_towards_point_400h(
        word!(game.CUR_ENEMY_INDEX),
        0,
        (game.player_x()).wrapping_sub(64),
        100,
        game,
    );
    let e: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
    session_slot!(
        game,
        BabyMetroid[e].function_timer =
            session_slot!(game, BabyMetroid[e].function_timer).wrapping_sub(1)
    );
    if (session_slot!(game, BabyMetroid[e].function_timer) & 0x8000) != 0 {
        session_slot! { game, BabyMetroid[e]; function = function::GO_RIGHT_GUILTILY, function_timer = 88 }
        go_right_guiltily(game);
    }
}
pub fn go_right_guiltily(game: &mut impl Lease) {
    gradually_accelerate_towards_point_400h(
        word!(game.CUR_ENEMY_INDEX),
        0,
        (game.player_x()).wrapping_add(96),
        104,
        game,
    );
    let e: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
    session_slot!(
        game,
        BabyMetroid[e].function_timer =
            session_slot!(game, BabyMetroid[e].function_timer).wrapping_sub(1)
    );
    if (session_slot!(game, BabyMetroid[e].function_timer) & 0x8000) != 0 {
        session_slot! { game, BabyMetroid[e]; function = function::SAMUS_RECOVERING, function_timer = 256 }
        session_slot! { game, BabyMetroid[e].base; current_instruction = ADDR_K_BABY_METROID_ILIST_F93A as u16, instruction_timer = 1 }
        session_slot!(game, BabyMetroid[e].base.timer = 0);
    }
}
pub fn fleeing(game: &mut impl Lease) {
    let rect = (-128i32 as u16, 64, 8, 8);
    gradually_accelerate_towards_point_400h(word!(game.CUR_ENEMY_INDEX), 0, rect.0, rect.1, game);
    if crate::actors::baby_metroid::separated_from_rect(word!(game.CUR_ENEMY_INDEX), rect, game)
        == 0
    {
        let e: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
        session_slot! { game, BabyMetroid[e]; x_velocity = 0, y_velocity = 0 }
        session_slot!(
            game,
            BabyMetroid[e].base.properties = (session_slot!(game, BabyMetroid[e].base.properties)
                as i32
                & !(K_ENEMY_PROPS_DISABLE_SAMUS_COLL as i32 | K_ENEMY_PROPS_INVISIBLE as i32))
                as u16
        );
        session_slot!(game, BabyMetroid[e].function = function::DISAPPEARED);
    }
}
pub fn make_flee(k: u16, game: &mut impl State) {
    session_slot!(
        game,
        BabyMetroid[k as i32].function = function::FLEE_REMORSEFULLY
    );
}
pub fn signal_to_leave(k: u16, game: &mut impl State) {
    if session_slot!(game, BabyMetroid[k as i32].function) == function::REMORSE {
        make_flee(k, game);
    }
}
pub fn gradually_accelerate_towards_samus(k: u16, game: &mut impl Lease) -> u8 {
    {
        let e: i32 = k as i32;
        let v2: u16 =
            abs16((session_slot!(game, BabyMetroid[e].base.x_pos)).wrapping_sub(game.player_x()));
        let mut proximity: u16;
        if v2 >= 2 {
            proximity = session_slot!(game, BabyMetroid[e].proximity_counter);
            if proximity != 0 {
                proximity = proximity.wrapping_sub(1);
                if (proximity & 0x8000) != 0 {
                    proximity = 0;
                }
            }
        } else {
            proximity = ((v2 >= 2) as i32
                + session_slot!(game, BabyMetroid[e].proximity_counter) as i32
                + 2) as u16;
        }
        session_slot!(game, BabyMetroid[e].proximity_counter = proximity);
        let aggro_timer: u16 = session_slot!(game, BabyMetroid[e].aggro_timer);
        let v5: u16;
        if aggro_timer != 0 {
            session_slot!(
                game,
                BabyMetroid[e].aggro_timer = (aggro_timer).wrapping_sub(1)
            );
            v5 = (game.player_y()).wrapping_sub(18);
        } else {
            v5 = 80;
            if (word!(game.RANDOM_NUMBER) & 0xfff) >= 0xfe0 {
                session_slot!(game, BabyMetroid[e].aggro_timer = 32);
            }
        }
        gradually_accelerate_towards_point_400h(
            word!(game.CUR_ENEMY_INDEX),
            8,
            game.player_x(),
            v5,
            game,
        );
        (session_slot!(
            game,
            BabyMetroid[word!(game.CUR_ENEMY_INDEX) as i32].proximity_counter
        ) >= 0x400
            || (game.player_x() as i32 - 128i32) & 0x8000 != 0) as u8
    }
}
pub fn gradually_accelerate_towards_point(
    k: u16,
    j: u16,
    r18: u16,
    r20: u16,
    r26: u16,
    game: &mut impl Lease,
) {
    {
        game.foreign(|game| game.homing_probe());
        let divisor = GRADUAL_ACCELERATION_DIVISORS[j as usize];
        gradually_accelerate_horizontally(k, r18, divisor as u16, r26, game);
        let e: i32 = k as i32;
        let distance =
            (session_slot!(game, BabyMetroid[e].base.y_pos) as i16).wrapping_sub(r20 as i16);
        if distance != 0 {
            session_slot!(
                game,
                BabyMetroid[e].y_velocity = homing_velocity(
                    distance,
                    session_slot!(game, BabyMetroid[e].y_velocity),
                    divisor as u8,
                    HomingReversal::ExtraForce(8),
                    1280,
                )
            );
        }
    }
}
pub fn gradually_accelerate_horizontally(
    k: u16,
    r18: u16,
    r24: u16,
    r26: u16,
    game: &mut impl State,
) {
    {
        let e: i32 = k as i32;
        let distance =
            (session_slot!(game, BabyMetroid[e].base.x_pos) as i16).wrapping_sub(r18 as i16);
        if distance != 0 {
            // Capture velocity before the screen check, as in both original branches.
            let velocity = session_slot!(game, BabyMetroid[e].x_velocity);
            let reversing = (velocity as i16 >= 0) == (distance > 0);
            let force = if reversing && crate::actors::baby_metroid::is_off_screen(k, game) & 1 != 0
            {
                r26.wrapping_add(8)
            } else {
                8
            };
            session_slot!(
                game,
                BabyMetroid[e].x_velocity = homing_velocity(
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
pub fn accelerate_towards_point(k: u16, a: u16, r18: u16, r20: u16, game: &mut impl State) -> u8 {
    let mut r28: u16 = 0;
    r28 = (r28).wrapping_add(crate::actors::baby_metroid::accelerate_towards_x(
        k, r18, a, game,
    ));
    r28 = (r28).wrapping_add(crate::actors::baby_metroid::accelerate_towards_y(
        k, r20, a, game,
    ));
    r28 >>= 1;
    ((r28) as u8) & (1u8)
}
pub fn handle_palette(r22: u16, r18: u16, game: &mut impl State) {
    let e: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
    let palette_timer: i8 = session_slot!(byte game, BabyMetroid[e].palette_animation[1]) as i8;
    if palette_timer != 0 {
        session_slot!(byte game, BabyMetroid[e].palette_animation[1] = ((palette_timer) as u8).wrapping_sub(1));
    } else {
        session_slot!(byte game, BabyMetroid[e].palette_animation[1] = session_slot!(game, BabyMetroid[e].palette_handler_delay) as u8);
        let v3: u8 =
            ((session_slot!(byte game, BabyMetroid[e].palette_animation[0]) as i32 + 1) & 7) as u8;
        session_slot!(byte game, BabyMetroid[e].palette_animation[0] = v3);
        let v4: u16 =
            crate::actors::baby_metroid::handle_cry(word!(game.CUR_ENEMY_INDEX), v3 as u16, game);
        copy_rom_colors_to_palette(r18, BANK, (r22 as i32 + 8 * v4 as i32) as u16, 4, game);
    };
}
crate::dispatch::instruction_callbacks! { pub fn (_k: u16, _jp: EnemyScript) -> EnemyScript;
    go_to_normal [EnemyScript::jump(ADDR_K_BABY_METROID_ILIST_F90E as u16)] { };
    go_to_latched_on [EnemyScript::jump(ADDR_K_BABY_METROID_ILIST_F924 as u16)] { };
    go_to_remorse [EnemyScript::jump(ADDR_K_BABY_METROID_ILIST_F93A as u16)] { };
}
pub fn initialize(game: &mut impl State) {
    word_transfers!(game, descending, 2048; le(TILEMAP_STUFF, 2047) <- value(0));
    let e: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
    session_slot!(
        game,
        BabyMetroid[e].base.properties = (session_slot!(game, BabyMetroid[e].base.properties)
            as i32
            | (K_ENEMY_PROPS_DISABLE_SAMUS_COLL as i32 | K_ENEMY_PROPS_BLOCK_PLASMA_BEAM as i32))
            as u16
    );
    session_slot!(game, BabyMetroid[e].base.palette_index = 1024);
    session_slot! { game, BabyMetroid[e].base; current_instruction = ADDR_K_BABY_METROID_ILIST_F90E as u16, instruction_timer = 1 }
    session_slot!(game, BabyMetroid[e].base.timer = 0);
    let mut v2: u16 = function::WAIT_FOR_CAMERA;
    if (crate::camera::current(game).layer1[0] & 0x8000) != 0 {
        session_slot!(
            game,
            BabyMetroid[e].base.properties = (session_slot!(game, BabyMetroid[e].base.properties)
                as i32
                | (K_ENEMY_PROPS_TANGIBLE as i32 | K_ENEMY_PROPS_INVISIBLE as i32))
                as u16
        );
        v2 = function::DISAPPEARED;
    }
    session_slot! { game, BabyMetroid[e]; function = v2, x_velocity = 0, y_velocity = 0, palette_handler_delay = 10 }
    session_slot!(game, BabyMetroid[e].contact_enabled = 0);
    copy_rom_colors_to_target_palette(
        BANK,
        cgram::SPRITE_PALETTE_1_COLOUR_0,
        ADDR_WORD_A9F8C6 as u16,
        0x10,
        game,
    );
    copy_rom_colors_to_target_palette(
        BANK,
        cgram::SPRITE_PALETTE_2_COLOUR_0,
        ADDR_K_BABY_METROID_IN_CUTSCENE_PALETTE as u16,
        0x10,
        game,
    );
    copy_rom_colors_to_target_palette(
        BANK,
        cgram::SPRITE_PALETTE_7_COLOUR_0,
        ADDR_K_DEAD_SIDEHOPPER_PALETTE_0 as u16,
        0x10,
        game,
    );
}
pub fn power_bomb_reaction(game: &mut impl Lease) {
    if session_slot!(
        game,
        BabyMetroid[word!(game.CUR_ENEMY_INDEX) as i32].contact_enabled
    ) != 0
    {
        signal_to_leave(word!(game.CUR_ENEMY_INDEX), game);
    }
    game.foreign(main_ai);
}
/// Main AI - baby Metroid: the frame runs on one lease.
pub fn main_ai(game: &mut impl Host) {
    baby_metroid_frame(&mut game.lease());
}
fn baby_metroid_frame(game: &mut impl Lease) {
    let e: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
    session_slot!(game, BabyMetroid[e].base.health = 0x7fff);
    run_baby_metroid_function(
        ((session_slot!(game, BabyMetroid[e].function)) as u32) | (BANK_BASE),
        word!(game.CUR_ENEMY_INDEX),
        game,
    );
    move_enemy_with_velocity(game);
    if word!(game.PALETTE_CHANGE_NUM) == 0 {
        handle_normal_palette(game);
    }
}
pub fn touch(game: &mut impl State) {
    let e: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
    if session_slot!(game, BabyMetroid[e].contact_enabled) != 0 {
        signal_to_leave(word!(game.CUR_ENEMY_INDEX), game);
        if SessionRam::byte(game, 0xa1f) == 3 && (game.player_x() as i32 - 512i32) & 0x8000 == 0 {
            let r18: u16 = (0x80
                - crate::actors::queries::angle_from_delta(
                    (game.player_x()).wrapping_sub(session_slot!(game, BabyMetroid[e].base.x_pos)),
                    (game.player_y()).wrapping_sub(session_slot!(game, BabyMetroid[e].base.y_pos)),
                ) as i32
                + 0x80) as u8 as u16;
            session_slot!(
                game,
                BabyMetroid[e].x_velocity = (session_slot!(game, BabyMetroid[e].x_velocity))
                    .wrapping_add(x_velocity_at_angle(0x40, r18))
            );
            session_slot!(
                game,
                BabyMetroid[e].y_velocity = (session_slot!(game, BabyMetroid[e].y_velocity))
                    .wrapping_add(y_velocity_at_angle(0x40, r18))
            );
        } else if session_slot!(game, BabyMetroid[e].function) == function::LATCH_ONTO_SAMUS {
            if accelerate_towards_point(
                word!(game.CUR_ENEMY_INDEX),
                0x200,
                game.player_x(),
                (game.player_y()).wrapping_sub(32),
                game,
            ) & 1
                != 0
            {
                session_slot! { game, BabyMetroid[e].base; current_instruction = ADDR_K_BABY_METROID_ILIST_F924 as u16, instruction_timer = 1 }
                session_slot!(game, BabyMetroid[e].base.timer = 0);
                session_slot!(game, BabyMetroid[e].palette_handler_delay = 1);
                session_slot! { game, BabyMetroid[e]; contact_enabled = 0, x_velocity = 0, y_velocity = 0 }
                session_slot!(
                    game,
                    BabyMetroid[e].function = function::START_DRAINING_SAMUS
                );
            }
        } else if session_slot!(game, BabyMetroid[e].function) == function::STARE_DOWN_SAMUS {
            session_slot!(game, BabyMetroid[e].function = function::LATCH_ONTO_SAMUS);
        }
    }
}
pub fn shot(game: &mut impl State) {
    let e: i32 = word!(game.CUR_ENEMY_INDEX) as i32;
    if session_slot!(game, BabyMetroid[e].contact_enabled) != 0 {
        signal_to_leave(word!(game.CUR_ENEMY_INDEX), game);
        let r18: u16 = -(crate::actors::queries::angle_from_delta(
            (word!(game.PROJECTILE_X_POS[0]))
                .wrapping_sub(session_slot!(game, BabyMetroid[e].base.x_pos)),
            (word!(game.PROJECTILE_Y_POS[0]))
                .wrapping_sub(session_slot!(game, BabyMetroid[e].base.y_pos)),
        ) as i32) as u8 as u16;
        let mut v1: u16 = (8u16).wrapping_mul(word!(
            game.PROJECTILE_DAMAGE[word!(game.COLLISION_DETECTION_INDEX)]
        ));
        if v1 >= 0xf0 {
            v1 = 240;
        }
        session_slot!(
            game,
            BabyMetroid[e].x_velocity = (session_slot!(game, BabyMetroid[e].x_velocity))
                .wrapping_add(x_velocity_at_angle(v1, r18))
        );
        session_slot!(
            game,
            BabyMetroid[e].y_velocity = (session_slot!(game, BabyMetroid[e].y_velocity))
                .wrapping_add(y_velocity_at_angle(v1, r18))
        );
    }
}
pub fn branch_or_queue_remorse_sound(
    _k: u16,
    jp: EnemyScript,
    game: &mut impl State,
) -> EnemyScript {
    if (word!(game.RANDOM_NUMBER) & 0x8000) == 0 {
        return EnemyScript::jump(jp.word(0));
    }
    queue_sfx2_max6(sound::BABY_METROID_REMORSE, game);
    jp.offset(1)
}
