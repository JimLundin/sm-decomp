//! The domain types the clean models share: quantities, poses and the world, in player terms.

/// A `P.S` quantity: the pixel part in the high 16 bits, the subpixel part in the low 16. A
/// signed one is held in two's complement: `-4.E000` is `FFFB.2000`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Default)]
pub struct Fixed(pub u32);

/// The header's notation and §9's arithmetic on `P.S` quantities.
impl Fixed {
    /// From a pixel part and a subpixel part.
    #[must_use]
    pub const fn new(pixel: u16, sub: u16) -> Self {
        let ([a, b], [c, d]) = (pixel.to_be_bytes(), sub.to_be_bytes());
        Self(u32::from_be_bytes([a, b, c, d]))
    }
    /// A whole number of pixels.
    #[must_use]
    pub const fn px(pixel: u16) -> Self {
        Self::new(pixel, 0)
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
    /// The value read as 8.8: the pixel part's low byte and the subpixel part's high byte
    /// (`Jumping`'s space jump window, `SlopesSlowTheWalk`'s cut).
    #[must_use]
    pub const fn as_8_8(self) -> u16 {
        let [_, b, c, _] = self.0.to_be_bytes();
        u16::from_be_bytes([b, c])
    }
    /// The same pixel part with another subpixel part.
    #[must_use]
    pub const fn with_sub(self, sub: u16) -> Self {
        Self::new(self.pixel(), sub)
    }
    /// Sum, the subpixel carrying into the pixel part, mod 2^16 on the pixel part (§9).
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
    /// Read signed: whether this points upward or leftward (below zero).
    #[must_use]
    pub const fn negative(self) -> bool {
        self.0.cast_signed() < 0
    }
    /// The same size the other way, `0 − self`.
    #[must_use]
    pub const fn negated(self) -> Self {
        Self(self.0.wrapping_neg())
    }
    /// Read signed, its size.
    #[must_use]
    pub const fn size(self) -> Self {
        if self.negative() {
            self.negated()
        } else {
            self
        }
    }
}

/// No speed, no distance: `0.0000`.
pub const ZERO: Fixed = Fixed(0);

/// `below(a, b)` on 16-bit parts (§9): bit 15 of `(a − b) mod 2^16` is set.
#[must_use]
pub const fn below(a: u16, b: u16) -> bool {
    a.wrapping_sub(b).cast_signed() < 0
}

/// A value for each axis: x rightward, y downward.
#[derive(Clone, Copy, Debug)]
pub struct Pair<T> {
    /// Rightward.
    pub x: T,
    /// Downward.
    pub y: T,
}

/// Her position: the centre of her body.
pub type Position = Pair<Fixed>;

/// A body's half-width and half-height, in pixels.
pub type Radius = Pair<u16>;

/// A direction: the way a move goes (`TouchedCells`), or the way a speed points. Listed
/// alphabetically, as the spec lists them; the order means nothing.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Heading {
    /// Downward.
    Down,
    /// Leftward.
    Left,
    /// Rightward.
    Right,
    /// Upward.
    Up,
}

/// Which way she faces (§2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Facing {
    /// Facing left.
    Left,
    /// Facing right.
    Right,
    /// Facing the player; counts as `Right` wherever a side is needed.
    Screen,
}

/// §2 facing: forward is the side she faces, back the other side.
impl Facing {
    /// The side she faces, as a heading.
    #[must_use]
    pub const fn side(self) -> Heading {
        if matches!(self, Self::Left) {
            Heading::Left
        } else {
            Heading::Right
        }
    }
    /// The other side: back, as she faces.
    #[must_use]
    pub const fn back(self) -> Self {
        if matches!(self, Self::Left) {
            Self::Right
        } else {
            Self::Left
        }
    }
}

/// §2 body.
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

/// §2 action.
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
    /// The moment she touches down, before she stands still, with what she landed from.
    Landing(LandedFrom),
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
    /// Turning round, on the ground or in the air, with the turn she is in.
    Turning(Turn),
    /// Uncurling from the ball.
    Unmorphing,
    /// A wall jump.
    WallJumping,
}

/// §2 aim: the arm's direction.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Aim {
    /// Unaimed.
    Forward,
    /// Up.
    Up,
    /// Up and forward.
    UpForward,
    /// Down and forward.
    DownForward,
    /// Down.
    Down,
}

/// §2 turn, for `Turning` only: the turn she is in. Its order means nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Turn {
    /// In a fall.
    Fall,
    /// On the ground, upright or crouched.
    Ground,
    /// In a take-off or a straight jump.
    Jump,
    /// The moonwalk's jump, still on the ground.
    MoonwalkJump,
}

/// §2 landed from, for `Landing` only. Its order means nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LandedFrom {
    /// A straight jump or a fall, upright.
    Jump,
    /// A spin or a wall jump.
    Spin,
}

/// §2 gun. Its order means nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gun {
    /// Held out in front, a sprite of its own.
    Drawn,
    /// Not shown.
    Stowed,
}

/// §2 legs. Their order means nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Legs {
    /// Drawn moving forward.
    Striding,
    /// Together.
    Together,
}

/// Her pose as the player sees it (§2), every part it lists.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pose {
    /// Her body: while it changes shape, the body she changes into.
    pub body: Body,
    /// What she is doing, with the turn she is in or what she landed from.
    pub action: Action,
    /// Which way she faces.
    pub facing: Facing,
    /// The arm's direction.
    pub aim: Aim,
    /// Her gun.
    pub gun: Gun,
    /// Her legs.
    pub legs: Legs,
}

/// The pad (§1.3): a set of its buttons, the ones held or the ones just pressed.
#[derive(Clone, Copy, Default)]
pub struct Buttons(pub u16);

/// The pad's buttons, each a set of one; `.and` joins sets.
pub mod button {
    use super::Buttons;
    /// Run.
    pub const RUN: Buttons = Buttons(1);
    /// Jump.
    pub const JUMP: Buttons = Buttons(2);
    /// Shoot.
    pub const SHOOT: Buttons = Buttons(4);
    /// Left.
    pub const LEFT: Buttons = Buttons(8);
    /// Right: the bit after left's.
    pub const RIGHT: Buttons = Buttons(16);
    /// Up.
    pub const UP: Buttons = Buttons(32);
    /// Down.
    pub const DOWN: Buttons = Buttons(64);
    /// Aim up.
    pub const AIM_UP: Buttons = Buttons(128);
    /// Aim down.
    pub const AIM_DOWN: Buttons = Buttons(256);
}

impl Buttons {
    /// Both sets.
    #[must_use]
    pub const fn and(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    /// Whether any of `these` is in the set.
    #[must_use]
    pub const fn any(self, these: Self) -> bool {
        self.0 & these.0 != 0
    }
    /// The set as seen facing `facing` (§7: forward and back are by her facing): facing left,
    /// left and right trade places, so that right stands for forward and left for back.
    #[must_use]
    pub const fn seen(self, facing: Facing) -> Self {
        let (left, right) = (self.0 & button::LEFT.0, self.0 & button::RIGHT.0);
        match facing {
            Facing::Left => Self(self.0 ^ left ^ right | left << 1 | right >> 1),
            Facing::Right | Facing::Screen => self,
        }
    }
}

/// Her equipment (§1.3).
#[derive(Clone, Copy, Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each is an item she has or hasn't, read on its own (§1.3)"
)]
pub struct Equipment {
    /// Hi-jump boots (`Jumping`).
    pub hi_jump: bool,
    /// Space jump (`Jumping`, `HowSheIsDrawn`).
    pub space_jump: bool,
    /// Spring ball (`AnimationEnds`).
    pub spring_ball: bool,
    /// Screw attack, seen only in the sprite (`HowSheIsDrawn`).
    pub screw_attack: bool,
    /// A suit other than the first, varia or gravity, seen only in the sprite (`HowSheIsDrawn`).
    pub other_suit: bool,
}

/// A 16 px cell of the room, by column and row.
#[derive(Clone, Copy)]
pub struct Cell {
    /// The column.
    pub column: u16,
    /// The row.
    pub row: u16,
}

/// `TouchedCells`: the way a cell is reached: a move's direction, or a look (the near-wall test,
/// pose fit's block looks, the shrink's look), which doesn't move her.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Way {
    /// A move, heading this way.
    Move(Heading),
    /// A look.
    Look,
}

/// `TouchedCells`: a cell a move or a look reached, and the way it did.
#[derive(Clone, Copy)]
pub struct Touch {
    /// The cell, as the cell it stands for where it stands for another.
    pub cell: Cell,
    /// The way it was reached.
    pub way: Way,
}

/// The kind of a 16 px cell (§8) as it meets her motion. `OtherBlockKinds`: the world answers
/// each kind of cell as one of these; what the kind then does is its own domain's.
#[derive(Clone, Copy)]
pub enum Block {
    /// Air to her motion: air, a door to a room, air with spikes, air her shots or bombs act on,
    /// reacting air.
    Air,
    /// Solid to her motion: a solid block, a door that leads nowhere, spikes, a block her shots
    /// break or her grapple holds.
    Solid,
    /// Solid or air, as its reaction answers this touch (`World::lets_through`): a block her
    /// bombs break, a reacting block.
    Answered,
    /// A sloped surface, with its shape.
    Slope(Slope),
    /// A stepped block: its 8 px quarters, upper row then lower, each left then right, `true`
    /// where solid.
    Stepped([[bool; 2]; 2]),
    /// A cell that stands for the cell it names; one that names nothing is air.
    StandsFor(Option<Cell>),
}

/// `BlocksStopFlush`: the game's frame count's parity, alternating every frame.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Parity {
    /// An even frame.
    Even,
    /// An odd frame.
    Odd,
}

/// Which way a sloped surface's solid part lies.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SlopeFaces {
    /// A sloped ceiling: solid above its surface.
    Ceiling,
    /// A sloped floor: solid below its surface.
    Floor,
}

/// `SlopesCarry`: where a sloped surface lies at one of its block's pixel columns.
#[derive(Clone, Copy)]
pub enum Surface {
    /// At this row of the block, 0 to 15: a floor's topmost solid pixel, a ceiling's lowest.
    Row(u8),
    /// An empty column: none of the surface, which lies just past the block's far edge.
    JustPast,
    /// A column far past: none of the surface, which lies more than one row past the far edge.
    FarPast,
}

/// A sloped surface's shape: world data.
#[derive(Clone, Copy)]
pub struct Slope {
    /// Floor or ceiling.
    pub faces: SlopeFaces,
    /// The surface at each of the block's pixel columns, from the left.
    pub surface: [Surface; 16],
    /// `SlopesSlowTheWalk`: a floor's walk factor, in 1/256ths (`0x100` is 1.00).
    pub factor: u16,
}

/// A solid enemy's box: its centre and half-sizes, in pixels.
pub struct Hitbox {
    /// Its centre.
    pub centre: Pair<u16>,
    /// Its half-sizes.
    pub radius: Radius,
}

/// The room as Samus meets it.
pub trait World {
    /// The kind of a 16 px cell.
    fn block(&self, cell: Cell) -> Block;
    /// `OtherBlockKinds`: whether an answered cell lets her through on this touch, a move's or a
    /// look's; the answer may change from one touch to the next.
    fn lets_through(&self, cell: Cell, way: Way) -> bool;
    /// The solid enemies.
    fn solid_enemies(&self) -> &[Hitbox];
}

/// What kind of behaviour a catalogue entry is.
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
pub enum Switch {
    /// Always on.
    Fixed,
    /// On or off.
    Toggle,
    /// Its values come from `Rules`.
    Parameter,
}
