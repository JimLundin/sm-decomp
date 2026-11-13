//! The domain types the clean models share: quantities, poses and the world, in player terms.

use core::num::Wrapping;

use enum_map::{Enum, EnumMap};

/// A `P.S` quantity: the pixel part in the high 16 bits, the subpixel part in the low 16.
///
/// A signed one is held in two's complement: `-4.E000` is `FFFB.2000`. A sum carries the
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
/// mod 2^16, as the pixel part of a position does. Its comparisons are unsigned, as
/// `Fixed`'s are; read a signed difference by its `.0`'s `cast_signed`.
pub type Px = Wrapping<u16>;

/// The specs' `P.S` notation: a quantity from its pixel part and its subpixel part.
#[must_use]
pub fn fixed(pixel: Px, sub: u16) -> Fixed {
    Wrapping(u32::from(pixel.0) << PIXEL_BITS | u32::from(sub))
}

/// A whole number of pixels, as a `P.S` quantity.
#[must_use]
pub fn px(pixel: Px) -> Fixed {
    fixed(pixel, 0)
}

/// The pixel part is a quantity's high 16 bits.
const PIXEL_BITS: u32 = 16;
/// `Jumping`, `SlopesSlowTheWalk`: an 8.8 reading takes a byte from each part.
const BYTE: u32 = 8;

/// The parts of a `P.S` quantity, and its sign read signed.
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
    /// `WordwiseTopSpeed`, `Dash`: the two 16-bit parts, which their tests read apart.
    fn words(self) -> (u16, u16) {
        let [a, b, c, d] = self.0.to_be_bytes();
        (u16::from_be_bytes([a, b]), u16::from_be_bytes([c, d]))
    }
    /// The pixel part, which `BlocksStopFlush` reads of a position.
    fn pixel(self) -> Px {
        Wrapping(self.words().0)
    }
    /// The subpixel part, which `BlocksStopFlush` writes FFFF or 0.
    fn sub(self) -> u16 {
        self.words().1
    }
    /// `Jumping`, `SlopesSlowTheWalk`: the 8.8 reading.
    fn to_8_8(self) -> Px {
        let (pixel, sub) = self.words();
        Wrapping(pixel << BYTE | sub >> BYTE)
    }
    /// `BlocksStopFlush`: a stop's subpixel written into a position.
    fn with_sub(self, sub: u16) -> Fixed {
        fixed(self.pixel(), sub)
    }
    /// A signed speed or move pointing upward or leftward.
    fn negative(self) -> bool {
        self.0.cast_signed() < 0
    }
    /// `WordwiseTopSpeed`: the test is on the speed's size.
    fn size(self) -> Fixed {
        if self.negative() { -self } else { self }
    }
}

/// No speed, no distance: `0.0000`.
pub const ZERO: Fixed = Wrapping(0);

/// One whole pixel: `1.0000`.
pub const PIXEL: Fixed = Wrapping(1 << PIXEL_BITS);

/// `WordwiseTopSpeed`, `Dash`: the spec's `below(a, b)` on two 16-bit parts of `P.S` quantities.
///
/// It compares two pixel parts, or two subpixel parts: bit 15 of `(a − b) mod 2^16` is set. It
/// is not an unsigned `a < b`: 1000 is not below C000, and C000 is below 4000 and below 0.
#[must_use]
pub const fn part_below(a: u16, b: u16) -> bool {
    a.wrapping_sub(b).cast_signed() < 0
}

/// `part_below` by the spec's own name, kept for the domains that call it so.
#[must_use]
pub const fn below(a: u16, b: u16) -> bool {
    part_below(a, b)
}

/// A colour of the screen: 15 bits, three channels of 5 bits each, red in bits 0 to 4, green in
/// 5 to 9 and blue in 10 to 14.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Colour(u16);

/// The highest colour: every channel at 31, white.
const LAST_COLOUR: u16 = 0x7FFF;
/// A channel's 5 bits.
const CHANNEL: u16 = 0x1F;
/// Where green starts in a colour's bits.
const GREEN_AT: u32 = 5;
/// Where blue starts in a colour's bits.
const BLUE_AT: u32 = 10;

impl Colour {
    /// Black: every channel 0.
    pub const BLACK: Self = Self(0);

    /// The colour of these 15 bits, if they are 15 bits (`0000` to `7FFF`).
    #[must_use]
    pub const fn new(bits: u16) -> Option<Self> {
        if bits <= LAST_COLOUR {
            Some(Self(bits))
        } else {
            None
        }
    }

    /// Its 15 bits.
    #[must_use]
    pub const fn bits(self) -> u16 {
        self.0
    }

    /// Its channels, red, green and blue, each 0 to 31.
    #[must_use]
    pub const fn channels(self) -> [u16; 3] {
        [
            self.0 & CHANNEL,
            (self.0 >> GREEN_AT) & CHANNEL,
            (self.0 >> BLUE_AT) & CHANNEL,
        ]
    }

    /// The colour of these channels, red, green and blue; each keeps its low 5 bits.
    #[must_use]
    pub const fn from_channels([red, green, blue]: [u16; 3]) -> Self {
        Self((red & CHANNEL) | ((green & CHANNEL) << GREEN_AT) | ((blue & CHANNEL) << BLUE_AT))
    }
}

/// A value for each axis: x rightward, y downward.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pair<T> {
    /// Rightward.
    pub x: T,
    /// Downward.
    pub y: T,
}

impl<T: Copy> Pair<T> {
    /// `BlocksStopFlush`: a move tests one axis at a time: the part along a move heading `h`,
    /// and the part across it.
    #[must_use]
    pub const fn along(self, h: Heading) -> (T, T) {
        if h.vertical() {
            (self.y, self.x)
        } else {
            (self.x, self.y)
        }
    }

    /// `BlocksStopFlush`: the pair whose part along a move heading `h` is `along`, and whose part
    /// across it is `across`: `along`'s inverse.
    #[must_use]
    pub const fn from_along(h: Heading, along: T, across: T) -> Self {
        let (x, y) = if h.vertical() {
            (across, along)
        } else {
            (along, across)
        };
        Self { x, y }
    }
}

/// Her position: the centre of her body.
pub type Position = Pair<Fixed>;

/// A body's half-width and half-height, in pixels.
pub type Radius = Pair<Px>;

/// A direction: the way a move goes (`TouchedCells`), or the way a speed points. Listed
/// alphabetically, as the spec lists them; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl Heading {
    /// `BlocksStopFlush`: whether a move heading this way is vertical.
    #[must_use]
    pub const fn vertical(self) -> bool {
        matches!(self, Self::Up | Self::Down)
    }
}

/// One of her two sides along x: the way she faces, or the way her horizontal speed points. Listed
/// alphabetically; the order means nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    /// Leftward.
    Left,
    /// Rightward.
    Right,
}

impl Side {
    /// The side she faces; facing the screen counts as `Right`.
    #[must_use]
    pub const fn of(facing: Facing) -> Self {
        if matches!(facing, Facing::Left) {
            Self::Left
        } else {
            Self::Right
        }
    }

    /// `BlocksStopFlush`: the way a horizontal move to this side goes.
    #[must_use]
    pub const fn heading(self) -> Heading {
        match self {
            Self::Left => Heading::Left,
            Self::Right => Heading::Right,
        }
    }
}

/// Which way she faces: a part of her pose.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Facing {
    /// Facing left.
    Left,
    /// Facing right.
    Right,
    /// Facing the player; counts as `Right` wherever a side is needed.
    Screen,
}

/// Forward is the side she faces (`Side::of`), back the other side.
impl Facing {
    /// `PadChart`, `TurningRound`: the other side, back, as she faces.
    #[must_use]
    pub const fn back(self) -> Self {
        if matches!(self, Self::Left) {
            Self::Right
        } else {
            Self::Left
        }
    }
}

/// Her body: a part of her pose.
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

/// What she is doing: a part of her pose.
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

/// Her aim, the arm's direction: a part of her pose.
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

/// The turn she is in, for `Turning` only. Its order means nothing.
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

/// What she landed from, for `Landing` only. Its order means nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LandedFrom {
    /// A straight jump or a fall, upright.
    Jump,
    /// A spin or a wall jump.
    Spin,
}

/// Her gun: a part of her pose. Its order means nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gun {
    /// Held out in front, a sprite of its own.
    Drawn,
    /// Not shown.
    Stowed,
}

/// Her legs: a part of her pose. Their order means nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Legs {
    /// Drawn moving forward.
    Striding,
    /// Together.
    Together,
}

/// Her pose as the player sees it, every part samus-movement lists.
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

/// The sprite her pose is drawn in, as samus-movement's `HowSheIsDrawn` chooses it: in
/// samus-appearance each pose has a clip for each sprite. Its order means nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sprite {
    /// Her plain sprite.
    Plain,
    /// The screw attack's.
    ScrewAttack,
    /// The space jump's.
    SpaceJump,
    /// The other suit's.
    OtherSuit,
}

/// The end her clip reaches this frame (samus-appearance `ClipEnds`), which samus-movement's
/// `AnimationEnds` reads; no end is `None` where it is held as an `Option`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClipEnd {
    /// The clip ends: samus-movement hands over her next pose.
    End,
    /// The clip ends unless the pad asks for a jump, which then takes its place.
    EndUnlessJump,
}

/// Her run's speed booster stage (samus-appearance `BoostStages`): 0 to 4, where 4 is boosting. The
/// shinespark, the echoes, her glow, the blocks a boost breaks and the enemies she touches read it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Enum)]
pub enum BoostStage {
    /// Stage 0: where every dash begins.
    #[default]
    Stage0,
    /// Stage 1.
    Stage1,
    /// Stage 2.
    Stage2,
    /// Stage 3.
    Stage3,
    /// Stage 4: boosting.
    Boosting,
}

impl BoostStage {
    /// Every stage, numbered 0 to 4 by its place here.
    pub const ALL: [Self; 5] = [
        Self::Stage0,
        Self::Stage1,
        Self::Stage2,
        Self::Stage3,
        Self::Boosting,
    ];
}

/// A sound her animation asks for (samus-appearance `WallJumpBranch`, `BoostStages`), for the sound
/// domain to play or not.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AnimationSound {
    /// A wall jump's plain spin.
    PlainSpin,
    /// A wall jump spun as the space jump.
    SpaceJump,
    /// A wall jump spun as the screw attack.
    ScrewAttack,
    /// She starts boosting.
    Boosting,
}

/// The pad: a set of its buttons, the ones held or the ones just pressed, each a bit of
/// `button`'s.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
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
    /// Every button of the pad, each of the above.
    pub const PAD: Buttons = Buttons(
        RUN.0 | JUMP.0 | SHOOT.0 | LEFT.0 | RIGHT.0 | UP.0 | DOWN.0 | AIM_UP.0 | AIM_DOWN.0,
    );
}

impl Buttons {
    /// `PadChart`: whether any of `these` is in the set, held or just pressed.
    #[must_use]
    pub const fn any(self, these: Self) -> bool {
        self.0 & these.0 != 0
    }
}

/// A 16 px cell of the room: x its column, y its row.
pub type Cell = Pair<u16>;

/// `TouchedCells`: the way a cell is reached: a move's direction, or a look (the near-wall test,
/// pose fit's block looks, the shrink's look), which doesn't move her.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    /// A move, heading this way.
    Move(Heading),
    /// A look.
    Look,
}

/// `TouchedCells`: a cell a move or a look reached, and the way it did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Touch {
    /// The cell, as the cell it stands for where it stands for another.
    pub cell: Cell,
    /// The way it was reached.
    pub way: Way,
}

/// The kind of a 16 px cell as it meets her motion. `OtherBlockKinds`: the world answers
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
    /// A cell that stands for the cell it names; one that names nothing is air. A chain of them
    /// ends at a cell of another kind or one naming nothing: a world whose chain loops is the
    /// caller's to refuse.
    StandsFor(Option<Cell>),
}

/// `BlocksStopFlush`: the game's frame count's parity, alternating every frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    /// The kind of a 16 px cell. Cells that stand for others never name each other in a loop.
    fn block(&self, cell: Cell) -> Block;
    /// `OtherBlockKinds`: whether an answered cell lets her through on this touch, a move's or a
    /// look's; the answer may change from one touch to the next.
    fn lets_through(&self, cell: Cell, way: Way) -> bool;
    /// The solid enemies.
    fn solid_enemies(&self) -> &[Hitbox];
}

/// What kind of behaviour a catalogue entry is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A deliberate game rule.
    Mechanic,
    /// Unintended behaviour the game has and the model reproduces.
    Quirk,
    /// When a rule acts: the order of the steps within a frame and which state each step reads,
    /// slowdown and loading time.
    Timing,
    /// Something the step reports for the caller to show or act on (which sprite, which cells she
    /// reached), not a rule of her motion.
    Output,
    /// A quirk of an output: what is reported departs from what the game means to show.
    Encoding,
}

/// How a catalogue entry can be adjusted: its spec entry's `switch:` line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Switch {
    /// `none`: always as the spec gives it, its numbers included.
    Fixed,
    /// A named rule of its own may stand in for it.
    Toggle,
    /// The numbers its `params` give come from `Rules`, and may be set to other values.
    Parameter,
}

/// How the spec checks a catalogue entry: its `checked:` line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Checked {
    /// By vectors: the oracle's records, constructed vectors, or both.
    ByVectors,
    /// On the original alone: no vector exercises it.
    OriginalOnly,
}

/// A set of members of the enum `K`: a table of whether each member is in it, read and changed
/// as `set[member]`. Its default is the empty set.
pub type Set<K> = EnumMap<K, bool>;

/// The set of the members listed, none if the list is empty: `set_of([Beam::Spazer])`. A table
/// whose cells are sets writes each cell this way.
#[must_use]
pub fn set_of<K: Enum, const N: usize>(members: [K; N]) -> Set<K> {
    members.into_iter().map(|member| (member, true)).collect()
}

/// What a set tells of its members, read from the set itself.
pub trait Members {
    /// Whether any member is in both sets.
    fn intersects(&self, other: &Self) -> bool;
    /// How many members are in it, a count of 16 bits as the specs' counts are.
    fn count(&self) -> Wrapping<u16>;
}

impl<K: Enum> Members for Set<K> {
    fn intersects(&self, other: &Self) -> bool {
        self.values().zip(other.values()).any(|(&a, &b)| a && b)
    }

    fn count(&self) -> Wrapping<u16> {
        self.values()
            .filter(|&&is_in| is_in)
            .map(|_| Wrapping(1))
            .sum()
    }
}

/// A table keyed by the enum `K`, built from its rows, one row for each member, each naming its
/// member (read by `member`), in any order: a reordered enum changes nothing.
#[must_use]
pub fn by_member<K, R, const N: usize>(mut rows: [R; N], member: fn(&R) -> K) -> EnumMap<K, R>
where
    K: Enum<Array<R> = [R; N]>,
{
    rows.sort_unstable_by_key(|row| member(row).into_usize());
    EnumMap::from_array(rows)
}

/// A region of the planet. The area of the room she is in is the frame's area.
///
/// Listed alphabetically; the order means nothing beyond keeping areas in sorted sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum Area {
    /// Brinstar.
    Brinstar,
    /// The Ceres space station.
    Ceres,
    /// Crateria, where the gunship lands.
    Crateria,
    /// Maridia.
    Maridia,
    /// Norfair.
    Norfair,
    /// Tourian.
    Tourian,
    /// The wrecked ship.
    WreckedShip,
}

impl Area {
    /// Every area, in the order listed.
    pub const ALL: &[Self] = &[
        Self::Brinstar,
        Self::Ceres,
        Self::Crateria,
        Self::Maridia,
        Self::Norfair,
        Self::Tourian,
        Self::WreckedShip,
    ];
}

/// A piece of equipment she can take that is not a beam or a tank.
///
/// Listed alphabetically; the order means nothing beyond keeping items in sorted sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum Item {
    /// Bombs, laid from the morph ball.
    Bombs,
    /// The grapple beam.
    GrappleBeam,
    /// The gravity suit.
    GravitySuit,
    /// The hi-jump boots: she jumps higher.
    HiJumpBoots,
    /// The morph ball: she rolls.
    MorphBall,
    /// The screw attack: she spins through enemies.
    ScrewAttack,
    /// The space jump.
    SpaceJump,
    /// The speed booster.
    SpeedBooster,
    /// The spring ball.
    SpringBall,
    /// The varia suit.
    VariaSuit,
    /// The x-ray scope.
    XRayScope,
}

/// A beam she can take.
///
/// Listed alphabetically; the order means nothing beyond keeping beams in sorted sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum Beam {
    /// The charge beam.
    Charge,
    /// The ice beam.
    Ice,
    /// The plasma beam.
    Plasma,
    /// The spazer.
    Spazer,
    /// The wave beam.
    Wave,
}

/// Something that, once it happens, changes the world for the rest of the run.
///
/// Listed alphabetically; the order means nothing beyond keeping events in sorted sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum Event {
    /// The chozo statue deep in Norfair has let the acid out of its room.
    AcidReceded,
    /// The animals she saved have escaped the planet.
    AnimalsFreed,
    /// Draygon's statue at Tourian's door has gone grey.
    DraygonStatueGrey,
    /// Maridia's glass tunnel is broken.
    GlassTunnelBroken,
    /// Kraid's statue at Tourian's door has gone grey.
    KraidStatueGrey,
    /// She outran the rising lava in Norfair's speed booster hall.
    LavaQuakeOutrun,
    /// The first of Tourian's four metroid rooms she meets is cleared.
    MetroidRoom1Empty,
    /// The second of Tourian's metroid rooms is cleared.
    MetroidRoom2Empty,
    /// The third of Tourian's metroid rooms is cleared.
    MetroidRoom3Empty,
    /// The fourth of Tourian's metroid rooms is cleared.
    MetroidRoom4Empty,
    /// The glass of Mother Brain's tank is broken.
    MotherBrainTankBroken,
    /// Phantoon's statue at Tourian's door has gone grey.
    PhantoonStatueGrey,
    /// The planet is awake: Crateria's creatures and its rain are out.
    PlanetAwake,
    /// Ridley's statue at Tourian's door has gone grey.
    RidleyStatueGrey,
    /// Shaktool has dug its path through Maridia's sand.
    ShaktoolPath,
    /// The planet's self-destruct is running: the escape.
    TimeBombSet,
    /// The way into Tourian is open.
    TourianOpen,
    /// Three of Tourian's rooms past the metroids have a version for this event, which nothing in
    /// the game makes happen.
    TourianRoomsUnset,
}

/// The kind of a boss, as the boss itself reports it when defeated. The same kind is a different
/// boss in each area.
///
/// Listed alphabetically; the order means nothing beyond keeping kinds in sorted sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Enum)]
pub enum BossKind {
    /// The area's main boss, such as Kraid in Brinstar or Ridley in Norfair.
    MainBoss,
    /// The area's lesser boss, such as Spore Spawn in Brinstar or Mother Brain in Tourian.
    MiniBoss,
    /// A torizo: Bomb Torizo in Crateria, Golden Torizo in Norfair.
    Torizo,
}

/// The id a room's population gives a place where an item lies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocationId(pub u16);

/// The id a room's population gives a door that stays open once opened.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DoorId(pub u16);

/// A save station's id within its area.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StationId(pub u16);

/// The way she came into a room. Entrances are numbered in a space of their own: an entrance is
/// never a `DoorId`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entrance(pub u16);

/// The last picture of a room's tileset: pictures are numbered `000` to `3FF` (blocks' notation).
pub const LAST_PICTURE: u16 = 0x3FF;

/// One of the room's tileset's pictures, by its index, `000` to `3FF`. Which picture a number is
/// belongs to the tileset; the models only set the number.
///
/// The HUD's pictures and her map's are numbered the same way, by the importer.
///
/// Its default, picture `000`, stands in for content not yet imported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Picture(u16);

impl Picture {
    /// The picture of this index, if the tileset has one (`000` to `3FF`).
    #[must_use]
    pub const fn new(index: u16) -> Option<Self> {
        if index <= LAST_PICTURE {
            Some(Self(index))
        } else {
            None
        }
    }

    /// Its index in the tileset.
    #[must_use]
    pub const fn index(self) -> u16 {
        self.0
    }
}

/// How a picture is flipped as a cell shows it. Its default is `None`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Flip {
    /// As it is.
    #[default]
    None,
    /// Flipped across: left for right.
    Across,
    /// Flipped down: top for bottom.
    Down,
    /// Flipped across and down.
    Both,
}

/// A look: the picture a cell shows, and how it is flipped (blocks' notation, `312:x`): a block
/// of the room, a cell of the HUD or of her map alike.
///
/// Its default, picture `000` unflipped, stands in for content not yet imported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Look {
    /// The tileset's picture.
    pub picture: Picture,
    /// How it is flipped.
    pub flip: Flip,
}

impl Look {
    /// The look that shows `picture` as it is, not flipped.
    #[must_use]
    pub const fn unflipped(picture: Picture) -> Self {
        Self {
            picture,
            flip: Flip::None,
        }
    }
}

/// The kind of one screen of the room, which decides what the camera's view may show of it; what
/// each kind lets the view show is the camera's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenKind {
    /// A closed screen.
    Closed,
    /// An open screen.
    Open,
    /// A short screen.
    Short,
}

/// A screen of the room: x its column, y its row, counted from the room's top-left.
pub type Screen = Pair<u16>;

/// A point of her body whose cell samus-movement reports as one she is inside (`TouchedCells`):
/// her top edge, her centre, her bottom edge. (The report itself goes bottom edge first.)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Point {
    /// Her top edge.
    Top,
    /// Her centre.
    Centre,
    /// Her bottom edge.
    Bottom,
}

/// What reaches a cell when her weapons or her grapple meet the room. Listed alphabetically;
/// the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum HitBy {
    /// A beam.
    Beam,
    /// A bomb's blast.
    Bomb,
    /// The grapple beam.
    Grapple,
    /// A missile.
    Missile,
    /// A power bomb's blast.
    PowerBomb,
    /// A super missile.
    SuperMissile,
}

/// How she moves, as the blocks she touches tell (her other domains decide it).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum Motion {
    /// At the speed booster's full speed.
    Dashing,
    /// Shinesparking.
    Shinesparking,
    /// Spinning in a screw attack (not every pose in which the screw attack hurts).
    ScrewAttacking,
}

/// A reaction's family: which rules start it. Listed alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum Family {
    /// A bomb block.
    Bomb,
    /// A crumble block.
    Crumble,
    /// A power bomb block.
    PowerBomb,
    /// A shot block.
    Shot,
    /// A speed block.
    Speed,
    /// A super missile block.
    SuperMissile,
}

/// The family whose blocks a cell marked `Nothing` is met as. Listed alphabetically.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum MetAs {
    /// Met as bomb blocks.
    Bomb,
    /// Met as crumble blocks.
    Crumble,
    /// Met as shot blocks.
    Shot,
}

/// One of a reaction's scripts. Listed alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum Script {
    /// What it does when it breaks.
    Broken,
    /// What a bomb that doesn't break it does: it shows itself.
    Revealed,
    /// What a bomb block does when her touch breaks it.
    Touched,
}
