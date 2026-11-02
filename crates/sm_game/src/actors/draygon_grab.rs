//! Draygon chasing and grabbing Samus, its rising spiral and tail whips, and
//! letting her go (bank $A5:8E19-$A5:94DC, $A5:9B9A).
//! Boss-specific words retain their native compatibility layout; no player copy exists.
use crate::actors::draygon_host::Lease;

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

/// While the grapple is connected to a block, flash and run the hurt AI.

/// Draygon body function - chase Samus ($A5:8E19): fly at Samus and grab her,
/// unless the grapple repels Draygon. Was `acquire`.
pub fn chase_samus<L: Lease>(game: &mut L) {
    game.foreign(|game| {
        owned::chase_samus(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}
/// Draygon body function - repelled by grapple ($A5:8F10). Was `cancel_grapple`.
pub fn repelled_by_grapple<S: owned::Host>(game: &mut S) {
    owned::repelled_by_grapple(game)
}
/// Draygon body function - grabbed Samus - moving to target position
/// ($A5:8F1E), then roar. Was `center`.
pub fn move_to_target<S: owned::Host>(k: u16, game: &mut S) {
    owned::move_to_target(k, game)
}
/// Draygon body function - grabbed Samus - rising spiral movement ($A5:8FD6):
/// widen and raise the spiral, now and then whipping the tail, until it is wide
/// or high enough for the final tail whips. Was `orbit`.
pub fn rising_spiral<L: Lease>(k: u16, game: &mut L) {
    game.foreign(|game| {
        owned::rising_spiral(k, &mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}
/// Draygon body function - grabbed Samus - tail whip ($A5:90D4). Was `pause`.
pub fn tail_whip<S: owned::Host>(k: u16, game: &mut S) {
    owned::tail_whip(k, game)
}
/// Draygon body function - grabbed Samus - final tail whips - start ($A5:9105).
/// Was `start_release`.
pub fn final_tail_whips<S: owned::Host>(k: u16, game: &mut S) {
    owned::final_tail_whips(k, game)
}
/// Move Samus with Draygon ($A5:94A9), which is also the final tail whips'
/// ongoing function ($A5:9124): hold her, and fly up once she is released.
/// Was `attach`.
pub fn move_samus_with_draygon<S: owned::Host>(k: u16, game: &mut S) {
    owned::move_samus_with_draygon(k, game)
}
/// Draygon body function - flail tail and fly straight up ($A5:9128). Was
/// `release`.
pub fn flail_tail_and_fly_straight_up<L: Lease>(game: &mut L) {
    game.foreign(|game| {
        owned::flail_tail_and_fly_straight_up(&mut crate::runtime_access::RuntimeAccess::runtime(
            game,
        ))
    })
}
/// Draygon body function - fly straight up ($A5:9154), then swoop right again.
/// Was `depart`.
pub fn fly_straight_up<S: owned::Host>(game: &mut S) {
    owned::fly_straight_up(game)
}

/// Instruction - Draygon tail whip hit ($A5:9B9A): hurt Samus, shake the
/// screen and raise a dust cloud. Was `slam`.
pub fn tail_whip_hit<L: Lease>(script: EnemyScript, game: &mut L) -> EnemyScript {
    game.foreign(|game| {
        owned::tail_whip_hit(
            script,
            &mut crate::runtime_access::RuntimeAccess::runtime(game),
        )
    })
}

#[path = "draygon_grab_owned.rs"]
pub mod owned;
