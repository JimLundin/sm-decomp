//! Samus's block collision: a move along one axis, stopped or bent by the blocks it reaches.
//!
//! Bank $94 log (<https://patrickjohnston.org/bank/94>): "Move Samus right" (`$94:971E`) and
//! "Move Samus down" (`$94:9763`) by a signed distance, with no solid enemy collision. Each
//! scans the blocks Samus's leading edge reaches across her span (`$94:9543` a column of rows;
//! `$94:959E` and `$94:95F5` a row, left to right on even frames and right to left on odd
//! ones), dispatching on each block's type (`$94:94D5` horizontal, `$94:94F5` vertical):
//! types 0-7 let her through, the extensions stand for the block their BTS points at
//! (`$94:9411`, `$94:9447`), slopes test their shape (`$94:8FBB`, `$94:8FDA`), and the
//! solid ones stop her at the block's edge (`$94:8F49`, `$94:8F82`). She then moves by the
//! distance as the scan left it.

use data::level::slope::{Shape, Slopes, Square, Surface};
use data::level::{BlockKind, Bts, Offset, PlmChoice, Slope};

use crate::plm::{self, TouchFault};
use crate::state::room::{Block, BlockColumns, BlockIndex, RoomScoped};
use crate::state::samus::{Fixed, Point, Speed};

pub mod inside;

/// What a move went through.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Room<'a> {
    /// The level's blocks, row-major.
    pub blocks: &'a [Block],
    /// Blocks a row.
    pub width: BlockColumns,
    /// The slope shapes.
    pub slopes: Option<&'a Slopes>,
}

/// The special block PLM choice of an item's block, BTS `$45`: its collision reactions spawn
/// the item collision detection PLM (`$94:9139` + `$45` * 2 = `$84:EED3`), whose setup lets
/// her through.
const ITEM_BLOCK: PlmChoice = PlmChoice::Common(0x45);

/// The special air BTS that triggers a scroll PLM: its collision reactions spawn the scroll
/// block touch PLM (`$94:9139` + `$46` * 2 = `$84:B6FF`), and so does its inside reaction at
/// Samus's centre (`$94:9956`).
const SCROLL_TRIGGER: Bts = Bts(0x46);

/// A block whose collision reaction spawns a PLM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Touch {
    /// A scroll trigger: the scroll block touch PLM (`$84:B6FF`).
    Scroll(BlockIndex),
    /// An item's block: the item collision detection PLM (`$84:EED3`).
    Item(BlockIndex),
}

/// The blocks a frame's moves reached whose reactions spawn a PLM, in the order their scans
/// reached them.
///
/// The collision reactions spawn the PLM during the scan (`$94:906F`, `$94:909D` for special
/// air, `$94:90CB`, `$94:9102` for special blocks, through `$84:84E7`), but the setups
/// (`$84:B393`, `$84:EEAB`) change only PLM slots, which no move reads, and the item's lets
/// her through (carry clear), so the touches apply after the moves in the same order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Touches(Vec<Touch>);

impl Touches {
    /// The touches, in order.
    #[must_use]
    pub fn touched(&self) -> &[Touch] {
        &self.0
    }

    /// Spawn each touch's PLM in order.
    ///
    /// # Errors
    ///
    /// [`TouchFault`] where a touch finds no PLM of its kind on its block.
    pub fn apply(self, room: &mut RoomScoped) -> Result<(), TouchFault> {
        self.0.into_iter().try_for_each(|touch| match touch {
            Touch::Scroll(block) => plm::touch_scroll_block(room, block),
            Touch::Item(block) => plm::touch_item_block(room, block),
        })
    }
}

/// Which way a vertical scan crosses Samus's span (`$94:9775`: by the frame counter's parity).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScanOrder {
    /// Even frames.
    #[default]
    LeftToRight,
    /// Odd frames.
    RightToLeft,
}

impl ScanOrder {
    /// The order on a frame whose requested-NMI count is `frame`.
    #[must_use]
    pub const fn of(frame: u16) -> Self {
        if frame & 1 == 0 {
            Self::LeftToRight
        } else {
            Self::RightToLeft
        }
    }
}

/// Where the frame's moves happen: the room and the vertical scans' order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Surroundings<'a> {
    /// The room's blocks.
    pub room: Room<'a>,
    /// The frame's vertical scan order.
    pub order: ScanOrder,
}

/// The way the collision direction (`$0B02`) says she moves vertically.
///
/// The non-square slopes read it, not the distance's sign (`$94:86FE`), and the elevator's
/// step down moves her down with the direction up (`$90:A397`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Heading {
    /// Up.
    Up,
    /// Down.
    Down,
}

/// Samus as her block collision sees her.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Body {
    /// Her X radius.
    pub x_radius: u16,
    /// Her pose's Y radius.
    pub y_radius: u16,
    /// Whether her Y speed is 0.0, which a walk along a non-square slope needs (`$94:84DE`).
    pub still_vertically: bool,
}

/// What a move did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Moved {
    /// The distance she moved, signed.
    pub distance: Speed,
    /// Whether a block stopped her (the routines' carry).
    pub collided: bool,
}

/// The side a wall stopped her on: the X speed killed flag (`$0DCE`), 4 left and 8 right
/// (bank $90 log, `$90:E5CE`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Moving left.
    Left,
    /// Moving right.
    Right,
}

/// The solid vertical collision result (`$0DC6`, bank $90 log, `$90:E606`, `$90:E61B`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VerticalContact {
    /// 0: no change.
    #[default]
    NoChange,
    /// 1: a downwards move was stopped.
    Landed,
    /// 2: a downwards move wasn't.
    Falling,
    /// 4: an upwards move was stopped.
    HitCeiling,
    /// 5: the spin jump's wall jump check found a wall to jump from (`$90:9E7F`), and she
    /// didn't move vertically.
    WallJumpTriggered,
}

/// What the frame's moves met, for the pose update to read (bank $91 log, `$91:E8B6` and
/// `$91:EADE`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Contacts {
    /// The wall a horizontal move ran into.
    pub wall: Option<Side>,
    /// The last vertical move's result.
    pub vertical: VerticalContact,
}

/// One axis's names for the routines' shared shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Axis {
    X,
    Y(Heading),
}

/// The scan's working values (`$12.$14`, `$18`, `$1A`, `$1C`, `$20`).
struct Scan<'a, 't> {
    room: Room<'a>,
    /// The scroll triggers reached so far.
    touches: &'t mut Touches,
    body: Body,
    axis: Axis,
    /// She moves by this, as the reactions adjust it.
    distance: Speed,
    /// The target position's pixel word.
    target: u16,
    /// The target boundary: her leading edge at the target.
    edge: u16,
    /// Blocks left after this one, from the span down (0 is the bottom or rightmost block).
    remaining: u16,
    /// Her span in blocks across the scan, less one.
    span: u16,
}

/// Whether a signed word is negative, as `BMI` tests it.
const fn negative(word: u16) -> bool {
    word & 0x8000 != 0
}

/// `max(0, word)` on a signed word.
const fn at_least_zero(word: u16) -> u16 {
    if negative(word) { 0 } else { word }
}

/// `min(0, word)` on a signed word.
const fn at_most_zero(word: u16) -> u16 {
    if negative(word) { word } else { 0 }
}

/// A position plus a distance, carried from the subpixels.
const fn advance(position: Fixed, distance: Speed) -> Fixed {
    let moved = Speed(
        Speed::new(position.pixels, position.subpixels)
            .0
            .wrapping_add(distance.0),
    );
    Fixed {
        pixels: moved.pixels(),
        subpixels: moved.subpixels(),
    }
}

/// Her span across an axis in blocks, less one (`$94:9495`, `$94:94B5`).
const fn block_span(position: u16, radius: u16) -> u16 {
    let near = position.wrapping_sub(radius) & 0xfff0;
    position
        .wrapping_add(radius)
        .wrapping_sub(1)
        .wrapping_sub(near)
        >> 4
}

/// The leading edge at `target` (`$20`): her near side moving back, her far side forwards.
const fn leading_edge(target: u16, radius: u16, distance: Speed) -> u16 {
    if negative(distance.pixels()) {
        target.wrapping_sub(radius)
    } else {
        target.wrapping_add(radius).wrapping_sub(1)
    }
}

/// The first block of a row: the multiplier takes the row's and the width's low bytes.
fn row_start(row: u16, width: BlockColumns) -> u16 {
    let [row, _] = row.to_le_bytes();
    let [width, _] = width.0.to_le_bytes();
    u16::from(row).wrapping_mul(u16::from(width))
}

impl Room<'_> {
    /// The block at `index`; past the level, air.
    // open: the original reads whatever work RAM follows the level there.
    fn kind(&self, index: u16) -> BlockKind {
        self.blocks
            .get(usize::from(index))
            .map_or_else(BlockKind::default, |block| block.kind)
    }

    /// `index`'s column: the divider takes the width's low byte, and a zero width leaves the
    /// dividend as the remainder.
    fn column(&self, index: u16) -> u16 {
        let [width, _] = self.width.0.to_le_bytes();
        index.checked_rem(u16::from(width)).unwrap_or(index)
    }
}

impl Scan<'_, '_> {
    /// The reaction to the block at `index` (`$94:9515`, `$94:952C`): whether it stops her.
    fn react(&mut self, position: &mut Point, index: u16) -> bool {
        let mut current = index;
        loop {
            match self.room.kind(current) {
                BlockKind::HorizontalExtension(Offset(0))
                | BlockKind::VerticalExtension(Offset(0)) => {
                    return false;
                }
                BlockKind::HorizontalExtension(Offset(offset)) => {
                    current = current.wrapping_add_signed(i16::from(offset));
                }
                BlockKind::VerticalExtension(Offset(offset)) => {
                    let rows = i16::from(offset).cast_unsigned();
                    current = current.wrapping_add(rows.wrapping_mul(self.room.width.0));
                }
                // Special air with BTS `$46` spawns the scroll block touch PLM on the block
                // the extensions led to (`$0DC4`) and lets her through (`$94:906F`,
                // `$94:909D`): the elevator ride's step reaches one with her bottom
                // (`area-brinstar` frame 92), when the inside handling doesn't run.
                BlockKind::SpecialAir(SCROLL_TRIGGER) => {
                    self.touches.0.push(Touch::Scroll(BlockIndex(current)));
                    return false;
                }
                // A special block with BTS `$45`, an item's drawn block, spawns the item
                // collision detection PLM on it and lets her through (`$94:90CB`, `$94:9102`):
                // `item-morph-ball` frame 817.
                BlockKind::Special(ITEM_BLOCK) => {
                    self.touches.0.push(Touch::Item(BlockIndex(current)));
                    return false;
                }
                // open: the other area independent special air spawns its PLM from the same
                // table and lets her through, area dependent special air (BTS bit 7) spawns
                // its area's, whose setup can stop her (`$94:9087`), and bombable air
                // (`$94:92F9`, `$94:9313`) its PLM; the spike air reaction does nothing
                // (`$94:901A`). Slice 0's rooms have no other special air and no bombable air.
                BlockKind::Air(_)
                | BlockKind::SpikeAir(_)
                | BlockKind::SpecialAir(_)
                | BlockKind::ShootableAir(_)
                | BlockKind::UnusedAir(_)
                | BlockKind::BombableAir(_) => return false,
                BlockKind::Slope(slope) => return self.slope(position, current, slope),
                // open: a door to a room starts the door transition and lets her through
                // (`$94:938B`, `$94:93CE`), a door to none arms an elevator under her standing
                // pose; spikes hurt her (`$94:904B`); special blocks and bombable blocks with a
                // PLM spawn it, which decides (`$94:90CB`, `$94:932D`). Slice 0 crosses no door
                // and its room's bombable blocks are bomb blocks, solid to the touch.
                BlockKind::Solid(_)
                | BlockKind::Door(_)
                | BlockKind::Spike(_)
                | BlockKind::Special(_)
                | BlockKind::Shootable(_)
                | BlockKind::Grapple(_)
                | BlockKind::Bombable(_) => {
                    self.solid(position);
                    return true;
                }
            }
        }
    }

    /// The solid block reaction (`$94:8F49`, `$94:8F82`): she stops at the block's edge, at
    /// the subpixel next to it.
    const fn solid(&mut self, position: &mut Point) {
        let (fixed, radius) = self.axis_of(position);
        let backwards = negative(self.distance.pixels());
        let pixels = if backwards {
            at_most_zero(
                (self.edge | 0x000f)
                    .wrapping_add(1)
                    .wrapping_add(radius)
                    .wrapping_sub(fixed.pixels),
            )
        } else {
            at_least_zero(
                (self.edge & 0xfff0)
                    .wrapping_sub(radius)
                    .wrapping_sub(fixed.pixels),
            )
        };
        fixed.subpixels = if backwards { 0 } else { 0xffff };
        self.distance = Speed::new(pixels, 0);
    }

    /// The scanned axis's position and radius.
    const fn axis_of<'p>(&self, position: &'p mut Point) -> (&'p mut Fixed, u16) {
        match self.axis {
            Axis::X => (&mut position.x, self.body.x_radius),
            Axis::Y(_) => (&mut position.y, self.body.y_radius),
        }
    }

    /// The slope reaction (`$94:8FBB`, `$94:8FDA`): by its shape.
    fn slope(&mut self, position: &mut Point, index: u16, slope: Slope) -> bool {
        match self
            .room
            .slopes
            .and_then(|slopes| slopes.shape(slope.shape))
        {
            Some(Shape::Square(square)) => self.square(position, slope, square),
            Some(Shape::Surface(surface)) => match self.axis {
                Axis::X => {
                    self.walk(slope, surface);
                    false
                }
                Axis::Y(heading) => self.surface(*position, index, slope, surface, heading),
            },
            None => false,
        }
    }

    /// A square slope (`$94:8D2B`, `$94:8DBD`): the half of the block her edge is in, in the
    /// one or two quadrants of it her span covers, by the shape's flips.
    fn square(&mut self, position: &mut Point, slope: Slope, square: Square) -> bool {
        let far_half = self.edge & 0x0008 != 0;
        let (across, radius) = match self.axis {
            Axis::X => (position.y.pixels, self.body.y_radius),
            Axis::Y(_) => (position.x.pixels, self.body.x_radius),
        };
        // The quadrants across the scan: the near one (top or left) and the far one.
        let (near, far) = if self.remaining == 0 {
            let far_side = across.wrapping_add(radius).wrapping_sub(1) & 0x0008 != 0;
            (true, far_side)
        } else if self.remaining == self.span {
            let near_side = across.wrapping_sub(radius) & 0x0008 != 0;
            (!near_side, true)
        } else {
            (true, true)
        };
        let solid = |far_across: bool| match self.axis {
            Axis::X => square.solid_at(far_half ^ slope.x_flip, far_across ^ slope.y_flip),
            Axis::Y(_) => square.solid_at(far_across ^ slope.x_flip, far_half ^ slope.y_flip),
        };
        if !((near && solid(false)) || (far && solid(true))) {
            return false;
        }
        let (fixed, radius) = self.axis_of(position);
        let backwards = negative(self.distance.pixels());
        let pixels = if backwards {
            at_most_zero(
                (self.edge | 0x0007)
                    .wrapping_add(1)
                    .wrapping_add(radius)
                    .wrapping_sub(fixed.pixels),
            )
        } else {
            // The second subtraction takes the first one's borrow (no SEC between them).
            let (rest, borrow) = (self.edge & 0xfff8).overflowing_sub(radius);
            at_least_zero(
                rest.wrapping_sub(fixed.pixels)
                    .wrapping_sub(u16::from(borrow)),
            )
        };
        fixed.subpixels = if backwards { 0 } else { 0xffff };
        self.distance = Speed::new(pixels, 0);
        true
    }

    /// Walking onto a non-square slope (`$94:84D6`): on a floor slope with no Y speed her
    /// distance scales by the shape's factor; she isn't stopped.
    fn walk(&mut self, slope: Slope, surface: Surface) {
        if slope.y_flip || !self.body.still_vertically {
            return;
        }
        // The distance in 1/256 pixels, a word: the pixel word's low byte and the subpixels'
        // high byte.
        let backwards = negative(self.distance.pixels());
        let [_, sub_high] = self.distance.subpixels().to_le_bytes();
        let [pixel_low, _] = self.distance.pixels().to_le_bytes();
        let fine = u16::from_le_bytes([sub_high, pixel_low]);
        let magnitude = if backwards { fine.wrapping_neg() } else { fine };
        let product = u32::from(magnitude).wrapping_mul(u32::from(surface.factor));
        self.distance = Speed(if backwards {
            product.wrapping_neg()
        } else {
            product
        });
    }

    /// A non-square slope met vertically (`$94:86FE`): only in Samus's own column; a floor
    /// slope moving down, a ceiling slope moving up, stops her where her edge meets the
    /// surface.
    fn surface(
        &mut self,
        position: Point,
        index: u16,
        slope: Slope,
        surface: Surface,
        heading: Heading,
    ) -> bool {
        let x = position.x.pixels;
        if self.room.column(index) != x >> 4 {
            return false;
        }
        let radius = self.body.y_radius;
        let (depth, ceiling) = match heading {
            Heading::Up => ((self.target.wrapping_sub(radius) & 0x000f) ^ 0x000f, true),
            Heading::Down => (
                self.target.wrapping_add(radius).wrapping_sub(1) & 0x000f,
                false,
            ),
        };
        if slope.y_flip != ceiling {
            return false;
        }
        let column = if slope.x_flip { x ^ 0x000f } else { x };
        let past = u16::from(surface.top(column))
            .wrapping_sub(depth)
            .wrapping_sub(1);
        if past != 0 && !negative(past) {
            return false;
        }
        self.distance = Speed::new(at_least_zero(self.distance.pixels().wrapping_add(past)), 0);
        true
    }
}

/// "Move Samus right by [$12].[$14], no solid enemy collision" (`$94:971E`): `distance`
/// signed, through the blocks of her rows (`$94:9543`).
///
/// The scroll triggers it reaches join `touches`.
pub fn move_x(
    room: Room<'_>,
    body: Body,
    position: &mut Point,
    distance: Speed,
    touches: &mut Touches,
) -> Moved {
    if distance == Speed::ZERO {
        return Moved {
            distance,
            collided: false,
        };
    }
    let moved = scan_x(room, body, position, distance, touches);
    position.x = advance(position.x, moved.distance);
    moved
}

/// "Samus block collision detection - horizontal" (`$94:9543`): the blocks of her rows at
/// her leading edge `distance` (signed) away.
///
/// The move and the wall jump check (`$94:967F`) share it. She doesn't move, but a solid
/// block puts her X subpixels next to it. The touches it reaches join `touches`.
pub fn scan_x(
    room: Room<'_>,
    body: Body,
    position: &mut Point,
    distance: Speed,
    touches: &mut Touches,
) -> Moved {
    let y = position.y.pixels;
    let first = row_start(y.wrapping_sub(body.y_radius) >> 4, room.width);
    let target = advance(position.x, distance).pixels;
    let edge = leading_edge(target, body.x_radius, distance);
    let breadth = block_span(y, body.y_radius);
    let mut scan = Scan {
        room,
        touches,
        body,
        axis: Axis::X,
        distance,
        target,
        edge,
        remaining: breadth,
        span: breadth,
    };
    let mut offset = (edge >> 4).wrapping_add(first).wrapping_mul(2);
    let row = room.width.0.wrapping_mul(2);
    let collided = loop {
        if scan.react(position, offset >> 1) {
            break true;
        }
        offset = offset.wrapping_add(row);
        scan.remaining = scan.remaining.wrapping_sub(1);
        if negative(scan.remaining) {
            break false;
        }
    };
    Moved {
        distance: scan.distance,
        collided,
    }
}

/// "Move Samus down by [$12].[$14], no solid enemy collision" (`$94:9763`): `distance`
/// signed, through the blocks of her columns in the frame's `order` (`$94:959E`,
/// `$94:95F5`).
///
/// The scroll triggers it reaches join `touches`.
// open: the "position was adjusted by a slope" flag (`$0DBA`) and the quicksand flag
// (`$1E71`) join with the slope alignment and quicksand.
pub fn move_y(
    room: Room<'_>,
    body: Body,
    position: &mut Point,
    distance: Speed,
    heading: Heading,
    order: ScanOrder,
    touches: &mut Touches,
) -> Moved {
    if distance == Speed::ZERO {
        return Moved {
            distance,
            collided: false,
        };
    }
    let x = position.x.pixels;
    let target = advance(position.y, distance).pixels;
    let edge = leading_edge(target, body.y_radius, distance);
    let first = row_start(edge >> 4, room.width);
    let breadth = block_span(x, body.x_radius);
    let (column, remaining) = match order {
        ScanOrder::LeftToRight => (x.wrapping_sub(body.x_radius) >> 4, breadth),
        ScanOrder::RightToLeft => (x.wrapping_add(body.x_radius).wrapping_sub(1) >> 4, 0),
    };
    let mut scan = Scan {
        room,
        touches,
        body,
        axis: Axis::Y(heading),
        distance,
        target,
        edge,
        remaining,
        span: breadth,
    };
    let mut offset = first.wrapping_add(column).wrapping_mul(2);
    let collided = loop {
        if scan.react(position, offset >> 1) {
            break true;
        }
        match order {
            ScanOrder::LeftToRight => {
                offset = offset.wrapping_add(2);
                scan.remaining = scan.remaining.wrapping_sub(1);
                if negative(scan.remaining) {
                    break false;
                }
            }
            ScanOrder::RightToLeft => {
                offset = offset.wrapping_sub(2);
                scan.remaining = scan.remaining.wrapping_add(1);
                if scan.span < scan.remaining {
                    break false;
                }
            }
        }
    };
    position.y = advance(position.y, scan.distance);
    Moved {
        distance: scan.distance,
        collided,
    }
}

#[cfg(test)]
mod tests;
