//! The domain types the clean models share: quantities, poses and the world, in player terms.

/// A `P.S` quantity: the pixel part in the high 16 bits, the subpixel part in the low 16.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fixed(pub u32);

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
    /// The value read as 8.8: the pixel part's low byte and the subpixel part's high byte.
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
}

/// `below(a, b)` on 16-bit parts (§9): bit 15 of `(a − b) mod 2^16` is set.
#[must_use]
pub const fn below(a: u16, b: u16) -> bool {
    a.wrapping_sub(b) > 0x7FFF
}

/// A signed `P.S` quantity as the player reads it: a size and a sign.
#[derive(Clone, Copy)]
pub struct Signed {
    /// Leftward or upward.
    pub negative: bool,
    /// The size.
    pub size: Fixed,
}

/// Her position: the centre of her body.
#[derive(Clone, Copy)]
pub struct Position {
    /// Rightward.
    pub x: Fixed,
    /// Downward.
    pub y: Fixed,
}

/// A direction: the way a move goes (`TouchedCells`), or the way a speed points.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Heading {
    /// Leftward.
    Left,
    /// Rightward.
    Right,
    /// Upward.
    Up,
    /// Downward.
    Down,
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

/// A body's half-width and half-height, in pixels.
#[derive(Clone, Copy)]
pub struct Radius {
    /// Half-width.
    pub x: u16,
    /// Half-height.
    pub y: u16,
}

/// The pad's buttons (§1.3), held or just pressed.
#[derive(Clone, Copy, Default)]
#[expect(clippy::struct_excessive_bools)]
pub struct Pad {
    /// Run.
    pub run: bool,
    /// Jump.
    pub jump: bool,
    /// Shoot.
    pub shoot: bool,
    /// Left.
    pub left: bool,
    /// Right.
    pub right: bool,
    /// Up.
    pub up: bool,
    /// Down.
    pub down: bool,
    /// Aim up.
    pub aim_up: bool,
    /// Aim down.
    pub aim_down: bool,
}

/// Her equipment (§1.3). Screw attack and the other suit change only the sprite, which follows
/// from the pose and these (§2's last paragraph).
#[derive(Clone, Copy, Default)]
#[expect(clippy::struct_excessive_bools)]
pub struct Equipment {
    /// Hi-jump boots (`Jumping`).
    pub hi_jump: bool,
    /// Space jump (`Jumping`).
    pub space_jump: bool,
    /// Spring ball (`AnimationEnds`).
    pub spring_ball: bool,
    /// Screw attack.
    pub screw_attack: bool,
    /// A suit other than the first: varia or gravity.
    pub other_suit: bool,
}

/// A 16 px cell of the room, by column and row.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    /// The column.
    pub column: u16,
    /// The row.
    pub row: u16,
}

/// `TouchedCells`: a cell a move reached, and the move's direction.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Touch {
    /// The cell, as the cell it stands for where it stands for another.
    pub cell: Cell,
    /// The move's direction.
    pub heading: Heading,
}

/// `OtherBlockKinds`: the kinds of cell that are air to her motion.
#[derive(Clone, Copy)]
pub enum AirKind {
    /// Plain air.
    Plain,
    /// A door that leads to a room: her passage to that room starts.
    DoorToRoom,
    /// Air with spikes in it.
    Spiked,
    /// Air her shots act on.
    ShotAir,
    /// Air her bombs act on.
    BombAir,
    /// Reacting air: a cell that changes the scrolling, a moving floor, an item.
    Reacting,
}

/// `OtherBlockKinds`: the kinds of cell that are solid to her motion.
#[derive(Clone, Copy)]
pub enum SolidKind {
    /// A plain solid block.
    Plain,
    /// A door that leads nowhere (an elevator's).
    DoorToNowhere,
    /// Spikes.
    Spikes,
    /// A block her shots break.
    ShotBlock,
    /// A block her grapple holds.
    GrappleBlock,
}

/// `OtherBlockKinds`: the kinds of cell whose reaction answers, each touch, solid or air.
#[derive(Clone, Copy)]
pub enum AnsweredKind {
    /// A block her bombs break.
    BombBlock,
    /// A reacting block: one that crumbles, one only speed breaks, an item's.
    Reacting,
}

/// The kind of a 16 px cell (§8).
#[derive(Clone, Copy)]
pub enum Block {
    /// Air to her motion.
    Air(AirKind),
    /// Solid to her motion.
    Solid(SolidKind),
    /// Solid or air, as its reaction answers this touch (`World::lets_through`).
    Answered(AnsweredKind),
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
#[derive(Clone, Copy)]
pub struct Hitbox {
    /// Its centre's x.
    pub x: u16,
    /// Its centre's y.
    pub y: u16,
    /// Its half-sizes.
    pub radius: Radius,
}

/// The room as Samus meets it.
pub trait World {
    /// The kind of a 16 px cell.
    fn block(&self, cell: Cell) -> Block;
    /// `OtherBlockKinds`: whether an answered cell lets her through on this touch; the answer
    /// may change from one touch to the next.
    fn lets_through(&self, cell: Cell, heading: Heading) -> bool;
    /// The solid enemies.
    fn solid_enemies(&self) -> &[Hitbox];
}

/// What kind of behaviour a catalogue entry is.
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
pub enum Switch {
    /// Always on.
    Fixed,
    /// On or off.
    Toggle,
    /// Its values come from `Rules`.
    Parameter,
}
