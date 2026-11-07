//! §8: what her moves meet: tilemap collision, one axis at a time, against the room's grid of
//! 16 px cells and the solid enemies' boxes. The world is data; each move asks what is at her
//! leading edge, and reports the cells it reaches (`TouchedCells`). Pixel arithmetic wraps mod
//! 2^16, as positions do.

use crate::rules::Rules;
use clean_types::Heading::{self, Down, Right, Up};
use clean_types::{
    Block, Cell, Fixed, Hitbox, Pair, Parity, Position, Radius, Slope, SlopeFaces, Surface, Touch,
    Way, World,
};

/// §8: a cell is 16 px, `1 << CELL_BITS`.
const CELL_BITS: u16 = 4;
/// §8: a cell's size in px.
const CELL_PX: u16 = 16;
/// §8: a pixel's place within its cell, `p & IN_CELL`, and a cell's last pixel row.
const IN_CELL: u16 = 15;
/// `BlocksStopFlush`: a stepped block's quarter is 8 px, `1 << QUARTER_BITS`.
const QUARTER_BITS: u16 = 3;
/// `BlocksStopFlush`: a quarter's size in px, and the bit of a pixel that says which half of
/// its cell it is in.
const QUARTER_PX: u16 = 8;
/// `BlocksStopFlush`: a pixel's place within its quarter.
const IN_QUARTER: u16 = 7;
/// `BlocksStopFlush`: the subpixel a stop moving right or down writes.
const FLUSH: u16 = u16::MAX;
/// `SlopesCarry`: a sloped block's last pixel row, read signed.
const LAST_ROW: i16 = 15;
/// `SlopesCarry`: an empty column's surface lies just past the block's far edge.
const JUST_PAST: i16 = 16;
/// `SlopesCarry`: a column far past's surface lies a row beyond that.
const FAR_PAST: i16 = 17;

/// §8: a body's box, her centre and half-sizes.
#[derive(Clone, Copy)]
pub struct Bounds {
    /// Her centre.
    pub at: Position,
    /// Her half-width and half-height.
    pub r: Radius,
}

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

/// §8: whether a move goes right or down, the way positions grow.
const fn grows(h: Heading) -> bool {
    matches!(h, Right | Down)
}

/// §8: whether a move is vertical.
const fn vertical(h: Heading) -> bool {
    matches!(h, Up | Down)
}

/// §8: a pair as along and across a move heading `h`, or a cell's `(column, row)` from its
/// place along and across.
const fn along<T: Copy>(p: Pair<T>, h: Heading) -> (T, T) {
    if vertical(h) { (p.y, p.x) } else { (p.x, p.y) }
}

/// §8: the cell a pixel is in, along one axis.
const fn cell(p: u16) -> u16 {
    p >> CELL_BITS
}

/// §8: pixel `p` moved `by` along `h`.
const fn shift(p: u16, by: u16, h: Heading) -> u16 {
    if grows(h) {
        p.wrapping_add(by)
    } else {
        p.wrapping_sub(by)
    }
}

/// `SolidEnemies`, `BlocksStopFlush`: how far pixel `p` lies ahead of pixel `from` along `h`
/// (negative read signed: behind).
const fn ahead(p: u16, from: u16, h: Heading) -> i16 {
    if grows(h) {
        p.wrapping_sub(from).cast_signed()
    } else {
        from.wrapping_sub(p).cast_signed()
    }
}

/// `BlocksStopFlush`: her leading edge heading `h`: `p + r − 1` right or down, `p − r` left or up.
fn edge(p: u16, r: u16, h: Heading) -> u16 {
    shift(p, r, h).wrapping_sub(u16::from(grows(h)))
}

/// `TouchedCells`, `SlopesSlowTheWalk`: the first and last pixel her body spans across `p`.
fn extent(p: u16, r: u16) -> (u16, u16) {
    (edge(p, r, Up), edge(p, r, Down))
}

/// `SolidEnemies`: whether two centres are closer than `reach`.
const fn closer(a: u16, b: u16, reach: u16) -> bool {
    a.wrapping_sub(b).cast_signed().unsigned_abs() < reach
}

/// `SlopesCarry`, the one comparison every slope rule reads: how far an edge on pixel row `y`
/// of a sloped block, at pixel column `x`, is into its surface. 0 is the surface's solid edge
/// row, more is deeper into its solid part, −1 the air row next to it (a floor's last air row
/// above, a ceiling's first air row below). Rows are counted from the surface's open side, so
/// an empty column's surface lies just past the block's far edge, and a column far past's a row
/// beyond that: it never comes nearer than −2.
fn into(s: Slope, x: u16, y: u16) -> i16 {
    let floor = s.faces == SlopeFaces::Floor;
    let open = |n: i16| if floor { n } else { LAST_ROW.wrapping_sub(n) };
    let row = s
        .surface
        .get(usize::from(x & IN_CELL))
        .map_or(0, |c| match *c {
            Surface::Row(n) => open(i16::from(n)),
            Surface::JustPast => JUST_PAST,
            Surface::FarPast => FAR_PAST,
        });
    open((y & IN_CELL).cast_signed()).wrapping_sub(row)
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
    /// §8: a move of `dist` heading `heading` from where her box `b` is.
    const fn new(b: Bounds, heading: Heading, dist: Fixed) -> Self {
        let ((from, across), (ra, rc)) = (along(b.at, heading), along(b.r, heading));
        Self {
            at: b.at,
            heading,
            from,
            to: if grows(heading) {
                from.plus(dist)
            } else {
                from.minus(dist)
            },
            across: across.pixel(),
            ra,
            rc,
        }
    }

    /// `BlocksStopFlush`: her leading edge at the target.
    fn lead(&self) -> u16 {
        edge(self.to.pixel(), self.ra, self.heading)
    }

    /// `BlocksStopFlush`, `SlopesCarry`: her position with her centre at `pixel` along the move
    /// and subpixel `sub`; a stop never moves her backwards.
    const fn put(&self, pixel: u16, sub: u16) -> Position {
        let start = self.from.pixel();
        let behind = ahead(pixel, start, self.heading) < 0;
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

    /// `BlocksStopFlush`: flush against the face of `len` px starting at pixel `face` along the
    /// axis, her subpixel FFFF moving right or down, 0 left or up.
    const fn flush(&self, face: u16, len: u16, stop: Stop) -> Moved {
        let (pixel, sub) = if grows(self.heading) {
            (face.wrapping_sub(self.ra), FLUSH)
        } else {
            (face.wrapping_add(len).wrapping_add(self.ra), 0)
        };
        Moved {
            to: self.put(pixel, sub),
            stop: Some(stop),
        }
    }

    /// `SolidEnemies`: an enemy she reaches is judged by the gap between the boxes from where
    /// she starts, its near side less hers: 0 is touching (no move, her y subpixel 0), a larger
    /// gap stops her that many whole pixels on, her subpixel kept, and a negative one never
    /// stops her. The centres must be closer than the half-sizes summed on both axes, along the
    /// move at her reach.
    const fn enemy(&self, e: &Hitbox, reach: u16) -> Option<Moved> {
        let ((ea, ec), (era, erc)) = (along(e.centre, self.heading), along(e.radius, self.heading));
        let start = self.from.pixel();
        let gap =
            ahead(ea, start, self.heading).wrapping_sub(era.wrapping_add(self.ra).cast_signed());
        let reached = gap >= 0
            && closer(reach, ea, self.ra.wrapping_add(era))
            && closer(self.across, ec, self.rc.wrapping_add(erc));
        if !reached {
            return None;
        }
        let touching = Position {
            y: self.at.y.with_sub(0),
            ..self.at
        };
        let stopped = self.put(
            shift(start, gap.cast_unsigned(), self.heading),
            self.from.sub(),
        );
        Some(Moved {
            to: if gap == 0 { touching } else { stopped },
            stop: Some(Stop::Enemy),
        })
    }

    /// `BlocksStopFlush`, `SlopesCarry`: what a block on her leading edge's line, the `k`th
    /// cell across the move, does to it: a solid one stops her against its face, a stepped one
    /// against its quarter's, and a sloped surface at her centre column as `slope` says.
    fn stop(&self, block: Block, k: u16, way: Way) -> Option<Moved> {
        let lead = self.lead();
        match block {
            Block::Solid | Block::Answered => {
                Some(self.flush(lead & !IN_CELL, CELL_PX, Stop::Flat))
            }
            Block::Stepped(q) if self.stepped(q, k) => {
                Some(self.flush(lead & !IN_QUARTER, QUARTER_PX, Stop::Stepped))
            }
            Block::Slope(s) if vertical(self.heading) && k == cell(self.at.x.pixel()) => {
                self.slope(s, way)
            }
            Block::Stepped(_) | Block::Slope(_) | Block::Air | Block::StandsFor(_) => None,
        }
    }

    /// `BlocksStopFlush`: whether a stepped block, the `k`th cell across the move, stops it: a
    /// solid quarter in the half her leading edge reaches, among those her body overlaps across
    /// it (its near half is quarter `2k` across, its far half `2k + 1`).
    fn stepped(&self, [upper, lower]: [[bool; 2]; 2], k: u16) -> bool {
        let (near, (lo, hi)) = (k << 1, extent(self.across, self.rc));
        let half = self.lead() & QUARTER_PX != 0;
        [
            (false, lo >> QUARTER_BITS <= near),
            (true, hi >> QUARTER_BITS > near),
        ]
        .into_iter()
        .any(|(far, overlaps)| {
            let (right, low) = along(Pair { x: half, y: far }, self.heading);
            let [l, r] = if low { lower } else { upper };
            overlaps && if right { r } else { l }
        })
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
    /// another is that cell's kind, the chain followed to its end; one that names nothing is
    /// air.
    fn resolve(&self, at: Cell) -> (Block, Cell) {
        let block = self.world.block(at);
        if let Block::StandsFor(names) = block {
            return names.map_or((Block::Air, at), |next| self.resolve(next));
        }
        (block, at)
    }

    /// `TouchedCells`, `BlocksStopFlush`: the cells of her leading edge's line at a move's
    /// target, in every cell her body spans across it, each with its place across the move: left
    /// to right or top row first, right to left on an `Odd` frame's vertical move.
    fn line(&self, a: &Axis) -> Vec<(u16, Cell)> {
        let (lo, hi) = extent(a.across, a.rc);
        let mut line: Vec<u16> = (cell(lo)..=cell(hi)).collect();
        if vertical(a.heading) && self.parity == Parity::Odd {
            line.reverse();
        }
        line.into_iter()
            .map(|k| {
                let (column, row) = along(
                    Pair {
                        x: cell(a.lead()),
                        y: k,
                    },
                    a.heading,
                );
                (k, Cell { column, row })
            })
            .collect()
    }

    /// `SlopesCarry`: how far pixel row `y` at column `x` is into its cell's surface, where that
    /// cell is a sloped surface facing `faces`.
    fn slope_into(&self, x: u16, y: u16, faces: SlopeFaces) -> Option<i16> {
        let at = Cell {
            column: cell(x),
            row: cell(y),
        };
        let (Block::Slope(s), _) = self.resolve(at) else {
            return None;
        };
        (s.faces == faces).then(|| into(s, x, y))
    }

    /// `TouchedCells`: the cells she is inside, from her box at the frame's start: her centre
    /// column at her bottom edge's row; at her centre's row, when it differs; at her top edge's
    /// row, when it differs from her bottom edge's and the three rows' exclusive-or is not 0.
    pub fn inside(&self, b: Bounds) -> Vec<Cell> {
        let (column, y) = (cell(b.at.x.pixel()), b.at.y.pixel());
        let ((top, bottom), centre) = (extent(y, b.r.y), cell(y));
        let (top, bottom) = (cell(top), cell(bottom));
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
    pub fn walk_shortened(&self, b: Bounds, heading: Heading, d: Fixed) -> Fixed {
        let line = self.line(&Axis::new(b, heading, d));
        line.into_iter().fold(d, |d, (_, at)| {
            if let (Block::Slope(s), _) = self.resolve(at)
                && s.faces == SlopeFaces::Floor
            {
                let cut = if grows(heading) {
                    d.as_8_8()
                } else {
                    d.negated().as_8_8().wrapping_neg()
                };
                Fixed(u32::from(cut).wrapping_mul(u32::from(s.factor)))
            } else {
                d
            }
        })
    }

    /// §8: a move of `dist` heading `heading`, stopped as §8 says and tested as `probe` says:
    /// where it leaves her, and what stopped it. Its cells go to the frame's report
    /// (`TouchedCells`).
    pub fn go(&mut self, b: Bounds, heading: Heading, dist: Fixed, probe: Probe) -> Moved {
        // BlocksStopFlush, SolidEnemies, TouchedCells: a move of exactly 0 is never stopped and
        // reaches no cells.
        if dist.is_zero() {
            return Moved {
                to: b.at,
                stop: None,
            };
        }
        // BlocksStopFlush: a horizontal move is limited to 15 px either way, its subpixel kept.
        let limit = self.rules.move_limit_px;
        let dist = if !vertical(heading) && dist.pixel() > limit {
            Fixed::new(limit, dist.sub())
        } else {
            dist
        };
        let a = Axis::new(b, heading, dist);
        let way = if probe == Probe::Move {
            Way::Move(heading)
        } else {
            Way::Look
        };
        // SolidEnemies: a solid enemy in the move's path decides it alone; blocks stop her only
        // on a path free of them.
        let enemy = if probe == Probe::Blocks {
            None
        } else {
            self.enemy(&a, dist)
        };
        let stopped = if enemy.is_some() || probe == Probe::Enemies {
            enemy
        } else {
            self.blocks(&a, way)
        };
        let to = a.put(a.to.pixel(), a.to.sub());
        stopped.unwrap_or(Moved { to, stop: None })
    }

    /// `PoseFitAfterMoves`, `ShrinkingLowersHer`: a vertical look by `by`, signed, downward
    /// positive: a test move. A block look of 8 px or more is stopped by whatever a look of 8 px
    /// the same way would stop; only when nothing stops that is it the look of its own distance.
    pub fn look(&mut self, b: Bounds, by: Fixed, probe: Probe) -> Moved {
        let (heading, short) = (if by.negative() { Up } else { Down }, self.rules.look);
        if probe == Probe::Blocks && by.size() >= short {
            let first = self.go(b, heading, short, probe);
            if first.stop.is_some() {
                return first;
            }
        }
        self.go(b, heading, by.size(), probe)
    }

    /// `SolidEnemies`: her reach is her box at the move's end pushed 1 px on: moving left or up
    /// unless the end's subpixel is 0, right or down unless both her start's and the move's are
    /// 0. The first enemy she reaches decides the move.
    fn enemy(&self, a: &Axis, dist: Fixed) -> Option<Moved> {
        let push = if grows(a.heading) {
            a.from.sub() | dist.sub()
        } else {
            a.to.sub()
        } != 0;
        let reach = shift(a.to.pixel(), u16::from(push), a.heading);
        self.world
            .solid_enemies()
            .iter()
            .find_map(|e| a.enemy(e, reach))
    }

    /// `BlocksStopFlush`: the cells of her leading edge's line at the target; the first that
    /// stops her decides. `TouchedCells`: the cells it reaches, up to the one that stops it, with
    /// the way it reached them. `OtherBlockKinds`: an answered cell is air on a touch its reaction
    /// lets through.
    fn blocks(&mut self, a: &Axis, way: Way) -> Option<Moved> {
        self.line(a).into_iter().find_map(|(k, at)| {
            let (block, at) = self.resolve(at);
            self.reached.push(Touch { cell: at, way });
            let through = matches!(block, Block::Answered) && self.world.lets_through(at, way);
            a.stop(block, k, way).filter(|_| !through)
        })
    }

    /// `SlopesCarry`: the two checks after every horizontal move, each asking one cell at her
    /// centre column. A sloped floor at her bottom edge's row, with that edge in its solid part,
    /// lifts her by whole pixels onto its last air row; then a sloped ceiling at her top edge's
    /// row, with that edge in its solid part or on the first air row under it, pushes her down
    /// onto that row. Either seats her: returns where, if she is seated.
    pub fn seat(&self, b: Bounds) -> Option<Position> {
        let (x, y) = (b.at.x.pixel(), b.at.y.pixel());
        let lift = self
            .slope_into(x, edge(y, b.r.y, Down), SlopeFaces::Floor)
            .filter(|d| *d >= 0)
            .map_or(0, |d| d.wrapping_add(1));
        let lifted = y.wrapping_sub(lift.cast_unsigned());
        let push = self
            .slope_into(x, edge(lifted, b.r.y, Up), SlopeFaces::Ceiling)
            .filter(|d| *d >= -1)
            .map(|d| d.wrapping_add(1));
        (lift > 0 || push.is_some()).then(|| Position {
            y: Fixed::new(
                lifted.wrapping_add(push.unwrap_or(0).cast_unsigned()),
                b.at.y.sub(),
            ),
            ..b.at
        })
    }
}
