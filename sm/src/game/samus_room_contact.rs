//! One owned Samus movement/contact operation, including synchronous mechanism reactions.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
}
impl Axis {
    fn other(self) -> Self {
        match self {
            Self::X => Self::Y,
            Self::Y => Self::X,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Width,
    Direction,
    Block,
    Frame,
    Collision,
    SlopeAdjusted,
    Quicksand,
    SlopeBts,
    YSpeedFraction,
    YSpeed,
    MultiplyLeft,
    MultiplyRight,
    Area,
    Invincibility,
    KnockbackTimer,
    Damage,
    Knockback,
    Facing,
    Pose,
    DoorBlocked,
}
/// The scan's displacement and geometry have one owner across every setup and retry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scan {
    pub displacement: i32,
    target: u16,
    edge: u16,
    remaining: u16,
    span: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Result {
    pub blocked: bool,
    pub displacement: i32,
}
/// Player and room fields stay live; content and effects are explicit synchronous dependencies.
pub trait Context {
    type Door: Copy;
    type Header;
    fn read(&self, field: Field) -> u16;
    fn write(&mut self, field: Field, value: u16);
    fn position(&self, axis: Axis) -> u16;
    fn fraction(&self, axis: Axis) -> u16;
    fn radius(&self, axis: Axis) -> u16;
    fn set_fraction(&mut self, axis: Axis, value: u16);
    fn add_position(&mut self, axis: Axis, displacement: i32);
    fn level(&self, block: u16) -> u16;
    fn bts(&self, block: u16) -> u8;
    fn square_occupied(&self, index: u16) -> bool;
    fn slope_factor(&self, index: u8) -> u16;
    fn slope_height(&self, index: usize) -> u8;
    /// Preserve the ordered neighboring-byte physical reads, including bank crossings.
    fn vertical_slope_bts(&mut self, block: u16) -> u16;
    fn boss_defeated(&mut self) -> bool;
    fn special_header(&mut self, bts: u8, solid: bool) -> Self::Header;
    fn bomb_header(&mut self, bts: u8) -> Self::Header;
    fn setup(&mut self, header: Self::Header, displacement: &mut i32) -> u8;
    fn mark_door_contact(&mut self);
    fn door(&mut self, bts: u8) -> Self::Door;
    fn door_has_destination(&mut self, door: Self::Door) -> bool;
    fn enter_door(&mut self, door: Self::Door);
}
pub(crate) fn span(position: u16, radius: u16) -> u16 {
    radius
        .wrapping_add(position)
        .wrapping_sub(1)
        .wrapping_sub(position.wrapping_sub(radius) & 0xfff0)
        >> 4
}
fn boundary_delta(edge: u16, radius: u16, position: u16, negative: bool, mask: u16) -> i32 {
    let delta = if negative {
        radius
            .wrapping_add(edge | mask)
            .wrapping_add(1)
            .wrapping_sub(position)
    } else {
        (edge & !mask).wrapping_sub(radius).wrapping_sub(position)
    } as i16;
    i32::from(if negative { delta.min(0) } else { delta.max(0) }) << 16
}
fn solid(c: &mut impl Context, s: &mut Scan, axis: Axis, mask: u16) -> u8 {
    let negative = s.displacement < 0;
    let radius = c.radius(axis);
    let position = c.position(axis);
    s.displacement = boundary_delta(s.edge, radius, position, negative, mask);
    c.set_fraction(axis, if negative { 0 } else { 0xffff });
    1
}
fn square(c: &mut impl Context, s: &mut Scan, axis: Axis, slope: u16, block: u16) -> u8 {
    let orientation = u16::from(c.bts(block) >> 6);
    let (shift, other) = if axis == Axis::X { (3, 2) } else { (2, 1) };
    let index = slope
        .wrapping_mul(4)
        .wrapping_add(orientation ^ ((s.edge & 8) >> shift));
    let position = c.position(axis.other());
    let radius = c.radius(axis.other());
    let occupied = if s.remaining == 0 && radius.wrapping_add(position).wrapping_sub(1) & 8 == 0 {
        c.square_occupied(index)
    } else if s.remaining != 0 && s.remaining == s.span && position.wrapping_sub(radius) & 8 != 0 {
        c.square_occupied(index ^ other)
    } else {
        c.square_occupied(index) || c.square_occupied(index ^ other)
    };
    if !occupied {
        return 0;
    }
    let negative = s.displacement < 0;
    solid(c, s, axis, 7);
    if axis == Axis::Y && !negative {
        c.write(Field::SlopeAdjusted, 1);
    }
    1
}
fn nonsquare(c: &mut impl Context, s: &mut Scan, axis: Axis, block: u16) -> u8 {
    if axis == Axis::X {
        let bts = c.read(Field::SlopeBts);
        if bts & 0x80 != 0 {
            return 0;
        }
        let speed =
            u32::from(c.read(Field::YSpeedFraction)) | u32::from(c.read(Field::YSpeed)) << 16;
        if speed != 0 {
            return 0;
        }
        let displacement = s.displacement;
        let shifted = (displacement >> 8) as u16;
        let factor = u32::from(c.slope_factor((bts & 0x1f) as u8));
        let multiplicand = if displacement < 0 {
            shifted.wrapping_neg()
        } else {
            shifted
        };
        c.write(Field::MultiplyLeft, multiplicand);
        c.write(Field::MultiplyRight, factor as u16);
        s.displacement = if displacement >= 0 {
            (u32::from(shifted) * factor) as i32
        } else {
            ((u32::from(shifted.wrapping_neg()) * factor) as i32).wrapping_neg()
        };
        return 0;
    }
    let upward = c.read(Field::Direction) & 1 != 0;
    let x = c.position(Axis::X);
    let width = c.read(Field::Width) as u8;
    let current = c.read(Field::Block);
    let column = if width == 0 {
        current
    } else {
        current % u16::from(width)
    };
    if x >> 4 != column {
        return 0;
    }
    let bts = c.bts(block);
    if (bts & 0x80 != 0) == upward {
        return 0;
    }
    let offset = if bts & 0x40 != 0 { x ^ 15 } else { x };
    let height = c.slope_height(usize::from(bts & 0x1f) * 16 + usize::from(offset & 15)) & 0x1f;
    let radius = c.radius(Axis::Y);
    let offset = if upward {
        radius.wrapping_add(s.target).wrapping_sub(1) & 15
    } else {
        (s.target.wrapping_sub(radius) & 15) ^ 15
    };
    let adjustment = i16::from(height) - (offset as i16) - 1;
    if adjustment > 0 {
        return 0;
    }
    let displacement = ((s.displacement >> 16) as i16)
        .wrapping_add(adjustment)
        .max(0);
    s.displacement = i32::from(displacement) << 16;
    1
}
fn slope(c: &mut impl Context, s: &mut Scan, axis: Axis) -> u8 {
    let block = c.read(Field::Block);
    let kind = u16::from(c.bts(block) & 0x1f);
    if kind < 5 {
        let block = c.read(Field::Block);
        return square(c, s, axis, kind, block);
    }
    let bts = if axis == Axis::X {
        u16::from(c.bts(c.read(Field::Block)))
    } else {
        c.vertical_slope_bts(c.read(Field::Block))
    };
    c.write(Field::SlopeBts, bts);
    let block = c.read(Field::Block);
    nonsquare(c, s, axis, block)
}
fn spike_damage(c: &mut impl Context, amount: u16) {
    if c.read(Field::Invincibility) != 0 {
        return;
    }
    c.write(Field::Invincibility, 60);
    c.write(Field::KnockbackTimer, 10);
    let damage = c.read(Field::Damage);
    c.write(Field::Damage, damage.wrapping_add(amount));
    c.write(
        Field::Knockback,
        u16::from((c.read(Field::Facing) ^ 12) & 8 != 0),
    );
}
fn spike(c: &mut impl Context, s: &mut Scan, axis: Axis) -> u8 {
    let bts = c.bts(c.read(Field::Block));
    assert!(bts < 16, "spike-block BTS index out of bounds: {bts}");
    match bts {
        0 => {
            if c.read(Field::Area) != 3 || c.boss_defeated() {
                spike_damage(c, 60);
            }
        }
        1 | 3 => spike_damage(c, 16),
        _ => {}
    }
    solid(c, s, axis, 15)
}
fn door(c: &mut impl Context, s: &mut Scan, axis: Axis) -> u8 {
    c.mark_door_contact();
    let bts = c.bts(c.read(Field::Block));
    let door = c.door(bts);
    if !c.door_has_destination(door) {
        if c.read(Field::Pose) < 9 {
            c.write(Field::DoorBlocked, 1);
        }
        solid(c, s, axis, 15)
    } else {
        c.enter_door(door);
        0
    }
}
fn special(c: &mut impl Context, s: &mut Scan, axis: Axis, solid_block: bool) -> u8 {
    let bts = c.bts(c.read(Field::Block));
    let header = c.special_header(bts, solid_block);
    let result = c.setup(header, &mut s.displacement) & 1;
    if solid_block {
        if result != 0 {
            solid(c, s, axis, 15)
        } else {
            result
        }
    } else if bts & 0x80 != 0 {
        result
    } else {
        0
    }
}
fn bombable(c: &mut impl Context, s: &mut Scan, axis: Axis, solid_block: bool) -> u8 {
    let bts = c.bts(c.read(Field::Block));
    if bts & 0x80 != 0 {
        return if solid_block {
            solid(c, s, axis, 15)
        } else {
            0
        };
    }
    let header = c.bomb_header(bts);
    let result = c.setup(header, &mut s.displacement) & 1;
    if solid_block && result != 0 {
        solid(c, s, axis, 15)
    } else {
        0
    }
}
fn redirect(c: &mut impl Context, axis: Axis) -> bool {
    let step = i16::from(c.bts(c.read(Field::Block)) as i8);
    if step == 0 {
        return false;
    }
    let delta = if axis == Axis::X {
        step as u16
    } else {
        (step as u16).wrapping_mul(c.read(Field::Width))
    };
    let block = c.read(Field::Block);
    c.write(Field::Block, block.wrapping_add(delta));
    true
}
fn react(c: &mut impl Context, s: &mut Scan, axis: Axis, offset: u16) -> u8 {
    c.write(Field::Block, offset >> 1);
    loop {
        let material = c.level(c.read(Field::Block)) >> 12;
        match material {
            0 | 2 | 4 | 6 => return 0,
            1 => return slope(c, s, axis),
            3 => return special(c, s, axis, false),
            5 => {
                if !redirect(c, Axis::X) {
                    return 0;
                }
            }
            7 => return bombable(c, s, axis, false),
            8 | 12 | 14 => return solid(c, s, axis, 15),
            9 => return door(c, s, axis),
            10 => return spike(c, s, axis),
            11 => return special(c, s, axis, true),
            13 => {
                if !redirect(c, Axis::Y) {
                    return 0;
                }
            }
            15 => return bombable(c, s, axis, true),
            _ => unreachable!(),
        }
    }
}
fn traverse(c: &mut impl Context, amount: i32, axis: Axis, reverse: bool) -> Result {
    // Preserve initial observation order; callbacks subsequently mutate only live fields.
    let span = span(c.position(axis.other()), c.radius(axis.other()));
    let (target, edge, mut offset) = if axis == Axis::X {
        let y = c.position(Axis::Y);
        let radius = c.radius(Axis::Y);
        let row = (y.wrapping_sub(radius) >> 4) as u8;
        let width = c.read(Field::Width) as u8;
        let row_start = u16::from(row) * u16::from(width);
        let x = c.position(Axis::X);
        let fraction = c.fraction(Axis::X);
        let target =
            (((u32::from(x) << 16) | u32::from(fraction)).wrapping_add(amount as u32) >> 16) as u16;
        let radius = c.radius(Axis::X);
        let edge = if amount >= 0 {
            radius.wrapping_add(target).wrapping_sub(1)
        } else {
            target.wrapping_sub(radius)
        };
        (
            target,
            edge,
            row_start.wrapping_add(edge >> 4).wrapping_mul(2),
        )
    } else {
        let y = c.position(Axis::Y);
        let fraction = c.fraction(Axis::Y);
        let target =
            (((u32::from(y) << 16) | u32::from(fraction)).wrapping_add(amount as u32) >> 16) as u16;
        let radius = c.radius(Axis::Y);
        let edge = if amount >= 0 {
            radius.wrapping_add(target).wrapping_sub(1)
        } else {
            target.wrapping_sub(radius)
        };
        let row = (edge >> 4) as u8;
        let width = c.read(Field::Width) as u8;
        let row_start = u16::from(row) * u16::from(width);
        let x = c.position(Axis::X);
        let radius = c.radius(Axis::X);
        let column = if reverse {
            radius.wrapping_add(x).wrapping_sub(1) >> 4
        } else {
            x.wrapping_sub(radius) >> 4
        };
        c.write(Field::Block, row_start.wrapping_add(column));
        (target, edge, c.read(Field::Block).wrapping_mul(2))
    };
    let mut scan = Scan {
        displacement: amount,
        target,
        edge,
        remaining: if reverse { 0 } else { span },
        span,
    };
    loop {
        if react(c, &mut scan, axis, offset) & 1 != 0 {
            return Result {
                blocked: true,
                displacement: scan.displacement,
            };
        }
        if axis == Axis::X {
            offset = offset.wrapping_add(c.read(Field::Width).wrapping_mul(2));
            scan.remaining = scan.remaining.wrapping_sub(1);
            if scan.remaining & 0x8000 != 0 {
                break;
            }
        } else if reverse {
            scan.remaining = scan.remaining.wrapping_add(1);
            if scan.span < scan.remaining {
                break;
            }
            offset = offset.wrapping_sub(2);
        } else {
            scan.remaining = scan.remaining.wrapping_sub(1);
            if scan.remaining & 0x8000 != 0 {
                break;
            }
            offset = offset.wrapping_add(2);
        }
    }
    Result {
        blocked: false,
        displacement: scan.displacement,
    }
}
fn pose_probe(c: &mut impl Context, amount: i32, axis: Axis) -> i32 {
    let direction = c.read(Field::Direction);
    c.write(Field::Direction, direction | 15);
    c.write(Field::Quicksand, 0);
    let reverse = axis == Axis::Y && c.read(Field::Frame) & 1 != 0;
    let result = traverse(c, amount, axis, reverse);
    c.write(Field::Collision, u16::from(result.blocked));
    result.displacement.wrapping_abs()
}
pub fn wall_jump(c: &mut impl Context, amount: i32) -> i32 {
    pose_probe(c, amount, Axis::X)
}
pub fn changed_pose(c: &mut impl Context, amount: i32) -> i32 {
    let integer = (amount >> 16) as i16;
    if integer.wrapping_abs() as u16 & 0xfff8 == 0 {
        return pose_probe(c, amount, Axis::Y);
    }
    let probe = i32::from(((integer as u16 & 0xfff0) | 8) as i16) << 16;
    let adjusted = pose_probe(c, probe, Axis::Y);
    if c.read(Field::Collision) != 0 {
        return adjusted;
    }
    pose_probe(c, amount, Axis::Y)
}
pub fn move_right(c: &mut impl Context, mut amount: i32) -> i32 {
    let blocked = if amount != 0 {
        c.write(Field::Quicksand, 0);
        let result = traverse(c, amount, Axis::X, false);
        amount = result.displacement;
        result.blocked
    } else {
        false
    };
    c.write(Field::Collision, u16::from(blocked));
    c.add_position(Axis::X, amount);
    amount
}
pub fn move_down(c: &mut impl Context, mut amount: i32) -> i32 {
    if amount != 0 {
        c.write(Field::SlopeAdjusted, 0);
        c.write(Field::Quicksand, 0);
        let reverse = c.read(Field::Frame) & 1 != 0;
        let result = traverse(c, amount, Axis::Y, reverse);
        amount = result.displacement;
        if result.blocked {
            c.write(Field::Collision, 1);
            c.add_position(Axis::Y, amount);
            return amount;
        }
    }
    let sand = c.read(Field::Quicksand);
    c.write(Field::Collision, u16::from(sand != 0));
    c.add_position(Axis::Y, amount);
    amount
}
