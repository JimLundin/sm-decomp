//! Projectile visibility and trail lifecycle; drawing is the synchronous edge.
#![forbid(unsafe_code)]
use super::{Host, content, host::Drawing, state::Trail};
use crate::shot;

fn side(h: &impl Host, pair: usize, right: bool) -> &Trail {
    let pair = &h.session().projectiles.trails[pair];
    if right { &pair.right } else { &pair.left }
}
fn side_mut(h: &mut impl Host, pair: usize, right: bool) -> &mut Trail {
    let pair = &mut h.session_mut().projectiles.trails[pair];
    if right {
        &mut pair.right
    } else {
        &mut pair.left
    }
}
pub fn emit(h: &mut impl Host, slot: u16) {
    let kind = shot!(h, slot, kind);
    let style = if kind & super::kinds::TYPE_MASK == 0 {
        kind & 63
    } else {
        let class = kind.to_ne_bytes()[1] & 15;
        if class >= 3 {
            return;
        }
        u16::from(class) + 31
    };
    let Some(pair) = (0..18)
        .rev()
        .find(|&pair| h.session().projectiles.trails[pair].left.remaining == 0)
    else {
        return;
    };
    // Only the left timer controls reservation; each write precedes the next lookup.
    h.session_mut().projectiles.trails[pair].left.remaining = 1;
    h.session_mut().projectiles.trails[pair].right.remaining = 1;
    let script = content::table(h, 0x90b5bb, style as isize);
    h.session_mut().projectiles.trails[pair].left.cursor = script;
    let script = content::table(h, 0x90b609, style as isize);
    h.session_mut().projectiles.trails[pair].right.cursor = script;
    let selected = h.session().projectiles.selected;
    let marker = super::animation::trail_frame(h, selected);
    let (x, y) = if h.inputs().ceres & 0x8000 == 0 {
        (shot!(h, selected, x), shot!(h, selected, y))
    } else {
        let (x, y) = project(h, selected);
        (
            h.inputs().camera_x.wrapping_add(x),
            h.inputs().camera_y.wrapping_add(y),
        )
    };
    let kind = shot!(h, selected, kind);
    let table = if kind & 0x20 != 0 {
        0x9ba4e3
    } else if kind & 0x10 != 0 {
        0x9ba4cb
    } else {
        0x9ba4b3
    };
    let first = content::table(h, table, (kind & 15) as isize)
        .wrapping_add(2 * (shot!(h, selected, direction) & 15));
    let first = h.record(0x9b, first);
    let second = first.word(0).wrapping_add(marker.wrapping_mul(4));
    let offsets = h.record(0x9b, second);
    let coordinate =
        |origin: u16, offset: u8| origin.wrapping_add(offset as i8 as u16).wrapping_sub(4);
    h.session_mut().projectiles.trails[pair].left.y = coordinate(y, offsets.byte(1));
    h.session_mut().projectiles.trails[pair].left.x = coordinate(x, offsets.byte(0));
    h.session_mut().projectiles.trails[pair].right.y = coordinate(y, offsets.byte(3));
    h.session_mut().projectiles.trails[pair].right.x = coordinate(x, offsets.byte(2));
    h.probe("player_projectiles.trail.emitted");
}
fn advance_trail(h: &mut impl Host, pair: usize, right: bool) -> bool {
    let remaining = side(h, pair, right).remaining;
    if remaining == 0 {
        return false;
    }
    side_mut(h, pair, right).remaining = remaining.wrapping_sub(1);
    if remaining != 1 {
        return true;
    }
    let mut cursor = i32::from(side(h, pair, right).cursor);
    loop {
        let value = h.record(0x90, cursor as u16).word(0);
        if value & 0x8000 == 0 {
            side_mut(h, pair, right).remaining = value;
            if value == 0 {
                return false;
            }
            let tile = h.record(0x90, cursor as u16).word(2);
            let following = (cursor + 4) as u16;
            side_mut(h, pair, right).tile = tile;
            side_mut(h, pair, right).cursor = following;
            return true;
        }
        // The left script advances linearly; the right script wraps at 16 bits.
        let next = if right {
            i32::from((cursor as u16).wrapping_add(2))
        } else {
            cursor + 2
        };
        match value {
            0xb525 => {
                let y = side(h, pair, false).y.wrapping_add(1);
                side_mut(h, pair, false).y = y;
            }
            0xb587 => {
                let y = side(h, pair, true).y.wrapping_add(1);
                side_mut(h, pair, true).y = y;
            }
            0xb5b3 => {
                let y = side(h, pair, false).y.wrapping_sub(1);
                side_mut(h, pair, false).y = y;
            }
            _ => {
                h.unknown();
            }
        }
        cursor = next;
    }
}
pub fn update_trails(h: &mut impl Host) {
    h.probe("trails.update");
    let frozen = h.inputs().frozen != 0;
    for pair in (0..18).rev() {
        for right in [false, true] {
            let enabled = if frozen {
                side(h, pair, right).remaining != 0
            } else {
                advance_trail(h, pair, right)
            };
            if enabled {
                h.draw_trail(pair, right, frozen);
            }
        }
    }
}

// Preserve the original A coefficient on both diagonal products.
pub fn project_xy((dx, dy): (u16, u16), (a, b, c): (u16, u16, u16)) -> (u16, u16) {
    let scale = |value: u16, coefficient: u16| {
        ((i32::from(value as i16) * i32::from(coefficient as i16)) as u32 >> 8) as u16
    };
    (
        scale(dx, a).wrapping_add(scale(dy, b)),
        scale(dx, c).wrapping_add(scale(dy, a)),
    )
}
pub fn project(h: &impl Host, slot: u16) -> (u16, u16) {
    let i = h.inputs();
    let (x, y) = project_xy(
        (
            shot!(h, slot, x).wrapping_sub(i.m7_x),
            i.m7_y.wrapping_sub(shot!(h, slot, y)),
        ),
        (i.m7_a, i.m7_b, i.m7_c),
    );
    (
        x.wrapping_add(i.m7_x).wrapping_sub(i.camera_x),
        i.m7_y.wrapping_sub(y).wrapping_sub(i.camera_y),
    )
}
pub fn project_player(h: &mut impl Host) {
    let i = h.inputs();
    let (x, y) = project_xy(
        (
            h.session().player.x.wrapping_sub(i.m7_x),
            i.m7_y.wrapping_sub(h.session().player.y),
        ),
        (i.m7_a, i.m7_b, i.m7_c),
    );
    h.session_mut().player.x = x.wrapping_add(h.inputs().m7_x);
    h.session_mut().player.y = h.inputs().m7_y.wrapping_sub(y);
}
pub fn draw_projectiles(h: &mut impl Host) {
    present(h, false);
    h.draw(Drawing::CrashEchoes);
    update_trails(h);
}
pub fn draw_explosions(h: &mut impl Host) {
    present(h, true);
}
fn present(h: &mut impl Host, explosions: bool) {
    let mut index = if explosions { 18 } else { 8 };
    let (mut x, mut y) = (0, 0);
    h.session_mut().projectiles.selected = index;
    loop {
        if shot!(h, index, cursor) != 0 {
            let kind = shot!(h, index, kind);
            let visible = if explosions {
                kind & super::kinds::TYPE_MASK >= super::kinds::POWER_BOMB
                    && (kind & super::kinds::TYPE_MASK != super::kinds::POWER_BOMB
                        || shot!(h, index, variable) != 0)
            } else if kind & 0xf10 != 0 {
                kind & super::kinds::TYPE_MASK < super::kinds::POWER_BOMB
            } else if kind & 0xc != 0 {
                (index & 2 != 0) == (h.inputs().frame & 2 != 0)
            } else {
                (index & 2 == 0) == (h.inputs().frame & 1 != 0)
            };
            if visible {
                'draw: {
                    let clip_y = if h.inputs().ceres & 0x8000 != 0
                        && (!explosions
                            || kind & super::kinds::TYPE_MASK != super::kinds::POWER_BOMB
                                && kind & super::kinds::TYPE_MASK != super::kinds::BOMB)
                    {
                        let point = project(h, index);
                        // Preserve carried-Y/projected-clipping behavior exactly.
                        if explosions {
                            y
                        } else {
                            x = point.0;
                            point.1
                        }
                    } else {
                        x = shot!(h, index, x).wrapping_sub(h.inputs().camera_x);
                        if explosions
                            && (x.wrapping_sub(304) & 0x8000 == 0
                                || x.wrapping_add(48) & 0x8000 != 0)
                        {
                            break 'draw;
                        }
                        y = shot!(h, index, y).wrapping_sub(h.inputs().camera_y);
                        y
                    };
                    if clip_y & 0xff00 == 0 && (explosions || shot!(h, index, sprite) & 0x8000 != 0)
                    {
                        h.probe(
                            if shot!(h, index, kind) & super::kinds::TYPE_MASK
                                >= super::kinds::POWER_BOMB
                            {
                                "player_projectiles.draw.explosion"
                            } else {
                                "player_projectiles.draw.projectile"
                            },
                        );
                        h.draw(Drawing::Projectile { slot: index, x, y });
                    }
                    index = h.session().projectiles.selected;
                }
            }
        }
        index = index.wrapping_sub(2);
        h.session_mut().projectiles.selected = index;
        if index & 0x8000 != 0 {
            return;
        }
    }
}
