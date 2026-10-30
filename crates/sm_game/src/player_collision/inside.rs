//! Sample the player's bottom, center and top for terrain effects.
use super::{
    effects::Host,
    geometry::{behavior as room_bts, level as room_level, redirect},
    operations::Axis,
};
use {
    crate::room_mechanisms::rules::treadmill::{Direction, carry},
    crate::{
        player_projectiles::{content, host::Host as Projectiles},
        room_mechanisms::host::Host as Mechanisms,
    },
};

fn normal_speed(h: &mut impl Host) {
    h.session_mut().player.contact.movement_speed_table = 0x9f49u16.wrapping_add(12);
}
fn special(h: &mut (impl Host + Projectiles + Mechanisms), bts: u8) {
    if bts & 0x80 != 0 {
        const TABLES: [u16; 8] = [
            0x9a06, 0x9a26, 0x9a46, 0x9a66, 0x9a86, 0x9aa6, 0x9ac6, 0x9ae6,
        ];
        let table = TABLES[usize::from(h.session().room.area)];
        let image = content::image(h);
        let at = content::offset(0x940000 | u32::from(table)) + usize::from(bts & 0x7f) * 2;
        assert!(at + 2 <= image.len(), "ROM read outside loaded allocation");
        let header = u16::from_le_bytes([image.read(at), image.read(at + 1)]);
        h.spawn_mechanism(header);
    } else {
        assert!(bts < 80, "special-air BTS index out of bounds: {bts}");
        match bts {
            8 | 9 => carry(
                h,
                if bts == 8 {
                    Direction::Right
                } else {
                    Direction::Left
                },
            ),
            10 | 11 => {
                h.session_mut().player.contact.external_x.fraction = 0;
                h.session_mut().player.contact.external_x.whole =
                    if bts == 10 { 2 } else { 0xfffe };
                normal_speed(h);
            }
            70 => {
                if h.session().player.contact.reaction_point == 1 {
                    h.spawn_mechanism(0xb6ff);
                }
            }
            _ => normal_speed(h),
        }
    }
}
fn react(h: &mut (impl Host + Projectiles + Mechanisms), x: u16, y: u16) {
    h.session_mut().room.collision_cursor = if x < 4096 && y < 4096 {
        (x >> 4) + (y >> 4) * u16::from(h.session().room.width as u8)
    } else {
        0xffff
    };
    loop {
        let kind = room_level(h, h.session().room.collision_cursor) >> 12;
        match kind {
            1 => {}
            2 => {
                let bts = room_bts(h, h.session().room.collision_cursor);
                assert!(bts < 16, "spike-air BTS index out of bounds: {bts}");
                if bts == 2 {
                    if h.session().player.contact.contact_attack == 0 {
                        super::geometry::spike_damage(h, 16);
                    }
                    normal_speed(h);
                }
            }
            3 => {
                let bts = room_bts(h, h.session().room.collision_cursor);
                special(h, bts);
            }
            5 | 13 => {
                if redirect(h, if kind == 5 { Axis::X } else { Axis::Y }) {
                    continue;
                }
            }
            _ => normal_speed(h),
        }
        break;
    }
}
pub fn sample(h: &mut (impl Host + Projectiles + Mechanisms)) {
    h.session_mut().player.contact.deceleration = 0;
    h.session_mut().player.contact.external_x.fraction = 0;
    h.session_mut().player.contact.external_x.whole = 0;
    h.session_mut().player.contact.external_y.fraction = 0;
    h.session_mut().player.contact.external_y.whole = 0;
    h.session_mut().player.contact.reaction_point = 0;
    let x = h.session().player.x;
    let bottom = h
        .session()
        .player
        .contact
        .radius
        .wrapping_add(h.session().player.y)
        .wrapping_sub(1);
    react(h, x, bottom);
    h.session_mut().player.contact.reaction_point = 1;
    let y = h.session().player.y;
    if (bottom ^ y) & 0xfff0 != 0 {
        let x = h.session().player.x;
        react(h, x, y);
    }
    h.session_mut().player.contact.reaction_point = 2;
    let y = h.session().player.y;
    let top = y.wrapping_sub(h.session().player.contact.radius);
    let changed_row_bits = (bottom ^ top) & 0xfff0;
    // Preserve the original XOR test, including live player changes after effects.
    if changed_row_bits != 0 && (y ^ changed_row_bits) & 0xfff0 != 0 {
        let x = h.session().player.x;
        react(h, x, top);
    }
}
