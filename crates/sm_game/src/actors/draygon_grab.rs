//! Draygon chasing and grabbing Samus, its rising spiral and tail whips, and
//! letting her go (bank $A5:8E19-$A5:94DC, $A5:9B9A).
//! Boss-specific words retain their native compatibility layout; no player copy exists.
#![forbid(unsafe_code)]
use crate::actors::boss_lease::{BossLease, BossState};
use crate::actors::draygon_flight::{facing_animation, fire_wall_turret, foam_at_mouth};
use crate::actors::draygon_host::{
    ARMS, BODY, Draygon, GRAPPLE_DROPPED, Host, INTANGIBLE, SIGN, TAIL, abs16, function,
    instruction_list as list,
};
use crate::actors::host::BossEffects as _;
use crate::actors::queries::angle_from_delta;
use crate::actors::session_slots::session_slot;

use crate::enemy_script::EnemyScript;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};

/// EnemyDef: the hurt AI time byte and the contact damage word.
pub const ENEMY_DEF_HURT_AI_TIME: usize = 13;
pub const ENEMY_DEF_DAMAGE: usize = 6;

/// Enemy definitions are in bank $A0.
const ENEMY_DEF_BANK: u8 = 0xa0;
/// Samus hangs 8 px ahead of and 28h px below the body; the grab reaches 8 px.
const HOLD_X: u16 = 8;
const HOLD_Y: u16 = 0x28;
const GRAB_RANGE: u16 = 8;
/// Player command 0Dh: Draygon grabs Samus; nonzero while the grapple is out.
const GRAB_COMMAND: u16 = 0xd;
/// Grapple flag bit 0: connected to a block; bit 1: Samus released.
const GRAPPLE_CONNECTED: u16 = 1;
const SAMUS_RELEASED: u16 = 2;
/// The flash lasts the hurt AI time plus 8 frames; AI handler bit 1 is hurt AI.
const FLASH_EXTRA: u16 = 8;
const HURT_AI: u16 = 2;
/// The target position, (100h, 180h), reached within 2 px.
const TARGET_X: u16 = 0x100;
const TARGET_Y: u16 = 0x180;
const TARGET_RANGE: u16 = 2;
/// The spiral starts at (100h, 180h), angle C0h, radius 0, angle delta 800h;
/// its radius grows 0.2000h a frame up to A0h while it rises 0.4000h a frame
/// to Y 40h.
const SPIRAL_ANGLE: u16 = 0xc0;
const SPIRAL_ANGLE_DELTA: u16 = 0x800;
const SPIRAL_GROWTH: u32 = 0x2000;
const MAX_SPIRAL_RADIUS: u16 = 0xa0;
const SPIRAL_RISE: u32 = 0x4000;
const SPIRAL_RISE_SUBPIXEL: u16 = 0x4000;
const SPIRAL_TOP: u16 = 0x40;
const SPIRAL_FOAM_FRAMES: u16 = 7;
/// A tail whip lasts 40h frames and lashes on its first.
const TAIL_WHIP_FRAMES: u16 = 0x40;
const TAIL_WHIP_LASH: u16 = 0x3f;
/// Flying straight up rises 4 px a frame.
const RISE: i16 = 4;
/// The tail whip hit resets the swoop Y acceleration, shakes BG1 (type 7) for
/// 20h frames and raises a big dust cloud (sprite object 15h) 10h px below Samus.
const INITIAL_SWOOP_Y_ACCELERATION: u16 = 0x18;
const EARTHQUAKE_FRAMES: u16 = 0x20;
const EARTHQUAKE_BG1_VERTICAL_3PX: u16 = 7;
const DUST_CLOUD: u16 = 0x15;
const DUST_CLOUD_Y: u16 = 0x10;
/// Angles are measured from the top, clockwise; 40h less is the maths angle.
const QUARTER_TURN: u8 = 0x40;

/// Move 2 px a frame along the angle to (`dx`, `dy`).
fn pursue<S: BossState + ?Sized>(dx: u16, dy: u16, game: &mut S) {
    let angle = QUARTER_TURN.wrapping_sub(angle_from_delta(dx, dy) as u8) as u16;
    game.set_step_scratch(angle);
    let vector = game.direction_step(angle, 2);
    game.step_along(angle, vector);
}
/// While the grapple is connected to a block, flash and run the hurt AI.
fn hurt_by_grapple<S: BossState + ?Sized>(game: &mut S) -> bool {
    if word!(game.samus_grapple_flags) & GRAPPLE_CONNECTED == 0 {
        return false;
    }
    let definition = session_slot!(game, Draygon[BODY].base.enemy_ptr);
    let duration = game
        .actor_record(ENEMY_DEF_BANK, definition)
        .byte(ENEMY_DEF_HURT_AI_TIME);
    session_slot!(
        game,
        Draygon[BODY].base.flash_timer = u16::from(duration).wrapping_add(FLASH_EXTRA)
    );
    session_slot!(game, Draygon[BODY].base.ai_handler_bits, |old| old
        | HURT_AI);
    true
}
fn hold_offset<S: BossState + ?Sized>(k: i32, game: &mut S) -> u16 {
    if session_slot!(game, Draygon[k].facing_right) == 0 {
        0u16.wrapping_sub(HOLD_X)
    } else {
        HOLD_X
    }
}
/// Draygon body function - chase Samus ($A5:8E19): fly at Samus and grab her,
/// unless the grapple repels Draygon. Was `acquire`.
pub fn chase_samus<L: BossLease + ?Sized>(game: &mut L)
where
    L::Game: Host,
{
    fire_wall_turret(game);
    if word!(game.samus_x_speed_divisor) == 0 {
        session_slot!(game, Draygon[BODY].function = function::FLY_STRAIGHT_UP);
        return;
    }
    session_slot!(game, Draygon[BODY].base.properties, |old| old | INTANGIBLE);
    let dx = hold_offset(BODY, game);
    let distance = abs16(
        session_slot!(game, Draygon[BODY].base.x_pos)
            .wrapping_add(dx)
            .wrapping_sub(game.player_x()),
    );
    if distance.wrapping_sub(GRAB_RANGE) & SIGN != 0
        && abs16(session_slot!(game, Draygon[BODY].base.y_pos).wrapping_sub(game.player_y()))
            .wrapping_sub(GRAB_RANGE)
            & SIGN
            != 0
    {
        facing_animation(
            BODY,
            ARMS,
            [list::ARMS_LEFT_GRAB, list::ARMS_RIGHT_GRAB],
            game,
        );
        let direction = u16::from(session_slot!(game, Draygon[BODY].facing_right) != 0);
        let repelled = game.foreign(|game| {
            let repelled = game.command_player(GRAB_COMMAND) != 0;
            if !repelled {
                game.grab_samus(direction);
            }
            repelled
        });
        if repelled {
            session_slot!(game, Draygon[BODY].function = function::REPELLED_BY_GRAPPLE);
        } else {
            session_slot!(game, Draygon[BODY]; spiral_x = TARGET_X, spiral_y = TARGET_Y,
                spiral_x_radius = 0, spiral_x_subradius = 0, spiral_angle = SPIRAL_ANGLE,
                spiral_angle_delta = SPIRAL_ANGLE_DELTA,
                function = function::GRABBED_MOVING_TO_TARGET);
        }
    } else {
        let dx = game
            .player_x()
            .wrapping_sub(session_slot!(game, Draygon[BODY].base.x_pos));
        let dy = game
            .player_y()
            .wrapping_sub(session_slot!(game, Draygon[BODY].base.y_pos));
        pursue(dx, dy, game);
    }
}
/// Draygon body function - repelled by grapple ($A5:8F10). Was `cancel_grapple`.
pub fn repelled_by_grapple<S: BossState + ?Sized>(game: &mut S) {
    word!(game.grapple_beam_function = GRAPPLE_DROPPED);
    session_slot!(
        game,
        Draygon[BODY].function = function::FLAIL_TAIL_AND_FLY_STRAIGHT_UP
    );
}
/// Draygon body function - grabbed Samus - moving to target position
/// ($A5:8F1E), then roar. Was `center`.
pub fn move_to_target<S: BossState + ?Sized>(k: u16, game: &mut S) {
    if hurt_by_grapple(game) {
        return;
    }
    let dx = abs16(session_slot!(game, Draygon[BODY].base.x_pos).wrapping_sub(TARGET_X));
    if dx.wrapping_sub(TARGET_RANGE) & SIGN != 0
        && abs16(session_slot!(game, Draygon[BODY].base.y_pos).wrapping_sub(TARGET_Y))
            .wrapping_sub(TARGET_RANGE)
            & SIGN
            != 0
    {
        session_slot!(
            game,
            Draygon[BODY].function = function::GRABBED_RISING_SPIRAL
        );
        facing_animation(
            BODY,
            k as i32,
            [list::BODY_LEFT_ROAR, list::BODY_RIGHT_ROAR],
            game,
        );
        session_slot!(game, Draygon[BODY].base.properties, |old| old | INTANGIBLE);
    } else {
        let dx = TARGET_X.wrapping_sub(session_slot!(game, Draygon[BODY].base.x_pos));
        let dy = TARGET_Y.wrapping_sub(session_slot!(game, Draygon[BODY].base.y_pos));
        pursue(dx, dy, game);
        move_samus_with_draygon(k, game);
    }
}
/// Draygon body function - grabbed Samus - rising spiral movement ($A5:8FD6):
/// widen and raise the spiral, now and then whipping the tail, until it is wide
/// or high enough for the final tail whips. Was `orbit`.
pub fn rising_spiral<L: BossLease + ?Sized>(k: u16, game: &mut L)
where
    L::Game: Host,
{
    if hurt_by_grapple(game) {
        return;
    }
    if word!(game.random_number) as u8 == 0 {
        session_slot!(game, Draygon[BODY]; tail_whip_timer = TAIL_WHIP_FRAMES,
            function = function::GRABBED_TAIL_WHIP);
        return;
    }
    session_slot!(
        game,
        Draygon[BODY].base.x_pos =
            session_slot!(game, Draygon[BODY].spiral_x).wrapping_add(game.scaled_cosine(
                session_slot!(game, Draygon[BODY].spiral_angle),
                session_slot!(game, Draygon[BODY].spiral_x_radius),
            ))
    );
    session_slot!(
        game,
        Draygon[BODY].base.y_pos =
            session_slot!(game, Draygon[BODY].spiral_y).wrapping_add(game.scaled_sine(
                session_slot!(game, Draygon[BODY].spiral_angle),
                session_slot!(game, Draygon[BODY].spiral_x_radius) >> 2,
            ))
    );
    foam_at_mouth(k, SPIRAL_FOAM_FRAMES, game);
    session_slot!(
        game,
        Draygon[BODY](spiral_x_radius, spiral_x_subradius) += SPIRAL_GROWTH
    );
    let mut finished =
        session_slot!(game, Draygon[BODY].spiral_x_radius).wrapping_sub(MAX_SPIRAL_RADIUS) & SIGN
            == 0;
    if !finished {
        let delta = session_slot!(game, Draygon[BODY].spiral_angle_delta).wrapping_sub(1);
        session_slot!(game, Draygon[BODY].spiral_angle_delta = delta);
        let angle = session_slot!(game, Draygon[BODY].spiral_angle).to_ne_bytes()[0];
        session_slot!(
            game,
            Draygon[BODY].spiral_angle = u16::from(angle.wrapping_add(delta.to_ne_bytes()[1]))
        );
        let subpixel = session_slot!(game, Draygon[BODY].spiral_y_subpixel);
        session_slot!(
            game,
            Draygon[BODY].spiral_y_subpixel = subpixel.wrapping_sub(SPIRAL_RISE_SUBPIXEL)
        );
        let y = (((u32::from(session_slot!(game, Draygon[BODY].spiral_y)) << 16)
            | u32::from(subpixel))
        .wrapping_sub(SPIRAL_RISE)
            >> 16) as u16;
        session_slot!(game, Draygon[BODY].spiral_y = y);
        finished = y.wrapping_sub(SPIRAL_TOP) & SIGN != 0;
    }
    if finished {
        session_slot!(
            game,
            Draygon[BODY].function = function::GRABBED_FINAL_TAIL_WHIPS_START
        );
    } else {
        move_samus_with_draygon(k, game);
    }
}
/// Draygon body function - grabbed Samus - tail whip ($A5:90D4). Was `pause`.
pub fn tail_whip<S: BossState + ?Sized>(k: u16, game: &mut S) {
    move_samus_with_draygon(k, game);
    let remaining = session_slot!(game, Draygon[BODY].tail_whip_timer).wrapping_sub(1);
    session_slot!(game, Draygon[BODY].tail_whip_timer = remaining);
    if remaining == 0 {
        session_slot!(
            game,
            Draygon[BODY].function = function::GRABBED_RISING_SPIRAL
        );
    } else if remaining == TAIL_WHIP_LASH {
        facing_animation(
            BODY,
            TAIL,
            [list::TAIL_LEFT_TAIL_WHIP, list::TAIL_RIGHT_TAIL_WHIP],
            game,
        );
    }
}
/// Draygon body function - grabbed Samus - final tail whips - start ($A5:9105).
/// Was `start_release`.
pub fn final_tail_whips<S: BossState + ?Sized>(k: u16, game: &mut S) {
    move_samus_with_draygon(k, game);
    facing_animation(
        k as i32,
        TAIL,
        [
            list::TAIL_LEFT_FINAL_TAIL_WHIPS,
            list::TAIL_RIGHT_FINAL_TAIL_WHIPS,
        ],
        game,
    );
    session_slot!(
        game,
        Draygon[BODY].function = function::GRABBED_FINAL_TAIL_WHIPS_ONGOING
    );
}
/// Move Samus with Draygon ($A5:94A9), which is also the final tail whips'
/// ongoing function ($A5:9124): hold her, and fly up once she is released.
/// Was `attach`.
pub fn move_samus_with_draygon<S: BossState + ?Sized>(k: u16, game: &mut S) {
    let dx = hold_offset(k as i32, game);
    game.set_player_x(session_slot!(game, Draygon[BODY].base.x_pos).wrapping_add(dx));
    game.set_player_y(session_slot!(game, Draygon[BODY].base.y_pos).wrapping_add(HOLD_Y));
    crate::player_position::correct_previous_position(game);
    if word!(game.samus_grapple_flags) & SAMUS_RELEASED != 0 {
        word!(game.samus_grapple_flags = 0);
        session_slot!(game, Draygon[BODY].function = function::FLY_STRAIGHT_UP);
    }
}
/// Draygon body function - flail tail and fly straight up ($A5:9128). Was
/// `release`.
pub fn flail_tail_and_fly_straight_up<L: BossLease + ?Sized>(game: &mut L)
where
    L::Game: Host,
{
    game.foreign(|game| game.release_samus());
    word!(game.samus_grapple_flags = 0);
    session_slot!(game, Draygon[BODY].function = function::FLY_STRAIGHT_UP);
    session_slot!(game, Draygon[BODY].base.properties, |old| old & !INTANGIBLE);
    facing_animation(
        BODY,
        TAIL,
        [list::TAIL_LEFT_TAIL_FLAIL, list::TAIL_RIGHT_TAIL_FLAIL],
        game,
    );
}
/// Draygon body function - fly straight up ($A5:9154), then swoop right again.
/// Was `depart`.
pub fn fly_straight_up<S: BossState + ?Sized>(game: &mut S) {
    fire_wall_turret(game);
    let y = (session_slot!(game, Draygon[BODY].base.y_pos) as i16).wrapping_sub(RISE);
    session_slot!(game, Draygon[BODY].base.y_pos = y as u16);
    if y < 0 {
        session_slot!(game, Draygon[BODY].base.properties, |old| old & !INTANGIBLE);
        session_slot!(game, Draygon[BODY]; function = function::SWOOP_RIGHT_SETUP, function_timer = 0);
        session_slot!(game, Draygon[BODY].base; x_pos = session_slot!(game, Draygon[BODY].left_reset_x),
            y_pos = session_slot!(game, Draygon[BODY].reset_y));
    }
}

/// Instruction - Draygon tail whip hit ($A5:9B9A): hurt Samus, shake the
/// screen and raise a dust cloud. Was `slam`.
pub fn tail_whip_hit<H: Host + ?Sized>(script: EnemyScript, game: &mut H) -> EnemyScript {
    slot!(
        game,
        Draygon[BODY].swoop_y_acceleration = INITIAL_SWOOP_Y_ACCELERATION
    );
    let definition = slot!(game, Draygon[BODY].base.enemy_ptr);
    let amount = game
        .legacy_native_record(ENEMY_DEF_BANK, definition)
        .word(ENEMY_DEF_DAMAGE);
    let amount = game.suit_damage(amount);
    game.damage_player(amount);
    word! {game; earthquake_timer = EARTHQUAKE_FRAMES; earthquake_type = EARTHQUAKE_BG1_VERTICAL_3PX; }
    let x = game.player_x();
    let y = game.player_y().wrapping_add(DUST_CLOUD_Y);
    game.create_sprite(x, y, DUST_CLOUD, 0);
    script
}
