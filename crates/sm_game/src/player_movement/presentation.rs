//! Shared footstep particles and ordered speed-echo publication.
#![forbid(unsafe_code)]
use super::{
    content::{Content, FootstepRegion, FootstepRule},
    effects::{EchoAxis, Host},
};
use {
    crate::player_transitions as transitions,
    crate::{liquid_surface::LiquidAtHeight, projectile_geometry::sine_offset, session::Session},
};

// Query each full-width angle lazily in the original threshold order.
pub fn xray_animation_frame(facing: u8, mut angle: impl FnMut() -> u16) -> u16 {
    let first = if facing == 4 { 153 } else { 25 };
    let band = (0..4)
        .find(|&band| angle().wrapping_sub(first + 25 * band) & 0x8000 != 0)
        .unwrap_or(4);
    if facing == 4 { 4 - band } else { band }
}

pub fn footsteps(h: &mut impl Host) {
    let region = Content::footstep_region(h);
    let rule = if region == FootstepRegion::Crateria && h.inputs().cinematic == 0 {
        Content::footstep_room_rule(h)
    } else if region == FootstepRegion::Sand {
        FootstepRule::Sand
    } else {
        FootstepRule::Default
    };
    let sand = match rule {
        FootstepRule::Sand => true,
        FootstepRule::SandWhenFxTen => h.inputs().transitions.fx_kind == 10,
        FootstepRule::SandAtOrBelow944 => h.session().player.y.wrapping_sub(944) & 0x8000 == 0,
        FootstepRule::Default => false,
    };
    if !sand && h.session().player.contact.speed_boost & 0xff00 != 1024 {
        return;
    }
    let bottom = transitions::loaded_bottom(h);
    if transitions::liquid_at_height(h, bottom) != LiquidAtHeight::None {
        return;
    }
    // Sand and boost dust differ only in particle kind; their positions/timing agree.
    let s = h.session_mut();
    if s.player.movement_and_facing as u8 == 4 {
        s.presentation.atmospheric_mut().slots[0].x = s.player.x.wrapping_sub(12);
        s.presentation.atmospheric_mut().slots[1].x = s.player.x.wrapping_add(8);
    } else {
        s.presentation.atmospheric_mut().slots[0].x = s.player.x.wrapping_add(12);
        s.presentation.atmospheric_mut().slots[1].x = s.player.x.wrapping_sub(8);
    }
    s.presentation.atmospheric_mut().slots[0].y = s.player.y.wrapping_add(16);
    s.presentation.atmospheric_mut().slots[1].y = s.player.y.wrapping_add(16);
    let kind = if sand { 256 } else { 1792 };
    s.presentation.atmospheric_mut().slots[0].frame_and_kind = kind;
    s.presentation.atmospheric_mut().slots[1].frame_and_kind = kind;
    s.presentation.atmospheric_mut().slots[0].remaining = 0x8002;
    s.presentation.atmospheric_mut().slots[1].remaining = 3;
}

pub fn record_echo(h: &mut impl Host) {
    if h.session().player.contact.speed_boost & 0xff00 == 1024
        && h.session().projectiles.echoes.index & 0x8000 == 0
        && h.inputs().transitions.game_time_frames & 3 == 0
    {
        let original = h.session().projectiles.echoes.index;
        let index = usize::from(h.session().projectiles.echoes.index >> 1);
        let x = h.session().player.x;
        if index < 4 {
            h.session_mut().projectiles.echoes.x[index] = x;
        } else {
            h.write_echo_coordinate(EchoAxis::X, index, x);
        }
        // Exceptional X destinations may alias player Y. Read it after publishing X.
        let y = h.session().player.y;
        if index < 4 {
            h.session_mut().projectiles.echoes.y[index] = y;
        } else {
            h.write_echo_coordinate(EchoAxis::Y, index, y);
        }
        let mut next = original.wrapping_add(2);
        if (next as i16).wrapping_sub(4) >= 0 {
            next = 0;
        }
        h.session_mut().projectiles.echoes.index = next;
    }
}

pub fn place_shinespark_crash(s: &mut Session) {
    for slot in [1, 0] {
        let offset = sine_offset::<true>(
            s.projectiles.echoes.x_speed[slot],
            u16::from(s.projectiles.echoes.index as u8),
        );
        s.projectiles.echoes.x[slot] = offset.x.wrapping_add(s.player.x);
        s.projectiles.echoes.y[slot] = offset.y.wrapping_add(s.player.y);
    }
}

pub fn create_shinespark_echoes(h: &mut impl Host) {
    const ANGLES: [u8; 12] = [
        0, 0x80, 0, 0x80, 0x40, 0xc0, 0x40, 0xc0, 0xe0, 0x60, 0x20, 0xa0,
    ];
    h.session_mut().projectiles.echoes.index = 0;
    if h.session().projectiles.count.wrapping_sub(5) & 0x8000 == 0 {
        return;
    }
    let first = if h.session().projectiles.count.wrapping_sub(4) & 0x8000 != 0 {
        3
    } else {
        4
    };
    for slot in first..=4 {
        let s = h.session_mut();
        s.projectiles.count = s.projectiles.count.wrapping_add(1);
        s.projectiles.echoes.x_speed[slot - 1] = 64;
        s.projectiles.echoes.x[slot - 1] = s.player.x;
        s.projectiles.echoes.y[slot - 1] = s.player.y;
        s.projectiles.shots[slot].kind = 0x8029;
        // Captured slots 6 then 8 retain separate initializer publication intervals.
        h.initialize_shinespark_echo((slot * 2) as u16);
        h.session_mut().projectiles.shots[slot].motion =
            crate::player_projectiles::handlers::SPEED_ECHO;
        // Reacquire pose, wrap its base to u16, then widen before adding lane 1.
        let index =
            usize::from(h.session().player.pose.wrapping_sub(201).wrapping_mul(2)) + (slot - 3);
        h.session_mut().projectiles.shots[slot].variable = u16::from(ANGLES[index]);
        h.session_mut().projectiles.shots[slot].x_speed = 0;
    }
}
