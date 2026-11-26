//! Ordinary fight drawing intents. The renderer draws tail, then wings, before the layered
//! room pass. It draws the body in its existing layer and, when the baby and attached door
//! share an opportunity, draws the baby first.

use types::ceres_fight::WingBeat;
use types::{Pair, Px, Side};

use crate::fight_wings::FightWings;
use crate::flight_tail::{FlightTail, TAIL_SEGMENTS, TailAngle};

/// Spec: `FightParts`. Ridley's facing at the ordinary draw opportunity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FightFacing {
    /// Wings face left.
    Left,
    /// No wings are drawn.
    Front,
    /// Wings face right.
    Right,
}

/// Spec: `FightParts`. Which half of the shared four-run flash cycle is current.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlashHalf {
    /// Turns zero and one.
    First,
    /// Turns two and three.
    Second,
}

const HURT_BASE: u16 = 13;
const EVEN_ODD_BIT: u16 = 1;
const SHOT_PARITIES: usize = 2;
const MIN_HURT_FRAMES: u16 = 2;

/// Spec: `FightParts`. Actor-framework inputs before its regular timer spend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FightFlash {
    /// Hurt frames before the accepted shots on this run.
    pub remaining: u16,
    /// Number of accepted shots, applied in contact order.
    pub accepted_shots: usize,
    /// The half of the shared cycle the room layers will use on this run.
    pub half: FlashHalf,
}

impl FightFlash {
    /// An unhurt actor with no accepted shots.
    pub const UNHURT: Self = Self {
        remaining: 0,
        accepted_shots: 0,
        half: FlashHalf::First,
    };

    /// Timer write for the actor framework, before it spends the regular run.
    #[must_use]
    pub const fn refresh(self) -> Option<u16> {
        if self.accepted_shots == 0 {
            return None;
        }
        // Adding the odd base flips parity. Every pair of shots returns to the starting parity.
        let parity = self.remaining & EVEN_ODD_BIT;
        let last_parity = if self.accepted_shots.is_multiple_of(SHOT_PARITIES) {
            parity ^ EVEN_ODD_BIT
        } else {
            parity
        };
        Some(HURT_BASE.saturating_add(last_parity))
    }

    /// Select the colours after applying all accepted shots, before spending the timer.
    #[must_use]
    pub fn colours(self) -> FightColours {
        if self.refresh().unwrap_or(self.remaining) >= MIN_HURT_FRAMES
            && self.half == FlashHalf::Second
        {
            FightColours::Hurt
        } else {
            FightColours::Normal
        }
    }
}

/// Spec: `FightParts`. Colour selection shared by the wings and tail.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FightColours {
    /// Ordinary colours.
    #[default]
    Normal,
    /// Hurt colours for the current flash turn.
    Hurt,
}

/// Visual inputs at an ordinary fight turn. The encounter phase decides whether it draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FightPictureInputs {
    /// Facing for the parts, not a change to the existing body look.
    pub facing: FightFacing,
    /// If false, omit tail pieces even if the tail moves.
    pub tail_visible: bool,
    /// Actor framework flash and shot inputs.
    pub flash: FightFlash,
}

impl FightPictureInputs {
    /// The ordinary left-facing, unhurt look with no special tail hiding.
    pub const LEFT_UNHURT: Self = Self {
        facing: FightFacing::Left,
        tail_visible: true,
        flash: FightFlash::UNHURT,
    };
}

/// Spec: `FightParts`. The wing look and its whole-pixel pre-hop anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WingIntent {
    /// Only a side-facing look has wings.
    pub facing: Side,
    /// The clock's current beat.
    pub beat: WingBeat,
    /// Whole room pixels, before any hop.
    pub anchor: Pair<Px>,
}

const ANGLES_PER_LOOK: u8 = 16;
const ROUND_TIP: u16 = 8;
const RESTING_TIP: u8 = 11;

/// Spec: `FightParts`. The tip's direction in sixteenths of a turn, from down toward right.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TailTipLook(u8);

impl TailTipLook {
    /// Numbered from zero through fifteen.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Spec: `FightParts`. Tail pieces in drawing order, from tip back to the base.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TailIntent {
    /// Whole room pixels, tip first.
    pub anchors: [Pair<Px>; TAIL_SEGMENTS],
    /// The tip's look; the links do not read it.
    pub tip: TailTipLook,
}

impl TailIntent {
    /// All pieces of the resting tail share one place.
    #[must_use]
    pub const fn resting(place: Pair<Px>) -> Self {
        Self {
            anchors: [place; TAIL_SEGMENTS],
            tip: TailTipLook(RESTING_TIP),
        }
    }

    /// Draw the tail at the places left by this run's swing.
    #[must_use]
    pub fn in_flight(tail: &FlightTail) -> Self {
        let [.., penultimate, tip] = &tail.segments;
        Self::linked(tail.places(), penultimate.angle, tip.angle)
    }

    /// Reverse the base-to-tip places and round the sum of the last two link angles.
    #[must_use]
    pub const fn linked(
        mut places: [Pair<Px>; TAIL_SEGMENTS],
        penultimate: TailAngle,
        last: TailAngle,
    ) -> Self {
        places.reverse();
        let sum = penultimate.0.wrapping_add(last.0).wrapping_add(ROUND_TIP);
        Self {
            anchors: places,
            tip: TailTipLook(sum.to_le_bytes()[0] / ANGLES_PER_LOOK),
        }
    }
}

/// Spec: `FightParts`. Ordinary drawing, tail before wings, with one colour selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FightParts {
    /// The visible tail, if any.
    pub tail: Option<TailIntent>,
    /// The wings, if side-facing.
    pub wing: Option<WingIntent>,
    /// Current shared part colours.
    pub colours: FightColours,
}

impl FightParts {
    /// Offer parts at their ordinary pre-hop drawing opportunity.
    #[must_use]
    pub fn at_anchor(
        wings: FightWings,
        anchor: Pair<Px>,
        tail: Option<TailIntent>,
        picture: FightPictureInputs,
    ) -> Self {
        let side = match picture.facing {
            FightFacing::Left => Some(Side::Left),
            FightFacing::Front => None,
            FightFacing::Right => Some(Side::Right),
        };
        Self {
            tail: tail.filter(|_| picture.tail_visible),
            wing: side.map(|facing| WingIntent {
                facing,
                beat: wings.beat,
                anchor,
            }),
            colours: picture.flash.colours(),
        }
    }
}

/// Spec: `FightPictureOrder`. Which part of the encounter owns the room picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FightPictureStage {
    /// Ordinary animation has not begun.
    BeforeOrdinaryAnimation,
    /// Ordinary animation is active, including the retreat before the flyby.
    Fighting,
    /// The getaway has begun or has finished.
    Getaway,
}

/// The opportunity for the baby and attached door relative to the layered room pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomLayerOrder {
    /// Before the room layers.
    Before,
    /// After all the room layers.
    After,
}

/// Spec: `FightPictureOrder`. Scheduling only: draw the existing baby look before the
/// attached door when both are present. Their owners still supply the actual drawables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FightPictureOrder {
    /// Shared baby and attached-door opportunity.
    pub opportunity: RoomLayerOrder,
    /// Whether the ordinary baby is still available, if its owner has a look to draw.
    pub baby: bool,
    /// Whether the door's owner has it attached.
    pub door: bool,
}

impl FightPictureStage {
    /// Choose the room order without taking ownership of the door's attachment.
    #[must_use]
    pub const fn order(self, door_attached: bool) -> FightPictureOrder {
        let (opportunity, baby) = match self {
            Self::BeforeOrdinaryAnimation => (RoomLayerOrder::Before, true),
            Self::Fighting => (RoomLayerOrder::After, true),
            Self::Getaway => (RoomLayerOrder::After, false),
        };
        FightPictureOrder {
            opportunity,
            baby,
            door: door_attached,
        }
    }
}
