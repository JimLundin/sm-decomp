//! §8: what her moves meet: tilemap collision, one axis at a time, against the room's grid of
//! 16 px cells and the solid enemies' boxes. The world is data; each move asks what is at her
//! leading edge, and reports the cells it reaches (`TouchedCells`). Pixel arithmetic wraps mod
//! 2^16, as positions do. A frame's moves share their room, rules and cells in `Tick`, whose
//! §8 methods these are.

use crate::tick::Tick;
use Probe::{Blocks, Enemies, Move};
use Stop::{Enemy, Flat};
use clean_types::Block::{Air, Answered, Solid, StandsFor};
use clean_types::Heading::{self, Down, Right, Up};
use clean_types::Parity::Odd;
use clean_types::SlopeFaces::{Ceiling, Floor};
use clean_types::Surface::{FarPast, JustPast, Row};
use clean_types::{
    Block, Cell, Fixed, Hitbox, Pair, Parts, Position, Radius, Slope, SlopeFaces, Touch, Way, ZERO,
    fixed,
};
use core::num::Wrapping;

/// §8: a cell is 16 px, `1 << CELL_BITS`.
const CELL_BITS: u16 = 4;
/// §8: a pixel's place within its cell, `p & IN_CELL`, and a cell's last pixel row.
const IN_CELL: u16 = 15;
/// `BlocksStopFlush`: a stepped block's quarter is 8 px, `1 << QUARTER_BITS`.
const QUARTER_BITS: u16 = 3;
/// `BlocksStopFlush`: the bit of a pixel that says which half of its cell it is in.
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

/// `SlopesCarry`, the one comparison every slope rule reads: how many rows past the air row next
/// to its surface (a floor's last air row above, a ceiling's first air row below) an edge on
/// pixel row `y` of a sloped block, at pixel column `x`, is. 0 is that air row, 1 the surface's
/// solid edge row, more is deeper into its solid part. Rows are counted from the surface's open
/// side, so an empty column's surface lies just past the block's far edge, and a column far
/// past's a row beyond that: it never comes nearer than −1.
fn past(s: Slope, x: u16, y: u16) -> i16 {
    let floor = s.faces == Floor;
    let open = |n: i16| if floor { n } else { LAST_ROW.wrapping_sub(n) };
    let row = s
        .surface
        .get(usize::from(x & IN_CELL))
        .map_or(NO_COLUMN, |c| match *c {
            Row(n) => open(i16::from(n)),
            JustPast => JUST_PAST,
            FarPast => FAR_PAST,
        });
    open((y & IN_CELL).cast_signed())
        .wrapping_sub(row)
        .wrapping_add(1)
}

/// `SlopesCarry`: a pixel's column in its block, `x & 15`, is always one of a sloped block's
/// 16; were it not, the column would hold none of the surface and never meet her, as one far
/// past.
const NO_COLUMN: i16 = FAR_PAST;

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
    /// The move (`SlopesCarry`: a sloped floor's stop counts its pixel part, not the carry).
    dist: Fixed,
    /// Her centre's pixel across the axis.
    across: u16,
    /// Her half-size along the axis.
    ra: u16,
    /// Her half-size across it.
    rc: u16,
}

impl Axis {
    /// §8: a move of `dist` heading `heading` from where her box `b` is.
    fn new(b: Bounds, heading: Heading, dist: Fixed) -> Self {
        let ((from, across), (ra, rc)) = (along(b.at, heading), along(b.r, heading));
        Self {
            at: b.at,
            heading,
            from,
            to: if grows(heading) {
                from + dist
            } else {
                from - dist
            },
            dist,
            across: across.pixel(),
            ra,
            rc,
        }
    }

    /// `BlocksStopFlush`: her leading edge at the target.
    fn lead(&self) -> u16 {
        edge(self.to.pixel(), self.ra, self.heading)
    }

    /// `BlocksStopFlush`, `SlopesCarry`: the move ending with her centre at `pixel` along it and
    /// subpixel `sub`, stopped by `stop`; a stop never moves her backwards.
    fn put(&self, pixel: u16, sub: u16, stop: Option<Stop>) -> Moved {
        let start = self.from.pixel();
        let behind = ahead(pixel, start, self.heading) < 0;
        let value = fixed(if behind { start } else { pixel }, sub);
        let (x, y) = if vertical(self.heading) {
            (self.at.x, value)
        } else {
            (value, self.at.y)
        };
        Moved {
            to: Position { x, y },
            stop,
        }
    }

    /// `BlocksStopFlush`: flush against the face of the block or quarter her leading edge is in
    /// (`within`, a pixel's place within it), her subpixel FFFF moving right or down, 0 left or
    /// up: her edge on its near side's pixel, or just past its far side's.
    fn flush(&self, within: u16, stop: Stop) -> Moved {
        let lead = self.lead();
        let (pixel, sub) = if grows(self.heading) {
            ((lead & !within).wrapping_sub(self.ra), FLUSH)
        } else {
            ((lead | within).wrapping_add(1).wrapping_add(self.ra), 0)
        };
        self.put(pixel, sub, Some(stop))
    }

    /// `SolidEnemies`: an enemy she reaches is judged by the gap between the boxes from where
    /// she starts, its near side less hers: 0 is touching (no move, her y subpixel 0), a larger
    /// gap stops her that many whole pixels on, her subpixel kept, and a negative one never
    /// stops her. The centres must be closer than the half-sizes summed on both axes, along the
    /// move at her reach.
    fn enemy(&self, e: &Hitbox, reach: u16) -> Option<Moved> {
        let ((ea, ec), (era, erc)) = (along(e.centre, self.heading), along(e.radius, self.heading));
        let start = self.from.pixel();
        let gap =
            ahead(ea, start, self.heading).wrapping_sub(era.wrapping_add(self.ra).cast_signed());
        let reached = gap >= 0
            && closer(reach, ea, self.ra.wrapping_add(era))
            && closer(self.across, ec, self.rc.wrapping_add(erc));
        let pixel = shift(start, gap.cast_unsigned(), self.heading);
        let mut moved = self.put(pixel, self.from.sub(), Some(Enemy));
        if gap == 0 {
            moved.to.y = moved.to.y.with_sub(0);
        }
        reached.then_some(moved)
    }

    /// `BlocksStopFlush`, `SlopesCarry`: what a block on her leading edge's line, the `k`th
    /// cell across the move, does to it: a solid one stops her against its face, a stepped one
    /// against its quarter's, and a sloped surface at her centre column as `slope` says.
    fn stop(&self, block: Block, k: u16, way: Way) -> Option<Moved> {
        match block {
            Solid | Answered => Some(self.flush(IN_CELL, Flat)),
            Block::Stepped(q) if self.stepped(q, k) => Some(self.flush(IN_QUARTER, Stop::Stepped)),
            Block::Slope(s) if vertical(self.heading) && k == cell(self.at.x.pixel()) => {
                self.slope(s, way)
            }
            Block::Stepped(_) | Block::Slope(_) | Air | StandsFor(_) => None,
        }
    }

    /// `BlocksStopFlush`: whether a stepped block, the `k`th cell across the move, stops it: a
    /// solid quarter in the half her leading edge reaches, among those her body overlaps across
    /// it (the block's quarters across are `2k`, its near half, and `2k + 1`, its far half).
    fn stepped(&self, [upper, lower]: [[bool; 2]; 2], k: u16) -> bool {
        let (lo, hi) = extent(self.across, self.rc);
        let half = self.lead() & QUARTER_PX != 0;
        let near = k << 1;
        ((lo >> QUARTER_BITS).max(near)..=(hi >> QUARTER_BITS).min(near | 1)).any(|q| {
            let (right, low) = along(
                Pair {
                    x: half,
                    y: q & 1 != 0,
                },
                self.heading,
            );
            let [l, r] = if low { lower } else { upper };
            if right { r } else { l }
        })
    }

    /// `SlopesCarry`: a sloped surface stops a vertical move only in the block at her centre
    /// column. A floor stops a move or a look down once her bottom edge at the target (her
    /// position plus the whole move, subpixels added) is on its last air row or past it: she
    /// moves the move's pixel part less how far past that row the edge is, by whole pixels, her
    /// subpixel kept, so a carried pixel leaves her 1 px higher. A ceiling stops a rise once her
    /// top edge reaches its first air row, with no move at all. A look up passes every sloped
    /// ceiling.
    fn slope(&self, s: Slope, way: Way) -> Option<Moved> {
        let past = past(s, self.at.x.pixel(), self.lead());
        let floor = s.faces == Floor;
        let meets = if floor {
            self.heading == Down
        } else {
            way == Way::Move(Up)
        };
        let start = self.from.pixel();
        let pixel = if floor {
            start
                .wrapping_add(self.dist.pixel())
                .wrapping_sub(past.cast_unsigned())
        } else {
            start
        };
        (meets && past >= 0).then(|| self.put(pixel, self.from.sub(), Some(Flat)))
    }
}

impl Tick<'_> {
    /// `OtherBlockKinds`: a cell's kind and the cell it is reported as. One that stands for
    /// another is that cell's kind, the chain followed to its end; one that names nothing is
    /// air.
    fn resolve(&self, at: Cell) -> (Block, Cell) {
        let block = self.world.block(at);
        if let StandsFor(names) = block {
            return names.map_or((Air, at), |next| self.resolve(next));
        }
        (block, at)
    }

    /// `TouchedCells`, `BlocksStopFlush`: the cells of her leading edge's line at a move's
    /// target, in every cell her body spans across it, each with its place across the move: left
    /// to right or top row first, right to left on an `Odd` frame's vertical move.
    fn line(&self, a: &Axis) -> Vec<(u16, Cell)> {
        let ((lo, hi), lead) = (extent(a.across, a.rc), cell(a.lead()));
        let mut line: Vec<u16> = (cell(lo)..=cell(hi)).collect();
        if vertical(a.heading) && self.frame.parity == Odd {
            line.reverse();
        }
        line.into_iter()
            .map(|k| {
                let (column, row) = along(Pair { x: lead, y: k }, a.heading);
                (k, Cell { column, row })
            })
            .collect()
    }

    /// `SlopesCarry`: how many rows past its surface's air row pixel row `y` at column `x` is,
    /// where its cell is a sloped surface facing `faces`.
    fn slope_past(&self, x: u16, y: u16, faces: SlopeFaces) -> Option<i16> {
        let at = Cell {
            column: cell(x),
            row: cell(y),
        };
        let (Block::Slope(s), _) = self.resolve(at) else {
            return None;
        };
        (s.faces == faces).then(|| past(s, x, y))
    }

    /// `TouchedCells`: the cells she is inside, from her box at the frame's start: her centre
    /// column at her bottom edge's row; at her centre's row, when it differs; at her top edge's
    /// row, when it differs from her bottom edge's and the three rows' exclusive-or is not 0.
    pub fn inside(&self, b: Bounds) -> Vec<Cell> {
        let (column, y) = (cell(b.at.x.pixel()), b.at.y.pixel());
        let (top, bottom) = extent(y, b.r.y);
        let [top, centre, bottom] = [top, y, bottom].map(cell);
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
                && s.faces == Floor
            {
                let cut = if grows(heading) {
                    d.to_8_8()
                } else {
                    (-d).to_8_8().wrapping_neg()
                };
                Wrapping(u32::from(cut)) * Wrapping(u32::from(s.factor))
            } else {
                d
            }
        })
    }

    /// §8: a move of `dist` heading `heading`, stopped as §8 says and tested as `probe` says:
    /// where it leaves her, and what stopped it. Its cells go to the frame's report
    /// (`TouchedCells`).
    pub fn go(&mut self, b: Bounds, heading: Heading, dist: Fixed, probe: Probe) -> Moved {
        // BlocksStopFlush: a horizontal move is limited to 15 px either way, its subpixel kept.
        let limit = fixed(self.rules.move_limit_px, dist.sub());
        let a = Axis::new(
            b,
            heading,
            if vertical(heading) {
                dist
            } else {
                dist.min(limit)
            },
        );
        let way = if probe == Move {
            Way::Move(heading)
        } else {
            Way::Look
        };
        // BlocksStopFlush, SolidEnemies, TouchedCells: a move of exactly 0 is never stopped and
        // reaches no cells. SolidEnemies: a solid enemy in the move's path decides it alone;
        // blocks stop her only on a path free of them.
        let enemy = if dist == ZERO || probe == Blocks {
            None
        } else {
            self.enemy(&a)
        };
        let stopped = if enemy.is_some() || dist == ZERO || probe == Enemies {
            enemy
        } else {
            self.blocks(&a, way)
        };
        stopped.unwrap_or_else(|| a.put(a.to.pixel(), a.to.sub(), None))
    }

    /// `PoseFitAfterMoves`, `ShrinkingLowersHer`: a vertical look by `by`, signed, downward
    /// positive: a test move. A block look of 8 px or more is stopped by whatever a look of 8 px
    /// the same way would stop; only when nothing stops that is it the look of its own distance.
    pub fn look(&mut self, b: Bounds, by: Fixed, probe: Probe) -> Moved {
        let (heading, size, short) = (
            if by.negative() { Up } else { Down },
            by.size(),
            self.rules.look,
        );
        let first = (probe == Blocks && size >= short).then(|| self.go(b, heading, short, probe));
        first
            .filter(|m| m.stop.is_some())
            .unwrap_or_else(|| self.go(b, heading, size, probe))
    }

    /// `SolidEnemies`: her reach is her box at the move's end pushed 1 px on: moving left or up
    /// unless the end's subpixel is 0, right or down unless both her start's and the move's are
    /// 0. The first enemy she reaches decides the move.
    fn enemy(&self, a: &Axis) -> Option<Moved> {
        let push = if grows(a.heading) {
            a.from.sub() | a.dist.sub()
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
            let through = matches!(block, Answered) && self.world.lets_through(at, way);
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
            .slope_past(x, edge(y, b.r.y, Down), Floor)
            .filter(|d| *d > 0);
        let lifted = y.wrapping_sub(lift.unwrap_or(0).cast_unsigned());
        let push = self
            .slope_past(x, edge(lifted, b.r.y, Up), Ceiling)
            .filter(|d| *d >= 0);
        let pushed = lifted.wrapping_add(push.unwrap_or(0).cast_unsigned());
        lift.or(push).map(|_| Position {
            y: fixed(pushed, b.at.y.sub()),
            ..b.at
        })
    }
}
