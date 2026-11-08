//! What the HUD reads each frame (§1.3): the values other domains own, the buttons, and where she
//! is on her map.

use clean_types::{Item, MapCell, Pair};

use crate::map::AreaMap;
use crate::rules::Boss;

/// The count of frames the game has finished, mod 256, as this frame begins; a lag frame isn't
/// one (§1.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FrameCount(pub u8);

/// Her energy now and her capacity (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct Energy {
    /// Her energy now.
    pub now: u16,
    /// Her capacity.
    pub capacity: u16,
}

/// Whether the reserve tanks refill on their own.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReserveMode {
    /// They refill on their own (`auto`).
    Auto,
    /// She refills from them by hand.
    Manual,
}

/// Her reserve tanks: the energy in them and how they refill (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct Reserve {
    /// The energy in the reserve tanks.
    pub energy: u16,
    /// How they refill.
    pub mode: ReserveMode,
}

/// A kind of ammunition the HUD counts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ammo {
    /// Missiles: three digits.
    Missiles,
    /// Super missiles: two digits.
    SuperMissiles,
    /// Power bombs: two digits.
    PowerBombs,
}

/// The most missiles the spec defines the HUD on (§1.5).
const MOST_MISSILES: u16 = 999;
/// The most super missiles or power bombs the spec defines the HUD on (§1.5).
const MOST_OF_THE_OTHERS: u16 = 99;

impl Ammo {
    /// Every kind, in the HUD's order.
    pub const ALL: [Self; 3] = [Self::Missiles, Self::SuperMissiles, Self::PowerBombs];

    /// The item it is, when selected.
    #[must_use]
    pub const fn item(self) -> Item {
        match self {
            Self::Missiles => Item::Missiles,
            Self::SuperMissiles => Item::SuperMissiles,
            Self::PowerBombs => Item::PowerBombs,
        }
    }

    /// The largest count or capacity the HUD is defined on (§1.5): 999 missiles, 99 of the
    /// others.
    #[must_use]
    pub const fn most(self) -> u16 {
        match self {
            Self::Missiles => MOST_MISSILES,
            Self::SuperMissiles | Self::PowerBombs => MOST_OF_THE_OTHERS,
        }
    }
}

/// A count of one ammunition and how many she can carry (§1.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Stock {
    /// How many she has.
    pub count: u16,
    /// How many she can carry.
    pub capacity: u16,
}

/// Her missiles, super missiles and power bombs, each within the HUD's domain (§1.5).
#[derive(Clone, Copy, Debug)]
pub struct Ammunition {
    missiles: Stock,
    super_missiles: Stock,
    power_bombs: Stock,
}

impl Ammunition {
    /// The three stocks, or `None` if a count or capacity is past what the HUD is defined on
    /// (§1.5: 999 missiles, 99 super missiles or power bombs).
    #[must_use]
    pub fn new(missiles: Stock, super_missiles: Stock, power_bombs: Stock) -> Option<Self> {
        let ammunition = Self {
            missiles,
            super_missiles,
            power_bombs,
        };
        Ammo::ALL
            .iter()
            .all(|&ammo| {
                let stock = ammunition.stock(ammo);
                stock.count <= ammo.most() && stock.capacity <= ammo.most()
            })
            .then_some(ammunition)
    }

    /// The stock of one kind.
    #[must_use]
    pub const fn stock(&self, ammo: Ammo) -> Stock {
        match ammo {
            Ammo::Missiles => self.missiles,
            Ammo::SuperMissiles => self.super_missiles,
            Ammo::PowerBombs => self.power_bombs,
        }
    }
}

/// The items she has on that the HUD can select without a count (§1.3).
#[derive(Clone, Copy, Debug, Default)]
pub struct Equipped {
    /// She has the grapple beam on.
    pub grapple_beam: bool,
    /// She has the x-ray scope on.
    pub x_ray_scope: bool,
}

/// What other domains report about her that the status bar shows (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct Status {
    /// Her energy.
    pub energy: Energy,
    /// Her reserve tanks.
    pub reserve: Reserve,
    /// Her ammunition.
    pub ammunition: Ammunition,
    /// The items she has on.
    pub equipped: Equipped,
}

impl Status {
    /// Spec: CyclingItems. Whether Select can land on `item`: an ammunition with a count above 0,
    /// or an item she has on.
    #[must_use]
    pub const fn has(&self, item: Item) -> bool {
        match item {
            Item::Missiles => self.ammunition.missiles.count != 0,
            Item::SuperMissiles => self.ammunition.super_missiles.count != 0,
            Item::PowerBombs => self.ammunition.power_bombs.count != 0,
            Item::GrappleBeam => self.equipped.grapple_beam,
            Item::XRayScope => self.equipped.x_ray_scope,
        }
    }
}

/// Item select this frame (§1.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelectButton {
    /// It didn't go down this frame.
    Up,
    /// It went down this frame.
    Pressed,
}

/// Item cancel this frame (§1.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CancelButton {
    /// Not held.
    Up,
    /// It went down this frame.
    Pressed,
    /// Held from before.
    Held,
}

/// Why a switch makes no click (§1.3; spec: `SwitchClick`). Listed alphabetically; the order
/// means nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mute {
    /// The grapple beam is out.
    Grapple,
    /// She spin jumps.
    SpinJump,
    /// Time is stopped.
    TimeStopped,
    /// She wall jumps.
    WallJump,
}

/// The buttons and what lets them act (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct Controls {
    /// Item select.
    pub select: SelectButton,
    /// Item cancel.
    pub cancel: CancelButton,
    /// The player controls her and she doesn't face the screen.
    pub can_switch: bool,
    /// The weapons cancelled the selection after a shot this frame.
    pub shot_cancels: bool,
    /// Why a switch makes no click, or `None`.
    pub muted_by: Option<Mute>,
}

/// Her room on her map (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct Room {
    /// Its top-left cell on her map.
    pub cell: MapCell,
    /// Its size in 16 px blocks, across and down.
    pub blocks: Pair<u16>,
}

/// What the mini-map reads (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct MiniMapInputs<'map> {
    /// False while she is held at a station, drained, entering or leaving the gunship, or while
    /// the game's opening shows her.
    pub runs: bool,
    /// From when a boss turns it off until something turns it on.
    pub off: bool,
    /// Where she is: whole pixels in her room, after this frame's moves and before a pose change
    /// places her.
    pub position: Pair<u16>,
    /// Her room.
    pub room: Room,
    /// Her map.
    pub map: &'map AreaMap,
}

/// Everything the HUD reads in one frame of play (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct FrameInputs<'map> {
    /// The frame count as this frame begins.
    pub frame: FrameCount,
    /// What the status bar shows.
    pub status: Status,
    /// The buttons.
    pub controls: Controls,
    /// The item whose pickup this frame draws its icon, or `None`.
    pub collected: Option<Item>,
    /// The boss that turns the mini-map off this frame (Mother Brain's first form falling), or
    /// `None`.
    pub boss: Option<Boss>,
    /// What the mini-map reads.
    pub mini_map: MiniMapInputs<'map>,
}
