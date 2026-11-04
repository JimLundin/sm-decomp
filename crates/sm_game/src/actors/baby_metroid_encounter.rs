//! The baby Metroid's encounter in Tourian: it drains the sidehopper, latches onto
//! Samus and drains her until she is nearly dead, then lets go and leaves.
//!
//! Was `BabyMetroid_*` in the original decompilation (bank $A9:EF37-$A9:F8A5).
//! Slot words are named through `baby_metroid`'s view after the bank log's
//! baby Metroid notes. The bank's dispatch ids are unchanged; the functions and
//! their reads and writes keep the original order. Mother Brain's shared
//! helpers come from `mother_brain_fight`.
use crate::actors::baby_metroid::{BabyMetroid, BabyMetroidEdit};
use crate::actors::corpses::DeadMonsterEdit;
use crate::actors::host::abs16;
use crate::actors::mother_brain_fight::OwnedWord;
use crate::actors::mother_brain_fight::random_number;
use crate::actors::mother_brain_fight::{
    BANK, BANK_BASE, Lease as FightLease, State, cgram, music, samus_command, sound,
};
use crate::actors::mother_brain_fight::{
    SHAKE_X_OFFSETS, SHAKE_Y_OFFSETS, bank_unreachable, move_enemy_with_velocity,
    player_mother_brain, queue_music, queue_sfx2_max6, run_samus_command,
    set_enemy_instruction_list, spawn_plm,
};
use crate::runtime_access::SessionRam;

use crate::constants::bank_a9::*;
use crate::constants::enemy_properties::*;
use crate::enemy_projectiles::shared_pre::{x_velocity_at_angle, y_velocity_at_angle};
use crate::enemy_script::EnemyScript;
use crate::palette_copies::{copy_rom_colors_to_palette, copy_rom_colors_to_target_palette};

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
    assert!(at(crate::actors::baby_metroid::LET_SIDEHOPPER_LIVE) == 11137067);
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

/// A Mother Brain lease with the baby Metroid's camera wait.
pub trait Lease: FightLease {
    /// Run the camera wait on the current Session, retaining the former
    /// foreign entry's preflight and publication if an effect fails.
    fn wait_for_camera(&mut self);
}

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
            *BabyMetroidEdit::of(game, k).function() = function::RUSH_TO_SIDEHOPPER;
        }
    };
    [11137133] rush_to_sidehopper [] {
        let e = k + 0x40;
        gradually_accelerate_towards_point_400h(
            k,
            0xf,
            BabyMetroid::of(game, e).actor().x.whole,
            (BabyMetroid::of(game, e).actor().y.whole).wrapping_sub(32),
            game,
        );
        if crate::actors::baby_metroid::separated_from_enemy(k, (k).wrapping_add(64), game) & 1 == 0 {
            *BabyMetroidEdit::of(game, k).function() = function::LATCH_ONTO_SIDEHOPPER;
        }
    };
    [11137172] latch_onto_sidehopper [] {
        let e1 = k + 0x40;
        if accelerate_towards_point(
            k,
            0x200,
            BabyMetroid::of(game, e1).actor().x.whole,
            (BabyMetroid::of(game, e1).actor().y.whole).wrapping_sub(32),
            game,
        ) & 1
            != 0
        {
            let e = k;
            *BabyMetroidEdit::of(game, e).x_velocity() = 0;
            *BabyMetroidEdit::of(game, e).y_velocity() = 0;
            BabyMetroidEdit::of(game, e).actor().x.whole = BabyMetroid::of(game, e1).actor().x.whole;
            BabyMetroidEdit::of(game, e).actor().y.whole = (BabyMetroid::of(game, e1).actor().y.whole).wrapping_sub(32);
            BabyMetroidEdit::of(game, e).actor().instruction = ADDR_K_BABY_METROID_ILIST_F924 as u16;
            BabyMetroidEdit::of(game, e).actor().instruction_delay = 1;
            BabyMetroidEdit::of(game, e).actor().timer = 0;
            *BabyMetroidEdit::of(game, e).function() = function::DRAINING_SIDEHOPPER;
            *BabyMetroidEdit::of(game, e).palette_handler_delay() = 1;
            *BabyMetroidEdit::of(game, e).contact_enabled() = 0;
            *BabyMetroidEdit::of(game, e).function_timer() = 320;
        }
    };
    [11137317] make_sidehopper_rottable [] {
        *DeadMonsterEdit::of(game, 0x40).drained_palette_index() = 1;
        let e = k;
        *BabyMetroidEdit::of(game, e).function() = function::MOVE_UP_AND_UNLOCK_CAMERA;
        *BabyMetroidEdit::of(game, e).function_timer() = 192;
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
        *BabyMetroidEdit::of(game, 0).function() = function::DRAINING_SAMUS;
        draining_samus(k, game);
    };
    @functions [pub fn (_k: u16, game: &mut impl Lease)];
    [11137728] back_off_guiltily [] {
        gradually_accelerate_towards_point_400h(
            game.current_enemy(),
            0,
            game.player_x(),
            104,
            game,
        );
        let e = game.current_enemy();
        *BabyMetroidEdit::of(game, e).function_timer() = BabyMetroid::of(game, e).function_timer().wrapping_sub(1);
        if (BabyMetroid::of(game, e).function_timer() & 0x8000) != 0 {
            queue_sfx2_max6(sound::BABY_METROID_GUILTY, game);
            *BabyMetroidEdit::of(game, e).function() = function::GO_LEFT_GUILTILY;
            *BabyMetroidEdit::of(game, e).function_timer() = 88;
            BabyMetroidEdit::of(game, e).actor().instruction = ADDR_K_BABY_METROID_ILIST_F924 as u16;
            BabyMetroidEdit::of(game, e).actor().instruction_delay = 1;
            BabyMetroidEdit::of(game, e).actor().timer = 0;
            go_left_guiltily(game);
        }
    };
    @functions [pub fn (k: u16, game: &mut impl Lease)];
    [11137888] flee_remorsefully [] {
        queue_sfx2_max6(sound::BABY_METROID_REMORSE, game);
        *BabyMetroidEdit::of(game, k).function() = function::FLEEING;
        fleeing(game);
    };
    [11137982] remorse [] {
        if gradually_accelerate_towards_samus(k, game) & 1 != 0 {
            make_flee(k, game);
        }
    };
    @functions [pub fn (k: u16, game: &mut impl Lease)];
    @prefix { let e = k; };
    [11136991] disappeared [] {
        *BabyMetroidEdit::of(game, e).x_velocity() = 0;
        *BabyMetroidEdit::of(game, e).y_velocity() = 0;
    };
    [11137067] let_sidehopper_live_for_a_bit [] {
        *BabyMetroidEdit::of(game, e).function() = function::QUEUE_BATTLE_MUSIC;
        *BabyMetroidEdit::of(game, e).function_timer() = 464;
        queue_battle_music(k, game);
    };
    [11137254] draining_sidehopper [] {
        let v2: i32 = (BabyMetroid::of(game, e).actor().frame as i32 & 6) >> 1;
        let e1 = k + 0x40;
        BabyMetroidEdit::of(game, e).actor().x.whole = (BabyMetroid::of(game, e1).actor().x.whole).wrapping_add((SHAKE_X_OFFSETS[v2 as usize]) as u16);
        BabyMetroidEdit::of(game, e).actor().y.whole = (BabyMetroid::of(game, e1).actor().y.whole as i32 + SHAKE_Y_OFFSETS[v2 as usize] as i32 - 32i32) as u16;
        let fresh13: u16 = BabyMetroid::of(game, e).function_timer();
        *BabyMetroidEdit::of(game, e).function_timer() = BabyMetroid::of(game, e).function_timer().wrapping_sub(1);
        if fresh13 == 1 {
            *BabyMetroidEdit::of(game, e).function() = function::MAKE_SIDEHOPPER_ROTTABLE;
            BabyMetroidEdit::of(game, e).actor().instruction = ADDR_K_BABY_METROID_ILIST_F906 as u16;
            BabyMetroidEdit::of(game, e).actor().instruction_delay = 1;
            BabyMetroidEdit::of(game, e).actor().timer = 0;
            *BabyMetroidEdit::of(game, e).palette_handler_delay() = 10;
        }
    };
    [11137336] move_up_and_unlock_camera [] {
        gradually_accelerate_towards_point_400h(k, 0, BabyMetroid::of(game, e).actor().x.whole, 104, game);
        *BabyMetroidEdit::of(game, e).function_timer() = BabyMetroid::of(game, e).function_timer().wrapping_sub(1);
        if (BabyMetroid::of(game, e).function_timer() & 0x8000) != 0 {
            *BabyMetroidEdit::of(game, e).function() = function::STARE_DOWN_SAMUS;
            *BabyMetroidEdit::of(game, e).contact_enabled() = 1;
            { let at: usize = crate::ram::bytes::SCROLLS; let bytes: [u8; 2] = ((u16::from_le_bytes([SessionRam::byte(game, at), SessionRam::byte(game, at + 1)])) | (0x100u16)).to_le_bytes(); SessionRam::set_byte(game, at, bytes[0]); SessionRam::set_byte(game, at + 1, bytes[1]); }
            { let at: usize = crate::ram::bytes::SCROLLS + 2; let bytes: [u8; 2] = ((u16::from_le_bytes([SessionRam::byte(game, at), SessionRam::byte(game, at + 1)])) | (0x100u16)).to_le_bytes(); SessionRam::set_byte(game, at, bytes[0]); SessionRam::set_byte(game, at + 1, bytes[1]); }
            spawn_plm(0x30, 0x3, 0xb763, game);
            spawn_plm(0x1f, 0x3, 0xb763, game);
        }
    };
    [11137408] stare_down_samus [] {
        let v2: u16 = abs16((BabyMetroid::of(game, e).actor().x.whole).wrapping_sub(game.player_x()));
        let mut proximity: u16;
        if v2 >= 8 {
            proximity = BabyMetroid::of(game, e).proximity_counter();
            if proximity != 0 {
                proximity = proximity.wrapping_sub(1);
                if (proximity & 0x8000) != 0 {
                    proximity = 0;
                }
            }
        } else {
            proximity = ((v2 >= 8) as i32 + BabyMetroid::of(game, e).proximity_counter() as i32 + 2) as u16;
        }
        *BabyMetroidEdit::of(game, e).proximity_counter() = proximity;
        if proximity >= 0x100 || (game.player_x() as i32 - 512i32) & 0x8000 != 0 {
            *BabyMetroidEdit::of(game, game.current_enemy()).function() = function::LATCH_ONTO_SAMUS;
        } else {
            let aggro_timer: u16 = BabyMetroid::of(game, e).aggro_timer();
            let v5: u16;
            if aggro_timer != 0 {
                *BabyMetroidEdit::of(game, e).aggro_timer() = (aggro_timer).wrapping_sub(1);
                v5 = game.player_y();
            } else {
                v5 = 80;
                if (random_number(game) & 0xfff) >= 0xfe0 {
                    *BabyMetroidEdit::of(game, e).aggro_timer() = 32;
                }
            }
            gradually_accelerate_towards_point_400h(
                game.current_enemy(),
                0xa,
                game.player_x(),
                v5,
                game,
            );
        };
    };
    [11137563] draining_samus [] {
        if game.owned(OwnedWord::SamusHealth) < 2 {
            { game.set_owned(OwnedWord::SpeedDivisor, 0); game.set_owned(OwnedWord::BombCounter, 0); }
            *BabyMetroidEdit::of(game, e).function() = function::START_HEEL_REALISATION;
            *BabyMetroidEdit::of(game, e).x_velocity() = 0;
            *BabyMetroidEdit::of(game, e).y_velocity() = 0;
            set_enemy_instruction_list(k, ADDR_K_BABY_METROID_ILIST_F906 as u16, game);
            *BabyMetroidEdit::of(game, e).palette_handler_delay() = 10;
            run_samus_command(samus_command::DISABLE_BLUE_FLASHING, game);
            player_mother_brain(0, game);
            *BabyMetroidEdit::of(game, e).cry_enabled() = 0;
            queue_music(music::SONG_2, game);
        } else {
            { game.set_owned(OwnedWord::Cooldown, 8); game.set_owned(OwnedWord::BombCounter, 5); game.set_owned(OwnedWord::SpeedDivisor, 2); }
            let y_speed = game.owned(OwnedWord::SamusYSpeed);
            if (y_speed as i32 - 4i32) & 0x8000 == 0 {
                game.set_owned(OwnedWord::SamusYSpeed, 2);
            }
            // The speed compare ($A9:F238) leaves the carry the shake's add takes:
            // set when the speed was at least 4, unsigned.
            let carry = u16::from(y_speed >= 4);
            let v2: i32 = (BabyMetroid::of(game, e).actor().frame as i32 & 6) >> 1;
            BabyMetroidEdit::of(game, e).actor().x.whole = (game.player_x()).wrapping_add((SHAKE_X_OFFSETS[v2 as usize]) as u16).wrapping_add(carry);
            BabyMetroidEdit::of(game, e).actor().y.whole = (game.player_y() as i32 + SHAKE_Y_OFFSETS[v2 as usize] as i32 - 20i32) as u16;
            game.owned_step(crate::actors::mother_brain_vitals::drain_health_by_baby_metroid);
        };
    };
    [11137698] start_heel_realisation [] {
        *BabyMetroidEdit::of(game, e).function() = function::HEEL_REALISATION;
        *BabyMetroidEdit::of(game, e).function_timer() = 120;
        heel_realisation(k, game);
    };
    [11137955] samus_recovering [] {
        *BabyMetroidEdit::of(game, e).function_timer() = BabyMetroid::of(game, e).function_timer().wrapping_sub(1);
        if (BabyMetroid::of(game, e).function_timer() & 0x8000) != 0 {
            player_mother_brain(2, game);
            *BabyMetroidEdit::of(game, e).contact_enabled() = 1;
            *BabyMetroidEdit::of(game, e).function() = function::REMORSE;
            remorse(k, game);
        } else {
            gradually_accelerate_towards_samus(k, game);
        };
    };
    @countdown [pub fn (k: u16, game: &mut impl Lease)];
    before {
        let e = k;
        *BabyMetroidEdit::of(game, e).function_timer() = BabyMetroid::of(game, e).function_timer().wrapping_sub(1);
    }
    when (BabyMetroid::of(game, e).function_timer() & 0x8000) != 0;
    [11137079] queue_battle_music {
        queue_music(music::SONG_0, game);
        *BabyMetroidEdit::of(game, e).function() = function::RUSH_TO_MIDDLE_OF_ROOM;
        rush_to_middle_of_room(k, game);
    };
    [11137710] heel_realisation {
        *BabyMetroidEdit::of(game, e).function() = function::BACK_OFF_GUILTILY;
        *BabyMetroidEdit::of(game, e).function_timer() = 192;
        back_off_guiltily(k, game);
    };
    @external {
        11136998 => [game.wait_for_camera()];
        11137787 => [go_left_guiltily(game)]; 11137828 => [go_right_guiltily(game)];
        11137901 => [fleeing(game)];
    }
}
pub fn go_left_guiltily(game: &mut impl Lease) {
    gradually_accelerate_towards_point_400h(
        game.current_enemy(),
        0,
        (game.player_x()).wrapping_sub(64),
        100,
        game,
    );
    let e = game.current_enemy();
    *BabyMetroidEdit::of(game, e).function_timer() =
        BabyMetroid::of(game, e).function_timer().wrapping_sub(1);
    if (BabyMetroid::of(game, e).function_timer() & 0x8000) != 0 {
        *BabyMetroidEdit::of(game, e).function() = function::GO_RIGHT_GUILTILY;
        *BabyMetroidEdit::of(game, e).function_timer() = 88;
        go_right_guiltily(game);
    }
}
pub fn go_right_guiltily(game: &mut impl Lease) {
    gradually_accelerate_towards_point_400h(
        game.current_enemy(),
        0,
        (game.player_x()).wrapping_add(96),
        104,
        game,
    );
    let e = game.current_enemy();
    *BabyMetroidEdit::of(game, e).function_timer() =
        BabyMetroid::of(game, e).function_timer().wrapping_sub(1);
    if (BabyMetroid::of(game, e).function_timer() & 0x8000) != 0 {
        *BabyMetroidEdit::of(game, e).function() = function::SAMUS_RECOVERING;
        *BabyMetroidEdit::of(game, e).function_timer() = 256;
        BabyMetroidEdit::of(game, e).actor().instruction = ADDR_K_BABY_METROID_ILIST_F93A as u16;
        BabyMetroidEdit::of(game, e).actor().instruction_delay = 1;
        BabyMetroidEdit::of(game, e).actor().timer = 0;
    }
}
pub fn fleeing(game: &mut impl Lease) {
    let rect = (-128i32 as u16, 64, 8, 8);
    gradually_accelerate_towards_point_400h(game.current_enemy(), 0, rect.0, rect.1, game);
    if crate::actors::baby_metroid::separated_from_rect(game.current_enemy(), rect, game) == 0 {
        let e = game.current_enemy();
        *BabyMetroidEdit::of(game, e).x_velocity() = 0;
        *BabyMetroidEdit::of(game, e).y_velocity() = 0;
        BabyMetroidEdit::of(game, e).actor().properties =
            (BabyMetroid::of(game, e).actor().properties as i32
                & !(K_ENEMY_PROPS_DISABLE_SAMUS_COLL as i32 | K_ENEMY_PROPS_INVISIBLE as i32))
                as u16;
        *BabyMetroidEdit::of(game, e).function() = function::DISAPPEARED;
    }
}
pub fn make_flee(k: u16, game: &mut impl State) {
    *BabyMetroidEdit::of(game, k).function() = function::FLEE_REMORSEFULLY;
}
pub fn signal_to_leave(k: u16, game: &mut impl State) {
    if BabyMetroid::of(game, k).function() == function::REMORSE {
        make_flee(k, game);
    }
}
pub fn gradually_accelerate_towards_samus(k: u16, game: &mut impl Lease) -> u8 {
    crate::probe::hit("baby_metroid.accelerate_towards_samus");
    {
        let e = k;
        let v2: u16 =
            abs16((BabyMetroid::of(game, e).actor().x.whole).wrapping_sub(game.player_x()));
        let mut proximity: u16;
        if v2 >= 2 {
            proximity = BabyMetroid::of(game, e).proximity_counter();
            if proximity != 0 {
                proximity = proximity.wrapping_sub(1);
                if (proximity & 0x8000) != 0 {
                    proximity = 0;
                }
            }
        } else {
            proximity =
                ((v2 >= 2) as i32 + BabyMetroid::of(game, e).proximity_counter() as i32 + 2) as u16;
        }
        *BabyMetroidEdit::of(game, e).proximity_counter() = proximity;
        let aggro_timer: u16 = BabyMetroid::of(game, e).aggro_timer();
        let v5: u16;
        if aggro_timer != 0 {
            *BabyMetroidEdit::of(game, e).aggro_timer() = (aggro_timer).wrapping_sub(1);
            v5 = (game.player_y()).wrapping_sub(18);
        } else {
            v5 = 80;
            if (random_number(game) & 0xfff) >= 0xfe0 {
                *BabyMetroidEdit::of(game, e).aggro_timer() = 32;
            }
        }
        gradually_accelerate_towards_point_400h(game.current_enemy(), 8, game.player_x(), v5, game);
        (BabyMetroid::of(game, game.current_enemy()).proximity_counter() >= 0x400
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
    game.owned_step(|_| crate::probe::hit("homing.metroid"));
    let divisor = GRADUAL_ACCELERATION_DIVISORS[j as usize] as u8;
    gradually_accelerate_horizontally(k, r18, u16::from(divisor), r26, game);
    let e = k;
    let position = BabyMetroid::of(game, e).actor().y.whole;
    let velocity = BabyMetroid::of(game, e).y_velocity();
    if let Some(velocity) = homing_step(position, r20, velocity, divisor, None, 0x500) {
        *BabyMetroidEdit::of(game, e).y_velocity() = velocity;
    }
}
pub fn gradually_accelerate_horizontally(
    k: u16,
    r18: u16,
    r24: u16,
    r26: u16,
    game: &mut impl State,
) {
    let e = k;
    let position = BabyMetroid::of(game, e).actor().x.whole;
    // Capture velocity before the screen check, as in both original branches.
    let velocity = BabyMetroid::of(game, e).x_velocity();
    let toward_left = position.wrapping_sub(r18) & 0x8000 == 0;
    let reversing = position != r18 && (velocity & 0x8000 == 0) == toward_left;
    let off_screen = reversing && crate::actors::baby_metroid::is_off_screen(k, game) & 1 != 0;
    let extra = off_screen.then_some(r26);
    if let Some(velocity) = homing_step(position, r18, velocity, r24 as u8, extra, 0x800) {
        *BabyMetroidEdit::of(game, e).x_velocity() = velocity;
    }
}

/// One axis of gradually accelerate towards point ($A9:F476 and $A9:F4E6),
/// with the 65816's carry: the position compare's carry feeds the final add
/// or subtract, and reversing chains its eight and its step through the carry.
/// `extra` is $1A when the reversing enemy is off-screen; the screen check
/// returns with the carry set, so the add takes one more. None when the
/// position is already the target.
fn homing_step(
    position: u16,
    target: u16,
    velocity: u16,
    divisor: u8,
    extra: Option<u16>,
    limit: u16,
) -> Option<u16> {
    let difference = position.wrapping_sub(target);
    let compare_carry = position >= target;
    if difference == 0 {
        return None;
    }
    let add = |a: u16, b: u16, carry: bool| {
        let sum = u32::from(a) + u32::from(b) + u32::from(carry);
        (sum as u16, sum > 0xffff)
    };
    let subtract = |a: u16, b: u16, carry: bool| {
        let borrow = u32::from(!carry);
        (
            a.wrapping_sub(b).wrapping_sub(borrow as u16),
            u32::from(a) >= u32::from(b) + borrow,
        )
    };
    Some(if difference & 0x8000 == 0 {
        let step = snes_divide(difference, divisor).max(1);
        let (velocity, carry) = if velocity & 0x8000 == 0 {
            let velocity = extra.map_or(velocity, |extra| subtract(velocity, extra, true).0);
            let (velocity, carry) = subtract(velocity, 8, true);
            subtract(velocity, step, carry)
        } else {
            (velocity, compare_carry)
        };
        let velocity = subtract(velocity, step, carry).0;
        let floor = 0u16.wrapping_sub(limit);
        if velocity.wrapping_sub(floor) & 0x8000 == 0 {
            velocity
        } else {
            floor
        }
    } else {
        let step = snes_divide(difference.wrapping_neg(), divisor).max(1);
        let (velocity, carry) = if velocity & 0x8000 != 0 {
            let velocity = extra.map_or(velocity, |extra| add(velocity, extra, true).0);
            let (velocity, carry) = add(velocity, 8, false);
            add(velocity, step, carry)
        } else {
            (velocity, compare_carry)
        };
        let velocity = add(velocity, step, carry).0;
        if velocity.wrapping_sub(limit) & 0x8000 != 0 {
            velocity
        } else {
            limit
        }
    })
}
/// The hardware divider's quotient; a zero divisor leaves FFFFh.
fn snes_divide(dividend: u16, divisor: u8) -> u16 {
    dividend.checked_div(u16::from(divisor)).unwrap_or(0xffff)
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
    let k = game.current_enemy();
    // The palette animation word: the frame in its low byte, the timer in its high.
    let [frame, timer] = BabyMetroid::of(game, k).palette_animation().to_le_bytes();
    if timer != 0 {
        *BabyMetroidEdit::of(game, k).palette_animation() =
            u16::from_le_bytes([frame, timer.wrapping_sub(1)]);
    } else {
        let delay = BabyMetroid::of(game, k)
            .palette_handler_delay()
            .to_le_bytes()[0];
        let frame = frame.wrapping_add(1) & 7;
        *BabyMetroidEdit::of(game, k).palette_animation() = u16::from_le_bytes([frame, delay]);
        let v4: u16 = crate::actors::baby_metroid::handle_cry(k, u16::from(frame), game);
        copy_rom_colors_to_palette(r18, BANK, (r22 as i32 + 8 * v4 as i32) as u16, 4, game);
    };
}
crate::dispatch::instruction_callbacks! { pub fn (_k: u16, _jp: EnemyScript) -> EnemyScript;
    go_to_normal [EnemyScript::jump(ADDR_K_BABY_METROID_ILIST_F90E as u16)] { };
    go_to_latched_on [EnemyScript::jump(ADDR_K_BABY_METROID_ILIST_F924 as u16)] { };
    go_to_remorse [EnemyScript::jump(ADDR_K_BABY_METROID_ILIST_F93A as u16)] { };
}
pub fn initialize(game: &mut impl State) {
    // Clear the BG2 tilemap staging, last word first.
    crate::actors::boss_lease::bg2_staging::fill(game, (0..2048).rev(), 0);
    let e = game.current_enemy();
    BabyMetroidEdit::of(game, e).actor().properties = (BabyMetroid::of(game, e).actor().properties
        as i32
        | (K_ENEMY_PROPS_DISABLE_SAMUS_COLL as i32 | K_ENEMY_PROPS_BLOCK_PLASMA_BEAM as i32))
        as u16;
    BabyMetroidEdit::of(game, e).actor().palette = 1024;
    BabyMetroidEdit::of(game, e).actor().instruction = ADDR_K_BABY_METROID_ILIST_F90E as u16;
    BabyMetroidEdit::of(game, e).actor().instruction_delay = 1;
    BabyMetroidEdit::of(game, e).actor().timer = 0;
    let mut v2: u16 = function::WAIT_FOR_CAMERA;
    if (game.session().room.camera.layer1[0] & 0x8000) != 0 {
        BabyMetroidEdit::of(game, e).actor().properties =
            (BabyMetroid::of(game, e).actor().properties as i32
                | (K_ENEMY_PROPS_TANGIBLE as i32 | K_ENEMY_PROPS_INVISIBLE as i32))
                as u16;
        v2 = function::DISAPPEARED;
    }
    *BabyMetroidEdit::of(game, e).function() = v2;
    *BabyMetroidEdit::of(game, e).x_velocity() = 0;
    *BabyMetroidEdit::of(game, e).y_velocity() = 0;
    *BabyMetroidEdit::of(game, e).palette_handler_delay() = 10;
    *BabyMetroidEdit::of(game, e).contact_enabled() = 0;
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
    if BabyMetroid::of(game, game.current_enemy()).contact_enabled() != 0 {
        signal_to_leave(game.current_enemy(), game);
    }
    main_ai(game);
}
/// Main AI - baby Metroid.
pub fn main_ai(game: &mut impl Lease) {
    let e = game.current_enemy();
    BabyMetroidEdit::of(game, e).actor().health = 0x7fff;
    run_baby_metroid_function(
        ((BabyMetroid::of(game, e).function()) as u32) | (BANK_BASE),
        game.current_enemy(),
        game,
    );
    move_enemy_with_velocity(game);
    if game.owned(OwnedWord::PaletteFadeStep) == 0 {
        handle_normal_palette(game);
    }
}
pub fn touch(game: &mut impl State) {
    let e = game.current_enemy();
    if BabyMetroid::of(game, e).contact_enabled() != 0 {
        signal_to_leave(game.current_enemy(), game);
        if SessionRam::byte(game, 0xa1f) == 3 && (game.player_x() as i32 - 512i32) & 0x8000 == 0 {
            let r18: u16 = (0x80
                - crate::actors::queries::angle_from_delta(
                    (game.player_x()).wrapping_sub(BabyMetroid::of(game, e).actor().x.whole),
                    (game.player_y()).wrapping_sub(BabyMetroid::of(game, e).actor().y.whole),
                ) as i32
                + 0x80) as u8 as u16;
            *BabyMetroidEdit::of(game, e).x_velocity() = BabyMetroid::of(game, e)
                .x_velocity()
                .wrapping_add(x_velocity_at_angle(0x40, r18));
            *BabyMetroidEdit::of(game, e).y_velocity() = BabyMetroid::of(game, e)
                .y_velocity()
                .wrapping_add(y_velocity_at_angle(0x40, r18));
        } else if BabyMetroid::of(game, e).function() == function::LATCH_ONTO_SAMUS {
            if accelerate_towards_point(
                game.current_enemy(),
                0x200,
                game.player_x(),
                (game.player_y()).wrapping_sub(32),
                game,
            ) & 1
                != 0
            {
                BabyMetroidEdit::of(game, e).actor().instruction =
                    ADDR_K_BABY_METROID_ILIST_F924 as u16;
                BabyMetroidEdit::of(game, e).actor().instruction_delay = 1;
                BabyMetroidEdit::of(game, e).actor().timer = 0;
                *BabyMetroidEdit::of(game, e).palette_handler_delay() = 1;
                *BabyMetroidEdit::of(game, e).contact_enabled() = 0;
                *BabyMetroidEdit::of(game, e).x_velocity() = 0;
                *BabyMetroidEdit::of(game, e).y_velocity() = 0;
                *BabyMetroidEdit::of(game, e).function() = function::START_DRAINING_SAMUS;
            }
        } else if BabyMetroid::of(game, e).function() == function::STARE_DOWN_SAMUS {
            *BabyMetroidEdit::of(game, e).function() = function::LATCH_ONTO_SAMUS;
        }
    }
}
pub fn shot(game: &mut impl State) {
    let e = game.current_enemy();
    if BabyMetroid::of(game, e).contact_enabled() != 0 {
        signal_to_leave(game.current_enemy(), game);
        let r18: u16 = -(crate::actors::queries::angle_from_delta(
            game.owned(OwnedWord::ShotX(0))
                .wrapping_sub(BabyMetroid::of(game, e).actor().x.whole),
            game.owned(OwnedWord::ShotY(0))
                .wrapping_sub(BabyMetroid::of(game, e).actor().y.whole),
        ) as i32) as u8 as u16;
        let mut v1: u16 = (8u16).wrapping_mul(game.owned(OwnedWord::ShotDamage(usize::from(
            game.owned(OwnedWord::CollisionIndex),
        ))));
        if v1 >= 0xf0 {
            v1 = 240;
        }
        *BabyMetroidEdit::of(game, e).x_velocity() = BabyMetroid::of(game, e)
            .x_velocity()
            .wrapping_add(x_velocity_at_angle(v1, r18));
        *BabyMetroidEdit::of(game, e).y_velocity() = BabyMetroid::of(game, e)
            .y_velocity()
            .wrapping_add(y_velocity_at_angle(v1, r18));
    }
}
pub fn branch_or_queue_remorse_sound(
    _k: u16,
    jp: EnemyScript,
    game: &mut impl State,
) -> EnemyScript {
    if (random_number(game) & 0x8000) == 0 {
        return EnemyScript::jump(jp.word(0));
    }
    queue_sfx2_max6(sound::BABY_METROID_REMORSE, game);
    jp.offset(1)
}
