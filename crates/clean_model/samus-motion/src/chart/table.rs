//! `PadChart`'s table (§7) as data: the rows of each situation, and `Aiming`'s rules.
//!
//! A row is `(keys, action, aim, flags, beats)`. It fits when every key it needs is there (the
//! keys are below: the buttons it names held or just pressed, and conditions on her situation,
//! which name no button). `action` is the next pose's action, `None` for "kept"; `aim` is the
//! `Aiming` rule, its first entry whose keys are all there giving the aim (none: the aim is
//! kept); `flags` are the next pose's parts and the change's effect; bit n of `beats` is set
//! where the row's Pad cell says it beats row n of its situation. The rows are in the spec's
//! order; their order decides nothing.

use super::Situation;
use clean_types::Action::{
    self, AgainstWall, Crouching, Falling, Jumping, Landing, Moonwalking, Morphing, Rolling,
    Running, Spinning, StandingUp, Still, TakingOff, Turning, Unmorphing, WallJumping,
};
use clean_types::Aim::{self, Down, DownForward, Forward, Up, UpForward};
use clean_types::Turn::{Fall, Ground, Jump, MoonwalkJump};
use clean_types::{Body, Facing, Gun, Pose};

/// A row of the chart.
pub type Row = (u32, Option<Action>, Aims, u16, u16);
/// An `Aiming` rule.
type Aims = &'static [(u32, Aim)];

// Keys: the buttons a row names held, forward and back by her facing at the frame's start.
/// Forward held.
const F: u32 = 1;
/// Back held.
const B: u32 = 2;
/// Jump held.
const J: u32 = 4;
/// Shoot held.
const S: u32 = 8;
/// Up held.
const U: u32 = 0x10;
/// Down held.
const D: u32 = 0x20;
/// Aim up held.
const AU: u32 = 0x40;
/// Aim down held.
const AD: u32 = 0x80;
/// Up or down held.
const UD: u32 = 0x100;
/// An aim button held.
const AIM: u32 = 0x200;
/// Up, down or an aim button held.
const UDA: u32 = 0x400;
/// Any button held.
const ANY: u32 = 0x800;
// The buttons a row names just pressed.
/// Down pressed.
const PD: u32 = 0x1000;
/// Jump pressed.
const PJ: u32 = 0x2000;
/// Up pressed.
const PU: u32 = 0x4000;
/// Back pressed.
const PB: u32 = 0x8000;
/// Shoot pressed.
const PS: u32 = 0x1_0000;
// Conditions on her situation.
/// Aiming `Down`.
const ADOWN: u32 = 0x2_0000;
/// Aimed `Forward`.
const AFWD: u32 = 0x4_0000;
/// Aimed `Up` or `DownForward`.
const AUPDF: u32 = 0x8_0000;
/// The gun stowed.
const STOWED: u32 = 0x10_0000;
/// The gun drawn.
const DRAWN: u32 = 0x20_0000;
/// Facing right (or the screen).
const RIGHT: u32 = 0x40_0000;
/// Facing left.
const LEFT: u32 = 0x80_0000;
/// On the ground.
const GROUND: u32 = 0x100_0000;
/// Off the ground.
const AIR: u32 = 0x200_0000;
/// The spring ball.
const SPRING: u32 = 0x400_0000;
/// Not the spring ball.
const PLAIN: u32 = 0x800_0000;
/// Jump the only button a row of hers names held.
const ONLYJ: u32 = 0x1000_0000;
/// Not landing.
const STANDS: u32 = 0x2000_0000;
/// The keys that are buttons a row names, which its rank counts.
pub const NAMED: u32 = 0x1_FFFF;
/// The keys that are buttons just pressed.
pub const PRESSED: u32 = 0x1_F000;

// Flags: the next pose's parts a row's cell names, then the change's effect.
/// Facing back.
pub const BACK: u16 = 1;
/// Gun drawn.
pub const DRAW: u16 = 2;
/// Gun as it was.
pub const ASWAS: u16 = 4;
/// Striding.
pub const STRIDE: u16 = 8;
/// `Jumping`'s take-off: she rises, follows the pad, and from a crouch unaimed is lifted.
pub const TAKEOFF: u16 = 0x10;
/// `Jumping`'s spin start or the spring ball's hop: she rises.
pub const RISE: u16 = 0x20;
/// `TurningRound`.
pub const TURN: u16 = 0x40;
/// She follows the pad again (a skid ends).
pub const ENDSKID: u16 = 0x80;
/// As running: the dash joins her speed and she carries on, braking.
pub const ASRUN: u16 = 0x100;
/// Dash 0.
pub const LOSEDASH: u16 = 0x200;
/// She stands at once: the body is upright, and pose fit's push keeps her feet where they were.
pub const STANDUP: u16 = 0x400;
/// Horizontal speed 0, dash 0, following the pad.
pub const HALT: u16 = 0x800;
/// `Moonfall` (`AnimationEnds` only).
pub const MOONFALL: u16 = 0x1000;

/// `PadChart`: the keys of her situation and the pad.
pub fn keys(sit: &Situation) -> u32 {
    let (pose, held, new) = (sit.pose, sit.pad, sit.pressed);
    let right = pose.facing != Facing::Left;
    let (fwd, back, back_new) = if right {
        (held.right, held.left, new.left)
    } else {
        (held.left, held.right, new.right)
    };
    let (ud, aim) = (held.up || held.down, held.aim_up || held.aim_down);
    let spring = pose.body == Body::SpringBall;
    // Each key in the order of its bit, from F (bit 0) to STANDS (bit 29).
    [
        fwd,
        back,
        held.jump,
        held.shoot,
        held.up,
        held.down,
        held.aim_up,
        held.aim_down,
        ud,
        aim,
        ud || aim,
        fwd || back || held.jump || held.shoot || held.run || ud || aim,
        new.down,
        new.jump,
        new.up,
        back_new,
        new.shoot,
        pose.aim == Down,
        pose.aim == Forward,
        matches!(pose.aim, Up | DownForward),
        pose.gun == Gun::Stowed,
        pose.gun == Gun::Drawn,
        right,
        !right,
        !sit.airborne,
        sit.airborne,
        spring,
        !spring,
        !(fwd || back || held.shoot || ud || aim),
        !matches!(pose.action, Landing(_)),
    ]
    .into_iter()
    .rev()
    .fold(0, |k, on| k << 1 | u32::from(on))
}

/// A row whose next pose is `action` aimed by `aims`, with no flags and beating no row.
const fn go(keys: u32, action: Action, aims: Aims) -> Row {
    (keys, Some(action), aims, 0, 0)
}

/// A row with flags, or beating other rows.
const fn r(keys: u32, action: Action, aims: Aims, flags: u16, beats: u16) -> Row {
    (keys, Some(action), aims, flags, beats)
}

/// A row that keeps her pose ("kept").
const fn kept(keys: u32, aims: Aims, flags: u16, beats: u16) -> Row {
    (keys, None, aims, flags, beats)
}

/// The rows of her situation (§7's "She is"); a changing body has none, so the pad has no say.
pub fn rows(p: Pose) -> &'static [Row] {
    let crouched = p.body == Body::Crouched;
    if super::changes_shape(p.action) {
        return &[];
    }
    if matches!(p.body, Body::Ball | Body::SpringBall) {
        return BALL;
    }
    match p.action {
        Turning(t) if crouched || p.aim == UpForward || matches!(t, Jump | Fall) => TURNING_KEPT,
        Turning(_) => TURNING_GROUND,
        Still if crouched => CROUCHED,
        Still | Landing(_) => STANDING,
        Running => RUNNING,
        TakingOff => TAKING_OFF,
        Jumping => STRAIGHT_JUMP,
        Spinning => SPINNING,
        Falling => FALLING,
        Moonwalking => MOONWALKING,
        WallJumping => WALL_JUMPING,
        AgainstWall => AGAINST_WALL,
        Rolling | Crouching | StandingUp | Morphing | Unmorphing => &[],
    }
}

// Aiming: the rules the rows name.
/// The aim kept.
const KEPT: Aims = &[];
/// `Forward`.
const FWD: Aims = &[(0, Forward)];
/// `Up`.
const UP: Aims = &[(0, Up)];
/// `Aiming`'s crouch rule.
const CROUCH: Aims = &[
    (AU | AD, Up),
    (AU, UpForward),
    (AD, DownForward),
    (0, Forward),
];
/// `Aiming`'s take-off rule.
const TAKE_OFF: Aims = &[(U, Up), (AU, UpForward), (AD, DownForward), (0, Forward)];
/// `Aiming`'s air rule.
const AIR_AIM: Aims = &[
    (F | U, UpForward),
    (F | D, DownForward),
    (U, Up),
    (D, Down),
    (AU, UpForward),
    (AD, DownForward),
    (0, Forward),
];
/// `UpForward` with aim up, else `DownForward`.
const AU_AD: Aims = &[(AU, UpForward), (0, DownForward)];
/// `UpForward` with up, else `DownForward`.
const U_D: Aims = &[(U, UpForward), (0, DownForward)];
/// `DownForward` with aim down, else `UpForward`.
const AD_AU: Aims = &[(AD, DownForward), (0, UpForward)];
/// `Up` with up, else `Down` with down, else `UpForward` with aim up, else `DownForward`.
const UP_DOWN: Aims = &[(U, Up), (D, Down), (AU, UpForward), (0, DownForward)];
/// `Up` with up, else `UpForward` with aim up, else `DownForward` with aim down, else `Down`.
const SPIN_AIM: Aims = &[(U, Up), (AU, UpForward), (AD, DownForward), (0, Down)];
/// The moonwalk's: `UpForward` with aim up, else `DownForward`; with both aim buttons,
/// `UpForward` facing left and `DownForward` facing right.
const MOON: Aims = &[
    (AU | AD | LEFT, UpForward),
    (AU | AD, DownForward),
    (AU, UpForward),
    (0, DownForward),
];
/// `Forward`, except landing, kept as she is.
const STANDS_FWD: Aims = &[(STANDS, Forward)];

/// Standing still, or landing.
const STANDING: &[Row] = &[
    go(PD, Crouching, CROUCH),
    r(PJ, TakingOff, TAKE_OFF, TAKEOFF, 0b1),
    r(B | S | AIM, Moonwalking, AD_AU, BACK, 0b1000),
    r(B | AU | AD, Turning(Ground), KEPT, BACK | TURN, 0),
    r(B | S, Moonwalking, FWD, BACK, 0),
    r(F | AIM, Running, AU_AD, 0, 0b100_0000),
    go(F | UD, Running, U_D),
    r(AU | AD, Still, UP, 0, 0b111_0000),
    go(F, Running, FWD),
    r(U, Still, UP, 0, 0b100_0000_0000),
    go(AIM, Still, AU_AD),
    r(B, Turning(Ground), KEPT, BACK | TURN, 0b110_0000_0000),
    kept(0, STANDS_FWD, ENDSKID, 0),
];

const RUNNING: &[Row] = &[
    r(PD, Crouching, FWD, 0, 0b10),
    r(PJ, Spinning, FWD, RISE, 0),
    r(F | S, Running, FWD, DRAW, 0),
    r(F | AIM, Running, AU_AD, 0, 0b1_0100),
    r(F | UD, Running, U_D, 0, 0b100),
    r(F, Running, FWD, ASWAS, 0),
    r(U, Still, UP, 0, 0b1000_0000),
    go(AIM, Still, AU_AD),
    r(B, Turning(Ground), KEPT, BACK | TURN, 0b1100_0000),
    r(0, Still, FWD, ASRUN, 0),
];

const TAKING_OFF: &[Row] = &[
    r(J | F, Jumping, FWD, STRIDE, 0),
    r(J | UDA, Jumping, UP_DOWN, 0, 0b1),
    r(J | B, Turning(Jump), KEPT, BACK | TURN, 0b10),
    r(S, Jumping, FWD, DRAW, 0),
    kept(0, KEPT, ASRUN, 0),
];

/// In a straight jump.
const STRAIGHT_JUMP: &[Row] = &[
    go(ADOWN | PD, Morphing, FWD),
    r(ADOWN | RIGHT | F | J | S, Jumping, FWD, DRAW, 0),
    r(F | UDA, Jumping, AIR_AIM, 0, 0b10),
    r(F, Jumping, FWD, STRIDE, 0b1_0000),
    r(S, Jumping, FWD, DRAW, 0b10_0000),
    go(STOWED | J, Jumping, FWD),
    r(UDA, Jumping, AIR_AIM, 0, 0b11_1000),
    r(B, Turning(Jump), KEPT, BACK | TURN, 0b1_0111_1000),
    kept(ADOWN | ONLYJ | J, KEPT, 0, 0b10_0000),
    kept(0, KEPT, ASRUN, 0),
];

const SPINNING: &[Row] = &[
    r(PS, Jumping, FWD, DRAW, 0),
    r(S | UDA, Jumping, UP_DOWN, 0, 0b100),
    kept(F | J, KEPT, 0, 0),
    r(UDA, Jumping, SPIN_AIM, 0, 0b11_0000),
    r(B, Spinning, FWD, BACK | TURN, 0),
    kept(F, KEPT, 0, 0),
    kept(0, KEPT, 0, 0),
];

/// Falling, upright. "Aiming `Down` or with the gun drawn, up or down held beats" the back row:
/// the two rows after it.
const FALLING: &[Row] = &[
    go(ADOWN | PD, Morphing, FWD),
    go(F | UD, Falling, U_D),
    r(S, Falling, FWD, DRAW, 0b1000),
    r(F, Falling, FWD, ASWAS, 0),
    r(UDA, Falling, AIR_AIM, 0, 0b1100),
    r(B, Turning(Fall), KEPT, BACK | TURN, 0b1_1100),
    r(ADOWN | UD, Falling, AIR_AIM, 0, 0b10_0000),
    r(DRAWN | UD, Falling, AIR_AIM, 0, 0b10_0000),
    kept(0, FWD, LOSEDASH, 0),
];

const CROUCHED: &[Row] = &[
    r(PD, Morphing, FWD, 0, 0b100),
    r(PU, StandingUp, CROUCH, 0, 0b1101),
    r(PJ, TakingOff, FWD, TAKEOFF, 0),
    r(PB, Turning(Ground), KEPT, BACK, 0b101),
    r(F | AIM, Still, FWD, STANDUP, 0),
    r(AU | AD, Still, UP, 0, 0b1_0000),
    r(F, Still, FWD, STANDUP, 0),
    go(AIM, Still, AU_AD),
    kept(0, FWD, ENDSKID, 0),
];

/// A ball, on the ground or in the air.
const BALL: &[Row] = &[
    r(PJ | SPRING | GROUND, Jumping, FWD, RISE, 0),
    r(PU, Unmorphing, FWD, 0, 0b1),
    r(PJ | PLAIN, Unmorphing, FWD, 0, 0b1),
    go(F | GROUND, Rolling, FWD),
    r(B | GROUND, Rolling, FWD, BACK | TURN, 0),
    kept(F | AIR, KEPT, 0, 0),
    kept(B | AIR, KEPT, BACK | TURN, 0),
    r(GROUND, Still, FWD, HALT, 0),
    kept(AIR | PLAIN, KEPT, ASRUN, 0),
    kept(AIR | SPRING, KEPT, HALT, 0),
];

/// Turning on the ground or from the moonwalk's jump, not aimed `UpForward`.
const TURNING_GROUND: &[Row] = &[
    r(AUPDF | PJ | F, Spinning, FWD, RISE, 0),
    r(PJ, TakingOff, FWD, TAKEOFF, 0),
    r(AFWD | F | J, Spinning, FWD, RISE, 0b10),
    kept(F, KEPT, 0, 0),
    kept(0, KEPT, ENDSKID, 0),
];

/// Turning in a jump or a fall, crouched turning, or turning on the ground or from the
/// moonwalk's jump aimed `UpForward`.
const TURNING_KEPT: &[Row] = &[kept(ANY, KEPT, 0, 0), kept(0, KEPT, ENDSKID, 0)];

/// Moonwalking: forward is the way she walks, back the way she looks.
const MOONWALKING: &[Row] = &[
    r(PD, Crouching, FWD, BACK, 0b10),
    r(PJ, Turning(MoonwalkJump), KEPT, BACK | TURN, 0),
    go(F | S | AIM, Moonwalking, MOON),
    go(F | S, Moonwalking, FWD),
    r(B, Running, FWD, BACK, 0),
    r(F, Turning(Ground), KEPT, BACK | TURN, 0),
    r(0, Still, FWD, BACK | ENDSKID, 0),
];

const WALL_JUMPING: &[Row] = &[
    go(PD, Morphing, FWD),
    r(S, Jumping, FWD, DRAW, 0b1_0000),
    r(AIM, Jumping, AU_AD, 0, 0b1_0010),
    r(B, Spinning, FWD, BACK | TURN, 0b1_0110),
    kept(J, KEPT, 0, 0),
    r(0, Spinning, FWD, HALT, 0),
];

const AGAINST_WALL: &[Row] = &[
    go(PD, Crouching, FWD),
    r(PJ, TakingOff, FWD, TAKEOFF, 0b1),
    r(B | AIM, Moonwalking, AD_AU, BACK, 0),
    r(F | UD, Running, U_D, 0, 0b1),
    go(F, Running, FWD),
    r(U, Still, UP, 0, 0b1100_0000),
    r(AIM, Still, AU_AD, 0, 0b1001_0000),
    r(B, Turning(Ground), KEPT, BACK | TURN, 0),
    kept(0, FWD, ENDSKID, 0),
];
