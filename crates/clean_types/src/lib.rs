//! The domain types the clean models share: quantities, poses and tables in player terms.

/// A `P.S` quantity: a 16-bit pixel part and a 16-bit subpixel part.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash)]
pub struct Fixed(u32);

impl Fixed {
    /// Nothing.
    pub const ZERO: Self = Self(0);

    /// From the whole value, pixel part in the high 16 bits: `0x0002_C000` is `2.C000`.
    #[must_use]
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// From a pixel part and a subpixel part.
    #[must_use]
    pub const fn from_parts(pixel: u16, sub: u16) -> Self {
        let [a, b] = pixel.to_be_bytes();
        let [c, d] = sub.to_be_bytes();
        Self(u32::from_be_bytes([a, b, c, d]))
    }

    /// A whole number of pixels.
    #[must_use]
    pub const fn px(pixel: u16) -> Self {
        Self::from_parts(pixel, 0)
    }

    /// The pixel part.
    #[must_use]
    pub const fn pixel(self) -> u16 {
        let [a, b, _, _] = self.0.to_be_bytes();
        u16::from_be_bytes([a, b])
    }

    /// The subpixel part.
    #[must_use]
    pub const fn sub(self) -> u16 {
        let [_, _, c, d] = self.0.to_be_bytes();
        u16::from_be_bytes([c, d])
    }

    /// The same pixel part with another subpixel part.
    #[must_use]
    pub const fn with_sub(self, sub: u16) -> Self {
        Self::from_parts(self.pixel(), sub)
    }

    /// The same subpixel part with another pixel part.
    #[must_use]
    pub const fn with_pixel(self, pixel: u16) -> Self {
        Self::from_parts(pixel, self.sub())
    }

    /// The value read as 8.8: the pixel part's low byte and the subpixel part's high byte.
    #[must_use]
    pub const fn as_8_8(self) -> u16 {
        let [_, b, c, _] = self.0.to_be_bytes();
        u16::from_be_bytes([b, c])
    }

    /// Sum, the subpixel carrying into the pixel part, mod 2^16 on the pixel part.
    #[must_use]
    pub const fn plus(self, other: Self) -> Self {
        Self(self.0.wrapping_add(other.0))
    }

    /// Difference, borrowing the same way.
    #[must_use]
    pub const fn minus(self, other: Self) -> Self {
        Self(self.0.wrapping_sub(other.0))
    }

    /// Whether this is zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

/// `below(a, b)` on 16-bit parts: bit 15 of `(a − b) mod 2^16` is set.
#[must_use]
pub const fn below(a: u16, b: u16) -> bool {
    a.wrapping_sub(b) & 0x8000 != 0
}

/// A signed `P.S` quantity as the player reads it: a size and a sign.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Signed {
    /// Leftward or upward.
    pub negative: bool,
    /// The size.
    pub size: Fixed,
}

/// Her position: the centre of her body.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Position {
    /// Rightward.
    pub x: Fixed,
    /// Downward.
    pub y: Fixed,
}

/// A side, for a way of moving or a wall.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    /// Leftward.
    Left,
    /// Rightward.
    Right,
}

/// Which way she faces.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Facing {
    /// Facing left.
    Left,
    /// Facing right.
    Right,
    /// Facing the player; counts as `Right` wherever a side is needed.
    Screen,
}

impl Facing {
    /// The side she faces.
    #[must_use]
    pub const fn side(self) -> Side {
        match self {
            Self::Left => Side::Left,
            Self::Right | Self::Screen => Side::Right,
        }
    }
}

/// Her body's shape.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Body {
    /// Standing height.
    Upright,
    /// Crouched.
    Crouched,
    /// The morph ball.
    Ball,
    /// The spring ball.
    SpringBall,
}

impl Body {
    /// Whether this is a ball.
    #[must_use]
    pub const fn is_ball(self) -> bool {
        matches!(self, Self::Ball | Self::SpringBall)
    }
}

/// What she is doing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    /// Pressed to the wall she ran into.
    AgainstWall,
    /// Crouching down.
    Crouching,
    /// Falling.
    Falling,
    /// A straight jump, or the spring ball's hop.
    Jumping,
    /// Moonwalking.
    Moonwalking,
    /// Curling into the ball.
    Morphing,
    /// Rolling as a ball.
    Rolling,
    /// Running.
    Running,
    /// A spin jump.
    Spinning,
    /// Standing up from a crouch.
    StandingUp,
    /// At rest.
    Still,
    /// The first poses of a straight jump.
    TakingOff,
    /// Turning round, on the ground or in the air.
    Turning,
    /// Uncurling from the ball.
    Unmorphing,
    /// A wall jump.
    WallJumping,
}

impl Action {
    /// Whether her body is changing shape.
    #[must_use]
    pub const fn changes_shape(self) -> bool {
        matches!(
            self,
            Self::Crouching | Self::StandingUp | Self::Morphing | Self::Unmorphing
        )
    }
}

/// The arm's direction.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Aim {
    /// Aimed down.
    Down,
    /// Aimed down and forward.
    DownForward,
    /// Unaimed.
    #[default]
    Forward,
    /// Aimed up.
    Up,
    /// Aimed up and forward.
    UpForward,
}

/// A body's half-width and half-height, in pixels.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Radius {
    /// Half-width.
    pub x: u16,
    /// Half-height.
    pub y: u16,
}

/// Her pose as the player sees it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pose {
    /// Her body's shape.
    pub body: Body,
    /// What she is doing.
    pub action: Action,
    /// Which way she faces.
    pub facing: Facing,
    /// The arm's direction.
    pub aim: Aim,
    /// Her body's size.
    pub radius: Radius,
}

/// A button on the pad.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Button {
    /// Run.
    Run,
    /// Jump.
    Jump,
    /// Shoot.
    Shoot,
    /// Left.
    Left,
    /// Right.
    Right,
    /// Up.
    Up,
    /// Down.
    Down,
    /// Aim up.
    AimUp,
    /// Aim down.
    AimDown,
}

impl Button {
    const fn bit(self) -> u16 {
        match self {
            Self::Run => 1,
            Self::Jump => 1 << 1,
            Self::Shoot => 1 << 2,
            Self::Left => 1 << 3,
            Self::Right => 1 << 4,
            Self::Up => 1 << 5,
            Self::Down => 1 << 6,
            Self::AimUp => 1 << 7,
            Self::AimDown => 1 << 8,
        }
    }
}

/// The buttons held.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Pad(u16);

impl Pad {
    /// The pad with this button held as well.
    #[must_use]
    pub const fn with(self, button: Button) -> Self {
        Self(self.0 | button.bit())
    }

    /// Whether a button is held.
    #[must_use]
    pub const fn held(self, button: Button) -> bool {
        self.0 & button.bit() != 0
    }

    /// Whether the side's button is held.
    #[must_use]
    pub const fn holds(self, side: Side) -> bool {
        match side {
            Side::Left => self.held(Button::Left),
            Side::Right => self.held(Button::Right),
        }
    }

    /// Whether she steers neither way: neither left nor right held.
    #[must_use]
    pub const fn steers_neither_way(self) -> bool {
        !self.holds(Side::Left) && !self.holds(Side::Right)
    }

    /// Whether no button at all is held.
    #[must_use]
    pub const fn none_held(self) -> bool {
        self.0 == 0
    }
}

/// The equipment that changes her motion.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Equipment {
    /// Hi-jump boots.
    pub hi_jump: bool,
    /// Space jump.
    pub space_jump: bool,
}

/// The kind of a 16 px block.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Block {
    /// Nothing.
    Air,
    /// A sloped surface, with its shape.
    Slope(Slope),
    /// Solid.
    Solid,
    /// A stepped block: solid in some of its four 8 px quarters.
    Stepped(Quarters),
}

/// The axis of a move.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Axis {
    /// Horizontal.
    X,
    /// Vertical.
    Y,
}

/// One half of a block along an axis: its first 8 px (left or top) or its second.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Half {
    /// The left or top half.
    First,
    /// The right or bottom half.
    Second,
}

/// Which halves of a block, along one axis, her body overlaps.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Halves {
    /// The left or top half.
    pub first: bool,
    /// The right or bottom half.
    pub second: bool,
}

/// A stepped block's solid quarters, by row half then column half.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Quarters(pub [[bool; 2]; 2]);

impl Quarters {
    /// Whether the quarter in this row half and column half is solid.
    #[must_use]
    pub const fn solid(self, row: Half, column: Half) -> bool {
        let [top, bottom] = self.0;
        let [left, right] = match row {
            Half::First => top,
            Half::Second => bottom,
        };
        match column {
            Half::First => left,
            Half::Second => right,
        }
    }

    /// `BlocksStopFlush`: whether a move along `axis` whose leading edge reaches the half
    /// `reached` stops there: a quarter of that half is solid among those her body overlaps
    /// across it (`across`).
    #[must_use]
    pub const fn stops(self, axis: Axis, reached: Half, across: Halves) -> bool {
        (across.first && self.along(axis, reached, Half::First))
            || (across.second && self.along(axis, reached, Half::Second))
    }

    /// Whether the quarter at half `along` of a move along `axis` and half `across` is solid.
    const fn along(self, axis: Axis, along: Half, across: Half) -> bool {
        match axis {
            Axis::X => self.solid(across, along),
            Axis::Y => self.solid(along, across),
        }
    }
}

/// `BlocksStopFlush`: the game's frame count's parity, alternating every frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Parity {
    /// An even frame.
    #[default]
    Even,
    /// An odd frame.
    Odd,
}

impl Parity {
    /// The other parity: the frame before or after.
    #[must_use]
    pub const fn other(self) -> Self {
        match self {
            Self::Even => Self::Odd,
            Self::Odd => Self::Even,
        }
    }
}

/// Which way a sloped surface's solid part lies.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlopeFaces {
    /// A sloped ceiling: solid above its surface.
    Ceiling,
    /// A sloped floor: solid below its surface.
    Floor,
}

/// A sloped surface's shape: world data.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Slope {
    /// Floor or ceiling.
    pub faces: SlopeFaces,
    /// At each of the block's pixel columns, the row within the block of the surface's solid
    /// pixel: a floor's topmost, a ceiling's lowest.
    pub surface: [u8; 16],
}

/// A solid enemy's box.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hitbox {
    /// Its centre, in pixels.
    pub x: u16,
    /// Its centre, in pixels.
    pub y: u16,
    /// Its half-sizes.
    pub radius: Radius,
}

/// The room as Samus meets it.
pub trait World {
    /// The block at a column and row of 16 px cells.
    fn block(&self, column: u16, row: u16) -> Block;
    /// The solid enemies.
    fn solid_enemies(&self) -> &[Hitbox];
}

/// What kind of behaviour a catalogue entry is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// A deliberate game rule.
    Mechanic,
    /// Unintended behaviour the model reproduces.
    Quirk,
    /// Slowdown and loading time.
    Timing,
    /// An output quirk.
    Encoding,
}

/// How a catalogue entry can be adjusted.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Switch {
    /// Always on.
    Fixed,
    /// On or off.
    Toggle,
    /// Its values come from `Rules`.
    Parameter,
}
