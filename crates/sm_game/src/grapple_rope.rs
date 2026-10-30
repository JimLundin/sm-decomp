//! Rope animation and placement on the shared player state and existing drawing edge.
#![forbid(unsafe_code)]
use {
    crate::actors::api::angle_from_delta,
    crate::{
        player_projectiles::host::{Drawing, Host},
        projectile_geometry::SINE_COSINE,
        session_access::SessionAccess,
        visual_host::{Content, Record},
    },
};
pub fn start(h: &mut impl SessionAccess) {
    for group in (0..4).rev() {
        let player = &mut h.session_mut().player;
        for lane in (0..4).rev() {
            player.grapple_animation_program[group * 4 + lane] = 0xb18b + (lane as u16) * 4;
        }
        for lane in (0..4).rev() {
            player.grapple_animation_delay[group * 4 + lane] = if lane == 3 {
                1
            } else {
                u16::from_le_bytes(1u16.to_ne_bytes())
            };
        }
    }
}
pub fn draw(h: &mut (impl Host + Content)) {
    let endpoint = h.session().player.grapple_endpoint;
    let angle = usize::from(angle_from_delta(
        endpoint.x.wrapping_sub(h.inputs().grapple_x),
        endpoint.y.wrapping_sub(h.inputs().grapple_y),
    ));
    let dx = (i32::from(SINE_COSINE[angle + 64]) << 11) as u32;
    let dy = (i32::from(SINE_COSINE[angle]) << 11) as u32;
    let high = h.session().player.grapple_angle >> 8;
    let mut tile = (high & 0x80) >> 1;
    tile = (tile | 2 * ((high ^ tile) & 0x40 ^ 0x40)) << 8;
    let mut x =
        ((i32::from(h.inputs().grapple_x) - i32::from(h.inputs().camera_x) - 4) << 16) as u32;
    let mut y =
        ((i32::from(h.inputs().grapple_y) - i32::from(h.inputs().camera_y) - 4) << 16) as u32;
    let length = h.session().player.grapple_length;
    if length & 0x8000 != 0 {
        return;
    }
    let mut remaining = i32::from(length / 8 & 15) - 1;
    let mut slot = 15;
    loop {
        let delay = h.session().player.grapple_animation_delay[slot];
        h.session_mut().player.grapple_animation_delay[slot] = delay.wrapping_sub(1);
        if delay == 1 {
            let mut cursor = h.session().player.grapple_animation_program[slot];
            loop {
                let record = h.legacy_native_record(0x94, cursor);
                if record.native_word(0) & 0x8000 == 0 {
                    h.session_mut().player.grapple_animation_delay[slot] = record.native_word(0);
                    h.session_mut().player.grapple_animation_program[slot] = cursor.wrapping_add(4);
                    break;
                }
                cursor = if record.native_word(0) == 0xb0f4 {
                    h.legacy_native_record(0x94, cursor.wrapping_add(2))
                        .native_word(0)
                } else {
                    u16::from(h.unknown())
                };
            }
        }
        if (x | y) & 0xff000000 != 0 {
            break;
        }
        let cursor = h.session().player.grapple_animation_program[slot].wrapping_sub(2);
        let character = h.legacy_native_record(0x94, cursor).native_word(0);
        h.draw(Drawing::GrappleSegment {
            x: (x >> 16) as u16,
            y: (y >> 16) as u16,
            tile: tile | character,
        });
        x = x.wrapping_add(dx);
        y = y.wrapping_add(dy);
        remaining -= 1;
        if remaining < 0 {
            break;
        }
        slot -= 1;
    }
    if matches!(h.session().player.pose, 0xb2 | 0xb3)
        || h.session()
            .player
            .grapple_endpoint
            .y
            .wrapping_sub(h.inputs().camera_y)
            & 0xff00
            == 0
    {
        h.draw(Drawing::GrappleEnd);
    }
}
