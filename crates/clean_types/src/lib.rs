//! The domain types the clean models share: quantities, poses and the world, in player terms.

use core::num::Wrapping;

/// A `P.S` quantity: the pixel part in the high 16 bits, the subpixel part in the low 16.
///
/// A signed one is held in two's complement: `-4.E000` is `FFFB.2000`. §9: a sum carries the
/// subpixel into the pixel part, mod 2^16 on the pixel part, so `+`, `-` and negation are
/// `Wrapping`'s.
///
/// Its comparisons (`<`, `<=`, `min`, `max`) are unsigned: they read the 32 bits as they stand,
/// so a signed quantity below zero compares as a large one. Compare a signed quantity by its
/// `Parts::size` and `Parts::negative`.
pub type Fixed = Wrapping<u32>;

/// A pixel part, or a whole number of pixels.
///
/// A pixel of a position along one axis, a distance, a body's half-size. Its arithmetic wraps
/// mod 2^16, as the pixel part of a position does (§9). Its comparisons are unsigned, as
/// `Fixed`'s are; read a signed difference by its `.0`'s `cast_signed`.
pub type Px = Wrapping<u16>;

/// The header's notation: a `P.S` quantity from its pixel part and its subpixel part.
#[must_use]
pub fn fixed(pixel: Px, sub: u16) -> Fixed {
    Wrapping(u32::from(pixel.0) << PIXEL_BITS | u32::from(sub))
}

/// A whole number of pixels, as a `P.S` quantity.
#[must_use]
pub fn px(pixel: Px) -> Fixed {
    fixed(pixel, 0)
}

/// §9: the pixel part is a quantity's high 16 bits.
const PIXEL_BITS: u32 = 16;
/// `Jumping`, `SlopesSlowTheWalk`: an 8.8 reading takes a byte from each part.
const BYTE: u32 = 8;

/// The parts of a `P.S` quantity (the header's notation), and its sign read signed (§1).
pub trait Parts {
    /// The pixel part and the subpixel part.
    fn words(self) -> (u16, u16);
    /// The pixel part.
    fn pixel(self) -> Px;
    /// The subpixel part.
    fn sub(self) -> u16;
    /// The value read as 8.8: the pixel part's low byte and the subpixel part's high byte
    /// (`Jumping`'s space jump window, `SlopesSlowTheWalk`'s cut).
    fn to_8_8(self) -> Px;
    /// The same pixel part with another subpixel part.
    fn with_sub(self, sub: u16) -> Fixed;
    /// Read signed: whether this points upward or leftward (below zero).
    fn negative(self) -> bool;
    /// Read signed, its size.
    fn size(self) -> Fixed;
}

impl Parts for Fixed {
    fn words(self) -> (u16, u16) {
        let [a, b, c, d] = self.0.to_be_bytes();
        (u16::from_be_bytes([a, b]), u16::from_be_bytes([c, d]))
    }
    fn pixel(self) -> Px {
        Wrapping(self.words().0)
    }
    fn sub(self) -> u16 {
        self.words().1
    }
    fn to_8_8(self) -> Px {
        let (pixel, sub) = self.words();
        Wrapping(pixel << BYTE | sub >> BYTE)
    }
    fn with_sub(self, sub: u16) -> Fixed {
        fixed(self.pixel(), sub)
    }
    fn negative(self) -> bool {
        self.0.cast_signed() < 0
    }
    fn size(self) -> Fixed {
        if self.negative() { -self } else { self }
    }
}

/// No speed, no distance: `0.0000`.
pub const ZERO: Fixed = Wrapping(0);

/// `WordwiseTopSpeed`, `Dash`: `below(a, b)` on 16-bit parts: bit 15 of `(a − b) mod 2^16` is
/// set.
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
pub type Radius = Pair<Px>;

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

/// The pad (§1.3): a set of its buttons, the ones held or the ones just pressed, each a bit of
/// `button`'s.
#[derive(Clone, Copy, Default)]
pub struct Buttons(pub u16);

/// The pad's buttons, each a set of one; sets join by their bits, `Buttons(a.0 | b.0)`.
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
    /// Right.
    pub const RIGHT: Buttons = Buttons(16);
    /// Up.
    pub const UP: Buttons = Buttons(32);
    /// Down.
    pub const DOWN: Buttons = Buttons(64);
    /// Aim up.
    pub const AIM_UP: Buttons = Buttons(128);
    /// Aim down.
    pub const AIM_DOWN: Buttons = Buttons(256);
    /// Left or right.
    pub const SIDES: Buttons = Buttons(LEFT.0 | RIGHT.0);
    /// Every button of the pad.
    pub const PAD: Buttons = Buttons(511);
}

impl Buttons {
    /// Whether any of `these` is in the set.
    #[must_use]
    pub const fn any(self, these: Self) -> bool {
        self.0 & these.0 != 0
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

/// A sloped surface's shape: world data.
#[derive(Clone, Copy)]
pub struct Slope {
    /// Floor or ceiling.
    pub faces: SlopeFaces,
    /// `SlopesCarry`: the surface at each of the block's pixel columns, from the left, held as
    /// its depth from the block's open side: a floor's top row as it is (counted from the top),
    /// a ceiling's bottom row counted up from the block's bottom row (15 less it); 16 is an empty
    /// column (the surface just past the block's far edge) and 17 a column far past, for a floor
    /// and a ceiling alike.
    pub depth: [Px; 16],
    /// `SlopesSlowTheWalk`: a floor's walk factor, in 1/256ths (`0x100` is 1.00).
    pub factor: Fixed,
}

/// A solid enemy's box: its centre and half-sizes, in pixels.
pub struct Hitbox {
    /// Its centre.
    pub centre: Pair<Px>,
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
    /// What a frame reports beyond her quantities, for the caller to act on.
    Output,
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

/// How the spec checks a catalogue entry: its `checked:` line.
#[derive(PartialEq, Eq)]
pub enum Checked {
    /// By vectors: the oracle's records, constructed vectors, or both.
    ByVectors,
    /// On the original alone: no vector exercises it.
    OriginalOnly,
}
