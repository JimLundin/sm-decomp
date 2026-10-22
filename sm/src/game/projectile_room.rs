//! Projectile motion and ordered room scans, independent of state storage.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Axis {
    X,
    Y,
}
impl Axis {
    pub fn other(self) -> Self {
        match self {
            Self::X => Self::Y,
            Self::Y => Self::X,
        }
    }
}
#[derive(Clone, Copy)]
pub enum Field {
    Position,
    Fraction,
    Radius,
    Speed,
    // Wave clipping reads the stored high byte, independently of native word order.
    ScreenHigh,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Beam,
    Wave,
    Missile,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scan {
    pub displacement: i32,
    pub position: u16,
    pub span: u16,
    pub edge: u16,
    pub missile: u16,
    pub auxiliary: u16,
    pub remaining: u16,
    pub unblocked: u16,
}
pub trait State {
    fn field(&self, slot: u16, axis: Axis, field: Field) -> u16;
    /// Commit fraction before position, before any following field or room reads.
    fn move_by(&mut self, slot: u16, axis: Axis, amount: i32);
}
pub trait Context<O> {
    fn width(&self, owner: &O) -> u16;
    fn scrolls(&self, owner: &O, axis: Axis) -> u16;
    fn react(&mut self, owner: O, axis: Axis, block: u16, scan: Scan) -> (O, Scan, u8);
    fn retire(&mut self, owner: O, slot: u16) -> O;
}
pub fn span(owner: &impl State, slot: u16, axis: Axis) -> u16 {
    let other = axis.other();
    let first = owner
        .field(slot, other, Field::Position)
        .wrapping_sub(owner.field(slot, other, Field::Radius))
        & 0xfff0;
    owner
        .field(slot, other, Field::Radius)
        .wrapping_add(owner.field(slot, other, Field::Position))
        .wrapping_sub(1)
        .wrapping_sub(first)
        >> 4
}
pub fn collide<O: State>(
    mut owner: O,
    slot: u16,
    axis: Axis,
    kind: Kind,
    context: &mut impl Context<O>,
) -> (O, u8) {
    let missile = kind == Kind::Missile;
    let wave = kind == Kind::Wave;
    let amount = i32::from(owner.field(slot, axis, Field::Speed) as i16) << 8;
    let span = if missile { 0 } else { span(&owner, slot, axis) };
    let row = if axis == Axis::X {
        let y = owner.field(slot, Axis::Y, Field::Position);
        let edge = if missile {
            y
        } else {
            y.wrapping_sub(owner.field(slot, Axis::Y, Field::Radius))
        };
        u16::from((edge >> 4) as u8) * u16::from(context.width(&owner) as u8)
    } else {
        0
    };
    owner.move_by(slot, axis, amount);
    if missile && axis == Axis::X {
        let _fraction = owner.field(slot, Axis::X, Field::Fraction);
    }
    let position = owner.field(slot, axis, Field::Position);
    let edge = if missile {
        position
    } else if amount < 0 {
        owner
            .field(slot, axis, Field::Position)
            .wrapping_sub(owner.field(slot, axis, Field::Radius))
    } else {
        owner
            .field(slot, axis, Field::Radius)
            .wrapping_add(owner.field(slot, axis, Field::Position))
            .wrapping_sub(1)
    };
    let mut block = if axis == Axis::X {
        let x = if missile {
            owner.field(slot, Axis::X, Field::Position)
        } else {
            edge
        };
        row.wrapping_add(x >> 4).wrapping_mul(2)
    } else if missile {
        let row = u16::from((owner.field(slot, Axis::Y, Field::Position) >> 4) as u8)
            * u16::from(context.width(&owner) as u8);
        row.wrapping_add(owner.field(slot, Axis::X, Field::Position) >> 4)
            .wrapping_mul(2)
    } else {
        let column = |owner: &O| {
            owner
                .field(slot, Axis::X, Field::Position)
                .wrapping_sub(owner.field(slot, Axis::X, Field::Radius))
                >> 4
        };
        // Non-wave reads the column before room width; wave reads it afterward.
        let before = if wave { 0 } else { column(&owner) };
        let row = u16::from((edge >> 4) as u8) * u16::from(context.width(&owner) as u8);
        row.wrapping_add(if wave { column(&owner) } else { before })
            .wrapping_mul(2)
    };
    let outside = |high: u16, scrolls: u16| (high as i16).wrapping_sub(scrolls as i16) >= 0;
    if !missile && span.wrapping_sub(16) & 0x8000 == 0 {
        return (owner, 0);
    }
    if !wave && outside(u16::from((edge >> 8) as u8), context.scrolls(&owner, axis)) {
        return (owner, 0);
    }
    let mut scan = Scan {
        displacement: amount,
        position,
        span,
        edge,
        missile: u16::from(missile),
        auxiliary: 0,
        remaining: span,
        unblocked: span,
    };
    if wave {
        let high = owner.field(slot, axis.other(), Field::ScreenHigh);
        if high & 0x80 != 0
            || outside(high, context.scrolls(&owner, axis.other()))
            || outside(u16::from((edge >> 8) as u8), context.scrolls(&owner, axis))
        {
            return (owner, 0);
        }
    }
    if missile {
        let (next, _, hit) = context.react(owner, axis, block, scan);
        owner = next;
        if hit == 0 {
            return (owner, 0);
        }
    } else {
        loop {
            let (next, returned, _) = context.react(owner, axis, block, scan);
            owner = next;
            scan = returned;
            block = block.wrapping_add(if axis == Axis::X {
                context.width(&owner).wrapping_mul(2)
            } else {
                2
            });
            scan.remaining = scan.remaining.wrapping_sub(1);
            if scan.remaining & 0x8000 != 0 {
                break;
            }
        }
        if wave || scan.unblocked & 0x8000 == 0 {
            return (owner, 0);
        }
    }
    (context.retire(owner, slot), 1)
}
