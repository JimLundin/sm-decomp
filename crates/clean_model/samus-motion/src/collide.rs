//! Tilemap collision (`BlocksStopFlush`, `SolidEnemies`, `SlopesCarry`, `TouchedCells`,
//! `OtherBlockKinds`): one axis at a time, against the room's grid of 16 px cells and the solid
//! enemies' boxes. The world is data; each move asks what is at her leading edge, and reports
//! the cells it reaches. Pixels are `Px`, whose arithmetic wraps mod 2^16 as positions do. A
//! frame's moves share their room, rules and cells in `Tick`, whose collision methods these are.

use crate::tick::Tick;
use Probe::{Blocks, Enemies, Look, Move, Walk};
use Stop::{Enemy, Flat, Seats};
use clean_types::Block::{self, Air, Answered, Solid, StandsFor, Stepped};
use clean_types::Heading::{self, Down, Right, Up};
use clean_types::Parity::Odd;
use clean_types::SlopeFaces::Floor;
use clean_types::{
    Cell, Fixed, Hitbox, Pair, Parts, Position, Px, Radius, Slope, Touch, Way, ZERO, fixed, px,
};
use core::num::Wrapping;

/// `BlocksStopFlush`: one pixel.
const ONE: Px = Wrapping(1);
/// `BlocksStopFlush`: no pixels.
const NONE: Px = Wrapping(0);
/// `BlocksStopFlush`: a cell is 16 px: a pixel's cell is `p >> CELL_BITS`.
const CELL_BITS: u32 = 4;
/// `BlocksStopFlush`: a pixel's place within its cell, `p & IN_CELL`, and a cell's last row.
const IN_CELL: Px = Wrapping(15);
/// `BlocksStopFlush`: a stepped block's quarter is 8 px: a pixel's quarter is `p >> 3`.
const QUARTER_BITS: u32 = 3;
/// `BlocksStopFlush`: the bit of a pixel that says which half of its cell it is in.
const QUARTER: Px = Wrapping(8);
/// `BlocksStopFlush`: a pixel's place within its quarter.
const IN_QUARTER: Px = Wrapping(7);
/// `BlocksStopFlush`: the subpixel a stop moving right or down writes.
const FLUSH: u16 = u16::MAX;
/// `SlopesCarry`: a column far past's depth, which never meets her. A pixel's column in its
/// block, `x & 15`, is always one of a sloped block's 16; were it not, the column would hold none
/// of the surface, as one far past.
const FAR_PAST: Px = Wrapping(17);

/// `BlocksStopFlush`: a body's box, her centre and half-sizes.
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
    /// `BlocksStopFlush`'s solid block, a stepped block met any way but downward, or
    /// `SlopesCarry`'s sloped surface.
    Flat,
    /// `BlocksStopFlush`'s stepped block, met by a downward move or look, which seats her
    /// (`SlopesCarry`).
    Seats,
    /// `SolidEnemies`.
    Enemy,
}

/// What a move or a look is tested against, and the way its cells are reported (`TouchedCells`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Probe {
    /// `SlopesSlowTheWalk`: a horizontal move made with her vertical speed exactly 0, which a
    /// sloped floor at her leading edge shortens; else as a move.
    Walk,
    /// A move: solid enemies first, then blocks; its cells with its direction.
    Move,
    /// The near-wall test (`WallJump`): as a move, its cells with the way `Look`.
    Look,
    /// Pose fit's enemy look: solid enemies alone, reaching no cells.
    Enemies,
    /// Pose fit's and the shrink's block look: blocks alone, its cells with the way `Look`.
    Blocks,
}

/// Where a move leaves her, what stopped it, and how far it got in whole px (a look's room). A
/// look doesn't move her: it leaves her where she was, with what it wrote to her as a stop would.
pub struct Moved {
    /// Her position.
    pub to: Position,
    /// What stopped the move.
    pub stop: Option<Stop>,
    /// How far it went along its way, in whole px.
    pub gone: Px,
}

/// `BlocksStopFlush`: whether a move goes right or down, the way positions grow.
const fn grows(h: Heading) -> bool {
    matches!(h, Right | Down)
}

/// `BlocksStopFlush`: the cell a pixel is in, along one axis.
const fn cell(p: Px) -> u16 {
    p.0 >> CELL_BITS
}

/// `BlocksStopFlush`: whether a pixel difference, read signed, is below zero.
const fn behind(p: Px) -> bool {
    p.0.cast_signed() < 0
}

/// `TouchedCells`, `BlocksStopFlush`: the first and last pixel a body of half-size `r` spans
/// about `p`.
fn extent(p: Px, r: Px) -> (Px, Px) {
    (p - r, p + r - ONE)
}

/// `BlocksStopFlush`: her leading edge heading `h`: `p + r − 1` right or down, `p − r` left or up.
fn edge(p: Px, r: Px, h: Heading) -> Px {
    let (lo, hi) = extent(p, r);
    if grows(h) { hi } else { lo }
}

/// `SlopesCarry`, the one comparison every slope rule reads: how many rows past the air row next
/// to its surface (a floor's last air row above, a ceiling's first air row below) an edge on
/// a sloped block, at pixel `at`, is, where it is on that row or past it, and the surface faces
/// a move heading `h` (a floor one down, a ceiling one up). 0 is that air row, 1 the surface's
/// solid edge row, more is deeper into its solid part. The row and the surface's depth are both
/// counted from the block's open side, so an empty column's air row is the block's far row, and
/// a column far past is never reached.
fn past(s: Slope, h: Heading, at: Pair<Px>) -> Option<Px> {
    let floor = s.faces == Floor;
    let row = (if floor { at.y } else { !at.y }) & IN_CELL;
    let depth = s.depth.get(usize::from((at.x & IN_CELL).0));
    let past = row + ONE - depth.copied().unwrap_or(FAR_PAST);
    (!behind(past) && floor == (h == Down)).then_some(past)
}

/// `BlocksStopFlush`: one move along its axis, from where she starts: where she is, the way she
/// moves and the way its cells are reported (`TouchedCells`), `dir`, a pixel on that way; her
/// centre along the axis, the move, her centre's pixel across the axis, and her half-sizes
/// along it and across it. Its target is derived from these.
#[derive(Clone, Copy)]
struct Axis {
    /// Her centre.
    at: Position,
    /// The way she moves.
    heading: Heading,
    /// The way its cells are reported: a move's direction, or a look.
    way: Way,
    /// One pixel on along the move: +1 or −1.
    dir: Px,
    /// Her centre along the axis.
    from: Fixed,
    /// The move (`SlopesCarry`: a sloped floor's stop counts its pixel part, not the carry).
    dist: Fixed,
    /// Her centre's pixel across the axis.
    across: Px,
    /// Her half-size along the axis.
    ra: Px,
    /// Her half-size across it.
    rc: Px,
}

impl Axis {
    /// `BlocksStopFlush`: her centre along the axis at the move's target, the move added.
    fn to(&self) -> Fixed {
        if grows(self.heading) {
            self.from + self.dist
        } else {
            self.from - self.dist
        }
    }

    /// `BlocksStopFlush`: how far pixel `p` lies ahead of where she starts, along the move.
    fn ahead(&self, p: Px) -> Px {
        (p - self.from.pixel()) * self.dir
    }

    /// `BlocksStopFlush`: her leading edge at the target.
    fn lead(&self) -> Px {
        edge(self.to().pixel(), self.ra, self.heading)
    }

    /// `BlocksStopFlush`, `SlopesCarry`: the move ending `by` whole px on from where she starts,
    /// with subpixel `sub`, stopped by `stop`; a stop never moves her backwards. A look leaves
    /// her pixel where it was (`PoseFitAfterMoves`, `WallJump`).
    fn put(&self, by: Px, sub: u16, stop: Option<Stop>) -> Moved {
        let gone = if behind(by) { NONE } else { by };
        let moved = if self.way == Way::Look { NONE } else { gone };
        let value = fixed(self.from.pixel() + moved * self.dir, sub);
        Moved {
            to: self.at.with(self.heading, value),
            stop,
            gone,
        }
    }

    /// `BlocksStopFlush`: flush against the face of the block or quarter her leading edge is in
    /// (`within`, a pixel's place within it), her subpixel FFFF moving right or down, 0 left or
    /// up: her edge on its near side's pixel, or just past its far side's.
    fn flush(&self, within: Px, stop: Stop) -> Moved {
        let lead = self.lead();
        let (pixel, sub) = if grows(self.heading) {
            ((lead & !within) - self.ra, FLUSH)
        } else {
            ((lead | within) + ONE + self.ra, 0)
        };
        self.put(self.ahead(pixel), sub, Some(stop))
    }

    /// `SolidEnemies`: an enemy she reaches is judged by the gap between the boxes from where
    /// she starts, its near side less hers: 0 is touching (no move, her y subpixel 0), a larger
    /// gap stops her that many whole pixels on, her subpixel kept, and a negative one never
    /// stops her. The centres must be closer than the half-sizes summed on both axes, along the
    /// move at her reach: her box at the move's end pushed 1 px on, moving left or up unless the
    /// end's subpixel is 0, right or down unless both her start's and the move's are 0.
    fn enemy(&self, e: &Hitbox) -> Option<Moved> {
        let ((ea, ec), (era, erc)) = (e.centre.along(self.heading), e.radius.along(self.heading));
        let gap = self.ahead(ea) - era - self.ra;
        let push = if grows(self.heading) {
            self.from.sub() | self.dist.sub()
        } else {
            self.to().sub()
        } != 0;
        let reach = self.to().pixel() + if push { self.dir } else { NONE };
        let near = |a: Px, b: Px, sum: Px| (a - b).0.cast_signed().unsigned_abs() < sum.0;
        let reached =
            !behind(gap) && near(reach, ea, self.ra + era) && near(self.across, ec, self.rc + erc);
        let mut moved = self.put(gap, self.from.sub(), Some(Enemy));
        if gap == NONE {
            moved.to.y = moved.to.y.with_sub(0);
        }
        reached.then_some(moved)
    }

    /// `BlocksStopFlush`, `SlopesCarry`: what a block on her leading edge's line, the `k`th
    /// cell across the move, does to it: a solid one stops her against its face, a stepped one
    /// against its quarter's (seating her, met downward), and a sloped surface at her centre
    /// column as `slope` says.
    fn stop(&self, block: Block, k: u16) -> Option<Moved> {
        let seats = if self.heading == Down { Seats } else { Flat };
        match block {
            Solid | Answered => Some(self.flush(IN_CELL, Flat)),
            Stepped(q) if self.stepped(q, k) => Some(self.flush(IN_QUARTER, seats)),
            Block::Slope(s) if self.heading.vertical() && k == cell(self.across) => self.slope(s),
            Stepped(_) | Block::Slope(_) | Air | StandsFor(_) => None,
        }
    }

    /// `BlocksStopFlush`: whether a stepped block, the `k`th cell across the move, stops it: a
    /// solid quarter in the half her leading edge reaches, among those her body overlaps across
    /// it (the block's quarters across are `2k`, its near half, and `2k + 1`, its far half).
    fn stepped(&self, [upper, lower]: [[bool; 2]; 2], k: u16) -> bool {
        let (lo, hi) = extent(self.across, self.rc);
        let half = self.lead() & QUARTER != NONE;
        let near = k << 1;
        ((lo.0 >> QUARTER_BITS).max(near)..=(hi.0 >> QUARTER_BITS).min(near | 1)).any(|q| {
            let far = q & 1 != 0;
            let at = Pair { x: far, y: far }.with(self.heading, half);
            let [l, r] = if at.y { lower } else { upper };
            if at.x { r } else { l }
        })
    }

    /// `SlopesCarry`: a sloped surface stops a vertical move only in the block at her centre
    /// column. A floor stops a move or a look down once her bottom edge at the target (her
    /// position plus the whole move, subpixels added) is on its last air row or past it: she
    /// moves the move's pixel part less how far past that row the edge is, by whole pixels, her
    /// subpixel kept, so a carried pixel leaves her 1 px higher. A ceiling stops a rise once her
    /// top edge reaches its first air row, with no move at all. A look up passes every sloped
    /// ceiling.
    fn slope(&self, s: Slope) -> Option<Moved> {
        let floor = s.faces == Floor;
        let at = Pair {
            x: self.across,
            y: self.lead(),
        };
        let past = past(s, self.heading, at).filter(|_| floor || self.way != Way::Look)?;
        let by = if floor {
            self.dist.pixel() - past
        } else {
            NONE
        };
        Some(self.put(by, self.from.sub(), Some(Flat)))
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
        if a.heading.vertical() && self.frame.parity == Odd {
            line.reverse();
        }
        line.into_iter()
            .map(|k| (k, Pair { x: k, y: k }.with(a.heading, lead)))
            .collect()
    }

    /// `SlopesCarry`, the one helper for a seat's lift and push: at her centre column and her
    /// edge heading `h` (her bottom edge for a floor, her top edge for a ceiling), how many rows
    /// past the air row next to its surface that edge is, where the cell is a sloped surface
    /// facing that way and the edge is on that air row or past it.
    fn sloped(&self, b: Bounds, h: Heading) -> Option<Px> {
        let at = Pair {
            x: b.at.x.pixel(),
            y: edge(b.at.y.pixel(), b.r.y, h),
        };
        let (Block::Slope(surface), _) = self.resolve(Pair {
            x: cell(at.x),
            y: cell(at.y),
        }) else {
            return None;
        };
        past(surface, h, at)
    }

    /// `TouchedCells`: the cells she is inside, from her box at the frame's start: her centre
    /// column at her bottom edge's row; at her centre's row, when it differs; at her top edge's
    /// row, when it differs from her bottom edge's and the three rows' exclusive-or is not 0.
    pub fn inside(&self, b: Bounds) -> Vec<Cell> {
        let (column, y) = (cell(b.at.x.pixel()), b.at.y.pixel());
        let (top, bottom) = extent(y, b.r.y);
        let [top, centre, bottom] = [top, y, bottom].map(cell);
        let rows = [
            Some(bottom),
            (centre != bottom).then_some(centre),
            (top != bottom && bottom ^ centre ^ top != 0).then_some(top),
        ];
        rows.into_iter()
            .flatten()
            .map(|row| self.resolve(Pair { x: column, y: row }).1)
            .collect()
    }

    /// `SlopesSlowTheWalk`: a walk shortened by the walk factor of each sloped floor in her
    /// leading edge's column at the unshortened target, in each row her body spans, top row
    /// first: cut to a multiple of 1/256 px towards minus infinity (read as 8.8 from the signed
    /// move), its pixel part kept mod 256, multiplied by the factor exactly, its direction put
    /// back.
    fn shortened(&self, a: &Axis) -> Fixed {
        self.line(a).into_iter().fold(a.dist, |d, (_, at)| {
            if let (Block::Slope(s), _) = self.resolve(at)
                && s.faces == Floor
            {
                let cut = if grows(a.heading) {
                    d.to_8_8()
                } else {
                    -(-d).to_8_8()
                };
                Wrapping(u32::from(cut.0)) * s.factor
            } else {
                d
            }
        })
    }

    /// `SolidEnemies`: the first solid enemy that stops a move, if any.
    fn enemies(&self, a: &Axis) -> Option<Moved> {
        self.world.solid_enemies().iter().find_map(|e| a.enemy(e))
    }

    /// `BlocksStopFlush`, `SolidEnemies`, `TouchedCells`: a move of `dist` heading `heading`,
    /// tested as `probe` says: where it leaves her, and what stopped it; its cells go to the
    /// frame's report. A walk is shortened first (`SlopesSlowTheWalk`), and a horizontal move is
    /// limited to 15 px either way, its subpixel kept. A move of exactly 0 is never stopped and
    /// reaches no cells. A solid enemy in the move's path decides it alone, the first she
    /// reaches; blocks stop her only on a path free of them.
    pub fn go(&mut self, b: Bounds, heading: Heading, dist: Fixed, probe: Probe) -> Moved {
        let ((from, across), (ra, rc)) = (b.at.along(heading), b.r.along(heading));
        let way = if matches!(probe, Walk | Move) {
            Way::Move(heading)
        } else {
            Way::Look
        };
        let dir = if grows(heading) { ONE } else { -ONE };
        let mut a = Axis {
            at: b.at,
            heading,
            way,
            dir,
            from,
            dist,
            across: across.pixel(),
            ra,
            rc,
        };
        if probe == Walk {
            a.dist = self.shortened(&a);
        }
        if !heading.vertical() {
            a.dist = a.dist.min(fixed(self.rules.move_limit_px, a.dist.sub()));
        }
        let stopped = match probe {
            _ if a.dist == ZERO => None,
            Enemies => self.enemies(&a),
            Blocks => self.blocks(&a),
            Walk | Move | Look => self.enemies(&a).or_else(|| self.blocks(&a)),
        };
        stopped.unwrap_or_else(|| a.put(a.ahead(a.to().pixel()), a.to().sub(), None))
    }

    /// `BlocksStopFlush`, `PoseFitAfterMoves`, `ShrinkingLowersHer`: a vertical move or look by
    /// `by`, signed, downward positive. A block look of 8 px or more is stopped by whatever a
    /// look of 8 px the same way would stop; only when nothing stops that is it the look of its
    /// own distance.
    pub fn vmove(&mut self, b: Bounds, by: Fixed, probe: Probe) -> Moved {
        let (heading, eight) = (if by.negative() { Up } else { Down }, self.rules.look);
        let first =
            (probe == Blocks && by.size() >= eight).then(|| self.go(b, heading, eight, probe));
        first
            .filter(|m| m.stop.is_some())
            .unwrap_or_else(|| self.go(b, heading, by.size(), probe))
    }

    /// `BlocksStopFlush`: the cells of her leading edge's line at the target; the first that
    /// stops her decides. `TouchedCells`: the cells it reaches, up to the one that stops it, with
    /// the way it reached them. `OtherBlockKinds`: an answered cell is air on a touch its reaction
    /// lets through.
    fn blocks(&mut self, a: &Axis) -> Option<Moved> {
        self.line(a).into_iter().find_map(|(k, at)| {
            let (block, at) = self.resolve(at);
            self.reached.push(Touch {
                cell: at,
                way: a.way,
            });
            let through = matches!(block, Answered) && self.world.lets_through(at, a.way);
            a.stop(block, k).filter(|_| !through)
        })
    }

    /// `SlopesCarry`: the two checks after every horizontal move, each asking one cell at her
    /// centre column. A sloped floor at her bottom edge's row, with that edge in its solid part,
    /// lifts her by whole pixels onto its last air row; then a sloped ceiling at her top edge's
    /// row, with that edge in its solid part or on the first air row under it, pushes her down
    /// onto that row. Either seats her: returns her y then, if she is seated.
    pub fn seat(&self, mut b: Bounds) -> Option<Fixed> {
        let lift = self.sloped(b, Down).filter(|p| *p != NONE);
        b.at.y -= px(lift.unwrap_or(NONE));
        let push = self.sloped(b, Up);
        lift.or(push).map(|_| b.at.y + px(push.unwrap_or(NONE)))
    }
}
