//! §8: what her moves meet. The world is data; each move asks what is at her leading edge, and
//! reports the cells it reaches (`TouchedCells`). Pixel arithmetic wraps mod 2^16, as positions
//! do; a cell is 16 px (`>> 4`, `& 15`) and a stepped block's quarter 8 px (`& 8`, `& !7`).

use crate::rules::Rules;
use clean_types::Heading::{self, Down, Right, Up};
use clean_types::{
    AirKind, Block, Cell, Fixed, Parity, Position, Radius, Slope, SlopeFaces, Surface, Touch, World,
};

/// What stopped a move.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Stop {
    /// `BlocksStopFlush`'s solid block, or `SlopesCarry`'s sloped surface.
    Flat,
    /// `BlocksStopFlush`'s stepped block, whose stop seats her (`SlopesCarry`).
    Stepped,
    /// `SolidEnemies`.
    Enemy,
}

/// One frame's moves: the room and rules they meet, the frame's parity (`BlocksStopFlush`), and
/// the cells they reach (`TouchedCells`).
pub struct Mover<'a> {
    /// The room.
    pub world: &'a dyn World,
    /// The rules.
    pub rules: &'a Rules,
    /// The frame's parity.
    pub parity: Parity,
    /// The cells her moves reach, in order.
    pub reached: Vec<Touch>,
}

/// Whether a move goes right or down, the way positions grow.
const fn grows(h: Heading) -> bool {
    matches!(h, Right | Down)
}

/// Pixel `p` moved `by` the way `g` says.
const fn shift(p: u16, by: u16, g: bool) -> u16 {
    if g {
        p.wrapping_add(by)
    } else {
        p.wrapping_sub(by)
    }
}

/// `BlocksStopFlush`: her leading edge, `p + r − 1` growing, `p − r` else.
const fn edge(p: u16, r: u16, g: bool) -> u16 {
    shift(p, r, g).wrapping_sub(if g { 1 } else { 0 })
}

/// `SolidEnemies`: whether two centres are closer than `reach`.
const fn closer(a: u16, b: u16, reach: u16) -> bool {
    a.wrapping_sub(b).cast_signed().unsigned_abs() < reach
}

/// `SlopesCarry`: the surface's row within its block at pixel column `x`: a floor's top, a
/// ceiling's bottom. An empty column's lies just past the far edge (16 under a floor, −1 over a
/// ceiling); a column far past lies a row beyond that, where no move or check ever meets it.
fn surface_row(s: Slope, x: u16) -> i16 {
    let floor = s.faces == SlopeFaces::Floor;
    s.surface.get(usize::from(x & 15)).map_or(0, |c| match *c {
        Surface::Row(n) => i16::from(n),
        Surface::JustPast if floor => 16,
        Surface::JustPast => -1,
        Surface::FarPast if floor => 17,
        Surface::FarPast => -2,
    })
}

/// `BlocksStopFlush`: whether a stepped block stops a move whose leading edge is at `lead`: a
/// solid quarter in the half the edge reaches, among those her body (`lo..=hi` across the move)
/// overlaps across it. `x` is the block's cell across the move.
fn stepped(q: [[bool; 2]; 2], v: bool, lead: u16, x: u16, (lo, hi): (u16, u16)) -> bool {
    let s = x.wrapping_mul(16);
    let [upper, lower] = q;
    [
        (false, lo.max(s).wrapping_sub(s) < 8),
        (true, hi.min(s.wrapping_add(15)).wrapping_sub(s) > 7),
    ]
    .into_iter()
    .any(|(across, overlaps)| {
        let (row, column) = if v {
            (lead & 8 != 0, across)
        } else {
            (across, lead & 8 != 0)
        };
        let [left, right] = if row { lower } else { upper };
        overlaps && if column { right } else { left }
    })
}

impl Mover<'_> {
    /// `OtherBlockKinds`: a cell's kind and the cell it is reported as. One that stands for
    /// another is that cell's kind, followed to the chain's end; one that names nothing, or a
    /// chain that comes back on itself, is air.
    fn resolve(&self, mut at: Cell) -> (Block, Cell) {
        let mut seen = Vec::new();
        loop {
            match self.world.block(at) {
                Block::StandsFor(Some(n)) if !seen.contains(&n) => {
                    seen.push(at);
                    at = n;
                }
                Block::StandsFor(_) => return (Block::Air(AirKind::Plain), at),
                b @ (Block::Air(_)
                | Block::Solid(_)
                | Block::Answered(_)
                | Block::Slope(_)
                | Block::Stepped(_)) => return (b, at),
            }
        }
    }

    /// `SlopesCarry`: the sloped surface holding pixel `(x, y)`: its faces, the pixel's row
    /// within the block, and the surface's row at that column.
    fn slope_at(&self, x: u16, y: u16) -> Option<(SlopeFaces, i16, i16)> {
        let at = Cell {
            column: x >> 4,
            row: y >> 4,
        };
        let (Block::Slope(s), _) = self.resolve(at) else {
            return None;
        };
        Some((s.faces, (y & 15).cast_signed(), surface_row(s, x)))
    }

    /// `TouchedCells`: the cells she is inside, from her position at the frame's start: her
    /// centre column at her bottom edge's row; at her centre's row, when it differs; at her top
    /// edge's row, when it differs from her bottom edge's and the three rows' exclusive-or is
    /// not 0.
    pub fn inside(&self, at: Position, r: Radius) -> Vec<Cell> {
        let (column, y) = (at.x.pixel() >> 4, at.y.pixel());
        let (bottom, centre, top) = (edge(y, r.y, true) >> 4, y >> 4, edge(y, r.y, false) >> 4);
        let mut rows = vec![bottom];
        if centre != bottom {
            rows.push(centre);
        }
        if top != bottom && bottom ^ centre ^ top != 0 {
            rows.push(top);
        }
        rows.into_iter()
            .map(|row| self.resolve(Cell { column, row }).1)
            .collect()
    }

    /// `SlopesSlowTheWalk`: a horizontal move of `d` heading `h`, shortened by the walk factor
    /// of each sloped floor in her leading edge's column at the unshortened target, in each row
    /// her body spans, top row first: cut to a multiple of 1/256 px towards minus infinity (read
    /// as 8.8 from the signed move), its pixel part kept mod 256, multiplied by the factor
    /// exactly, its direction put back.
    pub fn walk_shortened(&self, at: Position, r: Radius, heading: Heading, d: Fixed) -> Fixed {
        let grow = grows(heading);
        let target = if grow { at.x.plus(d) } else { at.x.minus(d) };
        let column = edge(target.pixel(), r.x, grow) >> 4;
        let centre = at.y.pixel();
        let rows = edge(centre, r.y, false) >> 4..=edge(centre, r.y, true) >> 4;
        rows.fold(d, |d, row| match self.resolve(Cell { column, row }).0 {
            Block::Slope(slope) if slope.faces == SlopeFaces::Floor => {
                let cut = if grow {
                    d.as_8_8()
                } else {
                    Fixed(0).minus(d).as_8_8().wrapping_neg()
                };
                Fixed(u32::from(cut).wrapping_mul(u32::from(slope.factor)))
            }
            Block::Air(_)
            | Block::Solid(_)
            | Block::Answered(_)
            | Block::Slope(_)
            | Block::Stepped(_)
            | Block::StandsFor(_) => d,
        })
    }

    /// A move of `dist` heading `heading`, stopped as §8 says: where it leaves her, and what
    /// stopped it. The cells it reaches go to the frame's report (`TouchedCells`).
    #[expect(clippy::too_many_lines)]
    pub fn go(
        &mut self,
        at: Position,
        r: Radius,
        heading: Heading,
        dist: Fixed,
    ) -> (Position, Option<Stop>) {
        // BlocksStopFlush, SolidEnemies, TouchedCells: a move of exactly 0 is never stopped and
        // reaches no cells.
        if dist.is_zero() {
            return (at, None);
        }
        let (vert, grow) = (matches!(heading, Up | Down), grows(heading));
        let limit = self.rules.move_limit_px;
        // BlocksStopFlush: a horizontal move is limited to 15 px either way, its subpixel kept.
        let dist = if !vert && dist.pixel() > limit {
            Fixed::new(limit, dist.sub())
        } else {
            dist
        };
        let (along, across, ra, rc) = if vert {
            (at.y, at.x.pixel(), r.y, r.x)
        } else {
            (at.x, at.y.pixel(), r.x, r.y)
        };
        let target = if grow {
            along.plus(dist)
        } else {
            along.minus(dist)
        };
        let start = along.pixel();
        // Her position with her centre at `pixel` along the move, and subpixel `sub`; a stop
        // never moves her backwards (BlocksStopFlush, SlopesCarry).
        let put = |pixel: u16, sub: u16| {
            let back = pixel.wrapping_sub(start).cast_signed();
            let behind = if grow { back < 0 } else { back > 0 };
            let value = Fixed::new(if behind { start } else { pixel }, sub);
            if vert {
                Position { y: value, ..at }
            } else {
                Position { x: value, ..at }
            }
        };
        // SolidEnemies: her reach is her box at the move's end pushed 1 px on: moving left or up
        // unless the end's subpixel is 0, right or down unless both her start's and the move's
        // are 0. An enemy she reaches is judged by the gap from where she starts, its near side
        // less hers, less 1: 0 is touching (no move, y subpixel 0), a larger gap stops her that
        // many whole pixels on, a negative one doesn't stop her. One decides the move alone.
        let push = if grow {
            along.sub() != 0 || dist.sub() != 0
        } else {
            target.sub() != 0
        };
        let reach = shift(target.pixel(), u16::from(push), grow);
        for enemy in self.world.solid_enemies() {
            let (ea, ec, era, erc) = if vert {
                (enemy.y, enemy.x, enemy.radius.y, enemy.radius.x)
            } else {
                (enemy.x, enemy.y, enemy.radius.x, enemy.radius.y)
            };
            let (ahead, behind) = if grow {
                (ea.wrapping_sub(era), start.wrapping_add(ra))
            } else {
                (start.wrapping_sub(ra), ea.wrapping_add(era))
            };
            let gap = ahead.wrapping_sub(behind).wrapping_sub(1).cast_signed();
            if gap >= 0
                && closer(reach, ea, ra.wrapping_add(era))
                && closer(across, ec, rc.wrapping_add(erc))
            {
                let to = if gap == 0 {
                    Position {
                        y: at.y.with_sub(0),
                        ..at
                    }
                } else {
                    put(shift(start, gap.cast_unsigned(), grow), along.sub())
                };
                return (to, Some(Stop::Enemy));
            }
        }
        // BlocksStopFlush: the line of her leading edge at the target, in every cell her body
        // spans across it, left to right (on an Odd frame's vertical move, right to left); the
        // first that stops her decides, flush against a face with subpixel FFFF or 0.
        let lead = edge(target.pixel(), ra, grow);
        let top = lead & !15;
        let face = |from: u16, len: u16| {
            let pixel = if grow {
                from.wrapping_sub(ra)
            } else {
                from.wrapping_add(len).wrapping_add(ra)
            };
            put(pixel, if grow { 0xFFFF } else { 0 })
        };
        let span = (
            across.wrapping_sub(rc),
            across.wrapping_add(rc).wrapping_sub(1),
        );
        let mut line: Vec<u16> = (span.0 >> 4..=span.1 >> 4).collect();
        if vert && self.parity == Parity::Odd {
            line.reverse();
        }
        for cell in line {
            let (column, row) = if vert {
                (cell, lead >> 4)
            } else {
                (lead >> 4, cell)
            };
            let (block, named) = self.resolve(Cell { column, row });
            self.reached.push(Touch {
                cell: named,
                heading,
            });
            let stop = match block {
                Block::Answered(_) if self.world.lets_through(named, heading) => None,
                Block::Solid(_) | Block::Answered(_) => Some((face(top, 16), Stop::Flat)),
                Block::Stepped(quarters) if stepped(quarters, vert, lead, cell, span) => {
                    Some((face(lead & !7, 8), Stop::Stepped))
                }
                // SlopesCarry: a sloped surface stops only a vertical move, at her centre column:
                // a floor once her bottom edge at the target reaches its last air row, on that
                // row by whole pixels; a ceiling once her top edge reaches its first air row, with
                // no move at all.
                Block::Slope(slope) if vert && cell == at.x.pixel() >> 4 => {
                    let off = (lead & 15).cast_signed();
                    let surface = surface_row(slope, at.x.pixel());
                    match slope.faces {
                        SlopeFaces::Floor if grow && off.wrapping_add(1) >= surface => {
                            let pixel = top.wrapping_add_signed(surface).wrapping_sub(ra);
                            Some((put(pixel, along.sub()), Stop::Flat))
                        }
                        SlopeFaces::Ceiling if !grow && off <= surface.wrapping_add(1) => {
                            Some((at, Stop::Flat))
                        }
                        SlopeFaces::Floor | SlopeFaces::Ceiling => None,
                    }
                }
                Block::Air(_) | Block::Stepped(_) | Block::Slope(_) | Block::StandsFor(_) => None,
            };
            if let Some((to, what)) = stop {
                return (to, Some(what));
            }
        }
        (put(target.pixel(), target.sub()), None)
    }

    /// `SlopesCarry`: the two checks after every horizontal move, each asking one cell at her
    /// centre column. A sloped floor at her bottom edge's row, with that edge in its solid part,
    /// lifts her by whole pixels onto its last air row; then a sloped ceiling at her top edge's
    /// row, with that edge in its solid part or on the first air row under it, pushes her down
    /// onto that row. Either seats her: returns where, if she is seated.
    pub fn seat(&self, at: Position, r: Radius) -> Option<Position> {
        let x = at.x.pixel();
        let lift = match self.slope_at(x, edge(at.y.pixel(), r.y, true)) {
            Some((SlopeFaces::Floor, off, row)) if off >= row => {
                off.wrapping_sub(row).wrapping_add(1)
            }
            _ => 0,
        };
        let lifted = at.y.pixel().wrapping_sub(lift.cast_unsigned());
        let push = match self.slope_at(x, edge(lifted, r.y, false)) {
            Some((SlopeFaces::Ceiling, off, row)) if off <= row.wrapping_add(1) => {
                Some(row.wrapping_add(1).wrapping_sub(off))
            }
            _ => None,
        };
        (lift > 0 || push.is_some()).then_some(Position {
            y: Fixed::new(
                lifted.wrapping_add(push.unwrap_or(0).cast_unsigned()),
                at.y.sub(),
            ),
            ..at
        })
    }

    /// `WallJump`: whether a test move of `reach` heading `h` would be stopped, and where the
    /// test leaves her. A block wall writes her x subpixel as a stop against it would, without
    /// moving her; a solid enemy writes her x nothing, one she touches clearing her y subpixel.
    /// The test's cells are its own, not the frame's report (`TouchedCells`).
    pub fn wall_test(
        &mut self,
        at: Position,
        r: Radius,
        h: Heading,
        reach: Fixed,
    ) -> (Position, bool) {
        let mark = self.reached.len();
        let (to, stop) = self.go(at, r, h, reach);
        self.reached.truncate(mark);
        let to = match stop {
            Some(Stop::Enemy) => Position { y: to.y, ..at },
            Some(Stop::Flat | Stop::Stepped) => Position {
                x: at.x.with_sub(if grows(h) { 0xFFFF } else { 0 }),
                ..at
            },
            None => at,
        };
        (to, stop.is_some())
    }
}
