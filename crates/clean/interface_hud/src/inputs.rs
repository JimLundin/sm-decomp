//! What the HUD reads each step (§1.3): in a frame of play, the values other domains own, the
//! buttons, what other domains drop before the status bar and after it, and where she is on her
//! map; outside play, the mini-map's turn-on, a new area and a drop.

use enum_map::EnumMap;
use types::Pair;

use crate::grid::Place;
use crate::item::{Ammo, SelectableItem};
use crate::map::{HerMap, MapCellSet};

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

/// §1.5: the largest count or capacity of `ammo` the HUD is defined on: missiles at most 999,
/// super missiles and power bombs at most 99.
const fn largest_allowed(ammo: Ammo) -> u16 {
    match ammo {
        Ammo::Missiles => 999,
        Ammo::SuperMissiles | Ammo::PowerBombs => 99,
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

/// Her stock of each ammunition, each within the HUD's domain (§1.5).
#[derive(Clone, Copy, Debug)]
pub struct Ammunition(EnumMap<Ammo, Stock>);

impl Ammunition {
    /// Her stocks, or `None` if a count or capacity is past what the HUD is defined on (§1.5).
    #[must_use]
    pub fn new(stocks: EnumMap<Ammo, Stock>) -> Option<Self> {
        stocks
            .iter()
            .all(|(ammo, stock)| stock.count.max(stock.capacity) <= largest_allowed(ammo))
            .then_some(Self(stocks))
    }

    /// Her stock of `ammo`.
    #[must_use]
    pub fn of(&self, ammo: Ammo) -> &Stock {
        &self.0[ammo]
    }

    /// Her stock of each ammunition, in the HUD's order.
    pub fn stocks(&self) -> impl Iterator<Item = (Ammo, &Stock)> {
        self.0.iter()
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
            SelectableItem::Ammo(ammo) => self.ammunition.of(ammo).count != 0,
            SelectableItem::GrappleBeam => self.equipped.grapple_beam,
            SelectableItem::XRayScope => self.equipped.x_ray_scope,
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

/// Why a switch makes no click (§1.3; spec: SwitchClick). Listed alphabetically; the order means
/// nothing.
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
    /// The player controls her, or a demo that plays by itself moves her, and she doesn't face
    /// the screen.
    pub can_switch: bool,
    /// Why a switch makes no click, or `None`.
    pub muted_by: Option<Mute>,
}

/// Her room on her map (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct Room {
    /// Its top-left cell on her map.
    pub cell: Place,
    /// Its size in 16 px blocks, across and down.
    pub blocks: Pair<u16>,
}

/// What the mini-map reads in a frame of play (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct MiniMapInputs<'map> {
    /// False while she is held at a station, drained, entering or leaving the gunship, dying, or
    /// while the game's opening shows her (the title screen's demos still run it).
    pub runs: bool,
    /// The mini-map turned on: an event of this step, not a state. True when something turned
    /// it on during this frame, ahead of the mini-map's turn (Crocomire's fall into the acid).
    /// A turn-on outside play is the step outside play's (`OutOfPlayInputs`). Whether the
    /// mini-map is on is the state's (`HudState::mini_map`).
    pub turned_on: bool,
    /// Where she is: whole pixels in her room, after this frame's moves and before a pose change
    /// places her.
    pub position: Pair<u16>,
    /// Her room.
    pub room: Room,
    /// Her map: its pictures and the cells it charts.
    pub map: &'map HerMap,
    /// She has collected her map, so the window shows the cells it charts.
    pub map_collected: bool,
}

/// The selection dropped (§1.3): what another domain drops at one point of a step, if anything.
///
/// When two drops arrive at one point, the selection's wins: the caller gives `Selection`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dropped {
    /// Nothing is dropped.
    Nothing,
    /// The selection: it becomes none, and with it the one-shot item.
    Selection,
    /// The one-shot item alone: the item stays selected, but isn't the one-shot item.
    OneShotItemAlone,
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
    /// The selection dropped before the status bar is drawn, so the bar shows the selection
    /// after it: the weapons' drop after a shot, Mother Brain's rainbow beam, an elevator
    /// starting with the game's drop-the-selection option on.
    pub dropped_before_status_bar: Dropped,
    /// The selection dropped after the status bar is drawn, so the bar shows the selection
    /// before it until the next frame of play: her death, on its first frame.
    pub dropped_after_status_bar: Dropped,
    /// The item whose pickup this frame draws its icon, or `None`.
    pub collected: Option<SelectableItem>,
    /// Mother Brain's first form falls this frame, her brain in the tank destroyed.
    pub mother_brain_falls: bool,
    /// What the mini-map reads.
    pub mini_map: MiniMapInputs<'map>,
}

/// A new area (§1.3): whether a step outside play leads into another area.
#[derive(Clone, Copy, Debug)]
pub enum NewArea<'cells> {
    /// She stays in the area she is in.
    SameArea,
    /// The step leads into another area: the explored cells Progress keeps for it.
    Entered(&'cells MapCellSet),
}

/// Everything the HUD reads in a step outside play (spec: OutOfPlay): a door's passage, a
/// load's other frames, a cinematic.
#[derive(Clone, Copy, Debug)]
pub struct OutOfPlayInputs<'cells> {
    /// The mini-map turned on in this step: a door's passage to the next room, a load, a new
    /// game or the demos' load.
    pub mini_map_turned_on: bool,
    /// Whether the step leads into another area.
    pub new_area: NewArea<'cells>,
    /// The selection dropped in this step: a door's passage with the game's drop-the-selection
    /// option on, the station exploding with her aboard.
    pub dropped: Dropped,
}
