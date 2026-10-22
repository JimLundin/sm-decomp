//! Trail-pair reservation and placement independent of the byte representation.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Side<C, S = u16> {
    pub remaining: u16,
    pub cursor: C,
    pub tile: S,
    pub position: Point,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pair<C, S = u16> {
    pub left: Side<C, S>,
    pub right: Side<C, S>,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pool<C, S = u16> {
    pub pairs: [Pair<C, S>; 18],
}
pub enum Write<C, S> {
    Timer(u16),
    Cursor(C),
    Tile(S),
    X(u16),
    Y(u16),
}
pub trait Owner: Sized {
    type Cursor: Copy;
    type Sprite: Copy;
    fn remaining(&self, pair: usize, half: Half) -> u16;
    fn cursor(&self, pair: usize, half: Half) -> Self::Cursor;
    fn y(&self, pair: usize, half: Half) -> u16;
    fn write(&mut self, pair: usize, half: Half, value: Write<Self::Cursor, Self::Sprite>);
}
impl<C: Copy, S: Copy> Pool<C, S> {
    pub fn side(&self, pair: usize, half: Half) -> &Side<C, S> {
        match half {
            Half::Left => &self.pairs[pair].left,
            Half::Right => &self.pairs[pair].right,
        }
    }
    fn side_mut(&mut self, pair: usize, half: Half) -> &mut Side<C, S> {
        match half {
            Half::Left => &mut self.pairs[pair].left,
            Half::Right => &mut self.pairs[pair].right,
        }
    }
}
impl<C: Copy, S: Copy> Owner for Pool<C, S> {
    type Cursor = C;
    type Sprite = S;
    fn remaining(&self, pair: usize, half: Half) -> u16 {
        self.side(pair, half).remaining
    }
    fn cursor(&self, pair: usize, half: Half) -> C {
        self.side(pair, half).cursor
    }
    fn y(&self, pair: usize, half: Half) -> u16 {
        self.side(pair, half).position.y
    }
    fn write(&mut self, pair: usize, half: Half, value: Write<C, S>) {
        let side = self.side_mut(pair, half);
        match value {
            Write::Timer(v) => side.remaining = v,
            Write::Cursor(v) => side.cursor = v,
            Write::Tile(v) => side.tile = v,
            Write::X(v) => side.position.x = v,
            Write::Y(v) => side.position.y = v,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Half {
    Left,
    Right,
}
pub trait Offsets {
    fn left_y(&self) -> i8;
    fn left_x(&self) -> i8;
    fn right_y(&self) -> i8;
    fn right_x(&self) -> i8;
}
impl Offsets for [i8; 4] {
    fn left_y(&self) -> i8 {
        self[1]
    }
    fn left_x(&self) -> i8 {
        self[0]
    }
    fn right_y(&self) -> i8 {
        self[3]
    }
    fn right_x(&self) -> i8 {
        self[2]
    }
}
pub enum Coordinate {
    LeftY,
    LeftX,
    RightY,
    RightX,
}
pub trait Context<O: Owner> {
    type Style: Copy;
    type Marker;
    type Offsets: Offsets;
    /// Publish committed fields if an encoded definition cannot supply the next coordinate.
    fn validate(&mut self, owner: O, _: &Self::Offsets, _: Coordinate) -> O {
        owner
    }
    fn style(&self, owner: &O, argument: u16) -> Option<Self::Style>;
    fn script(&mut self, owner: O, style: Self::Style, half: Half) -> (O, O::Cursor);
    fn selected_projectile(&self, owner: &O) -> u16;
    fn marker(&mut self, owner: O, projectile: u16) -> (O, Self::Marker);
    fn origin(&mut self, owner: O, projectile: u16) -> (O, Point);
    fn offsets(&mut self, owner: O, projectile: u16, marker: Self::Marker) -> (O, Self::Offsets);
}
pub fn emit<O: Owner>(mut owner: O, argument: u16, context: &mut impl Context<O>) -> O {
    let Some(style) = context.style(&owner, argument) else {
        return owner;
    };
    let Some(pair) = (0..18).rev().find(|&i| owner.remaining(i, Half::Left) == 0) else {
        return owner;
    };
    owner.write(pair, Half::Left, Write::Timer(1));
    owner.write(pair, Half::Right, Write::Timer(1));
    let (next, script) = context.script(owner, style, Half::Left);
    owner = next;
    owner.write(pair, Half::Left, Write::Cursor(script));
    let (next, script) = context.script(owner, style, Half::Right);
    owner = next;
    owner.write(pair, Half::Right, Write::Cursor(script));
    let projectile = context.selected_projectile(&owner);
    place(owner, projectile, pair, context)
}
pub fn place<O: Owner>(owner: O, projectile: u16, pair: usize, context: &mut impl Context<O>) -> O {
    let (owner, marker) = context.marker(owner, projectile);
    let (owner, origin) = context.origin(owner, projectile);
    let (mut owner, offsets) = context.offsets(owner, projectile, marker);
    let coordinate = |origin: u16, offset: i8| origin.wrapping_add(offset as u16).wrapping_sub(4);
    owner = context.validate(owner, &offsets, Coordinate::LeftY);
    owner.write(
        pair,
        Half::Left,
        Write::Y(coordinate(origin.y, offsets.left_y())),
    );
    owner = context.validate(owner, &offsets, Coordinate::LeftX);
    owner.write(
        pair,
        Half::Left,
        Write::X(coordinate(origin.x, offsets.left_x())),
    );
    owner = context.validate(owner, &offsets, Coordinate::RightY);
    owner.write(
        pair,
        Half::Right,
        Write::Y(coordinate(origin.y, offsets.right_y())),
    );
    owner = context.validate(owner, &offsets, Coordinate::RightX);
    owner.write(
        pair,
        Half::Right,
        Write::X(coordinate(origin.x, offsets.right_x())),
    );
    owner
}
