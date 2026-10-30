//! The grapple swing's rules, checked against the $7E RAM map's description of
//! bank $9B's swing words. Expected values are worked out by hand from that
//! description, not read back from the code:
//! - $0D26 angular velocity: unit pi/8000h, positive clockwise, capped at +-480h;
//! - $0D28 acceleration due to the swing's angle: up-right 6, down-right 24,
//!   up-left -6, down-left -24, halved with liquid physics;
//! - $0D2C deceleration due to the angle, applied only while Samus rises:
//!   up-right 1, down-right 5, up-left -1, down-left -5;
//! - $0D2E extra angular velocity, which does not take part in the cap;
//! - $0CFE beam length: a connected beam is at most 3Fh long.
//!
//! The angle ($0CFA) is a 16-bit turn: up-right is $0000-$3FFF, down-right
//! $4000-$7FFF, down-left $8000-$BFFF and up-left $C000-$FFFF; straight down
//! ($80xx) has no angular acceleration.
use sm_game::grapple_swing::{accelerate, apply_gravity, length_target};
use sm_game::session::Player;

fn at_angle(angle: u16) -> Player {
    Player {
        grapple_angle: angle,
        ..Player::default()
    }
}

fn changing_length(length: u16, delta: u16) -> Player {
    let mut player = Player {
        grapple_length: length,
        ..Player::default()
    };
    player.swing.length_delta = delta;
    player
}

fn signed(value: i16) -> u16 {
    value as u16
}

#[test]
fn gravity_accelerates_by_quadrant() {
    for (angle, acceleration, deceleration) in [
        (0x2000, 6, 1),
        (0x6000, 24, 5),
        (0xa000, -24, -5),
        (0xe000, -6, -1),
    ] {
        let mut player = at_angle(angle);
        apply_gravity(&mut player);
        assert_eq!(
            player.swing.angle_acceleration,
            signed(acceleration),
            "{angle:#x}"
        );
        assert_eq!(
            player.swing.rising_deceleration,
            signed(deceleration),
            "{angle:#x}"
        );
    }
}

#[test]
fn liquid_halves_the_angle_acceleration_only() {
    for (angle, acceleration, deceleration) in [(0x2000, 3, 1), (0x6000, 12, 5), (0xa000, -12, -5)]
    {
        let mut player = at_angle(angle);
        player.grapple_flags = 1;
        apply_gravity(&mut player);
        assert_eq!(
            player.swing.angle_acceleration,
            signed(acceleration),
            "{angle:#x}"
        );
        assert_eq!(
            player.swing.rising_deceleration,
            signed(deceleration),
            "{angle:#x}"
        );
    }
}

#[test]
fn straight_down_has_no_acceleration_and_settles_a_slow_swing() {
    let mut player = at_angle(0x8040);
    player.swing.angle_acceleration = 24;
    player.swing.rising_deceleration = 5;
    player.swing.angular_velocity = signed(-0x80);
    apply_gravity(&mut player);
    assert_eq!(player.swing.angle_acceleration, 0);
    assert_eq!(player.swing.rising_deceleration, 0);
    assert_eq!(
        player.swing.angular_velocity, 0,
        "under 100h, the swing stops"
    );

    let mut player = at_angle(0x8040);
    player.swing.angular_velocity = 0x180;
    apply_gravity(&mut player);
    assert_eq!(
        player.swing.angular_velocity, 0x180,
        "100h or more keeps swinging"
    );
}

#[test]
fn angular_velocity_is_capped_at_480h_both_ways() {
    // 470h + Ch input + 6 angle = 482h, capped to 480h.
    let mut player = at_angle(0x2000);
    player.swing.angular_velocity = 0x470;
    player.swing.input_acceleration = 0xc;
    player.swing.angle_acceleration = 6;
    accelerate(&mut player);
    assert_eq!(player.swing.angular_velocity, 0x480);

    // -47Ch - Ch - 6 = -48Eh; moving against the angle adds the -1
    // deceleration first (-48Fh), then the cap gives -480h.
    let mut player = at_angle(0x2000);
    player.swing.angular_velocity = signed(-0x47c);
    player.swing.input_acceleration = signed(-0xc);
    player.swing.angle_acceleration = signed(-6);
    player.swing.rising_deceleration = signed(-1);
    accelerate(&mut player);
    assert_eq!(player.swing.angular_velocity, signed(-0x480));
}

#[test]
fn rising_deceleration_applies_only_against_the_angle() {
    // Up-right (angle sign +), moving anticlockwise (velocity sign -): rising.
    // -100h + 6 = -FAh, then +1 deceleration = -F9h.
    let mut player = at_angle(0x2000);
    player.swing.angular_velocity = signed(-0x100);
    player.swing.angle_acceleration = 6;
    player.swing.rising_deceleration = 1;
    accelerate(&mut player);
    assert_eq!(player.swing.angular_velocity, signed(-0xf9));

    // Moving with the angle's sign: no deceleration. 100h + 6 = 106h.
    let mut player = at_angle(0x2000);
    player.swing.angular_velocity = 0x100;
    player.swing.angle_acceleration = 6;
    player.swing.rising_deceleration = 1;
    accelerate(&mut player);
    assert_eq!(player.swing.angular_velocity, 0x106);
}

#[test]
fn extra_velocity_is_outside_the_cap() {
    let mut player = at_angle(0x6000);
    player.swing.angular_velocity = 0x47f;
    player.swing.angle_acceleration = 24;
    player.swing.extra_angular_velocity = 0x300;
    accelerate(&mut player);
    assert_eq!(player.swing.angular_velocity, 0x480);
    assert_eq!(
        player.swing.extra_angular_velocity, 0x300,
        "extra velocity is untouched"
    );

    // 200h + 24 = 218h is under the cap; with 300h extra the swing turns at
    // 518h, but the extra does not count toward the cap.
    let mut player = at_angle(0x6000);
    player.swing.angular_velocity = 0x200;
    player.swing.angle_acceleration = 24;
    player.swing.extra_angular_velocity = 0x300;
    accelerate(&mut player);
    assert_eq!(player.swing.angular_velocity, 0x218);
}

#[test]
fn length_changes_stop_at_8_and_3fh() {
    // Lengthening 3Ch by 4 would pass 3Fh: it stops at 3Fh and the change ends.
    let mut player = changing_length(0x3c, 4);
    assert_eq!(length_target(&mut player), (0x3f, false));
    assert_eq!(player.swing.length_delta, 0);

    // Reaching 3Fh exactly also ends the change: 3Bh + 4 = 3Fh.
    let mut player = changing_length(0x3b, 4);
    assert_eq!(length_target(&mut player), (0x3f, false));
    assert_eq!(player.swing.length_delta, 0);

    // Shortening 9 by 2 would pass 8: it stops at 8.
    let mut player = changing_length(9, signed(-2));
    assert_eq!(length_target(&mut player), (8, true));
    assert_eq!(player.swing.length_delta, 0);

    // Inside the range the change continues: 20h - 2 = 1Eh.
    let mut player = changing_length(0x20, signed(-2));
    assert_eq!(length_target(&mut player), (0x1e, true));
    assert_eq!(player.swing.length_delta, signed(-2));
}
