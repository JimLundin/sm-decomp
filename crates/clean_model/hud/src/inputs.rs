//! What the HUD reads each frame (§1.3): the values other domains own, the buttons, and where she
//! is on her map; and what the HUD knows of each kind of ammunition it counts.

use clean_types::Pair;

use crate::item::SelectableItem;
use crate::map::{HerMap, MapCell};
use crate::pictures::HudPos;
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

/// What the HUD knows of a kind of ammunition.
#[derive(Debug)]
pub struct AmmoKind {
    /// The item it is, when selected.
    pub item: SelectableItem,
    /// The largest count or capacity the HUD is defined on (§1.5).
    pub most: u16,
    /// The tiles of its digits, the highest first (§1.2); as many as its count's digits.
    pub digits: &'static [HudPos],
}

/// The kinds of ammunition, in the HUD's order: missiles in three digits, at most 999; super
/// missiles and power bombs in two, at most 99 (§1.2, §1.5).
pub const AMMO_KINDS: [AmmoKind; 3] = [
    AmmoKind {
        item: SelectableItem::Missiles,
        most: 999,
        digits: &[HudPos::at(10, 2), HudPos::at(11, 2), HudPos::at(12, 2)],
    },
    AmmoKind {
        item: SelectableItem::SuperMissiles,
        most: 99,
        digits: &[HudPos::at(14, 2), HudPos::at(15, 2)],
    },
    AmmoKind {
        item: SelectableItem::PowerBombs,
        most: 99,
        digits: &[HudPos::at(17, 2), HudPos::at(18, 2)],
    },
];

/// A count of one ammunition and how many she can carry (§1.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Stock {
    /// How many she has.
    pub count: u16,
    /// How many she can carry.
    pub capacity: u16,
}

/// Her stock of each kind of ammunition, in the order of `AMMO_KINDS`, each within the HUD's
/// domain (§1.5).
#[derive(Clone, Copy, Debug)]
pub struct Ammunition([Stock; AMMO_KINDS.len()]);

impl Ammunition {
    /// Her stocks, in the order of `AMMO_KINDS`, or `None` if a count or capacity is past what
    /// the HUD is defined on (§1.5).
    #[must_use]
    pub fn new(stocks: [Stock; AMMO_KINDS.len()]) -> Option<Self> {
        let within = |(kind, stock): (&AmmoKind, &Stock)| {
            stock.count <= kind.most && stock.capacity <= kind.most
        };
        AMMO_KINDS
            .iter()
            .zip(&stocks)
            .all(within)
            .then_some(Self(stocks))
    }

    /// Each kind with her stock of it, in the HUD's order.
    pub fn kinds(&self) -> impl Iterator<Item = (&'static AmmoKind, Stock)> {
        AMMO_KINDS.iter().zip(self.0)
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
    pub fn has(&self, item: SelectableItem) -> bool {
        match item {
            SelectableItem::GrappleBeam => self.equipped.grapple_beam,
            SelectableItem::XRayScope => self.equipped.x_ray_scope,
            SelectableItem::Missiles
            | SelectableItem::SuperMissiles
            | SelectableItem::PowerBombs => self
                .ammunition
                .kinds()
                .any(|(kind, stock)| kind.item == item && stock.count != 0),
        }
    }
}

/// Item select this frame (§1.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelectButton {
    /// It didn't go down this frame.
    NotPressed,
    /// It went down this frame.
    Pressed,
}

/// Item cancel this frame (§1.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CancelButton {
    /// Neither pressed this frame nor held from before.
    NotPressed,
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
    /// From when a boss turns it off until something turns it on: the one source of whether the
    /// mini-map is off, so a frame whose boss turns it off says so here too.
    pub off: bool,
    /// Where she is: whole pixels in her room, after this frame's moves and before a pose change
    /// places her.
    pub position: Pair<u16>,
    /// Her room.
    pub room: Room,
    /// Her map.
    pub map: &'map HerMap,
    /// She has collected her map.
    pub map_collected: bool,
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
    pub collected: Option<SelectableItem>,
    /// The boss that turns the mini-map off this frame (Mother Brain's first form falling), or
    /// `None`.
    pub boss: Option<Boss>,
    /// What the mini-map reads.
    pub mini_map: MiniMapInputs<'map>,
}
