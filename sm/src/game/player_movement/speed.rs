//! Owned horizontal speed and vertical movement shared by player movement modes.
#![forbid(unsafe_code)]
use super::{content::SpeedRecord, effects::Host};
use crate::game::{
    liquid_surface::LiquidAtHeight,
    player_collision::operations as collision,
    player_transitions::{self as transitions, content::AnimationTable},
    session::{FixedMotion, Session},
    visual_host::Record,
};

pub(super) fn value(motion: &FixedMotion) -> u32 {
    (u32::from(motion.whole) << 16) | u32::from(motion.fraction)
}
pub(super) fn store(motion: &mut FixedMotion, value: u32) {
    motion.fraction = value as u16;
    motion.whole = (value >> 16) as u16;
}
pub(crate) fn record_camera_distance(s: &mut Session, axis: collision::Axis) {
    let p = &mut s.player;
    let (whole, fraction, before) = match axis {
        collision::Axis::X => (p.x, p.x_fraction, p.previous_position.x),
        collision::Axis::Y => (p.y, p.y_fraction, p.previous_position.y),
    };
    let current = (u32::from(whole) << 16) | u32::from(fraction);
    let previous = (u32::from(before.whole) << 16) | u32::from(before.fraction);
    // Direction uses the whole-word difference, even when fractions borrow.
    let distance = if (whole.wrapping_sub(before.whole) as i16) >= 0 {
        current.wrapping_sub(previous)
    } else {
        previous.wrapping_sub(current)
    };
    store(
        &mut p.camera_distance[axis as usize],
        distance.wrapping_add(1 << 16),
    );
}
fn environment(h: &mut impl Host) -> LiquidAtHeight {
    if h.session().inventory.equipment.active & 0x20 != 0 {
        LiquidAtHeight::None
    } else {
        let bottom = transitions::loaded_bottom(h);
        transitions::liquid_at_height(h, bottom)
    }
}
pub(crate) fn normal_speed_profile(h: &mut impl Host) -> SpeedRecord {
    let liquid = environment(h);
    if let Some(table) = h.liquid_speed_table(liquid) {
        h.session_mut().player.contact.movement_speed_table = table;
    }
    let p = &h.session().player;
    SpeedRecord::Selected(
        p.contact
            .movement_speed_table
            .wrapping_add(12 * (p.movement_and_facing >> 8)),
    )
}
pub(crate) fn grapple_speed_profile(h: &mut impl Host) -> SpeedRecord {
    match environment(h) {
        LiquidAtHeight::None => SpeedRecord::GrappleAir,
        LiquidAtHeight::Water => SpeedRecord::GrappleWater,
        LiquidAtHeight::LavaAcid => SpeedRecord::GrappleLava,
    }
}
pub(crate) fn update_vertical_acceleration(h: &mut impl Host) {
    let fraction = match environment(h) {
        LiquidAtHeight::None => 0x1c00,
        LiquidAtHeight::Water => 0x800,
        LiquidAtHeight::LavaAcid => 0x900,
    };
    h.session_mut().player.contact.y_acceleration.fraction = fraction;
    h.session_mut().player.contact.y_acceleration.whole = 0;
}
pub(crate) fn set_liquid_physics(h: &mut impl Host) {
    let bottom = transitions::loaded_bottom(h);
    let kind = match (h.inputs().transitions.fx_kind & 0xf) >> 1 {
        1 | 2
            if h.inputs().lava_y & 0x8000 == 0
                && h.inputs().lava_y.wrapping_sub(bottom) & 0x8000 != 0 =>
        {
            2
        }
        3 if h.inputs().fx_y & 0x8000 == 0
            && h.inputs().fx_y.wrapping_sub(bottom) & 0x8000 != 0
            && h.session().room.liquid_options & 4 == 0 =>
        {
            1
        }
        _ => 0,
    };
    h.session_mut().player.transitions.liquid_physics = kind;
}
pub(crate) fn update_extra_run(h: &mut impl Host) {
    let running = environment(h) == LiquidAtHeight::None
        && h.session().player.movement_and_facing >> 8 == 1
        && h.inputs().run_button & h.inputs().held != 0;
    if running {
        let boosted = h.session().inventory.equipment.active & 0x2000 != 0;
        if h.session().player.contact.momentum == 0 {
            h.session_mut().player.contact.momentum = 1;
            if boosted {
                h.session_mut().player.palette.timer = 1;
                h.session_mut().player.palette_frame = 0;
                let table = h.animation_table(AnimationTable::BoosterStageDurations);
                h.session_mut().player.contact.speed_boost = table.native_word(0);
            } else {
                h.session_mut().player.contact.speed_boost = 0;
            }
        }
        let maximum = if boosted { 7 } else { 2 };
        let extra = &mut h.session_mut().player.contact.extra_run;
        if (extra.whole as i16).wrapping_sub(maximum) >= 0 && (extra.fraction as i16) >= 0 {
            extra.whole = maximum as u16;
            extra.fraction = 0;
        } else {
            let next = value(extra).wrapping_add(0x1000);
            store(extra, next);
        }
    } else if h.session().player.contact.momentum == 0 {
        h.session_mut().player.contact.extra_run.whole = 0;
        h.session_mut().player.contact.extra_run.fraction = 0;
    }
    if h.session().player.contact.speed_boost & 0xff00 == 0x400 {
        h.session_mut().player.contact.contact_attack = 1;
    }
}
fn greater_quirked(whole: u16, fraction: u16, maximum: u16, maximum_fraction: u16) -> bool {
    (whole as i16).wrapping_sub(maximum as i16) >= 0
        && (whole != maximum
            || (fraction as i16).wrapping_sub(maximum_fraction as i16) >= 0
                && fraction != maximum_fraction)
}
fn calculate_base(h: &mut impl Host, profile: SpeedRecord, air: bool) -> (bool, i32) {
    // Preserve the capture before mode selection and only read the chosen fields.
    let record = h.speed_record(profile);
    let mode = h.session().player.contact.acceleration_mode;
    let mut capped = false;
    if if air { mode & 1 != 0 } else { mode != 0 } {
        let divisor = h.session().player.contact.deceleration as u8;
        let delta = if divisor != 0 {
            let whole = (u16::from(divisor) * u16::from(record.native_word(8) as u8)) >> 8;
            // The old byte! used the physical second byte, not a native-word shift.
            let fraction = u16::from(divisor) * u16::from(record.byte(11));
            (u32::from(whole) << 16) | u32::from(fraction)
        } else {
            (u32::from(record.native_word(8)) << 16) | u32::from(record.native_word(10))
        };
        let c = &mut h.session_mut().player.contact;
        let next = value(&c.base_run).wrapping_add(-(delta as i32) as u32);
        store(&mut c.base_run, next);
        if (c.base_run.whole as i16) < 0 {
            c.base_run.whole = 0;
            c.base_run.fraction = 0;
            c.acceleration_mode = 0;
        }
    } else {
        let delta = (u32::from(record.native_word(0)) << 16) | u32::from(record.native_word(2));
        let c = &mut h.session_mut().player.contact;
        let next = value(&c.base_run).wrapping_add(delta);
        store(&mut c.base_run, next);
        if greater_quirked(
            c.base_run.whole,
            c.base_run.fraction,
            record.native_word(4),
            record.native_word(6),
        ) {
            c.base_run.whole = record.native_word(4);
            c.base_run.fraction = record.native_word(6);
            capped = true;
        }
    }
    (capped, value(&h.session().player.contact.base_run) as i32)
}
pub(crate) fn base_speed(h: &mut impl Host, profile: SpeedRecord) -> i32 {
    calculate_base(h, profile, false).1
}
pub(crate) fn base_speed_air(h: &mut impl Host) -> (bool, i32) {
    let profile = normal_speed_profile(h);
    calculate_base(h, profile, true)
}
pub(crate) fn horizontal_air<const MOVE_AT_REST: bool>(
    h: &impl Host,
    base: i32,
) -> collision::HorizontalTransition {
    if h.session().player.contact.acceleration_mode != 0 || h.inputs().held & 0x300 != 0 {
        collision::HorizontalTransition::Move(collision::HorizontalRequest {
            base,
            direction: collision::HorizontalDirection::FromFacingAndAcceleration,
        })
    } else if MOVE_AT_REST {
        collision::HorizontalTransition::ClearAndMoveZero
    } else {
        collision::HorizontalTransition::ClearAndSkip
    }
}
pub(crate) fn handle_movement_x(h: &mut impl Host) {
    update_extra_run(h);
    let profile = normal_speed_profile(h);
    let base = base_speed(h, profile);
    collision::horizontal(
        h,
        collision::HorizontalRequest {
            base,
            direction: collision::HorizontalDirection::FromFacingAndAcceleration,
        },
    );
}
pub(crate) fn no_base_x(h: &mut impl Host) {
    collision::horizontal(
        h,
        collision::HorizontalRequest {
            base: 0,
            direction: collision::HorizontalDirection::FromFacingAndAcceleration,
        },
    );
}
pub(crate) fn check_jump_release(h: &mut impl Host) {
    if h.session().player.contact.y_direction == 1
        && (h.inputs().jump_button & h.inputs().held == 0
            || h.session().player.contact.y_speed.whole & 0x8000 != 0)
    {
        let c = &mut h.session_mut().player.contact;
        c.y_speed.fraction = 0;
        c.y_speed.whole = 0;
        c.y_direction = 2;
    }
}
pub(crate) fn check_start_falling(h: &mut impl Host) {
    let c = &mut h.session_mut().player.contact;
    if c.y_direction == 1 && c.y_speed.whole & 0x8000 != 0 {
        c.y_speed.fraction = 0;
        c.y_speed.whole = 0;
        c.y_direction = 2;
    }
}
fn displace_y(h: &mut impl Host, amount: i32) {
    collision::displace(
        h,
        collision::DisplacementRequest::Vertical {
            direction: if amount < 0 {
                collision::VerticalDirection::Up
            } else {
                collision::VerticalDirection::Down
            },
            amount,
        },
    );
}
pub(crate) fn move_y_with_speed(h: &mut impl Host) {
    let c = &mut h.session_mut().player.contact;
    // Displacement uses the old speed; acceleration prepares the next movement.
    let mut amount = value(&c.y_speed) as i32;
    if c.y_direction == 2 {
        if c.y_speed.whole != 5 {
            let next = value(&c.y_speed).wrapping_add(value(&c.y_acceleration));
            store(&mut c.y_speed, next);
        }
    } else {
        let next = value(&c.y_speed).wrapping_add(-(value(&c.y_acceleration) as i32) as u32);
        store(&mut c.y_speed, next);
    }
    if c.y_direction != 2 {
        amount = -amount;
    }
    amount = (amount as u32).wrapping_add(value(&c.external_y)) as i32;
    displace_y(h, amount);
}
pub(crate) fn check_and_move_y(h: &mut impl Host) -> bool {
    if h.session().player.contact.y_direction == 0 {
        return false;
    }
    check_start_falling(h);
    move_y_with_speed(h);
    true
}
pub(crate) fn no_speed_y(h: &mut impl Host) {
    let p = &h.session().player;
    let mut amount = value(&p.contact.external_y) as i32;
    if amount == 0 {
        if p.collision.slope_adjusted == 0 {
            amount = value(&p.contact.total_x) as i32;
        }
        // Ground adhesion always selects down, even for a negative total-X value.
        collision::displace(
            h,
            collision::DisplacementRequest::Vertical {
                direction: collision::VerticalDirection::Down,
                amount: amount + (1 << 16),
            },
        );
    } else {
        let direction = if amount < 0 {
            collision::VerticalDirection::Up
        } else {
            collision::VerticalDirection::Down
        };
        if amount >= 0 {
            amount += 1 << 16;
        }
        collision::displace(
            h,
            collision::DisplacementRequest::Vertical { direction, amount },
        );
    }
}
pub(crate) fn move_extra_y(h: &mut impl Host) {
    let amount = value(&h.session().player.contact.external_y) as i32;
    if amount != 0 {
        let direction = if amount < 0 {
            collision::VerticalDirection::Up
        } else {
            collision::VerticalDirection::Down
        };
        let amount = if amount < 0 {
            amount
        } else {
            amount + (1 << 16)
        };
        collision::displace(
            h,
            collision::DisplacementRequest::Vertical { direction, amount },
        );
    }
}
pub(crate) fn jumping(h: &mut impl Host) {
    h.probe("samus.move.jumping");
    update_extra_run(h);
    let pose = h.session().player.pose;
    if matches!(pose, 0x4b | 0x4c)
        || pose.wrapping_sub(0x55) & 0x8000 == 0 && pose.wrapping_sub(0x5b) & 0x8000 != 0
    {
        h.session_mut().player.contact.acceleration_mode = 0;
        no_base_x(h);
        move_extra_y(h);
        return;
    }
    check_jump_release(h);
    let (_, base) = base_speed_air(h);
    let request = if h.session().player.movement_and_facing >> 8 == 20 {
        if h.session().player.contact.acceleration_mode == 0 {
            h.session_mut().player.contact.acceleration_mode = 2;
        }
        collision::HorizontalTransition::Move(collision::HorizontalRequest {
            base,
            direction: collision::HorizontalDirection::FromFacingAndAcceleration,
        })
    } else {
        horizontal_air::<false>(h, base)
    };
    collision::horizontal_control(h, request);
    move_y_with_speed(h);
}
pub(crate) fn spin_jumping(h: &mut impl Host) {
    h.probe("samus.move.spinning");
    update_extra_run(h);
    check_jump_release(h);
    let (capped, base) = base_speed_air(h);
    let p = &h.session().player;
    let moving = capped
        || p.contact.acceleration_mode == 1
        || h.inputs().held
            & (if p.movement_and_facing as u8 == 4 {
                0x200
            } else {
                0x100
            })
            != 0;
    let request = if moving {
        if h.session().player.contact.acceleration_mode == 0 {
            h.session_mut().player.contact.acceleration_mode = 2;
        }
        collision::HorizontalTransition::Move(collision::HorizontalRequest {
            base,
            direction: collision::HorizontalDirection::FromFacingAndAcceleration,
        })
    } else {
        collision::HorizontalTransition::ClearAccelerationAndMoveZero
    };
    collision::horizontal_control(h, request);
    if !collision::jump_admission(
        h,
        collision::JumpRequest {
            kind: collision::JumpKind::Wall,
            amount: 8 << 16,
        },
    ) {
        move_y_with_speed(h);
    }
}
pub(crate) fn falling(h: &mut impl Host) {
    h.probe("samus.move.falling");
    update_extra_run(h);
    let (_, base) = base_speed_air(h);
    let request = horizontal_air::<false>(h, base);
    collision::horizontal_control(h, request);
    check_start_falling(h);
    move_y_with_speed(h);
}
pub(crate) fn morphed_falling(h: &mut impl Host) {
    h.probe("samus.ball.falling");
    let (_, base) = base_speed_air(h);
    let request = horizontal_air::<true>(h, base);
    collision::horizontal_control(h, request);
    check_start_falling(h);
    move_y_with_speed(h);
}
pub(crate) fn morphed_bouncing(h: &mut impl Host) {
    h.probe("samus.ball.bouncing");
    let (_, base) = base_speed_air(h);
    let request = horizontal_air::<true>(h, base);
    collision::horizontal_control(h, request);
    if h.session().player.transitions.knockback == 0 {
        if value(&h.session().player.contact.external_y) != 0 {
            let c = &mut h.session_mut().player.contact;
            c.y_direction = 2;
            c.y_speed.fraction = 0;
            c.y_speed.whole = 0;
            move_extra_y(h);
        } else {
            check_start_falling(h);
            move_y_with_speed(h);
        }
    }
}
pub(crate) fn bomb_rising_x(h: &mut impl Host) {
    h.probe("samus.bomb.rising-x");
    let (base, left) = if h.session().player.transitions.knockback != 0 {
        let profile = normal_speed_profile(h);
        let base = base_speed(h, profile);
        (base, h.session().player.knockback_x_direction == 0)
    } else {
        let base = base_speed(h, SpeedRecord::Bomb);
        (
            base,
            h.session().player.transitions.bomb_jump.to_raw() as u8 == 1,
        )
    };
    collision::horizontal(
        h,
        collision::HorizontalRequest {
            base,
            direction: if left {
                collision::HorizontalDirection::Left
            } else {
                collision::HorizontalDirection::Right
            },
        },
    );
}
