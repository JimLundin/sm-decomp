//! Projectile visibility and trail lifecycle; drawing is the synchronous edge.
#![forbid(unsafe_code)]
use super::{Host, content, host::Drawing, state::Trail};
use crate::game::projectile_trail_emission::{self as emission, Half, Owner, Point, Write};
use crate::game::projectile_trail_playback::{
    self as playback, Definitions, Instruction, Presentation,
};
use crate::rom::RomRecord;

fn side(h: &impl Host, pair: usize, half: Half) -> &Trail {
    let pair = &h.session().projectiles.trails[pair];
    match half {
        Half::Left => &pair.left,
        Half::Right => &pair.right,
    }
}
fn side_mut(h: &mut impl Host, pair: usize, half: Half) -> &mut Trail {
    let pair = &mut h.session_mut().projectiles.trails[pair];
    match half {
        Half::Left => &mut pair.left,
        Half::Right => &mut pair.right,
    }
}
impl<H: Host> Owner for &mut H {
    type Cursor = u16;
    type Sprite = u16;
    fn remaining(&self, pair: usize, half: Half) -> u16 {
        side(&**self, pair, half).remaining
    }
    fn cursor(&self, pair: usize, half: Half) -> u16 {
        side(&**self, pair, half).cursor
    }
    fn y(&self, pair: usize, half: Half) -> u16 {
        side(&**self, pair, half).y
    }
    fn write(&mut self, pair: usize, half: Half, value: Write<u16, u16>) {
        let side = side_mut(&mut **self, pair, half);
        match value {
            Write::Timer(v) => side.remaining = v,
            Write::Cursor(v) => side.cursor = v,
            Write::Tile(v) => side.tile = v,
            Write::X(v) => side.x = v,
            Write::Y(v) => side.y = v,
        }
    }
}
struct Offsets(RomRecord);
impl emission::Offsets for Offsets {
    fn left_y(&self) -> i8 {
        self.0.byte(1) as i8
    }
    fn left_x(&self) -> i8 {
        self.0.byte(0) as i8
    }
    fn right_y(&self) -> i8 {
        self.0.byte(3) as i8
    }
    fn right_x(&self) -> i8 {
        self.0.byte(2) as i8
    }
}
struct Trails {
    frozen: bool,
}
impl<'a, H: Host> emission::Context<&'a mut H> for Trails {
    type Style = u16;
    type Marker = u16;
    type Offsets = Offsets;
    fn style(&self, h: &&'a mut H, projectile: u16) -> Option<u16> {
        let kind = shot!(h, projectile, kind);
        if kind & 0xf00 == 0 {
            Some(kind & 63)
        } else {
            let class = kind.to_ne_bytes()[1] & 15;
            (class < 3).then_some(u16::from(class) + 31)
        }
    }
    fn script(&mut self, h: &'a mut H, style: u16, half: Half) -> (&'a mut H, u16) {
        let cursor = content::table(
            h,
            match half {
                Half::Left => 0x90b5bb,
                Half::Right => 0x90b609,
            },
            style as isize,
        );
        (h, cursor)
    }
    fn selected_projectile(&self, h: &&'a mut H) -> u16 {
        h.session().projectiles.selected
    }
    fn marker(&mut self, h: &'a mut H, slot: u16) -> (&'a mut H, u16) {
        let marker = super::animation::trail_frame(h, slot);
        (h, marker)
    }
    fn origin(&mut self, h: &'a mut H, slot: u16) -> (&'a mut H, Point) {
        let point = if h.inputs().ceres & 0x8000 == 0 {
            Point {
                x: shot!(h, slot, x),
                y: shot!(h, slot, y),
            }
        } else {
            let (x, y) = project(h, slot);
            Point {
                x: h.inputs().camera_x.wrapping_add(x),
                y: h.inputs().camera_y.wrapping_add(y),
            }
        };
        (h, point)
    }
    fn offsets(&mut self, h: &'a mut H, slot: u16, marker: u16) -> (&'a mut H, Offsets) {
        let kind = shot!(h, slot, kind);
        let table = if kind & 0x20 != 0 {
            0x9ba4e3
        } else if kind & 0x10 != 0 {
            0x9ba4cb
        } else {
            0x9ba4b3
        };
        let first = content::table(h, table, (kind & 15) as isize)
            .wrapping_add(2 * (shot!(h, slot, direction) & 15));
        let first = h.record(0x9b, first);
        let second = first.word(0).wrapping_add(marker.wrapping_mul(4));
        let second = h.record(0x9b, second);
        (h, Offsets(second))
    }
}
#[derive(Clone, Copy)]
struct ProgramCounter {
    value: i32,
    half: Half,
}
impl ProgramCounter {
    fn next(self) -> Self {
        Self {
            value: match self.half {
                Half::Left => self.value + 2,
                Half::Right => i32::from((self.value as u16).wrapping_add(2)),
            },
            ..self
        }
    }
}
impl<'a, H: Host> Definitions<&'a mut H> for Trails {
    type Cursor = ProgramCounter;
    fn start(&mut self, cursor: u16, half: Half) -> ProgramCounter {
        ProgramCounter {
            value: i32::from(cursor),
            half,
        }
    }
    fn instruction(
        &mut self,
        h: &'a mut H,
        cursor: ProgramCounter,
    ) -> (&'a mut H, Instruction<ProgramCounter>) {
        let value = h.record(0x90, cursor.value as u16).word(0);
        let instruction = if value & 0x8000 == 0 {
            Instruction::Frame { duration: value }
        } else {
            let next = cursor.next();
            match value {
                0xb525 => Instruction::Move {
                    target: Half::Left,
                    delta_y: 1,
                    next,
                },
                0xb587 => Instruction::Move {
                    target: Half::Right,
                    delta_y: 1,
                    next,
                },
                0xb5b3 => Instruction::Move {
                    target: Half::Left,
                    delta_y: -1,
                    next,
                },
                _ => Instruction::Unknown { next },
            }
        };
        (h, instruction)
    }
    fn frame(&mut self, h: &'a mut H, cursor: ProgramCounter) -> (&'a mut H, u16, u16) {
        let sprite = h.record(0x90, cursor.value as u16).word(2);
        (h, sprite, (cursor.value + 4) as u16)
    }
    fn unknown(&mut self, h: &'a mut H) -> &'a mut H {
        h.unknown();
        h
    }
}
impl<'a, H: Host> Presentation<&'a mut H> for Trails {
    fn draw(&mut self, h: &'a mut H, pair: usize, half: Half) -> &'a mut H {
        h.draw_trail(pair, half == Half::Right, self.frozen);
        h
    }
}
pub fn emit(h: &mut impl Host, slot: u16) {
    emission::emit(h, slot, &mut Trails { frozen: false });
}
pub fn update_trails(h: &mut impl Host) {
    h.probe("trails.update");
    let frozen = h.inputs().frozen != 0;
    playback::present(h, frozen, &mut Trails { frozen });
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
                kind & 0xf00 >= 0x300 && (kind & 0xf00 != 0x300 || shot!(h, index, variable) != 0)
            } else if kind & 0xf10 != 0 {
                kind & 0xf00 < 0x300
            } else if kind & 0xc != 0 {
                (index & 2 != 0) == (h.inputs().frame & 2 != 0)
            } else {
                (index & 2 == 0) == (h.inputs().frame & 1 != 0)
            };
            if visible {
                'draw: {
                    let clip_y = if h.inputs().ceres & 0x8000 != 0
                        && (!explosions || kind & 0xf00 != 0x300 && kind & 0xf00 != 0x500)
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
                        h.probe(if shot!(h, index, kind) & 0xf00 >= 0x300 {
                            "player_projectiles.draw.explosion"
                        } else {
                            "player_projectiles.draw.projectile"
                        });
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
