//! §8: what her moves meet: tilemap collision, one axis at a time, against the room's grid of
//! 16 px cells and the solid enemies' boxes. The world is data; each move asks what is at her
//! leading edge, and reports the cells it reaches (`TouchedCells`). Pixel arithmetic wraps mod
//! 2^16, as positions do; a cell is 16 px (`>> 4`, `& 15`) and a stepped block's quarter 8 px
//! (`& 8`, `& !7`).

use crate::rules::Rules;
use clean_types::Heading::{self, Down, Right, Up};
use clean_types::{
    Block, Cell, Fixed, Parity, Position, Radius, Slope, SlopeFaces, Surface, Touch, Way, World,
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

/// What a move or a look is tested against, and the way its cells are reported (`TouchedCells`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Probe {
    /// A move: solid enemies first, then blocks; its cells with its direction.
    Move,
    /// The near-wall test (`WallJump`): as a move, its cells with the way `Look`.
    Look,
    /// Pose fit's enemy look: solid enemies alone, reaching no cells.
    Enemies,
    /// Pose fit's and the shrink's block look: blocks alone, its cells with the way `Look`.
    Blocks,
}

/// Where a move leaves her, and what stopped it.
#[derive(Clone, Copy)]
pub struct Moved {
    /// Her position.
    pub to: Position,
    /// What stopped the move.
    pub stop: Option<Stop>,
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
    /// The cells her moves and looks reach, in order.
    pub reached: Vec<Touch>,
}

/// Whether a move goes right or down, the way positions grow.
const fn grows(h: Heading) -> bool {
    matches!(h, Right | Down)
}

/// Whether a move is vertical.
const fn vertical(h: Heading) -> bool {
    matches!(h, Up | Down)
}

/// The pair `(x, y)` as along and across a move heading `h`, or a cell's `(column, row)` from
/// its place along and across.
const fn split<T: Copy>(x: T, y: T, h: Heading) -> (T, T) {
    if vertical(h) { (y, x) } else { (x, y) }
}

/// Pixel `p` moved `by` along `h`.
const fn shift(p: u16, by: u16, h: Heading) -> u16 {
    if grows(h) {
        p.wrapping_add(by)
    } else {
        p.wrapping_sub(by)
    }
}

/// How far pixel `p` lies ahead of pixel `from` along `h` (negative read signed: behind).
const fn ahead(p: u16, from: u16, h: Heading) -> u16 {
    if grows(h) {
        p.wrapping_sub(from)
    } else {
        from.wrapping_sub(p)
    }
}

/// `BlocksStopFlush`: her leading edge heading `h`: `p + r − 1` right or down, `p − r` left or up.
const fn edge(p: u16, r: u16, h: Heading) -> u16 {
    if grows(h) {
        p.wrapping_add(r).wrapping_sub(1)
    } else {
        p.wrapping_sub(r)
    }
}

/// `SolidEnemies`: whether two centres are closer than `reach`.
const fn closer(a: u16, b: u16, reach: u16) -> bool {
    a.wrapping_sub(b).cast_signed().unsigned_abs() < reach
}

/// `SlopesCarry`, the one comparison every slope rule reads: how far an edge on pixel row `y`
/// of a sloped block, at pixel column `x`, is into its surface. 0 is the surface's solid edge
/// row, more is deeper into its solid part, −1 the air row next to it (a floor's last air row
/// above, a ceiling's first air row below). An empty column's surface lies just past the block's
/// far edge, and a column far past's a row beyond that, so it never comes nearer than −2.
fn into(s: Slope, x: u16, y: u16) -> i16 {
    let floor = s.faces == SlopeFaces::Floor;
    let row = s.surface.get(usize::from(x & 15)).map_or(0, |c| match *c {
        Surface::Row(n) if floor => i16::from(n),
        Surface::Row(n) => 15_i16.wrapping_sub(i16::from(n)),
        Surface::JustPast => 16,
        Surface::FarPast => 17,
    });
    let off = (y & 15).cast_signed();
    if floor { off } else { 15_i16.wrapping_sub(off) }.wrapping_sub(row)
}

/// `BlocksStopFlush`: whether a stepped block stops a move whose leading edge is at `lead`: a
/// solid quarter in the half the edge reaches, among those her body (`lo..=hi` across the move)
/// overlaps across it. `k` is the block's cell across the move, whose near half is quarter `2k`
/// across and far half `2k + 1`.
fn stepped(q: [[bool; 2]; 2], h: Heading, lead: u16, k: u16, (lo, hi): (u16, u16)) -> bool {
    let (near, [upper, lower]) = (k << 1, q);
    [(false, lo >> 3 <= near), (true, hi >> 3 > near)]
        .into_iter()
        .any(|(across, overlaps)| {
            let (column, row) = split(lead & 8 != 0, across, h);
            let [left, right] = if row { lower } else { upper };
            overlaps && if column { right } else { left }
        })
}

/// One move along its axis, as §8 reads it: from her centre `from` to the target `to`, her
/// centre `across` the axis, and her half-sizes along it and across it.
struct Axis {
    /// Where she starts.
    at: Position,
    /// The way she moves.
    heading: Heading,
    /// Her centre along the axis.
    from: Fixed,
    /// The target along the axis.
    to: Fixed,
    /// Her centre's pixel across the axis.
    across: u16,
    /// Her half-size along the axis.
    ra: u16,
    /// Her half-size across it.
    rc: u16,
}

impl Axis {
    /// A move of `dist` heading `heading` from `at`, her body `r`.
    const fn new(at: Position, r: Radius, heading: Heading, dist: Fixed) -> Self {
        let (from, across) = split(at.x, at.y, heading);
        let (ra, rc) = split(r.x, r.y, heading);
        let to = if grows(heading) {
            from.plus(dist)
        } else {
            from.minus(dist)
        };
        Self {
            at,
            heading,
            from,
            to,
            across: across.pixel(),
            ra,
            rc,
        }
    }

    /// `BlocksStopFlush`: her leading edge at the target.
    const fn lead(&self) -> u16 {
        edge(self.to.pixel(), self.ra, self.heading)
    }

    /// Her position with her centre at `pixel` along the move and subpixel `sub`; a stop never
    /// moves her backwards (`BlocksStopFlush`, `SlopesCarry`).
    const fn put(&self, pixel: u16, sub: u16) -> Position {
        let start = self.from.pixel();
        let behind = ahead(pixel, start, self.heading).cast_signed() < 0;
        let value = Fixed::new(if behind { start } else { pixel }, sub);
        if vertical(self.heading) {
            Position {
                y: value,
                ..self.at
            }
        } else {
            Position {
                x: value,
                ..self.at
            }
        }
    }

    /// `BlocksStopFlush`: flush against the face of `len` px starting at pixel `from` along the
    /// axis, her subpixel FFFF moving right or down, 0 left or up.
    const fn flush(&self, from: u16, len: u16, stop: Stop) -> Moved {
        let (pixel, sub) = if grows(self.heading) {
            (from.wrapping_sub(self.ra), 0xFFFF)
        } else {
            (from.wrapping_add(len).wrapping_add(self.ra), 0)
        };
        Moved {
            to: self.put(pixel, sub),
            stop: Some(stop),
        }
    }

    /// `SlopesCarry`: a sloped surface stops a vertical move only in the block at her centre
    /// column. A floor stops a move or a look down once her bottom edge at the target is on its
    /// last air row or past it, with that edge on that row, by whole pixels, her subpixel kept;
    /// a ceiling stops a rise once her top edge reaches its first air row, with no move at all.
    /// A look up passes every sloped ceiling.
    fn slope(&self, s: Slope, way: Way) -> Option<Moved> {
        let into = into(s, self.at.x.pixel(), self.lead());
        let floor = s.faces == SlopeFaces::Floor;
        let meets = if floor {
            self.heading == Down
        } else {
            way == Way::Move(Up)
        };
        (meets && into >= -1).then(|| Moved {
            to: if floor {
                let lift = into.wrapping_add(1).cast_unsigned();
                self.put(self.to.pixel().wrapping_sub(lift), self.from.sub())
            } else {
                self.at
            },
            stop: Some(Stop::Flat),
        })
    }
}

impl Mover<'_> {
    /// `OtherBlockKinds`: a cell's kind and the cell it is reported as. One that stands for
    /// another is that cell's kind, followed to the chain's end; one that names nothing, or a
    /// chain that comes back on itself, is air.
    fn resolve(&self, mut at: Cell) -> (Block, Cell) {
        let mut seen = Vec::new();
        loop {
            let block = self.world.block(at);
            let Block::StandsFor(names) = block else {
                return (block, at);
            };
            let Some(next) = names.filter(|n| !seen.contains(n)) else {
                return (Block::Air, at);
            };
            seen.push(at);
            at = next;
        }
    }

    /// `SlopesCarry`: how far pixel row `y` at column `x` is into its cell's surface, where that
    /// cell is a sloped surface facing `faces`.
    fn slope_into(&self, x: u16, y: u16, faces: SlopeFaces) -> Option<i16> {
        let cell = Cell {
            column: x >> 4,
            row: y >> 4,
        };
        let (Block::Slope(s), _) = self.resolve(cell) else {
            return None;
        };
        (s.faces == faces).then(|| into(s, x, y))
    }

    /// `TouchedCells`: the cells she is inside, from her position at the frame's start: her
    /// centre column at her bottom edge's row; at her centre's row, when it differs; at her top
    /// edge's row, when it differs from her bottom edge's and the three rows' exclusive-or is
    /// not 0.
    pub fn inside(&self, at: Position, r: Radius) -> Vec<Cell> {
        let (column, y) = (at.x.pixel() >> 4, at.y.pixel());
        let (bottom, centre, top) = (edge(y, r.y, Down) >> 4, y >> 4, edge(y, r.y, Up) >> 4);
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

    /// `SlopesSlowTheWalk`: a horizontal move of `d` heading `heading`, shortened by the walk
    /// factor of each sloped floor in her leading edge's column at the unshortened target, in
    /// each row her body spans, top row first: cut to a multiple of 1/256 px towards minus
    /// infinity (read as 8.8 from the signed move), its pixel part kept mod 256, multiplied by
    /// the factor exactly, its direction put back.
    pub fn walk_shortened(&self, at: Position, r: Radius, heading: Heading, d: Fixed) -> Fixed {
        let column = Axis::new(at, r, heading, d).lead() >> 4;
        let y = at.y.pixel();
        (edge(y, r.y, Up) >> 4..=edge(y, r.y, Down) >> 4).fold(d, |d, row| {
            let (block, _) = self.resolve(Cell { column, row });
            if let Block::Slope(s) = block
                && s.faces == SlopeFaces::Floor
            {
                let cut = if grows(heading) {
                    d.as_8_8()
                } else {
                    Fixed(0).minus(d).as_8_8().wrapping_neg()
                };
                Fixed(u32::from(cut).wrapping_mul(u32::from(s.factor)))
            } else {
                d
            }
        })
    }

    /// A move of `dist` heading `heading`, stopped as §8 says and tested as `probe` says: where
    /// it leaves her, and what stopped it. Its cells go to the frame's report.
    pub fn go(
        &mut self,
        at: Position,
        r: Radius,
        heading: Heading,
        dist: Fixed,
        probe: Probe,
    ) -> Moved {
        // BlocksStopFlush, SolidEnemies, TouchedCells: a move of exactly 0 is never stopped and
        // reaches no cells.
        if dist.is_zero() {
            return Moved { to: at, stop: None };
        }
        // BlocksStopFlush: a horizontal move is limited to 15 px either way, its subpixel kept.
        let limit = self.rules.move_limit_px;
        let dist = if !vertical(heading) && dist.pixel() > limit {
            Fixed::new(limit, dist.sub())
        } else {
            dist
        };
        let a = Axis::new(at, r, heading, dist);
        // SolidEnemies: a solid enemy in the move's path decides it alone; blocks stop her only
        // on a path free of them.
        let enemy = if probe == Probe::Blocks {
            None
        } else {
            self.enemy(&a, dist)
        };
        enemy
            .or_else(|| {
                if probe == Probe::Enemies {
                    None
                } else {
                    self.blocks(&a, probe)
                }
            })
            .unwrap_or_else(|| Moved {
                to: a.put(a.to.pixel(), a.to.sub()),
                stop: None,
            })
    }

    /// A look of `px` whole pixels (`PoseFitAfterMoves`, `ShrinkingLowersHer`, `WallJump`): a
    /// test move. A block look of 8 px or more is stopped by whatever a look of 8 px the same
    /// way would stop; only when nothing stops that is it the look of its own distance.
    pub fn look(
        &mut self,
        at: Position,
        r: Radius,
        heading: Heading,
        px: u16,
        probe: Probe,
    ) -> Moved {
        let short = self.rules.look_px;
        if probe == Probe::Blocks && px >= short {
            let first = self.go(at, r, heading, Fixed::px(short), probe);
            if first.stop.is_some() {
                return first;
            }
        }
        self.go(at, r, heading, Fixed::px(px), probe)
    }

    /// `SolidEnemies`: her reach is her box at the move's end pushed 1 px on: moving left or up
    /// unless the end's subpixel is 0, right or down unless both her start's and the move's are
    /// 0. An enemy she reaches is judged by the gap between the boxes from where she starts, its
    /// near side less hers: 0 is touching (no move, her y subpixel 0), a larger gap stops her
    /// that many whole pixels on, her subpixel kept, and a negative one never stops her.
    fn enemy(&self, a: &Axis, dist: Fixed) -> Option<Moved> {
        let push = if grows(a.heading) {
            a.from.sub() | dist.sub()
        } else {
            a.to.sub()
        } != 0;
        let reach = shift(a.to.pixel(), u16::from(push), a.heading);
        let start = a.from.pixel();
        self.world.solid_enemies().iter().find_map(|e| {
            let (ea, ec) = split(e.x, e.y, a.heading);
            let (era, erc) = split(e.radius.x, e.radius.y, a.heading);
            let gap = ahead(ea, start, a.heading)
                .wrapping_sub(era)
                .wrapping_sub(a.ra)
                .cast_signed();
            let reached = gap >= 0
                && closer(reach, ea, a.ra.wrapping_add(era))
                && closer(a.across, ec, a.rc.wrapping_add(erc));
            reached.then(|| Moved {
                to: if gap == 0 {
                    Position {
                        y: a.at.y.with_sub(0),
                        ..a.at
                    }
                } else {
                    a.put(shift(start, gap.cast_unsigned(), a.heading), a.from.sub())
                },
                stop: Some(Stop::Enemy),
            })
        })
    }

    /// `BlocksStopFlush`: the line of her leading edge at the target, in every cell her body
    /// spans across it, left to right (right to left on an Odd frame's vertical move); the first
    /// that stops her decides. `TouchedCells`: the cells it reaches, up to the one that stops it,
    /// with the way it reached them.
    fn blocks(&mut self, a: &Axis, probe: Probe) -> Option<Moved> {
        let lead = a.lead();
        let span = (
            a.across.wrapping_sub(a.rc),
            a.across.wrapping_add(a.rc).wrapping_sub(1),
        );
        let mut line: Vec<u16> = (span.0 >> 4..=span.1 >> 4).collect();
        if vertical(a.heading) && self.parity == Parity::Odd {
            line.reverse();
        }
        let way = if probe == Probe::Move {
            Way::Move(a.heading)
        } else {
            Way::Look
        };
        line.into_iter().find_map(|k| {
            let (column, row) = split(lead >> 4, k, a.heading);
            let (block, cell) = self.resolve(Cell { column, row });
            self.reached.push(Touch { cell, way });
            match block {
                // OtherBlockKinds: an answered cell is air on a touch its reaction lets through.
                Block::Answered if self.world.lets_through(cell, way) => None,
                Block::Solid | Block::Answered => Some(a.flush(lead & !15, 16, Stop::Flat)),
                Block::Stepped(q) if stepped(q, a.heading, lead, k, span) => {
                    Some(a.flush(lead & !7, 8, Stop::Stepped))
                }
                Block::Slope(s) if vertical(a.heading) && k == a.at.x.pixel() >> 4 => {
                    a.slope(s, way)
                }
                Block::Stepped(_) | Block::Slope(_) | Block::Air | Block::StandsFor(_) => None,
            }
        })
    }

    /// `SlopesCarry`: the two checks after every horizontal move, each asking one cell at her
    /// centre column. A sloped floor at her bottom edge's row, with that edge in its solid part,
    /// lifts her by whole pixels onto its last air row; then a sloped ceiling at her top edge's
    /// row, with that edge in its solid part or on the first air row under it, pushes her down
    /// onto that row. Either seats her: returns where, if she is seated.
    pub fn seat(&self, at: Position, r: Radius) -> Option<Position> {
        let (x, y) = (at.x.pixel(), at.y.pixel());
        let lift = self
            .slope_into(x, edge(y, r.y, Down), SlopeFaces::Floor)
            .filter(|d| *d >= 0)
            .map_or(0, |d| d.wrapping_add(1));
        let lifted = y.wrapping_sub(lift.cast_unsigned());
        let push = self
            .slope_into(x, edge(lifted, r.y, Up), SlopeFaces::Ceiling)
            .filter(|d| *d >= -1)
            .map(|d| d.wrapping_add(1));
        (lift > 0 || push.is_some()).then(|| Position {
            y: Fixed::new(
                lifted.wrapping_add(push.unwrap_or(0).cast_unsigned()),
                at.y.sub(),
            ),
            ..at
        })
    }
}
