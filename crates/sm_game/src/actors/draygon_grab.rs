//! Draygon chasing and grabbing Samus, its rising spiral and tail whips, and
//! letting her go (bank $A5:8E19-$A5:94DC, $A5:9B9A).
//! Boss-specific words retain their native compatibility layout; no player copy exists.
use crate::actors::draygon_flight::{facing_animation, fire_wall_turret, foam_at_mouth};
use crate::actors::draygon_host::Lease;
use crate::actors::draygon_host::{
    ARMS, BODY, Draygon, DraygonEdit, GRAPPLE_DROPPED, INTANGIBLE, SIGN, TAIL, abs16, function,
    instruction_list as list,
};
use crate::actors::queries::angle_from_delta;

use crate::menus::prelude::{battle, owned};

use crate::enemy_script::EnemyScript;

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
/// Leave a step's X and Y speeds in $0E24.$0E26 and $0E28.$0E2A, as the
/// original passes them to its move routine.
pub(crate) fn store_speeds<S: Lease>(vector: crate::geometry::Vector32, game: &mut S) {
    game.session_mut().actors.compatibility.scratch[1..].copy_from_slice(&[
        (vector.x >> 16) as u16,
        vector.x as u16,
        (vector.y >> 16) as u16,
        vector.y as u16,
    ]);
    game.words_stored();
}
fn pursue<S: Lease>(dx: u16, dy: u16, game: &mut S) {
    let angle = QUARTER_TURN.wrapping_sub(angle_from_delta(dx, dy) as u8) as u16;
    game.set_step_scratch(angle);
    let vector = game.direction_step(angle, 2);
    store_speeds(vector, game);
    game.step_along(angle, vector);
}
/// While the grapple is connected to a block, flash and run the hurt AI.
fn hurt_by_grapple<S: Lease>(game: &mut S) -> bool {
    if battle!(game.SamusGrappleFlags) & GRAPPLE_CONNECTED == 0 {
        return false;
    }
    let definition = Draygon::of(game, BODY).actor().definition;
    let duration = game
        .actor_record(ENEMY_DEF_BANK, definition)
        .byte(ENEMY_DEF_HURT_AI_TIME);
    DraygonEdit::of(game, BODY).actor().flash = u16::from(duration).wrapping_add(FLASH_EXTRA);
    DraygonEdit::of(game, BODY).actor().ai_handlers |= HURT_AI;
    true
}
fn hold_offset<S: Lease>(k: u16, game: &mut S) -> u16 {
    if Draygon::of(game, k).facing_right() == 0 {
        0u16.wrapping_sub(HOLD_X)
    } else {
        HOLD_X
    }
}
/// Draygon body function - chase Samus ($A5:8E19): fly at Samus and grab her,
/// unless the grapple repels Draygon.
pub fn chase_samus<L: Lease>(game: &mut L) {
    fire_wall_turret(game);
    if battle!(game.SamusXSpeedDivisor) == 0 {
        *DraygonEdit::of(game, BODY).function() = function::FLY_STRAIGHT_UP;
        return;
    }
    DraygonEdit::of(game, BODY).actor().properties |= INTANGIBLE;
    let dx = hold_offset(BODY, game);
    let distance = abs16(
        Draygon::of(game, BODY)
            .actor()
            .x
            .whole
            .wrapping_add(dx)
            .wrapping_sub(game.player_x()),
    );
    if distance.wrapping_sub(GRAB_RANGE) & SIGN != 0
        && abs16(
            Draygon::of(game, BODY)
                .actor()
                .y
                .whole
                .wrapping_sub(game.player_y()),
        )
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
        let direction = u16::from(Draygon::of(game, BODY).facing_right() != 0);
        let repelled = game.owned_step(|game| {
            use crate::player_transitions::{command, commands::finish_ceres_arrival};
            command(game, GRAB_COMMAND, finish_ceres_arrival) != 0
        });
        if !repelled {
            crate::probe::hit("draygon_grab.grab");
            game.owned_step(|game| {
                crate::player_movement::forced::grab_by_draygon(direction, game)
            });
        }
        if repelled {
            *DraygonEdit::of(game, BODY).function() = function::REPELLED_BY_GRAPPLE;
        } else {
            {
                let mut draygon = DraygonEdit::of(game, BODY);
                *draygon.spiral_x() = TARGET_X;
                *draygon.spiral_y() = TARGET_Y;
                *draygon.spiral_x_radius() = 0;
                *draygon.spiral_x_subradius() = 0;
                *draygon.spiral_angle() = SPIRAL_ANGLE;
                *draygon.spiral_angle_delta() = SPIRAL_ANGLE_DELTA;
                *draygon.function() = function::GRABBED_MOVING_TO_TARGET;
            }
        }
    } else {
        let dx = game
            .player_x()
            .wrapping_sub(Draygon::of(game, BODY).actor().x.whole);
        let dy = game
            .player_y()
            .wrapping_sub(Draygon::of(game, BODY).actor().y.whole);
        pursue(dx, dy, game);
    }
}
/// Draygon body function - repelled by grapple ($A5:8F10).
pub fn repelled_by_grapple<S: Lease>(game: &mut S) {
    crate::probe::hit("draygon_grab.repelled_by_grapple");
    battle!(game.GrappleBeamFunction = GRAPPLE_DROPPED);
    *DraygonEdit::of(game, BODY).function() = function::FLAIL_TAIL_AND_FLY_STRAIGHT_UP;
}
/// Draygon body function - grabbed Samus - moving to target position
/// ($A5:8F1E), then roar.
pub fn move_to_target<S: Lease>(k: u16, game: &mut S) {
    if hurt_by_grapple(game) {
        return;
    }
    let dx = abs16(
        Draygon::of(game, BODY)
            .actor()
            .x
            .whole
            .wrapping_sub(TARGET_X),
    );
    if dx.wrapping_sub(TARGET_RANGE) & SIGN != 0
        && abs16(
            Draygon::of(game, BODY)
                .actor()
                .y
                .whole
                .wrapping_sub(TARGET_Y),
        )
        .wrapping_sub(TARGET_RANGE)
            & SIGN
            != 0
    {
        *DraygonEdit::of(game, BODY).function() = function::GRABBED_RISING_SPIRAL;
        facing_animation(BODY, k, [list::BODY_LEFT_ROAR, list::BODY_RIGHT_ROAR], game);
        DraygonEdit::of(game, BODY).actor().properties |= INTANGIBLE;
    } else {
        let dx = TARGET_X.wrapping_sub(Draygon::of(game, BODY).actor().x.whole);
        let dy = TARGET_Y.wrapping_sub(Draygon::of(game, BODY).actor().y.whole);
        pursue(dx, dy, game);
        move_samus_with_draygon(k, game);
    }
}
/// Draygon body function - grabbed Samus - rising spiral movement ($A5:8FD6):
/// widen and raise the spiral, now and then whipping the tail, until it is wide
/// or high enough for the final tail whips.
pub fn rising_spiral<L: Lease>(k: u16, game: &mut L) {
    if hurt_by_grapple(game) {
        return;
    }
    if battle!(game.Random) as u8 == 0 {
        {
            let mut draygon = DraygonEdit::of(game, BODY);
            *draygon.tail_whip_timer() = TAIL_WHIP_FRAMES;
            *draygon.function() = function::GRABBED_TAIL_WHIP;
        }
        return;
    }
    DraygonEdit::of(game, BODY).actor().x.whole =
        Draygon::of(game, BODY)
            .spiral_x()
            .wrapping_add(game.scaled_cosine(
                Draygon::of(game, BODY).spiral_angle(),
                Draygon::of(game, BODY).spiral_x_radius(),
            ));
    DraygonEdit::of(game, BODY).actor().y.whole =
        Draygon::of(game, BODY)
            .spiral_y()
            .wrapping_add(game.scaled_sine(
                Draygon::of(game, BODY).spiral_angle(),
                Draygon::of(game, BODY).spiral_x_radius() >> 2,
            ));
    foam_at_mouth(k, SPIRAL_FOAM_FRAMES, game);
    {
        let mut draygon = DraygonEdit::of(game, BODY);
        let sum = ((u32::from(*draygon.spiral_x_radius()) << 16)
            | u32::from(*draygon.spiral_x_subradius()))
        .wrapping_add(SPIRAL_GROWTH);
        *draygon.spiral_x_subradius() = sum as u16;
        *draygon.spiral_x_radius() = (sum >> 16) as u16;
    }
    let mut finished = Draygon::of(game, BODY)
        .spiral_x_radius()
        .wrapping_sub(MAX_SPIRAL_RADIUS)
        & SIGN
        == 0;
    if !finished {
        let delta = Draygon::of(game, BODY).spiral_angle_delta().wrapping_sub(1);
        *DraygonEdit::of(game, BODY).spiral_angle_delta() = delta;
        let angle = Draygon::of(game, BODY).spiral_angle().to_ne_bytes()[0];
        *DraygonEdit::of(game, BODY).spiral_angle() =
            u16::from(angle.wrapping_add(delta.to_ne_bytes()[1]));
        let subpixel = Draygon::of(game, BODY).spiral_y_subpixel();
        *DraygonEdit::of(game, BODY).spiral_y_subpixel() =
            subpixel.wrapping_sub(SPIRAL_RISE_SUBPIXEL);
        let y = (((u32::from(Draygon::of(game, BODY).spiral_y()) << 16) | u32::from(subpixel))
            .wrapping_sub(SPIRAL_RISE)
            >> 16) as u16;
        *DraygonEdit::of(game, BODY).spiral_y() = y;
        finished = y.wrapping_sub(SPIRAL_TOP) & SIGN != 0;
    }
    if finished {
        *DraygonEdit::of(game, BODY).function() = function::GRABBED_FINAL_TAIL_WHIPS_START;
    } else {
        move_samus_with_draygon(k, game);
    }
}
/// Draygon body function - grabbed Samus - tail whip ($A5:90D4).
pub fn tail_whip<S: Lease>(k: u16, game: &mut S) {
    move_samus_with_draygon(k, game);
    let remaining = Draygon::of(game, BODY).tail_whip_timer().wrapping_sub(1);
    *DraygonEdit::of(game, BODY).tail_whip_timer() = remaining;
    if remaining == 0 {
        *DraygonEdit::of(game, BODY).function() = function::GRABBED_RISING_SPIRAL;
    } else if remaining == TAIL_WHIP_LASH {
        crate::probe::hit("draygon_grab.tail_whip");
        facing_animation(
            BODY,
            TAIL,
            [list::TAIL_LEFT_TAIL_WHIP, list::TAIL_RIGHT_TAIL_WHIP],
            game,
        );
    }
}
/// Draygon body function - grabbed Samus - final tail whips - start ($A5:9105).
pub fn final_tail_whips<S: Lease>(k: u16, game: &mut S) {
    crate::probe::hit("draygon_grab.final_tail_whips");
    move_samus_with_draygon(k, game);
    facing_animation(
        k,
        TAIL,
        [
            list::TAIL_LEFT_FINAL_TAIL_WHIPS,
            list::TAIL_RIGHT_FINAL_TAIL_WHIPS,
        ],
        game,
    );
    *DraygonEdit::of(game, BODY).function() = function::GRABBED_FINAL_TAIL_WHIPS_ONGOING;
}
/// Move Samus with Draygon ($A5:94A9), which is also the final tail whips'
/// ongoing function ($A5:9124): hold her, and fly up once she is released.
pub fn move_samus_with_draygon<S: Lease>(k: u16, game: &mut S) {
    let dx = hold_offset(k, game);
    game.set_player_x(Draygon::of(game, BODY).actor().x.whole.wrapping_add(dx));
    game.set_player_y(Draygon::of(game, BODY).actor().y.whole.wrapping_add(HOLD_Y));
    crate::player_position::correct_previous_position(game);
    if battle!(game.SamusGrappleFlags) & SAMUS_RELEASED != 0 {
        battle!(game.SamusGrappleFlags = 0);
        *DraygonEdit::of(game, BODY).function() = function::FLY_STRAIGHT_UP;
    }
}
/// Draygon body function - flail tail and fly straight up ($A5:9128).
pub fn flail_tail_and_fly_straight_up<L: Lease>(game: &mut L) {
    crate::probe::hit("draygon_grab.flail_tail_and_fly_straight_up");
    game.owned_step(crate::player_movement::forced::release_from_draygon);
    battle!(game.SamusGrappleFlags = 0);
    *DraygonEdit::of(game, BODY).function() = function::FLY_STRAIGHT_UP;
    DraygonEdit::of(game, BODY).actor().properties &= !INTANGIBLE;
    facing_animation(
        BODY,
        TAIL,
        [list::TAIL_LEFT_TAIL_FLAIL, list::TAIL_RIGHT_TAIL_FLAIL],
        game,
    );
}
/// Draygon body function - fly straight up ($A5:9154), then swoop right again.
pub fn fly_straight_up<S: Lease>(game: &mut S) {
    fire_wall_turret(game);
    let y = (Draygon::of(game, BODY).actor().y.whole as i16).wrapping_sub(RISE);
    DraygonEdit::of(game, BODY).actor().y.whole = y as u16;
    if y < 0 {
        DraygonEdit::of(game, BODY).actor().properties &= !INTANGIBLE;
        {
            let mut draygon = DraygonEdit::of(game, BODY);
            *draygon.function() = function::SWOOP_RIGHT_SETUP;
            *draygon.function_timer() = 0;
        }
        DraygonEdit::of(game, BODY).actor().x.whole = Draygon::of(game, BODY).left_reset_x();
        DraygonEdit::of(game, BODY).actor().y.whole = Draygon::of(game, BODY).reset_y();
    }
}

/// Instruction - Draygon tail whip hit ($A5:9B9A): hurt Samus, shake the
/// screen and raise a dust cloud.
pub fn tail_whip_hit<L: Lease>(script: EnemyScript, game: &mut L) -> EnemyScript {
    *DraygonEdit::of(game, BODY).swoop_y_acceleration() = INITIAL_SWOOP_Y_ACCELERATION;
    let definition = Draygon::of(game, BODY).actor().definition;
    game.owned_step(|game| {
        let record = crate::cartridge::native_record(ENEMY_DEF_BANK, definition, game);
        let amount = crate::actors::combat::suit_damage(game, record.word(ENEMY_DEF_DAMAGE));
        crate::samus_periodic_damage::contact(game, amount);
    });
    battle!(game; EarthquakeTimer = EARTHQUAKE_FRAMES; EarthquakeType = EARTHQUAKE_BG1_VERTICAL_3PX;);
    let x = game.player_x();
    let y = game.player_y().wrapping_add(DUST_CLOUD_Y);
    crate::actors::boss_lease::steps::create_sprite(game, x, y, DUST_CLOUD);
    script
}
